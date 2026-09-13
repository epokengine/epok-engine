# P1 — Registry and contracts (2026-09-13)

Rust-only phase. Reflection schema 8, the annotation grammar of design.md section 3, and
the host model of design.md section 5. No runtime file was edited (`runtime/` belongs to
the runtime agent); `runtime/object_model.hpp` is read-only input here.

## What was added

### `src/reflection_schema.rs` — schema 8

`SCHEMA_VERSION` 7 → 8. New public types, all `#[serde(default)]` on the `Class` side so
version-7 manifests deserialize unchanged and untouched classes never grow new wire fields
(`skip_serializing_if` on every optional/empty field):

| Type | Notes |
| --- | --- |
| `ClassFamily` | `Object, Actor, Component, World, Level, Behaviour`; `Behaviour` is `Default` (legacy). `label()` for diagnostics. |
| `Domain` | `None, World3D, World2D, UI`; `Default` is `None`. `label()`, `spatial()` (`!= None`). Serde renames: `world3d`, `world2d`, `ui`. |
| `Placement` | `placeable`, `spawnable`, `scene_managed` booleans. |
| `Cardinality` | `Single` (default), `Multiple`. |
| `ComponentContract` | `owners: BTreeSet<Domain>`, `requires`, `excludes`, `cardinality`, `can_root`, `capabilities: BTreeSet<String>`. |
| `DefaultComponent` | `id`, `field`, `class` (cpp_name), `root`, `attach_to`, `name`. |

`Class` gains `family: Option<ClassFamily>`, `domain: Option<Domain>`, `placement`,
`component: Option<ComponentContract>`, `default_components`, `explicit_abstract`.
`Type` gains `ObjectRef { class: Option<String> }`, `ActorRef { … }`, `ComponentRef { … }`;
`Type::label()` renders them, `members()` leaves them opaque (they are identities, not
records). `EntityRef` and `ClassRef` keep their meaning.

Tests: `version_seven_classes_deserialize_without_actor_metadata` (a schema-7 `Class` JSON
loads with all-default actor metadata and round-trips without emitting the new keys) and
`typed_object_references_label_their_class`.

### `src/header_extract.rs` — annotation grammar

`pub fn class_options(&[String]) -> Result<ClassOptions, String>` is the pure token parser;
`ClassOptions { blueprintable, explicit_abstract, family, domain, placement, component,
timeline_requires }`. `fn class()` now calls it once and fills the new `Class` fields from
it (`blueprintable` and `TimelineRequires` are read from the same parse, so there is a
single grammar).

`pub fn component_options(&[String]) -> Result<DefaultComponentOptions, String>` parses
`EPOK_COMPONENT(...)`. In `fn class()`, a child `FieldDecl` carrying an `EPOK_COMPONENT:`
annotation becomes a `schema::DefaultComponent`: the field's canonical type declaration must
itself carry `EPOK_CLASS` metadata, and the id comes from the existing `id()` helper with
prefix `EPOK_COMPONENT:`, so an explicit `Id="…"` survives a rename exactly as it does for
classes, properties and functions. A field may not be both `EPOK_PROPERTY` and
`EPOK_COMPONENT`.

### Annotation grammar as implemented

`EPOK_CLASS(...)`, comma separated, each token `Key` or `Key=Value` (surrounding double
quotes on a value are stripped; leading/trailing whitespace is trimmed; an empty token — the
`EPOK_CLASS()` case — is ignored):

| Token | Effect | Errors |
| --- | --- | --- |
| `Blueprintable` | `blueprintable = true` | — |
| `Id="uuid"` | consumed by `id()`, ignored here | non-UUID rejected by `id()` |
| `TimelineRequires=X` | collected; caller still rejects more than one | unknown component name |
| `Abstract` | `explicit_abstract = true` | — |
| `Family=Object\|Actor\|Component\|World\|Level` | declared family | unknown value; declared twice |
| `Domain=World3D\|World2D\|UI\|None` | declared domain | unknown value; declared twice |
| `Placeable`, `Spawnable` | placement flags | combined with `SceneManaged` |
| `SceneManaged` | placement flag | combined with `Placeable`/`Spawnable` |
| `Root` | `component.can_root = true` | — |
| `Owners=A\|B` | `component.owners`; `None` is not an owner domain | unknown/empty entry |
| `Requires=A\|B`, `Excludes=A\|B` | class-name lists | empty entry |
| `Cardinality=Single\|Multiple` | component cardinality | unknown value; declared twice |
| `Capability=name` | adds one target capability | empty value |
| anything else | — | `Unknown EPOK_CLASS option \`<token>\`` |

`component` is `Some(..)` only when at least one component token appears, so "declared" and
"inherited" stay distinguishable downstream.

`EPOK_COMPONENT(...)`: `Root`, `Name="display"`, `AttachTo=fieldName`, plus `Id="uuid"`
(consumed by `id()`). `Root` together with `AttachTo` is an error (a root has no attachment
parent); unknown tokens and empty `Name=`/`AttachTo=` values are errors.

Grammar limitation, by construction: the extractor splits Clang's annotation string on
commas before `class_options` sees it, so no option value may contain a comma (this already
held for schema 7).

### `src/object_model.rs` (new)

`Model::from_registry(&blueprint::Registry) -> Result<Model, Vec<Diagnostic>>` resolves the
whole graph depth-first with cycle detection and returns *every* diagnostic, not the first.
`ClassModel` carries only resolved values (`family`, `domain`, `placement`, `component`,
`abstract_class`, `blueprintable`, `final_class`, `default_components`, `provider`,
`backend`, `ancestry` root-first including the class itself). Consumers never walk the
parent chain themselves.

Resolution rules:

- **Parent**: unknown → `unknown-parent`; on the resolution path → `cyclic-parent`. A class
  with a diagnostic is omitted from the model.
- **Provider**: a `cpp` class whose parent is a `blueprint` class → `native-from-blueprint`.
  C++→C++, C++→BP→BP are legal and inherit everything.
- **Family**: inherited when not declared; redeclaring a *different* family is
  `family-mismatch`, except under `ClassFamily::Object`, which is the universal root the
  Actor/Component/World/Level roots declare themselves beneath (see Deviations).
- **Domain**: inherited; declaring a different non-`None` value than an inherited non-`None`
  one is `domain-mismatch`. Refining an inherited `None` (Actor → Actor3D) is legal.
- **Placement**: OR of declared and inherited; `scene_managed` forces
  `placeable = spawnable = false`. Any placement on a non-Actor family is
  `placement-on-non-actor`.
- **Component contract**: only for `ClassFamily::Component`; declaring one elsewhere is
  `component-contract-on-non-component`. Merge with the inherited contract: `owners` may
  only narrow (`owners-widened` otherwise; empty declared set inherits), `requires`,
  `excludes` and `capabilities` accumulate, `can_root` and `Cardinality` widen only.
- **Default components** accumulate down the chain; a subclass redeclaring the same field
  name replaces the inherited entry.
- Abstract, non-instantiable bases stay in `ancestry`.
- Legacy classes with no declared family whose chain ends at `epok::Behaviour`
  (`8ec3a9d4-…`) — or at nothing at all — resolve to `ClassFamily::Behaviour`, are never
  Actors, never placeable, and carry no component contract.

API: `class(id_or_cpp_name)`, `is_a`, `iter`, `len`/`is_empty`,
`eligible_parents(author, family)` (family-filtered, `derivable_by` mirrors
`script_backend::can_derive` over resolved values, sorted by `cpp_name`),
`validate_reparent` (`parent-not-derivable`, `family-crossing`, `domain-mismatch`,
`cyclic-parent`), `validate_component` (`unknown-class`, `not-an-actor`, `not-a-component`,
`abstract-component`, `owner-domain`), `validate_component_set` (`duplicate-root`,
`root-not-rootable`, `root-domain`, `missing-root`, `missing-requirement`,
`excluded-component`, `unknown-capability`, `duplicate-component`), `placeable()`,
`scene_script_parents()`. `Diagnostic { code, message, class }` — `code` is the stable key
for tests, the UI and MCP.

Stable ids: `pub const` for every base of design.md section 2 (`OBJECT_ID`, `ACTOR_ID`,
`ACTOR3D_ID`, `ACTOR2D_ID`, `UI_ACTOR_ID`, `SCENE_SCRIPT_ACTOR_ID`, `ACTOR_COMPONENT_ID`,
`SCENE_COMPONENT3D_ID`, `SCENE_COMPONENT2D_ID`, `UI_COMPONENT_ID`,
`RECT_TRANSFORM_COMPONENT_ID`, `AUDIO_COMPONENT_ID`, `LEGACY_BEHAVIOUR_COMPONENT_ID`,
`WORLD_ID`, `LEVEL_ID`, `BEHAVIOUR_ID`) and for the twelve identities reserved for the
P3/P8 component adapters.

`KNOWN_CAPABILITIES` is `["audio"]` in this phase: the only capability any declared
component needs today. Anything else produces `unknown-capability` *naming the capability*,
which is exactly the hook the exporter needs to map capabilities onto a target.

Tests (15): header-identity check, native-base resolution, legacy Behaviour chains,
C++/Blueprint chains plus `native-from-blueprint`, missing/cyclic parents, family and domain
redeclaration, owners narrowing/widening, contract/placement outside their family, reparent
(family crossing, domain, final, non-blueprintable, cycle, self, unknown), component domain
gating (UI component on `Actor3D` rejected, `AudioComponent` accepted on all three domains,
abstract component, non-actor owner), root uniqueness/domain/`can_root`/absence,
requires/excludes/capabilities, `Cardinality::Single` duplicates rejected and `Multiple`
accepted, `eligible_parents`/`placeable`/`scene_script_parents`, default-component
accumulation, and `Registry::model()`.

### `src/project.rs`, `src/blueprint.rs`, `src/blueprint_ir.rs`

- `runtime_sources()` stages `object_model.hpp` (first entry, ahead of `hud_core.hpp`).
- `Registry::model()` convenience wrapper over `Model::from_registry`.
- `blueprint_ir::cpp_type` rejects the three new `Type` variants with an explicit message:
  their Blueprint ABI belongs to the runtime object model, and guessing it here would
  fabricate a contract the runtime does not yet implement.
- Every existing `schema::Class` literal in the tree gained the six new fields with neutral
  values (`blueprint.rs`, `blueprint_compile.rs`, `blueprint_editor.rs`,
  `blueprint_templates.rs`, `script_backend.rs`, `timeline_tests.rs`).

### Other lists enumerating runtime headers

Checked. `src/project.rs::runtime_sources()` is the only explicit enumeration, and it is
also what the staging fingerprints consume (`write_changed` over the same list).
`tools/generate-api-reference.py` **globs** `runtime/*.hpp` (line 1053), so
`object_model.hpp` is picked up with no change; its `MODULE_CONTEXT` table has a graceful
fallback for unlisted module names. `runtime/epok.hpp` already includes the header
(line 248, owned by the runtime agent). No other list required an edit.

## Extractor limitations (not executed here)

libclang cannot run on this machine: extraction needs the MIPS compiler include paths and
`mipsel-none-elf-g++` is not installed (p0-baseline.md). Consequences:

- `class_options` and `component_options` are unit-tested as pure functions, and one test
  parses the real `runtime/object_model.hpp` text and asserts the grammar accepts all 15
  native declarations. The Clang-driven path around them (`fn class()` reading annotations,
  resolving the `EPOK_COMPONENT` field type to a reflected class) is **compile-checked
  only**; it has not produced a manifest on this machine.
- The `EPOK_COMPONENT` macro itself is defined in `runtime/object_model.hpp` (runtime
  agent). No project header uses it yet, so no default component has been extracted end to
  end.
- The `class_options` acceptance test lives in `src/header_extract.rs` and the id-constant
  test in `src/object_model.rs` because the two modules belong to different binaries
  (`header_extract` is compiled only into `epok-header-tool`, `object_model` only into
  `epok-editor`). Both read the same `include_str!("../runtime/object_model.hpp")`.
- First real extraction on an SDK machine must confirm: the annotation string Clang reports
  for `EPOK_CLASS` with `Owners=A|B` survives the `#__VA_ARGS__` stringification with the
  pipes intact, and that `EPOK_COMPONENT` fields of class type do not trip the
  "Reflected classes require defaulted constructors" check.

## Deviations from design.md

1. **`Family` under `Object`.** design.md section 3 says "declaring a different family than
   the parent is an extraction error", but section 2 has `Actor`, `Component`, `World` and
   `Level` all deriving from `epok::Object`, which declares `Family=Object`. Implemented
   rule: a family declaration is an error only when the inherited family is not `Object`;
   `Object` is the universal root beneath which the four family roots declare themselves.
   Without this, the header of section 2 cannot resolve at all.
2. **Contract merge direction for `Cardinality` and `Root`.** design.md fixes only the
   `owners` narrowing rule. `ComponentContract.cardinality` is a plain value, so a subclass
   that declares any component token would otherwise silently reset an inherited
   `Cardinality=Multiple` to `Single`. Implemented rule: `cardinality` and `can_root` widen
   only (`Multiple` and `can_root = true` are never lost), `requires`/`excludes`/
   `capabilities` accumulate. Narrowing cardinality is therefore not expressible; no base
   needs it today.
3. **`KNOWN_CAPABILITIES` is a placeholder.** design.md defers capability availability to
   the target ("unknown on target → build diagnostic"). P1 has no target table, so the host
   knows `audio` only and reports anything else as `unknown-capability` with the capability
   name in the message, for the exporter to resolve in a later phase.
4. **Diagnostics are collected, not fatal-on-first.** `from_registry` drops only the classes
   that failed and reports every diagnostic, so a project with one broken class still
   produces a reviewable list.
5. **Typed references are not yet Blueprint-callable.** `ObjectRef`/`ActorRef`/`ComponentRef`
   exist in the schema (as the contract requires) but `blueprint_ir::cpp_type` refuses them
   rather than inventing a runtime representation. Wiring belongs to the phase that lands
   `ObjectId`.

## Validation

| Step | Command | Result |
| --- | --- | --- |
| Targeted | `cargo test --locked object_model` | 15 passed, 0 failed |
| Targeted | `cargo test --locked reflection_schema` | 3 passed (editor bin) + 3 passed (header-tool bin), 0 failed |
| Targeted | `cargo test --locked header_extract` | 4 passed, 0 failed |
| Full suite | `cargo test --locked` (bin `epok-editor`) | **404 passed, 7 failed, 33 ignored** — baseline was 387/7/33; the same 7 pre-existing failures, +17 new tests, no new failure |
| Full suite | `cargo test --locked --bin epok-header-tool` | 9 passed, 0 failed (the editor bin fails first, so this bin is run separately) |
| Lint | `cargo clippy --locked --all-targets -- -D warnings` | 25 errors, byte-identical to the pre-existing baseline on this branch (verified with `git stash`); **no new lint from this phase** |
| Format | `cargo fmt --all` | new/modified code is rustfmt-clean; the repository is not fmt-clean at baseline (41 unrelated files), so every unrelated reformatting hunk was reverted |
| Build | `cargo build --locked --bins` | success (both `epok-editor` and `epok-header-tool`, which compiles `header_extract.rs`) |
| Host C++ | `python3 tests/runtime/verify_spatial.py` | not run — no `runtime/` change in this phase; the runtime agent owns that gate |
| MIPS / emulator | — | not run, no SDK on this machine (p0-baseline.md) |

Pre-existing failures, unchanged: `audio_contract_tests::audio_authoring_valid_target_error_and_tool_identity`,
`audio_contract_tests::audio_legacy_golden_outputs`,
`console::autoscroll_preserves_read_only_selection_and_copy`,
`editor::tests::attachment_undo_restores_overrides_and_rejects_intervening_edits`,
`editor::tests::invalid_source_marks_outputs_stale_and_keeps_running_snapshot`,
`gui::interaction_tests::scene_clicks_and_hierarchy_context_menu_use_real_imgui_events`,
`file_watch::tests::native_events_detect_create_rename_delete_and_preserved_timestamp`.

## Open items for the next phase

- Wire `Model` into the editor: component picker, Inspector, scene load, parent selectors.
  Nothing calls it yet, so `src/object_model.rs` carries a module-level `#![allow(dead_code)]`
  and `Registry::model()` an `#[allow(dead_code)]`; both should be removed once consumers exist.
- Blueprint asset version 4 (`family` hint and typed reference pins) and the Blueprint ABI
  for `ObjectRef`/`ActorRef`/`ComponentRef`.
- Replace `KNOWN_CAPABILITIES` with the exporter's target capability table.
- Run the Clang extractor on an SDK machine to validate the `EPOK_CLASS`/`EPOK_COMPONENT`
  path end to end.
