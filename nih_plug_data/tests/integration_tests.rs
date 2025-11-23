//! Integration tests for ValueTree functionality.

use nih_plug_data::{Value, ValueTree, ValueTreeListener};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct CountingListener {
    count: Arc<Mutex<usize>>,
}

impl CountingListener {
    fn new() -> Self {
        Self {
            count: Arc::new(Mutex::new(0)),
        }
    }

    fn get_count(&self) -> usize {
        *self.count.lock().unwrap()
    }
}

impl ValueTreeListener for CountingListener {
    fn value_changed(&mut self, _tree: &ValueTree, _property: &str) {
        *self.count.lock().unwrap() += 1;
    }

    fn child_added(&mut self, _parent: &ValueTree, _child: &ValueTree) {
        *self.count.lock().unwrap() += 1;
    }

    fn child_removed(&mut self, _parent: &ValueTree, _child: &ValueTree) {
        *self.count.lock().unwrap() += 1;
    }
}

#[test]
fn test_valuetree_basic_operations() {
    let mut tree = ValueTree::new("root");
    
    // Test property management
    tree.set_property("name", Value::String("test".to_string()));
    tree.set_property("count", Value::Int(42));
    tree.set_property("enabled", Value::Bool(true));
    tree.set_property("ratio", Value::Float(0.5));
    
    assert_eq!(tree.get_property("name"), Some(&Value::String("test".to_string())));
    assert_eq!(tree.get_property("count"), Some(&Value::Int(42)));
    assert_eq!(tree.get_property("enabled"), Some(&Value::Bool(true)));
    assert_eq!(tree.get_property("ratio"), Some(&Value::Float(0.5)));
    
    // Test hierarchical structure
    let child1 = ValueTree::new("child1");
    let child2 = ValueTree::new("child2");
    
    tree.add_child(child1);
    tree.add_child(child2);
    
    assert_eq!(tree.num_children(), 2);
    assert_eq!(tree.get_child(0).unwrap().type_name(), "child1");
    assert_eq!(tree.get_child(1).unwrap().type_name(), "child2");
    
    // Test child removal
    let removed = tree.remove_child(0);
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().type_name(), "child1");
    assert_eq!(tree.num_children(), 1);
}

#[test]
fn test_valuetree_listener_notifications() {
    let mut tree = ValueTree::new("root");
    let listener = CountingListener::new();
    let listener_clone = listener.clone();
    
    tree.add_listener(Box::new(listener_clone));
    
    // Property changes should trigger notifications
    tree.set_property("prop1", Value::Int(1));
    assert_eq!(listener.get_count(), 1);
    
    tree.set_property("prop2", Value::Int(2));
    assert_eq!(listener.get_count(), 2);
    
    // Child additions should trigger notifications
    tree.add_child(ValueTree::new("child1"));
    assert_eq!(listener.get_count(), 3);
    
    tree.add_child(ValueTree::new("child2"));
    assert_eq!(listener.get_count(), 4);
    
    // Child removals should trigger notifications
    tree.remove_child(0);
    assert_eq!(listener.get_count(), 5);
}

#[test]
#[cfg(feature = "valuetree")]
fn test_valuetree_xml_serialization() {
    let mut tree = ValueTree::new("root");
    tree.set_property("name", Value::String("test".to_string()));
    tree.set_property("count", Value::Int(42));
    
    let mut child = ValueTree::new("child");
    child.set_property("enabled", Value::Bool(true));
    tree.add_child(child);
    
    // Serialize to XML
    let xml = tree.to_xml();
    assert!(xml.contains("root"));
    assert!(xml.contains("child"));
    
    // Deserialize from XML
    let restored = ValueTree::from_xml(&xml).unwrap();
    assert_eq!(restored.type_name(), "root");
    assert_eq!(restored.num_children(), 1);
    
    // Note: Properties are restored but listeners are not serialized
    let name_prop = restored.get_property("name");
    assert!(name_prop.is_some());
}

#[test]
fn test_valuetree_clone() {
    let mut tree = ValueTree::new("root");
    tree.set_property("value", Value::Int(42));
    tree.add_child(ValueTree::new("child"));
    
    let cloned = tree.clone();
    
    assert_eq!(cloned.type_name(), tree.type_name());
    assert_eq!(cloned.get_property("value"), tree.get_property("value"));
    assert_eq!(cloned.num_children(), tree.num_children());
}

#[test]
fn test_valuetree_binary_serialization() {
    let mut tree = ValueTree::new("root");
    tree.set_property("name", Value::String("test".to_string()));
    tree.set_property("count", Value::Int(42));
    tree.set_property("ratio", Value::Float(0.5));
    tree.set_property("enabled", Value::Bool(true));
    
    let mut child = ValueTree::new("child");
    child.set_property("id", Value::Int(1));
    tree.add_child(child);
    
    // Serialize to binary
    let binary = tree.to_binary().unwrap();
    assert!(!binary.is_empty());
    
    // Deserialize from binary
    let restored = ValueTree::from_binary(&binary).unwrap();
    assert_eq!(restored.type_name(), "root");
    assert_eq!(restored.num_children(), 1);
    
    // Verify all properties are restored exactly
    assert_eq!(restored.get_property("name"), Some(&Value::String("test".to_string())));
    assert_eq!(restored.get_property("count"), Some(&Value::Int(42)));
    assert_eq!(restored.get_property("ratio"), Some(&Value::Float(0.5)));
    assert_eq!(restored.get_property("enabled"), Some(&Value::Bool(true)));
    
    // Verify child
    let restored_child = restored.get_child(0).unwrap();
    assert_eq!(restored_child.type_name(), "child");
    assert_eq!(restored_child.get_property("id"), Some(&Value::Int(1)));
}

// UndoManager tests
#[cfg(feature = "undo")]
mod undo_tests {
    use nih_plug_data::{UndoManager, UndoableAction};
    use std::sync::{Arc, Mutex};

    struct SetValueAction {
        value: i32,
        old_value: i32,
        target: Arc<Mutex<i32>>,
    }

    impl UndoableAction for SetValueAction {
        fn perform(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            *self.target.lock().unwrap() = self.value;
            Ok(())
        }

        fn undo(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            *self.target.lock().unwrap() = self.old_value;
            Ok(())
        }
    }

    #[test]
    fn test_undo_manager_basic_operations() {
        let mut manager = UndoManager::new();
        let target = Arc::new(Mutex::new(0));

        assert!(!manager.can_undo());
        assert!(!manager.can_redo());

        // Perform an action
        let action = Box::new(SetValueAction {
            value: 42,
            old_value: 0,
            target: target.clone(),
        });
        manager.perform(action).unwrap();

        assert_eq!(*target.lock().unwrap(), 42);
        assert!(manager.can_undo());
        assert!(!manager.can_redo());

        // Undo the action
        manager.undo().unwrap();
        assert_eq!(*target.lock().unwrap(), 0);
        assert!(!manager.can_undo());
        assert!(manager.can_redo());

        // Redo the action
        manager.redo().unwrap();
        assert_eq!(*target.lock().unwrap(), 42);
        assert!(manager.can_undo());
        assert!(!manager.can_redo());
    }

    #[test]
    fn test_undo_manager_multiple_actions() {
        let mut manager = UndoManager::new();
        let target = Arc::new(Mutex::new(0));

        // Perform multiple actions
        for i in 1..=5 {
            let action = Box::new(SetValueAction {
                value: i * 10,
                old_value: (i - 1) * 10,
                target: target.clone(),
            });
            manager.perform(action).unwrap();
        }

        assert_eq!(*target.lock().unwrap(), 50);

        // Undo all actions
        for i in (0..5).rev() {
            manager.undo().unwrap();
            assert_eq!(*target.lock().unwrap(), i * 10);
        }

        assert!(!manager.can_undo());
        assert!(manager.can_redo());

        // Redo all actions
        for i in 1..=5 {
            manager.redo().unwrap();
            assert_eq!(*target.lock().unwrap(), i * 10);
        }

        assert!(manager.can_undo());
        assert!(!manager.can_redo());
    }

    #[test]
    fn test_undo_manager_new_action_clears_redo() {
        let mut manager = UndoManager::new();
        let target = Arc::new(Mutex::new(0));

        // Perform and undo an action
        let action1 = Box::new(SetValueAction {
            value: 10,
            old_value: 0,
            target: target.clone(),
        });
        manager.perform(action1).unwrap();
        manager.undo().unwrap();

        assert!(manager.can_redo());

        // Perform a new action - should clear redo stack
        let action2 = Box::new(SetValueAction {
            value: 20,
            old_value: 0,
            target: target.clone(),
        });
        manager.perform(action2).unwrap();

        assert!(!manager.can_redo());
        assert_eq!(*target.lock().unwrap(), 20);
    }

    #[test]
    fn test_undo_manager_transactions() {
        let mut manager = UndoManager::new();
        let target = Arc::new(Mutex::new(0));

        // Begin a transaction
        manager.begin_transaction();

        // Perform multiple actions within the transaction
        for i in 1..=3 {
            let action = Box::new(SetValueAction {
                value: i * 10,
                old_value: (i - 1) * 10,
                target: target.clone(),
            });
            manager.perform(action).unwrap();
        }

        // End the transaction
        manager.end_transaction();

        assert_eq!(*target.lock().unwrap(), 30);
        assert!(manager.can_undo());

        // Undo should revert all actions in the transaction
        manager.undo().unwrap();
        assert_eq!(*target.lock().unwrap(), 0);
        assert!(!manager.can_undo());
        assert!(manager.can_redo());

        // Redo should replay all actions in the transaction
        manager.redo().unwrap();
        assert_eq!(*target.lock().unwrap(), 30);
    }

    #[test]
    fn test_undo_manager_nested_transactions() {
        let mut manager = UndoManager::new();
        let target = Arc::new(Mutex::new(0));

        // Begin outer transaction
        manager.begin_transaction();

        let action1 = Box::new(SetValueAction {
            value: 10,
            old_value: 0,
            target: target.clone(),
        });
        manager.perform(action1).unwrap();

        // Begin inner transaction
        manager.begin_transaction();

        let action2 = Box::new(SetValueAction {
            value: 20,
            old_value: 10,
            target: target.clone(),
        });
        manager.perform(action2).unwrap();

        // End inner transaction
        manager.end_transaction();

        let action3 = Box::new(SetValueAction {
            value: 30,
            old_value: 20,
            target: target.clone(),
        });
        manager.perform(action3).unwrap();

        // End outer transaction
        manager.end_transaction();

        assert_eq!(*target.lock().unwrap(), 30);

        // Undo should revert the outer transaction (which includes the inner one)
        manager.undo().unwrap();
        assert_eq!(*target.lock().unwrap(), 0);
    }

    #[test]
    fn test_undo_manager_clear() {
        let mut manager = UndoManager::new();
        let target = Arc::new(Mutex::new(0));

        // Perform some actions
        for i in 1..=3 {
            let action = Box::new(SetValueAction {
                value: i * 10,
                old_value: (i - 1) * 10,
                target: target.clone(),
            });
            manager.perform(action).unwrap();
        }

        manager.undo().unwrap();

        assert!(manager.can_undo());
        assert!(manager.can_redo());

        // Clear should remove all history
        manager.clear();

        assert!(!manager.can_undo());
        assert!(!manager.can_redo());
    }

    #[test]
    fn test_undo_manager_error_handling() {
        let mut manager = UndoManager::new();

        // Try to undo when there's nothing to undo
        assert!(manager.undo().is_err());

        // Try to redo when there's nothing to redo
        assert!(manager.redo().is_err());
    }

    struct FailingAction {
        should_fail_perform: bool,
        should_fail_undo: bool,
    }

    impl UndoableAction for FailingAction {
        fn perform(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            if self.should_fail_perform {
                Err("Perform failed".into())
            } else {
                Ok(())
            }
        }

        fn undo(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            if self.should_fail_undo {
                Err("Undo failed".into())
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn test_undo_manager_action_failure() {
        let mut manager = UndoManager::new();

        // Action that fails on perform
        let failing_action = Box::new(FailingAction {
            should_fail_perform: true,
            should_fail_undo: false,
        });

        assert!(manager.perform(failing_action).is_err());
        assert!(!manager.can_undo()); // Failed action should not be added to stack

        // Action that succeeds on perform but fails on undo
        let action = Box::new(FailingAction {
            should_fail_perform: false,
            should_fail_undo: true,
        });

        manager.perform(action).unwrap();
        assert!(manager.can_undo());

        // Undo should fail
        assert!(manager.undo().is_err());
    }
}
