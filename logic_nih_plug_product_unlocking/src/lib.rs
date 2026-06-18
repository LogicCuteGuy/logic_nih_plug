//! # logic_nih_plug_product_unlocking
//!
//! Product unlocking (keyfile generation + online unlock status state machine)
//! ported from JUCE for nih-plug.
//!
//! This crate provides pure-Rust implementations of JUCE's
//! [`juce_product_unlocking`](https://docs.juce.com/master/group__juce__product__unlocking.html)
//! module:
//!
//! - **[`key_generation`]** — `KeyGeneration`-style helpers that mint the
//!   human-readable keyfile text your server sends to a user to unlock a
//!   product. Mirrors the [JUCE `KeyGeneration`][juce-keygen] class.
//! - **[`online_unlock_status`]** — `OnlineUnlockStatus`-style state machine
//!   that runs on the client, verifies a keyfile against the local machine
//!   IDs, and persists its unlock state through your app's settings.
//!   Mirrors the [JUCE `OnlineUnlockStatus`][juce-status] class.
//! - **[`machine_id`]** — the `MachineIDUtilities` helpers (platform
//!   prefix, encoded machine IDs).
//!
//! [juce-keygen]: https://docs.juce.com/master/classjuce_1_1KeyGeneration.html
//! [juce-status]: https://docs.juce.com/master/classjuce_1_1OnlineUnlockStatus.html
//!
//! ## Feature flags
//!
//! | Feature | Default | What it adds |
//! |---|---|---|
//! | `key_generation` | ✅ | [`key_generation::generate_key_file`], [`key_generation::generate_expiring_key_file`], [`key_generation::decrypt_key_file`], [`key_generation::KeyFileData`] |
//! | `online_unlock_status` | ✅ | [`online_unlock_status::OnlineUnlockStatus`], [`online_unlock_status::UnlockStore`] trait, [`online_unlock_status::UnlockResult`] |
//! | `full` | — | All of the above |
//!
//! ## Threading
//!
//! `OnlineUnlockStatus` is a *single-threaded* state machine — its internal
//! `ValueTree` doesn't have interior mutability. The expected workflow is
//! to keep one `OnlineUnlockStatus` instance around for the duration of
//! your app and call `load()` at startup and `save()` whenever the state
//! changes. `attempt_webserver_unlock()` blocks on the network, so
//! (matching JUCE) you should run it on a background thread, not on the
//! GUI thread.
//!
//! ## Example
//!
//! ```rust,no_run
//! use logic_nih_plug_product_unlocking::key_generation::{
//!     decrypt_key_file, generate_key_file,
//! };
//! use logic_nih_plug_crypto::rsa_key::RSAKey;
//!
//! // Server side: mint a keyfile with your private key.
//! let private_key = RSAKey::generate(2048).expect("key generation");
//! let keyfile_text = generate_key_file(
//!     "MyApp",         // app name
//!     "joe@foo.bar",   // user email
//!     "Joe Bloggs",    // user name
//!     "MACHINEID123",  // machine ID
//!     &private_key,
//! )
//! .expect("keyfile generation");
//!
//! // Client side: load the public key, decrypt, check machine ID.
//! let public_key = private_key.clone().into_public_only();
//! let data = decrypt_key_file(&keyfile_text, &public_key).expect("decrypt");
//! assert_eq!(data.licensee, "Joe Bloggs");
//! assert_eq!(data.machine_numbers, vec!["MACHINEID123".to_string()]);
//! ```

#![warn(missing_docs)]

pub mod error;
pub mod key_generation;
pub mod machine_id;

#[cfg(feature = "online_unlock_status")]
pub mod online_unlock_status;

pub use error::KeyFileError;
