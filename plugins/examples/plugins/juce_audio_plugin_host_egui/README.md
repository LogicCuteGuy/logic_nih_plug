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
