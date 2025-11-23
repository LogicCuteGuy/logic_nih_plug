//! Integration tests for cryptographic utilities.
//!
//! Tests for:
//! - Base64 encoding/decoding (Requirement 21.5)
//! - Secure random number generation (Requirement 21.3)
//! - Digital signatures (Requirement 21.4)

use nih_plug_crypto::{
    base64_encode, base64_decode,
    generate_random_bytes, generate_random_u32, generate_random_u64, fill_random_bytes,
    SignatureKeyPair, verify_signature,
};
use rsa::traits::PublicKeyParts;

// ============================================================================
// Base64 Encoding/Decoding Tests (Requirement 21.5)
// ============================================================================

#[test]
fn test_base64_encode_simple() {
    let data = b"hello";
    let encoded = base64_encode(data).unwrap();
    assert_eq!(encoded, "aGVsbG8=");
}

#[test]
fn test_base64_decode_simple() {
    let encoded = "aGVsbG8=";
    let decoded = base64_decode(encoded).unwrap();
    assert_eq!(decoded, b"hello");
}

#[test]
fn test_base64_round_trip() {
    let original = b"The quick brown fox jumps over the lazy dog";
    let encoded = base64_encode(original).unwrap();
    let decoded = base64_decode(&encoded).unwrap();
    assert_eq!(decoded, original);
}

#[test]
fn test_base64_empty_data() {
    let data = b"";
    let encoded = base64_encode(data).unwrap();
    let decoded = base64_decode(&encoded).unwrap();
    assert_eq!(decoded, b"");
}

#[test]
fn test_base64_binary_data() {
    let data: Vec<u8> = (0..=255).collect();
    let encoded = base64_encode(&data).unwrap();
    let decoded = base64_decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn test_base64_invalid_input() {
    let invalid = "not valid base64!!!";
    let result = base64_decode(invalid);
    assert!(result.is_err());
}

// ============================================================================
// Secure Random Number Generation Tests (Requirement 21.3)
// ============================================================================

#[test]
fn test_generate_random_bytes_length() {
    let data = generate_random_bytes(32).unwrap();
    assert_eq!(data.len(), 32);
}

#[test]
fn test_generate_random_bytes_not_all_zeros() {
    let data = generate_random_bytes(32).unwrap();
    // Extremely unlikely to get all zeros with cryptographically secure RNG
    assert!(data.iter().any(|&b| b != 0));
}

#[test]
fn test_generate_random_bytes_different_calls() {
    let data1 = generate_random_bytes(32).unwrap();
    let data2 = generate_random_bytes(32).unwrap();
    // Extremely unlikely to get the same random data twice
    assert_ne!(data1, data2);
}

#[test]
fn test_fill_random_bytes() {
    let mut buffer = [0u8; 32];
    fill_random_bytes(&mut buffer).unwrap();
    // Check that not all bytes are zero
    assert!(buffer.iter().any(|&b| b != 0));
}

#[test]
fn test_generate_random_u32_different() {
    let val1 = generate_random_u32().unwrap();
    let val2 = generate_random_u32().unwrap();
    // Extremely unlikely to get the same value twice
    assert_ne!(val1, val2);
}

#[test]
fn test_generate_random_u64_different() {
    let val1 = generate_random_u64().unwrap();
    let val2 = generate_random_u64().unwrap();
    // Extremely unlikely to get the same value twice
    assert_ne!(val1, val2);
}

#[test]
fn test_random_bytes_zero_length() {
    let data = generate_random_bytes(0).unwrap();
    assert_eq!(data.len(), 0);
}

// ============================================================================
// Digital Signatures Tests (Requirement 21.4)
// ============================================================================

#[test]
fn test_signature_keypair_generation() {
    let keypair = SignatureKeyPair::generate(2048).unwrap();
    // Just verify it doesn't panic and returns a valid keypair
    assert!(keypair.public_key().n().bits() >= 2048);
}

#[test]
fn test_signature_sign_and_verify() {
    let keypair = SignatureKeyPair::generate(2048).unwrap();
    let data = b"message to sign";
    
    let signature = keypair.sign(data).unwrap();
    assert!(!signature.is_empty());
    
    keypair.verify(data, &signature).unwrap();
}

#[test]
fn test_signature_verify_wrong_data() {
    let keypair = SignatureKeyPair::generate(2048).unwrap();
    let data = b"original message";
    let wrong_data = b"tampered message";
    
    let signature = keypair.sign(data).unwrap();
    
    let result = keypair.verify(wrong_data, &signature);
    assert!(result.is_err());
}

#[test]
fn test_signature_verify_wrong_signature() {
    let keypair = SignatureKeyPair::generate(2048).unwrap();
    let data = b"message";
    
    let signature = keypair.sign(data).unwrap();
    let mut wrong_signature = signature.clone();
    wrong_signature[0] ^= 0xFF; // Flip bits
    
    let result = keypair.verify(data, &wrong_signature);
    assert!(result.is_err());
}

#[test]
fn test_signature_verify_with_public_key() {
    let keypair = SignatureKeyPair::generate(2048).unwrap();
    let data = b"test data";
    
    let signature = keypair.sign(data).unwrap();
    verify_signature(keypair.public_key(), data, &signature).unwrap();
}

#[test]
fn test_signature_different_messages() {
    let keypair = SignatureKeyPair::generate(2048).unwrap();
    let data1 = b"message 1";
    let data2 = b"message 2";
    
    let sig1 = keypair.sign(data1).unwrap();
    let sig2 = keypair.sign(data2).unwrap();
    
    // Different messages should produce different signatures
    assert_ne!(sig1, sig2);
    
    // Each signature should only verify its own message
    keypair.verify(data1, &sig1).unwrap();
    keypair.verify(data2, &sig2).unwrap();
    assert!(keypair.verify(data1, &sig2).is_err());
    assert!(keypair.verify(data2, &sig1).is_err());
}

#[test]
fn test_signature_empty_data() {
    let keypair = SignatureKeyPair::generate(2048).unwrap();
    let data = b"";
    
    let signature = keypair.sign(data).unwrap();
    keypair.verify(data, &signature).unwrap();
}

// ============================================================================
// Integration Tests - Combining Multiple Features
// ============================================================================

#[test]
fn test_sign_and_encode_signature() {
    let keypair = SignatureKeyPair::generate(2048).unwrap();
    let data = b"important message";
    
    // Sign the data
    let signature = keypair.sign(data).unwrap();
    
    // Encode signature as Base64 for transmission
    let encoded_sig = base64_encode(&signature).unwrap();
    
    // Decode and verify
    let decoded_sig = base64_decode(&encoded_sig).unwrap();
    keypair.verify(data, &decoded_sig).unwrap();
}

#[test]
fn test_random_data_signature() {
    let keypair = SignatureKeyPair::generate(2048).unwrap();
    
    // Generate random data
    let random_data = generate_random_bytes(256).unwrap();
    
    // Sign the random data
    let signature = keypair.sign(&random_data).unwrap();
    
    // Verify the signature
    keypair.verify(&random_data, &signature).unwrap();
}

#[test]
fn test_encode_random_data() {
    // Generate random data
    let random_data = generate_random_bytes(64).unwrap();
    
    // Encode it
    let encoded = base64_encode(&random_data).unwrap();
    
    // Decode and verify round-trip
    let decoded = base64_decode(&encoded).unwrap();
    assert_eq!(decoded, random_data);
}
