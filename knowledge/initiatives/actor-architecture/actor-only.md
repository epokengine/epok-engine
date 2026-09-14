# Actor-only authoring and runtime

Accepted direction, 2026-09-14: remove Entity and Behaviour as object models.
Existing projects may require recreation; do not add a compatibility layer or
rewrite users' existing projects. This supersedes the compatibility requirements
in design.md and the legacy inventory.

## Contract

- A map owns one collection of Actor instances. Actor3D, Actor2D and UIActor
  inherit Actor and obtain their spatial contract from a typed root component.
- ActorComponent is the only attachable script family, in C++ and Blueprint.
  Owner-domain validation applies equally to both providers.
- Built-in mesh, camera, collider, light, audio and UI data are components of
  those same actors. Renderer/collision caches are derived component data, with
  no separate object identity, creation API or script lifecycle.
- Creating from Scene View defaults to its domain's concrete actor base.
  Hierarchy creation opens a searchable inheritance tree filtered by that domain.
- A new placement belongs only to its map. Convert to Actor Blueprint captures
  its class, components and defaults into a reusable project asset and links the
  placement. Placement overrides and component identities are retained.
- Actor/component Blueprint spatial nodes use the typed owner's root. They never
  depend on a legacy entity slot or silently treat UI/2D as 3D.
- New scene and Blueprint document versions reject the retired formats with a
  clear recreation message. Shipped templates must produce the new formats.

## Verification

Exercise creation in all three views; component provider/domain compatibility;
conversion to an asset; placement of two independent instances; save/reopen;
undo/redo; C++ and Blueprint compilation; and a fresh Third Person PSX Play run.
Existing math, rendering and component-service checks remain relevant. Tests
whose sole contract is Entity/Behaviour compatibility are retired with that API.
