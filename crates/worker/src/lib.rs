mod utils;

use worker::*;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    utils::set_panic_hook();

    let router = Router::new()
        .get("/", |_, _| Response::ok("PushOver API"))
        .get("/health", |_, _| Response::ok("OK"));

    router.run(req, env).await
}
