# Terrain

Terrain is a heightmap surface you sculpt and paint in the viewport. It is an
authoring layer, not a second renderer: the grid is baked into the same chunk
format as blockout geometry, so it inherits chunk culling, retained packets,
precomputed visibility, geometry streaming and the lighting bake without a
separate runtime path.

Create one from **GameObject > Terrain** or **Project > Add > Terrain**. The
asset lands under `assets/Terrain/` and an actor that renders it is added to the
scene with baked lighting and static geometry already on.

## The grid

A terrain is a grid of square cells. Heights live on the cell corners,
materials on the cells. The whole grid is centred on its actor.

| Control | Default | Range | Meaning |
| --- | --- | --- | --- |
| Cells X / Z | 32 x 32 | 1–256 per axis | Grid resolution |
| Cell size | 4.0 | 0.25–15.0 | World units per cell |
| Atlas columns / rows | 1 x 1 | 1–8 each, 64 tiles total | How the surface texture is divided |
| Merge flat cells | 0 | 0–3 | Largest power-of-two block the baker may collapse |
| Heightfield collision | On | — | Cook a collider that samples the grid |

Heights are stored in Q8, which is one 256th of a unit and spans exactly the
±128 local units a chunk vertex can encode. The authored payload is binary, so
a 32 x 32 terrain is about six kilobytes: a brush stroke rewrites that, not a
mesh document, and undo can afford a snapshot of the whole grid per stroke.

## Sculpting

Open the Terrain window and drag in the Scene view. One stroke is one undo
entry and one save, taken when the mouse is released.

| Brush | Key | Effect |
| --- | --- | --- |
| Raise / Lower | 1 / 2 | Adds or subtracts strength per applied sample |
| Smooth | 3 | Averages each corner with its neighbours |
| Flatten | 4 | Levels towards the height under the first click |
| Set | 5 | Drives towards an absolute target height |
| Noise | 6 | Adds deterministic value noise; the same seed replays exactly |
| Paint | 7 | Assigns an atlas tile to the cells under the disc |

`[` and `]` resize the brush. Falloff is Smooth, Linear, Sharp or Constant;
Constant has no rim, which is how flat platforms and hard edges are cut.

The brush is drawn on the surface as a green disc that follows the relief,
brighter where the falloff bites hardest, with a rim at the radius. Brushes and
falloffs are icon strips with tooltips; the active one is highlighted.

While the terrain tool is stroking it owns the viewport, so selection and the
transform gizmo stay out of the way. Close the window to get them back.

The Blockout window has the same brush under **Build / Edit > Sculpt
selection**, applied to authored vertices instead of a grid. It displaces along
Y under the brush and splits any quad the displacement bends, because a
blockout face has to stay planar. Use it to bend existing geometry; use Terrain
for ground you want to paint and stroke.

## Surfaces

A new terrain adopts the engine's own ground atlas, so it draws grass
immediately and a path can be painted without sourcing art first. The atlas is
written into the project at `assets/Terrain/EpokTerrainAtlas.png` the first
time a terrain is created, and behaves like any imported texture afterwards:
inspect it, reimport it, or replace it in place with your own art.

It holds sixteen 64-pixel tiles in one 256-pixel page, one material per row:

| Row | Material | Columns |
| --- | --- | --- |
| 0 | Grass, the base | Four variants |
| 1 | Dirt, for paths | Fill, edge, corner, inner corner |
| 2 | Stone | Fill, edge, corner, inner corner |
| 3 | Water | Fill, edge, corner, inner corner |

## Autotiling

With **Autotile borders** on, the painted byte is a material row and the baker
picks the column and the rotation from each cell's four edge neighbours. You
paint a path; its borders resolve themselves.

The transition tiles are painted art, not a runtime blend: the console has no
multitexturing, so a material can only change at a cell boundary and the
gradient has to live inside the tile. That is why an autotiled atlas must be
four tiles wide, with row 0 as the base every overlay transitions onto.

Rotation follows the renderer. At rotation 0 a tile is seen with its top edge
toward -Z, and each step turns it a quarter clockwise, so the base material
sits to the north, east, south or west. Transition art is drawn with the base
at the top, and at the top-right for the corner shapes.

Four shapes per material is a deliberate reduction, and two cases fall outside
it. A cell whose opposite neighbours both differ is a one-cell strip, and an
isolated cell has no border at all; both stay filled rather than take a
mis-rotated border. A cell surrounded by its own material but with several
differing diagonals can only show one inner corner. Widen the feature by a
cell where that matters.

Merging compares resolved tiles, not painted materials, so two cells of one
material that resolved to different borders never merge into one stretched
quad.

Turn autotiling off for an atlas of your own that is not laid out this way;
the painted byte is then the tile index, and the Surface tab shows the whole
atlas as a clickable grid laid out like the texture.

One material covers the whole terrain, so the ground is a single texture page.
Painting picks a tile inside that texture's atlas rather than swapping
materials. A 2 x 2 atlas on a 256-pixel texture gives four 128-pixel tiles; an
8 x 8 atlas gives 64 tiles of 32 pixels.

Tiles are inset by half a texel, because the console's sampler has no clamp and
adjacent tiles would otherwise bleed into each other. Tile rotation is fixed
per stroke, or **Scatter**, which picks a deterministic rotation per cell so a
repeating ground texture stops reading as a grid.

There is no per-pixel blending between tiles: the hardware has no
multitexturing. Author transitions as their own tiles in the atlas, and use
baked corner colours for the rest.

Lighting is baked. A terrain actor is created with **Baked** lighting because
that is what terrain wants: per-corner light and ambient occlusion computed
once, costing nothing per frame, with fog modulating it at runtime. The quad
normal is the average of its four corner normals, which softens the boundary
between adjacent cells; true per-corner shading would need a normal per corner,
which the cooked quad format does not carry.

## Budgets

The Budget tab reports the figures that decide whether a terrain fits.

* **Quads.** One per cell, before merging. The whole scene has a resident
  budget of 3500 quads (7000 triangles); terrain competes for it with every
  other mesh. A 32 x 32 terrain is 1024 quads and leaves room for props.
* **Chunks.** Draw chunks are what the per-chunk bounds test pays for every
  frame, and a fine grid produces many of them. A chunk holds at most 96 quads
  and must span under 16 units, so the baker uses the largest block of cells
  that fits: three cells at 4 units, seven at 2 units, fifteen at 1 unit.
* **Collider.** Two bytes per grid corner. A 32 x 32 terrain costs about 2 KiB.
* **RAM.** Measured on the sample project, replacing its ground slab with a
  32 x 32 terrain at 4 units, baked lighting and heightfield collision on,
  raised static RAM from 899 KiB to 1.24 MiB out of the console's 2 MiB. That
  is roughly 370 bytes per quad: the cooked quad and its positions, the baked
  corner colours, and two retained packets per triangle per field. Turning
  **Retained Packets** off in Project Settings trades that back for per-frame
  packet building. Measure your own scene; this is one configuration.

Cells are alabeado — a cell's four corners need not be coplanar. The renderer
splits every quad into two triangles anyway, so a bent cell costs the same as a
flat one and half of what two triangle faces would.

### Merging

Merging collapses a block of coplanar cells that share a tile into one quad. It
is the only decimation the cooked format allows, because geometry is baked into
fixed arrays and nothing can be decimated at runtime. On ground with flat
plains and uniform slopes it removes a large fraction of the quads at no visual
cost.

It stretches the tile across the merged cells, which is why it is off by
default. A merged quad never straddles two chunks, and cells with different
tiles never merge.

## Limits

* A terrain spans at most 256 units per axis. Beyond that a vertex leaves the
  local window the chunk compiler encodes.
* A single cell must stay within about 16 units on every axis, including
  height. Author a cliff as several cells rather than one tall step; the build
  names the offending cell if you do not.
* Heightfield collision indexes the grid from the collider box's world minimum,
  so the actor must be translated only — no rotation, no scale. Change the cell
  size instead of scaling the actor. A rotated terrain is still drawn; the
  build only refuses to cook a collider for it.
* The heightfield is a surface to stand on, never an obstacle, exactly like the
  ramp collider. A character is lifted onto it rather than pushed sideways by
  it, which is what lets a slope be walked up from any side.
* Terrain is 2.5D. Caves, overhangs and holes are blockout geometry.

## Navigation and streaming

Add a **NavigationSurfaceComponent** to a terrain actor to feed its walkable
surface into the navigation bake. The bake consumes the same quads the renderer
draws, so a merged cell contributes one wide triangle pair rather than the
cells it replaced, and the 4096-triangle bake limit applies. A terrain's own
heightfield collider is exempt from the rule that a navigation surface must
have its box collider disabled, because it is a surface rather than an
obstacle.

Geometry streaming covers terrain exactly as it covers editable meshes: chunks
are packed into 64 KiB pages and read on demand. See
[Geometry streaming](streaming.md).
