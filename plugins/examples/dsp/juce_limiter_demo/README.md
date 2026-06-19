---
category: DSP
juce_source: examples/DSP/LimiterExample.h
---

# `juce_limiter_demo` — JUCE-style Brickwall Limiter

## What this example ports

- **JUCE source file**: [`examples/DSP/LimiterExample.h`](https://github.com/juce-framework/JUCE/blob/master/examples/DSP/LimiterExample.h)
- **What to learn**: peak detection with smoothed gain reduction for brickwall limiting.

## Parameters

| ID | Name | Range | Smoothing |
|---|---|---|---|
| `ceiling` | Ceiling | -20 → 0 dB | 1 ms logarithmic |
| `release` | Release | 5 → 500 ms | 50 ms linear |

## Building

```bash
cargo xtask bundle juce_limiter_demo --release
```

## Running the doc-tests

```bash
cargo test --doc -p juce_limiter_demo
```

## References

- [JUCE source](https://github.com/juce-framework/JUCE/blob/master/examples/DSP/LimiterExample.h)
