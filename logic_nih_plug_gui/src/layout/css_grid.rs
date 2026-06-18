//! CSS Grid layout system.
//!
//! This module provides a full CSS Grid-like layout engine with:
//! - Track sizing: `fr` units, fixed pixels, `auto`, `min-content`, `max-content`, `minmax()`
//! - Named grid areas
//! - Explicit grid item placement (row/column start/end, spanning)
//! - Row and column gaps
//! - Item alignment (`start`, `end`, `center`, `stretch`)
//!
//! # Examples
//!
//! ```
//! use logic_nih_plug_gui::layout::css_grid::{
//!     CssGrid, GridTrack, GridItem, GridPlacement, GridAlignment,
//! };
//!
//! // Create a 2-column, 1-row grid with a 10px gap
//! let mut grid = CssGrid::new();
//! grid.set_columns(vec![
//!     GridTrack::Fraction(1.0),
//!     GridTrack::Fraction(1.0),
//! ]);
//! grid.set_rows(vec![GridTrack::Fixed(100.0)]);
//! grid.set_column_gap(10.0);
//!
//! // Add two items — each takes one cell
//! grid.add_item(GridItem::new().with_placement(GridPlacement::cell(1, 1)));
//! grid.add_item(GridItem::new().with_placement(GridPlacement::cell(1, 2)));
//!
//! let rects = grid.layout(400.0, 100.0);
//! assert_eq!(rects.len(), 2);
//! // Item 0: x=0, width ≈ 195  (400-10)/2
//! // Item 1: x=205, width ≈ 195
//! ```

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Track sizing
// ---------------------------------------------------------------------------

/// A single grid track (row or column) size definition.
#[derive(Debug, Clone, PartialEq)]
pub enum GridTrack {
    /// Fixed size in pixels.
    Fixed(f32),
    /// Flexible fraction of remaining space (`1fr` = 1 fraction).
    Fraction(f32),
    /// Size determined by content (`auto` in CSS).
    Auto,
    /// Minimum content size.
    MinContent,
    /// Maximum content size.
    MaxContent,
    /// `minmax(min, max)` — clamped between two track sizes.
    MinMax(Box<GridTrack>, Box<GridTrack>),
}

impl GridTrack {
    /// Create a `fr` track: `GridTrack::Fraction(1.0)` for `1fr`.
    pub fn fr(value: f32) -> Self {
        GridTrack::Fraction(value)
    }

    /// Create a fixed pixel track.
    pub fn px(value: f32) -> Self {
        GridTrack::Fixed(value)
    }

    /// Create a `minmax()` track.
    pub fn minmax(min: GridTrack, max: GridTrack) -> Self {
        GridTrack::MinMax(Box::new(min), Box::new(max))
    }
}

// ---------------------------------------------------------------------------
// Grid placement
// ---------------------------------------------------------------------------

/// Where a grid item is placed.
#[derive(Debug, Clone, PartialEq)]
pub struct GridPlacement {
    /// Row start line (1-based).
    pub row_start: u32,
    /// Row end line (1-based, exclusive — like CSS `grid-row-end`).
    pub row_end: u32,
    /// Column start line (1-based).
    pub col_start: u32,
    /// Column end line (1-based, exclusive — like CSS `grid-column-end`).
    pub col_end: u32,
    /// Optional named area (for `grid-area` shorthand).
    pub area_name: Option<String>,
}

impl GridPlacement {
    /// Place item at a specific cell (row, col), each 1-based, spanning 1 cell.
    pub fn cell(row: u32, col: u32) -> Self {
        Self {
            row_start: row,
            row_end: row + 1,
            col_start: col,
            col_end: col + 1,
            area_name: None,
        }
    }

    /// Place item spanning multiple rows and columns.
    pub fn area(row_start: u32, row_end: u32, col_start: u32, col_end: u32) -> Self {
        Self {
            row_start,
            row_end,
            col_start,
            col_end,
            area_name: None,
        }
    }

    /// Place item using a named grid area.
    pub fn named(name: &str) -> Self {
        Self {
            row_start: 0,
            row_end: 0,
            col_start: 0,
            col_end: 0,
            area_name: Some(name.to_string()),
        }
    }

    /// Number of rows this item spans.
    pub fn row_span(&self) -> u32 {
        self.row_end.saturating_sub(self.row_start).max(1)
    }

    /// Number of columns this item spans.
    pub fn col_span(&self) -> u32 {
        self.col_end.saturating_sub(self.col_start).max(1)
    }
}

// ---------------------------------------------------------------------------
// Grid item alignment
// ---------------------------------------------------------------------------

/// How a grid item aligns within its grid area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridAlignment {
    /// Align to start of area.
    Start,
    /// Align to end of area.
    End,
    /// Center within area.
    Center,
    /// Stretch to fill area (default).
    Stretch,
}

impl Default for GridAlignment {
    fn default() -> Self {
        GridAlignment::Stretch
    }
}

// ---------------------------------------------------------------------------
// Grid item
// ---------------------------------------------------------------------------

/// A single item placed inside a CSS Grid.
#[derive(Debug, Clone)]
pub struct GridItem {
    /// Placement of this item.
    pub placement: GridPlacement,
    /// Row alignment override.
    pub row_align: GridAlignment,
    /// Column alignment override.
    pub col_align: GridAlignment,
    /// Explicit width (overrides track sizing if set).
    pub width: Option<f32>,
    /// Explicit height (overrides track sizing if set).
    pub height: Option<f32>,
}

impl GridItem {
    /// Create a new grid item with default (auto-placed, stretched) settings.
    pub fn new() -> Self {
        Self {
            placement: GridPlacement::cell(1, 1),
            row_align: GridAlignment::Stretch,
            col_align: GridAlignment::Stretch,
            width: None,
            height: None,
        }
    }

    /// Set the placement.
    pub fn with_placement(mut self, placement: GridPlacement) -> Self {
        self.placement = placement;
        self
    }

    /// Set row alignment.
    pub fn with_row_align(mut self, align: GridAlignment) -> Self {
        self.row_align = align;
        self
    }

    /// Set column alignment.
    pub fn with_col_align(mut self, align: GridAlignment) -> Self {
        self.col_align = align;
        self
    }

    /// Set explicit width.
    pub fn with_width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Set explicit height.
    pub fn with_height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }
}

impl Default for GridItem {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Named grid areas
// ---------------------------------------------------------------------------

/// A named grid area mapping an area name to its row/column extent.
#[derive(Debug, Clone)]
pub struct NamedArea {
    /// Area name.
    pub name: String,
    /// Row start line (1-based).
    pub row_start: u32,
    /// Row end line (1-based, exclusive).
    pub row_end: u32,
    /// Column start line (1-based).
    pub col_start: u32,
    /// Column end line (1-based, exclusive).
    pub col_end: u32,
}

// ---------------------------------------------------------------------------
// CSS Grid
// ---------------------------------------------------------------------------

/// A CSS Grid layout engine.
///
/// Defines column and row tracks, gap sizes, named areas, and a set of
/// items, then computes positioned rectangles for each item.
#[derive(Debug, Clone)]
pub struct CssGrid {
    /// Column track definitions.
    columns: Vec<GridTrack>,
    /// Row track definitions.
    rows: Vec<GridTrack>,
    /// Gap between columns (in pixels).
    column_gap: f32,
    /// Gap between rows (in pixels).
    row_gap: f32,
    /// Named grid areas.
    areas: Vec<NamedArea>,
    /// Items to lay out.
    items: Vec<GridItem>,
}

impl CssGrid {
    /// Create a new empty CSS Grid.
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
            column_gap: 0.0,
            row_gap: 0.0,
            areas: Vec::new(),
            items: Vec::new(),
        }
    }

    /// Set column track definitions.
    pub fn set_columns(&mut self, tracks: Vec<GridTrack>) {
        self.columns = tracks;
    }

    /// Set row track definitions.
    pub fn set_rows(&mut self, tracks: Vec<GridTrack>) {
        self.rows = tracks;
    }

    /// Set the gap between columns.
    pub fn set_column_gap(&mut self, gap: f32) {
        self.column_gap = gap;
    }

    /// Set the gap between rows.
    pub fn set_row_gap(&mut self, gap: f32) {
        self.row_gap = gap;
    }

    /// Set both row and column gaps to the same value.
    pub fn set_gap(&mut self, gap: f32) {
        self.column_gap = gap;
        self.row_gap = gap;
    }

    /// Add a named grid area.
    pub fn add_area(&mut self, area: NamedArea) {
        self.areas.push(area);
    }

    /// Add a named grid area by name and extent.
    pub fn add_area_simple(
        &mut self,
        name: &str,
        row_start: u32,
        row_end: u32,
        col_start: u32,
        col_end: u32,
    ) {
        self.areas.push(NamedArea {
            name: name.to_string(),
            row_start,
            row_end,
            col_start,
            col_end,
        });
    }

    /// Add a grid item.
    pub fn add_item(&mut self, item: GridItem) {
        self.items.push(item);
    }

    /// Number of column tracks.
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// Number of row tracks.
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Number of items.
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    /// Resolve named areas for items that use `GridPlacement::named`.
    fn resolve_named_placements(&mut self) {
        let area_map: HashMap<String, &NamedArea> = self
            .areas
            .iter()
            .map(|a| (a.name.clone(), a))
            .collect();

        for item in &mut self.items {
            if let Some(ref name) = item.placement.area_name {
                if let Some(area) = area_map.get(name) {
                    item.placement.row_start = area.row_start;
                    item.placement.row_end = area.row_end;
                    item.placement.col_start = area.col_start;
                    item.placement.col_end = area.col_end;
                    item.placement.area_name = None; // resolved
                }
            }
        }
    }

    /// Resolve a single track to a concrete pixel size given the available space
    /// and the total fraction weight.
    fn resolve_track(
        track: &GridTrack,
        available: f32,
        total_fr: f32,
        gap_count: u32,
        total_gap: f32,
    ) -> f32 {
        let space_for_tracks = available - total_gap;
        match track {
            GridTrack::Fixed(px) => *px,
            GridTrack::Fraction(fr) => {
                if total_fr > 0.0 {
                    (space_for_tracks * fr / total_fr).max(0.0)
                } else {
                    0.0
                }
            }
            GridTrack::Auto => 0.0, // auto tracks shrink to 0 without content info
            GridTrack::MinContent => 0.0,
            GridTrack::MaxContent => available,
            GridTrack::MinMax(min, max) => {
                // Resolve min with zero fr weight (absolute floor)
                let min_val = Self::resolve_track(min, available, 0.0, gap_count, total_gap);
                // Resolve max with actual fr weight (ceiling)
                let max_val = Self::resolve_track(max, available, total_fr, gap_count, total_gap);
                // Track takes the max value but is at least the min
                if max_val >= min_val { max_val } else { min_val }
            }
        }
    }

    /// Resolve all tracks to concrete pixel sizes.
    ///
    /// First pass: resolve non-`fr` tracks to get their absolute sizes.
    /// Second pass: distribute remaining space (after gaps and fixed tracks)
    /// proportionally to `fr` tracks.
    fn resolve_tracks(
        tracks: &[GridTrack],
        available: f32,
        gap: f32,
    ) -> Vec<f32> {
        let gap_count = tracks.len().saturating_sub(1) as u32;
        let total_gap = gap * gap_count as f32;

        let total_fr: f32 = tracks
            .iter()
            .filter_map(|t| {
                if let GridTrack::Fraction(f) = t {
                    Some(*f)
                } else {
                    None
                }
            })
            .sum();

        // First pass: resolve non-fr tracks to get their sizes
        let mut sizes: Vec<f32> = tracks
            .iter()
            .map(|t| {
                if matches!(t, GridTrack::Fraction(_)) {
                    0.0 // placeholder — resolved in second pass
                } else {
                    Self::resolve_track(t, available, total_fr, gap_count, total_gap)
                }
            })
            .collect();

        // Sum up non-fr sizes (these consume space from the available pool)
        let fixed_used: f32 = sizes.iter().sum();
        let fr_space = (available - total_gap - fixed_used).max(0.0);

        // Second pass: distribute fr_space to fr tracks
        for (i, track) in tracks.iter().enumerate() {
            if let GridTrack::Fraction(f) = track {
                sizes[i] = if total_fr > 0.0 {
                    fr_space * f / total_fr
                } else {
                    0.0
                };
            }
        }

        sizes
    }

    /// Compute positioned rectangles for all items within the given container size.
    ///
    /// Returns one `Rect` per item in the same order they were added.
    pub fn layout(&self, container_width: f32, container_height: f32) -> Vec<Rect> {
        if self.items.is_empty() {
            return Vec::new();
        }

        let mut grid = self.clone();
        grid.resolve_named_placements();

        let col_sizes = Self::resolve_tracks(&grid.columns, container_width, grid.column_gap);
        let row_sizes = Self::resolve_tracks(&grid.rows, container_height, grid.row_gap);

        // Precompute column start positions
        let mut col_offsets = Vec::with_capacity(col_sizes.len());
        let mut x = 0.0;
        for (i, &w) in col_sizes.iter().enumerate() {
            col_offsets.push(x);
            x += w;
            if i + 1 < col_sizes.len() {
                x += grid.column_gap;
            }
        }

        // Precompute row start positions
        let mut row_offsets = Vec::with_capacity(row_sizes.len());
        let mut y = 0.0;
        for (i, &h) in row_sizes.iter().enumerate() {
            row_offsets.push(y);
            y += h;
            if i + 1 < row_sizes.len() {
                y += grid.row_gap;
            }
        }

        grid.items
            .iter()
            .map(|item| {
                let p = &item.placement;
                let rs = (p.row_start as usize).saturating_sub(1).min(row_offsets.len());
                let re = (p.row_end as usize).saturating_sub(1).min(row_sizes.len());
                let cs = (p.col_start as usize).saturating_sub(1).min(col_offsets.len());
                let ce = (p.col_end as usize).saturating_sub(1).min(col_sizes.len());

                // Sum up widths/heights of spanned tracks
                let area_x = col_offsets.get(cs).copied().unwrap_or(0.0);
                let area_y = row_offsets.get(rs).copied().unwrap_or(0.0);

                let mut area_w: f32 = 0.0;
                for c in cs..ce.min(col_sizes.len()) {
                    area_w += col_sizes[c];
                    if c > cs {
                        area_w += grid.column_gap;
                    }
                }

                let mut area_h: f32 = 0.0;
                for r in rs..re.min(row_sizes.len()) {
                    area_h += row_sizes[r];
                    if r > rs {
                        area_h += grid.row_gap;
                    }
                }

                // Apply item size constraints
                let item_w = item.width.unwrap_or(area_w);
                let item_h = item.height.unwrap_or(area_h);

                // Apply alignment within area
                let x_offset = match item.col_align {
                    GridAlignment::Start => 0.0,
                    GridAlignment::End => (area_w - item_w).max(0.0),
                    GridAlignment::Center => ((area_w - item_w) / 2.0).max(0.0),
                    GridAlignment::Stretch => 0.0,
                };
                let y_offset = match item.row_align {
                    GridAlignment::Start => 0.0,
                    GridAlignment::End => (area_h - item_h).max(0.0),
                    GridAlignment::Center => ((area_h - item_h) / 2.0).max(0.0),
                    GridAlignment::Stretch => 0.0,
                };

                let final_w = if item.col_align == GridAlignment::Stretch {
                    area_w
                } else {
                    item_w.min(area_w)
                };
                let final_h = if item.row_align == GridAlignment::Stretch {
                    area_h
                } else {
                    item_h.min(area_h)
                };

                Rect::new(area_x + x_offset, area_y + y_offset, final_w, final_h)
            })
            .collect()
    }
}

impl Default for CssGrid {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Rect (local — mirrors flexbox::Rect but kept self-contained)
// ---------------------------------------------------------------------------

/// A positioned rectangle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    /// X coordinate.
    pub x: f32,
    /// Y coordinate.
    pub y: f32,
    /// Width.
    pub width: f32,
    /// Height.
    pub height: f32,
}

impl Rect {
    /// Create a new rectangle.
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Basic track resolution --

    #[test]
    fn fixed_tracks() {
        let mut grid = CssGrid::new();
        grid.set_columns(vec![GridTrack::px(100.0), GridTrack::px(200.0)]);
        grid.set_rows(vec![GridTrack::px(50.0)]);
        grid.add_item(GridItem::new().with_placement(GridPlacement::cell(1, 1)));
        grid.add_item(GridItem::new().with_placement(GridPlacement::cell(1, 2)));

        let rects = grid.layout(500.0, 100.0);
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0], Rect::new(0.0, 0.0, 100.0, 50.0));
        assert_eq!(rects[1], Rect::new(100.0, 0.0, 200.0, 50.0));
    }

    #[test]
    fn fr_tracks_share_space() {
        let mut grid = CssGrid::new();
        grid.set_columns(vec![GridTrack::fr(1.0), GridTrack::fr(1.0)]);
        grid.set_rows(vec![GridTrack::fr(1.0)]);
        grid.add_item(GridItem::new().with_placement(GridPlacement::cell(1, 1)));
        grid.add_item(GridItem::new().with_placement(GridPlacement::cell(1, 2)));

        let rects = grid.layout(400.0, 100.0);
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0].x, 0.0);
        assert_eq!(rects[0].width, 200.0);
        assert_eq!(rects[1].x, 200.0);
        assert_eq!(rects[1].width, 200.0);
    }

    #[test]
    fn fr_tracks_unequal() {
        let mut grid = CssGrid::new();
        grid.set_columns(vec![GridTrack::fr(1.0), GridTrack::fr(2.0)]);
        grid.set_rows(vec![GridTrack::fr(1.0)]);
        grid.add_item(GridItem::new().with_placement(GridPlacement::cell(1, 1)));
        grid.add_item(GridItem::new().with_placement(GridPlacement::cell(1, 2)));

        let rects = grid.layout(300.0, 100.0);
        assert_eq!(rects[0].width, 100.0); // 1/3 of 300
        assert_eq!(rects[1].width, 200.0); // 2/3 of 300
    }

    #[test]
    fn mixed_fixed_and_fr() {
        let mut grid = CssGrid::new();
        grid.set_columns(vec![GridTrack::px(100.0), GridTrack::fr(1.0)]);
        grid.set_rows(vec![GridTrack::fr(1.0)]);
        grid.add_item(GridItem::new().with_placement(GridPlacement::cell(1, 1)));
        grid.add_item(GridItem::new().with_placement(GridPlacement::cell(1, 2)));

        let rects = grid.layout(400.0, 100.0);
        assert_eq!(rects[0].width, 100.0);
        assert_eq!(rects[1].width, 300.0); // 400 - 100
    }

    // -- Gap handling --

    #[test]
    fn column_gap() {
        let mut grid = CssGrid::new();
        grid.set_columns(vec![GridTrack::fr(1.0), GridTrack::fr(1.0)]);
        grid.set_rows(vec![GridTrack::fr(1.0)]);
        grid.set_column_gap(20.0);
        grid.add_item(GridItem::new().with_placement(GridPlacement::cell(1, 1)));
        grid.add_item(GridItem::new().with_placement(GridPlacement::cell(1, 2)));

        let rects = grid.layout(400.0, 100.0);
        // Available: 400 - 20 = 380, each fr = 190
        assert_eq!(rects[0].x, 0.0);
        assert_eq!(rects[0].width, 190.0);
        assert_eq!(rects[1].x, 210.0); // 190 + 20
        assert_eq!(rects[1].width, 190.0);
    }

    #[test]
    fn row_gap() {
        let mut grid = CssGrid::new();
        grid.set_columns(vec![GridTrack::fr(1.0)]);
        grid.set_rows(vec![GridTrack::fr(1.0), GridTrack::fr(1.0)]);
        grid.set_row_gap(10.0);
        grid.add_item(GridItem::new().with_placement(GridPlacement::cell(1, 1)));
        grid.add_item(GridItem::new().with_placement(GridPlacement::cell(2, 1)));

        let rects = grid.layout(100.0, 210.0);
        // Available: 210 - 10 = 200, each fr = 100
        assert_eq!(rects[0].y, 0.0);
        assert_eq!(rects[0].height, 100.0);
        assert_eq!(rects[1].y, 110.0); // 100 + 10
        assert_eq!(rects[1].height, 100.0);
    }

    #[test]
    fn both_gaps() {
        let mut grid = CssGrid::new();
        grid.set_columns(vec![GridTrack::fr(1.0), GridTrack::fr(1.0)]);
        grid.set_rows(vec![GridTrack::fr(1.0), GridTrack::fr(1.0)]);
        grid.set_gap(10.0);

        for r in 1..=2 {
            for c in 1..=2 {
                grid.add_item(
                    GridItem::new().with_placement(GridPlacement::cell(r, c)),
                );
            }
        }

        let rects = grid.layout(210.0, 210.0);
        // Each cell: (210 - 10) / 2 = 100
        assert_eq!(rects[0], Rect::new(0.0, 0.0, 100.0, 100.0));
        assert_eq!(rects[1], Rect::new(110.0, 0.0, 100.0, 100.0));
        assert_eq!(rects[2], Rect::new(0.0, 110.0, 100.0, 100.0));
        assert_eq!(rects[3], Rect::new(110.0, 110.0, 100.0, 100.0));
    }

    // -- Spanning --

    #[test]
    fn column_span() {
        let mut grid = CssGrid::new();
        grid.set_columns(vec![
            GridTrack::fr(1.0),
            GridTrack::fr(1.0),
            GridTrack::fr(1.0),
        ]);
        grid.set_rows(vec![GridTrack::fr(1.0)]);
        // Item spans columns 1-3 (col_end=4 is exclusive)
        grid.add_item(
            GridItem::new().with_placement(GridPlacement::area(1, 2, 1, 4)),
        );
        grid.add_item(
            GridItem::new().with_placement(GridPlacement::cell(1, 1)),
        );

        let rects = grid.layout(300.0, 100.0);
        // Spanning item takes full width
        assert_eq!(rects[0].x, 0.0);
        assert_eq!(rects[0].width, 300.0);
    }

    #[test]
    fn row_span() {
        let mut grid = CssGrid::new();
        grid.set_columns(vec![GridTrack::fr(1.0), GridTrack::fr(1.0)]);
        grid.set_rows(vec![
            GridTrack::fr(1.0),
            GridTrack::fr(1.0),
            GridTrack::fr(1.0),
        ]);
        // Item spans rows 1-2 (row_end=3 exclusive)
        grid.add_item(
            GridItem::new().with_placement(GridPlacement::area(1, 3, 1, 2)),
        );

        let rects = grid.layout(200.0, 300.0);
        // Available: 300, each row = 100. Span = 2 rows + 0 gap = 200
        assert_eq!(rects[0].height, 200.0);
    }

    // -- Named areas --

    #[test]
    fn named_area() {
        let mut grid = CssGrid::new();
        grid.set_columns(vec![GridTrack::fr(1.0), GridTrack::fr(1.0)]);
        grid.set_rows(vec![GridTrack::fr(1.0), GridTrack::fr(1.0)]);
        grid.add_area_simple("header", 1, 2, 1, 3); // spans full width row 1

        grid.add_item(
            GridItem::new().with_placement(GridPlacement::named("header")),
        );
        grid.add_item(
            GridItem::new().with_placement(GridPlacement::cell(2, 1)),
        );
        grid.add_item(
            GridItem::new().with_placement(GridPlacement::cell(2, 2)),
        );

        let rects = grid.layout(200.0, 200.0);
        // Header spans full width
        assert_eq!(rects[0].x, 0.0);
        assert_eq!(rects[0].width, 200.0);
        assert_eq!(rects[0].height, 100.0);
    }

    // -- Item alignment --

    #[test]
    fn item_center_align() {
        let mut grid = CssGrid::new();
        grid.set_columns(vec![GridTrack::px(100.0)]);
        grid.set_rows(vec![GridTrack::px(100.0)]);
        grid.add_item(
            GridItem::new()
                .with_placement(GridPlacement::cell(1, 1))
                .with_width(40.0)
                .with_height(20.0)
                .with_row_align(GridAlignment::Center)
                .with_col_align(GridAlignment::Center),
        );

        let rects = grid.layout(100.0, 100.0);
        assert_eq!(
            rects[0],
            Rect::new(30.0, 40.0, 40.0, 20.0)
        );
    }

    #[test]
    fn item_start_align() {
        let mut grid = CssGrid::new();
        grid.set_columns(vec![GridTrack::px(100.0)]);
        grid.set_rows(vec![GridTrack::px(100.0)]);
        grid.add_item(
            GridItem::new()
                .with_placement(GridPlacement::cell(1, 1))
                .with_width(50.0)
                .with_height(30.0)
                .with_row_align(GridAlignment::Start)
                .with_col_align(GridAlignment::Start),
        );

        let rects = grid.layout(100.0, 100.0);
        assert_eq!(rects[0], Rect::new(0.0, 0.0, 50.0, 30.0));
    }

    #[test]
    fn item_end_align() {
        let mut grid = CssGrid::new();
        grid.set_columns(vec![GridTrack::px(100.0)]);
        grid.set_rows(vec![GridTrack::px(100.0)]);
        grid.add_item(
            GridItem::new()
                .with_placement(GridPlacement::cell(1, 1))
                .with_width(50.0)
                .with_height(30.0)
                .with_row_align(GridAlignment::End)
                .with_col_align(GridAlignment::End),
        );

        let rects = grid.layout(100.0, 100.0);
        assert_eq!(rects[0], Rect::new(50.0, 70.0, 50.0, 30.0));
    }

    // -- GridPlacement helpers --

    #[test]
    fn placement_cell_span() {
        let p = GridPlacement::cell(2, 3);
        assert_eq!(p.row_start, 2);
        assert_eq!(p.row_end, 3);
        assert_eq!(p.col_start, 3);
        assert_eq!(p.col_end, 4);
        assert_eq!(p.row_span(), 1);
        assert_eq!(p.col_span(), 1);
    }

    #[test]
    fn placement_area_span() {
        let p = GridPlacement::area(1, 3, 2, 5);
        assert_eq!(p.row_span(), 2);
        assert_eq!(p.col_span(), 3);
    }

    // -- Edge cases --

    #[test]
    fn empty_grid() {
        let grid = CssGrid::new();
        let rects = grid.layout(100.0, 100.0);
        assert!(rects.is_empty());
    }

    #[test]
    fn no_tracks_fills_zero() {
        let mut grid = CssGrid::new();
        grid.set_columns(vec![]);
        grid.set_rows(vec![GridTrack::fr(1.0)]);
        grid.add_item(GridItem::new().with_placement(GridPlacement::cell(1, 1)));

        let rects = grid.layout(400.0, 100.0);
        assert_eq!(rects.len(), 1);
        // No columns → width = 0
        assert_eq!(rects[0].width, 0.0);
    }

    #[test]
    fn minmax_track() {
        let mut grid = CssGrid::new();
        grid.set_columns(vec![GridTrack::minmax(
            GridTrack::px(50.0),
            GridTrack::px(150.0),
        )]);
        grid.set_rows(vec![GridTrack::fr(1.0)]);
        grid.add_item(GridItem::new().with_placement(GridPlacement::cell(1, 1)));

        let rects = grid.layout(200.0, 100.0);
        // minmax(50, 150): resolved to 150 (max side gets the fr share = 200, clamped to 150)
        assert_eq!(rects[0].width, 150.0);
    }

    #[test]
    fn minmax_constrains_to_min() {
        let mut grid = CssGrid::new();
        grid.set_columns(vec![GridTrack::minmax(
            GridTrack::px(200.0),
            GridTrack::px(100.0),
        )]);
        grid.set_rows(vec![GridTrack::fr(1.0)]);
        grid.add_item(GridItem::new().with_placement(GridPlacement::cell(1, 1)));

        let rects = grid.layout(200.0, 100.0);
        // min > max → minmax resolves to min=200, max=100 → clamp to [100,200] = 100
        assert_eq!(rects[0].width, 200.0);
    }

    // -- GridTrack builder --

    #[test]
    fn track_builders() {
        let t1 = GridTrack::fr(2.0);
        let t2 = GridTrack::px(100.0);
        let t3 = GridTrack::minmax(GridTrack::Auto, GridTrack::MaxContent);
        assert_eq!(t1, GridTrack::Fraction(2.0));
        assert_eq!(t2, GridTrack::Fixed(100.0));
        assert!(matches!(t3, GridTrack::MinMax(_, _)));
    }

    // -- Count accessors --

    #[test]
    fn count_accessors() {
        let mut grid = CssGrid::new();
        assert_eq!(grid.column_count(), 0);
        assert_eq!(grid.row_count(), 0);
        assert_eq!(grid.item_count(), 0);

        grid.set_columns(vec![GridTrack::fr(1.0), GridTrack::fr(1.0)]);
        grid.set_rows(vec![GridTrack::fr(1.0), GridTrack::fr(1.0), GridTrack::fr(1.0)]);
        assert_eq!(grid.column_count(), 2);
        assert_eq!(grid.row_count(), 3);

        grid.add_item(GridItem::new());
        grid.add_item(GridItem::new());
        assert_eq!(grid.item_count(), 2);
    }

    // -- Large real-world-ish layout --

    #[test]
    fn three_column_layout() {
        // | sidebar (200px) | main (1fr) | aside (1fr) |
        let mut grid = CssGrid::new();
        grid.set_columns(vec![
            GridTrack::px(200.0),
            GridTrack::fr(1.0),
            GridTrack::fr(1.0),
        ]);
        grid.set_rows(vec![GridTrack::fr(1.0)]);
        grid.set_column_gap(16.0);

        grid.add_item(
            GridItem::new().with_placement(GridPlacement::cell(1, 1)),
        );
        grid.add_item(
            GridItem::new().with_placement(GridPlacement::cell(1, 2)),
        );
        grid.add_item(
            GridItem::new().with_placement(GridPlacement::cell(1, 3)),
        );

        let rects = grid.layout(816.0, 600.0);
        // Available for tracks: 816 - 2*16 = 784. Fixed=200, remaining=584, each fr=292
        assert_eq!(rects[0], Rect::new(0.0, 0.0, 200.0, 600.0));
        assert_eq!(rects[1], Rect::new(216.0, 0.0, 292.0, 600.0));
        assert_eq!(rects[2], Rect::new(524.0, 0.0, 292.0, 600.0));
    }
}
