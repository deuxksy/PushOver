use worker::*;
use crate::types::{ErrorResponse, WebhookMessage};
use crate::middleware::{extract_token, unauthorized_response, with_cors};
use crate::crypto::{verify_signature, generate_signature};
use crate::pushover::PushOverClient;
use crate::db::Db;

pub async fn root(
    _req: Request,
    _ctx: RouteContext<()>,
) -> worker::Result<Response> {
    Response::ok("PushOver API")
}

pub async fn health(
    _req: Request,
    _ctx: RouteContext<()>,
) -> worker::Result<Response> {
    Response::ok("OK")
}

pub async fn not_found(
    _req: Request,
    _ctx: RouteContext<()>,
) -> worker::Result<Response> {
    Ok(Response::from_json(&ErrorResponse::not_found("Endpoint not found"))?)
}

/// POST /api/v1/messages - PushOver API로 메시지 발송
pub async fn send_message(
    mut req: Request,
    ctx: RouteContext<()>,
) -> worker::Result<Response> {
    let token = match extract_token(&req) {
        Ok(t) => t,
        Err(_) => return unauthorized_response("Missing or invalid Authorization header"),
    };

    let body: serde_json::Value = req.json().await?;

    let user = body.get("user")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::from("Missing 'user' field"))?;
    let message = body.get("message")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::from("Missing 'message' field"))?;

    let title = body.get("title").and_then(|v| v.as_str());
    let priority = body.get("priority").and_then(|v| v.as_i64()).map(|p| p as i32);
    let sound = body.get("sound").and_then(|v| v.as_str());
    let device = body.get("device").and_then(|v| v.as_str());
    let url = body.get("url").and_then(|v| v.as_str());
    let url_title = body.get("url_title").and_then(|v| v.as_str());
    let html = body.get("html").and_then(|v| v.as_bool());
    let retry = body.get("retry").and_then(|v| v.as_u64()).map(|r| r as u32);
    let expire = body.get("expire").and_then(|v| v.as_u64()).map(|e| e as u32);

    // PushOver API 호출
    let client = PushOverClient::from_env(&ctx.env)?;
    let result = match client.send_message(
        &token, user, message, title, priority, sound,
        device, url, url_title, html, retry, expire,
    ).await {
        Ok(r) => r,
        Err(e) => {
            // 실패 시 D1에 기록 (non-blocking)
            let msg_id = uuid::Uuid::new_v4().to_string();
            if let Ok(db) = Db::new(&ctx) {
                let _ = db.insert_message(
                    &msg_id, user, message, title,
                    priority.unwrap_or(0), sound, device,
                    url, url_title, html.unwrap_or(false),
                    "failed", None, Some(&token),
                ).await;
                let _ = db.upsert_failed_delivery(&msg_id, &e.to_string()).await;
            }
            return Ok(Response::from_json(&serde_json::json!({
                "status": "error",
                "message": format!("PushOver API error: {}", e),
                "request": uuid::Uuid::new_v4().to_string()
            }))?.with_status(502));
        }
    };

    // D1에 메시지 저장
    let msg_id = uuid::Uuid::new_v4().to_string();
    let db = Db::new(&ctx)?;
    db.insert_message(
        &msg_id, user, message, title,
        priority.unwrap_or(0), sound, device,
        url, url_title, html.unwrap_or(false),
        "sent", result.receipt.as_deref(), Some(&token),
    ).await?;

    // receipt가 있으면 별도 업데이트
    if let Some(ref receipt) = result.receipt {
        db.update_message_receipt(&msg_id, receipt).await?;
    }

    Ok(with_cors(Response::from_json(&serde_json::json!({
        "status": "success",
        "request": result.request,
        "receipt": result.receipt
    }))?))
}

/// GET /api/v1/messages/:receipt/status - 메시지 수신 상태 조회
pub async fn get_status(
    req: Request,
    ctx: RouteContext<()>,
) -> worker::Result<Response> {
    let _token = match extract_token(&req) {
        Ok(t) => t,
        Err(_) => return unauthorized_response("Missing or invalid Authorization header"),
    };

    let receipt = ctx.param("receipt")
        .ok_or_else(|| Error::from("Missing receipt parameter"))?;

    let db = Db::new(&ctx)?;
    let msg = db.get_message_by_receipt(receipt).await?
        .ok_or_else(|| Error::from("Message not found"))?;

    Ok(with_cors(Response::from_json(&serde_json::json!({
        "status": msg.status,
        "receipt": msg.receipt,
        "acknowledged": msg.acknowledged_at.is_some(),
        "delivered_at": msg.delivered_at,
        "acknowledged_at": msg.acknowledged_at,
        "created_at": msg.created_at
    }))?))
}

/// POST /api/v1/webhooks - PushOver callback 수신 (서명 검증 + 상태 업데이트)
pub async fn receive_webhook(
    mut req: Request,
    ctx: RouteContext<()>,
) -> worker::Result<Response> {
    let headers = req.headers();
    let signature = headers.get("X-Pushover-Signature")?
        .ok_or_else(|| Error::from("Missing signature header"))?;

    let body = req.text().await?;

    let secret = ctx.var("WEBHOOK_SECRET")
        .map_err(|_| Error::from("WEBHOOK_SECRET not configured"))?
        .to_string();

    if !verify_signature(&body, &signature, &secret)? {
        return Ok(Response::from_json(&ErrorResponse::unauthorized(
            "Invalid signature"
        ))?.with_status(401));
    }

    let payload: WebhookMessage = serde_json::from_str(&body)
        .map_err(|_| Error::from("Invalid JSON payload"))?;

    // receipt로 메시지 찾아서 상태 업데이트
    let db = Db::new(&ctx)?;
    if let Some(msg) = db.get_message_by_receipt(&payload.id).await? {
        let new_status = if payload.priority == 2 {
            "acknowledged"
        } else {
            "delivered"
        };
        db.update_message_status(&msg.id, new_status).await?;

        if new_status == "acknowledged" {
            db.acknowledge_message(&msg.id).await?;
        }

        // 등록된 webhook에 이벤트 전달
        trigger_webhooks(&db, &ctx, &msg.id, new_status, &msg.user_key).await?;
    }

    Ok(with_cors(Response::from_json(&serde_json::json!({
        "status": "success",
        "message": "Webhook received"
    }))?))
}

/// POST /api/v1/webhooks/register - Webhook 등록
pub async fn register_webhook(
    mut req: Request,
    ctx: RouteContext<()>,
) -> worker::Result<Response> {
    let user_key = match extract_token(&req) {
        Ok(t) => t,
        Err(_) => return unauthorized_response("Missing or invalid Authorization header"),
    };

    let body: serde_json::Value = req.json().await?;

    let url = body.get("url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::from("Missing 'url' field"))?;
    let events = body.get("events")
        .and_then(|v| v.as_str())
        .unwrap_or("delivered,acknowledged,expired");

    let id = format!("wh_{}", uuid::Uuid::new_v4().to_string().replace("-", "")[..20].to_string());
    let secret = generate_signature(&uuid::Uuid::new_v4().to_string(), "webhook-secret-salt")?;

    let db = Db::new(&ctx)?;
    db.insert_webhook(&id, &user_key, url, &secret, events).await?;

    Ok(with_cors(Response::from_json(&serde_json::json!({
        "status": "success",
        "message": "Webhook registered",
        "webhook": {
            "id": id,
            "url": url,
            "secret": secret,
            "events": events
        }
    }))?))
}

/// GET /api/v1/webhooks - Webhook 목록 조회
pub async fn get_webhooks(
    req: Request,
    ctx: RouteContext<()>,
) -> worker::Result<Response> {
    let user_key = match extract_token(&req) {
        Ok(t) => t,
        Err(_) => return unauthorized_response("Missing or invalid Authorization header"),
    };

    let db = Db::new(&ctx)?;
    let webhooks = db.get_webhooks_by_user(&user_key).await?;

    Ok(with_cors(Response::from_json(&serde_json::json!({
        "status": "success",
        "webhooks": webhooks
    }))?))
}

/// DELETE /api/v1/webhooks/:id - Webhook 삭제
pub async fn delete_webhook(
    req: Request,
    ctx: RouteContext<()>,
) -> worker::Result<Response> {
    let user_key = match extract_token(&req) {
        Ok(t) => t,
        Err(_) => return unauthorized_response("Missing or invalid Authorization header"),
    };

    let id = ctx.param("id")
        .ok_or_else(|| Error::from("Missing webhook ID"))?;

    let db = Db::new(&ctx)?;
    let deleted = db.delete_webhook(id, &user_key).await?;

    if deleted {
        Ok(with_cors(Response::from_json(&serde_json::json!({
            "status": "success",
            "message": "Webhook deleted"
        }))?))
    } else {
        Ok(Response::from_json(&ErrorResponse::not_found(
            "Webhook not found or not owned by you"
        ))?.with_status(404))
    }
}

/// 등록된 webhook에 이벤트 전달
async fn trigger_webhooks(
    db: &Db,
    _ctx: &RouteContext<()>,
    message_id: &str,
    event_type: &str,
    user_key: &str,
) -> Result<()> {
    let webhooks = db.get_webhooks_by_user(user_key).await?;

    for wh in webhooks {
        // events 필드가 이벤트 타입을 포함하는지 확인
        if !wh.events.contains(event_type) {
            continue;
        }

        let delivery_id = uuid::Uuid::new_v4().to_string();

        // webhook delivery 기록
        db.insert_webhook_delivery(
            &delivery_id, &wh.id, message_id, event_type,
        ).await?;

        // webhook 전송 시도
        let payload = serde_json::json!({
            "event": event_type,
            "message_id": message_id,
            "timestamp": crate::types::iso_timestamp(),
        });

        let headers = Headers::new();
        headers.set("Content-Type", "application/json")?;
        headers.set("X-Webhook-Signature", &generate_signature(&payload.to_string(), &wh.secret)?)?;

        let mut init = RequestInit::new();
        init.with_method(Method::Post);
        init.with_headers(headers);
        init.with_body(Some(wasm_bindgen::JsValue::from_str(&payload.to_string())));

        match Request::new_with_init(&wh.url, &init) {
            Ok(request) => {
                match Fetch::Request(request).send().await {
                    Ok(mut resp) => {
                        let code = resp.status_code() as i32;
                        let status = if (200..300).contains(&code) { "delivered" } else { "failed" };
                        let body_text = resp.text().await.ok();
                        db.update_webhook_delivery_status(
                            &delivery_id, status, Some(code), body_text.as_deref(),
                        ).await?;
                        db.update_webhook_last_triggered(&wh.id).await?;
                    }
                    Err(e) => {
                        db.update_webhook_delivery_status(
                            &delivery_id, "failed", None, Some(&e.to_string()),
                        ).await?;
                    }
                }
            }
            Err(e) => {
                db.update_webhook_delivery_status(
                    &delivery_id, "failed", None, Some(&e.to_string()),
                ).await?;
            }
        }
    }

    Ok(())
}
