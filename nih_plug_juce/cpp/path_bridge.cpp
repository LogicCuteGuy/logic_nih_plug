#include "juce_bridge.h"
#include <cstring>
#include <new>

namespace nih_plug_juce {

// Path operations

JucePath* create_path(int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions_ptr([&]() -> JucePath* {
        auto* path = new juce::Path();
        return reinterpret_cast<JucePath*>(path);
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

void delete_path(JucePath* ptr) {
    if (ptr) {
        delete reinterpret_cast<juce::Path*>(ptr);
    }
}

int32_t path_start_new_sub_path(JucePath* ptr, float x, float y,
                                 int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions([&]() {
        if (!ptr) {
            throw std::invalid_argument("Null path pointer");
        }
        auto* path = reinterpret_cast<juce::Path*>(ptr);
        path->startNewSubPath(x, y);
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

int32_t path_line_to(JucePath* ptr, float x, float y,
                     int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions([&]() {
        if (!ptr) {
            throw std::invalid_argument("Null path pointer");
        }
        auto* path = reinterpret_cast<juce::Path*>(ptr);
        path->lineTo(x, y);
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

int32_t path_quadratic_to(JucePath* ptr, float cx, float cy, float x, float y,
                          int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions([&]() {
        if (!ptr) {
            throw std::invalid_argument("Null path pointer");
        }
        auto* path = reinterpret_cast<juce::Path*>(ptr);
        path->quadraticTo(cx, cy, x, y);
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

int32_t path_cubic_to(JucePath* ptr, float cx1, float cy1, float cx2, float cy2,
                      float x, float y, int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions([&]() {
        if (!ptr) {
            throw std::invalid_argument("Null path pointer");
        }
        auto* path = reinterpret_cast<juce::Path*>(ptr);
        path->cubicTo(cx1, cy1, cx2, cy2, x, y);
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

int32_t path_add_rectangle(JucePath* ptr, float x, float y, float width, float height,
                           int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions([&]() {
        if (!ptr) {
            throw std::invalid_argument("Null path pointer");
        }
        auto* path = reinterpret_cast<juce::Path*>(ptr);
        path->addRectangle(x, y, width, height);
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

int32_t path_add_ellipse(JucePath* ptr, float x, float y, float width, float height,
                         int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions([&]() {
        if (!ptr) {
            throw std::invalid_argument("Null path pointer");
        }
        auto* path = reinterpret_cast<juce::Path*>(ptr);
        path->addEllipse(x, y, width, height);
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

int32_t path_add_arc(JucePath* ptr, float x, float y, float width, float height,
                     float start_angle, float end_angle,
                     int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions([&]() {
        if (!ptr) {
            throw std::invalid_argument("Null path pointer");
        }
        auto* path = reinterpret_cast<juce::Path*>(ptr);
        path->addArc(x, y, width, height, start_angle, end_angle);
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

int32_t path_close_sub_path(JucePath* ptr, int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions([&]() {
        if (!ptr) {
            throw std::invalid_argument("Null path pointer");
        }
        auto* path = reinterpret_cast<juce::Path*>(ptr);
        path->closeSubPath();
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

int32_t path_apply_transform(JucePath* ptr, const JuceAffineTransform* transform,
                             int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions([&]() {
        if (!ptr) {
            throw std::invalid_argument("Null path pointer");
        }
        if (!transform) {
            throw std::invalid_argument("Null transform pointer");
        }
        auto* path = reinterpret_cast<juce::Path*>(ptr);
        auto* t = reinterpret_cast<const juce::AffineTransform*>(transform);
        path->applyTransform(*t);
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

// AffineTransform operations

JuceAffineTransform* create_affine_transform_identity(int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions_ptr([&]() -> JuceAffineTransform* {
        auto* transform = new juce::AffineTransform();
        return reinterpret_cast<JuceAffineTransform*>(transform);
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

JuceAffineTransform* create_affine_transform_translation(float dx, float dy,
                                                         int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions_ptr([&]() -> JuceAffineTransform* {
        auto* transform = new juce::AffineTransform(juce::AffineTransform::translation(dx, dy));
        return reinterpret_cast<JuceAffineTransform*>(transform);
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

JuceAffineTransform* create_affine_transform_rotation(float angle_radians,
                                                      int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions_ptr([&]() -> JuceAffineTransform* {
        auto* transform = new juce::AffineTransform(juce::AffineTransform::rotation(angle_radians));
        return reinterpret_cast<JuceAffineTransform*>(transform);
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

JuceAffineTransform* create_affine_transform_scale(float sx, float sy,
                                                   int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions_ptr([&]() -> JuceAffineTransform* {
        auto* transform = new juce::AffineTransform(juce::AffineTransform::scale(sx, sy));
        return reinterpret_cast<JuceAffineTransform*>(transform);
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

void delete_affine_transform(JuceAffineTransform* ptr) {
    if (ptr) {
        delete reinterpret_cast<juce::AffineTransform*>(ptr);
    }
}

JuceAffineTransform* affine_transform_followed_by(const JuceAffineTransform* ptr,
                                                   const JuceAffineTransform* other,
                                                   int8_t* error_buffer, size_t buffer_size) {
    return catch_exceptions_ptr([&]() -> JuceAffineTransform* {
        if (!ptr) {
            throw std::invalid_argument("Null transform pointer");
        }
        if (!other) {
            throw std::invalid_argument("Null other transform pointer");
        }
        auto* transform = reinterpret_cast<const juce::AffineTransform*>(ptr);
        auto* other_transform = reinterpret_cast<const juce::AffineTransform*>(other);
        auto* result = new juce::AffineTransform(transform->followedBy(*other_transform));
        return reinterpret_cast<JuceAffineTransform*>(result);
    }, reinterpret_cast<char*>(error_buffer), buffer_size);
}

} // namespace nih_plug_juce
