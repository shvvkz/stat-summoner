use crate::models::data::EmojiId;
use crate::models::error::Error;
use crate::utils::get_emoji;
use mongodb::Collection;
use poise::serenity_prelude::CacheHttp;
use poise::serenity_prelude::{ComponentInteraction, CreateEmbed};
use std::str::FromStr;

/// Handles flex interactions with buttons in Discord messages.
///
/// This function processes button clicks in Discord related to the flex feature, specifically those with a custom_id
/// starting with `flex_user:`. It identifies the user and the selected role, updates the embed message to reflect the user's choice,
/// and manages role assignments based on the presence of existing users.
///
/// # Parameters:
/// - ctx: The context of the interaction, providing access to the bot and its utilities.
/// - message_component_interaction: The interaction object representing the button click event.
/// - ctx_data: The application's shared data, containing the MongoDB client and other configuration details.
///
/// # Returns:
/// - Result<(), Error>: Returns `Ok(())` if the interaction is processed successfully; otherwise, returns an `Error`.
///
/// # ⚠️ Notes:
/// - The function checks if the `custom_id` starts with `flex_user:` to identify flex actions.
/// - The `custom_id` must contain the role as a string after the prefix to parse the selected role.
/// - Updates the Discord embed fields based on user selections and ensures only one user is assigned per role.
/// - Handles setting and resetting user mentions in the embed fields dynamically.
///
/// # Example:
///
/// ```rust
/// handle_interaction_button_flex(ctx, message_component_interaction, ctx_data).await?;
/// ```
///
/// When a user clicks a "Flex" button for a specific role, the following actions occur:
/// 1. The selected role is parsed from the `custom_id`.
/// 2. The corresponding embed field is updated with the user's mention if it was previously empty.
/// 3. If the user was already set in another role, that field is reset to `TBD`.
/// 4. If the user is already in the selected role, the field is reset to `TBD`.
/// 5. The modified embed is then updated in the original Discord message.
///
/// # Errors:
/// - If the `custom_id` is not formatted correctly or the role is unrecognized, the function exits silently.
/// - If any database operation with MongoDB fails, an error is returned.
/// - If the message cannot be edited on Discord, an error is returned.
///
/// # See Also:
/// - `get_emoji`: Retrieves emojis from the MongoDB collection for use in embed fields.
/// - `Role`: Enum representing the different roles available in the flex feature.
///
/// # Related Structures:
/// - `EmojiId`: Represents the structure for storing emoji identifiers in the database.
/// - `Role`: Enum defining Top, Jungle, Mid, ADC, and Support roles.
///
/// # Dependencies:
/// - Relies on MongoDB for storing and retrieving emoji data.
/// - Uses `poise::serenity_prelude` for interacting with Discord messages and embeds.
/// - Parses roles from button `custom_id` strings using `FromStr` trait.
///
/// ```
pub async fn handle_interaction_button_flex(
    ctx: poise::serenity_prelude::Context,
    message_component_interaction: ComponentInteraction,
    ctx_data: &crate::models::data::Data,
) -> Result<(), Error> {
    let custom_id = &message_component_interaction.data.custom_id;
    let channel_id = message_component_interaction.message.channel_id;
    let message_id = message_component_interaction.message.id;
    let embed = message_component_interaction.message.embeds.get(0);
    let user_mention = format!("<@{}>", message_component_interaction.user.id);
    let mongo_client = ctx_data.mongo_client.clone();
    let collection = mongo_client
        .database("stat-summoner")
        .collection::<EmojiId>("emojis_id");

    if let Some(data) = custom_id.strip_prefix("flex_user:") {
        if let Ok(role) = Role::from_str(data) {
            if let Some(embed) = embed {
                let mut new_embed = embed.clone();
                let mut user_already_set = false;

                // Check if the user is already set in any field
                for field in &new_embed.fields {
                    if field.value == user_mention {
                        user_already_set = true;
                        break;
                    }
                }

                for field in &mut new_embed.fields {
                    if field.name == role.field_name(&collection).await? {
                        if field.value == user_mention {
                            // If the field value is already the user's mention, set it back to "TBD"
                            field.value = "TBD".to_string();
                        } else if field.value == "TBD" {
                            // If the field value is "TBD", set it to the user's mention
                            field.value = user_mention.clone();
                        }
                    } else if user_already_set && field.value == user_mention {
                        // If the user is already set in another field, set that field to "TBD"
                        field.value = "TBD".to_string();
                    }
                }

                let c_embed = CreateEmbed::from(new_embed);
                channel_id
                    .edit_message(
                        &ctx.http(),
                        message_id,
                        poise::serenity_prelude::EditMessage::default().embed(c_embed),
                    )
                    .await?;
                return Ok(());
            }
        }
    }
    Ok(())
}

enum Role {
    Top,
    Jungle,
    Mid,
    ADC,
    Support,
}

impl FromStr for Role {
    type Err = ();

    fn from_str(input: &str) -> Result<Role, Self::Err> {
        match input {
            "top" => Ok(Role::Top),
            "jungle" => Ok(Role::Jungle),
            "mid" => Ok(Role::Mid),
            "adc" => Ok(Role::ADC),
            "support" => Ok(Role::Support),
            _ => Err(()),
        }
    }
}

impl Role {
    async fn field_name(&self, collection_emojis: &Collection<EmojiId>) -> Result<String, Error> {
        match self {
            Role::Top => {
                let emoji = get_emoji(collection_emojis.clone(), "position", "TOP").await?;
                Ok(format!("Top: {}", emoji))
            }
            Role::Jungle => {
                let emoji = get_emoji(collection_emojis.clone(), "position", "JUNGLE").await?;
                Ok(format!("Jungle: {}", emoji))
            }
            Role::Mid => {
                let emoji = get_emoji(collection_emojis.clone(), "position", "MIDDLE").await?;
                Ok(format!("Mid: {}", emoji))
            }
            Role::ADC => {
                let emoji = get_emoji(collection_emojis.clone(), "position", "BOTTOM").await?;
                Ok(format!("ADCarry: {}", emoji))
            }
            Role::Support => {
                let emoji = get_emoji(collection_emojis.clone(), "position", "SUPPORT").await?;
                Ok(format!("Support: {}", emoji))
            }
        }
    }
}
