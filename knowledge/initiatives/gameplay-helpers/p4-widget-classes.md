# P4 widget class composition

Status: P4 validated. See [the final acceptance audit](p4-acceptance.md) and `artifacts/gameplay-helpers/p4/release-validation.json`. Earlier snapshots below retain their original scope.

The visual hierarchy belongs to the same registered class as its behavior. UI
classes derive from `UIActor`, and dialogue widgets derive from the common
dialogue presentation base. A conversation asset continues to own its content
graph. The removed dialogue-specific appearance window is superseded.

## Common authoring path

- Blueprint retains its existing `Template` inside the `.epokbp` class document.
- C++ and Lua retain their source declaration and class UUID. An optional,
  versioned `.epokclass` composition companion contains that UUID and the same
  `Template`. It adds no class identity, runtime hierarchy or behavior interpreter.
- The common resolver walks registered parent class identities, combines authored
  layers, applies overrides and remaps typed actor/component references. Placement,
  linked-instance refresh, scene reopening, Blueprint validation/placement/reset
  and console spawn-template preparation use this resolver. Authored sibling paint
  order is preserved independently of UUID ordering.
- Native default components are added through one materializer. Their stable
  local identities preserve references and instance overrides across refresh;
  adding defaults to a derived C++ class preserves its inherited controls.
- Companions reject duplicate owners, SDK-class edits, invalid versions, malformed
  identities and stale writes. Blueprint saves merge unrelated graph changes only
  when the captured composition still matches. Nil reference targets explicitly
  clear an inherited binding.
- Composition source signatures join stage provenance and saved-source observation.
  Invalid or changed documents invalidate their consumers. File duplication cannot
  copy class identities; creating a derived class is the supported reuse path.

## UI editor integration

The generic UI class designer shares the existing component hierarchy and typed
inspectors with the Blueprint editor. Project source context actions and an actor's
`Edit UI Class` button open it; `Behavior` opens the source or graph for that class.
The designer keeps inherited controls as overrides, supports undo/redo and Save
All, and protects unsaved edits when changing or closing documents.

The canvas uses the existing HUD layout/compiler and rasterizer with authored
assets and defaults. It does not compile or invoke project behavior for discovery.
It supports selecting and moving controls, normal component inspectors and typed
control binding. UI controls show their RectTransform rather than a redundant
world transform. Source context actions, inspector actions and `--open-ui-class`
open the same designer. `--capture-ui-class CLASS --actor UUID-or-name` imports
the first composition from a saved scene; replacing an existing authored root is
rejected to preserve placed instances. This initial editor integration still needs complete visual and
input acceptance; it is not the completed widget authoring release.

## Validation and remaining gates

The first two composition tests pass: actual C++, Blueprint and Lua declarations,
independent placement, native defaults added by a derived class, component-reference
remapping, branch duplication, linked refresh preserving instance edits,
save/reopen and console template preparation. Duplicate ownership, SDK ownership,
version and stale-write rejection are covered.

The preceding foundation passed **1,226 tests, with 88 ignored**, across three
nonempty suites; the editor suite takes 167.31 seconds. The pixel test deliberately
gives a background a UUID that sorts after its foreground label and verifies the
label still renders, then verifies that a text edit changes the preview pixels.
The provider tests also recompile the normal catalog after saving inherited
control overrides. Saved-source observation invalidates a consuming artifact
after a companion changes. Three focused Windows-path tests pass using canonical
project paths.

Validation exposed and corrected Blueprint source ownership, projection
synchronization, partial registries during scene refresh and UUID-based paint
ordering. A real UI-class editor capture exposed the paint-order defect before
the pixel regression was added. The initial capture is superseded by the final
visual inspection. Logs and current source signatures are recorded in
`artifacts/gameplay-helpers/p4/validation.json`.

Its debug editor build and Clippy with warnings denied passed. Actual
Blueprint, C++ and Lua classes derived from the dialogue base were captured
through the common importer and inspected in the same UI class designer. All
three display the authored panel, speaker name, body and choice label at
640×480, with 66 glyphs and zero dropped primitives. Captures are
`widget-class-blueprint.png`, `widget-class-cpp.png` and `widget-class-lua.png`.
The finished executable also rejects a second capture over an authored root;
the original class document is preserved. These captures validate default-data
preview, not dialogue execution or the complete interactive authoring workflow.

## Authored dialogue layout follow-up

The native presenter now binds text, participant portraits, visibility and choice
focus while preserving each control's RectTransform, font, color, alignment,
wrapping and panel material. The widget owns those values. Selection uses the
ordinary Canvas/Focusable contract. Starter geometry is authored once when a
composition is created; it is never reapplied during a session.

Cooked pagination profiles come from the same HUD layout core as the editor and
runtime, including imported font advances and each control's wrapping setting.
Distinct control sizes/fonts have distinct resource identities; position and
color do not multiply the page tables. A localized key used both as a paginated
line and as a name/option keeps independent uses, so a long line does not split
a participant's name. Exact authored source/widget bindings receive fit
diagnostics; an unrelated narrow widget does not invalidate a compatible one.
Native Play now includes source/Blueprint spawn compositions and their resources.

Project > Create > Dialogue Widgets provides Classic Box, Portrait Conversation,
Subtitles and Speech Bubble in C++, Blueprint or Lua. Each choice creates a normal
editable project class and captures its controls through the same class designer.
The CLI equivalent is:

```text
epok-editor --project PATH --new-dialogue-widget MyDialogue --gameplay cpp --presentation classic_box
```

The other presentation keys are `portrait_conversation`, `subtitles` and
`speech_bubble`; the other providers are `blueprint` and `lua`. Creating an
existing class is rejected. Starter controls anchor to the viewport and retain
ordinary instance overrides. The current composition costs 11 actors and
27 components, including four optional Focusable controls.

The generic UI designer offers authored defaults plus deterministic line, prompt,
choice and closed sample content for dialogue classes. These are render-only samples:
selection, dragging, properties and saving still use the original authored scene.
`--open-ui-class MyDialogue --ui-preview line|prompt|choices|closed` selects a sample for
editor captures. Samples do not execute project conditions or events and are not
the complete localized conversation-preview acceptance scenario.

The final follow-up workspace run passes **1,230 tests, with 88 ignored**, across
three nonempty suites; the editor suite takes 193.89 seconds. Clippy with warnings
denied, formatting, diff checks and the debug editor build pass. The tests cover
all 12 starter classes at both resolutions, render-only sample states, preserved
instance edits and deliberately empty custom widgets. Native Play spawns C++,
Blueprint and Lua widgets with distinct pagination and exercises cancellation and
destruction. This includes a nested Windows project path. Temporary compiler
staging stays in the canonical native cache outside the long content-key folder.
Object-registry capacity now includes the component density of spawn compositions;
the previous flat 32-object reserve could reject a single multi-control widget.

The native host test preserves authored geometry, fonts, colors, alignment,
wrapping and panel material through reveal, choices, selection and cancellation.
Dialogue, interaction, controls and reveal host suites pass, as do API-reference
freshness, the API parity inventory and its five tool tests.

Actual CLI creation and duplicate rejection pass for all 12 classes. Eight editor
captures cover all starters at both resolutions with zero dropped primitives;
the class documents remain byte-identical after the previews. A final editor
capture uses the executable after the host-staging path fix. These are placeholder
sample states, with an untextured portrait. Light-theme choice focus needs a
stronger visual distinction before finished-style acceptance.

Console acceptance uses a Lua widget overriding `present` and calling its parent,
with C++, Blueprint and Lua interaction rules in both domains. All three Lua
executions pass content, choices, authored-rectangle and lifecycle checks: static
Main RAM is 1,381,688 bytes for Native C++, 1,705,064 for VM bytecode and 1,764,456
for VM source. The VM-source standalone export rebuilds byte-identically. Heap
and stack peaks remain unknown. The unused Basic comparison stays byte-identical
to P3 at 964,400 static bytes and a 421,888-byte executable. Target and eight visual
captures precede the final host-only staging path adjustment; exact executable
hashes and unchanged native runtime headers remain recorded in the evidence.

P4 still requires reusable nested widget compositions, finished canvas editing,
full localized conversation preview, anchored speech bubbles, removal of obsolete
style appearance fields and final runtime/export validation of every finished
widget across all providers and both VM modes. Pagination is cooked for the authored control
size/font; changing those at runtime does not currently reflow a session. P5–P10
retain their full original scope.

## Prompt, continuation and widget lifecycle follow-up

`DialoguePresenter` now has optional TextComponent bindings for `prompt` and
`continuation`. A bound prompt paginates using its own font, size and wrapping,
remains visible during selection and does not replace the body control. An
unbound prompt falls back to the body before selection. The continuation control
retains its authored text and appearance; the binder toggles its enabled state
while waiting. A shared localized key can have independent line, prompt and
single-label uses. Unbound optional names retain the 511-byte content bound
without imposing a font or glyph requirement on invisible content.

The reflected `dialogue_opened` and `dialogue_closed` events receive the bounded
16-word DialogueSnapshot in all providers. Closing retains session/participant
identities and a terminal result after input capture is released. The kernel's
internal finish callback also receives the session's presenter so source
invalidation can still close the right widget; the adapter revalidates source
liveness before invoking project callbacks. Cleanup hides inactive controls too,
avoids duplicate closing and stops mutating controls after releasing ownership,
allowing a derived close animation. Registry replacement cannot make an old
widget invalidate an unrelated new session with colliding object identities.

The focused Rust run passes 34 tests. Native host tests pass authored-layout,
paged-question, continuation, reentry, disabled-control, source/widget destruction,
scene-unload, registry-replacement and close-animation scenarios. The full workspace
passes **1,231 tests, with 88 ignored**, across three nonempty suites; the editor
suite takes 199.73 seconds. Formatting, Clippy with warnings denied, the debug build,
API generation/freshness and the API inventory pass. The inventory classifies 773
public entries, 5,807 total and zero pending; its five tool tests pass.

Actual C++, Blueprint and Lua widgets override all three presentation events and
call their parent. Each runs with all three Lua execution modes and the same six
C++/Blueprint/Lua 2D/3D interaction targets. All nine cases pass 18 openings,
18 closings, 54 presentation observations, prompt and continuation visibility,
authored body geometry and cleanup/capture checks: **162 sessions** in total.
Static Main RAM is measured in bytes below; heap and stack peaks remain unknown.

| Widget provider | Native C++ | VM bytecode | VM source |
| --- | ---: | ---: | ---: |
| C++ | 1,399,784 | 1,718,992 | 1,778,384 |
| Blueprint | 1,404,024 | 1,723,240 | 1,782,632 |
| Lua | 1,404,024 | 1,731,504 | 1,790,896 |

The Lua-widget VM-source standalone export rebuilds byte-identically. Unused
Basic remains byte-identical to P3 at 964,400 static bytes and 421,888 executable
bytes. All 12 CLI-created starter classes preserve their composition and reject
duplicate creation. Thirty-two inspected captures cover line, prompt, choice and
closed samples for four presets at both resolutions, with zero dropped primitives.
Closed samples contain zero glyphs. These are render-only placeholder samples;
the portrait is untextured, panels need final styling and text-only focus needs
work. Evidence uses the same final debug executable, SHA-256
`ce487de8be1333999c08e09787cf8b5773036d670c4c7d041d1e83f9c20d3cf0`.
`artifacts/gameplay-helpers/p4/validation.json` records the current source hashes,
checks and captures; `widget-layout-validation.json` preserves the prior milestone.
This follow-up does not close P4.

### Findings from the preceding captures

Use the existing `SceneLibrary::project`/`camera_project` contract for 3D and
`world2d::view_camera` plus `world_to_screen` for 2D. Before binding a bubble to
a speaker, align projection validity between console and Native Play: console
`runtime/main.cpp::camera_project` rejected off-screen points and the far plane,
while `native/native_play_runner.cpp::camera_project` only rejected the near/
behind-camera case. The follow-up below addresses this discrepancy. Keep the
widget in ordinary screen-space UI and update its root through those existing
spatial conversions; a second world-space UI system is unnecessary.

The preceding runtime fallback checked whether a bound prompt was live, while
cooking retained its metrics. Falling back to a differently sized body could
use the wrong page layout. The follow-up below ties fallback to an unbound prompt
and covers deactivation and reactivation.

The updated Classic Box choice capture also has no distinct selected text row.
The generic HUD Focusable highlight currently modulates images and fills, while
starter choice rows contain Text only. Final visual acceptance must address this
through the shared UI control/rendering contract or ordinary authored controls;
do not restore dialogue-specific per-frame color/layout assignment. Light-theme
text also needs a clearly visible selected state.

## Shared focus and projection follow-up

The generic Focusable component now has an opt-in one-pixel outline, exposed by
the ordinary inspector and reflected `outline_enabled`/`set_outline` methods.
It uses Highlight as its frame colour, keeps authored text/markup colours and
uses up to four existing rectangle/rotated primitives. Hidden visuals do not
leave an outline behind. The native flag uses existing Focusable padding, and
the editor preview protocol uses a spare flag bit. Existing controls default
to no outline. New dialogue starters opt in, with a dark blue highlight on the
light bubble theme and amber on the dark themes. Actor/component cost remains
11/27.

Both whole-frame and per-node HUD caches include the outline state, including
white highlights whose tint does not change with focus. Nested canvases restore
their parent's selection before sibling traversal. Scene validation reserves
the largest outline in each Canvas for each primitive pool; dialogue validation
also renders every selectable row's focus. Tests cover ordinary/rotated frames,
small controls, hidden visuals, unchanged markup, cache parity and budgets.
Native preview pixels prove that a text-only control gains a visible frame on
a light background. Bound-but-inactive prompts no longer fall back to a body
with different cooked metrics; deactivation/reactivation has a host regression.

Console and Native Play now share the Q12 focal/clip math in
`runtime/camera_projection.hpp`. Near is inclusive, far/right/bottom are exclusive,
failure preserves output and widened intermediates reject extreme off-screen
coordinates. Missing active cameras fail in both adapters. The existing console
lens formula is retained. The new header is part of the canonical staged runtime.
Host tests cover both resolutions, mirrors, lens bounds, viewport edges and
overflow. A real Native Play class passes 16 public-API cases, including camera
deactivation/restoration. This provides the projection foundation; an anchored
widget composition/component still needs implementation and acceptance.

The workspace passes **1,233 tests, with 88 ignored**, across three nonempty
suites (editor suite 212.51 seconds). Clippy with warnings denied, formatting and
the debug build pass. API generation/freshness and its inventory pass with 775
public entries, 5,812 total and zero pending; five inventory-tool tests pass.
Current console and visual follow-up evidence uses the `widget-focus-*` prefix.
The preceding 162-session milestone remains in `widget-state-validation.json`;
it is historical evidence for its recorded sources, rather than a claim that
those nine cases were repeated after the shared UI/projection changes.

The current Lua-widget VM-source fixture passes 18 sessions with all three
interaction providers in both domains. Its public camera probe passes the same
16 cases as Native Play, including mirror, lens and missing-camera behavior.
The GPU-facing HUD statistics confirm the four outline rectangles during choices,
zero dropped primitives for that observation and zero residual HUD after cleanup.
Native outline getters/setters also round-trip. This exposed an acceptance-fixture
defect: logical parenting alone left its controls unattached to the Canvas, and
top-level `enabled` values did not initialize grouped Text data. The fixture now
authors the spatial attachment and hidden Text group explicitly. The initial
failure is retained in `widget-focus-console-initial.log`.

The current fixture uses 1,796,776 static bytes and its standalone export rebuilds
byte-identically. Basic uses 966,448 static bytes and a 423,936-byte executable,
2,048 bytes more in each than the preceding linked image after these shared-core
changes. This is measured cost, not a zero-cost claim. There are no extra widget
actors/components; the updated focus option keeps the starter at 11/27.
All 12 CLI-created classes preserve their composition and reject duplicate creation.
Eight inspected choice captures show clear amber/dark-blue frames in all presets
at 320×240 and 640×480, 58 glyphs and zero dropped primitives. Panel/subtitle polish,
the portrait placeholder, nested widgets and full conversation preview remain open.

### Anchored composition implementation

`ScreenAnchorComponent` implements bubble anchoring by targeting a normal child
RectTransform container. A Canvas deliberately ignores its own
RectTransform for layout, so moving the presenter Canvas cannot position a bubble.
The container has the bubble's authored extent/pivot and owns the existing
controls, with both logical parenting and spatial attachment. The anchor updates
only the container position, derives clamping from that extent and reuses the
shared 3D projection or existing 2D world matrix/camera conversion. Child fonts,
materials and geometry remain authored values. Count the additional container
and anchor component explicitly: Speech Bubble uses **12 actors / 29 components**;
the other starters remain **11 / 27**. The component is independently demand-loaded
through `EPOK_UI_ANCHOR`; a standalone anchored label does not enable interaction,
control capture or dialogue. Both headers join canonical runtime staging.

Off-screen, behind-camera, absent-camera and unavailable-target projections use a
visible bottom-center fallback. The source actor supplies narrator positioning;
the session already handles destroyed participants. Clearing an anchor target
stops following without overwriting a derived close animation. Components
run before their owning actor's `on_frame`; the presenter therefore refreshes the
anchor immediately when its target changes. Enabled invalid content
makes the presenter unavailable and releases input capture. Content must be a
direct child of the owning Canvas, fit the viewport/margins and have no arranging
LayoutContainer on that Canvas. Normalized anchors/pivots, rotated bounds, extreme
offsets and margins are checked before fixed arithmetic.

Host tests cover 2D translation/rotation/zoom, shared 3D projection, viewport
clamping, invalid geometry, destroyed targets, freezing, source binding and capture
release. The workspace passes **1,235 tests, with 88 ignored**, formatting, Clippy
with warnings denied and the debug build. Native Play spawns Speech Bubble in all
three providers with independent authored pagination and cancellation. The separate
native anchor fixture passes eight projection/fallback/freeze checks without
interaction or dialogue enabled.

The current console case passes 18 sessions with a Lua VM-source widget and three
interaction providers in both domains. A separate 3D anchor passes eight checks;
the Lua widget calls all four anchor methods and returns their enums/boolean to a
native observer in each of its 18 openings. The existing 16 projection checks,
focus contours and cleanup pass. This fixture, including instrumentation, uses
**1,828,288 static bytes**. Standalone rebuilds byte-identically. Basic remains
byte-identical to the preceding milestone at 966,448 static bytes / 423,936 EXE bytes.
This validates independent 3D anchoring on console and 2D anchoring in host tests;
the complete finished-bubble/provider/target matrix remains open.

All 12 starters pass both-resolution composition/pagination/preview tests and CLI
creation/duplicate protection. Their documents remain unchanged after previewing.
Eight inspected choice captures cover all four starters at both resolutions,
with 58 glyphs and zero dropped primitives. The new bubble container centers at
both viewport widths. These retain placeholder portrait and subtitle styling;
they are not finished-style acceptance. Current source/executable provenance is
`artifacts/gameplay-helpers/p4/validation.json`; the preceding milestone is retained
in `widget-focus-validation.json`. API
freshness covers 137 headers / 2,411 callables; parity includes `ui-anchor` as a
public UI surface (789 public capabilities, 5,849 total, zero pending
classifications). The class designer previews authored defaults;
the older isolated HUD simulator reports an explicit Native Play requirement.
Finished bubble styling and the final provider/target matrix remain P4 gates.

## Nested class composition follow-up

Implementation is in progress; this supersedes the earlier statement that nested
class composition is absent. The current `Template.instances` set marks local
child roots that instantiate an existing registered Actor class. Ordinary class
inheritance still supplies the composition root. No new runtime widget hierarchy,
class identity or behavior interpreter is introduced.

The resolver expands child compositions from C++, Blueprint and Lua through the
same sources used by placement, refresh and cook. Namespace identities derive
from the local instance root plus the child class's local object identities.
Repeated resolution is deterministic. Child bindings are resolved before parent
overrides, so parent edits can replace them without being overwritten. Expansion
preserves paint order and checks recursion, missing class IDs, the 32-actor
template bound and the aggregate 256-operation construction bound.

The shared component browser adds **Reusable class** and uses the same inspectors
for overrides. Class-defined descendants remain inherited; local instances can be
removed as a branch after external bindings are cleared. Scene capture preserves
child class instances and converts actual authored changes into overrides instead
of flattening their controls. Source UI capture rejects replacing an inherited
root. Blueprint capture retains stable existing actor/component identities and
protects already-authored nested composition from replacement.

Blueprint format 6 and source companion format 2 declare this serialized feature.
Reading an existing Blueprint 5 or companion 1 without nested-instance declarations
migrates in memory, preserving the file until an explicit save. Earlier unsupported
Blueprint formats remain rejected. A new nested declaration under an old format
is rejected rather than silently reinterpreted.

Nested Canvas controls use their authored bounds and rotation. The ordinary
**Screen overlay** property opts a nested Canvas into full-viewport coordinates,
without inheriting ancestor rotation or reserving a parent layout cell. Speech
Bubble enables it; world anchors require an overlay Canvas when physically nested.
The preview protocol uses a spare flag without changing its wire layout. Runtime
layout caching includes the flag, and hit order visits a nested Canvas only once.
Primitive budgeting uses the resolved transform, including ancestor rotations.

Composition/capture/refresh/cook tests pass all three providers, and host HUD/anchor
tests pass local/overlay bounds and rotation. Direct and indirect cycles and the
aggregate 256-operation boundary are covered. Real ImGui cursor input selects and
drags a rotated inherited child in each provider, with one undo step, redo and
save/reopen. The selected outline follows the rendered corners. Native Play runs
two containers with six independent C++/Blueprint/Lua child behaviors, typed text
bindings and authored local bounds, with 46 glyphs and no dropped primitives.

The workspace passes **1,240 tests / 88 ignored** (editor suite: 211.70 seconds).
The initial failed run and intermediate fixture corrections remain in separate
logs. API freshness covers 137 headers / 2,413 callables; parity has 791 public
capabilities / 5,852 total / zero pending classifications, and five tool tests pass.
Formatting, Clippy with warnings denied and the final debug build pass. The PSX
probe returns `[7, 6, 1, 46, 0]` in native, VM bytecode and VM source modes: all
three child behaviors, six unique bindings, valid local bounds, 46 glyphs and zero
dropped primitives. The instrumented fixture uses 1,117,392 / 1,422,256 / 1,483,696
static bytes respectively. These are whole-fixture costs. VM source standalone
rebuilds byte-identically. Basic uses 966,576 static bytes (+128), with the same
423,936-byte EXE size and a changed image hash. Ten screenshots were inspected:
the generic nested-class designer at both resolutions, and eight choice previews
covering four starters at both resolutions. The latter show 58 glyphs and zero
dropped primitives; placeholder portraits, opaque subtitles and final panel
styling remain unfinished. The independent nested fixture deliberately uses fixed
640-pixel columns, so its 320-pixel preview clips the later columns rather than
resizing authored controls. Its designer shows authored strings, while Play runs
the children's numeric updates. Current source/executable provenance is
`artifacts/gameplay-helpers/p4/validation.json` (also preserved as
`widget-nesting-validation.json`).
The preceding fully recorded anchor milestone is
`artifacts/gameplay-helpers/p4/widget-anchor-validation.json`.

This is a bounded nesting milestone, not complete UI authoring acceptance. The
generic component menus now admit Canvas on a UIActor root that already owns a
RectTransform, and nested Canvas can be added without replacing existing geometry
or graphics. The class designer still needs the full generic reflected component
property workflow. Localized dialogue samples, finished
starter visuals and the full provider/target acceptance matrix also remain open.

The target fixture is produced by
`native_play::tests::nested_widget_classes_keep_their_own_behavior_and_control_bindings_in_native_play`
with `EPOK_WIDGET_EVIDENCE_DIR` set to the destination project. It saves ordinary
sources, settings and the placed startup scene, excluding native caches.
`tests/integration/verify_widget_composition.py` builds and executes that project
in all three Lua modes and checks the independent probe, memory and export image.

## Generic component authoring follow-up

The component editing core is implemented after the recorded nesting milestone.
The scene Inspector and UI
class designer now share component choices, adapter initialization, model
validation and reflected value editors. Component selection uses its UUID in the
existing hierarchy. Class properties and component properties use the resolved
class scene for typed reference choices. Local edits are transactional; invalid
bindings, wrong owner domains and an exceeded component limit restore the prior
template. Inherited components remain class-owned, while local additions can be
removed after their references are cleared. Reset removes only the local property
override and restores inherited values/bindings.

The generic component toolbar also opens the component's existing C++, Blueprint
or Lua behavior editor. Screenshot automation can select a component using
`--open-ui-class WidgetClass --ui-component UUID-or-unique-name`. No separate
dialogue visual panel or component class catalog is introduced.

An explicit property value of None must remain distinct from deleting an
override. Blueprint 7 and companion 3 add a tagged literal None in the existing
member-edit table. Older null member edits retain their deletion semantics.
Blueprint 5/6 and companion 1/2 upgrade in memory when their feature declarations
match their format; mislabelled new features are rejected without rewriting.
The scene format is unchanged; this authoring path emits the tag in class
composition overrides. C++, Blueprint and Lua component fixtures exercise the
same editable properties and typed controls. The final workspace passes **1,244
tests / 88 ignored** (editor suite 192.78 seconds), formatting, Clippy with warnings
denied and the debug build. Six class-designer tests include real ImGui input in
all providers, explicit None, inherited reset, future parent-value propagation,
deletion/reference refusal, undo/redo, save/reopen and immediate budget rejection.
Native Play executes six C++/Blueprint/Lua components in duplicated widget classes.

The regenerated PSX fixture passes all three Lua modes with probe
`[7, 6, 1, 18, 0]`: three behaviors, six independent controls, correct typed owners
and configuration, 18 glyphs and zero dropped primitives. It uses 1,083,608 /
1,392,648 / 1,454,088 static bytes in native / bytecode / source modes. These are
whole-project instrumented costs. VM source standalone rebuilds byte-identically.
Basic remains byte-identical to the nesting image at 966,576 static bytes and a
423,936-byte EXE. The first-snapshot evidence is preserved separately, and the
final editor is SHA-256 `fe06ea77964a6d829d0bad012381ff241aea8e0d24c4d0049ec97898df89ebe0`.
All 11 captures (three component inspectors and eight choice previews) were
inspected; the reusable preset verifier passed after correcting existing
Blueprint/Lua identity lookup. The portrait placeholder, opaque subtitles and
panel sizing remain unfinished. Nullable scene-placement overrides, built-in
resource choices, localized preview and finished-style acceptance remain open.
The preceding nesting manifest certifies its own recorded sources, not this
follow-up.

### Placement inheritance acceptance

The scene inspector now uses the same property-edit and reset contract for actor
and component fields. Missing members remain deletion edits; an explicitly empty
reference is encoded as literal None. Reset removes only that property's authored
member edits and refreshes the linked class transactionally. Placement override
markers describe the placement's edits rather than inherited class values.

Scene format 7 declares nullable placement edits. Actor-format scene 6 loads and
upgrades in memory without rewriting its source; earlier retired formats remain
rejected. A scene marked 6 containing the new literal marker is rejected with a
version diagnostic. Existing plain-null deletion edits keep their meaning.

Placement inheritance acceptance passes: **1,247 workspace tests / 88 ignored**, formatting, Clippy with warnings denied, debug build, API freshness and five tool tests. The editor-path fixture covers C++, Blueprint and Lua actor/component None values, save/reopen, duplicated identities, inherited reset and subsequent class-default changes. Three inspected scene Inspector captures, each isolating one provider's branch, show only the placement's Target property override, with inherited Amount/Enabled fields unmarked. PSX passes native, VM bytecode and VM source with probe `[0, 6, 0, 84, 0]`: six empty component references, unchanged authored labels, 84 glyphs and zero drops. Whole-fixture static RAM is 1,083,608 / 1,392,648 / 1,454,088 bytes. VM source standalone rebuilds byte-identically. Unused Basic remains byte-identical at 966,576 static bytes and a 423,936-byte EXE (zero delta). Current provenance is `artifacts/gameplay-helpers/p4/widget-placement-validation.json`; all preceding manifests retain their own source/editor hashes.

### Project resource authoring acceptance

The scene, generic UI class designer and Blueprint composition inspector now
borrow the same project asset index and HUD string table. Image textures, text
fonts, material textures and playable audio use the existing typed reference
selector. Missing/incompatible references stay visible, and conflicting asset
identities are excluded from choices. The Strings button requests the ordinary
project string editor. Generic widget preview resolves authored text keys before
rendering, using the current preview language without executing project code.

Project resource authoring acceptance passes: **1,248 workspace tests / 88 ignored**, formatting, Clippy, debug build, API freshness and five tool tests. Scene, generic class and Blueprint composition inspectors share project asset selectors and HUD string keys. Real ImGui input selects a Font, Texture and localized key in C++, Blueprint and Lua; undo/redo, save/reopen and the normal Strings editor request pass. All providers render English/Spanish with seven/six glyphs and zero drops using the imported font, retaining the authored text. The test exposed and fixed a stale-language preview cache. Three captured designer previews were inspected. Unused Basic remains byte-identical at 966,576 static bytes / 423,936 EXE bytes, with zero delta. Runtime headers are unchanged; no new console/export claim is made for this editor-only milestone. Its provenance is `artifacts/gameplay-helpers/p4/widget-resources-validation.json`.

Actual conversation preview, component-specific inspection, persistent control focus-link authoring, finished styles and the final provider/domain/target/export matrix remain P4 gates. P5–P10 retain their full scope.

### Component inspection and persistent focus (validated authoring milestone)

Selecting a common UI component now draws that component's fields; selecting
Text does not draw RectTransform, Canvas, Image or Focusable controls. Audio uses
its own field set. The existing hierarchy, resource selectors and class document
remain authoritative. Component override reset buttons show property names.

Canvas initial focus and Focusable navigation are authored with persistent actor
UUIDs, restricted to focusable UI controls in the same Canvas. The cooker and HUD
preview resolve those identities into the existing runtime index fields. Capture,
duplication, namespace expansion and refresh reuse normal identity remapping.
Explicit None is preserved. Scene 8, Blueprint 8 and companion 4 declare the new
authoring feature; compatible prior documents upgrade only in memory. Legacy
scene indices are materialized while their original ordering is still known.

Runtime prototype instantiation also remaps local focus indices into allocated
data slots before BeginPlay. The focused native host contract and pinned MIPS
syntax check pass spawn, teardown, recreation and unchanged prototype data.
Integrated acceptance passes real component-picker input, inheritance,
undo/redo, save/reopen, and placed/duplicated/spawned classes in all three
providers. The workspace passes **1,251 tests / 88 ignored**, formatting, Clippy,
debug build, API freshness and five tool tests. Native Play and all three console
execution modes pass nine independent widgets with 18 glyphs and zero drops.
VM-source standalone export rebuilds byte-identically. Nine inspected designer
captures show Canvas, Text and Focusable fields in C++, Blueprint and Lua;
each preview has two glyphs and zero drops. Unused Basic remains at 966,576
static bytes and 423,936 EXE bytes, with zero size delta; its executable hash
changes. Provenance: `artifacts/gameplay-helpers/p4/widget-focus-links-validation.json`.
P4 remains open for the lifecycle issue below, actual conversation preview,
finished styles and the final matrix; P5–P10 retain their original scope.

Before the repair below, the additional lifecycle audit reproduced a runtime issue: destroying
only the focused child leaves the Canvas's compact focus index intact. Reusing
that data slot for another actor can alias the replacement (slot 4, generation
2 to 3 in `widget-focus-links-lifetime-audit.json`). The nine-instance authoring
and spawn fixture does not certify target-only destruction. Clear incoming
Canvas/neighbor indices during storage release and teardown, then verify slot
reuse, callback ordering and scene unload before P4 acceptance. That audit established the lifecycle gate addressed by the next milestone;
localized real-conversation preview and finished styles remain open.

### Focus lifecycle repair (validated milestone)

Level storage release clears incoming compact Canvas and navigation links after
end-play callbacks and before actor data can be reused. The same operation covers
unstarted spawn rollback. It examines disabled controls as well as enabled ones
and adds no per-actor storage, heap allocation or per-frame work.

The workspace passes **1,252 tests / 88 ignored**, formatting, Clippy, debug build,
API freshness and five tool tests. The full native Blueprint runtime suite and
pinned MIPS syntax pass; its focused contract covers failed preparation,
unrelated-link preservation, partial destruction, slot reuse and level unload.
The original reproduction now reports no stale alias (same slot 4, generation
2 to 3); its failing evidence remains preserved.

Native Play and all three console modes pass destruction of the focused child
in each dynamically spawned C++/Blueprint/Lua widget. Three replacement controls
reuse those slots with new generations without inheriting Canvas focus or sibling
navigation. The final frame has 15 glyphs and zero drops. Whole-fixture static
RAM is 1,111,736 / 1,416,616 / 1,478,056 bytes in Native, VM bytecode and VM source, respectively.
VM-source standalone export rebuilds byte-identically, including a fresh rebuild
from relocated exported inputs outside the source project. Unused Basic uses 968,624 static bytes / 425,984 EXE bytes, a +2,048 / +2,048 byte delta from the focus-authoring snapshot.
Provenance: `artifacts/gameplay-helpers/p4/widget-focus-lifetime-validation.json`.

This closes the reproduced focus lifecycle gate. P4 remains open for localized
real-conversation preview, synthetic input, four finished starters, obsolete visual
style fields and the complete provider/domain/target/export acceptance. P5–P10
retain all character, camera, helper and template requirements, including 30 cases.

### Real conversation preview (validated milestone)

The common UI class designer now has a Conversation preview state with ordinary
asset selectors, language and viewport choices, a speaker/start node, explicit
condition values, synthetic confirm/navigation/cancel input and time stepping.
These inputs belong to the preview and are not class composition properties.

The native editor adapter compiles the existing Session and DialoguePresenter in
an isolated namespace. It registers only native SDK bases, marshals the authored
control tree and cooked localized pages, and replays preview input through the
real input router. The normal presenter binds content, reveal, choices, portraits
and focus, and the normal HUD compiler draws it. Project classes and callback
implementations are never linked into this preview. Conditions use explicit
values, and actions/node entries produce a trace without game-side effects.
Explicit Play remains the authority for provider behavior and world anchoring.

Pagination uses the existing widget profile and imported fonts. A separate texture
bank includes conversation-only portraits without adding resource actors to the
class. The authored scene remains the source for editing and saving. A preview
context change restarts input replay; restarting the initial state also invalidates
the cache. The first focused run exposed a fixture that tried to recapture an
already authored composition; the editor correctly rejected it. The fixture now
edits the existing documents while preserving actor/component identities.

Acceptance passes **1,255 workspace tests / 88 ignored**, formatting, Clippy,
debug build, API freshness and five tool tests. The focused native/editor suite
passes three tests. Forty-eight native preview cases cover all three providers,
four starters, two languages and both resolutions with zero dropped primitives.
They retain imported fonts/colors, actual localized pages, name, choice pagination
and the conversation-only portrait in its authored area. A project `present`
override is not executed. The test exposed and fixed the adapter's missing reveal
feature flag; state-only assertions would not have caught its full-text drawing.

Real ImGui input passes conversation/condition selection, time step, repeated
initial-state restart, locale/start node, navigation, completion and event trace
in C++, Blueprint and Lua. Saving and reopening retain the class data and omit
preview inputs. Held confirm, explicit false branches, omitted/all-unavailable
options, cancellation and timing changes pass. The native adapter now preserves
logical hierarchy separately from physical attachment: disabling a transform
parent and disabling a logical parent have their distinct runtime effects.

Twenty-four native 320x240 renders, a full 640x480 Spanish portrait render and six
designer captures were inspected. The designer shows actual translated questions
and the second choice page (25/23 glyphs, zero drops) in all three providers.
Unused Basic remains byte-identical at 968,624 static bytes / 425,984 EXE bytes,
with zero delta. Runtime headers are unchanged, so this editor-only milestone
makes no new console/export claim. Its provenance is
`artifacts/gameplay-helpers/p4/widget-conversation-preview-validation.json`.

P4 remains open for four finished starters, removal of obsolete visual Style
fields and complete authoring/provider/domain/target/export acceptance. P5–P10
retain all characters, four camera rigs including SpringArm, helpers, six-template
three-flavor ports, complete-game rules and the full 30-case matrix.

## Widget-owned starter visuals and timing assets (validation in progress)

The authoring model now contains no Style appearance fields. Style version 2
contains line timing and three optional sounds; version 1 is projected in memory
without rewriting it. Saving preserves the original package outside the asset
catalog before writing v2. Appearance remains on ordinary controls and Texture
assets. The asset Inspector edits timing and sounds, with retained drafts and
stale-write protection. It is not a dialogue appearance designer.

DialoguePresenter adds one reflected default timing record and an optional
ImageComponent reference for a choice popup. An explicit timing asset replaces
the widget defaults. The popup follows selection visibility and the normal
session cleanup lifecycle. The cooked runtime Style likewise contains timing
and sounds only. Pagination uses authored widget bounds/fonts; a low-level
session without a widget has no inferred geometry and must fit its text payload.

All four starters remain ordinary C++, Blueprint or Lua classes. Classic and
portrait frames are nine-slice project textures; subtitles use a low centered
bar; the bubble is a rounded project image with a tail. A separate choice popup
keeps the bubble's anchored content bounded. These starters use 12 actors and
29 components, or 13 actors and 31 components for the anchored bubble. Existing
classes retain their own hierarchy and are not automatically replaced.

Current native C++ host checks pass, including default/explicit timing priority,
invalid default rejection without capture leaks, choice-popup visibility and
anchor invalidation. The refreshed workspace, GUI, visual, console and export
checks are still running; the validated manifest continues to point at the
preceding conversation-preview milestone until they pass.

Console acceptance exposed the existing 16-property hierarchy limit when the
probe derived from the presenter and added three properties. Default timing is
now a two-word DialogueTiming value in one reflected property, preserving three
project property slots without raising the global budget. Refreshed validation
is required before this milestone replaces the preceding manifest.

The native extractor accepts constant authoring values for this SDK configuration
record; runtime result/identity records retain their existing restriction. The Lua
class factory now leaves its sample `speed` assignment commented, matching empty
C++ and Blueprint declarations and retaining the same extension space. Existing
project source is not rewritten. The starter test adds three actual project
properties to every style/provider and validates the inherited timing default.

## Widget-owned appearance validation

The final grouped-timing implementation passes **1,258 workspace tests / 88
ignored**, Clippy with warnings denied, formatting, both debug executables, API
freshness, parity inventory and five tool tests. Native Session/widget host tests
pass. The SDK has 137 headers / 2,414 callables; parity records 795 public gameplay
declarations out of 5,843, with no unclassified declarations.

The updated console fixture is an actual DialoguePresenter subclass with three
project properties and its complete native component set. Long localized text
uses the authored 272x72 body area: four pages, six choices, condition/action
callbacks, completion and capture release pass. Its standalone rebuild is
byte-identical. The helper fixture passes Native C++, VM bytecode and VM source,
using 1,439,600 / 1,758,808 / 1,818,200 static bytes respectively. Its VM-source
standalone rebuild is also byte-identical. These measurements are static
allocation, not runtime heap/stack peaks.

Forty-eight native conversation cases produce 96 renders with zero drops. The
32 style/resolution/language/state groups are pixel-identical across providers.
Three contact sheets (48 native 320x240 renders), two 640x480 renders and sixteen
generic class-designer previews were inspected. Classic/portrait frames, the
solid subtitle bar, the bubble tail, separated choice popups, authored fonts and
colors, translations and selection remain visible. CLI creation protects all
twelve class identities and retains composition bytes through capture.

Unused Basic is byte-identical to the previous validated milestone: 968,624
static bytes / 425,984 EXE bytes, SHA-256
`75e62b43a5e3918fb35dd2d2fcc37a1671008fe81a01c6c2225ce1c631cd61b6`.
Evidence is recorded in `artifacts/gameplay-helpers/p4/widget-style-ownership-validation.json`.
P4 remains in progress for final authoring/provider/domain/target/export
acceptance. P5-P10 and the complete 30-case template matrix remain required.

## P4 release gate

P4 is validated by the requirement audit and `release-validation.json`. The final editor adds generic composition/resource/HUD usage reporting; its native text-size query reports 528-byte Text records with 512-byte buffers. Actual GUI input checks include a conversation-only portrait and imported font. Classic/Bubble captures show 12/29 and 13/31 actors/components respectively, eight text controls and zero drops. These are occupied-slot figures, distinct from linked static RAM.

The final provider matrix runs C++, Blueprint and Lua widget overrides in all three Lua execution modes: nine cases / 162 sessions, covering sources in both domains. VM-source exports for each widget provider rebuild byte-identically. Two new console fixtures use the actual 2D camera service at 320x240 and 640x480; all twenty anchor checks pass. Console runtime files match the preceding milestone exactly; the final editor change affects generic usage reporting only. P5 is enabled and P5-P10 remain required.
