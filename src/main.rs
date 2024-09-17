mod riot_api;
mod models;
mod embed;
mod utils;
mod commands;

use models::Data;
use poise::serenity_prelude::{self as serenity};
use riot_api::get_matchs_id;
use shuttle_runtime::SecretStore;
use shuttle_serenity::ShuttleSerenity;
use commands::lolstats::lolstats;
use commands::followgames::followgames;
use mongodb::{Client, options::{ClientOptions, ServerApi, ServerApiVersion}};
use mongodb::bson::doc;
use tokio::time::{sleep, Duration};
use tracing::log::error;
use futures::stream::StreamExt;

/// ⚙️ **Function**: Initializes and starts the Discord bot using the Shuttle runtime and Poise framework.
///
/// This function is the entry point for the Discord bot. It retrieves secrets (like the Discord token and Riot API key)
/// from the Shuttle runtime, sets up the bot's framework with its registered commands, and then starts the bot client
/// with the required intents.
///
/// # Parameters:
/// - `secret_store`: The Shuttle runtime secret store, which holds sensitive information such as the Discord token and Riot API key.
///
/// # Returns:
/// - `ShuttleSerenity`: An instance of the Serenity client wrapped in a result. It starts the bot client once all setup is complete.
///
/// # ⚠️ Notes:
/// - The bot framework is built using the Poise framework, which is designed for building Discord bots easily.
/// - The `lolstats` command is registered globally, meaning it will be available in all servers the bot is in.
/// - The function uses non-privileged gateway intents, meaning it doesn't request sensitive Discord information such as message content or member lists.
///
/// # Example:
/// This function is called automatically when the bot is deployed and run in the Shuttle environment.
///
/// ```rust
/// #[shuttle_runtime::main]
/// async fn main(secret_store: SecretStore) -> ShuttleSerenity {
///     // Bot setup and startup code
/// }
/// ```
///
/// The bot will start and listen to commands like `lolstats` once it is running.
#[shuttle_runtime::main]
async fn main(
    #[shuttle_runtime::Secrets] secret_store: SecretStore
) -> ShuttleSerenity {
    // Récupérer le token Discord, la clé Riot API et l'URI MongoDB depuis les secrets
    let discord_token = secret_store
        .get("DISCORD_TOKEN")
        .ok_or_else(|| anyhow::anyhow!("'DISCORD_TOKEN' was not found"))?;

    let riot_api_key = secret_store
        .get("RIOT_API_KEY")
        .ok_or_else(|| anyhow::anyhow!("'RIOT_API_KEY' was not found"))?;

    let mongodb_uri = secret_store
        .get("MONGODB_URI")
        .ok_or_else(|| anyhow::anyhow!("'MONGODB_URI' was not found"))?;

    // Initialiser MongoDB
    let mut client_options = ClientOptions::parse(&mongodb_uri).await.expect("Failed to parse MongoDB URI");
    let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
    client_options.server_api = Some(server_api);
    let mongo_client = Client::with_options(client_options).expect("Failed to create MongoDB client");

    // Lancer une tâche de fond pour vérifier la base de données toutes les 2 minutes
    let mongo_client_clone = mongo_client.clone();
    let riot_api_key_clone = riot_api_key.clone();
    tokio::spawn(async move {
        loop {
            // Exécuter la vérification périodique de la base de données
            match check_and_update_db(&mongo_client_clone, &riot_api_key_clone).await {
                Ok(_) => (),
                Err(e) => error!("Erreur lors de la vérification de la base de données : {:?}", e),
            }
            sleep(Duration::from_secs(120)).await; // Attendre 2 minutes
        }
    });
    let dd_json = riot_api::open_dd_json().await.unwrap();
    // Configurer le framework Poise avec les commandes
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![lolstats(), followgames()],
            ..Default::default()
        })
        .setup(move |_ctx, _ready, _framework| {
            let riot_api_key = riot_api_key.clone();
            let mongo_client = mongo_client.clone();
            let dd_json = dd_json.clone();
            Box::pin(async move {
                poise::builtins::register_globally(_ctx, &_framework.options().commands).await?;
                Ok(Data {
                    riot_api_key,
                    mongo_client,
                    dd_json,
                })
            })
        })
        .build();

    // Créer le client Discord avec le token et les intents nécessaires
    let client = serenity::ClientBuilder::new(discord_token, serenity::GatewayIntents::non_privileged())
        .framework(framework)
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    Ok(client.into())
}

/// Fonction de vérification et de mise à jour de la base de données
async fn check_and_update_db(mongo_client: &Client, riot_api_key: &str) -> Result<(), mongodb::error::Error> {
    let collection = mongo_client
        .database("stat-summoner")
        .collection::<models::SummonerFollowedData>("follower_summoner");

    // Compter le nombre de documents dans la collection
    let count = collection.estimated_document_count().await?;

    if count > 0 {
        println!("La base de données contient {} documents.", count);

        // Récupérer tous les documents dans la collection
        let mut cursor = collection.find(doc! {}).await?;

        // Itérer sur chaque document et afficher `puuid` et `last_match`
        while let Some(result) = cursor.next().await {
            match result {
                Ok(followed_summoner) => {
                    let puuid = followed_summoner.puuid;
                    let summoner_id = followed_summoner.summoner_id;
                    let last_match_id = followed_summoner.last_match_id;
                    let client = reqwest::Client::new();
                    // Use riot_api_key passed as a parameter
                    let match_id_from_riot = get_matchs_id(&client, &puuid, riot_api_key, 1).await.unwrap()[0].to_string();
                    if last_match_id == match_id_from_riot {
                        eprint!("pas de nouvelle partie");
                    }
                    else {
                        eprint!("Nouvelle partie pour : {}", puuid);
                    }
                    
                }
                Err(e) => {
                    println!("Erreur lors de la récupération d'un document : {:?}", e);
                }
            }
        }
    }

    Ok(())
}
