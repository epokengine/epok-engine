//! Resolve persistent authoring references before dispatching start, never by name.
use crate::{blueprint::Registry, reflection_schema::Type, scene::Scene};
use serde_json::Value;
use sha2::{Digest, Sha256};

/// The graph literals consumed by shared resource cooking, including literals
/// on input pins. Observation reuses this traversal without resolving assets.
pub fn graph_resources(
    asset: &crate::blueprint_asset::BlueprintAsset,
) -> impl Iterator<Item = (&Type, &Value)> {
    asset
        .functions
        .iter()
        .flat_map(|graph| graph.compilation_nodes())
        .flat_map(|node| {
            let literal = match &node.kind {
                crate::blueprint_asset::NodeKind::Literal { value_type, value } => {
                    Some((value_type, value))
                }
                _ => None,
            };
            literal
                .into_iter()
                .chain(node.inputs.values().filter_map(|input| match input {
                    crate::blueprint_asset::Input::Literal { value_type, value } => {
                        Some((value_type, value))
                    }
                    _ => None,
                }))
        })
        .filter(|(ty, _)| matches!(ty, Type::AssetRef { .. }))
}

/// Add non-component asset references to normal resource cooking. They are
/// reachable even when no authored renderer/audio source currently uses them.
pub fn resources(
    files: &[crate::blueprint_asset::AssetFile],
    scenes: &[Scene],
    registry: &Registry,
    index: &crate::assets::Index,
    extra: &[(Type, Value)],
) -> Result<Scene, String> {
    let mut ids = std::collections::BTreeMap::new();
    let mut add = |ty: &Type, value: &Value| -> Result<(), String> {
        if let Type::AssetRef { kind } = ty {
            if value.is_null() {
                return Ok(());
            }
            let id = value
                .as_str()
                .and_then(|id| uuid::Uuid::parse_str(id).ok())
                .ok_or("AssetRef must be a UUID")?;
            let record = index.resolve(id)?;
            let expected = crate::assets::Kind::runtime_reference(kind)?;
            if !expected.accepts_runtime(&record.meta.kind) {
                return Err(format!(
                    "Asset {id} is {:?}, expected {kind}",
                    record.meta.kind
                ));
            }
            ids.insert(id, expected);
        }
        Ok(())
    };
    for (ty, value) in extra {
        add(ty, value)?;
    }
    for class in registry.classes.values() {
        for property in registry.properties(&class.cpp_name) {
            add(&property.value_type, &property.default)?;
        }
    }
    for file in files {
        for (ty, value) in graph_resources(&file.asset) {
            add(ty, value)?;
        }
    }
    for scene in scenes {
        for entity in &scene.entities {
            if let Some(binding) = &entity.script
                && let Some(class) = registry.bound(binding)
            {
                for property in registry.properties(&class.cpp_name) {
                    add(
                        &property.value_type,
                        binding
                            .properties
                            .get(&property.name)
                            .unwrap_or(&property.default),
                    )?;
                }
            }
        }
    }
    let mut result = Scene::default();
    result.entities.clear();
    result.textures.clear();
    for (id, kind) in ids {
        let mut entity = crate::scene::Entity::cube(format!("Resource_{id}"));
        entity.kind = "Empty".into();
        match kind {
            crate::assets::Kind::Texture => entity.material.texture = Some(id),
            crate::assets::Kind::AudioClip | crate::assets::Kind::MusicSequence => {
                entity.audio = Some(crate::audio::AudioSource {
                    clip: Some(id),
                    play_on_start: false,
                    ..Default::default()
                })
            }
            _ => {
                return Err(format!(
                    "Blueprint runtime AssetRef<{kind:?}> is not supported; use Texture or AudioClip"
                ));
            }
        }
        result.entities.push(entity);
    }
    crate::texture::resolve(&mut result, index)?;
    crate::audio::validate_assets(&result, index)?;
    Ok(result)
}
pub fn asset_table(resources: &Scene) -> Result<String, String> {
    let textures = crate::texture::ids(resources);
    let audio = crate::audio::clip_ids(resources);
    let mut ids = std::collections::BTreeSet::new();
    let mut entries = vec![];
    for (kind, list) in [(1, &textures), (2, &audio)] {
        for (index, id) in list.iter().enumerate() {
            let compact = compact_id(&id.to_string());
            if compact == 0 || !ids.insert(compact) {
                return Err("Compact runtime asset identity collision".into());
            }
            entries.push(format!("{{UINT64_C({compact}),{kind},{index}}}"));
        }
    }
    let count = entries.len();
    if entries.is_empty() {
        entries.push("{}".into());
    }
    Ok(format!(
        "namespace epok::bp {{struct AssetInfo{{uint64_t id;uint32_t kind;int index;}};inline const AssetInfo assets[]={{{}}};inline int asset_index(uint64_t id,uint32_t kind){{for(size_t i=0;i<{count};++i)if(assets[i].id==id&&assets[i].kind==kind)return assets[i].index;return -1;}}}}\n",
        entries.join(",")
    ))
}

pub fn compact_id(id: &str) -> u64 {
    u64::from_le_bytes(Sha256::digest(id.as_bytes())[..8].try_into().unwrap())
}
pub fn inspector(
    ui: &imgui::Ui,
    label: &str,
    value: &mut Value,
    ty: &Type,
    scene: &Scene,
    registry: &Registry,
    index: &crate::assets::Index,
) -> bool {
    let choices: Vec<(String, String)> = match ty {
        Type::EntityRef { class: base } => scene
            .entities
            .iter()
            .enumerate()
            .filter(|(_, entity)| {
                base.as_ref().is_none_or(|base| {
                    entity
                        .script
                        .as_ref()
                        .and_then(|binding| registry.bound(binding))
                        .is_some_and(|class| class_is_a(registry, &class.id, base))
                })
            })
            .map(|(position, entity)| {
                (
                    entity.id.to_string(),
                    format!("{} [{}]", entity.name, position),
                )
            })
            .collect(),
        Type::ClassRef { base } => registry
            .classes
            .values()
            .filter(|class| class_is_a(registry, &class.id, base))
            .map(|class| (class.id.clone(), class.cpp_name.clone()))
            .collect(),
        Type::AssetRef { kind } => index
            .assets
            .iter()
            .filter_map(|(id, records)| {
                let [record] = records.as_slice() else {
                    return None;
                };
                let expected: crate::assets::Kind =
                    serde_json::from_value(Value::String(kind.clone())).ok()?;
                (record.meta.kind == expected).then(|| {
                    (
                        id.to_string(),
                        record
                            .path
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .into_owned(),
                    )
                })
            })
            .collect(),
        _ => return crate::script_values::inspector(ui, label, value, ty),
    };
    let current = value.as_str();
    let preview = if value.is_null() {
        "None".to_owned()
    } else {
        choices
            .iter()
            .find(|(id, _)| Some(id.as_str()) == current)
            .map(|(_, name)| name.clone())
            .unwrap_or_else(|| format!("Missing / incompatible: {}", value))
    };
    if let Some(_combo) = ui.begin_combo(label, &preview) {
        if ui
            .selectable_config("None")
            .selected(value.is_null())
            .build()
        {
            *value = Value::Null;
            return true;
        }
        for (id, name) in choices {
            if ui
                .selectable_config(format!("{name}##{id}"))
                .selected(Some(id.as_str()) == current)
                .build()
            {
                *value = Value::String(id);
                return true;
            }
        }
    }
    if ui.is_item_hovered() {
        ui.tooltip_text(format!("{}\n{}", ty.label(), value));
    }
    false
}
pub fn class_is_a(registry: &Registry, class: &str, base: &str) -> bool {
    registry.classes.get(class).is_some_and(|class| {
        registry
            .ancestry(&class.cpp_name)
            .iter()
            .any(|c| c.id == base)
    })
}
/// Called only for a deliberate duplicate operation; never guess reference intent from strings.
pub fn remap_duplicate(
    scene: &mut Scene,
    first: usize,
    identities: &std::collections::BTreeMap<uuid::Uuid, uuid::Uuid>,
    registry: &Registry,
) {
    for entity in &mut scene.entities[first..] {
        if let Some(component) = &mut entity.timeline {
            let _ = crate::timeline_scene::remap(component, identities, false);
        }
        if let Some(binding) = &mut entity.script {
            let Some(class) = registry.bound(binding) else {
                continue;
            };
            let names = registry
                .properties(&class.cpp_name)
                .into_iter()
                .filter(|p| matches!(p.value_type, Type::EntityRef { .. }))
                .map(|p| p.name.clone())
                .collect::<Vec<_>>();
            for name in names {
                if let Some(value) = binding.properties.get_mut(&name)
                    && let Some(id) = value
                        .as_str()
                        .and_then(|value| uuid::Uuid::parse_str(value).ok())
                    && let Some(replacement) = identities.get(&id)
                {
                    *value = Value::String(replacement.to_string());
                }
            }
        }
    }
}
pub fn assignment(
    target: &str,
    value: &Value,
    ty: &Type,
    scene: &Scene,
    registry: &Registry,
) -> Result<String, String> {
    match ty {
        Type::EntityRef { class } if !value.is_null() => {
            let id = value
                .as_str()
                .and_then(|id| uuid::Uuid::parse_str(id).ok())
                .ok_or("Invalid entity reference UUID")?;
            let (index,entity)=scene.entities.iter().enumerate().find(|(_,entity)|entity.id==id)
                .ok_or_else(||format!("{target}: referenced entity {id} is missing; reference preserved for repair"))?;
            if let Some(base) = class {
                let class = entity
                    .script
                    .as_ref()
                    .and_then(|binding| registry.bound(binding))
                    .ok_or_else(|| {
                        format!("{target}: referenced entity has no compatible behaviour")
                    })?;
                if !class_is_a(registry, &class.id, base) {
                    return Err(format!(
                        "{target}: entity class {} is not derived from {base}",
                        class.cpp_name
                    ));
                }
            }
            Ok(format!("{target} = epok::handle(&objects[{index}]);\n"))
        }
        Type::ClassRef { base } if !value.is_null() => {
            let id = value
                .as_str()
                .ok_or("Class reference must contain a stable class ID")?;
            if !class_is_a(registry, id, base) {
                return Err(format!(
                    "{target}: missing or incompatible class reference {id}, expected {base}"
                ));
            }
            crate::script_values::assignment(target, value, ty)
        }
        _ => crate::script_values::assignment(target, value, ty),
    }
}

/// Why one class identity now resolves to another. The Blueprint compiler (P5)
/// decides which reasons it applies to which reference kind; the table itself
/// only records what the document says.
#[allow(dead_code)] // P4 derives the table; the Blueprint compiler consumes it in P5.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RedirectReason {
    /// A `Behaviour` bound to a legacy entity is hosted by
    /// `epok::LegacyBehaviourComponent` on the migrated actor, so a reference to
    /// the behaviour class now reaches it through that component.
    BehaviourHostedByLegacyComponent,
    /// A legacy Blueprint class placed as an instance is the migrated actor's
    /// own class. The identity is unchanged but the family is not, so
    /// `Spawn`/`IsA`/`ClassRef` consumers must re-resolve it.
    BlueprintInstanceIsActorClass,
}
impl RedirectReason {
    #[allow(dead_code)] // P4 derives the table; the Blueprint compiler consumes it in P5.
    pub fn label(self) -> &'static str {
        match self {
            Self::BehaviourHostedByLegacyComponent => "behaviour-hosted-by-legacy-component",
            Self::BlueprintInstanceIsActorClass => "blueprint-instance-is-actor-class",
        }
    }
}

/// One legacy class identity and the identity it resolves to in the actor model
/// (design.md section 7: "old Blueprint classes used by Spawn/IsA/ClassRef →
/// redirect table").
#[allow(dead_code)] // P4 derives the table; the Blueprint compiler consumes it in P5.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Redirect {
    pub from: String,
    pub to: String,
    pub reason: RedirectReason,
}

/// The redirects a scene implies, sorted by `from` and free of duplicates.
///
/// P4 only derives the table; the Blueprint compiler consumes it in P5 when it
/// rewrites `ClassRef` literals, `IsA` checks and `Spawn` targets. Nothing here
/// rewrites the scene.
#[allow(dead_code)] // P4 derives the table; the Blueprint compiler consumes it in P5.
pub fn redirects(scene: &Scene) -> Vec<Redirect> {
    let mut table: std::collections::BTreeMap<String, Redirect> = Default::default();
    for entity in &scene.entities {
        if let Some(class) = entity.script.as_ref().and_then(|b| b.class_id.as_deref()) {
            table.entry(class.to_owned()).or_insert_with(|| Redirect {
                from: class.to_owned(),
                to: crate::object_model::LEGACY_BEHAVIOUR_COMPONENT_ID.to_owned(),
                reason: RedirectReason::BehaviourHostedByLegacyComponent,
            });
        }
        if let Some(instance) = &entity.blueprint_instance {
            table
                .entry(instance.class.clone())
                .or_insert_with(|| Redirect {
                    from: instance.class.clone(),
                    to: instance.class.clone(),
                    reason: RedirectReason::BlueprintInstanceIsActorClass,
                });
        }
    }
    table.into_values().collect()
}

/// Follows `table` from `id` to the identity it finally resolves to. Unknown ids
/// pass through unchanged and a cyclic table stops at the entry it re-enters, so
/// a damaged table can never hang the compiler.
#[allow(dead_code)] // P4 derives the table; the Blueprint compiler consumes it in P5.
pub fn resolve_class_id(id: &str, table: &[Redirect]) -> String {
    let mut current = id.to_owned();
    let mut seen = std::collections::BTreeSet::new();
    while seen.insert(current.clone()) {
        let Some(redirect) = table.iter().find(|r| r.from == current) else {
            break;
        };
        if redirect.to == current {
            break;
        }
        current = redirect.to.clone();
    }
    current
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn legacy_class_redirects_are_derived_from_the_scene_and_resolve_transitively() {
        let mut scene = Scene::default();
        assert!(redirects(&scene).is_empty());
        scene.entities[1].script = Some(crate::scene::ScriptBinding {
            name: "Spinner".into(),
            class_id: Some("class-spinner".into()),
            ..Default::default()
        });
        scene.entities[2].blueprint_instance = Some(crate::blueprint_templates::Instance {
            class: "class-door".into(),
            template_entity: uuid::Uuid::nil(),
            instance: scene.entities[2].id,
            overrides: Default::default(),
            parent: None,
        });
        let table = redirects(&scene);
        assert_eq!(
            table.iter().map(|r| r.from.as_str()).collect::<Vec<_>>(),
            ["class-door", "class-spinner"]
        );
        assert_eq!(
            resolve_class_id("class-spinner", &table),
            crate::object_model::LEGACY_BEHAVIOUR_COMPONENT_ID
        );
        // An identity-preserving entry and an unknown id both pass through.
        assert_eq!(resolve_class_id("class-door", &table), "class-door");
        assert_eq!(resolve_class_id("class-other", &table), "class-other");
        // Chains are followed, and a cycle terminates instead of hanging.
        let chained = vec![
            Redirect {
                from: "a".into(),
                to: "b".into(),
                reason: RedirectReason::BlueprintInstanceIsActorClass,
            },
            Redirect {
                from: "b".into(),
                to: "c".into(),
                reason: RedirectReason::BlueprintInstanceIsActorClass,
            },
            Redirect {
                from: "c".into(),
                to: "a".into(),
                reason: RedirectReason::BlueprintInstanceIsActorClass,
            },
        ];
        assert_eq!(resolve_class_id("a", &chained), "a");
        assert_eq!(resolve_class_id("b", &chained[..2]), "c");
    }
    #[test]
    fn entity_references_survive_reordering_and_fail_instead_of_binding_by_name() {
        let mut scene = Scene::default();
        let id = scene.entities[1].id;
        let ty = Type::EntityRef { class: None };
        let value = serde_json::json!(id.to_string());
        let registry = Registry::new();
        assert!(
            assignment("target", &value, &ty, &scene, &registry)
                .unwrap()
                .contains("objects[1]")
        );
        scene.entities.swap(1, 2);
        assert!(
            assignment("target", &value, &ty, &scene, &registry)
                .unwrap()
                .contains("objects[2]")
        );
        scene.entities.remove(2);
        assert!(
            assignment("target", &value, &ty, &scene, &registry)
                .unwrap_err()
                .contains("preserved")
        );
        assert!(
            assignment("target", &Value::Null, &ty, &scene, &registry)
                .unwrap()
                .contains("EntityHandle{}")
        );
    }
}
