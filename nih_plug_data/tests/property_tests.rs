//! Property-based tests for nih_plug_data.

use nih_plug_data::{Value, ValueTree, ValueTreeListener};
use proptest::prelude::*;
use std::sync::{Arc, Mutex};

/// A test listener that records all notifications.
#[derive(Clone)]
struct RecordingListener {
    value_changes: Arc<Mutex<Vec<(String, String)>>>,
    child_additions: Arc<Mutex<Vec<(String, String)>>>,
    child_removals: Arc<Mutex<Vec<(String, String)>>>,
}

impl RecordingListener {
    fn new() -> Self {
        Self {
            value_changes: Arc::new(Mutex::new(Vec::new())),
            child_additions: Arc::new(Mutex::new(Vec::new())),
            child_removals: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn value_change_count(&self) -> usize {
        self.value_changes.lock().unwrap().len()
    }

    fn child_addition_count(&self) -> usize {
        self.child_additions.lock().unwrap().len()
    }

    fn child_removal_count(&self) -> usize {
        self.child_removals.lock().unwrap().len()
    }
}

impl ValueTreeListener for RecordingListener {
    fn value_changed(&mut self, tree: &ValueTree, property: &str) {
        self.value_changes
            .lock()
            .unwrap()
            .push((tree.type_name().to_string(), property.to_string()));
    }

    fn child_added(&mut self, parent: &ValueTree, child: &ValueTree) {
        self.child_additions
            .lock()
            .unwrap()
            .push((parent.type_name().to_string(), child.type_name().to_string()));
    }

    fn child_removed(&mut self, parent: &ValueTree, child: &ValueTree) {
        self.child_removals
            .lock()
            .unwrap()
            .push((parent.type_name().to_string(), child.type_name().to_string()));
    }
}

// Strategy for generating Value enum
fn value_strategy() -> impl Strategy<Value = Value> {
    prop_oneof![
        any::<i32>().prop_map(Value::Int),
        any::<f32>().prop_map(Value::Float),
        any::<bool>().prop_map(Value::Bool),
        "[a-zA-Z0-9 ]{0,50}".prop_map(|s| Value::String(s)),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 100,
        ..ProptestConfig::default()
    })]

    /// **Feature: juce-modules-integration, Property 5: ValueTree serialization round-trip**
    /// **Validates: Requirements 16.3**
    ///
    /// For any ValueTree structure, serializing to XML and then deserializing should
    /// produce an equivalent ValueTree. Similarly, binary serialization should round-trip.
    #[test]
    #[cfg(feature = "valuetree")]
    fn prop_valuetree_xml_serialization_roundtrip(
        type_name in "[a-zA-Z][a-zA-Z0-9_]{0,19}",
        properties in prop::collection::hash_map("[a-zA-Z][a-zA-Z0-9_]{0,9}", value_strategy(), 0..10),
        num_children in 0usize..3,
    ) {
        // Create a ValueTree with properties
        let mut tree = ValueTree::new(&type_name);
        for (key, value) in properties.iter() {
            tree.set_property(key, value.clone());
        }

        // Add some children (limited depth to avoid stack overflow)
        for i in 0..num_children {
            let mut child = ValueTree::new(&format!("child_{}", i));
            child.set_property("index", Value::Int(i as i32));
            tree.add_child(child);
        }

        // Serialize to XML
        let xml = tree.to_xml();

        // Deserialize from XML
        let restored = ValueTree::from_xml(&xml)
            .expect("XML deserialization should succeed");

        // Verify structure
        prop_assert_eq!(tree.type_name(), restored.type_name());
        prop_assert_eq!(tree.num_children(), restored.num_children());

        // Verify properties
        for (key, value) in properties.iter() {
            let restored_value = restored.get_property(key);
            prop_assert!(restored_value.is_some(), "Property '{}' should exist", key);
            
            // For floats, we need approximate comparison due to string conversion
            match (value, restored_value.unwrap()) {
                (Value::Float(expected), Value::Float(actual)) => {
                    // Allow some precision loss from XML string conversion
                    prop_assert!((expected - actual).abs() < 0.0001 || 
                                 (expected.is_nan() && actual.is_nan()),
                                 "Float values should be approximately equal: {} vs {}", expected, actual);
                }
                (expected, actual) => {
                    prop_assert_eq!(expected, actual, "Property '{}' should match", key);
                }
            }
        }

        // Verify children
        for i in 0..num_children {
            let child = tree.get_child(i).unwrap();
            let restored_child = restored.get_child(i).unwrap();
            prop_assert_eq!(child.type_name(), restored_child.type_name());
        }
    }

    /// **Feature: juce-modules-integration, Property 5: ValueTree serialization round-trip**
    /// **Validates: Requirements 16.3**
    ///
    /// For any ValueTree structure, binary serialization should round-trip perfectly.
    #[test]
    fn prop_valuetree_binary_serialization_roundtrip(
        type_name in "[a-zA-Z][a-zA-Z0-9_]{0,19}",
        properties in prop::collection::hash_map("[a-zA-Z][a-zA-Z0-9_]{0,9}", value_strategy(), 0..10),
        num_children in 0usize..3,
    ) {
        // Create a ValueTree with properties
        let mut tree = ValueTree::new(&type_name);
        for (key, value) in properties.iter() {
            tree.set_property(key, value.clone());
        }

        // Add some children (limited depth to avoid stack overflow)
        for i in 0..num_children {
            let mut child = ValueTree::new(&format!("child_{}", i));
            child.set_property("index", Value::Int(i as i32));
            child.set_property("name", Value::String(format!("child_{}", i)));
            tree.add_child(child);
        }

        // Serialize to binary
        let binary = tree.to_binary()
            .expect("Binary serialization should succeed");

        // Deserialize from binary
        let restored = ValueTree::from_binary(&binary)
            .expect("Binary deserialization should succeed");

        // Verify structure
        prop_assert_eq!(tree.type_name(), restored.type_name());
        prop_assert_eq!(tree.num_children(), restored.num_children());

        // Verify properties - binary should preserve exact values
        for (key, value) in properties.iter() {
            let restored_value = restored.get_property(key);
            prop_assert!(restored_value.is_some(), "Property '{}' should exist", key);
            prop_assert_eq!(value, restored_value.unwrap(), "Property '{}' should match exactly", key);
        }

        // Verify children
        for i in 0..num_children {
            let child = tree.get_child(i).unwrap();
            let restored_child = restored.get_child(i).unwrap();
            prop_assert_eq!(child.type_name(), restored_child.type_name());
            
            // Verify child properties
            prop_assert_eq!(
                child.get_property("index"),
                restored_child.get_property("index")
            );
            prop_assert_eq!(
                child.get_property("name"),
                restored_child.get_property("name")
            );
        }
    }

    /// **Feature: juce-modules-integration, Property 10: ValueTree modifications trigger notifications**
    /// **Validates: Requirements 16.2**
    ///
    /// For any ValueTree with attached listeners, any modification should result in
    /// the appropriate change notification being sent to all listeners.
    #[test]
    fn prop_valuetree_modifications_trigger_notifications(
        type_name in "[a-zA-Z]{1,20}",
        property_name in "[a-zA-Z]{1,10}",
        int_value in -1000i32..1000,
        num_children in 0usize..5,
    ) {
        // Create a ValueTree with a listener
        let mut tree = ValueTree::new(&type_name);
        let listener = RecordingListener::new();
        let listener_clone = listener.clone();
        tree.add_listener(Box::new(listener_clone));

        // Test 1: Setting a property should trigger value_changed notification
        let initial_count = listener.value_change_count();
        tree.set_property(&property_name, Value::Int(int_value));
        prop_assert_eq!(listener.value_change_count(), initial_count + 1);

        // Test 2: Setting the same property again should trigger another notification
        tree.set_property(&property_name, Value::Int(int_value + 1));
        prop_assert_eq!(listener.value_change_count(), initial_count + 2);

        // Test 3: Adding children should trigger child_added notifications
        let initial_add_count = listener.child_addition_count();
        for i in 0..num_children {
            let child = ValueTree::new(&format!("child_{}", i));
            tree.add_child(child);
        }
        prop_assert_eq!(listener.child_addition_count(), initial_add_count + num_children);

        // Test 4: Removing children should trigger child_removed notifications
        let initial_remove_count = listener.child_removal_count();
        let children_to_remove = tree.num_children().min(2);
        for _ in 0..children_to_remove {
            tree.remove_child(0);
        }
        prop_assert_eq!(listener.child_removal_count(), initial_remove_count + children_to_remove);
    }

    /// Test that multiple property changes trigger multiple notifications.
    #[test]
    fn prop_multiple_property_changes_trigger_multiple_notifications(
        type_name in "[a-zA-Z]{1,20}",
        properties in prop::collection::hash_map("[a-zA-Z]{1,10}", 0i32..1000, 1..10)
    ) {
        let mut tree = ValueTree::new(&type_name);
        let listener = RecordingListener::new();
        let listener_clone = listener.clone();
        tree.add_listener(Box::new(listener_clone));

        let initial_count = listener.value_change_count();
        
        // Set each property
        for (key, value) in properties.iter() {
            tree.set_property(key, Value::Int(*value));
        }

        // Should have one notification per property set
        prop_assert_eq!(listener.value_change_count(), initial_count + properties.len());
    }

    /// Test that child operations trigger the correct number of notifications.
    #[test]
    fn prop_child_operations_trigger_correct_notifications(
        parent_name in "[a-zA-Z]{1,20}",
        num_children in 1usize..10,
    ) {
        let mut parent = ValueTree::new(&parent_name);
        let listener = RecordingListener::new();
        let listener_clone = listener.clone();
        parent.add_listener(Box::new(listener_clone));

        // Add children
        for i in 0..num_children {
            let child = ValueTree::new(&format!("child_{}", i));
            parent.add_child(child);
        }

        prop_assert_eq!(listener.child_addition_count(), num_children);
        prop_assert_eq!(listener.child_removal_count(), 0);

        // Remove all children
        for _ in 0..num_children {
            parent.remove_child(0);
        }

        prop_assert_eq!(listener.child_addition_count(), num_children);
        prop_assert_eq!(listener.child_removal_count(), num_children);
    }
}
