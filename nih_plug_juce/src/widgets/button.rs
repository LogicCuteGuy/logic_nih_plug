//! JUCE Button components.

use crate::bridge::ffi;
use crate::component::Component;
use crate::error::{JuceError, Result};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

pub struct TextButton {
    component: Component,
    _phantom: PhantomData<*mut ()>,
}

impl TextButton {
    pub fn new(text: &str) -> Result<Self> {
        let mut error_buf = vec![0u8; 256];
        
        let ptr = unsafe {
            ffi::create_text_button(
                text.as_ptr(),
                text.len(),
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
                    "Unknown error creating TextButton".to_string()
                ))
            } else {
                Err(JuceError::ComponentCreationFailed(error_msg))
            }
        } else {
            let component = unsafe { Component::from_raw(ptr) };
            
            Ok(TextButton {
                component,
                _phantom: PhantomData,
            })
        }
    }
    
    pub fn set_button_text(&mut self, text: &str) {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::button_set_text(ptr, text.as_ptr(), text.len());
        }
    }
    
    pub fn set_enabled(&mut self, enabled: bool) {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::button_set_enabled(ptr, enabled);
        }
    }
    
    pub fn set_colour(&mut self, colour_id: i32, r: u8, g: u8, b: u8, a: u8) {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::button_set_colour(ptr, colour_id, r, g, b, a);
        }
    }
    
    pub fn set_on_click<F>(&mut self, callback: F) -> Result<()>
    where
        F: Fn() + 'static,
    {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return Err(JuceError::NullPointer("Button pointer is null".to_string()));
        }
        
        let boxed = Box::new(callback);
        let raw = Box::into_raw(boxed);
        
        unsafe extern "C" fn trampoline<F>(closure_ptr: usize)
        where
            F: Fn(),
        {
            let closure = &*(closure_ptr as *const F);
            closure();
        }
        
        unsafe extern "C" fn drop_closure<F>(closure_ptr: usize)
        where
            F: Fn(),
        {
            let _ = Box::from_raw(closure_ptr as *mut F);
        }
        
        let mut error_buf = vec![0u8; 256];
        
        let result = unsafe {
            ffi::button_set_on_click(
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
            unsafe {
                let _ = Box::from_raw(raw);
            }
            
            let error_msg = String::from_utf8_lossy(&error_buf)
                .trim_end_matches('\0')
                .to_string();
            
            if error_msg.is_empty() {
                Err(JuceError::CallbackError(
                    "Unknown error setting button click callback".to_string()
                ))
            } else {
                Err(JuceError::CallbackError(error_msg))
            }
        }
    }
}

impl Deref for TextButton {
    type Target = Component;
    
    fn deref(&self) -> &Self::Target {
        &self.component
    }
}

impl DerefMut for TextButton {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.component
    }
}
