---
category: Audio
juce_source: examples/Audio/AudioWorkgroupDemo.h
---

# `audio_workgroup_demo` — Two-Node Audio Workgroup via SharedBuffer

## What this example ports

- **JUCE source file**: `examples/Audio/AudioWorkgroupDemo.h`
- **What to learn from this example**: how two `AudioIODeviceCallback`
  nodes can share a single audio buffer — the dataflow pattern that
  JUCE's `AudioWorkgroup` is built around.

## How it works

1. Creates a `SharedAudioBuffer` (`Arc<Mutex<Vec<f32>>>`) — the Rust
   equivalent of JUCE's shared `AudioBuffer<float>` inside an
   `AudioWorkgroup`.
2. `WorkgroupNodeA` writes a 1 kHz sine into the buffer on each callback.
3. `WorkgroupNodeB` reads the buffer and tracks peak amplitude.
4. Both nodes run against their own `MockAudioIODevice`, so both
   lifecycle logs can be asserted.

## Running

```bash
cargo run -p audio_workgroup_demo
```

## Running the tests

```bash
cargo test -p audio_workgroup_demo
```

## Exit codes

| Code | Meaning |
|------|---------|
| 0    | Success — both nodes completed the lifecycle |
| 3    | Workgroup lifecycle incomplete |

## References

- [`logic_nih_plug_audio_devices`](../../../logic_nih_plug_audio_devices/src/lib.rs) — `AudioIODeviceCallback`, `MockAudioIODevice`
- [`specs/001-juce-examples/spec.md`](../../../../specs/001-juce-examples/spec.md) — feature spec
