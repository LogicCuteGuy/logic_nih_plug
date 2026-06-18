//! The `AudioIODevice` trait + the `AudioDeviceInfo` probe struct.
//!
//! Concrete driver integrations (`cpal`, `coreaudio-rs`, `asio-sys`, …)
//! implement this trait to plug into [`AudioDeviceManager`](crate::AudioDeviceManager).

use crate::error::{AudioDevicesError, AudioDevicesResult};
use crate::io_callback::AudioIODeviceCallback;

/// Everything a driver knows about itself when probed by
/// [`AudioIODevice::get_device_info`](crate::AudioIODevice::get_device_info).
///
/// Mirrors the JUCE class of the same name (structurally — JUCE keeps
/// these as separate methods, but the bundled shape is identical).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioDeviceInfo {
    /// Human-readable name (e.g. `"MacBook Pro Speakers"`, `"External DAC"`).
    pub name: String,
    /// Sample rates the device supports, in Hz.
    pub sample_rates: Vec<u32>,
    /// Buffer sizes the device supports, in samples.
    pub buffer_sizes: Vec<u32>,
    /// Names of the available input channels, in order.
    pub input_channel_names: Vec<String>,
    /// Names of the available output channels, in order.
    pub output_channel_names: Vec<String>,
    /// The device's preferred input latency, in samples.
    pub input_latency_samples: u32,
    /// The device's preferred output latency, in samples.
    pub output_latency_samples: u32,
}

impl AudioDeviceInfo {
    /// Number of input channels the device exposes.
    pub fn num_input_channels(&self) -> usize {
        self.input_channel_names.len()
    }

    /// Number of output channels the device exposes.
    pub fn num_output_channels(&self) -> usize {
        self.output_channel_names.len()
    }

    /// Total channels (`input + output`).
    pub fn total_channels(&self) -> usize {
        self.num_input_channels() + self.num_output_channels()
    }

    /// `true` if the device has at least one input channel.
    pub fn has_input(&self) -> bool {
        !self.input_channel_names.is_empty()
    }

    /// `true` if the device has at least one output channel.
    pub fn has_output(&self) -> bool {
        !self.output_channel_names.is_empty()
    }

    /// Pick the closest supported sample rate to a requested one.
    pub fn closest_sample_rate(&self, requested: u32) -> Option<u32> {
        self.sample_rates
            .iter()
            .copied()
            .min_by_key(|&r| r.max(requested) - r.min(requested))
    }

    /// Pick the closest supported buffer size to a requested one.
    pub fn closest_buffer_size(&self, requested: u32) -> Option<u32> {
        self.buffer_sizes
            .iter()
            .copied()
            .min_by_key(|&s| s.max(requested) - s.min(requested))
    }

    /// Verify that a desired sample rate is supported.
    pub fn validate_sample_rate(&self, requested: u32) -> AudioDevicesResult<()> {
        if self.sample_rates.contains(&requested) {
            Ok(())
        } else {
            Err(AudioDevicesError::UnsupportedSampleRate {
                device: self.name.clone(),
                requested,
                supported: self.sample_rates.clone(),
            })
        }
    }

    /// Verify that a desired buffer size is supported.
    pub fn validate_buffer_size(&self, requested: u32) -> AudioDevicesResult<()> {
        if self.buffer_sizes.contains(&requested) {
            Ok(())
        } else {
            Err(AudioDevicesError::UnsupportedBufferSize {
                device: self.name.clone(),
                requested,
                supported: self.buffer_sizes.clone(),
            })
        }
    }
}

/// The interface every audio device driver implements.
///
/// The trait is **object-safe** so it can be stored as
/// `Box<dyn AudioIODevice>` inside
/// [`AudioDeviceManager`](crate::AudioDeviceManager). All methods are
/// `&mut self` because drivers mutate internal state (open/close, start/
/// stop, sample-rate negotiation).
///
/// Mirrors `juce::AudioIODevice`.
pub trait AudioIODevice: Send {
    /// The device's user-facing name.
    fn get_name(&self) -> &str;

    /// Everything the driver knows about itself.
    fn get_device_info(&self) -> AudioDeviceInfo;

    /// Names of the device's output channels, in order.
    fn get_output_channel_names(&self) -> Vec<String> {
        self.get_device_info().output_channel_names
    }

    /// Names of the device's input channels, in order.
    fn get_input_channel_names(&self) -> Vec<String> {
        self.get_device_info().input_channel_names
    }

    /// Default buffer size in samples.
    fn get_default_buffer_size(&self) -> u32 {
        self.get_device_info()
            .buffer_sizes
            .first()
            .copied()
            .unwrap_or(512)
    }

    /// Buffer size the device is currently using.
    fn get_current_buffer_size(&self) -> u32;

    /// Sample rate the device is currently using.
    fn get_current_sample_rate(&self) -> f64;

    /// Input latency in samples.
    fn get_input_latency_in_samples(&self) -> u32 {
        self.get_device_info().input_latency_samples
    }

    /// Output latency in samples.
    fn get_output_latency_in_samples(&self) -> u32 {
        self.get_device_info().output_latency_samples
    }

    /// Open the device. Returns `Ok(())` if successful. Some drivers keep
    /// the device open between `start`/`stop` cycles — `open` is the
    /// "claim exclusive access" step.
    fn open(&mut self, sample_rate: f64, buffer_size: u32) -> AudioDevicesResult<()>;

    /// Release exclusive access.
    fn close(&mut self);

    /// `true` if the device is currently open.
    fn is_open(&self) -> bool;

    /// Begin calling `callback`. `sample_rate` and `buffer_size` are the
    /// values the driver should configure itself to use.
    fn start(&mut self, callback: Box<dyn AudioIODeviceCallback>);

    /// Stop calling `callback`. The device remains open.
    fn stop(&mut self);

    /// `true` if the device is currently streaming.
    fn is_playing(&self) -> bool;

    /// The last error message reported by the driver, if any.
    fn get_last_error(&self) -> Option<String>;

    /// Optional: `true` if the driver exposes a control panel for buffer
    /// size / sample rate / device routing. Returns `false` by default.
    fn has_control_panel(&self) -> bool {
        false
    }

    /// Optional: show the driver's control panel. The default is a no-op.
    /// Implementations may return an error string if the panel failed to
    /// open.
    fn show_control_panel(&mut self) -> AudioDevicesResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_info() -> AudioDeviceInfo {
        AudioDeviceInfo {
            name: "Test Device".to_string(),
            sample_rates: vec![44_100, 48_000, 96_000],
            buffer_sizes: vec![128, 256, 512, 1024],
            input_channel_names: vec!["In 1".to_string(), "In 2".to_string()],
            output_channel_names: vec!["Out 1".to_string(), "Out 2".to_string()],
            input_latency_samples: 128,
            output_latency_samples: 256,
        }
    }

    #[test]
    fn info_channel_counts() {
        let info = make_info();
        assert_eq!(info.num_input_channels(), 2);
        assert_eq!(info.num_output_channels(), 2);
        assert_eq!(info.total_channels(), 4);
        assert!(info.has_input());
        assert!(info.has_output());
    }

    #[test]
    fn info_no_channels() {
        let mut info = make_info();
        info.input_channel_names.clear();
        info.output_channel_names.clear();
        assert!(!info.has_input());
        assert!(!info.has_output());
        assert_eq!(info.total_channels(), 0);
    }

    #[test]
    fn closest_sample_rate_picks_min_distance() {
        let info = make_info();
        assert_eq!(info.closest_sample_rate(47_000), Some(48_000));
        assert_eq!(info.closest_sample_rate(50_000), Some(48_000));
        assert_eq!(info.closest_sample_rate(96_000), Some(96_000));
        assert_eq!(info.closest_sample_rate(1_000_000), Some(96_000));
    }

    #[test]
    fn closest_buffer_size_picks_min_distance() {
        let info = make_info();
        assert_eq!(info.closest_buffer_size(300), Some(256));
        assert_eq!(info.closest_buffer_size(700), Some(512));
        assert_eq!(info.closest_buffer_size(1024), Some(1024));
    }

    #[test]
    fn closest_returns_none_when_lists_are_empty() {
        let mut info = make_info();
        info.sample_rates.clear();
        info.buffer_sizes.clear();
        assert_eq!(info.closest_sample_rate(48_000), None);
        assert_eq!(info.closest_buffer_size(512), None);
    }

    #[test]
    fn validate_sample_rate_accepts_known() {
        let info = make_info();
        assert!(info.validate_sample_rate(44_100).is_ok());
        assert!(info.validate_sample_rate(48_000).is_ok());
        assert!(info.validate_sample_rate(96_000).is_ok());
    }

    #[test]
    fn validate_sample_rate_rejects_unknown() {
        let info = make_info();
        let err = info.validate_sample_rate(192_000).unwrap_err();
        assert_eq!(
            err,
            AudioDevicesError::UnsupportedSampleRate {
                device: "Test Device".to_string(),
                requested: 192_000,
                supported: vec![44_100, 48_000, 96_000],
            }
        );
    }

    #[test]
    fn validate_buffer_size_accepts_known() {
        let info = make_info();
        assert!(info.validate_buffer_size(128).is_ok());
        assert!(info.validate_buffer_size(512).is_ok());
    }

    #[test]
    fn validate_buffer_size_rejects_unknown() {
        let info = make_info();
        let err = info.validate_buffer_size(2048).unwrap_err();
        assert_eq!(
            err,
            AudioDevicesError::UnsupportedBufferSize {
                device: "Test Device".to_string(),
                requested: 2048,
                supported: vec![128, 256, 512, 1024],
            }
        );
    }

    #[test]
    fn default_buffer_size_falls_back_to_512() {
        let mut info = make_info();
        info.buffer_sizes.clear();
        let device = CountingDevice { info };
        assert_eq!(device.get_default_buffer_size(), 512);
    }

    /// A bare-bones `AudioIODevice` implementation used only to verify the
    /// trait compiles and the default helpers (`has_control_panel`,
    /// `show_control_panel`) work.
    struct CountingDevice {
        info: AudioDeviceInfo,
    }

    impl AudioIODevice for CountingDevice {
        fn get_name(&self) -> &str {
            &self.info.name
        }
        fn get_device_info(&self) -> AudioDeviceInfo {
            self.info.clone()
        }
        fn get_current_buffer_size(&self) -> u32 {
            self.info.buffer_sizes[0]
        }
        fn get_current_sample_rate(&self) -> f64 {
            self.info.sample_rates[0] as f64
        }
        fn open(&mut self, _: f64, _: u32) -> AudioDevicesResult<()> {
            Ok(())
        }
        fn close(&mut self) {}
        fn is_open(&self) -> bool {
            false
        }
        fn start(&mut self, _: Box<dyn AudioIODeviceCallback>) {}
        fn stop(&mut self) {}
        fn is_playing(&self) -> bool {
            false
        }
        fn get_last_error(&self) -> Option<String> {
            None
        }
    }

    #[test]
    fn trait_default_helpers() {
        let device = CountingDevice { info: make_info() };
        // `has_control_panel` defaults to `false`, `show_control_panel` defaults to Ok.
        assert!(!device.has_control_panel());
        let mut device = device;
        assert!(device.show_control_panel().is_ok());
    }

    #[test]
    fn info_default_buffer_size_helper() {
        let info = make_info();
        assert_eq!(info.closest_buffer_size(512), Some(512));
    }
}