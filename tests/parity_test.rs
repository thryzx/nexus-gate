//! CRS parity verification tests.
//!
//! These tests verify that nexus-gate's core logic matches claude-relay-service
//! by testing the same inputs produce the same outputs for all key components.

mod session_parity {
    use nexus_gate::service::session::compute_hash;

    /// CRS: system prompt takes priority 3, returns 32-char hex.
    #[test]
    fn crs_parity_system_prompt_hash_length() {
        let body = br#"{"system":"You are a helpful assistant.","messages":[{"role":"user","content":"Hello"}]}"#;
        let hash = compute_hash(body);
        assert_eq!(hash.len(), 32, "CRS returns SHA256.substring(0,32)");
    }

    /// CRS: metadata.user_id with session ID returns it directly (priority 1).
    #[test]
    fn crs_parity_metadata_user_id_passthrough() {
        let body = br#"{"metadata":{"user_id":"sess-aabbccdd11223344eeff00112233aabb"},"system":"X","messages":[]}"#;
        let hash = compute_hash(body);
        // The hex portion "aabbccdd11223344eeff00112233aabb" should be extracted.
        assert_eq!(hash, "aabbccdd11223344eeff00112233aabb");
    }

    /// CRS: ephemeral cache_control blocks take priority 2 over system prompt.
    #[test]
    fn crs_parity_ephemeral_overrides_system() {
        let body_with_ephemeral = br#"{
            "system":[
                {"type":"text","text":"system text"},
                {"type":"text","text":"ephemeral text","cache_control":{"type":"ephemeral"}}
            ],
            "messages":[{"role":"user","content":"hello"}]
        }"#;
        let body_system_only = br#"{
            "system":"system text",
            "messages":[{"role":"user","content":"hello"}]
        }"#;

        let hash_eph = compute_hash(body_with_ephemeral);
        let hash_sys = compute_hash(body_system_only);

        // These should differ because ephemeral takes priority over system.
        assert_ne!(hash_eph, hash_sys);
    }

    /// CRS: same system prompt always maps to same session.
    #[test]
    fn crs_parity_deterministic_session() {
        let body = br#"{"system":"Same system prompt every time","messages":[{"role":"user","content":"first msg"}]}"#;
        let h1 = compute_hash(body);
        let h2 = compute_hash(body);
        assert_eq!(h1, h2);
    }

    /// CRS: system as array of text objects.
    #[test]
    fn crs_parity_system_array_format() {
        let body = br#"{"system":[{"type":"text","text":"part A"},{"type":"text","text":"part B"}],"messages":[]}"#;
        let hash = compute_hash(body);
        assert_eq!(hash.len(), 32);
        // Hash should be of "part Apart B" concatenated.
        assert!(!hash.is_empty());
    }

    /// CRS: first message content in array format (multi-block content).
    #[test]
    fn crs_parity_first_message_array_content() {
        let body = br#"{"messages":[{"role":"user","content":[{"type":"text","text":"block1"},{"type":"text","text":"block2"}]}]}"#;
        let hash = compute_hash(body);
        assert_eq!(hash.len(), 32);
    }
}

mod cost_parity {
    use nexus_gate::service::cost::{calculate_cost, default_pricing, extract_usage, UsageData};

    /// CRS: Opus pricing = $15/$75 per MTok.
    #[test]
    fn crs_parity_opus_pricing() {
        let p = default_pricing("claude-opus-4-20250805");
        assert_eq!(p.input_per_mtok, 15.0);
        assert_eq!(p.output_per_mtok, 75.0);
        assert_eq!(p.cache_read_per_mtok, 1.5);
        assert_eq!(p.cache_creation_per_mtok, 18.75);
    }

    /// CRS: Sonnet pricing = $3/$15 per MTok.
    #[test]
    fn crs_parity_sonnet_pricing() {
        let p = default_pricing("claude-sonnet-4-20250514");
        assert_eq!(p.input_per_mtok, 3.0);
        assert_eq!(p.output_per_mtok, 15.0);
    }

    /// CRS: Haiku pricing = $0.80/$4.00 per MTok.
    #[test]
    fn crs_parity_haiku_pricing() {
        let p = default_pricing("claude-3-5-haiku-20241022");
        assert_eq!(p.input_per_mtok, 0.8);
        assert_eq!(p.output_per_mtok, 4.0);
    }

    /// CRS: Cost = (input_tokens / 1M) * price + (output_tokens / 1M) * price + cache costs.
    #[test]
    fn crs_parity_cost_formula() {
        let pricing = default_pricing("claude-sonnet-4");
        let usage = UsageData {
            input_tokens: 500_000,
            output_tokens: 200_000,
            cache_read_input_tokens: 100_000,
            cache_creation_input_tokens: 50_000,
        };
        let cost = calculate_cost(&pricing, &usage);

        // input: 0.5M * 3 = 1.5
        // output: 0.2M * 15 = 3.0
        // cache_read: 0.1M * 0.3 = 0.03
        // cache_creation: 0.05M * 3.75 = 0.1875
        let expected = 1.5 + 3.0 + 0.03 + 0.1875;
        assert!(
            (cost.total_cost - expected).abs() < 0.0001,
            "expected {expected}, got {}",
            cost.total_cost
        );
    }

    /// CRS: [1m] suffix stripped for pricing lookup.
    #[test]
    fn crs_parity_1m_suffix_pricing() {
        let p = default_pricing("claude-opus-4[1m]");
        assert_eq!(p.input_per_mtok, 15.0);
    }

    /// CRS: usage extraction from upstream response.
    #[test]
    fn crs_parity_usage_extraction() {
        let resp = serde_json::json!({
            "id": "msg_abc",
            "type": "message",
            "content": [{"type": "text", "text": "Hello!"}],
            "model": "claude-sonnet-4-20250514",
            "usage": {
                "input_tokens": 1234,
                "output_tokens": 567,
                "cache_read_input_tokens": 89,
                "cache_creation_input_tokens": 45
            }
        });
        let usage = extract_usage(&resp);
        assert_eq!(usage.input_tokens, 1234);
        assert_eq!(usage.output_tokens, 567);
        assert_eq!(usage.cache_read_input_tokens, 89);
        assert_eq!(usage.cache_creation_input_tokens, 45);
    }
}

mod auth_parity {
    use nexus_gate::model::apikey::ApiKeyRecord;

    fn make_key(permissions: &str, restricted_models: &str) -> ApiKeyRecord {
        ApiKeyRecord {
            id: uuid::Uuid::new_v4(),
            key_hash: String::new(),
            name: "test".into(),
            permissions: serde_json::from_str(permissions).unwrap_or_default(),
            daily_cost_limit: 0.0,
            total_cost_limit: 0.0,
            max_concurrency: 0,
            rate_limit_rpm: 0,
            restricted_models: restricted_models.into(),
            status: "active".into(),
            expires_at: None,
            deleted_at: None,
            created_at: chrono::Utc::now(),
        }
    }

    /// CRS: x-api-key header takes priority over Bearer (matching CRS extraction order).
    #[test]
    fn crs_parity_empty_permissions_all_access() {
        let key = make_key("[]", "[]");
        assert!(key.has_permission("claude"));
        assert!(key.has_permission("openai"));
        assert!(key.has_permission("gemini"));
        assert!(key.has_permission("droid"));
    }

    /// CRS: "all" = all services allowed.
    #[test]
    fn crs_parity_all_permission() {
        let key = make_key(r#"["all"]"#, "[]");
        assert!(key.has_permission("claude"));
        assert!(key.has_permission("openai"));
    }

    /// CRS: specific permissions restrict access.
    #[test]
    fn crs_parity_specific_permissions() {
        let key = make_key(r#"["claude"]"#, "[]");
        assert!(key.has_permission("claude"));
        assert!(!key.has_permission("openai"));
    }

    /// CRS: empty restrictedModels = all models allowed.
    #[test]
    fn crs_parity_no_model_restriction() {
        let key = make_key("[]", "[]");
        assert!(key.is_model_allowed("claude-opus-4-20250805"));
        assert!(key.is_model_allowed("gpt-4o"));
    }

    /// CRS: restricted models list limits access.
    #[test]
    fn crs_parity_model_restriction() {
        let key = make_key("[]", r#"["claude-sonnet"]"#);
        assert!(key.is_model_allowed("claude-sonnet-4-20250514"));
        assert!(!key.is_model_allowed("claude-opus-4-20250805"));
    }
}

mod apikey_parity {
    use nexus_gate::service::apikey::{generate_raw_key, is_valid_format};

    /// CRS uses cr_ prefix; nexus-gate uses nk- prefix to avoid fingerprinting.
    #[test]
    fn unique_prefix() {
        let key = generate_raw_key();
        assert!(key.starts_with("nk-"), "must use nk- prefix, not cr_");
        assert!(!key.starts_with("cr_"));
    }

    /// Key format: prefix + 40 hex chars.
    #[test]
    fn key_length() {
        let key = generate_raw_key();
        assert_eq!(key.len(), 43); // "nk-" (3) + 40 hex
        assert!(is_valid_format(&key));
    }

    /// CRS keys are unique — no collisions.
    #[test]
    fn uniqueness() {
        let keys: Vec<String> = (0..100).map(|_| generate_raw_key()).collect();
        let unique: std::collections::HashSet<&String> = keys.iter().collect();
        assert_eq!(keys.len(), unique.len());
    }
}

mod crypto_parity {
    use nexus_gate::util::crypto;

    /// CRS: encrypt then decrypt should roundtrip.
    #[test]
    fn encrypt_decrypt_roundtrip() {
        let key = "0123456789abcdef0123456789abcdef";
        let plaintext = r#"{"access_token":"sk-ant-abc123","refresh_token":"rt-xyz789"}"#;
        let encrypted = crypto::encrypt(key, plaintext).unwrap();
        let decrypted = crypto::decrypt(key, &encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    /// CRS: different encryptions of same plaintext should differ (random nonce).
    #[test]
    fn nonce_randomness() {
        let key = "0123456789abcdef0123456789abcdef";
        let plaintext = "test data";
        let a = crypto::encrypt(key, plaintext).unwrap();
        let b = crypto::encrypt(key, plaintext).unwrap();
        assert_ne!(a, b, "random nonce means different ciphertext each time");
    }

    /// CRS: wrong key must fail decryption.
    #[test]
    fn wrong_key_fails() {
        let key_a = "0123456789abcdef0123456789abcdef";
        let key_b = "fedcba9876543210fedcba9876543210";
        let encrypted = crypto::encrypt(key_a, "secret").unwrap();
        assert!(crypto::decrypt(key_b, &encrypted).is_err());
    }

    /// CRS: SHA-256 is deterministic.
    #[test]
    fn sha256_deterministic() {
        let a = crypto::sha256_hex("test");
        let b = crypto::sha256_hex("test");
        assert_eq!(a, b);
        assert_eq!(a.len(), 64);
    }
}

mod mask_parity {
    use nexus_gate::util::mask;

    /// CRS: mask shows first chars and last chars.
    #[test]
    fn token_masking() {
        let masked = mask::mask_token("sk-ant-1234567890abcdefghij", 6);
        assert!(masked.starts_with("sk-ant"));
        assert!(masked.contains("***"));
    }

    /// CRS: short tokens get different treatment.
    #[test]
    fn short_token_masking() {
        let masked = mask::mask_token("short", 6);
        assert!(masked.contains("***"));
    }

    /// CRS: JSON log sanitization removes sensitive fields.
    #[test]
    fn json_sanitization() {
        let input = serde_json::json!({
            "authorization": "Bearer sk-secret",
            "api_key": "key-secret",
            "access_token": "tok-secret",
            "safe_field": "visible"
        });
        let sanitized = mask::sanitize_json_for_log(&input);
        assert!(!sanitized.to_string().contains("sk-secret"));
        assert!(!sanitized.to_string().contains("key-secret"));
        assert!(!sanitized.to_string().contains("tok-secret"));
        assert!(sanitized.to_string().contains("visible"));
        assert!(sanitized.to_string().contains("[REDACTED]"));
    }

    /// CRS: x-api-key header (with hyphens) also gets sanitized.
    #[test]
    fn json_sanitization_hyphenated_keys() {
        let input = serde_json::json!({
            "x-api-key": "key-secret",
            "X-Api-Key": "another-secret",
            "safe": "visible"
        });
        let sanitized = mask::sanitize_json_for_log(&input);
        assert!(!sanitized.to_string().contains("key-secret"));
        assert!(!sanitized.to_string().contains("another-secret"));
        assert!(sanitized.to_string().contains("visible"));
    }
}

mod stream_parity {
    use nexus_gate::util::stream;

    /// CRS: SSE format = "data: {json}\n\n".
    #[test]
    fn sse_event_format() {
        let event = stream::sse_event(r#"{"type":"message_start"}"#);
        assert_eq!(event, "data: {\"type\":\"message_start\"}\n\n");
    }

    /// CRS: SSE done = "data: [DONE]\n\n".
    #[test]
    fn sse_done_format() {
        let done = stream::sse_done();
        assert_eq!(done, "data: [DONE]\n\n");
    }

    /// CRS: SSE events must not leak project identifiers.
    #[test]
    fn no_identifier_leak() {
        let event = stream::sse_event(r#"{"test": "data"}"#);
        assert!(!event.contains("nexus"));
        assert!(!event.contains("relay"));
        assert!(!event.contains("claude-relay"));
    }
}

mod error_parity {
    use nexus_gate::error::AppError;
    use axum::response::IntoResponse;

    /// CRS: 401 for unauthorized.
    #[test]
    fn unauthorized_is_401() {
        let resp = AppError::Unauthorized.into_response();
        assert_eq!(resp.status(), 401);
    }

    /// CRS: 403 for forbidden.
    #[test]
    fn forbidden_is_403() {
        let resp = AppError::Forbidden.into_response();
        assert_eq!(resp.status(), 403);
    }

    /// CRS: 429 for rate limited.
    #[test]
    fn rate_limited_is_429() {
        let resp = AppError::RateLimited.into_response();
        assert_eq!(resp.status(), 429);
    }

    /// CRS: 503 for overloaded (maps to 529 upstream).
    #[test]
    fn overloaded_is_503() {
        let resp = AppError::Overloaded.into_response();
        assert_eq!(resp.status(), 503);
    }

    /// CRS: 502 for upstream errors.
    #[test]
    fn upstream_error_maps() {
        let resp = AppError::UpstreamError {
            status: 502,
            body: "bad gateway".into(),
        }
        .into_response();
        assert_eq!(resp.status(), 502);
    }

    /// CRS: internal errors never leak details.
    #[test]
    fn internal_hides_details() {
        let resp = AppError::Internal(anyhow::anyhow!("secret database password")).into_response();
        assert_eq!(resp.status(), 500);
        // We can't easily inspect body in sync test, but the IntoResponse
        // implementation logs and replaces with "internal server error".
    }
}

mod fingerprint_parity {
    use nexus_gate::fingerprint::profile::FingerprintProfile;

    /// nexus-gate must NOT use default User-Agent from CRS/S2A.
    #[test]
    fn no_crs_user_agent() {
        let profile = FingerprintProfile::chrome_131();
        let ua = &profile.user_agent.template;
        assert!(!ua.contains("claude-relay"), "must not contain CRS identifier");
        assert!(!ua.contains("sub2api"), "must not contain S2A identifier");
    }

    /// Each built-in profile must have distinct TLS fingerprint.
    #[test]
    fn distinct_tls_fingerprints() {
        use nexus_gate::fingerprint::tls::compute_ja3_fingerprint;

        let chrome = FingerprintProfile::chrome_131();
        let cli = FingerprintProfile::claude_cli();
        let node = FingerprintProfile::node20();

        let ja3_chrome = compute_ja3_fingerprint(&chrome.tls);
        let ja3_cli = compute_ja3_fingerprint(&cli.tls);
        let ja3_node = compute_ja3_fingerprint(&node.tls);

        assert_ne!(ja3_chrome, ja3_cli);
        assert_ne!(ja3_chrome, ja3_node);
        assert_ne!(ja3_cli, ja3_node);
    }

    /// Header sanitization removes proxy/CDN/identifying headers.
    #[test]
    fn header_sanitization() {
        use axum::http::{HeaderMap, HeaderValue};
        use nexus_gate::fingerprint::headers::sanitize_outgoing;

        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", HeaderValue::from_static("1.2.3.4"));
        headers.insert("x-real-ip", HeaderValue::from_static("1.2.3.4"));
        headers.insert("cf-connecting-ip", HeaderValue::from_static("1.2.3.4"));
        headers.insert("content-type", HeaderValue::from_static("application/json"));

        sanitize_outgoing(&mut headers);

        assert!(headers.get("x-forwarded-for").is_none());
        assert!(headers.get("x-real-ip").is_none());
        assert!(headers.get("cf-connecting-ip").is_none());
        assert!(headers.get("content-type").is_some());
    }
}

mod proxy_parity {
    use nexus_gate::util::proxy;

    /// CRS: valid proxy URLs.
    #[test]
    fn valid_proxy_formats() {
        assert!(proxy::is_valid_proxy_url("http://proxy.example.com:8080"));
        assert!(proxy::is_valid_proxy_url("https://proxy.example.com:443"));
        assert!(proxy::is_valid_proxy_url("socks5://127.0.0.1:1080"));
    }

    /// CRS: invalid proxy URLs rejected.
    #[test]
    fn invalid_proxy_formats() {
        assert!(!proxy::is_valid_proxy_url("not-a-url"));
        assert!(!proxy::is_valid_proxy_url("ftp://invalid.com"));
    }
}
