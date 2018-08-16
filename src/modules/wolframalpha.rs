use bot::Bot;
use discord::model::Message;
use hyper::client::Client;
use module;
use std::{collections::hash_map::HashMap, env, error::Error};
use url::Url;
use xml::{self, reader::XmlEvent};

pub struct Module<'a> {
    commands: HashMap<u32, &'a [&'a str]>,
}

lazy_static! {
    static ref WOLFRAMALPHA_API_BASE: Url =
        Url::parse("http://api.wolframalpha.com/v2/query").unwrap();
    static ref WOLFRAMALPHA_CLIENT_ID: Result<String, String> =
        env::var("YALTER_BOT_WOLFRAMALPHA_APPID").map_err(|_| {
            "Please set the YALTER_BOT_WOLFRAMALPHA_APPID environment variable".to_string()
        });
}

enum Commands {
    WA = 0,
}

struct Pod {
    image_url: Option<String>,
    plaintext: String,
}

#[derive(PartialEq)]
enum CurrentPod {
    InputInterpretation,
    Results,
}

impl<'a> module::Module for Module<'a> {
    fn new() -> Result<Box<module::Module>, String> {
        static WA: [&'static str; 2] = ["wolphramalpha", "wa"];
        let mut map: HashMap<u32, &[&str]> = HashMap::new();
        map.insert(Commands::WA as u32, &WA);
        WOLFRAMALPHA_CLIENT_ID
            .as_ref()
            .map_err(|s| s.clone())
            .and(Ok(Box::new(Module { commands: map })))
    }

    fn name(&self) -> &'static str {
        "Wolfram!Alpha"
    }

    fn description(&self) -> &'static str {
        "A command for querying the Wolfram!Alpha service."
    }

    fn commands(&self) -> &HashMap<u32, &[&str]> {
        &self.commands
    }

    fn command_description(&self, _: u32) -> &'static str {
        "Queries the Wolfram!Alpha service."
    }

    fn command_help_message(&self, _: u32) -> &'static str {
        "`!wa <input>` - Queries Wolfram!Alpha with the given input and returns the result. For example, `!wa int sin x / x dx, 0 < x < +inf`."
    }

    fn handle(&self, bot: &Bot, message: &Message, _id: u32, text: &str) {
        bot.broadcast_typing(message.channel_id); // This command takes a few seconds to process.

        let mut url = WOLFRAMALPHA_API_BASE.clone();
        url.query_pairs_mut()
            .append_pair("appid", WOLFRAMALPHA_CLIENT_ID.as_ref().unwrap())
            .append_pair("input", text);

        println!("URL: {}", url.as_str());

        let client = Client::new();
        match client.get(url.as_str()).send() {
            Ok(result) => {
                let mut input_interpretation: Option<Pod> = None;
                let mut results: Option<Pod> = None;
                let mut state = CurrentPod::InputInterpretation;

                let mut inside_plaintext = false;
                let mut error = false;

                let reader = xml::reader::EventReader::new(result);
                'xml_loop: for event in reader {
                    match event {
                        Ok(XmlEvent::StartElement {
                            name, attributes, ..
                        }) => match name.local_name.as_ref() {
                            "queryresult" => {
                                for attr in attributes {
                                    if attr.name.local_name == "numpods" {
                                        match attr.value.parse::<u8>() {
                                            Ok(0) | Err(_) => {
                                                bot.send(message.channel_id, "Wolfram!Alpha couldn't understand your input. :/");
                                                error = true;
                                                break 'xml_loop;
                                            }

                                            _ => {}
                                        }
                                    }
                                }
                            }

                            "pod" => {
                                for attr in attributes {
                                    if (attr.name.local_name == "title"
                                        && (attr.value != "Input"
                                            && attr.value != "Input interpretation"))
                                        || (attr.name.local_name == "id" && attr.value != "Input")
                                    {
                                        state = CurrentPod::Results
                                    }
                                }
                            }

                            "img" => {
                                for attr in attributes {
                                    if attr.name.local_name == "src" {
                                        let pod = match state {
                                            CurrentPod::InputInterpretation => {
                                                &mut input_interpretation
                                            }
                                            CurrentPod::Results => &mut results,
                                        };

                                        match pod {
                                            Some(ref mut x) => {
                                                x.image_url = Some(attr.value.clone())
                                            }
                                            None => {
                                                *pod = Some(Pod {
                                                    image_url: Some(attr.value.clone()),
                                                    plaintext: "".to_string(),
                                                })
                                            }
                                        }
                                    }
                                }
                            }

                            "plaintext" => {
                                inside_plaintext = true;
                            }

                            _ => {}
                        },

                        Ok(XmlEvent::Characters(string)) => {
                            if inside_plaintext {
                                let pod = match state {
                                    CurrentPod::InputInterpretation => &mut input_interpretation,
                                    CurrentPod::Results => &mut results,
                                };

                                match pod {
                                    Some(ref mut x) => x.plaintext = string.clone(),
                                    None => {
                                        *pod = Some(Pod {
                                            image_url: None,
                                            plaintext: string.clone(),
                                        })
                                    }
                                }

                                if state == CurrentPod::Results {
                                    break 'xml_loop;
                                }
                            }
                        }

                        Ok(XmlEvent::EndDocument) | Err(_) => {
                            break;
                        }

                        _ => {}
                    }
                }

                if let Some(pod) = input_interpretation {
                    let mut text = "Input interpretation:".to_string();
                    if !pod.plaintext.is_empty() {
                        text.push_str(&format!("\n```\n{}\n```", pod.plaintext));
                    }

                    if let Some(img) = pod.image_url {
                        match client.get(&img).send() {
                            Ok(result) => {
                                bot.send_file(
                                    message.channel_id,
                                    &text,
                                    result,
                                    "input_interpretation.gif",
                                );
                            }

                            Err(err) => {
                                if pod.plaintext.is_empty() {
                                    text = "".to_string();
                                }

                                if !text.is_empty() {
                                    text.push_str("\n\n");
                                }
                                text.push_str(&format!("Something's broken. :/ (Couldn't get the resulting image returned by the API: {})", err.description()));

                                bot.send(message.channel_id, &text);
                            }
                        }
                    }
                }

                if let Some(pod) = results {
                    let mut text = "Result:".to_string();
                    if !pod.plaintext.is_empty() {
                        text.push_str(&format!("\n```\n{}\n```", pod.plaintext));
                    }

                    if let Some(img) = pod.image_url {
                        match client.get(&img).send() {
                            Ok(result) => {
                                bot.send_file(message.channel_id, &text, result, "result.gif");
                            }

                            Err(err) => {
                                if pod.plaintext.is_empty() {
                                    text = "".to_string();
                                }

                                if !text.is_empty() {
                                    text.push_str("\n\n");
                                }
                                text.push_str(&format!("Something's broken. :/ (Couldn't get the resulting image returned by the API: {})", err.description()));

                                bot.send(message.channel_id, &text);
                            }
                        }
                    }
                } else if !error {
                    bot.send(message.channel_id, "Wolfram!Alpha didn't return a result pod. This probably means that the standard computation time exceeded.");
                }
            }

            Err(err) => {
                bot.send(
                    message.channel_id,
                    &format!(
                        "Couldn't communicate with http://api.wolframalpha.com. :( ({})",
                        err.description()
                    ),
                );
            }
        }
    }
}
