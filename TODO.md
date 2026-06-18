# TODO — JUCE Feature Parity for `logic_nih_plug`

This file tracks everything needed to bring the pure-Rust port to full JUCE
feature parity. Tasks are ordered by dependency and priority; work top-down
unless you have a specific reason to skip.

> **Already done** is summarized in [README.md](README.md) and
> [AGENTS.md](AGENTS.md). This file is the *backlog*.

---

## 1. Missing crates referenced in docs (highest priority)

These are mentioned in the README and example READMEs but the workspace
member does not exist. They block `plugins/examples/juce_multi_module` from
compiling.

- [x] **Port `juce_data_structures` → `logic_nih_plug_data`** — ✅ done (2025-06-17)
  - `ValueTree` (hierarchical, typed-property, serializable)
  - `UndoManager` (transactional undo/redo for `ValueTree`)
  - `CachedValue<T>` (auto-binding between `ValueTree` and a typed field)
  - `Value` / `ValueWithDefault` (JUCE `var`-style)
  - Properties: int, double, string, bool, array, binary, null
  - Default features: `valuetree`, `undo`; `full` = all

- [x] **Port `juce_crypto` → `logic_nih_plug_crypto`** — ✅ done (2026-06-17)
  - `Sha256` / `Sha1` / `Md5` streaming hash contexts (one-shot helpers,
    hex-encoded output, NIST test vectors verified)
  - `BigInteger` — parsing in any radix 2..=36, hex/decimal/byte-string
    conversions, bit-level access, modular exponentiation, GCD
  - `RSAKey` — 2048+ bit key generation, raw-component import
    (`from_public_components`, `from_private_components`),
    SHA-256 + PKCS#1 v1.5 sign/verify, debug-redacting `Debug` impl
  - 45 unit tests + 4 doc-tests passing under `--features full`; default
    (sha2-only) is a clean 9 + 2
  - Features: `sha2` (default), `sha1`, `md5`, `bignum`, `rsa`, `full`

- [x] **Port `juce_osc` → `logic_nih_plug_osc`** — ✅ done (2026-06-18)
  - `OscSender` (UDP, `rosc` crate)
  - `OscReceiver` with pattern matching (`/foo/bar i=42 f=3.14 s="x"`)
  - `OSCArgument` sum type
  - `OSCBundle` (time-tagged)
  - Default features: `sender`, `receiver`; `full` = both

- [x] **Port `juce_midi_ci` → `logic_nih_plug_midi_ci`** — ✅ done (2026-06-18)
  - MIDI-CI discovery (PE, PI, PE stream)
  - Profile configuration (MIDI 1.0 / MIDI 2.0)
  - Property exchange (per-channel `ValueTree`-like)
  - Default features: `discovery`, `profiles`, `property-exchange`; `full` = all
  - 32 message body types (Discovery, Profile, Property Exchange, Process Inquiry)
  - Transport-agnostic: consumes raw UMP bytes, hands outbound messages to a
    `MessageSink` trait the consumer implements

---

## 2. DSP module — core effects/analysis (JUCE `dsp` module)

The `logic_nih_plug_dsp` crate has filters/oscillators/convolution/envelopes
already. These are the missing processors.

- [x] **Dynamics** — `Compressor`, `Limiter`, `NoiseGate`, `Limiter` (lookahead) — ✅ done (2026-06-18)
  - `BallisticsFilter` (peak / RMS) — JUCE `dsp::BallisticsFilter`
  - `Compressor` — standard feed-forward compressor with threshold / ratio / attack / release
  - `NoiseGate` — RMS-driven downward expander / gate (used as expander at low ratios)
  - `Limiter` — two-stage (4:1 @ -10 dB → 1000:1 @ threshold) brick-wall limiter + smoothed output volume + hard clip
  - `LookaheadLimiter` — Limiter fronted by a delay line so output never overshoots on a step
  - 35 unit tests + 9 doc-tests passing under `--features processors`
  - Default features include `dynamics`; gated behind the `dynamics` / `processors` feature flags
- [x] **Reverb** (algorithmic) — Schroeder/Moorer-Filter (FreeVerb-style) topology, stereo decorrelation — ✅ done (2026-06-18)
  - 8 parallel low-pass-feedback comb filters per channel → 4 series allpass filters per channel
  - Stereo decorrelation via `STEREO_SPREAD=23`-sample offset on the right channel
  - `Parameters` struct mirrors `juce::Reverb::Parameters` (roomSize, damping, wetLevel, dryLevel, width, freezeMode)
  - 10 ms linear parameter smoothing via a small `SmoothedValue` helper for wet/dry/damping/feedback
  - `prepare(sample_rate, max_block_size)` reallocates comb/allpass buffers; `set_sample_rate` rescales the canonical FreeVerb tunings
  - `process_sample`, `process` (mono block), `process_stereo` (matches JUCE's `processStereo`)
  - 12 unit tests + 1 doc-test passing under `--features reverb`
  - Default features include `reverb`; gated behind the `reverb` / `processors` feature flags
- [x] **Delay** (feedback) — tempo-synced, ping-pong — ✅ done (2026-06-18)
  - `DelayLine<Interp>` primitive ported from `juce::dsp::DelayLine` with
    pluggable interpolation: `NoInterpolation`, `LinearInterpolation`,
    `Lagrange3rdInterpolation`, `ThiranInterpolation` (stateful allpass)
  - Inverted ring buffer (write_pos / read_pos decrement mod total_size)
    matching JUCE line-for-line; `total_size = max(4, max_delay + 2)`
  - `set_delay(samples)` clamps to `[0, total_size - 2]`, applies the
    canonical shift for `Lagrange3rd` (frac < 2 → +1) and `Thiran`
    (frac < 0.618 → +1) and pre-computes the Thiran alpha
    coefficient via the trait
  - `push_sample`, `pop_sample`, `pop_sample_with_delay(channel,
    delay, update_read_pointer)` (multi-tap), `process(input, output)`
    block form
  - `Delay` effect: feedback (0..1.2, hard-clipped to ±1.0),
    dry/wet mix, ping-pong (cross feedback), 10 ms linear smoothing
    on `mix`/`feedback`/`delay_time` (snaps on first set, ramps after)
  - 12 `NoteDivision` values: whole, half, quarter, eighth, sixteenth,
    thirty-second + dotted (half/quarter/eighth) + triplet
    (half/quarter/eighth), `delay_samples(bpm, sample_rate)` helper
  - `prepare(sample_rate, max_block_size)` allocates the per-line
    ring buffer to `max_delay_seconds * sample_rate`; `set_tempo_bpm`
    updates the target without re-allocating
  - 25 unit tests + 1 doc-test passing under `--features delay`
  - Default features include `delay`; gated behind the `delay` /
    `processors` feature flags
- [x] **Modulation** — `Phaser`, `Chorus`, `WahWah`, `LadderFilter` — ✅ done (2026-06-18)
  - `Phaser` — 6-stage TPT allpass phaser with subsampled sine LFO,
    per-channel feedback, dry/wet mix
  - `Chorus` — LFO-modulated `DelayLine<LinearInterpolation>` chorus /
    flanger / vibrato with feedback, dry/wet mix
  - `WahWah` — LFO-modulated TPT bandpass auto-wah with log-mapped
    centre frequency sweep
  - `LadderFilter` — Moog ladder with 6 modes (LPF/HPF/BPF × 12/24),
    tanh saturation LUT, smoothed cutoff/resonance, drive
  - 30 unit tests + 4 doc-tests passing under `--features modulation`
  - Gated behind the `modulation` / `processors` feature flags
- [x] **Mixer helpers** — `Panner` (equal-power, linear, square-root), `DryWet`,
  real `Gain` processor — ✅ done (2026-06-18)
  - `Panner` — 7 pan laws (Linear, Balanced, Sin3dB, Sin4p5dB, Sin6dB,
    SquareRoot3dB, SquareRoot4p5dB), smoothed L/R volumes,
    stereo in-place processing
  - `DryWetMixer` — 7 mixing rules, ring-buffer dry storage,
    push/mix API with per-sample smoothing
  - `Gain` — already complete (dB/linear, smoothing, `Processor` trait)
  - 25 unit tests + 6 doc-tests passing under `--features mixer`
  - Gated behind the `mixer` / `processors` feature flags
- [x] **Analysis** — `LoudnessMeter` (ITU-R BS.1770 K-weighting + momentary /
  short-term / integrated LUFS), `Oscilloscope`, `Follower`, `LevelMeter`
  - `LevelMeter` — peak / RMS metering with configurable attack/release ballistics,
    per-channel and global tracking, dB domain queries
  - `Follower` — single-path envelope follower with absolute / squared rectification,
    attack/release smoothing
  - `LoudnessMeter` — ITU-R BS.1770 K-weighting (high-shelf pre-filter + 38 Hz
    Butterworth high-pass), momentary (400 ms), short-term (3 s), integrated LUFS
    with absolute gating
  - `Oscilloscope` — circular min/max block buffer for waveform display,
    chronological iteration, age-based access
  - 33 unit tests + 9 doc-tests passing under `--features analysis`
  - Gated behind the `analysis` feature flag
- [x] **Pitch processing** — ✅ done (2026-06-18)
  - `PhaseVocoder` — STFT analysis → phase unwrapping → overlap-add synthesis
    with Hann windows, configurable FFT size and hop divisor
  - `PitchShift` — pitch shifting (±24 semitones or any fractional ratio)
    without changing duration, via phase vocoder + linear resampling
  - `TimeStretching` — time stretching / compressing without pitch change
  - `WindowingFunction` — 8 window types (rectangular, triangular, Hann,
    Hamming, Blackman, Blackman-Harris, FlatTop, Kaiser)
  - Proper OLA normalization with dynamic COLA gain computation
  - 29 unit tests + 2 doc-tests passing under `--features pitch`
  - Gated behind the `pitch` / `processors` feature flags
- [x] **Resampling** — ✅ done (2026-06-18)
  - `GenericInterpolator<T>` — circular buffer wrapper with `process()`, `process_adding()`, `process_advanced()`
  - `ZeroOrderHold` (latency 0, memory 1) — lo-fi staircase
  - `Linear` (latency 1, memory 2) — low-cost linear blend
  - `CatmullRom` (latency 2, memory 4) — cubic spline
  - `Lagrange` (latency 2, memory 5) — 4th-order polynomial
  - `WindowedSinc` (latency 100, memory 200) — Hann-windowed sinc with precomputed 1000-entry lookup table
  - `Interpolator` trait — `value_at_offset(&self, inputs, offset, write_pos)`
  - 14 unit tests + 2 doc-tests; gated behind `resampling` feature flag
- [x] **FFT upgrades** — ✅ done (2026-06-18)
  - `RealFFT` — real-only FFT path: real input → one-sided complex
    spectrum (N/2+1 bins). Uses `rustfft::FftPlanner` plans (forward
    + inverse) and discards the conjugate-symmetric half
  - `STFT` — short-time Fourier transform with per-frame
    `analyze_frame` / `synthesize_frame` and one-shot
    `process_block` (windowed OLA), with proper COLA normalisation
  - `WindowingFunction` — moved from `pitch` module to `analysis`
    module for cross-cutting access; 8 window types
    (Rectangular, Triangular, Hann, Hamming, Blackman,
    BlackmanHarris, FlatTop, Kaiser(β))
  - 17 unit tests + 4 doc-tests for `RealFFT` and `STFT`; 9
    windowing tests moved to `analysis`; gated behind `analysis`
    feature flag
- [x] **Cross-crate FFT work** — ✅ done (2026-06-18)
  - The `analysis::{FFT, RealFFT, STFT, WindowingFunction}` types are
    already re-exported at `logic_nih_plug_dsp::analysis::*` (plugin
    authors use them via the normal `use logic_nih_plug_dsp::...`
    path; the `logic_nih_plug` crate's `prelude` is reserved for
    host/plugin-integration types, not DSP).
  - `PhaseVocoder` now uses `RealFFT` internally instead of two
    separate `Arc<dyn rustfft::Fft<f32>>` plans. This halves the
    spectrum buffer (`fft_size/2 + 1` bins instead of `fft_size`),
    drops the manual mirror-after-phase-processing step, and makes
    the FFT path a single source of truth across `analysis` and
    `pitch`. All 20 pitch tests still pass.
  - `processors::pitch::WindowingFunction` continues to re-export
    from `analysis` for backwards compatibility.

---

## 3. Audio data layer

- [x] **Port `juce_audio_basics`** — ✅ done (2026-06-18)
  - `AudioSampleBuffer` (interleaved + non-interleaved) — non-interleaved
    storage (one `Vec<f32>` per channel), `interleave` / `deinterleave`
    helpers, `apply_gain`, `clear`, `copy_from`, `add_from`, `set_size`,
    `apply_channel_gains`
  - `AudioChannelSet` — `Mono`, `Stereo`, `Lrc`, `Lrs`, `Quadraphonic`,
    `FiveDotZero`, `FiveDotOne`, `SixDotOne`, `SevenDotZero`,
    `SevenDotOne`, `SevenDotOnePointTwo`, `SevenDotOnePointFour`,
    `Ambisonic(order)` (with `AmbisonicOrder::new(order)` validated
    against `MAX_AMBISONIC_ORDER = 7`), `Custom(n)`; channel names
    include abbreviation / description / speaker position (azimuth,
    elevation, distance)
  - `MidiMessage` — value type with `Vec<u8>` payload + `i32` time
    stamp. Builder methods: `note_on`, `note_off`, `note_off_zero_velocity`,
    `polyphonic_aftertouch`, `controller`, `program_change`,
    `channel_aftertouch`, `pitch_bend` (14-bit), `sys_ex` (full
    F0..F7 framing), `quarter_frame_msg`, `song_position_pointer_msg`,
    `song_select_msg`, `tune_request`, `clock`, `start`, `continue`,
    `stop`, `active_sensing`, `system_reset`, `all_notes_off`,
    `all_sound_off`, `reset_all_controllers`. Accessors: `is_note_on`,
    `is_note_off`, `note_number`, `velocity`, `controller_number`,
    `controller_value`, `pitch_bend_value`, `quarter_frame`,
    `song_position_pointer`, `song_select`, `sys_ex_payload`, etc.
  - `MidiMessage::parse` — single-pass parser, no allocations, handles
    channel messages, system common (SPP / song select / time code),
    realtime (`clock`, `start`, …), and SysEx (with explicit `0xF7`
    terminator). Running status is **not** handled — every parsed
    message starts with its status byte.
  - `MidiRPN` / `MidiRpnKind` — RPN vs NRPN, 7-bit or 14-bit values;
    `to_messages()` emits the parameter MSB/LSB CC pair plus the
    value MSB/LSB CC pair; `to_messages_with_null()` appends the
    null-parameter reset CCs. Standard RPN constants live in
    `standard_rpn::*` (pitch-bend sensitivity, MPE configuration,
    tuning program/bank, RPN null, …)
  - `MidiClock` — `samples ↔ ticks ↔ QN` math for the 24-ppqn MIDI
    clock. `samples_per_clock_tick`, `samples_to_ticks`, `ticks_to_samples`,
    `split_tick_delta`, plus BPM ↔ microseconds-per-QN conversions
  - `MTC` — `MtcRate` (24, 25, 29.97 drop, 30), `MtcTime` (with
    `to_frame_count` / `from_frame_count` for both non-drop and
    drop-frame), `MtcEncoder` (driven by `encode_frame()` calls;
    emits 8 quarter-frame messages per MTC frame plus an optional
    full-frame lead-in), `MtcFullFrame` (encode/decode the
    `[0xF0, 0x7F, 0x7F, 0x01, 0x01, hh, mm, ss, ff, 0xF7]` SysEx form)
  - 81 unit tests + 4 doc-tests passing under `--features full`
    (default = `["buffer", "midi"]`)
  - Features: `buffer` (default), `midi` (default), `full` = both
  - Crate: [`logic_nih_plug_audio_basics`](logic_nih_plug_audio_basics)

- [x] **Port `juce_audio_devices`** — ✅ done (2026-06-18)
  - `AudioDeviceSetup` — desired sample rate, buffer size, input /
    output channel counts; helpers `stereo_44100` / `stereo_48000`;
    validates against the active device's capabilities when one is
    attached
  - `AudioIODeviceType` — enum of every driver backend JUCE supports
    (`CoreAudio`, `ASIO`, `WASAPI`, `DirectSound`, `ALSA`, `JACK`,
    `AndroidAAudio`, `AndroidOpenSLES`, `Bela`, `IOSAudio`, `WebAudio`,
    `Dummy`), with `type_name()`, `description()`, `is_supported_on_current_platform()`,
    and `supported_on_current_platform()`
  - `DriverType::current()` — compile-time detection of the preferred
    driver for the current build target; `to_audio_io_device_type()`
    maps it to the matching `AudioIODeviceType`
  - `AudioIODevice` trait — `get_name`, `get_device_info`,
    `get_output_channel_names`, `get_input_channel_names`,
    `get_default_buffer_size`, `get_current_buffer_size`,
    `get_current_sample_rate`, `get_input_latency_in_samples`,
    `get_output_latency_in_samples`, `open` / `close` / `is_open`,
    `start` / `stop` / `is_playing`, `get_last_error`,
    `has_control_panel` / `show_control_panel`
  - `AudioDeviceInfo` — name, sample rates, buffer sizes, channel
    names, latencies; `validate_sample_rate` / `validate_buffer_size`
    helpers; `closest_sample_rate` / `closest_buffer_size` for
    nearest-neighbour fallback
  - `AudioIODeviceCallback` trait — `audio_device_about_to_start`,
    `audio_device_io_callback`, `audio_device_stopped` (real-time safe;
    mirrors `juce::AudioIODeviceCallback`); plus
    `AudioIODeviceCallbackData<'a>` with parallel input / output slice
    arrays and `NullAudioIODeviceCallback` for tests
  - `AudioDeviceManager` — long-lived orchestrator; holds the active
    `Box<dyn AudioIODevice>`, the desired `AudioDeviceSetup`, the
    current `AudioIODeviceType`, and the listener list. Lifecycle:
    `set_current_audio_device_type`, `set_current_audio_device`,
    `set_audio_device_setup` (reopens the active device if it's
    open), `open_device`, `play`, `stop`, `close_device`,
    `scan_device_names`, `play_test_sound` (no-op stub)
  - `AudioDeviceManagerState` — `Stopped` / `Open` / `Playing`
    tri-state machine
  - `AudioDeviceManagerListener` trait — fires on device-type change
    (`audio_device_manager_changed`) and setup change
    (`audio_device_setup_changed`)
  - `MockAudioIODevice` — concrete in-memory device for tests /
    headless hosts, with `event_log` + `callback_count` + `force_error`
    inspection helpers. `MockAudioIODeviceEvent::Opened` /
    `Closed` / `Started` / `Stopped` for lifecycle assertions.
  - 64 unit tests + 1 doc-test passing under `--features full`
  - Features: `manager` (default), `full` = `["manager"]`
  - Crate: [`logic_nih_plug_audio_devices`](logic_nih_plug_audio_devices)
  - Concrete driver bindings (`cpal`, `coreaudio-rs`, `asio-sys`, …)
    are intentionally NOT bundled — they require platform SDKs. The
    trait surface here is the integration point.

---

## 4. Core utilities (lightest items first)

- [x] **`juce_core` essentials** — ✅ shrunk (no crate, 2026-06-18)
  Every JUCE item here is already covered by Rust stdlib + existing
  workspace deps. No new crate needed; use the stdlib equivalent:
  - `File` → [`std::path::PathBuf`](https://doc.rust-lang.org/std/path/struct.PathBuf.html)
    + [`std::fs`](https://doc.rust-lang.org/std/fs/index.html)
  - `String` → [`std::string::String`](https://doc.rust-lang.org/std/string/struct.String.html)
  - `Array<T>` / `OwnedArray<T>` / `ReferenceCountedArray<T>` →
    [`std::vec::Vec`](https://doc.rust-lang.org/std/vec/struct.Vec.html)
    / `Vec<Box<T>>` / [`std::sync::Arc<[T]>`](https://doc.rust-lang.org/std/sync/struct.Arc.html)
    (or `Arc<Vec<T>>` if shrinking)
  - `Thread` → [`std::thread::JoinHandle`](https://doc.rust-lang.org/std/thread/struct.JoinHandle.html)
    (already used throughout [src/event_loop/background_thread.rs](src/event_loop/background_thread.rs))
  - `ThreadPool` → `rayon` (already in the dep tree)
  - `WaitableEvent` → `crossbeam_channel` (already in the dep tree) or
    `parking_lot::Mutex` + `Condvar`
  - `Time` / `RelativeTime` → [`std::time::Instant`](https://doc.rust-lang.org/std/time/struct.Instant.html)
    / [`std::time::Duration`](https://doc.rust-lang.org/std/time/struct.Duration.html)
  - `HighResolutionTimer` → `std::time::Instant::elapsed()` (Linux/macOS
    use `clock_gettime`, Windows uses `QueryPerformanceCounter` under
    the hood). If a sub-µs monotonic source is ever needed, add
    `quanta` then.
  - The realtime-safe trigger → message-loop pattern (`AsyncUpdater`)
    already lives in
    [`logic_nih_plug::event_loop::EventLoop::schedule_gui`](src/event_loop.rs);
    use that instead of inventing a new `AsyncUpdater` type.

- [x] **`juce_events`** — ✅ shrunk (no crate, 2026-06-18)
  - `Timer` (periodic tick) — backend-native: `egui::Context::request_repaint_after`,
    `iced::Subscription::interval`, `vizia::view::timer`. The pattern is
    the same in every backend; no shared abstraction needed.
  - `MessageManager::call_soon` (single-threaded GUI dispatch) → use
    [`EventLoop::schedule_gui`](src/event_loop.rs), which is already
    the realtime-safe → GUI-thread hop the framework provides.
  - `AsyncUpdater` (realtime-safe trigger → message-loop dispatch) → same
    as `MessageManager::call_soon`; covered by `schedule_gui`.

---

## 5. Graphics & GUI

- [x] **Expand `logic_nih_plug_graphics`** — ✅ done (2026-06-18)
  - `Path` (move/line/cubic/quad/close/subpath) via `PathBuilder` (wraps
    `tiny_skia::PathBuilder`)
  - `Stroke`, `FillType`, `Justification` (bitflags) — re-exported from
    `tiny_skia` or defined in `vector` module
  - `ColourGradient` (linear / radial) — wraps `tiny_skia::Shader<'static>`
  - `DropShadow` — colour + offset (no blur yet; tiny-skia doesn't ship
    Gaussian blur)
  - `Painter` — JUCE-style CPU paint target backed by `tiny_skia::Pixmap`;
    premultiplied-RGBA8 output with `data_straight()` for straight-alpha
    consumers. Fill/stroke paths, rects, gradients, shadows.
  - `GlyphArrangement` — shaped text layout with `PositionedGlyph` entries
  - `ImageConvolutionEngine` — reusable NxN convolution kernel; `box_blur_3x3`,
    `sharpen_3x3`, `edge_detect_3x3` presets
  - `Image::rescaled(new_w, new_h, RescaleFilter)` — `Nearest` / `Bilinear`
    (backed by `image` crate resize)
  - `Image::convolve` / `Image::convolve_in_place` — edge-clamped NxN
    convolution
  - `LineSpacing` — `Single` / `Multiple(f32)` / `Fixed(f32)`
  - `Font::get_string_width_float` (alias for `measure_text`),
    `Font::get_ascent`, `Font::get_descent`, `Font::get_height`
  - 58 unit tests + 33 doc-tests passing under `--features full`
  - New `vector` feature (opt-in via `tiny-skia` dep) to avoid pulling
    ~200 KiB into builds that only use pixel-pushing primitives

- [x] **Expand `logic_nih_plug_gui` controls** — ✅ done (2026-06-18)
  - `ComboBox` — drop-down selection with items, selection state, change callbacks
  - `TextEditor` — editable text field (single/multi-line, max length, read-only)
  - `CheckBox` / `ToggleButton` — boolean on/off controls with change callbacks
  - `ProgressBar` — horizontal progress bar (0.0–1.0) with optional text
  - `Tooltip` — tooltip manager with show/hide/delay
  - `DrawableButton` — button for custom-drawn content
  - `HyperlinkButton` — button styled as hyperlink with URL
  - `ImageComponent` — image display with scaling modes (`Fill`, `Fit`, `None`, `Stretch`)
  - All in `controls_extra.rs`; 46 unit tests passing; `render()` / `render_with_lookandfeel()` feature-gated on `graphics`
- [x] **`MidiKeyboardComponent`** — ✅ done (2026-06-18)
  - Visual piano keyboard with white/black key rendering, configurable
    MIDI note range (default C2–C6), and `KeyboardOrientation`
    (`Horizontal` / `Vertical`).
  - Mouse interaction: `mouse_down(x, y)` / `mouse_up()` / `mouse_drag(x, y)`
    with velocity based on vertical click position (top=hard, bottom=soft).
  - External active-note highlighting via `add_active_note` / `remove_active_note`.
  - Configurable key dimensions, colours, middle-C mapping, note-name display.
  - Hit testing with black-key priority (drawn on top, checked first).
  - Two-tier render: `render()` with hardcoded colours, `render_with_lookandfeel()`
    using the theme's `ColorScheme`.
  - 18 unit tests; in `controls_extra.rs`; re-exported as `MidiKeyboardComponent`
    and `KeyboardOrientation` at crate root.

- [x] **Expand layout** — ✅ done (2026-06-18)
  - `CssGrid` — full CSS Grid with `fr`/`px`/`auto`/`minmax()` track sizing,
    named grid areas, row/column spanning, gap control, per-item alignment
    (`Start`/`End`/`Center`/`Stretch`), and two-pass `fr` resolution that
    subtracts fixed tracks before distributing fractionally.
  - `RelativeCoordinate` — percentage or absolute pixel coordinates
    (`Absolute`, `Percent`, `FromRight`, `FromBottom`) with horizontal/
    vertical resolution against a parent dimension.
  - `RelativeRectangle` — four `RelativeCoordinate` edges for proportional
    child bounding boxes; `fill()`, `from_pixels()`, `from_percent()` helpers.
  - `AnimationFrameRate` — throttled redraw controller with configurable FPS,
    `should_frame(nanos)` / `should_frame_duration(Duration)`, time-until-next
    queries, enable/disable, and reset.
  - All in `layout/css_grid.rs`, `layout/relative.rs`, `layout/animation_frame_rate.rs`;
    22 CSS Grid + 19 relative + 18 AnimationFrameRate = 59 new unit tests;
    re-exported at crate root via `layout` module.

- [x] **Port `juce_opengl`** — ✅ done (2026-06-18)
  - `OpenGLContext` wrapper (build on top of `glow` + `baseview` GL context)
  - `OpenGLRenderer` trait + `RenderLoopDriver` frame-loop helper
  - `OpenGLHelpers` (compile-shader, link-program, check-errors, blend/depth/cull helpers)
  - `ShaderProgram` wrapper (compile, link, uniform setters, `UniformLocation`)
  - `OpenGLTexture` (create, upload, bind, mipmaps, filtering, wrapping)
  - `OpenGLFrameBuffer` (FBO with color + optional depth/stencil, read-back, resize)
  - `Matrix3D` / `Matrix4x4` (column-major GL math, perspective/ortho/look-at/rotation)
  - `TextureFormat`, `TextureDataType`, `TextureMinFilter` enums
  - 66 unit tests; feature-gated on `gl-editor`; re-exported at crate root
  - `OpenGLShaderCode` (JUCE GLSL translator) — deferred

- [x] **Port `juce_video`** — ✅ done (2026-06-18)
  - `VideoFrame` — RGBA8888 frame with pixel access, solid-colour factory
  - `VideoDecoder` — ffmpeg-next wrapper: open, decode, seek, rewind
    (feature `decoder`; requires FFmpeg system libs at build time)
  - `VideoComponent` — GUI component: play/pause/stop, push_frame,
    callbacks (playback started/stopped/error), speed/volume/position
    control, component delegation (feature `gui`)
  - `PlaybackState` — Stopped/Playing/Paused enum
  - `VideoError` — 7 error variants with `thiserror` display
  - Feature flags: `decoder` (default), `gui`, `full`; decoder feature
    is optional so the core types compile without FFmpeg
  - 39 unit tests (15 core + 24 component) passing

---

## 6. Audio processors (host side)

- [ ] **Port `juce_audio_processors` host utilities**
  - `AudioPluginInstance` trait (host → plugin bridge)
  - `PluginDescription`, `PluginDirectoryScanner`
  - `KnownPluginList`
  - Standalone host harness using `cpal` + `nih_plug_xtask` plugin-loader

- [ ] **Port `juce_audio_utils`**
  - `MidiKeyboardComponent`
  - `AudioThumbnail` (waveform preview)
  - `MidiFilePlayer` / `MidiFileWriter` (re-use
    `logic_nih_plug_audio_formats`)
  - `AudioFilePlayer` (streaming)
  - `MultiDocumentPanel`, `ApplicationCommandManager`

---

## 7. Examples & meta

- [ ] **Create `plugins/examples/juce_multi_module`** (referenced in
  [plugins/examples/JUCE_EXAMPLES.md](plugins/examples/JUCE_EXAMPLES.md) but
  the crate is missing)
  - Add it to the workspace in [Cargo.toml](Cargo.toml)
  - Implement a synth using oscillators + filter + ADSR + value-tree preset
  - Uses `logic_nih_plug_data`, `logic_nih_plug_crypto`,
    `logic_nih_plug_animation`, `logic_nih_plug_audio_formats`

- [ ] **Cross-link `TODO.md` from `AGENTS.md`** so future agents see the
  backlog immediately

- [ ] **Port `juce_product_unlocking`** *(low priority)*
  - `KeyGeneration`, `RSAKey`, `OnlineUnlockStatus`

---

## Conventions for new ports

- Mirror the style of an existing crate first. The closest reference points:
  - DSP algorithms → [logic_nih_plug_dsp/src](logic_nih_plug_dsp/src)
  - GUI components → [logic_nih_plug_gui/src](logic_nih_plug_gui/src)
  - I/O formats → [logic_nih_plug_audio_formats/src](logic_nih_plug_audio_formats/src)
- Add the new crate to the `[workspace].members` list in
  [Cargo.toml](Cargo.toml) **before** the first build.
- Provide Cargo features in the same shape: `default` = the safe surface,
  `full` = all features. Look at
  [logic_nih_plug_dsp/Cargo.toml](logic_nih_plug_dsp/Cargo.toml) for the
  pattern.
- Document with `//!` module headers and at least one runnable example.
- Add an entry to [CHANGELOG.md](CHANGELOG.md) for any public-API bump.
- No real-time allocations in DSP code. See the
  *Hard rules* in [AGENTS.md](AGENTS.md) §5.

---

## Status snapshot

| Crate | Status |
|---|---|
| `logic_nih_plug_animation` | ✅ Ported |
| `logic_nih_plug_audio_basics` | ✅ Ported (AudioSampleBuffer, AudioChannelSet, MidiMessage, MidiRPN, MidiClock, MTC) |
| `logic_nih_plug_audio_devices` | ✅ Ported (AudioDeviceManager, AudioIODevice trait, AudioIODeviceType, AudioDeviceSetup, AudioIODeviceCallback, MockAudioIODevice) |
| `logic_nih_plug_audio_formats` | ✅ Ported (WAV / AIFF / FLAC / OGG) |
| `logic_nih_plug_derive` | ✅ Ported |
| `logic_nih_plug_dsp` | ✅ Ported — Dynamics, Reverb, Delay, Modulation, Mixer, Analysis, Pitch, Resampling & FFT upgrades done (incl. PhaseVocoder ↔ RealFFT integration) |
| `logic_nih_plug_egui` / `_iced` / `_vizia` | ✅ Ported (GUI backends) |
| `logic_nih_plug_graphics` | ✅ Expanded — Path/Stroke/Fill/Gradient/DropShadow/Painter + GlyphArrangement/LineSpacing + Image rescale/convolve (§5 graphics items done) |
| `logic_nih_plug_gui` | 🟡 Partial — see §5 |
| `logic_nih_plug` core | ✅ Ported |
| `logic_nih_plug_xtask` / `cargo_logic_nih_plug` | ✅ Ported |
| `logic_nih_plug_data` | ✅ Ported (ValueTree, UndoManager, CachedValue) |
| `logic_nih_plug_crypto` | ✅ Ported (SHA-256/1/MD5, BigInteger, RSAKey) |
| `logic_nih_plug_osc` | ✅ Ported (OscSender, OscReceiver, OSCArgument, OSCBundle) |
| `logic_nih_plug_midi_ci` | ✅ Ported (32 message types, transport-agnostic) |
| `logic_nih_plug_video` | ✅ Ported (VideoFrame, VideoDecoder, VideoComponent; 39 tests) |
