// file_chooser_bridge.cpp
// C++ bridge functions for JUCE FileChooser
//
// This file implements FFI bridge functions for JUCE's FileChooser class,
// which provides native file open/save dialogs.

#include "juce_bridge.h"
#include <memory>
#include <functional>

namespace nih_plug_juce {

// Opaque wrapper for juce::FileChooser
struct JuceFileChooser {
    std::unique_ptr<juce::FileChooser> chooser;
    
    explicit JuceFileChooser(std::unique_ptr<juce::FileChooser> fc) 
        : chooser(std::move(fc)) {}
    ~JuceFileChooser() = default;
};

// Helper class to manage file chooser callbacks
class FileChooserCallback {
public:
    using PathCallback = std::function<void(const juce::String&)>;
    
    FileChooserCallback(size_t rust_closure, size_t invoke, size_t drop_fn)
        : rust_closure_(rust_closure)
        , invoke_(reinterpret_cast<void(*)(size_t, const uint8_t*, size_t)>(invoke))
        , drop_fn_(reinterpret_cast<void(*)(size_t)>(drop_fn))
    {
    }
    
    ~FileChooserCallback() {
        if (drop_fn_ && rust_closure_) {
            drop_fn_(rust_closure_);
        }
    }
    
    void invoke(const juce::String& path) {
        if (invoke_ && rust_closure_) {
            if (path.isEmpty()) {
                // User cancelled - pass null pointer
                invoke_(rust_closure_, nullptr, 0);
            } else {
                // Convert path to UTF-8 and pass to Rust
                auto utf8 = path.toUTF8();
                invoke_(rust_closure_, 
                       reinterpret_cast<const uint8_t*>(utf8.getAddress()), 
                       utf8.sizeInBytes());
            }
        }
    }
    
private:
    size_t rust_closure_;
    void (*invoke_)(size_t, const uint8_t*, size_t);
    void (*drop_fn_)(size_t);
};

// Create a new JUCE FileChooser
JuceFileChooser* create_file_chooser(
    const uint8_t* title,
    size_t title_len,
    const uint8_t* initial_dir,
    size_t initial_dir_len,
    const uint8_t* filters,
    size_t filters_len,
    int8_t* error_buffer,
    size_t buffer_size)
{
    try {
        // Convert UTF-8 bytes to juce::String
        juce::String title_str = juce::String::fromUTF8(
            reinterpret_cast<const char*>(title), 
            static_cast<int>(title_len));
        
        juce::String dir_str = juce::String::fromUTF8(
            reinterpret_cast<const char*>(initial_dir), 
            static_cast<int>(initial_dir_len));
        
        juce::String filters_str = juce::String::fromUTF8(
            reinterpret_cast<const char*>(filters), 
            static_cast<int>(filters_len));
        
        // Create the initial directory File object
        juce::File initial_file(dir_str);
        
        // Create the FileChooser
        auto chooser = std::make_unique<juce::FileChooser>(
            title_str,
            initial_file,
            filters_str
        );
        
        return new JuceFileChooser(std::move(chooser));
    } catch (const std::exception& e) {
        if (error_buffer && buffer_size > 0) {
            std::snprintf(reinterpret_cast<char*>(error_buffer), buffer_size, "%s", e.what());
        }
        return nullptr;
    } catch (...) {
        if (error_buffer && buffer_size > 0) {
            std::snprintf(reinterpret_cast<char*>(error_buffer), buffer_size, "Unknown C++ exception");
        }
        return nullptr;
    }
}

// Delete a JUCE FileChooser
void delete_file_chooser(JuceFileChooser* ptr)
{
    delete ptr;
}

// Browse for a file to open
int32_t file_chooser_browse_for_file_to_open(
    JuceFileChooser* ptr,
    size_t rust_closure,
    size_t invoke,
    size_t drop_fn,
    int8_t* error_buffer,
    size_t buffer_size)
{
    return catch_exceptions([&]() {
        if (!ptr || !ptr->chooser) {
            throw std::runtime_error("Invalid FileChooser pointer");
        }
        
        // Create a callback wrapper that will be owned by the lambda
        auto callback = std::make_shared<FileChooserCallback>(rust_closure, invoke, drop_fn);
        
        // Launch the file browser asynchronously
        ptr->chooser->launchAsync(
            juce::FileBrowserComponent::openMode | juce::FileBrowserComponent::canSelectFiles,
            [callback](const juce::FileChooser& chooser) {
                // Get the selected file
                juce::File result = chooser.getResult();
                
                if (result == juce::File()) {
                    // User cancelled - pass empty string
                    callback->invoke(juce::String());
                } else {
                    // User selected a file - pass the full path
                    callback->invoke(result.getFullPathName());
                }
            }
        );
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Browse for a file to save
int32_t file_chooser_browse_for_file_to_save(
    JuceFileChooser* ptr,
    size_t rust_closure,
    size_t invoke,
    size_t drop_fn,
    int8_t* error_buffer,
    size_t buffer_size)
{
    return catch_exceptions([&]() {
        if (!ptr || !ptr->chooser) {
            throw std::runtime_error("Invalid FileChooser pointer");
        }
        
        // Create a callback wrapper that will be owned by the lambda
        auto callback = std::make_shared<FileChooserCallback>(rust_closure, invoke, drop_fn);
        
        // Launch the file browser asynchronously
        ptr->chooser->launchAsync(
            juce::FileBrowserComponent::saveMode | juce::FileBrowserComponent::canSelectFiles,
            [callback](const juce::FileChooser& chooser) {
                // Get the selected file
                juce::File result = chooser.getResult();
                
                if (result == juce::File()) {
                    // User cancelled - pass empty string
                    callback->invoke(juce::String());
                } else {
                    // User selected a file - pass the full path
                    callback->invoke(result.getFullPathName());
                }
            }
        );
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

} // namespace nih_plug_juce
