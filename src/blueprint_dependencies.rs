//! Dependency footprints of successfully generated Blueprints. Resolution and
//! validation remain owned by the existing registry, compiler and asset loaders.
use crate::{
    artifact_dependencies::{self, Graph},
    blueprint::Registry,
    blueprint_asset::{AssetFile, Builtin, NodeKind, PlaybackCondition},
    blueprint_compile::Compilation,
    reflection_schema as schema,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

fn hash(value: impl serde::Serialize) -> String {
    crate::assets::hash(&serde_json::to_vec(&value).expect("Serializable dependency"))
}

/// Consumer-specific exposure matters: BlueprintCallable and timeline exposure
/// are separate contracts on the same authoritative reflected function ID.
pub fn reflected(key: &str, registry: &Registry) -> Option<String> {
    let (kind, tail) = key.split_once(':')?;
    let value = match kind {
        "blueprint-class" => {
            let c = registry.classes.get(tail)?;
            serde_json::json!([
                c.id,
                c.cpp_name,
                c.parent,
                c.provider,
                c.backend,
                c.abstract_class,
                c.final_class,
                c.blueprintable,
                c.timeline_component
            ])
        }
        "blueprint-layout" => {
            let c = registry.classes.get(tail)?;
            // Generated debugger accessors cover all inherited properties.
            let properties = c
                .properties
                .iter()
                .map(|p| {
                    (
                        &p.id,
                        serde_json::json!([p.id, p.name, p.value_type, p.default, p.editable]),
                    )
                })
                .collect::<BTreeMap<_, _>>();
            serde_json::json!(properties)
        }
        "blueprint-function" => {
            let (class, member) = tail.split_once(':')?;
            let c = registry.classes.get(class)?;
            let f = registry
                .ancestry(&c.cpp_name)
                .into_iter()
                .rev()
                .flat_map(|c| &c.functions)
                .find(|f| f.id == member || f.overrides.iter().any(|id| id == member))?;
            serde_json::json!([
                f.id,
                f.name,
                f.parameters,
                f.returns,
                f.callable,
                f.event,
                f.pure,
                f.abstract_method,
                f.final_method,
                f.access,
                f.overrides
            ])
        }
        _ => return None,
    };
    Some(hash(value))
}

/// Membership of the existing compiler input list, including duplicate counts.
/// Paths/layout do not change persistent class identity or resource reachability.
pub fn source_set(files: &[AssetFile]) -> String {
    let mut ids = files
        .iter()
        .map(|file| file.asset.id.as_str())
        .collect::<Vec<_>>();
    ids.sort_unstable();
    hash(ids)
}

/// Recheck current authored sources without recompiling or trusting cached code.
/// Build/export certification uses this even when no editor watcher is running.
pub fn observe_current(root: &Path) -> Result<(), String> {
    let files = crate::blueprint_asset::load_all(root).inspect_err(|error| {
        let _ = invalidate_all(root, error);
    })?;
    let needs_types = !files.is_empty()
        && artifact_dependencies::Graph::load(root)?
            .nodes
            .values()
            .any(|node| {
                node.dependencies
                    .iter()
                    .any(|key| key.starts_with("blueprint-audio:"))
            });
    if !needs_types {
        return observe_sources(root, &files, None);
    }
    let native = crate::scripts::native_catalog(root)
        .and_then(|scripts| crate::blueprint::native_registry(root, &scripts));
    observe_sources(root, &files, native.as_ref().ok())?;
    native.map(|_| ())
}

pub fn observe_sources(
    root: &Path,
    files: &[AssetFile],
    native: Option<&Registry>,
) -> Result<(), String> {
    // Share the compiler's declaration pass. A failed pass grants no type-based
    // exclusions; raw defaults remain dependencies and source errors still cook-fail.
    let declarations = native
        .and_then(|native| crate::blueprint_compile::declaration_registry(native, files).ok());
    // The compiler still owns identity validation. Observation must not certify
    // whichever duplicate happened to appear last in the existing source list.
    let mut identities = BTreeMap::new();
    for file in files {
        *identities.entry(file.asset.id.as_str()).or_insert(0usize) += 1;
    }
    artifact_dependencies::transaction(root, |graph| {
        graph.publish("blueprint-sources", source_set(files), BTreeSet::new());
        let current = files
            .iter()
            .filter(|file| identities[file.asset.id.as_str()] == 1)
            .flat_map(|f| {
                [
                    (format!("blueprint:{}", f.asset.id), f.asset.semantic_hash()),
                    (
                        format!("blueprint-audio:{}", f.asset.id),
                        crate::audio::blueprint_selection_signature(f, declarations.as_ref()),
                    ),
                ]
            })
            .collect::<BTreeMap<_, _>>();
        let removed = graph
            .nodes
            .keys()
            .filter(|key| {
                (key.starts_with("blueprint:") || key.starts_with("blueprint-audio:"))
                    && !current.contains_key(*key)
            })
            .cloned()
            .collect::<Vec<_>>();
        for key in removed {
            graph.invalidate(
                &key,
                "Blueprint source was removed or has duplicate identity",
            );
        }
        for (key, signature) in current {
            graph.publish(&key, signature, BTreeSet::new());
        }
    })?;
    Ok(())
}

pub fn observe_reflection(root: &Path, registry: &Registry) -> Result<(), String> {
    artifact_dependencies::transaction(root, |graph| {
        let keys = graph
            .nodes
            .keys()
            .filter(|key| {
                key.starts_with("blueprint-class:")
                    || key.starts_with("blueprint-layout:")
                    || key.starts_with("blueprint-function:")
            })
            .cloned()
            .collect::<Vec<_>>();
        for key in keys {
            match reflected(&key, registry) {
                Some(signature) => graph.publish(&key, signature, BTreeSet::new()),
                None => graph.invalidate(
                    &key,
                    "Reflected Blueprint dependency was removed or is incompatible",
                ),
            }
        }
    })?;
    Ok(())
}

pub fn invalidate(root: &Path, files: &[AssetFile], reason: &str) -> Result<(), String> {
    artifact_dependencies::transaction(root, |graph| {
        for file in files {
            graph.invalidate(&format!("blueprint:{}", file.asset.id), reason);
            graph.invalidate(&format!("generated-blueprint:{}", file.asset.id), reason);
        }
    })?;
    Ok(())
}

pub fn invalidate_all(root: &Path, reason: &str) -> Result<(), String> {
    artifact_dependencies::transaction(root, |graph| {
        let keys = graph
            .nodes
            .keys()
            .filter(|key| {
                key.starts_with("blueprint:")
                    || key.starts_with("blueprint-audio:")
                    || key.starts_with("generated-blueprint:")
                    || *key == "blueprint-sources"
            })
            .cloned()
            .collect::<Vec<_>>();
        for key in keys {
            graph.invalidate(&key, reason);
        }
    })?;
    Ok(())
}

fn class_inputs(id: &str, registry: &Registry, inputs: &mut BTreeMap<String, String>) {
    let Some(class) = registry.classes.get(id) else {
        return;
    };
    for ancestor in registry.ancestry(&class.cpp_name) {
        let key = format!("blueprint-class:{}", ancestor.id);
        inputs.insert(key.clone(), reflected(&key, registry).expect("Known class"));
    }
}

fn reference_type(ty: &schema::Type, registry: &Registry, inputs: &mut BTreeMap<String, String>) {
    match ty {
        schema::Type::ClassRef { base }
        | schema::Type::EntityRef { class: Some(base) }
        | schema::Type::EffectLayerRef { class: base } => class_inputs(base, registry, inputs),
        _ => {}
    }
}

fn reference_value(
    ty: &schema::Type,
    value: &serde_json::Value,
    registry: &Registry,
    inputs: &mut BTreeMap<String, String>,
) {
    reference_type(ty, registry, inputs);
    // Literal/default validation also consumes the selected class's ancestry.
    // An unchanged base does not make a removed or reparented choice valid.
    if matches!(ty, schema::Type::ClassRef { .. })
        && let Some(id) = value.as_str()
    {
        class_inputs(id, registry, inputs);
    }
}

pub type Footprints = BTreeMap<String, (String, BTreeMap<String, String>)>;

/// Capture the exact snapshots that produced the generated code. Reading source
/// files again here could incorrectly certify old code against a newer revision.
pub fn capture(
    files: &[AssetFile],
    registry: &Registry,
    artifacts: &crate::script_backend::Artifacts,
    timelines: &[(std::path::PathBuf, crate::timeline::TimelineAsset)],
    effects: &[(std::path::PathBuf, crate::particle_effect::ParticleEffect)],
    resources: &BTreeMap<String, BTreeMap<String, String>>,
) -> Result<Footprints, String> {
    let sources = files
        .iter()
        .map(|f| (&f.asset.id, &f.asset))
        .collect::<BTreeMap<_, _>>();
    let mut footprints = BTreeMap::new();
    for file in files {
        let a = &file.asset;
        let mut inputs = BTreeMap::from([
            (format!("blueprint:{}", a.id), a.semantic_hash()),
            ("blueprint-compiler".into(), "1".into()),
            (
                "blueprint-schema".into(),
                crate::blueprint_asset::VERSION.to_string(),
            ),
            (
                "reflection-schema".into(),
                schema::SCHEMA_VERSION.to_string(),
            ),
        ]);
        if let Some(references) = resources.get(&a.id) {
            inputs.extend(references.clone());
        }
        class_inputs(&a.id, registry, &mut inputs);
        for class in registry.ancestry(&a.name) {
            let key = format!("blueprint-layout:{}", class.id);
            inputs.insert(
                key.clone(),
                reflected(&key, registry).expect("Known class layout"),
            );
            for property in &class.properties {
                reference_value(
                    &property.value_type,
                    &property.default,
                    registry,
                    &mut inputs,
                );
            }
            if let Some(parent) = sources.get(&class.id) {
                inputs.insert(format!("blueprint:{}", parent.id), parent.semantic_hash());
            }
        }
        for function in &a.functions {
            if let Some(id) = &function.override_id {
                let key = format!("blueprint-function:{}:{id}", a.parent);
                if let Some(signature) = reflected(&key, registry) {
                    inputs.insert(key, signature);
                }
            }
            reference_type(&function.returns, registry, &mut inputs);
            for p in &function.parameters {
                reference_type(&p.value_type, registry, &mut inputs);
            }
            for node in function.compilation_nodes() {
                for input in node.inputs.values() {
                    if let crate::blueprint_asset::Input::Literal { value_type, value } = input {
                        reference_value(value_type, value, registry, &mut inputs);
                    }
                }
                match &node.kind {
                    NodeKind::Literal { value_type, value } => {
                        reference_value(value_type, value, registry, &mut inputs);
                    }
                    NodeKind::Call { function } | NodeKind::CallOn { function, .. } => {
                        let target = if let NodeKind::CallOn { class, .. } = &node.kind {
                            class
                        } else {
                            &a.id
                        };
                        class_inputs(target, registry, &mut inputs);
                        let key = format!("blueprint-function:{target}:{function}");
                        let signature = reflected(&key, registry).ok_or_else(|| {
                            format!("Missing validated function dependency {key}")
                        })?;
                        inputs.insert(key, signature);
                        if let Some(callee) = sources.get(target) {
                            inputs
                                .insert(format!("blueprint:{}", callee.id), callee.semantic_hash());
                        }
                    }
                    NodeKind::Builtin {
                        operation:
                            Builtin::Spawn { class }
                            | Builtin::IsA { class }
                            | Builtin::Cast { class }
                            | Builtin::SpawnClass { base: class },
                    } => class_inputs(class, registry, &mut inputs),
                    NodeKind::Builtin {
                        operation: Builtin::PlayTimelineAsset { asset },
                    } => {
                        let source = &timelines
                            .iter()
                            .find(|(_, t)| t.id.to_string() == *asset)
                            .ok_or("Validated timeline disappeared during dependency capture")?
                            .1;
                        inputs.insert(format!("timeline:{asset}"), source.semantic_hash());
                    }
                    NodeKind::Builtin {
                        operation: Builtin::SpawnParticleEffect { asset },
                    } => {
                        let effect = &effects
                            .iter()
                            .find(|(_, e)| e.id.to_string() == *asset)
                            .ok_or("Validated effect disappeared during dependency capture")?
                            .1;
                        inputs.insert(format!("effect:{asset}"), effect.semantic_hash());
                        inputs.insert(
                            format!("timeline:{}", effect.timeline.id),
                            effect.timeline.semantic_hash(),
                        );
                    }
                    NodeKind::WaitPlayback {
                        condition:
                            PlaybackCondition::Marker { timeline, marker }
                            | PlaybackCondition::SubscribeMarker { timeline, marker },
                    } => {
                        let source = &timelines
                            .iter()
                            .find(|(_, t)| t.id.to_string() == *timeline)
                            .ok_or(
                                "Validated marker timeline disappeared during dependency capture",
                            )?
                            .1;
                        let marker_source = source
                            .markers
                            .iter()
                            .find(|m| m.id.to_string() == *marker)
                            .ok_or("Validated marker disappeared during dependency capture")?;
                        inputs.insert(format!("timeline:{timeline}"), source.semantic_hash());
                        inputs.insert(
                            format!("marker:{timeline}:{marker}"),
                            hash([marker_source.tick]),
                        );
                    }
                    _ => {}
                }
            }
        }
        let outputs = [
            format!("scripts/generated/{}.hpp", a.id),
            format!("blueprints/{}.epokdebug", a.id),
        ]
        .into_iter()
        .map(|path| {
            artifacts
                .files
                .get(Path::new(&path))
                .map(|data| (path, crate::assets::hash(data)))
                .ok_or_else(|| {
                    format!(
                        "Blueprint {} is missing a generated dependency artifact",
                        a.id
                    )
                })
        })
        .collect::<Result<BTreeMap<_, _>, String>>()?;
        footprints.insert(
            format!("generated-blueprint:{}", a.id),
            (hash(outputs), inputs),
        );
    }
    Ok(footprints)
}

/// Publish only persisted source compilations, never an unsaved canvas preview.
/// Generated code still has to be staged and compiled for the target.
pub fn record(root: &Path, compiled: &Compilation) -> Result<(), String> {
    observe_reflection(root, &compiled.registry)?;
    artifact_dependencies::transaction(root, |graph: &mut Graph| {
        // Observe every leaf before publishing consumers. A caller can refer to
        // a later file, and mutually calling classes need no graph-order trick.
        for (_, inputs) in compiled.footprints.values() {
            for (id, signature) in inputs {
                graph.publish(id, signature.clone(), BTreeSet::new());
            }
        }
        for (id, (signature, inputs)) in &compiled.footprints {
            graph.publish(id, signature.clone(), inputs.keys().cloned().collect());
        }
    })?;
    Ok(())
}
