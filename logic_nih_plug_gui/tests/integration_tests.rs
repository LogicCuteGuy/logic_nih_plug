//! Integration tests for nih_plug_gui.

use nih_plug_gui::components::{Bounds, Component, ComponentState};

#[test]
fn test_component_hierarchy() {
    // Create a root component
    let mut root = Component::new("root");
    root.set_bounds(Bounds::new(0, 0, 800, 600)).unwrap();
    root.initialize();

    // Create child components
    let mut child1 = Component::new("child1");
    child1.set_bounds(Bounds::new(10, 10, 200, 100)).unwrap();

    let mut child2 = Component::new("child2");
    child2.set_bounds(Bounds::new(220, 10, 200, 100)).unwrap();

    // Add children to root
    root.add_child(child1.clone()).unwrap();
    root.add_child(child2.clone()).unwrap();

    assert_eq!(root.child_count(), 2);
    assert!(child1.has_parent());
    assert!(child2.has_parent());

    // Create grandchild
    let mut grandchild = Component::new("grandchild");
    grandchild.set_bounds(Bounds::new(5, 5, 50, 30)).unwrap();
    child1.add_child(grandchild.clone()).unwrap();

    assert_eq!(child1.child_count(), 1);
    assert!(grandchild.has_parent());
}

#[test]
fn test_component_removal() {
    let mut parent = Component::new("parent");
    parent.set_bounds(Bounds::new(0, 0, 400, 300)).unwrap();

    let child1 = Component::new("child1");
    let child2 = Component::new("child2");
    let child3 = Component::new("child3");

    parent.add_child(child1.clone()).unwrap();
    parent.add_child(child2.clone()).unwrap();
    parent.add_child(child3.clone()).unwrap();

    assert_eq!(parent.child_count(), 3);

    // Remove middle child
    let removed = parent.remove_child(1);
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().name(), "child2");
    assert_eq!(parent.child_count(), 2);
    assert!(!child2.has_parent());

    // Verify remaining children
    assert_eq!(parent.child(0).unwrap().name(), "child1");
    assert_eq!(parent.child(1).unwrap().name(), "child3");
}

#[test]
fn test_component_lifecycle_transitions() {
    let mut component = Component::new("test");
    
    // Start in Initializing state
    assert_eq!(component.state(), ComponentState::Initializing);
    
    // Initialize
    component.initialize();
    assert_eq!(component.state(), ComponentState::Active);
    
    // Can't initialize twice
    component.initialize();
    assert_eq!(component.state(), ComponentState::Active);
    
    // Destroy
    component.destroy();
    assert_eq!(component.state(), ComponentState::Destroying);
}

#[test]
fn test_component_visibility_and_enabled() {
    let mut component = Component::new("test");
    
    // Default state
    assert!(component.is_visible());
    assert!(component.is_enabled());
    
    // Hide component
    component.set_visible(false);
    assert!(!component.is_visible());
    
    // Disable component
    component.set_enabled(false);
    assert!(!component.is_enabled());
    
    // Re-enable
    component.set_visible(true);
    component.set_enabled(true);
    assert!(component.is_visible());
    assert!(component.is_enabled());
}

#[test]
fn test_bounds_validation() {
    let mut component = Component::new("test");
    
    // Valid bounds
    assert!(component.set_bounds(Bounds::new(0, 0, 100, 100)).is_ok());
    
    // Invalid bounds (zero width)
    assert!(component.set_bounds(Bounds::new(0, 0, 0, 100)).is_err());
    
    // Invalid bounds (zero height)
    assert!(component.set_bounds(Bounds::new(0, 0, 100, 0)).is_err());
}

#[test]
fn test_find_child_by_name_recursive() {
    let mut root = Component::new("root");
    root.set_bounds(Bounds::new(0, 0, 800, 600)).unwrap();
    
    let mut child1 = Component::new("child1");
    child1.set_bounds(Bounds::new(10, 10, 200, 100)).unwrap();
    
    let mut grandchild = Component::new("target");
    grandchild.set_bounds(Bounds::new(5, 5, 50, 30)).unwrap();
    
    child1.add_child(grandchild.clone()).unwrap();
    root.add_child(child1.clone()).unwrap();
    
    // Find direct child
    assert!(root.find_child_by_name("child1").is_some());
    
    // Find grandchild (only searches direct children)
    assert!(root.find_child_by_name("target").is_none());
    assert!(child1.find_child_by_name("target").is_some());
}

#[test]
fn test_component_destroy_removes_from_parent() {
    let mut parent = Component::new("parent");
    parent.set_bounds(Bounds::new(0, 0, 400, 300)).unwrap();
    
    let mut child = Component::new("child");
    child.set_bounds(Bounds::new(10, 10, 100, 50)).unwrap();
    
    parent.add_child(child.clone()).unwrap();
    assert_eq!(parent.child_count(), 1);
    
    // Destroy child
    child.destroy();
    
    // Child should be removed from parent
    assert_eq!(parent.child_count(), 0);
    assert!(!child.has_parent());
}

#[test]
fn test_remove_all_children() {
    let mut parent = Component::new("parent");
    parent.set_bounds(Bounds::new(0, 0, 400, 300)).unwrap();
    
    for i in 0..5 {
        let child = Component::new(&format!("child{}", i));
        parent.add_child(child).unwrap();
    }
    
    assert_eq!(parent.child_count(), 5);
    
    parent.remove_all_children();
    assert_eq!(parent.child_count(), 0);
}

#[test]
fn test_bounds_contains_point() {
    let bounds = Bounds::new(10, 20, 100, 50);
    
    // Inside
    assert!(bounds.contains(50, 40));
    assert!(bounds.contains(10, 20)); // Top-left corner
    assert!(bounds.contains(109, 69)); // Bottom-right corner (exclusive)
    
    // Outside
    assert!(!bounds.contains(9, 20)); // Left of bounds
    assert!(!bounds.contains(110, 40)); // Right of bounds
    assert!(!bounds.contains(50, 19)); // Above bounds
    assert!(!bounds.contains(50, 70)); // Below bounds
}

#[test]
fn test_component_clone_shares_data() {
    let mut component1 = Component::new("test");
    component1.set_bounds(Bounds::new(0, 0, 100, 100)).unwrap();
    
    let component2 = component1.clone();
    
    // Both should have the same ID (they share data)
    assert_eq!(component1.id(), component2.id());
    
    // Modifying one affects the other
    component1.set_visible(false);
    assert!(!component2.is_visible());
}

// Tests for UI controls

use nih_plug_gui::controls::{Button, ButtonState, Label, Slider, SliderOrientation, TextAlignment};

#[test]
fn test_button_integration() {
    let mut button = Button::new("Click Me");
    button.set_bounds(Bounds::new(10, 10, 100, 30)).unwrap();
    button.component_mut().initialize();
    
    assert_eq!(button.text(), "Click Me");
    assert_eq!(button.button_state(), ButtonState::Normal);
    assert!(button.is_enabled());
    assert!(button.is_visible());
    
    // Test state changes
    button.set_button_state(ButtonState::Hover);
    assert_eq!(button.button_state(), ButtonState::Hover);
    
    // Test disabling
    button.set_enabled(false);
    assert!(!button.is_enabled());
    assert_eq!(button.button_state(), ButtonState::Disabled);
}

#[test]
fn test_button_with_callback() {
    use std::sync::{Arc, Mutex};
    
    let mut button = Button::new("Test");
    let click_count = Arc::new(Mutex::new(0));
    let click_count_clone = click_count.clone();
    
    button.set_on_click(move || {
        *click_count_clone.lock().unwrap() += 1;
    });
    
    button.click();
    button.click();
    button.click();
    
    assert_eq!(*click_count.lock().unwrap(), 3);
}

#[test]
fn test_slider_integration() {
    let mut slider = Slider::new(SliderOrientation::Horizontal);
    slider.set_bounds(Bounds::new(10, 10, 200, 30)).unwrap();
    slider.component_mut().initialize();
    
    // Test range setting
    slider.set_range(0.0, 100.0).unwrap();
    assert_eq!(slider.min_value(), 0.0);
    assert_eq!(slider.max_value(), 100.0);
    
    // Test value setting and clamping
    slider.set_value(50.0);
    assert_eq!(slider.value(), 50.0);
    
    slider.set_value(150.0);
    assert_eq!(slider.value(), 100.0); // Clamped to max
    
    slider.set_value(-10.0);
    assert_eq!(slider.value(), 0.0); // Clamped to min
}

#[test]
fn test_slider_normalized_values() {
    let mut slider = Slider::new(SliderOrientation::Vertical);
    slider.set_range(0.0, 100.0).unwrap();
    
    slider.set_value(0.0);
    assert!((slider.normalized_value() - 0.0).abs() < f64::EPSILON);
    
    slider.set_value(50.0);
    assert!((slider.normalized_value() - 0.5).abs() < f64::EPSILON);
    
    slider.set_value(100.0);
    assert!((slider.normalized_value() - 1.0).abs() < f64::EPSILON);
    
    // Test setting normalized value
    slider.set_normalized_value(0.25);
    assert!((slider.value() - 25.0).abs() < f64::EPSILON);
}

#[test]
fn test_slider_with_callback() {
    use std::sync::{Arc, Mutex};
    
    let mut slider = Slider::new(SliderOrientation::Horizontal);
    slider.set_range(0.0, 100.0).unwrap();
    
    let values = Arc::new(Mutex::new(Vec::new()));
    let values_clone = values.clone();
    
    slider.set_on_value_change(move |v| {
        values_clone.lock().unwrap().push(v);
    });
    
    slider.set_value(25.0);
    slider.set_value(50.0);
    slider.set_value(75.0);
    
    let recorded_values = values.lock().unwrap();
    assert_eq!(recorded_values.len(), 3);
    assert_eq!(recorded_values[0], 25.0);
    assert_eq!(recorded_values[1], 50.0);
    assert_eq!(recorded_values[2], 75.0);
}

#[test]
fn test_label_integration() {
    let mut label = Label::new("Hello, World!");
    label.set_bounds(Bounds::new(10, 10, 200, 30)).unwrap();
    label.component_mut().initialize();
    
    assert_eq!(label.text(), "Hello, World!");
    assert_eq!(label.alignment(), TextAlignment::Left);
    assert_eq!(label.font_size(), 14);
    
    // Test text changes
    label.set_text("Updated Text");
    assert_eq!(label.text(), "Updated Text");
    
    // Test alignment
    label.set_alignment(TextAlignment::Center);
    assert_eq!(label.alignment(), TextAlignment::Center);
    
    label.set_alignment(TextAlignment::Right);
    assert_eq!(label.alignment(), TextAlignment::Right);
    
    // Test font size
    label.set_font_size(20);
    assert_eq!(label.font_size(), 20);
}

#[test]
fn test_controls_as_components() {
    // Test that controls can be used as components in a hierarchy
    let mut parent = Component::new("parent");
    parent.set_bounds(Bounds::new(0, 0, 400, 300)).unwrap();
    
    let mut button = Button::new("Button");
    button.set_bounds(Bounds::new(10, 10, 100, 30)).unwrap();
    
    let mut slider = Slider::new(SliderOrientation::Horizontal);
    slider.set_bounds(Bounds::new(10, 50, 200, 30)).unwrap();
    
    let mut label = Label::new("Label");
    label.set_bounds(Bounds::new(10, 90, 100, 30)).unwrap();
    
    // Add controls as children (using their underlying components)
    parent.add_child(button.component().clone()).unwrap();
    parent.add_child(slider.component().clone()).unwrap();
    parent.add_child(label.component().clone()).unwrap();
    
    assert_eq!(parent.child_count(), 3);
}

#[test]
fn test_slider_orientation() {
    let mut slider = Slider::new(SliderOrientation::Horizontal);
    assert_eq!(slider.orientation(), SliderOrientation::Horizontal);
    
    slider.set_orientation(SliderOrientation::Vertical);
    assert_eq!(slider.orientation(), SliderOrientation::Vertical);
}

#[test]
fn test_button_text_update() {
    let mut button = Button::new("Initial");
    assert_eq!(button.text(), "Initial");
    
    button.set_text("Updated");
    assert_eq!(button.text(), "Updated");
    
    button.set_text("");
    assert_eq!(button.text(), "");
}

#[test]
fn test_slider_invalid_range_error() {
    let mut slider = Slider::new(SliderOrientation::Horizontal);
    
    // Min >= Max should fail
    assert!(slider.set_range(100.0, 0.0).is_err());
    assert!(slider.set_range(50.0, 50.0).is_err());
    
    // Valid range should succeed
    assert!(slider.set_range(0.0, 100.0).is_ok());
}

// Tests for input handling

use nih_plug_gui::input::{
    EventResult, InputCallbacks, KeyCode, KeyboardEvent, Modifiers, MouseButton, MouseEvent,
};

#[test]
fn test_mouse_event_dispatch() {
    let mut component = Component::new("test");
    component.set_bounds(Bounds::new(0, 0, 100, 100)).unwrap();
    component.initialize();

    let event = MouseEvent::ButtonDown {
        x: 50,
        y: 50,
        button: MouseButton::Left,
        modifiers: Modifiers::none(),
    };

    // Default implementation returns NotHandled
    let result = component.dispatch_mouse_event(&event);
    assert_eq!(result, EventResult::NotHandled);
}

#[test]
fn test_mouse_event_out_of_bounds() {
    let mut component = Component::new("test");
    component.set_bounds(Bounds::new(0, 0, 100, 100)).unwrap();

    // Event outside component bounds
    let event = MouseEvent::ButtonDown {
        x: 150,
        y: 150,
        button: MouseButton::Left,
        modifiers: Modifiers::none(),
    };

    let result = component.dispatch_mouse_event(&event);
    assert_eq!(result, EventResult::NotHandled);
}

#[test]
fn test_keyboard_event_dispatch() {
    let mut component = Component::new("test");
    component.initialize();

    let event = KeyboardEvent::KeyDown {
        key: KeyCode::A,
        character: Some('a'),
        modifiers: Modifiers::none(),
        repeat: false,
    };

    let result = component.dispatch_keyboard_event(&event);
    assert_eq!(result, EventResult::NotHandled);
}

#[test]
fn test_keyboard_event_disabled_component() {
    let mut component = Component::new("test");
    component.set_enabled(false);

    let event = KeyboardEvent::KeyDown {
        key: KeyCode::A,
        character: Some('a'),
        modifiers: Modifiers::none(),
        repeat: false,
    };

    let result = component.dispatch_keyboard_event(&event);
    assert_eq!(result, EventResult::NotHandled);
}

#[test]
fn test_input_callbacks_mouse() {
    use std::sync::{Arc, Mutex};

    let mut callbacks = InputCallbacks::new();
    let component_id = 1;

    let handled = Arc::new(Mutex::new(false));
    let handled_clone = handled.clone();

    callbacks.add_mouse_callback(component_id, move |event| {
        *handled_clone.lock().unwrap() = true;
        let _ = event;
        EventResult::Handled
    });

    let event = MouseEvent::ButtonDown {
        x: 10,
        y: 20,
        button: MouseButton::Left,
        modifiers: Modifiers::none(),
    };

    let result = callbacks.dispatch_mouse_event(component_id, &event);
    assert_eq!(result, EventResult::Handled);
    assert!(*handled.lock().unwrap());
}

#[test]
fn test_input_callbacks_keyboard() {
    use std::sync::{Arc, Mutex};

    let mut callbacks = InputCallbacks::new();
    let component_id = 1;

    let key_pressed = Arc::new(Mutex::new(None));
    let key_pressed_clone = key_pressed.clone();

    callbacks.add_keyboard_callback(component_id, move |event| {
        if let Some(key) = event.key_code() {
            *key_pressed_clone.lock().unwrap() = Some(key);
        }
        EventResult::Handled
    });

    let event = KeyboardEvent::KeyDown {
        key: KeyCode::Space,
        character: Some(' '),
        modifiers: Modifiers::none(),
        repeat: false,
    };

    let result = callbacks.dispatch_keyboard_event(component_id, &event);
    assert_eq!(result, EventResult::Handled);
    assert_eq!(*key_pressed.lock().unwrap(), Some(KeyCode::Space));
}

#[test]
fn test_input_callbacks_remove() {
    let mut callbacks = InputCallbacks::new();
    let component_id = 1;

    callbacks.add_mouse_callback(component_id, |_| EventResult::Handled);
    callbacks.add_keyboard_callback(component_id, |_| EventResult::Handled);

    // Remove all callbacks for the component
    callbacks.remove_callbacks(component_id);

    let mouse_event = MouseEvent::ButtonDown {
        x: 10,
        y: 20,
        button: MouseButton::Left,
        modifiers: Modifiers::none(),
    };

    let keyboard_event = KeyboardEvent::KeyDown {
        key: KeyCode::A,
        character: Some('a'),
        modifiers: Modifiers::none(),
        repeat: false,
    };

    // Both should return NotHandled after removal
    assert_eq!(
        callbacks.dispatch_mouse_event(component_id, &mouse_event),
        EventResult::NotHandled
    );
    assert_eq!(
        callbacks.dispatch_keyboard_event(component_id, &keyboard_event),
        EventResult::NotHandled
    );
}

#[test]
fn test_modifiers() {
    let mods = Modifiers {
        shift: true,
        ctrl: false,
        alt: true,
        meta: false,
    };

    assert!(mods.shift);
    assert!(!mods.ctrl);
    assert!(mods.alt);
    assert!(!mods.meta);
    assert!(mods.any());

    let no_mods = Modifiers::none();
    assert!(!no_mods.any());
}

#[test]
fn test_mouse_event_types() {
    let event = MouseEvent::ButtonDown {
        x: 10,
        y: 20,
        button: MouseButton::Left,
        modifiers: Modifiers::none(),
    };
    assert_eq!(event.position(), (10, 20));
    assert!(event.modifiers().is_some());

    let event = MouseEvent::Enter { x: 5, y: 10 };
    assert_eq!(event.position(), (5, 10));
    assert!(event.modifiers().is_none());

    let event = MouseEvent::Wheel {
        x: 15,
        y: 25,
        delta_x: 0.0,
        delta_y: 1.0,
        modifiers: Modifiers::none(),
    };
    assert_eq!(event.position(), (15, 25));
    assert!(event.modifiers().is_some());
}

#[test]
fn test_keyboard_event_types() {
    let event = KeyboardEvent::KeyDown {
        key: KeyCode::A,
        character: Some('a'),
        modifiers: Modifiers::none(),
        repeat: false,
    };
    assert_eq!(event.key_code(), Some(KeyCode::A));
    assert!(event.modifiers().is_some());

    let event = KeyboardEvent::TextInput {
        text: "hello".to_string(),
    };
    assert_eq!(event.key_code(), None);
    assert!(event.modifiers().is_none());
}

#[test]
fn test_mouse_button_types() {
    let left = MouseButton::Left;
    let right = MouseButton::Right;
    let middle = MouseButton::Middle;
    let other = MouseButton::Other(4);

    assert_ne!(left, right);
    assert_ne!(left, middle);
    assert_ne!(left, other);
}

#[test]
fn test_key_code_types() {
    assert_eq!(KeyCode::A, KeyCode::A);
    assert_ne!(KeyCode::A, KeyCode::B);
    assert_ne!(KeyCode::Num0, KeyCode::Num1);
    assert_ne!(KeyCode::F1, KeyCode::F2);
    assert_ne!(KeyCode::Left, KeyCode::Right);
}

// Tests for layout management

#[cfg(feature = "layout")]
use nih_plug_gui::layout::{
    AbsoluteLayout, FlexAlign, FlexDirection, FlexLayout, GridLayout, SizeConstraint,
};

#[cfg(feature = "layout")]
#[test]
fn test_flex_layout_integration() {
    let mut layout = FlexLayout::new(FlexDirection::Horizontal);
    layout.set_spacing(10);
    layout.set_padding(5, 5, 5, 5);

    let mut parent = Component::new("parent");
    parent.set_bounds(Bounds::new(0, 0, 400, 100)).unwrap();

    // Add three children
    for i in 0..3 {
        let child = Component::new(&format!("child{}", i));
        parent.add_child(child).unwrap();
    }

    layout.apply(&mut parent).unwrap();

    // Verify children are laid out horizontally with spacing
    let c0 = parent.child(0).unwrap();
    let c1 = parent.child(1).unwrap();
    let c2 = parent.child(2).unwrap();

    // All should have same y position (accounting for padding)
    assert_eq!(c0.bounds().y, 5);
    assert_eq!(c1.bounds().y, 5);
    assert_eq!(c2.bounds().y, 5);

    // X positions should increase with spacing
    assert!(c1.bounds().x > c0.bounds().x);
    assert!(c2.bounds().x > c1.bounds().x);

    // Check spacing is correct
    let spacing_between_0_and_1 = c1.bounds().x - (c0.bounds().x + c0.bounds().width as i32);
    assert_eq!(spacing_between_0_and_1, 10);
}

#[cfg(feature = "layout")]
#[test]
fn test_flex_layout_vertical_integration() {
    let mut layout = FlexLayout::new(FlexDirection::Vertical);
    layout.set_spacing(5);

    let mut parent = Component::new("parent");
    parent.set_bounds(Bounds::new(0, 0, 100, 300)).unwrap();

    // Add three children
    for i in 0..3 {
        let child = Component::new(&format!("child{}", i));
        parent.add_child(child).unwrap();
    }

    layout.apply(&mut parent).unwrap();

    // Verify children are laid out vertically with spacing
    let c0 = parent.child(0).unwrap();
    let c1 = parent.child(1).unwrap();
    let c2 = parent.child(2).unwrap();

    // All should have same x position
    assert_eq!(c0.bounds().x, 0);
    assert_eq!(c1.bounds().x, 0);
    assert_eq!(c2.bounds().x, 0);

    // Y positions should increase with spacing
    assert!(c1.bounds().y > c0.bounds().y);
    assert!(c2.bounds().y > c1.bounds().y);

    // Check spacing is correct
    let spacing_between_0_and_1 = c1.bounds().y - (c0.bounds().y + c0.bounds().height as i32);
    assert_eq!(spacing_between_0_and_1, 5);
}

#[cfg(feature = "layout")]
#[test]
fn test_flex_layout_alignment() {
    let mut layout = FlexLayout::new(FlexDirection::Horizontal);
    layout.set_align(FlexAlign::Center);

    let mut parent = Component::new("parent");
    parent.set_bounds(Bounds::new(0, 0, 400, 100)).unwrap();

    let child = Component::new("child");
    parent.add_child(child.clone()).unwrap();

    layout.apply(&mut parent).unwrap();

    // Child should be centered vertically
    let c = parent.child(0).unwrap();
    let parent_height = parent.bounds().height;
    let child_height = c.bounds().height;
    let expected_y = (parent_height - child_height) / 2;
    assert_eq!(c.bounds().y as u32, expected_y);
}

#[cfg(feature = "layout")]
#[test]
fn test_grid_layout_integration() {
    let mut layout = GridLayout::new(2, 3).unwrap();
    layout.set_spacing(5);
    layout.set_padding(10, 10, 10, 10);

    let mut parent = Component::new("parent");
    parent.set_bounds(Bounds::new(0, 0, 400, 300)).unwrap();

    // Add 6 children (fills the 2x3 grid)
    for i in 0..6 {
        let child = Component::new(&format!("child{}", i));
        parent.add_child(child).unwrap();
    }

    layout.apply(&mut parent).unwrap();

    // Verify grid layout
    let c0 = parent.child(0).unwrap(); // Row 0, Col 0
    let c1 = parent.child(1).unwrap(); // Row 0, Col 1
    let c2 = parent.child(2).unwrap(); // Row 0, Col 2
    let c3 = parent.child(3).unwrap(); // Row 1, Col 0

    // First row should have same y
    assert_eq!(c0.bounds().y, c1.bounds().y);
    assert_eq!(c1.bounds().y, c2.bounds().y);

    // Second row should be below first row
    assert!(c3.bounds().y > c0.bounds().y);

    // Columns should have increasing x
    assert!(c1.bounds().x > c0.bounds().x);
    assert!(c2.bounds().x > c1.bounds().x);

    // All cells should have same size
    assert_eq!(c0.bounds().width, c1.bounds().width);
    assert_eq!(c0.bounds().height, c3.bounds().height);
}

#[cfg(feature = "layout")]
#[test]
fn test_absolute_layout_with_constraints() {
    let mut layout = AbsoluteLayout::new();
    layout.add_constraint(0, SizeConstraint::new().with_fixed_size(100, 50));
    layout.add_constraint(1, SizeConstraint::new().with_min_width(150).with_max_height(80));

    let mut parent = Component::new("parent");
    parent.set_bounds(Bounds::new(0, 0, 400, 300)).unwrap();

    let mut child1 = Component::new("child1");
    child1.set_bounds(Bounds::new(10, 10, 200, 100)).unwrap();

    let mut child2 = Component::new("child2");
    child2.set_bounds(Bounds::new(50, 50, 100, 150)).unwrap();

    parent.add_child(child1.clone()).unwrap();
    parent.add_child(child2.clone()).unwrap();

    layout.apply(&mut parent).unwrap();

    // Child 1 should have fixed size
    let c1 = parent.child(0).unwrap();
    assert_eq!(c1.bounds().width, 100);
    assert_eq!(c1.bounds().height, 50);
    // Position unchanged
    assert_eq!(c1.bounds().x, 10);
    assert_eq!(c1.bounds().y, 10);

    // Child 2 should respect min width and max height
    let c2 = parent.child(1).unwrap();
    assert_eq!(c2.bounds().width, 150); // Min width applied
    assert_eq!(c2.bounds().height, 80); // Max height applied
    // Position unchanged
    assert_eq!(c2.bounds().x, 50);
    assert_eq!(c2.bounds().y, 50);
}

#[cfg(feature = "layout")]
#[test]
fn test_size_constraint_preferred_size() {
    let constraint = SizeConstraint::new()
        .with_preferred_width(200)
        .with_preferred_height(100);

    assert_eq!(constraint.preferred_size(), Some((200, 100)));

    let constraint_no_pref = SizeConstraint::new().with_min_width(50);
    assert_eq!(constraint_no_pref.preferred_size(), None);
}

#[cfg(feature = "layout")]
#[test]
fn test_layout_with_controls() {
    // Test that layout works with actual UI controls
    let mut layout = FlexLayout::new(FlexDirection::Vertical);
    layout.set_spacing(10);

    let mut parent = Component::new("parent");
    parent.set_bounds(Bounds::new(0, 0, 300, 200)).unwrap();

    let mut button = Button::new("Button");
    button.set_bounds(Bounds::new(0, 0, 100, 30)).unwrap();

    let mut slider = Slider::new(SliderOrientation::Horizontal);
    slider.set_bounds(Bounds::new(0, 0, 200, 30)).unwrap();

    let mut label = Label::new("Label");
    label.set_bounds(Bounds::new(0, 0, 100, 20)).unwrap();

    parent.add_child(button.component().clone()).unwrap();
    parent.add_child(slider.component().clone()).unwrap();
    parent.add_child(label.component().clone()).unwrap();

    // Apply layout
    layout.apply(&mut parent).unwrap();

    // Verify controls are laid out vertically
    let c0 = parent.child(0).unwrap();
    let c1 = parent.child(1).unwrap();
    let c2 = parent.child(2).unwrap();

    assert!(c1.bounds().y > c0.bounds().y);
    assert!(c2.bounds().y > c1.bounds().y);
}

#[cfg(feature = "layout")]
#[test]
fn test_grid_layout_with_extra_children() {
    let layout = GridLayout::new(2, 2).unwrap();

    let mut parent = Component::new("parent");
    parent.set_bounds(Bounds::new(0, 0, 200, 200)).unwrap();

    // Add 6 children (grid only has 4 cells)
    for i in 0..6 {
        let child = Component::new(&format!("child{}", i));
        parent.add_child(child).unwrap();
    }

    // Should not error, just layout first 4
    layout.apply(&mut parent).unwrap();

    // First 4 should be laid out
    assert!(parent.child(0).is_some());
    assert!(parent.child(3).is_some());

    // Last 2 should still exist but not be laid out by grid
    assert!(parent.child(4).is_some());
    assert!(parent.child(5).is_some());
}

#[cfg(feature = "layout")]
#[test]
fn test_flex_layout_stretch_alignment() {
    let mut layout = FlexLayout::new(FlexDirection::Horizontal);
    layout.set_align(FlexAlign::Stretch);

    let mut parent = Component::new("parent");
    parent.set_bounds(Bounds::new(0, 0, 400, 100)).unwrap();

    let child = Component::new("child");
    parent.add_child(child.clone()).unwrap();

    layout.apply(&mut parent).unwrap();

    // Child should stretch to fill parent height
    let c = parent.child(0).unwrap();
    assert_eq!(c.bounds().height, parent.bounds().height);
}
