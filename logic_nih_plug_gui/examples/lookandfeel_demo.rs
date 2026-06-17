//! Demonstration of the LookAndFeel customization system.
//!
//! This example shows how to use themes and custom LookAndFeel implementations
//! to customize the appearance of UI components.

use logic_nih_plug_gui::components::Bounds;
use logic_nih_plug_gui::controls::{Button, ButtonState, Label, Slider, SliderOrientation, TextAlignment};
use logic_nih_plug_gui::lookandfeel::{ColorScheme, DefaultLookAndFeel, LookAndFeel, Theme};
use logic_nih_plug_graphics::Color;

// Custom LookAndFeel implementation with custom colors
struct CustomLookAndFeel {
    colors: ColorScheme,
}

impl CustomLookAndFeel {
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

impl LookAndFeel for CustomLookAndFeel {
    fn color_scheme(&self) -> &ColorScheme {
        &self.colors
    }

    // Override corner radius for more rounded corners
    fn corner_radius(&self) -> u32 {
        8
    }

    // Override border width for thicker borders
    fn border_width(&self) -> u32 {
        2
    }
}

fn main() {
    println!("LookAndFeel Demonstration");
    println!("=========================\n");

    // Create some UI components
    let mut button = Button::new("Click Me");
    button.set_bounds(Bounds::new(10, 10, 100, 30)).unwrap();

    let mut slider = Slider::new(SliderOrientation::Horizontal);
    slider.set_bounds(Bounds::new(10, 50, 200, 30)).unwrap();
    slider.set_range(0.0, 100.0).unwrap();
    slider.set_value(50.0);

    let mut label = Label::new("Hello, World!");
    label.set_bounds(Bounds::new(10, 90, 200, 30)).unwrap();
    label.set_alignment(TextAlignment::Center);

    // Demonstrate different themes
    println!("1. Light Theme");
    println!("--------------");
    let light_laf = DefaultLookAndFeel::with_theme(Theme::Light);
    demonstrate_theme(&light_laf, &button);

    println!("\n2. Dark Theme");
    println!("-------------");
    let dark_laf = DefaultLookAndFeel::with_theme(Theme::Dark);
    demonstrate_theme(&dark_laf, &button);

    println!("\n3. High Contrast Theme");
    println!("----------------------");
    let hc_laf = DefaultLookAndFeel::with_theme(Theme::HighContrast);
    demonstrate_theme(&hc_laf, &button);

    println!("\n4. Custom LookAndFeel");
    println!("---------------------");
    let custom_laf = CustomLookAndFeel::new();
    demonstrate_theme(&custom_laf, &button);

    // Demonstrate dynamic theme switching
    println!("\n5. Dynamic Theme Switching");
    println!("--------------------------");
    let mut dynamic_laf = DefaultLookAndFeel::new();
    println!("Initial theme: {:?}", dynamic_laf.theme());
    
    dynamic_laf.set_theme(Theme::Dark);
    println!("After switching to Dark: {:?}", dynamic_laf.theme());
    
    dynamic_laf.set_theme(Theme::HighContrast);
    println!("After switching to HighContrast: {:?}", dynamic_laf.theme());

    // Demonstrate component-specific colors
    println!("\n6. Component-Specific Colors");
    println!("-----------------------------");
    let laf = DefaultLookAndFeel::with_theme(Theme::Dark);
    
    println!("Button colors:");
    println!("  Normal: {:?}", laf.button_color(ButtonState::Normal));
    println!("  Hover: {:?}", laf.button_color(ButtonState::Hover));
    println!("  Pressed: {:?}", laf.button_color(ButtonState::Pressed));
    println!("  Disabled: {:?}", laf.button_color(ButtonState::Disabled));
    
    println!("\nSlider colors:");
    println!("  Track (enabled): {:?}", laf.slider_track_color(true));
    println!("  Track (disabled): {:?}", laf.slider_track_color(false));
    println!("  Thumb (enabled): {:?}", laf.slider_thumb_color(true));
    println!("  Thumb (disabled): {:?}", laf.slider_thumb_color(false));
    
    println!("\nLabel colors:");
    println!("  Text (enabled): {:?}", laf.label_text_color(true));
    println!("  Text (disabled): {:?}", laf.label_text_color(false));

    // Demonstrate metrics
    println!("\n7. Component Metrics");
    println!("--------------------");
    println!("Corner radius: {} px", laf.corner_radius());
    println!("Border width: {} px", laf.border_width());
    println!("Default font size: {} pt", laf.default_font_size());
    println!("Component padding: {} px", laf.component_padding());

    println!("\n8. Custom Metrics");
    println!("-----------------");
    let custom = CustomLookAndFeel::new();
    println!("Custom corner radius: {} px", custom.corner_radius());
    println!("Custom border width: {} px", custom.border_width());
}

fn demonstrate_theme(laf: &dyn LookAndFeel, button: &Button) {
    let colors = laf.color_scheme();
    println!("Background: {:?}", colors.background);
    println!("Foreground: {:?}", colors.foreground);
    println!("Accent: {:?}", colors.accent);
    println!("Button (Normal): {:?}", laf.button_color(button.button_state()));
}
