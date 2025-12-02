// document_window_bridge.cpp
// C++ implementation of DocumentWindow FFI bridge functions
//
// This file contains the C++ implementations of FFI functions for JUCE
// DocumentWindow component. All functions use exception handling to ensure
// that C++ exceptions are caught at the FFI boundary and converted to error codes
// or messages.

#include "juce_bridge.h"
#include <cstring>

namespace nih_plug_juce {

// ============================================================================
// DocumentWindow Operations
// ============================================================================

// Custom DocumentWindow class that supports close callbacks
class CallbackDocumentWindow : public juce::DocumentWindow {
public:
    CallbackDocumentWindow(const juce::String& title)
        : juce::DocumentWindow(title, 
                               juce::Colours::lightgrey,
                               juce::DocumentWindow::allButtons) {
        callback_closure = 0;
        callback_invoke = nullptr;
        callback_drop = nullptr;
    }
    
    ~CallbackDocumentWindow() override {
        // Clean up the Rust closure when the window is destroyed
        if (callback_drop && callback_closure) {
            callback_drop(callback_closure);
        }
    }
    
    void closeButtonPressed() override {
        // Invoke the Rust callback if set
        if (callback_invoke && callback_closure) {
            bool should_close = callback_invoke(callback_closure);
            if (should_close) {
                // Allow the window to close
                juce::DocumentWindow::closeButtonPressed();
            }
        } else {
            // No callback set, use default behavior
            juce::DocumentWindow::closeButtonPressed();
        }
    }
    
    void setCloseCallback(size_t closure, bool (*invoke)(size_t), void (*drop)(size_t)) {
        // Clean up old callback if it exists
        if (callback_drop && callback_closure) {
            callback_drop(callback_closure);
        }
        
        callback_closure = closure;
        callback_invoke = invoke;
        callback_drop = drop;
    }
    
private:
    size_t callback_closure;
    bool (*callback_invoke)(size_t);
    void (*callback_drop)(size_t);
};

// Create a new JUCE DocumentWindow
JuceComponent* create_document_window(const uint8_t* title, size_t title_len,
                                      int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions_ptr([&]() -> JuceComponent* {
        // Convert UTF-8 bytes to juce::String
        juce::String window_title = juce::String::fromUTF8(reinterpret_cast<const char*>(title), static_cast<int>(title_len));
        
        // Create a new CallbackDocumentWindow
        auto* window = new CallbackDocumentWindow(window_title);
        
        if (!window) {
            throw std::runtime_error("Failed to allocate DocumentWindow");
        }
        
        // Wrap in our opaque type
        return new JuceComponent(window);
        
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Set the content component for a document window, transferring ownership
int32_t document_window_set_content_owned(JuceComponent* ptr, JuceComponent* content,
                                          int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions([&]() {
        if (!ptr || !ptr->ptr) {
            throw std::runtime_error("DocumentWindow pointer is null");
        }
        
        if (!content || !content->ptr) {
            throw std::runtime_error("Content component pointer is null");
        }
        
        // Try to cast to DocumentWindow
        auto* window = dynamic_cast<juce::DocumentWindow*>(ptr->ptr);
        if (!window) {
            throw std::runtime_error("Component is not a DocumentWindow");
        }
        
        // Set the content, transferring ownership to JUCE
        // JUCE will manage the lifetime of the content component
        window->setContentOwned(content->ptr, true);
        
        // Important: We need to prevent the JuceComponent wrapper from deleting
        // the component pointer since JUCE now owns it
        content->ptr = nullptr;
        
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Set the name (title) of a document window
void document_window_set_name(JuceComponent* ptr, const uint8_t* name, size_t name_len) {
    if (ptr && ptr->ptr) {
        // Try to cast to DocumentWindow
        auto* window = dynamic_cast<juce::DocumentWindow*>(ptr->ptr);
        if (window) {
            // Convert UTF-8 bytes to juce::String
            juce::String window_name = juce::String::fromUTF8(reinterpret_cast<const char*>(name), static_cast<int>(name_len));
            window->setName(window_name);
        }
    }
}

// Set a close callback for a document window
int32_t document_window_set_on_close(JuceComponent* ptr,
                                     size_t rust_closure,
                                     size_t invoke_fn,
                                     size_t drop_fn,
                                     int8_t* error_buffer,
                                     size_t buffer_size) {
    return catch_exceptions([&]() {
        if (!ptr || !ptr->ptr) {
            throw std::runtime_error("DocumentWindow pointer is null");
        }
        
        // Try to cast to CallbackDocumentWindow
        auto* window = dynamic_cast<CallbackDocumentWindow*>(ptr->ptr);
        if (!window) {
            throw std::runtime_error("Component is not a CallbackDocumentWindow");
        }
        
        // Set the callback
        auto invoke = reinterpret_cast<bool (*)(size_t)>(invoke_fn);
        auto drop = reinterpret_cast<void (*)(size_t)>(drop_fn);
        
        window->setCloseCallback(rust_closure, invoke, drop);
        
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// ============================================================================
// ResizableWindow Operations
// ============================================================================

// Custom ResizableWindow class that supports resize callbacks
class CallbackResizableWindow : public juce::ResizableWindow {
public:
    CallbackResizableWindow(const juce::String& title)
        : juce::ResizableWindow(title, 
                                juce::Colours::lightgrey,
                                true) {  // true = add to desktop
        callback_closure = 0;
        callback_invoke = nullptr;
        callback_drop = nullptr;
    }
    
    ~CallbackResizableWindow() override {
        // Clean up the Rust closure when the window is destroyed
        if (callback_drop && callback_closure) {
            callback_drop(callback_closure);
        }
    }
    
    void resized() override {
        // Call parent implementation first
        juce::ResizableWindow::resized();
        
        // Invoke the Rust callback if set
        if (callback_invoke && callback_closure) {
            callback_invoke(callback_closure, getWidth(), getHeight());
        }
    }
    
    void setResizeCallback(size_t closure, void (*invoke)(size_t, int32_t, int32_t), void (*drop)(size_t)) {
        // Clean up old callback if it exists
        if (callback_drop && callback_closure) {
            callback_drop(callback_closure);
        }
        
        callback_closure = closure;
        callback_invoke = invoke;
        callback_drop = drop;
    }
    
private:
    size_t callback_closure;
    void (*callback_invoke)(size_t, int32_t, int32_t);
    void (*callback_drop)(size_t);
};

// Create a new JUCE ResizableWindow
JuceComponent* create_resizable_window(const uint8_t* title, size_t title_len,
                                       int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions_ptr([&]() -> JuceComponent* {
        // Convert UTF-8 bytes to juce::String
        juce::String window_title = juce::String::fromUTF8(reinterpret_cast<const char*>(title), static_cast<int>(title_len));
        
        // Create a new CallbackResizableWindow
        auto* window = new CallbackResizableWindow(window_title);
        
        if (!window) {
            throw std::runtime_error("Failed to allocate ResizableWindow");
        }
        
        // Set default properties
        window->setUsingNativeTitleBar(true);
        
        // Wrap in our opaque type
        return new JuceComponent(window);
        
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Enable or disable user resizing of the window
void resizable_window_set_resizable(JuceComponent* ptr, bool resizable) {
    if (ptr && ptr->ptr) {
        // Try to cast to ResizableWindow
        auto* window = dynamic_cast<juce::ResizableWindow*>(ptr->ptr);
        if (window) {
            window->setResizable(resizable, true);  // true = use corner resizer
        }
    }
}

// Set the minimum and maximum size constraints for the window
void resizable_window_set_resize_limits(JuceComponent* ptr,
                                        int32_t min_width, int32_t min_height,
                                        int32_t max_width, int32_t max_height) {
    if (ptr && ptr->ptr) {
        // Try to cast to ResizableWindow
        auto* window = dynamic_cast<juce::ResizableWindow*>(ptr->ptr);
        if (window) {
            window->setResizeLimits(min_width, min_height, max_width, max_height);
        }
    }
}

// Set a resize callback for a resizable window
int32_t resizable_window_set_on_resized(JuceComponent* ptr,
                                        size_t rust_closure,
                                        size_t invoke_fn,
                                        size_t drop_fn,
                                        int8_t* error_buffer,
                                        size_t buffer_size) {
    return catch_exceptions([&]() {
        if (!ptr || !ptr->ptr) {
            throw std::runtime_error("ResizableWindow pointer is null");
        }
        
        // Try to cast to CallbackResizableWindow
        auto* window = dynamic_cast<CallbackResizableWindow*>(ptr->ptr);
        if (!window) {
            throw std::runtime_error("Component is not a CallbackResizableWindow");
        }
        
        // Set the callback
        auto invoke = reinterpret_cast<void (*)(size_t, int32_t, int32_t)>(invoke_fn);
        auto drop = reinterpret_cast<void (*)(size_t)>(drop_fn);
        
        window->setResizeCallback(rust_closure, invoke, drop);
        
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// ============================================================================
// Viewport Operations
// ============================================================================

// Custom Viewport class that supports visible area changed callbacks
class CallbackViewport : public juce::Viewport {
public:
    CallbackViewport()
        : juce::Viewport() {
        callback_closure = 0;
        callback_invoke = nullptr;
        callback_drop = nullptr;
    }
    
    ~CallbackViewport() override {
        // Clean up the Rust closure when the viewport is destroyed
        if (callback_drop && callback_closure) {
            callback_drop(callback_closure);
        }
    }
    
    void visibleAreaChanged(const juce::Rectangle<int>& /*newVisibleArea*/) override {
        // Invoke the Rust callback if set
        if (callback_invoke && callback_closure) {
            callback_invoke(callback_closure);
        }
    }
    
    void setVisibleAreaChangedCallback(size_t closure, void (*invoke)(size_t), void (*drop)(size_t)) {
        // Clean up old callback if it exists
        if (callback_drop && callback_closure) {
            callback_drop(callback_closure);
        }
        
        callback_closure = closure;
        callback_invoke = invoke;
        callback_drop = drop;
    }
    
private:
    size_t callback_closure;
    void (*callback_invoke)(size_t);
    void (*callback_drop)(size_t);
};

// Create a new JUCE Viewport
JuceComponent* create_viewport(int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions_ptr([&]() -> JuceComponent* {
        // Create a new CallbackViewport
        auto* viewport = new CallbackViewport();
        
        if (!viewport) {
            throw std::runtime_error("Failed to allocate Viewport");
        }
        
        // Wrap in our opaque type
        return new JuceComponent(viewport);
        
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Set the component to be viewed in the viewport, transferring ownership
int32_t viewport_set_viewed_component(JuceComponent* ptr, JuceComponent* component,
                                      int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions([&]() {
        if (!ptr || !ptr->ptr) {
            throw std::runtime_error("Viewport pointer is null");
        }
        
        if (!component || !component->ptr) {
            throw std::runtime_error("Viewed component pointer is null");
        }
        
        // Try to cast to Viewport
        auto* viewport = dynamic_cast<juce::Viewport*>(ptr->ptr);
        if (!viewport) {
            throw std::runtime_error("Component is not a Viewport");
        }
        
        // Set the viewed component, transferring ownership to JUCE
        // JUCE will manage the lifetime of the component
        viewport->setViewedComponent(component->ptr, true);
        
        // Important: We need to prevent the JuceComponent wrapper from deleting
        // the component pointer since JUCE now owns it
        component->ptr = nullptr;
        
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Set the scroll position of the viewport
void viewport_set_view_position(JuceComponent* ptr, int32_t x, int32_t y) {
    if (ptr && ptr->ptr) {
        // Try to cast to Viewport
        auto* viewport = dynamic_cast<juce::Viewport*>(ptr->ptr);
        if (viewport) {
            viewport->setViewPosition(x, y);
        }
    }
}

// Set whether scrollbars are shown
void viewport_set_scrollbars_shown(JuceComponent* ptr, bool vertical, bool horizontal) {
    if (ptr && ptr->ptr) {
        // Try to cast to Viewport
        auto* viewport = dynamic_cast<juce::Viewport*>(ptr->ptr);
        if (viewport) {
            viewport->setScrollBarsShown(vertical, horizontal);
        }
    }
}

// Set a visible area changed callback for a viewport
int32_t viewport_set_on_visible_area_changed(JuceComponent* ptr,
                                             size_t rust_closure,
                                             size_t invoke_fn,
                                             size_t drop_fn,
                                             int8_t* error_buffer,
                                             size_t buffer_size) {
    return catch_exceptions([&]() {
        if (!ptr || !ptr->ptr) {
            throw std::runtime_error("Viewport pointer is null");
        }
        
        // Try to cast to CallbackViewport
        auto* viewport = dynamic_cast<CallbackViewport*>(ptr->ptr);
        if (!viewport) {
            throw std::runtime_error("Component is not a CallbackViewport");
        }
        
        // Set the callback
        auto invoke = reinterpret_cast<void (*)(size_t)>(invoke_fn);
        auto drop = reinterpret_cast<void (*)(size_t)>(drop_fn);
        
        viewport->setVisibleAreaChangedCallback(rust_closure, invoke, drop);
        
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

} // namespace nih_plug_juce
