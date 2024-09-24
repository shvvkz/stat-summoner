use crate::{models::data::EmojiId, utils::*};
use mongodb::Collection;
use poise::serenity_prelude::CreateEmbed;
use serde_json::Value;
use std::collections::HashMap;

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
