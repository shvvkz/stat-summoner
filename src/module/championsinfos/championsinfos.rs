use mongodb::bson::doc;
use poise::{CreateReply, Modal};
use crate::embed::{create_embed_error, schedule_message_deletion};
use crate::models::data::{ChampionData, Data, EmojiId};
use crate::models::error::Error;
use crate::models::modal::ChampionsInfosModal;
use crate::module::championsinfos::utils::create_embed_champions_info;
use crate::utils::{get_champion_id, get_champion_names};
use strsim::normalized_levenshtein;

/// Fetches and displays detailed information about a League of Legends champion based on user input.
///
/// This Discord command allows a user to input the name of a League of Legends champion.
/// It then fetches the champion's data from the database, including roles, winrate, banrate, popularity,
/// recommended runes, and core item build. The information is displayed in a formatted embed with appropriate emojis,
/// and the message is automatically deleted after a certain period to keep the chat clean.
///
/// # Parameters:
/// - `ctx`: The application context, providing access to Discord interaction methods, data dragon JSON, and the MongoDB client.
///
/// # Returns:
/// - `Result<(), Error>`: If successful, returns `Ok(())`; otherwise, returns an error.
///
/// # ⚠️ Notes:
/// - The command opens a modal dialog to collect the champion's name from the user.
/// - It uses fuzzy matching to find the best match for the champion name if the input is not exact.
/// - The message displaying the champion's information is automatically deleted after 60 seconds to keep the chat clean.
///
/// # Example:
/// ```rust
/// championsinfos(ctx).await?;
/// ```
///
/// This command displays information such as:
/// ```text
/// 📝 Informations sur Jhin
/// Rôles: AD Carry
/// Winrate: 51.74%
/// Banrate: 17.61%
/// Popularité: 27.57%
///
/// Runes:
/// **Rune principale:** <FleetFootwork Emoji>
/// <PresenceOfMind Emoji> <LegendBloodline Emoji> <CoupDeGrace Emoji>
///
/// **Runes secondaires:** <Celerity Emoji>
/// <GatheringStorm Emoji>
///
/// **Fragments:** <AdaptiveForce Emoji> <AdaptiveForce Emoji> <HealthScale Emoji>
///
/// Build d'objets:
/// <StatikkShiv Emoji> <RapidFirecannon Emoji> <InfinityEdge Emoji>
/// ```
///
/// # Errors:
/// - If the user does not provide any input in the modal.
/// - If the champion is not found in the database.
/// - If there is an error retrieving data from the database or during any asynchronous operation.
///
/// # See Also:
/// - `create_embed_champions_info`: Constructs the embed with the champion's information.
/// - `ChampionsInfosModal`: The modal dialog used to collect the champion's name from the user.
///
/// # Related Commands:
/// - `lolstats`: Fetches and displays LoL player stats based on user input.
#[poise::command(slash_command)]
pub async fn championsinfos(
    ctx: poise::ApplicationContext<'_, Data, Error>,
) -> Result<(), Error> {
    let modal_data: ChampionsInfosModal = match ChampionsInfosModal::execute(ctx).await {
        Ok(Some(data)) => data,
        Ok(None) => {
            let error_message = "Aucune donnée n'a été entrée.";
            let reply = ctx.send(create_embed_error(&error_message)).await?;
            schedule_message_deletion(reply, ctx).await?;
            return Ok(());
        }
        Err(_) => {
            let error_message = "Échec de la récupération des données du modal.";
            let reply = ctx.send(create_embed_error(&error_message)).await?;
            schedule_message_deletion(reply, ctx).await?;
            return Ok(());
        }
    };

    let input_name = modal_data.champion_name.trim().to_lowercase();
    let dd_json = &ctx.data().dd_json;
    let champion_names = get_champion_names(dd_json);
    if champion_names.is_empty() {
        let error_message = "Impossible de récupérer la liste des champions.";
        let reply = ctx.send(create_embed_error(&error_message)).await?;
        schedule_message_deletion(reply, ctx).await?;
        return Ok(());
    }

    let matched_champion = champion_names
        .iter()
        .max_by(|a, b| {
            let score_a = normalized_levenshtein(&input_name, &a.to_lowercase());
            let score_b = normalized_levenshtein(&input_name, &b.to_lowercase());
            score_a
                .partial_cmp(&score_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap();
    let matched_champion_id = get_champion_id(dd_json, matched_champion).unwrap();
    println!("Champion ID: {}", matched_champion_id);

    let mongo_client: &mongodb::Client = &ctx.data().mongo_client;
    let filter = doc! { "id_name": matched_champion_id};
    let collection_champions = mongo_client
        .database("stat-summoner")
        .collection::<ChampionData>("champions_data");
    let collection_emoji = mongo_client
        .database("stat-summoner")
        .collection::<EmojiId>("emojis_id");
    match collection_champions.find_one(filter).await {
        Ok(Some(champion_data)) => {
            let embed = create_embed_champions_info(champion_data, &collection_emoji).await?;
            let reply = CreateReply {embeds: vec![embed], ..Default::default()};
            let sent_message = ctx.send(reply).await?;
            if let Err(e) = schedule_message_deletion(sent_message, ctx).await {
                eprintln!("Failed to schedule message deletion: {}", e);
            }
        }
        Ok(None) => return Ok(()),
        Err(e) => {
            eprintln!("Erreur lors de la recherche de l'emoji: {:?}", e);
            return Ok(());
        }
    }


    Ok(())
}

