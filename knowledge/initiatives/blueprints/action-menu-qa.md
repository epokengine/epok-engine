# Blueprint action menu — 2026-09-10

The unbounded, flat right-click list has been replaced by a fixed action picker
with a category tree, semantic icons, focused search, context filtering and
keyboard selection. Right-click creation retains the graph-space click position.

## Reference and measured implementation

The user's visual reference was a compressed, scaled 672 × 504 screenshot of a
conventional node-graph action picker in which the menu occupies approximately
334 × 334 image pixels. The layout figures below were taken from the measured
reference rather than from that image's scale:

- 400 × 400 content with 5 units of surrounding padding; context header and search.
- Categorized actions, search expansion and bold 9-point category text.
- Regular 12-point context title and 16-unit function icons.
- Icon plus label rows, semantic tint and vertical padding.

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
renderer. Pixel identity with the reference is not established: the supplied
screenshot is scaled/compressed, and the icon artwork is licensed Codicons/MDI
rather than the reference's own assets. Font antialiasing also differs between
renderers.
The MDI function outline is used for the green/blue function symbol; license
and font regeneration provenance are in THIRD_PARTY_NOTICES.md and the resource
maintenance notes. No external source or artwork was copied into the implementation.
