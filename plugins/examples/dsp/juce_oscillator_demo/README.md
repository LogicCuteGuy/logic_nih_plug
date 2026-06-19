---
category: DSP
juce_source: examples/DSP/OscillatorDemo.h
---

# `juce_oscillator_demo` — JUCE-style 4-Waveform Oscillator

## What this example ports

- **JUCE source file**: [`examples/DSP/OscillatorDemo.h`](https://github.com/juce-framework/JUCE/blob/master/examples/DSP/OscillatorDemo.h)
- **What to learn**: how to build a real-time-safe waveform generator with phase accumulation.

## Parameters

| ID | Name | Range | Smoothing |
|---|---|---|---|
| `frequency` | Frequency | 20 → 20000 Hz (log) | 50 ms logarithmic |
| `waveform` | Waveform | 0..3 (Sine/Saw/Square/Triangle) | — |

## Building

```bash
cargo xtask bundle juce_oscillator_demo --release
```

## Running the doc-tests

```bash
cargo test --doc -p juce_oscillator_demo
```

## References

- [JUCE source](https://github.com/juce-framework/JUCE/blob/master/examples/DSP/OscillatorDemo.h)
