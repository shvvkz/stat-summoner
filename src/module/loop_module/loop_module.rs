use crate::models::data::{ChampionData, CoreBuildData, EmojiId, RunesData, SummonerFollowedData};
use crate::models::error::Error;
use crate::module::loop_module::utils::{fetch_core_build, fetch_runes};
use crate::module::loop_module::utils::{get_followed_summoners, process_followed_summoner};
use crate::riot_api::open_dd_json;
use futures::executor::block_on;
use mongodb::bson::{self, doc};
use mongodb::Client;
use poise::serenity_prelude as serenity;
use select::predicate::Predicate;
use serenity::http::Http;
use std::sync::Arc;
use tokio::task;

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

/// ⚙️ **Function**: Fetches champion data from League of Graphs and updates MongoDB.
///
/// This asynchronous function retrieves champion statistics, rune data, and core build information
/// from the League of Graphs website. It processes the HTML content to extract data for each champion
/// and updates or inserts this information into the MongoDB database. If the champion already exists in the database,
/// it updates their data; otherwise, it inserts a new document.
///
/// # Parameters:
/// - `mongo_client`: A reference to the MongoDB `Client`, used to query and update the MongoDB database.
///
/// # Returns:
/// - `Result<(), Box<dyn std::error::Error>>`: Returns an empty result if successful, or an error if any part of the process fails.
///
/// # Example:
/// This function is typically called to fetch and update champion data in a scheduled task:
///
/// ```rust
/// fetch_champion_data(&mongo_client).await?;
/// ```
///
/// # Notes:
/// - The function starts by sending an HTTP request to the League of Graphs page to fetch champion build data.
/// - It parses the HTML content using the `select` crate, extracting details such as popularity, win rate, and ban rate for each champion.
/// - For each champion, it also retrieves runes and core build information using the `fetch_runes` and `fetch_core_build` functions.
/// - The MongoDB collection `champions_data` is then updated with the latest data for each champion. If the champion already exists, the data is updated; otherwise, a new entry is inserted.
/// - The function makes use of `task::spawn_blocking` to handle blocking operations during HTML parsing.
pub async fn fetch_champion_data(mongo_client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://www.leagueofgraphs.com/champions/builds";
    let dd_json = open_dd_json().await.unwrap();
    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await?;

    let body = res.text().await?;

    let results: Vec<ChampionData> = task::spawn_blocking(move || {
        let document = select::document::Document::from(body.as_str());
        let mut results = Vec::new();

        for node in document
            .find(select::predicate::Class("data_table").descendant(select::predicate::Name("tr")))
        {
            let cells: Vec<_> = node.find(select::predicate::Name("td")).collect();
            if cells.len() > 5 {
                let name = cells[1]
                    .find(select::predicate::Class("name"))
                    .next()
                    .unwrap()
                    .text()
                    .trim()
                    .to_string();
                let role_text = cells[1]
                    .find(select::predicate::Name("i"))
                    .next()
                    .unwrap()
                    .text();
                let roles: Vec<String> =
                    role_text.split(',').map(|r| r.trim().to_string()).collect();

                let popularity = cells[2]
                    .find(select::predicate::Attr("data-value", ()))
                    .next()
                    .unwrap()
                    .attr("data-value")
                    .unwrap()
                    .to_string();
                let winrate = cells[3]
                    .find(select::predicate::Attr("data-value", ()))
                    .next()
                    .unwrap()
                    .attr("data-value")
                    .unwrap()
                    .to_string();
                let banrate = cells[4]
                    .find(select::predicate::Attr("data-value", ()))
                    .next()
                    .unwrap()
                    .attr("data-value")
                    .unwrap()
                    .to_string();

                let id_name = dd_json["data"]
                    .as_object()
                    .and_then(|data| {
                        data.values()
                            .find(|champion| champion["name"].as_str().map_or(false, |n| n == name))
                    })
                    .and_then(|champion| champion["id"].as_str())
                    .unwrap_or(&name)
                    .to_string();
                let default_runes = RunesData {
                    parent_primary_rune: String::new(),
                    child_primary_rune_1: String::new(),
                    child_primary_rune_2: String::new(),
                    child_primary_rune_3: String::new(),
                    child_secondary_rune_1: String::new(),
                    child_secondary_rune_2: String::new(),
                    tertiary_rune_1: String::new(),
                    tertiary_rune_2: String::new(),
                    tertiary_rune_3: String::new(),
                };
                let default_core_build = CoreBuildData {
                    first: String::new(),
                    second: String::new(),
                    third: String::new(),
                };
                let runes = block_on(fetch_runes(&id_name.to_lowercase())).unwrap_or(default_runes);

                let core_build = block_on(fetch_core_build(&id_name.to_lowercase()))
                    .unwrap_or(default_core_build);

                results.push(ChampionData {
                    name: name,
                    id_name: id_name,
                    role: roles,
                    popularity: popularity,
                    winrate: winrate,
                    banrate: banrate,
                    runes: runes,
                    core_build: core_build,
                });
            }
        }
        results
    })
    .await?;

    let collection = mongo_client
        .database("stat-summoner")
        .collection::<ChampionData>("champions_data");

    for champion in results {
        let filter = doc! { "name": &champion.name };

        if let Some(_) = collection.find_one(filter.clone()).await? {
            let update = doc! {
                "$set": {
                    "role": champion.role,
                    "popularity": champion.popularity,
                    "winrate": champion.winrate,
                    "banrate": champion.banrate,
                    "id_name": champion.id_name,
                    "runes": bson::to_document(&champion.runes).unwrap(),
                    "core_build":  bson::to_document(&champion.core_build).unwrap()
                }
            };
            collection.update_one(filter, update).await?;
        } else {
            collection.insert_one(champion).await?;
        }
    }

    log::info!("Mise à jour des données MongoDB terminée.");
    Ok(())
}
