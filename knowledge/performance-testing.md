# Performance regression and diagnostic workflows

Internal validation procedures for contributors continuing the performance work.
See [technical debt](performance-debt.md) for priorities and
[current measurements](performance-validation.md) for recorded results.
Run commands from the engine checkout and serialize all emulator runs.

## Streaming and visibility coverage

They also cover conservative visibility masks, streaming page ownership,
checksum failures, asynchronous reads, bounded startup warmup, warm-scene
prefetch suppression/invalidation and disabled-build overhead. Cursor cases
cover sequential views, advancement after consumption, active public borrows,
and default validation of arbitrary descriptors; retained geometry validates
its immutable chain once per allocation. The streaming
native checks use isolated projects on ports 8092/8093: the first compares exact
VRAM across six resident/visibility/streaming and retained-packet modes while
changing camera/parent transforms, evicting/reloading pages and alternating
material colors with fog across both parities. Eleven phases per mode also cover
four archive pages fitting four slots: every capture must retain exactly four
total reads, demonstrating warmup without subsequent eviction or reload and
exercising eager retained-packet rebuilds. It asserts actual retained
triangles so allocation fallback cannot make a retention test pass unnoticed,
and builds all six modes with `EPOK_PROFILE_DETAIL=1` so the descriptive
`streamed_chunks` count can be checked against visible chunks. Each mode's
report records `detail_timers: true`; these instrumented timings are not
release FPS results. Ordinary profiler captures expose this uncollected field
as JSON `null`, while read/failure/drop counters remain active. The test also
checks the MIPS map and EXE load range to ensure page buffers live in BSS
instead of being serialized as zero-filled payload. The zero-state pool remains
independent per instance; the map guard does not substitute for runtime
ownership and corruption tests. The
second tests shared CD ownership, XA interruption/restart, explicit stop and scene
reloads. Both require the configured CD authoring tools.
These are correctness checks, not FPS acceptance. Streaming, preloading and
precomputed visibility remain Experimental and may lower FPS. Capture matching
native profiles, then run
`python tools/compare_runtime.py --baseline BASE --candidate NEW --require-vram`.
The strict comparison rejects increased median/p95/worst intervals or CPU work
under the [step-group policy](performance-implementation.md#compilation-and-measurement),
dropped steps/triangles, incompatible recorded build instrumentation and
mismatched deterministic VRAM snapshots. Omit `--require-vram` only for a timing
comparison whose image correctness is checked separately; the report marks
visual validation as unchecked. Its unit tests cover tail spikes, startup loss,
incompatible flags, missing samples and image mismatches. When every sampled
step-count group has at least five samples in both captures, grouped CPU medians
replace the aggregate CPU-median gate. Global CPU p95/maximum and all frame-time
and safety gates remain strict; incomplete group coverage keeps the aggregate gate.

## Deterministic Forest replay

For a Forest project exposing the existing controller test hooks,
`tools/profile_forest_deterministic.py` creates an isolated instrumented copy.
The source project stays unchanged; the output includes the source patch,
provenance hashes, every completed frame, phase profiles and terminal VRAM.

```powershell
python tools/profile_forest_deterministic.py --project PATH_TO_FOREST --output artifacts/replay-default --editor target/release/epok-editor.exe --steps 1,2 --streaming off --visibility off
python tools/profile_forest_deterministic.py --project PATH_TO_FOREST --output artifacts/replay-streaming --editor target/release/epok-editor.exe --steps 1,2 --streaming on --visibility off --compare artifacts/replay-default
```

This is an instrumented benchmark, **not normal gameplay FPS**. The engine clock
remains paused while the original controller movement/collision update and sprite
advancement execute a deterministic tick schedule in each rendered frame. Use
`--steps 1`, `--steps 2`, or `--steps 1,2`; HTTP polling never advances the route.
Comparisons reject differing player/camera/sprite states, scheduled steps,
geometry counters or terminal VRAM, then apply strict timing gates globally and
per phase. Instrumentation adds identical work to both captures. Complement it
with `tools/profile_forest_route.py` and ordinary gameplay measurements to test
natural clock catch-up, transitions and streaming stalls. `--prepare-only` creates
the copy and patch without building or launching an emulator.
