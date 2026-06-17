//! SHA-256 streaming hash.
//!
//! Provides a streaming [`Sha256`] context that mirrors the shape of JUCE's
//! `juce::SHA256` class, plus a one-shot [`sha256`] helper and a
//! [`sha256_hex`] convenience that returns the digest as a lowercase hex string.
//!
//! All functions and methods in this module are deterministic and side-effect
//! free, so they're safe to call from any thread.

use sha2::Digest as _;

/// The length, in bytes, of a SHA-256 digest.
pub const SHA256_OUTPUT_LEN: usize = 32;

/// Streaming SHA-256 hash context.
///
/// Build one with [`Sha256::new`], feed it data with [`Sha256::update`], then
/// finalise into a 32-byte digest with [`Sha256::finalize`].
///
/// ```rust
/// use logic_nih_plug_crypto::sha256::{Sha256, SHA256_OUTPUT_LEN};
///
/// let mut h = Sha256::new();
/// h.update(b"hello, ");
/// h.update(b"world");
/// assert_eq!(h.finalize().len(), SHA256_OUTPUT_LEN);
/// ```
#[derive(Clone)]
pub struct Sha256 {
    inner: sha2::Sha256,
}

impl Sha256 {
    /// Creates a fresh SHA-256 context.
    pub fn new() -> Self {
        Self {
            inner: sha2::Sha256::new(),
        }
    }

    /// Feeds `data` into the hash.
    pub fn update(&mut self, data: &[u8]) {
        self.inner.update(data);
    }

    /// Consumes the context and returns the 32-byte SHA-256 digest of the
    /// input fed in so far.
    pub fn finalize(self) -> [u8; SHA256_OUTPUT_LEN] {
        let out = self.inner.finalize();
        let mut buf = [0u8; SHA256_OUTPUT_LEN];
        buf.copy_from_slice(&out);
        buf
    }

    /// Consumes the context and returns the digest as a lowercase hex string
    /// (`64` characters).
    pub fn finalize_hex(self) -> String {
        crate::hex::encode(&self.finalize())
    }
}

impl Default for Sha256 {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Sha256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sha256").finish_non_exhaustive()
    }
}

/// One-shot helper: returns the 32-byte SHA-256 digest of `data`.
pub fn sha256(data: &[u8]) -> [u8; SHA256_OUTPUT_LEN] {
    let mut h = Sha256::new();
    h.update(data);
    h.finalize()
}

/// One-shot helper: returns the SHA-256 digest of `data` as a lowercase hex
/// string.
pub fn sha256_hex(data: &[u8]) -> String {
    crate::hex::encode(&sha256(data))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Standard NIST test vector — the empty string SHA-256 digest.
    const EMPTY_DIGEST: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

    /// Standard NIST test vector — "abc".
    const ABC_DIGEST: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

    /// Standard NIST test vector — "a" repeated 1,000,000 times.
    const LONG_DIGEST: &str = "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0";

    #[test]
    fn empty_string() {
        assert_eq!(sha256_hex(b""), EMPTY_DIGEST);
    }

    #[test]
    fn abc() {
        assert_eq!(sha256_hex(b"abc"), ABC_DIGEST);
    }

    #[test]
    fn long_string() {
        // 1,000,000 'a's, broken across many update() calls.
        let mut h = Sha256::new();
        for _ in 0..10_000 {
            h.update(b"a".repeat(100).as_slice());
        }
        // One big buffer for comparison.
        let buf = vec![b'a'; 1_000_000];
        assert_eq!(h.finalize_hex(), LONG_DIGEST);
        assert_eq!(sha256_hex(&buf), LONG_DIGEST);
    }

    #[test]
    fn streaming_matches_one_shot() {
        let mut h = Sha256::new();
        h.update(b"part one. ");
        h.update(b"part two.");
        assert_eq!(h.finalize_hex(), sha256_hex(b"part one. part two."));
    }

    #[test]
    fn finalize_yields_32_bytes() {
        let h = Sha256::new();
        assert_eq!(h.finalize().len(), SHA256_OUTPUT_LEN);
    }

    #[test]
    fn default_equals_new() {
        let a = Sha256::default();
        let b = Sha256::new();
        assert_eq!(a.finalize_hex(), b.finalize_hex());
    }

    #[test]
    fn clone_is_consistent() {
        let mut a = Sha256::new();
        a.update(b"hello");
        let mut b = a.clone();
        a.update(b" world");
        b.update(b" world");
        assert_eq!(a.finalize_hex(), b.finalize_hex());
    }
}
