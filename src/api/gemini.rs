use crate::error::AppError;
use crate::model::apikey::ApiKeyRecord;
use crate::service::relay::RelayTarget;
use crate::state::AppState;
use axum::{
    body::Body,
    extract::{Path, State},
    http::Request,
    response::{Json, Response},
};
use serde::Serialize;

/// POST /v1beta/models/:model_action — Gemini generateContent etc.
pub async fn model_action(
    State(state): State<AppState>,
    Path(model_action): Path<String>,
    req: Request<Body>,
) -> Result<Response, AppError> {
    let key_record = req
        .extensions()
        .get::<ApiKeyRecord>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    let mut headers = req.headers().clone();
    let body = axum::body::to_bytes(req.into_body(), state.config.server.max_body_bytes)
        .await
        .map_err(|_| AppError::BadRequest("body too large".into()))?;

    // model_action is like "gemini-2.5-pro:generateContent"
    let model = model_action
        .split(':')
        .next()
        .unwrap_or(&model_action)
        .to_string();

    // Pass full model_action to relay layer for upstream URL construction.
    if let Ok(v) = axum::http::HeaderValue::from_str(&model_action) {
        headers.insert("x-ng-model-action", v);
    }

    crate::service::relay::forward(
        &state,
        &key_record,
        RelayTarget::Gemini,
        &model,
        headers,
        body,
    )
    .await
}

/// GET /v1beta/models — List available Gemini models.
pub async fn list_models(State(_state): State<AppState>) -> Json<ModelsResponse> {
    Json(ModelsResponse {
        models: vec![
            ModelInfo {
                name: "gemini-2.5-pro".into(),
            },
            ModelInfo {
                name: "gemini-2.5-flash".into(),
            },
        ],
    })
}

#[derive(Serialize)]
pub struct ModelsResponse {
    models: Vec<ModelInfo>,
}

#[derive(Serialize)]
pub struct ModelInfo {
    name: String,
}

#[cfg(test)]
mod tests {

    #[test]
    fn parse_model_from_action_path() {
        let input = "gemini-2.5-pro:generateContent";
        let model = input.split(':').next().unwrap();
        assert_eq!(model, "gemini-2.5-pro");
    }

    #[test]
    fn parse_model_without_action() {
        let input = "gemini-2.5-flash";
        let model = input.split(':').next().unwrap();
        assert_eq!(model, "gemini-2.5-flash");
    }
}
