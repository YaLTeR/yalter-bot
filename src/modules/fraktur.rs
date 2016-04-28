use bot::Bot;
use discord::model::Message;
use module;
use std::char;
use std::collections::hash_map::HashMap;

pub struct Module<'a> {
	commands: HashMap<u32, &'a [&'a str]>
}

enum Commands {
	Fraktur = 0
}

impl<'a> module::Module for Module<'a> {
	fn new() -> Self {
		static FRAKTUR: [&'static str; 1] = [ "fraktur" ];
		let mut map: HashMap<u32, &[&str]> = HashMap::new();
		map.insert(Commands::Fraktur as u32, &FRAKTUR);
		Module { commands: map }
	}

	fn name(&self) -> &'static str {
		"Fraktur"
	}

	fn description(&self) -> &'static str {
		"Provides the !fraktur command."
	}

	fn commands(&self) -> &HashMap<u32, &[&str]> {
		&self.commands
	}

	fn command_description(&self, _: u32) -> &'static str {
		"Prints the given text in fraktur (gothic math symbols)."
	}

	fn command_help_message(&self, _: u32) -> &'static str {
		"`!fraktur <text>` - Prints the given text in fraktur (gothic math symbols). Note that there are no regular versions of letters 'C', 'H', 'I', 'R', 'Z'; those are replaced with their bold versions."
	}

	fn handle(&self, bot: &Bot, message: &Message, _id: u32, text: &str) {
		let reply = text.chars().map(frakturize).collect::<String>();
		bot.send(&message.channel_id, &reply);
	}
}

fn frakturize(c: char) -> char {
	match c {
		'a'...'z' => char::from_u32(('𝔞' as u32) - ('a' as u32) + (c as u32)).unwrap(),
		// Those ones are absent from non-bold apparently
		'C' | 'H' | 'I' | 'R' | 'Z' => char::from_u32(('𝕬' as u32) - ('A' as u32) + (c as u32)).unwrap(),
		'A'...'Z' => char::from_u32(('𝔄' as u32) - ('A' as u32) + (c as u32)).unwrap(),
		_ => c
	}
}
