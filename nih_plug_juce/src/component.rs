//! JUCE Component wrapper.
//!
//! This module provides a safe Rust wrapper around JUCE's Component class,
//! which is the base class for all GUI elements in JUCE.
//!
//! # Thread Safety
//!
//! All Component operations must be performed on the JUCE message thread.
//! This is enforced through the type system - Component does not implement
//! `Send` or `Sync`, preventing it from being moved or shared across threads.
//!
//! Additionally, all public methods include debug assertions that verify
//! they are called on the message thread. These assertions help catch
//! threading violations during development.
//!
//! # Memory Management
//!
//! Components use RAII for automatic memory management. When a Component
//! is dropped, its C++ destructor is automatically called to free resources.
//!
//! # Parent-Child Relationships
//!
//! JUCE uses a parent-owns-children model. When you add a child component
//! to a parent, the parent takes ownership. The Rust wrapper maintains
//! references but the actual ownership is managed by JUCE.
//!
//! # Examples
//!
//! ```ignore
//! use nih_plug_juce::Component;
//!
//! let mut parent = Component::new()?;
//! parent.set_bounds(0, 0, 400, 300);
//! parent.set_visible(true);
//!
//! let mut child = Component::new()?;
//! child.set_bounds(10, 10, 100, 50);
//! parent.add_child(&child)?;
//! ```

use crate::assert_message_thread;
use crate::bridge::ffi;
use crate::error::{JuceError, Result};
use std::marker::PhantomData;
use std::ptr;

/// A JUCE Component - the base class for all GUI elements.
///
/// Component is the fundamental building block of JUCE GUIs. It represents
/// a rectangular area that can be displayed, positioned, and interacted with.
/// All JUCE widgets (buttons, sliders, labels, etc.) inherit from Component.
///
/// # Thread Safety
///
/// Component does not implement `Send` or `Sync`, enforcing that all GUI
/// operations occur on the message thread. This matches JUCE's threading
/// requirements.
///
/// # Memory Management
///
/// Components are automatically cleaned up when dropped. The Drop implementation
/// calls the C++ destructor to free JUCE resources.
///
/// # Examples
///
/// ```ignore
/// use nih_plug_juce::Component;
///
/// // Create a new component
/// let mut component = Component::new()?;
///
/// // Set its position and size
/// component.set_bounds(0, 0, 400, 300);
///
/// // Make it visible
/// component.set_visible(true);
///
/// // Component is automatically cleaned up when it goes out of scope
/// ```
pub struct Component {
    /// Opaque pointer to the C++ juce::Component object.
    /// This pointer is owned by this struct and will be freed in Drop.
    ptr: *mut ffi::JuceComponent,
    
    /// PhantomData to make Component !Send + !Sync.
    /// This enforces that Component can only be used on the thread where
    /// it was created (the message thread), matching JUCE's requirements.
    _phantom: PhantomData<*mut ()>,
}

impl Component {
    /// Create a new Component.
    ///
    /// This allocates a new juce::Component in C++ and returns a safe
    /// Rust wrapper around it.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Component)` on success, or an error if component
    /// creation failed.
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use nih_plug_juce::Component;
    ///
    /// let component = Component::new()?;
    /// ```
    pub fn new() -> Result<Self> {
        assert_message_thread!();
        
        let mut error_buf = vec![0u8; 256];
        
        let ptr = unsafe {
            ffi::create_component(
                error_buf.as_mut_ptr() as *mut i8,
                error_buf.len()
            )
        };
        
        if ptr.is_null() {
            let error_msg = String::from_utf8_lossy(&error_buf)
                .trim_end_matches('\0')
                .to_string();
            
            if error_msg.is_empty() {
                Err(JuceError::ComponentCreationFailed(
                    "Unknown error creating component".to_string()
                ))
            } else {
                Err(JuceError::ComponentCreationFailed(error_msg))
            }
        } else {
            Ok(Component {
                ptr,
                _phantom: PhantomData,
            })
        }
    }
    
    /// Create a Component from a raw pointer.
    ///
    /// This is used internally by widget wrappers to wrap C++ component pointers.
    ///
    /// # Safety
    ///
    /// The pointer must be a valid JuceComponent pointer created by one of the
    /// FFI creation functions. The Component takes ownership of the pointer and
    /// will free it when dropped.
    ///
    /// # Arguments
    ///
    /// * `ptr` - A valid pointer to a JuceComponent
    ///
    /// # Returns
    ///
    /// Returns a Component wrapping the pointer.
    pub(crate) unsafe fn from_raw(ptr: *mut ffi::JuceComponent) -> Self {
        Component {
            ptr,
            _phantom: PhantomData,
        }
    }
    
    /// Add a child component to this component.
    ///
    /// The child component will be added to this component's children list
    /// and will be displayed as part of this component. The child's position
    /// is relative to this component's top-left corner.
    ///
    /// # Arguments
    ///
    /// * `child` - The component to add as a child
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the operation failed.
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut parent = Component::new()?;
    /// let child = Component::new()?;
    /// parent.add_child(&child)?;
    /// ```
    pub fn add_child(&mut self, child: &Component) -> Result<()> {
        assert_message_thread!();
        
        if self.ptr.is_null() {
            return Err(JuceError::NullPointer("Component pointer is null".to_string()));
        }
        
        if child.ptr.is_null() {
            return Err(JuceError::NullPointer("Child component pointer is null".to_string()));
        }
        
        let mut error_buf = vec![0u8; 256];
        
        let result = unsafe {
            ffi::component_add_child(
                self.ptr,
                child.ptr,
                error_buf.as_mut_ptr() as *mut i8,
                error_buf.len()
            )
        };
        
        if result == 0 {
            Ok(())
        } else {
            let error_msg = String::from_utf8_lossy(&error_buf)
                .trim_end_matches('\0')
                .to_string();
            Err(JuceError::CppException(error_msg))
        }
    }
    
    /// Remove a child component from this component.
    ///
    /// The child component will be removed from this component's children list
    /// and will no longer be displayed as part of this component.
    ///
    /// # Arguments
    ///
    /// * `child` - The component to remove
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the operation failed.
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut parent = Component::new()?;
    /// let child = Component::new()?;
    /// parent.add_child(&child)?;
    /// parent.remove_child(&child)?;
    /// ```
    pub fn remove_child(&mut self, child: &Component) -> Result<()> {
        assert_message_thread!();
        
        if self.ptr.is_null() {
            return Err(JuceError::NullPointer("Component pointer is null".to_string()));
        }
        
        if child.ptr.is_null() {
            return Err(JuceError::NullPointer("Child component pointer is null".to_string()));
        }
        
        let mut error_buf = vec![0u8; 256];
        
        let result = unsafe {
            ffi::component_remove_child(
                self.ptr,
                child.ptr,
                error_buf.as_mut_ptr() as *mut i8,
                error_buf.len()
            )
        };
        
        if result == 0 {
            Ok(())
        } else {
            let error_msg = String::from_utf8_lossy(&error_buf)
                .trim_end_matches('\0')
                .to_string();
            Err(JuceError::CppException(error_msg))
        }
    }
    
    /// Set the position and size of this component.
    ///
    /// The bounds are specified as x, y coordinates (relative to the parent)
    /// and width, height dimensions.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate of the top-left corner
    /// * `y` - Y coordinate of the top-left corner
    /// * `width` - Width of the component
    /// * `height` - Height of the component
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut component = Component::new()?;
    /// component.set_bounds(10, 20, 300, 200);
    /// ```
    pub fn set_bounds(&mut self, x: i32, y: i32, width: i32, height: i32) {
        assert_message_thread!();
        
        if self.ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::component_set_bounds(self.ptr, x, y, width, height);
        }
    }
    
    /// Set whether this component is visible.
    ///
    /// Invisible components are not drawn and do not receive mouse events.
    ///
    /// # Arguments
    ///
    /// * `visible` - true to make the component visible, false to hide it
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut component = Component::new()?;
    /// component.set_visible(true);
    /// ```
    pub fn set_visible(&mut self, visible: bool) {
        assert_message_thread!();
        
        if self.ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::component_set_visible(self.ptr, visible);
        }
    }
    
    /// Trigger a repaint of this component.
    ///
    /// This marks the component as needing to be redrawn. The actual
    /// painting will occur asynchronously on the next paint cycle.
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut component = Component::new()?;
    /// // ... modify component state ...
    /// component.repaint();
    /// ```
    pub fn repaint(&mut self) {
        assert_message_thread!();
        
        if self.ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::component_repaint(self.ptr);
        }
    }
    
    /// Create a new Component that supports paint callbacks.
    ///
    /// This creates a component that can have a custom paint callback set.
    /// Use this instead of `new()` if you need to implement custom drawing.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Component)` on success, or an error if component
    /// creation failed.
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use nih_plug_juce::Component;
    ///
    /// let mut component = Component::new_with_paint_callback()?;
    /// component.set_paint_callback(|g| {
    ///     // Custom drawing code here
    /// });
    /// ```
    pub fn new_with_paint_callback() -> Result<Self> {
        assert_message_thread!();
        
        let mut error_buf = vec![0u8; 256];
        
        let ptr = unsafe {
            ffi::create_component_with_paint_callback(
                error_buf.as_mut_ptr() as *mut i8,
                error_buf.len()
            )
        };
        
        if ptr.is_null() {
            let error_msg = String::from_utf8_lossy(&error_buf)
                .trim_end_matches('\0')
                .to_string();
            
            if error_msg.is_empty() {
                Err(JuceError::ComponentCreationFailed(
                    "Unknown error creating component with paint callback".to_string()
                ))
            } else {
                Err(JuceError::ComponentCreationFailed(error_msg))
            }
        } else {
            Ok(Component {
                ptr,
                _phantom: PhantomData,
            })
        }
    }
    
    /// Set a paint callback for this component.
    ///
    /// The callback will be invoked whenever the component needs to be redrawn.
    /// The callback receives a mutable reference to a Graphics context that can
    /// be used for drawing operations.
    ///
    /// # Arguments
    ///
    /// * `callback` - A closure that takes a mutable Graphics reference
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the operation failed.
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    /// The callback will also be invoked on the message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use nih_plug_juce::{Component, Graphics};
    ///
    /// let mut component = Component::new_with_paint_callback()?;
    /// component.set_paint_callback(|g| {
    ///     g.fill_rect(10, 10, 100, 50);
    /// })?;
    /// ```
    pub fn set_paint_callback<F>(&mut self, callback: F) -> Result<()>
    where
        F: Fn(&mut crate::Graphics) + 'static,
    {
        assert_message_thread!();
        
        if self.ptr.is_null() {
            return Err(JuceError::NullPointer("Component pointer is null".to_string()));
        }
        
        // Box the closure and convert to raw pointer
        let boxed = Box::new(callback);
        let raw = Box::into_raw(boxed);
        
        // Define the trampoline function that will be called from C++
        unsafe extern "C" fn trampoline<F>(
            closure_ptr: usize,
            graphics_ptr: usize,
        ) where
            F: Fn(&mut crate::Graphics),
        {
            // Reconstruct the closure reference (without taking ownership)
            let closure = &*(closure_ptr as *const F);
            
            // Wrap the Graphics pointer
            let graphics_ptr = graphics_ptr as *mut ffi::JuceGraphics;
            let mut graphics = crate::Graphics::from_raw(graphics_ptr);
            
            // Invoke the Rust closure
            closure(&mut graphics);
        }
        
        // Define the drop function that will be called when the component is destroyed
        unsafe extern "C" fn drop_closure<F>(closure_ptr: usize)
        where
            F: Fn(&mut crate::Graphics),
        {
            // Take ownership and drop the closure
            let _ = Box::from_raw(closure_ptr as *mut F);
        }
        
        let mut error_buf = vec![0u8; 256];
        
        let result = unsafe {
            ffi::component_set_paint_callback(
                self.ptr,
                raw as usize,
                trampoline::<F> as usize,
                drop_closure::<F> as usize,
                error_buf.as_mut_ptr() as *mut i8,
                error_buf.len(),
            )
        };
        
        if result == 0 {
            Ok(())
        } else {
            // If setting the callback failed, we need to clean up the boxed closure
            unsafe {
                let _ = Box::from_raw(raw);
            }
            
            let error_msg = String::from_utf8_lossy(&error_buf)
                .trim_end_matches('\0')
                .to_string();
            Err(JuceError::CppException(error_msg))
        }
    }
    
    /// Set a mouse listener for this component.
    ///
    /// The listener will receive callbacks for mouse events (down, drag, up,
    /// enter, exit) that occur on this component.
    ///
    /// # Arguments
    ///
    /// * `listener` - A boxed trait object implementing MouseListener
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the operation failed.
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    /// The listener callbacks will also be invoked on the message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use nih_plug_juce::{Component, MouseListener, MouseEvent};
    ///
    /// struct MyListener;
    ///
    /// impl MouseListener for MyListener {
    ///     fn mouse_down(&mut self, event: &MouseEvent) {
    ///         println!("Clicked at ({}, {})", event.x, event.y);
    ///     }
    /// }
    ///
    /// let mut component = Component::new()?;
    /// component.set_mouse_listener(Box::new(MyListener))?;
    /// ```
    pub fn set_mouse_listener(&mut self, listener: Box<dyn crate::events::mouse::MouseListener>) -> Result<()> {
        assert_message_thread!();
        
        if self.ptr.is_null() {
            return Err(JuceError::NullPointer("Component pointer is null".to_string()));
        }
        
        // Box the listener and convert to raw pointer
        let boxed = Box::new(listener);
        let raw = Box::into_raw(boxed);
        
        // Define trampoline functions for each mouse event
        unsafe extern "C" fn mouse_down_trampoline(
            listener_ptr: usize,
            x: i32,
            y: i32,
            shift: bool,
            ctrl: bool,
            alt: bool,
            cmd: bool,
        ) {
            let listener = &mut *(listener_ptr as *mut Box<dyn crate::events::mouse::MouseListener>);
            let mods = crate::events::mouse::ModifierKeys::new(shift, ctrl, alt, cmd);
            let event = crate::events::mouse::MouseEvent::new(x, y, mods);
            listener.mouse_down(&event);
        }
        
        unsafe extern "C" fn mouse_drag_trampoline(
            listener_ptr: usize,
            x: i32,
            y: i32,
            shift: bool,
            ctrl: bool,
            alt: bool,
            cmd: bool,
        ) {
            let listener = &mut *(listener_ptr as *mut Box<dyn crate::events::mouse::MouseListener>);
            let mods = crate::events::mouse::ModifierKeys::new(shift, ctrl, alt, cmd);
            let event = crate::events::mouse::MouseEvent::new(x, y, mods);
            listener.mouse_drag(&event);
        }
        
        unsafe extern "C" fn mouse_up_trampoline(
            listener_ptr: usize,
            x: i32,
            y: i32,
            shift: bool,
            ctrl: bool,
            alt: bool,
            cmd: bool,
        ) {
            let listener = &mut *(listener_ptr as *mut Box<dyn crate::events::mouse::MouseListener>);
            let mods = crate::events::mouse::ModifierKeys::new(shift, ctrl, alt, cmd);
            let event = crate::events::mouse::MouseEvent::new(x, y, mods);
            listener.mouse_up(&event);
        }
        
        unsafe extern "C" fn mouse_enter_trampoline(
            listener_ptr: usize,
            x: i32,
            y: i32,
            shift: bool,
            ctrl: bool,
            alt: bool,
            cmd: bool,
        ) {
            let listener = &mut *(listener_ptr as *mut Box<dyn crate::events::mouse::MouseListener>);
            let mods = crate::events::mouse::ModifierKeys::new(shift, ctrl, alt, cmd);
            let event = crate::events::mouse::MouseEvent::new(x, y, mods);
            listener.mouse_enter(&event);
        }
        
        unsafe extern "C" fn mouse_exit_trampoline(
            listener_ptr: usize,
            x: i32,
            y: i32,
            shift: bool,
            ctrl: bool,
            alt: bool,
            cmd: bool,
        ) {
            let listener = &mut *(listener_ptr as *mut Box<dyn crate::events::mouse::MouseListener>);
            let mods = crate::events::mouse::ModifierKeys::new(shift, ctrl, alt, cmd);
            let event = crate::events::mouse::MouseEvent::new(x, y, mods);
            listener.mouse_exit(&event);
        }
        
        unsafe extern "C" fn drop_listener(listener_ptr: usize) {
            // Take ownership and drop the listener
            let _ = Box::from_raw(listener_ptr as *mut Box<dyn crate::events::mouse::MouseListener>);
        }
        
        let mut error_buf = vec![0u8; 256];
        
        let result = unsafe {
            ffi::component_set_mouse_listener(
                self.ptr,
                raw as usize,
                mouse_down_trampoline as usize,
                mouse_drag_trampoline as usize,
                mouse_up_trampoline as usize,
                mouse_enter_trampoline as usize,
                mouse_exit_trampoline as usize,
                drop_listener as usize,
                error_buf.as_mut_ptr() as *mut i8,
                error_buf.len(),
            )
        };
        
        if result == 0 {
            Ok(())
        } else {
            // If setting the listener failed, we need to clean up the boxed listener
            unsafe {
                let _ = Box::from_raw(raw);
            }
            
            let error_msg = String::from_utf8_lossy(&error_buf)
                .trim_end_matches('\0')
                .to_string();
            Err(JuceError::CppException(error_msg))
        }
    }
    
    /// Set whether this component wants keyboard focus.
    ///
    /// When a component wants keyboard focus, it can receive keyboard events
    /// when clicked or when focus is explicitly given to it.
    ///
    /// # Arguments
    ///
    /// * `wants` - true to enable keyboard focus, false to disable
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the operation failed.
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use nih_plug_juce::Component;
    ///
    /// let mut component = Component::new()?;
    /// component.set_wants_keyboard_focus(true)?;
    /// ```
    pub fn set_wants_keyboard_focus(&mut self, wants: bool) -> Result<()> {
        assert_message_thread!();
        
        if self.ptr.is_null() {
            return Err(JuceError::NullPointer("Component pointer is null".to_string()));
        }
        
        let mut error_buf = vec![0u8; 256];
        
        let result = unsafe {
            ffi::component_set_wants_keyboard_focus(
                self.ptr,
                wants,
                error_buf.as_mut_ptr() as *mut i8,
                error_buf.len(),
            )
        };
        
        if result == 0 {
            Ok(())
        } else {
            let error_msg = String::from_utf8_lossy(&error_buf)
                .trim_end_matches('\0')
                .to_string();
            Err(JuceError::CppException(error_msg))
        }
    }
    
    /// Set a keyboard listener for this component.
    ///
    /// The listener will receive callbacks for keyboard events (key pressed,
    /// key state changed, focus gained, focus lost) that occur on this component.
    ///
    /// Note: The component must have keyboard focus enabled via
    /// `set_wants_keyboard_focus(true)` to receive keyboard events.
    ///
    /// # Arguments
    ///
    /// * `listener` - A boxed trait object implementing KeyListener
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the operation failed.
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    /// The listener callbacks will also be invoked on the message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use nih_plug_juce::{Component, KeyListener, KeyPress};
    ///
    /// struct MyListener;
    ///
    /// impl KeyListener for MyListener {
    ///     fn key_pressed(&mut self, key: &KeyPress) -> bool {
    ///         println!("Key pressed: {}", key.key_code);
    ///         true // Consume the event
    ///     }
    /// }
    ///
    /// let mut component = Component::new()?;
    /// component.set_wants_keyboard_focus(true)?;
    /// component.set_key_listener(Box::new(MyListener))?;
    /// ```
    pub fn set_key_listener(&mut self, listener: Box<dyn crate::events::keyboard::KeyListener>) -> Result<()> {
        assert_message_thread!();
        
        if self.ptr.is_null() {
            return Err(JuceError::NullPointer("Component pointer is null".to_string()));
        }
        
        // Box the listener and convert to raw pointer
        let boxed = Box::new(listener);
        let raw = Box::into_raw(boxed);
        
        // Define trampoline functions for each keyboard event
        unsafe extern "C" fn key_pressed_trampoline(
            listener_ptr: usize,
            key_code: i32,
            shift: bool,
            ctrl: bool,
            alt: bool,
            cmd: bool,
        ) -> bool {
            let listener = &mut *(listener_ptr as *mut Box<dyn crate::events::keyboard::KeyListener>);
            let mods = crate::events::mouse::ModifierKeys::new(shift, ctrl, alt, cmd);
            let key = crate::events::keyboard::KeyPress::new(key_code, mods);
            listener.key_pressed(&key)
        }
        
        unsafe extern "C" fn key_state_changed_trampoline(listener_ptr: usize) -> bool {
            let listener = &mut *(listener_ptr as *mut Box<dyn crate::events::keyboard::KeyListener>);
            listener.key_state_changed()
        }
        
        unsafe extern "C" fn focus_gained_trampoline(listener_ptr: usize) {
            let listener = &mut *(listener_ptr as *mut Box<dyn crate::events::keyboard::KeyListener>);
            listener.focus_gained();
        }
        
        unsafe extern "C" fn focus_lost_trampoline(listener_ptr: usize) {
            let listener = &mut *(listener_ptr as *mut Box<dyn crate::events::keyboard::KeyListener>);
            listener.focus_lost();
        }
        
        unsafe extern "C" fn drop_listener(listener_ptr: usize) {
            // Take ownership and drop the listener
            let _ = Box::from_raw(listener_ptr as *mut Box<dyn crate::events::keyboard::KeyListener>);
        }
        
        let mut error_buf = vec![0u8; 256];
        
        let result = unsafe {
            ffi::component_set_key_listener(
                self.ptr,
                raw as usize,
                key_pressed_trampoline as usize,
                key_state_changed_trampoline as usize,
                focus_gained_trampoline as usize,
                focus_lost_trampoline as usize,
                drop_listener as usize,
                error_buf.as_mut_ptr() as *mut i8,
                error_buf.len(),
            )
        };
        
        if result == 0 {
            Ok(())
        } else {
            // If setting the listener failed, we need to clean up the boxed listener
            unsafe {
                let _ = Box::from_raw(raw);
            }
            
            let error_msg = String::from_utf8_lossy(&error_buf)
                .trim_end_matches('\0')
                .to_string();
            Err(JuceError::CppException(error_msg))
        }
    }
    
    /// Set the LookAndFeel for this component.
    ///
    /// This sets the LookAndFeel object that will be used to draw this component
    /// and its children. The LookAndFeel defines colors, fonts, and drawing methods.
    ///
    /// # Arguments
    ///
    /// * `laf` - A reference to the LookAndFeel to use
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the operation failed.
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Lifetime
    ///
    /// The LookAndFeel must outlive this component. The component holds a
    /// reference to the LookAndFeel, not ownership.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use nih_plug_juce::{Component, LookAndFeel, Colour};
    ///
    /// let mut laf = LookAndFeel::new_v4()?;
    /// laf.set_colour(0x1000100, Colour::from_rgb(100, 100, 200));
    ///
    /// let mut component = Component::new()?;
    /// component.set_look_and_feel(&laf)?;
    /// ```
    pub fn set_look_and_feel(&mut self, laf: &crate::lookandfeel::LookAndFeel) -> Result<()> {
        assert_message_thread!();
        
        if self.ptr.is_null() {
            return Err(JuceError::NullPointer("Component pointer is null".to_string()));
        }
        
        let mut error_buf = vec![0u8; 256];
        
        let result = unsafe {
            ffi::component_set_look_and_feel(
                self.ptr,
                laf.as_ptr(),
                error_buf.as_mut_ptr() as *mut i8,
                error_buf.len(),
            )
        };
        
        if result == 0 {
            Ok(())
        } else {
            let error_msg = String::from_utf8_lossy(&error_buf)
                .trim_end_matches('\0')
                .to_string();
            Err(JuceError::CppException(error_msg))
        }
    }
    
    /// Get the raw pointer to the underlying C++ component.
    ///
    /// # Safety
    ///
    /// This is an internal method used by other parts of the FFI layer.
    /// The returned pointer is only valid as long as this Component exists.
    #[doc(hidden)]
    pub(crate) fn as_ptr(&self) -> *mut ffi::JuceComponent {
        self.ptr
    }
    
    /// Get a mutable raw pointer to the underlying C++ component.
    ///
    /// # Safety
    ///
    /// This is an internal method used by other parts of the FFI layer.
    /// The returned pointer is only valid as long as this Component exists.
    #[doc(hidden)]
    pub(crate) fn as_ptr_mut(&mut self) -> *mut ffi::JuceComponent {
        self.ptr
    }
}

impl Drop for Component {
    /// Automatically clean up the C++ component when the Rust wrapper is dropped.
    ///
    /// This calls the C++ destructor to free JUCE resources.
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                ffi::delete_component(self.ptr);
            }
            self.ptr = ptr::null_mut();
        }
    }
}

// Explicitly do NOT implement Send or Sync for Component.
// This enforces that Component can only be used on the thread where
// it was created (the message thread), matching JUCE's requirements.
//
// The PhantomData<*mut ()> field already makes Component !Send + !Sync,
// but we add these explicit negative trait implementations for clarity
// and to generate better error messages.

// Note: Rust doesn't support explicit negative trait implementations,
// but the PhantomData<*mut ()> field achieves the same effect.

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_component_creation() {
        // Test that we can create a component
        let result = Component::new();
        assert!(result.is_ok(), "Component creation should succeed");
    }
    
    #[test]
    fn test_component_bounds() {
        // Test setting component bounds
        let mut component = Component::new().unwrap();
        component.set_bounds(10, 20, 300, 200);
        // If we get here without crashing, the test passed
    }
    
    #[test]
    fn test_component_visibility() {
        // Test setting component visibility
        let mut component = Component::new().unwrap();
        component.set_visible(true);
        component.set_visible(false);
        // If we get here without crashing, the test passed
    }
    
    #[test]
    fn test_component_repaint() {
        // Test triggering a repaint
        let mut component = Component::new().unwrap();
        component.repaint();
        // If we get here without crashing, the test passed
    }
    
    #[test]
    fn test_parent_child_relationship() {
        // Test adding and removing child components
        let mut parent = Component::new().unwrap();
        let child = Component::new().unwrap();
        
        let add_result = parent.add_child(&child);
        assert!(add_result.is_ok(), "Adding child should succeed");
        
        let remove_result = parent.remove_child(&child);
        assert!(remove_result.is_ok(), "Removing child should succeed");
    }
    
    #[test]
    fn test_mouse_listener() {
        use crate::events::mouse::{MouseListener, MouseEvent};
        use std::cell::RefCell;
        use std::rc::Rc;
        
        // Create a listener that tracks events
        struct TestListener {
            events: Rc<RefCell<Vec<String>>>,
        }
        
        impl MouseListener for TestListener {
            fn mouse_down(&mut self, event: &MouseEvent) {
                self.events.borrow_mut().push(format!("down({},{})", event.x, event.y));
            }
            
            fn mouse_drag(&mut self, event: &MouseEvent) {
                self.events.borrow_mut().push(format!("drag({},{})", event.x, event.y));
            }
            
            fn mouse_up(&mut self, event: &MouseEvent) {
                self.events.borrow_mut().push(format!("up({},{})", event.x, event.y));
            }
            
            fn mouse_enter(&mut self, event: &MouseEvent) {
                self.events.borrow_mut().push(format!("enter({},{})", event.x, event.y));
            }
            
            fn mouse_exit(&mut self, event: &MouseEvent) {
                self.events.borrow_mut().push(format!("exit({},{})", event.x, event.y));
            }
        }
        
        let events = Rc::new(RefCell::new(Vec::new()));
        let listener = TestListener {
            events: events.clone(),
        };
        
        // Create a component with paint callback support (which also supports mouse listeners)
        let mut component = Component::new_with_paint_callback().unwrap();
        
        // Set the mouse listener
        let result = component.set_mouse_listener(Box::new(listener));
        assert!(result.is_ok(), "Setting mouse listener should succeed");
        
        // Note: We can't actually trigger mouse events in a unit test without a real GUI,
        // but we've verified that the listener can be set without errors
    }
}
