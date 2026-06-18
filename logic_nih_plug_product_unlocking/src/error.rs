//! Error types for the product-unlocking crate.

use thiserror::Error;

#[cfg(feature = "key_generation")]
use crate::key_generation::LicenseResult;

/// Errors that can occur while generating, parsing, or applying a key file.
///
/// These mirror the [`juce::OnlineUnlockStatus::LicenseResult`] string
/// constants — see [`LicenseResult`] for the well-known failure modes.
#[derive(Debug, Error)]
pub enum KeyFileError {
    /// The keyfile was too short, had no `#` separator, or had unparsable
    /// hex/UTF-8 content.
    #[error("key file is malformed (missing '#' marker, bad hex, or non-UTF-8 payload)")]
    Malformed,

    /// The decrypted plaintext wasn't a valid `<key …/>` XML element (missing
    /// root tag, missing closing `/>`, etc.).
    #[error("key file XML is malformed: {0}")]
    InvalidXml(String),

    /// The product ID in the keyfile doesn't match the product ID the
    /// app expects. Mirror of [`LicenseResult::badProductID`].
    #[error("product ID in key file does not match: {expected} != {actual}")]
    ProductIdMismatch {
        /// The product ID the app declared via `UnlockStore::get_product_id()`.
        expected: String,
        /// The product ID found inside the keyfile's XML.
        actual: String,
    },

    /// The keyfile is missing either the `user` or the `email` attribute.
    /// Mirror of [`LicenseResult::badCredentials`].
    #[error("key file has empty user or email attribute")]
    BadCredentials,

    /// None of the machine numbers in the keyfile match any of the
    /// machine IDs the host reported. Mirror of [`LicenseResult::unlockFailed`].
    #[error("no machine number in the key file matches any of this host's machine IDs")]
    MachineNumberMismatch,

    /// The keyfile has an `expiryTime` attribute that is in the past. Mirror
    /// of [`LicenseResult::licenseExpired`].
    #[error("key file has expired (expiry timestamp {expiry_ms}, current {now_ms})")]
    LicenseExpired {
        /// Expiry timestamp from the keyfile, in milliseconds since the Unix epoch.
        expiry_ms: i64,
        /// The current time, in milliseconds since the Unix epoch.
        now_ms: i64,
    },

    /// The state machine hasn't been initialised yet (no public key, no
    /// state store, …). Mirror of [`LicenseResult::notReady`].
    #[error("online unlock is not ready (no public key, no machine ID, or no state store)")]
    NotReady,
}

impl KeyFileError {
    /// Returns the JUCE-style [`LicenseResult`] string for this error.
    pub fn as_license_result(&self) -> &'static str {
        match self {
            Self::NotReady => LicenseResult::NOT_READY,
            Self::BadCredentials => LicenseResult::BAD_CREDENTIALS,
            Self::ProductIdMismatch { .. } => LicenseResult::BAD_PRODUCT_ID,
            Self::LicenseExpired { .. } => LicenseResult::LICENSE_EXPIRED,
            Self::MachineNumberMismatch => LicenseResult::UNLOCK_FAILED,
            Self::Malformed | Self::InvalidXml(_) => LicenseResult::UNLOCK_FAILED,
        }
    }
}
