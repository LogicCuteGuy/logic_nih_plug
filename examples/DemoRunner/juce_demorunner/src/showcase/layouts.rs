//! Layouts showcase: FlexBox, CssGrid, AbsoluteLayout demos.

use super::DemoEntry;

/// All demos registered in the Layouts category.
pub fn registered() -> Vec<DemoEntry> {
    vec![
        DemoEntry::new(
            "flexbox",
            "FlexBox",
            "Flexbox-style row + column layout with grow/shrink",
        ),
        DemoEntry::new(
            "cssgrid",
            "CssGrid",
            "CSS Grid with fr/px/auto/minmax() track sizing",
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_at_least_two_demos() {
        let demos = registered();
        assert!(demos.len() >= 2, "expected ≥2 demos, got {}", demos.len());
        assert!(demos.iter().any(|d| d.id == "flexbox"));
        assert!(demos.iter().any(|d| d.id == "cssgrid"));
    }
}