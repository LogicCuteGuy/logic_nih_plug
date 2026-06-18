//! Windowing functions for FFT analysis and synthesis.
//!
//! A [`WindowingFunction`] can pre-compute a window of any size and
//! apply it to a buffer in-place. The Hann window is the default for
//! the phase vocoder because it satisfies the constant-overlap-add
//! (COLA) property at hop sizes of `fft_size / 2` and `fft_size / 4`.

/// Standard windowing functions used in spectral processing.
///
/// Each variant generates a window of the requested length. The
/// `kaiser` variant takes an additional `beta` parameter controlling
/// the sidelobe attenuation.
///
/// # Examples
///
/// ```
/// use logic_nih_plug_dsp::analysis::WindowingFunction;
///
/// let window = WindowingFunction::Hann.generate(1024);
/// assert_eq!(window.len(), 1024);
/// assert!((window[0]).abs() < 1e-6);  // Hann starts at 0
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowingFunction {
    /// Rectangular window (all ones). No attenuation.
    Rectangular,
    /// Triangular (Bartlett) window.
    Triangular,
    /// Hann window — the standard for phase vocoder OLA.
    Hann,
    /// Hamming window.
    Hamming,
    /// Blackman window.
    Blackman,
    /// Blackman-Harris window (4-term, −92 dB sidelobe).
    BlackmanHarris,
    /// Flat-top window (for amplitude accuracy in spectral analysis).
    FlatTop,
    /// Kaiser window with the given `beta` parameter.
    ///
    /// Higher `beta` gives narrower main lobe and lower sidelobes.
    /// Typical values: 3.0–10.0.
    Kaiser(f32),
}

impl WindowingFunction {
    /// Returns the window name as a human-readable string.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Rectangular => "Rectangular",
            Self::Triangular => "Triangular",
            Self::Hann => "Hann",
            Self::Hamming => "Hamming",
            Self::Blackman => "Blackman",
            Self::BlackmanHarris => "Blackman-Harris",
            Self::FlatTop => "Flat-top",
            Self::Kaiser(_) => "Kaiser",
        }
    }

    /// Generates a window of `size` samples, normalised so the
    /// maximum value is 1.0.
    ///
    /// The returned `Vec<f32>` has length `size`.
    pub fn generate(&self, size: usize) -> Vec<f32> {
        if size == 0 {
            return Vec::new();
        }

        let n = size as f32;
        let mut w = vec![0.0f32; size];

        match *self {
            Self::Rectangular => {
                w.fill(1.0);
            }
            Self::Triangular => {
                for i in 0..size {
                    w[i] = 1.0 - (2.0 * i as f32 / (n - 1.0) - 1.0).abs();
                }
            }
            Self::Hann => {
                for i in 0..size {
                    w[i] = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (n - 1.0)).cos());
                }
            }
            Self::Hamming => {
                for i in 0..size {
                    w[i] = 0.54 - 0.46 * (2.0 * std::f32::consts::PI * i as f32 / (n - 1.0)).cos();
                }
            }
            Self::Blackman => {
                for i in 0..size {
                    let t = i as f32 / (n - 1.0);
                    w[i] = 0.42 - 0.5 * (2.0 * std::f32::consts::PI * t).cos()
                        + 0.08 * (4.0 * std::f32::consts::PI * t).cos();
                }
            }
            Self::BlackmanHarris => {
                for i in 0..size {
                    let t = i as f32 / (n - 1.0);
                    w[i] = 0.35875 - 0.48829 * (2.0 * std::f32::consts::PI * t).cos()
                        + 0.14128 * (4.0 * std::f32::consts::PI * t).cos()
                        - 0.01168 * (6.0 * std::f32::consts::PI * t).cos();
                }
            }
            Self::FlatTop => {
                for i in 0..size {
                    let t = i as f32 / (n - 1.0);
                    w[i] = 0.21557895
                        - 0.41663158 * (2.0 * std::f32::consts::PI * t).cos()
                        + 0.277263158 * (4.0 * std::f32::consts::PI * t).cos()
                        - 0.083578947 * (6.0 * std::f32::consts::PI * t).cos()
                        + 0.006947368 * (8.0 * std::f32::consts::PI * t).cos();
                }
            }
            Self::Kaiser(beta) => {
                let alpha = (n - 1.0) / 2.0;
                let denom = bessel_i0(beta);
                for i in 0..size {
                    let x = beta * (1.0 - ((i as f32 - alpha) / alpha).powi(2)).sqrt();
                    w[i] = bessel_i0(x) / denom;
                }
            }
        }

        w
    }

    /// Multiplies `buffer` in-place with the window of the same
    /// length.
    pub fn apply(&self, buffer: &mut [f32]) {
        let w = self.generate(buffer.len());
        for (s, w_val) in buffer.iter_mut().zip(w.iter()) {
            *s *= w_val;
        }
    }
}

/// Modified Bessel function of the first kind, order zero (I₀).
///
/// Used only by the Kaiser window. Series expansion converges quickly
/// for typical `beta` values.
fn bessel_i0(x: f32) -> f32 {
    // I_0(x) = Σ_{k=0}^∞ (x/2)^{2k} / (k!)^2
    let half_x = x / 2.0;
    let mut sum = 1.0f32;
    let mut term = 1.0f32;
    for k in 1..=30 {
        term *= (half_x / k as f32) * (half_x / k as f32);
        sum += term;
        if term < 1e-12 {
            break;
        }
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rectangular_window() {
        let w = WindowingFunction::Rectangular.generate(64);
        assert_eq!(w.len(), 64);
        assert!(w.iter().all(|&v| (v - 1.0).abs() < 1e-6));
    }

    #[test]
    fn hann_window_ends_zero() {
        let w = WindowingFunction::Hann.generate(256);
        assert_eq!(w.len(), 256);
        // Hann starts and ends at 0
        assert!(w[0].abs() < 1e-6);
        assert!(w[255].abs() < 1e-6);
        // Peak at centre ≈ 1.0
        let peak = w.iter().cloned().fold(0.0f32, f32::max);
        assert!((peak - 1.0).abs() < 1e-4);
    }

    #[test]
    fn hamming_window_peak() {
        let w = WindowingFunction::Hamming.generate(128);
        // Hamming does NOT start at zero (≈ 0.08)
        assert!(w[0] > 0.05);
        let peak = w.iter().cloned().fold(0.0f32, f32::max);
        // Even-length windows don't sample at exact centre; allow 1e-3
        assert!((peak - 1.0).abs() < 1e-3, "hamming peak = {peak}");
    }

    #[test]
    fn blackman_harris_peak() {
        let w = WindowingFunction::BlackmanHarris.generate(512);
        let peak = w.iter().cloned().fold(0.0f32, f32::max);
        assert!((peak - 1.0).abs() < 1e-4);
    }

    #[test]
    fn kaiser_window_beta_3() {
        let w = WindowingFunction::Kaiser(3.0).generate(256);
        assert_eq!(w.len(), 256);
        let peak = w.iter().cloned().fold(0.0f32, f32::max);
        assert!((peak - 1.0).abs() < 1e-4);
    }

    #[test]
    fn zero_size() {
        let w = WindowingFunction::Hann.generate(0);
        assert!(w.is_empty());
    }

    #[test]
    fn apply_multiplies_in_place() {
        let window = WindowingFunction::Hann;
        let mut buf = vec![1.0f32; 64];
        window.apply(&mut buf);
        let expected = window.generate(64);
        for (b, e) in buf.iter().zip(expected.iter()) {
            assert!((b - e).abs() < 1e-6);
        }
    }

    #[test]
    fn triangular_symmetric() {
        let w = WindowingFunction::Triangular.generate(65);
        // Symmetric around centre
        for i in 0..32 {
            assert!((w[i] - w[64 - i]).abs() < 1e-6);
        }
    }

    #[test]
    fn flat_top_peak() {
        let w = WindowingFunction::FlatTop.generate(256);
        let peak = w.iter().cloned().fold(0.0f32, f32::max);
        // Even-length windows don't sample at exact centre; allow 1e-3
        assert!((peak - 1.0).abs() < 1e-3, "flat-top peak = {peak}");
    }
}
