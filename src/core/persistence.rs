// src/core/persistence.rs

use crate::core::crypto::{decrypt_payload, derive_key, encrypt_payload};
use crate::core::vault::Vault;
use rand::RngCore;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EncryptedVault {
    pub salt: Vec<u8>,
    pub ciphertext: Vec<u8>,
}

pub fn encrypt_vault(vault: &Vault, passphrase: &str) -> Result<EncryptedVault, String> {
    if vault.is_locked || vault.entries().is_none() {
        return Err(
            "Cannot encrypt a locked vault. Unlock the vault with entries before saving"
                .to_string(),
        );
    }

    // generate a 16 byte salt
    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);

    // derive key
    let key = derive_key(passphrase, &salt).map_err(|e| format!("Key derivation failed: {}", e))?;

    // serialise vault into json bytes
    let json_bytes =
        serde_json::to_vec(vault).map_err(|e| format!("Vault serialisation failed: {}", e))?;

    // encrypt json bytes
    let ciphertext =
        encrypt_payload(&key, &json_bytes).map_err(|e| format!("Encryption failed: {}", e))?;

    Ok(EncryptedVault {
        salt: salt.to_vec(),
        ciphertext,
    })
}

pub fn decrypt_vault(encrypted: &EncryptedVault, passphrase: &str) -> Result<Vault, String> {
    if encrypted.salt.len() != 16 {
        return Err("Invalid salt length: expected 16 bytes".to_string());
    }

    let mut salt_arr = [0u8; 16];
    salt_arr.copy_from_slice(&encrypted.salt);

    // re-derive key
    let key =
        derive_key(passphrase, &salt_arr).map_err(|e| format!("Key derivation failed: {}", e))?;

    // decrypt payload into json bytes
    let json_bytes = decrypt_payload(&key, &encrypted.ciphertext)?;

    // deserialise json bytes into Vault
    let vault: Vault = serde_json::from_slice(&json_bytes)
        .map_err(|e| format!("Vault deserialisation failed: {}", e))?;

    Ok(vault)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persistence_roundtrip() {
        // create vault
        let mut vault = Vault::new("vault-uuid-001", "Main Credentials");
        vault.unlock(vec![]);

        let passphrase = "correct-horse-battery-staple";
        let encrypted = encrypt_vault(&vault, passphrase).expect("Failed to encrypt vault");
        let decrypted = decrypt_vault(&encrypted, passphrase).expect("Failed to decrypt vault");

        assert_eq!(vault.id, decrypted.id);
        assert_eq!(vault.name, decrypted.name);

        let bad_decrypt = decrypt_vault(&encrypted, "bad-password");
        assert!(bad_decrypt.is_err());
    }

    #[test]
    fn test_cannot_encrypt_locked_vault() {
        let vault = Vault::new("vault-uuid-002", "Locked Vault");
        let result = encrypt_vault(&vault, "some-password");
        assert!(result.is_err());
    }
}
