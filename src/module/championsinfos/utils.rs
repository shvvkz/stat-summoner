
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter};
use mongodb::Collection;
use crate::models::data::{ChampionData, EmojiId};
use crate::models::error::Error;
use crate::utils::get_emoji;

/// ⚙️ Constructs a Discord embed containing detailed information about a League of Legends champion.
///
/// This function takes the champion's data and a collection of emojis to create a richly formatted Discord embed.
/// It includes the champion's roles, winrate, banrate, popularity, recommended runes (with emojis), and core item build (with emojis).
/// The embed is designed to provide users with an at-a-glance overview of the champion's statistics and recommended setups.
///
/// # Parameters:
/// - `champion_data`: A `ChampionData` struct containing the champion's information, including roles, runes, items, and statistics.
/// - `collection_emoji`: A reference to a MongoDB `Collection<EmojiId>` used to retrieve the appropriate emojis for runes and items.
///
/// # Returns:
/// - `Result<CreateEmbed, Error>`: On success, returns a `CreateEmbed` object representing the Discord embed.
///   On failure, returns an `Error` detailing what went wrong.
///
/// # ⚠️ Notes:
/// - The function retrieves emojis asynchronously for each rune and item using the `get_emoji` function.
/// - It formats numerical statistics (winrate, banrate, popularity) as percentages.
/// - The embed includes a thumbnail image of the champion, fetched from the Data Dragon API.
/// - The embed includes a footer indicating that the message will be deleted after 60 seconds.
///
/// # Example:
/// ```rust
/// let champion_data = /* Fetch or construct ChampionData */;
/// let embed = create_embed_champions_info(champion_data, &collection_emoji).await?;
/// ctx.send(|m| m.set_embed(embed)).await?;
/// ```
///
/// This function produces an embed displaying information such as:
/// ```text
/// 📝 Informations sur Jhin
/// Rôles: AD Carry
/// Winrate: 51.74%   Banrate: 17.61%   Popularité: 27.57%
///
/// Runes:
/// **Rune principale :** <FleetFootwork Emoji>
/// <PresenceOfMind Emoji> <LegendBloodline Emoji> <CoupDeGrace Emoji>
///
/// **Runes secondaires:**
/// <Celerity Emoji> <GatheringStorm Emoji>
///
/// **Fragments:** <AdaptiveForce Emoji> <AdaptiveForce Emoji> <HealthScale Emoji>
///
/// Build d'objets:
/// <StatikkShiv Emoji> <RapidFirecannon Emoji> <InfinityEdge Emoji>
/// ```
///
/// # Errors:
/// - If any of the asynchronous calls to `get_emoji` fail, the function returns an `Error`.
/// - If there is an issue parsing the numerical statistics from the `champion_data`.
///
/// # See Also:
/// - `get_emoji`: Retrieves the emoji string corresponding to a given rune or item name.
/// - `championsinfos`: The command that invokes this function to display the champion's information.
///
/// # Related Structures:
/// - `ChampionData`: Contains the champion's details used to construct the embed.
/// - `EmojiId`: Represents the mapping of rune or item names to their corresponding emoji IDs.
///
/// # Dependencies:
/// - This function relies on external data sources such as the Data Dragon API for champion images.
/// - It also depends on the MongoDB collection for retrieving emojis.
pub async fn create_embed_champions_info(
    champion_data: ChampionData,
    collection_emoji: &Collection<EmojiId>,
) -> Result<CreateEmbed, Error> {

    let primary_rune_emoji = get_emoji(
        collection_emoji.clone(),
        "rune",
        &champion_data.runes.parent_primary_rune,
    )
    .await?;

    let child_primary_rune_1_emoji = get_emoji(
        collection_emoji.clone(),
        "rune",
        &champion_data.runes.child_primary_rune_1,
    )
    .await?;

    let child_primary_rune_2_emoji = get_emoji(
        collection_emoji.clone(),
        "rune",
        &champion_data.runes.child_primary_rune_2,
    )
    .await?;

    let child_primary_rune_3_emoji = get_emoji(
        collection_emoji.clone(),
        "rune",
        &champion_data.runes.child_primary_rune_3,
    )
    .await?;

    let child_secondary_rune_1_emoji = get_emoji(
        collection_emoji.clone(),
        "rune",
        &champion_data.runes.child_secondary_rune_1,
    )
    .await?;

    let child_secondary_rune_2_emoji = get_emoji(
        collection_emoji.clone(),
        "rune",
        &champion_data.runes.child_secondary_rune_2,
    )
    .await?;

    // Récupérer les émojis pour les runes tertiaires
    let tertiary_rune_1_emoji = get_emoji(
        collection_emoji.clone(),
        "rune",
        &champion_data.runes.tertiary_rune_1,
    )
    .await?;

    let tertiary_rune_2_emoji = get_emoji(
        collection_emoji.clone(),
        "rune",
        &champion_data.runes.tertiary_rune_2,
    )
    .await?;

    let tertiary_rune_3_emoji = get_emoji(
        collection_emoji.clone(),
        "rune",
        &champion_data.runes.tertiary_rune_3,
    )
    .await?;

    // Récupérer les émojis pour les objets du build
    let core_item_1_emoji = get_emoji(
        collection_emoji.clone(),
        "item",
        &champion_data.core_build.first,
    )
    .await?;

    let core_item_2_emoji = get_emoji(
        collection_emoji.clone(),
        "item",
        &champion_data.core_build.second,
    )
    .await?;

    let core_item_3_emoji = get_emoji(
        collection_emoji.clone(),
        "item",
        &champion_data.core_build.third,
    )
    .await?;

    let popularity = champion_data
        .popularity
        .parse::<f64>()
        .unwrap_or(0.0) * 100.0;
    let winrate = champion_data
        .winrate
        .parse::<f64>()
        .unwrap_or(0.0) * 100.0;
    let banrate = champion_data
        .banrate
        .parse::<f64>()
        .unwrap_or(0.0) * 100.0;

    let runes_description = format!(
        "**Rune principale :** {}\n{} {} {}\n\n**Runes secondaires :** \n{} {}\n\n**Fragments :** {} {} {}",
        primary_rune_emoji,
        child_primary_rune_1_emoji,
        child_primary_rune_2_emoji,
        child_primary_rune_3_emoji,
        child_secondary_rune_1_emoji,
        child_secondary_rune_2_emoji,
        tertiary_rune_1_emoji,
        tertiary_rune_2_emoji,
        tertiary_rune_3_emoji
    );

    let core_build_description = format!(
        "{} {} {}",
        core_item_1_emoji, core_item_2_emoji, core_item_3_emoji
    );
    let embed = CreateEmbed::default()
        .title(format!("Informations sur {}", champion_data.name))
        .color(0x00ff00)
        .field("Rôles", champion_data.role.join(", "), false)
        .field("Winrate", format!("{:.2}%", winrate), true)
        .field("Banrate", format!("{:.2}%", banrate), true)
        .field("Popularité", format!("{:.2}%", popularity), true)
        .field("Runes", runes_description, false)
        .field("Build d'objets", core_build_description, false)
        .footer(CreateEmbedFooter::new(
            "Ce message sera supprimé dans 60 secondes.",
        ))
        .thumbnail(format!(
            "https://ddragon.leagueoflegends.com/cdn/14.14.1/img/champion/{}.png",
            champion_data.id_name
        ));

    Ok(embed)
}
