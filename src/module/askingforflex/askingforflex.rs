use poise::serenity_prelude::{
    CreateEmbed, CreateEmbedFooter, CreateMessage, Mentionable,
};
use poise::Modal;
use crate::embed::{create_embed_error, schedule_message_deletion};
use crate::models::data::Data;
use crate::models::error::Error;
use crate::models::modal::FlexAskingModal;
use crate::utils::manage_user;

#[poise::command(slash_command)]
pub async fn askingforflex(ctx: poise::ApplicationContext<'_, Data, Error>) -> Result<(), Error> {
    manage_user(
        ctx.author().id.to_string(),
        ctx.author().name.clone(),
        &ctx.data().mongo_client,
        false,
    )
    .await?;

    // Extraire l'ID et les rôles du serveur AVANT await
    let guild_roles = ctx.guild().map(|g| g.roles.clone()); // Clonage des rôles pour éviter le problème de `Send`

    // Exécuter le modal et récupérer les données utilisateur
    let modal_data = match FlexAskingModal::execute(ctx).await {
        Ok(Some(data)) => FlexAskingModal {
            starting_hour: {
                let time = data.starting_hour.trim();
                if time.len() == 5 && time.chars().nth(2) == Some(':') {
                    let hours = time[..2].parse::<u8>();
                    let minutes = time[3..].parse::<u8>();
                    if let (Ok(h), Ok(m)) = (hours, minutes) {
                        if h < 24 && m < 60 {
                            time.to_string()
                        } else {
                            let error_message = "Invalid time format. Hours must be between 00 and 23, and minutes between 00 and 59.";
                            let reply = ctx.send(create_embed_error(&error_message)).await?;
                            schedule_message_deletion(reply, ctx).await?;
                            return Ok(());
                        }
                    } else {
                        let error_message = "Invalid time format. Please use HH:MM format.";
                        let reply = ctx.send(create_embed_error(&error_message)).await?;
                        schedule_message_deletion(reply, ctx).await?;
                        return Ok(());
                    }
                } else {
                    let error_message = "Invalid time format. Please use HH:MM format.";
                    let reply = ctx.send(create_embed_error(&error_message)).await?;
                    schedule_message_deletion(reply, ctx).await?;
                    return Ok(());
                }
            },
            people_to_ask_by_role: data.people_to_ask_by_role.replace("@", ""),
        },
        Ok(None) => {
            let error_message = "Modal data not found.";
            let reply = ctx.send(create_embed_error(&error_message)).await?;
            schedule_message_deletion(reply, ctx).await?;
            return Ok(());
        }
        Err(e) => {
            println!("Error: {:?}", e);
            let error_message = "Failed to retrieve modal data.";
            let reply = ctx.send(create_embed_error(&error_message)).await?;
            schedule_message_deletion(reply, ctx).await?;
            return Ok(());
        }
    };

    // Récupérer le rôle mentionné (hors async pour éviter Send)
    let role_name = modal_data.people_to_ask_by_role.trim();
    let role_mention = match guild_roles {
        Some(roles) => roles
            .values()
            .find(|role| role.name.to_lowercase() == role_name.to_lowercase())
            .map(|role| role.id.mention().to_string()) // Mention réelle
            .unwrap_or_else(|| format!("@{}", role_name)), // Si le rôle n'existe pas, texte brut
        None => format!("@{}", role_name), // Pas de guilde trouvée
    };

    // Message avec mention du rôle
    let flex_message = format!(
        "Hey {} please wake up! {} is looking for a flex game at **{}**!",
        role_mention,
        ctx.author().mention(),
        modal_data.starting_hour
    );

    // Création de l'embed
    let embed = create_embed_flex(&ctx.author().name, &modal_data);

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
    let channel_id = ctx.channel_id();
    channel_id.send_message(&ctx.http(), builder).await?;

    Ok(())
}

// Fonction pour créer l'embed
fn create_embed_flex(author: &str, modal_data: &FlexAskingModal) -> CreateEmbed {
    CreateEmbed::default()
        .color(0x00ff00)
        .title("Flex Game Request")
        .description("A player is looking for a flex team!")
        .field("Requested by", author, true)
        .field("Start Time", &modal_data.starting_hour, true)
        .field("Top:", "TBD", false)
        .field("Jungle:", "TBD", false)
        .field("Mid:", "TBD", false)
        .field("ADCarry:", "TBD", false)
        .field("Support:", "TBD", false)
        .footer(CreateEmbedFooter::new(
            "This message will be deleted in 60 seconds.",
        ))
        .thumbnail("https://i.postimg.cc/9fKf2tYp/Logo.png")
}
