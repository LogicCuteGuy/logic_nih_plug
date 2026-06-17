use crossbeam::atomic::AtomicCell;
use logic_nih_plug::prelude::*;
use logic_nih_plug_graphics::{Color, Graphics};
use logic_nih_plug_gui::layout::flexbox::*;
use std::sync::Arc;

use crate::FlexBoxParams;

pub struct FlexBoxDemoEditor {
    pub params: Arc<FlexBoxParams>,
    pub scaling_factor: AtomicCell<Option<f32>>,
}

impl Editor for FlexBoxDemoEditor {
    fn spawn(
        &self,
        _parent: logic_nih_plug::editor::ParentWindowHandle,
        _context: Arc<dyn GuiContext>,
    ) -> Box<dyn std::any::Any + Send> {
        let params = Arc::clone(&self.params);

        // Create GL window with custom rendering
        let window = logic_nih_plug_gui::GlWindowBuilder::spawn(
            "FlexBox Layout Demo",
            900,
            700,
            move |graphics: &mut Graphics, (w, h)| {
                // Clear background
                graphics.set_color(Color::rgb(20, 20, 25));
                graphics.clear();

                // Draw title
                graphics.set_color(Color::rgb(220, 220, 220));
                draw_text_centered(graphics, "FlexBox Layout Demo", w as i32 / 2, 20, 20.0);

                // Get current parameter values
                let direction: FlexDirection = params.direction.value().into();
                let wrap: FlexWrap = params.wrap.value().into();
                let justify_content: JustifyContent = params.justify_content.value().into();
                let align_items: AlignItems = params.align_items.value().into();
                let align_content: AlignContent = params.align_content.value().into();
                let num_items = params.num_items.value() as usize;
                let container_width = params.container_width.value();
                let container_height = params.container_height.value();

                // Create FlexBox with current settings
                let mut flexbox = FlexBox::new();
                flexbox.direction = direction;
                flexbox.wrap = wrap;
                flexbox.justify_content = justify_content;
                flexbox.align_items = align_items;
                flexbox.align_content = align_content;

                // Add items with varying properties
                for i in 0..num_items {
                    let item = create_demo_item(i, num_items);
                    flexbox.add_item(item);
                }

                // Calculate layout
                let rects = flexbox.layout(container_width, container_height);

                // Draw FlexBox container
                let container_x = (w as f32 - container_width) / 2.0;
                let container_y = 80.0;

                draw_flexbox_container(
                    graphics,
                    container_x as i32,
                    container_y as i32,
                    container_width as u32,
                    container_height as u32,
                    &rects,
                );

                // Draw parameter values
                draw_parameters(
                    graphics,
                    &params,
                    20,
                    (container_y + container_height + 30.0) as i32,
                    w - 40,
                );

                // Draw instructions
                graphics.set_color(Color::rgb(120, 120, 120));
                draw_text_centered(
                    graphics,
                    "Use your DAW's automation to control FlexBox properties",
                    w as i32 / 2,
                    h as i32 - 20,
                    12.0,
                );
            },
        );

        struct EditorHandle {
            window: logic_nih_plug_gui::GlWindow,
        }
        unsafe impl Send for EditorHandle {}

        Box::new(EditorHandle { window })
    }

    fn size(&self) -> (u32, u32) {
        (900, 700)
    }

    fn set_scale_factor(&self, factor: f32) -> bool {
        if self.scaling_factor.load().is_some() {
            return false;
        }
        self.scaling_factor.store(Some(factor));
        true
    }

    fn param_value_changed(&self, _id: &str, _normalized_value: f32) {}
    fn param_modulation_changed(&self, _id: &str, _modulation_offset: f32) {}
    fn param_values_changed(&self) {}
}

/// Creates a demo flex item with varying properties
fn create_demo_item(index: usize, total: usize) -> FlexItem {
    // Vary item properties to demonstrate different behaviors
    let base_width = 60.0 + (index as f32 * 10.0) % 40.0;
    let base_height = 40.0 + (index as f32 * 15.0) % 30.0;

    let mut item = FlexItem::new()
        .with_width(base_width)
        .with_height(base_height)
        .with_margin(Margin::all(5.0));

    // Add flex-grow to some items
    if index % 3 == 0 {
        item = item.with_flex_grow(1.0);
    }

    // Add different align-self to some items
    if index == total - 1 && total > 1 {
        item = item.with_align_self(AlignSelf::FlexEnd);
    } else if index == 1 && total > 2 {
        item = item.with_align_self(AlignSelf::Center);
    }

    item
}

/// Draws the FlexBox container and its items
fn draw_flexbox_container(
    graphics: &mut Graphics,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    rects: &[Rect],
) {
    // Draw container background
    graphics.set_color(Color::rgb(30, 30, 35));
    graphics.fill_rect(x, y, width, height);

    // Draw container border
    graphics.set_color(Color::rgb(80, 80, 90));
    draw_rect_outline(graphics, x, y, width, height);

    // Draw each flex item
    for (i, rect) in rects.iter().enumerate() {
        // Calculate absolute position
        let item_x = x + rect.x as i32;
        let item_y = y + rect.y as i32;
        let item_width = rect.width as u32;
        let item_height = rect.height as u32;

        // Use different colors for items
        let hue = (i as f32 * 360.0 / rects.len() as f32) as i32;
        let color = hsl_to_rgb(hue, 70, 60);
        graphics.set_color(color);
        graphics.fill_rect(item_x, item_y, item_width, item_height);

        // Draw item border
        graphics.set_color(Color::rgb(255, 255, 255));
        draw_rect_outline(graphics, item_x, item_y, item_width, item_height);

        // Draw item index
        graphics.set_color(Color::rgb(255, 255, 255));
        let label = format!("{}", i);
        draw_text_centered(
            graphics,
            &label,
            item_x + item_width as i32 / 2,
            item_y + item_height as i32 / 2 - 5,
            12.0,
        );

        // Draw item dimensions
        let dim_label = format!("{}x{}", item_width, item_height);
        draw_text_centered(
            graphics,
            &dim_label,
            item_x + item_width as i32 / 2,
            item_y + item_height as i32 / 2 + 8,
            10.0,
        );
    }
}

/// Draws parameter values
fn draw_parameters(graphics: &mut Graphics, params: &FlexBoxParams, x: i32, y: i32, width: u32) {
    graphics.set_color(Color::rgb(180, 180, 180));

    let col_width = width / 3;
    let row_height = 25;

    // Column 1
    let mut current_y = y;
    draw_param_label(
        graphics,
        "Direction:",
        format_direction(params.direction.value()),
        x,
        current_y,
    );
    current_y += row_height;

    draw_param_label(
        graphics,
        "Wrap:",
        format_wrap(params.wrap.value()),
        x,
        current_y,
    );
    current_y += row_height;

    draw_param_label(
        graphics,
        "Justify Content:",
        format_justify(params.justify_content.value()),
        x,
        current_y,
    );

    // Column 2
    current_y = y;
    let col2_x = x + col_width as i32;
    draw_param_label(
        graphics,
        "Align Items:",
        format_align_items(params.align_items.value()),
        col2_x,
        current_y,
    );
    current_y += row_height;

    draw_param_label(
        graphics,
        "Align Content:",
        format_align_content(params.align_content.value()),
        col2_x,
        current_y,
    );
    current_y += row_height;

    draw_param_label(
        graphics,
        "Number of Items:",
        &format!("{}", params.num_items.value()),
        col2_x,
        current_y,
    );

    // Column 3
    current_y = y;
    let col3_x = x + (col_width * 2) as i32;
    draw_param_label(
        graphics,
        "Container Width:",
        &format!("{:.0} px", params.container_width.value()),
        col3_x,
        current_y,
    );
    current_y += row_height;

    draw_param_label(
        graphics,
        "Container Height:",
        &format!("{:.0} px", params.container_height.value()),
        col3_x,
        current_y,
    );
}

/// Draws a parameter label and value
fn draw_param_label(graphics: &mut Graphics, label: &str, value: &str, x: i32, y: i32) {
    draw_text_left(graphics, label, x, y, 12.0);
    draw_text_left(graphics, value, x + 150, y, 12.0);
}

/// Format direction enum
fn format_direction(dir: crate::FlexDirectionParam) -> &'static str {
    match dir {
        crate::FlexDirectionParam::Row => "Row",
        crate::FlexDirectionParam::RowReverse => "Row Reverse",
        crate::FlexDirectionParam::Column => "Column",
        crate::FlexDirectionParam::ColumnReverse => "Column Reverse",
    }
}

/// Format wrap enum
fn format_wrap(wrap: crate::FlexWrapParam) -> &'static str {
    match wrap {
        crate::FlexWrapParam::NoWrap => "No Wrap",
        crate::FlexWrapParam::Wrap => "Wrap",
        crate::FlexWrapParam::WrapReverse => "Wrap Reverse",
    }
}

/// Format justify content enum
fn format_justify(justify: crate::JustifyContentParam) -> &'static str {
    match justify {
        crate::JustifyContentParam::FlexStart => "Flex Start",
        crate::JustifyContentParam::FlexEnd => "Flex End",
        crate::JustifyContentParam::Center => "Center",
        crate::JustifyContentParam::SpaceBetween => "Space Between",
        crate::JustifyContentParam::SpaceAround => "Space Around",
    }
}

/// Format align items enum
fn format_align_items(align: crate::AlignItemsParam) -> &'static str {
    match align {
        crate::AlignItemsParam::FlexStart => "Flex Start",
        crate::AlignItemsParam::FlexEnd => "Flex End",
        crate::AlignItemsParam::Center => "Center",
        crate::AlignItemsParam::Stretch => "Stretch",
        crate::AlignItemsParam::Baseline => "Baseline",
    }
}

/// Format align content enum
fn format_align_content(align: crate::AlignContentParam) -> &'static str {
    match align {
        crate::AlignContentParam::FlexStart => "Flex Start",
        crate::AlignContentParam::FlexEnd => "Flex End",
        crate::AlignContentParam::Center => "Center",
        crate::AlignContentParam::SpaceBetween => "Space Between",
        crate::AlignContentParam::SpaceAround => "Space Around",
        crate::AlignContentParam::Stretch => "Stretch",
    }
}

/// Converts HSL to RGB color
fn hsl_to_rgb(h: i32, s: i32, l: i32) -> Color {
    let h = h as f32 / 360.0;
    let s = s as f32 / 100.0;
    let l = l as f32 / 100.0;

    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r, g, b) = if h < 1.0 / 6.0 {
        (c, x, 0.0)
    } else if h < 2.0 / 6.0 {
        (x, c, 0.0)
    } else if h < 3.0 / 6.0 {
        (0.0, c, x)
    } else if h < 4.0 / 6.0 {
        (0.0, x, c)
    } else if h < 5.0 / 6.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    Color::rgb(
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

/// Helper function to draw centered text (simplified - no actual font rendering)
fn draw_text_centered(graphics: &mut Graphics, text: &str, x: i32, y: i32, _size: f32) {
    // Approximate text width (8 pixels per character)
    let text_width = (text.len() * 8) as i32;
    let start_x = x - text_width / 2;

    // Draw simple text representation (placeholder)
    graphics.fill_rect(start_x, y, text_width as u32, 2);
}

/// Helper function to draw left-aligned text
fn draw_text_left(graphics: &mut Graphics, text: &str, x: i32, y: i32, _size: f32) {
    let text_width = (text.len() * 8) as u32;
    graphics.fill_rect(x, y, text_width, 2);
}

/// Helper function to draw a rectangle outline
fn draw_rect_outline(graphics: &mut Graphics, x: i32, y: i32, width: u32, height: u32) {
    let x2 = x + width as i32 - 1;
    let y2 = y + height as i32 - 1;
    
    // Top
    graphics.draw_line(x, y, x2, y);
    // Right
    graphics.draw_line(x2, y, x2, y2);
    // Bottom
    graphics.draw_line(x2, y2, x, y2);
    // Left
    graphics.draw_line(x, y2, x, y);
}
