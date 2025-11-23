//! Layout management demonstration.
//!
//! This example shows how to use the layout management system to automatically
//! position and size components.

use nih_plug_gui::components::{Bounds, Component};
use nih_plug_gui::controls::{Button, Label, Slider, SliderOrientation};
use nih_plug_gui::layout::{
    AbsoluteLayout, FlexAlign, FlexDirection, FlexLayout, GridLayout, SizeConstraint,
};

fn main() {
    println!("Layout Management Demo\n");

    // Demo 1: Flex Layout (Horizontal)
    demo_flex_horizontal();

    // Demo 2: Flex Layout (Vertical)
    demo_flex_vertical();

    // Demo 3: Grid Layout
    demo_grid_layout();

    // Demo 4: Absolute Layout with Constraints
    demo_absolute_layout();

    // Demo 5: Nested Layouts
    demo_nested_layouts();
}

fn demo_flex_horizontal() {
    println!("=== Flex Layout (Horizontal) ===");

    let mut layout = FlexLayout::new(FlexDirection::Horizontal);
    layout.set_spacing(10);
    layout.set_padding(5, 5, 5, 5);
    layout.set_align(FlexAlign::Center);

    let mut parent = Component::new("toolbar");
    parent.set_bounds(Bounds::new(0, 0, 400, 50)).unwrap();

    // Add buttons
    for i in 1..=4 {
        let mut button = Button::new(&format!("Button {}", i));
        button.set_bounds(Bounds::new(0, 0, 80, 30)).unwrap();
        parent.add_child(button.component().clone()).unwrap();
    }

    layout.apply(&mut parent).unwrap();

    println!("Parent bounds: {:?}", parent.bounds());
    for i in 0..parent.child_count() {
        let child = parent.child(i).unwrap();
        println!("  Child {} bounds: {:?}", i, child.bounds());
    }
    println!();
}

fn demo_flex_vertical() {
    println!("=== Flex Layout (Vertical) ===");

    let mut layout = FlexLayout::new(FlexDirection::Vertical);
    layout.set_spacing(10);
    layout.set_align(FlexAlign::Stretch);

    let mut parent = Component::new("sidebar");
    parent.set_bounds(Bounds::new(0, 0, 200, 400)).unwrap();

    // Add various controls
    let mut label = Label::new("Settings");
    label.set_bounds(Bounds::new(0, 0, 100, 30)).unwrap();
    parent.add_child(label.component().clone()).unwrap();

    let mut slider1 = Slider::new(SliderOrientation::Horizontal);
    slider1.set_bounds(Bounds::new(0, 0, 150, 30)).unwrap();
    parent.add_child(slider1.component().clone()).unwrap();

    let mut slider2 = Slider::new(SliderOrientation::Horizontal);
    slider2.set_bounds(Bounds::new(0, 0, 150, 30)).unwrap();
    parent.add_child(slider2.component().clone()).unwrap();

    let mut button = Button::new("Apply");
    button.set_bounds(Bounds::new(0, 0, 100, 30)).unwrap();
    parent.add_child(button.component().clone()).unwrap();

    layout.apply(&mut parent).unwrap();

    println!("Parent bounds: {:?}", parent.bounds());
    for i in 0..parent.child_count() {
        let child = parent.child(i).unwrap();
        println!("  Child {} bounds: {:?}", i, child.bounds());
    }
    println!();
}

fn demo_grid_layout() {
    println!("=== Grid Layout ===");

    let mut layout = GridLayout::new(3, 3).unwrap();
    layout.set_spacing(5);
    layout.set_padding(10, 10, 10, 10);

    let mut parent = Component::new("button_grid");
    parent.set_bounds(Bounds::new(0, 0, 300, 300)).unwrap();

    // Add 9 buttons in a 3x3 grid
    for i in 1..=9 {
        let mut button = Button::new(&format!("{}", i));
        button.set_bounds(Bounds::new(0, 0, 50, 50)).unwrap();
        parent.add_child(button.component().clone()).unwrap();
    }

    layout.apply(&mut parent).unwrap();

    println!("Parent bounds: {:?}", parent.bounds());
    for i in 0..parent.child_count() {
        let child = parent.child(i).unwrap();
        let row = i / 3;
        let col = i % 3;
        println!("  Button {} (row {}, col {}): {:?}", i + 1, row, col, child.bounds());
    }
    println!();
}

fn demo_absolute_layout() {
    println!("=== Absolute Layout with Constraints ===");

    let mut layout = AbsoluteLayout::new();

    // Add constraints for specific children
    layout.add_constraint(0, SizeConstraint::new().with_fixed_size(100, 50));
    layout.add_constraint(
        1,
        SizeConstraint::new()
            .with_min_width(150)
            .with_max_width(250)
            .with_preferred_height(30),
    );
    layout.add_constraint(
        2,
        SizeConstraint::new()
            .with_min_width(80)
            .with_min_height(80),
    );

    let mut parent = Component::new("canvas");
    parent.set_bounds(Bounds::new(0, 0, 500, 400)).unwrap();

    // Add components with initial positions
    let mut button = Button::new("Fixed Size");
    button.set_bounds(Bounds::new(10, 10, 200, 100)).unwrap();
    parent.add_child(button.component().clone()).unwrap();

    let mut slider = Slider::new(SliderOrientation::Horizontal);
    slider.set_bounds(Bounds::new(150, 50, 100, 20)).unwrap();
    parent.add_child(slider.component().clone()).unwrap();

    let mut label = Label::new("Min Size");
    label.set_bounds(Bounds::new(300, 200, 50, 50)).unwrap();
    parent.add_child(label.component().clone()).unwrap();

    layout.apply(&mut parent).unwrap();

    println!("Parent bounds: {:?}", parent.bounds());
    for i in 0..parent.child_count() {
        let child = parent.child(i).unwrap();
        println!("  Child {} bounds: {:?}", i, child.bounds());
    }
    println!();
}

fn demo_nested_layouts() {
    println!("=== Nested Layouts ===");

    // Create main vertical layout
    let mut main_layout = FlexLayout::new(FlexDirection::Vertical);
    main_layout.set_spacing(10);

    let mut window = Component::new("window");
    window.set_bounds(Bounds::new(0, 0, 400, 300)).unwrap();

    // Create header with horizontal layout
    let mut header = Component::new("header");
    header.set_bounds(Bounds::new(0, 0, 400, 50)).unwrap();

    let mut header_layout = FlexLayout::new(FlexDirection::Horizontal);
    header_layout.set_spacing(5);

    let mut title = Label::new("My Application");
    title.set_bounds(Bounds::new(0, 0, 100, 30)).unwrap();
    header.add_child(title.component().clone()).unwrap();

    let mut close_btn = Button::new("X");
    close_btn.set_bounds(Bounds::new(0, 0, 30, 30)).unwrap();
    header.add_child(close_btn.component().clone()).unwrap();

    header_layout.apply(&mut header).unwrap();

    // Create content area
    let mut content = Component::new("content");
    content.set_bounds(Bounds::new(0, 0, 400, 200)).unwrap();

    // Create footer with horizontal layout
    let mut footer = Component::new("footer");
    footer.set_bounds(Bounds::new(0, 0, 400, 50)).unwrap();

    let mut footer_layout = FlexLayout::new(FlexDirection::Horizontal);
    footer_layout.set_spacing(10);
    footer_layout.set_align(FlexAlign::End);

    let mut ok_btn = Button::new("OK");
    ok_btn.set_bounds(Bounds::new(0, 0, 80, 30)).unwrap();
    footer.add_child(ok_btn.component().clone()).unwrap();

    let mut cancel_btn = Button::new("Cancel");
    cancel_btn.set_bounds(Bounds::new(0, 0, 80, 30)).unwrap();
    footer.add_child(cancel_btn.component().clone()).unwrap();

    footer_layout.apply(&mut footer).unwrap();

    // Add sections to window
    window.add_child(header).unwrap();
    window.add_child(content).unwrap();
    window.add_child(footer).unwrap();

    main_layout.apply(&mut window).unwrap();

    println!("Window bounds: {:?}", window.bounds());
    for i in 0..window.child_count() {
        let section = window.child(i).unwrap();
        println!("  Section {} ({}) bounds: {:?}", i, section.name(), section.bounds());
    }
    println!();
}
