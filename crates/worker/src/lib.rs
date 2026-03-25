mod types;
mod crypto;
mod middleware;
mod utils;

use worker::*;
use types::ErrorResponse;
use middleware::{with_cors, handle_options};

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    utils::set_panic_hook();

    let router = Router::new()
        .get("/", |_, _| Response::ok("PushOver API"))
        .get("/health", |_, _| Response::ok("OK"))
        .get_async("/404", |_, _| async {
            Response::from_json(&ErrorResponse::not_found("Endpoint not found"))
        })
        .options_async("/*path", handle_options);

    Ok(with_cors(router.run(req, env).await?))
}
