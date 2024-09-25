use crate::models::data::{EmojiId, SummonerFollowedData};
use crate::models::error::Error;
use crate::module::loop_module::utils::{create_embed_loop, get_match_details};
use crate::riot_api::{get_matchs_id, get_matchs_info};
use chrono::Utc;
use futures::StreamExt;
use mongodb::bson::doc;
use mongodb::{Client, Collection};
use poise::serenity_prelude::{self as serenity, CreateMessage};
use serenity::http::Http;
use std::sync::Arc;

/// ⚙️ **Function**: Checks the database for followed summoners and updates their information from the Riot API.
///
/// This asynchronous function queries the MongoDB collection to check if there are any followed summoners.
/// If documents are present, it retrieves the followed summoners and processes each one by fetching the latest data from the Riot API.
/// The function is designed to keep the database in sync with real-time summoner information.
///
/// # Parameters:
/// - `mongo_client`: A reference to the MongoDB `Client`, used to query and update the database.
/// - `riot_api_key`: A string slice representing the Riot API key, required to make authorized API calls.
/// - `http`: An `Arc<Http>` reference to the HTTP client used for making requests to the Riot API.
///
/// # Returns:
/// - `Result<(), Error>`: Returns an empty result if successful, or an error if any part of the process fails.
///
/// # Example:
/// This function is used to periodically check and update summoner information.
///
/// ```rust
/// check_and_update_db(&mongo_client, riot_api_key, http.clone()).await?;
/// ```
///
/// # Notes:
/// - The function first checks if there are any documents in the `follower_summoner` collection. If the collection is empty, no further action is taken.
/// - For each followed summoner, the function retrieves their latest match data using the Riot API and updates the database accordingly.
pub async fn check_and_update_db(
    mongo_client: &Client,
    riot_api_key: &str,
    http: Arc<Http>,
) -> Result<(), Error> {
    let collection = mongo_client
        .database("stat-summoner")
        .collection::<SummonerFollowedData>("follower_summoner");
    let collection_emoji = mongo_client
        .database("stat-summoner")
        .collection::<EmojiId>("emojis_id");
    let count = collection.estimated_document_count().await?;

    if count > 0 {
        let followed_summoners = get_followed_summoners(&collection).await?;
        for followed_summoner in followed_summoners {
            process_followed_summoner(
                &collection,
                &followed_summoner,
                riot_api_key,
                http.clone(),
                collection_emoji.clone(),
            )
            .await?;
        }
    }

    Ok(())
}

/// ⚙️ **Function**: Retrieves all followed summoners from the database.
///
/// This asynchronous function queries the "follower_summoner" collection in the "stat-summoner" MongoDB database
/// to retrieve all documents representing followed summoners. It collects each `SummonerFollowedData` into a vector.
///
/// # Parameters:
/// - `collection`: A reference to the MongoDB collection containing `SummonerFollowedData` documents.
///
/// # Returns:
/// - `Result<Vec<SummonerFollowedData>, mongodb::error::Error>`: A vector of followed summoners if successful, or an error if the query fails.
///
/// # ⚠️ Notes:
/// - Prints an error message in French if a document retrieval fails.
/// - Ensure that the `SummonerFollowedData` struct aligns with the collection's document structure.
///
/// # Example:
/// ```rust
/// let summoners = get_followed_summoners(&collection).await?;
/// println!("Retrieved {} summoners.", summoners.len());
/// ```
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

/// ⚙️ **Function**: Processes a followed summoner by checking if their follow time has expired or if they have played a new match.
///
/// This asynchronous function handles the logic for a followed summoner. It checks if the follow time has expired and removes the summoner from the database if necessary. If the follow time is still valid, it checks for new matches and updates the summoner's information accordingly.
///
/// # Parameters:
/// - `collection`: A reference to a MongoDB `Collection<SummonerFollowedData>` that stores the followed summoners' data.
/// - `followed_summoner`: A reference to a `SummonerFollowedData` struct containing the summoner's information, including their follow duration and last match details.
/// - `riot_api_key`: A string slice containing the Riot Games API key for authenticating the API request.
/// - `http`: An `Arc<Http>` object used to send messages via the Discord API.
/// - `collection_emojis`: A MongoDB `Collection` containing emoji mappings, used to enrich the Discord embeds with custom emojis for roles and champions.
///
/// # Returns:
/// - `Result<(), Error>`: Returns `Ok(())` if the summoner was successfully processed (either by removing them from the database or updating their match info), or an error if something went wrong.
///
/// # Example:
/// This function is typically called as part of a loop or scheduled task that checks the status of followed summoners:
///
/// ```rust
/// let result = process_followed_summoner(collection, &followed_summoner, riot_api_key, http.clone(), collection_emojis).await;
/// if result.is_err() {
///     // Handle error (e.g., log failure or retry)
/// }
/// ```
///
/// # Notes:
/// - The function begins by checking if the follow time for the summoner has expired using the `is_follow_time_expired` function.
/// - If the follow time has expired, the summoner is removed from the MongoDB collection by calling `delete_follower`.
/// - If the summoner is still being followed, the function calls `update_follower_if_new_match` to check for new matches and potentially send an update to the associated Discord channel.
/// - This function ensures that summoners are only followed for the specified duration and that Discord channels are updated with relevant match information during the follow period.
async fn process_followed_summoner(
    collection: &Collection<SummonerFollowedData>,
    followed_summoner: &SummonerFollowedData,
    riot_api_key: &str,
    http: Arc<Http>,
    collection_emojis: Collection<EmojiId>,
) -> Result<(), Error> {
    if is_follow_time_expired(followed_summoner) {
        delete_follower(collection, followed_summoner).await?;
    } else {
        update_follower_if_new_match(
            collection,
            followed_summoner,
            riot_api_key,
            http,
            collection_emojis,
        )
        .await?;
    }
    Ok(())
}

/// ⚙️ **Function**: Determines if the follow time for a summoner has expired.
///
/// This function checks whether the current timestamp exceeds the stored follow end time for a summoner.
/// It parses the `time_end_follow` field from the `SummonerFollowedData` struct, compares it to the current UTC timestamp,
/// and returns `true` if the follow time has expired, or `false` otherwise.
///
/// # Parameters:
/// - `followed_summoner`: A reference to a `SummonerFollowedData` struct, which contains information about the summoner, including when the follow period ends.
///
/// # Returns:
/// - `bool`: Returns `true` if the current time is greater than the stored `time_end_follow`, meaning the follow period has expired. Returns `false` if the follow period is still active.
///
/// # Example:
/// This function is used to determine whether a summoner should be removed from the list of followed summoners.
///
/// ```rust
/// let expired = is_follow_time_expired(&followed_summoner);
/// if expired {
///     // Remove summoner from database
/// }
/// ```
///
/// # Notes:
/// - The function uses UTC time for comparison and assumes the `time_end_follow` is a valid timestamp that can be parsed into an `i64`. If parsing fails, it defaults to 0, which will always result in `true`.
fn is_follow_time_expired(followed_summoner: &SummonerFollowedData) -> bool {
    let time_end_follow = followed_summoner
        .time_end_follow
        .parse::<i64>()
        .unwrap_or(0);
    let current_timestamp = Utc::now().timestamp();
    current_timestamp > time_end_follow
}

/// ⚙️ **Function**: Deletes a followed summoner from the database.
///
/// This asynchronous function removes a summoner from the `follower_summoner` collection in MongoDB based on their `puuid`.
/// It logs the deletion action and ensures the summoner is no longer tracked in the database.
///
/// # Parameters:
/// - `collection`: A reference to the MongoDB `Collection<SummonerFollowedData>`, used to interact with the database and delete the summoner data.
/// - `followed_summoner`: A reference to a `SummonerFollowedData` struct, representing the summoner that is being deleted. The deletion is based on the summoner's `puuid`.
///
/// # Returns:
/// - `Result<(), mongodb::error::Error>`: Returns an empty result if successful, or an error if the deletion fails.
///
/// # Example:
/// This function is typically called when the follow time for a summoner has expired, and they need to be removed from the database:
///
/// ```rust
/// delete_follower(&collection, &followed_summoner).await?;
/// ```
///
/// # Notes:
/// - The `puuid` field is used as the unique identifier for deletion from the MongoDB collection.
/// - The function logs the `puuid` of the summoner being deleted using `eprintln!`, which outputs the message to the standard error stream.
async fn delete_follower(
    collection: &Collection<SummonerFollowedData>,
    followed_summoner: &SummonerFollowedData,
) -> Result<(), mongodb::error::Error> {
    eprintln!("Suppression de {}", followed_summoner.puuid);
    collection
        .delete_one(
            doc! { "puuid": &followed_summoner.puuid, "guild_id": &followed_summoner.guild_id },
        )
        .await?;
    Ok(())
}

/// ⚙️ **Function**: Updates a followed summoner's last match ID and sends a Discord update if a new match is detected.
///
/// This asynchronous function checks if a followed summoner has played a new match. If a new match is detected,
/// it updates the summoner's last match ID in the MongoDB collection and sends a match update to the appropriate Discord channel.
///
/// # Parameters:
/// - `collection`: A reference to a MongoDB `Collection<SummonerFollowedData>` that stores the followed summoners' data.
/// - `followed_summoner`: A reference to a `SummonerFollowedData` struct containing the summoner's information, including their PUUID, summoner ID, and last match ID.
/// - `riot_api_key`: A string slice containing the Riot Games API key for authenticating the API request.
/// - `http`: An `Arc<Http>` object used to send messages via the Discord API.
/// - `collection_emojis`: A MongoDB `Collection` containing emoji mappings, used to enhance the Discord embed with custom emojis for roles and champions.
///
/// # Returns:
/// - `Result<(), Error>`: Returns `Ok(())` if the last match ID was successfully updated and the match update was sent to Discord, or an error if something went wrong.
///
/// # Example:
/// This function is typically called periodically to check if a followed summoner has played a new match:
///
/// ```rust
/// let result = update_follower_if_new_match(collection, &followed_summoner, riot_api_key, http.clone(), collection_emojis).await;
/// if result.is_err() {
///     // Handle error (e.g., log failure or retry)
/// }
/// ```
///
/// # Notes:
/// - The function begins by creating an HTTP client using `reqwest` and fetching the latest match ID for the summoner using the `get_latest_match_id` function.
/// - If the new match ID is different from the stored `last_match_id`, the function updates the MongoDB collection with the new match ID.
/// - Once the database is updated, the function calls `send_match_update_to_discord` to send a match update to the Discord channel associated with the summoner.
/// - This function ensures that the Discord server is notified whenever the summoner completes a new match, keeping followers updated in real time.
async fn update_follower_if_new_match(
    collection: &Collection<SummonerFollowedData>,
    followed_summoner: &SummonerFollowedData,
    riot_api_key: &str,
    http: Arc<Http>,
    collection_emojis: Collection<EmojiId>,
) -> Result<(), Error> {
    let puuid = &followed_summoner.puuid;
    let summoner_id = &followed_summoner.summoner_id;
    let last_match_id = &followed_summoner.last_match_id;
    let guild_id = &followed_summoner.guild_id;
    let client = reqwest::Client::new();

    let match_id_from_riot = get_latest_match_id(&client, puuid, riot_api_key).await?;

    if last_match_id != &match_id_from_riot {
        collection
            .update_one(
                doc! {
                "puuid": puuid,
                "guild_id": guild_id
                },
                doc! { "$set": { "last_match_id": &match_id_from_riot } },
            )
            .await?;
        send_match_update_to_discord(
            followed_summoner,
            summoner_id,
            &match_id_from_riot,
            riot_api_key,
            http,
            collection_emojis,
        )
        .await?;
    }
    Ok(())
}

/// ⚙️ **Function**: Fetches the latest match ID for a given summoner using their PUUID.
///
/// This asynchronous function retrieves the most recent match ID for a summoner by making a request to the Riot API.
/// It uses the summoner's `puuid` to query their match history and returns the match ID of the most recent game.
///
/// # Parameters:
/// - `client`: A reference to the `reqwest::Client`, used to make HTTP requests to the Riot API.
/// - `puuid`: A string slice representing the summoner's PUUID (a unique identifier for each player in Riot's system).
/// - `riot_api_key`: A string slice representing the Riot API key, used for authorized requests.
///
/// # Returns:
/// - `Result<String, Error>`: Returns the latest match ID as a string if successful, or an error if the request or retrieval fails.
///
/// # Example:
/// This function is typically used to get the latest match ID for a summoner in order to check for new matches:
///
/// ```rust
/// let latest_match_id = get_latest_match_id(&client, puuid, riot_api_key).await?;
/// ```
///
/// # Notes:
/// - The function calls `get_matchs_id` to retrieve the match history and then returns the first match in the list, which corresponds to the most recent match.
/// - The `get_matchs_id` function is expected to return a vector of match IDs, from which the latest match (the first one) is extracted and returned.
async fn get_latest_match_id(
    client: &reqwest::Client,
    puuid: &str,
    riot_api_key: &str,
) -> Result<String, Error> {
    let matches = get_matchs_id(client, puuid, riot_api_key, 1).await?;
    Ok(matches[0].clone())
}

/// ⚙️ **Function**: Sends a match update to a specific Discord channel for a followed summoner.
///
/// This asynchronous function fetches match information for a followed summoner from the Riot API,
/// formats the details into an embed, and sends the embed as a message to the specified Discord channel.
///
/// # Parameters:
/// - `followed_summoner`: A reference to a `SummonerFollowedData` struct, which contains the summoner's name and the ID of the Discord channel to which the match update should be sent.
/// - `summoner_id`: A string slice representing the summoner's ID, used to identify the player's stats in the match.
/// - `match_id`: A string slice representing the match ID, used to fetch match details from the Riot API.
/// - `riot_api_key`: A string slice containing the Riot Games API key for authenticating the API request.
/// - `http`: An `Arc<Http>` object used to send messages via the Discord API.
/// - `collection_emojis`: A MongoDB `Collection` containing emoji mappings, used to add custom emojis to the embed for roles and champions.
///
/// # Returns:
/// - `Result<(), Error>`: Returns `Ok(())` if the match update was successfully sent to the Discord channel, or an error if something went wrong.
///
/// # Example:
/// This function is typically called after detecting that a followed summoner has completed a match:
///
/// ```rust
/// let result = send_match_update_to_discord(&followed_summoner, summoner_id, match_id, riot_api_key, http.clone(), collection_emojis).await;
/// if result.is_err() {
///     // Handle error (e.g., log failure or retry)
/// }
/// ```
///
/// # Notes:
/// - The function creates an HTTP client using `reqwest` to fetch match information from the Riot API.
/// - It retrieves detailed match data using the `get_matchs_info` and `get_match_details` functions.
/// - The function constructs a `CreateEmbed` object using the `create_embed_loop` function, which formats match statistics and adds emojis.
/// - The embed is sent as a message to the Discord channel specified in the `followed_summoner` struct.
/// - The Discord message is built using `CreateMessage` and sent asynchronously to the appropriate channel using the Discord API.
async fn send_match_update_to_discord(
    followed_summoner: &SummonerFollowedData,
    summoner_id: &str,
    match_id: &str,
    riot_api_key: &str,
    http: Arc<Http>,
    collection_emojis: Collection<EmojiId>,
) -> Result<(), Error> {
    let client = reqwest::Client::new();
    let info = get_matchs_info(&client, match_id, riot_api_key).await?;
    let info_json = get_match_details(&info, summoner_id).unwrap();
    let channel_id = serenity::model::id::ChannelId::new(followed_summoner.channel_id);
    let embed = create_embed_loop(&info_json, &followed_summoner.name, collection_emojis).await;
    let builder = CreateMessage::new().add_embed(embed);
    let _ = channel_id.send_message(&http, builder).await;
    Ok(())
}
