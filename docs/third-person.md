# Third Person template

Choose **New project > Third Person** in the Hub, or use:

```powershell
cargo run --locked -- --create-project "D:/Games/Third Person" --template third-person
cargo run --locked -- --create-project "D:/Games/Third Person" --template third-person --gameplay lua
```

The generated project is a lightweight playable example with an animated player, an
obstacle-aware orbit camera and an optimized low-poly arena. It contains no combat,
lock-on, NPC systems or audio.

## Gameplay flavors

The template is the engine's reference comparison between its three authoring systems.
C++, Blueprint and Lua generate the same level, the same actors, the same transforms and
colliders and the same animated character; the project input component is authored
in the selected language.

| Flavor | Generated source |
| --- | --- |
| C++ | `assets/scripts/ThirdPersonController.hpp` and `.cpp` |
| Blueprint | `assets/Blueprints/ThirdPersonController.epokbp` |
| Lua | `assets/scripts/ThirdPersonController.lua` |

The shared-component route passes all five execution cases: C++, Blueprint and
Lua native/VM bytecode/VM source, with 1,200 automatic steps and identical
semantic results. All three flavor exports rebuild byte-identically. The probe
projects use about 1.48 MiB static RAM; their observation components are included
in that cost. See `artifacts/gameplay-helpers/p5/third-person-validation.json` for
the precise scope and hashes. The shared-player extraction also passes all fifteen
foundation, arena and Dungeon routes with exact P5 semantic replays, plus three
byte-identical Third Person exports. Its separate receipt is
`artifacts/gameplay-helpers/p7/player-routes-validation.json`.

A project contains exactly one of them. The other two are never generated as a hidden
fallback. Each project controller derives from `LocalPlayer3DInputComponent` and
activates its typed camera binding. Extend `calculate_command` in the selected
language to add project movement rules. Shared motors, animation and cameras use
the same native implementation across all three authoring systems.

The choice only decides what is written once. Every generated project can add classes in
the other two systems afterwards and use them together in the same scene.

`ProjectSettings/Template.epoksettings` records the chosen source class as the
`controller` role, alongside `player`, `camera`, `locomotion` and `camera_input`.
These roles reference existing class UUIDs. The placed player, mesh, camera and
controller references use ordinary actor/component identities and survive display
name changes.

Template construction separates the shared player/camera assembly from arena
geometry and lighting. The Dungeon starter consumes that assembly with generated
World/Player handles, then supplies its own geometry and lighting. Player reuse
installs the character resources and selected input source directly.

The previous full controllers are [isolated comparison fixtures](../templates/legacy-third-person.md).
Independent Basic projects compile each retained implementation through the native
backend; current starter generation installs the shared-input implementation.

The Lua flavor starts on the project's default Lua execution setting, the ahead-of-time
native mode, which keeps the console build small while the source stays plain Lua.
**Edit > Project Settings** switches the same script to either virtual machine mode;
all three produce a working console build of this controller. The Blueprint flavor
compiles to native console code through the Blueprint backend; the source the author
edits is the graph.

The Player is a `Character3D` with inherited root, collider and motor components,
plus project input and `SkeletalLocomotionComponent`: five of eight component
slots. Its mesh remains a separate child actor. The camera is a
`ThirdPersonCamera` with root, camera, inherited SpringArm and orbit input: four
of eight slots. Its hierarchy group is logical; the rig writes a world-space pose.

### Shared behaviour

All three use the same shared mechanisms and tuning:

| Value | Setting |
| --- | --- |
| 6.0 | Maximum ground speed |
| 2.6 / 6.0 | Walk and run animation cycle speed |
| 540 deg/s | Turn rate towards the movement direction |
| 140 / 70 deg/s | Camera orbit and pitch rate |
| -10 to 55 deg | Camera pitch bounds |
| 5.4 / 1.1 | Camera boom length and height |
| 24.0 / 9.0 | Gravity and jump speed |
| -0.05 | Downward bias while grounded |
| 0.6 | Step height: climbed when blocked, and the drop it follows down without falling |
| 12.0 / 3.0 | Ground and air acceleration response |

Walking off a step or down a flight of stairs, the character probes the ground below
(at most the step height) and puts its feet on it, so it walks down without falling and
landing again on every step.

The shared Q12 kernels and property conversion give every flavor the same raw
values. Movement, turning, state selection and camera queries are implemented by
the same native mechanisms in console and Native Play. See
[characters](character-classes.md), [animation](character-animation.md) and
[camera rigs](camera-rigs.md) for their contracts and extension hooks.

Every flavor binds the camera actor and visual mesh by persistent identity.
The generator resolves the six animation roles into the driver's editable clip
record. Changing actor display names does not affect these references. Motor,
input, animation, camera and boom tuning are normal component properties.

## Controls

| Input | Action |
| --- | --- |
| Left stick / WASD | Camera-relative movement |
| Right stick / mouse | Orbit and pitch the camera |
| Cross / K | Jump |

Analog movement transitions between walk and run. The character also includes idle,
jump, fall and landing states. Keyboard movement is digital and therefore reaches run
speed.

## Scene hierarchy

The generated `Main.epokmap` is grouped for immediate editing:

- **World**
  - **Environment**
    - **Geometry** — seven editable optimized arena meshes
    - **Collision** — floor, walls, platforms and ramps
    - **Props** — three static blue cubes without scripts
    - **Lighting** — directional sun
  - **Gameplay**
    - **Camera**
    - **Player**
      - **Visual** — skeletal character mesh

Actor names describe roles rather than named characters. The model source is Aqua, but
the scene exposes it simply as `Player` and `Visual`.

## Rendering and cache

New Third Person projects use 320 × 240 progressive output and transform interpolation.
Template assets are copied directly into the new project once, whichever flavor is
chosen; generated data is not bundled.
Build, import and Native PC caches live under `.epok/`, which is excluded by the generated
`.gitignore`.
