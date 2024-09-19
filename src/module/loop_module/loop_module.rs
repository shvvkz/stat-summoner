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

/// ⚙️ **Function**: Processes a followed summoner by checking if their follow time has expired or if they have new match data.
///
/// This asynchronous function processes an individual summoner from the database by first checking if the follow time
/// has expired. If the follow time is up, the summoner is removed from the database. If the summoner is still being followed,
/// the function checks for new match data and updates the database accordingly.
///
/// # Parameters:
/// - `collection`: A reference to the MongoDB `Collection<SummonerFollowedData>`, used to query and update the database for followed summoners.
/// - `followed_summoner`: A reference to a `SummonerFollowedData` struct, representing the summoner being processed.
/// - `riot_api_key`: A string slice representing the Riot API key, used to make authorized API requests.
/// - `http`: An `Arc<Http>` reference to the HTTP client for sending requests to the Riot API.
///
/// # Returns:
/// - `Result<(), Error>`: Returns an empty result if the operation is successful, or an error if any part of the process fails.
///
/// # Example:
/// This function is called as part of the update process for followed summoners:
///
/// ```rust
/// process_followed_summoner(&collection, &followed_summoner, riot_api_key, http.clone()).await?;
/// ```
///
/// # Notes:
/// - If the follow time for the summoner has expired, the summoner is removed from the database by calling `delete_follower`.
/// - If the follow time has not expired, the function checks for new matches and updates the database using `update_follower_if_new_match`.
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
fn is_follow_time_expired(
    followed_summoner: &SummonerFollowedData
    ) -> bool {
        let time_end_follow = followed_summoner.time_end_follow.parse::<i64>().unwrap_or(0);
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
            .delete_one(doc! { "puuid": &followed_summoner.puuid })
            .await?;
        Ok(())
    }

/// ⚙️ **Function**: Updates a followed summoner's last match ID if a new match is found, and sends the update to Discord.
///
/// This asynchronous function checks if the followed summoner has played a new match by comparing the `last_match_id` 
/// stored in the database with the latest match ID fetched from the Riot API. If a new match is found, the function updates 
/// the `last_match_id` in the MongoDB collection and sends the match update to Discord.
///
/// # Parameters:
/// - `collection`: A reference to the MongoDB `Collection<SummonerFollowedData>`, used to query and update the summoner's last match ID.
/// - `followed_summoner`: A reference to a `SummonerFollowedData` struct, representing the summoner being processed.
/// - `riot_api_key`: A string slice representing the Riot API key, required to fetch the summoner's match history.
/// - `http`: An `Arc<Http>` reference to the HTTP client used to send match updates to Discord.
///
/// # Returns:
/// - `Result<(), Error>`: Returns an empty result if successful, or an error if the update or match retrieval fails.
///
/// # Example:
/// This function is used to keep the summoner's match data up to date in the database and notify the Discord server if a new match is played:
///
/// ```rust
/// update_follower_if_new_match(&collection, &followed_summoner, riot_api_key, http.clone()).await?;
/// ```
///
/// # Notes:
/// - The function retrieves the latest match ID from Riot's API by calling `get_latest_match_id`. If the match ID differs from the one stored in the database, the `last_match_id` is updated.
/// - If a new match is found, the function sends a notification to Discord via `send_match_update_to_discord`.
/// - The summoner's data in the database is updated using the `$set` operator to modify the `last_match_id` field.
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

/// ⚙️ **Function**: Sends a match update to a Discord channel after a new game is detected for a followed summoner.
///
/// This asynchronous function retrieves match details from the Riot API for the given `match_id` and formats them into a Discord message embed. 
/// The function then sends this embed to the specified Discord channel associated with the `followed_summoner`.
///
/// # Parameters:
/// - `followed_summoner`: A reference to a `SummonerFollowedData` struct, representing the summoner whose match update is being sent.
/// - `summoner_id`: A string slice representing the summoner's ID, used to extract specific match details.
/// - `match_id`: A string slice representing the match ID for the latest game, used to query the Riot API for match details.
/// - `riot_api_key`: A string slice representing the Riot API key, required for authorized requests to the Riot API.
/// - `http`: An `Arc<Http>` reference to the HTTP client used to send the message to the Discord channel.
///
/// # Returns:
/// - `Result<(), Error>`: Returns an empty result if the message is successfully sent, or an error if the operation fails.
///
/// # Example:
/// This function is called after detecting a new match for a summoner, and it posts a match summary in the configured Discord channel:
///
/// ```rust
/// send_match_update_to_discord(&followed_summoner, summoner_id, match_id, riot_api_key, http.clone()).await?;
/// ```
///
/// # Notes:
/// - The function retrieves match information using `get_matchs_info` and extracts relevant details for the specified summoner using `get_match_details`.
/// - It builds a Discord message embed using `create_embed_loop` and sends the message to the Discord channel associated with the summoner's `channel_id`.
/// - The function sends the message asynchronously using Serenity's `send_message` method for the given `ChannelId`.
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
