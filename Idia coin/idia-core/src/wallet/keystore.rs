//! Secure key storage implementation

use super::*;
use crate::crypto::StealthAddress;
use std::fs;
use std::io::{Read, Write};
use rand::rngs::OsRng;
use sha2::{Sha256, Digest};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};

/// Key store for managing wallet keys
pub struct KeyStore {
    /// Directory for key storage
    data_dir: PathBuf,
    /// Main stealth address
    stealth_address: StealthAddress,
    /// Encryption key for stored data
    encryption_key: [u8; 32],
}

impl KeyStore {
    /// Create a new key store
    pub fn new(data_dir: &PathBuf) -> Result<Self, WalletError> {
        fs::create_dir_all(data_dir)
            .map_err(|e| WalletError::KeyStoreError(e.to_string()))?;

        let key_file = data_dir.join("wallet.key");
        
        let (stealth_address, encryption_key) = if key_file.exists() {
            // Load existing keys
            Self::load_keys(&key_file)?
        } else {
            // Generate new keys
            let stealth_address = StealthAddress::new();
            let mut encryption_key = [0u8; 32];
            OsRng.fill_bytes(&mut encryption_key);
            
            // Save keys
            Self::save_keys(&key_file, &stealth_address, &encryption_key)?;
            
            (stealth_address, encryption_key)
        };

        Ok(Self {
            data_dir: data_dir.to_owned(),
            stealth_address,
            encryption_key,
        })
    }

    /// Load keys from file
    fn load_keys(path: &PathBuf) -> Result<(StealthAddress, [u8; 32]), WalletError> {
        let mut file = fs::File::open(path)
            .map_err(|e| WalletError::KeyStoreError(e.to_string()))?;
            
        let mut encrypted = Vec::new();
        file.read_to_end(&mut encrypted)
            .map_err(|e| WalletError::KeyStoreError(e.to_string()))?;

        // TODO: Implement proper key derivation from password
        let password = b"example_password";
        let mut key = [0u8; 32];
        key.copy_from_slice(&Sha256::digest(password));

        let cipher = Aes256Gcm::new(key.as_slice().into());
        let nonce = Nonce::from_slice(&encrypted[..12]);
        let data = cipher
            .decrypt(nonce, &encrypted[12..])
            .map_err(|e| WalletError::KeyStoreError(e.to_string()))?;

        let (stealth_address, encryption_key): (StealthAddress, [u8; 32]) = 
            bincode::deserialize(&data)
                .map_err(|e| WalletError::KeyStoreError(e.to_string()))?;

        Ok((stealth_address, encryption_key))
    }

    /// Save keys to file
    fn save_keys(
        path: &PathBuf,
        stealth_address: &StealthAddress,
        encryption_key: &[u8; 32],
    ) -> Result<(), WalletError> {
        let data = bincode::serialize(&(stealth_address, encryption_key))
            .map_err(|e| WalletError::KeyStoreError(e.to_string()))?;

        // TODO: Implement proper key derivation from password
        let password = b"example_password";
        let mut key = [0u8; 32];
        key.copy_from_slice(&Sha256::digest(password));

        let cipher = Aes256Gcm::new(key.as_slice().into());
        let nonce = Nonce::from_slice(&Sha256::digest(&encryption_key)[..12]);
        let encrypted = cipher
            .encrypt(nonce, data.as_slice())
            .map_err(|e| WalletError::KeyStoreError(e.to_string()))?;

        let mut file = fs::File::create(path)
            .map_err(|e| WalletError::KeyStoreError(e.to_string()))?;
        
        file.write_all(nonce)
            .and_then(|_| file.write_all(&encrypted))
            .map_err(|e| WalletError::KeyStoreError(e.to_string()))?;

        Ok(())
    }

    /// Get the wallet's stealth address
    pub fn get_stealth_address(&self) -> Result<StealthAddress, WalletError> {
        Ok(self.stealth_address.clone())
    }

    /// Encrypt data for storage
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, WalletError> {
        let cipher = Aes256Gcm::new(self.encryption_key.as_slice().into());
        let nonce = Nonce::from_slice(&Sha256::digest(data)[..12]);
        
        cipher
            .encrypt(nonce, data)
            .map_err(|e| WalletError::KeyStoreError(e.to_string()))
    }

    /// Decrypt stored data
    pub fn decrypt(&self, encrypted: &[u8]) -> Result<Vec<u8>, WalletError> {
        let cipher = Aes256Gcm::new(self.encryption_key.as_slice().into());
        let nonce = Nonce::from_slice(&encrypted[..12]);
        
        cipher
            .decrypt(nonce, &encrypted[12..])
            .map_err(|e| WalletError::KeyStoreError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_keystore_creation() {
        let dir = tempdir().unwrap();
        let keystore = KeyStore::new(&dir.path().to_path_buf()).unwrap();
        
        // Check that we can get the stealth address
        let addr = keystore.get_stealth_address().unwrap();
        assert!(addr.view_key.view_public.compress().as_bytes().len() == 32);
    }

    #[test]
    fn test_keystore_encryption() {
        let dir = tempdir().unwrap();
        let keystore = KeyStore::new(&dir.path().to_path_buf()).unwrap();
        
        let data = b"test data";
        let encrypted = keystore.encrypt(data).unwrap();
        let decrypted = keystore.decrypt(&encrypted).unwrap();
        
        assert_eq!(data.as_slice(), decrypted.as_slice());
    }
}