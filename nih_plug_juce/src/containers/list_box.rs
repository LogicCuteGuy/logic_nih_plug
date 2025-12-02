//! JUCE ListBox component.
//!
//! This module provides a safe Rust wrapper around JUCE's ListBox class,
//! which displays a scrollable list of items with custom rendering.
//!
//! # Thread Safety
//!
//! All ListBox operations must be performed on the JUCE message thread.
//! This is enforced through the type system - ListBox does not implement
//! `Send` or `Sync`.
//!
//! # Examples
//!
//! ```ignore
//! use nih_plug_juce::containers::{ListBox, ListBoxModel};
//! use nih_plug_juce::Graphics;
//!
//! struct MyListModel {
//!     items: Vec<String>,
//! }
//!
//! impl ListBoxModel for MyListModel {
//!     fn get_num_rows(&self) -> i32 {
//!         self.items.len() as i32
//!     }
//!
//!     fn paint_list_box_item(&self, row: i32, g: &mut Graphics, width: i32, height: i32, selected: bool) {
//!         if selected {
//!             g.fill_rect(0, 0, width, height);
//!         }
//!         // Draw item text...
//!     }
//!
//!     fn selected_rows_changed(&mut self, last_row_selected: i32) {
//!         println!("Selected row: {}", last_row_selected);
//!     }
//! }
//!
//! let mut list_box = ListBox::new()?;
//! let model = Box::new(MyListModel { items: vec!["Item 1".to_string(), "Item 2".to_string()] });
//! list_box.set_model(model)?;
//! ```

use crate::bridge::ffi;
use crate::component::Component;
use crate::error::{JuceError, Result};
use crate::graphics::Graphics;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// Trait for implementing custom list box models.
///
/// Implement this trait to provide custom rendering and behavior for list items.
/// The ListBox will call these methods to determine how many rows to display,
/// how to render each row, and when the selection changes.
///
/// # Thread Safety
///
/// All methods will be called on the JUCE message thread.
///
/// # Examples
///
/// ```ignore
/// struct MyListModel {
///     items: Vec<String>,
/// }
///
/// impl ListBoxModel for MyListModel {
///     fn get_num_rows(&self) -> i32 {
///         self.items.len() as i32
///     }
///
///     fn paint_list_box_item(&self, row: i32, g: &mut Graphics, width: i32, height: i32, selected: bool) {
///         // Custom rendering logic
///     }
///
///     fn selected_rows_changed(&mut self, last_row_selected: i32) {
///         // Handle selection change
///     }
/// }
/// ```
pub trait ListBoxModel {
    /// Get the number of rows in the list.
    ///
    /// This method is called by the ListBox to determine how many rows to display.
    ///
    /// # Returns
    ///
    /// The number of rows in the list.
    fn get_num_rows(&self) -> i32;

    /// Paint a list box item.
    ///
    /// This method is called by the ListBox to render each visible row.
    ///
    /// # Arguments
    ///
    /// * `row` - The row index to paint (0-based)
    /// * `g` - The Graphics context to draw with
    /// * `width` - The width of the row area
    /// * `height` - The height of the row area
    /// * `selected` - Whether this row is currently selected
    fn paint_list_box_item(&self, row: i32, g: &mut Graphics, width: i32, height: i32, selected: bool);

    /// Called when the selected rows change.
    ///
    /// This method is called by the ListBox when the user changes the selection.
    ///
    /// # Arguments
    ///
    /// * `last_row_selected` - The index of the last row that was selected
    fn selected_rows_changed(&mut self, last_row_selected: i32);
}

/// A JUCE ListBox - a scrollable list with custom rendering.
///
/// ListBox provides a scrollable list where each item can be rendered
/// using custom drawing code. The list behavior is defined by implementing
/// the ListBoxModel trait.
///
/// # Inheritance
///
/// ListBox inherits from Component through Deref/DerefMut, so all
/// Component methods are available on ListBox instances.
///
/// # Thread Safety
///
/// ListBox does not implement `Send` or `Sync`, enforcing that all
/// list box operations occur on the message thread.
///
/// # Examples
///
/// ```ignore
/// use nih_plug_juce::containers::{ListBox, ListBoxModel};
///
/// // Create a list box
/// let mut list_box = ListBox::new()?;
///
/// // Create and set a model
/// let model = Box::new(MyListModel::new());
/// list_box.set_model(model)?;
///
/// // Update the content when the model changes
/// list_box.update_content();
/// ```
pub struct ListBox {
    /// The underlying Component.
    /// This is wrapped to provide the inheritance pattern through Deref/DerefMut.
    component: Component,

    /// PhantomData to make ListBox !Send + !Sync.
    /// This enforces that ListBox can only be used on the thread where
    /// it was created (the message thread), matching JUCE's requirements.
    _phantom: PhantomData<*mut ()>,
}

impl ListBox {
    /// Create a new ListBox.
    ///
    /// This allocates a new juce::ListBox in C++ and returns a safe
    /// Rust wrapper around it.
    ///
    /// # Returns
    ///
    /// Returns `Ok(ListBox)` on success, or an error if list box
    /// creation failed.
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use nih_plug_juce::containers::ListBox;
    ///
    /// let list_box = ListBox::new()?;
    /// ```
    pub fn new() -> Result<Self> {
        let mut error_buf = vec![0u8; 256];

        let ptr = unsafe {
            ffi::create_list_box(error_buf.as_mut_ptr() as *mut i8, error_buf.len())
        };

        if ptr.is_null() {
            let error_msg = String::from_utf8_lossy(&error_buf)
                .trim_end_matches('\0')
                .to_string();

            if error_msg.is_empty() {
                Err(JuceError::ComponentCreationFailed(
                    "Unknown error creating ListBox".to_string(),
                ))
            } else {
                Err(JuceError::ComponentCreationFailed(error_msg))
            }
        } else {
            // Wrap the raw pointer in a Component
            // Safety: We know this is a valid JuceComponent pointer from create_list_box
            let component = unsafe { Component::from_raw(ptr) };

            Ok(ListBox {
                component,
                _phantom: PhantomData,
            })
        }
    }

    /// Set the model for this list box.
    ///
    /// The list box takes ownership of the model and will call its methods
    /// to determine the list content and rendering. The model will be kept
    /// alive for the lifetime of the list box.
    ///
    /// # Arguments
    ///
    /// * `model` - The list box model to use
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if setting the model failed.
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut list_box = ListBox::new()?;
    /// let model = Box::new(MyListModel::new());
    /// list_box.set_model(model)?;
    /// ```
    pub fn set_model(&mut self, model: Box<dyn ListBoxModel>) -> Result<()> {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return Err(JuceError::NullPointer("ListBox pointer is null".to_string()));
        }

        // Box the model and convert to raw pointer
        let boxed = Box::new(model);
        let raw = Box::into_raw(boxed);

        // Define the trampoline functions that will be called from C++
        unsafe extern "C" fn get_num_rows_trampoline(model_ptr: usize) -> i32 {
            // Reconstruct the model reference (without taking ownership)
            let model = &*(model_ptr as *const Box<dyn ListBoxModel>);
            model.get_num_rows()
        }

        unsafe extern "C" fn paint_list_box_item_trampoline(
            model_ptr: usize,
            row: i32,
            graphics_ptr: usize,
            width: i32,
            height: i32,
            selected: bool,
        ) {
            // Reconstruct the model reference (without taking ownership)
            let model = &*(model_ptr as *const Box<dyn ListBoxModel>);

            // Create a Graphics wrapper from the raw pointer
            // Safety: The graphics pointer is valid for the duration of this call
            let mut graphics = Graphics::from_raw(graphics_ptr as *mut crate::bridge::ffi::JuceGraphics);

            model.paint_list_box_item(row, &mut graphics, width, height, selected);

            // Prevent Graphics from being dropped since JUCE owns it
            std::mem::forget(graphics);
        }

        unsafe extern "C" fn selected_rows_changed_trampoline(
            model_ptr: usize,
            last_row_selected: i32,
        ) {
            // Reconstruct the model reference (without taking ownership)
            let model = &mut *(model_ptr as *mut Box<dyn ListBoxModel>);
            model.selected_rows_changed(last_row_selected);
        }

        unsafe extern "C" fn drop_model_trampoline(model_ptr: usize) {
            // Take ownership and drop the model
            let _ = Box::from_raw(model_ptr as *mut Box<dyn ListBoxModel>);
        }

        let mut error_buf = vec![0u8; 256];

        let result = unsafe {
            ffi::list_box_set_model(
                ptr,
                raw as usize,
                get_num_rows_trampoline as usize,
                paint_list_box_item_trampoline as usize,
                selected_rows_changed_trampoline as usize,
                drop_model_trampoline as usize,
                error_buf.as_mut_ptr() as *mut i8,
                error_buf.len(),
            )
        };

        if result == 0 {
            Ok(())
        } else {
            // If setting the model failed, we need to clean up the boxed model
            unsafe {
                let _ = Box::from_raw(raw);
            }

            let error_msg = String::from_utf8_lossy(&error_buf)
                .trim_end_matches('\0')
                .to_string();

            if error_msg.is_empty() {
                Err(JuceError::CallbackError(
                    "Unknown error setting list box model".to_string(),
                ))
            } else {
                Err(JuceError::CallbackError(error_msg))
            }
        }
    }

    /// Update the content of the list box.
    ///
    /// Call this method after the model's data has changed to refresh
    /// the list display. This will cause the list box to re-query the
    /// model for the number of rows and repaint all visible items.
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // After modifying the model's data
    /// list_box.update_content();
    /// ```
    pub fn update_content(&mut self) {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return;
        }

        unsafe {
            ffi::list_box_update_content(ptr);
        }
    }
}

impl Deref for ListBox {
    type Target = Component;

    fn deref(&self) -> &Self::Target {
        &self.component
    }
}

impl DerefMut for ListBox {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.component
    }
}

// Ensure ListBox is !Send + !Sync (enforced by PhantomData<*mut ()>)
// This prevents list boxes from being moved or shared across threads,
// which is required by JUCE's threading model.
