//! Graphics showcase: Painter gradient + Path stroke demo.

use super::DemoEntry;

/// All demos registered in the Graphics category.
pub fn registered() -> Vec<DemoEntry> {
    vec![
        DemoEntry::new(
            "painter_gradient",
            "Painter gradient",
            "A linear gradient drawn with logic_nih_plug_graphics::Painter",
        ),
        DemoEntry::new(
            "path_stroke",
            "Path stroke",
            "An open path stroked with variable width",
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_at_least_one_demo() {
        let demos = registered();
        assert!(!demos.is_empty(), "expected ≥1 demo, got 0");
        assert!(demos.iter().any(|d| d.id == "painter_gradient"));
    }
}