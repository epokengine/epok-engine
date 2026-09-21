# Third Person

A lightweight playable third-person starter project. The scene contains an optimized
low-poly arena, three static blue cubes and one animated player character.

## Controls

- Left stick or WASD: move relative to the camera.
- Right stick or mouse: orbit and pitch the camera.
- Cross or K: jump.

The player transitions between idle, walk, run, jump, fall and land animations. The
camera shortens its boom near level geometry. There is deliberately no combat,
lock-on, NPC logic or audio in this template.

The hierarchy is grouped under World, with separate Environment and Gameplay branches.
Level meshes remain editable, while their optimized packages keep the console build small.
Generated files and caches live under `.epok/`, are excluded from version control and are
recreated when required.

## Gameplay source

<!--GAMEPLAY-->

The gameplay flavor chosen when the project was created only decided which starter
implementation was written. Nothing is locked: C++ classes, Blueprints and Lua scripts
can be added to this project at any time and used together in the same scene.
