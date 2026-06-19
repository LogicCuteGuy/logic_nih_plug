//! # juce_demorunner
//!
//! A GUI showcase of every public component from `logic_nih_plug_gui`.
//! Mirrors `JUCE/examples/DemoRunner/Source/*`.
//!
//! The GUI backend is selected at build time via a feature flag
//! (`gui-egui` default, `gui-iced`, `gui-vizia`). The mutually exclusive
//! constraint is enforced at compile time (see `compile_error!` below).
//!
//! ## What to learn from this example
//!
//! - How to register demos in each of 5 categories
//!   (Controls, Layouts, Animation, Graphics, AudioViz).
//! - How to wire `logic_nih_plug_dsp::analysis::LevelMeter` and
//!   `Oscilloscope` to a real audio source.
//! - How to use `logic_nih_plug_animation::easing` for smooth
//!   parameter transitions.
//! - How to use `logic_nih_plug_graphics::Painter` to render
//!   gradients and paths.

pub mod showcase;

pub mod backend;

pub mod nav;

/// Which backend is currently active. The value is determined at
/// compile time by the feature flag the user selected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BackendKind {
    /// `egui` backend (default).
    Egui,
    /// `iced` backend.
    Iced,
    /// `vizia` backend.
    Vizia,
}

impl BackendKind {
    /// The currently-selected backend. Determined by the feature flags
    /// the crate was compiled with.
    pub fn current() -> Self {
        #[cfg(feature = "gui-egui")]
        {
            BackendKind::Egui
        }
        #[cfg(feature = "gui-iced")]
        {
            BackendKind::Iced
        }
        #[cfg(feature = "gui-vizia")]
        {
            BackendKind::Vizia
        }
    }

    /// Human-readable name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Egui => "egui",
            Self::Iced => "iced",
            Self::Vizia => "vizia",
        }
    }
}

#[cfg(any(
    all(feature = "gui-egui", feature = "gui-iced"),
    all(feature = "gui-egui", feature = "gui-vizia"),
    all(feature = "gui-iced", feature = "gui-vizia"),
    all(feature = "gui-egui", feature = "gui-iced", feature = "gui-vizia"),
))]
compile_error!(
    "Exactly one of `gui-egui`, `gui-iced`, `gui-vizia` must be enabled. \
     They are mutually exclusive backends."
);

/// The list of available backends. Always returns all three so the
/// registry can show what *could* be enabled by toggling features.
pub fn backend_registry() -> Vec<BackendKind> {
    vec![BackendKind::Egui, BackendKind::Iced, BackendKind::Vizia]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_registry_lists_all_three() {
        let registry = backend_registry();
        assert_eq!(registry.len(), 3);
        assert!(registry.contains(&BackendKind::Egui));
        assert!(registry.contains(&BackendKind::Iced));
        assert!(registry.contains(&BackendKind::Vizia));
    }

    #[test]
    fn default_backend_is_egui() {
        assert_eq!(BackendKind::current(), BackendKind::Egui);
    }

    #[test]
    fn backend_names_are_correct() {
        assert_eq!(BackendKind::Egui.name(), "egui");
        assert_eq!(BackendKind::Iced.name(), "iced");
        assert_eq!(BackendKind::Vizia.name(), "vizia");
    }
}