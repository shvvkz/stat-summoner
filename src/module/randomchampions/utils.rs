use futures::TryStreamExt;
use mongodb::bson::doc;
use rand::Rng;

use crate::models::{
    data::{ChampionData, Data},
    error::Error,
    role::Role,
};

/// ⚙️ Maps a `Role` enum value to its corresponding string representation as stored in the database.
///
/// This function takes a `Role` enum and converts it to a `String` that matches the format used in the database.
/// It is used to ensure consistency between the role representation in the code and the role names stored in the database.
///
/// # Parameters:
/// - `role`: A `Role` enum representing the role to be converted.
///
/// # Returns:
/// - `String`: The corresponding string representation of the role (e.g., "Top", "Jungler").
///
/// # Example:
/// ```rust
/// let role = Role::TOPLANE;
/// let role_str = match_role_with_database_roles(role);
/// assert_eq!(role_str, "Top");
/// ```
///
/// # ⚠️ Notes:
/// - The function uses Rust's `match` control flow to convert each `Role` variant to the corresponding string.
/// - This function is essential for ensuring compatibility between different parts of the code and the data storage layer.
///
/// # Related Enums:
/// - `Role`: Represents different League of Legends roles like `TOPLANE`, `JUNGLE`, `MIDLANE`, etc.
///
/// # See Also:
/// - `get_champions_by_role`: Uses the string representation of a role to query the database for champions with that role.
fn match_role_with_database_roles(role: Role) -> String {
    match role {
        Role::TOPLANE => "Top".to_string(),
        Role::JUNGLE => "Jungler".to_string(),
        Role::MIDLANE => "Mid".to_string(),
        Role::ADC => "AD Carry".to_string(),
        Role::SUPPORT => "Support".to_string(),
    }
}

/// ⚙️ Retrieves all champions that match the specified role from the database.
///
/// This asynchronous function queries a MongoDB collection to find all champions that include the specified role.
/// The function returns a vector of `ChampionData` objects that match the role provided.
///
/// # Parameters:
/// - `role`: A reference to a string representing the role to filter champions by (e.g., "Top", "Jungler").
/// - `collection`: A reference to a MongoDB collection of `ChampionData` representing the champion data stored in the database.
///
/// # Returns:
/// - `Result<Vec<ChampionData>, mongodb::error::Error>`: On success, returns a vector of `ChampionData` objects that match the role.
///   On failure, returns an `Error` from MongoDB indicating what went wrong.
///
/// # Example:
/// ```rust
/// let role = "AD Carry";
/// let champions = get_champions_by_role(role, &collection).await?;
/// for champion in champions {
///     println!("Champion: {}", champion.name);
/// }
/// ```
///
/// # ⚠️ Notes:
/// - The function uses MongoDB's `$in` operator to search for champions whose roles include the specified value.
/// - The result is collected into a vector using the `try_collect` method for convenience.
///
/// # Related Functions:
/// - `match_role_with_database_roles`: Converts a `Role` enum into the corresponding string representation, which can be used as input for this function.
///
/// # Dependencies:
/// - This function depends on a MongoDB collection that stores `ChampionData` documents.
/// - Requires the `futures` crate for the `try_collect` method to handle the cursor results asynchronously.
async fn get_champions_by_role(
    role: &str,
    collection: &mongodb::Collection<ChampionData>,
) -> mongodb::error::Result<Vec<ChampionData>> {
    let filter = doc! {
        "role": {
            "$in": [role]
        }
    };
    let cursor = collection.find(filter).await?;

    cursor.try_collect().await
}

/// ⚙️ Retrieves all champions from the database without filtering by role.
///
/// This asynchronous function queries a MongoDB collection to retrieve all `ChampionData` documents, regardless of their role.
/// It is used to obtain a complete list of champions stored in the collection.
///
/// # Parameters:
/// - `collection`: A reference to a MongoDB collection of `ChampionData` representing the champion data stored in the database.
///
/// # Returns:
/// - `Result<Vec<ChampionData>, mongodb::error::Error>`: On success, returns a vector of all `ChampionData` objects in the collection.
///   On failure, returns an `Error` from MongoDB detailing the issue.
///
/// # Example:
/// ```rust
/// let champions = get_champions_with_no_role(&collection).await?;
/// for champion in champions {
///     println!("Champion: {}", champion.name);
/// }
/// ```
///
/// # ⚠️ Notes:
/// - The function uses an empty filter (`{}`) to retrieve all documents in the `ChampionData` collection.
/// - The result is collected into a vector using the `try_collect` method, which allows for asynchronous processing of the cursor results.
///
/// # Related Functions:
/// - `get_champions_by_role`: Retrieves champions filtered by a specific role from the collection.
///
/// # Dependencies:
/// - This function depends on a MongoDB collection that stores `ChampionData` documents.
/// - Requires the `futures` crate for the `try_collect` method to handle the cursor results asynchronously.
async fn get_champions_with_no_role(
    collection: &mongodb::Collection<ChampionData>,
) -> mongodb::error::Result<Vec<ChampionData>> {
    let filter = doc! {};
    let cursor = collection.find(filter).await?;

    cursor.try_collect().await
}

/// ⚙️ Selects a random champion from a list of `ChampionData`.
///
/// This function takes a vector of `ChampionData` and returns a random champion from that list.
/// It uses a random number generator to choose an index, ensuring that each champion has an equal chance of being selected.
///
/// # Parameters:
/// - `champions`: A `Vec<ChampionData>` containing the list of champions from which a random champion will be selected.
///
/// # Returns:
/// - `ChampionData`: A clone of the randomly selected `ChampionData` object.
///
/// # Example:
/// ```rust
/// let champions_list = vec![champion1, champion2, champion3];
/// let random_champion = get_random_champion(champions_list);
/// println!("Selected Champion: {}", random_champion.name);
/// ```
///
/// # ⚠️ Notes:
/// - The function uses the `rand` crate to generate a random index, and selects the champion at that index.
/// - The selected champion is cloned before being returned, ensuring the original vector remains unmodified.
///
/// # Related Functions:
/// - `get_list_champions`: Retrieves a list of champions that can be passed to this function to select a random one.
///
/// # Dependencies:
/// - Requires the `rand` crate for generating a random index.
pub fn get_random_champion(champions: Vec<ChampionData>) -> ChampionData {
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..champions.len());
    let champion = &champions[index];
    champion.clone()
}

/// ⚙️ Retrieves a list of champions from the database, optionally filtered by role.
///
/// This asynchronous function queries the MongoDB collection of champions to retrieve either all champions or those matching a specific role.
/// If a role is provided, the function will filter the list accordingly. If no role is specified, all champions will be retrieved.
///
/// # Parameters:
/// - `ctx`: The command context, providing access to the bot's data and other utilities.
/// - `role`: An optional `Role` enum to filter the champions by role. If `None`, all champions are retrieved.
///
/// # Returns:
/// - `Result<Vec<ChampionData>, Error>`: On success, returns a vector of `ChampionData` objects representing the champions that match the criteria.
///   On failure, returns an `Error` indicating what went wrong during the database query.
///
/// # Example:
/// ```rust
/// let champions_list = get_list_champions(ctx, Some(Role::MIDLANE)).await?;
/// for champion in champions_list {
///     println!("Champion: {}", champion.name);
/// }
/// ```
///
/// # ⚠️ Notes:
/// - If `role` is `None`, the function calls `get_champions_with_no_role` to retrieve all champions.
/// - If `role` is provided, `match_role_with_database_roles` is used to convert the `Role` enum to a string, and `get_champions_by_role` is called to perform the filtered search.
/// - This function interacts with MongoDB asynchronously and relies on other helper functions for specific queries.
///
/// # Related Functions:
/// - `get_champions_with_no_role`: Retrieves all champions without filtering by role.
/// - `get_champions_by_role`: Retrieves champions filtered by a specific role from the collection.
/// - `match_role_with_database_roles`: Converts a `Role` enum into the corresponding string representation.
///
/// # Dependencies:
/// - Requires access to a MongoDB collection (`champions_data`) for champion information.
/// - The function depends on other helper functions for querying the MongoDB collection.
pub async fn get_list_champions(
    ctx: poise::ApplicationContext<'_, Data, Error>,
    role: Option<Role>,
) -> Result<Vec<ChampionData>, Error> {
    let mongo_client = &ctx.data().mongo_client;
    let collection = mongo_client
        .database("stat-summoner")
        .collection::<ChampionData>("champions_data");
    if role.is_none() {
        let champions = get_champions_with_no_role(&collection).await?;
        return Ok(champions);
    } else {
        let role = match_role_with_database_roles(role.unwrap());
        let champions = get_champions_by_role(&role, &collection).await?;
        return Ok(champions);
    }
}
