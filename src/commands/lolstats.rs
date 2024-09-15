use poise::Modal;
use reqwest::Client;
use std::collections::HashMap;
use crate::models::{Data, Error, LolStatsModal, Region};
use crate::riot_api::{get_puuid, get_summoner_id, get_rank_info, get_champions, get_matchs_id};
use crate::embed::{create_and_send_embed, create_embed_error, schedule_message_deletion};
use crate::utils::determine_solo_flex;

/// ⚙️ **Command Function**: Fetches and displays LoL player stats based on user input.
///
/// This Discord command allows a user to input their League of Legends in-game name and tag, then fetches 
/// the player's Solo/Duo and Flex rank, top champions, and recent match details from the Riot API. 
/// The results are displayed in a formatted embed and automatically deleted after 60 seconds.
///
/// # Parameters:
/// - `ctx`: The application context, providing access to Discord interaction methods and the Riot API key.
/// - `region`: The region selected by the user (e.g., `Region::EUW`, `Region::NA`) to fetch statistics from the appropriate server.
///
/// # Returns:
/// - `Result<(), Error>`: If successful, returns `Ok(())`, otherwise returns an error.
///
/// # ⚠️ Notes:
/// - The command opens a modal dialog to gather the player's in-game name and tag.
/// - The message displaying the player's stats is automatically deleted after 60 seconds to keep the chat clean.
///
/// # Example:
/// ```rust
/// lolstats(ctx, Region::EUW).await?;
/// ```
///
/// This command displays information such as:
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
#[poise::command(slash_command)]
pub async fn lolstats(
    ctx: poise::ApplicationContext<'_, Data, Error>,
    #[description = "Select your region"] region: Region,
    ) -> Result<(), Error> {
        let modal_data: LolStatsModal = match LolStatsModal::execute(ctx).await {
            Ok(Some(data)) => data,
            Ok(None) => return Err("Modal data not found.".into()),
            Err(_) => return Err("Failed to retrieve modal data.".into()),
        };

        let client = Client::new();
        let game_name_space = modal_data.game_name.replace(" ", "%20");

        let region_str = match region {
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
        };

        // Capturer chaque erreur et envoyer un message d'erreur si nécessaire
        let puuid = match get_puuid(&client, &game_name_space, &modal_data.tag_line, &ctx.data().riot_api_key).await {
            Ok(puuid) => puuid,
            Err(e) => {
                let error_message = format!("Error fetching PUUID: {}", e);
                let reply = ctx.send(create_embed_error(&error_message)).await?;
                let _ = schedule_message_deletion(reply, ctx).await;
                return Err(Error::from(""));
            }
        };

        let summoner_id = match get_summoner_id(&client, &region_str, &puuid, &ctx.data().riot_api_key).await {
            Ok(id) => id,
            Err(e) => {
                let error_message = format!("Error fetching summoner ID: {}", e);
                let reply = ctx.send(create_embed_error(&error_message)).await?;
                let _ = schedule_message_deletion(reply, ctx).await;
                return Err(Error::from(""));
            }
        };

        let rank_info = get_rank_info(&client, &region_str, &summoner_id, &ctx.data().riot_api_key).await?;
        let champions = get_champions(&client, &puuid, &region_str, &ctx.data().riot_api_key).await?;
        let match_ids = get_matchs_id(&client, &puuid, &ctx.data().riot_api_key).await?;

        let mut default_rank = HashMap::new();
        default_rank.insert("tier".to_string(), serde_json::Value::String("Unranked".to_string()));
        default_rank.insert("rank".to_string(), serde_json::Value::String("".to_string()));
        default_rank.insert("leaguePoints".to_string(), serde_json::Value::Number(0.into()));
        default_rank.insert("wins".to_string(), serde_json::Value::Number(0.into()));
        default_rank.insert("losses".to_string(), serde_json::Value::Number(0.into()));
        default_rank.insert("queueType".to_string(), serde_json::Value::String("".to_string()));
        let (solo_rank, flex_rank) = determine_solo_flex(&rank_info, &default_rank);
        let reply = create_and_send_embed(&modal_data, summoner_id, &solo_rank, &flex_rank, champions, match_ids, &ctx).await;
        let sent_message = ctx.send(reply).await?;
        if let Err(e) = schedule_message_deletion(sent_message, ctx).await {
            eprintln!("Failed to schedule message deletion: {}", e);
        }
        Ok(())
    }
