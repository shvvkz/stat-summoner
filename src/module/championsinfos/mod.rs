/// 🛠 **Module championsinfos**: Contains commands and utilities related to League of Legends champion information.
///
/// This module provides functionality for fetching and displaying detailed information about League of Legends champions.
/// It includes the `championsinfos` command, which allows users to input a champion's name and receive comprehensive data,
/// such as roles, winrate, banrate, popularity, recommended runes, and core item builds, all presented in a formatted Discord embed with appropriate emojis.
///
/// # Files in this module:
/// - `championsinfos.rs`: The command for fetching and displaying champion information.
/// - `utils.rs`: Utility functions used by the `championsinfos` command, such as functions for data retrieval, fuzzy matching, and processing.
///
/// # Example:
/// To use the commands in this module, ensure they are registered in the bot's main framework setup:
///
/// ```rust
/// use commands::championsinfos::championsinfos;
///
/// #[shuttle_runtime::main]
/// async fn main() {
///     let framework = poise::Framework::builder()
///         .options(poise::FrameworkOptions {
///             commands: vec![championsinfos()], // Register the championsinfos command
///             ..Default::default()
///         })
///         .build();
/// }
/// ```
///
/// The `championsinfos` command allows users to fetch detailed information about a specific champion by entering the champion's name.
/// It utilizes fuzzy matching to handle variations or misspellings in the champion's name input by the user.
/// The command presents the information in a well-structured Discord embed, enhancing user experience.
///
/// As more commands or utilities related to champion information are added, they will be included here and imported into the main bot setup.
pub mod championsinfos;
pub mod utils;
