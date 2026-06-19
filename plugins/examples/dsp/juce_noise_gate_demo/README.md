---
category: DSP
juce_source: examples/DSP/NoiseGateExample.h
---

# `juce_noise_gate_demo` — JUCE-style Noise Gate / Downward Expander

## What this example ports

- **JUCE source file**: [`examples/DSP/NoiseGateExample.h`](https://github.com/juce-framework/JUCE/blob/master/examples/DSP/NoiseGateExample.h)
- **What to learn**: envelope followers and threshold-based gain reduction.

## Parameters

| ID | Name | Range | Smoothing |
|---|---|---|---|
| `threshold` | Threshold | -80 → 0 dB | 50 ms linear |
| `ratio` | Ratio | 1 → 100 | 50 ms linear |
| `attack` | Attack | 0.1 → 100 ms | 50 ms linear |
| `release` | Release | 10 → 1000 ms | 50 ms linear |

## Building

```bash
cargo xtask bundle juce_noise_gate_demo --release
```

## Running the doc-tests

```bash
cargo test --doc -p juce_noise_gate_demo
```

## References

- [JUCE source](https://github.com/juce-framework/JUCE/blob/master/examples/DSP/NoiseGateExample.h)
