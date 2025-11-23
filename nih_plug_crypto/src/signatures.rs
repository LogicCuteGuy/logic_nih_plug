//! Digital signature creation and verification.
//!
//! This module provides utilities for creating and verifying digital signatures
//! using RSA with SHA-256.

use crate::error::CryptoError;
use rsa::{RsaPrivateKey, RsaPublicKey};
use rsa::pkcs1v15::{SigningKey, VerifyingKey, Signature};
use rsa::signature::{RandomizedSigner, SignatureEncoding, Verifier};
use sha2::Sha256;
use rand::rngs::OsRng;

/// A key pair for digital signatures.
///
/// Contains both a private key (for signing) and a public key (for verification).
pub struct SignatureKeyPair {
    private_key: RsaPrivateKey,
    public_key: RsaPublicKey,
}

impl SignatureKeyPair {
    /// Generates a new RSA key pair for digital signatures.
    ///
    /// # Arguments
    ///
    /// * `bits` - The key size in bits (typically 2048 or 4096)
    ///
    /// # Returns
    ///
    /// A new key pair.
    ///
    /// # Errors
    ///
    /// Returns an error if key generation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_crypto::SignatureKeyPair;
    ///
    /// let keypair = SignatureKeyPair::generate(2048).unwrap();
    /// ```
    pub fn generate(bits: usize) -> Result<Self, CryptoError> {
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, bits)
            .map_err(|e| CryptoError::SignatureCreationFailed(e.to_string()))?;
        let public_key = private_key.to_public_key();

        Ok(Self {
            private_key,
            public_key,
        })
    }

    /// Gets a reference to the public key.
    ///
    /// # Returns
    ///
    /// A reference to the public key.
    pub fn public_key(&self) -> &RsaPublicKey {
        &self.public_key
    }

    /// Signs data using the private key.
    ///
    /// Uses RSA with SHA-256 for signing.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to sign
    ///
    /// # Returns
    ///
    /// The signature as a vector of bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if signing fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_crypto::SignatureKeyPair;
    ///
    /// let keypair = SignatureKeyPair::generate(2048).unwrap();
    /// let signature = keypair.sign(b"hello world").unwrap();
    /// ```
    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let signing_key = SigningKey::<Sha256>::new(self.private_key.clone());
        let mut rng = OsRng;
        let signature = signing_key
            .sign_with_rng(&mut rng, data)
            .to_bytes()
            .to_vec();
        Ok(signature)
    }

    /// Verifies a signature using the public key.
    ///
    /// # Arguments
    ///
    /// * `data` - The original data that was signed
    /// * `signature` - The signature to verify
    ///
    /// # Returns
    ///
    /// `Ok(())` if the signature is valid, otherwise an error.
    ///
    /// # Errors
    ///
    /// Returns an error if verification fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_crypto::SignatureKeyPair;
    ///
    /// let keypair = SignatureKeyPair::generate(2048).unwrap();
    /// let signature = keypair.sign(b"hello world").unwrap();
    /// keypair.verify(b"hello world", &signature).unwrap();
    /// ```
    pub fn verify(&self, data: &[u8], signature: &[u8]) -> Result<(), CryptoError> {
        verify_signature(&self.public_key, data, signature)
    }
}

/// Verifies a signature using a public key.
///
/// # Arguments
///
/// * `public_key` - The public key to use for verification
/// * `data` - The original data that was signed
/// * `signature` - The signature to verify
///
/// # Returns
///
/// `Ok(())` if the signature is valid, otherwise an error.
///
/// # Errors
///
/// Returns an error if verification fails.
///
/// # Examples
///
/// ```
/// use nih_plug_crypto::{SignatureKeyPair, verify_signature};
///
/// let keypair = SignatureKeyPair::generate(2048).unwrap();
/// let signature = keypair.sign(b"hello world").unwrap();
/// verify_signature(keypair.public_key(), b"hello world", &signature).unwrap();
/// ```
pub fn verify_signature(
    public_key: &RsaPublicKey,
    data: &[u8],
    signature: &[u8],
) -> Result<(), CryptoError> {
    let verifying_key = VerifyingKey::<Sha256>::new(public_key.clone());
    
    let signature = Signature::try_from(signature)
        .map_err(|_| CryptoError::SignatureVerificationFailed)?;
    
    verifying_key
        .verify(data, &signature)
        .map_err(|_| CryptoError::SignatureVerificationFailed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rsa::traits::PublicKeyParts;

    #[test]
    fn test_generate_keypair() {
        let keypair = SignatureKeyPair::generate(2048).unwrap();
        assert!(keypair.public_key().size() > 0);
    }

    #[test]
    fn test_sign_and_verify() {
        let keypair = SignatureKeyPair::generate(2048).unwrap();
        let data = b"hello world";
        
        let signature = keypair.sign(data).unwrap();
        assert!(!signature.is_empty());
        
        keypair.verify(data, &signature).unwrap();
    }

    #[test]
    fn test_verify_with_wrong_data() {
        let keypair = SignatureKeyPair::generate(2048).unwrap();
        let data = b"hello world";
        let wrong_data = b"goodbye world";
        
        let signature = keypair.sign(data).unwrap();
        
        let result = keypair.verify(wrong_data, &signature);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_with_wrong_signature() {
        let keypair = SignatureKeyPair::generate(2048).unwrap();
        let data = b"hello world";
        
        let signature = keypair.sign(data).unwrap();
        let mut wrong_signature = signature.clone();
        wrong_signature[0] ^= 0xFF; // Flip bits in first byte
        
        let result = keypair.verify(data, &wrong_signature);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_signature_function() {
        let keypair = SignatureKeyPair::generate(2048).unwrap();
        let data = b"test data";
        
        let signature = keypair.sign(data).unwrap();
        verify_signature(keypair.public_key(), data, &signature).unwrap();
    }
}
