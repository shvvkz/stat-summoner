use poise::Modal;
use serde::{Deserialize, Serialize};
use mongodb::Client;
use serde_json::Value;

pub struct Data {
    pub riot_api_key: String,
    pub mongo_client: Client,
    pub dd_json: Value,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;

#[derive(Serialize, Deserialize, Debug)]
pub struct SummonerFollowedData {
    pub puuid: String,
    pub summoner_id: String,
    pub name: String,
    pub tag: String,
    pub region: String,
    pub last_match_id: String,
    pub time_end_follow: String,
}

#[derive(Debug, poise::ChoiceParameter)]
pub enum Region {
    EUW,
    NA,
    KR,
    EUNE,
    BR,
    LAN,
    LAS,
    OCE,
    RU,
    TR,
    JP,
}

#[derive(Debug, Modal)]
#[name = "Enter your League of Legends Stats Info"]
pub struct LolStatsModal {
    #[name = "Game Name"]
    #[placeholder = "Enter your game name (e.g., Faker)"]
    pub game_name: String,

    #[name = "Tag Line"]
    #[placeholder = "Enter your tag line (e.g., 1234)"]
    pub tag_line: String,
}

#[derive(Debug, Modal)]
#[name = "Enter the summoner info"]
pub struct FollowGamesModal {
    #[name = "Game Name"]
    #[placeholder = "Enter the game name (e.g., Faker)"]
    pub game_name: String,

    #[name = "Tag Line"]
    #[placeholder = "Enter the tag line (e.g., 1234)"]
    pub tag_line: String,

    #[name = "Time Followed (in hours)"]
    #[placeholder = "Enter the number of hours (e.g., 2)"]
    pub time_followed: String,
}


pub const QUEUE_ID_MAP: [(i64, &str); 10] = [
    (400, "Normal Draft"),
    (420, "Ranked Solo/Duo"),
    (430, "Normal Blind"),
    (440, "Ranked Flex"),
    (450, "ARAM"),
    (700, "Clash"),
    (830, "Co-op vs AI Intro"),
    (840, "Co-op vs AI Beginner"),
    (850, "Co-op vs AI Intermediate"),
    (900, "URF"),
];
