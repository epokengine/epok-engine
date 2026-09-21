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
colliders and the same animated character; only the class the Player binds differs.

| Flavor | Generated source | Console build |
| --- | --- | --- |
| C++ | `assets/scripts/ThirdPersonController.hpp` and `.cpp`, plus `CharacterMotion.hpp` and `CharacterClips.hpp` | 522 KiB executable, 1.38 MiB static RAM |
| Blueprint | `assets/Blueprints/ThirdPersonController.epokbp` | 552 KiB executable, 1.41 MiB static RAM |
| Lua | `assets/scripts/ThirdPersonController.lua` | 540 KiB executable, 1.40 MiB static RAM |

Those sizes are the whole game, not the controller: the arena, the character, the
animation and the runtime are the rest of it.

A project contains exactly one of them. The other two are never generated as a hidden
fallback, and no flavor calls another's implementation: the Blueprint graph and the Lua
script are complete controllers built from the same reflected engine operations the C++
uses, so the three are a genuine comparison rather than three wrappers.

The choice only decides what is written once. Every generated project can add classes in
the other two systems afterwards and use them together in the same scene.

The Lua flavor starts on the project's default Lua execution setting, the ahead-of-time
native mode, which keeps the console build small while the source stays plain Lua.
**Edit > Project Settings** switches the same script to either virtual machine mode;
all three produce a working console build of this controller. The Blueprint flavor
compiles to native console code through the Blueprint backend; the source the author
edits is the graph.

The Lua controller binds the camera's transform component rather than the camera actor,
and writes it with three scalars. That is what keeps it free of whole-vector locals,
which the virtual machine's value boundary cannot carry.

### Shared behaviour

All three implement the same contract, with the same tuning:

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
| 0.6 | Step height |
| 12.0 / 3.0 | Ground and air acceleration response |

Every one of those numbers is exactly representable in the engine's Q12 fixed point, so
the three flavors reach identical raw values rather than merely similar ones. The shared
operations they are combined with — `sine_degrees`, `cosine_degrees`, `square_root`,
`length2`, `wrap_degrees`, `delta_degrees`, `move_toward`, `move_toward_degrees`,
`heading_degrees` and `stick_intent` — are engine functions available to all three
languages, so none of them needs host floating point and the console result matches the
desktop one.

The Blueprint and Lua controllers bind the Camera actor and the Visual mesh through typed
references the template fills in when the project is created, and receive their six
animation clip indices the same way. Neither looks an actor up by display name, so
renaming anything in the scene is safe.

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
