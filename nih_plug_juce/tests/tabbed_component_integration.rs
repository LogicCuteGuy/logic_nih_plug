//! Integration tests for TabbedComponent.
//!
//! These tests verify that the TabbedComponent FFI bridge works correctly
//! by creating tabbed components, adding tabs, and testing callbacks.

use nih_plug_juce::containers::{TabbedComponent, TabOrientation};
use nih_plug_juce::component::Component;
use nih_plug_juce::drawing::Colour;
use std::sync::{Arc, Mutex};

#[test]
fn test_tabbed_component_creation() {
    // Test creating a tabbed component with different orientations
    let tabbed_top = TabbedComponent::new(TabOrientation::Top);
    assert!(tabbed_top.is_ok(), "Failed to create TabbedComponent with tabs at top");
    
    let tabbed_bottom = TabbedComponent::new(TabOrientation::Bottom);
    assert!(tabbed_bottom.is_ok(), "Failed to create TabbedComponent with tabs at bottom");
    
    let tabbed_left = TabbedComponent::new(TabOrientation::Left);
    assert!(tabbed_left.is_ok(), "Failed to create TabbedComponent with tabs at left");
    
    let tabbed_right = TabbedComponent::new(TabOrientation::Right);
    assert!(tabbed_right.is_ok(), "Failed to create TabbedComponent with tabs at right");
}

#[test]
fn test_add_tab() {
    let mut tabbed = TabbedComponent::new(TabOrientation::Top)
        .expect("Failed to create TabbedComponent");
    
    // Create content for the tab
    let content = Component::new().expect("Failed to create content component");
    
    // Create a colour for the tab
    let colour = Colour::from_rgb(100, 100, 100).expect("Failed to create colour");
    
    // Add a tab
    let result = tabbed.add_tab("Test Tab", colour, content);
    assert!(result.is_ok(), "Failed to add tab: {:?}", result.err());
}

#[test]
fn test_add_multiple_tabs() {
    let mut tabbed = TabbedComponent::new(TabOrientation::Top)
        .expect("Failed to create TabbedComponent");
    
    // Add multiple tabs
    for i in 0..3 {
        let content = Component::new().expect("Failed to create content component");
        let colour = Colour::from_rgb(100 + i * 20, 100, 100).expect("Failed to create colour");
        let tab_name = format!("Tab {}", i + 1);
        
        let result = tabbed.add_tab(&tab_name, colour, content);
        assert!(result.is_ok(), "Failed to add tab {}: {:?}", i + 1, result.err());
    }
}

#[test]
fn test_remove_tab() {
    let mut tabbed = TabbedComponent::new(TabOrientation::Top)
        .expect("Failed to create TabbedComponent");
    
    // Add some tabs
    for i in 0..3 {
        let content = Component::new().expect("Failed to create content component");
        let colour = Colour::from_rgb(100, 100, 100).expect("Failed to create colour");
        let tab_name = format!("Tab {}", i + 1);
        tabbed.add_tab(&tab_name, colour, content).expect("Failed to add tab");
    }
    
    // Remove the first tab
    tabbed.remove_tab(0);
    
    // This test just verifies that remove_tab doesn't crash
    // We can't easily verify the tab count without additional FFI functions
}

#[test]
fn test_set_current_tab_index() {
    let mut tabbed = TabbedComponent::new(TabOrientation::Top)
        .expect("Failed to create TabbedComponent");
    
    // Add some tabs
    for i in 0..3 {
        let content = Component::new().expect("Failed to create content component");
        let colour = Colour::from_rgb(100, 100, 100).expect("Failed to create colour");
        let tab_name = format!("Tab {}", i + 1);
        tabbed.add_tab(&tab_name, colour, content).expect("Failed to add tab");
    }
    
    // Set the current tab to the second tab
    tabbed.set_current_tab_index(1);
    
    // This test just verifies that set_current_tab_index doesn't crash
    // We can't easily verify the current tab without additional FFI functions
}

#[test]
fn test_tab_changed_callback() {
    let mut tabbed = TabbedComponent::new(TabOrientation::Top)
        .expect("Failed to create TabbedComponent");
    
    // Add some tabs
    for i in 0..3 {
        let content = Component::new().expect("Failed to create content component");
        let colour = Colour::from_rgb(100, 100, 100).expect("Failed to create colour");
        let tab_name = format!("Tab {}", i + 1);
        tabbed.add_tab(&tab_name, colour, content).expect("Failed to add tab");
    }
    
    // Set up a callback to track tab changes
    let callback_invoked = Arc::new(Mutex::new(false));
    let callback_invoked_clone = callback_invoked.clone();
    let last_index = Arc::new(Mutex::new(-1));
    let last_index_clone = last_index.clone();
    
    let result = tabbed.set_on_tab_changed(move |index| {
        *callback_invoked_clone.lock().unwrap() = true;
        *last_index_clone.lock().unwrap() = index;
    });
    
    assert!(result.is_ok(), "Failed to set tab changed callback: {:?}", result.err());
    
    // Note: We can't easily trigger the callback in a test without a message loop
    // This test just verifies that setting the callback doesn't crash
}

#[test]
fn test_tabbed_component_inherits_from_component() {
    let mut tabbed = TabbedComponent::new(TabOrientation::Top)
        .expect("Failed to create TabbedComponent");
    
    // Test that we can use Component methods through Deref
    tabbed.set_bounds(0, 0, 400, 300);
    tabbed.set_visible(true);
    tabbed.repaint();
    
    // This test verifies that TabbedComponent properly inherits from Component
}

#[test]
fn test_tab_orientation_values() {
    // Test that all orientation values work
    let orientations = vec![
        TabOrientation::Top,
        TabOrientation::Bottom,
        TabOrientation::Left,
        TabOrientation::Right,
    ];
    
    for orientation in orientations {
        let result = TabbedComponent::new(orientation);
        assert!(result.is_ok(), "Failed to create TabbedComponent with orientation {:?}", orientation);
    }
}
