//! Integration tests for keyboard event handling.

use nih_plug_juce::{Component, KeyListener, KeyPress};
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn test_keyboard_focus() {
    // Test that we can enable keyboard focus on a component
    let mut component = Component::new_with_paint_callback().unwrap();
    
    let result = component.set_wants_keyboard_focus(true);
    assert!(result.is_ok(), "Setting keyboard focus should succeed");
    
    let result = component.set_wants_keyboard_focus(false);
    assert!(result.is_ok(), "Disabling keyboard focus should succeed");
}

#[test]
fn test_keyboard_listener() {
    // Create a listener that tracks events
    struct TestListener {
        events: Rc<RefCell<Vec<String>>>,
    }
    
    impl KeyListener for TestListener {
        fn key_pressed(&mut self, key: &KeyPress) -> bool {
            self.events.borrow_mut().push(format!("key_pressed({})", key.key_code));
            true
        }
        
        fn key_state_changed(&mut self) -> bool {
            self.events.borrow_mut().push("key_state_changed".to_string());
            false
        }
        
        fn focus_gained(&mut self) {
            self.events.borrow_mut().push("focus_gained".to_string());
        }
        
        fn focus_lost(&mut self) {
            self.events.borrow_mut().push("focus_lost".to_string());
        }
    }
    
    let events = Rc::new(RefCell::new(Vec::new()));
    let listener = TestListener {
        events: events.clone(),
    };
    
    // Create a component with callback support (which also supports keyboard listeners)
    let mut component = Component::new_with_paint_callback().unwrap();
    
    // Enable keyboard focus
    component.set_wants_keyboard_focus(true).unwrap();
    
    // Set the keyboard listener
    let result = component.set_key_listener(Box::new(listener));
    assert!(result.is_ok(), "Setting keyboard listener should succeed");
    
    // Note: We can't actually trigger keyboard events in a unit test without a real GUI,
    // but we've verified that the listener can be set without errors
}

#[test]
fn test_key_press_helpers() {
    use nih_plug_juce::events::mouse::ModifierKeys;
    
    let mods = ModifierKeys::none();
    
    // Test letter detection
    let key_a = KeyPress::new(65, mods); // 'A'
    assert!(key_a.is_letter());
    assert!(!key_a.is_digit());
    
    // Test digit detection
    let key_0 = KeyPress::new(48, mods); // '0'
    assert!(key_0.is_digit());
    assert!(!key_0.is_letter());
    
    // Test special keys
    let space = KeyPress::new(32, mods);
    assert!(space.is_space());
    
    let enter = KeyPress::new(13, mods);
    assert!(enter.is_return());
    
    let escape = KeyPress::new(27, mods);
    assert!(escape.is_escape());
}
