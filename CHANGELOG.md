# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic
Versioning](https://semver.org/spec/v2.0.0.html).

Since there is no stable release yet, the changes are organized per day in
reverse chronological order. The main purpose of this document in its current
state is to list breaking changes.

## [2026-06-19]

### Added

- **`juce_product_unlocking` port** — new crate
  `logic_nih_plug_product_unlocking`:
  - `key_generation` module with `generate_key_file` /
    `generate_expiring_key_file` (server-side, textbook RSA
    `m^d mod n`) and `decrypt_key_file` /
    `KeyFileData` (client-side). Matches `juce::KeyGeneration`
    line-for-line on the keyfile format (human-readable comment
    header + `#`-prefixed 70-char-per-line hex blob).
  - `online_unlock_status` module with the
    `OnlineUnlockStatus<S: UnlockStore>` state machine — generic
    over a user-supplied store trait so the same code drives any
    marketplace backend. Implements `apply_key_file`,
    `attempt_webserver_unlock`, `load`/`save`, `is_unlocked`,
    `get_expiry_time_ms`, `clear`, plus the
    `LicenseResult` constants from JUCE's
    `juce::OnlineUnlockStatus::LicenseResult`.
  - `machine_id` module with `get_platform_prefix` /
    `get_encoded_id_string` / `get_unique_machine_id` /
    `get_local_machine_ids` (mirrors
    `juce::OnlineUnlockStatus::MachineIDUtilities`).
  - 22 unit tests + 3 doc-tests passing under `--features full`
    (default = `key_generation` + `online_unlock_status`).
- **`logic_nih_plug_crypto::RSAKey::d_bytes`** — accessor for the
  private exponent as a big-endian byte string. Lets callers (the
  `key_generation` module in particular) do textbook RSA
  `m^d mod n` directly without going through the `rsa` crate's
  PKCS#1-v1.5-only public surface. Returns `None` for public-only
  keys. Adds 1 round-trip + 1 doc-style use-test to the existing
  45-test / 4-doc-test suite in `logic_nih_plug_crypto`.
- **`logic_nih_plug_audio_devices::MockAudioIODeviceEvent` re-exported** — the `MockAudioIODeviceEvent` enum (Opened/Closed/Started/Stopped) is now accessible from the public API for writing deterministic lifecycle assertions against `MockAudioIODevice` without internal crate access. One new public re-export; no breaking changes.
- **JUCE-style Examples Portfolio** — 22 new crates porting JUCE `examples/` to Rust, organized in 5 user stories + Polish. Covers DSP plugins, standalone audio apps, a plugin host, file format utilities, and a multi-backend GUI showcase. All new crates compile, have unit tests, and are registered in `bundler.toml` and the workspace. Key crates added under `plugins/examples/dsp/` (8 DSP plugin demos), `examples/Audio/` (3 standalone audio apps), `examples/Plugins/` (headless plugin host CLI), `examples/Utilities/` (WAV reader/writer, MIDI inspector, OSC sender/receiver), and `examples/DemoRunner/juce_demorunner` (GUI showcase with egui/iced/vizia backends). New integration tests under `tests/` (id_uniqueness, readme_required, categorized). Updated `examples/README.md` categorized gallery. CI now builds DemoRunner with all 3 backends via `demorunner-backends` matrix in `.github/workflows/build.yml`.

## [2026-06-18]

### Added

- **`juce_audio_utils` port** — split across `logic_nih_plug_audio_formats`
  (SMF + transport) and `logic_nih_plug_gui` (commands + MDI):
  - `MidiFile` / `MidiFileTrack` / `MidiFileEvent` /
    `MidiFileFormat` in `logic_nih_plug_audio_formats::midi_file` —
    Standard MIDI File reader/writer supporting Format 0 + 1 + 2,
    PPQN time-base, VLQ-encoded delta times, running status,
    meta events (tempo / time signature / key signature / end of
    track), SysEx. 20 unit tests + 1 doc-test.
  - `MidiFilePlayer` in `logic_nih_plug_audio_formats::midi_file_player` —
    tempo-aware transport: `set_position_ticks` /
    `set_position_seconds`, `get_next_midi_block` emits events with
    sample-offset `time_stamp`, `loop_range`, `total_seconds`,
    `is_finished`. 11 unit tests + 1 doc-test.
  - `MidiMessage::meta_event(type, data)`,
    `tempo_meta(TempoEvent)`, `time_signature_meta(TimeSignature)`,
    `key_signature_meta(KeySignature)`, `end_of_track_meta()`,
    plus `meta_type()` / `meta_data()` accessors in
    `logic_nih_plug_audio_basics` (so SMF callers can construct
    meta events without hand-rolling bytes).
  - `TempoEvent` / `TimeSignature` / `KeySignature` types in
    `logic_nih_plug_audio_basics::mtc`, re-exported at crate root
    (the canonical implementations; the SMF reader/writer re-uses
    them).
  - `ApplicationCommandManager` family in
    `logic_nih_plug_gui::commands` — `CommandId`, `CommandFlags`
    bitflags, `KeyPress` (with virtual-key constants and
    `get_text_description()`), `CommandInfo` builder,
    `KeyPressMappingSet` (bidirectional mapping with compact-string
    round-trip), `ApplicationCommandTarget` trait, and the manager
    itself with chain dispatch. 28 unit tests + 1 doc-test.
  - `MultiDocumentPanel` in `logic_nih_plug_gui::multi_doc_panel` —
    tabbed MDI container with `MultiDocumentPanelLayout`
    (`FloatingWindows` / `MaximisedWindowsWithTabs`),
    `add_document(component, owned)` / `close_document(idx)` /
    `set_active_document_by_index` / `set_max_documents` /
    `set_background_colour(rgba)`. 22 unit tests + 1 doc-test.
  - **Skipped** (deferred to a follow-up):
    `AudioThumbnail` (data + GUI component — needs an async source
    loader and the GUI render path),
    `AudioFilePlayer` (small but not yet wired in).
  - All `logic_nih_plug_audio_formats` SMF/MIDI features are
    feature-gated on `midi` (default off, like `flac`/`ogg`) so
    the default build doesn't take on `logic_nih_plug_audio_basics`.

- **`logic_nih_plug_video`** — video playback crate ported from JUCE's
  `juce_video` module:
  - `VideoFrame` — RGBA8888 frame with pixel accessors and solid-colour
    test factory
  - `VideoDecoder` — ffmpeg-next wrapper for file-based decoding
    (open, next_frame, seek, rewind); feature-gated on `decoder`
  - `VideoComponent` — GUI component with play/pause/stop, push_frame
    for external decoders, callbacks, speed/volume/position control;
    feature-gated on `gui`
  - `PlaybackState` enum, `VideoError` (7 variants)
  - 39 unit tests; decoder feature is optional so core types compile
    without FFmpeg system libraries

- **`logic_nih_plug_audio_processors`** — host-side plugin discovery,
  scanning, and management ported from JUCE's `juce_audio_processors`:
  - `PluginDescription` — immutable metadata (name, manufacturer, version,
    format, unique ID, channel counts, file path) with tab-delimited
    compact serialization
  - `PluginFormat` trait — format-specific scanning/loading abstraction
  - `PluginFormatType` — 8 variants with platform detection, extensions,
    default search paths
  - `KnownPluginList` — persistent registry with deduplication, sorting
    (6 methods), change listener, serialization
  - `PluginDirectoryScanner` — incremental scanner with blacklisting,
    dead-man's-pedal file I/O, progress reporting
  - 65 unit tests + 14 doc-tests

- **`logic_nih_plug_gui` OpenGL utilities** — reusable OpenGL abstractions
  ported from JUCE's `juce_opengl` module, built on `glow`:
  - `OpenGLContext` wrapper with GL state helpers (clear, viewport, blend, depth, cull)
  - `ShaderProgram` (compile + link + uniform setters) + `OpenGLHelpers` static methods
  - `OpenGLTexture` (create, upload, bind, mipmaps, filtering, wrapping)
  - `OpenGLFrameBuffer` (FBO with color + depth/stencil, read-back, resize)
  - `OpenGLRenderer` trait + `RenderLoopDriver` frame-loop helper
  - `Matrix3D` / `Matrix4x4` (column-major, perspective/ortho/look-at/rotation)
  - Feature-gated on `gl-editor`; 66 unit tests.

- **`logic_nih_plug_gui` MidiKeyboardComponent** — visual MIDI piano
  keyboard control ported from JUCE: white/black key rendering, mouse
  interaction with velocity, external active-note highlighting,
  configurable range/orientation/colours, two-tier render, 18 tests.
  Re-exported as `MidiKeyboardComponent` and `KeyboardOrientation`.

- **`logic_nih_plug_gui` layout expansion** — new CSS Grid, relative
  coordinates, and animation frame rate modules:
  - `CssGrid` — full CSS Grid layout with `GridTrack` sizing (`Fraction`
    / `Fixed` / `Auto` / `MinContent` / `MaxContent` / `MinMax`),
    two-pass resolution (fixed tracks first, then `fr` distribution),
    row and column gaps, named `NamedArea` support, `GridPlacement`
    (`cell`, `area`, `named`), `GridItem` with per-item alignment
    (`Start` / `End` / `Center` / `Stretch`), and `Rect` output.
  - `RelativeCoordinate` — percentage-based or absolute pixel coordinates
    with `Absolute` / `Percent` / `FromRight` / `FromBottom` variants and
    `resolve_horizontal` / `resolve_vertical` methods.
  - `RelativeRectangle` — four `RelativeCoordinate` edges for proportional
    child bounds; `fill()`, `from_pixels()`, `from_percent()` constructors.
  - `AnimationFrameRate` — throttled redraw controller with configurable
    FPS or custom `Duration` interval, `should_frame` / `should_frame_duration`
    gating, time-until-next queries, enable/disable, and reset.
  - All types re-exported at `logic_nih_plug_gui` crate root.
  - 59 new unit tests (22 CSS Grid + 19 relative + 18 animation frame rate).

- **`logic_nih_plug_graphics` vector module** — JUCE-style 2D vector
  graphics backed by `tiny-skia` (opt-in `vector` feature):
  - `PathBuilder` — chainable `move_to` / `line_to` / `quad_to` /
    `cubic_to` / `close` / `start_new_sub_path` wrapping
    `tiny_skia::PathBuilder`.
  - `Stroke`, `FillType`, `Justification` (bitflags) — JUCE API
    parity types; `Stroke` and `FillRule` re-exported from
    `tiny_skia`.
  - `ColourGradient` — linear and radial gradient fills (wraps
    `tiny_skia::Shader<'static>`). Accepts `GradientStop` +
    `SpreadMode`, matching JUCE's `ColourGradient` constructor
    semantics.
  - `DropShadow` — colour + offset compositing; foreground rendered
    on top of an offset shadow pass.
  - `Painter` — CPU paint target backed by `tiny_skia::Pixmap`;
    premultiplied-RGBA8 output with `data_straight()` for
    straight-alpha consumers. Fill/stroke paths, rects, gradient
    fills, shadow compositing.
  - Common `tiny_skia` types re-exported at crate root (`Path`,
    `Stroke`, `Paint`, `Shader`, `GradientStop`, `SpreadMode`,
    `BlendMode`, `FillRule`, `LineCap`, `LineJoin`) for ergonomic
    `use logic_nih_plug_graphics::Path`.
  - 27 unit tests + 1 doc-test for the vector module.

- **`logic_nih_plug_graphics` text extensions**:
  - `GlyphArrangement` — shaped text layout with `PositionedGlyph`
    entries; `from_text(font, text, size, origin_x, origin_y)`,
    `translate(dx, dy)`, `width()`, `glyphs()`.
  - `LineSpacing` — `Single` / `Multiple(f32)` / `Fixed(f32)`
    enum with `line_distance(font, size)` helper.
  - `Font::get_string_width_float` (alias for `measure_text`),
    `Font::get_ascent`, `Font::get_descent` (positive convention),
    `Font::get_height`.
  - 11 new unit tests for text extensions.

- **`logic_nih_plug_graphics` image extensions**:
  - `Image::rescaled(new_w, new_h, RescaleFilter)` — `Nearest` /
    `Bilinear` up/downscale backed by `image` crate resize.
  - `Image::convolve` / `Image::convolve_in_place` — edge-clamped
    NxN convolution with arbitrary `f32` kernel.
  - `ImageConvolutionEngine` — reusable convolution engine with
    `box_blur_3x3`, `sharpen_3x3`, `edge_detect_3x3` presets.
  - `Image::new(w, h)` — transparent blank image constructor.
  - 12 new unit tests for image extensions.

### Added

- **`logic_nih_plug_audio_basics`** — new crate porting
  `juce_audio_basics` for the `logic_nih_plug` ecosystem:
  - `AudioSampleBuffer` — non-interleaved (JUCE-default) audio sample
    container with `interleave` / `deinterleave` helpers,
    `apply_gain` / `apply_channel_gains` / `clear` / `copy_from` /
    `add_from` / `set_size`.
  - `AudioChannelSet` — speaker / channel layouts: `Mono`, `Stereo`,
    `Lrc`, `Lrs`, `Quadraphonic`, `FiveDotZero`, `FiveDotOne`,
    `SixDotOne`, `SevenDotZero`, `SevenDotOne`, `SevenDotOnePointTwo`,
    `SevenDotOnePointFour`, `Ambisonic(order)` (with
    `MAX_AMBISONIC_ORDER = 7`), and `Custom(n)`. Channel names
    include abbreviation / description / speaker position.
  - `MidiMessage` — value type (`Vec<u8>` payload + `i32` time
    stamp) with builder methods for every common MIDI channel and
    system message, plus `parse` for the inverse direction
    (single-pass, no allocations).
  - `MidiRPN` / `MidiRpnKind` — RPN vs NRPN encoding/decoding for
    7-bit and 14-bit values, with `standard_rpn::*` constants for
    well-known RPN numbers (pitch-bend sensitivity, MPE configuration,
    tuning program/bank, RPN null, …).
  - `MidiClock` — sample/tick/QN math for the 24-ppqn MIDI clock
    (`samples_per_clock_tick`, `samples_to_ticks`, `ticks_to_samples`,
    `split_tick_delta`, BPM ↔ microseconds-per-QN).
  - `MtcRate` / `MtcTime` / `MtcEncoder` / `MtcFullFrame` — MTC
    timecode (24 / 25 / 29.97 drop / 30 fps), drop-frame-aware
    `to_frame_count` / `from_frame_count`, driven-by-`encode_frame()`
    encoder, plus the `[0xF0, 0x7F, 0x7F, 0x01, 0x01, hh, mm, ss, ff,
    0xF7]` full-frame SysEx form.
  - 81 unit tests + 4 doc-tests; default features
    `["buffer", "midi"]`, with `full` re-exporting everything.

- **FFT upgrades** (`logic_nih_plug_dsp::analysis`) — real-only FFT,
  STFT, and a top-level `WindowingFunction` re-export:
  - `RealFFT` — real-only FFT path: real input → one-sided complex
    spectrum (N/2+1 bins) and back. Uses `rustfft::FftPlanner`
    plans (forward + inverse) and exploits Hermitian symmetry.
  - `STFT` — short-time Fourier transform with per-frame
    `analyze_frame` / `synthesize_frame` and one-shot
    `process_block` (windowed OLA with COLA normalisation).
  - `WindowingFunction` — moved from `processors::pitch` to the
    `analysis` module for cross-cutting access; the pitch module
    re-exports it for backwards compatibility.
  - 17 unit tests + 4 doc-tests for `RealFFT` and `STFT`; 9
    windowing tests now under `analysis`; gated behind the
    existing `analysis` feature flag.

### Added

- **`logic_nih_plug_gui` controls_extra module** (`controls_extra.rs`) —
  JUCE-style high-level control types wrapping `Component`:
  - `ComboBox` — drop-down selection control with items, selection state,
    and change callbacks via closures.
  - `TextEditor` — editable text field supporting single/multi-line,
    max length, read-only mode, cursor position, and insert operations.
  - `ToggleButton` — button that toggles between on/off states with
    a change callback.
  - `CheckBox` — boolean checkbox with label and checked-change callback.
  - `ProgressBar` — horizontal progress bar (0.0–1.0) with optional
    display text.
  - `Tooltip` — tooltip manager with show/hide/delay configuration.
  - `DrawableButton` — button for custom-drawn content via closures.
  - `HyperlinkButton` — button styled as a hyperlink with an associated URL.
  - `ImageComponent` — image display with `ImageScalingMode` enum
    (`Fill`, `Fit`, `None`, `Stretch`).
  - All controls have `render()` and `render_with_lookandfeel()` methods,
    feature-gated behind the `graphics` feature.
  - 46 unit tests, all passing.

### Changed

- **PhaseVocoder** (`logic_nih_plug_dsp::processors::pitch`) — now
  uses `RealFFT` internally instead of two separate
  `Arc<dyn rustfft::Fft<f32>>` plans. Spectrum buffer is now
  one-sided (`fft_size/2 + 1` bins), the manual mirror-after-phase-
  processing step is gone, and the FFT path is a single source of
  truth across `analysis` and `pitch`. All 20 pitch tests pass.

- **`logic_nih_plug_audio_devices`** — new crate porting
  `juce_audio_devices` for the `logic_nih_plug` ecosystem:
  - `AudioDeviceSetup` — desired sample rate, buffer size, and
    input / output channel counts; helpers `stereo_44100` /
    `stereo_48000`. Validates against the active device's
    capabilities when one is attached.
  - `AudioIODeviceType` — enum of every driver backend JUCE
    supports (`CoreAudio`, `ASIO`, `WASAPI`, `DirectSound`, `ALSA`,
    `JACK`, `AndroidAAudio`, `AndroidOpenSLES`, `Bela`, `IOSAudio`,
    `WebAudio`, `Dummy`), with `type_name()`, `description()`,
    `is_supported_on_current_platform()`, and
    `supported_on_current_platform()`.
  - `DriverType::current()` — compile-time detection of the
    preferred driver for the current build target.
  - `AudioIODevice` trait — `get_name`, `get_device_info`,
    `get_output_channel_names`, `get_input_channel_names`,
    `get_default_buffer_size`, `get_current_buffer_size`,
    `get_current_sample_rate`, `get_input_latency_in_samples`,
    `get_output_latency_in_samples`, `open` / `close` / `is_open`,
    `start` / `stop` / `is_playing`, `get_last_error`, and optional
    `has_control_panel` / `show_control_panel`. Concrete driver
    integrations (cpal, coreaudio-rs, asio-sys, …) plug in by
    implementing this trait.
  - `AudioDeviceInfo` — name, sample rates, buffer sizes, channel
    names, latencies; `validate_sample_rate` /
    `validate_buffer_size` helpers; `closest_sample_rate` /
    `closest_buffer_size` for nearest-neighbour fallback.
  - `AudioIODeviceCallback` trait — `audio_device_about_to_start`,
    `audio_device_io_callback`, `audio_device_stopped`
    (real-time safe; mirrors `juce::AudioIODeviceCallback`); plus
    `AudioIODeviceCallbackData<'a>` with parallel input / output
    slice arrays and `NullAudioIODeviceCallback` for tests.
  - `AudioDeviceManager` — long-lived orchestrator; holds the
    active `Box<dyn AudioIODevice>`, the desired
    `AudioDeviceSetup`, the current `AudioIODeviceType`, and the
    listener list. Lifecycle: `set_current_audio_device_type`,
    `set_current_audio_device`, `set_audio_device_setup` (reopens
    the active device if it's open), `open_device`, `play`, `stop`,
    `close_device`, `scan_device_names`, `play_test_sound` (no-op
    stub).
  - `AudioDeviceManagerState` — `Stopped` / `Open` / `Playing`
    tri-state machine.
  - `AudioDeviceManagerListener` trait — fires on device-type
    change (`audio_device_manager_changed`) and setup change
    (`audio_device_setup_changed`).
  - `MockAudioIODevice` — concrete in-memory device for tests /
    headless hosts, with `event_log` + `callback_count` +
    `force_error` inspection helpers.
  - 64 unit tests + 1 doc-test passing under `--features full`.
  - Default features: `["manager"]`; `full` re-exports everything.
  - Concrete driver bindings (`cpal`, `coreaudio-rs`, `asio-sys`, …)
    are intentionally NOT bundled — they require platform SDKs.
    The trait surface here is the integration point.
- **`juce_core` essentials & `juce_events`** — TODO §4 & §4.5 shrunk:
  no new crate. Every JUCE item here is already covered by Rust
  stdlib or by a dep already in the workspace tree. Use the stdlib
  equivalents directly (full table in
  [AGENTS.md](AGENTS.md) §9):
  - `File` → `std::path::PathBuf` + `std::fs`
  - `Array<T>` / `OwnedArray<T>` / `ReferenceCountedArray<T>` →
    `Vec<T>` / `Vec<Box<T>>` / `Arc<[T]>`
  - `Thread` → `std::thread::JoinHandle`
  - `ThreadPool` → `rayon` (already in dep tree)
  - `WaitableEvent` → `crossbeam_channel` (already in dep tree)
  - `Time` / `RelativeTime` → `std::time::Instant` / `Duration`
  - `HighResolutionTimer` → `std::time::Instant::elapsed()`
  - `AsyncUpdater` / `MessageManager::call_soon` →
    `logic_nih_plug::event_loop::EventLoop::schedule_gui` (already
    the realtime-safe → GUI-thread hop the framework provides)
  - `Timer` (periodic tick) → backend-native:
    `egui::Context::request_repaint_after`,
    `iced::Subscription::interval`, `vizia::view::timer`
  The decision prevents `pub struct X(pub Y);` cargo-culting — if
  you find yourself reaching for a wrapper crate, stop and check
  whether the stdlib version already does what you need.

- **Analysis module** (`logic_nih_plug_dsp::analysis`) — four new analysis tools:
  - `LevelMeter` — peak / RMS metering with configurable attack/release ballistics,
    per-channel and global tracking, dB domain queries.
  - `Follower` — single-path envelope follower with absolute / squared rectification,
    attack/release smoothing.
  - `LoudnessMeter` — ITU-R BS.1770 K-weighting (high-shelf pre-filter + 38 Hz
    Butterworth high-pass), momentary (400 ms), short-term (3 s), integrated LUFS
    with absolute gating.
  - `Oscilloscope` — circular min/max block buffer for waveform display,
    chronological iteration, age-based access.
  - 33 unit tests + 9 doc-tests; gated behind `analysis` feature flag.

- **Pitch processing** (`logic_nih_plug_dsp::processors::pitch`) — phase
  vocoder, pitch shifting, and time stretching:
  - `PhaseVocoder` — STFT analysis → phase unwrapping → overlap-add
    synthesis with Hann windows, configurable FFT size and hop divisor.
  - `PitchShift` — pitch shifting without changing duration, via phase
    vocoder + linear resampling. Supports any fractional ratio.
  - `TimeStretching` — time stretching / compressing without pitch change.
  - `WindowingFunction` — 8 window types (rectangular, triangular, Hann,
    Hamming, Blackman, Blackman-Harris, FlatTop, Kaiser).
  - 29 unit tests + 2 doc-tests; gated behind `pitch` feature flag.

- **Resampling** (`logic_nih_plug_dsp::processors::resampling`) — sample-rate
  conversion and fractional-delay interpolation:
  - `GenericInterpolator<T>` — circular buffer wrapper that delegates to any
    [`Interpolator`] strategy.
  - `ZeroOrderHold` — lo-fi staircase (latency 0, memory 1).
  - `Linear` — low-cost linear blend (latency 1, memory 2).
  - `CatmullRom` — cubic spline (latency 2, memory 4).
  - `Lagrange` — 4th-order polynomial (latency 2, memory 5).
  - `WindowedSinc` — Hann-windowed sinc with precomputed 1000-entry lookup
    table (latency 100, memory 200).
  - 14 unit tests + 2 doc-tests; gated behind `resampling` feature flag.

- Mixer helpers ported from `juce::dsp` are now available in
  `logic_nih_plug_dsp::processors` (gated behind the new `mixer`
  feature flag, enabled by the existing `processors` umbrella feature):
  - `Panner` — stereo panner with 7 pan laws (linear, balanced,
    sin3dB, sin4p5dB, sin6dB, squareRoot3dB, squareRoot4p5dB).
    Smoothed L/R volumes, in-place stereo processing.
    Port of JUCE's `juce::dsp::Panner`.
  - `DryWetMixer` — dry/wet mixer with 7 mixing rules matching
    the panner laws. Ring-buffer dry storage with push/mix API.
    Port of JUCE's `juce::dsp::DryWetMixer` (simplified single-
    threaded API without AbstractFifo complexity).
  - `Gain` — already existed; no changes needed.
  - 25 unit tests + 6 doc-tests.

### Added

- Modulation processors ported from `juce::dsp` are now available in
  `logic_nih_plug_dsp::processors` (gated behind the new `modulation`
  feature flag, enabled by the existing `processors` umbrella feature):
  - `Phaser` — 6-stage LFO-modulated first-order allpass phaser.
    Sine LFO subsampled by factor 4, per-channel feedback, dry/wet
    mix. Allpass stages use the TPT (Topology-Preserving Transform)
    formulation from JUCE's `FirstOrderTPTFilter`.
  - `Chorus` — LFO-modulated delay line chorus / flanger / vibrato.
    Reuses `DelayLine<LinearInterpolation>` from the `delay` module.
    Classic chorus (7–8 ms centre delay), flanging (short delay,
    high feedback), and vibrato (mix = 1.0) modes.
  - `WahWah` — LFO-modulated resonant bandpass filter (auto-wah).
    TPT state variable filter bandpass, log-mapped centre frequency
    sweep between configurable min/max. Not a JUCE DSP widget but
    follows the same API conventions.
  - `LadderFilter` — Moog ladder filter with 6 modes (`LPF12`,
    `HPF12`, `BPF12`, `LPF24`, `HPF24`, `BPF24`), tanh saturation
    LUT, smoothed cutoff / resonance, drive parameter. Follows
    Valimaki (2006) as implemented in JUCE's `LadderFilter`.
  - 30 unit tests + 4 doc-tests.

### Added

- Delay effect ported from `juce::dsp::DelayLine` is now available in
  `logic_nih_plug_dsp::processors::delay` (gated behind the new
  `delay` feature flag, enabled by the existing `processors` umbrella
  feature):
  - `DelayLine<Interp>` — multi-channel, fractional-sample delay line
    with pluggable interpolation. Direct port of
    `juce::dsp::DelayLine<SampleType, InterpolationType>`. Uses an
    inverted ring buffer (write_pos / read_pos decrement mod
    `total_size`) so per-sample push/pop is O(1) without branches.
    `total_size = max(4, max_delay + 2)` matches JUCE exactly.
  - `DelayLineInterpolation` trait with four implementations matching
    the JUCE `DelayLineInterpolationTypes` line-for-line:
    `NoInterpolation` (integer read), `LinearInterpolation`
    (linear blend), `Lagrange3rdInterpolation` (4-tap, frac<2 → +1
    canonical shift), and `ThiranInterpolation` (1st-order allpass,
    frac<0.618 → +1 canonical shift, pre-computed `alpha`). The
    canonical-shift logic lives in a `canonicalize` hook on the trait
    so each interpolator controls its own kernel conditioning.
  - `set_delay(samples)` clamps to `[0, total_size - 2]`, then
    canonicalises through the trait. `push_sample`, `pop_sample`,
    `pop_sample_with_delay(channel, delay, update_read_pointer)`
    (the last form enables multi-tap reads without advancing the
    pointer, exactly like JUCE). `process(input, output)` block form.
  - `DefaultDelayLine = DelayLine<LinearInterpolation>` type alias
    for the common case.
  - `Delay` effect — feedback delay with `DelayParameters { ... }`
    (`delay_time_seconds`, `feedback` ∈ [0, 1.2], `mix` ∈ [0, 1],
    `ping_pong`, `tempo_sync`, `tempo_bpm`, `note_division`,
    `max_delay_seconds`, `enabled`). The delay-line input is
    hard-clipped to ±1.0 to prevent NaN/Inf at high feedback.
  - `process_sample` (mono, uses only the left line), `process`
    (mono block, `Processor` trait), and `process_stereo(in_l, in_r,
    out_l, out_r)` (stereo, with cross-feedback when `ping_pong` is
    on). `set_tempo_bpm` updates the target without re-allocating.
  - 12 `NoteDivision` values (whole, half, quarter, eighth, sixteenth,
    thirty-second + dotted-half/quarter/eighth + triplet-half/quarter/
    eighth) with a `delay_samples(bpm, sample_rate)` helper.
  - 10 ms linear parameter smoothing for `mix`, `feedback`, and
    `delay_time` via an internal `SmoothedValue` helper. The smoother
    **snaps** on the first `set_target_value` (no fade-in from 0) and
    ramps on subsequent changes, so the very first parameter setup
    uses the target value immediately while later UI changes stay
    click-free.
  - 25 new unit tests + 1 doc-test cover NoteDivision math, ring-buffer
    push/pop semantics, all four interpolators (integer, linear blend,
    multi-tap, Thiran impulse-through), tempo sync, ping-pong cross
    feedback, feedback-driven echo count, and `prepare` /
    `prepare_spec` round-trip. `cargo test --features "full" --lib`
    stays clean; clippy is silent on the new code (3 pre-existing
    warnings in `fir.rs` / `bias.rs` only).

- Algorithmic reverb ported from `juce::Reverb` and `juce::dsp::Reverb` is
  now available in `logic_nih_plug_dsp::processors::reverb` (gated behind
  the new `reverb` feature flag, enabled by the existing `processors`
  umbrella feature):
  - `Reverb` — FreeVerb-style stereo algorithmic reverb with
    Schroeder/Moorer-Filter topology: 8 parallel low-pass-feedback comb
    filters per channel summed into 4 series allpass filters per
    channel. Stereo decorrelation via a 23-sample offset on the right
    channel tunings.
  - `Reverb::Parameters` mirrors `juce::Reverb::Parameters`
    (roomSize, damping, wetLevel, dryLevel, width, freezeMode) with
    matching defaults (0.5, 0.5, 0.33, 0.4, 1.0, 0.0). Parameter
    changes are ramped over 10 ms via a small `SmoothedValue` helper.
  - `process_sample` (mono), `process` (mono block) and
    `process_stereo` (left/right, matches JUCE's `processStereo`)
    entry points. `set_enabled` bypasses the wet path for A/B.
  - 12 unit tests + 1 doc-test covering tunings, sample-rate scaling,
    damping/feedback mapping, freeze mode, enable/disable bypass,
    block vs. per-sample equivalence, and impulse-decay shape.
  - Internal: `CombFilter` and `AllPassFilter` (Schroeder allpass with
    0.5 feedback coefficient) matching `juce::Reverb`'s `CombFilter` /
    `AllPassFilter` line-for-line.

- Dynamics processors ported from `juce::dsp` are now available in
  `logic_nih_plug_dsp::processors` (gated behind the new `dynamics` feature
  flag, enabled by the existing `processors` umbrella feature):
  - `BallisticsFilter` — peak-rectifying / RMS attack-release envelope
    follower (`juce::dsp::BallisticsFilter`). `LevelCalculationType::Peak`
    is the default; `LevelCalculationType::Rms` squares before smoothing
    and square-roots afterwards, matching JUCE exactly.
  - `Compressor` — feed-forward compressor with threshold / ratio /
    attack / release controls (`juce::dsp::Compressor`). Per-channel
    state via `prepare_with_channels`; `set_ratio` panics below 1.0.
  - `NoiseGate` — RMS-driven downward expander / gate
    (`juce::dsp::NoiseGate`). Mirrors JUCE's two-filter topology
    (RMS pre-filter at 0 ms / 50 ms, then a smoothed envelope
    controlling the VCA).
  - `Limiter` — two-stage brick-wall limiter
    (`juce::dsp::Limiter`). First stage is 4:1 @ -10 dB with 2 ms / 200 ms
    ballistics; second stage is 1000:1 @ the user threshold with
    1 µs attack. A smoothed output volume compensates for the first-stage
    gain reduction and the output is hard-clipped to ±1.0.
  - `LookaheadLimiter` — wraps a `Limiter` with a per-channel delay
    line (default 5 ms) so the listener never hears an overshoot on a
    step transient.
  - Shared `ProcessSpec` (`sample_rate`, `num_channels`,
    `maximum_block_size`) and `db_to_linear` / `linear_to_db` helpers
    in `processors::dynamics`.
  - 35 new unit tests + 9 new doc-tests pass with `--features processors`;
    default-features lib build stays clean and clippy `-D warnings` is
    silent on every new module.

- New `logic_nih_plug_midi_ci` crate: pure-Rust port of `juce_midi_ci`
  (MIDI 2.0 Capability Inquiry). Provides 32 message body types (Discovery,
  Profile configuration, Property exchange, Process inquiry) and a
  transport-agnostic `Device` struct that parses incoming UMP payloads via
  `process_message` and dispatches them through a `DeviceListener` callback
  trait. Outbound messages are produced by the `Device` and surfaced through
  the consumer's `MessageSink` implementation. Core types include `Muid`
  (28-bit), `ChannelAddress`, `Profile` (5-byte ID), `DeviceInfo`,
  `CapabilityFlags`, `Category` (the wire status byte index), `Encoding`,
  `SubscriptionCommand`, and `ProtocolVersion`. Feature flags: `discovery`
  (default), `profiles` (default), `property-exchange` (default), `full` =
  all three. 47 unit tests + 1 doctest passing; clippy `-D warnings` clean
  on every feature combination.

- New `logic_nih_plug_osc` crate: pure-Rust port of `juce_osc`.
  Provides `OscSender` (synchronous UDP sender), `OscReceiver` (thread-driven
  UDP receiver with typed message listeners and pattern-matched dispatch),
  `OSCArgument` (sum type covering every OSC 1.0 argument type), `OSCMessage`,
  `OSCBundle`, and `OSCTimeTag` (with `immediate()` for "as soon as possible").
  Wire-format encoding/decoding is delegated to the `rosc` crate. Feature
  flags: `sender` (default), `receiver` (default), `full` = both. 27 unit
  tests + 5 doctests passing; clippy clean on every feature combination.

## [2025-02-23]

### Added

- New `logic_nih_plug_data` crate: pure-Rust port of `juce_data_structures`.
  Provides `ValueTree` (hierarchical observable state), `UndoManager`
  (transactional undo/redo), and `CachedValue<T>` (typed property binding).
  Feature flags: `valuetree` (default), `undo` (default), `full`.
- New `logic_nih_plug_crypto` crate: pure-Rust port of `juce_crypto`.
  Provides streaming SHA-256 / SHA-1 / MD5 hashing (with one-shot helpers
  and hex encoders), a `BigInteger` arbitrary-precision unsigned integer
  type, and `RSAKey` for key generation, signing and verification
  (SHA-256 + PKCS#1 v1.5). Feature flags: `sha2` (default), `sha1`,
  `md5`, `bignum`, `rsa`, `full`.

### Breaking changes

- `logic_nih_plug_egui` now uses egui 0.31.

### Added

- `logic_nih_plug_egui` has a new `ResizableWindow` widget that can be used to resize
  the plugin's editor.

### Changed

- The CLAP bindings were updated to 1.2.2. The only noticeable difference is
  that the remote controls exposed through `ClapPlugin::remote_controls()` now
  use the non-draft extension.

### Fixed

- Fixed a warning about future name clashes when compiling `logic_nih_plug`.

## [2024-12-23]

### Added

- `logic_nih_plug_vizia`'s `ParamSlider` has a new style that always shows the offset
  relative to the center of the slider.

## [2024-08-18]

### Breaking changes

- The minimum supported Rust version has been bumped to 1.80 to replace the last
  uses of `lazy_static` with `std::sync::LazyLock`.

## [2024-05-05]

### Breaking changes

- `logic_nih_plug_egui` has been updated from egui 0.26.1 to egui 0.27.2.
- `logic_nih_plug_vizia` has been updated to the latest version with some a additional
  patches. This includes a workaround for the problem where opening multiple
  instances of a plugin's GUI on Windows or macOS would result in crashes.

### Changed

- Two byte slices are now accepted in `NoteEvent::from_midi()` if the event
  doesn't use the third byte.

### Fixed

- Fixed a race condition in the VST3 GUI event loop on Linux. This could cause
  panics with certain versions of Carla.
- The CPAL backend now correctly handles situations where it receives fewer
  samples than configured.
- Fixed the handling of multichannel audio in the CPAL backend.

## [2024-05-04]

### Fixed

- Fixed a soundness issue in the buffer management where in-place input/output
  buffers may not have been recognized properly before.

## [2024-03-23]

### Added

- `logic_nih_plug_xtask` now detects and uses non-standard `target` directory
  locations if overridden through Cargo's settings.

## [2024-03-18]

### Changed

- (Keyboard) input events sent by the host through VST3's `IPlugView` interface
  are now explicitly ignored. This may allow a couple more keyboard events to
  reach through to plugin windows in hosts that use these.

## [2024-02-23]

### Fixed

- Fixed `logic_nih_plug_egui` panicking due to cursor icons not yet being implemented in baseview for MacOS and Windows.

## [2024-02-22]

### Breaking changes

- `logic_nih_plug_egui` has been updated from egui 0.22.0 to using egui 0.26.1.

## [2023-12-30]

### Breaking changes

- `logic_nih_plug_vizia` has been updated to the latest Vizia version. Vizia's styling
  system has changed a lot since the last update, so plugin GUIs and stylesheets
  may require small changes before they behave the same again. A summary of the
  most important changes can be found in Vizia PR
  [#291](https://github.com/vizia/vizia/pull/291). Some notable breaking changes
  include:

  - Font handling and choosing between different variations of the same font
    (e.g. `Noto Sans` versus `Noto Sans Light` versus `Noto Sans Light Italic`)
    works very differently now.
  - `ResizeHandle` now needs to be the last element in a GUI because of changes
    to Vizia's event targetting mechanism.

- The `raw_window_handle` version used by NIH-plug has been updated to version
  0.5.x.

### Added

- Added initial RISC-V support to `logic_nih_plug_xtask`.
  ([#95](https://github.com/robbert-vdh/nih-plug/pull/95)).

### Changed

- `ParentWindowHandle` has changed to be a sum type of different parent window
  handle types, similar to `RawWindowHandle`. This makes it easier to use GUI
  libraries that link against a different version of `raw_window_handle` than
  the one used by NIH-plug itself by simply wrapping around
  `ParentWindowHandle`.
- `nih_debug_assert*!()` failures are now promoted to a warning instead of a
  debug message. This makes the non-fatal debug assertion failures easier to
  spot.
- The minimum scale factor in `logic_nih_plug_vizia` has changed from 0.25 to 0.5.
  Vizia rounds things to single pixels, and below 0.5 scaling single pixel
  borders would disappear when not using a HiDPI setup.

### Fixed

- Various `baseview` dependencies now have their versions pinned.

## [2023-12-06]

### Fixed

- `nih_export_vst3!()` no longer requires `nih_debug_assert` to be in scope.

## [2023-11-05]

### Changed

- `FloatParam` and `IntParam` ranges can now be accessed using methods on the
  parameters ([#89](https://github.com/robbert-vdh/nih-plug/pull/89)).

## [2023-09-21]

### Fixed

- Fixed null pointers assertions in the low level buffer management code not
  working correctly.

## [2023-09-03]

### Added

- `nih_export_vst3!()` now also supports more than one plugin type argument,
  just like `nih_export_clap!()`.

### Fixed

- The `nih_export_*!()` macros now use `$crate` to refer to NIH-plug itself,
  which makes it possible to use the NIH-plug crate under a different name.

## [2023-08-05]

### Breaking changes

- The minimum supported Rust version has been bumped to 1.70 so we can start
  using `OnceCell` and `OnceLock` to phase out uses of `lazy_static`.

### Added

- `nih_export_clap!()` can now take more than one plugin type argument to allow
  exporting more than one plugin from a single plugin library.

## [2023-05-13]

### Fixed

- Removed the `Default` bound from the `SysExMessage::Buffer` type. This was a
  leftover from an older design.

## [2023-04-30]

### Changed

- Added debug assertions to make sure parameter ranges are valid. The minimum
  value must always be lower than the maximum value and they cannot be equal.

## [2023-04-27]

### Changed

- The `v2s_f32_rounded()` formatter now avoids returning negative zero values
  for roundtripping reasons since -0.0 and 0.0 correspond to the same normalized
  value.

## [2023-04-24]

### Breaking changes

- `Plugin::editor()` and `Plugin::task_executor()` now take `&mut self` instead
  of `&self` to make it easier to move data into these functions without
  involving interior mutability.

### Changed

- The `Plugin` trait's documentation has been updated to better clarify the
  structure and to more explicitly mention that the non-lifecycle methods are
  called once immediately after creating the plugin object.

### Fixed

- The logger now uses the correct local time offset on Linux instead of
  defaulting to UTC due to some implementation details of the underlying `time`
  crate.
- The buffer changes from March 31st broke the sample accurate automation
  feature. This has now been fixed.

## [2023-04-22]

### Added

- CLAP plugins can optionally declare pages of [remote
  controls](https://github.com/free-audio/clap/blob/main/include/clap/ext/draft/remote-controls.h)
  so DAWs can more automatically map pages of the plugin's parameters to
  hardware controllers. This is currently a draft extension, so until the
  extension is finalized host support may break at any moment.

### Changed

- The CLAP version has been updated to 1.1.8.
- The prelude module now also re-exports the following:
  - The `PluginApi` num.
  - The `Transport` struct.

### Fixed

- The upgrade to CLAP 1.1.8 caused NIH-plug to switch from the draft version of
  the voice info extension to the final version, fixing voice stacking with
  recent versions of Bitwig.

## [2023-04-05]

### Breaking changes

- The `nih_debug_assert*!()` macros are now upgraded to regular panicking
  `debug_assert!()` macros during tests.
- `SmoothingStyle::for_oversampling_factor()` has been removed in favor of a new
  mechanism that allows the smmoothers to be aware of oversampling. A new
  `Smoothingstyle::OversamplingAware(oversampling_times, style)` can be used to
  wrap another `Smoothingstyle` to make it aware of an oversampling amount that
  can change at runtime. The `oversampling_times` is an `Arc<AtomicF32>` that
  indicates the current oversampling amount. This makes it possible to link
  multiple parameters to the same oversampling amount, have different sets of
  parameters run at different effective sample rates, and automatically update
  those oversampling amounts/sample rate multipliers from a parameter callback.
- As a consequence of the above change, `Smoothingstyle` is no longer `Copy`
  since the `OversamplingAware` smoothing style contain an
  `Arc<Smoothingstyle>`. It can still be `Clone`d.

### Changed

- The prelude module now also re-exports the `AtomicF32` type since it's needed
  to use the new `Smoothingstyle::OversamplingAware`.

## [2023-04-01]

### Fixed

- Auxiliary output buffers are now always zeroed out in case the host didn't do
  this for us. This was a regression from before 2023-03-31.

## [2023-03-31]

### Changed

- Buffer management has been completely rewritten so it can be shared among all
  of NIH-plug's backends. This should not result in any noticeable changes, but
  it should reduce the chances of backend-specific bugs when it comes to
  interacting with audio buffers and it will make it simpler to implement buffer
  management for new plugin APIs.

### Fixed

- When a main IO audio buffers has more output channels than input channels, the
  excess output channels are now correctly filled with zeroes instead of
  containing whatever data was left in the host's output buffers. As part of
  this change NIH-plug's buffer management has been refactored to reuse the same
  logic in all of its wrappers.
- Any outstanding VST3 output events are now sent to the host during a parameter
  flush.

## [2023-03-21]

### Changed

- The logger now always shows the module in debug builds to make it easier to
  know where logging messages are sent from. Previously this was only done for
  the debug and trace message levels.
- The logger now filters out the `Mapped XXXX font faces in YYYms.` messages
  from cosmic text in release builds as this is unnecessary noise for end users.
- `logic_nih_plug_vizia`: `ParamButton`'s active color was made much lighter to make
  the text more readable, and the hover state has been fixed.

## [2023-03-18]

### Added

- `logic_nih_plug_vizia`: Added a `GuiContextEvent::Resize` event. The plugin can emit
  this to trigger a resize to its current size, as specified by its
  `ViziaState`'s size callback. This can be used to declaratively resize a
  plugin GUI and it removes some potential surface for making mistakes in the
  process. See `GuiContextEvent::Resize`'s documentation for an example.

## [2023-03-17]

### Added

- Added a `NoteEvent::channel()` method to get an event's channel, if it has
  any. ([#62](https://github.com/robbert-vdh/nih-plug/pull/62))

## [2023-03-07]

This document is now also used to keep track of non-breaking changes.

### Breaking changes

- The way window sizes work in `ViziaState` has been reworked to be more
  predictable and reliable. Instead of creating a `ViziaState` with a predefined
  size and then tracking the window's current size in that object, `ViziaState`
  now takes a callback that returns the window's current logical size. This can
  be used to compute the window's current size based on the plugin's state. The
  result is that window sizes always match the plugin's current state and
  recalling an old incorrect size is no longer possible.

### Added

- Debug builds now include debug assertions that detect incorrect use of the
  `GuiContext`'s parameter setting methods.

## [2023-02-28]

### Breaking changes

- `ViziaState::from_size()` now takes a third boolean argument to control
  whether the window's size is persisted or not. This avoids a potential bug
  where an old window size is recalled after the plugin's GUI's size has changed
  in an update to the plugin.

## [2023-02-20]

### Breaking changes

- The way audio IO layouts are configured has changed completely to align better
  with NIH-plug's current and future supported plugin API backends. Rather than
  defining a default layout and allowing the host/backend to change the channel
  counts by polling the `Plugin::accepts_bus_config()` function, the plugin now
  explicitly enumerates all supported audio IO layouts in a declarative fashion.
  This change gives the plugin more options for defining alternative audio port
  layouts including layouts with variable numbers of channels and ports, while
  simultaneously removing ambiguities and behavior that was previously governed
  by heuristics.

  All types surrounding bus layouts and port names have changed slightly to
  accommodate this change. Take a look at the updated examples for more details
  on how this works. The `Plugin::AUDIO_IO_LAYOUTS` field's documentation also
  contains an example for how to initialize the layouts slice.

- As a result of the above change, NIH-plug's standalones no longer have
  `--input` and `--output` command line arguments to change the number of input
  and output channels. Instead, they now have an `--audio-layout` option that
  lets the user select an audio layout from the list of available layouts by
  index. `--audio-layout=help` can be used to list those layouts.

## [2023-02-01]

### Breaking changes

- The `Vst3Plugin::VST3_CATEGORIES` string constant has been replaced by a
  `Vst3Plugin::VST3_SUBCATEGORIES` constant of type `&[Vst3SubCategory]`.
  `Vst3SubCategory` is an enum containing all of VST3's predefined categories,
  and it behaves similarly to the `ClapFeature` enum used for CLAP plugins. This
  makes defining subcategories for VST3 plugins easier and less error prone.

## [2023-01-31]

### Breaking changes

- NIH-plug has gained support MIDI SysEx in a simple, type-safe, and
  realtime-safe way. This sadly does mean that every `Plugin` instance now needs
  to define a `SysExMessage` type definition and constructor function as Rust
  does not yet support defaults for associated types (Rust issue
  [#29661](https://github.com/rust-lang/rust/issues/29661)):

  ```rust
  type SysExMessage = ();
  ```

- As the result of the above change, `NoteEvent` is now parameterized by a
  `SysExMessage` type. There is a new `PluginNoteEvent<P>` type synonym that can
  be parameterized by a `Plugin` to make using this slightly less verbose.

## [2023-01-12]

### Breaking changes

- The Vizia dependency has been updated. This updated version uses a new text
  rendering engine, so there are a couple breaking changes:
  - The names for some of Vizia's fonts have changed. The constants and font
    registration functions in `logic_nih_plug_vizia::assets` and
    `logic_nih_plug_vizia::vizia_assets` still have the same name, but all uses of the
    `font` CSS property and `.font()` view modifier will have to be changed.
  - Metrics for rendered text have change slightly. Most notably the height and
    vertical positioning of text is slightly different, so you may have to
    adjust your layout slightly accordingly.

## [2023-01-11]

### Breaking changes

- `Editor::param_values_changes()` is no longer called from the audio thread and
  thus no longer needs to be realtime safe.
- A new `Editor::param_value_changed(id, normalized_value)` method has been
  added. This is used to notify the plugin of changes to individual parameters.
- A similar new `Editor::param_modulation_changed(id, modulation_offset)` is
  used to inform the plugin of a parameter's new monophonic modulation offset.

## [2023-01-06]

### Breaking changes

- The threads used for the `.schedule_gui()` and `.schedule_background()`
  methods are now shared between all instances of a plugin. This makes
  `.schedule_gui()` on Linux behave more like it does on Windows and macOS, and
  there is now only a single background thread instead of each instance spawning
  their own thread.

## [2023-01-05]

### Breaking changes

- `Buffer::len()` has been renamed to `Buffer::samples()` to make this less
  ambiguous.
- `Block::len()` has been renamed to `Block::samples()`.

## [2022-11-17]

### Breaking changes

- The `Params` derive macro now also properly supports persistent fields in
  `#[nested]` parameter structs. This takes `#[nested(id_prefix = "...")]` and
  `#[nested(array)]` into account to allow multiple copies of a persistent
  field. This may break existing usages as serialized field data without a
  matching preffix or suffix is no longer passed to the child object.

## [2022-11-17]

### Breaking changes

- The order of `#[nested]` parameters in the parameter list now always follows
  the declaration order instead of nested parameters being ordered below regular
  parameters.

## [2022-11-08]

### Breaking changes

- The `Param::{next_previous}{_step,_normalized_step}()` functions now take an
  additional boolean argument to indicate that the range must be finer. This is
  used for floating point parameters to chop the range up into smaller segments
  when using Shift+scroll.

## [2022-11-07]

### Breaking changes

- `Param::plain_value()` and `Param::normalized_value()` have been renamed to
  `Param::modulated_plain_value()` and `Param::modulated_normalized_value()`.
  These functions are only used when creating GUIs, so this shouldn't break any
  other plugin code. This change was made to make it extra clear that these
  values do include monophonic modulation, as it's very easy to mistakenly use
  the wrong value when handling user input in GUI widgets.

## [2022-11-06]

### Breaking changes

- `logic_nih_plug_vizia::create_vizia_editor_without_theme()` has been removed, and
  `logic_nih_plug_vizia::create_vizia_editor()` has gained a new argument to specify
  what amount of theming to apply. This can now also be used to completely
  disable all theming include Vizia's built-in theme.
- `logic_nih_plug_vizia::create_vizia_editor()` no longer registers any fonts by
  default. Even when those fonts are not used, they will still be embedded in
  the binary, increasing its size by several megabytes. Instead, you can now
  register individual fonts by calling the
  `logic_nih_plug_vizia::assets::register_*()` functions. This means that you _must_
  call `logic_nih_plug_vizia::assets::register_noto_sans_light()` for the default
  theming to work. All of the plugins in this repo also use
  `logic_nih_plug_vizia::assets::register_noto_sans_thin()` as a title font.
- Additionally, the Vizia fork has been updated to not register _any_ default
  fonts for the same reason. If you previously relied on Vizia's default Roboto
  font, then you must now call `logic_nih_plug_vizia::vizia_assets::register_roboto()`
  at the start of your process function.

## [2022-10-23]

### Breaking changes

- `logic_nih_plug_vizia` has been updated. Widgets with custom drawing code will need
  to be updated because of changes in Vizia itself.

## [2022-10-22]

### Breaking changes

- The `Editor` trait and the `ParentWindowHandle` struct have been moved from
  `logic_nih_plug::plugin` to a new `logic_nih_plug::editor` module. If you only use the
  prelude module then you won't need to change anything.
- The `logic_nih_plug::context` module has been split up into
  `logic_nih_plug::context::init`, `logic_nih_plug::context::process`, and
  `logic_nih_plug::context::gui` to make it clearer which structs go with which
  context. You again don't have to change anything if you use the prelude.
- NIH-plug has gained support for asynchronously running background tasks in a
  simple, type-safe, and realtime-safe way. This sadly does mean that every
  `Plugin` instance now needs to define a `BackgroundTask` type definition and
  constructor function as Rust does not yet support defaults for associated
  types (Rust issue [#29661](https://github.com/rust-lang/rust/issues/29661)):

  ```rust
  type BackgroundTask = ();
  ```

- The `&mut impl InitContext` argument to `Plugin::initialize()` needs to be
  changed to `&mut impl InitContext<Self>`.
- The `&mut impl ProcessContext` argument to `Plugin::process()` needs to be
  changed to `&mut impl ProcessContext<Self>`.
- The `Plugin::editor()` method now also takes a
  `_async_executor: AsyncExecutor<Self>` parameter.

## [2022-10-20]

### Breaking changes

- Some items have been moved out of `logic_nih_plug::param::internals`. The main
  `Params` trait is now located under `logic_nih_plug::param`, and the
  `PersistentTrait` trait, implementations, and helper functions are now part of
  a new `logic_nih_plug::param::persist` module. Code importing the `Params` trait
  through the prelude module doesn't need to be changed.
- The `logic_nih_plug::param` module has been renamed to `logic_nih_plug::params`. Code that
  only uses the prelude module doesn't need to be changed.
- The `create_egui_editor()` function from `logic_nih_plug_egui` now also takes a
  build closure to apply initialization logic to the egui context.
- `Editor` and the editor handle returned by `Editor::spawn` now only require
  `Send` and no longer need `Sync`. This is not a breaking change, but it might
  be worth being aware of.
- Similar to the above change, `Plugin` also no longer requires `Sync`.

## [2022-10-13]

### Breaking changes

- The `#[nested]` parameter attribute has gained super powers and has its syntax
  changed. It can now automatically handle many situations that previously
  required custom `Params` implementations to have multiple almost identical
  copies of a parameter struct. The current version supports both fields with
  unique parameter ID prefixes, and arrays of parameter objects. See the
  [`Params`](https://nih-plug.robbertvanderhelm.nl/logic_nih_plug/param/internals/trait.Params.html)
  trait for more information on the new syntax.

## [2022-09-22]

### Breaking changes

- `logic_nih_plug_vizia` has been updated. Custom widgets will need to be updated
  because of changes Vizia itself.
- `logic_nih_plug_egui` has been updated from egui 0.17 to egui 0.19.

## [2022-09-06]

### Breaking changes

- Parameter values are now accessed using `param.value()` instead of
  `param.value`, with `param.value()` being an alias for the existing
  `param.plain_value()` function. The old approach, while perfectly safe in
  practice, was technically unsound because it used mutable pointers to
  parameters that may also be simultaneously read from in an editor GUI. With
  this change the parameters now use actual relaxed atomic stores and loads to
  avoid mutable aliasing, which means the value fields are now no longer
  directly accessible.

## [2022-09-04]

### Breaking changes

- `Smoother::next_block_mapped()` and `Smoother::next_block_exact_mapped()` have
  been redesigned. They now take an index of the element being generated and the
  float representation of the smoothed value. This makes it easier to use them
  for modulation, and it makes it possible to smoothly modulate integers and
  other stepped parameters. Additionally, the mapping functions are now also
  called for every produced value, even if the smoother has already finished
  smoothing and is always producing the same value.

## [2022-08-19]

### Breaking changes

- Standalones now use the plugin's default input and output channel counts
  instead of always defaulting to two inputs and two outputs.
- `Plugin::DEFAULT_NUM_INPUTS` and `Plugin::DEFAULT_NUM_OUTPUTS` have been
  renamed to `Plugin::DEFAULT_INPUT_CHANNELS` and
  `Plugin::DEFAULT_OUTPUT_CHANNELS` respectively to avoid confusion as these
  constants only affect the main input and output.

## [2022-07-18]

### Breaking changes

- `IntRange` and `FloatRange` no longer have min/max methods and instead have
  next/previous step methods. This is for better compatibility with the new
  reversed ranges.

## [2022-07-06]

### Breaking changes

- There are new `NoteEvent::PolyModulation` and `NoteEvent::MonoAutomation`
  events as part of polyphonic modulation support for CLAP plugins.
- The block smoothing API has been reworked. Instead of `Smoother`s having their
  own built-in block buffer, you now need to provide your own mutable slice for
  the smoother to fill. This makes the API easier to understand, more flexible,
  and it allows cloning smoothers without worrying about allocations.In
  addition, the new implementation is much more efficient when the smoothing
  period has ended before or during the block.

## [2022-07-05]

### Breaking changes

- The `ClapPlugin::CLAP_HARD_REALTIME` constant was moved to the general
  `Plugin` trait as `Plugin::HARD_REALTIME_ONLY`, and best-effort support for
  VST3 has been added.

## [2022-07-04]

### Breaking changes

- The `CLAP_DESCRIPTION`, `CLAP_MANUAL_URL`, and `CLAP_SUPPORT_URL` associated
  constants from the `ClapPlugin` are now optional and have the type
  `Option<&'static str>` instead of `&'static str`.
- Most `NoteEvent` variants now have an additional `voice_id` field.
- There is a new `NoteEvent::VoiceTerminated` event a plugin can send to let the
  host know a voice has been terminated. This needs to be output by CLAP plugins
  that support polyphonic modulation.
- There is a new `NoteEvent::Choke` event the host can send to a plugin to let
  it know that it should immediately terminate all sound associated with a voice
  or a key.

## [2022-07-02]

### Breaking changes

- The `Params::serialize_fields()` and `Params::deserialize_fields()` methods
  and the `State` struct now use `BTreeMap`s instead of `HashMap`s so the order
  is consistent the plugin's state to JSON multiple times. These things are part
  of NIH-plug's internals, so unless you're implementing the `Params` trait by
  hand you will not notice any changes.

## [2022-06-01]

### Breaking changes

- The `ClapPlugin::CLAP_FEATURES` field now uses an array of `ClapFeature`
  values instead of `&'static str`s. CLAP 0.26 contains many new predefined
  features, and the existing ones now use dashes instead of underscores. Custom
  features are still possible using `ClapFeature::Custom`.

## [2022-05-27]

### Breaking changes

- `Plugin::process()` now takes a new `aux: &mut AuxiliaryBuffers` parameter.
  This was needed to allow auxiliary (sidechain) inputs and outputs.
- The `Plugin::initialize()` method now takes a `&mut impl InitContext` instead
  of a `&mut impl ProcessContext`.

## [2022-05-22]

### Breaking changes

- The current processing mode is now stored in `BufferConfig`. Previously this
  could be fetched through a function on the `ProcessContext`, but this makes
  more sense as it remains constant until a plugin is deactivated. The
  `BufferConfig` now contains a field for the minimum buffer size that may or
  may not be set depending on the plugin API.
- Previously calling `param.non_automatable()` when constructing a parameter
  also made the parameter hidden. Hiding a parameter is now done through
  `param.hide()`, while `param.non_automatable()` simply makes it so that the
  parameter can only be changed manually and not through automation or
  modulation.

## ...

Who knows what happened at this point!
