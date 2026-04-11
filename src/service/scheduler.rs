use crate::error::AppError;
use crate::model::account::AccountRecord;
use crate::model::apikey::ApiKeyRecord;
use crate::service::relay::RelayTarget;
use crate::service::session;
use crate::state::AppState;
use bytes::Bytes;
use redis::AsyncCommands;

/// Select the best available account for a given request.
///
/// Strategy (matching CRS logic):
/// 1. Check API key dedicated account binding.
/// 2. Check sticky session cache — reuse if still valid.
/// 3. Query active accounts for the target platform.
/// 4. Filter: temp unavailability, rate limit, model support.
/// 5. Sort by priority, then by current load (concurrency).
/// 6. Bind sticky session for future requests.
pub async fn select_account(
    state: &AppState,
    key_record: &ApiKeyRecord,
    target: RelayTarget,
    model: &str,
    body: &Bytes,
) -> Result<AccountRecord, AppError> {
    let platform = platform_string(target);

    // 1. API key dedicated account binding.
    if let Some(account) = check_dedicated_account(state, key_record, platform).await? {
        if is_account_available(state, &account, model).await {
            return Ok(account);
        }
        // Dedicated account not available — fall through to pool.
    }

    // 2. Sticky session lookup.
    let session_hash = session::compute_hash(body);
    if let Some(account) = session::get_sticky(state, &session_hash).await? {
        if is_account_available(state, &account, model).await {
            tracing::debug!(account_id = %account.id, "sticky session hit");
            return Ok(account);
        }
        // Sticky account no longer valid — select a new one.
    }

    // 3. Fetch active accounts for this platform.
    let all_candidates = sqlx::query_as::<_, AccountRecord>(
        r#"
        SELECT * FROM accounts
        WHERE platform = $1 AND status = 'active'
        ORDER BY priority ASC, created_at ASC
        "#,
    )
    .bind(platform)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    if all_candidates.is_empty() {
        return Err(AppError::UpstreamUnavailable);
    }

    // 4. Filter candidates by availability + model support.
    let mut candidates = Vec::new();
    for acct in &all_candidates {
        if is_account_available(state, acct, model).await {
            candidates.push(acct.clone());
        }
    }

    if candidates.is_empty() {
        return Err(AppError::Overloaded);
    }

    // 5. Pick the candidate with the lowest current concurrency.
    let selected = pick_least_loaded(state, &candidates).await?;

    // 6. Bind sticky session.
    session::set_sticky(
        state,
        &session_hash,
        &selected.id.to_string(),
        state.config.scheduler.sticky_session_ttl_secs,
    )
    .await?;

    tracing::info!(
        account_id = %selected.id,
        platform = platform,
        model = model,
        "account selected"
    );

    Ok(selected)
}

/// Check if an API key has a dedicated account binding for this platform.
async fn check_dedicated_account(
    state: &AppState,
    key_record: &ApiKeyRecord,
    platform: &str,
) -> Result<Option<AccountRecord>, AppError> {
    // Permissions may contain a dedicated account ID per platform.
    // Parse from key_record.permissions JSON: {"dedicated_accounts": {"claude": "<uuid>", ...}}
    let perms = &key_record.permissions;
    if let Some(account_id_str) = perms["dedicated_accounts"][platform].as_str() {
        if let Ok(account_id) = account_id_str.parse::<uuid::Uuid>() {
            let record = sqlx::query_as::<_, AccountRecord>(
                "SELECT * FROM accounts WHERE id = $1 AND status = 'active'",
            )
            .bind(account_id)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

            return Ok(record);
        }
    }
    Ok(None)
}

/// Check if an account is available: not temp unavailable, not rate limited, model supported.
async fn is_account_available(state: &AppState, account: &AccountRecord, model: &str) -> bool {
    if account.status != "active" {
        return false;
    }

    // Check temporary unavailability (Redis TTL key from 529 handling).
    if is_temp_unavailable(state, account).await {
        return false;
    }

    // Check rate limit status (Redis sorted set).
    if is_rate_limited(state, account).await {
        return false;
    }

    // Check model support (if restricted).
    if !is_model_supported(account, model) {
        return false;
    }

    true
}

/// Check Redis for temp unavailability marker (set by 529 handler).
async fn is_temp_unavailable(state: &AppState, account: &AccountRecord) -> bool {
    let Ok(mut conn) = state.redis_conn().await else {
        return false;
    };
    let key = format!("cooldown:{}", account.id);
    let exists: bool = conn.exists(&key).await.unwrap_or(false);
    exists
}

/// Check Redis sorted set for rate limit status.
async fn is_rate_limited(state: &AppState, account: &AccountRecord) -> bool {
    let Ok(mut conn) = state.redis_conn().await else {
        return false;
    };
    let key = format!("ratelimit:{}", account.id);
    let now = chrono::Utc::now().timestamp() as f64;
    // Count entries with score (expiry) > now — still active rate limits.
    let count: i64 = conn
        .zcount(&key, now, "+inf")
        .await
        .unwrap_or(0);
    count > 0
}

/// Check if the requested model is supported by this account.
/// Basic model support logic: accounts may have allowed_models in credentials.
fn is_model_supported(account: &AccountRecord, _model: &str) -> bool {
    // By default all models are supported unless credentials contain restrictions.
    // Full Opus logic (free/pro/max tiers) would be implemented when account
    // subscription info is available. For now, allow all.
    let _ = account;
    true
}

/// From the candidate list, pick the account with the lowest current
/// concurrency count stored in Redis.
async fn pick_least_loaded(
    state: &AppState,
    candidates: &[AccountRecord],
) -> Result<AccountRecord, AppError> {
    let mut conn = state
        .redis_conn()
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    let mut best: Option<(AccountRecord, i64)> = None;

    for acct in candidates {
        let key = format!("conc:{}", acct.id);
        // Use ZCARD to count sorted set members (matching concurrency tracking via ZADD).
        let count: i64 = conn.zcard(&key).await.unwrap_or(0);

        if acct.max_concurrency > 0 && count >= acct.max_concurrency as i64 {
            continue; // slot full, skip
        }

        match &best {
            None => best = Some((acct.clone(), count)),
            Some((_, best_count)) if count < *best_count => {
                best = Some((acct.clone(), count));
            }
            _ => {}
        }
    }

    best.map(|(acct, _)| acct)
        .ok_or(AppError::Overloaded)
}

fn platform_string(target: RelayTarget) -> &'static str {
    match target {
        RelayTarget::Claude => "claude",
        RelayTarget::OpenAI => "openai",
        RelayTarget::Gemini => "gemini",
        RelayTarget::Bedrock => "bedrock",
        RelayTarget::Azure => "azure",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_strings() {
        assert_eq!(platform_string(RelayTarget::Claude), "claude");
        assert_eq!(platform_string(RelayTarget::OpenAI), "openai");
        assert_eq!(platform_string(RelayTarget::Gemini), "gemini");
        assert_eq!(platform_string(RelayTarget::Bedrock), "bedrock");
        assert_eq!(platform_string(RelayTarget::Azure), "azure");
    }

    #[test]
    fn model_support_default_allows_all() {
        let account = AccountRecord {
            id: uuid::Uuid::new_v4(),
            name: "test".into(),
            platform: "claude".into(),
            account_type: "oauth".into(),
            credentials_enc: "{}".into(),
            status: "active".into(),
            priority: 0,
            max_concurrency: 5,
            proxy_url: None,
            fingerprint_profile_id: None,
            description: None,
            schedulable: None,
            group_id: None,
            expires_at: None,
            rate_limit: None,
            extra_config: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        assert!(is_model_supported(&account, "claude-opus-4-20250805"));
        assert!(is_model_supported(&account, "claude-sonnet-4-20250514"));
    }
}
