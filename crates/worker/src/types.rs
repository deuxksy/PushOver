use serde::{Deserialize, Serialize};
use worker::js_sys::Date;
use pushover_sdk::Message;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookMessage {
    pub id: String,
    pub uuid: String,
    pub message: String,
    pub title: Option<String>,
    pub device: Option<String>,
    pub priority: i32,
    pub sound: Option<String>,
    pub url: Option<String>,
    pub url_title: Option<String>,
    pub timestamp: i64,
    pub html: bool,
    pub received_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub status: u16,
}

impl ErrorResponse {
    pub fn new(error: impl Into<String>, message: impl Into<String>, status: u16) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
            status,
        }
    }

    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::new("bad_request", msg, 400)
    }

    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::new("unauthorized", msg, 401)
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::new("not_found", msg, 404)
    }

    pub fn internal_error(msg: impl Into<String>) -> Self {
        Self::new("internal_error", msg, 500)
    }
}

pub fn current_timestamp() -> u64 {
    (Date::now() / 1000.0) as u64
}

pub fn iso_timestamp() -> String {
    let date = Date::new(&Date::now().into());
    date.to_iso_string().into()
}

impl From<Message> for WebhookMessage {
    fn from(msg: Message) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            uuid: uuid::Uuid::new_v4().to_string(),
            message: msg.message,
            title: msg.title,
            device: msg.device,
            priority: msg.priority.unwrap_or(0),
            sound: msg.sound,
            url: msg.url,
            url_title: msg.url_title,
            timestamp: msg.timestamp.unwrap_or_else(|| current_timestamp()) as i64,
            html: msg.html.unwrap_or(false),
            received_at: iso_timestamp(),
        }
    }
}
