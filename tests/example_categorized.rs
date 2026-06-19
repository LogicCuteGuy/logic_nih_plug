//! Workspace integration test: every example crate's `README.md` declares a
//! valid `category` front-matter line.
//!
//! Supports FR-001 (every example is categorized) and SC-008.

use std::fs;
use std::path::Path;

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

const VALID_CATEGORIES: &[&str] = &[
    "Audio", "DSP", "GUI", "Plugins", "Utilities", "DemoRunner",
];

fn collect_example_dirs() -> Vec<std::path::PathBuf> {
    let mut dirs = Vec::new();
    for prefix in ["plugins/examples", "examples"] {
        let root = workspace_root().join(prefix);
        if !root.is_dir() { continue; }
        walk(&root, &mut dirs);
    }
    dirs.sort();
    dirs
}

fn walk(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(it) => it,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.join("Cargo.toml").is_file() {
                out.push(path);
            } else {
                walk(&path, out);
            }
        }
    }
}

/// Extract the `category: <X>` front-matter value from a README's YAML block.
fn extract_category(text: &str) -> Option<String> {
    let mut in_front_matter = false;
    for line in text.lines() {
        if line.trim() == "---" {
            if !in_front_matter {
                in_front_matter = true;
                continue;
            } else {
                return None; // end of front matter without finding the category
            }
        }
        if in_front_matter {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("category:") {
                return Some(rest.trim().to_string());
            }
        }
    }
    None
}

#[test]
fn every_example_has_a_valid_category() {
    let dirs = collect_example_dirs();
    let mut missing = Vec::new();
    let mut invalid = Vec::new();

    for d in &dirs {
        let readme = d.join("README.md");
        if !readme.is_file() {
            // The `every_example_has_a_readme` test in example_readme_required.rs
            // covers this; we don't double-report.
            continue;
        }
        let text = match fs::read_to_string(&readme) {
            Ok(t) => t,
            Err(_) => continue,
        };
        match extract_category(&text) {
            None => missing.push(d.display().to_string()),
            Some(cat) if !VALID_CATEGORIES.contains(&cat.as_str()) => {
                invalid.push(format!("{}: unknown category `{}`", d.display(), cat))
            }
            Some(_) => {}
        }
    }

    let mut msg = String::new();
    if !missing.is_empty() {
        msg.push_str(&format!(
            "These examples are missing the `category:` front-matter field:\n  {}\n",
            missing.join("\n  ")
        ));
    }
    if !invalid.is_empty() {
        msg.push_str(&format!(
            "These examples have an invalid `category:` value:\n  {}\n",
            invalid.join("\n  ")
        ));
    }
    assert!(msg.is_empty(), "{}", msg);
}
