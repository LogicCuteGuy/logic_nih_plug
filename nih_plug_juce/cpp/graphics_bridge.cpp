// graphics_bridge.cpp
// C++ implementation of Graphics FFI bridge functions
//
// This file contains the C++ implementations of FFI functions for JUCE Graphics
// operations. Graphics contexts are typically provided during paint callbacks
// and are managed by JUCE, so these functions don't need to create or destroy
// Graphics objects.

#include "juce_bridge.h"
#include <cstring>

namespace nih_plug_juce {

// Fill a rectangle with the current color
void graphics_fill_rect(JuceGraphics* g, int32_t x, int32_t y, int32_t width, int32_t height) {
    try {
        if (g && g->ptr) {
            g->ptr->fillRect(x, y, width, height);
        }
    } catch (...) {
        // Silently catch exceptions in void functions to prevent crashes
    }
}

// Draw a rectangle outline with the current color
void graphics_draw_rect(JuceGraphics* g, int32_t x, int32_t y, int32_t width, int32_t height) {
    try {
        if (g && g->ptr) {
            g->ptr->drawRect(x, y, width, height);
        }
    } catch (...) {
        // Silently catch exceptions in void functions to prevent crashes
    }
}

// Fill an ellipse with the current color
void graphics_fill_ellipse(JuceGraphics* g, float x, float y, float width, float height) {
    try {
        if (g && g->ptr) {
            g->ptr->fillEllipse(x, y, width, height);
        }
    } catch (...) {
        // Silently catch exceptions in void functions to prevent crashes
    }
}

// Draw a line with the current color
void graphics_draw_line(JuceGraphics* g, float x1, float y1, float x2, float y2) {
    try {
        if (g && g->ptr) {
            g->ptr->drawLine(x1, y1, x2, y2);
        }
    } catch (...) {
        // Silently catch exceptions in void functions to prevent crashes
    }
}

// Set the current drawing color
void graphics_set_colour(JuceGraphics* g, const JuceColour* colour) {
    try {
        if (g && g->ptr && colour) {
            g->ptr->setColour(colour->colour);
        }
    } catch (...) {
        // Silently catch exceptions in void functions to prevent crashes
    }
}

// Draw text within a rectangle
void graphics_draw_text(JuceGraphics* g, const uint8_t* text, size_t text_len,
                       int32_t x, int32_t y, int32_t width, int32_t height,
                       int32_t justification) {
    try {
        if (g && g->ptr && text) {
            // Convert UTF-8 bytes to juce::String
            juce::String juceText(reinterpret_cast<const char*>(text), text_len);
            
            // Create rectangle for text bounds
            juce::Rectangle<int> bounds(x, y, width, height);
            
            // Draw the text with the specified justification
            g->ptr->drawText(juceText, bounds, juce::Justification(justification));
        }
    } catch (...) {
        // Silently catch exceptions in void functions to prevent crashes
    }
}

// Draw an image at the specified position
void graphics_draw_image_at(JuceGraphics* g, const JuceImage* image, int32_t x, int32_t y) {
    try {
        if (g && g->ptr && image) {
            g->ptr->drawImageAt(image->image, x, y);
        }
    } catch (...) {
        // Silently catch exceptions in void functions to prevent crashes
    }
}

// Stroke (outline) a path with the current color
void graphics_stroke_path(JuceGraphics* g, const JucePath* path) {
    try {
        if (g && g->ptr && path) {
            // Use default stroke type (1 pixel wide)
            juce::PathStrokeType strokeType(1.0f);
            g->ptr->strokePath(path->path, strokeType);
        }
    } catch (...) {
        // Silently catch exceptions in void functions to prevent crashes
    }
}

// Fill a path with the current color
void graphics_fill_path(JuceGraphics* g, const JucePath* path) {
    try {
        if (g && g->ptr && path) {
            g->ptr->fillPath(path->path);
        }
    } catch (...) {
        // Silently catch exceptions in void functions to prevent crashes
    }
}

} // namespace nih_plug_juce
