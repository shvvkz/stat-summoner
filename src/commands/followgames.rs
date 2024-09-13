use poise::Modal;
use reqwest::Client;
use std::collections::HashMap;

/// This command will allow users to follow live games.
/// Functionality will be implemented soon.
#[poise::command(slash_command)]
pub async fn followgames(ctx: Context<'_>) -> Result<(), Error> {
    // Inform the user that this feature is not available yet
    ctx.say("This feature will be added soon. Stay tuned!").await?;

    Ok(())
}
