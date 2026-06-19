---
category: DSP
juce_source: examples/DSP/ConvolutionExample.h
---

# `juce_convolution_demo` — JUCE-style Convolution Reverb

## What this example ports

- **JUCE source file**: [`examples/DSP/ConvolutionExample.h`](https://github.com/juce-framework/JUCE/blob/master/examples/DSP/ConvolutionExample.h)
- **What to learn**: time-domain convolution with a synthetic impulse response.

## Parameters

| ID | Name | Range | Smoothing |
|---|---|---|---|
| `ir_length` | IR Length | 64 → 8192 samples | — |
| `decay_time` | Decay Time | 0.05 → 5 s | — |

## Building

```bash
cargo xtask bundle juce_convolution_demo --release
```

## Running the doc-tests

```bash
cargo test --doc -p juce_convolution_demo
```

## References

- [JUCE source](https://github.com/juce-framework/JUCE/blob/master/examples/DSP/ConvolutionExample.h)
