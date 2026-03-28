use anyhow::Result;
use serde::Deserialize;

#[derive(Debug)]
pub struct SendOptions {
    pub message: String,
    pub title: Option<String>,
    pub user: Option<String>,
    pub token: Option<String>,
    pub device: Option<String>,
    pub priority: Option<i8>,
    pub url: Option<String>,
    pub url_title: Option<String>,
    pub sound: Option<String>,
    pub timestamp: Option<i64>,
    pub html: bool,
}

#[derive(Debug, Deserialize)]
struct SendResponse {
    status: String,
    request: Option<String>,
    receipt: Option<String>,
}

pub async fn execute(options: SendOptions) -> Result<()> {
    let config = crate::config::Config::load()?;
    let profile = config.get_default_profile()
        .ok_or_else(|| anyhow::anyhow!("No default profile found"))?;

    // Worker API credentials
    let worker_url = profile.api_endpoint.as_deref()
        .ok_or_else(|| anyhow::anyhow!("No api_endpoint configured in profile"))?;
    let worker_token = profile.worker_token.as_deref()
        .or_else(|| profile.api_token.as_deref())
        .ok_or_else(|| anyhow::anyhow!("No worker_token configured in profile"))?;

    // PushOver credentials (sent in body for Worker to forward)
    let user_key = options.user.unwrap_or_else(|| profile.user_key.clone());
    let pushover_token = options.token.clone()
        .or_else(|| profile.pushover_token.clone())
        .or_else(|| profile.api_token.clone())
        .ok_or_else(|| anyhow::anyhow!("No pushover_token configured in profile"))?;

    let mut body = serde_json::Map::new();
    body.insert("user".into(), serde_json::Value::String(user_key));
    body.insert("token".into(), serde_json::Value::String(pushover_token));
    body.insert("message".into(), serde_json::Value::String(options.message));

    if let Some(v) = options.title { body.insert("title".into(), serde_json::Value::String(v)); }
    if let Some(v) = options.device { body.insert("device".into(), serde_json::Value::String(v)); }
    if let Some(v) = options.sound { body.insert("sound".into(), serde_json::Value::String(v)); }
    if let Some(v) = options.url { body.insert("url".into(), serde_json::Value::String(v)); }
    if let Some(v) = options.url_title { body.insert("url_title".into(), serde_json::Value::String(v)); }
    if let Some(p) = options.priority {
        body.insert("priority".into(), serde_json::Value::Number(p.into()));
        if p == 2 {
            body.insert("retry".into(), serde_json::Value::Number(300.into()));
            body.insert("expire".into(), serde_json::Value::Number(3600.into()));
        }
    }
    if options.html {
        body.insert("html".into(), serde_json::Value::Number(1.into()));
    }
    if let Some(ts) = options.timestamp {
        body.insert("timestamp".into(), serde_json::Value::Number(ts.into()));
    }

    let url = format!("{}/api/v1/messages", worker_url.trim_end_matches('/'));
    let client = reqwest::Client::new();
    let response = client.post(&url)
        .header("Authorization", format!("Bearer {}", worker_token))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        anyhow::bail!("Worker API error ({}): {}", status, text);
    }

    let data: SendResponse = response.json().await?;

    println!("Message sent successfully!");
    println!("Status: {}", data.status);
    if let Some(receipt) = data.receipt {
        println!("Receipt: {}", receipt);
    }

    Ok(())
}
