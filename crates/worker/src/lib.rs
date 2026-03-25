mod types;
mod crypto;
mod middleware;
mod routes;
mod recovery;
mod utils;

use worker::*;
use types::ErrorResponse;
use middleware::{with_cors, handle_options};
use routes::{send_message, get_status, receive_webhook, register_webhook, get_webhooks, delete_webhook};
use recovery::handle_failed_messages;

#[event(scheduled)]
pub async fn scheduled(event: ScheduledEvent, env: Env, _ctx: worker::Context) -> Result<()> {
    handle_failed_messages(event, env, _ctx).await?;
    Ok(())
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    utils::set_panic_hook();

    let router = Router::new()
        .get("/", |_, _| Response::ok("PushOver API"))
        .get("/health", |_, _| Response::ok("OK"))
        .post_async("/api/v1/messages", send_message)
        .get_async("/api/v1/messages/:receipt/status", get_status)
        .post_async("/api/v1/webhooks", receive_webhook)
        .post_async("/api/v1/webhooks/register", register_webhook)
        .get_async("/api/v1/webhooks", get_webhooks)
        .delete_async("/api/v1/webhooks/:id", delete_webhook)
        .get_async("/404", |_, _| async {
            Response::from_json(&ErrorResponse::not_found("Endpoint not found"))
        })
        .options_async("/*path", handle_options);

    Ok(with_cors(router.run(req, env).await?))
}
