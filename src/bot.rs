use discord;
use discord::Discord;
use discord::model::*;
use hyper::status::StatusCode;
use module::Module;
use std::io::Read;

pub struct Bot<'a> {
	discord: &'a Discord,
	modules: Vec<Box<Module>>
}

impl<'a> Bot<'a> {
	pub fn new(discord: &'a Discord, modules: Vec<Box<Module>>) -> Self {
		Bot { discord: discord, modules: modules }
	}

	pub fn get_modules(&self) -> &Vec<Box<Module>> {
		&self.modules
	}

	pub fn send(&self, channel: &ChannelId, text: &str) {
		self.handle_error(channel,
			self.discord.send_message(
				channel,
				text,
				"",
				false));
	}

	pub fn send_file<R: Read>(&self, channel: &ChannelId, text: &str, file: R, filename: &str) {
		self.handle_error(channel,
			self.discord.send_file(
				channel,
				text,
				file,
				filename));
	}

	pub fn broadcast_typing(&self, channel: &ChannelId) {
		self.handle_error(channel, self.discord.broadcast_typing(channel));
	}

	fn handle_error<T>(&self, channel: &ChannelId, res: Result<T, discord::Error>) {
		if let Err(err) = res {
			if let discord::Error::Status(StatusCode::BadRequest, Some(ref value)) = err {
				if let Some(msg) = value.lookup("message.content")
					.and_then(|x| x.as_array())
					.and_then(|x| if x.len() == 0 {
						None
					} else {
						Some(x)
					})
					.and_then(|x| x[0].as_string()) {
					if msg == "String value is too long." {
						self.send(channel, "I tried sending a message but Discord told me it was too long. :(");
						return;
					}
				}
			}

			println!("[Warning] {:?}", err);
		}
	}
}
