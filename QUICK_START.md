# Quick Start Guide: JUCE Ported Modules

Get started with the JUCE ported modules in 5 minutes.

## Installation

Add the modules you need to your `Cargo.toml`:

```toml
[dependencies]
nih_plug = { path = "../nih_plug" }
nih_plug_dsp = { path = "../nih_plug_dsp", features = ["filters", "oscillators"] }
nih_plug_audio_formats = { path = "../nih_plug_audio_formats", features = ["wav"] }
```

## Basic Examples

### 1. DSP: Apply a Filter

```rust
use nih_plug_dsp::filters::IIRFilter;

// Create and configure a filter
let mut filter = IIRFilter::new();
filter.set_coefficients(&[1.0, 0.5], &[1.0, -0.5])?;

// Process audio
let input = vec![1.0; 512];
let mut output = vec![0.0; 512];
filter.process(&input, &mut output);
```

### 2. Generate a Waveform

```rust
use nih_plug_dsp::oscillators::{Oscillator, Waveform};

// Create an oscillator
let mut osc = Oscillator::new(44100.0);
osc.set_waveform(Waveform::Sine);
osc.set_frequency(440.0); // A4

// Generate samples
let mut output = vec![0.0; 512];
osc.process(&mut output);
```

### 3. Read an Audio File

```rust
use nih_plug_audio_formats::wav::WavReader;

// Open and read a WAV file
let mut reader = WavReader::open("audio.wav")?;
let samples = reader.read_all()?;

println!("Sample rate: {}", reader.sample_rate());
println!("Channels: {}", reader.num_channels());
```

### 4. Create a GUI Button

```rust
use nih_plug_gui::{Component, Button, Bounds};

// Create a parent component
let mut parent = Component::new("parent");
parent.set_bounds(Bounds::new(0, 0, 400, 300))?;

// Add a button
let mut button = Button::new("Click Me");
button.set_bounds(Bounds::new(10, 10, 100, 30))?;
button.set_callback(Box::new(|| {
    println!("Button clicked!");
}));

parent.add_child(button.into())?;
```

### 5. Send OSC Messages

```rust
use nih_plug_osc::{OscSender, OscMessage, OscType};

// Create a sender
let mut sender = OscSender::new("127.0.0.1:9000")?;

// Send a message
let message = OscMessage::new(
    "/synth/frequency",
    vec![OscType::Float(440.0)]
);
sender.send(&message)?;
```

### 6. Animate a Value

```rust
use nih_plug_animation::{Animation, AnimationState};
use nih_plug_animation::easing::ease_in_out_cubic;

// Create an animation
let mut anim = Animation::new(0.0, 100.0, 1.0, ease_in_out_cubic);
anim.start();

// In your update loop
loop {
    anim.update(0.016); // 16ms frame time
    let current = anim.current_value();
    
    if anim.is_complete() {
        break;
    }
}
```

### 7. State Variable Filter

```rust
use nih_plug_dsp::state_variable::{StateVariableFilter, FilterType};

// Create a resonant lowpass filter
let mut filter = StateVariableFilter::new();
filter.prepare(44100.0)?;
filter.set_type(FilterType::Lowpass);
filter.set_cutoff(1000.0);
filter.set_resonance(0.8);

// Process audio
let input = vec![1.0; 512];
let mut output = vec![0.0; 512];
filter.process(&input, &mut output);
```

### 8. FIR Filter Design

```rust
use nih_plug_dsp::fir::{FIRFilter, WindowFunction, design_lowpass};

// Design a linear-phase lowpass filter
let coeffs = design_lowpass(
    1000.0,                  // Cutoff frequency
    44100.0,                 // Sample rate
    65,                      // Filter length (odd number)
    WindowFunction::Hann,    // Window function
)?;

let mut filter = FIRFilter::new(coeffs);
let input = vec![1.0; 512];
let mut output = vec![0.0; 512];
filter.process(&input, &mut output);
```

### 9. Processor Chain (Overdrive Effect)

```rust
use nih_plug_dsp::processors::{
    ProcessorChain, Gain, Bias, WaveShaper, DCFilter, transfer_functions
};

// Build an overdrive effect
let mut chain = ProcessorChain::new();

// Input gain
let mut input_gain = Gain::new();
input_gain.set_gain_db(12.0);
chain.add(input_gain);

// Add DC bias for asymmetric distortion
let mut bias = Bias::new();
bias.set_bias(0.1);
chain.add(bias);

// Waveshaping
let shaper = WaveShaper::new(transfer_functions::tanh);
chain.add(shaper);

// Remove DC offset
let dc_filter = DCFilter::new();
chain.add(dc_filter);

// Output gain
let mut output_gain = Gain::new();
output_gain.set_gain_db(-6.0);
chain.add(output_gain);

// Process
chain.prepare(44100.0, 512);
let input = vec![0.5; 512];
let mut output = vec![0.0; 512];
chain.process(&input, &mut output);
```

### 10. FFT Spectrum Analysis

```rust
use nih_plug_dsp::analysis::FFT;

// Create 1024-point FFT
let fft = FFT::new(1024)?;

// Analyze audio
let input = vec![0.0; 1024];
let mut magnitudes = vec![0.0; 1024];
fft.forward_magnitude(&input, &mut magnitudes);

// magnitudes[0] = DC component
// magnitudes[512] = Nyquist frequency
// Each bin represents sample_rate / 1024 Hz
```

### 11. FlexBox Layout

```rust
use nih_plug_gui::layout::{
    FlexBox, FlexItem, FlexDirection, JustifyContent, AlignItems
};

// Create a horizontal layout
let mut flexbox = FlexBox::new();
flexbox.set_direction(FlexDirection::Row);
flexbox.set_justify_content(JustifyContent::SpaceBetween);
flexbox.set_align_items(AlignItems::Center);

// Add items
flexbox.add_item(FlexItem {
    width: Some(100.0),
    height: Some(50.0),
    ..Default::default()
});

flexbox.add_item(FlexItem {
    flex_grow: 1.0,  // Takes remaining space
    height: Some(50.0),
    ..Default::default()
});

flexbox.add_item(FlexItem {
    width: Some(100.0),
    height: Some(50.0),
    ..Default::default()
});

// Calculate layout
let rects = flexbox.layout(800.0, 600.0);
```

## Common Patterns

### Error Handling

All fallible operations return `Result`:

```rust
// Use ? operator for propagation
filter.set_coefficients(&coeffs)?;

// Or match for custom handling
match filter.set_coefficients(&coeffs) {
    Ok(()) => println!("Success"),
    Err(e) => eprintln!("Error: {}", e),
}
```

### Buffer Reuse

Avoid allocations in audio processing:

```rust
// Bad: Allocates every iteration
for _ in 0..1000 {
    let mut output = vec![0.0; 512];
    filter.process(&input, &mut output);
}

// Good: Reuse buffer
let mut output = vec![0.0; 512];
for _ in 0..1000 {
    filter.process(&input, &mut output);
}
```

### State Management

Reset DSP state when needed:

```rust
let mut filter = IIRFilter::new();
filter.set_coefficients(&coeffs)?;

// Process some audio...
filter.process(&input1, &mut output1);

// Reset for a new stream
filter.reset();

// Process new audio
filter.process(&input2, &mut output2);
```

## Feature Flags

Enable only what you need:

```toml
[dependencies]
# Minimal DSP
nih_plug_dsp = { path = "../nih_plug_dsp", features = ["filters"] }

# All DSP features
nih_plug_dsp = { path = "../nih_plug_dsp", features = ["full"] }

# Specific audio formats
nih_plug_audio_formats = { 
    path = "../nih_plug_audio_formats", 
    features = ["wav", "flac"] 
}
```

## Building a Plugin

Here's a minimal plugin using the ported modules:

```rust
use nih_plug::prelude::*;
use nih_plug_dsp::filters::IIRFilter;
use std::sync::Arc;

struct MyPlugin {
    params: Arc<MyParams>,
    filter: IIRFilter,
}

#[derive(Params)]
struct MyParams {
    #[id = "gain"]
    pub gain: FloatParam,
}

impl Default for MyPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(MyParams::default()),
            filter: IIRFilter::new(),
        }
    }
}

impl Plugin for MyPlugin {
    const NAME: &'static str = "My Plugin";
    const VENDOR: &'static str = "My Company";
    const URL: &'static str = "https://example.com";
    const EMAIL: &'static str = "info@example.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    type BackgroundTask = ();
    type SysExMessage = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // Initialize filter
        self.filter.set_coefficients(&[1.0, 0.5], &[1.0, -0.5]).unwrap();
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for channel_samples in buffer.iter_samples() {
            let gain = self.params.gain.smoothed.next();
            
            for sample in channel_samples {
                *sample = self.filter.process_sample(*sample) * gain;
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for MyPlugin {
    const CLAP_ID: &'static str = "com.example.my-plugin";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("My plugin description");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Filter,
    ];
}

impl Vst3Plugin for MyPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"MyPluginID123456";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Fx,
        Vst3SubCategory::Filter,
    ];
}

nih_export_clap!(MyPlugin);
nih_export_vst3!(MyPlugin);
```

## Migrating from JUCE

### State Variable Filter

**JUCE C++:**
```cpp
juce::dsp::StateVariableTPTFilter<float> filter;
filter.prepare(spec);
filter.setType(juce::dsp::StateVariableTPTFilterType::lowpass);
filter.setCutoffFrequency(1000.0f);
filter.setResonance(0.7f);
filter.process(context);
```

**nih-plug Rust:**
```rust
use nih_plug_dsp::state_variable::{StateVariableFilter, FilterType};

let mut filter = StateVariableFilter::new();
filter.prepare(44100.0)?;
filter.set_type(FilterType::Lowpass);
filter.set_cutoff(1000.0);
filter.set_resonance(0.7);
filter.process(&input, &mut output);
```

### FIR Filter

**JUCE C++:**
```cpp
juce::dsp::FIR::Filter<float> filter;
juce::dsp::FIR::Coefficients<float>::Ptr coeffs = 
    juce::dsp::FIR::Coefficients<float>::makeLowPass(44100.0, 1000.0, 65);
filter.coefficients = coeffs;
filter.process(context);
```

**nih-plug Rust:**
```rust
use nih_plug_dsp::fir::{FIRFilter, WindowFunction, design_lowpass};

let coeffs = design_lowpass(1000.0, 44100.0, 65, WindowFunction::Hann)?;
let mut filter = FIRFilter::new(coeffs);
filter.process(&input, &mut output);
```

### Processor Chain

**JUCE C++:**
```cpp
juce::dsp::ProcessorChain<Gain, WaveShaper, DCFilter> chain;
chain.prepare(spec);
chain.get<0>().setGainDecibels(12.0f);
chain.process(context);
```

**nih-plug Rust:**
```rust
use nih_plug_dsp::processors::{ProcessorChain, Gain, WaveShaper, DCFilter};

let mut chain = ProcessorChain::new();
let mut gain = Gain::new();
gain.set_gain_db(12.0);
chain.add(gain);
chain.add(WaveShaper::new(|x| x.tanh()));
chain.add(DCFilter::new());
chain.prepare(44100.0, 512);
chain.process(&input, &mut output);
```

### FFT

**JUCE C++:**
```cpp
juce::dsp::FFT fft(10);  // 2^10 = 1024 points
fft.performFrequencyOnlyForwardTransform(data);
```

**nih-plug Rust:**
```rust
use nih_plug_dsp::analysis::FFT;

let fft = FFT::new(1024)?;
let mut magnitudes = vec![0.0; 1024];
fft.forward_magnitude(&input, &mut magnitudes);
```

### FlexBox Layout

**JUCE C++:**
```cpp
juce::FlexBox flexbox;
flexbox.flexDirection = juce::FlexBox::Direction::row;
flexbox.justifyContent = juce::FlexBox::JustifyContent::spaceBetween;
flexbox.items.add(juce::FlexItem(100, 50));
flexbox.performLayout(bounds);
```

**nih-plug Rust:**
```rust
use nih_plug_gui::layout::{FlexBox, FlexItem, FlexDirection, JustifyContent};

let mut flexbox = FlexBox::new();
flexbox.set_direction(FlexDirection::Row);
flexbox.set_justify_content(JustifyContent::SpaceBetween);
flexbox.add_item(FlexItem {
    width: Some(100.0),
    height: Some(50.0),
    ..Default::default()
});
let rects = flexbox.layout(800.0, 600.0);
```

## Next Steps

1. **Read the documentation**
   - [Module Overview](README_PORTED_MODULES.md)
   - [API Reference](API_REFERENCE.md)
   - `cargo doc --open --workspace`

2. **Try the examples**
   ```bash
   cargo run --example smoothing_demo -p nih_plug_dsp
   cargo run --example animation_demo -p nih_plug_animation
   cargo run --bin state_variable_filter
   cargo run --bin overdrive
   cargo run --bin spectrum_analyzer
   cargo run --bin flexbox_demo
   ```

3. **Run the tests**
   ```bash
   cargo test --workspace
   ```

4. **Migrate from JUCE**
   - See [Migration Guide](MIGRATION_GUIDE.md)
   - Check [JUCE Examples](plugins/examples/JUCE_EXAMPLES.md)

5. **Join the community**
   - nih-plug Discord server
   - GitHub discussions

## Troubleshooting

### Compilation Errors

**Problem:** Missing features
```
error[E0433]: failed to resolve: could not find `filters` in `nih_plug_dsp`
```

**Solution:** Enable the required feature
```toml
nih_plug_dsp = { path = "../nih_plug_dsp", features = ["filters"] }
```

### Runtime Errors

**Problem:** Invalid coefficients
```rust
filter.set_coefficients(&[], &[])?; // Error!
```

**Solution:** Provide valid coefficients
```rust
filter.set_coefficients(&[1.0, 0.5], &[1.0, -0.5])?;
```

### Performance Issues

**Problem:** Allocations in audio callback

**Solution:** Pre-allocate buffers
```rust
// In initialize()
self.buffer = vec![0.0; max_buffer_size];

// In process()
filter.process(&input, &mut self.buffer[..buffer.len()]);
```

## Getting Help

- **Documentation**: `cargo doc --open --workspace`
- **Examples**: Check `examples/` in each crate
- **Issues**: Report on GitHub
- **Community**: nih-plug Discord

## Contributing

Found a bug or want to improve the modules?

1. Fork the repository
2. Create a feature branch
3. Add tests for your changes
4. Submit a pull request

## License

Dual-licensed under GPL v3 and MIT. See LICENSE files for details.
