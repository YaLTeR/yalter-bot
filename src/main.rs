extern crate hyper;
#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate rand;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate url;
extern crate xml;

use std::fs::File;
use std::io;
use std::io::Read;
use std::sync::Arc;
use std::thread;

extern crate discord;
use discord::{ChannelRef, Discord};
use discord::model::*;

mod module;
use module::Module;

mod bot;
use bot::*;

mod modules {
	pub mod hello;
	pub mod modules;
	pub mod fun;
	pub mod speedruncom;
	pub mod wolframalpha;
	pub mod invite;
	pub mod admin;
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

fn handle_command(bot: Arc<Bot>, message: Arc<Message>, command: &str, text: &str) {
	let command = command.to_lowercase();

	let mut index = None;

	'outer: for i in 0..bot.get_modules().len() {
		let module = &bot.get_modules()[i];

		for (&id, &cmds) in module.commands() {
			if let Some(_) = cmds.iter().find(|&&x| x == command) {
				index = Some((i, id));
				break 'outer;
			}
		}
	}

	if let Some((i, id)) = index {
		let text_copy = text.to_string();

		thread::spawn(move || {
			bot.get_modules()[i].handle(&bot, &message, id, &text_copy);
		});
	}
}

fn handle_attachment(bot: Arc<Bot>, message: Arc<Message>) {
	thread::spawn(move || {
		for module in bot.get_modules() {
			module.handle_attachment(&bot, &message);
		}
	});
}

fn main() {
	// Read the token from the file.
	let token = read_file("token.conf").expect("Error reading token.conf");

	// Log in to the API.
	let discord = Discord::from_bot_token(&token).expect("Login failed");

	let mut modules: Vec<Box<Module>> = Vec::new();
	modules.push(Box::new(modules::hello::Module::new()));
	modules.push(Box::new(modules::modules::Module::new()));
	modules.push(Box::new(modules::fun::Module::new()));
	modules.push(Box::new(modules::speedruncom::Module::new()));
	modules.push(Box::new(modules::admin::Module::new()));

	// The Wolfram!Alpha module requires an app-id to work.
	// Place your app-id into the appropriate spot inside modules/wolframalpha.rs.
	// modules.push(Box::new(modules::wolframalpha::Module::new()));

	// The Invite module requires a bot client ID to work.
	// Get it from https://discordapp.com/developers/applications/me
	// Place your client ID into the appropriate spot inside modules/invite.rs.
	// modules.push(Box::new(modules::invite::Module::new()));

	let mut bot = BotThreadUnsafe::new(discord, modules);

	// Main loop.
	loop {
		let event = match bot.receive_event() {
			Some(event) => event,
			None => {
				break;
			}
		};

		match event {
			Event::MessageCreate(message) => {
				let state = bot.get_sync().get_state().read().unwrap();

				// Skip the message if it comes from us.
				if message.author.id == state.user().id {
					continue
				}

				match state.find_channel(&message.channel_id) {
					Some(ChannelRef::Public(server, channel)) => {
						println!("[`{}` `#{}`] `{}`: `{}`", server.name, channel.name, message.author.name, message.content);
					}

					Some(ChannelRef::Group(group)) => {
						println!("[Group `{}`] `{}`: `{}`", group.name(), message.author.name, message.content);
					}

					Some(ChannelRef::Private(channel)) => {
						if message.author.name == channel.recipient.name {
							println!("[Private] `{}`: `{}`", message.author.name, message.content);
						} else {
							println!("[Private] To `{}`: `{}`", channel.recipient.name, message.content);
						}
					}

					None => println!("[Unknown Channel] `{}`: `{}`", message.author.name, message.content)
				}

				let message_shared = Arc::new(message);

				// Handle the commands.
				if let Some((command, text)) = parse_command(&message_shared.content) {
					handle_command(bot.get_sync().clone(), message_shared.clone(), command, text);
				}

				// Handle the attachments.
				if message_shared.attachments.len() > 0 {
					handle_attachment(bot.get_sync().clone(), message_shared);
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
