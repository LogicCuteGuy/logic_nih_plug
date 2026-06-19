//! Top-level navigation: category list → showcase page.

use crate::showcase::ShowcaseCategory;

/// The navigation state: which category is currently shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct NavState {
    /// Which category is currently selected. `None` = category list.
    pub selected: Option<ShowcaseCategory>,
}

impl NavState {
    /// Construct a fresh navigation state showing the category list.
    pub fn new() -> Self {
        Self::default()
    }

    /// Navigate to a category.
    pub fn select(&mut self, category: ShowcaseCategory) {
        self.selected = Some(category);
    }

    /// Go back to the category list.
    pub fn back(&mut self) {
        self.selected = None;
    }

    /// Whether the user is on a category page (vs the list).
    pub fn is_on_category(&self) -> bool {
        self.selected.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nav_state_starts_on_category_list() {
        let nav = NavState::new();
        assert!(!nav.is_on_category());
    }

    #[test]
    fn nav_state_selects_and_returns() {
        let mut nav = NavState::new();
        nav.select(ShowcaseCategory::Controls);
        assert!(nav.is_on_category());
        nav.back();
        assert!(!nav.is_on_category());
    }
}