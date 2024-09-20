/// 🛠 **Module commands**: Contains all bot commands for the Discord bot.
///
/// This module organizes the different commands used by the bot. Each command is stored in its own file
/// within the `commands` directory. These commands are registered and used through the bot's interaction
/// with Discord via the Poise framework.
///
/// # Files in this module:
/// - `whoisfollowed.rs`: The command for fetching and displaying the list of summoners being followed in the current Discord guild.
///
/// # Example:
/// To use commands in this module, ensure they are registered in the bot's main framework setup:
///
/// ```rust
/// use commands::whoisfollowed::whoisfollowed;
///
/// #[shuttle_runtime::main]
/// async fn main() {
///     let framework = poise::Framework::builder()
///         .options(poise::FrameworkOptions {
///             commands: vec![whoisfollowed()], // Register the whoisfollowed command
///             ..Default::default()
///         })
///         .build();
/// }
/// ```
/// A new command `whoisfollowed` allows users to retrieve a list of summoners currently being followed in the guild, along with the remaining follow time.
///
/// As more commands are added, they will be included here and imported into the main bot setup.
pub mod whoisfollowed;
pub mod utils;
