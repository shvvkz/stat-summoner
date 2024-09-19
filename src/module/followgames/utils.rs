
use crate::embed::schedule_message_deletion;
use crate::models::data::{Data, SummonerFollowedData};
use crate::models::error::Error;
use crate::models::modal::FollowGamesModal;
use mongodb::bson::doc;
use crate::embed::{create_embed_error, create_embed_sucess};

pub async fn add_user_to_db(
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
            let error_message = "Error user already followed.";
            let reply = ctx.send(create_embed_error(&error_message)).await?;
            schedule_message_deletion(reply, ctx).await?;
            return Ok(());
        }
        Ok(None) => {
            // get discord id from ctx
            let guild_id = ctx.guild_id().map(|id| id.get()).unwrap_or(0);
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
                guild_id: guild_id
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