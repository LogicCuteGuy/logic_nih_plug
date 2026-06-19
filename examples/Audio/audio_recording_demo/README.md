---
category: Audio
juce_source: examples/Audio/AudioRecordingDemo.h
---

# `audio_recording_demo` — Standalone Audio Recording via MockAudioIODevice

## What this example ports

- **JUCE source file**: `examples/Audio/AudioRecordingDemo.h`
- **What to learn from this example**: how to capture audio from a
  `MockAudioIODevice` and write it to a WAV file using
  `logic_nih_plug_audio_formats::wav::WavWriter`.

## How it works

1. Creates a `MockAudioIODevice` with one input channel.
2. Generates a 440 Hz sine wave as a synthetic audio source.
3. Feeds the sine through an `AudioIODeviceCallback` to capture samples.
4. Writes the captured audio to a WAV file.

## Running

```bash
cargo run -p audio_recording_demo -- output.wav
```

## Running the tests

```bash
cargo test -p audio_recording_demo
```

## Exit codes

| Code | Meaning |
|------|---------|
| 0    | Success |
| 2    | Missing or invalid CLI arguments |
| 3    | Audio device or file I/O error |

## References

- [`logic_nih_plug_audio_devices`](../../../logic_nih_plug_audio_devices/src/lib.rs) — audio device manager
- [`logic_nih_plug_audio_formats::wav`](../../../logic_nih_plug_audio_formats/src/wav.rs) — WAV reader/writer
- [`specs/001-juce-examples/spec.md`](../../../../specs/001-juce-examples/spec.md) — feature spec
