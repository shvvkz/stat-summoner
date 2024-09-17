use crate::riot_api::{get_matchs_id, get_puuid, get_summoner_id};
use crate::embed::schedule_message_deletion;
use crate::models::{Data, Error, FollowGamesModal, Region, SummonerFollowedData};
use mongodb::bson::doc;
use poise::Modal;
use crate::embed::{create_embed_error, create_embed_sucess};
use crate::utils::region_to_string;
use chrono::{Utc, Duration};

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
        },
        Err(_) => {
            let error_message = "Failed to retrieve modal data.";
            let reply = ctx.send(create_embed_error(&error_message)).await?;
            schedule_message_deletion(reply, ctx).await?;
            return Ok(());
        },
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

    let puuid = match get_puuid(&client, &game_name_space, &modal_data.tag_line, &ctx.data().riot_api_key).await {
        Ok(puuid) => puuid,
        Err(e) => {
            let error_message = format!("{}", e);
            let reply = ctx.send(create_embed_error(&error_message)).await?;
            schedule_message_deletion(reply, ctx).await?;
            return Ok(());
        }
    };

    let summoner_id = match get_summoner_id(&client, &region_str, &puuid, &ctx.data().riot_api_key).await {
        Ok(id) => id,
        Err(e) => {
            let error_message = format!("{}", e);
            let reply = ctx.send(create_embed_error(&error_message)).await?;
            schedule_message_deletion(reply, ctx).await?;
            return Ok(());
        }
    };
    let match_id = get_matchs_id(&client, &puuid, &ctx.data().riot_api_key, 1).await.unwrap()[0].to_string();
    let time_end_follow = (Utc::now() + Duration::hours(time_followed as i64)).timestamp().to_string();
    eprint!("match_id: {:?}", match_id);
    let mongo_client = &ctx.data().mongo_client;
    let collection = mongo_client
        .database("stat-summoner")
        .collection::<SummonerFollowedData>("follower_summoner");

    add_user_to_db(collection, ctx, modal_data, region_str, puuid, summoner_id, match_id, time_end_follow).await?;
    Ok(())
}

async fn add_user_to_db(
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
            let new_followed_summoner = SummonerFollowedData {
                puuid: puuid.clone(),
                summoner_id: summoner_id.clone(),
                name: modal_data.game_name.clone(),
                tag: modal_data.tag_line.clone(),
                region: region_str.to_string(),
                last_match_id: match_id.clone(),
                time_end_follow: time_end_follow.clone(),
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