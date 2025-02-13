use crate::models::data::Data;
use crate::models::error::Error;
use crate::module::askingforflex::interaction_flex_buttons::handle_interaction_button_flex;
/// Handles the interaction button black list.
///
/// This function processes interactions related to the black list of buttons.
/// It is used to manage and handle the logic when a button that is blacklisted
/// is interacted with.
///
/// # Arguments
///
/// * `interaction` - The interaction object that contains details about the interaction event.
///
/// # Returns
///
/// This function does not return a value.
///
/// # Examples
///
/// ```rust
/// use crate::module::suggestions::interaction_black_list::handle_interaction_button_black_list;
///
/// // Example usage
/// handle_interaction_button_black_list(interaction);
/// ```
///
/// # Errors
///
/// This function does not return errors directly, but it may cause side effects
/// or trigger other error-handling mechanisms within the application.
use crate::module::suggestions::interaction_black_list::handle_interaction_button_black_list;
use poise::serenity_prelude::Interaction;

pub async fn handle_button_click(
    ctx: poise::serenity_prelude::Context,
    interaction: Interaction,
    ctx_data: &Data,
) -> Result<(), Error> {
    if let Some(message_component_interaction) = interaction.message_component() {
        let http = ctx.http.clone();
        message_component_interaction.defer(http).await?;
        let custom_id = &message_component_interaction.data.custom_id;

        if custom_id.starts_with("blacklist_user:") {
            handle_interaction_button_black_list(ctx, message_component_interaction, ctx_data)
                .await?;
        } else if custom_id.starts_with("flex_user:") {
            handle_interaction_button_flex(ctx, message_component_interaction, ctx_data).await?;
        }
    }
    Ok(())
}
