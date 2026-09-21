# Native editor visual QA reports

## Sequencer visual QA

final result: passed

Date: 2026-09-16. The supplied 1075 x 603 reference sequencer capture was used as
the visual target for the panel proportions, hierarchy, toolbar density, ruler,
track rows, key marks, selection range, playhead and dark palette.

### Visual truth and comparison

- Reference: a 1075 x 603 sequencer capture supplied for this pass.
- Final native render: `artifacts/sequencer/sequencer-native.png` (2150 x 1206
  Retina capture of a 1075 x 603 editor surface).
- Direct normalized comparison: `artifacts/sequencer/sequencer-comparison.png`.
- State: a real `Shot0110.timeline.json` asset is open over Epok's live scene
  viewport, with `Transform · Position` selected and keys visible across the
  complete two-second range.
- The reference and implementation were opened side by side in a single image
  at the same logical size. The final pass found no actionable P0/P1/P2 issue.

### Required fidelity surfaces

- Layout: the viewport occupies the upper 54% and Sequencer fills the lower
  area; the title strip, transport toolbar, breadcrumb, hierarchy, ruler and
  status line follow the supplied geometry.
- Typography and icons: the editor uses bundled Roboto, Codicons and Font
  Awesome assets at compact native sizes. All toolbar icons render without
  missing-glyph boxes.
- Color and state: charcoal panels, alternating rows, teal selected section,
  orange keys and range end, and the thin current-time cursor match the target.
- Content: the viewport remains Epok's real camera/grid render. The desert and
  animal belong to the user's own scene and were not copied as a backdrop.

### Interaction evidence

- The real ImGui harness passes add-key, atomic undo/redo, validation and save.
- Sequencer unit coverage passes frame conversion, multikey drag and clipboard
  identity regeneration.
- Native controls implement playback, reverse, loop, frame stepping, snapping,
  zoom/pan, box and multi-selection, key drag/copy/paste/delete, interpolation,
  section trim/slip and curve/channel editing.
- The final fixture compiles through the timeline compiler and its native editor
  capture was inspected at full resolution.

### Verification

- `cargo check --locked`: passed.
- `cargo test --locked timeline`: 37 passed, two intentionally ignored.
- `cargo test --locked sequencer`: two passed.
- Explicit native ImGui timeline and effect editor tests: passed.
- C++ Blueprint/timeline/director/adapter verification: passed. The optional
  pinned MIPS compiler check was skipped because that SDK tool is unavailable.
- Full Rust suite: 653 passed, 51 ignored; two unrelated audio tests remain
  blocked by a missing `psxavenc` dependency and the repository's existing
  legacy audio-package golden mismatch.

## Blueprint canvas visual QA

final result: passed

Date: 2026-09-08. This accepts the native Blueprint visual language and usable
panel structure, not pixel-identical reference UI or external API compatibility.

## Visual truth and comparison

- Graph reference: user-supplied temporary image, not distributed with the repository
  (694 x 361, graph only).
- Full-editor reference: user-supplied temporary image, not distributed with the repository
  (1583 x 950).
- Final implementation: `docs/images/blueprint-editor.png` (1581 x 917, lossless native-render PNG).
- Native Windows content size: 1581 x 917. No CSS viewport or browser scaling;
  the native capture excludes the OS title bar and borders. No resampling,
  compositing or image-content edits were applied.
- State: compiled `BP_Interaction`, event graph open, Components and My Blueprint
  expanded, no node selected in Details, no unsaved marker. The real fixture has
  different gameplay from the reference: compare panel hierarchy, headers, pin rows,
  wires and controls, not identical labels/positions or nonexistent external APIs.
- Both references and the final native capture were opened together in the same
  tool result. The full view and the graph-only reference make the small headers,
  pin labels and checkbox rows readable; no additional crop was needed.

## Findings and comparison history

The initial capture (`artifacts/blueprints/canvas-before-qa.png`) had raw call
identifiers, missing category icons, text-only boolean defaults, a darker graph
background, and one left accordion. These P2 findings were fixed with humanized
titles, existing bundled Codicons, undoable checkboxes, the first reference's
neutral gray grid, and separate Components/My Blueprint/Details panels.

The next capture (`artifacts/blueprints/canvas-panels-first.png`) exposed node
text leaking over Details. Intersected draw-list clip rectangles now constrain
both body and header text to the canvas; the explicit ImGui test verifies an
off-viewport node cannot expand the clip rectangle. Template edits discovered
during this acceptance were made transactional and covered by regression tests.

The final helper-function fixture exposed an inline Amount value touching its
same-row Value output. Width now reserves both labels, the actual literal chip,
pin insets and an explicit separation. A regression covers numeric/boolean/long
literal rows. The final native capture visibly preserves the gap.

No actionable P0/P1/P2 finding remains for the supplied visual targets.

## Required fidelity surfaces

- Typography: compact native labels and target subtitles are readable, with
  humanized graph/member labels. Epok keeps its shipped editor font; the reference's
  original font metrics and platform rasterization are not claimed identical.
- Layout: Components above searchable My Blueprint at left; graph in the center;
  contextual Details at right. Compact dark bodies, small radii, colored headers,
  aligned pin rows and attached curved wires reproduce the reference grammar.
  Epok retains its own toolbar and diagnostics instead of decorative reference UI.
- Colors: neutral panels, gray fine/major grid, white execution wires, typed data
  colors and semantic headers. Small node color differences are user-authorized.
  The first graph reference defines the lighter grid; the later reference adds
  the panel structure rather than replacing that graph reference.
- Image quality: genuine unchanged editor capture, not synthetic imagery or a mock.
  Native editable geometry and shipped icon-font assets, no screenshot backdrop.
- Content: the displayed classes, helper, inherited variables and components are
  real Epok assets. Compile succeeds. No external gameplay APIs are advertised.

## Interaction evidence

Actual Windows editor: Compile succeeds; inline Active toggles; Undo restores it
and clears the dirty marker; Visual Cube selection shows component properties;
Compute Strength opens its real function graph and editable signature; Class
Defaults shows the inherited bool/Q12 values. Earlier native acceptance verified
node selection, drag with attached cables, and exact Undo restoration.

Explicit real-ImGui harness: create/connect/drag/undo, inline boolean editing,
variable creation/Details, component selection, existing Canvas/HUD preservation,
Add Light with exact Undo, component search with retained ancestors, inherited
variable search without accidental override, and off-viewport clipping pass.
Consolidated Rust suite: 154 passed, seven intentionally ignored; the canvas
harness and native resource-order regression were additionally run explicitly.

## Follow-up polish and limits

The initial published desktop JPEG included the Windows cursor over a toolbar
button. It has been replaced with a fresh capture from the editor's existing
`--screenshot` render-surface export, which excludes OS cursor/hover overlays.
The isolated fixture was compiled before opening; the fresh capture's diagnostics
panel is idle. The earlier interactive Compile/Class Defaults checks above remain
valid. The replacement was inspected at full size: no cursor or foreign windows.

P3: exact reference typography, toolbar chrome and node gloss are intentionally not
cloned; Epok retains its native design system. The screenshot is a desktop
editor, not a responsive mobile UI. Hardware, macOS Blueprint authoring and
unrestricted feature parity are outside the validated contract.

## Implementation checklist

- [x] Fix identified P2 node and panel differences.
- [x] Verify real native interactions and regression coverage.
- [x] Compare final native capture with both supplied references.
- [x] Save the real screenshot for README and website publication.

## Inspector visual QA

final result: passed

Date: 2026-09-19. The supplied 368 x 892 property-editor capture was used as the
visual target for the section bands, row grid, density and type size of Epok's
Inspector. Its palette was deliberately not copied: the panel draws in the
editor's own theme so it reads as one more Epok window.

### Visual truth and comparison

- Reference: a 368 x 892 property-editor capture supplied for this pass.
- Final native render: the Inspector column of an editor `--screenshot` export,
  taken with `--screenshot-inspector` over the sample project, downscaled from
  the Retina surface to the reference's logical scale.
- State: a Mesh actor is selected, so the panel shows the identity strip, an
  actor class section, a component section, Transform and Mesh Renderer, ending
  with Add Component.
- Reference and implementation were placed side by side at the same logical
  width, and the finished panel was also compared against the surrounding
  editor windows to confirm it does not read as a foreign surface.

### Required fidelity surfaces

- Palette: every tone comes from `gui::theme`. The panel, its fields, buttons,
  text and blue accents are the editor's, and the only colors the panel names
  are the band it paints behind a section header (the same gray a collapsing
  header already used), a lighter hover and pressed step, and the title-bar gray
  for the hairline under a band. Sampling the render shows the panel sharing the
  Hierarchy's background exactly.
- Geometry: 17 px rows on a 21 px pitch, 1 px frame rounding, properties inset
  from the panel edges while bands run edge to edge, and a value column at 40%
  of the panel width.
- Typography: the panel draws with the bundled proportional face at 13 px
  instead of the editor's default fixed-width face, so a label and its value fit
  on one row at the reference's panel width.
- Sections follow the engine's own components, not the reference's. The mesh
  selector used to head a "Mesh Filter" section of its own, which read as a
  component that could not be removed; Epok has a single mesh component, so the
  selector is now the first row of whichever renderer section is showing.
- Section commands: a section's destructive and reset commands are not buttons
  in its body. Each section that owns commands draws three dots at the right end
  of its band, and the same menu opens by right-clicking the band, which matches
  the reference's per-component menu and keeps the body to properties only.
- Rows: every property in the panel pairs a left label with its control,
  including toggles, pickers and the material rows that previously printed
  their label after the control. Labels that do not fit are clipped with an
  ellipsis and a tooltip instead of wrapping.

### Verification

- `cargo build --locked`: passed, no new warnings.
- `cargo test --locked`: 686 passed, 54 ignored, two failures. Both failures
  (`audio_legacy_golden_outputs`, `audio_authoring_valid_target_error_and_tool_identity`)
  reproduce on the unmodified tree and are unrelated to this change.
- `cargo test --locked gui::`: 9 passed, three intentionally ignored. The real
  ImGui Inspector harness covers the identity strip's row alignment, and the
  pinned-toolchain harness test was additionally run explicitly: it opens a
  component's section menu with a real click and removes the component from it.
- Touched files are rustfmt clean.

### Limits

- Numeric fields center their value because the underlying immediate-mode drag
  widget hardcodes that alignment; the reference left-aligns it. Faking it would
  mean reimplementing numeric editing, which is out of scope for a restyle.
- The panel's docked tab strip is painted by the dock host, outside the window's
  style scope, so it still follows the editor-wide theme.
- Only the Inspector was restyled. Every other window keeps the editor theme.
