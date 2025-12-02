# Design Document: JUCE FFI Integration

## Overview

This design document describes the architecture for integrating JUCE's GUI components into nih-plug through Foreign Function Interface (FFI) bindings. Rather than porting JUCE to pure Rust, we create safe Rust wrappers around the actual JUCE C++ library, providing access to JUCE's mature GUI ecosystem while maintaining Rust's safety guarantees.

### Design Philosophy

The integration follows these core principles:

1. **Zero-cost abstraction where possible**: Minimize FFI overhead through inline functions and efficient data passing
2. **Safety at the boundary**: Use Rust's type system to enforce JUCE's threading requirements and memory safety
3. **Idiomatic Rust APIs**: Expose JUCE functionality through Rust conventions (snake_case, Result types, closures)
4. **Modular architecture**: Allow developers to include only the JUCE modules they need
5. **Transparent FFI**: Make the C++ layer invisible to plugin developers through well-designed abstractions

### Key Design Decisions

**Decision 1: Use cxx crate for FFI layer**
- Rationale: cxx provides safe, ergonomic C++/Rust interop with automatic bridge code generation
- Alternative considered: bindgen (rejected due to lack of C++ class support and callback complexity)
- Trade-off: Requires explicit bridge definitions but provides better safety and ergonomics

**Decision 2: Opaque pointer pattern for JUCE objects**
- Rationale: JUCE objects have complex C++ internals that shouldn't be exposed to Rust
- Implementation: Wrap C++ pointers in Rust structs with Drop implementations for automatic cleanup
- Benefit: Prevents memory leaks and use-after-free through Rust's ownership system

**Decision 3: Message thread enforcement through type system**
- Rationale: JUCE requires all GUI operations on the message thread
- Implementation: GUI types don't implement Send/Sync, preventing cross-thread usage at compile time
- Benefit: Thread safety violations become compile errors rather than runtime crashes

**Decision 4: Callback bridging through trait objects**
- Rationale: JUCE uses virtual methods and callbacks extensively
- Implementation: Convert C++ callbacks to Rust closures using Box<dyn Fn> and trait objects
- Trade-off: Small allocation overhead but provides idiomatic Rust API

## Architecture

### Layer Structure

```
┌─────────────────────────────────────────┐
│   Plugin Developer Code (Rust)         │
│   - Uses idiomatic Rust APIs           │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│   Safe Rust Wrapper Layer              │
│   - Type-safe wrappers                 │
│   - Memory management (Drop)           │
│   - Thread safety enforcement          │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│   FFI Bridge Layer (cxx)               │
│   - C++ bridge functions               │
│   - Type conversions                   │
│   - Exception handling                 │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│   JUCE C++ Library                     │
│   - juce_gui_basics                    │
│   - juce_gui_extra                     │
│   - juce_graphics                      │
│   - juce_core                          │
└─────────────────────────────────────────┘
```

### Module Organization


```
nih_plug_juce/
├── src/
│   ├── lib.rs                    # Public API exports
│   ├── bridge.rs                 # cxx bridge definitions
│   ├── component.rs              # Component wrapper
│   ├── graphics.rs               # Graphics context wrapper
│   ├── widgets/
│   │   ├── button.rs             # Button components
│   │   ├── slider.rs             # Slider component
│   │   ├── label.rs              # Label component
│   │   ├── combo_box.rs          # ComboBox component
│   │   ├── text_editor.rs        # TextEditor component
│   │   ├── toggle_button.rs      # ToggleButton component
│   │   └── mod.rs
│   ├── containers/
│   │   ├── document_window.rs    # DocumentWindow wrapper
│   │   ├── resizable_window.rs   # ResizableWindow wrapper
│   │   ├── viewport.rs           # Viewport component
│   │   ├── tabbed_component.rs   # TabbedComponent wrapper
│   │   ├── list_box.rs           # ListBox component
│   │   ├── tree_view.rs          # TreeView component
│   │   └── mod.rs
│   ├── layout/
│   │   ├── flexbox.rs            # FlexBox layout
│   │   └── mod.rs
│   ├── drawing/
│   │   ├── colour.rs             # Colour wrapper
│   │   ├── font.rs               # Font wrapper
│   │   ├── image.rs              # Image wrapper
│   │   ├── path.rs               # Path wrapper
│   │   ├── transform.rs          # AffineTransform wrapper
│   │   ├── drawable.rs           # Drawable classes
│   │   └── mod.rs
│   ├── events/
│   │   ├── mouse.rs              # Mouse event handling
│   │   ├── keyboard.rs           # Keyboard event handling
│   │   ├── timer.rs              # Timer wrapper
│   │   └── mod.rs
│   ├── dialogs/
│   │   ├── alert_window.rs       # AlertWindow wrapper
│   │   ├── file_chooser.rs       # FileChooser wrapper
│   │   └── mod.rs
│   ├── lookandfeel.rs            # LookAndFeel system
│   ├── message_thread.rs         # Message thread utilities
│   ├── parameter_attachment.rs   # Parameter attachment
│   └── error.rs                  # Error types
├── cpp/
│   ├── juce_bridge.h             # C++ bridge header
│   ├── juce_bridge.cpp           # C++ bridge implementation
│   ├── component_bridge.cpp      # Component FFI functions
│   ├── graphics_bridge.cpp       # Graphics FFI functions
│   ├── widget_bridges.cpp        # Widget FFI functions
│   ├── callback_bridge.cpp       # Callback handling
│   └── CMakeLists.txt            # JUCE build configuration
└── build.rs                      # Build script for JUCE compilation
```

## Components and Interfaces

### Core Component System

**Component Wrapper**
```rust
pub struct Component {
    ptr: *mut ffi::JuceComponent,
    _phantom: PhantomData<*mut ()>, // !Send + !Sync
}

impl Component {
    pub fn new() -> Result<Self>;
    pub fn add_child(&mut self, child: &Component) -> Result<()>;
    pub fn remove_child(&mut self, child: &Component) -> Result<()>;
    pub fn set_bounds(&mut self, x: i32, y: i32, width: i32, height: i32);
    pub fn set_visible(&mut self, visible: bool);
    pub fn repaint(&mut self);
    pub fn set_paint_callback<F>(&mut self, callback: F) 
        where F: Fn(&Graphics) + 'static;
}

impl Drop for Component {
    fn drop(&mut self) {
        unsafe { ffi::delete_component(self.ptr) }
    }
}
```

**Design rationale**: 
- PhantomData ensures Component is !Send + !Sync, enforcing message thread usage
- Drop implementation ensures automatic C++ destructor calls
- Callbacks use trait objects for flexibility

### Graphics Context


```rust
pub struct Graphics<'a> {
    ptr: *mut ffi::JuceGraphics,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> Graphics<'a> {
    pub fn fill_rect(&mut self, x: i32, y: i32, width: i32, height: i32);
    pub fn draw_rect(&mut self, x: i32, y: i32, width: i32, height: i32);
    pub fn fill_ellipse(&mut self, x: f32, y: f32, width: f32, height: f32);
    pub fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32);
    pub fn set_colour(&mut self, colour: &Colour);
    pub fn draw_text(&mut self, text: &str, x: i32, y: i32, width: i32, height: i32, 
                     justification: Justification);
    pub fn draw_image_at(&mut self, image: &Image, x: i32, y: i32);
    pub fn stroke_path(&mut self, path: &Path, stroke: &PathStrokeType);
    pub fn fill_path(&mut self, path: &Path);
}
```

**Design rationale**:
- Lifetime parameter prevents Graphics from outliving the paint callback
- No Drop implementation needed (JUCE manages Graphics lifecycle)
- Methods mirror JUCE API but use Rust types

### Widget Components

**Button**
```rust
pub struct TextButton {
    component: Component,
}

impl TextButton {
    pub fn new(text: &str) -> Result<Self>;
    pub fn set_button_text(&mut self, text: &str);
    pub fn set_enabled(&mut self, enabled: bool);
    pub fn set_colour(&mut self, colour_id: ButtonColourId, colour: Colour);
    pub fn set_on_click<F>(&mut self, callback: F) 
        where F: Fn() + 'static;
}

impl Deref for TextButton {
    type Target = Component;
    fn deref(&self) -> &Self::Target { &self.component }
}

impl DerefMut for TextButton {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.component }
}
```

**Slider**
```rust
pub struct Slider {
    component: Component,
}

pub enum SliderStyle {
    Linear,
    Rotary,
    RotaryHorizontalDrag,
    RotaryVerticalDrag,
    TwoValueHorizontal,
    TwoValueVertical,
}

impl Slider {
    pub fn new(style: SliderStyle) -> Result<Self>;
    pub fn set_range(&mut self, min: f64, max: f64, interval: f64);
    pub fn set_value(&mut self, value: f64);
    pub fn get_value(&self) -> f64;
    pub fn set_on_value_change<F>(&mut self, callback: F) 
        where F: Fn(f64) + 'static;
}
```

**Label**
```rust
pub struct Label {
    component: Component,
}

impl Label {
    pub fn new(text: &str) -> Result<Self>;
    pub fn set_text(&mut self, text: &str);
    pub fn set_font(&mut self, font: Font);
    pub fn set_justification(&mut self, justification: Justification);
    pub fn set_editable(&mut self, editable: bool);
    pub fn set_on_text_change<F>(&mut self, callback: F) 
        where F: Fn(&str) + 'static;
}
```

**ComboBox**
```rust
pub struct ComboBox {
    component: Component,
}

impl ComboBox {
    pub fn new() -> Result<Self>;
    pub fn add_item(&mut self, text: &str, item_id: i32);
    pub fn clear(&mut self);
    pub fn set_selected_id(&mut self, item_id: i32);
    pub fn set_selected_index(&mut self, index: i32);
    pub fn get_selected_id(&self) -> i32;
    pub fn set_on_change<F>(&mut self, callback: F) 
        where F: Fn(i32) + 'static;
}
```

**TextEditor**
```rust
pub struct TextEditor {
    component: Component,
}

impl TextEditor {
    pub fn new() -> Result<Self>;
    pub fn set_text(&mut self, text: &str);
    pub fn get_text(&self) -> String;
    pub fn set_multiline(&mut self, multiline: bool);
    pub fn set_readonly(&mut self, readonly: bool);
    pub fn set_on_text_change<F>(&mut self, callback: F) 
        where F: Fn(&str) + 'static;
}
```

**ToggleButton**
```rust
pub struct ToggleButton {
    component: Component,
}

impl ToggleButton {
    pub fn new(text: &str) -> Result<Self>;
    pub fn set_toggle_state(&mut self, state: bool);
    pub fn get_toggle_state(&self) -> bool;
    pub fn set_radio_group_id(&mut self, group_id: i32);
    pub fn set_on_click<F>(&mut self, callback: F) 
        where F: Fn(bool) + 'static;
}
```

### Container Components

**DocumentWindow**
```rust
pub struct DocumentWindow {
    component: Component,
}

impl DocumentWindow {
    pub fn new(title: &str) -> Result<Self>;
    pub fn set_content_owned(&mut self, content: Component);
    pub fn set_visible(&mut self, visible: bool);
    pub fn set_name(&mut self, name: &str);
    pub fn set_on_close<F>(&mut self, callback: F) 
        where F: Fn() -> bool + 'static;
}
```

**ResizableWindow**
```rust
pub struct ResizableWindow {
    component: Component,
}

impl ResizableWindow {
    pub fn new(title: &str) -> Result<Self>;
    pub fn set_resizable(&mut self, resizable: bool);
    pub fn set_resize_limits(&mut self, min_w: i32, min_h: i32, max_w: i32, max_h: i32);
    pub fn set_on_resized<F>(&mut self, callback: F) 
        where F: Fn(i32, i32) + 'static;
}
```

**Viewport**
```rust
pub struct Viewport {
    component: Component,
}

impl Viewport {
    pub fn new() -> Result<Self>;
    pub fn set_viewed_component(&mut self, component: Component);
    pub fn set_view_position(&mut self, x: i32, y: i32);
    pub fn set_scrollbars_shown(&mut self, vertical: bool, horizontal: bool);
    pub fn set_on_visible_area_changed<F>(&mut self, callback: F) 
        where F: Fn() + 'static;
}
```

**TabbedComponent**
```rust
pub struct TabbedComponent {
    component: Component,
}

pub enum TabOrientation {
    Top,
    Bottom,
    Left,
    Right,
}

impl TabbedComponent {
    pub fn new(orientation: TabOrientation) -> Result<Self>;
    pub fn add_tab(&mut self, name: &str, colour: Colour, content: Component);
    pub fn remove_tab(&mut self, index: i32);
    pub fn set_current_tab_index(&mut self, index: i32);
    pub fn set_on_tab_changed<F>(&mut self, callback: F) 
        where F: Fn(i32) + 'static;
}
```

**ListBox**
```rust
pub trait ListBoxModel {
    fn get_num_rows(&self) -> i32;
    fn paint_list_box_item(&self, row: i32, g: &mut Graphics, width: i32, height: i32, 
                           selected: bool);
    fn selected_rows_changed(&mut self, last_row_selected: i32);
}

pub struct ListBox {
    component: Component,
}

impl ListBox {
    pub fn new() -> Result<Self>;
    pub fn set_model(&mut self, model: Box<dyn ListBoxModel>);
    pub fn update_content(&mut self);
}
```

**TreeView**
```rust
pub trait TreeViewItem {
    fn get_num_sub_items(&self) -> i32;
    fn get_sub_item(&self, index: i32) -> Option<Box<dyn TreeViewItem>>;
    fn paint_item(&self, g: &mut Graphics, width: i32, height: i32);
    fn item_clicked(&mut self);
}

pub struct TreeView {
    component: Component,
}

impl TreeView {
    pub fn new() -> Result<Self>;
    pub fn set_root_item(&mut self, root: Box<dyn TreeViewItem>);
}
```

### Layout System

**FlexBox**
```rust
pub struct FlexBox {
    ptr: *mut ffi::JuceFlexBox,
}

pub enum FlexDirection {
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

pub enum FlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}

pub struct FlexItem {
    pub component: Component,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: f32,
    pub min_width: f32,
    pub min_height: f32,
}

impl FlexBox {
    pub fn new() -> Self;
    pub fn set_direction(&mut self, direction: FlexDirection);
    pub fn set_wrap(&mut self, wrap: FlexWrap);
    pub fn add_item(&mut self, item: FlexItem);
    pub fn perform_layout(&mut self, bounds: Rectangle<i32>);
}
```

### Drawing Primitives

**Colour**
```rust
pub struct Colour {
    ptr: *mut ffi::JuceColour,
}

impl Colour {
    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self;
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self;
    pub fn from_hex(hex: &str) -> Result<Self>;
    pub fn to_hex(&self) -> String;
    pub fn with_alpha(&self, alpha: f32) -> Self;
    pub fn brighter(&self, amount: f32) -> Self;
    pub fn darker(&self, amount: f32) -> Self;
    pub fn interpolated_with(&self, other: &Colour, proportion: f32) -> Self;
}
```

**Font**
```rust
pub struct Font {
    ptr: *mut ffi::JuceFont,
}

impl Font {
    pub fn new(size: f32) -> Self;
    pub fn with_typeface(name: &str, size: f32) -> Self;
    pub fn set_bold(&mut self, bold: bool);
    pub fn set_italic(&mut self, italic: bool);
    pub fn set_underline(&mut self, underline: bool);
    pub fn get_string_width(&self, text: &str) -> i32;
    pub fn get_height(&self) -> i32;
    pub fn find_all_typeface_names() -> Vec<String>;
}
```

**Image**
```rust
pub struct Image {
    ptr: *mut ffi::JuceImage,
}

pub enum ImageFormat {
    RGB,
    ARGB,
    SingleChannel,
}

impl Image {
    pub fn new(format: ImageFormat, width: i32, height: i32) -> Self;
    pub fn load_from_file(path: &Path) -> Result<Self>;
    pub fn save_to_file(&self, path: &Path) -> Result<()>;
    pub fn get_graphics_context(&mut self) -> Graphics;
    pub fn apply_blur(&mut self, radius: f32);
}
```

**Path**
```rust
pub struct Path {
    ptr: *mut ffi::JucePath,
}

impl Path {
    pub fn new() -> Self;
    pub fn start_new_sub_path(&mut self, x: f32, y: f32);
    pub fn line_to(&mut self, x: f32, y: f32);
    pub fn quadratic_to(&mut self, cx: f32, cy: f32, x: f32, y: f32);
    pub fn cubic_to(&mut self, cx1: f32, cy1: f32, cx2: f32, cy2: f32, x: f32, y: f32);
    pub fn add_rectangle(&mut self, x: f32, y: f32, width: f32, height: f32);
    pub fn add_ellipse(&mut self, x: f32, y: f32, width: f32, height: f32);
    pub fn add_arc(&mut self, x: f32, y: f32, width: f32, height: f32, 
                   start_angle: f32, end_angle: f32);
    pub fn apply_transform(&mut self, transform: &AffineTransform);
}
```

**AffineTransform**
```rust
pub struct AffineTransform {
    ptr: *mut ffi::JuceAffineTransform,
}

impl AffineTransform {
    pub fn identity() -> Self;
    pub fn translation(dx: f32, dy: f32) -> Self;
    pub fn rotation(angle_radians: f32) -> Self;
    pub fn scale(sx: f32, sy: f32) -> Self;
    pub fn followed_by(&self, other: &AffineTransform) -> Self;
}
```

**Drawable**
```rust
pub struct Drawable {
    ptr: *mut ffi::JuceDrawable,
}

impl Drawable {
    pub fn create_from_svg(svg_data: &str) -> Result<Self>;
    pub fn create_from_image_data(data: &[u8]) -> Result<Self>;
    pub fn draw(&self, g: &mut Graphics, opacity: f32);
    pub fn set_transform_to_fit(&mut self, bounds: Rectangle<f32>);
}

pub struct DrawableButton {
    component: Component,
}

impl DrawableButton {
    pub fn new(name: &str) -> Result<Self>;
    pub fn set_images(&mut self, normal: &Drawable, over: Option<&Drawable>, 
                      down: Option<&Drawable>);
}
```

### Event Handling

**Mouse Events**
```rust
pub struct MouseEvent {
    pub x: i32,
    pub y: i32,
    pub mods: ModifierKeys,
}

pub struct ModifierKeys {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub cmd: bool,
}

pub trait MouseListener {
    fn mouse_down(&mut self, event: &MouseEvent) {}
    fn mouse_drag(&mut self, event: &MouseEvent) {}
    fn mouse_up(&mut self, event: &MouseEvent) {}
    fn mouse_enter(&mut self, event: &MouseEvent) {}
    fn mouse_exit(&mut self, event: &MouseEvent) {}
}

impl Component {
    pub fn set_mouse_listener(&mut self, listener: Box<dyn MouseListener>);
}
```

**Keyboard Events**
```rust
pub struct KeyPress {
    pub key_code: i32,
    pub mods: ModifierKeys,
}

pub trait KeyListener {
    fn key_pressed(&mut self, key: &KeyPress) -> bool;
    fn key_state_changed(&mut self) -> bool { false }
}

impl Component {
    pub fn set_wants_keyboard_focus(&mut self, wants: bool);
    pub fn set_key_listener(&mut self, listener: Box<dyn KeyListener>);
}
```

**Timer**
```rust
pub trait TimerCallback {
    fn timer_callback(&mut self);
}

pub struct Timer {
    ptr: *mut ffi::JuceTimer,
}

impl Timer {
    pub fn new<F>(callback: F) -> Self 
        where F: Fn() + 'static;
    pub fn start(&mut self, interval_ms: i32);
    pub fn stop(&mut self);
    pub fn is_running(&self) -> bool;
}
```

### Dialogs

**AlertWindow**
```rust
pub struct AlertWindow;

impl AlertWindow {
    pub fn show_message_box(title: &str, message: &str);
    pub fn show_message_box_async<F>(title: &str, message: &str, callback: F)
        where F: Fn() + 'static;
    pub fn show_ok_cancel_box<F>(title: &str, message: &str, callback: F)
        where F: Fn(bool) + 'static;
}
```

**FileChooser**
```rust
pub struct FileChooser {
    ptr: *mut ffi::JuceFileChooser,
}

impl FileChooser {
    pub fn new(title: &str, initial_dir: &Path, filters: &str) -> Self;
    pub fn browse_for_file_to_open<F>(&mut self, callback: F)
        where F: Fn(Option<PathBuf>) + 'static;
    pub fn browse_for_file_to_save<F>(&mut self, callback: F)
        where F: Fn(Option<PathBuf>) + 'static;
}
```

### LookAndFeel System


```rust
pub trait LookAndFeelMethods {
    fn draw_button_background(&self, g: &mut Graphics, button: &Component, 
                             colour: &Colour, is_highlighted: bool, is_down: bool);
    fn draw_slider(&self, g: &mut Graphics, x: i32, y: i32, width: i32, height: i32,
                   slider_pos: f32, min_slider_pos: f32, max_slider_pos: f32,
                   style: SliderStyle, slider: &Slider);
    fn draw_label(&self, g: &mut Graphics, label: &Label);
}

pub struct LookAndFeel {
    ptr: *mut ffi::JuceLookAndFeel,
}

impl LookAndFeel {
    pub fn new_v4() -> Self;
    pub fn set_colour(&mut self, colour_id: i32, colour: Colour);
    pub fn find_colour(&self, colour_id: i32) -> Colour;
}

impl Component {
    pub fn set_look_and_feel(&mut self, laf: &LookAndFeel);
}
```

### Parameter Attachment

```rust
pub struct SliderParameterAttachment {
    ptr: *mut ffi::JuceSliderParameterAttachment,
}

impl SliderParameterAttachment {
    pub fn new(slider: &mut Slider, parameter_id: &str) -> Result<Self>;
}
```

**Design rationale**: Parameter attachment is simplified for initial implementation. Full AudioProcessorValueTreeState integration will be added in a future iteration.

### Message Thread Utilities

```rust
pub struct MessageManager;

impl MessageManager {
    pub fn is_message_thread() -> bool;
    pub fn call_async<F>(callback: F) 
        where F: Fn() + Send + 'static;
}

#[macro_export]
macro_rules! assert_message_thread {
    () => {
        debug_assert!(
            MessageManager::is_message_thread(),
            "This operation must be called on the message thread"
        );
    };
}
```

**Design rationale**: 
- `call_async` allows safe UI updates from audio thread
- Debug assertions catch thread violations during development
- Type system prevents most violations at compile time

## Data Models

### FFI Bridge Types

The cxx bridge will define these shared types:

```rust
#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("juce_bridge.h");
        
        // Opaque types
        type JuceComponent;
        type JuceGraphics;
        type JuceColour;
        type JuceFont;
        type JuceImage;
        type JucePath;
        type JuceAffineTransform;
        type JuceFlexBox;
        type JuceTimer;
        type JuceLookAndFeel;
        
        // Component operations
        fn create_component() -> *mut JuceComponent;
        fn delete_component(ptr: *mut JuceComponent);
        fn component_add_child(parent: *mut JuceComponent, child: *mut JuceComponent);
        fn component_remove_child(parent: *mut JuceComponent, child: *mut JuceComponent);
        fn component_set_bounds(ptr: *mut JuceComponent, x: i32, y: i32, w: i32, h: i32);
        fn component_set_visible(ptr: *mut JuceComponent, visible: bool);
        fn component_repaint(ptr: *mut JuceComponent);
        
        // Graphics operations
        fn graphics_fill_rect(g: *mut JuceGraphics, x: i32, y: i32, w: i32, h: i32);
        fn graphics_draw_rect(g: *mut JuceGraphics, x: i32, y: i32, w: i32, h: i32);
        fn graphics_set_colour(g: *mut JuceGraphics, colour: *const JuceColour);
        fn graphics_draw_text(g: *mut JuceGraphics, text: &str, x: i32, y: i32, 
                             w: i32, h: i32, justification: i32);
        
        // Button operations
        fn create_text_button(text: &str) -> *mut JuceComponent;
        fn button_set_text(btn: *mut JuceComponent, text: &str);
        fn button_set_enabled(btn: *mut JuceComponent, enabled: bool);
        
        // Slider operations
        fn create_slider(style: i32) -> *mut JuceComponent;
        fn slider_set_range(slider: *mut JuceComponent, min: f64, max: f64, interval: f64);
        fn slider_set_value(slider: *mut JuceComponent, value: f64);
        fn slider_get_value(slider: *const JuceComponent) -> f64;
        
        // ... additional FFI functions
    }
}
```

### Callback Bridge Pattern

Callbacks are bridged using a trampoline pattern:

```cpp
// C++ side
struct CallbackBridge {
    void* rust_closure;
    void (*invoke)(void*);
};

void button_set_on_click(JuceComponent* btn, CallbackBridge bridge) {
    auto* button = static_cast<juce::TextButton*>(btn);
    button->onClick = [bridge]() {
        bridge.invoke(bridge.rust_closure);
    };
}
```

```rust
// Rust side
pub fn set_on_click<F>(&mut self, callback: F) 
    where F: Fn() + 'static 
{
    let boxed = Box::new(callback);
    let raw = Box::into_raw(boxed);
    
    extern "C" fn trampoline<F: Fn()>(ptr: *mut c_void) {
        let callback = unsafe { &*(ptr as *const F) };
        callback();
    }
    
    let bridge = CallbackBridge {
        rust_closure: raw as *mut c_void,
        invoke: trampoline::<F>,
    };
    
    unsafe { ffi::button_set_on_click(self.component.ptr, bridge) }
}
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*


### Core FFI Properties

**Property 1: Component creation succeeds**
*For any* component type (Button, Slider, Label, etc.), creating the component should return a valid wrapper without crashing.
**Validates: Requirements 1.1, 3.1, 4.1, 5.1, 6.1, 12.1, 18.1, 19.1, 20.1, 21.1, 22.1, 29.1, 34.1**

**Property 2: Component bounds round-trip**
*For any* component and any valid bounds (x, y, width, height), setting the bounds and then getting them back should return the same values.
**Validates: Requirements 1.5**

**Property 3: Parent-child relationship consistency**
*For any* parent component and child component, adding the child then removing it should restore the parent to its original state (child count).
**Validates: Requirements 1.3, 1.4**

**Property 4: Graphics operations don't crash**
*For any* valid graphics operation (fillRect, drawRect, fillEllipse, drawLine, drawText) with valid parameters, the operation should complete without crashing.
**Validates: Requirements 2.2, 2.3, 2.4**

**Property 5: Paint callback invocation**
*For any* component with a paint callback, triggering a repaint should invoke the callback with a valid Graphics context.
**Validates: Requirements 2.1**

### Widget Value Round-Trip Properties

**Property 6: Button text round-trip**
*For any* TextButton and any text string, setting the button text and then getting it back should return the same text.
**Validates: Requirements 3.2**

**Property 7: Button enabled state round-trip**
*For any* button and any boolean state, setting the enabled state and then getting it back should return the same state.
**Validates: Requirements 3.4**

**Property 8: Slider value round-trip**
*For any* slider with a valid range, setting a value within that range and then getting it back should return the same value (within floating point tolerance).
**Validates: Requirements 4.2, 4.3**

**Property 9: Label text round-trip**
*For any* label and any text string, setting the label text and then getting it back should return the same text.
**Validates: Requirements 5.2**

**Property 10: ComboBox selection round-trip**
*For any* ComboBox with items, setting a selected item ID and then getting it back should return the same ID.
**Validates: Requirements 6.3**

**Property 11: TextEditor text round-trip**
*For any* TextEditor and any text string, setting the text and then getting it back should return the same text.
**Validates: Requirements 12.2**

**Property 12: ToggleButton state round-trip**
*For any* ToggleButton and any boolean state, setting the toggle state and then getting it back should return the same state.
**Validates: Requirements 29.2**

### Callback Bridging Properties

**Property 13: Button click callback invocation**
*For any* button with an onClick callback, simulating a click should invoke the callback exactly once.
**Validates: Requirements 3.3**

**Property 14: Slider value change callback invocation**
*For any* slider with an onValueChange callback, changing the slider value should invoke the callback with the new value.
**Validates: Requirements 4.4**

**Property 15: ComboBox change callback invocation**
*For any* ComboBox with an onChange callback, changing the selection should invoke the callback with the new selection ID.
**Validates: Requirements 6.4**

**Property 16: TextEditor change callback invocation**
*For any* TextEditor with an onTextChange callback, changing the text should invoke the callback with the new text.
**Validates: Requirements 12.3**

**Property 17: Mouse event callback invocation**
*For any* component with mouse listeners, simulating mouse events (down, drag, up, enter, exit) should invoke the corresponding callbacks with correct MouseEvent data.
**Validates: Requirements 7.1, 7.2, 7.3, 7.4, 7.5**

**Property 18: Keyboard event callback invocation**
*For any* component with key listeners, simulating key presses should invoke the keyPressed callback with correct KeyPress data.
**Validates: Requirements 8.1**

**Property 19: Timer callback invocation**
*For any* timer with a callback and interval, starting the timer should result in the callback being invoked periodically.
**Validates: Requirements 11.2**

### Drawing and Graphics Properties

**Property 20: Colour creation and conversion**
*For any* valid RGBA values, creating a Colour and converting to hex and back should preserve the color values (within rounding tolerance).
**Validates: Requirements 13.1, 13.2**

**Property 21: Colour transformations preserve relationships**
*For any* colour, applying brighter() then darker() with the same amount should return approximately the original colour.
**Validates: Requirements 13.4**

**Property 22: Font attribute round-trip**
*For any* font with specific size and style (bold, italic), setting these attributes and querying them should return the same values.
**Validates: Requirements 14.1, 14.3**

**Property 23: Image load-save round-trip**
*For any* valid image format and dimensions, creating an image, saving it to a file, and loading it back should preserve the image dimensions and format.
**Validates: Requirements 15.1, 15.4**

**Property 24: Path operations don't crash**
*For any* sequence of valid path operations (lineTo, quadraticTo, cubicTo, addRectangle, addEllipse), the operations should complete without crashing.
**Validates: Requirements 31.2, 31.3**

**Property 25: AffineTransform composition**
*For any* transform, composing it with its inverse should result in the identity transform (within floating point tolerance).
**Validates: Requirements 32.2, 32.3, 32.4, 32.5**

### Layout Properties

**Property 26: FlexBox layout consistency**
*For any* FlexBox with items and target bounds, performing layout should position all items within the target bounds.
**Validates: Requirements 10.3**

**Property 27: FlexBox item addition**
*For any* FlexBox, adding N items should result in the FlexBox containing exactly N items.
**Validates: Requirements 10.2**

### Container and Dialog Properties

**Property 28: Viewport scroll position round-trip**
*For any* Viewport and any valid scroll position, setting the view position and then getting it back should return the same position.
**Validates: Requirements 21.3**

**Property 29: TabbedComponent tab management**
*For any* TabbedComponent, adding N tabs then removing M tabs (M ≤ N) should result in exactly N-M tabs remaining.
**Validates: Requirements 22.2, 22.3**

**Property 30: ListBox model integration**
*For any* ListBox with a model, the number of rows painted should equal the number returned by getNumRows().
**Validates: Requirements 19.2, 19.3, 19.4**

### Thread Safety Properties

**Property 31: Message thread callback execution**
*For any* callback posted via MessageManager::callAsync from a non-message thread, the callback should execute on the message thread.
**Validates: Requirements 17.2, 17.3**

### Exception Handling Properties

**Property 32: C++ exception conversion**
*For any* FFI operation that triggers a C++ exception, the exception should be caught at the FFI boundary and converted to a Rust Result::Err.
**Validates: Requirements 16.5**

### Parameter Attachment Properties

**Property 33: Slider parameter bidirectional sync**
*For any* slider with a parameter attachment, changing the slider value should update the parameter, and changing the parameter should update the slider value.
**Validates: Requirements 30.2, 30.3**

### Drawable Properties

**Property 34: SVG loading succeeds**
*For any* valid SVG data, creating a Drawable from the SVG should succeed without crashing.
**Validates: Requirements 25.1**

**Property 35: Drawable drawing doesn't crash**
*For any* Drawable and Graphics context, calling draw() should complete without crashing.
**Validates: Requirements 25.2**

## Error Handling

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum JuceError {
    #[error("FFI call failed: {0}")]
    FfiError(String),
    
    #[error("C++ exception: {0}")]
    CppException(String),
    
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    
    #[error("Component not found")]
    ComponentNotFound,
    
    #[error("File operation failed: {0}")]
    FileError(#[from] std::io::Error),
    
    #[error("Image format not supported: {0}")]
    UnsupportedImageFormat(String),
    
    #[error("Thread safety violation: operation must be called on message thread")]
    ThreadSafetyViolation,
}

pub type Result<T> = std::result::Result<T, JuceError>;
```

### Exception Handling Strategy

All C++ exceptions must be caught at the FFI boundary:

```cpp
// C++ bridge function
extern "C" int component_set_bounds_safe(JuceComponent* ptr, int x, int y, int w, int h, 
                                         char* error_buf, size_t error_buf_len) {
    try {
        auto* comp = static_cast<juce::Component*>(ptr);
        comp->setBounds(x, y, w, h);
        return 0; // Success
    } catch (const std::exception& e) {
        snprintf(error_buf, error_buf_len, "%s", e.what());
        return -1; // Error
    } catch (...) {
        snprintf(error_buf, error_buf_len, "Unknown C++ exception");
        return -1;
    }
}
```

```rust
// Rust wrapper
pub fn set_bounds(&mut self, x: i32, y: i32, width: i32, height: i32) -> Result<()> {
    let mut error_buf = vec![0u8; 256];
    let result = unsafe {
        ffi::component_set_bounds_safe(
            self.ptr,
            x, y, width, height,
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
```

### Thread Safety Enforcement

```rust
// Debug assertions in all GUI methods
pub fn set_bounds(&mut self, x: i32, y: i32, width: i32, height: i32) -> Result<()> {
    assert_message_thread!();
    // ... implementation
}

// Safe cross-thread UI updates
pub fn update_from_audio_thread(value: f32) {
    MessageManager::call_async(move || {
        // This closure runs on message thread
        // Safe to update UI here
    });
}
```

## Testing Strategy

### Unit Testing

Unit tests will verify specific FFI operations and edge cases:

- Component creation and destruction
- Setting and getting widget values
- Callback registration and invocation
- Error handling for invalid inputs
- Thread safety assertions

Example unit test:
```rust
#[test]
fn test_button_text_round_trip() {
    let mut button = TextButton::new("Initial").unwrap();
    button.set_button_text("Updated");
    assert_eq!(button.get_button_text(), "Updated");
}

#[test]
fn test_invalid_bounds() {
    let mut component = Component::new().unwrap();
    // Negative dimensions should be handled gracefully
    let result = component.set_bounds(0, 0, -100, -100);
    assert!(result.is_err());
}
```

### Property-Based Testing

Property-based tests will use the `proptest` crate to verify universal properties across many randomly generated inputs.

**Testing framework**: proptest (Rust property-based testing library)
**Configuration**: Each property test should run a minimum of 100 iterations

Example property test:
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn property_slider_value_round_trip(value in 0.0f64..1.0f64) {
        // Feature: juce-ffi-integration, Property 8: Slider value round-trip
        let mut slider = Slider::new(SliderStyle::Linear).unwrap();
        slider.set_range(0.0, 1.0, 0.01);
        slider.set_value(value);
        let retrieved = slider.get_value();
        prop_assert!((value - retrieved).abs() < 0.001);
    }
    
    #[test]
    fn property_component_creation_succeeds(
        component_type in prop_oneof![
            Just("button"),
            Just("slider"),
            Just("label"),
        ]
    ) {
        // Feature: juce-ffi-integration, Property 1: Component creation succeeds
        let result = match component_type {
            "button" => TextButton::new("Test").map(|_| ()),
            "slider" => Slider::new(SliderStyle::Linear).map(|_| ()),
            "label" => Label::new("Test").map(|_| ()),
            _ => unreachable!(),
        };
        prop_assert!(result.is_ok());
    }
}
```

### Integration Testing

Integration tests will verify end-to-end scenarios:

- Creating complete UI hierarchies
- Event flow from user interaction to callback
- Integration with nih-plug plugin system
- Cross-platform compatibility (Windows, macOS, Linux)

### Performance Testing

While not correctness properties, performance benchmarks will ensure FFI overhead is acceptable:

- Benchmark FFI call overhead vs native C++ JUCE
- Measure callback invocation latency
- Profile memory usage and allocation patterns
- Verify UI responsiveness under load

## Build System Integration

### Build Script (build.rs)

The build script will:

1. Detect platform (Windows, macOS, Linux)
2. Find or download JUCE library
3. Configure CMake for JUCE compilation
4. Compile selected JUCE modules (juce_gui_basics, juce_gui_extra, juce_graphics, juce_core)
5. Link JUCE static library
6. Generate cxx bridge code

```rust
// build.rs
fn main() {
    // Detect platform
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    
    // Configure JUCE build
    let juce_modules = vec![
        "juce_core",
        "juce_graphics",
        "juce_gui_basics",
        "juce_gui_extra",
    ];
    
    // Build JUCE using CMake
    let dst = cmake::Config::new("cpp")
        .define("JUCE_MODULES", juce_modules.join(";"))
        .build();
    
    // Link JUCE library
    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=juce");
    
    // Platform-specific system libraries
    match target_os.as_str() {
        "macos" => {
            println!("cargo:rustc-link-lib=framework=Cocoa");
            println!("cargo:rustc-link-lib=framework=CoreFoundation");
            println!("cargo:rustc-link-lib=framework=CoreGraphics");
        }
        "linux" => {
            println!("cargo:rustc-link-lib=X11");
            println!("cargo:rustc-link-lib=Xext");
            println!("cargo:rustc-link-lib=freetype");
        }
        "windows" => {
            println!("cargo:rustc-link-lib=gdi32");
            println!("cargo:rustc-link-lib=user32");
            println!("cargo:rustc-link-lib=comdlg32");
        }
        _ => {}
    }
    
    // Generate cxx bridge
    cxx_build::bridge("src/bridge.rs")
        .file("cpp/juce_bridge.cpp")
        .flag_if_supported("-std=c++17")
        .compile("juce_bridge");
}
```

### CMakeLists.txt for JUCE

```cmake
cmake_minimum_required(VERSION 3.15)
project(JuceFFI)

# Add JUCE
add_subdirectory(JUCE)

# Create library with selected modules
add_library(juce STATIC
    juce_bridge.cpp
    component_bridge.cpp
    graphics_bridge.cpp
    widget_bridges.cpp
    callback_bridge.cpp
)

target_link_libraries(juce
    juce::juce_core
    juce::juce_graphics
    juce::juce_gui_basics
    juce::juce_gui_extra
)

# Platform-specific settings
if(APPLE)
    target_compile_definitions(juce PRIVATE JUCE_MAC=1)
elseif(WIN32)
    target_compile_definitions(juce PRIVATE JUCE_WINDOWS=1)
elseif(UNIX)
    target_compile_definitions(juce PRIVATE JUCE_LINUX=1)
endif()
```

### Cargo Features for Modular Builds

```toml
[features]
default = ["gui_basics"]
gui_basics = []
gui_extra = ["gui_basics"]
graphics = []
full = ["gui_basics", "gui_extra", "graphics"]
```

## Performance Considerations

### FFI Overhead Minimization

1. **Inline FFI functions**: Mark frequently-called FFI functions as inline
2. **Batch operations**: Provide batch APIs for operations like adding multiple children
3. **Zero-copy where safe**: Pass references instead of copying data across FFI boundary
4. **Lazy initialization**: Defer expensive operations until needed

### Memory Management

1. **RAII pattern**: Use Drop trait for automatic cleanup
2. **Reference counting**: Use Rc/Arc where shared ownership is needed
3. **Arena allocation**: Consider arena allocator for temporary GUI objects
4. **Leak detection**: Provide debug mode with leak detection

### Callback Optimization

1. **Callback caching**: Cache callback pointers to avoid repeated allocations
2. **Trampoline optimization**: Use efficient trampoline functions for callbacks
3. **Event batching**: Batch multiple events when possible

## Documentation Plan

### API Documentation

- Comprehensive rustdoc for all public APIs
- Code examples for common use cases
- Thread safety requirements clearly documented
- Performance characteristics noted where relevant

### Examples

1. **Basic button example**: Simple plugin with a button
2. **Custom drawing example**: Plugin with custom graphics
3. **Complex layout example**: Plugin using FlexBox layout
4. **Parameter attachment example**: Slider connected to audio parameter

### Migration Guide

For developers familiar with JUCE C++:
- Mapping of C++ classes to Rust types
- Differences in ownership model
- Thread safety considerations
- Common patterns and idioms

## Future Enhancements

### Phase 2 Features

- Full AudioProcessorValueTreeState integration
- Advanced LookAndFeel customization
- OpenGL rendering support
- Additional juce_gui_extra components
- Accessibility support

### Performance Optimizations

- Profile-guided optimization of FFI layer
- SIMD optimizations for graphics operations
- Lazy component initialization
- Component pooling for frequently created/destroyed components

## Risks and Mitigations

### Risk 1: FFI Complexity
- **Mitigation**: Use cxx crate for safe FFI, comprehensive testing, clear documentation

### Risk 2: Thread Safety Violations
- **Mitigation**: Enforce through type system (!Send/!Sync), runtime assertions in debug mode

### Risk 3: Memory Leaks
- **Mitigation**: RAII pattern with Drop, leak detection in debug mode, thorough testing

### Risk 4: Platform Differences
- **Mitigation**: Extensive cross-platform testing, platform-specific test suites

### Risk 5: JUCE Version Compatibility
- **Mitigation**: Pin to specific JUCE version, document compatibility, provide upgrade path

### Risk 6: Build Complexity
- **Mitigation**: Comprehensive build script, clear error messages, pre-built binaries for common platforms
