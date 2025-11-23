//! Encoding utilities.
//!
//! This module provides Base64 encoding and decoding functionality.

use crate::error::CryptoError;
use base64::{Engine as _, engine::general_purpose};

/// Encodes data to Base64 format.
///
/// Uses the standard Base64 alphabet with padding.
///
/// # Arguments
///
/// * `data` - The data to encode
///
/// # Returns
///
/// The Base64-encoded string.
///
/// # Examples
///
/// ```
/// use nih_plug_crypto::base64_encode;
///
/// let encoded = base64_encode(b"hello").unwrap();
/// assert_eq!(encoded, "aGVsbG8=");
/// ```
pub fn base64_encode(data: &[u8]) -> Result<String, CryptoError> {
    Ok(general_purpose::STANDARD.encode(data))
}

/// Decodes data from Base64 format.
///
/// Uses the standard Base64 alphabet with padding.
///
/// # Arguments
///
/// * `encoded` - The Base64-encoded string
///
/// # Returns
///
/// The decoded data as a vector of bytes.
///
/// # Errors
///
/// Returns an error if the input is not valid Base64.
///
/// # Examples
///
/// ```
/// use nih_plug_crypto::base64_decode;
///
/// let decoded = base64_decode("aGVsbG8=").unwrap();
/// assert_eq!(decoded, b"hello");
/// ```
pub fn base64_decode(encoded: &str) -> Result<Vec<u8>, CryptoError> {
    general_purpose::STANDARD
        .decode(encoded)
        .map_err(|e| CryptoError::InvalidBase64(e.to_string()))
}
