---
category: DSP
juce_source: examples/DSP/PhaserExample.h
---

# `juce_phaser_demo` — JUCE-style 6-Stage Allpass Phaser

## What this example ports

- **JUCE source file**: [`examples/DSP/PhaserExample.h`](https://github.com/juce-framework/JUCE/blob/master/examples/DSP/PhaserExample.h)
- **What to learn**: LFO-modulated allpass filter cascades for phasing effects.

## Parameters

| ID | Name | Range | Smoothing |
|---|---|---|---|
| `rate` | Rate | 0.01 → 5 Hz | 50 ms linear |
| `depth` | Depth | 0 → 1 | 50 ms linear |
| `centre_freq` | Centre Freq | 200 → 8000 Hz (log) | 50 ms logarithmic |
| `feedback` | Feedback | 0 → 0.95 | 50 ms linear |
| `mix` | Mix | 0 → 1 | 50 ms linear |

## Building

```bash
cargo xtask bundle juce_phaser_demo --release
```

## Running the doc-tests

```bash
cargo test --doc -p juce_phaser_demo
```

## References

- [JUCE source](https://github.com/juce-framework/JUCE/blob/master/examples/DSP/PhaserExample.h)
