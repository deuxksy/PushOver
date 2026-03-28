use worker::*;
use crate::types::ErrorResponse;
use crate::db::Db;

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
    req: &Request,
    ctx: &RouteContext<()>,
) -> Result<Response> {
    let token = extract_token(req)?;

    let db = Db::new(ctx)?;
    match db.validate_token(&token).await? {
        Some(_user_key) => {
            // 토큰 유효 - RouteContext에 상태를 저장할 수 없으므로
            // 각 route에서 extract_token + validate_token 개별 호출 권장
            Response::ok("")
        }
        None => {
            Ok(Response::from_json(&ErrorResponse::unauthorized(
                "Invalid or inactive token"
            ))?.with_status(401))
        }
    }
}

/// Extract Bearer token from Authorization header. Returns the token string.
/// Routes should call this and handle 401 response themselves.
pub fn extract_token(req: &Request) -> Result<String> {
    let auth_header = req.headers().get("Authorization")?;
    match auth_header {
        Some(header) if header.starts_with("Bearer ") => {
            let token = header[7..].trim().to_string();
            if token.is_empty() {
                return Err(Error::from("Invalid token: empty bearer value"));
            }
            Ok(token)
        }
        Some(_) => Err(Error::from("Invalid Authorization header: expected Bearer token")),
        None => Err(Error::from("Missing Authorization header")),
    }
}

/// Helper to create a 401 unauthorized response from an error
pub fn unauthorized_response(error_msg: &str) -> Result<Response> {
    Ok(Response::from_json(&ErrorResponse::unauthorized(error_msg))?.with_status(401))
}
