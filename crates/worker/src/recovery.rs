use worker::*;
use crate::db::Db;
use crate::kv::Kv;
use crate::pushover::PushOverClient;
use crate::types::QueueMessage;

const MAX_RETRY_ATTEMPTS: u32 = 3;

pub async fn handle_failed_messages(
    _event: ScheduledEvent,
    env: Env,
    _ctx: ScheduleContext,
) -> Result<Response> {
    let db = Db::from_env(&env)?;
    let kv = Kv::new(&env)?;
    let client = PushOverClient::from_env(&env)?;

    let failed = db.get_failed_deliveries(MAX_RETRY_ATTEMPTS).await?;

    for delivery in failed {
        // KV에서 메시지 본문 복원 (QueueMessage JSON)
        let body = match kv.get_failed_message(&delivery.message_id).await? {
            Some(b) => b,
            None => {
                // KV TTL 만료 → 복원 불가, 영구 실패 처리
                db.update_message_status(&delivery.message_id, "permanently_failed").await?;
                db.delete_failed_delivery(&delivery.message_id).await?;
                continue;
            }
        };

        let msg: QueueMessage = match serde_json::from_str(&body) {
            Ok(m) => m,
            Err(_) => {
                let _ = db.delete_failed_delivery(&delivery.message_id).await;
                continue;
            }
        };

        match client.send_message(
            &msg.pushover_token, &msg.user, &msg.message,
            msg.title.as_deref(), msg.priority, msg.sound.as_deref(),
            msg.device.as_deref(), msg.url.as_deref(), msg.url_title.as_deref(),
            msg.html, msg.retry, msg.expire,
            None, None,
        ).await {
            Ok(result) => {
                db.update_message_status(&delivery.message_id, "sent").await?;
                if let Some(ref receipt) = result.receipt {
                    db.update_message_receipt(&delivery.message_id, receipt).await?;
                }
                db.delete_failed_delivery(&delivery.message_id).await?;
                kv.delete_failed_message(&delivery.message_id).await?;
            }
            Err(e) => {
                db.upsert_failed_delivery(&delivery.message_id, &e.to_string()).await?;

                if delivery.attempt_count + 1 >= MAX_RETRY_ATTEMPTS as i32 {
                    db.update_message_status(&delivery.message_id, "permanently_failed").await?;
                    db.delete_failed_delivery(&delivery.message_id).await?;
                }
            }
        }
    }

    Response::ok("Recovery completed")
}

/// 수동 재시도 (KV 백업에서 복원, fallback으로 D1 조회)
pub async fn retry_message(
    message_id: &str,
    env: &Env,
) -> Result<bool> {
    let db = Db::from_env(env)?;
    let kv = Kv::new(env)?;
    let client = PushOverClient::from_env(env)?;

    // KV에서 백업된 메시지 본문 복원 시도
    let body = match kv.get_failed_message(message_id).await? {
        Some(b) => b,
        None => {
            // KV 없으면 D1에서 직접 조회 (fallback)
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

            return match client.send_message(
                api_token, &msg.user_key, &msg.message,
                msg.title.as_deref(), Some(msg.priority), msg.sound.as_deref(),
                msg.device.as_deref(), msg.url.as_deref(), msg.url_title.as_deref(),
                Some(msg.html != 0), retry, expire,
                None, None,
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
            };
        }
    };

    let msg: QueueMessage = match serde_json::from_str(&body) {
        Ok(m) => m,
        Err(_) => return Ok(false),
    };

    match client.send_message(
        &msg.pushover_token, &msg.user, &msg.message,
        msg.title.as_deref(), msg.priority, msg.sound.as_deref(),
        msg.device.as_deref(), msg.url.as_deref(), msg.url_title.as_deref(),
        msg.html, msg.retry, msg.expire,
        None, None,
    ).await {
        Ok(result) => {
            db.update_message_status(&msg.id, "sent").await?;
            if let Some(ref receipt) = result.receipt {
                db.update_message_receipt(&msg.id, receipt).await?;
            }
            db.delete_failed_delivery(&msg.id).await?;
            kv.delete_failed_message(&msg.id).await?;
            Ok(true)
        }
        Err(e) => {
            db.upsert_failed_delivery(&msg.id, &e.to_string()).await?;
            Ok(false)
        }
    }
}
