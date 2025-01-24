use chrono::Utc;
use poise::serenity_prelude::{CreateEmbed, CreateMessage};
use poise::Modal;

use crate::embed::create_embed_success;
use crate::embed::{create_embed_error, schedule_message_deletion};
use crate::models::data::{Data, User};
use crate::models::error::Error;
use crate::models::modal::SuggestionsModal;
use crate::module::suggestions::utils::{can_user_make_suggestion, is_user_blacklisted};
use crate::utils::manage_user;

/// Handles suggestions submitted by users.
///
/// This command allows users to submit suggestions, which are posted as an embedded message in a specific Discord channel.
/// If the user is blacklisted, their submission is rejected. Suggestions are submitted via a modal and include a "Blacklist User"
/// button, which can be used by moderators to blacklist the user who made the suggestion.
///
/// # Parameters:
/// - `ctx`: The command's context, providing access to the bot, the message, and other utilities.
///
/// # Returns:
/// - `Result<(), Error>`: Returns `Ok(())` if the command executes successfully, otherwise returns an `Error`.
///
/// # ⚠️ Notes:
/// - This function checks if the user is blacklisted using `is_user_blacklisted`.
/// - Suggestions are displayed in the designated channel (`suggestions_channel_id`) as a Discord embed.
/// - A "Blacklist User" button is added to each suggestion, with a `custom_id` containing the user's ID and username.
/// - After submitting a suggestion, a confirmation message is sent to the user, which is scheduled for deletion after 10 seconds.
///
/// # Example:
/// ```rust
/// suggestion(ctx).await?;
/// ```
///
/// This command produces an embed displaying the suggestion:
/// ```text
/// 📝 New suggestion
/// User: JohnDoe
/// User ID: 123456789012345678
///
/// Suggestion content: "Add a new game mode"
/// ```
///
/// # Errors:
/// - If the user is blacklisted, a rejection message is sent to the user and the command exits early.
/// - If the modal submission fails, the command sends an error message to the user.
/// - If there is an issue sending the suggestion embed, an error is logged, and the user receives an error message.
///
/// # See Also:
/// - `is_user_blacklisted`: Checks if a user is blacklisted from making suggestions.
/// - `schedule_message_deletion`: Schedules a message for deletion after a specific time interval.
/// - `SuggestionsModal`: Handles the modal used for collecting suggestion details.
///
/// # Related Structures:
/// - `BlackList`: Represents a blacklisted user.
/// - `SuggestionsModal`: Used to capture the suggestion content from the user.
///
/// # Dependencies:
/// - This function relies on a MongoDB collection for storing blacklisted users.
/// - The embed includes a "Blacklist User" button to handle moderation actions.
#[poise::command(slash_command)]
pub async fn suggestion(ctx: poise::ApplicationContext<'_, Data, Error>) -> Result<(), Error> {
    let user_id = ctx.author().id.to_string();
    let username = ctx.author().name.clone();

    let mongo_client = &ctx.data().mongo_client;
    let collection = mongo_client
        .database("stat-summoner")
        .collection::<User>("users");

    if is_user_blacklisted(&collection, &user_id).await? {
        let error_message = "You are blacklisted from making suggestions.";
        let reply = ctx.send(create_embed_error(&error_message)).await?;
        schedule_message_deletion(reply, ctx).await?;
        return Ok(());
    }

    if !can_user_make_suggestion(&collection, &user_id).await? {
        let error_message =
            "You cannot make a new suggestion yet. Please wait a while before trying again.";
        let reply = ctx.send(create_embed_error(&error_message)).await?;
        schedule_message_deletion(reply, ctx).await?;
        return Ok(());
    }

    manage_user(
        ctx.author().id.to_string(),
        ctx.author().name.clone(),
        &ctx.data().mongo_client,
        true,
    )
    .await?;

    let modal_data = match SuggestionsModal::execute(ctx).await {
        Ok(Some(data)) => data,
        Ok(None) => {
            let error_message = "Modal data not found.";
            let reply = ctx.send(create_embed_error(&error_message)).await?;
            schedule_message_deletion(reply, ctx).await?;
            return Ok(());
        }
        Err(_) => {
            let error_message = "Failed to retrieve modal data.";
            let reply = ctx.send(create_embed_error(&error_message)).await?;
            schedule_message_deletion(reply, ctx).await?;
            return Ok(());
        }
    };

    let suggestion = modal_data.suggestion.clone();
    let channel_id = poise::serenity_prelude::ChannelId::from(ctx.data().suggestions_channel_id);
    let embed = CreateEmbed::default()
        .title("New suggestion")
        .description(suggestion)
        .field("User", username.clone(), false)
        .field("User ID", user_id.clone(), false)
        .timestamp(Utc::now())
        .color(0x00FF00);

    let custom_id = format!("blacklist_user:{}", user_id);
    let button = poise::serenity_prelude::CreateButton::new("Blacklist User")
        .custom_id(custom_id)
        .label("Blacklist User")
        .style(poise::serenity_prelude::ButtonStyle::Danger);

    let action_row = poise::serenity_prelude::CreateActionRow::Buttons(vec![button]);
    let builder = CreateMessage::new()
        .embed(embed)
        .components(vec![action_row]);
    channel_id.send_message(&ctx.http(), builder).await?;

    let success_message =
        "Suggestion Received.\n Your suggestion has been received and will be reviewed. Thank you!";
    let reply = ctx.send(create_embed_success(&success_message)).await?;
    schedule_message_deletion(reply, ctx).await?;
    Ok(())
}
