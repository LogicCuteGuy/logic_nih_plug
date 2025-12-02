//! JUCE FileChooser for file selection dialogs.
//!
//! This module provides a safe Rust wrapper around JUCE's FileChooser class,
//! which is used to display native file open/save dialogs.
//!
//! # Thread Safety
//!
//! All FileChooser operations must be performed on the JUCE message thread.
//! The browse methods are asynchronous and invoke callbacks when the user
//! makes a selection.
//!
//! # Examples
//!
//! ```ignore
//! use nih_plug_juce::dialogs::FileChooser;
//! use std::path::PathBuf;
//!
//! // Create a file chooser for opening audio files
//! let mut chooser = FileChooser::new(
//!     "Select Audio File",
//!     PathBuf::from("/home/user/Music"),
//!     "*.wav;*.mp3;*.flac"
//! );
//!
//! // Browse for a file to open
//! chooser.browse_for_file_to_open(|path| {
//!     if let Some(path) = path {
//!         println!("User selected: {:?}", path);
//!     } else {
//!         println!("User cancelled");
//!     }
//! })?;
//!
//! // Browse for a file to save
//! chooser.browse_for_file_to_save(|path| {
//!     if let Some(path) = path {
//!         println!("Save to: {:?}", path);
//!     }
//! })?;
//! ```

use crate::bridge::ffi;
use crate::error::{JuceError, Result};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

/// A JUCE FileChooser for displaying file selection dialogs.
///
/// FileChooser provides methods for showing native file open and save dialogs.
/// The dialogs are asynchronous and invoke callbacks when the user makes a selection.
///
/// # Thread Safety
///
/// All FileChooser methods must be called on the JUCE message thread.
/// Callbacks will also be invoked on the message thread.
///
/// # Examples
///
/// ```ignore
/// use nih_plug_juce::dialogs::FileChooser;
/// use std::path::PathBuf;
///
/// let mut chooser = FileChooser::new(
///     "Open File",
///     PathBuf::from("."),
///     "*.txt;*.md"
/// );
///
/// chooser.browse_for_file_to_open(|path| {
///     match path {
///         Some(p) => println!("Selected: {:?}", p),
///         None => println!("Cancelled"),
///     }
/// })?;
/// ```
pub struct FileChooser {
    ptr: *mut ffi::JuceFileChooser,
    _phantom: PhantomData<*mut ()>, // !Send + !Sync
}

impl FileChooser {
    /// Create a new FileChooser.
    ///
    /// # Arguments
    ///
    /// * `title` - The title to display in the dialog
    /// * `initial_dir` - The initial directory to show
    /// * `filters` - File filters in the format "*.ext1;*.ext2" (e.g., "*.wav;*.mp3")
    ///
    /// # Returns
    ///
    /// Returns a new FileChooser instance, or an error if creation failed.
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use nih_plug_juce::dialogs::FileChooser;
    /// use std::path::PathBuf;
    ///
    /// let chooser = FileChooser::new(
    ///     "Select Audio File",
    ///     PathBuf::from("/home/user/Music"),
    ///     "*.wav;*.mp3;*.flac"
    /// )?;
    /// ```
    pub fn new(title: &str, initial_dir: &Path, filters: &str) -> Result<Self> {
        let mut error_buf = vec![0u8; 256];
        
        // Convert PathBuf to string
        let dir_str = initial_dir
            .to_str()
            .ok_or_else(|| JuceError::InvalidParameter("Invalid path encoding".to_string()))?;
        
        let ptr = unsafe {
            ffi::create_file_chooser(
                title.as_ptr(),
                title.len(),
                dir_str.as_ptr(),
                dir_str.len(),
                filters.as_ptr(),
                filters.len(),
                error_buf.as_mut_ptr() as *mut i8,
                error_buf.len(),
            )
        };
        
        if ptr.is_null() {
            let error_msg = String::from_utf8_lossy(&error_buf)
                .trim_end_matches('\0')
                .to_string();
            
            if error_msg.is_empty() {
                Err(JuceError::ComponentCreationFailed(
                    "Failed to create FileChooser".to_string()
                ))
            } else {
                Err(JuceError::ComponentCreationFailed(error_msg))
            }
        } else {
            Ok(Self {
                ptr,
                _phantom: PhantomData,
            })
        }
    }
    
    /// Browse for a file to open.
    ///
    /// This displays a native file open dialog and returns immediately.
    /// When the user selects a file or cancels, the provided callback is invoked
    /// on the message thread with `Some(PathBuf)` for a selection or `None` for cancel.
    ///
    /// # Arguments
    ///
    /// * `callback` - A closure to invoke with the selected file path (or None if cancelled)
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if setting up the dialog failed.
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread. The callback
    /// will also be invoked on the message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use nih_plug_juce::dialogs::FileChooser;
    /// use std::path::PathBuf;
    ///
    /// let mut chooser = FileChooser::new("Open", PathBuf::from("."), "*.*")?;
    ///
    /// chooser.browse_for_file_to_open(|path| {
    ///     if let Some(path) = path {
    ///         println!("Opening file: {:?}", path);
    ///         // Load the file...
    ///     }
    /// })?;
    /// ```
    pub fn browse_for_file_to_open<F>(&mut self, callback: F) -> Result<()>
    where
        F: Fn(Option<PathBuf>) + 'static,
    {
        // Box the closure and convert to raw pointer
        let boxed = Box::new(callback);
        let raw = Box::into_raw(boxed);
        
        // Define the trampoline function that will be called from C++
        unsafe extern "C" fn trampoline<F>(
            closure_ptr: usize,
            path_ptr: *const u8,
            path_len: usize,
        ) where
            F: Fn(Option<PathBuf>),
        {
            // Reconstruct the closure reference (without taking ownership)
            let closure = &*(closure_ptr as *const F);
            
            // Convert the path from C++ to Rust PathBuf
            let path = if path_ptr.is_null() || path_len == 0 {
                None
            } else {
                let path_bytes = std::slice::from_raw_parts(path_ptr, path_len);
                let path_str = std::str::from_utf8_unchecked(path_bytes);
                Some(PathBuf::from(path_str))
            };
            
            // Invoke the Rust closure with the path
            closure(path);
        }
        
        // Define the drop function that will be called when the callback is no longer needed
        unsafe extern "C" fn drop_closure<F>(closure_ptr: usize)
        where
            F: Fn(Option<PathBuf>),
        {
            // Take ownership and drop the closure
            let _ = Box::from_raw(closure_ptr as *mut F);
        }
        
        let mut error_buf = vec![0u8; 256];
        
        let result = unsafe {
            ffi::file_chooser_browse_for_file_to_open(
                self.ptr,
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
                    "Unknown error browsing for file to open".to_string()
                ))
            } else {
                Err(JuceError::CallbackError(error_msg))
            }
        }
    }
    
    /// Browse for a file to save.
    ///
    /// This displays a native file save dialog and returns immediately.
    /// When the user selects a file or cancels, the provided callback is invoked
    /// on the message thread with `Some(PathBuf)` for a selection or `None` for cancel.
    ///
    /// # Arguments
    ///
    /// * `callback` - A closure to invoke with the selected file path (or None if cancelled)
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if setting up the dialog failed.
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread. The callback
    /// will also be invoked on the message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use nih_plug_juce::dialogs::FileChooser;
    /// use std::path::PathBuf;
    ///
    /// let mut chooser = FileChooser::new("Save As", PathBuf::from("."), "*.txt")?;
    ///
    /// chooser.browse_for_file_to_save(|path| {
    ///     if let Some(path) = path {
    ///         println!("Saving to: {:?}", path);
    ///         // Save the file...
    ///     }
    /// })?;
    /// ```
    pub fn browse_for_file_to_save<F>(&mut self, callback: F) -> Result<()>
    where
        F: Fn(Option<PathBuf>) + 'static,
    {
        // Box the closure and convert to raw pointer
        let boxed = Box::new(callback);
        let raw = Box::into_raw(boxed);
        
        // Define the trampoline function that will be called from C++
        unsafe extern "C" fn trampoline<F>(
            closure_ptr: usize,
            path_ptr: *const u8,
            path_len: usize,
        ) where
            F: Fn(Option<PathBuf>),
        {
            // Reconstruct the closure reference (without taking ownership)
            let closure = &*(closure_ptr as *const F);
            
            // Convert the path from C++ to Rust PathBuf
            let path = if path_ptr.is_null() || path_len == 0 {
                None
            } else {
                let path_bytes = std::slice::from_raw_parts(path_ptr, path_len);
                let path_str = std::str::from_utf8_unchecked(path_bytes);
                Some(PathBuf::from(path_str))
            };
            
            // Invoke the Rust closure with the path
            closure(path);
        }
        
        // Define the drop function that will be called when the callback is no longer needed
        unsafe extern "C" fn drop_closure<F>(closure_ptr: usize)
        where
            F: Fn(Option<PathBuf>),
        {
            // Take ownership and drop the closure
            let _ = Box::from_raw(closure_ptr as *mut F);
        }
        
        let mut error_buf = vec![0u8; 256];
        
        let result = unsafe {
            ffi::file_chooser_browse_for_file_to_save(
                self.ptr,
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
                    "Unknown error browsing for file to save".to_string()
                ))
            } else {
                Err(JuceError::CallbackError(error_msg))
            }
        }
    }
}

impl Drop for FileChooser {
    fn drop(&mut self) {
        unsafe {
            ffi::delete_file_chooser(self.ptr);
        }
    }
}
