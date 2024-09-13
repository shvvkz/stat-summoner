// use poise::Modal;
// use reqwest::Client;
// use std::collections::HashMap;
use crate::models::{Error, Data};

/// This command will allow users to follow live games.
/// Functionality will be implemented soon.
#[poise::command(slash_command)]
pub async fn followgames(
    ctx: poise::ApplicationContext<'_, Data, Error>
    ) -> Result<(), Error> {
        // Inform the user that this feature is not available yet
        ctx.say("This feature will be added soon. Stay tuned!").await?;

        Result::Ok(())
    }
