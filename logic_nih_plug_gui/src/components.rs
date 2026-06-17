//! GUI component implementations.
//!
//! This module provides the core component infrastructure for building UIs,
//! including lifecycle management and parent-child relationships.

use crate::error::{GuiError, Result};
use crate::input::{EventResult, KeyboardEvent, MouseEvent};
use std::cell::RefCell;
use std::rc::{Rc, Weak};

/// Unique identifier for components.
pub type ComponentId = usize;

/// Rectangle representing component bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bounds {
    /// X coordinate
    pub x: i32,
    /// Y coordinate
    pub y: i32,
    /// Width
    pub width: u32,
    /// Height
    pub height: u32,
}

impl Bounds {
    /// Create new bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_gui::components::Bounds;
    ///
    /// let bounds = Bounds::new(10, 20, 100, 50);
    /// assert_eq!(bounds.x, 10);
    /// assert_eq!(bounds.y, 20);
    /// assert_eq!(bounds.width, 100);
    /// assert_eq!(bounds.height, 50);
    /// ```
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }

    /// Check if bounds are valid (non-negative dimensions).
    pub fn is_valid(&self) -> bool {
        self.width > 0 && self.height > 0
    }

    /// Check if a point is within these bounds.
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x
            && x < self.x + self.width as i32
            && y >= self.y
            && y < self.y + self.height as i32
    }
}

/// Component lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentState {
    /// Component is being initialized
    Initializing,
    /// Component is ready and visible
    Active,
    /// Component is hidden but still in hierarchy
    Hidden,
    /// Component is being destroyed
    Destroying,
}

/// Internal component data.
struct ComponentData {
    id: ComponentId,
    name: String,
    bounds: Bounds,
    state: ComponentState,
    visible: bool,
    enabled: bool,
    parent: Option<Weak<RefCell<ComponentData>>>,
    children: Vec<Rc<RefCell<ComponentData>>>,
    // Optional per-component handlers for input events (allows controls to register behavior)
    mouse_handler: Option<Box<dyn FnMut(&crate::input::MouseEvent) -> EventResult>>,
    keyboard_handler: Option<Box<dyn FnMut(&crate::input::KeyboardEvent) -> EventResult>>,
}

/// A GUI component that can be displayed and interacted with.
///
/// Components form a tree hierarchy with parent-child relationships.
/// Each component has bounds, visibility, and lifecycle state.
///
/// # Examples
///
/// ```
/// use logic_nih_plug_gui::components::{Component, Bounds};
///
/// let mut parent = Component::new("parent");
/// parent.set_bounds(Bounds::new(0, 0, 400, 300)).unwrap();
///
/// let mut child = Component::new("child");
/// child.set_bounds(Bounds::new(10, 10, 100, 50)).unwrap();
///
/// parent.add_child(child).unwrap();
/// assert_eq!(parent.child_count(), 1);
/// ```
pub struct Component {
    data: Rc<RefCell<ComponentData>>,
}

impl Component {
    /// Create a new component with the given name.
    ///
    /// The component starts in the `Initializing` state with default bounds (0, 0, 0, 0).
    pub fn new(name: &str) -> Self {
        static NEXT_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        Self {
            data: Rc::new(RefCell::new(ComponentData {
                id,
                name: name.to_string(),
                bounds: Bounds::new(0, 0, 0, 0),
                state: ComponentState::Initializing,
                visible: true,
                enabled: true,
                parent: None,
                children: Vec::new(),
                mouse_handler: None,
                keyboard_handler: None,
            })),
        }
    }

    /// Get the component's unique ID.
    pub fn id(&self) -> ComponentId {
        self.data.borrow().id
    }

    /// Get the component's name.
    pub fn name(&self) -> String {
        self.data.borrow().name.clone()
    }

    /// Set the component's name.
    pub fn set_name(&mut self, name: &str) {
        self.data.borrow_mut().name = name.to_string();
    }

    /// Get the component's bounds.
    pub fn bounds(&self) -> Bounds {
        self.data.borrow().bounds
    }

    /// Set the component's bounds.
    ///
    /// Returns an error if the bounds are invalid (zero or negative dimensions).
    pub fn set_bounds(&mut self, bounds: Bounds) -> Result<()> {
        if !bounds.is_valid() {
            return Err(GuiError::InvalidBounds(
                bounds.x,
                bounds.y,
                bounds.width,
                bounds.height,
            ));
        }
        self.data.borrow_mut().bounds = bounds;
        Ok(())
    }

    /// Get the component's lifecycle state.
    pub fn state(&self) -> ComponentState {
        self.data.borrow().state
    }

    /// Set the component's lifecycle state.
    pub fn set_state(&mut self, state: ComponentState) {
        self.data.borrow_mut().state = state;
    }

    /// Check if the component is visible.
    pub fn is_visible(&self) -> bool {
        self.data.borrow().visible
    }

    /// Set the component's visibility.
    pub fn set_visible(&mut self, visible: bool) {
        self.data.borrow_mut().visible = visible;
    }

    /// Check if the component is enabled.
    pub fn is_enabled(&self) -> bool {
        self.data.borrow().enabled
    }

    /// Set whether the component is enabled.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.data.borrow_mut().enabled = enabled;
    }

    /// Check if this component has a parent.
    pub fn has_parent(&self) -> bool {
        self.data.borrow().parent.is_some()
    }

    /// Get the number of child components.
    pub fn child_count(&self) -> usize {
        self.data.borrow().children.len()
    }

    /// Add a child component.
    ///
    /// Returns an error if:
    /// - The child already has a parent
    /// - The child is the same as this component (self-reference)
    pub fn add_child(&mut self, child: Component) -> Result<()> {
        // Check for self-reference
        if self.id() == child.id() {
            return Err(GuiError::SelfReference);
        }

        // Check if child already has a parent
        if child.has_parent() {
            return Err(GuiError::AlreadyHasParent);
        }

        // Set parent reference in child
        child.data.borrow_mut().parent = Some(Rc::downgrade(&self.data));

        // Add child to our children list
        self.data.borrow_mut().children.push(child.data.clone());

        Ok(())
    }

    /// Remove a child component by index.
    ///
    /// Returns the removed component, or None if the index is out of bounds.
    pub fn remove_child(&mut self, index: usize) -> Option<Component> {
        let mut data = self.data.borrow_mut();
        if index >= data.children.len() {
            return None;
        }

        let child_data = data.children.remove(index);
        child_data.borrow_mut().parent = None;

        Some(Component { data: child_data })
    }

    /// Remove all child components.
    pub fn remove_all_children(&mut self) {
        let mut data = self.data.borrow_mut();
        for child in &data.children {
            child.borrow_mut().parent = None;
        }
        data.children.clear();
    }

    /// Get a child component by index.
    pub fn child(&self, index: usize) -> Option<Component> {
        let data = self.data.borrow();
        data.children.get(index).map(|child_data| Component {
            data: child_data.clone(),
        })
    }

    /// Find a child component by name.
    pub fn find_child_by_name(&self, name: &str) -> Option<Component> {
        let data = self.data.borrow();
        for child_data in &data.children {
            if child_data.borrow().name == name {
                return Some(Component {
                    data: child_data.clone(),
                });
            }
        }
        None
    }

    /// Check if a point is within this component's bounds.
    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        self.data.borrow().bounds.contains(x, y)
    }

    /// Initialize the component (transition from Initializing to Active).
    pub fn initialize(&mut self) {
        let mut data = self.data.borrow_mut();
        if data.state == ComponentState::Initializing {
            data.state = ComponentState::Active;
        }
    }

    /// Destroy the component (transition to Destroying state and remove from parent).
    pub fn destroy(&mut self) {
        // First, get the parent reference and our ID before borrowing mutably
        let (parent_weak, component_id) = {
            let data = self.data.borrow();
            (data.parent.clone(), data.id)
        };

        // Remove from parent if we have one (do this before mutating our own data)
        if let Some(parent_weak) = parent_weak {
            if let Some(parent_rc) = parent_weak.upgrade() {
                let mut parent_data = parent_rc.borrow_mut();
                parent_data.children.retain(|child| {
                    child.borrow().id != component_id
                });
            }
        }

        // Now mutate our own data
        let mut data = self.data.borrow_mut();
        data.state = ComponentState::Destroying;
        data.parent = None;

        // Clear all children
        for child in &data.children {
            child.borrow_mut().parent = None;
        }
        data.children.clear();
    }

    /// Handle a mouse event on this component.
    ///
    /// This method can be overridden by subclasses to provide custom mouse handling.
    /// The default implementation does nothing and returns `EventResult::NotHandled`.
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_gui::components::Component;
    /// use logic_nih_plug_gui::input::{MouseEvent, MouseButton, Modifiers, EventResult};
    ///
    /// let mut component = Component::new("test");
    /// let event = MouseEvent::ButtonDown {
    ///     x: 10,
    ///     y: 20,
    ///     button: MouseButton::Left,
    ///     modifiers: Modifiers::none(),
    /// };
    /// let result = component.handle_mouse_event(&event);
    /// assert_eq!(result, EventResult::NotHandled);
    /// ```
    pub fn handle_mouse_event(&mut self, event: &MouseEvent) -> EventResult {
        // If a handler closure has been registered, call it
        if let Some(handler) = self.data.borrow_mut().mouse_handler.as_mut() {
            return handler(event);
        }

        // Default: not handled
        EventResult::NotHandled
    }

    /// Handle a keyboard event on this component.
    ///
    /// This method can be overridden by subclasses to provide custom keyboard handling.
    /// The default implementation does nothing and returns `EventResult::NotHandled`.
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_gui::components::Component;
    /// use logic_nih_plug_gui::input::{KeyboardEvent, KeyCode, Modifiers, EventResult};
    ///
    /// let mut component = Component::new("test");
    /// let event = KeyboardEvent::KeyDown {
    ///     key: KeyCode::A,
    ///     character: Some('a'),
    ///     modifiers: Modifiers::none(),
    ///     repeat: false,
    /// };
    /// let result = component.handle_keyboard_event(&event);
    /// assert_eq!(result, EventResult::NotHandled);
    /// ```
    pub fn handle_keyboard_event(&mut self, event: &KeyboardEvent) -> EventResult {
        if let Some(handler) = self.data.borrow_mut().keyboard_handler.as_mut() {
            return handler(event);
        }

        EventResult::NotHandled
    }

    /// Register a mouse event handler closure for this component.
    pub fn set_mouse_handler<F>(&mut self, f: F)
    where
        F: FnMut(&MouseEvent) -> EventResult + 'static,
    {
        self.data.borrow_mut().mouse_handler = Some(Box::new(f));
    }

    /// Remove the component's mouse handler.
    pub fn clear_mouse_handler(&mut self) {
        self.data.borrow_mut().mouse_handler = None;
    }

    /// Register a keyboard event handler closure for this component.
    pub fn set_keyboard_handler<F>(&mut self, f: F)
    where
        F: FnMut(&KeyboardEvent) -> EventResult + 'static,
    {
        self.data.borrow_mut().keyboard_handler = Some(Box::new(f));
    }

    /// Remove the component's keyboard handler.
    pub fn clear_keyboard_handler(&mut self) {
        self.data.borrow_mut().keyboard_handler = None;
    }

    /// Dispatch a mouse event to this component and its children.
    ///
    /// This performs hit testing and routes the event to the appropriate child component.
    /// If no child handles the event, it's passed to this component.
    ///
    /// Returns `EventResult::Handled` if any component handled the event.
    pub fn dispatch_mouse_event(&mut self, event: &MouseEvent) -> EventResult {
        // Get the event position
        let (x, y) = event.position();

        // Check if the event is within our bounds
        if !self.contains_point(x, y) {
            return EventResult::NotHandled;
        }

        // Try to dispatch to children first (in reverse order, top to bottom)
        let child_count = self.child_count();
        for i in (0..child_count).rev() {
            if let Some(mut child) = self.child(i) {
                if child.is_visible() && child.is_enabled() {
                    let result = child.dispatch_mouse_event(event);
                    if result == EventResult::Handled {
                        return EventResult::Handled;
                    }
                }
            }
        }

        // If no child handled it, try to handle it ourselves
        self.handle_mouse_event(event)
    }

    /// Dispatch a keyboard event to this component.
    ///
    /// This passes the event to this component's keyboard handler.
    /// Subclasses can override `handle_keyboard_event` to provide custom behavior.
    ///
    /// Returns `EventResult::Handled` if the component handled the event.
    pub fn dispatch_keyboard_event(&mut self, event: &KeyboardEvent) -> EventResult {
        if !self.is_enabled() {
            return EventResult::NotHandled;
        }

        self.handle_keyboard_event(event)
    }
}

impl Clone for Component {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

impl std::fmt::Debug for Component {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = self.data.borrow();
        f.debug_struct("Component")
            .field("id", &data.id)
            .field("name", &data.name)
            .field("bounds", &data.bounds)
            .field("state", &data.state)
            .field("visible", &data.visible)
            .field("enabled", &data.enabled)
            .field("child_count", &data.children.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_creation() {
        let component = Component::new("test");
        assert_eq!(component.name(), "test");
        assert_eq!(component.state(), ComponentState::Initializing);
        assert!(component.is_visible());
        assert!(component.is_enabled());
        assert!(!component.has_parent());
        assert_eq!(component.child_count(), 0);
    }

    #[test]
    fn test_bounds() {
        let mut component = Component::new("test");
        let bounds = Bounds::new(10, 20, 100, 50);
        component.set_bounds(bounds).unwrap();
        assert_eq!(component.bounds(), bounds);
    }

    #[test]
    fn test_invalid_bounds() {
        let mut component = Component::new("test");
        let result = component.set_bounds(Bounds::new(0, 0, 0, 0));
        assert!(result.is_err());
    }

    #[test]
    fn test_parent_child_relationship() {
        let mut parent = Component::new("parent");
        let child = Component::new("child");

        parent.add_child(child.clone()).unwrap();
        assert_eq!(parent.child_count(), 1);
        assert!(child.has_parent());
    }

    #[test]
    fn test_cannot_add_child_twice() {
        let mut parent1 = Component::new("parent1");
        let mut parent2 = Component::new("parent2");
        let child = Component::new("child");

        parent1.add_child(child.clone()).unwrap();
        let result = parent2.add_child(child);
        assert!(result.is_err());
    }

    #[test]
    fn test_self_reference() {
        let mut component = Component::new("test");
        let result = component.add_child(component.clone());
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_child() {
        let mut parent = Component::new("parent");
        let child = Component::new("child");

        parent.add_child(child.clone()).unwrap();
        assert_eq!(parent.child_count(), 1);

        let removed = parent.remove_child(0);
        assert!(removed.is_some());
        assert_eq!(parent.child_count(), 0);
        assert!(!child.has_parent());
    }

    #[test]
    fn test_lifecycle() {
        let mut component = Component::new("test");
        assert_eq!(component.state(), ComponentState::Initializing);

        component.initialize();
        assert_eq!(component.state(), ComponentState::Active);

        component.destroy();
        assert_eq!(component.state(), ComponentState::Destroying);
    }

    #[test]
    fn test_find_child_by_name() {
        let mut parent = Component::new("parent");
        let child1 = Component::new("child1");
        let child2 = Component::new("child2");

        parent.add_child(child1).unwrap();
        parent.add_child(child2).unwrap();

        let found = parent.find_child_by_name("child2");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name(), "child2");

        let not_found = parent.find_child_by_name("child3");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_contains_point() {
        let mut component = Component::new("test");
        component.set_bounds(Bounds::new(10, 10, 100, 50)).unwrap();

        assert!(component.contains_point(50, 30));
        assert!(!component.contains_point(5, 5));
        assert!(!component.contains_point(150, 30));
    }

    #[test]
    fn test_mouse_event_handling() {
        use crate::input::{EventResult, MouseButton, MouseEvent, Modifiers};

        let mut component = Component::new("test");
        component.set_bounds(Bounds::new(0, 0, 100, 100)).unwrap();

        let event = MouseEvent::ButtonDown {
            x: 50,
            y: 50,
            button: MouseButton::Left,
            modifiers: Modifiers::none(),
        };

        // Default implementation returns NotHandled
        let result = component.handle_mouse_event(&event);
        assert_eq!(result, EventResult::NotHandled);
    }

    #[test]
    fn test_keyboard_event_handling() {
        use crate::input::{EventResult, KeyCode, KeyboardEvent, Modifiers};

        let mut component = Component::new("test");

        let event = KeyboardEvent::KeyDown {
            key: KeyCode::A,
            character: Some('a'),
            modifiers: Modifiers::none(),
            repeat: false,
        };

        // Default implementation returns NotHandled
        let result = component.handle_keyboard_event(&event);
        assert_eq!(result, EventResult::NotHandled);
    }

    #[test]
    fn test_dispatch_mouse_event_out_of_bounds() {
        use crate::input::{MouseButton, MouseEvent, Modifiers};

        let mut component = Component::new("test");
        component.set_bounds(Bounds::new(0, 0, 100, 100)).unwrap();

        // Event outside bounds
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
    fn test_dispatch_keyboard_event_disabled() {
        use crate::input::{KeyCode, KeyboardEvent, Modifiers};

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
}
