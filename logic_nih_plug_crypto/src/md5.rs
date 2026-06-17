//! MD5 streaming hash.
//!
//! MD5 is broken for cryptographic use; only use it for non-security
//! applications (cache keys, fingerprinting non-malicious inputs, etc.).
//!
//! The shape of this API intentionally mirrors [`crate::sha256`].

use md5::Digest as _;

/// The length, in bytes, of an MD5 digest.
pub const MD5_OUTPUT_LEN: usize = 16;

/// Streaming MD5 hash context.
///
/// ```rust
/// use logic_nih_plug_crypto::md5::{Md5, MD5_OUTPUT_LEN};
///
/// let mut h = Md5::new();
/// h.update(b"hello, ");
/// h.update(b"world");
/// assert_eq!(h.finalize().len(), MD5_OUTPUT_LEN);
/// ```
#[derive(Clone)]
pub struct Md5 {
    inner: md5::Md5,
}

impl Md5 {
    /// Creates a fresh MD5 context.
    pub fn new() -> Self {
        Self {
            inner: md5::Md5::new(),
        }
    }

    /// Feeds `data` into the hash.
    pub fn update(&mut self, data: &[u8]) {
        self.inner.update(data);
    }

    /// Consumes the context and returns the 16-byte MD5 digest of the input
    /// fed in so far.
    pub fn finalize(self) -> [u8; MD5_OUTPUT_LEN] {
        let out = self.inner.finalize();
        let mut buf = [0u8; MD5_OUTPUT_LEN];
        buf.copy_from_slice(&out);
        buf
    }

    /// Consumes the context and returns the digest as a lowercase hex string
    /// (`32` characters).
    pub fn finalize_hex(self) -> String {
        crate::hex::encode(&self.finalize())
    }
}

impl Default for Md5 {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Md5 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Md5").finish_non_exhaustive()
    }
}

/// One-shot helper: returns the 16-byte MD5 digest of `data`.
pub fn md5(data: &[u8]) -> [u8; MD5_OUTPUT_LEN] {
    let mut h = Md5::new();
    h.update(data);
    h.finalize()
}

/// One-shot helper: returns the MD5 digest of `data` as a lowercase hex
/// string.
pub fn md5_hex(data: &[u8]) -> String {
    crate::hex::encode(&md5(data))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// MD5("") = d41d8cd98f00b204e9800998ecf8427e
    const EMPTY_DIGEST: &str = "d41d8cd98f00b204e9800998ecf8427e";

    /// MD5("abc") = 900150983cd24fb0d6963f7d28e17f72
    const ABC_DIGEST: &str = "900150983cd24fb0d6963f7d28e17f72";

    /// MD5("The quick brown fox jumps over the lazy dog").
    const FOX_DIGEST: &str = "9e107d9d372bb6826bd81d3542a419d6";

    /// MD5("The quick brown fox jumps over the lazy dog.").
    /// Note the trailing period produces a different digest — classic gotcha.
    const FOX_DOT_DIGEST: &str = "e4d909c290d0fb1ca068ffaddf22cbd0";

    #[test]
    fn empty_string() {
        assert_eq!(md5_hex(b""), EMPTY_DIGEST);
    }

    #[test]
    fn abc() {
        assert_eq!(md5_hex(b"abc"), ABC_DIGEST);
    }

    #[test]
    fn fox() {
        assert_eq!(
            md5_hex(b"The quick brown fox jumps over the lazy dog"),
            FOX_DIGEST
        );
    }

    #[test]
    fn fox_with_dot() {
        assert_eq!(
            md5_hex(b"The quick brown fox jumps over the lazy dog."),
            FOX_DOT_DIGEST
        );
    }

    #[test]
    fn streaming_matches_one_shot() {
        let mut h = Md5::new();
        h.update(b"The quick brown fox ");
        h.update(b"jumps over the lazy dog");
        assert_eq!(h.finalize_hex(), FOX_DIGEST);
    }

    #[test]
    fn finalize_yields_16_bytes() {
        let h = Md5::new();
        assert_eq!(h.finalize().len(), MD5_OUTPUT_LEN);
    }
}
