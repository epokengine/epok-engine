# Phase 0 — Foundations (2026-10-13)

## 1. Target validation on the macOS host

The Apple Silicon host provides the Homebrew `mipsel-none-elf` GCC 16.2 toolchain,
pinned libclang 18.1.1 and PCSX-Redux. On this host:

| Check | Result |
| --- | --- |
| `cargo test --locked` (baseline, `develop` 7433fe6) | 900 passed, 71 ignored |
| `python3 tests/runtime/verify_spatial.py` (venv with `tools/requirements.txt`) | pass, includes `world2d.cpp` |
| `--build-psx` of `examples/sample-game` (C++ actor classes, reflection through libclang) | linked, 366.0 KiB EXE |
| `--screenshot-game` of the same build in PCSX-Redux | boots to `EPOK: runtime ready`, scene rendered |

So the actor runtime and the reflection extractor are no longer host-only on macOS:
both ran end to end through the real MIPS build and the emulator.

## 2. Measured MIPS sizes of the 2D runtime types

`world2d.hpp` previously carried only host (x86-64 clang) measurements. A probe
translation unit compiled with the exact runtime flags
(`mipsel-none-elf-g++ -std=c++20 -march=mips1 -mabi=32 -Os ...`) gives:

| Type | MIPS bytes | Host bytes |
| --- | ---: | ---: |
| `Camera2D` | 24 | 24 |
| `Collider2D` | 36 | 36 |
| `ColliderEntry2D` | 52 | 52 |
| `Aabb2D` | 16 | 16 |
| `Affine2D` | 24 | 24 |
| `Transform2D` | 24 | 24 |
| `SpatialHit2D` | 32 | 32 |
| `MoveResult2D` | 24 | 24 |
| `Pick2DEntry` | 32 | 32 |
| `CollisionWorld2D<32>` | 2,960 | 2,976 |
| `CollisionWorld2D<32,256>` | 5,264 | 5,280 |
| `Actor2D` | 216 | 184¹ |
| `SceneComponent2D` | 96 | 88¹ |
| `Actor3D` | 232 | 208¹ |
| `Level` | 688 | — |

¹ Host figures from p11-delivery; the object model gained fields since.

## 3. Resolution-aware VRAM reservation

`texture_cook::DISPLAY_RESERVE` reserved 640 columns at every row whatever the
display size. Framebuffers only cover the configured width, so 2D games at
320 x 240 wasted 320 columns (327,680 bytes, nearly half of VRAM).

- `texture_cook::display_reserve(size)` reserves exactly the display width.
  Free texture space is 376,832 bytes at 640, 704,512 bytes at 320 and
  770,048 bytes at 256 columns.
- Authored fonts are global, so they are still placed beyond column 640, and the
  same rectangles are then excluded from each bank's texture space. Font
  placement is identical for every resolution (unit test
  `fonts_keep_their_placement_whatever_the_display_width`).
- Every generated bank now `static_assert`s `textures_clear_of_display`, so a
  layout computed for the wrong display size cannot compile.
- Memory reports reserve the same columns (`memory::display_vram_layout`).

Emulator check: a 320 x 240 project with seven 256 x 256 8bpp textures (462,336
packed bytes, impossible beside 640-wide framebuffers) builds, places textures at
x = 512 and 576, and renders all seven correctly in PCSX-Redux.
