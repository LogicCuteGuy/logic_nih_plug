// list_box_bridge.cpp
// C++ implementation of ListBox FFI bridge functions
//
// This file contains the C++ implementations of FFI functions for JUCE
// ListBox. All functions use exception handling to ensure
// that C++ exceptions are caught at the FFI boundary and converted to error codes
// or messages.

#include "juce_bridge.h"
#include <cstring>

namespace nih_plug_juce {

// ============================================================================
// ListBox Operations
// ============================================================================

// Custom ListBoxModel class that bridges to Rust
class RustListBoxModel : public juce::ListBoxModel {
public:
    RustListBoxModel(size_t model_ptr,
                     int32_t (*get_num_rows_fn)(size_t),
                     void (*paint_item_fn)(size_t, int32_t, size_t, int32_t, int32_t, bool),
                     void (*selection_changed_fn)(size_t, int32_t),
                     void (*drop_fn)(size_t))
        : model_ptr_(model_ptr),
          get_num_rows_fn_(get_num_rows_fn),
          paint_item_fn_(paint_item_fn),
          selection_changed_fn_(selection_changed_fn),
          drop_fn_(drop_fn) {
    }
    
    ~RustListBoxModel() override {
        // Clean up the Rust model when the C++ model is destroyed
        if (drop_fn_ && model_ptr_) {
            drop_fn_(model_ptr_);
        }
    }
    
    int getNumRows() override {
        if (get_num_rows_fn_ && model_ptr_) {
            return get_num_rows_fn_(model_ptr_);
        }
        return 0;
    }
    
    void paintListBoxItem(int rowNumber, juce::Graphics& g,
                         int width, int height, bool rowIsSelected) override {
        if (paint_item_fn_ && model_ptr_) {
            // Wrap the Graphics context in a JuceGraphics wrapper
            JuceGraphics graphics_wrapper(&g);
            
            // Pass the wrapper pointer to Rust
            paint_item_fn_(model_ptr_, static_cast<int32_t>(rowNumber),
                          reinterpret_cast<size_t>(&graphics_wrapper),
                          static_cast<int32_t>(width),
                          static_cast<int32_t>(height),
                          rowIsSelected);
        }
    }
    
    void selectedRowsChanged(int lastRowSelected) override {
        if (selection_changed_fn_ && model_ptr_) {
            selection_changed_fn_(model_ptr_, static_cast<int32_t>(lastRowSelected));
        }
    }
    
private:
    size_t model_ptr_;
    int32_t (*get_num_rows_fn_)(size_t);
    void (*paint_item_fn_)(size_t, int32_t, size_t, int32_t, int32_t, bool);
    void (*selection_changed_fn_)(size_t, int32_t);
    void (*drop_fn_)(size_t);
};

// Create a new JUCE ListBox
JuceComponent* create_list_box(int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions_ptr([&]() -> JuceComponent* {
        // Create a new ListBox
        auto* list_box = new juce::ListBox();
        
        if (!list_box) {
            throw std::runtime_error("Failed to allocate ListBox");
        }
        
        // Wrap in our opaque type
        return new JuceComponent(list_box);
        
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Set the model for a list box
int32_t list_box_set_model(JuceComponent* ptr,
                           size_t model_ptr,
                           size_t get_num_rows,
                           size_t paint_item,
                           size_t selection_changed,
                           size_t drop_fn,
                           int8_t* error_buffer,
                           size_t buffer_size) {
    return catch_exceptions([&]() {
        if (!ptr || !ptr->ptr) {
            throw std::runtime_error("ListBox pointer is null");
        }
        
        // Try to cast to ListBox
        auto* list_box = dynamic_cast<juce::ListBox*>(ptr->ptr);
        if (!list_box) {
            throw std::runtime_error("Component is not a ListBox");
        }
        
        // Create the Rust model bridge
        auto get_num_rows_fn = reinterpret_cast<int32_t (*)(size_t)>(get_num_rows);
        auto paint_item_fn = reinterpret_cast<void (*)(size_t, int32_t, size_t, int32_t, int32_t, bool)>(paint_item);
        auto selection_changed_fn = reinterpret_cast<void (*)(size_t, int32_t)>(selection_changed);
        auto drop_model_fn = reinterpret_cast<void (*)(size_t)>(drop_fn);
        
        auto* rust_model = new RustListBoxModel(model_ptr, get_num_rows_fn, paint_item_fn,
                                                selection_changed_fn, drop_model_fn);
        
        if (!rust_model) {
            throw std::runtime_error("Failed to allocate RustListBoxModel");
        }
        
        // Set the model on the list box
        // The list box takes ownership of the model and will delete it when destroyed
        list_box->setModel(rust_model);
        
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Update the content of a list box
void list_box_update_content(JuceComponent* ptr) {
    if (ptr && ptr->ptr) {
        // Try to cast to ListBox
        auto* list_box = dynamic_cast<juce::ListBox*>(ptr->ptr);
        if (list_box) {
            list_box->updateContent();
        }
    }
}

} // namespace nih_plug_juce
