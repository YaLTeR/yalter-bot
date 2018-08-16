use bot::Bot;
use discord::model::Message;
use hldemo;
use hyper::Client;
use module;
use std::collections::hash_map::HashMap;
use std::io::Read;

pub struct Module<'a> {
    commands: HashMap<u32, &'a [&'a str]>,
}

// enum Commands {
// }

impl<'a> module::Module for Module<'a> {
    fn new() -> Result<Box<module::Module>, String> {
        let map: HashMap<u32, &[&str]> = HashMap::new();
        Ok(Box::new(Module { commands: map }))
    }

    fn name(&self) -> &'static str {
        "Demos"
    }

    fn description(&self) -> &'static str {
        "Says information about uploaded demos."
    }

    fn commands(&self) -> &HashMap<u32, &[&str]> {
        &self.commands
    }

    fn command_description(&self, _: u32) -> &'static str {
        unreachable!()
    }

    fn command_help_message(&self, _: u32) -> &'static str {
        unreachable!()
    }

    fn handle(&self, _bot: &Bot, _message: &Message, _id: u32, _text: &str) {
        unreachable!()
    }

    fn handle_attachment(&self, bot: &Bot, message: &Message) {
        for attachment in message
            .attachments
            .iter()
            .filter(|x| x.filename.ends_with(".dem"))
        {
            match process_demo_url(&attachment.url) {
                Ok(string) => bot.send(message.channel_id, &string),
                Err(err) => println!("Demos::handle_attachment error: {}", err),
            }
        }
    }
}

fn process_demo_url(url: &str) -> Result<String, String> {
    let client = Client::new();
    let mut res = client
        .get(url)
        .send()
        .map_err(|x| format!("network error on sending: {}", x))?;

    let mut bytes = Vec::new();
    res.read_to_end(&mut bytes)
        .map_err(|x| format!("network error on reading: {}", x))?;

    let demo = hldemo::Demo::parse_without_frames(&bytes)
        .map_err(|x| format!("error parsing demo: {}", x))?;
    let time = demo
        .directory
        .entries
        .iter()
        .filter(|e| e.entry_type != 0)
        .fold(0f32, |acc, e| acc + e.track_time);

    let mut result = "```\n".to_string();

    result.push_str(&format!(
        "Game: {}\n",
        String::from_utf8_lossy(demo.header.game_dir.split(|&x| x == 0).next().unwrap())
    ));
    result.push_str(&format!(
        "Map:  {}\n",
        String::from_utf8_lossy(demo.header.map_name.split(|&x| x == 0).next().unwrap())
    ));
    result.push_str(&format!("Time: {:.3}s\n", time));

    result.push_str("```");

    Ok(result)
}
