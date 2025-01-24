pub mod interaction_black_list;
/// 🛠 **Module suggestions**: Contains the logic and utilities for handling suggestions and interactions related to blacklisting.
///
/// This module organizes the functionality for the `suggestion` command and the interaction handling for
/// blacklisting users. Each component is separated into its respective file for clarity and modularity.
///
/// # Files in this module:
/// - `suggestions.rs`: Handles the `suggestion` command, which allows users to submit suggestions and includes the logic for embedding those suggestions in Discord.
/// - `interaction_black_list.rs`: Handles interactions with the "Blacklist User" button, including adding users to the blacklist and deleting the associated suggestion messages.
/// - `utils.rs`: Contains utility functions shared across the `suggestions` module, such as checking if a user is blacklisted and adding users to the blacklist.
///
/// # Example:
/// To use the commands and interaction handlers in this module, ensure they are registered in the bot's main framework setup:
///
/// ```rust
/// use module::suggestions::suggestions::suggestion;
/// use module::suggestions::interaction_black_list::handle_button_click;
///
/// #[shuttle_runtime::main]
/// async fn main() {
///     let framework = poise::Framework::builder()
///         .options(poise::FrameworkOptions {
///             commands: vec![suggestion()], // Register the suggestion command
///             event_handler: |ctx, event, _framework, data| {
///                 Box::pin(async move {
///                     if let Some(interaction) = event.interaction_create() {
///                         handle_button_click(ctx.clone(), interaction.clone(), data).await?;
///                     }
///                     Ok(())
///                 })
///             },
///             ..Default::default()
///         })
///         .build();
/// }
/// ```
///
/// As more features are added to this module, they will be included here and imported into the main bot setup.
pub mod suggestions;
pub mod utils;
