//! Input event handling for GUI components.
//!
//! This module provides types and traits for handling mouse and keyboard input events.

use crate::components::ComponentId;

/// Mouse button identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    /// Left mouse button
    Left,
    /// Right mouse button
    Right,
    /// Middle mouse button
    Middle,
    /// Additional mouse button (e.g., back button)
    Other(u8),
}

/// Keyboard modifier keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Modifiers {
    /// Shift key is pressed
    pub shift: bool,
    /// Control key is pressed (Command on macOS)
    pub ctrl: bool,
    /// Alt/Option key is pressed
    pub alt: bool,
    /// Meta/Windows/Command key is pressed
    pub meta: bool,
}

impl Modifiers {
    /// Create modifiers with no keys pressed.
    pub fn none() -> Self {
        Self {
            shift: false,
            ctrl: false,
            alt: false,
            meta: false,
        }
    }

    /// Check if any modifier key is pressed.
    pub fn any(&self) -> bool {
        self.shift || self.ctrl || self.alt || self.meta
    }
}

impl Default for Modifiers {
    fn default() -> Self {
        Self::none()
    }
}

/// Mouse event types.
#[derive(Debug, Clone)]
pub enum MouseEvent {
    /// Mouse button was pressed.
    ButtonDown {
        /// X coordinate relative to component
        x: i32,
        /// Y coordinate relative to component
        y: i32,
        /// Which button was pressed
        button: MouseButton,
        /// Modifier keys held during event
        modifiers: Modifiers,
    },
    /// Mouse button was released.
    ButtonUp {
        /// X coordinate relative to component
        x: i32,
        /// Y coordinate relative to component
        y: i32,
        /// Which button was released
        button: MouseButton,
        /// Modifier keys held during event
        modifiers: Modifiers,
    },
    /// Mouse was moved.
    Move {
        /// X coordinate relative to component
        x: i32,
        /// Y coordinate relative to component
        y: i32,
        /// Modifier keys held during event
        modifiers: Modifiers,
    },
    /// Mouse was dragged (moved while button held).
    Drag {
        /// X coordinate relative to component
        x: i32,
        /// Y coordinate relative to component
        y: i32,
        /// Which button is held
        button: MouseButton,
        /// Modifier keys held during event
        modifiers: Modifiers,
    },
    /// Mouse entered the component bounds.
    Enter {
        /// X coordinate relative to component
        x: i32,
        /// Y coordinate relative to component
        y: i32,
    },
    /// Mouse left the component bounds.
    Exit {
        /// X coordinate relative to component
        x: i32,
        /// Y coordinate relative to component
        y: i32,
    },
    /// Mouse wheel was scrolled.
    Wheel {
        /// X coordinate relative to component
        x: i32,
        /// Y coordinate relative to component
        y: i32,
        /// Horizontal scroll delta
        delta_x: f32,
        /// Vertical scroll delta
        delta_y: f32,
        /// Modifier keys held during event
        modifiers: Modifiers,
    },
}

impl MouseEvent {
    /// Get the position of the mouse event.
    pub fn position(&self) -> (i32, i32) {
        match self {
            MouseEvent::ButtonDown { x, y, .. }
            | MouseEvent::ButtonUp { x, y, .. }
            | MouseEvent::Move { x, y, .. }
            | MouseEvent::Drag { x, y, .. }
            | MouseEvent::Enter { x, y }
            | MouseEvent::Exit { x, y }
            | MouseEvent::Wheel { x, y, .. } => (*x, *y),
        }
    }

    /// Get the modifiers for the mouse event, if applicable.
    pub fn modifiers(&self) -> Option<Modifiers> {
        match self {
            MouseEvent::ButtonDown { modifiers, .. }
            | MouseEvent::ButtonUp { modifiers, .. }
            | MouseEvent::Move { modifiers, .. }
            | MouseEvent::Drag { modifiers, .. }
            | MouseEvent::Wheel { modifiers, .. } => Some(*modifiers),
            MouseEvent::Enter { .. } | MouseEvent::Exit { .. } => None,
        }
    }
}

/// Virtual key codes for keyboard events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum KeyCode {
    /// Letter keys A-Z
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    
    /// Number keys 0-9
    Num0, Num1, Num2, Num3, Num4,
    Num5, Num6, Num7, Num8, Num9,
    
    /// Function keys
    F1, F2, F3, F4, F5, F6,
    F7, F8, F9, F10, F11, F12,
    
    /// Arrow keys
    Left, Right, Up, Down,
    
    /// Special keys
    Escape,
    Tab,
    Backspace,
    Return,
    Space,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    Insert,
    
    /// Modifier keys
    Shift,
    Control,
    Alt,
    Meta,
    
    /// Other key
    Unknown(u32),
}

/// Keyboard event types.
#[derive(Debug, Clone)]
pub enum KeyboardEvent {
    /// Key was pressed down.
    KeyDown {
        /// Virtual key code
        key: KeyCode,
        /// Character representation (if printable)
        character: Option<char>,
        /// Modifier keys held during event
        modifiers: Modifiers,
        /// Whether this is a repeated key event
        repeat: bool,
    },
    /// Key was released.
    KeyUp {
        /// Virtual key code
        key: KeyCode,
        /// Modifier keys held during event
        modifiers: Modifiers,
    },
    /// Text input event (for IME and composed characters).
    TextInput {
        /// The input text
        text: String,
    },
}

impl KeyboardEvent {
    /// Get the modifiers for the keyboard event, if applicable.
    pub fn modifiers(&self) -> Option<Modifiers> {
        match self {
            KeyboardEvent::KeyDown { modifiers, .. }
            | KeyboardEvent::KeyUp { modifiers, .. } => Some(*modifiers),
            KeyboardEvent::TextInput { .. } => None,
        }
    }

    /// Get the key code for the keyboard event, if applicable.
    pub fn key_code(&self) -> Option<KeyCode> {
        match self {
            KeyboardEvent::KeyDown { key, .. } | KeyboardEvent::KeyUp { key, .. } => Some(*key),
            KeyboardEvent::TextInput { .. } => None,
        }
    }
}

/// Result of handling an input event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventResult {
    /// Event was handled and should not propagate further
    Handled,
    /// Event was not handled and should propagate to parent
    NotHandled,
}

/// Trait for components that handle mouse events.
pub trait MouseListener {
    /// Called when a mouse event occurs on this component.
    ///
    /// Returns `EventResult::Handled` if the event was handled,
    /// or `EventResult::NotHandled` to allow propagation to parent.
    fn on_mouse_event(&mut self, event: &MouseEvent) -> EventResult {
        let _ = event;
        EventResult::NotHandled
    }
}

/// Trait for components that handle keyboard events.
pub trait KeyboardListener {
    /// Called when a keyboard event occurs on this component.
    ///
    /// Returns `EventResult::Handled` if the event was handled,
    /// or `EventResult::NotHandled` to allow propagation.
    fn on_keyboard_event(&mut self, event: &KeyboardEvent) -> EventResult {
        let _ = event;
        EventResult::NotHandled
    }
}

/// Callback function type for mouse events.
pub type MouseCallback = Box<dyn FnMut(&MouseEvent) -> EventResult>;

/// Callback function type for keyboard events.
pub type KeyboardCallback = Box<dyn FnMut(&KeyboardEvent) -> EventResult>;

/// Manager for input event callbacks.
///
/// This allows components to register callbacks for mouse and keyboard events
/// without implementing the listener traits.
pub struct InputCallbacks {
    mouse_callbacks: Vec<(ComponentId, MouseCallback)>,
    keyboard_callbacks: Vec<(ComponentId, KeyboardCallback)>,
}

impl InputCallbacks {
    /// Create a new input callbacks manager.
    pub fn new() -> Self {
        Self {
            mouse_callbacks: Vec::new(),
            keyboard_callbacks: Vec::new(),
        }
    }

    /// Register a mouse event callback for a component.
    pub fn add_mouse_callback<F>(&mut self, component_id: ComponentId, callback: F)
    where
        F: FnMut(&MouseEvent) -> EventResult + 'static,
    {
        self.mouse_callbacks.push((component_id, Box::new(callback)));
    }

    /// Register a keyboard event callback for a component.
    pub fn add_keyboard_callback<F>(&mut self, component_id: ComponentId, callback: F)
    where
        F: FnMut(&KeyboardEvent) -> EventResult + 'static,
    {
        self.keyboard_callbacks.push((component_id, Box::new(callback)));
    }

    /// Remove all callbacks for a component.
    pub fn remove_callbacks(&mut self, component_id: ComponentId) {
        self.mouse_callbacks.retain(|(id, _)| *id != component_id);
        self.keyboard_callbacks.retain(|(id, _)| *id != component_id);
    }

    /// Dispatch a mouse event to registered callbacks for a component.
    pub fn dispatch_mouse_event(
        &mut self,
        component_id: ComponentId,
        event: &MouseEvent,
    ) -> EventResult {
        for (id, callback) in &mut self.mouse_callbacks {
            if *id == component_id {
                let result = callback(event);
                if result == EventResult::Handled {
                    return EventResult::Handled;
                }
            }
        }
        EventResult::NotHandled
    }

    /// Dispatch a keyboard event to registered callbacks for a component.
    pub fn dispatch_keyboard_event(
        &mut self,
        component_id: ComponentId,
        event: &KeyboardEvent,
    ) -> EventResult {
        for (id, callback) in &mut self.keyboard_callbacks {
            if *id == component_id {
                let result = callback(event);
                if result == EventResult::Handled {
                    return EventResult::Handled;
                }
            }
        }
        EventResult::NotHandled
    }
}

impl Default for InputCallbacks {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modifiers() {
        let mods = Modifiers::none();
        assert!(!mods.any());

        let mods = Modifiers {
            shift: true,
            ctrl: false,
            alt: false,
            meta: false,
        };
        assert!(mods.any());
    }

    #[test]
    fn test_mouse_event_position() {
        let event = MouseEvent::ButtonDown {
            x: 10,
            y: 20,
            button: MouseButton::Left,
            modifiers: Modifiers::none(),
        };
        assert_eq!(event.position(), (10, 20));
    }

    #[test]
    fn test_mouse_event_modifiers() {
        let mods = Modifiers {
            shift: true,
            ctrl: false,
            alt: false,
            meta: false,
        };
        let event = MouseEvent::Move {
            x: 10,
            y: 20,
            modifiers: mods,
        };
        assert_eq!(event.modifiers(), Some(mods));

        let event = MouseEvent::Enter { x: 10, y: 20 };
        assert_eq!(event.modifiers(), None);
    }

    #[test]
    fn test_keyboard_event_modifiers() {
        let mods = Modifiers {
            shift: true,
            ctrl: false,
            alt: false,
            meta: false,
        };
        let event = KeyboardEvent::KeyDown {
            key: KeyCode::A,
            character: Some('A'),
            modifiers: mods,
            repeat: false,
        };
        assert_eq!(event.modifiers(), Some(mods));
        assert_eq!(event.key_code(), Some(KeyCode::A));
    }

    #[test]
    fn test_input_callbacks() {
        use std::sync::{Arc, Mutex};
        
        let mut callbacks = InputCallbacks::new();
        let component_id = 1;

        let handled = Arc::new(Mutex::new(false));
        let handled_clone = handled.clone();
        callbacks.add_mouse_callback(component_id, move |_event| {
            *handled_clone.lock().unwrap() = true;
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
    fn test_remove_callbacks() {
        let mut callbacks = InputCallbacks::new();
        let component_id = 1;

        callbacks.add_mouse_callback(component_id, |_| EventResult::Handled);
        callbacks.add_keyboard_callback(component_id, |_| EventResult::Handled);

        callbacks.remove_callbacks(component_id);

        let mouse_event = MouseEvent::ButtonDown {
            x: 10,
            y: 20,
            button: MouseButton::Left,
            modifiers: Modifiers::none(),
        };
        let result = callbacks.dispatch_mouse_event(component_id, &mouse_event);
        assert_eq!(result, EventResult::NotHandled);
    }
}
