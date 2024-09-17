use crate::{embed::schedule_message_deletion, models::{Data, Error, FollowGamesModal, Region, SummonerFollowedData}};
use mongodb::bson::doc;
use tracing::log::error;
use poise::Modal;
use crate::embed::create_embed_error;

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
            return Ok(()); // Retourne Ok(()) pour terminer proprement
        },
        Err(_) => {
            let error_message = "Failed to retrieve modal data.";
            let reply = ctx.send(create_embed_error(&error_message)).await?;
            schedule_message_deletion(reply, ctx).await?;
            return Ok(()); // Retourne Ok(()) pour terminer proprement
        },
    };

    // Valider et convertir `time_followed` en u32
    let time_followed = match modal_data.time_followed.trim().parse::<u32>() {
        Ok(value) => value,
        Err(_) => {
            let error_message = "Invalid time format. Please enter a valid number of hours.";
            let reply = ctx.send(create_embed_error(&error_message)).await?;
            schedule_message_deletion(reply, ctx).await?;
            return Ok(()); // Retourne Ok(()) pour terminer proprement
        }
    };

    if time_followed == 0 || time_followed > 48 {
        let error_message = "Please enter a time between 1 and 48 hours.".to_string();
        let reply = ctx.send(create_embed_error(&error_message)).await?;
        schedule_message_deletion(reply, ctx).await?;
        return Ok(()); // Retourne Ok(()) pour terminer proprement
    }

    let mongo_client = &ctx.data().mongo_client;
    let collection = mongo_client
        .database("stat-summoner")
        .collection::<SummonerFollowedData>("follower_summoner");

    let puuid_to_search = "456";
    match collection.find_one(doc! { "puuid": puuid_to_search }).await {
        Ok(Some(followed_summoner)) => {
            let last_match_id = format!("Dernier match ID: {}", followed_summoner.last_match_id);
            ctx.say(format!(
                "Données du joueur trouvées : {:?}, {}",
                followed_summoner, last_match_id
            ))
            .await?;
        }
        Ok(None) => {
            ctx.say("Aucune donnée de joueur trouvée pour ce PUUID.").await?;
        }
        Err(e) => {
            error!("Erreur lors de la recherche MongoDB: {:?}", e);
            ctx.say("Erreur lors de la recherche MongoDB.").await?;
        }
    }

    Ok(())
}
