# Third Person arena

An editable low-poly arena with grey platforms, ramps, blue cubes and a controllable mannequin.
Open the scene, explore with right mouse + WASD, or edit a mesh through the Blockout panel.
Play includes character movement, jumping, collision and an orbiting follow camera through
the project-owned ThirdPersonController. The mannequin uses rigid boxes; skeletal animation
is not attached.

New projects use 320 x 240 progressive output with position interpolation enabled. This
avoids interlaced motion artifacts on physical displays. Resolution remains editable in
Edit > Project Settings > Rendering.

All geometry is generated locally as Epok assets. Curves use a small number of flat
segments, and the grid uses vertex-colored geometry. Each area has a separate editable
mesh package and named face groups. Materials can also use imported textures.
