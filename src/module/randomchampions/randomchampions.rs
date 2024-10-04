use crate::embed::schedule_message_deletion;
use crate::models::data::{Data, EmojiId};
use crate::models::error::Error;
use crate::models::role::Role;
use crate::module::championsinfos::utils::create_embed_champions_info;
use crate::module::randomchampions::utils::{get_list_champions, get_random_champion};
use poise::CreateReply;

/// Generates a random League of Legends champion embed and sends it as a Discord message.
///
/// This command selects a random champion from the available list, optionally filtered by role, and constructs a detailed Discord embed
/// containing information about that champion. The embed includes roles, winrate, banrate, popularity, runes (with emojis),
/// and the core item build for the randomly selected champion.
///
/// # Parameters:
/// - `ctx`: The command's context, providing access to the bot, the message, and other utilities.
/// - `role`: An optional parameter specifying the role of the champion. If provided, the champion list will be filtered accordingly.
///
/// # Returns:
/// - `Result<(), Error>`: Returns `Ok(())` if the command executes successfully, otherwise returns an `Error`.
///
/// # ⚠️ Notes:
/// - The function calls `get_list_champions` to retrieve a list of champions, optionally filtered by role.
/// - It uses `get_random_champion` to randomly select a champion from the filtered list.
/// - `create_embed_champions_info` is called to construct a richly formatted embed with the champion's details.
/// - After sending the embed, the message is scheduled for deletion after 60 seconds to keep the chat clean.
///
/// # Example:
/// ```rust
/// randomchampions(ctx, Some(Role::Top)).await?;
/// ```
///
/// This command produces an embed displaying information such as:
/// ```text
/// 📝 Information about Jhin
/// Roles: AD Carry
/// Winrate: 51.74%   Banrate: 17.61%   Popularity: 27.57%
///
/// Runes:
/// **Primary Rune:** <FleetFootwork Emoji>
/// <PresenceOfMind Emoji> <LegendBloodline Emoji> <CoupDeGrace Emoji>
///
/// **Secondary Runes:**
/// <Celerity Emoji> <GatheringStorm Emoji>
///
/// **Shards:** <AdaptiveForce Emoji> <AdaptiveForce Emoji> <HealthScale Emoji>
///
/// Item Build:
/// <StatikkShiv Emoji> <RapidFirecannon Emoji> <InfinityEdge Emoji>
/// ```
///
/// # Errors:
/// - If the retrieval of the champion list fails (`get_list_champions`), the function returns an `Error`.
/// - If there is an issue constructing the embed (`create_embed_champions_info`), an `Error` will be returned.
/// - If the message deletion fails, it will log the error, but the command will still complete successfully.
///
/// # See Also:
/// - `get_list_champions`: Retrieves the list of champions, filtered by role if specified.
/// - `get_random_champion`: Selects a random champion from the provided list.
/// - `create_embed_champions_info`: Constructs the champion information embed to be sent.
/// - `schedule_message_deletion`: Schedules a message for deletion after a specific time interval to maintain chat cleanliness.
///
/// # Related Structures:
/// - `ChampionData`: Contains the champion's details used to construct the embed.
/// - `EmojiId`: Represents the mapping of rune or item names to their corresponding emoji IDs.
///
/// # Dependencies:
/// - This function relies on a MongoDB collection for retrieving emojis.
/// - The embed includes images fetched from the Data Dragon API.
#[poise::command(slash_command)]
pub async fn randomchampions(
    ctx: poise::ApplicationContext<'_, Data, Error>,
    #[description = "Select a role (optional)"] role: Option<Role>,
) -> Result<(), Error> {
    let champions_list = get_list_champions(ctx, role).await?;
    let mongo_client = &ctx.data().mongo_client;
    let collection_emoji = mongo_client
        .database("stat-summoner")
        .collection::<EmojiId>("emojis_id");
    let champion_data = get_random_champion(champions_list);
    let embed = create_embed_champions_info(champion_data, &collection_emoji).await?;
    let reply = CreateReply {
        embeds: vec![embed],
        ..Default::default()
    };
    let sent_message = ctx.send(reply).await?;
    if let Err(e) = schedule_message_deletion(sent_message, ctx).await {
        log::error!("Failed to schedule message deletion: {}", e);
    }
    Ok(())
}
