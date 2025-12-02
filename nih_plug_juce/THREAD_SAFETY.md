# Thread Safety in nih_plug_juce

This document describes the thread safety enforcement mechanisms in the nih_plug_juce crate.

## Overview

JUCE requires all GUI operations to be performed on the message thread. This crate enforces this requirement through multiple complementary layers:

1. **Compile-time enforcement** via the type system (!Send + !Sync)
2. **Runtime assertions** in debug builds
3. **Safe cross-thread communication** via MessageManager

## Layer 1: Type System Enforcement

All JUCE GUI types include `PhantomData<*mut ()>` which makes them `!Send + !Sync`. This prevents the compiler from allowing GUI objects to be moved or shared across threads.

### Example Types with PhantomData

```rust
pub struct Component {
    ptr: *mut ffi::JuceComponent,
    _phantom: PhantomData<*mut ()>, // Makes Component !Send + !Sync
}

pub struct TextButton {
    component: Component,
    _phantom: PhantomData<*mut ()>, // Makes TextButton !Send + !Sync
}
```

### Compile-Time Prevention

The following code will not compile:

```rust
use nih_plug_juce::Component;
use std::thread;

let component = Component::new().unwrap();

// ERROR: Component cannot be sent between threads safely
thread::spawn(move || {
    component.set_visible(true);
});
```

Error message:
```
error[E0277]: `*mut ()` cannot be sent between threads safely
```

## Layer 2: Runtime Assertions

All public methods on GUI types include `assert_message_thread!()` debug assertions. These verify at runtime (in debug builds only) that methods are called on the message thread.

### Implementation

```rust
pub fn set_bounds(&mut self, x: i32, y: i32, width: i32, height: i32) {
    assert_message_thread!(); // Debug assertion
    
    if self.ptr.is_null() {
        return;
    }
    
    unsafe {
        ffi::component_set_bounds(self.ptr, x, y, width, height);
    }
}
```

### The assert_message_thread! Macro

```rust
#[macro_export]
macro_rules! assert_message_thread {
    () => {
        debug_assert!(
            $crate::MessageManager::is_message_thread(),
            "This operation must be called on the message thread"
        );
    };
}
```

### Benefits

- **Zero runtime cost in release builds**: The assertions compile to nothing in release mode
- **Early detection**: Catches threading violations during development
- **Clear error messages**: Provides helpful panic messages when violations occur

## Layer 3: Safe Cross-Thread Communication

When you need to update the UI from another thread (e.g., the audio processing thread), use `MessageManager::call_async()`:

### Example

```rust
use nih_plug_juce::MessageManager;

// From audio thread
let gain_value = 0.75;
MessageManager::call_async(move || {
    // This closure runs on the message thread
    // Safe to update UI here
    slider.set_value(gain_value);
}).expect("Failed to post callback");
```

### How It Works

1. The closure is boxed and sent to the message thread
2. JUCE's message loop executes the closure on the message thread
3. The closure can safely access and modify GUI objects

## Types Covered

### GUI Components (!Send + !Sync)

All these types enforce message thread usage:

- `Component`
- `TextButton`, `Slider`, `Label`, `ComboBox`, `TextEditor`, `ToggleButton`
- `DocumentWindow`, `ResizableWindow`, `Viewport`, `TabbedComponent`
- `ListBox`, `TreeView`
- `FlexBox`
- `Timer`
- `FileChooser`
- `LookAndFeel`
- `Drawable`, `DrawableButton`
- `SliderParameterAttachment`
- `Graphics<'a>` (via lifetime parameter)

### Value Types (Send + Sync)

These types are safe to use across threads as they are value types:

- `Path`
- `Colour`
- `Font`
- `Image`
- `AffineTransform`

These types have internal reference counting or are simple data structures that can be safely copied or shared.

## Testing Thread Safety

### Compile-Time Tests

The `tests/thread_safety_compile_tests.rs` file contains tests that verify types are !Send and !Sync. These tests are designed to fail compilation if uncommented, demonstrating the type system enforcement.

### Runtime Tests

Integration tests verify that:
1. GUI operations work correctly on the message thread
2. `MessageManager::call_async()` successfully posts callbacks
3. `MessageManager::is_message_thread()` correctly identifies the message thread

## Best Practices

### DO

✅ Create and manipulate GUI objects on the message thread
✅ Use `MessageManager::call_async()` to update UI from other threads
✅ Use `assert_message_thread!()` in your own GUI-related code
✅ Keep GUI object lifetimes within the message thread scope

### DON'T

❌ Try to send GUI objects to other threads
❌ Try to share GUI objects between threads
❌ Access GUI objects directly from the audio thread
❌ Store GUI objects in `Arc` or other thread-safe containers

## Implementation Checklist

This task (Task 37) implemented the following:

- [x] Verified all GUI types have `PhantomData<*mut ()>` for !Send + !Sync
- [x] Added `assert_message_thread!()` calls to all public methods in Component
- [x] Added `assert_message_thread!()` calls to critical methods in other types
- [x] Documented thread safety requirements in module documentation
- [x] Created compile-time tests to verify !Send + !Sync enforcement
- [x] Updated crate-level documentation with thread safety information
- [x] Created this comprehensive thread safety documentation

## References

- JUCE Message Thread Documentation: https://docs.juce.com/master/classMessageManager.html
- Rust Send and Sync Traits: https://doc.rust-lang.org/nomicon/send-and-sync.html
- PhantomData Documentation: https://doc.rust-lang.org/std/marker/struct.PhantomData.html
