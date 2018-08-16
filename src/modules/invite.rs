use bot::Bot;
use discord::model::Message;
use module;
use std::{collections::hash_map::HashMap, env};

pub struct Module<'a> {
    commands: HashMap<u32, &'a [&'a str]>,
}

lazy_static! {
    static ref INVITE_LINK: Result<String, String> = env::var("YALTER_BOT_CLIENT_ID")
        .map_err(|_| "Please set the YALTER_BOT_CLIENT_ID environment variable".to_string())
        .map(|client_id| format!(
            "https://discordapp.com/oauth2/authorize?client_id={}&scope=bot&permissions=271707152",
            client_id
        ));
}

enum Commands {
    Invite = 0,
}

impl<'a> module::Module for Module<'a> {
    fn new() -> Result<Box<module::Module>, String> {
        static INVITE: [&'static str; 1] = ["invite"];
        let mut map: HashMap<u32, &[&str]> = HashMap::new();
        map.insert(Commands::Invite as u32, &INVITE);
        INVITE_LINK
            .as_ref()
            .map_err(|s| s.clone())
            .and(Ok(Box::new(Module { commands: map })))
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
            message.author.id,
            &format!(
                "Follow this link to invite the bot to your server: {}",
                INVITE_LINK.as_ref().unwrap()
            ),
            message.channel_id,
        );
    }
}
