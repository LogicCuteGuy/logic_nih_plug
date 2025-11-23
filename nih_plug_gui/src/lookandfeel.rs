//! LookAndFeel customization system for UI components.
//!
//! This module provides a flexible appearance customization system inspired by JUCE's LookAndFeel.
//! It allows developers to customize the visual appearance of UI components through themes and
//! custom rendering implementations.
//!
//! # Examples
//!
//! ```
//! use nih_plug_gui::lookandfeel::{LookAndFeel, Theme, DefaultLookAndFeel};
//! use nih_plug_gui::controls::ButtonState;
//!
//! // Use the default look and feel
//! let laf = DefaultLookAndFeel::new();
//! let color = laf.button_color(ButtonState::Normal);
//!
//! // Use a dark theme
//! let dark_laf = DefaultLookAndFeel::with_theme(Theme::Dark);
//! let dark_color = dark_laf.button_color(ButtonState::Normal);
//! ```

use crate::controls::ButtonState;
use nih_plug_graphics::Color;

/// Predefined color themes for UI components.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    /// Light theme with bright backgrounds
    Light,
    /// Dark theme with dark backgrounds
    Dark,
    /// High contrast theme for accessibility
    HighContrast,
}

/// Color scheme for a theme.
#[derive(Debug, Clone, Copy)]
pub struct ColorScheme {
    /// Primary background color
    pub background: Color,
    /// Secondary background color (for panels, etc.)
    pub background_secondary: Color,
    /// Primary foreground/text color
    pub foreground: Color,
    /// Accent color for highlights and active elements
    pub accent: Color,
    /// Color for disabled elements
    pub disabled: Color,
    /// Border color
    pub border: Color,
    /// Hover color
    pub hover: Color,
    /// Pressed/active color
    pub pressed: Color,
}

impl ColorScheme {
    /// Get the color scheme for a theme.
    pub fn from_theme(theme: Theme) -> Self {
        match theme {
            Theme::Light => Self {
                background: Color::rgb(240, 240, 240),
                background_secondary: Color::rgb(255, 255, 255),
                foreground: Color::rgb(0, 0, 0),
                accent: Color::rgb(0, 120, 215),
                disabled: Color::rgb(160, 160, 160),
                border: Color::rgb(180, 180, 180),
                hover: Color::rgb(220, 220, 220),
                pressed: Color::rgb(200, 200, 200),
            },
            Theme::Dark => Self {
                background: Color::rgb(30, 30, 30),
                background_secondary: Color::rgb(45, 45, 45),
                foreground: Color::rgb(255, 255, 255),
                accent: Color::rgb(0, 120, 215),
                disabled: Color::rgb(100, 100, 100),
                border: Color::rgb(80, 80, 80),
                hover: Color::rgb(60, 60, 60),
                pressed: Color::rgb(50, 50, 50),
            },
            Theme::HighContrast => Self {
                background: Color::rgb(0, 0, 0),
                background_secondary: Color::rgb(0, 0, 0),
                foreground: Color::rgb(255, 255, 255),
                accent: Color::rgb(255, 255, 0),
                disabled: Color::rgb(128, 128, 128),
                border: Color::rgb(255, 255, 255),
                hover: Color::rgb(255, 255, 0),
                pressed: Color::rgb(255, 255, 0),
            },
        }
    }
}

/// Trait for customizing the appearance of UI components.
///
/// Implement this trait to create custom visual styles for your UI.
/// The default implementation provides a standard appearance.
///
/// # Examples
///
/// ```
/// use nih_plug_gui::lookandfeel::{LookAndFeel, ColorScheme, Theme};
/// use nih_plug_gui::controls::ButtonState;
/// use nih_plug_graphics::Color;
///
/// struct CustomLookAndFeel {
///     colors: ColorScheme,
/// }
///
/// impl LookAndFeel for CustomLookAndFeel {
///     fn button_color(&self, state: ButtonState) -> Color {
///         // Custom button colors
///         match state {
///             ButtonState::Normal => Color::rgb(100, 200, 100),
///             ButtonState::Hover => Color::rgb(120, 220, 120),
///             ButtonState::Pressed => Color::rgb(80, 180, 80),
///             ButtonState::Disabled => Color::rgb(150, 150, 150),
///         }
///     }
///
///     fn color_scheme(&self) -> &ColorScheme {
///         &self.colors
///     }
/// }
/// ```
pub trait LookAndFeel {
    /// Get the color scheme used by this look and feel.
    fn color_scheme(&self) -> &ColorScheme;

    /// Get the color for a button in the given state.
    fn button_color(&self, state: ButtonState) -> Color {
        let colors = self.color_scheme();
        match state {
            ButtonState::Normal => colors.background_secondary,
            ButtonState::Hover => colors.hover,
            ButtonState::Pressed => colors.pressed,
            ButtonState::Disabled => colors.disabled,
        }
    }

    /// Get the text color for a button in the given state.
    fn button_text_color(&self, state: ButtonState) -> Color {
        let colors = self.color_scheme();
        if state == ButtonState::Disabled {
            colors.disabled
        } else {
            colors.foreground
        }
    }

    /// Get the border color for a button.
    fn button_border_color(&self) -> Color {
        self.color_scheme().border
    }

    /// Get the track color for a slider.
    fn slider_track_color(&self, enabled: bool) -> Color {
        let colors = self.color_scheme();
        if enabled {
            colors.background
        } else {
            colors.disabled
        }
    }

    /// Get the thumb color for a slider.
    fn slider_thumb_color(&self, enabled: bool) -> Color {
        let colors = self.color_scheme();
        if enabled {
            colors.accent
        } else {
            colors.disabled
        }
    }

    /// Get the text color for a label.
    fn label_text_color(&self, enabled: bool) -> Color {
        let colors = self.color_scheme();
        if enabled {
            colors.foreground
        } else {
            colors.disabled
        }
    }

    /// Get the background color for the main window/panel.
    fn background_color(&self) -> Color {
        self.color_scheme().background
    }

    /// Get the border color for components.
    fn border_color(&self) -> Color {
        self.color_scheme().border
    }

    /// Get the accent color for highlights.
    fn accent_color(&self) -> Color {
        self.color_scheme().accent
    }

    /// Get the corner radius for rounded components (in pixels).
    fn corner_radius(&self) -> u32 {
        4
    }

    /// Get the border width for components (in pixels).
    fn border_width(&self) -> u32 {
        1
    }

    /// Get the default font size for text.
    fn default_font_size(&self) -> f32 {
        14.0
    }

    /// Get the padding inside components (in pixels).
    fn component_padding(&self) -> u32 {
        5
    }
}

/// Default implementation of LookAndFeel.
///
/// Provides a standard appearance with support for different themes.
///
/// # Examples
///
/// ```
/// use nih_plug_gui::lookandfeel::{DefaultLookAndFeel, Theme};
///
/// // Create with default (light) theme
/// let laf = DefaultLookAndFeel::new();
///
/// // Create with dark theme
/// let dark_laf = DefaultLookAndFeel::with_theme(Theme::Dark);
///
/// // Change theme
/// let mut laf = DefaultLookAndFeel::new();
/// laf.set_theme(Theme::HighContrast);
/// ```
pub struct DefaultLookAndFeel {
    colors: ColorScheme,
    theme: Theme,
}

impl DefaultLookAndFeel {
    /// Create a new DefaultLookAndFeel with the light theme.
    pub fn new() -> Self {
        Self::with_theme(Theme::Light)
    }

    /// Create a new DefaultLookAndFeel with the specified theme.
    pub fn with_theme(theme: Theme) -> Self {
        Self {
            colors: ColorScheme::from_theme(theme),
            theme,
        }
    }

    /// Get the current theme.
    pub fn theme(&self) -> Theme {
        self.theme
    }

    /// Set the theme and update colors.
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
        self.colors = ColorScheme::from_theme(theme);
    }
}

impl Default for DefaultLookAndFeel {
    fn default() -> Self {
        Self::new()
    }
}

impl LookAndFeel for DefaultLookAndFeel {
    fn color_scheme(&self) -> &ColorScheme {
        &self.colors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_color_schemes() {
        let light = ColorScheme::from_theme(Theme::Light);
        let dark = ColorScheme::from_theme(Theme::Dark);
        let high_contrast = ColorScheme::from_theme(Theme::HighContrast);

        // Light theme should have bright background
        assert_eq!(light.background.r, 240);
        
        // Dark theme should have dark background
        assert_eq!(dark.background.r, 30);
        
        // High contrast should have black background
        assert_eq!(high_contrast.background.r, 0);
    }

    #[test]
    fn test_default_lookandfeel() {
        let laf = DefaultLookAndFeel::new();
        assert_eq!(laf.theme(), Theme::Light);
        
        let color = laf.button_color(ButtonState::Normal);
        assert_eq!(color, Color::rgb(255, 255, 255));
    }

    #[test]
    fn test_lookandfeel_with_theme() {
        let laf = DefaultLookAndFeel::with_theme(Theme::Dark);
        assert_eq!(laf.theme(), Theme::Dark);
        
        let color = laf.button_color(ButtonState::Normal);
        assert_eq!(color, Color::rgb(45, 45, 45));
    }

    #[test]
    fn test_set_theme() {
        let mut laf = DefaultLookAndFeel::new();
        assert_eq!(laf.theme(), Theme::Light);
        
        laf.set_theme(Theme::Dark);
        assert_eq!(laf.theme(), Theme::Dark);
        
        let color = laf.button_color(ButtonState::Normal);
        assert_eq!(color, Color::rgb(45, 45, 45));
    }

    #[test]
    fn test_button_colors() {
        let laf = DefaultLookAndFeel::new();
        
        let normal = laf.button_color(ButtonState::Normal);
        let hover = laf.button_color(ButtonState::Hover);
        let pressed = laf.button_color(ButtonState::Pressed);
        let disabled = laf.button_color(ButtonState::Disabled);
        
        // All should be different
        assert_ne!(normal, hover);
        assert_ne!(hover, pressed);
        assert_ne!(pressed, disabled);
    }

    #[test]
    fn test_slider_colors() {
        let laf = DefaultLookAndFeel::new();
        
        let track_enabled = laf.slider_track_color(true);
        let track_disabled = laf.slider_track_color(false);
        let thumb_enabled = laf.slider_thumb_color(true);
        let thumb_disabled = laf.slider_thumb_color(false);
        
        assert_ne!(track_enabled, track_disabled);
        assert_ne!(thumb_enabled, thumb_disabled);
    }

    #[test]
    fn test_label_colors() {
        let laf = DefaultLookAndFeel::new();
        
        let enabled = laf.label_text_color(true);
        let disabled = laf.label_text_color(false);
        
        assert_ne!(enabled, disabled);
    }

    #[test]
    fn test_component_metrics() {
        let laf = DefaultLookAndFeel::new();
        
        assert_eq!(laf.corner_radius(), 4);
        assert_eq!(laf.border_width(), 1);
        assert_eq!(laf.default_font_size(), 14.0);
        assert_eq!(laf.component_padding(), 5);
    }

    #[test]
    fn test_high_contrast_theme() {
        let laf = DefaultLookAndFeel::with_theme(Theme::HighContrast);
        
        let bg = laf.background_color();
        let fg = laf.label_text_color(true);
        
        // High contrast should have black background and white foreground
        assert_eq!(bg, Color::rgb(0, 0, 0));
        assert_eq!(fg, Color::rgb(255, 255, 255));
    }
}
