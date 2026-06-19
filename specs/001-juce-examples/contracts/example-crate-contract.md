# Example-Crate Contract

**Feature**: [spec.md](../spec.md) | **Date**: 2026-06-19

Every new example crate in the portfolio MUST follow this contract. The contract is
enforced by `tests/example_readme_required.rs`, `tests/example_categorized.rs`, and
`tests/id_uniqueness.rs` at the workspace level.

---

## 1. `Cargo.toml` conventions

### 1.1 Plugin example (kind = `plugin`)

```toml
[package]
name = "<juce_rust_crate_name>"        # matches the ledger `rust_crate` column
version = "0.1.0"
edition = "2021"
authors = ["LogicCuteGuy <contact@logiccuteguy.com>", "NIH-plug Contributors"]
license = "ISC"
description = "<one-line summary of what JUCE file this ports>"

[lib]
crate-type = ["cdylib"]

[dependencies]
logic_nih_plug = { path = "../../../", features = ["assert_process_allocs"] }
# + only the sub-crates the example needs (dsp, gui, egui, data, etc.)
```

- `assert_process_allocs` is on by default for plugin examples; this is the CI gate
  for real-time safety (constitution G1).
- `crate-type = ["cdylib"]` is required for `cargo xtask bundle <name>`.

### 1.2 Standalone app example (kind = `standalone`)

```toml
[package]
name = "<juce_rust_crate_name>"
version = "0.1.0"
edition = "2021"
authors = ["LogicCuteGuy <contact@logiccuteguy.com>", "NIH-plug Contributors"]
license = "ISC"
description = "<one-line summary>"

[lib]
# No `crate-type = ["cdylib"]`; standalone apps don't ship as plugins.

[[bin]]
name = "<juce_rust_crate_name>"
path = "src/main.rs"

[dependencies]
# + whatever sub-crates the example needs
```

### 1.3 Plugin host (kind = `plugin-host`)

```toml
[package]
name = "<juce_rust_crate_name>"
version = "0.1.0"
edition = "2021"
authors = ["LogicCuteGuy <contact@logiccuteguy.com>", "NIH-plug Contributors"]
license = "ISC"
description = "<one-line summary>"

[lib]
crate-type = ["cdylib"]   # the host bundles as a VST3 plugin

[[bin]]
name = "<juce_rust_crate_name>_host"
path = "src/bin/host.rs"  # the host also runs as a standalone desktop app

[dependencies]
logic_nih_plug = { path = "../../../", features = ["assert_process_allocs"] }
logic_nih_plug_audio_devices = { path = "../../../logic_nih_plug_audio_devices" }
logic_nih_plug_audio_processors = { path = "../../../logic_nih_plug_audio_processors" }
# + egui for the GUI backend (per Q2 recommended default)
```

### 1.4 Showcase (kind = `showcase`)

```toml
[package]
name = "juce_demorunner"
version = "0.1.0"
edition = "2021"
authors = ["LogicCuteGuy <contact@logiccuteguy.com>", "NIH-plug Contributors"]
license = "ISC"
description = "Showcase of every public GUI component from logic_nih_plug_gui"

[features]
default = ["gui-egui"]
gui-egui = ["dep:logic_nih_plug_egui"]
gui-iced = ["dep:logic_nih_plug_iced"]
gui-vizia = ["dep:logic_nih_plug_vizia"]

[[bin]]
name = "juce_demorunner"
path = "src/main.rs"

[dependencies]
logic_nih_plug = { path = "../../" }
logic_nih_plug_gui = { path = "../../logic_nih_plug_gui" }
logic_nih_plug_egui = { path = "../../logic_nih_plug_egui", optional = true }
logic_nih_plug_iced = { path = "../../logic_nih_plug_iced", optional = true }
logic_nih_plug_vizia = { path = "../../logic_nih_plug_vizia", optional = true }
```

A `compile_error!` in `src/main.rs` enforces that exactly one of `gui-egui`,
`gui-iced`, `gui-vizia` is enabled.

## 2. `README.md` (FR-002, mandatory)

Every new crate ships a top-level `README.md` with these five sections, in this
order. The section headings are matched literally by `tests/example_readme_required.rs`:

```markdown
# <crate name> — <one-line summary>

## What this example ports

- **JUCE source file**: [`<path>`](<juce_source_link>)
- **What to learn from this example**: <1-3 sentence summary>

## Parameters

<bullet list of every parameter, its ID, and its range>

## Building

```bash
cargo xtask bundle <crate name> --release   # plugin examples
cargo run -p <crate name> -- <args>         # standalone examples
```

## Running the doc-tests

```bash
cargo test --doc -p <crate name>
```

## References

- [JUCE source](<juce_source_link>)
- [Workspace AGENTS.md](../../../../AGENTS.md)
- [Spec & plan](../../../../specs/001-juce-examples/)
```

Plus a YAML front-matter block at the very top:

```markdown
---
category: <Audio|DSP|GUI|Plugins|Utilities|DemoRunner>
juce_source: <path under examples/>
---
```

`tests/example_categorized.rs` (T017) parses the front-matter and rejects unknown
categories.

## 3. `src/lib.rs` (or `src/main.rs`)

### 3.1 Identity helper

Every plugin example uses a shared identity macro pattern. The IDs are derived
from the crate name and follow the workspace's prefix scheme:

```rust
// In src/lib.rs
use logic_nih_plug::prelude::*;

pub const VST3_CLASS_ID: [u8; 16] = *b"co.lnp.<category>.<example>.v3";
pub const CLAP_ID: &'static str = "co.logiccuteguy.<category>.<example>";

nih_export_vst3!(<CrateName>);
nih_export_clap!(<CrateName>);
```

A workspace integration test (`tests/id_uniqueness.rs`, T015) parses every crate's
`Cargo.toml` name + `lib.rs` for these strings and asserts no collisions.

### 3.2 Doc-test

Every public type in the example has **at least one doc-test** that exercises a
non-trivial behavior. The doc-test is inline `///` rustdoc code, runs under
`cargo test --doc -p <crate>`, and serves as the example's correctness gate.

Example pattern:

```rust
/// Apply soft-clipping to a buffer in place.
///
/// # Examples
///
/// ```
/// use juce_distortion_demo::soft_clip;
/// let mut buf = vec![0.0_f32; 1024];
/// soft_clip(&mut buf, 0.5);
/// let peak = buf.iter().fold(0.0_f32, |a, b| a.max(b.abs()));
/// assert!(peak < 0.5);     // peak reduced from 1.0 to <0.5 by soft-clip at drive=0.5
/// ```
pub fn soft_clip(buf: &mut [f32], drive: f32) { /* ... */ }
```

## 4. `bundler.toml` (plugin examples only)

Plugin examples register a row in the workspace's `bundler.toml`:

```toml
[<rust_crate>]
name = "<Human Readable Plugin Name>"
```

`cargo xtask known-packages` (existing) lists every registered plugin; SC-008
asserts every new example appears in that list.

## 5. CI smoke-test

Each example's `README.md` documents the exact `cargo xtask test-examples
--category <C>` invocation that exercises it. The `--category` filter is the
primary mechanism for per-user-story CI gates.

## 6. Exit codes (standalone examples, FR-006)

Standalone apps document a deterministic exit-code contract in their `README.md`:

| Code | Meaning |
|---|---|
| 0  | Success |
| 2  | Environment error (missing input file, no audio device, etc.) |
| 3  | Malformed input (bad WAV header, truncated SMF, etc.) |
| ≥4 | Implementation-defined; the example should still print a clear error message |

## 7. What is *not* part of this contract

- **No new workspace dependencies** (FR-012). Use only the existing
  `logic_nih_plug*` sub-crate tree plus the standard set declared in the root
  `Cargo.toml` workspace table.
- **No `#[id]` collisions** with existing plugins in the workspace
  (`tests/id_uniqueness.rs` enforces this).
- **No `process()` allocations, locks, or `println!`** (constitution G1, G6).
- **No VST2 + AAX combination** in the same crate (constitution G4).
- **No silent omissions** from the ledger. If an example can't be ported, the
  ledger row is `skipped(<module>)` or `deferred` with rationale.
