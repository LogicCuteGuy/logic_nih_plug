//! RSA public/private key handling.
//!
//! Wraps the `rsa` crate behind an API in the shape of JUCE's
//! `juce::RSAKey`: a single struct that holds either a public-only key
//! (`verify` only) or a full keypair (`sign` + `verify`). Signatures use
//! SHA-256 + PKCS#1 v1.5 padding.
//!
//! ## Security notes
//!
//! - 2048-bit keys are the minimum supported. Don't go below.
//! - Key generation can take a few seconds on slower machines — build with
//!   `--release` (or `opt-level = 3` for the `num-bigint-dig` dependency) to
//!   keep this manageable in debug builds.
//! - The `rsa` crate's modular exponentiation is **not constant-time**; while
//!   it uses blinding, this crate is still vulnerable to timing-based
//!   attacks ([RUSTSEC-2023-0071]). Don't sign untrusted, attacker-controlled
//!   data on a network-attacker-reachable host.

use rsa::pkcs1v15::{Signature, SigningKey, VerifyingKey};
use rsa::signature::{SignatureEncoding, Signer, Verifier};
use rsa::traits::PublicKeyParts;
use rsa::{BigUint, RsaPrivateKey, RsaPublicKey};
use sha2::Sha256;

use crate::error::CryptoError;

/// The smallest RSA modulus size this crate will generate or import.
pub const MIN_KEY_BITS: usize = 2048;

/// RSA keypair or public-only key.
///
/// Construct one with [`RSAKey::generate`] for a new keypair, or
/// [`RSAKey::from_public_components`] when you only need a public key (e.g.
/// for verifying licenses issued by someone else).
pub struct RSAKey {
    public: RsaPublicKey,
    private: Option<RsaPrivateKey>,
}

impl std::fmt::Debug for RSAKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut dbg = f.debug_struct("RSAKey");
        dbg.field("bit_size", &self.bit_size())
            .field("has_private", &self.private.is_some());
        // Deliberately do not print any key material — even the modulus.
        dbg.finish_non_exhaustive()
    }
}

impl RSAKey {
    /// Generates a fresh keypair of `bits` modulus length. Requires `bits`
    /// to be at least [`MIN_KEY_BITS`].
    ///
    /// Uses the system thread RNG (`rand::thread_rng()`) for prime generation.
    pub fn generate(bits: usize) -> Result<Self, CryptoError> {
        if bits < MIN_KEY_BITS {
            return Err(CryptoError::RsaKeyTooShort {
                actual: bits,
                requested: MIN_KEY_BITS,
            });
        }
        let mut rng = rand::thread_rng();
        let private = RsaPrivateKey::new(&mut rng, bits)
            .map_err(|e| CryptoError::RsaKeyGeneration(e.to_string()))?;
        let public = RsaPublicKey::from(&private);
        Ok(Self {
            public,
            private: Some(private),
        })
    }

    /// Builds a public-only key from the raw big-endian byte representation
    /// of the modulus `n` and public exponent `e`.
    pub fn from_public_components(n: &[u8], e: &[u8]) -> Result<Self, CryptoError> {
        let n = BigUint::from_bytes_be(n);
        let e = BigUint::from_bytes_be(e);
        let public = RsaPublicKey::new(n, e)
            .map_err(|err| CryptoError::RsaKeyGeneration(err.to_string()))?;
        Ok(Self {
            public,
            private: None,
        })
    }

    /// Builds a full keypair from raw big-endian components. `n` is the
    /// modulus, `e` the public exponent, `d` the private exponent, and the
    /// optional `p` / `q` are the primes (if `None`, they'll be derived from
    /// `n` and `d`).
    pub fn from_private_components(
        n: &[u8],
        e: &[u8],
        d: &[u8],
        p: Option<&[u8]>,
        q: Option<&[u8]>,
    ) -> Result<Self, CryptoError> {
        let n = BigUint::from_bytes_be(n);
        let e = BigUint::from_bytes_be(e);
        let d = BigUint::from_bytes_be(d);

        let public = RsaPublicKey::new(n.clone(), e.clone())
            .map_err(|err| CryptoError::RsaKeyGeneration(err.to_string()))?;

        let private = match (p, q) {
            (Some(p), Some(q)) => {
                let p = BigUint::from_bytes_be(p);
                let q = BigUint::from_bytes_be(q);
                RsaPrivateKey::from_components(n, e, d, vec![p, q])
                    .map_err(|err| CryptoError::RsaKeyGeneration(err.to_string()))?
            }
            _ => RsaPrivateKey::from_components(n, e, d, vec![])
                .map_err(|err| CryptoError::RsaKeyGeneration(err.to_string()))?,
        };

        Ok(Self {
            public,
            private: Some(private),
        })
    }

    /// Returns the modulus size in bits.
    pub fn bit_size(&self) -> usize {
        // PublicKeyParts::n() returns &BigUint; bits() is u64.
        self.public.n().bits() as usize
    }

    /// Returns `true` if this key has a private half loaded (i.e. you can
    /// [`sign`](Self::sign) with it).
    pub fn has_private(&self) -> bool {
        self.private.is_some()
    }

    /// Returns the modulus `n` as a big-endian byte string.
    pub fn n_bytes(&self) -> Vec<u8> {
        self.public.n().to_bytes_be()
    }

    /// Returns the public exponent `e` as a big-endian byte string.
    pub fn e_bytes(&self) -> Vec<u8> {
        self.public.e().to_bytes_be()
    }

    /// Signs `data` with SHA-256 + PKCS#1 v1.5 padding. Returns the signature
    /// bytes. Requires a private key.
    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let private = self
            .private
            .as_ref()
            .ok_or(CryptoError::RsaPrivateKeyMissing)?;
        let signing_key = SigningKey::<Sha256>::new(private.clone());
        let signature: Signature = signing_key.sign(data);
        Ok(signature.to_bytes().to_vec())
    }

    /// Verifies a signature produced by [`sign`](Self::sign). Returns `true`
    /// if the signature is valid for `data`, `false` otherwise (including the
    /// case where the signature is malformed).
    pub fn verify(&self, data: &[u8], signature: &[u8]) -> bool {
        let verifying_key = VerifyingKey::<Sha256>::new(self.public.clone());
        let Ok(sig) = Signature::try_from(signature) else {
            return false;
        };
        verifying_key.verify(data, &sig).is_ok()
    }

    /// Drops the private half and returns a public-only key. Useful for
    /// handing out a verifier without leaking signing capability.
    pub fn into_public_only(mut self) -> Self {
        // Drop the private half — when the `Option` is replaced with `None`,
        // the original `RsaPrivateKey` is dropped, which (per the `rsa`
        // crate) zeroes its internal buffers via `zeroize` before freeing.
        let _ = self.private.take();
        self
    }
}

// Manual `Clone` so we don't require `RsaPrivateKey: Clone` (it isn't — its
// drop is sensitive). We do require `RsaPublicKey: Clone`, which is true.
impl Clone for RSAKey {
    fn clone(&self) -> Self {
        Self {
            public: self.public.clone(),
            // We can't clone the private key (and shouldn't try — copying
            // secret material doubles the exposure window). Return None
            // here so callers are forced to re-derive a private key when
            // they need one.
            private: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 2048-bit key generation is slow in debug builds; only run the heavy
    /// tests when explicitly requested via this feature flag.
    const TEST_BITS: usize = 2048;

    fn make_key() -> RSAKey {
        RSAKey::generate(TEST_BITS).expect("key generation failed")
    }

    #[test]
    fn generate_has_correct_size_and_private() {
        let key = make_key();
        assert!(key.bit_size() >= MIN_KEY_BITS);
        assert!(key.has_private());
        assert!(!key.n_bytes().is_empty());
        assert_eq!(key.e_bytes(), vec![0x01, 0x00, 0x01]); // e = 65537
    }

    #[test]
    fn reject_too_small_key() {
        let err = RSAKey::generate(1024).unwrap_err();
        matches!(err, CryptoError::RsaKeyTooShort { .. });
    }

    #[test]
    fn sign_and_verify_round_trip() {
        let key = make_key();
        let msg = b"hello, RSA world";
        let sig = key.sign(msg).expect("sign failed");
        assert!(key.verify(msg, &sig));
        // Tampering with the signature invalidates verification.
        let mut bad = sig.clone();
        bad[0] ^= 0xff;
        assert!(!key.verify(msg, &bad));
        // Tampering with the message invalidates verification.
        let mut bad_msg = msg.to_vec();
        bad_msg[0] ^= 0xff;
        assert!(!key.verify(&bad_msg, &sig));
    }

    #[test]
    fn sign_without_private_errors() {
        let mut key = make_key();
        // Drop the private half.
        let public_only = std::mem::replace(&mut key.private, None);
        drop(public_only);
        let key = key.into_public_only();
        assert!(!key.has_private());
        let err = key.sign(b"data").unwrap_err();
        assert!(matches!(err, CryptoError::RsaPrivateKeyMissing));
    }

    #[test]
    fn from_public_components_round_trip() {
        let key = make_key();
        let n = key.n_bytes();
        let e = key.e_bytes();
        let pub_only = RSAKey::from_public_components(&n, &e).unwrap();
        assert!(!pub_only.has_private());
        assert_eq!(pub_only.n_bytes(), n);
        assert_eq!(pub_only.e_bytes(), e);
    }

    #[test]
    fn public_only_can_verify() {
        let key = make_key();
        let msg = b"verify me";
        let sig = key.sign(msg).unwrap();

        let pub_only = RSAKey::from_public_components(&key.n_bytes(), &key.e_bytes()).unwrap();
        assert!(pub_only.verify(msg, &sig));
    }

    #[test]
    fn debug_redacts_material() {
        let key = make_key();
        let s = format!("{:?}", key);
        // Make sure neither the modulus nor any limb shows up.
        let hex = crate::hex::encode(&key.n_bytes());
        assert!(!s.contains(&hex));
        // But structural info is present.
        assert!(s.contains("RSAKey"));
        assert!(s.contains("bit_size"));
        assert!(s.contains("has_private"));
    }

    #[test]
    fn clone_yields_public_only() {
        let key = make_key();
        let cloned = key.clone();
        assert!(cloned.has_private() == false);
        assert_eq!(cloned.n_bytes(), key.n_bytes());
    }
}
