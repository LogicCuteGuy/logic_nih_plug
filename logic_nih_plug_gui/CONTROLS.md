# UI Controls Implementation

This document describes the UI controls implemented for the logic_nih_plug_gui crate.

## Overview

Three basic UI controls have been implemented as type-safe wrappers:
- **Button**: A clickable button component
- **Slider**: A value selection component with horizontal/vertical orientation
- **Label**: A text display component

All controls integrate with the existing Component system and provide:
- Type-safe APIs
- Callback support for user interactions
- Bounds management
- Visibility and enabled state
- Optional rendering support (with `graphics` and `text` features)

## Button

A clickable button component that displays text and responds to clicks.

### Features
- Text display
- Multiple states: Normal, Hover, Pressed, Disabled
- Click callback support
- Enable/disable functionality
- Optional rendering with graphics

### Example
```rust
use logic_nih_plug_gui::controls::Button;
use logic_nih_plug_gui::components::Bounds;

let mut button = Button::new("Click Me");
button.set_bounds(Bounds::new(10, 10, 100, 30)).unwrap();
button.set_on_click(|| {
    println!("Button clicked!");
});
button.click();
```

## Slider

A value selection component that allows users to select a numeric value within a range.

### Features
- Horizontal and vertical orientation
- Configurable value range
- Value clamping
- Normalized value access (0.0 to 1.0)
- Value change callback support
- Optional rendering with graphics

### Example
```rust
use logic_nih_plug_gui::controls::{Slider, SliderOrientation};
use logic_nih_plug_gui::components::Bounds;

let mut slider = Slider::new(SliderOrientation::Horizontal);
slider.set_bounds(Bounds::new(10, 10, 200, 30)).unwrap();
slider.set_range(0.0, 100.0).unwrap();
slider.set_value(50.0);

slider.set_on_value_change(|value| {
    println!("Slider value: {}", value);
});
```

## Label

A text display component with configurable alignment and font size.

### Features
- Text display
- Alignment: Left, Center, Right
- Configurable font size
- Optional rendering with text feature

### Example
```rust
use logic_nih_plug_gui::controls::{Label, TextAlignment};
use logic_nih_plug_gui::components::Bounds;

let mut label = Label::new("Hello, World!");
label.set_bounds(Bounds::new(10, 10, 200, 30)).unwrap();
label.set_alignment(TextAlignment::Center);
label.set_font_size(16);
```

## Integration with Component System

All controls provide access to their underlying Component through:
- `component()` - Get immutable reference
- `component_mut()` - Get mutable reference

This allows controls to be added to component hierarchies:

```rust
let mut parent = Component::new("parent");
let mut button = Button::new("Click");
button.set_bounds(Bounds::new(10, 10, 100, 30)).unwrap();

parent.add_child(button.component().clone()).unwrap();
```

## Rendering

Controls support optional rendering through feature flags:
- `graphics` feature: Basic shape rendering
- `text` feature: Text rendering (requires Font object)

Example with rendering:
```rust
#[cfg(feature = "text")]
{
    use logic_nih_plug_graphics::{Graphics, Font, FontSettings};
    
    let mut graphics = Graphics::new(800, 600).unwrap();
    let font_data = include_bytes!("path/to/font.ttf");
    let font = Font::from_bytes(font_data, FontSettings::default()).unwrap();
    
    button.render_with_text(&mut graphics, &font).unwrap();
    label.render(&mut graphics, &font).unwrap();
}
```

## Testing

All controls have comprehensive unit and integration tests covering:
- Creation and initialization
- Property getters and setters
- Value validation and clamping
- Callback functionality
- Integration with Component system
- Bounds management
- State transitions

Run tests with:
```bash
cargo test -p logic_nih_plug_gui --all-features
```

## Requirements Satisfied

This implementation satisfies **Requirement 14.2**:
> WHEN a developer adds buttons, sliders, and labels THEN the system SHALL provide type-safe wrappers for each control type

All three control types are implemented as type-safe Rust structs with:
- Strong typing for all properties and methods
- Result types for fallible operations
- No unsafe code in public APIs
- Comprehensive documentation
- Full test coverage
