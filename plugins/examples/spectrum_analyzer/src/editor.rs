//! GUI editor for the spectrum analyzer plugin.
//!
//! This module implements the visual display of the spectrum analyzer,
//! including the spectrogram with color mapping and frequency/magnitude axes.

use atomic_float::AtomicF32;
use crossbeam::atomic::AtomicCell;
use nih_plug::prelude::*;
use nih_plug_graphics::{Color, Graphics};
use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::{AnalyzerParams, DISPLAY_BINS, SPECTROGRAM_HISTORY};

/// Editor for the spectrum analyzer plugin
pub struct SpectrumAnalyzerEditor {
    pub params: Arc<AnalyzerParams>,
    pub spectrum_data: Arc<[AtomicF32; DISPLAY_BINS]>,
    pub spectrogram_data: Arc<[[AtomicF32; DISPLAY_BINS]; SPECTROGRAM_HISTORY]>,
    pub scaling_factor: AtomicCell<Option<f32>>,
}

impl Editor for SpectrumAnalyzerEditor {
    fn spawn(
        &self,
        _parent: nih_plug::editor::ParentWindowHandle,
        _context: Arc<dyn GuiContext>,
    ) -> Box<dyn std::any::Any + Send> {
        let params = Arc::clone(&self.params);
        let spectrum_data = Arc::clone(&self.spectrum_data);
        let spectrogram_data = Arc::clone(&self.spectrogram_data);

        // Create GL window with custom rendering
        let window = nih_plug_gui::GlWindowBuilder::spawn(
            "Spectrum Analyzer",
            800,
            600,
            move |graphics: &mut Graphics, (w, h)| {
                draw_spectrum_analyzer(graphics, &params, &spectrum_data, &spectrogram_data, w, h);
            },
        );

        struct EditorHandle {
            window: nih_plug_gui::GlWindow,
        }
        unsafe impl Send for EditorHandle {}

        Box::new(EditorHandle { window })
    }

    fn size(&self) -> (u32, u32) {
        (800, 600)
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

/// Main drawing function for the spectrum analyzer
fn draw_spectrum_analyzer(
    graphics: &mut Graphics,
    params: &Arc<AnalyzerParams>,
    spectrum_data: &Arc<[AtomicF32; DISPLAY_BINS]>,
    spectrogram_data: &Arc<[[AtomicF32; DISPLAY_BINS]; SPECTROGRAM_HISTORY]>,
    width: u32,
    height: u32,
) {
    // Clear background
    graphics.set_color(Color::rgb(20, 20, 25));
    graphics.clear();

    // Define layout areas
    let margin = 40;
    let spectrum_height = 150;
    let spectrogram_top = margin + spectrum_height + 20;
    let spectrogram_height = height.saturating_sub(spectrogram_top + margin);

    // Draw title
    graphics.set_color(Color::rgb(220, 220, 220));
    draw_text_centered(graphics, "Spectrum Analyzer", width as i32 / 2, 15, 20.0);

    // Draw spectrum display (top)
    draw_spectrum(
        graphics,
        params,
        spectrum_data,
        margin,
        margin,
        width - 2 * margin,
        spectrum_height,
    );

    // Draw spectrogram display (bottom)
    draw_spectrogram(
        graphics,
        params,
        spectrogram_data,
        margin,
        spectrogram_top,
        width - 2 * margin,
        spectrogram_height,
    );

    // Draw axes and labels
    draw_axes(graphics, params, margin, width, height);
}

/// Draws the real-time magnitude spectrum
fn draw_spectrum(
    graphics: &mut Graphics,
    params: &Arc<AnalyzerParams>,
    spectrum_data: &Arc<[AtomicF32; DISPLAY_BINS]>,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) {
    // Draw background
    graphics.set_color(Color::rgb(30, 30, 35));
    graphics.fill_rect(x as i32, y as i32, width, height);

    // Get display range
    let min_db = params.min_db.value();
    let max_db = params.max_db.value();
    let db_range = max_db - min_db;

    // Draw spectrum line
    graphics.set_color(Color::rgb(100, 200, 255));

    for i in 0..(DISPLAY_BINS - 1) {
        let db1 = spectrum_data[i].load(Ordering::Relaxed);
        let db2 = spectrum_data[i + 1].load(Ordering::Relaxed);

        // Map frequency bin to x position (logarithmic)
        let freq_norm1 = ((i as f32 / DISPLAY_BINS as f32).max(0.001)).log10() / (-3.0_f32).log10();
        let freq_norm2 = (((i + 1) as f32 / DISPLAY_BINS as f32).max(0.001)).log10() / (-3.0_f32).log10();

        let x1 = x as f32 + width as f32 * freq_norm1;
        let x2 = x as f32 + width as f32 * freq_norm2;

        // Map dB to y position
        let db_norm1 = ((db1 - min_db) / db_range).clamp(0.0, 1.0);
        let db_norm2 = ((db2 - min_db) / db_range).clamp(0.0, 1.0);

        let y1 = y as f32 + height as f32 * (1.0 - db_norm1);
        let y2 = y as f32 + height as f32 * (1.0 - db_norm2);

        graphics.draw_line(x1 as i32, y1 as i32, x2 as i32, y2 as i32);
    }

    // Draw border (using lines since there's no draw_rect)
    graphics.set_color(Color::rgb(80, 80, 90));
    let x1 = x as i32;
    let y1 = y as i32;
    let x2 = (x + width) as i32;
    let y2 = (y + height) as i32;
    graphics.draw_line(x1, y1, x2, y1); // Top
    graphics.draw_line(x2, y1, x2, y2); // Right
    graphics.draw_line(x2, y2, x1, y2); // Bottom
    graphics.draw_line(x1, y2, x1, y1); // Left
}

/// Draws the spectrogram with color mapping
fn draw_spectrogram(
    graphics: &mut Graphics,
    params: &Arc<AnalyzerParams>,
    spectrogram_data: &Arc<[[AtomicF32; DISPLAY_BINS]; SPECTROGRAM_HISTORY]>,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) {
    // Get display range
    let min_db = params.min_db.value();
    let max_db = params.max_db.value();
    let db_range = max_db - min_db;

    // Calculate pixel sizes
    let pixel_width = (width as f32 / SPECTROGRAM_HISTORY as f32).ceil() as u32;
    let pixel_height = (height as f32 / DISPLAY_BINS as f32).ceil() as u32;

    // Draw spectrogram pixels
    for time_idx in 0..SPECTROGRAM_HISTORY {
        for freq_idx in 0..DISPLAY_BINS {
            let db = spectrogram_data[time_idx][freq_idx].load(Ordering::Relaxed);

            // Normalize dB to [0, 1]
            let normalized = ((db - min_db) / db_range).clamp(0.0, 1.0);

            // Map to color (blue -> cyan -> green -> yellow -> red)
            let color = value_to_color(normalized);

            // Calculate pixel position
            let px = x + time_idx as u32 * pixel_width;
            let py = y + height - (freq_idx as u32 + 1) * pixel_height;

            // Draw pixel
            graphics.set_color(color);
            graphics.fill_rect(px as i32, py as i32, pixel_width, pixel_height);
        }
    }

    // Draw border (using lines since there's no draw_rect)
    graphics.set_color(Color::rgb(80, 80, 90));
    let x1 = x as i32;
    let y1 = y as i32;
    let x2 = (x + width) as i32;
    let y2 = (y + height) as i32;
    graphics.draw_line(x1, y1, x2, y1); // Top
    graphics.draw_line(x2, y1, x2, y2); // Right
    graphics.draw_line(x2, y2, x1, y2); // Bottom
    graphics.draw_line(x1, y2, x1, y1); // Left
}

/// Draws frequency and magnitude axes with labels
fn draw_axes(
    graphics: &mut Graphics,
    params: &Arc<AnalyzerParams>,
    margin: u32,
    width: u32,
    height: u32,
) {
    graphics.set_color(Color::rgb(150, 150, 150));

    // Draw frequency labels (logarithmic: 20Hz, 100Hz, 1kHz, 10kHz, 20kHz)
    let freq_labels = [
        (20.0, "20Hz"),
        (100.0, "100Hz"),
        (1000.0, "1kHz"),
        (10000.0, "10kHz"),
        (20000.0, "20kHz"),
    ];

    let nyquist = 22050.0_f32; // Assume 44.1kHz sample rate
    for (freq, label) in freq_labels {
        if freq <= nyquist {
            let freq_norm = ((freq / nyquist).max(0.001_f32)).log10() / (-3.0_f32).log10();
            let x_pos = margin as f32 + (width - 2 * margin) as f32 * freq_norm;

            draw_text_centered(graphics, label, x_pos as i32, (height - 20) as i32, 10.0);
        }
    }

    // Draw magnitude labels
    let min_db = params.min_db.value();
    let max_db = params.max_db.value();

    graphics.set_color(Color::rgb(150, 150, 150));
    draw_text_left(graphics, &format!("{:.0} dB", max_db), 5, (margin + 5) as i32, 10.0);
    draw_text_left(graphics, &format!("{:.0} dB", min_db), 5, (margin + 145) as i32, 10.0);
}

/// Maps a normalized value [0, 1] to a color using a heat map
fn value_to_color(value: f32) -> Color {
    // Heat map: black -> blue -> cyan -> green -> yellow -> red
    let v = value.clamp(0.0, 1.0);

    if v < 0.2 {
        // Black to blue
        let t = v / 0.2;
        Color::rgb(0, 0, (t * 255.0) as u8)
    } else if v < 0.4 {
        // Blue to cyan
        let t = (v - 0.2) / 0.2;
        Color::rgb(0, (t * 255.0) as u8, 255)
    } else if v < 0.6 {
        // Cyan to green
        let t = (v - 0.4) / 0.2;
        Color::rgb(0, 255, ((1.0 - t) * 255.0) as u8)
    } else if v < 0.8 {
        // Green to yellow
        let t = (v - 0.6) / 0.2;
        Color::rgb((t * 255.0) as u8, 255, 0)
    } else {
        // Yellow to red
        let t = (v - 0.8) / 0.2;
        Color::rgb(255, ((1.0 - t) * 255.0) as u8, 0)
    }
}

/// Helper function to draw centered text (simplified - no actual font rendering)
fn draw_text_centered(graphics: &mut Graphics, text: &str, x: i32, y: i32, _size: f32) {
    // Approximate text width (6 pixels per character)
    let text_width = (text.len() * 6) as i32;
    let start_x = x - text_width / 2;

    // Draw simple text representation (placeholder)
    graphics.fill_rect(start_x, y, text_width as u32, 2);
}

/// Helper function to draw left-aligned text
fn draw_text_left(graphics: &mut Graphics, text: &str, x: i32, y: i32, _size: f32) {
    let text_width = (text.len() * 6) as u32;
    graphics.fill_rect(x, y, text_width, 2);
}
