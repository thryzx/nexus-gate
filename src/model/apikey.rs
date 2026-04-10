use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// API key record stored in PostgreSQL. The raw key is never stored —
/// only the SHA-256 hash.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiKeyRecord {
    pub id: Uuid,
    pub key_hash: String,
    pub name: String,
    pub permissions: String,  // JSON array: ["claude","openai"] or "[]" for all
    pub daily_cost_limit: f64,
    pub total_cost_limit: f64,
    pub max_concurrency: i32,
    pub rate_limit_rpm: i32,
    pub restricted_models: String, // JSON array of allowed model names, empty = all
    pub status: String,            // "active" | "disabled"
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl ApiKeyRecord {
    /// Check if this key has permission for the given service.
    /// Empty permissions = all services allowed (CRS parity).
    pub fn has_permission(&self, service: &str) -> bool {
        let perms: Vec<String> = serde_json::from_str(&self.permissions).unwrap_or_default();
        perms.is_empty() || perms.iter().any(|p| p == service || p == "all")
    }

    /// Check if a specific model is allowed by this key.
    /// Empty restricted_models = all models allowed.
    pub fn is_model_allowed(&self, model: &str) -> bool {
        let models: Vec<String> = serde_json::from_str(&self.restricted_models).unwrap_or_default();
        models.is_empty() || models.iter().any(|m| model.contains(m.as_str()))
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateApiKeyInput {
    pub name: String,
    pub permissions: Vec<String>,
    pub restricted_models: Option<Vec<String>>,
    pub daily_cost_limit: Option<f64>,
    pub total_cost_limit: Option<f64>,
    pub max_concurrency: Option<i32>,
    pub rate_limit_rpm: Option<i32>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_key(permissions: &str, restricted_models: &str) -> ApiKeyRecord {
        ApiKeyRecord {
            id: Uuid::new_v4(),
            key_hash: String::new(),
            name: "test".into(),
            permissions: permissions.into(),
            daily_cost_limit: 0.0,
            total_cost_limit: 0.0,
            max_concurrency: 0,
            rate_limit_rpm: 0,
            restricted_models: restricted_models.into(),
            status: "active".into(),
            expires_at: None,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn create_key_input_minimal() {
        let json = r#"{"name":"test-key","permissions":["claude"]}"#;
        let input: CreateApiKeyInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.name, "test-key");
    }

    #[test]
    fn empty_permissions_allows_all() {
        let key = make_key("[]", "[]");
        assert!(key.has_permission("claude"));
        assert!(key.has_permission("openai"));
    }

    #[test]
    fn specific_permissions() {
        let key = make_key(r#"["claude","gemini"]"#, "[]");
        assert!(key.has_permission("claude"));
        assert!(key.has_permission("gemini"));
        assert!(!key.has_permission("openai"));
    }

    #[test]
    fn all_permission() {
        let key = make_key(r#"["all"]"#, "[]");
        assert!(key.has_permission("openai"));
    }

    #[test]
    fn empty_models_allows_all() {
        let key = make_key("[]", "[]");
        assert!(key.is_model_allowed("claude-opus-4"));
    }

    #[test]
    fn restricted_models() {
        let key = make_key("[]", r#"["claude-sonnet","claude-haiku"]"#);
        assert!(key.is_model_allowed("claude-sonnet-4-20250514"));
        assert!(!key.is_model_allowed("claude-opus-4"));
    }
}
