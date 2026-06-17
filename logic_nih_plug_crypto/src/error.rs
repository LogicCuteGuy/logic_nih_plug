//! Error types for the crypto crate.

use thiserror::Error;

/// Errors that can occur in [`crate::big_integer`] and [`crate::rsa_key`]
/// operations.
#[derive(Debug, Error)]
pub enum CryptoError {
    /// A big-integer string failed to parse (wrong radix, malformed digits).
    #[error("failed to parse big-integer string {input:?} in radix {radix}")]
    BigIntParse {
        /// The string we tried to parse.
        input: String,
        /// The radix (base) we tried to use.
        radix: u32,
    },

    /// Subtraction / division / modulo by a value that made the operation
    /// undefined or produced a negative result on an unsigned integer.
    #[error("big-integer arithmetic failed: {0}")]
    BigIntArithmetic(&'static str),

    /// RSA key generation failed.
    #[error("RSA key generation failed: {0}")]
    RsaKeyGeneration(String),

    /// RSA signing failed.
    #[error("RSA signing failed: {0}")]
    RsaSigning(String),

    /// An RSA operation was attempted without a private key being loaded.
    #[error("RSA private key not available (public-only key)")]
    RsaPrivateKeyMissing,

    /// An RSA operation needs the key to be at least `requested` bits long.
    #[error("RSA key is too short: {actual} bits, need at least {requested}")]
    RsaKeyTooShort {
        /// The actual key size.
        actual: usize,
        /// The minimum required key size.
        requested: usize,
    },

    /// Input slice was the wrong length for the requested operation.
    #[error("invalid input length: expected {expected}, got {actual}")]
    InvalidLength {
        /// The expected length.
        expected: usize,
        /// The actual length.
        actual: usize,
    },
}
