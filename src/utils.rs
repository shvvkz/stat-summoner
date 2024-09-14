use chrono::{Utc, NaiveDateTime};
use crate::models::QUEUE_ID_MAP;
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
pub fn is_valid_game_mode(
    queue_id: i64
    ) -> bool {
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
pub fn time_since_game_ended(
    game_end_timestamp: u64
    ) -> String {
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
    ) -> (HashMap<String, serde_json::Value>, HashMap<String, serde_json::Value>) {
        if rank_info.get(0).unwrap_or(&default_rank).get("queueType").unwrap().as_str() == Some("RANKED_FLEX_SR") {
            let flex_rank = rank_info.get(0).unwrap_or(&default_rank).clone();
            let solo_rank = rank_info.get(1).unwrap_or(&default_rank).clone();
            (solo_rank, flex_rank)
        } else {
            let solo_rank = rank_info.get(0).unwrap_or(&default_rank).clone();
            let flex_rank = rank_info.get(1).unwrap_or(&default_rank).clone();
            (solo_rank, flex_rank)
        }
    }