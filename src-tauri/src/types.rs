use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardType {
    Meeting,
    Mr,
    Thread,
    Task,
    Review,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardStatus {
    Planned,
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Impact {
    Low,
    Mid,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    Manual,
    Calendar,
    Gitlab,
    Linear,
    Slack,
    Notion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: i64,
    pub title: String,
    pub card_type: CardType,
    pub status: CardStatus,
    pub impact: Option<Impact>,
    pub time_estimate: Option<f64>,
    pub url: Option<String>,
    pub week_id: Option<i64>,
    pub day_of_week: Option<i64>,
    pub position: i64,
    pub source: Source,
    pub external_id: Option<String>,
    pub notes: Option<String>,
    pub metadata: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Week {
    pub id: i64,
    pub year: i64,
    pub week_number: i64,
    pub start_date: String,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub id: i64,
    pub available_hours: f64,
    pub ai_provider: Option<String>,
    pub auto_ai: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Integration {
    pub id: String,
    pub enabled: bool,
    pub config: Option<String>,
    pub last_synced_at: Option<String>,
}
