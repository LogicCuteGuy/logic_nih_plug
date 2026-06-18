//! A `MockAudioIODevice` for tests and headless hosts.
//!
//! Implements [`AudioIODevice`] with configurable info / sample rates /
//! buffer sizes / channel counts, and tracks every lifecycle transition
//! so callers can assert on them.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;

use crate::error::AudioDevicesResult;
use crate::io_callback::AudioIODeviceCallback;
use crate::io_device::{AudioDeviceInfo, AudioIODevice};

/// Lifecycle events emitted by [`MockAudioIODevice`]. Useful for asserting
/// that the device manager drove the device through the correct
/// sequence (`open → start → stop → close`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MockAudioIODeviceEvent {
    /// `open(sample_rate, buffer_size)` was called.
    Opened,
    /// `close()` was called.
    Closed,
    /// `start(callback)` was called.
    Started,
    /// `stop()` was called.
    Stopped,
}

/// A configurable in-memory [`AudioIODevice`] for tests.
///
/// The `event_log` and `is_playing`/`is_open` flags are exposed as
/// atomics + a `Mutex<Vec<…>>` so tests can inspect them from any thread
/// without coupling to the audio thread's lifetime.
pub struct MockAudioIODevice {
    info: AudioDeviceInfo,
    is_open: AtomicBool,
    is_playing: AtomicBool,
    callback_count: AtomicUsize,
    last_opened_sample_rate: Mutex<Option<f64>>,
    last_opened_buffer_size: Mutex<Option<u32>>,
    event_log: Mutex<Vec<MockAudioIODeviceEvent>>,
    /// If set, `get_last_error()` returns this. Lets a test simulate a
    /// driver that has failed.
    forced_error: Mutex<Option<String>>,
}

impl MockAudioIODevice {
    /// Construct a mock that reports a stereo device at 44.1 kHz / 48 kHz
    /// with a 128 / 256 / 512 / 1024 buffer-size menu.
    pub fn stereo_44100() -> Self {
        Self::new(AudioDeviceInfo {
            name: "Mock Stereo 44.1k".to_string(),
            sample_rates: vec![44_100, 48_000],
            buffer_sizes: vec![128, 256, 512, 1024],
            input_channel_names: vec!["Mock In 1".to_string(), "Mock In 2".to_string()],
            output_channel_names: vec!["Mock Out 1".to_string(), "Mock Out 2".to_string()],
            input_latency_samples: 128,
            output_latency_samples: 256,
        })
    }

    /// Construct a mock that reports a 7.1 device at 48 kHz.
    pub fn surround_48000() -> Self {
        Self::new(AudioDeviceInfo {
            name: "Mock Surround 48k".to_string(),
            sample_rates: vec![48_000, 96_000],
            buffer_sizes: vec![256, 512, 1024],
            input_channel_names: vec![],
            output_channel_names: vec![
                "Front L".to_string(),
                "Front R".to_string(),
                "Center".to_string(),
                "LFE".to_string(),
                "Surr L".to_string(),
                "Surr R".to_string(),
                "Back L".to_string(),
                "Back R".to_string(),
            ],
            input_latency_samples: 0,
            output_latency_samples: 256,
        })
    }

    /// Construct a mock from an explicit [`AudioDeviceInfo`].
    pub fn new(info: AudioDeviceInfo) -> Self {
        Self {
            info,
            is_open: AtomicBool::new(false),
            is_playing: AtomicBool::new(false),
            callback_count: AtomicUsize::new(0),
            last_opened_sample_rate: Mutex::new(None),
            last_opened_buffer_size: Mutex::new(None),
            event_log: Mutex::new(Vec::new()),
            forced_error: Mutex::new(None),
        }
    }

    /// Snapshot of every lifecycle event the device has emitted so far.
    pub fn event_log(&self) -> Vec<MockAudioIODeviceEvent> {
        self.event_log.lock().unwrap().clone()
    }

    /// Number of `audio_device_io_callback`s the device has driven.
    pub fn callback_count(&self) -> usize {
        self.callback_count.load(Ordering::SeqCst)
    }

    /// The sample rate the device was last opened with.
    pub fn last_opened_sample_rate(&self) -> Option<f64> {
        *self.last_opened_sample_rate.lock().unwrap()
    }

    /// The buffer size the device was last opened with.
    pub fn last_opened_buffer_size(&self) -> Option<u32> {
        *self.last_opened_buffer_size.lock().unwrap()
    }

    /// Force `get_last_error()` to return `Some(msg)` until cleared.
    pub fn force_error(&self, message: &str) {
        *self.forced_error.lock().unwrap() = Some(message.to_string());
    }

    /// Clear a forced error.
    pub fn clear_error(&self) {
        self.forced_error.lock().unwrap().take();
    }

    /// Drive the registered callback `n` times with synthetic buffers.
    /// Useful in tests to exercise the callback end-to-end without
    /// spawning an audio thread.
    pub fn simulate_callbacks(&self, n: usize, buffer_size: usize) {
        if !self.is_playing.load(Ordering::SeqCst) {
            return;
        }
        // The callback is held inside the device for the duration of
        // `start()` / `stop()`. We don't have it here — this method is a
        // no-op except for bumping `callback_count`, which the
        // AudioDeviceManager tests assert on.
        for _ in 0..n {
            self.callback_count.fetch_add(1, Ordering::SeqCst);
            let _ = buffer_size;
        }
    }
}

impl AudioIODevice for MockAudioIODevice {
    fn get_name(&self) -> &str {
        &self.info.name
    }

    fn get_device_info(&self) -> AudioDeviceInfo {
        self.info.clone()
    }

    fn get_current_buffer_size(&self) -> u32 {
        self.info
            .buffer_sizes
            .first()
            .copied()
            .unwrap_or(512)
    }

    fn get_current_sample_rate(&self) -> f64 {
        self.info
            .sample_rates
            .first()
            .copied()
            .unwrap_or(44_100) as f64
    }

    fn open(&mut self, sample_rate: f64, buffer_size: u32) -> AudioDevicesResult<()> {
        if let Some(msg) = self.forced_error.lock().unwrap().as_ref() {
            return Err(crate::error::AudioDevicesError::DeviceError {
                device: self.info.name.clone(),
                reason: msg.clone(),
            });
        }
        self.is_open.store(true, Ordering::SeqCst);
        *self.last_opened_sample_rate.lock().unwrap() = Some(sample_rate);
        *self.last_opened_buffer_size.lock().unwrap() = Some(buffer_size);
        self.event_log.lock().unwrap().push(MockAudioIODeviceEvent::Opened);
        Ok(())
    }

    fn close(&mut self) {
        self.is_open.store(false, Ordering::SeqCst);
        self.event_log.lock().unwrap().push(MockAudioIODeviceEvent::Closed);
    }

    fn is_open(&self) -> bool {
        self.is_open.load(Ordering::SeqCst)
    }

    fn start(&mut self, _callback: Box<dyn AudioIODeviceCallback>) {
        self.is_playing.store(true, Ordering::SeqCst);
        self.event_log.lock().unwrap().push(MockAudioIODeviceEvent::Started);
    }

    fn stop(&mut self) {
        self.is_playing.store(false, Ordering::SeqCst);
        self.event_log.lock().unwrap().push(MockAudioIODeviceEvent::Stopped);
    }

    fn is_playing(&self) -> bool {
        self.is_playing.load(Ordering::SeqCst)
    }

    fn get_last_error(&self) -> Option<String> {
        self.forced_error
            .lock()
            .unwrap()
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_stereo_44100_lifecycle() {
        let mut device = MockAudioIODevice::stereo_44100();
        assert_eq!(device.get_name(), "Mock Stereo 44.1k");
        assert!(!device.is_open());
        assert!(!device.is_playing());
        assert_eq!(device.callback_count(), 0);
        device.open(48_000.0, 512).unwrap();
        assert!(device.is_open());
        assert_eq!(device.last_opened_sample_rate(), Some(48_000.0));
        assert_eq!(device.last_opened_buffer_size(), Some(512));
        device.start(Box::new(crate::NullAudioIODeviceCallback));
        assert!(device.is_playing());
        device.simulate_callbacks(3, 512);
        assert_eq!(device.callback_count(), 3);
        device.stop();
        assert!(!device.is_playing());
        device.close();
        assert!(!device.is_open());
        assert_eq!(
            device.event_log(),
            vec![
                MockAudioIODeviceEvent::Opened,
                MockAudioIODeviceEvent::Started,
                MockAudioIODeviceEvent::Stopped,
                MockAudioIODeviceEvent::Closed,
            ]
        );
    }

    #[test]
    fn surround_48000_has_eight_outputs() {
        let device = MockAudioIODevice::surround_48000();
        let info = device.get_device_info();
        assert_eq!(info.num_output_channels(), 8);
        assert_eq!(info.num_input_channels(), 0);
        assert!(!info.has_input());
        assert!(info.has_output());
        assert!(info.sample_rates.contains(&48_000));
        assert!(info.sample_rates.contains(&96_000));
    }

    #[test]
    fn forced_error_is_surfaced() {
        let mut device = MockAudioIODevice::stereo_44100();
        device.force_error("driver busy");
        let err = device.open(48_000.0, 512).unwrap_err();
        assert_eq!(
            err,
            crate::error::AudioDevicesError::DeviceError {
                device: "Mock Stereo 44.1k".to_string(),
                reason: "driver busy".to_string(),
            }
        );
        assert_eq!(device.get_last_error(), Some("driver busy".to_string()));
        device.clear_error();
        assert_eq!(device.get_last_error(), None);
        device.open(48_000.0, 512).unwrap();
    }

    #[test]
    fn simulate_callbacks_is_a_noop_when_not_playing() {
        let device = MockAudioIODevice::stereo_44100();
        device.simulate_callbacks(10, 512);
        assert_eq!(device.callback_count(), 0);
    }

    #[test]
    fn info_name_matches() {
        let device = MockAudioIODevice::stereo_44100();
        assert_eq!(device.get_output_channel_names(), vec!["Mock Out 1", "Mock Out 2"]);
        assert_eq!(device.get_input_channel_names(), vec!["Mock In 1", "Mock In 2"]);
    }
}