// tabbed_component_bridge.cpp
// C++ implementation of TabbedComponent FFI bridge functions
//
// This file contains the C++ implementations of FFI functions for JUCE
// TabbedComponent. All functions use exception handling to ensure
// that C++ exceptions are caught at the FFI boundary and converted to error codes
// or messages.

#include "juce_bridge.h"
#include <cstring>

namespace nih_plug_juce {

// ============================================================================
// TabbedComponent Operations
// ============================================================================

// Custom TabbedComponent class that supports tab changed callbacks
class CallbackTabbedComponent : public juce::TabbedComponent {
public:
    CallbackTabbedComponent(juce::TabbedButtonBar::Orientation orientation)
        : juce::TabbedComponent(orientation) {
        callback_closure = 0;
        callback_invoke = nullptr;
        callback_drop = nullptr;
    }
    
    ~CallbackTabbedComponent() override {
        // Clean up the Rust closure when the tabbed component is destroyed
        if (callback_drop && callback_closure) {
            callback_drop(callback_closure);
        }
    }
    
    void currentTabChanged(int newCurrentTabIndex, const juce::String& /*newCurrentTabName*/) override {
        // Invoke the Rust callback if set
        if (callback_invoke && callback_closure) {
            callback_invoke(callback_closure, static_cast<int32_t>(newCurrentTabIndex));
        }
    }
    
    void setTabChangedCallback(size_t closure, void (*invoke)(size_t, int32_t), void (*drop)(size_t)) {
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
    void (*callback_invoke)(size_t, int32_t);
    void (*callback_drop)(size_t);
};

// Create a new JUCE TabbedComponent
JuceComponent* create_tabbed_component(int32_t orientation,
                                       int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions_ptr([&]() -> JuceComponent* {
        // Convert orientation value to JUCE enum
        juce::TabbedButtonBar::Orientation juce_orientation;
        switch (orientation) {
            case 0:
                juce_orientation = juce::TabbedButtonBar::TabsAtTop;
                break;
            case 1:
                juce_orientation = juce::TabbedButtonBar::TabsAtBottom;
                break;
            case 2:
                juce_orientation = juce::TabbedButtonBar::TabsAtLeft;
                break;
            case 3:
                juce_orientation = juce::TabbedButtonBar::TabsAtRight;
                break;
            default:
                throw std::runtime_error("Invalid tab orientation value");
        }
        
        // Create a new CallbackTabbedComponent
        auto* tabbed = new CallbackTabbedComponent(juce_orientation);
        
        if (!tabbed) {
            throw std::runtime_error("Failed to allocate TabbedComponent");
        }
        
        // Wrap in our opaque type
        return new JuceComponent(tabbed);
        
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Add a tab to a tabbed component
int32_t tabbed_component_add_tab(JuceComponent* ptr, const uint8_t* name, size_t name_len,
                                 const JuceColour* colour, JuceComponent* content,
                                 int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions([&]() {
        if (!ptr || !ptr->ptr) {
            throw std::runtime_error("TabbedComponent pointer is null");
        }
        
        if (!colour) {
            throw std::runtime_error("Colour pointer is null");
        }
        
        if (!content || !content->ptr) {
            throw std::runtime_error("Content component pointer is null");
        }
        
        // Try to cast to TabbedComponent
        auto* tabbed = dynamic_cast<juce::TabbedComponent*>(ptr->ptr);
        if (!tabbed) {
            throw std::runtime_error("Component is not a TabbedComponent");
        }
        
        // Convert UTF-8 bytes to juce::String
        juce::String tab_name = juce::String::fromUTF8(reinterpret_cast<const char*>(name), static_cast<int>(name_len));
        
        // Add the tab, transferring ownership of the content component to JUCE
        // The third parameter (true) means JUCE will delete the component when the tab is removed
        tabbed->addTab(tab_name, colour->colour, content->ptr, true);
        
        // Important: We need to prevent the JuceComponent wrapper from deleting
        // the component pointer since JUCE now owns it
        content->ptr = nullptr;
        
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Remove a tab from a tabbed component
void tabbed_component_remove_tab(JuceComponent* ptr, int32_t index) {
    if (ptr && ptr->ptr) {
        // Try to cast to TabbedComponent
        auto* tabbed = dynamic_cast<juce::TabbedComponent*>(ptr->ptr);
        if (tabbed) {
            tabbed->removeTab(index);
        }
    }
}

// Set the current tab index
void tabbed_component_set_current_tab_index(JuceComponent* ptr, int32_t index) {
    if (ptr && ptr->ptr) {
        // Try to cast to TabbedComponent
        auto* tabbed = dynamic_cast<juce::TabbedComponent*>(ptr->ptr);
        if (tabbed) {
            tabbed->setCurrentTabIndex(index);
        }
    }
}

// Set a tab changed callback for a tabbed component
int32_t tabbed_component_set_on_tab_changed(JuceComponent* ptr,
                                            size_t rust_closure,
                                            size_t invoke_fn,
                                            size_t drop_fn,
                                            int8_t* error_buffer,
                                            size_t buffer_size) {
    return catch_exceptions([&]() {
        if (!ptr || !ptr->ptr) {
            throw std::runtime_error("TabbedComponent pointer is null");
        }
        
        // Try to cast to CallbackTabbedComponent
        auto* tabbed = dynamic_cast<CallbackTabbedComponent*>(ptr->ptr);
        if (!tabbed) {
            throw std::runtime_error("Component is not a CallbackTabbedComponent");
        }
        
        // Set the callback
        auto invoke = reinterpret_cast<void (*)(size_t, int32_t)>(invoke_fn);
        auto drop = reinterpret_cast<void (*)(size_t)>(drop_fn);
        
        tabbed->setTabChangedCallback(rust_closure, invoke, drop);
        
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

} // namespace nih_plug_juce
