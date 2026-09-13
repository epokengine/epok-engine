# P4 — Documents and migration (2026-09-13)

Implements design.md section 7 (scene document version 5) and section 5 of the plan
(legacy → actor mapping). New module `src/actor_document.rs`; `src/scene.rs` gains the two
fields, the version rules and the identity remapping; `src/blueprint_refs.rs` gains the
legacy class redirect table.

Nothing in this phase writes actors to disk on its own. `Scene::load` still leaves the bytes
of a legacy document untouched, and the version only rises to 5 when actor content actually
exists in the document being saved.

## Document format

`Scene` gains exactly two members. Both are `#[serde(default)]` and are omitted when empty,
so a version 3/4 document serializes byte-identically to before.

```rust
#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub actors: Vec<actor_document::ActorInstance>,
#[serde(default, skip_serializing_if = "Option::is_none")]
pub scene_script: Option<actor_document::SceneScript>,
```

A version 5 scene, shown as JSON (the file on disk is the same shape encoded as YAML by
`crate::document`):

```json
{
  "version": 5,
  "name": "SampleScene",
  "entities": [ /* unchanged legacy entities */ ],
  "actors": [
    {
      "id": "7d0f1c02-9f3a-4c0e-9d3a-6b1c2f7d4a11",
      "class": { "name": "epok::Actor3D", "class_id": "fc24ce9b-558c-49de-bc35-e040f350e486" },
      "name": "Hero",
      "active": true,
      "components": [
        {
          "id": "1a2b3c4d-0000-4000-8000-000000000001",
          "class": { "name": "epok::SceneComponent3D", "class_id": "ed73d249-b6cb-4a3c-a0e8-696de55e286f" },
          "name": "Transform",
          "root": true,
          "properties": { "position": [0.0, 0.5, 0.0], "rotation": [0.0, 0.0, 0.0], "scale": [1.0, 1.0, 1.0] },
          "overrides": ["position", "rotation", "scale"]
        },
        {
          "id": "1a2b3c4d-0000-4000-8000-000000000002",
          "class": { "name": "epok::AudioComponent", "class_id": "7f0eb028-5301-4ac7-b93b-5665fab12b20" },
          "name": "Footsteps"
        }
      ],
      "properties": { "speed": 2.5 },
      "overrides": ["speed"],
      "legacy_entity": "2f1d7b60-4c8a-4d2e-8f10-9a4b6c1d2e30"
    },
    {
      "id": "7d0f1c02-9f3a-4c0e-9d3a-6b1c2f7d4a12",
      "class": { "name": "epok::UIActor", "class_id": "b09bd2fa-8b09-4c0f-a33a-c3ca08b21d8f" },
      "name": "Health",
      "active": true,
      "logical_parent": "7d0f1c02-9f3a-4c0e-9d3a-6b1c2f7d4a11",
      "components": [
        {
          "id": "1a2b3c4d-0000-4000-8000-000000000003",
          "class": { "name": "epok::RectTransformComponent", "class_id": "dc805165-6c65-48dc-8ff8-4a638a5d21df" },
          "name": "RectTransform",
          "root": true
        }
      ]
    }
  ],
  "scene_script": {
    "parent": { "name": "epok::SceneScriptActor", "class_id": "b4c08aa0-fa85-4abf-8f45-7501e1c8a040" },
    "blueprint": { "version": 3, "id": "…", "name": "MapScript", "parent": "b4c08aa0-…", "…": "a full BlueprintAsset" }
  }
}
```

Notes on the shapes:

- `ClassReference { name, class_id }`, `Attachment { actor, component? }` and `SceneScript`
  use `deny_unknown_fields`: they carry no free-form authoring data, so an unknown member is
  a document error rather than something to preserve.
- `ActorInstance` and `ComponentInstance` deliberately do not: authored values live in
  `properties`, and `overrides` records which of them are explicit. Their `Deserialize` is
  hand-written exactly like `ScriptBinding`'s — a document with no `overrides` set treats
  every persisted property as an override, so orphan values from a class that changed shape
  survive a load/save cycle instead of being dropped.
- `attach` (spatial) and `logical_parent` (the hierarchy the author sees) are separate.
  `attach.component` absent means "the target actor's root component".
- `legacy_entity` is migration provenance only. It points at an entity of the same document.
- `SceneScript::blueprint` is a whole `crate::blueprint_asset::BlueprintAsset` embedded in
  the map; that module is untouched by this phase. `BlueprintAsset` has no `PartialEq`, so
  `SceneScript` compares the serialized asset instead.

### Versions

| Version | Meaning |
| --- | --- |
| 1–2 | pre-UUID legacy; identities are derived on load, never written back by a load |
| 3 | persistent entity UUIDs |
| 4 | timeline / particle-effect components present |
| 5 | `actors` and/or `scene_script` present |

`Scene::validate` accepts 1–5. `upgrade_entity_ids` raises the version to 5 only when
`actors` is non-empty or `scene_script` is `Some`, otherwise it keeps the 3/4 decision it
made before. A document whose version is above 5 is refused with

> This scene was written by a newer editor (document version N, this editor supports up to
> 5). Update the editor rather than opening it: the document is never loaded as empty.

and a version of 0 keeps the old `Unsupported scene version`. The refusal happens in
`validate`, which `load_unresolved` runs before anything else, so a newer document is never
partially read and never re-saved as an empty scene.

### Identity remapping

`Scene::duplicate_branch`, `Scene::delete_branch` and the new `Scene::duplicate_identities`
(the scene-duplication helper) keep the actor half consistent:

| Helper | Behaviour |
| --- | --- |
| `duplicate_branch` | copies the actors whose `legacy_entity` is inside the branch, gives every copied actor and component a fresh UUID, and rewrites `logical_parent`, `attach.actor`, `attach.component` and `attach_parent` to the copies. References that leave the branch keep their original target. |
| `delete_branch` | drops the actors whose `legacy_entity` was deleted and clears (rather than dangles) the `logical_parent` / `attach` of the survivors that pointed into the branch. |
| `duplicate_identities` | fresh UUIDs for every entity, actor and component, with `blueprint_instance.instance`, `legacy_entity` and every internal actor reference remapped, a fresh scene-Blueprint asset id, and every actor/component UUID literal inside the embedded Blueprint rewritten (the asset is walked as JSON, so pins, defaults and variables are covered). |

External asset UUIDs are never members of the remap table, so a texture, mesh or sound id
inside `properties` or inside the Blueprint survives every one of these operations unchanged.
This is asserted by `duplicating_a_scene_remaps_actor_and_component_identities_but_not_assets`.

## Migration mapping, as implemented

`actor_document::actor_view(scene, model) -> ActorView { actors, diagnostics }` derives the
actor model of a legacy scene in memory. It never mutates the scene, is deterministic across
machines and folder moves, and skips any entity that already has an authored actor
(`legacy_entity` match), so a half-migrated document is never derived twice.

Identities are derived the same way `Scene::upgrade_entity_ids` derives entity UUIDs: a
SHA-256 over the fixed salt `epok actor view identity v1`, the entity UUID and a role string,
truncated to 16 bytes (`actor_document::derived_id`). The role is `""` for the actor itself
and `root` / `camera` / `mesh` / `sprite` / `light` / `collider` / `canvas` / `image` /
`text` / `progress` / `audio` / `behaviour` for its components.

Domain: an entity that carries UI data and no 3D data becomes `UI`; everything else becomes
`World3D`.

| Legacy entity | Actor class | Components (in order) |
| --- | --- | --- |
| `kind = "Camera"` | `epok::Actor3D` | `SceneComponent3D` (root, transform), `Camera3DComponent` (`camera_fov`) |
| `kind = "Mesh"` | `epok::Actor3D` | `SceneComponent3D` (root), `Mesh3DComponent` (`material`, `lighting`) |
| `kind = "Empty"` with no data | `epok::Actor3D` | `SceneComponent3D` (root) |
| `sprite` | `epok::Actor3D` | `SceneComponent3D` (root), `Sprite3DComponent` — the whole `Sprite` plus `depth_bias` and `orientation` carried verbatim, and `animator` when a `sprite_animator` exists |
| `light` | `epok::Actor3D` | + `Light3DComponent` |
| `collider` | `epok::Actor3D` | + `Collider3DComponent` |
| `canvas` (with or without `rect`) | `epok::UIActor` | `RectTransformComponent` (root), `CanvasComponent` |
| `rect` only | `epok::UIActor` | `RectTransformComponent` (root) |
| `image` / `text` / `progress` | `epok::UIActor` | + `ImageComponent` / `TextComponent` / `ProgressBarComponent` |
| 3D data **and** UI data | `epok::Actor3D` | the 3D components; diagnostic `hybrid-entity-pending-conversion` |
| `audio` | any actor | + `AudioComponent` (`audio`) |
| `script` (a `ScriptBinding`) | any actor | + `LegacyBehaviourComponent` named after the binding, carrying its `properties` and `overrides` plus `behaviour` (the name) and `behaviour_class` (the `class_id`) |
| `blueprint_instance` | the Blueprint's own class | the actor's `class` is that class (resolved through the `Model` when one is available), and `legacy_entity` records the entity |
| `parent` index | — | `logical_parent` = the derived actor of the parent entity; `attach` is set as well, to the parent's root component, only when both entities are in the same domain |

The hybrid case deliberately keeps one identity. Splitting a 3D+UI entity into two actors
would break every persisted reference to it, so the view shows the 3D actor and reports the
conversion; P7 asks the author what the UI half should become.

`actor_view` also runs `Model::validate_component_set` over each derived actor when a model
is given, so a derivation that would not be legal shows up as a diagnostic rather than as a
silently broken document. On the fixture scene the only diagnostic produced is the hybrid one.

## Validation table

`Scene::validate()` is unchanged for every existing caller: it is now
`validate_with_model(None)` and runs only the structural half. `Scene::validate_with_model`
takes an `Option<&object_model::Model>`. Everything class-aware is asked of the `Model`;
none of its rules is re-implemented here.

`actor_document::validate(scene, model)` returns `Vec<object_model::Diagnostic>`;
`validate_message` joins them into the sentence the scene loader reports. It returns
immediately when a scene has neither actors nor a scene script, so classic documents pay
nothing.

| Check | Code | Needs a Model |
| --- | --- | --- |
| Actor and component UUIDs are non-nil and unique across entities, actors and components | `duplicate-identity` | no |
| Actor names are 1–128 bytes | `actor-name` | no |
| Class names are valid C++ identifiers (`scripts::class_identifier`) | `invalid-class-name` | no |
| Every key in `overrides` has a value in `properties` | `override-without-value` | no |
| `legacy_entity` names an entity of this scene | `dangling-legacy-entity` | no |
| At most one `root` component per actor | `duplicate-root` | no |
| `attach_parent` names a component of the same actor | `dangling-attach-parent` | no |
| `logical_parent` exists and is not the actor itself | `dangling-logical-parent` | no |
| `logical_parent` chains do not cycle | `cyclic-logical-parent` | no |
| `attach.actor` exists, is not the actor itself, and `attach.component` belongs to it | `dangling-attachment`, `self-attachment` | no |
| The attached actors share a domain | `attachment-domain` | yes |
| The actor class is a reflected class | `unknown-class` | yes |
| The class is in the Actor family | `not-an-actor` | yes |
| A `SceneScriptActor` subclass is never placed in `actors` | `scene-managed-actor` | yes |
| The class is `Placeable` | `not-placeable` | yes |
| The class is not abstract | `abstract-actor` | yes |
| Component set: unique root, root domain and rootability, requires/excludes, cardinality, capabilities | `Model::validate_component_set` codes | yes |
| `scene_script.parent` derives from `epok::SceneScriptActor` | `scene-script-parent`, `unknown-class` | yes |

## Legacy class redirects

`blueprint_refs::redirects(scene) -> Vec<Redirect>` derives the table design.md section 7
reserves for "old Blueprint classes used by Spawn/IsA/ClassRef", and
`blueprint_refs::resolve_class_id(id, &table) -> String` follows it. Unknown ids pass
through unchanged, chains are followed, and a cyclic table stops at the entry it re-enters
so a damaged table can never hang the compiler.

| `reason` | `from` → `to` |
| --- | --- |
| `BehaviourHostedByLegacyComponent` | a `ScriptBinding::class_id` used in the scene → `epok::LegacyBehaviourComponent` (`430cb0ca-…`), because the behaviour is now hosted by that component on the migrated actor |
| `BlueprintInstanceIsActorClass` | a `blueprint_instance.class` → itself: the identity is unchanged but the family is not, so `Spawn`/`IsA`/`ClassRef` consumers must re-resolve it |

**For the Blueprint compiler agent (P5):** this table is derived, never applied, in P4. The
compiler decides which `reason` applies to which reference kind — a `ClassRef` on a `Spawn`
node and a `ClassRef` used for an `IsA` test do not want the same answer — and calls
`resolve_class_id` where it does. The `to` of a `BlueprintInstanceIsActorClass` entry is
intentionally equal to its `from` so that applying the whole table blindly is a no-op for
that kind.

## Deferred

| Work | Phase |
| --- | --- |
| Cooking actors and components into the C++ tables (`project::scene_header_body_with_layout`, `ObjectRegistry` capacity, `Binding` records) | P10 |
| Editor UI: hierarchy and Inspector over actors, `SceneViewMode`, Map Settings, the actual conversion action for hybrid entities | P7 |
| Blueprint compiler consuming the redirect table and emitting actor classes | P5 |
| Writing derived actors into the document (a real migration rather than a view) | P7, behind an explicit author action |
| Manifest `default_scene_script_parent` and the scene Blueprint parent selector | P7 |

Because of that ordering, most of `actor_document`'s public surface has no caller yet; the
module carries `#![allow(dead_code)]` with the same justification `object_model.rs` uses,
and `Scene::duplicate_identities` and the redirect helpers carry per-item `allow`s.

## Validation run on this machine

| Step | Result |
| --- | --- |
| `cargo test --locked scene` | passes (the one failure is the pre-existing shared-ImGui `gui::interaction_tests::scene_clicks_and_hierarchy_context_menu_use_real_imgui_events`) |
| `cargo test --locked actor_document` | 14 passed, 0 failed |
| `cargo test --locked` | 421 passed, 7 failed, 33 ignored — the 7 are exactly the P0 baseline list, no new failures (baseline at `1b100db` was 404 passed) |
| `cargo clippy --locked --all-targets -- -D warnings` | 25 errors, exactly the pre-existing baseline count; no new ones |
| `cargo fmt` | run only on the touched files; unrelated reformatting reverted (the repo is not fmt-clean) |
| `cargo build --locked --bins` | succeeds |

MIPS builds, standalone exports and emulator runs were not executed (no SDK on this machine)
and nothing about them is claimed.

## Tests added

`src/actor_document.rs` (14):

- `version_five_scene_round_trips_actors_components_attachments_and_scene_script` — two
  `Actor3D` with different overrides, a `UIActor`, `AudioComponent` on both an `Actor3D` and
  a `UIActor`, an attachment and a logical parent, plus a `scene_script`.
- `version_six_is_rejected_as_written_by_a_newer_editor`
- `saving_actor_content_raises_the_version_and_classic_scenes_stay_at_three`
- `two_save_cycles_are_idempotent_for_legacy_and_version_five_documents` — a legacy fixture
  and a version 5 scene, each loaded and saved twice; also asserts that loading never
  rewrites the bytes.
- `duplicate_uuids_across_entities_actors_and_components_are_rejected`
- `unknown_class_and_invalid_class_names_are_rejected`
- `a_ui_component_on_a_three_d_actor_is_rejected_by_the_model`
- `a_scene_script_actor_may_not_be_placed_and_must_parent_the_scene_blueprint`
- `cyclic_logical_parents_and_dangling_references_are_rejected`
- `attachment_across_domains_is_rejected_only_when_a_model_is_available`
- `duplicating_a_scene_remaps_actor_and_component_identities_but_not_assets`
- `duplicate_and_delete_branch_carry_and_drop_the_actors_of_the_branch`
- `legacy_entities_map_to_actors_deterministically_without_touching_the_scene` — the
  camera / mesh / empty / sprite / canvas / rect+image / hybrid / script fixture.
- `the_derived_view_satisfies_the_model_and_reuses_authored_actors`

The model fixtures are hand-built `schema::Class` values, like the tests in
`src/object_model.rs`: libclang extraction needs the MIPS include paths and cannot run here.

`src/scene.rs` (2): `every_supported_scene_version_loads_and_a_newer_document_is_refused`,
`actor_content_survives_a_document_round_trip_and_raises_the_version`.

`src/blueprint_refs.rs` (1):
`legacy_class_redirects_are_derived_from_the_scene_and_resolve_transitively`.

## Open questions for later phases

- `upgrade_entity_ids` lowers the version of a version 5 document back to 3/4 if all its
  actors are removed. That is the literal reading of the design rule and keeps classic
  documents classic, but P7 should confirm it is what an author expects when they delete the
  last actor of a map.
- `actor_view` derives one actor per legacy entity, including UI entities that the cook
  currently treats as pure HUD data. P10 decides whether the cooked form keeps that 1:1
  relationship or collapses a canvas subtree.
- The redirect table currently has one entry kind per legacy construct. If P5 needs to
  distinguish a spawn target from an `IsA` base at the table level rather than at the call
  site, `RedirectReason` is the place to extend.
