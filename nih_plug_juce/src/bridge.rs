//! FFI bridge definitions using cxx.
//!
//! This module contains the cxx bridge that defines the interface between
//! Rust and C++ JUCE code. The bridge uses opaque pointers for C++ objects
//! and provides safe wrapper functions.
//!
//! # Architecture
//!
//! The bridge uses the `cxx` crate to provide safe FFI between Rust and C++.
//! C++ JUCE objects are represented as opaque types in Rust, with their
//! internal structure hidden. All operations on these objects go through
//! FFI functions defined in this bridge.
//!
//! # Exception Handling
//!
//! All C++ exceptions are caught at the FFI boundary and converted to
//! Rust Result types. This ensures that C++ exceptions never propagate
//! into Rust code, which would cause undefined behavior.
//!
//! # Thread Safety
//!
//! JUCE requires all GUI operations to be performed on the message thread.
//! This is enforced through the type system - GUI types don't implement
//! Send or Sync, preventing them from being moved or shared across threads.

/// Callback bridge structure for paint callbacks.
/// This structure holds a Rust closure and function pointers to invoke and drop it.
/// It must be repr(C) to match the C++ struct layout.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PaintCallbackBridge {
    pub rust_closure: *mut std::ffi::c_void,
    pub invoke: Option<unsafe extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void)>,
    pub drop: Option<unsafe extern "C" fn(*mut std::ffi::c_void)>,
}

#[cxx::bridge(namespace = "nih_plug_juce")]
pub mod ffi {
    // Opaque C++ types
    // These types represent C++ JUCE objects whose internal structure
    // is hidden from Rust. We only work with pointers to these types.
    
    unsafe extern "C++" {
        include!("nih_plug_juce/cpp/juce_bridge.h");
        
        // Core opaque types for JUCE objects
        
        /// Opaque pointer to a JUCE Component object.
        /// 
        /// This represents a juce::Component in C++, which is the base class
        /// for all GUI elements in JUCE. The actual C++ object is managed
        /// by the Rust wrapper through RAII.
        type JuceComponent;
        
        /// Opaque pointer to a JUCE Graphics context.
        /// 
        /// This represents a juce::Graphics object in C++, which is used
        /// for all drawing operations. Graphics contexts are typically
        /// provided during paint callbacks and should not be stored.
        type JuceGraphics;
        
        /// Opaque pointer to a JUCE Colour object.
        /// 
        /// This represents a juce::Colour in C++, which stores RGBA color
        /// values and provides color manipulation methods.
        type JuceColour;
        
        /// Opaque pointer to a JUCE Font object.
        /// 
        /// This represents a juce::Font in C++, which describes text
        /// rendering properties like typeface, size, and style.
        type JuceFont;
        
        /// Opaque pointer to a JUCE Image object.
        /// 
        /// This represents a juce::Image in C++, which stores pixel data
        /// and provides image manipulation methods.
        type JuceImage;
        
        /// Opaque pointer to a JUCE Path object.
        /// 
        /// This represents a juce::Path in C++, which describes vector
        /// graphics paths for drawing complex shapes.
        type JucePath;
        
        /// Opaque pointer to a JUCE AffineTransform object.
        /// 
        /// This represents a juce::AffineTransform in C++, which describes
        /// 2D transformations (translation, rotation, scaling).
        type JuceAffineTransform;
        
        /// Opaque pointer to a JUCE FlexBox object.
        /// 
        /// This represents a juce::FlexBox in C++, which provides flexible
        /// box layout for arranging components.
        type JuceFlexBox;
        
        /// Opaque pointer to a JUCE Timer object.
        /// 
        /// This represents a juce::Timer in C++, which provides periodic
        /// callbacks on the message thread.
        type JuceTimer;
        
        /// Opaque pointer to a JUCE LookAndFeel object.
        /// 
        /// This represents a juce::LookAndFeel in C++, which defines the
        /// visual appearance of components.
        type JuceLookAndFeel;
        
        /// Opaque pointer to a JUCE FileChooser object.
        /// 
        /// This represents a juce::FileChooser in C++, which provides
        /// native file open/save dialogs.
        type JuceFileChooser;
        
        // Initialization and utility functions
        
        /// Initialize the JUCE FFI bridge.
        /// 
        /// This function verifies that JUCE is properly linked and initialized.
        /// It should be called once at startup before using any JUCE functionality.
        /// 
        /// # Returns
        /// 
        /// Returns `true` if initialization succeeded, `false` otherwise.
        /// 
        /// # Safety
        /// 
        /// This function is safe to call multiple times, but should be called
        /// at least once before using any other JUCE functionality.
        fn initialize() -> bool;
        
        // Exception handling helper
        // This will be used by wrapper functions to convert C++ exceptions
        // to Rust Result types. The actual implementation will catch exceptions
        // and populate an error buffer.
        
        // Component operations
        
        /// Create a new JUCE Component.
        /// 
        /// # Arguments
        /// 
        /// * `error_buffer` - Buffer to store error message if creation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns a pointer to the created component, or null on error.
        /// If null is returned, the error buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// The returned pointer must be freed with `delete_component` when no longer needed.
        unsafe fn create_component(error_buffer: *mut i8, buffer_size: usize) -> *mut JuceComponent;
        
        /// Create a new JUCE Component that supports paint callbacks.
        /// 
        /// # Arguments
        /// 
        /// * `error_buffer` - Buffer to store error message if creation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns a pointer to the created component, or null on error.
        /// If null is returned, the error buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// The returned pointer must be freed with `delete_component` when no longer needed.
        unsafe fn create_component_with_paint_callback(error_buffer: *mut i8, buffer_size: usize) -> *mut JuceComponent;
        
        /// Delete a JUCE Component and free its resources.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the component to delete
        /// 
        /// # Safety
        /// 
        /// The pointer must have been created by `create_component` and must not
        /// be used after this call.
        unsafe fn delete_component(ptr: *mut JuceComponent);
        
        /// Add a child component to a parent component.
        /// 
        /// # Arguments
        /// 
        /// * `parent` - Pointer to the parent component
        /// * `child` - Pointer to the child component to add
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn component_add_child(parent: *mut JuceComponent, child: *mut JuceComponent,
                              error_buffer: *mut i8, buffer_size: usize) -> i32;
        
        /// Remove a child component from a parent component.
        /// 
        /// # Arguments
        /// 
        /// * `parent` - Pointer to the parent component
        /// * `child` - Pointer to the child component to remove
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn component_remove_child(parent: *mut JuceComponent, child: *mut JuceComponent,
                                 error_buffer: *mut i8, buffer_size: usize) -> i32;
        
        /// Set the bounds (position and size) of a component.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the component
        /// * `x` - X coordinate of the top-left corner
        /// * `y` - Y coordinate of the top-left corner
        /// * `width` - Width of the component
        /// * `height` - Height of the component
        unsafe fn component_set_bounds(ptr: *mut JuceComponent, x: i32, y: i32, 
                               width: i32, height: i32);
        
        /// Set whether a component is visible.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the component
        /// * `visible` - true to make visible, false to hide
        unsafe fn component_set_visible(ptr: *mut JuceComponent, visible: bool);
        
        /// Trigger a repaint of a component.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the component
        unsafe fn component_repaint(ptr: *mut JuceComponent);
        
        // Paint callback support
        
        /// Set a paint callback for a component.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the component
        /// * `rust_closure` - Pointer to the Rust closure (as usize)
        /// * `invoke` - Function pointer to invoke the closure (as usize)
        /// * `drop_fn` - Function pointer to drop the closure (as usize)
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn component_set_paint_callback(
            ptr: *mut JuceComponent,
            rust_closure: usize,
            invoke: usize,
            drop_fn: usize,
            error_buffer: *mut i8,
            buffer_size: usize
        ) -> i32;
        
        // Graphics operations
        
        /// Fill a rectangle with the current color.
        /// 
        /// # Arguments
        /// 
        /// * `g` - Pointer to the Graphics context
        /// * `x` - X coordinate of the top-left corner
        /// * `y` - Y coordinate of the top-left corner
        /// * `width` - Width of the rectangle
        /// * `height` - Height of the rectangle
        unsafe fn graphics_fill_rect(g: *mut JuceGraphics, x: i32, y: i32, 
                                     width: i32, height: i32);
        
        /// Draw a rectangle outline with the current color.
        /// 
        /// # Arguments
        /// 
        /// * `g` - Pointer to the Graphics context
        /// * `x` - X coordinate of the top-left corner
        /// * `y` - Y coordinate of the top-left corner
        /// * `width` - Width of the rectangle
        /// * `height` - Height of the rectangle
        unsafe fn graphics_draw_rect(g: *mut JuceGraphics, x: i32, y: i32, 
                                     width: i32, height: i32);
        
        /// Fill an ellipse with the current color.
        /// 
        /// # Arguments
        /// 
        /// * `g` - Pointer to the Graphics context
        /// * `x` - X coordinate of the bounding rectangle
        /// * `y` - Y coordinate of the bounding rectangle
        /// * `width` - Width of the bounding rectangle
        /// * `height` - Height of the bounding rectangle
        unsafe fn graphics_fill_ellipse(g: *mut JuceGraphics, x: f32, y: f32, 
                                        width: f32, height: f32);
        
        /// Draw a line with the current color.
        /// 
        /// # Arguments
        /// 
        /// * `g` - Pointer to the Graphics context
        /// * `x1` - X coordinate of the start point
        /// * `y1` - Y coordinate of the start point
        /// * `x2` - X coordinate of the end point
        /// * `y2` - Y coordinate of the end point
        unsafe fn graphics_draw_line(g: *mut JuceGraphics, x1: f32, y1: f32, 
                                     x2: f32, y2: f32);
        
        /// Set the current drawing color.
        /// 
        /// # Arguments
        /// 
        /// * `g` - Pointer to the Graphics context
        /// * `colour` - Pointer to the color to use
        unsafe fn graphics_set_colour(g: *mut JuceGraphics, colour: *const JuceColour);
        
        /// Draw text within a rectangle.
        /// 
        /// # Arguments
        /// 
        /// * `g` - Pointer to the Graphics context
        /// * `text` - The text to draw (C string pointer)
        /// * `text_len` - Length of the text string
        /// * `x` - X coordinate of the text rectangle
        /// * `y` - Y coordinate of the text rectangle
        /// * `width` - Width of the text rectangle
        /// * `height` - Height of the text rectangle
        /// * `justification` - Text justification flags
        unsafe fn graphics_draw_text(g: *mut JuceGraphics, text: *const u8, text_len: usize,
                                     x: i32, y: i32, width: i32, height: i32,
                                     justification: i32);
        
        /// Draw an image at the specified position.
        /// 
        /// # Arguments
        /// 
        /// * `g` - Pointer to the Graphics context
        /// * `image` - Pointer to the image to draw
        /// * `x` - X coordinate where the image should be drawn
        /// * `y` - Y coordinate where the image should be drawn
        unsafe fn graphics_draw_image_at(g: *mut JuceGraphics, image: *const JuceImage,
                                        x: i32, y: i32);
        
        /// Stroke (outline) a path with the current color.
        /// 
        /// # Arguments
        /// 
        /// * `g` - Pointer to the Graphics context
        /// * `path` - Pointer to the path to stroke
        unsafe fn graphics_stroke_path(g: *mut JuceGraphics, path: *const JucePath);
        
        /// Fill a path with the current color.
        /// 
        /// # Arguments
        /// 
        /// * `g` - Pointer to the Graphics context
        /// * `path` - Pointer to the path to fill
        unsafe fn graphics_fill_path(g: *mut JuceGraphics, path: *const JucePath);
        
        // Slider operations
        
        /// Create a new JUCE Slider with the specified style.
        /// 
        /// # Arguments
        /// 
        /// * `style` - The slider style (1=LinearHorizontal, 2=LinearVertical, etc.)
        /// * `error_buffer` - Buffer to store error message if creation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns a pointer to the created slider, or null on error.
        /// If null is returned, the error buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// The returned pointer must be freed with `delete_component` when no longer needed.
        unsafe fn create_slider(style: i32, error_buffer: *mut i8, buffer_size: usize) -> *mut JuceComponent;
        
        /// Set the range of a slider.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the slider component
        /// * `min` - Minimum value
        /// * `max` - Maximum value
        /// * `interval` - Snapping interval (0 for continuous)
        unsafe fn slider_set_range(ptr: *mut JuceComponent, min: f64, max: f64, interval: f64);
        
        /// Set the value of a slider.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the slider component
        /// * `value` - The new value
        unsafe fn slider_set_value(ptr: *mut JuceComponent, value: f64);
        
        /// Get the current value of a slider.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the slider component
        /// 
        /// # Returns
        /// 
        /// Returns the current slider value.
        unsafe fn slider_get_value(ptr: *const JuceComponent) -> f64;
        
        /// Set a value change callback for a slider.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the slider component
        /// * `rust_closure` - Pointer to the Rust closure (as usize)
        /// * `invoke` - Function pointer to invoke the closure (as usize)
        /// * `drop_fn` - Function pointer to drop the closure (as usize)
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn slider_set_on_value_change(
            ptr: *mut JuceComponent,
            rust_closure: usize,
            invoke: usize,
            drop_fn: usize,
            error_buffer: *mut i8,
            buffer_size: usize
        ) -> i32;
        
        // Button operations
        
        /// Create a new JUCE TextButton with the specified text.
        /// 
        /// # Arguments
        /// 
        /// * `text` - The button text (UTF-8 bytes)
        /// * `text_len` - Length of the text string
        /// * `error_buffer` - Buffer to store error message if creation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns a pointer to the created button, or null on error.
        /// If null is returned, the error buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// The returned pointer must be freed with `delete_component` when no longer needed.
        unsafe fn create_text_button(text: *const u8, text_len: usize, 
                                     error_buffer: *mut i8, buffer_size: usize) -> *mut JuceComponent;
        
        /// Set the text of a button.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the button component
        /// * `text` - The new button text (UTF-8 bytes)
        /// * `text_len` - Length of the text string
        unsafe fn button_set_text(ptr: *mut JuceComponent, text: *const u8, text_len: usize);
        
        /// Set whether a button is enabled.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the button component
        /// * `enabled` - true to enable, false to disable
        unsafe fn button_set_enabled(ptr: *mut JuceComponent, enabled: bool);
        
        /// Set a color for a button.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the button component
        /// * `colour_id` - The color ID to set
        /// * `r` - Red component (0-255)
        /// * `g` - Green component (0-255)
        /// * `b` - Blue component (0-255)
        /// * `a` - Alpha component (0-255)
        unsafe fn button_set_colour(ptr: *mut JuceComponent, colour_id: i32, 
                                    r: u8, g: u8, b: u8, a: u8);
        
        /// Set a click callback for a button.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the button component
        /// * `rust_closure` - Pointer to the Rust closure (as usize)
        /// * `invoke` - Function pointer to invoke the closure (as usize)
        /// * `drop_fn` - Function pointer to drop the closure (as usize)
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn button_set_on_click(
            ptr: *mut JuceComponent,
            rust_closure: usize,
            invoke: usize,
            drop_fn: usize,
            error_buffer: *mut i8,
            buffer_size: usize
        ) -> i32;
        
        // Label operations
        
        /// Create a new JUCE Label with the specified text.
        /// 
        /// # Arguments
        /// 
        /// * `text` - The label text (UTF-8 bytes)
        /// * `text_len` - Length of the text string
        /// * `error_buffer` - Buffer to store error message if creation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns a pointer to the created label, or null on error.
        /// If null is returned, the error buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// The returned pointer must be freed with `delete_component` when no longer needed.
        unsafe fn create_label(text: *const u8, text_len: usize, 
                              error_buffer: *mut i8, buffer_size: usize) -> *mut JuceComponent;
        
        /// Set the text of a label.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the label component
        /// * `text` - The new label text (UTF-8 bytes)
        /// * `text_len` - Length of the text string
        unsafe fn label_set_text(ptr: *mut JuceComponent, text: *const u8, text_len: usize);
        
        /// Set the font of a label.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the label component
        /// * `font_size` - The font size in points
        unsafe fn label_set_font(ptr: *mut JuceComponent, font_size: f32);
        
        /// Set the text justification of a label.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the label component
        /// * `justification` - Justification flags (JUCE Justification constants)
        unsafe fn label_set_justification(ptr: *mut JuceComponent, justification: i32);
        
        /// Set whether a label is editable.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the label component
        /// * `editable` - true to make editable, false to make read-only
        unsafe fn label_set_editable(ptr: *mut JuceComponent, editable: bool);
        
        /// Set a text change callback for a label.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the label component
        /// * `rust_closure` - Pointer to the Rust closure (as usize)
        /// * `invoke` - Function pointer to invoke the closure (as usize)
        /// * `drop_fn` - Function pointer to drop the closure (as usize)
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn label_set_on_text_change(
            ptr: *mut JuceComponent,
            rust_closure: usize,
            invoke: usize,
            drop_fn: usize,
            error_buffer: *mut i8,
            buffer_size: usize
        ) -> i32;
        
        // ComboBox operations
        
        /// Create a new JUCE ComboBox.
        /// 
        /// # Arguments
        /// 
        /// * `error_buffer` - Buffer to store error message if creation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns a pointer to the created combo box, or null on error.
        /// If null is returned, the error buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// The returned pointer must be freed with `delete_component` when no longer needed.
        unsafe fn create_combo_box(error_buffer: *mut i8, buffer_size: usize) -> *mut JuceComponent;
        
        /// Add an item to a combo box.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the combo box component
        /// * `text` - The item text (UTF-8 bytes)
        /// * `text_len` - Length of the text string
        /// * `item_id` - The ID for this item (must be > 0)
        unsafe fn combo_box_add_item(ptr: *mut JuceComponent, text: *const u8, text_len: usize, item_id: i32);
        
        /// Clear all items from a combo box.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the combo box component
        unsafe fn combo_box_clear(ptr: *mut JuceComponent);
        
        /// Set the selected item by ID.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the combo box component
        /// * `item_id` - The ID of the item to select
        unsafe fn combo_box_set_selected_id(ptr: *mut JuceComponent, item_id: i32);
        
        /// Set the selected item by index.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the combo box component
        /// * `index` - The index of the item to select (0-based)
        unsafe fn combo_box_set_selected_index(ptr: *mut JuceComponent, index: i32);
        
        /// Get the ID of the currently selected item.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the combo box component
        /// 
        /// # Returns
        /// 
        /// Returns the ID of the selected item, or 0 if no item is selected.
        unsafe fn combo_box_get_selected_id(ptr: *const JuceComponent) -> i32;
        
        /// Set a change callback for a combo box.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the combo box component
        /// * `rust_closure` - Pointer to the Rust closure (as usize)
        /// * `invoke` - Function pointer to invoke the closure (as usize)
        /// * `drop_fn` - Function pointer to drop the closure (as usize)
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn combo_box_set_on_change(
            ptr: *mut JuceComponent,
            rust_closure: usize,
            invoke: usize,
            drop_fn: usize,
            error_buffer: *mut i8,
            buffer_size: usize
        ) -> i32;
        
        // TextEditor operations
        
        /// Create a new JUCE TextEditor.
        /// 
        /// # Arguments
        /// 
        /// * `error_buffer` - Buffer to store error message if creation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns a pointer to the created text editor, or null on error.
        /// If null is returned, the error buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// The returned pointer must be freed with `delete_component` when no longer needed.
        unsafe fn create_text_editor(error_buffer: *mut i8, buffer_size: usize) -> *mut JuceComponent;
        
        /// Set the text of a text editor.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the text editor component
        /// * `text` - The new text (UTF-8 bytes)
        /// * `text_len` - Length of the text string
        unsafe fn text_editor_set_text(ptr: *mut JuceComponent, text: *const u8, text_len: usize);
        
        /// Get the text from a text editor.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the text editor component
        /// * `buffer` - Buffer to store the text (UTF-8 bytes)
        /// * `buffer_size` - Size of the buffer
        /// 
        /// # Returns
        /// 
        /// Returns the number of bytes written to the buffer (excluding null terminator).
        unsafe fn text_editor_get_text(ptr: *const JuceComponent, buffer: *mut u8, buffer_size: usize) -> usize;
        
        /// Set whether a text editor is multiline.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the text editor component
        /// * `multiline` - true for multiline, false for single line
        unsafe fn text_editor_set_multiline(ptr: *mut JuceComponent, multiline: bool);
        
        /// Set whether a text editor is read-only.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the text editor component
        /// * `readonly` - true for read-only, false for editable
        unsafe fn text_editor_set_readonly(ptr: *mut JuceComponent, readonly: bool);
        
        /// Set a text change callback for a text editor.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the text editor component
        /// * `rust_closure` - Pointer to the Rust closure (as usize)
        /// * `invoke` - Function pointer to invoke the closure (as usize)
        /// * `drop_fn` - Function pointer to drop the closure (as usize)
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn text_editor_set_on_text_change(
            ptr: *mut JuceComponent,
            rust_closure: usize,
            invoke: usize,
            drop_fn: usize,
            error_buffer: *mut i8,
            buffer_size: usize
        ) -> i32;
        
        // ToggleButton operations
        
        /// Create a new JUCE ToggleButton with the specified text.
        /// 
        /// # Arguments
        /// 
        /// * `text` - The button text (UTF-8 bytes)
        /// * `text_len` - Length of the text string
        /// * `error_buffer` - Buffer to store error message if creation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns a pointer to the created toggle button, or null on error.
        /// If null is returned, the error buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// The returned pointer must be freed with `delete_component` when no longer needed.
        unsafe fn create_toggle_button(text: *const u8, text_len: usize, 
                                       error_buffer: *mut i8, buffer_size: usize) -> *mut JuceComponent;
        
        /// Set the toggle state of a toggle button.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the toggle button component
        /// * `state` - true for on/checked, false for off/unchecked
        unsafe fn toggle_button_set_toggle_state(ptr: *mut JuceComponent, state: bool);
        
        /// Get the current toggle state of a toggle button.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the toggle button component
        /// 
        /// # Returns
        /// 
        /// Returns true if the button is on/checked, false if off/unchecked.
        unsafe fn toggle_button_get_toggle_state(ptr: *const JuceComponent) -> bool;
        
        /// Set the radio group ID for a toggle button.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the toggle button component
        /// * `group_id` - The radio group ID (0 to remove from groups)
        unsafe fn toggle_button_set_radio_group_id(ptr: *mut JuceComponent, group_id: i32);
        
        /// Set the text of a toggle button.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the toggle button component
        /// * `text` - The new button text (UTF-8 bytes)
        /// * `text_len` - Length of the text string
        unsafe fn toggle_button_set_text(ptr: *mut JuceComponent, text: *const u8, text_len: usize);
        
        /// Set a click callback for a toggle button.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the toggle button component
        /// * `rust_closure` - Pointer to the Rust closure (as usize)
        /// * `invoke` - Function pointer to invoke the closure (as usize)
        /// * `drop_fn` - Function pointer to drop the closure (as usize)
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn toggle_button_set_on_click(
            ptr: *mut JuceComponent,
            rust_closure: usize,
            invoke: usize,
            drop_fn: usize,
            error_buffer: *mut i8,
            buffer_size: usize
        ) -> i32;
        
        // Mouse event handling
        
        /// Set a mouse listener for a component.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the component
        /// * `listener_ptr` - Pointer to the Rust listener (as usize)
        /// * `mouse_down` - Function pointer for mouse down events (as usize)
        /// * `mouse_drag` - Function pointer for mouse drag events (as usize)
        /// * `mouse_up` - Function pointer for mouse up events (as usize)
        /// * `mouse_enter` - Function pointer for mouse enter events (as usize)
        /// * `mouse_exit` - Function pointer for mouse exit events (as usize)
        /// * `drop_fn` - Function pointer to drop the listener (as usize)
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn component_set_mouse_listener(
            ptr: *mut JuceComponent,
            listener_ptr: usize,
            mouse_down: usize,
            mouse_drag: usize,
            mouse_up: usize,
            mouse_enter: usize,
            mouse_exit: usize,
            drop_fn: usize,
            error_buffer: *mut i8,
            buffer_size: usize
        ) -> i32;
        
        // Keyboard event handling
        
        /// Set whether a component wants keyboard focus.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the component
        /// * `wants` - true to enable keyboard focus, false to disable
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn component_set_wants_keyboard_focus(
            ptr: *mut JuceComponent,
            wants: bool,
            error_buffer: *mut i8,
            buffer_size: usize
        ) -> i32;
        
        /// Set a keyboard listener for a component.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the component
        /// * `listener_ptr` - Pointer to the Rust listener (as usize)
        /// * `key_pressed` - Function pointer for key pressed events (as usize)
        /// * `key_state_changed` - Function pointer for key state changed events (as usize)
        /// * `focus_gained` - Function pointer for focus gained events (as usize)
        /// * `focus_lost` - Function pointer for focus lost events (as usize)
        /// * `drop_fn` - Function pointer to drop the listener (as usize)
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn component_set_key_listener(
            ptr: *mut JuceComponent,
            listener_ptr: usize,
            key_pressed: usize,
            key_state_changed: usize,
            focus_gained: usize,
            focus_lost: usize,
            drop_fn: usize,
            error_buffer: *mut i8,
            buffer_size: usize
        ) -> i32;
        
        // Timer operations
        
        /// Create a new JUCE Timer with a callback.
        /// 
        /// # Arguments
        /// 
        /// * `rust_closure` - Pointer to the Rust closure (as usize)
        /// * `invoke` - Function pointer to invoke the closure (as usize)
        /// * `drop_fn` - Function pointer to drop the closure (as usize)
        /// * `error_buffer` - Buffer to store error message if creation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns a pointer to the created timer, or null on error.
        /// If null is returned, the error buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// The returned pointer must be freed with `delete_timer` when no longer needed.
        unsafe fn create_timer(
            rust_closure: usize,
            invoke: usize,
            drop_fn: usize,
            error_buffer: *mut i8,
            buffer_size: usize
        ) -> *mut JuceTimer;
        
        /// Delete a JUCE Timer and free its resources.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the timer to delete
        /// 
        /// # Safety
        /// 
        /// The pointer must have been created by `create_timer` and must not
        /// be used after this call.
        unsafe fn delete_timer(ptr: *mut JuceTimer);
        
        /// Start a timer with the specified interval.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the timer
        /// * `interval_ms` - The interval in milliseconds
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn timer_start(
            ptr: *mut JuceTimer,
            interval_ms: i32,
            error_buffer: *mut i8,
            buffer_size: usize
        ) -> i32;
        
        /// Stop a timer.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the timer
        unsafe fn timer_stop(ptr: *mut JuceTimer);
        
        /// Check if a timer is currently running.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the timer
        /// 
        /// # Returns
        /// 
        /// Returns true if the timer is running, false otherwise.
        unsafe fn timer_is_running(ptr: *const JuceTimer) -> bool;
        
        // Colour operations
        
        /// Create a new JUCE Colour from RGBA values.
        /// 
        /// # Arguments
        /// 
        /// * `r` - Red component (0-255)
        /// * `g` - Green component (0-255)
        /// * `b` - Blue component (0-255)
        /// * `a` - Alpha component (0-255)
        /// * `error_buffer` - Buffer to store error message if creation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns a pointer to the created colour, or null on error.
        /// If null is returned, the error buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// The returned pointer must be freed with `delete_colour` when no longer needed.
        unsafe fn create_colour_rgba(r: u8, g: u8, b: u8, a: u8,
                                     error_buffer: *mut i8, buffer_size: usize) -> *mut JuceColour;
        
        /// Create a new JUCE Colour from a hexadecimal string.
        /// 
        /// # Arguments
        /// 
        /// * `hex` - Hexadecimal color string (UTF-8 bytes)
        /// * `hex_len` - Length of the hex string
        /// * `error_buffer` - Buffer to store error message if creation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns a pointer to the created colour, or null on error.
        /// If null is returned, the error buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// The returned pointer must be freed with `delete_colour` when no longer needed.
        unsafe fn create_colour_from_hex(hex: *const u8, hex_len: usize,
                                         error_buffer: *mut i8, buffer_size: usize) -> *mut JuceColour;
        
        /// Delete a JUCE Colour and free its resources.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the colour to delete
        /// 
        /// # Safety
        /// 
        /// The pointer must have been created by a colour creation function and must not
        /// be used after this call.
        unsafe fn delete_colour(ptr: *mut JuceColour);
        
        /// Convert a colour to a hexadecimal string.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the colour
        /// * `buffer` - Buffer to store the hex string (UTF-8 bytes)
        /// * `buffer_size` - Size of the buffer
        /// 
        /// # Returns
        /// 
        /// Returns the number of bytes written to the buffer (excluding null terminator).
        unsafe fn colour_to_hex(ptr: *const JuceColour, buffer: *mut u8, buffer_size: usize) -> usize;
        
        /// Create a new colour with a different alpha value.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the colour
        /// * `alpha` - New alpha value (0.0 = transparent, 1.0 = opaque)
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns a pointer to the new colour, or null on error.
        /// If null is returned, the error buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// The returned pointer must be freed with `delete_colour` when no longer needed.
        unsafe fn colour_with_alpha(ptr: *const JuceColour, alpha: f32,
                                    error_buffer: *mut i8, buffer_size: usize) -> *mut JuceColour;
        
        /// Create a brighter version of a colour.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the colour
        /// * `amount` - Amount to brighten (0.0 = no change, 1.0 = maximum)
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns a pointer to the new colour, or null on error.
        /// If null is returned, the error buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// The returned pointer must be freed with `delete_colour` when no longer needed.
        unsafe fn colour_brighter(ptr: *const JuceColour, amount: f32,
                                  error_buffer: *mut i8, buffer_size: usize) -> *mut JuceColour;
        
        /// Create a darker version of a colour.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the colour
        /// * `amount` - Amount to darken (0.0 = no change, 1.0 = maximum)
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns a pointer to the new colour, or null on error.
        /// If null is returned, the error buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// The returned pointer must be freed with `delete_colour` when no longer needed.
        unsafe fn colour_darker(ptr: *const JuceColour, amount: f32,
                                error_buffer: *mut i8, buffer_size: usize) -> *mut JuceColour;
        
        /// Create a colour interpolated between two colours.
        /// 
        /// # Arguments
        /// 
        /// * `ptr1` - Pointer to the first colour
        /// * `ptr2` - Pointer to the second colour
        /// * `proportion` - Interpolation amount (0.0 = first colour, 1.0 = second colour)
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns a pointer to the new colour, or null on error.
        /// If null is returned, the error buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// The returned pointer must be freed with `delete_colour` when no longer needed.
        unsafe fn colour_interpolated_with(ptr1: *const JuceColour, ptr2: *const JuceColour,
                                           proportion: f32, error_buffer: *mut i8,
                                           buffer_size: usize) -> *mut JuceColour;
        
        // Font operations
        
        /// Create a new JUCE Font with the specified size.
        /// 
        /// # Arguments
        /// 
        /// * `size` - Font size in points
        /// * `error_buffer` - Buffer to store error message if creation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns a pointer to the created font, or null on error.
        /// If null is returned, the error buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// The returned pointer must be freed with `delete_font` when no longer needed.
        unsafe fn create_font(size: f32, error_buffer: *mut i8, buffer_size: usize) -> *mut JuceFont;
        
        /// Create a new JUCE Font with a specific typeface and size.
        /// 
        /// # Arguments
        /// 
        /// * `typeface` - Typeface name (UTF-8 bytes)
        /// * `typeface_len` - Length of the typeface name
        /// * `size` - Font size in points
        /// * `error_buffer` - Buffer to store error message if creation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns a pointer to the created font, or null on error.
        /// If null is returned, the error buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// The returned pointer must be freed with `delete_font` when no longer needed.
        unsafe fn create_font_with_typeface(typeface: *const u8, typeface_len: usize, size: f32,
                                            error_buffer: *mut i8, buffer_size: usize) -> *mut JuceFont;
        
        /// Delete a JUCE Font and free its resources.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the font to delete
        /// 
        /// # Safety
        /// 
        /// The pointer must have been created by a font creation function and must not
        /// be used after this call.
        unsafe fn delete_font(ptr: *mut JuceFont);
        
        /// Set whether the font is bold.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the font
        /// * `bold` - true for bold, false for normal weight
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn font_set_bold(ptr: *mut JuceFont, bold: bool,
                                error_buffer: *mut i8, buffer_size: usize) -> i32;
        
        /// Set whether the font is italic.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the font
        /// * `italic` - true for italic, false for normal style
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn font_set_italic(ptr: *mut JuceFont, italic: bool,
                                  error_buffer: *mut i8, buffer_size: usize) -> i32;
        
        /// Set whether the font is underlined.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the font
        /// * `underline` - true for underlined, false for no underline
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn font_set_underline(ptr: *mut JuceFont, underline: bool,
                                     error_buffer: *mut i8, buffer_size: usize) -> i32;
        
        /// Get the width of a string when rendered with this font.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the font
        /// * `text` - The text to measure (UTF-8 bytes)
        /// * `text_len` - Length of the text string
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns the width in pixels, or -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn font_get_string_width(ptr: *const JuceFont, text: *const u8, text_len: usize,
                                        error_buffer: *mut i8, buffer_size: usize) -> i32;
        
        /// Get the height of the font.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the font
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns the height in pixels, or -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn font_get_height(ptr: *const JuceFont, error_buffer: *mut i8,
                                  buffer_size: usize) -> i32;
        
        /// Get the count of available typefaces on the system.
        /// 
        /// # Returns
        /// 
        /// Returns the number of available typefaces, or -1 on error.
        unsafe fn font_get_typeface_count() -> i32;
        
        /// Get the name of a typeface by index.
        /// 
        /// # Arguments
        /// 
        /// * `index` - Index of the typeface (0-based)
        /// * `buffer` - Buffer to store the typeface name (UTF-8 bytes)
        /// * `buffer_size` - Size of the buffer
        /// 
        /// # Returns
        /// 
        /// Returns the number of bytes written to the buffer (excluding null terminator),
        /// or 0 if the index is out of range.
        unsafe fn font_get_typeface_name(index: i32, buffer: *mut u8, buffer_size: usize) -> i32;
        
        // Image operations
        
        /// Create a new JUCE Image with the specified format and dimensions.
        /// 
        /// # Arguments
        /// 
        /// * `format` - Image format (1=RGB, 2=ARGB, 3=SingleChannel)
        /// * `width` - Width of the image in pixels
        /// * `height` - Height of the image in pixels
        /// * `error_buffer` - Buffer to store error message if creation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns a pointer to the created image, or null on error.
        /// If null is returned, the error buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// The returned pointer must be freed with `delete_image` when no longer needed.
        unsafe fn create_image(format: i32, width: i32, height: i32,
                               error_buffer: *mut i8, buffer_size: usize) -> *mut JuceImage;
        
        /// Load a JUCE Image from a file.
        /// 
        /// # Arguments
        /// 
        /// * `path` - File path (UTF-8 bytes)
        /// * `path_len` - Length of the path string
        /// * `error_buffer` - Buffer to store error message if loading fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns a pointer to the loaded image, or null on error.
        /// If null is returned, the error buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// The returned pointer must be freed with `delete_image` when no longer needed.
        unsafe fn load_image_from_file(path: *const u8, path_len: usize,
                                       error_buffer: *mut i8, buffer_size: usize) -> *mut JuceImage;
        
        /// Save a JUCE Image to a file.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the image
        /// * `path` - File path (UTF-8 bytes)
        /// * `path_len` - Length of the path string
        /// * `error_buffer` - Buffer to store error message if saving fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn save_image_to_file(ptr: *const JuceImage, path: *const u8, path_len: usize,
                                     error_buffer: *mut i8, buffer_size: usize) -> i32;
        
        /// Delete a JUCE Image and free its resources.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the image to delete
        /// 
        /// # Safety
        /// 
        /// The pointer must have been created by an image creation function and must not
        /// be used after this call.
        unsafe fn delete_image(ptr: *mut JuceImage);
        
        /// Get a graphics context for drawing to an image.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the image
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns a pointer to the graphics context, or null on error.
        /// If null is returned, the error buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// The returned graphics context is only valid as long as the image exists.
        /// It should not be deleted manually.
        unsafe fn image_get_graphics_context(ptr: *mut JuceImage,
                                             error_buffer: *mut i8, buffer_size: usize) -> *mut JuceGraphics;
        
        /// Apply a blur effect to an image.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the image
        /// * `radius` - Blur radius in pixels
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn image_apply_blur(ptr: *mut JuceImage, radius: f32,
                                   error_buffer: *mut i8, buffer_size: usize) -> i32;
        
        /// Get the width of an image.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the image
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns the width in pixels, or -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn image_get_width(ptr: *const JuceImage, error_buffer: *mut i8,
                                  buffer_size: usize) -> i32;
        
        /// Get the height of an image.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the image
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns the height in pixels, or -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn image_get_height(ptr: *const JuceImage, error_buffer: *mut i8,
                                   buffer_size: usize) -> i32;
        
        // Path operations
        
        /// Create a new JUCE Path.
        /// 
        /// # Arguments
        /// 
        /// * `error_buffer` - Buffer to store error message if creation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns a pointer to the created path, or null on error.
        /// If null is returned, the error buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// The returned pointer must be freed with `delete_path` when no longer needed.
        unsafe fn create_path(error_buffer: *mut i8, buffer_size: usize) -> *mut JucePath;
        
        /// Delete a JUCE Path and free its resources.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the path to delete
        /// 
        /// # Safety
        /// 
        /// The pointer must have been created by `create_path` and must not
        /// be used after this call.
        unsafe fn delete_path(ptr: *mut JucePath);
        
        /// Start a new sub-path at the specified position.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the path
        /// * `x` - X coordinate of the starting point
        /// * `y` - Y coordinate of the starting point
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn path_start_new_sub_path(ptr: *mut JucePath, x: f32, y: f32,
                                          error_buffer: *mut i8, buffer_size: usize) -> i32;
        
        /// Add a line from the current position to the specified point.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the path
        /// * `x` - X coordinate of the end point
        /// * `y` - Y coordinate of the end point
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn path_line_to(ptr: *mut JucePath, x: f32, y: f32,
                               error_buffer: *mut i8, buffer_size: usize) -> i32;
        
        /// Add a quadratic bezier curve from the current position.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the path
        /// * `cx` - X coordinate of the control point
        /// * `cy` - Y coordinate of the control point
        /// * `x` - X coordinate of the end point
        /// * `y` - Y coordinate of the end point
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn path_quadratic_to(ptr: *mut JucePath, cx: f32, cy: f32, x: f32, y: f32,
                                    error_buffer: *mut i8, buffer_size: usize) -> i32;
        
        /// Add a cubic bezier curve from the current position.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the path
        /// * `cx1` - X coordinate of the first control point
        /// * `cy1` - Y coordinate of the first control point
        /// * `cx2` - X coordinate of the second control point
        /// * `cy2` - Y coordinate of the second control point
        /// * `x` - X coordinate of the end point
        /// * `y` - Y coordinate of the end point
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn path_cubic_to(ptr: *mut JucePath, cx1: f32, cy1: f32, cx2: f32, cy2: f32,
                                x: f32, y: f32, error_buffer: *mut i8, buffer_size: usize) -> i32;
        
        /// Add a rectangle to the path.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the path
        /// * `x` - X coordinate of the top-left corner
        /// * `y` - Y coordinate of the top-left corner
        /// * `width` - Width of the rectangle
        /// * `height` - Height of the rectangle
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn path_add_rectangle(ptr: *mut JucePath, x: f32, y: f32, width: f32, height: f32,
                                     error_buffer: *mut i8, buffer_size: usize) -> i32;
        
        /// Add an ellipse to the path.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the path
        /// * `x` - X coordinate of the bounding rectangle
        /// * `y` - Y coordinate of the bounding rectangle
        /// * `width` - Width of the bounding rectangle
        /// * `height` - Height of the bounding rectangle
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn path_add_ellipse(ptr: *mut JucePath, x: f32, y: f32, width: f32, height: f32,
                                   error_buffer: *mut i8, buffer_size: usize) -> i32;
        
        /// Add an arc to the path.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the path
        /// * `x` - X coordinate of the bounding rectangle
        /// * `y` - Y coordinate of the bounding rectangle
        /// * `width` - Width of the bounding rectangle
        /// * `height` - Height of the bounding rectangle
        /// * `start_angle` - Starting angle in radians
        /// * `end_angle` - Ending angle in radians
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn path_add_arc(ptr: *mut JucePath, x: f32, y: f32, width: f32, height: f32,
                               start_angle: f32, end_angle: f32,
                               error_buffer: *mut i8, buffer_size: usize) -> i32;
        
        /// Close the current sub-path by adding a line back to its starting point.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the path
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn path_close_sub_path(ptr: *mut JucePath,
                                      error_buffer: *mut i8, buffer_size: usize) -> i32;
        
        /// Apply a transformation to the path.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the path
        /// * `transform` - Pointer to the transformation to apply
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn path_apply_transform(ptr: *mut JucePath, transform: *const JuceAffineTransform,
                                       error_buffer: *mut i8, buffer_size: usize) -> i32;
        
        // AffineTransform operations
        
        /// Create an identity transformation.
        /// 
        /// # Arguments
        /// 
        /// * `error_buffer` - Buffer to store error message if creation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns a pointer to the created transform, or null on error.
        /// If null is returned, the error buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// The returned pointer must be freed with `delete_affine_transform` when no longer needed.
        unsafe fn create_affine_transform_identity(error_buffer: *mut i8, buffer_size: usize) -> *mut JuceAffineTransform;
        
        /// Create a translation transformation.
        /// 
        /// # Arguments
        /// 
        /// * `dx` - Translation distance in X direction
        /// * `dy` - Translation distance in Y direction
        /// * `error_buffer` - Buffer to store error message if creation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns a pointer to the created transform, or null on error.
        /// If null is returned, the error buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// The returned pointer must be freed with `delete_affine_transform` when no longer needed.
        unsafe fn create_affine_transform_translation(dx: f32, dy: f32,
                                                      error_buffer: *mut i8, buffer_size: usize) -> *mut JuceAffineTransform;
        
        /// Create a rotation transformation.
        /// 
        /// # Arguments
        /// 
        /// * `angle_radians` - Rotation angle in radians
        /// * `error_buffer` - Buffer to store error message if creation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns a pointer to the created transform, or null on error.
        /// If null is returned, the error buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// The returned pointer must be freed with `delete_affine_transform` when no longer needed.
        unsafe fn create_affine_transform_rotation(angle_radians: f32,
                                                   error_buffer: *mut i8, buffer_size: usize) -> *mut JuceAffineTransform;
        
        /// Create a scaling transformation.
        /// 
        /// # Arguments
        /// 
        /// * `sx` - Scale factor in X direction
        /// * `sy` - Scale factor in Y direction
        /// * `error_buffer` - Buffer to store error message if creation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns a pointer to the created transform, or null on error.
        /// If null is returned, the error buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// The returned pointer must be freed with `delete_affine_transform` when no longer needed.
        unsafe fn create_affine_transform_scale(sx: f32, sy: f32,
                                                error_buffer: *mut i8, buffer_size: usize) -> *mut JuceAffineTransform;
        
        /// Delete a JUCE AffineTransform and free its resources.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the transform to delete
        /// 
        /// # Safety
        /// 
        /// The pointer must have been created by a transform creation function and must not
        /// be used after this call.
        unsafe fn delete_affine_transform(ptr: *mut JuceAffineTransform);
        
        /// Compose two transformations (this followed by other).
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the first transform
        /// * `other` - Pointer to the second transform to apply after the first
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns a pointer to the new composed transform, or null on error.
        /// If null is returned, the error buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// The returned pointer must be freed with `delete_affine_transform` when no longer needed.
        unsafe fn affine_transform_followed_by(ptr: *const JuceAffineTransform,
                                               other: *const JuceAffineTransform,
                                               error_buffer: *mut i8, buffer_size: usize) -> *mut JuceAffineTransform;
        
        // FlexBox operations
        
        /// Create a new JUCE FlexBox.
        /// 
        /// # Arguments
        /// 
        /// * `error_buffer` - Buffer to store error message if creation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns a pointer to the created flexbox, or null on error.
        /// If null is returned, the error buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// The returned pointer must be freed with `delete_flexbox` when no longer needed.
        unsafe fn create_flexbox(error_buffer: *mut i8, buffer_size: usize) -> *mut JuceFlexBox;
        
        /// Delete a JUCE FlexBox and free its resources.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the flexbox to delete
        /// 
        /// # Safety
        /// 
        /// The pointer must have been created by `create_flexbox` and must not
        /// be used after this call.
        unsafe fn delete_flexbox(ptr: *mut JuceFlexBox);
        
        /// Set the flex direction.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the flexbox
        /// * `direction` - Direction value (0=Row, 1=Column, 2=RowReverse, 3=ColumnReverse)
        unsafe fn flexbox_set_direction(ptr: *mut JuceFlexBox, direction: i32);
        
        /// Set the flex wrap behavior.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the flexbox
        /// * `wrap` - Wrap value (0=NoWrap, 1=Wrap, 2=WrapReverse)
        unsafe fn flexbox_set_wrap(ptr: *mut JuceFlexBox, wrap: i32);
        
        /// Set the justify content property.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the flexbox
        /// * `justify` - Justify value (0=FlexStart, 1=FlexEnd, 2=Center, 3=SpaceBetween, 4=SpaceAround)
        unsafe fn flexbox_set_justify_content(ptr: *mut JuceFlexBox, justify: i32);
        
        /// Set the align content property.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the flexbox
        /// * `align` - Align value (0=FlexStart, 1=FlexEnd, 2=Center, 3=SpaceBetween, 4=SpaceAround, 5=Stretch)
        unsafe fn flexbox_set_align_content(ptr: *mut JuceFlexBox, align: i32);
        
        /// Set the align items property.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the flexbox
        /// * `align` - Align value (0=FlexStart, 1=FlexEnd, 2=Center, 3=Stretch)
        unsafe fn flexbox_set_align_items(ptr: *mut JuceFlexBox, align: i32);
        
        /// Add an item to the flexbox.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the flexbox
        /// * `component` - Pointer to the component to add
        /// * `flex_grow` - Flex grow factor
        /// * `flex_shrink` - Flex shrink factor
        /// * `flex_basis` - Flex basis in pixels
        /// * `min_width` - Minimum width in pixels
        /// * `min_height` - Minimum height in pixels
        /// * `max_width` - Maximum width in pixels
        /// * `max_height` - Maximum height in pixels
        /// * `margin_top` - Top margin in pixels
        /// * `margin_right` - Right margin in pixels
        /// * `margin_bottom` - Bottom margin in pixels
        /// * `margin_left` - Left margin in pixels
        unsafe fn flexbox_add_item(
            ptr: *mut JuceFlexBox,
            component: *mut JuceComponent,
            flex_grow: f32,
            flex_shrink: f32,
            flex_basis: f32,
            min_width: f32,
            min_height: f32,
            max_width: f32,
            max_height: f32,
            margin_top: f32,
            margin_right: f32,
            margin_bottom: f32,
            margin_left: f32
        );
        
        /// Perform the flex layout within the specified bounds.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the flexbox
        /// * `x` - X coordinate of the layout area
        /// * `y` - Y coordinate of the layout area
        /// * `width` - Width of the layout area
        /// * `height` - Height of the layout area
        unsafe fn flexbox_perform_layout(ptr: *mut JuceFlexBox, x: i32, y: i32, width: i32, height: i32);
        
        // DocumentWindow operations
        
        /// Create a new JUCE DocumentWindow with the specified title.
        /// 
        /// # Arguments
        /// 
        /// * `title` - The window title (UTF-8 bytes)
        /// * `title_len` - Length of the title string
        /// * `error_buffer` - Buffer to store error message if creation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns a pointer to the created document window, or null on error.
        /// If null is returned, the error buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// The returned pointer must be freed with `delete_component` when no longer needed.
        unsafe fn create_document_window(title: *const u8, title_len: usize,
                                         error_buffer: *mut i8, buffer_size: usize) -> *mut JuceComponent;
        
        /// Set the content component for a document window, transferring ownership.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the document window
        /// * `content` - Pointer to the content component
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn document_window_set_content_owned(ptr: *mut JuceComponent, content: *mut JuceComponent,
                                                     error_buffer: *mut i8, buffer_size: usize) -> i32;
        
        /// Set the name (title) of a document window.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the document window
        /// * `name` - The new window title (UTF-8 bytes)
        /// * `name_len` - Length of the name string
        unsafe fn document_window_set_name(ptr: *mut JuceComponent, name: *const u8, name_len: usize);
        
        /// Set a close callback for a document window.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the document window
        /// * `rust_closure` - Pointer to the Rust closure (as usize)
        /// * `invoke` - Function pointer to invoke the closure (as usize)
        /// * `drop_fn` - Function pointer to drop the closure (as usize)
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn document_window_set_on_close(
            ptr: *mut JuceComponent,
            rust_closure: usize,
            invoke: usize,
            drop_fn: usize,
            error_buffer: *mut i8,
            buffer_size: usize
        ) -> i32;
        
        // ============================================================================
        // ResizableWindow Operations
        // ============================================================================
        
        /// Create a new JUCE ResizableWindow with the specified title.
        /// 
        /// # Arguments
        /// 
        /// * `title` - The window title (UTF-8 bytes)
        /// * `title_len` - Length of the title string
        /// * `error_buffer` - Buffer to store error message if creation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns a pointer to the created resizable window, or null on error.
        /// If null is returned, the error buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// The returned pointer must be freed with `delete_component` when no longer needed.
        unsafe fn create_resizable_window(title: *const u8, title_len: usize,
                                          error_buffer: *mut i8, buffer_size: usize) -> *mut JuceComponent;
        
        /// Enable or disable user resizing of the window.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the resizable window
        /// * `resizable` - Whether the window should be resizable
        unsafe fn resizable_window_set_resizable(ptr: *mut JuceComponent, resizable: bool);
        
        /// Set the minimum and maximum size constraints for the window.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the resizable window
        /// * `min_width` - Minimum window width in pixels
        /// * `min_height` - Minimum window height in pixels
        /// * `max_width` - Maximum window width in pixels
        /// * `max_height` - Maximum window height in pixels
        unsafe fn resizable_window_set_resize_limits(ptr: *mut JuceComponent,
                                                     min_width: i32, min_height: i32,
                                                     max_width: i32, max_height: i32);
        
        /// Set a resize callback for a resizable window.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the resizable window
        /// * `rust_closure` - Pointer to the Rust closure (as usize)
        /// * `invoke` - Function pointer to invoke the closure (as usize)
        /// * `drop_fn` - Function pointer to drop the closure (as usize)
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn resizable_window_set_on_resized(
            ptr: *mut JuceComponent,
            rust_closure: usize,
            invoke: usize,
            drop_fn: usize,
            error_buffer: *mut i8,
            buffer_size: usize
        ) -> i32;
        
        // ============================================================================
        // Viewport Operations
        // ============================================================================
        
        /// Create a new JUCE Viewport.
        /// 
        /// # Arguments
        /// 
        /// * `error_buffer` - Buffer to store error message if creation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns a pointer to the created viewport, or null on error.
        /// If null is returned, the error buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// The returned pointer must be freed with `delete_component` when no longer needed.
        unsafe fn create_viewport(error_buffer: *mut i8, buffer_size: usize) -> *mut JuceComponent;
        
        /// Set the component to be viewed in the viewport, transferring ownership.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the viewport
        /// * `component` - Pointer to the component to view
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn viewport_set_viewed_component(ptr: *mut JuceComponent, component: *mut JuceComponent,
                                                error_buffer: *mut i8, buffer_size: usize) -> i32;
        
        /// Set the scroll position of the viewport.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the viewport
        /// * `x` - X coordinate of the top-left corner of the visible area
        /// * `y` - Y coordinate of the top-left corner of the visible area
        unsafe fn viewport_set_view_position(ptr: *mut JuceComponent, x: i32, y: i32);
        
        /// Set whether scrollbars are shown.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the viewport
        /// * `vertical` - Whether to show the vertical scrollbar
        /// * `horizontal` - Whether to show the horizontal scrollbar
        unsafe fn viewport_set_scrollbars_shown(ptr: *mut JuceComponent, vertical: bool, horizontal: bool);
        
        /// Set a callback to be invoked when the visible area changes.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the viewport
        /// * `rust_closure` - Pointer to the Rust closure (as usize)
        /// * `invoke` - Function pointer to invoke the closure (as usize)
        /// * `drop_fn` - Function pointer to drop the closure (as usize)
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn viewport_set_on_visible_area_changed(
            ptr: *mut JuceComponent,
            rust_closure: usize,
            invoke: usize,
            drop_fn: usize,
            error_buffer: *mut i8,
            buffer_size: usize
        ) -> i32;
        
        // ============================================================================
        // TabbedComponent Operations
        // ============================================================================
        
        /// Create a new JUCE TabbedComponent with the specified orientation.
        /// 
        /// # Arguments
        /// 
        /// * `orientation` - Tab orientation (0=Top, 1=Bottom, 2=Left, 3=Right)
        /// * `error_buffer` - Buffer to store error message if creation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns a pointer to the created tabbed component, or null on error.
        /// If null is returned, the error buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// The returned pointer must be freed with `delete_component` when no longer needed.
        unsafe fn create_tabbed_component(orientation: i32,
                                          error_buffer: *mut i8, buffer_size: usize) -> *mut JuceComponent;
        
        /// Add a tab to a tabbed component.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the tabbed component
        /// * `name` - The tab name (UTF-8 bytes)
        /// * `name_len` - Length of the name string
        /// * `colour` - Pointer to the tab background colour
        /// * `content` - Pointer to the content component
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn tabbed_component_add_tab(ptr: *mut JuceComponent, name: *const u8, name_len: usize,
                                           colour: *const JuceColour, content: *mut JuceComponent,
                                           error_buffer: *mut i8, buffer_size: usize) -> i32;
        
        /// Remove a tab from a tabbed component.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the tabbed component
        /// * `index` - The index of the tab to remove (0-based)
        unsafe fn tabbed_component_remove_tab(ptr: *mut JuceComponent, index: i32);
        
        /// Set the current tab index.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the tabbed component
        /// * `index` - The index of the tab to select (0-based)
        unsafe fn tabbed_component_set_current_tab_index(ptr: *mut JuceComponent, index: i32);
        
        /// Set a callback to be invoked when the current tab changes.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the tabbed component
        /// * `rust_closure` - Pointer to the Rust closure (as usize)
        /// * `invoke` - Function pointer to invoke the closure (as usize)
        /// * `drop_fn` - Function pointer to drop the closure (as usize)
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn tabbed_component_set_on_tab_changed(
            ptr: *mut JuceComponent,
            rust_closure: usize,
            invoke: usize,
            drop_fn: usize,
            error_buffer: *mut i8,
            buffer_size: usize
        ) -> i32;
        
        // ============================================================================
        // ListBox Operations
        // ============================================================================
        
        /// Create a new JUCE ListBox.
        /// 
        /// # Arguments
        /// 
        /// * `error_buffer` - Buffer to store error message if creation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns a pointer to the created list box, or null on error.
        /// If null is returned, the error buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// The returned pointer must be freed with `delete_component` when no longer needed.
        unsafe fn create_list_box(error_buffer: *mut i8, buffer_size: usize) -> *mut JuceComponent;
        
        /// Set the model for a list box.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the list box component
        /// * `model_ptr` - Pointer to the Rust model (as usize)
        /// * `get_num_rows` - Function pointer to get the number of rows (as usize)
        /// * `paint_item` - Function pointer to paint an item (as usize)
        /// * `selection_changed` - Function pointer for selection changes (as usize)
        /// * `drop_fn` - Function pointer to drop the model (as usize)
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn list_box_set_model(
            ptr: *mut JuceComponent,
            model_ptr: usize,
            get_num_rows: usize,
            paint_item: usize,
            selection_changed: usize,
            drop_fn: usize,
            error_buffer: *mut i8,
            buffer_size: usize
        ) -> i32;
        
        /// Update the content of a list box.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the list box component
        unsafe fn list_box_update_content(ptr: *mut JuceComponent);
        
        // TreeView operations
        
        /// Create a new JUCE TreeView.
        /// 
        /// # Arguments
        /// 
        /// * `error_buffer` - Buffer to store error message if creation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns a pointer to the created tree view, or null on error.
        /// If null is returned, the error buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// The returned pointer must be freed with `delete_component` when no longer needed.
        unsafe fn create_tree_view(error_buffer: *mut i8, buffer_size: usize) -> *mut JuceComponent;
        
        /// Set the root item for a tree view.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the tree view component
        /// * `item_ptr` - Pointer to the Rust item (as usize)
        /// * `get_num_sub_items` - Function pointer to get the number of sub-items (as usize)
        /// * `get_sub_item` - Function pointer to get a sub-item (as usize)
        /// * `paint_item` - Function pointer to paint an item (as usize)
        /// * `item_clicked` - Function pointer for item clicks (as usize)
        /// * `drop_fn` - Function pointer to drop the item (as usize)
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn tree_view_set_root_item(
            ptr: *mut JuceComponent,
            item_ptr: usize,
            get_num_sub_items: usize,
            get_sub_item: usize,
            paint_item: usize,
            item_clicked: usize,
            drop_fn: usize,
            error_buffer: *mut i8,
            buffer_size: usize
        ) -> i32;
        
        // AlertWindow operations
        
        /// Show a synchronous message box.
        /// 
        /// This displays a simple message box with an OK button and blocks until
        /// the user dismisses it.
        /// 
        /// # Arguments
        /// 
        /// * `title` - The dialog title (UTF-8 bytes)
        /// * `title_len` - Length of the title string
        /// * `message` - The message text (UTF-8 bytes)
        /// * `message_len` - Length of the message string
        /// 
        /// # Safety
        /// 
        /// This function must be called on the JUCE message thread.
        unsafe fn alert_window_show_message_box(
            title: *const u8,
            title_len: usize,
            message: *const u8,
            message_len: usize
        );
        
        /// Show an asynchronous message box with a callback.
        /// 
        /// This displays a message box with an OK button and returns immediately.
        /// When the user dismisses the dialog, the callback is invoked.
        /// 
        /// # Arguments
        /// 
        /// * `title` - The dialog title (UTF-8 bytes)
        /// * `title_len` - Length of the title string
        /// * `message` - The message text (UTF-8 bytes)
        /// * `message_len` - Length of the message string
        /// * `rust_closure` - Pointer to the Rust closure (as usize)
        /// * `invoke` - Function pointer to invoke the closure (as usize)
        /// * `drop_fn` - Function pointer to drop the closure (as usize)
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// This function must be called on the JUCE message thread.
        unsafe fn alert_window_show_message_box_async(
            title: *const u8,
            title_len: usize,
            message: *const u8,
            message_len: usize,
            rust_closure: usize,
            invoke: usize,
            drop_fn: usize,
            error_buffer: *mut i8,
            buffer_size: usize
        ) -> i32;
        
        /// Show an OK/Cancel confirmation dialog with a callback.
        /// 
        /// This displays a dialog with OK and Cancel buttons and returns immediately.
        /// When the user clicks a button, the callback is invoked with true for OK
        /// or false for Cancel.
        /// 
        /// # Arguments
        /// 
        /// * `title` - The dialog title (UTF-8 bytes)
        /// * `title_len` - Length of the title string
        /// * `message` - The message text (UTF-8 bytes)
        /// * `message_len` - Length of the message string
        /// * `rust_closure` - Pointer to the Rust closure (as usize)
        /// * `invoke` - Function pointer to invoke the closure (as usize)
        /// * `drop_fn` - Function pointer to drop the closure (as usize)
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// This function must be called on the JUCE message thread.
        unsafe fn alert_window_show_ok_cancel_box(
            title: *const u8,
            title_len: usize,
            message: *const u8,
            message_len: usize,
            rust_closure: usize,
            invoke: usize,
            drop_fn: usize,
            error_buffer: *mut i8,
            buffer_size: usize
        ) -> i32;
        
        // FileChooser operations
        
        /// Create a new JUCE FileChooser.
        /// 
        /// # Arguments
        /// 
        /// * `title` - The dialog title (UTF-8 bytes)
        /// * `title_len` - Length of the title string
        /// * `initial_dir` - The initial directory path (UTF-8 bytes)
        /// * `initial_dir_len` - Length of the initial directory string
        /// * `filters` - File filters in format "*.ext1;*.ext2" (UTF-8 bytes)
        /// * `filters_len` - Length of the filters string
        /// * `error_buffer` - Buffer to store error message if creation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns a pointer to the created file chooser, or null on error.
        /// If null is returned, the error buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// The returned pointer must be freed with `delete_file_chooser` when no longer needed.
        /// This function must be called on the JUCE message thread.
        unsafe fn create_file_chooser(
            title: *const u8,
            title_len: usize,
            initial_dir: *const u8,
            initial_dir_len: usize,
            filters: *const u8,
            filters_len: usize,
            error_buffer: *mut i8,
            buffer_size: usize
        ) -> *mut JuceFileChooser;
        
        /// Delete a JUCE FileChooser and free its resources.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the file chooser to delete
        /// 
        /// # Safety
        /// 
        /// The pointer must have been created by `create_file_chooser` and must not
        /// be used after this call.
        unsafe fn delete_file_chooser(ptr: *mut JuceFileChooser);
        
        /// Browse for a file to open.
        /// 
        /// This displays a native file open dialog and returns immediately.
        /// When the user selects a file or cancels, the callback is invoked.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the file chooser
        /// * `rust_closure` - Pointer to the Rust closure (as usize)
        /// * `invoke` - Function pointer to invoke the closure (as usize)
        /// * `drop_fn` - Function pointer to drop the closure (as usize)
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// This function must be called on the JUCE message thread.
        /// The callback will be invoked with (closure_ptr, path_ptr, path_len).
        /// If the user cancels, path_ptr will be null and path_len will be 0.
        unsafe fn file_chooser_browse_for_file_to_open(
            ptr: *mut JuceFileChooser,
            rust_closure: usize,
            invoke: usize,
            drop_fn: usize,
            error_buffer: *mut i8,
            buffer_size: usize
        ) -> i32;
        
        /// Browse for a file to save.
        /// 
        /// This displays a native file save dialog and returns immediately.
        /// When the user selects a file or cancels, the callback is invoked.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the file chooser
        /// * `rust_closure` - Pointer to the Rust closure (as usize)
        /// * `invoke` - Function pointer to invoke the closure (as usize)
        /// * `drop_fn` - Function pointer to drop the closure (as usize)
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// This function must be called on the JUCE message thread.
        /// The callback will be invoked with (closure_ptr, path_ptr, path_len).
        /// If the user cancels, path_ptr will be null and path_len will be 0.
        unsafe fn file_chooser_browse_for_file_to_save(
            ptr: *mut JuceFileChooser,
            rust_closure: usize,
            invoke: usize,
            drop_fn: usize,
            error_buffer: *mut i8,
            buffer_size: usize
        ) -> i32;
        
        // Drawable operations
        
        /// Opaque pointer to a JUCE Drawable object.
        /// 
        /// This represents a juce::Drawable in C++, which is a vector graphics
        /// object that can be loaded from SVG or image data.
        type JuceDrawable;
        
        /// Create a Drawable from SVG data.
        /// 
        /// # Arguments
        /// 
        /// * `svg_data` - The SVG data (UTF-8 bytes)
        /// * `svg_len` - Length of the SVG data
        /// * `error_buffer` - Buffer to store error message if creation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns a pointer to the created drawable, or null on error.
        /// If null is returned, the error buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// The returned pointer must be freed with `delete_drawable` when no longer needed.
        unsafe fn create_drawable_from_svg(
            svg_data: *const u8,
            svg_len: usize,
            error_buffer: *mut i8,
            buffer_size: usize
        ) -> *mut JuceDrawable;
        
        /// Create a Drawable from image data.
        /// 
        /// # Arguments
        /// 
        /// * `image_data` - The image data bytes (PNG, JPEG, etc.)
        /// * `data_len` - Length of the image data
        /// * `error_buffer` - Buffer to store error message if creation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns a pointer to the created drawable, or null on error.
        /// If null is returned, the error buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// The returned pointer must be freed with `delete_drawable` when no longer needed.
        unsafe fn create_drawable_from_image_data(
            image_data: *const u8,
            data_len: usize,
            error_buffer: *mut i8,
            buffer_size: usize
        ) -> *mut JuceDrawable;
        
        /// Delete a JUCE Drawable and free its resources.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the drawable to delete
        /// 
        /// # Safety
        /// 
        /// The pointer must have been created by a drawable creation function and must not
        /// be used after this call.
        unsafe fn delete_drawable(ptr: *mut JuceDrawable);
        
        /// Draw a drawable to a Graphics context.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the drawable
        /// * `g` - Pointer to the Graphics context
        /// * `opacity` - Opacity to draw with (0.0 = transparent, 1.0 = opaque)
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn drawable_draw(
            ptr: *const JuceDrawable,
            g: *mut JuceGraphics,
            opacity: f32,
            error_buffer: *mut i8,
            buffer_size: usize
        ) -> i32;
        
        /// Set the drawable's transform to fit within the specified bounds.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the drawable
        /// * `x` - X coordinate of the bounding rectangle
        /// * `y` - Y coordinate of the bounding rectangle
        /// * `width` - Width of the bounding rectangle
        /// * `height` - Height of the bounding rectangle
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn drawable_set_transform_to_fit(
            ptr: *mut JuceDrawable,
            x: f32,
            y: f32,
            width: f32,
            height: f32,
            error_buffer: *mut i8,
            buffer_size: usize
        ) -> i32;
        
        // DrawableButton operations
        
        /// Create a new JUCE DrawableButton.
        /// 
        /// # Arguments
        /// 
        /// * `name` - The button name (UTF-8 bytes)
        /// * `name_len` - Length of the name string
        /// * `error_buffer` - Buffer to store error message if creation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns a pointer to the created drawable button, or null on error.
        /// If null is returned, the error buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// The returned pointer must be freed with `delete_component` when no longer needed.
        unsafe fn create_drawable_button(
            name: *const u8,
            name_len: usize,
            error_buffer: *mut i8,
            buffer_size: usize
        ) -> *mut JuceComponent;
        
        /// Set the images for a DrawableButton.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the drawable button component
        /// * `normal` - Pointer to the normal state drawable (required)
        /// * `over` - Pointer to the hover state drawable (optional, can be null)
        /// * `down` - Pointer to the pressed state drawable (optional, can be null)
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn drawable_button_set_images(
            ptr: *mut JuceComponent,
            normal: *const JuceDrawable,
            over: *const JuceDrawable,
            down: *const JuceDrawable,
            error_buffer: *mut i8,
            buffer_size: usize
        ) -> i32;
        
        // LookAndFeel operations
        
        /// Create a new JUCE LookAndFeel_V4.
        /// 
        /// # Arguments
        /// 
        /// * `error_buffer` - Buffer to store error message if creation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns a pointer to the created LookAndFeel, or null on error.
        /// If null is returned, the error buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// The returned pointer must be freed with `delete_lookandfeel` when no longer needed.
        unsafe fn create_lookandfeel_v4(
            error_buffer: *mut i8,
            buffer_size: usize
        ) -> *mut JuceLookAndFeel;
        
        /// Delete a JUCE LookAndFeel and free its resources.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the LookAndFeel to delete
        /// 
        /// # Safety
        /// 
        /// The pointer must have been created by `create_lookandfeel_v4` and must not
        /// be used after this call.
        unsafe fn delete_lookandfeel(ptr: *mut JuceLookAndFeel);
        
        /// Set a color for a specific color ID in a LookAndFeel.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the LookAndFeel
        /// * `colour_id` - The JUCE color ID to set
        /// * `colour` - Pointer to the color to use
        unsafe fn lookandfeel_set_colour(
            ptr: *mut JuceLookAndFeel,
            colour_id: i32,
            colour: *const JuceColour
        );
        
        /// Find the color for a specific color ID in a LookAndFeel.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the LookAndFeel
        /// * `colour_id` - The JUCE color ID to query
        /// 
        /// # Returns
        /// 
        /// Returns a pointer to the color for the given ID.
        /// The returned pointer is owned by the LookAndFeel and should not be freed.
        unsafe fn lookandfeel_find_colour(
            ptr: *const JuceLookAndFeel,
            colour_id: i32
        ) -> *const JuceColour;
        
        /// Set the LookAndFeel for a component.
        /// 
        /// # Arguments
        /// 
        /// * `component` - Pointer to the component
        /// * `laf` - Pointer to the LookAndFeel to use
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        unsafe fn component_set_look_and_feel(
            component: *mut JuceComponent,
            laf: *mut JuceLookAndFeel,
            error_buffer: *mut i8,
            buffer_size: usize
        ) -> i32;
        
        // Parameter attachment operations
        
        /// Opaque pointer to a JUCE SliderParameterAttachment object.
        /// 
        /// This represents a juce::SliderParameterAttachment in C++, which
        /// provides bidirectional synchronization between a slider and a parameter.
        type JuceSliderParameterAttachment;
        
        /// Create a new SliderParameterAttachment.
        /// 
        /// This establishes bidirectional synchronization between a slider
        /// and an audio parameter. When the slider value changes, the parameter
        /// is updated. When the parameter changes (e.g., from automation),
        /// the slider is updated.
        /// 
        /// # Arguments
        /// 
        /// * `slider` - Pointer to the slider component
        /// * `parameter_id` - The parameter ID (UTF-8 bytes)
        /// * `parameter_id_len` - Length of the parameter ID string
        /// * `error_buffer` - Buffer to store error message if creation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns a pointer to the created attachment, or null on error.
        /// If null is returned, the error buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// The returned pointer must be freed with `delete_slider_parameter_attachment`
        /// when no longer needed.
        unsafe fn create_slider_parameter_attachment(
            slider: *mut JuceComponent,
            parameter_id: *const u8,
            parameter_id_len: usize,
            error_buffer: *mut i8,
            buffer_size: usize
        ) -> *mut JuceSliderParameterAttachment;
        
        /// Delete a SliderParameterAttachment and free its resources.
        /// 
        /// This breaks the connection between the slider and parameter,
        /// stopping bidirectional synchronization.
        /// 
        /// # Arguments
        /// 
        /// * `ptr` - Pointer to the attachment to delete
        /// 
        /// # Safety
        /// 
        /// The pointer must have been created by `create_slider_parameter_attachment`
        /// and must not be used after this call.
        unsafe fn delete_slider_parameter_attachment(ptr: *mut JuceSliderParameterAttachment);
        
        // MessageManager operations
        
        /// Check if the current thread is the message thread.
        /// 
        /// JUCE requires all GUI operations to be performed on the message thread.
        /// This function queries JUCE to determine if the calling thread is the
        /// message thread.
        /// 
        /// # Returns
        /// 
        /// Returns `true` if the current thread is the message thread, `false` otherwise.
        /// 
        /// # Safety
        /// 
        /// This function is safe to call from any thread.
        unsafe fn message_manager_is_message_thread() -> bool;
        
        /// Post a callback to be executed on the message thread.
        /// 
        /// This function queues a closure for execution on the message thread.
        /// It's the safe way to update the UI from another thread (e.g., the
        /// audio processing thread).
        /// 
        /// The callback will be executed asynchronously - this function returns
        /// immediately without waiting for the callback to execute.
        /// 
        /// # Arguments
        /// 
        /// * `rust_closure` - Pointer to the Rust closure (as usize)
        /// * `invoke` - Function pointer to invoke the closure (as usize)
        /// * `error_buffer` - Buffer to store error message if operation fails
        /// * `buffer_size` - Size of the error buffer
        /// 
        /// # Returns
        /// 
        /// Returns 0 on success, -1 on error. If -1 is returned, the error
        /// buffer will contain an error message.
        /// 
        /// # Safety
        /// 
        /// The rust_closure pointer must remain valid until the callback is invoked
        /// on the message thread. The invoke function pointer must be a valid
        /// trampoline function that can safely call the closure.
        unsafe fn message_manager_call_async(
            rust_closure: usize,
            invoke: usize,
            error_buffer: *mut i8,
            buffer_size: usize
        ) -> i32;
    }
}
