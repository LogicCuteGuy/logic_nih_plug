//! Integration tests for JUCE ListBox FFI bindings.
//!
//! These tests verify that the ListBox component and ListBoxModel trait
//! work correctly through the FFI boundary.

use nih_plug_juce::containers::{ListBox, ListBoxModel};
use nih_plug_juce::drawing::Colour;
use nih_plug_juce::Graphics;
use std::sync::{Arc, Mutex};

/// Simple test model for ListBox
struct TestListModel {
    items: Vec<String>,
    last_selected: Arc<Mutex<Option<i32>>>,
}

impl TestListModel {
    fn new(items: Vec<String>) -> Self {
        TestListModel {
            items,
            last_selected: Arc::new(Mutex::new(None)),
        }
    }
}

impl ListBoxModel for TestListModel {
    fn get_num_rows(&self) -> i32 {
        self.items.len() as i32
    }

    fn paint_list_box_item(&self, row: i32, g: &mut Graphics, width: i32, height: i32, selected: bool) {
        // Draw a simple background
        if selected {
            let blue = Colour::from_rgb(100, 100, 255).unwrap();
            g.set_colour(&blue);
            g.fill_rect(0, 0, width, height);
        }

        // Draw the item text
        if row >= 0 && (row as usize) < self.items.len() {
            let black = Colour::from_rgb(0, 0, 0).unwrap();
            g.set_colour(&black);
            g.draw_text(
                &self.items[row as usize],
                5,
                0,
                width - 10,
                height,
                nih_plug_juce::graphics::Justification::CentredLeft,
            );
        }
    }

    fn selected_rows_changed(&mut self, last_row_selected: i32) {
        *self.last_selected.lock().unwrap() = Some(last_row_selected);
    }
}

#[test]
fn test_list_box_creation() {
    // Test that we can create a ListBox
    let result = ListBox::new();
    assert!(result.is_ok(), "Failed to create ListBox: {:?}", result.err());
}

#[test]
fn test_list_box_set_model() {
    // Create a ListBox
    let mut list_box = ListBox::new().expect("Failed to create ListBox");

    // Create a test model
    let items = vec![
        "Item 1".to_string(),
        "Item 2".to_string(),
        "Item 3".to_string(),
    ];
    let model = Box::new(TestListModel::new(items));

    // Set the model
    let result = list_box.set_model(model);
    assert!(result.is_ok(), "Failed to set model: {:?}", result.err());
}

#[test]
fn test_list_box_update_content() {
    // Create a ListBox
    let mut list_box = ListBox::new().expect("Failed to create ListBox");

    // Create a test model
    let items = vec!["Item 1".to_string(), "Item 2".to_string()];
    let model = Box::new(TestListModel::new(items));

    // Set the model
    list_box.set_model(model).expect("Failed to set model");

    // Update content should not crash
    list_box.update_content();
}

#[test]
fn test_list_box_with_empty_model() {
    // Create a ListBox
    let mut list_box = ListBox::new().expect("Failed to create ListBox");

    // Create an empty model
    let model = Box::new(TestListModel::new(vec![]));

    // Set the model
    let result = list_box.set_model(model);
    assert!(result.is_ok(), "Failed to set empty model: {:?}", result.err());

    // Update content should not crash
    list_box.update_content();
}

#[test]
fn test_list_box_component_methods() {
    // Create a ListBox
    let mut list_box = ListBox::new().expect("Failed to create ListBox");

    // Test that we can use Component methods through Deref
    list_box.set_bounds(0, 0, 200, 300);
    list_box.set_visible(true);
    list_box.repaint();

    // These should not crash
}

#[test]
fn test_list_box_model_with_many_items() {
    // Create a ListBox
    let mut list_box = ListBox::new().expect("Failed to create ListBox");

    // Create a model with many items
    let items: Vec<String> = (0..100).map(|i| format!("Item {}", i)).collect();
    let model = Box::new(TestListModel::new(items));

    // Set the model
    let result = list_box.set_model(model);
    assert!(result.is_ok(), "Failed to set model with many items: {:?}", result.err());

    // Update content should not crash
    list_box.update_content();
}
