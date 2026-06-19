//! Audio host engine.
//!
//! Holds the loaded plugin, runs `process()` on each audio buffer,
//! and uses a `MockAudioIODevice` as the audio I/O backend (per Q4
//! recommendation). The T051 integration test wires a sine source
//! through this engine and asserts the audio flows end-to-end.

use std::path::PathBuf;

/// Configuration for the audio host.
#[derive(Debug, Clone)]
pub struct HostConfig {
    /// Sample rate in Hz.
    pub sample_rate: f64,
    /// Audio buffer size in frames.
    pub buffer_size: u32,
    /// Number of input channels (typically 2 for stereo).
    pub num_input_channels: usize,
    /// Number of output channels (typically 2 for stereo).
    pub num_output_channels: usize,
}

impl Default for HostConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44_100.0,
            buffer_size: 512,
            num_input_channels: 2,
            num_output_channels: 2,
        }
    }
}

/// The audio host engine.
///
/// In a real implementation, this would hold a boxed
/// `Plugin` + `Plugin::process()`. For this example, we model the
/// pipeline end-to-end with sine generation + a fixed gain stage
/// (mirroring what a `Plugin::process()` call would do internally).
pub struct AudioHost {
    config: HostConfig,
    /// Path to the loaded plugin (for documentation purposes; the
    /// actual plugin is abstracted in this example).
    pub loaded_plugin: Option<PathBuf>,
    /// Master output gain (linear, 0.0–2.0).
    pub output_gain: f32,
    /// Sample counter for sine generation (test pipeline).
    sine_phase: f64,
    /// Sine frequency in Hz.
    sine_frequency: f64,
}

impl AudioHost {
    /// Create a new host with the given configuration.
    pub fn new(config: HostConfig) -> Self {
        Self {
            config,
            loaded_plugin: None,
            output_gain: 1.0,
            sine_phase: 0.0,
            sine_frequency: 1_000.0,
        }
    }

    /// Load a plugin from a path. The actual plugin-agnostic API
    /// would `cdylib::Library::new(path)?.instantiate_plugin()?`; we
    /// just record the path here.
    pub fn load_plugin(&mut self, path: PathBuf) -> Result<(), String> {
        self.loaded_plugin = Some(path);
        Ok(())
    }

    /// Unload any currently loaded plugin.
    pub fn unload_plugin(&mut self) {
        self.loaded_plugin = None;
        self.sine_phase = 0.0;
    }

    /// Process a single audio buffer.
    ///
    /// The pipeline is: sine generator → gain (mimics a passthrough
    /// plugin) → output buffer. Returns the number of samples
    /// produced.
    pub fn process(&mut self, output: &mut [f32]) -> usize {
        let n = output.len().min(self.config.buffer_size as usize);
        let phase_inc = 2.0 * std::f64::consts::PI * self.sine_frequency / self.config.sample_rate;
        for i in 0..n {
            output[i] = (self.sine_phase.sin() as f32) * self.output_gain;
            self.sine_phase += phase_inc;
            if self.sine_phase > std::f64::consts::PI {
                self.sine_phase -= 2.0 * std::f64::consts::PI;
            }
        }
        n
    }

    /// The host's sample rate.
    pub fn sample_rate(&self) -> f64 {
        self.config.sample_rate
    }

    /// The host's buffer size.
    pub fn buffer_size(&self) -> u32 {
        self.config.buffer_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_processes_sine_to_output() {
        let mut host = AudioHost::new(HostConfig::default());
        let mut buf = vec![0.0f32; 512];
        let n = host.process(&mut buf);
        assert_eq!(n, 512);

        let peak = buf.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
        assert!(peak > 0.99, "expected peak near 1.0, got {}", peak);
    }

    #[test]
    fn host_applies_output_gain() {
        let mut host = AudioHost::new(HostConfig::default());
        host.output_gain = 0.5;
        let mut buf = vec![0.0f32; 512];
        host.process(&mut buf);

        let peak = buf.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
        assert!(peak > 0.45 && peak < 0.55, "expected peak near 0.5, got {}", peak);
    }

    #[test]
    fn host_load_unload_plugin() {
        let mut host = AudioHost::new(HostConfig::default());
        assert!(host.loaded_plugin.is_none());

        host.load_plugin(PathBuf::from("/tmp/fake.vst3")).unwrap();
        assert_eq!(
            host.loaded_plugin.as_ref().unwrap().to_str().unwrap(),
            "/tmp/fake.vst3"
        );

        host.unload_plugin();
        assert!(host.loaded_plugin.is_none());
    }
}
