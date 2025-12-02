#include "juce_bridge.h"
#include <memory>

namespace nih_plug_juce {

// Create a Drawable from SVG data
JuceDrawable* create_drawable_from_svg(
    const uint8_t* svg_data,
    size_t svg_len,
    int8_t* error_buffer,
    size_t buffer_size)
{
    return catch_exceptions_ptr([&]() -> JuceDrawable* {
        if (!svg_data || svg_len == 0) {
            throw std::invalid_argument("SVG data is null or empty");
        }
        
        juce::String svgString(reinterpret_cast<const char*>(svg_data), svg_len);
        auto xml = juce::parseXML(svgString);
        
        if (xml == nullptr) {
            throw std::runtime_error("Failed to parse SVG XML");
        }
        
        auto drawable = juce::Drawable::createFromSVG(*xml);
        
        if (drawable == nullptr) {
            throw std::runtime_error("Failed to create Drawable from SVG");
        }
        
        return new JuceDrawable(drawable.release());
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Create a Drawable from image data
JuceDrawable* create_drawable_from_image_data(
    const uint8_t* image_data,
    size_t data_len,
    int8_t* error_buffer,
    size_t buffer_size)
{
    return catch_exceptions_ptr([&]() -> JuceDrawable* {
        if (!image_data || data_len == 0) {
            throw std::invalid_argument("Image data is null or empty");
        }
        
        juce::MemoryInputStream stream(image_data, data_len, false);
        auto drawable = juce::Drawable::createFromImageDataStream(stream);
        
        if (drawable == nullptr) {
            throw std::runtime_error("Failed to create Drawable from image data");
        }
        
        return new JuceDrawable(drawable.release());
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Delete a Drawable
void delete_drawable(JuceDrawable* ptr)
{
    if (ptr) {
        if (ptr->ptr) {
            delete ptr->ptr;
        }
        delete ptr;
    }
}

// Draw a drawable to a Graphics context
int32_t drawable_draw(
    const JuceDrawable* ptr,
    JuceGraphics* g,
    float opacity,
    int8_t* error_buffer,
    size_t buffer_size)
{
    return catch_exceptions([&]() {
        if (!ptr || !ptr->ptr) {
            throw std::invalid_argument("Drawable pointer is null");
        }
        
        if (!g || !g->ptr) {
            throw std::invalid_argument("Graphics pointer is null");
        }
        
        ptr->ptr->draw(*g->ptr, opacity);
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Set the drawable's transform to fit within bounds
int32_t drawable_set_transform_to_fit(
    JuceDrawable* ptr,
    float x,
    float y,
    float width,
    float height,
    int8_t* error_buffer,
    size_t buffer_size)
{
    return catch_exceptions([&]() {
        if (!ptr || !ptr->ptr) {
            throw std::invalid_argument("Drawable pointer is null");
        }
        
        juce::Rectangle<float> bounds(x, y, width, height);
        ptr->ptr->setTransformToFit(bounds, juce::RectanglePlacement::centred);
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Create a DrawableButton
JuceComponent* create_drawable_button(
    const uint8_t* name,
    size_t name_len,
    int8_t* error_buffer,
    size_t buffer_size)
{
    return catch_exceptions_ptr([&]() -> JuceComponent* {
        juce::String buttonName(reinterpret_cast<const char*>(name), name_len);
        
        auto* button = new juce::DrawableButton(
            buttonName,
            juce::DrawableButton::ImageFitted
        );
        
        return new JuceComponent(button);
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// Set images for a DrawableButton
int32_t drawable_button_set_images(
    JuceComponent* ptr,
    const JuceDrawable* normal,
    const JuceDrawable* over,
    const JuceDrawable* down,
    int8_t* error_buffer,
    size_t buffer_size)
{
    return catch_exceptions([&]() {
        if (!ptr || !ptr->ptr) {
            throw std::invalid_argument("Button pointer is null");
        }
        
        if (!normal || !normal->ptr) {
            throw std::invalid_argument("Normal drawable is required");
        }
        
        auto* button = dynamic_cast<juce::DrawableButton*>(ptr->ptr);
        if (!button) {
            throw std::invalid_argument("Component is not a DrawableButton");
        }
        
        auto* normalDrawable = normal->ptr;
        auto* overDrawable = (over && over->ptr) ? over->ptr : nullptr;
        auto* downDrawable = (down && down->ptr) ? down->ptr : nullptr;
        
        // Clone the drawables since DrawableButton takes ownership
        std::unique_ptr<juce::Drawable> normalClone(normalDrawable->createCopy());
        std::unique_ptr<juce::Drawable> overClone(overDrawable ? overDrawable->createCopy() : nullptr);
        std::unique_ptr<juce::Drawable> downClone(downDrawable ? downDrawable->createCopy() : nullptr);
        
        button->setImages(
            normalClone.get(),
            overClone.get(),
            downClone.get()
        );
        
        // Release ownership to the button
        normalClone.release();
        if (overClone) overClone.release();
        if (downClone) downClone.release();
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

} // namespace nih_plug_juce
