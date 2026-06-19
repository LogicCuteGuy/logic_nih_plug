//! AudioViz showcase: LevelMeter + Oscilloscope + Spectrum demos.

use super::DemoEntry;
use logic_nih_plug_dsp::analysis::LevelMeter;

/// All demos registered in the Audio visualization category.
pub fn registered() -> Vec<DemoEntry> {
    vec![
        DemoEntry::new(
            "level_meter",
            "Level meter",
            "A peak/RMS level meter fed by a LevelMeter",
        ),
        DemoEntry::new(
            "oscilloscope",
            "Oscilloscope",
            "A scrolling oscilloscope driven by a sample buffer",
        ),
        DemoEntry::new(
            "spectrum",
            "Spectrum analyzer",
            "A magnitude spectrum (FFT bins) shown as a bar chart",
        ),
    ]
}

/// Run a single sample through a freshly-constructed [`LevelMeter`]
/// and return the resulting peak. Used by the `level_meter` demo to
/// show that a non-silent input produces a non-zero reading.
pub fn meter_level_for_sample(sample: f32) -> f32 {
    let mut meter = LevelMeter::new();
    meter.process(&[&[sample]]);
    meter.peak()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_at_least_one_demo() {
        let demos = registered();
        assert!(!demos.is_empty(), "expected ≥1 demo, got 0");
        assert!(demos.iter().any(|d| d.id == "level_meter"));
    }

    #[test]
    fn meter_level_for_silent_sample_is_zero() {
        assert_eq!(meter_level_for_sample(0.0), 0.0);
    }

    #[test]
    fn meter_level_for_loud_sample_is_positive() {
        let level = meter_level_for_sample(1.0);
        assert!(level > 0.0, "expected level > 0, got {}", level);
    }
}