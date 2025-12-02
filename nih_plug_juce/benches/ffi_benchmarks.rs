//! Benchmarks for JUCE FFI operations.
//!
//! This benchmark suite measures the performance of FFI calls to JUCE C++ code
//! to verify that overhead is within acceptable limits (< 5% of native JUCE).
//!
//! Requirements: 33.1, 33.2, 33.3, 33.4, 33.5

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use nih_plug_juce::{
    component::Component,
    drawing::{colour::Colour, font::Font, image::Image, path::Path, transform::AffineTransform},
    events::timer::Timer,
    graphics::Graphics,
    widgets::{button::TextButton, label::Label, slider::{Slider, SliderStyle}},
};

// Benchmark component creation
fn bench_component_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("component_creation");
    
    group.bench_function("component_new", |b| {
        b.iter(|| {
            let _component = Component::new().unwrap();
        });
    });
    
    group.bench_function("button_new", |b| {
        b.iter(|| {
            let _button = TextButton::new(black_box("Test")).unwrap();
        });
    });
    
    group.bench_function("slider_new", |b| {
        b.iter(|| {
            let _slider = Slider::new(black_box(SliderStyle::LinearHorizontal)).unwrap();
        });
    });
    
    group.bench_function("label_new", |b| {
        b.iter(|| {
            let _label = Label::new(black_box("Test")).unwrap();
        });
    });
    
    group.finish();
}

// Benchmark component property setters
fn bench_component_properties(c: &mut Criterion) {
    let mut group = c.benchmark_group("component_properties");
    
    group.bench_function("set_bounds", |b| {
        let mut component = Component::new().unwrap();
        b.iter(|| {
            component.set_bounds(
                black_box(10),
                black_box(20),
                black_box(100),
                black_box(50),
            );
        });
    });
    
    group.bench_function("set_visible", |b| {
        let mut component = Component::new().unwrap();
        let mut visible = true;
        b.iter(|| {
            component.set_visible(black_box(visible));
            visible = !visible;
        });
    });
    
    group.bench_function("repaint", |b| {
        let mut component = Component::new().unwrap();
        b.iter(|| {
            component.repaint();
        });
    });
    
    group.finish();
}

// Benchmark parent-child operations
fn bench_parent_child_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("parent_child_operations");
    
    group.bench_function("add_child", |b| {
        let mut parent = Component::new().unwrap();
        b.iter(|| {
            let child = Component::new().unwrap();
            parent.add_child(&child).unwrap();
        });
    });
    
    group.bench_function("add_remove_child", |b| {
        let mut parent = Component::new().unwrap();
        b.iter(|| {
            let child = Component::new().unwrap();
            parent.add_child(&child).unwrap();
            parent.remove_child(&child).unwrap();
        });
    });
    
    // Benchmark adding multiple children
    for child_count in [5, 10, 20, 50].iter() {
        group.bench_with_input(
            BenchmarkId::new("add_multiple_children", child_count),
            child_count,
            |b, &count| {
                b.iter(|| {
                    let mut parent = Component::new().unwrap();
                    for _ in 0..count {
                        let child = Component::new().unwrap();
                        parent.add_child(&child).unwrap();
                    }
                });
            },
        );
    }
    
    group.finish();
}

// Benchmark widget operations
fn bench_widget_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("widget_operations");
    
    group.bench_function("button_set_text", |b| {
        let mut button = TextButton::new("Initial").unwrap();
        b.iter(|| {
            button.set_button_text(black_box("Updated"));
        });
    });
    
    group.bench_function("button_set_enabled", |b| {
        let mut button = TextButton::new("Test").unwrap();
        let mut enabled = true;
        b.iter(|| {
            button.set_enabled(black_box(enabled));
            enabled = !enabled;
        });
    });
    
    group.bench_function("slider_set_value", |b| {
        let mut slider = Slider::new(SliderStyle::LinearHorizontal).unwrap();
        slider.set_range(0.0, 1.0, 0.01);
        let mut value = 0.0;
        b.iter(|| {
            slider.set_value(black_box(value));
            value = (value + 0.1) % 1.0;
        });
    });
    
    group.bench_function("slider_get_value", |b| {
        let slider = Slider::new(SliderStyle::LinearHorizontal).unwrap();
        b.iter(|| {
            let _value = slider.get_value();
        });
    });
    
    group.bench_function("slider_set_range", |b| {
        let mut slider = Slider::new(SliderStyle::LinearHorizontal).unwrap();
        b.iter(|| {
            slider.set_range(black_box(0.0), black_box(100.0), black_box(1.0));
        });
    });
    
    group.bench_function("label_set_text", |b| {
        let mut label = Label::new("Initial").unwrap();
        b.iter(|| {
            label.set_text(black_box("Updated"));
        });
    });
    
    group.finish();
}

// Benchmark colour operations
fn bench_colour_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("colour_operations");
    
    group.bench_function("colour_from_rgba", |b| {
        b.iter(|| {
            let _colour = Colour::from_rgba(
                black_box(255),
                black_box(128),
                black_box(64),
                black_box(255),
            );
        });
    });
    
    group.bench_function("colour_from_rgb", |b| {
        b.iter(|| {
            let _colour = Colour::from_rgb(
                black_box(255),
                black_box(128),
                black_box(64),
            );
        });
    });
    
    group.bench_function("colour_with_alpha", |b| {
        let colour = Colour::from_rgb(255, 128, 64).unwrap();
        b.iter(|| {
            let _new_colour = colour.with_alpha(black_box(0.5));
        });
    });
    
    group.bench_function("colour_brighter", |b| {
        let colour = Colour::from_rgb(128, 128, 128).unwrap();
        b.iter(|| {
            let _new_colour = colour.brighter(black_box(0.2));
        });
    });
    
    group.bench_function("colour_darker", |b| {
        let colour = Colour::from_rgb(128, 128, 128).unwrap();
        b.iter(|| {
            let _new_colour = colour.darker(black_box(0.2));
        });
    });
    
    group.bench_function("colour_interpolated", |b| {
        let colour1 = Colour::from_rgb(255, 0, 0).unwrap();
        let colour2 = Colour::from_rgb(0, 0, 255).unwrap();
        b.iter(|| {
            let _new_colour = colour1.interpolated_with(&colour2, black_box(0.5));
        });
    });
    
    group.finish();
}

// Benchmark font operations
fn bench_font_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("font_operations");
    
    group.bench_function("font_new", |b| {
        b.iter(|| {
            let _font = Font::new(black_box(14.0)).unwrap();
        });
    });
    
    group.bench_function("font_set_bold", |b| {
        let mut font = Font::new(14.0).unwrap();
        let mut bold = true;
        b.iter(|| {
            font.set_bold(black_box(bold));
            bold = !bold;
        });
    });
    
    group.bench_function("font_set_italic", |b| {
        let mut font = Font::new(14.0).unwrap();
        let mut italic = true;
        b.iter(|| {
            font.set_italic(black_box(italic));
            italic = !italic;
        });
    });
    
    group.bench_function("font_get_string_width", |b| {
        let font = Font::new(14.0).unwrap();
        b.iter(|| {
            let _width = font.get_string_width(black_box("Hello, World!"));
        });
    });
    
    group.bench_function("font_get_height", |b| {
        let font = Font::new(14.0).unwrap();
        b.iter(|| {
            let _height = font.get_height();
        });
    });
    
    group.finish();
}

// Benchmark callback registration (overhead of setting up callbacks)
fn bench_callback_registration(c: &mut Criterion) {
    let mut group = c.benchmark_group("callback_registration");
    
    group.bench_function("button_set_on_click", |b| {
        let mut button = TextButton::new("Test").unwrap();
        b.iter(|| {
            button.set_on_click(|| {
                // Empty callback
            });
        });
    });
    
    group.bench_function("slider_set_on_value_change", |b| {
        let mut slider = Slider::new(SliderStyle::LinearHorizontal).unwrap();
        b.iter(|| {
            slider.set_on_value_change(|_value| {
                // Empty callback
            });
        });
    });
    
    group.finish();
}

// Benchmark round-trip operations (set + get)
fn bench_round_trip_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("round_trip_operations");
    
    group.bench_function("slider_value_round_trip", |b| {
        let mut slider = Slider::new(SliderStyle::LinearHorizontal).unwrap();
        slider.set_range(0.0, 1.0, 0.01);
        let mut value = 0.0;
        b.iter(|| {
            slider.set_value(black_box(value));
            let _retrieved = slider.get_value();
            value = (value + 0.1) % 1.0;
        });
    });
    
    group.bench_function("component_bounds_set_multiple", |b| {
        let mut component = Component::new().unwrap();
        let mut x = 0;
        b.iter(|| {
            component.set_bounds(black_box(x), black_box(0), black_box(100), black_box(50));
            component.set_bounds(black_box(x + 1), black_box(1), black_box(101), black_box(51));
            x = (x + 10) % 100;
        });
    });
    
    group.finish();
}

// Benchmark batch operations to measure cumulative overhead
fn bench_batch_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_operations");
    group.throughput(Throughput::Elements(100));
    
    group.bench_function("100_set_bounds", |b| {
        let mut component = Component::new().unwrap();
        b.iter(|| {
            for i in 0..100 {
                component.set_bounds(
                    black_box(i),
                    black_box(i),
                    black_box(100),
                    black_box(50),
                );
            }
        });
    });
    
    group.bench_function("100_set_visible", |b| {
        let mut component = Component::new().unwrap();
        b.iter(|| {
            for i in 0..100 {
                component.set_visible(black_box(i % 2 == 0));
            }
        });
    });
    
    group.bench_function("100_slider_set_value", |b| {
        let mut slider = Slider::new(SliderStyle::LinearHorizontal).unwrap();
        slider.set_range(0.0, 100.0, 1.0);
        b.iter(|| {
            for i in 0..100 {
                slider.set_value(black_box(i as f64));
            }
        });
    });
    
    group.finish();
}

// Benchmark memory allocation patterns
fn bench_allocation_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocation_patterns");
    
    group.bench_function("create_destroy_component", |b| {
        b.iter(|| {
            let component = Component::new().unwrap();
            drop(component);
        });
    });
    
    group.bench_function("create_destroy_button", |b| {
        b.iter(|| {
            let button = TextButton::new("Test").unwrap();
            drop(button);
        });
    });
    
    group.bench_function("create_destroy_slider", |b| {
        b.iter(|| {
            let slider = Slider::new(SliderStyle::LinearHorizontal).unwrap();
            drop(slider);
        });
    });
    
    // Benchmark creating and destroying many components
    group.bench_function("create_destroy_100_components", |b| {
        b.iter(|| {
            let mut components = Vec::new();
            for _ in 0..100 {
                components.push(Component::new().unwrap());
            }
            drop(components);
        });
    });
    
    group.finish();
}

// Benchmark string operations across FFI boundary
fn bench_string_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_operations");
    
    let short_text = "Hi";
    let medium_text = "Hello, World!";
    let long_text = "This is a longer text string that will be passed across the FFI boundary to test the overhead of string copying and conversion.";
    
    group.bench_function("button_set_text_short", |b| {
        let mut button = TextButton::new("Initial").unwrap();
        b.iter(|| {
            button.set_button_text(black_box(short_text));
        });
    });
    
    group.bench_function("button_set_text_medium", |b| {
        let mut button = TextButton::new("Initial").unwrap();
        b.iter(|| {
            button.set_button_text(black_box(medium_text));
        });
    });
    
    group.bench_function("button_set_text_long", |b| {
        let mut button = TextButton::new("Initial").unwrap();
        b.iter(|| {
            button.set_button_text(black_box(long_text));
        });
    });
    
    group.bench_function("label_set_text_short", |b| {
        let mut label = Label::new("Initial").unwrap();
        b.iter(|| {
            label.set_text(black_box(short_text));
        });
    });
    
    group.bench_function("label_set_text_medium", |b| {
        let mut label = Label::new("Initial").unwrap();
        b.iter(|| {
            label.set_text(black_box(medium_text));
        });
    });
    
    group.bench_function("label_set_text_long", |b| {
        let mut label = Label::new("Initial").unwrap();
        b.iter(|| {
            label.set_text(black_box(long_text));
        });
    });
    
    group.finish();
}

// Benchmark drawing operations
fn bench_drawing_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("drawing_operations");
    
    // Note: These benchmarks measure the overhead of setting up paint callbacks
    // Actual drawing performance would require a full message loop
    
    group.bench_function("set_paint_callback", |b| {
        let mut component = Component::new().unwrap();
        b.iter(|| {
            component.set_paint_callback(|_g: &mut Graphics| {
                // Empty paint callback
            });
        });
    });
    
    group.bench_function("path_creation", |b| {
        b.iter(|| {
            let mut path = Path::new().unwrap();
            path.start_new_sub_path(black_box(0.0), black_box(0.0));
            path.line_to(black_box(100.0), black_box(100.0));
            path.line_to(black_box(100.0), black_box(0.0));
            path.close_sub_path();
        });
    });
    
    group.bench_function("path_add_rectangle", |b| {
        let mut path = Path::new().unwrap();
        b.iter(|| {
            path.add_rectangle(
                black_box(10.0),
                black_box(10.0),
                black_box(100.0),
                black_box(50.0),
            );
        });
    });
    
    group.bench_function("path_add_ellipse", |b| {
        let mut path = Path::new().unwrap();
        b.iter(|| {
            path.add_ellipse(
                black_box(10.0),
                black_box(10.0),
                black_box(100.0),
                black_box(50.0),
            );
        });
    });
    
    group.bench_function("transform_creation", |b| {
        b.iter(|| {
            let _transform = AffineTransform::identity();
        });
    });
    
    group.bench_function("transform_translation", |b| {
        b.iter(|| {
            let _transform = AffineTransform::translation(black_box(10.0), black_box(20.0));
        });
    });
    
    group.bench_function("transform_rotation", |b| {
        b.iter(|| {
            let _transform = AffineTransform::rotation(black_box(0.5));
        });
    });
    
    group.bench_function("transform_scale", |b| {
        b.iter(|| {
            let _transform = AffineTransform::scale(black_box(2.0), black_box(2.0));
        });
    });
    
    group.bench_function("transform_composition", |b| {
        let t1 = AffineTransform::translation(10.0, 20.0).unwrap();
        let t2 = AffineTransform::rotation(0.5).unwrap();
        b.iter(|| {
            let _composed = t1.followed_by(&t2).unwrap();
        });
    });
    
    group.finish();
}

// Benchmark event handling setup
fn bench_event_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_handling");
    
    group.bench_function("timer_creation", |b| {
        b.iter(|| {
            let _timer = Timer::new(|| {
                // Empty callback
            });
        });
    });
    
    group.bench_function("timer_start_stop", |b| {
        let mut timer = Timer::new(|| {}).unwrap();
        b.iter(|| {
            timer.start(black_box(100)).unwrap();
            timer.stop();
        });
    });
    
    group.bench_function("timer_is_running", |b| {
        let timer = Timer::new(|| {}).unwrap();
        b.iter(|| {
            let _running = timer.is_running();
        });
    });
    
    // Benchmark mouse listener setup
    group.bench_function("set_mouse_listener", |b| {
        let mut component = Component::new().unwrap();
        b.iter(|| {
            component.set_mouse_listener(Box::new(TestMouseListener));
        });
    });
    
    // Benchmark keyboard listener setup
    group.bench_function("set_key_listener", |b| {
        let mut component = Component::new().unwrap();
        b.iter(|| {
            component.set_key_listener(Box::new(TestKeyListener));
        });
    });
    
    group.bench_function("set_wants_keyboard_focus", |b| {
        let mut component = Component::new().unwrap();
        let mut wants = true;
        b.iter(|| {
            component.set_wants_keyboard_focus(black_box(wants));
            wants = !wants;
        });
    });
    
    group.finish();
}

// Test mouse listener for benchmarking
struct TestMouseListener;

impl nih_plug_juce::events::mouse::MouseListener for TestMouseListener {
    fn mouse_down(&mut self, _event: &nih_plug_juce::events::mouse::MouseEvent) {}
    fn mouse_drag(&mut self, _event: &nih_plug_juce::events::mouse::MouseEvent) {}
    fn mouse_up(&mut self, _event: &nih_plug_juce::events::mouse::MouseEvent) {}
    fn mouse_enter(&mut self, _event: &nih_plug_juce::events::mouse::MouseEvent) {}
    fn mouse_exit(&mut self, _event: &nih_plug_juce::events::mouse::MouseEvent) {}
}

// Test key listener for benchmarking
struct TestKeyListener;

impl nih_plug_juce::events::keyboard::KeyListener for TestKeyListener {
    fn key_pressed(&mut self, _key: &nih_plug_juce::events::keyboard::KeyPress) -> bool {
        false
    }
}

// Benchmark callback invocation latency
fn bench_callback_invocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("callback_invocation");
    
    // Measure the overhead of callback invocation through FFI
    // Note: These measure registration overhead, not actual invocation
    // since invocation happens asynchronously on the message thread
    
    group.bench_function("button_callback_with_work", |b| {
        let mut button = TextButton::new("Test").unwrap();
        b.iter(|| {
            button.set_on_click(|| {
                // Simulate some work in the callback
                black_box(42 + 42);
            });
        });
    });
    
    group.bench_function("slider_callback_with_work", |b| {
        let mut slider = Slider::new(SliderStyle::LinearHorizontal).unwrap();
        b.iter(|| {
            slider.set_on_value_change(|value| {
                // Simulate some work in the callback
                black_box(value * 2.0);
            });
        });
    });
    
    group.bench_function("timer_callback_with_work", |b| {
        b.iter(|| {
            let _timer = Timer::new(|| {
                // Simulate some work in the callback
                black_box(42 + 42);
            }).unwrap();
        });
    });
    
    // Benchmark callback with captured state
    group.bench_function("button_callback_with_capture", |b| {
        let mut button = TextButton::new("Test").unwrap();
        let state = vec![1, 2, 3, 4, 5];
        b.iter(|| {
            let state_clone = state.clone();
            button.set_on_click(move || {
                black_box(&state_clone);
            });
        });
    });
    
    group.finish();
}

// Benchmark image operations
fn bench_image_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("image_operations");
    
    group.bench_function("image_creation_rgb", |b| {
        b.iter(|| {
            let _image = Image::new(
                black_box(nih_plug_juce::drawing::image::ImageFormat::RGB),
                black_box(100),
                black_box(100),
            );
        });
    });
    
    group.bench_function("image_creation_argb", |b| {
        b.iter(|| {
            let _image = Image::new(
                black_box(nih_plug_juce::drawing::image::ImageFormat::ARGB),
                black_box(100),
                black_box(100),
            );
        });
    });
    
    // Benchmark different image sizes
    for size in [50, 100, 200, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("image_creation_size", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let _image = Image::new(
                        nih_plug_juce::drawing::image::ImageFormat::ARGB,
                        black_box(size),
                        black_box(size),
                    );
                });
            },
        );
    }
    
    group.finish();
}

// Benchmark complex widget hierarchies
fn bench_widget_hierarchies(c: &mut Criterion) {
    let mut group = c.benchmark_group("widget_hierarchies");
    
    group.bench_function("create_simple_hierarchy", |b| {
        b.iter(|| {
            let mut parent = Component::new().unwrap();
            let button = TextButton::new("Button").unwrap();
            let slider = Slider::new(SliderStyle::LinearHorizontal).unwrap();
            let label = Label::new("Label").unwrap();
            
            parent.add_child(&button).unwrap();
            parent.add_child(&slider).unwrap();
            parent.add_child(&label).unwrap();
        });
    });
    
    group.bench_function("create_nested_hierarchy", |b| {
        b.iter(|| {
            let mut root = Component::new().unwrap();
            let mut container1 = Component::new().unwrap();
            let mut container2 = Component::new().unwrap();
            
            let button1 = TextButton::new("Button1").unwrap();
            let button2 = TextButton::new("Button2").unwrap();
            
            container1.add_child(&button1).unwrap();
            container2.add_child(&button2).unwrap();
            
            root.add_child(&container1).unwrap();
            root.add_child(&container2).unwrap();
        });
    });
    
    group.bench_function("create_complex_ui", |b| {
        b.iter(|| {
            let mut root = Component::new().unwrap();
            
            // Create a more realistic UI structure
            for i in 0..5 {
                let mut row = Component::new().unwrap();
                let label = Label::new(&format!("Param {}", i)).unwrap();
                let slider = Slider::new(SliderStyle::LinearHorizontal).unwrap();
                
                row.add_child(&label).unwrap();
                row.add_child(&slider).unwrap();
                root.add_child(&row).unwrap();
            }
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_component_creation,
    bench_component_properties,
    bench_parent_child_operations,
    bench_widget_operations,
    bench_colour_operations,
    bench_font_operations,
    bench_callback_registration,
    bench_round_trip_operations,
    bench_batch_operations,
    bench_allocation_patterns,
    bench_string_operations,
    bench_drawing_operations,
    bench_event_handling,
    bench_callback_invocation,
    bench_image_operations,
    bench_widget_hierarchies,
);

criterion_main!(benches);
