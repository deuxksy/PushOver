use anyhow::Result;
use clap::Parser;
use crate::config::Config;

#[derive(Debug, Parser)]
pub struct RegisterOptions {
    /// Worker API token to register
    #[arg(short = 't', long)]
    pub token: String,

    /// PushOver user key
    #[arg(short = 'u', long)]
    pub user_key: String,

    /// Token name/description (optional)
    #[arg(short = 'n', long)]
    pub name: Option<String>,
}

pub async fn execute(options: RegisterOptions) -> Result<()> {
    let config = Config::load()?;
    let profile = config.get_default_profile()
        .ok_or_else(|| anyhow::anyhow!("No default profile found"))?;

    let worker_url = profile.api_endpoint.as_deref()
        .ok_or_else(|| anyhow::anyhow!("No api_endpoint configured in profile"))?;

    println!("Registering token with PushOver Worker...");
    println!("Worker URL: {}", worker_url);

    // Call Worker API to register token
    let client = reqwest::Client::new();
    let url = format!("{}/api/v1/tokens/register", worker_url.trim_end_matches('/'));

    let mut body = serde_json::Map::new();
    body.insert("token".into(), serde_json::Value::String(options.token.clone()));
    body.insert("user_key".into(), serde_json::Value::String(options.user_key.clone()));

    if let Some(name) = &options.name {
        body.insert("name".into(), serde_json::Value::String(name.clone()));
    }

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;
;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        anyhow::bail!("Registration failed ({}): {}", status, text);
    }

    let data: serde_json::Value = response.json().await?;

    if data.get("status").and_then(|v| v.as_str()) == Some("success") {
        println!("\n✓ Token registered successfully!");
        println!("  Token: {}...", &options.token[..8]);
        println!("  User Key: {}...", &options.user_key[..8]);
        if let Some(name) = &options.name {
            println!("  Name: {}", name);
        }
        println!("\nYou can now use this token in the dashboard Settings > Worker tab.");
    } else {
        anyhow::bail!("Unexpected response: {}", data);
    }

    Ok(())
}
