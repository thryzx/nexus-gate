use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A fingerprint profile defines how the relay presents itself to upstream APIs.
/// Analogous to browser fingerprint profiles in anti-detect browsers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintProfile {
    pub id: Uuid,
    pub name: String,
    pub tls: TlsProfile,
    pub http2: Http2Settings,
    pub header_order: Vec<String>,
    pub user_agent: UserAgentConfig,
    pub extra_headers: std::collections::HashMap<String, String>,
}

/// TLS ClientHello fingerprint configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsProfile {
    /// Cipher suites in priority order (TLS 1.3 + 1.2).
    pub cipher_suites: Vec<String>,
    /// TLS extensions to advertise.
    pub extensions: Vec<String>,
    /// Supported elliptic curves.
    pub curves: Vec<String>,
    /// ALPN protocols (e.g. ["h2", "http/1.1"]).
    pub alpn: Vec<String>,
    /// Signature algorithms.
    pub signature_algorithms: Vec<String>,
}

/// HTTP/2 SETTINGS frame parameters — different clients have distinct
/// default values that can be used for fingerprinting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Http2Settings {
    pub header_table_size: u32,
    pub enable_push: bool,
    pub max_concurrent_streams: u32,
    pub initial_window_size: u32,
    pub max_frame_size: u32,
    pub max_header_list_size: u32,
}

/// User-Agent construction configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAgentConfig {
    /// Template string with placeholders: {version}, {os}, {arch}.
    pub template: String,
    /// Whether to auto-rotate versions.
    pub rotate: bool,
}

impl Default for FingerprintProfile {
    fn default() -> Self {
        Self::chrome_131()
    }
}

impl FingerprintProfile {
    /// Profile mimicking Chrome 131 on macOS.
    pub fn chrome_131() -> Self {
        Self {
            id: Uuid::nil(),
            name: "chrome-131-macos".into(),
            tls: TlsProfile {
                cipher_suites: vec![
                    "TLS_AES_128_GCM_SHA256".into(),
                    "TLS_AES_256_GCM_SHA384".into(),
                    "TLS_CHACHA20_POLY1305_SHA256".into(),
                    "TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256".into(),
                    "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256".into(),
                    "TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384".into(),
                    "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384".into(),
                    "TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256".into(),
                    "TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256".into(),
                ],
                extensions: vec![
                    "server_name".into(),
                    "extended_master_secret".into(),
                    "renegotiation_info".into(),
                    "supported_groups".into(),
                    "ec_point_formats".into(),
                    "session_ticket".into(),
                    "application_layer_protocol_negotiation".into(),
                    "status_request".into(),
                    "signature_algorithms".into(),
                    "signed_certificate_timestamp".into(),
                    "key_share".into(),
                    "psk_key_exchange_modes".into(),
                    "supported_versions".into(),
                    "compress_certificate".into(),
                    "application_settings".into(),
                ],
                curves: vec!["X25519".into(), "P-256".into(), "P-384".into()],
                alpn: vec!["h2".into(), "http/1.1".into()],
                signature_algorithms: vec![
                    "ecdsa_secp256r1_sha256".into(),
                    "rsa_pss_rsae_sha256".into(),
                    "rsa_pkcs1_sha256".into(),
                    "ecdsa_secp384r1_sha384".into(),
                    "rsa_pss_rsae_sha384".into(),
                    "rsa_pkcs1_sha384".into(),
                    "rsa_pss_rsae_sha512".into(),
                    "rsa_pkcs1_sha512".into(),
                ],
            },
            http2: Http2Settings {
                header_table_size: 65536,
                enable_push: false,
                max_concurrent_streams: 1000,
                initial_window_size: 6291456,
                max_frame_size: 16384,
                max_header_list_size: 262144,
            },
            header_order: vec![
                ":method".into(),
                ":authority".into(),
                ":scheme".into(),
                ":path".into(),
                "content-type".into(),
                "user-agent".into(),
                "accept".into(),
                "accept-encoding".into(),
                "accept-language".into(),
            ],
            user_agent: UserAgentConfig {
                template: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36".into(),
                rotate: false,
            },
            extra_headers: Default::default(),
        }
    }

    /// Profile mimicking the official Claude CLI.
    pub fn claude_cli() -> Self {
        Self {
            id: Uuid::nil(),
            name: "claude-cli".into(),
            tls: TlsProfile {
                cipher_suites: vec![
                    "TLS_AES_256_GCM_SHA384".into(),
                    "TLS_CHACHA20_POLY1305_SHA256".into(),
                    "TLS_AES_128_GCM_SHA256".into(),
                    "TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384".into(),
                    "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384".into(),
                    "TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256".into(),
                    "TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256".into(),
                    "TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256".into(),
                    "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256".into(),
                ],
                extensions: vec![
                    "server_name".into(),
                    "supported_groups".into(),
                    "signature_algorithms".into(),
                    "key_share".into(),
                    "psk_key_exchange_modes".into(),
                    "supported_versions".into(),
                    "application_layer_protocol_negotiation".into(),
                ],
                curves: vec!["X25519".into(), "P-256".into(), "P-384".into()],
                alpn: vec!["h2".into(), "http/1.1".into()],
                signature_algorithms: vec![
                    "ecdsa_secp256r1_sha256".into(),
                    "rsa_pss_rsae_sha256".into(),
                    "rsa_pkcs1_sha256".into(),
                ],
            },
            http2: Http2Settings {
                header_table_size: 4096,
                enable_push: false,
                max_concurrent_streams: 100,
                initial_window_size: 65535,
                max_frame_size: 16384,
                max_header_list_size: 16384,
            },
            header_order: vec![
                ":method".into(),
                ":path".into(),
                ":authority".into(),
                ":scheme".into(),
                "content-type".into(),
                "authorization".into(),
                "anthropic-version".into(),
                "user-agent".into(),
                "accept".into(),
            ],
            user_agent: UserAgentConfig {
                template: "claude-code/{version} ({os}; {arch})".into(),
                rotate: true,
            },
            extra_headers: [
                ("x-stainless-lang".to_string(), "js".to_string()),
                ("x-stainless-os".to_string(), "Darwin".to_string()),
                ("x-stainless-arch".to_string(), "arm64".to_string()),
                ("x-stainless-runtime".to_string(), "node".to_string()),
            ]
            .into(),
        }
    }

    /// Profile mimicking Node.js 20 / axios client.
    pub fn node20() -> Self {
        Self {
            id: Uuid::nil(),
            name: "node20-axios".into(),
            tls: TlsProfile {
                cipher_suites: vec![
                    "TLS_AES_256_GCM_SHA384".into(),
                    "TLS_CHACHA20_POLY1305_SHA256".into(),
                    "TLS_AES_128_GCM_SHA256".into(),
                    "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256".into(),
                    "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384".into(),
                ],
                extensions: vec![
                    "server_name".into(),
                    "supported_groups".into(),
                    "application_layer_protocol_negotiation".into(),
                    "supported_versions".into(),
                    "signature_algorithms".into(),
                    "key_share".into(),
                    "psk_key_exchange_modes".into(),
                ],
                curves: vec!["X25519".into(), "P-256".into(), "P-384".into()],
                alpn: vec!["http/1.1".into()],
                signature_algorithms: vec![
                    "ecdsa_secp256r1_sha256".into(),
                    "rsa_pss_rsae_sha256".into(),
                    "rsa_pkcs1_sha256".into(),
                ],
            },
            http2: Http2Settings::default(),
            header_order: vec![
                "accept".into(),
                "content-type".into(),
                "authorization".into(),
                "user-agent".into(),
                "accept-encoding".into(),
            ],
            user_agent: UserAgentConfig {
                template: "node-fetch/1.0 (+https://github.com/bitinn/node-fetch)".into(),
                rotate: false,
            },
            extra_headers: Default::default(),
        }
    }
}

impl Default for Http2Settings {
    fn default() -> Self {
        Self {
            header_table_size: 4096,
            enable_push: false,
            max_concurrent_streams: 100,
            initial_window_size: 65535,
            max_frame_size: 16384,
            max_header_list_size: 16384,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_profile_is_chrome() {
        let p = FingerprintProfile::default();
        assert_eq!(p.name, "chrome-131-macos");
    }

    #[test]
    fn chrome_has_h2_alpn() {
        let p = FingerprintProfile::chrome_131();
        assert!(p.tls.alpn.contains(&"h2".to_string()));
    }

    #[test]
    fn claude_cli_has_stainless_headers() {
        let p = FingerprintProfile::claude_cli();
        assert!(p.extra_headers.contains_key("x-stainless-lang"));
    }

    #[test]
    fn node20_uses_http11_only() {
        let p = FingerprintProfile::node20();
        assert_eq!(p.tls.alpn, vec!["http/1.1"]);
    }

    #[test]
    fn profiles_serialise_roundtrip() {
        let p = FingerprintProfile::chrome_131();
        let json = serde_json::to_string(&p).unwrap();
        let parsed: FingerprintProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, p.name);
        assert_eq!(parsed.tls.cipher_suites.len(), p.tls.cipher_suites.len());
    }
}
