use super::profile::FingerprintProfile;
use super::tls;
use crate::model::account::AccountRecord;
use anyhow::Result;
use std::time::Duration;

/// The FingerprintEngine orchestrates TLS, HTTP/2, and header-level
/// fingerprint manipulation to make each upstream request appear as
/// if it originates from a different client environment.
pub struct FingerprintEngine {
    default_profile: FingerprintProfile,
}

impl FingerprintEngine {
    pub fn new() -> Self {
        Self {
            default_profile: FingerprintProfile::default(),
        }
    }

    pub fn with_profile(profile: FingerprintProfile) -> Self {
        Self {
            default_profile: profile,
        }
    }

    /// Build a `reqwest::Client` configured to match the fingerprint
    /// profile of the given account. If the account has its own proxy,
    /// that is also applied.
    pub fn build_client(&self, account: &AccountRecord) -> Result<reqwest::Client> {
        let profile = &self.default_profile;

        // Start with base builder.
        let builder = reqwest::Client::builder()
            .timeout(Duration::from_secs(600))
            .connect_timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(4)
            .tcp_keepalive(Duration::from_secs(60))
            .tcp_nodelay(true);

        // Apply TLS fingerprint.
        let builder = tls::apply_tls_profile(builder, &profile.tls)?;

        // Apply proxy if configured on the account.
        let builder = if let Some(ref proxy_url) = account.proxy_url {
            let proxy = reqwest::Proxy::all(proxy_url)?;
            builder.proxy(proxy)
        } else {
            builder
        };

        // Disable default headers — we control everything via the profile.
        let builder = builder.default_headers(reqwest::header::HeaderMap::new());

        Ok(builder.build()?)
    }

    /// Get the active fingerprint profile.
    pub fn profile(&self) -> &FingerprintProfile {
        &self.default_profile
    }
}

impl Default for FingerprintEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn dummy_account() -> AccountRecord {
        AccountRecord {
            id: Uuid::new_v4(),
            name: "test".into(),
            platform: "claude".into(),
            account_type: "oauth".into(),
            credentials_enc: "".into(),
            status: "active".into(),
            priority: 50,
            max_concurrency: 2,
            proxy_url: None,
            fingerprint_profile_id: None,
            description: None,
            schedulable: None,
            group_id: None,
            expires_at: None,
            rate_limit: None,
            extra_config: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn build_client_default_profile() {
        let engine = FingerprintEngine::new();
        let client = engine.build_client(&dummy_account());
        assert!(client.is_ok());
    }

    #[test]
    fn build_client_with_proxy() {
        let engine = FingerprintEngine::new();
        let mut acct = dummy_account();
        acct.proxy_url = Some("socks5://127.0.0.1:1080".into());
        let client = engine.build_client(&acct);
        assert!(client.is_ok());
    }

    #[test]
    fn build_client_claude_cli_profile() {
        let engine = FingerprintEngine::with_profile(FingerprintProfile::claude_cli());
        let client = engine.build_client(&dummy_account());
        assert!(client.is_ok());
    }

    #[test]
    fn profile_accessor() {
        let engine = FingerprintEngine::new();
        assert_eq!(engine.profile().name, "chrome-131-macos");
    }
}
