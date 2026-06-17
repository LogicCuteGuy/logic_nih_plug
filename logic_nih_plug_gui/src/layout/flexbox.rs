//! CSS FlexBox-like layout system.
//!
//! This module provides a comprehensive FlexBox layout implementation
//! following CSS FlexBox specification patterns.

/// FlexBox direction - determines the main axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexDirection {
    /// Main axis is horizontal, left to right
    Row,
    /// Main axis is horizontal, right to left
    RowReverse,
    /// Main axis is vertical, top to bottom
    Column,
    /// Main axis is vertical, bottom to top
    ColumnReverse,
}

/// FlexBox wrapping behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexWrap {
    /// No wrapping - all items on one line
    NoWrap,
    /// Wrap to next line when needed
    Wrap,
    /// Wrap to next line in reverse order
    WrapReverse,
}

/// Main axis alignment (justify-content).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JustifyContent {
    /// Items packed at start of main axis
    FlexStart,
    /// Items packed at end of main axis
    FlexEnd,
    /// Items centered on main axis
    Center,
    /// Items evenly distributed, first at start, last at end
    SpaceBetween,
    /// Items evenly distributed with equal space around them
    SpaceAround,
}

/// Cross axis alignment for all items (align-items).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignItems {
    /// Items aligned at start of cross axis
    FlexStart,
    /// Items aligned at end of cross axis
    FlexEnd,
    /// Items centered on cross axis
    Center,
    /// Items stretched to fill cross axis
    Stretch,
    /// Items aligned at baseline
    Baseline,
}

/// Cross axis alignment for multi-line layouts (align-content).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignContent {
    /// Lines packed at start of cross axis
    FlexStart,
    /// Lines packed at end of cross axis
    FlexEnd,
    /// Lines centered on cross axis
    Center,
    /// Lines evenly distributed, first at start, last at end
    SpaceBetween,
    /// Lines evenly distributed with equal space around them
    SpaceAround,
    /// Lines stretched to fill cross axis
    Stretch,
}

/// Per-item cross axis alignment override (align-self).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignSelf {
    /// Use container's align-items value
    Auto,
    /// Aligned at start of cross axis
    FlexStart,
    /// Aligned at end of cross axis
    FlexEnd,
    /// Centered on cross axis
    Center,
    /// Stretched to fill cross axis
    Stretch,
    /// Aligned at baseline
    Baseline,
}

/// Margin specification for a flex item.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Margin {
    /// Top margin
    pub top: f32,
    /// Right margin
    pub right: f32,
    /// Bottom margin
    pub bottom: f32,
    /// Left margin
    pub left: f32,
}

impl Margin {
    /// Create a new margin with all sides set to zero.
    pub fn new() -> Self {
        Self {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        }
    }

    /// Create a margin with the same value on all sides.
    pub fn all(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    /// Create a margin with different horizontal and vertical values.
    pub fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }
}

impl Default for Margin {
    fn default() -> Self {
        Self::new()
    }
}

/// A rectangle representing position and size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    /// X coordinate
    pub x: f32,
    /// Y coordinate
    pub y: f32,
    /// Width
    pub width: f32,
    /// Height
    pub height: f32,
}

impl Rect {
    /// Create a new rectangle.
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }
}

/// A flex item with layout properties.
#[derive(Debug, Clone, PartialEq)]
pub struct FlexItem {
    /// Display order (lower values come first)
    pub order: i32,
    /// Flex grow factor (how much item grows relative to others)
    pub flex_grow: f32,
    /// Flex shrink factor (how much item shrinks relative to others)
    pub flex_shrink: f32,
    /// Initial main size before growing/shrinking
    pub flex_basis: f32,
    /// Per-item cross axis alignment override
    pub align_self: AlignSelf,
    /// Explicit width (None = auto)
    pub width: Option<f32>,
    /// Explicit height (None = auto)
    pub height: Option<f32>,
    /// Minimum width
    pub min_width: Option<f32>,
    /// Minimum height
    pub min_height: Option<f32>,
    /// Maximum width
    pub max_width: Option<f32>,
    /// Maximum height
    pub max_height: Option<f32>,
    /// Margin around the item
    pub margin: Margin,
}

impl FlexItem {
    /// Create a new flex item with default properties.
    pub fn new() -> Self {
        Self {
            order: 0,
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: 0.0,
            align_self: AlignSelf::Auto,
            width: None,
            height: None,
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
            margin: Margin::new(),
        }
    }

    /// Set the order.
    pub fn with_order(mut self, order: i32) -> Self {
        self.order = order;
        self
    }

    /// Set flex grow factor.
    pub fn with_flex_grow(mut self, grow: f32) -> Self {
        self.flex_grow = grow;
        self
    }

    /// Set flex shrink factor.
    pub fn with_flex_shrink(mut self, shrink: f32) -> Self {
        self.flex_shrink = shrink;
        self
    }

    /// Set flex basis.
    pub fn with_flex_basis(mut self, basis: f32) -> Self {
        self.flex_basis = basis;
        self
    }

    /// Set align-self.
    pub fn with_align_self(mut self, align: AlignSelf) -> Self {
        self.align_self = align;
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

    /// Set minimum width.
    pub fn with_min_width(mut self, min_width: f32) -> Self {
        self.min_width = Some(min_width);
        self
    }

    /// Set minimum height.
    pub fn with_min_height(mut self, min_height: f32) -> Self {
        self.min_height = Some(min_height);
        self
    }

    /// Set maximum width.
    pub fn with_max_width(mut self, max_width: f32) -> Self {
        self.max_width = Some(max_width);
        self
    }

    /// Set maximum height.
    pub fn with_max_height(mut self, max_height: f32) -> Self {
        self.max_height = Some(max_height);
        self
    }

    /// Set margin.
    pub fn with_margin(mut self, margin: Margin) -> Self {
        self.margin = margin;
        self
    }
}

impl Default for FlexItem {
    fn default() -> Self {
        Self::new()
    }
}

/// FlexBox container with layout algorithm.
#[derive(Debug, Clone, PartialEq)]
pub struct FlexBox {
    /// Main axis direction
    pub direction: FlexDirection,
    /// Wrapping behavior
    pub wrap: FlexWrap,
    /// Main axis alignment
    pub justify_content: JustifyContent,
    /// Cross axis alignment for all items
    pub align_items: AlignItems,
    /// Cross axis alignment for multi-line layouts
    pub align_content: AlignContent,
    /// Flex items
    pub items: Vec<FlexItem>,
}

impl FlexBox {
    /// Create a new FlexBox container with default properties.
    pub fn new() -> Self {
        Self {
            direction: FlexDirection::Row,
            wrap: FlexWrap::NoWrap,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Stretch,
            align_content: AlignContent::Stretch,
            items: Vec::new(),
        }
    }

    /// Add an item to the container.
    pub fn add_item(&mut self, item: FlexItem) {
        self.items.push(item);
    }

    /// Compute layout and return positioned rectangles for all items.
    ///
    /// Returns a vector of Rect, one for each item in the same order.
    pub fn layout(&self, container_width: f32, container_height: f32) -> Vec<Rect> {
        if self.items.is_empty() {
            return Vec::new();
        }

        // Sort items by order
        let mut indexed_items: Vec<(usize, &FlexItem)> = 
            self.items.iter().enumerate().collect();
        indexed_items.sort_by_key(|(_, item)| item.order);

        // Determine main and cross axis dimensions
        let (main_size, cross_size) = match self.direction {
            FlexDirection::Row | FlexDirection::RowReverse => (container_width, container_height),
            FlexDirection::Column | FlexDirection::ColumnReverse => (container_height, container_width),
        };

        // Layout items into lines
        let lines = self.layout_lines(&indexed_items, main_size);

        // Position lines on cross axis
        let line_positions = self.position_lines(&lines, cross_size);

        // Position items within each line
        let mut result = vec![Rect::new(0.0, 0.0, 0.0, 0.0); self.items.len()];
        
        for (line_idx, line) in lines.iter().enumerate() {
            let line_cross_pos = line_positions[line_idx];
            // For single-line layouts, use the full container cross size
            // For multi-line layouts, use the line's intrinsic cross size
            let line_cross_size = if lines.len() == 1 {
                cross_size
            } else {
                line.cross_size
            };
            
            self.position_items_in_line(
                line,
                main_size,
                line_cross_pos,
                line_cross_size,
                &mut result,
            );
        }

        result
    }

    // Layout items into lines based on wrapping
    fn layout_lines(&self, indexed_items: &[(usize, &FlexItem)], main_size: f32) -> Vec<FlexLine> {
        let mut lines = Vec::new();
        let mut current_line = FlexLine::new();

        // For reverse directions, we need to reverse the item order within each line
        let should_reverse_items = matches!(
            self.direction,
            FlexDirection::RowReverse | FlexDirection::ColumnReverse
        );

        for &(idx, item) in indexed_items {
            let item_main_size = self.get_item_main_size(item, main_size);
            
            // Check if we need to wrap
            if self.wrap != FlexWrap::NoWrap && !current_line.items.is_empty() {
                let line_size = current_line.main_size();
                if line_size + item_main_size > main_size {
                    // Reverse items in line if needed
                    if should_reverse_items {
                        current_line.items.reverse();
                    }
                    // Start new line
                    lines.push(current_line);
                    current_line = FlexLine::new();
                }
            }

            current_line.add_item(idx, item, item_main_size);
        }

        if !current_line.items.is_empty() {
            // Reverse items in last line if needed
            if should_reverse_items {
                current_line.items.reverse();
            }
            lines.push(current_line);
        }

        // Reverse lines if wrap-reverse
        if self.wrap == FlexWrap::WrapReverse {
            lines.reverse();
        }

        lines
    }

    // Get the main axis size for an item (base size before flex-grow/shrink)
    fn get_item_main_size(&self, item: &FlexItem, _container_main_size: f32) -> f32 {
        let base_size = match self.direction {
            FlexDirection::Row | FlexDirection::RowReverse => {
                item.width.unwrap_or(item.flex_basis)
            }
            FlexDirection::Column | FlexDirection::ColumnReverse => {
                item.height.unwrap_or(item.flex_basis)
            }
        };

        // Add margins
        let margin_main = match self.direction {
            FlexDirection::Row | FlexDirection::RowReverse => {
                item.margin.left + item.margin.right
            }
            FlexDirection::Column | FlexDirection::ColumnReverse => {
                item.margin.top + item.margin.bottom
            }
        };

        base_size + margin_main
    }

    // Resolve flexible lengths using flex-grow and flex-shrink
    fn resolve_flexible_lengths(&self, line: &FlexLine, container_main_size: f32) -> Vec<f32> {
        let mut sizes = Vec::with_capacity(line.items.len());
        
        // Calculate base sizes (flex-basis) - these include margins
        for &(_, ref _item, base_size) in &line.items {
            sizes.push(base_size);
        }

        let total_base_size: f32 = sizes.iter().sum();
        let free_space = container_main_size - total_base_size;

        if free_space > 0.0 {
            // Distribute free space using flex-grow
            let total_grow: f32 = line.items.iter().map(|(_, item, _)| item.flex_grow).sum();
            
            if total_grow > 0.0 {
                for (i, &(_, ref item, _)) in line.items.iter().enumerate() {
                    if item.flex_grow > 0.0 {
                        let grow_amount = (item.flex_grow / total_grow) * free_space;
                        sizes[i] += grow_amount;
                    }
                }
            }
        } else if free_space < 0.0 {
            // Shrink items using flex-shrink
            // Calculate the scaled flex shrink factor for each item
            let scaled_shrink_factors: Vec<f32> = line.items.iter()
                .enumerate()
                .map(|(i, (_, item, _))| {
                    // Get the content size (without margins) for shrinking calculation
                    let content_size = self.get_item_content_size(&line.items[i].1);
                    item.flex_shrink * content_size
                })
                .collect();
            
            let total_scaled_shrink: f32 = scaled_shrink_factors.iter().sum();
            
            if total_scaled_shrink > 0.0 {
                // Calculate how much each item should shrink
                let deficit = free_space.abs();
                
                // First pass: calculate ideal shrink amounts
                let mut shrink_amounts = vec![0.0; line.items.len()];
                for (i, &(_, ref item, _)) in line.items.iter().enumerate() {
                    if item.flex_shrink > 0.0 && scaled_shrink_factors[i] > 0.0 {
                        let shrink_ratio = scaled_shrink_factors[i] / total_scaled_shrink;
                        shrink_amounts[i] = shrink_ratio * deficit;
                    }
                }
                
                // Second pass: apply shrink amounts with constraints
                for (i, &(_, ref item, _)) in line.items.iter().enumerate() {
                    if item.flex_shrink > 0.0 {
                        let margin_size = self.get_item_margin_main(&line.items[i].1);
                        let min_size = margin_size.max(1.0); // Minimum 1px content + margins
                        
                        // Calculate new size
                        let new_size = sizes[i] - shrink_amounts[i];
                        
                        // Apply shrink, but don't go below minimum size
                        sizes[i] = new_size.max(min_size);
                    }
                }
                
                // If we couldn't shrink enough, items will overflow
                // This is acceptable behavior - better than overlapping items
            }
        }

        sizes
    }

    // Get the content size of an item (size without margins)
    fn get_item_content_size(&self, item: &FlexItem) -> f32 {
        match self.direction {
            FlexDirection::Row | FlexDirection::RowReverse => {
                item.width.unwrap_or(item.flex_basis)
            }
            FlexDirection::Column | FlexDirection::ColumnReverse => {
                item.height.unwrap_or(item.flex_basis)
            }
        }
    }

    // Get the margin size on the main axis
    fn get_item_margin_main(&self, item: &FlexItem) -> f32 {
        match self.direction {
            FlexDirection::Row | FlexDirection::RowReverse => {
                item.margin.left + item.margin.right
            }
            FlexDirection::Column | FlexDirection::ColumnReverse => {
                item.margin.top + item.margin.bottom
            }
        }
    }

    // Position lines on the cross axis
    fn position_lines(&self, lines: &[FlexLine], cross_size: f32) -> Vec<f32> {
        if lines.is_empty() {
            return Vec::new();
        }

        let total_cross_size: f32 = lines.iter().map(|l| l.cross_size).sum();
        let free_space = cross_size - total_cross_size;

        let mut positions = Vec::with_capacity(lines.len());
        let mut current_pos = 0.0;

        match self.align_content {
            AlignContent::FlexStart => {
                for line in lines {
                    positions.push(current_pos);
                    current_pos += line.cross_size;
                }
            }
            AlignContent::FlexEnd => {
                current_pos = free_space;
                for line in lines {
                    positions.push(current_pos);
                    current_pos += line.cross_size;
                }
            }
            AlignContent::Center => {
                current_pos = free_space / 2.0;
                for line in lines {
                    positions.push(current_pos);
                    current_pos += line.cross_size;
                }
            }
            AlignContent::SpaceBetween => {
                if lines.len() == 1 {
                    positions.push(0.0);
                } else {
                    let gap = free_space / (lines.len() - 1) as f32;
                    for line in lines.iter() {
                        positions.push(current_pos);
                        current_pos += line.cross_size + gap;
                    }
                }
            }
            AlignContent::SpaceAround => {
                let gap = free_space / lines.len() as f32;
                current_pos = gap / 2.0;
                for line in lines {
                    positions.push(current_pos);
                    current_pos += line.cross_size + gap;
                }
            }
            AlignContent::Stretch => {
                let extra_per_line = if lines.len() > 0 {
                    free_space / lines.len() as f32
                } else {
                    0.0
                };
                for line in lines {
                    positions.push(current_pos);
                    current_pos += line.cross_size + extra_per_line;
                }
            }
        }

        positions
    }

    // Position items within a line on the main axis
    fn position_items_in_line(
        &self,
        line: &FlexLine,
        container_main_size: f32,
        line_cross_pos: f32,
        line_cross_size: f32,
        result: &mut [Rect],
    ) {
        // Resolve flexible lengths (flex-grow and flex-shrink)
        let resolved_sizes = self.resolve_flexible_lengths(line, container_main_size);
        
        let total_main_size: f32 = resolved_sizes.iter().sum();
        let free_space = container_main_size - total_main_size;

        // Calculate main axis positions based on justify-content
        let mut main_positions = Vec::with_capacity(line.items.len());
        let mut current_pos = 0.0;

        match self.justify_content {
            JustifyContent::FlexStart => {
                for &item_size in &resolved_sizes {
                    main_positions.push(current_pos);
                    current_pos += item_size;
                }
            }
            JustifyContent::FlexEnd => {
                current_pos = free_space;
                for &item_size in &resolved_sizes {
                    main_positions.push(current_pos);
                    current_pos += item_size;
                }
            }
            JustifyContent::Center => {
                current_pos = free_space / 2.0;
                for &item_size in &resolved_sizes {
                    main_positions.push(current_pos);
                    current_pos += item_size;
                }
            }
            JustifyContent::SpaceBetween => {
                if line.items.len() == 1 {
                    main_positions.push(0.0);
                } else {
                    let gap = free_space / (line.items.len() - 1) as f32;
                    for &item_size in &resolved_sizes {
                        main_positions.push(current_pos);
                        current_pos += item_size + gap;
                    }
                }
            }
            JustifyContent::SpaceAround => {
                let gap = free_space / line.items.len() as f32;
                current_pos = gap / 2.0;
                for &item_size in &resolved_sizes {
                    main_positions.push(current_pos);
                    current_pos += item_size + gap;
                }
            }
        }

        // Position each item
        for (_i, &(item_idx, ref item, _)) in line.items.iter().enumerate() {
            let main_pos = main_positions[_i];
            let item_main_size = resolved_sizes[_i];
            
            // Determine cross axis alignment
            let align = match item.align_self {
                AlignSelf::Auto => self.align_items,
                AlignSelf::FlexStart => AlignItems::FlexStart,
                AlignSelf::FlexEnd => AlignItems::FlexEnd,
                AlignSelf::Center => AlignItems::Center,
                AlignSelf::Stretch => AlignItems::Stretch,
                AlignSelf::Baseline => AlignItems::Baseline,
            };

            // Get item cross size
            let item_cross_size = self.get_item_cross_size(item, line_cross_size, align);
            
            // Calculate cross axis position
            let cross_pos = match align {
                AlignItems::FlexStart => line_cross_pos,
                AlignItems::FlexEnd => line_cross_pos + line_cross_size - item_cross_size,
                AlignItems::Center => line_cross_pos + (line_cross_size - item_cross_size) / 2.0,
                AlignItems::Stretch => line_cross_pos,
                AlignItems::Baseline => line_cross_pos, // Simplified - would need baseline info
            };

            // Convert to x, y, width, height based on direction
            let rect = self.make_rect(
                main_pos,
                cross_pos,
                item_main_size,
                item_cross_size,
                item,
            );

            result[item_idx] = rect;
        }
    }

    // Get the cross axis size for an item
    fn get_item_cross_size(&self, item: &FlexItem, line_cross_size: f32, align: AlignItems) -> f32 {
        let base_size = match self.direction {
            FlexDirection::Row | FlexDirection::RowReverse => {
                if align == AlignItems::Stretch && item.height.is_none() {
                    line_cross_size
                } else {
                    item.height.unwrap_or(line_cross_size)
                }
            }
            FlexDirection::Column | FlexDirection::ColumnReverse => {
                if align == AlignItems::Stretch && item.width.is_none() {
                    line_cross_size
                } else {
                    item.width.unwrap_or(line_cross_size)
                }
            }
        };

        // Apply constraints
        let mut size = base_size;
        
        match self.direction {
            FlexDirection::Row | FlexDirection::RowReverse => {
                if let Some(min) = item.min_height {
                    size = size.max(min);
                }
                if let Some(max) = item.max_height {
                    size = size.min(max);
                }
            }
            FlexDirection::Column | FlexDirection::ColumnReverse => {
                if let Some(min) = item.min_width {
                    size = size.max(min);
                }
                if let Some(max) = item.max_width {
                    size = size.min(max);
                }
            }
        }

        size
    }

    // Create a Rect from main/cross positions and sizes
    fn make_rect(
        &self,
        main_pos: f32,
        cross_pos: f32,
        main_size: f32,
        cross_size: f32,
        item: &FlexItem,
    ) -> Rect {
        match self.direction {
            FlexDirection::Row | FlexDirection::RowReverse => Rect::new(
                main_pos + item.margin.left,
                cross_pos + item.margin.top,
                main_size - item.margin.left - item.margin.right,
                cross_size - item.margin.top - item.margin.bottom,
            ),
            FlexDirection::Column | FlexDirection::ColumnReverse => Rect::new(
                cross_pos + item.margin.left,
                main_pos + item.margin.top,
                cross_size - item.margin.left - item.margin.right,
                main_size - item.margin.top - item.margin.bottom,
            ),
        }
    }
}

impl Default for FlexBox {
    fn default() -> Self {
        Self::new()
    }
}

// Internal structure representing a line of flex items
#[derive(Debug, Clone)]
struct FlexLine {
    // (item_index, item_ref, main_size)
    items: Vec<(usize, FlexItem, f32)>,
    cross_size: f32,
}

impl FlexLine {
    fn new() -> Self {
        Self {
            items: Vec::new(),
            cross_size: 0.0,
        }
    }

    fn add_item(&mut self, idx: usize, item: &FlexItem, main_size: f32) {
        // Update cross size to be the maximum of all items
        let item_cross_size = match item.height {
            Some(h) => h + item.margin.top + item.margin.bottom,
            None => item.flex_basis + item.margin.top + item.margin.bottom,
        };
        self.cross_size = self.cross_size.max(item_cross_size);
        
        self.items.push((idx, item.clone(), main_size));
    }

    fn main_size(&self) -> f32 {
        self.items.iter().map(|(_, _, size)| size).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flexbox_creation() {
        let flexbox = FlexBox::new();
        assert_eq!(flexbox.direction, FlexDirection::Row);
        assert_eq!(flexbox.wrap, FlexWrap::NoWrap);
        assert_eq!(flexbox.justify_content, JustifyContent::FlexStart);
        assert_eq!(flexbox.align_items, AlignItems::Stretch);
        assert_eq!(flexbox.items.len(), 0);
    }

    #[test]
    fn test_flex_item_creation() {
        let item = FlexItem::new();
        assert_eq!(item.order, 0);
        assert_eq!(item.flex_grow, 0.0);
        assert_eq!(item.flex_shrink, 1.0);
        assert_eq!(item.flex_basis, 0.0);
        assert_eq!(item.align_self, AlignSelf::Auto);
    }

    #[test]
    fn test_flex_item_builder() {
        let item = FlexItem::new()
            .with_order(1)
            .with_flex_grow(1.0)
            .with_width(100.0)
            .with_height(50.0);

        assert_eq!(item.order, 1);
        assert_eq!(item.flex_grow, 1.0);
        assert_eq!(item.width, Some(100.0));
        assert_eq!(item.height, Some(50.0));
    }

    #[test]
    fn test_margin_creation() {
        let margin = Margin::new();
        assert_eq!(margin.top, 0.0);
        assert_eq!(margin.right, 0.0);
        assert_eq!(margin.bottom, 0.0);
        assert_eq!(margin.left, 0.0);
    }

    #[test]
    fn test_margin_all() {
        let margin = Margin::all(10.0);
        assert_eq!(margin.top, 10.0);
        assert_eq!(margin.right, 10.0);
        assert_eq!(margin.bottom, 10.0);
        assert_eq!(margin.left, 10.0);
    }

    #[test]
    fn test_margin_symmetric() {
        let margin = Margin::symmetric(20.0, 10.0);
        assert_eq!(margin.top, 10.0);
        assert_eq!(margin.right, 20.0);
        assert_eq!(margin.bottom, 10.0);
        assert_eq!(margin.left, 20.0);
    }

    #[test]
    fn test_simple_row_layout() {
        let mut flexbox = FlexBox::new();
        flexbox.direction = FlexDirection::Row;
        
        flexbox.add_item(FlexItem::new().with_width(100.0).with_height(50.0));
        flexbox.add_item(FlexItem::new().with_width(100.0).with_height(50.0));
        
        let rects = flexbox.layout(300.0, 100.0);
        
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0].x, 0.0);
        assert_eq!(rects[1].x, 100.0);
    }

    #[test]
    fn test_simple_column_layout() {
        let mut flexbox = FlexBox::new();
        flexbox.direction = FlexDirection::Column;
        
        flexbox.add_item(FlexItem::new().with_width(100.0).with_height(50.0));
        flexbox.add_item(FlexItem::new().with_width(100.0).with_height(50.0));
        
        let rects = flexbox.layout(300.0, 200.0);
        
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0].y, 0.0);
        assert_eq!(rects[1].y, 50.0);
    }

    #[test]
    fn test_justify_content_center() {
        let mut flexbox = FlexBox::new();
        flexbox.direction = FlexDirection::Row;
        flexbox.justify_content = JustifyContent::Center;
        
        flexbox.add_item(FlexItem::new().with_width(100.0).with_height(50.0));
        
        let rects = flexbox.layout(300.0, 100.0);
        
        assert_eq!(rects.len(), 1);
        // Should be centered: (300 - 100) / 2 = 100
        assert_eq!(rects[0].x, 100.0);
    }

    #[test]
    fn test_justify_content_space_between() {
        let mut flexbox = FlexBox::new();
        flexbox.direction = FlexDirection::Row;
        flexbox.justify_content = JustifyContent::SpaceBetween;
        
        flexbox.add_item(FlexItem::new().with_width(50.0).with_height(50.0));
        flexbox.add_item(FlexItem::new().with_width(50.0).with_height(50.0));
        
        let rects = flexbox.layout(300.0, 100.0);
        
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0].x, 0.0);
        // Second item at end: 300 - 50 = 250
        assert_eq!(rects[1].x, 250.0);
    }

    #[test]
    fn test_empty_flexbox() {
        let flexbox = FlexBox::new();
        let rects = flexbox.layout(300.0, 100.0);
        assert_eq!(rects.len(), 0);
    }

    #[test]
    fn test_item_with_margin() {
        let mut flexbox = FlexBox::new();
        flexbox.direction = FlexDirection::Row;
        
        let item = FlexItem::new()
            .with_width(100.0)
            .with_height(50.0)
            .with_margin(Margin::all(10.0));
        
        flexbox.add_item(item);
        
        let rects = flexbox.layout(300.0, 100.0);
        
        assert_eq!(rects.len(), 1);
        // Position should account for margin
        assert_eq!(rects[0].x, 10.0);
        assert_eq!(rects[0].y, 10.0);
        // Width includes margins in main size calculation, so it's the full width minus margins
        assert_eq!(rects[0].width, 100.0);
        assert_eq!(rects[0].height, 30.0);
    }

    #[test]
    fn test_two_items_row_layout() {
        let mut flexbox = FlexBox::new();
        flexbox.direction = FlexDirection::Row;
        
        flexbox.add_item(FlexItem::new().with_width(10.0).with_height(50.0));
        flexbox.add_item(FlexItem::new().with_width(10.0).with_height(50.0));
        
        let rects = flexbox.layout(300.0, 300.0);
        
        assert_eq!(rects.len(), 2);
        // First item should start at 0
        assert_eq!(rects[0].x, 0.0);
        assert_eq!(rects[0].width, 10.0);
        // Second item should start after first item
        assert_eq!(rects[1].x, 10.0);
        assert_eq!(rects[1].width, 10.0);
    }

    #[test]
    fn test_two_items_row_reverse_layout() {
        let mut flexbox = FlexBox::new();
        flexbox.direction = FlexDirection::RowReverse;
        
        flexbox.add_item(FlexItem::new().with_width(10.0).with_height(50.0));
        flexbox.add_item(FlexItem::new().with_width(10.0).with_height(50.0));
        
        let rects = flexbox.layout(300.0, 300.0);
        
        assert_eq!(rects.len(), 2);
        // In RowReverse, first item (index 0) should be on the right
        // Second item (index 1) should be on the left
        // So item 1 should have smaller x than item 0
        assert!(rects[1].x < rects[0].x, "Item 1 at x={}, Item 0 at x={}", rects[1].x, rects[0].x);
        // Items should not overlap
        assert!(rects[1].x + rects[1].width <= rects[0].x + 0.1);
    }

    #[test]
    fn test_many_items_with_shrinking() {
        let mut flexbox = FlexBox::new();
        flexbox.direction = FlexDirection::Row;
        flexbox.wrap = FlexWrap::NoWrap;
        
        // Add 8 items with total width > container width
        flexbox.add_item(FlexItem::new().with_width(100.0).with_height(50.0).with_flex_shrink(1.0));
        flexbox.add_item(FlexItem::new().with_width(100.0).with_height(50.0).with_flex_shrink(1.0));
        flexbox.add_item(FlexItem::new().with_width(100.0).with_height(50.0).with_flex_shrink(0.0)); // Won't shrink
        flexbox.add_item(FlexItem::new().with_width(100.0).with_height(50.0).with_flex_shrink(1.0));
        flexbox.add_item(FlexItem::new().with_width(100.0).with_height(50.0).with_flex_shrink(1.0));
        
        let rects = flexbox.layout(300.0, 300.0);
        
        assert_eq!(rects.len(), 5);
        
        // Check that no items overlap
        let mut sorted: Vec<_> = rects.iter().enumerate().collect();
        sorted.sort_by(|a, b| a.1.x.partial_cmp(&b.1.x).unwrap());
        
        for i in 0..sorted.len() - 1 {
            let current_right = sorted[i].1.x + sorted[i].1.width;
            let next_left = sorted[i + 1].1.x;
            assert!(
                next_left >= current_right - 0.1,
                "Item {} (x={}, width={}) overlaps with item {} (x={})",
                sorted[i].0, sorted[i].1.x, sorted[i].1.width,
                sorted[i + 1].0, sorted[i + 1].1.x
            );
        }
    }

    #[test]
    fn test_row_reverse_with_different_orders() {
        let mut flexbox = FlexBox::new();
        flexbox.direction = FlexDirection::RowReverse;
        flexbox.wrap = FlexWrap::NoWrap;
        
        // Add items with different orders - similar to the failing case
        flexbox.add_item(FlexItem::new().with_order(0).with_width(100.0).with_height(50.0).with_flex_shrink(0.0));
        flexbox.add_item(FlexItem::new().with_order(7).with_width(100.0).with_height(50.0).with_flex_shrink(0.0));
        flexbox.add_item(FlexItem::new().with_order(2).with_width(100.0).with_height(50.0).with_flex_shrink(0.0));
        flexbox.add_item(FlexItem::new().with_order(0).with_width(100.0).with_height(50.0).with_flex_shrink(0.0));
        flexbox.add_item(FlexItem::new().with_order(0).with_width(100.0).with_height(50.0).with_flex_shrink(0.0));
        
        let rects = flexbox.layout(300.0, 300.0);
        
        assert_eq!(rects.len(), 5);
        
        // Check that no items overlap
        let mut sorted: Vec<_> = rects.iter().enumerate().collect();
        sorted.sort_by(|a, b| a.1.x.partial_cmp(&b.1.x).unwrap());
        
        for i in 0..sorted.len() - 1 {
            let current_right = sorted[i].1.x + sorted[i].1.width;
            let next_left = sorted[i + 1].1.x;
            assert!(
                next_left >= current_right - 0.1,
                "Item {} (x={}, width={}) overlaps with item {} (x={})",
                sorted[i].0, sorted[i].1.x, sorted[i].1.width,
                sorted[i + 1].0, sorted[i + 1].1.x
            );
        }
    }
}
