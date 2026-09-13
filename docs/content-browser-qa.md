# Content Browser verification — 2026-09-10

## Editor responsiveness — 2026-09-12

- Source fingerprinting and timeline/effect discovery run in one read-only worker per project. The editor polls completion without waiting, then uses that catalog for both dependency observation and Blueprint pickers. Scene dependency publication still uses the current editor scene and registry.
- The Inspector and browser use the same per-file preview revision, preventing repeated waveform decoding and interrupted audition. Static model previews render again only after a model/camera change; clipped browser cards skip drawing and preview lookup.
- Measured with `--profile-editor` on an isolated copy of Ironwood (69 asset files), with no compiler running concurrently: 180 orbit frames after 30 warmup frames. Original Debug median/p95 frame interval: 16.71/51.61 ms; original Release: 16.68/41.20 ms. Debug after the fix: 16.69/20.43 ms. Two final Release runs: 16.71/17.59 ms and 16.66/17.49 ms. These are local measurements, not a general FPS guarantee.

- Validation: 79 selected tests passed (source observation/build invalidation, asset/dependency handling, media cache and real ImGui interactions); 14 environment-dependent tests remained ignored. Both editor and header tool built in Debug and Release with locked dependencies. The desktop shortcut now selects Release.

## File Inspector and numeric entry — 2026-09-12

- Two Inspector tests passed: native mesh and raw FBX loading without file writes, isolated preview scene construction, aspect fitting, metadata after source deletion and invalid-file errors.
- Four media-cache tests passed, including the 1024-pixel Inspector image path versus the 192-pixel browser thumbnail path; ten browser tests passed.
- The shared real-ImGui regression passed after adding single-click file selection, numeric click/release text entry, integer/decimal keyboard commits, horizontal numeric dragging without entering text mode, preview orbit/zoom, divider resizing and scene/camera/dirty-state preservation. Existing scene opening, Inspector Blueprint drops and editor interaction checks still pass.
- Debug and Release editor/header-tool binaries built with locked dependencies. Real native captures cover imported models, raw FBX, textures and audio; `images/asset-inspector-model.png` shows the actual GPU preview in the normal workspace.
- Preview state and GPU resources are separate from the project scene. This viewer shows the bind pose for skeletal/animation assets; animation playback remains in the specialized model editor. No new external dependency was added for numeric editing or model rendering.

## Media previews — 2026-09-12

- Four media tests passed: PNG aspect/alpha/size limits, audio snapshot playback after source deletion with saved trim, stereo decoding and post-downmix normalization, cache invalidation/eviction and cancellation of pending playback.
- Ten browser unit tests passed, including migration of existing preferences to hide importable originals while preserving scripts.
- The shared real-ImGui event regression passed, covering the Settings checkbox and persistence, Play/Stop in both tiles and list rows without opening Imports, scene double-clicks and Blueprint drag/drop.
- The Windows audio-output acceptance test passed: a short quiet stereo tone completed, and another buffer stopped early. This explicitly enabled test is otherwise ignored because it uses the host audio device.
- Debug and Release editor/header-tool binaries built with the locked dependencies. Actual editor captures checked default visibility, source/processed badge colors, malformed-image fallback, list controls and 96-pixel compact tiles. Reproduce with `tests/integration/create_content_preview_visual_fixture.py`; see `images/content-browser-previews.png`.
- Audio audition previews the saved import settings before SPU/XA encoding; it does not certify console compression artifacts or looping behavior.

The actual WGPU/ImGui renderer was captured and visually inspected at 1258x342, at 640x342, and inside the normal 1440x900 editor. The supplied screenshot was the design target. The isolated capture and the docked panel call the same implementation.

## Findings and corrections

- Charcoal background, blue tree selection, gold folders, compact source rows, navigation toolbar, search row, content count and bottom controls are present.
- Folder tiles fit seven columns at the reference width. The same fixture supplies the reference's first seven folder names in alphabetical order. Counts reflect actual files: 11 items rather than hardcoded reference text.
- The initial normal dock height clipped folder labels. Default layout now reserves approximately 330px for Project/Console, and tile size adapts when a saved layout provides less vertical space.
- At 640px, toolbar actions wrap and the grid uses fewer columns. Labels remain visible.
- Favorite/project/collection headings use gray backgrounds rather than selected blue. The folder tree uses gold icons. Search buttons and collection creation remain clickable.

## Executed checks

- `cargo test --locked`: 255 passed, 17 ignored using the repository's existing ignore annotations. This run included the first five Content Browser unit tests and the native event regression.
- After adding import, trash and scene-placement coverage, `cargo test --locked project_browser -- --nocapture`: 8 passed.
- After the final dock sizing changes, `cargo test --locked scene_clicks_and_hierarchy_context_menu_use_real_imgui_events -- --nocapture`: passed. The browser section exercises actual ImGui mouse/key events for single selection, Ctrl multiselection, Enter navigation, history, folder drag, the Move Here popup, the disk result and move undo.
- `cargo build --locked`: passed after the final implementation changes.
- `cargo fmt --all -- --check` and `git diff --check`: passed.
- `cargo clippy --locked --all-targets -- -D warnings`: blocked by six existing findings outside the new browser code: two collapsible conditions in `blueprint_editor.rs`, a function argument count in `pipeline.rs`, default-field assignments in `blueprint_action_menu_tests.rs` and `viewport.rs`, and a range loop in an existing `gui.rs` test. No Content Browser findings remained.

The new filesystem tests cover empty folders, collision/escape/self-nesting rejection, identity preservation during moves, independent IDs for copied packages, atomic rejection of unsupported class copies, scoped searches/history/collections, retained trash and restore collisions, external import overwrite rejection, and transactional audio placement into Scene.

## Remaining differences from the reference

This is a working Epok implementation, not a claim of pixel-identical parity. Folder geometry and toolbar icons are independently rendered, with small silhouette/stroke/spacing differences. The screenshot's yellow focus outline is not a permanent decoration. Item counts and project content come from the real project. Asset types, class creation, imports, commands and generated-data inspection use Epok's existing capabilities.

External providers, advanced search expressions, folder imports, class/model duplication outside the dedicated workflows, and live image thumbnails are not implemented here. See the behavior limits in `content-browser.md`.

## Native icons and project startup

The startup follow-up was validated against the previous committed engine plus this change in an isolated checkout because another task was actively editing Blueprint reflection in the shared checkout.

- Full Rust suite: 260 passed, 17 ignored. The UI regression restores a layout with Console selected, checks that Project becomes visible on opening, then verifies Console can still be selected afterward. Hub create/open interactions exercise asynchronous project preparation.
- Label regression covers native compound suffixes, dotted names, source files and folders. Loader coverage checks that work waits for splash presentation and returns invalid-project errors.
- `create_project_startup_visual_fixture.py` uses actual Blueprint, timeline, particle, texture, audio and FBX creation/import commands. Native WGPU captures of tiles, list rows, the docked editor and the loading screen were visually inspected. The ten native asset categories render distinct symbols, including the Skeleton bone icon.
- An invalid `--project` path was captured returning to the Hub with its error visible.
- Formatting and whitespace checks passed for this delivery.

The loader performs project/scene/script/index preparation on a worker. Final UI-state construction and GPU setup remain on the UI thread. The splash uses an indeterminate animation and does not claim percentage-based progress.

## Opening projects with previous build caches

Startup now baselines native external inputs and publishes the fully loaded scene before automatic source observation begins. Old build outputs remain stale, but restoring their provenance does not enqueue Build or focus Console. Regressions cover both scene and external-input cache changes and verify that edits made after opening still request a build.

The ignored `opened_project_remains_idle_with_auto_build_enabled` test was run explicitly against an isolated copy of Ironwood, including its assets and existing build/dependency caches. It remained idle with auto-build enabled. Run it with `EPOK_IDLE_PROJECT` pointing to a disposable project copy and a matching editor/header-tool build; it updates that copy's normal caches.
