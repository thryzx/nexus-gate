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
        // Accounts
        .route("/admin/accounts", get(api::admin::list_accounts))
        .route("/admin/accounts", post(api::admin::create_account))
        .route("/admin/accounts/:id", get(api::admin::get_account))
        .route("/admin/accounts/:id", axum::routing::put(api::admin::update_account))
        .route("/admin/accounts/:id", axum::routing::delete(api::admin::delete_account))
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
