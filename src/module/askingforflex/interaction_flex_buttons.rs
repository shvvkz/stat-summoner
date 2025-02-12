use crate::models::error::Error;
use poise::serenity_prelude::CacheHttp;
use poise::serenity_prelude::{ComponentInteraction, CreateEmbed};
use std::str::FromStr;

pub async fn handle_interaction_button_flex(
    ctx: poise::serenity_prelude::Context,
    message_component_interaction: ComponentInteraction,
) -> Result<(), Error> {
    let custom_id = &message_component_interaction.data.custom_id;
    let channel_id = message_component_interaction.message.channel_id;
    let message_id = message_component_interaction.message.id;
    let embed = message_component_interaction.message.embeds.get(0);
    let user_mention = format!("<@{}>", message_component_interaction.user.id);

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
                    if field.name == role.field_name() {
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
    fn field_name(&self) -> &str {
        match self {
            Role::Top => "Top:",
            Role::Jungle => "Jungle:",
            Role::Mid => "Mid:",
            Role::ADC => "ADCarry:",
            Role::Support => "Support:",
        }
    }
}
