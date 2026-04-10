use anyhow::Result;
use reqwest::Proxy;

/// Build a reqwest Proxy from a URL string.
/// Supports: http://, https://, socks5://, socks5h://
pub fn build_proxy(url: &str) -> Result<Proxy> {
    let proxy = Proxy::all(url)?;
    Ok(proxy)
}

/// Validate a proxy URL format.
pub fn is_valid_proxy_url(url: &str) -> bool {
    url.starts_with("http://")
        || url.starts_with("https://")
        || url.starts_with("socks5://")
        || url.starts_with("socks5h://")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_proxy_urls() {
        assert!(is_valid_proxy_url("http://127.0.0.1:8080"));
        assert!(is_valid_proxy_url("https://proxy.example.com:443"));
        assert!(is_valid_proxy_url("socks5://127.0.0.1:1080"));
        assert!(is_valid_proxy_url("socks5h://user:pass@host:1080"));
    }

    #[test]
    fn invalid_proxy_urls() {
        assert!(!is_valid_proxy_url("ftp://bad"));
        assert!(!is_valid_proxy_url("not-a-url"));
        assert!(!is_valid_proxy_url(""));
    }

    #[test]
    fn build_proxy_success() {
        let result = build_proxy("socks5://127.0.0.1:1080");
        assert!(result.is_ok());
    }
}
