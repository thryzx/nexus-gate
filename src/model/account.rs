use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Upstream AI account record stored in PostgreSQL.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AccountRecord {
    pub id: Uuid,
    pub name: String,
    pub platform: String,       // "claude" | "openai" | "gemini" | "bedrock" | "azure"
    pub account_type: String,   // "oauth" | "apikey" | "bedrock"
    pub credentials_enc: String, // AES-encrypted JSON
    pub status: String,         // "active" | "unavailable" | "disabled"
    pub priority: i32,
    pub max_concurrency: i32,
    pub proxy_url: Option<String>,
    pub fingerprint_profile_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAccountInput {
    pub name: String,
    pub platform: String,
    pub account_type: String,
    pub credentials: String, // plaintext JSON, will be encrypted before storage
    pub priority: Option<i32>,
    pub max_concurrency: Option<i32>,
    pub proxy_url: Option<String>,
    pub fingerprint_profile_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAccountInput {
    pub name: Option<String>,
    pub status: Option<String>,
    pub priority: Option<i32>,
    pub max_concurrency: Option<i32>,
    pub proxy_url: Option<String>,
    pub fingerprint_profile_id: Option<Uuid>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_input_deserialize() {
        let json = r#"{"name":"test","platform":"claude","account_type":"oauth","credentials":"{}"}"#;
        let input: CreateAccountInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.name, "test");
        assert_eq!(input.platform, "claude");
    }
}
