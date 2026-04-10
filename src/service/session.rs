use crate::error::AppError;
use crate::model::account::AccountRecord;
use crate::state::AppState;
use redis::AsyncCommands;
use sha2::{Digest, Sha256};

/// Compute a deterministic hash from the request body for sticky session binding.
///
/// Priority order (matches CRS logic):
/// 1. `metadata.user_id` — if contains a 32-char hex session ID, reuse directly.
/// 2. Ephemeral cache_control blocks — hash of all text with `cache_control.type == "ephemeral"`.
/// 3. System prompt — all text from `system` (string or array of objects).
/// 4. First user message content.
/// 5. Fallback — hash entire body.
///
/// Returns a 32-character hex string (first half of SHA-256).
pub fn compute_hash(body: &[u8]) -> String {
    let parsed: Option<serde_json::Value> = serde_json::from_slice(body).ok();

    let Some(val) = parsed else {
        return sha256_truncated(&String::from_utf8_lossy(body));
    };

    // Priority 1: metadata.user_id containing a session ID.
    if let Some(user_id) = val["metadata"]["user_id"].as_str() {
        if let Some(session_id) = extract_session_id(user_id) {
            return session_id;
        }
    }

    // Priority 2: Ephemeral cache_control blocks.
    let ephemeral = collect_ephemeral_text(&val);
    if !ephemeral.is_empty() {
        return sha256_truncated(&ephemeral);
    }

    // Priority 3: System prompt.
    let system_text = extract_system_text(&val);
    if !system_text.is_empty() {
        return sha256_truncated(&system_text);
    }

    // Priority 4: First user message.
    let first_msg = extract_first_message_text(&val);
    if !first_msg.is_empty() {
        return sha256_truncated(&first_msg);
    }

    // Fallback: hash entire body.
    sha256_truncated(&String::from_utf8_lossy(body))
}

/// SHA-256 hex, truncated to 32 chars (matches CRS .substring(0, 32)).
fn sha256_truncated(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let full = format!("{:x}", hasher.finalize());
    full[..32].to_string()
}

/// SHA-256 full 64-char hex (for non-session uses).
pub fn sha256_full(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Extract a 32-char hex session ID from user_id if present.
/// Looks for a contiguous 32+ hex char substring.
fn extract_session_id(user_id: &str) -> Option<String> {
    // Find the longest contiguous hex run.
    let mut best_start = 0;
    let mut best_len = 0;
    let mut cur_start = 0;
    let mut cur_len = 0;
    for (i, c) in user_id.chars().enumerate() {
        if c.is_ascii_hexdigit() {
            if cur_len == 0 {
                cur_start = i;
            }
            cur_len += 1;
        } else {
            if cur_len > best_len {
                best_start = cur_start;
                best_len = cur_len;
            }
            cur_len = 0;
        }
    }
    if cur_len > best_len {
        best_start = cur_start;
        best_len = cur_len;
    }
    if best_len >= 32 {
        Some(user_id[best_start..best_start + 32].to_lowercase())
    } else {
        None
    }
}

/// Collect text from all ephemeral cache_control blocks in system + messages.
fn collect_ephemeral_text(val: &serde_json::Value) -> String {
    let mut text = String::new();

    // System array blocks with cache_control.
    if let Some(system_arr) = val["system"].as_array() {
        for part in system_arr {
            if is_ephemeral(part) {
                if let Some(t) = part["text"].as_str() {
                    text.push_str(t);
                }
            }
        }
    }

    // Messages with ephemeral content.
    if let Some(messages) = val["messages"].as_array() {
        for msg in messages {
            if is_ephemeral(msg) {
                append_content_text(msg, &mut text);
            }
            // Also check individual content blocks within a message.
            if let Some(content_arr) = msg["content"].as_array() {
                for block in content_arr {
                    if is_ephemeral(block) {
                        if let Some(t) = block["text"].as_str() {
                            text.push_str(t);
                        }
                    }
                }
            }
        }
    }

    text
}

fn is_ephemeral(val: &serde_json::Value) -> bool {
    val["cache_control"]["type"].as_str() == Some("ephemeral")
}

/// Extract all text from `system` — handles both string and array-of-objects format.
fn extract_system_text(val: &serde_json::Value) -> String {
    match &val["system"] {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(arr) => arr
            .iter()
            .filter_map(|p| p["text"].as_str())
            .collect::<Vec<_>>()
            .join(""),
        _ => String::new(),
    }
}

/// Extract text from the first message's content.
fn extract_first_message_text(val: &serde_json::Value) -> String {
    let first = val["messages"].as_array().and_then(|m| m.first());
    let Some(msg) = first else {
        return String::new();
    };
    let mut text = String::new();
    append_content_text(msg, &mut text);
    text
}

/// Append text content from a message (handles string or array-of-objects content).
fn append_content_text(msg: &serde_json::Value, out: &mut String) {
    match &msg["content"] {
        serde_json::Value::String(s) => out.push_str(s),
        serde_json::Value::Array(arr) => {
            for block in arr {
                if let Some(t) = block["text"].as_str() {
                    out.push_str(t);
                }
            }
        }
        _ => {}
    }
}

/// Look up an existing sticky session binding in Redis.
/// Auto-renews TTL when remaining time < renewal threshold (matches CRS logic).
pub async fn get_sticky(
    state: &AppState,
    session_hash: &str,
) -> Result<Option<AccountRecord>, AppError> {
    let mut conn = state
        .redis_conn()
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    let key = format!("sess:{session_hash}");
    let account_id: Option<String> = conn
        .get(&key)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    let Some(account_id) = account_id else {
        return Ok(None);
    };

    // Auto-renew TTL if near expiry (CRS renews when remaining < 14 days up to 15 days).
    let ttl: i64 = conn
        .ttl(&key)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    let max_ttl = state.config.scheduler.sticky_session_ttl_secs as i64;
    let renewal_threshold = max_ttl - 86400; // renew when within 1 day of max_ttl
    if ttl > 0 && ttl < renewal_threshold {
        let _: () = conn
            .expire(&key, max_ttl)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
    }

    let id: uuid::Uuid = account_id
        .parse()
        .map_err(|_| AppError::Internal(anyhow::anyhow!("invalid uuid in session cache")))?;

    let record = sqlx::query_as::<_, AccountRecord>(
        "SELECT * FROM accounts WHERE id = $1 AND status = 'active'",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    Ok(record)
}

/// Bind a session hash to a specific account ID with TTL.
pub async fn set_sticky(
    state: &AppState,
    session_hash: &str,
    account_id: &str,
    ttl_secs: u64,
) -> Result<(), AppError> {
    let mut conn = state
        .redis_conn()
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    let key = format!("sess:{session_hash}");
    let _: () = conn
        .set_ex(&key, account_id, ttl_secs)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_deterministic() {
        let body = br#"{"system":"You are helpful","messages":[{"role":"user","content":"hello"}]}"#;
        let a = compute_hash(body);
        let b = compute_hash(body);
        assert_eq!(a, b);
    }

    #[test]
    fn hash_is_32_chars() {
        // CRS returns 32-char hex (SHA256 truncated), not 64.
        let body = br#"{"system":"test","messages":[]}"#;
        assert_eq!(compute_hash(body).len(), 32);
    }

    #[test]
    fn hash_varies_by_system_prompt() {
        let body_a = br#"{"system":"A","messages":[{"role":"user","content":"x"}]}"#;
        let body_b = br#"{"system":"B","messages":[{"role":"user","content":"x"}]}"#;
        assert_ne!(compute_hash(body_a), compute_hash(body_b));
    }

    #[test]
    fn hash_varies_by_first_message() {
        // No system prompt so priority 4 (first message) fires.
        let body_a = br#"{"messages":[{"role":"user","content":"hello"}]}"#;
        let body_b = br#"{"messages":[{"role":"user","content":"bye"}]}"#;
        assert_ne!(compute_hash(body_a), compute_hash(body_b));
    }

    #[test]
    fn hash_handles_non_json() {
        let body = b"not json at all";
        let h = compute_hash(body);
        assert_eq!(h.len(), 32);
    }

    #[test]
    fn priority_1_metadata_user_id() {
        // If metadata.user_id contains a 32-char hex, reuse it directly.
        let body = br#"{"metadata":{"user_id":"aabbccdd11223344aabbccdd11223344-extra"},"system":"X","messages":[]}"#;
        let h = compute_hash(body);
        assert_eq!(h, "aabbccdd11223344aabbccdd11223344");
    }

    #[test]
    fn priority_2_ephemeral_cache_blocks() {
        // Ephemeral cache_control text takes priority over system prompt.
        let body = br#"{
            "system":[
                {"type":"text","text":"normal"},
                {"type":"text","text":"cached","cache_control":{"type":"ephemeral"}}
            ],
            "messages":[]
        }"#;
        let h = compute_hash(body);
        // Should hash "cached" only, not "normal".
        assert_eq!(h, sha256_truncated("cached"));
    }

    #[test]
    fn priority_3_system_string() {
        let body = br#"{"system":"my system prompt","messages":[]}"#;
        let h = compute_hash(body);
        assert_eq!(h, sha256_truncated("my system prompt"));
    }

    #[test]
    fn priority_3_system_array() {
        // System as array of text objects.
        let body = br#"{"system":[{"type":"text","text":"part1"},{"type":"text","text":"part2"}],"messages":[]}"#;
        let h = compute_hash(body);
        assert_eq!(h, sha256_truncated("part1part2"));
    }

    #[test]
    fn priority_4_first_message_string_content() {
        let body = br#"{"messages":[{"role":"user","content":"hello world"}]}"#;
        let h = compute_hash(body);
        assert_eq!(h, sha256_truncated("hello world"));
    }

    #[test]
    fn priority_4_first_message_array_content() {
        let body = br#"{"messages":[{"role":"user","content":[{"type":"text","text":"block1"},{"type":"text","text":"block2"}]}]}"#;
        let h = compute_hash(body);
        assert_eq!(h, sha256_truncated("block1block2"));
    }

    #[test]
    fn ephemeral_in_message_content() {
        let body = br#"{
            "system":"ignored when ephemeral exists",
            "messages":[{
                "role":"user",
                "content":[
                    {"type":"text","text":"normal block"},
                    {"type":"text","text":"ephemeral block","cache_control":{"type":"ephemeral"}}
                ]
            }]
        }"#;
        let h = compute_hash(body);
        assert_eq!(h, sha256_truncated("ephemeral block"));
    }

    #[test]
    fn extract_session_id_from_user_id() {
        assert_eq!(
            extract_session_id("prefix-aabbccdd11223344aabbccdd11223344"),
            Some("aabbccdd11223344aabbccdd11223344".to_string())
        );
    }

    #[test]
    fn extract_session_id_too_short() {
        assert_eq!(extract_session_id("short-abc123"), None);
    }
}
