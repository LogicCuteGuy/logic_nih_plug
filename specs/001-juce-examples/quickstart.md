# Quickstart: JUCE-Style Examples Portfolio

**Feature**: [spec.md](./spec.md) | **Date**: 2026-06-19

This is a "I want to write X" learning path through the portfolio. Pick the row
that matches what you want to do; the linked example is the recommended starting
point.

## 1. Newcomer paths

| If you want to… | Start here | Then read | Then read |
|---|---|---|---|
| See the simplest possible plugin | `plugins/examples/dsp/juce_gain_demo` | `plugins/examples/juce_dsp_filter` | `plugins/examples/gain` |
| See a plugin with a GUI editor | `plugins/examples/gui/juce_flexbox_demo` | `plugins/examples/gain_gui_egui` | `plugins/examples/juce_gui_demo` |
| See a plugin with state persistence | `plugins/examples/gui/juce_component_demo` | `plugins/examples/gain` (uses `#[persist]`) | `plugins/examples/juce_multi_module` |
| Build a host (load other plugins) | `examples/Plugins/juce_audio_plugin_host_egui` | `examples/Plugins/plugin_host_cli` | (no further host example yet) |
| Build a standalone audio app | `examples/Audio/audio_playback_demo` | `examples/Audio/audio_recording_demo` | `examples/Audio/audio_workgroup_demo` |
| Read a WAV/AIFF file | `examples/Utilities/wav_reader` | `examples/Utilities/wav_writer` | `examples/Audio/audio_playback_demo` |
| Send/receive OSC | `examples/Utilities/osc_sender_demo` | `examples/Utilities/osc_receiver_demo` | (paired in a doc-test) |
| See every GUI widget in one app | `examples/DemoRunner/juce_demorunner` (egui) | same with `--features gui-iced` | same with `--features gui-vizia` |
| See every DSP category in one app | `examples/DemoRunner/juce_demorunner` "DSP" category | any of the 9 DSP examples | (then go read the sub-crate) |

## 2. By user story

| User story | Examples | MVP? |
|---|---|---|
| US1 — DSP Examples Portfolio | 8 plugin examples (distortion, oscillator, IIR, phaser, chorus, convolution, noise-gate, limiter) | ✓ MVP |
| US2 — Standalone Audio Apps | 3 standalone apps (playback, recording, workgroup) | |
| US3 — Plugin Host | 1 headless CLI host + 1 egui GUI host | |
| US4 — Audio & MIDI File Format Demos | 5 CLI demos (wav_reader, wav_writer, midi_file_inspector, osc_sender, osc_receiver) | |
| US5 — GUI DemoRunner | 1 showcase crate with 3 backends, 5 categories, ≥10 demos | |

## 3. Common commands

```bash
# Build + bundle a plugin example
cargo xtask bundle juce_distortion_demo --release

# Run a standalone example
cargo run -p audio_playback_demo -- examples/audio-assets/sine_1khz_1s.wav

# Run the DemoRunner with the default (egui) backend
cargo run -p juce_demorunner

# Run the DemoRunner with a different backend
cargo run -p juce_demorunner --no-default-features --features gui-iced
cargo run -p juce_demorunner --no-default-features --features gui-vizia

# Run doc-tests for one example
cargo test --doc -p juce_distortion_demo

# Run doc-tests for one user story's examples
cargo xtask test-examples --category DSP
cargo xtask test-examples --category Audio
cargo xtask test-examples --category Plugins
cargo xtask test-examples --category Utilities
cargo xtask test-examples --category DemoRunner

# Run all doc-tests
cargo test --doc --locked --workspace --features "simd,standalone,zstd"

# List every registered plugin
cargo xtask known-packages
```

## 4. SC-003 measurement (the 6-click proxy)

The portfolio's "discoverability" success criterion (SC-003) is measured by a
6-click proxy test:

1. Land on the repo's top-level `README.md`.
2. Click the "Examples" link.
3. Click into a category.
4. Click into an example.
5. Click the example's "What to learn from this example" section.
6. Click the JUCE source link to verify the port is faithful.

If a newcomer can complete the path in ≤6 clicks, SC-003 passes for that
example. The proxy is run manually for each new example during PR review;
an automated check is deferred to a follow-up issue.

## 5. What the examples are NOT

- **Not a tutorial.** Each example is a faithful port; if you want a tutorial,
  read the workspace's `QUICK_START.md` and `AGENTS.md` first.
- **Not a benchmark.** The `criterion` benchmarks live alongside the sub-crates
  (`logic_nih_plug_dsp/benches/`, etc.), not in the examples.
- **Not a substitute for the JUCE docs.** The example shows *how* the Rust port
  exposes the concept; for *what* the concept is, follow the link back to the
  JUCE source.
- **Not exhaustive of the underlying API.** The example shows the smallest
  end-to-end usage; the sub-crate's public API may offer more.
