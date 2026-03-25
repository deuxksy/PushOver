use worker::*;
use crate::middleware::require_auth;

pub async fn send_message(
    mut req: Request,
    ctx: RouteContext<()>,
) -> Result<Response> {
    // Authenticate request
    let auth_resp = require_auth(&req, &ctx).await?;
    if auth_resp.status_code() != 200 {
        return Ok(auth_resp);
    }

    // Parse request body
    let body: serde_json::Value = req.json().await?;

    // Extract fields (placeholder - will integrate with PushOver API)
    let message = body.get("message")
        .and_then(|v: &serde_json::Value| v.as_str())
        .ok_or_else(|| Error::from("Missing message field"))?;

    // TODO: Call PushOver API to send message
    // TODO: Store in D1 database

    Ok(Response::from_json(&serde_json::json!({
        "status": "success",
        "message": "Message sent",
        "request": message
    }))?)
}

pub async fn get_status(
    req: Request,
    ctx: RouteContext<()>,
) -> Result<Response> {
    // Authenticate request
    let auth_resp = require_auth(&req, &ctx).await?;
    if auth_resp.status_code() != 200 {
        return Ok(auth_resp);
    }

    // TODO: Get receipt ID from query params
    // TODO: Check message status from D1

    Ok(Response::from_json(&serde_json::json!({
        "status": "delivered",
        "acknowledged": true,
        "delivered_at": "2024-01-01T00:00:00Z"
    }))?)
}
