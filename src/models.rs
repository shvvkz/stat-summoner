use poise::Modal;

pub struct Data {
    pub riot_api_key: String,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;

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

/// Modal pour entrer le nom du joueur et le tag
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
