//! SHA-1 streaming hash.
//!
//! SHA-1 is considered cryptographically broken; only use it when you have a
//! legacy or non-security requirement (cache keys, content addressing on
//! trusted inputs, etc.). For new code, prefer [`crate::sha256`].
//!
//! The shape of this API intentionally mirrors [`crate::sha256`].

use sha1::Digest as _;

/// The length, in bytes, of a SHA-1 digest.
pub const SHA1_OUTPUT_LEN: usize = 20;

/// Streaming SHA-1 hash context.
///
/// ```rust
/// use logic_nih_plug_crypto::sha1::{Sha1, SHA1_OUTPUT_LEN};
///
/// let mut h = Sha1::new();
/// h.update(b"hello, ");
/// h.update(b"world");
/// assert_eq!(h.finalize().len(), SHA1_OUTPUT_LEN);
/// ```
#[derive(Clone)]
pub struct Sha1 {
    inner: sha1::Sha1,
}

impl Sha1 {
    /// Creates a fresh SHA-1 context.
    pub fn new() -> Self {
        Self {
            inner: sha1::Sha1::new(),
        }
    }

    /// Feeds `data` into the hash.
    pub fn update(&mut self, data: &[u8]) {
        self.inner.update(data);
    }

    /// Consumes the context and returns the 20-byte SHA-1 digest of the
    /// input fed in so far.
    pub fn finalize(self) -> [u8; SHA1_OUTPUT_LEN] {
        let out = self.inner.finalize();
        let mut buf = [0u8; SHA1_OUTPUT_LEN];
        buf.copy_from_slice(&out);
        buf
    }

    /// Consumes the context and returns the digest as a lowercase hex string
    /// (`40` characters).
    pub fn finalize_hex(self) -> String {
        crate::hex::encode(&self.finalize())
    }
}

impl Default for Sha1 {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Sha1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sha1").finish_non_exhaustive()
    }
}

/// One-shot helper: returns the 20-byte SHA-1 digest of `data`.
pub fn sha1(data: &[u8]) -> [u8; SHA1_OUTPUT_LEN] {
    let mut h = Sha1::new();
    h.update(data);
    h.finalize()
}

/// One-shot helper: returns the SHA-1 digest of `data` as a lowercase hex
/// string.
pub fn sha1_hex(data: &[u8]) -> String {
    crate::hex::encode(&sha1(data))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// SHA-1("") = da39a3ee5e6b4b0d3255bfef95601890afd80709
    const EMPTY_DIGEST: &str = "da39a3ee5e6b4b0d3255bfef95601890afd80709";

    /// SHA-1("abc") = a9993e364706816aba3e25717850c26c9cd0d89d
    const ABC_DIGEST: &str = "a9993e364706816aba3e25717850c26c9cd0d89d";

    /// SHA-1 of "The quick brown fox jumps over the lazy dog".
    const FOX_DIGEST: &str = "2fd4e1c67a2d28fced849ee1bb76e7391b93eb12";

    #[test]
    fn empty_string() {
        assert_eq!(sha1_hex(b""), EMPTY_DIGEST);
    }

    #[test]
    fn abc() {
        assert_eq!(sha1_hex(b"abc"), ABC_DIGEST);
    }

    #[test]
    fn fox() {
        assert_eq!(
            sha1_hex(b"The quick brown fox jumps over the lazy dog"),
            FOX_DIGEST
        );
    }

    #[test]
    fn streaming_matches_one_shot() {
        let mut h = Sha1::new();
        h.update(b"The quick brown fox ");
        h.update(b"jumps over the lazy dog");
        assert_eq!(h.finalize_hex(), FOX_DIGEST);
    }

    #[test]
    fn finalize_yields_20_bytes() {
        let h = Sha1::new();
        assert_eq!(h.finalize().len(), SHA1_OUTPUT_LEN);
    }
}
