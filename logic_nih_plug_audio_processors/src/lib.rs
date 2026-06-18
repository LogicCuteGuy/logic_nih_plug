//! # logic_nih_plug_audio_processors
//!
//! Host-side plugin discovery, scanning, and management ported from
//! [JUCE's `juce_audio_processors` module](https://docs.juce.com/master/juce_audio_processors_README.html)
//! for the `logic_nih_plug` ecosystem.
//!
//! ## What's inside
//!
//! - [`PluginDescription`] — immutable metadata about a discovered plugin
//!   (name, manufacturer, version, format, file path, unique IDs, channel
//!   counts). Serializable to JSON for caching.
//! - [`PluginFormat`] — trait for format-specific scanning and loading
//!   (VST3, CLAP, AU, LV2, …). Implement this for each plugin API.
//! - [`PluginFormatType`] — enum of known plugin formats with platform
//!   detection and standard search-path helpers.
//! - [`KnownPluginList`] — persistent registry of [`PluginDescription`]
//!   entries. Deduplicates, sorts, and serializes to JSON. Provides
//!   change-notification via a callback.
//! - [`PluginDirectoryScanner`] — incremental directory scanner. Feeds
//!   discovered files to a [`PluginFormat`] and populates a
//!   [`KnownPluginList`] with progress reporting.
//!
//! ## Feature flags
//!
//! | Flag          | Default | What it gates                                                    |
//! |---------------|---------|------------------------------------------------------------------|
//! | `description` | ✅      | [`PluginDescription`], [`PluginFormat`], [`PluginFormatType`]     |
//! | `list`        | ✅      | [`KnownPluginList`]                                              |
//! | `scanner`     | —       | [`PluginDirectoryScanner`] (requires `list`)                     |
//! | `full`        | —       | All of the above                                                  |
//!
//! ## Example
//!
//! ```rust
//! use logic_nih_plug_audio_processors::{
//!     PluginDescription, PluginFormatType,
//! };
//!
//! let desc = PluginDescription {
//!     name: "My Synth".into(),
//!     manufacturer_name: "Acme".into(),
//!     version: "1.0.0".into(),
//!     format: PluginFormatType::Vst3,
//!     unique_id: "com.acme.mysynth".into(),
//!     ..PluginDescription::default()
//! };
//!
//! assert_eq!(desc.format_name(), "VST3");
//! assert_eq!(desc.identifier_string(), "VST3:com.acme.mysynth");
//! ```

#![warn(missing_docs)]

mod description;
mod error;
mod format;
mod list;
#[cfg(feature = "scanner")]
mod scanner;

pub use description::PluginDescription;
pub use error::{AudioProcessorsError, AudioProcessorsResult};
pub use format::{PluginFormat, PluginFormatType, NullPluginFormat};
pub use list::{KnownPluginList, PluginListSortMethod};

#[cfg(feature = "scanner")]
pub use scanner::PluginDirectoryScanner;
