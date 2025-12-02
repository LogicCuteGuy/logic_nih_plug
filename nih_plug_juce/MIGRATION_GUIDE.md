# JUCE C++ to Rust FFI Migration Guide

This guide helps JUCE C++ developers transition to using JUCE through Rust FFI bindings in nih-plug. It covers the mapping between C++ and Rust types, ownership model differences, common patterns, and troubleshooting.

## Table of Contents

1. [Quick Start](#quick-start)
2. [Type Mappings](#type-mappings)
3. [Ownership Model Differences](#ownership-model-differences)
4. [Common Patterns and Idioms](#common-patterns-and-idioms)
5. [Thread Safety](#thread-safety)
6. [Callbacks and Event Handling](#callbacks-and-event-handling)
7. [Memory Management](#memory-management)
8. [Troubleshooting](#troubleshooting)

## Quick Start

### C++ JUCE Plugin UI
```cpp
class MyPluginEditor : public juce::AudioProcessorEditor {
public:
    MyPluginEditor() {
        addAndMakeVisible(button);
        button.setButtonText("Click Me");
        button.onClick = [this] { handleClick(); };
        
        setSize(400, 300);
    }
    
private:
    juce::TextButton button;
    
    void handleClick() {
        // Handle button click
    }
};
```

### Rust FFI Equivalent
```rust
use nih_plug_juce::prelude::*;

struct MyPluginEditor {
    root: Component,
    button: TextButton,
}

impl MyPluginEditor {
    fn new() -> Result<Self> {
        let mut root = Component::new()?;
        let mut button = TextButton::new("Click Me")?;
        
        button.set_on_click(|| {
            // Handle button click
        });
        
        root.add_child(&button)?;
        root.set_bounds(0, 0, 400, 300);
        
        Ok(Self { root, button })
    }
}
```

## Type Mappings

### Core Component Classes

| C++ Type | Rust Type | Notes |
|----------|-----------|-------|
| `juce::Component` | `Component` | Base component wrapper |
| `juce::TextButton` | `TextButton` | Implements `Deref<Target=Component>` |
| `juce::Slider` | `Slider` | Implements `Deref<Target=Component>` |
| `juce::Label` | `Label` | Implements `Deref<Target=Component>` |
| `juce::ComboBox` | `ComboBox` | Implements `Deref<Target=Component>` |
| `juce::TextEditor` | `TextEditor` | Implements `Deref<Target=Component>` |
| `juce::ToggleButton` | `ToggleButton` | Implements `Deref<Target=Component>` |

### Graphics and Drawing

| C++ Type | Rust Type | Notes |
|----------|-----------|-------|
| `juce::Graphics` | `Graphics<'a>` | Lifetime-bound to paint callback |
| `juce::Colour` | `Colour` | Owned wrapper with Drop |
| `juce::Font` | `Font` | Owned wrapper with Drop |
| `juce::Image` | `Image` | Owned wrapper with Drop |
| `juce::Path` | `Path` | Owned wrapper with Drop |
| `juce::AffineTransform` | `AffineTransform` | Owned wrapper with Drop |
| `juce::Drawable` | `Drawable` | Owned wrapper with Drop |

### Container Components

| C++ Type | Rust Type | Notes |
|----------|-----------|-------|
| `juce::DocumentWindow` | `DocumentWindow` | Implements `Deref<Target=Component>` |
| `juce::ResizableWindow` | `ResizableWindow` | Implements `Deref<Target=Component>` |
| `juce::Viewport` | `Viewport` | Implements `Deref<Target=Component>` |
| `juce::TabbedComponent` | `TabbedComponent` | Implements `Deref<Target=Component>` |
| `juce::ListBox` | `ListBox` | Implements `Deref<Target=Component>` |
| `juce::TreeView` | `TreeView` | Implements `Deref<Target=Component>` |

### Layout

| C++ Type | Rust Type | Notes |
|----------|-----------|-------|
| `juce::FlexBox` | `FlexBox` | Owned wrapper with Drop |
| `juce::FlexItem` | `FlexItem` | Value type |
| `juce::FlexBox::Direction` | `FlexDirection` | Enum |
| `juce::FlexBox::Wrap` | `FlexWrap` | Enum |

### Events

| C++ Type | Rust Type | Notes |
|----------|-----------|-------|
| `juce::MouseEvent` | `MouseEvent` | Value type |
| `juce::ModifierKeys` | `ModifierKeys` | Value type |
| `juce::KeyPress` | `KeyPress` | Value type |
| `juce::MouseListener` | `MouseListener` trait | Trait-based callbacks |
| `juce::KeyListener` | `KeyListener` trait | Trait-based callbacks |

### Dialogs

| C++ Type | Rust Type | Notes |
|----------|-----------|-------|
| `juce::AlertWindow` | `AlertWindow` | Static methods only |
| `juce::FileChooser` | `FileChooser` | Owned wrapper with Drop |

### Utilities

| C++ Type | Rust Type | Notes |
|----------|-----------|-------|
| `juce::Timer` | `Timer` | Owned wrapper with Drop |
| `juce::MessageManager` | `MessageManager` | Static methods only |
| `juce::LookAndFeel` | `LookAndFeel` | Owned wrapper with Drop |

## Ownership Model Differences

### C++ Ownership in JUCE

In C++, JUCE uses several ownership patterns:

1. **Parent owns children**: When you call `addAndMakeVisible()`, the parent component takes ownership
2. **Manual memory management**: You use `new`/`delete` or smart pointers
3. **Reference counting**: Some objects use `ReferenceCountedObjectPtr`

```cpp
// C++ - Parent owns child
class MyComponent : public juce::Component {
    juce::TextButton button; // Owned by value
    
    MyComponent() {
        addAndMakeVisible(button); // Parent manages lifetime
    }
};
```

### Rust Ownership in FFI Bindings

In Rust, ownership is explicit and enforced by the compiler:

1. **Explicit ownership**: Each component is owned by a Rust struct
2. **Borrowing for hierarchy**: Parent-child relationships use borrowing, not ownership transfer
3. **RAII cleanup**: Drop trait automatically cleans up C++ objects

```rust
// Rust - Explicit ownership
struct MyComponent {
    root: Component,
    button: TextButton, // Owned by this struct
}

impl MyComponent {
    fn new() -> Result<Self> {
        let mut root = Component::new()?;
        let button = TextButton::new("Click")?;
        
        // Borrow button to add to parent - ownership stays here
        root.add_child(&button)?;
        
        Ok(Self { root, button })
    }
}

// Drop automatically called when MyComponent goes out of scope
```

### Key Differences

| Aspect | C++ JUCE | Rust FFI |
|--------|----------|----------|
| **Ownership transfer** | `addAndMakeVisible()` transfers ownership | `add_child()` borrows, ownership stays with caller |
| **Cleanup** | Manual or parent-managed | Automatic via Drop trait |
| **Lifetime tracking** | Runtime (can cause use-after-free) | Compile-time (prevents use-after-free) |
| **Null pointers** | Possible | Not possible (use `Option<T>`) |

### Important: Keep Components Alive

**C++:**
```cpp
void MyComponent::paint(Graphics& g) {
    TextButton tempButton; // Lives on stack
    addAndMakeVisible(tempButton); // Parent takes ownership
    // tempButton destroyed at end of scope - CRASH!
}
```

**Rust:**
```rust
fn paint(&mut self, g: &mut Graphics) {
    let temp_button = TextButton::new("Temp")?; // Lives on stack
    self.root.add_child(&temp_button)?; // Borrows only
    // temp_button dropped at end of scope - CRASH!
}

// CORRECT: Store as field
struct MyComponent {
    button: TextButton, // Owned, lives as long as MyComponent
}
```

**Rule**: In Rust, you must keep components alive by storing them in a struct field. The parent only borrows a reference.

## Common Patterns and Idioms

### Pattern 1: Component Creation and Setup

**C++:**
```cpp
class MyEditor : public AudioProcessorEditor {
public:
    MyEditor() {
        slider.setSliderStyle(Slider::Rotary);
        slider.setRange(0.0, 1.0, 0.01);
        slider.setValue(0.5);
        slider.onValueChange = [this] { handleSliderChange(); };
        addAndMakeVisible(slider);
        
        setSize(400, 300);
    }
    
private:
    Slider slider;
};
```

**Rust:**
```rust
struct MyEditor {
    root: Component,
    slider: Slider,
}

impl MyEditor {
    fn new() -> Result<Self> {
        let mut root = Component::new()?;
        let mut slider = Slider::new(SliderStyle::Rotary)?;
        
        slider.set_range(0.0, 1.0, 0.01);
        slider.set_value(0.5);
        slider.set_on_value_change(|value| {
            // Handle slider change
            println!("Slider value: {}", value);
        });
        
        root.add_child(&slider)?;
        root.set_bounds(0, 0, 400, 300);
        
        Ok(Self { root, slider })
    }
}
```

### Pattern 2: Custom Painting

**C++:**
```cpp
class MyComponent : public Component {
    void paint(Graphics& g) override {
        g.fillAll(Colours::black);
        g.setColour(Colours::white);
        g.drawRect(getLocalBounds(), 2);
        g.drawText("Hello", getLocalBounds(), Justification::centred);
    }
};
```

**Rust:**
```rust
let mut component = Component::new()?;

component.set_paint_callback(|g| {
    // Fill background
    g.set_colour(&Colour::from_rgb(0, 0, 0));
    g.fill_rect(0, 0, 400, 300);
    
    // Draw border
    g.set_colour(&Colour::from_rgb(255, 255, 255));
    g.draw_rect(0, 0, 400, 300);
    
    // Draw text
    g.draw_text("Hello", 0, 0, 400, 300, Justification::Centred);
});
```

### Pattern 3: Layout with FlexBox

**C++:**
```cpp
void MyComponent::resized() {
    FlexBox fb;
    fb.flexDirection = FlexBox::Direction::column;
    fb.items.add(FlexItem(button1).withFlex(1));
    fb.items.add(FlexItem(button2).withFlex(1));
    fb.performLayout(getLocalBounds());
}
```

**Rust:**
```rust
fn layout_components(&mut self) {
    let mut flexbox = FlexBox::new();
    flexbox.set_direction(FlexDirection::Column);
    
    flexbox.add_item(FlexItem {
        component: &self.button1,
        flex_grow: 1.0,
        flex_shrink: 1.0,
        flex_basis: 0.0,
        min_width: 0.0,
        min_height: 0.0,
    });
    
    flexbox.add_item(FlexItem {
        component: &self.button2,
        flex_grow: 1.0,
        flex_shrink: 1.0,
        flex_basis: 0.0,
        min_width: 0.0,
        min_height: 0.0,
    });
    
    flexbox.perform_layout(Rectangle::new(0, 0, 400, 300));
}
```

### Pattern 4: Modal Dialogs

**C++:**
```cpp
void showDialog() {
    AlertWindow::showMessageBoxAsync(
        AlertWindow::InfoIcon,
        "Title",
        "Message",
        "OK"
    );
}
```

**Rust:**
```rust
fn show_dialog() {
    AlertWindow::show_message_box_async(
        "Title",
        "Message",
        || {
            println!("Dialog closed");
        }
    );
}
```

### Pattern 5: File Selection

**C++:**
```cpp
void chooseFile() {
    fileChooser = std::make_unique<FileChooser>(
        "Select a file",
        File::getSpecialLocation(File::userHomeDirectory),
        "*.wav;*.mp3"
    );
    
    fileChooser->launchAsync(
        FileBrowserComponent::openMode | FileBrowserComponent::canSelectFiles,
        [this](const FileChooser& fc) {
            auto file = fc.getResult();
            if (file.existsAsFile()) {
                loadFile(file);
            }
        }
    );
}
```

**Rust:**
```rust
fn choose_file(&mut self) {
    let mut chooser = FileChooser::new(
        "Select a file",
        Path::new("/home/user"),
        "*.wav;*.mp3"
    );
    
    chooser.browse_for_file_to_open(|path| {
        if let Some(file_path) = path {
            println!("Selected: {:?}", file_path);
            // Load file
        }
    });
}
```

### Pattern 6: Timers

**C++:**
```cpp
class MyComponent : public Component, public Timer {
public:
    MyComponent() {
        startTimer(100); // 100ms interval
    }
    
    void timerCallback() override {
        // Called every 100ms
        repaint();
    }
};
```

**Rust:**
```rust
struct MyComponent {
    root: Component,
    timer: Timer,
}

impl MyComponent {
    fn new() -> Result<Self> {
        let root = Component::new()?;
        let mut timer = Timer::new(|| {
            // Called every 100ms
            println!("Timer fired");
        });
        
        timer.start(100); // 100ms interval
        
        Ok(Self { root, timer })
    }
}
```

### Pattern 7: Mouse Event Handling

**C++:**
```cpp
class MyComponent : public Component {
    void mouseDown(const MouseEvent& e) override {
        if (e.mods.isLeftButtonDown()) {
            // Handle left click
        }
    }
    
    void mouseDrag(const MouseEvent& e) override {
        // Handle drag
    }
};
```

**Rust:**
```rust
struct MyMouseListener;

impl MouseListener for MyMouseListener {
    fn mouse_down(&mut self, event: &MouseEvent) {
        if event.mods.is_left_button_down() {
            // Handle left click
        }
    }
    
    fn mouse_drag(&mut self, event: &MouseEvent) {
        // Handle drag
    }
}

// Set listener on component
component.set_mouse_listener(Box::new(MyMouseListener));
```

### Pattern 8: LookAndFeel Customization

**C++:**
```cpp
class MyLookAndFeel : public LookAndFeel_V4 {
public:
    void drawButtonBackground(Graphics& g, Button& button,
                            const Colour& backgroundColour,
                            bool isMouseOverButton,
                            bool isButtonDown) override {
        // Custom drawing
    }
};

// Usage
MyLookAndFeel laf;
button.setLookAndFeel(&laf);
```

**Rust:**
```rust
// Use built-in LookAndFeel
let mut laf = LookAndFeel::new_v4();
laf.set_colour(ButtonColourId::ButtonOnColour as i32, 
               Colour::from_rgb(100, 150, 200));

button.set_look_and_feel(&laf);

// Note: Custom LookAndFeel subclassing requires trait implementation
// See LookAndFeelMethods trait in documentation
```

## Thread Safety

### C++ JUCE Thread Model

JUCE requires all GUI operations on the message thread:

```cpp
// C++ - Manual thread checking
void audioThreadCallback() {
    // WRONG - crashes!
    // button.setButtonText("Processing");
    
    // CORRECT - post to message thread
    MessageManager::callAsync([this] {
        button.setButtonText("Processing");
    });
}
```

### Rust FFI Thread Safety

Rust enforces thread safety at compile time:

```rust
// Rust - Compile-time enforcement
fn audio_thread_callback() {
    // WRONG - won't compile! Component is !Send
    // self.button.set_button_text("Processing");
    
    // CORRECT - use MessageManager
    MessageManager::call_async(|| {
        // This closure runs on message thread
        // Access to GUI components here
    });
}
```

### Key Differences

| Aspect | C++ JUCE | Rust FFI |
|--------|----------|----------|
| **Thread checking** | Runtime (crashes if wrong) | Compile-time (won't compile) |
| **Type system** | No thread safety in types | `!Send + !Sync` prevents cross-thread usage |
| **Debug assertions** | Optional | Built-in with `assert_message_thread!()` |

### Thread Safety Rules

1. **GUI types are `!Send + !Sync`**: Cannot be moved or shared across threads
2. **Use `MessageManager::call_async()`**: For cross-thread UI updates
3. **Debug assertions**: Use `assert_message_thread!()` in debug builds

```rust
pub fn update_ui(&mut self) {
    assert_message_thread!(); // Debug-only check
    self.button.set_button_text("Updated");
}
```

## Callbacks and Event Handling

### C++ Callbacks

JUCE uses several callback mechanisms:

```cpp
// Lambda callbacks
button.onClick = [this] { handleClick(); };

// Virtual method overrides
void paint(Graphics& g) override { }

// Listener interfaces
class MyListener : public Button::Listener {
    void buttonClicked(Button* button) override { }
};
```

### Rust Callbacks

Rust uses closures and trait objects:

```rust
// Closure callbacks
button.set_on_click(|| {
    println!("Clicked!");
});

// Closure with capture
let counter = Arc::new(Mutex::new(0));
let counter_clone = counter.clone();
button.set_on_click(move || {
    let mut count = counter_clone.lock().unwrap();
    *count += 1;
    println!("Click count: {}", *count);
});

// Trait-based callbacks
struct MyMouseListener {
    click_count: usize,
}

impl MouseListener for MyMouseListener {
    fn mouse_down(&mut self, event: &MouseEvent) {
        self.click_count += 1;
        println!("Clicks: {}", self.click_count);
    }
}

component.set_mouse_listener(Box::new(MyMouseListener { click_count: 0 }));
```

### Callback Lifetime Management

**Important**: Callbacks must be `'static` because they're stored in C++ and called later:

```rust
// WRONG - captures reference with non-static lifetime
let text = String::from("Hello");
button.set_on_click(|| {
    println!("{}", text); // Error: closure may outlive `text`
});

// CORRECT - move ownership into closure
let text = String::from("Hello");
button.set_on_click(move || {
    println!("{}", text); // OK: text moved into closure
});

// CORRECT - use Arc for shared state
let text = Arc::new(String::from("Hello"));
let text_clone = text.clone();
button.set_on_click(move || {
    println!("{}", text_clone); // OK: Arc is cloneable
});
```

## Memory Management

### Automatic Cleanup

All JUCE FFI types implement `Drop` for automatic cleanup:

```rust
{
    let button = TextButton::new("Click")?;
    // Use button...
} // button.drop() called automatically, C++ object deleted
```

### Parent-Child Relationships

**Critical**: In Rust, the parent does NOT take ownership of children:

```cpp
// C++ - Parent owns child
void MyComponent::addButton() {
    auto* button = new TextButton("Click");
    addAndMakeVisible(button); // Parent takes ownership
    // button will be deleted when parent is deleted
}
```

```rust
// Rust - Parent borrows child
struct MyComponent {
    root: Component,
    button: TextButton, // MUST store as field!
}

impl MyComponent {
    fn new() -> Result<Self> {
        let mut root = Component::new()?;
        let button = TextButton::new("Click")?;
        
        root.add_child(&button)?; // Borrows only!
        
        // MUST return button to keep it alive
        Ok(Self { root, button })
    }
}
```

### Common Memory Pitfalls

**Pitfall 1: Temporary components**
```rust
// WRONG - button dropped before parent uses it
fn setup_ui(&mut self) {
    let button = TextButton::new("Click")?;
    self.root.add_child(&button)?;
    // button dropped here - CRASH!
}

// CORRECT - store button as field
struct MyUI {
    root: Component,
    button: TextButton, // Stored as field
}
```

**Pitfall 2: Callback captures**
```rust
// WRONG - captures self reference
impl MyComponent {
    fn setup(&mut self) {
        self.button.set_on_click(|| {
            self.update(); // Error: can't capture mutable self
        });
    }
}

// CORRECT - use message passing or Arc<Mutex<T>>
impl MyComponent {
    fn setup(&mut self) {
        let state = Arc::new(Mutex::new(ComponentState::default()));
        let state_clone = state.clone();
        
        self.button.set_on_click(move || {
            let mut s = state_clone.lock().unwrap();
            s.update();
        });
    }
}
```

## Troubleshooting

### Build Issues

#### Issue: "JUCE not found"
```
error: Could not find JUCE installation
```

**Solution**: Ensure JUCE submodule is initialized:
```bash
git submodule update --init --recursive
```

#### Issue: "CMake not found"
```
error: CMake must be installed to build JUCE
```

**Solution**: Install CMake:
```bash
# Ubuntu/Debian
sudo apt-get install cmake

# macOS
brew install cmake

# Windows
# Download from https://cmake.org/download/
```

#### Issue: Platform-specific linker errors

**Linux**: Missing system dependencies
```bash
sudo apt-get install libx11-dev libxrandr-dev libxinerama-dev \
    libxcursor-dev libfreetype6-dev libasound2-dev
```

**macOS**: Missing frameworks
```bash
# Ensure Xcode Command Line Tools installed
xcode-select --install
```

**Windows**: MSVC toolchain required
```bash
# Install Visual Studio 2019 or later with C++ tools
# Or install Build Tools for Visual Studio
```

### Runtime Issues

#### Issue: Segmentation fault on component creation

**Cause**: Message thread not initialized

**Solution**: Ensure JUCE message manager is initialized:
```rust
// In your plugin initialization
MessageManager::get_instance(); // Initializes message thread
```

#### Issue: "Not on message thread" assertion

**Cause**: GUI operation called from wrong thread

**Solution**: Use `MessageManager::call_async()`:
```rust
// From audio thread
MessageManager::call_async(|| {
    // GUI operations here
    button.set_button_text("Updated");
});
```

#### Issue: Component not visible

**Cause**: Forgot to call `set_visible(true)` or `add_child()`

**Solution**:
```rust
let mut component = Component::new()?;
component.set_visible(true); // Make visible
root.add_child(&component)?; // Add to parent
```

#### Issue: Callback not firing

**Cause**: Component dropped before callback fires

**Solution**: Store component as struct field:
```rust
struct MyUI {
    button: TextButton, // Keep alive!
}
```

### Compilation Issues

#### Issue: "trait `Send` is not implemented"

**Cause**: Trying to send GUI type across threads

**Solution**: GUI types are `!Send` by design. Use `MessageManager::call_async()`:
```rust
// WRONG
std::thread::spawn(move || {
    button.set_button_text("Text"); // Error: Button is !Send
});

// CORRECT
std::thread::spawn(|| {
    MessageManager::call_async(|| {
        // Access button here through shared state
    });
});
```

#### Issue: "closure may outlive the current function"

**Cause**: Callback captures non-'static reference

**Solution**: Use `move` and owned data:
```rust
// WRONG
let text = String::from("Hello");
button.set_on_click(|| {
    println!("{}", text); // Error: text not 'static
});

// CORRECT
let text = String::from("Hello");
button.set_on_click(move || {
    println!("{}", text); // OK: text moved into closure
});
```

### Performance Issues

#### Issue: Slow UI updates

**Cause**: Excessive FFI calls in tight loops

**Solution**: Batch operations:
```rust
// SLOW - many FFI calls
for i in 0..1000 {
    component.set_bounds(i, i, 100, 100);
    component.repaint();
}

// FAST - single update
component.set_bounds(final_x, final_y, 100, 100);
component.repaint();
```

#### Issue: High memory usage

**Cause**: Not dropping temporary objects

**Solution**: Use explicit scopes:
```rust
// Create temporary graphics objects in scope
{
    let image = Image::new(ImageFormat::ARGB, 1000, 1000)?;
    // Use image...
} // image dropped here, memory freed
```

### Debugging Tips

1. **Enable debug assertions**: Run with `RUST_BACKTRACE=1` for better error messages
2. **Check message thread**: Use `assert_message_thread!()` liberally
3. **Verify component lifetime**: Ensure components stored as fields
4. **Profile FFI calls**: Use `cargo flamegraph` to identify bottlenecks
5. **Check C++ exceptions**: FFI layer converts exceptions to `Result::Err`

### Getting Help

- **Documentation**: Run `cargo doc --open` for API docs
- **Examples**: See `plugins/examples/juce_ffi_*` for working examples
- **Issues**: Report bugs at the nih-plug repository
- **JUCE Forums**: For JUCE-specific questions (behavior, not FFI)

## Best Practices Summary

1. ✅ **Store components as struct fields** - Don't create temporary components
2. ✅ **Use `Result<T>` for error handling** - Check for FFI errors
3. ✅ **Keep callbacks `'static`** - Use `move` and owned data
4. ✅ **Respect thread boundaries** - Use `MessageManager::call_async()`
5. ✅ **Batch FFI calls** - Minimize cross-boundary overhead
6. ✅ **Use debug assertions** - Catch thread violations early
7. ✅ **Read the examples** - Learn from working code
8. ✅ **Profile performance** - Measure before optimizing

## Conclusion

The JUCE FFI bindings provide safe, idiomatic Rust access to JUCE's powerful GUI system. While there are differences from C++ JUCE, the Rust type system prevents many common bugs at compile time. Follow the patterns in this guide and refer to the examples for best practices.

Happy coding! 🎵🦀
