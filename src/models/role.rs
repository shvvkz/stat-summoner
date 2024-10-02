#[derive(Debug, poise::ChoiceParameter)]
pub enum Role {
    TOPLANE,
    JUNGLE,
    MIDLANE,
    ADC,
    SUPPORT,
}
