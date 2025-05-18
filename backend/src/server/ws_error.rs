/// Centralized helpers for WebSocket and HTTP error responses.
///
/// Use these helpers to ensure all error messages are consistent, explicit, and include a code and context.
use actix_web::{HttpResponse, http::StatusCode};

/// Formats a WebSocket error message as a JSON string.
///
/// # Arguments
/// - `code`: Unique error code (e.g. "INVALID_ACTION").
/// - `message`: Human-readable error message (in English).
/// - `context`: Optional context (e.g. player_id, game_id).
pub fn ws_error_message(code: &str, message: &str, context: Option<&str>) -> String {
    let context_str = context.unwrap_or("");
    format!(
        r#"{{"action":"Error","data":{{"code":"{}","message":"{}","context":"{}"}}}}"#,
        code, message, context_str
    )
}

/// Returns a WebSocket message for session kicked (unicity violation).
pub fn ws_session_kicked_message(context: Option<&str>) -> String {
    ws_error_message(
        "SESSION_KICKED",
        "You have been disconnected because another session has connected with your wallet.",
        context,
    )
}

/// Returns an HTTP error response with a JSON body.
///
/// # Arguments
/// - `code`: Unique error code.
/// - `message`: Human-readable error message.
/// - `context`: Optional context string.
/// - `status`: HTTP status code.
pub fn http_error_response(
    code: &str,
    message: &str,
    context: Option<&str>,
    status: StatusCode,
) -> HttpResponse {
    let context_str = context.unwrap_or("");
    let body = format!(
        r#"{{"error":{{"code":"{}","message":"{}","context":"{}"}}}}"#,
        code, message, context_str
    );
    HttpResponse::build(status).content_type("application/json").body(body)
}
