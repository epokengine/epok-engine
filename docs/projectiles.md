# Projectile actors

Add a **ProjectilePool3DComponent** and a **SpatialQueryContextComponent** to an
ordinary 3D actor. Assign up to eight existing 3D actors to the pool's Visual
references. These actors must be active, have no spatial attachment and have no
enabled collider. Logical parenting is supported. The component uses world-space
linear motion; the chosen visual actors can have their own derived classes.

Sprite visibility management requires active Sprite3DComponents. Disable it to
manage mesh visibility in project rules. The optional Sprite Grid advances an
authored horizontal animation by simulation steps. It changes regions and enabled
state; the project retains texture, size, color and impact presentation.

`launch(first, count, spawn)` chooses a free slot in a bounded range and returns
a checked handle. `step(first, count, dt, filter)` advances up to four shots over
one collision snapshot. Read each `result(batch, index)` before the next step on
that pool. Results include motion/hit/expiry status, handle, target, contact point
and a project tag. Handle and batch identities reject cross-pool or stale use.

A gameplay `reset()` stops all shots while preserving the last immutable impact
batch, so a death/victory callback can finish processing that batch. The next
step replaces it. C++ may instead retain the by-value native `step_shared` result
while reusing an existing query snapshot. Invalid batch parameters refuse the
whole step. A stale binding stops its affected slot; arithmetic overflow preserves
that shot while other valid slots advance. A positive step performs its full
displacement even when the shot expires during that step.

Derive or compose ordinary classes to choose firing, teams, targeting, damage,
rewards and audiovisual effects in C++, Blueprint or Lua. The helper has no
automatic polling, game rules or global projectile storage. It is available
through the World Gameplay helper catalog independently of templates.
