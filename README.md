# yalter-bot
A simple Discord bot written in Rust using the [discord-rs](https://github.com/SpaceManiac/discord-rs) library.

This is my "check out / learn Rust" project.

### Usage
The bot expects some environment variables:
- `YALTER_BOT_TOKEN` — the Discord bot token,
- `YALTER_BOT_CLIENT_ID` — the Discord bot client ID, set to enable the invite module,
- `YALTER_BOT_WOLFRAMALPHA_APPID` — the Wolfram!Alpha app ID, set to enable the Wolfram!Alpha module.

### Basic commands
- `!modules` - view information about modules and their commands.
- `!commands` - list all available commands.
- `!help <command>` - get help for a given command.
