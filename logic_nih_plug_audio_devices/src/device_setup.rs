//! The `AudioDeviceSetup` struct — the desired configuration passed between
//! the host, the [`AudioDeviceManager`](crate::AudioDeviceManager), and the
//! underlying [`AudioIODevice`](crate::AudioIODevice).
//!
//! Mirrors `juce::AudioDeviceManager::AudioDeviceSetup` field-for-field.

use crate::error::{AudioDevicesError, AudioDevicesResult};

/// The configuration the host wants the device manager to use.
///
/// Field names match JUCE's `AudioDeviceManager::AudioDeviceSetup` so the
/// values can be persisted via `serde` (when the `serde` feature is enabled
/// on the consumer side) and reloaded across sessions without any mapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioDeviceSetup {
    /// The human-readable name of the desired output device, or `None` to
    /// use the system default.
    pub output_device_name: Option<String>,
    /// The human-readable name of the desired input device, or `None` to
    /// use the system default.
    pub input_device_name: Option<String>,
    /// The desired sample rate in Hz.
    pub sample_rate: u32,
    /// The desired buffer size in samples. Some drivers round this to a
    /// nearby supported value; check `device.get_current_buffer_size()`
    /// after `set_audio_device_setup()` to read back what actually got
    /// applied.
    pub buffer_size: u32,
    /// The number of input channels to open.
    pub input_channels: usize,
    /// The number of output channels to open.
    pub output_channels: usize,
    /// Whether to fall back to the OS default input device when
    /// `input_device_name` doesn't match anything that's currently plugged
    /// in.
    pub use_default_input_device: bool,
    /// Whether to fall back to the OS default output device when
    /// `output_device_name` doesn't match anything that's currently plugged
    /// in.
    pub use_default_output_device: bool,
}

impl Default for AudioDeviceSetup {
    fn default() -> Self {
        Self {
            output_device_name: None,
            input_device_name: None,
            sample_rate: 44_100,
            buffer_size: 512,
            input_channels: 0,
            output_channels: 2,
            use_default_input_device: true,
            use_default_output_device: true,
        }
    }
}

impl AudioDeviceSetup {
    /// A canonical 2-in/2-out setup at 44.1 kHz / 512 frames — the closest
    /// thing JUCE has to a default.
    pub fn stereo_44100(buffer_size: u32) -> Self {
        Self {
            output_device_name: None,
            input_device_name: None,
            sample_rate: 44_100,
            buffer_size,
            input_channels: 2,
            output_channels: 2,
            use_default_input_device: true,
            use_default_output_device: true,
        }
    }

    /// A canonical 2-in/2-out setup at 48 kHz / 512 frames.
    pub fn stereo_48000(buffer_size: u32) -> Self {
        Self {
            output_device_name: None,
            input_device_name: None,
            sample_rate: 48_000,
            buffer_size,
            input_channels: 2,
            output_channels: 2,
            use_default_input_device: true,
            use_default_output_device: true,
        }
    }

    /// True if the setup uses the system default for both the input and the
    /// output side.
    pub fn uses_default_devices(&self) -> bool {
        self.use_default_input_device && self.use_default_output_device
    }

    /// `true` if `output_channels == 0` (no playback).
    pub fn is_output_disabled(&self) -> bool {
        self.output_channels == 0
    }

    /// `true` if `input_channels == 0` (no capture).
    pub fn is_input_disabled(&self) -> bool {
        self.input_channels == 0
    }

    /// Total channels (`input_channels + output_channels`).
    pub fn total_channels(&self) -> usize {
        self.input_channels + self.output_channels
    }

    /// The effective audio-thread callback frequency in Hz
    /// (`sample_rate / buffer_size`).
    pub fn callback_frequency_hz(&self) -> f64 {
        if self.buffer_size == 0 {
            0.0
        } else {
            self.sample_rate as f64 / self.buffer_size as f64
        }
    }

    /// Validate this setup. Returns the first field that fails.
    pub fn validate(&self) -> AudioDevicesResult<()> {
        if self.sample_rate == 0 {
            return Err(AudioDevicesError::InvalidSampleRate(self.sample_rate));
        }
        if self.buffer_size == 0 {
            return Err(AudioDevicesError::InvalidBufferSize(self.buffer_size));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_stereo_44_1k_512() {
        let s = AudioDeviceSetup::default();
        assert_eq!(s.sample_rate, 44_100);
        assert_eq!(s.buffer_size, 512);
        assert_eq!(s.input_channels, 0);
        assert_eq!(s.output_channels, 2);
        assert!(s.use_default_input_device);
        assert!(s.use_default_output_device);
        assert!(s.output_device_name.is_none());
        assert!(s.input_device_name.is_none());
    }

    #[test]
    fn stereo_44100_helper() {
        let s = AudioDeviceSetup::stereo_44100(256);
        assert_eq!(s.sample_rate, 44_100);
        assert_eq!(s.buffer_size, 256);
        assert_eq!(s.input_channels, 2);
        assert_eq!(s.output_channels, 2);
    }

    #[test]
    fn stereo_48000_helper() {
        let s = AudioDeviceSetup::stereo_48000(1024);
        assert_eq!(s.sample_rate, 48_000);
        assert_eq!(s.buffer_size, 1024);
        assert_eq!(s.input_channels, 2);
        assert_eq!(s.output_channels, 2);
    }

    #[test]
    fn helpers_use_default_devices() {
        assert!(AudioDeviceSetup::stereo_44100(512).uses_default_devices());
        assert!(AudioDeviceSetup::stereo_48000(512).uses_default_devices());
        assert!(AudioDeviceSetup::default().uses_default_devices());
    }

    #[test]
    fn is_input_output_disabled() {
        let s = AudioDeviceSetup::default();
        assert!(!s.is_output_disabled());
        assert!(s.is_input_disabled());
        let s2 = AudioDeviceSetup {
            output_channels: 0,
            ..AudioDeviceSetup::default()
        };
        assert!(s2.is_output_disabled());
    }

    #[test]
    fn total_channels() {
        let s = AudioDeviceSetup::default();
        assert_eq!(s.total_channels(), 2);
        let s2 = AudioDeviceSetup {
            input_channels: 4,
            ..AudioDeviceSetup::default()
        };
        assert_eq!(s2.total_channels(), 6);
    }

    #[test]
    fn callback_frequency_hz() {
        let s = AudioDeviceSetup::stereo_48000(480);
        assert!((s.callback_frequency_hz() - 100.0).abs() < 1e-9);
        let s = AudioDeviceSetup::stereo_44100(441);
        assert!((s.callback_frequency_hz() - 100.0).abs() < 1e-9);
    }

    #[test]
    fn validate_accepts_default() {
        assert!(AudioDeviceSetup::default().validate().is_ok());
    }

    #[test]
    fn validate_rejects_zero_sample_rate() {
        let s = AudioDeviceSetup {
            sample_rate: 0,
            ..AudioDeviceSetup::default()
        };
        assert_eq!(
            s.validate(),
            Err(AudioDevicesError::InvalidSampleRate(0))
        );
    }

    #[test]
    fn validate_rejects_zero_buffer_size() {
        let s = AudioDeviceSetup {
            buffer_size: 0,
            ..AudioDeviceSetup::default()
        };
        assert_eq!(
            s.validate(),
            Err(AudioDevicesError::InvalidBufferSize(0))
        );
    }

    #[test]
    fn field_round_trip_after_clone() {
        let s = AudioDeviceSetup {
            output_device_name: Some("External DAC".to_string()),
            input_device_name: None,
            sample_rate: 96_000,
            buffer_size: 128,
            input_channels: 1,
            output_channels: 8,
            use_default_input_device: false,
            use_default_output_device: false,
        };
        let cloned = s.clone();
        assert_eq!(s, cloned);
        assert_eq!(cloned.output_device_name.as_deref(), Some("External DAC"));
        assert_eq!(cloned.sample_rate, 96_000);
        assert_eq!(cloned.buffer_size, 128);
        assert_eq!(cloned.input_channels, 1);
        assert_eq!(cloned.output_channels, 8);
        assert!(!cloned.uses_default_devices());
        assert!(!cloned.is_input_disabled());
        assert!(!cloned.is_output_disabled());
    }

    #[test]
    fn callback_frequency_is_zero_when_buffer_size_is_zero() {
        let s = AudioDeviceSetup {
            buffer_size: 0,
            ..AudioDeviceSetup::default()
        };
        // Doesn't validate, but the math helper should still not panic.
        assert_eq!(s.callback_frequency_hz(), 0.0);
    }
}