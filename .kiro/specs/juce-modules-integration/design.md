# Design Document

## Overview

This design document outlines the architecture for porting JUCE modules to native Rust code within the nih-plug framework. Rather than creating FFI bindings to C++, we will translate JUCE's algorithms and functionality directly to idiomatic Rust. This approach provides:

- **No C++ dependencies**: Pure Rust implementation
- **Better performance**: No FFI overhead, better optimization opportunities
- **Rust safety**: Leverage Rust's type system and borrow checker
- **Easier maintenance**: No build complexity from C++ interop
- **Better integration**: Direct use of nih-plug types and patterns

The port will analyze all 23 JUCE modules and implement functionality that adds value to nih-plug, skipping features that nih-plug already provides (like plugin wrappers, parameter management, etc.).

## Architecture

### Module Selection Strategy

Before porting, we analyze each JUCE module to determine if it should be ported:

**Port Priority:**
1. **High Priority** - Unique functionality not in nih-plug or Rust ecosystem
2. **Medium Priority** - Useful but alternatives exist
3. **Skip** - Duplicates nih-plug functionality or not relevant

**Module Analysis:**

| JUCE Module | Priority | Rationale |
|-------------|----------|-----------|
| juce_dsp | High | DSP algorithms (filters, oscillators, convolution) |
| juce_audio_formats | High | Audio file I/O (WAV, AIFF, FLAC, OGG) |
| juce_graphics | High | 2D drawing primitives |
| juce_gui_basics | High | UI components for plugin GUIs |
| juce_data_structures | Medium | ValueTree, UndoManager |
| juce_osc | Medium | OSC networking |
| juce_cryptography | Medium | Hashing, encryption |
| juce_animation | Medium | UI animations |
| juce_midi_ci | Medium | MIDI 2.0 capabilities |
| juce_core | Low | Most functionality in Rust std |
| juce_events | Low | Rust has async/await |
| juce_audio_processors | Skip | nih-plug provides this |
| juce_audio_plugin_client | Skip | nih-plug provides this |
| juce_audio_devices | Skip | Not needed for plugins |
| juce_audio_utils | Skip | Host-specific, not for plugins |

### Crate Organization

Each ported module becomes a separate Rust crate:

```
nih-plug/
├── nih_plug_dsp/          # Ported from juce_dsp
├── nih_plug_audio_formats/ # Ported from juce_audio_formats  
├── nih_plug_graphics/      # Ported from juce_graphics
├── nih_plug_gui/           # Ported from juce_gui_basics
├── nih_plug_data/          # Ported from juce_data_structures
├── nih_plug_osc/           # Ported from juce_osc
├── nih_plug_crypto/        # Ported from juce_cryptography
├── nih_plug_animation/     # Ported from juce_animation
└── nih_plug_midi_ci/       # Ported from juce_midi_ci
```

### Porting Process

For each module:

1. **Analyze**: Study JUCE C++ source code
2. **Design**: Plan Rust API that's idiomatic
3. **Implement**: Translate algorithms to Rust
4. **Test**: Property-based tests for correctness
5. **Benchmark**: Ensure performance is comparable
6. **Document**: Comprehensive rustdoc


## Components and Interfaces

### nih_plug_dsp Module

Ports JUCE's DSP algorithms to pure Rust.

**Key Components:**

```rust
// IIR Filter
pub struct IIRFilter {
    coefficients: Vec<f32>,
    state: Vec<f32>,
    sample_rate: f32,
}

impl IIRFilter {
    pub fn new() -> Self;
    pub fn set_coefficients(&mut self, coeffs: &[f32]) -> Result<()>;
    pub fn process(&mut self, input: &[f32], output: &mut [f32]);
    pub fn process_buffer(&mut self, buffer: &mut Buffer);
    pub fn reset(&mut self);
}

// Oscillator
pub enum Waveform {
    Sine,
    Saw,
    Square,
    Triangle,
}

pub struct Oscillator {
    waveform: Waveform,
    phase: f32,
    frequency: f32,
    sample_rate: f32,
}

impl Oscillator {
    pub fn new(sample_rate: f32) -> Self;
    pub fn set_frequency(&mut self, freq: f32);
    pub fn set_waveform(&mut self, waveform: Waveform);
    pub fn process(&mut self, output: &mut [f32]);
    pub fn reset(&mut self);
}

// Convolution
pub struct Convolution {
    impulse_response: Vec<f32>,
    fft_size: usize,
    // FFT buffers
}

impl Convolution {
    pub fn new() -> Self;
    pub fn load_impulse_response(&mut self, ir: &[f32], sample_rate: f32) -> Result<()>;
    pub fn process(&mut self, input: &[f32], output: &mut [f32]);
}

// ADSR Envelope
pub struct Envelope {
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    state: EnvelopeState,
}

impl Envelope {
    pub fn new() -> Self;
    pub fn set_adsr(&mut self, a: f32, d: f32, s: f32, r: f32);
    pub fn note_on(&mut self);
    pub fn note_off(&mut self);
    pub fn get_next_sample(&mut self) -> f32;
}
```

### nih_plug_audio_formats Module

Ports JUCE's audio file I/O.

```rust
pub struct AudioFileReader {
    format: AudioFormat,
    sample_rate: f32,
    num_channels: usize,
    num_samples: usize,
}

pub enum AudioFormat {
    Wav,
    Aiff,
    Flac,
    Ogg,
}

impl AudioFileReader {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self>;
    pub fn read_samples(&mut self, buffer: &mut [Vec<f32>]) -> Result<usize>;
    pub fn read_all(&mut self) -> Result<Vec<Vec<f32>>>;
    pub fn sample_rate(&self) -> f32;
    pub fn num_channels(&self) -> usize;
    pub fn num_samples(&self) -> usize;
}

pub struct AudioFileWriter {
    format: AudioFormat,
    sample_rate: f32,
    num_channels: usize,
}

impl AudioFileWriter {
    pub fn create<P: AsRef<Path>>(
        path: P,
        format: AudioFormat,
        sample_rate: f32,
        num_channels: usize,
    ) -> Result<Self>;
    
    pub fn write_samples(&mut self, samples: &[Vec<f32>]) -> Result<()>;
}
```

### nih_plug_graphics Module

Ports JUCE's 2D graphics primitives.

```rust
pub struct Graphics {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Graphics {
    pub fn new(width: u32, height: u32) -> Self;
    pub fn set_color(&mut self, color: Color);
    pub fn fill_rect(&mut self, x: i32, y: i32, width: u32, height: u32);
    pub fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32);
    pub fn draw_circle(&mut self, x: i32, y: i32, radius: u32);
    pub fn draw_text(&mut self, text: &str, x: i32, y: i32, font_size: u32);
    pub fn as_bytes(&self) -> &[u8];
}
```

### nih_plug_data Module

Ports JUCE's data structures.

```rust
// ValueTree - hierarchical data structure
pub struct ValueTree {
    type_name: String,
    properties: HashMap<String, Value>,
    children: Vec<ValueTree>,
    listeners: Vec<Box<dyn ValueTreeListener>>,
}

pub enum Value {
    Int(i32),
    Float(f32),
    String(String),
    Bool(bool),
}

pub trait ValueTreeListener {
    fn value_changed(&mut self, tree: &ValueTree, property: &str);
    fn child_added(&mut self, parent: &ValueTree, child: &ValueTree);
    fn child_removed(&mut self, parent: &ValueTree, child: &ValueTree);
}

impl ValueTree {
    pub fn new(type_name: &str) -> Self;
    pub fn set_property(&mut self, name: &str, value: Value);
    pub fn get_property(&self, name: &str) -> Option<&Value>;
    pub fn add_child(&mut self, child: ValueTree);
    pub fn remove_child(&mut self, index: usize) -> Option<ValueTree>;
    pub fn add_listener(&mut self, listener: Box<dyn ValueTreeListener>);
    pub fn to_xml(&self) -> String;
    pub fn from_xml(xml: &str) -> Result<Self>;
}

// UndoManager
pub struct UndoManager {
    undo_stack: Vec<Box<dyn UndoableAction>>,
    redo_stack: Vec<Box<dyn UndoableAction>>,
}

pub trait UndoableAction {
    fn perform(&mut self) -> Result<()>;
    fn undo(&mut self) -> Result<()>;
}

impl UndoManager {
    pub fn new() -> Self;
    pub fn perform(&mut self, action: Box<dyn UndoableAction>) -> Result<()>;
    pub fn undo(&mut self) -> Result<()>;
    pub fn redo(&mut self) -> Result<()>;
    pub fn can_undo(&self) -> bool;
    pub fn can_redo(&self) -> bool;
}
```

## Data Models

### Memory Management

All ported code uses Rust's ownership model:

- **Owned Types**: Standard Rust ownership
- **Borrowed Types**: Standard Rust borrowing with lifetimes
- **Reference Counted**: Use `Arc<T>` for shared ownership when needed

No unsafe code in public APIs. Internal implementations may use unsafe for performance-critical sections, but must be thoroughly documented and tested.

### Thread Safety

Thread safety is enforced through Rust's type system:

```rust
// Thread-safe types implement Send + Sync
pub struct IIRFilter { /* ... */ }
unsafe impl Send for IIRFilter {}
unsafe impl Sync for IIRFilter {}

// UI types are NOT thread-safe
pub struct Component { /* ... */ }
// Deliberately no Send/Sync
```

### Error Handling

All fallible operations return `Result<T, E>`:

```rust
#[derive(Debug, thiserror::Error)]
pub enum DspError {
    #[error("Invalid sample rate: {0}")]
    InvalidSampleRate(f32),
    
    #[error("Invalid buffer size: {0}")]
    InvalidBufferSize(usize),
    
    #[error("Invalid coefficients")]
    InvalidCoefficients,
}

#[derive(Debug, thiserror::Error)]
pub enum AudioFormatError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
    
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property Reflection

Analyzing the requirements for redundancy:

**Redundancy Analysis:**
- Memory safety properties (1.2, 7.1, 7.2) can be unified since Rust handles this automatically
- Error handling properties (8.2, 32.1) are redundant - one property suffices
- Thread safety properties (33.1, 33.4) can be combined
- Performance properties should focus on algorithmic correctness, not FFI overhead

**Consolidated Properties:**

Property 1: Audio buffer processing preserves data
*For any* audio buffer with arbitrary channel count and sample count, processing through ported DSP components should preserve all sample values within numerical precision limits
**Validates: Requirements 1.3**

Property 2: Filter state persistence across process calls
*For any* filter instance and sequence of audio blocks, processing multiple blocks should maintain correct internal state such that the output depends on all previous inputs
**Validates: Requirements 2.3**

Property 3: Reset restores initial state
*For any* stateful DSP object (filter, oscillator, envelope), resetting it should produce the same state as a freshly constructed instance
**Validates: Requirements 2.4, 3.5**

Property 4: Audio file round-trip preserves data
*For any* audio buffer, writing to a file and reading it back should produce equivalent sample values within the precision of the file format
**Validates: Requirements 6.1, 6.2**

Property 5: ValueTree serialization round-trip
*For any* ValueTree structure, serializing to XML and then deserializing should produce an equivalent ValueTree
**Validates: Requirements 16.3**

Property 6: Error handling uses Result types
*For any* fallible operation, the API should return a Result type rather than panicking
**Validates: Requirements 8.2, 32.1**

Property 7: Thread safety enforced by type system
*For any* type that is thread-safe, it should implement Send and Sync traits; for types that are not thread-safe, these traits should not be implemented
**Validates: Requirements 33.1, 33.4**

Property 8: API naming follows Rust conventions
*For any* public function, the name should follow snake_case convention
**Validates: Requirements 8.1**

Property 9: Clone performs deep copy or is not implemented
*For any* type, if Clone is implemented, cloning should produce an independent copy; if deep copying doesn't make sense, Clone should not be implemented
**Validates: Requirements 7.5**

Property 10: ValueTree modifications trigger notifications
*For any* ValueTree with attached listeners, any modification should result in the appropriate change notification being sent to all listeners
**Validates: Requirements 16.2**

Property 11: Documentation completeness
*For any* public API item (function, struct, enum, trait), rustdoc documentation should be present
**Validates: Requirements 9.1**

Property 12: Oscillator phase continuity
*For any* oscillator, changing frequency should maintain phase continuity without clicks or discontinuities
**Validates: Requirements 3.4**

Property 13: Filter coefficient validation
*For any* filter, setting invalid coefficients should return an error rather than producing undefined behavior
**Validates: Requirements 2.2**

Property 14: Modular compilation enforcement
*For any* module that is not included in dependencies, attempting to use its APIs should result in a compile-time error
**Validates: Requirements 35.5**

## Error Handling

### Error Type Hierarchy

```rust
// DSP errors
#[derive(Debug, thiserror::Error)]
pub enum DspError {
    #[error("Invalid sample rate: {0}")]
    InvalidSampleRate(f32),
    
    #[error("Invalid buffer size: {0}")]
    InvalidBufferSize(usize),
    
    #[error("Invalid coefficients")]
    InvalidCoefficients,
    
    #[error("Invalid frequency: {0}")]
    InvalidFrequency(f32),
}

// Audio format errors
#[derive(Debug, thiserror::Error)]
pub enum AudioFormatError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
    
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Invalid audio data")]
    InvalidData,
}

// Graphics errors
#[derive(Debug, thiserror::Error)]
pub enum GraphicsError {
    #[error("Invalid dimensions: {0}x{1}")]
    InvalidDimensions(u32, u32),
    
    #[error("Invalid color value")]
    InvalidColor,
}

// Data structure errors
#[derive(Debug, thiserror::Error)]
pub enum DataError {
    #[error("Invalid XML: {0}")]
    InvalidXml(String),
    
    #[error("Property not found: {0}")]
    PropertyNotFound(String),
    
    #[error("Invalid value type")]
    InvalidValueType,
}
```

## Testing Strategy

### Unit Testing

Unit tests verify specific functionality:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_filter_reset() {
        let mut filter = IIRFilter::new();
        filter.set_coefficients(&[1.0, 0.5, 0.25]).unwrap();
        
        // Process some audio
        let input = vec![1.0; 100];
        let mut output1 = vec![0.0; 100];
        filter.process(&input, &mut output1);
        
        // Reset filter
        filter.reset();
        
        // Process same input again
        let mut output2 = vec![0.0; 100];
        filter.process(&input, &mut output2);
        
        // Outputs should be identical after reset
        assert_eq!(output1, output2);
    }
    
    #[test]
    fn test_oscillator_frequency_change() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_frequency(440.0);
        
        let mut output = vec![0.0; 100];
        osc.process(&mut output);
        
        // Change frequency mid-buffer
        osc.set_frequency(880.0);
        osc.process(&mut output);
        
        // Should not have discontinuities
        for i in 1..output.len() {
            let diff = (output[i] - output[i-1]).abs();
            assert!(diff < 0.5, "Discontinuity detected");
        }
    }
}
```

### Property-Based Testing

Using `proptest` for comprehensive testing:

```rust
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 100,
        ..ProptestConfig::default()
    })]
    
    // **Feature: juce-modules-integration, Property 3: Reset restores initial state**
    #[test]
    fn prop_filter_reset_restores_initial_state(
        coeffs in prop::collection::vec(-1.0f32..=1.0f32, 1..10),
        input in prop::collection::vec(-1.0f32..=1.0f32, 1..100)
    ) {
        let mut filter1 = IIRFilter::new();
        filter1.set_coefficients(&coeffs).unwrap();
        
        let mut filter2 = IIRFilter::new();
        filter2.set_coefficients(&coeffs).unwrap();
        
        // Process with filter2 then reset
        let mut temp = vec![0.0; input.len()];
        filter2.process(&input, &mut temp);
        filter2.reset();
        
        // Both filters should now produce identical output
        let mut output1 = vec![0.0; input.len()];
        let mut output2 = vec![0.0; input.len()];
        filter1.process(&input, &mut output1);
        filter2.process(&input, &mut output2);
        
        for (a, b) in output1.iter().zip(output2.iter()) {
            prop_assert!((a - b).abs() < 1e-6);
        }
    }
    
    // **Feature: juce-modules-integration, Property 4: Audio file round-trip preserves data**
    #[test]
    fn prop_audio_file_roundtrip(
        samples in prop::collection::vec(
            prop::collection::vec(-1.0f32..=1.0f32, 100..1000),
            1..8
        )
    ) {
        let temp_file = "test_output.wav";
        
        // Write
        let mut writer = AudioFileWriter::create(
            temp_file,
            AudioFormat::Wav,
            44100.0,
            samples.len()
        ).unwrap();
        writer.write_samples(&samples).unwrap();
        drop(writer);
        
        // Read
        let mut reader = AudioFileReader::open(temp_file).unwrap();
        let read_samples = reader.read_all().unwrap();
        
        // Compare
        prop_assert_eq!(samples.len(), read_samples.len());
        for (orig_ch, read_ch) in samples.iter().zip(read_samples.iter()) {
            prop_assert_eq!(orig_ch.len(), read_ch.len());
            for (orig, read) in orig_ch.iter().zip(read_ch.iter()) {
                // WAV is 16-bit, so precision is limited
                prop_assert!((orig - read).abs() < 1.0 / 32768.0);
            }
        }
        
        std::fs::remove_file(temp_file).ok();
    }
    
    // **Feature: juce-modules-integration, Property 5: ValueTree serialization round-trip**
    #[test]
    fn prop_valuetree_xml_roundtrip(
        type_name in "[a-zA-Z]{1,20}",
        props in prop::collection::hash_map("[a-zA-Z]{1,10}", 0i32..1000, 0..10)
    ) {
        let mut tree = ValueTree::new(&type_name);
        for (key, val) in props.iter() {
            tree.set_property(key, Value::Int(*val));
        }
        
        let xml = tree.to_xml();
        let restored = ValueTree::from_xml(&xml).unwrap();
        
        prop_assert_eq!(tree.type_name(), restored.type_name());
        for (key, val) in props.iter() {
            let restored_val = restored.get_property(key).unwrap();
            match restored_val {
                Value::Int(i) => prop_assert_eq!(*val, *i),
                _ => prop_assert!(false, "Wrong value type"),
            }
        }
    }
}
```

### Benchmarking

Use `criterion` for performance testing:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_filter(c: &mut Criterion) {
    let mut filter = IIRFilter::new();
    filter.set_coefficients(&[1.0, 0.5, 0.25]).unwrap();
    let input = vec![0.5; 1024];
    let mut output = vec![0.0; 1024];
    
    c.bench_function("iir_filter_1024", |b| {
        b.iter(|| {
            filter.process(black_box(&input), black_box(&mut output))
        });
    });
}

criterion_group!(benches, benchmark_filter);
criterion_main!(benches);
```

## Build System Design

### Cargo Workspace

Organize as a Cargo workspace:

```toml
# Cargo.toml (workspace root)
[workspace]
members = [
    "nih_plug",
    "nih_plug_dsp",
    "nih_plug_audio_formats",
    "nih_plug_graphics",
    "nih_plug_gui",
    "nih_plug_data",
    "nih_plug_osc",
    "nih_plug_crypto",
    "nih_plug_animation",
    "nih_plug_midi_ci",
]

[workspace.dependencies]
thiserror = "1.0"
proptest = "1.0"
criterion = "0.5"
```

### Feature Flags

Each crate supports optional features:

```toml
# nih_plug_dsp/Cargo.toml
[features]
default = ["filters", "oscillators"]
filters = []
oscillators = []
convolution = ["rustfft"]
envelopes = []
full = ["filters", "oscillators", "convolution", "envelopes"]

[dependencies]
rustfft = { version = "6.0", optional = true }
```

## Documentation Strategy

### API Documentation

Comprehensive rustdoc for all public APIs:

```rust
/// An IIR (Infinite Impulse Response) filter.
///
/// This filter processes audio using configurable coefficients and maintains
/// internal state across process calls.
///
/// # Examples
///
/// ```
/// use nih_plug_dsp::IIRFilter;
///
/// let mut filter = IIRFilter::new();
/// filter.set_coefficients(&[1.0, 0.5, 0.25]).unwrap();
///
/// let input = vec![1.0, 0.5, 0.25, 0.0];
/// let mut output = vec![0.0; 4];
/// filter.process(&input, &mut output);
/// ```
///
/// # Performance
///
/// Processing 1024 samples takes approximately 5-10μs on modern CPUs.
///
/// # Thread Safety
///
/// This type is `Send` but not `Sync`. Each thread should have its own instance.
pub struct IIRFilter {
    // ...
}
```

### Module Documentation

```rust
//! # nih_plug_dsp
//!
//! Digital signal processing algorithms ported from JUCE.
//!
//! This crate provides pure Rust implementations of common DSP algorithms:
//!
//! - **Filters**: IIR and FIR filters
//! - **Oscillators**: Sine, saw, square, triangle waveforms
//! - **Convolution**: FFT-based convolution for reverb
//! - **Envelopes**: ADSR envelope generators
//!
//! ## Examples
//!
//! See the `examples/` directory for complete plugin examples.
```

## Performance Considerations

### Optimization Strategies

1. **SIMD**: Use portable SIMD where beneficial
2. **Inlining**: Mark hot paths with `#[inline]`
3. **Zero-copy**: Use slices and references
4. **Buffer reuse**: Minimize allocations

```rust
#[inline]
pub fn process_sample(&mut self, input: f32) -> f32 {
    // Hot path - inline for performance
}
```

### Benchmarking Goals

Target performance equal to or better than JUCE C++:
- Filter processing: < 10μs per 1024 samples
- Oscillator generation: < 5μs per 1024 samples
- File I/O: Comparable to C++ implementations

## Migration Path

For existing nih-plug users:

1. Add ported crates to dependencies
2. Replace custom DSP with ported implementations
3. Update code to use new APIs
4. Test thoroughly

Example:

```rust
// Before: Custom implementation
struct MyFilter { /* ... */ }

// After: Use ported code
use nih_plug_dsp::IIRFilter;

struct MyPlugin {
    filter: IIRFilter,
}
```
