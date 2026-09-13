//! Scene-owned TimelineComponent authoring data. Runtime handles are never saved.
use crate::{blueprint::Registry, scene::Scene, timeline, timeline_compile};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Component {
    pub version: u32,
    pub asset: Option<Uuid>,
    pub bindings: timeline::Bindings,
    pub enabled: bool,
    pub play_on_start: bool,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}
impl Default for Component {
    fn default() -> Self {
        Self {
            version: 1,
            asset: None,
            bindings: Default::default(),
            enabled: true,
            play_on_start: true,
            extra: Default::default(),
        }
    }
}
pub struct Prepared {
    pub source: timeline::TimelineAsset,
    pub compiled: timeline_compile::Compiled,
}
pub fn remap(
    component: &mut Component,
    identities: &BTreeMap<Uuid, Uuid>,
    strict: bool,
) -> Result<(), String> {
    for target in component.bindings.values_mut() {
        let Some(id) = *target else { continue };
        if let Some(replacement) = identities.get(&id) {
            *target = Some(*replacement);
        } else if strict {
            return Err(format!(
                "Timeline template binding {id} points outside its subtree; clear or replace it first"
            ));
        }
    }
    Ok(())
}
#[derive(Default)]
pub struct Inspector {
    assets: Vec<(std::path::PathBuf, timeline::TimelineAsset)>,
    checked: Option<std::time::Instant>,
    error: Option<String>,
}
pub fn inspector(
    ui: &imgui::Ui,
    editor: &mut crate::editor::Editor,
    entity: &mut crate::scene::Entity,
) {
    let Some(component) = &mut entity.timeline else {
        return;
    };
    if !ui.collapsing_header("Timeline Component", imgui::TreeNodeFlags::DEFAULT_OPEN) {
        return;
    }
    if editor
        .timeline_inspector
        .checked
        .is_none_or(|t| t.elapsed().as_secs_f32() >= 1.)
    {
        editor.timeline_inspector.checked = Some(std::time::Instant::now());
        match timeline::inspect_playback(&editor.root) {
            Ok(catalog) => {
                editor.timeline_inspector.assets = catalog.timelines;
                editor.timeline_inspector.error =
                    (!catalog.errors.is_empty()).then(|| catalog.errors.join("\n"));
            }
            Err(error) => {
                editor.timeline_inspector.assets.clear();
                editor.timeline_inspector.error = Some(error);
            }
        }
    }
    if let Some(error) = &editor.timeline_inspector.error {
        ui.text_wrapped(format!("Timeline catalog has errors: {error}"));
    }
    ui.checkbox("Enabled##timeline", &mut component.enabled);
    ui.checkbox("Play on start##timeline", &mut component.play_on_start);
    let selected = editor
        .timeline_inspector
        .assets
        .iter()
        .find(|(_, a)| Some(a.id) == component.asset);
    let preview = selected.map_or_else(
        || {
            component
                .asset
                .map_or("None".into(), |id| format!("Missing asset {id}"))
        },
        |(_, a)| a.name.clone(),
    );
    if let Some(_combo) = ui.begin_combo("Timeline asset", &preview) {
        if ui.selectable("None") {
            component.asset = None;
        }
        for (_, asset) in &editor.timeline_inspector.assets {
            if ui.selectable(format!("{}##{}", asset.name, asset.id)) {
                component.asset = Some(asset.id);
            }
        }
    }
    let mut open = None;
    if let Some((path, asset)) = editor
        .timeline_inspector
        .assets
        .iter()
        .find(|(_, a)| Some(a.id) == component.asset)
    {
        if ui.small_button("Open Timeline") {
            open = Some(path.clone());
        }
        for slot in &asset.slots {
            let _id = ui.push_id(slot.id.to_string());
            let mut value = component
                .bindings
                .get(&slot.id)
                .copied()
                .flatten()
                .map_or(serde_json::Value::Null, |id| serde_json::json!(id));
            let label = format!(
                "{}{}",
                slot.name,
                if slot.required {
                    " (required)"
                } else {
                    " (optional)"
                }
            );
            if crate::blueprint_refs::inspector(
                ui,
                &label,
                &mut value,
                &slot.target,
                &editor.scene,
                &editor.class_registry,
                &editor.assets.index,
            ) {
                component.bindings.insert(
                    slot.id,
                    value.as_str().and_then(|id| Uuid::parse_str(id).ok()),
                );
            }
        }
        for diagnostic in
            asset.validate_bindings(&component.bindings, &editor.scene, &editor.class_registry)
        {
            ui.text_wrapped(diagnostic.to_string());
        }
    }
    if ui.small_button("Remove Timeline Component") {
        entity.timeline = None;
    }
    ui.separator();
    if let Some(path) = open
        && let Err(error) = editor.timeline_editor.open(&path)
    {
        editor.log(error);
    }
}
pub fn prepare(
    root: &Path,
    scenes: &[Scene],
    registry: &Registry,
    direct: &std::collections::BTreeSet<Uuid>,
) -> Result<Vec<Prepared>, String> {
    let mut required = direct.clone();
    for scene in scenes {
        let mut count = 0;
        for entity in &scene.entities {
            let Some(component) = &entity.timeline else {
                continue;
            };
            count += 1;
            if component.version != 1 || !component.extra.is_empty() {
                return Err(format!(
                    "Scene {} / {}: unsupported TimelineComponent schema; source preserved",
                    scene.name, entity.id
                ));
            }
            required.insert(component.asset.ok_or_else(|| {
                format!(
                    "Scene {} / {}: TimelineComponent has no asset UUID",
                    scene.name, entity.id
                )
            })?);
        }
        if count > 8 {
            return Err(format!(
                "Scene {} exceeds eight TimelineComponents",
                scene.name
            ));
        }
    }
    if required.is_empty() {
        return Ok(vec![]);
    }
    let mut result = vec![];
    for (_, source) in timeline::load_referenced(root, &required, false).inspect_err(|error| {
        let _ = timeline_compile::invalidate_catalog(root, error);
    })? {
        if !required.remove(&source.id) {
            continue;
        }
        let preview = timeline_compile::refresh(root, &source, registry)?;
        if preview.stale {
            return Err(preview
                .diagnostics
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n"));
        }
        for scene in scenes {
            for entity in &scene.entities {
                if let Some(component) = &entity.timeline
                    && component.asset == Some(source.id)
                {
                    let errors = source
                        .validate_bindings(&component.bindings, scene, registry)
                        .into_iter()
                        .filter(|d| d.required)
                        .map(|d| format!("Scene {} / {}: {d}", scene.name, entity.id))
                        .collect::<Vec<_>>();
                    if !errors.is_empty() {
                        return Err(errors.join("\n"));
                    }
                }
            }
        }
        result.push(Prepared {
            source,
            compiled: preview
                .compiled
                .ok_or("Timeline cook has no valid artifact")?,
        });
    }
    if !required.is_empty() {
        let error = format!(
            "Missing TimelineAsset UUIDs: {}",
            required
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        );
        timeline_compile::invalidate_catalog(root, &error)?;
        return Err(error);
    }
    result.sort_by_key(|p| p.compiled.asset);
    crate::timeline_runtime::validate_ids(result.iter().map(|p| &p.compiled), registry)?;
    Ok(result)
}
pub fn setup(scene: &Scene, timelines: &[Prepared], registry: &Registry) -> Result<String, String> {
    let mut out = String::from("epok::timeline::component_count=0;\n");
    for (owner, entity) in scene.entities.iter().enumerate() {
        let Some(component) = &entity.timeline else {
            continue;
        };
        let prepared = timelines
            .iter()
            .find(|t| Some(t.compiled.asset) == component.asset)
            .ok_or("Missing validated scene timeline")?;
        out += &format!(
            "{{auto& component=epok::timeline::components[epok::timeline::component_count++];component={{}};component.asset=&epok::timeline::cooked::asset_{}::asset;component.owner=epok::handle(&objects[{owner}]);component.enabled={};component.automatic={};\n",
            prepared.compiled.asset.simple(),
            component.enabled,
            component.play_on_start
        );
        for (i, slot) in prepared.compiled.slots.iter().enumerate() {
            let value = component
                .bindings
                .get(&slot.id)
                .copied()
                .flatten()
                .map_or(serde_json::Value::Null, |id| serde_json::json!(id));
            match crate::blueprint_refs::assignment(
                &format!("component.targets[{i}]"),
                &value,
                &slot.target,
                scene,
                registry,
            ) {
                Ok(assignment) => out += &assignment,
                Err(error) if slot.required => return Err(error),
                Err(_) => out += &format!("component.targets[{i}]={{}};\n"),
            }
        }
        out += "}\n";
    }
    Ok(out)
}
pub fn setup_template(
    scene: &Scene,
    timelines: &[Prepared],
    registry: &Registry,
) -> Result<String, String> {
    let mut out = String::new();
    for (owner, entity) in scene.entities.iter().enumerate() {
        let Some(component) = &entity.timeline else {
            continue;
        };
        let prepared = timelines
            .iter()
            .find(|t| Some(t.compiled.asset) == component.asset)
            .ok_or("Missing validated template timeline")?;
        out += &format!(
            "{{auto* component=epok::timeline::configure(handles[{owner}],epok::timeline::cooked::asset_{}::asset,{},{});if(!component)return false;\n",
            prepared.compiled.asset.simple(),
            component.enabled,
            component.play_on_start
        );
        for (i, slot) in prepared.compiled.slots.iter().enumerate() {
            let value = component
                .bindings
                .get(&slot.id)
                .copied()
                .flatten()
                .map_or(serde_json::Value::Null, |id| serde_json::json!(id));
            let mut assignment = match crate::blueprint_refs::assignment(
                &format!("component->targets[{i}]"),
                &value,
                &slot.target,
                scene,
                registry,
            ) {
                Ok(code) => code,
                Err(error) if slot.required => return Err(error),
                Err(_) => format!("component->targets[{i}]={{}};\n"),
            };
            for index in 0..scene.entities.len() {
                assignment = assignment.replace(
                    &format!("epok::handle(&objects[{index}])"),
                    &format!("handles[{index}]"),
                );
            }
            out += &assignment;
        }
        out += "}\n";
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scene_component_roundtrip_preserves_authoring_ids_and_migrates_only_when_used() {
        let mut scene = Scene::default();
        let entity = scene.entities[1].id;
        let slot = Uuid::new_v4();
        let asset = Uuid::new_v4();
        scene.entities[0].timeline = Some(Component {
            asset: Some(asset),
            bindings: BTreeMap::from([(slot, Some(entity))]),
            ..Default::default()
        });
        scene.upgrade_entity_ids();
        assert_eq!(scene.version, 4);
        let bytes = serde_json::to_vec(&scene).unwrap();
        let mut loaded: Scene = serde_json::from_slice(&bytes).unwrap();
        loaded.entities.swap(1, 2);
        let component = loaded.entities[0].timeline.as_ref().unwrap();
        assert_eq!(component.asset, Some(asset));
        assert_eq!(component.bindings[&slot], Some(entity));
        let data = serde_json::to_string(component).unwrap();
        assert!(!data.contains("generation") && !data.contains("index"));
        assert_eq!(Scene::default().version, 3);
    }
}
