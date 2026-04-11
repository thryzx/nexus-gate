use crate::error::AppError;
use crate::model::apikey::ApiKeyRecord;
use crate::state::AppState;
use axum::{
    body::Body,
    extract::State,
    http::{header, Request},
    middleware::Next,
    response::Response,
};
use sha2::{Digest, Sha256};

/// Extract the API key from multiple possible locations,
/// then validate against the database.
///
/// Full CRS-compatible auth chain:
/// 1. Extract key from header/query
/// 2. Hash and DB lookup
/// 3. Check status (active)
/// 4. Check expiration
/// 5. Check service permission
/// 6. Check model restriction
pub async fn require_api_key(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let raw_key = extract_key(&req).ok_or(AppError::Unauthorized)?;

    // Validate prefix (CRS checks cr_ prefix; we check nk-).
    if !raw_key.starts_with("nk-") && !raw_key.is_empty() {
        // Allow any key format that passes DB lookup, but log a warning.
        tracing::debug!("non-standard key prefix");
    }

    let key_hash = hash_key(&raw_key);

    let record = sqlx::query_as::<_, ApiKeyRecord>(
        r#"
        SELECT id, key_hash, name, permissions, daily_cost_limit,
               total_cost_limit, max_concurrency, rate_limit_rpm,
               restricted_models, status, expires_at, deleted_at, created_at
        FROM api_keys
        WHERE key_hash = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(&key_hash)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?
    .ok_or(AppError::Unauthorized)?;

    // Check status.
    if record.status != "active" {
        return Err(AppError::Forbidden);
    }

    // Check expiration.
    if let Some(exp) = record.expires_at {
        if exp < chrono::Utc::now() {
            return Err(AppError::Forbidden);
        }
    }

    // Check service permission based on request path.
    let service = detect_service(req.uri().path());
    if !record.has_permission(service) {
        return Err(AppError::Forbidden);
    }

    // Check model restriction (CRS checks in auth layer pre-request).
    if !record.restricted_models.is_empty() && record.restricted_models != "[]" {
        // Extract body bytes to peek at model field.
        let (parts, body) = req.into_parts();
        let body_bytes = axum::body::to_bytes(body, 1024 * 64)
            .await
            .unwrap_or_default();
        if let Ok(parsed) = serde_json::from_slice::<serde_json::Value>(&body_bytes) {
            if let Some(model) = parsed["model"].as_str() {
                if !record.is_model_allowed(model) {
                    return Err(AppError::Forbidden);
                }
            }
        }
        // Reconstruct the request with the consumed body.
        let mut new_req = Request::from_parts(parts, Body::from(body_bytes));
        new_req.extensions_mut().insert(record);
        return Ok(next.run(new_req).await);
    }

    // Attach the validated key record to request extensions for downstream use.
    req.extensions_mut().insert(record);

    Ok(next.run(req).await)
}

/// Detect which service is being accessed based on URL path.
fn detect_service(path: &str) -> &str {
    if path.starts_with("/v1/messages") || path.starts_with("/v1/messages/") {
        "claude"
    } else if path.starts_with("/v1/chat/completions") || path.starts_with("/v1/responses") {
        "openai"
    } else if path.starts_with("/v1beta/models") {
        "gemini"
    } else {
        "unknown"
    }
}

/// Look for the API key in standard header locations.
/// Order matches CRS: x-api-key → x-goog-api-key → Authorization Bearer → api-key → ?key.
fn extract_key(req: &Request<Body>) -> Option<String> {
    // 1. x-api-key header (highest priority, matches CRS)
    if let Some(key) = req.headers().get("x-api-key") {
        if let Ok(val) = key.to_str() {
            return Some(val.trim().to_string());
        }
    }

    // 2. x-goog-api-key header (Gemini clients)
    if let Some(key) = req.headers().get("x-goog-api-key") {
        if let Ok(val) = key.to_str() {
            return Some(val.trim().to_string());
        }
    }

    // 3. Authorization: Bearer <key>
    if let Some(auth) = req.headers().get(header::AUTHORIZATION) {
        if let Ok(val) = auth.to_str() {
            let stripped = val.strip_prefix("Bearer ").unwrap_or(val);
            return Some(stripped.trim().to_string());
        }
    }

    // 4. api-key header (Azure OpenAI clients)
    if let Some(key) = req.headers().get("api-key") {
        if let Ok(val) = key.to_str() {
            return Some(val.trim().to_string());
        }
    }

    // 5. Query parameter ?key=...
    if let Some(query) = req.uri().query() {
        for pair in query.split('&') {
            if let Some(val) = pair.strip_prefix("key=") {
                return Some(val.to_string());
            }
        }
    }

    None
}

/// SHA-256 hash of the raw API key for storage comparison.
pub fn hash_key(raw: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    hex::encode(hasher.finalize())
}

// hex encoding helper (avoid extra dependency)
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes
            .as_ref()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;

    #[test]
    fn hash_deterministic() {
        let a = hash_key("test-key-123");
        let b = hash_key("test-key-123");
        assert_eq!(a, b);
        assert_eq!(a.len(), 64); // SHA-256 hex = 64 chars
    }

    #[test]
    fn hash_different_keys() {
        assert_ne!(hash_key("key-a"), hash_key("key-b"));
    }

    #[test]
    fn extract_from_bearer() {
        let req = Request::builder()
            .header("Authorization", "Bearer sk-test123")
            .body(Body::empty())
            .unwrap();
        assert_eq!(extract_key(&req), Some("sk-test123".into()));
    }

    #[test]
    fn extract_from_x_api_key() {
        let req = Request::builder()
            .header("x-api-key", "my-key")
            .body(Body::empty())
            .unwrap();
        assert_eq!(extract_key(&req), Some("my-key".into()));
    }

    #[test]
    fn extract_from_query() {
        let req = Request::builder()
            .uri("/v1/messages?key=qp-key&other=1")
            .body(Body::empty())
            .unwrap();
        assert_eq!(extract_key(&req), Some("qp-key".into()));
    }

    #[test]
    fn extract_none_when_missing() {
        let req = Request::builder().body(Body::empty()).unwrap();
        assert_eq!(extract_key(&req), None);
    }

    #[test]
    fn x_api_key_takes_priority_over_bearer() {
        let req = Request::builder()
            .header("Authorization", "Bearer bearer-key")
            .header("x-api-key", "header-key")
            .body(Body::empty())
            .unwrap();
        assert_eq!(extract_key(&req), Some("header-key".into()));
    }

    #[test]
    fn extract_from_x_goog_api_key() {
        let req = Request::builder()
            .header("x-goog-api-key", "goog-key")
            .body(Body::empty())
            .unwrap();
        assert_eq!(extract_key(&req), Some("goog-key".into()));
    }

    #[test]
    fn extract_from_api_key_header() {
        let req = Request::builder()
            .header("api-key", "azure-key")
            .body(Body::empty())
            .unwrap();
        assert_eq!(extract_key(&req), Some("azure-key".into()));
    }

    #[test]
    fn bearer_without_prefix() {
        let req = Request::builder()
            .header("Authorization", "raw-token-no-bearer-prefix")
            .body(Body::empty())
            .unwrap();
        assert_eq!(extract_key(&req), Some("raw-token-no-bearer-prefix".into()));
    }

    #[test]
    fn detect_claude_service() {
        assert_eq!(detect_service("/v1/messages"), "claude");
        assert_eq!(detect_service("/v1/messages/count_tokens"), "claude");
    }

    #[test]
    fn detect_openai_service() {
        assert_eq!(detect_service("/v1/chat/completions"), "openai");
        assert_eq!(detect_service("/v1/responses"), "openai");
    }

    #[test]
    fn detect_gemini_service() {
        assert_eq!(detect_service("/v1beta/models"), "gemini");
        assert_eq!(detect_service("/v1beta/models/gemini-pro:generateContent"), "gemini");
    }

    #[test]
    fn detect_unknown_service() {
        assert_eq!(detect_service("/other/path"), "unknown");
    }
}
