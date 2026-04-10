use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;

/// Top-level application configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub auth: AuthConfig,
    pub upstream: UpstreamConfig,
    pub scheduler: SchedulerConfig,
    pub rate_limit: RateLimitConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    #[serde(default = "default_request_timeout")]
    pub request_timeout_secs: u64,
    #[serde(default = "default_max_body")]
    pub max_body_bytes: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    #[serde(default = "default_max_conn")]
    pub max_connections: u32,
    #[serde(default = "default_min_conn")]
    pub min_connections: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    #[serde(default = "default_pool_size")]
    pub pool_size: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub encryption_key: String,
    #[serde(default = "default_session_ttl")]
    pub session_ttl_hours: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpstreamConfig {
    pub claude: UpstreamEntry,
    pub openai: UpstreamEntry,
    pub gemini: UpstreamEntry,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpstreamEntry {
    pub base_url: String,
    #[serde(default)]
    pub api_version: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SchedulerConfig {
    #[serde(default = "default_sticky_ttl")]
    pub sticky_session_ttl_secs: u64,
    #[serde(default = "default_cleanup_interval")]
    pub concurrency_cleanup_interval_secs: u64,
    #[serde(default = "default_overload_cooldown")]
    pub overload_cooldown_secs: u64,
    #[serde(default = "default_rate_limit_cooldown")]
    pub rate_limit_cooldown_secs: u64,
    #[serde(default = "default_concurrency_lease")]
    pub concurrency_lease_secs: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitConfig {
    #[serde(default = "default_global_rpm")]
    pub global_rpm: u32,
    #[serde(default = "default_per_key_rpm")]
    pub per_key_rpm: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default = "default_log_format")]
    pub format: String,
}

impl AppConfig {
    /// Load config from TOML file (if exists) then overlay environment variables.
    pub fn load() -> Result<Self> {
        let config_path = std::env::var("CONFIG_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("configs/default.toml"));

        let mut builder = String::new();

        if config_path.exists() {
            builder = std::fs::read_to_string(&config_path)
                .with_context(|| format!("reading config from {}", config_path.display()))?;
        }

        let mut cfg: AppConfig = toml::from_str(&builder).unwrap_or_else(|_| default_config());

        // Environment variable overrides
        if let Ok(v) = std::env::var("PORT") {
            if let Ok(p) = v.parse() {
                cfg.server.port = p;
            }
        }
        if let Ok(v) = std::env::var("HOST") {
            cfg.server.host = v;
        }
        if let Ok(v) = std::env::var("DATABASE_URL") {
            cfg.database.url = v;
        }
        if let Ok(v) = std::env::var("REDIS_URL") {
            cfg.redis.url = v;
        }
        if let Ok(v) = std::env::var("JWT_SECRET") {
            cfg.auth.jwt_secret = v;
        }
        if let Ok(v) = std::env::var("ENCRYPTION_KEY") {
            cfg.auth.encryption_key = v;
        }

        Ok(cfg)
    }
}

fn default_config() -> AppConfig {
    AppConfig {
        server: ServerConfig {
            host: "0.0.0.0".into(),
            port: 7900,
            request_timeout_secs: 600,
            max_body_bytes: 104_857_600,
        },
        database: DatabaseConfig {
            url: String::new(),
            max_connections: 20,
            min_connections: 2,
        },
        redis: RedisConfig {
            url: "redis://localhost:6379".into(),
            pool_size: 10,
        },
        auth: AuthConfig {
            jwt_secret: String::new(),
            encryption_key: String::new(),
            session_ttl_hours: 24,
        },
        upstream: UpstreamConfig {
            claude: UpstreamEntry {
                base_url: "https://api.anthropic.com".into(),
                api_version: Some("2023-06-01".into()),
            },
            openai: UpstreamEntry {
                base_url: "https://api.openai.com".into(),
                api_version: None,
            },
            gemini: UpstreamEntry {
                base_url: "https://generativelanguage.googleapis.com".into(),
                api_version: None,
            },
        },
        scheduler: SchedulerConfig {
            sticky_session_ttl_secs: 1_123_200, // 13 days
            concurrency_cleanup_interval_secs: 60,
            overload_cooldown_secs: 300,
            rate_limit_cooldown_secs: 300,
            concurrency_lease_secs: 300, // CRS default: 300s (5 min)
        },
        rate_limit: RateLimitConfig {
            global_rpm: 600,
            per_key_rpm: 60,
        },
        logging: LoggingConfig {
            level: "info".into(),
            format: "json".into(),
        },
    }
}

fn default_request_timeout() -> u64 {
    600
}
fn default_max_body() -> usize {
    104_857_600
}
fn default_max_conn() -> u32 {
    20
}
fn default_min_conn() -> u32 {
    2
}
fn default_pool_size() -> u32 {
    10
}
fn default_session_ttl() -> u64 {
    24
}
fn default_sticky_ttl() -> u64 {
    1_123_200 // 13 days in seconds (CRS uses 13-15 day TTL)
}
fn default_cleanup_interval() -> u64 {
    60
}
fn default_overload_cooldown() -> u64 {
    300
}
fn default_rate_limit_cooldown() -> u64 {
    300
}
fn default_concurrency_lease() -> u64 {
    120
}
fn default_global_rpm() -> u32 {
    600
}
fn default_per_key_rpm() -> u32 {
    60
}
fn default_log_level() -> String {
    "info".into()
}
fn default_log_format() -> String {
    "json".into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_default_toml() {
        let raw = include_str!("../configs/default.toml");
        let cfg: AppConfig = toml::from_str(raw).expect("default.toml should parse");
        assert_eq!(cfg.server.port, 7900);
        assert_eq!(cfg.database.max_connections, 20);
        assert_eq!(cfg.upstream.claude.base_url, "https://api.anthropic.com");
    }

    #[test]
    fn default_config_is_valid() {
        let cfg = default_config();
        assert_eq!(cfg.server.port, 7900);
        assert!(!cfg.upstream.openai.base_url.is_empty());
    }
}
