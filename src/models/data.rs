use mongodb::Client;
use serde_json::Value;
use serde::{Deserialize, Serialize};
pub struct Data {
    pub riot_api_key: String,
    pub mongo_client: Client,
    pub dd_json: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SummonerFollowedData {
    pub puuid: String,
    pub summoner_id: String,
    pub name: String,
    pub tag: String,
    pub region: String,
    pub last_match_id: String,
    pub time_end_follow: String,
    pub channel_id: u64,
    pub guild_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmojiId{
    pub role: String,
    pub name: String,
    pub id_emoji: String
}