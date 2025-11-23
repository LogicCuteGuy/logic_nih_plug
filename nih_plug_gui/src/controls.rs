//! UI control components (Button, Slider, Label).
//!
//! This module provides standard UI controls that can be used to build plugin interfaces.

use crate::components::{Bounds, Component};
use crate::error::{GuiError, Result};

#[cfg(feature = "graphics")]
use crate::lookandfeel::LookAndFeel;

#[cfg(any(feature = "graphics", feature = "text"))]
use nih_plug_graphics::{Color, Graphics};

/// Button state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    /// Button is in normal state
    Normal,
    /// Mouse is hovering over button
    Hover,
    /// Button is being pressed
    Pressed,
    /// Button is disabled
    Disabled,
}

/// A clickable button component.
///
/// Buttons display text and can be clicked to trigger actions.
///
/// # Examples
///
/// ```
/// use nih_plug_gui::controls::Button;
/// use nih_plug_gui::components::Bounds;
///
/// let mut button = Button::new("Click Me");
/// button.set_bounds(Bounds::new(10, 10, 100, 30)).unwrap();
/// button.set_enabled(true);
///
/// assert_eq!(button.text(), "Click Me");
/// assert!(button.is_enabled());
/// ```
pub struct Button {
    component: Component,
    text: String,
    button_state: ButtonState,
    on_click: Option<Box<dyn FnMut()>>,
}

impl Button {
    /// Create a new button with the given text.
    pub fn new(text: &str) -> Self {
        Self {
            component: Component::new(&format!("Button_{}", text)),
            text: text.to_string(),
            button_state: ButtonState::Normal,
            on_click: None,
        }
    }

    /// Get the button's text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Set the button's text.
    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }

    /// Get the button's state.
    pub fn button_state(&self) -> ButtonState {
        self.button_state
    }

    /// Set the button's state.
    pub fn set_button_state(&mut self, state: ButtonState) {
        self.button_state = state;
    }

    /// Set the click callback.
    pub fn set_on_click<F>(&mut self, callback: F)
    where
        F: FnMut() + 'static,
    {
        self.on_click = Some(Box::new(callback));
    }

    /// Trigger the click callback if set.
    pub fn click(&mut self) {
        if let Some(ref mut callback) = self.on_click {
            callback();
        }
    }

    /// Get the underlying component.
    pub fn component(&self) -> &Component {
        &self.component
    }

    /// Get mutable reference to the underlying component.
    pub fn component_mut(&mut self) -> &mut Component {
        &mut self.component
    }

    /// Set the button's bounds.
    pub fn set_bounds(&mut self, bounds: Bounds) -> Result<()> {
        self.component.set_bounds(bounds)
    }

    /// Get the button's bounds.
    pub fn bounds(&self) -> Bounds {
        self.component.bounds()
    }

    /// Check if the button is enabled.
    pub fn is_enabled(&self) -> bool {
        self.component.is_enabled()
    }

    /// Set whether the button is enabled.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.component.set_enabled(enabled);
        self.button_state = if enabled {
            ButtonState::Normal
        } else {
            ButtonState::Disabled
        };
    }

    /// Check if the button is visible.
    pub fn is_visible(&self) -> bool {
        self.component.is_visible()
    }

    /// Set the button's visibility.
    pub fn set_visible(&mut self, visible: bool) {
        self.component.set_visible(visible);
    }

    /// Render the button to a graphics context.
    ///
    /// Note: This is a simplified rendering without text support.
    /// For text rendering, use the `text` feature and provide a Font.
    #[cfg(feature = "graphics")]
    pub fn render(&self, graphics: &mut Graphics) -> Result<()> {
        let bounds = self.bounds();
        
        // Choose color based on state
        let color = match self.button_state {
            ButtonState::Normal => Color::rgb(200, 200, 200),
            ButtonState::Hover => Color::rgb(220, 220, 220),
            ButtonState::Pressed => Color::rgb(150, 150, 150),
            ButtonState::Disabled => Color::rgb(100, 100, 100),
        };

        graphics.set_color(color);
        graphics.fill_rect(bounds.x, bounds.y, bounds.width, bounds.height);

        // Draw border (using lines)
        graphics.set_color(Color::rgb(0, 0, 0));
        // Top
        graphics.draw_line(bounds.x, bounds.y, 
                          bounds.x + bounds.width as i32, bounds.y);
        // Right
        graphics.draw_line(bounds.x + bounds.width as i32, bounds.y,
                          bounds.x + bounds.width as i32, bounds.y + bounds.height as i32);
        // Bottom
        graphics.draw_line(bounds.x + bounds.width as i32, bounds.y + bounds.height as i32,
                          bounds.x, bounds.y + bounds.height as i32);
        // Left
        graphics.draw_line(bounds.x, bounds.y + bounds.height as i32,
                          bounds.x, bounds.y);

        Ok(())
    }

    /// Render the button with text to a graphics context.
    ///
    /// This method requires the `text` feature to be enabled.
    #[cfg(feature = "text")]
    pub fn render_with_text(&self, graphics: &mut Graphics, font: &nih_plug_graphics::Font) -> Result<()> {
        // First render the button background and border
        self.render(graphics)?;

        // Draw text (centered)
        if !self.text.is_empty() {
            let bounds = self.bounds();
            graphics.set_color(Color::rgb(0, 0, 0));
            let text_x = bounds.x + (bounds.width as i32 / 2);
            let text_y = bounds.y + (bounds.height as i32 / 2);
            graphics.draw_text(&self.text, text_x, text_y, font, 14.0);
        }

        Ok(())
    }

    /// Render the button using a LookAndFeel for styling.
    ///
    /// This method uses the provided LookAndFeel to determine colors and styling.
    #[cfg(feature = "graphics")]
    pub fn render_with_lookandfeel(&self, graphics: &mut Graphics, laf: &dyn LookAndFeel) -> Result<()> {
        let bounds = self.bounds();
        
        // Get colors from LookAndFeel
        let bg_color = laf.button_color(self.button_state);
        let border_color = laf.button_border_color();

        // Draw background
        graphics.set_color(bg_color);
        graphics.fill_rect(bounds.x, bounds.y, bounds.width, bounds.height);

        // Draw border
        graphics.set_color(border_color);
        let border_width = laf.border_width() as i32;
        for i in 0..border_width {
            // Top
            graphics.draw_line(bounds.x + i, bounds.y + i, 
                              bounds.x + bounds.width as i32 - i, bounds.y + i);
            // Right
            graphics.draw_line(bounds.x + bounds.width as i32 - i - 1, bounds.y + i,
                              bounds.x + bounds.width as i32 - i - 1, bounds.y + bounds.height as i32 - i);
            // Bottom
            graphics.draw_line(bounds.x + bounds.width as i32 - i, bounds.y + bounds.height as i32 - i - 1,
                              bounds.x + i, bounds.y + bounds.height as i32 - i - 1);
            // Left
            graphics.draw_line(bounds.x + i, bounds.y + bounds.height as i32 - i,
                              bounds.x + i, bounds.y + i);
        }

        Ok(())
    }

    /// Render the button with text using a LookAndFeel for styling.
    ///
    /// This method requires the `text` feature to be enabled.
    #[cfg(feature = "text")]
    pub fn render_with_lookandfeel_and_text(
        &self,
        graphics: &mut Graphics,
        font: &nih_plug_graphics::Font,
        laf: &dyn LookAndFeel,
    ) -> Result<()> {
        // First render the button background and border with LookAndFeel
        self.render_with_lookandfeel(graphics, laf)?;

        // Draw text (centered) with LookAndFeel colors
        if !self.text.is_empty() {
            let bounds = self.bounds();
            let text_color = laf.button_text_color(self.button_state);
            graphics.set_color(text_color);
            
            let text_x = bounds.x + (bounds.width as i32 / 2);
            let text_y = bounds.y + (bounds.height as i32 / 2);
            let font_size = laf.default_font_size();
            
            graphics.draw_text(&self.text, text_x, text_y, font, font_size);
        }

        Ok(())
    }
}

/// Slider orientation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SliderOrientation {
    /// Horizontal slider
    Horizontal,
    /// Vertical slider
    Vertical,
}

/// A slider component for selecting numeric values.
///
/// Sliders allow users to select a value within a range by dragging a thumb.
///
/// # Examples
///
/// ```
/// use nih_plug_gui::controls::{Slider, SliderOrientation};
/// use nih_plug_gui::components::Bounds;
///
/// let mut slider = Slider::new(SliderOrientation::Horizontal);
/// slider.set_bounds(Bounds::new(10, 10, 200, 30)).unwrap();
/// slider.set_range(0.0, 100.0);
/// slider.set_value(50.0);
///
/// assert_eq!(slider.value(), 50.0);
/// assert_eq!(slider.min_value(), 0.0);
/// assert_eq!(slider.max_value(), 100.0);
/// ```
pub struct Slider {
    component: Component,
    orientation: SliderOrientation,
    value: f64,
    min_value: f64,
    max_value: f64,
    on_value_change: Option<Box<dyn FnMut(f64)>>,
}

impl Slider {
    /// Create a new slider with the given orientation.
    pub fn new(orientation: SliderOrientation) -> Self {
        Self {
            component: Component::new("Slider"),
            orientation,
            value: 0.0,
            min_value: 0.0,
            max_value: 1.0,
            on_value_change: None,
        }
    }

    /// Get the slider's orientation.
    pub fn orientation(&self) -> SliderOrientation {
        self.orientation
    }

    /// Set the slider's orientation.
    pub fn set_orientation(&mut self, orientation: SliderOrientation) {
        self.orientation = orientation;
    }

    /// Get the current value.
    pub fn value(&self) -> f64 {
        self.value
    }

    /// Set the current value.
    ///
    /// The value will be clamped to the min/max range.
    pub fn set_value(&mut self, value: f64) {
        let clamped = value.clamp(self.min_value, self.max_value);
        if (self.value - clamped).abs() > f64::EPSILON {
            self.value = clamped;
            if let Some(ref mut callback) = self.on_value_change {
                callback(self.value);
            }
        }
    }

    /// Get the minimum value.
    pub fn min_value(&self) -> f64 {
        self.min_value
    }

    /// Get the maximum value.
    pub fn max_value(&self) -> f64 {
        self.max_value
    }

    /// Set the value range.
    ///
    /// Returns an error if min >= max.
    pub fn set_range(&mut self, min: f64, max: f64) -> Result<()> {
        if min >= max {
            return Err(GuiError::InvalidRange(min, max));
        }
        self.min_value = min;
        self.max_value = max;
        // Re-clamp current value
        self.value = self.value.clamp(min, max);
        Ok(())
    }

    /// Set the value change callback.
    pub fn set_on_value_change<F>(&mut self, callback: F)
    where
        F: FnMut(f64) + 'static,
    {
        self.on_value_change = Some(Box::new(callback));
    }

    /// Get the normalized value (0.0 to 1.0).
    pub fn normalized_value(&self) -> f64 {
        if (self.max_value - self.min_value).abs() < f64::EPSILON {
            0.0
        } else {
            (self.value - self.min_value) / (self.max_value - self.min_value)
        }
    }

    /// Set the value from a normalized value (0.0 to 1.0).
    pub fn set_normalized_value(&mut self, normalized: f64) {
        let clamped = normalized.clamp(0.0, 1.0);
        let value = self.min_value + clamped * (self.max_value - self.min_value);
        self.set_value(value);
    }

    /// Get the underlying component.
    pub fn component(&self) -> &Component {
        &self.component
    }

    /// Get mutable reference to the underlying component.
    pub fn component_mut(&mut self) -> &mut Component {
        &mut self.component
    }

    /// Set the slider's bounds.
    pub fn set_bounds(&mut self, bounds: Bounds) -> Result<()> {
        self.component.set_bounds(bounds)
    }

    /// Get the slider's bounds.
    pub fn bounds(&self) -> Bounds {
        self.component.bounds()
    }

    /// Check if the slider is enabled.
    pub fn is_enabled(&self) -> bool {
        self.component.is_enabled()
    }

    /// Set whether the slider is enabled.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.component.set_enabled(enabled);
    }

    /// Check if the slider is visible.
    pub fn is_visible(&self) -> bool {
        self.component.is_visible()
    }

    /// Set the slider's visibility.
    pub fn set_visible(&mut self, visible: bool) {
        self.component.set_visible(visible);
    }

    /// Render the slider to a graphics context.
    #[cfg(feature = "graphics")]
    pub fn render(&self, graphics: &mut Graphics) -> Result<()> {
        let bounds = self.bounds();
        let normalized = self.normalized_value();

        // Draw track
        graphics.set_color(Color::rgb(150, 150, 150));
        match self.orientation {
            SliderOrientation::Horizontal => {
                let track_height = bounds.height / 4;
                let track_y = bounds.y + (bounds.height as i32 / 2) - (track_height as i32 / 2);
                graphics.fill_rect(bounds.x, track_y, bounds.width, track_height);
            }
            SliderOrientation::Vertical => {
                let track_width = bounds.width / 4;
                let track_x = bounds.x + (bounds.width as i32 / 2) - (track_width as i32 / 2);
                graphics.fill_rect(track_x, bounds.y, track_width, bounds.height);
            }
        }

        // Draw thumb
        graphics.set_color(if self.is_enabled() {
            Color::rgb(100, 100, 200)
        } else {
            Color::rgb(100, 100, 100)
        });

        let thumb_size = 10;
        match self.orientation {
            SliderOrientation::Horizontal => {
                let thumb_x = bounds.x + (normalized * bounds.width as f64) as i32 - thumb_size / 2;
                let thumb_y = bounds.y + (bounds.height as i32 / 2) - thumb_size / 2;
                graphics.fill_rect(thumb_x, thumb_y, thumb_size as u32, thumb_size as u32);
            }
            SliderOrientation::Vertical => {
                let thumb_x = bounds.x + (bounds.width as i32 / 2) - thumb_size / 2;
                let thumb_y = bounds.y + ((1.0 - normalized) * bounds.height as f64) as i32 - thumb_size / 2;
                graphics.fill_rect(thumb_x, thumb_y, thumb_size as u32, thumb_size as u32);
            }
        }

        Ok(())
    }

    /// Render the slider using a LookAndFeel for styling.
    ///
    /// This method uses the provided LookAndFeel to determine colors and styling.
    #[cfg(feature = "graphics")]
    pub fn render_with_lookandfeel(&self, graphics: &mut Graphics, laf: &dyn LookAndFeel) -> Result<()> {
        let bounds = self.bounds();
        let normalized = self.normalized_value();
        let enabled = self.is_enabled();

        // Get colors from LookAndFeel
        let track_color = laf.slider_track_color(enabled);
        let thumb_color = laf.slider_thumb_color(enabled);

        // Draw track
        graphics.set_color(track_color);
        match self.orientation {
            SliderOrientation::Horizontal => {
                let track_height = bounds.height / 4;
                let track_y = bounds.y + (bounds.height as i32 / 2) - (track_height as i32 / 2);
                graphics.fill_rect(bounds.x, track_y, bounds.width, track_height);
            }
            SliderOrientation::Vertical => {
                let track_width = bounds.width / 4;
                let track_x = bounds.x + (bounds.width as i32 / 2) - (track_width as i32 / 2);
                graphics.fill_rect(track_x, bounds.y, track_width, bounds.height);
            }
        }

        // Draw thumb
        graphics.set_color(thumb_color);
        let thumb_size = 10;
        match self.orientation {
            SliderOrientation::Horizontal => {
                let thumb_x = bounds.x + (normalized * bounds.width as f64) as i32 - thumb_size / 2;
                let thumb_y = bounds.y + (bounds.height as i32 / 2) - thumb_size / 2;
                graphics.fill_rect(thumb_x, thumb_y, thumb_size as u32, thumb_size as u32);
            }
            SliderOrientation::Vertical => {
                let thumb_x = bounds.x + (bounds.width as i32 / 2) - thumb_size / 2;
                let thumb_y = bounds.y + ((1.0 - normalized) * bounds.height as f64) as i32 - thumb_size / 2;
                graphics.fill_rect(thumb_x, thumb_y, thumb_size as u32, thumb_size as u32);
            }
        }

        Ok(())
    }
}

/// Text alignment for labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlignment {
    /// Align text to the left
    Left,
    /// Center text
    Center,
    /// Align text to the right
    Right,
}

/// A label component for displaying text.
///
/// Labels display static or dynamic text with configurable alignment and styling.
///
/// # Examples
///
/// ```
/// use nih_plug_gui::controls::{Label, TextAlignment};
/// use nih_plug_gui::components::Bounds;
///
/// let mut label = Label::new("Hello, World!");
/// label.set_bounds(Bounds::new(10, 10, 200, 30)).unwrap();
/// label.set_alignment(TextAlignment::Center);
///
/// assert_eq!(label.text(), "Hello, World!");
/// assert_eq!(label.alignment(), TextAlignment::Center);
/// ```
pub struct Label {
    component: Component,
    text: String,
    alignment: TextAlignment,
    font_size: u32,
}

impl Label {
    /// Create a new label with the given text.
    pub fn new(text: &str) -> Self {
        Self {
            component: Component::new(&format!("Label_{}", text)),
            text: text.to_string(),
            alignment: TextAlignment::Left,
            font_size: 14,
        }
    }

    /// Get the label's text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Set the label's text.
    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }

    /// Get the text alignment.
    pub fn alignment(&self) -> TextAlignment {
        self.alignment
    }

    /// Set the text alignment.
    pub fn set_alignment(&mut self, alignment: TextAlignment) {
        self.alignment = alignment;
    }

    /// Get the font size.
    pub fn font_size(&self) -> u32 {
        self.font_size
    }

    /// Set the font size.
    pub fn set_font_size(&mut self, size: u32) {
        self.font_size = size;
    }

    /// Get the underlying component.
    pub fn component(&self) -> &Component {
        &self.component
    }

    /// Get mutable reference to the underlying component.
    pub fn component_mut(&mut self) -> &mut Component {
        &mut self.component
    }

    /// Set the label's bounds.
    pub fn set_bounds(&mut self, bounds: Bounds) -> Result<()> {
        self.component.set_bounds(bounds)
    }

    /// Get the label's bounds.
    pub fn bounds(&self) -> Bounds {
        self.component.bounds()
    }

    /// Check if the label is visible.
    pub fn is_visible(&self) -> bool {
        self.component.is_visible()
    }

    /// Set the label's visibility.
    pub fn set_visible(&mut self, visible: bool) {
        self.component.set_visible(visible);
    }

    /// Render the label to a graphics context.
    ///
    /// This method requires the `text` feature to be enabled and a Font to be provided.
    #[cfg(feature = "text")]
    pub fn render(&self, graphics: &mut Graphics, font: &nih_plug_graphics::Font) -> Result<()> {
        let bounds = self.bounds();

        if self.text.is_empty() {
            return Ok(());
        }

        graphics.set_color(Color::rgb(0, 0, 0));

        let text_x = match self.alignment {
            TextAlignment::Left => bounds.x + 5,
            TextAlignment::Center => bounds.x + (bounds.width as i32 / 2),
            TextAlignment::Right => bounds.x + bounds.width as i32 - 5,
        };

        let text_y = bounds.y + (bounds.height as i32 / 2);

        graphics.draw_text(&self.text, text_x, text_y, font, self.font_size as f32);

        Ok(())
    }

    /// Render the label using a LookAndFeel for styling.
    ///
    /// This method requires the `text` feature to be enabled and a Font to be provided.
    #[cfg(feature = "text")]
    pub fn render_with_lookandfeel(
        &self,
        graphics: &mut Graphics,
        font: &nih_plug_graphics::Font,
        laf: &dyn LookAndFeel,
    ) -> Result<()> {
        let bounds = self.bounds();

        if self.text.is_empty() {
            return Ok(());
        }

        // Get text color from LookAndFeel
        let text_color = laf.label_text_color(self.component.is_enabled());
        graphics.set_color(text_color);

        let padding = laf.component_padding() as i32;
        let text_x = match self.alignment {
            TextAlignment::Left => bounds.x + padding,
            TextAlignment::Center => bounds.x + (bounds.width as i32 / 2),
            TextAlignment::Right => bounds.x + bounds.width as i32 - padding,
        };

        let text_y = bounds.y + (bounds.height as i32 / 2);
        let font_size = if self.font_size == 14 {
            laf.default_font_size()
        } else {
            self.font_size as f32
        };

        graphics.draw_text(&self.text, text_x, text_y, font, font_size);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_creation() {
        let button = Button::new("Test");
        assert_eq!(button.text(), "Test");
        assert_eq!(button.button_state(), ButtonState::Normal);
        assert!(button.is_enabled());
    }

    #[test]
    fn test_button_text() {
        let mut button = Button::new("Initial");
        button.set_text("Updated");
        assert_eq!(button.text(), "Updated");
    }

    #[test]
    fn test_button_state() {
        let mut button = Button::new("Test");
        button.set_button_state(ButtonState::Hover);
        assert_eq!(button.button_state(), ButtonState::Hover);
    }

    #[test]
    fn test_button_enabled() {
        let mut button = Button::new("Test");
        button.set_enabled(false);
        assert!(!button.is_enabled());
        assert_eq!(button.button_state(), ButtonState::Disabled);
    }

    #[test]
    fn test_button_click() {
        use std::sync::{Arc, Mutex};
        
        let mut button = Button::new("Test");
        let clicked = Arc::new(Mutex::new(false));
        let clicked_clone = clicked.clone();
        
        button.set_on_click(move || {
            *clicked_clone.lock().unwrap() = true;
        });
        
        button.click();
        assert!(*clicked.lock().unwrap());
    }

    #[test]
    fn test_slider_creation() {
        let slider = Slider::new(SliderOrientation::Horizontal);
        assert_eq!(slider.orientation(), SliderOrientation::Horizontal);
        assert_eq!(slider.value(), 0.0);
        assert_eq!(slider.min_value(), 0.0);
        assert_eq!(slider.max_value(), 1.0);
    }

    #[test]
    fn test_slider_value() {
        let mut slider = Slider::new(SliderOrientation::Horizontal);
        slider.set_range(0.0, 100.0).unwrap();
        slider.set_value(50.0);
        assert_eq!(slider.value(), 50.0);
    }

    #[test]
    fn test_slider_value_clamping() {
        let mut slider = Slider::new(SliderOrientation::Horizontal);
        slider.set_range(0.0, 100.0).unwrap();
        slider.set_value(150.0);
        assert_eq!(slider.value(), 100.0);
        slider.set_value(-10.0);
        assert_eq!(slider.value(), 0.0);
    }

    #[test]
    fn test_slider_invalid_range() {
        let mut slider = Slider::new(SliderOrientation::Horizontal);
        let result = slider.set_range(100.0, 0.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_slider_normalized_value() {
        let mut slider = Slider::new(SliderOrientation::Horizontal);
        slider.set_range(0.0, 100.0).unwrap();
        slider.set_value(50.0);
        assert!((slider.normalized_value() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_slider_set_normalized_value() {
        let mut slider = Slider::new(SliderOrientation::Horizontal);
        slider.set_range(0.0, 100.0).unwrap();
        slider.set_normalized_value(0.75);
        assert!((slider.value() - 75.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_slider_value_change_callback() {
        use std::sync::{Arc, Mutex};
        
        let mut slider = Slider::new(SliderOrientation::Horizontal);
        slider.set_range(0.0, 100.0).unwrap();
        
        let last_value = Arc::new(Mutex::new(0.0));
        let last_value_clone = last_value.clone();
        
        slider.set_on_value_change(move |v| {
            *last_value_clone.lock().unwrap() = v;
        });
        
        slider.set_value(50.0);
        assert_eq!(*last_value.lock().unwrap(), 50.0);
    }

    #[test]
    fn test_label_creation() {
        let label = Label::new("Test Label");
        assert_eq!(label.text(), "Test Label");
        assert_eq!(label.alignment(), TextAlignment::Left);
        assert_eq!(label.font_size(), 14);
    }

    #[test]
    fn test_label_text() {
        let mut label = Label::new("Initial");
        label.set_text("Updated");
        assert_eq!(label.text(), "Updated");
    }

    #[test]
    fn test_label_alignment() {
        let mut label = Label::new("Test");
        label.set_alignment(TextAlignment::Center);
        assert_eq!(label.alignment(), TextAlignment::Center);
    }

    #[test]
    fn test_label_font_size() {
        let mut label = Label::new("Test");
        label.set_font_size(20);
        assert_eq!(label.font_size(), 20);
    }
}
