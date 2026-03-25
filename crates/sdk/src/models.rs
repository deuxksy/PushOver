use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sound: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority_arg: Option<PriorityArgs>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityArgs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expire: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub status: i32,
    pub request: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receipt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub id: String,
    pub checksum: String,
    pub message: i32,
    pub title: String,
    pub message_title: String,
    pub message_timestamp: i64,
    pub message_html: Option<String>,
    pub message_url: Option<String>,
    pub message_url_title: Option<String>,
    pub priority: i32,
    pub sound: String,
    pub device: String,
    pub userid: String,
    pub pushed: i64,
    pub receipt: Option<String>,
    pub emergency: i32,
}
