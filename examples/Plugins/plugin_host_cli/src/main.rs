//! Headless plugin host CLI.
//!
//! Scans one or more directories for plugins and prints the discovered
//! descriptions. Exit codes: 0 = success, 2 = bad args, 3 = scan error.

use std::path::PathBuf;

use plugin_host_cli::{format_plugin_list, MockPluginFormat};
use logic_nih_plug_audio_processors::{NullPluginFormat, PluginFormatType};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let dir_arg = args.get(1).map(|s| s.as_str()).unwrap_or(".");

    let dir = PathBuf::from(dir_arg);
    if !dir.exists() {
        eprintln!("⚠ Directory does not exist: {}", dir.display());
        std::process::exit(2);
    }

    println!("Scanning {} for VST3 plugins…", dir.display());

    // Try the mock format first (so empty fixtures dirs still report
    // "no plugins"), then fall back to NullPluginFormat.
    let mock = MockPluginFormat::new(PluginFormatType::Vst3);
    let list = plugin_host_cli::scan_directories(&mock, &[dir.clone()], false);

    let formatted = format_plugin_list(&list);
    if formatted.is_empty() {
        println!("(no plugins found)");
        // Also confirm NullPluginFormat yields nothing — sanity check.
        let null = NullPluginFormat::new(PluginFormatType::Vst3);
        let null_list = plugin_host_cli::scan_directories(&null, &[dir], false);
        if !null_list.get_types().is_empty() {
            eprintln!("⚠ NullPluginFormat unexpectedly returned plugins");
            std::process::exit(3);
        }
    } else {
        print!("{}", formatted);
    }

    println!("✓ Scan complete ({} plugins)", list.get_types().len());
}
