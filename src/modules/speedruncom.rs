use bot::Bot;
use discord::model::Message;
use hyper;
use hyper::client::Client;
use hyper::header::UserAgent;
use module;
use std::collections::hash_map::HashMap;
use std::error;
use std::error::Error;
use std::fmt;
use std::time::Duration;
use serde_json;
use url::Url;

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
			format!("Couldn't communicate with https://www.speedrun.com. :( ({})", err.description())
		},
		Err(MyError::NoSuchGame) => {
			"There's no such game on speedrun.com! :O".to_string()
		},
		Err(err) => {
			format!("Something's broken. :/ ({})", err.description())
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

struct SubcategoryVariable {
	id: String,
	values: HashMap<String, String> // ID to name
}

struct Category {
	id: String,
	name: String,
	subcategory_variable: Option<SubcategoryVariable>
}

fn get_wrs(text: &str) -> Result<(String, Vec<WR>), MyError> {
	let mut games = SPEEDRUNCOM_API_BASE.join("games").unwrap();
	games.query_pairs_mut()
		.append_pair("name", text)
		.append_pair("embed", "categories.variables")
		.append_pair("max", "1");
		
	let client = Client::new();
	let result = try!(client.get(games.as_str()).header(USERAGENT.clone()).send());

	let json: serde_json::Value = try!(serde_json::from_reader(result));
	let data = try!(json.lookup("data")
		.and_then(|x| x.as_array())
		.ok_or("Couldn't get `data`"));

	if data.len() == 0 {
		return Err(MyError::NoSuchGame);
	}

	let ref game = data[0];
	let game_id = try!(game.lookup("id")
		.and_then(|x| x.as_str())
		.ok_or("Couldn't get `id`"));
	let game_name = try!(game.lookup("names.international")
		.and_then(|x| x.as_str())
		.ok_or("Couldn't get `names.international`"));

	let categories_data = try!(game.lookup("categories.data")
		.and_then(|x| x.as_array())
		.ok_or("Couldn't get `categories.data`"));

	let mut categories = Vec::new();

	for category in categories_data {
		let category_type = try!(category.lookup("type")
			.and_then(|x| x.as_str())
			.ok_or("Couldn't get `categories.data.type`"));

		if category_type == "per-game" {
			let id = try!(category.lookup("id")
				.and_then(|x| x.as_str())
				.ok_or("Couldn't get `categories.data.id`"));
			let name = try!(category.lookup("name")
				.and_then(|x| x.as_str())
				.ok_or("Couldn't get `categories.data.name`"));

			let variables = try!(category.lookup("variables.data")
				.and_then(|x| x.as_array())
				.ok_or("Couldn't get `categories.variables.data`"));

			let mut subcategory_variable = None;

			for variable in variables {
				if variable.lookup("is-subcategory").and_then(|x| x.as_bool()).unwrap_or(false) {
					let var_id = try!(variable.lookup("id")
						.and_then(|x| x.as_str())
						.ok_or("Couldn't get `variable.id`"));

					let values = try!(variable.lookup("values.choices")
						.and_then(|x| x.as_object())
						.ok_or("Couldn't get `variable.values.choices`"));

					if values.is_empty() {
						return Err(MyError::Custom("The subcategory variable doesn't have any valid values. o_O".to_owned()));
					}

					let mut temp = HashMap::new();
					for (val_id, val_name) in values {
						temp.insert(
							val_id.clone(),
							try!(val_name.as_str().map(|x| x.to_owned()).ok_or("One of the subcategories is not a string. o_O"))
						);
					}

					subcategory_variable = Some(SubcategoryVariable{
						id: var_id.to_owned(),
						values: temp
					});

					break;
				}
			}

			categories.push(Category{ id: id.to_owned(), name: name.to_owned(), subcategory_variable: subcategory_variable });
		}
	}

	if categories.is_empty() {
		return Err(MyError::Custom(format!("*{}* doesn't seem to have any categories. :/", game_name)));
	}

	let mut wrs = Vec::new();

	for category in categories {
		if let Some(subcategory_variable) = category.subcategory_variable {
			// Get runs for each subcategory value.

			for (val_id, val_name) in subcategory_variable.values {
				let mut leaderboard = try!(SPEEDRUNCOM_API_BASE.join(
					format!("leaderboards/{}/category/{}", game_id, category.id).as_str()
				).map_err(|x| x.to_string()));

				leaderboard.query_pairs_mut()
					.append_pair("top", "1")
					.append_pair("embed", "players")
					.append_pair(&format!("var-{}", subcategory_variable.id), &val_id);

				let result = try!(client.get(leaderboard.as_str()).header(USERAGENT.clone()).send());
				let json: serde_json::value::Value = try!(serde_json::from_reader(result));

				let runs = try!(json.lookup("data.runs")
					.and_then(|x| x.as_array())
					.ok_or("[leaderboard] Couldn't get `data.runs`"));

				if runs.is_empty() {
					// Empty subcategory.
					continue;
				}

				let run = try!(runs[0].lookup("run").ok_or("[leaderboard] Couldn't get `runs[0].run`"));

				let time_in_seconds = try!(run.lookup("times")
					.and_then(|x| x.lookup("primary_t"))
					.and_then(|x| x.as_f64())
					.ok_or("[leaderboard] Couldn't get `runs[0].run.times.primary_t"));
				let time = Duration::from_millis((time_in_seconds * 1000f64) as u64);

				let players_data = try!(json.lookup("data.players.data")
					.and_then(|x| x.as_array())
					.and_then(|x| if x.len() == 0 {
						None
					} else {
						Some(x)
					})
					.ok_or("[leaderboard] Couldn't get the players array"));

				let mut players = Vec::new();
				for player in players_data {
					let name = try!(player.lookup("names.international")
						.or(player.lookup("name"))
						.and_then(|x| x.as_str())
						.ok_or("[leaderboard] Couldn't get a player's name"));

					players.push(name.to_string());
				}

				wrs.push(WR { category: category.name.clone(), subcategory: Some(val_name), players: players, time: time });
			}
		} else {
			// No subcategories, just get runs.

			let mut leaderboard = try!(SPEEDRUNCOM_API_BASE.join(
				format!("leaderboards/{}/category/{}", game_id, category.id).as_str()
			).map_err(|x| x.to_string()));

			leaderboard.query_pairs_mut()
				.append_pair("top", "1")
				.append_pair("embed", "players");

			let result = try!(client.get(leaderboard.as_str()).header(USERAGENT.clone()).send());
			let json: serde_json::value::Value = try!(serde_json::from_reader(result));

			let runs = try!(json.lookup("data.runs")
				.and_then(|x| x.as_array())
				.ok_or("[leaderboard] Couldn't get `data.runs`"));

			if runs.is_empty() {
				// Empty category.
				continue;
			}

			let run = try!(runs[0].lookup("run").ok_or("[leaderboard] Couldn't get `runs[0].run`"));

			let time_in_seconds = try!(run.lookup("times")
				.and_then(|x| x.lookup("primary_t"))
				.and_then(|x| x.as_f64())
				.ok_or("[leaderboard] Couldn't get `runs[0].run.times.primary_t"));
			let time = Duration::from_millis((time_in_seconds * 1000f64) as u64);

			let players_data = try!(json.lookup("data.players.data")
				.and_then(|x| x.as_array())
				.and_then(|x| if x.len() == 0 {
					None
				} else {
					Some(x)
				})
				.ok_or("[leaderboard] Couldn't get the players array"));

			let mut players = Vec::new();
			for player in players_data {
				let name = try!(player.lookup("names.international")
					.or(player.lookup("name"))
					.and_then(|x| x.as_str())
					.ok_or("[leaderboard] Couldn't get a player's name"));

				players.push(name.to_string());
			}

			wrs.push(WR { category: category.name.clone(), subcategory: None, players: players, time: time });
		}
	}
	
	Ok((game_name.to_string(), wrs))
}
