use mongodb::bson::doc;
use crate::embed::schedule_message_deletion;
use crate::models::data::{Data, SummonerFollowedData};
use crate::models::error::Error;
use crate::module::whoisfollowed::utils::{get_data_followed_summoner, create_embed_followed_summoner};

/// Retrieves and displays the list of summoners followed in the current Discord guild.
///
/// This slash command fetches the summoners being followed within the Discord guild where the command is invoked.
/// It queries the MongoDB collection for follow data and creates an embed message that lists all tracked summoners, along with the time remaining for each follow.
/// The message is set to automatically delete after 60 seconds.
///
/// # Parameters:
/// - `ctx`: The context of the command, which includes information about the current Discord guild, channel, and bot data.
///   The `ctx` is used to access the MongoDB client, retrieve the guild's ID, and send the resulting message.
///
/// # Returns:
/// - `Result<(), Error>`: Returns `Ok(())` on success, or an `Error` if something goes wrong during database access or message creation.
///
/// # ⚠️ Notes:
/// - The function retrieves the guild's ID and queries the `follower_summoner` collection for summoners being tracked in that guild.
/// - It uses the `get_data_followed_summoner` function to gather the data and the `create_embed_followed_summoner` function to construct the embed message.
/// - The message is automatically deleted after 60 seconds using the `schedule_message_deletion` function.
/// - The command can only be used in a Discord server (guild), not in direct messages.
///
/// # Example:
/// ```rust
/// #[poise::command(slash_command)]
/// pub async fn whoisfollowed(
///     ctx: poise::ApplicationContext<'_, Data, Error>,
/// ) -> Result<(), Error> {
///     let mongo_client = &ctx.data().mongo_client;
///     let collection = mongo_client
///         .database("stat-summoner")
///         .collection::<SummonerFollowedData>("follower_summoner");
///
///     let guild_id = ctx.guild_id().map(|id| id.get()).unwrap_or(0).to_string();
///
///     let followed_data = get_data_followed_summoner(collection, guild_id).await?;
///
///     let reply = ctx.send(create_embed_followed_summoner(followed_data)).await?;
///     schedule_message_deletion(reply, ctx).await?;
///     return Ok(());
/// }
/// ```
///
/// This command will create an embed showing all followed summoners in the guild where the command is run, along with their remaining follow time.
#[poise::command(
    slash_command,
)]
pub async fn whoisfollowed(
    ctx: poise::ApplicationContext<'_, Data, Error>,
    ) -> Result<(), Error> {
        let mongo_client = &ctx.data().mongo_client;
        let collection = mongo_client
            .database("stat-summoner")
            .collection::<SummonerFollowedData>("follower_summoner");

        let guild_id = ctx.guild_id().map(|id| id.get()).unwrap_or(0).to_string();
        let followed_data = get_data_followed_summoner(collection, guild_id).await?;
        let reply = ctx.send(create_embed_followed_summoner(followed_data)).await?;
        schedule_message_deletion(reply, ctx).await?;
        return Ok(());
    }
