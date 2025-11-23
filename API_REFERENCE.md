# API Reference: JUCE Ported Modules

This document provides a comprehensive API reference for all ported JUCE modules.

## Table of Contents

- [nih_plug_dsp](#nih_plug_dsp)
- [nih_plug_audio_formats](#nih_plug_audio_formats)
- [nih_plug_data](#nih_plug_data)
- [nih_plug_graphics](#nih_plug_graphics)
- [nih_plug_gui](#nih_plug_gui)
- [nih_plug_osc](#nih_plug_osc)
- [nih_plug_crypto](#nih_plug_crypto)
- [nih_plug_animation](#nih_plug_animation)
- [nih_plug_midi_ci](#nih_plug_midi_ci)

---

## nih_plug_dsp

Digital signal processing algorithms.

### Modules

- `filters` - IIR filter implementations
- `oscillators` - Waveform generators
- `convolution` - FFT-based convolution
- `envelopes` - ADSR envelope generators
- `smoothing` - Parameter smoothing utilities
- `util` - DSP utility functions

### filters::IIRFilter

Infinite Impulse Response filter using Transposed Direct Form II structure.

#### Methods

```rust
// Construction
pub fn new() -> Self

// Configuration
pub fn set_coefficients(&mut self, b_coeffs: &[f32], a_coeffs: &[f32]) -> Result<(), DspError>

// Processing
pub fn process(&mut self, input: &[f32], output: &mut [f32])
pub fn process_sample(&mut self, input: f32) -> f32

// State management
pub fn reset(&mut self)
pub fn reset_to(&mut self, value: f32)

// Queries
pub fn order(&self) -> usize
```

#### Example

```rust
use nih_plug_dsp::filters::IIRFilter;

let mut filter = IIRFilter::new();
filter.set_coefficients(&[1.0, 0.5], &[1.0, -0.5])?;

let input = vec![1.0; 512];
let mut output = vec![0.0; 512];
filter.process(&input, &mut output);
```

### oscillators::Oscillator

Waveform generator with multiple waveform types.

#### Methods

```rust
// Construction
pub fn new(sample_rate: f32) -> Self

// Configuration
pub fn set_waveform(&mut self, waveform: Waveform)
pub fn set_frequency(&mut self, frequency: f32)
pub fn set_sample_rate(&mut self, sample_rate: f32)

// Processing
pub fn process(&mut self, output: &mut [f32])
pub fn process_sample(&mut self) -> f32

// State management
pub fn reset(&mut self)
pub fn set_phase(&mut self, phase: f32)

// Queries
pub fn frequency(&self) -> f32
pub fn phase(&self) -> f32
```

#### Waveform Types

```rust
pub enum Waveform {
    Sine,
    Saw,
    Square,
    Triangle,
}
```

#### Example

```rust
use nih_plug_dsp::oscillators::{Oscillator, Waveform};

let mut osc = Oscillator::new(44100.0);
osc.set_waveform(Waveform::Sine);
osc.set_frequency(440.0);

let mut output = vec![0.0; 512];
osc.process(&mut output);
```

### envelopes::Envelope

ADSR envelope generator.

#### Methods

```rust
// Construction
pub fn new(sample_rate: f32) -> Self

// Configuration
pub fn set_adsr(&mut self, attack: f32, decay: f32, sustain: f32, release: f32)
pub fn set_sample_rate(&mut self, sample_rate: f32)

// Triggering
pub fn note_on(&mut self)
pub fn note_off(&mut self)

// Processing
pub fn get_next_sample(&mut self) -> f32
pub fn process(&mut self, output: &mut [f32])

// State management
pub fn reset(&mut self)

// Queries
pub fn is_active(&self) -> bool
pub fn current_phase(&self) -> EnvelopePhase
```

#### Example

```rust
use nih_plug_dsp::envelopes::Envelope;

let mut envelope = Envelope::new(44100.0);
envelope.set_adsr(0.1, 0.2, 0.7, 0.3);

envelope.note_on();
let value = envelope.get_next_sample();
envelope.note_off();
```

### smoothing::SmoothedValue

Parameter smoothing for click-free automation.

#### Methods

```rust
// Construction
pub fn new(sample_rate: f32, smoothing_time: f32) -> Self

// Configuration
pub fn set_target(&mut self, target: f32)
pub fn skip_to(&mut self, value: f32)
pub fn set_sample_rate(&mut self, sample_rate: f32)

// Processing
pub fn next(&mut self) -> f32
pub fn process(&mut self, output: &mut [f32])

// Queries
pub fn current(&self) -> f32
pub fn target(&self) -> f32
pub fn is_smoothing(&self) -> bool
```

#### Example

```rust
use nih_plug_dsp::smoothing::SmoothedValue;

let mut smoothed = SmoothedValue::new(44100.0, 0.05);
smoothed.set_target(1.0);

let current = smoothed.next();
```

---

## nih_plug_audio_formats

Audio file format support.

### Common Types

```rust
pub enum AudioFormat {
    Wav,
    Aiff,
    Flac,
    Ogg,
}

pub struct AudioMetadata {
    pub sample_rate: f32,
    pub num_channels: usize,
    pub num_frames: usize,
    pub bit_depth: Option<u16>,
}
```

### wav::WavReader

WAV file reader.

#### Methods

```rust
// Construction
pub fn open<P: AsRef<Path>>(path: P) -> Result<Self>

// Reading
pub fn read_samples(&mut self, num_frames: usize) -> Result<Vec<Vec<f32>>>
pub fn read_all(&mut self) -> Result<Vec<Vec<f32>>>

// Queries
pub fn metadata(&self) -> &AudioMetadata
pub fn sample_rate(&self) -> f32
pub fn num_channels(&self) -> usize
pub fn num_frames(&self) -> usize
```

#### Example

```rust
use nih_plug_audio_formats::wav::WavReader;

let mut reader = WavReader::open("audio.wav")?;
let samples = reader.read_all()?;
```

### wav::WavWriter

WAV file writer.

#### Methods

```rust
// Construction
pub fn create<P: AsRef<Path>>(
    path: P,
    sample_rate: f32,
    num_channels: usize,
    bit_depth: u16
) -> Result<Self>

// Writing
pub fn write_samples(&mut self, samples: &[Vec<f32>]) -> Result<()>
pub fn write_interleaved(&mut self, samples: &[f32]) -> Result<()>
```

#### Example

```rust
use nih_plug_audio_formats::wav::WavWriter;

let mut writer = WavWriter::create("output.wav", 44100.0, 2, 16)?;
writer.write_samples(&samples)?;
```

---

## nih_plug_data

Data structures for state management.

### valuetree::ValueTree

Hierarchical data structure with change notifications.

#### Methods

```rust
// Construction
pub fn new(type_name: &str) -> Self

// Properties
pub fn set_property(&mut self, name: &str, value: Value)
pub fn get_property(&self, name: &str) -> Option<&Value>
pub fn remove_property(&mut self, name: &str) -> Option<Value>
pub fn has_property(&self, name: &str) -> bool

// Children
pub fn add_child(&mut self, child: ValueTree)
pub fn remove_child(&mut self, index: usize) -> Option<ValueTree>
pub fn get_child(&self, index: usize) -> Option<&ValueTree>
pub fn child_count(&self) -> usize

// Listeners
pub fn add_listener(&mut self, listener: Box<dyn ValueTreeListener>)

// Serialization
pub fn to_xml(&self) -> String
pub fn from_xml(xml: &str) -> Result<Self>
pub fn to_binary(&self) -> Vec<u8>
pub fn from_binary(data: &[u8]) -> Result<Self>
```

#### Value Types

```rust
pub enum Value {
    Int(i32),
    Float(f32),
    String(String),
    Bool(bool),
}
```

#### Example

```rust
use nih_plug_data::{ValueTree, Value};

let mut tree = ValueTree::new("root");
tree.set_property("name", Value::String("value".to_string()));

let child = ValueTree::new("child");
tree.add_child(child);
```

### undo::UndoManager

Undo/redo functionality.

#### Methods

```rust
// Construction
pub fn new() -> Self

// Operations
pub fn perform(&mut self, action: Box<dyn UndoableAction>) -> Result<()>
pub fn undo(&mut self) -> Result<()>
pub fn redo(&mut self) -> Result<()>

// Transactions
pub fn begin_transaction(&mut self)
pub fn end_transaction(&mut self)

// Queries
pub fn can_undo(&self) -> bool
pub fn can_redo(&self) -> bool
pub fn undo_description(&self) -> Option<&str>
pub fn redo_description(&self) -> Option<&str>

// State management
pub fn clear(&mut self)
```

#### Example

```rust
use nih_plug_data::{UndoManager, UndoableAction};

struct MyAction;

impl UndoableAction for MyAction {
    fn perform(&mut self) -> Result<(), DataError> {
        // Perform action
        Ok(())
    }
    
    fn undo(&mut self) -> Result<(), DataError> {
        // Undo action
        Ok(())
    }
}

let mut undo_manager = UndoManager::new();
undo_manager.perform(Box::new(MyAction))?;
undo_manager.undo()?;
```

---

## nih_plug_graphics

2D graphics primitives.

### Graphics

Main graphics context for drawing.

#### Methods

```rust
// Construction
pub fn new(width: u32, height: u32) -> Result<Self>

// Color
pub fn set_color(&mut self, color: Color)

// Primitives
pub fn fill_rect(&mut self, x: i32, y: i32, width: u32, height: u32)
pub fn draw_rect(&mut self, x: i32, y: i32, width: u32, height: u32, thickness: u32)
pub fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, thickness: u32)
pub fn draw_circle(&mut self, x: i32, y: i32, radius: u32)
pub fn fill_circle(&mut self, x: i32, y: i32, radius: u32)

// Images
pub fn draw_image(&mut self, image: &Image, x: i32, y: i32) -> Result<()>

// Text
pub fn draw_text(&mut self, text: &str, x: i32, y: i32, font: &Font) -> Result<()>

// Transforms
pub fn set_transform(&mut self, transform: Transform)
pub fn reset_transform(&mut self)

// Output
pub fn as_bytes(&self) -> &[u8]
pub fn width(&self) -> u32
pub fn height(&self) -> u32
```

#### Example

```rust
use nih_plug_graphics::{Graphics, Color};

let mut graphics = Graphics::new(800, 600)?;
graphics.set_color(Color::rgb(255, 0, 0));
graphics.fill_rect(10, 10, 100, 100);
```

### Color

Color representation with alpha channel.

#### Methods

```rust
// Construction
pub fn rgb(r: u8, g: u8, b: u8) -> Self
pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self
pub fn from_hex(hex: &str) -> Result<Self>

// Queries
pub fn r(&self) -> u8
pub fn g(&self) -> u8
pub fn b(&self) -> u8
pub fn a(&self) -> u8

// Manipulation
pub fn with_alpha(&self, alpha: u8) -> Self
pub fn blend(&self, other: &Color) -> Self
```

---

## nih_plug_gui

GUI component framework.

### Component

Base component with lifecycle management.

#### Methods

```rust
// Construction
pub fn new(name: &str) -> Self

// Hierarchy
pub fn add_child(&mut self, child: Component) -> Result<()>
pub fn remove_child(&mut self, index: usize) -> Option<Component>
pub fn child_count(&self) -> usize

// Bounds
pub fn set_bounds(&mut self, bounds: Bounds) -> Result<()>
pub fn bounds(&self) -> Bounds

// Lifecycle
pub fn initialize(&mut self)
pub fn destroy(&mut self)

// State
pub fn state(&self) -> ComponentState
pub fn set_visible(&mut self, visible: bool)
pub fn is_visible(&self) -> bool
```

### Button

Button control with click handling.

#### Methods

```rust
// Construction
pub fn new(text: &str) -> Self

// Configuration
pub fn set_text(&mut self, text: &str)
pub fn set_callback(&mut self, callback: Box<dyn Fn()>)

// State
pub fn state(&self) -> ButtonState
pub fn is_pressed(&self) -> bool
```

### Slider

Slider control for numeric values.

#### Methods

```rust
// Construction
pub fn new(orientation: SliderOrientation) -> Self

// Configuration
pub fn set_range(&mut self, min: f32, max: f32)
pub fn set_value(&mut self, value: f32)
pub fn set_callback(&mut self, callback: Box<dyn Fn(f32)>)

// Queries
pub fn value(&self) -> f32
pub fn min(&self) -> f32
pub fn max(&self) -> f32
```

---

## nih_plug_osc

Open Sound Control protocol.

### OscMessage

OSC message with address and arguments.

#### Methods

```rust
// Construction
pub fn new(address: &str, args: Vec<OscType>) -> Self

// Queries
pub fn address(&self) -> &str
pub fn args(&self) -> &[OscType]

// Serialization
pub fn to_bytes(&self) -> Vec<u8>
pub fn from_bytes(bytes: &[u8]) -> Result<Self>
```

### OscType

OSC data types.

```rust
pub enum OscType {
    Int(i32),
    Float(f32),
    String(String),
    Blob(Vec<u8>),
    Time(OscTime),
    Midi(OscMidi),
    Color(OscColor),
    True,
    False,
    Nil,
    Impulse,
}
```

### OscSender

Send OSC messages.

#### Methods

```rust
// Construction
pub fn new(address: &str) -> Result<Self>

// Sending
pub fn send(&mut self, message: &OscMessage) -> Result<()>
pub fn send_bundle(&mut self, bundle: &OscBundle) -> Result<()>
```

### OscReceiver

Receive OSC messages.

#### Methods

```rust
// Construction
pub fn bind(address: &str) -> Result<Self>

// Receiving
pub fn receive(&mut self) -> Result<OscPacket>
pub fn receive_timeout(&mut self, timeout: Duration) -> Result<OscPacket>
```

---

## nih_plug_crypto

Cryptography utilities.

### Hashing

```rust
// MD5
pub fn md5(data: &[u8]) -> [u8; 16]

// SHA-256
pub fn sha256(data: &[u8]) -> [u8; 32]

// SHA-512
pub fn sha512(data: &[u8]) -> [u8; 64]
```

### Encoding

```rust
// Base64
pub fn base64_encode(data: &[u8]) -> Result<String>
pub fn base64_decode(encoded: &str) -> Result<Vec<u8>>
```

### Random

```rust
// Generate random bytes
pub fn generate_random_bytes(count: usize) -> Result<Vec<u8>>
pub fn fill_random_bytes(buffer: &mut [u8]) -> Result<()>

// Generate random numbers
pub fn generate_random_u32() -> Result<u32>
pub fn generate_random_u64() -> Result<u64>
```

---

## nih_plug_animation

Animation and easing functions.

### Animation

Value animation with easing.

#### Methods

```rust
// Construction
pub fn new(start: f32, end: f32, duration: f32, easing: EasingFunction) -> Self

// Control
pub fn start(&mut self)
pub fn update(&mut self, delta_time: f32)
pub fn cancel(&mut self)
pub fn reset(&mut self)
pub fn jump_to_end(&mut self)

// Configuration
pub fn set_target(&mut self, target: f32)

// Queries
pub fn current_value(&self) -> f32
pub fn progress(&self) -> f32
pub fn state(&self) -> AnimationState
pub fn is_complete(&self) -> bool
pub fn is_running(&self) -> bool
```

### Easing Functions

```rust
pub fn linear(t: f32) -> f32
pub fn ease_in_quad(t: f32) -> f32
pub fn ease_out_quad(t: f32) -> f32
pub fn ease_in_out_quad(t: f32) -> f32
pub fn ease_in_cubic(t: f32) -> f32
pub fn ease_out_cubic(t: f32) -> f32
pub fn ease_in_out_cubic(t: f32) -> f32
// ... and more
```

---

## nih_plug_midi_ci

MIDI Capability Inquiry protocol.

### Discovery

```rust
pub struct DiscoveryInquiry {
    pub source_muid: Muid,
    pub device_info: DeviceInfo,
    pub capabilities: DeviceCapabilities,
}

impl DiscoveryInquiry {
    pub fn new(source_muid: Muid, device_info: DeviceInfo, capabilities: DeviceCapabilities) -> Self
    pub fn to_message(&self) -> MidiCiMessage
}
```

### Profiles

```rust
pub struct ProfileInquiry {
    pub source_muid: Muid,
    pub destination_muid: Muid,
}

pub struct SetProfileOn {
    pub source_muid: Muid,
    pub destination_muid: Muid,
    pub profile_id: ProfileId,
}
```

### Properties

```rust
pub struct PropertyGetData {
    pub source_muid: Muid,
    pub destination_muid: Muid,
    pub request_id: u8,
    pub header: String,
}

pub struct PropertySetData {
    pub source_muid: Muid,
    pub destination_muid: Muid,
    pub request_id: u8,
    pub header: String,
    pub body: Vec<u8>,
}
```

---

## Error Types

Each module defines its own error type:

```rust
// DSP
pub enum DspError {
    InvalidSampleRate(f32),
    InvalidBufferSize(usize),
    InvalidCoefficients,
    InvalidFrequency(f32),
}

// Audio Formats
pub enum AudioFormatError {
    FileNotFound(String),
    UnsupportedFormat(String),
    IoError(std::io::Error),
    InvalidData,
}

// Data
pub enum DataError {
    InvalidXml(String),
    PropertyNotFound(String),
    InvalidValueType,
}

// Graphics
pub enum GraphicsError {
    InvalidDimensions(u32, u32),
    InvalidColor,
    ImageLoadError(String),
}

// GUI
pub enum GuiError {
    InvalidBounds,
    InvalidHierarchy,
    ComponentNotFound,
}

// OSC
pub enum OscError {
    InvalidAddress(String),
    InvalidType,
    NetworkError(String),
}

// Crypto
pub enum CryptoError {
    InvalidInput,
    EncryptionFailed,
    DecryptionFailed,
}

// Animation
pub enum AnimationError {
    InvalidDuration,
    InvalidEasing,
}

// MIDI-CI
pub enum MidiCiError {
    InvalidMuid,
    InvalidMessage,
    ParseError(String),
}
```

---

## Thread Safety

### Send Types

These types can be sent between threads:

- All DSP types (IIRFilter, Oscillator, Envelope, SmoothedValue)
- All audio format types (readers, writers)
- All data types (ValueTree, UndoManager)
- All OSC types (sender, receiver, messages)
- All crypto types
- All animation types
- All MIDI-CI types

### Sync Types

These types can be shared between threads (with appropriate synchronization):

- Most DSP types (when wrapped in Arc<Mutex<T>>)
- Audio metadata types
- OSC message types
- Crypto utility functions
- MIDI-CI message types

### Not Thread-Safe

These types should not be shared between threads:

- GUI components (Component, Button, Slider, etc.)
- Graphics contexts
- Active audio file readers/writers

---

## Performance Notes

### Hot Paths

These operations are optimized for real-time audio processing:

- `IIRFilter::process()` - Specialized implementations for common orders
- `Oscillator::process()` - SIMD-friendly inner loops
- `SmoothedValue::next()` - Inlined for zero overhead
- `Envelope::get_next_sample()` - Minimal branching

### Allocations

These operations may allocate and should be avoided in audio callbacks:

- File I/O operations
- XML/binary serialization
- String operations
- Vector resizing

### Denormals

DSP operations include denormal prevention:

- Filters snap very small values to zero
- Oscillators use phase wrapping
- Envelopes clamp output values

---

## See Also

- [Migration Guide](MIGRATION_GUIDE.md) - Migrating from JUCE C++
- [README](README_PORTED_MODULES.md) - Module overview
- Full API documentation: `cargo doc --open --workspace`
