# AI working context

Write all Markdown in English. Keep internal context, investigation notes and
handoffs and technical debt under `knowledge/`. Do not put agent handoffs,
acceptance investigations, unresolved regression analysis or implementation
backlogs in `docs/`. Public documentation may remain under `docs/`, and
README files may remain beside the components they describe.

`docs/` is for people using Epok to make games. `knowledge/` is for AI agents
and developers maintaining Epok itself. User-facing option descriptions and
observable behavior belong in `docs/`; implementation contracts, diagnostic
methodology, technical debt and continuation plans belong in `knowledge/`.

## Maintainer workflows

- [Local testing](maintainers/testing.md) covers editor, exporter and runtime checks.

## Initiatives

- [Actor architecture](initiatives/actor-architecture/README.md) records the
  Object/Actor/Component model: the binding design contract, the P0–P10 phase
  reports, the closing delivery and test matrix, and the inventory of legacy
  compatibility adapters with their removal conditions.

## Performance work

- [Performance guide](performance-guide.md) records the existing optimization
  history, the experimental streaming implementation and its acceptance criteria.
- [ForestTest validation](performance-validation.md) records the current native
  measurements and links to the detailed acceptance evidence.
- [Deterministic moving validation](performance-deterministic-validation.md) records matched-state v19 results and the remaining strict phase failures.
- [Moving-route evidence](performance-route-validation.md) preserves the v12 comparison with the original engine; current-default activation failures are recorded in the current validation report.
- [Technical debt](performance-debt.md), [implementation contracts](performance-implementation.md),
  [streaming internals](streaming-implementation.md) and [validation workflows](performance-testing.md)
  provide the continuation context for future AI agents.
- [Runtime performance](../docs/performance.md), [streaming usage](../docs/streaming.md)
  and [project settings](../docs/settings.md) describe supported behavior.
- Geometry Streaming and Precomputed Visibility remain experimental and disabled
  by default. A smaller executable or matching screenshot is not evidence of an
  FPS improvement. Check activation against the current disabled configuration,
  as well as against the original baseline.
- Compare median, p95 and maximum native frame time, CPU work grouped by equal
  fixed simulation step counts, dropped simulation steps and dropped triangles.
  Stationary captures also require exact VRAM equality. Moving-route captures
  record polling overshoot and are not identical frame-by-frame replays.
  If all step groups have sufficient coverage, the global CPU median is
  informational and the group medians are gated; global CPU tails and frame
  times remain strict. This prevents a changed step-count mixture from being
  mistaken for additional CPU work.
- Use `tools/profile_runtime.py`, `tools/compare_runtime.py` and
  `tools/profile_forest_route.py`. Rebuild the editor after changing embedded
  runtime sources. Serialize emulator runs and do not rebuild an editor binary
  while a test is using it.

## Local evidence and provenance

The September 2026 ForestTest investigation uses isolated copies below
`artifacts/streaming-review/1788837596791027600/`. Its original baseline was
rebuilt from commit `8fe73066bace7a895d2258a31067ab68d7718a79`, with only the three
pre-existing environment-effect changes restored. See `baseline-provenance.json`
there for hashes and `baseline-repeat-2/profile.json` for the repeated capture.
Artifacts are local evidence, not necessarily present in a fresh checkout.

Preserve the user's pre-existing changes in `runtime/main.cpp`,
`runtime/effects.hpp` and `docs/environment-effects.md`. Use Git history and status to determine the current publication state. The
historical guide's earlier publication statements describe their own snapshots.

## Descriptive streaming diagnostics

The per-chunk streamed_chunks count is collected only with EPOK_PROFILE_DETAIL
(for example, tools/profile_runtime.py --detail). Production profiles expose it
as null with streamed_chunks_available=false; the native field uses UINT32_MAX
without changing the PerformanceStats layout. Read, stall, error, timeout, failed
chunk and dropped-step/triangle counters remain active. Compare timing only
between builds using the same instrumentation. Native counter assertions use
detailed builds; production FPS acceptance uses detail disabled on both sides.
