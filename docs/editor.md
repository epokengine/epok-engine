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

Build, Play and connection failures open Console with a persistent red error
message above the selectable output. The log also records an `ERROR:` entry with
the event time. **Clear** dismisses the message; starting a new build/Play clears
the previous message. Serial runtime reports include unexpected Unirom halts,
reported CPU exceptions, PsyQo aborts and Lua callback errors. A requested pause
is not an error. See [physical PSX diagnostics](play.md#physical-psx).

## Scene navigation

The Hierarchy, Scene, Game, Code, Inspector, Project and Console panels can be docked inside the editor window. **Window > Code Editor** opens the integrated [Lua/C++ workspace](code-editor.md) with a script tree, tabs and language service setup. Layout is stored in the selected project's `UserSettings/editor-layout-v2.ini`. Use Layout > Default or Window > Reset Layout to restore the initial arrangement. Use File > Projects to save/close the current project and create or open another.

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
| Wheel | Move the camera forward/backward along its viewing direction, using Speed, without a zoom limit or field-of-view change |
| Right mouse held + wheel | Adjust Camera Speed (also available from the camera dropdown in Scene) |
| Escape | Release camera capture |
| 1 / 2 / 3 in Blockout | Faces / edges / vertices |
| E / Q in Blockout, without right mouse | Extrude / push inward; Q bevels selected edges |
| F | Frame the selected entity |

Gizmo shortcuts are suppressed during flight. Click a mesh to select the nearest visible hit, or click the background to clear selection. Camera and gizmo drags do not change selection. Use Hierarchy to select empty entities and cameras without selectable Scene icons.

The camera button at the upper-right of the Scene toolbar opens **Camera Speed**,
which sets how fast the Scene camera flies, pans and dollies.
Its slider runs from **1 to 8**, with **4** retaining Epok's default of five world
units per second. Each level doubles the speed; fractional values allow finer
adjustment. **Camera Speed Scalar** multiplies it by **1 to 128**.
**Distance Based Camera Speed** optionally scales movement by the eye-to-pivot
distance, relative to the default 12-unit distance. The menu displays the resulting
world units per second; Shift temporarily multiplies that by four.
RMB + wheel updates the same speed without moving the camera. Wheel dolly uses
the selected speed too. These choices are saved locally per map and survive
**Reset View**; **Reset Speed** restores navigation defaults. Set defaults for new
views under **Edit > Editor Preferences > Viewports**. Saved views from older
versions preserve their existing movement speed.

The top **Build** menu contains **Build Project**, **Build Lighting**, packaging
and export actions, and **Lighting Settings**. Scene edits retain baked shadows;
an amber warning indicates when to run **Build Lighting** again.

### Orientation control

The upper-right Scene camera orientation widget shows colored X, Y and Z
endpoints. Click an endpoint to look toward the pivot from that signed world
axis, or click the central cube to return to a three-quarter view. Hovering the
widget reveals a translucent circular area: hold the left mouse button and drag
anywhere inside it to orbit freely around the current pivot. Dragging an endpoint
or the cube also orbits; alignment happens only on a click without dragging.
The drag continues outside the widget until release. Escape cancels the gesture
and restores its starting orientation; losing focus stops it.

The control preserves the current pivot, distance and lens, keeps a tiny pole
offset for a stable orbit basis, and saves only the local Scene view without
editing authored scene data. Orthographic camera snapping is not implemented yet.

Entering camera navigation releases an active text field. Releasing the right mouse button, pressing Escape, or losing application focus releases cursor capture. Geometry shortcuts are suppressed while flying or entering text. See [Blockout](blockout.md) for bevel requirements and editing shortcuts.

Scene is a GPU preview. Game shows the emulated PSX output, including its quantization and ordering-table behavior.

## View modes

Scene opens on one of three authoring modes, selected with the **3D / 2D / UI** buttons
above the viewport.

| Mode | Shows |
| --- | --- |
| 3D | The 3D world: the shaded viewport, its camera controls, grid and gizmos |
| 2D | The 2D world editor: select/move/rotate/scale tools, snapping, rulers, real sprite, tile and collider previews, camera bounds, and tile painting with a palette and TileSet editor. See [2D worlds](2d.md) |
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

The creation menu — the Hierarchy's **+** button, its right-click menus and **GameObject** —
starts with ready-made actors whose components are already set up, so a light, a camera or
an emitter never has to be assembled from an empty actor:

| Entry | Creates |
| --- | --- |
| **Create Empty** | an actor with only a transform |
| **3D Object** | **Cube**, **Blockout Mesh** (with its own editable mesh), **Sprite (Billboard)**, **Terrain**, **Dungeon Generator** |
| **2D Object** | every 2D node: Sprite2D, Camera2D, CharacterBody2D, TileMapLayer and the rest (see [2D worlds](2d.md)) |
| **Light** | **Directional Light**, **Point Light** |
| **Audio** | **Audio Source** |
| **Effects** | **Particle Emitter**, **Particle Effect**, **Blob Shadow** |
| **Physics** | **Box Collider**, **Trigger Volume** (a collider that detects without blocking) |
| **UI** | Canvas, Panel, Image, Label, Text Area, Progress Bar, Button, Check Box, Slider and the layout containers |
| **Camera** | a camera the game can render through |

Every group is offered in every view; creating something of another domain switches the
Scene window to the view that shows it. A 3D actor created at the map root stands at the
Scene view's pivot, and the new actor is selected. Each entry is one undo step, except that
a Dungeon Generator's first generated layout is a second one. Hovering an entry describes
what it creates.

Below them, **Instantiate Actor...** picks a class or Blueprint in a dialog, and the
**Actor** submenu lists every placeable class, grouped by 3D, 2D, UI and Logic, for an
actor with exactly the components its class declares (or the root component its domain
requires when it declares none). That submenu is empty, with an explanation, in a project
whose classes the editor cannot resolve — see [Actors and components](actors.md) for what
the actor model needs.

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

A context menu on an actor creates a child; a menu on empty space or on the map root creates a root.

Click an actor to select it alone. Ctrl+click adds an actor to the selection or takes it out, and Shift+click selects every row shown between the last actor clicked without Shift and the clicked one (with Ctrl held as well, the range is added to the selection). Every selected actor is highlighted. The most recently clicked one is the primary actor: the Inspector, the gizmos and the single-actor tools follow it.

The context menu on an actor lists **Rename** (F2), **Duplicate** (Ctrl+D), **Delete** (Del) and **Select Children** first, then **Move to Map Root**, **Convert to Actor Blueprint...** and **Save as Dungeon Room...**, then the creation items. Right-clicking a selected actor keeps the selection, and Duplicate, Delete and Move to Map Root then act on every selected actor, each as one undo step; right-clicking any other actor selects it alone first. Ctrl+D, the Del key and the Edit menu's Duplicate and Delete act on the whole selection the same way. A selected actor whose parent is selected too is copied once, with its parent's branch, and the copies become the selection. Select Children selects the clicked actor and every actor under it. Everything created from the menu (a ready-made actor, Instantiate Child Actor..., a class from **Actor**) becomes that actor's child in one undo step; a parent of the same domain carries the child at its origin. UI widgets go under the actor when it is a UI element, otherwise under a Canvas. Select Children and Save as Dungeon Room stay visible but disabled on an actor without children.

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

The bridge delivers these inputs to the emulated controller. Gameplay can read
buttons and axes and request finite motor pulses through the C++, Blueprint and
Lua [Input API](input-collision.md). Native PC device assignment and the short
rumble test are under Project Settings > Controls.

Pause/Resume controls execution. Step advances one VBlank and pauses the CPU again. A game that renders across several VBlanks may need several steps before its image changes. The emulated image uses nearest filtering and integer scale when space permits.

The external debugger window is hidden after the first frame. Use Debugger or Window > Emulator Debugger to show it. Emulator audio remains handled by PCSX-Redux.

For external authoring tools, enable the optional local server in **Edit > Editor Preferences > Integrations / MCP**. It exposes scene/component edits, assets, source files, screenshots and emulator controls through MCP. It starts disabled; see [Integrations / MCP](mcp.md) for connection configuration and revision/Undo behavior.

## Prototype limits

Tags and Layers are not implemented. Entity activation is displayed as a disabled placeholder and does not affect the runtime. Floating panels stay inside the application window. Several actors can be selected in the Hierarchy, but dragging moves only the dragged actor, the Inspector edits only the primary actor and the Scene view highlights only the primary actor. Scene Undo/Redo covers the open map's last 32 edits, as described above; the Blueprint, mesh and timeline editors keep their own stacks, and [Blockout](blockout.md) provides face/edge/vertex selection and geometry Undo/Redo.

Actors support inline rename, Duplicate, Delete and drag-and-drop reparenting in the Hierarchy, and the Inspector's Add Component popup and per-component Remove follow the class compatibility rules (the root and inherited components cannot be removed). Converting an actor to another class or domain is not offered; the [MCP actor tools](mcp.md#actors) and `--add-actor` provide the same edits for automation.

Authored local transforms remain within +/-10000; exported scales and inherited basis coefficients are limited to 64. Inherited world positions must fit signed 32-bit Q12. Rendering subtracts the camera's world origin before rotation; the camera-relative far plane and the GTE's 16-bit vertex inputs do not limit absolute map positions to +/-128. It clips triangles against the camera frustum and uses double-buffered ordering tables. Those tables cannot resolve all intersecting geometry. Lighting and HUD have additional [lighting](lighting.md) and [HUD](hud.md) budgets.

## Imported assets and sound

Drop WAV, MP3, FLAC or Ogg Vorbis files under the project’s `assets/` folder. The detection notification opens **Imports**, where each file can be imported with its own conversion settings or omitted. Imported AudioClips appear in Project; selecting one opens its source state, waveform and move/reimport actions. Add **Audio Source** in Inspector, choose a clip, and press Play. See [Assets and audio](assets.md) for external moves, recovery and PSX budgets.
