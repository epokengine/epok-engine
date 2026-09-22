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
        for actor in &scene.actors {
            for (reference, values) in std::iter::once((&actor.class, &actor.properties))
                .chain(actor.components.iter().map(|c| (&c.class, &c.properties)))
            {
                if let Some(class) = reference
                    .class_id
                    .as_ref()
                    .and_then(|id| registry.classes.get(id))
                    .or_else(|| registry.named(&reference.name))
                {
                    for property in registry.properties(&class.cpp_name) {
                        add(
                            &property.value_type,
                            values.get(&property.name).unwrap_or(&property.default),
                        )?;
                    }
                }
            }
        }
    }
    let mut result = Scene::default();
    result.actors.clear();
    result.textures.clear();
    for (id, kind) in ids {
        let mut entity = crate::scene::Actor::cube(format!("Resource_{id}"));
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
        result.actors.push(entity);
    }
    crate::texture::resolve(&mut result, index)?;
    crate::hud::resolve_fonts(&mut result, index)?;
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
        Type::ObjectRef { class: base }
        | Type::ActorRef { class: base }
        | Type::ComponentRef { class: base } => {
            let compatible = |class: &crate::actor_document::ClassReference| {
                base.as_ref().is_none_or(|base| {
                    class
                        .class_id
                        .as_ref()
                        .is_some_and(|id| class_is_a(registry, id, base))
                })
            };
            let mut choices = Vec::new();
            for actor in &scene.actors {
                if !matches!(ty, Type::ComponentRef { .. }) && compatible(&actor.class) {
                    choices.push((actor.id.to_string(), actor.name.clone()));
                }
                if !matches!(ty, Type::ActorRef { .. }) {
                    for component in &actor.components {
                        if compatible(&component.class) {
                            choices.push((
                                component.id.to_string(),
                                format!("{} / {}", actor.name, component.name),
                            ));
                        }
                    }
                }
            }
            choices
        }
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
    // Serialized Blueprint references carry stable class ids, while Lua's
    // author-facing typed-reference constructors use readable C++ names. Both
    // spellings describe the same reflected class and must pass through the
    // same compatibility check before cooking an ObjectId.
    let class = registry
        .classes
        .get(class)
        .or_else(|| registry.named(class));
    let base = registry
        .classes
        .get(base)
        .or_else(|| registry.named(base))
        .map_or(base, |class| class.id.as_str());
    class.is_some_and(|class| {
        registry
            .ancestry(&class.cpp_name)
            .iter()
            .any(|candidate| candidate.id == base)
    })
}
/// Called only for a deliberate duplicate operation; never guess reference intent from strings.
pub fn remap_duplicate(
    scene: &mut Scene,
    first: usize,
    identities: &std::collections::BTreeMap<uuid::Uuid, uuid::Uuid>,
    registry: &Registry,
) {
    let _ = registry;
    for actor in &mut scene.actors[first..] {
        crate::actor_document::remap_actor(actor, identities);
    }
}
pub fn assignment(
    target: &str,
    value: &Value,
    ty: &Type,
    scene: &Scene,
    registry: &Registry,
) -> Result<String, String> {
    assignment_in(target, value, ty, scene, registry, None)
}

/// Bind a service after actor creation, outside an ActorTable apply callback.
/// `data` names the original scene-order ActorData pointer (including spawned slots).
pub fn data_slot_assignment(
    target: &str,
    value: &Value,
    ty: &Type,
    scene: &Scene,
    registry: &Registry,
    data: impl Fn(usize) -> String,
) -> Result<String, String> {
    assignment_in(target, value, ty, scene, registry, Some(&data))
}

fn assignment_in(
    target: &str,
    value: &Value,
    ty: &Type,
    scene: &Scene,
    registry: &Registry,
    data: Option<&dyn Fn(usize) -> String>,
) -> Result<String, String> {
    match ty {
        Type::ObjectRef { class } | Type::ActorRef { class } | Type::ComponentRef { class }
            if !value.is_null() =>
        {
            let id = value
                .as_str()
                .and_then(|v| uuid::Uuid::parse_str(v).ok())
                .ok_or("Invalid object reference UUID")?;
            let mut order = (0..scene.actors.len()).collect::<Vec<_>>();
            order.sort_by_key(|i| {
                let mut depth = 0;
                let mut parent = scene.actors[*i].logical_parent;
                while let Some(id) = parent {
                    depth += 1;
                    if depth > 64 {
                        break;
                    }
                    parent = scene
                        .actors
                        .iter()
                        .find(|a| a.id == id)
                        .and_then(|a| a.logical_parent);
                }
                depth
            });
            for (slot, index) in order.iter().enumerate() {
                let actor = &scene.actors[*index];
                let found = if actor.id == id && !matches!(ty, Type::ComponentRef { .. }) {
                    Some((
                        &actor.class,
                        data.map_or_else(
                            || format!("actors[{slot}]"),
                            |data| format!("epok::actor_for_slot({})", data(*index)),
                        ),
                    ))
                } else if !matches!(ty, Type::ActorRef { .. }) {
                    actor.components.iter().position(|c| c.id == id).map(|i| {
                        (
                            &actor.components[i].class,
                            data.map_or_else(
                                || format!("registry.resolve<epok::Actor>(actors[{slot}])->component_id({i})"),
                                |data| {
                                    let pointer = data(*index);
                                    format!("(({pointer}) && ({pointer})->owner ? ({pointer})->owner->component_id({i}) : epok::ObjectId{{}})")
                                },
                            ),
                        )
                    })
                } else {
                    None
                };
                if let Some((reference, expression)) = found {
                    if class.as_ref().is_some_and(|base| {
                        reference
                            .class_id
                            .as_ref()
                            .is_none_or(|id| !class_is_a(registry, id, base))
                    }) {
                        return Err(format!(
                            "{target}: incompatible referenced class {}",
                            reference.name
                        ));
                    }
                    return Ok(format!("{target} = {expression};\n"));
                }
            }
            Err(format!(
                "{target}: referenced Actor or ActorComponent {id} is missing or incompatible"
            ))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reflection_schema as schema;
    use std::path::PathBuf;

    fn class(id: &str, cpp_name: &str, parent: Option<&str>) -> schema::Class {
        schema::Class {
            id: id.into(),
            provider: schema::native_provider(),
            backend: schema::native_backend(),
            cpp_name: cpp_name.into(),
            parent: parent.map(str::to_owned),
            abstract_class: false,
            final_class: false,
            blueprintable: true,
            timeline_component: None,
            family: None,
            domain: None,
            placement: schema::Placement::default(),
            component: None,
            default_components: vec![],
            explicit_abstract: false,
            properties: vec![],
            functions: vec![],
            source: schema::Location {
                file: PathBuf::from("typed-reference-test.hpp"),
                line: 1,
                column: 1,
            },
        }
    }

    #[test]
    fn typed_reference_compatibility_accepts_stable_ids_and_cpp_names() {
        let mut registry = Registry::new();
        let base = class("base-id", "epok::Base", None);
        let child = class("child-id", "epok::Child", Some("base-id"));
        registry.classes.insert(base.id.clone(), base);
        registry.classes.insert(child.id.clone(), child);

        for derived in ["child-id", "epok::Child"] {
            for ancestor in ["base-id", "epok::Base"] {
                assert!(class_is_a(&registry, derived, ancestor));
            }
        }
        assert!(!class_is_a(&registry, "base-id", "child-id"));
        assert!(!class_is_a(&registry, "missing", "base-id"));
    }
}
