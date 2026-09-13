# Runtime services and scene banks

Project Settings > Maps & Build registers additional scenes in `ProjectSettings/Maps.epoksettings`:

```yaml
scenes:
  - assets/scenes/Night.epokmap
```

The exported startup scene is bank 0. Names must be unique; paths must remain inside `assets/scenes`. Up to 15 additional banks are supported. `request_scene("Night")` or `request_scene(size_t(1))` queues a transition at the next frame boundary. `current_scene()` and `scene_loading()` report state.

Editor Play defaults to the open scene only, including unsaved edits, as bank 0.
Choose Whole game to start at the project startup scene and include registered
maps. An open registered map uses its unsaved snapshot. Duplicate map paths are
included only once; different files with duplicate scene names remain an error.
See [Play profiles and transitions](play.md) for targets and customization.

Banks are prelinked into the executable, so immutable source geometry, scripts and texture data must fit PSX main RAM. One object pool is sized for the largest bank plus 32 reusable slots. Shared texture pixels and palettes are emitted once; each bank has an independently validated VRAM layout that replaces the previous bank on activation. Shared resident SFX remain bounded by SPU capacity. This profile does not stream arbitrary large maps from CD.

Transitions fade picture and audio out before stopping outgoing audio, waiting
for XA callbacks, invalidating handles and calling Behaviour disable/destroy
hooks. They clear caches, restore initial data/properties and run incoming scripts.
A resident loading overlay remains visible during loading, followed by a picture
and audio fade in. Geometry warmup can read CD or PC data when streaming is enabled.
Lifecycle callback requests queue behind the current fade. Game-owned C++ globals
survive scene switches; Behaviour member state resets.

## Object lifetime

`handle(entity)` returns a slot and generation. `handle.get()` returns null after destruction or scene replacement. Raw pointers remain useful for immediate access but do not detect slot reuse.

`set_active(entity, false)` affects descendants, stops their audio, and skips updates, rendering, lighting, shadows and collision. Inactive objects retain data. `destroy_entity` destroys the subtree, removes particles, invalidates handles and frees dynamic slots. Authored slots keep their script bindings until the scene is reloaded. Slots retained by asynchronous XA callbacks remain quarantined until the callback releases them.

All scripts are bound before any `start` callback. Authored component data and script properties are initialized before `start`; active scripts then receive `on_enable`. Scripts receive `on_disable` and `on_destroy` on the corresponding lifecycle transitions. Exported Behaviour classes must be default-constructible and assignable so a bank reload can reset their members.

## Timing and utilities

See [input and collision](input-collision.md) for measured fixed-step simulation and pause controls. Include `utility.hpp` for these allocation-free helpers:

- `Tween`: value interpolation with linear, smoothstep and quadratic easing; explicit cancel and one-shot completion. Apply its value to the property chosen by the game.
- `EventQueue<N>`: bounded FIFO carrying an event kind, integer value and generation-checked source handle. Overflow increments `dropped`.
- `Sequence`: owns up to 256 timed event steps, handles multiple crossed steps, completes once and can be cancelled.
- `Focus<N>` and `layout_list`: generic UI navigation and layout; see [HUD](hud.md).

Helpers do not run behind the game's back: call `advance(dt)` from a Behaviour update, and poll events/completions there. A queued source handle may become invalid before consumption; check it when needed.

Memory Card operations use a separate process-owned asynchronous service and survive scene transitions. See [Memory Card](memory-card.md).
