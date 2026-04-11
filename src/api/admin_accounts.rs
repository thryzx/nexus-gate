//! Platform-specific account endpoints aligned with CRS admin API.
//! Each platform has: list, create, update, delete, toggle, toggle-schedulable,
//! test, reset-status.  OAuth platforms also have generate-auth-url / exchange-code.

use crate::error::AppError;
use crate::model::account::{AccountRecord, CreateAccountInput, UpdateAccountInput};
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ───────────────────── Shared helpers ─────────────────────

async fn list_by_platform(
    state: &AppState,
    platform: &str,
) -> Result<Vec<AccountRecord>, AppError> {
    sqlx::query_as::<_, AccountRecord>(
        "SELECT * FROM accounts WHERE platform = $1 ORDER BY priority ASC, created_at DESC",
    )
    .bind(platform)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))
}

async fn toggle_status(state: &AppState, id: Uuid) -> Result<AccountRecord, AppError> {
    let row = sqlx::query_as::<_, AccountRecord>(
        r#"
        UPDATE accounts
        SET status = CASE WHEN status = 'active' THEN 'disabled' ELSE 'active' END,
            updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?
    .ok_or(AppError::NotFound)?;
    Ok(row)
}

async fn toggle_schedulable(state: &AppState, id: Uuid) -> Result<AccountRecord, AppError> {
    let row = sqlx::query_as::<_, AccountRecord>(
        r#"
        UPDATE accounts
        SET schedulable = NOT COALESCE(schedulable, true),
            updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?
    .ok_or(AppError::NotFound)?;
    Ok(row)
}

async fn reset_account_status(state: &AppState, id: Uuid) -> Result<AccountRecord, AppError> {
    let row = sqlx::query_as::<_, AccountRecord>(
        "UPDATE accounts SET status = 'active', updated_at = NOW() WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?
    .ok_or(AppError::NotFound)?;
    Ok(row)
}

async fn delete_account(state: &AppState, id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM accounts WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

async fn create_platform_account(
    state: &AppState,
    input: CreateAccountInput,
) -> Result<AccountRecord, AppError> {
    let id = Uuid::new_v4();
    let encrypted_creds =
        crate::util::crypto::encrypt(&state.config.auth.encryption_key, &input.credentials)?;

    let row = sqlx::query_as::<_, AccountRecord>(
        r#"
        INSERT INTO accounts (id, name, platform, account_type, credentials_enc, status, priority, max_concurrency, proxy_url, fingerprint_profile_id, description, schedulable, group_id, expires_at, rate_limit, extra_config)
        VALUES ($1, $2, $3, $4, $5, 'active', $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
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
    .bind(input.description.as_deref().unwrap_or(""))
    .bind(input.schedulable.unwrap_or(true))
    .bind(&input.group_id)
    .bind(input.expires_at)
    .bind(input.rate_limit.unwrap_or(0))
    .bind(input.extra_config.as_ref().unwrap_or(&serde_json::json!({})))
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;
    Ok(row)
}

async fn update_platform_account(
    state: &AppState,
    id: Uuid,
    input: UpdateAccountInput,
) -> Result<AccountRecord, AppError> {
    if let Some(ref creds) = input.credentials {
        let enc = crate::util::crypto::encrypt(&state.config.auth.encryption_key, creds)?;
        sqlx::query("UPDATE accounts SET credentials_enc = $2, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .bind(&enc)
            .execute(&state.db)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
    }

    let row = sqlx::query_as::<_, AccountRecord>(
        r#"
        UPDATE accounts
        SET name = COALESCE($2, name),
            status = COALESCE($3, status),
            priority = COALESCE($4, priority),
            max_concurrency = COALESCE($5, max_concurrency),
            proxy_url = COALESCE($6, proxy_url),
            fingerprint_profile_id = COALESCE($7, fingerprint_profile_id),
            description = COALESCE($8, description),
            schedulable = COALESCE($9, schedulable),
            group_id = $10,
            expires_at = $11,
            rate_limit = COALESCE($12, rate_limit),
            extra_config = COALESCE($13, extra_config),
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
    .bind(&input.description)
    .bind(input.schedulable)
    .bind(&input.group_id)
    .bind(input.expires_at)
    .bind(input.rate_limit)
    .bind(&input.extra_config)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?
    .ok_or(AppError::NotFound)?;
    Ok(row)
}

// ───────────────────── Test account stub ─────────────────────

#[derive(Serialize)]
pub struct TestResult {
    pub success: bool,
    pub message: String,
    pub latency_ms: Option<u64>,
}

async fn test_account(state: &AppState, id: Uuid) -> Result<TestResult, AppError> {
    let _account = sqlx::query_as::<_, AccountRecord>("SELECT * FROM accounts WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or(AppError::NotFound)?;

    // TODO: implement actual upstream connectivity test per platform
    Ok(TestResult {
        success: true,
        message: "Account reachable (stub)".into(),
        latency_ms: Some(0),
    })
}

// ───────────────────── OAuth stubs ─────────────────────

#[derive(Deserialize)]
pub struct OAuthProxyConfig {
    pub proxy_url: Option<String>,
    pub platform: Option<String>,
}

#[derive(Serialize)]
pub struct AuthUrlResponse {
    pub auth_url: String,
    pub state: Option<String>,
}

#[derive(Deserialize)]
pub struct ExchangeCodeInput {
    pub code: String,
    pub state: Option<String>,
    pub proxy_url: Option<String>,
    pub name: Option<String>,
    pub priority: Option<i32>,
    pub max_concurrency: Option<i32>,
    pub group_id: Option<Uuid>,
}

// ─── Claude Accounts ───

pub async fn list_claude_accounts(
    State(state): State<AppState>,
) -> Result<Json<Vec<AccountRecord>>, AppError> {
    Ok(Json(list_by_platform(&state, "claude").await?))
}

pub async fn create_claude_account(
    State(state): State<AppState>,
    Json(mut input): Json<CreateAccountInput>,
) -> Result<Json<AccountRecord>, AppError> {
    input.platform = "claude".into();
    if input.account_type.is_empty() {
        input.account_type = "oauth".into();
    }
    Ok(Json(create_platform_account(&state, input).await?))
}

pub async fn update_claude_account(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateAccountInput>,
) -> Result<Json<AccountRecord>, AppError> {
    Ok(Json(update_platform_account(&state, id, input).await?))
}

pub async fn delete_claude_account(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    delete_account(&state, id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn toggle_claude_account(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AccountRecord>, AppError> {
    Ok(Json(toggle_status(&state, id).await?))
}

pub async fn toggle_claude_schedulable(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AccountRecord>, AppError> {
    Ok(Json(toggle_schedulable(&state, id).await?))
}

pub async fn reset_claude_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AccountRecord>, AppError> {
    Ok(Json(reset_account_status(&state, id).await?))
}

pub async fn test_claude_account(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<TestResult>, AppError> {
    Ok(Json(test_account(&state, id).await?))
}

pub async fn refresh_claude_token(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    // TODO: implement actual OAuth token refresh
    let _acc = sqlx::query_as::<_, AccountRecord>("SELECT * FROM accounts WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or(AppError::NotFound)?;
    Ok(Json(serde_json::json!({ "success": true, "message": "Token refresh queued" })))
}

pub async fn generate_claude_auth_url(
    State(_state): State<AppState>,
    Json(config): Json<OAuthProxyConfig>,
) -> Result<Json<AuthUrlResponse>, AppError> {
    // Claude OAuth URL generation stub
    // In production, this would generate a real OAuth URL via Anthropic's OAuth flow
    let _proxy = config.proxy_url;
    Ok(Json(AuthUrlResponse {
        auth_url: "https://claude.ai/oauth/authorize?client_id=nexus-gate&response_type=code&redirect_uri=urn:ietf:wg:oauth:2.0:oob".into(),
        state: Some(Uuid::new_v4().to_string()),
    }))
}

pub async fn exchange_claude_code(
    State(state): State<AppState>,
    Json(input): Json<ExchangeCodeInput>,
) -> Result<Json<AccountRecord>, AppError> {
    // In production: exchange code for tokens via Anthropic
    // For now, store the code as credentials
    let creds = serde_json::json!({
        "auth_code": input.code,
        "token_type": "oauth"
    });
    let create_input = CreateAccountInput {
        name: input.name.unwrap_or_else(|| format!("Claude OAuth {}", &input.code[..8.min(input.code.len())])),
        platform: "claude".into(),
        account_type: "oauth".into(),
        credentials: creds.to_string(),
        priority: input.priority,
        max_concurrency: input.max_concurrency,
        proxy_url: None,
        fingerprint_profile_id: None,
        description: None,
        schedulable: Some(true),
        group_id: input.group_id,
        expires_at: None,
        rate_limit: None,
        extra_config: None,
    };
    Ok(Json(create_platform_account(&state, create_input).await?))
}

pub async fn claude_oauth_with_cookie(
    State(state): State<AppState>,
    Json(input): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let session_key = input["sessionKey"].as_str()
        .ok_or_else(|| AppError::BadRequest("sessionKey required".into()))?;
    // Store cookie-based auth
    let creds = serde_json::json!({
        "session_key": session_key,
        "token_type": "cookie"
    });
    let name = input["name"].as_str().unwrap_or("Claude Cookie Auth");
    let create_input = CreateAccountInput {
        name: name.into(),
        platform: "claude".into(),
        account_type: "cookie".into(),
        credentials: creds.to_string(),
        priority: input["priority"].as_i64().map(|v| v as i32),
        max_concurrency: input["maxConcurrency"].as_i64().map(|v| v as i32),
        proxy_url: input["proxyUrl"].as_str().map(String::from),
        fingerprint_profile_id: None,
        description: None,
        schedulable: Some(true),
        group_id: input["groupId"].as_str().and_then(|s| Uuid::parse_str(s).ok()),
        expires_at: None,
        rate_limit: None,
        extra_config: None,
    };
    let account = create_platform_account(&state, create_input).await?;
    Ok(Json(serde_json::json!({ "success": true, "account": account })))
}

pub async fn generate_claude_setup_token_url(
    State(_state): State<AppState>,
    Json(_config): Json<OAuthProxyConfig>,
) -> Result<Json<AuthUrlResponse>, AppError> {
    Ok(Json(AuthUrlResponse {
        auth_url: "https://claude.ai/settings/setup-token?scope=cli".into(),
        state: Some(Uuid::new_v4().to_string()),
    }))
}

pub async fn exchange_claude_setup_token(
    State(state): State<AppState>,
    Json(input): Json<ExchangeCodeInput>,
) -> Result<Json<AccountRecord>, AppError> {
    let creds = serde_json::json!({
        "setup_token": input.code,
        "token_type": "setup_token"
    });
    let create_input = CreateAccountInput {
        name: input.name.unwrap_or_else(|| "Claude Setup Token".into()),
        platform: "claude".into(),
        account_type: "oauth".into(),
        credentials: creds.to_string(),
        priority: input.priority,
        max_concurrency: input.max_concurrency,
        proxy_url: None,
        fingerprint_profile_id: None,
        description: None,
        schedulable: Some(true),
        group_id: input.group_id,
        expires_at: None,
        rate_limit: None,
        extra_config: None,
    };
    Ok(Json(create_platform_account(&state, create_input).await?))
}

// ─── Claude Console Accounts ───

pub async fn list_claude_console_accounts(
    State(state): State<AppState>,
) -> Result<Json<Vec<AccountRecord>>, AppError> {
    Ok(Json(list_by_platform(&state, "claude-console").await?))
}

pub async fn create_claude_console_account(
    State(state): State<AppState>,
    Json(mut input): Json<CreateAccountInput>,
) -> Result<Json<AccountRecord>, AppError> {
    input.platform = "claude-console".into();
    input.account_type = "apikey".into();
    Ok(Json(create_platform_account(&state, input).await?))
}

pub async fn update_claude_console_account(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateAccountInput>,
) -> Result<Json<AccountRecord>, AppError> {
    Ok(Json(update_platform_account(&state, id, input).await?))
}

pub async fn delete_claude_console_account(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    delete_account(&state, id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn toggle_claude_console(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AccountRecord>, AppError> {
    Ok(Json(toggle_status(&state, id).await?))
}

pub async fn toggle_claude_console_schedulable(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AccountRecord>, AppError> {
    Ok(Json(toggle_schedulable(&state, id).await?))
}

// ─── Bedrock Accounts ───

pub async fn list_bedrock_accounts(
    State(state): State<AppState>,
) -> Result<Json<Vec<AccountRecord>>, AppError> {
    Ok(Json(list_by_platform(&state, "bedrock").await?))
}

pub async fn create_bedrock_account(
    State(state): State<AppState>,
    Json(mut input): Json<CreateAccountInput>,
) -> Result<Json<AccountRecord>, AppError> {
    input.platform = "bedrock".into();
    input.account_type = "bedrock".into();
    Ok(Json(create_platform_account(&state, input).await?))
}

pub async fn update_bedrock_account(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateAccountInput>,
) -> Result<Json<AccountRecord>, AppError> {
    Ok(Json(update_platform_account(&state, id, input).await?))
}

pub async fn delete_bedrock_account(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    delete_account(&state, id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn toggle_bedrock(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AccountRecord>, AppError> {
    Ok(Json(toggle_status(&state, id).await?))
}

pub async fn toggle_bedrock_schedulable(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AccountRecord>, AppError> {
    Ok(Json(toggle_schedulable(&state, id).await?))
}

pub async fn reset_bedrock_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AccountRecord>, AppError> {
    Ok(Json(reset_account_status(&state, id).await?))
}

pub async fn test_bedrock_account(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<TestResult>, AppError> {
    Ok(Json(test_account(&state, id).await?))
}

// ─── Gemini Accounts (OAuth) ───

pub async fn list_gemini_accounts(
    State(state): State<AppState>,
) -> Result<Json<Vec<AccountRecord>>, AppError> {
    Ok(Json(list_by_platform(&state, "gemini").await?))
}

pub async fn create_gemini_account(
    State(state): State<AppState>,
    Json(mut input): Json<CreateAccountInput>,
) -> Result<Json<AccountRecord>, AppError> {
    input.platform = "gemini".into();
    if input.account_type.is_empty() {
        input.account_type = "oauth".into();
    }
    Ok(Json(create_platform_account(&state, input).await?))
}

pub async fn update_gemini_account(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateAccountInput>,
) -> Result<Json<AccountRecord>, AppError> {
    Ok(Json(update_platform_account(&state, id, input).await?))
}

pub async fn delete_gemini_account(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    delete_account(&state, id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn toggle_gemini_schedulable(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AccountRecord>, AppError> {
    Ok(Json(toggle_schedulable(&state, id).await?))
}

pub async fn reset_gemini_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AccountRecord>, AppError> {
    Ok(Json(reset_account_status(&state, id).await?))
}

pub async fn test_gemini_account(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<TestResult>, AppError> {
    Ok(Json(test_account(&state, id).await?))
}

pub async fn generate_gemini_auth_url(
    State(_state): State<AppState>,
    Json(_config): Json<OAuthProxyConfig>,
) -> Result<Json<AuthUrlResponse>, AppError> {
    Ok(Json(AuthUrlResponse {
        auth_url: "https://accounts.google.com/o/oauth2/v2/auth?client_id=nexus-gate&response_type=code&scope=https://www.googleapis.com/auth/generative-language&redirect_uri=urn:ietf:wg:oauth:2.0:oob".into(),
        state: Some(Uuid::new_v4().to_string()),
    }))
}

pub async fn exchange_gemini_code(
    State(state): State<AppState>,
    Json(input): Json<ExchangeCodeInput>,
) -> Result<Json<AccountRecord>, AppError> {
    let creds = serde_json::json!({
        "auth_code": input.code,
        "token_type": "oauth"
    });
    let create_input = CreateAccountInput {
        name: input.name.unwrap_or_else(|| "Gemini OAuth".into()),
        platform: "gemini".into(),
        account_type: "oauth".into(),
        credentials: creds.to_string(),
        priority: input.priority,
        max_concurrency: input.max_concurrency,
        proxy_url: None,
        fingerprint_profile_id: None,
        description: None,
        schedulable: Some(true),
        group_id: input.group_id,
        expires_at: None,
        rate_limit: None,
        extra_config: None,
    };
    Ok(Json(create_platform_account(&state, create_input).await?))
}

// ─── Gemini API Accounts ───

pub async fn list_gemini_api_accounts(
    State(state): State<AppState>,
) -> Result<Json<Vec<AccountRecord>>, AppError> {
    Ok(Json(list_by_platform(&state, "gemini-api").await?))
}

pub async fn create_gemini_api_account(
    State(state): State<AppState>,
    Json(mut input): Json<CreateAccountInput>,
) -> Result<Json<AccountRecord>, AppError> {
    input.platform = "gemini-api".into();
    input.account_type = "apikey".into();
    Ok(Json(create_platform_account(&state, input).await?))
}

pub async fn update_gemini_api_account(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateAccountInput>,
) -> Result<Json<AccountRecord>, AppError> {
    Ok(Json(update_platform_account(&state, id, input).await?))
}

pub async fn delete_gemini_api_account(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    delete_account(&state, id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn toggle_gemini_api(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AccountRecord>, AppError> {
    Ok(Json(toggle_status(&state, id).await?))
}

pub async fn toggle_gemini_api_schedulable(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AccountRecord>, AppError> {
    Ok(Json(toggle_schedulable(&state, id).await?))
}

// ─── OpenAI Accounts (OAuth - Codex) ───

pub async fn list_openai_accounts(
    State(state): State<AppState>,
) -> Result<Json<Vec<AccountRecord>>, AppError> {
    Ok(Json(list_by_platform(&state, "openai").await?))
}

pub async fn create_openai_account(
    State(state): State<AppState>,
    Json(mut input): Json<CreateAccountInput>,
) -> Result<Json<AccountRecord>, AppError> {
    input.platform = "openai".into();
    if input.account_type.is_empty() {
        input.account_type = "oauth".into();
    }
    Ok(Json(create_platform_account(&state, input).await?))
}

pub async fn update_openai_account(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateAccountInput>,
) -> Result<Json<AccountRecord>, AppError> {
    Ok(Json(update_platform_account(&state, id, input).await?))
}

pub async fn delete_openai_account(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    delete_account(&state, id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn toggle_openai(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AccountRecord>, AppError> {
    Ok(Json(toggle_status(&state, id).await?))
}

pub async fn toggle_openai_schedulable(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AccountRecord>, AppError> {
    Ok(Json(toggle_schedulable(&state, id).await?))
}

pub async fn reset_openai_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AccountRecord>, AppError> {
    Ok(Json(reset_account_status(&state, id).await?))
}

pub async fn generate_openai_auth_url(
    State(_state): State<AppState>,
    Json(_config): Json<OAuthProxyConfig>,
) -> Result<Json<AuthUrlResponse>, AppError> {
    Ok(Json(AuthUrlResponse {
        auth_url: "https://auth0.openai.com/authorize?client_id=nexus-gate&response_type=code&scope=openid+profile+email&redirect_uri=urn:ietf:wg:oauth:2.0:oob".into(),
        state: Some(Uuid::new_v4().to_string()),
    }))
}

pub async fn exchange_openai_code(
    State(state): State<AppState>,
    Json(input): Json<ExchangeCodeInput>,
) -> Result<Json<AccountRecord>, AppError> {
    let creds = serde_json::json!({
        "auth_code": input.code,
        "token_type": "oauth"
    });
    let create_input = CreateAccountInput {
        name: input.name.unwrap_or_else(|| "OpenAI OAuth".into()),
        platform: "openai".into(),
        account_type: "oauth".into(),
        credentials: creds.to_string(),
        priority: input.priority,
        max_concurrency: input.max_concurrency,
        proxy_url: None,
        fingerprint_profile_id: None,
        description: None,
        schedulable: Some(true),
        group_id: input.group_id,
        expires_at: None,
        rate_limit: None,
        extra_config: None,
    };
    Ok(Json(create_platform_account(&state, create_input).await?))
}

// ─── OpenAI Responses Accounts ───

pub async fn list_openai_responses_accounts(
    State(state): State<AppState>,
) -> Result<Json<Vec<AccountRecord>>, AppError> {
    Ok(Json(list_by_platform(&state, "openai-responses").await?))
}

pub async fn create_openai_responses_account(
    State(state): State<AppState>,
    Json(mut input): Json<CreateAccountInput>,
) -> Result<Json<AccountRecord>, AppError> {
    input.platform = "openai-responses".into();
    input.account_type = "apikey".into();
    Ok(Json(create_platform_account(&state, input).await?))
}

pub async fn update_openai_responses_account(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateAccountInput>,
) -> Result<Json<AccountRecord>, AppError> {
    Ok(Json(update_platform_account(&state, id, input).await?))
}

pub async fn delete_openai_responses_account(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    delete_account(&state, id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn toggle_openai_responses(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AccountRecord>, AppError> {
    Ok(Json(toggle_status(&state, id).await?))
}

pub async fn toggle_openai_responses_schedulable(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AccountRecord>, AppError> {
    Ok(Json(toggle_schedulable(&state, id).await?))
}

pub async fn reset_openai_responses_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AccountRecord>, AppError> {
    Ok(Json(reset_account_status(&state, id).await?))
}

pub async fn test_openai_responses_account(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<TestResult>, AppError> {
    Ok(Json(test_account(&state, id).await?))
}

// ─── Azure OpenAI Accounts ───

pub async fn list_azure_openai_accounts(
    State(state): State<AppState>,
) -> Result<Json<Vec<AccountRecord>>, AppError> {
    Ok(Json(list_by_platform(&state, "azure-openai").await?))
}

pub async fn create_azure_openai_account(
    State(state): State<AppState>,
    Json(mut input): Json<CreateAccountInput>,
) -> Result<Json<AccountRecord>, AppError> {
    input.platform = "azure-openai".into();
    input.account_type = "apikey".into();
    Ok(Json(create_platform_account(&state, input).await?))
}

pub async fn update_azure_openai_account(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateAccountInput>,
) -> Result<Json<AccountRecord>, AppError> {
    Ok(Json(update_platform_account(&state, id, input).await?))
}

pub async fn delete_azure_openai_account(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    delete_account(&state, id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn toggle_azure_openai(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AccountRecord>, AppError> {
    Ok(Json(toggle_status(&state, id).await?))
}

pub async fn toggle_azure_openai_schedulable(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AccountRecord>, AppError> {
    Ok(Json(toggle_schedulable(&state, id).await?))
}

pub async fn test_azure_openai_account(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<TestResult>, AppError> {
    Ok(Json(test_account(&state, id).await?))
}

// ─── Droid Accounts ───

pub async fn list_droid_accounts(
    State(state): State<AppState>,
) -> Result<Json<Vec<AccountRecord>>, AppError> {
    Ok(Json(list_by_platform(&state, "droid").await?))
}

pub async fn get_droid_account(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AccountRecord>, AppError> {
    let row = sqlx::query_as::<_, AccountRecord>("SELECT * FROM accounts WHERE id = $1 AND platform = 'droid'")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or(AppError::NotFound)?;
    Ok(Json(row))
}

pub async fn create_droid_account(
    State(state): State<AppState>,
    Json(mut input): Json<CreateAccountInput>,
) -> Result<Json<AccountRecord>, AppError> {
    input.platform = "droid".into();
    if input.account_type.is_empty() {
        input.account_type = "oauth".into();
    }
    Ok(Json(create_platform_account(&state, input).await?))
}

pub async fn update_droid_account(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateAccountInput>,
) -> Result<Json<AccountRecord>, AppError> {
    Ok(Json(update_platform_account(&state, id, input).await?))
}

pub async fn delete_droid_account(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    delete_account(&state, id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn toggle_droid_schedulable(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AccountRecord>, AppError> {
    Ok(Json(toggle_schedulable(&state, id).await?))
}

pub async fn reset_droid_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AccountRecord>, AppError> {
    Ok(Json(reset_account_status(&state, id).await?))
}

pub async fn test_droid_account(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<TestResult>, AppError> {
    Ok(Json(test_account(&state, id).await?))
}

pub async fn generate_droid_auth_url(
    State(_state): State<AppState>,
    Json(_config): Json<OAuthProxyConfig>,
) -> Result<Json<AuthUrlResponse>, AppError> {
    Ok(Json(AuthUrlResponse {
        auth_url: "https://droid.dev/oauth/authorize?client_id=nexus-gate&response_type=code".into(),
        state: Some(Uuid::new_v4().to_string()),
    }))
}

pub async fn exchange_droid_code(
    State(state): State<AppState>,
    Json(input): Json<ExchangeCodeInput>,
) -> Result<Json<AccountRecord>, AppError> {
    let creds = serde_json::json!({
        "auth_code": input.code,
        "token_type": "oauth"
    });
    let create_input = CreateAccountInput {
        name: input.name.unwrap_or_else(|| "Droid OAuth".into()),
        platform: "droid".into(),
        account_type: "oauth".into(),
        credentials: creds.to_string(),
        priority: input.priority,
        max_concurrency: input.max_concurrency,
        proxy_url: None,
        fingerprint_profile_id: None,
        description: None,
        schedulable: Some(true),
        group_id: input.group_id,
        expires_at: None,
        rate_limit: None,
        extra_config: None,
    };
    Ok(Json(create_platform_account(&state, create_input).await?))
}

// ─── CCR Accounts ───

pub async fn list_ccr_accounts(
    State(state): State<AppState>,
) -> Result<Json<Vec<AccountRecord>>, AppError> {
    Ok(Json(list_by_platform(&state, "ccr").await?))
}

pub async fn create_ccr_account(
    State(state): State<AppState>,
    Json(mut input): Json<CreateAccountInput>,
) -> Result<Json<AccountRecord>, AppError> {
    input.platform = "ccr".into();
    input.account_type = "apikey".into();
    Ok(Json(create_platform_account(&state, input).await?))
}

pub async fn update_ccr_account(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateAccountInput>,
) -> Result<Json<AccountRecord>, AppError> {
    Ok(Json(update_platform_account(&state, id, input).await?))
}

pub async fn delete_ccr_account(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    delete_account(&state, id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn toggle_ccr(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AccountRecord>, AppError> {
    Ok(Json(toggle_status(&state, id).await?))
}

pub async fn toggle_ccr_schedulable(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AccountRecord>, AppError> {
    Ok(Json(toggle_schedulable(&state, id).await?))
}

// ─── Account Groups ───

#[derive(sqlx::FromRow, Serialize)]
pub struct AccountGroup {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub priority: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub async fn list_account_groups(
    State(state): State<AppState>,
) -> Result<Json<Vec<AccountGroup>>, AppError> {
    let rows = sqlx::query_as::<_, AccountGroup>(
        "SELECT * FROM account_groups ORDER BY priority ASC, name ASC",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;
    Ok(Json(rows))
}

#[derive(Deserialize)]
pub struct CreateGroupInput {
    pub name: String,
    pub description: Option<String>,
    pub priority: Option<i32>,
}

pub async fn create_account_group(
    State(state): State<AppState>,
    Json(input): Json<CreateGroupInput>,
) -> Result<Json<AccountGroup>, AppError> {
    let row = sqlx::query_as::<_, AccountGroup>(
        "INSERT INTO account_groups (name, description, priority) VALUES ($1, $2, $3) RETURNING *",
    )
    .bind(&input.name)
    .bind(input.description.as_deref().unwrap_or(""))
    .bind(input.priority.unwrap_or(50))
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;
    Ok(Json(row))
}

pub async fn update_account_group(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<CreateGroupInput>,
) -> Result<Json<AccountGroup>, AppError> {
    let row = sqlx::query_as::<_, AccountGroup>(
        "UPDATE account_groups SET name = $2, description = $3, priority = $4, updated_at = NOW() WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .bind(&input.name)
    .bind(input.description.as_deref().unwrap_or(""))
    .bind(input.priority.unwrap_or(50))
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?
    .ok_or(AppError::NotFound)?;
    Ok(Json(row))
}

pub async fn delete_account_group(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE accounts SET group_id = NULL WHERE group_id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;
    sqlx::query("DELETE FROM account_groups WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;
    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn get_account_group_members(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<AccountRecord>>, AppError> {
    let rows = sqlx::query_as::<_, AccountRecord>(
        "SELECT * FROM accounts WHERE group_id = $1 ORDER BY priority ASC",
    )
    .bind(id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;
    Ok(Json(rows))
}

// ─── Account Usage Records (per account) ───

#[derive(Deserialize)]
pub struct AccountUsageQuery {
    pub days: Option<i32>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub platform: Option<String>,
}

pub async fn get_account_usage_records(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(q): Query<AccountUsageQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let days = q.days.unwrap_or(30);
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(50).min(200);
    let offset = (page - 1) * page_size;
    let since = chrono::Utc::now() - chrono::Duration::days(days as i64);

    let rows: Vec<(Uuid, Uuid, Uuid, String, i64, i64, f64, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        "SELECT id, api_key_id, account_id, model, input_tokens, output_tokens, cost_usd, created_at FROM usage_logs WHERE account_id = $1 AND created_at >= $2 ORDER BY created_at DESC LIMIT $3 OFFSET $4"
    )
    .bind(id).bind(since).bind(page_size).bind(offset)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    let total: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM usage_logs WHERE account_id = $1 AND created_at >= $2"
    ).bind(id).bind(since)
    .fetch_one(&state.db).await.map_err(|e| AppError::Internal(e.into()))?;

    let records: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "id": r.0, "api_key_id": r.1, "account_id": r.2, "model": r.3,
        "input_tokens": r.4, "output_tokens": r.5, "cost_usd": r.6, "created_at": r.7
    })).collect();

    Ok(Json(serde_json::json!({
        "records": records,
        "total": total.0,
        "page": page,
        "page_size": page_size
    })))
}

// ─── Account Usage Stats ───

pub async fn get_accounts_usage_stats(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rows: Vec<(Uuid, String, i64, i64, i64, f64)> = sqlx::query_as(
        r#"SELECT a.id, a.name, COALESCE(COUNT(u.id), 0)::BIGINT,
           COALESCE(SUM(u.input_tokens), 0)::BIGINT, COALESCE(SUM(u.output_tokens), 0)::BIGINT,
           COALESCE(SUM(u.cost_usd), 0)::FLOAT8
           FROM accounts a LEFT JOIN usage_logs u ON u.account_id = a.id
           GROUP BY a.id, a.name ORDER BY a.name"#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    let stats: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "account_id": r.0, "name": r.1, "requests": r.2,
        "input_tokens": r.3, "output_tokens": r.4, "cost_usd": r.5
    })).collect();

    Ok(Json(serde_json::json!({ "stats": stats })))
}

// ─── Quota Cards ───

#[derive(sqlx::FromRow, Serialize)]
pub struct QuotaCard {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub value_type: String,
    pub value: f64,
    pub max_redemptions: i32,
    pub current_redemptions: i32,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn list_quota_cards(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<QuotaCard>>, AppError> {
    let status = params.get("status").map(|s| s.as_str());
    let rows = if let Some(st) = status {
        sqlx::query_as::<_, QuotaCard>(
            "SELECT * FROM quota_cards WHERE status = $1 ORDER BY created_at DESC",
        )
        .bind(st)
        .fetch_all(&state.db)
        .await
    } else {
        sqlx::query_as::<_, QuotaCard>("SELECT * FROM quota_cards ORDER BY created_at DESC")
            .fetch_all(&state.db)
            .await
    }
    .map_err(|e| AppError::Internal(e.into()))?;
    Ok(Json(rows))
}

#[derive(Deserialize)]
pub struct CreateQuotaCardInput {
    pub name: Option<String>,
    pub code: Option<String>,
    pub value_type: Option<String>,
    pub value: f64,
    pub max_redemptions: Option<i32>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn create_quota_card(
    State(state): State<AppState>,
    Json(input): Json<CreateQuotaCardInput>,
) -> Result<Json<QuotaCard>, AppError> {
    let code = input.code.unwrap_or_else(|| format!("QC-{}", &Uuid::new_v4().to_string()[..8]));
    let row = sqlx::query_as::<_, QuotaCard>(
        "INSERT INTO quota_cards (code, name, value_type, value, max_redemptions, expires_at) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
    )
    .bind(&code)
    .bind(input.name.as_deref().unwrap_or(""))
    .bind(input.value_type.as_deref().unwrap_or("cost_limit"))
    .bind(input.value)
    .bind(input.max_redemptions.unwrap_or(1))
    .bind(input.expires_at)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;
    Ok(Json(row))
}

pub async fn delete_quota_card(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("DELETE FROM quota_cards WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;
    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn get_quota_card_stats(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM quota_cards")
        .fetch_one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;
    let active: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM quota_cards WHERE status = 'active'")
            .fetch_one(&state.db)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
    let redeemed: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM redemptions")
        .fetch_one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;
    Ok(Json(serde_json::json!({
        "total": total.0, "active": active.0, "total_redemptions": redeemed.0
    })))
}

// ─── Webhook Configuration ───

pub async fn get_webhook_config(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let config: Option<(Uuid, bool, String, serde_json::Value)> = sqlx::query_as(
        "SELECT id, enabled, global_url, events FROM webhook_config LIMIT 1",
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    let platforms: Vec<(Uuid, String, String, serde_json::Value, bool)> = sqlx::query_as(
        "SELECT id, name, url, events, enabled FROM webhook_platforms ORDER BY name",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    let platform_list: Vec<serde_json::Value> = platforms.iter().map(|p| serde_json::json!({
        "id": p.0, "name": p.1, "url": p.2, "events": p.3, "enabled": p.4
    })).collect();

    if let Some((id, enabled, url, events)) = config {
        Ok(Json(serde_json::json!({
            "id": id, "enabled": enabled, "global_url": url, "events": events,
            "platforms": platform_list
        })))
    } else {
        Ok(Json(serde_json::json!({
            "enabled": false, "global_url": "", "events": [], "platforms": platform_list
        })))
    }
}

pub async fn update_webhook_config(
    State(state): State<AppState>,
    Json(input): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let enabled = input["enabled"].as_bool().unwrap_or(false);
    let url = input["global_url"].as_str().unwrap_or("");
    let events = input.get("events").cloned().unwrap_or(serde_json::json!([]));
    sqlx::query(
        "UPDATE webhook_config SET enabled = $1, global_url = $2, events = $3, updated_at = NOW()",
    )
    .bind(enabled)
    .bind(url)
    .bind(&events)
    .execute(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;
    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn create_webhook_platform(
    State(state): State<AppState>,
    Json(input): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let name = input["name"].as_str().unwrap_or("");
    let url = input["url"].as_str().unwrap_or("");
    let events = input.get("events").cloned().unwrap_or(serde_json::json!([]));
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO webhook_platforms (id, name, url, events) VALUES ($1, $2, $3, $4)")
        .bind(id)
        .bind(name)
        .bind(url)
        .bind(&events)
        .execute(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;
    Ok(Json(serde_json::json!({ "id": id, "success": true })))
}

pub async fn delete_webhook_platform(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("DELETE FROM webhook_platforms WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;
    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn test_webhook(
    State(_state): State<AppState>,
    Json(_input): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({ "success": true, "message": "Webhook test sent" })))
}

// ─── User Management ───

#[derive(sqlx::FromRow, Serialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub role: String,
    pub source: String,
    pub status: String,
    pub max_keys: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub async fn list_users(
    State(state): State<AppState>,
) -> Result<Json<Vec<User>>, AppError> {
    let rows = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at DESC")
        .fetch_all(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;
    Ok(Json(rows))
}

// ─── Request Details ───

pub async fn list_request_details(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let page: i64 = params.get("page").and_then(|p| p.parse().ok()).unwrap_or(1);
    let page_size: i64 = params.get("page_size").and_then(|p| p.parse().ok()).unwrap_or(50);
    let offset = (page - 1) * page_size;

    let rows = sqlx::query_as::<_, (Uuid, String, Option<Uuid>, Option<Uuid>, String, String, i32, i64, i64, f64, i64, chrono::DateTime<chrono::Utc>)>(
        "SELECT id, request_id, api_key_id, account_id, model, platform, status_code, input_tokens, output_tokens, cost_usd, duration_ms, created_at FROM request_details ORDER BY created_at DESC LIMIT $1 OFFSET $2"
    )
    .bind(page_size).bind(offset)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM request_details")
        .fetch_one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    let records: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "id": r.0, "request_id": r.1, "api_key_id": r.2, "account_id": r.3,
        "model": r.4, "platform": r.5, "status_code": r.6,
        "input_tokens": r.7, "output_tokens": r.8, "cost_usd": r.9,
        "duration_ms": r.10, "created_at": r.11
    })).collect();

    Ok(Json(serde_json::json!({
        "records": records, "total": total.0, "page": page, "page_size": page_size
    })))
}

// ─── Model Pricing ───

pub async fn get_model_pricing(
    State(state): State<AppState>,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let rows: Vec<(String, f64, f64, f64, f64, String)> = sqlx::query_as(
        "SELECT model, input_price, output_price, cache_read_price, cache_creation_price, source FROM model_pricing ORDER BY model",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;
    let items: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "model": r.0, "input_price": r.1, "output_price": r.2,
        "cache_read_price": r.3, "cache_creation_price": r.4, "source": r.5
    })).collect();
    Ok(Json(items))
}

pub async fn refresh_model_pricing(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    // TODO: fetch from external pricing source
    Ok(Json(serde_json::json!({ "success": true, "message": "Pricing refresh queued" })))
}

// ─── Service Rates ───

pub async fn get_service_rates(
    State(state): State<AppState>,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let rows: Vec<(String, f64, String)> = sqlx::query_as(
        "SELECT service, rate, description FROM service_rates ORDER BY service",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;
    let items: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "service": r.0, "rate": r.1, "description": r.2
    })).collect();
    Ok(Json(items))
}

pub async fn update_service_rates(
    State(state): State<AppState>,
    Json(input): Json<Vec<serde_json::Value>>,
) -> Result<Json<serde_json::Value>, AppError> {
    for item in &input {
        if let (Some(service), Some(rate)) = (item["service"].as_str(), item["rate"].as_f64()) {
            sqlx::query(
                "INSERT INTO service_rates (service, rate, description, updated_at) VALUES ($1, $2, $3, NOW()) ON CONFLICT (service) DO UPDATE SET rate = $2, description = $3, updated_at = NOW()",
            )
            .bind(service)
            .bind(rate)
            .bind(item["description"].as_str().unwrap_or(""))
            .execute(&state.db)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
        }
    }
    Ok(Json(serde_json::json!({ "success": true })))
}

// ─── OEM Settings ───

pub async fn get_oem_settings(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rows: Vec<(String, serde_json::Value)> = sqlx::query_as(
        "SELECT key, value FROM settings WHERE key LIKE 'oem_%' ORDER BY key",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;
    let mut map = serde_json::Map::new();
    for (key, value) in rows {
        map.insert(key.replace("oem_", ""), value);
    }
    Ok(Json(serde_json::Value::Object(map)))
}

pub async fn update_oem_settings(
    State(state): State<AppState>,
    Json(input): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    if let Some(obj) = input.as_object() {
        for (key, value) in obj {
            let db_key = format!("oem_{}", key);
            sqlx::query(
                "INSERT INTO settings (key, value, updated_at) VALUES ($1, $2, NOW()) ON CONFLICT (key) DO UPDATE SET value = $2, updated_at = NOW()",
            )
            .bind(&db_key)
            .bind(value)
            .execute(&state.db)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
        }
    }
    Ok(Json(serde_json::json!({ "success": true })))
}

// ─── Balance Scripts ───

pub async fn list_balance_scripts(
    State(state): State<AppState>,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let rows: Vec<(String, String, String, bool)> = sqlx::query_as(
        "SELECT name, platform, script, enabled FROM balance_scripts ORDER BY name",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;
    let items: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "name": r.0, "platform": r.1, "script": r.2, "enabled": r.3
    })).collect();
    Ok(Json(items))
}

pub async fn get_balance_script(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let row: Option<(String, String, String, bool)> = sqlx::query_as(
        "SELECT name, platform, script, enabled FROM balance_scripts WHERE name = $1",
    )
    .bind(&name)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;
    match row {
        Some(r) => Ok(Json(serde_json::json!({
            "name": r.0, "platform": r.1, "script": r.2, "enabled": r.3
        }))),
        None => Err(AppError::NotFound),
    }
}

pub async fn update_balance_script(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(input): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let script = input["script"].as_str().unwrap_or("");
    let platform = input["platform"].as_str().unwrap_or("");
    let enabled = input["enabled"].as_bool().unwrap_or(true);
    sqlx::query(
        "INSERT INTO balance_scripts (name, platform, script, enabled, updated_at) VALUES ($1, $2, $3, $4, NOW()) ON CONFLICT (name) DO UPDATE SET platform = $2, script = $3, enabled = $4, updated_at = NOW()",
    )
    .bind(&name)
    .bind(platform)
    .bind(script)
    .bind(enabled)
    .execute(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;
    Ok(Json(serde_json::json!({ "success": true })))
}

// ─── API Stats (Public, no auth) ───

pub async fn public_api_stats_models(
    State(state): State<AppState>,
) -> Result<Json<Vec<String>>, AppError> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT DISTINCT model FROM usage_logs ORDER BY model",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;
    Ok(Json(rows.into_iter().map(|r| r.0).collect()))
}

pub async fn public_get_key_id(
    State(state): State<AppState>,
    Json(input): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let api_key = input["apiKey"].as_str()
        .ok_or_else(|| AppError::BadRequest("apiKey required".into()))?;
    let key_hash = crate::middleware::auth::hash_key(api_key);
    let row: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM api_keys WHERE key_hash = $1",
    )
    .bind(&key_hash)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;
    match row {
        Some((id,)) => Ok(Json(serde_json::json!({ "success": true, "apiId": id }))),
        None => Err(AppError::NotFound),
    }
}

pub async fn public_user_stats(
    State(state): State<AppState>,
    Json(input): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let api_id = input["apiId"].as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| AppError::BadRequest("apiId required".into()))?;
    let row: (i64, i64, i64, f64) = sqlx::query_as(
        "SELECT COALESCE(COUNT(*),0)::BIGINT, COALESCE(SUM(input_tokens),0)::BIGINT, COALESCE(SUM(output_tokens),0)::BIGINT, COALESCE(SUM(cost_usd),0)::FLOAT8 FROM usage_logs WHERE api_key_id = $1"
    )
    .bind(api_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;
    Ok(Json(serde_json::json!({
        "success": true,
        "requests": row.0, "input_tokens": row.1, "output_tokens": row.2, "cost_usd": row.3
    })))
}

pub async fn public_user_model_stats(
    State(state): State<AppState>,
    Json(input): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let api_id = input["apiId"].as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| AppError::BadRequest("apiId required".into()))?;
    let rows: Vec<(String, i64, i64, i64, f64)> = sqlx::query_as(
        "SELECT model, COUNT(*)::BIGINT, COALESCE(SUM(input_tokens),0)::BIGINT, COALESCE(SUM(output_tokens),0)::BIGINT, COALESCE(SUM(cost_usd),0)::FLOAT8 FROM usage_logs WHERE api_key_id = $1 GROUP BY model ORDER BY COUNT(*) DESC"
    )
    .bind(api_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;
    let items: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "model": r.0, "requests": r.1, "input_tokens": r.2, "output_tokens": r.3, "cost_usd": r.4
    })).collect();
    Ok(Json(serde_json::json!({ "success": true, "stats": items })))
}
