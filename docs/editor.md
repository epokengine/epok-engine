# Using the editor

Numeric fields support both keyboard entry and mouse adjustment: click and release without dragging to edit the text; hold and drag horizontally to change the value. The pointer shows horizontal arrows during the drag, including vector components, and returns to its normal cursor on release. Enter commits text and Escape cancels text editing. This applies to component values, transforms, scripts, asset tools and numeric editor settings. The combined text/spin control uses Dear ImGui's native click-to-input mode rather than a separate overlay.

Selecting a file in Project shows its metadata in the Inspector with a resizable preview below, separating properties from previews. See [Content Browser](content-browser.md) for supported previews and controls.

**Edit > Project Settings** opens project identity, startup scene, automatic compilation and native resolution options. **Edit > Editor Preferences** opens local navigation and Game/emulator display preferences. Both have category navigation, search and explicit Apply. See [Settings](settings.md).

## Console

Console output is a read-only text area. Every displayed line begins with a local
`YYYY-MM-DD HH:MM:SS.mmm` timestamp; every line of a multiline message keeps the
same event time. Drag to select parts of a line or several lines, double-click to
select a word, use Ctrl+A to select all and Ctrl+C to copy. Typing, paste and
Delete cannot change the output or edit scene objects while the console text is
active. Click a compiler diagnostic, then **Open source** to navigate to its file
and line; **Clear** empties the log. New output preserves the selection when appended.

## Scene navigation

The Hierarchy, Scene, Game, Inspector, Project and Console panels can be docked inside the editor window. Layout is stored in the selected project's `UserSettings/editor-layout-v2.ini`. Use Layout > Default or Window > Reset Layout to restore the initial arrangement. Use File > Projects to save/close the current project and create or open another.

Editor panels and tool windows share a charcoal theme with blue selection and focus accents. Toolbars wrap when space is limited; Inspector labels move above their fields in narrow panels. Project uses a Content Browser with a folder tree, gold folder tiles, breadcrumbs, Favorites and Collections. Settings switches between tiles and a table with asset types and paths. Hover truncated names or status messages to read them in full. The skeletal preview expands with its window. See [Content Browser](content-browser.md) for navigation and file operations.

| Input | Scene action |
| --- | --- |
| Q | Selection tool (outside Blockout) |
| W / E / R | Move / rotate / scale gizmo (outside Blockout) |
| Middle mouse drag | Pan |
| Right mouse drag | Look around in place, using captured relative mouse motion |
| Right mouse held + WASD | Fly |
| Right mouse held + Q / E | Fly down / up |
| Shift during flight | Increase speed |
| Alt + left mouse drag | Orbit around the focus point |
| Wheel | Move forward/backward without changing field of view |
| Right mouse held + wheel | Adjust flight speed (also available as Speed in Scene) |
| Escape | Release camera capture |
| 1 / 2 / 3 in Blockout | Faces / edges / vertices |
| E / Q in Blockout, without right mouse | Extrude / push inward; Q bevels selected edges |
| F | Frame the selected entity |

Gizmo shortcuts are suppressed during flight. Click a mesh to select the nearest visible hit, or click the background to clear selection. Camera and gizmo drags do not change selection. Use Hierarchy to select empty entities and cameras without selectable Scene icons.

### Orientation control

The upper-right **VIEW** control is the Scene camera orientation widget. Its
colored X, Y and Z rows mirror the transform-axis colors; choose `+` or `-` to
look toward the pivot from that signed world axis. The control preserves the
current pivot and distance, keeps a tiny pole offset for a stable orbit basis,
and marks the local Scene view as changed without editing authored scene data.
`Persp` identifies the current perspective authoring view; orthographic camera
snapping is not implemented yet.

Entering camera navigation releases an active text field. Releasing the right mouse button, pressing Escape, or losing application focus releases cursor capture. Geometry shortcuts are suppressed while flying or entering text. See [Blockout](blockout.md) for bevel requirements and editing shortcuts.

Scene is a GPU preview. Game shows the emulated PSX output, including its quantization and ordering-table behavior.

## View modes

Scene opens on one of three authoring modes, selected with the **3D / 2D / UI** buttons
above the viewport.

| Mode | Shows |
| --- | --- |
| 3D | The 3D world: the shaded viewport, its camera controls, grid and gizmos |
| 2D | The 2D world. Camera2D authoring arrives in a later release; the mode already lists this map's 2D actors in the Hierarchy |
| UI | The Canvas / HUD editor described in [HUD and 2D entities](hud.md) |

Switching modes never edits the map and never restarts the 3D preview: the camera,
selection and simulation state are exactly where you left them when you come back.
Leaving UI stops the HUD preview, as it always has.

## Hierarchy and materials

The Hierarchy lists what belongs to the current view mode: 3D shows the 3D world, UI shows
canvases and rect elements, 2D shows 2D actors. Domain-neutral logic and resource objects
appear in every mode, so a procedural UI's controllers, audio and resource holders remain
editable while its preview is visible. An item of another spatial domain that has children
in the current one still appears, greyed out, so branches keep their shape and order. Those
lines are labels only — they cannot be selected, renamed or used as a drop target.

Filtering changes nothing in the document. A selection made in one mode stays selected when
you switch away and comes back highlighted when you return.

Actors authored in the map's document appear below the entities with their own icon, named
after the actor and nested by their logical parent. Selecting one opens the Actor section in
the Inspector: its class, domain, identity, an editable Active checkbox and a read-only list
of its components and authored properties. Adding and removing components arrives in a later
release.

Create an actor from the Actor submenu of any creation menu; its classes are grouped by 3D,
2D, UI and Logic. A new actor is created at the map root with the components its class
declares, or the root component its domain requires when it declares none. The submenu is
empty, with an explanation, in a project whose classes the editor cannot resolve — see
[Actors and components](actors.md) for what the actor model needs.

The first line of the Hierarchy is the **map root**, named after the scene. It is not an
object: clicking it opens **Map Settings**, which shows the map's name, document version and
entity/actor counts, its scene Blueprint and its HUD budget. A map without a scene Blueprint
offers **Create Scene Blueprint**, using the parent chosen there or the project default.
Opening an embedded scene Blueprint for editing arrives in a later release.

**Undo and redo.** Ctrl+Z and Ctrl+Y (or Ctrl+Shift+Z) step through scene edits while the
Hierarchy, Scene or Inspector has focus and the game is not playing; the same steps are in
the Edit menu. The last 32 edits of the open map are kept, and opening another map starts a
fresh history. Component attachment and Blueprint edits keep their own Edit-menu entries and
are undone first when they are the most recent change.

Create cubes or empty entities through GameObject or the Hierarchy context menu. A context menu on an entity can create a child; a menu on empty space creates a root. New entities become selected and their parent branches expand.

Rename by double-clicking the name, pressing F2 or using Rename. Enter or leaving the field confirms; Escape cancels. Empty names are rejected. Duplicate and Delete affect the entire branch.

Drag an entity onto another to reparent it while preserving its world transform. Drop it on the scene header or empty hierarchy space to make it a root. The Parent inspector field offers the same relationship control. Parent > Keep Local changes the parent while retaining local values.

Transforms are relative to the parent. Parent motion, rotation and scale affect descendants in Scene and on PSX, including cameras and scripted empty entities. Up to 32 hierarchy levels are supported; cycles are rejected.

Combining rotation and nonuniform scale can create shear. Rendering preserves it, but a world-preserving reparent that cannot be represented as one Transform is rejected. Zero and negative scales are unsupported.

Mesh Renderer offers per-object color and Unlit, Baked Vertex or Realtime lighting. Removing Mesh Renderer preserves the entity, children, Transform and scripts. Cube is the available primitive. See [Lighting](lighting.md) for materials and shadows.

## Game controls

Click Game to give it keyboard focus. Escape, a tab change or loss of focus releases controller buttons.

| Keyboard | PSX controller |
| --- | --- |
| Arrows or WASD | D-pad |
| I / L / K / J | Triangle / Circle / Cross / Square |
| Q / E | L1 / R1 |
| 1 / 3 | L2 / R2 |
| Enter / Backspace | Start / Select |

The bridge delivers these inputs to the emulated controller. The sample Spinner does not use them, and an Epok C++ input API is not implemented yet.

Pause/Resume controls execution. Step advances one VBlank and pauses the CPU again. A game that renders across several VBlanks may need several steps before its image changes. The emulated image uses nearest filtering and integer scale when space permits.

The external debugger window is hidden after the first frame. Use Debugger or Window > Emulator Debugger to show it. Emulator audio remains handled by PCSX-Redux.

For AI-assisted authoring, enable the optional local server in **Edit > Editor Preferences > AI / MCP**. It exposes scene/component edits, assets, source files, screenshots and emulator controls through MCP. It starts disabled; see [AI assistants / MCP](mcp.md) for connection configuration and revision/Undo behavior.

## Prototype limits

Tags and Layers are not implemented. Entity activation is displayed as a disabled placeholder and does not affect the runtime. Floating panels stay inside the application window. Entity multiselection is not implemented. Scene Undo/Redo covers the open map's last 32 edits, as described above; the Blueprint, mesh and timeline editors keep their own stacks, and [Blockout](blockout.md) provides face/edge/vertex selection and geometry Undo/Redo.

Actors support inline rename, Duplicate, Delete and drag-and-drop reparenting in the Hierarchy, and the Inspector's Add Component popup and per-component Remove follow the class compatibility rules (the root and inherited components cannot be removed). Converting an actor to another class or domain is not offered; the [MCP actor tools](mcp.md#actors) and `--add-actor` provide the same edits for automation.

The initial renderer limits exported positions to +/-128 and scales to 64. It clips triangles against the camera frustum and uses double-buffered ordering tables. Those tables cannot resolve all intersecting geometry. Lighting and HUD have additional [lighting](lighting.md) and [HUD](hud.md) budgets.

## Imported assets and sound

Drop WAV, MP3, FLAC or Ogg Vorbis files under the project’s `assets/` folder. The detection notification opens **Imports**, where each file can be imported with its own conversion settings or omitted. Imported AudioClips appear in Project; selecting one opens its source state, waveform and move/reimport actions. Add **Audio Source** in Inspector, choose a clip, and press Play. See [Assets and audio](assets.md) for external moves, recovery and PSX budgets.
