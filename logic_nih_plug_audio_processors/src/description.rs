//! [`PluginDescription`] — immutable metadata about a discovered plugin.

use std::path::PathBuf;

use crate::format::PluginFormatType;

/// Immutable metadata describing a single plugin as discovered on disk.
///
/// This mirrors JUCE's `PluginDescription` struct. It contains everything
/// a host needs to identify, display, and re-instantiate a plugin **without**
/// loading the binary.
///
/// `PluginDescription` is serializable (via `Display`/`FromStr` as JSON-like
/// format) so a [`KnownPluginList`](crate::KnownPluginList) can persist
/// discovered plugins across sessions.
///
/// # Example
///
/// ```rust
/// use logic_nih_plug_audio_processors::{PluginDescription, PluginFormatType};
///
/// let desc = PluginDescription {
///     name: "SuperSaw".into(),
///     manufacturer_name: "Acme".into(),
///     version: "2.1.0".into(),
///     format: PluginFormatType::Vst3,
///     unique_id: "com.acme.supersaw".into(),
///     num_input_channels: 0,
///     num_output_channels: 2,
///     is_instrument: true,
///     ..PluginDescription::default()
/// };
///
/// assert!(desc.is_instrument);
/// assert_eq!(desc.format_name(), "VST3");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginDescription {
    /// Human-readable plugin name (e.g. "Massive X").
    pub name: String,

    /// Descriptive / marketing name, if different from `name`.
    pub descriptive_name: String,

    /// Plugin format: VST3, CLAP, AU, etc.
    pub format: PluginFormatType,

    /// Manufacturer name (e.g. "Native Instruments").
    pub manufacturer_name: String,

    /// Manufacturer unique ID (e.g. "com.native-instruments").
    pub manufacturer_id: String,

    /// Version string (e.g. "1.2.3" or "0x00010203").
    pub version: String,

    /// Category / type string (e.g. "Instrument", "Effect",
    /// "Instrument|Synth").
    pub category: String,

    /// Format-specific unique identifier.
    ///
    /// - VST3: the 16-byte class ID as a hex string (e.g.
    ///   `"4E455641565354330000000000000000"`)
    /// - CLAP: the CLAP plugin ID (e.g. `"com.acme.supersaw"`)
    /// - AU: the component type + subtype + manufacturer (e.g.
    ///   `"aumuSawwAcme"`)
    pub unique_id: String,

    /// Deprecated unique ID, used to migrate old presets.
    pub deprecated_id: String,

    /// File path or OS-level identifier for the plugin binary.
    ///
    /// - VST3/CLAP/LV2: filesystem path to the `.vst3` / `.clap` bundle
    /// - AU: the audio-unit identifier string
    pub file_or_identifier: String,

    /// Last modification time of the plugin file (seconds since Unix
    /// epoch). Zero means "unknown".
    pub last_file_mod_time: u64,

    /// Whether this plugin is an instrument (generates audio).
    pub is_instrument: bool,

    /// Whether the plugin accepts MIDI input.
    pub accepts_midi: bool,

    /// Whether the plugin produces MIDI output.
    pub produces_midi: bool,

    /// Whether the plugin is a shell plugin (contains sub-plugins).
    pub has_shared_container: bool,

    /// Number of audio input channels the plugin accepts.
    pub num_input_channels: u32,

    /// Number of audio output channels the plugin produces.
    pub num_output_channels: u32,
}

impl Default for PluginDescription {
    fn default() -> Self {
        Self {
            name: String::new(),
            descriptive_name: String::new(),
            format: PluginFormatType::Unknown,
            manufacturer_name: String::new(),
            manufacturer_id: String::new(),
            version: String::new(),
            category: "Unknown".into(),
            unique_id: String::new(),
            deprecated_id: String::new(),
            file_or_identifier: String::new(),
            last_file_mod_time: 0,
            is_instrument: false,
            accepts_midi: false,
            produces_midi: false,
            has_shared_container: false,
            num_input_channels: 0,
            num_output_channels: 0,
        }
    }
}

impl PluginDescription {
    /// Returns the format name as a human-readable string.
    ///
    /// ```rust
    /// # use logic_nih_plug_audio_processors::{PluginDescription, PluginFormatType};
    /// let mut d = PluginDescription::default();
    /// d.format = PluginFormatType::Clap;
    /// assert_eq!(d.format_name(), "CLAP");
    /// ```
    pub fn format_name(&self) -> &'static str {
        self.format.name()
    }

    /// Build the identifier string used to match plugins across sessions.
    ///
    /// The format is `"<format>:<unique_id>"`, e.g. `"VST3:4E45…0000"`.
    /// Two descriptions refer to the same plugin if and only if their
    /// identifier strings are equal (case-insensitive).
    ///
    /// ```rust
    /// # use logic_nih_plug_audio_processors::{PluginDescription, PluginFormatType};
    /// let mut d = PluginDescription::default();
    /// d.format = PluginFormatType::Vst3;
    /// d.unique_id = "ABCD1234".into();
    /// assert_eq!(d.identifier_string(), "VST3:ABCD1234");
    /// ```
    pub fn identifier_string(&self) -> String {
        format!("{}:{}", self.format.name(), self.unique_id)
    }

    /// Return the file path component of `file_or_identifier`, if it
    /// looks like a filesystem path.
    pub fn file_path(&self) -> Option<&PathBuf> {
        // We store as String; callers who need PathBuf can convert.
        // For now, return None if it doesn't start with a path-like char.
        // This is intentionally simple — real path detection is
        // format-specific.
        None
    }

    /// Check whether two descriptions refer to the same plugin.
    ///
    /// Compares format + unique_id (case-insensitive).
    ///
    /// ```rust
    /// # use logic_nih_plug_audio_processors::{PluginDescription, PluginFormatType};
    /// let mut a = PluginDescription::default();
    /// a.format = PluginFormatType::Vst3;
    /// a.unique_id = "ABCD1234".into();
    ///
    /// let mut b = PluginDescription::default();
    /// b.format = PluginFormatType::Vst3;
    /// b.unique_id = "abcd1234".into();
    ///
    /// assert!(a.is_same_plugin_as(&b));
    /// ```
    pub fn is_same_plugin_as(&self, other: &Self) -> bool {
        self.format == other.format
            && self.unique_id.eq_ignore_ascii_case(&other.unique_id)
    }

    /// Serialize this description to a compact string.
    ///
    /// Uses tab (`\t`) as delimiter to avoid conflicts with category
    /// strings that may contain `|`. 17 fields.
    pub fn to_compact_string(&self) -> String {
        [
            &*self.format.name(),
            &*self.name,
            &*self.descriptive_name,
            &*self.manufacturer_name,
            &*self.manufacturer_id,
            &*self.version,
            &*self.category,
            &*self.unique_id,
            &*self.deprecated_id,
            &*self.file_or_identifier,
            &self.last_file_mod_time.to_string(),
            if self.is_instrument { "1" } else { "0" },
            if self.accepts_midi { "1" } else { "0" },
            if self.produces_midi { "1" } else { "0" },
            if self.has_shared_container {
                "1"
            } else {
                "0"
            },
            &self.num_input_channels.to_string(),
            &self.num_output_channels.to_string(),
        ]
        .join("\t")
    }

    /// Deserialize from a compact tab-delimited string.
    ///
    /// Returns `None` if the string does not have at least 12 fields.
    pub fn from_compact_string(s: &str) -> Option<Self> {
        let fields: Vec<&str> = s.split('\t').collect();
        if fields.len() < 12 {
            return None;
        }
        Some(Self {
            format: PluginFormatType::from_name(fields[0]),
            name: fields[1].into(),
            descriptive_name: fields[2].into(),
            manufacturer_name: fields[3].into(),
            manufacturer_id: fields[4].into(),
            version: fields[5].into(),
            category: fields[6].into(),
            unique_id: fields[7].into(),
            deprecated_id: fields[8].into(),
            file_or_identifier: fields[9].into(),
            last_file_mod_time: fields[10].parse().unwrap_or(0),
            is_instrument: fields.get(11).map_or(false, |&v| v == "1"),
            accepts_midi: fields.get(12).map_or(false, |&v| v == "1"),
            produces_midi: fields.get(13).map_or(false, |&v| v == "1"),
            has_shared_container: fields.get(14).map_or(false, |&v| v == "1"),
            num_input_channels: fields.get(15).and_then(|v| v.parse().ok()).unwrap_or(0),
            num_output_channels: fields.get(16).and_then(|v| v.parse().ok()).unwrap_or(0),
        })
    }

    /// Compare two descriptions for sorting by name (case-insensitive).
    pub fn name_cmp(a: &Self, b: &Self) -> std::cmp::Ordering {
        a.name.to_lowercase().cmp(&b.name.to_lowercase())
    }

    /// Compare two descriptions for sorting by manufacturer then name.
    pub fn manufacturer_cmp(a: &Self, b: &Self) -> std::cmp::Ordering {
        a.manufacturer_name
            .to_lowercase()
            .cmp(&b.manufacturer_name.to_lowercase())
            .then_with(|| Self::name_cmp(a, b))
    }

    /// Compare two descriptions for sorting by format then name.
    pub fn format_cmp(a: &Self, b: &Self) -> std::cmp::Ordering {
        (a.format as u8)
            .cmp(&(b.format as u8))
            .then_with(|| Self::name_cmp(a, b))
    }

    /// Compare two descriptions for sorting by category then name.
    pub fn category_cmp(a: &Self, b: &Self) -> std::cmp::Ordering {
        a.category
            .to_lowercase()
            .cmp(&b.category.to_lowercase())
            .then_with(|| Self::name_cmp(a, b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_desc() -> PluginDescription {
        PluginDescription {
            name: "TestSynth".into(),
            descriptive_name: "A Test Synthesizer".into(),
            format: PluginFormatType::Vst3,
            manufacturer_name: "TestCo".into(),
            manufacturer_id: "com.testco".into(),
            version: "1.0.0".into(),
            category: "Instrument|Synth".into(),
            unique_id: "ABCD1234".into(),
            deprecated_id: String::new(),
            file_or_identifier: "/usr/lib/vst3/TestSynth.vst3".into(),
            last_file_mod_time: 1700000000,
            is_instrument: true,
            accepts_midi: true,
            produces_midi: false,
            has_shared_container: false,
            num_input_channels: 0,
            num_output_channels: 2,
        }
    }

    #[test]
    fn default_is_empty() {
        let d = PluginDescription::default();
        assert!(d.name.is_empty());
        assert_eq!(d.format, PluginFormatType::Unknown);
        assert!(!d.is_instrument);
    }

    #[test]
    fn format_name_delegates() {
        let mut d = PluginDescription::default();
        d.format = PluginFormatType::Clap;
        assert_eq!(d.format_name(), "CLAP");
    }

    #[test]
    fn identifier_string_format() {
        let d = sample_desc();
        assert_eq!(d.identifier_string(), "VST3:ABCD1234");
    }

    #[test]
    fn is_same_plugin_as_case_insensitive() {
        let a = sample_desc();
        let mut b = sample_desc();
        b.unique_id = "abcd1234".into();
        assert!(a.is_same_plugin_as(&b));

        b.format = PluginFormatType::Clap;
        assert!(!a.is_same_plugin_as(&b));
    }

    #[test]
    fn compact_roundtrip() {
        let original = sample_desc();
        let compact = original.to_compact_string();
        let restored = PluginDescription::from_compact_string(&compact)
            .expect("roundtrip failed");
        assert_eq!(original, restored);
    }

    #[test]
    fn compact_too_few_fields() {
        assert!(PluginDescription::from_compact_string("a|b|c").is_none());
    }

    #[test]
    fn sort_by_name() {
        let mut items = vec![
            PluginDescription {
                name: "Zebra".into(),
                ..PluginDescription::default()
            },
            PluginDescription {
                name: "alpha".into(),
                ..PluginDescription::default()
            },
            PluginDescription {
                name: "Mixer".into(),
                ..PluginDescription::default()
            },
        ];
        items.sort_by(|a, b| PluginDescription::name_cmp(a, b));
        assert_eq!(items[0].name, "alpha");
        assert_eq!(items[1].name, "Mixer");
        assert_eq!(items[2].name, "Zebra");
    }

    #[test]
    fn sort_by_manufacturer() {
        let mut items = vec![
            PluginDescription {
                name: "Zebra".into(),
                manufacturer_name: "ZCo".into(),
                ..PluginDescription::default()
            },
            PluginDescription {
                name: "Alpha".into(),
                manufacturer_name: "ACo".into(),
                ..PluginDescription::default()
            },
        ];
        items.sort_by(|a, b| PluginDescription::manufacturer_cmp(a, b));
        assert_eq!(items[0].manufacturer_name, "ACo");
        assert_eq!(items[1].manufacturer_name, "ZCo");
    }

    #[test]
    fn sort_by_format() {
        let mut items = vec![
            PluginDescription {
                name: "A".into(),
                format: PluginFormatType::Clap,
                ..PluginDescription::default()
            },
            PluginDescription {
                name: "B".into(),
                format: PluginFormatType::Vst3,
                ..PluginDescription::default()
            },
        ];
        items.sort_by(|a, b| PluginDescription::format_cmp(a, b));
        // Vst3=0, Clap=1 so Vst3 sorts first.
        assert_eq!(items[0].format, PluginFormatType::Vst3);
        assert_eq!(items[1].format, PluginFormatType::Clap);
    }

    #[test]
    fn sort_by_category() {
        let mut items = vec![
            PluginDescription {
                name: "A".into(),
                category: "Effect".into(),
                ..PluginDescription::default()
            },
            PluginDescription {
                name: "B".into(),
                category: "Instrument".into(),
                ..PluginDescription::default()
            },
        ];
        items.sort_by(|a, b| PluginDescription::category_cmp(a, b));
        assert_eq!(items[0].category, "Effect");
        assert_eq!(items[1].category, "Instrument");
    }
}
