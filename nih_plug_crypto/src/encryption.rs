//! Encryption algorithms.
//!
//! This module provides implementations of various encryption algorithms
//! including RSA and Blowfish.

use crate::error::CryptoError;

#[cfg(feature = "encryption")]
use cipher::{BlockDecrypt, BlockEncrypt, KeyInit};
#[cfg(feature = "encryption")]
use rand::rngs::OsRng;
#[cfg(feature = "encryption")]
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
#[cfg(feature = "encryption")]
use blowfish::Blowfish;

/// Supported encryption algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncryptionAlgorithm {
    /// RSA encryption algorithm.
    RSA,
    /// Blowfish encryption algorithm.
    Blowfish,
}

/// An encryptor for encrypting and decrypting data.
///
/// This type provides encryption and decryption using RSA or Blowfish algorithms.
pub struct Encryptor {
    algorithm: EncryptionAlgorithm,
    #[cfg(feature = "encryption")]
    rsa_private_key: Option<RsaPrivateKey>,
    #[cfg(feature = "encryption")]
    rsa_public_key: Option<RsaPublicKey>,
}

impl Encryptor {
    /// Creates a new encryptor with the specified algorithm.
    ///
    /// For RSA encryption, this generates a new 2048-bit key pair.
    /// For Blowfish, keys are provided during encrypt/decrypt operations.
    ///
    /// # Arguments
    ///
    /// * `algorithm` - The encryption algorithm to use
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_crypto::{Encryptor, EncryptionAlgorithm};
    ///
    /// let encryptor = Encryptor::new(EncryptionAlgorithm::Blowfish);
    /// ```
    #[cfg(feature = "encryption")]
    pub fn new(algorithm: EncryptionAlgorithm) -> Self {
        let (rsa_private_key, rsa_public_key) = if algorithm == EncryptionAlgorithm::RSA {
            // Generate RSA key pair
            let mut rng = OsRng;
            let bits = 2048;
            match RsaPrivateKey::new(&mut rng, bits) {
                Ok(private_key) => {
                    let public_key = RsaPublicKey::from(&private_key);
                    (Some(private_key), Some(public_key))
                }
                Err(_) => (None, None),
            }
        } else {
            (None, None)
        };

        Self {
            algorithm,
            rsa_private_key,
            rsa_public_key,
        }
    }

    /// Creates a new encryptor with the specified algorithm.
    ///
    /// This is a stub implementation when the encryption feature is not enabled.
    #[cfg(not(feature = "encryption"))]
    pub fn new(algorithm: EncryptionAlgorithm) -> Self {
        Self { algorithm }
    }

    /// Creates a new RSA encryptor with provided keys.
    ///
    /// # Arguments
    ///
    /// * `private_key` - The RSA private key for decryption
    /// * `public_key` - The RSA public key for encryption
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use nih_plug_crypto::Encryptor;
    /// use rsa::{RsaPrivateKey, RsaPublicKey};
    ///
    /// let private_key = RsaPrivateKey::new(&mut rng, 2048).unwrap();
    /// let public_key = RsaPublicKey::from(&private_key);
    /// let encryptor = Encryptor::with_rsa_keys(private_key, public_key);
    /// ```
    #[cfg(feature = "encryption")]
    pub fn with_rsa_keys(private_key: RsaPrivateKey, public_key: RsaPublicKey) -> Self {
        Self {
            algorithm: EncryptionAlgorithm::RSA,
            rsa_private_key: Some(private_key),
            rsa_public_key: Some(public_key),
        }
    }

    /// Encrypts the input data.
    ///
    /// For RSA: The key parameter is ignored, and the public key is used.
    /// For Blowfish: The key parameter must be between 4 and 56 bytes.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to encrypt
    /// * `key` - The encryption key (used for Blowfish, ignored for RSA)
    ///
    /// # Returns
    ///
    /// The encrypted data as a vector of bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if encryption fails.
    #[cfg(feature = "encryption")]
    pub fn encrypt(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>, CryptoError> {
        match self.algorithm {
            EncryptionAlgorithm::RSA => self.encrypt_rsa(data),
            EncryptionAlgorithm::Blowfish => self.encrypt_blowfish(data, key),
        }
    }

    /// Encrypts the input data (stub when encryption feature is disabled).
    #[cfg(not(feature = "encryption"))]
    pub fn encrypt(&self, _data: &[u8], _key: &[u8]) -> Result<Vec<u8>, CryptoError> {
        Err(CryptoError::EncryptionFailed(
            "Encryption feature not enabled".to_string(),
        ))
    }

    #[cfg(feature = "encryption")]
    fn encrypt_rsa(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let public_key = self.rsa_public_key.as_ref().ok_or_else(|| {
            CryptoError::EncryptionFailed("RSA public key not available".to_string())
        })?;

        let mut rng = OsRng;
        public_key
            .encrypt(&mut rng, Pkcs1v15Encrypt, data)
            .map_err(|e| CryptoError::EncryptionFailed(format!("RSA encryption failed: {}", e)))
    }

    #[cfg(feature = "encryption")]
    fn encrypt_blowfish(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>, CryptoError> {
        use cipher::generic_array::GenericArray;
        use cipher::typenum::U8;
        
        // Blowfish key must be between 4 and 56 bytes
        if key.len() < 4 || key.len() > 56 {
            return Err(CryptoError::InvalidKeyLength {
                expected: 56,
                actual: key.len(),
            });
        }

        // Blowfish operates on 8-byte blocks
        if data.len() % 8 != 0 {
            return Err(CryptoError::InvalidDataLength(
                "Data length must be a multiple of 8 bytes for Blowfish".to_string(),
            ));
        }

        let cipher = <Blowfish as KeyInit>::new_from_slice(key).map_err(|e| {
            CryptoError::EncryptionFailed(format!("Failed to create Blowfish cipher: {}", e))
        })?;

        let mut result = data.to_vec();
        
        // Process each 8-byte block
        for chunk in result.chunks_exact_mut(8) {
            let block = GenericArray::<u8, U8>::from_mut_slice(chunk);
            cipher.encrypt_block(block);
        }

        Ok(result)
    }

    /// Decrypts the input data.
    ///
    /// For RSA: The key parameter is ignored, and the private key is used.
    /// For Blowfish: The key parameter must be between 4 and 56 bytes.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to decrypt
    /// * `key` - The decryption key (used for Blowfish, ignored for RSA)
    ///
    /// # Returns
    ///
    /// The decrypted data as a vector of bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if decryption fails.
    #[cfg(feature = "encryption")]
    pub fn decrypt(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>, CryptoError> {
        match self.algorithm {
            EncryptionAlgorithm::RSA => self.decrypt_rsa(data),
            EncryptionAlgorithm::Blowfish => self.decrypt_blowfish(data, key),
        }
    }

    /// Decrypts the input data (stub when encryption feature is disabled).
    #[cfg(not(feature = "encryption"))]
    pub fn decrypt(&self, _data: &[u8], _key: &[u8]) -> Result<Vec<u8>, CryptoError> {
        Err(CryptoError::DecryptionFailed(
            "Encryption feature not enabled".to_string(),
        ))
    }

    #[cfg(feature = "encryption")]
    fn decrypt_rsa(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let private_key = self.rsa_private_key.as_ref().ok_or_else(|| {
            CryptoError::DecryptionFailed("RSA private key not available".to_string())
        })?;

        private_key
            .decrypt(Pkcs1v15Encrypt, data)
            .map_err(|e| CryptoError::DecryptionFailed(format!("RSA decryption failed: {}", e)))
    }

    #[cfg(feature = "encryption")]
    fn decrypt_blowfish(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>, CryptoError> {
        use cipher::generic_array::GenericArray;
        use cipher::typenum::U8;
        
        // Blowfish key must be between 4 and 56 bytes
        if key.len() < 4 || key.len() > 56 {
            return Err(CryptoError::InvalidKeyLength {
                expected: 56,
                actual: key.len(),
            });
        }

        // Blowfish operates on 8-byte blocks
        if data.len() % 8 != 0 {
            return Err(CryptoError::InvalidDataLength(
                "Data length must be a multiple of 8 bytes for Blowfish".to_string(),
            ));
        }

        let cipher = <Blowfish as KeyInit>::new_from_slice(key).map_err(|e| {
            CryptoError::DecryptionFailed(format!("Failed to create Blowfish cipher: {}", e))
        })?;

        let mut result = data.to_vec();
        
        // Process each 8-byte block
        for chunk in result.chunks_exact_mut(8) {
            let block = GenericArray::<u8, U8>::from_mut_slice(chunk);
            cipher.decrypt_block(block);
        }

        Ok(result)
    }

    /// Returns the algorithm used by this encryptor.
    pub fn algorithm(&self) -> EncryptionAlgorithm {
        self.algorithm
    }
}
