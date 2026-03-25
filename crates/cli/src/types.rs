use chrono::{DateTime, Utc};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MessageRecord {
    pub id: String,
    pub message: String,
    pub title: Option<String>,
    pub status: String,
    pub sent_at: DateTime<Utc>,
}
