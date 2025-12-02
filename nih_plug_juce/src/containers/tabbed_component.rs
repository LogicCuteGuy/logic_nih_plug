//! JUCE TabbedComponent.
//!
//! This module provides a safe Rust wrapper around JUCE's TabbedComponent class,
//! which provides a tabbed interface for organizing multiple components.
//!
//! # Thread Safety
//!
//! All TabbedComponent operations must be performed on the JUCE message thread.
//! This is enforced through the type system - TabbedComponent does not implement
//! `Send` or `Sync`.
//!
//! # Examples
//!
//! ```ignore
//! use nih_plug_juce::containers::{TabbedComponent, TabOrientation};
//! use nih_plug_juce::Component;
//! use nih_plug_juce::drawing::Colour;
//!
//! let mut tabbed = TabbedComponent::new(TabOrientation::Top)?;
//! let tab1_content = Component::new()?;
//! let tab2_content = Component::new()?;
//! 
//! tabbed.add_tab("Tab 1", Colour::from_rgb(100, 100, 100), tab1_content)?;
//! tabbed.add_tab("Tab 2", Colour::from_rgb(150, 150, 150), tab2_content)?;
//! tabbed.set_current_tab_index(0);
//! ```

use crate::bridge::ffi;
use crate::component::Component;
use crate::drawing::Colour;
use crate::error::{JuceError, Result};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// Tab orientation for TabbedComponent.
///
/// Specifies where the tabs should be positioned relative to the content area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabOrientation {
    /// Tabs at the top of the content area.
    Top,
    /// Tabs at the bottom of the content area.
    Bottom,
    /// Tabs on the left side of the content area.
    Left,
    /// Tabs on the right side of the content area.
    Right,
}

impl TabOrientation {
    /// Convert to the integer value expected by JUCE.
    fn to_juce_value(self) -> i32 {
        match self {
            TabOrientation::Top => 0,
            TabOrientation::Bottom => 1,
            TabOrientation::Left => 2,
            TabOrientation::Right => 3,
        }
    }
}

/// A JUCE TabbedComponent - a tabbed interface for organizing components.
///
/// TabbedComponent provides a tabbed interface where each tab contains a
/// different component. Users can switch between tabs to view different content.
///
/// # Inheritance
///
/// TabbedComponent inherits from Component through Deref/DerefMut, so all
/// Component methods are available on TabbedComponent instances.
///
/// # Thread Safety
///
/// TabbedComponent does not implement `Send` or `Sync`, enforcing that all
/// tabbed component operations occur on the message thread.
///
/// # Examples
///
/// ```ignore
/// use nih_plug_juce::containers::{TabbedComponent, TabOrientation};
/// use nih_plug_juce::Component;
/// use nih_plug_juce::drawing::Colour;
///
/// // Create a tabbed component with tabs at the top
/// let mut tabbed = TabbedComponent::new(TabOrientation::Top)?;
///
/// // Create content for tabs
/// let mut tab1 = Component::new()?;
/// tab1.set_bounds(0, 0, 400, 300);
/// let mut tab2 = Component::new()?;
/// tab2.set_bounds(0, 0, 400, 300);
///
/// // Add tabs
/// let tab_colour = Colour::from_rgb(100, 100, 100);
/// tabbed.add_tab("Settings", tab_colour.clone(), tab1)?;
/// tabbed.add_tab("About", tab_colour, tab2)?;
///
/// // Set the active tab
/// tabbed.set_current_tab_index(0);
/// ```
pub struct TabbedComponent {
    /// The underlying Component.
    /// This is wrapped to provide the inheritance pattern through Deref/DerefMut.
    component: Component,
    
    /// PhantomData to make TabbedComponent !Send + !Sync.
    /// This enforces that TabbedComponent can only be used on the thread where
    /// it was created (the message thread), matching JUCE's requirements.
    _phantom: PhantomData<*mut ()>,
}

impl TabbedComponent {
    /// Create a new TabbedComponent with the specified tab orientation.
    ///
    /// This allocates a new juce::TabbedComponent in C++ and returns a safe
    /// Rust wrapper around it.
    ///
    /// # Arguments
    ///
    /// * `orientation` - The position of the tabs relative to the content area
    ///
    /// # Returns
    ///
    /// Returns `Ok(TabbedComponent)` on success, or an error if tabbed component
    /// creation failed.
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use nih_plug_juce::containers::{TabbedComponent, TabOrientation};
    ///
    /// let tabbed = TabbedComponent::new(TabOrientation::Top)?;
    /// ```
    pub fn new(orientation: TabOrientation) -> Result<Self> {
        let mut error_buf = vec![0u8; 256];
        
        let ptr = unsafe {
            ffi::create_tabbed_component(
                orientation.to_juce_value(),
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
                    "Unknown error creating TabbedComponent".to_string()
                ))
            } else {
                Err(JuceError::ComponentCreationFailed(error_msg))
            }
        } else {
            // Wrap the raw pointer in a Component
            // Safety: We know this is a valid JuceComponent pointer from create_tabbed_component
            let component = unsafe { Component::from_raw(ptr) };
            
            Ok(TabbedComponent {
                component,
                _phantom: PhantomData,
            })
        }
    }
    
    /// Add a new tab with the specified name, color, and content.
    ///
    /// The tabbed component takes ownership of the content component and will
    /// manage its lifetime. The content will be displayed when the tab is selected.
    ///
    /// # Arguments
    ///
    /// * `name` - The name to display on the tab
    /// * `colour` - The background color for the tab
    /// * `content` - The component to display when this tab is selected
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the operation failed.
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut tabbed = TabbedComponent::new(TabOrientation::Top)?;
    /// let content = Component::new()?;
    /// let colour = Colour::from_rgb(100, 100, 100);
    /// tabbed.add_tab("My Tab", colour, content)?;
    /// ```
    pub fn add_tab(&mut self, name: &str, colour: Colour, content: Component) -> Result<()> {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return Err(JuceError::NullPointer("TabbedComponent pointer is null".to_string()));
        }
        
        let content_ptr = content.as_ptr();
        if content_ptr.is_null() {
            return Err(JuceError::NullPointer("Content component pointer is null".to_string()));
        }
        
        let colour_ptr = unsafe { colour.as_ptr() };
        if colour_ptr.is_null() {
            return Err(JuceError::NullPointer("Colour pointer is null".to_string()));
        }
        
        let mut error_buf = vec![0u8; 256];
        
        let result = unsafe {
            ffi::tabbed_component_add_tab(
                ptr,
                name.as_ptr(),
                name.len(),
                colour_ptr,
                content_ptr,
                error_buf.as_mut_ptr() as *mut i8,
                error_buf.len()
            )
        };
        
        if result == 0 {
            // Prevent the content component from being dropped since JUCE now owns it
            std::mem::forget(content);
            // Colour is copied by JUCE, so we don't need to forget it
            Ok(())
        } else {
            let error_msg = String::from_utf8_lossy(&error_buf)
                .trim_end_matches('\0')
                .to_string();
            Err(JuceError::CppException(error_msg))
        }
    }
    
    /// Remove a tab at the specified index.
    ///
    /// This removes the tab and its associated content component. The content
    /// component will be destroyed by JUCE.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the tab to remove (0-based)
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut tabbed = TabbedComponent::new(TabOrientation::Top)?;
    /// // ... add some tabs ...
    /// tabbed.remove_tab(0); // Remove the first tab
    /// ```
    pub fn remove_tab(&mut self, index: i32) {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::tabbed_component_remove_tab(ptr, index);
        }
    }
    
    /// Set the currently selected tab by index.
    ///
    /// This switches to the specified tab, making its content visible.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the tab to select (0-based)
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut tabbed = TabbedComponent::new(TabOrientation::Top)?;
    /// // ... add some tabs ...
    /// tabbed.set_current_tab_index(1); // Switch to the second tab
    /// ```
    pub fn set_current_tab_index(&mut self, index: i32) {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::tabbed_component_set_current_tab_index(ptr, index);
        }
    }
    
    /// Set a callback to be invoked when the current tab changes.
    ///
    /// The callback receives the index of the newly selected tab.
    ///
    /// # Arguments
    ///
    /// * `callback` - A closure that receives the new tab index
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
    /// let mut tabbed = TabbedComponent::new(TabOrientation::Top)?;
    /// tabbed.set_on_tab_changed(|index| {
    ///     println!("Switched to tab {}", index);
    /// })?;
    /// ```
    pub fn set_on_tab_changed<F>(&mut self, callback: F) -> Result<()>
    where
        F: Fn(i32) + 'static,
    {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return Err(JuceError::NullPointer("TabbedComponent pointer is null".to_string()));
        }
        
        // Box the closure and convert to raw pointer
        let boxed = Box::new(callback);
        let raw = Box::into_raw(boxed);
        
        // Define the trampoline function that will be called from C++
        unsafe extern "C" fn trampoline<F>(closure_ptr: usize, index: i32)
        where
            F: Fn(i32),
        {
            // Reconstruct the closure reference (without taking ownership)
            let closure = &*(closure_ptr as *const F);
            
            // Invoke the Rust closure
            closure(index);
        }
        
        // Define the drop function that will be called when the tabbed component is destroyed
        unsafe extern "C" fn drop_closure<F>(closure_ptr: usize)
        where
            F: Fn(i32),
        {
            // Take ownership and drop the closure
            let _ = Box::from_raw(closure_ptr as *mut F);
        }
        
        let mut error_buf = vec![0u8; 256];
        
        let result = unsafe {
            ffi::tabbed_component_set_on_tab_changed(
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
                    "Unknown error setting tab changed callback".to_string()
                ))
            } else {
                Err(JuceError::CallbackError(error_msg))
            }
        }
    }
}

impl Deref for TabbedComponent {
    type Target = Component;
    
    fn deref(&self) -> &Self::Target {
        &self.component
    }
}

impl DerefMut for TabbedComponent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.component
    }
}

// Ensure TabbedComponent is !Send + !Sync (enforced by PhantomData<*mut ()>)
// This prevents tabbed components from being moved or shared across threads,
// which is required by JUCE's threading model.
