# HUD and 2D entities

Create Canvas, Panel, Image, Text or Progress Bar through GameObject > UI or Hierarchy's context menu. Epok adds a Canvas automatically when necessary. These are ordinary scene entities with hierarchy, scripts, renaming, duplication and persistence.

Panel combines RectTransform and Image. Text and ProgressBar are independent graphics components. Add Component can add Canvas to an empty root entity, RectTransform to a UI child, and graphics to an entity with RectTransform.

## Layout

Scene's **UI** mode opens Canvas editing. It is one of the three authoring modes — **3D**, **2D** and **UI** — selected above the viewport; see [View modes](editor.md#view-modes). Select an element to drag it, or resize it from its lower-right corner. Inspector exposes Anchor Min/Max, Pivot, Position and Size Delta with anchor presets.

The Hierarchy in UI mode lists this map's canvases and rect elements together with its UI actors. Domain-neutral Empty objects, scripts, audio and resource holders remain visible and editable here; this keeps the authored side of a procedural HUD available beside its preview. Runtime-only elements created by a script are not added to the authored Hierarchy. An element of another spatial domain that still has UI children appears greyed out so the branch keeps its shape; those lines are labels only.

Anchors and pivots range from 0 to 1. Positions use reference pixels with +Y pointing up. Size Delta adds to the space between anchors so elements can stretch with their parent. Reparenting with world preservation keeps the visible rectangle.

The Canvas mode is **Screen Space - Overlay** at the resolution selected in [Project Settings](settings.md), 640 x 480 by default. Positions and sizes use native pixels; anchors follow the canvas size. PSX draws it after the 3D world. The 3D Transform does not change RectTransform layout. Draw order is parents before children and siblings in creation order.

Graphics clip to the Canvas, not their parent's rectangle. Text is limited to its own width. Disabling Canvas hides its subtree; graphics also have Enabled flags.

## Automatic layout

**Horizontal Box**, **Vertical Box**, **Grid**, **Margin** and **Center** are rects carrying a **Layout Container**. A container measures its children and assigns their rectangles, so a list of items needs no hand-placed coordinates. A child of a container ignores its own anchors and Position; everything outside a container resolves from anchors exactly as before, so existing maps are unaffected.

Every child of a container is measured by one rule, per axis: the larger of what its content needs, its own Size Delta, and its Layout Element Minimum. Content means the space unwrapped text occupies for a label, and — for a box that is itself a container — the size its own children add up to. So a nested Vertical Box whose items are 60 wide but whose Size Delta is 150 asks for 150; set that Size Delta to zero and it follows its content instead. Size Delta is a floor, never a ceiling.

The container then hands each child a cell: a Horizontal Box divides its width left to right, a Vertical Box its height top to bottom, and a Grid fills `Columns` cells per row starting at the top-left. Margin and Center give their children the rectangle shrunk by the four padding values, in left, top, right, bottom order — the order Image's nine-slice borders use.

A container measures and places exactly the children it draws. Clearing an element's Active flag takes it out of the list entirely: the items after it move up and the box measures smaller, rather than a hole being left where it used to be.

**Layout Element** decides what the child does inside that cell, per axis. **Fill** takes the whole cell; **Expand** claims a share of the space left over after every minimum is satisfied, divided in proportion to **Stretch**; **Shrink Center** and **Shrink End** keep the measured size and move it within the cell. With no Layout Element a child fills its cell without expanding, which packs a box tightly against its first edge.

Layout runs in the same fixed-point pass the console uses, so the editor viewport, the native preview and the PSX build agree on every rectangle. Each laid-out rect still costs one entry of the scene's `hud_budget.layouts`.

## UI actors

A map can also hold **UI actors**: classes deriving from `epok::UIActor`, whose root
component is a `epok::RectTransformComponent` and which follow the same anchored
layout rules as the entities above. They are authored from the Actor submenu, appear
in the UI mode of the Hierarchy, and may carry any component whose owners include the
UI domain — `epok::AudioComponent` among them, which is how a menu sound and a world
sound end up being the same kind of thing. See
[Actors and components](actors.md) for the class model and its current limits.

Canvas entities and UI actors coexist in one map. Nothing about the entities above
changes, and the HUD budget covers both.

## Native component access

```cpp
if (auto* bar = entity().get<epok::ProgressBar>())
    bar->value = 0.5;
if (auto* label = epok::find_entity("Score"))
    if (auto* text = label->get<epok::Text>())
        text->set_text("SCORE 00100");
```

Entities and components can also be created in C++:

```cpp
auto* canvas = epok::create_entity("HUD");
if (!canvas) return;
canvas->add<epok::Canvas>();

auto* label = epok::create_entity("Score", canvas);
if (!label) return;
auto& rect = label->add<epok::RectTransform>();
rect.anchor_min[0] = rect.anchor_max[0] = rect.pivot[0] = 0.0;
rect.anchor_min[1] = rect.anchor_max[1] = rect.pivot[1] = 1.0;
rect.position[0] = 12.0;
rect.position[1] = -12.0;
rect.size[0] = 160.0;
rect.size[1] = 16.0;
label->add<epok::Text>().set_text("SCORE 00000");
```

Add RectTransform before graphics, beneath a Canvas or another RectTransform. Use `EntityHandle` for references that must detect destruction or scene changes. `destroy_entity` releases a runtime slot for reuse; creation returns null when all 32 additional slots are occupied. Play changes are not saved into authored scene files.

## Current limits

Image supports an imported Texture UUID and an atlas rectangle in pixels. Zero width/height selects the remaining texture from the specified origin. Transparent texels reveal previously drawn content. Optional left/top/right/bottom nine-slice borders preserve corner sizes while stretching the center; small destination rectangles shrink borders proportionally.

Text uses the attributed mig68000 8 × 16 bitmap with derived Spanish glyphs: `áéíóúüñÁÉÍÓÚÜÑ¿¡`. It accepts up to 511 UTF-8 bytes, explicit newlines and optional character wrapping. Text clips to its own complete glyph cells and the screen. ProgressBar retains its fill/background colors.

The scene's `hud_budget` configures layouts, rectangles, text components and glyphs. Defaults are 128 / 256 / 64 / 1024; maximums are 128 / 512 / 64 / 2048. Edit them in **Map Settings**, opened by clicking the map root at the top of the Hierarchy; Project Settings > Rendering keeps the heading and a button that opens it. Nine-slice images budget nine rectangles. The editor rejects authored over-budget scenes; dynamic excess is omitted and counted in `hud_stats.dropped`. Standalone builds reserve the maximum requested by their registered scenes.

`utility.hpp` supplies generic `Focus`, `EventQueue`, and `layout_list` APIs. Focus skips inactive/destroyed handles, supports forward/reverse navigation, and emits activation/cancel events. These are primitives for the game's UI logic. Parent masks, rotated rectangles, scalable text and world-space Canvas remain outside this profile.

The editor and PSX use the same generated bitmap data. Font and derived glyph attribution is preserved in [third-party notices](../THIRD_PARTY_NOTICES.md).
