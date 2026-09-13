# Blueprint action menu — 2026-09-10

The unbounded, flat right-click list has been replaced by a fixed action picker
with a category tree, semantic icons, focused search, context filtering and
keyboard selection. Right-click creation retains the graph-space click position.

## Reference and measured implementation

The user's visual reference is
`C:/Users/Adolfo/AppData/Local/Temp/codex-clipboard-d47610ca-89df-46fd-82ef-d6c757caf2d8.png`.
It is a compressed, scaled 672 × 504 image. The menu occupies approximately
334 × 334 image pixels. The original layout, rather than that image's scale,
is established by Epic's source:

- [SBlueprintActionMenu.cpp](https://github.com/EpicGames/UnrealEngine/blob/release/Engine/Source/Editor/Kismet/Private/SBlueprintActionMenu.cpp):
  400 × 400 content with 5 units of surrounding padding; context header and search.
- [SGraphActionMenu.cpp](https://github.com/EpicGames/UnrealEngine/blob/release/Engine/Source/Editor/GraphEditor/Private/SGraphActionMenu.cpp):
  categorized actions, search expansion and bold 9-point category text.
- [StarshipStyle.cpp](https://github.com/EpicGames/UnrealEngine/blob/release/Engine/Source/Editor/EditorStyle/Private/StarshipStyle.cpp):
  regular 12-point context title and 16-unit function icons.
- [SBlueprintPalette.cpp](https://github.com/EpicGames/UnrealEngine/blob/release/Engine/Source/Editor/Kismet/Private/SBlueprintPalette.cpp):
  icon plus label rows, semantic tint and vertical padding.

Epok uses 410 × 410 outer units, Roboto 16px title, Roboto 12px labels,
Roboto Bold 12px categories, 18-unit category rows, 24-unit action rows,
14-unit tree indentation and 16-unit icons. The search has rounded ends,
a persistent magnifier and a working clear button. Only the tree scrolls.

The dominant reference colors were sampled and verified again in the actual
WGPU render: body RGB (25,25,25), header (55,55,55), search (14,14,14).
The current outer border is one unit and RGB (64,64,64).

## Validation

`cargo test --bin epok-editor blueprint_editor -- --include-ignored --test-threads=1`
passes all 21 tests, including the existing production graph mouse-event test.
Coverage includes category expansion without closing, creating a Branch,
search focus on opening, case-insensitive multi-word search with ancestors,
real socket filtering, Enter, Escape, long-list keyboard scrolling, and existing
connect/drag/undo behavior.

Set `EPOK_BP_ACTION_CAPTURE` to an output directory when running that command
to produce `blueprint-action-menu.png` and `blueprint-action-search.png`.
These are the production picker rendered through imgui-wgpu on an sRGB GPU
target, using an isolated test document; they are not screenshots of a running
user project. Both captures were opened alongside the supplied reference.
Visual iteration fixed missing font glyphs, the initial font mismatch, row
spacing, search/icon alignment, header shade, clear-button chrome and icon tint.

## Fidelity boundary

This implements the source layout and sampled palette in Epok's native ImGui
renderer. Pixel identity with Slate is not established: the supplied screenshot
is scaled/compressed, and the icon artwork is licensed Codicons/MDI rather than
Epic's original assets. Font antialiasing also differs between renderers.
The MDI function outline is used for the green/blue function symbol; license
and font regeneration provenance are in THIRD_PARTY_NOTICES.md and the resource
maintenance notes. No external source or artwork was copied into the implementation.
