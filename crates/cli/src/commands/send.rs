use anyhow::Result;
use pushover_sdk::{Message, PushOverClient};
use std::env;

pub async fn execute(
    message: String,
    title: Option<String>,
    user: Option<String>,
    token: Option<String>,
    device: Option<String>,
    priority: Option<i8>,
    url: Option<String>,
    url_title: Option<String>,
    sound: Option<String>,
    timestamp: Option<i64>,
    html: bool,
) -> Result<()> {
    // Load config and get credentials
    let config = crate::config::Config::load()?;
    let profile = config.get_default_profile()
        .ok_or_else(|| anyhow::anyhow!("No default profile found"))?;

    let user_key = user.unwrap_or_else(|| profile.user_key.clone());
    let api_token = token.unwrap_or_else(|| profile.api_token.clone());

    // Environment variable fallback
    let user_key = env::var("PUSHOVER_USER_KEY").unwrap_or(user_key);
    let api_token = env::var("PUSHOVER_API_TOKEN").unwrap_or(api_token);

    let client = PushOverClient::new(user_key, api_token);

    let mut msg = Message {
        message,
        title: None,
        priority: None,
        priority_arg: None,
        sound: None,
        device: None,
        url: None,
        url_title: None,
        html: None,
        timestamp: None,
    };

    msg.title = title;
    msg.device = device;
    msg.sound = sound;
    msg.url = url;
    msg.url_title = url_title;

    if let Some(p) = priority {
        msg.priority = Some(p as i32);
        if p == 2 {
            msg.priority_arg = Some(pushover_sdk::PriorityArgs {
                retry: Some(300),
                expire: Some(3600),
            });
        }
    }

    if html {
        msg.html = Some(true);
    }

    if let Some(ts) = timestamp {
        msg.timestamp = Some(ts as u64);
    }

    let response = client.send(msg).await?;

    println!("Message sent successfully!");
    println!("Status: {}", response.status);
    if let Some(receipt) = response.receipt {
        println!("Receipt: {}", receipt);
    }

    Ok(())
}
