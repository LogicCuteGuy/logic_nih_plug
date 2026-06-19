---
category: Plugins
---

# `examples/` — Workspace Standalone App Examples

This directory is the workspace's **standalone app examples** gallery. Every
crate here is a runnable binary (`[[bin]]` target) that demonstrates a
non-plugin workflow. Plugin examples live in
[`plugins/examples/`](../plugins/examples/README.md).

## What's here

- **`Audio/`** — standalone audio apps: file playback, recording, MIDI
  playback, FFT, synthesis. These are the ports of `JUCE/examples/Audio/`.
- **`Utilities/`** — cross-cutting utilities: file-IO CLI demos (WAV
  reader/writer, SMF inspector), OSC sender/receiver.
- **`Plugins/`** — the **plugin host** example. One crate for the headless CLI
  (`plugin_host_cli`) and one for the `egui` GUI host
  (`juce_audio_plugin_host_egui`).
- **`DemoRunner/`** — the **showcase app**. One crate
  (`juce_demorunner`) with feature-flag-selected backends (`gui-egui` default,
  `gui-iced`, `gui-vizia`).

## Audio (mirrors `JUCE/examples/Audio/`)

Crate | Mirrors JUCE example | What it demonstrates
---|---|---
[`audio_playback_demo`](./Audio/audio_playback_demo) | `examples/Audio/AudioPlaybackDemo.h` | Plays a WAV file through `MockAudioIODevice` (CI) or real output
[`audio_recording_demo`](./Audio/audio_recording_demo) | `examples/Audio/AudioRecordingDemo.h` | Records input to a WAV file
[`audio_workgroup_demo`](./Audio/audio_workgroup_demo) | `examples/Audio/AudioWorkgroupDemo.h` | Two-node `AudioWorkgroup` sharing a buffer

## Utilities (mirrors `JUCE/examples/Utilities/`)

Crate | Mirrors JUCE example | What it demonstrates
---|---|---
[`wav_reader`](./Utilities/wav_reader) | — | Prints a WAV file's header summary
[`wav_writer`](./Utilities/wav_writer) | — | Writes a 1-second sine WAV and round-trips it
[`midi_file_inspector`](./Utilities/midi_file_inspector) | — | Prints an SMF file's tempo, tracks, events
[`osc_sender_demo`](./Utilities/osc_sender_demo) | `examples/Utilities/OSCDemo.h` | Sends OSC bundles
[`osc_receiver_demo`](./Utilities/osc_receiver_demo) | `examples/Utilities/OSCDemo.h` | Receives OSC and prints messages

## Plugins (mirrors `JUCE/examples/Plugins/`)

Crate | Mirrors JUCE example | What it demonstrates
---|---|---
[`plugin_host_cli`](./Plugins/plugin_host_cli) | `examples/Plugins/HostPluginDemo.h` | Headless CLI: scans a dir, prints discovered plugins
[`juce_audio_plugin_host_egui`](./Plugins/juce_audio_plugin_host_egui) | `examples/Plugins/HostPluginDemo.h` | `egui` GUI host: scan + load + param slider + state save/load

## DemoRunner (mirrors `JUCE/examples/DemoRunner/`)

Crate | Mirrors JUCE example | What it demonstrates
---|---|---
[`juce_demorunner`](./DemoRunner/juce_demorunner) | `examples/DemoRunner/Source/*` | Showcase of every public GUI component with 3 backends

## Shared assets

The `audio-assets/` and `midi-assets/` directories at the top level hold
small reference fixtures used by the doc-tests in these crates:

- `audio-assets/sine_1khz_1s.wav` — 1 kHz mono 16-bit 44.1 kHz 1-second sine
- `midi-assets/single_note.mid` — SMF format-1 with one C4 note

## Running an example

```bash
# Standalone app
cargo run -p audio_playback_demo -- audio-assets/sine_1khz_1s.wav

# Plugin host CLI
cargo run -p plugin_host_cli -- ./test-vst3/

# DemoRunner with the default (egui) backend
cargo run -p juce_demorunner
# With iced
cargo run -p juce_demorunner --no-default-features --features gui-iced
# With vizia
cargo run -p juce_demorunner --no-default-features --features gui-vizia
```

## Running the doc-tests

```bash
cargo test --doc -p <crate name>
# or, for every example in a category:
cargo xtask test-examples --category Audio
cargo xtask test-examples --category Utilities
cargo xtask test-examples --category Plugins
cargo xtask test-examples --category DemoRunner
```

## Exit codes (standalone examples, FR-006)

| Code | Meaning |
|---|---|
| 0  | Success |
| 2  | Environment error (missing input file, no audio device, etc.) |
| 3  | Malformed input (bad WAV header, truncated SMF, etc.) |
| ≥4 | Implementation-defined; the example still prints a clear error message |

## How to add a new example

1. Pick the JUCE example to port (see
   [`specs/001-juce-examples/example-inventory.md`](../specs/001-juce-examples/example-inventory.md)
   for the current ledger).
2. Copy a minimal existing example crate (`wav_reader` is the simplest CLI).
3. Add a top-level `README.md` matching
   [`specs/001-juce-examples/contracts/example-crate-contract.md`](../specs/001-juce-examples/contracts/example-crate-contract.md).
4. Update the ledger row's `status` to `ported`.
5. Run `cargo xtask test-examples` to confirm doc-tests pass.

## References

- [`specs/001-juce-examples/spec.md`](../specs/001-juce-examples/spec.md) — feature spec
- [`specs/001-juce-examples/plan.md`](../specs/001-juce-examples/plan.md) — implementation plan
- [`specs/001-juce-examples/example-inventory.md`](../specs/001-juce-examples/example-inventory.md) — ledger
- [`specs/001-juce-examples/quickstart.md`](../specs/001-juce-examples/quickstart.md) — newcomer guide
- [`AGENTS.md`](../AGENTS.md) — workspace rules
- [JUCE `examples/`](https://github.com/juce-framework/JUCE/tree/master/examples)
