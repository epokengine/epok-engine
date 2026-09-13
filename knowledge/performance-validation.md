# ForestTest performance validation

These are native PCSX-Redux captures of an isolated ForestTest copy, with identical stationary VRAM.
Geometry Streaming and Precomputed Visibility remain experimental and disabled by default.

| Configuration | Samples | Median CPU ticks (1 / 2 steps) | Median frame µs | p95 / maximum frame µs | FPS from median interval | FPS from mean interval |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| original | 105 | 408 / 436 | 28288 | 28352 / 28352 | 35.351 | 36.227 |
| default | 100 | 382 / 410 | 26624 | 26688 / 26688 | 37.560 | 38.681 |
| streaming | 95 | 381 / 409 | 26560 | 26624 / 26624 | 37.651 | 38.711 |
| visibility | 98 | 360 / 388 | 23488 | 25280 / 25280 | 42.575 | 41.352 |
| both | 94 | 360 / 388 | 23488 | 25280 / 25280 | 42.575 | 41.214 |

The two FPS columns use different summaries of sampled intervals; neither is a physical-console guarantee.
The mixed one-step/two-step frame population can shift the global median. CPU comparisons also hold the step count equal.

## Stationary acceptance checks

- `original_to_default`: **PASS**.
- `original_to_streaming`: **PASS**.
- `original_to_visibility`: **PASS**.
- `original_to_both`: **PASS**.
- `default_to_streaming`: **PASS**.
- `visibility_to_both`: **PASS**.

The gate checks frame median, p95 and maximum, CPU work including equal-step groups, dropped steps/triangles and exact VRAM.
A failed activation check must remain visible even when the candidate beats the original baseline.

## Evidence

- [Detailed acceptance JSON](../artifacts/streaming-review/1788837596791027600/recovered19-acceptance.json).
- [Baseline provenance](../artifacts/streaming-review/1788837596791027600/baseline-provenance.json).
- [original profile](../artifacts/streaming-review/1788837596791027600/baseline-repeat-2/profile.json).
- [default profile](../artifacts/streaming-review/1788837596791027600/recovered19-default/profile.json).
- [streaming profile](../artifacts/streaming-review/1788837596791027600/recovered19-streaming/profile.json).
- [visibility profile](../artifacts/streaming-review/1788837596791027600/recovered19-visibility/profile.json).
- [both profile](../artifacts/streaming-review/1788837596791027600/recovered19-both/profile.json).

Startup loading is recorded in cumulative streaming counters and separately identified by warmup counters.
It happens before the simulation clock starts; later stalls are not removed by resetting that clock.
Demand reads in larger working sets, oversized retained objects and XA restart behavior remain experimental limitations.

## Scope and remaining acceptance failures

These v19 captures use the production runtime with detailed profiling disabled.
They run the original ForestTest controller in an isolated project copy. The
original project and its settings were not changed. All six stationary gates
pass, including activation against the current disabled configuration.

The matched-state moving benchmark is reported separately in
[Deterministic ForestTest validation](performance-deterministic-validation.md).
It manually schedules simulation inside a copied controller and is a diagnostic
workload, not a normal gameplay FPS measurement. All four modes match player,
camera, sprite, step count, submitted geometry and terminal VRAM. Visibility
passes every tested phase; streaming still fails a 32-microsecond phase median,
and adding streaming to visibility still fails small idle-phase timing gates.
Global improvements do not erase these phase failures. Strict no-regression
acceptance is therefore incomplete, and both features remain experimental.
No thresholds were relaxed to make these results pass.

The earlier natural route was not an exact replay because HTTP input polling
overshot requested ticks. Its v14 aggregate improvement and transition ambiguity
are preserved in [the historical validation](performance-validation-v12.md).
The prior prototype failures remain in their original JSON artifacts.

## Loading and diagnostics

Both streamed captures loaded two active pages (131072 bytes) during startup,
with 1056832 microseconds of cumulative CD wait before the simulation clock
started. Their full archive contains three pages and fits the four-slot pool.
There were no further reads, waits, read errors or timeouts during these captures.
Warmup is an explicit loading phase, not a hidden reset of gameplay stall time.

The descriptive streamed_chunks counter is collected only in detailed builds.
Production JSON uses null and streamed_chunks_available=false; UINT32_MAX is
the native unavailable sentinel. This preserves the telemetry layout while
removing per-chunk descriptive counting from ordinary rendering. Error, failed
chunk, read, stall, timeout and dropped-step/triangle counters remain active.
Compare timing only with matching instrumentation.

## Correctness and implementation checks

- The final [v19 native geometry report](../artifacts/streaming/1788853200130266600/report.json)
  passes all 66 phases across six modes with exact VRAM equality. Every phase
  renders 1600 triangles; retained modes also report 1600 retained triangles.
  Failed chunks, dropped triangles, read errors and timeouts are zero.
- All six native geometry modes use detail instrumentation, independently of
  production FPS captures. Successful streamed chunk counts equal visible
  chunks (9 or 10); resident modes report zero streamed chunks.
- Two-slot demand pools perform ten reads for four pages, demonstrating eviction
  and reload. The four-slot fitting pool performs exactly four reads throughout
  all phases. Complete pools occupy BSS outside the PS-X EXE load image:
  131128 bytes for demand and 262232 bytes for the fitting pool.
- All 13 host runtime programs pass, including conservative visibility, exact
  frustum bounds, page ownership, range validation, checksums, timeouts, stable
  bindings and disabled streaming paths.
- The profile tools pass 11 unit tests; the comparator passes 15 and BSS layout
  checks pass two. Debug and release editors were rebuilt with the final runtime.
- Rust validation recorded 105 passing tests and three ignored tests; Clippy
  was clean. Subsequent changes were in runtime, profiling tools and documentation.
- The latest [XA report](../artifacts/streaming-xa/1788849247095774600/report.json)
  passed all six phases: four geometry reads, three interruptions and six
  playback starts, with no read errors or timeouts. Required reads restart XA
  from the beginning of the track; uninterrupted playback is not implemented.

The user's pre-existing environment-effect implementation and its documentation
remain byte-identical to the saved pre-work snapshots. Publication history is recorded in Git. Physical-console validation remains
separate.
