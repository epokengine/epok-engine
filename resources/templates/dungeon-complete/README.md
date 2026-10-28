# Dungeon Complete

A complete playable action game set in Reactor Station. Find the access key,
fight the robot guards and defeat the reactor guardian. The entry room contains
a PocketStation link terminal and a trading PC.

## Play

The project starts in the embedded PlayStation emulator, with streamed geometry
read from the PC. Press Play to build and run. The level keeps the validated
320x240 output, retained geometry, two streaming pages and fixed actor pools.
For a disc build select CD on demand. For physical serial Play select PSX via
serial and PC on demand; configure the adapter in the editor's local settings.

| Action | Controller | Keyboard / mouse |
| --- | --- | --- |
| Move | Left stick / D-pad | WASD |
| Camera | Right stick | Mouse |
| Shoot | Square | J / left mouse |
| Free aim | L1 | Q / right mouse |
| Lock / cycle target | R1 | E |
| Jump or interact | Cross | K / Space |
| Inventory | Triangle | I |
| Close menu | Circle | L |
| Retry / restart | Start | Enter |

Taking damage briefly flashes a thin red border around the screen and requests
a 120 ms vibration on Pad 1. The border narrows and dims over 220 ms, leaving the
center of the screen clear. Only a hit that reduces health triggers the feedback;
the existing invulnerability period prevents repeated pulses from blocked hits.
The border also works without a rumble-capable controller. Restart and scene
exit clear both effects. The settings live in `DungeonGame.cpp`.

## PocketStation and trading

Interact with the floating PocketStation, choose the card slot, and select
**Install / start minigame**. Asteroid Catcher occupies one card block. Move
left/right and press the center Fire button to start. Bright asteroids become
ore and dark debris costs a life; catches save immediately. Hold Fire for five
seconds and release at **READY / LET GO** to save and return to the native menu.

**Transfer asteroids** starts the installed miner if it is at the native menu,
then reads its saved ore. The PSX inventory is saved before acknowledging the
PocketStation receipt; repeating a transfer does not award the same ore twice.
The editor emulates PocketStation in slot 1. Hardware supports either slot;
installation, upgrade, exit and transfer were verified with a real device in
slot 2. Keep a memory card in slot 1 for the game inventory.

At the trading PC, sell ore for 10 credits each and buy medkits, energy tanks or
permanent buster upgrades. Open the inventory to use healing items. Economy
progress survives a dungeon restart; an unsuccessful save does not consume an
item or complete a purchase. New template projects contain no saved games.

## Edit

- `assets/scenes/DungeonGame.epokmap`: the complete level and its 64 actors.
- `assets/scripts/DungeonCombat`: C++ gameplay, controller, encounters, UI,
  inventory, shop, game flow and asynchronous PocketStation transactions.
- `assets/PocketStation/AsteroidCatcher.epokasset`: editable Pocket Lua source,
  icons and persistent-save configuration.
- `assets/Characters`: the animated player and robot assets used by the level.
- `assets/Dungeons`: level geometry, textures and the two hub terminals.
- `assets/Audio/SoundEffects` and `assets/DataTables/SoundEffects.epokasset`:
  sounds mapped by named gameplay events.
- `assets/Audio/Music`: the project-provided MIDI and its trimmed SoundFont.
  Only the song's three presets remain; their samples and synthesis settings
  are preserved. The native SPU conversion retains its 40 KiB instrument budget.
- `assets/UI`: gauge, combat sprites and bitmap font.

Combat HUD also carries the initially disabled Damage Border Image. Its
untextured frame uses four HUD rectangles and no additional actors or textures,
so the level keeps its 64-actor pool. The Canvas stretches to the viewport edges.

The game begins with music, waits one second, fades in and unlocks controls two
seconds later. Death locks input, waits two seconds, fades audio and picture out,
waits two more seconds and restarts the level.

This template includes its existing C++ implementation and the Pocket Lua miner.
New C++, Blueprint and Lua components can still be added normally. The separate
Dungeon template remains the procedural starter with three gameplay flavors.

Sound effects are from Kenney's Sci-fi, Impact and Interface packs (CC0).
The SoundFont derives from FluidR3Mono_GM (MIT); the font is Press Start 2P (OFL).
Their notices are retained in `Credits/`. MIDI and character assets are retained
from the source Dungeon project.
