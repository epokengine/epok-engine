# Dungeon Complete

A complete playable action game set in Reactor Station. Find the access key,
fight the robot guards and defeat the reactor guardian. The entry room contains
a PocketStation link terminal and a trading PC.

## Budgets

This game fills most of the console's 2 MiB, so it spends RAM deliberately. Its
gameplay code compiles for size; `DungeonGame.cpp`, `DungeonPlayerController.cpp`,
`DungeonPlayerCombat.hpp` and `DungeonEncounter.hpp` run every tick and stay at
speed inside `EPOK_SPEED_BEGIN`/`EPOK_SPEED_END`. The scene's Runtime Spawn Reserve
is six actors, the size of its largest menu. A new menu with more controls needs a
higher reserve; the build reports the exact size it needs.

A frame over one vblank runs two simulation steps. `DungeonGame` and
`DungeonPlayerController` leave presentation for the step that is drawn, using
`epok::time.drawn_step`, and doors reuse their last rule result while nothing changes.
Keep new per-tick presentation behind the same check. A rule that reads a value other
than its arguments should still be pure, or a resting door will not see the change.

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
exit clear both effects. Edit the selected `DungeonPlayerCombat` policy and
`DungeonGameHUDWidget` class to change the feedback and presentation.

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

- `assets/scenes/DungeonGame.epokmap`: the complete level and its 58 actors.
- C++, Blueprint and Lua AOT authoring share the same level. C++ rules live in
  `assets/scripts/DungeonCombat`; Blueprint rules live in `assets/Blueprints/DungeonCombat`;
  Lua rules live in `.lua` files under `assets/scripts/DungeonCombat`.
- Game policies and ordinary HUD, terminal, shop and inventory widget classes
  are editable in the selected language. Shared native files provide data,
  scene bindings and runtime mechanisms. Lua's generated C++ belongs to build
  output; edit the Lua rules directly.
- Widget layouts and child controls use the common UI class designer. Blueprint
  owns its embedded compositions; C++/Lua use the ordinary companion class assets.
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
leaving six actor slots within the level's 64-actor limit. The Canvas stretches
to the viewport edges.

Reactor Station holds music and six independent sound-effect AudioComponents.
DungeonGame binds them and the terminal's interface source through editable
component references. Each channel retains its own clip, volume, priority and
playback lifetime; renaming an actor does not change these sound bindings.

Player Aqua is a Character3D with its inherited Collider and Motor components.
The controller submits commands to that motor for gravity, jumping,
movement and collision response. Motor tuning is editable in its component
Inspector; automatic ticking is disabled because the controller simulates it
after choosing input and facing. Its project rules still choose facing,
attacks, animation and camera modes. `DungeonCamera` derives from
`ThirdPersonCamera` and uses its SpringArm for obstruction handling; the project
class supplies the normal, lock-on and free-aim view proposals. The character's
visual and camera and the camera's target use editable typed references.

Station Collision owns the shared SpatialQueryContextComponent used by game
queries, character movement and camera obstruction. The camera keeps its logical parent under
Reactor Station while applying world-space poses. Its automatic tick is disabled
because the player controller refreshes it after movement. The player retains
two available component slots within the eight-component limit. This level uses
coplanar floors, so its Motor has a zero step height; other projects can enable
the shared stair policy through the same property.

Follow Camera also holds the project-owned Run Flow component. DungeonGame binds
it through the typed `run_flow` reference. Its timing configuration controls the
music lead-in, fade-in, control delay, death hold, fade-out and restart hold;
`reactor_entry_z` sets the boss-entry threshold. The shared project contract is
`DungeonRunFlowBase.hpp`; `DungeonRunFlow.hpp` contains the C++ transitions.
The runtime status is exposed separately from editable tuning. This component
does not alter persistent inventory when it requests a run restart.

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
