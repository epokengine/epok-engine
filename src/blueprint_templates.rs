//! Inherited entity templates and a deterministic, bounded host construction backend.
//!
//! Construction edits authored component data only. It never executes native
//! script bodies, starts a game, loads a MIPS module, or evaluates arbitrary JSON.
use crate::{
    blueprint::Registry,
    reflection_schema::Type,
    scene::{Actor, ClassDefaults, Scene},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

pub const MAX_ENTITIES: usize = 32;
pub const MAX_CONSTRUCTION_OPS: usize = 256;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Instance {
    pub class: String,
    pub template_entity: Uuid,
    /// Asset component UUID to placed component UUID, stable across refreshes.
    #[serde(default)]
    pub component_ids: BTreeMap<Uuid, Uuid>,
    /// Persistent UUID of the placed root entity, shared by the entire instance.
    pub instance: Uuid,
    #[serde(default)]
    pub overrides: BTreeMap<String, Value>,
    /// Explicit authored-UUID reparenting; absent means inherited template parent.
    #[serde(default)]
    pub parent: Option<ParentOverride>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Template {
    #[serde(default)]
    pub actors: Vec<TemplateEntity>,
    #[serde(default)]
    pub overrides: BTreeMap<Uuid, EntityOverride>,
    #[serde(default)]
    pub references: Vec<ObjectReference>,
    #[serde(default)]
    pub construction: Vec<ConstructionOp>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TemplateEntity {
    /// entity.id is the persistent template entity ID, never a runtime handle.
    pub entity: Actor,
    pub parent: Option<Uuid>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EntityOverride {
    /// Keys are stable versioned built-in member IDs, not UI labels/array offsets.
    #[serde(default)]
    pub members: BTreeMap<String, Value>,
    #[serde(default)]
    pub parent: Option<ParentOverride>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ParentOverride {
    Root,
    Actor { entity: Uuid },
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ObjectReference {
    pub owner: Uuid,
    /// Stable reflected property ID; renames do not silently retarget a reference.
    pub member: String,
    pub target: Uuid,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum ConstructionOp {
    Translate { entity: Uuid, offset: [f32; 3] },
    SetPosition { entity: Uuid, value: [f32; 3] },
    SetRotation { entity: Uuid, value: [f32; 3] },
    SetScale { entity: Uuid, value: [f32; 3] },
    SetActive { entity: Uuid, active: bool },
    SetColor { entity: Uuid, color: [f32; 3] },
    Reparent { entity: Uuid, parent: Option<Uuid> },
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ResolvedTemplate {
    pub actors: Vec<TemplateEntity>,
    pub references: Vec<ObjectReference>,
}
#[derive(Clone, Debug)]
pub struct Placement {
    pub root: usize,
    pub actors: Vec<usize>,
    pub identities: BTreeMap<Uuid, Uuid>,
}

impl Template {
    pub fn root(name: &str) -> Self {
        let mut entity = Actor::cube(name.into());
        entity.kind = "Empty".into();
        entity.position = [0.; 3];
        crate::actor_components::sync(&mut entity);
        Self {
            actors: vec![TemplateEntity {
                entity,
                parent: None,
            }],
            ..Self::default()
        }
    }
    pub fn add_child(&mut self, parent: Uuid, name: &str) -> Uuid {
        let mut entity = Actor::cube(name.into());
        entity.kind = "Empty".into();
        entity.position = [0.; 3];
        crate::actor_components::sync(&mut entity);
        entity.attach = Some(crate::actor_document::Attachment {
            actor: parent,
            component: None,
        });
        let id = entity.id;
        self.actors.push(TemplateEntity {
            entity,
            parent: Some(parent),
        });
        id
    }
    pub fn remove_local(&mut self, id: Uuid) -> Result<(), String> {
        if !self.actors.iter().any(|item| item.entity.id == id) {
            return Err("Inherited template actors cannot be deleted; deactivate explicitly or edit the declaring Blueprint.".into());
        }
        if self.actors.iter().any(|item| item.parent == Some(id))
            || self
                .references
                .iter()
                .any(|r| r.target == id || r.owner == id)
        {
            return Err(
                "Template entity is referenced or has children; resolve those dependencies first."
                    .into(),
            );
        }
        self.actors.retain(|item| item.entity.id != id);
        self.overrides.remove(&id);
        Ok(())
    }
}

fn member_field(id: &str) -> Option<&'static str> {
    Some(match id {
        "epok.actor.name.v1" => "name",
        "epok.actor.active.v1" => "active",
        "epok.actor.components.v1" => "components",
        "epok.actor.properties.v1" => "properties",
        "epok.actor.overrides.v1" => "overrides",
        _ => return None,
    })
}
fn apply_override(item: &mut TemplateEntity, edits: &EntityOverride) -> Result<(), String> {
    apply_instance_edits(&mut item.entity, &edits.members)?;
    if let Some(parent) = &edits.parent {
        item.parent = match parent {
            ParentOverride::Root => None,
            ParentOverride::Actor { entity } => Some(*entity),
        };
    }
    Ok(())
}

/// Project component overrides through the same typed applicator as template
/// resolution. Unknown members and invalid values remain observable, not lost.
fn audio_members_signature(
    members: &BTreeMap<String, Value>,
    registry: Option<&Registry>,
) -> Option<String> {
    let selected=members.iter().filter_map(|(path,value)| {
        let parts=path.split('/').collect::<Vec<_>>();
        let property=parts.iter().position(|p|*p=="properties").and_then(|i|parts.get(i+1)).copied();
        let selected=match property {
            Some("position"|"rotation"|"scale"|"material"|"lighting"|"camera_fov"|"camera_sky_color"|"sprite"|"sprite_animator"|"particle_emitter"|"collider"|"palette_animator"|"skeletal_mesh"|"editable_mesh"|"terrain"|"canvas"|"rect"|"image"|"text"|"progress"|"light"|"blob_shadow")=>return None,
            Some("audio")=>{
                if parts.last()==Some(&"audio") {value.get("clip").cloned().unwrap_or(Value::Null)}
                else if parts.last()==Some(&"clip"){value.clone()}else{return None}
            }
            Some("timeline"|"particle_effect")=>{
                if parts.last().copied()==property {value.get("asset").cloned().unwrap_or(Value::Null)}
                else if parts.last()==Some(&"asset"){value.clone()}else{return None}
            }
            Some(key)=>{
                if registry.is_some_and(|r| {
                    let declarations=r.classes.values().flat_map(|c|c.properties.iter()).filter(|p|p.name==key).collect::<Vec<_>>();
                    !declarations.is_empty() && declarations.iter().all(|p|!matches!(&p.value_type,Type::AssetRef{kind} if matches!(kind.as_str(),"AudioClip"|"PlayableAudio"|"MusicSequence")))
                }) {return None;}
                value.clone()
            }
            None if matches!(path.as_str(),"/name"|"/active"|"/attach"|"/overrides")=>return None,
            None if parts.len()==3 && parts[1]=="components"=>{
                let Some(class)=value.get("class").and_then(|c|c.get("class_id")).and_then(Value::as_str) else{return Some((path,value.clone()))};
                if class==crate::object_model::AUDIO_COMPONENT_ID {value.pointer("/properties/audio/clip").cloned().unwrap_or(Value::Null)}
                else if matches!(class,crate::actor_components::TIMELINE|crate::actor_components::EFFECT){
                    value.pointer("/properties/timeline/asset").or_else(||value.pointer("/properties/particle_effect/asset")).cloned().unwrap_or(Value::Null)
                }else if crate::actor_components::native(class){return None}else{value.clone()}
            }
            _=>value.clone(),
        };
        Some((path,selected))
    }).collect::<BTreeMap<_,_>>();
    (!selected.is_empty()).then(|| crate::scene_dependencies::hash(selected))
}

pub fn instance_audio_signature(instance: &Instance, registry: Option<&Registry>) -> String {
    crate::scene_dependencies::hash((
        &instance.class,
        instance.template_entity,
        instance.instance,
        audio_members_signature(&instance.overrides, registry),
    ))
}

pub fn template_audio_signature(template: &Template, registry: Option<&Registry>) -> String {
    // Keep resource-bearing entity identities: a derived override may select
    // just one entity. Do not collapse duplicates; validation owns that error.
    let empty = crate::audio::selection_signature(
        &Scene {
            actors: vec![],
            ..Default::default()
        },
        registry,
    );
    let mut actors = template
        .actors
        .iter()
        .map(|item| {
            (
                item.entity.id,
                crate::audio::selection_signature(
                    &Scene {
                        actors: vec![item.entity.clone()],
                        ..Default::default()
                    },
                    registry,
                ),
            )
        })
        .filter(|(_, signature)| *signature != empty)
        .collect::<Vec<_>>();
    actors.sort();
    let overrides = template
        .overrides
        .iter()
        .filter_map(|(id, edits)| {
            audio_members_signature(&edits.members, registry).map(|signature| (id, signature))
        })
        .collect::<BTreeMap<_, _>>();
    // These operations cannot select resources, including on inactive actors.
    // An added operation must explicitly define its resource behavior here.
    for operation in &template.construction {
        match operation {
            ConstructionOp::Translate { .. }
            | ConstructionOp::SetPosition { .. }
            | ConstructionOp::SetRotation { .. }
            | ConstructionOp::SetScale { .. }
            | ConstructionOp::SetActive { .. }
            | ConstructionOp::SetColor { .. }
            | ConstructionOp::Reparent { .. } => {}
        }
    }
    crate::scene_dependencies::hash((actors, overrides))
}

/// Resolve already ordered native-to-derived template layers. Inputs are never
/// modified on error. Derived overrides retain their identity and explicitness.
pub fn resolve(layers: &[&Template]) -> Result<ResolvedTemplate, String> {
    if layers.len() > 64 {
        return Err("Template inheritance exceeds 64 classes.".into());
    }
    let mut actors = BTreeMap::<Uuid, TemplateEntity>::new();
    let mut references = BTreeMap::new();
    let mut operations = vec![];
    for layer in layers {
        for item in &layer.actors {
            if item.entity.id.is_nil()
                || item.entity.parent.is_some()
                || actors.insert(item.entity.id, item.clone()).is_some()
            {
                return Err("Template entity requires a unique non-nil UUID and UUID parent; numeric scene parents are not portable.".into());
            }
        }
        if actors.len() > MAX_ENTITIES {
            return Err(format!("Template exceeds {MAX_ENTITIES} actors."));
        }
        for (id, edits) in &layer.overrides {
            apply_override(
                actors.get_mut(id).ok_or_else(|| {
                    format!(
                        "Orphaned template override {id}; restore its entity or migrate explicitly."
                    )
                })?,
                edits,
            )?;
        }
        let mut local_references = BTreeSet::new();
        for reference in &layer.references {
            if !local_references.insert((reference.owner, reference.member.as_str())) {
                return Err("Duplicate template entity-reference member in one class.".into());
            }
            references.insert(
                (reference.owner, reference.member.clone()),
                reference.clone(),
            );
        }
        operations.extend(layer.construction.iter());
    }
    if operations.len() > MAX_CONSTRUCTION_OPS {
        return Err(format!(
            "Construction exceeds {MAX_CONSTRUCTION_OPS} operations."
        ));
    }
    for operation in operations {
        construct(&mut actors, operation)?;
    }
    let result = ResolvedTemplate {
        actors: actors.into_values().collect(),
        references: references.into_values().collect(),
    };
    result.validate()?;
    Ok(result)
}
/// Resolve one class directly from the indexed asset set. Native classes have no
/// template layer; missing parents and cycles are errors rather than empty data.
pub fn resolve_assets(
    files: &[crate::blueprint_asset::AssetFile],
    registry: &Registry,
    id: &str,
) -> Result<ResolvedTemplate, String> {
    let mut assets = BTreeMap::new();
    for file in files {
        if assets.insert(file.asset.id.as_str(), &file.asset).is_some() {
            return Err("Duplicate Blueprint class identity while resolving template.".into());
        }
    }
    let mut current = id;
    let mut seen = BTreeSet::new();
    let mut layers = vec![];
    loop {
        if !seen.insert(current) {
            return Err(format!("Template inheritance cycle at {current}."));
        }
        if let Some(asset) = assets.get(current) {
            layers.push(&asset.template);
            current = &asset.parent;
        } else if registry.classes.contains_key(current) {
            break;
        } else {
            return Err(format!("Template parent class {current} is missing."));
        }
    }
    layers.reverse();
    resolve(&layers)
}
fn quantize(value: f32) -> Result<i32, String> {
    let raw = (f64::from(value) * 4096.).round();
    if !raw.is_finite() || raw < f64::from(i32::MIN) || raw > f64::from(i32::MAX) {
        return Err("Construction value exceeds the signed Q12 range.".into());
    }
    Ok(raw as i32)
}
fn quantize_vector(value: [f32; 3]) -> Result<[f32; 3], String> {
    let mut result = [0.; 3];
    for i in 0..3 {
        result[i] = quantize(value[i])? as f32 / 4096.;
    }
    Ok(result)
}
fn construct(
    actors: &mut BTreeMap<Uuid, TemplateEntity>,
    operation: &ConstructionOp,
) -> Result<(), String> {
    let id = match operation {
        ConstructionOp::Translate { entity, .. }
        | ConstructionOp::SetPosition { entity, .. }
        | ConstructionOp::SetRotation { entity, .. }
        | ConstructionOp::SetScale { entity, .. }
        | ConstructionOp::SetActive { entity, .. }
        | ConstructionOp::SetColor { entity, .. }
        | ConstructionOp::Reparent { entity, .. } => entity,
    };
    let item = actors
        .get_mut(id)
        .ok_or_else(|| format!("Construction references missing template entity {id}."))?;
    match operation {
        ConstructionOp::Translate { offset, .. } => {
            for (index, value) in offset.iter().enumerate() {
                let sum = i64::from(quantize(item.entity.position[index])?)
                    + i64::from(quantize(*value)?);
                item.entity.position[index] =
                    sum.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as f32 / 4096.;
            }
        }
        ConstructionOp::SetPosition { value, .. } => {
            item.entity.position = quantize_vector(*value)?
        }
        ConstructionOp::SetRotation { value, .. } => {
            item.entity.rotation = quantize_vector(*value)?
        }
        ConstructionOp::SetScale { value, .. } => item.entity.scale = quantize_vector(*value)?,
        ConstructionOp::SetActive { active, .. } => item.entity.active = *active,
        ConstructionOp::SetColor { color, .. } => item.entity.material.color = *color,
        ConstructionOp::Reparent { parent, .. } => item.parent = *parent,
    }
    Ok(())
}
impl ResolvedTemplate {
    pub fn root(&self) -> Option<Uuid> {
        self.actors
            .iter()
            .find(|item| item.parent.is_none())
            .map(|item| item.entity.id)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.actors.is_empty() {
            return if self.references.is_empty() {
                Ok(())
            } else {
                Err("An empty template cannot contain references.".into())
            };
        }
        if self.actors.len() > MAX_ENTITIES
            || self
                .actors
                .iter()
                .filter(|item| item.parent.is_none())
                .count()
                != 1
        {
            return Err("Template requires exactly one root and at most 32 actors.".into());
        }
        let indices = self
            .actors
            .iter()
            .enumerate()
            .map(|(index, item)| (item.entity.id, index))
            .collect::<BTreeMap<_, _>>();
        let mut scene = Scene::default();
        scene.actors.clear();
        for item in &self.actors {
            let mut entity = item.entity.clone();
            entity.logical_parent = item.parent;
            entity.parent = item
                .parent
                .map(|id| {
                    indices
                        .get(&id)
                        .copied()
                        .ok_or_else(|| format!("Missing template parent {id}."))
                })
                .transpose()?;
            scene.actors.push(entity);
        }
        scene.validate()?;
        let identities = crate::actor_document::fresh_identities(&scene.actors);
        for reference in &self.references {
            if !identities.contains_key(&reference.owner)
                || !identities.contains_key(&reference.target)
                || reference.member.is_empty()
            {
                return Err(
                    "Template entity reference has a missing owner, target, or member ID.".into(),
                );
            }
        }
        Ok(())
    }
    /// Construct the exact local scene prototype used both for placement and
    /// cooking. Cookers resolve resources through the usual scene pipeline.
    pub fn scene(&self, binding: ClassDefaults, registry: &Registry) -> Result<Scene, String> {
        self.validate()?;
        let root = self.root().ok_or("Blueprint has no Actor template.")?;
        let class = registry
            .bound(&binding)
            .ok_or("Actor Blueprint class is unresolved.")?;
        let mut scene = Scene::default();
        scene.actors.clear();
        scene.name = "ActorTemplate".into();
        for item in &self.actors {
            let mut actor = item.entity.clone();
            actor.logical_parent = item.parent;
            if actor.id == root {
                let base = actor
                    .class
                    .class_id
                    .as_deref()
                    .or_else(|| registry.named(&actor.class.name).map(|c| c.id.as_str()))
                    .ok_or("Template root class is unresolved")?;
                if !crate::blueprint_refs::class_is_a(registry, &class.id, base) {
                    return Err("Template root must be an ancestor of the Actor Blueprint being instantiated".into());
                }
                actor.class =
                    crate::actor_document::ClassReference::new(&class.cpp_name, &class.id);
                actor.properties.extend(binding.properties.clone());
                actor.overrides.extend(binding.overrides.clone());
            }
            scene.actors.push(actor);
        }
        scene.refresh_actor_hierarchy();
        scene.sync_actor_components();
        for reference in &self.references {
            let owner = scene
                .actors
                .iter_mut()
                .find_map(|a| {
                    if a.id == reference.owner {
                        Some((&a.class, &mut a.properties, &mut a.overrides))
                    } else {
                        a.components
                            .iter_mut()
                            .find(|c| c.id == reference.owner)
                            .map(|c| (&c.class, &mut c.properties, &mut c.overrides))
                    }
                })
                .ok_or("Template reference owner is missing")?;
            let property = registry
                .properties(&owner.0.name)
                .into_iter()
                .find(|p| p.id == reference.member)
                .ok_or("Orphaned template reference member")?;
            if !matches!(
                property.value_type,
                Type::ObjectRef { .. } | Type::ActorRef { .. } | Type::ComponentRef { .. }
            ) {
                return Err(
                    "Template reference member must be an Actor or ActorComponent reference".into(),
                );
            }
            owner
                .1
                .insert(property.name.clone(), serde_json::json!(reference.target));
            owner.2.insert(property.name.clone());
        }
        validate_properties(&scene, registry)?;
        scene.validate()?;
        Ok(scene)
    }
}

pub fn place(
    scene: &mut Scene,
    template: &ResolvedTemplate,
    binding: ClassDefaults,
    registry: &Registry,
    parent: Option<usize>,
) -> Result<Placement, String> {
    if parent.is_some_and(|p| p >= scene.actors.len()) {
        return Err("Placement parent is missing.".into());
    }
    let class = registry
        .bound(&binding)
        .ok_or("Actor class is unresolved.")?
        .id
        .clone();
    let prototype = template.scene(binding, registry)?;
    let template_root = template.root().ok_or("Blueprint has no Actor template.")?;
    let identities = crate::actor_document::fresh_identities(&prototype.actors);
    let mut candidate = scene.clone();
    let base = candidate.actors.len();
    let root = base
        + prototype
            .actors
            .iter()
            .position(|a| a.id == template_root)
            .ok_or("Template root is missing.")?;
    for mut actor in prototype.actors {
        let template_id = actor.id;
        let component_ids = actor
            .components
            .iter()
            .map(|c| (c.id, identities[&c.id]))
            .collect();
        crate::actor_document::remap_actor(&mut actor, &identities);
        if template_id == template_root {
            actor.logical_parent = parent.map(|p| scene.actors[p].id);
        }
        actor.data.blueprint_instance = Some(Instance {
            class: class.clone(),
            template_entity: template_id,
            instance: identities[&template_root],
            component_ids,
            overrides: BTreeMap::new(),
            parent: None,
        });
        actor.name = candidate.unique_actor_name(&actor.name);
        candidate.actors.push(actor);
    }
    candidate.refresh_actor_hierarchy();
    candidate.sync_actor_components();
    candidate.bake = None;
    candidate.validate()?;
    let actors = (base..candidate.actors.len()).collect();
    *scene = candidate;
    Ok(Placement {
        root,
        actors,
        identities,
    })
}

/// Captures component composition and Actor properties with fresh asset-local identities.
pub fn capture(scene: &Scene, root: usize, registry: &Registry) -> Result<Template, String> {
    let mut source = scene.clone();
    source.sync_actor_components();
    source.validate()?;
    let root_id = source.actors.get(root).ok_or("Select an Actor first.")?.id;
    let branch = source.actor_branch(root_id);
    let actors = source
        .actors
        .iter()
        .filter(|a| branch.contains(&a.id))
        .cloned()
        .collect::<Vec<_>>();
    let identities = crate::actor_document::fresh_identities(&actors);
    let mut template = Template::default();
    for mut actor in actors {
        let is_root = actor.id == root_id;
        crate::actor_document::remap_actor(&mut actor, &identities);
        if is_root {
            actor.logical_parent = None;
            actor.attach = None;
        }
        actor.data.parent = None;
        actor.data.blueprint_instance = None;
        let parent = actor.logical_parent;
        template.actors.push(TemplateEntity {
            entity: actor,
            parent,
        });
    }
    resolve(&[&template])?;
    let local = Scene {
        actors: template.actors.iter().map(|i| i.entity.clone()).collect(),
        ..Default::default()
    };
    validate_properties(&local, registry)?;
    for actor in &local.actors {
        for (owner, class, values) in std::iter::once((actor.id, &actor.class, &actor.properties))
            .chain(
                actor
                    .components
                    .iter()
                    .map(|c| (c.id, &c.class, &c.properties)),
            )
        {
            for property in registry.properties(&class.name) {
                if matches!(
                    property.value_type,
                    Type::ObjectRef { .. } | Type::ActorRef { .. } | Type::ComponentRef { .. }
                ) && let Some(target) = values
                    .get(&property.name)
                    .and_then(Value::as_str)
                    .and_then(|s| Uuid::parse_str(s).ok())
                {
                    template.references.push(ObjectReference {
                        owner,
                        member: property.id.clone(),
                        target,
                    });
                }
            }
        }
    }
    Ok(template)
}

fn validate_properties(scene: &Scene, registry: &Registry) -> Result<(), String> {
    let ids = scene
        .actors
        .iter()
        .flat_map(|a| std::iter::once(a.id).chain(a.components.iter().map(|c| c.id)))
        .collect::<BTreeSet<_>>();
    for actor in &scene.actors {
        for target in actor
            .timeline
            .iter()
            .flat_map(|c| c.bindings.values())
            .chain(
                actor
                    .particle_effect
                    .iter()
                    .flat_map(|c| c.bindings.values()),
            )
            .flatten()
        {
            if !ids.contains(target) {
                return Err(format!(
                    "Playback binding {target} points outside the Actor Blueprint subtree"
                ));
            }
        }
        for (class, values) in std::iter::once((&actor.class, &actor.properties))
            .chain(actor.components.iter().map(|c| (&c.class, &c.properties)))
        {
            if class
                .class_id
                .as_deref()
                .is_some_and(crate::actor_components::native)
            {
                continue;
            }
            let reflected = registry.properties(&class.name);
            for (name, value) in values {
                let property=reflected.iter().find(|p|p.name==*name).ok_or_else(||format!("Orphaned property {}.{name}; repair the class before creating an Actor Blueprint",class.name))?;
                if matches!(
                    property.value_type,
                    Type::ObjectRef { .. } | Type::ActorRef { .. } | Type::ComponentRef { .. }
                ) && let Some(target) = value.as_str().and_then(|s| Uuid::parse_str(s).ok())
                    && !ids.contains(&target)
                {
                    return Err(format!(
                        "{}.{} references an object outside the Actor Blueprint subtree",
                        class.name, name
                    ));
                }
                crate::blueprint_refs::assignment(
                    name,
                    value,
                    &property.value_type,
                    scene,
                    registry,
                )?;
            }
        }
    }
    Ok(())
}

/// Call only for a known user edit. Reading or refreshing an instance must never
/// infer override intent from equality with a default.
pub fn record_overrides(original: &Actor, edited: &mut Actor) {
    if original.id != edited.id || edited.blueprint_instance.is_none() {
        return;
    }
    let Ok(edits) = changed_members(original, edited) else {
        return;
    };
    let overrides = &mut edited.blueprint_instance.as_mut().unwrap().overrides;
    for (path, value) in edits {
        overrides.retain(|key, _| !key.starts_with(&format!("{path}/")));
        overrides.insert(path, value);
    }
}
pub(crate) fn changed_members(
    original: &Actor,
    edited: &Actor,
) -> Result<BTreeMap<String, Value>, String> {
    let before = serde_json::to_value(original).map_err(|e| e.to_string())?;
    let after = serde_json::to_value(edited).map_err(|e| e.to_string())?;
    let mut edits = BTreeMap::new();
    for key in [
        "name",
        "active",
        "attach",
        "components",
        "properties",
        "overrides",
    ] {
        diff_instance(&before[key], &after[key], &format!("/{key}"), &mut edits);
    }
    Ok(edits)
}
fn escape_key(key: &str) -> String {
    key.replace('~', "~0").replace('/', "~1")
}
fn diff_instance(before: &Value, after: &Value, path: &str, edits: &mut BTreeMap<String, Value>) {
    if before == after {
        return;
    }
    if path == "/components" {
        let a = before.as_array().cloned().unwrap_or_default();
        let b = after.as_array().cloned().unwrap_or_default();
        let ids = a
            .iter()
            .chain(&b)
            .filter_map(|c| c["id"].as_str())
            .collect::<BTreeSet<_>>();
        for id in ids {
            diff_instance(
                a.iter().find(|c| c["id"] == id).unwrap_or(&Value::Null),
                b.iter().find(|c| c["id"] == id).unwrap_or(&Value::Null),
                &format!("{path}/{id}"),
                edits,
            );
        }
    } else if let (Some(a), Some(b)) = (before.as_object(), after.as_object()) {
        for key in a.keys().chain(b.keys()).collect::<BTreeSet<_>>() {
            diff_instance(
                a.get(key).unwrap_or(&Value::Null),
                b.get(key).unwrap_or(&Value::Null),
                &format!("{path}/{}", escape_key(key)),
                edits,
            );
        }
    } else {
        edits.insert(path.into(), after.clone());
    }
}
fn apply_instance_edits(actor: &mut Actor, edits: &BTreeMap<String, Value>) -> Result<(), String> {
    let mut document = serde_json::to_value(&*actor).map_err(|e| e.to_string())?;
    for (path, value) in edits {
        if ![
            "/name",
            "/active",
            "/attach",
            "/components",
            "/properties",
            "/overrides",
        ]
        .iter()
        .any(|prefix| path == prefix || path.starts_with(&format!("{prefix}/")))
        {
            return Err(format!(
                "Unknown Actor template member {path}; authored override preserved"
            ));
        }
        let mut keys = path
            .strip_prefix('/')
            .ok_or("Invalid instance override path")?
            .split('/')
            .map(|s| s.replace("~1", "/").replace("~0", "~"))
            .collect::<Vec<_>>();
        let last = keys.pop().ok_or("Empty instance override path")?;
        let mut current = &mut document;
        for key in keys {
            if current.is_array() {
                let array = current.as_array_mut().unwrap();
                let index=array.iter().position(|item|item["id"]==key).ok_or_else(||format!("An overridden component {key} was removed from the Blueprint; reset its overrides first."))?;
                current = &mut array[index];
            } else {
                current = current
                    .as_object_mut()
                    .ok_or("Invalid override object")?
                    .entry(key)
                    .or_insert_with(|| serde_json::json!({}));
            }
        }
        if let Some(array) = current.as_array_mut() {
            let index = array.iter().position(|item| item["id"] == last);
            if value.is_null() {
                if let Some(i) = index {
                    array.remove(i);
                }
            } else if let Some(i) = index {
                array[i] = value.clone();
            } else {
                array.push(value.clone());
            }
        } else {
            let fields = current.as_object_mut().ok_or("Invalid override member")?;
            if value.is_null() {
                fields.remove(&last);
            } else {
                fields.insert(last, value.clone());
            }
        }
    }
    *actor = serde_json::from_value(document).map_err(|e| e.to_string())?;
    Ok(())
}
pub fn record_parent_override(scene: &mut Scene, index: usize) -> Result<(), String> {
    let entity = scene
        .actors
        .get(index)
        .ok_or("Edited entity does not exist.")?;
    let parent = match entity.parent {
        Some(index) => ParentOverride::Actor {
            entity: scene.actors.get(index).ok_or("Parent does not exist.")?.id,
        },
        None => ParentOverride::Root,
    };
    if let Some(instance) = &mut scene.actors[index].blueprint_instance {
        instance.parent = Some(parent);
    }
    Ok(())
}

/// Apply updated inherited templates to linked scene instances in one validated
/// transaction. Unknown removed actors/fields remain untouched with an error;
/// new descendants receive fresh authoring UUIDs, never recycled runtime handles.
pub fn refresh_instances(
    scene: &mut Scene,
    files: &[crate::blueprint_asset::AssetFile],
    registry: &Registry,
) -> Result<(), String> {
    scene.validate()?;
    let mut candidate = scene.clone();
    let mut groups = BTreeMap::<Uuid, Vec<usize>>::new();
    for (index, actor) in scene.actors.iter().enumerate() {
        if let Some(state) = &actor.blueprint_instance {
            groups.entry(state.instance).or_default().push(index);
        }
    }
    for (root_id, indices) in groups {
        let root_index = *indices
            .iter()
            .find(|&&i| scene.actors[i].id == root_id)
            .ok_or("Linked Actor root is missing")?;
        let root_state = scene.actors[root_index]
            .blueprint_instance
            .as_ref()
            .unwrap();
        let class = registry
            .classes
            .get(&root_state.class)
            .ok_or("Linked Blueprint class is missing")?;
        let resolved = resolve_assets(files, registry, &class.id)?;
        let template_root = resolved
            .root()
            .ok_or("Linked Blueprint has no Actor template")?;
        if root_state.template_entity != template_root {
            return Err("Linked Blueprint root identity changed".into());
        }
        let prototype = resolved.scene(
            ClassDefaults {
                name: class.cpp_name.clone(),
                class_id: Some(class.id.clone()),
                provider: class.provider.clone(),
                backend: class.backend.clone(),
                ..Default::default()
            },
            registry,
        )?;
        let mut identities = BTreeMap::new();
        let mut positions = BTreeMap::new();
        for index in indices {
            let actor = &scene.actors[index];
            let state = actor.blueprint_instance.as_ref().unwrap();
            if state.class != class.id
                || !prototype
                    .actors
                    .iter()
                    .any(|a| a.id == state.template_entity)
                || positions.insert(state.template_entity, index).is_some()
            {
                return Err("Linked Actor is missing from its Blueprint template".into());
            }
            identities.insert(state.template_entity, actor.id);
            identities.extend(state.component_ids.clone());
        }
        for actor in &prototype.actors {
            identities.entry(actor.id).or_insert_with(Uuid::new_v4);
            for component in &actor.components {
                identities.entry(component.id).or_insert_with(Uuid::new_v4);
            }
        }
        for mut actor in prototype.actors {
            let template_id = actor.id;
            let index = positions.get(&template_id).copied();
            let mut state = index
                .and_then(|i| scene.actors[i].blueprint_instance.clone())
                .unwrap_or(Instance {
                    class: class.id.clone(),
                    template_entity: template_id,
                    component_ids: BTreeMap::new(),
                    instance: root_id,
                    overrides: BTreeMap::new(),
                    parent: None,
                });
            state.component_ids = actor
                .components
                .iter()
                .map(|c| (c.id, identities[&c.id]))
                .collect();
            crate::actor_document::remap_actor(&mut actor, &identities);
            if template_id == template_root {
                actor.logical_parent = scene.actors[root_index].logical_parent;
                actor.attach = scene.actors[root_index].attach.clone();
            }
            apply_instance_edits(&mut actor, &state.overrides)?;
            if let Some(parent) = &state.parent {
                actor.logical_parent = match parent {
                    ParentOverride::Root => None,
                    ParentOverride::Actor { entity } => Some(*entity),
                };
            }
            actor.blueprint_instance = Some(state);
            if let Some(i) = index {
                candidate.actors[i] = actor;
            } else {
                candidate.actors.push(actor);
            }
        }
    }
    candidate.refresh_actor_hierarchy();
    candidate.sync_actor_components();
    candidate.validate()?;
    if candidate.actors != scene.actors {
        candidate.bake = None;
    }
    *scene = candidate;
    Ok(())
}

pub fn resource_ids(template: &ResolvedTemplate) -> BTreeSet<Uuid> {
    let mut ids = BTreeSet::new();
    for item in &template.actors {
        let e = &item.entity;
        ids.extend(e.material.texture);
        if let Some(sprite) = &e.sprite {
            ids.extend(sprite.texture);
        }
        if let Some(image) = &e.image {
            ids.extend(image.texture);
        }
        for audio in crate::audio::sources(e) {
            ids.extend(audio.clip);
        }
        if let Some(mesh) = &e.editable_mesh {
            ids.insert(mesh.asset);
            for material in mesh.materials.values() {
                ids.extend(material.texture);
            }
        }
        if let Some(t) = &e.terrain {
            ids.insert(t.asset);
            ids.extend(t.material.texture);
        }
        if let Some(mesh) = &e.skeletal_mesh {
            ids.insert(mesh.asset);
            ids.extend(mesh.clip);
        }
        if let Some(emitter) = &e.particle_emitter {
            ids.extend(emitter.sprite.texture);
        }
    }
    ids
}
pub fn validate_resources(
    template: &ResolvedTemplate,
    index: &crate::assets::Index,
) -> Result<(), String> {
    for id in resource_ids(template) {
        index.resolve(id)?;
    }
    let expect = |id: Uuid, kind: crate::assets::Kind| -> Result<(), String> {
        let record = index.resolve(id)?;
        if !kind.accepts_runtime(&record.meta.kind) {
            return Err(format!(
                "Template resource {id} must be {kind:?}, found {:?}.",
                record.meta.kind
            ));
        }
        Ok(())
    };
    for item in &template.actors {
        let e = &item.entity;
        let mut textures = vec![];
        textures.extend(e.material.texture);
        if let Some(sprite) = &e.sprite {
            textures.extend(sprite.texture);
        }
        if let Some(image) = &e.image {
            textures.extend(image.texture);
        }
        if let Some(emitter) = &e.particle_emitter {
            textures.extend(emitter.sprite.texture);
        }
        if let Some(audio) = &e.audio
            && let Some(id) = audio.clip
        {
            expect(id, crate::assets::Kind::AudioClip)?;
        }
        if let Some(mesh) = &e.editable_mesh {
            expect(mesh.asset, crate::assets::Kind::EditableMesh)?;
            for material in mesh.materials.values() {
                textures.extend(material.texture);
            }
        }
        if let Some(t) = &e.terrain {
            expect(t.asset, crate::assets::Kind::Terrain)?;
            textures.extend(t.material.texture);
        }
        if let Some(mesh) = &e.skeletal_mesh {
            expect(mesh.asset, crate::assets::Kind::SkeletalMesh)?;
            if let Some(id) = mesh.clip {
                expect(id, crate::assets::Kind::AnimationClip)?;
            }
        }
        for id in textures {
            expect(id, crate::assets::Kind::Texture)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn fixture_registry() -> Registry {
        let mut registry = crate::actor_document::tests::registry();
        let c = crate::actor_document::tests::class(
            "class",
            "BP_Test",
            Some(crate::object_model::ACTOR3D_ID),
        );
        registry.classes.insert(c.id.clone(), c);
        registry
    }
    fn binding() -> ClassDefaults {
        ClassDefaults {
            name: "BP_Test".into(),
            ..Default::default()
        }
    }
    #[test]
    fn captured_root_values_survive_derived_placement_and_cooking() {
        use crate::reflection_schema as schema;
        let location = schema::Location {
            file: "test.hpp".into(),
            line: 1,
            column: 1,
        };
        let base = schema::Class {
            family: None,
            domain: None,
            placement: Default::default(),
            component: None,
            default_components: vec![],
            explicit_abstract: false,
            id: "base".into(),
            cpp_name: "Base".into(),
            parent: None,
            provider: schema::native_provider(),
            backend: schema::native_backend(),
            abstract_class: false,
            final_class: false,
            timeline_component: None,
            blueprintable: true,
            functions: vec![],
            source: location.clone(),
            properties: vec![schema::Property {
                id: "health-id".into(),
                name: "health".into(),
                value_type: Type::Int32,
                default: json!(150),
                editable: true,
                timeline: None,
                source: location,
            }],
        };
        let mut derived = base.clone();
        derived.id = "derived".into();
        derived.cpp_name = "Derived".into();
        derived.parent = Some(base.id.clone());
        derived.properties.clear();
        let mut unrelated = derived.clone();
        unrelated.id = "other".into();
        unrelated.cpp_name = "Other".into();
        unrelated.parent = None;
        let mut registry = fixture_registry();
        for class in [base, derived, unrelated] {
            registry.classes.insert(class.id.clone(), class);
        }
        let authored = ClassDefaults {
            name: "Base".into(),
            class_id: Some("base".into()),
            properties: BTreeMap::from([("health".into(), json!(80))]),
            member_ids: BTreeMap::from([("health".into(), "health-id".into())]),
            overrides: std::collections::BTreeSet::from(["health".into()]),
            ..Default::default()
        };
        let target = ClassDefaults {
            name: "Derived".into(),
            class_id: Some("derived".into()),
            ..Default::default()
        };
        let mut scene = Scene::default();
        scene.actors[1].set_class_defaults(&(authored.clone()));
        let template = capture(&scene, 1, &registry).unwrap();
        let resolved = resolve(&[&template]).unwrap();
        for destination in [authored.clone(), target.clone()] {
            let prototype = resolved.scene(destination.clone(), &registry).unwrap();
            let root = prototype
                .actors
                .iter()
                .find(|entity| entity.parent.is_none())
                .unwrap();
            assert_eq!(root.properties["health"], json!(80));
            assert_eq!(root.class.class_id, destination.class_id);
            let placed = place(&mut scene, &resolved, destination, &registry, None).unwrap();
            assert_eq!(scene.actors[placed.root].properties["health"], json!(80));
        }
        let mut explicit = target.clone();
        explicit.properties.insert("health".into(), json!(42));
        let prototype = resolved.scene(explicit, &registry).unwrap();
        assert_eq!(
            prototype
                .actors
                .iter()
                .find(|entity| entity.parent.is_none())
                .unwrap()
                .properties["health"],
            json!(42)
        );
        let before = scene.clone();
        let wrong = ClassDefaults {
            name: "Other".into(),
            class_id: Some("other".into()),
            ..Default::default()
        };
        assert!(place(&mut scene, &resolved, wrong, &registry, None).is_err());
        assert_eq!(scene, before);
        let mut orphan = template.clone();
        orphan
            .actors
            .iter_mut()
            .find(|entity| entity.parent.is_none())
            .unwrap()
            .entity
            .properties
            .insert("removed".into(), json!(1));
        assert!(
            resolve(&[&orphan])
                .unwrap()
                .scene(target, &registry)
                .unwrap_err()
                .contains("Orphaned")
        );
    }
    #[test]
    fn template_inheritance_construction_and_placement_are_isolated() {
        let mut base = Template::root("Root");
        let root = base.actors[0].entity.id;
        let child = base.add_child(root, "Child");
        let mut derived = Template::default();
        derived.overrides.entry(child).or_default().members.insert(
            format!(
                "/components/{}/properties/position",
                base.actors[1].entity.root().unwrap().id
            ),
            json!([1., 2., 3.]),
        );
        derived.construction.push(ConstructionOp::Translate {
            entity: child,
            offset: [0.5, 0., 0.],
        });
        let resolved = resolve(&[&base, &derived]).unwrap();
        assert_eq!(
            resolved
                .actors
                .iter()
                .find(|item| item.entity.id == child)
                .unwrap()
                .entity
                .position,
            [1.5, 2., 3.]
        );
        assert_eq!(base.actors[1].entity.position, [0.; 3]);
        let mut scene = Scene::default();
        let original = scene.actors.len();
        let first = place(
            &mut scene,
            &resolved,
            binding(),
            &fixture_registry(),
            Some(0),
        )
        .unwrap();
        let second = place(&mut scene, &resolved, binding(), &fixture_registry(), None).unwrap();
        assert_eq!(first.actors.len(), 2);
        assert_eq!(scene.actors.len(), original + 4);
        assert_ne!(first.identities[&root], second.identities[&root]);
        assert_eq!(scene.actors[first.root].parent, Some(0));
        assert_eq!(scene.actors[second.root].parent, None);
        assert_eq!(scene.actors[first.root].class.name, "BP_Test");
        assert_eq!(
            resolved
                .scene(binding(), &fixture_registry())
                .unwrap()
                .actors
                .iter()
                .find(|e| e.id == child)
                .unwrap()
                .position,
            [1.5, 2., 3.]
        );
    }
    #[test]
    fn invalid_templates_and_failed_placement_preserve_authoring_data() {
        let mut base = Template::root("Root");
        let root = base.actors[0].entity.id;
        let child = base.add_child(root, "Child");
        let mut derived = Template::default();
        derived.overrides.entry(root).or_default().parent =
            Some(ParentOverride::Actor { entity: child });
        assert!(resolve(&[&base, &derived]).is_err());
        derived.overrides.clear();
        derived.overrides.entry(Uuid::new_v4()).or_default();
        assert!(resolve(&[&base, &derived]).is_err());
        derived.overrides.clear();
        derived
            .overrides
            .entry(child)
            .or_default()
            .members
            .insert("unknown".into(), json!(1));
        assert!(
            resolve(&[&base, &derived])
                .unwrap_err()
                .contains("preserved")
        );
        let resolved = resolve(&[&base]).unwrap();
        let mut scene = Scene::default();
        let original = scene.clone();
        assert!(
            place(
                &mut scene,
                &resolved,
                binding(),
                &fixture_registry(),
                Some(999)
            )
            .is_err()
        );
        assert_eq!(scene, original);
        assert!(base.remove_local(root).is_err());
        assert!(base.remove_local(Uuid::new_v4()).is_err());
        base.remove_local(child).unwrap();
        assert_eq!(base.actors.len(), 1);
    }
    #[test]
    fn construction_is_bounded_quantized_and_validated() {
        let mut template = Template::root("Root");
        let root = template.actors[0].entity.id;
        template.construction.push(ConstructionOp::Translate {
            entity: root,
            offset: [0.0001, 0.0002, -0.0002],
        });
        assert_eq!(
            resolve(&[&template]).unwrap().actors[0].entity.position,
            [0., 1. / 4096., -1. / 4096.]
        );
        template.construction.push(ConstructionOp::SetScale {
            entity: root,
            value: [0., 1., 1.],
        });
        assert!(resolve(&[&template]).is_err());
        template.construction = vec![
            ConstructionOp::SetActive {
                entity: root,
                active: true
            };
            MAX_CONSTRUCTION_OPS + 1
        ];
        assert!(resolve(&[&template]).is_err());
        template.construction = vec![ConstructionOp::Translate {
            entity: root,
            offset: [f32::NAN, 0., 0.],
        }];
        assert!(resolve(&[&template]).is_err());
    }
    #[test]
    fn captured_hierarchy_gets_portable_ids_and_missing_resources_fail() {
        let mut scene = Scene::default();
        scene.actors[2].parent = Some(1);
        let captured = capture(&scene, 1, &fixture_registry()).unwrap();
        assert_eq!(captured.actors.len(), 2);
        assert!(
            captured
                .actors
                .iter()
                .all(|item| item.entity.parent.is_none()
                    && !scene.actors.iter().any(|e| e.id == item.entity.id))
        );
        let mut resolved = resolve(&[&captured]).unwrap();
        resolved.actors[0].entity.material.texture = Some(Uuid::new_v4());
        assert!(
            validate_resources(&resolved, &crate::assets::Index::default())
                .unwrap_err()
                .contains("Missing asset")
        );
    }
    #[test]
    fn template_entity_references_remap_by_identity_and_fail_without_data_loss() {
        use crate::reflection_schema as schema;
        let mut registry = fixture_registry();
        registry.classes.insert(
            "class".into(),
            schema::Class {
                family: None,
                domain: None,
                placement: Default::default(),
                component: None,
                default_components: vec![],
                explicit_abstract: false,
                id: "class".into(),
                provider: schema::native_provider(),
                backend: schema::native_backend(),
                cpp_name: "BP_Test".into(),
                parent: Some(crate::object_model::ACTOR3D_ID.into()),
                abstract_class: false,
                final_class: false,
                timeline_component: None,
                blueprintable: true,
                functions: vec![],
                source: schema::Location {
                    file: "BP_Test.epokbp".into(),
                    line: 1,
                    column: 1,
                },
                properties: vec![schema::Property {
                    id: "target-property".into(),
                    name: "target".into(),
                    value_type: Type::ObjectRef { class: None },
                    default: Value::Null,
                    editable: true,
                    timeline: None,
                    source: schema::Location {
                        file: "BP_Test.epokbp".into(),
                        line: 1,
                        column: 1,
                    },
                }],
            },
        );
        let mut template = Template::root("Owner");
        let root = template.actors[0].entity.id;
        let target = template.add_child(root, "Target");
        template.references.push(ObjectReference {
            owner: root,
            member: "target-property".into(),
            target,
        });
        let timeline_slot = Uuid::new_v4();
        template.actors[0].entity.timeline = Some(crate::timeline_scene::Component {
            asset: Some(Uuid::new_v4()),
            bindings: BTreeMap::from([(timeline_slot, Some(target))]),
            ..Default::default()
        });
        let resolved = resolve(&[&template]).unwrap();
        let mut scene = Scene::default();
        let first = place(&mut scene, &resolved, binding(), &registry, None).unwrap();
        let second = place(&mut scene, &resolved, binding(), &registry, None).unwrap();
        assert_eq!(
            scene.actors[first.root].properties["target"],
            json!(first.identities[&target].to_string())
        );
        assert_eq!(
            scene.actors[second.root].properties["target"],
            json!(second.identities[&target].to_string())
        );
        let captured = capture(&scene, first.root, &registry).unwrap();
        assert_eq!(
            scene.actors[first.root].timeline.as_ref().unwrap().bindings[&timeline_slot],
            Some(first.identities[&target])
        );
        assert_eq!(
            scene.actors[second.root]
                .timeline
                .as_ref()
                .unwrap()
                .bindings[&timeline_slot],
            Some(second.identities[&target])
        );
        let captured_target = captured
            .actors
            .iter()
            .find_map(|item| item.entity.timeline.as_ref())
            .unwrap()
            .bindings[&timeline_slot]
            .unwrap();
        assert!(
            captured
                .actors
                .iter()
                .any(|item| item.entity.id == captured_target)
        );
        let preserved = scene.clone();
        let external_timeline_target = scene.actors[0].id;
        scene.actors[first.root]
            .timeline
            .as_mut()
            .unwrap()
            .bindings
            .insert(timeline_slot, Some(external_timeline_target));
        assert!(
            capture(&scene, first.root, &registry)
                .unwrap_err()
                .contains("outside")
        );
        scene = preserved;
        assert_eq!(captured.references.len(), 1);
        assert!(
            captured
                .actors
                .iter()
                .any(|item| item.entity.id == captured.references[0].target)
        );
        let before = scene.clone();
        let mut invalid = resolved.clone();
        invalid.references[0].member = "removed-member".into();
        assert!(
            place(&mut scene, &invalid, binding(), &registry, None)
                .unwrap_err()
                .contains("Orphaned template reference member")
        );
        assert_eq!(scene, before);
        let external = scene.actors[0].id.to_string();
        scene.actors[first.root]
            .properties
            .insert("target".into(), json!(external));
        assert!(
            capture(&scene, first.root, &registry)
                .unwrap_err()
                .contains("outside")
        );
    }
    #[test]
    fn linked_instances_refresh_defaults_preserve_explicit_edits_and_keep_orphans() {
        use crate::{
            blueprint_asset::{AssetFile, BlueprintAsset},
            reflection_schema as schema,
        };
        let native = schema::Class {
            family: None,
            domain: None,
            placement: Default::default(),
            component: None,
            default_components: vec![],
            explicit_abstract: false,
            id: "native".into(),
            provider: schema::native_provider(),
            backend: schema::native_backend(),
            cpp_name: "Native".into(),
            parent: Some(crate::object_model::ACTOR3D_ID.into()),
            abstract_class: false,
            final_class: false,
            timeline_component: None,
            blueprintable: true,
            functions: vec![],
            properties: vec![],
            source: schema::Location {
                file: "Native.hpp".into(),
                line: 1,
                column: 1,
            },
        };
        let mut asset = BlueprintAsset::new("BP_Test".into(), "native".into());
        asset.template = Template::root("Root");
        let root = asset.template.actors[0].entity.id;
        let slot = Uuid::new_v4();
        asset.template.actors[0].entity.timeline = Some(crate::timeline_scene::Component {
            asset: Some(Uuid::new_v4()),
            bindings: BTreeMap::from([(slot, Some(root))]),
            ..Default::default()
        });
        let visual = schema::Class {
            family: None,
            domain: None,
            placement: Default::default(),
            component: None,
            default_components: vec![],
            explicit_abstract: false,
            id: asset.id.clone(),
            cpp_name: asset.name.clone(),
            parent: Some("native".into()),
            ..native.clone()
        };
        let mut registry = fixture_registry();
        registry.classes.insert(native.id.clone(), native);
        registry.classes.insert(visual.id.clone(), visual);
        let mut files = vec![AssetFile::file("BP_Test.epokbp".into(), asset)];
        let class = files[0].asset.id.clone();
        let resolved = resolve_assets(&files, &registry, &class).unwrap();
        let binding = ClassDefaults {
            name: "BP_Test".into(),
            class_id: Some(class.clone()),
            ..Default::default()
        };
        let mut scene = Scene::default();
        let first = place(&mut scene, &resolved, binding.clone(), &registry, None).unwrap();
        let second = place(&mut scene, &resolved, binding, &registry, None).unwrap();
        let original_audio = crate::audio::selection_signature(&scene, None);
        let before = scene.actors[first.root].clone();
        scene.actors[first.root].position = [9., 0., 0.];
        record_overrides(&before, &mut scene.actors[first.root]);
        assert_eq!(
            original_audio,
            crate::audio::selection_signature(&scene, None)
        );
        files[0].asset.template.actors[0].entity.position = [2., 3., 4.];
        let child = files[0].asset.template.add_child(root, "New Child");
        refresh_instances(&mut scene, &files, &registry).unwrap();
        assert_eq!(
            scene.actors[first.root].timeline.as_ref().unwrap().bindings[&slot],
            Some(first.identities[&root])
        );
        assert_eq!(
            scene.actors[second.root]
                .timeline
                .as_ref()
                .unwrap()
                .bindings[&slot],
            Some(second.identities[&root])
        );
        assert_eq!(scene.actors[first.root].position, [9., 0., 0.]);
        assert_eq!(scene.actors[second.root].position, [2., 3., 4.]);
        assert_eq!(
            scene
                .actors
                .iter()
                .filter(|entity| entity
                    .blueprint_instance
                    .as_ref()
                    .is_some_and(|state| state.template_entity == child))
                .count(),
            2
        );
        assert_eq!(scene.actors[first.root].id, first.identities[&root]);
        let position_path = format!(
            "/components/{}/properties/position",
            scene.actors[first.root].root().unwrap().id
        );
        // Reset explicitly removes the override; simply reading equal values does not.
        scene.actors[first.root]
            .blueprint_instance
            .as_mut()
            .unwrap()
            .overrides
            .remove(&position_path);
        refresh_instances(&mut scene, &files, &registry).unwrap();
        assert_eq!(scene.actors[first.root].position, [2., 3., 4.]);
        let clip = Uuid::new_v4();
        let before_audio = crate::audio::selection_signature(&scene, None);
        let before = scene.actors[first.root].clone();
        scene.actors[first.root].audio = Some(crate::audio::AudioSource {
            clip: Some(clip),
            volume: 0.5,
            ..Default::default()
        });
        record_overrides(&before, &mut scene.actors[first.root]);
        assert_ne!(
            before_audio,
            crate::audio::selection_signature(&scene, None)
        );
        refresh_instances(&mut scene, &files, &registry).unwrap();
        assert_eq!(crate::audio::clip_ids(&scene), vec![clip]);
        let selected = crate::audio::selection_signature(&scene, None);
        let before = scene.actors[first.root].clone();
        scene.actors[first.root].audio.as_mut().unwrap().volume = 0.25;
        record_overrides(&before, &mut scene.actors[first.root]);
        refresh_instances(&mut scene, &files, &registry).unwrap();
        assert_eq!(selected, crate::audio::selection_signature(&scene, None));
        let before = scene.actors[first.root].clone();
        scene.actors[first.root].audio = None;
        record_overrides(&before, &mut scene.actors[first.root]);
        refresh_instances(&mut scene, &files, &registry).unwrap();
        assert!(crate::audio::clip_ids(&scene).is_empty());
        let valid = crate::audio::selection_signature(&scene, None);
        scene.actors[first.root]
            .blueprint_instance
            .as_mut()
            .unwrap()
            .overrides
            .insert("future.component.audio.v2".into(), json!({"clip": clip}));
        let _ = valid;
        let preserved = scene.clone();
        assert!(
            refresh_instances(&mut scene, &files, &registry)
                .unwrap_err()
                .contains("Unknown Actor template member")
        );
        assert_eq!(scene, preserved);
        scene.actors[first.root]
            .blueprint_instance
            .as_mut()
            .unwrap()
            .overrides
            .remove("future.component.audio.v2");
        let before = scene.clone();
        files[0].asset.template.remove_local(child).unwrap();
        assert!(
            refresh_instances(&mut scene, &files, &registry)
                .unwrap_err()
                .contains("missing from its Blueprint template")
        );
        assert_eq!(scene, before);
    }
}
