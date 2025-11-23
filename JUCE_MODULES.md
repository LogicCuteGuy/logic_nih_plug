# JUCE Modules Integration

This document describes the JUCE modules that have been ported to native Rust for use with nih-plug.

## Overview

Rather than creating FFI bindings to C++, we have translated JUCE's algorithms and functionality directly to idiomatic Rust code. This provides:

- **No C++ dependencies**: Pure Rust implementation
- **Better performance**: No FFI overhead, better optimization opportunities
- **Rust safety**: Leverage Rust's type system and borrow checker
- **Easier maintenance**: No build complexity from C++ interop
- **Better integration**: Direct use of nih-plug types and patterns

## Available Modules

### nih_plug_dsp

Digital signal processing algorithms ported from JUCE's `juce_dsp` module.

**Features:**
- `filters` - IIR and FIR filters (default)
- `oscillators` - Sine, saw, square, triangle waveforms (default)
- `convolution` - FFT-based convolution for reverb
- `envelopes` - ADSR envelope generators
- `smoothing` - Parameter smoothing utilities
- `full` - All features enabled

**Usage:**
```toml
[dependencies]
nih_plug_dsp = { path = "../nih_plug_dsp", features = ["full"] }
```

### nih_plug_audio_formats

Audio file format support ported from JUCE's `juce_audio_formats` module.

**Features:**
- `wav` - WAV file support (default)
- `aiff` - AIFF file support (default)
- `flac` - FLAC file support
- `ogg` - OGG Vorbis file support
- `full` - All features enabled

**Usage:**
```toml
[dependencies]
nih_plug_audio_formats = { path = "../nih_plug_audio_formats", features = ["full"] }
```

### nih_plug_data

Data structures ported from JUCE's `juce_data_structures` module.

**Features:**
- `valuetree` - Hierarchical data structure with change notifications (default)
- `undo` - Undo/redo functionality
- `full` - All features enabled

**Usage:**
```toml
[dependencies]
nih_plug_data = { path = "../nih_plug_data", features = ["full"] }
```

### nih_plug_graphics

2D graphics primitives ported from JUCE's `juce_graphics` module.

**Features:**
- `primitives` - Rectangle, line, circle drawing (default)
- `images` - PNG, JPEG, GIF loading and rendering
- `text` - Font rendering and text layout
- `full` - All features enabled

**Usage:**
```toml
[dependencies]
nih_plug_graphics = { path = "../nih_plug_graphics", features = ["full"] }
```

### nih_plug_gui

GUI components ported from JUCE's `juce_gui_basics` module.

**Features:**
- `components` - Button, Slider, Label, and other UI controls (default)
- `layout` - Layout managers and constraints
- `full` - All features enabled

**Usage:**
```toml
[dependencies]
nih_plug_gui = { path = "../nih_plug_gui", features = ["full"] }
```

### nih_plug_osc

Open Sound Control support ported from JUCE's `juce_osc` module.

**Features:**
- `sender` - Send OSC messages over UDP/TCP (default)
- `receiver` - Receive and parse OSC messages (default)
- `bundles` - Timestamped message groups
- `full` - All features enabled

**Usage:**
```toml
[dependencies]
nih_plug_osc = { path = "../nih_plug_osc", features = ["full"] }
```

### nih_plug_crypto

Cryptography utilities ported from JUCE's `juce_cryptography` module.

**Features:**
- `hashing` - MD5, SHA-256, SHA-512 (default)
- `encryption` - RSA, Blowfish
- `base64` - Base64 encoding/decoding
- `full` - All features enabled

**Usage:**
```toml
[dependencies]
nih_plug_crypto = { path = "../nih_plug_crypto", features = ["full"] }
```

### nih_plug_animation

Animation utilities ported from JUCE's `juce_animation` module.

**Features:**
- `easing` - Various easing functions for smooth animations (default)
- `chaining` - Sequence multiple animations
- `full` - All features enabled

**Usage:**
```toml
[dependencies]
nih_plug_animation = { path = "../nih_plug_animation", features = ["full"] }
```

### nih_plug_midi_ci

MIDI-CI support ported from JUCE's `juce_midi_ci` module.

**Features:**
- `protocol` - MIDI-CI message parsing and generation (default)
- `profiles` - Profile negotiation
- `properties` - Property exchange
- `discovery` - Device discovery
- `full` - All features enabled

**Usage:**
```toml
[dependencies]
nih_plug_midi_ci = { path = "../nih_plug_midi_ci", features = ["full"] }
```

## Development

### Building

Build all modules:
```bash
cargo build --workspace
```

Build a specific module:
```bash
cargo build --package nih_plug_dsp --all-features
```

### Testing

Run all tests:
```bash
cargo test --workspace
```

Run tests for a specific module:
```bash
cargo test --package nih_plug_dsp --all-features
```

Run property-based tests:
```bash
cargo test --package nih_plug_dsp --all-features -- --include-ignored
```

### Benchmarking

Run benchmarks for a module:
```bash
cargo bench --package nih_plug_dsp
```

### Documentation

Generate documentation:
```bash
cargo doc --workspace --all-features --no-deps --open
```

## CI/CD

The JUCE modules have dedicated CI/CD workflows that:

- Build and test on Linux, Windows, and macOS
- Run clippy for linting
- Check code formatting
- Run benchmarks
- Generate documentation

See `.github/workflows/juce_modules.yml` for details.

## Implementation Status

This is the initial project structure setup. Individual modules will be implemented according to the tasks in `.kiro/specs/juce-modules-integration/tasks.md`.

## Contributing

When contributing to JUCE module ports:

1. Follow Rust naming conventions (snake_case)
2. Use Result types for error handling
3. Provide comprehensive rustdoc comments
4. Write property-based tests for correctness
5. Add benchmarks for performance-critical code
6. Ensure thread safety is properly enforced

## License

ISC License - Same as nih-plug
