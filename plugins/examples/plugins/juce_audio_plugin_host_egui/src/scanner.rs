//! Plugin scanner for the host.
//!
//! Wraps `logic_nih_plug_audio_processors::PluginDirectoryScanner` with
//! a host-friendly API.

use std::path::PathBuf;

use logic_nih_plug_audio_processors::{
    KnownPluginList, NullPluginFormat, PluginDescription, PluginDirectoryScanner,
    PluginFormat, PluginFormatType,
};

/// The host's scanner — a thin wrapper around the framework's scanner.
pub struct HostScanner {
    /// Which format to scan for. Defaults to `Vst3`.
    pub format: PluginFormatType,
    /// Whether to recurse into subdirectories.
    pub recursive: bool,
}

impl Default for HostScanner {
    fn default() -> Self {
        Self {
            format: PluginFormatType::Vst3,
            recursive: false,
        }
    }
}

impl HostScanner {
    /// Create a scanner for the given format.
    pub fn new(format: PluginFormatType) -> Self {
        Self {
            format,
            recursive: false,
        }
    }

    /// Set recursion.
    pub fn recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
    }

    /// Scan a single directory and return the discovered plugins.
    pub fn scan(&self, directory: &PathBuf) -> KnownPluginList {
        scan_for_plugins(&NullPluginFormat::new(self.format), directory, self.recursive)
    }
}

/// Scan a directory using a custom [`PluginFormat`] implementation.
/// The test suite uses a stub format that returns synthetic plugins;
/// production code would use the real VST3/CLAP format.
pub fn scan_for_plugins(
    format: &dyn PluginFormat,
    directory: &PathBuf,
    recursive: bool,
) -> KnownPluginList {
    let mut list = KnownPluginList::new();
    if !directory.exists() {
        return list;
    }
    let mut scanner = PluginDirectoryScanner::new(format, std::slice::from_ref(directory), recursive);
    while let Some(descs) = scanner.scan_next_file(false) {
        for desc in descs {
            list.add_type(desc);
        }
    }
    list
}

/// A stub plugin format that returns one fake plugin per file. Used
/// by the T051 integration test to simulate a discovered plugin
/// without needing a real VST3/CLAP bundle.
pub struct StubPluginFormat {
    /// Display name to use for the fake plugin.
    pub name: String,
    /// Manufacturer to attribute the fake plugin to.
    pub manufacturer: String,
}

impl Default for StubPluginFormat {
    fn default() -> Self {
        Self {
            name: "Stub Plugin".to_string(),
            manufacturer: "LogicCuteGuy (test)".to_string(),
        }
    }
}

impl StubPluginFormat {
    /// Create a stub format that returns one fake plugin per file.
    pub fn new() -> Self {
        Self::default()
    }

    /// Return one stub `PluginDescription` per file.
    pub fn descriptions_for(&self, dir: &PathBuf) -> Vec<PluginDescription> {
        let mut descs = Vec::new();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "vst3" || ext == "clap" {
                        descs.push(PluginDescription {
                            name: self.name.clone(),
                            manufacturer_name: self.manufacturer.clone(),
                            version: "0.1.0".to_string(),
                            format: PluginFormatType::Vst3,
                            unique_id: format!("stub.{}", path.display()),
                            file_or_identifier: path.to_string_lossy().to_string(),
                            last_file_mod_time: 0,
                            category: "Stub".to_string(),
                            is_instrument: false,
                            ..PluginDescription::default()
                        });
                    }
                }
            }
        }
        descs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_returns_empty_for_missing_directory() {
        let scanner = HostScanner::new(PluginFormatType::Vst3);
        let list = scanner.scan(&PathBuf::from("/this/does/not/exist"));
        assert!(list.get_types().is_empty());
    }

    #[test]
    fn stub_plugin_format_descriptions_for_fixtures() {
        let dir = std::env::temp_dir().join("juce_audio_plugin_host_egui_fixtures");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("Stub.vst3"), b"").unwrap();
        let descs = StubPluginFormat::new().descriptions_for(&dir);
        assert_eq!(descs.len(), 1);
        assert_eq!(descs[0].name, "Stub Plugin");
    }
}
