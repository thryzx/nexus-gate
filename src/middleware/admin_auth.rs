use crate::error::AppError;
use crate::state::AppState;
use axum::{
    body::Body,
    extract::State,
    http::{header, Request},
    middleware::Next,
    response::Response,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Middleware that requires a valid JWT Bearer token for admin endpoints.
/// Uses HMAC-SHA256 to verify the token signature against `config.auth.jwt_secret`.
pub async fn require_admin_jwt(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;

    verify_jwt_signature(token, &state.config.auth.jwt_secret)?;

    Ok(next.run(req).await)
}

/// Minimal JWT HS256 signature verification (header.payload.signature).
fn verify_jwt_signature(token: &str, secret: &str) -> Result<(), AppError> {
    let parts: Vec<&str> = token.splitn(3, '.').collect();
    if parts.len() != 3 {
        return Err(AppError::Unauthorized);
    }

    let signing_input = format!("{}.{}", parts[0], parts[1]);
    let signature_bytes = URL_SAFE_NO_PAD
        .decode(parts[2])
        .map_err(|_| AppError::Unauthorized)?;

    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).map_err(|_| AppError::Unauthorized)?;
    mac.update(signing_input.as_bytes());
    mac.verify_slice(&signature_bytes)
        .map_err(|_| AppError::Unauthorized)?;

    // Decode payload and check expiration.
    let payload_json = URL_SAFE_NO_PAD
        .decode(parts[1])
        .map_err(|_| AppError::Unauthorized)?;
    if let Ok(claims) = serde_json::from_slice::<serde_json::Value>(&payload_json) {
        if let Some(exp) = claims["exp"].as_i64() {
            let now = chrono::Utc::now().timestamp();
            if now > exp {
                return Err(AppError::Unauthorized);
            }
        }
    }

    Ok(())
}
