use poise::Modal;

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

#[derive(Debug, Modal)]
#[name = "Enter the summoner info"]
pub struct ChampionsInfosModal {
    #[name = "Champion Name"]
    #[placeholder = "Enter the champion name (e.g., Jinx)"]
    pub champion_name: String,
}
