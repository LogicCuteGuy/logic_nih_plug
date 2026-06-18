//! Machine-ID helpers ported from JUCE's `OnlineUnlockStatus::MachineIDUtilities`.
//!
//! These helpers are used to derive the short alphanumeric identifiers the
//! keyfile gets bound to. Comparison is case-insensitive (the lookup logic
//! in [`crate::online_unlock_status`] upper-cases both sides), so the
//! casing in [`get_encoded_id_string`] is purely cosmetic.
//!
//! ## Why a single-character platform prefix?
//!
//! JUCE's `getPlatformPrefix()` returns one of `'W'` / `'M'` / `'L'` /
//! `'I'` / `'A'` / `'B'`. We mirror that so the machine IDs a Rust
//! client produces look the same as the machine IDs a JUCE client
//! produces — important if you're running a mixed fleet against a
//! shared licensing server.

/// One-character prefix identifying the OS a machine ID was minted on.
///
/// - `W` — Windows
/// - `M` — macOS
/// - `L` — Linux
/// - `B` — *BSD
/// - `I` — iOS
/// - `A` — Android
/// - `?` — unknown
pub fn get_platform_prefix() -> char {
    #[cfg(target_os = "windows")]
    {
        'W'
    }
    #[cfg(target_os = "macos")]
    {
        'M'
    }
    #[cfg(target_os = "linux")]
    {
        'L'
    }
    #[cfg(target_os = "freebsd")]
    {
        'B'
    }
    #[cfg(target_os = "openbsd")]
    {
        'B'
    }
    #[cfg(target_os = "netbsd")]
    {
        'B'
    }
    #[cfg(target_os = "ios")]
    {
        'I'
    }
    #[cfg(target_os = "android")]
    {
        'A'
    }
    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "linux",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "ios",
        target_os = "android",
    )))]
    {
        '?'
    }
}

/// Encodes `input_string` as a machine ID: the platform prefix character
/// followed by the first 9 hex digits of `MD5(input_string + "salt_1" +
/// prefix).hex()` upper-cased.
///
/// JUCE: `getEncodedIDString`.
pub fn get_encoded_id_string(input_string: &str) -> String {
    let prefix = get_platform_prefix();
    let salt = format!("{input_string}salt_1{prefix}");
    let hex = logic_nih_plug_crypto::md5::md5_hex(salt.as_bytes());
    let head: String = hex.chars().take(9).collect();
    format!("{prefix}{}", head.to_uppercase())
}

/// Returns an encoded machine ID derived from
/// [`std::process::id()`] (the host process ID).
///
/// JUCE's `getUniqueMachineID()` uses
/// `SystemStats::getUniqueDeviceID()`, which is a stable per-machine
/// identifier across reboots. We don't have a portable equivalent
/// in the stdlib, so this implementation derives the ID from the
/// process ID. The user is expected to override
/// [`crate::online_unlock_status::UnlockStore::get_local_machine_ids`]
/// with a stable per-machine ID source for production use; the
/// default is good enough for tests and for plugins that are
/// shipped as standalone apps with their own settings file.
pub fn get_unique_machine_id() -> String {
    let pid = std::process::id();
    get_encoded_id_string(&format!("{pid}"))
}

/// Returns a non-empty list of machine IDs, derived from the current
/// process.
///
/// This is a *very rough* default — most users will want to override
/// [`crate::online_unlock_status::UnlockStore::get_local_machine_ids`]
/// with stable per-machine IDs (MAC addresses, disk serial numbers,
/// machine GUIDs, …). For the default case we return a single ID
/// derived from the process ID so the unlock flow has something to
/// work with.
pub fn get_local_machine_ids() -> Vec<String> {
    vec![get_unique_machine_id()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_prefix_is_stable() {
        let p = get_platform_prefix();
        assert!(p.is_ascii_uppercase() || p == '?');
    }

    #[test]
    fn encoded_id_is_ten_chars() {
        // 1 prefix char + 9 hex chars.
        let id = get_encoded_id_string("test-input");
        assert_eq!(id.len(), 10);
        assert_eq!(id.chars().next(), Some(get_platform_prefix()));
        assert!(id.chars().skip(1).all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn encoded_id_is_deterministic() {
        let a = get_encoded_id_string("hello, world");
        let b = get_encoded_id_string("hello, world");
        assert_eq!(a, b);
        // And a different input produces a different ID.
        let c = get_encoded_id_string("different");
        assert_ne!(a, c);
    }

    #[test]
    fn local_machine_ids_is_non_empty() {
        let ids = get_local_machine_ids();
        assert!(!ids.is_empty());
    }
}
