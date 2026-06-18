# AGENTS.md — guidance for AI coding agents

Rust audio-plugin framework (`logic_nih_plug`). ~30 plugins, many sub-crates. **Smallest correct change; keep the codebase intact.**

> Prefer linking to existing docs over restating them. See [DOCUMENTATION_INDEX.md](DOCUMENTATION_INDEX.md) for the full index.

---

## 1. Start here

| Doc | What it covers |
|---|---|
| [README.md](README.md) | Project overview, plugin list, features |
| [TODO.md](TODO.md) | JUCE port backlog with per-crate status |
| [QUICK_START.md](QUICK_START.md) | 5-min intro to ported modules (dsp, audio_formats, gui, animation) |
| [src/prelude.rs](src/prelude.rs) | Everything re-exported by `use logic_nih_plug::prelude::*` |

---

## 2. Build & test

| Task | Command |
|---|---|
| Build one plugin | `cargo build -p <crate>` |
| Build workspace | `cargo build --workspace` (slow) |
| Run all tests (CI) | `cargo test --locked --workspace --features "simd,standalone,zstd"` |
| Smoke build (no VST3) | `cargo build --no-default-features` |
| Bundle plugin | `cargo xtask bundle <pkg> --release` → `target/bundled/` |
| macOS universal | `cargo xtask bundle-universal -p <pkg> --release` |
| List bundled pkgs | `cargo xtask known-packages` |

**Never** `cargo test --all-features` — `logic_nih_plug_iced` has mutually exclusive features. `xtask` alias in [.cargo/config.toml](.cargo/config.toml). CI pins `nightly` for `simd`. See [.github/workflows/build.yml](.github/workflows/build.yml) for Linux GUI/X11 deps.

---

## 3. Workspace layout

| Path | Role |
|---|---|
| [src/](src/) | Core: `Plugin` trait, `Buffer`, `Params`, wrappers, `debug`/`util`/`formatters` |
| `logic_nih_plug_derive/` | `#[derive(Params)]` proc macro |
| `logic_nih_plug_dsp/` | DSP: filters, oscillators, convolution, envelopes, dynamics, reverb, delay, modulation, mixer, analysis (FFT, RealFFT, STFT, WindowingFunction, LevelMeter, Follower, LoudnessMeter, Oscilloscope), pitch (PhaseVocoder, PitchShift, TimeStretching, WindowingFunction re-export), resampling (GenericInterpolator, ZeroOrderHold, Linear, CatmullRom, Lagrange, WindowedSinc). Features: `filters`+`oscillators` (default); `dynamics`/`reverb`/`delay`/`modulation`/`mixer`/`resampling` under `processors`; `analysis` for analysis tools (incl. RealFFT/STFT); `pitch` for pitch/time processing; `full` enables all |
| `logic_nih_plug_gui/` | JUCE-style `Component`/`Button`/FlexBox port + BYO-GUI helpers. Features: `components`, `layout`, `graphics`, `text`, `softbuffer-editor`, `gl-editor`, `full` |
| `logic_nih_plug_egui`/`_iced`/`_vizia` | GUI backends |
| `logic_nih_plug_audio_formats/` | WAV/AIFF (+ optional FLAC/OGG) |
| `logic_nih_plug_data/` | `ValueTree`, `UndoManager`, `CachedValue<T>` |
| `logic_nih_plug_osc/` | OSC sender/receiver, messages, bundles |
| `logic_nih_plug_midi_ci/` | MIDI 2.0 Capability Inquiry |
| `logic_nih_plug_xtask/` + `xtask/` | Bundling lib + shim |
| `logic_nih_plug_graphics/`, `_animation/`, `_crypto/` | 2D primitives, easing/chaining, SHA/MD5/RSA |
| `plugins/examples/` | Example plugins. See [FORMAT_EXAMPLES.md](plugins/examples/FORMAT_EXAMPLES.md) |
| `plugins/soft_vacuum/` etc. | Real plugins; all listed in [bundler.toml](bundler.toml) |

All plugin crates: `crate-type = ["cdylib"]`.

---

## 4. Canonical plugin skeleton

See [plugins/examples/gain/src/lib.rs](plugins/examples/gain/src/lib.rs) — the minimal working reference. Multi-format variants in [FORMAT_EXAMPLES.md](plugins/examples/FORMAT_EXAMPLES.md).

---

## 5. Hard rules

1. **`process()` is real-time.** Zero allocations, no `Mutex::lock`, no `println!`, no syscalls. Debug-only `assert_process_allocs` feature enforces this.
2. **`initialize()` is the only allocation-heavy path.** Everything else is realtime-safe.
3. **Cross-thread comms:** `Arc<AtomicF32>`, `parking_lot::Mutex` (try_lock), crossbeam channels. See `peak_meter` in `plugins/examples/gain_gui_egui/`.
4. **Stable param IDs:** `#[id = "…"]` params, `#[persist = "…"]` state, `#[nested(group = "…")]` groups. Others silently ignored.
5. **`SAMPLE_ACCURATE_AUTOMATION: true`** — wrapper splits buffers at automation points.
6. **Unique per-API constants:** `VST3_CLASS_ID` = `[u8; 16]` (4-char prefix), `CLAP_ID` globally unique.
7. **No large stack arrays on Windows** (tiny audio-thread stack) — use `Box<[T; N]>`.
8. **Use `nih_log!`/`nih_dbg!`** from `logic_nih_plug::debug`, not `println!`/`dbg!`/`assert!`.

---

## 6. Pitfalls

- `simd` needs **nightly** Rust.
- `vst2` requires a local Steinberg SDK copy (no longer redistributed).
- `aax` is a stub — needs Avid SDK + cert + manufacturer ID.
- `au`/`auv3` are macOS/iOS only.
- Don't enable `vst2` + `aax` in one plugin (linker collision).

---

## 7. Conventions

- New real plugin? Copy `plugins/soft_vacuum` (oversampling) or `plugins/safety_limiter` (param callbacks).
- Add a `bundler.toml` entry for CI pickup.
- `nih_export_*!` calls go at the **end** of `lib.rs`.
- Tests: mirror sub-crate layout; `proptest` for property tests, `criterion = "0.5"` for benches.
- JUCE port changes → update [MIGRATION_GUIDE.md](MIGRATION_GUIDE.md), [JUCE_MODULES.md](JUCE_MODULES.md), [VALIDATION_TEST_SUMMARY.md](VALIDATION_TEST_SUMMARY.md).
- Public API changes → [CHANGELOG.md](CHANGELOG.md) + [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md).

---

## 8. Reference by task

| Task | Where to look |
|---|---|
| New plugin format | [FORMAT_EXAMPLES.md](plugins/examples/FORMAT_EXAMPLES.md), [MULTI_FORMAT_EXPORT.md](MULTI_FORMAT_EXPORT.md) |
| New param type / smoothing | [src/params/](src/params/), `logic_nih_plug_derive/` |
| DSP work | [API_REFERENCE.md](API_REFERENCE.md), [BENCHMARKING.md](BENCHMARKING.md) |
| OSC work | [logic_nih_plug_osc/src](logic_nih_plug_osc/src) |
| Ship a release | [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md), [RELEASE_NOTES.md](RELEASE_NOTES.md) |
