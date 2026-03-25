use anyhow::Result;
use pushover_sdk::{Message, PushOverClient};
use std::env;

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

pub async fn execute(options: SendOptions) -> Result<()> {
    // Load config and get credentials
    let config = crate::config::Config::load()?;
    let profile = config.get_default_profile()
        .ok_or_else(|| anyhow::anyhow!("No default profile found"))?;

    let user_key = options.user.unwrap_or_else(|| profile.user_key.clone());
    let api_token = options.token.unwrap_or_else(|| profile.api_token.clone());

    // Environment variable fallback
    let user_key = env::var("PUSHOVER_USER_KEY").unwrap_or(user_key);
    let api_token = env::var("PUSHOVER_API_TOKEN").unwrap_or(api_token);

    let client = PushOverClient::new(user_key, api_token);

    let mut msg = Message {
        message: options.message,
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

    msg.title = options.title;
    msg.device = options.device;
    msg.sound = options.sound;
    msg.url = options.url;
    msg.url_title = options.url_title;

    if let Some(p) = options.priority {
        msg.priority = Some(p as i32);
        if p == 2 {
            msg.priority_arg = Some(pushover_sdk::PriorityArgs {
                retry: Some(300),
                expire: Some(3600),
            });
        }
    }

    if options.html {
        msg.html = Some(true);
    }

    if let Some(ts) = options.timestamp {
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
