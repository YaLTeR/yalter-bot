use bot::Bot;
use discord::model::Message;
use module;
use std::collections::hash_map::HashMap;

pub struct Module<'a> {
	commands: HashMap<u32, &'a [&'a str]>
}

const BOT_CLIENT_ID: &'static str = "PUT YOUR BOT CLIENT ID HERE";

lazy_static! {
	static ref INVITE_LINK: String = format!(
		"https://discordapp.com/oauth2/authorize?client_id={}&scope=bot&permissions=52224",
		BOT_CLIENT_ID
	);
}

enum Commands {
	Invite = 0
}

impl<'a> module::Module for Module<'a> {
	fn new() -> Self {
		static INVITE: [&'static str; 1] = [ "invite" ];
		let mut map: HashMap<u32, &[&str]> = HashMap::new();
		map.insert(Commands::Invite as u32, &INVITE);
		Module { commands: map }
	}

	fn name(&self) -> &'static str {
		"Invite"
	}

	fn description(&self) -> &'static str {
		"Provides the !invite command."
	}

	fn commands(&self) -> &HashMap<u32, &[&str]> {
		&self.commands
	}

	fn command_description(&self, _: u32) -> &'static str {
		"Sends you a PM with a link to invite the bot to your own server."
	}

	fn command_help_message(&self, _: u32) -> &'static str {
		"`!invite` - Get the invite link for the bot."
	}

	fn handle(&self, bot: &Bot, message: &Message, _id: u32, _text: &str) {
		bot.send_pm(
			&message.author.id,
			&format!("Follow this link to invite the bot to your server: {}", *INVITE_LINK),
			&message.channel_id
		);
	}
}
