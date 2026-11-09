//! WireGuard key handling. Keys are standard Curve25519 keys; we never
//! reimplement the crypto ourselves, we use the audited x25519-dalek crate.

use base64::{engine::general_purpose::STANDARD, Engine as _};
use x25519_dalek::{PublicKey, StaticSecret};

use crate::CoreError;

/// Generate a fresh random private key.
pub fn generate_private_key() -> [u8; 32] {
    StaticSecret::random().to_bytes()
}

/// Derive the public key for a private key.
pub fn public_key_from_private(private: &[u8; 32]) -> [u8; 32] {
    let secret = StaticSecret::from(*private);
    PublicKey::from(&secret).to_bytes()
}

/// Generate a fresh preshared key (same entropy source).
pub fn generate_preshared_key() -> [u8; 32] {
    StaticSecret::random().to_bytes()
}

pub fn key_to_base64(key: &[u8; 32]) -> String {
    STANDARD.encode(key)
}

pub fn key_from_base64(s: &str) -> Result<[u8; 32], CoreError> {
    let bytes = STANDARD
        .decode(s.trim())
        .map_err(|e| CoreError::BadKey(e.to_string()))?;
    if bytes.len() != 32 {
        return Err(CoreError::BadKey(format!(
            "decoded key is {} bytes, expected 32",
            bytes.len()
        )));
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let priv_k = generate_private_key();
        let b64 = key_to_base64(&priv_k);
        assert_eq!(b64.len(), 44);
        let decoded = key_from_base64(&b64).unwrap();
        assert_eq!(decoded, priv_k);
        let pub_k = public_key_from_private(&priv_k);
        assert_ne!(pub_k, priv_k);
        assert_eq!(pub_k.len(), 32);
    }

    #[test]
    fn rejects_bad_keys() {
        assert!(key_from_base64("not base64!!!").is_err());
        let short = base64::engine::Engine::encode(&STANDARD, [1u8; 31]);
        assert!(key_from_base64(&short).is_err());
    }
}
