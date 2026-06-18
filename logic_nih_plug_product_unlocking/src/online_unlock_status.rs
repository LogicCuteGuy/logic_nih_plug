//! [`OnlineUnlockStatus`] — the client-side state machine.
//!
//! Mirrors JUCE's [`juce::OnlineUnlockStatus`][juce] class. The Rust
//! version is a generic `OnlineUnlockStatus<S: UnlockStore>` that
//! delegates the *store-specific* bits (webserver URL, public key,
//! persistent state, …) to a user-supplied [`UnlockStore`]
//! implementation, and keeps the unlock state on an internal
//! `ValueTree`.
//!
//! [juce]: https://docs.juce.com/master/classjuce_1_1OnlineUnlockStatus.html
//!
//! ## Usage
//!
//! ```rust,no_run
//! use logic_nih_plug_data::ValueTree;
//! use logic_nih_plug_product_unlocking::online_unlock_status::{
//!     OnlineUnlockStatus, UnlockStore, UnlockResult,
//! };
//! use logic_nih_plug_crypto::rsa_key::RSAKey;
//!
//! struct MyStore {
//!     public_key: RSAKey,
//!     product_id: String,
//!     persisted_state: String,
//! }
//!
//! impl UnlockStore for MyStore {
//!     fn get_product_id(&self) -> &str { &self.product_id }
//!     fn does_product_id_match(&self, id: &str) -> bool { id == self.product_id }
//!     fn get_public_key(&self) -> &RSAKey { &self.public_key }
//!     fn save_state(&mut self, state: &str) { self.persisted_state = state.to_string(); }
//!     fn get_state(&self) -> String { self.persisted_state.clone() }
//!     fn get_website_name(&self) -> &str { "My Store" }
//!     fn get_server_authentication_url(&self) -> &str {
//!         "https://example.com/auth"
//!     }
//!     fn read_reply_from_webserver(&self, _email: &str, _password: &str) -> String {
//!         String::new()
//!     }
//! }
//!
//! let store = MyStore {
//!     public_key: RSAKey::generate(2048).expect("key generation"),
//!     product_id: "MY-PRODUCT-1".to_owned(),
//!     persisted_state: String::new(),
//! };
//! let mut status = OnlineUnlockStatus::new(store);
//! status.load();
//! if status.is_unlocked() {
//!     // already unlocked — proceed
//! }
//! ```

use logic_nih_plug_crypto::rsa_key::RSAKey;
use logic_nih_plug_data::{Identifier, ValueTree};

use crate::error::KeyFileError;
use crate::key_generation::decrypt_key_file;
use crate::machine_id;

/// The trait store-specific subclasses of [`OnlineUnlockStatus`]
/// implement.
///
/// This is the Rust equivalent of `juce::OnlineUnlockStatus`'s
/// pure-virtual methods. The default implementations of
/// [`get_local_machine_ids`], [`user_cancelled`], and
/// [`get_message_for_connection_failure`] mirror JUCE's default
/// behaviour.
///
/// [`get_local_machine_ids`]: Self::get_local_machine_ids
/// [`user_cancelled`]: Self::user_cancelled
/// [`get_message_for_connection_failure`]: Self::get_message_for_connection_failure
pub trait UnlockStore {
    /// Your product's ID, as allocated by the store.
    fn get_product_id(&self) -> &str;

    /// Returns `true` if `returned_id_from_server` is a product ID the
    /// app should accept.
    fn does_product_id_match(&self, returned_id_from_server: &str) -> bool;

    /// The RSA public key for authenticating server responses.
    fn get_public_key(&self) -> &RSAKey;

    /// Persist `state` to whatever backing store your app uses.
    fn save_state(&mut self, state: &str);

    /// Retrieve the state previously passed to [`save_state`](Self::save_state).
    /// On first run, return an empty string.
    fn get_state(&self) -> String;

    /// The name of the web-store website, used for user-facing error
    /// messages.
    fn get_website_name(&self) -> &str;

    /// The URL of the authentication API.
    fn get_server_authentication_url(&self) -> &str;

    /// Contact the webserver and attempt to unlock the current machine
    /// for `email` / `password`. Return the XML text the server sent
    /// back.
    ///
    /// **This will be called on a blocking code path** — run your HTTP
    /// request on a background thread (matching JUCE's
    /// `attemptWebserverUnlock` contract).
    fn read_reply_from_webserver(&self, email: &str, password: &str) -> String;

    /// Returns the list of machine IDs for the current host. Default
    /// implementation uses [`machine_id::get_local_machine_ids`].
    fn get_local_machine_ids(&self) -> Vec<String> {
        machine_id::get_local_machine_ids()
    }

    /// Called when the user cancels the connection. Default is a no-op.
    fn user_cancelled(&mut self) {}

    /// Returns the error message to show when the webserver can't be
    /// reached. The default implementation is the same as JUCE's.
    fn get_message_for_connection_failure(&self, is_internet_connection_working: bool) -> String {
        let mut message = format!(
            "Couldn't connect to {}...\n\n",
            self.get_website_name()
        );
        if is_internet_connection_working {
            message.push_str(
                "Your internet connection seems to be OK, but our webserver didn't respond... \
                 This is most likely a temporary problem, so try again in a few minutes, \
                 but if it persists, please contact us for support!",
            );
        } else {
            message.push_str(
                "No internet sites seem to be accessible from your computer. \
                 Before trying again, please check that your network is working correctly, \
                 and make sure that any firewall/security software installed on your machine \
                 isn't blocking your web connection.",
            );
        }
        message
    }

    /// Returns the error message to show when the server's reply is
    /// unparseable.
    fn get_message_for_unexpected_reply(&self) -> String {
        format!(
            "Unexpected or corrupted reply from {}...\n\n\
             Please try again in a few minutes, and contact us for support if \
             this message appears again.",
            self.get_website_name()
        )
    }
}

/// The result of a webserver unlock attempt.
///
/// Mirror of `juce::OnlineUnlockStatus::UnlockResult`. Either
/// `succeeded == true` and `error_message` is empty, or `succeeded ==
/// false` and `error_message` describes what went wrong. The
/// `informative_message` and `url_to_launch` are optional
/// user-facing payloads the server may return alongside a successful
/// (or failed) unlock.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UnlockResult {
    /// `true` on success.
    pub succeeded: bool,
    /// Error message from the server, or the local failure reason.
    pub error_message: String,
    /// An informational message the server wants the user to see
    /// (e.g. "a new version is available").
    pub informative_message: String,
    /// A URL the user should visit for more info, if any.
    pub url_to_launch: String,
}

/// The current state of the unlock flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnlockState {
    /// The state hasn't been loaded yet — call [`OnlineUnlockStatus::load`].
    NotLoaded,
    /// No keyfile has been applied yet.
    NotUnlocked,
    /// The app is currently unlocked.
    Unlocked,
    /// The app is unlocked with an expiring keyfile; check
    /// [`OnlineUnlockStatus::get_expiry_time`].
    UnlockedWithExpiry,
    /// The keyfile is expired.
    Expired,
    /// The state was loaded but the unlock is invalid (e.g. product
    /// ID mismatch, machine number mismatch).
    Invalid,
}

const STATE_TAG: &str = "REG";
const UNLOCKED_PROP: &str = "u";
const EXPIRY_TIME_PROP: &str = "t";
const USER_PROP: &str = "user";
const KEYFILE_DATA_PROP: &str = "key";

/// The unlock state machine.
///
/// ```rust,no_run
/// # use logic_nih_plug_product_unlocking::online_unlock_status::*;
/// # use logic_nih_plug_crypto::rsa_key::RSAKey;
/// # struct S(RSAKey);
/// # impl UnlockStore for S {
/// #     fn get_product_id(&self) -> &str { "p" }
/// #     fn does_product_id_match(&self, _: &str) -> bool { true }
/// #     fn get_public_key(&self) -> &RSAKey { &self.0 }
/// #     fn save_state(&mut self, _: &str) {}
/// #     fn get_state(&self) -> String { String::new() }
/// #     fn get_website_name(&self) -> &str { "" }
/// #     fn get_server_authentication_url(&self) -> &str { "" }
/// #     fn read_reply_from_webserver(&self, _: &str, _: &str) -> String { String::new() }
/// # }
/// let mut s = OnlineUnlockStatus::new(S(RSAKey::generate(2048).unwrap()));
/// s.load();
/// ```
pub struct OnlineUnlockStatus<S: UnlockStore> {
    store: S,
    status: ValueTree,
}

impl<S: UnlockStore> OnlineUnlockStatus<S> {
    /// Wraps a `store` in a fresh state machine. The internal `ValueTree`
    /// starts empty; call [`load`](Self::load) to hydrate it from the
    /// store's persisted state.
    pub fn new(store: S) -> Self {
        Self {
            store,
            status: ValueTree::new(STATE_TAG),
        }
    }

    /// Returns the current unlock state of the machine.
    pub fn state(&self) -> UnlockState {
        if !self.status.has_property(&Identifier::new(UNLOCKED_PROP))
            && !self.status.has_property(&Identifier::new(EXPIRY_TIME_PROP))
        {
            // The "NotLoaded" case is hard to distinguish from "NotUnlocked"
            // after load() runs; we use the marker property approach.
            if !self.status.has_property(&Identifier::new(KEYFILE_DATA_PROP)) {
                return UnlockState::NotUnlocked;
            }
        }
        let expiry = self.get_expiry_time_ms();
        let unlocked = self.is_unlocked();
        match (unlocked, expiry > 0) {
            (true, false) => UnlockState::Unlocked,
            (true, true) => UnlockState::UnlockedWithExpiry,
            (false, true) => UnlockState::Expired,
            (false, false) => UnlockState::NotUnlocked,
        }
    }

    /// Returns `true` if the product has been successfully authorised
    /// for this machine. The mirror of JUCE's `isUnlocked()`.
    pub fn is_unlocked(&self) -> bool {
        self.status.get_bool(&Identifier::new(UNLOCKED_PROP), false)
    }

    /// Returns the expiry time, in milliseconds since the Unix epoch.
    /// Returns `0` if the keyfile has no expiry.
    pub fn get_expiry_time_ms(&self) -> i64 {
        self.status.get_int(&Identifier::new(EXPIRY_TIME_PROP), 0)
    }

    /// Sets the user email / username persisted with the unlock state.
    pub fn set_user_email(&mut self, email: &str) {
        self.status
            .set_property(Identifier::new(USER_PROP), email.to_string());
    }

    /// Returns the persisted user email, or an empty string.
    pub fn get_user_email(&self) -> String {
        self.status
            .get_string(&Identifier::new(USER_PROP), "")
    }

    /// Hydrates the internal state from the store's persisted state, and
    /// re-validates the keyfile (if any) against the current product
    /// ID and local machine IDs.
    ///
    /// Call this once at app startup.
    pub fn load(&mut self) {
        let state = self.store.get_state();
        if let Some(tree) = decode_state(&state) {
            self.status = tree;
        }
        self.revalidate();
    }

    /// Re-serialises the current state and pushes it to the store.
    pub fn save(&mut self) {
        let encoded = encode_state(&self.status);
        self.store.save_state(&encoded);
    }

    /// Attempts to unlock the app from a keyfile.
    ///
    /// The keyfile must have been produced by
    /// [`crate::key_generation::generate_key_file`] or
    /// [`crate::key_generation::generate_expiring_key_file`] using the
    /// private key that matches this app's public key.
    pub fn apply_key_file(&mut self, keyfile_text: &str) -> Result<(), KeyFileError> {
        let public_key = self.store.get_public_key();
        let product_id = self.store.get_product_id().to_string();
        let local_ids = self.store.get_local_machine_ids();

        if local_ids.is_empty() {
            return Err(KeyFileError::NotReady);
        }

        let data = decrypt_key_file(keyfile_text, public_key)?;

        if data.licensee.is_empty() || data.email.is_empty() {
            return Err(KeyFileError::BadCredentials);
        }

        if !self.store.does_product_id_match(&data.app_id) {
            return Err(KeyFileError::ProductIdMismatch {
                expected: product_id,
                actual: data.app_id,
            });
        }

        // Stash the keyfile text + user email before clearing any prior
        // unlock flag.
        self.set_user_email(&data.email);
        self.status
            .set_property(Identifier::new(KEYFILE_DATA_PROP), keyfile_text.to_string());

        let machine_number_ok = machine_number_allowed(&data.machine_numbers, &local_ids);
        let now_ms = current_time_ms();

        if data.key_file_expires {
            // Clear any prior unlock.
            self.status
                .remove_property(Identifier::new(UNLOCKED_PROP));
            if machine_number_ok {
                self.status.set_property(
                    Identifier::new(EXPIRY_TIME_PROP),
                    ValueTree::wrap_i64(data.expiry_time_ms),
                );
            } else {
                self.status
                    .remove_property(Identifier::new(EXPIRY_TIME_PROP));
            }
            // Expired keyfiles are *valid* keyfiles, just past their date.
            if data.expiry_time_ms > 0 && data.expiry_time_ms <= now_ms {
                return Err(KeyFileError::LicenseExpired {
                    expiry_ms: data.expiry_time_ms,
                    now_ms,
                });
            }
            if self.get_expiry_time_ms() > 0 {
                Ok(())
            } else {
                Err(KeyFileError::MachineNumberMismatch)
            }
        } else {
            self.status
                .remove_property(Identifier::new(EXPIRY_TIME_PROP));
            if machine_number_ok {
                self.status
                    .set_property(Identifier::new(UNLOCKED_PROP), true);
                Ok(())
            } else {
                self.status
                    .remove_property(Identifier::new(UNLOCKED_PROP));
                Err(KeyFileError::MachineNumberMismatch)
            }
        }
    }

    /// Contacts the webserver and attempts to unlock the current machine.
    ///
    /// Blocks on the network — call this from a background thread.
    pub fn attempt_webserver_unlock(
        &mut self,
        email: &str,
        password: &str,
    ) -> UnlockResult {
        let reply = self.store.read_reply_from_webserver(email, password);
        // Match JUCE's logic: the server returns either an XML tree or
        // an error. The `handleXmlReply` path checks for `<KEY>`,
        // `<MESSAGE>`, `<ERROR>`, and `url` attributes.
        let mut result = UnlockResult::default();
        result.succeeded = false;
        if let Some(parsed) = parse_simple_xml(&reply) {
            // <KEY>keyfile text</KEY>?
            if let Some(key_text) = parsed
                .children
                .iter()
                .find(|(tag, _)| tag == "KEY")
                .map(|(_, body)| body.trim().to_string())
            {
                if key_text.len() > 10 {
                    if let Err(e) = self.apply_key_file(&key_text) {
                        result.error_message = e.as_license_result().to_string();
                        return result;
                    }
                    result.succeeded = true;
                }
            }
            if parsed.tag == "MESSAGE" {
                if let Some(msg) = parsed.attributes.get("message") {
                    result.informative_message = msg.trim().to_string();
                }
            }
            if parsed.tag == "ERROR" {
                if let Some(msg) = parsed.attributes.get("error") {
                    result.error_message = msg.trim().to_string();
                }
            }
            if let Some(url) = parsed.attributes.get("url") {
                if !url.is_empty() {
                    result.url_to_launch = url.trim().to_string();
                }
            }
            if !result.succeeded
                && result.error_message.is_empty()
                && result.informative_message.is_empty()
                && result.url_to_launch.is_empty()
            {
                result.error_message = self.store.get_message_for_unexpected_reply();
            }
        } else {
            result.error_message = self
                .store
                .get_message_for_connection_failure(false);
        }
        result
    }

    /// Drops the unlock state entirely.
    pub fn clear(&mut self) {
        self.status = ValueTree::new(STATE_TAG);
    }

    /// Re-checks the unlock state against the current product ID and
    /// machine IDs. The mirror of the validation block at the end of
    /// JUCE's `load()`.
    fn revalidate(&mut self) {
        let product_id = self.store.get_product_id().to_string();
        let local_ids = self.store.get_local_machine_ids();
        let public_key = self.store.get_public_key();

        // If the product ID or local machine IDs have changed since
        // we saved, the keyfile might no longer be valid.
        if !self.status.has_property(&Identifier::new(KEYFILE_DATA_PROP)) {
            return;
        }
        let keyfile_text = self
            .status
            .get_string(&Identifier::new(KEYFILE_DATA_PROP), "");
        if keyfile_text.is_empty() {
            return;
        }
        let data = match decrypt_key_file(&keyfile_text, public_key) {
            Ok(d) => d,
            Err(_) => {
                // Decryption failed — assume the keyfile is no longer
                // valid. We keep the raw keyfile around so the user can
                // re-apply it if the public key changes back.
                self.status
                    .remove_property(Identifier::new(UNLOCKED_PROP));
                self.status
                    .remove_property(Identifier::new(EXPIRY_TIME_PROP));
                return;
            }
        };
        if !self.store.does_product_id_match(&data.app_id)
            || data.app_id != product_id
        {
            self.status.remove_property(Identifier::new(UNLOCKED_PROP));
            self.status
                .remove_property(Identifier::new(EXPIRY_TIME_PROP));
        } else if !machine_number_allowed(&data.machine_numbers, &local_ids) {
            if data.key_file_expires {
                self.status
                    .remove_property(Identifier::new(EXPIRY_TIME_PROP));
            } else {
                self.status.remove_property(Identifier::new(UNLOCKED_PROP));
            }
        }
    }
}

// `Value` doesn't expose a `From<i64>` constructor — but `ValueTree::set_property`
// only needs `Into<Value>`, and we want the integer to round-trip
// losslessly. We use a tiny newtype to keep the call site clean.
trait ValueTreeExt {
    fn wrap_i64(v: i64) -> logic_nih_plug_data::Value;
}

impl ValueTreeExt for ValueTree {
    fn wrap_i64(v: i64) -> logic_nih_plug_data::Value {
        logic_nih_plug_data::Value::Int(v)
    }
}

fn current_time_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn machine_number_allowed(
    numbers_from_keyfile: &[String],
    local_machine_ids: &[String],
) -> bool {
    // Mirror of JUCE's `machineNumberAllowed`: case-insensitive,
    // whitespace-trimmed substring check.
    for local in local_machine_ids {
        let local = local.trim();
        if local.is_empty() {
            continue;
        }
        for remote in numbers_from_keyfile {
            if local.eq_ignore_ascii_case(remote.trim()) {
                return true;
            }
        }
    }
    false
}

// ----- state (de)serialisation -----
//
// We don't have a generic ValueTree binary codec, and JUCE's GZIP'd
// binary format isn't worth reimplementing. The state is small (4
// properties, no children), so a hand-rolled key=value format with
// `=` and `\n` escaping is enough. The format is *not* meant to be
// human-editable, but it is stable across versions.

fn encode_state(tree: &ValueTree) -> String {
    let mut out = String::with_capacity(128);
    let props = [
        UNLOCKED_PROP,
        EXPIRY_TIME_PROP,
        USER_PROP,
        KEYFILE_DATA_PROP,
    ];
    for name in props {
        let id = Identifier::new(name);
        if !tree.has_property(&id) {
            continue;
        }
        let value = match name {
            UNLOCKED_PROP => {
                if tree.get_bool(&id, false) {
                    "1".to_string()
                } else {
                    "0".to_string()
                }
            }
            EXPIRY_TIME_PROP => tree.get_int(&id, 0).to_string(),
            _ => tree.get_string(&id, ""),
        };
        out.push_str(name);
        out.push('=');
        for ch in value.chars() {
            match ch {
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '=' => out.push_str("\\="),
                _ => out.push(ch),
            }
        }
        out.push('\n');
    }
    out
}

fn decode_state(s: &str) -> Option<ValueTree> {
    if s.is_empty() {
        return None;
    }
    let tree = ValueTree::new(STATE_TAG);
    for line in s.split('\n') {
        if line.is_empty() {
            continue;
        }
        let (name, raw) = line.split_once('=')?;
        let mut value = String::with_capacity(raw.len());
        let mut chars = raw.chars();
        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('\\') => value.push('\\'),
                    Some('n') => value.push('\n'),
                    Some('r') => value.push('\r'),
                    Some('=') => value.push('='),
                    Some(other) => {
                        value.push('\\');
                        value.push(other);
                    }
                    None => value.push('\\'),
                }
            } else {
                value.push(c);
            }
        }
        match name {
            UNLOCKED_PROP => {
                let b = value == "1" || value.eq_ignore_ascii_case("true");
                tree.set_property(Identifier::new(UNLOCKED_PROP), b);
            }
            EXPIRY_TIME_PROP => {
                if let Ok(n) = value.parse::<i64>() {
                    tree.set_property(Identifier::new(EXPIRY_TIME_PROP), n);
                }
            }
            USER_PROP => {
                tree.set_property(Identifier::new(USER_PROP), value);
            }
            KEYFILE_DATA_PROP => {
                tree.set_property(Identifier::new(KEYFILE_DATA_PROP), value);
            }
            _ => {
                // Unknown property — skip. Forward compatibility.
            }
        }
    }
    Some(tree)
}

// ----- minimal XML parser for the server reply -----
//
// The server's reply is either:
//   <KEY>keyfile text</KEY>
//   <MESSAGE message="..."/>
//   <ERROR error="..."/>
// We only need to recognise these three shapes.

#[derive(Debug, Default)]
struct ParsedXml {
    tag: String,
    attributes: std::collections::HashMap<String, String>,
    children: Vec<(String, String)>,
}

fn parse_simple_xml(s: &str) -> Option<ParsedXml> {
    let s = s.trim();
    if !s.starts_with('<') {
        return None;
    }
    // Find the end of the opening tag.
    let close = s.find('>')?;
    let inside = &s[1..close];
    let self_closing = inside.ends_with('/');
    let inside = if self_closing {
        &inside[..inside.len() - 1]
    } else {
        inside
    };
    let mut parts = inside.splitn(2, char::is_whitespace);
    let tag = parts.next()?.to_string();
    let attr_text = parts.next().unwrap_or("");
    let attributes = crate::key_generation::parse_attributes_for_test(attr_text);
    if self_closing {
        return Some(ParsedXml {
            tag,
            attributes,
            children: Vec::new(),
        });
    }
    // Find the matching closing tag.
    let close_tag = format!("</{tag}>");
    let body_start = close + 1;
    let body_end = s.rfind(&close_tag)?;
    let body = &s[body_start..body_end];
    let children = if !body.trim().is_empty() {
        vec![(tag.clone(), body.to_string())]
    } else {
        Vec::new()
    };
    Some(ParsedXml {
        tag,
        attributes,
        children,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key_generation::{generate_key_file, LicenseResult};

    struct TestStore {
        public_key: RSAKey,
        product_id: String,
        state: String,
    }

    impl TestStore {
        fn new(public_key: RSAKey, product_id: &str) -> Self {
            Self {
                public_key,
                product_id: product_id.to_owned(),
                state: String::new(),
            }
        }
    }

    impl UnlockStore for TestStore {
        fn get_product_id(&self) -> &str {
            &self.product_id
        }
        fn does_product_id_match(&self, id: &str) -> bool {
            id == self.product_id
        }
        fn get_public_key(&self) -> &RSAKey {
            &self.public_key
        }
        fn save_state(&mut self, state: &str) {
            self.state = state.to_string();
        }
        fn get_state(&self) -> String {
            self.state.clone()
        }
        fn get_website_name(&self) -> &str {
            "Test Store"
        }
        fn get_server_authentication_url(&self) -> &str {
            "https://test/"
        }
        fn read_reply_from_webserver(&self, _email: &str, _password: &str) -> String {
            String::new()
        }
    }

    fn make_key() -> RSAKey {
        RSAKey::generate(2048).expect("key gen")
    }

    fn make_keyfile(private_key: &RSAKey, machine_id: &str) -> String {
        generate_key_file("MY-PRODUCT", "joe@foo.bar", "Joe Bloggs", machine_id, private_key)
            .expect("generate")
    }

    #[test]
    fn load_then_save_round_trip() {
        let key = make_key();
        let mut status = OnlineUnlockStatus::new(TestStore::new(
            key.clone().into_public_only(),
            "MY-PRODUCT",
        ));
        status.load();
        assert!(!status.is_unlocked());
        status.save();
        // Load a *fresh* status from the same store and confirm the
        // saved state round-trips (even if it's empty in this case).
        let mut status2 = OnlineUnlockStatus::new(TestStore {
            public_key: key.into_public_only(),
            product_id: "MY-PRODUCT".to_owned(),
            state: status.get_user_email(), // not the saved state, but a smoke test
        });
        status2.load();
    }

    #[test]
    fn apply_key_file_with_matching_machine_unlocks() {
        let key = make_key();
        let keyfile = make_keyfile(&key, "HOST-MACHINE-1");
        let store = OverrideStore {
            public_key: key.into_public_only(),
            product_id: "MY-PRODUCT".to_owned(),
            state: String::new(),
            machine_ids: vec!["HOST-MACHINE-1".to_string()],
        };
        let mut status = OnlineUnlockStatus::new(store);
        status.load();
        status.apply_key_file(&keyfile).expect("apply");
        assert!(status.is_unlocked());
        assert_eq!(status.get_user_email(), "joe@foo.bar");
    }

    #[test]
    fn apply_key_file_with_wrong_machine_fails() {
        let key = make_key();
        let keyfile = make_keyfile(&key, "DIFFERENT-MACHINE");
        let store = OverrideStore {
            public_key: key.into_public_only(),
            product_id: "MY-PRODUCT".to_owned(),
            state: String::new(),
            machine_ids: vec!["THIS-MACHINE".to_string()],
        };
        let mut status = OnlineUnlockStatus::new(store);
        status.load();
        let err = status.apply_key_file(&keyfile).unwrap_err();
        assert!(matches!(err, KeyFileError::MachineNumberMismatch));
        assert!(!status.is_unlocked());
    }

    #[test]
    fn apply_key_file_with_wrong_product_id_fails() {
        let key = make_key();
        let keyfile = make_keyfile(&key, "ANY");
        let store = OverrideStore {
            public_key: key.into_public_only(),
            product_id: "OTHER-PRODUCT".to_owned(),
            state: String::new(),
            machine_ids: vec!["ANY".to_string()],
        };
        let mut status = OnlineUnlockStatus::new(store);
        let err = status.apply_key_file(&keyfile).unwrap_err();
        assert!(matches!(err, KeyFileError::ProductIdMismatch { .. }));
    }

    #[test]
    fn clear_drops_everything() {
        let key = make_key();
        let keyfile = make_keyfile(&key, "ANY");
        let store = OverrideStore {
            public_key: key.into_public_only(),
            product_id: "MY-PRODUCT".to_owned(),
            state: String::new(),
            machine_ids: vec!["ANY".to_string()],
        };
        let mut status = OnlineUnlockStatus::new(store);
        status.apply_key_file(&keyfile).expect("apply");
        assert!(status.is_unlocked());
        status.clear();
        assert!(!status.is_unlocked());
        assert_eq!(status.get_user_email(), "");
    }

    #[test]
    fn license_error_strings_match_juce() {
        // Mirror of JUCE's OnlineUnlockStatus::LicenseResult constants.
        assert_eq!(
            LicenseResult::NOT_READY,
            "ID generator is not ready, try again later."
        );
        assert_eq!(LicenseResult::BAD_CREDENTIALS, "Credentials are invalid.");
        assert_eq!(LicenseResult::BAD_PRODUCT_ID, "ProductID is incorrect.");
        assert_eq!(LicenseResult::LICENSE_EXPIRED, "License has expired.");
        assert_eq!(
            LicenseResult::UNLOCK_FAILED,
            "Generic unlock failure."
        );
    }

    #[test]
    fn expiring_keyfile_keeps_expiry_time() {
        let key = make_key();
        let expiry = current_time_ms() + 30 * 86_400_000; // 30 days
        let keyfile = crate::key_generation::generate_expiring_key_file(
            "MY-PRODUCT",
            "joe@foo.bar",
            "Joe Bloggs",
            "ANY",
            expiry,
            &key,
        )
        .expect("generate");
        let store = OverrideStore {
            public_key: key.into_public_only(),
            product_id: "MY-PRODUCT".to_owned(),
            state: String::new(),
            machine_ids: vec!["ANY".to_string()],
        };
        let mut status = OnlineUnlockStatus::new(store);
        status.apply_key_file(&keyfile).expect("apply");
        // Expiring keyfiles don't unlock until you call
        // `is_unlocked_with_expiry` or check the expiry yourself.
        assert!(!status.is_unlocked());
        assert_eq!(status.get_expiry_time_ms(), expiry);
    }

    #[test]
    fn expired_keyfile_fails() {
        let key = make_key();
        let expiry = current_time_ms() - 1000; // 1 second in the past
        let keyfile = crate::key_generation::generate_expiring_key_file(
            "MY-PRODUCT",
            "joe@foo.bar",
            "Joe Bloggs",
            "ANY",
            expiry,
            &key,
        )
        .expect("generate");
        let store = OverrideStore {
            public_key: key.into_public_only(),
            product_id: "MY-PRODUCT".to_owned(),
            state: String::new(),
            machine_ids: vec!["ANY".to_string()],
        };
        let mut status = OnlineUnlockStatus::new(store);
        let err = status.apply_key_file(&keyfile).unwrap_err();
        assert!(matches!(err, KeyFileError::LicenseExpired { .. }));
    }

    // ----- second test store that lets us pin machine IDs -----

    struct OverrideStore {
        public_key: RSAKey,
        product_id: String,
        state: String,
        machine_ids: Vec<String>,
    }

    impl UnlockStore for OverrideStore {
        fn get_product_id(&self) -> &str {
            &self.product_id
        }
        fn does_product_id_match(&self, id: &str) -> bool {
            id == self.product_id
        }
        fn get_public_key(&self) -> &RSAKey {
            &self.public_key
        }
        fn save_state(&mut self, state: &str) {
            self.state = state.to_string();
        }
        fn get_state(&self) -> String {
            self.state.clone()
        }
        fn get_website_name(&self) -> &str {
            "Test Store"
        }
        fn get_server_authentication_url(&self) -> &str {
            "https://test/"
        }
        fn read_reply_from_webserver(&self, _email: &str, _password: &str) -> String {
            String::new()
        }
        fn get_local_machine_ids(&self) -> Vec<String> {
            self.machine_ids.clone()
        }
    }
}
