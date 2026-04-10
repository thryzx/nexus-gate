use crate::error::AppError;
use crate::model::apikey::ApiKeyRecord;
use crate::state::AppState;
use axum::{body::Body, extract::State, http::Request, middleware::Next, response::Response};
use redis::AsyncCommands;

/// Sliding-window rate limiter backed by Redis.
pub async fn check(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let key_record = req
        .extensions()
        .get::<ApiKeyRecord>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    let rpm = if key_record.rate_limit_rpm > 0 {
        key_record.rate_limit_rpm as u32
    } else {
        state.config.rate_limit.per_key_rpm
    };

    let window_key = format!("rl:{}:{}", key_record.id, current_minute());

    let mut conn = state
        .redis_conn()
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    let count: u32 = conn
        .incr(&window_key, 1u32)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    if count == 1 {
        let _: () = conn
            .expire(&window_key, 60)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
    }

    if count > rpm {
        return Err(AppError::RateLimited);
    }

    Ok(next.run(req).await)
}

fn current_minute() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        / 60
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minute_monotonic() {
        let a = current_minute();
        let b = current_minute();
        assert!(b >= a);
    }
}
