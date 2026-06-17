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

- [ ] **Port `juce_osc` → `logic_nih_plug_osc`**
  - `OscSender` (UDP, `rosc` crate)
  - `OscReceiver` with pattern matching (`/foo/bar i=42 f=3.14 s="x"`)
  - `OSCArgument` sum type
  - `OSCBundle` (time-tagged)
  - Default features: `sender`, `receiver`; `full` = both

- [ ] **Port `juce_midi_ci` → `logic_nih_plug_midi_ci`**
  - MIDI-CI discovery (PE, PI, PE stream)
  - Profile configuration (MIDI 1.0 / MIDI 2.0)
  - Property exchange (per-channel `ValueTree`-like)
  - Default features: `discovery`, `profiles`, `property-exchange`; `full` = all

---

## 2. DSP module — core effects/analysis (JUCE `dsp` module)

The `logic_nih_plug_dsp` crate has filters/oscillators/convolution/envelopes
already. These are the missing processors.

- [ ] **Dynamics** — `Compressor`, `Limiter`, `NoiseGate`, `Limiter` (lookahead)
- [ ] **Reverb** (algorithmic) — Schroeder allpass + comb topology, stereo
  decorrelation
- [ ] **Delay** (feedback) — tempo-synced, ping-pong
- [ ] **Modulation** — `Phaser`, `Chorus`, `WahWah`, `LadderFilter`
- [ ] **Mixer helpers** — `Panner` (equal-power, linear, square-root), `DryWet`,
  real `Gain` processor (struct exists but is incomplete)
- [ ] **Analysis** — `LoudnessMeter` (ITU-R BS.1770 K-weighting + momentary /
  short-term / integrated LUFS), `Oscilloscope`, `Follower`, `LevelMeter`
- [ ] **Pitch processing** — `PitchShift` (phase vocoder), `TimeStretching`
- [ ] **Resampling** — `WindowedSincInterpolator` (all orders), `Lagrange`,
  `CatmullRom`, `ZeroOrderHold`, `Linear`
- [ ] **FFT upgrades** — real-only FFT path (`rustfft`'s `FftPlanner` plan
  variant), `WindowingFunction` enum, `dsp::STFT` (forward-inverse helpers)

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
| `logic_nih_plug_dsp` | 🟡 Partial — see §2 for missing processors |
| `logic_nih_plug_egui` / `_iced` / `_vizia` | ✅ Ported (GUI backends) |
| `logic_nih_plug_graphics` | 🟡 Partial — see §5 |
| `logic_nih_plug_gui` | 🟡 Partial — see §5 |
| `logic_nih_plug` core | ✅ Ported |
| `logic_nih_plug_xtask` / `cargo_logic_nih_plug` | ✅ Ported |
| `logic_nih_plug_data` | ✅ Ported (ValueTree, UndoManager, CachedValue) |
| `logic_nih_plug_crypto` | ✅ Ported (SHA-256/1/MD5, BigInteger, RSAKey) |
| `logic_nih_plug_osc` | ❌ Missing (§1) |
| `logic_nih_plug_midi_ci` | ❌ Missing (§1) |
