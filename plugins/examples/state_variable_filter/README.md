# State Variable Filter Example

This example plugin demonstrates the state variable filter (TPT) from `logic_nih_plug_dsp` with a real-time frequency response visualization using `logic_nih_plug_gui`.

## Features

- **State Variable Filter**: Uses the Topology-Preserving Transform (TPT) method for stable filtering
- **Multiple Filter Types**: Lowpass, Bandpass, and Highpass modes
- **Real-time Visualization**: Frequency response display that updates as parameters change using OpenGL rendering
- **Smooth Parameter Control**: Cutoff frequency and resonance with smooth transitions
- **Stereo Processing**: Independent filtering for left and right channels
- **JUCE-style GUI**: Uses logic_nih_plug_gui (ported JUCE GUI components) for the interface

## Parameters

- **Filter Type**: Choose between Lowpass, Bandpass, and Highpass
- **Cutoff Frequency**: 20 Hz to 20 kHz (logarithmic scale)
- **Resonance**: 0.1 to 1.0 (Q factor)

## Building

Build the plugin using cargo-nih-plug:

```bash
cargo xtask bundle state_variable_filter --release
```

## Technical Details

The plugin uses the TPT (Topology-Preserving Transform) state variable filter implementation, which ensures stability at all parameter settings. The filter can produce lowpass, bandpass, and highpass outputs simultaneously by selecting different internal nodes.

The frequency response visualization calculates the magnitude response at multiple frequency points and displays them in real-time as parameters change.

## Requirements Validated

This example validates the following requirements from the JUCE examples validation spec:

- **Requirement 1.1**: State variable filter with multiple filter types
- **Requirement 1.2**: Smooth filter type changes without clicks
- **Requirement 1.3**: Stable filtering at all parameter settings
- **Requirement 10.3**: Example demonstrating ported JUCE DSP features

## What this example ports

- **JUCE source**: `examples/DSP/StateVariableFilterDemo.h`.
- **What to learn from this example**: how to drive the ported TPT state-variable filter from `logic_nih_plug_dsp` with `nih_plug` parameters, and how to mirror the frequency-response visualiser from the JUCE demo using `logic_nih_plug_gui`'s OpenGL widgets.

## Running the doc-tests

```bash
cargo test -p state_variable_filter --doc
cargo test -p state_variable_filter
```

The `--doc` run covers the embedded doctests in `lib.rs` (cutoff/resonance ranges and TPT stability notes); the plain `cargo test` runs the integration suite that steps the cutoff and asserts the magnitude response matches the analytical transfer function.

## References

- [`logic_nih_plug_dsp`](../../../logic_nih_plug_dsp/src/lib.rs) — TPT state-variable filter
- [`logic_nih_plug_gui`](../../../logic_nih_plug_gui/src/lib.rs) — OpenGL frequency-response widget
- [`specs/001-juce-examples/spec.md`](../../../specs/001-juce-examples/spec.md) — JUCE examples validation spec (reqs 1.1, 1.2, 1.3, 10.3)

## JUCE fidelity checklist

- **Topology**: uses the TPT (Topology-Preserving Transform) state-variable structure, the same one as JUCE's `StateVariableTPTFilter`, so the filter is stable at every cutoff/Q setting (req 1.3).
- **Modes**: lowpass, bandpass, and highpass are produced by selecting different internal nodes of the same SVF core, mirroring JUCE's `setType()` API.
- **Parameter ranges**: cutoff 20 Hz – 20 kHz on a logarithmic scale and resonance/Q 0.1 – 1.0, matching the JUCE demo so existing presets transfer unchanged.
- **Smoothing**: cutoff and resonance changes are smoothed across blocks so that filter-type or cutoff switches remain click-free (req 1.2).
