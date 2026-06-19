<!-- Sync Impact Report (v0.0.0 → v1.0.0)
=================================================
Version change: 0.0.0 (placeholder template) → 1.0.0 (initial ratified governance)
Bump rationale:  MAJOR — first stable ratification of governance content; no
                 prior numbered version existed.

Modified principles: none (initial publication, all new)
Added sections:
  - Core Principles §I–V (Real-Time Safety, Stable Public Identifiers,
    Smallest Correct Change, Multi-Format & Cross-Platform First,
    JUCE Port Fidelity)
  - Audio-Thread Discipline & Cross-Thread Communication
  - Testing, Documentation & Public-API Standards
  - Governance (amendment procedure, versioning, compliance review)
Removed sections: none

Templates requiring updates:
  - .specify/templates/plan-template.md   ✅ updated  — Constitution Check
        block now references the seven gates G1–G7 drawn from §I–V, the
        Audio-Thread Discipline section, and the Testing/Documentation
        section. Re-verified during initial ratification.
  - .specify/templates/spec-template.md    ✅ no change required (technology-
        agnostic requirement structure is compatible).
  - .specify/templates/tasks-template.md   ✅ no change required (task shape
        already accommodates test-first and per-story phases).
  - .specify/templates/checklist-template.md  ✅ no change required.
  - .specify/workflows/speckit/workflow.yml  ✅ no change required.

Runtime guidance docs (for AI agents and contributors):
  - AGENTS.md  ✅ already encodes §I, §II, §IV and the Audio-Thread
        Discipline section as hard rules; this constitution ratifies
        those rules into Spec-Kit governance. No edit needed.
  - README.md  ✅ no edit needed.
  - docs/getting-started.md  ✅ no edit needed (already references the
        same hard rules).

Deferred items:
  - TODO(RATIFICATION_DATE_PRECEDENT): A "pre-constitution" project
        adoption date is unknown. Treated as the same date as
        LAST_AMENDED_DATE for this initial ratification.
-->

# logic_nih_plug Constitution

This constitution is the **non-negotiable governance** for the
`logic_nih_plug` workspace — a Rust audio-plugin framework forked from
[`robbert-vdh/nih-plug`](https://github.com/robbert-vdh/nih-plug) with
pure-Rust ports of selected JUCE modules. It supersedes ad-hoc practices
documented in `AGENTS.md`, `README.md`, and inline source comments.

Where this constitution conflicts with an `AGENTS.md` rule, **this
constitution wins**. Where `AGENTS.md` is more specific (e.g., exact
crate paths, exact identifier strings), follow `AGENTS.md` after first
confirming compliance with the corresponding principle below.

---

## Core Principles

### I. Real-Time Safety (NON-NEGOTIABLE)

The audio-processing thread is the hardest real-time deadline in the
project. Violations cause glitches, dropouts, and host DAW crashes.

- `Plugin::process()` and any code it transitively calls MUST NOT
  allocate, MUST NOT block on locks (`std::sync::Mutex::lock`,
  `RwLock::read`/`write`, parking_lot blocking variants), MUST NOT
  perform syscalls (`println!`, `dbg!`, file I/O, network), and MUST
  NOT panic in the audio path.
- `Plugin::initialize()` is the only allocation-heavy lifecycle
  method. Everything an `initialize()` allocates MUST live for the
  lifetime of the plugin or be guarded by a `Vec::with_capacity`-style
  reservation performed up front.
- The debug feature `assert_process_allocs` MUST remain enabled in CI
  builds so allocation regressions are caught before release.
- Time-varying parameters MUST be smoothed. Direct writes to audible
  coefficients (gain, frequency, mix) MUST go through a `Smoother` or
  per-sample ramp.
- Rationale: a single allocation in `process()` triggers an
  `mmap`/heap-page fault that can stall the audio thread for
  milliseconds — audible as a click on every host that runs the plugin.

### II. Stable Public Identifiers

Identifier strings are part of the plugin's wire contract with the
host and the user's saved state. Silent renames lose user presets and
break automation.

- Parameter IDs (`#[id = "…"]`), persistent-state fields
  (`#[persist = "…"]`), nested-parameter group names
  (`#[nested(group = "…")]`, `array`), VST3 class IDs
  (`VST3_CLASS_ID`), CLAP IDs (`CLAP_ID`), and per-format unique
  identifiers MUST be treated as **versioned contracts**.
- Renaming a Rust field that carries one of these attributes MUST NOT
  change the string. Add the new attribute on the new field; deprecate
  the old by `#[id = "old-id"]` until a major version bump removes it.
- New parameters MAY be appended; existing ones MUST NOT be reordered
  (host automation lanes are positional).
- Rationale: VST3 hosts serialize by ID; CLAP hosts by ID; AU by
  parameter address. A rename silently drops the user's saved value
  with no warning.

### III. Smallest Correct Change (YAGNI)

Prefer the minimum change that satisfies the requirement. Reuse before
build; stdlib before dependency; in-tree before new.

- New functionality MUST ship as a self-contained sub-crate under the
  workspace root when it can stand alone. Do not extend existing
  modules past their stated scope.
- Cross-platform shims MUST use Rust stdlib (`PathBuf`, `Instant`,
  `Duration`, `Vec`, `Arc`) or dependencies already in the workspace
  dependency tree. Do **not** create a `logic_nih_plug_core` (or
  similar) crate that wraps them — that is cargo-culting
  `pub struct X(pub Y);`.
- Speculative abstractions (traits with one implementer, generic
  parameters with one instantiation, `Box<dyn Trait>` for a single
  concrete type) MUST be rejected in review.
- A new external dependency MUST be justified in the PR description
  by naming the stdlib / in-tree alternative that was considered and
  rejected, and why.
- Rationale: the existing workspace already carries ~20 sub-crates
  and 30 plugin crates; each additional abstraction layer makes the
  build graph harder to reason about and slows CI.

### IV. Multi-Format & Cross-Platform First

A plugin in this workspace MUST be exportable across all supported
formats (CLAP, VST3, AU, AUv3, LV2, AAX, standalone) and runnable on
Linux, macOS, and Windows from a single source.

- Every plugin crate MUST compile under
  `cargo build -p <crate>` with the workspace default features
  (which include VST3 and CLAP). Other formats are opt-in via the
  format-specific feature.
- VST2 and AAX MUST NOT be enabled together in a single plugin crate
  (linker collision). They ship as separate crates
  (see `plugins/examples/gain_vst2`).
- Per-format unique identifiers (`VST3_CLASS_ID`, `CLAP_ID`) MUST be
  globally unique within this workspace. The xtask bundler MUST
  reject duplicate identifiers at build time.
- `aax` is a stub and `au`/`auv3` are macOS/iOS-only. Code paths
  that assume a single platform MUST be guarded with
  `#[cfg(target_os = "…")]`; cross-platform code MUST be exercised
  on at least one CI runner per OS (see
  `.github/workflows/build.yml`).
- Rationale: a fork claiming JUCE parity MUST cover the same plugin
  format matrix JUCE supports, otherwise users cannot adopt it as a
  drop-in.

### V. JUCE Port Fidelity (Fork-Specific)

This fork's distinguishing mission is pure-Rust ports of JUCE modules.
The port contract is the public-API boundary, not internal
implementation.

- A sub-crate that ports a JUCE module MUST expose, at its public
  surface, types and methods whose names, semantics, and signatures
  match the corresponding JUCE C++ class or function (e.g.
  `juce::ValueTree` → `logic_nih_plug_data::ValueTree`,
  `juce::OnlineUnlockStatus` →
  `logic_nih_plug_product_unlocking::OnlineUnlockStatus`).
- Type names, error variants, and module paths MUST keep the JUCE
  spelling (case and CamelCase). Pure-Rust idioms (`From`, `Into`,
  `Result`, `&[T]`) are encouraged internally but MUST NOT leak into
  public re-exports where the equivalent JUCE call exists.
- Tests SHOULD include at least one named "matches JUCE behavior"
  regression per non-trivial public type, with a citation to the
  corresponding JUCE source line.
- Rationale: the entire adoption story for this fork is "JUCE users
  can move to Rust without re-learning the API." Breaking that
  contract defeats the fork's purpose.

---

## Audio-Thread Discipline & Cross-Thread Communication

This section operationalizes Principle I. Treat the rules below as
non-negotiable when touching any code reachable from `process()`.

- Cross-thread state MUST use one of:
  - `Arc<AtomicF32>` / `Arc<AtomicI32>` / `Arc<AtomicBool>` for
    primitive signals.
  - `Arc<parking_lot::Mutex<T>>` with `try_lock()` on the audio side.
  - `crossbeam_channel::Sender` / `Receiver` for bounded message
    passing.
- `std::sync::Mutex::lock()`, `RwLock::read()`, and `RwLock::write()`
  MUST NOT appear in audio-thread-reachable code. Clippy and CI
  lint configurations SHOULD flag them.
- Cross-thread wake-ups for parameter changes MUST use the
  framework's `EventLoop::schedule_gui` / parameter-smoothing path,
  not hand-rolled polling.
- Logging in any audio-reachable path MUST go through `nih_log!` and
  `nih_dbg!` from `logic_nih_plug::debug`. `println!`, `dbg!`, and
  `assert!` MUST NOT be used in plugin source; use `nih_dbg!` and
  `nih_assert!` instead.
- Stack frames larger than a few KiB MUST be heap-allocated
  (`Box<[T; N]>`) on Windows, where the audio thread's default stack
  is small.
- Rationale: each rule above is a specific failure mode that has been
  observed in the wild or in this codebase's history.

---

## Testing, Documentation & Public-API Standards

- Every public type in a sub-crate MUST have at least one unit test
  under `#[cfg(test)] mod tests` and at least one doc-test in its
  `lib.rs` module-level docs or its type-level doc-comment, except
  where the type is a marker / ZST.
- DSP math MUST be covered by `proptest` property tests (see
  `proptest = "1.8"` in `[workspace.dependencies]`).
- Benchmarks use `criterion = "0.5"` (workspace standard) and live
  in each crate's `benches/` directory.
- New real plugins MUST be added to `bundler.toml` so CI picks them up.
- New JUCE-ported crates MUST add a `README.md` with a feature table,
  a `### Added` entry in `CHANGELOG.md` under the current date, and
  flip the matching `- [ ]` to `- [x] — ✅ done (DATE)` in `TODO.md`.
- Public API breakage MUST be recorded under a new dated section in
  `CHANGELOG.md` (the project's breaking-change log).
- The CI command
  `cargo test --locked --workspace --features "simd,standalone,zstd"`
  is the source of truth for "tests pass." `cargo test --all-features`
  MUST NOT be used — `logic_nih_plug_iced` has mutually exclusive
  features.
- Rationale: testing discipline and CHANGELOG discipline are the only
  mechanisms that keep a 50-crate workspace reviewable.

---

## Governance

This constitution supersedes all other development practices in the
repository unless a section explicitly defers to a more specific
document (e.g., this constitution defers to `AGENTS.md` for exact
crate paths).

**Amendment procedure.**

1. Open a PR titled `docs: amend constitution to vX.Y.Z (summary)`.
2. The PR description MUST list every principle or section changed,
   added, or removed, with one-sentence rationale each.
3. Reviewers MUST verify that dependent templates
   (`.specify/templates/*.md`) and runtime guidance
   (`AGENTS.md`, `README.md`, `docs/getting-started.md`) remain
   consistent. Any inconsistency is a blocker.
4. On merge, the `Sync Impact Report` at the top of
   `.specify/memory/constitution.md` MUST be updated to reflect the
   new version and the templates that were re-synced.

**Versioning policy.**

- **MAJOR** (X.0.0): removal or incompatible redefinition of an
  existing principle, or governance section rewrite.
- **MINOR** (0.X.0): a new principle, a new section, or a
  material expansion of existing guidance.
- **PATCH** (0.0.X): clarifications, wording, typo fixes,
  non-semantic refinements.

**Compliance review.**

Every PR that adds or modifies plugin code MUST, in its description,
confirm: (a) no audio-thread allocations, (b) identifier strings
unchanged or explicitly versioned, (c) tests added for new public
types, (d) CHANGELOG entry under today's date if behavior or API
changed. Reviewers MUST reject PRs missing any of these.

**Runtime guidance.**

`AGENTS.md` is the primary runtime guidance for AI coding agents and
contributors — it encodes Principle I, II, IV and the
Audio-Thread Discipline section as the project's day-to-day hard
rules. This constitution is the higher-level governance; when the two
disagree, the constitution wins, and the next amendment updates
`AGENTS.md` to match.

**Version**: 1.0.0 | **Ratified**: 2026-06-19 | **Last Amended**: 2026-06-19