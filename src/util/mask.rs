/// Mask sensitive tokens for safe logging.
/// Shows the first `visible` characters and replaces the rest with `***`.
pub fn mask_token(token: &str, visible: usize) -> String {
    if token.len() <= visible {
        return "***".to_string();
    }
    format!("{}***", &token[..visible])
}

/// Mask an API key for logging — show prefix + first 4 chars.
pub fn mask_api_key(key: &str) -> String {
    if key.len() <= 7 {
        return "***".to_string();
    }
    format!("{}***", &key[..7])
}

/// Remove all sensitive values from a JSON object for logging.
pub fn sanitize_json_for_log(value: &serde_json::Value) -> serde_json::Value {
    let sensitive_keys = [
        "access_token",
        "refresh_token",
        "api_key",
        "apiKey",
        "password",
        "secret",
        "credentials",
        "authorization",
        "token",
    ];

    match value {
        serde_json::Value::Object(map) => {
            let mut clean = serde_json::Map::new();
            for (k, v) in map {
                let normalized = k.to_lowercase().replace('-', "_");
                if sensitive_keys.iter().any(|s| normalized.contains(s)) {
                    clean.insert(k.clone(), serde_json::Value::String("[REDACTED]".into()));
                } else {
                    clean.insert(k.clone(), sanitize_json_for_log(v));
                }
            }
            serde_json::Value::Object(clean)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(sanitize_json_for_log).collect())
        }
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn mask_token_basic() {
        assert_eq!(mask_token("sk-1234567890abcdef", 6), "sk-123***");
    }

    #[test]
    fn mask_short_token() {
        assert_eq!(mask_token("ab", 6), "***");
    }

    #[test]
    fn mask_api_key_format() {
        assert_eq!(mask_api_key("nk-0123456789abcdef"), "nk-0123***");
    }

    #[test]
    fn mask_api_key_short() {
        assert_eq!(mask_api_key("short"), "***");
    }

    #[test]
    fn sanitize_redacts_tokens() {
        let input = json!({
            "name": "test",
            "access_token": "secret-value",
            "nested": {
                "password": "hunter2",
                "safe": "visible"
            }
        });

        let sanitized = sanitize_json_for_log(&input);
        assert_eq!(sanitized["access_token"], "[REDACTED]");
        assert_eq!(sanitized["nested"]["password"], "[REDACTED]");
        assert_eq!(sanitized["nested"]["safe"], "visible");
        assert_eq!(sanitized["name"], "test");
    }

    #[test]
    fn sanitize_preserves_non_sensitive() {
        let input = json!({"model": "claude-sonnet", "messages": []});
        let sanitized = sanitize_json_for_log(&input);
        assert_eq!(sanitized, input);
    }
}
