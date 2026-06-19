---
category: DSP
juce_source: examples/DSP/FilterExample.h
---

# `juce_iir_filter_demo` — JUCE-style IIR Biquad Filter

## What this example ports

- **JUCE source file**: [`examples/DSP/FilterExample.h`](https://github.com/juce-framework/JUCE/blob/master/examples/DSP/FilterExample.h)
- **What to learn**: how to implement a second-order IIR filter with real-time parameter smoothing.

## Parameters

| ID | Name | Range | Smoothing |
|---|---|---|---|
| `cutoff` | Cutoff | 20 → 20000 Hz (log) | 50 ms logarithmic |
| `resonance` | Resonance | 0.1 → 20 (log) | 50 ms linear |
| `filter_type` | Type | 0..2 (LP/HP/BP) | — |

## Building

```bash
cargo xtask bundle juce_iir_filter_demo --release
```

## Running the doc-tests

```bash
cargo test --doc -p juce_iir_filter_demo
```

## References

- [JUCE source](https://github.com/juce-framework/JUCE/blob/master/examples/DSP/FilterExample.h)
