use crate::{
    models::{
        data::{CoreBuildData, EmojiId, RunesData, SummonerFollowedData},
        error::Error,
    },
    riot_api::{get_matchs_id, get_matchs_info},
    utils::*,
};
use chrono::Utc;
use futures::StreamExt;
use mongodb::{bson::doc, Collection};
use poise::serenity_prelude::{self as serenity, CreateEmbed, CreateMessage, Http};
use regex::Regex;
use select::document::Document;
use select::predicate::{Class, Name};
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};

/// ⚙️ **Function**: Extracts relevant match details for a given summoner from the match information.
///
/// This function retrieves detailed information about a match, focusing on the summoner specified by their `summoner_id`.
/// It validates the game mode, identifies the summoner's performance, and compares their stats with the enemy team in each role (TOP, JUNGLE, MIDDLE, BOTTOM, UTILITY).
///
/// # Parameters:
/// - `match_info`: A reference to a `Value` (from the `serde_json` crate) containing the entire match data fetched from the Riot API.
/// - `summoner_id`: A string slice representing the summoner's ID, used to locate their stats in the match data.
///
/// # Returns:
/// - `Option<Value>`: Returns a JSON object containing the match result (Victory or Defeat) and detailed role-based stats comparisons, or `None` if the game mode is invalid or the data is not available.
///
/// # Example:
/// This function is typically used to extract and format match details for reporting to a Discord channel:
///
/// ```rust
/// let match_details = get_match_details(&match_info, summoner_id);
/// if let Some(details) = match_details {
///     // Process match details for further use
/// }
/// ```
///
/// # Notes:
/// - The function first checks if the game mode is valid using `is_valid_game_mode`. If the game mode is invalid, the function returns `None`.
/// - It then searches for the summoner in the participants list and identifies their team and match result (Victory or Defeat).
/// - The function separates the participants into two teams (the summoner's team and the enemy team) and compares stats for each role.
/// - It generates JSON-formatted role matchups comparing stats between the summoner's team and their opponents for each role.
pub fn get_match_details(match_info: &Value, summoner_id: &str) -> Option<Value> {
    let queue_id = match_info["info"]["queueId"].as_i64().unwrap_or(-1);
    let (game_duration_minutes, game_duration_secondes) =
        seconds_to_time(match_info["info"]["gameDuration"].as_u64().unwrap_or(0));
    let game_duration_string = format!("{}:{}", game_duration_minutes, game_duration_secondes);
    // utilise QUEUE_ID_MAP qui est une constante dans models/constants.rs qui contient une liste de game modes faisant correspondre id -> game mode en str
    let game_mode = get_game_mode(queue_id);

    let participants = match_info["info"]["participants"].as_array()?;
    let participant = participants
        .iter()
        .find(|p| p["summonerId"].as_str().unwrap_or("") == summoner_id)?;

    let team_id = participant["teamId"].as_i64().unwrap_or(0);
    let win = participant["win"].as_bool().unwrap_or(false);
    let game_result = if win { "Victory" } else { "Defeat" };

    let mut team_participants: HashMap<String, &Value> = HashMap::new();
    let mut enemy_participants: HashMap<String, &Value> = HashMap::new();

    for p in participants {
        let position = p["teamPosition"].as_str().unwrap_or("UNKNOWN").to_string();
        let p_team_id = p["teamId"].as_i64().unwrap_or(0);
        if p_team_id == team_id {
            team_participants.insert(position.clone(), p);
        } else {
            enemy_participants.insert(position.clone(), p);
        }
    }

    let roles = vec!["TOP", "JUNGLE", "MIDDLE", "BOTTOM", "UTILITY"];

    let mut matchups = Vec::new();

    for role in roles {
        if let (Some(team_p), Some(enemy_p)) =
            (team_participants.get(role), enemy_participants.get(role))
        {
            let team_stats = extract_participant_stats(team_p);
            let enemy_stats = extract_participant_stats(enemy_p);

            let matchup = serde_json::json!({
                "role": role,
                "team": team_stats,
                "enemy": enemy_stats
            });

            matchups.push(matchup);
        }
    }

    Some(serde_json::json!({
        "gameMode": game_mode,
        "gameResult": game_result,
        "gameDuration": game_duration_string,
        "matchups": matchups
    }))
}

/// ⚙️ **Function**: Creates a detailed embed for a player's match performance in Discord.
///
/// This asynchronous function generates a `CreateEmbed` object that includes detailed statistics
/// of a player's match, such as the game mode, result, duration, and a role-by-role comparison
/// of the player's team versus the enemy team. The embed is enriched with emojis and formatted
/// data to make it visually appealing for Discord.
///
/// # Parameters:
/// - `info_json`: A reference to a `Value` (from the `serde_json` crate) containing the match data fetched from the Riot API.
/// - `player_name`: A string slice representing the player's name, used for the embed's title.
/// - `collection_emoji`: A MongoDB `Collection` containing emoji mappings, which are used to enhance the embed with role and champion-specific emojis.
///
/// # Returns:
/// - `CreateEmbed`: Returns a `CreateEmbed` object containing the formatted match data, including role-based comparisons and game metadata, ready to be sent to a Discord channel.
///
/// # Example:
/// This function is typically used to send detailed match information to a Discord channel:
///
/// ```rust
/// let embed = create_embed_loop(&info_json, "PlayerName", collection_emoji).await;
/// // Send the embed to a Discord channel using your bot's message-sending logic
/// ```
///
/// # Notes:
/// - The function begins by extracting key game metadata (game mode, result, and duration) from `info_json`.
/// - Based on the match result, it selects appropriate emojis and colors for the embed.
/// - The function then constructs the title and proceeds to iterate over the available role-based matchups, comparing the stats of the player's team with the enemy team for each role (TOP, JUNGLE, MIDDLE, BOTTOM, UTILITY).
/// - Role and champion names are replaced by their corresponding emojis from the `collection_emoji`, retrieved using the `get_emoji` function.
/// - The function formats team and enemy stats (kills, deaths, assists, CS, gold, vision score) for each role and adds them as fields in the embed.
/// - It returns a fully constructed `CreateEmbed` ready to be sent in a Discord message.
pub async fn create_embed_loop(
    info_json: &Value,
    player_name: &str,
    collection_emoji: Collection<EmojiId>,
) -> CreateEmbed {
    let game_mode = info_json["gameMode"].as_str().unwrap_or("Unknown");
    let game_result = info_json["gameResult"].as_str().unwrap_or("Unknown");
    let game_duration = info_json["gameDuration"].as_str().unwrap_or("00:00");
    let game_result_emoji = if game_result == "Victory" {
        "🏆"
    } else {
        "❌"
    };
    let game_result_thumbnail = if game_result == "Victory" {
        "https://i.postimg.cc/CxwjnWVk/pngegg.png"
    } else {
        "https://i.postimg.cc/XJBF0WwS/pngwing-com.png"
    };
    let color: i32 = if game_result == "Victory" {
        0x00ff00
    } else {
        0xff0000
    };

    // Construct the embed title
    let title = format!(
        "**{}** - **{}: {} {} - {} **",
        player_name, game_mode, game_result, game_result_emoji, game_duration
    );

    let roles_order = ["TOP", "JUNGLE", "MIDDLE", "BOTTOM", "UTILITY"];
    let mut matchups_by_role = std::collections::HashMap::new();
    if let Some(matchups) = info_json["matchups"].as_array() {
        for matchup in matchups {
            if let Some(role) = matchup["role"].as_str() {
                matchups_by_role.insert(role.to_uppercase(), matchup);
            }
        }
    }
    let mut embed = CreateEmbed::new()
        .title(title)
        .color(color)
        .thumbnail(game_result_thumbnail);

    for role in &roles_order {
        if let Some(matchup) = matchups_by_role.get(&role.to_uppercase()) {
            let team_player = &matchup["team"];
            let enemy_player = &matchup["enemy"];
            let role_label = match *role {
                "TOP" => format!(
                    "**{} TOP**\n",
                    get_emoji(collection_emoji.clone(), "position", "TOP")
                        .await
                        .unwrap_or("🔼".to_string())
                ),
                "JUNGLE" => format!(
                    "**{} JUNGLE**\n",
                    get_emoji(collection_emoji.clone(), "position", "JUNGLE")
                        .await
                        .unwrap_or("🌲".to_string())
                ),
                "MIDDLE" => format!(
                    "**{} MIDDLE**\n",
                    get_emoji(collection_emoji.clone(), "position", "MIDDLE")
                        .await
                        .unwrap_or("🛣️".to_string())
                ),
                "BOTTOM" => format!(
                    "**{} BOTTOM**\n",
                    get_emoji(collection_emoji.clone(), "position", "BOTTOM")
                        .await
                        .unwrap_or("🔽".to_string())
                ),
                "UTILITY" => format!(
                    "**{} SUPPORT**\n",
                    get_emoji(collection_emoji.clone(), "position", "SUPPORT")
                        .await
                        .unwrap_or("🛡️".to_string())
                ),
                _ => "**UNKNOWN**\n".to_string(),
            };

            // Team player stats
            let team_stats = format!(
                "{} **{}**\nK/D/A: **{}/{}/{}** | CS: **{}** | Gold: {} | Vision: {}",
                get_emoji(
                    collection_emoji.clone(),
                    "champions",
                    team_player["championName"].as_str().unwrap_or("Unknown")
                )
                .await
                .unwrap_or(
                    team_player["championName"]
                        .as_str()
                        .unwrap_or("Unknown")
                        .to_string()
                ),
                team_player["summonerName"].as_str().unwrap_or("Unknown"),
                team_player["kills"].as_u64().unwrap_or(0),
                team_player["deaths"].as_u64().unwrap_or(0),
                team_player["assists"].as_u64().unwrap_or(0),
                team_player["totalFarm"].as_u64().unwrap_or(0),
                format_gold_k(team_player["goldEarned"].as_u64().unwrap_or(0)),
                team_player["visionScore"].as_u64().unwrap_or(0)
            );

            // Enemy player stats
            let enemy_stats = format!(
                "{} **{}**\nK/D/A: **{}/{}/{}** | CS: **{}** | Gold: {} | Vision: {}",
                get_emoji(
                    collection_emoji.clone(),
                    "champions",
                    enemy_player["championName"].as_str().unwrap_or("Unknown")
                )
                .await
                .unwrap_or(
                    enemy_player["championName"]
                        .as_str()
                        .unwrap_or("Unknown")
                        .to_string()
                ),
                enemy_player["summonerName"].as_str().unwrap_or("Unknown"),
                enemy_player["kills"].as_u64().unwrap_or(0),
                enemy_player["deaths"].as_u64().unwrap_or(0),
                enemy_player["assists"].as_u64().unwrap_or(0),
                enemy_player["totalFarm"].as_u64().unwrap_or(0),
                format_gold_k(enemy_player["goldEarned"].as_u64().unwrap_or(0)),
                enemy_player["visionScore"].as_u64().unwrap_or(0)
            );

            // Combine team and enemy stats
            let field_value = format!("{}\n{}", team_stats, enemy_stats);

            // Add the field to the embed
            embed = embed.field(role_label, field_value, false);
        }
    }

    embed
}

/// ⚙️ **Function**: Extracts key participant statistics from a match for a given player.
///
/// This function retrieves important statistics for a participant in a League of Legends match, such as their summoner name,
/// champion name, kills, deaths, assists, total farm, gold earned, and vision score. The extracted stats are returned as a JSON object (`serde_json::Value`).
///
/// # Parameters:
/// - `p`: A reference to a `serde_json::Value` object representing a participant in the match. This object contains all of the participant's stats and data.
///
/// # Returns:
/// - `Value`: Returns a JSON object containing the player's stats, including their summoner name, champion name, K/D/A (kills, deaths, assists),
/// total farm (minions and neutral monsters killed), gold earned, gold per minute, and vision score.
///
/// # Example:
/// This function is used to format and extract individual player stats from the match data:
///
/// ```rust
/// let player_stats = extract_participant_stats(&participant);
/// println!("{}", player_stats["summonerName"]);
/// ```
///
/// # Notes:
/// - The summoner's name is prioritized over their Riot ID game name, but if the summoner name is missing, the Riot ID is used as a fallback.
/// - Total farm is calculated as the sum of minions killed and neutral monsters killed.
/// - The stats returned include the summoner's name, champion, K/D/A, farm, gold, gold per minute, and vision score, which are useful for comparing performance across teams.
fn extract_participant_stats(p: &Value) -> Value {
    let riot_id_game_name = p["riotIdGameName"].as_str().unwrap_or("Unknown");
    let summoner_name = if p["summonerName"].as_str().unwrap_or("Unknown").is_empty() {
        riot_id_game_name
    } else {
        p["summonerName"].as_str().unwrap_or("Unknown")
    };
    let champion_name = p["championName"].as_str().unwrap_or("Unknown");
    let kills = p["kills"].as_u64().unwrap_or(0);
    let deaths = p["deaths"].as_u64().unwrap_or(0);
    let assists = p["assists"].as_u64().unwrap_or(0);
    let total_minions_killed = p["totalMinionsKilled"].as_u64().unwrap_or(0);
    let neutral_minions_killed = p["neutralMinionsKilled"].as_u64().unwrap_or(0);
    let total_farm = total_minions_killed + neutral_minions_killed;
    let gold_earned = p["goldEarned"].as_u64().unwrap_or(0);
    let vision_score = p["visionScore"].as_u64().unwrap_or(0);

    serde_json::json!({
        "summonerName": summoner_name,
        "championName": champion_name,
        "kills": kills,
        "deaths": deaths,
        "assists": assists,
        "totalFarm": total_farm,
        "goldEarned": gold_earned,
        "visionScore": vision_score
    })
}

/// ⚙️ **Function**: Formats the amount of gold earned in a match into a more readable "k" notation when appropriate.
///
/// This function takes an amount of gold as input and formats it into a human-readable string. If the amount is less than 1000,
/// it returns the gold value as a simple string. If the gold is 1000 or more, it formats the value in "k" notation (e.g., 1500 becomes "1.5k").
///
/// # Parameters:
/// - `gold`: A `u64` value representing the amount of gold earned by a player in a match.
///
/// # Returns:
/// - `String`: A formatted string representing the gold amount. If the amount is less than 1000, it returns the value as is.
/// For amounts equal to or greater than 1000, it returns a string in "k" notation (e.g., "1k", "1.5k") with a comma used as the decimal separator.
///
/// # Example:
/// ```rust
/// let formatted_gold = format_gold_k(1500);
/// assert_eq!(formatted_gold, "1,5k");
/// ```
///
/// # Notes:
/// - For gold values with no fractional part, the result will omit the decimal point (e.g., 1000 will be formatted as "1k" instead of "1.0k").
/// - The function uses a comma to separate the decimal part, following European formatting conventions.
fn format_gold_k(gold: u64) -> String {
    if gold < 1000 {
        gold.to_string()
    } else {
        let gold_f64 = (gold as f64) / 1000.0;
        if gold_f64.fract() == 0.0 {
            format!("{}k", gold_f64 as u64)
        } else {
            let formatted = format!("{:.1}", gold_f64).replace('.', ",");
            format!("{}k", formatted)
        }
    }
}

/// ⚙️ **Function**: Retrieves all followed summoners from the database.
///
/// This asynchronous function queries the "follower_summoner" collection in the "stat-summoner" MongoDB database
/// to retrieve all documents representing followed summoners. It collects each `SummonerFollowedData` into a vector.
///
/// # Parameters:
/// - `collection`: A reference to the MongoDB collection containing `SummonerFollowedData` documents.
///
/// # Returns:
/// - `Result<Vec<SummonerFollowedData>, mongodb::error::Error>`: A vector of followed summoners if successful, or an error if the query fails.
///
/// # ⚠️ Notes:
/// - Prints an error message in French if a document retrieval fails.
/// - Ensure that the `SummonerFollowedData` struct aligns with the collection's document structure.
///
/// # Example:
/// ```rust
/// let summoners = get_followed_summoners(&collection).await?;
/// println!("Retrieved {} summoners.", summoners.len());
/// ```
pub async fn get_followed_summoners(
    collection: &Collection<SummonerFollowedData>,
) -> Result<Vec<SummonerFollowedData>, mongodb::error::Error> {
    let mut cursor = collection.find(doc! {}).await?;
    let mut followed_summoners = Vec::new();

    while let Some(result) = cursor.next().await {
        match result {
            Ok(followed_summoner) => {
                followed_summoners.push(followed_summoner);
            }
            Err(e) => {
                println!("Erreur lors de la récupération d'un document : {:?}", e);
            }
        }
    }

    Ok(followed_summoners)
}

/// ⚙️ **Function**: Processes a followed summoner by checking if their follow time has expired or if they have played a new match.
///
/// This asynchronous function handles the logic for a followed summoner. It checks if the follow time has expired and removes the summoner from the database if necessary. If the follow time is still valid, it checks for new matches and updates the summoner's information accordingly.
///
/// # Parameters:
/// - `collection`: A reference to a MongoDB `Collection<SummonerFollowedData>` that stores the followed summoners' data.
/// - `followed_summoner`: A reference to a `SummonerFollowedData` struct containing the summoner's information, including their follow duration and last match details.
/// - `riot_api_key`: A string slice containing the Riot Games API key for authenticating the API request.
/// - `http`: An `Arc<Http>` object used to send messages via the Discord API.
/// - `collection_emojis`: A MongoDB `Collection` containing emoji mappings, used to enrich the Discord embeds with custom emojis for roles and champions.
///
/// # Returns:
/// - `Result<(), Error>`: Returns `Ok(())` if the summoner was successfully processed (either by removing them from the database or updating their match info), or an error if something went wrong.
///
/// # Example:
/// This function is typically called as part of a loop or scheduled task that checks the status of followed summoners:
///
/// ```rust
/// let result = process_followed_summoner(collection, &followed_summoner, riot_api_key, http.clone(), collection_emojis).await;
/// if result.is_err() {
///     // Handle error (e.g., log failure or retry)
/// }
/// ```
///
/// # Notes:
/// - The function begins by checking if the follow time for the summoner has expired using the `is_follow_time_expired` function.
/// - If the follow time has expired, the summoner is removed from the MongoDB collection by calling `delete_follower`.
/// - If the summoner is still being followed, the function calls `update_follower_if_new_match` to check for new matches and potentially send an update to the associated Discord channel.
/// - This function ensures that summoners are only followed for the specified duration and that Discord channels are updated with relevant match information during the follow period.
pub async fn process_followed_summoner(
    collection: &Collection<SummonerFollowedData>,
    followed_summoner: &SummonerFollowedData,
    riot_api_key: &str,
    http: Arc<Http>,
    collection_emojis: Collection<EmojiId>,
) -> Result<(), Error> {
    if is_follow_time_expired(followed_summoner) {
        delete_follower(collection, followed_summoner).await?;
    } else {
        update_follower_if_new_match(
            collection,
            followed_summoner,
            riot_api_key,
            http,
            collection_emojis,
        )
        .await?;
    }
    Ok(())
}

/// ⚙️ **Function**: Determines if the follow time for a summoner has expired.
///
/// This function checks whether the current timestamp exceeds the stored follow end time for a summoner.
/// It parses the `time_end_follow` field from the `SummonerFollowedData` struct, compares it to the current UTC timestamp,
/// and returns `true` if the follow time has expired, or `false` otherwise.
///
/// # Parameters:
/// - `followed_summoner`: A reference to a `SummonerFollowedData` struct, which contains information about the summoner, including when the follow period ends.
///
/// # Returns:
/// - `bool`: Returns `true` if the current time is greater than the stored `time_end_follow`, meaning the follow period has expired. Returns `false` if the follow period is still active.
///
/// # Example:
/// This function is used to determine whether a summoner should be removed from the list of followed summoners.
///
/// ```rust
/// let expired = is_follow_time_expired(&followed_summoner);
/// if expired {
///     // Remove summoner from database
/// }
/// ```
///
/// # Notes:
/// - The function uses UTC time for comparison and assumes the `time_end_follow` is a valid timestamp that can be parsed into an `i64`. If parsing fails, it defaults to 0, which will always result in `true`.
fn is_follow_time_expired(followed_summoner: &SummonerFollowedData) -> bool {
    let time_end_follow = followed_summoner
        .time_end_follow
        .parse::<i64>()
        .unwrap_or(0);
    let current_timestamp = Utc::now().timestamp();
    current_timestamp > time_end_follow
}

/// ⚙️ **Function**: Deletes a followed summoner from the database.
///
/// This asynchronous function removes a summoner from the `follower_summoner` collection in MongoDB based on their `puuid`.
/// It logs the deletion action and ensures the summoner is no longer tracked in the database.
///
/// # Parameters:
/// - `collection`: A reference to the MongoDB `Collection<SummonerFollowedData>`, used to interact with the database and delete the summoner data.
/// - `followed_summoner`: A reference to a `SummonerFollowedData` struct, representing the summoner that is being deleted. The deletion is based on the summoner's `puuid`.
///
/// # Returns:
/// - `Result<(), mongodb::error::Error>`: Returns an empty result if successful, or an error if the deletion fails.
///
/// # Example:
/// This function is typically called when the follow time for a summoner has expired, and they need to be removed from the database:
///
/// ```rust
/// delete_follower(&collection, &followed_summoner).await?;
/// ```
///
/// # Notes:
/// - The `puuid` field is used as the unique identifier for deletion from the MongoDB collection.
/// - The function logs the `puuid` of the summoner being deleted using `eprintln!`, which outputs the message to the standard error stream.
async fn delete_follower(
    collection: &Collection<SummonerFollowedData>,
    followed_summoner: &SummonerFollowedData,
) -> Result<(), mongodb::error::Error> {
    eprintln!("Suppression de {}", followed_summoner.puuid);
    collection
        .delete_one(
            doc! { "puuid": &followed_summoner.puuid, "guild_id": &followed_summoner.guild_id },
        )
        .await?;
    Ok(())
}

/// ⚙️ **Function**: Updates a followed summoner's last match ID and sends a Discord update if a new match is detected.
///
/// This asynchronous function checks if a followed summoner has played a new match. If a new match is detected,
/// it updates the summoner's last match ID in the MongoDB collection and sends a match update to the appropriate Discord channel.
///
/// # Parameters:
/// - `collection`: A reference to a MongoDB `Collection<SummonerFollowedData>` that stores the followed summoners' data.
/// - `followed_summoner`: A reference to a `SummonerFollowedData` struct containing the summoner's information, including their PUUID, summoner ID, and last match ID.
/// - `riot_api_key`: A string slice containing the Riot Games API key for authenticating the API request.
/// - `http`: An `Arc<Http>` object used to send messages via the Discord API.
/// - `collection_emojis`: A MongoDB `Collection` containing emoji mappings, used to enhance the Discord embed with custom emojis for roles and champions.
///
/// # Returns:
/// - `Result<(), Error>`: Returns `Ok(())` if the last match ID was successfully updated and the match update was sent to Discord, or an error if something went wrong.
///
/// # Example:
/// This function is typically called periodically to check if a followed summoner has played a new match:
///
/// ```rust
/// let result = update_follower_if_new_match(collection, &followed_summoner, riot_api_key, http.clone(), collection_emojis).await;
/// if result.is_err() {
///     // Handle error (e.g., log failure or retry)
/// }
/// ```
///
/// # Notes:
/// - The function begins by creating an HTTP client using `reqwest` and fetching the latest match ID for the summoner using the `get_latest_match_id` function.
/// - If the new match ID is different from the stored `last_match_id`, the function updates the MongoDB collection with the new match ID.
/// - Once the database is updated, the function calls `send_match_update_to_discord` to send a match update to the Discord channel associated with the summoner.
/// - This function ensures that the Discord server is notified whenever the summoner completes a new match, keeping followers updated in real time.
async fn update_follower_if_new_match(
    collection: &Collection<SummonerFollowedData>,
    followed_summoner: &SummonerFollowedData,
    riot_api_key: &str,
    http: Arc<Http>,
    collection_emojis: Collection<EmojiId>,
) -> Result<(), Error> {
    let puuid = &followed_summoner.puuid;
    let summoner_id = &followed_summoner.summoner_id;
    let last_match_id = &followed_summoner.last_match_id;
    let guild_id = &followed_summoner.guild_id;
    let client = reqwest::Client::new();

    let match_id_from_riot = get_latest_match_id(&client, puuid, riot_api_key).await?;

    if last_match_id != &match_id_from_riot {
        collection
            .update_one(
                doc! {
                "puuid": puuid,
                "guild_id": guild_id
                },
                doc! { "$set": { "last_match_id": &match_id_from_riot } },
            )
            .await?;
        send_match_update_to_discord(
            followed_summoner,
            summoner_id,
            &match_id_from_riot,
            riot_api_key,
            http,
            collection_emojis,
        )
        .await?;
    }
    Ok(())
}

/// ⚙️ **Function**: Fetches the latest match ID for a given summoner using their PUUID.
///
/// This asynchronous function retrieves the most recent match ID for a summoner by making a request to the Riot API.
/// It uses the summoner's `puuid` to query their match history and returns the match ID of the most recent game.
///
/// # Parameters:
/// - `client`: A reference to the `reqwest::Client`, used to make HTTP requests to the Riot API.
/// - `puuid`: A string slice representing the summoner's PUUID (a unique identifier for each player in Riot's system).
/// - `riot_api_key`: A string slice representing the Riot API key, used for authorized requests.
///
/// # Returns:
/// - `Result<String, Error>`: Returns the latest match ID as a string if successful, or an error if the request or retrieval fails.
///
/// # Example:
/// This function is typically used to get the latest match ID for a summoner in order to check for new matches:
///
/// ```rust
/// let latest_match_id = get_latest_match_id(&client, puuid, riot_api_key).await?;
/// ```
///
/// # Notes:
/// - The function calls `get_matchs_id` to retrieve the match history and then returns the first match in the list, which corresponds to the most recent match.
/// - The `get_matchs_id` function is expected to return a vector of match IDs, from which the latest match (the first one) is extracted and returned.
async fn get_latest_match_id(
    client: &reqwest::Client,
    puuid: &str,
    riot_api_key: &str,
) -> Result<String, Error> {
    let matches = get_matchs_id(client, puuid, riot_api_key, 1).await?;
    Ok(matches[0].clone())
}

/// ⚙️ **Function**: Sends a match update to a specific Discord channel for a followed summoner.
///
/// This asynchronous function fetches match information for a followed summoner from the Riot API,
/// formats the details into an embed, and sends the embed as a message to the specified Discord channel.
///
/// # Parameters:
/// - `followed_summoner`: A reference to a `SummonerFollowedData` struct, which contains the summoner's name and the ID of the Discord channel to which the match update should be sent.
/// - `summoner_id`: A string slice representing the summoner's ID, used to identify the player's stats in the match.
/// - `match_id`: A string slice representing the match ID, used to fetch match details from the Riot API.
/// - `riot_api_key`: A string slice containing the Riot Games API key for authenticating the API request.
/// - `http`: An `Arc<Http>` object used to send messages via the Discord API.
/// - `collection_emojis`: A MongoDB `Collection` containing emoji mappings, used to add custom emojis to the embed for roles and champions.
///
/// # Returns:
/// - `Result<(), Error>`: Returns `Ok(())` if the match update was successfully sent to the Discord channel, or an error if something went wrong.
///
/// # Example:
/// This function is typically called after detecting that a followed summoner has completed a match:
///
/// ```rust
/// let result = send_match_update_to_discord(&followed_summoner, summoner_id, match_id, riot_api_key, http.clone(), collection_emojis).await;
/// if result.is_err() {
///     // Handle error (e.g., log failure or retry)
/// }
/// ```
///
/// # Notes:
/// - The function creates an HTTP client using `reqwest` to fetch match information from the Riot API.
/// - It retrieves detailed match data using the `get_matchs_info` and `get_match_details` functions.
/// - The function constructs a `CreateEmbed` object using the `create_embed_loop` function, which formats match statistics and adds emojis.
/// - The embed is sent as a message to the Discord channel specified in the `followed_summoner` struct.
/// - The Discord message is built using `CreateMessage` and sent asynchronously to the appropriate channel using the Discord API.
async fn send_match_update_to_discord(
    followed_summoner: &SummonerFollowedData,
    summoner_id: &str,
    match_id: &str,
    riot_api_key: &str,
    http: Arc<Http>,
    collection_emojis: Collection<EmojiId>,
) -> Result<(), Error> {
    let client = reqwest::Client::new();
    let info = get_matchs_info(&client, match_id, riot_api_key).await?;
    let info_json = get_match_details(&info, summoner_id).unwrap();
    let channel_id = serenity::model::id::ChannelId::new(followed_summoner.channel_id);
    let embed = create_embed_loop(&info_json, &followed_summoner.name, collection_emojis).await;
    let builder = CreateMessage::new().add_embed(embed);
    let _ = channel_id.send_message(&http, builder).await;
    Ok(())
}

/// ⚙️ **Function**: Fetches rune data for a specific champion from League of Graphs.
///
/// This asynchronous function retrieves the rune build information for a given champion
/// by making an HTTP request to League of Graphs. It then parses the HTML response to
/// extract the rune tables and returns the runes in the `RunesData` structure.
///
/// # Parameters:
/// - `champion_id`: A string slice representing the champion's identifier, used to build the URL for fetching the rune information.
///
/// # Returns:
/// - `Result<RunesData, Error>`: Returns a `RunesData` struct with the champion's rune information if successful, or an error if something goes wrong during the HTTP request or parsing.
///
/// # Example:
/// This function is typically called to retrieve the rune data for a specific champion:
///
/// ```rust
/// let runes = fetch_runes("Rammus").await?;
/// println!("{:?}", runes);
/// ```
///
/// # Notes:
/// - The function makes an HTTP request to the League of Graphs page using the champion's ID to construct the URL.
/// - It then parses the HTML to find the rune tables and extracts the relevant rune data.
/// - The `extract_runes` function is used to process the HTML and return the rune information in the `RunesData` structure.
/// - This function expects two rune tables (primary and secondary) to be present in the response, otherwise it will panic with an `unwrap()` error.
pub async fn fetch_runes(champion_id: &str) -> Result<RunesData, Error> {
    let url = format!(
        "https://www.leagueofgraphs.com/champions/builds/{}",
        champion_id
    );
    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await?;
    let body = res.text().await?;
    let document = Document::from(body.as_str());

    // Logique pour extraire les runes, en utilisant `RunesData` comme la structure finale
    let first_rune_table = document.find(Class("perksTableOverview")).next().unwrap();
    let secondary_rune_table = document.find(Class("perksTableOverview")).nth(1).unwrap();

    let runes = extract_runes(first_rune_table, secondary_rune_table);
    Ok(runes)
}

/// ⚙️ **Function**: Fetches core build data for a specific champion from League of Graphs.
///
/// This asynchronous function retrieves the core build item information for a given champion
/// by making an HTTP request to League of Graphs. It then parses the HTML response to locate
/// the core build section and extracts the items used in the build.
///
/// # Parameters:
/// - `champion_id`: A string slice representing the champion's identifier, used to build the URL for fetching the core build information.
///
/// # Returns:
/// - `Result<CoreBuildData, Error>`: Returns a `CoreBuildData` struct containing the champion's core build items if successful, or an error if something goes wrong during the HTTP request or parsing.
///
/// # Example:
/// This function is typically called to retrieve the core build data for a specific champion:
///
/// ```rust
/// let core_build = fetch_core_build("Jinx").await?;
/// println!("{:?}", core_build);
/// ```
///
/// # Notes:
/// - The function makes an HTTP request to the League of Graphs page using the champion's ID to construct the URL.
/// - It parses the HTML to find the core build section by searching for an `h3` element containing the text "Core Build".
/// - Once the core build header is found, the function searches for its parent element and the `iconsRow` div where the build items are listed.
/// - The function calls `extract_core_build` to process the icons and return the items in the `CoreBuildData` structure.
/// - If the core build header or the `iconsRow` div is not found, an error is returned.
pub async fn fetch_core_build(champion_id: &str) -> Result<CoreBuildData, Error> {
    let url = format!(
        "https://www.leagueofgraphs.com/champions/builds/{}",
        champion_id
    );
    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await?;
    let body = res.text().await?;
    let document = select::document::Document::from(body.as_str());

    if let Some(core_build_header) = document
        .find(Name("h3"))
        .find(|node| node.text().contains("Core Build"))
    {
        if let Some(parent_div) = core_build_header.parent() {
            if let Some(icons_row) = parent_div.find(Class("iconsRow")).next() {
                let core_build = extract_core_build(icons_row);
                return Ok(core_build);
            } else {
                return Err(Box::from("Erreur: Impossible de trouver 'iconsRow'"));
            }
        } else {
            return Err(Box::from(
                "Erreur: Impossible de trouver le parent de 'Core Build'",
            ));
        }
    } else {
        return Err(Box::from(
            "Erreur: Impossible de trouver le header 'Core Build'",
        ));
    }
}

/// ⚙️ **Function**: Extracts rune data from two HTML tables.
///
/// This function processes two HTML tables (representing primary and secondary runes) and extracts
/// the rune images by filtering out those that are not visible (with `opacity: 0.2`) and by cleaning
/// the `alt` attributes of the remaining images. The function then constructs a `RunesData` struct
/// containing all rune data if exactly 9 runes are found.
///
/// # Parameters:
/// - `first_table`: A `select::node::Node` representing the primary rune table.
/// - `second_table`: A `select::node::Node` representing the secondary rune table.
///
/// # Returns:
/// - `RunesData`: Returns a `RunesData` struct containing the extracted rune information. If the number of
/// runes found is not exactly 9, it returns a `RunesData` struct with empty strings for all fields.
///
/// # Example:
/// This function is typically called to process rune tables extracted from a web page:
///
/// ```rust
/// let runes_data = extract_runes(primary_table, secondary_table);
/// println!("{:?}", runes_data);
/// ```
///
/// # Notes:
/// - The function first collects all `img` tags from both the primary and secondary rune tables.
/// - It filters out images that have a parent `div` with `opacity: 0.2` and skips any image without a valid `alt` attribute.
/// - The `clean_alt_text` function is applied to clean up the `alt` text before it is added to the final rune list.
/// - The function expects exactly 9 runes: 4 primary runes, 2 secondary runes, and 3 tertiary runes. If this condition is not met, an empty `RunesData` struct is returned.
fn extract_runes(first_table: select::node::Node, second_table: select::node::Node) -> RunesData {
    let images = first_table
        .find(Name("img"))
        .chain(second_table.find(Name("img")))
        .filter_map(|img| {
            if let Some(parent_div) = img.parent() {
                if parent_div.attr("style") != Some("opacity: 0.2;") {
                    if let Some(alt) = img.attr("alt") {
                        if !alt.trim().is_empty() {
                            return Some(clean_alt_text(alt));
                        }
                    }
                }
            }
            None
        })
        .collect::<Vec<String>>();

    if images.len() == 9 {
        RunesData {
            parent_primary_rune: images[0].clone(),
            child_primary_rune_1: images[1].clone(),
            child_primary_rune_2: images[2].clone(),
            child_primary_rune_3: images[3].clone(),
            child_secondary_rune_1: images[4].clone(),
            child_secondary_rune_2: images[5].clone(),
            tertiary_rune_1: images[6].clone(),
            tertiary_rune_2: images[7].clone(),
            tertiary_rune_3: images[8].clone(),
        }
    } else {
        RunesData {
            parent_primary_rune: String::new(),
            child_primary_rune_1: String::new(),
            child_primary_rune_2: String::new(),
            child_primary_rune_3: String::new(),
            child_secondary_rune_1: String::new(),
            child_secondary_rune_2: String::new(),
            tertiary_rune_1: String::new(),
            tertiary_rune_2: String::new(),
            tertiary_rune_3: String::new(),
        }
    }
}

/// ⚙️ **Function**: Extracts the core build items from an HTML `iconsRow` div.
///
/// This function processes a div containing the core build items for a champion, typically found
/// in the `iconsRow` HTML element. It extracts the `alt` attributes of the item images, cleans them,
/// and returns the first, second, and third items in the build as a `CoreBuildData` struct.
///
/// # Parameters:
/// - `icons_row`: A `select::node::Node` representing the `iconsRow` div that contains the item icons for the core build.
///
/// # Returns:
/// - `CoreBuildData`: Returns a `CoreBuildData` struct containing the names of the first, second, and third core build items.
///
/// # Example:
/// This function is typically called to process the core build for a specific champion:
///
/// ```rust
/// let core_build = extract_core_build(icons_row);
/// println!("{:?}", core_build);
/// ```
///
/// # Notes:
/// - The function collects all `img` tags within the `iconsRow` div and extracts the `alt` attributes, which contain the names of the items.
/// - The `clean_alt_text` function is used to clean the `alt` text by removing unnecessary characters and formatting it.
/// - The function assumes that the images vector contains at least four elements, where the first image is ignored and the second, third, and fourth images represent the core build items.
/// - If the `iconsRow` div does not contain enough images, this could result in an `index out of bounds` error, so ensure the data is well-formed before calling the function.
fn extract_core_build(icons_row: select::node::Node) -> CoreBuildData {
    let images = icons_row
        .find(Name("img"))
        .filter_map(|img| img.attr("alt"))
        .map(clean_alt_text)
        .collect::<Vec<String>>();
    CoreBuildData {
        first: images[1].clone(),
        second: images[2].clone(),
        third: images[3].clone(),
    }
}

/// ⚙️ **Function**: Cleans the alt text for an item or rune and applies special formatting.
///
/// This function processes the `alt` attribute text extracted from HTML elements, removes unwanted characters,
/// and applies special rules. If the `alt` text contains certain characters (parentheses, hyphens, and plus signs),
/// it returns "Health Scale". Otherwise, it cleans the text by removing parentheses, numbers, and specific symbols.
///
/// # Parameters:
/// - `alt`: A string slice representing the `alt` text from an HTML `img` tag that needs to be cleaned.
///
/// # Returns:
/// - `String`: Returns a cleaned version of the `alt` text. If the `alt` text matches specific patterns (parentheses, hyphen, and plus signs),
/// it returns "Health Scale". Otherwise, it returns the cleaned text with unwanted characters removed.
///
/// # Example:
/// This function is typically called to clean the text from the `alt` attributes of item or rune images:
///
/// ```rust
/// let clean_text = clean_alt_text("Health (100) + 10% - 5%");
/// println!("{}", clean_text); // Output: "Health Scale"
///
/// let clean_text = clean_alt_text("Sunfire Aegis");
/// println!("{}", clean_text); // Output: "SunfireAegis"
/// ```
///
/// # Notes:
/// - If the `alt` text contains parentheses `()`, a hyphen `-`, and a plus sign `+`, the function returns "Health Scale".
/// - It uses regular expressions to remove unwanted characters such as parentheses, numbers, percentage symbols, commas, and others.
/// - Spaces are also removed in the final output.
fn clean_alt_text(alt: &str) -> String {
    if alt.contains('(') && alt.contains(')') && alt.contains('-') && alt.contains('+') {
        return "Health Scale".to_string();
    }
    let re_parentheses = Regex::new(r"\(.*?\)").unwrap();
    let cleaned_alt = re_parentheses.replace_all(alt, "");
    let re_unwanted = Regex::new(r"[+:1234567890%-',]").unwrap();
    let cleaned_alt = re_unwanted.replace_all(&cleaned_alt, "").trim().to_string();

    cleaned_alt.replace(" ", "")
}
