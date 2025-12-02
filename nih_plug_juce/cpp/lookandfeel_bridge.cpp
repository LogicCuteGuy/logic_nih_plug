#include "juce_bridge.h"
#include <juce_gui_basics/juce_gui_basics.h>

namespace nih_plug_juce {

// LookAndFeel operations

JuceLookAndFeel* create_lookandfeel_v4(int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions_ptr([&]() -> JuceLookAndFeel* {
        auto* laf = new juce::LookAndFeel_V4();
        return new JuceLookAndFeel(laf);
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

void delete_lookandfeel(JuceLookAndFeel* ptr) {
    if (ptr && ptr->ptr) {
        delete ptr->ptr;
        delete ptr;
    }
}

void lookandfeel_set_colour(JuceLookAndFeel* ptr, int32_t colour_id, const JuceColour* colour) {
    if (ptr && ptr->ptr && colour) {
        ptr->ptr->setColour(colour_id, colour->colour);
    }
}

const JuceColour* lookandfeel_find_colour(const JuceLookAndFeel* ptr, int32_t colour_id) {
    if (ptr && ptr->ptr) {
        // Create a new Colour object on the heap and return it
        // The Rust side will take ownership and free it when dropped
        auto colour = ptr->ptr->findColour(colour_id);
        return new JuceColour(colour);
    }
    return nullptr;
}

int32_t component_set_look_and_feel(JuceComponent* component, JuceLookAndFeel* laf,
                                     int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions([&]() {
        if (!component || !component->ptr) {
            throw std::runtime_error("Component pointer is null");
        }
        if (!laf || !laf->ptr) {
            throw std::runtime_error("LookAndFeel pointer is null");
        }
        
        component->ptr->setLookAndFeel(laf->ptr);
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

} // namespace nih_plug_juce
