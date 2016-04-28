extern crate hyper;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate serde_json;
extern crate url;
extern crate xml;

use std::fs::File;
use std::io;
use std::io::Read;

extern crate discord;
use discord::{Discord, State};
use discord::model::*;

mod module;
use module::Module;

mod bot;
use bot::Bot;

mod modules {
	pub mod hello;
	pub mod modules;
	pub mod fraktur;
	pub mod speedruncom;
	pub mod wolframalpha;
}

fn read_file(filename: &str) -> Result<String, io::Error> {
	let mut f = try!(File::open(filename));
	let mut s = String::new();

	match f.read_to_string(&mut s) {
		Ok(_) => Ok(s),
		Err(e) => Err(e)
	}
}

fn parse_command(message: &str) -> Option<(&str, &str)> {
	// Commands must start with a ! and are at least one symbol long
	// (excluding the exclamation mark).
	if !message.starts_with('!') || message.len() == 1 {
		return None;
	}

	// Chop off the !.
	let message = message.split_at(1).1;

	// Separate the command from the arguments.
	match message.find(char::is_whitespace) {
		Some(pos) => {
			// Commands cannot be empty.
			if pos == 0 {
				return None;
			}

			// a is the command excluding the !, b is the rest of the message.
			let (a, b) = message.split_at(pos);

			// Chop off the first whitespace character.
			// To do that we need to figure out where the next character is at.
			let mut indices = b.char_indices();
			// We know that there is at least one character in b (the whitespace).
			indices.next();

			// Check if there are more characters than that one whitespace.
			if let Some((x, _)) = indices.next() {
				Some((a, b.split_at(x).1))
			} else {
				Some((a, ""))
			}
		},

		// No whitespace character means no arguments.
		None => Some((message, ""))
	}
}

fn main() {
	// Read the token from the file.
	let token = read_file("token.conf").expect("Error reading token.conf");

	// Log in to the API.
	let discord = Discord::from_bot_token(&token).expect("Login failed");

	// Connect.
	let (mut connection, ready) = discord.connect().expect("Connect failed");
	println!("[Ready] {} is serving {} servers.", ready.user.username, ready.servers.len());
	let mut state = State::new(ready);

	let bot = Bot::new(&discord, vec![
		Box::new(modules::hello::Module::new()),
		Box::new(modules::modules::Module::new()),
		Box::new(modules::fraktur::Module::new()),
		Box::new(modules::speedruncom::Module::new())/*,
		Box::new(modules::wolframalpha::Module::new())*/

		// The Wolfram!Alpha module requires an app-id to work.
		// Place your app-id into the appropriate spot inside modules/wolframalpha.rs.
	]);

	// Main loop.
	loop {
		let event = match connection.recv_event() {
			Ok(event) => event,
			Err(err) => {
				println!("[Warning] Receive error: {:?}.", err);

				match err {
					discord::Error::WebSocket(..) => {
						// The connection was dropped, try to reconnect.
						let (new_connection, ready) = discord.connect().expect("Connect failed");
						connection = new_connection;
						state = State::new(ready);
						println!("[Ready] Reconnected successfully.");
					},
					discord::Error::Closed(..) => break,
					_ => {}
				}

				continue
			}
		};
		state.update(&event);

		match event {
			Event::MessageCreate(message) => {
				// Skip the message if it comes from us.
				if message.author.id == state.user().id {
					continue
				}

				println!("Message by `{}` ({}): `{}`", message.author.name, message.author.id.0, message.content);

				if let Some((command, text)) = parse_command(&message.content) {
					let command = command.to_lowercase();

					'outer: for m in bot.get_modules() {
						for (&id, &cmds) in m.commands() {
							if let Some(_) = cmds.iter().find(|&&x| x == command) {
								m.handle(&bot, &message, id, text);
								break 'outer;
							}
						}
					}
				}
			}
			_ => {} // Discard other events.
		}
	}
}

#[cfg(test)]
mod tests {
	use super::parse_command;

	#[test]
	fn parse_command_noargs() {
		assert_eq!(Some(("command", "")), parse_command("!command"));
	}

	#[test]
	fn parse_command_noargs_onespace() {
		assert_eq!(Some(("command", "")), parse_command("!command "));
	}

	#[test]
	fn parse_command_noargs_twospaces() {
		assert_eq!(Some(("command", " ")), parse_command("!command  "));
	}

	#[test]
	fn parse_command_usual() {
		assert_eq!(Some(("my_cmd", "a bunch of arguments")), parse_command("!my_cmd a bunch of arguments"));
	}

	#[test]
	fn parse_command_newline() {
		assert_eq!(Some(("test", "arg")), parse_command("!test\narg"));
	}

	#[test]
	fn parse_command_newlines() {
		assert_eq!(Some(("blah", "\n\nargs\nare\nhere\n\n")), parse_command("!blah\n\n\nargs\nare\nhere\n\n"));
	}

	#[test]
	fn parse_command_notcommand() {
		assert_eq!(None, parse_command("Hello"));
	}

	#[test]
	fn parse_command_empty_command() {
		assert_eq!(None, parse_command("!"));
	}

	#[test]
	fn parse_command_empty_command_with_arguments() {
		assert_eq!(None, parse_command("! blah"));
	}

	#[test]
	fn parse_command_unicode() {
		assert_eq!(Some(("КрутаяКоманда1337💖忠犬ハ", "チ公Да")), parse_command("!КрутаяКоманда1337💖忠犬ハ チ公Да"));
	}
}
