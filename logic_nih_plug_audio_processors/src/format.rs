//! [`PluginFormat`] trait and [`PluginFormatType`] enum for format-specific
//! scanning and loading.

use std::path::{Path, PathBuf};

use crate::description::PluginDescription;

/// Enumeration of known plugin formats with platform detection.
///
/// Each variant maps to one plugin API. The `Unknown` variant is a
/// fallback for unrecognised formats.
///
/// # Example
///
/// ```rust
/// use logic_nih_plug_audio_processors::PluginFormatType;
///
/// assert_eq!(PluginFormatType::Vst3.name(), "VST3");
/// assert!(PluginFormatType::Vst3.is_supported_on_current_platform());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum PluginFormatType {
    /// VST3 ( Steinberg )
    Vst3 = 0,
    /// CLAP (Audio Unit replacement / open standard)
    Clap = 1,
    /// Audio Unit (macOS / iOS only)
    Au = 2,
    /// LV2 (open standard)
    Lv2 = 3,
    /// VST2 (legacy, Steinberg)
    Vst2 = 4,
    /// AAX (Avid)
    Aax = 5,
    /// LADSPA (Linux)
    Ladspa = 6,
    /// Unrecognised format.
    Unknown = 255,
}

impl PluginFormatType {
    /// Human-readable format name.
    ///
    /// ```rust
    /// # use logic_nih_plug_audio_processors::PluginFormatType;
    /// assert_eq!(PluginFormatType::Clap.name(), "CLAP");
    /// assert_eq!(PluginFormatType::Unknown.name(), "Unknown");
    /// ```
    pub fn name(self) -> &'static str {
        match self {
            Self::Vst3 => "VST3",
            Self::Clap => "CLAP",
            Self::Au => "AU",
            Self::Lv2 => "LV2",
            Self::Vst2 => "VST2",
            Self::Aax => "AAX",
            Self::Ladspa => "LADSPA",
            Self::Unknown => "Unknown",
        }
    }

    /// Parse a format name (case-insensitive) back to the enum variant.
    ///
    /// ```rust
    /// # use logic_nih_plug_audio_processors::PluginFormatType;
    /// assert_eq!(PluginFormatType::from_name("vst3"), PluginFormatType::Vst3);
    /// assert_eq!(PluginFormatType::from_name("clap"), PluginFormatType::Clap);
    /// assert_eq!(PluginFormatType::from_name("bogus"), PluginFormatType::Unknown);
    /// ```
    pub fn from_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "vst3" => Self::Vst3,
            "clap" => Self::Clap,
            "au" | "audiounit" | "audio unit" => Self::Au,
            "lv2" => Self::Lv2,
            "vst2" | "vst" => Self::Vst2,
            "aax" => Self::Aax,
            "ladspa" => Self::Ladspa,
            _ => Self::Unknown,
        }
    }

    /// File extension(s) associated with this format.
    ///
    /// Returns a slice of extensions **without** the leading dot.
    ///
    /// ```rust
    /// # use logic_nih_plug_audio_processors::PluginFormatType;
    /// assert_eq!(PluginFormatType::Vst3.extensions(), &["vst3"]);
    /// assert_eq!(PluginFormatType::Clap.extensions(), &["clap"]);
    /// ```
    pub fn extensions(self) -> &'static [&'static str] {
        match self {
            Self::Vst3 => &["vst3"],
            Self::Clap => &["clap"],
            Self::Au => &["component"],
            Self::Lv2 => &["lv2"],
            Self::Vst2 => &["dll", "so", "dylib"],
            Self::Aax => &["aaxplugin"],
            Self::Ladspa => &["so"],
            Self::Unknown => &[],
        }
    }

    /// Whether this format is supported on the current compilation target.
    ///
    /// ```rust
    /// # use logic_nih_plug_audio_processors::PluginFormatType;
    /// // VST3 and CLAP are always "supported" (the framework can host them).
    /// assert!(PluginFormatType::Vst3.is_supported_on_current_platform());
    /// assert!(PluginFormatType::Clap.is_supported_on_current_platform());
    /// ```
    pub fn is_supported_on_current_platform(self) -> bool {
        match self {
            Self::Vst3 | Self::Clap | Self::Lv2 | Self::Vst2 => true,
            Self::Au => cfg!(target_os = "macos") || cfg!(target_os = "ios"),
            Self::Aax => true,
            Self::Ladspa => cfg!(target_os = "linux"),
            Self::Unknown => false,
        }
    }

    /// Default filesystem directories where plugins of this format are
    /// typically installed. Returns an empty slice for formats without
    /// standard locations (e.g. `Unknown`, `Au`).
    pub fn default_search_paths(self) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // System-wide paths
        if cfg!(target_os = "windows") {
            match self {
                Self::Vst3 => {
                    if let Ok(common) = std::env::var("COMMONPROGRAMFILES") {
                        paths.push(PathBuf::from(common).join("VST3"));
                    }
                    if let Ok(program) = std::env::var("PROGRAMFILES") {
                        paths.push(PathBuf::from(program).join("VST3"));
                    }
                }
                Self::Clap => {
                    if let Ok(common) = std::env::var("COMMONPROGRAMFILES") {
                        paths.push(PathBuf::from(common).join("CLAP"));
                    }
                }
                Self::Vst2 => {
                    if let Ok(program) = std::env::var("PROGRAMFILES") {
                        paths.push(PathBuf::from(program).join("VSTPlugins"));
                    }
                }
                _ => {}
            }
        } else if cfg!(target_os = "macos") {
            let home = std::env::var("HOME").unwrap_or_default();
            let home = PathBuf::from(home);
            match self {
                Self::Vst3 => {
                    paths.push(PathBuf::from("/Library/Audio/Plug-Ins/VST3"));
                    paths.push(home.join("Library/Audio/Plug-Ins/VST3"));
                }
                Self::Clap => {
                    paths.push(PathBuf::from("/Library/Audio/Plug-Ins/CLAP"));
                    paths.push(home.join("Library/Audio/Plug-Ins/CLAP"));
                }
                Self::Au => {
                    paths.push(PathBuf::from("/Library/Audio/Plug-Ins/Components"));
                    paths.push(home.join("Library/Audio/Plug-Ins/Components"));
                }
                Self::Vst2 => {
                    paths.push(PathBuf::from("/Library/Audio/Plug-Ins/VST"));
                    paths.push(home.join("Library/Audio/Plug-Ins/VST"));
                }
                _ => {}
            }
        } else if cfg!(target_os = "linux") {
            let home = std::env::var("HOME").unwrap_or_default();
            let home = PathBuf::from(home);
            match self {
                Self::Vst3 => {
                    paths.push(PathBuf::from("/usr/lib/vst3"));
                    paths.push(PathBuf::from("/usr/local/lib/vst3"));
                    paths.push(home.join(".vst3"));
                }
                Self::Clap => {
                    paths.push(PathBuf::from("/usr/lib/clap"));
                    paths.push(PathBuf::from("/usr/local/lib/clap"));
                    paths.push(home.join(".clap"));
                }
                Self::Lv2 => {
                    paths.push(PathBuf::from("/usr/lib/lv2"));
                    paths.push(PathBuf::from("/usr/local/lib/lv2"));
                    paths.push(home.join(".lv2"));
                }
                Self::Vst2 => {
                    paths.push(PathBuf::from("/usr/lib/vst"));
                    paths.push(PathBuf::from("/usr/local/lib/vst"));
                    paths.push(home.join(".vst"));
                }
                Self::Ladspa => {
                    paths.push(PathBuf::from("/usr/lib/ladspa"));
                    paths.push(PathBuf::from("/usr/local/lib/ladspa"));
                }
                _ => {}
            }
        }

        paths
    }

    /// Check whether a file path might contain a plugin of this format,
    /// based purely on the file extension (no binary inspection).
    ///
    /// ```rust
    /// # use logic_nih_plug_audio_processors::PluginFormatType;
    /// assert!(PluginFormatType::Vst3.file_might_be_plugin(std::path::Path::new("synth.vst3")));
    /// assert!(!PluginFormatType::Vst3.file_might_be_plugin(std::path::Path::new("synth.clap")));
    /// ```
    pub fn file_might_be_plugin(self, path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map_or(false, |ext| {
                self.extensions()
                    .iter()
                    .any(|e| e.eq_ignore_ascii_case(ext))
            })
    }
}

/// Trait for format-specific plugin scanning and loading.
///
/// Each plugin API (VST3, CLAP, AU, LV2, …) implements this trait to
/// provide format-specific discovery and instantiation logic. The host
/// calls these methods through a trait object, keeping the scanning and
/// loading code format-agnostic.
///
/// # Required methods
///
/// - [`format_type`](PluginFormat::format_type) — which format this
///   implementation handles.
/// - [`find_plugins_in_file`](PluginFormat::find_plugins_in_file) —
///   extract all [`PluginDescription`]s from a single binary.
///
/// # Provided methods
///
/// - [`default_search_paths`](PluginFormat::default_search_paths) —
///   standard directories for this format.
/// - [`file_might_contain_plugin`](PluginFormat::file_might_contain_plugin)
///   — quick extension-based check.
/// - [`needs_rescanning`](PluginFormat::needs_rescanning) — whether a
///   plugin binary has changed since it was last scanned.
/// - [`plugin_still_exists`](PluginFormat::plugin_still_exists) —
///   whether a plugin file is still present on disk.
///
/// # Example
///
/// ```rust
/// use logic_nih_plug_audio_processors::{
///     PluginDescription, PluginFormat, PluginFormatType,
/// };
///
/// struct Vst3Format;
///
/// impl PluginFormat for Vst3Format {
///     fn format_type(&self) -> PluginFormatType {
///         PluginFormatType::Vst3
///     }
///
///     fn find_plugins_in_file(
///         &self,
///         _path: &std::path::Path,
///     ) -> Vec<PluginDescription> {
///         // In a real implementation, this would dlopen the VST3
///         // bundle and query IPluginFactory.
///         vec![]
///     }
/// }
///
/// let fmt = Vst3Format;
/// assert_eq!(fmt.format_type(), PluginFormatType::Vst3);
/// assert!(fmt.default_search_paths().len() > 0 || cfg!(not(target_os = "windows")));
/// ```
pub trait PluginFormat: Send + Sync {
    /// Which plugin format this implementation handles.
    fn format_type(&self) -> PluginFormatType;

    /// Extract all plugin descriptions from a single binary file.
    ///
    /// A shell plugin (e.g. Kontakt, MeldaProduction MXXX) may return
    /// multiple descriptions. Most plugins return exactly one.
    ///
    /// The implementation should **not** add results to a
    /// [`KnownPluginList`](crate::KnownPluginList) — just return the
    /// descriptions.
    fn find_plugins_in_file(&self, path: &Path) -> Vec<PluginDescription>;

    /// Default directories to search for plugins of this format.
    ///
    /// Delegates to [`PluginFormatType::default_search_paths`].
    fn default_search_paths(&self) -> Vec<PathBuf> {
        self.format_type().default_search_paths()
    }

    /// Quick check: does this file extension suggest a plugin of this
    /// format? No binary inspection.
    fn file_might_contain_plugin(&self, path: &Path) -> bool {
        self.format_type().file_might_be_plugin(path)
    }

    /// Whether a previously-scanned plugin needs to be re-scanned,
    /// based on the file's modification time.
    ///
    /// Returns `true` if the file was modified after `last_scan_time`,
    /// or if `last_scan_time` is 0 (unknown).
    fn needs_rescanning(&self, desc: &PluginDescription, last_scan_time: u64) -> bool {
        if last_scan_time == 0 {
            return true;
        }
        desc.last_file_mod_time > last_scan_time
    }

    /// Whether a plugin file still exists on disk and can be loaded.
    fn plugin_still_exists(&self, desc: &PluginDescription) -> bool {
        Path::new(&desc.file_or_identifier).exists()
    }
}

/// A trivial [`PluginFormat`] implementation that never finds any plugins.
///
/// Useful as a placeholder or for testing.
pub struct NullPluginFormat {
    format: PluginFormatType,
}

impl NullPluginFormat {
    /// Create a new `NullPluginFormat` for the given format type.
    pub fn new(format: PluginFormatType) -> Self {
        Self { format }
    }
}

impl PluginFormat for NullPluginFormat {
    fn format_type(&self) -> PluginFormatType {
        self.format
    }

    fn find_plugins_in_file(&self, _path: &Path) -> Vec<PluginDescription> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── PluginFormatType ──

    #[test]
    fn format_name_roundtrip() {
        for fmt in [
            PluginFormatType::Vst3,
            PluginFormatType::Clap,
            PluginFormatType::Au,
            PluginFormatType::Lv2,
            PluginFormatType::Vst2,
            PluginFormatType::Aax,
            PluginFormatType::Ladspa,
        ] {
            assert_eq!(PluginFormatType::from_name(fmt.name()), fmt);
        }
    }

    #[test]
    fn from_name_case_insensitive() {
        assert_eq!(PluginFormatType::from_name("VST3"), PluginFormatType::Vst3);
        assert_eq!(PluginFormatType::from_name("Clap"), PluginFormatType::Clap);
        assert_eq!(PluginFormatType::from_name("UNKNOWN"), PluginFormatType::Unknown);
    }

    #[test]
    fn extensions_non_empty_for_known() {
        for fmt in [
            PluginFormatType::Vst3,
            PluginFormatType::Clap,
            PluginFormatType::Au,
            PluginFormatType::Lv2,
            PluginFormatType::Vst2,
            PluginFormatType::Aax,
        ] {
            assert!(
                !fmt.extensions().is_empty(),
                "{:?} should have extensions",
                fmt
            );
        }
    }

    #[test]
    fn unknown_has_no_extensions() {
        assert!(PluginFormatType::Unknown.extensions().is_empty());
    }

    #[test]
    fn file_might_be_plugin_matches_extension() {
        assert!(PluginFormatType::Vst3.file_might_be_plugin(
            Path::new("/usr/lib/vst3/Synth.vst3")
        ));
        assert!(!PluginFormatType::Vst3.file_might_be_plugin(
            Path::new("/usr/lib/vst3/Synth.clap")
        ));
        assert!(!PluginFormatType::Vst3.file_might_be_plugin(
            Path::new("/usr/lib/vst3/no-extension")
        ));
    }

    #[test]
    fn file_might_be_plugin_case_insensitive() {
        assert!(PluginFormatType::Clap.file_might_be_plugin(
            Path::new("Synth.CLAP")
        ));
    }

    #[test]
    fn supported_on_current_platform_smoke() {
        // VST3 and CLAP should be supported everywhere.
        assert!(PluginFormatType::Vst3.is_supported_on_current_platform());
        assert!(PluginFormatType::Clap.is_supported_on_current_platform());
        // AU is only macOS/iOS.
        #[cfg(not(any(target_os = "macos", target_os = "ios")))]
        assert!(!PluginFormatType::Au.is_supported_on_current_platform());
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        assert!(PluginFormatType::Au.is_supported_on_current_platform());
    }

    #[test]
    fn default_search_paths_returns_vec() {
        let paths = PluginFormatType::Vst3.default_search_paths();
        // On Windows we always get at least one path.
        #[cfg(target_os = "windows")]
        assert!(!paths.is_empty());
        // On other platforms the check is best-effort.
        let _ = paths;
    }

    // ── PluginFormat trait ──

    #[test]
    fn null_format_returns_empty() {
        let fmt = NullPluginFormat::new(PluginFormatType::Vst3);
        assert_eq!(fmt.format_type(), PluginFormatType::Vst3);
        assert!(fmt
            .find_plugins_in_file(Path::new("dummy.vst3"))
            .is_empty());
    }

    #[test]
    fn needs_rescanning_unknown_time() {
        let fmt = NullPluginFormat::new(PluginFormatType::Clap);
        let desc = PluginDescription {
            last_file_mod_time: 100,
            ..PluginDescription::default()
        };
        // last_scan_time = 0 → always rescan
        assert!(fmt.needs_rescanning(&desc, 0));
    }

    #[test]
    fn needs_rescanning_unchanged() {
        let fmt = NullPluginFormat::new(PluginFormatType::Clap);
        let desc = PluginDescription {
            last_file_mod_time: 100,
            ..PluginDescription::default()
        };
        assert!(!fmt.needs_rescanning(&desc, 200));
    }

    #[test]
    fn needs_rescanning_changed() {
        let fmt = NullPluginFormat::new(PluginFormatType::Clap);
        let desc = PluginDescription {
            last_file_mod_time: 300,
            ..PluginDescription::default()
        };
        assert!(fmt.needs_rescanning(&desc, 200));
    }

    #[test]
    fn plugin_still_exists_nonexistent() {
        let fmt = NullPluginFormat::new(PluginFormatType::Vst3);
        let desc = PluginDescription {
            file_or_identifier: "/nonexistent/path/to/plugin.vst3".into(),
            ..PluginDescription::default()
        };
        assert!(!fmt.plugin_still_exists(&desc));
    }
}
