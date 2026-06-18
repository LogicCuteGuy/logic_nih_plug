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

- [ ] **Port `juce_audio_basics`**
  - `AudioSampleBuffer` (interleaved / non-interleaved) — Rust already has
    `nih_plug::buffer::Buffer`; add a parallel `AudioSampleBuffer` for
    cross-crate use
  - `MidiMessage` parser/builder (full status bytes, sysex, meta, RPN/NRPN)
  - `MidiRPN`, `MidiClock`, `MTC` helpers
  - `AudioChannelSet` (mono, stereo, 5.1, 7.1, ambisonics)

- [ ] **Port `juce_audio_devices`**
  - `AudioDeviceManager`, `AudioIODeviceType` enum
  - `AudioDeviceSetup` (sample rate, buffer size, channels)
  - `AudioInputDevice` / `AudioOutputDevice` callbacks
  - Driver type detection: `CoreAudio`, `ASIO`, `WASAPI`, `ALSA`, `JACK`

---

## 4. Core utilities (lightest items first)

- [ ] **Port `juce_core` essentials**
  - `File` (path + metadata + read/write helpers; defer FS operations to
    `std::fs`)
  - `String` thin wrapper (Rust `String` is enough; defer)
  - `Array<T>`, `OwnedArray<T>`, `ReferenceCountedArray<T>` — use `Vec<T>`,
    document when to use `Arc<Vec<T>>`
  - `Thread`, `ThreadPool`, `WaitableEvent` (crossbeam) — already partially
    available through `nih_plug` background tasks
  - `Time`, `RelativeTime`, `HighResolutionTimer`

- [ ] **Port `juce_events`**
  - `Timer` (tick callback, start/stop)
  - `MessageManager` (single-threaded `call_soon` for the GUI thread)
  - `AsyncUpdater` (realtime-safe trigger → message-loop dispatch)

---

## 5. Graphics & GUI

- [ ] **Expand `logic_nih_plug_graphics`**
  - `Path` (move/line/cubic/quad/close/subpath)
  - `Stroke`, `FillType`, `Justification`
  - `ColourGradient` (linear / radial)
  - `DropShadow`
  - `GlyphArrangement` / shaped text layout
  - `ImageConvolutionEngine`, `Image::rescaled`, `Image::convolve`
  - `LineSpacing`, `Font::getStringWidthFloat`

- [ ] **Expand `logic_nih_plug_gui` controls** (only `Button`/`Slider`/`Label`
  exist)
  - `ComboBox`
  - `TextEditor`
  - `CheckBox` / `ToggleButton`
  - `ProgressBar`
  - `Tooltip`
  - `DrawableButton`, `HyperlinkButton`
  - `ImageComponent`, `MidiKeyboardComponent` (see §6)

- [ ] **Expand layout**
  - `Grid` (CSS Grid)
  - `RelativeRectangle` / `RelativeCoordinate`
  - `AnimationFrameRate` (throttled redraw)

- [ ] **Port `juce_opengl`**
  - `OpenGLContext` wrapper (build on top of the existing `gl-editor` feature
    in `logic_nih_plug_gui`)
  - `OpenGLRenderer` trait
  - `OpenGLHelpers` (compile-shader, link-program, check-errors)
  - `OpenGLShaderCode` (juce shader code translator) — defer

- [ ] **Port `juce_video`** *(low priority)*
  - `VideoComponent` (frame decoder; needs `ffmpeg-next`)

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
| `logic_nih_plug_audio_formats` | ✅ Ported (WAV / AIFF / FLAC / OGG) |
| `logic_nih_plug_derive` | ✅ Ported |
| `logic_nih_plug_dsp` | ✅ Ported — Dynamics, Reverb, Delay, Modulation, Mixer, Analysis, Pitch, Resampling & FFT upgrades done (incl. PhaseVocoder ↔ RealFFT integration) |
| `logic_nih_plug_egui` / `_iced` / `_vizia` | ✅ Ported (GUI backends) |
| `logic_nih_plug_graphics` | 🟡 Partial — see §5 |
| `logic_nih_plug_gui` | 🟡 Partial — see §5 |
| `logic_nih_plug` core | ✅ Ported |
| `logic_nih_plug_xtask` / `cargo_logic_nih_plug` | ✅ Ported |
| `logic_nih_plug_data` | ✅ Ported (ValueTree, UndoManager, CachedValue) |
| `logic_nih_plug_crypto` | ✅ Ported (SHA-256/1/MD5, BigInteger, RSAKey) |
| `logic_nih_plug_osc` | ✅ Ported (OscSender, OscReceiver, OSCArgument, OSCBundle) |
| `logic_nih_plug_midi_ci` | ✅ Ported (32 message types, transport-agnostic) |
