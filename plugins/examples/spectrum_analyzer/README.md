# Spectrum Analyzer Example

This example demonstrates real-time spectrum analysis using FFT from `nih_plug_dsp`. It features:

- **Real-time FFT Analysis**: Uses the FFT module for frequency-domain analysis
- **Spectrogram Display**: Visual representation of frequency content over time with color mapping
- **Windowing**: Applies Hann window to reduce spectral leakage
- **Overlap-Add**: Uses 75% overlap for smooth temporal resolution
- **Frequency and Magnitude Axes**: Clear labeling of frequency (Hz) and magnitude (dB)

## Features

- FFT size selection (512, 1024, 2048, 4096)
- Adjustable display range (dB)
- Color-mapped spectrogram showing frequency content over time
- Logarithmic frequency axis for better visualization
- Real-time magnitude spectrum display

## Requirements

This example validates:
- Requirements 6.1: Power-of-2 FFT sizes
- Requirements 6.2: Forward FFT for time-to-frequency conversion
- Requirements 6.4: Magnitude spectrum without phase
- Requirements 10.3: Example demonstrating FFT features

## Building

```bash
cargo xtask bundle spectrum_analyzer --release
```

## Usage

The plugin processes incoming audio and displays its frequency content in real-time. The spectrogram shows how the frequency content changes over time, with brighter colors indicating higher magnitudes.
