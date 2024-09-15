mod riot_api;
mod models;
mod embed;
mod utils;
mod commands;

use models::Data;
use poise::serenity_prelude::{self as serenity};
use shuttle_runtime::SecretStore;
use shuttle_serenity::ShuttleSerenity;
use commands::lolstats::lolstats;
use commands::followgames::followgames;


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
        let discord_token = secret_store
            .get("DISCORD_TOKEN")
            .ok_or_else(|| anyhow::anyhow!("'DISCORD_TOKEN' was not found"))?;

        let riot_api_key = secret_store
            .get("RIOT_API_KEY")
            .ok_or_else(|| anyhow::anyhow!("'RIOT_API_KEY' was not found"))?;
        let mongodb_uri = secret_store
            .get("MONGODB_URI")
            .ok_or_else(|| anyhow::anyhow!("'MONGODB_URI' was not found"))?;
        
        let framework = poise::Framework::builder()
            .options(poise::FrameworkOptions {
                commands: vec![lolstats(), followgames()],
                ..Default::default()
            })
            .setup(move |_ctx, _ready, _framework| {
                Box::pin(async move {
                    poise::builtins::register_globally(_ctx, &_framework.options().commands).await?;
                    Ok(Data {
                        riot_api_key,
                        mongodb_uri,
                    })
                })
            })
            .build();

        let client = serenity::ClientBuilder::new(discord_token, serenity::GatewayIntents::non_privileged())
            .framework(framework)
            .await
            .map_err(shuttle_runtime::CustomError::new)?;

        Ok(client.into())
    }
