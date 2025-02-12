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
    if let Some(data) = custom_id.strip_prefix("flex_user:") {
        if let Ok(role) = Role::from_str(data) {
            if let Some(embed) = embed {
                for field in &embed.fields {
                    if field.name == role.field_name() && field.value == "TBD" {
                        let mut new_embed = embed.clone();
                        for field in &mut new_embed.fields {
                            if field.name == role.field_name() {
                                field.value =
                                    format!("<@{}>", message_component_interaction.user.id);
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
