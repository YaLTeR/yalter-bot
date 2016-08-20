use discord;
use discord::*;
use discord::model::*;
use hyper::status::StatusCode;
use module::Module;
use std::io::Read;
use std::sync::{Arc, RwLock};

pub struct BotThreadUnsafe {
	connection: Connection,
	sync_part: Arc<Bot>
}

pub struct Bot {
	discord: Discord,
	state: RwLock<State>,
	modules: Vec<Box<Module>>
}

impl BotThreadUnsafe {
	pub fn new(discord: Discord, modules: Vec<Box<Module>>) -> Self {
		// Connect.
		let (connection, ready) = discord.connect().expect("Connect failed");
		println!("[Ready] {} is serving {} servers.", ready.user.username, ready.servers.len());

		BotThreadUnsafe {
			connection: connection,
			sync_part: Arc::new(Bot {
				discord: discord,
				state: RwLock::new(State::new(ready)),
				modules: modules
			})
		}
	}

	pub fn receive_event(&mut self) -> Option<Event> {
		let event = match self.connection.recv_event() {
			Ok(event) => event,
			Err(err) => {
				println!("[Warning] Receive error: {:?}.", err);

				match err {
					discord::Error::WebSocket(..) => {
						// The connection was dropped, try to reconnect.
						let (new_connection, ready) = self.sync_part.discord.connect().expect("Connect failed");
						self.connection = new_connection;
						*self.sync_part.state.write().unwrap() = State::new(ready);
						println!("[Ready] Reconnected successfully.");
					},
					discord::Error::Closed(..) => {
						return None
					},
					_ => {}
				}

				return self.receive_event();
			}
		};

		self.sync_part.state.write().unwrap().update(&event);

		Some(event)
	}

	pub fn get_sync(&self) -> &Arc<Bot> {
		&self.sync_part
	}
}

impl Bot {
	pub fn get_modules(&self) -> &Vec<Box<Module>> {
		&self.modules
	}

	pub fn get_state(&self) -> &RwLock<State> {
		&self.state
	}

	pub fn send(&self, channel: &ChannelId, text: &str) {
		self.handle_error(channel,
			self.discord.send_message(
				channel,
				text,
				"",
				false));
	}

	#[allow(dead_code)]
	pub fn edit_or_send_new(&self, channel: &ChannelId, message: &Result<Message>, text: &str) -> Result<Message> {
		match *message {
			Ok(ref msg) => {
				self.discord.edit_message(&msg.channel_id, &msg.id, text)
			},

			Err(_) => {
				self.discord.send_message(channel, text, "", false)
			}
		}
	}

	pub fn send_pm(&self, user: &UserId, text: &str, error_reporting_channel: &ChannelId) {
		match self.discord.create_private_channel(user) {
			Ok(private_channel) => {
				self.handle_error(error_reporting_channel,
					self.discord.send_message(
						&private_channel.id,
						text,
						"",
						false));
			},

			Err(err) => {
				self.handle_error(error_reporting_channel,
					self.discord.send_message(
						error_reporting_channel,
						&format!("Error creating a private channel: `{:?}`.", err),
						"",
						false));
			}
		}
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

	pub fn delete_messages(&self, channel: ChannelId, messages: &[MessageId]) {
		// The Discord API accepts up to 100 at once.
		for chunk in messages.chunks(100) {
			self.handle_error(
				&channel,
				self.discord.delete_messages(channel, chunk)
			);
		}
	}

	pub fn get_messages(&self, channel: ChannelId, what: GetMessages, limit: u64) -> Result<Vec<Message>> {
		self.handle_error_and_return(self.discord.get_messages(channel, what, Some(limit)))
	}

	pub fn get_member(&self, server: ServerId, user: UserId) -> Result<Member> {
		self.handle_error_and_return(self.discord.get_member(server, user))
	}

	pub fn create_channel(&self, server: &ServerId, name: &str, kind: ChannelType) -> Result<Channel> {
		self.handle_error_and_return(self.discord.create_channel(server, name, kind))
	}

	pub fn create_permissions(&self, channel: ChannelId, target: PermissionOverwrite) {
		let _ = self.handle_error_and_return(self.discord.create_permission(channel, target));
	}

	fn handle_error<T>(&self, channel: &ChannelId, res: Result<T>) {
		if let Err(err) = res {
			if let discord::Error::Status(StatusCode::BadRequest, Some(ref value)) = err {
				if let Some(msg) = value.lookup("message.content")
					.and_then(|x| x.as_array())
					.and_then(|x| if x.len() == 0 {
						None
					} else {
						Some(x)
					})
					.and_then(|x| x[0].as_str()) {
					if msg == "String value is too long." {
						self.send(channel, "I tried sending a message but Discord told me it was too long. :(");
					}
				}
			}

			println!("[Warning] {:?}", err);
		}
	}

	fn handle_error_and_return<T>(&self, res: Result<T>) -> Result<T> {
		if let Err(ref err) = res {
			println!("[Warning] {:?}", err);
		}

		return res;
	}
}
