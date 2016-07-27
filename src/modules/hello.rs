use bot::Bot;
use discord::model::Message;
use module;
use rand;
use rand::distributions::{IndependentSample, Range};
use std::collections::hash_map::HashMap;

pub struct Module<'a> {
	commands: HashMap<u32, &'a [&'a str]>
}

enum Commands {
	Hello = 0
}

impl<'a> module::Module for Module<'a> {
	fn new() -> Self {
		static HELLO: [&'static str; 1] = [ "hello" ];
		let mut map: HashMap<u32, &[&str]> = HashMap::new();
		map.insert(Commands::Hello as u32, &HELLO);
		Module { commands: map }
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
		let emojis: [&'static str; 22] = [
			"👌", "👌🏻", "👌🏼", "👌🏽", "👌🏾", "👌🏿",
			"👍", "👍🏻", "👍🏼", "👍🏽", "👍🏾", "👍🏿",
			"🌝", "😄", "🔥", "💯", "🆒", "🚽", "🚾", "❤", "⚠", "✅"
		];

		let mut rng = rand::thread_rng();
		let index = Range::new(0, emojis.len()).ind_sample(&mut rng);

		bot.send(&message.channel_id, &format!("Hi, {}! {}", message.author.mention(), emojis[index]));
	}
}
