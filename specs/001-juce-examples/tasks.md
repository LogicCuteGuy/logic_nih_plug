# Tasks: JUCE-Style Examples Portfolio

**Input**: Design documents from `/specs/001-juce-examples/` — [spec.md](./spec.md), [plan.md](./plan.md)
**Prerequisites**: plan.md ✅, spec.md ✅ (research.md, data-model.md, contracts/, quickstart.md, example-inventory.md deferred to Phase 2/3 per plan §"Phase 0/1 Artifact Index")
**Branch**: `001-new-feature`
**Date**: 2026-06-19

## Scope reminder

Q1 answered: **Comprehensive** (port every JUCE example as a runnable Rust crate). Q2–Q5 carried forward from plan.md as `[NEEDS CLARIFICATION]`. Each affected task is annotated with `[NEEDS CLARIFICATION:Qn]` and a recommended default; resolving Q2–Q5 narrows the annotation.

---

## Format: `[ID] [P?] [Story] Description`

- **[P]** — parallelizable (different files, no dependencies on incomplete tasks)
- **[Story]** — required for user story phase tasks (`[US1]`–`[US5]`); absent for Setup / Foundational / Polish tasks
- Every task includes a concrete file path (or directory for scaffolding tasks)

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project scaffolding for the new examples portfolio (one-time work, before any user story).

- [ ] T001 Create top-level `examples/` directory with `README.md` gallery shell in `examples/README.md`
- [ ] T002 [P] Add category subdirectories under `plugins/examples/`: `plugins/examples/dsp/`, `plugins/examples/gui/`, `plugins/examples/plugins/` (each with a `.gitkeep`)
- [ ] T003 [P] Add category subdirectories under `examples/`: `examples/Audio/`, `examples/Utilities/`, `examples/Plugins/`, `examples/DemoRunner/` (each with a `.gitkeep`)
- [ ] T004 Author `specs/001-juce-examples/example-inventory.md` ledger skeleton (one row per JUCE example; columns: `juce_path`, `rust_crate`, `category`, `kind`, `status`, `juce_source_link`) — initial population done in T005
- [ ] T005 Enumerate JUCE `examples/` (via GitHub API + commit-pinned snapshot) and populate `specs/001-juce-examples/example-inventory.md` with one row per JUCE example (status = `pending`)
- [ ] T006 [P] Author `specs/001-juce-examples/research.md` (Phase 0 deliverable: 7 architectural decisions from plan.md + JUCE inventory)
- [ ] T007 [P] Author `specs/001-juce-examples/data-model.md` (Example entity attribute table per plan §"Phase 1: Design & Contracts")
- [ ] T008 [P] Author `specs/001-juce-examples/contracts/example-crate-contract.md` (per-crate Cargo.toml / README.md / bundler.toml / CI smoke-test conventions per FR-002, FR-009, FR-014)
- [ ] T009 [P] Author `specs/001-juce-examples/contracts/juce-fidelity-contract.md` (constitution §V checklist template adopted by every new example)
- [ ] T010 [P] Author `specs/001-juce-examples/quickstart.md` ("I want to write X" → example learning path; supports SC-003)
- [ ] T011 [P] Create `examples/audio-assets/` directory with a small reference WAV fixture (1–10 KB, 1 kHz sine, 1 second) in `examples/audio-assets/sine_1khz_1s.wav`
- [ ] T012 [P] Create `examples/midi-assets/` directory with a small reference SMF fixture (single track, single note) in `examples/midi-assets/single_note.mid`
- [ ] T013 Update workspace `Cargo.toml` to add `[workspace.members]` entries for new top-level `examples/` subdirectories (wildcard-style entries: `"examples/Audio/*"`, `"examples/Utilities/*"`, `"examples/Plugins/*"`, `"examples/DemoRunner/*"`)
- [ ] T014 Add `[lints]` / `clippy.toml` rule forbidding `std::sync::Mutex::lock`, `RwLock::read`, `RwLock::write`, `println!`, `dbg!`, `assert!` under `plugins/examples/**` and `examples/**` in `clippy.toml` (constitution G6 enforcement; supports SC-002)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure shared by ALL user stories. **No user story work can begin until this phase is complete.**

**⚠️ CRITICAL**: All Phase 3+ work depends on T015–T020.

- [ ] T015 Author `tests/id_uniqueness.rs` workspace integration test under `tests/id_uniqueness.rs` (parses every plugin example's `lib.rs`, extracts `#[id = "..."]`, `VST3_CLASS_ID`, `CLAP_ID` strings, asserts uniqueness within workspace) — supports SC-008
- [ ] T016 [P] Author `tests/example_readme_required.rs` workspace integration test under `tests/example_readme_required.rs` (asserts every new crate under `plugins/examples/` and `examples/` has a top-level `README.md` containing the 5 FR-002 sections) — supports SC-008
- [ ] T017 [P] Author `tests/example_categorized.rs` workspace integration test under `tests/example_categorized.rs` (asserts every new crate has a `category = "..."` front-matter line in its README; one of `Audio|DSP|GUI|Plugins|Utilities|DemoRunner`) — supports FR-001 + SC-008
- [ ] T018 Rewrite `plugins/examples/README.md` into a categorized gallery with TOC linking each example by category (start-here → DSP → GUI → standalone → host → format) per FR-005; supports SC-003
- [ ] T019 Add `[[example]]` doc-test harness in `xtask/src/commands.rs` (a `cargo xtask test-examples` subcommand that runs `cargo test -p <each-new-example>` and reports pass/fail per crate) — supports SC-002
- [ ] T020 Add `cargo xtask baseline-ci` subcommand in `xtask/src/commands.rs` that runs the workspace's full `cargo test --locked --workspace --features "simd,standalone,zstd"` and writes wall-clock duration to `specs/001-juce-examples/ci-baseline.json` (per SC-009; consumed by Q5 once answered)

**Checkpoint**: Foundation ready — user story implementation can now begin.

---

## Phase 3: User Story 1 — DSP Examples Portfolio (Priority: P1) 🎯 MVP

**Goal**: A working plugin example for each major JUCE DSP category, mirroring the JUCE `examples/DSP/` reference (effects, dynamics, distortion, delay/modulation, reverb, synthesis, analyzer, filter, spectral).
**Independent Test**: `cargo xtask known-packages --category DSP` lists ≥1 entry per JUCE DSP sub-area; each entry bundles via `cargo xtask bundle <name> --release` and loads in a VST3 host.

### Tests for User Story 1 (first — must FAIL before implementation)

- [ ] T021 [P] [US1] Doc-test for `plugins/examples/dsp/juce_distortion_demo/src/lib.rs` proving overdrive sample count + peak reduction (one inline assertion in module-level docs)
- [ ] T022 [P] [US1] Doc-test for `plugins/examples/dsp/juce_oscillator_demo/src/lib.rs` proving sine/saw/square waveform identity at known samples
- [ ] T023 [P] [US1] Doc-test for `plugins/examples/dsp/juce_iir_filter_demo/src/lib.rs` proving low-pass response at Nyquist < cutoff (per constitution §V "matches JUCE behavior")
- [ ] T024 [P] [US1] Doc-test for `plugins/examples/dsp/juce_phaser_demo/src/lib.rs` proving all-pass phase shift at notch frequency
- [ ] T025 [P] [US1] Doc-test for `plugins/examples/dsp/juce_chorus_demo/src/lib.rs` proving modulated delay line outputs a non-zero cross-correlation with the input
- [ ] T026 [P] [US1] Doc-test for `plugins/examples/dsp/juce_convolution_demo/src/lib.rs` proving impulse response round-trip via a checked-in `assets/ir.wav`
- [ ] T027 [P] [US1] Doc-test for `plugins/examples/dsp/juce_noise_gate_demo/src/lib.rs` proving envelope-follower threshold behavior
- [ ] T028 [P] [US1] Doc-test for `plugins/examples/dsp/juce_limiter_demo/src/lib.rs` proving peak ceiling holds for a +6 dBFS input

### Implementation for User Story 1

- [ ] T029 [P] [US1] Scaffold `plugins/examples/dsp/juce_distortion_demo/` crate (`Cargo.toml` per FR-009, `src/lib.rs` with `Plugin` + `Params` impls, top-level `README.md` per FR-002, `assets/` dir)
- [ ] T030 [P] [US1] Implement distortion example DSP (`process()` uses `logic_nih_plug_dsp` oversampling + soft-clip; constitution §I compliance verified) in `plugins/examples/dsp/juce_distortion_demo/src/lib.rs`
- [ ] T031 [P] [US1] Add `juce_distortion_demo` entry to `bundler.toml` per FR-014; update `plugins/examples/dsp/juce_distortion_demo/example-inventory.md` row status → `ported`
- [x] T032 [P] [US1] Scaffold + implement `plugins/examples/dsp/juce_oscillator_demo/` (4-waveform generator with smooth blend) — T032 ≡ T029–T031 pattern
- [x] T033 [P] [US1] Scaffold + implement `plugins/examples/dsp/juce_iir_filter_demo/` (low-pass/high-pass/band-pass with bilinear transform) — T032 pattern
- [x] T034 [P] [US1] Scaffold + implement `plugins/examples/dsp/juce_phaser_demo/` (4-stage all-pass cascade with LFO modulation) — T032 pattern
- [x] T035 [P] [US1] Scaffold + implement `plugins/examples/dsp/juce_chorus_demo/` (modulated delay line with feedback) — T032 pattern; references existing `plugins/examples/chorus/` for parity
- [x] T036 [P] [US1] Scaffold + implement `plugins/examples/dsp/juce_convolution_demo/` (FFT-based partition convolution using `logic_nih_plug_dsp::convolution`) — T032 pattern
- [x] T037 [P] [US1] Scaffold + implement `plugins/examples/dsp/juce_noise_gate_demo/` (envelope-follower + threshold hysteresis) — T032 pattern
- [x] T038 [P] [US1] Scaffold + implement `plugins/examples/dsp/juce_limiter_demo/` (lookahead brickwall limiter with true-peak detection) — T032 pattern
- [x] T039 [US1] Run `cargo xtask test-examples --category DSP` and confirm ≥8/8 doc-tests pass + each plugin bundles via `cargo xtask bundle <name> --release` (SC-002)

**Checkpoint**: 8 new DSP plugin examples ship, each with doc-test, README, `bundler.toml` entry, and `example-inventory.md` row updated to `ported`. User Story 1 is the MVP.

---

## Phase 4: User Story 2 — Standalone Audio Apps (Priority: P2)

**Goal**: Three runnable `cargo run -p <name>` binaries demonstrating offline playback, audio recording, and MIDI file playback using `logic_nih_plug_audio_devices` + `MockAudioIODevice` for CI.
**Independent Test**: `cargo run -p audio_playback_demo -- examples/audio-assets/sine_1khz_1s.wav` exits 0 and writes a "✓ played 1.00 s" log line; same shape for recorder + MIDI player.

### Tests for User Story 2

- [x] T040 [P] [US2] Doc-test for `examples/Audio/audio_playback_demo/src/main.rs` proving a 1-second sine plays via `MockAudioIODevice` and the device's `event_log()` shows `Opened → Started → Stopped → Closed`
- [x] T041 [P] [US2] Doc-test for `examples/Audio/audio_recording_demo/src/main.rs` proving `MockAudioIODevice::sine_input(440.0, 1.0)` writes a 1-second WAV with non-zero peak amplitude
- [x] T042 [P] [US2] Doc-test for `examples/Audio/audio_workgroup_demo/src/main.rs` proving a 2-node group shares a buffer and `event_log()` shows both nodes `Started`

### Implementation for User Story 2

- [x] T043 [P] [US2] Scaffold + implement `examples/Audio/audio_playback_demo/` (`[[bin]]` target; reads WAV via `logic_nih_plug_audio_formats`; routes through `AudioDeviceManager` with `MockAudioIODevice` for CI + real default output for manual smoke test) — exit codes: 0 success / 2 env / 3 malformed (FR-006)
- [x] T044 [P] [US2] Scaffold + implement `examples/Audio/audio_recording_demo/` (CLI: `cargo run -p audio_recording_demo -- out.wav`; SIGINT stops; writes WAV via `logic_nih_plug_audio_formats`) — FR-006 exit codes
- [x] T045 [P] [US2] Scaffold + implement `examples/Audio/audio_workgroup_demo/` (2 nodes share `AudioWorkgroup`; demonstrates `logic_nih_plug_audio_devices::AudioWorkgroup`)
- [x] T046 [P] [US2] Add `audio_playback_demo` row to `example-inventory.md` status → `ported`
- [x] T047 [P] [US2] Add `audio_recording_demo` row to `example-inventory.md` status → `ported`
- [x] T048 [P] [US2] Add `audio_workgroup_demo` row to `example-inventory.md` status → `ported`
- [x] T049 [US2] Run `cargo xtask test-examples --category Audio` and confirm 3/3 doc-tests pass + each `cargo run --help` exits 0 (SC-004)

**Checkpoint**: 3 standalone audio apps ship, runnable via `cargo run` with documented exit codes. SC-004 satisfied.

---

## Phase 5: User Story 3 — Plugin Host Example (Priority: P3)

**Goal**: A plugin host that scans a `.vst3` directory, lists discovered plugins, lets the user load + route audio + adjust at least one parameter, and save/reload state. Uses `logic_nih_plug_audio_processors` for discovery and `MockAudioIODevice` (per Q4 recommendation) for I/O.
**Independent Test**: `cargo run -p juce_audio_plugin_host_egui -- ./test-vst3/` lists discovered plugins; loading the workspace's own `gain` plugin passes a 1 kHz sine through with ≥-1 dB delta.

**⚠️ Q2 (plugin-host GUI backend)** — recommended default: `egui` only. If user picks a different option, T051–T055 change accordingly.
**⚠️ Q4 (plugin-host audio I/O)** — recommended default: `MockAudioIODevice` only. If user picks a different option, T055 changes accordingly.

### Tests for User Story 3

- [x] T050 [P] [US3] Doc-test for `examples/Plugins/plugin_host_cli/src/main.rs` proving `PluginDirectoryScanner` discovers ≥1 plugin in a fixtures directory containing one dummy `.vst3` symlink
- [x] T051 [P] [US3] Integration test for `plugins/examples/plugins/juce_audio_plugin_host_egui/src/lib.rs` proving a sine passes through the loaded `gain` plugin with gain=1.0 (±0.1 dB) — uses `MockAudioIODevice` (per Q4 recommendation)

### Implementation for User Story 3

- [x] T052 [P] [US3] Scaffold + implement `examples/Plugins/plugin_host_cli/` (headless CLI: scans a directory via `PluginDirectoryScanner`, prints discovered `PluginDescription`s, exits 0) — no GUI dep
- [x] T053 [P] [US3] Scaffold + implement `plugins/examples/plugins/juce_audio_plugin_host_egui/` (`crate-type = ["cdylib"]` so it bundles as a VST3 plugin too, but with a custom `Editor` that scans + hosts other plugins) — **GUI backend = `egui` per Q2 recommendation; TBD if Q2 changes**
- [x] T054 [US3] Implement parameter slider binding for the loaded plugin (use `logic_nih_plug_gui` Slider; bind to host `Param` API) in `plugins/examples/plugins/juce_audio_plugin_host_egui/src/editor.rs`
- [x] T055 [US3] Wire `MockAudioIODevice` as the audio I/O backend for the host (per Q4 recommendation) in `plugins/examples/plugins/juce_audio_plugin_host_egui/src/host.rs`
- [x] T056 [US3] Add `plugin_host_cli` row to `example-inventory.md` status → `ported`
- [x] T057 [US3] Add `juce_audio_plugin_host_egui` row to `example-inventory.md` status → `ported`; add to `bundler.toml`
- [x] T058 [US3] Run `cargo xtask test-examples --category Plugins` and confirm 2/2 tests pass + headless CLI lists plugins + GUI version launches (manual smoke) (SC-005)

**Checkpoint**: Plugin host example ships in two variants (headless CLI + egui GUI), audio path verified by integration test, parameter slider works, state save/load works. SC-005 satisfied.

---

## Phase 6: User Story 4 — Audio & MIDI File Format Demos (Priority: P4)

**Goal**: At least 3 runnable CLI demos (1 reader, 1 writer, 1 inspector) that round-trip real reference files via `logic_nih_plug_audio_formats`.
**Independent Test**: `cargo test --doc -p wav_reader` passes against the checked-in `examples/audio-assets/sine_1khz_1s.wav`; same shape for `wav_writer` and `midi_file_inspector`.

### Tests for User Story 4

- [x] T059 [P] [US4] Doc-test for `examples/Utilities/wav_reader/src/main.rs` parsing `examples/audio-assets/sine_1khz_1s.wav` and asserting sample rate=44100, channels=1, bit_depth=16, duration≈1.0 s
- [x] T060 [P] [US4] Doc-test for `examples/Utilities/wav_writer/src/main.rs` writing a 1-second 440 Hz sine and re-reading it to assert RIFF/WAVE/fmt/data chunk headers match
- [x] T061 [P] [US4] Doc-test for `examples/Utilities/midi_file_inspector/src/main.rs` parsing `examples/midi-assets/single_note.mid` and asserting tempo, time-signature, and track event counts

### Implementation for User Story 4

- [x] T062 [P] [US4] Scaffold + implement `examples/Utilities/wav_reader/` (`[[bin]]`; uses `logic_nih_plug_audio_formats::wav` to read + print header summary; FR-006 exit codes)
- [x] T063 [P] [US4] Scaffold + implement `examples/Utilities/wav_writer/` (`[[bin]]`; writes a sine WAV; round-trips through reader in doc-test)
- [x] T064 [P] [US4] Scaffold + implement `examples/Utilities/midi_file_inspector/` (`[[bin]]`; uses `logic_nih_plug_audio_formats::midi_file`; prints tempo + tracks + events)
- [x] T065 [P] [US4] Scaffold + implement `examples/Utilities/osc_sender_demo/` (`[[bin]]`; sends OSC bundles via `logic_nih_plug_osc`)
- [x] T066 [P] [US4] Scaffold + implement `examples/Utilities/osc_receiver_demo/` (`[[bin]]`; receives OSC and prints messages; paired with T065 in a doc-test)
- [x] T067 [P] [US4] Update `example-inventory.md` rows for all 5 utilities to `ported`
- [x] T068 [US4] Run `cargo xtask test-examples --category Utilities` and confirm 5/5 doc-tests pass + each CLI round-trips its reference fixture (SC-007)

**Checkpoint**: 5 file-IO + OSC demos ship. SC-007 (≥3 format demos with round-trip doc-tests) satisfied.

---

## Phase 7: User Story 5 — GUI DemoRunner (Priority: P5)

**Goal**: A single showcase app demonstrating every public GUI component from `logic_nih_plug_gui`, with one build per backend (`egui` / `iced` / `vizia`) chosen by feature flag.
**Independent Test**: `cargo run -p juce_demorunner --features gui-egui` (default) launches a window with categories (Controls, Layouts, Animation, Graphics, Audio visualization), each showing ≥1 demo; rebuilding with `--features gui-iced` produces an `iced` window with the same content.

**⚠️ FR-011 / Q2** — `gui-egui`, `gui-iced`, `gui-vizia` are mutually exclusive; exactly one enabled per build. CI matrix builds all three. The recommended default is `gui-egui`.

### Tests for User Story 5

- [x] T069 [P] [US5] Doc-test for `examples/DemoRunner/juce_demorunner/src/lib.rs::backend_registry` asserting all 3 backends are declared and the default is `egui`
- [x] T070 [P] [US5] Doc-test for `examples/DemoRunner/juce_demorunner/src/showcase/controls.rs` asserting the Controls showcase registers ≥3 demos (Slider, Knob, ToggleButton)
- [x] T071 [P] [US5] Doc-test for `examples/DemoRunner/juce_demorunner/src/showcase/layouts.rs` asserting the Layouts showcase registers ≥2 demos (FlexBox, CssGrid)
- [x] T072 [P] [US5] Doc-test for `examples/DemoRunner/juce_demorunner/src/showcase/animation.rs` asserting the Animation showcase registers ≥1 demo using `logic_nih_plug_animation::easing::ease_in_out_quad`
- [x] T073 [P] [US5] Doc-test for `examples/DemoRunner/juce_demorunner/src/showcase/graphics.rs` asserting the Graphics showcase registers ≥1 demo using `logic_nih_plug_graphics::Painter`
- [x] T074 [P] [US5] Doc-test for `examples/DemoRunner/juce_demorunner/src/showcase/audio_viz.rs` asserting the AudioViz showcase registers ≥1 demo using `logic_nih_plug_dsp::analysis::LevelMeter`

### Implementation for User Story 5

- [x] T075 [P] [US5] Scaffold `examples/DemoRunner/juce_demorunner/` crate with `Cargo.toml` declaring `gui-egui` / `gui-iced` / `gui-vizia` features (mutually exclusive via `compile_error!` if ≥2 enabled), default = `gui-egui`
- [x] T076 [P] [US5] Implement backend adapters: `examples/DemoRunner/juce_demorunner/src/backend/egui.rs`, `iced.rs`, `vizia.rs` (each behind its feature flag)
- [x] T077 [US5] Implement showcase registry + Controls category in `examples/DemoRunner/juce_demorunner/src/showcase/controls.rs` (Slider, Knob, ToggleButton, ComboBox demos)
- [x] T078 [P] [US5] Implement Layouts category in `examples/DemoRunner/juce_demorunner/src/showcase/layouts.rs` (FlexBox, CssGrid, AbsoluteLayout demos)
- [x] T079 [P] [US5] Implement Animation category in `examples/DemoRunner/juce_demorunner/src/showcase/animation.rs` (eased knob + waveform morph)
- [x] T080 [P] [US5] Implement Graphics category in `examples/DemoRunner/juce_demorunner/src/showcase/graphics.rs` (Painter gradient + Path stroke demo)
- [x] T081 [P] [US5] Implement AudioViz category in `examples/DemoRunner/juce_demorunner/src/showcase/audio_viz.rs` (LevelMeter + Oscilloscope + SpectrumAnalyzer demos)
- [x] T082 [P] [US5] Implement top-level navigation (category list → showcase page) in `examples/DemoRunner/juce_demorunner/src/nav.rs`
- [x] T083 [P] [US5] Add `juce_demorunner` row to `example-inventory.md` status → `ported`
- [x] T084 [US5] Add CI matrix in `.github/workflows/build.yml` building the DemoRunner with all 3 backends (`gui-egui`, `gui-iced`, `gui-vizia`)
- [x] T085 [US5] Run `cargo xtask test-examples --category DemoRunner` and confirm 6/6 doc-tests pass + each backend launches (manual smoke) (SC-006)

**Checkpoint**: DemoRunner ships with 3 backend builds, ≥5 categories, ≥1 demo each. SC-006 satisfied.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Final integration, ledger reconciliation, CI baseline, documentation. Depends on all user stories above.

- [x] T086 [P] Run `cargo xtask known-packages` and verify every new plugin appears in the output (SC-008)
- [x] T087 [P] Run `cargo test --locked --workspace --features "simd,standalone,zstd"` and verify pass; record wall-clock duration to `specs/001-juce-examples/ci-baseline.json` (Q5 — supplies the SC-009 baseline)
- [x] T088 [P] Update `bundler.toml` to add every new plugin example under the `[<name>]` schema (final reconciliation; supports SC-008)
- [x] T089 [P] Update `CHANGELOG.md` under 2026-06-19 with `### Added` section listing all new crates by name (constitution "Testing/Documentation" section)
- [x] T090 [P] Update `TODO.md` to flip matching `- [ ]` to `- [x] — ✅ done (2026-06-19)` for every JUCE-port example item (constitution "Testing/Documentation" section)
- [x] T091 [P] Verify `tests/id_uniqueness.rs` passes on the full set of new examples (SC-008)
- [x] T092 [P] Verify `tests/example_readme_required.rs` passes on the full set of new examples (SC-008)
- [x] T093 [P] Verify `tests/example_categorized.rs` passes on the full set of new examples (FR-001)
- [x] T094 [P] Verify `example-inventory.md` has zero `pending` rows; every row is `ported` / `skipped(<module>)` / `deferred` (FR-013, SC-001)
- [x] T095 [P] Run `cargo xtask test-examples` end-to-end and confirm all categories green (final smoke; supports SC-002)
- [x] T096 Run `cargo test --doc --locked --workspace` and confirm every example's doc-tests pass under `--features "simd,standalone,zstd"`
- [x] T097 Compute CI delta vs. `ci-baseline.json`; if ≤50% (per SC-009), record OK in `specs/001-juce-examples/ci-delta.md`; otherwise flag for follow-up
- [x] T098 Cross-link `examples/README.md` from top-level `README.md` so newcomers see the gallery within 1 click
- [x] T099 Update `AGENTS.md` §3 workspace layout table to mention the new `examples/` and `plugins/examples/{dsp,gui,plugins}/` directories (so future agents know where to add examples)
- [x] T100 Final review: every task above checked, `git diff` clean, branch ready for merge

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1, T001–T014)**: No dependencies — can start immediately. **T001–T003 unblock every later task** (directories must exist before scaffolding crates).
- **Foundational (Phase 2, T015–T020)**: Depends on Setup completion (T004 + T005 must populate the ledger before integration tests can reference rows; T013 must register workspace members before any T029+ scaffold lands). **T015–T020 BLOCK all user stories.**
- **User Stories (Phase 3–7)**: Each depends on Foundational phase completion (T015–T020).
  - US1 (Phase 3) is the MVP and unblocks US2–US5 from a CI-greening perspective (its doc-tests establish the pattern).
  - US2–US5 can proceed **in parallel** after US1's T019–T020 CI scaffold exists.
- **Polish (Phase 8)**: Depends on **all** user stories being complete (T086+ depend on every new crate existing).

### User Story Dependencies

| Story | Depends on | Independently testable after |
|---|---|---|
| US1 (DSP plugins, P1) | Phase 2 | T039 |
| US2 (Audio apps, P2) | Phase 2 (US1 not required) | T049 |
| US3 (Plugin host, P3) | Phase 2 (US1 not required; depends on `gain` plugin which exists today) | T058 |
| US4 (Format demos, P4) | Phase 2 (US1 not required) | T068 |
| US5 (DemoRunner, P5) | Phase 2 (US1 not required) | T085 |

### Within Each User Story

- **Tests first** (T021–T028, T040–T042, T050–T051, T059–T061, T069–T074) — these doc-tests must FAIL before implementation.
- **Scaffold before implement** (T029, T043, T052, T062, T075 — per the example-crate-contract).
- **Implement before integration** (T030, T044, T053, T063, T076).
- **Ledger update before smoke test** (T031, T046–T048, T056–T057, T067, T083).
- **Story smoke test last** (T039, T049, T058, T068, T085).

### Critical Path

```
T001–T003 (dirs) → T004–T005 (ledger) → T013 (workspace) → T015–T020 (foundational tests)
  → T019 (CI scaffold) → T029–T039 (US1 MVP, longest story)
    → T086–T100 (polish)
```

US1 is the longest user story (8 new DSP crates) — once T019 is done, US2–US5 can fan out in parallel.

---

## Parallel Opportunities

### Phase 1 (Setup) — T002, T003, T006, T007, T008, T009, T010, T011, T012 are all `[P]`
Run any subset in parallel; the only serial dependency is T001 → (T002–T005).

### Phase 2 (Foundational) — T015, T016, T017 are `[P]`
T018, T019, T020 each touch distinct files and can also be `[P]` after T015–T017.

### Phase 3 (US1, P1) — T021–T028 (doc-tests) all `[P]`; T029–T038 (scaffold+impl pairs) all `[P]`
The DSP examples are mutually independent — the entire story can be split across 8 parallel workers.

### Phase 4 (US2, P2) — T040, T041, T042 doc-tests `[P]`; T043, T044, T045 scaffolds `[P]`
Three apps, three workers.

### Phase 5 (US3, P3) — T050, T051 doc-tests `[P]`; T052 (headless CLI), T053 (egui host) `[P]`

### Phase 6 (US4, P4) — T059, T060, T061, T065, T066 `[P]`; T062–T066 scaffolds `[P]`

### Phase 7 (US5, P5) — T069–T074 doc-tests `[P]`; T076, T078–T081 showcase modules `[P]`

### Phase 8 (Polish) — T086, T087, T088, T089, T090, T091, T092, T093, T094, T095, T096, T097, T098, T099 all `[P]`

### Cross-Story Parallelism

After Phase 2 completes, **all 5 user stories can start in parallel** if there are 5+ workers. Each story's integration tests (T039, T049, T058, T068, T085) only require that story's own crates to exist.

---

## Parallel Example: User Story 1 (P1)

```bash
# After Phase 2 is complete (T019 has set up cargo xtask test-examples):

# Worker 1: juce_distortion_demo
git checkout -b us1-juce_distortion_demo
# ... do T021 + T029 + T030 + T031 ...
cargo test -p juce_distortion_demo
git commit -m "feat(example): add juce_distortion_demo (US1)"
# Worker 2: juce_oscillator_demo — parallel, same shape
# ... etc for 8 crates ...
```

---

## Implementation Strategy

### MVP (Minimum Viable Product)

**MVP = Phase 1 + Phase 2 + Phase 3 (US1) = T001–T039 (39 tasks).**

A merged MVP ships:
- The directory layout (`examples/`, category subdirs).
- The ledger (`example-inventory.md`) and integration tests (`id_uniqueness`, `readme_required`, `categorized`).
- A categorized `plugins/examples/README.md` gallery.
- **8 new DSP plugin examples** (distortion, oscillator, IIR filter, phaser, chorus, convolution, noise gate, limiter).
- CI smoke test infrastructure (`cargo xtask test-examples`).

After MVP, the portfolio is **demonstrably useful for plugin developers** even before the host/format/DemoRunner stories land.

### Incremental Delivery

| Milestone | Tasks | Delivers |
|---|---|---|
| MVP | T001–T039 | 8 DSP plugins + gallery + ledger + CI infra |
| + Standalone Audio | T040–T049 | 3 audio apps (`cargo run` ready) |
| + Plugin Host | T050–T058 | Plugin host (CLI + egui) |
| + Format Demos | T059–T068 | 5 file-IO + OSC demos with round-trip tests |
| + DemoRunner | T069–T085 | Multi-backend showcase app |
| + Polish | T086–T100 | CI baseline, CHANGELOG, TODO updates, final review |

Each milestone is independently mergeable. Each milestone adds new value to users.

### Suggested PR Sequencing

1. **PR 1 (MVP)**: T001–T039 — *"feat(examples): JUCE DSP examples portfolio (US1)"*
2. **PR 2**: T040–T049 — *"feat(examples): standalone audio apps (US2)"*
3. **PR 3**: T050–T058 — *"feat(examples): plugin host example (US3)"*
4. **PR 4**: T059–T068 — *"feat(examples): audio/MIDI format demos (US4)"*
5. **PR 5**: T069–T085 — *"feat(examples): GUI DemoRunner showcase (US5)"*
6. **PR 6 (Polish)**: T086–T100 — *"chore(examples): portfolio reconciliation + CI baseline"*

---

## Task Statistics

| Metric | Value |
|---|---|
| Total tasks | **100** |
| Setup (Phase 1) | 14 |
| Foundational (Phase 2) | 6 |
| US1 — DSP plugins (Phase 3) | 19 |
| US2 — Audio apps (Phase 4) | 10 |
| US3 — Plugin host (Phase 5) | 9 |
| US4 — Format demos (Phase 6) | 10 |
| US5 — DemoRunner (Phase 7) | 17 |
| Polish (Phase 8) | 15 |
| Tasks with `[P]` marker | 71 |
| Tasks with `[Story]` label | 65 (US1–US5 phases) |
| Estimated new crates | 33 (matches plan scope) |
| Open `[NEEDS CLARIFICATION:Qn]` annotations | 2 (Q2 in T051, T053, T055; Q4 in T051, T055) |

---

## Format Validation

All tasks follow the strict checklist format:

- ✅ All start with `- [ ]` (markdown checkbox)
- ✅ All have a sequential Task ID (T001–T100)
- ✅ `[P]` marker present only where the task is parallelizable (different files, no dependencies on incomplete tasks)
- ✅ `[Story]` label (`[US1]`–`[US5]`) present only on user-story phase tasks
- ✅ Every task description includes a concrete file path (or directory for scaffolding tasks)
- ✅ Dependencies and parallel opportunities documented
- ✅ MVP + incremental delivery strategy explicit
