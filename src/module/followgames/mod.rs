/// 🛠 **Module commands**: Contains all bot commands for the Discord bot.
/// 
/// This module organizes the different commands used by the bot. Each command is stored in its own file
/// within the `commands` directory. These commands are registered and used through the bot's interaction
/// with Discord via the Poise framework.
/// 
/// # Files in this module:
/// - `followgames.rs`: The command for following a player's games and tracking their match data for a specified period.
/// 
/// # Example:
/// To use commands in this module, ensure they are registered in the bot's main framework setup:
/// 
/// ```rust
/// use commands::followgames::followgames;
/// 
/// #[shuttle_runtime::main]
/// async fn main() {
///     let framework = poise::Framework::builder()
///         .options(poise::FrameworkOptions {
///             commands: vec![followgames()], // Register the followgames command
///             ..Default::default()
///         })
///         .build();
/// }
/// ```
/// The `followgames` command allows users to track the games of a summoner in real time for a period between 1 and 48 hours.
/// 
/// As more commands are added, they will be included here and imported into the main bot setup.
pub mod followgames;
pub mod utils;
