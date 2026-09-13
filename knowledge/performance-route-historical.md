> Historical measurement snapshot, preserved from the earlier route-final run.
> These figures are not the final acceptance result for the latest source.
> Raw evidence remains in `artifacts/streaming-review/1788837596791027600/route-final/`;
> see [the performance guide](performance-guide.md) for context and current limitations.

# Forest moving-route comparison

Both native gameplay runs passed. All eight phase checks and the full-route timing check passed for median, p95 and worst sampled CPU/frame time, with no dropped simulation steps or triangles.

The phases use the same commands and target ticks, but polling overshoot changes actual tick counts and positions. These are comparable route phases, not an exact frame-for-frame replay. Short idle phases contain only 5–9 samples. No byte-identical screenshot claim is made.

| Phase | Frames baseline → final | Ticks baseline → final | Median CPU ticks | Median frame µs | Native FPS |
|---|---:|---:|---:|---:|---:|
| 01-idle | 8 → 7 | 13 → 14 | 436 → 369 | 28288 → 24000 | 35.35 → 41.67 |
| 02-walk-right | 20 → 23 | 43 → 42 | 453 → 441 | 29376 → 28608 | 34.04 → 34.96 |
| 03-idle-right | 5 → 8 | 14 → 15 | 443 → 434 | 28736 → 28160 | 34.80 → 35.51 |
| 04-run-left | 23 → 22 | 41 → 40 | 453 → 442.5 | 29440 → 28768 | 33.97 → 34.76 |
| 05-run-diagonal | 16 → 18 | 31 → 32 | 455 → 446 | 29504 → 28928 | 33.89 → 34.57 |
| 06-cliff-approach | 60 → 57 | 110 → 114 | 448 → 439 | 29088 → 28224 | 34.38 → 35.43 |
| 07-cliff-collision | 20 → 18 | 37 → 38 | 439 → 431 | 28480 → 28032 | 35.11 → 35.67 |
| 08-idle-back | 8 → 9 | 13 → 13 | 431 → 391 | 27968 → 25408 | 35.76 → 39.36 |

| Phase | Median triangles | GTE vertices | Tested chunks | Visible chunks | Retained triangles |
|---|---:|---:|---:|---:|---:|
| 01-idle | 607.0 → 607 | 1092.0 → 1092 | 168.0 → 149 | 72.0 → 72 | 607.0 → 607 |
| 02-walk-right | 609.5 → 613 | 1092.0 → 1092 | 168.0 → 149 | 72.0 → 72 | 609.5 → 613 |
| 03-idle-right | 621 → 621.0 | 1084 → 1092.0 | 168 → 156.0 | 71 → 72.0 | 621 → 621.0 |
| 04-run-left | 620 → 620.0 | 1092 → 1092.0 | 168 → 156.0 | 72 → 72.0 | 620 → 620.0 |
| 05-run-diagonal | 608.5 → 609.0 | 1110.0 → 1110.0 | 168.0 → 149.0 | 72.0 → 72.0 | 608.5 → 609.0 |
| 06-cliff-approach | 606.0 → 606 | 1091.0 → 1091 | 168.0 → 149 | 73.0 → 73 | 606.0 → 606 |
| 07-cliff-collision | 582.0 → 585.0 | 1080.0 → 1080.0 | 168.0 → 149.0 | 72.0 → 72.0 | 582.0 → 585.0 |
| 08-idle-back | 580.0 → 585 | 1080.0 → 1080 | 168.0 → 149 | 72.0 → 72 | 580.0 → 585 |

Startup CD loading is reported separately; its counters are cumulative and remain included in total streaming waits.

```json
{
  "streaming": {
    "available": true,
    "first": {
      "reads": 2,
      "bytes": 131072,
      "stalls": 2,
      "stall_us": 1056960,
      "errors": 0,
      "timeouts": 0,
      "xa_interruptions": 0
    },
    "last": {
      "reads": 2,
      "bytes": 131072,
      "stalls": 2,
      "stall_us": 1056960,
      "errors": 0,
      "timeouts": 0,
      "xa_interruptions": 0
    },
    "during_capture": {
      "reads": 0,
      "bytes": 0,
      "stalls": 0,
      "stall_us": 0,
      "errors": 0,
      "timeouts": 0,
      "xa_interruptions": 0
    }
  },
  "streaming_warmup": {
    "available": true,
    "first": {
      "attempts": 1,
      "pages": 2,
      "reads": 2,
      "stall_us": 1056960,
      "rejected": 0,
      "failures": 0
    },
    "last": {
      "attempts": 1,
      "pages": 2,
      "reads": 2,
      "stall_us": 1056960,
      "rejected": 0,
      "failures": 0
    },
    "during_capture": {
      "attempts": 0,
      "pages": 0,
      "reads": 0,
      "stall_us": 0,
      "rejected": 0,
      "failures": 0
    }
  }
}
```
