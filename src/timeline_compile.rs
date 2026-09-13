//! Host-only cooking and explicitly stale preview caches. Builds never consume a failed cache.
use crate::{
    blueprint::Registry,
    timeline::{self, Diagnostic, TimelineAsset},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompiledTrack {
    pub id: Uuid,
    pub slot: Uuid,
    pub property: String,
    pub class: String,
    pub field: String,
    pub priority: i16,
    pub restore: timeline::Restore,
    pub channels: Vec<Vec<(i32, i32)>>,
    pub value_type: crate::reflection_schema::Type,
    pub interpolation: timeline::Interpolation,
    pub blend: timeline::Blend,
    pub key_ids: Vec<Uuid>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Compiled {
    pub asset: Uuid,
    pub duration_ticks: i32,
    pub loop_mode: timeline::LoopMode,
    pub slots: Vec<timeline::Slot>,
    pub tracks: Vec<CompiledTrack>,
    pub markers: Vec<(i32, Uuid)>,
    pub events: Vec<CompiledEvent>,
    pub dependencies: BTreeMap<String, String>,
    pub signature: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompiledEvent {
    pub track: Uuid,
    pub key: Uuid,
    pub slot: Uuid,
    pub tick: i32,
    pub class: String,
    pub function: String,
    pub method: String,
    pub call: crate::reflection_schema::TimelineCall,
    pub arguments: Vec<CookedArgument>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CookedArgument {
    pub value_type: crate::reflection_schema::Type,
    pub lanes: [i32; 4],
    pub resource: Option<Uuid>,
    pub slot: Option<Uuid>,
}
/// Resolve only dependency IDs emitted by this compiler, through the existing
/// registry. These fingerprints deliberately exclude unrelated class members.
pub fn reflection_dependency(key: &str, registry: &Registry) -> Option<String> {
    if key == "reflection-schema" {
        return Some(crate::reflection_schema::SCHEMA_VERSION.to_string());
    }
    if key == "timeline-compiler" {
        return Some("5".into());
    }
    let (kind, tail) = key.split_once(':')?;
    let value = match kind {
        "class" => {
            let class = registry.classes.get(tail)?;
            serde_json::json!([
                class.id,
                class.cpp_name,
                class.parent,
                class.provider,
                class.backend,
                class.timeline_component
            ])
        }
        "property" => {
            let (class, property) = tail.split_once(':')?;
            let class = registry.classes.get(class)?;
            let property = registry
                .properties(&class.cpp_name)
                .into_iter()
                .find(|p| p.id == property)?;
            serde_json::json!([
                property.id,
                property.name,
                property.value_type,
                property.editable,
                property.timeline
            ])
        }
        "function" => {
            let (class, function) = tail.split_once(':')?;
            let class = registry.classes.get(class)?;
            let f = registry
                .ancestry(&class.cpp_name)
                .into_iter()
                .rev()
                .flat_map(|c| c.functions.iter())
                .find(|f| f.id == function || f.overrides.iter().any(|id| id == function))?;
            serde_json::json!([
                class.id,
                class.cpp_name,
                f.id,
                f.name,
                f.parameters,
                f.returns,
                f.timeline
            ])
        }
        _ => return None,
    };
    Some(crate::assets::hash(&serde_json::to_vec(&value).unwrap()))
}

/// Actor-side half of the `TimelineRequires=` restriction (actor-architecture P9).
///
/// `timeline::validate_components` answers the question for a legacy entity, by
/// looking at the entity's own component members. An actor has no such members:
/// its components are separate objects, so the same requirement is satisfied by
/// the component class reserved for it in design.md section 2 being present in
/// the actor's component set. Neither check replaces the other — a legacy entity
/// keeps the entity-member check, an actor gets this one — and the set of
/// requirements is unchanged, so no timeline that compiles today stops compiling.
///
/// Requirements with no reserved actor component (`PaletteAnimator`,
/// `ParticleEmitter`) are skipped here; they stay enforced on legacy entities
/// until those component classes exist.
// P9 states the actor-side rule ahead of the actor timeline binding that calls it
// (the editor still binds timelines to legacy entities), exactly as P1 introduced
// the object model ahead of its consumers.
#[allow(dead_code)]
pub fn verify_actor_requirements(
    adapter_class: &str,
    owner_class: &str,
    components: &[crate::object_model::ComponentSpec],
    registry: &Registry,
    model: &crate::object_model::Model,
) -> Result<(), Vec<crate::object_model::Diagnostic>> {
    let Some(class) = registry.classes.get(adapter_class) else {
        return Ok(());
    };
    let mut diagnostics = Vec::new();
    for requirement in registry
        .ancestry(&class.cpp_name)
        .iter()
        .filter_map(|class| class.timeline_component)
    {
        if let Err(diagnostic) =
            model.validate_timeline_requirement(owner_class, components, requirement)
        {
            diagnostics.push(diagnostic);
        }
    }
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

pub fn compile(asset: &TimelineAsset, registry: &Registry) -> Result<Compiled, Vec<Diagnostic>> {
    let errors = asset.validate(registry);
    if !errors.is_empty() {
        return Err(errors);
    }
    let mut dependencies = BTreeMap::from([
        (format!("timeline:{}", asset.id), asset.semantic_hash()),
        (
            "reflection-schema".into(),
            crate::reflection_schema::SCHEMA_VERSION.to_string(),
        ),
        ("timeline-compiler".into(), "5".into()),
    ]);
    for slot in &asset.slots {
        let id = slot.class_id().expect("validated slot class");
        for class in registry.ancestry(&registry.classes[id].cpp_name) {
            let key = format!("class:{}", class.id);
            dependencies.insert(key.clone(), reflection_dependency(&key, registry).unwrap());
        }
    }
    let mut tracks = vec![];
    for track in &asset.tracks {
        let slot = asset.slots.iter().find(|s| s.id == track.slot).unwrap();
        let class = slot.class_id().expect("validated slot class");
        let class = &registry.classes[class];
        let property = registry
            .properties(&class.cpp_name)
            .into_iter()
            .find(|p| p.id == track.property)
            .unwrap();
        let key = format!("property:{}:{}", class.id, property.id);
        dependencies.insert(key.clone(), reflection_dependency(&key, registry).unwrap());
        let mut keys = track
            .keys
            .iter()
            .map(|k| (k.tick, timeline::pack(&k.value, &track.value_type).unwrap()))
            .collect::<Vec<_>>();
        keys.sort_by_key(|k| k.0);
        let mut key_ids = track
            .keys
            .iter()
            .map(|k| (k.tick, k.id))
            .collect::<Vec<_>>();
        key_ids.sort();
        tracks.push(CompiledTrack {
            id: track.id,
            slot: track.slot,
            property: track.property.clone(),
            class: class.cpp_name.clone(),
            field: property.name.clone(),
            priority: track.priority,
            restore: track.restore,
            channels: (0..timeline::channels(&track.value_type))
                .map(|c| keys.iter().map(|(t, v)| (*t, v[c])).collect())
                .collect(),
            value_type: track.value_type.clone(),
            interpolation: track.interpolation,
            blend: track.blend,
            key_ids: key_ids.into_iter().map(|(_, id)| id).collect(),
        });
    }
    tracks.sort_by_key(|t| (t.priority, t.blend == timeline::Blend::Additive, t.id));
    let mut markers = asset
        .markers
        .iter()
        .map(|m| (m.tick, m.id))
        .collect::<Vec<_>>();
    markers.sort();
    let mut events = vec![];
    for track in &asset.events {
        let slot = asset.slots.iter().find(|s| s.id == track.slot).unwrap();
        let id = slot.class_id().expect("validated slot class");
        let class = &registry.classes[id];
        let f = registry
            .ancestry(&class.cpp_name)
            .into_iter()
            .rev()
            .flat_map(|c| c.functions.iter())
            .find(|f| f.id == track.function || f.overrides.contains(&track.function))
            .unwrap();
        let key = format!("function:{}:{}", class.id, track.function);
        dependencies.insert(key.clone(), reflection_dependency(&key, registry).unwrap());
        for key in &track.keys {
            events.push(CompiledEvent {
                track: track.id,
                key: key.id,
                slot: track.slot,
                tick: key.tick,
                class: class.cpp_name.clone(),
                function: track.function.clone(),
                method: f.name.clone(),
                call: f.timeline.unwrap(),
                arguments: f
                    .parameters
                    .iter()
                    .map(|p| {
                        let mut arg = CookedArgument {
                            value_type: p.value_type.clone(),
                            lanes: [0; 4],
                            resource: None,
                            slot: None,
                        };
                        match &key.arguments[&p.name] {
                            timeline::Argument::Slot { slot } => arg.slot = Some(*slot),
                            timeline::Argument::Literal { value_type, value } => {
                                if matches!(
                                    value_type,
                                    crate::reflection_schema::Type::AssetRef { .. }
                                ) {
                                    arg.resource =
                                        value.as_str().map(|id| Uuid::parse_str(id).unwrap());
                                } else {
                                    arg.lanes = timeline::pack(value, value_type).unwrap();
                                }
                            }
                        }
                        arg
                    })
                    .collect(),
            });
        }
    }
    events.sort_by_key(|e| (e.tick, e.key));
    let mut slots = asset.slots.clone();
    slots.sort_by_key(|s| s.id);
    let signature = crate::assets::hash(&serde_json::to_vec(&dependencies).unwrap());
    Ok(Compiled {
        asset: asset.id,
        duration_ticks: asset.duration_ticks,
        loop_mode: asset.loop_mode,
        slots,
        tracks,
        markers,
        events,
        dependencies,
        signature,
    })
}
impl Compiled {
    pub fn ordered_signals(&self) -> Vec<(i32, Uuid, bool, usize)> {
        let mut signals = self
            .markers
            .iter()
            .enumerate()
            .map(|(i, (tick, id))| (*tick, *id, false, i))
            .chain(
                self.events
                    .iter()
                    .enumerate()
                    .map(|(i, e)| (e.tick, e.key, true, i)),
            )
            .collect::<Vec<_>>();
        signals.sort();
        signals
    }
    fn resolve_resources(&mut self, index: &crate::assets::Index) -> Result<(), Vec<Diagnostic>> {
        let mut errors = vec![];
        for event in &self.events {
            for arg in &event.arguments {
                let Some(id) = arg.resource else { continue };
                let result = index.resolve(id).and_then(|record| {
                    let crate::reflection_schema::Type::AssetRef { kind } = &arg.value_type else {
                        unreachable!()
                    };
                    let expected: crate::assets::Kind =
                        serde_json::from_value(serde_json::Value::String(kind.clone()))
                            .map_err(|_| format!("Unsupported asset kind {kind}"))?;
                    if record.meta.kind != expected {
                        return Err(format!(
                            "Asset {id} is {:?}, expected {kind}",
                            record.meta.kind
                        ));
                    }
                    Ok(record)
                });
                match result {
                    Ok(record) => {
                        self.dependencies.insert(
                            format!("asset:{id}"),
                            crate::assets::cache_key(&record.meta),
                        );
                    }
                    Err(message) => errors.push(Diagnostic {
                        asset: self.asset,
                        item: event.key,
                        required: true,
                        message,
                    }),
                }
            }
        }
        if !errors.is_empty() {
            return Err(errors);
        }
        self.signature = crate::assets::hash(&serde_json::to_vec(&self.dependencies).unwrap());
        Ok(())
    }
    #[cfg(test)]
    pub fn sample(&self, tick: i32) -> Vec<(Uuid, i32)> {
        self.tracks
            .iter()
            .map(|t| {
                (
                    t.id,
                    crate::timeline_curve::sample_mode(
                        &t.channels[0],
                        tick.clamp(0, self.duration_ticks),
                        t.interpolation as u8,
                        matches!(t.value_type, crate::reflection_schema::Type::UInt32),
                    ),
                )
            })
            .collect()
    }
    pub fn sample_values(&self, tick: i32) -> Vec<(Uuid, [i32; 4])> {
        self.tracks
            .iter()
            .map(|t| {
                let mut out = [0; 4];
                for (i, keys) in t.channels.iter().enumerate() {
                    out[i] = crate::timeline_curve::sample_mode(
                        keys,
                        tick.clamp(0, self.duration_ticks),
                        t.interpolation as u8,
                        matches!(t.value_type, crate::reflection_schema::Type::UInt32),
                    );
                }
                (t.id, out)
            })
            .collect()
    }
    /// Emit immutable tables. `timeline_runtime` adds typed, checked binding
    /// accessors and the runtime director's asset definition.
    pub fn tables(&self) -> String {
        let stem = self.asset.simple();
        let mut out = format!(
            "#pragma once\n#include \"timeline.hpp\"\nnamespace epok::timeline::cooked::asset_{stem} {{\n"
        );
        for t in &self.tracks {
            for (i, keys) in t.channels.iter().enumerate() {
                let values = keys
                    .iter()
                    .map(|(tick, value)| format!("{{{tick},{value}}}"))
                    .collect::<Vec<_>>()
                    .join(",");
                out += &format!(
                    "inline constexpr Key track_{}_{i}[]={{{values}}};\n",
                    t.id.simple()
                );
            }
            let curves = t
                .channels
                .iter()
                .enumerate()
                .map(|(i, keys)| {
                    format!(
                        "{{track_{}_{i},{},Interpolation::{:?},{}}}",
                        t.id.simple(),
                        keys.len(),
                        t.interpolation,
                        matches!(t.value_type, crate::reflection_schema::Type::UInt32)
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            out += &format!(
                "inline constexpr Curve curves_{}[]={{{curves}}};\n",
                t.id.simple()
            );
        }
        for event in &self.events {
            if event.arguments.is_empty() {
                continue;
            }
            let args = event
                .arguments
                .iter()
                .map(|arg| {
                    let values = arg
                        .lanes
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(",");
                    let resource = arg
                        .resource
                        .map_or(0, |id| crate::blueprint_refs::compact_id(&id.to_string()));
                    let slot = arg.slot.map_or(-1, |id| {
                        self.slots.iter().position(|s| s.id == id).unwrap() as i32
                    });
                    format!("{{{{{values}}},UINT64_C({resource}),{slot}}}")
                })
                .collect::<Vec<_>>()
                .join(",");
            out += &format!(
                "inline constexpr Argument arguments_{}[]={{{args}}};\n",
                event.key.simple()
            );
        }
        let signals = self.ordered_signals();
        if !signals.is_empty() {
            let data = signals
                .iter()
                .map(|(tick, _, event, i)| format!("{{{tick},{i},{event}}}"))
                .collect::<Vec<_>>()
                .join(",");
            out += &format!("inline constexpr Signal signals_{stem}[]={{{data}}};\n");
        }
        if !self.markers.is_empty() {
            let markers = self
                .markers
                .iter()
                .enumerate()
                .map(|(i, (tick, _))| format!("{{{tick},{i}}}"))
                .collect::<Vec<_>>()
                .join(",");
            out += &format!("inline constexpr Marker markers_{stem}[]={{{markers}}};\n");
        }
        out + "}\n"
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PreviewCache {
    pub compiled: Option<Compiled>,
    pub stale: bool,
    pub diagnostics: Vec<Diagnostic>,
}
fn apply_stale_graph(
    root: &Path,
    graph: &crate::artifact_dependencies::Graph,
) -> Result<(), String> {
    for (key, node) in &graph.nodes {
        if !node.stale.is_empty()
            && let Some(id) = key
                .strip_prefix("cooked-timeline:")
                .and_then(|s| Uuid::parse_str(s).ok())
        {
            let reason = node
                .stale
                .iter()
                .map(|(id, reason)| format!("{id}: {reason}"))
                .collect::<Vec<_>>()
                .join("\n");
            invalidate_cache(root, id, &reason)?;
        }
    }
    Ok(())
}

/// Refresh reflected dependency observations even when the corresponding
/// Timeline window is closed. Deleted IDs retain their old consumer edges.
pub fn observe_reflection(root: &Path, registry: &Registry) -> Result<(), String> {
    // Import existing cache footprints when upgrading a project that predates
    // the graph. This is provenance migration, never a fallback for cooking.
    let mut previous = vec![];
    let folder = root.join(".epok/timelines");
    if folder.is_dir() {
        for entry in std::fs::read_dir(folder).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "json")
                && let Ok(bytes) = crate::assets::read_bounded(&path)
                && let Ok(cache) = serde_json::from_slice::<PreviewCache>(&bytes)
                && let Some(compiled) = &cache.compiled
                && path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .is_some_and(|s| s == compiled.asset.to_string())
            {
                previous.push(cache);
            }
        }
    }
    let graph = crate::artifact_dependencies::transaction(root, |graph| {
        for cache in previous {
            let compiled = cache.compiled.expect("Checked cache");
            let key = format!("cooked-timeline:{}", compiled.asset);
            if graph.nodes.contains_key(&key) {
                continue;
            }
            for (id, signature) in &compiled.dependencies {
                if !graph.nodes.contains_key(id) {
                    graph.publish(id, signature.clone(), Default::default());
                }
            }
            graph.publish(
                &key,
                compiled.signature,
                compiled.dependencies.keys().cloned().collect(),
            );
            if cache.stale
                || compiled
                    .dependencies
                    .iter()
                    .any(|(id, stamp)| graph.nodes[id].signature.as_ref() != Some(stamp))
            {
                graph.invalidate(&key, "Imported preview requires fresh validation");
            }
        }
        let keys = graph
            .nodes
            .keys()
            .filter(|key| {
                key.starts_with("class:")
                    || key.starts_with("property:")
                    || key.starts_with("function:")
                    || key.as_str() == "reflection-schema"
                    || key.as_str() == "timeline-compiler"
            })
            .cloned()
            .collect::<Vec<_>>();
        for key in keys {
            if let Some(signature) = reflection_dependency(&key, registry) {
                graph.publish(&key, signature, Default::default());
            } else {
                graph.invalidate(&key, "Reflected dependency was removed or is incompatible");
            }
        }
    })?;
    apply_stale_graph(root, &graph)
}

#[derive(Default)]
pub struct SourceWatch {
    // Editor observation history, not a resolver or persisted asset registry.
    // Keep raw source signatures independent of preview/build graph publication.
    previous: Option<BTreeMap<String, String>>,
}
impl SourceWatch {
    #[cfg(test)]
    pub fn poll(&mut self, root: &Path, target: &str) -> Result<(BTreeSet<Uuid>, bool), String> {
        self.accept(root, target, &timeline::inspect_playback(root)?)
    }
    pub fn accept(
        &mut self,
        root: &Path,
        target: &str,
        catalog: &timeline::PlaybackSources,
    ) -> Result<(BTreeSet<Uuid>, bool), String> {
        let (valid, sources, graph) = observe_source_catalog(root, catalog)?;
        let changed = self
            .previous
            .as_ref()
            .map(|previous| {
                previous
                    .keys()
                    .chain(sources.keys())
                    .filter(|key| previous.get(*key) != sources.get(*key))
                    .cloned()
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default();
        let affected = graph.depends_on_any(&format!("stage:{target}"), &changed);
        self.previous = Some(sources);
        Ok((valid, affected))
    }
}
pub fn observe_sources(root: &Path) -> Result<BTreeSet<Uuid>, String> {
    observe_source_snapshot(root).map(|(valid, _, _)| valid)
}
type SourceSnapshot = (
    BTreeSet<Uuid>,
    BTreeMap<String, String>,
    crate::artifact_dependencies::Graph,
);
fn observe_source_snapshot(root: &Path) -> Result<SourceSnapshot, String> {
    observe_source_catalog(root, &timeline::inspect_playback(root)?)
}
fn observe_source_catalog(
    root: &Path,
    catalog: &timeline::PlaybackSources,
) -> Result<SourceSnapshot, String> {
    fn timeline_inputs(sources: &mut BTreeMap<String, String>, asset: &TimelineAsset) {
        sources.insert(format!("timeline:{}", asset.id), asset.semantic_hash());
        sources.insert(
            format!("timeline-audio:{}", asset.id),
            crate::audio::timeline_selection_signature(asset),
        );
        for marker in &asset.markers {
            sources.insert(
                format!("marker:{}:{}", asset.id, marker.id),
                crate::assets::hash(&serde_json::to_vec(&[marker.tick]).unwrap()),
            );
        }
    }
    let mut sources = BTreeMap::new();
    let mut valid = BTreeSet::new();
    for (_, asset) in &catalog.timelines {
        timeline_inputs(&mut sources, asset);
        valid.insert(asset.id);
    }
    for (_, effect) in &catalog.effects {
        sources.insert(format!("effect:{}", effect.id), effect.semantic_hash());
        timeline_inputs(&mut sources, &effect.timeline);
        valid.insert(effect.timeline.id);
    }
    let graph = crate::artifact_dependencies::transaction(root, |graph| {
        let removed = graph
            .nodes
            .keys()
            .filter(|key| {
                (key.starts_with("timeline:")
                    || key.starts_with("timeline-audio:")
                    || key.starts_with("effect:")
                    || key.starts_with("marker:"))
                    && !sources.contains_key(*key)
            })
            .cloned()
            .collect::<Vec<_>>();
        for key in removed {
            graph.invalidate(
                &key,
                "Timeline/effect source or marker was removed, is unreadable or is ambiguous",
            );
        }
        for (key, signature) in &sources {
            graph.publish(key, signature.clone(), Default::default());
        }
    })?;
    apply_stale_graph(root, &graph)?;
    Ok((valid, sources, graph))
}

pub fn observe_resources(root: &Path, index: &crate::assets::Index) -> Result<(), String> {
    let graph = crate::artifact_dependencies::transaction(root, |graph| {
        if graph.nodes.contains_key("default-sound-bank") {
            match crate::workspace::read_manifest(root).ok().and_then(|m| m.default_sound_bank) {
                Some(id) => graph.publish("default-sound-bank", id.to_string(), Default::default()),
                None => graph.invalidate("default-sound-bank", "Project Default SoundBank is missing or unreadable"),
            }
        }
        let keys = graph
            .nodes
            .keys()
            .filter_map(|key| {
                key.strip_prefix("asset:").or_else(|| key.strip_prefix("audio-cook:"))
                    .and_then(|id| Uuid::parse_str(id).ok())
                    .map(|id| (key.clone(), id))
            })
            .collect::<Vec<_>>();
        for (key, id) in keys {
            if key.starts_with("audio-cook:") {
                match index.resolve(id).and_then(|record| crate::assets::cook_key(root, &record.meta)) {
                    Ok(signature) => graph.publish(&key, signature, Default::default()),
                    Err(error) => graph.invalidate(&key, &error),
                }
                continue;
            }
            match index.resolve(id) {
                Ok(record) => graph.publish(
                    &key,
                    crate::assets::cache_key(&record.meta),
                    Default::default(),
                ),
                Err(error) => graph.invalidate(&key, &error),
            }
        }
    })?;
    apply_stale_graph(root, &graph)
}

/// Source parse failures hide only their own identities. Reflection failures
/// still use invalidate_all because typed member contracts cannot be refreshed.
pub fn invalidate_catalog(root: &Path, error: &str) -> Result<(), String> {
    match observe_sources(root) {
        Ok(_) => Ok(()),
        Err(_) => invalidate_all(root, error),
    }
}

fn record_preview(root: &Path, asset: &TimelineAsset, result: &PreviewCache) -> Result<(), String> {
    let graph = crate::artifact_dependencies::transaction(root, |graph| {
        graph.publish(
            &format!("timeline:{}", asset.id),
            asset.semantic_hash(),
            Default::default(),
        );
        let key = format!("cooked-timeline:{}", asset.id);
        if !result.stale
            && let Some(compiled) = &result.compiled
        {
            for (id, signature) in &compiled.dependencies {
                graph.publish(id, signature.clone(), Default::default());
            }
            graph.publish(
                &key,
                compiled.signature.clone(),
                compiled.dependencies.keys().cloned().collect(),
            );
        } else {
            graph.invalidate(
                &key,
                &result
                    .diagnostics
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("\n"),
            );
        }
    })?;
    apply_stale_graph(root, &graph)
}

pub fn compile_index(
    asset: &TimelineAsset,
    registry: &Registry,
    index: &crate::assets::Index,
) -> Result<Compiled, Vec<Diagnostic>> {
    let mut compiled = compile(asset, registry)?;
    compiled.resolve_resources(index)?;
    Ok(compiled)
}
pub fn refresh(
    root: &Path,
    asset: &TimelineAsset,
    registry: &Registry,
) -> Result<PreviewCache, String> {
    let folder = root.join(".epok/timelines");
    let path = folder.join(format!("{}.json", asset.id));
    let old = std::fs::read(&path)
        .ok()
        .and_then(|b| serde_json::from_slice::<PreviewCache>(&b).ok());
    let result = match compile_index(
        asset,
        registry,
        &crate::assets::scan(root, &mut Default::default()),
    )
    .and_then(|compiled| {
        let runtime = crate::timeline_runtime::header(&compiled, registry).map_err(|message| {
            vec![Diagnostic {
                asset: asset.id,
                item: asset.id,
                required: true,
                message,
            }]
        })?;
        Ok((compiled, runtime))
    }) {
        Ok((compiled, runtime)) => {
            crate::project::write_changed(
                &folder.join(format!("{}.hh", asset.id)),
                compiled.tables().as_bytes(),
            )?;
            crate::project::write_changed(
                &folder.join(format!("{}.runtime.hh", asset.id)),
                runtime.as_bytes(),
            )?;
            PreviewCache {
                compiled: Some(compiled),
                stale: false,
                diagnostics: vec![],
            }
        }
        Err(diagnostics) => PreviewCache {
            compiled: old.and_then(|p| p.compiled),
            stale: true,
            diagnostics,
        },
    };
    record_preview(root, asset, &result)?;
    crate::project::write_changed(
        &path,
        &serde_json::to_vec_pretty(&result).map_err(|e| e.to_string())?,
    )?;
    Ok(result)
}
pub fn compile_project(root: &Path, registry: &Registry) -> Result<Vec<Compiled>, String> {
    observe_reflection(root, registry)?;
    let mut output = vec![];
    let mut errors = vec![];
    let assets = timeline::load_all(root).inspect_err(|error| {
        let _ = invalidate_catalog(root, error);
    })?;
    for (_, asset) in assets {
        let result = refresh(root, &asset, registry)?;
        if result.stale {
            errors.extend(result.diagnostics.iter().map(ToString::to_string));
        } else if let Some(compiled) = result.compiled {
            output.push(compiled);
        }
    }
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    Ok(output)
}
/// An unusable catalog cannot establish which old descriptors are trustworthy.
/// Retain cached navigation data, but never leave a successful cook status behind.
pub fn invalidate_all(root: &Path, error: &str) -> Result<(), String> {
    let folder = root.join(".epok/timelines");
    if !folder.exists() {
        return Ok(());
    }
    for entry in std::fs::read_dir(folder).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if !entry.file_type().map_err(|e| e.to_string())?.is_file()
            || path.extension().is_none_or(|e| e != "json")
        {
            continue;
        }
        let Some(id) = path
            .file_stem()
            .and_then(|s| s.to_str())
            .and_then(|s| Uuid::parse_str(s).ok())
        else {
            continue;
        };
        invalidate(root, id, error)?;
    }
    Ok(())
}
pub fn invalidate(root: &Path, id: Uuid, error: &str) -> Result<(), String> {
    let graph = crate::artifact_dependencies::transaction(root, |graph| {
        graph.invalidate(&format!("cooked-timeline:{id}"), error);
    })?;
    apply_stale_graph(root, &graph)
}
fn invalidate_cache(root: &Path, id: Uuid, error: &str) -> Result<(), String> {
    let path = root.join(format!(".epok/timelines/{id}.json"));
    let Ok(mut cache) = std::fs::read(&path)
        .map_err(|e| e.to_string())
        .and_then(|bytes| {
            serde_json::from_slice::<PreviewCache>(&bytes).map_err(|e| e.to_string())
        })
    else {
        return Ok(());
    };
    cache.stale = true;
    cache.diagnostics = vec![Diagnostic {
        asset: id,
        item: id,
        required: true,
        message: error.into(),
    }];
    crate::project::write_changed(
        &path,
        &serde_json::to_vec_pretty(&cache).map_err(|e| e.to_string())?,
    )
}
pub fn compile_current_project(root: &Path) -> Result<Vec<Compiled>, String> {
    let registry = crate::scripts::catalog(root)
        .and_then(|catalog| crate::blueprint::native_registry(root, &catalog))
        .inspect_err(|error| {
            let _ = invalidate_all(root, error);
        })?;
    compile_project(root, &registry)
}

#[cfg(test)]
mod actor_requirement_tests {
    use super::*;
    use crate::object_model::{self, ComponentSpec, Model};
    use crate::reflection_schema::{
        self as schema, Cardinality, ClassFamily, ComponentContract, Domain,
        TimelineComponentRequirement as Component,
    };

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
            placement: Default::default(),
            component: None,
            default_components: vec![],
            explicit_abstract: false,
            properties: vec![],
            functions: vec![],
            source: schema::Location {
                file: "assets/scripts/Adapter.hpp".into(),
                line: 1,
                column: 1,
            },
        }
    }
    fn spec(class: &str) -> ComponentSpec {
        ComponentSpec {
            id: Uuid::new_v4(),
            class: class.into(),
            root: false,
        }
    }
    /// Just enough of design.md section 2 to resolve Actor3D and the two
    /// components the test needs; the full table is covered in `object_model`.
    fn native_registry() -> Registry {
        let mut object = class(object_model::OBJECT_ID, "epok::Object", None);
        object.family = Some(ClassFamily::Object);
        object.explicit_abstract = true;
        let mut actor = class(
            object_model::ACTOR_ID,
            "epok::Actor",
            Some(object_model::OBJECT_ID),
        );
        actor.family = Some(ClassFamily::Actor);
        actor.domain = Some(Domain::None);
        actor.explicit_abstract = true;
        let mut actor3d = class(
            object_model::ACTOR3D_ID,
            "epok::Actor3D",
            Some(object_model::ACTOR_ID),
        );
        actor3d.domain = Some(Domain::World3D);
        actor3d.placement.placeable = true;
        let mut component = class(
            object_model::ACTOR_COMPONENT_ID,
            "epok::ActorComponent",
            Some(object_model::OBJECT_ID),
        );
        component.family = Some(ClassFamily::Component);
        component.domain = Some(Domain::None);
        component.explicit_abstract = true;
        let mut scene3d = class(
            object_model::SCENE_COMPONENT3D_ID,
            "epok::SceneComponent3D",
            Some(object_model::ACTOR_COMPONENT_ID),
        );
        scene3d.domain = Some(Domain::World3D);
        scene3d.component = Some(ComponentContract {
            owners: [Domain::World3D].into_iter().collect(),
            can_root: true,
            ..Default::default()
        });
        let mut audio = class(
            object_model::AUDIO_COMPONENT_ID,
            "epok::AudioComponent",
            Some(object_model::ACTOR_COMPONENT_ID),
        );
        audio.domain = Some(Domain::None);
        audio.component = Some(ComponentContract {
            owners: [Domain::World3D, Domain::World2D, Domain::UI]
                .into_iter()
                .collect(),
            cardinality: Cardinality::Multiple,
            capabilities: ["audio".to_owned()].into_iter().collect(),
            ..Default::default()
        });
        let mut registry = Registry::new();
        for c in [object, actor, actor3d, component, scene3d, audio] {
            registry.classes.insert(c.id.clone(), c);
        }
        registry
    }

    #[test]
    fn timeline_requires_is_checked_against_an_actor_component_set() {
        let mut registry = native_registry();
        let mut base = class("adapter:Audio", "AudioAdapter", None);
        base.timeline_component = Some(Component::AudioSource);
        registry.classes.insert(base.id.clone(), base);
        // Inheritance: the derived adapter carries its parent's requirement.
        let derived = class(
            "adapter:Derived",
            "DerivedAudioAdapter",
            Some("adapter:Audio"),
        );
        registry.classes.insert(derived.id.clone(), derived);

        let model = Model::from_registry(&native_registry()).expect("native bases resolve");
        let root = spec(object_model::SCENE_COMPONENT3D_ID);
        let audio = spec(object_model::AUDIO_COMPONENT_ID);

        // An actor that carries AudioComponent satisfies TimelineRequires=AudioSource.
        assert!(
            verify_actor_requirements(
                "adapter:Audio",
                object_model::ACTOR3D_ID,
                &[root.clone(), audio],
                &registry,
                &model,
            )
            .is_ok()
        );
        // One that does not gets a diagnostic naming the missing component.
        let diagnostics = verify_actor_requirements(
            "adapter:Derived",
            object_model::ACTOR3D_ID,
            std::slice::from_ref(&root),
            &registry,
            &model,
        )
        .unwrap_err();
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "missing-timeline-component");
        assert!(diagnostics[0].message.contains("epok::AudioComponent"));
        // An adapter class the registry does not know is not invented into a failure.
        assert!(
            verify_actor_requirements(
                "adapter:Absent",
                object_model::ACTOR3D_ID,
                &[root],
                &registry,
                &model,
            )
            .is_ok()
        );
    }
}
