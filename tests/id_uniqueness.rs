//! Workspace integration test: every plugin example's `#[id = "..."]` /
//! `VST3_CLASS_ID` / `CLAP_ID` strings must be unique within the workspace.
//!
//! Supports SC-008 (every example's identifiers are unique).
//!
//! This test is part of the `cargo test --workspace` invocation; it does NOT
//! build any plugin. It statically parses every `Cargo.toml` and `src/lib.rs`
//! in `plugins/examples/*/` and `examples/*/` and asserts no identifier
//! collisions.

use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Workspace root, resolved from `CARGO_MANIFEST_DIR` (this test crate's dir).
fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn collect_plugin_crates() -> Vec<std::path::PathBuf> {
    let mut crates = Vec::new();
    for prefix in ["plugins/examples", "examples"] {
        let dir = workspace_root().join(prefix);
        if !dir.is_dir() { continue; }
        walk_for_cargo_tomls(&dir, &mut crates);
    }
    crates.sort();
    crates
}

fn walk_for_cargo_tomls(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(it) => it,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_for_cargo_tomls(&path, out);
        } else if path.file_name().map(|n| n == "Cargo.toml").unwrap_or(false) {
            // Only treat as a plugin crate if it declares `crate-type = ["cdylib"]`
            if let Ok(text) = fs::read_to_string(&path) {
                if text.contains("cdylib") {
                    out.push(path);
                }
            }
        }
    }
}

/// Extract `name = "..."` from a crate's `Cargo.toml` (first match in `[package]`).
fn crate_name(cargo_toml: &Path) -> Option<String> {
    let text = fs::read_to_string(cargo_toml).ok()?;
    let mut in_package = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_package = trimmed == "[package]";
            continue;
        }
        if in_package && trimmed.starts_with("name") {
            if let Some(start) = trimmed.find('"') {
                if let Some(end) = trimmed[start + 1..].find('"') {
                    return Some(trimmed[start + 1..start + 1 + end].to_string());
                }
            }
        }
    }
    None
}

/// Extract all string literals from a crate's `src/lib.rs` and `src/main.rs` that
/// look like identifiers (must contain a dot, must not be a `use` path).
fn extract_identifiers(src_dir: &Path) -> HashSet<String> {
    let mut ids = HashSet::new();
    let candidates = ["lib.rs", "main.rs", "bin/host.rs"];
    for rel in candidates {
        let path = src_dir.join(rel);
        if let Ok(text) = fs::read_to_string(&path) {
            for line in text.lines() {
                // Look for lines that look like:
                //   pub const VST3_CLASS_ID: [u8; 16] = *b"...16-byte-string...";
                //   pub const CLAP_ID: &'static str = "...";
                //   #[id = "..."]
                let line = line.trim();
                let value = if let Some(idx) = line.find("CLAP_ID") {
                    extract_string_after(&line[idx..])
                } else if let Some(idx) = line.find("VST3_CLASS_ID") {
                    extract_string_after(&line[idx..])
                } else if line.starts_with("#[id") {
                    extract_string_after(line)
                } else {
                    None
                };
                if let Some(s) = value {
                    if s.contains('.') || s.len() >= 8 {
                        ids.insert(s);
                    }
                }
            }
        }
    }
    ids
}

fn extract_string_after(s: &str) -> Option<String> {
    let first = s.find('"')?;
    let rest = &s[first + 1..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

#[test]
fn no_plugin_identifier_collisions() {
    let crates = collect_plugin_crates();
    let mut all_ids: Vec<(String, String)> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for cargo_toml in &crates {
        let name = match crate_name(cargo_toml) {
            Some(n) => n,
            None => continue,
        };
        let src_dir = cargo_toml.parent().unwrap().join("src");
        let ids = extract_identifiers(&src_dir);
        for id in &ids {
            if !seen.insert(id.clone()) {
                // Find the previous owner for a helpful error message.
                let prev = all_ids.iter().find(|(i, _)| i == id).map(|(_, c)| c.clone());
                panic!(
                    "Identifier `{id}` is declared by `{name}` (in `{}`) but was already declared by `{}`.",
                    cargo_toml.display(),
                    prev.unwrap_or_else(|| "<unknown>".to_string())
                );
            }
            all_ids.push((id.clone(), name.clone()));
        }
    }

    assert!(
        !all_ids.is_empty(),
        "Expected at least one plugin example to scan; found none. \
         (Did `plugins/examples/` and `examples/` lose their plugin crates?)"
    );
}

#[test]
fn every_plugin_crate_has_a_distinct_name() {
    let crates = collect_plugin_crates();
    let mut names: HashSet<String> = HashSet::new();
    for cargo_toml in &crates {
        if let Some(name) = crate_name(cargo_toml) {
            assert!(
                names.insert(name.clone()),
                "Plugin crate name `{name}` is duplicated (in `{}`).",
                cargo_toml.display()
            );
        }
    }
}
