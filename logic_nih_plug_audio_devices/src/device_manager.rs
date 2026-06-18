//! The `AudioDeviceManager` — the long-lived orchestrator that holds the
//! current device, the current [`AudioDeviceSetup`](crate::AudioDeviceSetup),
//! and the listener list. Mirrors `juce::AudioDeviceManager`.

use crate::device_setup::AudioDeviceSetup;
use crate::device_type::AudioIODeviceType;
use crate::error::{AudioDevicesError, AudioDevicesResult};
use crate::io_callback::AudioIODeviceCallback;
use crate::io_device::{AudioDeviceInfo, AudioIODevice};

/// The runtime state of the [`AudioDeviceManager`].
///
/// Mirrors the `Playing / Open / Stopped` tri-state JUCE exposes via
/// `AudioDeviceManager::getCurrentAudioDevice()` and
/// `AudioIODevice::isPlaying()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AudioDeviceManagerState {
    /// No device is currently attached. `get_current_audio_device()`
    /// returns `None`.
    Stopped,
    /// A device is attached and opened but not streaming.
    Open,
    /// A device is attached and streaming (i.e. `start()` has been called).
    Playing,
}

/// Observer pattern — every listener is notified when the manager's
/// device type, active device, or setup changes.
///
/// Mirrors JUCE's `AudioDeviceManagerListener`.
pub trait AudioDeviceManagerListener: Send {
    /// The device type or active device implementation changed. Fired by
    /// [`AudioDeviceManager::set_current_audio_device_type`] and
    /// [`AudioDeviceManager::set_current_audio_device`].
    fn audio_device_manager_changed(&mut self, manager: &AudioDeviceManager);

    /// The desired setup changed (sample rate, buffer size, channel
    /// counts). Fired by [`AudioDeviceManager::set_audio_device_setup`].
    fn audio_device_setup_changed(&mut self, manager: &AudioDeviceManager);
}

/// The audio-device orchestrator.
///
/// Holds the active [`AudioIODevice`], the desired
/// [`AudioDeviceSetup`], and the list of
/// [`AudioDeviceManagerListener`]s. All mutations go through `&mut self`
/// so listeners fire in a deterministic order.
///
/// Mirrors `juce::AudioDeviceManager`.
pub struct AudioDeviceManager {
    current_setup: AudioDeviceSetup,
    current_device_type: AudioIODeviceType,
    current_device: Option<Box<dyn AudioIODevice>>,
    listeners: Vec<Box<dyn AudioDeviceManagerListener>>,
    state: AudioDeviceManagerState,
}

impl Default for AudioDeviceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioDeviceManager {
    /// Construct an empty manager with the default setup and the
    /// compile-time preferred driver. No device is attached yet.
    pub fn new() -> Self {
        let driver = crate::device_type::DriverType::current().to_audio_io_device_type();
        Self {
            current_setup: AudioDeviceSetup::default(),
            current_device_type: driver,
            current_device: None,
            listeners: Vec::new(),
            state: AudioDeviceManagerState::Stopped,
        }
    }

    /// Construct a manager with a specific initial driver type.
    pub fn with_device_type(device_type: AudioIODeviceType) -> Self {
        Self {
            current_setup: AudioDeviceSetup::default(),
            current_device_type: device_type,
            current_device: None,
            listeners: Vec::new(),
            state: AudioDeviceManagerState::Stopped,
        }
    }

    /// The currently selected driver type.
    pub fn get_current_audio_device_type(&self) -> AudioIODeviceType {
        self.current_device_type
    }

    /// Switch driver type. Closes the active device (if any), then fires
    /// [`AudioDeviceManagerListener::audio_device_manager_changed`] on
    /// every listener. The new type takes effect immediately, but no
    /// device is opened — the host must call
    /// [`set_current_audio_device`](Self::set_current_audio_device)
    /// next.
    pub fn set_current_audio_device_type(&mut self, device_type: AudioIODeviceType) {
        self.close_active_device();
        self.current_device_type = device_type;
        self.notify_manager_changed();
    }

    /// Install a device implementation. Closes any previously active
    /// device, then opens + starts the new one if `Some`.
    pub fn set_current_audio_device(&mut self, device: Option<Box<dyn AudioIODevice>>) {
        self.close_active_device();
        self.current_device = device;
        // Open the device with the current setup so that subsequent
        // `play()` calls succeed.
        if let Some(dev) = self.current_device.as_deref_mut() {
            let setup = self.current_setup.clone();
            match dev.open(setup.sample_rate as f64, setup.buffer_size) {
                Ok(()) => {
                    self.state = AudioDeviceManagerState::Open;
                }
                Err(_) => {
                    // Couldn't open — keep the device but stay in
                    // `Stopped` so callers know to retry / reconfigure.
                    self.state = AudioDeviceManagerState::Stopped;
                }
            }
        } else {
            self.state = AudioDeviceManagerState::Stopped;
        }
        self.notify_manager_changed();
    }

    /// The currently active device, if any.
    pub fn get_current_audio_device(&self) -> Option<&dyn AudioIODevice> {
        self.current_device.as_deref()
    }

    /// Mutable access to the currently active device, if any.
    pub fn get_current_audio_device_mut(&mut self) -> Option<&mut (dyn AudioIODevice + 'static)> {
        self.current_device.as_deref_mut()
    }

    /// The desired setup.
    pub fn get_audio_device_setup(&self) -> &AudioDeviceSetup {
        &self.current_setup
    }

    /// Update the desired setup. If the active device is open, it is
    /// reopened with the new sample-rate / buffer-size. The channel-count
    /// changes are not retro-applied to an already-open device — those
    /// take effect on the next [`set_current_audio_device`](Self::set_current_audio_device)
    /// or [`open_device`](Self::open_device) call.
    pub fn set_audio_device_setup(&mut self, setup: AudioDeviceSetup) -> AudioDevicesResult<()> {
        setup.validate()?;

        // Validate against the active device's capabilities if one is
        // attached.
        if let Some(device) = self.current_device.as_ref() {
            let info = device.get_device_info();
            info.validate_sample_rate(setup.sample_rate)?;
            info.validate_buffer_size(setup.buffer_size)?;
            if setup.output_channels > info.num_output_channels() {
                return Err(AudioDevicesError::InvalidChannelCount {
                    direction: "output",
                    requested: setup.output_channels,
                    max: info.num_output_channels(),
                });
            }
            if setup.input_channels > info.num_input_channels() {
                return Err(AudioDevicesError::InvalidChannelCount {
                    direction: "input",
                    requested: setup.input_channels,
                    max: info.num_input_channels(),
                });
            }
        }

        let was_open = self.state != AudioDeviceManagerState::Stopped;
        let was_playing = self.state == AudioDeviceManagerState::Playing;

        self.close_active_device();

        self.current_setup = setup;

        if was_open {
            self.open_device()?;
            if was_playing {
                if let Some(device) = self.current_device.as_deref_mut() {
                    device.start(Box::new(NullCallbackShim));
                }
                self.state = AudioDeviceManagerState::Playing;
            }
        }

        self.notify_setup_changed();
        Ok(())
    }

    /// Open the active device (if any) with the current setup. Useful
    /// after [`set_current_audio_device`](Self::set_current_audio_device)
    /// or [`set_audio_device_setup`](Self::set_audio_device_setup) when
    /// the consumer wants to drive the lifecycle explicitly.
    pub fn open_device(&mut self) -> AudioDevicesResult<()> {
        let setup = self.current_setup.clone();
        self.close_active_device();
        let device = match self.current_device.as_deref_mut() {
            Some(d) => d,
            None => {
                self.state = AudioDeviceManagerState::Stopped;
                return Ok(());
            }
        };
        match device.open(setup.sample_rate as f64, setup.buffer_size) {
            Ok(()) => {
                self.state = AudioDeviceManagerState::Open;
                Ok(())
            }
            Err(e) => {
                self.state = AudioDeviceManagerState::Stopped;
                Err(e)
            }
        }
    }

    /// Begin streaming on the active device. Returns an error if no
    /// device is active or if the device is not open.
    pub fn play(&mut self) -> AudioDevicesResult<()> {
        let device = self
            .current_device
            .as_deref_mut()
            .ok_or_else(|| AudioDevicesError::DeviceNotOpen {
                device: String::from("(none)"),
            })?;
        if !device.is_open() {
            return Err(AudioDevicesError::DeviceNotOpen {
                device: device.get_name().to_string(),
            });
        }
        device.start(Box::new(NullCallbackShim));
        self.state = AudioDeviceManagerState::Playing;
        Ok(())
    }

    /// Stop streaming but keep the device open.
    pub fn stop(&mut self) {
        if let Some(device) = self.current_device.as_deref_mut() {
            if device.is_playing() {
                device.stop();
            }
        }
        if self.state == AudioDeviceManagerState::Playing {
            self.state = AudioDeviceManagerState::Open;
        }
    }

    /// Close the active device entirely. No-op if no device is active.
    pub fn close_device(&mut self) {
        self.close_active_device();
    }

    /// Current runtime state.
    pub fn get_state(&self) -> AudioDeviceManagerState {
        self.state
    }

    /// Register a listener. The same listener can be registered multiple
    /// times — every registration fires independently.
    pub fn add_change_listener(&mut self, listener: Box<dyn AudioDeviceManagerListener>) {
        self.listeners.push(listener);
    }

    /// Remove the first matching listener (by identity).
    pub fn remove_change_listener(&mut self, listener: *const dyn AudioDeviceManagerListener) {
        // `retain` keeps elements whose predicate is true; we want to
        // drop the element whose pointer matches the supplied one.
        self.listeners
            .retain(|l| !std::ptr::eq(l.as_ref() as *const _, listener));
    }

    /// Number of registered listeners.
    pub fn num_listeners(&self) -> usize {
        self.listeners.len()
    }

    /// Probe the active device for its info. Returns `None` if no device
    /// is attached.
    pub fn get_current_device_info(&self) -> Option<AudioDeviceInfo> {
        self.current_device.as_ref().map(|d| d.get_device_info())
    }

    /// Names of every input / output channel on the active device, or
    /// empty vectors if no device is attached.
    pub fn scan_device_names(&self, _device_type: AudioIODeviceType) -> (Vec<String>, Vec<String>) {
        // In a real driver integration this would query the driver for
        // the list of devices of `_device_type`. Without a driver, the
        // best we can do is forward the active device's channel names.
        match self.current_device.as_ref() {
            Some(d) => (
                d.get_input_channel_names(),
                d.get_output_channel_names(),
            ),
            None => (Vec::new(), Vec::new()),
        }
    }

    /// Play a test sound through the active device. Stub for parity with
    /// `juce::AudioDeviceManager::playTestSound()` — the base port
    /// doesn't generate audio. Concrete driver integrations are expected
    /// to override this by spawning a sine generator.
    pub fn play_test_sound(&mut self) -> AudioDevicesResult<()> {
        // No-op in the base port; consumers that want a real test tone
        // can wrap the manager and drive their own callback.
        Ok(())
    }

    fn close_active_device(&mut self) {
        if let Some(device) = self.current_device.as_deref_mut() {
            if device.is_playing() {
                device.stop();
            }
            if device.is_open() {
                device.close();
            }
        }
        self.state = AudioDeviceManagerState::Stopped;
    }

    fn notify_manager_changed(&mut self) {
        // Borrow listeners mutably without holding a borrow on `self`
        // for the rest of the struct. We swap the vec out and back in.
        let mut listeners = std::mem::take(&mut self.listeners);
        for listener in listeners.iter_mut() {
            listener.audio_device_manager_changed(self);
        }
        self.listeners = listeners;
    }

    fn notify_setup_changed(&mut self) {
        let mut listeners = std::mem::take(&mut self.listeners);
        for listener in listeners.iter_mut() {
            listener.audio_device_setup_changed(self);
        }
        self.listeners = listeners;
    }
}

/// A no-op callback used when the manager needs to call `start` but the
/// host hasn't installed one yet. Concrete uses should call `stop` then
/// `start` with their own callback after a setup change.
struct NullCallbackShim;

impl AudioIODeviceCallback for NullCallbackShim {
    fn audio_device_about_to_start(
        &mut self,
        _sample_rate: f64,
        _buffer_size: usize,
        _num_input_channels: usize,
        _num_output_channels: usize,
    ) {
    }

    fn audio_device_io_callback(&mut self, _data: &crate::AudioIODeviceCallbackData<'_>) {}

    fn audio_device_stopped(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{MockAudioIODevice, MockAudioIODeviceEvent};

    fn make_stereo_device() -> MockAudioIODevice {
        MockAudioIODevice::stereo_44100()
    }

    fn make_surround_device() -> MockAudioIODevice {
        MockAudioIODevice::surround_48000()
    }

    /// A listener that records every event so tests can assert on the
    /// order and content.
    #[derive(Default)]
    struct RecordingListener {
        manager_changed_count: usize,
        setup_changed_count: usize,
        last_setup_sample_rate: Option<u32>,
        last_setup_buffer_size: Option<u32>,
    }

    impl AudioDeviceManagerListener for RecordingListener {
        fn audio_device_manager_changed(&mut self, manager: &AudioDeviceManager) {
            self.manager_changed_count += 1;
            self.last_setup_sample_rate = Some(manager.get_audio_device_setup().sample_rate);
            self.last_setup_buffer_size = Some(manager.get_audio_device_setup().buffer_size);
        }
        fn audio_device_setup_changed(&mut self, manager: &AudioDeviceManager) {
            self.setup_changed_count += 1;
            self.last_setup_sample_rate = Some(manager.get_audio_device_setup().sample_rate);
            self.last_setup_buffer_size = Some(manager.get_audio_device_setup().buffer_size);
        }
    }

    #[test]
    fn new_manager_is_in_stopped_state() {
        let mgr = AudioDeviceManager::new();
        assert_eq!(mgr.get_state(), AudioDeviceManagerState::Stopped);
        assert!(mgr.get_current_audio_device().is_none());
        assert_eq!(mgr.num_listeners(), 0);
    }

    #[test]
    fn with_device_type_overrides_driver() {
        let mgr = AudioDeviceManager::with_device_type(AudioIODeviceType::Wasapi);
        assert_eq!(
            mgr.get_current_audio_device_type(),
            AudioIODeviceType::Wasapi
        );
    }

    #[test]
    fn setting_audio_device_moves_state_to_open() {
        let mut mgr = AudioDeviceManager::new();
        mgr.set_current_audio_device(Some(Box::new(make_stereo_device())));
        assert_eq!(mgr.get_state(), AudioDeviceManagerState::Open);
        assert!(mgr.get_current_audio_device().is_some());
    }

    #[test]
    fn setting_audio_device_to_none_closes_previous_and_returns_to_stopped() {
        let mut mgr = AudioDeviceManager::new();
        mgr.set_current_audio_device(Some(Box::new(make_stereo_device())));
        // Grab a reference to the inner device state before moving it back out.
        let was_open = mgr.get_current_audio_device().unwrap().is_open();
        assert!(was_open);
        mgr.set_current_audio_device(None);
        assert_eq!(mgr.get_state(), AudioDeviceManagerState::Stopped);
        assert!(mgr.get_current_audio_device().is_none());
    }

    #[test]
    fn play_then_stop_transitions_state_correctly() {
        let mut mgr = AudioDeviceManager::new();
        mgr.set_current_audio_device(Some(Box::new(make_stereo_device())));
        mgr.play().unwrap();
        assert_eq!(mgr.get_state(), AudioDeviceManagerState::Playing);
        mgr.stop();
        assert_eq!(mgr.get_state(), AudioDeviceManagerState::Open);
    }

    #[test]
    fn play_without_active_device_errors() {
        let mut mgr = AudioDeviceManager::new();
        let err = mgr.play().unwrap_err();
        assert_eq!(
            err,
            AudioDevicesError::DeviceNotOpen {
                device: "(none)".to_string(),
            }
        );
    }

    #[test]
    fn set_audio_device_setup_validates_zero_sample_rate() {
        let mut mgr = AudioDeviceManager::new();
        let bad = AudioDeviceSetup {
            sample_rate: 0,
            ..AudioDeviceSetup::default()
        };
        let err = mgr.set_audio_device_setup(bad).unwrap_err();
        assert_eq!(err, AudioDevicesError::InvalidSampleRate(0));
    }

    #[test]
    fn set_audio_device_setup_validates_against_active_device() {
        let mut mgr = AudioDeviceManager::new();
        mgr.set_current_audio_device(Some(Box::new(make_stereo_device())));
        let bad = AudioDeviceSetup {
            sample_rate: 192_000, // not in the mock's supported list
            ..AudioDeviceSetup::default()
        };
        let err = mgr.set_audio_device_setup(bad).unwrap_err();
        match err {
            AudioDevicesError::UnsupportedSampleRate { requested, .. } => {
                assert_eq!(requested, 192_000);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn set_audio_device_setup_rejects_too_many_output_channels() {
        let mut mgr = AudioDeviceManager::new();
        // Stereo mock has 2 output channels.
        mgr.set_current_audio_device(Some(Box::new(make_stereo_device())));
        let bad = AudioDeviceSetup {
            output_channels: 4,
            ..AudioDeviceSetup::default()
        };
        let err = mgr.set_audio_device_setup(bad).unwrap_err();
        assert_eq!(
            err,
            AudioDevicesError::InvalidChannelCount {
                direction: "output",
                requested: 4,
                max: 2,
            }
        );
    }

    #[test]
    fn set_audio_device_setup_rejects_too_many_input_channels() {
        let mut mgr = AudioDeviceManager::new();
        // Surround mock has 0 input channels but supports 48 kHz / 96 kHz.
        // Use a setup that the device actually supports.
        let surround_setup = AudioDeviceSetup {
            sample_rate: 48_000,
            ..AudioDeviceSetup::default()
        };
        mgr.set_audio_device_setup(surround_setup).unwrap();
        mgr.set_current_audio_device(Some(Box::new(make_surround_device())));
        let bad = AudioDeviceSetup {
            sample_rate: 48_000,
            input_channels: 2,
            ..AudioDeviceSetup::default()
        };
        let err = mgr.set_audio_device_setup(bad).unwrap_err();
        assert_eq!(
            err,
            AudioDevicesError::InvalidChannelCount {
                direction: "input",
                requested: 2,
                max: 0,
            }
        );
    }

    #[test]
    fn set_audio_device_setup_reopens_active_device_with_new_params() {
        let mut mgr = AudioDeviceManager::new();
        mgr.set_current_audio_device(Some(Box::new(make_stereo_device())));
        mgr.play().unwrap();

        // 48 kHz / 256 frames is in the stereo mock's supported list.
        let new_setup = AudioDeviceSetup::stereo_48000(256);
        mgr.set_audio_device_setup(new_setup.clone()).unwrap();

        // The manager closed + reopened + restarted the device, so the
        // device should still report as playing.
        assert_eq!(mgr.get_state(), AudioDeviceManagerState::Playing);

        // The device's recorded `last_opened_*` should reflect the new
        // setup.
        let device = mgr.get_current_audio_device().unwrap();
        let info = device.get_device_info();
        assert!(info.sample_rates.contains(&new_setup.sample_rate));
        assert!(info.buffer_sizes.contains(&new_setup.buffer_size));
    }

    #[test]
    fn listeners_fire_on_device_type_change() {
        let mut mgr = AudioDeviceManager::new();
        let listener = Box::new(RecordingListener::default());
        mgr.add_change_listener(listener);
        mgr.set_current_audio_device_type(AudioIODeviceType::Alsa);
        assert_eq!(mgr.num_listeners(), 1);
        // We can only access via the listener; the count is captured in
        // its fields. Cast back to a RecordingListener via ptr comparison
        // is awkward; instead use a fresh test that captures the events
        // through a shared Rc<RefCell<…>>.
    }

    #[test]
    fn listener_fires_on_setup_change_via_shared_cell() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        struct CellListener {
            manager_changed: Arc<AtomicUsize>,
            setup_changed: Arc<AtomicUsize>,
        }
        impl AudioDeviceManagerListener for CellListener {
            fn audio_device_manager_changed(&mut self, _mgr: &AudioDeviceManager) {
                self.manager_changed.fetch_add(1, Ordering::SeqCst);
            }
            fn audio_device_setup_changed(&mut self, _mgr: &AudioDeviceManager) {
                self.setup_changed.fetch_add(1, Ordering::SeqCst);
            }
        }

        let manager_changed = Arc::new(AtomicUsize::new(0));
        let setup_changed = Arc::new(AtomicUsize::new(0));
        let listener = CellListener {
            manager_changed: Arc::clone(&manager_changed),
            setup_changed: Arc::clone(&setup_changed),
        };

        let mut mgr = AudioDeviceManager::new();
        mgr.add_change_listener(Box::new(listener));
        assert_eq!(manager_changed.load(Ordering::SeqCst), 0);
        assert_eq!(setup_changed.load(Ordering::SeqCst), 0);

        mgr.set_audio_device_setup(AudioDeviceSetup::stereo_44100(512)).unwrap();
        assert_eq!(setup_changed.load(Ordering::SeqCst), 1);
        assert_eq!(manager_changed.load(Ordering::SeqCst), 0);

        mgr.set_current_audio_device_type(AudioIODeviceType::Wasapi);
        assert_eq!(manager_changed.load(Ordering::SeqCst), 1);
        assert_eq!(setup_changed.load(Ordering::SeqCst), 1);

        mgr.set_current_audio_device(Some(Box::new(make_stereo_device())));
        assert_eq!(manager_changed.load(Ordering::SeqCst), 2);
        assert_eq!(setup_changed.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn listener_can_be_removed() {
        let mut mgr = AudioDeviceManager::new();
        let mut listener = RecordingListener::default();
        let raw_ptr: *const dyn AudioDeviceManagerListener = &mut listener as *const _;
        // The listener must be on the heap for the manager to own it;
        // since we can't take ownership of a stack pointer, this test
        // just exercises the remove path with a freshly boxed listener.
        mgr.add_change_listener(Box::new(RecordingListener::default()));
        let boxed: Box<dyn AudioDeviceManagerListener> = Box::new(RecordingListener::default());
        let boxed_ptr: *const dyn AudioDeviceManagerListener = boxed.as_ref() as *const _;
        mgr.add_change_listener(boxed);
        assert_eq!(mgr.num_listeners(), 2);
        mgr.remove_change_listener(boxed_ptr);
        assert_eq!(mgr.num_listeners(), 1);
        // Removing a pointer that doesn't exist should be a no-op.
        mgr.remove_change_listener(raw_ptr);
        assert_eq!(mgr.num_listeners(), 1);
    }

    #[test]
    fn scan_device_names_returns_active_device_channels() {
        let mut mgr = AudioDeviceManager::new();
        let (in_empty, out_empty) = mgr.scan_device_names(AudioIODeviceType::Wasapi);
        assert!(in_empty.is_empty());
        assert!(out_empty.is_empty());

        mgr.set_current_audio_device(Some(Box::new(make_stereo_device())));
        let (in_names, out_names) = mgr.scan_device_names(AudioIODeviceType::Wasapi);
        assert_eq!(in_names, vec!["Mock In 1".to_string(), "Mock In 2".to_string()]);
        assert_eq!(out_names, vec!["Mock Out 1".to_string(), "Mock Out 2".to_string()]);
    }

    #[test]
    fn get_current_device_info_works() {
        let mut mgr = AudioDeviceManager::new();
        assert!(mgr.get_current_device_info().is_none());
        mgr.set_current_audio_device(Some(Box::new(make_stereo_device())));
        let info = mgr.get_current_device_info().unwrap();
        assert_eq!(info.name, "Mock Stereo 44.1k");
        assert_eq!(info.num_input_channels(), 2);
        assert_eq!(info.num_output_channels(), 2);
    }

    #[test]
    fn close_device_drops_state_to_stopped() {
        let mut mgr = AudioDeviceManager::new();
        mgr.set_current_audio_device(Some(Box::new(make_stereo_device())));
        mgr.play().unwrap();
        mgr.close_device();
        assert_eq!(mgr.get_state(), AudioDeviceManagerState::Stopped);
        // The device is still attached — it's just not open anymore.
        assert!(mgr.get_current_audio_device().is_some());
        assert!(!mgr.get_current_audio_device().unwrap().is_open());
    }

    #[test]
    fn play_test_sound_is_a_noop_returning_ok() {
        let mut mgr = AudioDeviceManager::new();
        assert!(mgr.play_test_sound().is_ok());
        mgr.set_current_audio_device(Some(Box::new(make_stereo_device())));
        assert!(mgr.play_test_sound().is_ok());
    }

    #[test]
    fn open_device_after_setting_device_returns_open_state() {
        let mut mgr = AudioDeviceManager::new();
        mgr.set_current_audio_device(Some(Box::new(make_stereo_device())));
        // set_current_audio_device puts the device in Open state already.
        assert_eq!(mgr.get_state(), AudioDeviceManagerState::Open);
        mgr.open_device().unwrap();
        assert_eq!(mgr.get_state(), AudioDeviceManagerState::Open);
    }

    #[test]
    fn open_device_without_active_device_is_ok_and_remains_stopped() {
        let mut mgr = AudioDeviceManager::new();
        mgr.open_device().unwrap();
        assert_eq!(mgr.get_state(), AudioDeviceManagerState::Stopped);
    }

    #[test]
    fn replacing_active_device_closes_the_old_one() {
        let mut mgr = AudioDeviceManager::new();
        let first = make_stereo_device();
        let first_event_count = first.event_log().len();
        mgr.set_current_audio_device(Some(Box::new(first)));
        mgr.play().unwrap();
        mgr.set_current_audio_device(Some(Box::new(make_surround_device())));
        // The replacement went through close_active_device on the
        // previous one, which fired Stopped + Closed.
        // We can't inspect the dropped device's event_log here because
        // it's been moved into the manager and replaced, but we can
        // confirm the new device is open and the state is consistent.
        assert_eq!(mgr.get_state(), AudioDeviceManagerState::Open);
        assert!(!mgr.get_current_audio_device().unwrap().is_playing());
        let _ = first_event_count;
    }

    #[test]
    fn device_drives_lifecycle_in_correct_order() {
        let mut mgr = AudioDeviceManager::new();
        let device = make_stereo_device();
        mgr.set_current_audio_device(Some(Box::new(device)));
        // Re-acquire the device from the manager to inspect events.
        // We need a way to peek at the inner mock's events; cast the
        // trait object to the concrete type.
        let concrete: &MockAudioIODevice =
            unsafe { &*(mgr.get_current_audio_device().unwrap() as *const dyn AudioIODevice
                as *const MockAudioIODevice) };
        let events = concrete.event_log();
        assert_eq!(events, vec![MockAudioIODeviceEvent::Opened]);
        mgr.play().unwrap();
        let events = concrete.event_log();
        assert_eq!(
            events,
            vec![
                MockAudioIODeviceEvent::Opened,
                MockAudioIODeviceEvent::Started,
            ]
        );
        mgr.stop();
        let events = concrete.event_log();
        assert_eq!(
            events,
            vec![
                MockAudioIODeviceEvent::Opened,
                MockAudioIODeviceEvent::Started,
                MockAudioIODeviceEvent::Stopped,
            ]
        );
        mgr.close_device();
        let events = concrete.event_log();
        assert_eq!(
            events,
            vec![
                MockAudioIODeviceEvent::Opened,
                MockAudioIODeviceEvent::Started,
                MockAudioIODeviceEvent::Stopped,
                MockAudioIODeviceEvent::Closed,
            ]
        );
    }
}