// font_bridge.cpp
// C++ implementation of Font FFI bridge functions
//
// This file contains the C++ implementations of FFI functions for JUCE Font
// operations. Fonts are value types in JUCE and can be safely copied and
// manipulated.

#include "juce_bridge.h"
#include <cstring>

namespace nih_plug_juce {

// Create a new JUCE Font with the specified size
JuceFont* create_font(float size, int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions_ptr([&]() -> JuceFont* {
        if (size <= 0.0f) {
            throw std::invalid_argument("Font size must be positive");
        }
        
        juce::Font font(size);
        return new JuceFont(font);
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Create a new JUCE Font with a specific typeface and size
JuceFont* create_font_with_typeface(const uint8_t* typeface, size_t typeface_len, float size,
                                    int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions_ptr([&]() -> JuceFont* {
        if (!typeface || typeface_len == 0) {
            throw std::invalid_argument("Typeface name cannot be empty");
        }
        
        if (size <= 0.0f) {
            throw std::invalid_argument("Font size must be positive");
        }
        
        // Convert UTF-8 bytes to juce::String
        juce::String typefaceName(reinterpret_cast<const char*>(typeface), typeface_len);
        
        // Create font with the specified typeface
        juce::Font font(typefaceName, size, juce::Font::plain);
        return new JuceFont(font);
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Delete a JUCE Font and free its resources
void delete_font(JuceFont* ptr) {
    if (ptr) {
        delete ptr;
    }
}

// Set whether the font is bold
int32_t font_set_bold(JuceFont* ptr, bool bold, int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions([&]() {
        if (!ptr) {
            throw std::invalid_argument("Font pointer cannot be null");
        }
        
        ptr->font.setBold(bold);
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Set whether the font is italic
int32_t font_set_italic(JuceFont* ptr, bool italic, int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions([&]() {
        if (!ptr) {
            throw std::invalid_argument("Font pointer cannot be null");
        }
        
        ptr->font.setItalic(italic);
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Set whether the font is underlined
int32_t font_set_underline(JuceFont* ptr, bool underline, int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions([&]() {
        if (!ptr) {
            throw std::invalid_argument("Font pointer cannot be null");
        }
        
        ptr->font.setUnderline(underline);
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Get the width of a string when rendered with this font
int32_t font_get_string_width(const JuceFont* ptr, const uint8_t* text, size_t text_len,
                              int8_t* error_buffer, size_t buffer_size) {
    try {
        if (!ptr) {
            throw std::invalid_argument("Font pointer cannot be null");
        }
        
        if (!text || text_len == 0) {
            return 0; // Empty string has zero width
        }
        
        // Convert UTF-8 bytes to juce::String
        juce::String textString(reinterpret_cast<const char*>(text), text_len);
        
        // Get the string width
        int width = ptr->font.getStringWidth(textString);
        return width;
    } catch (const std::exception& e) {
        if (error_buffer && buffer_size > 0) {
            std::snprintf(reinterpret_cast<char*>(error_buffer), buffer_size, "%s", e.what());
        }
        return -1; // Error
    } catch (...) {
        if (error_buffer && buffer_size > 0) {
            std::snprintf(reinterpret_cast<char*>(error_buffer), buffer_size, "Unknown C++ exception");
        }
        return -1; // Error
    }
}

// Get the height of the font
int32_t font_get_height(const JuceFont* ptr, int8_t* error_buffer, size_t buffer_size) {
    try {
        if (!ptr) {
            throw std::invalid_argument("Font pointer cannot be null");
        }
        
        // Get the font height
        int height = static_cast<int>(ptr->font.getHeight());
        return height;
    } catch (const std::exception& e) {
        if (error_buffer && buffer_size > 0) {
            std::snprintf(reinterpret_cast<char*>(error_buffer), buffer_size, "%s", e.what());
        }
        return -1; // Error
    } catch (...) {
        if (error_buffer && buffer_size > 0) {
            std::snprintf(reinterpret_cast<char*>(error_buffer), buffer_size, "Unknown C++ exception");
        }
        return -1; // Error
    }
}

// Get the count of available typefaces on the system
int32_t font_get_typeface_count() {
    try {
        juce::StringArray typefaces = juce::Font::findAllTypefaceNames();
        return typefaces.size();
    } catch (...) {
        return -1; // Error
    }
}

// Get the name of a typeface by index
int32_t font_get_typeface_name(int32_t index, uint8_t* buffer, size_t buffer_size) {
    try {
        if (!buffer || buffer_size == 0) {
            return 0;
        }
        
        juce::StringArray typefaces = juce::Font::findAllTypefaceNames();
        
        if (index < 0 || index >= typefaces.size()) {
            return 0; // Index out of range
        }
        
        juce::String typefaceName = typefaces[index];
        
        // Convert to UTF-8 and copy to buffer
        const char* utf8 = typefaceName.toRawUTF8();
        size_t len = std::strlen(utf8);
        size_t copyLen = std::min(len, buffer_size - 1);
        
        std::memcpy(buffer, utf8, copyLen);
        buffer[copyLen] = '\0';
        
        return static_cast<int32_t>(copyLen);
    } catch (...) {
        return 0; // Error
    }
}

} // namespace nih_plug_juce
