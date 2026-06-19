//! CLI demo: print a WAV file's header summary.
//!
//! Usage: `cargo run -p wav_reader -- <path-to.wav>`

use wav_reader::summarize_wav;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = match args.get(1) {
        Some(p) => p.clone(),
        None => {
            eprintln!("Usage: wav_reader <path.wav>");
            std::process::exit(2);
        }
    };

    let summary = match summarize_wav(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("⚠ {}", e);
            std::process::exit(3);
        }
    };

    println!("=== WAV file summary ===");
    println!("Path:               {}", path);
    println!("Sample rate:        {} Hz", summary.sample_rate);
    println!("Channels:           {}", summary.num_channels);
    println!("Bit depth:          {:?}", summary.bit_depth);
    println!("Frames:             {}", summary.num_frames);
    println!(
        "Duration:           {:.3} s",
        summary.duration_secs()
    );
    println!("========================");
    println!("Peak amplitude:     {:.4}", summary.peak_amplitude);
}