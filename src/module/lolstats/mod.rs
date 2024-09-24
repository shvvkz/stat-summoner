/// 🛠 **Module commands**: Contains all bot commands for the Discord bot.
///
/// This module organizes the different commands used by the bot. Each command is stored in its own file
/// within the `commands` directory. These commands are registered and used through the bot's interaction
/// with Discord via the Poise framework.
///
/// # Files in this module:
/// - `lolstats.rs`: The command for fetching and displaying League of Legends player stats.
///
/// # Example:
/// To use commands in this module, ensure they are registered in the bot's main framework setup:
///
/// ```rust
/// use commands::lolstats::lolstats;
///
/// #[shuttle_runtime::main]
/// async fn main() {
///     let framework = poise::Framework::builder()
///         .options(poise::FrameworkOptions {
///             commands: vec![lolstats()], // Register the lolstats command
///             ..Default::default()
///         })
///         .build();
/// }
/// ```
/// new command `followgames` will be added to the bot's command list soon.
///
/// As more commands are added, they will be included here and imported into the main bot setup.
pub mod lolstats;
pub mod utils;
