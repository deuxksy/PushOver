use anyhow::Result;
use serde::Deserialize;
use crate::config::Config;

#[derive(Debug, Deserialize)]
struct HistoryResponse {
    status: String,
    messages: Vec<HistoryMessage>,
}

#[derive(Debug, Deserialize)]
struct HistoryMessage {
    id: String,
    message: String,
    title: Option<String>,
    status: String,
    receipt: Option<String>,
    sent_at: Option<String>,
    created_at: String,
}

pub async fn execute(
    limit: Option<usize>,
    _user: Option<String>,
    _token: Option<String>,
) -> Result<()> {
    let config = Config::load()?;
    let profile = config.get_default_profile()
        .ok_or_else(|| anyhow::anyhow!("No default profile configured"))?;

    let worker_url = profile.api_endpoint.as_deref()
        .ok_or_else(|| anyhow::anyhow!("No api_endpoint configured in profile"))?;

    let limit = limit.unwrap_or(10);
    let url = format!("{}/api/v1/messages?limit={}", worker_url.trim_end_matches('/'), limit);

    let client = reqwest::Client::new();
    let response = client.get(&url)
        .header("Authorization", format!("Bearer {}", profile.api_token))
        .header("Content-Type", "application/json")
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("API error ({}): {}", status, body);
    }

    let data: HistoryResponse = response.json().await?;

    println!("Message History (last {}):\n", limit);
    println!("{:<12} {:<20} {:<12} Message", "ID", "Created", "Status");
    println!("{}", "-".repeat(75));

    for msg in &data.messages {
        let id_short = if msg.id.len() > 8 {
            &msg.id[..8]
        } else {
            &msg.id
        };
        let time = msg.sent_at.as_deref().unwrap_or(&msg.created_at);
        let display = match &msg.title {
            Some(t) => format!("{}: {}", t, msg.message),
            None => msg.message.clone(),
        };
        let truncated = if display.len() > 40 {
            format!("{}...", &display[..37])
        } else {
            display
        };
        println!("{:<12} {:<20} {:<12} {}", id_short, time, msg.status, truncated);
    }

    if data.messages.is_empty() {
        println!("No messages found.");
    }

    Ok(())
}
