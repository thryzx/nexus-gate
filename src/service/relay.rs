use crate::error::AppError;
use crate::fingerprint::engine::FingerprintEngine;
use crate::model::account::AccountRecord;
use crate::model::apikey::ApiKeyRecord;
use crate::service::{account as account_svc, cost, scheduler};
use crate::state::AppState;
use crate::util::crypto;
use axum::{
    body::Body,
    http::{header, HeaderMap, HeaderValue, Response, StatusCode},
};
use bytes::Bytes;
use redis::AsyncCommands;
use uuid::Uuid;

/// Which upstream platform to target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelayTarget {
    Claude,
    OpenAI,
    Gemini,
    Bedrock,
    Azure,
}

/// Core relay: select account → build fingerprinted client → forward → stream back.
///
/// Handles upstream error classification matching CRS:
/// - 401 → mark account error
/// - 403 → mark account blocked
/// - 429 → mark account rate-limited with cooldown
/// - 529 → mark account temp unavailable
pub async fn forward(
    state: &AppState,
    key_record: &ApiKeyRecord,
    target: RelayTarget,
    model: &str,
    client_headers: HeaderMap,
    body: Bytes,
) -> Result<Response<Body>, AppError> {
    // 1. Select an available account via the scheduler.
    let account = scheduler::select_account(state, key_record, target, model, &body).await?;

    // 2. Track concurrency: add request to sorted set.
    let request_id = Uuid::new_v4().to_string();
    let lease_secs = state.config.scheduler.concurrency_lease_secs;
    acquire_concurrency_slot(state, &account, &request_id, lease_secs).await?;

    // 3. Build a fingerprinted HTTP client for this account.
    let fp_engine = FingerprintEngine::new();
    let http_client = fp_engine
        .build_client(&account)
        .map_err(|e| AppError::Internal(e.into()))?;

    // 4. Construct upstream URL.
    let upstream_url = build_upstream_url(state, target, &client_headers);

    // 5. Build upstream request with sanitised headers + injected auth.
    let upstream_headers = build_upstream_headers(state, target, &account, &client_headers);

    let req_builder = http_client
        .post(&upstream_url)
        .headers(upstream_headers)
        .body(body.to_vec());

    // 6. Send to upstream.
    let upstream_resp = match req_builder.send().await {
        Ok(resp) => resp,
        Err(e) => {
            release_concurrency_slot(state, &account, &request_id).await;
            if e.is_timeout() {
                return Err(AppError::Timeout);
            }
            return Err(AppError::Internal(e.into()));
        }
    };

    let status_u16 = upstream_resp.status().as_u16();
    let status = StatusCode::from_u16(status_u16).unwrap_or(StatusCode::BAD_GATEWAY);

    // 7. Handle upstream error status codes (CRS-compatible).
    if status_u16 >= 400 {
        let resp_body = upstream_resp.text().await.unwrap_or_default();
        release_concurrency_slot(state, &account, &request_id).await;
        return handle_upstream_error(state, &account, status_u16, &resp_body).await;
    }

    // 8. Stream successful response back, releasing concurrency on completion.
    let resp_headers = upstream_resp.headers().clone();

    let mut response = Response::builder().status(status);
    for (k, v) in resp_headers.iter() {
        response = response.header(k, v);
    }

    // Move upstream_resp into a background spawned task that:
    // - forwards each chunk to the client via mpsc channel
    // - accumulates bytes for usage extraction
    // - releases concurrency slot when stream completes (not after lease timeout)
    let state_bg = state.clone();
    let account_bg = account.clone();
    let rid_bg = request_id.clone();
    let key_id = key_record.id;
    let model_owned = model.to_string();
    let state_usage = state.clone();

    let (tx, rx) = tokio::sync::mpsc::channel::<Result<Bytes, std::io::Error>>(32);

    tokio::spawn(async move {
        let mut all_bytes = Vec::new();
        let mut stream = upstream_resp.bytes_stream();
        loop {
            use futures::StreamExt as _;
            match stream.next().await {
                Some(Ok(data)) => {
                    all_bytes.extend_from_slice(&data);
                    if tx.send(Ok(data)).await.is_err() {
                        break; // Client disconnected.
                    }
                }
                Some(Err(e)) => {
                    let _ = tx.send(Err(std::io::Error::new(std::io::ErrorKind::Other, e)));
                    break;
                }
                None => break, // Stream complete.
            }
        }
        drop(tx);
        // Release concurrency slot immediately when stream completes.
        release_concurrency_slot(&state_bg, &account_bg, &rid_bg).await;
        // Best-effort usage capture.
        capture_usage_best_effort(&state_usage, key_id, account_bg.id, &model_owned, &all_bytes)
            .await;
    });

    let body_stream = tokio_stream::wrappers::ReceiverStream::new(rx);
    let body = Body::from_stream(body_stream);

    response
        .body(body)
        .map_err(|e| AppError::Internal(e.into()))
}

/// Handle upstream error responses — classify and update account state (CRS parity).
async fn handle_upstream_error(
    state: &AppState,
    account: &AccountRecord,
    status: u16,
    body: &str,
) -> Result<Response<Body>, AppError> {
    match status {
        401 => {
            tracing::warn!(account_id = %account.id, "upstream 401 → error");
            let _ = account_svc::mark_error(state, account.id).await;
            Err(AppError::UpstreamError {
                status: 401,
                body: body.to_string(),
            })
        }
        400 => {
            // CRS: 400 from upstream = org disabled/banned → block for 90 days.
            tracing::warn!(account_id = %account.id, "upstream 400 → blocked");
            let _ = account_svc::mark_blocked(state, account.id).await;
            Err(AppError::UpstreamError {
                status: 400,
                body: body.to_string(),
            })
        }
        403 => {
            tracing::warn!(account_id = %account.id, "upstream 403 → blocked");
            let _ = account_svc::mark_blocked(state, account.id).await;
            Err(AppError::UpstreamError {
                status: 403,
                body: body.to_string(),
            })
        }
        429 => {
            tracing::warn!(account_id = %account.id, "upstream 429 → rate-limited");
            let cooldown = state.config.scheduler.rate_limit_cooldown_secs;
            let _ = mark_rate_limited(state, account, cooldown).await;
            Err(AppError::RateLimited)
        }
        529 => {
            tracing::warn!(account_id = %account.id, "upstream 529 → temp unavailable");
            let _ = account_svc::mark_unavailable(state, account.id).await;
            Err(AppError::Overloaded)
        }
        500..=599 => {
            // CRS: generic 5xx → mark temp unavailable.
            tracing::warn!(account_id = %account.id, status, "upstream 5xx → temp unavailable");
            let _ = account_svc::mark_unavailable(state, account.id).await;
            Err(AppError::UpstreamError {
                status,
                body: body.to_string(),
            })
        }
        _ => Err(AppError::UpstreamError {
            status,
            body: body.to_string(),
        }),
    }
}

// ═══════════════════════════════════════════════════════════════
//  Concurrency tracking (Redis Sorted Set, CRS-compatible)
// ═══════════════════════════════════════════════════════════════

/// Add request to sorted set. Score = lease expiry timestamp. Member = request_id.
async fn acquire_concurrency_slot(
    state: &AppState,
    account: &AccountRecord,
    request_id: &str,
    lease_secs: u64,
) -> Result<(), AppError> {
    let mut conn = state
        .redis_conn()
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    let key = format!("conc:{}", account.id);
    let expiry = chrono::Utc::now().timestamp() as f64 + lease_secs as f64;

    let _: () = conn
        .zadd(&key, request_id, expiry)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    // Safety-net key expiry.
    let _: () = conn
        .expire(&key, (lease_secs * 2) as i64)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    // Evict expired entries.
    let now = chrono::Utc::now().timestamp() as f64;
    let _: () = conn
        .zrembyscore(&key, "-inf", now)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    Ok(())
}

/// Remove request from sorted set.
async fn release_concurrency_slot(state: &AppState, account: &AccountRecord, request_id: &str) {
    let Ok(mut conn) = state.redis_conn().await else {
        return;
    };
    let key = format!("conc:{}", account.id);
    let _: Result<(), _> = conn.zrem(&key, request_id).await;
}

/// Mark an account as rate-limited via Redis sorted set.
async fn mark_rate_limited(
    state: &AppState,
    account: &AccountRecord,
    cooldown_secs: u64,
) -> Result<(), AppError> {
    let mut conn = state
        .redis_conn()
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    let key = format!("ratelimit:{}", account.id);
    let expiry = chrono::Utc::now().timestamp() as f64 + cooldown_secs as f64;
    let member = format!("rl:{}", Uuid::new_v4());

    let _: () = conn
        .zadd(&key, &member, expiry)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    // Cleanup old entries.
    let now = chrono::Utc::now().timestamp() as f64;
    let _: () = conn
        .zrembyscore(&key, "-inf", now)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    Ok(())
}

// ═══════════════════════════════════════════════════════════════
//  URL + headers construction
// ═══════════════════════════════════════════════════════════════

fn build_upstream_url(state: &AppState, target: RelayTarget, headers: &HeaderMap) -> String {
    match target {
        RelayTarget::Claude => {
            format!("{}/v1/messages", state.config.upstream.claude.base_url)
        }
        RelayTarget::OpenAI => {
            format!(
                "{}/v1/chat/completions",
                state.config.upstream.openai.base_url
            )
        }
        RelayTarget::Gemini => {
            // Gemini URL requires model/action path from the original request.
            // The path is passed via a custom header set by the Gemini route handler.
            let model_action = headers
                .get("x-ng-model-action")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("gemini-2.5-pro:generateContent");
            format!(
                "{}/v1beta/models/{}",
                state.config.upstream.gemini.base_url, model_action
            )
        }
        RelayTarget::Bedrock => todo!("bedrock URL construction"),
        RelayTarget::Azure => todo!("azure URL construction"),
    }
}

fn build_upstream_headers(
    state: &AppState,
    target: RelayTarget,
    account: &AccountRecord,
    client_headers: &HeaderMap,
) -> HeaderMap {
    let mut headers = HeaderMap::new();

    // Copy safe headers only (strip user-agent — fingerprint module controls it).
    let allowed = [header::CONTENT_TYPE, header::ACCEPT];
    for key in &allowed {
        if let Some(val) = client_headers.get(key) {
            headers.insert(key.clone(), val.clone());
        }
    }

    headers
        .entry(header::CONTENT_TYPE)
        .or_insert_with(|| HeaderValue::from_static("application/json"));

    // Decrypt credentials and inject platform-specific auth.
    let creds = decrypt_credentials(&account.credentials_enc, &state.config.auth.encryption_key);

    match target {
        RelayTarget::Claude => {
            if let Some(ver) = &state.config.upstream.claude.api_version {
                if let Ok(v) = HeaderValue::from_str(ver) {
                    headers.insert("anthropic-version", v);
                }
            }
            // Auth injection: OAuth accounts use Authorization: Bearer,
            // API key accounts use x-api-key (CRS parity).
            if let Some(access_token) = creds.get("access_token") {
                // OAuth account — use Bearer token.
                if let Ok(v) = HeaderValue::from_str(&format!("Bearer {access_token}")) {
                    headers.insert(header::AUTHORIZATION, v);
                }
            } else if let Some(api_key) = creds.get("api_key") {
                // API key account — use x-api-key header.
                if let Ok(v) = HeaderValue::from_str(api_key) {
                    headers.insert("x-api-key", v);
                }
            }
            // Inject default beta headers (CRS always sends these).
            let mut beta_parts = vec![
                "claude-code-20250219",
                "oauth-2025-04-20",
                "interleaved-thinking-2025-05-14",
            ];
            // non-Haiku gets fine-grained-tool-streaming
            beta_parts.push("fine-grained-tool-streaming-2025-05-14");
            // Merge with client's anthropic-beta header if present.
            if let Some(client_beta) = client_headers.get("anthropic-beta") {
                if let Ok(val) = client_beta.to_str() {
                    for part in val.split(',') {
                        let trimmed = part.trim();
                        if !trimmed.is_empty() && !beta_parts.contains(&trimmed) {
                            beta_parts.push(trimmed);
                        }
                    }
                }
            }
            if let Ok(v) = HeaderValue::from_str(&beta_parts.join(", ")) {
                headers.insert("anthropic-beta", v);
            }
        }
        RelayTarget::OpenAI => {
            if let Some(token) = creds.get("api_key") {
                if let Ok(v) = HeaderValue::from_str(&format!("Bearer {token}")) {
                    headers.insert(header::AUTHORIZATION, v);
                }
            }
        }
        RelayTarget::Gemini => {
            if let Some(token) = creds.get("api_key") {
                if let Ok(v) = HeaderValue::from_str(token) {
                    headers.insert("x-goog-api-key", v);
                }
            }
        }
        _ => {}
    }

    headers
}

fn decrypt_credentials(
    encrypted: &str,
    encryption_key: &str,
) -> std::collections::HashMap<String, String> {
    let decrypted = crypto::decrypt(encrypted, encryption_key).unwrap_or_default();
    serde_json::from_str(&decrypted).unwrap_or_default()
}

/// Best-effort usage extraction from accumulated response bytes.
/// Handles both non-streaming JSON and SSE stream (extracts from final event).
async fn capture_usage_best_effort(
    state: &AppState,
    api_key_id: Uuid,
    account_id: Uuid,
    model: &str,
    data: &[u8],
) {
    let text = String::from_utf8_lossy(data);

    // Try non-streaming response first (complete JSON).
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
        let usage = cost::extract_usage(&json);
        if usage.input_tokens > 0 || usage.output_tokens > 0 {
            let pricing = cost::default_pricing(model);
            let breakdown = cost::calculate_cost(&pricing, &usage);
            let _ =
                cost::record_usage(state, api_key_id, account_id, model, &usage, breakdown.total_cost)
                    .await;
            return;
        }
    }

    // SSE stream: find the last data event containing usage info.
    for line in text.lines().rev() {
        let line = line.trim();
        if let Some(json_str) = line.strip_prefix("data: ") {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(json_str) {
                let usage = cost::extract_usage(&json);
                if usage.input_tokens > 0 || usage.output_tokens > 0 {
                    let pricing = cost::default_pricing(model);
                    let breakdown = cost::calculate_cost(&pricing, &usage);
                    let _ = cost::record_usage(
                        state,
                        api_key_id,
                        account_id,
                        model,
                        &usage,
                        breakdown.total_cost,
                    )
                    .await;
                    return;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relay_targets_are_distinct() {
        assert_ne!(RelayTarget::Claude, RelayTarget::OpenAI);
        assert_ne!(RelayTarget::Gemini, RelayTarget::Bedrock);
    }

    #[test]
    fn decrypt_empty_returns_empty_map() {
        let creds = decrypt_credentials("", "0123456789abcdef0123456789abcdef");
        assert!(creds.is_empty());
    }

    #[test]
    fn decrypt_invalid_returns_empty_map() {
        let creds = decrypt_credentials("not-encrypted", "0123456789abcdef0123456789abcdef");
        assert!(creds.is_empty());
    }
}
