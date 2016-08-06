use std::collections::hash_map::HashMap;
use bot::Bot;
use discord::model::Message;
use std::marker::{Send, Sync};

pub trait Module : Send + Sync {
	fn new() -> Self where Self: Sized;

	// Module name.
	fn name(&self) -> &'static str;

	// Short module description.
	fn description(&self) -> &'static str;

	// A map of identifier -> command names.
	// One command identifier may have multiple command names associated with it.
	// Command names must be lowercase.
	fn commands(&self) -> &HashMap<u32, &[&str]>;

	// Short command description.
	fn command_description(&self, id: u32) -> &str;

	// A help message which describes how the command works.
	fn command_help_message(&self, id: u32) -> &str;

	// A function that handles the given command.
	fn handle(&self, bot: &Bot, message: &Message, id: u32, text: &str);

	// A function that gets called when someone sends a message with an attachment.
	fn handle_attachment(&self, _bot: &Bot, _message: &Message) {
	}
}
