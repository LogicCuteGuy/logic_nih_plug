// juce_bridge.cpp
// Implementation of the JUCE FFI bridge
//
// This file contains the C++ implementation of FFI functions that bridge
// between Rust and JUCE. All functions use exception handling to ensure
// that C++ exceptions are caught at the FFI boundary and converted to
// error codes or messages that can be safely handled in Rust.

#include "juce_bridge.h"
#include <sstream>

namespace nih_plug_juce {

bool initialize() {
    // Verify JUCE is properly initialized
    // This is a simple test to ensure JUCE modules are linked correctly
    
    try {
        // Test juce_core - String operations
        juce::String testString("JUCE FFI Bridge");
        if (testString.isEmpty()) {
            return false;
        }
        
        // Verify string contains expected content
        if (!testString.contains("JUCE")) {
            return false;
        }
        
        // Test juce_graphics - Colour operations
        juce::Colour testColour(juce::uint8(0xFF), juce::uint8(0x00), juce::uint8(0x00), juce::uint8(0xFF));
        if (testColour.getRed() != 0xFF) {
            return false;
        }
        if (testColour.getGreen() != 0x00) {
            return false;
        }
        if (testColour.getBlue() != 0x00) {
            return false;
        }
        
        // Test juce_gui_basics - Component creation
        // We don't actually create a component here since we're not on the message thread,
        // but we can verify the class exists
        static_assert(std::is_base_of<juce::Component, juce::Component>::value,
                     "juce::Component should be available");
        
        // Test juce_events - MessageManager availability
        // Just verify the class exists, don't try to use it
        static_assert(sizeof(juce::MessageManager) > 0,
                     "juce::MessageManager should be available");
        
        // If we got here, basic JUCE functionality is working
        return true;
        
    } catch (const std::exception& e) {
        // If any exception occurred during initialization, JUCE is not working properly
        return false;
    } catch (...) {
        // Unknown exception
        return false;
    }
}

std::string get_version() {
    std::ostringstream oss;
    oss << VERSION_MAJOR << "." << VERSION_MINOR << "." << VERSION_PATCH;
    return oss.str();
}

// Example of exception-safe FFI function pattern
// This demonstrates how all future FFI functions should be structured
// to catch C++ exceptions and convert them to error codes.
//
// int example_ffi_function(SomeType* ptr, char* error_buffer, size_t buffer_size) {
//     return catch_exceptions([&]() {
//         if (!ptr) {
//             throw std::invalid_argument("Null pointer provided");
//         }
//         
//         // Perform the actual operation
//         ptr->someMethod();
//         
//     }, error_buffer, buffer_size);
// }

} // namespace nih_plug_juce
