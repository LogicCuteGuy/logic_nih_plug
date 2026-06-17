//! # logic_nih_plug_crypto
//!
//! Hashing, big-integer arithmetic and RSA keys ported from JUCE for nih-plug.
//!
//! Each functional area is gated behind its own Cargo feature so that you only
//! pull in the cryptography primitives you actually need:
//!
//! - [`sha256`] — streaming SHA-256 (also used internally by the `rsa`
//!   feature).
//! - [`sha1`] — streaming SHA-1.
//! - [`md5`] — streaming MD5.
//! - [`big_integer`] — arbitrary-precision unsigned integer arithmetic.
//! - [`rsa_key`] — RSA key generation, signing and verification.
//!
//! ## Feature flags
//!
//! | Feature | Default | What it adds |
//! |---|---|---|
//! | `sha2` | ✅ | SHA-256 streaming hash + one-shot helpers |
//! | `sha1` | — | SHA-1 streaming hash + one-shot helpers |
//! | `md5` | — | MD5 streaming hash + one-shot helpers |
//! | `bignum` | — | [`BigInteger`](big_integer::BigInteger) |
//! | `rsa` | — | [`RSAKey`](rsa_key::RSAKey) (also enables `sha2`) |
//! | `full` | — | All of the above |
//!
//! ## Example
//!
//! ```rust
//! use logic_nih_plug_crypto::sha256::{Sha256, sha256_hex};
//!
//! // One-shot
//! assert_eq!(
//!     sha256_hex(b"abc"),
//!     "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
//! );
//!
//! // Streaming
//! let mut h = Sha256::new();
//! h.update(b"hello, ");
//! h.update(b"world");
//! assert_eq!(h.finalize_hex(), sha256_hex(b"hello, world"));
//! ```

#![warn(missing_docs)]

pub mod error;

mod hex;

#[cfg(feature = "sha2")]
pub mod sha256;

#[cfg(feature = "sha1")]
pub mod sha1;

#[cfg(feature = "md5")]
pub mod md5;

#[cfg(feature = "bignum")]
pub mod big_integer;

#[cfg(feature = "rsa")]
pub mod rsa_key;

pub use error::CryptoError;
