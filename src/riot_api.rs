use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use crate::models::Error;

/// ⚙️ **Function**: Fetches the player's PUUID (Player Unique Identifier) from the Riot API.
///
/// This function sends a request to the Riot API to retrieve the PUUID of a player based on their in-game name and tag line.
/// The PUUID is a globally unique identifier used across Riot's systems to identify players.
///
/// # Parameters:
/// - `client`: An instance of the `reqwest::Client` used to send HTTP requests.
/// - `game_name_space`: The player's in-game name (spaces should be replaced with `%20`).
/// - `tag_line`: The player's tag line, typically a four-digit number associated with their Riot account.
/// - `riot_api_key`: The API key used to authenticate the request with the Riot API.
///
/// # Returns:
/// - `Result<String, Error>`: The PUUID as a string if the request is successful, or an error if the player does not exist or the request fails.
///
/// # ⚠️ Notes:
/// - If the player does not exist or the information provided is incorrect, the function will return an error message.
/// - The PUUID is a critical identifier that is used in subsequent requests to fetch match and player data.
///
/// # Example:
/// ```rust
/// let puuid = get_puuid(&client, "Faker", "1234", riot_api_key).await?;
/// ```
///
/// The resulting `puuid` will be a unique string identifier, such as:
/// ```text
/// "abcd1234-efgh5678-ijkl91011-mnop1213"
/// ```
pub async fn get_puuid(
    client: &Client,
    game_name_space: &str,
    tag_line: &str,
    riot_api_key: &str
    ) -> Result<String, Error> {
        let puuid_url = format!(
            "https://europe.api.riotgames.com/riot/account/v1/accounts/by-riot-id/{}/{}?api_key={}",
            game_name_space, tag_line, riot_api_key
        );

        let response = client.get(&puuid_url).send().await?;
        let puuid_json: Value = response.json().await?;
        let puuid = puuid_json.get("puuid").and_then(Value::as_str).unwrap_or("").to_string();

        if puuid.is_empty() {
            Err("The player could not be found. Please verify that the region, game name, and tag line you provided are correct, and try again.".into())
        } else {
            Ok(puuid)
        }
    }

/// ⚙️ **Function**: Retrieves recent match IDs for a given player using their PUUID.
///
/// This function sends a request to the Riot API to fetch the IDs of the player's recent matches based on their PUUID. 
/// The match IDs are used to fetch detailed match data in subsequent API requests.
///
/// # Parameters:
/// - `client`: An instance of the `reqwest::Client` used to send HTTP requests.
/// - `puuid`: The player's unique PUUID (Player Unique Identifier), used to identify them across Riot's services.
/// - `riot_api_key`: The API key used to authenticate the request with the Riot API.
/// - `nb_match`: The number of recent matches to retrieve.
///
/// # Returns:
/// - `Result<Vec<String>, Error>`: A vector containing the IDs of the player's recent matches, or an error if the request fails.
///
/// # ⚠️ Notes:
/// - The function retrieves the most recent 5 matches by default. This can be adjusted in the API URL if necessary.
/// - Each match ID is a unique string that can be used to query detailed match information.
/// - The `puuid` must be valid for the request to return match IDs successfully.
///
/// # Example:
/// ```rust
/// let match_ids = get_matchs_id(&client, "abcd1234-efgh5678-ijkl91011-mnop1213", riot_api_key, 5).await?;
/// ```
///
/// The resulting `match_ids` will be a vector of strings, such as:
/// ```text
/// ["EUW1_1234567890", "EUW1_0987654321", "EUW1_2345678901"]
/// ```
pub async fn get_matchs_id(
    client: &Client,
    puuid: &str,
    riot_api_key: &str,
    nb_match: u32
    ) -> Result<Vec<String>, Error> {
        let matchs_url = format!(
            "https://europe.api.riotgames.com/lol/match/v5/matches/by-puuid/{}/ids?&count={}&api_key={}",
            puuid, nb_match.to_string(),  riot_api_key
        );

        let response = client.get(&matchs_url).send().await?;
        let matchs_id: Vec<String> = response.json().await?;
        Ok(matchs_id)
    }



/// ⚙️ **Function**: Fetches the summoner ID for a player using their PUUID.
///
/// This function sends a request to the Riot API to retrieve the summoner ID of a player, which is used for further 
/// requests related to match history and player information. The summoner ID is specific to the player's account in 
/// the given region.
///
/// # Parameters:
/// - `client`: An instance of the `reqwest::Client` used to send HTTP requests.
/// - `region_str`: A string representing the region (e.g., `euw1`, `na1`, `kr`) where the player's account is located.
/// - `puuid`: The player's unique PUUID (Player Unique Identifier), which is used to identify them across Riot's services.
/// - `riot_api_key`: The API key used to authenticate the request with the Riot API.
///
/// # Returns:
/// - `Result<String, Error>`: The summoner ID as a string if the request is successful, or an error if the player cannot be found or the request fails.
///
/// # ⚠️ Notes:
/// - If the summoner ID cannot be retrieved (e.g., due to incorrect region or PUUID), the function logs an error and returns an appropriate message.
/// - The summoner ID is required for many other API requests, such as retrieving ranked data and match history.
///
/// # Example:
/// ```rust
/// let summoner_id = get_summoner_id(&client, "euw1", "abcd1234-efgh5678-ijkl91011-mnop1213", riot_api_key).await?;
/// ```
///
/// The resulting `summoner_id` will be a unique string, such as:
/// ```text
/// "abcdef1234567890abcdef1234567890"
/// ```
pub async fn get_summoner_id(
    client: &Client,
    region_str: &str,
    puuid: &str,
    riot_api_key: &str
    ) -> Result<String, Error> {
        let summoner_url = format!(
            "https://{}.api.riotgames.com/lol/summoner/v4/summoners/by-puuid/{}?api_key={}",
            region_str, puuid, riot_api_key
        );

        let response = client.get(&summoner_url).send().await?;
        let summoner_json: Value = response.json().await?;
        let summoner_id = summoner_json.get("id").and_then(Value::as_str).unwrap_or("").to_string();
        if summoner_id.is_empty() {
            Err("Error retrieving summoner ID. Please verify that the region, game name, and tag line you provided are correct, and try again.".into())
        } else {
            Ok(summoner_id)
        }
    }

/// ⚙️ **Function**: Fetches ranked information for a player using their summoner ID.
///
/// This function sends a request to the Riot API to retrieve ranked information for a player, including their rank, 
/// division, league points (LP), wins, and losses in different game modes (e.g., Solo/Duo, Flex).
///
/// # Parameters:
/// - `client`: An instance of the `reqwest::Client` used to send HTTP requests.
/// - `region_str`: A string representing the region (e.g., `euw1`, `na1`, `kr`) where the player's account is located.
/// - `summoner_id`: The unique summoner ID of the player, used to identify them in the ranked ladder.
/// - `riot_api_key`: The API key used to authenticate the request with the Riot API.
///
/// # Returns:
/// - `Result<Vec<HashMap<String, Value>>, Error>`: A vector of `HashMap` objects containing ranked information for each game mode (Solo/Duo, Flex) or an error if the request fails.
///
/// # ⚠️ Notes:
/// - The returned ranked information includes game modes like Solo/Duo and Flex, along with details such as tier, rank, LP, wins, and losses.
/// - The function returns an empty vector if no ranked data is found for the player in the specified region.
///
/// # Example:
/// ```rust
/// let rank_info = get_rank_info(&client, "euw1", "abcdef1234567890abcdef1234567890", riot_api_key).await?;
/// ```
///
/// The resulting `rank_info` will contain ranked data for different game modes, such as:
/// ```json
/// [
///   {
///     "queueType": "RANKED_SOLO_5x5",
///     "tier": "Gold",
///     "rank": "IV",
///     "leaguePoints": 50,
///     "wins": 30,
///     "losses": 20
///   },
///   {
///     "queueType": "RANKED_FLEX_SR",
///     "tier": "Silver",
///     "rank": "II",
///     "leaguePoints": 75,
///     "wins": 15,
///     "losses": 10
///   }
/// ]
/// ```
pub async fn get_rank_info(
    client: &Client,
    region_str: &str,
    summoner_id: &str,
    riot_api_key: &str
    ) -> Result<Vec<HashMap<String, Value>>, Error> {
        let rank_url = format!(
            "https://{}.api.riotgames.com/lol/league/v4/entries/by-summoner/{}?api_key={}",
            region_str, summoner_id, riot_api_key
        );
        let response = client.get(&rank_url).send().await?;
        Ok(response.json().await?)
    }

/// ⚙️ **Function**: Retrieves the top 10 champions for a player based on champion mastery.
///
/// This function sends a request to the Riot API to fetch the player's top 10 champions based on their mastery score.
/// The information returned includes champion mastery level and points for each champion.
///
/// # Parameters:
/// - `client`: An instance of the `reqwest::Client` used to send HTTP requests.
/// - `puuid`: The player's unique PUUID (Player Unique Identifier), used to identify the player in Riot's systems.
/// - `region`: A string representing the region (e.g., `euw1`, `na1`, `kr`) where the player's account is located.
/// - `riot_api_key`: The API key used to authenticate the request with the Riot API.
///
/// # Returns:
/// - `Result<Vec<HashMap<String, Value>>, Error>`: A vector of `HashMap` objects, where each entry contains champion mastery details, or an error if the request fails.
///
/// # ⚠️ Notes:
/// - The function returns the top 10 champions based on mastery points, but this count can be adjusted in the API URL.
/// - The information includes each champion's ID, mastery level, and mastery points.
/// - The function requires a valid `puuid` and `region` for the request to succeed.
///
/// # Example:
/// ```rust
/// let top_champions = get_champions(&client, "abcd1234-efgh5678-ijkl91011-mnop1213", "euw1", riot_api_key).await?;
/// ```
///
/// The resulting `top_champions` vector will contain data like:
/// ```json
/// [
///   {
///     "championId": 157,
///     "championLevel": 7,
///     "championPoints": 500000
///   },
///   {
///     "championId": 238,
///     "championLevel": 6,
///     "championPoints": 350000
///   }
/// ]
/// ```
pub async fn get_champions(
    client: &Client,
    puuid: &str,
    region: &str,
    riot_api_key: &str
    ) -> Result<Vec<HashMap<String, Value>>, Error> {
        let champions_url = format!(
            "https://{}.api.riotgames.com/lol/champion-mastery/v4/champion-masteries/by-puuid/{}/top?count=10&api_key={}",
            region, puuid, riot_api_key
        );
        let response = client.get(&champions_url).send().await?;
        Ok(response.json().await?)
    }

/// ⚙️ **Function**: Fetches the latest champion data from Data Dragon (Riot's official static data service).
///
/// This function sends a request to Data Dragon to retrieve the latest static data about League of Legends champions,
/// such as champion names, IDs, and related information. The data is returned as a JSON object and can be used to map
/// champion IDs to their names and other static details.
///
/// # Returns:
/// - `Result<Value, Error>`: A JSON object containing champion data if the request is successful, or an error if the request fails.
///
/// # ⚠️ Notes:
/// - The request fetches champion data in French (`fr_FR`), but the language can be changed by modifying the URL.
/// - Data Dragon provides static, versioned data, which means this data may not always be up to date with the latest game patches unless the URL version is updated.
///
/// # Example:
/// ```rust
/// let dd_json = open_dd_json().await?;
/// ```
///
/// The resulting `dd_json` will contain champion data like:
/// ```json
/// {
///   "type": "champion",
///   "format": "standAloneComplex",
///   "data": {
///     "Aatrox": {
///       "id": "Aatrox",
///       "key": "266",
///       "name": "Aatrox",
///       ...
///     },
///     ...
///   }
/// }
/// ```
pub async fn open_dd_json(
    ) -> Result<Value, Error> {
        let dd_json = reqwest::get("https://ddragon.leagueoflegends.com/cdn/14.18.1/data/fr_FR/champion.json").await?.json().await?;
        Ok(dd_json)
    }

/// ⚙️ **Function**: Fetches detailed information about a specific match using the match ID.
///
/// This function sends a request to the Riot API to retrieve detailed information about a match, such as 
/// participants, game duration, result (win/loss), and other match-related statistics. The match data is returned 
/// as a JSON object.
///
/// # Parameters:
/// - `client`: An instance of the `reqwest::Client` used to send HTTP requests.
/// - `match_id`: The unique ID of the match to retrieve. Each match is assigned a unique identifier in the Riot API.
/// - `riot_api_key`: The API key used to authenticate the request with the Riot API.
///
/// # Returns:
/// - `Result<Value, Error>`: A JSON object containing detailed match data if the request is successful, or an error if the request fails.
///
/// # ⚠️ Notes:
/// - The match data includes detailed statistics for each participant, including champion played, kills, deaths, assists, and more.
/// - The `match_id` must be valid for the request to succeed; otherwise, the function returns an error.
///
/// # Example:
/// ```rust
/// let match_info = get_matchs_info(&client, "EUW1_1234567890", riot_api_key).await?;
/// ```
///
/// The resulting `match_info` will contain detailed match data like:
/// ```json
/// {
///   "metadata": {
///     "dataVersion": "2",
///     "matchId": "EUW1_1234567890"
///   },
///   "info": {
///     "gameCreation": 1625000000000,
///     "gameDuration": 1800,
///     "participants": [
///       {
///         "summonerName": "Faker",
///         "championName": "Yasuo",
///         "kills": 10,
///         "deaths": 2,
///         "assists": 8,
///         ...
///       }
///     ]
///   }
/// }
/// ```
pub async fn get_matchs_info(
    client: &Client,
    match_id: &str,
    riot_api_key: &str
    ) -> Result<Value, Error> {
        let matchs_info_url = format!(
            "https://europe.api.riotgames.com/lol/match/v5/matches/{}?api_key={}",
            match_id, riot_api_key
        );
        let response = client.get(&matchs_info_url).send().await?;
        let matchs_info: Value = response.json().await?;
        Ok(matchs_info)
    }