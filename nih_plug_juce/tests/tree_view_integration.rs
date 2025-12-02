//! Integration tests for TreeView component.
//!
//! These tests verify that the TreeView FFI bridge works correctly,
//! including creating tree views, setting root items, and handling
//! tree item callbacks.

use nih_plug_juce::containers::{TreeView, TreeViewItem};
use nih_plug_juce::graphics::Graphics;
use nih_plug_juce::Result;

/// Simple tree item for testing
#[derive(Clone)]
struct TestTreeItem {
    name: String,
    children: Vec<TestTreeItem>,
}

impl TestTreeItem {
    fn new(name: &str) -> Self {
        TestTreeItem {
            name: name.to_string(),
            children: Vec::new(),
        }
    }
    
    fn with_children(name: &str, children: Vec<TestTreeItem>) -> Self {
        TestTreeItem {
            name: name.to_string(),
            children,
        }
    }
}

impl TreeViewItem for TestTreeItem {
    fn get_num_sub_items(&self) -> i32 {
        self.children.len() as i32
    }
    
    fn get_sub_item(&self, index: i32) -> Option<Box<dyn TreeViewItem>> {
        self.children.get(index as usize).map(|child| {
            Box::new(TestTreeItem {
                name: child.name.clone(),
                children: child.children.clone(),
            }) as Box<dyn TreeViewItem>
        })
    }
    
    fn paint_item(&self, _g: &mut Graphics, _width: i32, _height: i32) {
        // Simple paint implementation for testing
    }
    
    fn item_clicked(&mut self) {
        // Handle click for testing
    }
}

#[test]
fn test_tree_view_creation() -> Result<()> {
    // Test that we can create a TreeView
    let _tree_view = TreeView::new()?;
    
    // If we got here without panicking, the tree view was created successfully
    
    Ok(())
}

#[test]
fn test_tree_view_set_root_item() -> Result<()> {
    // Create a tree view
    let mut tree_view = TreeView::new()?;
    
    // Create a simple root item
    let root = Box::new(TestTreeItem::new("Root"));
    
    // Set the root item
    tree_view.set_root_item(root)?;
    
    Ok(())
}

#[test]
fn test_tree_view_with_children() -> Result<()> {
    // Create a tree view
    let mut tree_view = TreeView::new()?;
    
    // Create a tree structure
    let root = Box::new(TestTreeItem::with_children(
        "Root",
        vec![
            TestTreeItem::new("Child 1"),
            TestTreeItem::new("Child 2"),
            TestTreeItem::with_children(
                "Child 3",
                vec![
                    TestTreeItem::new("Grandchild 1"),
                    TestTreeItem::new("Grandchild 2"),
                ],
            ),
        ],
    ));
    
    // Set the root item
    tree_view.set_root_item(root)?;
    
    Ok(())
}

#[test]
fn test_tree_view_component_operations() -> Result<()> {
    // Create a tree view
    let mut tree_view = TreeView::new()?;
    
    // Test that we can use Component methods through Deref
    tree_view.set_bounds(0, 0, 300, 400);
    tree_view.set_visible(true);
    
    // Create and set a root item
    let root = Box::new(TestTreeItem::new("Root"));
    tree_view.set_root_item(root)?;
    
    Ok(())
}

#[test]
fn test_tree_view_empty_root() -> Result<()> {
    // Create a tree view
    let mut tree_view = TreeView::new()?;
    
    // Create a root item with no children
    let root = Box::new(TestTreeItem::new("Empty Root"));
    
    // Set the root item
    tree_view.set_root_item(root)?;
    
    Ok(())
}
