# Skeletal character measurements

Revision `e3b56518175ddd633ccc962042a86d30d7275234` with an uncommitted working tree, 320x240, 3 runs of 7 s per workload, first 2.0 s of samples discarded.

## What these numbers are

* Emulator timing for one fixed camera and scene; not hardware timing and not host/editor FPS.
* Timing scopes overlap (render includes vertex and polygon; skeletal work happens inside them). Do not add them together.
* One sample per distinct completed frame, polled at 20 Hz; not a continuous per-frame trace.
* Scanline counters have a resolution of about 64 us per unit, so single-unit differences are at the edge of the instrument.
* One NTSC vblank is about 263 scanline units.

## Environment

| Item | Value |
| --- | --- |
| Host | Darwin 25.4.0 arm64 |
| Emulator | .tools/macos/redux/PCSX-Redux.app/Contents/MacOS/pcsx-redux |
| Editor | `/Users/francisco.rivero/GithubProjects/p/epok-engine/target/debug/epok-editor` sha256 `eab9e9aec0feced6` |
| EpokSeamAtlas.png | 149 B, sha256 `2a16fb5dc8801208` |
| EpokSeamCharacter.fbx | 16628 B, sha256 `cee331b34706a9aa` |
| EpokManySequences.fbx | 58987 B, sha256 `baedc4131e9154fc` |
| EpokMannequin.fbx | 226972 B, sha256 `01911fc635f50ed0` |

## Workloads

| Key | Workload | Storage | Instances | Clips | Camera |
| --- | --- | --- | --- | --- | --- |
| A | 1 instance, untextured, rigid | RigidGte | 1 | Clip00 | (0, 0.75, -1.8) |
| B | 1 instance, textured, rigid | RigidGte | 1 | Clip00 | (0, 0.75, -1.8) |
| C | 1 instance, textured, baked | BakedVertices | 1 | Clip00 | (0, 0.75, -1.8) |
| D | 1 instance, textured, lit materials (CPU rigid) | CpuRigid | 1 | Clip00 | (0, 0.75, -1.8) |
| I | 4 instances off-screen, rigid | RigidGte | 4 | Clip00, Clip01, Clip02, Clip00 | (0, 0.75, -1.8) |
| J | 4 instances off-screen, baked | BakedVertices | 4 | Clip00, Clip01, Clip02, Clip00 | (0, 0.75, -1.8) |
| Q1R | Rigid visible, one current-model vertex query/frame | RigidGte | 1 | Clip00 | (0, 0.75, -1.8) |
| Q4R | Rigid visible, unsorted/repeated four-vertex batch/frame | RigidGte | 1 | Clip00 | (0, 0.75, -1.8) |
| QMR | Rigid visible, 32 sparse current vertices/frame | RigidGte | 1 | Clip00 | (0, 0.75, -1.8) |
| QAB | Baked visible, all current vertices/frame | BakedVertices | 1 | Clip00 | (0, 0.75, -1.8) |
| QBB | Baked visible, sparse world bone queries/frame | BakedVertices | 1 | Clip00 | (0, 0.75, -1.8) |
| QCR | Rigid culled, explicit vertex query/frame | RigidGte | 1 | Clip00 | (0, 0.75, -1.8) |

## Frame timing (scanline units, median of all warm samples)

| Key | n | frame | p95 | max | render | vertex | polygon | skeletal | frame us | dropped steps |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| A | 122 | 56 | 56 | 56 | 31 | 11 | 12 | 5 | 16832 | 0 |
| B | 118 | 58 | 58 | 59 | 31 | 11 | 12 | 5 | 16832 | 0 |
| C | 121 | 52 | 52 | 52 | 24 | 3 | 11 | 3 | 16832 | 0 |
| D | 110 | 85 | 86 | 86 | 58 | 3 | 34 | 14 | 16832 | 0 |
| I | 110 | 47 | 47 | 47 | 9 | 0 | 0 | 0 | 16832 | 0 |
| J | 106 | 48 | 48 | 48 | 9 | 0 | 0 | 0 | 16832 | 0 |
| Q1R | 111 | 65 | 65 | 66 | 31 | 12 | 12 | 6 | 16832 | 0 |
| Q4R | 110 | 69 | 69 | 69 | 30 | 12 | 11 | 5 | 16832 | 0 |
| QMR | 109 | 1029 | 1030 | 1030 | 31 | 12 | 11 | 5 | 66304 | 0 |
| QAB | 107 | 79 | 80 | 80 | 24 | 3 | 11 | 3 | 16832 | 0 |
| QBB | 109 | 78 | 79 | 79 | 24 | 3 | 11 | 3 | 16832 | 0 |
| QCR | 110 | 36 | 36 | 37 | 2 | 0 | 0 | 0 | 16832 | 0 |

Per-run medians of `frame_scanlines`, to show run-to-run spread:

| Key | run 0 | run 1 | run 2 |
| --- | --- | --- | --- |
| A | 56 | 56 | 56 |
| B | 58 | 58 | 58 |
| C | 52 | 52 | 52 |
| D | 85 | 85 | 85 |
| I | 47 | 47 | 47 |
| J | 48 | 48 | 48 |
| Q1R | 65 | 65 | 65 |
| Q4R | 69 | 69 | 69 |
| QMR | 1029 | 1029 | 1029 |
| QAB | 79 | 79 | 79 |
| QBB | 78 | 78 | 78 |
| QCR | 36 | 36 | 36 |

## Work counters (median)

| Key | gte_vertices | software_vertices | triangles | clipped | backfaces | dropped triangles | bone matrices | cpu vertices | decoded vertices | visible chunks |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| A | 32 | 0 | 27 | 0 | 13 | 0 | 5 | 0 | 0 | 1 |
| B | 32 | 0 | 27 | 0 | 13 | 0 | 5 | 0 | 0 | 1 |
| C | 32 | 0 | 27 | 0 | 13 | 0 | 0 | 0 | 32 | 1 |
| D | 32 | 0 | 27 | 0 | 13 | 0 | 5 | 32 | 0 | 1 |
| I | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| J | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| Q1R | 32 | 0 | 27 | 0 | 13 | 0 | 5 | 0 | 0 | 1 |
| Q4R | 32 | 0 | 27 | 0 | 13 | 0 | 5 | 0 | 0 | 1 |
| QMR | 32 | 0 | 27 | 0 | 13 | 0 | 5 | 0 | 0 | 1 |
| QAB | 32 | 0 | 27 | 0 | 13 | 0 | 0 | 0 | 32 | 1 |
| QBB | 32 | 0 | 27 | 0 | 13 | 0 | 0 | 0 | 32 | 1 |
| QCR | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |

## Memory and size (bytes)

| Key | geometry | rigid pose | rigid desc. | baked frames | baked desc. | host tracks | animation total | animator/char | shared scratch | .ps-exe | static RAM | VRAM texture | VRAM palette |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| A | 3224 | 2700 | 290 | 0 | 0 | 2700 | 2990 | 20 | 6216 | 256000 | 825848 | 0 | 0 |
| B | 3224 | 2700 | 290 | 0 | 0 | 2700 | 2990 | 20 | 6216 | 278528 | 848376 | 2048 | 512 |
| C | 3180 | 0 | 0 | 7424 | 816 | 2700 | 8240 | 20 | 6216 | 282624 | 852472 | 2048 | 512 |
| D | 3224 | 2700 | 290 | 0 | 0 | 2700 | 2990 | 20 | 6216 | 278528 | 848376 | 2048 | 512 |
| I | 3224 | 2700 | 290 | 0 | 0 | 2700 | 2990 | 20 | 6216 | 282624 | 894576 | 2048 | 512 |
| J | 3180 | 0 | 0 | 7424 | 816 | 2700 | 8240 | 20 | 6216 | 288768 | 900720 | 2048 | 512 |
| Q1R | 3288 | 2700 | 290 | 0 | 0 | 2700 | 2990 | 20 | 6216 | 286720 | 860680 | 2048 | 512 |
| Q4R | 3288 | 2700 | 290 | 0 | 0 | 2700 | 2990 | 20 | 6216 | 286720 | 860680 | 2048 | 512 |
| QMR | 3288 | 2700 | 290 | 0 | 0 | 2700 | 2990 | 20 | 6216 | 286720 | 860680 | 2048 | 512 |
| QAB | 3180 | 2700 | 0 | 7424 | 1454 | 2700 | 11578 | 20 | 6216 | 294912 | 868872 | 2048 | 512 |
| QBB | 3180 | 2700 | 0 | 7424 | 1454 | 2700 | 11578 | 20 | 6216 | 294912 | 868872 | 2048 | 512 |
| QCR | 3288 | 2700 | 290 | 0 | 0 | 2700 | 2990 | 20 | 6216 | 286720 | 860680 | 2048 | 512 |

The animation budget limit reported by the cooker is 524288 B per model.

## Query sidecars

| Key | portable remap | baked seek tables | bone hierarchies | bone track tables |
| --- | --- | --- | --- | --- |
| A | False | 0 | 2 | 6 |
| B | False | 0 | 2 | 6 |
| C | False | 0 | 0 | 0 |
| D | False | 0 | 2 | 6 |
| I | False | 0 | 2 | 6 |
| J | False | 0 | 0 | 0 |
| Q1R | True | 0 | 2 | 6 |
| Q4R | True | 0 | 2 | 6 |
| QMR | True | 0 | 2 | 6 |
| QAB | False | 102 | 2 | 6 |
| QBB | False | 102 | 2 | 6 |
| QCR | True | 0 | 2 | 6 |
These are linked cooker tables, not inferred API availability. A no-query build must report no portable remap/seek/baked bone sidecars; rigid rendering retains its normal bone hierarchy and tracks.

## Explicit query work (median per completed frame)

| Key | calls | vertices | bones | decoded bytes | failures |
| --- | --- | --- | --- | --- | --- |
| Q1R | 1 | 1 | 3 | 0 | 0 |
| Q4R | 1 | 4 | 5 | 0 | 0 |
| QMR | 112 | 112 | 374.5 | 0 | 0 |
| QAB | 32 | 32 | 0 | 816 | 0 |
| QBB | 5 | 0 | 5 | 0 | 0 |
| QCR | 1 | 1 | 3 | 0 | 0 |
Counters are runtime measurements from the benchmark-only native oracle. They are deltas of cumulative counters divided by the actual completed-frame distance between debugger samples.

## Headroom against each resource

| Key | frame scanlines | static RAM | VRAM | animation bytes | clipped | dropped triangles |
| --- | --- | --- | --- | --- | --- | --- |
| A | 56 / 263 | 825848 / 2097152 | 630784 / 1048576 | 2990 | 0 | 0 |
| B | 58 / 263 | 848376 / 2097152 | 633344 / 1048576 | 2990 | 0 | 0 |
| C | 52 / 263 | 852472 / 2097152 | 633344 / 1048576 | 8240 | 0 | 0 |
| D | 85 / 263 | 848376 / 2097152 | 633344 / 1048576 | 2990 | 0 | 0 |
| I | 47 / 263 | 894576 / 2097152 | 633344 / 1048576 | 2990 | 0 | 0 |
| J | 48 / 263 | 900720 / 2097152 | 633344 / 1048576 | 8240 | 0 | 0 |
| Q1R | 65 / 263 | 860680 / 2097152 | 633344 / 1048576 | 2990 | 0 | 0 |
| Q4R | 69 / 263 | 860680 / 2097152 | 633344 / 1048576 | 2990 | 0 | 0 |
| QMR | 1029 / 263 (over) | 860680 / 2097152 | 633344 / 1048576 | 2990 | 0 | 0 |
| QAB | 79 / 263 | 868872 / 2097152 | 633344 / 1048576 | 11578 | 0 | 0 |
| QBB | 78 / 263 | 868872 / 2097152 | 633344 / 1048576 | 11578 | 0 | 0 |
| QCR | 36 / 263 | 860680 / 2097152 | 633344 / 1048576 | 2990 | 0 | 0 |
