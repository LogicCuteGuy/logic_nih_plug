// message_thread_bridge.cpp
// C++ bridge implementation for JUCE MessageManager operations
//
// This file implements FFI functions for checking if code is running on
// the message thread and for posting callbacks to the message thread.

#include "juce_bridge.h"

namespace nih_plug_juce {

// ==============================================================================
// MessageManager Operations
// ==============================================================================

bool message_manager_is_message_thread() {
    // Query JUCE to check if the current thread is the message thread
    auto* mm = juce::MessageManager::getInstanceWithoutCreating();
    if (mm == nullptr) {
        // MessageManager hasn't been created yet, so we're not on the message thread
        return false;
    }
    
    return mm->isThisTheMessageThread();
}

int32_t message_manager_call_async(size_t rust_closure,
                                   size_t invoke,
                                   int8_t* error_buffer,
                                   size_t buffer_size) {
    return catch_exceptions([&]() {
        // Verify MessageManager exists
        auto* mm = juce::MessageManager::getInstance();
        if (mm == nullptr) {
            throw std::runtime_error("MessageManager not initialized");
        }
        
        // Create a callback bridge structure
        struct CallbackBridge {
            void* rust_closure;
            void (*invoke)(void*);
        };
        
        CallbackBridge bridge;
        bridge.rust_closure = reinterpret_cast<void*>(rust_closure);
        bridge.invoke = reinterpret_cast<void(*)(void*)>(invoke);
        
        // Post the callback to the message thread
        // We use callAsync which queues the callback for execution on the message thread
        // The lambda captures the bridge by value, so it's safe even if this function returns
        juce::MessageManager::callAsync([bridge]() {
            // This lambda runs on the message thread
            // Call the Rust trampoline function which will invoke the actual Rust closure
            if (bridge.invoke != nullptr && bridge.rust_closure != nullptr) {
                bridge.invoke(bridge.rust_closure);
            }
        });
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

} // namespace nih_plug_juce
