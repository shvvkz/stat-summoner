use crate::embed::create_embed_error;
use crate::embed::schedule_message_deletion;
use crate::models::data::Data;
use crate::models::data::SummonerFollowedData;
use crate::models::error::Error;
use crate::models::modal::FollowGamesModal;
use crate::models::region::Region;
use crate::module::followgames::utils::check_and_add_in_db;
use crate::riot_api::{get_matchs_id, get_puuid, get_summoner_id};
use crate::utils::region_to_string;
use chrono::{Duration, Utc};
use poise::Modal;

/// Starts following a player's games for a specified duration.
///
/// This slash command allows a user to follow a player's games for a certain amount of time (between 1 and 48 hours).
/// It retrieves the player's PUUID and Summoner ID from the Riot API, and stores their information in the database
/// for tracking future games.
///
/// # Parameters:
/// - `ctx`: The `poise::ApplicationContext` provides the context in which the command is executed, including access to the Discord interaction and data.
/// - `region`: A `Region` enum value selected by the user, indicating the player's region (e.g., NA, EUW, etc.).
///
/// # Returns:
/// - `Result<(), Error>`: Returns an empty result if successful, or an error if the process fails.
///
/// # Example:
/// This command can be triggered in Discord using the `/followgames` command, and requires the user to input their game name, tagline, and duration for following:
///
/// ```rust
/// /followgames region: NA
/// ```
///
/// # Flow:
/// 1. The command opens a modal where the user inputs their game name, tag line, and duration to follow the games.
/// 2. It validates the input and ensures that the follow duration is between 1 and 48 hours.
/// 3. The Riot API is queried to retrieve the player's PUUID and Summoner ID.
/// 4. The player's data is stored in the database, allowing the bot to follow their games for the specified duration.
///
/// # Notes:
/// - The command opens a modal using `FollowGamesModal::execute` to collect the player's game name and follow duration.
/// - If the follow duration is invalid or the player is not found, an error message is sent to the Discord channel.
/// - The player's PUUID and Summoner ID are fetched from the Riot API and stored in the MongoDB database, enabling game tracking.
#[poise::command(slash_command)]
pub async fn followgames(
    ctx: poise::ApplicationContext<'_, Data, Error>,
    #[description = "Select your region"] region: Region,
) -> Result<(), Error> {
    let modal_data = match FollowGamesModal::execute(ctx).await {
        Ok(Some(data)) => data,
        Ok(None) => {
            let error_message = "Modal data not found.";
            let reply = ctx.send(create_embed_error(&error_message)).await?;
            schedule_message_deletion(reply, ctx).await?;
            return Ok(());
        }
        Err(_) => {
            let error_message = "Failed to retrieve modal data.";
            let reply = ctx.send(create_embed_error(&error_message)).await?;
            schedule_message_deletion(reply, ctx).await?;
            return Ok(());
        }
    };

    let time_followed = match modal_data.time_followed.trim().parse::<u32>() {
        Ok(value) => value,
        Err(_) => {
            let error_message = "Invalid time format. Please enter a valid number of hours.";
            let reply = ctx.send(create_embed_error(&error_message)).await?;
            schedule_message_deletion(reply, ctx).await?;
            return Ok(());
        }
    };

    if time_followed == 0 || time_followed > 48 {
        let error_message = "Please enter a time between 1 and 48 hours.".to_string();
        let reply = ctx.send(create_embed_error(&error_message)).await?;
        schedule_message_deletion(reply, ctx).await?;
        return Ok(());
    }

    let client = reqwest::Client::new();
    let game_name_space = modal_data.game_name.replace(" ", "%20");
    let region_str = region_to_string(&region);
    let puuid = match get_puuid(
        &client,
        &game_name_space,
        &modal_data.tag_line,
        &ctx.data().riot_api_key,
    )
    .await
    {
        Ok(puuid) => puuid,
        Err(e) => {
            let error_message = format!("{}", e);
            let reply = ctx.send(create_embed_error(&error_message)).await?;
            schedule_message_deletion(reply, ctx).await?;
            return Ok(());
        }
    };

    let summoner_id =
        match get_summoner_id(&client, &region_str, &puuid, &ctx.data().riot_api_key).await {
            Ok(id) => id,
            Err(e) => {
                let error_message = format!("{}", e);
                let reply = ctx.send(create_embed_error(&error_message)).await?;
                schedule_message_deletion(reply, ctx).await?;
                return Ok(());
            }
        };
    let match_id = get_matchs_id(&client, &puuid, &ctx.data().riot_api_key, 1)
        .await
        .unwrap()[0]
        .to_string();
    let time_end_follow = (Utc::now() + Duration::hours(time_followed as i64))
        .timestamp()
        .to_string();
    eprint!("match_id: {:?}", match_id);
    let mongo_client = &ctx.data().mongo_client;
    let collection = mongo_client
        .database("stat-summoner")
        .collection::<SummonerFollowedData>("follower_summoner");

    check_and_add_in_db(
        collection,
        ctx,
        modal_data,
        region_str,
        puuid,
        summoner_id,
        match_id,
        time_end_follow,
    )
    .await?;
    Ok(())
}
