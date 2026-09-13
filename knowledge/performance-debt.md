# Performance technical debt

Audience: AI agents and Epok engine developers. This is an implementation
backlog and acceptance handoff, not an engine user guide. Keep this material in
`knowledge/`; `docs/` describes how game developers use the engine.

## Current status

Geometry Streaming and Precomputed Visibility are implemented, configurable,
experimental and disabled by default. The v19 production stationary comparison
passes all six gates; 66 native geometry phases preserve exact VRAM. These
results do not complete the strict no-regression requirement.

Use the [current production report](performance-validation.md) and
[matched-state replay report](performance-deterministic-validation.md) as the
sources of measured results. Preserve earlier failed captures. Do not describe
smaller executables, matching images or global averages as proof that every
activation path preserves FPS.

## Implementation order

| Priority | Open work | Completion evidence |
| --- | --- | --- |
| 1 | Resolve streaming activation regressions in the deterministic Forest replay | Streaming versus default must pass the cliff phase: its current median interval rises from 24544 to 24576 microseconds. Both versus visibility must pass idle: p95/max rise by 64 microseconds and CPU median by 0.5 tick. Retain every existing gate and exact state/geometry/VRAM checks. |
| 2 | Improve demand loading when the active working set exceeds the pool | Matched cold/warm traversal, direction changes and eviction/reload must show no extra dropped work or timing regression for the target workload. Record loading and gameplay stalls separately. |
| 3 | Retain useful geometry when a whole object exceeds the record pool | Replace or improve whole-object fallback with bounded allocation that preserves packet parity, material/fog invalidation, ownership and exact rendering. Verify native CPU cost and RAM use. |
| 4 | Improve geometry/XA coexistence | Required reads currently restart XA at the track beginning. Any resume or packing change needs real CD ownership, stop/replacement, scene lifecycle and audio validation. |
| 5 | Validate broader workloads and physical hardware | Cover rotation, shared geometry, scene changes, larger archives and representative consoles. Emulator results must remain identified as emulator measurements. |

Host timing also showed overhead during continuous rotation; it is diagnostic
evidence, not PSX FPS. Keep visibility optional while investigating workloads
where its cache and lookup costs do not pay for themselves.

## Parallel work boundaries

CPU/render investigation, bounded retention design and CD/XA investigation can
proceed independently after agreeing on shared runtime ownership. Coordinate
edits to `runtime/main.cpp`, geometry bindings and telemetry layouts. Serialize
native emulator runs; never rebuild an editor executable while a capture uses
it. Run combined feature comparisons after integration, because isolated gains
do not establish combined activation acceptance.

## Contracts and reproduction

- [Performance implementation](performance-implementation.md): visibility
  caches, retained geometry and strict comparison rules.
- [Streaming implementation](streaming-implementation.md): page ownership,
  immutable descriptors, binding, BSS, warmup, failures and XA limitations.
- [Diagnostic workflows](performance-testing.md): native fixtures and isolated
  deterministic replay. Its manual schedule is not ordinary gameplay FPS.
- [Performance guide](performance-guide.md): historical decisions, original
  baseline provenance and earlier failed approaches.

Production marks the descriptive streamed-chunk count as not collected;
detailed builds collect it. Safety counters stay active in both modes. Compare
matching instrumentation and preserve the production unavailable sentinel.

Do not enable experimental options by default or mark this debt complete until
the corresponding acceptance evidence passes. Keep user-facing warnings in
Project Settings and usage documentation accurate; keep investigations, task
plans, implementation contracts and benchmark reports here.
