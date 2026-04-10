use crate::error::AppError;
use crate::model::account::{AccountRecord, CreateAccountInput, UpdateAccountInput};
use crate::model::apikey::{ApiKeyRecord, CreateApiKeyInput, UpdateApiKeyInput};
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{NaiveDate, Utc};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

// ───────────────────── Admin Login ─────────────────────

#[derive(Deserialize)]
pub struct LoginInput {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub username: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(input): Json<LoginInput>,
) -> Result<Json<LoginResponse>, AppError> {
    let password_hash = hex::encode(Sha256::digest(input.password.as_bytes()));

    let row: Option<(Uuid, String)> = sqlx::query_as(
        "SELECT id, username FROM admin_users WHERE username = $1 AND password_hash = $2",
    )
    .bind(&input.username)
    .bind(&password_hash)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    let (_, username) = row.ok_or(AppError::Unauthorized)?;

    let token = generate_jwt(&username, &state.config.auth.jwt_secret, state.config.auth.session_ttl_hours)?;

    Ok(Json(LoginResponse { token, username }))
}

fn generate_jwt(username: &str, secret: &str, ttl_hours: u64) -> Result<String, AppError> {
    let header = URL_SAFE_NO_PAD.encode(r#"{"alg":"HS256","typ":"JWT"}"#.as_bytes());
    let exp = Utc::now().timestamp() + (ttl_hours as i64 * 3600);
    let payload_json = serde_json::json!({
        "sub": username,
        "exp": exp,
        "iat": Utc::now().timestamp()
    });
    let payload = URL_SAFE_NO_PAD.encode(payload_json.to_string().as_bytes());
    let signing_input = format!("{}.{}", header, payload);

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|_| AppError::Internal(anyhow::anyhow!("HMAC init failed")))?;
    mac.update(signing_input.as_bytes());
    let signature = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());

    Ok(format!("{}.{}", signing_input, signature))
}

#[derive(Deserialize)]
pub struct ChangePasswordInput {
    pub old_password: String,
    pub new_password: String,
}

pub async fn change_password(
    State(state): State<AppState>,
    Json(input): Json<ChangePasswordInput>,
) -> Result<Json<serde_json::Value>, AppError> {
    let old_hash = hex::encode(Sha256::digest(input.old_password.as_bytes()));
    let new_hash = hex::encode(Sha256::digest(input.new_password.as_bytes()));

    let result = sqlx::query(
        "UPDATE admin_users SET password_hash = $1 WHERE password_hash = $2",
    )
    .bind(&new_hash)
    .bind(&old_hash)
    .execute(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::BadRequest("incorrect old password".into()));
    }

    Ok(Json(serde_json::json!({ "success": true })))
}

// ───────────────────── Dashboard ─────────────────────

#[derive(Serialize)]
pub struct DashboardResponse {
    pub accounts: AccountStats,
    pub api_keys: ApiKeyStats,
    pub usage: UsageSummary,
}

#[derive(Serialize)]
pub struct AccountStats {
    pub total: i64,
    pub active: i64,
    pub by_platform: Vec<PlatformCount>,
}

#[derive(Serialize)]
pub struct PlatformCount {
    pub platform: String,
    pub count: i64,
}

#[derive(Serialize)]
pub struct ApiKeyStats {
    pub total: i64,
    pub active: i64,
}

#[derive(Serialize)]
pub struct UsageSummary {
    pub total_requests: i64,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_cost_usd: f64,
    pub today_requests: i64,
    pub today_cost_usd: f64,
}

pub async fn dashboard(
    State(state): State<AppState>,
) -> Result<Json<DashboardResponse>, AppError> {
    // Account stats
    let total_accounts: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM accounts")
        .fetch_one(&state.db).await.map_err(|e| AppError::Internal(e.into()))?;

    let active_accounts: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM accounts WHERE status = 'active'"
    ).fetch_one(&state.db).await.map_err(|e| AppError::Internal(e.into()))?;

    let platform_counts: Vec<PlatformCount> = sqlx::query_as::<_, (String, i64)>(
        "SELECT platform, COUNT(*) FROM accounts GROUP BY platform ORDER BY platform"
    )
    .fetch_all(&state.db).await
    .map_err(|e| AppError::Internal(e.into()))?
    .into_iter()
    .map(|(platform, count)| PlatformCount { platform, count })
    .collect();

    // API Key stats
    let total_keys: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM api_keys")
        .fetch_one(&state.db).await.map_err(|e| AppError::Internal(e.into()))?;

    let active_keys: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM api_keys WHERE status = 'active'"
    ).fetch_one(&state.db).await.map_err(|e| AppError::Internal(e.into()))?;

    // Usage stats
    let usage_total: (i64, i64, i64, f64) = sqlx::query_as(
        "SELECT COALESCE(COUNT(*),0)::BIGINT, COALESCE(SUM(input_tokens),0)::BIGINT, COALESCE(SUM(output_tokens),0)::BIGINT, COALESCE(SUM(cost_usd),0)::FLOAT8 FROM usage_logs"
    ).fetch_one(&state.db).await.map_err(|e| AppError::Internal(e.into()))?;

    let today = Utc::now().date_naive();
    let today_usage: (i64, f64) = sqlx::query_as(
        "SELECT COALESCE(COUNT(*),0)::BIGINT, COALESCE(SUM(cost_usd),0)::FLOAT8 FROM usage_logs WHERE created_at::date = $1"
    ).bind(today).fetch_one(&state.db).await.map_err(|e| AppError::Internal(e.into()))?;

    Ok(Json(DashboardResponse {
        accounts: AccountStats {
            total: total_accounts.0,
            active: active_accounts.0,
            by_platform: platform_counts,
        },
        api_keys: ApiKeyStats {
            total: total_keys.0,
            active: active_keys.0,
        },
        usage: UsageSummary {
            total_requests: usage_total.0,
            total_input_tokens: usage_total.1,
            total_output_tokens: usage_total.2,
            total_cost_usd: usage_total.3,
            today_requests: today_usage.0,
            today_cost_usd: today_usage.1,
        },
    }))
}

// ───────────────────── Account CRUD ─────────────────────

pub async fn list_accounts(
    State(state): State<AppState>,
) -> Result<Json<Vec<AccountRecord>>, AppError> {
    let rows = sqlx::query_as::<_, AccountRecord>("SELECT * FROM accounts ORDER BY created_at DESC")
        .fetch_all(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;
    Ok(Json(rows))
}

pub async fn get_account(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AccountRecord>, AppError> {
    let row = sqlx::query_as::<_, AccountRecord>("SELECT * FROM accounts WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or(AppError::NotFound)?;
    Ok(Json(row))
}

pub async fn create_account(
    State(state): State<AppState>,
    Json(input): Json<CreateAccountInput>,
) -> Result<Json<AccountRecord>, AppError> {
    let id = Uuid::new_v4();
    let encrypted_creds =
        crate::util::crypto::encrypt(&state.config.auth.encryption_key, &input.credentials)?;

    let row = sqlx::query_as::<_, AccountRecord>(
        r#"
        INSERT INTO accounts (id, name, platform, account_type, credentials_enc, status, priority, max_concurrency, proxy_url, fingerprint_profile_id)
        VALUES ($1, $2, $3, $4, $5, 'active', $6, $7, $8, $9)
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(&input.name)
    .bind(&input.platform)
    .bind(&input.account_type)
    .bind(&encrypted_creds)
    .bind(input.priority.unwrap_or(50))
    .bind(input.max_concurrency.unwrap_or(1))
    .bind(&input.proxy_url)
    .bind(&input.fingerprint_profile_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    Ok(Json(row))
}

pub async fn update_account(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateAccountInput>,
) -> Result<Json<AccountRecord>, AppError> {
    let row = sqlx::query_as::<_, AccountRecord>(
        r#"
        UPDATE accounts
        SET name = COALESCE($2, name),
            status = COALESCE($3, status),
            priority = COALESCE($4, priority),
            max_concurrency = COALESCE($5, max_concurrency),
            proxy_url = COALESCE($6, proxy_url),
            fingerprint_profile_id = COALESCE($7, fingerprint_profile_id),
            updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(&input.name)
    .bind(&input.status)
    .bind(input.priority)
    .bind(input.max_concurrency)
    .bind(&input.proxy_url)
    .bind(&input.fingerprint_profile_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?
    .ok_or(AppError::NotFound)?;

    Ok(Json(row))
}

pub async fn delete_account(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<DeleteResult>, AppError> {
    let result = sqlx::query("DELETE FROM accounts WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(DeleteResult { deleted: true }))
}

// ───────────────────── API Key CRUD ─────────────────────

pub async fn list_keys(
    State(state): State<AppState>,
) -> Result<Json<Vec<ApiKeyRecord>>, AppError> {
    let rows =
        sqlx::query_as::<_, ApiKeyRecord>("SELECT * FROM api_keys ORDER BY created_at DESC")
            .fetch_all(&state.db)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
    Ok(Json(rows))
}

pub async fn get_key(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiKeyRecord>, AppError> {
    let row = sqlx::query_as::<_, ApiKeyRecord>("SELECT * FROM api_keys WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or(AppError::NotFound)?;
    Ok(Json(row))
}

pub async fn create_key(
    State(state): State<AppState>,
    Json(input): Json<CreateApiKeyInput>,
) -> Result<Json<KeyCreatedResponse>, AppError> {
    let raw_key = crate::service::apikey::generate_raw_key();
    let key_hash = crate::middleware::auth::hash_key(&raw_key);
    let id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO api_keys (id, key_hash, name, permissions, daily_cost_limit, total_cost_limit, max_concurrency, rate_limit_rpm, restricted_models, status, expires_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'active', $10)
        "#,
    )
    .bind(id)
    .bind(&key_hash)
    .bind(&input.name)
    .bind(&serde_json::to_value(&input.permissions).unwrap_or_default())
    .bind(input.daily_cost_limit.unwrap_or(0.0))
    .bind(input.total_cost_limit.unwrap_or(0.0))
    .bind(input.max_concurrency.unwrap_or(0))
    .bind(input.rate_limit_rpm.unwrap_or(0))
    .bind(&serde_json::to_value(&input.restricted_models.clone().unwrap_or_default()).unwrap_or_default())
    .bind(input.expires_at)
    .execute(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    Ok(Json(KeyCreatedResponse { id, key: raw_key }))
}

pub async fn update_key(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateApiKeyInput>,
) -> Result<Json<ApiKeyRecord>, AppError> {
    let row = sqlx::query_as::<_, ApiKeyRecord>(
        r#"
        UPDATE api_keys
        SET name = COALESCE($2, name),
            permissions = COALESCE($3, permissions::jsonb)::text,
            daily_cost_limit = COALESCE($4, daily_cost_limit),
            total_cost_limit = COALESCE($5, total_cost_limit),
            max_concurrency = COALESCE($6, max_concurrency),
            rate_limit_rpm = COALESCE($7, rate_limit_rpm),
            restricted_models = COALESCE($8, restricted_models),
            status = COALESCE($9, status),
            expires_at = COALESCE($10, expires_at)
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(&input.name)
    .bind(input.permissions.as_ref().map(|p| serde_json::to_string(p).unwrap_or_default()))
    .bind(input.daily_cost_limit)
    .bind(input.total_cost_limit)
    .bind(input.max_concurrency)
    .bind(input.rate_limit_rpm)
    .bind(input.restricted_models.as_ref().map(|m| serde_json::to_string(m).unwrap_or_default()))
    .bind(&input.status)
    .bind(input.expires_at)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?
    .ok_or(AppError::NotFound)?;

    Ok(Json(row))
}

pub async fn delete_key(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<DeleteResult>, AppError> {
    let result = sqlx::query("DELETE FROM api_keys WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(DeleteResult { deleted: true }))
}

// ───────────────────── Usage / Stats ─────────────────────

#[derive(Deserialize)]
pub struct UsageQuery {
    pub days: Option<i32>,
    pub api_key_id: Option<Uuid>,
    pub account_id: Option<Uuid>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Serialize)]
pub struct UsageRecordsResponse {
    pub records: Vec<UsageRecordView>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(sqlx::FromRow, Serialize)]
pub struct UsageRecordView {
    pub id: Uuid,
    pub api_key_id: Uuid,
    pub account_id: Uuid,
    pub model: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cost_usd: f64,
    pub created_at: chrono::DateTime<Utc>,
}

pub async fn usage_records(
    State(state): State<AppState>,
    Query(q): Query<UsageQuery>,
) -> Result<Json<UsageRecordsResponse>, AppError> {
    let days = q.days.unwrap_or(30);
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(50).min(200);
    let offset = (page - 1) * page_size;

    let since = Utc::now() - chrono::Duration::days(days as i64);

    let (records, total) = if let Some(key_id) = q.api_key_id {
        let rows = sqlx::query_as::<_, UsageRecordView>(
            "SELECT id, api_key_id, account_id, model, input_tokens, output_tokens, cost_usd, created_at FROM usage_logs WHERE api_key_id = $1 AND created_at >= $2 ORDER BY created_at DESC LIMIT $3 OFFSET $4"
        ).bind(key_id).bind(since).bind(page_size).bind(offset)
        .fetch_all(&state.db).await.map_err(|e| AppError::Internal(e.into()))?;

        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM usage_logs WHERE api_key_id = $1 AND created_at >= $2"
        ).bind(key_id).bind(since)
        .fetch_one(&state.db).await.map_err(|e| AppError::Internal(e.into()))?;

        (rows, count.0)
    } else if let Some(acc_id) = q.account_id {
        let rows = sqlx::query_as::<_, UsageRecordView>(
            "SELECT id, api_key_id, account_id, model, input_tokens, output_tokens, cost_usd, created_at FROM usage_logs WHERE account_id = $1 AND created_at >= $2 ORDER BY created_at DESC LIMIT $3 OFFSET $4"
        ).bind(acc_id).bind(since).bind(page_size).bind(offset)
        .fetch_all(&state.db).await.map_err(|e| AppError::Internal(e.into()))?;

        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM usage_logs WHERE account_id = $1 AND created_at >= $2"
        ).bind(acc_id).bind(since)
        .fetch_one(&state.db).await.map_err(|e| AppError::Internal(e.into()))?;

        (rows, count.0)
    } else {
        let rows = sqlx::query_as::<_, UsageRecordView>(
            "SELECT id, api_key_id, account_id, model, input_tokens, output_tokens, cost_usd, created_at FROM usage_logs WHERE created_at >= $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
        ).bind(since).bind(page_size).bind(offset)
        .fetch_all(&state.db).await.map_err(|e| AppError::Internal(e.into()))?;

        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM usage_logs WHERE created_at >= $1"
        ).bind(since)
        .fetch_one(&state.db).await.map_err(|e| AppError::Internal(e.into()))?;

        (rows, count.0)
    };

    Ok(Json(UsageRecordsResponse {
        records,
        total,
        page,
        page_size,
    }))
}

#[derive(Serialize)]
pub struct UsageTrendPoint {
    pub date: NaiveDate,
    pub requests: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cost_usd: f64,
}

pub async fn usage_trends(
    State(state): State<AppState>,
    Query(q): Query<UsageQuery>,
) -> Result<Json<Vec<UsageTrendPoint>>, AppError> {
    let days = q.days.unwrap_or(30);
    let since = Utc::now() - chrono::Duration::days(days as i64);

    let rows = sqlx::query_as::<_, (NaiveDate, i64, i64, i64, f64)>(
        r#"
        SELECT created_at::date as date,
               COUNT(*)::BIGINT,
               COALESCE(SUM(input_tokens), 0)::BIGINT,
               COALESCE(SUM(output_tokens), 0)::BIGINT,
               COALESCE(SUM(cost_usd), 0)::FLOAT8
        FROM usage_logs
        WHERE created_at >= $1
        GROUP BY created_at::date
        ORDER BY date
        "#,
    )
    .bind(since)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    let points: Vec<UsageTrendPoint> = rows
        .into_iter()
        .map(|(date, requests, input_tokens, output_tokens, cost_usd)| UsageTrendPoint {
            date,
            requests,
            input_tokens,
            output_tokens,
            cost_usd,
        })
        .collect();

    Ok(Json(points))
}

#[derive(Serialize)]
pub struct ModelUsageItem {
    pub model: String,
    pub requests: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cost_usd: f64,
}

pub async fn usage_by_model(
    State(state): State<AppState>,
    Query(q): Query<UsageQuery>,
) -> Result<Json<Vec<ModelUsageItem>>, AppError> {
    let days = q.days.unwrap_or(30);
    let since = Utc::now() - chrono::Duration::days(days as i64);

    let rows = sqlx::query_as::<_, (String, i64, i64, i64, f64)>(
        r#"
        SELECT model, COUNT(*)::BIGINT, COALESCE(SUM(input_tokens),0)::BIGINT,
               COALESCE(SUM(output_tokens),0)::BIGINT, COALESCE(SUM(cost_usd),0)::FLOAT8
        FROM usage_logs WHERE created_at >= $1
        GROUP BY model ORDER BY COUNT(*) DESC
        "#,
    )
    .bind(since)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    let items: Vec<ModelUsageItem> = rows
        .into_iter()
        .map(|(model, requests, input_tokens, output_tokens, cost_usd)| ModelUsageItem {
            model,
            requests,
            input_tokens,
            output_tokens,
            cost_usd,
        })
        .collect();

    Ok(Json(items))
}

pub async fn stats(State(state): State<AppState>) -> Result<Json<StatsResponse>, AppError> {
    let account_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM accounts")
        .fetch_one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    let key_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM api_keys")
        .fetch_one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    Ok(Json(StatsResponse {
        accounts: account_count.0,
        api_keys: key_count.0,
    }))
}

// ───────────────────── Fingerprint Profiles ─────────────────────

pub async fn list_fingerprint_profiles(
    State(state): State<AppState>,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let rows = sqlx::query_scalar::<_, serde_json::Value>(
        "SELECT row_to_json(fp) FROM fingerprint_profiles fp ORDER BY created_at DESC",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;
    Ok(Json(rows))
}

pub async fn get_fingerprint_profile(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let row = sqlx::query_scalar::<_, serde_json::Value>(
        "SELECT row_to_json(fp) FROM fingerprint_profiles fp WHERE fp.id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?
    .ok_or(AppError::NotFound)?;
    Ok(Json(row))
}

pub async fn create_fingerprint_profile(
    State(state): State<AppState>,
    Json(input): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let id = Uuid::new_v4();
    let name = input["name"]
        .as_str()
        .ok_or_else(|| AppError::BadRequest("name required".into()))?;

    sqlx::query(
        r#"
        INSERT INTO fingerprint_profiles (id, name, tls_profile, http2_settings, header_order, user_agent_template, extra_headers)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(id)
    .bind(name)
    .bind(&input["tls_profile"])
    .bind(&input["http2_settings"])
    .bind(&input["header_order"])
    .bind(input["user_agent_template"].as_str().unwrap_or_default())
    .bind(&input["extra_headers"])
    .execute(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    Ok(Json(serde_json::json!({ "id": id, "name": name })))
}

pub async fn update_fingerprint_profile(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query(
        r#"
        UPDATE fingerprint_profiles
        SET name = COALESCE($2, name),
            tls_profile = COALESCE($3, tls_profile),
            http2_settings = COALESCE($4, http2_settings),
            header_order = COALESCE($5, header_order),
            user_agent_template = COALESCE($6, user_agent_template),
            extra_headers = COALESCE($7, extra_headers)
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(input.get("name").and_then(|v| v.as_str()))
    .bind(input.get("tls_profile"))
    .bind(input.get("http2_settings"))
    .bind(input.get("header_order"))
    .bind(input.get("user_agent_template").and_then(|v| v.as_str()))
    .bind(input.get("extra_headers"))
    .execute(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    get_fingerprint_profile(State(state), Path(id)).await
}

pub async fn delete_fingerprint_profile(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<DeleteResult>, AppError> {
    let result = sqlx::query("DELETE FROM fingerprint_profiles WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(DeleteResult { deleted: true }))
}

// ───────────────────── Settings ─────────────────────

pub async fn get_settings(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rows: Vec<(String, serde_json::Value)> =
        sqlx::query_as("SELECT key, value FROM settings ORDER BY key")
            .fetch_all(&state.db)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

    let mut map = serde_json::Map::new();
    for (key, value) in rows {
        map.insert(key, value);
    }
    Ok(Json(serde_json::Value::Object(map)))
}

#[derive(Deserialize)]
pub struct SettingInput {
    pub key: String,
    pub value: serde_json::Value,
}

pub async fn update_setting(
    State(state): State<AppState>,
    Json(input): Json<SettingInput>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query(
        r#"
        INSERT INTO settings (key, value, updated_at) VALUES ($1, $2, NOW())
        ON CONFLICT (key) DO UPDATE SET value = $2, updated_at = NOW()
        "#,
    )
    .bind(&input.key)
    .bind(&input.value)
    .execute(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    Ok(Json(serde_json::json!({ "success": true })))
}

// ───────────────────── Response types ─────────────────────

#[derive(Serialize)]
pub struct DeleteResult {
    pub deleted: bool,
}

#[derive(Serialize)]
pub struct KeyCreatedResponse {
    pub id: Uuid,
    pub key: String,
}

#[derive(Serialize)]
pub struct StatsResponse {
    pub accounts: i64,
    pub api_keys: i64,
}
