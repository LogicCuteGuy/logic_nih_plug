//! Workspace integration test: every example crate has a top-level `README.md`
//! containing the five FR-002 sections.
//!
//! Supports SC-008 (the examples portfolio is uniform and discoverable).

use std::fs;
use std::path::Path;

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// Every crate under `plugins/examples/*` and `examples/*` that is a workspace
/// member (per `Cargo.toml`) must have a `README.md`.
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
            // Only descend into directories that look like a crate (have a Cargo.toml).
            if path.join("Cargo.toml").is_file() {
                out.push(path);
            } else {
                walk(&path, out);
            }
        }
    }
}

const REQUIRED_SECTIONS: &[&str] = &[
    "## What this example ports",
    "## Parameters",
    "## Building",
    "## Running the doc-tests",
    "## References",
    "## JUCE fidelity checklist",
];

#[test]
fn every_example_has_a_readme() {
    let dirs = collect_example_dirs();
    let mut missing = Vec::new();
    for d in &dirs {
        if !d.join("README.md").is_file() {
            missing.push(d.display().to_string());
        }
    }
    assert!(
        missing.is_empty(),
        "These example crates are missing a top-level README.md:\n  {}",
        missing.join("\n  ")
    );
}

#[test]
fn every_readme_has_all_required_sections() {
    let dirs = collect_example_dirs();
    let mut violations = Vec::new();
    for d in &dirs {
        let readme_path = d.join("README.md");
        if !readme_path.is_file() { continue; }
        let text = match fs::read_to_string(&readme_path) {
            Ok(t) => t,
            Err(_) => continue,
        };
        for section in REQUIRED_SECTIONS {
            if !text.contains(section) {
                violations.push(format!(
                    "  - {} is missing section `{}`",
                    d.display(),
                    section
                ));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "The following README.md files are missing required sections:\n{}",
        violations.join("\n")
    );
}
