use crate::error::AppError;
use crate::model::account::{AccountRecord, CreateAccountInput, UpdateAccountInput};
use crate::model::apikey::{ApiKeyRecord, CreateApiKeyInput};
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    Json,
};
use serde::Serialize;
use uuid::Uuid;

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

pub async fn create_key(
    State(state): State<AppState>,
    Json(input): Json<CreateApiKeyInput>,
) -> Result<Json<KeyCreatedResponse>, AppError> {
    let raw_key = crate::service::apikey::generate_raw_key();
    let key_hash = crate::middleware::auth::hash_key(&raw_key);
    let id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO api_keys (id, key_hash, name, permissions, daily_cost_limit, total_cost_limit, max_concurrency, rate_limit_rpm, status, expires_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'active', $9)
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
    .bind(input.expires_at)
    .execute(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    Ok(Json(KeyCreatedResponse { id, key: raw_key }))
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

// ───────────────────── Stats ─────────────────────

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

// ───────────────────── Response types ─────────────────────

#[derive(Serialize)]
pub struct DeleteResult {
    deleted: bool,
}

#[derive(Serialize)]
pub struct KeyCreatedResponse {
    id: Uuid,
    key: String,
}

#[derive(Serialize)]
pub struct StatsResponse {
    accounts: i64,
    api_keys: i64,
}
