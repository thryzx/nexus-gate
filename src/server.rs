use crate::api;
use crate::config::AppConfig;
use crate::error::AppError;
use crate::middleware::{admin_auth, auth, rate_limit, request_id};
use crate::state::AppState;
use anyhow::Result;
use axum::{
    middleware as axum_mw,
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

/// Build the full Axum router with all routes and middleware.
pub fn build_router(state: AppState) -> Router {
    // ── Public health route (no auth) ──
    let health = Router::new().route("/health", get(api::health::check));

    // ── Admin login (no JWT required) ──
    let admin_public = Router::new()
        .route("/admin/login", post(api::admin::login));

    // ── Claude-compatible endpoints ──
    let claude = Router::new()
        .route("/v1/messages", post(api::claude::messages))
        .route("/v1/messages/count_tokens", post(api::claude::count_tokens));

    // ── OpenAI-compatible endpoints ──
    let openai = Router::new()
        .route("/v1/chat/completions", post(api::openai::chat_completions))
        .route("/v1/responses", post(api::openai::responses));

    // ── Gemini-compatible endpoints ──
    let gemini = Router::new()
        .route(
            "/v1beta/models/:model_action",
            post(api::gemini::model_action),
        )
        .route("/v1beta/models", get(api::gemini::list_models));

    // ── Admin endpoints (JWT auth) ──
    let admin = Router::new()
        // Dashboard
        .route("/admin/dashboard", get(api::admin::dashboard))
        .route("/admin/change-password", post(api::admin::change_password))
        // Accounts (generic)
        .route("/admin/accounts", get(api::admin::list_accounts))
        .route("/admin/accounts", post(api::admin::create_account))
        .route("/admin/accounts/:id", get(api::admin::get_account))
        .route("/admin/accounts/:id", axum::routing::put(api::admin::update_account))
        .route("/admin/accounts/:id", axum::routing::delete(api::admin::delete_account))
        // Account usage
        .route("/admin/accounts/:id/usage-records", get(api::admin_accounts::get_account_usage_records))
        .route("/admin/accounts/usage-stats", get(api::admin_accounts::get_accounts_usage_stats))
        // Claude accounts
        .route("/admin/claude-accounts", get(api::admin_accounts::list_claude_accounts))
        .route("/admin/claude-accounts", post(api::admin_accounts::create_claude_account))
        .route("/admin/claude-accounts/:id", axum::routing::put(api::admin_accounts::update_claude_account))
        .route("/admin/claude-accounts/:id", axum::routing::delete(api::admin_accounts::delete_claude_account))
        .route("/admin/claude-accounts/:id/toggle", axum::routing::put(api::admin_accounts::toggle_claude_account))
        .route("/admin/claude-accounts/:id/toggle-schedulable", axum::routing::put(api::admin_accounts::toggle_claude_schedulable))
        .route("/admin/claude-accounts/:id/reset-status", post(api::admin_accounts::reset_claude_status))
        .route("/admin/claude-accounts/:id/test", post(api::admin_accounts::test_claude_account))
        .route("/admin/claude-accounts/:id/refresh", post(api::admin_accounts::refresh_claude_token))
        .route("/admin/claude-accounts/generate-auth-url", post(api::admin_accounts::generate_claude_auth_url))
        .route("/admin/claude-accounts/exchange-code", post(api::admin_accounts::exchange_claude_code))
        .route("/admin/claude-accounts/oauth-with-cookie", post(api::admin_accounts::claude_oauth_with_cookie))
        .route("/admin/claude-accounts/generate-setup-token-url", post(api::admin_accounts::generate_claude_setup_token_url))
        .route("/admin/claude-accounts/exchange-setup-token-code", post(api::admin_accounts::exchange_claude_setup_token))
        // Claude Console accounts
        .route("/admin/claude-console-accounts", get(api::admin_accounts::list_claude_console_accounts))
        .route("/admin/claude-console-accounts", post(api::admin_accounts::create_claude_console_account))
        .route("/admin/claude-console-accounts/:id", axum::routing::put(api::admin_accounts::update_claude_console_account))
        .route("/admin/claude-console-accounts/:id", axum::routing::delete(api::admin_accounts::delete_claude_console_account))
        .route("/admin/claude-console-accounts/:id/toggle", axum::routing::put(api::admin_accounts::toggle_claude_console))
        .route("/admin/claude-console-accounts/:id/toggle-schedulable", axum::routing::put(api::admin_accounts::toggle_claude_console_schedulable))
        // Bedrock accounts
        .route("/admin/bedrock-accounts", get(api::admin_accounts::list_bedrock_accounts))
        .route("/admin/bedrock-accounts", post(api::admin_accounts::create_bedrock_account))
        .route("/admin/bedrock-accounts/:id", axum::routing::put(api::admin_accounts::update_bedrock_account))
        .route("/admin/bedrock-accounts/:id", axum::routing::delete(api::admin_accounts::delete_bedrock_account))
        .route("/admin/bedrock-accounts/:id/toggle", axum::routing::put(api::admin_accounts::toggle_bedrock))
        .route("/admin/bedrock-accounts/:id/toggle-schedulable", axum::routing::put(api::admin_accounts::toggle_bedrock_schedulable))
        .route("/admin/bedrock-accounts/:id/reset-status", post(api::admin_accounts::reset_bedrock_status))
        .route("/admin/bedrock-accounts/:id/test", post(api::admin_accounts::test_bedrock_account))
        // Gemini accounts (OAuth)
        .route("/admin/gemini-accounts", get(api::admin_accounts::list_gemini_accounts))
        .route("/admin/gemini-accounts", post(api::admin_accounts::create_gemini_account))
        .route("/admin/gemini-accounts/:id", axum::routing::put(api::admin_accounts::update_gemini_account))
        .route("/admin/gemini-accounts/:id", axum::routing::delete(api::admin_accounts::delete_gemini_account))
        .route("/admin/gemini-accounts/:id/toggle-schedulable", axum::routing::put(api::admin_accounts::toggle_gemini_schedulable))
        .route("/admin/gemini-accounts/:id/reset-status", post(api::admin_accounts::reset_gemini_status))
        .route("/admin/gemini-accounts/:id/test", post(api::admin_accounts::test_gemini_account))
        .route("/admin/gemini-accounts/generate-auth-url", post(api::admin_accounts::generate_gemini_auth_url))
        .route("/admin/gemini-accounts/exchange-code", post(api::admin_accounts::exchange_gemini_code))
        // Gemini API accounts
        .route("/admin/gemini-api-accounts", get(api::admin_accounts::list_gemini_api_accounts))
        .route("/admin/gemini-api-accounts", post(api::admin_accounts::create_gemini_api_account))
        .route("/admin/gemini-api-accounts/:id", axum::routing::put(api::admin_accounts::update_gemini_api_account))
        .route("/admin/gemini-api-accounts/:id", axum::routing::delete(api::admin_accounts::delete_gemini_api_account))
        .route("/admin/gemini-api-accounts/:id/toggle", axum::routing::put(api::admin_accounts::toggle_gemini_api))
        .route("/admin/gemini-api-accounts/:id/toggle-schedulable", axum::routing::put(api::admin_accounts::toggle_gemini_api_schedulable))
        // OpenAI accounts (OAuth)
        .route("/admin/openai-accounts", get(api::admin_accounts::list_openai_accounts))
        .route("/admin/openai-accounts", post(api::admin_accounts::create_openai_account))
        .route("/admin/openai-accounts/:id", axum::routing::put(api::admin_accounts::update_openai_account))
        .route("/admin/openai-accounts/:id", axum::routing::delete(api::admin_accounts::delete_openai_account))
        .route("/admin/openai-accounts/:id/toggle", axum::routing::put(api::admin_accounts::toggle_openai))
        .route("/admin/openai-accounts/:id/toggle-schedulable", axum::routing::put(api::admin_accounts::toggle_openai_schedulable))
        .route("/admin/openai-accounts/:id/reset-status", post(api::admin_accounts::reset_openai_status))
        .route("/admin/openai-accounts/generate-auth-url", post(api::admin_accounts::generate_openai_auth_url))
        .route("/admin/openai-accounts/exchange-code", post(api::admin_accounts::exchange_openai_code))
        // OpenAI Responses accounts
        .route("/admin/openai-responses-accounts", get(api::admin_accounts::list_openai_responses_accounts))
        .route("/admin/openai-responses-accounts", post(api::admin_accounts::create_openai_responses_account))
        .route("/admin/openai-responses-accounts/:id", axum::routing::put(api::admin_accounts::update_openai_responses_account))
        .route("/admin/openai-responses-accounts/:id", axum::routing::delete(api::admin_accounts::delete_openai_responses_account))
        .route("/admin/openai-responses-accounts/:id/toggle", axum::routing::put(api::admin_accounts::toggle_openai_responses))
        .route("/admin/openai-responses-accounts/:id/toggle-schedulable", axum::routing::put(api::admin_accounts::toggle_openai_responses_schedulable))
        .route("/admin/openai-responses-accounts/:id/reset-status", post(api::admin_accounts::reset_openai_responses_status))
        .route("/admin/openai-responses-accounts/:id/test", post(api::admin_accounts::test_openai_responses_account))
        // Azure OpenAI accounts
        .route("/admin/azure-openai-accounts", get(api::admin_accounts::list_azure_openai_accounts))
        .route("/admin/azure-openai-accounts", post(api::admin_accounts::create_azure_openai_account))
        .route("/admin/azure-openai-accounts/:id", axum::routing::put(api::admin_accounts::update_azure_openai_account))
        .route("/admin/azure-openai-accounts/:id", axum::routing::delete(api::admin_accounts::delete_azure_openai_account))
        .route("/admin/azure-openai-accounts/:id/toggle", axum::routing::put(api::admin_accounts::toggle_azure_openai))
        .route("/admin/azure-openai-accounts/:id/toggle-schedulable", axum::routing::put(api::admin_accounts::toggle_azure_openai_schedulable))
        .route("/admin/azure-openai-accounts/:id/test", post(api::admin_accounts::test_azure_openai_account))
        // Droid accounts
        .route("/admin/droid-accounts", get(api::admin_accounts::list_droid_accounts))
        .route("/admin/droid-accounts", post(api::admin_accounts::create_droid_account))
        .route("/admin/droid-accounts/:id", get(api::admin_accounts::get_droid_account))
        .route("/admin/droid-accounts/:id", axum::routing::put(api::admin_accounts::update_droid_account))
        .route("/admin/droid-accounts/:id", axum::routing::delete(api::admin_accounts::delete_droid_account))
        .route("/admin/droid-accounts/:id/toggle-schedulable", axum::routing::put(api::admin_accounts::toggle_droid_schedulable))
        .route("/admin/droid-accounts/:id/reset-status", post(api::admin_accounts::reset_droid_status))
        .route("/admin/droid-accounts/:id/test", post(api::admin_accounts::test_droid_account))
        .route("/admin/droid-accounts/generate-auth-url", post(api::admin_accounts::generate_droid_auth_url))
        .route("/admin/droid-accounts/exchange-code", post(api::admin_accounts::exchange_droid_code))
        // CCR accounts
        .route("/admin/ccr-accounts", get(api::admin_accounts::list_ccr_accounts))
        .route("/admin/ccr-accounts", post(api::admin_accounts::create_ccr_account))
        .route("/admin/ccr-accounts/:id", axum::routing::put(api::admin_accounts::update_ccr_account))
        .route("/admin/ccr-accounts/:id", axum::routing::delete(api::admin_accounts::delete_ccr_account))
        .route("/admin/ccr-accounts/:id/toggle", axum::routing::put(api::admin_accounts::toggle_ccr))
        .route("/admin/ccr-accounts/:id/toggle-schedulable", axum::routing::put(api::admin_accounts::toggle_ccr_schedulable))
        // Account Groups
        .route("/admin/account-groups", get(api::admin_accounts::list_account_groups))
        .route("/admin/account-groups", post(api::admin_accounts::create_account_group))
        .route("/admin/account-groups/:id", axum::routing::put(api::admin_accounts::update_account_group))
        .route("/admin/account-groups/:id", axum::routing::delete(api::admin_accounts::delete_account_group))
        .route("/admin/account-groups/:id/members", get(api::admin_accounts::get_account_group_members))
        // Quota Cards
        .route("/admin/quota-cards", get(api::admin_accounts::list_quota_cards))
        .route("/admin/quota-cards", post(api::admin_accounts::create_quota_card))
        .route("/admin/quota-cards/:id", axum::routing::delete(api::admin_accounts::delete_quota_card))
        .route("/admin/quota-cards/stats", get(api::admin_accounts::get_quota_card_stats))
        // Webhook
        .route("/admin/webhook/config", get(api::admin_accounts::get_webhook_config))
        .route("/admin/webhook/config", post(api::admin_accounts::update_webhook_config))
        .route("/admin/webhook/platforms", post(api::admin_accounts::create_webhook_platform))
        .route("/admin/webhook/platforms/:id", axum::routing::delete(api::admin_accounts::delete_webhook_platform))
        .route("/admin/webhook/test", post(api::admin_accounts::test_webhook))
        // User Management
        .route("/admin/users", get(api::admin_accounts::list_users))
        // Request Details
        .route("/admin/request-details", get(api::admin_accounts::list_request_details))
        // Model Pricing
        .route("/admin/models/pricing", get(api::admin_accounts::get_model_pricing))
        .route("/admin/models/pricing/refresh", post(api::admin_accounts::refresh_model_pricing))
        // Service Rates
        .route("/admin/service-rates", get(api::admin_accounts::get_service_rates))
        .route("/admin/service-rates", axum::routing::put(api::admin_accounts::update_service_rates))
        // OEM Settings
        .route("/admin/oem-settings", get(api::admin_accounts::get_oem_settings))
        .route("/admin/oem-settings", axum::routing::put(api::admin_accounts::update_oem_settings))
        // Balance Scripts
        .route("/admin/balance-scripts", get(api::admin_accounts::list_balance_scripts))
        .route("/admin/balance-scripts/:name", get(api::admin_accounts::get_balance_script))
        .route("/admin/balance-scripts/:name", axum::routing::put(api::admin_accounts::update_balance_script))
        // API Keys
        .route("/admin/keys", get(api::admin::list_keys))
        .route("/admin/keys", post(api::admin::create_key))
        .route("/admin/keys/:id", get(api::admin::get_key))
        .route("/admin/keys/:id", axum::routing::put(api::admin::update_key))
        .route("/admin/keys/:id", axum::routing::delete(api::admin::delete_key))
        // Usage
        .route("/admin/usage/records", get(api::admin::usage_records))
        .route("/admin/usage/trends", get(api::admin::usage_trends))
        .route("/admin/usage/models", get(api::admin::usage_by_model))
        // Fingerprints
        .route("/admin/fingerprints", get(api::admin::list_fingerprint_profiles))
        .route("/admin/fingerprints", post(api::admin::create_fingerprint_profile))
        .route("/admin/fingerprints/:id", get(api::admin::get_fingerprint_profile))
        .route("/admin/fingerprints/:id", axum::routing::put(api::admin::update_fingerprint_profile))
        .route("/admin/fingerprints/:id", axum::routing::delete(api::admin::delete_fingerprint_profile))
        // Settings
        .route("/admin/settings", get(api::admin::get_settings))
        .route("/admin/settings", post(api::admin::update_setting))
        // Stats (legacy)
        .route("/admin/stats", get(api::admin::stats))
        .layer(axum_mw::from_fn_with_state(
            state.clone(),
            admin_auth::require_admin_jwt,
        ));

    // ── Public API Stats endpoints (no auth) ──
    let api_stats = Router::new()
        .route("/apiStats/models", get(api::admin_accounts::public_api_stats_models))
        .route("/apiStats/api/get-key-id", post(api::admin_accounts::public_get_key_id))
        .route("/apiStats/api/user-stats", post(api::admin_accounts::public_user_stats))
        .route("/apiStats/api/user-model-stats", post(api::admin_accounts::public_user_model_stats))
        .route("/apiStats/service-rates", get(api::admin_accounts::get_service_rates));

    // ── Compose everything ──
    let api_routes = Router::new()
        .merge(claude)
        .merge(openai)
        .merge(gemini)
        .layer(axum_mw::from_fn_with_state(
            state.clone(),
            auth::require_api_key,
        ))
        .layer(axum_mw::from_fn_with_state(
            state.clone(),
            rate_limit::check,
        ));

    // ── Static file serving for admin SPA ──
    let spa_dir = std::env::var("SPA_DIR").unwrap_or_else(|_| "web/admin-spa/dist".into());
    let index_file = format!("{}/index.html", spa_dir);

    let static_files = ServeDir::new(&spa_dir)
        .not_found_service(ServeFile::new(&index_file));

    Router::new()
        .merge(health)
        .merge(admin_public)
        .merge(api_stats)
        .merge(api_routes)
        .merge(admin)
        .nest_service("/admin-ui", static_files)
        .fallback(|| async { AppError::NotFound })
        .layer(axum_mw::from_fn(request_id::inject))
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new().gzip(true))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// Start the server.
pub async fn run(config: AppConfig) -> Result<()> {
    let addr = SocketAddr::new(config.server.host.parse()?, config.server.port);

    let state = AppState::new(config).await?;

    // Run database migrations
    sqlx::migrate!("./migrations").run(&state.db).await?;
    tracing::info!("database migrations applied");

    let app = build_router(state);

    tracing::info!(%addr, "listening");
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
