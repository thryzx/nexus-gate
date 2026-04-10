use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Usage log entry — one row per completed API request.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UsageRecord {
    pub id: Uuid,
    pub api_key_id: Uuid,
    pub account_id: Uuid,
    pub model: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cost_usd: f64,
    pub created_at: DateTime<Utc>,
}
