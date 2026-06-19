# Implementation Plan: [FEATURE]

**Branch**: `[###-feature-name]` | **Date**: [DATE] | **Spec**: [link]

**Input**: Feature specification from `/specs/[###-feature-name]/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

[Extract from feature spec: primary requirement + technical approach from research]

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: [e.g., Python 3.11, Swift 5.9, Rust 1.75 or NEEDS CLARIFICATION]

**Primary Dependencies**: [e.g., FastAPI, UIKit, LLVM or NEEDS CLARIFICATION]

**Storage**: [if applicable, e.g., PostgreSQL, CoreData, files or N/A]

**Testing**: [e.g., pytest, XCTest, cargo test or NEEDS CLARIFICATION]

**Target Platform**: [e.g., Linux server, iOS 15+, WASM or NEEDS CLARIFICATION]

**Project Type**: [e.g., library/cli/web-service/mobile-app/compiler/desktop-app or NEEDS CLARIFICATION]

**Performance Goals**: [domain-specific, e.g., 1000 req/s, 10k lines/sec, 60 fps or NEEDS CLARIFICATION]

**Constraints**: [domain-specific, e.g., <200ms p95, <100MB memory, offline-capable or NEEDS CLARIFICATION]

**Scale/Scope**: [domain-specific, e.g., 10k users, 1M LOC, 50 screens or NEEDS CLARIFICATION]

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

The plan MUST demonstrate compliance with each gate below. If any gate
fails, the failure MUST be either (a) resolved before plan approval, or
(b) explicitly justified in the **Complexity Tracking** table.

| Gate | Source | Verification |
|---|---|---|
| **G1. Real-time safety** | Constitution §I; AGENTS.md §5.1 | No new code path is reachable from `Plugin::process()` that allocates, locks (blocking), or syscalls. `assert_process_allocs` CI gate still passes. |
| **G2. Identifier stability** | Constitution §II | New `#[id = "…"]`, `#[persist = "…"]`, `#[nested(group = "…")]`, `VST3_CLASS_ID`, `CLAP_ID` strings are new additions, not renames of existing identifiers. |
| **G3. Smallest correct change** | Constitution §III | New functionality lands in an existing or new sub-crate, not by extending a module past its scope. No speculative abstraction introduced. Any new external dependency justified against stdlib / in-tree alternatives in the PR description. |
| **G4. Multi-format / cross-platform** | Constitution §IV | Plugin compiles with workspace default features (VST3 + CLAP). Other formats opt-in. No VST2 + AAX collision. New `#[cfg(target_os = "…")]` paths are exercised on a matching CI runner. |
| **G5. JUCE port fidelity** | Constitution §V | Sub-crate ports of JUCE modules preserve the upstream JUCE spelling and semantics at the public-API boundary. At least one named "matches JUCE behavior" regression is added for non-trivial public types. |
| **G6. Audio-thread discipline** | Constitution, "Audio-Thread Discipline" section | Cross-thread state uses `Arc<Atomic*>`, `parking_lot::Mutex` + `try_lock`, or `crossbeam_channel`. No `std::sync::Mutex::lock` / `RwLock` in audio-reachable code. Logging via `nih_log!`/`nih_dbg!`. |
| **G7. Tests & docs** | Constitution, "Testing, Documentation & Public-API Standards" | New public types have at least one unit test and one doc-test. DSP math has `proptest` coverage. CHANGELOG entry under today's date for any behavior / API change. `bundler.toml` updated for new real plugins. |

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)
<!--
  ACTION REQUIRED: Replace the placeholder tree below with the concrete layout
  for this feature. Delete unused options and expand the chosen structure with
  real paths (e.g., apps/admin, packages/something). The delivered plan must
  not include Option labels.
-->

```text
# [REMOVE IF UNUSED] Option 1: Single project (DEFAULT)
src/
├── models/
├── services/
├── cli/
└── lib/

tests/
├── contract/
├── integration/
└── unit/

# [REMOVE IF UNUSED] Option 2: Web application (when "frontend" + "backend" detected)
backend/
├── src/
│   ├── models/
│   ├── services/
│   └── api/
└── tests/

frontend/
├── src/
│   ├── components/
│   ├── pages/
│   └── services/
└── tests/

# [REMOVE IF UNUSED] Option 3: Mobile + API (when "iOS/Android" detected)
api/
└── [same as backend above]

ios/ or android/
└── [platform-specific structure: feature modules, UI flows, platform tests]
```

**Structure Decision**: [Document the selected structure and reference the real
directories captured above]

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |
