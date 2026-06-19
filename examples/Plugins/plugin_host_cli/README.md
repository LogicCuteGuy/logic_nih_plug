---
category: Plugins
juce_source: examples/Plugins/HostPluginDemo.h
---

# `plugin_host_cli` — Headless Plugin Host CLI

## What this example ports

- **JUCE source file**: `examples/Plugins/HostPluginDemo.h`
- **What to learn from this example**: how to drive
  `logic_nih_plug_audio_processors::PluginDirectoryScanner` from a
  headless binary, without any GUI dependencies. Suitable for CI
  smoke tests and server-side plugin scanning.

## How it works

1. Accepts a directory path as a CLI argument.
2. Creates a [`MockPluginFormat`] (or `NullPluginFormat`) and feeds
   it to a `PluginDirectoryScanner`.
3. The scanner walks the directory, returning one
   `PluginDescription` per candidate file.
4. Each description is added to a `KnownPluginList`, which is
   printed to stdout.

## Running

```bash
cargo run -p plugin_host_cli -- ./test-vst3/
```

## Running the tests

```bash
cargo test -p plugin_host_cli
```

## Exit codes

| Code | Meaning |
|------|---------|
| 0    | Success |
| 2    | Bad args / directory does not exist |
| 3    | Scanner error (sanity check failure) |

## References

- [`logic_nih_plug_audio_processors`](../../../logic_nih_plug_audio_processors/src/lib.rs) — scanner + plugin list
- [`specs/001-juce-examples/spec.md`](../../../../specs/001-juce-examples/spec.md) — feature spec
