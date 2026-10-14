# Dungeon

A playable dungeon crawler starter: the Third Person character explores a generated
sci-fi dungeon with rooms in different finishes, columns along the walls, a main path
to the goal, side branches with treasure, a loop, one-way shortcuts and a locked door
whose key waits somewhere else.

## Controls

- Left stick or WASD: move relative to the camera.
- Right stick or mouse: orbit and pitch the camera.
- Cross or K: jump.

Walk into the gold key to pick it up; the red door it opens disappears. Only the
character picks keys up.

## The dungeon

Select **Dungeon** in the Hierarchy to edit its Dungeon Generator in the Inspector:

- **New seed** generates another dungeon from the same recipe; **Generate** rebuilds
  it after a change. Both are one undo step.
- **Recipe** and **Style** are the built-in ones. Press **New** beside either to get an
  editable copy: the recipe decides the route (path lengths, branches, keys and locks),
  the style what every spot becomes (textures, columns, props, actors).
- **Fit sizes to it** sizes cells, walls and doors to the character; **Ceiling** closes
  the dungeon from above.

The meter under the settings shows the PlayStation budget the map will use. See the
Dungeon Generator guide in the Epok documentation for every setting.

## Gameplay source

<!--GAMEPLAY-->

The gameplay flavor chosen when the project was created only decided which starter
implementation was written. Nothing is locked: C++ classes, Blueprints and Lua scripts
can be added to this project at any time and used together in the same scene.
