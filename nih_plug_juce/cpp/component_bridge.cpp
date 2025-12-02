// component_bridge.cpp
// C++ implementation of Component FFI bridge functions
//
// This file contains the C++ implementations of FFI functions for JUCE Component
// operations. All functions use exception handling to ensure that C++ exceptions
// are caught at the FFI boundary and converted to error codes or messages.

#include "juce_bridge.h"
#include <cstring>

namespace nih_plug_juce {

// Mouse listener callback bridge structure
struct MouseListenerBridge {
    void* listener_ptr;
    void (*mouse_down)(size_t, int32_t, int32_t, bool, bool, bool, bool);
    void (*mouse_drag)(size_t, int32_t, int32_t, bool, bool, bool, bool);
    void (*mouse_up)(size_t, int32_t, int32_t, bool, bool, bool, bool);
    void (*mouse_enter)(size_t, int32_t, int32_t, bool, bool, bool, bool);
    void (*mouse_exit)(size_t, int32_t, int32_t, bool, bool, bool, bool);
    void (*drop)(size_t);
};

// Keyboard listener callback bridge structure
struct KeyListenerBridge {
    void* listener_ptr;
    bool (*key_pressed)(size_t, int32_t, bool, bool, bool, bool);
    bool (*key_state_changed)(size_t);
    void (*focus_gained)(size_t);
    void (*focus_lost)(size_t);
    void (*drop)(size_t);
};

// Custom component class that supports paint callbacks, mouse listeners, and keyboard listeners
class CallbackComponent : public juce::Component {
public:
    CallbackComponent() 
        : callback_bridge{nullptr, nullptr, nullptr},
          mouse_listener_bridge{nullptr, nullptr, nullptr, nullptr, nullptr, nullptr, nullptr},
          key_listener_bridge{nullptr, nullptr, nullptr, nullptr, nullptr, nullptr} {}
    
    ~CallbackComponent() override {
        // Clean up the Rust closure when the component is destroyed
        if (callback_bridge.drop && callback_bridge.rust_closure) {
            callback_bridge.drop(callback_bridge.rust_closure);
        }
        
        // Clean up the mouse listener
        if (mouse_listener_bridge.drop && mouse_listener_bridge.listener_ptr) {
            mouse_listener_bridge.drop(reinterpret_cast<size_t>(mouse_listener_bridge.listener_ptr));
        }
        
        // Clean up the keyboard listener
        if (key_listener_bridge.drop && key_listener_bridge.listener_ptr) {
            key_listener_bridge.drop(reinterpret_cast<size_t>(key_listener_bridge.listener_ptr));
        }
    }
    
    void setPaintCallback(PaintCallbackBridge bridge) {
        // Clean up old callback if it exists
        if (callback_bridge.drop && callback_bridge.rust_closure) {
            callback_bridge.drop(callback_bridge.rust_closure);
        }
        
        callback_bridge = bridge;
    }
    
    void paint(juce::Graphics& g) override {
        // Invoke the Rust callback if it's set
        if (callback_bridge.invoke && callback_bridge.rust_closure) {
            // Wrap the juce::Graphics in our opaque type
            JuceGraphics graphics_wrapper(&g);
            callback_bridge.invoke(callback_bridge.rust_closure, &graphics_wrapper);
        }
    }
    
    void setMouseListener(MouseListenerBridge bridge) {
        // Clean up old listener if it exists
        if (mouse_listener_bridge.drop && mouse_listener_bridge.listener_ptr) {
            mouse_listener_bridge.drop(reinterpret_cast<size_t>(mouse_listener_bridge.listener_ptr));
        }
        
        mouse_listener_bridge = bridge;
    }
    
    void mouseDown(const juce::MouseEvent& event) override {
        if (mouse_listener_bridge.mouse_down && mouse_listener_bridge.listener_ptr) {
            auto mods = event.mods;
            mouse_listener_bridge.mouse_down(
                reinterpret_cast<size_t>(mouse_listener_bridge.listener_ptr),
                event.x,
                event.y,
                mods.isShiftDown(),
                mods.isCtrlDown(),
                mods.isAltDown(),
                mods.isCommandDown()
            );
        }
    }
    
    void mouseDrag(const juce::MouseEvent& event) override {
        if (mouse_listener_bridge.mouse_drag && mouse_listener_bridge.listener_ptr) {
            auto mods = event.mods;
            mouse_listener_bridge.mouse_drag(
                reinterpret_cast<size_t>(mouse_listener_bridge.listener_ptr),
                event.x,
                event.y,
                mods.isShiftDown(),
                mods.isCtrlDown(),
                mods.isAltDown(),
                mods.isCommandDown()
            );
        }
    }
    
    void mouseUp(const juce::MouseEvent& event) override {
        if (mouse_listener_bridge.mouse_up && mouse_listener_bridge.listener_ptr) {
            auto mods = event.mods;
            mouse_listener_bridge.mouse_up(
                reinterpret_cast<size_t>(mouse_listener_bridge.listener_ptr),
                event.x,
                event.y,
                mods.isShiftDown(),
                mods.isCtrlDown(),
                mods.isAltDown(),
                mods.isCommandDown()
            );
        }
    }
    
    void mouseEnter(const juce::MouseEvent& event) override {
        if (mouse_listener_bridge.mouse_enter && mouse_listener_bridge.listener_ptr) {
            auto mods = event.mods;
            mouse_listener_bridge.mouse_enter(
                reinterpret_cast<size_t>(mouse_listener_bridge.listener_ptr),
                event.x,
                event.y,
                mods.isShiftDown(),
                mods.isCtrlDown(),
                mods.isAltDown(),
                mods.isCommandDown()
            );
        }
    }
    
    void mouseExit(const juce::MouseEvent& event) override {
        if (mouse_listener_bridge.mouse_exit && mouse_listener_bridge.listener_ptr) {
            auto mods = event.mods;
            mouse_listener_bridge.mouse_exit(
                reinterpret_cast<size_t>(mouse_listener_bridge.listener_ptr),
                event.x,
                event.y,
                mods.isShiftDown(),
                mods.isCtrlDown(),
                mods.isAltDown(),
                mods.isCommandDown()
            );
        }
    }
    
    void setKeyListener(KeyListenerBridge bridge) {
        // Clean up old listener if it exists
        if (key_listener_bridge.drop && key_listener_bridge.listener_ptr) {
            key_listener_bridge.drop(reinterpret_cast<size_t>(key_listener_bridge.listener_ptr));
        }
        
        key_listener_bridge = bridge;
    }
    
    bool keyPressed(const juce::KeyPress& key) override {
        if (key_listener_bridge.key_pressed && key_listener_bridge.listener_ptr) {
            auto mods = key.getModifiers();
            return key_listener_bridge.key_pressed(
                reinterpret_cast<size_t>(key_listener_bridge.listener_ptr),
                key.getKeyCode(),
                mods.isShiftDown(),
                mods.isCtrlDown(),
                mods.isAltDown(),
                mods.isCommandDown()
            );
        }
        return false;
    }
    
    bool keyStateChanged(bool isKeyDown) override {
        if (key_listener_bridge.key_state_changed && key_listener_bridge.listener_ptr) {
            return key_listener_bridge.key_state_changed(
                reinterpret_cast<size_t>(key_listener_bridge.listener_ptr)
            );
        }
        return false;
    }
    
    void focusGained(FocusChangeType cause) override {
        if (key_listener_bridge.focus_gained && key_listener_bridge.listener_ptr) {
            key_listener_bridge.focus_gained(
                reinterpret_cast<size_t>(key_listener_bridge.listener_ptr)
            );
        }
    }
    
    void focusLost(FocusChangeType cause) override {
        if (key_listener_bridge.focus_lost && key_listener_bridge.listener_ptr) {
            key_listener_bridge.focus_lost(
                reinterpret_cast<size_t>(key_listener_bridge.listener_ptr)
            );
        }
    }
    
private:
    PaintCallbackBridge callback_bridge;
    MouseListenerBridge mouse_listener_bridge;
    KeyListenerBridge key_listener_bridge;
};

// Create a new JUCE Component
JuceComponent* create_component(int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions_ptr([&]() -> JuceComponent* {
        // Create a new juce::Component
        auto* component = new juce::Component();
        
        if (!component) {
            throw std::runtime_error("Failed to allocate Component");
        }
        
        // Wrap in our opaque type
        return new JuceComponent(component);
        
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Create a new JUCE Component that supports paint callbacks
JuceComponent* create_component_with_paint_callback(int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions_ptr([&]() -> JuceComponent* {
        // Create a new CallbackComponent
        auto* component = new CallbackComponent();
        
        if (!component) {
            throw std::runtime_error("Failed to allocate CallbackComponent");
        }
        
        // Wrap in our opaque type
        return new JuceComponent(component);
        
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Delete a JUCE Component
void delete_component(JuceComponent* ptr) {
    if (ptr) {
        // Delete the actual juce::Component
        if (ptr->ptr) {
            delete ptr->ptr;
            ptr->ptr = nullptr;
        }
        // Delete the wrapper
        delete ptr;
    }
}

// Add a child component to a parent
int32_t component_add_child(JuceComponent* parent, JuceComponent* child,
                            int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions([&]() {
        if (!parent) {
            throw std::invalid_argument("Parent component pointer is null");
        }
        if (!parent->ptr) {
            throw std::invalid_argument("Parent component is invalid");
        }
        if (!child) {
            throw std::invalid_argument("Child component pointer is null");
        }
        if (!child->ptr) {
            throw std::invalid_argument("Child component is invalid");
        }
        
        // Add the child component and make it visible
        parent->ptr->addAndMakeVisible(child->ptr);
        
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Remove a child component from a parent
int32_t component_remove_child(JuceComponent* parent, JuceComponent* child,
                               int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions([&]() {
        if (!parent) {
            throw std::invalid_argument("Parent component pointer is null");
        }
        if (!parent->ptr) {
            throw std::invalid_argument("Parent component is invalid");
        }
        if (!child) {
            throw std::invalid_argument("Child component pointer is null");
        }
        if (!child->ptr) {
            throw std::invalid_argument("Child component is invalid");
        }
        
        // Remove the child component
        parent->ptr->removeChildComponent(child->ptr);
        
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Set the bounds of a component
void component_set_bounds(JuceComponent* ptr, int32_t x, int32_t y, int32_t width, int32_t height) {
    try {
        if (ptr && ptr->ptr) {
            // Set the component bounds
            // Note: JUCE allows negative dimensions, which it treats as zero
            ptr->ptr->setBounds(x, y, width, height);
        }
    } catch (...) {
        // Silently catch exceptions in void functions to prevent crashes
    }
}

// Set whether a component is visible
void component_set_visible(JuceComponent* ptr, bool visible) {
    try {
        if (ptr && ptr->ptr) {
            ptr->ptr->setVisible(visible);
        }
    } catch (...) {
        // Silently catch exceptions in void functions to prevent crashes
    }
}

// Trigger a repaint of a component
void component_repaint(JuceComponent* ptr) {
    try {
        if (ptr && ptr->ptr) {
            ptr->ptr->repaint();
        }
    } catch (...) {
        // Silently catch exceptions in void functions to prevent crashes
    }
}

// Set a paint callback for a component
int32_t component_set_paint_callback(JuceComponent* ptr,
                                     size_t rust_closure,
                                     size_t invoke,
                                     size_t drop_fn,
                                     int8_t* error_buffer,
                                     size_t buffer_size) {
    return catch_exceptions([&]() {
        if (!ptr) {
            throw std::invalid_argument("Component pointer is null");
        }
        if (!ptr->ptr) {
            throw std::invalid_argument("Component is invalid");
        }
        
        // Try to cast to CallbackComponent
        auto* callback_comp = dynamic_cast<CallbackComponent*>(ptr->ptr);
        if (!callback_comp) {
            throw std::runtime_error("Component does not support paint callbacks. "
                                   "Use Component::new_with_paint_callback() to create a component with callback support.");
        }
        
        // Create the callback bridge
        PaintCallbackBridge callback;
        callback.rust_closure = reinterpret_cast<void*>(rust_closure);
        callback.invoke = reinterpret_cast<void (*)(void*, JuceGraphics*)>(invoke);
        callback.drop = reinterpret_cast<void (*)(void*)>(drop_fn);
        
        // Set the paint callback
        callback_comp->setPaintCallback(callback);
        
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Set a mouse listener for a component
int32_t component_set_mouse_listener(JuceComponent* ptr,
                                     size_t listener_ptr,
                                     size_t mouse_down,
                                     size_t mouse_drag,
                                     size_t mouse_up,
                                     size_t mouse_enter,
                                     size_t mouse_exit,
                                     size_t drop_fn,
                                     int8_t* error_buffer,
                                     size_t buffer_size) {
    return catch_exceptions([&]() {
        if (!ptr) {
            throw std::invalid_argument("Component pointer is null");
        }
        if (!ptr->ptr) {
            throw std::invalid_argument("Component is invalid");
        }
        
        // For now, we only support mouse listeners on CallbackComponent
        // In the future, we could support it on any component by wrapping them
        auto* callback_comp = dynamic_cast<CallbackComponent*>(ptr->ptr);
        if (!callback_comp) {
            throw std::runtime_error("Component does not support mouse listeners. "
                                   "Use Component::new_with_paint_callback() to create a component with listener support.");
        }
        
        // Create the mouse listener bridge
        MouseListenerBridge listener;
        listener.listener_ptr = reinterpret_cast<void*>(listener_ptr);
        listener.mouse_down = reinterpret_cast<void (*)(size_t, int32_t, int32_t, bool, bool, bool, bool)>(mouse_down);
        listener.mouse_drag = reinterpret_cast<void (*)(size_t, int32_t, int32_t, bool, bool, bool, bool)>(mouse_drag);
        listener.mouse_up = reinterpret_cast<void (*)(size_t, int32_t, int32_t, bool, bool, bool, bool)>(mouse_up);
        listener.mouse_enter = reinterpret_cast<void (*)(size_t, int32_t, int32_t, bool, bool, bool, bool)>(mouse_enter);
        listener.mouse_exit = reinterpret_cast<void (*)(size_t, int32_t, int32_t, bool, bool, bool, bool)>(mouse_exit);
        listener.drop = reinterpret_cast<void (*)(size_t)>(drop_fn);
        
        // Set the mouse listener
        callback_comp->setMouseListener(listener);
        
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Set whether a component wants keyboard focus
int32_t component_set_wants_keyboard_focus(JuceComponent* ptr,
                                           bool wants,
                                           int8_t* error_buffer,
                                           size_t buffer_size) {
    return catch_exceptions([&]() {
        if (!ptr) {
            throw std::invalid_argument("Component pointer is null");
        }
        if (!ptr->ptr) {
            throw std::invalid_argument("Component is invalid");
        }
        
        // Set whether the component wants keyboard focus
        ptr->ptr->setWantsKeyboardFocus(wants);
        
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Set a keyboard listener for a component
int32_t component_set_key_listener(JuceComponent* ptr,
                                   size_t listener_ptr,
                                   size_t key_pressed,
                                   size_t key_state_changed,
                                   size_t focus_gained,
                                   size_t focus_lost,
                                   size_t drop_fn,
                                   int8_t* error_buffer,
                                   size_t buffer_size) {
    return catch_exceptions([&]() {
        if (!ptr) {
            throw std::invalid_argument("Component pointer is null");
        }
        if (!ptr->ptr) {
            throw std::invalid_argument("Component is invalid");
        }
        
        // For now, we only support keyboard listeners on CallbackComponent
        // In the future, we could support it on any component by wrapping them
        auto* callback_comp = dynamic_cast<CallbackComponent*>(ptr->ptr);
        if (!callback_comp) {
            throw std::runtime_error("Component does not support keyboard listeners. "
                                   "Use Component::new_with_paint_callback() to create a component with listener support.");
        }
        
        // Create the keyboard listener bridge
        KeyListenerBridge listener;
        listener.listener_ptr = reinterpret_cast<void*>(listener_ptr);
        listener.key_pressed = reinterpret_cast<bool (*)(size_t, int32_t, bool, bool, bool, bool)>(key_pressed);
        listener.key_state_changed = reinterpret_cast<bool (*)(size_t)>(key_state_changed);
        listener.focus_gained = reinterpret_cast<void (*)(size_t)>(focus_gained);
        listener.focus_lost = reinterpret_cast<void (*)(size_t)>(focus_lost);
        listener.drop = reinterpret_cast<void (*)(size_t)>(drop_fn);
        
        // Set the keyboard listener
        callback_comp->setKeyListener(listener);
        
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

} // namespace nih_plug_juce
