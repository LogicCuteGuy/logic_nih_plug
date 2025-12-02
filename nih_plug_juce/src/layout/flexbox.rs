//! FlexBox layout system for arranging components.
//!
//! This module provides a Rust wrapper around JUCE's FlexBox layout system,
//! which allows flexible and responsive arrangement of components using
//! CSS Flexbox-like semantics.
//!
//! # Thread Safety
//!
//! All FlexBox operations must be performed on the JUCE message thread.
//! This is enforced through the type system - FlexBox does not implement
//! `Send` or `Sync`.
//!
//! # Examples
//!
//! ```ignore
//! use nih_plug_juce::layout::{FlexBox, FlexItem, FlexDirection, FlexWrap};
//! use nih_plug_juce::Component;
//!
//! // Create a flexbox layout
//! let mut flexbox = FlexBox::new()?;
//! flexbox.set_direction(FlexDirection::Row);
//! flexbox.set_wrap(FlexWrap::Wrap);
//!
//! // Create components
//! let component1 = Component::new()?;
//! let component2 = Component::new()?;
//! let component3 = Component::new()?;
//!
//! // Create flex items with different properties
//! let item1 = FlexItem::new(&component1)
//!     .with_flex_grow(1.0)
//!     .with_min_width(100.0)
//!     .with_margin(5.0, 5.0, 5.0, 5.0);
//!
//! let item2 = FlexItem::new(&component2)
//!     .with_flex_grow(2.0)
//!     .with_min_width(150.0)
//!     .with_margin(5.0, 5.0, 5.0, 5.0);
//!
//! let item3 = FlexItem::new(&component3)
//!     .with_flex_grow(1.0)
//!     .with_min_width(100.0)
//!     .with_margin(5.0, 5.0, 5.0, 5.0);
//!
//! // Add items to flexbox
//! flexbox.add_item(item1);
//! flexbox.add_item(item2);
//! flexbox.add_item(item3);
//!
//! // Perform layout within bounds (x, y, width, height)
//! flexbox.perform_layout(0, 0, 800, 600);
//! ```

use crate::bridge::ffi;
use crate::component::Component;
use crate::error::{JuceError, Result};
use std::marker::PhantomData;
use std::ptr;

/// Direction for flex layout.
///
/// This determines the main axis along which flex items are laid out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexDirection {
    /// Items are laid out horizontally from left to right.
    Row,
    /// Items are laid out vertically from top to bottom.
    Column,
    /// Items are laid out horizontally from right to left.
    RowReverse,
    /// Items are laid out vertically from bottom to top.
    ColumnReverse,
}

impl FlexDirection {
    /// Convert to the C++ enum value.
    fn to_cpp_value(self) -> i32 {
        match self {
            FlexDirection::Row => 0,
            FlexDirection::Column => 1,
            FlexDirection::RowReverse => 2,
            FlexDirection::ColumnReverse => 3,
        }
    }
}

/// Wrapping behavior for flex items.
///
/// This determines whether items wrap to a new line when they don't fit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexWrap {
    /// Items do not wrap; they stay on a single line.
    NoWrap,
    /// Items wrap to new lines as needed.
    Wrap,
    /// Items wrap to new lines in reverse order.
    WrapReverse,
}

impl FlexWrap {
    /// Convert to the C++ enum value.
    fn to_cpp_value(self) -> i32 {
        match self {
            FlexWrap::NoWrap => 0,
            FlexWrap::Wrap => 1,
            FlexWrap::WrapReverse => 2,
        }
    }
}

/// Justification of items along the main axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JustifyContent {
    /// Items are packed toward the start of the flex direction.
    FlexStart,
    /// Items are packed toward the end of the flex direction.
    FlexEnd,
    /// Items are centered along the main axis.
    Center,
    /// Items are evenly distributed with space between them.
    SpaceBetween,
    /// Items are evenly distributed with space around them.
    SpaceAround,
}

impl JustifyContent {
    /// Convert to the C++ enum value.
    fn to_cpp_value(self) -> i32 {
        match self {
            JustifyContent::FlexStart => 0,
            JustifyContent::FlexEnd => 1,
            JustifyContent::Center => 2,
            JustifyContent::SpaceBetween => 3,
            JustifyContent::SpaceAround => 4,
        }
    }
}

/// Alignment of items along the cross axis for multi-line layouts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignContent {
    /// Lines are packed toward the start of the cross axis.
    FlexStart,
    /// Lines are packed toward the end of the cross axis.
    FlexEnd,
    /// Lines are centered along the cross axis.
    Center,
    /// Lines are evenly distributed with space between them.
    SpaceBetween,
    /// Lines are evenly distributed with space around them.
    SpaceAround,
    /// Lines stretch to fill the container.
    Stretch,
}

impl AlignContent {
    /// Convert to the C++ enum value.
    fn to_cpp_value(self) -> i32 {
        match self {
            AlignContent::FlexStart => 0,
            AlignContent::FlexEnd => 1,
            AlignContent::Center => 2,
            AlignContent::SpaceBetween => 3,
            AlignContent::SpaceAround => 4,
            AlignContent::Stretch => 5,
        }
    }
}

/// Alignment of items along the cross axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignItems {
    /// Items are aligned to the start of the cross axis.
    FlexStart,
    /// Items are aligned to the end of the cross axis.
    FlexEnd,
    /// Items are centered along the cross axis.
    Center,
    /// Items are stretched to fill the cross axis.
    Stretch,
}

impl AlignItems {
    /// Convert to the C++ enum value.
    fn to_cpp_value(self) -> i32 {
        match self {
            AlignItems::FlexStart => 0,
            AlignItems::FlexEnd => 1,
            AlignItems::Center => 2,
            AlignItems::Stretch => 3,
        }
    }
}

/// A flex item that can be added to a FlexBox.
///
/// FlexItem wraps a Component and specifies how it should be sized
/// and positioned within the flex layout.
#[derive(Debug)]
pub struct FlexItem {
    /// The component to be laid out.
    component_ptr: *mut ffi::JuceComponent,
    /// How much the item should grow relative to other items.
    pub flex_grow: f32,
    /// How much the item should shrink relative to other items.
    pub flex_shrink: f32,
    /// The initial size of the item before growing/shrinking.
    pub flex_basis: f32,
    /// Minimum width of the item.
    pub min_width: f32,
    /// Minimum height of the item.
    pub min_height: f32,
    /// Maximum width of the item.
    pub max_width: f32,
    /// Maximum height of the item.
    pub max_height: f32,
    /// Margin around the item (top, right, bottom, left).
    pub margin: (f32, f32, f32, f32),
}

impl FlexItem {
    /// Create a new FlexItem from a Component.
    ///
    /// # Arguments
    ///
    /// * `component` - The component to wrap in this flex item
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use nih_plug_juce::layout::FlexItem;
    /// use nih_plug_juce::Component;
    ///
    /// let component = Component::new()?;
    /// let item = FlexItem::new(component);
    /// ```
    pub fn new(component: &Component) -> Self {
        FlexItem {
            component_ptr: component.as_ptr(),
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: 0.0,
            min_width: 0.0,
            min_height: 0.0,
            max_width: f32::MAX,
            max_height: f32::MAX,
            margin: (0.0, 0.0, 0.0, 0.0),
        }
    }
    
    /// Set the flex grow factor.
    ///
    /// This determines how much the item will grow relative to other items
    /// when there is extra space available.
    ///
    /// # Arguments
    ///
    /// * `grow` - The grow factor (0.0 = don't grow, 1.0 = normal growth)
    pub fn with_flex_grow(mut self, grow: f32) -> Self {
        self.flex_grow = grow;
        self
    }
    
    /// Set the flex shrink factor.
    ///
    /// This determines how much the item will shrink relative to other items
    /// when there is not enough space available.
    ///
    /// # Arguments
    ///
    /// * `shrink` - The shrink factor (0.0 = don't shrink, 1.0 = normal shrinking)
    pub fn with_flex_shrink(mut self, shrink: f32) -> Self {
        self.flex_shrink = shrink;
        self
    }
    
    /// Set the flex basis.
    ///
    /// This is the initial size of the item before growing or shrinking.
    ///
    /// # Arguments
    ///
    /// * `basis` - The initial size in pixels
    pub fn with_flex_basis(mut self, basis: f32) -> Self {
        self.flex_basis = basis;
        self
    }
    
    /// Set the minimum width.
    ///
    /// # Arguments
    ///
    /// * `width` - The minimum width in pixels
    pub fn with_min_width(mut self, width: f32) -> Self {
        self.min_width = width;
        self
    }
    
    /// Set the minimum height.
    ///
    /// # Arguments
    ///
    /// * `height` - The minimum height in pixels
    pub fn with_min_height(mut self, height: f32) -> Self {
        self.min_height = height;
        self
    }
    
    /// Set the maximum width.
    ///
    /// # Arguments
    ///
    /// * `width` - The maximum width in pixels
    pub fn with_max_width(mut self, width: f32) -> Self {
        self.max_width = width;
        self
    }
    
    /// Set the maximum height.
    ///
    /// # Arguments
    ///
    /// * `height` - The maximum height in pixels
    pub fn with_max_height(mut self, height: f32) -> Self {
        self.max_height = height;
        self
    }
    
    /// Set the margin around the item.
    ///
    /// # Arguments
    ///
    /// * `top` - Top margin in pixels
    /// * `right` - Right margin in pixels
    /// * `bottom` - Bottom margin in pixels
    /// * `left` - Left margin in pixels
    pub fn with_margin(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        self.margin = (top, right, bottom, left);
        self
    }
}

/// A FlexBox layout container.
///
/// FlexBox provides a flexible way to arrange components using CSS Flexbox-like
/// semantics. Items can be laid out in rows or columns, with control over
/// wrapping, justification, and alignment.
///
/// # Thread Safety
///
/// FlexBox does not implement `Send` or `Sync`, enforcing that all layout
/// operations occur on the message thread.
///
/// # Examples
///
/// ```ignore
/// use nih_plug_juce::layout::{FlexBox, FlexItem, FlexDirection};
/// use nih_plug_juce::Component;
///
/// let mut flexbox = FlexBox::new();
/// flexbox.set_direction(FlexDirection::Row);
/// flexbox.set_wrap(FlexWrap::Wrap);
///
/// let component1 = Component::new()?;
/// let item1 = FlexItem::new(&component1).with_flex_grow(1.0);
/// flexbox.add_item(item1);
///
/// let component2 = Component::new()?;
/// let item2 = FlexItem::new(&component2).with_flex_grow(2.0);
/// flexbox.add_item(item2);
///
/// // Perform layout within bounds
/// flexbox.perform_layout(0, 0, 800, 600);
/// ```
pub struct FlexBox {
    /// Opaque pointer to the C++ juce::FlexBox object.
    ptr: *mut ffi::JuceFlexBox,
    
    /// PhantomData to make FlexBox !Send + !Sync.
    _phantom: PhantomData<*mut ()>,
}

impl FlexBox {
    /// Create a new FlexBox.
    ///
    /// # Returns
    ///
    /// Returns a new FlexBox with default settings.
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use nih_plug_juce::layout::FlexBox;
    ///
    /// let flexbox = FlexBox::new();
    /// ```
    pub fn new() -> Result<Self> {
        let mut error_buf = vec![0u8; 256];
        
        let ptr = unsafe {
            ffi::create_flexbox(
                error_buf.as_mut_ptr() as *mut i8,
                error_buf.len()
            )
        };
        
        if ptr.is_null() {
            let error_msg = String::from_utf8_lossy(&error_buf)
                .trim_end_matches('\0')
                .to_string();
            
            if error_msg.is_empty() {
                Err(JuceError::FfiError("Unknown error creating FlexBox".to_string()))
            } else {
                Err(JuceError::FfiError(error_msg))
            }
        } else {
            Ok(FlexBox {
                ptr,
                _phantom: PhantomData,
            })
        }
    }
    
    /// Set the flex direction.
    ///
    /// This determines the main axis along which items are laid out.
    ///
    /// # Arguments
    ///
    /// * `direction` - The flex direction
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use nih_plug_juce::layout::{FlexBox, FlexDirection};
    ///
    /// let mut flexbox = FlexBox::new();
    /// flexbox.set_direction(FlexDirection::Row);
    /// ```
    pub fn set_direction(&mut self, direction: FlexDirection) {
        if self.ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::flexbox_set_direction(self.ptr, direction.to_cpp_value());
        }
    }
    
    /// Set the flex wrap behavior.
    ///
    /// This determines whether items wrap to new lines when they don't fit.
    ///
    /// # Arguments
    ///
    /// * `wrap` - The wrap behavior
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use nih_plug_juce::layout::{FlexBox, FlexWrap};
    ///
    /// let mut flexbox = FlexBox::new();
    /// flexbox.set_wrap(FlexWrap::Wrap);
    /// ```
    pub fn set_wrap(&mut self, wrap: FlexWrap) {
        if self.ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::flexbox_set_wrap(self.ptr, wrap.to_cpp_value());
        }
    }
    
    /// Set the justify content property.
    ///
    /// This determines how items are distributed along the main axis.
    ///
    /// # Arguments
    ///
    /// * `justify` - The justification mode
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    pub fn set_justify_content(&mut self, justify: JustifyContent) {
        if self.ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::flexbox_set_justify_content(self.ptr, justify.to_cpp_value());
        }
    }
    
    /// Set the align content property.
    ///
    /// This determines how lines are distributed along the cross axis
    /// in multi-line layouts.
    ///
    /// # Arguments
    ///
    /// * `align` - The alignment mode
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    pub fn set_align_content(&mut self, align: AlignContent) {
        if self.ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::flexbox_set_align_content(self.ptr, align.to_cpp_value());
        }
    }
    
    /// Set the align items property.
    ///
    /// This determines how items are aligned along the cross axis.
    ///
    /// # Arguments
    ///
    /// * `align` - The alignment mode
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    pub fn set_align_items(&mut self, align: AlignItems) {
        if self.ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::flexbox_set_align_items(self.ptr, align.to_cpp_value());
        }
    }
    
    /// Add an item to the flex container.
    ///
    /// # Arguments
    ///
    /// * `item` - The flex item to add
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use nih_plug_juce::layout::{FlexBox, FlexItem};
    /// use nih_plug_juce::Component;
    ///
    /// let mut flexbox = FlexBox::new();
    /// let component = Component::new()?;
    /// let item = FlexItem::new(&component).with_flex_grow(1.0);
    /// flexbox.add_item(item);
    /// ```
    pub fn add_item(&mut self, item: FlexItem) {
        if self.ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::flexbox_add_item(
                self.ptr,
                item.component_ptr,
                item.flex_grow,
                item.flex_shrink,
                item.flex_basis,
                item.min_width,
                item.min_height,
                item.max_width,
                item.max_height,
                item.margin.0,
                item.margin.1,
                item.margin.2,
                item.margin.3,
            );
        }
    }
    
    /// Perform the flex layout within the specified bounds.
    ///
    /// This calculates the positions and sizes of all items and applies
    /// them to the components.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate of the layout area
    /// * `y` - Y coordinate of the layout area
    /// * `width` - Width of the layout area
    /// * `height` - Height of the layout area
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use nih_plug_juce::layout::FlexBox;
    ///
    /// let mut flexbox = FlexBox::new();
    /// // ... add items ...
    /// flexbox.perform_layout(0, 0, 800, 600);
    /// ```
    pub fn perform_layout(&mut self, x: i32, y: i32, width: i32, height: i32) {
        if self.ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::flexbox_perform_layout(self.ptr, x, y, width, height);
        }
    }
}

impl Default for FlexBox {
    fn default() -> Self {
        Self::new().expect("Failed to create default FlexBox")
    }
}

impl Drop for FlexBox {
    /// Automatically clean up the C++ FlexBox when the Rust wrapper is dropped.
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                ffi::delete_flexbox(self.ptr);
            }
            self.ptr = ptr::null_mut();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_flexbox_creation() {
        let result = FlexBox::new();
        assert!(result.is_ok(), "FlexBox creation should succeed");
    }
    
    #[test]
    fn test_flexbox_direction() {
        let mut flexbox = FlexBox::new().unwrap();
        flexbox.set_direction(FlexDirection::Row);
        flexbox.set_direction(FlexDirection::Column);
        flexbox.set_direction(FlexDirection::RowReverse);
        flexbox.set_direction(FlexDirection::ColumnReverse);
    }
    
    #[test]
    fn test_flexbox_wrap() {
        let mut flexbox = FlexBox::new().unwrap();
        flexbox.set_wrap(FlexWrap::NoWrap);
        flexbox.set_wrap(FlexWrap::Wrap);
        flexbox.set_wrap(FlexWrap::WrapReverse);
    }
    
    #[test]
    fn test_flex_item_builder() {
        let component = Component::new().unwrap();
        let item = FlexItem::new(&component)
            .with_flex_grow(1.0)
            .with_flex_shrink(0.5)
            .with_flex_basis(100.0)
            .with_min_width(50.0)
            .with_min_height(50.0)
            .with_max_width(200.0)
            .with_max_height(200.0)
            .with_margin(5.0, 10.0, 5.0, 10.0);
        
        assert_eq!(item.flex_grow, 1.0);
        assert_eq!(item.flex_shrink, 0.5);
        assert_eq!(item.flex_basis, 100.0);
        assert_eq!(item.min_width, 50.0);
        assert_eq!(item.min_height, 50.0);
        assert_eq!(item.max_width, 200.0);
        assert_eq!(item.max_height, 200.0);
        assert_eq!(item.margin, (5.0, 10.0, 5.0, 10.0));
    }
}
