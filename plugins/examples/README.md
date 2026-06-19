---
category: Plugins
---

# `plugins/examples/` — Workspace Plugin Examples

This directory is the workspace's **plugin examples** gallery. Every crate here
is a runnable VST3 + CLAP plugin (`crate-type = ["cdylib"]`) that demonstrates
a specific feature of the `logic_nih_plug` framework, often by porting a
[JUCE `examples/`](https://github.com/juce-framework/JUCE/tree/master/examples)
reference.

> **Looking for the standalone app examples?** See
> [`/examples/`](../../examples/README.md) — audio apps, file-IO demos, the
> plugin host, and the DemoRunner.

## Start here

If you are new to the framework, the recommended reading order is:

1. [`gain`](./gain) — the absolute minimum plugin (`Plugin` + `Params` + a
   `FloatParam` with smoothing). No editor. ~80 lines of `lib.rs`.
2. [`gain_gui_egui`](./gain_gui_egui) — `gain` with an `egui` editor that
   binds a slider to the `FloatParam`.
3. [`juce_dsp_filter`](./juce_dsp_filter) — the same `gain` shape but with
   real DSP (IIR filter from `logic_nih_plug_dsp`).
4. [`juce_multi_module`](./juce_multi_module) — a polyphonic synth that
   composes DSP, ValueTree persistence, animation, and crypto into one crate.

Once those make sense, browse the categories below.

## DSP plugins (mirrors `JUCE/examples/DSP/`)

Crate | Mirrors JUCE example | What it demonstrates
---|---|---
[`juce_dsp_filter`](./juce_dsp_filter) | `examples/DSP/IIRFilterDemo.h` | Biquad IIR filter with parameter smoothing
[`juce_distortion_demo`](./dsp/juce_distortion_demo) | `examples/DSP/OverdriveDemo.h` | Soft-clip oversampled distortion
[`juce_oscillator_demo`](./dsp/juce_oscillator_demo) | `examples/DSP/OscillatorDemo.h` | 4-waveform generator
[`juce_iir_filter_demo`](./dsp/juce_iir_filter_demo) | `examples/DSP/IIRFilterDemo.h` | Biquad low-pass/high-pass/band-pass
[`juce_phaser_demo`](./dsp/juce_phaser_demo) | `examples/DSP/PhaserDemo.h` | 4-stage all-pass cascade with LFO
[`juce_chorus_demo`](./dsp/juce_chorus_demo) | `examples/DSP/ChorusDemo.h` | Modulated delay line with feedback
[`juce_convolution_demo`](./dsp/juce_convolution_demo) | `examples/DSP/ConvolutionDemo.h` | FFT partition convolution
[`juce_noise_gate_demo`](./dsp/juce_noise_gate_demo) | `examples/DSP/NoiseGateDemo.h` | Envelope-follower + hysteresis
[`juce_limiter_demo`](./dsp/juce_limiter_demo) | `examples/DSP/LimiterDemo.h` | Lookahead brickwall limiter
[`overdrive`](./overdrive) | `examples/DSP/OverdriveDemo.h` | Pre-existing overdrive example (kept for reference)
[`state_variable_filter`](./state_variable_filter) | `examples/DSP/StateVariableFilterDemo.h` | SVF in TPT form
[`sine`](./sine) | — | Pure sine generator (no JUCE equivalent)
[`stft`](./stft) | `examples/Audio/SimpleFFTDemo.h` | STFT spectrogram (pre-existing)
[`spectrum_analyzer`](./spectrum_analyzer) | — | FFT spectrum display (pre-existing)
[`poly_mod_synth`](./poly_mod_synth) | — | Polyphonic mod synth (pre-existing)
[`delay`](./delay) | — | Delay line (pre-existing)
[`reverb`](./reverb) | — | Reverb (pre-existing)
[`chorus`](./chorus) | — | Chorus (pre-existing)
[`sidechain_compressor`](./sidechain_compressor) | — | Sidechain compressor (pre-existing)
[`midi_inverter`](./midi_inverter) | — | MIDI inverter (pre-existing)
[`sysex`](./sysex) | — | SysEx demo (pre-existing)
[`note_expressions`](./note_expressions) | — | Note expressions (pre-existing)

## GUI plugins (mirrors `JUCE/examples/GUI/`)

Crate | Mirrors JUCE example | What it demonstrates
---|---|---
[`juce_gui_demo`](./juce_gui_demo) | — | A showcase plugin exercising every GUI primitive
[`juce_flexbox_demo`](./flexbox_demo) | `examples/GUI/FlexBoxDemo.h` | FlexBox layout
[`flexbox_demo`](./flexbox_demo) | `examples/GUI/FlexBoxDemo.h` | (alias; pre-existing)
[`byo_gui_gl`](./byo_gui_gl) | — | BYO-GUI with `glutin`
[`byo_gui_softbuffer`](./byo_gui_softbuffer) | — | BYO-GUI with `softbuffer`
[`byo_gui_wgpu`](./byo_gui_wgpu) | — | BYO-GUI with `wgpu`
[`gain_gui_egui`](./gain_gui_egui) | — | Gain + egui editor (canonical reference)
[`gain_gui_iced`](./gain_gui_iced) | — | Gain + iced editor
[`gain_gui_vizia`](./gain_gui_vizia) | — | Gain + vizia editor

## Plugin-format demos

Crate | What it demonstrates
---|---
[`gain`](./gain) | Default VST3 + CLAP export
[`gain_multi_format`](./gain_multi_format) | All formats (VST3, CLAP, AU, AUv3, LV2, AAX, VST2) at once
[`gain_vst2`](./gain_vst2) | VST2 export (requires local Steinberg SDK)
[`gain_au`](./gain_au) | macOS AU export
[`gain_auv3`](./gain_auv3) | iOS AUv3 export
[`gain_lv2`](./gain_lv2) | Linux LV2 export
[`gain_aax`](./gain_aax) | AAX export (stub; requires Avid SDK)

## Composite / multi-module

Crate | What it demonstrates
---|---
[`juce_multi_module`](./juce_multi_module) | Polyphonic synth combining DSP + ValueTree + crypto + animation

## Building a plugin

```bash
cargo xtask bundle <crate name> --release
```

The output is written to `target/bundled/`.

## Running the doc-tests

```bash
cargo test --doc -p <crate name>
# or, for every example:
cargo xtask test-examples
```

## Categories

Every crate here is one of:

- **`DSP`** — a plugin that does audio processing.
- **`GUI`** — a plugin whose editor demonstrates a GUI primitive.
- **`Plugins`** — a plugin that exercises a plugin-format feature.
- **`Audio`** — a plugin whose output is a self-contained audio app.
- **`Utilities`** — a plugin that wraps a cross-cutting utility.
- **`DemoRunner`** — a single plugin that demonstrates every category above.

The category is recorded in the README's YAML front-matter (`category: DSP`)
and validated by `tests/example_categorized.rs`.

## How to add a new example

1. Pick the JUCE example to port (see
   [`specs/001-juce-examples/example-inventory.md`](../../specs/001-juce-examples/example-inventory.md)
   for the current ledger).
2. Copy a minimal existing example crate (`gain` is the smallest).
3. Add a top-level `README.md` matching
   [`specs/001-juce-examples/contracts/example-crate-contract.md`](../../specs/001-juce-examples/contracts/example-crate-contract.md)
   (5 sections + front-matter).
4. Add the crate to `bundler.toml` if it's a plugin.
5. Update the ledger row's `status` to `ported`.
6. Run `cargo xtask test-examples` to confirm doc-tests pass.

## References

- [`specs/001-juce-examples/spec.md`](../../specs/001-juce-examples/spec.md) — feature spec
- [`specs/001-juce-examples/plan.md`](../../specs/001-juce-examples/plan.md) — implementation plan
- [`specs/001-juce-examples/example-inventory.md`](../../specs/001-juce-examples/example-inventory.md) — ledger
- [`specs/001-juce-examples/contracts/example-crate-contract.md`](../../specs/001-juce-examples/contracts/example-crate-contract.md)
- [`AGENTS.md`](../../AGENTS.md) — workspace rules
- [JUCE `examples/`](https://github.com/juce-framework/JUCE/tree/master/examples)
