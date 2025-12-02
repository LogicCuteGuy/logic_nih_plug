//! Error types for JUCE FFI operations.
//!
//! This module provides comprehensive error handling for all JUCE GUI operations
//! performed through the FFI boundary. All errors are designed to provide clear,
//! actionable messages to help developers diagnose and fix issues.
//!
//! # Error Categories
//!
//! - **FFI Errors**: Low-level FFI call failures
//! - **C++ Exceptions**: Exceptions caught at the FFI boundary
//! - **Parameter Validation**: Invalid parameters passed to JUCE functions
//! - **Component Errors**: Component creation, lookup, or lifecycle issues
//! - **File Operations**: File I/O errors when loading/saving resources
//! - **Thread Safety**: Violations of JUCE's message thread requirements
//! - **Image Operations**: Image format or processing errors
//!
//! # Thread Safety
//!
//! Many JUCE operations must be performed on the message thread. Attempting to
//! call GUI operations from other threads will result in a `ThreadSafetyViolation`
//! error in debug builds, or undefined behavior in release builds. Always use
//! `MessageManager::call_async()` to safely update UI from other threads.
//!
//! # Examples
//!
//! ```ignore
//! use nih_plug_juce::{Component, JuceError, Result};
//!
//! fn create_ui() -> Result<Component> {
//!     let component = Component::new()?;
//!     component.set_bounds(0, 0, 400, 300)?;
//!     Ok(component)
//! }
//!
//! // Handle errors with context
//! match create_ui() {
//!     Ok(comp) => println!("UI created successfully"),
//!     Err(JuceError::ComponentCreationFailed(msg)) => {
//!         eprintln!("Failed to create component: {}", msg);
//!     }
//!     Err(e) => eprintln!("Unexpected error: {}", e),
//! }
//! ```

use thiserror::Error;

/// Result type alias for JUCE operations.
///
/// This is a convenience type alias that uses [`JuceError`] as the error type.
/// All JUCE FFI operations that can fail return this type.
///
/// # Examples
///
/// ```ignore
/// use nih_plug_juce::Result;
///
/// fn create_button(text: &str) -> Result<TextButton> {
///     TextButton::new(text)
/// }
/// ```
pub type Result<T> = std::result::Result<T, JuceError>;

/// Errors that can occur when using JUCE GUI components through FFI.
///
/// This enum covers all error conditions that can arise when interfacing with
/// JUCE's C++ GUI library. Each variant provides detailed context about what
/// went wrong to aid in debugging.
///
/// # Error Handling Strategy
///
/// All C++ exceptions are caught at the FFI boundary and converted to
/// [`JuceError::CppException`]. This ensures that C++ exceptions never
/// propagate into Rust code, which would cause undefined behavior.
///
/// # Thread Safety
///
/// Operations that violate JUCE's message thread requirement will produce
/// [`JuceError::ThreadSafetyViolation`] errors in debug builds. In release
/// builds, thread safety is enforced through the type system (GUI types
/// don't implement `Send` or `Sync`).
#[derive(Error, Debug)]
pub enum JuceError {
    /// An FFI call to JUCE failed.
    ///
    /// This error indicates a low-level failure in the FFI layer, such as
    /// a function returning an error code or a null pointer when a valid
    /// pointer was expected.
    ///
    /// # Context
    ///
    /// The error message includes details about which FFI function failed
    /// and why, when available.
    #[error("FFI call failed: {0}")]
    FfiError(String),

    /// A C++ exception was caught at the FFI boundary.
    ///
    /// JUCE C++ code can throw exceptions in exceptional circumstances.
    /// All FFI bridge functions catch these exceptions and convert them
    /// to this error variant, preventing C++ exceptions from propagating
    /// into Rust code.
    ///
    /// # Context
    ///
    /// The error message contains the exception message from C++, which
    /// typically includes details about what went wrong.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// match component.set_bounds(-100, -100, 0, 0) {
    ///     Err(JuceError::CppException(msg)) => {
    ///         eprintln!("JUCE threw exception: {}", msg);
    ///     }
    ///     _ => {}
    /// }
    /// ```
    #[error("C++ exception: {0}")]
    CppException(String),

    /// Invalid parameter provided to a JUCE function.
    ///
    /// This error occurs when a parameter fails validation before being
    /// passed to JUCE. Common cases include:
    /// - Negative dimensions for components
    /// - Out-of-range values for sliders or other controls
    /// - Invalid color values
    /// - Malformed strings or paths
    ///
    /// # Context
    ///
    /// The error message describes which parameter was invalid and why.
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// A required JUCE component was not found.
    ///
    /// This error occurs when attempting to access a component that doesn't
    /// exist, such as:
    /// - Looking up a child component by ID that doesn't exist
    /// - Accessing a tab that has been removed
    /// - Referencing a component that has been destroyed
    #[error("Component not found")]
    ComponentNotFound,

    /// Failed to create a JUCE component.
    ///
    /// Component creation can fail for various reasons:
    /// - Insufficient memory
    /// - Platform-specific initialization failures
    /// - Invalid component configuration
    ///
    /// # Context
    ///
    /// The error message includes the component type and reason for failure.
    #[error("Failed to create JUCE component: {0}")]
    ComponentCreationFailed(String),

    /// Attempted to use a null or invalid pointer.
    ///
    /// This error indicates a serious bug in the FFI layer where a null
    /// pointer was encountered when a valid pointer was expected. This
    /// should not occur in normal usage.
    ///
    /// # Context
    ///
    /// The error message describes which pointer was null and where.
    #[error("Null pointer error: {0}")]
    NullPointer(String),

    /// File operation failed.
    ///
    /// This error wraps standard I/O errors that occur when loading or
    /// saving files, such as:
    /// - Loading images from disk
    /// - Saving images to disk
    /// - Loading SVG files for Drawables
    /// - File chooser operations
    ///
    /// # Context
    ///
    /// The underlying [`std::io::Error`] provides details about what failed.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// match Image::load_from_file(path) {
    ///     Err(JuceError::FileError(io_err)) => {
    ///         eprintln!("Failed to load image: {}", io_err);
    ///     }
    ///     _ => {}
    /// }
    /// ```
    #[error("File operation failed: {0}")]
    FileError(#[from] std::io::Error),

    /// Image format is not supported.
    ///
    /// This error occurs when attempting to load or save an image in a
    /// format that JUCE doesn't support, or when the image data is
    /// corrupted or malformed.
    ///
    /// # Supported Formats
    ///
    /// JUCE typically supports: PNG, JPEG, GIF, BMP
    ///
    /// # Context
    ///
    /// The error message includes the format that was attempted.
    #[error("Image format not supported: {0}")]
    UnsupportedImageFormat(String),

    /// Thread safety violation detected.
    ///
    /// JUCE requires all GUI operations to be performed on the message thread.
    /// This error occurs when a GUI operation is attempted from a different
    /// thread in debug builds.
    ///
    /// # Prevention
    ///
    /// To update UI from other threads (e.g., the audio thread), use
    /// `MessageManager::call_async()`:
    ///
    /// ```ignore
    /// // From audio thread
    /// let value = compute_value();
    /// MessageManager::call_async(move || {
    ///     // This closure runs on the message thread
    ///     slider.set_value(value);
    /// });
    /// ```
    ///
    /// # Type System Enforcement
    ///
    /// In release builds, thread safety is enforced at compile time through
    /// the type system - GUI types don't implement `Send` or `Sync`, preventing
    /// them from being moved or shared across threads.
    #[error("Thread safety violation: operation must be called on message thread")]
    ThreadSafetyViolation,

    /// A general JUCE operation failed.
    ///
    /// This is a catch-all error for operations that don't fit into more
    /// specific categories. The error message provides context about what
    /// operation failed.
    ///
    /// # Context
    ///
    /// The error message describes the operation that failed and any
    /// available details about why.
    #[error("JUCE operation failed: {0}")]
    OperationFailed(String),
    
    /// Failed to set a callback on a component.
    ///
    /// This error occurs when attempting to set a callback (such as onClick,
    /// onValueChange, onTextChange) on a component and the operation fails.
    /// This can happen if:
    /// - The component doesn't support callbacks
    /// - The component pointer is invalid
    /// - Memory allocation for the callback failed
    ///
    /// # Context
    ///
    /// The error message describes which callback failed to set and why.
    #[error("Failed to set callback: {0}")]
    CallbackError(String),
}

// Implement conversion from string types for convenience
impl From<String> for JuceError {
    fn from(msg: String) -> Self {
        JuceError::OperationFailed(msg)
    }
}

impl From<&str> for JuceError {
    fn from(msg: &str) -> Self {
        JuceError::OperationFailed(msg.to_string())
    }
}
