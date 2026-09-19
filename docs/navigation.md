# NavLite

NavLite is a baked, fixed-capacity walking graph rather than a polygon navigation mesh. Search and movement are separate from decisions such as patrol, chase and flee.

## Authoring

1. Add static `Collider3DComponent` floors and obstacles. Navigation uses their **world AABBs**, just like Epok collision, not render triangles. Triggers are excluded.
2. Choose **Build > Add Navigation Bake Volume**. Its actor's position and scale delimit a unit cube; rotation is enclosed conservatively by a world AABB.
3. In its `NavigationBakeVolumeComponent`, set `spacing`, `radius`, `height`, and `collision_mask`. All volumes in a map share one profile. Keep the actual agent's world collider footprint inside the baked radius and its height below the baked height.
4. Choose **Build > Bake Navigation**, then save the map. Select a volume to see its bounds and graph: green is current, orange needs a rebake. Play/Build automatically rebakes stale inputs. The graph is stored in the map's `navigation` cache, not as hundreds of actors.
5. Add a `NavigationAgentComponent` and an enabled box collider to an unparented Actor3D. Use a foot-origin model, unit actor scale, and place its collider above the feet. Set `move_on_start` and target coordinates to try it without code, or call `move_to(x,y,z)` from C++/Blueprint. `stop()`, `arrived()`, `failed()` and `status()` are reflected.

Put movable objects on a layer excluded from the bake's mask. Navigation agents are automatically excluded. The movement mask can still include dynamic obstacles: encountering one stops safely with `Blocked`; the AI decides when to request another route. It does not repeatedly search every tick.

`face_movement` rotates toward cardinal movement; models should face local +Z. Optional `moving_clip` and `idle_clip` are model clip indices (-1 leaves animation entirely to gameplay). Speed is in world units per second.

## Budgets and behavior

| Limit | Default |
|---|---:|
| Nodes / links per node | 512 / 4 |
| Simultaneous queued/searching/following requests | 8 |
| Waypoints per request | 64 |
| Shared search work units per rendered frame | 64 |
| Immutable node size | 14 bytes |
| Shared service, including all eight paths | about 3.4 KB |

The uniform cardinal graph uses breadth-first search (shortest number of grid edges). There is one reusable search workspace, not one per NPC. Projection, visited-array initialization, expansion, and reconstruction are all incremental and charged against the budget. No heap allocations or runtime baking. `epok::nav::world.budget` changes the shared budget; `stats.work`, `expanded`, `completed`, and `rejected` expose measurements. Collision/movement has its own cost, in addition to search.

Requests queue fairly; path buffers remain owned until arrival/cancellation. Destruction and disabling cancel requests, handles are generation checked, and switching maps resets the service. `NoPath`, `TooLong`, `OffGraph`, and `Blocked` are distinguishable failures. A full request pool makes `move_to` return false. Endpoints snap to a node within one spacing unit (L1 distance); arrival means reaching that projected node, not an arbitrary point inside a wall.

## v1 scope

This version supports flat box-collider walkable surfaces and static obstacles, including thin walls, low ceilings, clearance and separate vertical levels. It deliberately has no diagonal shortcuts. A connection must have continuous floor support; a single slab under modular render tiles is the simplest authoring layout.

Ramps, stair traversal, arbitrary triangle terrain, off-mesh jump/climb links, dynamic graph rebuilding and crowd avoidance are **not implemented in v1**. A ramp inside the bake volume reports an error instead of silently generating an unsafe route. A bigger bake fails with a capacity diagnostic; it never truncates silently.

## CLI and verification

```text
epok-editor --project <project> --bake-navigation --scene assets/scenes/NavigationTest.epokmap
epok-editor --project <project> --play-psx --scene assets/scenes/NavigationTest.epokmap
cargo test --bin epok-editor navigation::tests
python tests/runtime/verify_spatial.py navigation gameplay_api
```

Replace the example scene path with a map in your project. For on-target integration checks, place two agents on opposite sides of an obstacle, verify that both route around it and arrive, and monitor animator progress and the shared search budget. Keep captured RAM/GPU evidence under the project's ignored `.epok/` directory.
