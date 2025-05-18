/// Centralized helpers for WebSocket and HTTP error responses.
///
/// Use these helpers to ensure all error messages are consistent, explicit, and include a code and context.
use actix_web::{HttpResponse, http::StatusCode};
use log::{warn, error};
use serde_json::{json, Value};

/// Formats a WebSocket error message as a JSON string.
///
/// # Arguments
/// - `code`: Unique error code (e.g. "INVALID_ACTION").
/// - `message`: Human-readable error message (in English).
/// - `context`: Optional context (can be a string or a JSON object).
pub fn ws_error_message<S: AsRef<str>>(code: S, message: S, context: Option<Value>) -> String {
    let mut data = json!({
        "code": code.as_ref(),
        "message": message.as_ref(),
    });

    if let Some(ref ctx) = context {
        data["context"] = ctx.clone();
    }

    let json_msg = json!({
        "action": "Error",
        "data": data
    });

    let result = json_msg.to_string();
    
    // Log the error with a simplified context representation
    let context_str = match &context {
        Some(ctx) => format!("{}", ctx),
        None => "None".to_string(),
    };
    warn!("[WS_ERROR] code={} message='{}' context={}", code.as_ref(), message.as_ref(), context_str);
    
    result
}

/// Example usage for a temporary ban:
/// ```
/// let ban_remaining_secs = 42;
/// let wallet = "0x123...";
/// let context = json!({
///     "wallet": wallet,
///     "ban_remaining_secs": ban_remaining_secs
/// });
/// let msg = ws_error_message("BANNED", "You have been banned for spamming. Please try again later.", Some(context));
/// ```

/// Returns a WebSocket message for session kicked (unicity violation).
pub fn ws_session_kicked_message(context: Option<Value>) -> String {
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
/// - `context`: Optional context (can be a string or a JSON object).
/// - `status`: HTTP status code.
pub fn http_error_response<S: AsRef<str>>(
    code: S,
    message: S,
    context: Option<Value>,
    status: StatusCode,
) -> HttpResponse {
    let mut error_data = json!({
        "code": code.as_ref(),
        "message": message.as_ref(),
    });

    if let Some(ref ctx) = context {
        error_data["context"] = ctx.clone();
    }

    let body = json!({
        "error": error_data
    }).to_string();

    // Log the error with a simplified context representation
    let context_str = match &context {
        Some(ctx) => format!("{}", ctx),
        None => "None".to_string(),
    };
    error!("[HTTP_ERROR] code={} message='{}' context={}", code.as_ref(), message.as_ref(), context_str);
    
    HttpResponse::build(status).content_type("application/json").body(body)
}
