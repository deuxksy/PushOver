use worker::*;
use crate::types::ErrorResponse;

pub fn with_cors(mut response: Response) -> Response {
    let headers = response.headers_mut();
    let _ = headers.set("Access-Control-Allow-Origin", "*");
    let _ = headers.set("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS");
    let _ = headers.set("Access-Control-Allow-Headers", "Content-Type, Authorization");
    response
}

pub async fn handle_options(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    Ok(with_cors(Response::empty()?.with_status(204)))
}

pub async fn require_auth(
    req: Request,
    _ctx: RouteContext<()>,
) -> Result<Response> {
    let auth_header = req.headers().get("Authorization")?;

    let api_token = match auth_header {
        Some(header) => {
            if header.starts_with("Bearer ") {
                header[7..].to_string()
            } else {
                return Ok(Response::from_json(&ErrorResponse::unauthorized(
                    "Missing Bearer token"
                ))?.with_status(401));
            }
        }
        None => {
            return Ok(Response::from_json(&ErrorResponse::unauthorized(
                "Missing Authorization header"
            ))?.with_status(401));
        }
    };

    // Validate token (placeholder - will implement with D1)
    if api_token.is_empty() {
        return Ok(Response::from_json(&ErrorResponse::unauthorized(
            "Invalid token"
        ))?.with_status(401));
    }

    // TODO: Store validated token in context for downstream handlers
    // Note: RouteContext doesn't support setting params dynamically
    // This will need to be implemented with custom state or D1 lookup

    // Continue to next handler
    Response::ok("")
}
