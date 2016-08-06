use bot::Bot;
use discord::ChannelRef;
use discord::model::Message;
use module;
use rand;
use rand::distributions::{IndependentSample, Range};
use regex::Regex;
use std::char;
use std::collections::hash_map::HashMap;

pub struct Module<'a> {
	commands: HashMap<u32, &'a [&'a str]>
}

lazy_static! {
	static ref TEMPERATURE_REGEX: Regex = Regex::new(r"\s*([+-]?[0-9]+(\.[0-9]*)?)\s*([CcFf]).*").unwrap();
	static ref ROLL_REGEX: Regex = Regex::new(r"\s*(([0-9]+)(\s|$))?.*").unwrap();
}

enum Commands {
	Fraktur = 0,
	Temperature = 1,
	Roll = 2,
	Pick = 3,
	Info = 4
}

impl<'a> module::Module for Module<'a> {
	fn new() -> Self {
		let mut map: HashMap<u32, &[&str]> = HashMap::new();
		static FRAKTUR: [&'static str; 1] = [ "fraktur" ];
		map.insert(Commands::Fraktur as u32, &FRAKTUR);
		static TEMPERATURE: [&'static str; 2] = [ "temperature", "temp" ];
		map.insert(Commands::Temperature as u32, &TEMPERATURE);
		static ROLL: [&'static str; 1] = [ "roll" ];
		map.insert(Commands::Roll as u32, &ROLL);
		static PICK: [&'static str; 2] = [ "pick", "choose" ];
		map.insert(Commands::Pick as u32, &PICK);
		static INFO: [&'static str; 2] = [ "information", "info" ];
		map.insert(Commands::Info as u32, &INFO);
		Module { commands: map }
	}

	fn name(&self) -> &'static str {
		"Fun"
	}

	fn description(&self) -> &'static str {
		"Various random commands."
	}

	fn commands(&self) -> &HashMap<u32, &[&str]> {
		&self.commands
	}

	fn command_description(&self, id: u32) -> &'static str {
		match id {
			x if x == Commands::Fraktur as u32 =>
				"Prints the given text in fraktur (gothic math symbols).",
			x if x == Commands::Temperature as u32 =>
				"Converts the temperature between Celsius and Fahrenheit.",
			x if x == Commands::Roll as u32 =>
				"Prints a random number.",
			x if x == Commands::Pick as u32 =>
				"Randomly picks one of the given options.",
			x if x == Commands::Info as u32 =>
				"Prints out some information about the server.",
			_ => panic!("Fun::command_description - invalid id.")
		}
	}

	fn command_help_message(&self, id: u32) -> &'static str {
		match id {
			x if x == Commands::Fraktur as u32 =>
				"`!fraktur <text>` - Prints the given text in fraktur (gothic math symbols). Note that there are no regular versions of letters 'C', 'H', 'I', 'R', 'Z'; those are replaced with their bold versions.",
			x if x == Commands::Temperature as u32 =>
				"`!temperature <number> <C or F>` - Converts the temperature into another scale. For example, `!temp 5C` outputs 41.",
			x if x == Commands::Roll as u32 =>
				"`!roll [high]` - Prints a random number between 0 and 99, or between 0 and high - 1, inclusive.",
			x if x == Commands::Pick as u32 =>
				"`!pick something;something else[;third option[;...]]` - Randomly picks one of the given options.",
			x if x == Commands::Info as u32 =>
				"`!information` - Prints out some information about the server.",
			_ => panic!("Fun::command_help_message - invalid id.")
		}
	}

	fn handle(&self, bot: &Bot, message: &Message, id: u32, text: &str) {
		match id {
			x if x == Commands::Fraktur as u32 =>
				self.handle_fraktur(bot, message, text),
			x if x == Commands::Temperature as u32 =>
				self.handle_temperature(bot, message, text),
			x if x == Commands::Roll as u32 =>
				self.handle_roll(bot, message, text),
			x if x == Commands::Pick as u32 =>
				self.handle_pick(bot, message, text),
			x if x == Commands::Info as u32 =>
				self.handle_info(bot, message, text),
			_ => panic!("Fun::handle - invalid id.")
		}
	}
}

impl<'a> Module<'a> {
	fn handle_fraktur(&self, bot: &Bot, message: &Message, text: &str) {
		let reply = text.chars().map(frakturize).collect::<String>();
		bot.send(&message.channel_id, &reply);
	}

	fn handle_temperature(&self, bot: &Bot, message: &Message, text: &str) {
		if let Some(caps) = TEMPERATURE_REGEX.captures(text) {
			let value = caps.at(1).unwrap().parse::<f32>().unwrap();
			let letter = caps.at(3).unwrap().chars().next().unwrap();

			let converted_value = match letter {
				'C' | 'c' => 9f32 * value / 5f32 + 32f32,
				'F' | 'f' => 5f32 * (value - 32f32) / 9f32,
				_ => panic!("Regex error in Fun::handle_temperature.")
			};

			let converted_letter = match letter {
				'C' | 'c' => 'F',
				'F' | 'f' => 'C',
				_ => panic!("Regex error in Fun::handle_temperature.")
			};

			bot.send(
				&message.channel_id,
				&format!(
					"{:.2}°{} is **{:.2}**°{}.",
					value,
					letter.to_uppercase().next().unwrap(),
					converted_value,
					converted_letter
				)
			);
		} else {
			bot.send(
				&message.channel_id,
				<Module as module::Module>::command_help_message(&self, Commands::Temperature as u32)
			);
		}
	}

	fn handle_roll(&self, bot: &Bot, message: &Message, text: &str) {
		let caps = ROLL_REGEX.captures(text).unwrap();
		let max = caps.at(2)
		              .and_then(|x| x.parse::<u64>().ok())
		              .map(|x| if x == 0 { 100 } else { x })
		              .unwrap_or(100);

		let mut rng = rand::thread_rng();
		let number = Range::new(0, max).ind_sample(&mut rng);

		bot.send(
			&message.channel_id,
			&format!("{} rolled **{}**!", message.author.mention(), number)
		);
	}

	fn handle_pick(&self, bot: &Bot, message: &Message, text: &str) {
		let options: Vec<&str> = text.split(';').filter(|x| x.len() > 0).collect();

		if options.len() < 2 {
			bot.send(
				&message.channel_id,
				<Module as module::Module>::command_help_message(&self, Commands::Pick as u32)
			);
		} else {
			let mut rng = rand::thread_rng();
			let index = Range::new(0, options.len()).ind_sample(&mut rng);

			bot.send(
				&message.channel_id,
				&format!("{}: I pick {}!", message.author.mention(), options[index])
			);
		}
	}

	fn handle_info(&self, bot: &Bot, message: &Message, _text: &str) {
		match bot.get_state().read().unwrap().find_channel(&message.channel_id) {
			Some(ChannelRef::Private(channel)) => {
				bot.send(&message.channel_id, &format!("```{:#?}```", channel));
			},

			Some(ChannelRef::Public(server, channel)) => {
				let mut buf = format!(
					"```Server ID: {},\n\
					    Owner ID: {},\n\
					    Member count: {},\n\
					    Icon: {},\n\
					    Roles:",
					server.id.0,
					server.owner_id.0,
					server.member_count,
					if let Some(ref icon) = server.icon { &icon } else { "N/A" }
				);

				if server.roles.len() == 0 {
					buf.push_str(" N/A");
				} else {
					for role in &server.roles {
						buf.push_str(&format!("\n- {} '{}'", role.id.0, role.name));
					}
				}

				buf.push_str(&format!("\n\nChannel ID: {}```", channel.id.0));

				bot.send(&message.channel_id, &buf);
			},

			None => {
				bot.send(&message.channel_id, "Huh, I couldn't get this channel's info for some reason. Try again I guess?");
			}
		}
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
