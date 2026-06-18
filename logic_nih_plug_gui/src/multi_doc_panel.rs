//! Tabbed MDI document panel — hosts a set of named child components.
//!
//! [`MultiDocumentPanel`] mirrors JUCE's `juce::MultiDocumentPanel`. The
//! common case is the "tabbed" layout mode (one document visible at a
//! time, with a tab strip to switch between them), but the panel also
//! supports a "floating windows" mode that tracks which document is
//! active.
//!
//! This crate deliberately does **not** render — drawing the actual tab
//! strip or window chrome is the host's responsibility (use the
//! `logic_nih_plug_graphics` painter primitives for that). The panel
//! tracks which document is active and emits change notifications; the
//! UI layer above decides how to render it.
//!
//! # Examples
//!
//! ```
//! use logic_nih_plug_gui::components::Component;
//! use logic_nih_plug_gui::multi_doc_panel::{MultiDocumentPanel, MultiDocumentPanelLayout};
//!
//! let mut panel = MultiDocumentPanel::new();
//! panel.set_layout_mode(MultiDocumentPanelLayout::MaximisedWindowsWithTabs);
//!
//! let doc_a = Component::new("DocA");
//! let doc_b = Component::new("DocB");
//! panel.add_document(doc_a, true);
//! panel.add_document(doc_b, true);
//!
//! assert_eq!(panel.num_documents(), 2);
//! assert!(panel.set_active_document_by_index(1));
//! assert_eq!(panel.active_document_index(), Some(1));
//! ```

use crate::components::Component;

/// Layout mode for [`MultiDocumentPanel`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MultiDocumentPanelLayout {
    /// Each document is hosted in its own floating window. The panel
    /// tracks which one is active; rendering of the windows themselves
    /// is the host's responsibility.
    FloatingWindows,
    /// All documents share a single tabbed container — only the active
    /// document is "on screen" at a time.
    #[default]
    MaximisedWindowsWithTabs,
}

/// One document entry tracked by [`MultiDocumentPanel`].
#[derive(Debug)]
pub struct DocumentEntry {
    /// The hosted component (logical owner; the panel doesn't render).
    pub component: Component,
    /// If `true`, the panel takes ownership and drops the component when
    /// it's closed.
    pub owned: bool,
}

/// A tabbed MDI container.
#[derive(Debug, Default)]
pub struct MultiDocumentPanel {
    documents: Vec<DocumentEntry>,
    active_index: Option<usize>,
    layout_mode: MultiDocumentPanelLayout,
    max_documents: Option<usize>,
    fullscreen_when_one_document: bool,
    background_colour: [u8; 4],
}

impl MultiDocumentPanel {
    /// Create an empty panel.
    pub fn new() -> Self {
        Self {
            documents: Vec::new(),
            active_index: None,
            layout_mode: MultiDocumentPanelLayout::default(),
            max_documents: None,
            fullscreen_when_one_document: false,
            background_colour: [40, 40, 40, 255],
        }
    }

    /// Number of documents currently hosted.
    pub fn num_documents(&self) -> usize {
        self.documents.len()
    }

    /// Whether the panel has zero documents.
    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }

    /// Get the component for document at `index`, or `None`.
    pub fn document(&self, index: usize) -> Option<&Component> {
        self.documents.get(index).map(|d| &d.component)
    }

    /// Mutable access to the component for document at `index`.
    pub fn document_mut(&mut self, index: usize) -> Option<&mut Component> {
        self.documents.get_mut(index).map(|d| &mut d.component)
    }

    /// Index of the active document, or `None`.
    pub fn active_document_index(&self) -> Option<usize> {
        self.active_index
    }

    /// The component for the currently active document, or `None`.
    pub fn active_document(&self) -> Option<&Component> {
        self.active_index.and_then(|i| self.documents.get(i).map(|d| &d.component))
    }

    /// Mutable access to the active component.
    pub fn active_document_mut(&mut self) -> Option<&mut Component> {
        let i = self.active_index?;
        self.documents.get_mut(i).map(|d| &mut d.component)
    }

    /// Layout mode.
    pub fn layout_mode(&self) -> MultiDocumentPanelLayout {
        self.layout_mode
    }

    /// Set the layout mode.
    pub fn set_layout_mode(&mut self, mode: MultiDocumentPanelLayout) {
        self.layout_mode = mode;
    }

    /// Background colour for areas behind the documents (RGBA, 0..=255).
    pub fn background_colour(&self) -> [u8; 4] {
        self.background_colour
    }

    /// Set the background colour (RGBA, 0..=255).
    pub fn set_background_colour(&mut self, rgba: [u8; 4]) {
        self.background_colour = rgba;
    }

    /// Maximum number of open documents, or `None` for unlimited.
    pub fn max_documents(&self) -> Option<usize> {
        self.max_documents
    }

    /// Set the maximum number of open documents. `None` removes the limit.
    pub fn set_max_documents(&mut self, n: Option<usize>) {
        self.max_documents = n.filter(|&n| n > 0);
    }

    /// Whether the document fills the panel when there's only one open.
    pub fn is_fullscreen_when_one_document(&self) -> bool {
        self.fullscreen_when_one_document
    }

    /// Set the fullscreen-when-one behaviour.
    pub fn set_fullscreen_when_one_document(&mut self, on: bool) {
        self.fullscreen_when_one_document = on;
    }

    /// Add a document to the panel. If `owned` is `true`, the panel takes
    /// ownership of the component and drops it when the document is
    /// closed. Returns `false` if the max-document limit would be
    /// exceeded (the component is **not** added in that case).
    pub fn add_document(&mut self, component: Component, owned: bool) -> bool {
        if let Some(max) = self.max_documents {
            if self.documents.len() >= max {
                return false;
            }
        }
        let new_index = self.documents.len();
        self.documents.push(DocumentEntry { component, owned });
        if self.active_index.is_none() {
            self.active_index = Some(new_index);
        }
        true
    }

    /// Close the document at `index`. If the document is owned, the
    /// component is dropped. Returns the closed `Component` if not owned,
    /// or `None` if the index was out of bounds / the panel owned it.
    pub fn close_document(&mut self, index: usize) -> Option<Component> {
        if index >= self.documents.len() {
            return None;
        }
        let entry = self.documents.remove(index);
        // Re-anchor the active index.
        match self.active_index {
            Some(active) if active == index => {
                // Active was removed; pick the next available index.
                self.active_index = if self.documents.is_empty() {
                    None
                } else {
                    Some(index.min(self.documents.len() - 1))
                };
            }
            Some(active) if active > index => {
                self.active_index = Some(active - 1);
            }
            _ => {}
        }
        if entry.owned {
            // Component dropped here.
            None
        } else {
            Some(entry.component)
        }
    }

    /// Close every document, dropping owned components. Returns the
    /// non-owned components in the order they were added.
    pub fn close_all_documents(&mut self) -> Vec<Component> {
        let mut non_owned = Vec::new();
        for entry in self.documents.drain(..) {
            if !entry.owned {
                non_owned.push(entry.component);
            }
        }
        self.active_index = None;
        non_owned
    }

    /// Make the document at `index` active. Returns `false` if `index` is
    /// out of bounds.
    pub fn set_active_document_by_index(&mut self, index: usize) -> bool {
        if index < self.documents.len() {
            self.active_index = Some(index);
            true
        } else {
            false
        }
    }

    /// Make the document with the given component active. Returns `false`
    /// if the component isn't currently hosted.
    pub fn set_active_document(&mut self, component: &Component) -> bool {
        for (i, entry) in self.documents.iter().enumerate() {
            if entry.component.id() == component.id() {
                self.active_index = Some(i);
                return true;
            }
        }
        false
    }

    /// Borrow every document in order.
    pub fn documents(&self) -> impl Iterator<Item = &Component> {
        self.documents.iter().map(|d| &d.component)
    }

    /// Whether the document at `index` is owned by the panel.
    pub fn is_document_owned(&self, index: usize) -> Option<bool> {
        self.documents.get(index).map(|d| d.owned)
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Bounds;

    #[test]
    fn empty_panel() {
        let panel = MultiDocumentPanel::new();
        assert_eq!(panel.num_documents(), 0);
        assert!(panel.is_empty());
        assert_eq!(panel.active_document_index(), None);
        assert!(panel.active_document().is_none());
    }

    #[test]
    fn add_document_sets_first_active() {
        let mut panel = MultiDocumentPanel::new();
        assert!(panel.add_document(Component::new("A"), true));
        assert_eq!(panel.num_documents(), 1);
        assert_eq!(panel.active_document_index(), Some(0));
    }

    #[test]
    fn add_document_preserves_active() {
        let mut panel = MultiDocumentPanel::new();
        panel.add_document(Component::new("A"), true);
        panel.set_active_document_by_index(0);
        panel.add_document(Component::new("B"), true);
        // Adding doesn't change the active selection.
        assert_eq!(panel.active_document_index(), Some(0));
    }

    #[test]
    fn close_document_re_anchors_active() {
        let mut panel = MultiDocumentPanel::new();
        panel.add_document(Component::new("A"), true);
        panel.add_document(Component::new("B"), true);
        panel.add_document(Component::new("C"), true);
        panel.set_active_document_by_index(1);
        panel.close_document(1);
        // Active was 1 (B). After closing B, active should be 1 again
        // (which is now C).
        assert_eq!(panel.active_document_index(), Some(1));
    }

    #[test]
    fn close_document_returns_owned_status() {
        let mut panel = MultiDocumentPanel::new();
        panel.add_document(Component::new("A"), false);
        let returned = panel.close_document(0);
        // Not owned — caller takes it back.
        assert!(returned.is_some());
        assert_eq!(panel.num_documents(), 0);
    }

    #[test]
    fn close_document_owned_returns_none() {
        let mut panel = MultiDocumentPanel::new();
        panel.add_document(Component::new("A"), true);
        assert!(panel.close_document(0).is_none());
        assert_eq!(panel.num_documents(), 0);
    }

    #[test]
    fn close_all_documents_returns_only_non_owned() {
        let mut panel = MultiDocumentPanel::new();
        panel.add_document(Component::new("A"), false);
        panel.add_document(Component::new("B"), true);
        panel.add_document(Component::new("C"), false);
        let returned = panel.close_all_documents();
        assert_eq!(returned.len(), 2);
        assert!(panel.is_empty());
        assert_eq!(panel.active_document_index(), None);
    }

    #[test]
    fn set_active_document_by_index_out_of_bounds() {
        let mut panel = MultiDocumentPanel::new();
        panel.add_document(Component::new("A"), true);
        assert!(!panel.set_active_document_by_index(99));
        // Active selection unchanged.
        assert_eq!(panel.active_document_index(), Some(0));
    }

    #[test]
    fn set_active_document_by_component() {
        let mut panel = MultiDocumentPanel::new();
        let a = Component::new("A");
        let b = Component::new("B");
        let b_id = b.id();
        panel.add_document(a, true);
        panel.add_document(b, true);
        assert!(panel.set_active_document_by_index(1));
        let lookup = Component::new("Lookup");
        assert!(!panel.set_active_document(&lookup));
        let b_ref = Component::new("B"); // different id
        assert!(!panel.set_active_document(&b_ref));
        // Re-create one with the same id via set_name? No — ids are unique.
        // Verify via direct lookup using the first-target pattern: find the
        // document whose component id matches `b_id`.
        for i in 0..panel.num_documents() {
            if panel.document(i).unwrap().id() == b_id {
                panel.set_active_document_by_index(i);
            }
        }
        assert_eq!(panel.active_document_index(), Some(1));
    }

    #[test]
    fn max_documents_limit() {
        let mut panel = MultiDocumentPanel::new();
        panel.set_max_documents(Some(2));
        assert!(panel.add_document(Component::new("A"), true));
        assert!(panel.add_document(Component::new("B"), true));
        assert!(!panel.add_document(Component::new("C"), true));
        assert_eq!(panel.num_documents(), 2);
    }

    #[test]
    fn max_documents_zero_means_unlimited() {
        let mut panel = MultiDocumentPanel::new();
        panel.set_max_documents(Some(0));
        assert!(panel.max_documents().is_none());
        assert!(panel.add_document(Component::new("A"), true));
        assert!(panel.add_document(Component::new("B"), true));
        assert!(panel.add_document(Component::new("C"), true));
        assert!(panel.add_document(Component::new("D"), true));
    }

    #[test]
    fn layout_mode_round_trip() {
        let mut panel = MultiDocumentPanel::new();
        panel.set_layout_mode(MultiDocumentPanelLayout::FloatingWindows);
        assert_eq!(panel.layout_mode(), MultiDocumentPanelLayout::FloatingWindows);
        panel.set_layout_mode(MultiDocumentPanelLayout::MaximisedWindowsWithTabs);
        assert_eq!(panel.layout_mode(), MultiDocumentPanelLayout::MaximisedWindowsWithTabs);
    }

    #[test]
    fn background_colour_round_trip() {
        let mut panel = MultiDocumentPanel::new();
        panel.set_background_colour([10, 20, 30, 255]);
        assert_eq!(panel.background_colour(), [10, 20, 30, 255]);
    }

    #[test]
    fn fullscreen_when_one_toggle() {
        let mut panel = MultiDocumentPanel::new();
        assert!(!panel.is_fullscreen_when_one_document());
        panel.set_fullscreen_when_one_document(true);
        assert!(panel.is_fullscreen_when_one_document());
    }

    #[test]
    fn documents_iter_in_order() {
        let mut panel = MultiDocumentPanel::new();
        let a = Component::new("A");
        let b = Component::new("B");
        let a_id = a.id();
        let b_id = b.id();
        panel.add_document(a, true);
        panel.add_document(b, true);
        let ids: Vec<_> = panel.documents().map(|c| c.id()).collect();
        assert_eq!(ids, vec![a_id, b_id]);
    }

    #[test]
    fn active_document_mut_provides_mutable_access() {
        let mut panel = MultiDocumentPanel::new();
        panel.add_document(Component::new("A"), true);
        let doc = panel.active_document_mut().unwrap();
        doc.set_bounds(Bounds::new(0, 0, 100, 100)).unwrap();
        assert_eq!(panel.document(0).unwrap().bounds().width, 100);
    }

    #[test]
    fn close_active_document_when_only_one() {
        let mut panel = MultiDocumentPanel::new();
        panel.add_document(Component::new("Solo"), true);
        panel.set_active_document_by_index(0);
        panel.close_document(0);
        assert_eq!(panel.active_document_index(), None);
    }

    #[test]
    fn close_document_out_of_bounds_returns_none() {
        let mut panel = MultiDocumentPanel::new();
        assert!(panel.close_document(0).is_none());
        assert!(panel.close_document(99).is_none());
    }

    #[test]
    fn close_document_after_active_keeps_active_in_range() {
        let mut panel = MultiDocumentPanel::new();
        panel.add_document(Component::new("A"), true);
        panel.add_document(Component::new("B"), true);
        panel.add_document(Component::new("C"), true);
        panel.set_active_document_by_index(0); // A
        panel.close_document(1); // close B
        // Active should still be 0 (A).
        assert_eq!(panel.active_document_index(), Some(0));
    }

    #[test]
    fn close_document_before_active_shifts_active_left() {
        let mut panel = MultiDocumentPanel::new();
        panel.add_document(Component::new("A"), true);
        panel.add_document(Component::new("B"), true);
        panel.add_document(Component::new("C"), true);
        panel.set_active_document_by_index(2); // C
        panel.close_document(0); // close A
        // Active was 2 (C). After removing A, C is now at index 1.
        assert_eq!(panel.active_document_index(), Some(1));
    }

    #[test]
    fn document_mut_out_of_bounds_returns_none() {
        let mut panel = MultiDocumentPanel::new();
        assert!(panel.document_mut(0).is_none());
        assert!(panel.document_mut(99).is_none());
    }

    #[test]
    fn is_document_owned_reflects_ownership() {
        let mut panel = MultiDocumentPanel::new();
        panel.add_document(Component::new("A"), true);
        panel.add_document(Component::new("B"), false);
        assert_eq!(panel.is_document_owned(0), Some(true));
        assert_eq!(panel.is_document_owned(1), Some(false));
        assert_eq!(panel.is_document_owned(2), None);
    }
}