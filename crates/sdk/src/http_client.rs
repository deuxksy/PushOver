use crate::{Message, PushOverError, Response};
use reqwest::Client;
use std::borrow::Cow;

const PUSHOVER_API_BASE: &str = "https://api.pushover.net";

pub struct PushOverClient {
    client: Client,
    user_key: String,
    api_token: String,
    base_url: String,
}

impl PushOverClient {
    pub fn new(user_key: String, api_token: String) -> Self {
        Self {
            client: Client::new(),
            user_key,
            api_token,
            base_url: PUSHOVER_API_BASE.to_string(),
        }
    }

    /// 테스트를 위해 custom base URL을 사용하는 클라이언트 생성
    pub fn with_base_url(user_key: String, api_token: String, base_url: String) -> Self {
        Self {
            client: Client::new(),
            user_key,
            api_token,
            base_url,
        }
    }

    pub async fn send(&self, msg: Message) -> Result<Response, PushOverError> {
        let mut form: Vec<(Cow<str>, Cow<str>)> = vec![
            ("user".into(), self.user_key.as_str().into()),
            ("token".into(), self.api_token.as_str().into()),
            ("message".into(), msg.message.as_str().into()),
        ];

        if let Some(title) = &msg.title {
            form.push(("title".into(), title.as_str().into()));
        }
        if let Some(priority) = msg.priority {
            form.push(("priority".into(), priority.to_string().into()));
        }
        if let Some(sound) = &msg.sound {
            form.push(("sound".into(), sound.as_str().into()));
        }
        if let Some(device) = &msg.device {
            form.push(("device".into(), device.as_str().into()));
        }
        if let Some(url) = &msg.url {
            form.push(("url".into(), url.as_str().into()));
        }
        if let Some(url_title) = &msg.url_title {
            form.push(("url_title".into(), url_title.as_str().into()));
        }
        if let Some(html) = msg.html {
            form.push(("html".into(), if html { "1" } else { "0" }.into()));
        }
        if let Some(timestamp) = msg.timestamp {
            form.push(("timestamp".into(), timestamp.to_string().into()));
        }

        // Priority arguments
        if let Some(priority_arg) = &msg.priority_arg {
            if let Some(expire) = priority_arg.expire {
                form.push(("expire".into(), expire.to_string().into()));
            }
            if let Some(retry) = priority_arg.retry {
                form.push(("retry".into(), retry.to_string().into()));
            }
        }

        let url = format!("{}/1/messages.json", self.base_url);
        let resp = self.client.post(&url).form(&form).send().await?;

        let status = resp.status().as_u16();
        let body = resp.text().await?;

        if status == 200 {
            Ok(serde_json::from_str(&body)?)
        } else {
            Err(PushOverError::ApiError {
                status,
                message: body,
            })
        }
    }
}
