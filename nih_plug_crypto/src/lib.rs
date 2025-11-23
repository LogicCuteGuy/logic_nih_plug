//! # nih_plug_crypto
//!
//! Cryptography utilities ported from JUCE.
//!
//! This crate provides:
//!
//! - **Hashing**: MD5, SHA-256, SHA-512 algorithms
//! - **Encryption**: RSA and Blowfish encryption
//! - **Encoding**: Base64 encoding/decoding
//! - **Random**: Cryptographically secure random number generation
//! - **Signatures**: Digital signature creation and verification
//!
//! ## Examples
//!
//! ```
//! use nih_plug_crypto::{base64_encode, generate_random_bytes};
//!
//! // Base64 encoding
//! let encoded = base64_encode(b"hello").unwrap();
//! assert_eq!(encoded, "aGVsbG8=");
//!
//! // Random number generation
//! let random_data = generate_random_bytes(32).unwrap();
//! assert_eq!(random_data.len(), 32);
//! ```

#![warn(missing_docs)]

pub mod error;

#[cfg(feature = "hashing")]
pub mod hashing;

#[cfg(feature = "encryption")]
pub mod encryption;

pub mod encoding;
pub mod random;

#[cfg(feature = "encryption")]
pub mod signatures;

pub use error::CryptoError;

#[cfg(feature = "hashing")]
pub use hashing::{Hasher, HashAlgorithm, md5, sha256, sha512};

#[cfg(feature = "encryption")]
pub use encryption::{Encryptor, EncryptionAlgorithm};

pub use encoding::{base64_encode, base64_decode};
pub use random::{fill_random_bytes, generate_random_bytes, generate_random_u32, generate_random_u64};

#[cfg(feature = "encryption")]
pub use signatures::{SignatureKeyPair, verify_signature};
