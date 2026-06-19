# Research: JUCE-Style Examples Portfolio

**Feature**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Date**: 2026-06-19
**Source**: [juce-framework/JUCE `examples/`](https://github.com/juce-framework/JUCE/tree/master/examples)
(commit-pinned snapshot, fetched 2026-06-19)

## 1. JUCE inventory snapshot

The JUCE `examples/` tree (master @ 2026-06-19) contains **80 example files** spread across
six category directories, plus scaffolding (`Assets/`, `CMake/`, `CMakeLists.txt`, `extern/`,
`Builds/`, `JuceLibraryCode/`, `Source/`) that is not Rust-portable. Per-category counts:

| Category | Examples | Notes |
|---|---|---|
| `Audio` | 13 | Standalone app demos (playback, recording, synthesis, MIDI, MPE, UMP, workgroup, capability inquiry, settings, latency, FFT, plucked strings) |
| `DSP` | 9 | Plugin demos (gain, oscillator, IIR/FIR/state-variable filters, oversampling, overdrive, convolution, SIMD, wave shaper) |
| `GUI` | 27 | Plugin demos of JUCE widget classes (sliders, look-and-feel, OpenGL, video, web, images, fonts, code editor, MDI, animations, …) |
| `Plugins` | 14 | Plugin demos covering ARA, AUv3, arpeggiator, multi-out, sampler, surround, web view, host, gain, dsp-module, midi logger, noise gate, reaper-embedded |
| `Utilities` | 16 | Cross-cutting utilities (analytics, box2d, child process, crypto, IAP, JS, live constants, multithreading, networking, OSC, push, system info, timers, unit tests, value trees, XML/JSON) |
| `DemoRunner` | 1 | One showcase app containing all categories under `Source/` (no per-example file split) |

Full ledger in [example-inventory.md](./example-inventory.md).

## 2. Architectural decisions

### AD-1. Sub-crate tree, not new top-level workspace deps

**Decision**: every new example is a workspace member that depends on the existing
`logic_nih_plug*` sub-crate tree (`dsp`, `gui`, `egui`/`iced`/`vizia`, `audio_formats`,
`audio_devices`, `audio_processors`, `data`, `animation`, `crypto`, `osc`, `graphics`,
`audio_basics`). **No new top-level workspace dependencies** (FR-012).

**Rationale**: the project's constitution §III rejects speculative abstractions. New
`Cargo.toml` deps would be cargo-culting, not a real constraint.

### AD-2. Plugin vs standalone `crate-type`

**Decision**:
- Plugin examples → `crate-type = ["cdylib"]` (matches every existing `plugins/examples/*`).
- Standalone app examples → `[[bin]]` target, **no** `crate-type = ["cdylib"]`. The library
  may still be exposed via `[lib]` for doc-tests, but the binary is the user-facing surface.

**Rationale**: cargo bundling (`cargo xtask bundle <name>`) requires `cdylib`. Standalone
apps don't ship as plugins; mixing the two `crate-type`s in one crate confuses the bundler.

### AD-3. DemoRunner backend feature flags are mutually exclusive

**Decision**: `juce_demorunner` exposes `gui-egui` / `gui-iced` / `gui-vizia` features.
A `compile_error!` fires if ≥2 are enabled. Default = `gui-egui` (matches the
`gain_gui_egui` reference and the rest of the workspace's egui bias).

**Rationale**: each backend imports its own widget library; linking all three would inflate
binary size and risk feature-gate conflicts (mirrors the same `compile_error!` pattern
already used by `logic_nih_plug_iced`).

### AD-4. Plugin host audio I/O via `MockAudioIODevice`

**Decision**: `juce_audio_plugin_host_egui` uses `logic_nih_plug_audio_devices::MockAudioIODevice`
as the audio I/O sink for the host. No `cpal` / `asio-sys` runtime dependency is added.

**Rationale** (per plan §Complexity Tracking Q4): keeps the example dep-free,
cross-platform-deterministic, and CI-friendly. Real I/O is left to the user's host
application; the example demonstrates the **host glue**, not the audio driver.

### AD-5. Single source of truth for plugin identity

**Decision**: every plugin example's identity is its `Cargo.toml` package name. The
`#[id = "..."]` / `VST3_CLASS_ID` / `CLAP_ID` strings are derived from that name by a
shared helper macro in the example's `src/lib.rs` (see [example-crate-contract](./contracts/example-crate-contract.md)).
A workspace integration test (`tests/id_uniqueness.rs`, T015) enforces no collisions.

### AD-6. Doc-tests as the primary correctness gate

**Decision**: every new crate ships a doc-test in its `src/lib.rs` that proves the
example's behavior against a small reference fixture (or against a closed-form
mathematical identity for DSP). `cargo xtask test-examples` (T019) runs them and
reports pass/fail per crate.

**Rationale**: doc-tests are `cargo test --doc` compatible (no extra CI config), run
in the crate's own context (so they have access to its private types), and double as
human-readable usage examples.

### AD-7. CI baseline is a one-time measurement

**Decision**: `cargo xtask baseline-ci` (T020) measures current workspace wall-clock
once and writes it to `ci-baseline.json`. Subsequent runs compute a delta in T097.

**Rationale**: SC-009 requires "≤50% CI growth" — a fixed reference is operationally
simpler than a moving average and matches the constitution's "smallest correct change"
tenet (no extra machinery for trend tracking).

## 3. Open questions

The plan's `[NEEDS CLARIFICATION]` items (Q2-Q5) remain open. Each has a recommended
default in [plan.md Complexity Tracking](./plan.md#complexity-tracking). The
implementation uses the recommended default; resolving Q2-Q5 narrows the
`[NEEDS CLARIFICATION:Qn]` annotations in [tasks.md](./tasks.md).

| # | Question | Recommended default |
|---|---|---|
| Q2 | Plugin-host GUI backend | `egui` only |
| Q3 | SC-003 measurement | 6-click proxy test |
| Q4 | Plugin-host audio I/O | `MockAudioIODevice` only |
| Q5 | SC-009 baseline | One-time measurement on Linux CI |

## 4. References

- [JUCE `examples/`](https://github.com/juce-framework/JUCE/tree/master/examples)
- [Project `AGENTS.md`](../../AGENTS.md) — hard rules, build commands, conventions
- [Project `TODO.md`](../../TODO.md) — JUCE port backlog
- [`logic_nih_plug_dsp` README](../../logic_nih_plug_dsp/README.md) — DSP module inventory
- [`logic_nih_plug_gui` README](../../logic_nih_plug_gui/README.md) — GUI module inventory
- [`logic_nih_plug_audio_devices` MockAudioIODevice](../../logic_nih_plug_audio_devices/src/mock.rs)
