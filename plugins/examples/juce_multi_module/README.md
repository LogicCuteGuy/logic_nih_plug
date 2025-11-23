# JUCE Multi-Module Synthesizer Example

This advanced example demonstrates using multiple ported JUCE modules together in a single plugin. It showcases how different modules can work together to create a feature-rich synthesizer.

## Modules Used

### DSP Module (`nih_plug_dsp`)
- **Oscillators**: Generates sine, saw, square, and triangle waveforms
- **Filters**: IIR low-pass filter for tone shaping
- **Envelopes**: ADSR envelope for amplitude control
- **Smoothing**: Parameter smoothing for glitch-free changes

### Data Module (`nih_plug_data`)
- **ValueTree**: Hierarchical preset storage
- **Serialization**: XML-based preset save/load capability

### Animation Module (`nih_plug_animation`)
- **Easing Functions**: Smooth filter cutoff modulation
- **Animation Curves**: InOutQuad easing for natural movement

### Crypto Module (`nih_plug_crypto`)
- **Hashing**: SHA-256 hash generation for preset verification
- **Integrity**: Ensures preset data hasn't been corrupted

### Audio Formats Module (`nih_plug_audio_formats`)
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
- Load impulse responses using `nih_plug_audio_formats`
- Add GUI using `nih_plug_gui` components
- Implement preset browser with ValueTree
- Add more complex modulation routing
- Integrate OSC control using `nih_plug_osc`
