use chrono::Utc;
use crate::models::data::SummonerFollowedData;
use poise::serenity_prelude::{self as serenity, CreateMessage};
use crate::riot_api::{get_matchs_id, get_matchs_info};
use serenity::http::Http;
use crate::module::loop_module::utils::{get_match_details, create_embed_loop};
use mongodb::{Client, Collection};
use mongodb::bson::doc;
use std::sync::Arc;
use futures::StreamExt;
use crate::models::error::Error;

pub async fn check_and_update_db(
    mongo_client: &Client,
    riot_api_key: &str,
    http: Arc<Http>,
) -> Result<(), Error> {
    let collection = mongo_client
        .database("stat-summoner")
        .collection::<SummonerFollowedData>("follower_summoner");

    let count = collection.estimated_document_count().await?;

    if count > 0 {
        println!("La base de données contient {} documents.", count);

        let followed_summoners = get_followed_summoners(&collection).await?;

        for followed_summoner in followed_summoners {
            process_followed_summoner(&collection, &followed_summoner, riot_api_key, http.clone())
                .await?;
        }
    }

    Ok(())
}

async fn get_followed_summoners(
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

async fn process_followed_summoner(
    collection: &Collection<SummonerFollowedData>,
    followed_summoner: &SummonerFollowedData,
    riot_api_key: &str,
    http: Arc<Http>,
) -> Result<(), Error> {
    if is_follow_time_expired(followed_summoner) {
        delete_follower(collection, followed_summoner).await?;
    } else {
        update_follower_if_new_match(collection, followed_summoner, riot_api_key, http).await?;
    }
    Ok(())
}

fn is_follow_time_expired(followed_summoner: &SummonerFollowedData) -> bool {
    let time_end_follow = followed_summoner.time_end_follow.parse::<i64>().unwrap_or(0);
    let current_timestamp = Utc::now().timestamp();
    current_timestamp > time_end_follow
}

async fn delete_follower(
    collection: &Collection<SummonerFollowedData>,
    followed_summoner: &SummonerFollowedData,
) -> Result<(), mongodb::error::Error> {
    eprintln!("Suppression de {}", followed_summoner.puuid);
    collection
        .delete_one(doc! { "puuid": &followed_summoner.puuid })
        .await?;
    Ok(())
}

async fn update_follower_if_new_match(
    collection: &Collection<SummonerFollowedData>,
    followed_summoner: &SummonerFollowedData,
    riot_api_key: &str,
    http: Arc<Http>,
) -> Result<(), Error> {
    let puuid = &followed_summoner.puuid;
    let summoner_id = &followed_summoner.summoner_id;
    let last_match_id = &followed_summoner.last_match_id;
    let client = reqwest::Client::new();

    let match_id_from_riot = get_latest_match_id(&client, puuid, riot_api_key).await?;

    if last_match_id != &match_id_from_riot {
        collection
            .update_one(
                doc! { "puuid": puuid },
                doc! { "$set": { "last_match_id": &match_id_from_riot } },
            )
            .await?;

        send_match_update_to_discord(
            followed_summoner,
            summoner_id,
            &match_id_from_riot,
            riot_api_key,
            http,
        )
        .await?;
    }

    Ok(())
}

async fn get_latest_match_id(
    client: &reqwest::Client,
    puuid: &str,
    riot_api_key: &str,
) -> Result<String, Error> {
    let matches = get_matchs_id(client, puuid, riot_api_key, 1).await?;
    Ok(matches[0].clone())
}

async fn send_match_update_to_discord(
    followed_summoner: &SummonerFollowedData,
    summoner_id: &str,
    match_id: &str,
    riot_api_key: &str,
    http: Arc<Http>,
) -> Result<(), Error> {
    let client = reqwest::Client::new();
    let info = get_matchs_info(&client, match_id, riot_api_key).await?;
    let info_json = get_match_details(&info, summoner_id).unwrap();
    let channel_id = serenity::model::id::ChannelId::new(followed_summoner.channel_id);
    let embed = create_embed_loop(&info_json, &followed_summoner.name);
    let builder = CreateMessage::new().add_embed(embed);
    let _ = channel_id.send_message(&http, builder).await;
    Ok(())
}
