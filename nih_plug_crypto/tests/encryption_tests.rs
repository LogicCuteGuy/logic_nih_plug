//! Tests for encryption algorithms.

#[cfg(feature = "encryption")]
mod encryption_tests {
    use nih_plug_crypto::{Encryptor, EncryptionAlgorithm};

    #[test]
    fn test_rsa_encrypt_decrypt() {
        let encryptor = Encryptor::new(EncryptionAlgorithm::RSA);
        let plaintext = b"Hello, RSA encryption!";
        
        // Encrypt
        let ciphertext = encryptor.encrypt(plaintext, &[]).expect("Encryption should succeed");
        
        // Verify ciphertext is different from plaintext
        assert_ne!(ciphertext.as_slice(), plaintext);
        
        // Decrypt
        let decrypted = encryptor.decrypt(&ciphertext, &[]).expect("Decryption should succeed");
        
        // Verify decrypted matches original
        assert_eq!(decrypted.as_slice(), plaintext);
    }

    #[test]
    fn test_blowfish_encrypt_decrypt() {
        let encryptor = Encryptor::new(EncryptionAlgorithm::Blowfish);
        let key = b"my_secret_key_16"; // 16 bytes
        
        // Blowfish requires data to be a multiple of 8 bytes
        let plaintext = b"12345678"; // 8 bytes
        
        // Encrypt
        let ciphertext = encryptor.encrypt(plaintext, key).expect("Encryption should succeed");
        
        // Verify ciphertext is different from plaintext
        assert_ne!(ciphertext.as_slice(), plaintext);
        
        // Decrypt
        let decrypted = encryptor.decrypt(&ciphertext, key).expect("Decryption should succeed");
        
        // Verify decrypted matches original
        assert_eq!(decrypted.as_slice(), plaintext);
    }

    #[test]
    fn test_blowfish_multiple_blocks() {
        let encryptor = Encryptor::new(EncryptionAlgorithm::Blowfish);
        let key = b"test_key_1234567"; // 16 bytes
        
        // 24 bytes = 3 blocks of 8 bytes
        let plaintext = b"123456789012345678901234";
        
        // Encrypt
        let ciphertext = encryptor.encrypt(plaintext, key).expect("Encryption should succeed");
        
        // Verify ciphertext is different from plaintext
        assert_ne!(ciphertext.as_slice(), plaintext);
        
        // Decrypt
        let decrypted = encryptor.decrypt(&ciphertext, key).expect("Decryption should succeed");
        
        // Verify decrypted matches original
        assert_eq!(decrypted.as_slice(), plaintext);
    }

    #[test]
    fn test_blowfish_invalid_key_length() {
        let encryptor = Encryptor::new(EncryptionAlgorithm::Blowfish);
        let short_key = b"abc"; // Too short (< 4 bytes)
        let plaintext = b"12345678";
        
        // Should fail with invalid key length
        let result = encryptor.encrypt(plaintext, short_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_blowfish_invalid_data_length() {
        let encryptor = Encryptor::new(EncryptionAlgorithm::Blowfish);
        let key = b"valid_key_here";
        let plaintext = b"1234567"; // 7 bytes, not a multiple of 8
        
        // Should fail with invalid data length
        let result = encryptor.encrypt(plaintext, key);
        assert!(result.is_err());
    }

    #[test]
    fn test_blowfish_different_keys_produce_different_ciphertext() {
        let encryptor = Encryptor::new(EncryptionAlgorithm::Blowfish);
        let key1 = b"key_number_one!!";
        let key2 = b"key_number_two!!";
        let plaintext = b"12345678";
        
        let ciphertext1 = encryptor.encrypt(plaintext, key1).expect("Encryption should succeed");
        let ciphertext2 = encryptor.encrypt(plaintext, key2).expect("Encryption should succeed");
        
        // Different keys should produce different ciphertext
        assert_ne!(ciphertext1, ciphertext2);
    }

    #[test]
    fn test_rsa_different_instances_different_keys() {
        let encryptor1 = Encryptor::new(EncryptionAlgorithm::RSA);
        let encryptor2 = Encryptor::new(EncryptionAlgorithm::RSA);
        let plaintext = b"Test message";
        
        let ciphertext1 = encryptor1.encrypt(plaintext, &[]).expect("Encryption should succeed");
        let ciphertext2 = encryptor2.encrypt(plaintext, &[]).expect("Encryption should succeed");
        
        // Different RSA key pairs should produce different ciphertext
        assert_ne!(ciphertext1, ciphertext2);
        
        // Each encryptor should only be able to decrypt its own ciphertext
        let decrypted1 = encryptor1.decrypt(&ciphertext1, &[]).expect("Decryption should succeed");
        assert_eq!(decrypted1.as_slice(), plaintext);
        
        // Attempting to decrypt with wrong key should fail
        let result = encryptor1.decrypt(&ciphertext2, &[]);
        assert!(result.is_err());
    }
}
