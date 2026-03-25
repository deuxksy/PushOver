mod types;
mod utils;

use worker::*;
use types::ErrorResponse;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    utils::set_panic_hook();

    let router = Router::new()
        .get("/", |_, _| Response::ok("PushOver API"))
        .get("/health", |_, _| Response::ok("OK"))
        .get_async("/404", |_, _| async {
            Response::from_json(&ErrorResponse::not_found("Endpoint not found"))
        });

    Ok(router.run(req, env).await?)
}
