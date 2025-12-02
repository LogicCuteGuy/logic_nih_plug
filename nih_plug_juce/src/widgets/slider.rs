//! JUCE Slider components.
//!
//! This module provides safe Rust wrappers around JUCE's Slider component,
//! which is used for parameter controls and value selection.
//!
//! # Thread Safety
//!
//! All slider operations must be performed on the JUCE message thread.
//! This is enforced through the type system - sliders do not implement
//! `Send` or `Sync`.
//!
//! # Examples
//!
//! ```ignore
//! use nih_plug_juce::widgets::{Slider, SliderStyle};
//!
//! let mut slider = Slider::new(SliderStyle::Linear)?;
//! slider.set_bounds(10, 10, 200, 30);
//! slider.set_range(0.0, 100.0, 0.1);
//! slider.set_value(50.0);
//! slider.set_on_value_change(|value| {
//!     println!("Slider value: {}", value);
//! })?;
//! ```

use crate::bridge::ffi;
use crate::component::Component;
use crate::error::{JuceError, Result};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// Slider style enumeration.
///
/// Defines the visual style and interaction behavior of a slider.
/// Different styles are appropriate for different use cases.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SliderStyle {
    /// Linear horizontal slider (default style)
    LinearHorizontal = 1,
    
    /// Linear vertical slider
    LinearVertical = 2,
    
    /// Linear bar slider (shows a filled bar)
    LinearBar = 3,
    
    /// Linear bar slider with vertical orientation
    LinearBarVertical = 4,
    
    /// Rotary slider (knob)
    Rotary = 5,
    
    /// Rotary slider controlled by horizontal dragging
    RotaryHorizontalDrag = 6,
    
    /// Rotary slider controlled by vertical dragging
    RotaryVerticalDrag = 7,
    
    /// Rotary slider controlled by horizontal or vertical dragging
    RotaryHorizontalVerticalDrag = 8,
    
    /// Slider with two values (range slider) - horizontal
    TwoValueHorizontal = 9,
    
    /// Slider with two values (range slider) - vertical
    TwoValueVertical = 10,
    
    /// Slider with three values - horizontal
    ThreeValueHorizontal = 11,
    
    /// Slider with three values - vertical
    ThreeValueVertical = 12,
}

/// A JUCE Slider - a control for selecting numeric values.
///
/// Slider is one of the most commonly used GUI components for audio plugins.
/// It can be displayed in various styles (linear, rotary, etc.) and provides
/// smooth value changes with optional snapping to intervals.
///
/// # Inheritance
///
/// Slider inherits from Component through Deref/DerefMut, so all
/// Component methods are available on Slider instances.
///
/// # Thread Safety
///
/// Slider does not implement `Send` or `Sync`, enforcing that all
/// slider operations occur on the message thread.
///
/// # Examples
///
/// ```ignore
/// use nih_plug_juce::widgets::{Slider, SliderStyle};
///
/// // Create a rotary slider
/// let mut slider = Slider::new(SliderStyle::Rotary)?;
///
/// // Set its position and size (inherited from Component)
/// slider.set_bounds(10, 10, 100, 100);
///
/// // Set slider-specific properties
/// slider.set_range(0.0, 1.0, 0.01);
/// slider.set_value(0.5);
///
/// // Set a value change callback
/// slider.set_on_value_change(|value| {
///     println!("New value: {}", value);
/// })?;
/// ```
pub struct Slider {
    /// The underlying Component.
    /// This is wrapped to provide the inheritance pattern through Deref/DerefMut.
    component: Component,
    
    /// PhantomData to make Slider !Send + !Sync.
    /// This enforces that Slider can only be used on the thread where
    /// it was created (the message thread), matching JUCE's requirements.
    _phantom: PhantomData<*mut ()>,
}

impl Slider {
    /// Create a new Slider with the specified style.
    ///
    /// This allocates a new juce::Slider in C++ and returns a safe
    /// Rust wrapper around it.
    ///
    /// # Arguments
    ///
    /// * `style` - The visual style of the slider
    ///
    /// # Returns
    ///
    /// Returns `Ok(Slider)` on success, or an error if slider
    /// creation failed.
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use nih_plug_juce::widgets::{Slider, SliderStyle};
    ///
    /// let slider = Slider::new(SliderStyle::Rotary)?;
    /// ```
    pub fn new(style: SliderStyle) -> Result<Self> {
        let mut error_buf = vec![0u8; 256];
        
        let ptr = unsafe {
            ffi::create_slider(
                style as i32,
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
                    "Unknown error creating Slider".to_string()
                ))
            } else {
                Err(JuceError::ComponentCreationFailed(error_msg))
            }
        } else {
            // Wrap the raw pointer in a Component
            // Safety: We know this is a valid JuceComponent pointer from create_slider
            let component = unsafe { Component::from_raw(ptr) };
            
            Ok(Slider {
                component,
                _phantom: PhantomData,
            })
        }
    }
    
    /// Set the range of values the slider can represent.
    ///
    /// This defines the minimum and maximum values, as well as the
    /// interval for snapping. If interval is 0, the slider will be
    /// continuous without snapping.
    ///
    /// # Arguments
    ///
    /// * `min` - Minimum value
    /// * `max` - Maximum value
    /// * `interval` - Snapping interval (0 for continuous)
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut slider = Slider::new(SliderStyle::Linear)?;
    /// slider.set_range(0.0, 100.0, 1.0); // Integer values 0-100
    /// ```
    pub fn set_range(&mut self, min: f64, max: f64, interval: f64) {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::slider_set_range(ptr, min, max, interval);
        }
    }
    
    /// Set the current value of the slider.
    ///
    /// This updates the slider's value. The value will be clamped to
    /// the slider's range and snapped to the interval if one is set.
    /// This will not trigger the value change callback.
    ///
    /// # Arguments
    ///
    /// * `value` - The new value
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut slider = Slider::new(SliderStyle::Rotary)?;
    /// slider.set_range(0.0, 1.0, 0.0);
    /// slider.set_value(0.5);
    /// ```
    pub fn set_value(&mut self, value: f64) {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::slider_set_value(ptr, value);
        }
    }
    
    /// Get the current value of the slider.
    ///
    /// # Returns
    ///
    /// Returns the current slider value.
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let slider = Slider::new(SliderStyle::Linear)?;
    /// let value = slider.get_value();
    /// println!("Current value: {}", value);
    /// ```
    pub fn get_value(&self) -> f64 {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return 0.0;
        }
        
        unsafe {
            ffi::slider_get_value(ptr)
        }
    }
    
    /// Set a callback to be invoked when the slider value changes.
    ///
    /// The callback will be invoked on the message thread whenever the
    /// slider value is changed by the user. The callback receives the
    /// new value as a parameter.
    ///
    /// # Arguments
    ///
    /// * `callback` - A closure to invoke when the value changes
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
    /// let mut slider = Slider::new(SliderStyle::Rotary)?;
    /// slider.set_on_value_change(|value| {
    ///     println!("Slider value changed to: {}", value);
    /// })?;
    /// ```
    pub fn set_on_value_change<F>(&mut self, callback: F) -> Result<()>
    where
        F: Fn(f64) + 'static,
    {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return Err(JuceError::NullPointer("Slider pointer is null".to_string()));
        }
        
        // Box the closure and convert to raw pointer
        let boxed = Box::new(callback);
        let raw = Box::into_raw(boxed);
        
        // Define the trampoline function that will be called from C++
        unsafe extern "C" fn trampoline<F>(closure_ptr: usize, value: f64)
        where
            F: Fn(f64),
        {
            // Reconstruct the closure reference (without taking ownership)
            let closure = &*(closure_ptr as *const F);
            
            // Invoke the Rust closure with the value
            closure(value);
        }
        
        // Define the drop function that will be called when the slider is destroyed
        unsafe extern "C" fn drop_closure<F>(closure_ptr: usize)
        where
            F: Fn(f64),
        {
            // Take ownership and drop the closure
            let _ = Box::from_raw(closure_ptr as *mut F);
        }
        
        let mut error_buf = vec![0u8; 256];
        
        let result = unsafe {
            ffi::slider_set_on_value_change(
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
                    "Unknown error setting slider value change callback".to_string()
                ))
            } else {
                Err(JuceError::CallbackError(error_msg))
            }
        }
    }
}

impl Deref for Slider {
    type Target = Component;
    
    fn deref(&self) -> &Self::Target {
        &self.component
    }
}

impl DerefMut for Slider {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.component
    }
}

// Ensure Slider is !Send + !Sync (enforced by PhantomData<*mut ()>)
// This prevents sliders from being moved or shared across threads,
// which is required by JUCE's threading model.
