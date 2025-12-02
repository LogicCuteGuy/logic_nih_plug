// alert_window_bridge.cpp
// C++ bridge functions for JUCE AlertWindow
//
// This file implements FFI bridge functions for JUCE's AlertWindow class,
// which provides message boxes and confirmation dialogs.

#include "juce_bridge.h"
#include <memory>
#include <functional>

namespace nih_plug_juce {

// Helper class to manage async alert window callbacks
class AlertWindowCallback {
public:
    using VoidCallback = std::function<void()>;
    using BoolCallback = std::function<void(bool)>;
    
    // For message box async (void callback)
    AlertWindowCallback(size_t rust_closure, size_t invoke, size_t drop_fn, bool is_bool_callback)
        : rust_closure_(rust_closure)
        , invoke_void_(reinterpret_cast<void(*)(size_t)>(invoke))
        , invoke_bool_(reinterpret_cast<void(*)(size_t, bool)>(invoke))
        , drop_fn_(reinterpret_cast<void(*)(size_t)>(drop_fn))
        , is_bool_callback_(is_bool_callback)
    {
    }
    
    ~AlertWindowCallback() {
        if (drop_fn_ && rust_closure_) {
            drop_fn_(rust_closure_);
        }
    }
    
    void invoke() {
        if (invoke_void_ && rust_closure_) {
            invoke_void_(rust_closure_);
        }
    }
    
    void invoke(bool result) {
        if (invoke_bool_ && rust_closure_) {
            invoke_bool_(rust_closure_, result);
        }
    }
    
private:
    size_t rust_closure_;
    void (*invoke_void_)(size_t);
    void (*invoke_bool_)(size_t, bool);
    void (*drop_fn_)(size_t);
    bool is_bool_callback_;
};

// Show a synchronous message box
void alert_window_show_message_box(
    const uint8_t* title,
    size_t title_len,
    const uint8_t* message,
    size_t message_len)
{
    // Convert UTF-8 bytes to juce::String
    juce::String title_str = juce::String::fromUTF8(reinterpret_cast<const char*>(title), static_cast<int>(title_len));
    juce::String message_str = juce::String::fromUTF8(reinterpret_cast<const char*>(message), static_cast<int>(message_len));
    
    // Show the message box (blocks until dismissed)
    juce::AlertWindow::showMessageBoxAsync(
        juce::MessageBoxIconType::InfoIcon,
        title_str,
        message_str,
        juce::String(),  // No button text (uses default "OK")
        nullptr,         // No associated component
        nullptr          // No callback (synchronous behavior)
    );
}

// Show an asynchronous message box with a callback
int32_t alert_window_show_message_box_async(
    const uint8_t* title,
    size_t title_len,
    const uint8_t* message,
    size_t message_len,
    size_t rust_closure,
    size_t invoke,
    size_t drop_fn,
    int8_t* error_buffer,
    size_t buffer_size)
{
    return catch_exceptions([&]() {
        // Convert UTF-8 bytes to juce::String
        juce::String title_str = juce::String::fromUTF8(reinterpret_cast<const char*>(title), static_cast<int>(title_len));
        juce::String message_str = juce::String::fromUTF8(reinterpret_cast<const char*>(message), static_cast<int>(message_len));
        
        // Create a callback wrapper that will be owned by the lambda
        auto callback = std::make_shared<AlertWindowCallback>(rust_closure, invoke, drop_fn, false);
        
        // Show the message box asynchronously
        juce::AlertWindow::showMessageBoxAsync(
            juce::MessageBoxIconType::InfoIcon,
            title_str,
            message_str,
            juce::String(),  // No button text (uses default "OK")
            nullptr,         // No associated component
            juce::ModalCallbackFunction::create([callback](int) {
                // Invoke the Rust callback when the dialog is dismissed
                callback->invoke();
            })
        );
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Show an OK/Cancel confirmation dialog with a callback
int32_t alert_window_show_ok_cancel_box(
    const uint8_t* title,
    size_t title_len,
    const uint8_t* message,
    size_t message_len,
    size_t rust_closure,
    size_t invoke,
    size_t drop_fn,
    int8_t* error_buffer,
    size_t buffer_size)
{
    return catch_exceptions([&]() {
        // Convert UTF-8 bytes to juce::String
        juce::String title_str = juce::String::fromUTF8(reinterpret_cast<const char*>(title), static_cast<int>(title_len));
        juce::String message_str = juce::String::fromUTF8(reinterpret_cast<const char*>(message), static_cast<int>(message_len));
        
        // Create a callback wrapper that will be owned by the lambda
        auto callback = std::make_shared<AlertWindowCallback>(rust_closure, invoke, drop_fn, true);
        
        // Show the OK/Cancel box asynchronously
        juce::AlertWindow::showOkCancelBox(
            juce::MessageBoxIconType::QuestionIcon,
            title_str,
            message_str,
            juce::String(),  // No OK button text (uses default "OK")
            juce::String(),  // No Cancel button text (uses default "Cancel")
            nullptr,         // No associated component
            juce::ModalCallbackFunction::create([callback](int result) {
                // result is 1 for OK, 0 for Cancel
                callback->invoke(result == 1);
            })
        );
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

} // namespace nih_plug_juce

