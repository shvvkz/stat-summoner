use crate::models::{Data, Error, SummonerFollowedData};
use mongodb::bson::doc;
use tracing::log::error;

#[poise::command(slash_command)]
pub async fn followgames(
    ctx: poise::ApplicationContext<'_, Data, Error>
) -> Result<(), Error> {
    // Utiliser le client MongoDB déjà présent dans `ctx.data()`
    let mongo_client = &ctx.data().mongo_client;

    // Collection de la base de données MongoDB
    let collection = mongo_client
        .database("stat-summoner")
        .collection::<SummonerFollowedData>("follower_summoner");

    // Requête MongoDB pour récupérer les informations d'un utilisateur avec un `puuid` donné
    let puuid_to_search = "456"; // Remplacez ceci par le `puuid` que vous souhaitez rechercher
    match collection.find_one(doc! { "puuid": puuid_to_search }).await {
        Ok(Some(followed_summoner)) => {
            // Afficher les données du joueur dans Discord, y compris le champ last_match
            let last_match_id = format!("Dernier match ID: {}", followed_summoner.last_match_id);

            // Afficher les données du joueur dans Discord
            ctx.say(format!(
                "Données du joueur trouvées : {:?}, {}",
                followed_summoner, last_match_id
            ))
            .await?;
        }
        Ok(None) => {
            // Aucun document trouvé
            ctx.say("Aucune donnée de joueur trouvée pour ce PUUID.").await?;
        }
        Err(e) => {
            // Erreur lors de la recherche dans MongoDB
            error!("Erreur lors de la recherche MongoDB: {:?}", e);
            ctx.say("Erreur lors de la recherche MongoDB.").await?;
        }
    }

    Ok(())
}
