use bot::Bot;
use discord::model::Message;
use module;
use std::cmp::Ordering;
use std::collections::hash_map::HashMap;

struct Command<'a> {
	module: &'a module::Module,
	id: u32,
	names: &'a [&'a str]
}

impl<'a> Command<'a> {
	fn cmp(&self, other: &Self) -> Ordering {
		match self.names[0].cmp(&other.names[0]) {
			Ordering::Equal => {
				match self.module.name().cmp(&other.module.name()) {
					Ordering::Equal => {
						self.module.command_description(self.id).cmp(&other.module.command_description(other.id))
					},
					x => x
				}
			},
			x => x
		}
	}
}

pub struct Module<'a> {
	commands: HashMap<u32, &'a [&'a str]>
}

enum Commands {
	Modules = 0,
	Commands = 1,
	Command = 2
}

impl<'a> Module<'a> {
	fn handle_modules(&self, bot: &Bot, message: &Message, text: &str) {
		if text.len() == 0 {
			let mut buf = "List of available modules:".to_string();
			for m in bot.get_modules() {
				buf.push_str(format!("\n- `{}`: {}", m.name(), m.description()).as_str());
			}

			bot.send(message.channel_id, &buf);
			return;
		}

		let text_lc = text.to_lowercase();

		for m in bot.get_modules() {
			if m.name().to_lowercase() == text_lc {
				let mut buf = format!("`{}`: {}", m.name(), m.description());

				let mut commands: Vec<Command> = Vec::new();
				for (&id, &cmds) in m.commands() {
					commands.push(Command { module: &**m, id: id, names: &cmds });
				}

				if commands.len() == 0 {
					buf.push_str("\nThere are no commands defined by this module.");
				} else {
					commands.sort_by(|a, b| a.cmp(b));

					buf.push_str("\nCommand list:");
					for c in commands {
						let mut first = true;
						for alias in c.names {
							if first {
								first = false;
								buf.push_str(format!("\n- `!{}`", alias).as_str());
							} else {
								buf.push_str(format!(", `!{}`", alias).as_str());
							}
						}

						buf.push_str(format!(": {}", c.module.command_description(c.id)).as_str());
					}
				}

				bot.send(message.channel_id, &buf);
				return;
			}
		}

		bot.send(message.channel_id, format!("There is no module called `{}`.", text).as_str());
	}

	fn handle_commands(&self, bot: &Bot, message: &Message, _text: &str) {
		let mut commands: Vec<Command> = Vec::new();
		for m in bot.get_modules() {
			for (&id, &cmds) in m.commands() {
				commands.push(Command { module: &**m, id: id, names: &cmds });
			}
		}

		commands.sort_by(|a, b| a.cmp(b));

		let mut buf = "Available commands:".to_string();
		for c in commands {
			let mut first = true;
			for alias in c.names {
				if first {
					first = false;
					buf.push_str(format!("\n- `!{}`", alias).as_str());
				} else {
					buf.push_str(format!(", `!{}`", alias).as_str());
				}
			}

			buf.push_str(
				format!(
					" (module `{}`): {}",
					c.module.name(),
					c.module.command_description(c.id)).as_str());
		}

		bot.send(message.channel_id, &buf);
	}

	fn handle_command(&self, bot: &Bot, message: &Message, text: &str) {
		let text = if text.starts_with('!') {
			text.split_at(1).1
		} else {
			text
		};

		if text.len() == 0 {
			bot.send(
				message.channel_id,
				&format!(
					"Bot version {} using **discord-rs**.\n\
					 `!mods` - list modules!\n\
					 `!mod <name>` - list commands of a module!\n\
					 `!help <command>` - help for a command!\n\
					 \n\
					 Or simply:\n\
					 `!commands` - list all commands!",
					env!("CARGO_PKG_VERSION")
				)
			);
			return;
		}

		let text = text.to_lowercase();

		let mut buf = String::new();
		for m in bot.get_modules() {
			for (&id, &cmds) in m.commands() {
				for &alias in cmds {
					if alias == text {
						if buf.len() != 0 {
							buf.push_str("\n\n");
						}

						buf.push_str(format!("`!{}`", text).as_str());

						for alias in cmds {
							if *alias != text {
								buf.push_str(format!(", `!{}`", alias).as_str());
							}
						}

						buf.push_str(format!(": {}\n{}", m.command_description(id), m.command_help_message(id)).as_str());
						break;
					}
				}
			}
		}

		if buf.len() == 0 {
			bot.send(message.channel_id, format!("Could not find the `!{}` command in any of the modules!", text).as_str());
		} else {
			bot.send(message.channel_id, buf.as_str());
		}
	}
}

impl<'a> module::Module for Module<'a> {
	fn new() -> Result<Box<module::Module>, String> {
		let mut map: HashMap<u32, &[&str]> = HashMap::new();
		static MODULES: [&'static str; 4] = [ "modules", "module", "mods", "mod" ];
		map.insert(Commands::Modules as u32, &MODULES);
		static COMMANDS: [&'static str; 2] = [ "commands", "cmds" ];
		map.insert(Commands::Commands as u32, &COMMANDS);
		static COMMAND: [&'static str; 3] = [ "help", "command", "cmd" ];
		map.insert(Commands::Command as u32, &COMMAND);
		Ok(Box::new(Module { commands: map }))
	}

	fn name(&self) -> &'static str {
		"Modules"
	}

	fn description(&self) -> &'static str {
		"A module for enumerating modules and printing information about them."
	}

	fn commands(&self) -> &HashMap<u32, &[&str]> {
		&self.commands
	}

	fn command_description(&self, id: u32) -> &'static str {
		match id {
			x if x == Commands::Modules as u32 =>
				"Information about modules.",
			x if x == Commands::Commands as u32 =>
				"Lists all available commands.",
			x if x == Commands::Command as u32 =>
				"Gets information about the specified command.",
			_ => panic!("Modules::command_description - invalid id.")
		}
	}

	fn command_help_message(&self, id: u32) -> &'static str {
		match id {
			x if x == Commands::Modules as u32 =>
				"`!modules` - lists all available modules;\n\
				 `!modules <name>` - gets information about the specified module and lists its commands.",
			x if x == Commands::Commands as u32 =>
				"`!commands` - lists all available commands.",
			x if x == Commands::Command as u32 =>
				"`!help <command>` - gets information about the specified command.",
			_ => panic!("Modules::command_help_message - invalid id.")
		}
	}

	fn handle(&self, bot: &Bot, message: &Message, id: u32, text: &str) {
		match id {
			x if x == Commands::Modules as u32 =>
				self.handle_modules(bot, message, text),
			x if x == Commands::Commands as u32 =>
				self.handle_commands(bot, message, text),
			x if x == Commands::Command as u32 =>
				self.handle_command(bot, message, text),
			_ => panic!("Modules::handle - invalid id.")
		}
	}
}
