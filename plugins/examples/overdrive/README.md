# Overdrive Effect Example

This example demonstrates processor chain composition by creating an overdrive effect using the `logic_nih_plug_dsp` processor chain system.

## Signal Chain

The overdrive effect uses the following signal processing chain:

```
Input → Gain (Drive) → Bias → WaveShaper → DC Filter → Gain (Output) → Output
```

### Chain Components

1. **Input Gain (Drive)**: Amplifies the input signal to drive the wave shaper harder
2. **Bias**: Adds DC offset for asymmetric distortion characteristics
3. **WaveShaper**: Applies tanh saturation for smooth, analog-style distortion
4. **DC Filter**: Removes unwanted DC offset introduced by asymmetric distortion
5. **Output Gain**: Controls the final output level

## Parameters

- **Drive** (0-24 dB): Controls how hard the signal drives the distortion
- **Bias** (-0.5 to 0.5): Adds DC offset for asymmetric distortion
- **Output** (-24 to 12 dB): Controls the output level

## Requirements Validated

This example validates the following requirements from the JUCE examples validation spec:

- **3.4**: Processor chain composition
- **4.2**: Sequential audio processing through chain
- **5.1**: Bias processor for DC offset
- **6.1**: Wave shaper with custom transfer functions
- **11.1**: DC filter for removing DC offset
- **12.1**: Gain processor with decibel control

## Building

```bash
cargo xtask bundle overdrive --release
```

## Usage

The plugin can be loaded in any DAW that supports CLAP or VST3 plugins. Try these settings:

- **Clean Boost**: Drive=6dB, Bias=0.0, Output=-6dB
- **Warm Saturation**: Drive=12dB, Bias=0.1, Output=-12dB
- **Heavy Overdrive**: Drive=18dB, Bias=0.2, Output=-18dB
- **Asymmetric Distortion**: Drive=12dB, Bias=0.3, Output=-12dB

## Implementation Notes

This example demonstrates:

- Composing multiple processors in sequence to create a complex effect
- Configuring processors with appropriate parameters
- Processing audio through a chain of processors
- Parameter smoothing for click-free parameter changes
- Proper initialization and reset of processor state

The implementation processes audio through each processor in sequence:
1. Each sample passes through the input gain
2. Then through the bias processor
3. Then through the wave shaper
4. Then through the DC filter
5. Finally through the output gain

This architecture demonstrates the processor chain concept and makes it easy to understand the signal flow. The modular design allows for easy modification of the chain by adding, removing, or reordering processors.

## What this example ports

- **JUCE source**: `examples/ DSP/PluginProcessorChainDemo.h` (processor-chain composition).
- **What to learn from this example**: how to compose the ported `Gain`, `Bias`, `WaveShaper`, and `DCFilter` processors from `logic_nih_plug_dsp` into a single named chain, in the same order the JUCE demo uses.

## Running the doc-tests

```bash
cargo test -p overdrive --doc
cargo test -p overdrive
```

The `--doc` run executes the doctests in the crate's `lib.rs` (chain ordering, parameter ranges, and signal-flow snippets); the plain `cargo test` run executes the integration test that drives the full drive → bias → waveshaper → DC filter → output chain through `MockAudioIODevice`.

## References

- [`logic_nih_plug_dsp`](../../../logic_nih_plug_dsp/src/lib.rs) — `Gain`, `Bias`, `WaveShaper`, `DCFilter` processors
- [`plugins/examples/overdrive/src/lib.rs`](../../../plugins/examples/overdrive/src/lib.rs) — chain composition entry point
- [`specs/001-juce-examples/spec.md`](../../../specs/001-juce-examples/spec.md) — JUCE examples validation spec (reqs 3.4, 4.2, 5.1, 6.1, 11.1, 12.1)

## JUCE fidelity checklist

- **Signal chain order**: input gain → bias → waveshaper → DC filter → output gain, matching the JUCE `PluginProcessorChainDemo` order so that the bias-shaping-then-DC-removal invariant is preserved.
- **Drive range**: drive gain spans 0–24 dB, exactly as the JUCE demo's input gain stage.
- **Bias range**: bias offset is clamped to −0.5 … +0.5 to keep the asymmetry behaviour identical to the upstream example.
- **DC removal**: the post-shaper `DCFilter` is configured with the same time constant as the JUCE reference, so low-frequency content is restored equivalently after asymmetric clipping.
