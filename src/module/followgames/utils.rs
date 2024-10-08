use crate::embed::schedule_message_deletion;
use crate::embed::{create_embed_error, create_embed_sucess};
use crate::models::data::{Data, SummonerFollowedData};
use crate::models::error::Error;
use crate::models::modal::FollowGamesModal;
use mongodb::bson::doc;

/// ⚙️ **Function**: Adds a summoner to the database for game follow-up if they are not already being followed.
///
/// This asynchronous function checks if a summoner is already being followed by querying the MongoDB collection using their `puuid`.
/// If they are not followed yet, it adds the summoner's data to the database and returns a success message. If the summoner is already followed, an error message is returned.
///
/// # Parameters:
/// - `collection`: A MongoDB collection (`mongodb::Collection<SummonerFollowedData>`) where the summoner's follow data is stored.
/// - `ctx`: The `poise::ApplicationContext` provides the context for the Discord interaction, including the ability to send responses.
/// - `modal_data`: A `FollowGamesModal` struct containing the user's input data from the modal (game name, tag line, etc.).
/// - `region_str`: A string representing the summoner's region (e.g., "NA", "EUW").
/// - `puuid`: A string containing the summoner's unique PUUID (player unique identifier from Riot's API).
/// - `guild_id`: An integer representing the ID of the Discord guild.
/// - `summoner_id`: A string containing the summoner's unique Summoner ID from Riot's API.
/// - `match_id`: A string representing the summoner's latest match ID.
/// - `time_end_follow`: A string representing the timestamp for when the follow period ends.
///
/// # Returns:
/// - `Result<(), Error>`: Returns an empty result if the operation is successful, or an error if any part of the process fails.
///
/// # Example:
/// This function is used internally to add a summoner to the follow list after a successful interaction with the `/followgames` command:
///
/// ```rust
/// check_and_add_in_db(collection, ctx, modal_data, region_str, puuid, summoner_id, match_id, time_end_follow).await?;
/// ```
///
/// # Notes:
/// - If the user is already being followed, an error message is sent to the Discord channel using `create_embed_error`.
/// - If the user is successfully added to the database, a success message is sent using `create_embed_sucess`.
/// - The function makes sure to handle errors from both MongoDB operations and Discord message sending by logging appropriate error messages.
pub async fn check_and_add_in_db(
    collection: mongodb::Collection<SummonerFollowedData>,
    ctx: poise::ApplicationContext<'_, Data, Error>,
    modal_data: FollowGamesModal,
    region_str: String,
    puuid: String,
    summoner_id: String,
    match_id: String,
    time_end_follow: String,
) -> Result<(), Error> {
    match collection.find_one(doc! { "puuid": puuid.clone() }).await {
        Ok(Some(_followed_summoner)) => {
            let guild_id = ctx.guild_id().map(|id| id.get()).unwrap_or(0).to_string();
            if _followed_summoner.guild_id == guild_id {
                match collection
                    .update_one(
                        doc! { "puuid": puuid.clone(), "guild_id": guild_id },
                        doc! { "$set": { "time_end_follow": time_end_follow.clone() } },
                    )
                    .await
                {
                    Ok(_) => {
                        let success_message = "Success, tracking time has been updated.";
                        let reply = ctx.send(create_embed_sucess(&success_message)).await?;
                        schedule_message_deletion(reply, ctx).await?;
                        return Ok(());
                    }
                    Err(_) => {
                        let error_message = "Error, failed to update tracking time.";
                        let reply = ctx.send(create_embed_error(&error_message)).await?;
                        schedule_message_deletion(reply, ctx).await?;
                        return Ok(());
                    }
                }
            } else {
                let guild_id = ctx.guild_id().map(|id| id.get()).unwrap_or(0).to_string();
                let channel_id = ctx.channel_id().get();
                let new_followed_summoner = SummonerFollowedData {
                    puuid: puuid.clone(),
                    summoner_id: summoner_id.clone(),
                    name: modal_data.game_name.clone(),
                    tag: modal_data.tag_line.clone(),
                    region: region_str.to_string(),
                    last_match_id: match_id.clone(),
                    time_end_follow: time_end_follow.clone(),
                    channel_id: channel_id,
                    guild_id: guild_id,
                };
                match collection.insert_one(new_followed_summoner).await {
                    Ok(_) => {
                        let sucess_message = "User has been followed.";
                        let reply = ctx.send(create_embed_sucess(&sucess_message)).await?;
                        schedule_message_deletion(reply, ctx).await?;
                        return Ok(());
                    }
                    Err(e) => {
                        let error_message = format!("Error inserting user to MongoDB: {}", e);
                        let reply = ctx.send(create_embed_error(&error_message)).await?;
                        schedule_message_deletion(reply, ctx).await?;
                        return Ok(());
                    }
                }
            }
        }
        Ok(None) => {
            let guild_id = ctx.guild_id().map(|id| id.get()).unwrap_or(0).to_string();
            let channel_id = ctx.channel_id().get();
            let new_followed_summoner = SummonerFollowedData {
                puuid: puuid.clone(),
                summoner_id: summoner_id.clone(),
                name: modal_data.game_name.clone(),
                tag: modal_data.tag_line.clone(),
                region: region_str.to_string(),
                last_match_id: match_id.clone(),
                time_end_follow: time_end_follow.clone(),
                channel_id: channel_id,
                guild_id: guild_id,
            };
            match collection.insert_one(new_followed_summoner).await {
                Ok(_) => {
                    let sucess_message = "User has been followed.";
                    let reply = ctx.send(create_embed_sucess(&sucess_message)).await?;
                    schedule_message_deletion(reply, ctx).await?;
                    return Ok(());
                }
                Err(e) => {
                    let error_message = format!("Error inserting user to MongoDB: {}", e);
                    let reply = ctx.send(create_embed_error(&error_message)).await?;
                    schedule_message_deletion(reply, ctx).await?;
                    return Ok(());
                }
            }
        }
        Err(e) => {
            let error_message = format!("Error collecting informations from MongoDB: {}", e);
            let reply = ctx.send(create_embed_error(&error_message)).await?;
            schedule_message_deletion(reply, ctx).await?;
            return Ok(());
        }
    }
}
