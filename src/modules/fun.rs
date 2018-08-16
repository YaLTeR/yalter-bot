use bot::Bot;
use circular_queue::CircularQueue;
use discord::{model::*, ChannelRef};
use module;
use rand::{self, Rng};
use regex::Regex;
use std::{char, collections::hash_map::HashMap, sync::RwLock};

const COMMAND_MESSAGE_QUEUE_SIZE: usize = 64;

#[derive(Debug, Clone, Copy)]
struct CommandMessage {
    /// Message with the command.
    command: MessageId,
    /// Author of the message with the command.
    author: UserId,
    /// Message the bot sent as a response.
    output: MessageId,
}

pub struct Module<'a> {
    commands: HashMap<u32, &'a [&'a str]>,
    command_messages: RwLock<HashMap<ChannelId, CircularQueue<CommandMessage>>>,
}

lazy_static! {
    static ref TEMPERATURE_REGEX: Regex =
        Regex::new(r"\s*([+-]?[0-9]+(\.[0-9]*)?)\s*([CcFf]).*").unwrap();
    static ref ROLL_REGEX: Regex = Regex::new(r"\s*(([0-9]+)(\s|$))?.*").unwrap();
    static ref ROOM_ALLOW_PERMS: Permissions = permissions::VOICE_CONNECT
        | permissions::VOICE_SPEAK
        | permissions::MANAGE_CHANNELS
        | permissions::MANAGE_ROLES;
    static ref ROOM_DENY_PERMS: Permissions = permissions::VOICE_CONNECT;
}

enum Commands {
    Fraktur = 0,
    Temperature = 1,
    Roll = 2,
    Pick = 3,
    Info = 4,
    Room = 5,
    Aesthetic = 6,
    Smallcaps = 7,
}

impl<'a> module::Module for Module<'a> {
    fn new() -> Result<Box<module::Module>, String> {
        let mut map: HashMap<u32, &[&str]> = HashMap::new();
        static FRAKTUR: [&'static str; 1] = ["fraktur"];
        map.insert(Commands::Fraktur as u32, &FRAKTUR);
        static TEMPERATURE: [&'static str; 2] = ["temperature", "temp"];
        map.insert(Commands::Temperature as u32, &TEMPERATURE);
        static ROLL: [&'static str; 1] = ["roll"];
        map.insert(Commands::Roll as u32, &ROLL);
        static PICK: [&'static str; 2] = ["pick", "choose"];
        map.insert(Commands::Pick as u32, &PICK);
        static INFO: [&'static str; 2] = ["information", "info"];
        map.insert(Commands::Info as u32, &INFO);
        static ROOM: [&'static str; 1] = ["room"];
        map.insert(Commands::Room as u32, &ROOM);
        static AESTHETIC: [&'static str; 2] = ["aesthetic", "fullwidth"];
        map.insert(Commands::Aesthetic as u32, &AESTHETIC);
        static SMALLCAPS: [&'static str; 1] = ["smallcaps"];
        map.insert(Commands::Smallcaps as u32, &SMALLCAPS);
        Ok(Box::new(Module {
            commands: map,
            command_messages: RwLock::new(HashMap::new()),
        }))
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
            x if x == Commands::Fraktur as u32 => {
                "Prints the given text in 𝔣𝔯𝔞𝔨𝔱𝔲𝔯 (gothic math symbols)."
            }
            x if x == Commands::Temperature as u32 => {
                "Converts the temperature between Celsius and Fahrenheit."
            }
            x if x == Commands::Roll as u32 => "Prints a random number.",
            x if x == Commands::Pick as u32 => "Randomly picks one of the given options.",
            x if x == Commands::Info as u32 => "Prints out some information about the server.",
            x if x == Commands::Room as u32 => "Makes private voice rooms.",
            x if x == Commands::Aesthetic as u32 => {
                "Prints the given text in ｆｕｌｌｗｉｄｔｈ characters."
            }
            x if x == Commands::Smallcaps as u32 => {
                "Converts capital letters to ꜱᴍᴀʟʟ ᴄᴀᴘɪᴛᴀʟ letters."
            }
            _ => panic!("Fun::command_description - invalid id."),
        }
    }

    fn command_help_message(&self, id: u32) -> &'static str {
        match id {
            x if x == Commands::Fraktur as u32 => {
                "`!fraktur <text>` - Prints the given text in 𝔣𝔯𝔞𝔨𝔱𝔲𝔯 (gothic math symbols). Note that there are no regular versions of letters 'C', 'H', 'I', 'R', 'Z'; those are replaced with their bold versions."
            }
            x if x == Commands::Temperature as u32 => {
                "`!temperature <number> <C or F>` - Converts the temperature into another scale. For example, `!temp 5C` outputs 41."
            }
            x if x == Commands::Roll as u32 => {
                "`!roll [high]` - Prints a random number between 0 and 99, or between 0 and high - 1, inclusive."
            }
            x if x == Commands::Pick as u32 => {
                "`!pick something;something else[;third option[;...]]` - Randomly picks one of the given options."
            }
            x if x == Commands::Info as u32 => {
                "`!information` - Prints out some information about the server."
            }
            x if x == Commands::Room as u32 => {
                "`!room <user or role mention(-s)>` - Makes a private voice room for you and mentioned users. The room is __NOT YET__ automatically deleted after a certain amount of time when everyone leaves it."
            }
            x if x == Commands::Aesthetic as u32 => {
                "`!aesthetic <text>` - Prints the given text in ｆｕｌｌｗｉｄｔｈ characters."
            }
            x if x == Commands::Smallcaps as u32 => {
                "`!smallcaps <text>` - Converts capital letters to ꜱᴍᴀʟʟ ᴄᴀᴘɪᴛᴀʟ letters. Note that there are no small capital versions of letters 'Q' and 'X'."
            }
            _ => panic!("Fun::command_help_message - invalid id."),
        }
    }

    fn handle(&self, bot: &Bot, message: &Message, id: u32, text: &str) {
        match id {
            x if x == Commands::Fraktur as u32 => self.handle_fraktur(bot, message, text),
            x if x == Commands::Temperature as u32 => self.handle_temperature(bot, message, text),
            x if x == Commands::Roll as u32 => self.handle_roll(bot, message, text),
            x if x == Commands::Pick as u32 => self.handle_pick(bot, message, text),
            x if x == Commands::Info as u32 => self.handle_info(bot, message, text),
            x if x == Commands::Room as u32 => self.handle_room(bot, message, text),
            x if x == Commands::Aesthetic as u32 => self.handle_aesthetic(bot, message, text),
            x if x == Commands::Smallcaps as u32 => self.handle_smallcaps(bot, message, text),
            _ => panic!("Fun::handle - invalid id."),
        }
    }

    fn handle_message_update(&self, bot: &Bot, channel_id: ChannelId, id: MessageId) {
        if let Some(output) = self.find_command_message(channel_id, id) {
            if let Ok(message) = bot.get_message(channel_id, output.output) {
                bot.edit(
                    channel_id,
                    output.output,
                    &format!(
                        "{:.2000}",
                        &format!("{} said: {}", output.author.mention(), message.content)
                    ),
                );
            }
        }
    }

    fn handle_message_delete(&self, bot: &Bot, channel_id: ChannelId, id: MessageId) {
        if let Some(output) = self.find_command_message(channel_id, id) {
            if let Ok(message) = bot.get_message(channel_id, output.output) {
                bot.edit(
                    channel_id,
                    output.output,
                    &format!(
                        "{:.2000}",
                        &format!("{} said: {}", output.author.mention(), message.content)
                    ),
                );
            }
        }
    }
}

impl<'a> Module<'a> {
    fn remember_command_message(
        &self,
        channel_id: ChannelId,
        command_id: MessageId,
        author: UserId,
        output_id: MessageId,
    ) {
        self.command_messages
            .write()
            .unwrap()
            .entry(channel_id)
            .or_insert_with(|| CircularQueue::with_capacity(COMMAND_MESSAGE_QUEUE_SIZE))
            .push(CommandMessage {
                command: command_id,
                author,
                output: output_id,
            });
    }

    fn find_command_message(
        &self,
        channel_id: ChannelId,
        command_id: MessageId,
    ) -> Option<CommandMessage> {
        self.command_messages
            .read()
            .unwrap()
            .get(&channel_id)
            .and_then(|x| x.iter().find(|x| x.command == command_id).cloned())
    }

    fn handle_fraktur(&self, bot: &Bot, message: &Message, text: &str) {
        let reply = text.chars().map(frakturize).collect::<String>();
        if let Some(output) = bot.send_and_get(message.channel_id, &reply) {
            self.remember_command_message(
                message.channel_id,
                message.id,
                message.author.id,
                output.id,
            );
        }
    }

    fn handle_aesthetic(&self, bot: &Bot, message: &Message, text: &str) {
        let reply = text.chars().map(make_fullwidth).collect::<String>();
        if let Some(output) = bot.send_and_get(message.channel_id, &reply) {
            self.remember_command_message(
                message.channel_id,
                message.id,
                message.author.id,
                output.id,
            );
        }
    }

    fn handle_smallcaps(&self, bot: &Bot, message: &Message, text: &str) {
        let reply = text.chars().map(make_smallcaps).collect::<String>();
        if let Some(output) = bot.send_and_get(message.channel_id, &reply) {
            self.remember_command_message(
                message.channel_id,
                message.id,
                message.author.id,
                output.id,
            );
        }
    }

    fn handle_temperature(&self, bot: &Bot, message: &Message, text: &str) {
        if let Some(caps) = TEMPERATURE_REGEX.captures(text) {
            let value = caps.get(1).unwrap().as_str().parse::<f32>().unwrap();
            let letter = caps.get(3).unwrap().as_str().chars().next().unwrap();

            let converted_value = match letter {
                'C' | 'c' => 9f32 * value / 5f32 + 32f32,
                'F' | 'f' => 5f32 * (value - 32f32) / 9f32,
                _ => panic!("Regex error in Fun::handle_temperature."),
            };

            let converted_letter = match letter {
                'C' | 'c' => 'F',
                'F' | 'f' => 'C',
                _ => panic!("Regex error in Fun::handle_temperature."),
            };

            bot.send(
                message.channel_id,
                &format!(
                    "{:.2}°{} is **{:.2}**°{}.",
                    value,
                    letter.to_uppercase().next().unwrap(),
                    converted_value,
                    converted_letter
                ),
            );
        } else {
            bot.send(
                message.channel_id,
                <Module as module::Module>::command_help_message(
                    &self,
                    Commands::Temperature as u32,
                ),
            );
        }
    }

    fn handle_roll(&self, bot: &Bot, message: &Message, text: &str) {
        let caps = ROLL_REGEX.captures(text).unwrap();
        let max = caps
            .get(2)
            .and_then(|x| x.as_str().parse::<u64>().ok())
            .map(|x| if x == 0 { 100 } else { x })
            .unwrap_or(100);

        let number = rand::thread_rng().gen_range(0, max);

        bot.send(
            message.channel_id,
            &format!("{} rolled **{}**!", message.author.mention(), number),
        );
    }

    fn handle_pick(&self, bot: &Bot, message: &Message, text: &str) {
        let options: Vec<&str> = text.split(';').filter(|x| !x.is_empty()).collect();

        if options.len() < 2 {
            bot.send(
                message.channel_id,
                <Module as module::Module>::command_help_message(&self, Commands::Pick as u32),
            );
        } else {
            let choice = rand::thread_rng().choose(&options).unwrap();

            bot.send(
                message.channel_id,
                &format!("{}: I pick {}!", message.author.mention(), choice),
            );
        }
    }

    fn handle_info(&self, bot: &Bot, message: &Message, _text: &str) {
        match bot
            .get_state()
            .read()
            .unwrap()
            .find_channel(message.channel_id)
        {
            Some(ChannelRef::Private(channel)) => {
                bot.send(message.channel_id, &format!("```{:#?}```", channel));
            }

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
                    if let Some(ref icon) = server.icon {
                        &icon
                    } else {
                        "N/A"
                    }
                );

                if server.roles.is_empty() {
                    buf.push_str(" N/A");
                } else {
                    for role in &server.roles {
                        buf.push_str(&format!("\n- {} '{}'", role.id.0, role.name));
                    }
                }

                buf.push_str(&format!("\n\nChannel ID: {}```", channel.id.0));

                bot.send(message.channel_id, &buf);
            }

            Some(ChannelRef::Group(group)) => {
                bot.send(message.channel_id, &format!("```{:#?}```", group));
            }

            None => {
                bot.send(
                    message.channel_id,
                    "Huh, I couldn't get this channel's info for some reason. Try again I guess?",
                );
            }
        }
    }

    fn handle_room(&self, bot: &Bot, message: &Message, _text: &str) {
        match bot
            .get_state()
            .read()
            .unwrap()
            .find_channel(message.channel_id)
        {
            Some(ChannelRef::Private(_)) | Some(ChannelRef::Group(_)) => {
                bot.send(message.channel_id, "Well, what do you expect me to do?");
            }

            Some(ChannelRef::Public(server, _)) => {
                if !message.mentions.is_empty() || !message.mention_roles.is_empty() {
                    let number = rand::random::<u64>();

                    match bot.create_channel(
                        server.id,
                        &format!("🤖 - ybot - {:x}", number),
                        ChannelType::Voice,
                    ) {
                        Ok(Channel::Public(new_channel)) => {
                            // Ban @everyone from joining.
                            bot.create_permissions(
                                new_channel.id,
                                PermissionOverwrite {
                                    kind: PermissionOverwriteType::Role(RoleId(server.id.0)),
                                    allow: Permissions::empty(),
                                    deny: *ROOM_DENY_PERMS,
                                },
                            );

                            // Allow the message author to join and speak.
                            bot.create_permissions(
                                new_channel.id,
                                PermissionOverwrite {
                                    kind: PermissionOverwriteType::Member(message.author.id),
                                    allow: *ROOM_ALLOW_PERMS,
                                    deny: Permissions::empty(),
                                },
                            );

                            // Allow the mentioned users / roles to join and speak.
                            for user in &message.mentions {
                                bot.create_permissions(
                                    new_channel.id,
                                    PermissionOverwrite {
                                        kind: PermissionOverwriteType::Member(user.id),
                                        allow: *ROOM_ALLOW_PERMS,
                                        deny: Permissions::empty(),
                                    },
                                );
                            }

                            for role_id in &message.mention_roles {
                                bot.create_permissions(
                                    new_channel.id,
                                    PermissionOverwrite {
                                        kind: PermissionOverwriteType::Role(*role_id),
                                        allow: *ROOM_ALLOW_PERMS,
                                        deny: Permissions::empty(),
                                    },
                                );
                            }
                        }

                        Ok(Channel::Private(_)) => {
                            bot.send(
                                message.channel_id,
                                "I made a private channel?! How did I what.",
                            );
                        }

                        Ok(Channel::Group(_)) => {
                            bot.send(message.channel_id, "I made a group?! How did I what.");
                        }

                        Err(err) => {
                            bot.send(
                                message.channel_id,
                                &format!("Couldn't create a new channel: {} :/", err),
                            );
                        }
                    }
                } else {
                    bot.send(
                        message.channel_id,
                        <Module as module::Module>::command_help_message(
                            &self,
                            Commands::Room as u32,
                        ),
                    );
                }
            }

            None => {
                bot.send(
                    message.channel_id,
                    "Huh, I couldn't get this channel's info for some reason. Try again I guess?",
                );
            }
        }
    }
}

fn frakturize(c: char) -> char {
    match c {
        'a'...'z' => char::from_u32(('𝔞' as u32) - ('a' as u32) + (c as u32)).unwrap(),
        // Those ones are absent from non-bold apparently
        'C' | 'H' | 'I' | 'R' | 'Z' => {
            char::from_u32(('𝕬' as u32) - ('A' as u32) + (c as u32)).unwrap()
        }
        'A'...'Z' => char::from_u32(('𝔄' as u32) - ('A' as u32) + (c as u32)).unwrap(),
        _ => c,
    }
}

fn make_fullwidth(c: char) -> char {
    match c {
        '!'...'~' => char::from_u32(('！' as u32) - ('!' as u32) + (c as u32)).unwrap(),
        ' ' => '　',
        _ => c,
    }
}

fn make_smallcaps(c: char) -> char {
    let original = "ABCDEFGHIJKLMNOPRSTUVWYZÆŒÐƷƎŁƆШГΛПРΨΩЛ";
    let smallcaps = "ᴀʙᴄᴅᴇꜰɢʜɪᴊᴋʟᴍɴᴏᴘʀꜱᴛᴜᴠᴡʏᴢᴁɶᴆᴣⱻᴌᴐꟺᴦᴧᴨᴘᴪꭥл";

    if let Some(m) = original.chars().enumerate().find(|x| x.1 == c) {
        return smallcaps.chars().nth(m.0).unwrap();
    }

    c
}
