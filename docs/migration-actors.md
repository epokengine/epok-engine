# Migrating an existing project to actors

Epok's [actor model](actors.md) is additive. An existing project keeps working with
no migration step, and nothing in a map is rewritten until you save it. This page is
the checklist for opening a project you already have.

## What happens when you open a map

Nothing on disk. Opening a map does not rewrite its bytes, does not change its
document version and does not mark it as changed. Your entities keep their indices,
their components, their Behaviour bindings and their identities.

What the editor adds is a **view**: wherever a tool needs to see a map as actors, it
derives one actor per legacy entity in memory. The derivation is deterministic —
the same map always produces the same identities, on any machine and from any folder
— and it never touches the document. `scene_actors` over MCP shows this view next to
the actors you authored, and marks which is which.

One thing is prepared in memory on opening: a map with no **scene Blueprint** is
given the default one (see [Scene Blueprint](actors.md#the-scene-blueprint)). That is
not an edit either — the map stays clean and the file is untouched — but it is part
of what the next explicit save writes.

## What happens when you save

A map is written as **document version 5** as soon as it actually contains actor
content: an authored actor, or a scene Blueprint. A map with neither keeps the
version it had (3 or 4), byte for byte.

Version 5 adds exactly two things to the document: an `actors` list and a
`scene_script` entry. Legacy entities are written exactly as before, in the same
place, so a version 5 map is still the map you had plus the new half.

Two consequences worth planning for:

- Saving a map that had no scene Blueprint writes the default one, which is what
  raises it to version 5 even if you never created an actor.
- Removing the last actor **and** the scene Blueprint from a version 5 map takes it
  back down to 3 or 4 on the next save. That is deliberate — a map with no actor
  content is a classic map — but it means the version is not a one-way door.

## How legacy entities map to actors

This is the derivation the in-memory view uses, and the shape a future explicit
migration would write. Components are listed in the order they appear on the actor.

| Legacy entity | Actor class | Components |
| --- | --- | --- |
| `kind: Camera` | `epok::Actor3D` | `SceneComponent3D` (root), `Camera3DComponent` |
| `kind: Mesh` | `epok::Actor3D` | `SceneComponent3D` (root), `Mesh3DComponent` |
| `kind: Empty` with no other data | `epok::Actor3D` | `SceneComponent3D` (root) |
| has a sprite | `epok::Actor3D` | `SceneComponent3D` (root), `Sprite3DComponent` — the sprite, its depth bias and orientation, plus its animator when one exists |
| has a light | `epok::Actor3D` | + `Light3DComponent` |
| has a collider | `epok::Actor3D` | + `Collider3DComponent` |
| has a canvas | `epok::UIActor` | `RectTransformComponent` (root), `CanvasComponent` |
| has a rect only | `epok::UIActor` | `RectTransformComponent` (root) |
| has an image / text / progress bar | `epok::UIActor` | + `ImageComponent` / `TextComponent` / `ProgressBarComponent` |
| has 3D data **and** UI data | `epok::Actor3D` | the 3D components only, plus a pending-conversion diagnostic |
| has an audio source | any of the above | + `AudioComponent` |
| has a Behaviour script | any of the above | + `LegacyBehaviourComponent` |
| is a Blueprint instance | that Blueprint's own class | the actor's class is the Blueprint class |
| has a parent entity | — | the parent's derived actor becomes the logical parent; spatial attachment is added only when both entities are in the same domain |

An entity that carries both 3D and UI data keeps **one** identity: splitting it would
break every reference to it. The view shows the 3D actor and reports the conversion as
pending. The entity keeps rendering and running exactly as before.

## Behaviour scripts

A Behaviour keeps its contract. On a migrated actor it is hosted by a
`epok::LegacyBehaviourComponent`, which forwards `start`, `update`, `frame_update`,
`on_enable`, `on_disable`, `on_destroy` and `on_trigger` to it, each exactly once.
`frame_update` still runs once per rendered frame even while the simulation is
paused, and `update` still does not.

The rule to remember: an entity's Behaviour is driven **either** by the legacy
binding **or** by an actor's compatibility component, never by both. You do not have
to arrange that; it is how the two halves are wired.

Writing new gameplay as an actor class instead of a Behaviour is a choice, not a
requirement. One Behaviour per entity is still the limit for Behaviours.

## Class references in Blueprints

`Spawn`, `Spawn Class`, `Is A` and stored `ClassRef` values keep working. Where a
legacy construct now has a different meaning, the editor derives a redirect for it:

| What it was | What it becomes |
| --- | --- |
| The class of a Behaviour bound to an entity | `epok::LegacyBehaviourComponent`, because the Behaviour is now hosted by that component on the migrated actor |
| The class of a Blueprint instance | the same class identity — but it is now an Actor-family class, so a `Spawn` / `Is A` / `ClassRef` consumer resolves it as one |

Nothing about a saved graph changes, and no `.epokbp` is rewritten. If a class
reference stops resolving after a reparent, repair it with the explicit remap
controls described in [Blueprints](blueprints.md#inheritance-and-identity).

## MCP clients

The `scene_2d` boolean on `editor_view` still works and still means what it always
meant: `true` is the UI mode, `false` is 3D. The new `view_mode` argument takes
`"3d"`, `"2d"` or `"ui"`, and `view_mode` wins when both are sent. `editor_state`
and every `editor_view` reply report both spellings.

The one thing to know when updating a client: the **2D** mode reports
`scene_2d: false`, because a 2D world is not the Canvas editor. A client that polls
`scene_2d` to decide whether to capture the HUD will see `false` in a mode that has
no 3D viewport either. Read `view_mode` instead.

The actor tools (`scene_actors`, `scene_add_actor`, `scene_remove_actor`,
`scene_set_actor`) are additive; the entity tools are unchanged. See
[AI assistants / MCP](mcp.md#actors).

## After upgrading: what to check

- **Your build is stale, and that is correct.** A map's actors and its scene
  Blueprint are part of the map's build provenance, so adding an actor override or
  creating a scene Blueprint marks the build stale and the next Build or Play
  recompiles. A byte-identical re-save does not.
- **Restart the native HUD preview once.** The preview protocol is now version 2. A
  preview executable cached from an older build is rejected with a message telling
  you to restart it, rather than being decoded with the wrong layout. See
  [Native HUD preview](native-hud-preview.md).
- **Check Map Settings on each map you care about.** The HUD budget controls moved
  there from Project Settings; Project Settings keeps the heading and an **Open Map
  Settings** button.
- **Set the project default if you want one.** *Default Scene Blueprint Parent*
  under Project Settings > Project > Description is only a suggestion for maps that
  do not have a scene Blueprint yet; it never changes a map that already has one.
- **On macOS, expect empty actor menus.** The reflection toolchain is not
  provisioned there, so the project's class model cannot be built, the Actor submenu
  is empty and no default scene Blueprint is proposed. Maps opened on macOS are not
  damaged; they simply do not gain actor content.
- **Console validation is still pending.** Building a map with actor content for the
  PlayStation, exporting it and measuring it in the emulator has not been run in this
  delivery. Do that on a machine with the SDK before shipping actor content.

## Rolling back

Version 5 is not readable by an editor from before this release: an older editor
refuses the document rather than opening it as an empty map. Likewise, this editor
refuses a document written by a *newer* editor instead of reading it partially.

So if you may need to go back:

1. Keep a copy of the map, or commit it, **before** the first save that writes
   version 5.
2. If you have already saved, removing every authored actor and the scene Blueprint
   and saving again writes the map as version 3/4 once more — but a map whose scene
   Blueprint you have authored has content that cannot come back from that.

Duplicating a map in the Project browser is safe either way: the copy gets its own
identities — a new scene Blueprint class, new actors and entities, and every
map-scoped reference rewritten to the copy — while shared content such as textures,
meshes and sounds is referenced, not copied.
