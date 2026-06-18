//! [`KnownPluginList`] — persistent registry of discovered plugins.

use std::collections::HashSet;

use crate::description::PluginDescription;
use crate::format::PluginFormatType;

/// Sort method for [`KnownPluginList`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginListSortMethod {
    /// Sort by plugin name (case-insensitive alphabetical).
    Name,
    /// Sort by category then name.
    Category,
    /// Sort by manufacturer then name.
    Manufacturer,
    /// Sort by format then name.
    Format,
    /// Sort by file path then name.
    FileLocation,
    /// Sort by last modification time (newest first).
    RecentlyUpdated,
}

/// Persistent list of discovered [`PluginDescription`] entries.
///
/// This mirrors JUCE's `KnownPluginList`. It provides:
///
/// - **Deduplication** by format + unique_id (case-insensitive).
/// - **Sorting** by name, category, manufacturer, format, file location,
///   or recency.
/// - **Change callback** — register a listener to be notified when the
///   list changes.
/// - **Compact serialization** — one plugin per line, pipe-delimited.
///   Load/save to a file for cross-session caching.
///
/// # Example
///
/// ```rust
/// use logic_nih_plug_audio_processors::{
///     KnownPluginList, PluginDescription, PluginFormatType,
/// };
///
/// let mut list = KnownPluginList::new();
///
/// let desc = PluginDescription {
///     name: "SuperSaw".into(),
///     format: PluginFormatType::Vst3,
///     unique_id: "com.acme.supersaw".into(),
///     ..PluginDescription::default()
/// };
///
/// list.add_type(desc);
/// assert_eq!(list.num_types(), 1);
/// assert_eq!(list.get_type(0).unwrap().name, "SuperSaw");
/// ```
pub struct KnownPluginList {
    /// All known plugin descriptions.
    types: Vec<PluginDescription>,
    /// Set of identifier strings for fast dedup checks.
    ids: HashSet<String>,
    /// Change counter — incremented on every mutation.
    change_count: u64,
    /// Registered change listeners.
    listeners: Vec<Box<dyn Fn(&KnownPluginList) + Send + Sync>>,
}

impl KnownPluginList {
    /// Create an empty plugin list.
    pub fn new() -> Self {
        Self {
            types: Vec::new(),
            ids: HashSet::new(),
            change_count: 0,
            listeners: Vec::new(),
        }
    }

    /// Number of plugins in the list.
    pub fn num_types(&self) -> usize {
        self.types.len()
    }

    /// Whether the list is empty.
    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }

    /// Get a reference to the plugin at the given index.
    pub fn get_type(&self, index: usize) -> Option<&PluginDescription> {
        self.types.get(index)
    }

    /// Get a mutable reference to the plugin at the given index.
    pub fn get_type_mut(&mut self, index: usize) -> Option<&mut PluginDescription> {
        self.types.get_mut(index)
    }

    /// Iterator over all plugin descriptions.
    pub fn iter(&self) -> impl Iterator<Item = &PluginDescription> {
        self.types.iter()
    }

    /// Current change counter. Increments on every add/remove/clear.
    pub fn change_count(&self) -> u64 {
        self.change_count
    }

    /// Register a listener that is called whenever the list changes.
    ///
    /// The listener receives a reference to the list. Listeners are
    /// called in registration order.
    pub fn add_listener<F: Fn(&KnownPluginList) + Send + Sync + 'static>(
        &mut self,
        listener: F,
    ) {
        self.listeners.push(Box::new(listener));
    }

    /// Add a plugin type to the list. If a plugin with the same format
    /// and unique_id already exists, it is **replaced**.
    ///
    /// Returns `true` if a new entry was added (or an existing one was
    /// updated), `false` if the description was invalid.
    pub fn add_type(&mut self, desc: PluginDescription) -> bool {
        if desc.unique_id.is_empty() {
            return false;
        }

        let id = desc.identifier_string();
        if let Some(existing) = self.types.iter().position(|d| {
            d.identifier_string().eq_ignore_ascii_case(&id)
        }) {
            self.types[existing] = desc;
        } else {
            self.ids.insert(id);
            self.types.push(desc);
        }
        self.change_count += 1;
        self.notify_listeners();
        true
    }

    /// Remove the plugin at the given index. Returns the removed
    /// description, or `None` if the index was out of range.
    pub fn remove_type(&mut self, index: usize) -> Option<PluginDescription> {
        if index < self.types.len() {
            let removed = self.types.remove(index);
            self.ids.remove(&removed.identifier_string());
            self.change_count += 1;
            self.notify_listeners();
            Some(removed)
        } else {
            None
        }
    }

    /// Remove all plugins whose identifier matches the given one
    /// (case-insensitive format + unique_id).
    ///
    /// Returns the number of entries removed.
    pub fn remove_type_by_id(&mut self, identifier: &str) -> usize {
        let before = self.types.len();
        self.types.retain(|d| {
            let keep = !d
                .identifier_string()
                .eq_ignore_ascii_case(identifier);
            if !keep {
                self.ids.remove(&d.identifier_string());
            }
            keep
        });
        let removed = before - self.types.len();
        if removed > 0 {
            self.change_count += 1;
            self.notify_listeners();
        }
        removed
    }

    /// Remove all entries from the list.
    pub fn clear(&mut self) {
        self.types.clear();
        self.ids.clear();
        self.change_count += 1;
        self.notify_listeners();
    }

    /// Get all plugin descriptions.
    pub fn get_types(&self) -> &[PluginDescription] {
        &self.types
    }

    /// Get all plugin descriptions for a specific format.
    pub fn get_types_for_format(&self, format: PluginFormatType) -> Vec<&PluginDescription> {
        self.types.iter().filter(|d| d.format == format).collect()
    }

    /// Find a plugin by its identifier string (case-insensitive).
    pub fn get_type_for_identifier(&self, identifier: &str) -> Option<&PluginDescription> {
        self.types
            .iter()
            .find(|d| d.identifier_string().eq_ignore_ascii_case(identifier))
    }

    /// Find a plugin by file path.
    pub fn get_type_for_file(&self, path: &str) -> Option<&PluginDescription> {
        self.types.iter().find(|d| d.file_or_identifier == path)
    }

    /// Sort the list using the given method.
    pub fn sort(&mut self, method: PluginListSortMethod) {
        match method {
            PluginListSortMethod::Name => {
                self.types
                    .sort_by(|a, b| PluginDescription::name_cmp(a, b));
            }
            PluginListSortMethod::Category => {
                self.types
                    .sort_by(|a, b| PluginDescription::category_cmp(a, b));
            }
            PluginListSortMethod::Manufacturer => {
                self.types
                    .sort_by(|a, b| PluginDescription::manufacturer_cmp(a, b));
            }
            PluginListSortMethod::Format => {
                self.types
                    .sort_by(|a, b| PluginDescription::format_cmp(a, b));
            }
            PluginListSortMethod::FileLocation => {
                self.types
                    .sort_by(|a, b| a.file_or_identifier.cmp(&b.file_or_identifier));
            }
            PluginListSortMethod::RecentlyUpdated => {
                self.types
                    .sort_by(|a, b| b.last_file_mod_time.cmp(&a.last_file_mod_time));
            }
        }
        self.change_count += 1;
        self.notify_listeners();
    }

    /// Serialize the list to a compact string (one plugin per line,
    /// pipe-delimited).
    pub fn to_compact_string(&self) -> String {
        self.types
            .iter()
            .map(|d| d.to_compact_string())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Deserialize from a compact string. Skips malformed lines.
    pub fn from_compact_string(s: &str) -> Self {
        let mut list = Self::new();
        for line in s.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Some(desc) = PluginDescription::from_compact_string(line) {
                // Use internal add (no dedup check needed since we trust
                // the serialized form).
                list.ids.insert(desc.identifier_string());
                list.types.push(desc);
            }
        }
        list.change_count = 0; // loading doesn't count as a change
        list
    }

    /// Serialize to a simple newline-separated list of plugin names
    /// (for display / debugging).
    pub fn to_name_list(&self) -> String {
        self.types
            .iter()
            .map(|d| format!("{} [{}]", d.name, d.format_name()))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Remove plugins whose files no longer exist on disk.
    ///
    /// Returns the number of entries removed.
    pub fn remove_dead_plugins(&mut self) -> usize {
        let before = self.types.len();
        self.types.retain(|d| {
            let path = std::path::Path::new(&d.file_or_identifier);
            // Keep if it doesn't look like a path (e.g. AU identifier)
            // or if the file exists.
            !d.file_or_identifier.contains('/') && !d.file_or_identifier.contains('\\')
                || path.exists()
        });
        let removed = before - self.types.len();
        if removed > 0 {
            self.ids = self.types.iter().map(|d| d.identifier_string()).collect();
            self.change_count += 1;
            self.notify_listeners();
        }
        removed
    }

    /// Notify all registered listeners.
    fn notify_listeners(&self) {
        for listener in &self.listeners {
            listener(self);
        }
    }
}

impl Default for KnownPluginList {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vst3_synth() -> PluginDescription {
        PluginDescription {
            name: "Synth".into(),
            format: PluginFormatType::Vst3,
            unique_id: "com.test.synth".into(),
            category: "Instrument".into(),
            file_or_identifier: "/usr/lib/vst3/Synth.vst3".into(),
            last_file_mod_time: 1000,
            ..PluginDescription::default()
        }
    }

    fn clap_effect() -> PluginDescription {
        PluginDescription {
            name: "Reverb".into(),
            format: PluginFormatType::Clap,
            unique_id: "com.test.reverb".into(),
            category: "Effect".into(),
            manufacturer_name: "TestCo".into(),
            file_or_identifier: "/usr/lib/clap/Reverb.clap".into(),
            last_file_mod_time: 2000,
            ..PluginDescription::default()
        }
    }

    fn vst3_effect() -> PluginDescription {
        PluginDescription {
            name: "Delay".into(),
            format: PluginFormatType::Vst3,
            unique_id: "com.test.delay".into(),
            category: "Effect".into(),
            manufacturer_name: "OtherCo".into(),
            file_or_identifier: "/usr/lib/vst3/Delay.vst3".into(),
            last_file_mod_time: 500,
            ..PluginDescription::default()
        }
    }

    #[test]
    fn new_list_is_empty() {
        let list = KnownPluginList::new();
        assert!(list.is_empty());
        assert_eq!(list.num_types(), 0);
    }

    #[test]
    fn add_type_returns_true() {
        let mut list = KnownPluginList::new();
        assert!(list.add_type(vst3_synth()));
        assert_eq!(list.num_types(), 1);
    }

    #[test]
    fn add_type_empty_id_returns_false() {
        let mut list = KnownPluginList::new();
        let mut desc = PluginDescription::default();
        desc.unique_id = String::new();
        assert!(!list.add_type(desc));
        assert_eq!(list.num_types(), 0);
    }

    #[test]
    fn add_type_deduplicates() {
        let mut list = KnownPluginList::new();
        list.add_type(vst3_synth());
        let mut updated = vst3_synth();
        updated.version = "2.0.0".into();
        list.add_type(updated);
        assert_eq!(list.num_types(), 1);
        assert_eq!(list.get_type(0).unwrap().version, "2.0.0");
    }

    #[test]
    fn add_type_dedup_is_case_insensitive() {
        let mut list = KnownPluginList::new();
        list.add_type(vst3_synth());
        let mut dup = vst3_synth();
        dup.unique_id = "COM.TEST.SYNTH".into();
        list.add_type(dup);
        assert_eq!(list.num_types(), 1);
    }

    #[test]
    fn remove_type() {
        let mut list = KnownPluginList::new();
        list.add_type(vst3_synth());
        let removed = list.remove_type(0);
        assert!(removed.is_some());
        assert!(list.is_empty());
    }

    #[test]
    fn remove_type_out_of_range() {
        let mut list = KnownPluginList::new();
        assert!(list.remove_type(0).is_none());
    }

    #[test]
    fn remove_type_by_id() {
        let mut list = KnownPluginList::new();
        list.add_type(vst3_synth());
        list.add_type(clap_effect());
        let removed = list.remove_type_by_id("VST3:com.test.synth");
        assert_eq!(removed, 1);
        assert_eq!(list.num_types(), 1);
    }

    #[test]
    fn clear() {
        let mut list = KnownPluginList::new();
        list.add_type(vst3_synth());
        list.add_type(clap_effect());
        list.clear();
        assert!(list.is_empty());
    }

    #[test]
    fn get_types_for_format() {
        let mut list = KnownPluginList::new();
        list.add_type(vst3_synth());
        list.add_type(clap_effect());
        list.add_type(vst3_effect());

        let vst3 = list.get_types_for_format(PluginFormatType::Vst3);
        assert_eq!(vst3.len(), 2);
        let clap = list.get_types_for_format(PluginFormatType::Clap);
        assert_eq!(clap.len(), 1);
    }

    #[test]
    fn get_type_for_identifier() {
        let mut list = KnownPluginList::new();
        list.add_type(vst3_synth());
        assert!(list.get_type_for_identifier("VST3:com.test.synth").is_some());
        assert!(list
            .get_type_for_identifier("vst3:com.test.SYNTH")
            .is_some());
        assert!(list.get_type_for_identifier("CLAP:com.test.synth").is_none());
    }

    #[test]
    fn get_type_for_file() {
        let mut list = KnownPluginList::new();
        list.add_type(vst3_synth());
        assert!(list
            .get_type_for_file("/usr/lib/vst3/Synth.vst3")
            .is_some());
        assert!(list.get_type_for_file("/nonexistent").is_none());
    }

    #[test]
    fn sort_by_name() {
        let mut list = KnownPluginList::new();
        list.add_type(clap_effect()); // "Reverb"
        list.add_type(vst3_synth()); // "Synth"
        list.add_type(vst3_effect()); // "Delay"

        list.sort(PluginListSortMethod::Name);
        assert_eq!(list.get_type(0).unwrap().name, "Delay");
        assert_eq!(list.get_type(1).unwrap().name, "Reverb");
        assert_eq!(list.get_type(2).unwrap().name, "Synth");
    }

    #[test]
    fn sort_by_category() {
        let mut list = KnownPluginList::new();
        list.add_type(vst3_synth()); // "Instrument"
        list.add_type(clap_effect()); // "Effect"

        list.sort(PluginListSortMethod::Category);
        assert_eq!(list.get_type(0).unwrap().category, "Effect");
        assert_eq!(list.get_type(1).unwrap().category, "Instrument");
    }

    #[test]
    fn sort_by_manufacturer() {
        let mut list = KnownPluginList::new();
        list.add_type(vst3_effect()); // "OtherCo"
        list.add_type(clap_effect()); // "TestCo"

        list.sort(PluginListSortMethod::Manufacturer);
        assert_eq!(
            list.get_type(0).unwrap().manufacturer_name,
            "OtherCo"
        );
        assert_eq!(list.get_type(1).unwrap().manufacturer_name, "TestCo");
    }

    #[test]
    fn sort_by_format() {
        let mut list = KnownPluginList::new();
        list.add_type(vst3_synth());
        list.add_type(clap_effect());

        list.sort(PluginListSortMethod::Format);
        // Vst3=0, Clap=1 so Vst3 sorts first.
        assert_eq!(list.get_type(0).unwrap().format, PluginFormatType::Vst3);
        assert_eq!(list.get_type(1).unwrap().format, PluginFormatType::Clap);
    }

    #[test]
    fn sort_by_recently_updated() {
        let mut list = KnownPluginList::new();
        list.add_type(vst3_effect()); // 500
        list.add_type(clap_effect()); // 2000
        list.add_type(vst3_synth()); // 1000

        list.sort(PluginListSortMethod::RecentlyUpdated);
        assert_eq!(list.get_type(0).unwrap().last_file_mod_time, 2000);
        assert_eq!(list.get_type(1).unwrap().last_file_mod_time, 1000);
        assert_eq!(list.get_type(2).unwrap().last_file_mod_time, 500);
    }

    #[test]
    fn sort_by_file_location() {
        let mut list = KnownPluginList::new();
        list.add_type(vst3_synth()); // /usr/lib/vst3/Synth.vst3
        list.add_type(clap_effect()); // /usr/lib/clap/Reverb.clap

        list.sort(PluginListSortMethod::FileLocation);
        assert_eq!(
            list.get_type(0).unwrap().file_or_identifier,
            "/usr/lib/clap/Reverb.clap"
        );
        assert_eq!(
            list.get_type(1).unwrap().file_or_identifier,
            "/usr/lib/vst3/Synth.vst3"
        );
    }

    #[test]
    fn compact_roundtrip() {
        let mut list = KnownPluginList::new();
        list.add_type(vst3_synth());
        list.add_type(clap_effect());

        let compact = list.to_compact_string();
        let restored = KnownPluginList::from_compact_string(&compact);
        assert_eq!(restored.num_types(), 2);
        assert_eq!(restored.get_type(0).unwrap().name, "Synth");
        assert_eq!(restored.get_type(1).unwrap().name, "Reverb");
    }

    #[test]
    fn from_compact_string_skips_malformed() {
        let data = "bad|data\n".to_string() + &vst3_synth().to_compact_string() + "\ngarbage";
        let list = KnownPluginList::from_compact_string(&data);
        assert_eq!(list.num_types(), 1);
    }

    #[test]
    fn to_name_list() {
        let mut list = KnownPluginList::new();
        list.add_type(vst3_synth());
        list.add_type(clap_effect());

        let names = list.to_name_list();
        assert!(names.contains("Synth [VST3]"));
        assert!(names.contains("Reverb [CLAP]"));
    }

    #[test]
    fn change_count_increments() {
        let mut list = KnownPluginList::new();
        assert_eq!(list.change_count(), 0);
        list.add_type(vst3_synth());
        assert_eq!(list.change_count(), 1);
        list.remove_type(0);
        assert_eq!(list.change_count(), 2);
        list.clear();
        assert_eq!(list.change_count(), 3);
    }

    #[test]
    fn listener_notified() {
        use std::sync::atomic::{AtomicU64, Ordering};
        use std::sync::Arc;

        let count = Arc::new(AtomicU64::new(0));
        let count_clone = count.clone();

        let mut list = KnownPluginList::new();
        list.add_listener(move |_| {
            count_clone.fetch_add(1, Ordering::SeqCst);
        });

        list.add_type(vst3_synth());
        assert_eq!(count.load(Ordering::SeqCst), 1);

        list.remove_type(0);
        assert_eq!(count.load(Ordering::SeqCst), 2);

        list.clear();
        assert_eq!(count.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn iter_yields_all() {
        let mut list = KnownPluginList::new();
        list.add_type(vst3_synth());
        list.add_type(clap_effect());

        let names: Vec<&str> = list.iter().map(|d| d.name.as_str()).collect();
        assert_eq!(names, vec!["Synth", "Reverb"]);
    }

    #[test]
    fn default_is_same_as_new() {
        let a = KnownPluginList::new();
        let b = KnownPluginList::default();
        assert_eq!(a.num_types(), b.num_types());
    }
}
