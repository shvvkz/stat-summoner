/// 🛠 **Module commands**: Contains all bot commands for the Discord bot.
///
/// This module organizes the different commands used by the bot. Each command is stored in its own file
/// within the `commands` directory. These commands are registered and used through the bot's interaction
/// with Discord via the Poise framework.
///
/// # Files in this module:
/// - `randomchampions.rs`: The command for selecting a random League of Legends champion and displaying its information.
///
/// # Example:
/// To use the command in this module, ensure it is registered in the bot's main framework setup:
///
/// ```rust
/// use commands::randomchampions::randomchampions;
///
/// #[shuttle_runtime::main]
/// async fn main() {
///     let framework = poise::Framework::builder()
///         .options(poise::FrameworkOptions {
///             commands: vec![randomchampions()], // Register the randomchampions command
///             ..Default::default()
///         })
///         .build();
/// }
/// ```
///
/// As more commands are added, they will be included here and imported into the main bot setup.
pub mod randomchampions;
pub mod utils;
