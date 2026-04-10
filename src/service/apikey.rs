use rand::Rng;

/// Generate a random API key string.
/// Format: `nk-<40 random hex chars>` — completely unique prefix,
/// no relation to any existing relay service.
pub fn generate_raw_key() -> String {
    let mut rng = rand::thread_rng();
    let random_bytes: [u8; 20] = rng.gen();
    let hex: String = random_bytes.iter().map(|b| format!("{b:02x}")).collect();
    format!("nk-{hex}")
}

/// Validate the raw key format before hashing.
pub fn is_valid_format(key: &str) -> bool {
    key.starts_with("nk-") && key.len() == 43 // "nk-" (3) + 40 hex
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_key_format() {
        let key = generate_raw_key();
        assert!(key.starts_with("nk-"));
        assert_eq!(key.len(), 43);
    }

    #[test]
    fn generated_keys_unique() {
        let a = generate_raw_key();
        let b = generate_raw_key();
        assert_ne!(a, b);
    }

    #[test]
    fn valid_format_check() {
        assert!(is_valid_format("nk-0123456789abcdef0123456789abcdef01234567"));
        assert!(!is_valid_format("cr_something"));
        assert!(!is_valid_format("sk-short"));
        assert!(!is_valid_format("nk-tooshort"));
    }
}
