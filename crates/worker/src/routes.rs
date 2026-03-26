use worker::*;
use crate::types::{ErrorResponse, WebhookMessage};
use crate::middleware::require_auth;
use crate::crypto::verify_signature;

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

pub async fn send_message(
    mut req: Request,
    ctx: RouteContext<()>,
) -> worker::Result<Response> {
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
) -> worker::Result<Response> {
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

pub async fn receive_webhook(
    mut req: Request,
    ctx: RouteContext<()>,
) -> worker::Result<Response> {
    // Get signature from headers
    let headers = req.headers();
    let signature = headers.get("X-Pushover-Signature")?
        .ok_or_else(|| Error::from("Missing signature header"))?;

    // Convert String to &str
    let signature_str = signature.as_str();

    // Get raw body as string
    let body = req.text().await?;

    // Get webhook secret from env (placeholder)
    let secret = ctx.var("WEBHOOK_SECRET")
        .map_err(|_| Error::from("WEBHOOK_SECRET not configured"))?
        .to_string();

    // Verify signature
    let is_valid = verify_signature(&body, signature_str, &secret)?;

    if !is_valid {
        return Ok(Response::from_json(&ErrorResponse::unauthorized(
            "Invalid signature"
        ))?);
    }

    // Parse webhook payload
    let _payload: WebhookMessage = serde_json::from_str(&body)
        .map_err(|_| Error::from("Invalid JSON payload"))?;

    // TODO: Store in D1 database

    Ok(Response::from_json(&serde_json::json!({
        "status": "success",
        "message": "Webhook received"
    }))?)
}

pub async fn register_webhook(
    mut req: Request,
    ctx: RouteContext<()>,
) -> worker::Result<Response> {
    let auth_resp = require_auth(&req, &ctx).await?;
    if auth_resp.status_code() != 200 {
        return Ok(auth_resp);
    }

    let body: serde_json::Value = req.json().await?;

    let url = body.get("url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::from("Missing url field"))?;

    // TODO: Store in D1 database
    // TODO: Generate webhook secret

    Ok(Response::from_json(&serde_json::json!({
        "status": "success",
        "message": "Webhook registered",
        "webhook": {
            "id": "wh_123",
            "url": url,
            "secret": "placeholder_secret"
        }
    }))?)
}

pub async fn get_webhooks(
    req: Request,
    ctx: RouteContext<()>,
) -> worker::Result<Response> {
    let auth_resp = require_auth(&req, &ctx).await?;
    if auth_resp.status_code() != 200 {
        return Ok(auth_resp);
    }

    // TODO: Fetch from D1 database

    Ok(Response::from_json(&serde_json::json!({
        "status": "success",
        "webhooks": []
    }))?)
}

pub async fn delete_webhook(
    req: Request,
    ctx: RouteContext<()>,
) -> worker::Result<Response> {
    let auth_resp = require_auth(&req, &ctx).await?;
    if auth_resp.status_code() != 200 {
        return Ok(auth_resp);
    }

    // TODO: Get webhook ID from path params
    // TODO: Delete from D1 database

    Ok(Response::from_json(&serde_json::json!({
        "status": "success",
        "message": "Webhook deleted"
    }))?)
}
