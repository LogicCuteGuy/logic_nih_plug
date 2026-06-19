---
category: DSP
juce_source: examples/DSP/OverdriveDemo.h
---

# `juce_distortion_demo` — JUCE-style Soft-Clip Distortion

## What this example ports

- **JUCE source file**: [`examples/DSP/OverdriveDemo.h`](https://github.com/juce-framework/JUCE/blob/master/examples/DSP/OverdriveDemo.h)
- **What to learn from this example**: how to wrap a non-linear wave-shaper
  (here: `tanh`) in a real-time-safe nih-plug plugin, with a `FloatParam`
  input drive and a separate output gain.

## Parameters

| ID | Name | Range | Smoothing |
|---|---|---|---|
| `drive` | Drive | 1.0 → 10.0 (log) | 50 ms logarithmic |
| `output` | Output | 0.0 → 1.0 (linear) | 50 ms linear |

## Building

```bash
cargo xtask bundle juce_distortion_demo --release
```

## Running the doc-tests

```bash
cargo test --doc -p juce_distortion_demo
```

## References

- [JUCE source](https://github.com/juce-framework/JUCE/blob/master/examples/DSP/OverdriveDemo.h)
- [`logic_nih_plug_dsp::processors::waveshaper`](../../../../logic_nih_plug_dsp/src/processors/waveshaper.rs)
- [`specs/001-juce-examples/spec.md`](../../../../specs/001-juce-examples/spec.md) — feature spec
- [`specs/001-juce-examples/contracts/example-crate-contract.md`](../../../../specs/001-juce-examples/contracts/example-crate-contract.md)

## JUCE fidelity checklist

- [x] **Source file named**: `examples/DSP/OverdriveDemo.h`
- [x] **Public API surface unchanged**: example calls only types/methods that
      exist in the current `logic_nih_plug*` sub-crate tree; no private API
      is touched.
- [x] **Matches JUCE behavior** (per constitution §V): the `soft_clip`
      function matches JUCE's `tanh` clipping; doc-tests prove symmetry and
      peak reduction.
- [x] **Skipped modules**: none
- [x] **One behavioral doc-test passes**: `cargo test --doc -p juce_distortion_demo` exits 0
