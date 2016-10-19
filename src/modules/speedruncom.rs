use bot::Bot;
use discord::model::Message;
use hyper;
use hyper::client::Client;
use hyper::header::UserAgent;
use module;
use std::collections::BTreeMap;
use std::collections::hash_map::HashMap;
use std::error;
use std::fmt;
use std::time::Duration;
use serde_json;
use url::Url;

include!(concat!(env!("OUT_DIR"), "/speedruncom_types.rs"));

pub struct Module<'a> {
	commands: HashMap<u32, &'a [&'a str]>
}

lazy_static! {
	static ref SPEEDRUNCOM_API_BASE: Url = Url::parse("https://www.speedrun.com/api/v1/").unwrap();
	static ref USERAGENT: UserAgent = UserAgent(concat!("yalter-bot/", env!("CARGO_PKG_VERSION")).to_string());
}

#[derive(Debug)]
enum MyError {
	Network(hyper::error::Error),
	Json(serde_json::error::Error),
	NoSuchGame,
	Custom(String)
}

impl fmt::Display for MyError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			MyError::Network(ref err) => write!(f, "Network error: {}", err),
			MyError::Json(ref err) => write!(f, "JSON error: {}", err),
			MyError::NoSuchGame => write!(f, "There's no such game on speedrun.com!"),
			MyError::Custom(ref err) => write!(f, "{}", err)
		}
	}
}

impl error::Error for MyError {
	fn description(&self) -> &str {
		match *self {
			MyError::Network(ref err) => err.description(),
			MyError::Json(ref err) => err.description(),
			MyError::NoSuchGame => "There's no such game on speedrun.com!",
			MyError::Custom(ref err) => err
		}
	}

	fn cause(&self) -> Option<&error::Error> {
		match *self {
			MyError::Network(ref err) => Some(err),
			MyError::Json(ref err) => Some(err),
			MyError::NoSuchGame => None,
			MyError::Custom(ref _err) => None
		}
	}
}

impl From<hyper::error::Error> for MyError {
	fn from(err: hyper::error::Error) -> MyError {
		MyError::Network(err)
	}
}

impl From<serde_json::error::Error> for MyError {
	fn from(err: serde_json::error::Error) -> MyError {
		MyError::Json(err)
	}
}

impl<'a> From<&'a str> for MyError {
	fn from(err: &'a str) -> MyError {
		MyError::Custom(err.to_string())
	}
}

impl From<String> for MyError {
	fn from(err: String) -> MyError {
		MyError::Custom(err)
	}
}

enum Commands {
	WR = 0
}

impl<'a> module::Module for Module<'a> {
	fn new() -> Self {
		static WR: [&'static str; 2] = [ "worldrecord", "wr" ];
		let mut map: HashMap<u32, &[&str]> = HashMap::new();
		map.insert(Commands::WR as u32, &WR);
		Module { commands: map }
	}

	fn name(&self) -> &'static str {
		"Speedrun"
	}

	fn description(&self) -> &'static str {
		"Various speedrun-related commands."
	}

	fn commands(&self) -> &HashMap<u32, &[&str]> {
		&self.commands
	}

	fn command_description(&self, _: u32) -> &'static str {
		"Shows the world record times."
	}

	fn command_help_message(&self, _: u32) -> &'static str {
		"`!wr <game>` - Shows the world record times for all categories for the given game. For example, `!wr Half-Life`."
	}

	fn handle(&self, bot: &Bot, message: &Message, _id: u32, text: &str) {
		handle_wr(&bot, &message, &text);
	}
}

fn handle_wr(bot: &Bot, message: &Message, text: &str) {
	bot.send(&message.channel_id, match get_wrs(&text) {
		Ok((game, wrs)) => {
			if wrs.len() == 0 {
				format!("**{}** has no world records. :|", game)
			} else {
				let mut buf = format!("World records for **{}**:", game);
				for mut wr in wrs {
					buf.push_str(&format!("\n{}", wr.category));

					if let Some(subcategory) = wr.subcategory {
						buf.push_str(&format!(" ({})", subcategory));
					}

					buf.push_str(&format!(": **{}** by {}", format_time(&wr.time), wr.players[0]));

					wr.players.remove(0);
					if let Some(last_player) = wr.players.pop() {
						for player in wr.players {
							buf.push_str(&format!(", {}", player));
						}
						buf.push_str(&format!(" and {}", last_player));
					}

					buf.push('!');
				}
				buf
			}
		},
		Err(MyError::Network(err)) => {
			format!("Couldn't communicate with https://www.speedrun.com. :( ({})", err)
		},
		Err(MyError::NoSuchGame) => {
			"There's no such game on speedrun.com! :O".to_string()
		},
		Err(err) => {
			format!("Something's broken. :/ ({})", err)
		}
	}.as_str());
}

fn format_time(time: &Duration) -> String {
	let total_seconds = time.as_secs();
	let nanoseconds = time.subsec_nanos();
	
	let hours = total_seconds / 3600;
	let minutes = total_seconds / 60 - hours * 60;
	let seconds = total_seconds - minutes * 60 - hours * 3600;
	let milliseconds = nanoseconds / 1000000;
	
	let mut buf = String::new();
	if hours > 0 {
		buf.push_str(format!("{:02}:{:02}:{:02}", hours, minutes, seconds).as_str());
	} else {
		buf.push_str(format!("{:02}:{:02}", minutes, seconds).as_str());
	}
	
	if milliseconds > 0 {
		buf.push_str(format!(".{:03}", milliseconds).as_str());
	}
	
	buf
}

struct WR {
	category: String,
	subcategory: Option<String>,
	players: Vec<String>,
	time: Duration
}

fn get_wrs(text: &str) -> Result<(String, Vec<WR>), MyError> {
	let mut games = SPEEDRUNCOM_API_BASE.join("games").unwrap();
	games.query_pairs_mut()
		.append_pair("name", text)
		.append_pair("embed", "categories.variables")
		.append_pair("max", "1");
		
	let client = Client::new();
	let result = try!(client.get(games.as_str()).header(USERAGENT.clone()).send());

	let games: APIGames = try!(serde_json::de::from_reader(result));
	if games.data.is_empty() {
		return Err(MyError::NoSuchGame);
	}

	let game = games.data.into_iter().next().unwrap();

	let categories: Vec<APIGamesCategoriesData> = game.categories.data.into_iter().filter(|x| x.type_ == "per-game").collect();
	if categories.is_empty() {
		return Err(MyError::Custom(format!("*{}* doesn't seem to have any categories. :/", game.names.international)));
	}

	let mut wrs = Vec::new();

	for category in categories.into_iter() {
		if let Some(subcategory_variable) = category.variables.data.into_iter().filter(|x| x.is_subcategory).next() {
			// Get runs for each subcategory value.

			for (value_id, value) in subcategory_variable.values.values {
				let mut leaderboard = try!(SPEEDRUNCOM_API_BASE.join(
					&format!("leaderboards/{}/category/{}", game.id, category.id)
				).map_err(|x| x.to_string()));

				leaderboard.query_pairs_mut()
					.append_pair("top", "1")
					.append_pair("embed", "players")
					.append_pair(&format!("var-{}", subcategory_variable.id), &value_id);

				let result = try!(client.get(leaderboard.as_str()).header(USERAGENT.clone()).send());

				let leaderboard: APILeaderboards = try!(serde_json::de::from_reader(result));

				let runs = leaderboard.data.runs;
				if runs.is_empty() {
					// Empty subcategory.
					continue;
				}

				let time = Duration::from_millis((runs[0].run.times.primary_t * 1000f64) as u64);

				let players: Vec<String> = leaderboard.data.players.data.into_iter()
				                                                        .map(|x| x.names.map(|n| n.international)
				                                                                        .or(x.name)
				                                                                        .unwrap_or("nameless player".to_owned()))
				                                                        .collect();

				wrs.push(WR { category: category.name.clone(), subcategory: Some(value.label), players: players, time: time });
			}
		} else {
			// No subcategories, just get runs.

			let mut leaderboard = try!(SPEEDRUNCOM_API_BASE.join(
				&format!("leaderboards/{}/category/{}", game.id, category.id)
			).map_err(|x| x.to_string()));

			leaderboard.query_pairs_mut()
				.append_pair("top", "1")
				.append_pair("embed", "players");

			let result = try!(client.get(leaderboard.as_str()).header(USERAGENT.clone()).send());
			let leaderboard: APILeaderboards = try!(serde_json::de::from_reader(result));

			let runs = leaderboard.data.runs;
			if runs.is_empty() {
				// Empty category.
				continue;
			}

			let time = Duration::from_millis((runs[0].run.times.primary_t * 1000f64) as u64);

			let players: Vec<String> = leaderboard.data.players.data.into_iter()
			                                                        .map(|x| x.names.map(|n| n.international)
			                                                                        .or(x.name)
			                                                                        .unwrap_or("nameless player".to_owned()))
			                                                        .collect();

			wrs.push(WR { category: category.name.clone(), subcategory: None, players: players, time: time });
		}
	}
	
	Ok((game.names.international, wrs))
}
