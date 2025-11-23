//! Error types for cryptography operations.

use thiserror::Error;

/// Errors that can occur during cryptographic operations.
#[derive(Debug, Error)]
pub enum CryptoError {
    /// Invalid key length provided.
    #[error("Invalid key length: expected {expected}, got {actual}")]
    InvalidKeyLength {
        /// Expected key length in bytes
        expected: usize,
        /// Actual key length in bytes
        actual: usize,
    },

    /// Invalid data length for encryption/decryption.
    #[error("Invalid data length: {0}")]
    InvalidDataLength(String),

    /// Encryption operation failed.
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    /// Decryption operation failed.
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    /// Invalid Base64 encoding.
    #[error("Invalid Base64 encoding: {0}")]
    InvalidBase64(String),

    /// Invalid input data.
    #[error("Invalid input data: {0}")]
    InvalidInput(String),

    /// Hashing operation failed.
    #[error("Hashing failed: {0}")]
    HashingFailed(String),

    /// Random number generation failed.
    #[error("Random number generation failed: {0}")]
    RandomGenerationFailed(String),

    /// Digital signature verification failed.
    #[error("Signature verification failed")]
    SignatureVerificationFailed,

    /// Digital signature creation failed.
    #[error("Signature creation failed: {0}")]
    SignatureCreationFailed(String),
}
