use bot::Bot;
use discord::model::Message;
use module;
use rand::{self, Rng};
use std::collections::hash_map::HashMap;

pub struct Module<'a> {
    commands: HashMap<u32, &'a [&'a str]>,
}

enum Commands {
    Hello = 0,
}

impl<'a> module::Module for Module<'a> {
    fn new() -> Result<Box<module::Module>, String> {
        static HELLO: [&'static str; 2] = ["hello", "hi"];
        let mut map: HashMap<u32, &[&str]> = HashMap::new();
        map.insert(Commands::Hello as u32, &HELLO);
        Ok(Box::new(Module { commands: map }))
    }

    fn name(&self) -> &'static str {
        "Hello"
    }

    fn description(&self) -> &'static str {
        "Provides the !hello command."
    }

    fn commands(&self) -> &HashMap<u32, &[&str]> {
        &self.commands
    }

    fn command_description(&self, _: u32) -> &'static str {
        "Prints a greeting message."
    }

    fn command_help_message(&self, _: u32) -> &'static str {
        "`!hello` - Prints a greeting message."
    }

    fn handle(&self, bot: &Bot, message: &Message, _id: u32, _text: &str) {
        let emojis: [&'static str; 22] = ["👌", "👌🏻", "👌🏼", "👌🏽", "👌🏾",
                                          "👌🏿", "👍", "👍🏻", "👍🏼", "👍🏽",
                                          "👍🏾", "👍🏿", "🌝", "😄", "🔥", "💯",
                                          "🆒", "🚽", "🚾", "❤", "⚠", "✅"];

        let emoji = rand::thread_rng().choose(&emojis).unwrap();
        bot.send(message.channel_id,
                 &format!("Hi, {}! {}", message.author.mention(), emoji));
    }
}
