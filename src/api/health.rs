use crate::state::AppState;
use axum::{extract::State, Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
    version: &'static str,
}

pub async fn check(State(_state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_response_shape() {
        let resp = HealthResponse {
            status: "ok",
            version: "0.1.0",
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["status"], "ok");
        assert!(json["version"].is_string());
    }
}
