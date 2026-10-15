# Phase 8: Keys only the player picks up

Delivered on `feature/dungeon-generator`.

## What shipped

| Area | Delivered |
| --- | --- |
| Keys | Each key has a `player` property; it is picked up only when the actor entering its trigger is that one (anything, while `player` names no live actor). The generator writes its **Player** (the former Move to spawn) into every key, in 3D and 2D and in all three flavors. |
| Reference equality | Lua `==`/`~=` and Blueprint **Equal**/**Not equal** compare two object, actor or component references of any classes by identity; C++ already could. Every reference is one `epok::ObjectId` (one 32-bit word for the Lua VM), so nothing changes at runtime. In the Blueprint editor, wiring a reference widens both pins to Object. Ordering on references and mixing a reference with a number stay rejected. |
| Player field | **Player** in the generator band moves the actor to the spawn room's centre in 2D too (through any parent's position, rotation and scale), and lists the actors of the generator's own domain. With keys but no live Player the band warns; **Use Player** picks an actor called Player and generates again. |
| Upgrades | Unedited copies of earlier key and door scripts (by SHA-256: the phase 5 C++ pair sharing a key ring, and the phase 7 keys in C++, Lua and Blueprint) are replaced when the dungeon is generated; edited copies are kept. The report names what it updated. |

## Verification

| Check | Result |
| --- | --- |
| Tests | Reference equality in the Blueprint IR and the Lua frontend (different classes compare; ordering and numbers are refused); keys without a Player name nobody and the report warns, with a Player every key names it in 3D; in 2D a Player under a moved, turned and scaled parent lands on the Spawn marker and every key names it; the template's key names its character; each flavor's key compiles and publishes `lock` and `player`. |
| Upgrades | `--generate-dungeon` on projects made by phases 5 and 7 replaced their unedited `DungeonKey.hpp` (and the phase 5 `DungeonDoor.hpp`), `DungeonKey.lua` and `DungeonKey.epokbp`. |
| UI | A scripted click on **Use Player** in the band sets the Player, generates again, every key names it and the warning goes away. |
| `--build-psx` | Dungeon template projects: C++ 1.81 MiB, Lua 1.83 MiB, Blueprint 1.84 MiB static RAM (unchanged) |
| PCSX-Redux | In each flavor, the player one step before the key: with the key naming the player, the key is gone and the goal room's door open; with the key naming another live actor (the Spawn marker), the player walks through the key, which stays, and the door stays closed. |

## Limits and follow-ups

- One player per generator; keys any of several players can pick up need a key class
  of the project's own.
- Once the actor a key names is destroyed, anything picks that key up.
