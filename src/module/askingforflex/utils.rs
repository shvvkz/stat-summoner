use crate::embed::{create_embed_error, schedule_message_deletion};
use crate::models::data::{Data, EmojiId};
use crate::models::error::Error;
use crate::models::modal::FlexAskingModal;
use mongodb::Collection;
use poise::serenity_prelude::{CreateEmbed, Mentionable, Role, RoleId};
use std::collections::HashMap;

/// Valide le format de l'heure saisie dans le modal.
/// Si le format est incorrect, envoie un message d'erreur et programme sa suppression.
/// Retourne `Some(FlexAskingModal)` avec l'heure validée ou `None` en cas d'erreur.
pub async fn parse_flex_modal(
    ctx: poise::ApplicationContext<'_, Data, Error>,
    modal: Option<FlexAskingModal>,
) -> Option<FlexAskingModal> {
    if let Some(data) = modal {
        let time = data.starting_hour.trim();
        if time.len() != 5 || time.chars().nth(2) != Some(':') {
            let error_message = "Invalid time format. Please use HH:MM format.";
            if let Ok(reply) = ctx.send(create_embed_error(&error_message)).await {
                let _ = schedule_message_deletion(reply, ctx).await;
            }
            return None;
        }
        let hours: Result<u8, _> = time[..2].parse();
        let minutes: Result<u8, _> = time[3..].parse();
        if let (Ok(h), Ok(m)) = (hours, minutes) {
            if h >= 24 || m >= 60 {
                let error_message = "Invalid time format. Hours must be between 00 and 23, and minutes between 00 and 59.";
                if let Ok(reply) = ctx.send(create_embed_error(&error_message)).await {
                    let _ = schedule_message_deletion(reply, ctx).await;
                }
                return None;
            }
        } else {
            let error_message = "Invalid time format. Please use HH:MM format.";
            if let Ok(reply) = ctx.send(create_embed_error(&error_message)).await {
                let _ = schedule_message_deletion(reply, ctx).await;
            }
            return None;
        }
        Some(FlexAskingModal {
            starting_hour: time.to_string(),
            people_to_ask_by_role: data.people_to_ask_by_role.replace("@", ""),
        })
    } else {
        let error_message = "Modal data not found.";
        if let Ok(reply) = ctx.send(create_embed_error(&error_message)).await {
            let _ = schedule_message_deletion(reply, ctx).await;
        }
        None
    }
}

/// Génère la mention du rôle à partir du nom donné et de la liste des rôles du serveur.
pub fn compute_role_mention(guild_roles: Option<HashMap<RoleId, Role>>, role_name: &str) -> String {
    if let Some(roles) = guild_roles {
        roles
            .values()
            .find(|role| role.name.to_lowercase() == role_name.to_lowercase())
            .map(|role| role.id.mention().to_string())
            .unwrap_or_else(|| format!("@{}", role_name))
    } else {
        format!("@{}", role_name)
    }
}

/// Construit l'embed pour la demande de flex game en récupérant les émojis via la collection MongoDB.
pub async fn create_flex_embed(
    author: &str,
    modal_data: &FlexAskingModal,
    collection_emojis: Collection<EmojiId>,
) -> Result<CreateEmbed, Error> {
    // On suppose que la fonction `get_emoji` est déjà définie dans utils.
    let emoji_top: String =
        crate::utils::get_emoji(collection_emojis.clone(), "position", "TOP").await?;
    let emoji_jungle: String =
        crate::utils::get_emoji(collection_emojis.clone(), "position", "JUNGLE").await?;
    let emoji_mid: String =
        crate::utils::get_emoji(collection_emojis.clone(), "position", "MIDDLE").await?;
    let emoji_adc: String =
        crate::utils::get_emoji(collection_emojis.clone(), "position", "BOTTOM").await?;
    let emoji_support: String =
        crate::utils::get_emoji(collection_emojis.clone(), "position", "SUPPORT").await?;

    Ok(CreateEmbed::default()
        .color(0x00ff00)
        .title("Flex Game Request")
        .description("A player is looking for a flex team!")
        .field("Requested by", author, true)
        .field("Start Time", &modal_data.starting_hour, true)
        .field(format!("{}:", emoji_top), "TBD", false)
        .field(format!("{}:", emoji_jungle), "TBD", false)
        .field(format!("{}:", emoji_mid), "TBD", false)
        .field(format!("{}:", emoji_adc), "TBD", false)
        .field(format!("{}:", emoji_support), "TBD", false)
        .thumbnail("https://i.postimg.cc/9fKf2tYp/Logo.png"))
}
