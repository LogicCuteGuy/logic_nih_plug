//! A simple non-interleaved audio sample buffer.
//!
//! [`AudioSampleBuffer`] mirrors the JUCE class of the same name. It owns its
//! samples (one [`Vec<f32>`] per channel) and is the right thing to reach
//! for when you need an allocation-owning container — for example when
//! loading samples from a WAV file, or when copying audio between two
//! graphs.
//!
//! If you're inside a plugin's `process()` callback, you almost certainly
//! want [`logic_nih_plug::buffer::Buffer`] instead — it wraps the host's
//! preallocated scratch buffers and does no allocation. This type is for
//! the outside-of-`process()` use cases.
//!
//! ## Layout
//!
//! Storage is **non-interleaved**: channel 0 lives in `channels[0]`, channel
//! 1 in `channels[1]`, etc. This matches JUCE's default storage layout and
//! means that inner loops touch a single channel's samples with no strided
//! access. For the interleaved case, use [`AudioSampleBuffer::interleave`]
//! and [`AudioSampleBuffer::deinterleave`].
//!
//! ## Example
//!
//! ```
//! use logic_nih_plug_audio_basics::{AudioChannelSet, AudioSampleBuffer};
//!
//! let mut buf = AudioSampleBuffer::new(AudioChannelSet::Stereo, 4);
//! buf.set_sample(0, 0, 1.0);
//! buf.set_sample(1, 1, -1.0);
//!
//! let mut interleaved = vec![0.0_f32; 8];
//! buf.interleave(&mut interleaved);
//! assert_eq!(interleaved, vec![1.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0]);
//! ```

use crate::audio_channel_set::AudioChannelSet;
use crate::error::{AudioBasicsError, AudioBasicsResult};

/// An allocation-owning non-interleaved audio buffer.
///
/// Channels are stored as independent [`Vec<f32>`]s; each channel's vector
/// has length [`AudioSampleBuffer::num_samples`]. All channels are the
/// same length.
#[derive(Debug, Clone, PartialEq)]
pub struct AudioSampleBuffer {
    /// One `Vec<f32>` per channel. Each `Vec` has length `num_samples`.
    channels: Vec<Vec<f32>>,
    /// The number of samples per channel. Stored separately from
    /// `channels[0].len()` so we can keep the invariant explicit.
    num_samples: usize,
}

impl AudioSampleBuffer {
    /// Create a new buffer covering the given channel layout and sample
    /// count. All samples are initialised to zero.
    ///
    /// # Errors
    ///
    /// Returns [`AudioBasicsError::InvalidSize`] if `num_samples == 0`.
    pub fn new(layout: AudioChannelSet, num_samples: usize) -> Self {
        // `AudioChannelSet` cannot have zero channels, so the only failure
        // mode here is `num_samples == 0`. We don't want to make this
        // fallible for what is, in practice, always a programmer error, so
        // we assert rather than return Result — but in debug builds the
        // assert will fire loud.
        assert!(num_samples > 0, "AudioSampleBuffer: num_samples must be > 0");
        let n_channels = layout.num_channels();
        let channels = (0..n_channels).map(|_| vec![0.0; num_samples]).collect();
        Self {
            channels,
            num_samples,
        }
    }

    /// The number of channels in this buffer.
    #[inline]
    pub fn num_channels(&self) -> usize {
        self.channels.len()
    }

    /// The number of samples per channel.
    #[inline]
    pub fn num_samples(&self) -> usize {
        self.num_samples
    }

    /// Returns `true` if the buffer has no channels or no samples.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.num_samples == 0 || self.channels.is_empty()
    }

    /// The total number of sample slots in the buffer (`channels × samples`).
    #[inline]
    pub fn num_sample_slots(&self) -> usize {
        self.num_channels() * self.num_samples
    }

    /// Immutable access to one channel's sample vector.
    ///
    /// # Panics
    ///
    /// Panics if `channel` is out of range.
    #[inline]
    pub fn channel(&self, channel: usize) -> &[f32] {
        &self.channels[channel]
    }

    /// Mutable access to one channel's sample vector.
    ///
    /// # Panics
    ///
    /// Panics if `channel` is out of range.
    #[inline]
    pub fn channel_mut(&mut self, channel: usize) -> &mut [f32] {
        &mut self.channels[channel]
    }

    /// Read a single sample at `(channel, sample_index)`.
    ///
    /// # Panics
    ///
    /// Panics if `channel` or `sample_index` is out of range.
    #[inline]
    pub fn get_sample(&self, channel: usize, sample_index: usize) -> f32 {
        self.channels[channel][sample_index]
    }

    /// Write a single sample at `(channel, sample_index)`.
    ///
    /// # Panics
    ///
    /// Panics if `channel` or `sample_index` is out of range.
    #[inline]
    pub fn set_sample(&mut self, channel: usize, sample_index: usize, value: f32) {
        self.channels[channel][sample_index] = value;
    }

    /// Add `value` to the sample at `(channel, sample_index)`.
    ///
    /// # Panics
    ///
    /// Panics if `channel` or `sample_index` is out of range.
    #[inline]
    pub fn add_sample(&mut self, channel: usize, sample_index: usize, value: f32) {
        self.channels[channel][sample_index] += value;
    }

    /// Apply a linear gain to every sample in the buffer.
    pub fn apply_gain(&mut self, gain: f32) {
        for ch in &mut self.channels {
            for s in ch.iter_mut() {
                *s *= gain;
            }
        }
    }

    /// Apply a per-channel linear gain.
    ///
    /// # Panics
    ///
    /// Panics if `gains.len() != self.num_channels()`.
    pub fn apply_channel_gains(&mut self, gains: &[f32]) {
        assert_eq!(
            gains.len(),
            self.num_channels(),
            "gains.len() must equal num_channels"
        );
        for (ch, &g) in self.channels.iter_mut().zip(gains.iter()) {
            for s in ch.iter_mut() {
                *s *= g;
            }
        }
    }

    /// Zero out every sample in the buffer.
    pub fn clear(&mut self) {
        for ch in &mut self.channels {
            for s in ch.iter_mut() {
                *s = 0.0;
            }
        }
    }

    /// Copy every sample from `source` into `self`. The two buffers must
    /// have the same channel count and sample count.
    ///
    /// # Panics
    ///
    /// Panics if the channel counts differ, or if any channel of `source`
    /// has a different length than the corresponding channel of `self`.
    pub fn copy_from(&mut self, source: &AudioSampleBuffer) {
        assert_eq!(
            self.num_channels(),
            source.num_channels(),
            "channel counts differ"
        );
        for (dst, src) in self.channels.iter_mut().zip(source.channels.iter()) {
            assert_eq!(dst.len(), src.len(), "channel lengths differ");
            dst.copy_from_slice(src);
        }
    }

    /// Add every sample of `source` into `self`. Channel counts and sample
    /// counts must match (see [`copy_from`][Self::copy_from]).
    ///
    /// # Panics
    ///
    /// Same as [`copy_from`][Self::copy_from].
    pub fn add_from(&mut self, source: &AudioSampleBuffer) {
        assert_eq!(
            self.num_channels(),
            source.num_channels(),
            "channel counts differ"
        );
        for (dst, src) in self.channels.iter_mut().zip(source.channels.iter()) {
            assert_eq!(dst.len(), src.len(), "channel lengths differ");
            for (d, s) in dst.iter_mut().zip(src.iter()) {
                *d += *s;
            }
        }
    }

    /// Resize the buffer in place to `new_num_samples` samples per channel.
    /// The new samples (if growing) are initialised to zero.
    ///
    /// # Errors
    ///
    /// Returns [`AudioBasicsError::InvalidSize`] if `new_num_samples == 0`.
    pub fn set_size(&mut self, new_num_samples: usize) -> AudioBasicsResult<()> {
        if new_num_samples == 0 {
            return Err(AudioBasicsError::InvalidSize(0));
        }
        for ch in &mut self.channels {
            ch.resize(new_num_samples, 0.0);
        }
        self.num_samples = new_num_samples;
        Ok(())
    }

    /// Interleave the buffer's samples into a single `[f32]`.
    ///
    /// The output is laid out in the JUCE order: `[L0, R0, L1, R1, …]` for
    /// stereo, `[L0, R0, C0, LFE0, Ls0, Rs0, …]` for 5.1, etc.
    ///
    /// # Errors
    ///
    /// Returns [`AudioBasicsError::InvalidSize`] if `output.len() !=
    /// self.num_channels() * self.num_samples()`.
    pub fn interleave(&self, output: &mut [f32]) -> AudioBasicsResult<()> {
        let expected = self.num_sample_slots();
        if output.len() != expected {
            return Err(AudioBasicsError::InvalidSize(output.len()));
        }
        let nc = self.num_channels();
        let ns = self.num_samples;
        for (frame, slot) in output.chunks_exact_mut(nc).enumerate() {
            for (ch_idx, dst) in slot.iter_mut().enumerate() {
                *dst = self.channels[ch_idx][frame];
            }
        }
        // Suppress the unused `ns` lint in release builds where the loop
        // bound is encoded in `chunks_exact_mut`.
        let _ = ns;
        Ok(())
    }

    /// Deinterleave a single `[f32]` into the buffer, replacing its
    /// current contents. The input is laid out in the same order as
    /// [`interleave`][Self::interleave] produces.
    ///
    /// # Errors
    ///
    /// Returns [`AudioBasicsError::InvalidSize`] if `input.len() !=
    /// self.num_channels() * self.num_samples()`.
    pub fn deinterleave(&mut self, input: &[f32]) -> AudioBasicsResult<()> {
        let expected = self.num_sample_slots();
        if input.len() != expected {
            return Err(AudioBasicsError::InvalidSize(input.len()));
        }
        let nc = self.num_channels();
        for (frame, slot) in input.chunks_exact(nc).enumerate() {
            for (ch_idx, &sample) in slot.iter().enumerate() {
                self.channels[ch_idx][frame] = sample;
            }
        }
        Ok(())
    }

    /// Read-only slice-of-channels view (`[&[f32]; N]`-like, but heap-sized).
    ///
    /// Mostly useful for testing. For per-channel processing inside an
    /// audio thread, prefer [`channel_mut`][Self::channel_mut] in a loop.
    pub fn channels(&self) -> &[Vec<f32>] {
        &self.channels
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_zeroed_and_correct_shape() {
        let buf = AudioSampleBuffer::new(AudioChannelSet::FiveDotOne, 100);
        assert_eq!(buf.num_channels(), 6);
        assert_eq!(buf.num_samples(), 100);
        assert_eq!(buf.num_sample_slots(), 600);
        for ch in 0..6 {
            for s in 0..100 {
                assert_eq!(buf.get_sample(ch, s), 0.0);
            }
        }
    }

    #[test]
    fn set_get_sample_roundtrip() {
        let mut buf = AudioSampleBuffer::new(AudioChannelSet::Stereo, 8);
        buf.set_sample(0, 3, 1.5);
        buf.set_sample(1, 7, -0.25);
        assert_eq!(buf.get_sample(0, 3), 1.5);
        assert_eq!(buf.get_sample(1, 7), -0.25);
        assert_eq!(buf.get_sample(0, 7), 0.0);
    }

    #[test]
    fn add_sample_accumulates() {
        let mut buf = AudioSampleBuffer::new(AudioChannelSet::Mono, 4);
        buf.add_sample(0, 0, 1.0);
        buf.add_sample(0, 0, 2.5);
        assert_eq!(buf.get_sample(0, 0), 3.5);
    }

    #[test]
    fn apply_gain_scales_everything() {
        let mut buf = AudioSampleBuffer::new(AudioChannelSet::Stereo, 4);
        buf.set_sample(0, 0, 1.0);
        buf.set_sample(1, 3, 2.0);
        buf.apply_gain(0.5);
        assert_eq!(buf.get_sample(0, 0), 0.5);
        assert_eq!(buf.get_sample(1, 3), 1.0);
    }

    #[test]
    fn apply_channel_gains() {
        let mut buf = AudioSampleBuffer::new(AudioChannelSet::Stereo, 2);
        buf.set_sample(0, 0, 1.0);
        buf.set_sample(1, 1, 1.0);
        buf.apply_channel_gains(&[0.25, 0.75]);
        assert_eq!(buf.get_sample(0, 0), 0.25);
        assert_eq!(buf.get_sample(1, 1), 0.75);
    }

    #[test]
    fn clear_zeros_everything() {
        let mut buf = AudioSampleBuffer::new(AudioChannelSet::Mono, 4);
        buf.set_sample(0, 0, 1.0);
        buf.set_sample(0, 3, -1.0);
        buf.clear();
        for s in 0..4 {
            assert_eq!(buf.get_sample(0, s), 0.0);
        }
    }

    #[test]
    fn copy_from_replaces_contents() {
        let mut dst = AudioSampleBuffer::new(AudioChannelSet::Stereo, 3);
        let mut src = AudioSampleBuffer::new(AudioChannelSet::Stereo, 3);
        src.set_sample(0, 0, 0.5);
        src.set_sample(1, 2, -0.5);
        dst.copy_from(&src);
        assert_eq!(dst.get_sample(0, 0), 0.5);
        assert_eq!(dst.get_sample(1, 2), -0.5);
    }

    #[test]
    fn add_from_accumulates() {
        let mut dst = AudioSampleBuffer::new(AudioChannelSet::Stereo, 2);
        dst.set_sample(0, 0, 1.0);
        let mut src = AudioSampleBuffer::new(AudioChannelSet::Stereo, 2);
        src.set_sample(0, 0, 0.25);
        src.set_sample(1, 1, -0.5);
        dst.add_from(&src);
        assert_eq!(dst.get_sample(0, 0), 1.25);
        assert_eq!(dst.get_sample(1, 1), -0.5);
    }

    #[test]
    fn set_size_grows_and_zero_pads() {
        let mut buf = AudioSampleBuffer::new(AudioChannelSet::Mono, 2);
        buf.set_sample(0, 0, 7.0);
        buf.set_sample(0, 1, 8.0);
        buf.set_size(4).unwrap();
        assert_eq!(buf.num_samples(), 4);
        assert_eq!(buf.get_sample(0, 0), 7.0);
        assert_eq!(buf.get_sample(0, 1), 8.0);
        assert_eq!(buf.get_sample(0, 2), 0.0);
        assert_eq!(buf.get_sample(0, 3), 0.0);
    }

    #[test]
    fn set_size_shrinks() {
        let mut buf = AudioSampleBuffer::new(AudioChannelSet::Mono, 4);
        buf.set_sample(0, 0, 1.0);
        buf.set_sample(0, 3, 4.0);
        buf.set_size(2).unwrap();
        assert_eq!(buf.num_samples(), 2);
        assert_eq!(buf.get_sample(0, 0), 1.0);
    }

    #[test]
    fn set_size_zero_is_rejected() {
        let mut buf = AudioSampleBuffer::new(AudioChannelSet::Mono, 4);
        assert_eq!(buf.set_size(0), Err(AudioBasicsError::InvalidSize(0)));
    }

    #[test]
    fn interleave_roundtrip() {
        let mut buf = AudioSampleBuffer::new(AudioChannelSet::Stereo, 3);
        buf.set_sample(0, 0, 1.0);
        buf.set_sample(1, 0, 2.0);
        buf.set_sample(0, 1, 3.0);
        buf.set_sample(1, 2, 4.0);

        let mut flat = vec![0.0_f32; 6];
        buf.interleave(&mut flat).unwrap();
        assert_eq!(flat, vec![1.0, 2.0, 3.0, 0.0, 0.0, 4.0]);

        let mut restored = AudioSampleBuffer::new(AudioChannelSet::Stereo, 3);
        restored.deinterleave(&flat).unwrap();
        assert_eq!(restored, buf);
    }

    #[test]
    fn interleave_wrong_size_is_rejected() {
        let buf = AudioSampleBuffer::new(AudioChannelSet::Stereo, 4);
        let mut out = vec![0.0_f32; 7];
        assert_eq!(
            buf.interleave(&mut out),
            Err(AudioBasicsError::InvalidSize(7))
        );
    }

    #[test]
    fn interleave_then_deinterleave_matches_for_five_dot_one() {
        let mut buf = AudioSampleBuffer::new(AudioChannelSet::FiveDotOne, 8);
        for ch in 0..6 {
            for s in 0..8 {
                buf.set_sample(ch, s, (ch * 10 + s) as f32);
            }
        }
        let mut flat = vec![0.0_f32; buf.num_sample_slots()];
        buf.interleave(&mut flat).unwrap();

        let mut round = AudioSampleBuffer::new(AudioChannelSet::FiveDotOne, 8);
        round.deinterleave(&flat).unwrap();
        assert_eq!(round, buf);
    }

    #[test]
    fn channel_accessors() {
        let mut buf = AudioSampleBuffer::new(AudioChannelSet::Stereo, 2);
        buf.set_sample(0, 0, 1.0);
        buf.set_sample(1, 1, -1.0);
        assert_eq!(buf.channel(0), &[1.0, 0.0]);
        assert_eq!(buf.channel(1), &[0.0, -1.0]);
        buf.channel_mut(0)[1] = 0.5;
        assert_eq!(buf.get_sample(0, 1), 0.5);
    }

    #[test]
    fn mono_buffer_helpers() {
        let mut buf = AudioSampleBuffer::new(AudioChannelSet::Mono, 4);
        assert!(buf.num_channels() == 1);
        buf.set_sample(0, 0, 1.0);
        assert!(!buf.is_empty());
    }
}
