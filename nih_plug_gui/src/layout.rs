//! Layout management for GUI components.
//!
//! This module provides layout managers and constraints for positioning
//! and sizing components automatically.
//!
//! ## Layout Managers
//!
//! Layout managers automatically position and size child components according
//! to specific rules:
//!
//! - **FlexLayout**: Flexible box layout (horizontal or vertical)
//! - **FlexBox**: CSS FlexBox-like layout with comprehensive features
//! - **GridLayout**: Grid-based layout with rows and columns
//! - **AbsoluteLayout**: Manual positioning with constraints
//!
//! ## Constraints
//!
//! Constraints define how components should be sized and positioned:
//!
//! - Minimum/maximum size constraints
//! - Aspect ratio constraints
//! - Relative positioning constraints
//!
//! ## Examples
//!
//! ```
//! use nih_plug_gui::layout::{FlexLayout, FlexDirection};
//! use nih_plug_gui::components::{Component, Bounds};
//!
//! // Create a horizontal flex layout
//! let mut layout = FlexLayout::new(FlexDirection::Horizontal);
//! layout.set_spacing(10);
//! layout.set_padding(5, 5, 5, 5);
//!
//! // Create components
//! let mut parent = Component::new("parent");
//! parent.set_bounds(Bounds::new(0, 0, 400, 100)).unwrap();
//!
//! let mut child1 = Component::new("child1");
//! let mut child2 = Component::new("child2");
//!
//! parent.add_child(child1.clone()).unwrap();
//! parent.add_child(child2.clone()).unwrap();
//!
//! // Apply layout
//! layout.apply(&mut parent).unwrap();
//! ```

pub mod flexbox;

use crate::components::{Bounds, Component};
use crate::error::{GuiError, Result};

pub use flexbox::{
    AlignContent, AlignItems, AlignSelf, FlexBox, FlexDirection as FlexBoxDirection,
    FlexItem, FlexWrap, JustifyContent, Margin, Rect,
};

/// Size constraint for a component.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SizeConstraint {
    /// Minimum width (None = no constraint)
    pub min_width: Option<u32>,
    /// Maximum width (None = no constraint)
    pub max_width: Option<u32>,
    /// Minimum height (None = no constraint)
    pub min_height: Option<u32>,
    /// Maximum height (None = no constraint)
    pub max_height: Option<u32>,
    /// Preferred width (None = use available space)
    pub preferred_width: Option<u32>,
    /// Preferred height (None = use available space)
    pub preferred_height: Option<u32>,
}

impl SizeConstraint {
    /// Create a new size constraint with no restrictions.
    pub fn new() -> Self {
        Self {
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
            preferred_width: None,
            preferred_height: None,
        }
    }

    /// Set minimum width.
    pub fn with_min_width(mut self, width: u32) -> Self {
        self.min_width = Some(width);
        self
    }

    /// Set maximum width.
    pub fn with_max_width(mut self, width: u32) -> Self {
        self.max_width = Some(width);
        self
    }

    /// Set minimum height.
    pub fn with_min_height(mut self, height: u32) -> Self {
        self.min_height = Some(height);
        self
    }

    /// Set maximum height.
    pub fn with_max_height(mut self, height: u32) -> Self {
        self.max_height = Some(height);
        self
    }

    /// Set preferred width.
    pub fn with_preferred_width(mut self, width: u32) -> Self {
        self.preferred_width = Some(width);
        self
    }

    /// Set preferred height.
    pub fn with_preferred_height(mut self, height: u32) -> Self {
        self.preferred_height = Some(height);
        self
    }

    /// Set fixed size (min = max = preferred).
    pub fn with_fixed_size(mut self, width: u32, height: u32) -> Self {
        self.min_width = Some(width);
        self.max_width = Some(width);
        self.preferred_width = Some(width);
        self.min_height = Some(height);
        self.max_height = Some(height);
        self.preferred_height = Some(height);
        self
    }

    /// Apply constraints to a size, returning the constrained size.
    pub fn constrain(&self, width: u32, height: u32) -> (u32, u32) {
        let mut w = width;
        let mut h = height;

        // Apply width constraints
        if let Some(min_w) = self.min_width {
            w = w.max(min_w);
        }
        if let Some(max_w) = self.max_width {
            w = w.min(max_w);
        }

        // Apply height constraints
        if let Some(min_h) = self.min_height {
            h = h.max(min_h);
        }
        if let Some(max_h) = self.max_height {
            h = h.min(max_h);
        }

        (w, h)
    }

    /// Get the preferred size, or None if no preference.
    pub fn preferred_size(&self) -> Option<(u32, u32)> {
        match (self.preferred_width, self.preferred_height) {
            (Some(w), Some(h)) => Some((w, h)),
            _ => None,
        }
    }
}

impl Default for SizeConstraint {
    fn default() -> Self {
        Self::new()
    }
}

/// Flex layout direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexDirection {
    /// Horizontal layout (left to right)
    Horizontal,
    /// Vertical layout (top to bottom)
    Vertical,
}

/// Flex layout alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexAlign {
    /// Align to start (left/top)
    Start,
    /// Align to center
    Center,
    /// Align to end (right/bottom)
    End,
    /// Stretch to fill
    Stretch,
}

/// Flexible box layout manager.
///
/// Arranges children in a row or column with configurable spacing and alignment.
#[derive(Debug, Clone)]
pub struct FlexLayout {
    direction: FlexDirection,
    spacing: u32,
    padding_left: u32,
    padding_right: u32,
    padding_top: u32,
    padding_bottom: u32,
    align: FlexAlign,
}

impl FlexLayout {
    /// Create a new flex layout with the given direction.
    pub fn new(direction: FlexDirection) -> Self {
        Self {
            direction,
            spacing: 0,
            padding_left: 0,
            padding_right: 0,
            padding_top: 0,
            padding_bottom: 0,
            align: FlexAlign::Start,
        }
    }

    /// Set spacing between children.
    pub fn set_spacing(&mut self, spacing: u32) {
        self.spacing = spacing;
    }

    /// Set padding on all sides.
    pub fn set_padding(&mut self, left: u32, right: u32, top: u32, bottom: u32) {
        self.padding_left = left;
        self.padding_right = right;
        self.padding_top = top;
        self.padding_bottom = bottom;
    }

    /// Set alignment.
    pub fn set_align(&mut self, align: FlexAlign) {
        self.align = align;
    }

    /// Apply the layout to a component and its children.
    pub fn apply(&self, component: &mut Component) -> Result<()> {
        let bounds = component.bounds();
        let child_count = component.child_count();

        if child_count == 0 {
            return Ok(());
        }

        // Calculate available space
        let available_width = bounds
            .width
            .saturating_sub(self.padding_left + self.padding_right);
        let available_height = bounds
            .height
            .saturating_sub(self.padding_top + self.padding_bottom);

        match self.direction {
            FlexDirection::Horizontal => {
                self.layout_horizontal(component, available_width, available_height)
            }
            FlexDirection::Vertical => {
                self.layout_vertical(component, available_width, available_height)
            }
        }
    }

    fn layout_horizontal(
        &self,
        component: &mut Component,
        available_width: u32,
        available_height: u32,
    ) -> Result<()> {
        let child_count = component.child_count();
        let total_spacing = self.spacing.saturating_mul(child_count.saturating_sub(1) as u32);
        let available_for_children = available_width.saturating_sub(total_spacing);

        // Calculate child width (ensure at least 1)
        let child_width = (available_for_children / child_count.max(1) as u32).max(1);
        // Ensure height is at least 1
        let available_height = available_height.max(1);

        let mut x = self.padding_left as i32;
        let y = self.padding_top as i32;

        for i in 0..child_count {
            if let Some(mut child) = component.child(i) {
                let height = match self.align {
                    FlexAlign::Stretch => available_height,
                    _ => {
                        let child_height = child.bounds().height;
                        if child_height == 0 {
                            available_height
                        } else {
                            child_height.min(available_height)
                        }
                    }
                };

                let y_offset = match self.align {
                    FlexAlign::Start => 0,
                    FlexAlign::Center => (available_height.saturating_sub(height) / 2) as i32,
                    FlexAlign::End => available_height.saturating_sub(height) as i32,
                    FlexAlign::Stretch => 0,
                };

                child.set_bounds(Bounds::new(x, y + y_offset, child_width, height))?;
                x += child_width as i32 + self.spacing as i32;
            }
        }

        Ok(())
    }

    fn layout_vertical(
        &self,
        component: &mut Component,
        available_width: u32,
        available_height: u32,
    ) -> Result<()> {
        let child_count = component.child_count();
        let total_spacing = self.spacing.saturating_mul(child_count.saturating_sub(1) as u32);
        let available_for_children = available_height.saturating_sub(total_spacing);

        // Calculate child height (ensure at least 1)
        let child_height = (available_for_children / child_count.max(1) as u32).max(1);
        // Ensure width is at least 1
        let available_width = available_width.max(1);

        let x = self.padding_left as i32;
        let mut y = self.padding_top as i32;

        for i in 0..child_count {
            if let Some(mut child) = component.child(i) {
                let width = match self.align {
                    FlexAlign::Stretch => available_width,
                    _ => {
                        let child_width = child.bounds().width;
                        if child_width == 0 {
                            available_width
                        } else {
                            child_width.min(available_width)
                        }
                    }
                };

                let x_offset = match self.align {
                    FlexAlign::Start => 0,
                    FlexAlign::Center => (available_width.saturating_sub(width) / 2) as i32,
                    FlexAlign::End => available_width.saturating_sub(width) as i32,
                    FlexAlign::Stretch => 0,
                };

                child.set_bounds(Bounds::new(x + x_offset, y, width, child_height))?;
                y += child_height as i32 + self.spacing as i32;
            }
        }

        Ok(())
    }
}

/// Grid layout manager.
///
/// Arranges children in a grid with configurable rows and columns.
#[derive(Debug, Clone)]
pub struct GridLayout {
    rows: usize,
    columns: usize,
    spacing: u32,
    padding_left: u32,
    padding_right: u32,
    padding_top: u32,
    padding_bottom: u32,
}

impl GridLayout {
    /// Create a new grid layout with the given number of rows and columns.
    pub fn new(rows: usize, columns: usize) -> Result<Self> {
        if rows == 0 || columns == 0 {
            return Err(GuiError::InvalidLayout(
                "Grid must have at least 1 row and 1 column".to_string(),
            ));
        }

        Ok(Self {
            rows,
            columns,
            spacing: 0,
            padding_left: 0,
            padding_right: 0,
            padding_top: 0,
            padding_bottom: 0,
        })
    }

    /// Set spacing between cells.
    pub fn set_spacing(&mut self, spacing: u32) {
        self.spacing = spacing;
    }

    /// Set padding on all sides.
    pub fn set_padding(&mut self, left: u32, right: u32, top: u32, bottom: u32) {
        self.padding_left = left;
        self.padding_right = right;
        self.padding_top = top;
        self.padding_bottom = bottom;
    }

    /// Apply the layout to a component and its children.
    pub fn apply(&self, component: &mut Component) -> Result<()> {
        let bounds = component.bounds();
        let child_count = component.child_count();

        if child_count == 0 {
            return Ok(());
        }

        // Calculate available space
        let available_width = bounds
            .width
            .saturating_sub(self.padding_left + self.padding_right);
        let available_height = bounds
            .height
            .saturating_sub(self.padding_top + self.padding_bottom);

        // Calculate cell size
        let h_spacing = self.spacing.saturating_mul(self.columns.saturating_sub(1) as u32);
        let v_spacing = self.spacing.saturating_mul(self.rows.saturating_sub(1) as u32);

        let cell_width = available_width.saturating_sub(h_spacing) / self.columns.max(1) as u32;
        let cell_height = available_height.saturating_sub(v_spacing) / self.rows.max(1) as u32;

        // Position children
        for i in 0..child_count.min(self.rows * self.columns) {
            if let Some(mut child) = component.child(i) {
                let row = i / self.columns;
                let col = i % self.columns;

                let x = self.padding_left as i32
                    + (col as u32 * (cell_width + self.spacing)) as i32;
                let y = self.padding_top as i32
                    + (row as u32 * (cell_height + self.spacing)) as i32;

                child.set_bounds(Bounds::new(x, y, cell_width, cell_height))?;
            }
        }

        Ok(())
    }
}

/// Absolute layout manager.
///
/// Allows manual positioning of children with optional constraints.
#[derive(Debug, Clone)]
pub struct AbsoluteLayout {
    constraints: Vec<(usize, SizeConstraint)>,
}

impl AbsoluteLayout {
    /// Create a new absolute layout.
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
        }
    }

    /// Add a constraint for a child at the given index.
    pub fn add_constraint(&mut self, child_index: usize, constraint: SizeConstraint) {
        self.constraints.push((child_index, constraint));
    }

    /// Apply the layout to a component and its children.
    ///
    /// This applies size constraints but does not change positions.
    pub fn apply(&self, component: &mut Component) -> Result<()> {
        for (child_index, constraint) in &self.constraints {
            if let Some(mut child) = component.child(*child_index) {
                let bounds = child.bounds();
                let (width, height) = constraint.constrain(bounds.width, bounds.height);

                // Use preferred size if available
                let (final_width, final_height) = constraint
                    .preferred_size()
                    .unwrap_or((width, height));

                child.set_bounds(Bounds::new(bounds.x, bounds.y, final_width, final_height))?;
            }
        }

        Ok(())
    }
}

impl Default for AbsoluteLayout {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_constraint_creation() {
        let constraint = SizeConstraint::new();
        assert_eq!(constraint.min_width, None);
        assert_eq!(constraint.max_width, None);
        assert_eq!(constraint.min_height, None);
        assert_eq!(constraint.max_height, None);
    }

    #[test]
    fn test_size_constraint_builder() {
        let constraint = SizeConstraint::new()
            .with_min_width(100)
            .with_max_width(200)
            .with_min_height(50)
            .with_max_height(150);

        assert_eq!(constraint.min_width, Some(100));
        assert_eq!(constraint.max_width, Some(200));
        assert_eq!(constraint.min_height, Some(50));
        assert_eq!(constraint.max_height, Some(150));
    }

    #[test]
    fn test_size_constraint_fixed_size() {
        let constraint = SizeConstraint::new().with_fixed_size(100, 50);

        assert_eq!(constraint.min_width, Some(100));
        assert_eq!(constraint.max_width, Some(100));
        assert_eq!(constraint.preferred_width, Some(100));
        assert_eq!(constraint.min_height, Some(50));
        assert_eq!(constraint.max_height, Some(50));
        assert_eq!(constraint.preferred_height, Some(50));
    }

    #[test]
    fn test_size_constraint_constrain() {
        let constraint = SizeConstraint::new()
            .with_min_width(100)
            .with_max_width(200)
            .with_min_height(50)
            .with_max_height(150);

        // Within bounds
        assert_eq!(constraint.constrain(150, 100), (150, 100));

        // Below minimum
        assert_eq!(constraint.constrain(50, 25), (100, 50));

        // Above maximum
        assert_eq!(constraint.constrain(300, 200), (200, 150));
    }

    #[test]
    fn test_flex_layout_horizontal() {
        let mut layout = FlexLayout::new(FlexDirection::Horizontal);
        layout.set_spacing(10);

        let mut parent = Component::new("parent");
        parent.set_bounds(Bounds::new(0, 0, 400, 100)).unwrap();

        let child1 = Component::new("child1");
        let child2 = Component::new("child2");
        let child3 = Component::new("child3");

        parent.add_child(child1.clone()).unwrap();
        parent.add_child(child2.clone()).unwrap();
        parent.add_child(child3.clone()).unwrap();

        layout.apply(&mut parent).unwrap();

        // Check that children are laid out horizontally
        let c1 = parent.child(0).unwrap();
        let c2 = parent.child(1).unwrap();
        let c3 = parent.child(2).unwrap();

        assert_eq!(c1.bounds().x, 0);
        assert!(c2.bounds().x > c1.bounds().x);
        assert!(c3.bounds().x > c2.bounds().x);
    }

    #[test]
    fn test_flex_layout_vertical() {
        let mut layout = FlexLayout::new(FlexDirection::Vertical);
        layout.set_spacing(10);

        let mut parent = Component::new("parent");
        parent.set_bounds(Bounds::new(0, 0, 100, 400)).unwrap();

        let child1 = Component::new("child1");
        let child2 = Component::new("child2");
        let child3 = Component::new("child3");

        parent.add_child(child1.clone()).unwrap();
        parent.add_child(child2.clone()).unwrap();
        parent.add_child(child3.clone()).unwrap();

        layout.apply(&mut parent).unwrap();

        // Check that children are laid out vertically
        let c1 = parent.child(0).unwrap();
        let c2 = parent.child(1).unwrap();
        let c3 = parent.child(2).unwrap();

        assert_eq!(c1.bounds().y, 0);
        assert!(c2.bounds().y > c1.bounds().y);
        assert!(c3.bounds().y > c2.bounds().y);
    }

    #[test]
    fn test_flex_layout_with_padding() {
        let mut layout = FlexLayout::new(FlexDirection::Horizontal);
        layout.set_padding(10, 10, 5, 5);

        let mut parent = Component::new("parent");
        parent.set_bounds(Bounds::new(0, 0, 400, 100)).unwrap();

        let child = Component::new("child");
        parent.add_child(child.clone()).unwrap();

        layout.apply(&mut parent).unwrap();

        let c = parent.child(0).unwrap();
        assert_eq!(c.bounds().x, 10);
        assert_eq!(c.bounds().y, 5);
    }

    #[test]
    fn test_grid_layout() {
        let mut layout = GridLayout::new(2, 3).unwrap();
        layout.set_spacing(5);

        let mut parent = Component::new("parent");
        parent.set_bounds(Bounds::new(0, 0, 300, 200)).unwrap();

        for i in 0..6 {
            let child = Component::new(&format!("child{}", i));
            parent.add_child(child).unwrap();
        }

        layout.apply(&mut parent).unwrap();

        // Check that children are in a grid
        let c0 = parent.child(0).unwrap();
        let c1 = parent.child(1).unwrap();
        let c3 = parent.child(3).unwrap();

        // First row
        assert_eq!(c0.bounds().y, c1.bounds().y);
        // Second row is below first
        assert!(c3.bounds().y > c0.bounds().y);
    }

    #[test]
    fn test_grid_layout_invalid() {
        let result = GridLayout::new(0, 1);
        assert!(result.is_err());

        let result = GridLayout::new(1, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_absolute_layout() {
        let mut layout = AbsoluteLayout::new();
        layout.add_constraint(
            0,
            SizeConstraint::new().with_fixed_size(100, 50),
        );

        let mut parent = Component::new("parent");
        parent.set_bounds(Bounds::new(0, 0, 400, 300)).unwrap();

        let mut child = Component::new("child");
        child.set_bounds(Bounds::new(10, 10, 200, 100)).unwrap();
        parent.add_child(child.clone()).unwrap();

        layout.apply(&mut parent).unwrap();

        let c = parent.child(0).unwrap();
        // Position unchanged
        assert_eq!(c.bounds().x, 10);
        assert_eq!(c.bounds().y, 10);
        // Size constrained
        assert_eq!(c.bounds().width, 100);
        assert_eq!(c.bounds().height, 50);
    }

    #[test]
    fn test_flex_layout_empty_parent() {
        let layout = FlexLayout::new(FlexDirection::Horizontal);
        let mut parent = Component::new("parent");
        parent.set_bounds(Bounds::new(0, 0, 400, 100)).unwrap();

        // Should not error with no children
        let result = layout.apply(&mut parent);
        assert!(result.is_ok());
    }

    #[test]
    fn test_grid_layout_fewer_children_than_cells() {
        let layout = GridLayout::new(3, 3).unwrap();
        let mut parent = Component::new("parent");
        parent.set_bounds(Bounds::new(0, 0, 300, 300)).unwrap();

        // Only add 2 children (grid has 9 cells)
        parent.add_child(Component::new("child1")).unwrap();
        parent.add_child(Component::new("child2")).unwrap();

        // Should not error
        let result = layout.apply(&mut parent);
        assert!(result.is_ok());
    }
}
