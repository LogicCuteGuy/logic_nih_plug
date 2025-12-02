// image_bridge.cpp
// C++ implementation of Image FFI bridge functions
//
// This file contains the C++ implementations of FFI functions for JUCE Image
// operations. Images are value types in JUCE with internal reference counting,
// so they can be safely copied and manipulated.

#include "juce_bridge.h"
#include <cstring>

namespace nih_plug_juce {

// Create a new JUCE Image with the specified format and dimensions
JuceImage* create_image(int32_t format, int32_t width, int32_t height,
                        int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions_ptr([&]() -> JuceImage* {
        if (width <= 0 || height <= 0) {
            throw std::invalid_argument("Image dimensions must be positive");
        }
        
        // Convert format code to JUCE Image::PixelFormat
        juce::Image::PixelFormat pixelFormat;
        switch (format) {
            case 1: // RGB
                pixelFormat = juce::Image::RGB;
                break;
            case 2: // ARGB
                pixelFormat = juce::Image::ARGB;
                break;
            case 3: // SingleChannel
                pixelFormat = juce::Image::SingleChannel;
                break;
            default:
                throw std::invalid_argument("Invalid image format");
        }
        
        juce::Image image(pixelFormat, width, height, true);
        
        if (!image.isValid()) {
            throw std::runtime_error("Failed to create image");
        }
        
        return new JuceImage(image);
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Load a JUCE Image from a file
JuceImage* load_image_from_file(const uint8_t* path, size_t path_len,
                                int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions_ptr([&]() -> JuceImage* {
        if (!path || path_len == 0) {
            throw std::invalid_argument("File path cannot be empty");
        }
        
        // Convert UTF-8 bytes to juce::String
        juce::String pathString(reinterpret_cast<const char*>(path), path_len);
        
        // Create a File object
        juce::File file(pathString);
        
        if (!file.existsAsFile()) {
            throw std::runtime_error("File does not exist: " + pathString.toStdString());
        }
        
        // Load the image
        juce::Image image = juce::ImageFileFormat::loadFrom(file);
        
        if (!image.isValid()) {
            throw std::runtime_error("Failed to load image from file: " + pathString.toStdString());
        }
        
        return new JuceImage(image);
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Save a JUCE Image to a file
int32_t save_image_to_file(const JuceImage* ptr, const uint8_t* path, size_t path_len,
                           int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions([&]() {
        if (!ptr) {
            throw std::invalid_argument("Image pointer cannot be null");
        }
        
        if (!path || path_len == 0) {
            throw std::invalid_argument("File path cannot be empty");
        }
        
        if (!ptr->image.isValid()) {
            throw std::runtime_error("Image is not valid");
        }
        
        // Convert UTF-8 bytes to juce::String
        juce::String pathString(reinterpret_cast<const char*>(path), path_len);
        
        // Create a File object
        juce::File file(pathString);
        
        // Determine format from file extension
        juce::String extension = file.getFileExtension().toLowerCase();
        
        // Create an output stream
        std::unique_ptr<juce::FileOutputStream> outputStream(file.createOutputStream());
        
        if (!outputStream) {
            throw std::runtime_error("Failed to create output stream for file: " + pathString.toStdString());
        }
        
        // Choose the appropriate image format
        bool success = false;
        
        if (extension == ".png") {
            juce::PNGImageFormat pngFormat;
            success = pngFormat.writeImageToStream(ptr->image, *outputStream);
        } else if (extension == ".jpg" || extension == ".jpeg") {
            juce::JPEGImageFormat jpegFormat;
            success = jpegFormat.writeImageToStream(ptr->image, *outputStream);
        } else if (extension == ".gif") {
            juce::GIFImageFormat gifFormat;
            success = gifFormat.writeImageToStream(ptr->image, *outputStream);
        } else {
            // Default to PNG for unknown extensions
            juce::PNGImageFormat pngFormat;
            success = pngFormat.writeImageToStream(ptr->image, *outputStream);
        }
        
        if (!success) {
            throw std::runtime_error("Failed to write image to file: " + pathString.toStdString());
        }
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Delete a JUCE Image and free its resources
void delete_image(JuceImage* ptr) {
    if (ptr) {
        delete ptr;
    }
}

// Get a graphics context for drawing to an image
JuceGraphics* image_get_graphics_context(JuceImage* ptr,
                                         int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions_ptr([&]() -> JuceGraphics* {
        if (!ptr) {
            throw std::invalid_argument("Image pointer cannot be null");
        }
        
        if (!ptr->image.isValid()) {
            throw std::runtime_error("Image is not valid");
        }
        
        // Create a Graphics object for the image
        // Note: The Graphics object must be kept alive as long as it's being used
        // JUCE's Image::BitmapData provides access to the pixel data
        juce::Graphics* graphics = new juce::Graphics(ptr->image);
        
        return new JuceGraphics(graphics);
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Apply a blur effect to an image
int32_t image_apply_blur(JuceImage* ptr, float radius,
                        int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions([&]() {
        if (!ptr) {
            throw std::invalid_argument("Image pointer cannot be null");
        }
        
        if (!ptr->image.isValid()) {
            throw std::runtime_error("Image is not valid");
        }
        
        if (radius < 0.0f) {
            throw std::invalid_argument("Blur radius must be non-negative");
        }
        
        // Apply Gaussian blur using JUCE's image processing
        // JUCE provides blur through ImageConvolutionKernel
        // For a simple implementation, we'll use a box blur approximation
        
        if (radius <= 0.0f) {
            return; // No blur needed
        }
        
        juce::Image blurred = ptr->image.createCopy();
        
        // Get image dimensions
        int width = blurred.getWidth();
        int height = blurred.getHeight();
        
        if (width <= 0 || height <= 0) {
            return;
        }
        
        // Simple box blur implementation
        // This is a basic blur - a full Gaussian blur would require
        // more complex convolution kernel implementation
        
        // For now, we'll apply a simple averaging filter
        // by sampling neighboring pixels
        int kernelSize = static_cast<int>(radius * 2.0f + 1.0f);
        if (kernelSize < 3) kernelSize = 3;
        if (kernelSize % 2 == 0) kernelSize++; // Ensure odd size
        
        int halfKernel = kernelSize / 2;
        
        // Create a temporary copy for reading
        juce::Image temp = ptr->image.createCopy();
        
        // Apply horizontal blur pass
        for (int y = 0; y < height; ++y) {
            for (int x = 0; x < width; ++x) {
                int r = 0, g = 0, b = 0, a = 0;
                int count = 0;
                
                for (int kx = -halfKernel; kx <= halfKernel; ++kx) {
                    int sx = x + kx;
                    if (sx >= 0 && sx < width) {
                        juce::Colour pixel = temp.getPixelAt(sx, y);
                        r += pixel.getRed();
                        g += pixel.getGreen();
                        b += pixel.getBlue();
                        a += pixel.getAlpha();
                        count++;
                    }
                }
                
                if (count > 0) {
                    blurred.setPixelAt(x, y, juce::Colour(
                        static_cast<uint8_t>(r / count),
                        static_cast<uint8_t>(g / count),
                        static_cast<uint8_t>(b / count),
                        static_cast<uint8_t>(a / count)
                    ));
                }
            }
        }
        
        // Apply vertical blur pass
        temp = blurred.createCopy();
        for (int y = 0; y < height; ++y) {
            for (int x = 0; x < width; ++x) {
                int r = 0, g = 0, b = 0, a = 0;
                int count = 0;
                
                for (int ky = -halfKernel; ky <= halfKernel; ++ky) {
                    int sy = y + ky;
                    if (sy >= 0 && sy < height) {
                        juce::Colour pixel = temp.getPixelAt(x, sy);
                        r += pixel.getRed();
                        g += pixel.getGreen();
                        b += pixel.getBlue();
                        a += pixel.getAlpha();
                        count++;
                    }
                }
                
                if (count > 0) {
                    blurred.setPixelAt(x, y, juce::Colour(
                        static_cast<uint8_t>(r / count),
                        static_cast<uint8_t>(g / count),
                        static_cast<uint8_t>(b / count),
                        static_cast<uint8_t>(a / count)
                    ));
                }
            }
        }
        
        // Replace the original image with the blurred version
        ptr->image = blurred;
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Get the width of an image
int32_t image_get_width(const JuceImage* ptr, int8_t* error_buffer, size_t buffer_size) {
    try {
        if (!ptr) {
            throw std::invalid_argument("Image pointer cannot be null");
        }
        
        if (!ptr->image.isValid()) {
            throw std::runtime_error("Image is not valid");
        }
        
        return ptr->image.getWidth();
    } catch (const std::exception& e) {
        if (error_buffer && buffer_size > 0) {
            std::snprintf(reinterpret_cast<char*>(error_buffer), buffer_size, "%s", e.what());
        }
        return -1;
    } catch (...) {
        if (error_buffer && buffer_size > 0) {
            std::snprintf(reinterpret_cast<char*>(error_buffer), buffer_size, "Unknown C++ exception");
        }
        return -1;
    }
}

// Get the height of an image
int32_t image_get_height(const JuceImage* ptr, int8_t* error_buffer, size_t buffer_size) {
    try {
        if (!ptr) {
            throw std::invalid_argument("Image pointer cannot be null");
        }
        
        if (!ptr->image.isValid()) {
            throw std::runtime_error("Image is not valid");
        }
        
        return ptr->image.getHeight();
    } catch (const std::exception& e) {
        if (error_buffer && buffer_size > 0) {
            std::snprintf(reinterpret_cast<char*>(error_buffer), buffer_size, "%s", e.what());
        }
        return -1;
    } catch (...) {
        if (error_buffer && buffer_size > 0) {
            std::snprintf(reinterpret_cast<char*>(error_buffer), buffer_size, "Unknown C++ exception");
        }
        return -1;
    }
}

} // namespace nih_plug_juce
