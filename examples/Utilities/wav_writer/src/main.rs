//! CLI demo: write a 1-second 440 Hz sine WAV file.

use wav_writer::{roundtrip_check, write_sine_wav};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let output_path = match args.get(1) {
        Some(p) => p.clone(),
        None => {
            eprintln!("Usage: wav_writer <output.wav>");
            std::process::exit(2);
        }
    };

    let sample_rate = 44_100.0_f32;
    let duration_secs = 1.0_f32;

    let (peak, num_frames) = match write_sine_wav(&output_path, sample_rate, duration_secs) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("⚠ Failed to write WAV: {}", e);
            std::process::exit(3);
        }
    };

    println!(
        "✓ Wrote {} ({} frames @ {} Hz, peak {:.4})",
        output_path, num_frames, sample_rate, peak
    );

    // Round-trip check.
    match roundtrip_check(&output_path, sample_rate, 1) {
        Ok((_, _, read_frames)) => {
            println!("✓ Round-trip OK: read back {} frames", read_frames);
        }
        Err(e) => {
            eprintln!("⚠ Round-trip mismatch: {}", e);
            std::process::exit(3);
        }
    }
}