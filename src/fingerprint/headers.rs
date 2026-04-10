use super::profile::{FingerprintProfile, UserAgentConfig};
use axum::http::{HeaderMap, HeaderValue};

/// Apply the fingerprint profile's header ordering and extra headers
/// to an outgoing header map, preserving required headers while matching
/// the target client's header ordering signature.
pub fn apply_header_profile(
    base_headers: &HeaderMap,
    profile: &FingerprintProfile,
) -> HeaderMap {
    let mut ordered = HeaderMap::new();

    // 1. Insert headers according to the profile's defined order.
    for key_name in &profile.header_order {
        // Skip pseudo-headers (HTTP/2 only, handled by transport).
        if key_name.starts_with(':') {
            continue;
        }
        if let Some(val) = base_headers.get(key_name.as_str()) {
            if let Ok(k) = axum::http::header::HeaderName::from_bytes(key_name.as_bytes()) {
                ordered.insert(k, val.clone());
            }
        }
    }

    // 2. Append any base headers that weren't in the ordering.
    for (key, val) in base_headers.iter() {
        if !ordered.contains_key(key) {
            ordered.insert(key.clone(), val.clone());
        }
    }

    // 3. Inject extra headers defined in the profile.
    for (k, v) in &profile.extra_headers {
        if let (Ok(name), Ok(val)) = (
            axum::http::header::HeaderName::from_bytes(k.as_bytes()),
            HeaderValue::from_str(v),
        ) {
            ordered.insert(name, val);
        }
    }

    // 4. Set User-Agent from the profile.
    let ua = resolve_user_agent(&profile.user_agent);
    if let Ok(val) = HeaderValue::from_str(&ua) {
        ordered.insert(axum::http::header::USER_AGENT, val);
    }

    ordered
}

/// Resolve the User-Agent string from the template.
fn resolve_user_agent(config: &UserAgentConfig) -> String {
    let version = if config.rotate {
        // In production: rotate through recent versions.
        // For now, use a realistic recent version.
        "1.0.33"
    } else {
        ""
    };

    config
        .template
        .replace("{version}", version)
        .replace("{os}", std::env::consts::OS)
        .replace("{arch}", std::env::consts::ARCH)
}

/// Strip all identifying headers that could leak proxy information.
pub fn sanitize_outgoing(headers: &mut HeaderMap) {
    let remove = [
        "x-forwarded-for",
        "x-forwarded-proto",
        "x-forwarded-host",
        "x-real-ip",
        "via",
        "forwarded",
        "cf-connecting-ip",
        "cf-ray",
        "cf-ipcountry",
        "cf-visitor",
        "cdn-loop",
        "x-amzn-trace-id",
        "x-vercel-id",
    ];

    for name in &remove {
        headers.remove(*name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fingerprint::profile::FingerprintProfile;

    #[test]
    fn header_ordering_applied() {
        let profile = FingerprintProfile::chrome_131();
        let mut base = HeaderMap::new();
        base.insert("accept", HeaderValue::from_static("*/*"));
        base.insert("content-type", HeaderValue::from_static("application/json"));
        base.insert("user-agent", HeaderValue::from_static("old-ua"));

        let result = apply_header_profile(&base, &profile);

        // User-Agent should be replaced by the profile's.
        assert!(result.get("user-agent").unwrap().to_str().unwrap().contains("Chrome"));
    }

    #[test]
    fn extra_headers_injected() {
        let profile = FingerprintProfile::claude_cli();
        let base = HeaderMap::new();
        let result = apply_header_profile(&base, &profile);
        assert_eq!(
            result.get("x-stainless-lang").unwrap().to_str().unwrap(),
            "js"
        );
    }

    #[test]
    fn sanitize_removes_proxy_headers() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", HeaderValue::from_static("1.2.3.4"));
        headers.insert("cf-ray", HeaderValue::from_static("abc123"));
        headers.insert("content-type", HeaderValue::from_static("application/json"));

        sanitize_outgoing(&mut headers);

        assert!(headers.get("x-forwarded-for").is_none());
        assert!(headers.get("cf-ray").is_none());
        assert!(headers.get("content-type").is_some()); // preserved
    }

    #[test]
    fn resolve_ua_with_placeholders() {
        let config = UserAgentConfig {
            template: "test/{version} ({os}; {arch})".into(),
            rotate: true,
        };
        let ua = resolve_user_agent(&config);
        assert!(ua.contains("1.0.33"));
        assert!(!ua.contains("{os}"));
        assert!(!ua.contains("{arch}"));
    }
}
