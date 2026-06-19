//! # juce_audio_plugin_host_egui
//!
//! A plugin host example, with an `egui`-based editor, that scans a
//! directory for plugins, loads one, and lets the user adjust its
//! parameters.
//!
//! This crate is also the host for the **T051 integration test**:
//! `MockAudioIODevice` is wired as the audio I/O backend (per Q4
//! recommendation), and a sine is routed through the loaded plugin
//! so the test can assert that audio flows end-to-end.
//!
//! ## What this example ports
//!
//! - **JUCE source file**: `examples/Plugins/HostPluginDemo.h`
//!
//! ## Architecture
//!
//! The host is split into three modules:
//!
//! - [`host`] — the audio engine: holds the loaded plugin, runs
//!   `process()` on each audio buffer, and forwards parameter
//!   changes back to the editor.
//! - [`editor`] — the `egui` UI: a scan button, a plugin list, a
//!   parameter slider, and a state save/load button pair.
//! - [`scanner`] — wraps `PluginDirectoryScanner` for the host.
//!
//! ## Running
//!
//! ```bash
//! cargo run -p juce_audio_plugin_host_egui --features standalone -- ./test-vst3/
//! ```

pub mod editor;
pub mod host;
pub mod scanner;

/// Re-exports for tests.
pub use host::{AudioHost, HostConfig};
pub use scanner::{scan_for_plugins, HostScanner};
