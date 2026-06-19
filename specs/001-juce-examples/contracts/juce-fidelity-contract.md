# JUCE-Fidelity Contract

**Feature**: [spec.md](../spec.md) | **Constitution**: §V "JUCE Port Fidelity"
**Date**: 2026-06-19

Every new example crate in the JUCE-Style Examples Portfolio MUST satisfy this
contract. It is the per-crate instance of the constitution's "JUCE Port Fidelity"
principle.

---

## 1. Constitution §V verbatim

> Port only what the JUCE reference implementation does. Where the JUCE example
> uses a JUCE module that is not yet ported to Rust, the example's status is
> `skipped(<module>)` per FR-013, never silent.

## 2. Per-crate checklist

Every example's `README.md` includes a "## JUCE fidelity checklist" section
(near the end) that fills in this template:

```markdown
## JUCE fidelity checklist

- [ ] **Source file named**: <path under JUCE examples/>
- [ ] **Public API surface unchanged**: the example calls only types/methods
      that exist in the current `logic_nih_plug*` sub-crate tree; no private
      API is touched.
- [ ] **Matches JUCE behavior** (per constitution §V): <describe the
      algorithm or observable behavior the example replicates, with a
      reference to a doc-test that proves it>
- [ ] **Skipped modules** (if any): <`none` | a comma-separated list of
      JUCE module names that are not yet ported; the example status in
      the ledger is then `skipped(<module>)` not `ported`>
- [ ] **One behavioral doc-test** passes: `cargo test --doc -p <crate>` exits 0
```

## 3. Algorithm: what counts as a "match"

A ported example "matches JUCE behavior" when **at least one of** the following
holds, in priority order:

1. **Bit-exact round-trip** against a checked-in reference fixture
   (e.g. `assets/ir.wav` for `ConvolutionDemo`). Used by US1 convolution.
2. **Mathematical identity** at a known sample (e.g. `OscillatorDemo`'s sine
   output at sample 0 is `0.0`, at sample `N/4` is `1.0`).
3. **Closed-form invariant** (e.g. `IIRFilterDemo`'s low-pass at Nyquist
   output is `<` the input at the cutoff frequency).
4. **Lifecycle assertion** for I/O demos (`MockAudioIODevice::event_log()`
   shows `Opened → Started → Stopped → Closed`).
5. **Header / metadata round-trip** for file-IO demos (WAV RIFF/WAVE/fmt/data
   chunks match a reference; SMF tempo + tracks + events match a reference).

The doc-test selects the highest-priority applicable match for the example's
domain. If none of these applies, the example's ledger row becomes
`skipped(<module>)` and the README documents the gap explicitly.

## 4. Skipped modules — the escape hatch

JUCE has many modules that the Rust port does not (yet) cover. Examples that
depend on a missing module are not silently dropped — they are recorded in
the ledger with `status = skipped(<module>)` and a "Skipped modules" line
in the README explaining what is missing and what would unblock the port.

Examples of currently-not-ported JUCE modules (incomplete list — check the
ledger for accuracy):

- `juce_graphics` (full) — only the `logic_nih_plug_graphics` subset is ported
- `juce_video` capture + camera — only playback is ported
- `juce_cryptography` (streaming primitives)
- `juce_analytics` (HockeyApp / Sentry integration)
- `juce_box2d` binding
- `juce_opengl` (full; only a subset is exposed via `logic_nih_plug_gui::opengl`)
- `juce_webkit` (WebViewPluginDemo)
- `juce_audio_utils` (UI components like the audio settings panel)
- `juce_ARA` (ARA SDK integration)

When the underlying module lands in the workspace, the example's status flips
from `skipped(<module>)` to `ported` and the implementation work resumes.

## 5. Enforcement

| Rule | Test / Tool | Where |
|---|---|---|
| Every example README has a "JUCE fidelity checklist" section | `tests/example_readme_required.rs` (T016) | `cargo test --workspace` |
| Doc-test exists and passes | `cargo test --doc -p <crate>` | per-example CI smoke (T019, T039, T049, T058, T068, T085) |
| No example is silently omitted | ledger `status` column | manual review (T094) |
| The check list names the source file | README front-matter `juce_source:` | `tests/example_readme_required.rs` (T016) |

## 6. Why this matters

Without this contract, the portfolio would drift into "example-shaped" code
that demonstrates the Rust API but does not actually port the JUCE example.
That defeats the portfolio's purpose: a developer familiar with JUCE should
be able to open the Rust port and immediately recognize the algorithm and
the structure of the C++ original. The contract keeps the port honest.
