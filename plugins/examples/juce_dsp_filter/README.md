# JUCE DSP Filter Example

This example demonstrates using the ported JUCE DSP modules in a nih-plug plugin.

## Features

- **IIR Filter**: Uses the ported JUCE IIR filter implementation
- **Low-Pass Filter**: Second-order Butterworth-style low-pass filter
- **Smooth Parameter Changes**: Demonstrates parameter smoothing
- **Stereo Processing**: Independent left and right channel filtering
- **Coefficient Calculation**: Shows how to calculate filter coefficients using bilinear transform

## Parameters

- **Cutoff**: Filter cutoff frequency (20 Hz - 20 kHz)
- **Resonance**: Filter resonance/Q factor (0.1 - 10.0)

## Building

```bash
cargo xtask bundle juce_dsp_filter --release
```

## Usage

This plugin demonstrates the basic usage of the `nih_plug_dsp` crate's filter module. The filter coefficients are calculated using the ported JUCE algorithms and applied to the audio stream in real-time.

The plugin showcases:
- Initializing DSP components with sample rate
- Updating filter coefficients based on parameter changes
- Processing audio samples through the filter
- Resetting filter state when needed
