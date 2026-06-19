//! # logic_nih_plug_audio_devices
//!
//! Audio device manager + I/O device abstraction ported from
//! [JUCE's `juce_audio_devices` module](https://docs.juce.com/master/juce_audio_devices_README.html)
//! for the `logic_nih_plug` ecosystem.
//!
//! ## What's inside
//!
//! - [`AudioDeviceSetup`] — the desired configuration (sample rate, buffer
//!   size, input / output channel counts).
//! - [`AudioIODeviceType`] — enum of every driver backend JUCE supports
//!   (`CoreAudio`, `ASIO`, `WASAPI`, `DirectSound`, `ALSA`, `JACK`, …).
//!   `AudioIODeviceType::current()` returns the one that is actually
//!   available at compile time on the host.
//! - [`AudioIODevice`] — trait every device implements. Concrete drivers
//!   (`cpal`/`coreaudio-rs`/…) plug in by implementing this trait.
//! - [`AudioIODeviceCallback`] — the audio-thread callback trait. The host
//!   forwards raw `*const f32` input buffers / `*mut f32` output buffers
//!   to the consumer's `audio_device_io_callback`.
//! - [`AudioDeviceManager`] — the long-lived orchestrator. Holds the current
//!   device, fires `AudioDeviceManagerListener` events when the device type
//!   or sample-rate / buffer-size changes.
//! - [`AudioDeviceInfo`] — name + sample rates + buffer sizes + channel
//!   counts reported by a device at probe time.
//!
//! ## Feature flags
//!
//! | Flag      | Default | What it gates                                                                        |
//! |-----------|---------|--------------------------------------------------------------------------------------|
//! | `manager` | ✅      | The whole crate — `AudioDeviceManager`, `AudioIODevice`, `AudioIODeviceType`, …      |
//! | `full`    | —       | Equivalent to the default set                                                        |
//!
//! ## Example
//!
//! ```
//! use logic_nih_plug_audio_devices::{
//!     AudioDeviceManager, AudioDeviceSetup, AudioIODevice, AudioIODeviceCallback,
//!     AudioDeviceInfo,
//! };
//!
//! // A 2-in/2-out setup at 48 kHz / 512 frames.
//! let setup = AudioDeviceSetup::stereo_48000(512);
//!
//! let mut mgr = AudioDeviceManager::new();
//! mgr.set_audio_device_setup(setup);
//! assert_eq!(mgr.get_audio_device_setup().sample_rate, 48_000);
//! ```

#![warn(missing_docs)]

mod device_setup;
mod device_type;
mod error;
mod io_callback;
mod io_device;
#[cfg(feature = "manager")]
mod device_manager;
mod mock;

pub use device_setup::AudioDeviceSetup;
pub use device_type::{AudioIODeviceType, DriverType};
pub use error::{AudioDevicesError, AudioDevicesResult};
pub use io_callback::{
    AudioIODeviceCallback, AudioIODeviceCallbackData, NullAudioIODeviceCallback,
};
pub use io_device::{AudioDeviceInfo, AudioIODevice};
pub use mock::{MockAudioIODevice, MockAudioIODeviceEvent};

#[cfg(feature = "manager")]
pub use device_manager::{
    AudioDeviceManager, AudioDeviceManagerListener, AudioDeviceManagerState,
};