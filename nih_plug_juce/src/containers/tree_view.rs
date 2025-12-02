//! JUCE TreeView component.
//!
//! This module provides a safe Rust wrapper around JUCE's TreeView class,
//! which displays hierarchical data in a tree structure with expandable/collapsible nodes.
//!
//! # Thread Safety
//!
//! All TreeView operations must be performed on the JUCE message thread.
//! This is enforced through the type system - TreeView does not implement
//! `Send` or `Sync`.
//!
//! # Examples
//!
//! ```ignore
//! use nih_plug_juce::containers::{TreeView, TreeViewItem};
//! use nih_plug_juce::Graphics;
//!
//! struct MyTreeItem {
//!     name: String,
//!     children: Vec<Box<dyn TreeViewItem>>,
//! }
//!
//! impl TreeViewItem for MyTreeItem {
//!     fn get_num_sub_items(&self) -> i32 {
//!         self.children.len() as i32
//!     }
//!
//!     fn get_sub_item(&self, index: i32) -> Option<Box<dyn TreeViewItem>> {
//!         self.children.get(index as usize).map(|child| {
//!             // Clone or create a new item
//!             // Note: This is a simplified example
//!             None
//!         })
//!     }
//!
//!     fn paint_item(&self, g: &mut Graphics, width: i32, height: i32) {
//!         // Draw item...
//!     }
//!
//!     fn item_clicked(&mut self) {
//!         println!("Item clicked: {}", self.name);
//!     }
//! }
//!
//! let mut tree_view = TreeView::new()?;
//! let root = Box::new(MyTreeItem { name: "Root".to_string(), children: vec![] });
//! tree_view.set_root_item(root)?;
//! ```

use crate::bridge::ffi;
use crate::component::Component;
use crate::error::{JuceError, Result};
use crate::graphics::Graphics;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// Trait for implementing custom tree view items.
///
/// Implement this trait to provide custom rendering and behavior for tree items.
/// The TreeView will call these methods to determine the tree structure,
/// how to render each item, and when items are clicked.
///
/// # Thread Safety
///
/// All methods will be called on the JUCE message thread.
///
/// # Examples
///
/// ```ignore
/// struct MyTreeItem {
///     name: String,
///     children: Vec<MyTreeItem>,
/// }
///
/// impl TreeViewItem for MyTreeItem {
///     fn get_num_sub_items(&self) -> i32 {
///         self.children.len() as i32
///     }
///
///     fn get_sub_item(&self, index: i32) -> Option<Box<dyn TreeViewItem>> {
///         // Return a boxed sub-item
///         None
///     }
///
///     fn paint_item(&self, g: &mut Graphics, width: i32, height: i32) {
///         // Custom rendering logic
///     }
///
///     fn item_clicked(&mut self) {
///         // Handle click
///     }
/// }
/// ```
pub trait TreeViewItem {
    /// Get the number of sub-items (children) of this item.
    ///
    /// This method is called by the TreeView to determine how many children
    /// this item has.
    ///
    /// # Returns
    ///
    /// The number of sub-items.
    fn get_num_sub_items(&self) -> i32;

    /// Get a sub-item by index.
    ///
    /// This method is called by the TreeView to retrieve a specific child item.
    /// The returned item should be a new boxed trait object.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the sub-item to retrieve (0-based)
    ///
    /// # Returns
    ///
    /// The sub-item at the specified index, or None if the index is out of bounds.
    fn get_sub_item(&self, index: i32) -> Option<Box<dyn TreeViewItem>>;

    /// Paint this tree item.
    ///
    /// This method is called by the TreeView to render this item.
    ///
    /// # Arguments
    ///
    /// * `g` - The Graphics context to draw with
    /// * `width` - The width of the item area
    /// * `height` - The height of the item area
    fn paint_item(&self, g: &mut Graphics, width: i32, height: i32);

    /// Called when this item is clicked.
    ///
    /// This method is called by the TreeView when the user clicks on this item.
    fn item_clicked(&mut self);
}

/// A JUCE TreeView - displays hierarchical data in a tree structure.
///
/// TreeView provides a tree display where each item can have children
/// and can be expanded or collapsed. The tree behavior is defined by
/// implementing the TreeViewItem trait.
///
/// # Inheritance
///
/// TreeView inherits from Component through Deref/DerefMut, so all
/// Component methods are available on TreeView instances.
///
/// # Thread Safety
///
/// TreeView does not implement `Send` or `Sync`, enforcing that all
/// tree view operations occur on the message thread.
///
/// # Examples
///
/// ```ignore
/// use nih_plug_juce::containers::{TreeView, TreeViewItem};
///
/// // Create a tree view
/// let mut tree_view = TreeView::new()?;
///
/// // Create and set a root item
/// let root = Box::new(MyTreeItem::new());
/// tree_view.set_root_item(root)?;
/// ```
pub struct TreeView {
    /// The underlying Component.
    /// This is wrapped to provide the inheritance pattern through Deref/DerefMut.
    component: Component,

    /// PhantomData to make TreeView !Send + !Sync.
    /// This enforces that TreeView can only be used on the thread where
    /// it was created (the message thread), matching JUCE's requirements.
    _phantom: PhantomData<*mut ()>,
}

impl TreeView {
    /// Create a new TreeView.
    ///
    /// This allocates a new juce::TreeView in C++ and returns a safe
    /// Rust wrapper around it.
    ///
    /// # Returns
    ///
    /// Returns `Ok(TreeView)` on success, or an error if tree view
    /// creation failed.
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use nih_plug_juce::containers::TreeView;
    ///
    /// let tree_view = TreeView::new()?;
    /// ```
    pub fn new() -> Result<Self> {
        let mut error_buf = vec![0u8; 256];

        let ptr = unsafe {
            ffi::create_tree_view(error_buf.as_mut_ptr() as *mut i8, error_buf.len())
        };

        if ptr.is_null() {
            let error_msg = String::from_utf8_lossy(&error_buf)
                .trim_end_matches('\0')
                .to_string();

            if error_msg.is_empty() {
                Err(JuceError::ComponentCreationFailed(
                    "Unknown error creating TreeView".to_string(),
                ))
            } else {
                Err(JuceError::ComponentCreationFailed(error_msg))
            }
        } else {
            // Wrap the raw pointer in a Component
            // Safety: We know this is a valid JuceComponent pointer from create_tree_view
            let component = unsafe { Component::from_raw(ptr) };

            Ok(TreeView {
                component,
                _phantom: PhantomData,
            })
        }
    }

    /// Set the root item for this tree view.
    ///
    /// The tree view takes ownership of the root item and will call its methods
    /// to determine the tree structure and rendering. The root item will be kept
    /// alive for the lifetime of the tree view.
    ///
    /// # Arguments
    ///
    /// * `root` - The root tree view item to use
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if setting the root item failed.
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut tree_view = TreeView::new()?;
    /// let root = Box::new(MyTreeItem::new());
    /// tree_view.set_root_item(root)?;
    /// ```
    pub fn set_root_item(&mut self, root: Box<dyn TreeViewItem>) -> Result<()> {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return Err(JuceError::NullPointer("TreeView pointer is null".to_string()));
        }

        // Box the root item and convert to raw pointer
        let boxed = Box::new(root);
        let raw = Box::into_raw(boxed);

        // Define the trampoline functions that will be called from C++
        unsafe extern "C" fn get_num_sub_items_trampoline(item_ptr: usize) -> i32 {
            // Reconstruct the item reference (without taking ownership)
            let item = &*(item_ptr as *const Box<dyn TreeViewItem>);
            item.get_num_sub_items()
        }

        unsafe extern "C" fn get_sub_item_trampoline(
            item_ptr: usize,
            index: i32,
        ) -> usize {
            // Reconstruct the item reference (without taking ownership)
            let item = &*(item_ptr as *const Box<dyn TreeViewItem>);
            
            if let Some(sub_item) = item.get_sub_item(index) {
                // Box the sub-item and return as raw pointer
                let boxed = Box::new(sub_item);
                Box::into_raw(boxed) as usize
            } else {
                0 // nullptr
            }
        }

        unsafe extern "C" fn paint_item_trampoline(
            item_ptr: usize,
            graphics_ptr: usize,
            width: i32,
            height: i32,
        ) {
            // Reconstruct the item reference (without taking ownership)
            let item = &*(item_ptr as *const Box<dyn TreeViewItem>);

            // Create a Graphics wrapper from the raw pointer
            // Safety: The graphics pointer is valid for the duration of this call
            let mut graphics = Graphics::from_raw(graphics_ptr as *mut crate::bridge::ffi::JuceGraphics);

            item.paint_item(&mut graphics, width, height);

            // Prevent Graphics from being dropped since JUCE owns it
            std::mem::forget(graphics);
        }

        unsafe extern "C" fn item_clicked_trampoline(item_ptr: usize) {
            // Reconstruct the item reference (without taking ownership)
            let item = &mut *(item_ptr as *mut Box<dyn TreeViewItem>);
            item.item_clicked();
        }

        unsafe extern "C" fn drop_item_trampoline(item_ptr: usize) {
            // Take ownership and drop the item
            let _ = Box::from_raw(item_ptr as *mut Box<dyn TreeViewItem>);
        }

        let mut error_buf = vec![0u8; 256];

        let result = unsafe {
            ffi::tree_view_set_root_item(
                ptr,
                raw as usize,
                get_num_sub_items_trampoline as usize,
                get_sub_item_trampoline as usize,
                paint_item_trampoline as usize,
                item_clicked_trampoline as usize,
                drop_item_trampoline as usize,
                error_buf.as_mut_ptr() as *mut i8,
                error_buf.len(),
            )
        };

        if result == 0 {
            Ok(())
        } else {
            // If setting the root item failed, we need to clean up the boxed item
            unsafe {
                let _ = Box::from_raw(raw);
            }

            let error_msg = String::from_utf8_lossy(&error_buf)
                .trim_end_matches('\0')
                .to_string();

            if error_msg.is_empty() {
                Err(JuceError::CallbackError(
                    "Unknown error setting tree view root item".to_string(),
                ))
            } else {
                Err(JuceError::CallbackError(error_msg))
            }
        }
    }
}

impl Deref for TreeView {
    type Target = Component;

    fn deref(&self) -> &Self::Target {
        &self.component
    }
}

impl DerefMut for TreeView {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.component
    }
}

// Ensure TreeView is !Send + !Sync (enforced by PhantomData<*mut ()>)
// This prevents tree views from being moved or shared across threads,
// which is required by JUCE's threading model.
