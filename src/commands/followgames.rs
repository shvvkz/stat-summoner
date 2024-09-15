use crate::models::{Data, Error, SummonerFollowedData};
use mongodb::{bson::doc, options::{ClientOptions, ServerApi, ServerApiVersion}, Client};
use poise::serenity_prelude::client;
use tracing::log::error;

/// This command will allow users to follow live games.
/// Functionality will be implemented soon.
#[poise::command(slash_command)]
pub async fn followgames(
    ctx: poise::ApplicationContext<'_, Data, Error>
) -> Result<(), Error> {
    // Initialiser les options du client MongoDB
    let mut client_options = ClientOptions::parse(&ctx.data().mongodb_uri).await?;

    // Définir la version de l'API MongoDB sur Stable API v1
    let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
    client_options.server_api = Some(server_api);

    // Connexion au cluster MongoDB
    let client = Client::with_options(client_options)?;

    // Ping la base de données pour tester la connexion
    match client
        .database("stat-summoner")
        .run_command(doc! { "ping": 1 })
        .await
    {
        Ok(_) => {
            // Si le ping réussit, enregistrer un message de succès
            ctx.say("Ping MongoDB réussi !").await?;
        }
        Err(e) => {
            // En cas d'erreur, capturer et afficher l'erreur
            error!("Erreur lors du ping de MongoDB: {:?}", e);
            ctx.say("Erreur lors du ping de MongoDB.").await?;
        }
    }

    // Collection de la base de données MongoDB
    let collection = client
        .database("stat-summoner")
        .collection::<SummonerFollowedData>("summoner_data");

    // Requête MongoDB pour récupérer les informations d'un utilisateur avec un `puuid` donné
    let puuid_to_search = "123"; // Remplacez ceci par le `puuid` que vous souhaitez rechercher
    match collection.find_one(doc! { "puuid": puuid_to_search }).await {
        Ok(Some(summoner_data)) => {
            // Si un document est trouvé, l'afficher dans le contexte Discord
            ctx.say(format!(
                "Données du joueur trouvées : {:?}",
                summoner_data
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
