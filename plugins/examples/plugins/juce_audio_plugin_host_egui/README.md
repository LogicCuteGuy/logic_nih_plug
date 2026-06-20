---
category: Plugins
juce_source: examples/Plugins/HostPluginDemo.h
---

# `juce_audio_plugin_host_egui` — Plugin Host (egui)

## What this example ports

- **JUCE source file**: `examples/Plugins/HostPluginDemo.h`
- **What to learn from this example**: how to wire the framework's
  plugin discovery (`PluginDirectoryScanner` + `KnownPluginList`) to
  a custom `egui` editor with parameter slider binding, and how to
  use `MockAudioIODevice` for CI tests of the audio path.

## Architecture

The crate is split into three modules:

| Module | Purpose |
|---|---|
| [`host`](src/host.rs) | Audio engine — holds the loaded plugin, runs `process()` per buffer |
| [`editor`](src/editor.rs) | Parameter binding + state save/load plumbing |
| [`scanner`](src/scanner.rs) | Wraps `PluginDirectoryScanner` + `KnownPluginList` |

The `egui`-driven UI window itself is feature-gated on the `gui`
feature; this crate ships the **plumbing** so the audio path can be
exercised by the integration test (`cargo test`).

## Running

```bash
cargo run -p juce_audio_plugin_host_egui --features standalone -- ./test-vst3/
```

## Running the tests

```bash
cargo test -p juce_audio_plugin_host_egui
```

## References

- [`logic_nih_plug_audio_processors`](../../../logic_nih_plug_audio_processors/src/lib.rs) — scanner
- [`logic_nih_plug_audio_devices`](../../../logic_nih_plug_audio_devices/src/lib.rs) — `MockAudioIODevice`
- [`specs/001-juce-examples/spec.md`](../../../../specs/001-juce-examples/spec.md) — feature spec

## Parameters

Not applicable — this example is a *plugin host* rather than a plugin with its own parameters; it surfaces parameters belonging to whatever scan-loaded plugin is currently hosted in the `host` engine. The visible host-level UI controls are limited to scan, load, and play/stop rather than audio parameters.

## Building

```bash
cargo build -p juce_audio_plugin_host_egui --features standalone --release
```

The crate is also built as part of the standard workspace bundle step:

```bash
cargo xtask bundle juce_audio_plugin_host_egui --release
```

## Running the doc-tests

```bash
cargo test -p juce_audio_plugin_host_egui --doc
cargo test -p juce_audio_plugin_host_egui
```

The first command runs the doctests embedded in the crate's `lib.rs`; the second runs the integration test suite that exercises the host engine and the `PluginDirectoryScanner` wrapper.

## JUCE fidelity checklist

- **Plugin discovery**: `PluginDirectoryScanner` + `KnownPluginList` are wired through `scanner.rs` exactly as in `examples/Plugins/HostPluginDemo.h`, preserving JUCE's scan/sort/dedupe pipeline.
- **Audio path**: the `host` engine calls `process()` on the loaded plugin per buffer in the same order JUCE's host demo does, including the `MockAudioIODevice` fallback for offline/CI runs.
- **Parameter binding**: `editor.rs` mirrors JUCE's slider-to-APVTS binding pattern via `nih_plug::ParamPtr` so the host UI controls behave like the original `HostPluginDemo` panel.
- **State save/load**: the host preserves JUCE's chunk-based state round-tripping for any loaded plugin instance.
