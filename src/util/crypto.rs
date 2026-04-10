use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::{Context, Result};
use crate::error::AppError;
use rand::RngCore;
use sha2::{Digest, Sha256};

/// Encrypt plaintext using AES-256-GCM.
/// Returns: base64(nonce || ciphertext).
pub fn encrypt(key: &str, plaintext: &str) -> Result<String, AppError> {
    let key_bytes = derive_key(key);
    let cipher =
        Aes256Gcm::new_from_slice(&key_bytes).map_err(|e| AppError::Internal(e.into()))?;

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| AppError::Internal(anyhow::anyhow!("encryption failed: {e}")))?;

    let mut combined = nonce_bytes.to_vec();
    combined.extend(ciphertext);

    Ok(base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &combined,
    ))
}

/// Decrypt data produced by `encrypt`.
pub fn decrypt(key: &str, encoded: &str) -> Result<String> {
    use base64::Engine;
    let combined = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .context("base64 decode")?;

    if combined.len() < 12 {
        anyhow::bail!("ciphertext too short");
    }

    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let key_bytes = derive_key(key);
    let cipher =
        Aes256Gcm::new_from_slice(&key_bytes).context("invalid key")?;
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| anyhow::anyhow!("decryption failed"))?;

    String::from_utf8(plaintext).context("utf8 decode")
}

/// Derive a 32-byte key from an arbitrary-length secret.
fn derive_key(secret: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    hasher.finalize().into()
}

/// SHA-256 hash a string, returning hex-encoded result.
pub fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let key = "test-encryption-key-32-chars!!!!";
        let plaintext = r#"{"access_token":"sk-xxx","refresh_token":"rt-yyy"}"#;

        let encrypted = encrypt(key, plaintext).unwrap();
        assert_ne!(encrypted, plaintext);
        assert!(!encrypted.contains("sk-xxx"));

        let decrypted = decrypt(key, &encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn decrypt_wrong_key_fails() {
        let encrypted = encrypt("key-a", "secret").unwrap();
        let result = decrypt("key-b", &encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn different_encryptions_different_output() {
        let key = "same-key";
        let a = encrypt(key, "hello").unwrap();
        let b = encrypt(key, "hello").unwrap();
        // Different nonces → different ciphertexts.
        assert_ne!(a, b);
    }

    #[test]
    fn sha256_deterministic() {
        let a = sha256_hex("test");
        let b = sha256_hex("test");
        assert_eq!(a, b);
        assert_eq!(a.len(), 64);
    }

    #[test]
    fn sha256_different_inputs() {
        assert_ne!(sha256_hex("a"), sha256_hex("b"));
    }
}
