use crate::models::data::SummonerFollowedData;
use crate::models::error::Error;
use chrono::{Duration, Utc};
use futures::StreamExt;
use mongodb::bson::doc;
use mongodb::Collection;
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter};
use poise::CreateReply;
use serde_json::json;
use serde_json::Value;

/// ⚙️ **Function**: Fetches the list of summoners followed in a specific Discord guild.
///
/// This asynchronous function retrieves data about summoners followed within a particular Discord guild.
/// It queries the provided MongoDB collection for records matching the specified `guild_id` and
/// returns a list of summoners, along with the remaining follow duration for each.
/// If the follow has ended, it will return "Follow ended" for that summoner.
///
/// # Parameters:
/// - `collection`: The MongoDB collection containing follow data, where each document represents a summoner being followed.
/// - `guild_id`: A `String` representing the unique identifier of the Discord guild. This is used to filter the summoners
///   being followed in that specific guild.
///
/// # Returns:
/// - `Result<Value, Error>`: On success, it returns a `serde_json::Value` object containing a list of tracked summoners,
///   each with their `name` and `time_remaining` (formatted as a human-readable string or "Follow ended" if the follow has expired).
///   In case of an error, it returns an `Error` object.
///
/// # ⚠️ Notes:
/// - The function calculates the remaining follow duration by comparing the current timestamp with the `time_end_follow`
///   value from each summoner's record.
/// - If a summoner's follow has expired, the time remaining is returned as "Follow ended".
/// - The duration is formatted as a readable string for convenience.
///
/// # Example:
/// ```rust
/// let collection: Collection<SummonerFollowedData> = db.collection("follower_summoner");
/// let guild_id = "1234567890".to_string();
/// let result = get_data_followed_summoner(collection, guild_id).await?;
///
/// // The result would look like:
/// /// ```json
/// /// {
/// ///   "tracked_summoners": [
/// ///     {
/// ///       "name": "Summoner1",
/// ///       "time_remaining": "2 hours 15 minutes"
/// ///     },
/// ///     {
/// ///       "name": "Summoner2",
/// ///       "time_remaining": "Follow ended"
/// ///     }
/// ///   ]
/// /// }
/// ```
pub async fn get_data_followed_summoner(
    collection: Collection<SummonerFollowedData>,
    guild_id: String,
) -> Result<Value, Error> {
    let current_timestamp = Utc::now().timestamp();
    let mut cursor = collection.find(doc! { "guild_id": guild_id }).await?;
    let mut summoners = Vec::new();
    while let Some(followed_data) = cursor.next().await {
        if let Ok(data) = followed_data {
            let name = &data.name;
            let time_end_follow = data.time_end_follow.parse::<i64>().unwrap();

            let remaining_duration = time_end_follow - current_timestamp;
            let time_remaining_str = if remaining_duration > 0 {
                let duration = Duration::seconds(remaining_duration);
                format_duration(duration)
            } else {
                "Follow ended".to_string()
            };
            let summoner = json!({
                "name": name,
                "time_remaining": time_remaining_str
            });
            summoners.push(summoner);
        }
    }
    Ok(json!({ "tracked_summoners": summoners }))
}

/// ⚙️ **Function**: Formats a `Duration` into a human-readable string.
///
/// This function takes a `Duration` and returns a string representing the remaining time in a human-readable format.
/// The function distinguishes between days, hours, and minutes, with specific rules for singular and plural terms.
/// If the remaining time is less than a minute, it returns "less than a minute".
///
/// # Parameters:
/// - `duration`: A `Duration` object representing the time span to format. The function will extract the number of
///   days, hours, and minutes from this duration to create a user-friendly time description.
///
/// # Returns:
/// - `String`: A human-readable string indicating how much time is left, formatted as:
/// - "in 1 day", "in 1 day and X hours", "in X hours", "in X minutes", or "less than a minute".
///   The string changes based on the length of the duration.
///
/// # ⚠️ Notes:
/// - If the duration is greater than a day, the function formats the result as "in X days and Y hours",
///   or "in X days" if there are no remaining hours.
/// - If the duration is less than a day but more than an hour, the result is formatted as "in X hours".
/// - For durations less than an hour but more than a minute, it returns "in X minutes".
/// - If the duration is less than a minute, the function returns "less than a minute".
///
/// # Example:
/// ```rust
/// let duration = Duration::hours(5);
/// let formatted = format_duration(duration);
/// assert_eq!(formatted, "in 5 hours");
///
/// let short_duration = Duration::minutes(1);
/// let formatted_short = format_duration(short_duration);
/// assert_eq!(formatted_short, "in 1 minute");
/// ```
///
/// The function will return the appropriate formatted string based on the duration passed in.

fn format_duration(duration: Duration) -> String {
    let days = duration.num_days();
    let hours = duration.num_hours() % 24;
    let minutes = duration.num_minutes() % 60;

    if days > 0 {
        if hours > 0 {
            if hours == 1 {
                return format!("in 1 day and 1 hour");
            } else {
                return format!("in 1 day and {} hours", hours);
            }
        } else {
            return format!("in 1 day");
        }
    } else if hours > 0 {
        if hours == 1 {
            return format!("in 1 hour");
        } else {
            return format!("in {} hours", hours);
        }
    } else if minutes > 0 {
        if minutes == 1 {
            return format!("in 1 minute");
        } else {
            return format!("in {} minutes", minutes);
        }
    } else {
        return "less than a minute".to_string();
    }
}

/// ⚙️ **Function**: Creates an embed displaying the list of followed summoners.
///
/// This function constructs a Discord embed message that lists all summoners being followed in a guild.
/// It includes the remaining time for each summoner's follow or a message if no summoners are currently being tracked.
/// The embed has a default purple color and includes a footer stating that the message will be deleted after 60 seconds.
///
/// # Parameters:
/// - `data`: A `serde_json::Value` object containing the list of tracked summoners.
///   The `data` is expected to have a `tracked_summoners` field, which is an array of objects with each summoner's name and follow duration.
///
/// # Returns:
/// - `CreateReply`: A Discord reply object containing the constructed embed. This can be sent to a Discord channel.
///   The embed includes fields with each summoner's name and the remaining follow time, or a message stating that no summoners are currently being followed.
///
/// # ⚠️ Notes:
/// - If no summoners are found in the `tracked_summoners` array, the embed will display "No summoners are currently being followed".
/// - The embed's color is set to purple (`0xA020F0`), and a footer is included indicating that the message will be deleted after 60 seconds.
/// - Each summoner's follow information is displayed in the format: `Follow ends in: X time`.
///
/// # Example:
/// ```rust
/// let data = json!({
///     "tracked_summoners": [
///         {
///             "name": "Summoner1",
///             "time_remaining": "2 hours 15 minutes"
///         },
///         {
///             "name": "Summoner2",
///             "time_remaining": "Follow ended"
///         }
///     ]
/// });
/// let embed_reply = create_embed_followed_summoner(data);
/// ctx.send(embed_reply).await?;
/// ```
///
/// This example would produce an embed listing two summoners, with their remaining follow durations.
pub fn create_embed_followed_summoner(data: Value) -> CreateReply {
    let binding = vec![];
    let tracked_summoners = data["tracked_summoners"].as_array().unwrap_or(&binding);
    let mut embed = CreateEmbed::new()
        .title("Tracked Summoners")
        .color(0xA020F0)
        .footer(CreateEmbedFooter::new(
            "This message will be deleted in 60 seconds.",
        ))
        .thumbnail("https://i.postimg.cc/9fKf2tYp/Logo.png");

    if tracked_summoners.is_empty() {
        embed = embed.field(
            "",
            "No summoners are currently being followed".to_string(),
            false,
        );
        return CreateReply {
            embeds: vec![embed],
            ..Default::default()
        };
    }
    for summoner in tracked_summoners {
        let name = summoner["name"].as_str().unwrap_or("Unknown");
        let time_remaining = summoner["time_remaining"].as_str().unwrap_or("Unknown");

        embed = embed.field(name, format!("Follow ends in: {}", time_remaining), false);
    }

    CreateReply {
        embeds: vec![embed],
        ..Default::default()
    }
}
