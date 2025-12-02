// widget_bridge.cpp
// C++ implementation of Widget FFI bridge functions
//
// This file contains the C++ implementations of FFI functions for JUCE widget
// components (Button, Label, etc.). All functions use exception handling to ensure
// that C++ exceptions are caught at the FFI boundary and converted to error codes
// or messages.

#include "juce_bridge.h"
#include <cstring>

namespace nih_plug_juce {

// ============================================================================
// Button Operations
// ============================================================================

// Custom button class that supports click callbacks
class CallbackButton : public juce::TextButton {
public:
    CallbackButton(const juce::String& text) : juce::TextButton(text) {
        callback_closure = 0;
        callback_invoke = nullptr;
        callback_drop = nullptr;
    }
    
    ~CallbackButton() override {
        // Clean up the Rust closure when the button is destroyed
        if (callback_drop && callback_closure) {
            callback_drop(callback_closure);
        }
    }
    
    void setClickCallback(size_t closure, void (*invoke)(size_t), void (*drop)(size_t)) {
        // Clean up old callback if it exists
        if (callback_drop && callback_closure) {
            callback_drop(callback_closure);
        }
        
        callback_closure = closure;
        callback_invoke = invoke;
        callback_drop = drop;
        
        // Set the onClick lambda
        onClick = [this]() {
            if (callback_invoke && callback_closure) {
                callback_invoke(callback_closure);
            }
        };
    }
    
private:
    size_t callback_closure;
    void (*callback_invoke)(size_t);
    void (*callback_drop)(size_t);
};

// Create a new JUCE TextButton
JuceComponent* create_text_button(const uint8_t* text, size_t text_len,
                                  int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions_ptr([&]() -> JuceComponent* {
        // Convert UTF-8 bytes to juce::String
        juce::String button_text = juce::String::fromUTF8(reinterpret_cast<const char*>(text), static_cast<int>(text_len));
        
        // Create a new CallbackButton
        auto* button = new CallbackButton(button_text);
        
        if (!button) {
            throw std::runtime_error("Failed to allocate TextButton");
        }
        
        // Wrap in our opaque type
        return new JuceComponent(button);
        
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Set the text of a button
void button_set_text(JuceComponent* ptr, const uint8_t* text, size_t text_len) {
    try {
        if (ptr && ptr->ptr) {
            // Try to cast to TextButton
            auto* button = dynamic_cast<juce::TextButton*>(ptr->ptr);
            if (button) {
                // Convert UTF-8 bytes to juce::String
                juce::String button_text = juce::String::fromUTF8(reinterpret_cast<const char*>(text), static_cast<int>(text_len));
                button->setButtonText(button_text);
            }
        }
    } catch (...) {
        // Silently catch exceptions in void functions to prevent crashes
    }
}

// Set whether a button is enabled
void button_set_enabled(JuceComponent* ptr, bool enabled) {
    try {
        if (ptr && ptr->ptr) {
            ptr->ptr->setEnabled(enabled);
        }
    } catch (...) {
        // Silently catch exceptions in void functions to prevent crashes
    }
}

// Set a color for a button
void button_set_colour(JuceComponent* ptr, int32_t colour_id, 
                      uint8_t r, uint8_t g, uint8_t b, uint8_t a) {
    try {
        if (ptr && ptr->ptr) {
            juce::Colour colour(r, g, b, a);
            ptr->ptr->setColour(colour_id, colour);
        }
    } catch (...) {
        // Silently catch exceptions in void functions to prevent crashes
    }
}

// Set a click callback for a button
int32_t button_set_on_click(JuceComponent* ptr,
                            size_t rust_closure,
                            size_t invoke,
                            size_t drop_fn,
                            int8_t* error_buffer,
                            size_t buffer_size) {
    return catch_exceptions([&]() {
        if (!ptr) {
            throw std::invalid_argument("Button pointer is null");
        }
        if (!ptr->ptr) {
            throw std::invalid_argument("Button is invalid");
        }
        
        // Try to cast to CallbackButton
        auto* callback_btn = dynamic_cast<CallbackButton*>(ptr->ptr);
        if (!callback_btn) {
            throw std::runtime_error("Button does not support callbacks. "
                                   "This should not happen - all buttons created through FFI support callbacks.");
        }
        
        // Set the click callback
        callback_btn->setClickCallback(
            rust_closure,
            reinterpret_cast<void (*)(size_t)>(invoke),
            reinterpret_cast<void (*)(size_t)>(drop_fn)
        );
        
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// ============================================================================
// Label Operations
// ============================================================================

// Custom label class that supports text change callbacks
class CallbackLabel : public juce::Label {
public:
    CallbackLabel(const juce::String& text) : juce::Label(juce::String(), text) {
        callback_closure = 0;
        callback_invoke = nullptr;
        callback_drop = nullptr;
    }
    
    ~CallbackLabel() override {
        // Clean up the Rust closure when the label is destroyed
        if (callback_drop && callback_closure) {
            callback_drop(callback_closure);
        }
    }
    
    void setTextChangeCallback(size_t closure, 
                              void (*invoke)(size_t, const uint8_t*, size_t), 
                              void (*drop)(size_t)) {
        // Clean up old callback if it exists
        if (callback_drop && callback_closure) {
            callback_drop(callback_closure);
        }
        
        callback_closure = closure;
        callback_invoke = invoke;
        callback_drop = drop;
        
        // Set the onTextChange lambda
        onTextChange = [this]() {
            if (callback_invoke && callback_closure) {
                juce::String text = getText();
                auto utf8 = text.toRawUTF8();
                callback_invoke(callback_closure, 
                              reinterpret_cast<const uint8_t*>(utf8), 
                              std::strlen(utf8));
            }
        };
    }
    
private:
    size_t callback_closure;
    void (*callback_invoke)(size_t, const uint8_t*, size_t);
    void (*callback_drop)(size_t);
};

// Create a new JUCE Label
JuceComponent* create_label(const uint8_t* text, size_t text_len,
                           int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions_ptr([&]() -> JuceComponent* {
        // Convert UTF-8 bytes to juce::String
        juce::String label_text = juce::String::fromUTF8(reinterpret_cast<const char*>(text), static_cast<int>(text_len));
        
        // Create a new CallbackLabel
        auto* label = new CallbackLabel(label_text);
        
        if (!label) {
            throw std::runtime_error("Failed to allocate Label");
        }
        
        // Wrap in our opaque type
        return new JuceComponent(label);
        
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Set the text of a label
void label_set_text(JuceComponent* ptr, const uint8_t* text, size_t text_len) {
    try {
        if (ptr && ptr->ptr) {
            // Try to cast to Label
            auto* label = dynamic_cast<juce::Label*>(ptr->ptr);
            if (label) {
                // Convert UTF-8 bytes to juce::String
                juce::String label_text = juce::String::fromUTF8(reinterpret_cast<const char*>(text), static_cast<int>(text_len));
                label->setText(label_text, juce::dontSendNotification);
            }
        }
    } catch (...) {
        // Silently catch exceptions in void functions to prevent crashes
    }
}

// Set the font of a label
void label_set_font(JuceComponent* ptr, float font_size) {
    try {
        if (ptr && ptr->ptr) {
            // Try to cast to Label
            auto* label = dynamic_cast<juce::Label*>(ptr->ptr);
            if (label) {
                juce::Font font(font_size);
                label->setFont(font);
            }
        }
    } catch (...) {
        // Silently catch exceptions in void functions to prevent crashes
    }
}

// Set the text justification of a label
void label_set_justification(JuceComponent* ptr, int32_t justification) {
    try {
        if (ptr && ptr->ptr) {
            // Try to cast to Label
            auto* label = dynamic_cast<juce::Label*>(ptr->ptr);
            if (label) {
                label->setJustificationType(juce::Justification(justification));
            }
        }
    } catch (...) {
        // Silently catch exceptions in void functions to prevent crashes
    }
}

// Set whether a label is editable
void label_set_editable(JuceComponent* ptr, bool editable) {
    try {
        if (ptr && ptr->ptr) {
            // Try to cast to Label
            auto* label = dynamic_cast<juce::Label*>(ptr->ptr);
            if (label) {
                label->setEditable(editable);
            }
        }
    } catch (...) {
        // Silently catch exceptions in void functions to prevent crashes
    }
}

// Set a text change callback for a label
int32_t label_set_on_text_change(JuceComponent* ptr,
                                 size_t rust_closure,
                                 size_t invoke,
                                 size_t drop_fn,
                                 int8_t* error_buffer,
                                 size_t buffer_size) {
    return catch_exceptions([&]() {
        if (!ptr) {
            throw std::invalid_argument("Label pointer is null");
        }
        if (!ptr->ptr) {
            throw std::invalid_argument("Label is invalid");
        }
        
        // Try to cast to CallbackLabel
        auto* callback_label = dynamic_cast<CallbackLabel*>(ptr->ptr);
        if (!callback_label) {
            throw std::runtime_error("Label does not support callbacks. "
                                   "This should not happen - all labels created through FFI support callbacks.");
        }
        
        // Set the text change callback
        callback_label->setTextChangeCallback(
            rust_closure,
            reinterpret_cast<void (*)(size_t, const uint8_t*, size_t)>(invoke),
            reinterpret_cast<void (*)(size_t)>(drop_fn)
        );
        
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// ============================================================================
// Slider Operations
// ============================================================================

// Custom slider class that supports value change callbacks
class CallbackSlider : public juce::Slider {
public:
    CallbackSlider(juce::Slider::SliderStyle style) : juce::Slider(style, juce::Slider::NoTextBox) {
        callback_closure = 0;
        callback_invoke = nullptr;
        callback_drop = nullptr;
    }
    
    ~CallbackSlider() override {
        // Clean up the Rust closure when the slider is destroyed
        if (callback_drop && callback_closure) {
            callback_drop(callback_closure);
        }
    }
    
    void setValueChangeCallback(size_t closure, 
                               void (*invoke)(size_t, double), 
                               void (*drop)(size_t)) {
        // Clean up old callback if it exists
        if (callback_drop && callback_closure) {
            callback_drop(callback_closure);
        }
        
        callback_closure = closure;
        callback_invoke = invoke;
        callback_drop = drop;
        
        // Set the onValueChange lambda
        onValueChange = [this]() {
            if (callback_invoke && callback_closure) {
                callback_invoke(callback_closure, getValue());
            }
        };
    }
    
private:
    size_t callback_closure;
    void (*callback_invoke)(size_t, double);
    void (*callback_drop)(size_t);
};

// Create a new JUCE Slider
JuceComponent* create_slider(int32_t style, int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions_ptr([&]() -> JuceComponent* {
        // Map style integer to JUCE SliderStyle enum
        juce::Slider::SliderStyle slider_style;
        switch (style) {
            case 1: slider_style = juce::Slider::LinearHorizontal; break;
            case 2: slider_style = juce::Slider::LinearVertical; break;
            case 3: slider_style = juce::Slider::LinearBar; break;
            case 4: slider_style = juce::Slider::LinearBarVertical; break;
            case 5: slider_style = juce::Slider::Rotary; break;
            case 6: slider_style = juce::Slider::RotaryHorizontalDrag; break;
            case 7: slider_style = juce::Slider::RotaryVerticalDrag; break;
            case 8: slider_style = juce::Slider::RotaryHorizontalVerticalDrag; break;
            case 9: slider_style = juce::Slider::TwoValueHorizontal; break;
            case 10: slider_style = juce::Slider::TwoValueVertical; break;
            case 11: slider_style = juce::Slider::ThreeValueHorizontal; break;
            case 12: slider_style = juce::Slider::ThreeValueVertical; break;
            default:
                throw std::invalid_argument("Invalid slider style");
        }
        
        // Create a new CallbackSlider
        auto* slider = new CallbackSlider(slider_style);
        
        if (!slider) {
            throw std::runtime_error("Failed to allocate Slider");
        }
        
        // Set default range
        slider->setRange(0.0, 1.0, 0.0);
        
        // Wrap in our opaque type
        return new JuceComponent(slider);
        
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Set the range of a slider
void slider_set_range(JuceComponent* ptr, double min, double max, double interval) {
    try {
        if (ptr && ptr->ptr) {
            // Try to cast to Slider
            auto* slider = dynamic_cast<juce::Slider*>(ptr->ptr);
            if (slider) {
                slider->setRange(min, max, interval);
            }
        }
    } catch (...) {
        // Silently catch exceptions in void functions to prevent crashes
    }
}

// Set the value of a slider
void slider_set_value(JuceComponent* ptr, double value) {
    try {
        if (ptr && ptr->ptr) {
            // Try to cast to Slider
            auto* slider = dynamic_cast<juce::Slider*>(ptr->ptr);
            if (slider) {
                slider->setValue(value, juce::dontSendNotification);
            }
        }
    } catch (...) {
        // Silently catch exceptions in void functions to prevent crashes
    }
}

// Get the current value of a slider
double slider_get_value(const JuceComponent* ptr) {
    if (ptr && ptr->ptr) {
        // Try to cast to Slider
        auto* slider = dynamic_cast<juce::Slider*>(ptr->ptr);
        if (slider) {
            return slider->getValue();
        }
    }
    return 0.0;
}

// Set a value change callback for a slider
int32_t slider_set_on_value_change(JuceComponent* ptr,
                                   size_t rust_closure,
                                   size_t invoke,
                                   size_t drop_fn,
                                   int8_t* error_buffer,
                                   size_t buffer_size) {
    return catch_exceptions([&]() {
        if (!ptr) {
            throw std::invalid_argument("Slider pointer is null");
        }
        if (!ptr->ptr) {
            throw std::invalid_argument("Slider is invalid");
        }
        
        // Try to cast to CallbackSlider
        auto* callback_slider = dynamic_cast<CallbackSlider*>(ptr->ptr);
        if (!callback_slider) {
            throw std::runtime_error("Slider does not support callbacks. "
                                   "This should not happen - all sliders created through FFI support callbacks.");
        }
        
        // Set the value change callback
        callback_slider->setValueChangeCallback(
            rust_closure,
            reinterpret_cast<void (*)(size_t, double)>(invoke),
            reinterpret_cast<void (*)(size_t)>(drop_fn)
        );
        
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// ============================================================================
// ComboBox Operations
// ============================================================================

// Custom combo box class that supports change callbacks
class CallbackComboBox : public juce::ComboBox {
public:
    CallbackComboBox() : juce::ComboBox() {
        callback_closure = 0;
        callback_invoke = nullptr;
        callback_drop = nullptr;
    }
    
    ~CallbackComboBox() override {
        // Clean up the Rust closure when the combo box is destroyed
        if (callback_drop && callback_closure) {
            callback_drop(callback_closure);
        }
    }
    
    void setChangeCallback(size_t closure, 
                          void (*invoke)(size_t, int32_t), 
                          void (*drop)(size_t)) {
        // Clean up old callback if it exists
        if (callback_drop && callback_closure) {
            callback_drop(callback_closure);
        }
        
        callback_closure = closure;
        callback_invoke = invoke;
        callback_drop = drop;
        
        // Set the onChange lambda
        onChange = [this]() {
            if (callback_invoke && callback_closure) {
                callback_invoke(callback_closure, getSelectedId());
            }
        };
    }
    
private:
    size_t callback_closure;
    void (*callback_invoke)(size_t, int32_t);
    void (*callback_drop)(size_t);
};

// Create a new JUCE ComboBox
JuceComponent* create_combo_box(int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions_ptr([&]() -> JuceComponent* {
        // Create a new CallbackComboBox
        auto* combo = new CallbackComboBox();
        
        if (!combo) {
            throw std::runtime_error("Failed to allocate ComboBox");
        }
        
        // Wrap in our opaque type
        return new JuceComponent(combo);
        
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Add an item to a combo box
void combo_box_add_item(JuceComponent* ptr, const uint8_t* text, size_t text_len, int32_t item_id) {
    try {
        if (ptr && ptr->ptr) {
            // Try to cast to ComboBox
            auto* combo = dynamic_cast<juce::ComboBox*>(ptr->ptr);
            if (combo) {
                // Convert UTF-8 bytes to juce::String
                juce::String item_text = juce::String::fromUTF8(reinterpret_cast<const char*>(text), static_cast<int>(text_len));
                combo->addItem(item_text, item_id);
            }
        }
    } catch (...) {
        // Silently catch exceptions in void functions to prevent crashes
    }
}

// Clear all items from a combo box
void combo_box_clear(JuceComponent* ptr) {
    try {
        if (ptr && ptr->ptr) {
            // Try to cast to ComboBox
            auto* combo = dynamic_cast<juce::ComboBox*>(ptr->ptr);
            if (combo) {
                combo->clear();
            }
        }
    } catch (...) {
        // Silently catch exceptions in void functions to prevent crashes
    }
}

// Set the selected item by ID
void combo_box_set_selected_id(JuceComponent* ptr, int32_t item_id) {
    try {
        if (ptr && ptr->ptr) {
            // Try to cast to ComboBox
            auto* combo = dynamic_cast<juce::ComboBox*>(ptr->ptr);
            if (combo) {
                combo->setSelectedId(item_id, juce::dontSendNotification);
            }
        }
    } catch (...) {
        // Silently catch exceptions in void functions to prevent crashes
    }
}

// Set the selected item by index
void combo_box_set_selected_index(JuceComponent* ptr, int32_t index) {
    try {
        if (ptr && ptr->ptr) {
            // Try to cast to ComboBox
            auto* combo = dynamic_cast<juce::ComboBox*>(ptr->ptr);
            if (combo) {
                combo->setSelectedItemIndex(index, juce::dontSendNotification);
            }
        }
    } catch (...) {
        // Silently catch exceptions in void functions to prevent crashes
    }
}

// Get the ID of the currently selected item
int32_t combo_box_get_selected_id(const JuceComponent* ptr) {
    if (ptr && ptr->ptr) {
        // Try to cast to ComboBox
        auto* combo = dynamic_cast<juce::ComboBox*>(ptr->ptr);
        if (combo) {
            return combo->getSelectedId();
        }
    }
    return 0;
}

// Set a change callback for a combo box
int32_t combo_box_set_on_change(JuceComponent* ptr,
                                size_t rust_closure,
                                size_t invoke,
                                size_t drop_fn,
                                int8_t* error_buffer,
                                size_t buffer_size) {
    return catch_exceptions([&]() {
        if (!ptr) {
            throw std::invalid_argument("ComboBox pointer is null");
        }
        if (!ptr->ptr) {
            throw std::invalid_argument("ComboBox is invalid");
        }
        
        // Try to cast to CallbackComboBox
        auto* callback_combo = dynamic_cast<CallbackComboBox*>(ptr->ptr);
        if (!callback_combo) {
            throw std::runtime_error("ComboBox does not support callbacks. "
                                   "This should not happen - all combo boxes created through FFI support callbacks.");
        }
        
        // Set the change callback
        callback_combo->setChangeCallback(
            rust_closure,
            reinterpret_cast<void (*)(size_t, int32_t)>(invoke),
            reinterpret_cast<void (*)(size_t)>(drop_fn)
        );
        
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// ============================================================================
// TextEditor Operations
// ============================================================================

// Custom text editor class that supports text change callbacks
class CallbackTextEditor : public juce::TextEditor {
public:
    CallbackTextEditor() : juce::TextEditor() {
        callback_closure = 0;
        callback_invoke = nullptr;
        callback_drop = nullptr;
    }
    
    ~CallbackTextEditor() override {
        // Clean up the Rust closure when the text editor is destroyed
        if (callback_drop && callback_closure) {
            callback_drop(callback_closure);
        }
    }
    
    void setTextChangeCallback(size_t closure, 
                              void (*invoke)(size_t, const uint8_t*, size_t), 
                              void (*drop)(size_t)) {
        // Clean up old callback if it exists
        if (callback_drop && callback_closure) {
            callback_drop(callback_closure);
        }
        
        callback_closure = closure;
        callback_invoke = invoke;
        callback_drop = drop;
        
        // Set the onTextChange lambda
        onTextChange = [this]() {
            if (callback_invoke && callback_closure) {
                juce::String text = getText();
                auto utf8 = text.toRawUTF8();
                callback_invoke(callback_closure, 
                              reinterpret_cast<const uint8_t*>(utf8), 
                              std::strlen(utf8));
            }
        };
    }
    
private:
    size_t callback_closure;
    void (*callback_invoke)(size_t, const uint8_t*, size_t);
    void (*callback_drop)(size_t);
};

// Create a new JUCE TextEditor
JuceComponent* create_text_editor(int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions_ptr([&]() -> JuceComponent* {
        // Create a new CallbackTextEditor
        auto* editor = new CallbackTextEditor();
        
        if (!editor) {
            throw std::runtime_error("Failed to allocate TextEditor");
        }
        
        // Wrap in our opaque type
        return new JuceComponent(editor);
        
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Set the text of a text editor
void text_editor_set_text(JuceComponent* ptr, const uint8_t* text, size_t text_len) {
    try {
        if (ptr && ptr->ptr) {
            // Try to cast to TextEditor
            auto* editor = dynamic_cast<juce::TextEditor*>(ptr->ptr);
            if (editor) {
                // Convert UTF-8 bytes to juce::String
                juce::String editor_text = juce::String::fromUTF8(reinterpret_cast<const char*>(text), static_cast<int>(text_len));
                editor->setText(editor_text, false);
            }
        }
    } catch (...) {
        // Silently catch exceptions in void functions to prevent crashes
    }
}

// Get the text from a text editor
size_t text_editor_get_text(const JuceComponent* ptr, uint8_t* buffer, size_t buffer_size) {
    if (ptr && ptr->ptr && buffer && buffer_size > 0) {
        // Try to cast to TextEditor
        auto* editor = dynamic_cast<juce::TextEditor*>(ptr->ptr);
        if (editor) {
            juce::String text = editor->getText();
            auto utf8 = text.toRawUTF8();
            size_t text_len = std::strlen(utf8);
            
            // Copy as much as will fit in the buffer
            size_t copy_len = std::min(text_len, buffer_size - 1);
            std::memcpy(buffer, utf8, copy_len);
            buffer[copy_len] = '\0';
            
            // Return the actual length (not including null terminator)
            return text_len;
        }
    }
    
    if (buffer && buffer_size > 0) {
        buffer[0] = '\0';
    }
    return 0;
}

// Set whether a text editor is multiline
void text_editor_set_multiline(JuceComponent* ptr, bool multiline) {
    try {
        if (ptr && ptr->ptr) {
            // Try to cast to TextEditor
            auto* editor = dynamic_cast<juce::TextEditor*>(ptr->ptr);
            if (editor) {
                editor->setMultiLine(multiline);
            }
        }
    } catch (...) {
        // Silently catch exceptions in void functions to prevent crashes
    }
}

// Set whether a text editor is read-only
void text_editor_set_readonly(JuceComponent* ptr, bool readonly) {
    try {
        if (ptr && ptr->ptr) {
            // Try to cast to TextEditor
            auto* editor = dynamic_cast<juce::TextEditor*>(ptr->ptr);
            if (editor) {
                editor->setReadOnly(readonly);
            }
        }
    } catch (...) {
        // Silently catch exceptions in void functions to prevent crashes
    }
}

// Set a text change callback for a text editor
int32_t text_editor_set_on_text_change(JuceComponent* ptr,
                                       size_t rust_closure,
                                       size_t invoke,
                                       size_t drop_fn,
                                       int8_t* error_buffer,
                                       size_t buffer_size) {
    return catch_exceptions([&]() {
        if (!ptr) {
            throw std::invalid_argument("TextEditor pointer is null");
        }
        if (!ptr->ptr) {
            throw std::invalid_argument("TextEditor is invalid");
        }
        
        // Try to cast to CallbackTextEditor
        auto* callback_editor = dynamic_cast<CallbackTextEditor*>(ptr->ptr);
        if (!callback_editor) {
            throw std::runtime_error("TextEditor does not support callbacks. "
                                   "This should not happen - all text editors created through FFI support callbacks.");
        }
        
        // Set the text change callback
        callback_editor->setTextChangeCallback(
            rust_closure,
            reinterpret_cast<void (*)(size_t, const uint8_t*, size_t)>(invoke),
            reinterpret_cast<void (*)(size_t)>(drop_fn)
        );
        
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// ============================================================================
// ToggleButton Operations
// ============================================================================

// Custom toggle button class that supports click callbacks with state
class CallbackToggleButton : public juce::ToggleButton {
public:
    CallbackToggleButton(const juce::String& text) : juce::ToggleButton(text) {
        callback_closure = 0;
        callback_invoke = nullptr;
        callback_drop = nullptr;
    }
    
    ~CallbackToggleButton() override {
        // Clean up the Rust closure when the button is destroyed
        if (callback_drop && callback_closure) {
            callback_drop(callback_closure);
        }
    }
    
    void setClickCallback(size_t closure, void (*invoke)(size_t, bool), void (*drop)(size_t)) {
        // Clean up old callback if it exists
        if (callback_drop && callback_closure) {
            callback_drop(callback_closure);
        }
        
        callback_closure = closure;
        callback_invoke = invoke;
        callback_drop = drop;
        
        // Set the onClick lambda
        onClick = [this]() {
            if (callback_invoke && callback_closure) {
                callback_invoke(callback_closure, getToggleState());
            }
        };
    }
    
private:
    size_t callback_closure;
    void (*callback_invoke)(size_t, bool);
    void (*callback_drop)(size_t);
};

// Create a new JUCE ToggleButton
JuceComponent* create_toggle_button(const uint8_t* text, size_t text_len,
                                    int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions_ptr([&]() -> JuceComponent* {
        // Convert UTF-8 bytes to juce::String
        juce::String button_text = juce::String::fromUTF8(reinterpret_cast<const char*>(text), static_cast<int>(text_len));
        
        // Create a new CallbackToggleButton
        auto* button = new CallbackToggleButton(button_text);
        
        if (!button) {
            throw std::runtime_error("Failed to allocate ToggleButton");
        }
        
        // Wrap in our opaque type
        return new JuceComponent(button);
        
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Set the toggle state of a toggle button
void toggle_button_set_toggle_state(JuceComponent* ptr, bool state) {
    try {
        if (ptr && ptr->ptr) {
            // Try to cast to ToggleButton
            auto* button = dynamic_cast<juce::ToggleButton*>(ptr->ptr);
            if (button) {
                button->setToggleState(state, juce::dontSendNotification);
            }
        }
    } catch (...) {
        // Silently catch exceptions in void functions to prevent crashes
    }
}

// Get the current toggle state of a toggle button
bool toggle_button_get_toggle_state(const JuceComponent* ptr) {
    if (ptr && ptr->ptr) {
        // Try to cast to ToggleButton
        auto* button = dynamic_cast<juce::ToggleButton*>(ptr->ptr);
        if (button) {
            return button->getToggleState();
        }
    }
    return false;
}

// Set the radio group ID for a toggle button
void toggle_button_set_radio_group_id(JuceComponent* ptr, int32_t group_id) {
    try {
        if (ptr && ptr->ptr) {
            // Try to cast to ToggleButton
            auto* button = dynamic_cast<juce::ToggleButton*>(ptr->ptr);
            if (button) {
                button->setRadioGroupId(group_id);
            }
        }
    } catch (...) {
        // Silently catch exceptions in void functions to prevent crashes
    }
}

// Set the text of a toggle button
void toggle_button_set_text(JuceComponent* ptr, const uint8_t* text, size_t text_len) {
    try {
        if (ptr && ptr->ptr) {
            // Try to cast to ToggleButton
            auto* button = dynamic_cast<juce::ToggleButton*>(ptr->ptr);
            if (button) {
                // Convert UTF-8 bytes to juce::String
                juce::String button_text = juce::String::fromUTF8(reinterpret_cast<const char*>(text), static_cast<int>(text_len));
                button->setButtonText(button_text);
            }
        }
    } catch (...) {
        // Silently catch exceptions in void functions to prevent crashes
    }
}

// Set a click callback for a toggle button
int32_t toggle_button_set_on_click(JuceComponent* ptr,
                                   size_t rust_closure,
                                   size_t invoke,
                                   size_t drop_fn,
                                   int8_t* error_buffer,
                                   size_t buffer_size) {
    return catch_exceptions([&]() {
        if (!ptr) {
            throw std::invalid_argument("ToggleButton pointer is null");
        }
        if (!ptr->ptr) {
            throw std::invalid_argument("ToggleButton is invalid");
        }
        
        // Try to cast to CallbackToggleButton
        auto* callback_btn = dynamic_cast<CallbackToggleButton*>(ptr->ptr);
        if (!callback_btn) {
            throw std::runtime_error("ToggleButton does not support callbacks. "
                                   "This should not happen - all toggle buttons created through FFI support callbacks.");
        }
        
        // Set the click callback
        callback_btn->setClickCallback(
            rust_closure,
            reinterpret_cast<void (*)(size_t, bool)>(invoke),
            reinterpret_cast<void (*)(size_t)>(drop_fn)
        );
        
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

} // namespace nih_plug_juce
