//! CLI demo: parse a Standard MIDI File and print its metadata.
//!
//! Usage: `cargo run -p midi_file_inspector -- <path-to.mid>`

use midi_file_inspector::summarize_midi_file;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = match args.get(1) {
        Some(p) => p.clone(),
        None => {
            eprintln!("Usage: midi_file_inspector <path.mid>");
            std::process::exit(2);
        }
    };

    let summary = match summarize_midi_file(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("⚠ {}", e);
            std::process::exit(3);
        }
    };

    println!("=== MIDI file summary ===");
    println!("Path:                  {}", path);
    println!("Format:                {:?}", summary.format);
    println!("Ticks per quarter:     {}", summary.ticks_per_quarter_note);
    println!("Tracks:                {}", summary.num_tracks);
    println!("Total events:          {}", summary.total_events);
    if let Some(micros) = summary.first_tempo_micros_per_quarter {
        let bpm = 60_000_000 / micros;
        println!("Tempo (first):         {} µs/qn (~{} BPM)", micros, bpm);
    }
    if let Some((num, den)) = summary.time_signature {
        println!("Time signature:        {}/{}", num, den);
    }
    println!("==========================");
}