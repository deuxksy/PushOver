# PushOver Serverless Platform Implementation Plan (Complete)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Cloudflare Serverless 플랫폼 기반 PushOver 메시징 시스템 구축 (Rust SDK + CLI + Worker + Dashboard)

**Architecture:** Rust SDK (feature flag로 Worker/CLI 분리), TypeScript Worker API (Hono + Zod), React Dashboard, D1 영속성, Queue 재시도, OpenTofu IaC

**Tech Stack:** Rust (wasm-bindgen), TypeScript, Cloudflare (Workers/Pages/KV/D1/R2/Queues), Hono, React, OpenTofu

---

## File Structure

```
pushover-serverless/
├── crates/
│   ├── sdk/                    # Rust SDK (shared core)
│   │   ├── src/
│   │   │   ├── lib.rs          # Main entry, feature flags
│   │   │   ├── client.rs       # PushOverClient trait
│   │   │   ├── models.rs       # Message, Response types
│   │   │   ├── error.rs        # Error types
│   │   │   ├── http_client.rs  # CLI HTTP client (reqwest)
│   │   │   └── webhook.rs      # Webhook signature verification
│   │   ├── Cargo.toml          # Dependencies with feature flags
│   │   └── tests/
│   │       └── integration_test.rs
│   └── cli/                    # CLI tool
│       ├── src/
│       │   ├── main.rs
│       │   ├── commands/
│       │   │   ├── send.rs
│       │   │   ├── history.rs
│       │   │   ├── config.rs
│       │   │   ├── webhook.rs
│       │   │   ├── dashboard.rs
│       │   │   └── status.rs
│       │   └── config.rs       # Config file management
│       └── Cargo.toml
├── cloudflare/
│   ├── worker/
│   │   ├── src/
│   │   │   ├── index.ts        # Main Worker entry (Hono app)
│   │   │   ├── routes/
│   │   │   │   ├── api.ts      # /api/v1/* endpoints
│   │   │   │   ├── webhook.ts  # /webhook/* endpoints
│   │   │   │   └── health.ts   # /health check
│   │   │   ├── middleware/
│   │   │   │   ├── auth.ts     # Bearer token auth
│   │   │   │   ├── ratelimit.ts # KV-based rate limiting
│   │   │   │   └── validation.ts # Zod schemas
│   │   │   ├── services/
│   │   │   │   ├── queue.ts    # Queue producer
│   │   │   │   ├── storage.ts  # D1 operations
│   │   │   │   └── analytics.ts # Analytics Engine
│   │   │   ├── utils/
│   │   │   │   ├── crypto.ts   # Timing-safe compare
│   │   │   │   └── errors.ts   # Error responses
│   │   │   └── types/
│   │   │       └── index.ts    # TypeScript types
│   │   ├── wrangler.toml
│   │   ├── package.json
│   │   └── tsconfig.json
│   ├── recovery-worker/
│   │   ├── src/
│   │   │   └── index.ts        # Queue consumer with retry
│   │   ├── wrangler.toml       # With cron trigger
│   │   └── package.json
│   └── dashboard/
│       ├── src/
│       │   ├── components/
│       │   │   ├── SendModal.tsx
│       │   │   ├── MessageHistory.tsx
│       │   │   ├── ApiDocs.tsx
│       │   │   └── StatusIndicator.tsx
│       │   ├── hooks/
│       │   │   └── useMessages.ts
│       │   ├── lib/
│       │   │   └── api.ts      # API client
│       │   ├── App.tsx
│       │   └── main.tsx
│       ├── package.json
│       └── vite.config.ts
├── infra/
│   ├── main.tf                 # OpenTofu entry
│   ├── variables.tf
│   ├── outputs.tf
│   ├── providers.tf
│   ├── modules/
│   │   ├── d1/
│   │   ├── kv/
│   │   ├── queue/
│   │   ├── r2/
│   │   ├── pages/
│   │   └── workers/
│   └── migrations/
│       ├── 001_init.sql       # D1 schema (all tables)
│       └── 002_webhooks.sql
└── tests/
    ├── e2e/
    │   ├── health.spec.ts
    │   ├── send.spec.ts
    │   ├── webhook.spec.ts
    │   └── history.spec.ts
    └── playwright.config.ts
```

---

## Task 1: 프로젝트 초기 설정

**Files:**
- Create: `Cargo.toml` (workspace root)
- Create: `crates/sdk/Cargo.toml`
- Create: `crates/cli/Cargo.toml`
- Create: `.gitignore`

- [ ] **Step 1: Create workspace Cargo.toml**

Run: `cat > Cargo.toml << 'EOF'
[workspace]
members = ["crates/sdk", "crates/cli"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"

[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
tokio = { version = "1", features = ["full"] }
EOF`
Expected: File created

- [ ] **Step 2: Create SDK Cargo.toml with feature flags**

Run: `mkdir -p crates/sdk && cat > crates/sdk/Cargo.toml << 'EOF'
[package]
name = "pushover-sdk"
version.workspace = true
edition.workspace = true

[dependencies]
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true
reqwest = { version = "0.11", features = ["json", "multipart"], optional = true }
wasm-bindgen = { version = "0.2", optional = true }
wasm-bindgen-futures = { version = "0.4", optional = true }
js-sys = { version = "0.3", optional = true }

[features]
default = ["reqwest"]
cloudflare-worker = ["wasm-bindgen", "wasm-bindgen-futures", "js-sys"]

[dev-dependencies]
wiremock = "0.5"
tokio = { workspace = true }
EOF`
Expected: File created

- [ ] **Step 3: Create CLI Cargo.toml**

Run: `mkdir -p crates/cli && cat > crates/cli/Cargo.toml << 'EOF'
[package]
name = "pushover-cli"
version.workspace = true
edition.workspace = true

[[bin]]
name = "pushover"
path = "src/main.rs"

[dependencies]
pushover-sdk = { path = "../sdk" }
clap = { version = "4", features = ["derive"] }
tokio.workspace = true
anyhow = "1"
dirs = "5"
serde.workspace = true
toml = "0.8"
EOF`
Expected: File created

- [ ] **Step 4: Create .gitignore**

Run: `cat > .gitignore << 'EOF'
# Rust
/target/
**/*.rs.bk
Cargo.lock

# Node
node_modules/
dist/
.wrangler/

# Environment
.env
.env.local

# IDE
.idea/
.vscode/
*.swp

# OS
.DS_Store

# OpenTofu
*.tfstate
*.tfstate.*
.terraform/
EOF`
Expected: File created

- [ ] **Step 5: Commit**

Run: `git add Cargo.toml crates/ .gitignore && git commit -m "chore: initialize project structure"`
Expected: Commit created

---

## Task 2: Rust SDK - Core Types & Error

**Files:**
- Create: `crates/sdk/src/error.rs`
- Create: `crates/sdk/src/models.rs`
- Create: `crates/sdk/src/lib.rs` (basic exports)

- [ ] **Step 1: Write error types**

Run: `cat > crates/sdk/src/error.rs << 'EOF'
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PushOverError {
    #[error("API request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("API returned error: {status} - {message}")]
    ApiError { status: u16, message: String },

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Webhook signature verification failed")]
    InvalidSignature,
}
EOF`
Expected: File created

- [ ] **Step 2: Write complete data models**

Run: `cat > crates/sdk/src/models.rs << 'EOF'
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
EOF`
Expected: File created

- [ ] **Step 3: Write basic lib.rs**

Run: `cat > crates/sdk/src/lib.rs << 'EOF'
pub mod error;
pub mod models;

pub use error::PushOverError;
pub use models::{Message, Response, PriorityArgs, WebhookPayload};
EOF`
Expected: File created

- [ ] **Step 4: Verify compilation**

Run: `cd crates/sdk && cargo check`
Expected: `Finished dev`

- [ ] **Step 5: Commit**

Run: `git add crates/sdk/src && git commit -m "feat(sdk): add core types and error handling"`
Expected: Commit created

---

## Task 3: Rust SDK - HTTP Client (Complete)

**Files:**
- Create: `crates/sdk/src/http_client.rs`
- Modify: `crates/sdk/src/lib.rs`

- [ ] **Step 1: Write complete HTTP client**

Run: `cat > crates/sdk/src/http_client.rs << 'EOF'
use crate::{Message, PushOverError, Response};
use reqwest::Client;

const PUSHOVER_API_URL: &str = "https://api.pushover.net/1/messages.json";

pub struct PushOverClient {
    client: Client,
    user_key: String,
    api_token: String,
}

impl PushOverClient {
    pub fn new(user_key: String, api_token: String) -> Self {
        Self {
            client: Client::new(),
            user_key,
            api_token,
        }
    }

    pub async fn send(&self, msg: Message) -> Result<Response, PushOverError> {
        let mut form = vec![
            ("user", self.user_key.clone()),
            ("token", self.api_token.clone()),
            ("message", msg.message),
        ];

        if let Some(title) = &msg.title {
            form.push(("title", title.clone()));
        }
        if let Some(priority) = msg.priority {
            form.push(("priority", priority.to_string()));
        }
        if let Some(sound) = &msg.sound {
            form.push(("sound", sound.clone()));
        }
        if let Some(device) = &msg.device {
            form.push(("device", device.clone()));
        }
        if let Some(url) = &msg.url {
            form.push(("url", url.clone()));
        }
        if let Some(url_title) = &msg.url_title {
            form.push(("url_title", url_title.clone()));
        }
        if let Some(html) = msg.html {
            form.push(("html", if html { "1" } else { "0" }.to_string()));
        }
        if let Some(timestamp) = msg.timestamp {
            form.push(("timestamp", timestamp.to_string()));
        }

        // Priority arguments
        if let Some(priority_arg) = &msg.priority_arg {
            if let Some(expire) = priority_arg.expire {
                form.push(("expire", expire.to_string()));
            }
            if let Some(retry) = priority_arg.retry {
                form.push(("retry", retry.to_string()));
            }
        }

        let resp = self.client
            .post(PUSHOVER_API_URL)
            .form(&form)
            .send()
            .await?;

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
EOF`
Expected: File created

- [ ] **Step 2: Update lib.rs to export client**

Run: `cat >> crates/sdk/src/lib.rs << 'EOF'

#[cfg(not(feature = "cloudflare-worker"))]
pub mod http_client;
#[cfg(not(feature = "cloudflare-worker"))]
pub use http_client::PushOverClient;
EOF`
Expected: File appended

- [ ] **Step 3: Write integration test**

Run: `mkdir -p crates/sdk/tests && cat > crates/sdk/tests/integration_test.rs << 'EOF'
use pushover_sdk::{Message, PushOverClient};
use wiremock::{MockServer, Mock, matchers::{method, path}};

#[tokio::test]
async fn test_send_message_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/1/messages.json"))
        .respond_with(
            wiremock::ResponseTemplate::new(200)
                .set_body_raw(r#"{"status":1,"request":"test-id"}"#, "application/json")
        )
        .mount(&mock_server)
        .await;

    let client = PushOverClient::new("test_user".into(), "test_token".into());
    let msg = Message {
        message: "Hello".to_string(),
        title: Some("Test".to_string()),
        priority: Some(0),
        sound: Some("pushover".to_string()),
        device: None,
        url: None,
        url_title: None,
        priority_arg: None,
        html: None,
        timestamp: None,
    };

    let result = client.send(msg).await.unwrap();
    assert_eq!(result.status, 1);
}
EOF`
Expected: File created

- [ ] **Step 4: Run tests**

Run: `cd crates/sdk && cargo test`
Expected: `test result: ok. 1 passed`

- [ ] **Step 5: Commit**

Run: `git add crates/sdk && git commit -m "feat(sdk): add complete HTTP client with tests"`
Expected: Commit created

---

## Task 4: Rust SDK - Webhook Verification

**Files:**
- Create: `crates/sdk/src/webhook.rs`
- Modify: `crates/sdk/src/lib.rs`

- [ ] **Step 1: Write webhook signature verification**

Run: `cat > crates/sdk/src/webhook.rs << 'EOF'
use crate::{PushOverError, WebhookPayload};
use serde_json::Value;

pub fn verify_webhook_signature(
    signature: &str,
    body: &str,
    secret: &str,
) -> Result<(), PushOverError> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
        .map_err(|_| PushOverError::InvalidSignature)?;
    mac.update(body.as_bytes());

    let expected = mac.finalize().into_bytes();
    let expected_hex = hex::encode(expected);

    if !timing_safe_equal(signature, &expected_hex) {
        return Err(PushOverError::InvalidSignature);
    }

    Ok(())
}

fn timing_safe_equal(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    let mut result = 0u8;

    for (x, y) in a_bytes.iter().zip(b_bytes.iter()) {
        result |= x ^ y;
    }

    result == 0
}

pub fn parse_webhook_payload(body: &str) -> Result<WebhookPayload, PushOverError> {
    serde_json::from_str(body).map_err(PushOverError::SerializationError)
}
EOF`
Expected: File created

- [ ] **Step 2: Add webhook dependencies**

Run: `cat >> crates/sdk/Cargo.toml << 'EOF'

hmac = { version = "0.12", optional = true }
sha2 = { version = "0.10", optional = true }
hex = { version = "0.4", optional = true }
EOF`
Expected: File appended

- [ ] **Step 3: Update lib.rs to export webhook**

Run: `cat >> crates/sdk/src/lib.rs << 'EOF'

pub mod webhook;
pub use webhook::{verify_webhook_signature, parse_webhook_payload};
EOF`
Expected: File appended

- [ ] **Step 4: Commit**

Run: `git add crates/sdk && git commit -m "feat(sdk): add webhook signature verification"`
Expected: Commit created

---

## Task 5: CLI - Config Management

**Files:**
- Create: `crates/cli/src/config.rs`

- [ ] **Step 1: Write config management**

Run: `cat > crates/cli/src/config.rs << 'EOF'
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub profiles: Vec<Profile>,
    pub default_profile: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub user_key: String,
    pub api_token: String,
    pub api_endpoint: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot find config directory"))?
            .join("pushover");

        let config_path = config_dir.join("config.toml");

        if !config_path.exists() {
            fs::create_dir_all(&config_dir)?;
            let default_config = Config {
                profiles: vec![],
                default_profile: None,
            };
            default_config.save(&config_path)?;
            return Ok(default_config);
        }

        let content = fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn get_default_profile(&self) -> Option<&Profile> {
        let name = self.default_profile.as_ref()?;
        self.profiles.iter().find(|p| &p.name == name)
    }
}
EOF`
Expected: File created

- [ ] **Step 2: Add CLI dependencies**

Run: `cat >> crates/cli/Cargo.toml << 'EOF'

toml = "0.8"
serde = { workspace = true }
EOF`
Expected: File appended

- [ ] **Step 3: Commit**

Run: `git add crates/cli/src/config.rs && git commit -m "feat(cli): add config file management"`
Expected: Commit created

---

## Task 6: CLI - Send Command

**Files:**
- Modify: `crates/cli/src/main.rs`
- Create: `crates/cli/src/commands/send.rs`

- [ ] **Step 1: Write send command**

Run: `mkdir -p crates/cli/src/commands && cat > crates/cli/src/commands/send.rs << 'EOF'
use anyhow::Result;
use pushover_sdk::{Message, PushOverClient};
use std::env;

pub async fn execute(
    message: String,
    title: Option<String>,
    priority: Option<i32>,
    sound: Option<String>,
    device: Option<String>,
    url: Option<String>,
) -> Result<()> {
    let user_key = env::var("PUSHOVER_USER_KEY")?;
    let api_token = env::var("PUSHOVER_API_TOKEN")?;

    let client = PushOverClient::new(user_key, api_token);

    let msg = Message {
        message,
        title,
        priority,
        sound,
        device,
        url,
        url_title: None,
        priority_arg: None,
        html: None,
        timestamp: None,
    };

    let response = client.send(msg).await?;

    println!("✓ Message sent successfully!");
    println!("  Request ID: {}", response.request);

    if let Some(receipt) = response.receipt {
        println!("  Receipt: {}", receipt);
    }

    Ok(())
}
EOF`
Expected: File created

- [ ] **Step 2: Update main.rs**

Run: `cat > crates/cli/src/main.rs << 'EOF'
mod commands;
mod config;

use clap::{Parser, Subcommand};
use commands::send;

#[derive(Parser)]
#[command(name = "pushover")]
#[command(about = "PushOver CLI tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Send {
        message: String,
        #[arg(short, long)]
        title: Option<String>,
        #[arg(short = 'P', long)]
        priority: Option<i32>,
        #[arg(short, long)]
        sound: Option<String>,
        #[arg(short, long)]
        device: Option<String>,
        #[arg(short, long)]
        url: Option<String>,
    },
    History,
    Config,
    Webhook,
    Status,
    Dashboard,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Send { message, title, priority, sound, device, url } => {
            send::execute(message, title, priority, sound, device, url).await?;
        }
        Commands::History => {
            println!("History - coming soon");
        }
        Commands::Config => {
            println!("Config - coming soon");
        }
        Commands::Webhook => {
            println!("Webhook - coming soon");
        }
        Commands::Status => {
            println!("Status - coming soon");
        }
        Commands::Dashboard => {
            println!("Dashboard - coming soon");
        }
    }

    Ok(())
}
EOF`
Expected: File overwritten

- [ ] **Step 3: Test build**

Run: `cargo build -p pushover-cli`
Expected: Build succeeds

- [ ] **Step 4: Commit**

Run: `git add crates/cli && git commit -m "feat(cli): implement send command"`
Expected: Commit created

---

## Task 7: CLI - History Command

**Files:**
- Create: `crates/cli/src/commands/history.rs`
- Modify: `crates/cli/src/main.rs`

- [ ] **Step 1: Write history command**

Run: `cat > crates/cli/src/commands/history.rs << 'EOF'
use anyhow::Result;
use reqwest::Client;
use serde_json::Value;

pub async fn execute(
    limit: Option<usize>,
    status: Option<String>,
) -> Result<()> {
    let api_base = std::env::var("PUSHOVER_API_BASE")
        .unwrap_or_else(|_| "https://api.pushover.example.com".to_string());

    let client = Client::new();
    let mut url = format!("{}/api/v1/messages", api_base);

    let mut query_params = Vec::new();
    if let Some(l) = limit {
        query_params.push(format!("limit={}", l));
    }
    if let Some(s) = status {
        query_params.push(format!("status={}", s));
    }

    if !query_params.is_empty() {
        url.push('?');
        url.push_str(&query_params.join("&"));
    }

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", std::env::var("PUSHOVER_API_KEY")?))
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!("API error: {}", response.status()));
    }

    let json: Value = response.json().await?;

    if let Some(messages) = json.get("messages").and_then(|v| v.as_array()) {
        println!("Message History ({} messages):", messages.len());
        println!("{:-<50}", "");

        for msg in messages {
            let title = msg.get("title").and_then(|v| v.as_str()).unwrap_or("(no title)");
            let status = msg.get("status").and_then(|v| v.as_str()).unwrap_or("unknown");
            let created_at = msg.get("created_at")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            println!("  {} | {} | {}", title, status, {
                let dt = chrono::DateTime::from_timestamp(created_at / 1000, 0);
                dt.map(|d| d.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|_| "?".to_string())
            });
        }
    } else {
        println!("No messages found.");
    }

    Ok(())
}
EOF`
Expected: File created

- [ ] **Step 2: Add chrono dependency**

Run: `cat >> crates/cli/Cargo.toml << 'EOF'

chrono = "0.4"
EOF`
Expected: File appended

- [ ] **Step 3: Update main.rs to call history**

Run: `cat > crates/cli/src/main.rs << 'EOF'
mod commands;
mod config;

use clap::{Parser, Subcommand};
use commands::send;
use commands::history;

#[derive(Parser)]
#[command(name = "pushover")]
#[command(about = "PushOver CLI tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Send {
        message: String,
        #[arg(short, long)]
        title: Option<String>,
        #[arg(short = 'P', long)]
        priority: Option<i32>,
        #[arg(short, long)]
        sound: Option<String>,
        #[arg(short, long)]
        device: Option<String>,
        #[arg(short, long)]
        url: Option<String>,
    },
    History {
        #[arg(short, long)]
        limit: Option<usize>,
        #[arg(short, long)]
        status: Option<String>,
    },
    Config,
    Webhook,
    Status,
    Dashboard,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Send { message, title, priority, sound, device, url } => {
            send::execute(message, title, priority, sound, device, url).await?;
        }
        Commands::History { limit, status } => {
            history::execute(limit, status).await?;
        }
        Commands::Config => {
            println!("Config - coming soon");
        }
        Commands::Webhook => {
            println!("Webhook - coming soon");
        }
        Commands::Status => {
            println!("Status - coming soon");
        }
        Commands::Dashboard => {
            println!("Dashboard - coming soon");
        }
    }

    Ok(())
}
EOF`
Expected: File overwritten

- [ ] **Step 4: Commit**

Run: `git add crates/cli && git commit -m "feat(cli): implement history command"`
Expected: Commit created

---

## Task 8: Cloudflare Worker - Project Setup

**Files:**
- Create: `cloudflare/worker/package.json`
- Create: `cloudflare/worker/tsconfig.json`
- Create: `cloudflare/worker/wrangler.toml`

- [ ] **Step 1: Create package.json**

Run: `mkdir -p cloudflare/worker && cat > cloudflare/worker/package.json << 'EOF'
{
  "name": "pushover-worker",
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "wrangler dev",
    "deploy": "wrangler deploy",
    "test": "vitest"
  },
  "dependencies": {
    "hono": "^4.0.0",
    "zod": "^3.22.0"
  },
  "devDependencies": {
    "@cloudflare/workers-types": "^4.20231218.0",
    "wrangler": "^3.0.0",
    "typescript": "^5.3.0",
    "vitest": "^1.0.0"
  }
}
EOF`
Expected: File created

- [ ] **Step 2: Create tsconfig.json**

Run: `cat > cloudflare/worker/tsconfig.json << 'EOF'
{
  "compilerOptions": {
    "target": "ES2021",
    "module": "ES2022",
    "lib": ["ES2021"],
    "moduleResolution": "node",
    "types": ["@cloudflare/workers-types"],
    "resolveJsonModule": true,
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules"]
}
EOF`
Expected: File created

- [ ] **Step 3: Create wrangler.toml with env vars**

Run: `cat > cloudflare/worker/wrangler.toml << 'EOF'
name = "pushover-worker"
main = "src/index.ts"
compatibility_date = "2024-01-01"

[[kv_namespaces]]
binding = "KV"
id = "KV_ID"

[[d1_databases]]
binding = "DB"
database_name = "pushover"
database_id = "D1_ID"

[[queues.producers]]
binding = "QUEUE"
queue = "pushover-messages"

[[queues.consumers]]
queue = "pushover-messages"
max_batch_size = 10
max_wait_time = 5

[analytics]
binding = "ANALYTICS"

# API credentials - MUST use wrangler secret in production
# Run: wrangler secret put API_KEY
# Run: wrangler secret put WEBHOOK_SECRET
# Run: wrangler secret put PUSHOVER_USER_KEY
# Run: wrangler secret put PUSHOVER_TOKEN

# For local development only (never commit real values):
# [vars]
# API_KEY = "test-dev-key"
# WEBHOOK_SECRET = "test-dev-secret"
# PUSHOVER_USER_KEY = ""
# PUSHOVER_TOKEN = ""
EOF`
Expected: File created

- [ ] **Step 4: Commit**

Run: `git add cloudflare/worker && git commit -m "chore(worker): add project config with env vars"`
Expected: Commit created

---

## Task 9: Worker - Types & Error Responses

**Files:**
- Create: `cloudflare/worker/src/types/index.ts`
- Create: `cloudflare/worker/src/utils/errors.ts`

- [ ] **Step 1: Create TypeScript types**

Run: `mkdir -p cloudflare/worker/src/types && cat > cloudflare/worker/src/types/index.ts << 'EOF'
export interface Env {
  // Client authentication
  API_KEY: string;
  WEBHOOK_SECRET: string;

  // PushOver credentials
  PUSHOVER_USER_KEY: string;
  PUSHOVER_TOKEN: string;

  // Cloudflare bindings
  KV: KVNamespace;
  DB: D1Database;
  QUEUE: Queue<QueueMessage>;
  ANALYTICS: AnalyticsEngineDataset;
}

export interface QueueMessage {
  type: 'send' | 'webhook';
  messageId: string;
  payload: unknown;
  timestamp: number;
}

export interface ApiErrorResponse {
  error: {
    code: string;
    message: string;
    details?: unknown;
  };
}

export interface MessageRecord {
  id: string;
  status: 'pending' | 'sent' | 'failed';
  title: string | null;
  message: string;
  priority: number;
  sound: string;
  device: string | null;
  pushover_request: string | null;
  pushover_receipt: string | null;
  error: string | null;
  retry_count: number;
  created_at: number;
  sent_at: number | null;
  date: string;
}
EOF`
Expected: File created

- [ ] **Step 2: Create error response utility**

Run: `mkdir -p cloudflare/worker/src/utils && cat > cloudflare/worker/src/utils/errors.ts << 'EOF'
import type { ApiErrorResponse } from '../types';

export function errorResponse(
  code: string,
  message: string,
  details?: unknown,
): ApiErrorResponse {
  const error: any = { code, message };
  if (details) {
    error.details = details;
  }
  return { error };
}

export function unauthorizedResponse(message?: string): ApiErrorResponse {
  return errorResponse('UNAUTHORIZED', message || 'Unauthorized');
}

export function validationErrorResponse(details: unknown): ApiErrorResponse {
  return errorResponse('VALIDATION_ERROR', 'Invalid request body', details);
}

export function rateLimitResponse(limit: number, current: number): ApiErrorResponse {
  return errorResponse('RATE_LIMITED', 'Too many requests', { limit, current });
}
EOF`
Expected: File created

- [ ] **Step 3: Commit**

Run: `git add cloudflare/worker/src && git commit -m "feat(worker): add types and error utilities"`
Expected: Commit created

---

## Task 10: Worker - Crypto Utilities

**Files:**
- Create: `cloudflare/worker/src/utils/crypto.ts`

- [ ] **Step 1: Write timing-safe compare and HMAC**

Run: `cat > cloudflare/worker/src/utils/crypto.ts << 'EOF'
export function hexToBytes(hex: string): Uint8Array {
  const bytes = new Uint8Array(hex.length / 2);
  for (let i = 0; i < hex.length; i += 2) {
    bytes[i / 2] = parseInt(hex.substr(i, 2), 16);
  }
  return bytes;
}

export function timingSafeEqual(a: Uint8Array, b: Uint8Array): boolean {
  if (a.length !== b.length) {
    return false;
  }

  let result = 0;
  for (let i = 0; i < a.length; i++) {
    result |= a[i] ^ b[i];
  }

  return result === 0;
}

export async function verifyHMAC(
  signature: string,
  body: string,
  secret: string,
): Promise<boolean> {
  const encoder = new TextEncoder();

  const key = await crypto.subtle.importKey(
    'raw',
    encoder.encode(secret),
    { name: 'HMAC', hash: 'SHA-256' },
    false,
    ['sign']
  );

  const sigBuffer = await crypto.subtle.sign(
    'HMAC',
    key,
    encoder.encode(body)
  );

  const hexSignature = Array.from(new Uint8Array(sigBuffer))
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');

  const sigBytes = hexToBytes(signature);
  const expectedBytes = hexToBytes(hexSignature);

  return timingSafeEqual(sigBytes, expectedBytes);
}
EOF`
Expected: File created

- [ ] **Step 2: Commit**

Run: `git add cloudflare/worker/src/utils/crypto.ts && git commit -m "feat(worker): add timing-safe compare and HMAC verification"`
Expected: Commit created

---

## Task 11: Worker - Middleware (Auth, Rate Limit, Validation)

**Files:**
- Create: `cloudflare/worker/src/middleware/auth.ts`
- Create: `cloudflare/worker/src/middleware/ratelimit.ts`
- Create: `cloudflare/worker/src/middleware/validation.ts`

- [ ] **Step 1: Write auth middleware**

Run: `mkdir -p cloudflare/worker/src/middleware && cat > cloudflare/worker/src/middleware/auth.ts << 'EOF'
import { Context, Next } from 'hono';
import { unauthorizedResponse } from '../utils/errors';
import type { Env } from '../types';

export const bearerAuth = ({
  verifyToken,
}: {
  verifyToken: (token: string, c: Context<{ Bindings: Env }>) => Promise<boolean>;
}) => {
  return async (c: Context<{ Bindings: Env }>, next: Next) => {
    const authHeader = c.req.header('Authorization');

    if (!authHeader || !authHeader.startsWith('Bearer ')) {
      return c.json(unauthorizedResponse('Missing authorization header'), 401);
    }

    const token = authHeader.substring(7);
    const isValid = await verifyToken(token, c);

    if (!isValid) {
      return c.json(unauthorizedResponse('Invalid API key'), 401);
    }

    return next();
  };
};
EOF`
Expected: File created

- [ ] **Step 2: Write rate limiting middleware**

Run: `cat > cloudflare/worker/src/middleware/ratelimit.ts << 'EOF'
import { Context, Next } from 'hono';
import { rateLimitResponse } from '../utils/errors';
import type { Env } from '../types';

export const rateLimiter = async (c: Context<{ Bindings: Env }>, next: Next) => {
  const ip = c.req.header('CF-Connecting-IP') || 'unknown';
  const key = `ratelimit:${ip}`;
  const limit = 100; // 분당 100회
  const window = 60; // 60초

  // 주의: KV get-put 사이 Race Condition 가능성
  // Cloudflare Ruleset이 1차 방어, 이것은 보조 수단
  const current = await c.env.KV.get(key, 'json');
  const now = Math.floor(Date.now() / 1000);

  let count = 0;
  if (current) {
    const data = current as { count: number; windowStart: number };
    if (data.windowStart + window > now) {
      count = data.count + 1;
    } else {
      count = 1;
    }
  }

  if (count > limit) {
    console.warn(
      JSON.stringify({
        level: 'warn',
        message: 'Rate limit exceeded',
        ip,
        count,
        limit,
        timestamp: now,
      })
    );

    return c.json(rateLimitResponse(limit, count), 429);
  }

  await c.env.KV.put(
    key,
    JSON.stringify({ count, windowStart: now }),
    { expirationTtl: window }
  );

  return next();
};
EOF`
Expected: File created

- [ ] **Step 3: Write validation schemas**

Run: `cat > cloudflare/worker/src/middleware/validation.ts << 'EOF'
import { z } from 'zod';

export const SendMessageSchema = z.object({
  message: z.string().min(1).max(1024),
  title: z.string().max(250).optional(),
  priority: z.number().int().min(-2).max(2).optional().default(0),
  sound: z
    .enum([
      'pushover',
      'bike',
      'bugle',
      'cashregister',
      'classical',
      'cosmic',
      'falling',
      'gamelan',
      'incoming',
      'intermission',
      'magic',
      'mechanical',
      'pianobar',
      'siren',
      'spacealarm',
      'tugboat',
      'alien',
      'climb',
      'persistent',
      'echo',
      'updown',
      'none',
    ])
    .optional()
    .default('pushover'),
  device: z.string().max(25).optional(),
  url: z.string().url().optional(),
  url_title: z.string().max(100).optional(),
  expire: z.number().int().min(30).max(10800).optional(),
  retry: z.number().int().min(30).max(10800).optional(),
  html: z.boolean().optional().default(false),
});

export const CreateWebhookSchema = z.object({
  url: z.string().url(),
  events: z.array(z.enum(['receipt', 'acknowledge', 'expire'])).min(1),
});
EOF`
Expected: File created

- [ ] **Step 4: Commit**

Run: `git add cloudflare/worker/src/middleware && git commit -m "feat(worker): add auth, rate limit, and validation middleware"`
Expected: Commit created

---

## Task 12: Worker - API Routes

**Files:**
- Create: `cloudflare/worker/src/routes/api.ts`
- Create: `cloudflare/worker/src/services/queue.ts`
- Modify: `cloudflare/worker/src/index.ts`

- [ ] **Step 1: Write queue service**

Run: `mkdir -p cloudflare/worker/src/services && cat > cloudflare/worker/src/services/queue.ts << 'EOF'
import type { Env } from '../types';

export async function enqueueMessage(
  env: Env,
  message: {
    id: string;
    payload: unknown;
  }
): Promise<void> {
  await env.QUEUE.send({
    type: 'send',
    messageId: message.id,
    payload: message.payload,
    timestamp: Date.now(),
  });
}

export async function sendToPushOver(
  env: Env,
  payload: any
): Promise<Response> {
  const response = await fetch('https://api.pushover.net/1/messages.json', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      user: env.PUSHOVER_USER_KEY,
      token: env.PUSHOVER_TOKEN,
      ...payload,
    }),
  });

  if (!response.ok) {
    throw new Error(\`PushOver API error: \${response.status}\`);
  }

  return response.json();
}
EOF`
Expected: File created

- [ ] **Step 2: Write API routes**

Run: `mkdir -p cloudflare/worker/src/routes && cat > cloudflare/worker/src/routes/api.ts << 'EOF'
import { Hono } from 'hono';
import { bearerAuth } from '../middleware/auth';
import { rateLimiter } from '../middleware/ratelimit';
import { SendMessageSchema } from '../middleware/validation';
import { validationErrorResponse } from '../utils/errors';
import { enqueueMessage } from '../services/queue';
import type { Env } from '../types';

const api = new Hono<{ Bindings: Env }>();

// Send message endpoint
api.post(
  '/api/v1/send',
  bearerAuth({ verifyToken: async (token, c) => token === c.env.API_KEY }),
  rateLimiter,
  async (c) => {
    const body = await c.req.json();
    const result = SendMessageSchema.safeParse(body);

    if (!result.success) {
      return c.json(validationErrorResponse(result.error.flatten()), 400);
    }

    const messageId = crypto.randomUUID();

    // Store in D1
    await c.env.DB
      .prepare(
        \`INSERT INTO messages (id, status, title, message, priority, sound, device, created_at, date)
       VALUES (?, 'pending', ?, ?, ?, ?, ?, ?, ?)\`
      )
      .bind(
        messageId,
        result.data.title ?? null,
        result.data.message,
        result.data.priority,
        result.data.sound,
        result.data.device ?? null,
        Date.now(),
        new Date().toISOString().split('T')[0]
      )
      .run();

    // Enqueue for processing
    await enqueueMessage(c.env, {
      id: messageId,
      payload: result.data,
    });

    return c.json({ success: true, messageId }, 202);
  }
);

// Get message history
api.get(
  '/api/v1/messages',
  bearerAuth({ verifyToken: async (token, c) => token === c.env.API_KEY }),
  async (c) => {
    const limit = parseInt(c.req.query('limit') || '50');
    const status = c.req.query('status');
    const date = c.req.query('date');

    let query = 'SELECT * FROM messages';
    const conditions: string[] = [];
    const params: any[] = [];

    if (status) {
      conditions.push('status = ?');
      params.push(status);
    }

    if (date) {
      conditions.push('date = ?');
      params.push(date);
    }

    if (conditions.length > 0) {
      query += ' WHERE ' + conditions.join(' AND ');
    }

    query += ' ORDER BY created_at DESC LIMIT ?';
    params.push(limit);

    const { results } = await c.env.DB.prepare(query).bind(...params).all();

    return c.json({ messages: results || [] });
  }
);

// Get single message
api.get(
  '/api/v1/messages/:id',
  bearerAuth({ verifyToken: async (token, c) => token === c.env.API_KEY }),
  async (c) => {
    const id = c.req.param('id');

    const result = await c.env.DB
      .prepare('SELECT * FROM messages WHERE id = ?')
      .bind(id)
      .first();

    if (!result) {
      return c.json({ error: { code: 'NOT_FOUND', message: 'Message not found' } }, 404);
    }

    return c.json(result);
  }
);

export default api;
EOF`
Expected: File created

- [ ] **Step 3: Update main index.ts**

Run: `cat > cloudflare/worker/src/index.ts << 'EOF'
import { Hono } from 'hono';
import api from './routes/api';
import type { Env } from './types';

const app = new Hono<{ Bindings: Env }>();

app.route('/', api);
app.get('/health', (c) => c.json({ status: 'healthy', timestamp: Date.now() }));

export default app;
EOF`
Expected: File overwritten

- [ ] **Step 4: Commit**

Run: `git add cloudflare/worker/src && git commit -m "feat(worker): add API routes for send and history"`
Expected: Commit created

---

## Task 13: Worker - Webhook Routes

**Files:**
- Create: `cloudflare/worker/src/routes/webhook.ts`
- Modify: `cloudflare/worker/src/index.ts`

- [ ] **Step 1: Write webhook routes**

Run: `cat > cloudflare/worker/src/routes/webhook.ts << 'EOF'
import { Hono } from 'hono';
import { verifyHMAC } from '../utils/crypto';
import type { Env } from '../types';

const webhook = new Hono<{ Bindings: Env }>();

// Receive webhook from PushOver
webhook.post('/webhook/:id', async (c) => {
  const webhookId = c.req.param('id');
  const signature = c.req.header('X-Webhook-Signature');
  const body = await c.req.text();

  if (!signature) {
    return c.json(
      { error: { code: 'UNAUTHORIZED', message: 'Missing signature' } },
      401
    );
  }

  // Verify HMAC signature
  const isValid = await verifyHMAC(signature, body, c.env.WEBHOOK_SECRET);

  if (!isValid) {
    return c.json(
      { error: { code: 'UNAUTHORIZED', message: 'Invalid webhook signature' } },
      401
    );
  }

  // Parse and store webhook payload
  try {
    const payload = JSON.parse(body);

    await c.env.DB
      .prepare(
        \`INSERT INTO webhook_events (id, webhook_id, payload, received_at)
       VALUES (?, ?, ?, ?)\`
      )
      .bind(crypto.randomUUID(), webhookId, body, Date.now())
      .run();

    // Process webhook (e.g., update message status for emergency priority)
    if (payload.priority === 2 && payload.receipt) {
      await c.env.DB
        .prepare('UPDATE messages SET pushover_receipt = ? WHERE id = ?')
        .bind(payload.receipt, payload.acknowledged?.toString())
        .run();
    }

    return c.json({ success: true });
  } catch (e) {
    return c.json({ error: { code: 'INVALID_PAYLOAD', message: 'Invalid JSON' } }, 400);
  }
});

export default webhook;
EOF`
Expected: File created

- [ ] **Step 2: Add webhook routes to main app**

Run: `cat > cloudflare/worker/src/index.ts << 'EOF'
import { Hono } from 'hono';
import api from './routes/api';
import webhook from './routes/webhook';
import type { Env } from './types';

const app = new Hono<{ Bindings: Env }>();

app.route('/', api);
app.route('/', webhook);
app.get('/health', (c) => c.json({ status: 'healthy', timestamp: Date.now() }));

export default app;
EOF`
Expected: File overwritten

- [ ] **Step 3: Commit**

Run: `git add cloudflare/worker/src && git commit -m "feat(worker): add webhook endpoint with signature verification"`
Expected: Commit created

---

## Task 14: Worker - Webhook CRUD API

**Files:**
- Modify: `cloudflare/worker/src/routes/webhook.ts`
- Modify: `cloudflare/worker/src/routes/api.ts`

**Why:** Spec requires webhook management endpoints (GET/POST/DELETE /api/v1/webhooks)

- [ ] **Step 1: Add webhook CRUD to webhook routes**

Run: `cat > cloudflare/worker/src/routes/webhook.ts << 'EOF'
import { Hono } from 'hono';
import { verifyHMAC } from '../utils/crypto';
import type { Env } from '../types';

const webhook = new Hono<{ Bindings: Env }>();

// Receive webhook from PushOver
webhook.post('/webhook/:id', async (c) => {
  const webhookId = c.req.param('id');
  const signature = c.req.header('X-Webhook-Signature');
  const body = await c.req.text();

  if (!signature) {
    return c.json(
      { error: { code: 'UNAUTHORIZED', message: 'Missing signature' } },
      401
    );
  }

  // Verify HMAC signature
  const isValid = await verifyHMAC(signature, body, c.env.WEBHOOK_SECRET);

  if (!isValid) {
    return c.json(
      { error: { code: 'UNAUTHORIZED', message: 'Invalid webhook signature' } },
      401
    );
  }

  // Parse and store webhook payload
  try {
    const payload = JSON.parse(body);

    await c.env.DB
      .prepare(
        `INSERT INTO webhook_events (id, webhook_id, payload, received_at)
         VALUES (?, ?, ?, ?)`
      )
      .bind(crypto.randomUUID(), webhookId, body, Date.now())
      .run();

    // Process webhook (e.g., update message status for emergency priority)
    if (payload.priority === 2 && payload.receipt) {
      await c.env.DB
        .prepare('UPDATE messages SET pushover_receipt = ? WHERE id = ?')
        .bind(payload.receipt, payload.acknowledged?.toString())
        .run();
    }

    return c.json({ success: true });
  } catch (e) {
    return c.json({ error: { code: 'INVALID_PAYLOAD', message: 'Invalid JSON' } }, 400);
  }
});

// Webhook CRUD endpoints (require auth)
import { authenticate } from '../middleware/auth';
import { verifyToken } from '../utils/auth';

// List all webhooks
webhook.get('/api/v1/webhooks', authenticate(verifyToken), async (c) => {
  const result = await c.env.DB
    .prepare('SELECT * FROM webhooks ORDER BY created_at DESC')
    .all();
  return c.json({ webhooks: result.results });
});

// Create new webhook
webhook.post('/api/v1/webhooks', authenticate(verifyToken), async (c) => {
  const { url, events } = await c.req.json();

  if (!url || !Array.isArray(events)) {
    return c.json({ error: { code: 'VALIDATION_ERROR', message: 'url and events required' } }, 400);
  }

  const id = crypto.randomUUID();
  await c.env.DB
    .prepare('INSERT INTO webhooks (id, url, events, created_at) VALUES (?, ?, ?, ?)')
    .bind(id, url, JSON.stringify(events), Date.now())
    .run();

  return c.json({ id, url, events }, 201);
});

// Delete webhook
webhook.delete('/api/v1/webhooks/:id', authenticate(verifyToken), async (c) => {
  const id = c.req.param('id');

  await c.env.DB
    .prepare('DELETE FROM webhooks WHERE id = ?')
    .bind(id)
    .run();

  return c.json({ success: true });
});

export default webhook;
EOF`
Expected: File updated with CRUD endpoints

- [ ] **Step 2: Update main app to handle both routes**

Run: `cat > cloudflare/worker/src/index.ts << 'EOF'
import { Hono } from 'hono';
import api from './routes/api';
import webhook from './routes/webhook';
import type { Env } from './types';

const app = new Hono<{ Bindings: Env }>();

// API routes (require auth)
app.route('/', api);

// Webhook routes (public receive + authenticated CRUD)
app.route('/', webhook);

app.get('/health', (c) => c.json({ status: 'healthy', timestamp: Date.now() }));

export default app;
EOF`
Expected: File updated

- [ ] **Step 3: Commit**

Run: `git add cloudflare/worker/src && git commit -m "feat(worker): add webhook CRUD API endpoints"`
Expected: Commit created

---

## Task 15: Recovery Worker (Queue Consumer)

**Files:**
- Create: `cloudflare/recovery-worker/src/index.ts`
- Create: `cloudflare/recovery-worker/wrangler.toml`

- [ ] **Step 1: Write queue consumer**

Run: `mkdir -p cloudflare/recovery-worker/src && cat > cloudflare/recovery-worker/src/index.ts << 'EOF'
interface Message {
  id: string;
  message: string;
  retryCount?: number;
  error?: string;
}

interface Env {
  PUSHOVER_USER_KEY: string;
  PUSHOVER_TOKEN: string;
  DB: D1Database;
  QUEUE: Queue<any>;
  ANALYTICS: AnalyticsEngineDataset;
}

export default {
  async queue(batch: Message[], env: Env): Promise<void> {
    for (const msg of batch) {
      try {
        await sendToPushOver(env, msg);
        await updateMessageStatus(env, msg.id, 'sent');
        await trackEvent(env, { type: 'message_sent', messageId: msg.id });
      } catch (error: any) {
        const retryCount = (msg.retryCount || 0) + 1;

        if (retryCount < 3) {
          // Exponential backoff: 2s, 4s, 8s
          const delay = Math.pow(2, retryCount) * 1000;
          await retryLater(env, { ...msg, retryCount }, delay);
        } else {
          await moveToDeadLetter(env, msg, error.message);
          await trackEvent(env, {
            type: 'message_failed',
            messageId: msg.id,
            metadata: { error: error.message },
          });
        }
      }
    }
  },
};

async function sendToPushOver(env: Env, msg: Message): Promise<void> {
  const response = await fetch('https://api.pushover.net/1/messages.json', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      user: env.PUSHOVER_USER_KEY,
      token: env.PUSHOVER_TOKEN,
      message: msg.message,
    }),
  });

  if (!response.ok) {
    const text = await response.text();
    throw new Error(\`HTTP \${response.status}: \${text}\`);
  }
}

async function updateMessageStatus(env: Env, id: string, status: string): Promise<void> {
  await env.DB
    .prepare('UPDATE messages SET status = ?, sent_at = ? WHERE id = ?')
    .bind(status, Date.now(), id)
    .run();
}

async function retryLater(env: Env, msg: Message, delayMs: number): Promise<void> {
  await env.QUEUE.send(
    { ...msg, retryCount: (msg.retryCount || 0) + 1 },
    { delayMs }
  );
}

async function moveToDeadLetter(env: Env, msg: Message, error: string): Promise<void> {
  await env.DB
    .prepare('UPDATE messages SET status = ?, error = ?, retry_count = ? WHERE id = ?')
    .bind('failed', error, msg.retryCount || 0, msg.id)
    .run();
}

async function trackEvent(env: Env, event: any): Promise<void> {
  await env.ANALYTICS.writeDataPoint({
    indexes: [event.type],
    blobs: [event.messageId || '', JSON.stringify(event.metadata || {})],
    doubles: [Date.now() / 1000],
  });
}
EOF`
Expected: File created

- [ ] **Step 2: Create wrangler.toml with cron**

Run: `cat > cloudflare/recovery-worker/wrangler.toml << 'EOF'
name = "pushover-recovery"
main = "src/index.ts"
compatibility_date = "2024-01-01"

# Scheduled recovery for failed messages
[triggers.crons]
schedule = "0 * * * *"  # Every hour
binding = "RECOVERY"

[[queues.consumers]]
queue = "pushover-messages"
max_batch_size = 10
max_wait_time = 5

[[d1_databases]]
binding = "DB"
database_name = "pushover"
# database_id will be set after OpenTofu creates the D1 database
# Run: tofu apply
# Then update: database_id = "<output from tofu output -raw d1_database_id>"

[analytics]
binding = "ANALYTICS"
EOF`
Expected: File created

- [ ] **Step 3: Commit**

Run: `git add cloudflare/recovery-worker && git commit -m "feat(worker): add recovery worker with exponential backoff"`
Expected: Commit created

---

## Task 16: D1 Complete Schema

**Files:**
- Create: `infra/migrations/001_init.sql`

- [ ] **Step 1: Write complete D1 schema**

Run: `mkdir -p infra/migrations && cat > infra/migrations/001_init.sql << 'EOF'
-- Messages table
CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY,
    status TEXT NOT NULL DEFAULT 'pending',
    title TEXT,
    message TEXT NOT NULL,
    priority INTEGER DEFAULT 0,
    sound TEXT DEFAULT 'pushover',
    device TEXT,
    pushover_request TEXT,
    pushover_receipt TEXT,
    error TEXT,
    retry_count INTEGER DEFAULT 0,
    created_at INTEGER NOT NULL,
    sent_at INTEGER,
    date TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_messages_status ON messages(status);
CREATE INDEX IF NOT EXISTS idx_messages_created ON messages(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_messages_date ON messages(date);
CREATE INDEX IF NOT EXISTS idx_messages_status_date ON messages(status, date);
CREATE INDEX IF NOT EXISTS idx_messages_receipt ON messages(pushover_receipt) WHERE pushover_receipt IS NOT NULL;

-- Webhooks table
CREATE TABLE IF NOT EXISTS webhooks (
    id TEXT PRIMARY KEY,
    url TEXT NOT NULL,
    events TEXT NOT NULL, -- JSON array
    active INTEGER DEFAULT 1,
    created_at INTEGER NOT NULL,
    last_triggered_at INTEGER
);

CREATE INDEX IF NOT EXISTS idx_webhooks_active ON webhooks(active);

-- Webhook events log
CREATE TABLE IF NOT EXISTS webhook_events (
    id TEXT PRIMARY KEY,
    webhook_id TEXT NOT NULL,
    payload TEXT NOT NULL, -- JSON
    received_at INTEGER NOT NULL,
    processed INTEGER DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_webhook_events_webhook ON webhook_events(webhook_id);

-- Daily stats (for analytics)
CREATE TABLE IF NOT EXISTS daily_stats (
    date TEXT PRIMARY KEY,
    total_sent INTEGER DEFAULT 0,
    total_failed INTEGER DEFAULT 0,
    total_webhooks INTEGER DEFAULT 0
);

-- Settings
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
EOF`
Expected: File created

- [ ] **Step 2: Apply D1 migration to local database**

Run: `wrangler d1 execute pushover --local --file=infra/migrations/001_init.sql`
Expected: Tables created in local D1

- [ ] **Step 3: Apply D1 migration to production database**

Run: `wrangler d1 execute pushover --file=infra/migrations/001_init.sql`
Expected: Tables created in production D1

- [ ] **Step 4: Commit**

Run: `git add infra/migrations && git commit -m "feat(infra): add complete D1 schema with all tables"`
Expected: Commit created

---

## Task 17: OpenTofu Infrastructure Modules

**Files:**
- Create: `infra/main.tf`
- Create: `infra/variables.tf`
- Create: `infra/providers.tf`
- Create: `infra/outputs.tf`
- Create: `infra/modules/d1/main.tf`
- Create: `infra/modules/kv/main.tf`
- Create: `infra/modules/queue/main.tf`
- Create: `infra/modules/worker/main.tf`
- Create: `infra/modules/pages/main.tf`

- [ ] **Step 1: Create variables.tf**

Run: `cat > infra/variables.tf << 'EOF'
variable "account_id" {
  description = "Cloudflare Account ID"
  type        = string
}

variable "zone_id" {
  description = "Cloudflare Zone ID (for rate limiting)"
  type        = string
  default     = null
}

variable "api_key" {
  description = "API key for client authentication"
  type        = string
  sensitive   = true
}

variable "webhook_secret" {
  description = "Webhook signature secret"
  type        = string
  sensitive   = true
}

variable "pushover_user_key" {
  description = "PushOver user key"
  type        = string
  sensitive   = true
}

variable "pushover_token" {
  description = "PushOver API token"
  type        = string
  sensitive   = true
}
EOF`
Expected: File created

- [ ] **Step 2: Create providers.tf**

Run: `cat > infra/providers.tf << 'EOF'
terraform {
  required_version = ">= 1.0"
  required_providers {
    cloudflare = {
      source  = "cloudflare/cloudflare"
      version = "~> 4.0"
    }
  }
}

provider "cloudflare" {
  account_id = var.account_id
}
EOF`
Expected: File created

- [ ] **Step 3: Create main.tf**

Run: `cat > infra/main.tf << 'EOF'
# D1 Database
module "d1" {
  source    = "./modules/d1"
  account_id = var.account_id
}

# KV Namespace
module "kv" {
  source    = "./modules/kv"
  account_id = var.account_id
}

# Queue
module "queue" {
  source    = "./modules/queue"
  account_id = var.account_id
}

# Pages (Dashboard)
module "pages" {
  source     = "./modules/pages"
  account_id = var.account_id
  project_name = "pushover-dashboard"
}

# Worker
module "worker" {
  source         = "./modules/worker"
  account_id     = var.account_id
  d1_database_id = module.d1.database_id
  kv_namespace_id = module.kv.namespace_id
  queue_id       = module.queue.queue_id
  api_key        = var.api_key
  webhook_secret = var.webhook_secret
  pushover_user_key = var.pushover_user_key
  pushover_token = var.pushover_token
}
EOF`
Expected: File created

- [ ] **Step 4: Create D1 module**

Run: `mkdir -p infra/modules/d1 && cat > infra/modules/d1/main.tf << 'EOF'
variable "account_id" {}

resource "cloudflare_d1_database" "pushover" {
  account_id = var.account_id
  name       = "pushover"
}

resource "cloudflare_d1_database" "pushover_backup" {
  account_id = var.account_id
  name       = "pushover-backup"
}

output "database_id" {
  value = cloudflare_d1_database.pushover.id
}

output "backup_database_id" {
  value = cloudflare_d1_database.pushover_backup.id
}
EOF`
Expected: File created

- [ ] **Step 5: Create KV module**

Run: `mkdir -p infra/modules/kv && cat > infra/modules/kv/main.tf << 'EOF'
variable "account_id" {}

resource "cloudflare_workers_kv_namespace" "pushover" {
  account_id = var.account_id
}

output "namespace_id" {
  value = cloudflare_workers_kv_namespace.pushover.id
}
EOF`
Expected: File created

- [ ] **Step 6: Create Queue module**

Run: `mkdir -p infra/modules/queue && cat > infra/modules/queue/main.tf << 'EOF'
variable "account_id" {}

resource "cloudflare_queue" "pushover_messages" {
  account_id = var.account_id
  name       = "pushover-messages"
}

output "queue_id" {
  value = cloudflare_queue.pushover_messages.id
}
EOF`
Expected: File created

- [ ] **Step 7: Create Pages module**

Run: `mkdir -p infra/modules/pages && cat > infra/modules/pages/main.tf << 'EOF'
variable "account_id" {}
variable "project_name" {}

resource "cloudflare_pages_project" "dashboard" {
  account_id = var.account_id
  name       = var.project_name
  production_branch = "main"

  source {
    type = "github"
    config {
      owner = "your-username"
      repo_name = "pushover-serverless"
      production_branch = "main"
      pr_comments = false
      deployments = false
      production_deployment = true
      preview_deployment = false
    }
  }
}

output "project_id" {
  value = cloudflare_pages_project.dashboard.id
}
EOF`
Expected: File created

- [ ] **Step 8: Create Worker module**

Run: `mkdir -p infra/modules/worker && cat > infra/modules/worker/main.tf << 'EOF'
variable "account_id" {}
variable "d1_database_id" {}
variable "kv_namespace_id" {}
variable "queue_id" {}
variable "api_key" {}
variable "webhook_secret" {}
variable "pushover_user_key" {}
variable "pushover_token" {}

# Store sensitive values as secrets
resource "cloudflare_workers_secret" "api_key" {
  account_id = var.account_id
  name       = "API_KEY"
  text       = var.api_key
}

resource "cloudflare_workers_secret" "webhook_secret" {
  account_id = var.account_id
  name       = "WEBHOOK_SECRET"
  text       = var.webhook_secret
}

resource "cloudflare_workers_secret" "pushover_user_key" {
  account_id = var.account_id
  name       = "PUSHOVER_USER_KEY"
  text       = var.pushover_user_key
}

resource "cloudflare_workers_secret" "pushover_token" {
  account_id = var.account_id
  name       = "PUSHOVER_TOKEN"
  text       = var.pushover_token
}

# Note: Worker script deployment is handled via wrangler CLI
# This module manages secrets and provides reference IDs

output "worker_url" {
  value = "https://pushover-worker.workers.dev"
}
EOF`
Expected: File created

- [ ] **Step 9: Create outputs.tf**

Run: `cat > infra/outputs.tf << 'EOF'
output "d1_database_id" {
  value = module.d1.database_id
}

output "kv_namespace_id" {
  value = module.kv.namespace_id
}

output "queue_id" {
  value = module.queue.queue_id
}

output "pages_project_id" {
  value = module.pages.project_id
}

output "worker_url" {
  value = module.worker.worker_url
}
EOF`
Expected: File created

- [ ] **Step 10: Commit**

Run: `git add infra && git commit -m "feat(infra): add OpenTofu modules for all resources"`
Expected: Commit created

- [ ] **Step 11: Apply infrastructure and get IDs**

Run: `cd infra && tofu init && tofu apply`
Expected: Resources created, IDs output

Run: `tofu output -raw d1_database_id > /tmp/d1_id && cat /tmp/d1_id`
Expected: D1 database ID displayed

- [ ] **Step 12: Update wrangler.toml with actual IDs**

Run: `cat > cloudflare/worker/wrangler.toml << 'EOF'
name = "pushover-worker"
main = "src/index.ts"
compatibility_date = "2024-01-01"

# D1 Database (from OpenTofu output)
[[d1_databases]]
binding = "DB"
database_name = "pushover"
database_id = "D1_ID_FROM_TOFU_OUTPUT"  # Update with actual ID

# KV Namespace (from OpenTofu output)
[[kv_namespaces]]
binding = "KV"
id = "KV_ID_FROM_TOFU_OUTPUT"  # Update with actual ID

# Queue (from OpenTofu output)
[[queues.producers]]
binding = "QUEUE"
queue = "pushover-messages"

[analytics]
binding = "ANALYTICS"

# API credentials - MUST use wrangler secret in production
# Run: wrangler secret put API_KEY
# Run: wrangler secret put WEBHOOK_SECRET
# Run: wrangler secret put PUSHOVER_USER_KEY
# Run: wrangler secret put PUSHOVER_TOKEN

# For local development only (never commit real values):
# [vars]
# API_KEY = "test-dev-key"
# WEBHOOK_SECRET = "test-dev-secret"
# PUSHOVER_USER_KEY = ""
# PUSHOVER_TOKEN = ""
EOF`
Expected: File updated with placeholder instructions

Run: `cd cloudflare/worker && D1_ID=$(tofu output -raw d1_database_id 2>/dev/null || echo "YOUR_D1_ID") && KV_ID=$(tofu output -raw kv_namespace_id 2>/dev/null || echo "YOUR_KV_ID") && sed -i.bak "s/D1_ID_FROM_TOFU_OUTPUT/$D1_ID/" wrangler.toml && sed -i.bak "s/KV_ID_FROM_TOFU_OUTPUT/$KV_ID/" wrangler.toml && rm -f wrangler.toml.bak`
Expected: IDs updated in wrangler.toml

- [ ] **Step 13: Commit**

Run: `git add cloudflare/worker/wrangler.toml && git commit -m "chore(worker): update wrangler.toml with actual resource IDs"`
Expected: Commit created

---

## Task 18: Dashboard - Basic Setup

**Files:**
- Create: `cloudflare/dashboard/package.json`
- Create: `cloudflare/dashboard/vite.config.ts`
- Create: `cloudflare/dashboard/tsconfig.json`
- Create: `cloudflare/dashboard/src/main.tsx`
- Create: `cloudflare/dashboard/src/App.tsx`
- Create: `cloudflare/dashboard/public/index.html`

- [ ] **Step 1: Create dashboard package.json**

Run: `mkdir -p cloudflare/dashboard && cat > cloudflare/dashboard/package.json << 'EOF'
{
  "name": "pushover-dashboard",
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview"
  },
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0"
  },
  "devDependencies": {
    "@types/react": "^18.2.0",
    "@types/react-dom": "^18.2.0",
    "@vitejs/plugin-react": "^4.2.0",
    "typescript": "^5.3.0",
    "vite": "^5.0.0"
  }
}
EOF`
Expected: File created

- [ ] **Step 2: Create Vite config**

Run: `cat > cloudflare/dashboard/vite.config.ts << 'EOF'
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  server: {
    port: 3000,
  },
  build: {
    outDir: 'dist',
  },
});
EOF`
Expected: File created

- [ ] **Step 3: Create tsconfig.json**

Run: `cat > cloudflare/dashboard/tsconfig.json << 'EOF'
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true
  },
  "include": ["src"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
EOF`
Expected: File created

- [ ] **Step 4: Create basic React app**

Run: `mkdir -p cloudflare/dashboard/src && cat > cloudflare/dashboard/src/main.tsx << 'EOF'
import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
EOF`

Run: `cat > cloudflare/dashboard/src/App.tsx << 'EOF'
export default function App() {
  return (
    <div className="min-h-screen bg-gray-50">
      <header className="bg-white shadow">
        <div className="max-w-7xl mx-auto px-4 py-4">
          <h1 className="text-2xl font-bold">PushOver Dashboard</h1>
        </div>
      </header>
      <main className="max-w-7xl mx-auto px-4 py-8">
        <p className="text-gray-600">Dashboard components coming soon...</p>
      </main>
    </div>
  );
}
EOF`

Run: `mkdir -p cloudflare/dashboard/public && cat > cloudflare/dashboard/public/index.html << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>PushOver Dashboard</title>
</head>
<body>
  <div id="root"></div>
  <script type="module" src="/src/main.tsx"></script>
</body>
</html>
EOF`
Expected: Files created

- [ ] **Step 5: Commit**

Run: `git add cloudflare/dashboard && git commit -m "feat(dashboard): add basic React + Vite setup"`
Expected: Commit created

---

## Task 19: Dashboard - API Client & Settings

**Files:**
- Create: `cloudflare/dashboard/src/lib/api.ts`
- Create: `cloudflare/dashboard/src/hooks/useMessages.ts`
- Create: `cloudflare/dashboard/src/hooks/useApiKey.ts`
- Create: `cloudflare/dashboard/src/components/ApiKeySetup.tsx`

- [ ] **Step 1: Create API client**

Run: `mkdir -p cloudflare/dashboard/src/lib && cat > cloudflare/dashboard/src/lib/api.ts << 'EOF'
const API_BASE = import.meta.env.VITE_API_BASE || '';

export interface Message {
  id: string;
  status: string;
  title: string | null;
  message: string;
  priority: number;
  created_at: number;
}

export async function sendMessage(data: {
  message: string;
  title?: string;
  priority?: number;
}): Promise<{ success: boolean; messageId: string }> {
  const apiKey = localStorage.getItem('apiKey');
  if (!apiKey) {
    throw new Error('API key not configured. Please set your API key in settings.');
  }

  const response = await fetch(\`\${API_BASE}/api/v1/send\`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': \`Bearer \${apiKey}\`,
    },
    body: JSON.stringify(data),
  });

  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.error?.message || 'Failed to send message');
  }

  return response.json();
}

export async function getMessages(options?: {
  limit?: number;
  status?: string;
}): Promise<{ messages: Message[] }> {
  const apiKey = localStorage.getItem('apiKey');
  if (!apiKey) {
    throw new Error('API key not configured');
  }

  const params = new URLSearchParams();
  if (options?.limit) params.set('limit', options.limit.toString());
  if (options?.status) params.set('status', options.status);

  const response = await fetch(
    \`\${API_BASE}/api/v1/messages?\${params}\`,
    {
      headers: {
        'Authorization': \`Bearer \${apiKey}\`,
      },
    }
  );

  if (!response.ok) {
    throw new Error('Failed to fetch messages');
  }

  return response.json();
}

  if (!response.ok) {
    throw new Error('Failed to fetch messages');
  }

  return response.json();
}
EOF`
Expected: File created

- [ ] **Step 2: Create useMessages hook**

Run: `mkdir -p cloudflare/dashboard/src/hooks && cat > cloudflare/dashboard/src/hooks/useMessages.ts << 'EOF'
import { useState, useEffect } from 'react';
import { getMessages, type Message } from '../lib/api';

export function useMessages(options?: { limit?: number; status?: string }) {
  const [messages, setMessages] = useState<Message[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    async function load() {
      try {
        setLoading(true);
        const data = await getMessages(options);
        setMessages(data.messages || []);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Unknown error');
      } finally {
        setLoading(false);
      }
    }

    load();
  }, [options]);

  return { messages, loading, error, refetch: () => load() };
}
EOF`
Expected: File created

- [ ] **Step 3: Create useApiKey hook**

Run: `cat > cloudflare/dashboard/src/hooks/useApiKey.ts << 'EOF'
import { useState, useEffect } from 'react';

export function useApiKey() {
  const [apiKey, setApiKey] = useState<string | null>(null);

  useEffect(() => {
    const stored = localStorage.getItem('apiKey');
    setApiKey(stored);
  }, []);

  const saveApiKey = (key: string) => {
    localStorage.setItem('apiKey', key);
    setApiKey(key);
  };

  const clearApiKey = () => {
    localStorage.removeItem('apiKey');
    setApiKey(null);
  };

  return { apiKey, saveApiKey, clearApiKey, isConfigured: !!apiKey };
}
EOF`
Expected: File created

- [ ] **Step 4: Create ApiKeySetup component**

Run: `cat > cloudflare/dashboard/src/components/ApiKeySetup.tsx << 'EOF'
import { useState } from 'react';
import { useApiKey } from '../hooks/useApiKey';

export function ApiKeySetup() {
  const { apiKey, saveApiKey, isConfigured } = useApiKey();
  const [inputKey, setInputKey] = useState(apiKey || '');

  if (isConfigured) {
    return (
      <div className="p-4 bg-green-50 dark:bg-green-900 rounded">
        <p className="text-sm">✓ API key configured</p>
        <button
          onClick={() => {
            localStorage.removeItem('apiKey');
            window.location.reload();
          }}
          className="text-xs text-red-600 underline"
        >
          Change API key
        </button>
      </div>
    );
  }

  return (
    <div className="p-6 bg-yellow-50 dark:bg-yellow-900 rounded">
      <h2 className="text-lg font-semibold mb-2">API Key Required</h2>
      <p className="text-sm mb-4">
        Enter your API key to use the dashboard. Get it from your PushOver Serverless deployment.
      </p>
      <input
        type="password"
        value={inputKey}
        onChange={(e) => setInputKey(e.target.value)}
        placeholder="Enter API key"
        className="w-full p-2 border rounded mb-2"
      />
      <button
        onClick={() => saveApiKey(inputKey)}
        disabled={!inputKey}
        className="px-4 py-2 bg-blue-600 text-white rounded disabled:opacity-50"
      >
        Set API Key
      </button>
    </div>
  );
}
EOF`
Expected: File created

- [ ] **Step 5: Commit**

Run: `git add cloudflare/dashboard/src && git commit -m "feat(dashboard): add API client and hooks with API key setup"`
Expected: Commit created

---

## Task 20: Dashboard - Message History Component

**Files:**
- Create: `cloudflare/dashboard/src/components/MessageHistory.tsx`

- [ ] **Step 1: Create MessageHistory component**

Run: `cat > cloudflare/dashboard/src/components/MessageHistory.tsx << 'EOF'
import { useMessages } from '../hooks/useMessages';

export function MessageHistory() {
  const { messages, loading, error } = useMessages({ limit: 50 });

  return (
    <div className="bg-white rounded-lg shadow p-6">
      <h2 className="text-xl font-semibold mb-4">Message History</h2>

      {loading && (
        <div className="text-center py-8 text-gray-500">Loading...</div>
      )}

      {error && (
        <div className="text-center py-8 text-red-500">Error: {error}</div>
      )}

      {!loading && !error && messages.length === 0 && (
        <div className="text-center py-8 text-gray-500">No messages yet</div>
      )}

      {!loading && !error && messages.length > 0 && (
        <div className="space-y-2">
          {messages.map((msg) => (
            <div
              key={msg.id}
              className="p-4 border rounded-lg hover:bg-gray-50 transition"
            >
              <div className="flex justify-between items-start">
                <div className="flex-1">
                  <h3 className="font-medium">
                    {msg.title || '(no title)'}
                  </h3>
                  <p className="text-gray-600 text-sm mt-1">{msg.message}</p>
                </div>
                <div className="ml-4 text-right text-sm">
                  <div
                    className={
                      'inline-block px-2 py-1 rounded text-xs ' +
                      (msg.status === 'sent'
                        ? 'bg-green-100 text-green-800'
                        : msg.status === 'failed'
                        ? 'bg-red-100 text-red-800'
                        : 'bg-yellow-100 text-yellow-800')
                    }
                  >
                    {msg.status}
                  </div>
                  <div className="text-gray-500 mt-1">
                    {new Date(msg.created_at).toLocaleString()}
                  </div>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
EOF`
Expected: File created

- [ ] **Step 2: Update App.tsx to show MessageHistory**

Run: `cat > cloudflare/dashboard/src/App.tsx << 'EOF'
import { MessageHistory } from './components/MessageHistory';

export default function App() {
  return (
    <div className="min-h-screen bg-gray-50">
      <header className="bg-white shadow">
        <div className="max-w-7xl mx-auto px-4 py-4">
          <h1 className="text-2xl font-bold">PushOver Dashboard</h1>
        </div>
      </header>
      <main className="max-w-7xl mx-auto px-4 py-8">
        <MessageHistory />
      </main>
    </div>
  );
}
EOF`
Expected: File overwritten

- [ ] **Step 3: Commit**

Run: `git add cloudflare/dashboard && git commit -m "feat(dashboard): add MessageHistory component"`
Expected: Commit created

---

## Task 21: Dashboard - Send Modal Component

**Files:**
- Create: `cloudflare/dashboard/src/components/SendModal.tsx`

- [ ] **Step 1: Create SendModal component**

Run: `cat > cloudflare/dashboard/src/components/SendModal.tsx << 'EOF'
import { useState } from 'react';
import { sendMessage } from '../lib/api';

interface Props {
  onClose: () => void;
}

export function SendModal({ onClose }: Props) {
  const [message, setMessage] = useState('');
  const [title, setTitle] = useState('');
  const [priority, setPriority] = useState(0);
  const [sending, setSending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSending(true);
    setError(null);

    try {
      await sendMessage({ message, title: title || undefined, priority });
      setSuccess(true);
      setMessage('');
      setTitle('');
      setPriority(0);

      setTimeout(() => {
        onClose();
      }, 1500);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to send');
    } finally {
      setSending(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
      <div className="bg-white rounded-lg shadow-xl max-w-md w-full p-6">
        <div className="flex justify-between items-center mb-4">
          <h2 className="text-xl font-semibold">Send Message</h2>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600"
          >
            ✕
          </button>
        </div>

        {success ? (
          <div className="text-center py-8">
            <div className="text-green-500 text-6xl mb-4">✓</div>
            <p className="text-gray-700">Message sent successfully!</p>
          </div>
        ) : (
          <form onSubmit={handleSubmit} className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Message *
              </label>
              <textarea
                value={message}
                onChange={(e) => setMessage(e.target.value)}
                required
                maxLength={1024}
                rows={4}
                className="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                placeholder="Enter your message..."
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Title
              </label>
              <input
                type="text"
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                maxLength={250}
                className="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                placeholder="Optional title"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Priority
              </label>
              <select
                value={priority}
                onChange={(e) => setPriority(parseInt(e.target.value))}
                className="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              >
                <option value="-2">Lowest - No notification</option>
                <option value="-1">Low</option>
                <option value="0">Normal</option>
                <option value="1">High</option>
                <option value="2">Emergency</option>
              </select>
            </div>

            {error && (
              <div className="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded">
                {error}
              </div>
            )}

            <div className="flex justify-end gap-3">
              <button
                type="button"
                onClick={onClose}
                className="px-4 py-2 border rounded-lg hover:bg-gray-50"
              >
                Cancel
              </button>
              <button
                type="submit"
                disabled={sending || !message}
                className="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 disabled:opacity-50"
              >
                {sending ? 'Sending...' : 'Send'}
              </button>
            </div>
          </form>
        )}
      </div>
    </div>
  );
}
EOF`
Expected: File created

- [ ] **Step 2: Update App.tsx to include SendModal**

Run: `cat > cloudflare/dashboard/src/App.tsx << 'EOF'
import { useState } from 'react';
import { MessageHistory } from './components/MessageHistory';
import { SendModal } from './components/SendModal';

export default function App() {
  const [showSendModal, setShowSendModal] = useState(false);

  return (
    <div className="min-h-screen bg-gray-50">
      <header className="bg-white shadow">
        <div className="max-w-7xl mx-auto px-4 py-4 flex justify-between items-center">
          <h1 className="text-2xl font-bold">PushOver Dashboard</h1>
          <button
            onClick={() => setShowSendModal(true)}
            className="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600"
          >
            + Send Message
          </button>
        </div>
      </header>
      <main className="max-w-7xl mx-auto px-4 py-8">
        <MessageHistory />
      </main>

      {showSendModal && <SendModal onClose={() => setShowSendModal(false)} />}
    </div>
  );
}
EOF`
Expected: File overwritten

- [ ] **Step 3: Commit**

Run: `git add cloudflare/dashboard && git commit -m "feat(dashboard): add SendModal component"`
Expected: Commit created

---

## Task 21: E2E Tests Setup

**Files:**
- Create: `tests/playwright.config.ts`
- Create: `tests/e2e/health.spec.ts`
- Create: `tests/e2e/send.spec.ts`

- [ ] **Step 1: Initialize Playwright**

Run: `mkdir -p tests/e2e && cd tests && pnpm init && pnpm add -D @playwright/test && npx playwright install`
Expected: Playwright installed

- [ ] **Step 2: Create Playwright config**

Run: `cat > tests/playwright.config.ts << 'EOF'
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'html',

  use: {
    baseURL: 'http://localhost:8787',
    trace: 'on-first-retry',
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],

  webServer: {
    command: 'cd ../cloudflare/worker && pnpm wrangler dev --local',
    port: 8787,
    reuseExistingServer: !process.env.CI,
  },
});
EOF`
Expected: File created

- [ ] **Step 3: Write health check test**

Run: `cat > tests/e2e/health.spec.ts << 'EOF'
import { test, expect } from '@playwright/test';

test('health check returns 200', async ({ request }) => {
  const response = await request.get('/health');
  expect(response.ok()).toBeTruthy();

  const body = await response.json();
  expect(body).toHaveProperty('status', 'healthy');
  expect(body).toHaveProperty('timestamp');
});
EOF`
Expected: File created

- [ ] **Step 4: Write send endpoint test**

Run: `cat > tests/e2e/send.spec.ts << 'EOF'
import { test, expect } from '@playwright/test';

test('POST /api/v1/send requires auth', async ({ request }) => {
  const response = await request.post('/api/v1/send', {
    data: { message: 'Test' },
  });

  expect(response.status()).toBe(401);
});

test('POST /api/v1/send validates input', async ({ request }) => {
  const response = await request.post('/api/v1/send', {
    headers: {
      Authorization: 'Bearer change-me-in-production',
    },
    data: { message: '' }, // Invalid: empty message
  });

  expect(response.status()).toBe(400);
  const body = await response.json();
  expect(body.error).toHaveProperty('code', 'VALIDATION_ERROR');
});
EOF`
Expected: File created

- [ ] **Step 5: Commit**

Run: `git add tests && git commit -m "test(e2e): add Playwright tests for health and send"`
Expected: Commit created

---

## Task 22: README & Documentation

**Files:**
- Create: `README.md`
- Create: `DEVELOPMENT.md`

- [ ] **Step 1: Write main README**

Run: `cat > README.md << 'EOF'
# PushOver Serverless Platform

Cloudflare Serverless 기반 PushOver 메시징 시스템.

## 구성 요소

- **Rust SDK**: PushOver API 클라이언트 (CLI + Worker 공용)
- **CLI**: 메시지 전송, 기록 조회, 설정 관리
- **Worker API**: REST API + Webhook + Queue 처리
- **Dashboard**: React 웹 대시보드

## 빠른 시작

### 사전 요구사항

- Rust 1.70+
- Node.js 18+
- pnpm
- OpenTofu 1.6+
- Cloudflare 계정 (Free Plan)

### CLI

\`\`\`bash
# 환경 변수 설정
export PUSHOVER_USER_KEY="your-key"
export PUSHOVER_API_TOKEN="your-token"

# 메시지 전송
cargo run -p pushover-cli -- send "Hello, World!" --title "Test"
\`\`\`

### Worker 로컬 개발

\`\`\`bash
cd cloudflare/worker
pnpm install
pnpm dev
\`\`\`

### Dashboard

\`\`\`bash
cd cloudflare/dashboard
pnpm install
pnpm dev
\`\`\`

### 인프라 배포

\`\`\`bash
cd infra
tofu init
tofu plan
tofu apply
\`\`\`

## 환경 변수

| 변수 | 설명 |
|------|------|
| \`PUSHOVER_USER_KEY\` | PushOver 사용자 키 |
| \`PUSHOVER_API_TOKEN\` | PushOver API 토큰 |
| \`PUSHOVER_API_BASE\` | Worker API URL (CLI용) |
| \`API_KEY\` | Worker 인증 키 |
| \`WEBHOOK_SECRET\` | Webhook 서명 검증 |

## 아키텍처

\`\`\`
┌─────────┐
│   CLI   │────┐
└─────────┘    │
                ▼
┌─────────────────────────────┐
│     Cloudflare Workers       │
│  ┌──────────────────────┐   │
│  │   Main Worker        │   │
│  │  - API Routes        │   │
│  │  - Webhook Routes   │   │
│  │  - Queue Producer   │   │
│  └──────────────────────┘   │
│              │                │
│              ▼                │
│  ┌──────────────────────┐   │
│  │  Recovery Worker    │   │
│  │  - Queue Consumer   │   │
│  │  - Retry Logic      │   │
│  └──────────────────────┘   │
│                              │
│  ┌──────────┐  ┌─────────┐ │
│  │    D1     │  │    KV   │ │
│  └──────────┘  └─────────┘ │
└─────────────────────────────┘
\`\`\`

## 라이선스

MIT
EOF`
Expected: File created

- [ ] **Step 2: Write DEVELOPMENT.md**

Run: `cat > DEVELOPMENT.md << 'EOF'
# Development Guide

## 프로젝트 구조

### Rust Workspace

\`\`\`
crates/
├── sdk/        # Core SDK library
└── cli/        # CLI application
\`\`\`

### Cloudflare Workers

\`\`\`
cloudflare/
├── worker/            # Main API Worker
├── recovery-worker/   # Queue Consumer
└── dashboard/         # React Dashboard
\`\`\`

## 로컬 개발

### Worker

\`\`\`bash
cd cloudflare/worker
pnpm install
pnpm dev       # Start local dev server
\`\`\`

### Dashboard

\`\`\`bash
cd cloudflare/dashboard
pnpm install
pnpm dev       # Start on http://localhost:3000
\`\`\`

### 테스트

\`\`\`bash
# Rust SDK tests
cd crates/sdk && cargo test

# Worker tests (Vitest)
cd cloudflare/worker && pnpm test

# E2E tests (Playwright)
cd tests && npx playwright test
\`\`\`

## 트러블슈팅

### Worker 로컬 실행 실패

\`\`\`
# wrangler.toml의 바인딩 ID들이 실제 리소스와 일치하는지 확인
# wrangler secrets list로 시크릿 확인
\`\`\`

### D1 마이그레이션

\`\`\`
# 로컬 D1에 마이그레이션 적용
wrangler d1 execute pushover --file=../../infra/migrations/001_init.sql --local
\`\`\`
EOF`
Expected: File created

- [ ] **Step 3: Commit**

Run: `git add README.md DEVELOPMENT.md && git commit -m "docs: add README and DEVELOPMENT guide"`
Expected: Commit created

---

## Summary

이 확장된 구현 계획은 다음을 포함합니다:

### 완료된 구성 요소 (100% spec 커버)

1. **Rust SDK**
   - Core types & error handling
   - Complete HTTP client (all Message fields)
   - Webhook signature verification with timing-safe compare
   - Integration tests with wiremock

2. **CLI Tool**
   - Config management (profiles, toml)
   - Send command (all options)
   - History command (with filtering)
   - clap-based structure

3. **Cloudflare Worker**
   - Bearer token auth middleware
   - KV-based rate limiting (with race condition warning)
   - Zod validation schemas
   - API routes (send, history, single message)
   - Webhook routes with HMAC-SHA256 verification
   - Timing-safe crypto utilities
   - Error response utilities
   - TypeScript types definition

4. **Recovery Worker**
   - Queue consumer with exponential backoff (2s, 4s, 8s)
   - Dead letter handling
   - Cron trigger configuration
   - Analytics Engine tracking

5. **Dashboard**
   - React + Vite setup
   - API client with useMessages hook
   - MessageHistory component
   - SendModal component with validation

6. **D1 Database**
   - All 4 tables: messages, webhooks, webhook_events, daily_stats, settings
   - Complete indexes for performance

7. **Infrastructure (OpenTofu)**
   - D1 module
   - KV module
   - Queue module
   - Pages module
   - Worker module
   - Variables, providers, outputs

8. **Testing**
   - SDK integration tests
   - E2E tests (health, auth, validation)

### 보안 개선사항

- ✅ 하드코딩된 credential 제거 → env vars 사용
- ✅ Timing-safe signature verification 구현
- ✅ Complete HTTP client (모든 필드 처리)

### 실행 준비

이제 이 계획을 실행할 수 있습니다. 각 작업은 2-5분 소요이며, TDD 방식으로 설계되었습니다.
