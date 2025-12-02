# JUCE FFI Complex Layout Example

This example demonstrates advanced layout capabilities using JUCE's FlexBox system through FFI bindings in a nih-plug plugin.

## Features Demonstrated

This example showcases:

- **FlexBox Layout System**: Using JUCE's FlexBox for flexible, responsive layouts
- **Multiple Layout Patterns**: Horizontal, vertical, grid, and nested layouts
- **Component Hierarchies**: Building complex UIs with nested component structures
- **Responsive Design**: Layouts that adapt to different window sizes
- **Flex Properties**: Using flex-grow, flex-shrink, flex-basis for proportional sizing
- **Margins and Spacing**: Proper spacing between components
- **Layout Directions**: Row, column, row-reverse, column-reverse
- **Wrapping Behavior**: Creating grid-like layouts with flex-wrap
- **Justification and Alignment**: Controlling item distribution and alignment
- **Real-World UI Patterns**: Sidebar layouts, dashboard layouts, plugin control panels

## Examples Included

The plugin includes 7 comprehensive layout examples:

### 1. Basic Horizontal Layout
Three equal columns with margins, demonstrating basic row layout with equal flex-grow values.

### 2. Vertical Sections
Header, content, and footer layout showing different flex-grow values for proportional sizing.

### 3. Grid Layout
12-item grid using flex-wrap to create a responsive grid pattern with uniform spacing.

### 4. Nested Layouts
Sidebar with vertical button list + main content area with grid panels, demonstrating component hierarchies.

### 5. Plugin UI Layout
Real-world multi-band EQ interface with title bar, visualization area, and control sections.

### 6. Responsive Layout
8 cards that adapt to available space, demonstrating min/max constraints and responsive behavior.

### 7. Dashboard Layout
Complex dashboard with stats bar, waveform display, spectrum analyzer, and control panels.

## Building

Build the plugin using cargo:

```bash
cargo build --release
```

Or build just this example:

```bash
cargo build --release -p juce_ffi_layout
```

## Usage

### As a Plugin

The plugin can be loaded in any DAW that supports CLAP or VST3 formats. The compiled plugin will be in:

- **CLAP**: `target/release/libjuce_ffi_layout.clap` (Linux/macOS) or `target/release/juce_ffi_layout.clap` (Windows)
- **VST3**: `target/release/libjuce_ffi_layout.vst3`

### As a Code Reference

The main value of this example is as a code reference. Each example function demonstrates different layout patterns:

```rust
// Example 1: Basic horizontal layout
fn example_basic_horizontal_layout() -> Result<Component>

// Example 2: Vertical sections (header/content/footer)
fn example_vertical_sections() -> Result<Component>

// Example 3: Grid layout with wrapping
fn example_grid_layout() -> Result<Component>

// Example 4: Nested layouts (sidebar + content)
fn example_nested_layouts() -> Result<Component>

// Example 5: Plugin UI with controls and visualization
fn example_plugin_ui_layout() -> Result<Component>

// Example 6: Responsive layout
fn example_responsive_layout() -> Result<Component>

// Example 7: Dashboard layout
fn example_dashboard_layout() -> Result<Component>
```

## Key Concepts

### FlexBox Basics

```rust
// Create a FlexBox
let mut flexbox = FlexBox::new()?;

// Set direction (Row, Column, RowReverse, ColumnReverse)
flexbox.set_direction(FlexDirection::Row);

// Set wrapping behavior
flexbox.set_wrap(FlexWrap::Wrap);

// Set justification along main axis
flexbox.set_justify_content(JustifyContent::SpaceBetween);

// Set alignment along cross axis
flexbox.set_align_items(AlignItems::Center);
```

### Creating Flex Items

```rust
// Create a component
let component = Component::new()?;

// Wrap it in a FlexItem with properties
let item = FlexItem::new(&component)
    .with_flex_grow(1.0)        // How much it grows
    .with_flex_shrink(1.0)      // How much it shrinks
    .with_flex_basis(200.0)     // Initial size
    .with_min_width(100.0)      // Minimum width
    .with_max_width(400.0)      // Maximum width
    .with_margin(10.0, 10.0, 10.0, 10.0);  // Top, right, bottom, left

// Add to flexbox
flexbox.add_item(item);
```

### Performing Layout

```rust
// Calculate and apply layout within bounds
flexbox.perform_layout(x, y, width, height);
```

## Layout Patterns

### Equal Columns

```rust
// Three equal columns
for component in &components {
    let item = FlexItem::new(component)
        .with_flex_grow(1.0);  // All grow equally
    flexbox.add_item(item);
}
```

### Fixed + Flexible

```rust
// Sidebar (fixed) + content (flexible)
let sidebar_item = FlexItem::new(&sidebar)
    .with_flex_grow(0.0)
    .with_flex_basis(200.0);  // Fixed 200px

let content_item = FlexItem::new(&content)
    .with_flex_grow(1.0);  // Grows to fill space

flexbox.add_item(sidebar_item);
flexbox.add_item(content_item);
```

### Grid with Wrapping

```rust
flexbox.set_direction(FlexDirection::Row);
flexbox.set_wrap(FlexWrap::Wrap);

for component in &components {
    let item = FlexItem::new(component)
        .with_flex_grow(0.0)
        .with_flex_basis(250.0)  // Fixed width
        .with_min_height(150.0)
        .with_margin(10.0, 10.0, 10.0, 10.0);
    flexbox.add_item(item);
}
```

### Nested Layouts

```rust
// Outer vertical layout
let mut outer_flexbox = FlexBox::new()?;
outer_flexbox.set_direction(FlexDirection::Column);

// Inner horizontal layout
let mut inner_flexbox = FlexBox::new()?;
inner_flexbox.set_direction(FlexDirection::Row);

// Add items to inner, then add inner container to outer
```

## Thread Safety

All JUCE GUI operations, including FlexBox layout, must be performed on the JUCE message thread. The type system enforces this - FlexBox and Component do not implement `Send` or `Sync`.

In a real plugin editor, you would:

1. Create components on the message thread
2. Set up layouts on the message thread
3. Use `MessageManager::call_async()` to update UI from other threads

## Performance Considerations

- FlexBox layout calculation is efficient but should be done only when needed (window resize, component addition/removal)
- Cache layout results when possible
- For very complex layouts, consider breaking into smaller sub-layouts
- The FFI overhead for layout operations is minimal (< 5% compared to native C++ JUCE)

## Integration with nih-plug

To integrate these layouts into a real nih-plug editor:

1. Create your layout in the editor's constructor
2. Store component references in the editor struct
3. Connect widgets to parameters using callbacks or parameter attachments
4. Handle window resize events by recalculating layout
5. Use `repaint()` to trigger redraws when needed

## Further Reading

- [JUCE FlexBox Documentation](https://docs.juce.com/master/classFlexBox.html)
- [CSS Flexbox Guide](https://css-tricks.com/snippets/css/a-guide-to-flexbox/) - JUCE FlexBox follows similar semantics
- [nih-plug Documentation](https://github.com/robbert-vdh/nih-plug)
- [JUCE FFI Integration Design Document](../../../.kiro/specs/juce-ffi-integration/design.md)

## License

ISC License - See the root LICENSE file for details.
