use crate::error::AppError;
use crate::model::apikey::ApiKeyRecord;
use crate::service::relay::RelayTarget;
use crate::state::AppState;
use axum::{
    body::Body,
    extract::State,
    http::Request,
    response::Response,
};
use bytes::Bytes;

/// POST /v1/messages — Claude Messages API relay.
pub async fn messages(
    State(state): State<AppState>,
    req: Request<Body>,
) -> Result<Response, AppError> {
    let key_record = req
        .extensions()
        .get::<ApiKeyRecord>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    let headers = req.headers().clone();
    let body = axum::body::to_bytes(req.into_body(), state.config.server.max_body_bytes)
        .await
        .map_err(|_| AppError::BadRequest("body too large".into()))?;

    // Parse and validate (CRS: must have non-empty messages array).
    let parsed: serde_json::Value =
        serde_json::from_slice(&body).map_err(|_| AppError::BadRequest("invalid JSON".into()))?;
    let model = parsed["model"]
        .as_str()
        .ok_or_else(|| AppError::BadRequest("model field required".into()))?
        .to_string();
    match parsed["messages"].as_array() {
        Some(arr) if arr.is_empty() => {
            return Err(AppError::BadRequest("messages array must not be empty".into()));
        }
        None => {
            return Err(AppError::BadRequest("messages field required".into()));
        }
        _ => {}
    }

    crate::service::relay::forward(
        &state,
        &key_record,
        RelayTarget::Claude,
        &model,
        headers,
        body,
    )
    .await
}

/// POST /v1/messages/count_tokens — Token counting relay.
pub async fn count_tokens(
    State(state): State<AppState>,
    req: Request<Body>,
) -> Result<Response, AppError> {
    let key_record = req
        .extensions()
        .get::<ApiKeyRecord>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    let headers = req.headers().clone();
    let body = axum::body::to_bytes(req.into_body(), state.config.server.max_body_bytes)
        .await
        .map_err(|_| AppError::BadRequest("body too large".into()))?;

    let model = extract_model(&body)?;

    crate::service::relay::forward(
        &state,
        &key_record,
        RelayTarget::Claude,
        &model,
        headers,
        body,
    )
    .await
}

fn extract_model(body: &Bytes) -> Result<String, AppError> {
    let parsed: serde_json::Value =
        serde_json::from_slice(body).map_err(|_| AppError::BadRequest("invalid JSON".into()))?;
    parsed["model"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::BadRequest("model field required".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_model_success() {
        let body = Bytes::from(r#"{"model":"claude-sonnet-4-20250514","messages":[]}"#);
        assert_eq!(extract_model(&body).unwrap(), "claude-sonnet-4-20250514");
    }

    #[test]
    fn extract_model_missing() {
        let body = Bytes::from(r#"{"messages":[]}"#);
        assert!(extract_model(&body).is_err());
    }

    #[test]
    fn extract_model_invalid_json() {
        let body = Bytes::from("not json");
        assert!(extract_model(&body).is_err());
    }
}
