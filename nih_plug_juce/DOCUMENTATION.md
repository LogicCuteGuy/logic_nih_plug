# nih_plug_juce API Documentation Guide

This document provides an overview of the API documentation structure and guidelines for using and contributing to the nih_plug_juce crate.

## Documentation Structure

The API documentation is organized into the following modules:

### Core Modules

- **`component`** - Base Component class for all GUI elements
- **`graphics`** - Graphics context for 2D drawing operations
- **`error`** - Error types and Result alias
- **`bridge`** - FFI bridge layer (internal, not for public use)

### Widget Modules

- **`widgets::button`** - TextButton component
- **`widgets::slider`** - Slider component (linear, rotary, etc.)
- **`widgets::label`** - Label component for text display
- **`widgets::combo_box`** - ComboBox dropdown component
- **`widgets::text_editor`** - TextEditor for multi-line text input
- **`widgets::toggle_button`** - ToggleButton for checkboxes and toggles

### Container Modules

- **`containers::document_window`** - Top-level window
- **`containers::resizable_window`** - Resizable window
- **`containers::viewport`** - Scrollable viewport
- **`containers::tabbed_component`** - Tabbed interface
- **`containers::list_box`** - Scrollable list with custom model
- **`containers::tree_view`** - Hierarchical tree view

### Drawing Modules

- **`drawing::colour`** - RGB/RGBA color representation
- **`drawing::font`** - Font management and text measurement
- **`drawing::image`** - Bitmap image loading and manipulation
- **`drawing::path`** - Vector path for custom shapes
- **`drawing::transform`** - 2D affine transformations
- **`drawing::drawable`** - SVG and vector graphics

### Layout Modules

- **`layout::flexbox`** - Flexbox layout system

### Event Modules

- **`events::mouse`** - Mouse event handling
- **`events::keyboard`** - Keyboard event handling
- **`events::timer`** - Timer for periodic callbacks

### Dialog Modules

- **`dialogs::alert_window`** - Message boxes and alerts
- **`dialogs::file_chooser`** - File open/save dialogs

### Utility Modules

- **`message_thread`** - Message thread utilities for safe cross-thread UI updates
- **`lookandfeel`** - LookAndFeel system for customizing component appearance
- **`parameter_attachment`** - Slider parameter attachment for audio parameters

## Documentation Standards

All public APIs in this crate follow these documentation standards:

### Module-Level Documentation

Each module includes:
- **Overview**: What the module provides
- **Thread Safety**: Thread safety requirements and enforcement
- **Examples**: Basic usage examples
- **Related Modules**: Links to related functionality

### Type Documentation

Each public type includes:
- **Purpose**: What the type represents
- **Thread Safety**: Whether it implements Send/Sync and why
- **Memory Management**: How the type manages resources
- **Examples**: Basic usage examples
- **See Also**: Links to related types

### Method Documentation

Each public method includes:
- **Purpose**: What the method does
- **Arguments**: Description of each parameter
- **Returns**: What the method returns
- **Thread Safety**: Thread requirements (all GUI methods require message thread)
- **Examples**: Usage examples where helpful
- **Errors**: Possible error conditions (for Result-returning methods)

### Performance Documentation

Where relevant, documentation includes:
- **FFI Overhead**: Approximate overhead for FFI calls
- **Allocation**: Whether the operation allocates memory
- **Complexity**: Time/space complexity for non-trivial operations

## Thread Safety Documentation

Thread safety is a critical aspect of this crate. All documentation clearly states:

1. **Type-level thread safety**: Whether a type implements Send/Sync
2. **Method-level requirements**: Which thread methods must be called from
3. **Cross-thread patterns**: How to safely update UI from other threads

Example:
```rust
/// Set the slider value.
///
/// # Thread Safety
///
/// This function must be called on the JUCE message thread.
/// To update from another thread, use `MessageManager::call_async()`.
pub fn set_value(&mut self, value: f64) { ... }
```

## FFI Overhead Documentation

Performance-sensitive operations document their FFI overhead:

```rust
/// Create a new Component.
///
/// # Performance
///
/// - **FFI overhead**: ~5-10 nanoseconds
/// - **Total time**: ~10-50 microseconds (dominated by JUCE allocation)
/// - **Allocations**: 1 C++ allocation, 1 Rust allocation
pub fn new() -> Result<Self> { ... }
```

## Error Documentation

All fallible operations document possible errors:

```rust
/// Load an image from a file.
///
/// # Errors
///
/// Returns an error if:
/// - The file doesn't exist ([`JuceError::FileError`])
/// - The file format is not supported ([`JuceError::UnsupportedImageFormat`])
/// - The file is corrupted ([`JuceError::CppException`])
pub fn load_from_file(path: &Path) -> Result<Self> { ... }
```

## Example Documentation

Examples are provided at multiple levels:

### Module-Level Examples

Show common use cases for the module:

```rust
//! # Examples
//!
//! ```ignore
//! use nih_plug_juce::widgets::TextButton;
//!
//! let mut button = TextButton::new("Click Me")?;
//! button.set_bounds(10, 10, 100, 30);
//! button.set_on_click(|| {
//!     println!("Button clicked!");
//! })?;
//! ```
```

### Type-Level Examples

Show how to create and use the type:

```rust
/// # Examples
///
/// ```ignore
/// let mut slider = Slider::new(SliderStyle::Linear)?;
/// slider.set_range(0.0, 1.0, 0.01);
/// slider.set_value(0.5);
/// ```
```

### Method-Level Examples

Show specific method usage:

```rust
/// # Examples
///
/// ```ignore
/// button.set_on_click(|| {
///     println!("Button clicked!");
/// })?;
/// ```
```

## Viewing Documentation

To view the complete API documentation:

```bash
cargo doc --open -p nih_plug_juce
```

This will build and open the documentation in your default web browser.

## Contributing to Documentation

When adding new features or modifying existing ones:

1. **Add rustdoc comments** to all public items
2. **Include examples** for non-trivial functionality
3. **Document thread safety** requirements
4. **Document errors** for Result-returning methods
5. **Document performance** characteristics where relevant
6. **Link to related items** using `[`Type`]` syntax
7. **Test examples** to ensure they compile (use `ignore` for examples that need a full JUCE application)

### Documentation Checklist

Before submitting a PR with new public APIs:

- [ ] Module-level documentation with overview and examples
- [ ] Type documentation with purpose and thread safety
- [ ] Method documentation with arguments, returns, and examples
- [ ] Thread safety requirements clearly stated
- [ ] Error conditions documented for Result-returning methods
- [ ] Performance characteristics noted where relevant
- [ ] Examples provided and tested
- [ ] Links to related types and methods

## Common Documentation Patterns

### Thread Safety Pattern

```rust
/// # Thread Safety
///
/// This function must be called on the JUCE message thread.
/// All GUI operations in JUCE must occur on the message thread.
///
/// To update UI from another thread (e.g., audio processing thread):
///
/// ```ignore
/// MessageManager::call_async(move || {
///     // Safe to call GUI methods here
///     component.set_visible(true);
/// });
/// ```
```

### Error Handling Pattern

```rust
/// # Errors
///
/// Returns an error if:
/// - The component pointer is null ([`JuceError::NullPointer`])
/// - A C++ exception occurs ([`JuceError::CppException`])
/// - The operation is called from the wrong thread ([`JuceError::ThreadSafetyViolation`])
```

### Performance Pattern

```rust
/// # Performance
///
/// - **FFI overhead**: ~10-20 nanoseconds
/// - **Allocations**: None (modifies existing component)
/// - **Complexity**: O(1)
```

## Additional Resources

- [JUCE Documentation](https://docs.juce.com/) - Official JUCE C++ documentation
- [cxx Documentation](https://cxx.rs/) - Documentation for the cxx FFI crate
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) - Rust API design guidelines

## Questions?

If you have questions about the API or documentation:

1. Check the API documentation: `cargo doc --open -p nih_plug_juce`
2. Look at examples in `plugins/examples/`
3. Read the design document in `.kiro/specs/juce-ffi-integration/design.md`
4. Open an issue on GitHub
