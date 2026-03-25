mod commands;
mod config;

use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(name = "pushover")]
#[command(about = "PushOver CLI tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Send a push notification
    Send {
        /// The message content
        message: String,

        /// Message title
        #[arg(short, long)]
        title: Option<String>,

        /// User key (overrides config)
        #[arg(short = 'u', long)]
        user: Option<String>,

        /// API token (overrides config)
        #[arg(short = 't', long)]
        token: Option<String>,

        /// Target device name
        #[arg(short, long)]
        device: Option<String>,

        /// Message priority (-2 to 2)
        #[arg(short, long)]
        priority: Option<i8>,

        /// Supplementary URL
        #[arg(long)]
        url: Option<String>,

        /// URL title
        #[arg(long)]
        url_title: Option<String>,

        /// Notification sound
        #[arg(long)]
        sound: Option<String>,

        /// Unix timestamp
        #[arg(short = 'T', long)]
        timestamp: Option<i64>,

        /// Enable HTML formatting
        #[arg(long)]
        html: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Send { message, title, user, token, device, priority, url, url_title, sound, timestamp, html } => {
            commands::send::execute(message, title, user, token, device, priority, url, url_title, sound, timestamp, html).await?;
        }
    }

    Ok(())
}
