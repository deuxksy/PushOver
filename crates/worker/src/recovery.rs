use worker::*;
use crate::db::Db;
use crate::pushover::PushOverClient;

const MAX_RETRY_ATTEMPTS: u32 = 3;

pub async fn handle_failed_messages(
    _event: ScheduledEvent,
    env: Env,
    _ctx: ScheduleContext,
) -> Result<Response> {
    let db = Db::from_env(&env)?;
    let client = PushOverClient::from_env(&env)?;

    let failed = db.get_failed_deliveries(MAX_RETRY_ATTEMPTS).await?;

    for delivery in failed {
        let msg = match db.get_message_by_id(&delivery.message_id).await? {
            Some(m) => m,
            None => {
                let _ = db.delete_failed_delivery(&delivery.message_id).await;
                continue;
            }
        };

        let api_token = match msg.api_token.as_deref() {
            Some(t) if !t.is_empty() => t,
            _ => {
                let _ = db.update_message_status(&msg.id, "permanently_failed").await;
                let _ = db.delete_failed_delivery(&msg.id).await;
                continue;
            }
        };

        // PushOver API 재시도
        let retry = msg.retry.map(|r| r as u32);
        let expire = msg.expire.map(|e| e as u32);

        match client.send_message(
            api_token,
            &msg.user_key,
            &msg.message,
            msg.title.as_deref(),
            Some(msg.priority),
            msg.sound.as_deref(),
            msg.device.as_deref(),
            msg.url.as_deref(),
            msg.url_title.as_deref(),
            Some(msg.html != 0),
            retry,
            expire,
        ).await {
            Ok(result) => {
                db.update_message_status(&msg.id, "sent").await?;
                if let Some(ref receipt) = result.receipt {
                    db.update_message_receipt(&msg.id, receipt).await?;
                }
                db.delete_failed_delivery(&msg.id).await?;
            }
            Err(e) => {
                db.upsert_failed_delivery(&msg.id, &e.to_string()).await?;

                if delivery.attempt_count + 1 >= MAX_RETRY_ATTEMPTS as i32 {
                    db.update_message_status(&msg.id, "permanently_failed").await?;
                    db.delete_failed_delivery(&msg.id).await?;
                }
            }
        }
    }

    Response::ok("Recovery completed")
}

pub async fn retry_message(
    message_id: &str,
    env: &Env,
) -> Result<bool> {
    let db = Db::from_env(env)?;
    let client = PushOverClient::from_env(env)?;

    let msg = match db.get_message_by_id(message_id).await? {
        Some(m) => m,
        None => return Ok(false),
    };

    let api_token = match msg.api_token.as_deref() {
        Some(t) if !t.is_empty() => t,
        _ => return Ok(false),
    };

    let retry = msg.retry.map(|r| r as u32);
    let expire = msg.expire.map(|e| e as u32);

    match client.send_message(
        api_token,
        &msg.user_key,
        &msg.message,
        msg.title.as_deref(),
        Some(msg.priority),
        msg.sound.as_deref(),
        msg.device.as_deref(),
        msg.url.as_deref(),
        msg.url_title.as_deref(),
        Some(msg.html != 0),
        retry,
        expire,
    ).await {
        Ok(result) => {
            db.update_message_status(&msg.id, "sent").await?;
            if let Some(ref receipt) = result.receipt {
                db.update_message_receipt(&msg.id, receipt).await?;
            }
            db.delete_failed_delivery(&msg.id).await?;
            Ok(true)
        }
        Err(e) => {
            db.upsert_failed_delivery(&msg.id, &e.to_string()).await?;
            Ok(false)
        }
    }
}
