use mongodb::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
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
pub struct EmojiId {
    pub role: String,
    pub name: String,
    pub id_emoji: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChampionData {
    pub name: String,
    pub id_name: String,
    pub role: Vec<String>,
    pub popularity: String,
    pub winrate: String,
    pub banrate: String,
    pub runes: RunesData,
    pub core_build: CoreBuildData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunesData {
    pub parent_primary_rune: String,
    pub child_primary_rune_1: String,
    pub child_primary_rune_2: String,
    pub child_primary_rune_3: String,
    pub child_secondary_rune_1: String,
    pub child_secondary_rune_2: String,
    pub tertiary_rune_1: String,
    pub tertiary_rune_2: String,
    pub tertiary_rune_3: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CoreBuildData {
    pub first: String,
    pub second: String,
    pub third: String,
}
