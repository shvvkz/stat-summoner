use crate::models::error::Error;
use crate::models::modal::LolStatsModal;
use crate::{
    models::data::{Data, EmojiId},
    utils::get_emoji,
};
use mongodb::Collection;
use poise::ReplyHandle;
use poise::{
    serenity_prelude::{self as serenity},
    CreateReply,
};
use serde_json::Value;
use serenity::builder::{CreateEmbed, CreateEmbedFooter};
use tokio::time::{sleep, Duration};

/// ⚙️ **Function**: Creates a rich embed message displaying League of Legends player stats and match details.
///
/// This function constructs a `CreateEmbed` message containing information about the player's Solo/Duo and Flex ranks,
/// top champions, and detailed information about recent matches. The generated embed is used for displaying formatted
/// stats in Discord messages.
///
/// # Parameters:
/// - `modal_data`: Contains the player's in-game name and tag, used to personalize the embed title.
/// - `solo_rank`: A JSON-like value containing the player's Solo/Duo rank information, including tier, division, LP, wins, losses, and winrate.
/// - `flex_rank`: A JSON-like value containing the player's Flex rank information, similar to `solo_rank`.
/// - `champions_info`: A formatted string representing the player's top champions, their levels, and mastery points.
/// - `match_details`: A vector of JSON-like values representing detailed match information, including K/D/A, farm, game duration, and result.
///
/// # Returns:
/// - `CreateEmbed`: The formatted embed message ready to be sent in a Discord channel.
///
/// # ⚠️ Notes:
/// - If no match details are available, the embed will indicate that no recent normal or ranked matches were found.
/// - The embed displays rank information differently depending on whether the player has earned League Points (LP) in their rank.
///
/// # Example:
/// ```rust
/// let embed = create_embed(modal_data, solo_rank, flex_rank, champions_info, match_details);
/// ctx.send(|m| m.set_embed(embed)).await?;
/// ```
///
/// The resulting embed will contain information such as:
/// ```text
/// 📊 Stats for Faker#1234
/// 🔱 **Solo/Duo Rank**: Gold I (100 LP)
/// 🌀 **Flex Rank**: Silver IV (50 LP)
/// 💥 **Top Champions**:
/// Yasuo - Level: 7 - Points: 123456
/// 📜 **Match Details**:
/// Victory - **Yasuo**, 2 hours ago (Ranked Solo/Duo):
/// K/D/A: **10/2/8** | **200 CS** | Duration: **30:45**
/// ⏳ Played: **2 hours ago**
/// ```
pub async fn create_embed(
    modal_data: &LolStatsModal,
    solo_rank: Value,
    flex_rank: Value,
    champions_info: String,
    match_details: Vec<Value>,
    collection_emoji: Collection<EmojiId>,
) -> Result<CreateEmbed, Error> {
    // Récupérer les émojis pour le rang solo et flex
    let solo_rank_tier = solo_rank["tier"].as_str().unwrap_or("Unknown");
    let solo_emoji = get_emoji(collection_emoji.clone(), "rank", solo_rank_tier)
        .await
        .unwrap_or(solo_rank_tier.to_string());
    println!("{}", solo_emoji);

    let flex_rank_tier = flex_rank["tier"].as_str().unwrap_or("Unknown");
    let flex_emoji = get_emoji(collection_emoji.clone(), "rank", flex_rank_tier)
        .await
        .unwrap_or(flex_rank_tier.to_string());

    // Construction de la chaîne du rang Solo/Duo
    let solo_rank_str = if solo_rank["lp"].as_i64().unwrap_or(0) > 0 {
        if !solo_rank["division"].as_str().unwrap_or("").is_empty() {
            format!(
                "**{} {}** - {} LP",
                solo_emoji,
                solo_rank["division"].as_str().unwrap(),
                solo_rank["lp"].as_i64().unwrap()
            )
        } else {
            format!(
                "**{}** - {} LP",
                solo_emoji,
                solo_rank["lp"].as_i64().unwrap()
            )
        }
    } else {
        format!("**{}**", solo_emoji)
    };

    // Construction de la chaîne du rang Flex
    let flex_rank_str = if flex_rank["lp"].as_i64().unwrap_or(0) > 0 {
        if !flex_rank["division"].as_str().unwrap_or("").is_empty() {
            format!(
                "**{} {}** ({} LP)",
                flex_emoji,
                flex_rank["division"].as_str().unwrap(),
                flex_rank["lp"].as_i64().unwrap()
            )
        } else {
            format!(
                "**{}** ({} LP)",
                flex_emoji,
                flex_rank["lp"].as_i64().unwrap()
            )
        }
    } else {
        format!("**{}**", flex_emoji)
    };

    // Construction de l'embed
    let embed = CreateEmbed::default()
        .title(format!("📊 Stats for **{}#{}**", modal_data.game_name, modal_data.tag_line))
        .color(0x00ff00)
        .field("**Solo/Duo Rank**", solo_rank_str, false)
        .field("🏆 **Wins**", format!("**{}**", solo_rank["wins"].as_i64().unwrap_or(-1)), true)
        .field("❌ **Losses**", format!("**{}**", solo_rank["losses"].as_i64().unwrap_or(-1)), true)
        .field(
            "📊 **Winrate**",
            format!("**{:.2}%**", solo_rank["winrate"].as_f64().unwrap_or(-1.0)),
            true
        )
        .field("**Flex Rank**", flex_rank_str, false)
        .field("🏆 **Wins**", format!("**{}**", flex_rank["wins"].as_i64().unwrap_or(-1)), true)
        .field("❌ **Losses**", format!("**{}**", flex_rank["losses"].as_i64().unwrap_or(-1)), true)
        .field(
            "📊 **Winrate**",
            format!("**{:.2}%**", flex_rank["winrate"].as_f64().unwrap_or(-1.0)),
            true
        )
        .field("💥 **Top Champions**", champions_info, false)
        .field(
            "📜 **Match Details**",
            if match_details.is_empty() {
                "No match found on Normal and ranked game".to_string()
            } else {
                match_details
                    .iter()
                    .map(|match_detail| {
                        format!(
                            "{} - **{}**, {} ({}):\nK/D/A: **{}** | **{} CS** | Duration: **{}**\n⏳ Played: **{}**\n\n",
                            match_detail.get("Result").unwrap().as_str().unwrap(),
                            match_detail.get("champion_name").unwrap().as_str().unwrap(),
                            match_detail.get("time_elapsed").unwrap().as_str().unwrap(),
                            match_detail.get("game_type").unwrap().as_str().unwrap(),
                            match_detail.get("K/D/A").unwrap().as_str().unwrap(),
                            match_detail.get("Farm").unwrap().as_u64().unwrap(),
                            match_detail.get("Duration").unwrap().as_str().unwrap(),
                            match_detail.get("time_elapsed").unwrap().as_str().unwrap()
                        )
                    })
                    .collect::<String>()
            },
            false
        )
        .footer(CreateEmbedFooter::new("This message will be deleted in 60 seconds."))
        .thumbnail("https://i.postimg.cc/VL3pc27P/Frame-102-1.png");

    Ok(embed)
}

/// ⚙️ **Function**: Creates an embed displaying an error message for Discord interactions.
///
/// This function constructs a Discord embed message that displays a given error message in a formatted way.
/// The embed is styled with a red color to indicate an error and includes a default title of "Error".
/// The embed is returned as part of a `CreateReply`, which can be sent to a Discord channel.
///
/// # Parameters:
/// - `error_message`: A string slice containing the error message to be displayed in the embed's description.
///   This message is intended to provide feedback to the user, typically in case of API errors, invalid inputs,
///   or other issues encountered during the bot's execution.
///
/// # Returns:
/// - `CreateReply`: A response object that includes the error embed. This is ready to be sent to a Discord channel.
///
/// # ⚠️ Notes:
/// - The embed's color is set to red (`0xff0000`) to visually signify an error.
/// - The title of the embed is always set to "Error", and the provided `error_message` is used in the description.
/// - The function is primarily used to provide user-friendly error messages in response to invalid inputs
///   or issues in API calls.
///
/// # Example:
/// ```rust
/// let error_reply = create_embed_error("Failed to fetch data from the Riot API.");
/// ctx.send(error_reply).await?;
/// ```
///
/// The resulting embed message will look like this:
/// ```text
/// ❌ **Error**
/// Failed to fetch data from the Riot API.
/// ```
pub fn create_embed_error(error_message: &str) -> CreateReply {
    let embed: CreateEmbed = CreateEmbed::default()
        .title("Error")
        .description(error_message)
        .color(0xff0000)
        .footer(CreateEmbedFooter::new(
            "This message will be deleted in 60 seconds.",
        ));
    CreateReply {
        embeds: vec![embed],
        ..Default::default()
    }
}

/// ⚙️ **Function**: Creates a success embed reply for Discord messages.
///
/// This function generates a Discord embed with the title "Sucess", a description provided by the `sucess_message` parameter,
/// a green color to indicate success, and a footer notifying that the message will be deleted in 60 seconds. It returns a
/// `CreateReply` containing the embed, suitable for sending as a response to a Discord interaction or message.
///
/// # Parameters:
/// - `sucess_message`: A string slice that holds the success message to be displayed in the embed's description.
///
/// # Returns:
/// - `CreateReply`: A Discord reply containing the constructed success embed.
///
/// # ⚠️ Notes:
/// - The embed's footer is in French: "This message will be deleted in 60 seconds." ("This message will be deleted in 60 seconds.").
/// - The title "Sucess" appears to have a typo and might be intended as "Success".
/// - The embed uses a green color (`0x00ff00`) to visually indicate a successful operation.
///
/// # Example:
/// ```rust
/// let reply = create_embed_sucess("Operation completed successfully!");
/// // Use `reply` to send the embed in a Discord channel
/// ```
pub fn create_embed_sucess(sucess_message: &str) -> CreateReply {
    let embed: CreateEmbed = CreateEmbed::default()
        .title("Sucess")
        .description(sucess_message)
        .color(0x00ff00)
        .footer(CreateEmbedFooter::new(
            "This message will be deleted in 60 seconds.",
        ));
    CreateReply {
        embeds: vec![embed],
        ..Default::default()
    }
}

/// ⚙️ **Function**: Schedules the deletion of a Discord message after a delay.
///
/// This function delays the deletion of a Discord message by 60 seconds. After the delay, the function attempts
/// to delete the message from the channel. It ensures that messages sent by the bot (e.g., error messages or
/// status updates) are automatically removed after a certain time to keep the chat clean.
///
/// # Parameters:
/// - `sent_message`: A `ReplyHandle` representing the message to be deleted. This handle provides access to the
///   message object, allowing the function to delete it once the delay has passed.
/// - `ctx`: The application context, which provides access to Discord methods (such as message deletion)
///   and other necessary data like API keys.
///
/// # Returns:
/// - `Result<(), Error>`: If successful, returns `Ok(())`. If an error occurs while fetching the message
///   or deleting it, returns an `Error`.
///
/// # ⚠️ Notes:
/// - The function uses `tokio::time::sleep` to pause execution for 60 seconds before attempting to delete the message.
/// - If the message cannot be fetched (e.g., due to permissions or being deleted manually), the deletion attempt will silently fail.
/// - This function is typically used in scenarios where temporary messages (like errors or status updates)
///   need to be cleaned up automatically after a short period.
///
/// # Example:
/// ```rust
/// schedule_message_deletion(sent_message, ctx).await?;
/// ```
///
/// After 60 seconds, the message will be deleted from the Discord channel.
pub async fn schedule_message_deletion(
    sent_message: ReplyHandle<'_>,
    ctx: poise::ApplicationContext<'_, Data, Error>,
) -> Result<(), Error> {
    sleep(Duration::from_secs(60)).await;
    if let Ok(sent_msg) = sent_message.message().await {
        sent_msg.delete(&ctx.serenity_context().http).await?;
    }
    Ok(())
}
