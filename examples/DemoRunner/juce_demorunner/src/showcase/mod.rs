//! Showcase registry — every demo registers itself in one of the 5
//! categories (Controls, Layouts, Animation, Graphics, AudioViz).

/// The 5 showcase categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShowcaseCategory {
    /// Widgets / controls (Slider, Knob, ToggleButton, etc).
    Controls,
    /// Layouts (FlexBox, CssGrid, AbsoluteLayout).
    Layouts,
    /// Eased animations (easing curves, morphing).
    Animation,
    /// Vector graphics (Painter, gradient, path stroke).
    Graphics,
    /// Audio visualization (LevelMeter, Oscilloscope, Spectrum).
    AudioViz,
}

impl ShowcaseCategory {
    /// Display name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Controls => "Controls",
            Self::Layouts => "Layouts",
            Self::Animation => "Animation",
            Self::Graphics => "Graphics",
            Self::AudioViz => "Audio visualization",
        }
    }

    /// All 5 categories in display order.
    pub fn all() -> [Self; 5] {
        [
            Self::Controls,
            Self::Layouts,
            Self::Animation,
            Self::Graphics,
            Self::AudioViz,
        ]
    }
}

/// A single demo entry.
#[derive(Debug, Clone)]
pub struct DemoEntry {
    /// Demo ID (unique within its category).
    pub id: String,
    /// Human-readable title.
    pub title: String,
    /// One-line description.
    pub description: String,
}

impl DemoEntry {
    /// Create a new demo entry.
    pub fn new(id: impl Into<String>, title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: description.into(),
        }
    }
}

pub mod controls;
pub mod layouts;
pub mod animation;
pub mod graphics;
pub mod audio_viz;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_categories_listed() {
        let cats = ShowcaseCategory::all();
        assert_eq!(cats.len(), 5);
        assert_eq!(cats[0], ShowcaseCategory::Controls);
        assert_eq!(cats[4], ShowcaseCategory::AudioViz);
    }

    #[test]
    fn category_names_are_non_empty() {
        for c in ShowcaseCategory::all() {
            assert!(!c.name().is_empty());
        }
    }
}