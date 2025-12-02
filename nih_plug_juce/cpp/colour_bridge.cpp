// colour_bridge.cpp
// C++ implementation of Colour FFI bridge functions
//
// This file contains the C++ implementations of FFI functions for JUCE Colour
// operations. Colours are value types in JUCE and can be safely copied and
// manipulated.

#include "juce_bridge.h"
#include <cstring>

namespace nih_plug_juce {

// Create a new JUCE Colour from RGBA values
JuceColour* create_colour_rgba(uint8_t r, uint8_t g, uint8_t b, uint8_t a,
                               int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions_ptr([&]() -> JuceColour* {
        juce::Colour colour = juce::Colour(r, g, b, a);
        return new JuceColour(colour);
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Create a new JUCE Colour from a hexadecimal string
JuceColour* create_colour_from_hex(const uint8_t* hex, size_t hex_len,
                                   int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions_ptr([&]() -> JuceColour* {
        if (!hex || hex_len == 0) {
            throw std::invalid_argument("Hex string cannot be empty");
        }
        
        // Convert UTF-8 bytes to juce::String
        juce::String hexString(reinterpret_cast<const char*>(hex), hex_len);
        
        // Remove '#' prefix if present
        if (hexString.startsWith("#")) {
            hexString = hexString.substring(1);
        }
        
        // Parse the hex string
        // JUCE expects hex strings in the format RRGGBB or RRGGBBAA
        if (hexString.length() == 3) {
            // Short form RGB: expand to RRGGBB
            juce::String expanded;
            for (int i = 0; i < 3; ++i) {
                auto c = hexString[i];
                expanded += juce::String::charToString(c);
                expanded += juce::String::charToString(c);
            }
            hexString = expanded;
        }
        
        if (hexString.length() != 6 && hexString.length() != 8) {
            throw std::invalid_argument("Invalid hex color format. Expected RRGGBB or RRGGBBAA");
        }
        
        // Validate hex characters
        for (int i = 0; i < hexString.length(); ++i) {
            auto c = hexString[i];
            if (!((c >= '0' && c <= '9') || (c >= 'A' && c <= 'F') || (c >= 'a' && c <= 'f'))) {
                throw std::invalid_argument("Invalid hex character in color string");
            }
        }
        
        // Parse hex string manually to avoid endianness issues
        juce::Colour colour;
        if (hexString.length() == 6) {
            // RGB format - parse as RRGGBB
            uint8_t r = static_cast<uint8_t>(hexString.substring(0, 2).getHexValue32());
            uint8_t g = static_cast<uint8_t>(hexString.substring(2, 4).getHexValue32());
            uint8_t b = static_cast<uint8_t>(hexString.substring(4, 6).getHexValue32());
            colour = juce::Colour(r, g, b);
        } else {
            // RGBA format - parse as RRGGBBAA
            uint8_t r = static_cast<uint8_t>(hexString.substring(0, 2).getHexValue32());
            uint8_t g = static_cast<uint8_t>(hexString.substring(2, 4).getHexValue32());
            uint8_t b = static_cast<uint8_t>(hexString.substring(4, 6).getHexValue32());
            uint8_t a = static_cast<uint8_t>(hexString.substring(6, 8).getHexValue32());
            colour = juce::Colour(r, g, b, a);
        }
        
        return new JuceColour(colour);
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Delete a JUCE Colour and free its resources
void delete_colour(JuceColour* ptr) {
    if (ptr) {
        delete ptr;
    }
}

// Convert a colour to a hexadecimal string
size_t colour_to_hex(const JuceColour* ptr, uint8_t* buffer, size_t buffer_size) {
    if (!ptr || !buffer || buffer_size == 0) {
        return 0;
    }
    
    // Get RGBA components
    uint8_t r = ptr->colour.getRed();
    uint8_t g = ptr->colour.getGreen();
    uint8_t b = ptr->colour.getBlue();
    uint8_t a = ptr->colour.getAlpha();
    
    // Format as RRGGBBAA hex string
    char hexBuffer[9];
    std::snprintf(hexBuffer, sizeof(hexBuffer), "%02X%02X%02X%02X", r, g, b, a);
    
    // Copy to buffer
    size_t len = std::min(static_cast<size_t>(8), buffer_size - 1);
    std::memcpy(buffer, hexBuffer, len);
    buffer[len] = '\0';
    
    return len;
}

// Create a new colour with a different alpha value
JuceColour* colour_with_alpha(const JuceColour* ptr, float alpha,
                              int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions_ptr([&]() -> JuceColour* {
        if (!ptr) {
            throw std::invalid_argument("Colour pointer cannot be null");
        }
        
        // Clamp alpha to valid range [0.0, 1.0]
        float clampedAlpha = std::max(0.0f, std::min(1.0f, alpha));
        
        juce::Colour newColour = ptr->colour.withAlpha(clampedAlpha);
        return new JuceColour(newColour);
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Create a brighter version of a colour
JuceColour* colour_brighter(const JuceColour* ptr, float amount,
                            int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions_ptr([&]() -> JuceColour* {
        if (!ptr) {
            throw std::invalid_argument("Colour pointer cannot be null");
        }
        
        // Clamp amount to valid range [0.0, 1.0]
        float clampedAmount = std::max(0.0f, std::min(1.0f, amount));
        
        juce::Colour newColour = ptr->colour.brighter(clampedAmount);
        return new JuceColour(newColour);
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Create a darker version of a colour
JuceColour* colour_darker(const JuceColour* ptr, float amount,
                          int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions_ptr([&]() -> JuceColour* {
        if (!ptr) {
            throw std::invalid_argument("Colour pointer cannot be null");
        }
        
        // Clamp amount to valid range [0.0, 1.0]
        float clampedAmount = std::max(0.0f, std::min(1.0f, amount));
        
        juce::Colour newColour = ptr->colour.darker(clampedAmount);
        return new JuceColour(newColour);
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Create a colour interpolated between two colours
JuceColour* colour_interpolated_with(const JuceColour* ptr1, const JuceColour* ptr2,
                                     float proportion, int8_t* error_buffer,
                                     size_t buffer_size) {
    return catch_exceptions_ptr([&]() -> JuceColour* {
        if (!ptr1 || !ptr2) {
            throw std::invalid_argument("Colour pointers cannot be null");
        }
        
        // Clamp proportion to valid range [0.0, 1.0]
        float clampedProportion = std::max(0.0f, std::min(1.0f, proportion));
        
        juce::Colour newColour = ptr1->colour.interpolatedWith(ptr2->colour, clampedProportion);
        return new JuceColour(newColour);
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

} // namespace nih_plug_juce
