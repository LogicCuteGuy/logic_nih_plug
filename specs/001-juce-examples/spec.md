# Feature Specification: JUCE-Style Examples Portfolio

**Feature Branch**: `001-new-feature` *(renamable to `001-juce-examples` on first commit)*
**Created**: 2026-06-19
**Status**: Draft
**Input**: User description: "i want example like https://github.com/juce-framework/JUCE/tree/master/examples"

---

## Summary

Build a **comprehensive**, JUCE-aligned examples portfolio for `logic_nih_plug` by porting **every example** JUCE ships in its [`examples/`](https://github.com/juce-framework/JUCE/tree/master/examples) directory into a runnable Rust crate — organized by the same category layout (Audio, DSP, GUI, Plugins, Utilities, DemoRunner).

The workspace already ships ~30 plugin examples under `plugins/examples/`. The feature (1) preserves those existing examples, (2) re-classifies them under the JUCE category layout, (3) adds every missing JUCE example as a new crate (estimated ~40+ new crates total), and (4) ships a top-level `examples/README.md` that mirrors JUCE's example gallery structure for newcomer navigation.

---

## Clarifications

### Session 2026-06-19

- Q: Which concrete examples go in this portfolio? → A: **Option A — Comprehensive**. Port every example JUCE ships across the 6 categories (Audio, DSP, GUI, Plugins, Utilities, DemoRunner) as a runnable Rust crate in the workspace. Estimated scope: ~40+ new crates (see FR-013, FR-014, SC-001, and the example-inventory ledger under `specs/001-juce-examples/`).

## User Scenarios & Testing *(mandatory)*

User stories are prioritized as independently deliverable slices. Each story alone produces a working, demonstrable artifact.

### User Story 1 — DSP Examples Portfolio (Priority: P1)

A plugin developer browsing the repo wants to find a working example for every major DSP category JUCE ships: effects, dynamics, instruments, analyzers. They want each example to be small, single-purpose, and runnable.

**Why this priority**: DSP examples are the highest-traffic category for any plugin framework. They are the first thing newcomers copy/paste from. Currently the workspace has good but uneven DSP coverage; filling the gaps is the single biggest impact per example added.

**Independent Test**: Each new DSP example builds with `cargo build -p <example>`, bundles via `cargo xtask bundle <example> --release`, loads in a VST3 host, processes audio without dropouts, and has a `README.md` describing parameters and signal flow.

**Acceptance Scenarios**:

1. **Given** the `plugins/examples/` directory, **When** a developer lists its contents, **Then** each major DSP category from JUCE's `examples/DSP/` (dynamics, distortion, delay/modulation, reverb, synthesis, analyzer, filter, spectral) is represented by at least one example plugin.
2. **Given** a new DSP example, **When** the developer reads its `README.md`, **Then** the document lists parameters, signal-flow diagram (text), module dependencies, build command, and a "what to learn from this example" section.
3. **Given** a DSP example uses a sub-crate, **When** the example is built, **Then** the build succeeds without modifying the sub-crate's public API (the example adapts to existing APIs).

---

### User Story 2 — Standalone Audio Apps (Priority: P2)

A user wants to run the framework **outside** a DAW — for example, an offline audio file player, a live audio recorder, or a MIDI file player. They want runnable binaries they can launch from the command line or double-click.

**Why this priority**: JUCE's `examples/Audio/` directory has `AudioPlaybackDemo`, `AudioRecordingDemo`, `AudioWorkgroupDemo`, etc. The corresponding `logic_nih_plug_audio_devices` crate exists in the workspace but currently has **zero** consumer examples. This is the second-largest gap after DSP coverage.

**Independent Test**: Each standalone app builds via `cargo build -p <example>` (no `cdylib` requirement), launches and produces audio output (player) or records audio input (recorder) without a DAW host, and reads/writes a documented file format.

**Acceptance Scenarios**:

1. **Given** the standalone audio player example, **When** a developer runs `cargo run -p audio_player -- path/to/file.wav`, **Then** the WAV file plays through the default output device with a visible progress indicator.
2. **Given** the standalone audio recorder example, **When** a developer runs `cargo run -p audio_recorder -- out.wav`, **Then** the default input device is captured to `out.wav` until the user sends SIGINT, with elapsed time visible.
3. **Given** the standalone MIDI file player example, **When** a developer runs `cargo run -p midi_file_player -- song.mid`, **Then** the SMF events are sent to a selectable MIDI output (or rendered to audio if no MIDI output is available).

---

### User Story 3 — Plugin Host Example (Priority: P3)

A user wants to load VST3 / CLAP plugins inside their own application — for example, to build a custom plugin chain, a plugin-testing harness, or a headless plugin scanner. The `logic_nih_plug_audio_processors` crate provides `PluginDescription`, `KnownPluginList`, and `PluginDirectoryScanner`, but no consumer example demonstrates it.

**Why this priority**: Plugin hosting is a third-pillar capability (after plugin authoring and standalone apps). JUCE's `examples/Plugins/` has `AudioPluginHost` — a flagship example. Without one, the framework's host-side value is invisible.

**Independent Test**: The host example builds, scans a user-specified directory for VST3 plugins, lists them in a simple UI, lets the user load a plugin, route audio through it, and adjust at least one parameter.

**Acceptance Scenarios**:

1. **Given** a directory of `.vst3` bundles, **When** the host example is launched with that directory path, **Then** it scans the directory and displays a list of discovered plugins (name, vendor, category, unique ID).
2. **Given** a discovered plugin, **When** the user selects and "loads" it, **Then** the plugin is instantiated, its audio passes through (verified by an audible sine sweep), and at least one parameter is exposed as a slider.
3. **Given** the loaded plugin, **When** the user clicks "save state", **Then** the plugin's parameter state is serialized to disk and can be reloaded on the next launch.

---

### User Story 4 — Audio & MIDI File Format Demos (Priority: P4)

A user wants to read or write WAV / AIFF / FLAC / OGG / Standard MIDI Files from Rust code without going through a DAW. The `logic_nih_plug_audio_formats` and `logic_nih_plug_audio_formats::midi_file` crates exist but have no end-to-end examples.

**Why this priority**: File I/O is the foundation of any offline-rendering toolchain. JUCE's `examples/Utilities/` and `examples/Audio/` cover this. Lowest complexity per example but high reuse value.

**Independent Test**: Each format example builds, reads an input file (or writes an output file), and prints or writes back a header/metadata summary that matches the file format's documented structure.

**Acceptance Scenarios**:

1. **Given** a `.wav` file, **When** the wav_reader example runs, **Then** it prints sample rate, channel count, bit depth, and duration, and verifies the file against its declared format.
2. **Given** the wav_writer example, **When** a developer runs it with `--seconds 5 --freq 440`, **Then** a 5-second 440 Hz sine WAV is written with correct RIFF/WAVE/fmt/data chunks.
3. **Given** a `.mid` file, **When** the midi_file_inspector example runs, **Then** it prints tempo, time signature, key signature, and a track-by-track event summary.

---

### User Story 5 — GUI DemoRunner (Priority: P5)

A user wants to see every GUI component from `logic_nih_plug_gui` and `logic_nih_plug_egui` / `_iced` / `_vizia` in a single runnable showcase — without needing a DAW host. JUCE's `examples/DemoRunner` is a flagship reference app.

**Why this priority**: Discoverability of GUI components is currently weak (each gain_gui_* example shows one backend). A single showcase app reduces the time-to-first-render for new components.

**Independent Test**: The DemoRunner builds for at least one GUI backend (`egui`, `iced`, or `vizia`), launches as a standalone window, and visually demonstrates every public GUI component (sliders, knobs, buttons, toggles, labels, plots, meters, keyboard, combobox, text editor).

**Acceptance Scenarios**:

1. **Given** the DemoRunner is launched, **When** it opens, **Then** it shows a top-level navigation list of categories (Controls, Layouts, Animation, Graphics, Audio visualization) and at least one demo per category.
2. **Given** a Controls category demo, **When** the user interacts with a slider, **Then** a label updates in real-time and a smooth-tweened knob visually tracks the slider position.
3. **Given** the DemoRunner, **When** the developer switches GUI backends by rebuilding with `--features egui` / `--features iced` / `--features vizia`, **Then** the same demo content is rendered by the new backend without code changes.

---

### Edge Cases

- What happens when an example's optional dependency is missing (e.g., `cpal`, `asio-sys` for audio devices, `ffmpeg-next` for video)? → Each example's README must list optional deps and a `--no-default-features` build path.
- What happens when no audio device is available at runtime? → Standalone apps must fail with a clear, human-readable error (not a panic) and exit code 2.
- What happens when the user provides a malformed input file (e.g., a corrupted WAV or invalid SMF)? → Format examples must print the specific parse error and exit code 3, not panic.
- What happens when a plugin host example is run on a platform without VST3 support (e.g., Linux without `wine`)? → The example must list available formats dynamically; missing formats are skipped, not fatal.
- What happens when an example's GUI backend is requested but not compiled in? → The example's CLI lists available backends and exits with a helpful message when a wrong one is requested.

---

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The examples portfolio MUST organize `plugins/examples/` (and a new `examples/` directory for non-plugin apps, if created) by category labels matching JUCE's taxonomy: `Audio`, `DSP`, `GUI`, `Plugins`, `Utilities`, `DemoRunner`.
- **FR-002**: Each new example MUST ship a top-level `README.md` documenting: (a) what the example demonstrates, (b) parameters / inputs / outputs, (c) sub-crate dependencies, (d) build & bundle commands, (e) "what to learn from this example" takeaway.
- **FR-003**: Each new example MUST build successfully with `cargo build -p <example>` using the project's documented CI flags (`--locked`, `simd,standalone,zstd` where applicable).
- **FR-004**: Each plugin-format example MUST bundle successfully via `cargo xtask bundle <example> --release` and produce a loadable artifact for at least VST3 + CLAP.
- **FR-005**: The top-level `plugins/examples/README.md` MUST be replaced or augmented with a categorized learning path (start-here → DSP → GUI → standalone → host → format), modeled after JUCE's example gallery.
- **FR-006**: Standalone app examples MUST be `[[bin]]` targets (not `cdylib`) and MUST exit with documented exit codes on errors (0 = success, 2 = environment/IO error, 3 = malformed input).
- **FR-007**: Each example MUST comply with the project's constitution §I (Real-Time Safety) — no allocations in audio-thread paths, no blocking locks, no `println!` in `process()`.
- **FR-008**: Each example MUST comply with the constitution §II (Stable Public Identifiers) — `#[id = "..."]` plugin IDs MUST be globally unique and `#[persist = "..."]` state keys MUST NOT change once shipped.
- **FR-009**: Each example MUST have a `Cargo.toml` entry following the workspace pattern: `crate-type = ["cdylib"]` for plugins, `[[bin]]` for standalone apps; `logic_nih_plug = { path = "../../../", features = ["assert_process_allocs"] }` for plugins that exercise real-time paths.
- **FR-010**: Format reader/writer examples MUST round-trip a reference file (read → print metadata → write → re-read → diff) and pass a doc-test or unit test demonstrating correctness.
- **FR-011**: The DemoRunner MUST be a single crate with feature flags for each GUI backend (`egui`, `iced`, `vizia`), exactly one of which is enabled per build, so the same demo content renders in any backend.
- **FR-012**: New examples MUST NOT add new top-level workspace dependencies beyond what is already in `Cargo.toml`. Sub-crate features can be enabled, but no new external crates.
- **FR-013**: The portfolio MUST include every example JUCE ships in the 6 categories (`Audio/`, `DSP/`, `GUI/`, `Plugins/`, `Utilities/`, `DemoRunner/`), as enumerated in [JUCE `examples/`](https://github.com/juce-framework/JUCE/tree/master/examples). Examples that depend on a JUCE module not yet ported to `logic_nih_plug` MUST be recorded in the example-inventory ledger with status `skipped(<module>)` and a one-line reason — they MUST NOT be silently omitted.
- **FR-014**: Each new crate MUST be registered in `bundler.toml` (if it is a plugin) with a globally unique `[[bin]]` `name` and `crate-type = ["cdylib"]`, OR registered as a workspace `[[bin]]` target with `[[bin]]` `name` (if it is a standalone app).

### Key Entities

- **Example**: A single Cargo crate under `plugins/examples/` (plugin) or `examples/` (standalone app). Attributes: name, category (Audio/DSP/GUI/Plugins/Utilities/DemoRunner), format(s), sub-crate deps, has-GUI (bool), README presence (bool), CI build status.
- **Category**: A label grouping related examples. Mirrors JUCE's example taxonomy. Each example belongs to exactly one primary category.
- **DemoRunner Showcase Item**: A single GUI component demo inside the DemoRunner. Attributes: component name, source crate, interactive (bool), backend-agnostic (bool).

---

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The examples portfolio covers all **6 categories** (Audio, DSP, GUI, Plugins, Utilities, DemoRunner) and ports **every example** JUCE ships in each. The full list of ported examples MUST be enumerated in `specs/001-juce-examples/example-inventory.md` (one row per JUCE example, columns: JUCE path, Rust crate, status: `ported` / `skipped(<module>)` / `deferred`). `skipped` and `deferred` rows are permitted only when FR-013's recording requirement is met.
- **SC-002**: Each new example passes `cargo build -p <example>` and (for plugins) `cargo xtask bundle <example> --release` in the workspace's CI environment on Ubuntu, macOS, and Windows, with at most 5% of CI time spent building examples.
- **SC-003**: A newcomer reading the top-level `plugins/examples/README.md` can locate the right example for their goal (e.g., "I want to write a delay plugin") in under 60 seconds — measured by a usability checklist on the README's table of contents.
- **SC-004**: At least **3 standalone app examples** exist that can be launched with a single `cargo run` command and demonstrate offline audio playback, audio recording, and MIDI file playback respectively.
- **SC-005**: At least **1 plugin-host example** exists that can scan a `.vst3` directory and instantiate a discovered plugin with audio passing through.
- **SC-006**: At least **1 DemoRunner** exists that visually demonstrates every public GUI component type from `logic_nih_plug_gui` in at least one supported backend, with category navigation.
- **SC-007**: At least **3 audio/MIDI format demo examples** exist (one reader, one writer, one inspector), each round-tripping a real reference file in its doc-test.
- **SC-008**: 100% of new examples have a top-level `README.md`, 100% of plugin examples appear in `bundler.toml`, and 100% of new plugin IDs are unique (verified by grep + a uniqueness test).
- **SC-009**: The total CI build time for the workspace does not increase by more than **50%** after the examples portfolio is added (the existing `cargo test --locked --workspace` is the source of truth). The baseline shall be measured once before the first new example is added; CI matrix parallelism and shared `target/` caching shall be used to keep wall-clock time manageable.

---

## Assumptions

- **A1**: "Examples like JUCE's examples" is interpreted as: a **comprehensive** port — replicate every example JUCE ships in the 6 categories (Audio, DSP, GUI, Plugins, Utilities, DemoRunner) as a runnable Rust crate, organized under the matching category directory. Each example is a faithful behavioral port of the JUCE example it mirrors; visual styling, exact parameter names, and audio-routing topology match the JUCE reference.
- **A2**: The existing 30+ examples in `plugins/examples/` remain; this feature adds and organizes, it does not delete or rename existing examples (unless strictly required for category organization, in which case old paths become aliases).
- **A3**: Plugin format coverage targets the project's current matrix (VST3 + CLAP first; AU/AUv3 mac-only; LV2 Linux; AAX stub-only). Examples follow the multi-format pattern from `plugins/examples/gain_multi_format/` where applicable.
- **A4**: Standalone apps target desktop hosts (Linux/macOS/Windows). Mobile targets (iOS AUv3, Android) are out of scope unless explicitly requested.
- **A5**: The `logic_nih_plug_audio_devices`, `logic_nih_plug_audio_formats`, and `logic_nih_plug_audio_processors` sub-crates already provide the APIs needed; no new sub-crate is required for this feature.
- **A6**: Examples use the project's stable Rust toolchain (`rust-version = "1.80"`) and do not require `nightly` unless explicitly listed in their README.
- **A7**: The existing constitution (v1.0.0) governs this feature — especially §I (Real-Time Safety), §III (Smallest Correct Change — no speculative flexibility), and §V (JUCE Port Fidelity — match the JUCE module's behavior where a port exists).

---

## Out of Scope

- New sub-crates (the feature reuses existing ones).
- New GUI backends beyond the existing three (egui / iced / vizia).
- Mobile (iOS / Android) host apps.
- Network streaming examples beyond what `logic_nih_plug_osc` already covers.
- Video examples (the `logic_nih_plug_video` crate already ships its own examples).

---

## Open Questions

None. The feature scope, categories, and priorities are derived from the JUCE reference and the existing workspace state. Any narrowing or expansion should happen in `/speckit.clarify` before planning.
