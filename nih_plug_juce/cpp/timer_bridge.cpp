// timer_bridge.cpp
// C++ implementation of Timer FFI bridge functions
//
// This file contains the C++ implementations of FFI functions for JUCE Timer.
// All functions use exception handling to ensure that C++ exceptions are caught
// at the FFI boundary and converted to error codes or messages.

#include "juce_bridge.h"

namespace nih_plug_juce {

// ============================================================================
// Timer Operations
// ============================================================================

// Custom timer class that supports callbacks
class CallbackTimer : public juce::Timer {
public:
    CallbackTimer(size_t closure, void (*invoke)(size_t), void (*drop)(size_t))
        : callback_closure(closure)
        , callback_invoke(invoke)
        , callback_drop(drop)
    {
    }
    
    ~CallbackTimer() override {
        // Stop the timer first to ensure no more callbacks
        stopTimer();
        
        // Clean up the Rust closure when the timer is destroyed
        if (callback_drop && callback_closure) {
            callback_drop(callback_closure);
        }
    }
    
    void timerCallback() override {
        // Invoke the Rust closure
        if (callback_invoke && callback_closure) {
            callback_invoke(callback_closure);
        }
    }
    
private:
    size_t callback_closure;
    void (*callback_invoke)(size_t);
    void (*callback_drop)(size_t);
    
    JUCE_DECLARE_NON_COPYABLE_WITH_LEAK_DETECTOR(CallbackTimer)
};

// Create a new JUCE Timer
JuceTimer* create_timer(size_t rust_closure,
                       size_t invoke,
                       size_t drop_fn,
                       int8_t* error_buffer,
                       size_t buffer_size) {
    return catch_exceptions_ptr([&]() -> JuceTimer* {
        if (!invoke) {
            throw std::invalid_argument("Timer callback invoke function is null");
        }
        if (!drop_fn) {
            throw std::invalid_argument("Timer callback drop function is null");
        }
        
        // Create a new CallbackTimer
        auto* timer = new CallbackTimer(
            rust_closure,
            reinterpret_cast<void (*)(size_t)>(invoke),
            reinterpret_cast<void (*)(size_t)>(drop_fn)
        );
        
        if (!timer) {
            throw std::runtime_error("Failed to allocate Timer");
        }
        
        // Wrap in our opaque type
        return new JuceTimer(timer);
        
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Delete a JUCE Timer
void delete_timer(JuceTimer* ptr) {
    if (ptr) {
        if (ptr->ptr) {
            delete ptr->ptr;
        }
        delete ptr;
    }
}

// Start a timer with the specified interval
int32_t timer_start(JuceTimer* ptr,
                   int32_t interval_ms,
                   int8_t* error_buffer,
                   size_t buffer_size) {
    return catch_exceptions([&]() {
        if (!ptr) {
            throw std::invalid_argument("Timer pointer is null");
        }
        if (!ptr->ptr) {
            throw std::invalid_argument("Timer is invalid");
        }
        if (interval_ms <= 0) {
            throw std::invalid_argument("Timer interval must be positive");
        }
        
        // Start the timer
        ptr->ptr->startTimer(interval_ms);
        
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Stop a timer
void timer_stop(JuceTimer* ptr) {
    if (ptr && ptr->ptr) {
        ptr->ptr->stopTimer();
    }
}

// Check if a timer is currently running
bool timer_is_running(const JuceTimer* ptr) {
    if (ptr && ptr->ptr) {
        return ptr->ptr->isTimerRunning();
    }
    return false;
}

} // namespace nih_plug_juce
