# NavLite

NavLite is Epok's navigation system. We develop one implementation in place;
before Epok 1.0, project updates take priority over maintaining parallel legacy
implementations. Cached navigation is regenerated when its inputs change.

## Authoring

1. Add floors, ramps and obstacles with `Collider3DComponent`. Box tops and
   `slope_rise`/`slope_axis` ramps contribute walkable surfaces. Collider ramps
   use the collision solver's world AABB and world X/Z slope convention.
2. For terrain authored as an **EditableMesh**, add `NavigationSurfaceComponent`
   to that actor. Its transformed triangles supply floors, slope tests and bake
   obstruction tests. Disable the enclosing box collider on that same actor;
   a box would fill the space under/around its mesh. Keep colliders for obstacles
   that must physically collide with other gameplay systems. Navigation surfaces
   supply footing to navigation agents; they do not add general triangle physics
   to player controllers, raycasts or projectiles.
3. Choose **Build > Add Navigation Bake Volume**. Position and scale delimit its
   bounds; rotation is conservatively enclosed by a world AABB.
4. Set the common profile on every volume in the map:

   | Property | Meaning | Default |
   |---|---|---:|
   | `spacing` | Distance between grid samples | 0.5 |
   | `radius` | Horizontal half width of the supported footprint | 0.18 |
   | `height` | Required clearance above the feet | 0.5 |
   | `step_height` | Maximum abrupt rise/drop between surface samples | 0.4 |
   | `max_slope` | Maximum walkable inclination, degrees | 45 |
   | `collision_mask` | Collider layers used by the bake | 1 |

5. Select a volume to see a translucent green walkable area, point markers and
   connections. The editor generates and caches this preview automatically,
   even without a saved bake, and clips it to the selected volume. The Scene
   toolbar reports its point count or the reason generation failed. This
   preview does not edit the scene or its materials. Choose **Build > Bake
   Navigation** and save the map to persist a bake. Build/Play also rebakes stale
   data automatically. Mesh geometry, collider ramp parameters, transforms,
   profile settings and authored links participate in the fingerprint.
6. Add `NavigationAgentComponent` and a box collider to an unparented Actor3D.
   Use a foot-origin model and a collider above the feet, no larger than the
   baked profile. Call `move_to(x,y,z)` in C++/Blueprint, or set `move_on_start`
   and the target coordinates. `stop`, `arrived`, `failed` and `status` are also
   reflected. Optional `moving_clip`/`idle_clip` values select animation clips.

Surfaces may be slopes, separate stair treads, or neighboring modular pieces.
The baker samples support across the footprint and along each connection;
unsupported gaps, steep slopes, tall risers, walls and low ceilings reject the
connection. Smaller spacing resolves smaller features but increases node count.
Agents sample the baked surfaces while moving and perform collision-checked
lift/forward/lower motion over steps. Their box rests on the highest supporting
point of its footprint, so a wide box sits slightly above the center of a slope.

## Jumps and climbs

Add `NavigationLinkComponent` to an empty actor. Its origin is the start;
`end_x/end_y/end_z` is an endpoint in that actor's local space. Both endpoints
must project onto walkable nodes within one grid spacing.

- `kind = 1`: jump with a parabolic arc; `arc_height` is its lift above the line
  joining the endpoints.
- `kind = 2`: climb, moving vertically at the start and then horizontally to the
  destination. Place the start far enough from a wall to clear the footprint.
- `duration`: seconds to perform the traversal.
- `bidirectional`: generate a reverse link too. Each direction is validated.

The baker checks the entire trajectory for clearance. Agents execute it over
time, checking live collision, and resume walking after reaching the endpoint.
These are authored movement trajectories; gameplay can supply matching jump or
climb animations. They do not infer jump capability from the character mesh.

## Moving obstacles and rebuilding

Add `NavigationObstacleComponent` to a moving door, crate, or other actor with
an enabled, non-trigger collider. The component is excluded from static bake
geometry and registers its current world bounds at runtime. `registered()`
reports whether one of the 16 obstacle slots was available. Disabling or
destroying it releases that slot and restores affected routes.

The service rebuilds traversable connections incrementally from obstacle
snapshots, then publishes them. Moving obstacles do not restart the rebuild on
every tick. Search and rebuilding share the work budget. Live checks stop an
agent before entering an occupied edge; affected routes are requested again.
This updates connectivity over the baked walking surfaces.

For procedural topology changes, C++ can call `epok::nav::world.rebuild(graph)`
with replacement nodes, surfaces and traversal links. It preserves request
handles and destinations, discards obsolete paths and projects agents again.
The caller owns those arrays and must keep them alive until the next rebuild or
scene change. There is no runtime triangle-to-grid bake: construct the replacement
graph yourself or supply a graph prepared by the editor.

## Agent coordination

`avoid_agents` enables occupancy and next-node reservations, including the baked
footprint. Agents wait for an occupied destination or request another route
around it. With `auto_repath`, blocked movement retries after `repath_delay`
(0.5 seconds), and reports `Blocked` after `blocked_timeout` (5 seconds without
progress; zero disables the timeout). Collision masks should include NPC layers
so physical collision remains the final check between reserved nodes.

Reservations coordinate the eight active requests. They do not create passing
space in a one-agent-wide dead end; such a route can time out. Stationary actors
that have stopped requesting paths should also have colliders, or an obstacle
component if they should affect route search. This is discrete coordination,
not a continuous velocity-based crowd simulation.

## Budgets

| Limit | Value |
|---|---:|
| Nodes | 512 per map |
| Connections per node | 4 grid neighbors + 2 authored traversals |
| Directed jump/climb links | 32 |
| Walkable triangles | 4096 |
| Active requests / waypoints per request | 8 / 64 |
| Dynamic obstacle slots | 16 |
| Shared search/rebuild work units per frame | 64 by default |
| Node storage | 18 bytes |
| Service workspace | approximately 5.3 KiB |

`world.budget` controls shared work. `stats.work`, `expanded`, `completed`,
`rejected`, `rebuilt` and `yields` expose counters. Surface queries and collision
movement have additional bounded costs outside the search budget. Use simple
navigation meshes for dense visual terrain. Bake capacity errors are explicit;
data is never silently truncated.

Search uses breadth-first traversal and minimizes connection count, including
authored links; it does not optimize travel time. Endpoints snap within one
spacing unit using L1 distance. Arrival means reaching that projected node.
Scene transitions reset graph, obstacles, reservations and request handles.

## Verification

```text
cargo test --bin epok-editor navigation::tests
python tests/runtime/verify_spatial.py navigation navigation_motion gameplay_api
epok-editor --project <project> --bake-navigation --scene assets/scenes/YourMap.epokmap
epok-editor --project <project> --play-psx --scene assets/scenes/YourMap.epokmap
```

The tests cover slopes and their limits, stairs in both directions, modular
floors, terrain triangles, gaps, ceilings, thin walls, directed links, trajectory
obstruction, moving obstacles, reopening routes, reservations, topology
replacement, bounded work, real collision movement and traversal completion.
