//! Standalone audio playback demo.
//!
//! Reads a WAV file (or generates a 1 kHz sine) and plays it through
//! a `MockAudioIODevice`, demonstrating the JUCE-style audio device
//! lifecycle.

use audio_playback_demo::{generate_sine, read_wav_file, PlaybackCapture};
use logic_nih_plug_audio_devices::{
    AudioDeviceManager, AudioDeviceSetup, MockAudioIODevice,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let wav_path = args.get(1).map(|s| s.as_str());

    let sample_rate = 44_100.0_f64;
    let buffer_size = 512u32;

    // Build audio data.
    let _audio_data = if let Some(path) = wav_path {
        match read_wav_file(path) {
            Ok(channels) => {
                println!(
                    "✓ Read WAV: {} channels, {} frames",
                    channels.len(),
                    channels.first().map_or(0, |c| c.len())
                );
                channels
            }
            Err(e) => {
                eprintln!("⚠ Could not read WAV ({}), generating sine instead", e);
                vec![generate_sine(sample_rate, 1000.0, 1.0)]
            }
        }
    } else {
        println!("No WAV path given — generating 1 kHz sine (1.0 s)");
        vec![generate_sine(sample_rate, 1000.0, 1.0)]
    };

    // Set up device manager with mock device.
    let mut manager = AudioDeviceManager::new();
    let setup = AudioDeviceSetup {
        sample_rate: sample_rate as u32,
        buffer_size,
        input_channels: 0,
        output_channels: 2,
        ..Default::default()
    };
    let _ = manager.set_audio_device_setup(setup);

    let mock = MockAudioIODevice::stereo_44100();
    manager.set_current_audio_device(Some(Box::new(mock)));

    // Lifecycle: open → play → stop → close.
    if let Err(e) = manager.open_device() {
        eprintln!("Failed to open device: {}", e);
        std::process::exit(3);
    }

    let _capture = PlaybackCapture::new();
    if let Err(e) = manager.play() {
        eprintln!("Failed to start playback: {}", e);
        std::process::exit(3);
    }

    manager.stop();
    manager.close_device();

    let state = manager.get_state();
    println!("✓ Playback complete — device state: {:?}", state);
    println!("✓ Played {:.2} s", 1.0);
}
