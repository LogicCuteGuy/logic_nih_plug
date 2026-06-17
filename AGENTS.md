# AGENTS.md — guidance for AI coding agents

This is a Rust audio-plugin framework (`logic_nih_plug`) with many sub-crates and ~30 example/real plugins. Keep the codebase intact and reach for the smallest correct change.

> **Prefer linking to existing docs over restating them.** This file is the table of contents and the *logic_nih_plug-specific* gotchas only.

---

## 1. Start here

- [README.md](README.md) — project overview, plugin list, framework feature summary.
- [DOCUMENTATION_INDEX.md](DOCUMENTATION_INDEX.md) — index of all the root-level `.md` files.
- [TODO.md](TODO.md) — full backlog of JUCE features still to be ported, with status per crate.
- [QUICK_START.md](QUICK_START.md) — 5-minute intro to the JUCE-ported modules (`logic_nih_plug_dsp`, `logic_nih_plug_audio_formats`, `logic_nih_plug_gui`, `logic_nih_plug_animation`).
- `logic_nih_plug::prelude` — every plugin starts with `use logic_nih_plug::prelude::*;`. Read [src/prelude.rs](src/prelude.rs) when in doubt about what is re-exported.

---

## 2. Build & test

All commands assume the repo root.

| Task | Command |
|---|---|
| Build one plugin | `cargo build -p <crate>` (e.g. `-p gain`, `-p soft_vacuum`) |
| Build everything | `cargo build --workspace` (slow — every plugin compiles) |
| Run all tests (CI-canonical) | `cargo test --locked --workspace --features "simd,standalone,zstd"` |
| No-VST3 smoke build | `cargo build --no-default-features` |
| Bundle a plugin | `cargo xtask bundle <pkg> --release` → output in `target/bundled/` |
| macOS universal bundle | `cargo xtask bundle-universal -p <pkg> --release` |
| List bundled plugins | `cargo xtask known-packages` |
| Per-target | `cargo xtask bundle <pkg> --release --target x86_64-unknown-linux-gnu` |

> **Never** run `cargo test --all-features` — `logic_nih_plug_iced` has mutually exclusive features. The `--features "simd,standalone,zstd"` set above is what CI uses.

The `xtask` alias is defined in [.cargo/config.toml](.cargo/config.toml); it runs the `xtask` shim binary in release mode so `serde(-derive)` is only built once. The shim itself lives in [xtask/src/main.rs](xtask/src/main.rs) and just calls `logic_nih_plug_xtask::main()`. CI pins `dtolnay/rust-toolchain@nightly` because `simd` needs nightly `std::simd`.

Linux CI installs the GUI/X11 dev packages — see [.github/workflows/build.yml](.github/workflows/build.yml).

---

## 3. Workspace layout (high level)

| Path | Role |
|---|---|
| [src/](src/) | The `logic_nih_plug` core library: `Plugin` trait, `Buffer`, `ProcessContext`, `Params` system, per-API wrappers, `debug`/`util`/`formatters`. |
| `logic_nih_plug_derive/` | `#[derive(Params)]` proc macro. |
| `logic_nih_plug_xtask/` + `xtask/` | Bundling library + in-repo shim. Vendored into downstream plugin repos — see its README. |
| `cargo_logic_nih_plug/` | `cargo nih-plug` subcommand (same engine as `cargo xtask`). |
| `logic_nih_plug_egui` / `_iced` / `_vizia` | GUI backends. |
| `logic_nih_plug_gui` | BYO-GUI helpers **and** a JUCE-style `Component`/`Button`/FlexBox port. Features: `components`, `layout`, `graphics`, `text`, `softbuffer-editor`, `gl-editor`, `full`. |
| `logic_nih_plug_dsp` | Filters, oscillators, convolution, envelopes, smoothing, processors, analysis. Features: `filters`, `oscillators` (default), plus optional ones; `full` turns them all on. |
| `logic_nih_plug_audio_formats` | WAV/AIFF (+ optional FLAC/OGG) encoding. |
| `logic_nih_plug_graphics` | 2D primitives (+ optional images/text). |
| `logic_nih_plug_animation` | Easing + chaining. |
| `plugins/examples/` | Bare-format and GUI examples. See [plugins/examples/FORMAT_EXAMPLES.md](plugins/examples/FORMAT_EXAMPLES.md) and [plugins/examples/JUCE_EXAMPLES.md](plugins/examples/JUCE_EXAMPLES.md). |
| `plugins/{soft_vacuum,buffr_glitch,crisp,crossover,diopser,loudness_war_winner,puberty_simulator,safety_limiter,spectral_compressor}/` | Real plugin crates; all listed in [bundler.toml](bundler.toml). |

Each plugin crate declares `crate-type = ["cdylib"]`.

---

## 4. Canonical plugin skeleton

```rust
// filepath: plugins/examples/gain/src/lib.rs
use logic_nih_plug::prelude::*;

struct Gain { params: Arc<GainParams> }

#[derive(Params)]
struct GainParams {
    #[id = "gain"]
    pub gain: FloatParam,
}

impl Default for Gain {
    fn default() -> Self {
        Self { params: Arc::new(GainParams {
            gain: FloatParam::new("Gain", 0.0,
                FloatRange::Linear { min: -60.0, max: 0.0 })
                .with_smoother(SmoothingStyle::Logarithmic(50.0))
                .with_unit(" dB"),
        }) }
    }
}

impl Plugin for Gain {
    const NAME: &'static str = "Gain";
    const VENDOR: &'static str = "...";
    const URL: &'static str = "...";
    const EMAIL: &'static str = "...";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[ /* first = default */ ];
    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> { self.params.clone() }
    fn process(&mut self, buffer: &mut Buffer, _aux: &mut AuxiliaryBuffers,
               _ctx: &mut impl ProcessContext<Self>) -> ProcessStatus { ProcessStatus::Normal }
    fn editor(&mut self, _ex: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> { None }
    fn initialize(&mut self, _bus: &BusLayout, _buf: &BufferConfig,
                  _ctx: &mut impl InitContext<Self>) -> bool { true }
    fn reset(&mut self) {}
    fn deactivate(&mut self) {}
}

impl ClapPlugin for Gain {
    const CLAP_ID: &'static str = "...";
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect];
}
impl Vst3Plugin for Gain {
    const VST3_CLASS_ID: [u8; 16] = *b"GainPlug_________";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Fx];
}

nih_export_clap!(Gain);
nih_export_vst3!(Gain);
```

Live reference: [plugins/examples/gain/src/lib.rs](plugins/examples/gain/src/lib.rs).

For the multi-format, VST2, AU, AUv3, LV2, and AAX variants see [plugins/examples/FORMAT_EXAMPLES.md](plugins/examples/FORMAT_EXAMPLES.md).

---

## 5. Hard rules — break these and the plugin is wrong

1. **`process()` is real-time.** No allocations, no `std::sync::Mutex` blocking, no `println!`, no syscalls. `String` formatting allocates. The `assert_process_allocs` feature (debug only) enforces this — temporarily disable it when debugging panics in DSP code.
2. **`initialize()` is the only place that's allowed to allocate heavily.** Everything else must be realtime-safe.
3. **Cross-thread communication is `Arc<AtomicF32>` / `parking_lot::Mutex` (try_lock) / crossbeam channels.** See `peak_meter` in `plugins/examples/gain_gui_egui/`.
4. **Params need stable IDs.** `#[id = "…"]` for parameters, `#[persist = "…"]` for non-parameter state, `#[nested(group = "…")]` for grouping, `#[nested(array, group = "…")]` for arrays. Anything else is silently ignored.
5. **`SAMPLE_ACCURATE_AUTOMATION: true`** is what most plugins want — the wrapper splits buffers at automation points.
6. **Per-API trait constants must be globally unique**: `VST3_CLASS_ID` is a `[u8; 16]` (use a 4-char ASCII prefix), `CLAP_ID` must be unique across all types in one `nih_export_clap!` (debug-asserted).
7. **Windows has tiny audio-thread stack sizes.** Don't put large arrays on the stack (`let arr = [0.0; 4096];`) — use `Box<[T; N]>` (see `plugins/soft_vacuum`).
8. **Use `nih_log!`/`nih_dbg!`/`assert_*` from `logic_nih_plug::debug`** instead of `println!`/`eprintln!`/`dbg!`/`assert!`.

---

## 6. Pitfalls & environment quirks

- **`simd` requires nightly Rust.** CI pins `dtolnay/rust-toolchain@nightly`.
- **`vst2` cannot be built out-of-the-box.** Steinberg no longer redistributes the VST2 SDK; `vst2-sys` expects a local copy.
- **`aax` is a stub feature.** Building `gain_aax` requires the AAX SDK from Avid + a registered manufacturer/product ID + a code-signing cert.
- **`au` / `auv3` are macOS/iOS only** (gated by `cfg!(target_os = "macos")`) and won't even attempt to build on Linux/Windows.
- **Don't enable `vst2` and `aax` in the same plugin** — they collide at the linker.
- **`target/` is git-ignored**; it has a `CACHEDIR.TAG` for system caches. Windows CI does *not* cache `target/` (runner overflows).

---

## 7. Conventions when adding code

- **Match the style of an existing plugin first.** For a new real plugin, copy `plugins/soft_vacuum` (oversampling + smoothing) or `plugins/safety_limiter` (parameter callbacks) rather than starting from scratch.
- **Add a `bundler.toml` entry** when introducing a real plugin so CI picks it up via `cargo xtask known-packages`.
- **All plugin crate types are `cdylib`.** `nih_export_*!` calls go at the **end** of `lib.rs` after the trait impls.
- **For tests, mirror the sub-crate's existing layout** (e.g. `logic_nih_plug_dsp/tests/`). Use `proptest` for property tests; the framework already standardises on `criterion = "0.5"` for benches.
- **Update the JUCE port validation docs** when changing a ported module: [MIGRATION_GUIDE.md](MIGRATION_GUIDE.md), [JUCE_MODULES.md](JUCE_MODULES.md), [VALIDATION_TEST_SUMMARY.md](VALIDATION_TEST_SUMMARY.md).
- **Bumps to the public framework API** should add an entry to [CHANGELOG.md](CHANGELOG.md) and be checked against [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md).

---

## 8. Where to look next

- **Want to add a new plugin format?** Read [plugins/examples/FORMAT_EXAMPLES.md](plugins/examples/FORMAT_EXAMPLES.md) and [MULTI_FORMAT_EXPORT.md](MULTI_FORMAT_EXPORT.md).
- **Want to add a parameter smoothing style / new param type?** Look at [src/params/](src/params/) and the derive crate `logic_nih_plug_derive/`.
- **Want to touch DSP?** [API_REFERENCE.md](API_REFERENCE.md) is the API surface for ported modules; [BENCHMARKING.md](BENCHMARKING.md) explains how to add a `criterion` bench.
- **Want to ship a release?** [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md), [RELEASE_NOTES.md](RELEASE_NOTES.md), [RELEASE_SUMMARY.md](RELEASE_SUMMARY.md) — and the CI workflows in [.github/workflows/](.github/workflows/).
