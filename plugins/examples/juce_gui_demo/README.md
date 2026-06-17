# JUCE GUI Demo

This example demonstrates using the ported JUCE GUI components in a nih-plug plugin.

## Features

- **Component System**: Uses the ported JUCE component hierarchy
- **UI Controls**: Demonstrates Button, Slider, and Label components
- **LookAndFeel**: Shows theme customization with dark/light themes
- **Layout Management**: Demonstrates bounds-based positioning

## Components Demonstrated

- **Label**: Text display with alignment and font size control
- **Slider**: Value control with horizontal/vertical orientation
- **Button**: Interactive button with state management
- **Component**: Base component with parent-child relationships

## Building

```bash
cargo xtask bundle juce_gui_demo --release
```

## Usage

This plugin demonstrates the basic usage of the `logic_nih_plug_gui` crate's component system. While the plugin itself uses standard nih-plug parameter automation, the code includes an example function `create_example_gui()` that shows how to:

1. Create and configure GUI components
2. Set up component hierarchies
3. Apply themes using LookAndFeel
4. Position components using bounds

## Note

This example focuses on demonstrating the ported JUCE GUI component API. Full integration with nih-plug's editor system would require additional implementation using one of the supported GUI frameworks (egui, iced, or vizia) as a rendering backend.

The `create_example_gui()` function serves as a reference for how to use the ported components in your own plugins.
