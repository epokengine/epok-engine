# Phase 7: Room variants, columns, three-flavor keys and the Dungeon template

Delivered on `feature/dungeon-generator`.

## What shipped

| Area | Delivered |
| --- | --- |
| Per-room pieces | Style pieces can roll their chance once per room (**Same for the whole room**), from a stream keyed by piece and room; corridor spots still roll alone. |
| Texture variants | The built-in style gives each room one of three floors (steel plates, teal grid, amber-framed grate) and one of three walls (cyan strip panels, wide ribs under an amber band, pipes over a red warning band). The texture tool draws them. |
| Columns | A Column spot marks `(L - 1) / 2` evenly spaced points on each straight run of room wall of `L` cells. Built-in Blockout there raises a textured pilaster out of the wall (front, sides, top without a ceiling) with its own collision box. |
| Keys and doors | Redesigned so nothing is shared but a reference: a key deactivates on its trigger; a door deactivates once the actor its `key` property names is inactive. The key ring and the key's spin are gone, which made the Lua and Blueprint flavors possible: `src/dungeon_gameplay.rs` holds the three flavors' class identities, installs any of them, and tells the generator which one a project holds (C++ is installed when none is). Lua casts to `Actor3D` or `Actor2D` because calls need a concrete class; Blueprint branches on Is Valid before calling `Actor::active`. |
| Dungeon template | `src/dungeon_template.rs`: the Third Person scene with its Environment replaced by a Dungeon Generator (built-in recipe and style, seed 1207, the Player as spawn actor), the key and door of the chosen flavor, a fresh lighting bake, a Scene view over the dungeon and a README naming the flavor's sources. Hub card and preview (the card list now scrolls; `--screenshot-new-project --new-project-template dungeon` captures the Hub with a template selected and scrolled into view), `--template dungeon`, docs. |

## Verification

| Check | Result |
| --- | --- |
| Tests | Per-room rolls give one floor and one wall per room and variety across rooms; columns only in rooms; every face's UVs within 0..1 and every wall, column and pedestal face looking into the room; each flavor's key and door compile through the real C++ catalog, Lua compiler and Blueprint compiler and publish `lock` and `key`; the template in each flavor places the character on the spawn, one key, doors naming it, stays within 64 actors and 3,500 quads, has a valid bake and resolves every class |
| `--build-psx` | Dungeon template: C++ 1.81 MiB, Lua 1.83 MiB, Blueprint 1.84 MiB static RAM |
| PCSX-Redux | In each flavor, with the key staged before the player in the goal room: the door is closed, and one step later the key is gone and the doorway open |

## Limits and follow-ups

- Keys react to any moving collider entering their trigger (phase 8: only the Player).
- Columns stand on rooms' straight walls only; small rooms get few.
- Projects that installed the phase 5 C++ key ring keep it; the generator writes the
  `key` property its doors no longer declare until those scripts are replaced (phase 8:
  unedited copies are replaced when the dungeon is generated).
