extern crate circular_queue;
extern crate hldemo;
extern crate hyper;
#[macro_use]
extern crate lazy_static;
extern crate rand;
extern crate regex;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate url;
extern crate xml;

use std::env;
use std::sync::Arc;
use std::thread;

extern crate discord;
use discord::model::*;
use discord::{ChannelRef, Discord};

mod module;
use module::Module;

mod bot;
use bot::*;

mod modules {
    pub mod admin;
    pub mod demos;
    pub mod fun;
    pub mod hello;
    pub mod invite;
    pub mod modules;
    pub mod speedruncom;
    pub mod wolframalpha;
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
        }

        // No whitespace character means no arguments.
        None => Some((message, "")),
    }
}

fn handle_command(bot: Arc<Bot>, message: Arc<Message>, command: &str, text: &str) {
    let command = command.to_lowercase();

    let mut index = None;

    'outer: for i in 0..bot.get_modules().len() {
        let module = &bot.get_modules()[i];

        for (&id, &cmds) in module.commands() {
            if cmds.iter().any(|&x| x == command) {
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

fn handle_message_update(bot: Arc<Bot>, channel_id: ChannelId, id: MessageId) {
    thread::spawn(move || {
        for module in bot.get_modules() {
            module.handle_message_update(&bot, channel_id, id);
        }
    });
}

fn handle_message_delete(bot: Arc<Bot>, channel_id: ChannelId, id: MessageId) {
    thread::spawn(move || {
        for module in bot.get_modules() {
            module.handle_message_delete(&bot, channel_id, id);
        }
    });
}

fn main() {
    // Read the token.
    let token =
        env::var("YALTER_BOT_TOKEN").expect("Please set the YALTER_BOT_TOKEN environment variable");

    // Log in to the API.
    let discord = Discord::from_bot_token(&token).expect("Login failed");

    let modules = vec![
        modules::hello::Module::new(),
        modules::modules::Module::new(),
        modules::fun::Module::new(),
        modules::speedruncom::Module::new(),
        modules::admin::Module::new(),
        modules::wolframalpha::Module::new(),
        modules::invite::Module::new(),
        modules::demos::Module::new(),
    ].into_iter()
    .filter_map(|m| match m {
        Ok(m) => Some(m),
        Err(err) => {
            println!("{}", err);
            None
        }
    }).collect();

    let mut bot = BotThreadUnsafe::new(discord, modules);

    // Main loop.
    while let Some(event) = bot.receive_event() {
        match event {
            Event::MessageCreate(message) => {
                let state = bot.get_sync().get_state().read().unwrap();

                // Skip the message if it comes from us.
                if message.author.id == state.user().id {
                    continue;
                }

                match state.find_channel(message.channel_id) {
                    Some(ChannelRef::Public(server, channel)) => {
                        println!(
                            "[`{}` `#{}`] `{}`: `{}`",
                            server.name, channel.name, message.author.name, message.content
                        );
                    }

                    Some(ChannelRef::Group(group)) => {
                        println!(
                            "[Group `{}`] `{}`: `{}`",
                            group.name(),
                            message.author.name,
                            message.content
                        );
                    }

                    Some(ChannelRef::Private(channel)) => {
                        if message.author.name == channel.recipient.name {
                            println!("[Private] `{}`: `{}`", message.author.name, message.content);
                        } else {
                            println!(
                                "[Private] To `{}`: `{}`",
                                channel.recipient.name, message.content
                            );
                        }
                    }

                    None => println!(
                        "[Unknown Channel] `{}`: `{}`",
                        message.author.name, message.content
                    ),
                }

                let message_shared = Arc::new(message);

                // Handle the commands.
                if let Some((command, text)) = parse_command(&message_shared.content) {
                    handle_command(
                        bot.get_sync().clone(),
                        message_shared.clone(),
                        command,
                        text,
                    );
                }

                // Handle the attachments.
                if !message_shared.attachments.is_empty() {
                    handle_attachment(bot.get_sync().clone(), message_shared);
                }
            }

            Event::MessageUpdate { id, channel_id, .. } => {
                handle_message_update(bot.get_sync().clone(), channel_id, id);
            }

            Event::MessageDelete {
                channel_id,
                message_id,
            } => {
                handle_message_delete(bot.get_sync().clone(), channel_id, message_id);
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
        assert_eq!(
            Some(("my_cmd", "a bunch of arguments")),
            parse_command("!my_cmd a bunch of arguments")
        );
    }

    #[test]
    fn parse_command_newline() {
        assert_eq!(Some(("test", "arg")), parse_command("!test\narg"));
    }

    #[test]
    fn parse_command_newlines() {
        assert_eq!(
            Some(("blah", "\n\nargs\nare\nhere\n\n")),
            parse_command("!blah\n\n\nargs\nare\nhere\n\n")
        );
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
        assert_eq!(
            Some(("КрутаяКоманда1337💖忠犬ハ", "チ公Да")),
            parse_command("!КрутаяКоманда1337💖忠犬ハ チ公Да")
        );
    }
}
