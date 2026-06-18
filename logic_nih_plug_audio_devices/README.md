# logic_nih_plug_audio_devices

Audio device manager + I/O device abstraction ported from
[JUCE's `juce_audio_devices` module](https://docs.juce.com/master/juce_audio_devices_README.html)
for the `logic_nih_plug` ecosystem.

This crate mirrors the split JUCE itself uses: a thin *abstraction* (the
[`AudioIODevice`](src/io_device.rs) trait + the [`AudioIODeviceType`](src/device_type.rs) enum)
and an *orchestrator* ([`AudioDeviceManager`](src/device_manager.rs)) that holds the current
device, watches for device-list changes, and forwards change events to
listeners. Concrete driver integrations (`cpal`, `coreaudio-rs`, `asio-sys`,
…) plug in by implementing [`AudioIODevice`](src/io_device.rs); they are
intentionally **not** bundled here so that this crate compiles on every
platform without pulling in platform SDKs.

## What's included

| Module                                | Purpose                                                                       |
|---------------------------------------|-------------------------------------------------------------------------------|
| [`device_setup`](src/device_setup.rs) | `AudioDeviceSetup` — desired sample rate, buffer size, channel counts        |
| [`device_type`](src/device_type.rs)   | `AudioIODeviceType` enum (CoreAudio, ASIO, WASAPI, ALSA, JACK, …) + compile-time `DriverType::current()` |
| [`io_device`](src/io_device.rs)       | `AudioIODevice` trait + `AudioDeviceInfo` probe result                         |
| [`io_callback`](src/io_callback.rs)   | `AudioIODeviceCallback` trait — the audio-thread callback contract            |
| [`device_manager`](src/device_manager.rs) | `AudioDeviceManager` + `AudioDeviceManagerListener` for change events    |
| [`error`](src/error.rs)               | The unified error enum (`AudioDevicesError`)                                  |

## Feature flags

| Flag      | Default | What it adds                                                                                  |
|-----------|---------|-----------------------------------------------------------------------------------------------|
| `manager` | ✅      | `AudioDeviceManager`, `AudioIODevice`, `AudioIODeviceType`, `AudioDeviceSetup`, callbacks     |
| `full`    | —       | Equivalent to the default set                                                                |

```toml
[dependencies]
logic_nih_plug_audio_devices = { version = "0", default-features = false, features = ["manager"] }
```

## Quick start

```rust
use logic_nih_plug_audio_devices::{
    AudioDeviceManager, AudioDeviceSetup, AudioIODevice, AudioDeviceInfo,
};

let setup = AudioDeviceSetup::stereo_48000(512);

let mut mgr = AudioDeviceManager::new();
mgr.set_audio_device_setup(setup);
assert_eq!(mgr.get_audio_device_setup().sample_rate, 48_000);
```

A real driver integration looks like this (sketch):

```rust,ignore
use logic_nih_plug_audio_devices::{AudioIODevice, AudioDeviceInfo, AudioIODeviceCallback};
use cpal::{Stream, StreamConfig};

struct CpalAudioIODevice {
    name: String,
    info: AudioDeviceInfo,
    stream: Option<Stream>,
}

impl AudioIODevice for CpalAudioIODevice { /* forward calls to `stream` */ }

let device = Box::new(CpalAudioIODevice { /* … */ }) as Box<dyn AudioIODevice>;
let mut mgr = AudioDeviceManager::new();
mgr.set_current_audio_device(Some(device));
mgr.set_audio_device_setup(AudioDeviceSetup::stereo_48000(512));
```

## Relationship to `logic_nih_plug::wrapper::standalone`

The standalone wrapper at `src/wrapper/standalone/backend/cpal.rs` is a
**driver integration** — it owns the cpal / midir streams directly. This
crate is a **device abstraction** that the standalone backend can be
refactored to sit on top of, in the same way JUCE itself uses
`juce_audio_devices` underneath `juce_audio_plugin_client`. The port in this
crate does not touch the standalone wrapper, so the existing standalone
behavior is preserved.

## JUCE parity notes

- `AudioDeviceSetup` mirrors `juce::AudioDeviceManager::AudioDeviceSetup`
  field-for-field (`outputDeviceName`, `inputDeviceName`, `sampleRate`,
  `bufferSize`, `inputChannels`, `outputChannels`, `useDefaultInputDevice`,
  `useDefaultOutputDevice`).
- `AudioIODeviceType` mirrors `juce::AudioIODeviceType` — one enum entry per
  JUCE driver plus `Dummy` for the null backend.
- `AudioIODevice` trait mirrors `juce::AudioIODevice` — same method set
  (start/stop/open/close, latency queries, sample-rate / buffer-size queries,
  optional control panel).
- `AudioDeviceManager` mirrors `juce::AudioDeviceManager` — same listener
  events (`changeListenerCallback` triggered on device type / setup change).
- Concrete driver bindings (CoreAudio, ASIO, WASAPI, ALSA, JACK, …) are
  **not** in scope for this port — they require platform SDKs and are
  exposed via `cpal` or direct FFI in the wider ecosystem.