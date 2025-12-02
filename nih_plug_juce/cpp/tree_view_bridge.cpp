// tree_view_bridge.cpp
// C++ implementation of TreeView FFI bridge functions
//
// This file contains the C++ implementations of FFI functions for JUCE
// TreeView. All functions use exception handling to ensure
// that C++ exceptions are caught at the FFI boundary and converted to error codes
// or messages.

#include "juce_bridge.h"
#include <cstring>

namespace nih_plug_juce {

// ============================================================================
// TreeView Operations
// ============================================================================

// Custom TreeViewItem class that bridges to Rust
class RustTreeViewItem : public juce::TreeViewItem {
public:
    RustTreeViewItem(size_t item_ptr,
                     int32_t (*get_num_sub_items_fn)(size_t),
                     size_t (*get_sub_item_fn)(size_t, int32_t),
                     void (*paint_item_fn)(size_t, size_t, int32_t, int32_t),
                     void (*item_clicked_fn)(size_t),
                     void (*drop_fn)(size_t))
        : item_ptr_(item_ptr),
          get_num_sub_items_fn_(get_num_sub_items_fn),
          get_sub_item_fn_(get_sub_item_fn),
          paint_item_fn_(paint_item_fn),
          item_clicked_fn_(item_clicked_fn),
          drop_fn_(drop_fn),
          sub_items_initialized_(false) {
    }
    
    ~RustTreeViewItem() override {
        // Clean up the Rust item when the C++ item is destroyed
        if (drop_fn_ && item_ptr_) {
            drop_fn_(item_ptr_);
        }
    }
    
    bool mightContainSubItems() override {
        // Check if this item has any sub-items
        if (get_num_sub_items_fn_ && item_ptr_) {
            return get_num_sub_items_fn_(item_ptr_) > 0;
        }
        return false;
    }
    
    void paintItem(juce::Graphics& g, int width, int height) override {
        if (paint_item_fn_ && item_ptr_) {
            // Wrap the Graphics context in a JuceGraphics wrapper
            JuceGraphics graphics_wrapper(&g);
            
            // Pass the wrapper pointer to Rust
            paint_item_fn_(item_ptr_,
                          reinterpret_cast<size_t>(&graphics_wrapper),
                          static_cast<int32_t>(width),
                          static_cast<int32_t>(height));
        }
    }
    
    void itemClicked(const juce::MouseEvent&) override {
        if (item_clicked_fn_ && item_ptr_) {
            item_clicked_fn_(item_ptr_);
        }
    }
    
    void itemOpennessChanged(bool isNowOpen) override {
        // When the item is opened, populate its sub-items
        if (isNowOpen && !sub_items_initialized_) {
            initializeSubItems();
        }
    }
    
private:
    void initializeSubItems() {
        if (sub_items_initialized_ || !get_num_sub_items_fn_ || !get_sub_item_fn_ || !item_ptr_) {
            return;
        }
        
        // Get the number of sub-items from Rust
        int32_t num_sub_items = get_num_sub_items_fn_(item_ptr_);
        
        // Create and add each sub-item
        for (int32_t i = 0; i < num_sub_items; ++i) {
            size_t sub_item_ptr = get_sub_item_fn_(item_ptr_, i);
            
            if (sub_item_ptr != 0) {
                // Create a C++ wrapper for the Rust item
                auto* rust_item = new RustTreeViewItem(
                    sub_item_ptr,
                    get_num_sub_items_fn_,
                    get_sub_item_fn_,
                    paint_item_fn_,
                    item_clicked_fn_,
                    drop_fn_
                );
                
                // Add it to this item's sub-items
                // JUCE takes ownership of the item
                addSubItem(rust_item);
            }
        }
        
        sub_items_initialized_ = true;
    }
    
    size_t item_ptr_;
    int32_t (*get_num_sub_items_fn_)(size_t);
    size_t (*get_sub_item_fn_)(size_t, int32_t);
    void (*paint_item_fn_)(size_t, size_t, int32_t, int32_t);
    void (*item_clicked_fn_)(size_t);
    void (*drop_fn_)(size_t);
    bool sub_items_initialized_;
};

// Create a new JUCE TreeView
JuceComponent* create_tree_view(int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions_ptr([&]() -> JuceComponent* {
        // Create a new TreeView
        auto* tree_view = new juce::TreeView();
        
        if (!tree_view) {
            throw std::runtime_error("Failed to allocate TreeView");
        }
        
        // Wrap in our opaque type
        return new JuceComponent(tree_view);
        
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Set the root item for a tree view
int32_t tree_view_set_root_item(JuceComponent* ptr,
                                size_t item_ptr,
                                size_t get_num_sub_items,
                                size_t get_sub_item,
                                size_t paint_item,
                                size_t item_clicked,
                                size_t drop_fn,
                                int8_t* error_buffer,
                                size_t buffer_size) {
    return catch_exceptions([&]() {
        if (!ptr || !ptr->ptr) {
            throw std::runtime_error("TreeView pointer is null");
        }
        
        // Try to cast to TreeView
        auto* tree_view = dynamic_cast<juce::TreeView*>(ptr->ptr);
        if (!tree_view) {
            throw std::runtime_error("Component is not a TreeView");
        }
        
        // Create the Rust item bridge
        auto get_num_sub_items_fn = reinterpret_cast<int32_t (*)(size_t)>(get_num_sub_items);
        auto get_sub_item_fn = reinterpret_cast<size_t (*)(size_t, int32_t)>(get_sub_item);
        auto paint_item_fn = reinterpret_cast<void (*)(size_t, size_t, int32_t, int32_t)>(paint_item);
        auto item_clicked_fn = reinterpret_cast<void (*)(size_t)>(item_clicked);
        auto drop_item_fn = reinterpret_cast<void (*)(size_t)>(drop_fn);
        
        auto* rust_item = new RustTreeViewItem(item_ptr, get_num_sub_items_fn, get_sub_item_fn,
                                               paint_item_fn, item_clicked_fn, drop_item_fn);
        
        if (!rust_item) {
            throw std::runtime_error("Failed to allocate RustTreeViewItem");
        }
        
        // Set the root item on the tree view
        // The tree view takes ownership of the item and will delete it when destroyed
        tree_view->setRootItem(rust_item);
        
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

} // namespace nih_plug_juce
