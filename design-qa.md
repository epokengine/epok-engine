# Blueprint canvas visual QA

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
