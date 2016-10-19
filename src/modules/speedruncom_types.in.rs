// Stuff for the /games API call.

#[derive(Deserialize, Debug)]
struct APIGames {
	data: Vec<APIGamesData>
}

#[derive(Deserialize, Debug)]
struct APIGamesData {
	id: String,
	names: APIGamesNames,
	categories: APIGamesCategories
}

#[derive(Deserialize, Debug)]
struct APIGamesNames {
	international: String
}

#[derive(Deserialize, Debug)]
struct APIGamesCategories {
	data: Vec<APIGamesCategoriesData>
}

#[derive(Deserialize, Debug)]
struct APIGamesCategoriesData {
	id: String,
	name: String,
	#[serde(rename="type")]
	type_: String,
	variables: APIGamesCategoriesVariables
}

#[derive(Deserialize, Debug)]
struct APIGamesCategoriesVariables {
	data: Vec<APIGamesCategoriesVariablesData>
}

#[derive(Deserialize, Debug)]
struct APIGamesCategoriesVariablesData {
	id: String,
	#[serde(rename="is-subcategory")]
	is_subcategory: bool,
	values: APIGamesCategoriesVariablesValues
}

#[derive(Deserialize, Debug)]
struct APIGamesCategoriesVariablesValues {
	values: BTreeMap<String, APIGamesCategoriesVariablesValuesValue>
}

#[derive(Deserialize, Debug)]
struct APIGamesCategoriesVariablesValuesValue {
	label: String
}

// Stuff for the /leaderboards API call.

#[derive(Deserialize, Debug)]
struct APILeaderboards {
	data: APILeaderboardsData
}

#[derive(Deserialize, Debug)]
struct APILeaderboardsData {
	runs: Vec<APILeaderboardsRun>,
	players: APILeaderboardsPlayers
}

#[derive(Deserialize, Debug)]
struct APILeaderboardsRun {
	run: APILeaderboardsRunRun
}

#[derive(Deserialize, Debug)]
struct APILeaderboardsRunRun {
	times: APILeaderboardsRunRunTimes
}

#[derive(Deserialize, Debug)]
struct APILeaderboardsRunRunTimes {
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
