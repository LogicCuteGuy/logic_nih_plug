// flexbox_bridge.cpp
// C++ implementation of FlexBox FFI bridge functions

#include "juce_bridge.h"
#include <memory>

namespace nih_plug_juce {

// ============================================================================
// FlexBox operations
// ============================================================================

JuceFlexBox* create_flexbox(int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions_ptr([&]() -> JuceFlexBox* {
        return new JuceFlexBox();
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

void delete_flexbox(JuceFlexBox* ptr) {
    if (ptr) {
        delete ptr;
    }
}

void flexbox_set_direction(JuceFlexBox* ptr, int32_t direction) {
    try {
        if (!ptr) return;
        
        switch (direction) {
            case 0:
                ptr->flexbox.flexDirection = juce::FlexBox::Direction::row;
                break;
            case 1:
                ptr->flexbox.flexDirection = juce::FlexBox::Direction::column;
                break;
            case 2:
                ptr->flexbox.flexDirection = juce::FlexBox::Direction::rowReverse;
                break;
            case 3:
                ptr->flexbox.flexDirection = juce::FlexBox::Direction::columnReverse;
                break;
            default:
                ptr->flexbox.flexDirection = juce::FlexBox::Direction::row;
                break;
        }
    } catch (...) {
        // Silently catch exceptions in void functions to prevent crashes
        // In debug builds, this would be logged
    }
}

void flexbox_set_wrap(JuceFlexBox* ptr, int32_t wrap) {
    try {
        if (!ptr) return;
        
        switch (wrap) {
            case 0:
                ptr->flexbox.flexWrap = juce::FlexBox::Wrap::noWrap;
                break;
            case 1:
                ptr->flexbox.flexWrap = juce::FlexBox::Wrap::wrap;
                break;
            case 2:
                ptr->flexbox.flexWrap = juce::FlexBox::Wrap::wrapReverse;
                break;
            default:
                ptr->flexbox.flexWrap = juce::FlexBox::Wrap::noWrap;
                break;
        }
    } catch (...) {
        // Silently catch exceptions in void functions to prevent crashes
    }
}

void flexbox_set_justify_content(JuceFlexBox* ptr, int32_t justify) {
    try {
        if (!ptr) return;
        
        switch (justify) {
            case 0:
                ptr->flexbox.justifyContent = juce::FlexBox::JustifyContent::flexStart;
                break;
            case 1:
                ptr->flexbox.justifyContent = juce::FlexBox::JustifyContent::flexEnd;
                break;
            case 2:
                ptr->flexbox.justifyContent = juce::FlexBox::JustifyContent::center;
                break;
            case 3:
                ptr->flexbox.justifyContent = juce::FlexBox::JustifyContent::spaceBetween;
                break;
            case 4:
                ptr->flexbox.justifyContent = juce::FlexBox::JustifyContent::spaceAround;
                break;
            default:
                ptr->flexbox.justifyContent = juce::FlexBox::JustifyContent::flexStart;
                break;
        }
    } catch (...) {
        // Silently catch exceptions in void functions to prevent crashes
    }
}

void flexbox_set_align_content(JuceFlexBox* ptr, int32_t align) {
    try {
        if (!ptr) return;
        
        switch (align) {
            case 0:
                ptr->flexbox.alignContent = juce::FlexBox::AlignContent::flexStart;
                break;
            case 1:
                ptr->flexbox.alignContent = juce::FlexBox::AlignContent::flexEnd;
                break;
            case 2:
                ptr->flexbox.alignContent = juce::FlexBox::AlignContent::center;
                break;
            case 3:
                ptr->flexbox.alignContent = juce::FlexBox::AlignContent::spaceBetween;
                break;
            case 4:
                ptr->flexbox.alignContent = juce::FlexBox::AlignContent::spaceAround;
                break;
            case 5:
                ptr->flexbox.alignContent = juce::FlexBox::AlignContent::stretch;
                break;
            default:
                ptr->flexbox.alignContent = juce::FlexBox::AlignContent::stretch;
                break;
        }
    } catch (...) {
        // Silently catch exceptions in void functions to prevent crashes
    }
}

void flexbox_set_align_items(JuceFlexBox* ptr, int32_t align) {
    try {
        if (!ptr) return;
        
        switch (align) {
            case 0:
                ptr->flexbox.alignItems = juce::FlexBox::AlignItems::flexStart;
                break;
            case 1:
                ptr->flexbox.alignItems = juce::FlexBox::AlignItems::flexEnd;
                break;
            case 2:
                ptr->flexbox.alignItems = juce::FlexBox::AlignItems::center;
                break;
            case 3:
                ptr->flexbox.alignItems = juce::FlexBox::AlignItems::stretch;
                break;
            default:
                ptr->flexbox.alignItems = juce::FlexBox::AlignItems::stretch;
                break;
        }
    } catch (...) {
        // Silently catch exceptions in void functions to prevent crashes
    }
}

void flexbox_add_item(JuceFlexBox* ptr,
                     JuceComponent* component,
                     float flex_grow,
                     float flex_shrink,
                     float flex_basis,
                     float min_width,
                     float min_height,
                     float max_width,
                     float max_height,
                     float margin_top,
                     float margin_right,
                     float margin_bottom,
                     float margin_left) {
    try {
        if (!ptr || !component || !component->ptr) return;
        
        juce::FlexItem item(*component->ptr);
        
        // Set flex properties
        item.flexGrow = flex_grow;
        item.flexShrink = flex_shrink;
        item.flexBasis = flex_basis;
        
        // Set size constraints
        item.minWidth = min_width;
        item.minHeight = min_height;
        item.maxWidth = max_width;
        item.maxHeight = max_height;
        
        // Set margins
        item.margin = juce::FlexItem::Margin(margin_top, margin_right, margin_bottom, margin_left);
        
        // Add the item to the flexbox
        ptr->flexbox.items.add(item);
    } catch (...) {
        // Silently catch exceptions in void functions to prevent crashes
    }
}

void flexbox_perform_layout(JuceFlexBox* ptr, int32_t x, int32_t y, int32_t width, int32_t height) {
    try {
        if (!ptr) return;
        
        juce::Rectangle<float> bounds(static_cast<float>(x), 
                                      static_cast<float>(y), 
                                      static_cast<float>(width), 
                                      static_cast<float>(height));
        
        ptr->flexbox.performLayout(bounds);
    } catch (...) {
        // Silently catch exceptions in void functions to prevent crashes
    }
}

} // namespace nih_plug_juce
