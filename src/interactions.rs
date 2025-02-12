use crate::models::data::Data;
use crate::models::error::Error;
use crate::module::askingforflex::interaction_flex_buttons::handle_interaction_button_flex;
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
            handle_interaction_button_flex(ctx, message_component_interaction).await?;
        }
    }
    Ok(())
}
