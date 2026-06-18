//! [`PluginDirectoryScanner`] — incremental directory scanner for plugins.

use std::path::{Path, PathBuf};

use crate::description::PluginDescription;
use crate::format::PluginFormat;

/// Incremental directory scanner that discovers plugin files and extracts
/// [`PluginDescription`]s from them.
///
/// This mirrors JUCE's `PluginDirectoryScanner`. It walks one or more
/// directories, feeding each candidate file to a [`PluginFormat`] for
/// inspection. The scan is incremental — call
/// [`scan_next_file`](PluginDirectoryScanner::scan_next_file) repeatedly
/// (e.g. once per UI frame) to avoid blocking.
///
/// The scanner is **decoupled** from
/// [`KnownPluginList`](crate::KnownPluginList): `scan_next_file` returns
/// discovered descriptions, and the caller decides how to add them. This
/// avoids borrow-checker friction and gives callers control over
/// deduplication and persistence.
///
/// # Crash safety
///
/// If a plugin crashes during scanning, call
/// [`blacklist_current_file`](PluginDirectoryScanner::blacklist_current_file)
/// to add its path to the **blacklist**. Use
/// [`write_dead_mans_pedal`](PluginDirectoryScanner::write_dead_mans_pedal)
/// to persist the blacklist across sessions.
///
/// # Example
///
/// ```rust,no_run
/// use logic_nih_plug_audio_processors::{
///     KnownPluginList, PluginDirectoryScanner, PluginFormatType,
///     NullPluginFormat,
/// };
///
/// let mut list = KnownPluginList::new();
/// let format = NullPluginFormat::new(PluginFormatType::Vst3);
/// let mut scanner = PluginDirectoryScanner::new(
///     &format,
///     &["/usr/lib/vst3".into()],
///     true,
/// );
///
/// // Scan all files (blocks — in a real app, call scan_next_file
/// // in a loop with UI updates).
/// while let Some(descs) = scanner.scan_next_file(false) {
///     for desc in descs {
///         list.add_type(desc);
///     }
/// }
/// ```
pub struct PluginDirectoryScanner<'a> {
    format: &'a dyn PluginFormat,
    files: Vec<PathBuf>,
    current_index: usize,
    blacklist: Vec<PathBuf>,
    failed_files: Vec<PathBuf>,
    /// Number of files scanned so far.
    scanned_count: usize,
}

impl<'a> PluginDirectoryScanner<'a> {
    /// Create a new scanner.
    ///
    /// - `format` — which plugin format to scan for.
    /// - `directories` — directories to search.
    /// - `search_recursively` — whether to descend into subdirectories.
    pub fn new(
        format: &'a dyn PluginFormat,
        directories: &[PathBuf],
        search_recursively: bool,
    ) -> Self {
        let mut files = Vec::new();
        for dir in directories {
            Self::collect_files(dir, search_recursively, format, &mut files);
        }
        Self {
            format,
            files,
            current_index: 0,
            blacklist: Vec::new(),
            failed_files: Vec::new(),
            scanned_count: 0,
        }
    }

    /// Create a scanner with an explicit blacklist (dead man's pedal).
    ///
    /// Files in `blacklist` are skipped during scanning.
    pub fn with_blacklist(
        format: &'a dyn PluginFormat,
        directories: &[PathBuf],
        search_recursively: bool,
        blacklist: Vec<PathBuf>,
    ) -> Self {
        let mut scanner = Self::new(format, directories, search_recursively);
        scanner.blacklist = blacklist;
        scanner
    }

    /// Scan the next file and return discovered plugin descriptions.
    ///
    /// Returns `Some(vec)` with one or more descriptions if a plugin was
    /// found, or `None` when scanning is complete. Files that don't
    /// contain plugins of the expected format are silently skipped.
    pub fn scan_next_file(
        &mut self,
        _dont_rescan: bool,
    ) -> Option<Vec<PluginDescription>> {
        while self.current_index < self.files.len() {
            let path = &self.files[self.current_index];
            self.current_index += 1;

            // Skip blacklisted files.
            if self.blacklist.iter().any(|b| b == path) {
                continue;
            }

            // Scan this file.
            let results = self.format.find_plugins_in_file(path);
            self.scanned_count += 1;

            if results.is_empty() {
                continue;
            }

            return Some(results);
        }
        None
    }

    /// Skip the current file without scanning it.
    pub fn skip_next_file(&mut self) {
        if self.current_index < self.files.len() {
            self.current_index += 1;
        }
    }

    /// Report that the current file caused a crash or error. It will
    /// be added to the internal blacklist for this session.
    pub fn blacklist_current_file(&mut self) {
        if self.current_index > 0 {
            let idx = self.current_index - 1;
            if idx < self.files.len() {
                self.failed_files.push(self.files[idx].clone());
            }
        }
    }

    /// Path of the next file that will be scanned.
    pub fn next_file_to_scan(&self) -> Option<&Path> {
        self.files.get(self.current_index).map(|p| p.as_path())
    }

    /// Scan progress as a value from 0.0 to 1.0.
    pub fn progress(&self) -> f32 {
        if self.files.is_empty() {
            return 1.0;
        }
        self.current_index as f32 / self.files.len() as f32
    }

    /// Total number of candidate files found.
    pub fn total_files(&self) -> usize {
        self.files.len()
    }

    /// Number of files scanned so far.
    pub fn scanned_files(&self) -> usize {
        self.scanned_count
    }

    /// Files that crashed or failed during scanning.
    pub fn failed_files(&self) -> &[PathBuf] {
        &self.failed_files
    }

    /// The blacklist (dead man's pedal) paths.
    pub fn blacklist(&self) -> &[PathBuf] {
        &self.blacklist
    }

    /// Format type being scanned.
    pub fn format_type(&self) -> crate::format::PluginFormatType {
        self.format.format_type()
    }

    /// Recursively collect plugin candidate files from a directory.
    fn collect_files(
        dir: &Path,
        recursive: bool,
        format: &dyn PluginFormat,
        files: &mut Vec<PathBuf>,
    ) {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if recursive {
                    Self::collect_files(&path, true, format, files);
                }
            } else if format.file_might_contain_plugin(&path) {
                files.push(path);
            }
        }
    }

    /// Write the failed files list to a "dead man's pedal" file.
    ///
    /// Each line is one absolute path. Returns the number of paths
    /// written.
    pub fn write_dead_mans_pedal(
        path: &Path,
        failed: &[PathBuf],
    ) -> std::io::Result<usize> {
        use std::io::Write;
        let mut f = std::fs::File::create(path)?;
        for p in failed {
            writeln!(f, "{}", p.display())?;
        }
        Ok(failed.len())
    }

    /// Read a "dead man's pedal" file into a list of paths.
    pub fn read_dead_mans_pedal(path: &Path) -> std::io::Result<Vec<PathBuf>> {
        let content = std::fs::read_to_string(path)?;
        Ok(content
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| PathBuf::from(l.trim()))
            .collect())
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::NullPluginFormat;
    use crate::{KnownPluginList, PluginDescription, PluginFormatType};
    use std::fs;

    /// A mock format that finds plugins in `.vst3` files.
    struct MockVst3Format {
        plugins: Vec<PluginDescription>,
    }

    impl MockVst3Format {
        fn new() -> Self {
            Self {
                plugins: Vec::new(),
            }
        }

        fn add_plugin_for_file(&mut self, path: &str, name: &str) {
            self.plugins.push(PluginDescription {
                name: name.into(),
                format: PluginFormatType::Vst3,
                unique_id: format!("mock:{name}"),
                file_or_identifier: path.into(),
                ..PluginDescription::default()
            });
        }
    }

    impl PluginFormat for MockVst3Format {
        fn format_type(&self) -> PluginFormatType {
            PluginFormatType::Vst3
        }

        fn find_plugins_in_file(&self, path: &Path) -> Vec<PluginDescription> {
            let path_str = path.to_string_lossy().to_string();
            self.plugins
                .iter()
                .filter(|d| d.file_or_identifier == path_str)
                .cloned()
                .collect()
        }
    }

    fn create_test_dir() -> PathBuf {
        let dir = PathBuf::from(format!(
            "target/test_plugin_scanner_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn create_fake_plugin(dir: &Path, name: &str) -> PathBuf {
        let path = dir.join(format!("{name}.vst3"));
        fs::write(&path, b"fake plugin binary").unwrap();
        path
    }

    #[test]
    fn scanner_finds_files() {
        let dir = create_test_dir();
        create_fake_plugin(&dir, "Synth");
        create_fake_plugin(&dir, "Effect");

        let format = NullPluginFormat::new(PluginFormatType::Vst3);
        let scanner =
            PluginDirectoryScanner::new(&format, &[dir.clone()], false);

        assert_eq!(scanner.total_files(), 2);
        assert_eq!(scanner.scanned_files(), 0);
        assert!((scanner.progress() - 0.0).abs() < f32::EPSILON);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn scanner_collects_plugins_incrementally() {
        let dir = create_test_dir();
        let p1 = create_fake_plugin(&dir, "Alpha");
        let p2 = create_fake_plugin(&dir, "Beta");

        let mut mock = MockVst3Format::new();
        mock.add_plugin_for_file(&p1.to_string_lossy(), "Alpha");
        mock.add_plugin_for_file(&p2.to_string_lossy(), "Beta");

        let mut list = KnownPluginList::new();
        let mut scanner =
            PluginDirectoryScanner::new(&mock, &[dir.clone()], false);

        assert_eq!(scanner.total_files(), 2);

        // Scan first file.
        let result = scanner.scan_next_file(false);
        assert!(result.is_some());
        for desc in result.unwrap() {
            list.add_type(desc);
        }
        assert_eq!(list.num_types(), 1);
        assert!((scanner.progress() - 0.5).abs() < f32::EPSILON);

        // Scan second file.
        let result = scanner.scan_next_file(false);
        assert!(result.is_some());
        for desc in result.unwrap() {
            list.add_type(desc);
        }
        assert_eq!(list.num_types(), 2);
        assert!((scanner.progress() - 1.0).abs() < f32::EPSILON);

        // No more files.
        assert!(scanner.scan_next_file(false).is_none());

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn scanner_skips_blacklisted_files() {
        let dir = create_test_dir();
        let p1 = create_fake_plugin(&dir, "Good");
        let _p2 = create_fake_plugin(&dir, "Bad");

        let mut mock = MockVst3Format::new();
        mock.add_plugin_for_file(&p1.to_string_lossy(), "Good");

        let bad_path = dir.join("Bad.vst3");
        let mut scanner = PluginDirectoryScanner::with_blacklist(
            &mock,
            &[dir.clone()],
            false,
            vec![bad_path],
        );

        // Should find Good (Bad is blacklisted).
        let mut list = KnownPluginList::new();
        if let Some(descs) = scanner.scan_next_file(false) {
            for d in descs {
                list.add_type(d);
            }
        }
        assert_eq!(list.num_types(), 1);
        assert_eq!(list.get_type(0).unwrap().name, "Good");

        // No more files (Bad was blacklisted).
        assert!(scanner.scan_next_file(false).is_none());

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn scanner_skip_next_file() {
        let dir = create_test_dir();
        create_fake_plugin(&dir, "Alpha");
        create_fake_plugin(&dir, "Beta");

        let format = NullPluginFormat::new(PluginFormatType::Vst3);
        let mut scanner =
            PluginDirectoryScanner::new(&format, &[dir.clone()], false);

        // Skip the first file.
        scanner.skip_next_file();
        assert_eq!(scanner.scanned_files(), 0);

        // Next scan will process Beta.
        scanner.scan_next_file(false);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn scanner_blacklist_current_file() {
        let dir = create_test_dir();
        create_fake_plugin(&dir, "Crasher");

        let format = NullPluginFormat::new(PluginFormatType::Vst3);
        let mut scanner =
            PluginDirectoryScanner::new(&format, &[dir.clone()], false);

        scanner.scan_next_file(false);
        scanner.blacklist_current_file();

        assert_eq!(scanner.failed_files().len(), 1);
        assert!(scanner.failed_files()[0]
            .to_string_lossy()
            .contains("Crasher"));

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn scanner_progress_empty_dir() {
        let dir = create_test_dir();
        let format = NullPluginFormat::new(PluginFormatType::Vst3);
        let scanner =
            PluginDirectoryScanner::new(&format, &[dir.clone()], false);

        assert_eq!(scanner.progress(), 1.0);
        assert_eq!(scanner.total_files(), 0);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn scanner_recursive() {
        let dir = create_test_dir();
        let sub = dir.join("subdir");
        fs::create_dir_all(&sub).unwrap();
        create_fake_plugin(&dir, "TopLevel");
        create_fake_plugin(&sub, "Nested");

        let format = NullPluginFormat::new(PluginFormatType::Vst3);
        let scanner =
            PluginDirectoryScanner::new(&format, &[dir.clone()], true);

        assert_eq!(scanner.total_files(), 2);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn scanner_nonexistent_dir() {
        let format = NullPluginFormat::new(PluginFormatType::Vst3);
        let scanner = PluginDirectoryScanner::new(
            &format,
            &[PathBuf::from("/nonexistent/dir/that/does/not/exist")],
            false,
        );
        assert_eq!(scanner.total_files(), 0);
    }

    #[test]
    fn scanner_format_type() {
        let format = NullPluginFormat::new(PluginFormatType::Clap);
        let scanner = PluginDirectoryScanner::new(&format, &[], false);
        assert_eq!(scanner.format_type(), PluginFormatType::Clap);
    }

    #[test]
    fn dead_mans_pedal_roundtrip() {
        let dir = create_test_dir();
        let pedal = dir.join("dead_mans_pedal.txt");

        let paths = vec![
            PathBuf::from("/usr/lib/vst3/Bad.vst3"),
            PathBuf::from("/usr/lib/vst3/Crasher.clap"),
        ];

        let written =
            PluginDirectoryScanner::write_dead_mans_pedal(&pedal, &paths)
                .unwrap();
        assert_eq!(written, 2);

        let read =
            PluginDirectoryScanner::read_dead_mans_pedal(&pedal).unwrap();
        assert_eq!(read, paths);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn scanner_next_file_to_scan() {
        let dir = create_test_dir();
        create_fake_plugin(&dir, "First");

        let format = NullPluginFormat::new(PluginFormatType::Vst3);
        let mut scanner =
            PluginDirectoryScanner::new(&format, &[dir.clone()], false);

        let next = scanner.next_file_to_scan();
        assert!(next.is_some());
        assert!(next.unwrap().to_string_lossy().contains("First"));

        scanner.scan_next_file(false);
        assert!(scanner.next_file_to_scan().is_none());

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn scanner_empty_directories() {
        let format = NullPluginFormat::new(PluginFormatType::Vst3);
        let mut scanner = PluginDirectoryScanner::new(&format, &[], false);
        assert_eq!(scanner.total_files(), 0);
        assert!(scanner.scan_next_file(false).is_none());
    }
}
