//! Scene document representation of the Object / Actor / Component model.
//!
//! This module owns three things and nothing else:
//!
//! 1. the persisted shapes of `Scene::actors` / `Scene::scene_script`
//!    (design.md section 7, scene document version 5);
//! 2. `validate`, the document-side rules, which delegate every compatibility
//!    question to [`crate::object_model::Model`] and never re-implement it;
//! 3. `actor_view`, the read-only migration that derives actors from legacy
//!    entities in memory so the editor can show the actor model before any
//!    document is rewritten.
//!
//! Nothing here ever writes to a scene: loading a legacy document must leave its
//! bytes untouched, and the version only rises to 5 once actor content is
//! actually authored (`Scene::upgrade_entity_ids`).
//!
//! P4 lands the document and the view ahead of their consumers: the editor UI is
//! P7 and cooking actors into the C++ tables is P10, so most of the public
//! surface is currently only exercised by the tests in this module.
#![allow(dead_code)]

use crate::{
    object_model::{self, Diagnostic, Model},
    reflection_schema::{ClassFamily, Domain},
    scene::Scene,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

/// Highest scene document version this editor writes and accepts.
pub const SCENE_VERSION: u32 = 5;

/// Salt of the deterministic actor-view derivation. It is part of the algorithm:
/// changing it renumbers every derived actor, so it never changes.
const VIEW_SALT: &[u8] = b"epok actor view identity v1";

fn is_false(value: &bool) -> bool {
    !*value
}
fn default_true() -> bool {
    true
}

/// A reference to a reflected class. `name` is the readable `cpp_name` and is
/// what a human edits; `class_id` is the stable identity used to survive
/// renames and is absent for documents written before a class had one.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ClassReference {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub class_id: Option<String>,
}
impl ClassReference {
    pub fn new(name: &str, class_id: &str) -> Self {
        Self {
            name: name.to_owned(),
            class_id: Some(class_id.to_owned()),
        }
    }
    /// Resolves through the model by identity first, then by readable name.
    pub fn resolve<'a>(&self, model: &'a Model) -> Option<&'a object_model::ClassModel> {
        self.class_id
            .as_deref()
            .and_then(|id| model.class(id))
            .or_else(|| model.class(&self.name))
    }
}

/// Spatial attachment of an actor to a component of another actor. `component`
/// absent means "the target actor's root component".
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Attachment {
    pub actor: Uuid,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component: Option<Uuid>,
}

/// One component on an actor instance.
///
/// Like [`crate::scene::ScriptBinding`] this type deliberately does not deny
/// unknown members: authored values live in `properties` and orphan values from
/// a class that changed shape are retained there instead of being dropped.
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct ComponentInstance {
    pub id: Uuid,
    pub class: ClassReference,
    pub name: String,
    /// Exactly one component of a spatial actor is its root.
    #[serde(default, skip_serializing_if = "is_false")]
    pub root: bool,
    /// Component-to-component attachment inside the same actor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attach_parent: Option<Uuid>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub properties: BTreeMap<String, Value>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub overrides: BTreeSet<String>,
    /// Contributed by the class (an `EPOK_COMPONENT` default) rather than added
    /// on this instance. Inherited components may be overridden, never removed.
    #[serde(default, skip_serializing_if = "is_false")]
    pub inherited: bool,
}
impl ComponentInstance {
    pub fn new(id: Uuid, class: ClassReference, name: &str) -> Self {
        Self {
            id,
            class,
            name: name.to_owned(),
            root: false,
            attach_parent: None,
            properties: BTreeMap::new(),
            overrides: BTreeSet::new(),
            inherited: false,
        }
    }
    fn with(mut self, key: &str, value: Value) -> Self {
        self.properties.insert(key.to_owned(), value);
        self.overrides.insert(key.to_owned());
        self
    }
    fn rooted(mut self) -> Self {
        self.root = true;
        self
    }
}
impl<'de> Deserialize<'de> for ComponentInstance {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Raw {
            id: Uuid,
            class: ClassReference,
            name: String,
            #[serde(default)]
            root: bool,
            #[serde(default)]
            attach_parent: Option<Uuid>,
            #[serde(default)]
            properties: BTreeMap<String, Value>,
            #[serde(default)]
            overrides: Option<BTreeSet<String>>,
            #[serde(default)]
            inherited: bool,
        }
        let raw = Raw::deserialize(deserializer)?;
        // Documents without an explicitness set retain every persisted value as
        // an override, exactly like a legacy `ScriptBinding`.
        let mut overrides = raw.overrides.unwrap_or_default();
        overrides.extend(raw.properties.keys().cloned());
        Ok(Self {
            id: raw.id,
            class: raw.class,
            name: raw.name,
            root: raw.root,
            attach_parent: raw.attach_parent,
            properties: raw.properties,
            overrides,
            inherited: raw.inherited,
        })
    }
}

/// One placed actor. `logical_parent` is the hierarchy the author sees; `attach`
/// is the spatial relationship and only exists between actors of one domain.
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct ActorInstance {
    pub id: Uuid,
    pub class: ClassReference,
    pub name: String,
    #[serde(default = "default_true")]
    pub active: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logical_parent: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attach: Option<Attachment>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub components: Vec<ComponentInstance>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub properties: BTreeMap<String, Value>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub overrides: BTreeSet<String>,
    /// Migration provenance: the legacy entity this actor was derived from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legacy_entity: Option<Uuid>,
}
impl ActorInstance {
    pub fn new(id: Uuid, class: ClassReference, name: &str) -> Self {
        Self {
            id,
            class,
            name: name.to_owned(),
            active: true,
            logical_parent: None,
            attach: None,
            components: Vec::new(),
            properties: BTreeMap::new(),
            overrides: BTreeSet::new(),
            legacy_entity: None,
        }
    }
    pub fn root(&self) -> Option<&ComponentInstance> {
        self.components.iter().find(|c| c.root)
    }
}
impl<'de> Deserialize<'de> for ActorInstance {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Raw {
            id: Uuid,
            class: ClassReference,
            name: String,
            #[serde(default = "default_true")]
            active: bool,
            #[serde(default)]
            logical_parent: Option<Uuid>,
            #[serde(default)]
            attach: Option<Attachment>,
            #[serde(default)]
            components: Vec<ComponentInstance>,
            #[serde(default)]
            properties: BTreeMap<String, Value>,
            #[serde(default)]
            overrides: Option<BTreeSet<String>>,
            #[serde(default)]
            legacy_entity: Option<Uuid>,
        }
        let raw = Raw::deserialize(deserializer)?;
        let mut overrides = raw.overrides.unwrap_or_default();
        overrides.extend(raw.properties.keys().cloned());
        Ok(Self {
            id: raw.id,
            class: raw.class,
            name: raw.name,
            active: raw.active,
            logical_parent: raw.logical_parent,
            attach: raw.attach,
            components: raw.components,
            properties: raw.properties,
            overrides,
            legacy_entity: raw.legacy_entity,
        })
    }
}

/// The scene Blueprint: one `SceneScriptActor` subclass owned by the map.
///
/// `blueprint` is a full [`crate::blueprint_asset::BlueprintAsset`] embedded in
/// the map rather than a separate `.epokbp` file, because it has no life outside
/// this scene.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SceneScript {
    pub parent: ClassReference,
    pub blueprint: crate::blueprint_asset::BlueprintAsset,
}
// `BlueprintAsset` intentionally does not derive `PartialEq` (it carries free
// JSON), so scene equality compares the serialized document instead.
impl PartialEq for SceneScript {
    fn eq(&self, other: &Self) -> bool {
        self.parent == other.parent
            && serde_json::to_value(&self.blueprint).ok()
                == serde_json::to_value(&other.blueprint).ok()
    }
}

// ---------------------------------------------------------------------------
// Identity remapping (duplicate branch / duplicate scene)
// ---------------------------------------------------------------------------

/// `epok::SceneComponent2D` → `SceneComponent2D`. The editor labels classes by
/// their unqualified name; the document always keeps the qualified one.
pub fn short_class_name(cpp_name: &str) -> &str {
    cpp_name.rsplit("::").next().unwrap_or(cpp_name)
}

/// A component name no other component of `actor` already uses. Component names
/// are unique inside one actor, never across the map.
pub fn unique_component_name(actor: &ActorInstance, base: &str) -> String {
    let mut name = base.to_owned();
    let mut suffix = 1;
    while actor.components.iter().any(|c| c.name == name) {
        name = format!("{base}.{suffix:03}");
        suffix += 1;
    }
    name
}

/// Old UUID → new UUID for actors and components. External asset UUIDs are never
/// members of the table, so they survive a remap untouched.
pub type IdentityMap = BTreeMap<Uuid, Uuid>;

/// Fresh identities for every actor and component in `actors`.
pub fn fresh_identities(actors: &[ActorInstance]) -> IdentityMap {
    let mut map = IdentityMap::new();
    for actor in actors {
        map.insert(actor.id, Uuid::new_v4());
        for component in &actor.components {
            map.insert(component.id, Uuid::new_v4());
        }
    }
    map
}

/// Rewrites every internal reference of one actor through `map`. References to
/// identities outside the table (another actor that was not copied, an asset
/// UUID inside `properties`) are left exactly as they are.
pub fn remap_actor(actor: &mut ActorInstance, map: &IdentityMap) {
    if let Some(id) = map.get(&actor.id) {
        actor.id = *id;
    }
    if let Some(parent) = actor.logical_parent.as_mut()
        && let Some(id) = map.get(parent)
    {
        *parent = *id;
    }
    if let Some(attach) = actor.attach.as_mut() {
        if let Some(id) = map.get(&attach.actor) {
            attach.actor = *id;
        }
        if let Some(component) = attach.component.as_mut()
            && let Some(id) = map.get(component)
        {
            *component = *id;
        }
    }
    for component in &mut actor.components {
        if let Some(id) = map.get(&component.id) {
            component.id = *id;
        }
        if let Some(parent) = component.attach_parent.as_mut()
            && let Some(id) = map.get(parent)
        {
            *parent = *id;
        }
    }
}

/// Rewrites the scene Blueprint: a fresh asset id plus every UUID literal that
/// names a remapped actor or component. The asset is walked as JSON so pins,
/// defaults and variables are covered without duplicating the graph schema.
pub fn remap_scene_script(script: &mut SceneScript, map: &IdentityMap, fresh_id: bool) {
    let Ok(mut value) = serde_json::to_value(&script.blueprint) else {
        return;
    };
    remap_json(&mut value, map);
    if fresh_id {
        value["id"] = Value::String(Uuid::new_v4().to_string());
    }
    if let Ok(asset) = serde_json::from_value(value) {
        script.blueprint = asset;
    }
}

fn remap_json(value: &mut Value, map: &IdentityMap) {
    match value {
        Value::String(text) => {
            if let Ok(id) = Uuid::parse_str(text)
                && let Some(new) = map.get(&id)
            {
                *text = new.to_string();
            }
        }
        Value::Array(items) => items.iter_mut().for_each(|item| remap_json(item, map)),
        Value::Object(fields) => fields
            .iter_mut()
            .for_each(|(_, item)| remap_json(item, map)),
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

/// Document rules for `Scene::actors` / `Scene::scene_script`.
///
/// Without a `model` only the structural half runs: identity uniqueness, class
/// names, reference targets and cycles. That is the offline path used by tests
/// and by any tool that has no reflection registry. Every compatibility question
/// — does the class exist, is it an actor, may it be placed, is this component
/// legal on this actor — is asked of [`Model`] and is never re-implemented here.
pub fn validate(scene: &Scene, model: Option<&Model>) -> Result<(), Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    if scene.actors.is_empty() && scene.scene_script.is_none() {
        return Ok(());
    }
    let mut identities: BTreeSet<Uuid> = scene
        .entities
        .iter()
        .filter(|e| !e.id.is_nil())
        .map(|e| e.id)
        .collect();
    let entity_ids = identities.clone();
    let mut actors: BTreeMap<Uuid, &ActorInstance> = BTreeMap::new();
    let mut components: BTreeMap<Uuid, Uuid> = BTreeMap::new(); // component -> owning actor

    for actor in &scene.actors {
        if actor.id.is_nil() || !identities.insert(actor.id) {
            diagnostics.push(Diagnostic {
                code: "duplicate-identity",
                message: format!(
                    "Actor UUID {} is missing or already used in this scene. Duplicate through the editor to assign new identities.",
                    actor.id
                ),
                class: Some(actor.class.name.clone()),
            });
        }
        actors.insert(actor.id, actor);
        if actor.name.trim().is_empty() || actor.name.len() > 128 {
            diagnostics.push(Diagnostic {
                code: "actor-name",
                message: "Actor names must contain 1–128 bytes".into(),
                class: Some(actor.class.name.clone()),
            });
        }
        check_class_name(&actor.class, &mut diagnostics);
        check_overrides(
            &actor.properties,
            &actor.overrides,
            &actor.name,
            &mut diagnostics,
        );
        if actor
            .legacy_entity
            .is_some_and(|id| !entity_ids.contains(&id))
        {
            diagnostics.push(Diagnostic {
                code: "dangling-legacy-entity",
                message: format!(
                    "Actor `{}` records provenance from entity {} which no longer exists",
                    actor.name,
                    actor.legacy_entity.expect("checked")
                ),
                class: Some(actor.class.name.clone()),
            });
        }
        let mut roots = 0;
        for component in &actor.components {
            if component.id.is_nil() || !identities.insert(component.id) {
                diagnostics.push(Diagnostic {
                    code: "duplicate-identity",
                    message: format!(
                        "Component UUID {} is missing or already used in this scene.",
                        component.id
                    ),
                    class: Some(component.class.name.clone()),
                });
            }
            components.insert(component.id, actor.id);
            check_class_name(&component.class, &mut diagnostics);
            check_overrides(
                &component.properties,
                &component.overrides,
                &component.name,
                &mut diagnostics,
            );
            roots += usize::from(component.root);
        }
        if roots > 1 {
            diagnostics.push(Diagnostic {
                code: "duplicate-root",
                message: format!("Actor `{}` declares {roots} root components", actor.name),
                class: Some(actor.class.name.clone()),
            });
        }
        for component in &actor.components {
            if let Some(parent) = component.attach_parent
                && !actor.components.iter().any(|c| c.id == parent)
            {
                diagnostics.push(Diagnostic {
                    code: "dangling-attach-parent",
                    message: format!(
                        "Component `{}` attaches to {parent}, which is not a component of `{}`",
                        component.name, actor.name
                    ),
                    class: Some(component.class.name.clone()),
                });
            }
        }
    }
    if let Some(script) = &scene.scene_script {
        check_class_name(&script.parent, &mut diagnostics);
    }

    // References between actors.
    for actor in &scene.actors {
        if let Some(parent) = actor.logical_parent {
            if parent == actor.id || !actors.contains_key(&parent) {
                diagnostics.push(Diagnostic {
                    code: "dangling-logical-parent",
                    message: format!(
                        "Actor `{}` has a missing or self-referencing logical parent",
                        actor.name
                    ),
                    class: Some(actor.class.name.clone()),
                });
            } else if cycles(actor, &actors) {
                diagnostics.push(Diagnostic {
                    code: "cyclic-logical-parent",
                    message: format!("Actor `{}` is its own ancestor", actor.name),
                    class: Some(actor.class.name.clone()),
                });
            }
        }
        if let Some(attach) = &actor.attach {
            let Some(target) = actors.get(&attach.actor) else {
                diagnostics.push(Diagnostic {
                    code: "dangling-attachment",
                    message: format!(
                        "Actor `{}` attaches to actor {}, which is not in this scene",
                        actor.name, attach.actor
                    ),
                    class: Some(actor.class.name.clone()),
                });
                continue;
            };
            if attach.actor == actor.id {
                diagnostics.push(Diagnostic {
                    code: "self-attachment",
                    message: format!("Actor `{}` cannot attach to itself", actor.name),
                    class: Some(actor.class.name.clone()),
                });
            }
            if let Some(component) = attach.component
                && components.get(&component) != Some(&attach.actor)
            {
                diagnostics.push(Diagnostic {
                    code: "dangling-attachment",
                    message: format!(
                        "Actor `{}` attaches to component {component}, which does not belong to `{}`",
                        actor.name, target.name
                    ),
                    class: Some(actor.class.name.clone()),
                });
            }
            if let Some(model) = model {
                let (Some(mine), Some(theirs)) =
                    (actor.class.resolve(model), target.class.resolve(model))
                else {
                    continue;
                };
                if mine.domain != theirs.domain {
                    diagnostics.push(Diagnostic {
                        code: "attachment-domain",
                        message: format!(
                            "`{}` is {} and cannot attach to `{}`, which is {}",
                            actor.name,
                            mine.domain.label(),
                            target.name,
                            theirs.domain.label()
                        ),
                        class: Some(actor.class.name.clone()),
                    });
                }
            }
        }
    }

    if let Some(model) = model {
        validate_against_model(scene, model, &mut diagnostics);
    }
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

/// Joins the diagnostics of [`validate`] into the single sentence the scene
/// loader and the editor's error path expect.
pub fn validate_message(scene: &Scene, model: Option<&Model>) -> Result<(), String> {
    validate(scene, model).map_err(|diagnostics| {
        diagnostics
            .iter()
            .map(|d| d.message.as_str())
            .collect::<Vec<_>>()
            .join("; ")
    })
}

fn check_class_name(class: &ClassReference, diagnostics: &mut Vec<Diagnostic>) {
    if !crate::scripts::class_identifier(&class.name) {
        diagnostics.push(Diagnostic {
            code: "invalid-class-name",
            message: format!("`{}` is not a valid class name", class.name),
            class: Some(class.name.clone()),
        });
    }
}

fn check_overrides(
    properties: &BTreeMap<String, Value>,
    overrides: &BTreeSet<String>,
    owner: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for key in overrides {
        if !properties.contains_key(key) {
            diagnostics.push(Diagnostic {
                code: "override-without-value",
                message: format!("`{owner}` marks `{key}` as overridden but stores no value"),
                class: None,
            });
        }
    }
}

fn cycles(actor: &ActorInstance, actors: &BTreeMap<Uuid, &ActorInstance>) -> bool {
    let mut current = actor.logical_parent;
    for _ in 0..=actors.len() {
        let Some(id) = current else {
            return false;
        };
        if id == actor.id {
            return true;
        }
        current = actors.get(&id).and_then(|a| a.logical_parent);
    }
    true
}

fn validate_against_model(scene: &Scene, model: &Model, diagnostics: &mut Vec<Diagnostic>) {
    for actor in &scene.actors {
        let Some(class) = actor.class.resolve(model) else {
            diagnostics.push(Diagnostic {
                code: "unknown-class",
                message: format!(
                    "Actor `{}` refers to `{}`, which is not a reflected class",
                    actor.name, actor.class.name
                ),
                class: Some(actor.class.name.clone()),
            });
            continue;
        };
        if class.family != ClassFamily::Actor {
            diagnostics.push(Diagnostic {
                code: "not-an-actor",
                message: format!(
                    "`{}` is a {} class and cannot be placed in a map",
                    class.cpp_name,
                    class.family.label()
                ),
                class: Some(class.cpp_name.clone()),
            });
            continue;
        }
        if model.is_a(&class.id, object_model::SCENE_SCRIPT_ACTOR_ID) {
            diagnostics.push(Diagnostic {
                code: "scene-managed-actor",
                message: format!(
                    "`{}` is a SceneScriptActor; the level loader creates it and it is never placed as an actor",
                    class.cpp_name
                ),
                class: Some(class.cpp_name.clone()),
            });
        } else if !class.placement.placeable {
            diagnostics.push(Diagnostic {
                code: "not-placeable",
                message: format!("`{}` is not Placeable", class.cpp_name),
                class: Some(class.cpp_name.clone()),
            });
        }
        if !class.instantiable() {
            diagnostics.push(Diagnostic {
                code: "abstract-actor",
                message: format!("`{}` is abstract and cannot be placed", class.cpp_name),
                class: Some(class.cpp_name.clone()),
            });
        }
        let specs = actor
            .components
            .iter()
            .map(|c| object_model::ComponentSpec {
                id: c.id,
                class: c
                    .class
                    .class_id
                    .clone()
                    .unwrap_or_else(|| c.class.name.clone()),
                root: c.root,
            })
            .collect::<Vec<_>>();
        if let Err(found) = model.validate_component_set(&class.id, &specs) {
            diagnostics.extend(found);
        }
    }
    if let Some(script) = &scene.scene_script {
        match script.parent.resolve(model) {
            Some(parent) if model.is_a(&parent.id, object_model::SCENE_SCRIPT_ACTOR_ID) => {}
            Some(parent) => diagnostics.push(Diagnostic {
                code: "scene-script-parent",
                message: format!(
                    "The scene Blueprint derives from `{}`; it must derive from `epok::SceneScriptActor`",
                    parent.cpp_name
                ),
                class: Some(parent.cpp_name.clone()),
            }),
            None => diagnostics.push(Diagnostic {
                code: "unknown-class",
                message: format!(
                    "The scene Blueprint derives from `{}`, which is not a reflected class",
                    script.parent.name
                ),
                class: Some(script.parent.name.clone()),
            }),
        }
    }
}

// ---------------------------------------------------------------------------
// Legacy view
// ---------------------------------------------------------------------------

/// The actor model of a scene that still stores legacy entities, plus whatever
/// could not be represented faithfully.
#[derive(Clone, Debug, Default)]
pub struct ActorView {
    pub actors: Vec<ActorInstance>,
    pub diagnostics: Vec<Diagnostic>,
}
impl ActorView {
    pub fn actor(&self, legacy_entity: Uuid) -> Option<&ActorInstance> {
        self.actors
            .iter()
            .find(|a| a.legacy_entity == Some(legacy_entity))
    }
    pub fn codes(&self) -> Vec<&'static str> {
        self.diagnostics.iter().map(|d| d.code).collect()
    }
}

/// Deterministic derived identity for one role of one legacy entity.
///
/// The same derivation style as `Scene::upgrade_entity_ids`: a SHA-256 over a
/// fixed salt and the stable inputs, truncated to 16 bytes. Two reads of the
/// same document therefore produce the same actor UUIDs, on any machine and
/// after the folder has been moved.
pub fn derived_id(entity: Uuid, role: &str) -> Uuid {
    let mut digest = Sha256::new();
    digest.update(VIEW_SALT);
    digest.update(entity.as_bytes());
    digest.update([0u8]);
    digest.update(role.as_bytes());
    let bytes: [u8; 16] = digest.finalize()[..16].try_into().expect("16 bytes");
    Uuid::from_bytes(bytes)
}

/// `true` when the entity carries UI authoring data.
fn has_ui(entity: &crate::scene::Entity) -> bool {
    entity.canvas.is_some()
        || entity.rect.is_some()
        || entity.image.is_some()
        || entity.text.is_some()
        || entity.progress.is_some()
}

/// `true` when the entity carries 3D authoring data. `Empty` with nothing on it
/// is not 3D data by itself; it becomes a bare `Actor3D` only when no UI data
/// claims it.
fn has_3d(entity: &crate::scene::Entity) -> bool {
    entity.kind != "Empty"
        || entity.sprite.is_some()
        || entity.light.is_some()
        || entity.collider.is_some()
        || entity.editable_mesh.is_some()
        || entity.skeletal_mesh.is_some()
        || entity.particle_emitter.is_some()
        || entity.particle_effect.is_some()
        || entity.blob_shadow.is_some()
}

/// Derives the actor model of a legacy scene, in memory and without touching the
/// document (design.md section 7). Already-authored `scene.actors` are returned
/// unchanged and no entity is derived twice into them.
///
/// `model`, when present, is used to name Blueprint classes and to report the
/// component sets that the derived actors would not satisfy.
pub fn actor_view(scene: &Scene, model: Option<&Model>) -> ActorView {
    let mut view = ActorView {
        actors: scene.actors.clone(),
        ..Default::default()
    };
    let already: BTreeSet<Uuid> = scene
        .actors
        .iter()
        .filter_map(|a| a.legacy_entity)
        .collect();
    let derived: Vec<Option<(Uuid, Domain)>> = scene
        .entities
        .iter()
        .map(|entity| {
            (!entity.id.is_nil() && !already.contains(&entity.id))
                .then(|| (derived_id(entity.id, ""), domain_of(entity)))
        })
        .collect();

    for (index, entity) in scene.entities.iter().enumerate() {
        let Some((actor_id, domain)) = derived[index] else {
            continue;
        };
        let mut actor = derive_actor(entity, actor_id, domain, model, &mut view.diagnostics);
        if let Some(parent) = entity.parent
            && let Some((parent_actor, parent_domain)) = derived.get(parent).copied().flatten()
        {
            actor.logical_parent = Some(parent_actor);
            if parent_domain == domain {
                actor.attach = Some(Attachment {
                    actor: parent_actor,
                    component: Some(derived_id(scene.entities[parent].id, "root")),
                });
            }
        }
        view.actors.push(actor);
    }
    if let Some(model) = model {
        for actor in &view.actors {
            let Some(class) = actor.class.resolve(model) else {
                continue;
            };
            let specs = actor
                .components
                .iter()
                .map(|c| object_model::ComponentSpec {
                    id: c.id,
                    class: c
                        .class
                        .class_id
                        .clone()
                        .unwrap_or_else(|| c.class.name.clone()),
                    root: c.root,
                })
                .collect::<Vec<_>>();
            if let Err(found) = model.validate_component_set(&class.id, &specs) {
                view.diagnostics.extend(found);
            }
        }
    }
    view
}

fn domain_of(entity: &crate::scene::Entity) -> Domain {
    if has_ui(entity) && !has_3d(entity) {
        Domain::UI
    } else {
        Domain::World3D
    }
}

fn derive_actor(
    entity: &crate::scene::Entity,
    id: Uuid,
    domain: Domain,
    model: Option<&Model>,
    diagnostics: &mut Vec<Diagnostic>,
) -> ActorInstance {
    let class = actor_class(entity, domain, model, diagnostics);
    let mut actor = ActorInstance::new(id, class, &entity.name);
    actor.active = entity.active;
    actor.legacy_entity = Some(entity.id);
    let component = |role: &str, name: &str, class: (&str, &str)| {
        ComponentInstance::new(
            derived_id(entity.id, role),
            ClassReference::new(class.0, class.1),
            name,
        )
    };
    if domain == Domain::UI {
        // The root is always the RectTransform: a Canvas is an additional
        // component on the same actor, never a second root.
        let mut root = component(
            "root",
            "RectTransform",
            (
                "epok::RectTransformComponent",
                object_model::RECT_TRANSFORM_COMPONENT_ID,
            ),
        )
        .rooted();
        if let Some(rect) = &entity.rect {
            root = root.with("rect", json(rect));
        }
        actor.components.push(root);
        if let Some(canvas) = &entity.canvas {
            actor.components.push(
                component(
                    "canvas",
                    "Canvas",
                    ("epok::CanvasComponent", object_model::CANVAS_COMPONENT_ID),
                )
                .with("canvas", json(canvas)),
            );
        }
        if let Some(image) = &entity.image {
            actor.components.push(
                component(
                    "image",
                    "Image",
                    ("epok::ImageComponent", object_model::IMAGE_COMPONENT_ID),
                )
                .with("image", json(image)),
            );
        }
        if let Some(text) = &entity.text {
            actor.components.push(
                component(
                    "text",
                    "Text",
                    ("epok::TextComponent", object_model::TEXT_COMPONENT_ID),
                )
                .with("text", json(text)),
            );
        }
        if let Some(progress) = &entity.progress {
            actor.components.push(
                component(
                    "progress",
                    "ProgressBar",
                    (
                        "epok::ProgressBarComponent",
                        object_model::PROGRESS_BAR_COMPONENT_ID,
                    ),
                )
                .with("progress", json(progress)),
            );
        }
    } else {
        if has_ui(entity) {
            // Splitting the identity would break every persisted reference to
            // this entity, so the hybrid keeps its 3D actor and P7 asks the
            // author what the UI half should become.
            diagnostics.push(Diagnostic {
                code: "hybrid-entity-pending-conversion",
                message: format!(
                    "`{}` carries both 3D and UI data. It is shown as a 3D actor; its UI components need an explicit conversion.",
                    entity.name
                ),
                class: Some(entity.name.clone()),
            });
        }
        let root = component(
            "root",
            "Transform",
            ("epok::SceneComponent3D", object_model::SCENE_COMPONENT3D_ID),
        )
        .rooted()
        .with("position", json(&entity.position))
        .with("rotation", json(&entity.rotation))
        .with("scale", json(&entity.scale));
        actor.components.push(root);
        match entity.kind.as_str() {
            "Camera" => actor.components.push(
                component(
                    "camera",
                    "Camera",
                    (
                        "epok::Camera3DComponent",
                        object_model::CAMERA3D_COMPONENT_ID,
                    ),
                )
                .with("camera_fov", json(&entity.camera_fov)),
            ),
            "Mesh" => actor.components.push(
                component(
                    "mesh",
                    "Mesh",
                    ("epok::Mesh3DComponent", object_model::MESH3D_COMPONENT_ID),
                )
                .with("material", json(&entity.material))
                .with("lighting", json(&entity.lighting)),
            ),
            _ => {}
        }
        if let Some(sprite) = &entity.sprite {
            // depth_bias and the ordering-relevant fields are carried verbatim;
            // the sprite's own serialization is the contract.
            let mut sprite_component = component(
                "sprite",
                "Sprite",
                (
                    "epok::Sprite3DComponent",
                    object_model::SPRITE3D_COMPONENT_ID,
                ),
            )
            .with("sprite", json(sprite))
            .with("depth_bias", json(&sprite.depth_bias))
            .with("orientation", json(&sprite.orientation));
            if let Some(animator) = &entity.sprite_animator {
                sprite_component = sprite_component.with("animator", json(animator));
            }
            actor.components.push(sprite_component);
        }
        if let Some(light) = &entity.light {
            actor.components.push(
                component(
                    "light",
                    "Light",
                    ("epok::Light3DComponent", object_model::LIGHT3D_COMPONENT_ID),
                )
                .with("light", json(light)),
            );
        }
        if let Some(collider) = &entity.collider {
            actor.components.push(
                component(
                    "collider",
                    "Collider",
                    (
                        "epok::Collider3DComponent",
                        object_model::COLLIDER3D_COMPONENT_ID,
                    ),
                )
                .with("collider", json(collider)),
            );
        }
    }
    if let Some(audio) = &entity.audio {
        actor.components.push(
            ComponentInstance::new(
                derived_id(entity.id, "audio"),
                ClassReference::new("epok::AudioComponent", object_model::AUDIO_COMPONENT_ID),
                "Audio",
            )
            .with("audio", json(audio)),
        );
    }
    if let Some(script) = &entity.script {
        let mut behaviour = ComponentInstance::new(
            derived_id(entity.id, "behaviour"),
            ClassReference::new(
                "epok::LegacyBehaviourComponent",
                object_model::LEGACY_BEHAVIOUR_COMPONENT_ID,
            ),
            &script.name,
        );
        behaviour.properties = script.properties.clone();
        behaviour.overrides = script.overrides.clone();
        behaviour
            .properties
            .insert("behaviour".into(), Value::String(script.name.clone()));
        behaviour.overrides.insert("behaviour".into());
        if let Some(class_id) = &script.class_id {
            behaviour
                .properties
                .insert("behaviour_class".into(), Value::String(class_id.clone()));
            behaviour.overrides.insert("behaviour_class".into());
        }
        actor.components.push(behaviour);
    }
    actor
}

/// The class reference of a derived actor. A Blueprint instance keeps its class
/// identity so the compiler can find it again; everything else is one of the
/// two minimal placeable bases.
fn actor_class(
    entity: &crate::scene::Entity,
    domain: Domain,
    model: Option<&Model>,
    diagnostics: &mut Vec<Diagnostic>,
) -> ClassReference {
    let base = if domain == Domain::UI {
        ClassReference::new("epok::UIActor", object_model::UI_ACTOR_ID)
    } else {
        ClassReference::new("epok::Actor3D", object_model::ACTOR3D_ID)
    };
    let Some(instance) = &entity.blueprint_instance else {
        return base;
    };
    match model.and_then(|model| model.class(&instance.class)) {
        Some(class) => ClassReference::new(&class.cpp_name, &class.id),
        None => {
            diagnostics.push(Diagnostic {
                code: "unknown-blueprint-class",
                message: format!(
                    "`{}` is an instance of Blueprint class {}, which is not in the current registry; the derived actor falls back to `{}`",
                    entity.name, instance.class, base.name
                ),
                class: Some(base.name.clone()),
            });
            // The identity is retained even when the class is missing, so the
            // redirect table can repair it later without losing the link.
            ClassReference {
                name: base.name.clone(),
                class_id: Some(instance.class.clone()),
            }
        }
    }
}

fn json<T: Serialize>(value: &T) -> Value {
    serde_json::to_value(value).unwrap_or(Value::Null)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reflection_schema::{
        self as schema, Cardinality, ComponentContract, Extension, Location, Placement,
    };
    use crate::scene::Entity;
    use serde_json::json as j;
    use std::fs;

    // ---- model fixtures -------------------------------------------------
    // Hand-built declarations, like the tests in `object_model.rs`: libclang
    // extraction needs the MIPS include paths and cannot run on a host.
    fn class(id: &str, name: &str, parent: Option<&str>) -> schema::Class {
        schema::Class {
            id: id.into(),
            provider: schema::native_provider(),
            backend: schema::native_backend(),
            cpp_name: name.into(),
            parent: parent.map(str::to_owned),
            abstract_class: false,
            final_class: false,
            blueprintable: true,
            timeline_component: None,
            family: None,
            domain: None,
            placement: Placement::default(),
            component: None,
            default_components: vec![],
            explicit_abstract: false,
            properties: vec![],
            functions: vec![],
            source: Location {
                file: "object_model.hpp".into(),
                line: 1,
                column: 1,
            },
        }
    }
    fn actor_class_decl(id: &str, name: &str, domain: Domain, placeable: bool) -> schema::Class {
        let mut c = class(id, name, Some(object_model::ACTOR_ID));
        c.domain = Some(domain);
        c.placement.placeable = placeable;
        c.placement.spawnable = placeable;
        c.placement.scene_managed = !placeable;
        c
    }
    fn component_decl(
        id: &str,
        name: &str,
        parent: &str,
        domain: Domain,
        owners: &[Domain],
        can_root: bool,
    ) -> schema::Class {
        let mut c = class(id, name, Some(parent));
        c.domain = Some(domain);
        c.component = Some(ComponentContract {
            owners: owners.iter().copied().collect(),
            can_root,
            ..Default::default()
        });
        c
    }
    /// The native bases plus the P3/P8 adapters the migration names.
    fn model() -> Model {
        let all = [Domain::World3D, Domain::World2D, Domain::UI];
        let mut classes = vec![];
        let mut object = class(object_model::OBJECT_ID, "epok::Object", None);
        object.family = Some(ClassFamily::Object);
        object.explicit_abstract = true;
        object.blueprintable = false;
        classes.push(object);
        let mut actor = class(
            object_model::ACTOR_ID,
            "epok::Actor",
            Some(object_model::OBJECT_ID),
        );
        actor.family = Some(ClassFamily::Actor);
        actor.domain = Some(Domain::None);
        actor.explicit_abstract = true;
        classes.push(actor);
        classes.push(actor_class_decl(
            object_model::ACTOR3D_ID,
            "epok::Actor3D",
            Domain::World3D,
            true,
        ));
        classes.push(actor_class_decl(
            object_model::UI_ACTOR_ID,
            "epok::UIActor",
            Domain::UI,
            true,
        ));
        let mut scene_script = actor_class_decl(
            object_model::SCENE_SCRIPT_ACTOR_ID,
            "epok::SceneScriptActor",
            Domain::None,
            false,
        );
        scene_script.domain = Some(Domain::None);
        classes.push(scene_script);
        let mut component = class(
            object_model::ACTOR_COMPONENT_ID,
            "epok::ActorComponent",
            Some(object_model::OBJECT_ID),
        );
        component.family = Some(ClassFamily::Component);
        component.domain = Some(Domain::None);
        component.explicit_abstract = true;
        classes.push(component);
        classes.push(component_decl(
            object_model::SCENE_COMPONENT3D_ID,
            "epok::SceneComponent3D",
            object_model::ACTOR_COMPONENT_ID,
            Domain::World3D,
            &[Domain::World3D],
            true,
        ));
        let mut ui = component_decl(
            object_model::UI_COMPONENT_ID,
            "epok::UIComponent",
            object_model::ACTOR_COMPONENT_ID,
            Domain::UI,
            &[Domain::UI],
            false,
        );
        ui.explicit_abstract = true;
        classes.push(ui);
        classes.push(component_decl(
            object_model::RECT_TRANSFORM_COMPONENT_ID,
            "epok::RectTransformComponent",
            object_model::UI_COMPONENT_ID,
            Domain::UI,
            &[Domain::UI],
            true,
        ));
        for (id, name) in [
            (object_model::MESH3D_COMPONENT_ID, "epok::Mesh3DComponent"),
            (
                object_model::CAMERA3D_COMPONENT_ID,
                "epok::Camera3DComponent",
            ),
            (object_model::LIGHT3D_COMPONENT_ID, "epok::Light3DComponent"),
            (
                object_model::COLLIDER3D_COMPONENT_ID,
                "epok::Collider3DComponent",
            ),
            (
                object_model::SPRITE3D_COMPONENT_ID,
                "epok::Sprite3DComponent",
            ),
        ] {
            classes.push(component_decl(
                id,
                name,
                object_model::SCENE_COMPONENT3D_ID,
                Domain::World3D,
                &[Domain::World3D],
                false,
            ));
        }
        for (id, name) in [
            (object_model::CANVAS_COMPONENT_ID, "epok::CanvasComponent"),
            (object_model::IMAGE_COMPONENT_ID, "epok::ImageComponent"),
            (object_model::TEXT_COMPONENT_ID, "epok::TextComponent"),
            (
                object_model::PROGRESS_BAR_COMPONENT_ID,
                "epok::ProgressBarComponent",
            ),
        ] {
            classes.push(component_decl(
                id,
                name,
                object_model::UI_COMPONENT_ID,
                Domain::UI,
                &[Domain::UI],
                false,
            ));
        }
        let mut audio = class(
            object_model::AUDIO_COMPONENT_ID,
            "epok::AudioComponent",
            Some(object_model::ACTOR_COMPONENT_ID),
        );
        audio.domain = Some(Domain::None);
        audio.component = Some(ComponentContract {
            owners: all.into_iter().collect(),
            cardinality: Cardinality::Multiple,
            capabilities: ["audio".to_owned()].into_iter().collect(),
            ..Default::default()
        });
        classes.push(audio);
        let mut legacy = class(
            object_model::LEGACY_BEHAVIOUR_COMPONENT_ID,
            "epok::LegacyBehaviourComponent",
            Some(object_model::ACTOR_COMPONENT_ID),
        );
        legacy.domain = Some(Domain::None);
        legacy.blueprintable = false;
        legacy.component = Some(ComponentContract {
            owners: all.into_iter().collect(),
            cardinality: Cardinality::Multiple,
            ..Default::default()
        });
        classes.push(legacy);
        let mut registry = crate::blueprint::Registry::new();
        for c in classes {
            registry.classes.insert(c.id.clone(), c);
        }
        Model::from_registry(&registry).expect("fixture model resolves")
    }
    fn _unused(_: Extension) {}

    // ---- document fixtures ----------------------------------------------
    fn actor3d(name: &str) -> ActorInstance {
        let id = Uuid::new_v4();
        let mut actor = ActorInstance::new(
            id,
            ClassReference::new("epok::Actor3D", object_model::ACTOR3D_ID),
            name,
        );
        actor.components.push(
            ComponentInstance::new(
                Uuid::new_v4(),
                ClassReference::new("epok::SceneComponent3D", object_model::SCENE_COMPONENT3D_ID),
                "Transform",
            )
            .rooted(),
        );
        actor
    }
    fn ui_actor(name: &str) -> ActorInstance {
        let mut actor = ActorInstance::new(
            Uuid::new_v4(),
            ClassReference::new("epok::UIActor", object_model::UI_ACTOR_ID),
            name,
        );
        actor.components.push(
            ComponentInstance::new(
                Uuid::new_v4(),
                ClassReference::new(
                    "epok::RectTransformComponent",
                    object_model::RECT_TRANSFORM_COMPONENT_ID,
                ),
                "RectTransform",
            )
            .rooted(),
        );
        actor
    }
    fn audio_component() -> ComponentInstance {
        ComponentInstance::new(
            Uuid::new_v4(),
            ClassReference::new("epok::AudioComponent", object_model::AUDIO_COMPONENT_ID),
            "Audio",
        )
    }
    fn scene_script() -> SceneScript {
        SceneScript {
            parent: ClassReference::new(
                "epok::SceneScriptActor",
                object_model::SCENE_SCRIPT_ACTOR_ID,
            ),
            blueprint: crate::blueprint_asset::BlueprintAsset::new(
                "MapScript".into(),
                object_model::SCENE_SCRIPT_ACTOR_ID.into(),
            ),
        }
    }

    // ---- round trip ------------------------------------------------------

    #[test]
    fn version_five_scene_round_trips_actors_components_attachments_and_scene_script() {
        let mut scene = Scene::default();
        let mut first = actor3d("Hero");
        first.properties.insert("speed".into(), j!(2.5));
        first.overrides.insert("speed".into());
        let mut second = actor3d("Sidekick");
        second.properties.insert("speed".into(), j!(1.0));
        second.overrides.insert("speed".into());
        second.logical_parent = Some(first.id);
        second.attach = Some(Attachment {
            actor: first.id,
            component: Some(first.components[0].id),
        });
        second.components.push(audio_component());
        let mut hud = ui_actor("Health");
        hud.logical_parent = Some(first.id);
        hud.components.push(audio_component());
        scene.actors = vec![first, second, hud];
        scene.scene_script = Some(scene_script());
        scene.version = 5;
        scene.validate().unwrap();
        scene.validate_with_model(Some(&model())).unwrap();

        let bytes = crate::document::to_vec(&scene).unwrap();
        let decoded: Scene = crate::document::from_slice(&bytes).unwrap();
        assert_eq!(decoded, scene);
        assert_eq!(
            decoded.actors[1].attach.as_ref().unwrap().actor,
            scene.actors[0].id
        );
        assert_eq!(decoded.actors[2].components.len(), 2);
        assert_eq!(
            decoded.scene_script.as_ref().unwrap().blueprint.name,
            "MapScript"
        );
    }

    #[test]
    fn version_six_is_rejected_as_written_by_a_newer_editor() {
        let scene = Scene {
            version: 6,
            ..Scene::default()
        };
        let error = scene.validate().unwrap_err();
        assert!(error.contains("newer editor"), "{error}");
        // And never silently read as an empty document.
        assert_eq!(scene.entities.len(), 4);
    }

    #[test]
    fn saving_actor_content_raises_the_version_and_classic_scenes_stay_at_three() {
        let mut scene = Scene::default();
        scene.upgrade_entity_ids();
        assert_eq!(scene.version, 3);
        scene.actors.push(actor3d("Hero"));
        scene.upgrade_entity_ids();
        assert_eq!(scene.version, 5);
        scene.actors.clear();
        scene.version = 3;
        scene.scene_script = Some(scene_script());
        scene.upgrade_entity_ids();
        assert_eq!(scene.version, 5);
    }

    #[test]
    fn two_save_cycles_are_idempotent_for_legacy_and_version_five_documents() {
        let root = crate::workspace::tests::temp("actor-document-idempotent");
        fs::create_dir_all(&root).unwrap();

        // Legacy fixture: reading never rewrites the bytes, and two save cycles
        // converge on the first saved bytes.
        let legacy_path = root.join("Legacy.epokmap");
        let mut legacy = serde_json::to_value(Scene::default()).unwrap();
        legacy["version"] = j!(2);
        for entity in legacy["entities"].as_array_mut().unwrap() {
            entity.as_object_mut().unwrap().remove("id");
        }
        let original = serde_json::to_vec(&legacy).unwrap();
        fs::write(&legacy_path, &original).unwrap();
        let loaded = Scene::load(&legacy_path).unwrap();
        assert_eq!(
            fs::read(&legacy_path).unwrap(),
            original,
            "load must not rewrite the document"
        );
        loaded.save(&legacy_path).unwrap();
        let first = fs::read(&legacy_path).unwrap();
        let reloaded = Scene::load(&legacy_path).unwrap();
        assert_eq!(reloaded, loaded);
        reloaded.save(&legacy_path).unwrap();
        assert_eq!(fs::read(&legacy_path).unwrap(), first);

        // Version 5 with actors, components and a scene script.
        let path = root.join("Actors.epokmap");
        let mut scene = Scene::default();
        let mut hero = actor3d("Hero");
        hero.components.push(audio_component());
        let mut hud = ui_actor("Health");
        hud.logical_parent = Some(hero.id);
        scene.actors = vec![hero, hud];
        scene.scene_script = Some(scene_script());
        scene.save(&path).unwrap();
        let first = fs::read(&path).unwrap();
        let loaded = Scene::load(&path).unwrap();
        assert_eq!(loaded.version, 5);
        assert_eq!(loaded.actors.len(), 2);
        loaded.save(&path).unwrap();
        assert_eq!(fs::read(&path).unwrap(), first);
        assert_eq!(Scene::load(&path).unwrap(), loaded);
        fs::remove_dir_all(root).unwrap();
    }

    // ---- validation ------------------------------------------------------

    #[test]
    fn duplicate_uuids_across_entities_actors_and_components_are_rejected() {
        let mut scene = Scene::default();
        let mut a = actor3d("A");
        let b = actor3d("B");
        scene.actors = vec![a.clone(), b.clone()];
        scene.validate().unwrap();
        scene.actors[1].id = scene.actors[0].id;
        assert!(scene.validate().unwrap_err().contains("already used"));
        // A component may not reuse an entity identity either.
        a.components[0].id = scene.entities[0].id;
        scene.actors = vec![a, b];
        assert!(scene.validate().unwrap_err().contains("already used"));
    }

    #[test]
    fn unknown_class_and_invalid_class_names_are_rejected() {
        let mut scene = Scene::default();
        let mut actor = actor3d("Hero");
        actor.class = ClassReference {
            name: "game::Missing".into(),
            class_id: None,
        };
        scene.actors = vec![actor];
        // Structural validation accepts it: the name is a valid identifier.
        scene.validate().unwrap();
        let error = scene.validate_with_model(Some(&model())).unwrap_err();
        assert!(error.contains("not a reflected class"), "{error}");
        scene.actors[0].class.name = "1nvalid name".into();
        assert!(scene.validate().unwrap_err().contains("not a valid class"));
    }

    #[test]
    fn a_ui_component_on_a_three_d_actor_is_rejected_by_the_model() {
        let mut scene = Scene::default();
        let mut actor = actor3d("Hero");
        actor.components.push(ComponentInstance::new(
            Uuid::new_v4(),
            ClassReference::new("epok::ImageComponent", object_model::IMAGE_COMPONENT_ID),
            "Image",
        ));
        scene.actors = vec![actor];
        scene.validate().unwrap();
        let error = scene.validate_with_model(Some(&model())).unwrap_err();
        assert!(error.contains("may only be owned by UI actors"), "{error}");
    }

    #[test]
    fn a_scene_script_actor_may_not_be_placed_and_must_parent_the_scene_blueprint() {
        let mut scene = Scene::default();
        let mut actor = actor3d("Script");
        actor.class = ClassReference::new(
            "epok::SceneScriptActor",
            object_model::SCENE_SCRIPT_ACTOR_ID,
        );
        actor.components.clear();
        scene.actors = vec![actor];
        let model = model();
        let error = scene.validate_with_model(Some(&model)).unwrap_err();
        assert!(error.contains("never placed as an actor"), "{error}");

        scene.actors.clear();
        let mut script = scene_script();
        script.parent = ClassReference::new("epok::Actor3D", object_model::ACTOR3D_ID);
        scene.scene_script = Some(script);
        let error = scene.validate_with_model(Some(&model)).unwrap_err();
        assert!(error.contains("must derive from"), "{error}");
        scene.scene_script = Some(scene_script());
        scene.validate_with_model(Some(&model)).unwrap();
    }

    #[test]
    fn cyclic_logical_parents_and_dangling_references_are_rejected() {
        let mut scene = Scene::default();
        let mut a = actor3d("A");
        let mut b = actor3d("B");
        a.logical_parent = Some(b.id);
        b.logical_parent = Some(a.id);
        scene.actors = vec![a.clone(), b];
        assert!(scene.validate().unwrap_err().contains("own ancestor"));

        a.logical_parent = Some(Uuid::new_v4());
        scene.actors = vec![a.clone()];
        assert!(scene.validate().unwrap_err().contains("logical parent"));

        a.logical_parent = None;
        a.attach = Some(Attachment {
            actor: Uuid::new_v4(),
            component: None,
        });
        scene.actors = vec![a.clone()];
        assert!(scene.validate().unwrap_err().contains("not in this scene"));

        a.attach = None;
        a.legacy_entity = Some(Uuid::new_v4());
        scene.actors = vec![a];
        assert!(scene.validate().unwrap_err().contains("no longer exists"));
    }

    #[test]
    fn attachment_across_domains_is_rejected_only_when_a_model_is_available() {
        let mut scene = Scene::default();
        let hero = actor3d("Hero");
        let mut hud = ui_actor("Health");
        hud.attach = Some(Attachment {
            actor: hero.id,
            component: Some(hero.components[0].id),
        });
        scene.actors = vec![hero, hud];
        scene.validate().unwrap();
        let error = scene.validate_with_model(Some(&model())).unwrap_err();
        assert!(error.contains("cannot attach"), "{error}");
    }

    // ---- duplication -----------------------------------------------------

    #[test]
    fn duplicating_a_scene_remaps_actor_and_component_identities_but_not_assets() {
        let mut scene = Scene::default();
        let asset = Uuid::new_v4();
        let mut hero = actor3d("Hero");
        hero.legacy_entity = Some(scene.entities[1].id);
        hero.properties.insert("mesh".into(), j!(asset.to_string()));
        hero.overrides.insert("mesh".into());
        let mut hud = ui_actor("Health");
        hud.logical_parent = Some(hero.id);
        hud.attach = Some(Attachment {
            actor: hero.id,
            component: Some(hero.components[0].id),
        });
        let old_hero = hero.id;
        let old_root = hero.components[0].id;
        scene.actors = vec![hero, hud];
        let mut script = scene_script();
        script
            .blueprint
            .defaults
            .insert("target".into(), j!(old_hero.to_string()));
        script
            .blueprint
            .defaults
            .insert("texture".into(), j!(asset.to_string()));
        let old_blueprint = script.blueprint.id.clone();
        scene.scene_script = Some(script);

        let mut copy = scene.clone();
        copy.duplicate_identities();
        copy.validate().unwrap();
        assert_ne!(copy.actors[0].id, old_hero);
        assert_ne!(copy.actors[0].components[0].id, old_root);
        assert_eq!(copy.actors[1].logical_parent, Some(copy.actors[0].id));
        assert_eq!(
            copy.actors[1].attach.as_ref().unwrap().component,
            Some(copy.actors[0].components[0].id)
        );
        assert_eq!(copy.actors[0].properties["mesh"], j!(asset.to_string()));
        assert_eq!(copy.actors[0].legacy_entity, Some(copy.entities[1].id));
        assert_ne!(copy.entities[1].id, scene.entities[1].id);
        let blueprint = &copy.scene_script.as_ref().unwrap().blueprint;
        assert_ne!(blueprint.id, old_blueprint);
        assert_eq!(
            blueprint.defaults["target"],
            j!(copy.actors[0].id.to_string())
        );
        assert_eq!(blueprint.defaults["texture"], j!(asset.to_string()));
    }

    #[test]
    fn duplicate_and_delete_branch_carry_and_drop_the_actors_of_the_branch() {
        let mut scene = Scene::default();
        let mut hero = actor3d("Hero");
        hero.legacy_entity = Some(scene.entities[1].id);
        let mut child = actor3d("Child");
        child.legacy_entity = Some(scene.entities[2].id);
        child.logical_parent = Some(hero.id);
        child.attach = Some(Attachment {
            actor: hero.id,
            component: Some(hero.components[0].id),
        });
        let outside = actor3d("Outside");
        let outside_id = outside.id;
        scene.entities[2].parent = Some(1);
        scene.actors = vec![hero, child, outside];
        scene.validate().unwrap();

        let copy = scene.duplicate_branch(1).unwrap();
        scene.validate().unwrap();
        assert_eq!(scene.actors.len(), 5);
        let copied_hero = scene
            .actors
            .iter()
            .find(|a| a.legacy_entity == Some(scene.entities[copy].id))
            .expect("the copied root carries the copied entity");
        let copied_child = scene
            .actors
            .iter()
            .find(|a| a.legacy_entity == Some(scene.entities[copy + 1].id))
            .expect("the copied child");
        assert_eq!(copied_child.logical_parent, Some(copied_hero.id));
        assert_ne!(copied_hero.id, scene.actors[0].id);

        scene.delete_branch(1);
        scene.validate().unwrap();
        // The originals are gone; the copies and the unrelated actor survive.
        assert!(scene.actors.iter().any(|a| a.id == outside_id));
        assert_eq!(scene.actors.len(), 3);
    }

    // ---- legacy view -----------------------------------------------------

    fn legacy_scene() -> Scene {
        let mut scene = Scene {
            version: 4,
            entities: vec![],
            ..Scene::default()
        };
        let mut camera = Entity::cube("Main Camera".into());
        camera.kind = "Camera".into();
        let mesh = Entity::cube("Cube".into());
        let mut empty = Entity::cube("Pivot".into());
        empty.kind = "Empty".into();
        empty.parent = Some(1);
        let mut sprite = Entity::cube("Coin".into());
        sprite.kind = "Empty".into();
        sprite.sprite = Some(crate::sprites::Sprite {
            depth_bias: -7,
            ..Default::default()
        });
        let mut canvas = Entity::cube("Canvas".into());
        canvas.kind = "Empty".into();
        canvas.canvas = Some(crate::hud::Canvas::default());
        canvas.rect = Some(crate::hud::RectTransform::default());
        let mut image = Entity::cube("Bar".into());
        image.kind = "Empty".into();
        image.parent = Some(4);
        image.rect = Some(crate::hud::RectTransform::default());
        image.image = Some(crate::hud::Image::default());
        let mut hybrid = Entity::cube("Hybrid".into());
        hybrid.rect = Some(crate::hud::RectTransform::default());
        let mut scripted = Entity::cube("Scripted".into());
        scripted.kind = "Empty".into();
        scripted.script = Some(crate::scene::ScriptBinding {
            name: "Spinner".into(),
            class_id: Some("11111111-2222-3333-4444-555555555555".into()),
            properties: [("speed".to_owned(), j!(90.0))].into_iter().collect(),
            overrides: ["speed".to_owned()].into_iter().collect(),
            ..Default::default()
        });
        scripted.audio = Some(Default::default());
        scene.entities = vec![camera, mesh, empty, sprite, canvas, image, hybrid, scripted];
        for (index, entity) in scene.entities.iter_mut().enumerate() {
            entity.id = Uuid::from_u128(0xA000 + index as u128);
        }
        scene
    }

    #[test]
    fn legacy_entities_map_to_actors_deterministically_without_touching_the_scene() {
        let scene = legacy_scene();
        let before = scene.clone();
        let view = actor_view(&scene, None);
        assert_eq!(scene, before, "actor_view never writes to the scene");
        assert_eq!(actor_view(&scene, None).actors, view.actors);
        assert_eq!(view.actors.len(), scene.entities.len());

        let classes = |name: &str| -> Vec<String> {
            view.actors
                .iter()
                .find(|a| a.name == name)
                .unwrap()
                .components
                .iter()
                .map(|c| c.class.name.clone())
                .collect()
        };
        let actor = |name: &str| view.actors.iter().find(|a| a.name == name).unwrap();

        assert_eq!(actor("Main Camera").class.name, "epok::Actor3D");
        assert_eq!(
            classes("Main Camera"),
            ["epok::SceneComponent3D", "epok::Camera3DComponent"]
        );
        assert_eq!(
            classes("Cube"),
            ["epok::SceneComponent3D", "epok::Mesh3DComponent"]
        );
        assert_eq!(classes("Pivot"), ["epok::SceneComponent3D"]);
        assert_eq!(
            classes("Coin"),
            ["epok::SceneComponent3D", "epok::Sprite3DComponent"]
        );
        let coin = actor("Coin");
        assert_eq!(coin.components[1].properties["depth_bias"], j!(-7));

        assert_eq!(actor("Canvas").class.name, "epok::UIActor");
        assert_eq!(
            classes("Canvas"),
            ["epok::RectTransformComponent", "epok::CanvasComponent"]
        );
        assert_eq!(
            classes("Bar"),
            ["epok::RectTransformComponent", "epok::ImageComponent"]
        );
        // UI parenting stays inside the UI domain.
        let bar = actor("Bar");
        assert_eq!(bar.logical_parent, Some(actor("Canvas").id));
        assert_eq!(bar.attach.as_ref().unwrap().actor, actor("Canvas").id);

        // Hybrid: 3D identity kept, conversion reported.
        assert_eq!(actor("Hybrid").class.name, "epok::Actor3D");
        assert!(view.codes().contains(&"hybrid-entity-pending-conversion"));

        // Script + audio.
        assert_eq!(
            classes("Scripted"),
            [
                "epok::SceneComponent3D",
                "epok::AudioComponent",
                "epok::LegacyBehaviourComponent"
            ]
        );
        let behaviour = actor("Scripted").components.last().unwrap();
        assert_eq!(behaviour.name, "Spinner");
        assert_eq!(behaviour.properties["speed"], j!(90.0));
        assert!(behaviour.overrides.contains("speed"));
        assert_eq!(
            behaviour.properties["behaviour_class"],
            j!("11111111-2222-3333-4444-555555555555")
        );

        // A 3D child of a 3D parent attaches spatially; a 3D actor never
        // attaches to a UI one.
        let pivot = actor("Pivot");
        assert_eq!(pivot.logical_parent, Some(actor("Cube").id));
        assert!(pivot.attach.is_some());

        // Provenance points back at the entity it came from.
        for (entity, actor) in scene.entities.iter().zip(&view.actors) {
            assert_eq!(actor.legacy_entity, Some(entity.id));
            assert_eq!(actor.id, derived_id(entity.id, ""));
        }
    }

    #[test]
    fn the_derived_view_satisfies_the_model_and_reuses_authored_actors() {
        let mut scene = legacy_scene();
        let model = model();
        let view = actor_view(&scene, Some(&model));
        assert_eq!(
            view.codes()
                .into_iter()
                .filter(|c| *c != "hybrid-entity-pending-conversion")
                .collect::<Vec<_>>(),
            Vec::<&str>::new(),
            "{:?}",
            view.diagnostics
        );
        // An entity that already has an authored actor is not derived twice.
        let mut authored = actor3d("Authored Cube");
        authored.legacy_entity = Some(scene.entities[1].id);
        scene.actors = vec![authored];
        let view = actor_view(&scene, Some(&model));
        assert_eq!(view.actors.len(), scene.entities.len());
        assert_eq!(
            view.actors
                .iter()
                .filter(|a| a.legacy_entity == Some(scene.entities[1].id))
                .count(),
            1
        );
    }
}
