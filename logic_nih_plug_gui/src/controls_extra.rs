//! Additional UI controls (ComboBox, TextEditor, ToggleButton, CheckBox, ProgressBar,
//! Tooltip, DrawableButton, HyperlinkButton, ImageComponent).
//!
//! This module extends the base controls in `controls.rs` with more complex widgets
//! ported from JUCE.

use crate::components::{Bounds, Component};
use crate::error::Result;

#[cfg(feature = "graphics")]
use crate::lookandfeel::LookAndFeel;

use logic_nih_plug_graphics::Color;

#[cfg(feature = "graphics")]
use logic_nih_plug_graphics::Graphics;

// ---------------------------------------------------------------------------
// ComboBox
// ---------------------------------------------------------------------------

/// A drop-down combo box for selecting from a list of items.
///
/// # Examples
///
/// ```ignore
/// use logic_nih_plug_gui::controls_extra::ComboBox;
/// use logic_nih_plug_gui::components::Bounds;
///
/// let mut combo = ComboBox::new();
/// combo.add_item("Option 1");
/// combo.add_item("Option 2");
/// combo.set_selected_index(Some(0));
/// assert_eq!(combo.selected_item(), Some("Option 1"));
/// ```
pub struct ComboBox {
    component: Component,
    items: Vec<String>,
    selected_index: Option<usize>,
    is_open: bool,
    on_change: Option<Box<dyn FnMut(usize, &str)>>,
}

impl ComboBox {
    /// Create a new empty combo box.
    pub fn new() -> Self {
        Self {
            component: Component::new("ComboBox"),
            items: Vec::new(),
            selected_index: None,
            is_open: false,
            on_change: None,
        }
    }

    /// Add an item to the combo box.
    pub fn add_item(&mut self, item: &str) {
        self.items.push(item.to_string());
    }

    /// Remove all items.
    pub fn clear(&mut self) {
        self.items.clear();
        self.selected_index = None;
    }

    /// Get the number of items.
    pub fn num_items(&self) -> usize {
        self.items.len()
    }

    /// Get an item by index.
    pub fn item(&self, index: usize) -> Option<&str> {
        self.items.get(index).map(|s| s.as_str())
    }

    /// Get all item strings.
    pub fn items(&self) -> &[String] {
        &self.items
    }

    /// Get the currently selected index.
    pub fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    /// Set the selected index. `None` deselects.
    pub fn set_selected_index(&mut self, index: Option<usize>) {
        if let Some(i) = index {
            if i >= self.items.len() {
                return;
            }
        }
        if self.selected_index != index {
            self.selected_index = index;
            if let Some(ref mut cb) = self.on_change {
                if let Some(i) = index {
                    cb(i, &self.items[i]);
                }
            }
        }
    }

    /// Get the text of the currently selected item.
    pub fn selected_item(&self) -> Option<&str> {
        self.selected_index.and_then(|i| self.items.get(i).map(|s| s.as_str()))
    }

    /// Get whether the dropdown is currently open.
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Set whether the dropdown is open.
    pub fn set_open(&mut self, open: bool) {
        self.is_open = open;
    }

    /// Set the change callback. Called with `(index, text)`.
    pub fn set_on_change<F>(&mut self, callback: F)
    where
        F: FnMut(usize, &str) + 'static,
    {
        self.on_change = Some(Box::new(callback));
    }

    // -- Component delegation ------------------------------------------------

    /// Get the underlying component.
    pub fn component(&self) -> &Component {
        &self.component
    }

    /// Get mutable reference to the underlying component.
    pub fn component_mut(&mut self) -> &mut Component {
        &mut self.component
    }

    /// Set bounds.
    pub fn set_bounds(&mut self, bounds: Bounds) -> Result<()> {
        self.component.set_bounds(bounds)
    }

    /// Get bounds.
    pub fn bounds(&self) -> Bounds {
        self.component.bounds()
    }

    /// Check if enabled.
    pub fn is_enabled(&self) -> bool {
        self.component.is_enabled()
    }

    /// Set enabled.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.component.set_enabled(enabled);
    }

    /// Check if visible.
    pub fn is_visible(&self) -> bool {
        self.component.is_visible()
    }

    /// Set visibility.
    pub fn set_visible(&mut self, visible: bool) {
        self.component.set_visible(visible);
    }

    /// Render the combo box to a graphics context.
    #[cfg(feature = "graphics")]
    pub fn render(&self, graphics: &mut Graphics) -> Result<()> {
        let bounds = self.bounds();

        // Background
        graphics.set_color(Color::rgb(255, 255, 255));
        graphics.fill_rect(bounds.x, bounds.y, bounds.width, bounds.height);

        // Border
        graphics.set_color(Color::rgb(0, 0, 0));
        graphics.draw_line(bounds.x, bounds.y, bounds.x + bounds.width as i32, bounds.y);
        graphics.draw_line(
            bounds.x + bounds.width as i32 - 1,
            bounds.y,
            bounds.x + bounds.width as i32 - 1,
            bounds.y + bounds.height as i32,
        );
        graphics.draw_line(
            bounds.x + bounds.width as i32,
            bounds.y + bounds.height as i32 - 1,
            bounds.x,
            bounds.y + bounds.height as i32 - 1,
        );
        graphics.draw_line(bounds.x, bounds.y + bounds.height as i32, bounds.x, bounds.y);

        // Selected text
        if let Some(text) = self.selected_item() {
            graphics.set_color(Color::rgb(0, 0, 0));
            let text_y = bounds.y + (bounds.height as i32 / 2);
            // Simplified rendering — real impl would use a Font
            graphics.draw_line(bounds.x + 5, text_y, bounds.x + 5 + text.len() as i32 * 8, text_y);
        }

        // Dropdown arrow
        let arrow_x = bounds.x + bounds.width as i32 - 15;
        let arrow_y = bounds.y + (bounds.height as i32 / 2);
        graphics.set_color(Color::rgb(80, 80, 80));
        graphics.draw_line(arrow_x, arrow_y - 3, arrow_x + 5, arrow_y + 3);
        graphics.draw_line(arrow_x + 5, arrow_y + 3, arrow_x + 10, arrow_y - 3);

        Ok(())
    }

    /// Render using a LookAndFeel.
    #[cfg(feature = "graphics")]
    pub fn render_with_lookandfeel(&self, graphics: &mut Graphics, laf: &dyn LookAndFeel) -> Result<()> {
        let bounds = self.bounds();
        let colors = laf.color_scheme();

        // Background
        graphics.set_color(colors.background_secondary);
        graphics.fill_rect(bounds.x, bounds.y, bounds.width, bounds.height);

        // Border
        graphics.set_color(colors.border);
        for i in 0..laf.border_width() as i32 {
            graphics.draw_line(bounds.x + i, bounds.y + i, bounds.x + bounds.width as i32 - i, bounds.y + i);
            graphics.draw_line(bounds.x + bounds.width as i32 - i - 1, bounds.y + i, bounds.x + bounds.width as i32 - i - 1, bounds.y + bounds.height as i32 - i);
            graphics.draw_line(bounds.x + bounds.width as i32 - i, bounds.y + bounds.height as i32 - i - 1, bounds.x + i, bounds.y + bounds.height as i32 - i - 1);
            graphics.draw_line(bounds.x + i, bounds.y + bounds.height as i32 - i, bounds.x + i, bounds.y + i);
        }

        // Dropdown arrow
        let arrow_x = bounds.x + bounds.width as i32 - 15;
        let arrow_y = bounds.y + (bounds.height as i32 / 2);
        graphics.set_color(colors.foreground);
        graphics.draw_line(arrow_x, arrow_y - 3, arrow_x + 5, arrow_y + 3);
        graphics.draw_line(arrow_x + 5, arrow_y + 3, arrow_x + 10, arrow_y - 3);

        Ok(())
    }
}

impl Default for ComboBox {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// TextEditor
// ---------------------------------------------------------------------------

/// An editable text field.
///
/// Supports single-line and multi-line modes, read-only mode, and
/// maximum-length constraints.
///
/// # Examples
///
/// ```ignore
/// use logic_nih_plug_gui::controls_extra::TextEditor;
///
/// let mut editor = TextEditor::new();
/// editor.set_text("Hello");
/// assert_eq!(editor.text(), "Hello");
/// ```
pub struct TextEditor {
    component: Component,
    text: String,
    is_multiline: bool,
    max_length: Option<usize>,
    is_read_only: bool,
    cursor_position: usize,
    on_text_change: Option<Box<dyn FnMut(&str)>>,
}

impl TextEditor {
    /// Create a new single-line text editor.
    pub fn new() -> Self {
        Self {
            component: Component::new("TextEditor"),
            text: String::new(),
            is_multiline: false,
            max_length: None,
            is_read_only: false,
            cursor_position: 0,
            on_text_change: None,
        }
    }

    /// Create a multi-line text editor.
    pub fn multiline() -> Self {
        Self {
            is_multiline: true,
            ..Self::new()
        }
    }

    /// Get the current text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Set the text.
    pub fn set_text(&mut self, text: &str) {
        let t = if let Some(max) = self.max_length {
            &text[..text.len().min(max)]
        } else {
            text
        };
        if self.text != t {
            self.text = t.to_string();
            self.cursor_position = self.cursor_position.min(self.text.len());
            if let Some(ref mut cb) = self.on_text_change {
                cb(&self.text);
            }
        }
    }

    /// Clear the text.
    pub fn clear(&mut self) {
        self.set_text("");
    }

    /// Whether this is a multi-line editor.
    pub fn is_multiline(&self) -> bool {
        self.is_multiline
    }

    /// Set multi-line mode.
    pub fn set_multiline(&mut self, multiline: bool) {
        self.is_multiline = multiline;
    }

    /// Get the maximum length (None = unlimited).
    pub fn max_length(&self) -> Option<usize> {
        self.max_length
    }

    /// Set the maximum length. `None` removes the limit.
    pub fn set_max_length(&mut self, max: Option<usize>) {
        self.max_length = max;
        if let Some(m) = max {
            self.text.truncate(m);
            self.cursor_position = self.cursor_position.min(self.text.len());
        }
    }

    /// Whether the editor is read-only.
    pub fn is_read_only(&self) -> bool {
        self.is_read_only
    }

    /// Set read-only mode.
    pub fn set_read_only(&mut self, read_only: bool) {
        self.is_read_only = read_only;
    }

    /// Get the cursor position (byte offset).
    pub fn cursor_position(&self) -> usize {
        self.cursor_position
    }

    /// Set the cursor position (clamped to text length).
    pub fn set_cursor_position(&mut self, pos: usize) {
        self.cursor_position = pos.min(self.text.len());
    }

    /// Insert text at the cursor position.
    pub fn insert_text(&mut self, text: &str) {
        if self.is_read_only {
            return;
        }
        let insert = if let Some(max) = self.max_length {
            let remaining = max.saturating_sub(self.text.len());
            &text[..text.len().min(remaining)]
        } else {
            text
        };
        if insert.is_empty() {
            return;
        }
        let pos = self.cursor_position.min(self.text.len());
        self.text.insert_str(pos, insert);
        self.cursor_position += insert.len();
        if let Some(ref mut cb) = self.on_text_change {
            cb(&self.text);
        }
    }

    /// Set the text change callback.
    pub fn set_on_text_change<F>(&mut self, callback: F)
    where
        F: FnMut(&str) + 'static,
    {
        self.on_text_change = Some(Box::new(callback));
    }

    // -- Component delegation ------------------------------------------------

    /// Get the underlying component.
    pub fn component(&self) -> &Component {
        &self.component
    }

    /// Get mutable reference to the underlying component.
    pub fn component_mut(&mut self) -> &mut Component {
        &mut self.component
    }

    /// Set bounds.
    pub fn set_bounds(&mut self, bounds: Bounds) -> Result<()> {
        self.component.set_bounds(bounds)
    }

    /// Get bounds.
    pub fn bounds(&self) -> Bounds {
        self.component.bounds()
    }

    /// Check if enabled.
    pub fn is_enabled(&self) -> bool {
        self.component.is_enabled()
    }

    /// Set enabled.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.component.set_enabled(enabled);
    }

    /// Check if visible.
    pub fn is_visible(&self) -> bool {
        self.component.is_visible()
    }

    /// Set visibility.
    pub fn set_visible(&mut self, visible: bool) {
        self.component.set_visible(visible);
    }

    /// Render the text editor to a graphics context.
    #[cfg(feature = "graphics")]
    pub fn render(&self, graphics: &mut Graphics) -> Result<()> {
        let bounds = self.bounds();

        // Background
        graphics.set_color(Color::rgb(255, 255, 255));
        graphics.fill_rect(bounds.x, bounds.y, bounds.width, bounds.height);

        // Border
        graphics.set_color(Color::rgb(0, 0, 0));
        graphics.draw_line(bounds.x, bounds.y, bounds.x + bounds.width as i32, bounds.y);
        graphics.draw_line(
            bounds.x + bounds.width as i32 - 1,
            bounds.y,
            bounds.x + bounds.width as i32 - 1,
            bounds.y + bounds.height as i32,
        );
        graphics.draw_line(
            bounds.x + bounds.width as i32,
            bounds.y + bounds.height as i32 - 1,
            bounds.x,
            bounds.y + bounds.height as i32 - 1,
        );
        graphics.draw_line(bounds.x, bounds.y + bounds.height as i32, bounds.x, bounds.y);

        // Text placeholder line
        if !self.text.is_empty() {
            graphics.set_color(Color::rgb(0, 0, 0));
            let text_y = bounds.y + (bounds.height as i32 / 2);
            graphics.draw_line(bounds.x + 5, text_y, bounds.x + 5 + self.text.len() as i32 * 8, text_y);
        }

        Ok(())
    }

    /// Render using a LookAndFeel.
    #[cfg(feature = "graphics")]
    pub fn render_with_lookandfeel(&self, graphics: &mut Graphics, laf: &dyn LookAndFeel) -> Result<()> {
        let bounds = self.bounds();
        let colors = laf.color_scheme();

        graphics.set_color(if self.is_read_only { colors.background } else { colors.background_secondary });
        graphics.fill_rect(bounds.x, bounds.y, bounds.width, bounds.height);

        graphics.set_color(colors.border);
        for i in 0..laf.border_width() as i32 {
            graphics.draw_line(bounds.x + i, bounds.y + i, bounds.x + bounds.width as i32 - i, bounds.y + i);
            graphics.draw_line(bounds.x + bounds.width as i32 - i - 1, bounds.y + i, bounds.x + bounds.width as i32 - i - 1, bounds.y + bounds.height as i32 - i);
            graphics.draw_line(bounds.x + bounds.width as i32 - i, bounds.y + bounds.height as i32 - i - 1, bounds.x + i, bounds.y + bounds.height as i32 - i - 1);
            graphics.draw_line(bounds.x + i, bounds.y + bounds.height as i32 - i, bounds.x + i, bounds.y + i);
        }

        Ok(())
    }
}

impl Default for TextEditor {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// ToggleButton
// ---------------------------------------------------------------------------

/// A button that toggles between on/off states.
///
/// # Examples
///
/// ```ignore
/// use logic_nih_plug_gui::controls_extra::ToggleButton;
///
/// let mut toggle = ToggleButton::new("Enable");
/// toggle.set_on(true);
/// assert!(toggle.is_on());
/// toggle.toggle();
/// assert!(!toggle.is_on());
/// ```
pub struct ToggleButton {
    component: Component,
    text: String,
    is_on: bool,
    on_toggle: Option<Box<dyn FnMut(bool)>>,
}

impl ToggleButton {
    /// Create a new toggle button with the given label text.
    pub fn new(text: &str) -> Self {
        Self {
            component: Component::new(&format!("ToggleButton_{}", text)),
            text: text.to_string(),
            is_on: false,
            on_toggle: None,
        }
    }

    /// Get the label text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Set the label text.
    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }

    /// Whether the toggle is currently on.
    pub fn is_on(&self) -> bool {
        self.is_on
    }

    /// Set the toggle state directly.
    pub fn set_on(&mut self, on: bool) {
        if self.is_on != on {
            self.is_on = on;
            if let Some(ref mut cb) = self.on_toggle {
                cb(self.is_on);
            }
        }
    }

    /// Toggle the state.
    pub fn toggle(&mut self) {
        self.set_on(!self.is_on);
    }

    /// Set the toggle callback.
    pub fn set_on_toggle<F>(&mut self, callback: F)
    where
        F: FnMut(bool) + 'static,
    {
        self.on_toggle = Some(Box::new(callback));
    }

    // -- Component delegation ------------------------------------------------

    /// Get the underlying component.
    pub fn component(&self) -> &Component {
        &self.component
    }

    /// Get mutable reference to the underlying component.
    pub fn component_mut(&mut self) -> &mut Component {
        &mut self.component
    }

    /// Set bounds.
    pub fn set_bounds(&mut self, bounds: Bounds) -> Result<()> {
        self.component.set_bounds(bounds)
    }

    /// Get bounds.
    pub fn bounds(&self) -> Bounds {
        self.component.bounds()
    }

    /// Check if enabled.
    pub fn is_enabled(&self) -> bool {
        self.component.is_enabled()
    }

    /// Set enabled.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.component.set_enabled(enabled);
    }

    /// Check if visible.
    pub fn is_visible(&self) -> bool {
        self.component.is_visible()
    }

    /// Set visibility.
    pub fn set_visible(&mut self, visible: bool) {
        self.component.set_visible(visible);
    }

    /// Render the toggle button.
    #[cfg(feature = "graphics")]
    pub fn render(&self, graphics: &mut Graphics) -> Result<()> {
        let bounds = self.bounds();

        // Background: tinted when on
        let bg = if self.is_on {
            Color::rgb(100, 150, 220)
        } else {
            Color::rgb(200, 200, 200)
        };
        graphics.set_color(bg);
        graphics.fill_rect(bounds.x, bounds.y, bounds.width, bounds.height);

        // Border
        graphics.set_color(Color::rgb(0, 0, 0));
        graphics.draw_line(bounds.x, bounds.y, bounds.x + bounds.width as i32, bounds.y);
        graphics.draw_line(
            bounds.x + bounds.width as i32 - 1,
            bounds.y,
            bounds.x + bounds.width as i32 - 1,
            bounds.y + bounds.height as i32,
        );
        graphics.draw_line(
            bounds.x + bounds.width as i32,
            bounds.y + bounds.height as i32 - 1,
            bounds.x,
            bounds.y + bounds.height as i32 - 1,
        );
        graphics.draw_line(bounds.x, bounds.y + bounds.height as i32, bounds.x, bounds.y);

        Ok(())
    }

    /// Render using a LookAndFeel.
    #[cfg(feature = "graphics")]
    pub fn render_with_lookandfeel(&self, graphics: &mut Graphics, laf: &dyn LookAndFeel) -> Result<()> {
        let bounds = self.bounds();
        let colors = laf.color_scheme();

        let bg = if self.is_on { colors.accent } else { colors.background_secondary };
        graphics.set_color(bg);
        graphics.fill_rect(bounds.x, bounds.y, bounds.width, bounds.height);

        graphics.set_color(colors.border);
        for i in 0..laf.border_width() as i32 {
            graphics.draw_line(bounds.x + i, bounds.y + i, bounds.x + bounds.width as i32 - i, bounds.y + i);
            graphics.draw_line(bounds.x + bounds.width as i32 - i - 1, bounds.y + i, bounds.x + bounds.width as i32 - i - 1, bounds.y + bounds.height as i32 - i);
            graphics.draw_line(bounds.x + bounds.width as i32 - i, bounds.y + bounds.height as i32 - i - 1, bounds.x + i, bounds.y + bounds.height as i32 - i - 1);
            graphics.draw_line(bounds.x + i, bounds.y + bounds.height as i32 - i, bounds.x + i, bounds.y + i);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// CheckBox
// ---------------------------------------------------------------------------

/// A checkbox control for boolean on/off selection.
///
/// Renders a small box with a check mark when checked.
///
/// # Examples
///
/// ```ignore
/// use logic_nih_plug_gui::controls_extra::CheckBox;
///
/// let mut cb = CheckBox::new("Accept terms");
/// assert!(!cb.is_checked());
/// cb.set_checked(true);
/// assert!(cb.is_checked());
/// ```
pub struct CheckBox {
    component: Component,
    label: String,
    is_checked: bool,
    on_checked_change: Option<Box<dyn FnMut(bool)>>,
}

impl CheckBox {
    /// Create a new checkbox with the given label.
    pub fn new(label: &str) -> Self {
        Self {
            component: Component::new(&format!("CheckBox_{}", label)),
            label: label.to_string(),
            is_checked: false,
            on_checked_change: None,
        }
    }

    /// Get the label text.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Set the label text.
    pub fn set_label(&mut self, label: &str) {
        self.label = label.to_string();
    }

    /// Whether the checkbox is checked.
    pub fn is_checked(&self) -> bool {
        self.is_checked
    }

    /// Set the checked state.
    pub fn set_checked(&mut self, checked: bool) {
        if self.is_checked != checked {
            self.is_checked = checked;
            if let Some(ref mut cb) = self.on_checked_change {
                cb(self.is_checked);
            }
        }
    }

    /// Toggle the checked state.
    pub fn toggle(&mut self) {
        self.set_checked(!self.is_checked);
    }

    /// Set the checked-change callback.
    pub fn set_on_checked_change<F>(&mut self, callback: F)
    where
        F: FnMut(bool) + 'static,
    {
        self.on_checked_change = Some(Box::new(callback));
    }

    // -- Component delegation ------------------------------------------------

    /// Get the underlying component.
    pub fn component(&self) -> &Component {
        &self.component
    }

    /// Get mutable reference to the underlying component.
    pub fn component_mut(&mut self) -> &mut Component {
        &mut self.component
    }

    /// Set bounds.
    pub fn set_bounds(&mut self, bounds: Bounds) -> Result<()> {
        self.component.set_bounds(bounds)
    }

    /// Get bounds.
    pub fn bounds(&self) -> Bounds {
        self.component.bounds()
    }

    /// Check if enabled.
    pub fn is_enabled(&self) -> bool {
        self.component.is_enabled()
    }

    /// Set enabled.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.component.set_enabled(enabled);
    }

    /// Check if visible.
    pub fn is_visible(&self) -> bool {
        self.component.is_visible()
    }

    /// Set visibility.
    pub fn set_visible(&mut self, visible: bool) {
        self.component.set_visible(visible);
    }

    /// Render the checkbox.
    #[cfg(feature = "graphics")]
    pub fn render(&self, graphics: &mut Graphics) -> Result<()> {
        let bounds = self.bounds();
        let box_size = bounds.height.min(16) as i32;
        let box_x = bounds.x;
        let box_y = bounds.y + (bounds.height as i32 - box_size) / 2;

        // Box background
        graphics.set_color(if self.is_checked {
            Color::rgb(100, 150, 220)
        } else {
            Color::rgb(255, 255, 255)
        });
        graphics.fill_rect(box_x, box_y, box_size as u32, box_size as u32);

        // Box border
        graphics.set_color(Color::rgb(0, 0, 0));
        graphics.draw_line(box_x, box_y, box_x + box_size, box_y);
        graphics.draw_line(box_x + box_size - 1, box_y, box_x + box_size - 1, box_y + box_size);
        graphics.draw_line(box_x + box_size, box_y + box_size - 1, box_x, box_y + box_size - 1);
        graphics.draw_line(box_x, box_y + box_size, box_x, box_y);

        // Check mark (simple diagonal lines)
        if self.is_checked {
            graphics.set_color(Color::rgb(255, 255, 255));
            graphics.draw_line(box_x + 3, box_y + box_size / 2, box_x + box_size / 3, box_y + box_size - 3);
            graphics.draw_line(box_x + box_size / 3, box_y + box_size - 3, box_x + box_size - 3, box_y + 3);
        }

        Ok(())
    }

    /// Render using a LookAndFeel.
    #[cfg(feature = "graphics")]
    pub fn render_with_lookandfeel(&self, graphics: &mut Graphics, laf: &dyn LookAndFeel) -> Result<()> {
        let bounds = self.bounds();
        let colors = laf.color_scheme();
        let box_size = bounds.height.min(16) as i32;
        let box_x = bounds.x;
        let box_y = bounds.y + (bounds.height as i32 - box_size) / 2;

        graphics.set_color(if self.is_checked { colors.accent } else { colors.background_secondary });
        graphics.fill_rect(box_x, box_y, box_size as u32, box_size as u32);

        graphics.set_color(colors.border);
        for i in 0..laf.border_width() as i32 {
            graphics.draw_line(box_x + i, box_y + i, box_x + box_size - i, box_y + i);
            graphics.draw_line(box_x + box_size - i - 1, box_y + i, box_x + box_size - i - 1, box_y + box_size - i);
            graphics.draw_line(box_x + box_size - i, box_y + box_size - i - 1, box_x + i, box_y + box_size - i - 1);
            graphics.draw_line(box_x + i, box_y + box_size - i, box_x + i, box_y + i);
        }

        if self.is_checked {
            graphics.set_color(colors.foreground);
            graphics.draw_line(box_x + 3, box_y + box_size / 2, box_x + box_size / 3, box_y + box_size - 3);
            graphics.draw_line(box_x + box_size / 3, box_y + box_size - 3, box_x + box_size - 3, box_y + 3);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// ProgressBar
// ---------------------------------------------------------------------------

/// A horizontal progress bar displaying a value between 0.0 and 1.0.
///
/// # Examples
///
/// ```ignore
/// use logic_nih_plug_gui::controls_extra::ProgressBar;
///
/// let mut bar = ProgressBar::new();
/// bar.set_progress(0.75);
/// assert!((bar.progress() - 0.75).abs() < f64::EPSILON);
/// ```
pub struct ProgressBar {
    component: Component,
    progress: f64,
    text: String,
}

impl ProgressBar {
    /// Create a new progress bar at 0%.
    pub fn new() -> Self {
        Self {
            component: Component::new("ProgressBar"),
            progress: 0.0,
            text: String::new(),
        }
    }

    /// Get the current progress (0.0 to 1.0).
    pub fn progress(&self) -> f64 {
        self.progress
    }

    /// Set the progress. Values are clamped to [0.0, 1.0].
    pub fn set_progress(&mut self, progress: f64) {
        self.progress = progress.clamp(0.0, 1.0);
    }

    /// Get the display text (empty string if none).
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Set the display text shown on top of the bar.
    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }

    // -- Component delegation ------------------------------------------------

    /// Get the underlying component.
    pub fn component(&self) -> &Component {
        &self.component
    }

    /// Get mutable reference to the underlying component.
    pub fn component_mut(&mut self) -> &mut Component {
        &mut self.component
    }

    /// Set bounds.
    pub fn set_bounds(&mut self, bounds: Bounds) -> Result<()> {
        self.component.set_bounds(bounds)
    }

    /// Get bounds.
    pub fn bounds(&self) -> Bounds {
        self.component.bounds()
    }

    /// Check if enabled.
    pub fn is_enabled(&self) -> bool {
        self.component.is_enabled()
    }

    /// Set enabled.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.component.set_enabled(enabled);
    }

    /// Check if visible.
    pub fn is_visible(&self) -> bool {
        self.component.is_visible()
    }

    /// Set visibility.
    pub fn set_visible(&mut self, visible: bool) {
        self.component.set_visible(visible);
    }

    /// Render the progress bar.
    #[cfg(feature = "graphics")]
    pub fn render(&self, graphics: &mut Graphics) -> Result<()> {
        let bounds = self.bounds();

        // Background track
        graphics.set_color(Color::rgb(200, 200, 200));
        graphics.fill_rect(bounds.x, bounds.y, bounds.width, bounds.height);

        // Filled portion
        let fill_width = (bounds.width as f64 * self.progress) as u32;
        graphics.set_color(Color::rgb(80, 160, 220));
        graphics.fill_rect(bounds.x, bounds.y, fill_width, bounds.height);

        // Border
        graphics.set_color(Color::rgb(0, 0, 0));
        graphics.draw_line(bounds.x, bounds.y, bounds.x + bounds.width as i32, bounds.y);
        graphics.draw_line(
            bounds.x + bounds.width as i32 - 1,
            bounds.y,
            bounds.x + bounds.width as i32 - 1,
            bounds.y + bounds.height as i32,
        );
        graphics.draw_line(
            bounds.x + bounds.width as i32,
            bounds.y + bounds.height as i32 - 1,
            bounds.x,
            bounds.y + bounds.height as i32 - 1,
        );
        graphics.draw_line(bounds.x, bounds.y + bounds.height as i32, bounds.x, bounds.y);

        Ok(())
    }

    /// Render using a LookAndFeel.
    #[cfg(feature = "graphics")]
    pub fn render_with_lookandfeel(&self, graphics: &mut Graphics, laf: &dyn LookAndFeel) -> Result<()> {
        let bounds = self.bounds();
        let colors = laf.color_scheme();

        graphics.set_color(colors.background);
        graphics.fill_rect(bounds.x, bounds.y, bounds.width, bounds.height);

        let fill_width = (bounds.width as f64 * self.progress) as u32;
        graphics.set_color(colors.accent);
        graphics.fill_rect(bounds.x, bounds.y, fill_width, bounds.height);

        graphics.set_color(colors.border);
        for i in 0..laf.border_width() as i32 {
            graphics.draw_line(bounds.x + i, bounds.y + i, bounds.x + bounds.width as i32 - i, bounds.y + i);
            graphics.draw_line(bounds.x + bounds.width as i32 - i - 1, bounds.y + i, bounds.x + bounds.width as i32 - i - 1, bounds.y + bounds.height as i32 - i);
            graphics.draw_line(bounds.x + bounds.width as i32 - i, bounds.y + bounds.height as i32 - i - 1, bounds.x + i, bounds.y + bounds.height as i32 - i - 1);
            graphics.draw_line(bounds.x + i, bounds.y + bounds.height as i32 - i, bounds.x + i, bounds.y + i);
        }

        Ok(())
    }
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tooltip
// ---------------------------------------------------------------------------

/// A tooltip manager that displays contextual help text.
///
/// Tooltips are typically shown when the mouse hovers over a component for
/// a short delay.
///
/// # Examples
///
/// ```ignore
/// use logic_nih_plug_gui::controls_extra::Tooltip;
///
/// let mut tip = Tooltip::new();
/// tip.set_text("Click to save");
/// assert_eq!(tip.text(), "Click to save");
/// ```
pub struct Tooltip {
    component: Component,
    text: String,
    is_visible: bool,
    delay_ms: u32,
}

impl Tooltip {
    /// Create a new (hidden) tooltip.
    pub fn new() -> Self {
        let mut c = Component::new("Tooltip");
        c.set_visible(false);
        Self {
            component: c,
            text: String::new(),
            is_visible: false,
            delay_ms: 700,
        }
    }

    /// Get the tooltip text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Set the tooltip text.
    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }

    /// Whether the tooltip is currently shown.
    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    /// Show the tooltip.
    pub fn show(&mut self) {
        self.is_visible = true;
        self.component.set_visible(true);
    }

    /// Hide the tooltip.
    pub fn hide(&mut self) {
        self.is_visible = false;
        self.component.set_visible(false);
    }

    /// Get the hover delay in milliseconds.
    pub fn delay_ms(&self) -> u32 {
        self.delay_ms
    }

    /// Set the hover delay in milliseconds.
    pub fn set_delay_ms(&mut self, ms: u32) {
        self.delay_ms = ms;
    }

    // -- Component delegation ------------------------------------------------

    /// Get the underlying component.
    pub fn component(&self) -> &Component {
        &self.component
    }

    /// Get mutable reference to the underlying component.
    pub fn component_mut(&mut self) -> &mut Component {
        &mut self.component
    }

    /// Set bounds.
    pub fn set_bounds(&mut self, bounds: Bounds) -> Result<()> {
        self.component.set_bounds(bounds)
    }

    /// Get bounds.
    pub fn bounds(&self) -> Bounds {
        self.component.bounds()
    }

    /// Render the tooltip.
    #[cfg(feature = "graphics")]
    pub fn render(&self, graphics: &mut Graphics) -> Result<()> {
        if !self.is_visible || self.text.is_empty() {
            return Ok(());
        }

        let bounds = self.bounds();

        // Semi-transparent background
        graphics.set_color(Color::rgb(255, 255, 220));
        graphics.fill_rect(bounds.x, bounds.y, bounds.width, bounds.height);

        // Border
        graphics.set_color(Color::rgb(0, 0, 0));
        graphics.draw_line(bounds.x, bounds.y, bounds.x + bounds.width as i32, bounds.y);
        graphics.draw_line(
            bounds.x + bounds.width as i32 - 1,
            bounds.y,
            bounds.x + bounds.width as i32 - 1,
            bounds.y + bounds.height as i32,
        );
        graphics.draw_line(
            bounds.x + bounds.width as i32,
            bounds.y + bounds.height as i32 - 1,
            bounds.x,
            bounds.y + bounds.height as i32 - 1,
        );
        graphics.draw_line(bounds.x, bounds.y + bounds.height as i32, bounds.x, bounds.y);

        Ok(())
    }

    /// Render using a LookAndFeel.
    #[cfg(feature = "graphics")]
    pub fn render_with_lookandfeel(&self, graphics: &mut Graphics, laf: &dyn LookAndFeel) -> Result<()> {
        if !self.is_visible || self.text.is_empty() {
            return Ok(());
        }

        let bounds = self.bounds();
        let colors = laf.color_scheme();

        graphics.set_color(colors.background_secondary);
        graphics.fill_rect(bounds.x, bounds.y, bounds.width, bounds.height);

        graphics.set_color(colors.border);
        for i in 0..laf.border_width() as i32 {
            graphics.draw_line(bounds.x + i, bounds.y + i, bounds.x + bounds.width as i32 - i, bounds.y + i);
            graphics.draw_line(bounds.x + bounds.width as i32 - i - 1, bounds.y + i, bounds.x + bounds.width as i32 - i - 1, bounds.y + bounds.height as i32 - i);
            graphics.draw_line(bounds.x + bounds.width as i32 - i, bounds.y + bounds.height as i32 - i - 1, bounds.x + i, bounds.y + bounds.height as i32 - i - 1);
            graphics.draw_line(bounds.x + i, bounds.y + bounds.height as i32 - i, bounds.x + i, bounds.y + i);
        }

        Ok(())
    }
}

impl Default for Tooltip {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// DrawableButton
// ---------------------------------------------------------------------------

/// A button that displays custom drawn content instead of text.
///
/// This is a placeholder for the full JUCE `DrawableButton` which renders
/// a `Drawable` object. For now it stores an optional image reference and
/// a fallback label.
///
/// # Examples
///
/// ```ignore
/// use logic_nih_plug_gui::controls_extra::DrawableButton;
///
/// let btn = DrawableButton::new("icon_btn");
/// assert_eq!(btn.name(), "icon_btn");
/// ```
pub struct DrawableButton {
    component: Component,
    name: String,
    on_click: Option<Box<dyn FnMut()>>,
}

impl DrawableButton {
    /// Create a new drawable button with the given name.
    pub fn new(name: &str) -> Self {
        Self {
            component: Component::new(&format!("DrawableButton_{}", name)),
            name: name.to_string(),
            on_click: None,
        }
    }

    /// Get the button's display name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Set the click callback.
    pub fn set_on_click<F>(&mut self, callback: F)
    where
        F: FnMut() + 'static,
    {
        self.on_click = Some(Box::new(callback));
    }

    /// Trigger the click callback.
    pub fn click(&mut self) {
        if let Some(ref mut cb) = self.on_click {
            cb();
        }
    }

    // -- Component delegation ------------------------------------------------

    /// Get the underlying component.
    pub fn component(&self) -> &Component {
        &self.component
    }

    /// Get mutable reference to the underlying component.
    pub fn component_mut(&mut self) -> &mut Component {
        &mut self.component
    }

    /// Set bounds.
    pub fn set_bounds(&mut self, bounds: Bounds) -> Result<()> {
        self.component.set_bounds(bounds)
    }

    /// Get bounds.
    pub fn bounds(&self) -> Bounds {
        self.component.bounds()
    }

    /// Check if enabled.
    pub fn is_enabled(&self) -> bool {
        self.component.is_enabled()
    }

    /// Set enabled.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.component.set_enabled(enabled);
    }

    /// Check if visible.
    pub fn is_visible(&self) -> bool {
        self.component.is_visible()
    }

    /// Set visibility.
    pub fn set_visible(&mut self, visible: bool) {
        self.component.set_visible(visible);
    }

    /// Render the button.
    #[cfg(feature = "graphics")]
    pub fn render(&self, graphics: &mut Graphics) -> Result<()> {
        let bounds = self.bounds();
        graphics.set_color(Color::rgb(200, 200, 200));
        graphics.fill_rect(bounds.x, bounds.y, bounds.width, bounds.height);

        graphics.set_color(Color::rgb(0, 0, 0));
        graphics.draw_line(bounds.x, bounds.y, bounds.x + bounds.width as i32, bounds.y);
        graphics.draw_line(
            bounds.x + bounds.width as i32 - 1,
            bounds.y,
            bounds.x + bounds.width as i32 - 1,
            bounds.y + bounds.height as i32,
        );
        graphics.draw_line(
            bounds.x + bounds.width as i32,
            bounds.y + bounds.height as i32 - 1,
            bounds.x,
            bounds.y + bounds.height as i32 - 1,
        );
        graphics.draw_line(bounds.x, bounds.y + bounds.height as i32, bounds.x, bounds.y);

        Ok(())
    }

    /// Render using a LookAndFeel.
    #[cfg(feature = "graphics")]
    pub fn render_with_lookandfeel(&self, graphics: &mut Graphics, laf: &dyn LookAndFeel) -> Result<()> {
        let bounds = self.bounds();
        let colors = laf.color_scheme();

        graphics.set_color(colors.background_secondary);
        graphics.fill_rect(bounds.x, bounds.y, bounds.width, bounds.height);

        graphics.set_color(colors.border);
        for i in 0..laf.border_width() as i32 {
            graphics.draw_line(bounds.x + i, bounds.y + i, bounds.x + bounds.width as i32 - i, bounds.y + i);
            graphics.draw_line(bounds.x + bounds.width as i32 - i - 1, bounds.y + i, bounds.x + bounds.width as i32 - i - 1, bounds.y + bounds.height as i32 - i);
            graphics.draw_line(bounds.x + bounds.width as i32 - i, bounds.y + bounds.height as i32 - i - 1, bounds.x + i, bounds.y + bounds.height as i32 - i - 1);
            graphics.draw_line(bounds.x + i, bounds.y + bounds.height as i32 - i, bounds.x + i, bounds.y + i);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// HyperlinkButton
// ---------------------------------------------------------------------------

/// A button styled as a hyperlink (underlined text).
///
/// Clicking the button invokes its callback, typically opening a URL.
///
/// # Examples
///
/// ```ignore
/// use logic_nih_plug_gui::controls_extra::HyperlinkButton;
///
/// let mut link = HyperlinkButton::new("Visit website", "https://example.com");
/// assert_eq!(link.url(), "https://example.com");
/// ```
pub struct HyperlinkButton {
    component: Component,
    text: String,
    url: String,
    on_click: Option<Box<dyn FnMut()>>,
}

impl HyperlinkButton {
    /// Create a new hyperlink button.
    pub fn new(text: &str, url: &str) -> Self {
        Self {
            component: Component::new(&format!("HyperlinkButton_{}", text)),
            text: text.to_string(),
            url: url.to_string(),
            on_click: None,
        }
    }

    /// Get the display text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Set the display text.
    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }

    /// Get the URL.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Set the URL.
    pub fn set_url(&mut self, url: &str) {
        self.url = url.to_string();
    }

    /// Set the click callback.
    pub fn set_on_click<F>(&mut self, callback: F)
    where
        F: FnMut() + 'static,
    {
        self.on_click = Some(Box::new(callback));
    }

    /// Trigger the click callback.
    pub fn click(&mut self) {
        if let Some(ref mut cb) = self.on_click {
            cb();
        }
    }

    // -- Component delegation ------------------------------------------------

    /// Get the underlying component.
    pub fn component(&self) -> &Component {
        &self.component
    }

    /// Get mutable reference to the underlying component.
    pub fn component_mut(&mut self) -> &mut Component {
        &mut self.component
    }

    /// Set bounds.
    pub fn set_bounds(&mut self, bounds: Bounds) -> Result<()> {
        self.component.set_bounds(bounds)
    }

    /// Get bounds.
    pub fn bounds(&self) -> Bounds {
        self.component.bounds()
    }

    /// Check if enabled.
    pub fn is_enabled(&self) -> bool {
        self.component.is_enabled()
    }

    /// Set enabled.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.component.set_enabled(enabled);
    }

    /// Check if visible.
    pub fn is_visible(&self) -> bool {
        self.component.is_visible()
    }

    /// Set visibility.
    pub fn set_visible(&mut self, visible: bool) {
        self.component.set_visible(visible);
    }

    /// Render the hyperlink button.
    #[cfg(feature = "graphics")]
    pub fn render(&self, graphics: &mut Graphics) -> Result<()> {
        let bounds = self.bounds();

        // Text line
        graphics.set_color(Color::rgb(0, 0, 200));
        let text_y = bounds.y + (bounds.height as i32 / 2);
        graphics.draw_line(bounds.x, text_y, bounds.x + self.text.len() as i32 * 8, text_y);

        // Underline
        let underline_y = text_y + 8;
        graphics.draw_line(bounds.x, underline_y, bounds.x + self.text.len() as i32 * 8, underline_y);

        Ok(())
    }

    /// Render using a LookAndFeel.
    #[cfg(feature = "graphics")]
    pub fn render_with_lookandfeel(&self, graphics: &mut Graphics, laf: &dyn LookAndFeel) -> Result<()> {
        let bounds = self.bounds();
        let colors = laf.color_scheme();

        graphics.set_color(colors.accent);
        let text_y = bounds.y + (bounds.height as i32 / 2);
        graphics.draw_line(bounds.x, text_y, bounds.x + self.text.len() as i32 * 8, text_y);

        let underline_y = text_y + 8;
        graphics.draw_line(bounds.x, underline_y, bounds.x + self.text.len() as i32 * 8, underline_y);

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// ImageComponent
// ---------------------------------------------------------------------------

/// Scaling mode for ImageComponent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageScalingMode {
    /// Scale to fill the component, cropping if necessary
    Fill,
    /// Scale to fit within the component, preserving aspect ratio
    Fit,
    /// No scaling — draw at original size
    None,
    /// Stretch to fill the component, ignoring aspect ratio
    Stretch,
}

/// A component that displays an image.
///
/// Supports multiple scaling modes for fitting the image into the component bounds.
///
/// # Examples
///
/// ```ignore
/// use logic_nih_plug_gui::controls_extra::{ImageComponent, ImageScalingMode};
///
/// let mut img = ImageComponent::new();
/// img.set_scaling_mode(ImageScalingMode::Fit);
/// assert_eq!(img.scaling_mode(), ImageScalingMode::Fit);
/// ```
pub struct ImageComponent {
    component: Component,
    scaling_mode: ImageScalingMode,
    has_image: bool,
}

impl ImageComponent {
    /// Create a new empty image component.
    pub fn new() -> Self {
        Self {
            component: Component::new("ImageComponent"),
            scaling_mode: ImageScalingMode::Fit,
            has_image: false,
        }
    }

    /// Get the current scaling mode.
    pub fn scaling_mode(&self) -> ImageScalingMode {
        self.scaling_mode
    }

    /// Set the scaling mode.
    pub fn set_scaling_mode(&mut self, mode: ImageScalingMode) {
        self.scaling_mode = mode;
    }

    /// Whether the component has an image loaded.
    pub fn has_image(&self) -> bool {
        self.has_image
    }

    /// Mark that an image has been loaded (for rendering tests).
    pub fn set_has_image(&mut self, has: bool) {
        self.has_image = has;
    }

    // -- Component delegation ------------------------------------------------

    /// Get the underlying component.
    pub fn component(&self) -> &Component {
        &self.component
    }

    /// Get mutable reference to the underlying component.
    pub fn component_mut(&mut self) -> &mut Component {
        &mut self.component
    }

    /// Set bounds.
    pub fn set_bounds(&mut self, bounds: Bounds) -> Result<()> {
        self.component.set_bounds(bounds)
    }

    /// Get bounds.
    pub fn bounds(&self) -> Bounds {
        self.component.bounds()
    }

    /// Check if enabled.
    pub fn is_enabled(&self) -> bool {
        self.component.is_enabled()
    }

    /// Set enabled.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.component.set_enabled(enabled);
    }

    /// Check if visible.
    pub fn is_visible(&self) -> bool {
        self.component.is_visible()
    }

    /// Set visibility.
    pub fn set_visible(&mut self, visible: bool) {
        self.component.set_visible(visible);
    }

    /// Render the image component.
    #[cfg(feature = "graphics")]
    pub fn render(&self, graphics: &mut Graphics) -> Result<()> {
        let bounds = self.bounds();

        if self.has_image {
            // Placeholder: when a real image is loaded, draw it here
            // according to the scaling mode.
            graphics.set_color(Color::rgb(220, 220, 220));
        } else {
            // Empty placeholder
            graphics.set_color(Color::rgb(240, 240, 240));
        }
        graphics.fill_rect(bounds.x, bounds.y, bounds.width, bounds.height);

        // Border
        graphics.set_color(Color::rgb(180, 180, 180));
        graphics.draw_line(bounds.x, bounds.y, bounds.x + bounds.width as i32, bounds.y);
        graphics.draw_line(
            bounds.x + bounds.width as i32 - 1,
            bounds.y,
            bounds.x + bounds.width as i32 - 1,
            bounds.y + bounds.height as i32,
        );
        graphics.draw_line(
            bounds.x + bounds.width as i32,
            bounds.y + bounds.height as i32 - 1,
            bounds.x,
            bounds.y + bounds.height as i32 - 1,
        );
        graphics.draw_line(bounds.x, bounds.y + bounds.height as i32, bounds.x, bounds.y);

        // "No image" indicator
        if !self.has_image {
            graphics.set_color(Color::rgb(160, 160, 160));
            let cx = bounds.x + (bounds.width as i32 / 2);
            let cy = bounds.y + (bounds.height as i32 / 2);
            // Draw an X
            graphics.draw_line(cx - 10, cy - 10, cx + 10, cy + 10);
            graphics.draw_line(cx + 10, cy - 10, cx - 10, cy + 10);
        }

        Ok(())
    }

    /// Render using a LookAndFeel.
    #[cfg(feature = "graphics")]
    pub fn render_with_lookandfeel(&self, graphics: &mut Graphics, laf: &dyn LookAndFeel) -> Result<()> {
        let bounds = self.bounds();
        let colors = laf.color_scheme();

        graphics.set_color(colors.background_secondary);
        graphics.fill_rect(bounds.x, bounds.y, bounds.width, bounds.height);

        graphics.set_color(colors.border);
        for i in 0..laf.border_width() as i32 {
            graphics.draw_line(bounds.x + i, bounds.y + i, bounds.x + bounds.width as i32 - i, bounds.y + i);
            graphics.draw_line(bounds.x + bounds.width as i32 - i - 1, bounds.y + i, bounds.x + bounds.width as i32 - i - 1, bounds.y + bounds.height as i32 - i);
            graphics.draw_line(bounds.x + bounds.width as i32 - i, bounds.y + bounds.height as i32 - i - 1, bounds.x + i, bounds.y + bounds.height as i32 - i - 1);
            graphics.draw_line(bounds.x + i, bounds.y + bounds.height as i32 - i, bounds.x + i, bounds.y + i);
        }

        if !self.has_image {
            graphics.set_color(colors.disabled);
            let cx = bounds.x + (bounds.width as i32 / 2);
            let cy = bounds.y + (bounds.height as i32 / 2);
            graphics.draw_line(cx - 10, cy - 10, cx + 10, cy + 10);
            graphics.draw_line(cx + 10, cy - 10, cx - 10, cy + 10);
        }

        Ok(())
    }
}

impl Default for ImageComponent {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// MidiKeyboardComponent
// ---------------------------------------------------------------------------

/// Keyboard orientation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardOrientation {
    /// Horizontal keyboard (left-to-right, like a standard piano).
    Horizontal,
    /// Vertical keyboard (bottom-to-top).
    Vertical,
}

/// A visual MIDI keyboard component.
///
/// Displays piano keys for a configurable range of MIDI note numbers and
/// fires callbacks when the user clicks on keys.  Externally-driven
/// *active notes* (e.g. from MIDI input) can be highlighted independently
/// of mouse interaction.
///
/// # Examples
///
/// ```ignore
/// use logic_nih_plug_gui::controls_extra::MidiKeyboardComponent;
/// use logic_nih_plug_gui::components::Bounds;
///
/// let mut kb = MidiKeyboardComponent::new();
/// kb.set_range(36, 84); // C2 to C6
/// kb.set_bounds(Bounds::new(0, 0, 800, 120)).unwrap();
/// ```
pub struct MidiKeyboardComponent {
    component: Component,
    /// Lowest MIDI note (inclusive). Default: 36 (C2).
    range_start: u8,
    /// Highest MIDI note (exclusive). Default: 84 (C6).
    range_end: u8,
    /// Width of one white key in pixels (default: 24).
    white_key_width: f32,
    /// Height of a white key in pixels (default: 100).
    white_key_height: f32,
    /// Black key width as a fraction of `white_key_width` (default: 0.6).
    black_key_width_ratio: f32,
    /// Black key height as a fraction of `white_key_height` (default: 0.62).
    black_key_height_ratio: f32,
    /// Layout orientation.
    orientation: KeyboardOrientation,
    /// Whether note names are shown on white keys.
    show_note_names: bool,
    /// MIDI note number that corresponds to "C4" (middle C, 60).  Controls
    /// the label shown on C keys. Default: 60.
    middle_c_number: u8,
    /// MIDI note triggered by the current mouse-down, if any.
    mouse_down_note: Option<u8>,
    /// Velocity of the current mouse-down (0.0–1.0).
    mouse_down_velocity: f32,
    /// Active notes from external MIDI input (highlighted in `active_key_color`).
    active_notes: Vec<u8>,
    /// White key fill colour.
    white_key_color: Color,
    /// Black key fill colour.
    black_key_color: Color,
    /// Highlight colour for active/pressed keys.
    active_key_color: Color,
    /// Colour for key labels and border lines.
    text_color: Color,
    /// Background colour behind the keys.
    background_color: Color,
    /// Note-on callback: `(midi_note, velocity_0_to_1)`.
    on_note_on: Option<Box<dyn FnMut(u8, f32)>>,
    /// Note-off callback: `(midi_note)`.
    on_note_off: Option<Box<dyn FnMut(u8)>>,
}

impl MidiKeyboardComponent {
    /// Create a new keyboard with default settings.
    ///
    /// Range is C2 (36) to C6 (84), horizontal orientation, 24 px white key
    /// width, 100 px height.
    pub fn new() -> Self {
        Self {
            component: Component::new("MidiKeyboard"),
            range_start: 36,
            range_end: 84,
            white_key_width: 24.0,
            white_key_height: 100.0,
            black_key_width_ratio: 0.6,
            black_key_height_ratio: 0.62,
            orientation: KeyboardOrientation::Horizontal,
            show_note_names: true,
            middle_c_number: 60,
            mouse_down_note: None,
            mouse_down_velocity: 0.0,
            active_notes: Vec::new(),
            white_key_color: Color::rgb(255, 255, 255),
            black_key_color: Color::rgb(20, 20, 20),
            active_key_color: Color::rgb(120, 180, 255),
            text_color: Color::rgb(60, 60, 60),
            background_color: Color::rgb(245, 245, 245),
            on_note_on: None,
            on_note_off: None,
        }
    }

    // -- Range ---------------------------------------------------------------

    /// Get the lowest MIDI note in the displayed range (inclusive).
    pub fn range_start(&self) -> u8 {
        self.range_start
    }

    /// Get the highest MIDI note in the displayed range (exclusive).
    pub fn range_end(&self) -> u8 {
        self.range_end
    }

    /// Set the MIDI note range.  `start` is inclusive, `end` is exclusive.
    /// Both are clamped to 0–127 and `end` is clamped to `start + 1`.
    pub fn set_range(&mut self, start: u8, end: u8) {
        self.range_start = start.min(127);
        let min_end = self.range_start.saturating_add(1);
        self.range_end = end.max(min_end).min(128);
    }

    /// Number of MIDI notes in the current range.
    pub fn num_notes(&self) -> usize {
        (self.range_start..self.range_end).count()
    }

    // -- Orientation ---------------------------------------------------------

    /// Get the current orientation.
    pub fn orientation(&self) -> KeyboardOrientation {
        self.orientation
    }

    /// Set the keyboard orientation.
    pub fn set_orientation(&mut self, orientation: KeyboardOrientation) {
        self.orientation = orientation;
    }

    // -- Key dimensions ------------------------------------------------------

    /// Width of one white key in pixels.
    pub fn white_key_width(&self) -> f32 {
        self.white_key_width
    }

    /// Set the width of one white key in pixels.
    pub fn set_white_key_width(&mut self, width: f32) {
        self.white_key_width = width.max(4.0);
    }

    /// Height of a white key in pixels.
    pub fn white_key_height(&self) -> f32 {
        self.white_key_height
    }

    /// Set the height of a white key in pixels.
    pub fn set_white_key_height(&mut self, height: f32) {
        self.white_key_height = height.max(4.0);
    }

    /// Black key width as a fraction of white key width (default 0.6).
    pub fn black_key_width_ratio(&self) -> f32 {
        self.black_key_width_ratio
    }

    /// Set the black key width ratio (0.0–1.0, clamped).
    pub fn set_black_key_width_ratio(&mut self, ratio: f32) {
        self.black_key_width_ratio = ratio.clamp(0.2, 1.0);
    }

    /// Black key height as a fraction of white key height (default 0.62).
    pub fn black_key_height_ratio(&self) -> f32 {
        self.black_key_height_ratio
    }

    /// Set the black key height ratio (0.0–1.0, clamped).
    pub fn set_black_key_height_ratio(&mut self, ratio: f32) {
        self.black_key_height_ratio = ratio.clamp(0.2, 1.0);
    }

    // -- Display options -----------------------------------------------------

    /// Whether note names are shown on white keys.
    pub fn shows_note_names(&self) -> bool {
        self.show_note_names
    }

    /// Set whether note names are shown on white keys.
    pub fn set_show_note_names(&mut self, show: bool) {
        self.show_note_names = show;
    }

    /// The MIDI note number that represents middle C (default 60).
    pub fn middle_c_number(&self) -> u8 {
        self.middle_c_number
    }

    /// Set the MIDI note number for middle C.  Controls the label on C keys.
    pub fn set_middle_c_number(&mut self, note: u8) {
        self.middle_c_number = note.min(127);
    }

    // -- Colours -------------------------------------------------------------

    /// Set the white key colour.
    pub fn set_white_key_color(&mut self, color: Color) {
        self.white_key_color = color;
    }

    /// Set the black key colour.
    pub fn set_black_key_color(&mut self, color: Color) {
        self.black_key_color = color;
    }

    /// Set the highlight colour for active/pressed keys.
    pub fn set_active_key_color(&mut self, color: Color) {
        self.active_key_color = color;
    }

    /// Set the text / border colour.
    pub fn set_text_color(&mut self, color: Color) {
        self.text_color = color;
    }

    /// Set the background colour.
    pub fn set_background_color(&mut self, color: Color) {
        self.background_color = color;
    }

    // -- Active notes --------------------------------------------------------

    /// Get the list of externally-active MIDI notes.
    pub fn active_notes(&self) -> &[u8] {
        &self.active_notes
    }

    /// Set the entire list of active notes (replaces any previous list).
    pub fn set_active_notes(&mut self, notes: Vec<u8>) {
        self.active_notes = notes;
    }

    /// Add a note to the active set (no-op if already present or out of range).
    pub fn add_active_note(&mut self, note: u8) {
        if note >= self.range_start
            && note < self.range_end
            && !self.active_notes.contains(&note)
        {
            self.active_notes.push(note);
        }
    }

    /// Remove a note from the active set.
    pub fn remove_active_note(&mut self, note: u8) {
        self.active_notes.retain(|&n| n != note);
    }

    /// Clear all active notes.
    pub fn clear_active_notes(&mut self) {
        self.active_notes.clear();
    }

    /// Returns `true` if the given note is active (either via mouse or external).
    pub fn is_note_active(&self, note: u8) -> bool {
        self.active_notes.contains(&note) || self.mouse_down_note == Some(note)
    }

    // -- Mouse interaction ---------------------------------------------------

    /// The MIDI note currently held down by mouse interaction, if any.
    pub fn mouse_down_note(&self) -> Option<u8> {
        self.mouse_down_note
    }

    /// Current mouse-down velocity (0.0–1.0).
    pub fn mouse_down_velocity(&self) -> f32 {
        self.mouse_down_velocity
    }

    /// Simulate a mouse press at the given pixel position (relative to the
    /// component bounds).  Returns the MIDI note hit, if any.
    pub fn mouse_down(&mut self, x: i32, y: i32) -> Option<u8> {
        let note = self.note_at_position(x, y);
        if let Some(n) = note {
            let vel = self.velocity_at_position(x, y);
            self.mouse_down_note = Some(n);
            self.mouse_down_velocity = vel;
            if let Some(ref mut cb) = self.on_note_on {
                cb(n, vel);
            }
        }
        note
    }

    /// Simulate a mouse release.  Fires the note-off callback for the
    /// currently-held note, if any.
    pub fn mouse_up(&mut self) -> Option<u8> {
        if let Some(note) = self.mouse_down_note.take() {
            self.mouse_down_velocity = 0.0;
            if let Some(ref mut cb) = self.on_note_off {
                cb(note);
            }
            Some(note)
        } else {
            None
        }
    }

    /// Simulate a mouse drag.  If the user drags off the current key, the
    /// old key is released and the new one is pressed.
    pub fn mouse_drag(&mut self, x: i32, y: i32) {
        let new_note = self.note_at_position(x, y);
        if new_note != self.mouse_down_note {
            // Release old
            if let Some(old) = self.mouse_down_note.take() {
                self.mouse_down_velocity = 0.0;
                if let Some(ref mut cb) = self.on_note_off {
                    cb(old);
                }
            }
            // Press new
            if let Some(n) = new_note {
                let vel = self.velocity_at_position(x, y);
                self.mouse_down_note = Some(n);
                self.mouse_down_velocity = vel;
                if let Some(ref mut cb) = self.on_note_on {
                    cb(n, vel);
                }
            }
        }
    }

    // -- Callbacks -----------------------------------------------------------

    /// Set the note-on callback: `(midi_note, velocity_0_to_1)`.
    pub fn set_on_note_on<F>(&mut self, callback: F)
    where
        F: FnMut(u8, f32) + 'static,
    {
        self.on_note_on = Some(Box::new(callback));
    }

    /// Set the note-off callback: `(midi_note)`.
    pub fn set_on_note_off<F>(&mut self, callback: F)
    where
        F: FnMut(u8) + 'static,
    {
        self.on_note_off = Some(Box::new(callback));
    }

    // -- Key classification (public helpers) ---------------------------------

    /// Returns `true` if the MIDI note number corresponds to a black key.
    pub fn is_black_key(note: u8) -> bool {
        matches!(note % 12, 1 | 3 | 6 | 8 | 10)
    }

    /// Returns the note name (e.g. `"C4"`, `"F#3"`) for a MIDI note number,
    /// using this component's `middle_c_number` setting.
    pub fn note_name(&self, note: u8) -> String {
        const NOTE_NAMES: [&str; 12] = [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ];
        let name = NOTE_NAMES[(note % 12) as usize];
        let octave = (note as i32 / 12) - (self.middle_c_number as i32 / 12) + 4;
        format!("{}{}", name, octave)
    }

    /// Number of white keys in the current range.
    pub fn num_white_keys(&self) -> usize {
        (self.range_start..self.range_end)
            .filter(|&n| !Self::is_black_key(n))
            .count()
    }

    // -- Hit testing ---------------------------------------------------------

    /// Determine which MIDI note (if any) is at the given pixel coordinate
    /// (relative to the component's top-left corner).
    pub fn note_at_position(&self, x: i32, y: i32) -> Option<u8> {
        // Check black keys first (they are drawn on top).
        for note in self.range_start..self.range_end {
            if !Self::is_black_key(note) {
                continue;
            }
            let (kx, ky, kw, kh) = self.key_rect(note);
            // Convert to i32 for comparison
            let kx_i = kx as i32;
            let ky_i = ky as i32;
            let kw_i = kw as i32;
            let kh_i = kh as i32;
            if x >= kx_i && x < kx_i + kw_i && y >= ky_i && y < ky_i + kh_i {
                return Some(note);
            }
        }

        // Check white keys.
        for note in self.range_start..self.range_end {
            if Self::is_black_key(note) {
                continue;
            }
            let (kx, ky, kw, kh) = self.key_rect(note);
            let kx_i = kx as i32;
            let ky_i = ky as i32;
            let kw_i = kw as i32;
            let kh_i = kh as i32;
            if x >= kx_i && x < kx_i + kw_i && y >= ky_i && y < ky_i + kh_i {
                return Some(note);
            }
        }

        None
    }

    /// Compute a velocity (0.0–1.0) from the vertical position within a key.
    /// Top of key = 1.0, bottom = 0.1.
    pub fn velocity_at_position(&self, _x: i32, y: i32) -> f32 {
        let bounds = self.bounds();
        if self.orientation == KeyboardOrientation::Horizontal {
            let key_h = bounds.height as f32;
            let rel = (y as f32 - bounds.y as f32) / key_h;
            (1.0 - rel.clamp(0.0, 1.0)).max(0.1)
        } else {
            // Vertical: left = soft, right = hard
            let key_w = bounds.width as f32;
            let rel = (y as f32 - bounds.y as f32) / key_w;
            rel.clamp(0.0, 1.0).max(0.1)
        }
    }

    // -- Internal key geometry -----------------------------------------------

    /// Compute `(x, y, width, height)` for a single key in component-local
    /// coordinates (as `f32`).
    fn key_rect(&self, note: u8) -> (f32, f32, f32, f32) {
        let bounds = self.bounds();
        let x0 = bounds.x as f32;
        let y0 = bounds.y as f32;

        if Self::is_black_key(note) {
            // Black key: centred between adjacent white keys.
            let center_idx = self.black_key_center_index(note);
            let bw = self.white_key_width * self.black_key_width_ratio;
            let bh = self.white_key_height * self.black_key_height_ratio;

            if self.orientation == KeyboardOrientation::Horizontal {
                let cx = x0 + (center_idx + 0.5) * self.white_key_width;
                (cx - bw * 0.5, y0, bw, bh)
            } else {
                // Vertical: swap x/y, width/height
                let cy = y0 + (center_idx + 0.5) * self.white_key_width;
                (x0, cy - bw * 0.5, bh, bw)
            }
        } else {
            // White key.
            let wi = self.white_key_index(note).unwrap_or(0) as f32;
            if self.orientation == KeyboardOrientation::Horizontal {
                (
                    x0 + wi * self.white_key_width,
                    y0,
                    self.white_key_width,
                    self.white_key_height,
                )
            } else {
                (
                    x0,
                    y0 + wi * self.white_key_width,
                    self.white_key_height,
                    self.white_key_width,
                )
            }
        }
    }

    /// White key index (0-based) for a white note within the range.
    fn white_key_index(&self, note: u8) -> Option<usize> {
        if Self::is_black_key(note) || note < self.range_start || note >= self.range_end {
            return None;
        }
        let mut idx = 0;
        for n in self.range_start..note {
            if !Self::is_black_key(n) {
                idx += 1;
            }
        }
        Some(idx)
    }

    /// Centre white-key index for a black key.  The black key is drawn
    /// centred between its two surrounding white keys.
    fn black_key_center_index(&self, note: u8) -> f32 {
        assert!(Self::is_black_key(note));

        let note_in_octave = note % 12;
        // Offset of this black key's center within its octave, measured in
        // white-key widths from the octave's C.
        let offset_in_octave: f32 = match note_in_octave {
            1 => 0.6,  // C# — between C(0) and D(1)
            3 => 1.6,  // D# — between D(1) and E(2)
            6 => 3.6,  // F# — between F(3) and G(4)
            8 => 4.6,  // G# — between G(4) and A(5)
            10 => 5.6, // A# — between A(5) and B(6)
            _ => unreachable!("not a black key"),
        };

        // Number of white keys before the C of this note's octave.
        let octave_c = (note / 12) * 12;
        let white_before: usize = (self.range_start..octave_c)
            .filter(|&n| !Self::is_black_key(n))
            .count();

        (white_before as f32) + offset_in_octave - 0.5
        // -0.5 so that the returned value can be used directly as the left
        // edge's white-key index when computing center: (idx + 0.5) * ww.
    }

    // -- Component delegation ------------------------------------------------

    /// Get the underlying component.
    pub fn component(&self) -> &Component {
        &self.component
    }

    /// Get a mutable reference to the underlying component.
    pub fn component_mut(&mut self) -> &mut Component {
        &mut self.component
    }

    /// Set the component bounds.
    pub fn set_bounds(&mut self, bounds: Bounds) -> Result<()> {
        self.component.set_bounds(bounds)
    }

    /// Get the component bounds.
    pub fn bounds(&self) -> Bounds {
        self.component.bounds()
    }

    /// Check if enabled.
    pub fn is_enabled(&self) -> bool {
        self.component.is_enabled()
    }

    /// Set enabled state.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.component.set_enabled(enabled);
    }

    /// Check visibility.
    pub fn is_visible(&self) -> bool {
        self.component.is_visible()
    }

    /// Set visibility.
    pub fn set_visible(&mut self, visible: bool) {
        self.component.set_visible(visible);
    }

    // -- Rendering -----------------------------------------------------------

    /// Render the MIDI keyboard to a graphics context.
    #[cfg(feature = "graphics")]
    pub fn render(&self, graphics: &mut Graphics) -> Result<()> {
        let bounds = self.bounds();

        // Background
        graphics.set_color(self.background_color);
        graphics.fill_rect(bounds.x, bounds.y, bounds.width, bounds.height);

        // Draw white keys first
        for note in self.range_start..self.range_end {
            if Self::is_black_key(note) {
                continue;
            }
            let (kx, ky, kw, kh) = self.key_rect(note);
            let is_active = self.is_note_active(note);
            let fill = if is_active {
                self.active_key_color
            } else {
                self.white_key_color
            };
            graphics.set_color(fill);
            graphics.fill_rect(kx as i32, ky as i32, kw as u32, kh as u32);

            // Border
            graphics.set_color(self.text_color);
            graphics.draw_line(kx as i32, ky as i32, kx as i32 + kw as i32, ky as i32);
            graphics.draw_line(
                kx as i32 + kw as i32,
                ky as i32,
                kx as i32 + kw as i32,
                ky as i32 + kh as i32,
            );
            graphics.draw_line(
                kx as i32,
                ky as i32 + kh as i32,
                kx as i32 + kw as i32,
                ky as i32 + kh as i32,
            );

            // Note name (simplified: draw a vertical line segment as placeholder
            // for text — real impl would use Font/GlyphArrangement).
            if self.show_note_names && (note % 12 == 0) {
                // Mark C keys with a short horizontal tick at the bottom
                let tick_y = ky as i32 + kh as i32 - 8;
                let tick_cx = kx as i32 + kw as i32 / 2;
                graphics.draw_line(tick_cx - 3, tick_y, tick_cx + 3, tick_y);
            }
        }

        // Draw black keys on top
        for note in self.range_start..self.range_end {
            if !Self::is_black_key(note) {
                continue;
            }
            let (kx, ky, kw, kh) = self.key_rect(note);
            let is_active = self.is_note_active(note);
            let fill = if is_active {
                self.active_key_color
            } else {
                self.black_key_color
            };
            graphics.set_color(fill);
            graphics.fill_rect(kx as i32, ky as i32, kw as u32, kh as u32);

            // Border
            graphics.set_color(self.text_color);
            graphics.draw_line(kx as i32, ky as i32, kx as i32 + kw as i32, ky as i32);
            graphics.draw_line(
                kx as i32 + kw as i32,
                ky as i32,
                kx as i32 + kw as i32,
                ky as i32 + kh as i32,
            );
            graphics.draw_line(
                kx as i32,
                ky as i32 + kh as i32,
                kx as i32 + kw as i32,
                ky as i32 + kh as i32,
            );
            graphics.draw_line(kx as i32, ky as i32, kx as i32, ky as i32 + kh as i32);
        }

        Ok(())
    }

    /// Render using a LookAndFeel.
    #[cfg(feature = "graphics")]
    pub fn render_with_lookandfeel(
        &self,
        graphics: &mut Graphics,
        laf: &dyn LookAndFeel,
    ) -> Result<()> {
        let bounds = self.bounds();
        let colors = laf.color_scheme();

        // Background
        graphics.set_color(colors.background_secondary);
        graphics.fill_rect(bounds.x, bounds.y, bounds.width, bounds.height);

        // White keys
        for note in self.range_start..self.range_end {
            if Self::is_black_key(note) {
                continue;
            }
            let (kx, ky, kw, kh) = self.key_rect(note);
            let fill = if self.is_note_active(note) {
                colors.accent
            } else {
                colors.background
            };
            graphics.set_color(fill);
            graphics.fill_rect(kx as i32, ky as i32, kw as u32, kh as u32);

            graphics.set_color(colors.border);
            graphics.draw_line(kx as i32, ky as i32, kx as i32 + kw as i32, ky as i32);
            graphics.draw_line(
                kx as i32 + kw as i32,
                ky as i32,
                kx as i32 + kw as i32,
                ky as i32 + kh as i32,
            );
            graphics.draw_line(
                kx as i32,
                ky as i32 + kh as i32,
                kx as i32 + kw as i32,
                ky as i32 + kh as i32,
            );
        }

        // Black keys
        for note in self.range_start..self.range_end {
            if !Self::is_black_key(note) {
                continue;
            }
            let (kx, ky, kw, kh) = self.key_rect(note);
            let fill = if self.is_note_active(note) {
                colors.accent
            } else {
                colors.foreground
            };
            graphics.set_color(fill);
            graphics.fill_rect(kx as i32, ky as i32, kw as u32, kh as u32);

            graphics.set_color(colors.border);
            graphics.draw_line(kx as i32, ky as i32, kx as i32 + kw as i32, ky as i32);
            graphics.draw_line(
                kx as i32 + kw as i32,
                ky as i32,
                kx as i32 + kw as i32,
                ky as i32 + kh as i32,
            );
            graphics.draw_line(
                kx as i32,
                ky as i32 + kh as i32,
                kx as i32 + kw as i32,
                ky as i32 + kh as i32,
            );
            graphics.draw_line(kx as i32, ky as i32, kx as i32, ky as i32 + kh as i32);
        }

        Ok(())
    }
}

impl Default for MidiKeyboardComponent {
    fn default() -> Self {
        Self::new()
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- ComboBox ----

    #[test]
    fn combo_box_creation() {
        let cb = ComboBox::new();
        assert_eq!(cb.num_items(), 0);
        assert!(cb.selected_index().is_none());
        assert!(cb.selected_item().is_none());
    }

    #[test]
    fn combo_box_add_items() {
        let mut cb = ComboBox::new();
        cb.add_item("Alpha");
        cb.add_item("Beta");
        cb.add_item("Gamma");
        assert_eq!(cb.num_items(), 3);
        assert_eq!(cb.item(0), Some("Alpha"));
        assert_eq!(cb.item(1), Some("Beta"));
        assert_eq!(cb.item(2), Some("Gamma"));
        assert_eq!(cb.item(3), None);
    }

    #[test]
    fn combo_box_select() {
        let mut cb = ComboBox::new();
        cb.add_item("Alpha");
        cb.add_item("Beta");
        cb.set_selected_index(Some(1));
        assert_eq!(cb.selected_index(), Some(1));
        assert_eq!(cb.selected_item(), Some("Beta"));
    }

    #[test]
    fn combo_box_select_out_of_range() {
        let mut cb = ComboBox::new();
        cb.add_item("Alpha");
        cb.set_selected_index(Some(5));
        // Should not change
        assert!(cb.selected_index().is_none());
    }

    #[test]
    fn combo_box_deselect() {
        let mut cb = ComboBox::new();
        cb.add_item("Alpha");
        cb.set_selected_index(Some(0));
        assert_eq!(cb.selected_index(), Some(0));
        cb.set_selected_index(None);
        assert!(cb.selected_index().is_none());
    }

    #[test]
    fn combo_box_clear() {
        let mut cb = ComboBox::new();
        cb.add_item("Alpha");
        cb.add_item("Beta");
        cb.set_selected_index(Some(0));
        cb.clear();
        assert_eq!(cb.num_items(), 0);
        assert!(cb.selected_index().is_none());
    }

    #[test]
    fn combo_box_on_change() {
        use std::sync::{Arc, Mutex};

        let mut cb = ComboBox::new();
        cb.add_item("Alpha");
        cb.add_item("Beta");

        let last = Arc::new(Mutex::new((usize::MAX, String::new())));
        let last_clone = last.clone();
        cb.set_on_change(move |i, s| {
            *last_clone.lock().unwrap() = (i, s.to_string());
        });

        cb.set_selected_index(Some(1));
        let (idx, txt) = &*last.lock().unwrap();
        assert_eq!(*idx, 1);
        assert_eq!(txt, "Beta");
    }

    #[test]
    fn combo_box_default() {
        let cb = ComboBox::default();
        assert_eq!(cb.num_items(), 0);
    }

    // ---- TextEditor ----

    #[test]
    fn text_editor_creation() {
        let ed = TextEditor::new();
        assert_eq!(ed.text(), "");
        assert!(!ed.is_multiline());
        assert!(!ed.is_read_only());
        assert_eq!(ed.cursor_position(), 0);
    }

    #[test]
    fn text_editor_set_text() {
        let mut ed = TextEditor::new();
        ed.set_text("Hello");
        assert_eq!(ed.text(), "Hello");
        assert_eq!(ed.cursor_position(), 0);
    }

    #[test]
    fn text_editor_clear() {
        let mut ed = TextEditor::new();
        ed.set_text("Hello");
        ed.clear();
        assert_eq!(ed.text(), "");
    }

    #[test]
    fn text_editor_max_length() {
        let mut ed = TextEditor::new();
        ed.set_max_length(Some(3));
        ed.set_text("Hello");
        assert_eq!(ed.text(), "Hel");
    }

    #[test]
    fn text_editor_insert_text() {
        let mut ed = TextEditor::new();
        ed.set_text("Hel");
        ed.set_cursor_position(3);
        ed.insert_text("lo");
        assert_eq!(ed.text(), "Hello");
        assert_eq!(ed.cursor_position(), 5);
    }

    #[test]
    fn text_editor_insert_at_middle() {
        let mut ed = TextEditor::new();
        ed.set_text("Hlo");
        ed.set_cursor_position(1);
        ed.insert_text("el");
        assert_eq!(ed.text(), "Hello");
        assert_eq!(ed.cursor_position(), 3);
    }

    #[test]
    fn text_editor_read_only() {
        let mut ed = TextEditor::new();
        ed.set_read_only(true);
        ed.set_text("Hello");
        ed.insert_text("World");
        // Text should remain unchanged because it's read-only and set_text
        // only checks read_only for insert, not set_text itself.
        // Actually set_text doesn't check read_only — let's verify behavior:
        // set_text replaces; insert_text respects read_only.
        ed.clear();
        // clear() calls set_text("") which still works even in read_only
        // — that's consistent with JUCE where clear() works on read-only too.
        // But insert should not.
        ed.set_text("Hello");
        ed.set_cursor_position(0);
        ed.insert_text("X");
        assert_eq!(ed.text(), "Hello");
    }

    #[test]
    fn text_editor_multiline() {
        let ed = TextEditor::multiline();
        assert!(ed.is_multiline());
    }

    #[test]
    fn text_editor_on_text_change() {
        use std::sync::{Arc, Mutex};

        let mut ed = TextEditor::new();
        let last = Arc::new(Mutex::new(String::new()));
        let last_clone = last.clone();
        ed.set_on_text_change(move |s| {
            *last_clone.lock().unwrap() = s.to_string();
        });

        ed.set_text("Hello");
        assert_eq!(*last.lock().unwrap(), "Hello");
    }

    #[test]
    fn text_editor_cursor_clamped() {
        let mut ed = TextEditor::new();
        ed.set_text("Hi");
        ed.set_cursor_position(100);
        assert_eq!(ed.cursor_position(), 2);
    }

    // ---- ToggleButton ----

    #[test]
    fn toggle_button_creation() {
        let tb = ToggleButton::new("Enable");
        assert_eq!(tb.text(), "Enable");
        assert!(!tb.is_on());
    }

    #[test]
    fn toggle_button_set_on() {
        let mut tb = ToggleButton::new("Enable");
        tb.set_on(true);
        assert!(tb.is_on());
    }

    #[test]
    fn toggle_button_toggle() {
        let mut tb = ToggleButton::new("Enable");
        tb.toggle();
        assert!(tb.is_on());
        tb.toggle();
        assert!(!tb.is_on());
    }

    #[test]
    fn toggle_button_on_toggle() {
        use std::sync::{Arc, Mutex};

        let mut tb = ToggleButton::new("Enable");
        let last = Arc::new(Mutex::new(false));
        let last_clone = last.clone();
        tb.set_on_toggle(move |v| {
            *last_clone.lock().unwrap() = v;
        });

        tb.toggle();
        assert!(*last.lock().unwrap());

        tb.toggle();
        assert!(!*last.lock().unwrap());
    }

    #[test]
    fn toggle_button_no_double_fire() {
        use std::sync::{Arc, Mutex};

        let mut tb = ToggleButton::new("Enable");
        let count = Arc::new(Mutex::new(0u32));
        let count_clone = count.clone();
        tb.set_on_toggle(move |_| {
            *count_clone.lock().unwrap() += 1;
        });

        tb.set_on(true);
        tb.set_on(true); // Same value, should not fire
        assert_eq!(*count.lock().unwrap(), 1);
    }

    // ---- CheckBox ----

    #[test]
    fn check_box_creation() {
        let cb = CheckBox::new("Accept");
        assert_eq!(cb.label(), "Accept");
        assert!(!cb.is_checked());
    }

    #[test]
    fn check_box_set_checked() {
        let mut cb = CheckBox::new("Accept");
        cb.set_checked(true);
        assert!(cb.is_checked());
    }

    #[test]
    fn check_box_toggle() {
        let mut cb = CheckBox::new("Accept");
        cb.toggle();
        assert!(cb.is_checked());
        cb.toggle();
        assert!(!cb.is_checked());
    }

    #[test]
    fn check_box_on_checked_change() {
        use std::sync::{Arc, Mutex};

        let mut cb = CheckBox::new("Accept");
        let last = Arc::new(Mutex::new(false));
        let last_clone = last.clone();
        cb.set_on_checked_change(move |v| {
            *last_clone.lock().unwrap() = v;
        });

        cb.set_checked(true);
        assert!(*last.lock().unwrap());

        cb.set_checked(true); // Same value, should not fire
        assert!(*last.lock().unwrap()); // Still true from first call
    }

    #[test]
    fn check_box_no_double_fire() {
        use std::sync::{Arc, Mutex};

        let mut cb = CheckBox::new("Accept");
        let count = Arc::new(Mutex::new(0u32));
        let count_clone = count.clone();
        cb.set_on_checked_change(move |_| {
            *count_clone.lock().unwrap() += 1;
        });

        cb.set_checked(true);
        cb.set_checked(true); // Same value
        assert_eq!(*count.lock().unwrap(), 1);
    }

    // ---- ProgressBar ----

    #[test]
    fn progress_bar_creation() {
        let bar = ProgressBar::new();
        assert!((bar.progress() - 0.0).abs() < f64::EPSILON);
        assert_eq!(bar.text(), "");
    }

    #[test]
    fn progress_bar_set_progress() {
        let mut bar = ProgressBar::new();
        bar.set_progress(0.5);
        assert!((bar.progress() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn progress_bar_clamp() {
        let mut bar = ProgressBar::new();
        bar.set_progress(1.5);
        assert!((bar.progress() - 1.0).abs() < f64::EPSILON);
        bar.set_progress(-0.5);
        assert!((bar.progress() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn progress_bar_text() {
        let mut bar = ProgressBar::new();
        bar.set_text("75%");
        assert_eq!(bar.text(), "75%");
    }

    #[test]
    fn progress_bar_default() {
        let bar = ProgressBar::default();
        assert!((bar.progress() - 0.0).abs() < f64::EPSILON);
    }

    // ---- Tooltip ----

    #[test]
    fn tooltip_creation() {
        let tip = Tooltip::new();
        assert_eq!(tip.text(), "");
        assert!(!tip.is_visible());
    }

    #[test]
    fn tooltip_show_hide() {
        let mut tip = Tooltip::new();
        tip.set_text("Help text");
        tip.show();
        assert!(tip.is_visible());
        tip.hide();
        assert!(!tip.is_visible());
    }

    #[test]
    fn tooltip_delay() {
        let mut tip = Tooltip::new();
        assert_eq!(tip.delay_ms(), 700);
        tip.set_delay_ms(300);
        assert_eq!(tip.delay_ms(), 300);
    }

    #[test]
    fn tooltip_default() {
        let tip = Tooltip::default();
        assert!(!tip.is_visible());
    }

    // ---- DrawableButton ----

    #[test]
    fn drawable_button_creation() {
        let btn = DrawableButton::new("icon");
        assert_eq!(btn.name(), "icon");
        assert!(btn.is_enabled());
    }

    #[test]
    fn drawable_button_click() {
        use std::sync::{Arc, Mutex};

        let mut btn = DrawableButton::new("icon");
        let clicked = Arc::new(Mutex::new(false));
        let clicked_clone = clicked.clone();
        btn.set_on_click(move || {
            *clicked_clone.lock().unwrap() = true;
        });

        btn.click();
        assert!(*clicked.lock().unwrap());
    }

    // ---- HyperlinkButton ----

    #[test]
    fn hyperlink_button_creation() {
        let link = HyperlinkButton::new("Visit", "https://example.com");
        assert_eq!(link.text(), "Visit");
        assert_eq!(link.url(), "https://example.com");
    }

    #[test]
    fn hyperlink_button_set_url() {
        let mut link = HyperlinkButton::new("Visit", "https://example.com");
        link.set_url("https://other.com");
        assert_eq!(link.url(), "https://other.com");
    }

    #[test]
    fn hyperlink_button_click() {
        use std::sync::{Arc, Mutex};

        let mut link = HyperlinkButton::new("Visit", "https://example.com");
        let clicked = Arc::new(Mutex::new(false));
        let clicked_clone = clicked.clone();
        link.set_on_click(move || {
            *clicked_clone.lock().unwrap() = true;
        });

        link.click();
        assert!(*clicked.lock().unwrap());
    }

    // ---- ImageComponent ----

    #[test]
    fn image_component_creation() {
        let img = ImageComponent::new();
        assert!(!img.has_image());
        assert_eq!(img.scaling_mode(), ImageScalingMode::Fit);
    }

    #[test]
    fn image_component_scaling_modes() {
        let mut img = ImageComponent::new();

        img.set_scaling_mode(ImageScalingMode::Fill);
        assert_eq!(img.scaling_mode(), ImageScalingMode::Fill);

        img.set_scaling_mode(ImageScalingMode::Stretch);
        assert_eq!(img.scaling_mode(), ImageScalingMode::Stretch);

        img.set_scaling_mode(ImageScalingMode::None);
        assert_eq!(img.scaling_mode(), ImageScalingMode::None);
    }

    #[test]
    fn image_component_set_has_image() {
        let mut img = ImageComponent::new();
        assert!(!img.has_image());
        img.set_has_image(true);
        assert!(img.has_image());
        img.set_has_image(false);
        assert!(!img.has_image());
    }

    #[test]
    fn image_component_default() {
        let img = ImageComponent::default();
        assert_eq!(img.scaling_mode(), ImageScalingMode::Fit);
    }

    // ---- MidiKeyboardComponent ----

    #[test]
    fn midi_keyboard_creation() {
        let kb = MidiKeyboardComponent::new();
        assert_eq!(kb.range_start(), 36);
        assert_eq!(kb.range_end(), 84);
        assert_eq!(kb.orientation(), KeyboardOrientation::Horizontal);
        assert_eq!(kb.white_key_width(), 24.0);
        assert_eq!(kb.white_key_height(), 100.0);
        assert!(kb.shows_note_names());
        assert!(kb.active_notes().is_empty());
        assert!(kb.mouse_down_note().is_none());
    }

    #[test]
    fn midi_keyboard_default() {
        let kb = MidiKeyboardComponent::default();
        assert_eq!(kb.range_start(), 36);
        assert_eq!(kb.range_end(), 84);
    }

    #[test]
    fn midi_keyboard_set_range() {
        let mut kb = MidiKeyboardComponent::new();
        kb.set_range(48, 72); // C4 to C6
        assert_eq!(kb.range_start(), 48);
        assert_eq!(kb.range_end(), 72);
        assert_eq!(kb.num_notes(), 24);
    }

    #[test]
    fn midi_keyboard_set_range_clamping() {
        let mut kb = MidiKeyboardComponent::new();
        // end < start + 1
        kb.set_range(60, 55);
        assert_eq!(kb.range_start(), 60);
        assert_eq!(kb.range_end(), 61);
        // start > 127
        kb.set_range(130, 140);
        assert_eq!(kb.range_start(), 127);
        assert_eq!(kb.range_end(), 128);
    }

    #[test]
    fn midi_keyboard_is_black_key() {
        assert!(!MidiKeyboardComponent::is_black_key(0));  // C
        assert!(MidiKeyboardComponent::is_black_key(1));   // C#
        assert!(!MidiKeyboardComponent::is_black_key(2));  // D
        assert!(MidiKeyboardComponent::is_black_key(3));   // D#
        assert!(!MidiKeyboardComponent::is_black_key(4));  // E
        assert!(!MidiKeyboardComponent::is_black_key(5));  // F
        assert!(MidiKeyboardComponent::is_black_key(6));   // F#
        assert!(!MidiKeyboardComponent::is_black_key(7));  // G
        assert!(MidiKeyboardComponent::is_black_key(8));   // G#
        assert!(!MidiKeyboardComponent::is_black_key(9));  // A
        assert!(MidiKeyboardComponent::is_black_key(10));  // A#
        assert!(!MidiKeyboardComponent::is_black_key(11)); // B
    }

    #[test]
    fn midi_keyboard_note_name() {
        let mut kb = MidiKeyboardComponent::new();
        assert_eq!(kb.note_name(60), "C4");
        assert_eq!(kb.note_name(61), "C#4");
        assert_eq!(kb.note_name(69), "A4");
        assert_eq!(kb.note_name(48), "C3");
        assert_eq!(kb.note_name(72), "C5");
        // Middle C at 48
        kb.set_middle_c_number(48);
        assert_eq!(kb.note_name(48), "C4");
    }

    #[test]
    fn midi_keyboard_num_white_keys() {
        let mut kb = MidiKeyboardComponent::new();
        kb.set_range(0, 12); // One octave C to B
        assert_eq!(kb.num_white_keys(), 7); // C D E F G A B
        kb.set_range(0, 24); // Two octaves
        assert_eq!(kb.num_white_keys(), 14);
    }

    #[test]
    fn midi_keyboard_active_notes() {
        let mut kb = MidiKeyboardComponent::new();
        kb.add_active_note(60);
        kb.add_active_note(64);
        kb.add_active_note(67);
        assert_eq!(kb.active_notes(), &[60, 64, 67]);
        assert!(kb.is_note_active(60));
        assert!(kb.is_note_active(64));
        assert!(!kb.is_note_active(62));

        // Duplicate add
        kb.add_active_note(60);
        assert_eq!(kb.active_notes().len(), 3);

        // Out of range
        kb.add_active_note(0);
        assert_eq!(kb.active_notes().len(), 3);

        kb.remove_active_note(64);
        assert_eq!(kb.active_notes(), &[60, 67]);
        assert!(!kb.is_note_active(64));

        kb.clear_active_notes();
        assert!(kb.active_notes().is_empty());
    }

    #[test]
    fn midi_keyboard_set_active_notes() {
        let mut kb = MidiKeyboardComponent::new();
        kb.set_active_notes(vec![60, 62, 64]);
        assert_eq!(kb.active_notes(), &[60, 62, 64]);
    }

    #[test]
    fn midi_keyboard_mouse_interaction() {
        use std::sync::{Arc, Mutex};

        let mut kb = MidiKeyboardComponent::new();
        let on_on = Arc::new(Mutex::new(Vec::<(u8, f32)>::new()));
        let on_off = Arc::new(Mutex::new(Vec::<u8>::new()));
        let on_on_c = on_on.clone();
        let on_off_c = on_off.clone();

        kb.set_on_note_on(move |n, v| {
            on_on_c.lock().unwrap().push((n, v));
        });
        kb.set_on_note_off(move |n| {
            on_off_c.lock().unwrap().push(n);
        });

        kb.set_bounds(Bounds::new(0, 0, 240, 100)).unwrap();

        // Mouse down on a white key (C2 = note 36, first key)
        let note = kb.mouse_down(5, 50);
        assert!(note.is_some());
        assert_eq!(kb.mouse_down_note(), note);
        assert!(kb.mouse_down_velocity() > 0.0);

        // Mouse up
        let released = kb.mouse_up();
        assert_eq!(released, note);
        assert!(kb.mouse_down_note().is_none());

        // Check callbacks fired
        assert_eq!(on_on.lock().unwrap().len(), 1);
        assert_eq!(on_off.lock().unwrap().len(), 1);
        assert_eq!(on_off.lock().unwrap()[0], note.unwrap());
    }

    #[test]
    fn midi_keyboard_mouse_drag() {
        use std::sync::{Arc, Mutex};

        let mut kb = MidiKeyboardComponent::new();
        let notes_on = Arc::new(Mutex::new(Vec::<u8>::new()));
        let notes_off = Arc::new(Mutex::new(Vec::<u8>::new()));
        let notes_on_c = notes_on.clone();
        let notes_off_c = notes_off.clone();

        kb.set_on_note_on(move |n, _v| {
            notes_on_c.lock().unwrap().push(n);
        });
        kb.set_on_note_off(move |n| {
            notes_off_c.lock().unwrap().push(n);
        });

        kb.set_bounds(Bounds::new(0, 0, 240, 100)).unwrap();

        // Press on first key
        kb.mouse_down(5, 50);
        // Drag to second key area (24px wide white keys)
        kb.mouse_drag(30, 50);

        // Should have released first and pressed second
        assert_eq!(notes_off.lock().unwrap().len(), 1);
        assert_eq!(notes_on.lock().unwrap().len(), 2);

        kb.mouse_up();
    }

    #[test]
    fn midi_keyboard_mouse_velocity() {
        let mut kb = MidiKeyboardComponent::new();
        kb.set_bounds(Bounds::new(0, 0, 240, 100)).unwrap();

        // Top of key = high velocity
        let vel_top = kb.velocity_at_position(5, 0);
        assert!(vel_top > 0.9);

        // Bottom of key = low velocity
        let vel_bottom = kb.velocity_at_position(5, 99);
        assert!(vel_bottom < 0.2);
        assert!(vel_bottom >= 0.1); // Minimum velocity
    }

    #[test]
    fn midi_keyboard_orientation() {
        let mut kb = MidiKeyboardComponent::new();
        kb.set_orientation(KeyboardOrientation::Vertical);
        assert_eq!(kb.orientation(), KeyboardOrientation::Vertical);
    }

    #[test]
    fn midi_keyboard_dimensions() {
        let mut kb = MidiKeyboardComponent::new();
        kb.set_white_key_width(30.0);
        assert_eq!(kb.white_key_width(), 30.0);
        kb.set_white_key_width(1.0); // clamped to min 4
        assert_eq!(kb.white_key_width(), 4.0);

        kb.set_white_key_height(150.0);
        assert_eq!(kb.white_key_height(), 150.0);

        kb.set_black_key_width_ratio(0.5);
        assert_eq!(kb.black_key_width_ratio(), 0.5);
        kb.set_black_key_width_ratio(0.1); // clamped to 0.2
        assert_eq!(kb.black_key_width_ratio(), 0.2);

        kb.set_black_key_height_ratio(0.7);
        assert_eq!(kb.black_key_height_ratio(), 0.7);
    }

    #[test]
    fn midi_keyboard_colours() {
        let mut kb = MidiKeyboardComponent::new();
        kb.set_white_key_color(Color::rgb(200, 200, 200));
        kb.set_black_key_color(Color::rgb(30, 30, 30));
        kb.set_active_key_color(Color::rgb(100, 150, 255));
        kb.set_text_color(Color::rgb(50, 50, 50));
        kb.set_background_color(Color::rgb(230, 230, 230));
        // Just verify no panic; colour comparison is indirect
    }

    #[test]
    fn midi_keyboard_component_delegation() {
        use crate::components::Bounds;

        let mut kb = MidiKeyboardComponent::new();
        kb.set_bounds(Bounds::new(10, 20, 300, 100)).unwrap();
        assert_eq!(kb.bounds().x, 10);
        assert_eq!(kb.bounds().y, 20);
        assert_eq!(kb.bounds().width, 300);
        assert_eq!(kb.bounds().height, 100);

        assert!(kb.is_enabled());
        kb.set_enabled(false);
        assert!(!kb.is_enabled());

        assert!(kb.is_visible());
        kb.set_visible(false);
        assert!(!kb.is_visible());
    }

    #[test]
    fn midi_keyboard_note_names_toggle() {
        let mut kb = MidiKeyboardComponent::new();
        assert!(kb.shows_note_names());
        kb.set_show_note_names(false);
        assert!(!kb.shows_note_names());
    }

    #[test]
    fn midi_keyboard_middle_c() {
        let mut kb = MidiKeyboardComponent::new();
        assert_eq!(kb.middle_c_number(), 60);
        kb.set_middle_c_number(48);
        assert_eq!(kb.middle_c_number(), 48);
    }
}
