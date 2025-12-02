//! JUCE ComboBox components.
//!
//! This module provides safe Rust wrappers around JUCE's ComboBox component,
//! which is used for dropdown selection menus.
//!
//! # Thread Safety
//!
//! All combo box operations must be performed on the JUCE message thread.
//! This is enforced through the type system - combo boxes do not implement
//! `Send` or `Sync`.
//!
//! # Examples
//!
//! ```ignore
//! use nih_plug_juce::widgets::ComboBox;
//!
//! let mut combo = ComboBox::new()?;
//! combo.set_bounds(10, 10, 150, 30);
//! combo.add_item("Option 1", 1);
//! combo.add_item("Option 2", 2);
//! combo.set_selected_id(1);
//! combo.set_on_change(|id| {
//!     println!("Selected item ID: {}", id);
//! })?;
//! ```

use crate::bridge::ffi;
use crate::component::Component;
use crate::error::{JuceError, Result};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// A JUCE ComboBox - a dropdown selection component.
///
/// ComboBox provides a dropdown menu for selecting from a list of items.
/// Each item has a text label and an integer ID. The component can trigger
/// callbacks when the selection changes.
///
/// # Inheritance
///
/// ComboBox inherits from Component through Deref/DerefMut, so all
/// Component methods are available on ComboBox instances.
///
/// # Thread Safety
///
/// ComboBox does not implement `Send` or `Sync`, enforcing that all
/// combo box operations occur on the message thread.
///
/// # Examples
///
/// ```ignore
/// use nih_plug_juce::widgets::ComboBox;
///
/// // Create a combo box
/// let mut combo = ComboBox::new()?;
///
/// // Set its position and size (inherited from Component)
/// combo.set_bounds(10, 10, 150, 30);
///
/// // Add items
/// combo.add_item("Red", 1);
/// combo.add_item("Green", 2);
/// combo.add_item("Blue", 3);
///
/// // Set initial selection
/// combo.set_selected_id(1);
///
/// // Set a change callback
/// combo.set_on_change(|id| {
///     println!("Selected item ID: {}", id);
/// })?;
/// ```
pub struct ComboBox {
    /// The underlying Component.
    /// This is wrapped to provide the inheritance pattern through Deref/DerefMut.
    component: Component,
    
    /// PhantomData to make ComboBox !Send + !Sync.
    /// This enforces that ComboBox can only be used on the thread where
    /// it was created (the message thread), matching JUCE's requirements.
    _phantom: PhantomData<*mut ()>,
}

impl ComboBox {
    /// Create a new ComboBox.
    ///
    /// This allocates a new juce::ComboBox in C++ and returns a safe
    /// Rust wrapper around it.
    ///
    /// # Returns
    ///
    /// Returns `Ok(ComboBox)` on success, or an error if combo box
    /// creation failed.
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use nih_plug_juce::widgets::ComboBox;
    ///
    /// let combo = ComboBox::new()?;
    /// ```
    pub fn new() -> Result<Self> {
        let mut error_buf = vec![0u8; 256];
        
        let ptr = unsafe {
            ffi::create_combo_box(
                error_buf.as_mut_ptr() as *mut i8,
                error_buf.len()
            )
        };
        
        if ptr.is_null() {
            let error_msg = String::from_utf8_lossy(&error_buf)
                .trim_end_matches('\0')
                .to_string();
            
            if error_msg.is_empty() {
                Err(JuceError::ComponentCreationFailed(
                    "Unknown error creating ComboBox".to_string()
                ))
            } else {
                Err(JuceError::ComponentCreationFailed(error_msg))
            }
        } else {
            // Wrap the raw pointer in a Component
            // Safety: We know this is a valid JuceComponent pointer from create_combo_box
            let component = unsafe { Component::from_raw(ptr) };
            
            Ok(ComboBox {
                component,
                _phantom: PhantomData,
            })
        }
    }
    
    /// Add an item to the combo box.
    ///
    /// Items are displayed in the order they are added. Each item has
    /// a text label and an integer ID that can be used to identify it.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to display for this item
    /// * `item_id` - A unique integer ID for this item (must be > 0)
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut combo = ComboBox::new()?;
    /// combo.add_item("First", 1);
    /// combo.add_item("Second", 2);
    /// ```
    pub fn add_item(&mut self, text: &str, item_id: i32) {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::combo_box_add_item(ptr, text.as_ptr(), text.len(), item_id);
        }
    }
    
    /// Clear all items from the combo box.
    ///
    /// This removes all items, leaving the combo box empty.
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut combo = ComboBox::new()?;
    /// combo.add_item("Item 1", 1);
    /// combo.add_item("Item 2", 2);
    /// combo.clear(); // Now empty
    /// ```
    pub fn clear(&mut self) {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::combo_box_clear(ptr);
        }
    }
    
    /// Set the selected item by ID.
    ///
    /// This selects the item with the specified ID. If no item with
    /// that ID exists, the selection is cleared. This will not trigger
    /// the change callback.
    ///
    /// # Arguments
    ///
    /// * `item_id` - The ID of the item to select
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut combo = ComboBox::new()?;
    /// combo.add_item("Red", 1);
    /// combo.add_item("Green", 2);
    /// combo.set_selected_id(2); // Select "Green"
    /// ```
    pub fn set_selected_id(&mut self, item_id: i32) {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::combo_box_set_selected_id(ptr, item_id);
        }
    }
    
    /// Set the selected item by index.
    ///
    /// This selects the item at the specified index (0-based). If the
    /// index is out of range, the selection is cleared. This will not
    /// trigger the change callback.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the item to select (0-based)
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut combo = ComboBox::new()?;
    /// combo.add_item("First", 1);
    /// combo.add_item("Second", 2);
    /// combo.set_selected_index(1); // Select "Second"
    /// ```
    pub fn set_selected_index(&mut self, index: i32) {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::combo_box_set_selected_index(ptr, index);
        }
    }
    
    /// Get the ID of the currently selected item.
    ///
    /// # Returns
    ///
    /// Returns the ID of the selected item, or 0 if no item is selected.
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let combo = ComboBox::new()?;
    /// let selected_id = combo.get_selected_id();
    /// println!("Selected ID: {}", selected_id);
    /// ```
    pub fn get_selected_id(&self) -> i32 {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return 0;
        }
        
        unsafe {
            ffi::combo_box_get_selected_id(ptr)
        }
    }
    
    /// Set a callback to be invoked when the selection changes.
    ///
    /// The callback will be invoked on the message thread whenever the
    /// selected item changes. The callback receives the ID of the newly
    /// selected item as a parameter.
    ///
    /// # Arguments
    ///
    /// * `callback` - A closure to invoke when the selection changes
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if setting the callback failed.
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    /// The callback will also be invoked on the message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut combo = ComboBox::new()?;
    /// combo.add_item("Option 1", 1);
    /// combo.add_item("Option 2", 2);
    /// combo.set_on_change(|id| {
    ///     println!("Selection changed to ID: {}", id);
    /// })?;
    /// ```
    pub fn set_on_change<F>(&mut self, callback: F) -> Result<()>
    where
        F: Fn(i32) + 'static,
    {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return Err(JuceError::NullPointer("ComboBox pointer is null".to_string()));
        }
        
        // Box the closure and convert to raw pointer
        let boxed = Box::new(callback);
        let raw = Box::into_raw(boxed);
        
        // Define the trampoline function that will be called from C++
        unsafe extern "C" fn trampoline<F>(closure_ptr: usize, item_id: i32)
        where
            F: Fn(i32),
        {
            // Reconstruct the closure reference (without taking ownership)
            let closure = &*(closure_ptr as *const F);
            
            // Invoke the Rust closure with the item ID
            closure(item_id);
        }
        
        // Define the drop function that will be called when the combo box is destroyed
        unsafe extern "C" fn drop_closure<F>(closure_ptr: usize)
        where
            F: Fn(i32),
        {
            // Take ownership and drop the closure
            let _ = Box::from_raw(closure_ptr as *mut F);
        }
        
        let mut error_buf = vec![0u8; 256];
        
        let result = unsafe {
            ffi::combo_box_set_on_change(
                ptr,
                raw as usize,
                trampoline::<F> as usize,
                drop_closure::<F> as usize,
                error_buf.as_mut_ptr() as *mut i8,
                error_buf.len(),
            )
        };
        
        if result == 0 {
            Ok(())
        } else {
            // If setting the callback failed, we need to clean up the boxed closure
            unsafe {
                let _ = Box::from_raw(raw);
            }
            
            let error_msg = String::from_utf8_lossy(&error_buf)
                .trim_end_matches('\0')
                .to_string();
            
            if error_msg.is_empty() {
                Err(JuceError::CallbackError(
                    "Unknown error setting combo box change callback".to_string()
                ))
            } else {
                Err(JuceError::CallbackError(error_msg))
            }
        }
    }
}

impl Deref for ComboBox {
    type Target = Component;
    
    fn deref(&self) -> &Self::Target {
        &self.component
    }
}

impl DerefMut for ComboBox {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.component
    }
}

// Ensure ComboBox is !Send + !Sync (enforced by PhantomData<*mut ()>)
// This prevents combo boxes from being moved or shared across threads,
// which is required by JUCE's threading model.
