use crate::embed::create_embed;
use crate::models::constants::QUEUE_ID_MAP;
use crate::models::data::{Data, EmojiId};
use crate::models::error::Error;
use crate::models::modal::LolStatsModal;
use crate::riot_api::get_matchs_info;
use crate::utils::{get_emoji, is_valid_game_mode, seconds_to_time, time_since_game_ended};
use mongodb::Collection;
use poise::CreateReply;
use reqwest::Client;
use serde_json::{Map, Value};
use std::collections::HashMap;

/// ⚙️ **Function**: Fetches data and creates an embed displaying League of Legends player stats and match details.
///
/// This function orchestrates the process of fetching rank, champion, and match data, and formats this information
/// into an embed message. The embed is then prepared for sending in a Discord channel.
///
/// # Parameters:
/// - `modal_data`: A modal containing the player's in-game name and tag, used to personalize the embed title.
/// - `summoner_id`: The unique ID of the summoner (player) whose data is being fetched. This is used to query relevant match and rank data.
/// - `solo_rank`: A HashMap containing the player's Solo/Duo rank information, such as tier, LP, wins, losses, and winrate.
/// - `flex_rank`: A HashMap containing the player's Flex rank information, structured similarly to `solo_rank`.
/// - `champions`: A vector of HashMaps, where each HashMap contains information about the player's top champions (e.g., champion level and mastery points).
/// - `match_ids`: A vector of match IDs representing recent matches played by the user.
/// - `ctx`: The application context, which includes methods for interacting with Discord and accessing API keys for fetching data.
///
/// # Returns:
/// - `CreateReply`: A formatted reply containing the embed message, ready to be sent to a Discord channel.
///
/// # ⚠️ Notes:
/// - The function fetches champion data from Data Dragon and match data from the Riot API, ensuring that up-to-date information is displayed.
/// - If no match details are found, the embed will indicate that no recent ranked or normal matches were played.
/// - The function extracts and formats data for Solo/Duo and Flex ranks, as well as champion and match details.
///
/// # Example:
/// ```rust
/// let embed_reply = create_and_send_embed_lolstats(modal_data, summoner_id, &solo_rank, &flex_rank, champions, match_ids, &ctx).await;
/// ctx.send(embed_reply).await?;
/// ```
///
/// The resulting embed message will contain player stats like:
/// ```text
/// 📊 Stats for Faker#1234
/// 🔱 **Solo/Duo Rank**: Gold I (100 LP)
/// 🌀 **Flex Rank**: Silver IV (50 LP)
/// 💥 **Top Champions**:
/// Yasuo - Level: 7 - Points: 123456
/// 📜 **Match Details**:
/// Victory - **Yasuo**, 2 hours ago (Ranked Solo/Duo):
/// K/D/A: **10/2/8** | **200 CS** | Duration: **30:45**
/// ⏳ Played: **2 hours ago**
/// ```
pub async fn create_and_send_embed_lolstats(
    modal_data: &LolStatsModal,
    summoner_id: String,
    solo_rank: &HashMap<String, Value>,
    flex_rank: &HashMap<String, Value>,
    champions: Vec<HashMap<String, Value>>,
    match_ids: Vec<String>,
    ctx: &poise::ApplicationContext<'_, Data, Error>,
    collection_emoji: Collection<EmojiId>,
) -> CreateReply {
    let dd_json = &*ctx.data().dd_json.read().await;
    let champions_data = dd_json["data"].as_object().unwrap();

    let solo_rank = extract_rank_info(solo_rank);
    let flex_rank = extract_rank_info(flex_rank);
    let champions_info =
        extract_champions_info(champions, champions_data, collection_emoji.clone()).await;
    let match_details = extract_match_info(match_ids, ctx, summoner_id).await;

    let embed = create_embed(
        modal_data,
        solo_rank,
        flex_rank,
        champions_info,
        match_details,
        collection_emoji.clone(),
    )
    .await
    .unwrap();

    CreateReply {
        embeds: vec![embed],
        ..Default::default()
    }
}

/// ⚙️ **Function**: Extracts and returns League of Legends rank information.
///
/// This function processes rank data to extract key details such as tier, division, league points (LP),
/// wins, losses, and winrate. The resulting information is formatted into a JSON-like value for use in
/// other parts of the application, such as creating embeds for Discord.
///
/// # Parameters:
/// - `rank_data`: A HashMap containing the player's rank information, typically fetched from the Riot API.
///   This data includes keys such as `"tier"`, `"rank"`, `"leaguePoints"`, `"wins"`, and `"losses"`.
///
/// # Returns:
/// - `Value`: A JSON-like value containing the extracted rank information:
///     - `tier`: The rank tier (e.g., "Gold", "Platinum"), defaults to "Unranked" if not present.
///     - `division`: The rank division (e.g., "I", "II"), empty if not present.
///     - `lp`: League points, defaults to 0 if not present.
///     - `wins`: Number of wins, defaults to 0 if not present.
///     - `losses`: Number of losses, defaults to 0 if not present.
///     - `winrate`: The player's winrate, calculated as `wins / (wins + losses)`, defaults to 0 if no games are played.
///
/// # ⚠️ Notes:
/// - If the player is unranked or data is missing, the function will return default values such as `"Unranked"` for
///   the tier, and `0` for LP, wins, and losses.
/// - The winrate is calculated as a percentage and will return `0.0%` if there are no games played (i.e., wins + losses = 0).
///
/// # Example:
/// ```rust
/// let rank_data = some_function_fetching_rank_data();
/// let rank_info = extract_rank_info(&rank_data);
/// ```
///
/// The resulting `rank_info` will be in the following format:
/// ```json
/// {
///     "tier": "Gold",
///     "division": "II",
///     "lp": 45,
///     "wins": 20,
///     "losses": 15,
///     "winrate": 57.14
/// }
/// ```
fn extract_rank_info(rank_data: &HashMap<String, Value>) -> Value {
    let tier = rank_data
        .get("tier")
        .and_then(|v| v.as_str())
        .unwrap_or("Unranked");
    let division = rank_data.get("rank").and_then(|v| v.as_str()).unwrap_or("");
    let lp = rank_data
        .get("leaguePoints")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let wins = rank_data.get("wins").and_then(|v| v.as_i64()).unwrap_or(0);
    let losses = rank_data
        .get("losses")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let winrate = if wins + losses > 0 {
        ((wins as f64) / ((wins + losses) as f64)) * 100.0
    } else {
        0.0
    };
    return serde_json::json!({
        "tier": tier,
        "division": division,
        "lp": lp,
        "wins": wins,
        "losses": losses,
        "winrate": winrate
    });
}

/// ⚙️ **Function**: Extracts and formats champion information for display.
///
/// This function processes a list of champion details and matches each champion ID to the corresponding
/// champion name from the provided champion data (typically fetched from Data Dragon). It then formats
/// and returns a string that includes each champion's name, level, and mastery points.
///
/// # Parameters:
/// - `champions`: A vector of HashMaps, where each HashMap contains information about a player's champion
///   (e.g., champion ID, level, mastery points). This is typically fetched from the Riot API.
/// - `champions_data`: A HashMap containing the full list of champion data from Data Dragon, which is used
///   to map champion IDs to their names.
///
/// # Returns:
/// - `String`: A formatted string containing information about each champion:
///     - Champion name
///     - Champion level
///     - Champion mastery points
///
/// The returned string will display each champion on a new line, formatted like this:
/// ```text
/// Yasuo - Level: 7 - Points: 123456
/// Zed - Level: 6 - Points: 98765
/// Lee Sin - Level: 5 - Points: 54321
/// ```
///
/// # ⚠️ Notes:
/// - If a champion's ID cannot be matched to a name in `champions_data`, the champion will be listed as "Unknown Champion".
/// - This function assumes that every champion in the `champions` list has valid data for level and mastery points.
///
/// # Example:
/// ```rust
/// let champions = some_function_fetching_champions();
/// let champions_data = some_function_fetching_champion_data();
/// let formatted_champions = extract_champions_info(champions, champions_data);
/// ```
///
/// The resulting `formatted_champions` string will be:
/// ```text
/// Yasuo - Level: 7 - Points: 123456
/// Zed - Level: 6 - Points: 98765
/// Lee Sin - Level: 5 - Points: 54321
/// ```
async fn extract_champions_info(
    champions: Vec<HashMap<String, Value>>,
    champions_data: &Map<String, Value>,
    collection_emoji: Collection<EmojiId>,
) -> String {
    let mut champion_info_strings = Vec::new();

    for champion in champions {
        let champion_id = champion
            .get("championId")
            .unwrap()
            .as_i64()
            .unwrap()
            .to_string();
        let champion_name = champions_data
            .values()
            .find_map(|data| {
                let champ = data.as_object().unwrap();
                if champ.get("key").unwrap() == &Value::String(champion_id.clone()) {
                    Some(champ.get("id").unwrap().as_str().unwrap())
                } else {
                    None
                }
            })
            .unwrap_or("Unknown Champion");

        let champion_level = champion.get("championLevel").unwrap().as_i64().unwrap();
        let champion_points = champion.get("championPoints").unwrap().as_i64().unwrap();
        let champion_emoji = get_emoji(collection_emoji.clone(), "champions", champion_name)
            .await
            .unwrap_or(champion_name.to_string());
        champion_info_strings.push(format!(
            "{} - Level: {} - Points: {}",
            champion_emoji, champion_level, champion_points
        ));
    }
    champion_info_strings.join("\n")
}

/// ⚙️ **Function**: Extracts detailed information from recent League of Legends matches.
///
/// This function processes a list of match IDs, fetching and extracting key match information
/// such as champion played, kills, deaths, assists (K/D/A), total farm, game duration, and
/// match outcome (victory or defeat). The extracted data is returned as a vector of JSON-like
/// values for use in other parts of the application, such as creating embeds for Discord.
///
/// # Parameters:
/// - `match_ids`: A vector of match IDs to fetch and process. Each ID corresponds to a recent match played by the user.
/// - `ctx`: The application context, which includes the Riot API key for fetching match data and methods for interacting with Discord.
/// - `summoner_id`: The unique ID of the summoner (player) whose match data is being processed. This is used to find the player's data within each match.
///
/// # Returns:
/// - `Vec<Value>`: A vector of JSON-like values, where each entry contains information about a match:
///     - `champion_name`: The name of the champion played in the match.
///     - `K/D/A`: The player's kills, deaths, and assists in the match.
///     - `Farm`: The total number of minions and neutral monsters killed.
///     - `Result`: The outcome of the match (Victory or Defeat).
///     - `Duration`: The duration of the match in minutes and seconds.
///     - `time_elapsed`: The time since the match ended, formatted as seconds, minutes, hours, or days ago.
///     - `game_type`: The type of game played (e.g., Ranked Solo/Duo, ARAM).
///
/// # ⚠️ Notes:
/// - Only matches with a valid game mode (as determined by `is_valid_game_mode()`) are processed.
/// - If a match does not contain the player's data, it is skipped.
/// - The function uses the `time_since_game_ended` utility to calculate how long ago the match was played.
///
/// # Example:
/// ```rust
/// let match_ids = vec!["EUW1_1234567890", "EUW1_0987654321"];
/// let match_info = extract_match_info(match_ids, ctx, summoner_id).await;
/// ```
///
/// The resulting `match_info` vector will contain data for each match, such as:
/// ```json
/// [
///   {
///     "champion_name": "Yasuo",
///     "K/D/A": "10/2/8",
///     "Farm": 220,
///     "Result": "Victory",
///     "Duration": "30:12",
///     "time_elapsed": "2 hours ago",
///     "game_type": "Ranked Solo/Duo"
///   },
///   {
///     "champion_name": "Zed",
///     "K/D/A": "7/5/10",
///     "Farm": 180,
///     "Result": "Defeat",
///     "Duration": "28:45",
///     "time_elapsed": "1 day ago",
///     "game_type": "Ranked Flex"
///   }
/// ]
/// ```
async fn extract_match_info(
    match_ids: Vec<String>,
    ctx: &poise::ApplicationContext<'_, Data, Error>,
    summoner_id: String,
) -> Vec<Value> {
    let mut match_details = Vec::<Value>::new();
    for id in &match_ids {
        let info = get_matchs_info(&Client::new(), id, &ctx.data().riot_api_key)
            .await
            .unwrap();
        let queue_id = info["info"]["queueId"].as_i64().unwrap_or(-1);
        if is_valid_game_mode(queue_id) {
            let participants = info["info"]["participants"].as_array().unwrap();
            if let Some(participant) = participants
                .iter()
                .find(|p| p["summonerId"].as_str().unwrap() == summoner_id)
            {
                let champion_name = participant["championName"].as_str().unwrap_or("Unknown");
                let kills = participant["kills"].as_u64().unwrap_or(0);
                let deaths = participant["deaths"].as_u64().unwrap_or(0);
                let assists = participant["assists"].as_u64().unwrap_or(0);
                let total_farm = participant["totalMinionsKilled"].as_u64().unwrap_or(0)
                    + participant["neutralMinionsKilled"].as_u64().unwrap_or(0);
                let win = participant["win"].as_bool().unwrap_or(false);
                let game_result = if win { "Victory" } else { "Defeat" };
                let game_duration = info["info"]["gameDuration"].as_u64().unwrap_or(0);
                let game_end_timestamp = info["info"]["gameEndTimestamp"].as_u64().unwrap_or(0);
                let time_since_game_ended = time_since_game_ended(game_end_timestamp);
                let (game_duration_minutes, game_duration_seconds) = seconds_to_time(game_duration);
                let game_type = QUEUE_ID_MAP
                    .iter()
                    .find(|(id, _)| *id == queue_id)
                    .unwrap()
                    .1;
                match_details.push(serde_json::json!({
                    "champion_name": champion_name,
                    "K/D/A": format!("{}/{}/{}", kills, deaths, assists),
                    "Farm": total_farm,
                    "Result": game_result,
                    "Duration": format!("{}:{}", game_duration_minutes, game_duration_seconds),
                    "time_elapsed": time_since_game_ended,
                    "game_type": game_type
                }));
            }
        }
    }
    match_details
}
