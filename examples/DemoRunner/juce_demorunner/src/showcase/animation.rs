//! Animation showcase: eased knob + waveform morph demos.

use super::DemoEntry;
use logic_nih_plug_animation::easing::ease_in_out_quad;

/// All demos registered in the Animation category.
pub fn registered() -> Vec<DemoEntry> {
    vec![
        DemoEntry::new(
            "eased_knob",
            "Eased knob",
            "A knob whose value is interpolated with ease_in_out_quad",
        ),
        DemoEntry::new(
            "waveform_morph",
            "Waveform morph",
            "A waveform that morphs between sine/saw/square with easing",
        ),
    ]
}

/// Apply `ease_in_out_quad` to a 0..1 progress value. Used by the
/// `eased_knob` and `waveform_morph` demos.
pub fn ease_progress(progress: f32) -> f32 {
    ease_in_out_quad(progress.clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_at_least_one_demo() {
        let demos = registered();
        assert!(!demos.is_empty(), "expected ≥1 demo, got 0");
        assert!(demos.iter().any(|d| d.id == "eased_knob"));
    }

    #[test]
    fn ease_progress_endpoints_match() {
        assert!((ease_progress(0.0) - 0.0).abs() < 1e-6);
        assert!((ease_progress(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn ease_progress_is_monotonic() {
        // Sample at 11 points; each successive value should be ≥ the
        // previous (ease_in_out_quad is strictly monotonic on [0,1]).
        let mut prev = ease_progress(0.0);
        for i in 1..=10 {
            let v = ease_progress(i as f32 / 10.0);
            assert!(v >= prev, "ease_progress not monotonic at {}: {} < {}", i, v, prev);
            prev = v;
        }
    }
}