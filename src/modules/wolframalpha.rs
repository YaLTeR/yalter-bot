use bot::Bot;
use discord::model::Message;
use failure::{self, ResultExt};
use hyper::client::Client;
use module;
use serde::{Deserialize, Deserializer};
use serde_xml_rs::deserialize;
use std::{collections::hash_map::HashMap, env, error::Error};
use url::Url;

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

fn parse_bool<'de, D>(d: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    let s = String::deserialize(d)?;
    match &s[..] {
        "true" => Ok(true),
        "false" => Ok(false),
        other => Err(D::Error::custom(format!(
            "got {}, but expected `true` or `false`",
            other
        ))),
    }
}

#[derive(Deserialize)]
#[serde(rename = "queryresult")]
struct QueryResult {
    #[serde(deserialize_with = "parse_bool")]
    success: bool,
    #[serde(deserialize_with = "parse_bool")]
    error: bool,

    #[serde(rename = "pod", default)]
    pods: Vec<Pod>,

    #[serde(rename = "didyoumeans")]
    did_you_means: Option<DidYouMeans>,
}

#[derive(Deserialize)]
#[serde(rename = "pod")]
struct Pod {
    title: String,

    #[serde(rename = "subpod", default)]
    subpods: Vec<SubPod>,
}

#[derive(Deserialize)]
#[serde(rename = "subpod")]
struct SubPod {
    #[serde(rename = "img")]
    image: Option<Img>,
    plaintext: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename = "img")]
struct Img {
    src: String,
}

#[derive(Deserialize)]
#[serde(rename = "didyoumeans")]
struct DidYouMeans {
    #[serde(rename = "didyoumean", default)]
    did_you_means: Vec<DidYouMean>,
}

#[derive(Deserialize)]
#[serde(rename = "didyoumean")]
struct DidYouMean {
    #[serde(rename = "$value")]
    contents: String,
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
        if text.is_empty() {
            bot.send(
                message.channel_id,
                self.command_help_message(Commands::WA as u32),
            );
            return;
        }

        bot.broadcast_typing(message.channel_id); // This command takes a few seconds to process.

        if let Err(err) = self.handle_wa(bot, message, text) {
            let mut buf = format!("{}", err);

            for cause in err.iter_causes() {
                buf.push_str(&format!("\nCaused by: {}", cause));
            }

            bot.send(message.channel_id, &buf);
        }
    }
}

impl<'a> Module<'a> {
    fn handle_wa(&self, bot: &Bot, message: &Message, text: &str) -> Result<(), failure::Error> {
        let mut url = WOLFRAMALPHA_API_BASE.clone();
        url.query_pairs_mut()
            .append_pair("appid", WOLFRAMALPHA_CLIENT_ID.as_ref().unwrap())
            .append_pair("input", text);

        println!("URL: {}", url.as_str());

        let client = Client::new();
        let response = client
            .get(url.as_str())
            .send()
            .context("Couldn't communicate with http://api.wolframalpha.com. :(")?;
        let result: QueryResult = deserialize(response)
            .context("Couldn't parse Wolfram!Alpha's response, call YaLTeR!")?;

        // TODO: this returns an <error> tag too, which gets clashed with the error field.
        ensure!(!result.error, "Invalid request, call YaLTeR!");

        if !result.success {
            let mut text = "Wolphram!Alpha couldn't understand your input. :/".to_owned();

            if let Some(did_you_means) = result.did_you_means {
                let did_you_means = did_you_means.did_you_means;

                if did_you_means.len() == 1 {
                    text.push_str(&format!(
                        "\n\nDid you mean `{}`?",
                        did_you_means[0].contents
                    ));
                } else if did_you_means.len() > 1 {
                    text.push_str("\n\nDid you mean:");

                    for dym in did_you_means {
                        text.push_str(&format!("\n• `{}`", dym.contents));
                    }
                }
            }

            bail!(text);
        }

        let send_pod_contents = |pod: &Pod, text: &str, image_filename| {
            let mut text = text.to_owned();

            if let Some(subpod) = pod.subpods.iter().find(|s| {
                !s.plaintext
                    .as_ref()
                    .map(String::as_ref)
                    .unwrap_or("")
                    .is_empty()
            }) {
                text.push_str(&format!(
                    "\n```\n{}\n```",
                    subpod.plaintext.as_ref().unwrap()
                ));
            }

            if let Some(subpod) = pod.subpods.iter().find(|s| s.image.is_some()) {
                match client.get(&subpod.image.as_ref().unwrap().src).send() {
                    Ok(result) => {
                        bot.send_file(message.channel_id, &text, result, image_filename);
                    }
                    Err(err) => {
                        if !text.is_empty() {
                            text.push_str("\n\n");
                        }

                        text.push_str(&format!(
                            "Something's broken. :/ \
                             (Couldn't get the resulting image returned by the API: {})",
                            err.description()
                        ));

                        bot.send(message.channel_id, &text);
                    }
                }
            }
        };

        let is_input_interpretation =
            |pod: &Pod| pod.title == "Input" || pod.title == "Input interpretation";

        if let Some(input_interpretation) =
            result.pods.iter().find(|pod| is_input_interpretation(pod))
        {
            send_pod_contents(
                input_interpretation,
                "Input interpretation:",
                "input_interpretation.gif",
            );
        }

        if let Some(results) = result.pods.iter().find(|pod| !is_input_interpretation(pod)) {
            send_pod_contents(results, "Result:", "result.gif");
        } else {
            bot.send(
                message.channel_id,
                "Wolfram!Alpha didn't return a result pod. \
                 This probably means that the standard computation time exceeded.",
            );
        }

        Ok(())
    }
}
