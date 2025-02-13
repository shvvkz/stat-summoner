/// 🛠 **Module askingforflex**: Contains commands and utilities related to interaction with flex buttons.
///
/// This module provides functionality for handling interactions with flex buttons in a Discord bot context.
/// It includes the `askingforflex` command, which allows users to interact with buttons and receive appropriate responses,
/// as well as utility functions to support these interactions.
///
/// # Files in this module:
/// - `askingforflex.rs`: The command for handling flex button interactions.
/// - `interaction_flex_buttons.rs`: Functions and handlers for different flex button interactions.
/// - `utils.rs`: Utility functions used by the `askingforflex` command and button interactions.
///
/// # Example:
/// To use the commands in this module, ensure they are registered in the bot's main framework setup:
///
/// ```rust
/// use module::askingforflex::askingforflex;
///
/// #[shuttle_runtime::main]
/// async fn main() {
///     let framework = poise::Framework::builder()
///         .options(poise::FrameworkOptions {
///             commands: vec![askingforflex()], // Register the askingforflex command
///             ..Default::default()
///         })
///         .build();
/// }
/// ```
///
/// The `askingforflex` command allows users to interact with flex buttons and receive responses based on their interactions.
/// The module includes handlers for different button interactions and utility functions to support these operations.
/// The command and interactions are designed to enhance user engagement and provide a dynamic experience.
///
/// As more commands or utilities related to flex button interactions are added, they will be included here and imported into the main bot setup.
pub mod askingforflex;
pub mod interaction_flex_buttons;
pub mod utils;
