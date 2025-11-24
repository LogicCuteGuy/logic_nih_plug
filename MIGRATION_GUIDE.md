# Migration Guide: JUCE to nih-plug Ported Modules

This guide helps you migrate from JUCE C++ code to the ported Rust modules in nih-plug.

## Table of Contents

- [Overview](#overview)
- [Module Mapping](#module-mapping)
- [General Patterns](#general-patterns)
- [Module-Specific Migration](#module-specific-migration)
  - [DSP (juce_dsp → nih_plug_dsp)](#dsp-juce_dsp--nih_plug_dsp)
  - [Audio Formats (juce_audio_formats → nih_plug_audio_formats)](#audio-formats-juce_audio_formats--nih_plug_audio_formats)
  - [Data Structures (juce_data_structures → nih_plug_data)](#data-structures-juce_data_structures--nih_plug_data)
  - [Graphics (juce_graphics → nih_plug_graphics)](#graphics-juce_graphics--nih_plug_graphics)
  - [GUI (juce_gui_basics → nih_plug_gui)](#gui-juce_gui_basics--nih_plug_gui)
  - [OSC (juce_osc → nih_plug_osc)](#osc-juce_osc--nih_plug_osc)
  - [Cryptography (juce_cryptography → nih_plug_crypto)](#cryptography-juce_cryptography--nih_plug_crypto)
  - [Animation (juce_animation → nih_plug_animation)](#animation-juce_animation--nih_plug_animation)
  - [MIDI-CI (juce_midi_ci → nih_plug_midi_ci)](#midi-ci-juce_midi_ci--nih_plug_midi_ci)
- [Common Pitfalls](#common-pitfalls)
- [Performance Considerations](#performance-considerations)

## Overview

The ported modules translate JUCE's C++ algorithms and APIs to idiomatic Rust. Key differences:

- **No FFI overhead**: Pure Rust implementations
- **Memory safety**: Rust's ownership system prevents common bugs
- **Error handling**: Result types instead of exceptions
- **Naming conventions**: snake_case instead of camelCase
- **Lifetimes**: Explicit lifetime management
- **No unsafe in public APIs**: Safe by default

## Module Mapping

| JUCE Module | nih-plug Crate | Status |
|-------------|----------------|--------|
| juce_dsp | nih_plug_dsp | ✅ Complete |
| juce_audio_formats | nih_plug_audio_formats | ✅ Complete |
| juce_data_structures | nih_plug_data | ✅ Complete |
| juce_graphics | nih_plug_graphics | ✅ Complete |
| juce_gui_basics | nih_plug_gui | ✅ Complete |
| juce_osc | nih_plug_osc | ✅ Complete |
| juce_cryptography | nih_plug_crypto | ✅ Complete |
| juce_animation | nih_plug_animation | ✅ Complete |
| juce_midi_ci | nih_plug_midi_ci | ✅ Complete |

## General Patterns

### Memory Management

**JUCE (C++):**
```cpp
// Manual memory management
auto* filter = new IIRFilter();
// ... use filter ...
delete filter;

// Or with smart pointers
std::unique_ptr<IIRFilter> filter = std::make_unique<IIRFilter>();
```

**nih-plug (Rust):**
```rust
// Automatic memory management via ownership
let mut filter = IIRFilter::new();
// ... use filter ...
// Automatically cleaned up when it goes out of scope
```

### Error Handling

**JUCE (C++):**
```cpp
// Exceptions or return codes
try {
    filter.setCoefficients(coeffs);
} catch (const std::exception& e) {
    // Handle error
}
```

**nih-plug (Rust):**
```rust
// Result types
match filter.set_coefficients(&coeffs) {
    Ok(()) => { /* Success */ },
    Err(e) => { /* Handle error */ },
}

// Or with ? operator
filter.set_coefficients(&coeffs)?;
```

### Naming Conventions

**JUCE (C++):**
```cpp
filter.setCoefficients(coeffs);
filter.processSamples(input, output);
auto sampleRate = filter.getSampleRate();
```

**nih-plug (Rust):**
```rust
filter.set_coefficients(&coeffs)?;
filter.process_samples(&input, &mut output);
let sample_rate = filter.sample_rate();
```

### Thread Safety

**JUCE (C++):**
```cpp
// Manual synchronization
CriticalSection lock;
ScopedLock sl(lock);
// ... access shared data ...
```

**nih-plug (Rust):**
```rust
// Type system enforces thread safety
use std::sync::{Arc, Mutex};

let shared_data = Arc::new(Mutex::new(data));
let guard = shared_data.lock().unwrap();
// ... access shared data ...
```

## Module-Specific Migration

### DSP (juce_dsp → nih_plug_dsp)

#### IIR Filters

**JUCE (C++):**
```cpp
#include <juce_dsp/juce_dsp.h>

juce::dsp::IIR::Filter<float> filter;
auto coefficients = juce::dsp::IIR::Coefficients<float>::makeLowPass(
    44100.0, 1000.0, 0.707
);
filter.coefficients = coefficients;

juce::dsp::AudioBlock<float> block(buffer);
juce::dsp::ProcessContextReplacing<float> context(block);
filter.process(context);
```

**nih-plug (Rust):**
```rust
use nih_plug_dsp::filters::IIRFilter;

let mut filter = IIRFilter::new();
let coeffs = IIRFilter::make_low_pass(44100.0, 1000.0, 0.707);
filter.set_coefficients(&coeffs)?;

filter.process(&input, &mut output);
```

#### Oscillators

**JUCE (C++):**
```cpp
juce::dsp::Oscillator<float> osc;
osc.initialise([](float x) { return std::sin(x); });
osc.setFrequency(440.0);
osc.prepare({44100.0, 512, 1});

osc.processSample(0.0); // Generate sample
```

**nih-plug (Rust):**
```rust
use nih_plug_dsp::oscillators::{Oscillator, Waveform};

let mut osc = Oscillator::new(44100.0);
osc.set_waveform(Waveform::Sine);
osc.set_frequency(440.0);

let mut output = vec![0.0; 512];
osc.process(&mut output);
```

#### ADSR Envelope

**JUCE (C++):**
```cpp
juce::ADSR envelope;
juce::ADSR::Parameters params;
params.attack = 0.1f;
params.decay = 0.2f;
params.sustain = 0.7f;
params.release = 0.3f;
envelope.setParameters(params);

envelope.noteOn();
float value = envelope.getNextSample();
envelope.noteOff();
```

**nih-plug (Rust):**
```rust
use nih_plug_dsp::envelopes::Envelope;

let mut envelope = Envelope::new(44100.0);
envelope.set_adsr(0.1, 0.2, 0.7, 0.3);

envelope.note_on();
let value = envelope.get_next_sample();
envelope.note_off();
```

#### Smoothed Values

**JUCE (C++):**
```cpp
juce::SmoothedValue<float> smoothed;
smoothed.reset(44100.0, 0.05); // 50ms smoothing
smoothed.setTargetValue(1.0);

float current = smoothed.getNextValue();
```

**nih-plug (Rust):**
```rust
use nih_plug_dsp::smoothing::SmoothedValue;

let mut smoothed = SmoothedValue::new(44100.0, 0.05);
smoothed.set_target(1.0);

let current = smoothed.next();
```

#### State Variable Filter (TPT)

**JUCE (C++):**
```cpp
juce::dsp::StateVariableTPTFilter<float> filter;
juce::dsp::ProcessSpec spec;
spec.sampleRate = 44100.0;
spec.maximumBlockSize = 512;
spec.numChannels = 1;

filter.prepare(spec);
filter.setType(juce::dsp::StateVariableTPTFilterType::lowpass);
filter.setCutoffFrequency(1000.0f);
filter.setResonance(0.7f);

juce::dsp::AudioBlock<float> block(buffer);
juce::dsp::ProcessContextReplacing<float> context(block);
filter.process(context);
```

**nih-plug (Rust):**
```rust
use nih_plug_dsp::state_variable::{StateVariableFilter, FilterType};

let mut filter = StateVariableFilter::new();
filter.prepare(44100.0)?;
filter.set_type(FilterType::Lowpass);
filter.set_cutoff(1000.0);
filter.set_resonance(0.7);

filter.process(&input, &mut output);
```

#### FIR Filters

**JUCE (C++):**
```cpp
juce::dsp::FIR::Filter<float> filter;
auto coeffs = juce::dsp::FIR::Coefficients<float>::makeLowPass(
    44100.0,
    1000.0,
    65
);
filter.coefficients = coeffs;

juce::dsp::AudioBlock<float> block(buffer);
juce::dsp::ProcessContextReplacing<float> context(block);
filter.process(context);
```

**nih-plug (Rust):**
```rust
use nih_plug_dsp::fir::{FIRFilter, WindowFunction, design_lowpass};

let coeffs = design_lowpass(
    1000.0,
    44100.0,
    65,
    WindowFunction::Hann
)?;
let mut filter = FIRFilter::new(coeffs);

filter.process(&input, &mut output);
```

#### Processor Chain

**JUCE (C++):**
```cpp
juce::dsp::ProcessorChain<
    juce::dsp::Gain<float>,
    juce::dsp::WaveShaper<float>,
    juce::dsp::IIR::Filter<float>
> chain;

juce::dsp::ProcessSpec spec;
spec.sampleRate = 44100.0;
spec.maximumBlockSize = 512;
spec.numChannels = 1;
chain.prepare(spec);

chain.get<0>().setGainDecibels(12.0f);
chain.get<1>().functionToUse = [](float x) { return std::tanh(x); };

juce::dsp::AudioBlock<float> block(buffer);
juce::dsp::ProcessContextReplacing<float> context(block);
chain.process(context);
```

**nih-plug (Rust):**
```rust
use nih_plug_dsp::processors::{
    ProcessorChain, Gain, WaveShaper, DCFilter, transfer_functions
};

let mut chain = ProcessorChain::new();

let mut gain = Gain::new();
gain.set_gain_db(12.0);
chain.add(gain);

let shaper = WaveShaper::new(transfer_functions::tanh);
chain.add(shaper);

let dc_filter = DCFilter::new();
chain.add(dc_filter);

chain.prepare(44100.0, 512);
chain.process(&input, &mut output);
```

#### FFT

**JUCE (C++):**
```cpp
juce::dsp::FFT fft(10); // 2^10 = 1024 points

std::vector<float> timeDomain(1024);
std::vector<float> frequencyDomain(2048); // Complex data needs 2x space

fft.performFrequencyOnlyForwardTransform(timeDomain.data());
```

**nih-plug (Rust):**
```rust
use nih_plug_dsp::analysis::FFT;
use num_complex::Complex;

let fft = FFT::new(1024)?;

let input = vec![0.0; 1024];
let mut spectrum = vec![Complex::new(0.0, 0.0); 1024];
fft.forward(&input, &mut spectrum);

// Or get magnitude only
let mut magnitudes = vec![0.0; 1024];
fft.forward_magnitude(&input, &mut magnitudes);
```

#### Gain Processor

**JUCE (C++):**
```cpp
juce::dsp::Gain<float> gain;
gain.setGainDecibels(6.0f);

juce::dsp::AudioBlock<float> block(buffer);
juce::dsp::ProcessContextReplacing<float> context(block);
gain.process(context);
```

**nih-plug (Rust):**
```rust
use nih_plug_dsp::processors::Gain;

let mut gain = Gain::new();
gain.prepare(44100.0, 512);
gain.set_gain_db(6.0);

gain.process(&input, &mut output);
```

#### Bias/DC Offset

**JUCE (C++):**
```cpp
juce::dsp::Bias<float> bias;
bias.setBias(0.1f);

juce::dsp::AudioBlock<float> block(buffer);
juce::dsp::ProcessContextReplacing<float> context(block);
bias.process(context);
```

**nih-plug (Rust):**
```rust
use nih_plug_dsp::processors::Bias;

let mut bias = Bias::new();
bias.prepare(44100.0, 512);
bias.set_bias(0.1);

bias.process(&input, &mut output);
```

#### WaveShaper

**JUCE (C++):**
```cpp
juce::dsp::WaveShaper<float> shaper;
shaper.functionToUse = [](float x) { return std::tanh(x); };

juce::dsp::AudioBlock<float> block(buffer);
juce::dsp::ProcessContextReplacing<float> context(block);
shaper.process(context);
```

**nih-plug (Rust):**
```rust
use nih_plug_dsp::processors::{WaveShaper, transfer_functions};

// Use predefined function
let mut shaper = WaveShaper::new(transfer_functions::tanh);
shaper.prepare(44100.0, 512);
shaper.process(&input, &mut output);

// Or custom function
let custom_shaper = WaveShaper::new(|x| x * x * x);
```

### Audio Formats (juce_audio_formats → nih_plug_audio_formats)

#### Reading Audio Files

**JUCE (C++):**
```cpp
juce::AudioFormatManager formatManager;
formatManager.registerBasicFormats();

auto* reader = formatManager.createReaderFor(juce::File("audio.wav"));
if (reader != nullptr) {
    juce::AudioBuffer<float> buffer(reader->numChannels, reader->lengthInSamples);
    reader->read(&buffer, 0, reader->lengthInSamples, 0, true, true);
    delete reader;
}
```

**nih-plug (Rust):**
```rust
use nih_plug_audio_formats::wav::WavReader;

let mut reader = WavReader::open("audio.wav")?;
let metadata = reader.metadata();
let samples = reader.read_all()?;

println!("Sample rate: {}", metadata.sample_rate);
println!("Channels: {}", metadata.num_channels);
```

#### Writing Audio Files

**JUCE (C++):**
```cpp
juce::WavAudioFormat format;
auto* writer = format.createWriterFor(
    new juce::FileOutputStream(juce::File("output.wav")),
    44100.0,
    2,
    16,
    {},
    0
);

if (writer != nullptr) {
    writer->writeFromAudioSampleBuffer(buffer, 0, buffer.getNumSamples());
    delete writer;
}
```

**nih-plug (Rust):**
```rust
use nih_plug_audio_formats::wav::WavWriter;

let mut writer = WavWriter::create(
    "output.wav",
    44100.0,
    2,
    16
)?;

writer.write_samples(&samples)?;
```

### Data Structures (juce_data_structures → nih_plug_data)

#### ValueTree

**JUCE (C++):**
```cpp
juce::ValueTree tree("root");
tree.setProperty("name", "value", nullptr);

juce::ValueTree child("child");
tree.appendChild(child, nullptr);

auto xml = tree.toXmlString();
```

**nih-plug (Rust):**
```rust
use nih_plug_data::{ValueTree, Value};

let mut tree = ValueTree::new("root");
tree.set_property("name", Value::String("value".to_string()));

let child = ValueTree::new("child");
tree.add_child(child);

let xml = tree.to_xml();
```

#### UndoManager

**JUCE (C++):**
```cpp
juce::UndoManager undoManager;

class MyAction : public juce::UndoableAction {
    bool perform() override { /* ... */ return true; }
    bool undo() override { /* ... */ return true; }
};

undoManager.perform(new MyAction());
undoManager.undo();
undoManager.redo();
```

**nih-plug (Rust):**
```rust
use nih_plug_data::{UndoManager, UndoableAction};

struct MyAction { /* ... */ }

impl UndoableAction for MyAction {
    fn perform(&mut self) -> Result<(), DataError> {
        // ... perform action ...
        Ok(())
    }
    
    fn undo(&mut self) -> Result<(), DataError> {
        // ... undo action ...
        Ok(())
    }
}

let mut undo_manager = UndoManager::new();
undo_manager.perform(Box::new(MyAction { /* ... */ }))?;
undo_manager.undo()?;
undo_manager.redo()?;
```

### Graphics (juce_graphics → nih_plug_graphics)

#### Drawing Primitives

**JUCE (C++):**
```cpp
juce::Graphics g(image);
g.setColour(juce::Colours::red);
g.fillRect(10, 10, 100, 100);
g.drawLine(0, 0, 100, 100, 2.0f);
```

**nih-plug (Rust):**
```rust
use nih_plug_graphics::{Graphics, Color};

let mut graphics = Graphics::new(800, 600)?;
graphics.set_color(Color::rgb(255, 0, 0));
graphics.fill_rect(10, 10, 100, 100);
graphics.draw_line(0, 0, 100, 100, 2);
```

#### Loading Images

**JUCE (C++):**
```cpp
juce::Image image = juce::ImageFileFormat::loadFrom(juce::File("image.png"));
g.drawImageAt(image, 0, 0);
```

**nih-plug (Rust):**
```rust
use nih_plug_graphics::Image;

let image = Image::load("image.png")?;
graphics.draw_image(&image, 0, 0)?;
```

### GUI (juce_gui_basics → nih_plug_gui)

#### Components

**JUCE (C++):**
```cpp
class MyComponent : public juce::Component {
public:
    MyComponent() {
        addAndMakeVisible(button);
        button.onClick = [this] { buttonClicked(); };
    }
    
    void resized() override {
        button.setBounds(10, 10, 100, 30);
    }
    
private:
    juce::TextButton button{"Click Me"};
};
```

**nih-plug (Rust):**
```rust
use nih_plug_gui::{Component, Button, Bounds};

let mut parent = Component::new("parent");
parent.set_bounds(Bounds::new(0, 0, 400, 300))?;

let mut button = Button::new("Click Me");
button.set_bounds(Bounds::new(10, 10, 100, 30))?;
button.set_callback(Box::new(|| {
    println!("Button clicked!");
}));

parent.add_child(button.into())?;
```

#### LookAndFeel

**JUCE (C++):**
```cpp
class MyLookAndFeel : public juce::LookAndFeel_V4 {
    void drawButtonBackground(juce::Graphics& g, juce::Button& button, 
                             const juce::Colour& backgroundColour,
                             bool isMouseOverButton, bool isButtonDown) override {
        // Custom drawing
    }
};

MyLookAndFeel laf;
button.setLookAndFeel(&laf);
```

**nih-plug (Rust):**
```rust
use nih_plug_gui::lookandfeel::{LookAndFeel, DefaultLookAndFeel, Theme};

let laf = DefaultLookAndFeel::with_theme(Theme::Dark);
let button_color = laf.button_color(ButtonState::Normal);

// Or implement custom LookAndFeel trait
struct MyLookAndFeel;

impl LookAndFeel for MyLookAndFeel {
    fn button_color(&self, state: ButtonState) -> Color {
        // Custom colors
        Color::rgb(100, 150, 200)
    }
    // ... other methods ...
}
```

#### FlexBox Layout

**JUCE (C++):**
```cpp
juce::FlexBox flexbox;
flexbox.flexDirection = juce::FlexBox::Direction::row;
flexbox.flexWrap = juce::FlexBox::Wrap::wrap;
flexbox.justifyContent = juce::FlexBox::JustifyContent::spaceBetween;
flexbox.alignItems = juce::FlexBox::AlignItems::center;

flexbox.items.add(juce::FlexItem(100, 50).withMargin(10));
flexbox.items.add(juce::FlexItem().withFlex(1).withHeight(50));
flexbox.items.add(juce::FlexItem(100, 50).withMargin(10));

flexbox.performLayout(bounds);
```

**nih-plug (Rust):**
```rust
use nih_plug_gui::layout::{
    FlexBox, FlexItem, FlexDirection, FlexWrap, 
    JustifyContent, AlignItems, Margin
};

let mut flexbox = FlexBox::new();
flexbox.set_direction(FlexDirection::Row);
flexbox.set_wrap(FlexWrap::Wrap);
flexbox.set_justify_content(JustifyContent::SpaceBetween);
flexbox.set_align_items(AlignItems::Center);

flexbox.add_item(FlexItem {
    width: Some(100.0),
    height: Some(50.0),
    margin: Margin::all(10.0),
    ..Default::default()
});

flexbox.add_item(FlexItem {
    flex_grow: 1.0,
    height: Some(50.0),
    ..Default::default()
});

flexbox.add_item(FlexItem {
    width: Some(100.0),
    height: Some(50.0),
    margin: Margin::all(10.0),
    ..Default::default()
});

let rects = flexbox.layout(800.0, 600.0);
```

### OSC (juce_osc → nih_plug_osc)

#### Sending OSC Messages

**JUCE (C++):**
```cpp
juce::OSCSender sender;
sender.connect("127.0.0.1", 9000);

juce::OSCMessage message("/synth/frequency");
message.addFloat32(440.0f);
sender.send(message);
```

**nih-plug (Rust):**
```rust
use nih_plug_osc::{OscSender, OscMessage, OscType};

let mut sender = OscSender::new("127.0.0.1:9000")?;

let message = OscMessage::new(
    "/synth/frequency",
    vec![OscType::Float(440.0)]
);
sender.send(&message)?;
```

#### Receiving OSC Messages

**JUCE (C++):**
```cpp
class MyReceiver : public juce::OSCReceiver::Listener<juce::OSCReceiver::MessageLoopCallback> {
    void oscMessageReceived(const juce::OSCMessage& message) override {
        if (message.getAddressPattern() == "/synth/frequency") {
            float freq = message[0].getFloat32();
        }
    }
};

juce::OSCReceiver receiver;
receiver.addListener(&myReceiver);
receiver.connect(9000);
```

**nih-plug (Rust):**
```rust
use nih_plug_osc::{OscReceiver, OscType};

let mut receiver = OscReceiver::bind("0.0.0.0:9000")?;

loop {
    if let Ok(packet) = receiver.receive() {
        if let Some(message) = packet.as_message() {
            if message.address == "/synth/frequency" {
                if let Some(OscType::Float(freq)) = message.args.get(0) {
                    println!("Frequency: {}", freq);
                }
            }
        }
    }
}
```

### Cryptography (juce_cryptography → nih_plug_crypto)

#### Hashing

**JUCE (C++):**
```cpp
juce::MD5 md5(data.getData(), data.getSize());
juce::String hash = md5.toHexString();
```

**nih-plug (Rust):**
```rust
use nih_plug_crypto::hashing::md5;

let hash = md5(b"data");
let hex_string = hex::encode(hash);
```

#### Encryption

**JUCE (C++):**
```cpp
juce::RSAKey privateKey, publicKey;
juce::RSAKey::createKeyPair(publicKey, privateKey, 1024);

juce::MemoryBlock encrypted;
publicKey.encryptToMemoryBlock(data, encrypted);
```

**nih-plug (Rust):**
```rust
use nih_plug_crypto::encryption::{Encryptor, EncryptionAlgorithm};

let mut encryptor = Encryptor::new(EncryptionAlgorithm::Rsa)?;
let encrypted = encryptor.encrypt(b"data")?;
let decrypted = encryptor.decrypt(&encrypted)?;
```

### Animation (juce_animation → nih_plug_animation)

**JUCE (C++):**
```cpp
juce::AnimatedPosition<float> position;
position.setPosition(0.0);
position.setSpeed(1.0);
position.setTarget(100.0);

// In timer callback
position.update(deltaTime);
float current = position.getPosition();
```

**nih-plug (Rust):**
```rust
use nih_plug_animation::{Animation, AnimationState};
use nih_plug_animation::easing::ease_in_out_cubic;

let mut anim = Animation::new(0.0, 100.0, 1.0, ease_in_out_cubic);
anim.start();

// In update loop
anim.update(delta_time);
let current = anim.current_value();

if anim.is_complete() {
    println!("Animation finished!");
}
```

### MIDI-CI (juce_midi_ci → nih_plug_midi_ci)

**JUCE (C++):**
```cpp
// JUCE doesn't have built-in MIDI-CI support
// This is new functionality
```

**nih-plug (Rust):**
```rust
use nih_plug_midi_ci::{
    discovery::{DiscoveryInquiry, DeviceCapabilities},
    protocol::{DeviceInfo, Muid},
};

let my_muid = Muid::new(0x1234567)?;
let device_info = DeviceInfo::new(
    vec![0x7D],
    0x1234,
    0x5678,
    0x010000,
);
let capabilities = DeviceCapabilities::all();

let inquiry = DiscoveryInquiry::new(my_muid, device_info, capabilities);
let message = inquiry.to_message();
let sysex = message.to_sysex();
```

## Common Pitfalls

### 1. Forgetting to Handle Errors

**Wrong:**
```rust
let filter = IIRFilter::new();
filter.set_coefficients(&coeffs); // Compile error!
```

**Right:**
```rust
let mut filter = IIRFilter::new();
filter.set_coefficients(&coeffs)?; // Or use match/unwrap
```

### 2. Mutability

**Wrong:**
```rust
let filter = IIRFilter::new();
filter.process(&input, &mut output); // Compile error! filter is immutable
```

**Right:**
```rust
let mut filter = IIRFilter::new();
filter.process(&input, &mut output);
```

### 3. Ownership and Borrowing

**Wrong:**
```rust
let samples = vec![0.0; 1024];
filter.process(&samples, &mut samples); // Compile error! Can't borrow as mutable and immutable
```

**Right:**
```rust
let input = vec![0.0; 1024];
let mut output = vec![0.0; 1024];
filter.process(&input, &mut output);
```

### 4. Lifetime Issues

**Wrong:**
```rust
fn get_reader() -> AudioFileReader {
    let reader = AudioFileReader::open("file.wav")?;
    reader // May have lifetime issues
}
```

**Right:**
```rust
fn get_reader() -> Result<AudioFileReader, AudioFormatError> {
    AudioFileReader::open("file.wav")
}
```

## Performance Considerations

### 1. Avoid Unnecessary Allocations

**Less efficient:**
```rust
for _ in 0..1000 {
    let mut output = vec![0.0; 512];
    filter.process(&input, &mut output);
}
```

**More efficient:**
```rust
let mut output = vec![0.0; 512];
for _ in 0..1000 {
    filter.process(&input, &mut output);
}
```

### 2. Use Slices Instead of Vectors

**Less efficient:**
```rust
fn process_audio(input: Vec<f32>) -> Vec<f32> {
    let mut output = vec![0.0; input.len()];
    // ... process ...
    output
}
```

**More efficient:**
```rust
fn process_audio(input: &[f32], output: &mut [f32]) {
    // ... process ...
}
```

### 3. Leverage Iterators

**Less efficient:**
```rust
for i in 0..samples.len() {
    samples[i] *= 0.5;
}
```

**More efficient:**
```rust
samples.iter_mut().for_each(|s| *s *= 0.5);
```

### 4. Profile Before Optimizing

Use `cargo bench` with criterion to identify bottlenecks:

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

## Getting Help

- **Documentation**: Run `cargo doc --open` to view full API documentation
- **Examples**: Check the `examples/` directory in each crate
- **Issues**: Report bugs or request features on GitHub
- **Community**: Join the nih-plug Discord server

## Contributing

Found a bug or want to improve the ported modules? Contributions are welcome!

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Submit a pull request

## License

The ported modules maintain compatibility with both JUCE's GPL/Commercial license
and nih-plug's permissive license. See LICENSE files for details.
