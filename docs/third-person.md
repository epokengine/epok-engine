# Third Person template

Choose **New project > Third Person** in the Hub, or use:

```powershell
cargo run --locked -- --create-project "D:/Games/Third Person" --template third-person
```

The generated project is a lightweight playable example with an animated player, an
obstacle-aware orbit camera and an optimized low-poly arena. It contains no combat,
lock-on, NPC systems or audio.

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
Template assets are copied directly into the new project; generated data is not bundled.
Build, import and Native PC caches live under `.epok/`, which is excluded by the generated
`.gitignore`.
