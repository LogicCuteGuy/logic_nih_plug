//! Cryptographically secure random number generation.
//!
//! This module provides utilities for generating cryptographically secure random data.

use crate::error::CryptoError;
use getrandom::getrandom;

/// Generates cryptographically secure random bytes.
///
/// Uses the operating system's secure random number generator.
///
/// # Arguments
///
/// * `buffer` - The buffer to fill with random bytes
///
/// # Errors
///
/// Returns an error if the random number generator fails.
///
/// # Examples
///
/// ```
/// use nih_plug_crypto::fill_random_bytes;
///
/// let mut buffer = [0u8; 32];
/// fill_random_bytes(&mut buffer).unwrap();
/// // buffer now contains 32 random bytes
/// ```
pub fn fill_random_bytes(buffer: &mut [u8]) -> Result<(), CryptoError> {
    getrandom(buffer).map_err(|e| CryptoError::RandomGenerationFailed(e.to_string()))
}

/// Generates a vector of cryptographically secure random bytes.
///
/// # Arguments
///
/// * `length` - The number of random bytes to generate
///
/// # Returns
///
/// A vector containing the requested number of random bytes.
///
/// # Errors
///
/// Returns an error if the random number generator fails.
///
/// # Examples
///
/// ```
/// use nih_plug_crypto::generate_random_bytes;
///
/// let random_data = generate_random_bytes(32).unwrap();
/// assert_eq!(random_data.len(), 32);
/// ```
pub fn generate_random_bytes(length: usize) -> Result<Vec<u8>, CryptoError> {
    let mut buffer = vec![0u8; length];
    fill_random_bytes(&mut buffer)?;
    Ok(buffer)
}

/// Generates a cryptographically secure random u32.
///
/// # Returns
///
/// A random u32 value.
///
/// # Errors
///
/// Returns an error if the random number generator fails.
///
/// # Examples
///
/// ```
/// use nih_plug_crypto::generate_random_u32;
///
/// let random_value = generate_random_u32().unwrap();
/// ```
pub fn generate_random_u32() -> Result<u32, CryptoError> {
    let mut buffer = [0u8; 4];
    fill_random_bytes(&mut buffer)?;
    Ok(u32::from_le_bytes(buffer))
}

/// Generates a cryptographically secure random u64.
///
/// # Returns
///
/// A random u64 value.
///
/// # Errors
///
/// Returns an error if the random number generator fails.
///
/// # Examples
///
/// ```
/// use nih_plug_crypto::generate_random_u64;
///
/// let random_value = generate_random_u64().unwrap();
/// ```
pub fn generate_random_u64() -> Result<u64, CryptoError> {
    let mut buffer = [0u8; 8];
    fill_random_bytes(&mut buffer)?;
    Ok(u64::from_le_bytes(buffer))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fill_random_bytes() {
        let mut buffer = [0u8; 32];
        fill_random_bytes(&mut buffer).unwrap();
        
        // Check that not all bytes are zero (extremely unlikely with random data)
        assert!(buffer.iter().any(|&b| b != 0));
    }

    #[test]
    fn test_generate_random_bytes() {
        let random_data = generate_random_bytes(32).unwrap();
        assert_eq!(random_data.len(), 32);
        
        // Check that not all bytes are zero
        assert!(random_data.iter().any(|&b| b != 0));
    }

    #[test]
    fn test_generate_random_u32() {
        let value1 = generate_random_u32().unwrap();
        let value2 = generate_random_u32().unwrap();
        
        // Extremely unlikely to get the same value twice
        assert_ne!(value1, value2);
    }

    #[test]
    fn test_generate_random_u64() {
        let value1 = generate_random_u64().unwrap();
        let value2 = generate_random_u64().unwrap();
        
        // Extremely unlikely to get the same value twice
        assert_ne!(value1, value2);
    }
}
