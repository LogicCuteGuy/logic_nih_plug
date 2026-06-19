---
category: DemoRunner
juce_source: examples/DemoRunner/Source/*
---

# `juce_demorunner` — GUI Showcase (egui/iced/vizia)

## What this example ports

- **JUCE source file**: `examples/DemoRunner/Source/*` (all categories)
- **What to learn**: how to wire every public component from
  `logic_nih_plug_gui` into a single showcase app, with a
  feature-flag-selected backend (egui default, iced, vizia).

## Architecture

The crate is split into 3 modules:

| Module | Purpose |
|---|---|
| [`backend`](src/backend/mod.rs) | Runtime backend selection |
| [`nav`](src/nav.rs) | Top-level navigation (category list → page) |
| [`showcase`](src/showcase/mod.rs) | 5 categories × N demos registry |

The 5 categories mirror the JUCE DemoRunner:

- **Controls** — Slider, Knob, ToggleButton
- **Layouts** — FlexBox, CssGrid
- **Animation** — Eased knob, Waveform morph (uses `ease_in_out_quad`)
- **Graphics** — Painter gradient, Path stroke
- **AudioViz** — LevelMeter, Oscilloscope, Spectrum

## Backends

Exactly one of `gui-egui`, `gui-iced`, `gui-vizia` must be enabled
(mutually exclusive via `compile_error!`). The default is
`gui-egui`.

```bash
cargo run -p juce_demorunner                          # default (egui)
cargo run -p juce_demorunner --no-default-features --features gui-iced
cargo run -p juce_demorunner --no-default-features --features gui-vizia
```

## Running

```bash
cargo run -p juce_demorunner
```

## Running the tests

```bash
cargo test -p juce_demorunner
```

## References

- [`logic_nih_plug_gui`](../../../logic_nih_plug_gui/src/lib.rs) — components
- [`logic_nih_plug_dsp::analysis`](../../../logic_nih_plug_dsp/src/analysis/mod.rs) — LevelMeter + Oscilloscope
- [`logic_nih_plug_animation`](../../../logic_nih_plug_animation/src/lib.rs) — easing
- [`logic_nih_plug_graphics`](../../../logic_nih_plug_graphics/src/lib.rs) — Painter
- [`specs/001-juce-examples/spec.md`](../../../../specs/001-juce-examples/spec.md) — feature spec