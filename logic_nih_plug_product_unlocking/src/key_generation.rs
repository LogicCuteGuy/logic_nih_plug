//! Keyfile generation, decryption, and parsing.
//!
//! This is the server-side half of JUCE's product-unlocking module: it mints
//! a human-readable text blob (the "keyfile") that the user can drop into
//! the client app, plus the client-side helpers for reading the keyfile
//! back. The encryption is **raw textbook RSA** — `m^e mod n` / `c^d mod n` —
//! matching [`juce::KeyGeneration`].
//!
//! [`juce::KeyGeneration`]: https://docs.juce.com/master/classjuce_1_1KeyGeneration.html
//!
//! ## Keyfile format
//!
//! A keyfile is a block of plain text:
//!
//! ```text
//! Keyfile for <app_name>
//! User: <user_name>
//! Email: <user_email>
//! Machine numbers: <comma- or semicolon-separated>
//! Created: <RFC 2822-ish timestamp>
//!
//! #<70-char hex chunk>
//! #<70-char hex chunk>
//! ...
//! ```
//!
//! The hex after the last `#` is the raw-RSA ciphertext of a single-line
//! `<key …/>` XML element holding the same metadata as the comment header.
//! The XML is what the client actually decrypts and validates; the human
//! header is purely informational.

use num_bigint::BigUint;

use crate::error::KeyFileError;
use logic_nih_plug_crypto::rsa_key::RSAKey;

/// The well-known failure strings [`KeyFileError::as_license_result`]
/// returns. Mirrors `juce::OnlineUnlockStatus::LicenseResult`.
pub struct LicenseResult;

impl LicenseResult {
    /// "ID generator is not ready, try again later."
    pub const NOT_READY: &'static str = "ID generator is not ready, try again later.";
    /// "Credentials are invalid."
    pub const BAD_CREDENTIALS: &'static str = "Credentials are invalid.";
    /// "ProductID is incorrect."
    pub const BAD_PRODUCT_ID: &'static str = "ProductID is incorrect.";
    /// "License has expired."
    pub const LICENSE_EXPIRED: &'static str = "License has expired.";
    /// "Generic unlock failure."
    pub const UNLOCK_FAILED: &'static str = "Generic unlock failure.";
}

/// The metadata extracted from a key file after decryption.
///
/// This is the value returned by [`decrypt_key_file`]. The fields are
/// populated from the `<key …/>` XML element, *not* from the human-readable
/// header (the human header is for the user; the XML is the source of truth
/// after decryption).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyFileData {
    /// The `user` attribute — the licensee's display name.
    pub licensee: String,
    /// The `email` attribute — the licensee's email.
    pub email: String,
    /// The `app` attribute — the product ID the keyfile is valid for.
    pub app_id: String,
    /// The `mach` (or `expiring_mach`) attribute, split on `,` or `;`.
    pub machine_numbers: Vec<String>,
    /// `true` if the keyfile had an `expiryTime` attribute (and therefore
    /// an `expiring_mach` instead of `mach`).
    pub key_file_expires: bool,
    /// The `expiryTime` attribute, parsed as a hex-encoded millisecond
    /// timestamp since the Unix epoch. Zero when [`Self::key_file_expires`]
    /// is `false`.
    pub expiry_time_ms: i64,
}

/// Generates a license key file with the given `app_name`, `user_email`,
/// `user_name`, and `machine_numbers`. The returned string is a block of
/// plain text the user can drop into the app, or have the marketplace
/// server send to them.
///
/// `private_key` must have a private half loaded (i.e. it must have been
/// produced by [`RSAKey::generate`] or [`RSAKey::from_private_components`])
/// — the encryption is the textbook RSA `m^d mod n` operation.
///
/// `machine_numbers` is a list of machine IDs the user is allowed to
/// unlock the product on, separated by `,`, `;`, or whitespace. Comparison
/// is case-insensitive and ignores leading/trailing whitespace per
/// machine number.
///
/// # Errors
///
/// Returns [`KeyFileError::Malformed`] if `private_key` doesn't have a
/// private half loaded (you can't sign without `d`).
pub fn generate_key_file(
    app_name: &str,
    user_email: &str,
    user_name: &str,
    machine_numbers: &str,
    private_key: &RSAKey,
) -> Result<String, KeyFileError> {
    let xml = build_key_xml(
        "mach",
        app_name,
        user_email,
        user_name,
        machine_numbers,
        None,
    );
    let comment = build_key_comment(app_name, user_email, user_name, machine_numbers, None);
    encrypt_and_format(&xml, &comment, private_key)
}

/// Like [`generate_key_file`] but the resulting keyfile has an expiry
/// time. After `expiry_time_ms` (milliseconds since the Unix epoch),
/// [`decrypt_key_file`] / [`KeyFileData`] will still return the metadata
/// but [`crate::online_unlock_status::OnlineUnlockStatus::is_unlocked`]
/// will report `false` and you'll need to check the expiry yourself.
///
/// The keyfile uses the `expiring_mach` attribute instead of `mach` and
/// also stores `expiryTime` as a hex-encoded millisecond timestamp.
pub fn generate_expiring_key_file(
    app_name: &str,
    user_email: &str,
    user_name: &str,
    machine_numbers: &str,
    expiry_time_ms: i64,
    private_key: &RSAKey,
) -> Result<String, KeyFileError> {
    let xml = build_key_xml(
        "expiring_mach",
        app_name,
        user_email,
        user_name,
        machine_numbers,
        Some(expiry_time_ms),
    );
    let comment = build_key_comment(
        app_name,
        user_email,
        user_name,
        machine_numbers,
        Some(expiry_time_ms),
    );
    encrypt_and_format(&xml, &comment, private_key)
}

/// Decrypts `keyfile_text` with `public_key` and returns the metadata it
/// contains.
///
/// This is the client-side counterpart to [`generate_key_file`] — it
/// expects a keyfile produced by a server that holds the matching private
/// key. The decryption is the textbook RSA `c^e mod n` operation, so you
/// need an [`RSAKey`] constructed from the public components (typically
/// [`RSAKey::from_public_components`], or a public-only clone of the
/// issuer's keypair).
///
/// # Errors
///
/// - [`KeyFileError::Malformed`] — the keyfile was missing the `#`
///   marker, the hex blob was unparsable, or the decrypted bytes weren't
///   valid UTF-8.
/// - [`KeyFileError::InvalidXml`] — the decrypted bytes weren't a
///   `<key …/>` element, or the element didn't close properly.
pub fn decrypt_key_file(
    keyfile_text: &str,
    public_key: &RSAKey,
) -> Result<KeyFileData, KeyFileError> {
    let hex = extract_hex_payload(keyfile_text)?;
    let ciphertext = BigUint::parse_bytes(hex.as_bytes(), 16)
        .ok_or(KeyFileError::Malformed)?;
    let plaintext_bytes = raw_rsa_decrypt(&ciphertext, public_key);
    let xml_text = std::str::from_utf8(&plaintext_bytes).map_err(|_| KeyFileError::Malformed)?;
    parse_key_xml(xml_text)
}

// ----- internals -----

/// 70 chars per hex line, matching JUCE.
const HEX_CHARS_PER_LINE: usize = 70;

/// Maximum plaintext size, in bytes. A 2048-bit RSA key can only encrypt
/// up to 256 bytes; a 4096-bit key can encrypt up to 512. We cap at 1024
/// to match the largest reasonable key the host might generate.
const MAX_PLAINTEXT_BYTES: usize = 1024;

fn build_key_xml(
    mach_attr: &str,
    app_name: &str,
    user_email: &str,
    user_name: &str,
    machine_numbers: &str,
    expiry_time_ms: Option<i64>,
) -> String {
    // JUCE uses `String::toHexString(Time::getCurrentTime().toMilliseconds())`
    // for the `date` field; the value is purely informational and not
    // validated on decryption, but we include it for parity.
    let date_hex = {
        let ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        format!("{ms:x}")
    };

    let mut xml = String::with_capacity(128);
    xml.push_str("<key");
    push_xml_attr(&mut xml, "user", user_name);
    push_xml_attr(&mut xml, "email", user_email);
    push_xml_attr(&mut xml, mach_attr, machine_numbers);
    push_xml_attr(&mut xml, "app", app_name);
    push_xml_attr(&mut xml, "date", &date_hex);
    if let Some(ms) = expiry_time_ms {
        push_xml_attr(&mut xml, "expiryTime", &format!("{ms:x}"));
    }
    xml.push_str("/>");
    xml
}

fn push_xml_attr(out: &mut String, key: &str, value: &str) {
    out.push(' ');
    out.push_str(key);
    out.push_str("=\"");
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(ch),
        }
    }
    out.push('"');
}

fn build_key_comment(
    app_name: &str,
    user_email: &str,
    user_name: &str,
    machine_numbers: &str,
    expiry_time_ms: Option<i64>,
) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let secs_in_day = 86_400;
    // Roll our own (very small) `Time::toString(true, true)`-equivalent —
    // we don't need full locale, just a stable "YYYY-MM-DD HH:MM:SS UTC"
    // rendering that matches the shape JUCE produces.
    let date_str = format_unix_timestamp(now);

    let mut comment = String::with_capacity(128);
    comment.push_str("Keyfile for ");
    comment.push_str(app_name);
    comment.push('\n');
    if !user_name.is_empty() {
        comment.push_str("User: ");
        comment.push_str(user_name);
        comment.push('\n');
    }
    comment.push_str("Email: ");
    comment.push_str(user_email);
    comment.push('\n');
    comment.push_str("Machine numbers: ");
    comment.push_str(machine_numbers);
    comment.push('\n');
    comment.push_str("Created: ");
    comment.push_str(&date_str);
    if let Some(ms) = expiry_time_ms {
        let expiry_secs = ms / 1000;
        let _ = secs_in_day; // suppress unused warning while keeping the constant
        comment.push('\n');
        comment.push_str("Expires: ");
        comment.push_str(&format_unix_timestamp(expiry_secs));
    }
    comment.push('\n');
    comment
}

fn format_unix_timestamp(secs: i64) -> String {
    // Convert (days-since-epoch, secs-into-day) to Y-M-D H:M:S UTC.
    let secs_per_day = 86_400;
    let days = secs.div_euclid(secs_per_day);
    let time = secs.rem_euclid(secs_per_day);
    let hour = time / 3600;
    let minute = (time / 60) % 60;
    let second = time % 60;
    let (y, m, d) = civil_from_days(days);
    format!("{y:04}-{m:02}-{d:02} {hour:02}:{minute:02}:{second:02}")
}

/// Howard Hinnant's `civil_from_days` — converts days-since-Unix-epoch
/// (1970-01-01 = 0) to (year, month, day) without going through chrono.
fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m, d)
}

fn encrypt_and_format(
    xml: &str,
    comment: &str,
    private_key: &RSAKey,
) -> Result<String, KeyFileError> {
    if !private_key.has_private() {
        return Err(KeyFileError::Malformed);
    }
    let plaintext = xml.as_bytes();
    if plaintext.len() > MAX_PLAINTEXT_BYTES {
        return Err(KeyFileError::Malformed);
    }
    let m = BigUint::from_bytes_be(plaintext);
    let ciphertext = raw_rsa_encrypt(&m, private_key);
    let hex = format!("{ciphertext:x}");

    let mut out = String::with_capacity(comment.len() + 8 + hex.len() * 2);
    out.push_str(comment);
    let bytes = hex.as_bytes();
    // JUCE's keyfile format: the hex blob is split into 70-char lines,
    // with a single `#` prefix on the FIRST line only. The client uses
    // `rfind('#')` (or equivalent) to find the start of the hex blob
    // — having a `#` only on the first line makes this unambiguous.
    for (i, chunk) in bytes.chunks(HEX_CHARS_PER_LINE).enumerate() {
        if i == 0 {
            out.push('#');
        }
        // SAFETY: `chunk` is a sub-slice of an ASCII hex string.
        out.push_str(std::str::from_utf8(chunk).expect("hex is ASCII"));
        out.push('\n');
    }
    Ok(out)
}

fn extract_hex_payload(keyfile_text: &str) -> Result<String, KeyFileError> {
    // The XML hex is the text after the *last* `#` in the file. The
    // human header has no `#` in it, so this is unambiguous.
    let idx = keyfile_text
        .rfind('#')
        .ok_or(KeyFileError::Malformed)?;
    let mut hex = String::with_capacity(keyfile_text.len() - idx);
    for ch in keyfile_text[idx + 1..].chars() {
        if ch == '\n' || ch == '\r' || ch == ' ' || ch == '\t' {
            continue;
        }
        hex.push(ch);
    }
    if hex.is_empty() {
        return Err(KeyFileError::Malformed);
    }
    Ok(hex)
}

fn raw_rsa_encrypt(plaintext: &BigUint, private_key: &RSAKey) -> BigUint {
    // Textbook RSA: c = m^d mod n. The private key is used to *sign*
    // (verify by the holder of the public key); the public key is used
    // to *encrypt* (decrypt by the holder of the private key). JUCE's
    // `KeyGeneration` flips the convention — the server signs with the
    // private key so anyone with the public key can verify. We mirror
    // that: caller passes the private key, we apply `m^d mod n` here.
    let n = BigUint::from_bytes_be(&private_key.n_bytes());
    let d_bytes = private_key
        .d_bytes()
        .expect("caller checked has_private");
    let d = BigUint::from_bytes_be(&d_bytes);
    plaintext.modpow(&d, &n)
}

fn raw_rsa_decrypt(ciphertext: &BigUint, public_key: &RSAKey) -> Vec<u8> {
    // m = c^e mod n.
    let n = BigUint::from_bytes_be(&public_key.n_bytes());
    let e = BigUint::from_bytes_be(&public_key.e_bytes());
    let m = ciphertext.modpow(&e, &n);
    // `m` is the same `BigUint` we encrypted with (modular inverse is
    // exact), so `m.to_bytes_be()` reproduces the original byte string
    // exactly. We do *not* pad back to the modulus size — that would
    // add leading zeros that aren't part of the plaintext.
    m.to_bytes_be()
}

fn parse_key_xml(xml_text: &str) -> Result<KeyFileData, KeyFileError> {
    // The XML is always a single line: `<key …/>`. We parse just enough
    // to extract the attributes — no need to pull in a full XML library
    // for a one-tag document.
    let trimmed = xml_text.trim();
    let body = trimmed
        .strip_prefix("<key")
        .or_else(|| trimmed.strip_prefix("<key "))
        .ok_or_else(|| KeyFileError::InvalidXml("missing <key tag".into()))?;
    let body = body
        .trim_start()
        .strip_suffix("/>")
        .ok_or_else(|| KeyFileError::InvalidXml("missing /> close".into()))?;

    let attrs = parse_attributes(body);

    let licensee = attrs
        .get("user")
        .cloned()
        .unwrap_or_default();
    let email = attrs
        .get("email")
        .cloned()
        .unwrap_or_default();
    let app_id = attrs.get("app").cloned().unwrap_or_default();

    let (machine_numbers, key_file_expires) = if let Some(expiring) = attrs.get("expiring_mach") {
        (split_machine_numbers(expiring), true)
    } else if let Some(mach) = attrs.get("mach") {
        (split_machine_numbers(mach), false)
    } else {
        (Vec::new(), false)
    };

    let expiry_time_ms = if key_file_expires {
        attrs
            .get("expiryTime")
            .and_then(|s| u64::from_str_radix(s, 16).ok())
            .map(|v| v as i64)
            .unwrap_or(0)
    } else {
        0
    };

    Ok(KeyFileData {
        licensee,
        email,
        app_id,
        machine_numbers,
        key_file_expires,
        expiry_time_ms,
    })
}

fn split_machine_numbers(s: &str) -> Vec<String> {
    s.split(|c: char| c == ',' || c == ';' || c.is_whitespace())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}

fn parse_attributes(s: &str) -> std::collections::HashMap<String, String> {
    parse_attributes_for_test(s)
}

/// Public re-export of [`parse_attributes`] for the sibling
/// `online_unlock_status` module, which uses the same minimal XML
/// attribute parser to handle server replies.
pub(crate) fn parse_attributes_for_test(s: &str) -> std::collections::HashMap<String, String> {
    let mut out = std::collections::HashMap::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Skip whitespace.
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        // Read attribute name.
        let name_start = i;
        while i < bytes.len() && bytes[i] != b'=' && !bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        let name = &s[name_start..i];
        // Skip whitespace and `=`.
        while i < bytes.len() && (bytes[i].is_ascii_whitespace() || bytes[i] == b'=') {
            i += 1;
        }
        // Read quoted value.
        if i >= bytes.len() || bytes[i] != b'"' {
            // Malformed — skip the rest.
            break;
        }
        i += 1;
        let value_start = i;
        let mut value = String::new();
        while i < bytes.len() && bytes[i] != b'"' {
            if bytes[i] == b'&' {
                // Minimal entity decoder: &amp; &lt; &gt; &quot; &apos;
                let entity_start = i;
                if let Some(end) = s[i..].find(';') {
                    let entity = &s[i..i + end + 1];
                    let decoded = match entity {
                        "&amp;" => Some('&'),
                        "&lt;" => Some('<'),
                        "&gt;" => Some('>'),
                        "&quot;" => Some('"'),
                        "&apos;" => Some('\''),
                        _ => None,
                    };
                    if let Some(ch) = decoded {
                        value.push(ch);
                        i += entity.len();
                        continue;
                    }
                    let _ = entity_start;
                }
            }
            value.push(bytes[i] as char);
            i += 1;
        }
        let _ = value_start;
        if i < bytes.len() {
            i += 1; // skip closing `"`
        }
        if !name.is_empty() {
            out.insert(name.to_owned(), value);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_key() -> RSAKey {
        RSAKey::generate(2048).expect("key generation failed")
    }

    #[test]
    fn roundtrip_standard_keyfile() {
        let key = make_key();
        let text = generate_key_file(
            "MyApp",
            "joe@foo.bar",
            "Joe Bloggs",
            "MACHINE1,MACHINE2",
            &key,
        )
        .expect("generate");

        // Comment header is human-readable.
        assert!(text.contains("Keyfile for MyApp"));
        assert!(text.contains("Email: joe@foo.bar"));
        assert!(text.contains("User: Joe Bloggs"));
        assert!(text.contains("Machine numbers: MACHINE1,MACHINE2"));

        let public = key.clone().into_public_only();
        let data = decrypt_key_file(&text, &public).expect("decrypt");
        assert_eq!(data.licensee, "Joe Bloggs");
        assert_eq!(data.email, "joe@foo.bar");
        assert_eq!(data.app_id, "MyApp");
        assert_eq!(
            data.machine_numbers,
            vec!["MACHINE1".to_string(), "MACHINE2".to_string()]
        );
        assert!(!data.key_file_expires);
        assert_eq!(data.expiry_time_ms, 0);
    }

    #[test]
    fn roundtrip_expiring_keyfile() {
        let key = make_key();
        let expiry = 0x1234_5678_9abc_def0_i64;
        let text = generate_expiring_key_file(
            "MyApp",
            "joe@foo.bar",
            "Joe Bloggs",
            "MACHINE1",
            expiry,
            &key,
        )
        .expect("generate");

        let public = key.clone().into_public_only();
        let data = decrypt_key_file(&text, &public).expect("decrypt");
        assert!(data.key_file_expires);
        assert_eq!(data.expiry_time_ms, expiry);
    }

    #[test]
    fn wrong_key_fails_to_decrypt() {
        let issuer = make_key();
        let attacker = make_key();
        let text = generate_key_file("MyApp", "x@y", "x", "M1", &issuer).expect("generate");
        // The attacker can't decrypt: the decrypted bytes will be
        // random-looking garbage and almost certainly not parse as XML.
        let result = decrypt_key_file(&text, &attacker.into_public_only());
        // Either we get an InvalidXml error, or the user/email/app fields
        // are wrong — both are fine outcomes for this test.
        if let Ok(data) = result {
            // The XML parser is lenient enough that random bytes
            // occasionally form a valid-looking document with empty
            // attributes. In that case, the data should at least be
            // wrong.
            assert!(data.licensee != "x" || data.email != "x@y" || data.app_id != "MyApp");
        }
    }

    #[test]
    fn missing_marker_is_malformed() {
        let key = make_key();
        let result = decrypt_key_file("no marker here", &key.into_public_only());
        assert!(matches!(result, Err(KeyFileError::Malformed)));
    }

    #[test]
    fn missing_private_half_is_malformed() {
        let key = make_key().into_public_only();
        let result = generate_key_file("MyApp", "x@y", "x", "M1", &key);
        assert!(matches!(result, Err(KeyFileError::Malformed)));
    }

    #[test]
    fn split_machine_numbers_handles_separators() {
        assert_eq!(
            split_machine_numbers("a, b;c d"),
            vec!["a", "b", "c", "d"]
        );
        assert_eq!(split_machine_numbers("only-one"), vec!["only-one"]);
        assert!(split_machine_numbers("").is_empty());
        assert!(split_machine_numbers(" ,  , ").is_empty());
    }

    #[test]
    fn xml_attrs_parse_with_entities() {
        let attrs = parse_attributes(r#"user="Joe &amp; Jane" email="a@b" app="x""#);
        assert_eq!(attrs.get("user").map(String::as_str), Some("Joe & Jane"));
        assert_eq!(attrs.get("email").map(String::as_str), Some("a@b"));
        assert_eq!(attrs.get("app").map(String::as_str), Some("x"));
    }

    #[test]
    fn license_result_strings_are_stable() {
        // These are part of our public API — the strings flow into error
        // UIs of consumers.
        assert_eq!(LicenseResult::NOT_READY, "ID generator is not ready, try again later.");
        assert_eq!(LicenseResult::BAD_CREDENTIALS, "Credentials are invalid.");
        assert_eq!(LicenseResult::BAD_PRODUCT_ID, "ProductID is incorrect.");
        assert_eq!(LicenseResult::LICENSE_EXPIRED, "License has expired.");
        assert_eq!(LicenseResult::UNLOCK_FAILED, "Generic unlock failure.");
    }

    #[test]
    fn civil_from_days_matches_unix_epoch() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(1), (1970, 1, 2));
        assert_eq!(civil_from_days(365), (1971, 1, 1));
        assert_eq!(civil_from_days(19723), (2024, 1, 1));
    }

    #[test]
    fn raw_rsa_math_round_trips() {
        let key = make_key();
        let m = BigUint::from_bytes_be(b"hello, world");
        let n = BigUint::from_bytes_be(&key.n_bytes());
        let d = BigUint::from_bytes_be(&key.d_bytes().unwrap());
        let e = BigUint::from_bytes_be(&key.e_bytes());
        let c = m.modpow(&d, &n);
        let m2 = c.modpow(&e, &n);
        assert_eq!(m, m2);
    }
}
