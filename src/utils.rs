use crate::models::constants::QUEUE_ID_MAP;
use crate::models::data::EmojiId;
use crate::models::region::Region;
use chrono::{NaiveDateTime, Utc};
use mongodb::bson::doc;
use mongodb::Collection;
use serde::de::value::Error;
use serde_json::Value;
use std::collections::HashMap;

/// ⚙️ **Function**: Checks if a given queue ID corresponds to a valid game mode.
///
/// This function verifies if the provided `queue_id` matches any valid game modes listed in the `QUEUE_ID_MAP`.
/// The `QUEUE_ID_MAP` contains a predefined set of game modes such as ranked, normal, and ARAM.
///
/// # Parameters:
/// - `queue_id`: The ID of the game queue (e.g., Ranked Solo/Duo, ARAM) to validate.
///
/// # Returns:
/// - `bool`: Returns `true` if the `queue_id` matches a valid game mode in `QUEUE_ID_MAP`, otherwise returns `false`.
///
/// # ⚠️ Notes:
/// - `QUEUE_ID_MAP` contains predefined game modes, so any queue ID not included in this map will return `false`.
/// - This function is useful for filtering out game modes that aren't relevant or valid for certain statistics (e.g., custom games).
///
/// # Example:
/// ```rust
/// let is_valid = is_valid_game_mode(420);  // Ranked Solo/Duo
/// if is_valid {
///     println!("This is a valid game mode.");
/// }
/// ```
///
/// If `queue_id` is valid, such as `420` for Ranked Solo/Duo, the result will be:
/// ```text
/// true
/// ```
pub fn is_valid_game_mode(queue_id: i64) -> bool {
    QUEUE_ID_MAP.iter().any(|&(id, _)| id == queue_id)
}

/// ⚙️ **Function**: Calculates the time elapsed since a game ended and returns it as a human-readable string.
///
/// This function computes the duration between the game's end timestamp and the current time. It returns a string
/// representing how much time has passed, formatted in seconds, minutes, hours, days, months, or years, depending on the duration.
///
/// # Parameters:
/// - `game_end_timestamp`: A UNIX timestamp (in milliseconds) representing when the game ended.
///
/// # Returns:
/// - `String`: A human-readable string representing how long ago the game ended (e.g., "5 minutes ago", "2 hours ago").
///
/// # ⚠️ Notes:
/// - The function converts the timestamp from milliseconds to seconds before performing the calculation.
/// - If the duration is less than 60 seconds, the result will be in seconds. If it's less than 24 hours, the result will be in minutes or hours, and so on.
///
/// # Example:
/// ```rust
/// let time_elapsed = time_since_game_ended(1625000000000);
/// println!("{}", time_elapsed);  // Output: "5 hours ago"
/// ```
///
/// The resulting string will vary depending on the duration since the game ended:
/// ```text
/// "2 minutes ago"
/// "5 days ago"
/// "1 year ago"
/// ```
pub fn time_since_game_ended(game_end_timestamp: u64) -> String {
    let game_end_time = NaiveDateTime::from_timestamp_opt((game_end_timestamp / 1000) as i64, 0)
        .expect("Invalid timestamp");
    let now = Utc::now().naive_utc();
    let duration = now.signed_duration_since(game_end_time);

    if duration.num_seconds() < 60 {
        format!("{} seconds ago", duration.num_seconds())
    } else if duration.num_minutes() < 60 {
        format!("{} minutes ago", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{} hours ago", duration.num_hours())
    } else if duration.num_days() < 30 {
        format!("{} days ago", duration.num_days())
    } else if duration.num_days() < 365 {
        format!("{} months ago", duration.num_days() / 30)
    } else {
        format!("{} years ago", duration.num_days() / 365)
    }
}

/// ⚙️ **Function**: Determines Solo/Duo and Flex ranks from rank information.
///
/// This function analyzes a list of rank information and determines the Solo/Duo and Flex ranks based on the provided data.
/// It checks the `queueType` field in the rank data to distinguish between Solo/Duo and Flex ranks. If no rank data is available
/// for a specific queue, it returns a default rank.
///
/// # Parameters:
/// - `rank_info`: A vector containing rank information in the form of a list of `HashMap<String, serde_json::Value>`. Each `HashMap` represents a rank type with various rank data, including `queueType`.
/// - `default_rank`: A default rank (`HashMap<String, serde_json::Value>`) to return if the corresponding rank information is missing.
///
/// # Returns:
/// - `(HashMap<String, serde_json::Value>, HashMap<String, serde_json::Value>)`: A tuple containing two `HashMap` values, where the first element is the Solo/Duo rank and the second is the Flex rank.
///
/// # ⚠️ Notes:
/// - The function expects the `queueType` field to differentiate between "RANKED_SOLO_5x5" and "RANKED_FLEX_SR".
/// - If rank information is missing for either Solo/Duo or Flex, the function returns the `default_rank` for that rank type.
/// - It assumes that the first element in the `rank_info` corresponds to Flex if `queueType` is "RANKED_FLEX_SR", otherwise it assumes the first element is Solo/Duo.
///
/// # Example:
/// ```rust
/// let rank_info = vec![
///     hashmap! { "queueType".to_string() => serde_json::Value::String("RANKED_FLEX_SR".to_string()) },
///     hashmap! { "queueType".to_string() => serde_json::Value::String("RANKED_SOLO_5x5".to_string()) }
/// ];
/// let default_rank = hashmap! { "tier".to_string() => serde_json::Value::String("UNRANKED".to_string()) };
///
/// let (solo_rank, flex_rank) = determine_solo_flex(&rank_info, &default_rank);
///
/// assert_eq!(solo_rank.get("queueType").unwrap(), "RANKED_SOLO_5x5");
/// assert_eq!(flex_rank.get("queueType").unwrap(), "RANKED_FLEX_SR");
/// ```
///
/// In this example, the function will correctly identify the Solo/Duo and Flex ranks based on the `queueType` values provided in `rank_info`.
pub fn determine_solo_flex(
    rank_info: &Vec<HashMap<String, serde_json::Value>>,
    default_rank: &HashMap<String, serde_json::Value>,
) -> (
    HashMap<String, serde_json::Value>,
    HashMap<String, serde_json::Value>,
) {
    if rank_info
        .get(0)
        .unwrap_or(&default_rank)
        .get("queueType")
        .unwrap()
        .as_str()
        == Some("RANKED_FLEX_SR")
    {
        let flex_rank = rank_info.get(0).unwrap_or(&default_rank).clone();
        let solo_rank = rank_info.get(1).unwrap_or(&default_rank).clone();
        (solo_rank, flex_rank)
    } else {
        let solo_rank = rank_info.get(0).unwrap_or(&default_rank).clone();
        let flex_rank = rank_info.get(1).unwrap_or(&default_rank).clone();
        (solo_rank, flex_rank)
    }
}

/// ⚙️ **Function**: Converts a `Region` enum into its corresponding server string representation.
///
/// This function takes a reference to a `Region` enum and returns a string representing the
/// appropriate server for that region. It maps each region to its official server shorthand,
/// which is used in API requests to the Riot Games platform.
///
/// # Parameters:
/// - region: A reference to a `Region` enum, representing the different League of Legends regions.
///
/// # Returns:
/// - `String`: A string that corresponds to the server shorthand for the provided region.
///
/// # Supported Regions:
/// - **NA**: Maps to "na1"
/// - **EUW**: Maps to "euw1"
/// - **EUNE**: Maps to "eun1"
/// - **KR**: Maps to "kr"
/// - **BR**: Maps to "br1"
/// - **LAN**: Maps to "la1"
/// - **LAS**: Maps to "la2"
/// - **OCE**: Maps to "oc1"
/// - **RU**: Maps to "ru"
/// - **TR**: Maps to "tr1"
/// - **JP**: Maps to "jp1"
///
/// # Example:
/// This function can be used when you need to retrieve the corresponding server for a specific region.
///
/// ```rust
/// let server = region_to_string(&Region::NA);
/// assert_eq!(server, "na1");
/// ```
pub fn region_to_string(region: &Region) -> String {
    (match region {
        Region::NA => "na1",
        Region::EUW => "euw1",
        Region::EUNE => "eun1",
        Region::KR => "kr",
        Region::BR => "br1",
        Region::LAN => "la1",
        Region::LAS => "la2",
        Region::OCE => "oc1",
        Region::RU => "ru",
        Region::TR => "tr1",
        Region::JP => "jp1",
    })
    .to_string()
}

/// ⚙️ **Function**: Converts a duration in seconds into a tuple representing minutes and seconds.
///
/// This function takes a duration in seconds and converts it into a more human-readable format, returning
/// the number of minutes and the remaining seconds as a tuple of strings. This is useful for displaying
/// game durations or other time intervals in a clear way.
///
/// # Parameters:
/// - `seconds`: A `u64` value representing the total duration in seconds.
///
/// # Returns:
/// - `(String, String)`: A tuple where the first value is the number of minutes, and the second value is the number of seconds (formatted as two digits if necessary).
///
/// # Example:
/// This function is useful when converting raw game duration data into a more readable format.
///
/// ```rust
/// let (minutes, seconds) = seconds_to_time(645);
/// assert_eq!(minutes, "10");
/// assert_eq!(seconds, "45");
/// ```
/// In this example, 645 seconds are converted to 10 minutes and 45 seconds.
///
/// # Notes:
/// - The seconds part is always formatted as two digits. For example, if the input is 610 seconds (10 minutes and 10 seconds), the result will be `"10", "10"`.
pub fn seconds_to_time(seconds: u64) -> (String, String) {
    let game_duration_minutes = seconds / 60;
    let game_duration_seconds = seconds % 60;
    let game_duration_seconds_str: String;
    if game_duration_seconds < 10 {
        game_duration_seconds_str = format!("0{}", game_duration_seconds);
    } else {
        game_duration_seconds_str = game_duration_seconds.to_string();
    }
    (game_duration_minutes.to_string(), game_duration_seconds_str)
}
/// ⚙️ **Function**: Retrieves a custom emoji string based on role and name from a MongoDB collection.
///
/// This asynchronous function searches a MongoDB collection for a custom emoji corresponding to a specific role and name.
/// If found, it formats the emoji in a string compatible with Discord. If not found, it returns the provided name as a fallback.
///
/// # Parameters:
/// - `collection`: A MongoDB `Collection<EmojiId>` containing the emoji mappings, where each document maps a role and name to an emoji ID.
/// - `role`: A string slice representing the role of the emoji (e.g., "position", "champions").
/// - `name`: A string slice representing the name of the emoji (e.g., "TOP", "JUNGLE", champion names).
///
/// # Returns:
/// - `Result<String, Error>`: Returns a `Result` containing the formatted emoji string (if found) or the name as a fallback.
///   In case of errors, it logs the error and returns the name.
///
/// # Example:
/// This function can be used to retrieve custom emojis for roles or champions when creating embeds for Discord:
///
/// ```rust
/// let emoji = get_emoji(collection_emojis, "position", "TOP").await?;
/// println!("The emoji for TOP is: {}", emoji);
/// ```
///
/// # Notes:
/// - The function creates a MongoDB filter to search for the emoji based on the role and name fields.
/// - If an emoji is found, it formats the emoji string in the form `<:name:id>`, which is recognized by Discord.
/// - If no emoji is found or an error occurs, the function returns the `name` string as a fallback and logs any errors encountered during the search.
pub async fn get_emoji(
    collection: Collection<EmojiId>,
    role: &str,
    name: &str,
) -> Result<String, Error> {
    let filter = doc! { "role": role, "name": name };

    match collection.find_one(filter).await {
        Ok(Some(emoji_id)) => {
            let emoji_str = format!("<:{}:{}>", name, emoji_id.id_emoji);
            Ok(emoji_str)
        }
        Ok(None) => Ok(name.to_string()),
        Err(e) => {
            eprintln!("Erreur lors de la recherche de l'emoji: {:?}", e);
            Ok(name.to_string())
        }
    }
}

/// ⚙️ **Function**: Retrieves the game mode corresponding to a given queue ID.
///
/// This function looks up the game mode based on a provided `queue_id` using a predefined mapping (`QUEUE_ID_MAP`)
/// of queue IDs to game modes. If the `queue_id` is not found in the map, it returns "Unknown".
///
/// # Parameters:
/// - `queue_id`: An `i64` representing the queue ID for which the game mode is being queried.
///
/// # Returns:
/// - `&'static str`: Returns a string slice representing the game mode name corresponding to the queue ID, or "Unknown" if the queue ID is not found.
///
/// # Example:
/// This function can be used to retrieve the game mode based on the queue ID returned from match data:
///
/// ```rust
/// let queue_id = 420; // Example queue ID for Ranked Solo/Duo
/// let game_mode = get_game_mode(queue_id);
/// println!("The game mode is: {}", game_mode);
/// ```
///
/// # Notes:
/// - The function iterates over the `QUEUE_ID_MAP`, a predefined list of tuples mapping queue IDs to game modes.
/// - If the queue ID is found in the map, the corresponding game mode is returned immediately.
/// - If the queue ID is not found, the function defaults to returning "Unknown".
pub fn get_game_mode(queue_id: i64) -> &'static str {
    for &(id, mode) in QUEUE_ID_MAP.iter() {
        if id == queue_id {
            return mode;
        }
    }
    "Unknown"
}

pub fn get_champion_names(dd_json: &Value) -> Vec<String> {
    // Obtenir le champ "data" qui contient les champions
    let data = &dd_json["data"];

    // Vérifier que "data" est un objet
    if let Some(champion_map) = data.as_object() {
        // Itérer sur les valeurs (données des champions)
        champion_map
            .values()
            .filter_map(|champion| champion["name"].as_str().map(|s| s.to_string()))
            .collect()
    } else {
        vec![]
    }
}

pub fn get_champion_id(dd_json: &Value, name: &str) -> Option<String> {
    let data = &dd_json["data"];
    if let Some(champion_map) = data.as_object() {
        for (_, champion_value) in champion_map {
            // Obtenir le nom du champion
            if let Some(champion_name) = champion_value["name"].as_str() {
                if champion_name.eq_ignore_ascii_case(name) {
                    if let Some(champion_id) = champion_value["id"].as_str() {
                        return Some(champion_id.to_string());
                    }
                }
            }
        }
    }
    // Si aucun champion correspondant n'est trouvé, retourner None
    None
}
