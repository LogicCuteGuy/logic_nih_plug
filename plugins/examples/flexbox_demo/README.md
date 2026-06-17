# FlexBox Layout Demo

This example demonstrates the FlexBox layout system from `logic_nih_plug_gui` with interactive controls for all FlexBox properties.

## Features

- **Interactive Controls**: All FlexBox properties can be controlled via DAW automation:
  - `flex-direction`: Row, Row Reverse, Column, Column Reverse
  - `flex-wrap`: No Wrap, Wrap, Wrap Reverse
  - `justify-content`: Flex Start, Flex End, Center, Space Between, Space Around
  - `align-items`: Flex Start, Flex End, Center, Stretch, Baseline
  - `align-content`: Flex Start, Flex End, Center, Space Between, Space Around, Stretch
  - Number of items (1-20)
  - Container width and height (for responsive testing)

- **Visual Feedback**: 
  - Each flex item is displayed with a unique color
  - Item dimensions and positions are shown in real-time
  - Container boundaries are clearly marked
  - Items show their index and size

- **Responsive Layout**: 
  - Adjust container size to see how layout adapts
  - Test wrapping behavior with different container sizes
  - Observe how items grow and shrink

## Requirements Validated

This example validates the following requirements from the JUCE Examples Validation spec:

- **8.1**: FlexBox direction (row, row-reverse, column, column-reverse)
- **8.2**: FlexBox wrapping (nowrap, wrap, wrap-reverse)
- **8.3**: Justify-content (flex-start, flex-end, center, space-between, space-around)
- **8.4**: Align-items and align-content properties
- **8.5**: Align-self property (demonstrated on specific items)
- **10.3**: Example demonstrating FlexBox features

## Building

Build this example with:

```bash
cargo xtask bundle flexbox_demo --release
```

## Usage

1. Load the plugin in your DAW
2. Open the plugin GUI to see the FlexBox layout visualization
3. Use your DAW's automation to adjust FlexBox properties
4. Observe how the layout changes in real-time
5. Try different combinations of properties to understand FlexBox behavior

## FlexBox Properties Explained

### Direction
Controls the main axis direction:
- **Row**: Items flow left to right
- **Row Reverse**: Items flow right to left
- **Column**: Items flow top to bottom
- **Column Reverse**: Items flow bottom to top

### Wrap
Controls whether items wrap to new lines:
- **No Wrap**: All items on one line (may shrink)
- **Wrap**: Items wrap to new lines when needed
- **Wrap Reverse**: Items wrap in reverse order

### Justify Content
Controls spacing on the main axis:
- **Flex Start**: Items packed at start
- **Flex End**: Items packed at end
- **Center**: Items centered
- **Space Between**: Even spacing, first/last at edges
- **Space Around**: Even spacing around all items

### Align Items
Controls alignment on the cross axis:
- **Flex Start**: Items aligned at start
- **Flex End**: Items aligned at end
- **Center**: Items centered
- **Stretch**: Items stretched to fill
- **Baseline**: Items aligned at baseline

### Align Content
Controls line spacing in multi-line layouts:
- **Flex Start**: Lines packed at start
- **Flex End**: Lines packed at end
- **Center**: Lines centered
- **Space Between**: Even spacing between lines
- **Space Around**: Even spacing around lines
- **Stretch**: Lines stretched to fill

## Implementation Notes

This example demonstrates:
- Creating a FlexBox container with configurable properties
- Adding flex items with varying sizes and properties
- Computing layout with the `layout()` method
- Rendering the computed layout with visual feedback
- Responsive behavior with adjustable container size
- Different item properties (flex-grow, align-self, margins)

The example uses a simple audio pass-through, as it's focused on demonstrating GUI layout capabilities.
