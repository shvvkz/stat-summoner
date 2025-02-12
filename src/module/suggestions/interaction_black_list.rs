use poise::serenity_prelude::ComponentInteraction;

use crate::models::{
    data::{Data, User},
    error::Error,
};

use super::utils::add_user_to_blacklist;

/// Handles interactions with buttons in Discord messages.
///
/// This function processes button clicks in Discord, specifically those with a `custom_id`
/// starting with `blacklist_user:`. It extracts the user ID and username from the `custom_id`,
/// adds the user to a MongoDB blacklist collection, and deletes the original message containing
/// the button.
///
/// # Parameters:
/// - `ctx`: The context of the interaction, providing access to the bot and its utilities.
/// - `interaction`: The interaction object representing the button click.
/// - `ctx_data`: The application's shared data, containing the MongoDB client and other configuration.
///
/// # Returns:
/// - `Result<(), Error>`: Returns `Ok(())` if the interaction is handled successfully; otherwise, returns an `Error`.
///
/// # ⚠️ Notes:
/// - The function checks if the `custom_id` starts with `blacklist_user:` to identify blacklist actions.
/// - The `custom_id` must be formatted as `blacklist_user:{user_id}|{username}` to extract the required data.
/// - After adding the user to the blacklist, the message containing the button is deleted.
///
/// # Example:
/// ```rust
/// handle_button_click(ctx, interaction, ctx_data).await?;
/// ```
///
/// When a moderator clicks the "Blacklist User" button on a suggestion, the following actions occur:
/// 1. The user who submitted the suggestion is added to the blacklist in MongoDB.
/// 2. The original suggestion message is deleted.
/// 3. Errors during these actions are logged but do not stop the execution.
///
/// # Errors:
/// - If the user cannot be added to the blacklist, an error is logged.
/// - If the original message cannot be deleted, an error is logged.
/// - If the `custom_id` does not have the correct format, the function exits without doing anything.
///
/// # See Also:
/// - `add_user_to_blacklist`: Adds a user to the MongoDB blacklist collection.
/// - `BlackList`: Represents the structure of a blacklisted user in the database.
///
/// # Related Structures:
/// - `Interaction`: Represents the interaction object from Discord.
/// - `BlackList`: Contains information about blacklisted users.
///
/// # Dependencies:
/// - Relies on MongoDB for storing blacklisted users.
/// - Uses `custom_id` from Discord buttons to identify actions and extract relevant data.
pub async fn handle_interaction_button_black_list(
    ctx: poise::serenity_prelude::Context,
    message_component_interaction: ComponentInteraction,
    ctx_data: &Data,
) -> Result<(), Error> {
    let custom_id = &message_component_interaction.data.custom_id;
    if let Some(data) = custom_id.strip_prefix("blacklist_user:") {
        let user_id = data;
        let mongo_client = ctx_data.mongo_client.clone();
        let collection = mongo_client
            .database("stat-summoner")
            .collection::<User>("users");

        if let Err(e) = add_user_to_blacklist(&collection, user_id).await {
            log::error!("Erreur lors de l'ajout à la blacklist : {:?}", e);
        }

        if let Err(e) = ctx
            .http
            .delete_message(
                message_component_interaction.channel_id,
                message_component_interaction.message.id,
                None,
            )
            .await
        {
            log::error!("Erreur lors de la suppression du message : {:?}", e);
        }
    }
    Ok(())
}
