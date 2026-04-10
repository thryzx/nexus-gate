use crate::config::AppConfig;
use sqlx::PgPool;
use std::sync::Arc;

/// Shared application state injected into every handler via Axum's `State`.
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: redis::Client,
    pub config: Arc<AppConfig>,
}

impl AppState {
    pub async fn new(config: AppConfig) -> anyhow::Result<Self> {
        let db = PgPool::connect_lazy(&config.database.url)?;

        let redis = redis::Client::open(config.redis.url.as_str())?;

        Ok(Self {
            db,
            redis,
            config: Arc::new(config),
        })
    }

    /// Obtain an async Redis connection from the pool.
    pub async fn redis_conn(&self) -> anyhow::Result<redis::aio::MultiplexedConnection> {
        Ok(self.redis.get_multiplexed_async_connection().await?)
    }
}
