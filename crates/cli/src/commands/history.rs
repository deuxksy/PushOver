use anyhow::Result;
use crate::config::Config;

pub async fn execute(
    limit: Option<usize>,
    _user: Option<String>,
    _token: Option<String>,
) -> Result<()> {
    let config = Config::load()?;
    let _profile = config.get_default_profile()
        .ok_or_else(|| anyhow::anyhow!("No default profile configured"))?;

    // TODO: Fetch from Worker API
    let records = [
        ("msg_001", "2024-01-01T00:00:00Z", "delivered", "Test", "Test message"),
        ("msg_002", "2024-01-01T01:00:00Z", "acknowledged", "Alert", "System alert"),
    ];

    let limit = limit.unwrap_or(10);

    println!("📜 Message History (last {}):\n", limit);
    println!("{:<12} {:<20} {:<12} Message", "ID", "Sent At", "Status");
    println!("{}", "-".repeat(75));

    for (id, sent_at, status, title, message) in records.iter().take(limit) {
        println!(
            "{:<12} {:<20} {:<12} {}: {}",
            id, sent_at, status, title, message
        );
    }

    Ok(())
}
