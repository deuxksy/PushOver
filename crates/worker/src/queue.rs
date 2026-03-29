use worker::*;
use crate::types::QueueMessage;
use crate::pushover::PushOverClient;
use crate::db::Db;
use crate::kv::Kv;

/// 단일 Queue 메시지 소비
pub async fn consume_message(
    msg: &QueueMessage,
    env: &Env,
) -> Result<()> {
    let client = PushOverClient::from_env(env)?;
    let db = Db::from_env(env)?;
    let kv = Kv::new(env)?;

    let result = match client.send_message(
        &msg.pushover_token,
        &msg.user,
        &msg.message,
        msg.title.as_deref(),
        msg.priority,
        msg.sound.as_deref(),
        msg.device.as_deref(),
        msg.url.as_deref(),
        msg.url_title.as_deref(),
        msg.html,
        msg.retry,
        msg.expire,
    ).await {
        Ok(r) => r,
        Err(e) => {
            // 실패: KV에 메시지 본문 백업 + D1 failed_deliveries 기록
            let body = serde_json::to_string(&msg).unwrap_or_default();
            let _ = kv.backup_failed_message(&msg.id, &body).await;
            let _ = db.upsert_failed_delivery(&msg.id, &e.to_string()).await;

            // D1 메시지 상태를 failed로 업데이트
            let _ = db.update_message_status(&msg.id, "failed").await;

            return Err(e);
        }
    };

    // 성공: D1 메시지 상태 업데이트 (status=sent)
    db.update_message_status(&msg.id, "sent").await?;

    if let Some(ref receipt) = result.receipt {
        db.update_message_receipt(&msg.id, receipt).await?;
    }

    Ok(())
}

/// Batch 메시지 소비 (Cloudflare Queue 배치 처리)
pub async fn consume_batch(
    messages: &[QueueMessage],
    env: &Env,
) {
    for msg in messages {
        if let Err(e) = consume_message(msg, env).await {
            console_error!("Queue consumer error for msg {}: {}", msg.id, e);
        }
    }
}
