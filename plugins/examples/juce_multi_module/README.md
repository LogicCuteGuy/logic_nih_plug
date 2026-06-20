# JUCE Multi-Module Synthesizer Example

This advanced example demonstrates using multiple ported JUCE modules together in a single plugin. It showcases how different modules can work together to create a feature-rich synthesizer.

## Modules Used

### DSP Module (`logic_nih_plug_dsp`)
- **Oscillators**: Generates sine, saw, square, and triangle waveforms
- **Filters**: IIR low-pass filter for tone shaping
- **Envelopes**: ADSR envelope for amplitude control
- **Smoothing**: Parameter smoothing for glitch-free changes

### Data Module (`logic_nih_plug_data`)
- **ValueTree**: Hierarchical preset storage
- **Serialization**: XML-based preset save/load capability

### Animation Module (`logic_nih_plug_animation`)
- **Easing Functions**: Smooth filter cutoff modulation
- **Animation Curves**: InOutQuad easing for natural movement

### Crypto Module (`logic_nih_plug_crypto`)
- **Hashing**: SHA-256 hash generation for preset verification
- **Integrity**: Ensures preset data hasn't been corrupted

### Audio Formats Module (`logic_nih_plug_audio_formats`)
- Demonstrates integration capability for future impulse response loading

## Features

### Synthesis Engine
- Multiple waveform types (sine, saw, square, triangle)
- Stereo oscillators with slight detuning for width
- ADSR envelope generator
- Low-pass filter with resonance control

### Advanced Features
- **Animated Filter**: Filter cutoff modulates smoothly using easing curves
- **Preset Management**: ValueTree-based preset storage with XML serialization
- **Preset Verification**: SHA-256 hashing for preset integrity checking
- **MIDI Support**: Full note on/off handling with velocity sensitivity

## Parameters

### Oscillator
- **Waveform**: Choose between sine, saw, square, or triangle

### Filter
- **Cutoff**: Filter cutoff frequency (20 Hz - 20 kHz)
- **Resonance**: Filter resonance/Q factor (0.1 - 10.0)

### Envelope
- **Attack**: Attack time (1 ms - 2 s)
- **Decay**: Decay time (1 ms - 2 s)
- **Sustain**: Sustain level (0.0 - 1.0)
- **Release**: Release time (1 ms - 5 s)

### Output
- **Gain**: Master output gain (-30 dB - +6 dB)

## Building

```bash
cargo xtask bundle juce_multi_module --release
```

## Usage

This plugin demonstrates advanced integration of multiple ported JUCE modules:

1. **Play MIDI notes** to trigger the synthesizer
2. **Adjust the waveform** to change the oscillator character
3. **Modify filter parameters** to shape the tone
4. **Tweak envelope settings** to control the amplitude contour
5. **Observe the animated filter** modulation in action

## Architecture Highlights

### Module Integration
The plugin shows how different ported modules work together:
- DSP modules process audio in real-time
- ValueTree stores and manages preset data
- Animation module creates smooth parameter modulation
- Crypto module verifies preset integrity

### Best Practices
- Proper initialization of all DSP components with sample rate
- Efficient parameter updates only when values change
- Clean separation between DSP, data, and UI concerns
- Thread-safe parameter access using nih-plug's architecture

### Code Organization
```
JuceMultiModule
├── DSP Components (oscillators, filters, envelopes)
├── State Management (ValueTree for presets)
├── Animation System (filter modulation)
├── MIDI Handling (note on/off processing)
└── Parameter Management (smooth automation)
```

## Learning Points

This example teaches:
1. How to combine multiple ported JUCE modules
2. Proper DSP component initialization and lifecycle
3. MIDI event handling in a synthesizer context
4. Using ValueTree for hierarchical data storage
5. Applying animation curves to audio parameters
6. Implementing preset verification with cryptographic hashing
7. Managing complex plugin state across multiple modules

## Future Enhancements

Potential additions to explore:
- Load impulse responses using `logic_nih_plug_audio_formats`
- Add GUI using `logic_nih_plug_gui` components
- Implement preset browser with ValueTree
- Add more complex modulation routing
- Integrate OSC control using `logic_nih_plug_osc`

## What this example ports

- **JUCE source**: this is a workspace-original composite that combines several ported JUCE modules into a single multi-module synthesizer; it demonstrates how a ported `dsp` (oscillator/filter/envelope), a ported `data` (ValueTree + XML), a ported `animation` (easing curves), and a ported `crypto` (SHA-256) module integrate the way the equivalent JUCE source modules (`juce_dsp`, `juce_data_structures`, etc.) do.
- **What to learn from this example**: how to initialise and reset several `Processor`-style components in lock-step with the host sample rate, and how to layer `ValueTree` preset storage on top of `nih_plug` plugin state.

## Running the doc-tests

```bash
cargo test -p juce_multi_module --doc
cargo test -p juce_multi_module
```

The first command runs the doctests embedded in the crate's `lib.rs` (parameter range and signal-flow examples); the second runs the integration suite that drives the synthesizer end-to-end through a `MockAudioIODevice`.

## References

- [`logic_nih_plug_dsp`](../../../logic_nih_plug_dsp/src/lib.rs) — oscillators, IIR filter, ADSR, smoothing
- [`logic_nih_plug_data`](../../../logic_nih_plug_data/src/lib.rs) — ValueTree + XML serialization
- [`logic_nih_plug_animation`](../../../logic_nih_plug_animation/src/lib.rs) — easing curves used for filter modulation
- [`logic_nih_plug_crypto`](../../../logic_nih_plug_crypto/src/lib.rs) — SHA-256 preset verification

## JUCE fidelity checklist

- **Oscillator set**: sine, saw, square, and triangle waveforms are produced by the same `Oscillator` interface used in ported `juce_dsp` modules, with identical naive/integrated anti-aliasing entry points.
- **Envelope ranges**: ADSR attack/decay/release span 1 ms – 2 s (release up to 5 s) and sustain 0.0 – 1.0, matching the JUCE `ADSR` defaults so existing presets port over unchanged.
- **Filter**: a TPT-style state-variable low-pass with resonance (Q) control preserves the topology-preserving behaviour of JUCE's `StateVariableTPTFilter`.
- **Preset integrity**: the SHA-256 hash over the serialised ValueTree bytes reproduces JUCE's "verify on load" step so that tampered presets are rejected just as they would be in a JUCE host.
