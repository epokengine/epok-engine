# Historical v12 Forest moving-route validation

This report compares v12 with the original baseline only. It is not activation acceptance against current defaults. The subsequent current-default comparison failed during movement; see [current validation](performance-validation.md).

The final runtime v12 capture passes the full-route timing gate and all eight phase gates against `route-baseline-2`. Both native gameplay checks pass. All comparable CPU groups pass; there are no sampled dropped simulation steps, dropped triangles, GTE validation errors, or failed streamed chunks. This establishes the result for these captures, not a guarantee for every camera path or physical console.

Evidence: [complete comparison JSON](../artifacts/streaming-review/1788837596791027600/route-accepted/route-comparison.json), [baseline route](../artifacts/streaming-review/1788837596791027600/route-baseline-2/route.json), [final route](../artifacts/streaming-review/1788837596791027600/route-accepted/route.json). Earlier route reports remain historical and unchanged.

The candidate enables retained geometry, precomputed visibility, geometry streaming, and prefetch at 640 by 480, with four pool pages and a 4096-triangle frame budget. Detail timers and GTE validation are disabled in both timing builds. Zero GTE error counters here do not replace a validation-enabled correctness run.

## Timing and FPS

The complete route contains 160 baseline and 178 candidate sampled frames. CPU timer counts are the recorded frame_scanlines units. FPS below is 1,000,000 divided by the median or mean sampled frame interval; the latter is not an arithmetic average of instantaneous FPS and is not contiguous whole-run throughput.

| Phase | Samples baseline -> final | CPU median | CPU p95 | CPU max | Frame median us | Frame p95 us | Frame max us | Gate |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| all-route | 160 -> 178 | 441.5 -> 422 | 455 -> 438 | 456 -> 440 | 28672 -> 27392 | 29504 -> 28416 | 29568 -> 28544 | PASS |
| 01-idle | 8 -> 9 | 436 -> 389 | 436 -> 390 | 436 -> 390 | 28288 -> 25344 | 28352 -> 25344 | 28352 -> 25344 | PASS |
| 02-walk-right | 20 -> 24 | 453 -> 432 | 455 -> 436 | 455 -> 436 | 29376 -> 28064 | 29504 -> 28288 | 29504 -> 28352 | PASS |
| 03-idle-right | 5 -> 8 | 443 -> 425.5 | 456 -> 426 | 456 -> 426 | 28736 -> 26816 | 29568 -> 27712 | 29568 -> 27712 | PASS |
| 04-run-left | 23 -> 24 | 453 -> 435 | 455 -> 436 | 455 -> 436 | 29440 -> 28224 | 29504 -> 28352 | 29568 -> 28352 | PASS |
| 05-run-diagonal | 16 -> 19 | 455 -> 437 | 455 -> 440 | 455 -> 440 | 29504 -> 28416 | 29568 -> 28544 | 29568 -> 28544 | PASS |
| 06-cliff-approach | 60 -> 62 | 448 -> 427 | 455 -> 437 | 455 -> 438 | 29088 -> 27712 | 29504 -> 28416 | 29568 -> 28416 | PASS |
| 07-cliff-collision | 20 -> 23 | 439 -> 389 | 440 -> 423 | 440 -> 423 | 28480 -> 25344 | 28544 -> 27456 | 28608 -> 27456 | PASS |
| 08-idle-back | 8 -> 9 | 431 -> 354 | 439 -> 385 | 439 -> 385 | 27968 -> 23104 | 28480 -> 25088 | 28480 -> 25088 | PASS |

| Phase | FPS at median interval | Mean interval us | FPS at mean interval |
|---|---:|---:|---:|
| all-route | 34.88 -> 36.51 | 28425.20 -> 26786.16 | 35.18 -> 37.33 |
| 01-idle | 35.35 -> 39.46 | 27616.00 -> 24519.11 | 36.21 -> 40.78 |
| 02-walk-right | 34.04 -> 35.63 | 28774.40 -> 27109.33 | 34.75 -> 36.89 |
| 03-idle-right | 34.80 -> 37.29 | 28172.80 -> 26696.00 | 35.50 -> 37.46 |
| 04-run-left | 33.97 -> 35.43 | 28827.83 -> 27469.33 | 34.69 -> 36.40 |
| 05-run-diagonal | 33.89 -> 35.19 | 28760.00 -> 27712.00 | 34.77 -> 36.09 |
| 06-cliff-approach | 34.38 -> 36.09 | 28484.27 -> 27162.84 | 35.11 -> 36.82 |
| 07-cliff-collision | 35.11 -> 39.46 | 27968.00 -> 26006.26 | 35.76 -> 38.45 |
| 08-idle-back | 35.76 -> 43.28 | 27392.00 -> 23893.33 | 36.51 -> 41.85 |

For completeness, the arithmetic mean of sampled instantaneous FPS is 35.24 -> 37.46. This descriptive metric is not a separate acceptance gate.

## CPU at equal simulation-step counts

The comparator requires at least five samples in both captures for each step-count group. It reports sparse groups without accepting or rejecting their CPU distributions. Only when every observed step count is covered does it treat the aggregate CPU median as informative; its report includes non_gating_reason. Group medians/p95/max, global CPU p95/max, all frame-time statistics, and safety checks remain strict, with no tolerance.

| Phase | Steps | Samples baseline -> final | CPU median | CPU p95 | CPU max | Status |
|---|---:|---:|---:|---:|---:|---|
| all-route | 1 | 47 -> 70 | 416 -> 396 | 419 -> 401 | 420 -> 404 | PASS |
| all-route | 2 | 113 -> 108 | 452 -> 434 | 455 -> 439 | 456 -> 440 | PASS |
| 01-idle | 1 | 3 -> 4 | Not compared | Not compared | Not compared | Insufficient samples |
| 01-idle | 2 | 5 -> 5 | 436 -> 390 | 436 -> 390 | 436 -> 390 | PASS |
| 02-walk-right | 1 | 5 -> 10 | 417 -> 396 | 419 -> 400 | 419 -> 400 | PASS |
| 02-walk-right | 2 | 15 -> 14 | 453 -> 434.5 | 455 -> 436 | 455 -> 436 | PASS |
| 03-idle-right | 1 | 2 -> 3 | Not compared | Not compared | Not compared | Insufficient samples |
| 03-idle-right | 2 | 3 -> 5 | Not compared | Not compared | Not compared | Insufficient samples |
| 04-run-left | 1 | 6 -> 8 | 418 -> 399 | 420 -> 400 | 420 -> 400 | PASS |
| 04-run-left | 2 | 17 -> 16 | 454 -> 436 | 455 -> 436 | 455 -> 436 | PASS |
| 05-run-diagonal | 1 | 5 -> 6 | 418 -> 401.5 | 419 -> 404 | 419 -> 404 | PASS |
| 05-run-diagonal | 2 | 11 -> 13 | 455 -> 439 | 455 -> 440 | 455 -> 440 | PASS |
| 06-cliff-approach | 1 | 18 -> 24 | 415 -> 398 | 420 -> 401 | 420 -> 401 | PASS |
| 06-cliff-approach | 2 | 42 -> 38 | 452 -> 434.5 | 455 -> 438 | 455 -> 438 | PASS |
| 07-cliff-collision | 1 | 5 -> 10 | 406 -> 389 | 406 -> 390 | 406 -> 390 | PASS |
| 07-cliff-collision | 2 | 15 -> 13 | 439 -> 422 | 440 -> 423 | 440 -> 423 | PASS |
| 08-idle-back | 1 | 3 -> 5 | Not compared | Not compared | Not compared | Insufficient samples |
| 08-idle-back | 2 | 5 -> 4 | Not compared | Not compared | Not compared | Insufficient samples |

Short idle phases have only five to nine total samples. Phase 01 covers only its two-step group; phases 03 and 08 cover neither group. Their global timing gates still pass, but they provide weaker conditional CPU evidence than the moving phases and complete route.

## Workload and trajectory

| Phase | Median triangles | Median GTE vertices | Median tested chunks | Median visible chunks | Ticks baseline -> final |
|---|---:|---:|---:|---:|---:|
| all-route | 607.5 -> 607 | 1092 -> 1092 | 168 -> 149 | 72 -> 72 | All phases |
| 01-idle | 607 -> 607 | 1092 -> 1092 | 168 -> 149 | 72 -> 72 | 13 -> 14 |
| 02-walk-right | 609.5 -> 612.5 | 1092 -> 1092 | 168 -> 149 | 72 -> 72 | 43 -> 42 |
| 03-idle-right | 621 -> 621.5 | 1084 -> 1092 | 168 -> 156 | 71 -> 72 | 14 -> 15 |
| 04-run-left | 620 -> 619 | 1092 -> 1092 | 168 -> 156 | 72 -> 72 | 41 -> 43 |
| 05-run-diagonal | 608.5 -> 606 | 1110 -> 1118 | 168 -> 149 | 72 -> 73 | 31 -> 30 |
| 06-cliff-approach | 606 -> 604 | 1091 -> 1106 | 168 -> 149 | 73 -> 74 | 110 -> 110 |
| 07-cliff-collision | 582 -> 582 | 1080 -> 1080 | 168 -> 149 | 72 -> 72 | 37 -> 37 |
| 08-idle-back | 580 -> 583 | 1080 -> 1080 | 168 -> 149 | 72 -> 72 | 13 -> 14 |

Commands and target ticks match, but HTTP polling and pause acknowledgement can overshoot. Actual tick counts, phase-boundary positions, and visible work differ. The baseline ends the cliff approach at x=0.30712890625, z=7.219482421875; the candidate ends at x=0.12841796875, z=6.010009765625. Both remain at their respective positions during the cliff-collision phase and pass the gameplay checks. Different collision positions on different trajectories do not establish a physics regression; these captures also cannot prove identical collision replay. Late-route performance improvements therefore cannot be attributed solely to renderer changes.

Completed-frame performance counters and live probe/clock values may refer to adjacent frames. Candidate phase 05 ends at x=0.079345703125, z=1.47802734375, while phase 06 starts at x=0.12841796875, z=1.527099609375; the recorded boundary difference is retained. No byte-identical VRAM claim is made or used as a gate for these nonidentical routes.

## Streaming and safety

The final capture begins with two completed page reads totaling 131072 bytes and two demand waits totaling 1056832 us (about 1.057 seconds). Warmup reports one attempt, two pages, no rejected set and no failure. These startup costs remain in the cumulative counters; they are outside the warm route timing samples. Across the route, additional reads, bytes, stalls, wait time, errors, timeouts and XA interruptions are all zero. The full two-page archive fits the four-page pool, so this route does not measure page eviction or demand streaming under pool pressure.

Both runs report zero dropped simulation steps and zero maximum dropped triangles. Candidate streamed failures, stream errors and timeouts are zero. Gameplay run/walk and diagonal ratios are preserved in the comparison JSON. Existing deterministic visual and demand-streaming fixture validations are separate evidence; this route does not replace them.
