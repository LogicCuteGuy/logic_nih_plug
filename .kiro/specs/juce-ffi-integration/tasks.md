# Implementation Plan: JUCE FFI Integration

## Overview
This plan implements FFI bindings to JUCE's C++ GUI library, allowing nih-plug developers to use JUCE's mature GUI components through safe Rust wrappers. The implementation follows an incremental approach, building from core infrastructure to complete widget support.

## Phase 1: Project Setup and Core Infrastructure

- [x] 1. Create nih_plug_juce crate structure
  - Create `nih_plug_juce/` directory with standard Rust crate layout
  - Create `nih_plug_juce/src/lib.rs` with module declarations
  - Create `nih_plug_juce/Cargo.toml` with dependencies (cxx, thiserror)
  - Create `nih_plug_juce/cpp/` directory for C++ bridge code
  - Create `nih_plug_juce/build.rs` for build script
  - Add `nih_plug_juce` to workspace members in root `Cargo.toml`
  - _Requirements: 26.1, 26.2, 26.3, 26.4, 26.5, 35.1, 35.2, 35.3, 35.4, 35.5_

- [x] 2. Set up build system with CMake integration
  - Implement `build.rs` to detect platform (Windows, macOS, Linux)logic@logic-ThinkPad-A285:~/Code/logic_nih_plug$
  - Configure CMake to build JUCE modules (juce_core, juce_graphics, juce_gui_basics)
  - Set up platform-specific compiler flags and system library linking
  - Create `cpp/CMakeLists.txt` for JUCE library compilation
  - Test that `cargo build` successfully compiles and links JUCE
  - _Requirements: 26.1, 26.2, 26.3, 26.4, 26.5_

- [x] 3. Implement error handling infrastructure
  - Create `src/error.rs` with `JuceError` enum covering all error cases
  - Implement `Result<T>` type alias for consistent error handling
  - Add error conversion traits (From implementations)
  - Document error types with rustdoc comments
  - _Requirements: 27.2, 28.1, 28.3_

- [x] 4. Set up cxx bridge foundation
  - Create `src/bridge.rs` with initial cxx::bridge module
  - Define opaque C++ types (JuceComponent, JuceGraphics, etc.)
  - Create `cpp/juce_bridge.h` and `cpp/juce_bridge.cpp` with includes
  - Implement basic exception handling pattern in C++ bridge functions
  - Test that cxx bridge compiles and links correctly
  - _Requirements: 16.5, 27.1_

## Phase 2: Core Component System

- [x] 5. Implement Component wrapper
  - Create `src/component.rs` with Component struct using opaque pointer pattern
  - Implement Drop trait for automatic C++ destructor calls
  - Add PhantomData to enforce !Send + !Sync for thread safety
  - Implement `new()`, `add_child()`, `remove_child()` methods
  - Implement `set_bounds()`, `set_visible()`, `repaint()` methods
  - Add C++ bridge functions in `cpp/component_bridge.cpp`
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 16.1, 16.2, 16.3, 16.4, 17.1_

- [ ]* 5.1 Write property test for component creation
  - **Property 1: Component creation succeeds**
  - **Validates: Requirements 1.1**

- [ ]* 5.2 Write property test for component bounds round-trip
  - **Property 2: Component bounds round-trip**
  - **Validates: Requirements 1.5**

- [ ]* 5.3 Write property test for parent-child relationship
  - **Property 3: Parent-child relationship consistency**
  - **Validates: Requirements 1.3, 1.4**

- [x] 6. Implement Graphics context wrapper
  - Create `src/graphics.rs` with Graphics struct and lifetime parameter
  - Implement drawing methods: `fill_rect()`, `draw_rect()`, `fill_ellipse()`, `draw_line()`
  - Implement `set_colour()`, `draw_text()`, `draw_image_at()` methods
  - Implement `stroke_path()` and `fill_path()` for Path support
  - Add C++ bridge functions in `cpp/graphics_bridge.cpp`
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_

- [ ]* 6.1 Write property test for graphics operations
  - **Property 4: Graphics operations don't crash**
  - **Validates: Requirements 2.2, 2.3, 2.4**

- [x] 7. Implement paint callback system
  - Add `set_paint_callback()` method to Component
  - Implement callback trampoline pattern in C++ bridge
  - Create callback bridge infrastructure for Rust closures
  - Test paint callback invocation with Graphics context
  - _Requirements: 2.1_

- [ ]* 7.1 Write property test for paint callback invocation
  - **Property 5: Paint callback invocation**
  - **Validates: Requirements 2.1**

## Phase 3: Basic Widget Components

- [x] 8. Implement TextButton component
  - Create `src/widgets/button.rs` with TextButton struct
  - Implement Deref/DerefMut to Component for inheritance pattern
  - Implement `new()`, `set_button_text()`, `set_enabled()` methods
  - Implement `set_colour()` for button color customization
  - Implement `set_on_click()` callback with closure support
  - Add C++ bridge functions for button operations
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_

- [ ]* 8.1 Write property test for button text round-trip
  - **Property 6: Button text round-trip**
  - **Validates: Requirements 3.2**

- [ ]* 8.2 Write property test for button enabled state
  - **Property 7: Button enabled state round-trip**
  - **Validates: Requirements 3.4**

- [ ]* 8.3 Write property test for button click callback
  - **Property 13: Button click callback invocation**
  - **Validates: Requirements 3.3**

- [x] 9. Implement Slider component
  - Create `src/widgets/slider.rs` with Slider struct and SliderStyle enum
  - Implement `new()`, `set_range()`, `set_value()`, `get_value()` methods
  - Implement `set_on_value_change()` callback
  - Add support for different slider styles (Linear, Rotary, etc.)
  - Add C++ bridge functions for slider operations
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5_

- [ ]* 9.1 Write property test for slider value round-trip
  - **Property 8: Slider value round-trip**
  - **Validates: Requirements 4.2, 4.3**

- [ ]* 9.2 Write property test for slider value change callback
  - **Property 14: Slider value change callback invocation**
  - **Validates: Requirements 4.4**

- [x] 10. Implement Label component
  - Create `src/widgets/label.rs` with Label struct
  - Implement `new()`, `set_text()`, `set_font()`, `set_justification()` methods
  - Implement `set_editable()` with text change callback support
  - Add C++ bridge functions for label operations
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_

- [ ]* 10.1 Write property test for label text round-trip
  - **Property 9: Label text round-trip**
  - **Validates: Requirements 5.2**

- [x] 11. Implement ComboBox component
  - Create `src/widgets/combo_box.rs` with ComboBox struct
  - Implement `new()`, `add_item()`, `clear()` methods
  - Implement `set_selected_id()`, `set_selected_index()`, `get_selected_id()` methods
  - Implement `set_on_change()` callback
  - Add C++ bridge functions for combo box operations
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_

- [ ]* 11.1 Write property test for combo box selection round-trip
  - **Property 10: ComboBox selection round-trip**
  - **Validates: Requirements 6.3**

- [ ]* 11.2 Write property test for combo box change callback
  - **Property 15: ComboBox change callback invocation**
  - **Validates: Requirements 6.4**

- [x] 12. Checkpoint - Ensure all tests pass
  - Run all property tests and unit tests
  - Verify basic widget functionality works end-to-end
  - Ask the user if questions arise

## Phase 4: Additional Widgets and Input Handling

- [x] 13. Implement TextEditor component
  - Create `src/widgets/text_editor.rs` with TextEditor struct
  - Implement `new()`, `set_text()`, `get_text()` methods
  - Implement `set_multiline()`, `set_readonly()` methods
  - Implement `set_on_text_change()` callback
  - Add C++ bridge functions for text editor operations
  - _Requirements: 12.1, 12.2, 12.3, 12.4, 12.5_

- [ ]* 13.1 Write property test for text editor text round-trip
  - **Property 11: TextEditor text round-trip**
  - **Validates: Requirements 12.2**

- [ ]* 13.2 Write property test for text editor change callback
  - **Property 16: TextEditor change callback invocation**
  - **Validates: Requirements 12.3**

- [x] 14. Implement ToggleButton component
  - Create `src/widgets/toggle_button.rs` with ToggleButton struct
  - Implement `new()`, `set_toggle_state()`, `get_toggle_state()` methods
  - Implement `set_radio_group_id()` for radio button groups
  - Implement `set_on_click()` callback with state parameter
  - Add C++ bridge functions for toggle button operations
  - _Requirements: 29.1, 29.2, 29.3, 29.4, 29.5_

- [ ]* 14.1 Write property test for toggle button state round-trip
  - **Property 12: ToggleButton state round-trip**
  - **Validates: Requirements 29.2**

- [x] 15. Implement mouse event handling
  - Create `src/events/mouse.rs` with MouseEvent and ModifierKeys structs
  - Create MouseListener trait with event methods
  - Implement `set_mouse_listener()` on Component
  - Bridge C++ mouse callbacks (mouseDown, mouseDrag, mouseUp, mouseEnter, mouseExit)
  - Add C++ bridge functions for mouse event handling
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5_

- [ ]* 15.1 Write property test for mouse event callbacks
  - **Property 17: Mouse event callback invocation**
  - **Validates: Requirements 7.1, 7.2, 7.3, 7.4, 7.5**

- [x] 16. Implement keyboard event handling
  - Create `src/events/keyboard.rs` with KeyPress struct
  - Create KeyListener trait with key event methods
  - Implement `set_wants_keyboard_focus()` and `set_key_listener()` on Component
  - Bridge C++ keyboard callbacks (keyPressed, keyStateChanged, focusGained, focusLost)
  - Add C++ bridge functions for keyboard event handling
  - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5_

- [x] 17. Implement Timer component
  - Create `src/events/timer.rs` with Timer struct and TimerCallback trait
  - Implement `new()`, `start()`, `stop()`, `is_running()` methods
  - Bridge C++ timer callback to Rust closure
  - Add C++ bridge functions for timer operations
  - _Requirements: 11.1, 11.2, 11.3, 11.4, 11.5_

- [ ]* 17.1 Write property test for timer callback invocation
  - **Property 19: Timer callback invocation**
  - **Validates: Requirements 11.2**

## Phase 5: Drawing Primitives and Graphics

- [x] 18. Implement Colour system
  - Create `src/drawing/colour.rs` with Colour struct
  - Implement `from_rgba()`, `from_rgb()`, `from_hex()`, `to_hex()` methods
  - Implement color transformation methods: `with_alpha()`, `brighter()`, `darker()`
  - Implement `interpolated_with()` for color blending
  - Add C++ bridge functions for colour operations
  - _Requirements: 13.1, 13.2, 13.3, 13.4, 13.5_

- [ ]* 18.1 Write property test for colour creation and conversion
  - **Property 20: Colour creation and conversion**
  - **Validates: Requirements 13.1, 13.2**

- [ ]* 18.2 Write property test for colour transformations
  - **Property 21: Colour transformations preserve relationships**
  - **Validates: Requirements 13.4**

- [x] 19. Implement Font system
  - Create `src/drawing/font.rs` with Font struct
  - Implement `new()`, `with_typeface()` constructors
  - Implement `set_bold()`, `set_italic()`, `set_underline()` methods
  - Implement `get_string_width()`, `get_height()` for text measurement
  - Implement `find_all_typeface_names()` for font discovery
  - Add C++ bridge functions for font operations
  - _Requirements: 14.1, 14.2, 14.3, 14.4, 14.5_

- [ ]* 19.1 Write property test for font attributes
  - **Property 22: Font attribute round-trip**
  - **Validates: Requirements 14.1, 14.3**

- [x] 20. Implement Image system
  - Create `src/drawing/image.rs` with Image struct and ImageFormat enum
  - Implement `new()`, `load_from_file()`, `save_to_file()` methods
  - Implement `get_graphics_context()` for drawing to images
  - Implement `apply_blur()` and other image effects
  - Add C++ bridge functions for image operations
  - _Requirements: 15.1, 15.2, 15.3, 15.4, 15.5_

- [ ]* 20.1 Write property test for image load-save round-trip
  - **Property 23: Image load-save round-trip**
  - **Validates: Requirements 15.1, 15.4**

- [x] 21. Implement Path system
  - Create `src/drawing/path.rs` with Path struct
  - Implement `new()`, `start_new_sub_path()`, `line_to()` methods
  - Implement `quadratic_to()`, `cubic_to()` for curves
  - Implement `add_rectangle()`, `add_ellipse()`, `add_arc()` for shapes
  - Implement `apply_transform()` for transformations
  - Add C++ bridge functions for path operations
  - _Requirements: 31.1, 31.2, 31.3, 31.4, 31.5_

- [ ]* 21.1 Write property test for path operations
  - **Property 24: Path operations don't crash**
  - **Validates: Requirements 31.2, 31.3**

- [x] 22. Implement AffineTransform system
  - Create `src/drawing/transform.rs` with AffineTransform struct
  - Implement `identity()`, `translation()`, `rotation()`, `scale()` constructors
  - Implement `followed_by()` for transform composition
  - Add C++ bridge functions for transform operations
  - _Requirements: 32.1, 32.2, 32.3, 32.4, 32.5_

- [ ]* 22.1 Write property test for transform composition
  - **Property 25: AffineTransform composition**
  - **Validates: Requirements 32.2, 32.3, 32.4, 32.5**

- [x] 23. Checkpoint - Ensure all tests pass
  - Run all property tests and unit tests for drawing primitives
  - Verify graphics operations work correctly
  - Ask the user if questions arise

## Phase 6: Layout and Container Components

- [x] 24. Implement FlexBox layout system
  - Create `src/layout/flexbox.rs` with FlexBox, FlexItem structs
  - Create FlexDirection, FlexWrap enums
  - Implement `new()`, `set_direction()`, `set_wrap()` methods
  - Implement `add_item()`, `perform_layout()` methods
  - Add C++ bridge functions for flexbox operations
  - _Requirements: 10.1, 10.2, 10.3, 10.4, 10.5_

- [ ]* 24.1 Write property test for flexbox layout consistency
  - **Property 26: FlexBox layout consistency**
  - **Validates: Requirements 10.3**

- [ ]* 24.2 Write property test for flexbox item addition
  - **Property 27: FlexBox item addition**
  - **Validates: Requirements 10.2**

- [x] 25. Implement DocumentWindow component
  - Create `src/containers/document_window.rs` with DocumentWindow struct
  - Implement `new()`, `set_content_owned()`, `set_visible()` methods
  - Implement `set_name()` and `set_on_close()` callback
  - Add C++ bridge functions for document window operations
  - _Requirements: 18.1, 18.2, 18.3, 18.4, 18.5_

- [x] 26. Implement ResizableWindow component
  - Create `src/containers/resizable_window.rs` with ResizableWindow struct
  - Implement `new()`, `set_resizable()`, `set_resize_limits()` methods
  - Implement `set_on_resized()` callback
  - Add C++ bridge functions for resizable window operations
  - _Requirements: 34.1, 34.2, 34.3, 34.4, 34.5_

- [x] 27. Implement Viewport component
  - Create `src/containers/viewport.rs` with Viewport struct
  - Implement `new()`, `set_viewed_component()`, `set_view_position()` methods
  - Implement `set_scrollbars_shown()` and `set_on_visible_area_changed()` callback
  - Add C++ bridge functions for viewport operations
  - _Requirements: 21.1, 21.2, 21.3, 21.4, 21.5_

- [ ]* 27.1 Write property test for viewport scroll position
  - **Property 28: Viewport scroll position round-trip**
  - **Validates: Requirements 21.3**

- [x] 28. Implement TabbedComponent
  - Create `src/containers/tabbed_component.rs` with TabbedComponent struct
  - Create TabOrientation enum
  - Implement `new()`, `add_tab()`, `remove_tab()` methods
  - Implement `set_current_tab_index()` and `set_on_tab_changed()` callback
  - Add C++ bridge functions for tabbed component operations
  - _Requirements: 22.1, 22.2, 22.3, 22.4, 22.5_

- [ ]* 28.1 Write property test for tab management
  - **Property 29: TabbedComponent tab management**
  - **Validates: Requirements 22.2, 22.3**

- [x] 29. Implement ListBox component
  - Create `src/containers/list_box.rs` with ListBox struct and ListBoxModel trait
  - Implement `new()`, `set_model()`, `update_content()` methods
  - Bridge ListBoxModel trait methods to C++ virtual functions
  - Add C++ bridge functions for list box operations
  - _Requirements: 19.1, 19.2, 19.3, 19.4, 19.5_

- [ ]* 29.1 Write property test for list box model integration
  - **Property 30: ListBox model integration**
  - **Validates: Requirements 19.2, 19.3, 19.4**

- [x] 30. Implement TreeView component
  - Create `src/containers/tree_view.rs` with TreeView struct and TreeViewItem trait
  - Implement `new()`, `set_root_item()` methods
  - Bridge TreeViewItem trait methods to C++ virtual functions
  - Add C++ bridge functions for tree view operations
  - _Requirements: 20.1, 20.2, 20.3, 20.4, 20.5_

## Phase 7: Dialogs and Advanced Features

- [x] 31. Implement AlertWindow dialogs
  - Create `src/dialogs/alert_window.rs` with AlertWindow struct
  - Implement `show_message_box()`, `show_message_box_async()` methods
  - Implement `show_ok_cancel_box()` with callback support
  - Add support for custom dialogs with `addButton()`
  - Add C++ bridge functions for alert window operations
  - _Requirements: 23.1, 23.2, 23.3, 23.4, 23.5_

- [x] 32. Implement FileChooser dialogs
  - Create `src/dialogs/file_chooser.rs` with FileChooser struct
  - Implement `new()`, `browse_for_file_to_open()`, `browse_for_file_to_save()` methods
  - Bridge file selection callbacks to Rust with PathBuf conversion
  - Add C++ bridge functions for file chooser operations
  - _Requirements: 24.1, 24.2, 24.3, 24.4, 24.5_

- [x] 33. Implement Drawable system
  - Create `src/drawing/drawable.rs` with Drawable and DrawableButton structs
  - Implement `create_from_svg()`, `create_from_image_data()` methods
  - Implement `draw()`, `set_transform_to_fit()` methods
  - Implement DrawableButton with `set_images()` method
  - Add C++ bridge functions for drawable operations
  - _Requirements: 25.1, 25.2, 25.3, 25.4, 25.5_

- [ ]* 33.1 Write property test for SVG loading
  - **Property 34: SVG loading succeeds**
  - **Validates: Requirements 25.1**

- [ ]* 33.2 Write property test for drawable drawing
  - **Property 35: Drawable drawing doesn't crash**
  - **Validates: Requirements 25.2**

- [x] 34. Implement LookAndFeel system
  - Create `src/lookandfeel.rs` with LookAndFeel struct and LookAndFeelMethods trait
  - Implement `new_v4()`, `set_colour()`, `find_colour()` methods
  - Implement `set_look_and_feel()` on Component
  - Bridge LookAndFeel virtual methods to Rust trait implementations
  - Add C++ bridge functions for look and feel operations
  - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5_

- [x] 35. Implement parameter attachment system
  - Create `src/parameter_attachment.rs` with SliderParameterAttachment struct
  - Implement `new()` to create attachment between slider and parameter
  - Implement bidirectional synchronization (slider ↔ parameter)
  - Add C++ bridge functions for parameter attachment
  - _Requirements: 30.1, 30.2, 30.3, 30.4, 30.5_

- [ ]* 35.1 Write property test for parameter attachment sync
  - **Property 33: Slider parameter bidirectional sync**
  - **Validates: Requirements 30.2, 30.3**

## Phase 8: Thread Safety and Message Thread

- [x] 36. Implement message thread utilities
  - Create `src/message_thread.rs` with MessageManager struct
  - Implement `is_message_thread()` to check current thread
  - Implement `call_async()` for safe cross-thread UI updates
  - Create `assert_message_thread!()` macro for debug assertions
  - Add C++ bridge functions for message manager operations
  - _Requirements: 17.1, 17.2, 17.3, 17.4, 17.5_

- [ ]* 36.1 Write property test for message thread callback execution
  - **Property 31: Message thread callback execution**
  - **Validates: Requirements 17.2, 17.3**

- [x] 37. Add thread safety enforcement to all GUI types
  - Verify all GUI types use PhantomData for !Send + !Sync
  - Add `assert_message_thread!()` calls to all public methods
  - Document thread safety requirements in rustdoc
  - Test that cross-thread usage fails at compile time
  - _Requirements: 17.1, 17.5_

- [x] 38. Implement C++ exception handling for all FFI functions
  - Add exception catching to all C++ bridge functions
  - Convert C++ exceptions to Rust Result types
  - Test exception handling with invalid inputs
  - Document error conditions in rustdoc
  - _Requirements: 16.5_

- [ ]* 38.1 Write property test for exception conversion
  - **Property 32: C++ exception conversion**
  - **Validates: Requirements 16.5**

- [x] 39. Checkpoint - Ensure all tests pass
  - Run complete test suite including all property tests
  - Verify thread safety enforcement works correctly
  - Verify exception handling works correctly
  - Ask the user if questions arise

## Phase 9: Documentation and Examples

- [x] 40. Write comprehensive API documentation
  - Add rustdoc comments to all public types and methods
  - Document thread safety requirements for each type
  - Document FFI overhead and performance characteristics
  - Add code examples to documentation
  - Generate and review documentation with `cargo doc`
  - _Requirements: 28.1, 28.2, 28.4, 28.5_

- [x] 41. Create basic button example plugin
  - Create `plugins/examples/juce_ffi_button/` directory
  - Implement simple plugin with JUCE button using FFI
  - Demonstrate component creation, callbacks, and parameter integration
  - Add README with build and usage instructions
  - _Requirements: 28.2_

- [x] 42. Create custom drawing example plugin
  - Create `plugins/examples/juce_ffi_drawing/` directory
  - Implement plugin with custom graphics using paint callbacks
  - Demonstrate Graphics context usage, colors, fonts, and paths
  - Add README with build and usage instructions
  - _Requirements: 28.2_

- [x] 43. Create complex layout example plugin
  - Create `plugins/examples/juce_ffi_layout/` directory
  - Implement plugin using FlexBox layout with multiple components
  - Demonstrate responsive layout and component hierarchies
  - Add README with build and usage instructions
  - _Requirements: 28.2_

- [x] 44. Write migration guide for JUCE C++ developers
  - Document mapping of C++ classes to Rust types
  - Explain ownership model differences
  - Provide common patterns and idioms
  - Add troubleshooting section
  - _Requirements: 28.1_

## Phase 10: Performance Optimization and Final Testing

- [x] 45. Optimize FFI call overhead
  - Profile FFI call performance vs native C++ JUCE
  - Implement inline FFI functions where possible
  - Minimize data copies across FFI boundary
  - Verify performance is within 5% of native JUCE
  - _Requirements: 33.1, 33.2, 33.3, 33.4, 33.5_

- [x] 46. Add performance benchmarks
  - Create benchmarks for common operations (component creation, drawing, events)
  - Benchmark callback invocation latency
  - Benchmark memory usage and allocation patterns
  - Document performance characteristics
  - _Requirements: 33.3, 33.4_

- [x] 47. Cross-platform testing
  - Test on Windows (MSVC toolchain)
  - Test on macOS (Clang toolchain)
  - Test on Linux (GCC toolchain)
  - Verify all examples build and run on all platforms
  - Document platform-specific issues and workarounds
  - _Requirements: 26.4_

- [x] 48. Final checkpoint - Complete test suite
  - Run all unit tests across all platforms
  - Run all property tests with 100+ iterations
  - Run all benchmarks and verify performance
  - Run all example plugins and verify functionality
  - Review and finalize documentation
  - Ask the user if questions arise

## Notes

- Tasks marked with `*` are optional testing tasks that can be skipped for faster MVP development
- Each property test should run a minimum of 100 iterations as specified in the design
- Property tests must include the exact comment format: `**Feature: juce-ffi-integration, Property N: [property text]**`
- All GUI operations must be performed on the message thread - enforced through type system
- Exception handling at FFI boundary is critical for safety
- Build system must handle platform differences transparently
