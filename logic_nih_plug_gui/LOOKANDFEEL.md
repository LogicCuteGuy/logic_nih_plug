# LookAndFeel Customization Guide

The LookAndFeel system provides a flexible way to customize the appearance of UI components in nih_plug_gui. It's inspired by JUCE's LookAndFeel system and allows you to create consistent visual themes across your plugin interfaces.

## Overview

The LookAndFeel system consists of:

- **`LookAndFeel` trait**: Defines methods for customizing component appearance
- **`DefaultLookAndFeel`**: A standard implementation with built-in themes
- **`Theme` enum**: Predefined color themes (Light, Dark, HighContrast)
- **`ColorScheme`**: A collection of colors used by a theme

## Quick Start

### Using Built-in Themes

```rust
use nih_plug_gui::lookandfeel::{DefaultLookAndFeel, Theme};
use nih_plug_gui::controls::Button;
use nih_plug_gui::components::Bounds;

// Create a button
let mut button = Button::new("Click Me");
button.set_bounds(Bounds::new(10, 10, 100, 30)).unwrap();

// Create a LookAndFeel with dark theme
let laf = DefaultLookAndFeel::with_theme(Theme::Dark);

// Use the LookAndFeel to get colors
let button_color = laf.button_color(button.button_state());
```

### Available Themes

1. **Light**: Bright backgrounds, dark text (default)
2. **Dark**: Dark backgrounds, light text
3. **HighContrast**: Maximum contrast for accessibility

### Dynamic Theme Switching

```rust
let mut laf = DefaultLookAndFeel::new(); // Starts with Light theme

// Switch to dark theme
laf.set_theme(Theme::Dark);

// Switch to high contrast
laf.set_theme(Theme::HighContrast);
```

## Creating Custom LookAndFeel

You can create your own LookAndFeel by implementing the `LookAndFeel` trait:

```rust
use nih_plug_gui::lookandfeel::{LookAndFeel, ColorScheme};
use nih_plug_gui::controls::ButtonState;
use nih_plug_graphics::Color;

struct MyCustomLookAndFeel {
    colors: ColorScheme,
}

impl MyCustomLookAndFeel {
    fn new() -> Self {
        Self {
            colors: ColorScheme {
                background: Color::rgb(50, 50, 80),
                background_secondary: Color::rgb(70, 70, 100),
                foreground: Color::rgb(255, 255, 200),
                accent: Color::rgb(255, 100, 100),
                disabled: Color::rgb(100, 100, 120),
                border: Color::rgb(150, 150, 180),
                hover: Color::rgb(90, 90, 120),
                pressed: Color::rgb(60, 60, 90),
            },
        }
    }
}

impl LookAndFeel for MyCustomLookAndFeel {
    fn color_scheme(&self) -> &ColorScheme {
        &self.colors
    }

    // Override specific methods for custom behavior
    fn corner_radius(&self) -> u32 {
        8 // More rounded corners
    }

    fn border_width(&self) -> u32 {
        2 // Thicker borders
    }
}
```

## LookAndFeel Methods

The `LookAndFeel` trait provides methods for customizing various aspects of components:

### Color Methods

- `button_color(state: ButtonState) -> Color`: Button background color
- `button_text_color(state: ButtonState) -> Color`: Button text color
- `button_border_color() -> Color`: Button border color
- `slider_track_color(enabled: bool) -> Color`: Slider track color
- `slider_thumb_color(enabled: bool) -> Color`: Slider thumb color
- `label_text_color(enabled: bool) -> Color`: Label text color
- `background_color() -> Color`: Main background color
- `border_color() -> Color`: General border color
- `accent_color() -> Color`: Accent/highlight color

### Metric Methods

- `corner_radius() -> u32`: Corner radius in pixels (default: 4)
- `border_width() -> u32`: Border width in pixels (default: 1)
- `default_font_size() -> f32`: Default font size in points (default: 14.0)
- `component_padding() -> u32`: Internal padding in pixels (default: 5)

## Using LookAndFeel with Components

Components provide methods to render with a LookAndFeel:

### Button

```rust
use nih_plug_gui::controls::Button;
use nih_plug_gui::lookandfeel::DefaultLookAndFeel;
use nih_plug_graphics::Graphics;

let button = Button::new("Click Me");
let laf = DefaultLookAndFeel::new();
let mut graphics = Graphics::new(800, 600);

// Render with LookAndFeel (requires graphics feature)
#[cfg(feature = "graphics")]
button.render_with_lookandfeel(&mut graphics, &laf).unwrap();

// Render with text and LookAndFeel (requires text feature)
#[cfg(feature = "text")]
{
    use nih_plug_graphics::Font;
    let font = Font::from_bytes(include_bytes!("font.ttf")).unwrap();
    button.render_with_lookandfeel_and_text(&mut graphics, &font, &laf).unwrap();
}
```

### Slider

```rust
use nih_plug_gui::controls::{Slider, SliderOrientation};
use nih_plug_gui::lookandfeel::DefaultLookAndFeel;

let slider = Slider::new(SliderOrientation::Horizontal);
let laf = DefaultLookAndFeel::new();

#[cfg(feature = "graphics")]
{
    use nih_plug_graphics::Graphics;
    let mut graphics = Graphics::new(800, 600);
    slider.render_with_lookandfeel(&mut graphics, &laf).unwrap();
}
```

### Label

```rust
use nih_plug_gui::controls::Label;
use nih_plug_gui::lookandfeel::DefaultLookAndFeel;

let label = Label::new("Hello, World!");
let laf = DefaultLookAndFeel::new();

#[cfg(feature = "text")]
{
    use nih_plug_graphics::{Graphics, Font};
    let mut graphics = Graphics::new(800, 600);
    let font = Font::from_bytes(include_bytes!("font.ttf")).unwrap();
    label.render_with_lookandfeel(&mut graphics, &font, &laf).unwrap();
}
```

## Color Schemes

Each theme has a predefined color scheme:

### Light Theme
- Background: `rgb(240, 240, 240)`
- Foreground: `rgb(0, 0, 0)`
- Accent: `rgb(0, 120, 215)`

### Dark Theme
- Background: `rgb(30, 30, 30)`
- Foreground: `rgb(255, 255, 255)`
- Accent: `rgb(0, 120, 215)`

### High Contrast Theme
- Background: `rgb(0, 0, 0)`
- Foreground: `rgb(255, 255, 255)`
- Accent: `rgb(255, 255, 0)`

## Best Practices

1. **Consistency**: Use the same LookAndFeel instance across all components in your UI for a consistent appearance.

2. **Accessibility**: Consider using the HighContrast theme or creating custom high-contrast themes for users with visual impairments.

3. **User Preferences**: Allow users to choose their preferred theme and save it in plugin state.

4. **Custom Themes**: When creating custom themes, ensure sufficient contrast between foreground and background colors.

5. **Performance**: Create LookAndFeel instances once and reuse them rather than creating new instances for each render.

## Examples

See the `examples/lookandfeel_demo.rs` file for a complete demonstration of the LookAndFeel system, including:

- Using built-in themes
- Creating custom LookAndFeel implementations
- Dynamic theme switching
- Component-specific color customization

## Feature Flags

The LookAndFeel system is available when the `components` feature is enabled (which is part of the default features). To use rendering methods:

- Enable `graphics` feature for basic rendering
- Enable `text` feature for text rendering (also enables `graphics`)

```toml
[dependencies]
nih_plug_gui = { version = "0.0.0", features = ["components", "graphics", "text"] }
```
