use std::collections::HashMap;
use chrono::Utc;
use poise::serenity_prelude::CreateEmbed;
use serde_json::Value;
use crate::utils::*;


/// Traite les informations d'un match pour extraire les détails des participants.
/// Retourne un objet JSON structuré ou `None` si le mode de jeu n'est pas valide.
pub fn get_match_details(match_info: &Value, summoner_id: &str) -> Option<Value> {
    let queue_id = match_info["info"]["queueId"].as_i64().unwrap_or(-1);
    if !is_valid_game_mode(queue_id) {
        return None;
    }

    let participants = match_info["info"]["participants"].as_array()?;

    // Trouver le participant correspondant au summoner_id
    let participant = participants
        .iter()
        .find(|p| p["summonerId"].as_str().unwrap_or("") == summoner_id)?;

    let team_id = participant["teamId"].as_i64().unwrap_or(0);
    let win = participant["win"].as_bool().unwrap_or(false);
    let game_result = if win { "Victory" } else { "Defeat" };

    // Séparer les participants par équipe
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
        "gameResult": game_result,
        "matchups": matchups
    }))
}

/// Function to create a Discord embed for match details
pub fn create_embed_loop(info_json: &Value, player_name: &str) -> CreateEmbed {
    // Extract game result and corresponding emoji
    let game_result = info_json["gameResult"].as_str().unwrap_or("Unknown");
    let game_result_emoji = if game_result == "Victory" { "🏆" } else { "❌" };
    let color = if game_result == "Victory" { 0x00ff00 } else { 0xff0000 };

    // Get current UTC time and format it
    let now = Utc::now();
    let formatted_time = now.format("%Y/%m/%d %H:%M:%S").to_string();

    // Construct the embed title
    let title = format!(
        "**{}** - **Game Result : {} {} - {}**",
        player_name, game_result, game_result_emoji, formatted_time
    );

    // Define role order and corresponding emojis or image URLs
    let roles_order = ["TOP", "JUNGLE", "MIDDLE", "BOTTOM", "UTILITY"];
    let mut matchups_by_role = std::collections::HashMap::new();

    // Organize matchups by role
    if let Some(matchups) = info_json["matchups"].as_array() {
        for matchup in matchups {
            if let Some(role) = matchup["role"].as_str() {
                matchups_by_role.insert(role.to_uppercase(), matchup);
            }
        }
    }

    // Initialize the embed with title, color, and footer
    let mut embed = CreateEmbed::new()
        .title(title)
        .color(color); // You can choose any color you like

    // Iterate through each role and add a field for each matchup
    for role in &roles_order {
        if let Some(matchup) = matchups_by_role.get(&role.to_uppercase()) {
            let team_player = &matchup["team"];
            let enemy_player = &matchup["enemy"];
            let separator = "-----------------------------------------------------------------";
            let role_label = match *role {
                "TOP" => "**🔼 TOP**",
                "JUNGLE" => "**🌲 JUNGLE**",
                "MIDDLE" => "**🛣️ MIDDLE**",
                "BOTTOM" => "**🔽 BOTTOM**",
                "UTILITY" => "**🛡️ SUPPORT**",
                _ => "**UNKNOWN**",
            };
            //concatenation de separator et role_label
            let embed_name = format!("{}{}", separator, role_label);

            // Team player stats
            let team_stats = format!(

                "**{}** ({})\nK/D/A: **{}/{}/{}** | CS: **{}** | Gold: {} ({} g/m)",
                team_player["summonerName"].as_str().unwrap_or("Unknown"),
                team_player["championName"].as_str().unwrap_or("Unknown"),
                team_player["kills"].as_u64().unwrap_or(0),
                team_player["deaths"].as_u64().unwrap_or(0),
                team_player["assists"].as_u64().unwrap_or(0),
                team_player["totalFarm"].as_u64().unwrap_or(0),
                format_gold_k(team_player["goldEarned"].as_u64().unwrap_or(0)),
                team_player["goldPerMinute"].as_u64().unwrap_or(0)
            );

            // Enemy player stats
            let enemy_stats = format!(
                "**{}** ({})\nK/D/A: **{}/{}/{}** | CS: **{}** | Gold: {} ({} g/m)",
                enemy_player["summonerName"].as_str().unwrap_or("Unknown"),
                enemy_player["championName"].as_str().unwrap_or("Unknown"),
                enemy_player["kills"].as_u64().unwrap_or(0),
                enemy_player["deaths"].as_u64().unwrap_or(0),
                enemy_player["assists"].as_u64().unwrap_or(0),
                enemy_player["totalFarm"].as_u64().unwrap_or(0),
                format_gold_k(enemy_player["goldEarned"].as_u64().unwrap_or(0)),
                enemy_player["goldPerMinute"].as_u64().unwrap_or(0)
            );

            // Combine team and enemy stats
            let field_value = format!("{}\n{}", team_stats, enemy_stats);

            // Add the field to the embed
            embed = embed.field(
                embed_name,
                field_value,
                false
            );
        }
    }

    embed
}


fn extract_participant_stats(p: &Value) -> Value {
    let riot_id_game_name = p["riotIdGameName"].as_str().unwrap_or("Unknown");
    let summoner_name = if p["summonerName"].as_str().unwrap_or("Unknown").is_empty() { riot_id_game_name } else { p["summonerName"].as_str().unwrap_or("Unknown") };
    let champion_name = p["championName"].as_str().unwrap_or("Unknown");
    let kills = p["kills"].as_u64().unwrap_or(0);
    let deaths = p["deaths"].as_u64().unwrap_or(0);
    let assists = p["assists"].as_u64().unwrap_or(0);
    let total_minions_killed = p["totalMinionsKilled"].as_u64().unwrap_or(0);
    let neutral_minions_killed = p["neutralMinionsKilled"].as_u64().unwrap_or(0);
    let total_farm = total_minions_killed + neutral_minions_killed;
    let gold_per_minute = p["goldPerMinute"].as_u64().unwrap_or(0);
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
        "goldPerMinute": gold_per_minute,
        "visionScore": vision_score
    })
}

fn format_gold_k(gold: u64) -> String {
    if gold < 1000 {
        gold.to_string()
    } else {
        let gold_f64 = gold as f64 / 1000.0;
        if gold_f64.fract() == 0.0 {
            format!("{}k", gold_f64 as u64)
        } else {
            let formatted = format!("{:.1}", gold_f64).replace('.', ",");
            format!("{}k", formatted)
        }
    }
}
