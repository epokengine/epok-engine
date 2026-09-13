# Deterministic ForestTest validation: v19

Visibility passes every measured phase against the current disabled configuration. Streaming improves the global measurements but still fails one phase; adding streaming to visibility also fails one phase. These failures remain acceptance failures. Geometry Streaming and Precomputed Visibility remain experimental and disabled by default.

This is a diagnostic replay in an isolated ForestTest copy, not a normal gameplay FPS measurement. The engine clock remains paused while the copied controller explicitly performs a fixed sequence of one or two updates per rendered frame, including sprite advancement. The original controller movement, collision and camera logic runs unchanged inside that schedule. Each configuration records 196 rendered frames and 291 manual ticks across eight phases. The original project is untouched. Natural gameplay and physical-console validation remain separate requirements.

All four runs use the same release editor, instrumentation, 640×480 display, retained geometry, four page slots, 4096-triangle budget and enabled prefetch setting. Detailed profiling and GTE validation are disabled in every timing run. The two feature flags are the only manifest differences. Every comparison requires identical per-frame player/camera/sprite state, scheduled steps, visible chunks, transformed vertices and submitted triangles, plus byte-identical terminal VRAM. These checks pass for every v19 pair reported below; terminal VRAM SHA-256 is `dcde2455e6f228c021a075e99580c1daeb98c5cae882acd688bbeb2afc2919e8`.

| Configuration | CPU ticks median / p95 / max | Frame interval µs median / p95 / max | Total CPU delta vs default | Total interval delta vs default |
| --- | ---: | ---: | ---: | ---: |
| default | 378 / 401 / 402 | 24576 / 26048 / 26112 | 0 | 0 |
| streaming | 377 / 400 / 401 | 24576 / 25984 / 26048 | −142 | −9472 µs |
| visibility | 369 / 391 / 392 | 24000 / 25408 / 25472 | −2060 | −131840 µs |
| both | 368 / 391 / 392 | 23936 / 25408 / 25472 | −2091 | −135040 µs |

CPU ticks are approximately 64 µs. Global CPU medians for one/two manual steps are respectively 375/398 (default), 374/397 (streaming), 366/388 (visibility), and 366/388 (both). The comparator preserves its existing equal-step CPU gates, global timing tails and safety gates. Phase groups too small for equal-step comparison retain the aggregate CPU median gate. No thresholds, tolerances or failed phases were removed.

| Comparison | Global gates | All phase gates | Exact failing metrics: baseline → candidate |
| --- | --- | --- | --- |
| default → streaming | PASS | **FAIL** | `06-cliff-approach` (74 frames): interval median 24544 → 24576 µs (**+32 µs**) |
| default → visibility | PASS | PASS | None |
| visibility → both | PASS | **FAIL** | `01-idle` (8 frames): interval p95 and max 23744 → 23808 µs (**+64 µs** each); CPU median 359 → 359.5 ticks (**+0.5**) |
| default → both | PASS | PASS | None |

For the failed streaming cliff phase, CPU p95/max improve from 401 to 399 ticks and interval p95 improves from 26048 to 25920 µs. These improvements do not cancel the median failure. Both versus visibility totals improve by 31 CPU ticks and 3200 µs, but its idle failures remain. Timer quantization is a possible explanation for small deltas, not established evidence that permits ignoring them. No finer-timer substitution was implemented.

Both streamed runs read two pages (131072 bytes) during explicit startup warmup. Streaming alone records 1056832 µs of CD wait; both records 1056896 µs. Each has two reads/two stalls, zero errors/timeouts, and no further reads during replay. Loading time is separate from steady rendering and has not been hidden by resetting the gameplay clock. This small working set fits the pool; it does not demonstrate stall-free demand streaming, XA coexistence or oversized retained geometry.

The descriptive `streamed_chunks` counter is collected only with `EPOK_PROFILE_DETAIL`. Production uses the existing field's `UINT32_MAX` unavailable sentinel; profiler JSON reports `null` with `streamed_chunks_available=false`. Safety counters and CD reads/stalls/errors remain active. This replay does not capture `streamed_chunks` and makes no claim about its value. Its dropped steps/triangles are zero. GTE validation is disabled here and must be checked in separate diagnostic runs.

The A/A protocol check compares `deterministic16-default` with `deterministic16-default-repeat`: all 196 per-frame CPU deltas are exactly zero, and state, geometry, VRAM, global gates and all phase gates pass. It uses the same 196-row instrumentation as v19, but an earlier editor binary; it is evidence for protocol repeatability, not a repeated v19 measurement.

Evidence root: `artifacts/streaming-review/1788837596791027600/`.

- v19 profiles and full provenance: [default](../artifacts/streaming-review/1788837596791027600/deterministic19-default/report.json), [streaming](../artifacts/streaming-review/1788837596791027600/deterministic19-streaming/report.json), [visibility](../artifacts/streaming-review/1788837596791027600/deterministic19-visibility/report.json), [both](../artifacts/streaming-review/1788837596791027600/deterministic19-both/report.json). Each directory contains `profile.json`, eight phase profiles (`01-idle.json` through `08-idle-back.json`), `frames.bin`, `ram.bin`, `vram.bin`, `screen.png`, `patch.diff`, the original controller and the isolated project/build logs. `report.json.passed` describes capture correctness; comparison acceptance is recorded separately.
- Strict comparisons: [streaming vs default](../artifacts/streaming-review/1788837596791027600/deterministic19-streaming/comparison.json), [visibility vs default](../artifacts/streaming-review/1788837596791027600/deterministic19-visibility/comparison.json), [both vs visibility](../artifacts/streaming-review/1788837596791027600/deterministic19-both/comparison.json), [both vs default](../artifacts/streaming-review/1788837596791027600/deterministic19-both/comparison-default.json).
- A/A evidence: [baseline](../artifacts/streaming-review/1788837596791027600/deterministic16-default/profile.json), [repeat comparison](../artifacts/streaming-review/1788837596791027600/deterministic16-default-repeat/comparison.json), [repeat provenance](../artifacts/streaming-review/1788837596791027600/deterministic16-default-repeat/report.json).
- Separate final production stationary results: [acceptance JSON](../artifacts/streaming-review/1788837596791027600/recovered19-acceptance.json), [validation report](performance-validation.md). All six stationary comparisons pass; that result does not supersede the replay phase failures.

SHA-256 provenance shared by all four v19 runs:

- Release editor: `8fba35afa9c090c62b8504ab0e7a7697d4c263fde7c9977b1a75064ce170afb3`.
- Original controller: `e651ab955a92ce0b0239c21b7696fdfcaa8cb9ebb20ba10dd2ffa6fa53e87494`.
- Instrumented controller: `2eb68c3bd8707e70b14d88ffcc23a17df945c287a560aff234a2ccff65b64e1f`.

Strict no-regression acceptance remains incomplete. The current evidence supports improvements in these measured workloads and correct matching output, not a guarantee that activating either feature never reduces FPS on every scene or platform.
