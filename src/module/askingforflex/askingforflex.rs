use crate::models::data::{Data, EmojiId};
use crate::models::error::Error;
use crate::models::modal::FlexAskingModal;
use crate::module::askingforflex::utils::{
    compute_role_mention, create_flex_embed, parse_flex_modal,
};
use crate::utils::manage_user;
use poise::serenity_prelude::{CreateMessage, Mentionable};
use poise::Modal;

/// Handles the flex asking command.
///
/// This command updates or creates the user in the database, retrieves the emoji collection,
/// fetches the guild roles, executes the modal, validates the data, generates the role mention,
/// composes the flex request message, creates the embed, and sends the message with buttons for
/// other users to join.
///
/// # Arguments
///
/// * `ctx` - The application context containing data and error types.
///
/// # Returns
///
/// * `Result<(), Error>` - Returns an empty result on success or an error on failure.
#[poise::command(slash_command)]
pub async fn askingforflex(ctx: poise::ApplicationContext<'_, Data, Error>) -> Result<(), Error> {
    manage_user(
        ctx.author().id.to_string(),
        ctx.author().name.clone(),
        &ctx.data().mongo_client,
        false,
    )
    .await?;

    let collection_emojis = ctx
        .data()
        .mongo_client
        .database("stat-summoner")
        .collection::<EmojiId>("emojis_id");

    let guild_roles = ctx.guild().map(|g| g.roles.clone());

    let raw_modal = FlexAskingModal::execute(ctx).await?;
    let modal_data = match parse_flex_modal(ctx, raw_modal).await {
        Some(data) => data,
        None => return Ok(()),
    };

    let role_mention = compute_role_mention(guild_roles, modal_data.people_to_ask_by_role.trim());

    let flex_message = format!(
        "Hey {} please wake up! {} is looking for a flex game at **{}**!",
        role_mention,
        ctx.author().mention(),
        modal_data.starting_hour
    );

    let embed = create_flex_embed(&ctx.author().name, &modal_data, collection_emojis).await?;

    let button_top = poise::serenity_prelude::CreateButton::new("Top")
        .custom_id("flex_user:top")
        .label("Top")
        .style(poise::serenity_prelude::ButtonStyle::Primary);

    let button_jungle = poise::serenity_prelude::CreateButton::new("Jungle")
        .custom_id("flex_user:jungle")
        .label("Jungle")
        .style(poise::serenity_prelude::ButtonStyle::Primary);

    let button_mid = poise::serenity_prelude::CreateButton::new("Mid")
        .custom_id("flex_user:mid")
        .label("Mid")
        .style(poise::serenity_prelude::ButtonStyle::Primary);

    let button_adc = poise::serenity_prelude::CreateButton::new("ADC")
        .custom_id("flex_user:adc")
        .label("ADCarry")
        .style(poise::serenity_prelude::ButtonStyle::Primary);

    let button_support = poise::serenity_prelude::CreateButton::new("Support")
        .custom_id("flex_user:support")
        .label("Support")
        .style(poise::serenity_prelude::ButtonStyle::Primary);

    let builder = CreateMessage::new()
        .content(flex_message)
        .embed(embed)
        .button(button_top)
        .button(button_jungle)
        .button(button_mid)
        .button(button_adc)
        .button(button_support);
    ctx.channel_id().send_message(&ctx.http(), builder).await?;

    Ok(())
}
