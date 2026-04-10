use super::profile::TlsProfile;
use anyhow::Result;
use reqwest::ClientBuilder;

/// Apply TLS fingerprint settings to a reqwest ClientBuilder.
///
/// NOTE: Full TLS ClientHello manipulation requires a custom TLS backend
/// (e.g. boring-sys or a patched rustls). This implementation applies
/// what's possible with reqwest's built-in rustls support. For production
/// use, integrate with `boring` crate for full cipher suite ordering and
/// extension control.
pub fn apply_tls_profile(builder: ClientBuilder, _profile: &TlsProfile) -> Result<ClientBuilder> {
    let mut builder = builder
        .use_rustls_tls()
        .https_only(false); // allow explicit HTTP for dev

    // Set minimum TLS version to 1.2 (matching real clients).
    builder = builder.min_tls_version(reqwest::tls::Version::TLS_1_2);

    // ALPN configuration.
    // Note: reqwest/rustls automatically handles h2/http1.1 ALPN.
    // Full ALPN control requires a custom TLS connector (boring-ssl).
    // For now, we rely on reqwest's default ALPN negotiation.

    Ok(builder)
}

/// Compute JA3 hash from a TLS profile for debugging/verification.
/// JA3 = MD5(SSLVersion,Ciphers,Extensions,EllipticCurves,EllipticCurveFormats)
pub fn compute_ja3_fingerprint(profile: &TlsProfile) -> String {
    use sha2::{Digest, Sha256};

    let ciphers = profile.cipher_suites.join("-");
    let extensions = profile.extensions.join("-");
    let curves = profile.curves.join("-");

    let ja3_raw = format!("771,{ciphers},{extensions},{curves},0");

    let mut hasher = Sha256::new();
    hasher.update(ja3_raw.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fingerprint::profile::FingerprintProfile;

    #[test]
    fn ja3_deterministic() {
        let profile = FingerprintProfile::chrome_131();
        let a = compute_ja3_fingerprint(&profile.tls);
        let b = compute_ja3_fingerprint(&profile.tls);
        assert_eq!(a, b);
    }

    #[test]
    fn ja3_differs_between_profiles() {
        let chrome = FingerprintProfile::chrome_131();
        let node = FingerprintProfile::node20();
        let a = compute_ja3_fingerprint(&chrome.tls);
        let b = compute_ja3_fingerprint(&node.tls);
        assert_ne!(a, b);
    }
}
