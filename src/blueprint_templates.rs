//! Inherited entity templates and a deterministic, bounded host construction backend.
//!
//! Construction edits authored component data only. It never executes native
//! script bodies, starts a game, loads a MIPS module, or evaluates arbitrary JSON.
use crate::{
    blueprint::Registry,
    reflection_schema::Type,
    scene::{Entity, Scene, ScriptBinding},
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
    pub entities: Vec<TemplateEntity>,
    #[serde(default)]
    pub overrides: BTreeMap<Uuid, EntityOverride>,
    #[serde(default)]
    pub references: Vec<EntityReference>,
    #[serde(default)]
    pub construction: Vec<ConstructionOp>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TemplateEntity {
    /// entity.id is the persistent template entity ID, never a runtime handle.
    pub entity: Entity,
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
    Entity { entity: Uuid },
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EntityReference {
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
    pub entities: Vec<TemplateEntity>,
    pub references: Vec<EntityReference>,
}
#[derive(Clone, Debug)]
pub struct Placement {
    pub root: usize,
    pub entities: Vec<usize>,
    pub identities: BTreeMap<Uuid, Uuid>,
}

impl Template {
    pub fn root(name: &str) -> Self {
        let mut entity = Entity::cube(name.into());
        entity.kind = "Empty".into();
        entity.position = [0.; 3];
        Self {
            entities: vec![TemplateEntity {
                entity,
                parent: None,
            }],
            ..Self::default()
        }
    }
    pub fn add_child(&mut self, parent: Uuid, name: &str) -> Uuid {
        let mut entity = Entity::cube(name.into());
        entity.kind = "Empty".into();
        entity.position = [0.; 3];
        let id = entity.id;
        self.entities.push(TemplateEntity {
            entity,
            parent: Some(parent),
        });
        id
    }
    pub fn remove_local(&mut self, id: Uuid) -> Result<(), String> {
        if !self.entities.iter().any(|item| item.entity.id == id) {
            return Err("Inherited template entities cannot be deleted; deactivate explicitly or edit the declaring Blueprint.".into());
        }
        if self.entities.iter().any(|item| item.parent == Some(id))
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
        self.entities.retain(|item| item.entity.id != id);
        self.overrides.remove(&id);
        Ok(())
    }
}

fn member_field(id: &str) -> Option<&'static str> {
    Some(match id {
        "uq.entity.name.v1" => "name",
        "uq.entity.kind.v1" => "kind",
        "uq.entity.active.v1" => "active",
        "uq.entity.position.v1" => "position",
        "uq.entity.rotation.v1" => "rotation",
        "uq.entity.scale.v1" => "scale",
        "uq.entity.material.v1" => "material",
        "uq.entity.camera_fov.v1" => "camera_fov",
        "uq.component.sprite.v1" => "sprite",
        "uq.component.sprite_animator.v1" => "sprite_animator",
        "uq.component.particle_emitter.v1" => "particle_emitter",
        "epok.component.timeline.v1" => "timeline",
        "epok.component.particle_effect.v1" => "particle_effect",
        "uq.component.collider.v1" => "collider",
        "uq.component.palette_animator.v1" => "palette_animator",
        "uq.component.skeletal_mesh.v1" => "skeletal_mesh",
        "uq.component.editable_mesh.v1" => "editable_mesh",
        "uq.component.audio.v1" => "audio",
        "uq.component.canvas.v1" => "canvas",
        "uq.component.rect.v1" => "rect",
        "uq.component.image.v1" => "image",
        "uq.component.text.v1" => "text",
        "uq.component.progress.v1" => "progress",
        "uq.component.lighting.v1" => "lighting",
        "uq.component.light.v1" => "light",
        "uq.component.blob_shadow.v1" => "blob_shadow",
        "uq.component.script.v1" => "script",
        _ => return None,
    })
}
fn apply_override(item: &mut TemplateEntity, edits: &EntityOverride) -> Result<(), String> {
    let mut document = serde_json::to_value(&item.entity).map_err(|error| error.to_string())?;
    for (id, value) in &edits.members {
        let field = member_field(id).ok_or_else(|| {
            format!("Unknown template member {id}; preserved override requires migration.")
        })?;
        document[field] = value.clone();
    }
    item.entity = serde_json::from_value(document).map_err(|error| {
        format!(
            "Template {}: invalid typed override: {error}",
            item.entity.id
        )
    })?;
    if let Some(parent) = &edits.parent {
        item.parent = match parent {
            ParentOverride::Root => None,
            ParentOverride::Entity { entity } => Some(*entity),
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
    let selected = members
        .iter()
        .filter_map(|(id, value)| {
            match member_field(id) {
                Some(
                    "name" | "kind" | "active" | "position" | "rotation" | "scale" | "material"
                    | "camera_fov" | "sprite" | "sprite_animator" | "particle_emitter" | "collider"
                    | "palette_animator" | "skeletal_mesh" | "editable_mesh" | "canvas" | "rect"
                    | "image" | "text" | "progress" | "lighting" | "light" | "blob_shadow",
                ) => None,
                Some("audio" | "script" | "timeline" | "particle_effect") => {
                    let mut entity = Entity::cube("Audio selection".into());
                    entity.id = Uuid::nil();
                    let mut item = TemplateEntity {
                        entity,
                        parent: None,
                    };
                    let edits = EntityOverride {
                        members: [(id.clone(), value.clone())].into(),
                        ..Default::default()
                    };
                    let selection = apply_override(&mut item, &edits).map(|()| {
                        crate::audio::selection_signature(
                            &Scene {
                                entities: vec![item.entity],
                                ..Default::default()
                            },
                            registry,
                        )
                    });
                    Some((
                        id,
                        match selection {
                            Ok(signature) => serde_json::json!(["typed", signature]),
                            Err(_) => serde_json::json!(["invalid", value]),
                        },
                    ))
                }
                // New component kinds must remain conservative until their resource
                // behavior is explicitly included in this projection.
                _ => Some((id, serde_json::json!(["unclassified", value]))),
            }
        })
        .collect::<BTreeMap<_, _>>();
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
            entities: vec![],
            ..Default::default()
        },
        registry,
    );
    let mut entities = template
        .entities
        .iter()
        .map(|item| {
            (
                item.entity.id,
                crate::audio::selection_signature(
                    &Scene {
                        entities: vec![item.entity.clone()],
                        ..Default::default()
                    },
                    registry,
                ),
            )
        })
        .filter(|(_, signature)| *signature != empty)
        .collect::<Vec<_>>();
    entities.sort();
    let overrides = template
        .overrides
        .iter()
        .filter_map(|(id, edits)| {
            audio_members_signature(&edits.members, registry).map(|signature| (id, signature))
        })
        .collect::<BTreeMap<_, _>>();
    // These operations cannot select resources, including on inactive entities.
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
    crate::scene_dependencies::hash((entities, overrides))
}

/// Resolve already ordered native-to-derived template layers. Inputs are never
/// modified on error. Derived overrides retain their identity and explicitness.
pub fn resolve(layers: &[&Template]) -> Result<ResolvedTemplate, String> {
    if layers.len() > 64 {
        return Err("Template inheritance exceeds 64 classes.".into());
    }
    let mut entities = BTreeMap::<Uuid, TemplateEntity>::new();
    let mut references = BTreeMap::new();
    let mut operations = vec![];
    for layer in layers {
        for item in &layer.entities {
            if item.entity.id.is_nil()
                || item.entity.parent.is_some()
                || entities.insert(item.entity.id, item.clone()).is_some()
            {
                return Err("Template entity requires a unique non-nil UUID and UUID parent; numeric scene parents are not portable.".into());
            }
        }
        if entities.len() > MAX_ENTITIES {
            return Err(format!("Template exceeds {MAX_ENTITIES} entities."));
        }
        for (id, edits) in &layer.overrides {
            apply_override(
                entities.get_mut(id).ok_or_else(|| {
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
        construct(&mut entities, operation)?;
    }
    let result = ResolvedTemplate {
        entities: entities.into_values().collect(),
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
    entities: &mut BTreeMap<Uuid, TemplateEntity>,
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
    let item = entities
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
        self.entities
            .iter()
            .find(|item| item.parent.is_none())
            .map(|item| item.entity.id)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.entities.is_empty() {
            return if self.references.is_empty() {
                Ok(())
            } else {
                Err("An empty template cannot contain references.".into())
            };
        }
        if self.entities.len() > MAX_ENTITIES
            || self
                .entities
                .iter()
                .filter(|item| item.parent.is_none())
                .count()
                != 1
        {
            return Err("Template requires exactly one root and at most 32 entities.".into());
        }
        let indices = self
            .entities
            .iter()
            .enumerate()
            .map(|(index, item)| (item.entity.id, index))
            .collect::<BTreeMap<_, _>>();
        let mut scene = Scene::default();
        scene.entities.clear();
        for item in &self.entities {
            let mut entity = item.entity.clone();
            entity.parent = item
                .parent
                .map(|id| {
                    indices
                        .get(&id)
                        .copied()
                        .ok_or_else(|| format!("Missing template parent {id}."))
                })
                .transpose()?;
            scene.entities.push(entity);
        }
        scene.validate()?;
        for reference in &self.references {
            if !indices.contains_key(&reference.owner)
                || !indices.contains_key(&reference.target)
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
    pub fn scene(&self, binding: ScriptBinding, registry: &Registry) -> Result<Scene, String> {
        self.validate()?;
        let root = self.root().ok_or("Blueprint has no entity template.")?;
        let indices = self
            .entities
            .iter()
            .enumerate()
            .map(|(index, item)| (item.entity.id, index))
            .collect::<BTreeMap<_, _>>();
        let mut scene = Scene::default();
        scene.entities.clear();
        scene.name = "BlueprintTemplate".into();
        for item in &self.entities {
            let mut entity = item.entity.clone();
            entity.parent = item.parent.map(|id| indices[&id]);
            if entity.id == root {
                let mut effective = binding.clone();
                if let Some(authored) = &entity.script {
                    let source = registry.bound(authored).ok_or(
                        "Template root behaviour class is unavailable; its values were preserved.",
                    )?;
                    let target = registry
                        .bound(&binding)
                        .ok_or("Template root destination class is unavailable.")?;
                    if !registry
                        .ancestry(&target.cpp_name)
                        .iter()
                        .any(|class| class.id == source.id)
                    {
                        return Err("Template root behaviour must be the destination class or an ancestor; its values were preserved.".into());
                    }
                    let properties = registry.properties(&target.cpp_name);
                    for (name, value) in &authored.properties {
                        let property = properties.iter().find(|property| property.name == *name)
                            .ok_or_else(|| format!("Orphaned template root property {name} is preserved; migrate it explicitly."))?;
                        if authored.member_ids.get(name).is_some_and(|id| {
                            id != &property.id
                                && id != &format!("legacy:{}:{name}", source.cpp_name)
                        }) {
                            return Err(format!(
                                "Template root property {name} changed identity; migrate it explicitly."
                            ));
                        }
                        if !effective.properties.contains_key(name) {
                            effective.properties.insert(name.clone(), value.clone());
                            effective
                                .member_ids
                                .insert(name.clone(), property.id.clone());
                            if authored.overrides.contains(name) {
                                effective.overrides.insert(name.clone());
                            }
                        }
                    }
                }
                entity.script = Some(effective);
            }
            scene.entities.push(entity);
        }
        for reference in &self.references {
            let owner = &mut scene.entities[indices[&reference.owner]];
            let binding = owner.script.as_mut().ok_or_else(|| {
                format!("Template reference owner {} has no behaviour.", owner.name)
            })?;
            let class = registry.bound(binding).ok_or_else(|| {
                format!(
                    "Template reference owner {} has an unavailable class.",
                    owner.name
                )
            })?;
            let property = registry
                .properties(&class.cpp_name)
                .into_iter()
                .find(|property| property.id == reference.member)
                .ok_or_else(|| {
                    format!(
                        "Template reference property {} is missing; migrate explicitly.",
                        reference.member
                    )
                })?;
            if !matches!(property.value_type, Type::EntityRef { .. }) {
                return Err(format!(
                    "Template reference {} is not EntityRef.",
                    reference.member
                ));
            }
            binding.properties.insert(
                property.name.clone(),
                Value::String(reference.target.to_string()),
            );
            binding
                .member_ids
                .insert(property.name.clone(), property.id.clone());
            binding.overrides.insert(property.name.clone());
            if let Type::EntityRef {
                class: Some(expected),
            } = &property.value_type
            {
                let target = &scene.entities[indices[&reference.target]];
                let class=target.script.as_ref().and_then(|binding|registry.bound(binding)).ok_or_else(||format!("Template reference {} requires a target behaviour of class {expected}.",reference.member))?;
                if !registry
                    .ancestry(&class.cpp_name)
                    .iter()
                    .any(|class| class.id == *expected)
                {
                    return Err(format!(
                        "Template reference {} target does not derive from {expected}.",
                        reference.member
                    ));
                }
            }
        }
        scene.validate()?;
        Ok(scene)
    }
}
pub fn place(
    scene: &mut Scene,
    template: &ResolvedTemplate,
    binding: ScriptBinding,
    registry: &Registry,
    parent: Option<usize>,
) -> Result<Placement, String> {
    if parent.is_some_and(|index| index >= scene.entities.len()) {
        return Err("Placement parent does not exist.".into());
    }
    let class = binding
        .class_id
        .clone()
        .or_else(|| registry.bound(&binding).map(|class| class.id.clone()))
        .unwrap_or_else(|| binding.name.clone());
    let prototype = template.scene(binding, registry)?;
    let mut candidate = scene.clone();
    let base = candidate.entities.len();
    let identities = prototype
        .entities
        .iter()
        .map(|entity| (entity.id, Uuid::new_v4()))
        .collect::<BTreeMap<_, _>>();
    let root_id = template.root().ok_or("Blueprint has no entity template.")?;
    let root = base
        + prototype
            .entities
            .iter()
            .position(|entity| entity.id == root_id)
            .ok_or("Template root missing.")?;
    for mut entity in prototype.entities {
        let old_id = entity.id;
        entity.id = identities[&old_id];
        if let Some(component) = &mut entity.timeline {
            crate::timeline_scene::remap(component, &identities, true)?;
        }
        if let Some(component) = &mut entity.particle_effect {
            crate::particle_effect_scene::remap(component, &identities, true)?;
        }
        entity.blueprint_instance = Some(Instance {
            class: class.clone(),
            template_entity: old_id,
            instance: identities[&root_id],
            overrides: BTreeMap::new(),
            parent: None,
        });
        entity.parent = entity.parent.map(|index| index + base).or(parent);
        if let Some(binding) = &mut entity.script {
            for reference in template
                .references
                .iter()
                .filter(|reference| reference.owner == old_id)
            {
                if let Some((name, _)) = binding
                    .member_ids
                    .iter()
                    .find(|(_, id)| **id == reference.member)
                {
                    binding.properties.insert(
                        name.clone(),
                        Value::String(identities[&reference.target].to_string()),
                    );
                }
            }
        }
        candidate.entities.push(entity);
    }
    candidate.bake = None;
    candidate.validate()?;
    let entities = (base..candidate.entities.len()).collect();
    *scene = candidate;
    Ok(Placement {
        root,
        entities,
        identities,
    })
}

/// Capture a selected scene subtree without retaining links to entities outside
/// it. Reflected EntityRef values become stable explicit template references.
pub fn capture(scene: &Scene, root: usize, registry: &Registry) -> Result<Template, String> {
    scene.validate()?;
    if root >= scene.entities.len() {
        return Err("Capture root does not exist.".into());
    }
    let selected = (0..scene.entities.len())
        .filter(|&index| scene.is_descendant(index, root))
        .collect::<Vec<_>>();
    let identities = selected
        .iter()
        .map(|&index| (scene.entities[index].id, Uuid::new_v4()))
        .collect::<BTreeMap<_, _>>();
    let mut template = Template::default();
    for index in selected {
        let mut entity = scene.entities[index].clone();
        entity.id = identities[&entity.id];
        if let Some(component) = &mut entity.timeline {
            crate::timeline_scene::remap(component, &identities, true)?;
        }
        if let Some(component) = &mut entity.particle_effect {
            crate::particle_effect_scene::remap(component, &identities, true)?;
        }
        entity.blueprint_instance = None;
        let parent = if index == root {
            None
        } else {
            entity.parent.map(|p| identities[&scene.entities[p].id])
        };
        entity.parent = None;
        if let Some(binding) = &mut entity.script {
            let class = registry
                .bound(binding)
                .ok_or_else(|| format!("Cannot capture unavailable behaviour {}.", binding.name))?;
            for property in registry.properties(&class.cpp_name) {
                if !matches!(property.value_type, Type::EntityRef { .. }) {
                    continue;
                }
                let value = binding
                    .properties
                    .get(&property.name)
                    .unwrap_or(&property.default);
                if value.is_null() {
                    continue;
                }
                let source = value
                    .as_str()
                    .and_then(|value| Uuid::parse_str(value).ok())
                    .ok_or("Invalid captured EntityRef.")?;
                let target = *identities.get(&source).ok_or("Template capture cannot retain an EntityRef outside its subtree; clear or replace it first.")?;
                template.references.push(EntityReference {
                    owner: entity.id,
                    member: property.id.clone(),
                    target,
                });
                binding
                    .properties
                    .insert(property.name.clone(), Value::Null);
            }
        }
        template.entities.push(TemplateEntity { entity, parent });
    }
    resolve(&[&template])?;
    Ok(template)
}

/// Call only for a known user edit. Reading or refreshing an instance must never
/// infer override intent from equality with a default.
pub fn record_overrides(original: &Entity, edited: &mut Entity) {
    if original.id != edited.id {
        return;
    }
    let Ok(before) = serde_json::to_value(original) else {
        return;
    };
    let Ok(after) = serde_json::to_value(&*edited) else {
        return;
    };
    let Some(instance) = &mut edited.blueprint_instance else {
        return;
    };
    let Some(before) = before.as_object() else {
        return;
    };
    let Some(after) = after.as_object() else {
        return;
    };
    let keys = before.keys().chain(after.keys()).collect::<BTreeSet<_>>();
    for field in keys {
        let category = if matches!(
            field.as_str(),
            "name"
                | "kind"
                | "active"
                | "position"
                | "rotation"
                | "scale"
                | "material"
                | "camera_fov"
        ) {
            "entity"
        } else {
            "component"
        };
        let id = if field == "timeline" {
            "epok.component.timeline.v1".into()
        } else if field == "particle_effect" {
            "epok.component.particle_effect.v1".into()
        } else {
            format!("uq.{category}.{field}.v1")
        };
        if member_field(&id).is_some() && before.get(field) != after.get(field) {
            instance
                .overrides
                .insert(id, after.get(field).cloned().unwrap_or(Value::Null));
        }
    }
}
pub fn record_parent_override(scene: &mut Scene, index: usize) -> Result<(), String> {
    let entity = scene
        .entities
        .get(index)
        .ok_or("Edited entity does not exist.")?;
    let parent = match entity.parent {
        Some(index) => ParentOverride::Entity {
            entity: scene
                .entities
                .get(index)
                .ok_or("Parent does not exist.")?
                .id,
        },
        None => ParentOverride::Root,
    };
    if let Some(instance) = &mut scene.entities[index].blueprint_instance {
        instance.parent = Some(parent);
    }
    Ok(())
}

/// Apply updated inherited templates to linked scene instances in one validated
/// transaction. Unknown removed entities/fields remain untouched with an error;
/// new descendants receive fresh authoring UUIDs, never recycled runtime handles.
pub fn refresh_instances(
    scene: &mut Scene,
    files: &[crate::blueprint_asset::AssetFile],
    registry: &Registry,
) -> Result<(), String> {
    scene.validate()?;
    let mut candidate = scene.clone();
    let mut groups = BTreeMap::<Uuid, Vec<usize>>::new();
    for (index, entity) in scene.entities.iter().enumerate() {
        if let Some(instance) = &entity.blueprint_instance {
            groups.entry(instance.instance).or_default().push(index);
        }
    }
    for (root_id, indices) in groups {
        let root_index = *indices.iter().find(|&&index|scene.entities[index].id == root_id).ok_or("Linked Blueprint instance root is missing; restore it or explicitly unlink the remaining entities.")?;
        let root_state = scene.entities[root_index]
            .blueprint_instance
            .as_ref()
            .unwrap();
        let class = registry.classes.get(&root_state.class).ok_or_else(|| {
            format!(
                "Linked Blueprint class {} is unavailable; instance data was preserved.",
                root_state.class
            )
        })?;
        let resolved = resolve_assets(files, registry, &class.id)?;
        let binding = ScriptBinding {
            name: class.cpp_name.clone(),
            class_id: Some(class.id.clone()),
            provider: class.provider.clone(),
            backend: class.backend.clone(),
            ..Default::default()
        };
        let prototype = resolved.scene(binding, registry)?;
        let template_root = resolved
            .root()
            .ok_or("Linked Blueprint lost its template; instance data was preserved.")?;
        if root_state.template_entity != template_root {
            return Err(
                "Linked Blueprint changed its root identity; explicit migration is required."
                    .into(),
            );
        }
        let available = prototype
            .entities
            .iter()
            .map(|entity| entity.id)
            .collect::<BTreeSet<_>>();
        let mut positions = BTreeMap::new();
        let mut identities = BTreeMap::new();
        for &index in &indices {
            let entity = &scene.entities[index];
            let state = entity.blueprint_instance.as_ref().unwrap();
            if state.class != class.id
                || !available.contains(&state.template_entity)
                || positions.insert(state.template_entity, index).is_some()
            {
                return Err(format!(
                    "Orphaned or conflicting linked template entity {}; instance data was preserved.",
                    state.template_entity
                ));
            }
            identities.insert(state.template_entity, entity.id);
        }
        for entity in &prototype.entities {
            if let std::collections::btree_map::Entry::Vacant(position) = positions.entry(entity.id)
            {
                let id = Uuid::new_v4();
                let mut added = entity.clone();
                added.id = id;
                position.insert(candidate.entities.len());
                identities.insert(entity.id, id);
                candidate.entities.push(added);
            }
        }
        let authored = candidate
            .entities
            .iter()
            .enumerate()
            .map(|(index, entity)| (entity.id, index))
            .collect::<BTreeMap<_, _>>();
        for mut entity in prototype.entities.clone() {
            let template_id = entity.id;
            let index = positions[&template_id];
            let state = scene
                .entities
                .get(index)
                .and_then(|entity| entity.blueprint_instance.clone())
                .unwrap_or_else(|| Instance {
                    class: class.id.clone(),
                    template_entity: template_id,
                    instance: root_id,
                    overrides: BTreeMap::new(),
                    parent: None,
                });
            entity.id = identities[&template_id];
            if let Some(component) = &mut entity.timeline {
                crate::timeline_scene::remap(component, &identities, true)?;
            }
            if let Some(component) = &mut entity.particle_effect {
                crate::particle_effect_scene::remap(component, &identities, true)?;
            }
            entity.parent = if template_id == template_root {
                scene.entities[root_index].parent
            } else {
                entity
                    .parent
                    .map(|parent| positions[&prototype.entities[parent].id])
            };
            if let Some(binding) = &mut entity.script {
                for reference in resolved
                    .references
                    .iter()
                    .filter(|reference| reference.owner == template_id)
                {
                    if let Some((name, _)) = binding
                        .member_ids
                        .iter()
                        .find(|(_, id)| **id == reference.member)
                    {
                        binding.properties.insert(
                            name.clone(),
                            Value::String(identities[&reference.target].to_string()),
                        );
                    }
                }
            }
            let mut item = TemplateEntity {
                entity,
                parent: None,
            };
            apply_override(
                &mut item,
                &EntityOverride {
                    members: state.overrides.clone(),
                    parent: None,
                },
            )?;
            if let Some(parent) = &state.parent {
                item.entity.parent = match parent {
                    ParentOverride::Root => None,
                    ParentOverride::Entity { entity } => Some(*authored.get(entity).ok_or(
                        "Instance parent override points to a missing entity; data was preserved.",
                    )?),
                };
            }
            if template_id == template_root
                && item
                    .entity
                    .script
                    .as_ref()
                    .and_then(|binding| registry.bound(binding))
                    .is_none_or(|bound| bound.id != class.id)
            {
                return Err("Linked instance root behaviour was replaced; unlink or restore it explicitly before refreshing.".into());
            }
            item.entity.blueprint_instance = Some(state);
            candidate.entities[index] = item.entity;
        }
    }
    candidate.validate()?;
    if candidate.entities != scene.entities {
        candidate.bake = None;
    }
    *scene = candidate;
    Ok(())
}

pub fn resource_ids(template: &ResolvedTemplate) -> BTreeSet<Uuid> {
    let mut ids = BTreeSet::new();
    for item in &template.entities {
        let e = &item.entity;
        ids.extend(e.material.texture);
        if let Some(sprite) = &e.sprite {
            ids.extend(sprite.texture);
        }
        if let Some(image) = &e.image {
            ids.extend(image.texture);
        }
        if let Some(audio) = &e.audio {
            ids.extend(audio.clip);
        }
        if let Some(mesh) = &e.editable_mesh {
            ids.insert(mesh.asset);
            for material in mesh.materials.values() {
                ids.extend(material.texture);
            }
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
    for item in &template.entities {
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
    fn binding() -> ScriptBinding {
        ScriptBinding {
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
        let mut registry = Registry::new();
        for class in [base, derived, unrelated] {
            registry.classes.insert(class.id.clone(), class);
        }
        let authored = ScriptBinding {
            name: "Base".into(),
            class_id: Some("base".into()),
            properties: BTreeMap::from([("health".into(), json!(80))]),
            member_ids: BTreeMap::from([("health".into(), "health-id".into())]),
            overrides: std::collections::BTreeSet::from(["health".into()]),
            ..Default::default()
        };
        let target = ScriptBinding {
            name: "Derived".into(),
            class_id: Some("derived".into()),
            ..Default::default()
        };
        let mut scene = Scene::default();
        scene.entities[1].script = Some(authored.clone());
        let template = capture(&scene, 1, &registry).unwrap();
        let resolved = resolve(&[&template]).unwrap();
        for destination in [authored.clone(), target.clone()] {
            let prototype = resolved.scene(destination.clone(), &registry).unwrap();
            let root = prototype
                .entities
                .iter()
                .find(|entity| entity.parent.is_none())
                .unwrap();
            assert_eq!(
                root.script.as_ref().unwrap().properties["health"],
                json!(80)
            );
            assert_eq!(root.script.as_ref().unwrap().class_id, destination.class_id);
            let placed = place(&mut scene, &resolved, destination, &registry, None).unwrap();
            assert_eq!(
                scene.entities[placed.root]
                    .script
                    .as_ref()
                    .unwrap()
                    .properties["health"],
                json!(80)
            );
        }
        let mut explicit = target.clone();
        explicit.properties.insert("health".into(), json!(42));
        let prototype = resolved.scene(explicit, &registry).unwrap();
        assert_eq!(
            prototype
                .entities
                .iter()
                .find(|entity| entity.parent.is_none())
                .unwrap()
                .script
                .as_ref()
                .unwrap()
                .properties["health"],
            json!(42)
        );
        let before = scene.clone();
        let wrong = ScriptBinding {
            name: "Other".into(),
            class_id: Some("other".into()),
            ..Default::default()
        };
        assert!(place(&mut scene, &resolved, wrong, &registry, None).is_err());
        assert_eq!(scene, before);
        let mut orphan = template.clone();
        orphan
            .entities
            .iter_mut()
            .find(|entity| entity.parent.is_none())
            .unwrap()
            .entity
            .script
            .as_mut()
            .unwrap()
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
        let root = base.entities[0].entity.id;
        let child = base.add_child(root, "Child");
        let mut derived = Template::default();
        derived
            .overrides
            .entry(child)
            .or_default()
            .members
            .insert("uq.entity.position.v1".into(), json!([1., 2., 3.]));
        derived.construction.push(ConstructionOp::Translate {
            entity: child,
            offset: [0.5, 0., 0.],
        });
        let resolved = resolve(&[&base, &derived]).unwrap();
        assert_eq!(
            resolved
                .entities
                .iter()
                .find(|item| item.entity.id == child)
                .unwrap()
                .entity
                .position,
            [1.5, 2., 3.]
        );
        assert_eq!(base.entities[1].entity.position, [0.; 3]);
        let mut scene = Scene::default();
        let original = scene.entities.len();
        let first = place(&mut scene, &resolved, binding(), &Registry::new(), Some(0)).unwrap();
        let second = place(&mut scene, &resolved, binding(), &Registry::new(), None).unwrap();
        assert_eq!(first.entities.len(), 2);
        assert_eq!(scene.entities.len(), original + 4);
        assert_ne!(first.identities[&root], second.identities[&root]);
        assert_eq!(scene.entities[first.root].parent, Some(0));
        assert_eq!(scene.entities[second.root].parent, None);
        assert_eq!(
            scene.entities[first.root].script.as_ref().unwrap().name,
            "BP_Test"
        );
        assert_eq!(
            resolved
                .scene(binding(), &Registry::new())
                .unwrap()
                .entities
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
        let root = base.entities[0].entity.id;
        let child = base.add_child(root, "Child");
        let mut derived = Template::default();
        derived.overrides.entry(root).or_default().parent =
            Some(ParentOverride::Entity { entity: child });
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
                &Registry::new(),
                Some(999)
            )
            .is_err()
        );
        assert_eq!(scene, original);
        assert!(base.remove_local(root).is_err());
        assert!(base.remove_local(Uuid::new_v4()).is_err());
        base.remove_local(child).unwrap();
        assert_eq!(base.entities.len(), 1);
    }
    #[test]
    fn construction_is_bounded_quantized_and_validated() {
        let mut template = Template::root("Root");
        let root = template.entities[0].entity.id;
        template.construction.push(ConstructionOp::Translate {
            entity: root,
            offset: [0.0001, 0.0002, -0.0002],
        });
        assert_eq!(
            resolve(&[&template]).unwrap().entities[0].entity.position,
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
        scene.entities[2].parent = Some(1);
        let captured = capture(&scene, 1, &Registry::new()).unwrap();
        assert_eq!(captured.entities.len(), 2);
        assert!(
            captured
                .entities
                .iter()
                .all(|item| item.entity.parent.is_none()
                    && !scene.entities.iter().any(|e| e.id == item.entity.id))
        );
        let mut resolved = resolve(&[&captured]).unwrap();
        resolved.entities[0].entity.material.texture = Some(Uuid::new_v4());
        assert!(
            validate_resources(&resolved, &crate::assets::Index::default())
                .unwrap_err()
                .contains("Missing asset")
        );
    }
    #[test]
    fn template_entity_references_remap_by_identity_and_fail_without_data_loss() {
        use crate::reflection_schema as schema;
        let mut registry = Registry::new();
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
                parent: None,
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
                    value_type: Type::EntityRef { class: None },
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
        let root = template.entities[0].entity.id;
        let target = template.add_child(root, "Target");
        template.references.push(EntityReference {
            owner: root,
            member: "target-property".into(),
            target,
        });
        let timeline_slot = Uuid::new_v4();
        template.entities[0].entity.timeline = Some(crate::timeline_scene::Component {
            asset: Some(Uuid::new_v4()),
            bindings: BTreeMap::from([(timeline_slot, Some(target))]),
            ..Default::default()
        });
        let resolved = resolve(&[&template]).unwrap();
        let mut scene = Scene::default();
        let first = place(&mut scene, &resolved, binding(), &registry, None).unwrap();
        let second = place(&mut scene, &resolved, binding(), &registry, None).unwrap();
        assert_eq!(
            scene.entities[first.root]
                .script
                .as_ref()
                .unwrap()
                .properties["target"],
            json!(first.identities[&target].to_string())
        );
        assert_eq!(
            scene.entities[second.root]
                .script
                .as_ref()
                .unwrap()
                .properties["target"],
            json!(second.identities[&target].to_string())
        );
        let captured = capture(&scene, first.root, &registry).unwrap();
        assert_eq!(
            scene.entities[first.root]
                .timeline
                .as_ref()
                .unwrap()
                .bindings[&timeline_slot],
            Some(first.identities[&target])
        );
        assert_eq!(
            scene.entities[second.root]
                .timeline
                .as_ref()
                .unwrap()
                .bindings[&timeline_slot],
            Some(second.identities[&target])
        );
        let captured_target = captured
            .entities
            .iter()
            .find_map(|item| item.entity.timeline.as_ref())
            .unwrap()
            .bindings[&timeline_slot]
            .unwrap();
        assert!(
            captured
                .entities
                .iter()
                .any(|item| item.entity.id == captured_target)
        );
        let preserved = scene.clone();
        let external_timeline_target = scene.entities[0].id;
        scene.entities[first.root]
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
                .entities
                .iter()
                .any(|item| item.entity.id == captured.references[0].target)
        );
        let before = scene.clone();
        let mut invalid = resolved.clone();
        invalid.references[0].member = "removed-member".into();
        assert!(
            place(&mut scene, &invalid, binding(), &registry, None)
                .unwrap_err()
                .contains("migrate")
        );
        assert_eq!(scene, before);
        let external = scene.entities[0].id.to_string();
        scene.entities[first.root]
            .script
            .as_mut()
            .unwrap()
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
            parent: None,
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
        let root = asset.template.entities[0].entity.id;
        let slot = Uuid::new_v4();
        asset.template.entities[0].entity.timeline = Some(crate::timeline_scene::Component {
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
        let mut registry = Registry::new();
        registry.classes.insert(native.id.clone(), native);
        registry.classes.insert(visual.id.clone(), visual);
        let mut files = vec![AssetFile::file("BP_Test.epokbp".into(), asset)];
        let class = files[0].asset.id.clone();
        let resolved = resolve_assets(&files, &registry, &class).unwrap();
        let binding = ScriptBinding {
            name: "BP_Test".into(),
            class_id: Some(class.clone()),
            ..Default::default()
        };
        let mut scene = Scene::default();
        let first = place(&mut scene, &resolved, binding.clone(), &registry, None).unwrap();
        let second = place(&mut scene, &resolved, binding, &registry, None).unwrap();
        let original_audio = crate::audio::selection_signature(&scene, None);
        let before = scene.entities[first.root].clone();
        scene.entities[first.root].position = [9., 0., 0.];
        record_overrides(&before, &mut scene.entities[first.root]);
        assert_eq!(
            original_audio,
            crate::audio::selection_signature(&scene, None)
        );
        files[0].asset.template.entities[0].entity.position = [2., 3., 4.];
        let child = files[0].asset.template.add_child(root, "New Child");
        refresh_instances(&mut scene, &files, &registry).unwrap();
        assert_eq!(
            scene.entities[first.root]
                .timeline
                .as_ref()
                .unwrap()
                .bindings[&slot],
            Some(first.identities[&root])
        );
        assert_eq!(
            scene.entities[second.root]
                .timeline
                .as_ref()
                .unwrap()
                .bindings[&slot],
            Some(second.identities[&root])
        );
        assert_eq!(scene.entities[first.root].position, [9., 0., 0.]);
        assert_eq!(scene.entities[second.root].position, [2., 3., 4.]);
        assert_eq!(
            scene
                .entities
                .iter()
                .filter(|entity| entity
                    .blueprint_instance
                    .as_ref()
                    .is_some_and(|state| state.template_entity == child))
                .count(),
            2
        );
        assert_eq!(scene.entities[first.root].id, first.identities[&root]);
        // Reset explicitly removes the override; simply reading equal values does not.
        scene.entities[first.root]
            .blueprint_instance
            .as_mut()
            .unwrap()
            .overrides
            .remove("uq.entity.position.v1");
        refresh_instances(&mut scene, &files, &registry).unwrap();
        assert_eq!(scene.entities[first.root].position, [2., 3., 4.]);
        let clip = Uuid::new_v4();
        let before_audio = crate::audio::selection_signature(&scene, None);
        scene.entities[first.root]
            .blueprint_instance
            .as_mut()
            .unwrap()
            .overrides
            .insert(
                "uq.component.audio.v1".into(),
                json!({"clip": clip, "volume": 0.5}),
            );
        assert_ne!(
            before_audio,
            crate::audio::selection_signature(&scene, None)
        );
        refresh_instances(&mut scene, &files, &registry).unwrap();
        assert_eq!(crate::audio::clip_ids(&scene), vec![clip]);
        let selected = crate::audio::selection_signature(&scene, None);
        scene.entities[first.root]
            .blueprint_instance
            .as_mut()
            .unwrap()
            .overrides
            .insert(
                "uq.component.audio.v1".into(),
                json!({"clip": clip, "volume": 0.25}),
            );
        refresh_instances(&mut scene, &files, &registry).unwrap();
        assert_eq!(selected, crate::audio::selection_signature(&scene, None));
        scene.entities[first.root]
            .blueprint_instance
            .as_mut()
            .unwrap()
            .overrides
            .insert("uq.component.audio.v1".into(), json!(null));
        assert_ne!(selected, crate::audio::selection_signature(&scene, None));
        refresh_instances(&mut scene, &files, &registry).unwrap();
        assert!(crate::audio::clip_ids(&scene).is_empty());
        let valid = crate::audio::selection_signature(&scene, None);
        scene.entities[first.root]
            .blueprint_instance
            .as_mut()
            .unwrap()
            .overrides
            .insert("future.component.audio.v2".into(), json!({"clip": clip}));
        assert_ne!(valid, crate::audio::selection_signature(&scene, None));
        let preserved = scene.clone();
        assert!(
            refresh_instances(&mut scene, &files, &registry)
                .unwrap_err()
                .contains("Unknown template member")
        );
        assert_eq!(scene, preserved);
        scene.entities[first.root]
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
                .contains("Orphaned")
        );
        assert_eq!(scene, before);
    }
}
