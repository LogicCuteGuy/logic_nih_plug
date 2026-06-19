---
category: Audio
juce_source: examples/Audio/AudioPlaybackDemo.h
---

# `audio_playback_demo` — Standalone WAV Playback via MockAudioIODevice

## What this example ports

- **JUCE source file**: `examples/Audio/AudioPlaybackDemo.h`
- **What to learn from this example**: how to wire
  `logic_nih_plug_audio_formats::wav::WavReader` to
  `logic_nih_plug_audio_devices::AudioDeviceManager` with a
  `MockAudioIODevice` for CI-friendly, deterministic playback tests.

## How it works

1. Reads a WAV file (or generates a 1 kHz sine if none provided).
2. Creates an `AudioDeviceManager` with a `MockAudioIODevice`.
3. Drives the device lifecycle: `open → start → stop → close`.
4. Verifies the lifecycle transitions via `MockAudioIODevice::event_log()`.

## Running

```bash
cargo run -p audio_playback_demo -- examples/audio-assets/sine_1khz_1s.wav
```

Or without a WAV file (generates a 1 kHz sine):

```bash
cargo run -p audio_playback_demo
```

## Running the tests

```bash
cargo test -p audio_playback_demo
```

## Exit codes

| Code | Meaning |
|------|---------|
| 0    | Success |
| 2    | Missing or invalid CLI arguments |
| 3    | Audio device error |

## References

- [`logic_nih_plug_audio_devices`](../../../logic_nih_plug_audio_devices/src/lib.rs) — audio device manager
- [`logic_nih_plug_audio_formats::wav`](../../../logic_nih_plug_audio_formats/src/wav.rs) — WAV reader/writer
- [`specs/001-juce-examples/spec.md`](../../../../specs/001-juce-examples/spec.md) — feature spec
