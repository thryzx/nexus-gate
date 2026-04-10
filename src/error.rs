use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

/// Unified application error type.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("authentication required")]
    Unauthorized,

    #[error("permission denied")]
    Forbidden,

    #[error("cost limit exceeded")]
    PaymentRequired,

    #[error("not found")]
    NotFound,

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("rate limit exceeded")]
    RateLimited,

    #[error("upstream unavailable")]
    UpstreamUnavailable,

    #[error("upstream error: {status}")]
    UpstreamError { status: u16, body: String },

    #[error("request timeout")]
    Timeout,

    #[error("service overloaded")]
    Overloaded,

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

/// Wire format returned to clients — mirrors Anthropic error shape for
/// Claude-compatible endpoints, but contains no project-identifying info.
#[derive(Serialize)]
struct ErrorBody {
    r#type: &'static str,
    error: ErrorDetail,
}

#[derive(Serialize)]
struct ErrorDetail {
    r#type: String,
    message: String,
}

impl AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::PaymentRequired => StatusCode::PAYMENT_REQUIRED,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            Self::UpstreamUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            Self::UpstreamError { status, .. } => {
                StatusCode::from_u16(*status).unwrap_or(StatusCode::BAD_GATEWAY)
            }
            Self::Timeout => StatusCode::GATEWAY_TIMEOUT,
            Self::Overloaded => StatusCode::SERVICE_UNAVAILABLE,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_type(&self) -> &str {
        match self {
            Self::Unauthorized | Self::Forbidden => "authentication_error",
            Self::PaymentRequired => "billing_error",
            Self::NotFound => "not_found_error",
            Self::BadRequest(_) => "invalid_request_error",
            Self::RateLimited => "rate_limit_error",
            Self::UpstreamUnavailable | Self::UpstreamError { .. } | Self::Overloaded => {
                "api_error"
            }
            Self::Timeout => "timeout_error",
            Self::Internal(_) => "api_error",
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Never leak internal details to the client.
        let message = match &self {
            Self::Internal(e) => {
                tracing::error!(error = %e, "internal error");
                "internal server error".to_string()
            }
            other => other.to_string(),
        };

        let body = ErrorBody {
            r#type: "error",
            error: ErrorDetail {
                r#type: self.error_type().to_string(),
                message,
            },
        };

        (self.status_code(), axum::Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_status_codes() {
        assert_eq!(AppError::Unauthorized.status_code(), StatusCode::UNAUTHORIZED);
        assert_eq!(AppError::RateLimited.status_code(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(
            AppError::BadRequest("x".into()).status_code(),
            StatusCode::BAD_REQUEST
        );
    }

    #[test]
    fn error_type_strings() {
        assert_eq!(AppError::Unauthorized.error_type(), "authentication_error");
        assert_eq!(AppError::Timeout.error_type(), "timeout_error");
    }

    #[test]
    fn internal_error_hides_details() {
        let err = AppError::Internal(anyhow::anyhow!("secret db password"));
        let body = serde_json::to_string(&ErrorBody {
            r#type: "error",
            error: ErrorDetail {
                r#type: err.error_type().to_string(),
                message: "internal server error".to_string(),
            },
        })
        .unwrap();
        assert!(!body.contains("secret db password"));
        assert!(body.contains("internal server error"));
    }
}
