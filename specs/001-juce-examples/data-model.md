# Data Model: JUCE-Style Examples Portfolio

**Feature**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Date**: 2026-06-19

This file describes the entity-attribute model for the examples portfolio. It is the
source of truth for what "an example" *is* in the workspace; every new crate must
satisfy the schema below.

## 1. Entities

### 1.1 `Example`

The fundamental unit of the portfolio. A single Rust crate that ports one JUCE
example file (or, in special cases, a coherent group of files — see `WebViewPluginDemo`
in the [ledger](./example-inventory.md)).

| Attribute | Type | Required | Source-of-truth | Notes |
|---|---|---|---|---|
| `juce_path` | string | yes | [ledger](./example-inventory.md) | Path under `https://github.com/juce-framework/JUCE/tree/master/` |
| `juce_source_link` | URL | yes | ledger | Permalink to the file on `master` |
| `rust_crate` | string | yes | ledger + workspace `Cargo.toml` | Workspace member name (must be a valid Rust crate name and a path under `plugins/examples/*` or `examples/*`) |
| `category` | enum | yes | `Cargo.toml` + README front-matter | One of: `Audio`, `DSP`, `GUI`, `Plugins`, `Utilities`, `DemoRunner` |
| `kind` | enum | yes | `Cargo.toml` `crate-type` | `plugin` (`cdylib`), `standalone` (`[[bin]]`), `plugin-host` (`cdylib` + custom editor), `showcase` (`[[bin]]` + GUI) |
| `status` | enum | yes | ledger | `pending` → `ported` / `skipped(<module>)` / `deferred` / `existing` |
| `crate_path` | path | yes | workspace `Cargo.toml` | Absolute path under workspace root |
| `readme` | path | yes | contract | Top-level `README.md` with 5 FR-002 sections |
| `doc_tests` | list of inline `///` blocks | yes | `src/lib.rs` or `src/main.rs` | ≥1 doc-test per public type, per constitution "Testing/Documentation" |
| `juce_fidelity_checklist` | object | yes | [juce-fidelity-contract](./contracts/juce-fidelity-contract.md) | Per-crate instance of the constitution §V checklist |
| `bundler_entry` | path | conditional: `kind == plugin` | `bundler.toml` | `[<rust_crate>]` section with `name = "<Human Readable>"` |
| `assets` | list of paths | optional | per crate | Small reference fixtures (e.g. `assets/ir.wav`) checked in |
| `ci_smoke` | string | yes | per crate | `cargo xtask test-examples --category <C>` invocation that exercises it |
| `ci_dry_run_exit_code` | integer | yes | contract | One of `0` (success), `2` (env), `3` (malformed); see FR-006 |

### 1.2 `Category` (enum)

```text
Audio       // standalone apps demonstrating audio I/O
DSP         // plugins demonstrating DSP algorithms
GUI         // plugins demonstrating the GUI framework
Plugins     // plugins (or plugin hosts) demonstrating plugin-format features
Utilities   // standalone apps demonstrating cross-cutting utilities
DemoRunner  // a single showcase crate that exercises all of the above
```

### 1.3 `Kind` (enum)

```text
plugin       // crate-type = ["cdylib"]; exports VST3 + CLAP via nih_export_*
standalone   // [[bin]] target; demonstrates a non-plugin workflow
plugin-host  // crate-type = ["cdylib"]; the plugin *contains* a host for other plugins
showcase     // [[bin]] target; visual showcase, not a workload app
```

### 1.4 `Status` (enum)

```text
pending              // identified in the ledger; no Rust crate yet
ported               // crate exists, builds, doc-tests pass, bundler row updated
skipped(<module>)    // a JUCE module the example depends on is not yet ported; never silent
deferred             // explicitly out of scope this iteration (rationale in ledger note)
existing             // crate pre-dates this feature; README is updated with category front-matter
```

### 1.5 `Format(s)` (set of strings)

The plugin formats the crate bundles to. Per the existing convention:

```text
VST3         // every plugin example bundles VST3 by default
CLAP         // every plugin example bundles CLAP by default
AU           // macOS only; not used in default workspace
AUv3         // macOS/iOS only; not used in default workspace
LV2          // Linux only; not used in default workspace
AAX          // stub; not used in default workspace
VST2         // optional; requires local Steinberg SDK; not used in default workspace
Standalone   // [[bin]]; uses logic_nih_plug's standalone feature
```

The `[[bin]]` *host* crate (`juce_audio_plugin_host_egui`) bundles **both** as a VST3
plugin (`cdylib`) and as a standalone binary (`[[bin]]`) so the same code can be
loaded as a VST3 host in a DAW or run as a desktop application — see FR-006's
exception case and the `plugin_host_egui` ledger row.

## 2. Relationships

```text
Category  1 ────< Example        // each Example belongs to exactly one Category
Kind      1 ────< Example        // each Example is one Kind
Status    1 ────< Example        // each Example has one Status at any moment
Example   1 ────1 BundleConfig   // plugin examples only; matches bundler.toml
Example   1 ────< DocTest        // ≥1 doc-test per public type
Example   1 ────1 Readme         // 5-section README per FR-002
Example   1 ────< Asset          // optional; checked-in reference fixtures
```

## 3. Validation rules (enforced by workspace integration tests)

| Rule | Test | Enforced by |
|---|---|---|
| Every new crate under `plugins/examples/` and `examples/` has a `README.md` | `tests/example_readme_required.rs` (T016) | `cargo test --workspace` |
| README contains all 5 FR-002 sections | `tests/example_readme_required.rs` (T016) | same |
| README declares `category = "..."` front-matter | `tests/example_categorized.rs` (T017) | same |
| `category` ∈ {`Audio`,`DSP`,`GUI`,`Plugins`,`Utilities`,`DemoRunner`} | `tests/example_categorized.rs` (T017) | same |
| Every `#[id = "..."]` / `VST3_CLASS_ID` / `CLAP_ID` is workspace-unique | `tests/id_uniqueness.rs` (T015) | same |
| Every plugin example has a `bundler.toml` row | `cargo xtask known-packages` (existing) | pre-existing xtask |
| Every example in the ledger has a non-`pending` status by Phase 8 | manual review + CI grep | T094 |
| `process()` is alloc-free for every plugin example | `assert_process_allocs` feature in default build | per-example `Cargo.toml` |

## 4. Examples of the schema in action

A complete row from the ledger (US1, the first DSP example to be ported):

```text
juce_path:           examples/DSP/OverdriveDemo.h
juce_source_link:    https://github.com/juce-framework/JUCE/blob/master/examples/DSP/OverdriveDemo.h
rust_crate:          juce_distortion_demo
category:            DSP
kind:                plugin
status:              pending        // → ported after T029-T031
crate_path:          plugins/examples/dsp/juce_distortion_demo/
readme:              plugins/examples/dsp/juce_distortion_demo/README.md
doc_tests:           ["overdrive sample count + peak reduction"]
juce_fidelity_checklist:
  source_file_named:    ✓
  matches_juce_behavior: ✓ (per constitution §V)
  skipped_modules:      none
bundler_entry:        bundler.toml → [juce_distortion_demo] name = "JUCE Distortion Demo"
assets:               none
ci_smoke:             cargo xtask test-examples --category DSP
ci_dry_run_exit_code: 0
```

A standalone app example (US2, audio playback):

```text
juce_path:           examples/Audio/AudioPlaybackDemo.h
juce_source_link:    https://github.com/juce-framework/JUCE/blob/master/examples/Audio/AudioPlaybackDemo.h
rust_crate:          audio_playback_demo
category:            Audio
kind:                standalone
status:              pending        // → ported after T043-T046
crate_path:          examples/Audio/audio_playback_demo/
readme:              examples/Audio/audio_playback_demo/README.md
doc_tests:           ["MockAudioIODevice event_log shows Opened→Started→Stopped→Closed"]
juce_fidelity_checklist:
  source_file_named:    ✓
  matches_juce_behavior: ✓
  skipped_modules:      none
bundler_entry:        n/a           (standalone has no plugin bundle)
assets:               examples/audio-assets/sine_1khz_1s.wav
ci_smoke:             cargo run -p audio_playback_demo -- examples/audio-assets/sine_1khz_1s.wav
ci_dry_run_exit_code: 0
```

## 5. Where the schema lives

| Layer | File |
|---|---|
| Authoritative ledger (per-example status) | `specs/001-juce-examples/example-inventory.md` |
| Schema (this file) | `specs/001-juce-examples/data-model.md` |
| Per-crate conventions | `specs/001-juce-examples/contracts/example-crate-contract.md` |
| Per-crate fidelity checklist | `specs/001-juce-examples/contracts/juce-fidelity-contract.md` |
| Validation (Rust) | `tests/id_uniqueness.rs`, `tests/example_readme_required.rs`, `tests/example_categorized.rs` |
