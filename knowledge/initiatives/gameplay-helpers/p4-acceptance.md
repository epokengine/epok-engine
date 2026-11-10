# P4 acceptance audit

Status: validated. All rows below have scoped evidence. P5-P10 remain in scope.
Final evidence: `artifacts/gameplay-helpers/p4/release-validation.json`.

Architecture recheck, 30 October 2026: dialogue presentation uses the common
`UIActor` class designer and editable C++/Blueprint/Lua subclasses of
`DialoguePresenter`. The removed appearance window is historical evidence only;
the ordinary timing asset Inspector contains timing and sounds. Re-ran
`python tests/runtime/verify_spatial.py dialogue_widget dialogue hud_reveal`:
all three native suites passed, including preservation of authored control
geometry, font, colors and panel material. This focused recheck does not replace
the scoped provider/export evidence below.

Architecture recheck, 6 November 2026: the current source still uses the common
UI class designer, with `UIActor` as the general widget base and
`DialoguePresenter` as its dialogue specialization. The ordinary timing
Inspector exposes timing and sounds; appearance remains on the class's child
controls. `cargo test --locked --bin epok-editor dialogue_widget -- --nocapture`
passes all four selected tests, including starter classes in C++, Blueprint and
Lua. `python tests/runtime/verify_spatial.py dialogue_widget dialogue hud_reveal`
passes all three host suites. This is a focused architecture regression check,
not a new full-workspace, console or standalone-export acceptance run.

## Implementation requirements

| Requirement | Authoritative implementation and evidence | Current result |
| --- | --- | --- |
| Graphical conversation editing and localization | `dialogue_editor`, `dialogue_graph`, `dialogue_asset`; real mouse linking, undo/redo, protected removal, duplication, save conflicts and native compile tests | Passed in the workspace suite |
| One widget/class model in C++, Blueprint and Lua | `class_composition`, `class_designer`, shared Template resolver; inheritance, typed bindings, duplication, refresh, nullable resets, components, real drag/undo/save/reopen | Passed in the workspace suite and prior composition console fixtures |
| Four ordinary editable starters | `dialogue_widget` / `dialogue_starter`; 12 real source classes, three additional project properties per class, two viewports, 16 inspected generic-designer captures | Passed; source provenance in `widget-style-ownership-validation.json` |
| Widget-owned appearance, timing and sound configuration | Native presenter changes content/visibility/focus only; Style v2 holds timing/sounds; v1 migration preserves the original package | Passed in native host, Native Play, GUI and migration tests |
| 3D and 2D bubble anchoring with fallback | `ui_anchor` and shared projection services; 3D console probe and 2D/3D host cases | Passed: ten console checks at each resolution, including the actual 2D camera service |
| Shared presentation and visible resource costs | Six sources share one presenter in the integration fixture; generic designer now shows class composition, Text slot storage, resource banks and HUD budgets | Passed: real GUI test and inspected Classic/Bubble captures |
| Deterministic localized preview using the runtime binder | Native Session/input/presenter/HUD compiler, explicit conditions, event trace; 48 cases / 96 renders, zero drops | Passed; actual project callbacks run only in Play |
| Existing automation surfaces | CLI class/asset creation, reflection, source/Blueprint compilation, common UI capture, Native Play and PSX/export verifiers | Passed; no second authoring state or runtime catalog |

## Release acceptance

| Requirement | Evidence and scope | Current result |
| --- | --- | --- |
| Place/configure an object, assign conversation/widget and Play without engine edits | Shared actor/component pickers and typed property editors; real GUI component/reference tests; native interaction/session integration | Passed across the existing authoring and runtime fixtures |
| Four styles, both resolutions, long translations and choice pagination | 48 localized native cases, 96 renders, four-page console fixture, six choices; 16 generic designer captures | Passed |
| Presentation changes preserve conversation semantics | Every starter runs the same graph to the same selected choice/action/completion; geometry/fonts/colors survive binding. Configuration selects the presenter for a session; live editor reload resets Play normally | Passed for the specified configuration workflow; no live-session presenter-transfer API is claimed |
| Derived presentation in all three providers retains parent behavior | The provider widget fixture overrides open/present/close, calls parent binding and verifies geometry, prompt/continuation, focus and cleanup | Passed: nine provider/execution cases and byte-identical VM-source exports for each widget provider |
| Inheritance/duplication/save/reopen/Play preserve child controls | Composition/class designer tests, nested Native Play and console fixtures, shared authored-layout cook and native binder | Passed |
| Complete 2D/3D provider fixtures without blocked controls or HUD drops | Six source components (three providers in each domain), 18 sessions per provider/execution case, held input/cancel/destruction/unload checks | Passed: nine provider/execution cases, 162 sessions in both domains |

Each final evidence record must retain its executable and runtime source hashes.
Editor-only resource reporting does not change the console runtime; the release
record will distinguish that change from the executable used for the provider
matrix. Static linked RAM is measured separately from occupied-slot capacity and
from runtime heap/stack peaks, which remain P8/P10 work.
