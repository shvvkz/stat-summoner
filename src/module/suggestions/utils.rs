use crate::models::data::BlackList;
use chrono::Utc;
use mongodb::bson::doc;
use mongodb::Collection;

/// ⚙️ Checks if a user is blacklisted.
///
/// This asynchronous function queries a MongoDB collection to check whether a user with the specified `user_id` exists in the blacklist.
/// It returns `true` if the user is found, and `false` otherwise.
///
/// # Parameters:
/// - `collection`: A reference to a MongoDB collection of `BlackList` documents.
/// - `user_id`: A string slice representing the ID of the user to check.
///
/// # Returns:
/// - `Result<bool, mongodb::error::Error>`: On success, returns `true` if the user is blacklisted, `false` otherwise.
///   On failure, returns an `Error` from MongoDB detailing the issue.
///
/// # Example:
/// ```rust
/// let is_blacklisted = is_user_blacklisted(&collection, "123456789012345678").await?;
/// if is_blacklisted {
///     println!("User is blacklisted.");
/// } else {
///     println!("User is not blacklisted.");
/// }
/// ```
///
/// # ⚠️ Notes:
/// - The function uses MongoDB's `find_one` method to search for a document with a matching `user_id`.
/// - If no matching document is found, the function returns `false`.
///
/// # Related Structures:
/// - `BlackList`: Represents the structure of a blacklisted user in the database.
///
/// # Dependencies:
/// - Requires a MongoDB collection containing `BlackList` documents.
pub async fn is_user_blacklisted(
    collection: &Collection<BlackList>,
    user_id: &str,
) -> mongodb::error::Result<bool> {
    collection
        .find_one(doc! { "user_id": user_id })
        .await
        .map(|opt| opt.is_some())
}

/// ⚙️ Adds a user to the blacklist.
///
/// This asynchronous function inserts a new document into a MongoDB collection to blacklist a user.
/// The document includes the `user_id`, `username`, and a timestamp indicating when the user was added to the blacklist.
///
/// # Parameters:
/// - `collection`: A reference to a MongoDB collection of `BlackList` documents.
/// - `user_id`: A string slice representing the ID of the user to blacklist.
/// - `username`: A string slice representing the username of the user to blacklist.
///
/// # Returns:
/// - `Result<(), mongodb::error::Error>`: Returns `Ok(())` if the user is successfully blacklisted.
///   On failure, returns an `Error` from MongoDB detailing the issue.
///
/// # Example:
/// ```rust
/// add_user_to_blacklist(&collection, "123456789012345678", "JohnDoe").await?;
/// println!("User has been blacklisted.");
/// ```
///
/// # ⚠️ Notes:
/// - The function creates a new `BlackList` entry with the provided `user_id` and `username`.
/// - The `created_at` field is set to the current timestamp in seconds since the UNIX epoch.
///
/// # Related Structures:
/// - `BlackList`: Represents the structure of a blacklisted user in the database.
///
/// # Dependencies:
/// - Requires a MongoDB collection containing `BlackList` documents.
/// - Uses the `chrono` crate to generate a timestamp for the `created_at` field.
pub async fn add_user_to_blacklist(
    collection: &Collection<BlackList>,
    user_id: &str,
    username: &str,
) -> mongodb::error::Result<()> {
    let blacklist_entry = BlackList {
        user_id: user_id.to_string(),
        username: username.to_string(),
        created_at: Utc::now().timestamp() as u64,
    };

    collection.insert_one(blacklist_entry).await?;
    Ok(())
}
