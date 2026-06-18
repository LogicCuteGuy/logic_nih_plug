//! The `AudioIODeviceCallback` trait — the audio-thread callback contract.
//!
//! Mirrors `juce::AudioIODeviceCallback`.

/// The data handed to the audio thread on each callback.
///
/// The buffers are `&[f32]` slices — non-interleaved, one slice per channel
/// — to keep the contract allocation-free. The callback runs on the
/// real-time audio thread, so implementations must NOT allocate, lock
/// mutexes that can block, or call into the OS.
#[derive(Debug)]
pub struct AudioIODeviceCallbackData<'a> {
    /// The active input channels (read-only). `None` if the channel is
    /// disabled.
    pub input_channels: &'a [&'a [f32]],
    /// The active output channels (mutable). `None` if the channel is
    /// disabled.
    pub output_channels: &'a [&'a mut [f32]],
    /// Number of samples in each buffer (all channels are guaranteed to
    /// have the same length).
    pub num_samples: usize,
}

impl<'a> AudioIODeviceCallbackData<'a> {
    /// Construct a callback data view from parallel slices.
    pub fn new(
        input_channels: &'a [&'a [f32]],
        output_channels: &'a [&'a mut [f32]],
        num_samples: usize,
    ) -> Self {
        Self {
            input_channels,
            output_channels,
            num_samples,
        }
    }

    /// Number of input channels that are currently active (i.e. buffers
    /// that have data).
    pub fn num_input_channels(&self) -> usize {
        self.input_channels.len()
    }

    /// Number of output channels that are currently active.
    pub fn num_output_channels(&self) -> usize {
        self.output_channels.len()
    }
}

/// The audio-thread callback contract.
///
/// Implementations receive three lifecycle events from
/// [`AudioIODevice`](crate::AudioIODevice):
///
/// 1. `audio_device_about_to_start` — once, before the first
///    `audio_device_io_callback`. Use this to allocate scratch buffers or
///    prepare smoothing state.
/// 2. `audio_device_io_callback` — once per audio buffer. **Real-time**
///    thread — no allocations, no syscalls, no blocking locks.
/// 3. `audio_device_stopped` — once, after the last callback. Use this to
///    release scratch state.
///
/// Mirrors `juce::AudioIODeviceCallback`.
pub trait AudioIODeviceCallback: Send {
    /// Called once, on the audio thread, immediately before the first
    /// `audio_device_io_callback`. The `sample_rate`, `buffer_size`, and
    /// channel counts are passed in directly so the callback doesn't need
    /// to query the device.
    fn audio_device_about_to_start(
        &mut self,
        sample_rate: f64,
        buffer_size: usize,
        num_input_channels: usize,
        num_output_channels: usize,
    );

    /// Called once per audio buffer, on the audio thread. Real-time
    /// constraints apply — see [`AudioIODeviceCallbackData`](crate::AudioIODeviceCallbackData).
    fn audio_device_io_callback(&mut self, data: &AudioIODeviceCallbackData<'_>);

    /// Called once, on the audio thread, immediately after the last
    /// `audio_device_io_callback`. Use this to release scratch state.
    fn audio_device_stopped(&mut self);
}

/// A no-op `AudioIODeviceCallback` for testing and for hosts that don't
/// need lifecycle events.
#[derive(Debug, Default, Clone, Copy)]
pub struct NullAudioIODeviceCallback;

impl AudioIODeviceCallback for NullAudioIODeviceCallback {
    fn audio_device_about_to_start(
        &mut self,
        _sample_rate: f64,
        _buffer_size: usize,
        _num_input_channels: usize,
        _num_output_channels: usize,
    ) {
    }

    fn audio_device_io_callback(&mut self, _data: &AudioIODeviceCallbackData<'_>) {}

    fn audio_device_stopped(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_callback_lifecycle_is_a_no_op() {
        let mut cb = NullAudioIODeviceCallback;
        cb.audio_device_about_to_start(48_000.0, 512, 2, 2);
        cb.audio_device_io_callback(&AudioIODeviceCallbackData {
            input_channels: &[],
            output_channels: &[],
            num_samples: 0,
        });
        cb.audio_device_stopped();
    }

    #[test]
    fn callback_data_counts_channels() {
        let in0 = [0.0_f32; 4];
        let in1 = [0.0_f32; 4];
        let mut out0 = [0.0_f32; 4];
        let mut out1 = [0.0_f32; 4];
        let inputs: [&[f32]; 2] = [&in0, &in1];
        let outputs: [&mut [f32]; 2] = [&mut out0, &mut out1];
        let data = AudioIODeviceCallbackData::new(&inputs, &outputs, 4);
        assert_eq!(data.num_input_channels(), 2);
        assert_eq!(data.num_output_channels(), 2);
        assert_eq!(data.num_samples, 4);
    }

    /// A callback that records every lifecycle event so we can assert the
    /// order of calls in the [`AudioDeviceManager`](crate::AudioDeviceManager)
    /// tests.
    #[derive(Default)]
    struct RecordingCallback {
        events: Vec<&'static str>,
        sample_rate: f64,
        buffer_size: usize,
        num_in: usize,
        num_out: usize,
        callbacks_received: usize,
    }

    impl AudioIODeviceCallback for RecordingCallback {
        fn audio_device_about_to_start(
            &mut self,
            sample_rate: f64,
            buffer_size: usize,
            num_input_channels: usize,
            num_output_channels: usize,
        ) {
            self.events.push("about_to_start");
            self.sample_rate = sample_rate;
            self.buffer_size = buffer_size;
            self.num_in = num_input_channels;
            self.num_out = num_output_channels;
        }
        fn audio_device_io_callback(&mut self, _data: &AudioIODeviceCallbackData<'_>) {
            self.events.push("io_callback");
            self.callbacks_received += 1;
        }
        fn audio_device_stopped(&mut self) {
            self.events.push("stopped");
        }
    }

    #[test]
    fn recording_callback_lifecycle() {
        let mut cb = RecordingCallback::default();
        cb.audio_device_about_to_start(48_000.0, 512, 2, 2);
        cb.audio_device_io_callback(&AudioIODeviceCallbackData {
            input_channels: &[],
            output_channels: &[],
            num_samples: 512,
        });
        cb.audio_device_io_callback(&AudioIODeviceCallbackData {
            input_channels: &[],
            output_channels: &[],
            num_samples: 512,
        });
        cb.audio_device_stopped();
        assert_eq!(
            cb.events,
            vec!["about_to_start", "io_callback", "io_callback", "stopped"]
        );
        assert_eq!(cb.sample_rate, 48_000.0);
        assert_eq!(cb.buffer_size, 512);
        assert_eq!(cb.num_in, 2);
        assert_eq!(cb.num_out, 2);
        assert_eq!(cb.callbacks_received, 2);
    }
}