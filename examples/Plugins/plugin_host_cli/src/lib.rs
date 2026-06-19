//! # plugin_host_cli
//!
//! A headless plugin host CLI that scans a directory for plugins and
//! prints the discovered [`PluginDescription`]s to stdout.
//!
//! Mirrors `examples/Plugins/HostPluginDemo.h` from JUCE. The
//! headless CLI variant demonstrates that
//! `logic_nih_plug_audio_processors::PluginDirectoryScanner` works
//! end-to-end without any GUI dependencies, making it suitable for
//! CI smoke tests.
//!
//! ## What to learn from this example
//!
//! - How to implement [`logic_nih_plug_audio_processors::PluginFormat`]
//!   for a custom format (or use [`NullPluginFormat`] for tests).
//! - How to drive [`PluginDirectoryScanner`] to scan one or more
//!   directories.
//! - How to populate a [`KnownPluginList`] from the scanner's output.
//!
//! ## Running
//!
//! ```bash
//! cargo run -p plugin_host_cli -- ./test-vst3/
//! ```

use std::path::{Path, PathBuf};

use logic_nih_plug_audio_processors::{
    KnownPluginList, NullPluginFormat, PluginDescription, PluginDirectoryScanner,
    PluginFormat, PluginFormatType,
};

/// A test-friendly [`PluginFormat`] that **pretends** to find a plugin
/// in every file with the matching extension. Used by the doc-test
/// (T050) and integration tests to drive the scanner without
/// requiring a real `.vst3` / `.clap` bundle on disk.
pub struct MockPluginFormat {
    /// Which plugin format this mock represents.
    format: PluginFormatType,
    /// Plugin name to return.
    name: String,
    /// Manufacturer to return.
    manufacturer: String,
}

impl MockPluginFormat {
    /// Create a mock that returns one plugin description per scanned
    /// file (with the format's expected file extension).
    pub fn new(format: PluginFormatType) -> Self {
        Self {
            format,
            name: "Mock Plugin".to_string(),
            manufacturer: "LogicCuteGuy (test)".to_string(),
        }
    }
}

impl PluginFormat for MockPluginFormat {
    fn format_type(&self) -> PluginFormatType {
        self.format
    }

    fn find_plugins_in_file(&self, path: &Path) -> Vec<PluginDescription> {
        // The scanner only calls us for files whose extension matches
        // (via `file_might_contain_plugin`), so any file we receive
        // here is a candidate — return one stub description per call.
        vec![PluginDescription {
            name: self.name.clone(),
            manufacturer_name: self.manufacturer.clone(),
            version: "0.1.0".to_string(),
            format: self.format,
            unique_id: format!("mock.{}", path.display()),
            file_or_identifier: path.to_string_lossy().to_string(),
            last_file_mod_time: 0,
            category: "Mock".to_string(),
            is_instrument: false,
            ..PluginDescription::default()
        }]
    }
}

/// Scan one or more directories for plugins of the given format.
///
/// - `format` — which plugin format to scan for
/// - `directories` — directories to scan (non-recursive by default)
/// - `recursive` — whether to descend into subdirectories
///
/// Returns the populated [`KnownPluginList`].
pub fn scan_directories(
    format: &dyn PluginFormat,
    directories: &[PathBuf],
    recursive: bool,
) -> KnownPluginList {
    let mut list = KnownPluginList::new();
    let mut scanner = PluginDirectoryScanner::new(format, directories, recursive);
    while let Some(descs) = scanner.scan_next_file(false) {
        for desc in descs {
            list.add_type(desc);
        }
    }
    list
}

/// Format a [`KnownPluginList`] as a human-readable string, one line per
/// plugin.
pub fn format_plugin_list(list: &KnownPluginList) -> String {
    let mut out = String::new();
    for desc in list.get_types() {
        out.push_str(&format!(
            "  • {} ({} v{}) — {}\n",
            desc.name,
            desc.manufacturer_name,
            desc.version,
            desc.format.name()
        ));
    }
    out
}

/// Scan the given directory using the [`NullPluginFormat`] (returns
/// no plugins, so the result is always empty). Useful as a
/// "no plugins discovered" smoke test.
pub fn scan_with_null_format(directories: &[PathBuf]) -> KnownPluginList {
    let format = NullPluginFormat::new(PluginFormatType::Vst3);
    scan_directories(&format, directories, false)
}

/// Create a fixtures directory in a temporary location with a single
/// dummy `.vst3` file inside. Returns the directory path. Used by the
/// T050 doc-test.
pub fn make_fixture_dir_with_vst3() -> PathBuf {
    let dir = std::env::temp_dir().join("plugin_host_cli_fixtures");
    let _ = std::fs::create_dir_all(&dir);
    let file = dir.join("MockPlugin.vst3");
    // Touch the file (empty is fine — the scanner only checks
    // extension, not file content).
    let _ = std::fs::write(&file, b"");
    dir
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_plugin_format_returns_one_description_per_file() {
        let fmt = MockPluginFormat::new(PluginFormatType::Vst3);
        let descs = fmt.find_plugins_in_file(Path::new("/tmp/foo.vst3"));
        assert_eq!(descs.len(), 1);
        assert_eq!(descs[0].name, "Mock Plugin");
        assert_eq!(descs[0].format, PluginFormatType::Vst3);
    }

    #[test]
    fn scanner_finds_fixture_plugin() {
        let dir = make_fixture_dir_with_vst3();
        let fmt = MockPluginFormat::new(PluginFormatType::Vst3);
        let list = scan_directories(&fmt, &[dir], false);
        assert!(
            !list.get_types().is_empty(),
            "expected the scanner to find at least 1 plugin in the fixtures dir"
        );
    }

    #[test]
    fn null_format_returns_empty_list() {
        let dir = make_fixture_dir_with_vst3();
        let list = scan_with_null_format(&[dir]);
        assert!(list.get_types().is_empty());
    }

    #[test]
    fn format_plugin_list_includes_names() {
        let mut list = KnownPluginList::new();
        list.add_type(PluginDescription {
            name: "Test Plugin".into(),
            format: PluginFormatType::Clap,
            unique_id: "test.plugin.12345".into(),
            manufacturer_name: "TestMfg".into(),
            version: "1.0.0".into(),
            ..PluginDescription::default()
        });
        let formatted = format_plugin_list(&list);
        assert!(formatted.contains("Test Plugin"));
        assert!(formatted.contains("CLAP"));
        assert!(formatted.contains("1.0.0"));
    }
}
