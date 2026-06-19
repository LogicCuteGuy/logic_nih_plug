//! Controls showcase: Slider, Knob, ToggleButton demos.

use super::DemoEntry;

/// All demos registered in the Controls category.
pub fn registered() -> Vec<DemoEntry> {
    vec![
        DemoEntry::new(
            "slider",
            "Slider",
            "A horizontal/vertical slider with smooth parameter binding",
        ),
        DemoEntry::new(
            "knob",
            "Knob",
            "A rotary knob with drag-to-set and value popup",
        ),
        DemoEntry::new(
            "toggle",
            "ToggleButton",
            "A boolean on/off toggle",
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_at_least_three_demos() {
        let demos = registered();
        assert!(demos.len() >= 3, "expected ≥3 demos, got {}", demos.len());
        assert!(demos.iter().any(|d| d.id == "slider"));
        assert!(demos.iter().any(|d| d.id == "knob"));
        assert!(demos.iter().any(|d| d.id == "toggle"));
    }
}