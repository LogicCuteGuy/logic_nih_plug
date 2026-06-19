---
category: DSP
juce_source: examples/DSP/ChorusExample.h
---

# `juce_chorus_demo` — JUCE-style Modulated Delay Chorus

## What this example ports

- **JUCE source file**: [`examples/DSP/ChorusExample.h`](https://github.com/juce-framework/JUCE/blob/master/examples/DSP/ChorusExample.h)
- **What to learn**: LFO-modulated delay lines for chorus, flanger, and vibrato effects.

## Parameters

| ID | Name | Range | Smoothing |
|---|---|---|---|
| `rate` | Rate | 0.05 → 10 Hz | 50 ms linear |
| `depth` | Depth | 0 → 1 | 50 ms linear |
| `centre_delay` | Centre Delay | 1 → 50 ms | 50 ms linear |
| `feedback` | Feedback | -0.95 → 0.95 | 50 ms linear |
| `mix` | Mix | 0 → 1 | 50 ms linear |

## Building

```bash
cargo xtask bundle juce_chorus_demo --release
```

## Running the doc-tests

```bash
cargo test --doc -p juce_chorus_demo
```

## References

- [JUCE source](https://github.com/juce-framework/JUCE/blob/master/examples/DSP/ChorusExample.h)
