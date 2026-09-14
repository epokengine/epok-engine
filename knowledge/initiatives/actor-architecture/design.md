# Design contract — Object / Actor / Component model

The actor-only decision in [actor-only.md](actor-only.md), accepted on 2026-09-14,
supersedes this document's Entity/Behaviour compatibility requirements.

This is the binding contract for every phase of the actor-architecture initiative. Agents
implementing a phase follow these names, IDs and rules; deviations are recorded here first.

## 1. Vocabulary

| Term | Meaning |
| --- | --- |
| Object | Root of the reflected class system (`epok::Object`). Not every struct derives from it. |
| Actor | Placeable/spawnable unit owned by a Level. Abstract; has no universal transform. |
| Actor3D / Actor2D / UIActor | Concrete minimal actors per domain; their root component defines the spatial/layout contract. |
| SceneScriptActor | One per loaded Level; created by the loader, never placed manually. |
| ActorComponent | Behaviour/data attached to exactly one Actor; has its own identity and lifetime. |
| Domain | `World3D`, `World2D`, `UI`, or `None` (domain-less logic). Derived from the class, never stored as truth. |
| Family | `Object`, `Actor`, `Component`, `World`, `Level`, `Behaviour` (legacy). A parent change never crosses families. |
| LegacyEntityStorage | The old `epok::Object` slot record. `epok::Entity` remains its alias. |

## 2. Stable identities of native base classes

The runtime header and the Rust constants must carry exactly these UUIDs. A unit test in
`src/object_model.rs` parses `runtime/object_model.hpp` and checks every ID below.

| Class | Family | Domain | Flags | Id |
| --- | --- | --- | --- | --- |
| `epok::Object` | Object | None | Abstract | `26a54c0d-ca81-41ca-aecc-d0346a6357d2` |
| `epok::Actor` | Actor | None | Abstract, Blueprintable | `6e6efc67-66c4-4dae-90f8-7c8c4e612dea` |
| `epok::Actor3D` | Actor | World3D | Blueprintable, Placeable, Spawnable | `fc24ce9b-558c-49de-bc35-e040f350e486` |
| `epok::Actor2D` | Actor | World2D | Blueprintable, Placeable, Spawnable | `5308054e-0aaa-4d53-963b-440cf0c71916` |
| `epok::UIActor` | Actor | UI | Blueprintable, Placeable, Spawnable | `b09bd2fa-8b09-4c0f-a33a-c3ca08b21d8f` |
| `epok::SceneScriptActor` | Actor | None | Blueprintable, SceneManaged | `b4c08aa0-fa85-4abf-8f45-7501e1c8a040` |
| `epok::ActorComponent` | Component | None | Abstract, Blueprintable | `2e5021ee-d14d-4d77-9112-455f29d639d2` |
| `epok::SceneComponent3D` | Component | World3D | Blueprintable, Root, Owners=World3D | `ed73d249-b6cb-4a3c-a0e8-696de55e286f` |
| `epok::SceneComponent2D` | Component | World2D | Blueprintable, Root, Owners=World2D | `27887770-a779-4a16-863c-5abd32786cfa` |
| `epok::UIComponent` | Component | UI | Abstract, Blueprintable, Owners=UI | `83bffb60-2c33-4be1-9041-8c8f4c395b86` |
| `epok::RectTransformComponent` | Component | UI | Blueprintable, Root, Owners=UI | `dc805165-6c65-48dc-8ff8-4a638a5d21df` |
| `epok::AudioComponent` | Component | None | Blueprintable, Owners=World3D\|World2D\|UI, Cardinality=Multiple, Capability=audio | `7f0eb028-5301-4ac7-b93b-5665fab12b20` |
| `epok::World` | World | None | Abstract | `ab2d72b4-fcf6-4b30-8494-43fc0a7cf46c` |
| `epok::Level` | Level | None | Abstract | `b4683321-83e2-4b90-bd87-bb314f9eda2e` |
| `epok::LegacyBehaviourComponent` | Component | None | Owners=World3D\|World2D\|UI, Cardinality=Multiple | `430cb0ca-21c3-420c-96a3-da2f7b99781a` |
| `epok::Behaviour` (existing) | Behaviour | None | Blueprintable | `8ec3a9d4-13f1-4727-b4b1-591202c82490` |

Reserved for P3/P8 component adapters (use these when the class is introduced):

| Class | Id |
| --- | --- |
| `epok::Mesh3DComponent` | `580f99b1-c905-4f96-b34f-807c51335ba0` |
| `epok::Sprite3DComponent` | `0b68656b-364b-441f-ad86-ddc0408e80a0` |
| `epok::Camera3DComponent` | `9fe2abff-f285-435d-976d-825c5db5420a` |
| `epok::Light3DComponent` | `e18d296c-5820-4557-9386-8b12b9ca37a2` |
| `epok::Collider3DComponent` | `e8431f94-e526-4d7d-aace-8c8fae7955e6` |
| `epok::Sprite2DComponent` | `892ac5c9-eae8-48b8-853a-db93d20a537b` |
| `epok::Camera2DComponent` | `7245e90e-d900-4976-b738-0f95caa1d15d` |
| `epok::Collider2DComponent` | `aad9ca37-7c63-4ca9-8a23-5f4ff49e35fe` |
| `epok::CanvasComponent` | `f2cfb26b-af53-4a46-9d91-1debea72e01b` |
| `epok::ImageComponent` | `d04c78d6-23bd-40d7-88f1-b1afc1b05b5b` |
| `epok::TextComponent` | `fd7f11d1-7ccf-40e8-a7ea-56d89deb3f34` |
| `epok::ProgressBarComponent` | `28bf5245-5d80-4cba-a77d-1d74479ac276` |

Compact runtime IDs come from the existing `crate::blueprint_refs::compact_id`; the cook
still rejects collisions.

## 3. Annotation syntax (extractor contract, reflection schema 8)

`EPOK_CLASS(...)` options are comma separated `Key` or `Key=Value` tokens. Existing tokens
(`Blueprintable`, `Id="uuid"`, `TimelineRequires=X`) keep their meaning. New tokens:

| Token | Applies to | Meaning |
| --- | --- | --- |
| `Family=Object\|Actor\|Component\|World\|Level` | native bases only | Declares the family root. Derived classes inherit the family from their parent; declaring a different family than the parent is an extraction error. |
| `Domain=World3D\|World2D\|UI\|None` | actors, components | Actor domain or component's own domain. Derived classes inherit; they may not change it. |
| `Placeable` | actors | May be placed in a map from the editor. |
| `Spawnable` | actors | May be spawned at runtime (`SpawnActor`). |
| `SceneManaged` | actors | Created by the Level loader; never placeable/spawnable. |
| `Root` | components | May be an actor's root (spatial/layout root). |
| `Owners=World3D\|World2D\|UI` | components | Domains of actors that may own it. `\|` separated, no spaces. A descendant may only narrow this set. |
| `Requires=ClassName[\|ClassName]` | components | Component classes that must exist on the same owner. |
| `Excludes=ClassName[\|ClassName]` | components | Component classes that may not coexist. |
| `Cardinality=Single\|Multiple` | components | Default `Single`. |
| `Capability=name` | components | Target capability the cooked build must provide (`audio`, `physics2d`, ...). Unknown on target → build diagnostic. |
| `Abstract` | any | Explicit abstract flag for bases that have no pure virtuals. `is_abstract_record` still sets it too. |

`EPOK_COMPONENT(...)` (new macro, empty on MIPS) annotates an instance field whose type is a
reflected component class. Options: `Root`, `Name="display"`, `AttachTo=fieldName`. The
extractor records it in `Class.default_components`. Constructors remain defaulted; defaults
stay declarative and inspectable.

The extractor implementation parses option tokens in a pure function
`header_extract::class_options(&[String]) -> Result<ClassOptions, String>` that has unit
tests without libclang.

## 4. Reflection schema additions (`src/reflection_schema.rs`)

`SCHEMA_VERSION` becomes 8. All new fields are `#[serde(default)]` so version 7 manifests
still deserialize; the cache key already includes the schema version.

```rust
pub enum ClassFamily { Object, Actor, Component, World, Level, Behaviour }   // Default: Behaviour
pub enum Domain { None, World3D, World2D, UI }                              // Default: None
pub struct Placement { pub placeable: bool, pub spawnable: bool, pub scene_managed: bool }
pub enum Cardinality { Single, Multiple }
pub struct ComponentContract {
    pub owners: BTreeSet<Domain>,       // empty = inherits parent; on the root base it means "declared"
    pub requires: Vec<String>,          // cpp_name of required component classes
    pub excludes: Vec<String>,
    pub cardinality: Cardinality,
    pub can_root: bool,
    pub capabilities: BTreeSet<String>,
}
pub struct DefaultComponent { pub id: String, pub field: String, pub class: String /* cpp_name */, pub root: bool, pub attach_to: Option<String>, pub name: Option<String> }
// Added to Class:
pub family: Option<ClassFamily>,               // as declared; None = inherit
pub domain: Option<Domain>,                    // as declared; None = inherit
pub placement: Placement,
pub component: Option<ComponentContract>,
pub default_components: Vec<DefaultComponent>,
pub explicit_abstract: bool,
// Added to Type:
ObjectRef { class: Option<String> }, ActorRef { class: Option<String> }, ComponentRef { class: Option<String> }
```

`EntityRef` stays for legacy content. `ClassRef { base }` continues; `SpawnActor` accepts
only `ClassRef` whose base resolves to the Actor family.

## 5. Host model (`src/object_model.rs`)

Single source of truth for compatibility. Every consumer (component picker, Inspector,
scene load, Blueprint compiler, exporter, C++ API generation, MCP) calls these functions;
none re-implements the rules.

```rust
pub struct ClassModel { pub id, pub cpp_name, pub family: ClassFamily, pub domain: Domain, pub placement, pub component: Option<ComponentContract>, pub abstract_class, pub blueprintable, pub final_class, pub default_components }
pub struct Model { classes: BTreeMap<String /*id*/, ClassModel> }
impl Model {
    pub fn from_registry(registry: &blueprint::Registry) -> Result<Model, Vec<Diagnostic>>; // resolves inheritance, validates roots/cycles/family/domain/narrowing
    pub fn class(&self, id_or_cpp_name) -> Option<&ClassModel>;
    pub fn is_a(&self, class, base) -> bool;
    pub fn eligible_parents(&self, author: &Extension, family: ClassFamily) -> Vec<&ClassModel>;
    pub fn validate_reparent(&self, class, new_parent) -> Result<(), Vec<Diagnostic>>;  // same family, domain compatible, blueprintable, not final, no cycle
    pub fn validate_component(&self, owner_class, component_class) -> Result<(), Diagnostic>;
    pub fn validate_component_set(&self, owner_class, components: &[ComponentSpec]) -> Result<(), Vec<Diagnostic>>; // unique root, root required for spatial domains, requires/excludes, cardinality, capabilities
    pub fn placeable(&self) -> impl Iterator<Item=&ClassModel>;
    pub fn scene_script_parents(&self) -> impl Iterator<Item=&ClassModel>;  // SceneScriptActor subclasses only
}
pub struct ComponentSpec { pub id: Uuid, pub class: String, pub root: bool }
pub struct Diagnostic { pub code: &'static str, pub message: String, pub class: Option<String> }
```

Rules enforced by `from_registry`:

- Exactly one native root per family among the base IDs of section 2; every class reaches a
  native class through its parent chain (Blueprints included). C++ → C++ → BP → BP is legal;
  BP → C++ (a native class deriving from generated Blueprint C++) is rejected.
- Family and domain are inherited; redeclaring a different value is a diagnostic.
- A component descendant may narrow `owners`, never widen it.
- Abstract, non-instantiable bases remain in ancestry tables.
- Tick is not assumed; `wants_tick` is a runtime property, not a reflection flag.

## 6. Runtime contract (`runtime/object_model.hpp`, C++20, no RTTI/exceptions/heap)

Included at the end of `runtime/epok.hpp` after the namespace closes (like
`effect_types.hpp`) so the extractor reflects the bases automatically. Must compile for
MIPS (`-fno-rtti -fno-exceptions -ffreestanding`) and for host tests with clang++/g++/MSVC.

```cpp
namespace epok {
struct ObjectId { uint16_t index=0xffff; uint16_t generation=0; };       // compact; 0xffff = null
enum class ObjectState : uint8_t { Unused, Reserved, Initialized, Playing, EndingPlay, Destroyed };
enum class EndPlayReason : uint8_t { Destroyed, LevelUnloaded, Quit };
enum class Domain : uint8_t { None, World3D, World2D, UI };
enum class Family : uint8_t { Object, Actor, Component, World, Level };
struct ClassDescriptor { uint64_t id, parent; Family family; Domain domain; uint8_t owners_mask; uint8_t flags; Object* (*create)(); void (*destroy)(Object*); };
// flags bits: Abstract=1, Placeable=2, Spawnable=4, SceneManaged=8, Root=16, Multiple=32
extern const ClassDescriptor object_classes[]; extern const size_t object_class_count;   // cooked
const ClassDescriptor* find_object_class(uint64_t id); bool object_class_is_a(uint64_t child, uint64_t parent);

class Object {            // virtual dtor; class_id() virtual; ObjectId self; ObjectState state
public: virtual uint64_t class_id() const = 0; ObjectId id() const; ObjectState state() const; ... };
class Actor : public Object {  // owner Level, name[33], root component id, components (ObjectId[8]), logical parent ActorId, active flag
  EPOK_FUNCTION(BlueprintEvent) virtual void begin_play() {}
  EPOK_FUNCTION(BlueprintEvent) virtual void tick(Fixed dt) {}
  EPOK_FUNCTION(BlueprintEvent) virtual void end_play(EndPlayReason) {}
  EPOK_FUNCTION(BlueprintEvent) virtual void on_enable() {}  EPOK_FUNCTION(BlueprintEvent) virtual void on_disable() {}
  Entity* entity();      // legacy adapter: canonical slot of the root scene component, nullptr for UI/2D-less actors
};
class Actor3D : public Actor { SceneComponent3D root storage lives in the actor pool, registered as root during defaults };
class Actor2D, UIActor likewise with SceneComponent2D / RectTransformComponent roots.
class SceneScriptActor : public Actor { };
class ActorComponent : public Object { ObjectId owner; Actor* get_owner(); begin_play/tick/end_play/on_enable/on_disable events }
class SceneComponent3D : public ActorComponent { Transform* transform (view over canonical LegacyEntityStorage slot or own storage); ObjectId attach_parent; }
class SceneComponent2D : public ActorComponent { Transform2D { Fixed position[2], rotation, scale[2]; int16_t draw_order; } ; attach_parent }
class UIComponent : public ActorComponent {}; class RectTransformComponent : public UIComponent { RectTransform* rect view or storage }
class AudioComponent : public ActorComponent { AudioSource* source view; play/stop/is_playing forward to the existing audio service }
class LegacyBehaviourComponent : public ActorComponent { Behaviour* behaviour; forwards start/update/frame_update/on_enable/on_disable/on_destroy }
class Level : public Object { bounded actor table; ObjectId scene_script; spawn/destroy batch API; deferred queue processed at batch end }
class World : public Object { active Level; services context }
template<class T, size_t N> struct ObjectPool { aligned storage, used[], construct/destroy concrete T, no slicing };
struct ObjectRegistry { slots {Object*, generation, state, class}; capacity fixed by cook; acquire/release; resolve<T>(ObjectId) with generation and is_a check; dispatch depth counter for callback quarantine };
}
```

Lifecycle order (section 4.7 of the plan) implemented in `Level::spawn_batch`:

1. reserve all instances and components (fail → release everything, no orphans);
2. apply defaults/templates/overrides, set roots and owners;
3. resolve persistent references to `ObjectId`;
4. initialize/register (state `Initialized`);
5. `begin_play` components then owning actor, in stable batch order;
6. SceneScriptActor `begin_play` after initial level actors;
7. tick actors/components that registered for tick, then scene script tick;
8. exit: scene script `end_play` first while actors are alive; then deactivate/cancel, component `end_play`, actor `end_play`; release storage only when no dispatch is active.

`end_play` runs exactly once with a reason. Spawn/destroy requested inside a callback is
deferred to the end of the current batch. Reusing a slot bumps the generation so old
`ObjectId`s never resolve. Handles are never stored in persisted data.

Host tests live in `tests/runtime/object_model.cpp` and are added to `SOURCES` in
`tests/runtime/verify_spatial.py`. They compile against a stub environment like
`tests/runtime/lifecycle.cpp` does (no PsyQo hardware).

## 7. Documents (scene version 5, Blueprint asset version 4)

`Scene` gains `actors: Vec<ActorInstance>` and `scene_script: Option<SceneScript>`; both
`#[serde(default)]`. Loading never rewrites bytes; the in-memory view derives actors from
legacy entities (`Scene::actor_view()`) until the document is saved with actor content, at
which point `version` becomes 5. Version > 5 is rejected, never read as empty.

```rust
pub struct ActorInstance { pub id: Uuid, pub class: ClassReference { name, class_id }, pub name: String, pub active: bool,
    pub logical_parent: Option<Uuid>, pub attach: Option<Attachment { actor: Uuid, component: Option<Uuid> }>,
    pub components: Vec<ComponentInstance>, pub properties: BTreeMap<String, Value>, pub overrides: BTreeSet<String>,
    pub legacy_entity: Option<Uuid> /* migration provenance */ }
pub struct ComponentInstance { pub id: Uuid, pub class: ClassReference, pub name: String, pub root: bool, pub attach_parent: Option<Uuid>,
    pub properties: BTreeMap<String, Value>, pub overrides: BTreeSet<String>, pub inherited: bool }
pub struct SceneScript { pub parent: ClassReference /* SceneScriptActor subclass */, pub blueprint: blueprint_asset::BlueprintAsset }
```

Legacy mapping (plan section 5): Mesh/Camera/Empty/Light → `Actor3D` + component adapters;
Sprite → `Actor3D` + `Sprite3DComponent`; Canvas/RectTransform → `UIActor` + UI components;
hybrid 3D+UI entities → compatibility view with a conversion diagnostic; `Behaviour` bindings
→ `LegacyBehaviourComponent`; old Blueprint classes used by Spawn/IsA/ClassRef → redirect
table `blueprint_refs::redirects`.

## 8. Editor

`Editor.scene_2d: bool` is replaced by `SceneViewMode { ThreeD, TwoD, UI }` with
`scene_2d()` compatibility accessors; MCP keeps accepting `scene_2d` (true → `UI`) and adds
`view_mode`. Hierarchy panels filter by the domain of each actor; filtering never renumbers
or edits actors. The map root node opens Map Settings (scene Blueprint parent selector and
"Open Scene Blueprint").

## 9. Versions reserved

| Contract | New value |
| --- | --- |
| Reflection schema | 8 |
| Scene document | 5 |
| Blueprint asset | 4 (adds `family` hint and typed reference pins) |
| Native HUD preview protocol | bump when its ABI changes (P9) |

## 10. Validation ladder per change

1. targeted `cargo test --locked <module>`;
2. `python3 tests/runtime/verify_spatial.py` (host C++ contracts, includes object model tests);
3. `cargo clippy --locked --all-targets -- -D warnings` and `cargo fmt --all -- --check`;
4. `cargo build --locked --bins`;
5. MIPS build/export and emulator runs: only on a machine with the SDK; report as "not run" otherwise.
