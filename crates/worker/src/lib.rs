mod types;
mod crypto;
mod middleware;
mod routes;
mod recovery;
mod utils;
mod pushover;
mod db;

use worker::*;
use middleware::{with_cors, handle_options};
use routes::{root, health, not_found, send_message, get_messages, get_status, receive_webhook, register_webhook, get_webhooks, delete_webhook, register_token};
use recovery::handle_failed_messages;

pub use pushover::PushOverClient;
pub use db::Db;

#[event(scheduled)]
pub async fn scheduled(event: ScheduledEvent, env: Env, ctx: ScheduleContext) {
    handle_failed_messages(event, env, ctx).await.ok();
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    utils::set_panic_hook();

    let router = Router::new()
        .get_async("/", root)
        .get_async("/health", health)
        .post_async("/api/v1/messages", send_message)
        .get_async("/api/v1/messages", get_messages)
        .get_async("/api/v1/messages/:receipt/status", get_status)
        .post_async("/api/v1/webhooks", receive_webhook)
        .post_async("/api/v1/webhooks/register", register_webhook)
        .get_async("/api/v1/webhooks", get_webhooks)
        .delete_async("/api/v1/webhooks/:id", delete_webhook)
        .post_async("/api/v1/tokens/register", register_token)
        .get_async("/404", not_found)
        .options_async("/*path", handle_options);

    let response = match router.run(req, env).await {
        Ok(resp) => resp,
        Err(e) => {
            let body = serde_json::json!({
                "status": "error",
                "message": e.to_string()
            });
            Response::from_json(&body).unwrap_or_else(|_| {
                Response::ok("Internal Server Error").unwrap()
            }).with_status(500)
        }
    };
    Ok(with_cors(response))
}
