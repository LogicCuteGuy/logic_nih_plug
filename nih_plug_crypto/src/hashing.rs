//! Hashing algorithms.
//!
//! This module provides implementations of various cryptographic hash functions
//! including MD5, SHA-256, and SHA-512.
//!
//! # Examples
//!
//! ```
//! use nih_plug_crypto::{Hasher, HashAlgorithm};
//!
//! let hasher = Hasher::new(HashAlgorithm::SHA256);
//! let hash = hasher.hash(b"Hello, world!").unwrap();
//! assert_eq!(hash.len(), 32); // SHA-256 produces 32 bytes
//! ```

use crate::error::CryptoError;
use md5::Md5;
use sha2::{Digest, Sha256, Sha512};

/// Supported hash algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgorithm {
    /// MD5 hash algorithm (128-bit).
    /// 
    /// **Warning**: MD5 is cryptographically broken and should not be used for security purposes.
    /// It is provided for compatibility with legacy systems only.
    MD5,
    /// SHA-256 hash algorithm (256-bit).
    SHA256,
    /// SHA-512 hash algorithm (512-bit).
    SHA512,
}

impl HashAlgorithm {
    /// Returns the output size in bytes for this hash algorithm.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_crypto::HashAlgorithm;
    ///
    /// assert_eq!(HashAlgorithm::MD5.output_size(), 16);
    /// assert_eq!(HashAlgorithm::SHA256.output_size(), 32);
    /// assert_eq!(HashAlgorithm::SHA512.output_size(), 64);
    /// ```
    pub fn output_size(&self) -> usize {
        match self {
            HashAlgorithm::MD5 => 16,
            HashAlgorithm::SHA256 => 32,
            HashAlgorithm::SHA512 => 64,
        }
    }
}

/// A hasher for computing cryptographic hashes.
///
/// This type provides a unified interface for computing hashes using different algorithms.
///
/// # Examples
///
/// ```
/// use nih_plug_crypto::{Hasher, HashAlgorithm};
///
/// let hasher = Hasher::new(HashAlgorithm::SHA256);
/// let hash = hasher.hash(b"Hello, world!").unwrap();
/// ```
pub struct Hasher {
    algorithm: HashAlgorithm,
}

impl Hasher {
    /// Creates a new hasher with the specified algorithm.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_crypto::{Hasher, HashAlgorithm};
    ///
    /// let hasher = Hasher::new(HashAlgorithm::SHA256);
    /// ```
    pub fn new(algorithm: HashAlgorithm) -> Self {
        Self { algorithm }
    }

    /// Computes the hash of the input data.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to hash
    ///
    /// # Returns
    ///
    /// The hash as a vector of bytes. The length depends on the algorithm:
    /// - MD5: 16 bytes
    /// - SHA-256: 32 bytes
    /// - SHA-512: 64 bytes
    ///
    /// # Errors
    ///
    /// This method currently does not return errors, but the Result type is used
    /// for future compatibility.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_crypto::{Hasher, HashAlgorithm};
    ///
    /// let hasher = Hasher::new(HashAlgorithm::SHA256);
    /// let hash = hasher.hash(b"Hello, world!").unwrap();
    /// assert_eq!(hash.len(), 32);
    /// ```
    pub fn hash(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        match self.algorithm {
            HashAlgorithm::MD5 => {
                let mut hasher = Md5::new();
                hasher.update(data);
                Ok(hasher.finalize().to_vec())
            }
            HashAlgorithm::SHA256 => {
                let mut hasher = Sha256::new();
                hasher.update(data);
                Ok(hasher.finalize().to_vec())
            }
            HashAlgorithm::SHA512 => {
                let mut hasher = Sha512::new();
                hasher.update(data);
                Ok(hasher.finalize().to_vec())
            }
        }
    }

    /// Returns the algorithm used by this hasher.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_crypto::{Hasher, HashAlgorithm};
    ///
    /// let hasher = Hasher::new(HashAlgorithm::SHA256);
    /// assert_eq!(hasher.algorithm(), HashAlgorithm::SHA256);
    /// ```
    pub fn algorithm(&self) -> HashAlgorithm {
        self.algorithm
    }
}

/// Convenience function to compute an MD5 hash.
///
/// **Warning**: MD5 is cryptographically broken and should not be used for security purposes.
///
/// # Examples
///
/// ```
/// use nih_plug_crypto::hashing::md5;
///
/// let hash = md5(b"Hello, world!").unwrap();
/// assert_eq!(hash.len(), 16);
/// ```
pub fn md5(data: &[u8]) -> Result<Vec<u8>, CryptoError> {
    Hasher::new(HashAlgorithm::MD5).hash(data)
}

/// Convenience function to compute a SHA-256 hash.
///
/// # Examples
///
/// ```
/// use nih_plug_crypto::hashing::sha256;
///
/// let hash = sha256(b"Hello, world!").unwrap();
/// assert_eq!(hash.len(), 32);
/// ```
pub fn sha256(data: &[u8]) -> Result<Vec<u8>, CryptoError> {
    Hasher::new(HashAlgorithm::SHA256).hash(data)
}

/// Convenience function to compute a SHA-512 hash.
///
/// # Examples
///
/// ```
/// use nih_plug_crypto::hashing::sha512;
///
/// let hash = sha512(b"Hello, world!").unwrap();
/// assert_eq!(hash.len(), 64);
/// ```
pub fn sha512(data: &[u8]) -> Result<Vec<u8>, CryptoError> {
    Hasher::new(HashAlgorithm::SHA512).hash(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_md5_basic() {
        let hasher = Hasher::new(HashAlgorithm::MD5);
        let hash = hasher.hash(b"").unwrap();
        assert_eq!(hash.len(), 16);
        
        // Known MD5 hash of empty string
        let expected = hex::decode("d41d8cd98f00b204e9800998ecf8427e").unwrap();
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_md5_hello_world() {
        let hasher = Hasher::new(HashAlgorithm::MD5);
        let hash = hasher.hash(b"Hello, world!").unwrap();
        
        // Known MD5 hash of "Hello, world!"
        let expected = hex::decode("6cd3556deb0da54bca060b4c39479839").unwrap();
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_sha256_basic() {
        let hasher = Hasher::new(HashAlgorithm::SHA256);
        let hash = hasher.hash(b"").unwrap();
        assert_eq!(hash.len(), 32);
        
        // Known SHA-256 hash of empty string
        let expected = hex::decode("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855").unwrap();
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_sha256_hello_world() {
        let hasher = Hasher::new(HashAlgorithm::SHA256);
        let hash = hasher.hash(b"Hello, world!").unwrap();
        
        // Known SHA-256 hash of "Hello, world!"
        let expected = hex::decode("315f5bdb76d078c43b8ac0064e4a0164612b1fce77c869345bfc94c75894edd3").unwrap();
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_sha512_basic() {
        let hasher = Hasher::new(HashAlgorithm::SHA512);
        let hash = hasher.hash(b"").unwrap();
        assert_eq!(hash.len(), 64);
        
        // Known SHA-512 hash of empty string
        let expected = hex::decode("cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e").unwrap();
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_sha512_hello_world() {
        let hasher = Hasher::new(HashAlgorithm::SHA512);
        let hash = hasher.hash(b"Hello, world!").unwrap();
        
        // Known SHA-512 hash of "Hello, world!"
        let expected = hex::decode("c1527cd893c124773d811911970c8fe6e857d6df5dc9226bd8a160614c0cd963a4ddea2b94bb7d36021ef9d865d5cea294a82dd49a0bb269f51f6e7a57f79421").unwrap();
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_convenience_functions() {
        let data = b"test data";
        
        let md5_hash = md5(data).unwrap();
        assert_eq!(md5_hash.len(), 16);
        
        let sha256_hash = sha256(data).unwrap();
        assert_eq!(sha256_hash.len(), 32);
        
        let sha512_hash = sha512(data).unwrap();
        assert_eq!(sha512_hash.len(), 64);
    }

    #[test]
    fn test_algorithm_output_size() {
        assert_eq!(HashAlgorithm::MD5.output_size(), 16);
        assert_eq!(HashAlgorithm::SHA256.output_size(), 32);
        assert_eq!(HashAlgorithm::SHA512.output_size(), 64);
    }

    #[test]
    fn test_hasher_algorithm() {
        let hasher = Hasher::new(HashAlgorithm::SHA256);
        assert_eq!(hasher.algorithm(), HashAlgorithm::SHA256);
    }
}
