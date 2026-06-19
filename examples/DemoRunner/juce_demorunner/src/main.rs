//! Headless driver for the DemoRunner showcase.
//!
//! The actual GUI window is opened by the chosen backend's runtime
//! (egui / iced / vizia). This binary just prints the resolved
//! backend + a summary of the registered demos, so a CI smoke test
//! can assert the registry is wired up correctly.

use juce_demorunner::{
    backend::ActiveBackend,
    showcase::{
        animation, audio_viz, controls, graphics, layouts, ShowcaseCategory,
    },
};

fn main() {
    let backend = ActiveBackend::resolve();
    println!("✓ DemoRunner active backend: {}", backend.kind.name());

    for cat in ShowcaseCategory::all() {
        let demos = match cat {
            ShowcaseCategory::Controls => controls::registered(),
            ShowcaseCategory::Layouts => layouts::registered(),
            ShowcaseCategory::Animation => animation::registered(),
            ShowcaseCategory::Graphics => graphics::registered(),
            ShowcaseCategory::AudioViz => audio_viz::registered(),
        };
        println!("  {}: {} demo(s)", cat.name(), demos.len());
        for d in &demos {
            println!("    • {} — {}", d.title, d.description);
        }
    }
}