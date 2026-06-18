//! Error types for `logic_nih_plug_audio_devices`.

use thiserror::Error;

/// Errors that can occur while configuring or operating an audio device.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AudioDevicesError {
    /// The user-supplied sample rate isn't in the device's supported list.
    #[error("sample rate {requested} Hz is not supported by device '{device}' (supported: {supported:?})")]
    UnsupportedSampleRate {
        /// The device the sample rate was requested for.
        device: String,
        /// The sample rate the user asked for.
        requested: u32,
        /// The list of sample rates the device actually supports.
        supported: Vec<u32>,
    },

    /// The user-supplied buffer size isn't in the device's supported list.
    #[error("buffer size {requested} samples is not supported by device '{device}' (supported: {supported:?})")]
    UnsupportedBufferSize {
        /// The device the buffer size was requested for.
        device: String,
        /// The buffer size the user asked for.
        requested: u32,
        /// The list of buffer sizes the device actually supports.
        supported: Vec<u32>,
    },

    /// The device reported zero input or output channels in the desired setup
    /// but the host asked for a positive number.
    #[error("device '{device}' exposes {available} {direction} channels but {requested} were requested")]
    NotEnoughChannels {
        /// The device that was rejected.
        device: String,
        /// `"input"` or `"output"` — which side didn't have enough channels.
        direction: &'static str,
        /// How many channels the device has.
        available: usize,
        /// How many channels the user asked for.
        requested: usize,
    },

    /// An attempt was made to operate on a device that is not currently open.
    #[error("device '{device}' is not open")]
    DeviceNotOpen {
        /// The device the operation was attempted on.
        device: String,
    },

    /// The device reported an error while opening / starting / stopping.
    #[error("device '{device}' failed: {reason}")]
    DeviceError {
        /// The device that reported the error.
        device: String,
        /// The OS / driver error message.
        reason: String,
    },

    /// `set_audio_device_setup` was called with a sample rate of zero.
    #[error("invalid sample rate {0} (must be > 0)")]
    InvalidSampleRate(u32),

    /// `set_audio_device_setup` was called with a buffer size of zero.
    #[error("invalid buffer size {0} (must be > 0)")]
    InvalidBufferSize(u32),

    /// A channel count in `AudioDeviceSetup` was set to a number larger than
    /// the maximum that `AudioDeviceInfo` advertised for the active device.
    #[error("invalid channel count {requested} (must be in 0..={max})")]
    InvalidChannelCount {
        /// Which side the bad count was on.
        direction: &'static str,
        /// The bad count.
        requested: usize,
        /// The largest value that would have been accepted.
        max: usize,
    },
}

/// Convenience alias used throughout the crate.
pub type AudioDevicesResult<T> = Result<T, AudioDevicesError>;