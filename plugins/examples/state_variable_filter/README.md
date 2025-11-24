# State Variable Filter Example

This example plugin demonstrates the state variable filter (TPT) from `nih_plug_dsp` with a real-time frequency response visualization using `nih_plug_gui`.

## Features

- **State Variable Filter**: Uses the Topology-Preserving Transform (TPT) method for stable filtering
- **Multiple Filter Types**: Lowpass, Bandpass, and Highpass modes
- **Real-time Visualization**: Frequency response display that updates as parameters change using OpenGL rendering
- **Smooth Parameter Control**: Cutoff frequency and resonance with smooth transitions
- **Stereo Processing**: Independent filtering for left and right channels
- **JUCE-style GUI**: Uses nih_plug_gui (ported JUCE GUI components) for the interface

## Parameters

- **Filter Type**: Choose between Lowpass, Bandpass, and Highpass
- **Cutoff Frequency**: 20 Hz to 20 kHz (logarithmic scale)
- **Resonance**: 0.1 to 1.0 (Q factor)

## Building

Build the plugin using cargo-nih-plug:

```bash
cargo xtask bundle state_variable_filter --release
```

## Technical Details

The plugin uses the TPT (Topology-Preserving Transform) state variable filter implementation, which ensures stability at all parameter settings. The filter can produce lowpass, bandpass, and highpass outputs simultaneously by selecting different internal nodes.

The frequency response visualization calculates the magnitude response at multiple frequency points and displays them in real-time as parameters change.

## Requirements Validated

This example validates the following requirements from the JUCE examples validation spec:

- **Requirement 1.1**: State variable filter with multiple filter types
- **Requirement 1.2**: Smooth filter type changes without clicks
- **Requirement 1.3**: Stable filtering at all parameter settings
- **Requirement 10.3**: Example demonstrating ported JUCE DSP features
