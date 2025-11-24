# Overdrive Effect Example

This example demonstrates processor chain composition by creating an overdrive effect using the `nih_plug_dsp` processor chain system.

## Signal Chain

The overdrive effect uses the following signal processing chain:

```
Input → Gain (Drive) → Bias → WaveShaper → DC Filter → Gain (Output) → Output
```

### Chain Components

1. **Input Gain (Drive)**: Amplifies the input signal to drive the wave shaper harder
2. **Bias**: Adds DC offset for asymmetric distortion characteristics
3. **WaveShaper**: Applies tanh saturation for smooth, analog-style distortion
4. **DC Filter**: Removes unwanted DC offset introduced by asymmetric distortion
5. **Output Gain**: Controls the final output level

## Parameters

- **Drive** (0-24 dB): Controls how hard the signal drives the distortion
- **Bias** (-0.5 to 0.5): Adds DC offset for asymmetric distortion
- **Output** (-24 to 12 dB): Controls the output level

## Requirements Validated

This example validates the following requirements from the JUCE examples validation spec:

- **3.4**: Processor chain composition
- **4.2**: Sequential audio processing through chain
- **5.1**: Bias processor for DC offset
- **6.1**: Wave shaper with custom transfer functions
- **11.1**: DC filter for removing DC offset
- **12.1**: Gain processor with decibel control

## Building

```bash
cargo xtask bundle overdrive --release
```

## Usage

The plugin can be loaded in any DAW that supports CLAP or VST3 plugins. Try these settings:

- **Clean Boost**: Drive=6dB, Bias=0.0, Output=-6dB
- **Warm Saturation**: Drive=12dB, Bias=0.1, Output=-12dB
- **Heavy Overdrive**: Drive=18dB, Bias=0.2, Output=-18dB
- **Asymmetric Distortion**: Drive=12dB, Bias=0.3, Output=-12dB

## Implementation Notes

This example demonstrates:

- Composing multiple processors in sequence to create a complex effect
- Configuring processors with appropriate parameters
- Processing audio through a chain of processors
- Parameter smoothing for click-free parameter changes
- Proper initialization and reset of processor state

The implementation processes audio through each processor in sequence:
1. Each sample passes through the input gain
2. Then through the bias processor
3. Then through the wave shaper
4. Then through the DC filter
5. Finally through the output gain

This architecture demonstrates the processor chain concept and makes it easy to understand the signal flow. The modular design allows for easy modification of the chain by adding, removing, or reordering processors.
