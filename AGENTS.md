# AGENTS.md — guidance for AI coding agents

Rust audio-plugin framework (`logic_nih_plug`). ~30 plugins, many sub-crates. **Smallest correct change; keep the codebase intact.**

> Prefer linking to existing docs over restating them. See [DOCUMENTATION_INDEX.md](DOCUMENTATION_INDEX.md) for the full index.

---

## 1. Start here

| Doc | What it covers |
|---|---|
| [README.md](README.md) | Project overview, plugin list, features |
| [TODO.md](TODO.md) | JUCE port backlog with per-crate status — **check this first when starting a new port** |
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
| `logic_nih_plug_gui/` | JUCE-style `Component`/`Button`/FlexBox port + BYO-GUI helpers. **`controls_extra`** module adds `ComboBox`, `TextEditor`, `ToggleButton`, `CheckBox`, `ProgressBar`, `Tooltip`, `DrawableButton`, `HyperlinkButton`, `ImageComponent` (with `ImageScalingMode` enum), `MidiKeyboardComponent` (visual piano keyboard with mouse interaction, active-note highlighting, `KeyboardOrientation`). All wrap `Component`, support closure callbacks, and have `render()` / `render_with_lookandfeel()` (feature-gated on `graphics`). **`layout`** module: `CssGrid` (CSS Grid with `fr`/`px`/`auto`/`minmax()` track sizing, named areas, spanning, gaps), `RelativeCoordinate`/`RelativeRectangle` (percentage-based positioning), `AnimationFrameRate` (throttled redraw), plus existing `FlexLayout`/`FlexBox`/`GridLayout`/`AbsoluteLayout`. **`opengl`** module (feature `gl-editor`): `OpenGLContext`, `ShaderProgram`, `OpenGLHelpers`, `OpenGLTexture`, `OpenGLFrameBuffer`, `OpenGLRenderer` trait + `RenderLoopDriver`, `Matrix3D`/`Matrix4x4`. 262 unit tests. Features: `components`, `layout`, `graphics`, `text`, `softbuffer-editor`, `gl-editor`, `full` |
| `logic_nih_plug_egui`/`_iced`/`_vizia` | GUI backends |
| `logic_nih_plug_audio_formats/` | WAV/AIFF (+ optional FLAC/OGG) |
| `logic_nih_plug_audio_basics/` | `AudioSampleBuffer`, `AudioChannelSet`, `MidiMessage`, `MidiRPN`, `MidiClock`, `MtcRate`/`MtcTime`/`MtcEncoder`/`MtcFullFrame` |
| `logic_nih_plug_audio_devices/` | `AudioDeviceManager`, `AudioIODevice` trait, `AudioIODeviceType` / `DriverType` enums, `AudioDeviceSetup`, `AudioIODeviceCallback` trait, `MockAudioIODevice`. Concrete driver bindings (cpal, coreaudio-rs, asio-sys, …) are intentionally NOT bundled — they plug in by implementing `AudioIODevice`. Features: `manager` (default), `full` = `["manager"]` |
| `logic_nih_plug_data/` | `ValueTree`, `UndoManager`, `CachedValue<T>` |
| `logic_nih_plug_osc/` | OSC sender/receiver, messages, bundles |
| `logic_nih_plug_midi_ci/` | MIDI 2.0 Capability Inquiry |
| `logic_nih_plug_product_unlocking/` | JUCE-style product unlocking: server-side keyfile generator (`generate_key_file` / `generate_expiring_key_file` using textbook RSA `m^d mod n`), client-side state machine (`OnlineUnlockStatus<S: UnlockStore>` with `apply_key_file`, `attempt_webserver_unlock`, `load`/`save`), and `machine_id` helpers (platform prefix, encoded IDs). Depends on `logic_nih_plug_crypto` and `logic_nih_plug_data`. 22 tests + 3 doc-tests. Features: `key_generation` (default), `online_unlock_status` (default), `full` |
| `logic_nih_plug_video/` | Video playback: `VideoFrame` (RGBA8888), `VideoDecoder` (ffmpeg-next, feature `decoder`), `VideoComponent` (GUI, feature `gui`). 39 tests. Features: `decoder` (default), `gui`, `full` |
| `logic_nih_plug_audio_processors/` | Host-side plugin discovery/management: `PluginDescription`, `PluginFormat` trait, `PluginFormatType`, `KnownPluginList`, `PluginDirectoryScanner`, `NullPluginFormat`. 79 tests. Features: `scanner`, `full` |
| `logic_nih_plug_audio_formats::midi_file[_player]` | SMF (`.mid`) read/write + tempo-aware transport (`MidiFilePlayer`). Feature `midi` (default off). 31 tests. `TempoEvent` / `TimeSignature` / `KeySignature` types live in `logic_nih_plug_audio_basics::mtc`. |
| `logic_nih_plug_gui::commands` | `ApplicationCommandManager`, `KeyPress`, `KeyPressMappingSet`, `CommandInfo`, `ApplicationCommandTarget` trait. 28 tests. |
| `logic_nih_plug_gui::multi_doc_panel` | Tabbed MDI container. `MultiDocumentPanel`, `MultiDocumentPanelLayout`. 22 tests. |
| `logic_nih_plug_xtask/` + `xtask/` | Bundling lib + shim |
| `logic_nih_plug_graphics/`, `_animation/`, `_crypto/` | 2D primitives + vector graphics (tiny-skia backed `Painter` / `Path` / `Stroke` / `ColourGradient` / `DropShadow`), glyph arrangement, image rescale/convolve, easing/chaining, SHA/MD5/RSA (incl. `RSAKey::d_bytes` for raw textbook-RSA ops) |
| `plugins/examples/dsp/` | JUCE DSP port demos — `juce_distortion_demo`, `juce_oscillator_demo`, `juce_iir_filter_demo`, `juce_phaser_demo`, `juce_chorus_demo`, `juce_convolution_demo`, `juce_noise_gate_demo`, `juce_limiter_demo`. All `cdylib`, bundled via `bundler.toml`. |
| `plugins/examples/plugins/` | Plugin host examples — `juce_audio_plugin_host_egui` (scanner + editor scaffolding, lib target) |
| `plugins/examples/gui/` | GUI port demos — future JUCE GUI example ports (currently scaffolded) |
| `plugins/examples/` | Example plugins (excluding `dsp/`, `plugins/`, `gui/`). See [FORMAT_EXAMPLES.md](plugins/examples/FORMAT_EXAMPLES.md) |
| `examples/Audio/` | Standalone audio apps: `audio_playback_demo` (WAV playback via `MockAudioIODevice`), `audio_recording_demo` (WAV recording), `audio_workgroup_demo` (2-node shared buffer). All `[[bin]]` targets. |
| `examples/Plugins/` | `plugin_host_cli` — headless CLI plugin scanner + list printer (`[[bin]]`). |
| `examples/Utilities/` | `wav_reader`, `wav_writer`, `midi_file_inspector`, `osc_sender_demo`, `osc_receiver_demo`. All `[[bin]]` file-IO / OSC demos. |
| `examples/DemoRunner/` | `juce_demorunner` — GUI showcase with mutually exclusive `gui-egui` / `gui-iced` / `gui-vizia` backends (lib + bin). |
| `examples/` | Categorized gallery (see [examples/README.md](examples/README.md)). `audio-assets/` and `midi-assets/` hold reference WAV/MIDI fixtures for doc-tests. |
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
| `juce_core` / `juce_events` equivalents | §9 below — no `logic_nih_plug_core` crate, use stdlib + existing deps |

---

## 9. `juce_core` / `juce_events` — no crate, use stdlib + existing deps

Every JUCE item in [TODO.md](TODO.md) §4 is already covered by Rust
stdlib or by a dep already in the workspace tree. **Do not** create a
`logic_nih_plug_core` (or similar) crate that wraps them — that's just
cargo-culting `pub struct X(pub Y);`. Use the stdlib equivalents directly:

| JUCE item | Rust equivalent | Notes |
|---|---|---|
| `File` | [`std::path::PathBuf`](https://doc.rust-lang.org/std/path/struct.PathBuf.html) + [`std::fs`](https://doc.rust-lang.org/std/fs/index.html) | Path metadata via `std::fs::metadata`. |
| `String` wrapper | [`std::string::String`](https://doc.rust-lang.org/std/string/struct.String.html) | Rust has UTF-8 by default — JUCE's `juce::String` workarounds don't apply. |
| `Array<T>` / `OwnedArray<T>` / `ReferenceCountedArray<T>` | [`Vec<T>`](https://doc.rust-lang.org/std/vec/struct.Vec.html) / `Vec<Box<T>>` / [`Arc<[T]>`](https://doc.rust-lang.org/std/sync/struct.Arc.html) | Use `Arc<Vec<T>>` only if you need shrinking after sharing. |
| `Thread` | [`std::thread::JoinHandle`](https://doc.rust-lang.org/std/thread/struct.JoinHandle.html) | Already used throughout [src/event_loop/background_thread.rs](src/event_loop/background_thread.rs). |
| `ThreadPool` | `rayon` | Already in the dep tree via the workspace. |
| `WaitableEvent` | `crossbeam_channel` (already in tree) or `parking_lot::Mutex` + `Condvar` | See [src/event_loop/background_thread.rs](src/event_loop/background_thread.rs) for the canonical pattern. |
| `Time` / `RelativeTime` | [`std::time::Instant`](https://doc.rust-lang.org/std/time/struct.Instant.html) / [`std::time::Duration`](https://doc.rust-lang.org/std/time/struct.Duration.html) | Monotonic by construction. |
| `HighResolutionTimer` | `std::time::Instant::elapsed()` | Linux/macOS use `clock_gettime`; Windows uses `QueryPerformanceCounter`. Add `quanta` only if a sub-µs monotonic source is ever needed. |
| `MessageManager::call_soon` (single-threaded GUI dispatch) | [`logic_nih_plug::event_loop::EventLoop::schedule_gui`](src/event_loop.rs) | Already the realtime-safe → GUI-thread hop the framework provides. |
| `AsyncUpdater` (realtime-safe trigger → message-loop dispatch) | Same as `MessageManager::call_soon` — use `EventLoop::schedule_gui` | Realtime-safe enqueue, dispatched on the GUI thread. |
| `Timer` (periodic tick) | Backend-native: `egui::Context::request_repaint_after`, `iced::Subscription::interval`, `vizia::view::timer` | No shared abstraction; every backend already does this idiomatically. |

If you find yourself reaching for a wrapper crate for any of these,
stop and check whether the stdlib version already does what you need —
it almost certainly does.
