# Documentation Index

The repo's docs are scattered. This file lists what actually exists in the
working copy and what `AGENTS.md` points to but doesn't.

## Root

- [README.md](README.md) — framework overview, plugins, features, build
- [AGENTS.md](AGENTS.md) — table of contents for AI coding agents
- [CHANGELOG.md](CHANGELOG.md) — per-day breaking changes (no stable release yet)

## Sub-crate docs

| Crate | Doc |
|---|---|
| `logic_nih_plug_animation` | [README.md](logic_nih_plug_animation/README.md) |
| `logic_nih_plug_egui` | [README.md](logic_nih_plug_egui/README.md) |
| `logic_nih_plug_iced` | [README.md](logic_nih_plug_iced/README.md) |
| `logic_nih_plug_vizia` | [README.md](logic_nih_plug_vizia/README.md) |
| `logic_nih_plug_xtask` | [README.md](logic_nih_plug_xtask/README.md) |
| `logic_nih_plug_gui` | [CONTROLS.md](logic_nih_plug_gui/CONTROLS.md), [LOOKANDFEEL.md](logic_nih_plug_gui/LOOKANDFEEL.md) (no README) |

`logic_nih_plug_audio_formats`, `logic_nih_plug_derive`, `logic_nih_plug_dsp`, `logic_nih_plug_graphics`
have no README — read their `src/lib.rs` module docs and tests instead.

## Example plugins

- [FORMAT_EXAMPLES.md](plugins/examples/FORMAT_EXAMPLES.md) — VST2/AU/AUv3/LV2/AAX/multi-format variants
- [JUCE_EXAMPLES.md](plugins/examples/JUCE_EXAMPLES.md) — `juce_dsp_filter`, `juce_gui_demo`
- Per-format READMEs: [gain_vst2](plugins/examples/gain_vst2/README.md), [gain_aax](plugins/examples/gain_aax/README.md), [gain_au](plugins/examples/gain_au/README.md), [gain_auv3](plugins/examples/gain_auv3/README.md), [overdrive](plugins/examples/overdrive/README.md)

## Real plugins

| Plugin | README | CHANGELOG |
|---|---|---|
| buffr_glitch | [link](plugins/buffr_glitch/README.md) | [link](plugins/buffr_glitch/CHANGELOG.md) |
| crossover | [link](plugins/crossover/README.md) | — |
| crisp | [link](plugins/crisp/README.md) | — |
| diopser | [link](plugins/diopser/README.md) | [link](plugins/diopser/CHANGELOG.md) |
| loudness_war_winner | [link](plugins/loudness_war_winner/README.md) | — |
| puberty_simulator | [link](plugins/puberty_simulator/README.md) | — |
| safety_limiter | [link](plugins/safety_limiter/README.md) | [link](plugins/safety_limiter/CHANGELOG.md) |
| soft_vacuum | [link](plugins/soft_vacuum/README.md) | [link](plugins/soft_vacuum/CHANGELOG.md) |
| spectral_compressor | [link](plugins/spectral_compressor/README.md) | [link](plugins/spectral_compressor/CHANGELOG.md) |

## Bundler

[cargo_logic_nih_plug/README.md](cargo_logic_nih_plug/README.md) — `cargo nih-plug` subcommand.

## `docs/`

This folder.

| File | Purpose |
|---|---|
| [README.md](README.md) | this index |
| [getting-started.md](getting-started.md) | build, test, toolchain, hard rules |
| [plugin-skeleton.md](plugin-skeleton.md) | minimal `gain`-style plugin walkthrough |
| [dsp-and-gui.md](dsp-and-gui.md) | `logic_nih_plug_dsp`, GUI backends, BYO-GUI, animation |
| [bundling.md](bundling.md) | `cargo xtask`, formats, `cargo-nih-plug`, CI |
| [plugins.md](plugins.md) | the 9 real plugins + examples directory |
| [git-workflow.md](git-workflow.md) | remotes, branches, CI gates, fork sync |

The published framework docs live at https://nih-plug.robbertvanderhelm.nl/.

## Files AGENTS.md links to but that are not in the working copy

These are referenced by `AGENTS.md` as if they exist, but they're absent from
this checkout. Follow the upstream `master` branch or the published docs site
for them:

- `QUICK_START.md` — see README's JUCE Ported Modules + `logic_nih_plug_dsp` `lib.rs`
- `MIGRATION_GUIDE.md`, `JUCE_MODULES.md`, `VALIDATION_TEST_SUMMARY.md` — published docs site
- `RELEASE_CHECKLIST.md`, `RELEASE_NOTES.md`, `RELEASE_SUMMARY.md` — published docs site
- `MULTI_FORMAT_EXPORT.md` — partially covered by `FORMAT_EXAMPLES.md`
- `BENCHMARKING.md` — `criterion = "0.5"` is the standard; benches live in each crate's `benches/`
- `API_REFERENCE.md` — published docs site
