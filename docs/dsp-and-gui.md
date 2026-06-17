# DSP & GUI Modules

JUCE-ported crates live in this workspace. All use `thiserror = "1.0"` and ship
`criterion` benches. Source: their `src/lib.rs` module docs.

## DSP — `nih_plug_dsp`

[Cargo.toml](../nih_plug_dsp/Cargo.toml). Features:

- `oscillators` (default), `filters` (default)
- `simd` — uses `std::simd`, requires nightly
- `envelopes`, `convolution`, `analysis`, `processors`, `util`, `state_variable`
- `full` — all of the above

Public surface: [API_REFERENCE.md](../API_REFERENCE.md) (upstream).

What lives here:

| File | What's in it |
|---|---|
| [filters.rs](../nih_plug_dsp/src/filters.rs) | Biquad family, RBJ cookbook shapes |
| [state_variable.rs](../nih_plug_dsp/src/state_variable_state.rs) | SVF (TPT) |
| [oscillators.rs](../nih_plug_dsp/src/oscillators.rs) | Sine, saw, square, triangle, wavetable |
| [envelopes.rs](../nih_plug_dsp/src/envelopes.rs) | AR, ADSR |
| [convolution.rs](../nih_plug_dsp/src/convolution.rs) | FFT-based convolution |
| [smoothing.rs](../nih_plug_dsp/src/smoothing.rs) | `Smoother` (one-pole, log, linear, exponential) |
| [analysis/](../nih_plug_dsp/src/analysis/) | Spectrum, peak, RMS |
| [simd/](../nih_plug_dsp/src/simd/) | SIMD adapters, gated by `simd` |

Tests live under [nih_plug_dsp/tests](../nih_plug_dsp/tests/) — `property_tests.rs`,
JUCE-validation tests, and per-module suites.

## GUI backends — pick one

| Crate | Backend | Adapter |
|---|---|---|
| [nih_plug_egui](../nih_plug_egui/) | egui | egui-baseview |
| [nih_plug_iced](../nih_plug_iced/) | iced | iced_baseview (OpenGL by default; `wgpu` opt-in) |
| [nih_plug_vizia](../nih_plug_vizia/) | VIZIA | built-in |

`nih_plug_egui` ships a `ResizableWindow` widget (added 2025-02-23). Egui 0.31+
required.

## BYO-GUI — `nih_plug_gui`

[Cargo.toml](../nih_plug_gui/Cargo.toml). Features: `components`, `layout`,
`graphics`, `text`, `softbuffer-editor`, `gl-editor`, `full`.

- [CONTROLS.md](../nih_plug_gui/CONTROLS.md) — custom controls reference
- [LOOKANDFEEL.md](../nih_plug_gui/LOOKANDFEEL.md) — theming
- BYO-GUI examples: [byo_gui_gl](../plugins/examples/byo_gui_gl/),
  [byo_gui_softbuffer](../plugins/examples/byo_gui_softbuffer/),
  [byo_gui_wgpu](../plugins/examples/byo_gui_wgpu/)

## Animation — `nih_plug_animation`

[README](../nih_plug_animation/README.md). `Animation` + `AnimationSequence`
+ 30+ easing functions. Demo: `cargo run --example animation_demo --features full`.

## Audio I/O & graphics

- `nih_plug_audio_formats` — WAV/AIFF (FLAC, OGG optional)
- `nih_plug_graphics` — 2D primitives

## Tests

For per-crate test commands, see [getting-started.md](getting-started.md).
