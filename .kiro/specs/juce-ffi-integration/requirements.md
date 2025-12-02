# Requirements Document

## Introduction

This document specifies the requirements for integrating JUCE's GUI components into nih-plug through FFI (Foreign Function Interface) bindings. Rather than porting JUCE's GUI code to pure Rust, this project will create safe Rust wrappers around the actual JUCE C++ GUI library, allowing direct use of JUCE's mature and feature-rich GUI system while maintaining Rust's safety guarantees at the API boundary.

This approach provides access to JUCE's complete GUI ecosystem, including juce_gui_basics (core components), juce_gui_extra (advanced widgets), and juce_graphics (2D rendering). The FFI layer will handle memory management, event callbacks, thread safety, and type conversions between C++ and Rust, while exposing idiomatic Rust APIs to plugin developers.

The integration will focus on enabling nih-plug developers to build professional plugin UIs using JUCE's proven component system, including buttons, sliders, labels, combo boxes, text editors, and custom drawing capabilities.

## Glossary

- **JUCE**: Jules' Utility Class Extensions, a C++ framework for audio applications
- **nih-plug**: A Rust framework for creating audio plugins
- **FFI**: Foreign Function Interface, mechanism for calling C++ code from Rust
- **Binding**: Rust wrapper code that interfaces with C++ through FFI
- **Component**: A GUI element that can be displayed and interacted with (base class for all JUCE UI elements)
- **Graphics Context**: An object used for drawing operations (juce::Graphics)
- **LookAndFeel**: A class that defines the visual appearance of components
- **Message Thread**: The main thread where all GUI operations must occur in JUCE
- **Opaque Pointer**: A pointer to C++ object whose internal structure is hidden from Rust
- **Build Script**: Rust build.rs file that compiles C++ code during cargo build
- **cxx**: A Rust crate for safe C++/Rust interop
- **bindgen**: A tool for automatically generating Rust FFI bindings from C++ headers
- **FlexBox**: A layout system for arranging components responsively
- **Bounds**: The position and size of a component (x, y, width, height)
- **Repaint**: Triggering a component to redraw itself
- **Mouse Event**: User interaction with mouse (click, drag, hover, etc.)
- **Keyboard Event**: User interaction with keyboard (key press, key release, etc.)
- **Timer**: A mechanism for periodic callbacks on the message thread
- **Modal Component**: A component that blocks interaction with other components

## Requirements

### Requirement 1

**User Story:** As a plugin developer, I want to create JUCE components through FFI, so that I can build plugin UIs using JUCE's component system.

#### Acceptance Criteria

1. WHEN a developer creates a Component THEN the system SHALL allocate the C++ juce::Component object and return a safe Rust wrapper
2. WHEN a Component is dropped THEN the system SHALL call the C++ destructor to free resources
3. WHEN a developer adds a child component THEN the system SHALL call the C++ addAndMakeVisible or addChildComponent method
4. WHEN a developer removes a child component THEN the system SHALL call the C++ removeChildComponent method
5. WHEN a developer sets component bounds THEN the system SHALL call the C++ setBounds method with x, y, width, height parameters

### Requirement 2

**User Story:** As a plugin developer, I want to use JUCE's Graphics context for custom drawing, so that I can create custom visualizations.

#### Acceptance Criteria

1. WHEN a developer implements paint callback THEN the system SHALL provide a wrapped juce::Graphics object
2. WHEN a developer draws shapes THEN the system SHALL call C++ fillRect, drawRect, fillEllipse, drawLine methods
3. WHEN a developer sets colors THEN the system SHALL call C++ setColour method with JUCE Colour objects
4. WHEN a developer draws text THEN the system SHALL call C++ drawText method with font and positioning
5. WHEN a developer loads and draws images THEN the system SHALL use C++ juce::Image and drawImageAt methods

### Requirement 3

**User Story:** As a plugin developer, I want to use JUCE's Button component, so that I can add clickable buttons to my UI.

#### Acceptance Criteria

1. WHEN a developer creates a TextButton THEN the system SHALL create the C++ juce::TextButton object
2. WHEN a developer sets button text THEN the system SHALL call the C++ setButtonText method
3. WHEN a button is clicked THEN the system SHALL bridge the C++ onClick callback to a Rust closure
4. WHEN a developer enables/disables a button THEN the system SHALL call the C++ setEnabled method
5. WHEN a developer sets button colors THEN the system SHALL call C++ setColour method for button color IDs

### Requirement 4

**User Story:** As a plugin developer, I want to use JUCE's Slider component, so that I can add parameter controls to my UI.

#### Acceptance Criteria

1. WHEN a developer creates a Slider THEN the system SHALL create the C++ juce::Slider object
2. WHEN a developer sets slider range THEN the system SHALL call the C++ setRange method with min, max, interval
3. WHEN a developer sets slider value THEN the system SHALL call the C++ setValue method
4. WHEN a slider value changes THEN the system SHALL bridge the C++ onValueChange callback to a Rust closure
5. WHEN a developer sets slider style THEN the system SHALL call the C++ setSliderStyle method (Linear, Rotary, etc.)

### Requirement 5

**User Story:** As a plugin developer, I want to use JUCE's Label component, so that I can display text in my UI.

#### Acceptance Criteria

1. WHEN a developer creates a Label THEN the system SHALL create the C++ juce::Label object
2. WHEN a developer sets label text THEN the system SHALL call the C++ setText method
3. WHEN a developer sets text justification THEN the system SHALL call the C++ setJustificationType method
4. WHEN a developer sets font THEN the system SHALL call the C++ setFont method with Font object
5. WHEN a developer makes label editable THEN the system SHALL call the C++ setEditable method and bridge text change callbacks

### Requirement 6

**User Story:** As a plugin developer, I want to use JUCE's ComboBox component, so that I can provide dropdown selection in my UI.

#### Acceptance Criteria

1. WHEN a developer creates a ComboBox THEN the system SHALL create the C++ juce::ComboBox object
2. WHEN a developer adds items THEN the system SHALL call the C++ addItem method with item text and ID
3. WHEN a developer sets selected item THEN the system SHALL call the C++ setSelectedId or setSelectedItemIndex method
4. WHEN selection changes THEN the system SHALL bridge the C++ onChange callback to a Rust closure
5. WHEN a developer clears items THEN the system SHALL call the C++ clear method

### Requirement 7

**User Story:** As a plugin developer, I want to handle mouse events on components, so that I can implement custom interactions.

#### Acceptance Criteria

1. WHEN a mouse button is pressed THEN the system SHALL bridge the C++ mouseDown callback to Rust with MouseEvent data
2. WHEN a mouse is dragged THEN the system SHALL bridge the C++ mouseDrag callback to Rust with MouseEvent data
3. WHEN a mouse button is released THEN the system SHALL bridge the C++ mouseUp callback to Rust with MouseEvent data
4. WHEN a mouse enters a component THEN the system SHALL bridge the C++ mouseEnter callback to Rust
5. WHEN a mouse exits a component THEN the system SHALL bridge the C++ mouseExit callback to Rust

### Requirement 8

**User Story:** As a plugin developer, I want to handle keyboard events on components, so that I can implement keyboard shortcuts.

#### Acceptance Criteria

1. WHEN a key is pressed THEN the system SHALL bridge the C++ keyPressed callback to Rust with KeyPress data
2. WHEN a key is released THEN the system SHALL bridge the C++ keyStateChanged callback to Rust
3. WHEN a developer checks modifier keys THEN the system SHALL provide access to shift, ctrl, alt, cmd state
4. WHEN a developer wants keyboard focus THEN the system SHALL call the C++ setWantsKeyboardFocus method
5. WHEN focus changes THEN the system SHALL bridge the C++ focusGained and focusLost callbacks to Rust

### Requirement 9

**User Story:** As a plugin developer, I want to use JUCE's LookAndFeel system, so that I can customize the appearance of components.

#### Acceptance Criteria

1. WHEN a developer creates a custom LookAndFeel THEN the system SHALL allow subclassing C++ LookAndFeel_V4
2. WHEN a developer sets a component's LookAndFeel THEN the system SHALL call the C++ setLookAndFeel method
3. WHEN a developer overrides drawing methods THEN the system SHALL bridge C++ virtual methods to Rust trait implementations
4. WHEN a developer queries colors THEN the system SHALL call C++ findColour method
5. WHEN a developer sets colors THEN the system SHALL call C++ setColour method for component color IDs

### Requirement 10

**User Story:** As a plugin developer, I want to use JUCE's FlexBox layout, so that I can create responsive layouts.

#### Acceptance Criteria

1. WHEN a developer creates a FlexBox THEN the system SHALL create the C++ juce::FlexBox object
2. WHEN a developer adds flex items THEN the system SHALL call C++ items.add method with FlexItem
3. WHEN a developer performs layout THEN the system SHALL call the C++ performLayout method with target bounds
4. WHEN a developer sets flex direction THEN the system SHALL set C++ flexDirection (row, column, rowReverse, columnReverse)
5. WHEN a developer sets flex properties THEN the system SHALL set C++ flexWrap, justifyContent, alignContent, alignItems

### Requirement 11

**User Story:** As a plugin developer, I want to use JUCE's Timer for periodic updates, so that I can animate UI elements.

#### Acceptance Criteria

1. WHEN a developer starts a timer THEN the system SHALL call the C++ startTimer method with interval in milliseconds
2. WHEN a timer fires THEN the system SHALL bridge the C++ timerCallback to a Rust closure
3. WHEN a developer stops a timer THEN the system SHALL call the C++ stopTimer method
4. WHEN a developer checks timer status THEN the system SHALL call the C++ isTimerRunning method
5. WHEN multiple timers are needed THEN the system SHALL support multiple timer IDs through startTimerHz

### Requirement 12

**User Story:** As a plugin developer, I want to use JUCE's TextEditor component, so that I can provide text input in my UI.

#### Acceptance Criteria

1. WHEN a developer creates a TextEditor THEN the system SHALL create the C++ juce::TextEditor object
2. WHEN a developer sets text THEN the system SHALL call the C++ setText method
3. WHEN text changes THEN the system SHALL bridge the C++ onTextChange callback to a Rust closure
4. WHEN a developer sets multiline mode THEN the system SHALL call the C++ setMultiLine method
5. WHEN a developer sets read-only mode THEN the system SHALL call the C++ setReadOnly method

### Requirement 13

**User Story:** As a plugin developer, I want to use JUCE's Colour system, so that I can work with colors consistently.

#### Acceptance Criteria

1. WHEN a developer creates a Colour THEN the system SHALL create the C++ juce::Colour object from RGBA values
2. WHEN a developer converts to/from hex THEN the system SHALL use C++ Colour::fromString and toString methods
3. WHEN a developer blends colors THEN the system SHALL call C++ interpolatedWith or overlaidWith methods
4. WHEN a developer adjusts brightness THEN the system SHALL call C++ brighter, darker, withBrightness methods
5. WHEN a developer adjusts alpha THEN the system SHALL call C++ withAlpha method

### Requirement 14

**User Story:** As a plugin developer, I want to use JUCE's Font system, so that I can render text with different fonts and styles.

#### Acceptance Criteria

1. WHEN a developer creates a Font THEN the system SHALL create the C++ juce::Font object with size and style
2. WHEN a developer sets font family THEN the system SHALL call C++ Font constructor with typeface name
3. WHEN a developer sets font style THEN the system SHALL call C++ setBold, setItalic, setUnderline methods
4. WHEN a developer measures text THEN the system SHALL call C++ getStringWidth and getHeight methods
5. WHEN a developer queries available fonts THEN the system SHALL use C++ Font::findAllTypefaceNames

### Requirement 15

**User Story:** As a plugin developer, I want to use JUCE's Image class, so that I can load and display images.

#### Acceptance Criteria

1. WHEN a developer loads an image from file THEN the system SHALL use C++ ImageFileFormat::loadFrom
2. WHEN a developer creates an image THEN the system SHALL create C++ juce::Image with format and dimensions
3. WHEN a developer draws to an image THEN the system SHALL use C++ Graphics context from image
4. WHEN a developer saves an image THEN the system SHALL use C++ ImageFileFormat::writeImageToStream
5. WHEN a developer applies effects THEN the system SHALL use C++ ImageEffects methods (blur, sharpen, etc.)

### Requirement 16

**User Story:** As a plugin developer, I want safe memory management for JUCE GUI objects, so that I avoid memory leaks.

#### Acceptance Criteria

1. WHEN a JUCE GUI object is created THEN the system SHALL wrap the C++ pointer in a Rust struct with Drop implementation
2. WHEN a JUCE GUI object goes out of scope THEN the system SHALL automatically call the C++ destructor
3. WHEN a developer accesses a JUCE object THEN the system SHALL prevent use-after-free through Rust's borrow checker
4. WHEN parent-child relationships exist THEN the system SHALL manage ownership correctly (parent owns children in JUCE)
5. WHEN a C++ exception occurs THEN the system SHALL catch it at the FFI boundary and convert to a Rust Result

### Requirement 17

**User Story:** As a plugin developer, I want thread-safe access to JUCE GUI objects, so that I can update UI from audio thread safely.

#### Acceptance Criteria

1. WHEN a JUCE GUI type requires message thread THEN the system SHALL NOT implement Send or Sync traits
2. WHEN a developer needs to update UI from another thread THEN the system SHALL provide MessageManager::callAsync wrapper
3. WHEN a developer posts a message THEN the system SHALL queue the closure for execution on the message thread
4. WHEN a developer checks current thread THEN the system SHALL call C++ MessageManager::getInstance()->isThisTheMessageThread()
5. WHEN a developer violates thread safety THEN the system SHALL prevent compilation through type system

### Requirement 18

**User Story:** As a plugin developer, I want to use JUCE's DocumentWindow, so that I can create standalone windows.

#### Acceptance Criteria

1. WHEN a developer creates a DocumentWindow THEN the system SHALL create the C++ juce::DocumentWindow object
2. WHEN a developer sets window content THEN the system SHALL call the C++ setContentOwned or setContentNonOwned method
3. WHEN a developer shows the window THEN the system SHALL call the C++ setVisible method
4. WHEN a developer sets window title THEN the system SHALL call the C++ setName method
5. WHEN window is closed THEN the system SHALL bridge the C++ closeButtonPressed callback to Rust

### Requirement 19

**User Story:** As a plugin developer, I want to use JUCE's ListBox component, so that I can display scrollable lists.

#### Acceptance Criteria

1. WHEN a developer creates a ListBox THEN the system SHALL create the C++ juce::ListBox object
2. WHEN a developer sets list model THEN the system SHALL bridge C++ ListBoxModel virtual methods to Rust trait
3. WHEN a developer gets row count THEN the system SHALL call the Rust trait's getNumRows implementation
4. WHEN a developer paints row THEN the system SHALL call the Rust trait's paintListBoxItem implementation
5. WHEN a row is selected THEN the system SHALL bridge the C++ selectedRowsChanged callback to Rust

### Requirement 20

**User Story:** As a plugin developer, I want to use JUCE's TreeView component, so that I can display hierarchical data.

#### Acceptance Criteria

1. WHEN a developer creates a TreeView THEN the system SHALL create the C++ juce::TreeView object
2. WHEN a developer sets root item THEN the system SHALL call the C++ setRootItem method with TreeViewItem
3. WHEN a developer implements tree items THEN the system SHALL bridge C++ TreeViewItem virtual methods to Rust trait
4. WHEN a developer expands/collapses items THEN the system SHALL call C++ setOpen method
5. WHEN an item is selected THEN the system SHALL bridge the C++ itemSelectionChanged callback to Rust

### Requirement 21

**User Story:** As a plugin developer, I want to use JUCE's Viewport component, so that I can create scrollable areas.

#### Acceptance Criteria

1. WHEN a developer creates a Viewport THEN the system SHALL create the C++ juce::Viewport object
2. WHEN a developer sets viewed component THEN the system SHALL call the C++ setViewedComponent method
3. WHEN a developer sets scroll position THEN the system SHALL call the C++ setViewPosition method
4. WHEN a developer enables scrollbars THEN the system SHALL call the C++ setScrollBarsShown method
5. WHEN viewport scrolls THEN the system SHALL bridge the C++ visibleAreaChanged callback to Rust

### Requirement 22

**User Story:** As a plugin developer, I want to use JUCE's TabbedComponent, so that I can create tabbed interfaces.

#### Acceptance Criteria

1. WHEN a developer creates a TabbedComponent THEN the system SHALL create the C++ juce::TabbedComponent object
2. WHEN a developer adds a tab THEN the system SHALL call the C++ addTab method with name, color, and content component
3. WHEN a developer removes a tab THEN the system SHALL call the C++ removeTab method
4. WHEN a tab is changed THEN the system SHALL bridge the C++ currentTabChanged callback to Rust
5. WHEN a developer sets tab position THEN the system SHALL call the C++ setOrientation method (top, bottom, left, right)

### Requirement 23

**User Story:** As a plugin developer, I want to use JUCE's AlertWindow, so that I can show dialogs and alerts.

#### Acceptance Criteria

1. WHEN a developer shows an alert THEN the system SHALL call C++ AlertWindow::showMessageBox or showMessageBoxAsync
2. WHEN a developer shows a question THEN the system SHALL call C++ AlertWindow::showOkCancelBox with callback
3. WHEN a developer shows custom dialog THEN the system SHALL create C++ AlertWindow with custom components
4. WHEN a developer adds buttons THEN the system SHALL call the C++ addButton method
5. WHEN a button is clicked THEN the system SHALL bridge the C++ callback to Rust with button index

### Requirement 24

**User Story:** As a plugin developer, I want to use JUCE's FileChooser, so that I can let users select files.

#### Acceptance Criteria

1. WHEN a developer creates a FileChooser THEN the system SHALL create the C++ juce::FileChooser object
2. WHEN a developer shows file browser THEN the system SHALL call the C++ browseForFileToOpen or browseForFileToSave method
3. WHEN a file is selected THEN the system SHALL bridge the C++ callback to Rust with File object
4. WHEN a developer sets file filters THEN the system SHALL pass wildcard patterns to C++ constructor
5. WHEN a developer gets selected file THEN the system SHALL call the C++ getResult method and convert to Rust PathBuf

### Requirement 25

**User Story:** As a plugin developer, I want to use JUCE's Drawable classes, so that I can work with vector graphics.

#### Acceptance Criteria

1. WHEN a developer loads SVG THEN the system SHALL use C++ Drawable::createFromSVG or createFromImageData
2. WHEN a developer draws a Drawable THEN the system SHALL call the C++ draw method with Graphics context
3. WHEN a developer sets Drawable bounds THEN the system SHALL call the C++ setTransformToFit method
4. WHEN a developer creates DrawableButton THEN the system SHALL create C++ DrawableButton with Drawable images
5. WHEN a developer replaces images THEN the system SHALL call C++ setImages method with normal, over, down states

### Requirement 26

**User Story:** As a build system maintainer, I want automated JUCE GUI compilation, so that JUCE is built as part of the Rust build process.

#### Acceptance Criteria

1. WHEN a developer runs cargo build THEN the system SHALL compile JUCE GUI modules (juce_gui_basics, juce_gui_extra, juce_graphics) using build.rs
2. WHEN JUCE is compiled THEN the system SHALL use CMake or direct compiler invocation with appropriate flags
3. WHEN linking occurs THEN the system SHALL link the JUCE static library into the Rust binary
4. WHEN platform-specific code is needed THEN the system SHALL apply correct compiler flags for Windows, macOS, and Linux
5. WHEN JUCE dependencies are missing THEN the system SHALL provide clear error messages about required system libraries

### Requirement 27

**User Story:** As a plugin developer, I want idiomatic Rust APIs for JUCE GUI, so that the FFI layer is transparent.

#### Acceptance Criteria

1. WHEN a developer uses JUCE GUI APIs THEN the system SHALL follow Rust naming conventions (snake_case for functions)
2. WHEN errors can occur THEN the system SHALL use Result types for error handling
3. WHEN a developer configures components THEN the system SHALL use builder patterns where appropriate
4. WHEN a developer sets callbacks THEN the system SHALL accept Rust closures and Box<dyn Fn> traits
5. WHEN a developer queries state THEN the system SHALL return Option types for nullable values

### Requirement 28

**User Story:** As a plugin developer, I want comprehensive documentation for JUCE GUI FFI, so that I understand how to build UIs.

#### Acceptance Criteria

1. WHEN a developer views the documentation THEN the system SHALL provide rustdoc comments for all public GUI APIs
2. WHEN a developer looks for examples THEN the system SHALL include at least three complete plugin UI examples
3. WHEN a developer encounters an error THEN the system SHALL provide clear error messages with context
4. WHEN a developer reads the documentation THEN the system SHALL explain message thread requirements and FFI safety
5. WHEN a developer needs to understand performance THEN the system SHALL document FFI overhead for GUI operations

### Requirement 29

**User Story:** As a plugin developer, I want to use JUCE's ToggleButton component, so that I can add checkboxes and toggle switches.

#### Acceptance Criteria

1. WHEN a developer creates a ToggleButton THEN the system SHALL create the C++ juce::ToggleButton object
2. WHEN a developer sets toggle state THEN the system SHALL call the C++ setToggleState method
3. WHEN toggle state changes THEN the system SHALL bridge the C++ onClick callback to Rust with new state
4. WHEN a developer sets button text THEN the system SHALL call the C++ setButtonText method
5. WHEN a developer creates radio buttons THEN the system SHALL call the C++ setRadioGroupId method

### Requirement 30

**User Story:** As a plugin developer, I want to use JUCE's Slider attachments, so that I can connect sliders to audio parameters.

#### Acceptance Criteria

1. WHEN a developer creates a SliderParameterAttachment THEN the system SHALL create the C++ juce::SliderParameterAttachment object
2. WHEN a slider value changes THEN the system SHALL automatically update the attached parameter through C++
3. WHEN a parameter changes THEN the system SHALL automatically update the slider value through C++
4. WHEN a developer detaches THEN the system SHALL call the C++ destructor to break the connection
5. WHEN a developer uses AudioProcessorValueTreeState THEN the system SHALL integrate with JUCE's parameter system

### Requirement 31

**User Story:** As a plugin developer, I want to use JUCE's Path class for custom shapes, so that I can draw complex vector graphics.

#### Acceptance Criteria

1. WHEN a developer creates a Path THEN the system SHALL create the C++ juce::Path object
2. WHEN a developer adds lines THEN the system SHALL call C++ lineTo, quadraticTo, cubicTo methods
3. WHEN a developer adds shapes THEN the system SHALL call C++ addRectangle, addEllipse, addArc methods
4. WHEN a developer strokes or fills path THEN the system SHALL use Graphics::strokePath or fillPath
5. WHEN a developer transforms path THEN the system SHALL call C++ applyTransform method with AffineTransform

### Requirement 32

**User Story:** As a plugin developer, I want to use JUCE's AffineTransform for transformations, so that I can rotate, scale, and translate graphics.

#### Acceptance Criteria

1. WHEN a developer creates an AffineTransform THEN the system SHALL create the C++ juce::AffineTransform object
2. WHEN a developer applies rotation THEN the system SHALL call the C++ rotated method
3. WHEN a developer applies scaling THEN the system SHALL call the C++ scaled method
4. WHEN a developer applies translation THEN the system SHALL call the C++ translated method
5. WHEN a developer combines transforms THEN the system SHALL call the C++ followedBy method

### Requirement 33

**User Story:** As a plugin developer, I want minimal FFI overhead for GUI operations, so that UI remains responsive.

#### Acceptance Criteria

1. WHEN a developer calls a JUCE GUI method THEN the system SHALL use inline FFI functions where possible
2. WHEN a developer passes data THEN the system SHALL minimize copies across FFI boundary
3. WHEN a developer repaints components THEN the system SHALL achieve performance within 5% of native C++ JUCE
4. WHEN a developer profiles code THEN the system SHALL show clear FFI overhead in profiling tools
5. WHEN a developer handles events THEN the system SHALL provide zero-copy event data where safe

### Requirement 34

**User Story:** As a plugin developer, I want to use JUCE's ResizableWindow, so that I can create resizable plugin windows.

#### Acceptance Criteria

1. WHEN a developer creates a ResizableWindow THEN the system SHALL create the C++ juce::ResizableWindow object
2. WHEN a developer enables resizing THEN the system SHALL call the C++ setResizable method
3. WHEN window is resized THEN the system SHALL bridge the C++ resized callback to Rust
4. WHEN a developer sets size limits THEN the system SHALL call the C++ setResizeLimits method
5. WHEN a developer adds resize corner THEN the system SHALL call the C++ setResizeLimits with corner size

### Requirement 35

**User Story:** As a plugin developer, I want modular JUCE GUI FFI bindings, so that I can include only the GUI modules I need.

#### Acceptance Criteria

1. WHEN a developer specifies dependencies THEN the system SHALL allow selecting juce_gui_basics, juce_gui_extra, juce_graphics separately
2. WHEN a developer builds with a subset of modules THEN the system SHALL only compile and link selected JUCE GUI modules
3. WHEN a developer uses a module THEN the system SHALL automatically include its JUCE dependencies (e.g., juce_core)
4. WHEN a developer checks binary size THEN the system SHALL show the contribution of each JUCE GUI module
5. WHEN a developer disables a module THEN the system SHALL prevent access to its APIs at compile time
