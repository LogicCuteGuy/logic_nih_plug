// juce_bridge.h
// Main header file for the JUCE FFI bridge
//
// This file contains C++ declarations for FFI functions that bridge
// between Rust and JUCE. It defines opaque types and bridge functions
// that are exposed to Rust through the cxx crate.
//
// All C++ exceptions are caught at the FFI boundary and converted to
// error codes or error messages that can be safely handled in Rust.

#pragma once

#include <cstddef>
#include <cstdint>
#include <exception>
#include <string>

// Define JUCE global header before including any JUCE modules
// This is required by JUCE to properly configure platform-specific settings
#define JUCE_GLOBAL_MODULE_SETTINGS_INCLUDED 1

// Ensure debug/release mode consistency
// JUCE requires all compilation units to be built in the same mode
#ifndef NDEBUG
  #define DEBUG 1
#else
  #define NDEBUG 1
#endif

// Include JUCE headers
#include <juce_core/juce_core.h>
#include <juce_events/juce_events.h>
#include <juce_graphics/juce_graphics.h>
#include <juce_gui_basics/juce_gui_basics.h>

namespace nih_plug_juce {

// Version information
constexpr int VERSION_MAJOR = 0;
constexpr int VERSION_MINOR = 1;
constexpr int VERSION_PATCH = 0;

// Opaque type definitions
// These types are exposed to Rust as opaque pointers through cxx.
// The actual C++ classes are hidden from Rust, which only sees
// pointers to these types.

/// Opaque wrapper for juce::Component
struct JuceComponent {
    juce::Component* ptr;
    
    explicit JuceComponent(juce::Component* p) : ptr(p) {}
    ~JuceComponent() = default;
};

/// Opaque wrapper for juce::Graphics
struct JuceGraphics {
    juce::Graphics* ptr;
    
    explicit JuceGraphics(juce::Graphics* p) : ptr(p) {}
    ~JuceGraphics() = default;
};

/// Opaque wrapper for juce::Colour
struct JuceColour {
    juce::Colour colour;
    
    JuceColour() : colour() {}
    explicit JuceColour(const juce::Colour& c) : colour(c) {}
    ~JuceColour() = default;
};

/// Opaque wrapper for juce::Font
struct JuceFont {
    juce::Font font;
    
    JuceFont() : font() {}
    explicit JuceFont(const juce::Font& f) : font(f) {}
    ~JuceFont() = default;
};

/// Opaque wrapper for juce::Image
struct JuceImage {
    juce::Image image;
    
    JuceImage() : image() {}
    explicit JuceImage(const juce::Image& img) : image(img) {}
    ~JuceImage() = default;
};

/// Opaque wrapper for juce::Path
struct JucePath {
    juce::Path path;
    
    JucePath() : path() {}
    explicit JucePath(const juce::Path& p) : path(p) {}
    ~JucePath() = default;
};

/// Opaque wrapper for juce::AffineTransform
struct JuceAffineTransform {
    juce::AffineTransform transform;
    
    JuceAffineTransform() : transform() {}
    explicit JuceAffineTransform(const juce::AffineTransform& t) : transform(t) {}
    ~JuceAffineTransform() = default;
};

/// Opaque wrapper for juce::FlexBox
struct JuceFlexBox {
    juce::FlexBox flexbox;
    
    JuceFlexBox() : flexbox() {}
    explicit JuceFlexBox(const juce::FlexBox& fb) : flexbox(fb) {}
    ~JuceFlexBox() = default;
};

/// Opaque wrapper for juce::Timer
struct JuceTimer {
    juce::Timer* ptr;
    
    explicit JuceTimer(juce::Timer* p) : ptr(p) {}
    ~JuceTimer() = default;
};

/// Opaque wrapper for juce::LookAndFeel
struct JuceLookAndFeel {
    juce::LookAndFeel* ptr;
    
    explicit JuceLookAndFeel(juce::LookAndFeel* p) : ptr(p) {}
    ~JuceLookAndFeel() = default;
};

/// Opaque wrapper for juce::FileChooser
/// Forward declaration - actual definition is in file_chooser_bridge.cpp
struct JuceFileChooser;

/// Opaque wrapper for juce::Drawable
struct JuceDrawable {
    juce::Drawable* ptr;
    
    explicit JuceDrawable(juce::Drawable* p) : ptr(p) {}
    ~JuceDrawable() = default;
};

// Exception handling utilities

/// Exception handler that catches C++ exceptions and converts them to error messages.
/// 
/// This template function wraps any operation that might throw a C++ exception,
/// catches the exception, and stores the error message in the provided buffer.
/// 
/// @param operation The operation to execute (typically a lambda)
/// @param error_buffer Buffer to store error message if an exception occurs
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
template<typename Func>
inline int catch_exceptions(Func&& operation, char* error_buffer, size_t buffer_size) {
    try {
        operation();
        return 0; // Success
    } catch (const std::exception& e) {
        if (error_buffer && buffer_size > 0) {
            std::snprintf(error_buffer, buffer_size, "%s", e.what());
        }
        return -1; // Error
    } catch (...) {
        if (error_buffer && buffer_size > 0) {
            std::snprintf(error_buffer, buffer_size, "Unknown C++ exception");
        }
        return -1; // Error
    }
}

/// Exception handler for operations that return a pointer.
/// 
/// Similar to catch_exceptions, but for operations that return a pointer.
/// Returns nullptr on error.
/// 
/// @param operation The operation to execute (typically a lambda)
/// @param error_buffer Buffer to store error message if an exception occurs
/// @param buffer_size Size of the error buffer
/// @return Pointer on success, nullptr on error
template<typename Func>
inline auto catch_exceptions_ptr(Func&& operation, char* error_buffer, size_t buffer_size) 
    -> decltype(operation()) {
    try {
        return operation();
    } catch (const std::exception& e) {
        if (error_buffer && buffer_size > 0) {
            std::snprintf(error_buffer, buffer_size, "%s", e.what());
        }
        return nullptr;
    } catch (...) {
        if (error_buffer && buffer_size > 0) {
            std::snprintf(error_buffer, buffer_size, "Unknown C++ exception");
        }
        return nullptr;
    }
}

// Initialization and utility functions

/// Initialize the JUCE FFI bridge.
/// 
/// This function verifies that JUCE is properly linked and initialized.
/// It performs basic sanity checks on JUCE functionality.
/// 
/// @return true if initialization succeeded, false otherwise
bool initialize();

/// Get the version string of the JUCE FFI bridge.
/// 
/// @return Version string in the format "major.minor.patch"
std::string get_version();

// Component operations

/// Create a new JUCE Component.
/// 
/// @param error_buffer Buffer to store error message if creation fails
/// @param buffer_size Size of the error buffer
/// @return Pointer to the created component, or nullptr on error
JuceComponent* create_component(int8_t* error_buffer, size_t buffer_size);

/// Create a new JUCE Component that supports paint callbacks.
/// 
/// @param error_buffer Buffer to store error message if creation fails
/// @param buffer_size Size of the error buffer
/// @return Pointer to the created component, or nullptr on error
JuceComponent* create_component_with_paint_callback(int8_t* error_buffer, size_t buffer_size);

/// Delete a JUCE Component and free its resources.
/// 
/// @param ptr Pointer to the component to delete
void delete_component(JuceComponent* ptr);

/// Add a child component to a parent component.
/// 
/// @param parent Pointer to the parent component
/// @param child Pointer to the child component to add
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t component_add_child(JuceComponent* parent, JuceComponent* child,
                            int8_t* error_buffer, size_t buffer_size);

/// Remove a child component from a parent component.
/// 
/// @param parent Pointer to the parent component
/// @param child Pointer to the child component to remove
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t component_remove_child(JuceComponent* parent, JuceComponent* child,
                               int8_t* error_buffer, size_t buffer_size);

/// Set the bounds (position and size) of a component.
/// 
/// @param ptr Pointer to the component
/// @param x X coordinate of the top-left corner
/// @param y Y coordinate of the top-left corner
/// @param width Width of the component
/// @param height Height of the component
void component_set_bounds(JuceComponent* ptr, int32_t x, int32_t y, int32_t width, int32_t height);

/// Set whether a component is visible.
/// 
/// @param ptr Pointer to the component
/// @param visible true to make visible, false to hide
void component_set_visible(JuceComponent* ptr, bool visible);

/// Trigger a repaint of a component.
/// 
/// @param ptr Pointer to the component
void component_repaint(JuceComponent* ptr);

// Paint callback support

/// Callback bridge structure for paint callbacks.
/// This structure holds a Rust closure and a function pointer to invoke it.
struct PaintCallbackBridge {
    void* rust_closure;
    void (*invoke)(void*, JuceGraphics*);
    void (*drop)(void*);
};

/// Set a paint callback for a component.
/// 
/// @param ptr Pointer to the component
/// @param rust_closure Pointer to the Rust closure (as size_t)
/// @param invoke Function pointer to invoke the closure (as size_t)
/// @param drop_fn Function pointer to drop the closure (as size_t)
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t component_set_paint_callback(JuceComponent* ptr,
                                     size_t rust_closure,
                                     size_t invoke,
                                     size_t drop_fn,
                                     int8_t* error_buffer,
                                     size_t buffer_size);

// Graphics operations

/// Fill a rectangle with the current color.
/// 
/// @param g Pointer to the Graphics context
/// @param x X coordinate of the top-left corner
/// @param y Y coordinate of the top-left corner
/// @param width Width of the rectangle
/// @param height Height of the rectangle
void graphics_fill_rect(JuceGraphics* g, int32_t x, int32_t y, int32_t width, int32_t height);

/// Draw a rectangle outline with the current color.
/// 
/// @param g Pointer to the Graphics context
/// @param x X coordinate of the top-left corner
/// @param y Y coordinate of the top-left corner
/// @param width Width of the rectangle
/// @param height Height of the rectangle
void graphics_draw_rect(JuceGraphics* g, int32_t x, int32_t y, int32_t width, int32_t height);

/// Fill an ellipse with the current color.
/// 
/// @param g Pointer to the Graphics context
/// @param x X coordinate of the bounding rectangle
/// @param y Y coordinate of the bounding rectangle
/// @param width Width of the bounding rectangle
/// @param height Height of the bounding rectangle
void graphics_fill_ellipse(JuceGraphics* g, float x, float y, float width, float height);

/// Draw a line with the current color.
/// 
/// @param g Pointer to the Graphics context
/// @param x1 X coordinate of the start point
/// @param y1 Y coordinate of the start point
/// @param x2 X coordinate of the end point
/// @param y2 Y coordinate of the end point
void graphics_draw_line(JuceGraphics* g, float x1, float y1, float x2, float y2);

/// Set the current drawing color.
/// 
/// @param g Pointer to the Graphics context
/// @param colour Pointer to the color to use
void graphics_set_colour(JuceGraphics* g, const JuceColour* colour);

/// Draw text within a rectangle.
/// 
/// @param g Pointer to the Graphics context
/// @param text The text to draw (unsigned char pointer for UTF-8)
/// @param text_len Length of the text string
/// @param x X coordinate of the text rectangle
/// @param y Y coordinate of the text rectangle
/// @param width Width of the text rectangle
/// @param height Height of the text rectangle
/// @param justification Text justification flags
void graphics_draw_text(JuceGraphics* g, const uint8_t* text, size_t text_len,
                       int32_t x, int32_t y, int32_t width, int32_t height,
                       int32_t justification);

/// Draw an image at the specified position.
/// 
/// @param g Pointer to the Graphics context
/// @param image Pointer to the image to draw
/// @param x X coordinate where the image should be drawn
/// @param y Y coordinate where the image should be drawn
void graphics_draw_image_at(JuceGraphics* g, const JuceImage* image, int32_t x, int32_t y);

/// Stroke (outline) a path with the current color.
/// 
/// @param g Pointer to the Graphics context
/// @param path Pointer to the path to stroke
void graphics_stroke_path(JuceGraphics* g, const JucePath* path);

/// Fill a path with the current color.
/// 
/// @param g Pointer to the Graphics context
/// @param path Pointer to the path to fill
void graphics_fill_path(JuceGraphics* g, const JucePath* path);

// Button operations

/// Create a new JUCE TextButton with the specified text.
/// 
/// @param text The button text (UTF-8 bytes)
/// @param text_len Length of the text string
/// @param error_buffer Buffer to store error message if creation fails
/// @param buffer_size Size of the error buffer
/// @return Pointer to the created button, or nullptr on error
JuceComponent* create_text_button(const uint8_t* text, size_t text_len,
                                  int8_t* error_buffer, size_t buffer_size);

/// Set the text of a button.
/// 
/// @param ptr Pointer to the button component
/// @param text The new button text (UTF-8 bytes)
/// @param text_len Length of the text string
void button_set_text(JuceComponent* ptr, const uint8_t* text, size_t text_len);

/// Set whether a button is enabled.
/// 
/// @param ptr Pointer to the button component
/// @param enabled true to enable, false to disable
void button_set_enabled(JuceComponent* ptr, bool enabled);

/// Set a color for a button.
/// 
/// @param ptr Pointer to the button component
/// @param colour_id The color ID to set
/// @param r Red component (0-255)
/// @param g Green component (0-255)
/// @param b Blue component (0-255)
/// @param a Alpha component (0-255)
void button_set_colour(JuceComponent* ptr, int32_t colour_id, 
                      uint8_t r, uint8_t g, uint8_t b, uint8_t a);

/// Set a click callback for a button.
/// 
/// @param ptr Pointer to the button component
/// @param rust_closure Pointer to the Rust closure (as size_t)
/// @param invoke Function pointer to invoke the closure (as size_t)
/// @param drop_fn Function pointer to drop the closure (as size_t)
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t button_set_on_click(JuceComponent* ptr,
                            size_t rust_closure,
                            size_t invoke,
                            size_t drop_fn,
                            int8_t* error_buffer,
                            size_t buffer_size);

// Label operations

/// Create a new JUCE Label with the specified text.
/// 
/// @param text The label text (UTF-8 bytes)
/// @param text_len Length of the text string
/// @param error_buffer Buffer to store error message if creation fails
/// @param buffer_size Size of the error buffer
/// @return Pointer to the created label, or nullptr on error
JuceComponent* create_label(const uint8_t* text, size_t text_len,
                           int8_t* error_buffer, size_t buffer_size);

/// Set the text of a label.
/// 
/// @param ptr Pointer to the label component
/// @param text The new label text (UTF-8 bytes)
/// @param text_len Length of the text string
void label_set_text(JuceComponent* ptr, const uint8_t* text, size_t text_len);

/// Set the font of a label.
/// 
/// @param ptr Pointer to the label component
/// @param font_size The font size in points
void label_set_font(JuceComponent* ptr, float font_size);

/// Set the text justification of a label.
/// 
/// @param ptr Pointer to the label component
/// @param justification Justification flags (JUCE Justification constants)
void label_set_justification(JuceComponent* ptr, int32_t justification);

/// Set whether a label is editable.
/// 
/// @param ptr Pointer to the label component
/// @param editable true to make editable, false to make read-only
void label_set_editable(JuceComponent* ptr, bool editable);

/// Set a text change callback for a label.
/// 
/// @param ptr Pointer to the label component
/// @param rust_closure Pointer to the Rust closure (as size_t)
/// @param invoke Function pointer to invoke the closure (as size_t)
/// @param drop_fn Function pointer to drop the closure (as size_t)
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t label_set_on_text_change(JuceComponent* ptr,
                                 size_t rust_closure,
                                 size_t invoke,
                                 size_t drop_fn,
                                 int8_t* error_buffer,
                                 size_t buffer_size);

// Slider operations

/// Create a new JUCE Slider with the specified style.
/// 
/// @param style The slider style (1=LinearHorizontal, 2=LinearVertical, etc.)
/// @param error_buffer Buffer to store error message if creation fails
/// @param buffer_size Size of the error buffer
/// @return Pointer to the created slider, or nullptr on error
JuceComponent* create_slider(int32_t style, int8_t* error_buffer, size_t buffer_size);

/// Set the range of a slider.
/// 
/// @param ptr Pointer to the slider component
/// @param min Minimum value
/// @param max Maximum value
/// @param interval Snapping interval (0 for continuous)
void slider_set_range(JuceComponent* ptr, double min, double max, double interval);

/// Set the value of a slider.
/// 
/// @param ptr Pointer to the slider component
/// @param value The new value
void slider_set_value(JuceComponent* ptr, double value);

/// Get the current value of a slider.
/// 
/// @param ptr Pointer to the slider component
/// @return The current slider value
double slider_get_value(const JuceComponent* ptr);

/// Set a value change callback for a slider.
/// 
/// @param ptr Pointer to the slider component
/// @param rust_closure Pointer to the Rust closure (as size_t)
/// @param invoke Function pointer to invoke the closure (as size_t)
/// @param drop_fn Function pointer to drop the closure (as size_t)
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t slider_set_on_value_change(JuceComponent* ptr,
                                   size_t rust_closure,
                                   size_t invoke,
                                   size_t drop_fn,
                                   int8_t* error_buffer,
                                   size_t buffer_size);

// ComboBox operations

/// Create a new JUCE ComboBox.
/// 
/// @param error_buffer Buffer to store error message if creation fails
/// @param buffer_size Size of the error buffer
/// @return Pointer to the created combo box, or nullptr on error
JuceComponent* create_combo_box(int8_t* error_buffer, size_t buffer_size);

/// Add an item to a combo box.
/// 
/// @param ptr Pointer to the combo box component
/// @param text The item text (UTF-8 bytes)
/// @param text_len Length of the text string
/// @param item_id The ID for this item (must be > 0)
void combo_box_add_item(JuceComponent* ptr, const uint8_t* text, size_t text_len, int32_t item_id);

/// Clear all items from a combo box.
/// 
/// @param ptr Pointer to the combo box component
void combo_box_clear(JuceComponent* ptr);

/// Set the selected item by ID.
/// 
/// @param ptr Pointer to the combo box component
/// @param item_id The ID of the item to select
void combo_box_set_selected_id(JuceComponent* ptr, int32_t item_id);

/// Set the selected item by index.
/// 
/// @param ptr Pointer to the combo box component
/// @param index The index of the item to select (0-based)
void combo_box_set_selected_index(JuceComponent* ptr, int32_t index);

/// Get the ID of the currently selected item.
/// 
/// @param ptr Pointer to the combo box component
/// @return The ID of the selected item, or 0 if no item is selected
int32_t combo_box_get_selected_id(const JuceComponent* ptr);

/// Set a change callback for a combo box.
/// 
/// @param ptr Pointer to the combo box component
/// @param rust_closure Pointer to the Rust closure (as size_t)
/// @param invoke Function pointer to invoke the closure (as size_t)
/// @param drop_fn Function pointer to drop the closure (as size_t)
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t combo_box_set_on_change(JuceComponent* ptr,
                                size_t rust_closure,
                                size_t invoke,
                                size_t drop_fn,
                                int8_t* error_buffer,
                                size_t buffer_size);

// TextEditor operations

/// Create a new JUCE TextEditor.
/// 
/// @param error_buffer Buffer to store error message if creation fails
/// @param buffer_size Size of the error buffer
/// @return Pointer to the created text editor, or nullptr on error
JuceComponent* create_text_editor(int8_t* error_buffer, size_t buffer_size);

/// Set the text of a text editor.
/// 
/// @param ptr Pointer to the text editor component
/// @param text The new text (UTF-8 bytes)
/// @param text_len Length of the text string
void text_editor_set_text(JuceComponent* ptr, const uint8_t* text, size_t text_len);

/// Get the text from a text editor.
/// 
/// @param ptr Pointer to the text editor component
/// @param buffer Buffer to store the text (UTF-8 bytes)
/// @param buffer_size Size of the buffer
/// @return The number of bytes written to the buffer (excluding null terminator)
size_t text_editor_get_text(const JuceComponent* ptr, uint8_t* buffer, size_t buffer_size);

/// Set whether a text editor is multiline.
/// 
/// @param ptr Pointer to the text editor component
/// @param multiline true for multiline, false for single line
void text_editor_set_multiline(JuceComponent* ptr, bool multiline);

/// Set whether a text editor is read-only.
/// 
/// @param ptr Pointer to the text editor component
/// @param readonly true for read-only, false for editable
void text_editor_set_readonly(JuceComponent* ptr, bool readonly);

/// Set a text change callback for a text editor.
/// 
/// @param ptr Pointer to the text editor component
/// @param rust_closure Pointer to the Rust closure (as size_t)
/// @param invoke Function pointer to invoke the closure (as size_t)
/// @param drop_fn Function pointer to drop the closure (as size_t)
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t text_editor_set_on_text_change(JuceComponent* ptr,
                                       size_t rust_closure,
                                       size_t invoke,
                                       size_t drop_fn,
                                       int8_t* error_buffer,
                                       size_t buffer_size);

// ToggleButton operations

/// Create a new JUCE ToggleButton with the specified text.
/// 
/// @param text The button text (UTF-8 bytes)
/// @param text_len Length of the text string
/// @param error_buffer Buffer to store error message if creation fails
/// @param buffer_size Size of the error buffer
/// @return Pointer to the created toggle button, or nullptr on error
JuceComponent* create_toggle_button(const uint8_t* text, size_t text_len,
                                    int8_t* error_buffer, size_t buffer_size);

/// Set the toggle state of a toggle button.
/// 
/// @param ptr Pointer to the toggle button component
/// @param state true for on/checked, false for off/unchecked
void toggle_button_set_toggle_state(JuceComponent* ptr, bool state);

/// Get the current toggle state of a toggle button.
/// 
/// @param ptr Pointer to the toggle button component
/// @return true if the button is on/checked, false if off/unchecked
bool toggle_button_get_toggle_state(const JuceComponent* ptr);

/// Set the radio group ID for a toggle button.
/// 
/// @param ptr Pointer to the toggle button component
/// @param group_id The radio group ID (0 to remove from groups)
void toggle_button_set_radio_group_id(JuceComponent* ptr, int32_t group_id);

/// Set the text of a toggle button.
/// 
/// @param ptr Pointer to the toggle button component
/// @param text The new button text (UTF-8 bytes)
/// @param text_len Length of the text string
void toggle_button_set_text(JuceComponent* ptr, const uint8_t* text, size_t text_len);

/// Set a click callback for a toggle button.
/// 
/// @param ptr Pointer to the toggle button component
/// @param rust_closure Pointer to the Rust closure (as size_t)
/// @param invoke Function pointer to invoke the closure (as size_t)
/// @param drop_fn Function pointer to drop the closure (as size_t)
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t toggle_button_set_on_click(JuceComponent* ptr,
                                   size_t rust_closure,
                                   size_t invoke,
                                   size_t drop_fn,
                                   int8_t* error_buffer,
                                   size_t buffer_size);

// Mouse event handling

/// Set a mouse listener for a component.
/// 
/// @param ptr Pointer to the component
/// @param listener_ptr Pointer to the Rust listener (as size_t)
/// @param mouse_down Function pointer for mouse down events (as size_t)
/// @param mouse_drag Function pointer for mouse drag events (as size_t)
/// @param mouse_up Function pointer for mouse up events (as size_t)
/// @param mouse_enter Function pointer for mouse enter events (as size_t)
/// @param mouse_exit Function pointer for mouse exit events (as size_t)
/// @param drop_fn Function pointer to drop the listener (as size_t)
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t component_set_mouse_listener(JuceComponent* ptr,
                                     size_t listener_ptr,
                                     size_t mouse_down,
                                     size_t mouse_drag,
                                     size_t mouse_up,
                                     size_t mouse_enter,
                                     size_t mouse_exit,
                                     size_t drop_fn,
                                     int8_t* error_buffer,
                                     size_t buffer_size);

// Keyboard event handling

/// Set whether a component wants keyboard focus.
/// 
/// @param ptr Pointer to the component
/// @param wants true to enable keyboard focus, false to disable
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t component_set_wants_keyboard_focus(JuceComponent* ptr,
                                           bool wants,
                                           int8_t* error_buffer,
                                           size_t buffer_size);

/// Set a keyboard listener for a component.
/// 
/// @param ptr Pointer to the component
/// @param listener_ptr Pointer to the Rust listener (as size_t)
/// @param key_pressed Function pointer for key pressed events (as size_t)
/// @param key_state_changed Function pointer for key state changed events (as size_t)
/// @param focus_gained Function pointer for focus gained events (as size_t)
/// @param focus_lost Function pointer for focus lost events (as size_t)
/// @param drop_fn Function pointer to drop the listener (as size_t)
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t component_set_key_listener(JuceComponent* ptr,
                                   size_t listener_ptr,
                                   size_t key_pressed,
                                   size_t key_state_changed,
                                   size_t focus_gained,
                                   size_t focus_lost,
                                   size_t drop_fn,
                                   int8_t* error_buffer,
                                   size_t buffer_size);

// Timer operations

/// Create a new JUCE Timer with a callback.
/// 
/// @param rust_closure Pointer to the Rust closure (as size_t)
/// @param invoke Function pointer to invoke the closure (as size_t)
/// @param drop_fn Function pointer to drop the closure (as size_t)
/// @param error_buffer Buffer to store error message if creation fails
/// @param buffer_size Size of the error buffer
/// @return Pointer to the created timer, or nullptr on error
JuceTimer* create_timer(size_t rust_closure,
                       size_t invoke,
                       size_t drop_fn,
                       int8_t* error_buffer,
                       size_t buffer_size);

/// Delete a JUCE Timer and free its resources.
/// 
/// @param ptr Pointer to the timer to delete
void delete_timer(JuceTimer* ptr);

/// Start a timer with the specified interval.
/// 
/// @param ptr Pointer to the timer
/// @param interval_ms The interval in milliseconds
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t timer_start(JuceTimer* ptr,
                   int32_t interval_ms,
                   int8_t* error_buffer,
                   size_t buffer_size);

/// Stop a timer.
/// 
/// @param ptr Pointer to the timer
void timer_stop(JuceTimer* ptr);

/// Check if a timer is currently running.
/// 
/// @param ptr Pointer to the timer
/// @return true if the timer is running, false otherwise
bool timer_is_running(const JuceTimer* ptr);

// Colour operations

/// Create a new JUCE Colour from RGBA values.
/// 
/// @param r Red component (0-255)
/// @param g Green component (0-255)
/// @param b Blue component (0-255)
/// @param a Alpha component (0-255)
/// @param error_buffer Buffer to store error message if creation fails
/// @param buffer_size Size of the error buffer
/// @return Pointer to the created colour, or nullptr on error
JuceColour* create_colour_rgba(uint8_t r, uint8_t g, uint8_t b, uint8_t a,
                               int8_t* error_buffer, size_t buffer_size);

/// Create a new JUCE Colour from a hexadecimal string.
/// 
/// @param hex Hexadecimal color string (UTF-8 bytes)
/// @param hex_len Length of the hex string
/// @param error_buffer Buffer to store error message if creation fails
/// @param buffer_size Size of the error buffer
/// @return Pointer to the created colour, or nullptr on error
JuceColour* create_colour_from_hex(const uint8_t* hex, size_t hex_len,
                                   int8_t* error_buffer, size_t buffer_size);

/// Delete a JUCE Colour and free its resources.
/// 
/// @param ptr Pointer to the colour to delete
void delete_colour(JuceColour* ptr);

/// Convert a colour to a hexadecimal string.
/// 
/// @param ptr Pointer to the colour
/// @param buffer Buffer to store the hex string (UTF-8 bytes)
/// @param buffer_size Size of the buffer
/// @return The number of bytes written to the buffer (excluding null terminator)
size_t colour_to_hex(const JuceColour* ptr, uint8_t* buffer, size_t buffer_size);

/// Create a new colour with a different alpha value.
/// 
/// @param ptr Pointer to the colour
/// @param alpha New alpha value (0.0 = transparent, 1.0 = opaque)
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return Pointer to the new colour, or nullptr on error
JuceColour* colour_with_alpha(const JuceColour* ptr, float alpha,
                              int8_t* error_buffer, size_t buffer_size);

/// Create a brighter version of a colour.
/// 
/// @param ptr Pointer to the colour
/// @param amount Amount to brighten (0.0 = no change, 1.0 = maximum)
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return Pointer to the new colour, or nullptr on error
JuceColour* colour_brighter(const JuceColour* ptr, float amount,
                            int8_t* error_buffer, size_t buffer_size);

/// Create a darker version of a colour.
/// 
/// @param ptr Pointer to the colour
/// @param amount Amount to darken (0.0 = no change, 1.0 = maximum)
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return Pointer to the new colour, or nullptr on error
JuceColour* colour_darker(const JuceColour* ptr, float amount,
                          int8_t* error_buffer, size_t buffer_size);

/// Create a colour interpolated between two colours.
/// 
/// @param ptr1 Pointer to the first colour
/// @param ptr2 Pointer to the second colour
/// @param proportion Interpolation amount (0.0 = first colour, 1.0 = second colour)
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return Pointer to the new colour, or nullptr on error
JuceColour* colour_interpolated_with(const JuceColour* ptr1, const JuceColour* ptr2,
                                     float proportion, int8_t* error_buffer,
                                     size_t buffer_size);

// Font operations

/// Create a new JUCE Font with the specified size.
/// 
/// @param size Font size in points
/// @param error_buffer Buffer to store error message if creation fails
/// @param buffer_size Size of the error buffer
/// @return Pointer to the created font, or nullptr on error
JuceFont* create_font(float size, int8_t* error_buffer, size_t buffer_size);

/// Create a new JUCE Font with a specific typeface and size.
/// 
/// @param typeface Typeface name (UTF-8 bytes)
/// @param typeface_len Length of the typeface name
/// @param size Font size in points
/// @param error_buffer Buffer to store error message if creation fails
/// @param buffer_size Size of the error buffer
/// @return Pointer to the created font, or nullptr on error
JuceFont* create_font_with_typeface(const uint8_t* typeface, size_t typeface_len, float size,
                                    int8_t* error_buffer, size_t buffer_size);

/// Delete a JUCE Font and free its resources.
/// 
/// @param ptr Pointer to the font to delete
void delete_font(JuceFont* ptr);

/// Set whether the font is bold.
/// 
/// @param ptr Pointer to the font
/// @param bold true for bold, false for normal weight
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t font_set_bold(JuceFont* ptr, bool bold, int8_t* error_buffer, size_t buffer_size);

/// Set whether the font is italic.
/// 
/// @param ptr Pointer to the font
/// @param italic true for italic, false for normal style
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t font_set_italic(JuceFont* ptr, bool italic, int8_t* error_buffer, size_t buffer_size);

/// Set whether the font is underlined.
/// 
/// @param ptr Pointer to the font
/// @param underline true for underlined, false for no underline
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t font_set_underline(JuceFont* ptr, bool underline, int8_t* error_buffer, size_t buffer_size);

/// Get the width of a string when rendered with this font.
/// 
/// @param ptr Pointer to the font
/// @param text The text to measure (UTF-8 bytes)
/// @param text_len Length of the text string
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return The width in pixels, or -1 on error
int32_t font_get_string_width(const JuceFont* ptr, const uint8_t* text, size_t text_len,
                              int8_t* error_buffer, size_t buffer_size);

/// Get the height of the font.
/// 
/// @param ptr Pointer to the font
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return The height in pixels, or -1 on error
int32_t font_get_height(const JuceFont* ptr, int8_t* error_buffer, size_t buffer_size);

/// Get the count of available typefaces on the system.
/// 
/// @return The number of available typefaces, or -1 on error
int32_t font_get_typeface_count();

/// Get the name of a typeface by index.
/// 
/// @param index Index of the typeface (0-based)
/// @param buffer Buffer to store the typeface name (UTF-8 bytes)
/// @param buffer_size Size of the buffer
/// @return The number of bytes written to the buffer (excluding null terminator),
///         or 0 if the index is out of range
int32_t font_get_typeface_name(int32_t index, uint8_t* buffer, size_t buffer_size);

// Image operations

/// Create a new JUCE Image with the specified format and dimensions.
/// 
/// @param format Image format (1=RGB, 2=ARGB, 3=SingleChannel)
/// @param width Width of the image in pixels
/// @param height Height of the image in pixels
/// @param error_buffer Buffer to store error message if creation fails
/// @param buffer_size Size of the error buffer
/// @return Pointer to the created image, or nullptr on error
JuceImage* create_image(int32_t format, int32_t width, int32_t height,
                        int8_t* error_buffer, size_t buffer_size);

/// Load a JUCE Image from a file.
/// 
/// @param path File path (UTF-8 bytes)
/// @param path_len Length of the path string
/// @param error_buffer Buffer to store error message if loading fails
/// @param buffer_size Size of the error buffer
/// @return Pointer to the loaded image, or nullptr on error
JuceImage* load_image_from_file(const uint8_t* path, size_t path_len,
                                int8_t* error_buffer, size_t buffer_size);

/// Save a JUCE Image to a file.
/// 
/// @param ptr Pointer to the image
/// @param path File path (UTF-8 bytes)
/// @param path_len Length of the path string
/// @param error_buffer Buffer to store error message if saving fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t save_image_to_file(const JuceImage* ptr, const uint8_t* path, size_t path_len,
                           int8_t* error_buffer, size_t buffer_size);

/// Delete a JUCE Image and free its resources.
/// 
/// @param ptr Pointer to the image to delete
void delete_image(JuceImage* ptr);

/// Get a graphics context for drawing to an image.
/// 
/// @param ptr Pointer to the image
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return Pointer to the graphics context, or nullptr on error
JuceGraphics* image_get_graphics_context(JuceImage* ptr,
                                         int8_t* error_buffer, size_t buffer_size);

/// Apply a blur effect to an image.
/// 
/// @param ptr Pointer to the image
/// @param radius Blur radius in pixels
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t image_apply_blur(JuceImage* ptr, float radius,
                        int8_t* error_buffer, size_t buffer_size);

/// Get the width of an image.
/// 
/// @param ptr Pointer to the image
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return The width in pixels, or -1 on error
int32_t image_get_width(const JuceImage* ptr, int8_t* error_buffer, size_t buffer_size);

/// Get the height of an image.
/// 
/// @param ptr Pointer to the image
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return The height in pixels, or -1 on error
int32_t image_get_height(const JuceImage* ptr, int8_t* error_buffer, size_t buffer_size);

// ============================================================================
// Path operations
// ============================================================================

/// Create a new JUCE Path.
/// 
/// @param error_buffer Buffer to store error message if creation fails
/// @param buffer_size Size of the error buffer
/// @return Pointer to the created path, or nullptr on error
JucePath* create_path(int8_t* error_buffer, size_t buffer_size);

/// Delete a JUCE Path and free its resources.
/// 
/// @param ptr Pointer to the path to delete
void delete_path(JucePath* ptr);

/// Start a new sub-path at the specified position.
/// 
/// @param ptr Pointer to the path
/// @param x X coordinate of the starting point
/// @param y Y coordinate of the starting point
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t path_start_new_sub_path(JucePath* ptr, float x, float y,
                                 int8_t* error_buffer, size_t buffer_size);

/// Add a line from the current position to the specified point.
/// 
/// @param ptr Pointer to the path
/// @param x X coordinate of the end point
/// @param y Y coordinate of the end point
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t path_line_to(JucePath* ptr, float x, float y,
                     int8_t* error_buffer, size_t buffer_size);

/// Add a quadratic bezier curve from the current position.
/// 
/// @param ptr Pointer to the path
/// @param cx X coordinate of the control point
/// @param cy Y coordinate of the control point
/// @param x X coordinate of the end point
/// @param y Y coordinate of the end point
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t path_quadratic_to(JucePath* ptr, float cx, float cy, float x, float y,
                          int8_t* error_buffer, size_t buffer_size);

/// Add a cubic bezier curve from the current position.
/// 
/// @param ptr Pointer to the path
/// @param cx1 X coordinate of the first control point
/// @param cy1 Y coordinate of the first control point
/// @param cx2 X coordinate of the second control point
/// @param cy2 Y coordinate of the second control point
/// @param x X coordinate of the end point
/// @param y Y coordinate of the end point
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t path_cubic_to(JucePath* ptr, float cx1, float cy1, float cx2, float cy2,
                      float x, float y, int8_t* error_buffer, size_t buffer_size);

/// Add a rectangle to the path.
/// 
/// @param ptr Pointer to the path
/// @param x X coordinate of the top-left corner
/// @param y Y coordinate of the top-left corner
/// @param width Width of the rectangle
/// @param height Height of the rectangle
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t path_add_rectangle(JucePath* ptr, float x, float y, float width, float height,
                           int8_t* error_buffer, size_t buffer_size);

/// Add an ellipse to the path.
/// 
/// @param ptr Pointer to the path
/// @param x X coordinate of the bounding rectangle
/// @param y Y coordinate of the bounding rectangle
/// @param width Width of the bounding rectangle
/// @param height Height of the bounding rectangle
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t path_add_ellipse(JucePath* ptr, float x, float y, float width, float height,
                         int8_t* error_buffer, size_t buffer_size);

/// Add an arc to the path.
/// 
/// @param ptr Pointer to the path
/// @param x X coordinate of the bounding rectangle
/// @param y Y coordinate of the bounding rectangle
/// @param width Width of the bounding rectangle
/// @param height Height of the bounding rectangle
/// @param start_angle Starting angle in radians
/// @param end_angle Ending angle in radians
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t path_add_arc(JucePath* ptr, float x, float y, float width, float height,
                     float start_angle, float end_angle,
                     int8_t* error_buffer, size_t buffer_size);

/// Close the current sub-path by adding a line back to its starting point.
/// 
/// @param ptr Pointer to the path
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t path_close_sub_path(JucePath* ptr, int8_t* error_buffer, size_t buffer_size);

/// Apply a transformation to the path.
/// 
/// @param ptr Pointer to the path
/// @param transform Pointer to the transformation to apply
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t path_apply_transform(JucePath* ptr, const JuceAffineTransform* transform,
                             int8_t* error_buffer, size_t buffer_size);

// ============================================================================
// AffineTransform operations
// ============================================================================

/// Create an identity transformation.
/// 
/// @param error_buffer Buffer to store error message if creation fails
/// @param buffer_size Size of the error buffer
/// @return Pointer to the created transform, or nullptr on error
JuceAffineTransform* create_affine_transform_identity(int8_t* error_buffer, size_t buffer_size);

/// Create a translation transformation.
/// 
/// @param dx Translation distance in X direction
/// @param dy Translation distance in Y direction
/// @param error_buffer Buffer to store error message if creation fails
/// @param buffer_size Size of the error buffer
/// @return Pointer to the created transform, or nullptr on error
JuceAffineTransform* create_affine_transform_translation(float dx, float dy,
                                                         int8_t* error_buffer, size_t buffer_size);

/// Create a rotation transformation.
/// 
/// @param angle_radians Rotation angle in radians
/// @param error_buffer Buffer to store error message if creation fails
/// @param buffer_size Size of the error buffer
/// @return Pointer to the created transform, or nullptr on error
JuceAffineTransform* create_affine_transform_rotation(float angle_radians,
                                                      int8_t* error_buffer, size_t buffer_size);

/// Create a scaling transformation.
/// 
/// @param sx Scale factor in X direction
/// @param sy Scale factor in Y direction
/// @param error_buffer Buffer to store error message if creation fails
/// @param buffer_size Size of the error buffer
/// @return Pointer to the created transform, or nullptr on error
JuceAffineTransform* create_affine_transform_scale(float sx, float sy,
                                                   int8_t* error_buffer, size_t buffer_size);

/// Delete a JUCE AffineTransform and free its resources.
/// 
/// @param ptr Pointer to the transform to delete
void delete_affine_transform(JuceAffineTransform* ptr);

/// Compose two transformations (this followed by other).
/// 
/// @param ptr Pointer to the first transform
/// @param other Pointer to the second transform to apply after the first
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return Pointer to the new composed transform, or nullptr on error
JuceAffineTransform* affine_transform_followed_by(const JuceAffineTransform* ptr,
                                                   const JuceAffineTransform* other,
                                                   int8_t* error_buffer, size_t buffer_size);

// ============================================================================
// FlexBox operations
// ============================================================================

/// Create a new JUCE FlexBox.
/// 
/// @param error_buffer Buffer to store error message if creation fails
/// @param buffer_size Size of the error buffer
/// @return Pointer to the created flexbox, or nullptr on error
JuceFlexBox* create_flexbox(int8_t* error_buffer, size_t buffer_size);

/// Delete a JUCE FlexBox and free its resources.
/// 
/// @param ptr Pointer to the flexbox to delete
void delete_flexbox(JuceFlexBox* ptr);

/// Set the flex direction.
/// 
/// @param ptr Pointer to the flexbox
/// @param direction Direction value (0=Row, 1=Column, 2=RowReverse, 3=ColumnReverse)
void flexbox_set_direction(JuceFlexBox* ptr, int32_t direction);

/// Set the flex wrap behavior.
/// 
/// @param ptr Pointer to the flexbox
/// @param wrap Wrap value (0=NoWrap, 1=Wrap, 2=WrapReverse)
void flexbox_set_wrap(JuceFlexBox* ptr, int32_t wrap);

/// Set the justify content property.
/// 
/// @param ptr Pointer to the flexbox
/// @param justify Justify value (0=FlexStart, 1=FlexEnd, 2=Center, 3=SpaceBetween, 4=SpaceAround)
void flexbox_set_justify_content(JuceFlexBox* ptr, int32_t justify);

/// Set the align content property.
/// 
/// @param ptr Pointer to the flexbox
/// @param align Align value (0=FlexStart, 1=FlexEnd, 2=Center, 3=SpaceBetween, 4=SpaceAround, 5=Stretch)
void flexbox_set_align_content(JuceFlexBox* ptr, int32_t align);

/// Set the align items property.
/// 
/// @param ptr Pointer to the flexbox
/// @param align Align value (0=FlexStart, 1=FlexEnd, 2=Center, 3=Stretch)
void flexbox_set_align_items(JuceFlexBox* ptr, int32_t align);

/// Add an item to the flexbox.
/// 
/// @param ptr Pointer to the flexbox
/// @param component Pointer to the component to add
/// @param flex_grow Flex grow factor
/// @param flex_shrink Flex shrink factor
/// @param flex_basis Flex basis in pixels
/// @param min_width Minimum width in pixels
/// @param min_height Minimum height in pixels
/// @param max_width Maximum width in pixels
/// @param max_height Maximum height in pixels
/// @param margin_top Top margin in pixels
/// @param margin_right Right margin in pixels
/// @param margin_bottom Bottom margin in pixels
/// @param margin_left Left margin in pixels
void flexbox_add_item(JuceFlexBox* ptr,
                     JuceComponent* component,
                     float flex_grow,
                     float flex_shrink,
                     float flex_basis,
                     float min_width,
                     float min_height,
                     float max_width,
                     float max_height,
                     float margin_top,
                     float margin_right,
                     float margin_bottom,
                     float margin_left);

/// Perform the flex layout within the specified bounds.
/// 
/// @param ptr Pointer to the flexbox
/// @param x X coordinate of the layout area
/// @param y Y coordinate of the layout area
/// @param width Width of the layout area
/// @param height Height of the layout area
void flexbox_perform_layout(JuceFlexBox* ptr, int32_t x, int32_t y, int32_t width, int32_t height);

// ============================================================================
// DocumentWindow Operations
// ============================================================================

/// Create a new JUCE DocumentWindow with the specified title.
/// 
/// @param title The window title (UTF-8 bytes)
/// @param title_len Length of the title string
/// @param error_buffer Buffer to store error message if creation fails
/// @param buffer_size Size of the error buffer
/// @return Pointer to the created document window, or null on error
JuceComponent* create_document_window(const uint8_t* title, size_t title_len,
                                      int8_t* error_buffer, size_t buffer_size);

/// Set the content component for a document window, transferring ownership.
/// 
/// @param ptr Pointer to the document window
/// @param content Pointer to the content component
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t document_window_set_content_owned(JuceComponent* ptr, JuceComponent* content,
                                          int8_t* error_buffer, size_t buffer_size);

/// Set the name (title) of a document window.
/// 
/// @param ptr Pointer to the document window
/// @param name The new window title (UTF-8 bytes)
/// @param name_len Length of the name string
void document_window_set_name(JuceComponent* ptr, const uint8_t* name, size_t name_len);

/// Set a close callback for a document window.
/// 
/// @param ptr Pointer to the document window
/// @param rust_closure Pointer to the Rust closure (as usize)
/// @param invoke Function pointer to invoke the closure (as usize)
/// @param drop_fn Function pointer to drop the closure (as usize)
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t document_window_set_on_close(JuceComponent* ptr,
                                     size_t rust_closure,
                                     size_t invoke,
                                     size_t drop_fn,
                                     int8_t* error_buffer,
                                     size_t buffer_size);

// ============================================================================
// ResizableWindow Operations
// ============================================================================

/// Create a new JUCE ResizableWindow with the specified title.
/// 
/// @param title The window title (UTF-8 bytes)
/// @param title_len Length of the title string
/// @param error_buffer Buffer to store error message if creation fails
/// @param buffer_size Size of the error buffer
/// @return Pointer to the created resizable window, or null on error
JuceComponent* create_resizable_window(const uint8_t* title, size_t title_len,
                                       int8_t* error_buffer, size_t buffer_size);

/// Enable or disable user resizing of the window.
/// 
/// @param ptr Pointer to the resizable window
/// @param resizable Whether the window should be resizable
void resizable_window_set_resizable(JuceComponent* ptr, bool resizable);

/// Set the minimum and maximum size constraints for the window.
/// 
/// @param ptr Pointer to the resizable window
/// @param min_width Minimum window width in pixels
/// @param min_height Minimum window height in pixels
/// @param max_width Maximum window width in pixels
/// @param max_height Maximum window height in pixels
void resizable_window_set_resize_limits(JuceComponent* ptr,
                                        int32_t min_width, int32_t min_height,
                                        int32_t max_width, int32_t max_height);

/// Set a resize callback for a resizable window.
/// 
/// @param ptr Pointer to the resizable window
/// @param rust_closure Pointer to the Rust closure (as usize)
/// @param invoke Function pointer to invoke the closure (as usize)
/// @param drop_fn Function pointer to drop the closure (as usize)
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t resizable_window_set_on_resized(JuceComponent* ptr,
                                        size_t rust_closure,
                                        size_t invoke,
                                        size_t drop_fn,
                                        int8_t* error_buffer,
                                        size_t buffer_size);

// ============================================================================
// Viewport Operations
// ============================================================================

/// Create a new JUCE Viewport.
/// 
/// @param error_buffer Buffer to store error message if creation fails
/// @param buffer_size Size of the error buffer
/// @return Pointer to the created viewport, or null on error
JuceComponent* create_viewport(int8_t* error_buffer, size_t buffer_size);

/// Set the component to be viewed in the viewport, transferring ownership.
/// 
/// @param ptr Pointer to the viewport
/// @param component Pointer to the component to view
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t viewport_set_viewed_component(JuceComponent* ptr, JuceComponent* component,
                                      int8_t* error_buffer, size_t buffer_size);

/// Set the scroll position of the viewport.
/// 
/// @param ptr Pointer to the viewport
/// @param x X coordinate of the top-left corner of the visible area
/// @param y Y coordinate of the top-left corner of the visible area
void viewport_set_view_position(JuceComponent* ptr, int32_t x, int32_t y);

/// Set whether scrollbars are shown.
/// 
/// @param ptr Pointer to the viewport
/// @param vertical Whether to show the vertical scrollbar
/// @param horizontal Whether to show the horizontal scrollbar
void viewport_set_scrollbars_shown(JuceComponent* ptr, bool vertical, bool horizontal);

/// Set a callback to be invoked when the visible area changes.
/// 
/// @param ptr Pointer to the viewport
/// @param rust_closure Pointer to the Rust closure (as usize)
/// @param invoke Function pointer to invoke the closure (as usize)
/// @param drop_fn Function pointer to drop the closure (as usize)
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t viewport_set_on_visible_area_changed(JuceComponent* ptr,
                                             size_t rust_closure,
                                             size_t invoke,
                                             size_t drop_fn,
                                             int8_t* error_buffer,
                                             size_t buffer_size);

// ============================================================================
// TabbedComponent Operations
// ============================================================================

/// Create a new JUCE TabbedComponent with the specified orientation.
/// 
/// @param orientation Tab orientation (0=Top, 1=Bottom, 2=Left, 3=Right)
/// @param error_buffer Buffer to store error message if creation fails
/// @param buffer_size Size of the error buffer
/// @return Pointer to the created tabbed component, or null on error
JuceComponent* create_tabbed_component(int32_t orientation,
                                       int8_t* error_buffer, size_t buffer_size);

/// Add a tab to a tabbed component.
/// 
/// @param ptr Pointer to the tabbed component
/// @param name The tab name (UTF-8 bytes)
/// @param name_len Length of the name string
/// @param colour Pointer to the tab background colour
/// @param content Pointer to the content component
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t tabbed_component_add_tab(JuceComponent* ptr, const uint8_t* name, size_t name_len,
                                 const JuceColour* colour, JuceComponent* content,
                                 int8_t* error_buffer, size_t buffer_size);

/// Remove a tab from a tabbed component.
/// 
/// @param ptr Pointer to the tabbed component
/// @param index The index of the tab to remove (0-based)
void tabbed_component_remove_tab(JuceComponent* ptr, int32_t index);

/// Set the current tab index.
/// 
/// @param ptr Pointer to the tabbed component
/// @param index The index of the tab to select (0-based)
void tabbed_component_set_current_tab_index(JuceComponent* ptr, int32_t index);

/// Set a callback to be invoked when the current tab changes.
/// 
/// @param ptr Pointer to the tabbed component
/// @param rust_closure Pointer to the Rust closure (as usize)
/// @param invoke Function pointer to invoke the closure (as usize)
/// @param drop_fn Function pointer to drop the closure (as usize)
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t tabbed_component_set_on_tab_changed(JuceComponent* ptr,
                                            size_t rust_closure,
                                            size_t invoke,
                                            size_t drop_fn,
                                            int8_t* error_buffer,
                                            size_t buffer_size);

// ============================================================================
// ListBox Operations
// ============================================================================

/// Create a new JUCE ListBox.
/// 
/// @param error_buffer Buffer to store error message if creation fails
/// @param buffer_size Size of the error buffer
/// @return Pointer to the created list box, or null on error
JuceComponent* create_list_box(int8_t* error_buffer, size_t buffer_size);

/// Set the model for a list box.
/// 
/// @param ptr Pointer to the list box component
/// @param model_ptr Pointer to the Rust model (as size_t)
/// @param get_num_rows Function pointer to get the number of rows (as size_t)
/// @param paint_item Function pointer to paint an item (as size_t)
/// @param selection_changed Function pointer for selection changes (as size_t)
/// @param drop_fn Function pointer to drop the model (as size_t)
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t list_box_set_model(JuceComponent* ptr,
                           size_t model_ptr,
                           size_t get_num_rows,
                           size_t paint_item,
                           size_t selection_changed,
                           size_t drop_fn,
                           int8_t* error_buffer,
                           size_t buffer_size);

/// Update the content of a list box.
/// 
/// @param ptr Pointer to the list box component
void list_box_update_content(JuceComponent* ptr);

// ============================================================================
// TreeView Operations
// ============================================================================

/// Create a new JUCE TreeView.
/// 
/// @param error_buffer Buffer to store error message if creation fails
/// @param buffer_size Size of the error buffer
/// @return Pointer to the created tree view, or null on error
JuceComponent* create_tree_view(int8_t* error_buffer, size_t buffer_size);

/// Set the root item for a tree view.
/// 
/// @param ptr Pointer to the tree view component
/// @param item_ptr Pointer to the Rust item (as size_t)
/// @param get_num_sub_items Function pointer to get the number of sub-items (as size_t)
/// @param get_sub_item Function pointer to get a sub-item (as size_t)
/// @param paint_item Function pointer to paint an item (as size_t)
/// @param item_clicked Function pointer for item clicks (as size_t)
/// @param drop_fn Function pointer to drop the item (as size_t)
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t tree_view_set_root_item(JuceComponent* ptr,
                                size_t item_ptr,
                                size_t get_num_sub_items,
                                size_t get_sub_item,
                                size_t paint_item,
                                size_t item_clicked,
                                size_t drop_fn,
                                int8_t* error_buffer,
                                size_t buffer_size);

// ============================================================================
// AlertWindow Operations
// ============================================================================

/// Show a synchronous message box.
/// 
/// This displays a simple message box with an OK button and blocks until
/// the user dismisses it.
/// 
/// @param title The dialog title (UTF-8 bytes)
/// @param title_len Length of the title string
/// @param message The message text (UTF-8 bytes)
/// @param message_len Length of the message string
void alert_window_show_message_box(const uint8_t* title,
                                   size_t title_len,
                                   const uint8_t* message,
                                   size_t message_len);

/// Show an asynchronous message box with a callback.
/// 
/// This displays a message box with an OK button and returns immediately.
/// When the user dismisses the dialog, the callback is invoked.
/// 
/// @param title The dialog title (UTF-8 bytes)
/// @param title_len Length of the title string
/// @param message The message text (UTF-8 bytes)
/// @param message_len Length of the message string
/// @param rust_closure Pointer to the Rust closure (as size_t)
/// @param invoke Function pointer to invoke the closure (as size_t)
/// @param drop_fn Function pointer to drop the closure (as size_t)
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t alert_window_show_message_box_async(const uint8_t* title,
                                            size_t title_len,
                                            const uint8_t* message,
                                            size_t message_len,
                                            size_t rust_closure,
                                            size_t invoke,
                                            size_t drop_fn,
                                            int8_t* error_buffer,
                                            size_t buffer_size);

/// Show an OK/Cancel confirmation dialog with a callback.
/// 
/// This displays a dialog with OK and Cancel buttons and returns immediately.
/// When the user clicks a button, the callback is invoked with true for OK
/// or false for Cancel.
/// 
/// @param title The dialog title (UTF-8 bytes)
/// @param title_len Length of the title string
/// @param message The message text (UTF-8 bytes)
/// @param message_len Length of the message string
/// @param rust_closure Pointer to the Rust closure (as size_t)
/// @param invoke Function pointer to invoke the closure (as size_t)
/// @param drop_fn Function pointer to drop the closure (as size_t)
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t alert_window_show_ok_cancel_box(const uint8_t* title,
                                        size_t title_len,
                                        const uint8_t* message,
                                        size_t message_len,
                                        size_t rust_closure,
                                        size_t invoke,
                                        size_t drop_fn,
                                        int8_t* error_buffer,
                                        size_t buffer_size);

// ============================================================================
// FileChooser Operations
// ============================================================================

/// Create a new JUCE FileChooser.
/// 
/// @param title The dialog title (UTF-8 bytes)
/// @param title_len Length of the title string
/// @param initial_dir The initial directory path (UTF-8 bytes)
/// @param initial_dir_len Length of the initial directory string
/// @param filters File filters in format "*.ext1;*.ext2" (UTF-8 bytes)
/// @param filters_len Length of the filters string
/// @param error_buffer Buffer to store error message if creation fails
/// @param buffer_size Size of the error buffer
/// @return Pointer to the created file chooser, or null on error
JuceFileChooser* create_file_chooser(const uint8_t* title,
                                     size_t title_len,
                                     const uint8_t* initial_dir,
                                     size_t initial_dir_len,
                                     const uint8_t* filters,
                                     size_t filters_len,
                                     int8_t* error_buffer,
                                     size_t buffer_size);

/// Delete a JUCE FileChooser and free its resources.
/// 
/// @param ptr Pointer to the file chooser to delete
void delete_file_chooser(JuceFileChooser* ptr);

/// Browse for a file to open.
/// 
/// This displays a native file open dialog and returns immediately.
/// When the user selects a file or cancels, the callback is invoked.
/// 
/// @param ptr Pointer to the file chooser
/// @param rust_closure Pointer to the Rust closure (as size_t)
/// @param invoke Function pointer to invoke the closure (as size_t)
/// @param drop_fn Function pointer to drop the closure (as size_t)
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t file_chooser_browse_for_file_to_open(JuceFileChooser* ptr,
                                             size_t rust_closure,
                                             size_t invoke,
                                             size_t drop_fn,
                                             int8_t* error_buffer,
                                             size_t buffer_size);

/// Browse for a file to save.
/// 
/// This displays a native file save dialog and returns immediately.
/// When the user selects a file or cancels, the callback is invoked.
/// 
/// @param ptr Pointer to the file chooser
/// @param rust_closure Pointer to the Rust closure (as size_t)
/// @param invoke Function pointer to invoke the closure (as size_t)
/// @param drop_fn Function pointer to drop the closure (as size_t)
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t file_chooser_browse_for_file_to_save(JuceFileChooser* ptr,
                                             size_t rust_closure,
                                             size_t invoke,
                                             size_t drop_fn,
                                             int8_t* error_buffer,
                                             size_t buffer_size);

// ==============================================================================
// Drawable Operations
// ==============================================================================

/// Create a Drawable from SVG data.
/// 
/// @param svg_data The SVG data (UTF-8 bytes)
/// @param svg_len Length of the SVG data
/// @param error_buffer Buffer to store error message if creation fails
/// @param buffer_size Size of the error buffer
/// @return Pointer to the created drawable, or null on error
JuceDrawable* create_drawable_from_svg(const uint8_t* svg_data,
                                       size_t svg_len,
                                       int8_t* error_buffer,
                                       size_t buffer_size);

/// Create a Drawable from image data.
/// 
/// @param image_data The image data bytes (PNG, JPEG, etc.)
/// @param data_len Length of the image data
/// @param error_buffer Buffer to store error message if creation fails
/// @param buffer_size Size of the error buffer
/// @return Pointer to the created drawable, or null on error
JuceDrawable* create_drawable_from_image_data(const uint8_t* image_data,
                                              size_t data_len,
                                              int8_t* error_buffer,
                                              size_t buffer_size);

/// Delete a JUCE Drawable and free its resources.
/// 
/// @param ptr Pointer to the drawable to delete
void delete_drawable(JuceDrawable* ptr);

/// Draw a drawable to a Graphics context.
/// 
/// @param ptr Pointer to the drawable
/// @param g Pointer to the Graphics context
/// @param opacity Opacity to draw with (0.0 = transparent, 1.0 = opaque)
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t drawable_draw(const JuceDrawable* ptr,
                      JuceGraphics* g,
                      float opacity,
                      int8_t* error_buffer,
                      size_t buffer_size);

/// Set the drawable's transform to fit within bounds.
/// 
/// @param ptr Pointer to the drawable
/// @param x X coordinate of the bounding rectangle
/// @param y Y coordinate of the bounding rectangle
/// @param width Width of the bounding rectangle
/// @param height Height of the bounding rectangle
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t drawable_set_transform_to_fit(JuceDrawable* ptr,
                                      float x,
                                      float y,
                                      float width,
                                      float height,
                                      int8_t* error_buffer,
                                      size_t buffer_size);

// ==============================================================================
// DrawableButton Operations
// ==============================================================================

/// Create a new JUCE DrawableButton.
/// 
/// @param name The button name (UTF-8 bytes)
/// @param name_len Length of the name string
/// @param error_buffer Buffer to store error message if creation fails
/// @param buffer_size Size of the error buffer
/// @return Pointer to the created drawable button, or null on error
JuceComponent* create_drawable_button(const uint8_t* name,
                                      size_t name_len,
                                      int8_t* error_buffer,
                                      size_t buffer_size);

/// Set the images for a DrawableButton.
/// 
/// @param ptr Pointer to the drawable button component
/// @param normal Pointer to the normal state drawable (required)
/// @param over Pointer to the hover state drawable (optional, can be null)
/// @param down Pointer to the pressed state drawable (optional, can be null)
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t drawable_button_set_images(JuceComponent* ptr,
                                   const JuceDrawable* normal,
                                   const JuceDrawable* over,
                                   const JuceDrawable* down,
                                   int8_t* error_buffer,
                                   size_t buffer_size);

// LookAndFeel operations

/// Create a new JUCE LookAndFeel_V4.
/// 
/// @param error_buffer Buffer to store error message if creation fails
/// @param buffer_size Size of the error buffer
/// @return Pointer to the created LookAndFeel, or null on error
JuceLookAndFeel* create_lookandfeel_v4(int8_t* error_buffer, size_t buffer_size);

/// Delete a JUCE LookAndFeel and free its resources.
/// 
/// @param ptr Pointer to the LookAndFeel to delete
void delete_lookandfeel(JuceLookAndFeel* ptr);

/// Set a color for a specific color ID in a LookAndFeel.
/// 
/// @param ptr Pointer to the LookAndFeel
/// @param colour_id The JUCE color ID to set
/// @param colour Pointer to the color to use
void lookandfeel_set_colour(JuceLookAndFeel* ptr, int32_t colour_id, const JuceColour* colour);

/// Find the color for a specific color ID in a LookAndFeel.
/// 
/// @param ptr Pointer to the LookAndFeel
/// @param colour_id The JUCE color ID to query
/// @return Pointer to the color for the given ID
const JuceColour* lookandfeel_find_colour(const JuceLookAndFeel* ptr, int32_t colour_id);

/// Set the LookAndFeel for a component.
/// 
/// @param component Pointer to the component
/// @param laf Pointer to the LookAndFeel to use
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t component_set_look_and_feel(JuceComponent* component,
                                    JuceLookAndFeel* laf,
                                    int8_t* error_buffer,
                                    size_t buffer_size);

// Parameter attachment operations

/// Opaque wrapper for juce::SliderParameterAttachment
/// Forward declaration - actual definition is in parameter_attachment_bridge.cpp
struct JuceSliderParameterAttachment;

/// Create a new SliderParameterAttachment.
/// 
/// This establishes bidirectional synchronization between a slider
/// and an audio parameter. When the slider value changes, the parameter
/// is updated. When the parameter changes (e.g., from automation),
/// the slider is updated.
/// 
/// @param slider Pointer to the slider component
/// @param parameter_id The parameter ID (UTF-8 bytes)
/// @param parameter_id_len Length of the parameter ID string
/// @param error_buffer Buffer to store error message if creation fails
/// @param buffer_size Size of the error buffer
/// @return Pointer to the created attachment, or nullptr on error
JuceSliderParameterAttachment* create_slider_parameter_attachment(
    JuceComponent* slider,
    const uint8_t* parameter_id,
    size_t parameter_id_len,
    int8_t* error_buffer,
    size_t buffer_size);

/// Delete a SliderParameterAttachment and free its resources.
/// 
/// This breaks the connection between the slider and parameter,
/// stopping bidirectional synchronization.
/// 
/// @param ptr Pointer to the attachment to delete
void delete_slider_parameter_attachment(JuceSliderParameterAttachment* ptr);

// ==============================================================================
// MessageManager Operations
// ==============================================================================

/// Check if the current thread is the message thread.
/// 
/// JUCE requires all GUI operations to be performed on the message thread.
/// This function queries JUCE to determine if the calling thread is the
/// message thread.
/// 
/// @return true if the current thread is the message thread, false otherwise
bool message_manager_is_message_thread();

/// Post a callback to be executed on the message thread.
/// 
/// This function queues a closure for execution on the message thread.
/// It's the safe way to update the UI from another thread (e.g., the
/// audio processing thread).
/// 
/// The callback will be executed asynchronously - this function returns
/// immediately without waiting for the callback to execute.
/// 
/// @param rust_closure Pointer to the Rust closure (as size_t)
/// @param invoke Function pointer to invoke the closure (as size_t)
/// @param error_buffer Buffer to store error message if operation fails
/// @param buffer_size Size of the error buffer
/// @return 0 on success, -1 on error
int32_t message_manager_call_async(size_t rust_closure,
                                   size_t invoke,
                                   int8_t* error_buffer,
                                   size_t buffer_size);

} // namespace nih_plug_juce

