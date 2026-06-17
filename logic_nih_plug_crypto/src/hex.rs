//! Tiny hex-encoding helper used by the hash modules.
//!
//! We avoid pulling in the `hex` crate just for this — it's two loops and a
//! lookup table.

/// Lowercase hex-encode a byte slice. Output length is always `2 * bytes.len()`.
#[allow(dead_code)] // unused when no hash feature is enabled
pub(crate) fn encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        assert_eq!(encode(&[]), "");
    }

    #[test]
    fn known_vectors() {
        assert_eq!(encode(&[0x00]), "00");
        assert_eq!(encode(&[0xff]), "ff");
        assert_eq!(encode(&[0xde, 0xad, 0xbe, 0xef]), "deadbeef");
        assert_eq!(encode(b"abc"), "616263");
    }
}
