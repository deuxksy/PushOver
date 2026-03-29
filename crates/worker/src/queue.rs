use worker::*;
use base64::Engine;
use crate::types::QueueMessage;
use crate::pushover::PushOverClient;
use crate::db::Db;
use crate::kv::Kv;
use crate::r2::R2;

/// 바이트 매직 넘버로 MIME 타입 추론
fn detect_mime_type(bytes: &[u8]) -> &'static str {
    if bytes.len() >= 4 {
        match &bytes[0..4] {
            [0x89, 0x50, 0x4E, 0x47] => return "image/png",
            _ => {}
        }
    }
    if bytes.len() >= 3 {
        match &bytes[0..3] {
            [0xFF, 0xD8, 0xFF] => return "image/jpeg",
            [0x47, 0x49, 0x46] => return "image/gif",
            _ => {}
        }
    }
    "image/jpeg"
}

/// 단일 Queue 메시지 소비
pub async fn consume_message(
    msg: &QueueMessage,
    env: &Env,
) -> Result<()> {
    let client = PushOverClient::from_env(env)?;
    let db = Db::from_env(env)?;
    let kv = Kv::new(env)?;

    // R2에서 이미지 조회 → base64 인코딩
    let (attachment_base64, attachment_type) = if let Some(ref image_url) = msg.image_url {
        let r2 = R2::new(env)?;
        let key = image_url.trim_start_matches("/images/");
        match r2.get_image(key).await {
            Ok(Some(bytes)) => {
                let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                let mime = detect_mime_type(&bytes);
                console_log!("Image fetched from R2: {} ({} bytes, {})", key, bytes.len(), mime);
                (Some(b64), Some(mime.to_string()))
            }
            Ok(None) => {
                console_error!("Image not found in R2: {}", key);
                (None, None)
            }
            Err(e) => {
                console_error!("R2 fetch error for {}: {}", key, e);
                (None, None)
            }
        }
    } else {
        (None, None)
    };

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
        attachment_base64.as_deref(),
        attachment_type.as_deref(),
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
