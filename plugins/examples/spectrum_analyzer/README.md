# Spectrum Analyzer Example

This example demonstrates real-time spectrum analysis using FFT from `logic_nih_plug_dsp`. It features:

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

## What this example ports

- **JUCE source**: `examples/DSP/SpectrumAnalyserDemo.h`.
- **What to learn from this example**: how to wire the ported FFT module from `logic_nih_plug_dsp` into a real-time analyser with a Hann window, 75% overlap, and a colour-mapped spectrogram display.

## Parameters

- **FFT size**: select 512, 1024, 2048, or 4096 samples — all power-of-two to match `juce_dsp::FFT`'s contract (req 6.1).
- **Display range (dB)**: floor of the colour map, configurable so low-level content remains visible.
- **Overlap**: 75% (fixed) between successive FFT frames, matching the JUCE demo's smooth temporal resolution.

## Running the doc-tests

```bash
cargo test -p spectrum_analyzer --doc
cargo test -p spectrum_analyzer
```

The `--doc` run executes the doctests in `lib.rs` (FFT setup, windowing notes, dB conversion); the plain `cargo test` runs the integration test that feeds a known sine into `MockAudioIODevice` and verifies the magnitude peak lands on the expected bin (reqs 6.2 and 6.4).

## References

- [`logic_nih_plug_dsp`](../../../logic_nih_plug_dsp/src/lib.rs) — FFT, windowing, magnitude conversion
- [`logic_nih_plug_gui`](../../../logic_nih_plug_gui/src/lib.rs) — colour-mapped spectrogram widget
- [`specs/001-juce-examples/spec.md`](../../../specs/001-juce-examples/spec.md) — JUCE examples validation spec (reqs 6.1, 6.2, 6.4, 10.3)

## JUCE fidelity checklist

- **FFT sizes**: only power-of-two sizes (512/1024/2048/4096) are exposed, matching `juce_dsp::FFT::Size` exactly (req 6.1).
- **Forward transform**: uses the forward complex-to-complex FFT path for time-to-frequency conversion, identical to `juce_dsp::FFT::perform` (req 6.2).
- **Magnitude only**: phase is intentionally discarded — only the magnitude spectrum is displayed, just like the JUCE demo (req 6.4).
- **Windowing and overlap**: a Hann window is applied per frame with 75% overlap, reproducing the spectral-leak reduction and temporal smoothing that the JUCE reference achieves.
