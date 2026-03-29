use worker::*;
use wasm_bindgen::JsValue;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushOverResponse {
    pub status: i32,
    pub request: String,
    #[serde(default)]
    pub receipt: Option<String>,
    #[serde(default)]
    pub errors: Option<Vec<String>>,
}

impl PushOverResponse {
    pub fn is_success(&self) -> bool {
        self.status == 1
    }
}

pub struct PushOverClient {
    api_url: String,
}

impl PushOverClient {
    pub fn new(api_url: &str) -> Self {
        Self {
            api_url: api_url.to_string(),
        }
    }

    pub fn from_env(env: &Env) -> Result<Self> {
        let api_url = env.var("PUSHOVER_API_URL")
            .map(|v| v.to_string())
            .unwrap_or_else(|_| "https://api.pushover.net".to_string());
        Ok(Self::new(&api_url))
    }

    /// Send a message via PushOver API using form-encoded POST.
    pub async fn send_message(
        &self,
        token: &str,
        user: &str,
        message: &str,
        title: Option<&str>,
        priority: Option<i32>,
        sound: Option<&str>,
        device: Option<&str>,
        url: Option<&str>,
        url_title: Option<&str>,
        html: Option<bool>,
        retry: Option<u32>,
        expire: Option<u32>,
        attachment_base64: Option<&str>,
        attachment_type: Option<&str>,
    ) -> Result<PushOverResponse> {
        // Build form-encoded body
        let mut params: Vec<(String, String)> = vec![
            ("token".to_string(), token.to_string()),
            ("user".to_string(), user.to_string()),
            ("message".to_string(), message.to_string()),
        ];

        if let Some(title) = title {
            params.push(("title".to_string(), title.to_string()));
        }
        if let Some(priority) = priority {
            params.push(("priority".to_string(), priority.to_string()));
        }
        if let Some(sound) = sound {
            params.push(("sound".to_string(), sound.to_string()));
        }
        if let Some(device) = device {
            params.push(("device".to_string(), device.to_string()));
        }
        if let Some(url) = url {
            params.push(("url".to_string(), url.to_string()));
        }
        if let Some(url_title) = url_title {
            params.push(("url_title".to_string(), url_title.to_string()));
        }
        if let Some(html) = html {
            params.push(("html".to_string(), if html { "1".to_string() } else { "0".to_string() }));
        }
        if let Some(retry) = retry {
            params.push(("retry".to_string(), retry.to_string()));
        }
        if let Some(expire) = expire {
            params.push(("expire".to_string(), expire.to_string()));
        }
        if let Some(attachment_base64) = attachment_base64 {
            params.push(("attachment_base64".to_string(), attachment_base64.to_string()));
        }
        if let Some(attachment_type) = attachment_type {
            params.push(("attachment_type".to_string(), attachment_type.to_string()));
        }

        let body = form_urlencode(&params);

        // Create request
        let url = format!("{}/1/messages.json", self.api_url);
        let headers = Headers::new();
        headers.set("Content-Type", "application/x-www-form-urlencoded")?;

        let mut init = RequestInit::new();
        init.with_method(Method::Post);
        init.with_headers(headers);
        init.with_body(Some(JsValue::from_str(&body)));

        let request = Request::new_with_init(&url, &init)?;
        let mut response = Fetch::Request(request).send().await?;

        let status = response.status_code();
        let result: PushOverResponse = response.json().await?;

        if !result.is_success() {
            let errors = result.errors.clone().unwrap_or_default().join(", ");
            return Err(Error::from(format!(
                "PushOver API error ({}): {}", status, errors
            )));
        }

        Ok(result)
    }
}

/// Simple form URL encoding. Encodes keys and values.
fn form_urlencode(params: &[(String, String)]) -> String {
    params
        .iter()
        .map(|(k, v)| {
            format!(
                "{}={}",
                percent_encode(k),
                percent_encode(v)
            )
        })
        .collect::<Vec<_>>()
        .join("&")
}

/// Minimal percent-encoding for form values.
/// Encodes characters that are not unreserved in URL encoding.
fn percent_encode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            b' ' => "+".to_string(),
            _ => format!("%{:02X}", b),
        })
        .collect()
}
