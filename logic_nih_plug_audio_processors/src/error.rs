//! Error types for host-side plugin management.

use std::fmt;

/// Errors that can occur during plugin discovery, scanning, or loading.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioProcessorsError {
    /// The plugin file could not be opened or read.
    FileOpenFailed(String),

    /// The plugin binary does not contain a recognized plugin of the
    /// expected format.
    NotAPlugin(String),

    /// Multiple plugins were found in a single binary (shell plugin)
    /// but only one was expected.
    MultiplePluginsFound(Vec<String>),

    /// The plugin reported an incompatible I/O configuration.
    IncompatibleLayout {
        /// Number of inputs the plugin requires.
        required_inputs: u32,
        /// Number of outputs the plugin requires.
        required_outputs: u32,
    },

    /// The plugin could not be instantiated (e.g. missing runtime,
    /// version mismatch, corrupted binary).
    InstantiationFailed(String),

    /// A scanning operation was cancelled or the directory was
    /// inaccessible.
    ScanFailed(String),

    /// Serialization or deserialization of the plugin list failed.
    SerializationError(String),
}

impl fmt::Display for AudioProcessorsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileOpenFailed(path) => write!(f, "Could not open plugin file: {path}"),
            Self::NotAPlugin(path) => write!(f, "Not a recognized plugin: {path}"),
            Self::MultiplePluginsFound(names) => {
                write!(f, "Multiple plugins found: {}", names.join(", "))
            }
            Self::IncompatibleLayout {
                required_inputs,
                required_outputs,
            } => write!(
                f,
                "Incompatible I/O layout: {required_inputs} in / {required_outputs} out"
            ),
            Self::InstantiationFailed(msg) => {
                write!(f, "Plugin instantiation failed: {msg}")
            }
            Self::ScanFailed(msg) => write!(f, "Scan failed: {msg}"),
            Self::SerializationError(msg) => {
                write!(f, "Serialization error: {msg}")
            }
        }
    }
}

impl std::error::Error for AudioProcessorsError {}

/// A convenience alias for results in this crate.
pub type AudioProcessorsResult<T> = Result<T, AudioProcessorsError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display() {
        let e = AudioProcessorsError::FileOpenFailed("/foo.vst3".into());
        assert_eq!(
            e.to_string(),
            "Could not open plugin file: /foo.vst3"
        );
    }

    #[test]
    fn error_is_clone_and_eq() {
        let a = AudioProcessorsError::NotAPlugin("x.clap".into());
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn error_is_std_error() {
        let e: Box<dyn std::error::Error> =
            Box::new(AudioProcessorsError::ScanFailed("timeout".into()));
        assert!(e.to_string().contains("timeout"));
    }

    #[test]
    fn multiple_plugins_display() {
        let e = AudioProcessorsError::MultiplePluginsFound(vec![
            "Alpha".into(),
            "Beta".into(),
        ]);
        assert_eq!(e.to_string(), "Multiple plugins found: Alpha, Beta");
    }

    #[test]
    fn incompatible_layout_display() {
        let e = AudioProcessorsError::IncompatibleLayout {
            required_inputs: 2,
            required_outputs: 6,
        };
        assert!(e.to_string().contains("2 in / 6 out"));
    }
}
