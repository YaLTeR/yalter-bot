// Stuff for the /games API call.

#[derive(Deserialize, Debug)]
struct APIGames {
	data: Vec<APIGamesData>
}

#[derive(Deserialize, Debug)]
struct APIGamesData {
	id: String,
	names: APIGamesNames,
	categories: Option<APICategories>
}

#[derive(Deserialize, Debug)]
struct APIGamesNames {
	international: String
}

#[derive(Deserialize, Debug)]
struct APICategories {
	data: Vec<APICategoryData>
}

#[derive(Deserialize, Debug)]
struct APICategoryData {
	id: String,
	name: String,
	#[serde(rename="type")]
	type_: String,
	variables: APICategoryVariables
}

#[derive(Deserialize, Debug)]
struct APICategoryVariables {
	data: Vec<APICategoryVariablesData>
}

#[derive(Deserialize, Debug)]
struct APICategoryVariablesData {
	id: String,
	#[serde(rename="is-subcategory")]
	is_subcategory: bool,
	values: APICategoryVariablesValues
}

#[derive(Deserialize, Debug)]
struct APICategoryVariablesValues {
	values: BTreeMap<String, APICategoryVariablesValuesValue>
}

#[derive(Deserialize, Debug)]
struct APICategoryVariablesValuesValue {
	label: String
}

// Stuff for the /leaderboards API call.

#[derive(Deserialize, Debug)]
struct APILeaderboards {
	data: APILeaderboardsData
}

#[derive(Deserialize, Debug)]
struct APILeaderboardsData {
	runs: Vec<APIRun>,
	players: APILeaderboardsPlayers
}

#[derive(Deserialize, Debug)]
struct APIRun {
	place: u64,
	run: APIRunRun,
	category: Option<APICategory>
}

#[derive(Deserialize, Debug)]
struct APICategory {
	data: APICategoryData
}

#[derive(Deserialize, Debug)]
struct APIRunRun {
	times: APIRunRunTimes,
	values: BTreeMap<String, String> // Variable ID to value ID.
}

#[derive(Deserialize, Debug)]
struct APIRunRunTimes {
	primary_t: f64
}

#[derive(Deserialize, Debug)]
struct APILeaderboardsPlayers {
	data: Vec<APILeaderboardsPlayersData>
}

#[derive(Deserialize, Debug)]
struct APILeaderboardsPlayersData {
	names: Option<APILeaderboardsPlayersNames>,
	name: Option<String>
}

#[derive(Deserialize, Debug)]
struct APILeaderboardsPlayersNames {
	international: String
}

// Stuff for the /users API call.

#[derive(Deserialize, Debug)]
struct APIUsers {
	status: Option<u64>,
	data: Option<Vec<APIRun>>
}
