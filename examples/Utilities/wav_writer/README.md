---
category: Utilities
juce_source: examples/Utilities/CMDLineDemo.h
---

# `wav_writer` — WAV File Writer CLI (1s 440 Hz Sine)

## What this example ports

- **JUCE source file**: `examples/Utilities/CMDLineDemo.h` (WAV writer variant)
- **What to learn**: how to use `logic_nih_plug_audio_formats::wav::WavWriter`
  to write a multi-channel WAV file from raw f32 samples and verify the
  output round-trips through `WavReader`.

## How it works

1. Accepts an output path as a CLI argument.
2. Generates a 1-second 440 Hz sine wave.
3. Writes it as a 16-bit mono WAV at 44.1 kHz.
4. Reads it back and asserts the sample rate / channel count / frame count match.

## Running

```bash
cargo run -p wav_writer -- /tmp/output.wav
```

## Running the tests

```bash
cargo test -p wav_writer
```

## Exit codes

| Code | Meaning |
|------|---------|
| 0    | Success |
| 2    | Bad args / missing path |
| 3    | Write or round-trip error |

## References

- [`logic_nih_plug_audio_formats::wav`](../../../logic_nih_plug_audio_formats/src/wav.rs) — `WavWriter` + `WavReader`
- [`specs/001-juce-examples/spec.md`](../../../../specs/001-juce-examples/spec.md) — feature spec