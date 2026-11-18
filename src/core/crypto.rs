// src/core/crypto.rs

use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, KeyInit},
};
use argon2::{Algorithm, Argon2, Params, Version};
use rand::RngCore;
use rand::rngs::OsRng;

pub fn derive_key(passphrase: &str, salt: &[u8]) -> Result<[u8; 32], argon2::Error> {
    let params = Params::new(65536, 3, 1, None)?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = [0u8; 32];
    argon2.hash_password_into(passphrase.as_bytes(), salt, &mut key)?;
    Ok(key)
}

// encrypt plaintext byte data with Aes256Gcm
pub fn encrypt_payload(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>, aes_gcm::Error> {
    let cipher = Aes256Gcm::new(&Key::<Aes256Gcm>::from(*key));

    // generate random 96 bit nonce
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::try_from(nonce_bytes.as_slice()).map_err(|_| aes_gcm::Error)?;

    let ciphertext = cipher.encrypt(&nonce, plaintext)?;

    // prepend nonce to ciphertext for transmission
    let mut encoded = nonce.to_vec();
    encoded.extend(ciphertext);
    Ok(encoded)
}

pub fn decrypt_payload(key: &[u8; 32], encoded: &[u8]) -> Result<Vec<u8>, String> {
    if encoded.len() < 12 {
        return Err("Encrypted payload is too short".to_string());
    }

    let (nonce_bytes, ciphertext) = encoded.split_at(12);
    let cipher = Aes256Gcm::new(&Key::<Aes256Gcm>::from(*key));
    let nonce = Nonce::try_from(nonce_bytes)
        .map_err(|_| "Failed to extract 12-byte nonce from payload slice".to_string())?;

    cipher
        .decrypt(&nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_derivation_and_encryption_roundtrip() {
        let passphrase = "correct-horse-battery-staple";

        let mut salt = [0u8; 16];
        OsRng.fill_bytes(&mut salt);

        let key = derive_key(passphrase, &salt).expect("Key derivation failed");

        let original_data = b"my super secret vault entries payload data";
        let encrypted = encrypt_payload(&key, original_data).expect("Encryption failed");

        assert!(encrypted.len() > original_data.len());

        let decrypted = decrypt_payload(&key, &encrypted).expect("Decryption failed");
        assert_eq!(decrypted, original_data);
    }
}
