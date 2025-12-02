//! Integration tests for FlexBox layout system.

use nih_plug_juce::{Component, FlexBox, FlexItem, FlexDirection, FlexWrap};

#[test]
fn test_flexbox_basic_layout() {
    // Create a flexbox
    let mut flexbox = FlexBox::new().expect("Failed to create FlexBox");
    
    // Set layout properties
    flexbox.set_direction(FlexDirection::Row);
    flexbox.set_wrap(FlexWrap::NoWrap);
    
    // Create some components
    let component1 = Component::new().expect("Failed to create component 1");
    let component2 = Component::new().expect("Failed to create component 2");
    let component3 = Component::new().expect("Failed to create component 3");
    
    // Create flex items with different grow factors
    let item1 = FlexItem::new(&component1)
        .with_flex_grow(1.0)
        .with_min_width(50.0)
        .with_min_height(50.0);
    
    let item2 = FlexItem::new(&component2)
        .with_flex_grow(2.0)
        .with_min_width(50.0)
        .with_min_height(50.0);
    
    let item3 = FlexItem::new(&component3)
        .with_flex_grow(1.0)
        .with_min_width(50.0)
        .with_min_height(50.0);
    
    // Add items to flexbox
    flexbox.add_item(item1);
    flexbox.add_item(item2);
    flexbox.add_item(item3);
    
    // Perform layout
    flexbox.perform_layout(0, 0, 800, 600);
    
    // If we get here without crashing, the test passed
}

#[test]
fn test_flexbox_column_layout() {
    // Create a flexbox with column direction
    let mut flexbox = FlexBox::new().expect("Failed to create FlexBox");
    flexbox.set_direction(FlexDirection::Column);
    
    // Create components
    let component1 = Component::new().expect("Failed to create component 1");
    let component2 = Component::new().expect("Failed to create component 2");
    
    // Create flex items
    let item1 = FlexItem::new(&component1)
        .with_flex_grow(1.0)
        .with_margin(10.0, 10.0, 10.0, 10.0);
    
    let item2 = FlexItem::new(&component2)
        .with_flex_grow(1.0)
        .with_margin(10.0, 10.0, 10.0, 10.0);
    
    // Add items
    flexbox.add_item(item1);
    flexbox.add_item(item2);
    
    // Perform layout
    flexbox.perform_layout(0, 0, 400, 600);
}

#[test]
fn test_flexbox_with_constraints() {
    // Create a flexbox
    let mut flexbox = FlexBox::new().expect("Failed to create FlexBox");
    flexbox.set_direction(FlexDirection::Row);
    
    // Create a component with size constraints
    let component = Component::new().expect("Failed to create component");
    
    let item = FlexItem::new(&component)
        .with_flex_grow(1.0)
        .with_flex_shrink(0.5)
        .with_flex_basis(100.0)
        .with_min_width(50.0)
        .with_min_height(50.0)
        .with_max_width(200.0)
        .with_max_height(200.0);
    
    flexbox.add_item(item);
    
    // Perform layout
    flexbox.perform_layout(0, 0, 800, 600);
}

#[test]
fn test_flexbox_wrap() {
    // Create a flexbox with wrapping enabled
    let mut flexbox = FlexBox::new().expect("Failed to create FlexBox");
    flexbox.set_direction(FlexDirection::Row);
    flexbox.set_wrap(FlexWrap::Wrap);
    
    // Create multiple components that should wrap
    // Keep components alive for the duration of the test
    let components: Vec<_> = (0..5)
        .map(|_| Component::new().expect("Failed to create component"))
        .collect();
    
    for component in &components {
        let item = FlexItem::new(component)
            .with_flex_basis(200.0)
            .with_min_width(200.0)
            .with_min_height(100.0);
        flexbox.add_item(item);
    }
    
    // Perform layout in a narrow container to force wrapping
    flexbox.perform_layout(0, 0, 500, 600);
}

#[test]
fn test_flexbox_empty() {
    // Test that an empty flexbox doesn't crash
    let mut flexbox = FlexBox::new().expect("Failed to create FlexBox");
    flexbox.perform_layout(0, 0, 800, 600);
}
