# Shared 3D query context

SpatialQueryContextComponent is an ordinary, inheritable world component. C++,
Blueprint and Lua use the same existing collision backend through its reflected
methods. It adds a synchronization boundary, without creating another physics
world or moving game decisions into a native game controller.

Place it on a 3D actor and bind a component reference from consumers. The default
`automatic_reset` clears its batch each Input tick. Derived tick implementations
must call the parent to retain that behavior. Activation and level exit also
clear the batch.

- `raycast_segment` returns the usual owned CollisionHitSample.
- `raycast_pair` executes two rays against one synchronized world and returns
  independent `first` and `second` CollisionHitSample values.
- `move` resolves a live actor, moves it through the existing collision kernel
  and publishes its resulting bounds for following queries.
- `publish_actor` publishes direct collider/transform edits. It accepts inactive
  actors so disabling a collider can remove its old bounds from the snapshot.
- `reset_queries` starts a new synchronization boundary.

Actor arguments belong to the context owner's level. Invalid/stale references,
foreign-level actors and inactive owners return default results without calling
the collision service. Moving an inactive subject is rejected. A ray's invalid
nonempty ignore reference is rejected rather than silently ignoring another
actor.

When several systems edit colliders inside one batch, publish those edits before
the next query. Advanced C++ consumers may borrow `batch()` for the native kernel.
That reference is internal to native consumers; scripts use the typed methods.
Setting `automatic_reset` to false requires explicit publication of every direct
edit and reset when scene/collision ownership changes. Prefer the default for
ordinary components.

Lua uses the normal API names:

```lua
local query = epok.cast(self.query_context, "epok::SpatialQueryContextComponent")
local origin = epok.math.vector3(0.0, 1.0, 0.0)
local direction = epok.math.vector3(0.0, 0.0, 10.0)
local hit = query:raycast_segment(origin, direction, 1, self.subject, false)
```

The generated API reference lists each signature and value field. Host adapter
tests cover ownership, activation, reset and slot reuse; provider integration
uses actual C++/Blueprint/Lua policies against the console collision backend.
