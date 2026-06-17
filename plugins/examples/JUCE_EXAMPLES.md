# JUCE Module Port Examples

This directory contains example plugins demonstrating the ported JUCE modules in nih-plug.

## Available Examples

### 1. juce_dsp_filter - Basic DSP Example

**Location**: `plugins/examples/juce_dsp_filter/`

A simple audio filter plugin demonstrating the `logic_nih_plug_dsp` module.

**Features**:
- IIR filter implementation
- Second-order low-pass filter
- Parameter smoothing
- Stereo processing
- Coefficient calculation using bilinear transform

**Modules Used**:
- `logic_nih_plug_dsp` (filters)

**Build**:
```bash
cargo xtask bundle juce_dsp_filter --release
```

### 2. juce_gui_demo - GUI Components Example

**Location**: `plugins/examples/juce_gui_demo/`

A plugin demonstrating the `logic_nih_plug_gui` component system.

**Features**:
- Component hierarchy management
- Button, Slider, and Label controls
- LookAndFeel theming system
- Bounds-based layout

**Modules Used**:
- `logic_nih_plug_gui` (components)
- `logic_nih_plug_graphics` (color, primitives)

**Build**:
```bash
cargo xtask bundle juce_gui_demo --release
```

**Note**: This example focuses on demonstrating the component API. The `create_example_gui()` function shows how to use the ported components, though full GUI integration would require a rendering backend.

### 3. juce_multi_module - Advanced Multi-Module Example

**Location**: `plugins/examples/juce_multi_module/`

An advanced synthesizer plugin demonstrating multiple ported JUCE modules working together.

**Features**:
- Complete synthesizer engine with oscillators, filters, and envelopes
- MIDI note handling with velocity sensitivity
- ValueTree-based preset management
- Animated filter modulation using easing functions
- SHA-256 preset verification
- Multiple waveform types (sine, saw, square, triangle)

**Modules Used**:
- `logic_nih_plug_dsp` (oscillators, filters, envelopes, smoothing)
- `logic_nih_plug_data` (ValueTree for preset storage)
- `logic_nih_plug_animation` (easing functions)
- `logic_nih_plug_crypto` (SHA-256 hashing)
- `logic_nih_plug_audio_formats` (for future IR loading)

**Build**:
```bash
cargo xtask bundle juce_multi_module --release
```

## Learning Path

We recommend exploring the examples in this order:

1. **Start with juce_dsp_filter**: Learn the basics of using a single ported module
2. **Move to juce_gui_demo**: Understand the component system and UI framework
3. **Study juce_multi_module**: See how multiple modules integrate in a real plugin

## Key Concepts Demonstrated

### DSP Processing
- Initializing DSP components with sample rate
- Calculating filter coefficients
- Processing audio samples
- Resetting filter state

### GUI Components
- Creating and configuring components
- Setting up component hierarchies
- Applying themes with LookAndFeel
- Positioning components using bounds

### Data Management
- Using ValueTree for hierarchical data storage
- Serializing and deserializing presets
- Managing plugin state

### Animation
- Applying easing functions to parameters
- Creating smooth modulation

### Cryptography
- Generating hashes for data verification
- Ensuring preset integrity

## Building All Examples

To build all JUCE module examples at once:

```bash
cargo xtask bundle juce_dsp_filter juce_gui_demo juce_multi_module --release
```

## Testing

Each example can be tested in your DAW or using the standalone wrapper:

```bash
# Run as standalone
cargo run --package juce_multi_module --release
```

## Further Reading

- See individual README files in each example directory for detailed information
- Check the API documentation: `cargo doc --open`
- Review the ported module source code in the respective crate directories

## Contributing

If you create additional examples using the ported JUCE modules, please consider contributing them back to the project!
