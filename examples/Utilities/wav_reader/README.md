---
category: Utilities
juce_source: examples/Utilities/CMDLineDemo.h
---

# `wav_reader` — WAV File Header Summary CLI

## What this example ports

- **JUCE source file**: `examples/Utilities/CMDLineDemo.h` (WAV reader variant)
- **What to learn**: how to use `logic_nih_plug_audio_formats::wav::WavReader`
  to parse a WAV file and print its metadata + peak amplitude.

## How it works

1. Accepts a WAV file path as a CLI argument.
2. Opens it via `WavReader::open`, reads metadata.
3. Reads all samples to compute peak amplitude.
4. Prints a human-readable summary.

## Running

```bash
cargo run -p wav_reader -- examples/audio-assets/sine_1khz_1s.wav
```

## Running the tests

```bash
cargo test -p wav_reader
```

## Exit codes

| Code | Meaning |
|------|---------|
| 0    | Success |
| 2    | Bad args / missing path |
| 3    | File I/O or parse error |

## References

- [`logic_nih_plug_audio_formats::wav`](../../../logic_nih_plug_audio_formats/src/wav.rs) — `WavReader`
- [`specs/001-juce-examples/spec.md`](../../../../specs/001-juce-examples/spec.md) — feature spec