# Historical v12 stationary acceptance and v13/v14 development

These are native PCSX-Redux captures of an isolated ForestTest copy, with identical stationary VRAM.
Geometry Streaming and Precomputed Visibility remain experimental and disabled by default.

| Configuration | Samples | Median CPU ticks (1 / 2 steps) | Median frame µs | p95 / maximum frame µs | FPS from median interval | FPS from mean interval |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| original | 105 | 408 / 436 | 28288 | 28352 / 28352 | 35.351 | 36.227 |
| default | 102 | 382 / 410 | 26624 | 26688 / 26688 | 37.560 | 38.728 |
| streaming | 102 | 382 / 410 | 26624 | 26688 / 26688 | 37.560 | 38.750 |
| visibility | 104 | 361 / 389 | 23552 | 25344 / 25344 | 42.459 | 41.149 |
| both | 294 | 361 / 389 | 23552 | 25344 / 25344 | 42.459 | 41.147 |

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
When all step groups have sufficient coverage, only the aggregated CPU median
is informational; its value and explanation remain in the detailed report.
Group medians and all global tail/frame-time checks remain strict.

The first `recovered12-both` attempt produced no completed frames and supplies
no FPS result. The retained repeat captured 294 frames over a 45-second run;
its longer observation window is shown explicitly rather than replacing the
failed attempt's files.

## Evidence

- [Detailed acceptance JSON](../artifacts/streaming-review/1788837596791027600/recovered12-acceptance.json).
- [Baseline provenance](../artifacts/streaming-review/1788837596791027600/baseline-provenance.json).
- [original profile](../artifacts/streaming-review/1788837596791027600/baseline-repeat-2/profile.json).
- [default profile](../artifacts/streaming-review/1788837596791027600/recovered12-default/profile.json).
- [streaming profile](../artifacts/streaming-review/1788837596791027600/recovered12-streaming/profile.json).
- [visibility profile](../artifacts/streaming-review/1788837596791027600/recovered12-visibility/profile.json).
- [both profile](../artifacts/streaming-review/1788837596791027600/recovered12-both-repeat/profile.json).

Startup loading is recorded in cumulative streaming counters and separately identified by warmup counters.
It happens before the simulation clock starts; later stalls are not removed by resetting that clock.
Demand reads in larger working sets, oversized retained objects and XA restart behavior remain experimental limitations.

## Correctness and implementation checks

- [Final native geometry report](../artifacts/streaming/1788848044942703000/report.json):
  all 66 phases passed exact VRAM comparisons across six modes. Streamed chunk
  totals equal successful visible chunks, and retained modes render 1600 retained
  triangles per phase. No read errors or dropped triangles occurred.
- Two-slot demand pools performed ten reads for four pages, proving eviction
  and reload. Four-slot pools loaded each of the four pages once. Both pool
  layouts were in BSS outside the executable's load image.
- [XA report](../artifacts/streaming-xa/1788845280917670500/report.json): six phases
  passed, with four geometry reads, three interruptions and six playback starts;
  no read errors or timeouts. Playback resumes from the beginning of a track.
- All 13 host runtime programs passed, including ownership, range validation,
  checksum failures, timeouts, cached bindings and disabled paths. The comparator
  passed 15 unit tests; profile tools passed five and BSS layout checks passed two.
- Rust validation completed with 105 passing tests and three ignored tests;
  Clippy was clean. Subsequent runtime-only changes were rebuilt for debug and
  release and exercised by the native checks above.

One initial native acceptance attempt closed PCSX-Redux with host exit code
`0xc0000005` before the first streaming frame. Its artifacts remain in
`artifacts/streaming/1788847956813261000/`; the clean repeat above completed.
The test's initialization loop now waits for its initialized probe instead of
forcing resume during emulator startup. Phase pause/resume behavior is unchanged.

## Moving activation remains under development

The stationary results above do not establish moving activation acceptance.
The v12 combined route passed against the original engine but failed against
the current default configuration by approximately 6–10 CPU ticks during movement.
The v13 adaptive policy bypasses plane lookup while a matrix changes and reduces
that difference to approximately 2–3 ticks; it still fails the strict activation gate.
Both captures and their failures are preserved. Further optimization is in progress.

- [v12 moving activation](../artifacts/streaming-review/1788837596791027600/route-accepted/activation-versus-current-default.json).
- [v13 moving activation](../artifacts/streaming-review/1788837596791027600/route-adaptive/activation-versus-current-default.json).

The v14 basis-product cache subsequently passed the full natural-route comparison:
CPU medians fell by eight ticks in both one-step and two-step groups, and median
frame time fell by 1056 microseconds. Seven of eight phase comparisons passed.
The remaining stop-transition comparison captured different simulation work
(candidate simulation79/world49/two collider rebuilds versus baseline68/41/zero);
the candidate render portion was faster (300 versus308 ticks). This is not a
matched-state per-frame acceptance result; deterministic replay is being added
to resolve the ambiguity without relaxing the comparator.

- [v14 natural-route comparison](../artifacts/streaming-review/1788837596791027600/route-basis-cache/activation-versus-current-default.json).
