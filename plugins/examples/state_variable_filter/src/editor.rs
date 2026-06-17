use atomic_float::AtomicF32;
use crossbeam::atomic::AtomicCell;
use logic_nih_plug::prelude::*;
use logic_nih_plug_graphics::{Color, Graphics};
use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::FilterParams;

pub struct StateVariableFilterEditor {
    pub params: Arc<FilterParams>,
    pub frequency_response: Arc<[AtomicF32; 128]>,
    pub scaling_factor: AtomicCell<Option<f32>>,
}

impl Editor for StateVariableFilterEditor {
    fn spawn(
        &self,
        _parent: logic_nih_plug::editor::ParentWindowHandle,
        _context: Arc<dyn GuiContext>,
    ) -> Box<dyn std::any::Any + Send> {
        let params = Arc::clone(&self.params);
        let frequency_response = Arc::clone(&self.frequency_response);

        // Create GL window with custom rendering
        let window = logic_nih_plug_gui::GlWindowBuilder::spawn(
            "State Variable Filter",
            600,
            400,
            move |graphics: &mut Graphics, (w, h)| {
                // Clear background
                graphics.set_color(Color::rgb(24, 24, 24));
                graphics.clear();

                // Draw title
                graphics.set_color(Color::rgb(220, 220, 220));
                draw_text_centered(graphics, "State Variable Filter", w as i32 / 2, 20, 24.0);

                // Draw frequency response visualization
                draw_frequency_response(graphics, &frequency_response, 50, 60, w - 100, 180);

                // Draw parameter labels and values
                let filter_type_str = match params.filter_type.value() {
                    crate::FilterType::Lowpass => "Lowpass",
                    crate::FilterType::Bandpass => "Bandpass",
                    crate::FilterType::Highpass => "Highpass",
                };

                let cutoff_str = format!("{:.0} Hz", params.cutoff.value());
                let resonance_str = format!("{:.3}", params.resonance.value());

                graphics.set_color(Color::rgb(180, 180, 180));
                let y_offset = 260;

                // Filter Type
                draw_text_left(graphics, "Filter Type:", 50, y_offset, 14.0);
                draw_text_left(graphics, filter_type_str, 200, y_offset, 14.0);

                // Cutoff
                draw_text_left(graphics, "Cutoff:", 50, y_offset + 30, 14.0);
                draw_text_left(graphics, &cutoff_str, 200, y_offset + 30, 14.0);

                // Resonance
                draw_text_left(graphics, "Resonance:", 50, y_offset + 60, 14.0);
                draw_text_left(graphics, &resonance_str, 200, y_offset + 60, 14.0);

                // Draw instructions
                graphics.set_color(Color::rgb(120, 120, 120));
                draw_text_centered(
                    graphics,
                    "Use your DAW's automation to control parameters",
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
        (600, 400)
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

/// Draws the frequency response visualization
fn draw_frequency_response(
    graphics: &mut Graphics,
    frequency_response: &[AtomicF32; 128],
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) {
    // Draw background
    graphics.set_color(Color::rgb(30, 30, 35));
    graphics.fill_rect(x as i32, y as i32, width, height);

    // Draw grid lines
    graphics.set_color(Color::rgb(50, 50, 55));

    // Horizontal grid lines (dB levels)
    let db_levels = [-24.0, -12.0, 0.0, 12.0, 24.0];
    for &db in &db_levels {
        let grid_y = db_to_y(db, y, height);
        graphics.draw_line(x as i32, grid_y, (x + width) as i32, grid_y);
    }

    // Vertical grid lines (frequencies)
    let frequencies = [100.0, 1000.0, 10000.0];
    for &freq in &frequencies {
        let grid_x = freq_to_x(freq, x, width);
        graphics.draw_line(grid_x, y as i32, grid_x, (y + height) as i32);
    }

    // Draw frequency response curve
    graphics.set_color(Color::rgb(100, 200, 255));

    for i in 0..127 {
        let magnitude_db1 = frequency_response[i].load(Ordering::Relaxed);
        let magnitude_db2 = frequency_response[i + 1].load(Ordering::Relaxed);

        let t1 = i as f32 / 127.0;
        let t2 = (i + 1) as f32 / 127.0;

        let freq1 = 20.0_f32 * (20000.0_f32 / 20.0_f32).powf(t1);
        let freq2 = 20.0_f32 * (20000.0_f32 / 20.0_f32).powf(t2);

        let x1 = freq_to_x(freq1, x, width);
        let y1 = db_to_y(magnitude_db1, y, height);
        let x2 = freq_to_x(freq2, x, width);
        let y2 = db_to_y(magnitude_db2, y, height);

        graphics.draw_line(x1, y1, x2, y2);
    }

    // Draw frequency labels
    graphics.set_color(Color::rgb(150, 150, 150));
    let label_y = (y + height + 5) as i32;

    draw_text_centered(graphics, "20", freq_to_x(20.0, x, width), label_y, 10.0);
    draw_text_centered(graphics, "100", freq_to_x(100.0, x, width), label_y, 10.0);
    draw_text_centered(graphics, "1k", freq_to_x(1000.0, x, width), label_y, 10.0);
    draw_text_centered(graphics, "10k", freq_to_x(10000.0, x, width), label_y, 10.0);
    draw_text_centered(graphics, "20k", freq_to_x(20000.0, x, width), label_y, 10.0);

    // Draw dB labels
    let label_x = (x - 5) as i32;
    for &db in &[-24.0, 0.0, 24.0] {
        let label_str = format!("{:+.0}", db);
        draw_text_right(graphics, &label_str, label_x, db_to_y(db, y, height), 10.0);
    }
}

/// Converts frequency to x coordinate (logarithmic scale)
fn freq_to_x(freq: f32, x_offset: u32, width: u32) -> i32 {
    let min_freq = 20.0f32;
    let max_freq = 20000.0f32;

    let t = (freq.log10() - min_freq.log10()) / (max_freq.log10() - min_freq.log10());
    (x_offset as f32 + t * width as f32) as i32
}

/// Converts dB value to y coordinate
fn db_to_y(db: f32, y_offset: u32, height: u32) -> i32 {
    let min_db = -30.0;
    let max_db = 30.0;

    // Invert y axis (higher dB at top)
    let t = (db - min_db) / (max_db - min_db);
    (y_offset as f32 + height as f32 * (1.0 - t)) as i32
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

/// Helper function to draw right-aligned text
fn draw_text_right(graphics: &mut Graphics, text: &str, x: i32, y: i32, _size: f32) {
    let text_width = (text.len() * 8) as i32;
    let start_x = x - text_width;
    graphics.fill_rect(start_x, y, text_width as u32, 2);
}
