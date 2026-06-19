# Plugins

Nine real plugin crates ship in this repo. All listed in
[bundler.toml](../bundler.toml); each builds to `target/bundled/<name>/`.

| Plugin | What it does |
|---|---|
| [buffr_glitch](../plugins/buffr_glitch/) | MIDI-triggered buffer repeat; CD-skip textures |
| [crisp](../plugins/crisp/) | Bright crispy top end for bass (Polarity "Fake Distortion") |
| [crossover](../plugins/crossover/) | 2–5 band split with linear-phase option, aux outputs |
| [diopser](../plugins/diopser/) | Phase rotation around a frequency; allpass stack |
| [loudness_war_winner](../plugins/loudness_war_winner/) | Yes, it really does that |
| [puberty_simulator](../plugins/puberty_simulator/) | Octave-down + voice-crack texture |
| [safety_limiter](../plugins/safety_limiter/) | Plays SOS in Morse code when peaks exceed threshold |
| [soft_vacuum](../plugins/soft_vacuum/) | Airwindows Hard Vacuum port with up to 16× oversampling |
| [spectral_compressor](../plugins/spectral_compressor/) | Up to 16384-band OTT-style spectral compression |

Build any of them:

```shell
cargo xtask bundle <name> --release
```

## Picking a reference for a new plugin

- **Oversampling + smoothing** → `soft_vacuum`
- **Parameter callbacks** → `safety_limiter`
- **GUI integration** → `examples/gain_gui_egui`
- **SIMD DSP** → `diopser` (uses `logic_nih_plug_dsp` `simd` feature, nightly only)
- **FFT-heavy** → `spectral_compressor`

## Examples

[plugins/examples/](../plugins/examples/) — bare-format and GUI skeletons:

- [gain](../plugins/examples/gain/) — minimal stub
- [gain_gui_egui](../plugins/examples/gain_gui_egui/) — egui editor + peak meter (cross-thread `Arc<AtomicF32>` reference)
- [gain_gui_iced](../plugins/examples/gain_gui_iced/), [gain_gui_vizia](../plugins/examples/gain_gui_vizia/)
- [gain_multi_format](../plugins/examples/gain_multi_format/), [gain_vst2](../plugins/examples/gain_vst2/), [gain_au](../plugins/examples/gain_au/), [gain_auv3](../plugins/examples/gain_auv3/), [gain_lv2](../plugins/examples/gain_lv2/), [gain_aax](../plugins/examples/gain_aax/)
- [overdrive](../plugins/examples/overdrive/), [sine](../plugins/examples/sine/)
- [midi_inverter](../plugins/examples/midi_inverter/), [sysex](../plugins/examples/sysex/), [poly_mod_synth](../plugins/examples/poly_mod_synth/)
- [stft](../plugins/examples/stft/), [spectrum_analyzer](../plugins/examples/spectrum_analyzer/)
- [state_variable_filter](../plugins/examples/state_variable_filter/), [juce_dsp_filter](../plugins/examples/juce_dsp_filter/)
- [flexbox_demo](../plugins/examples/flexbox_demo/) — `logic_nih_plug_gui` FlexBox port
- [delay](../plugins/examples/delay/) — tempo-synced feedback delay with ping-pong
- [reverb](../plugins/examples/reverb/) — FreeVerb-style algorithmic reverb
- [chorus](../plugins/examples/chorus/) — LFO-modulated stereo chorus
- [sidechain_compressor](../plugins/examples/sidechain_compressor/) — aux sidechain input + envelope-follower compressor
- [note_expressions](../plugins/examples/note_expressions/) — polyphonic synth reacting to every `NoteEvent::*Poly*` event

See [FORMAT_EXAMPLES.md](../plugins/examples/FORMAT_EXAMPLES.md) and
[JUCE_EXAMPLES.md](../plugins/examples/JUCE_EXAMPLES.md).
