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

/// POST /v1/chat/completions — OpenAI Chat Completions relay.
pub async fn chat_completions(
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
        RelayTarget::OpenAI,
        &model,
        headers,
        body,
    )
    .await
}

/// POST /v1/responses — OpenAI Responses API relay.
pub async fn responses(
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
        RelayTarget::OpenAI,
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
    fn extract_openai_model() {
        let body = Bytes::from(r#"{"model":"gpt-4o","messages":[{"role":"user","content":"hi"}]}"#);
        assert_eq!(extract_model(&body).unwrap(), "gpt-4o");
    }
}
