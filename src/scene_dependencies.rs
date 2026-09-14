//! Host provenance for existing scene documents and the editor's unsaved input.
//! Paths identify document inputs; entity and asset identity stays UUID-based.
use crate::{artifact_dependencies, scene::Scene};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

pub fn signature(scene: &Scene) -> String {
    // A worker's LightingBaked event updates this derived cache. It must not
    // invalidate the very Play that produced it; authored lighting is included.
    let mut source = scene.clone();
    source.bake = None;
    hash(source)
}
pub fn hash(value: impl serde::Serialize) -> String {
    crate::assets::hash(&serde_json::to_vec(&value).expect("Serializable scene dependency"))
}
pub fn catalog_signature(catalog: &[crate::scripts::Script]) -> String {
    hash(
        catalog
            .iter()
            .map(|script| {
                serde_json::json!([
                    script,
                    script.header_path(),
                    script
                        .classes
                        .iter()
                        .map(|c| serde_json::json!([
                            c.id,
                            c.cpp_name,
                            c.parent,
                            c.provider,
                            c.backend,
                            c.abstract_class,
                            c.final_class,
                            c.blueprintable,
                            c.timeline_component,
                            c.properties
                                .iter()
                                .map(|p| serde_json::json!([
                                    p.id,
                                    p.name,
                                    p.value_type,
                                    p.default,
                                    p.editable,
                                    p.timeline
                                ]))
                                .collect::<Vec<_>>()
                        ]))
                        .collect::<Vec<_>>()
                ])
            })
            .collect::<Vec<_>>(),
    )
}
pub fn observe_catalog(
    root: &Path,
    catalog: Result<&[crate::scripts::Script], &str>,
) -> Result<(), String> {
    artifact_dependencies::transaction(root, |graph| match catalog {
        Ok(catalog) => {
            graph.publish(
                "scene-catalog",
                catalog_signature(catalog),
                Default::default(),
            );
            graph.publish(
                "audio-catalog",
                crate::audio::catalog_selection_signature(root, catalog),
                Default::default(),
            );
        }
        Err(error) => {
            graph.invalidate("scene-catalog", error);
            graph.invalidate("audio-catalog", error);
        }
    })?;
    Ok(())
}

#[derive(Clone)]
pub enum Origin {
    Anonymous(String),
    Saved(PathBuf, String),
    Editor(PathBuf, String),
}
pub struct Input {
    pub scene: Scene,
    pub origin: Origin,
    pub play: Option<crate::play::Profile>,
    pub play_settings_signature: Option<String>,
    pub editor_override: Option<(PathBuf, Scene)>,
}
impl Input {
    /// Build resolution happens later, after template refresh. Capture the
    /// migrated authoring snapshot from the existing loader exactly once.
    pub fn load(path: &Path) -> Result<Self, String> {
        let scene = Scene::load_unresolved(path)?;
        let origin = Origin::Saved(path.to_owned(), signature(&scene));
        Ok(Self {
            scene,
            origin,
            play: None,
            play_settings_signature: None,
            editor_override: None,
        })
    }
    pub fn editor(path: PathBuf, scene: Scene) -> Self {
        let origin = Origin::Editor(path, signature(&scene));
        Self {
            scene,
            origin,
            play: None,
            play_settings_signature: None,
            editor_override: None,
        }
    }
}
impl From<Scene> for Input {
    fn from(scene: Scene) -> Self {
        let origin = Origin::Anonymous(signature(&scene));
        Self {
            scene,
            origin,
            play: None,
            play_settings_signature: None,
            editor_override: None,
        }
    }
}
fn relative(root: &Path, path: &Path) -> Result<String, String> {
    let canonical_root = std::fs::canonicalize(root).map_err(|e| e.to_string())?;
    let canonical_path = std::fs::canonicalize(path).ok();
    let relative = canonical_path
        .as_deref()
        .and_then(|p| p.strip_prefix(&canonical_root).ok())
        .or_else(|| path.strip_prefix(root).ok())
        .ok_or("Scene dependency must remain inside the project")?;
    let relative = relative.to_string_lossy().replace('\\', "/");
    crate::assets::inside(root, &relative)?;
    Ok(relative)
}
impl Origin {
    pub fn dependency(&self, root: &Path, target: &str) -> Result<(String, String), String> {
        Ok(match self {
            Self::Anonymous(signature) => (format!("scene-input:{target}"), signature.clone()),
            Self::Saved(path, signature) => (
                format!("scene-file:{}", relative(root, path)?),
                signature.clone(),
            ),
            Self::Editor(path, signature) => (
                format!("scene-editor:{}", relative(root, path)?),
                signature.clone(),
            ),
        })
    }
}

/// Observe already consumed scene files plus the active editor document. Closed
/// unsaved snapshots remain stale navigation data, never a saved-scene fallback.
/// Returns whether either build target consumes an input changed in this poll.
pub fn observe_with_registry(
    root: &Path,
    path: &Path,
    scene: &Scene,
    registry: Result<&crate::blueprint::Registry, &str>,
) -> Result<bool, String> {
    observe_inputs(root, Some((path, scene)), Some(registry))
}

#[cfg(test)]
pub fn observe(root: &Path, path: &Path, scene: &Scene) -> Result<bool, String> {
    observe_inputs(root, Some((path, scene)), None)
}

/// Certification rereads consumed disk inputs, but has no authority to replace
/// submitted editor snapshots or declare their documents closed.
pub fn observe_saved(root: &Path) -> Result<(), String> {
    observe_inputs(root, None, None).map(|_| ())
}

/// Settings UI already owns/validated this snapshot. Invalidate its consumers
/// immediately without parsing scenes, refreshing scripts or scheduling a job.
pub fn apply_settings(
    root: &Path,
    manifest: &crate::workspace::Manifest,
    maps: Option<&crate::scene_bank::Registry>,
) -> Result<(), String> {
    let mut values = BTreeMap::from([
        ("scene-render-settings", hash(manifest.rendering)),
        (
            "display-settings",
            crate::assets::hash(manifest.rendering.header()?.as_bytes()),
        ),
        ("scene-debug-settings", hash(manifest.debug)),
        ("scene-lua-settings", manifest.lua_execution.signature()),
        ("scene-transition-settings", hash(&manifest.transition)),
        (
            "scene-play-settings",
            hash(Some((&manifest.play, &manifest.startup_scene))),
        ),
    ]);
    if let Some(maps) = maps {
        values.insert("scene-registry", hash(maps));
    }
    artifact_dependencies::transaction(root, |graph| {
        for (key, signature) in values {
            if graph.nodes.contains_key(key) {
                graph.publish(key, signature, Default::default());
            }
        }
    })
    .map(|_| ())
}

fn observe_inputs(
    root: &Path,
    editor: Option<(&Path, &Scene)>,
    supplied_registry: Option<Result<&crate::blueprint::Registry, &str>>,
) -> Result<bool, String> {
    inspect_inputs(root, editor, supplied_registry)?.publish(root)
}

/// Read/parse/hash on a worker; publication is a guarded, short UI transaction.
pub struct Observation {
    values: BTreeMap<String, Result<String, String>>,
    baseline: BTreeMap<String, Option<artifact_dependencies::Node>>,
}
pub fn inspect_with_registry(
    root: &Path,
    path: &Path,
    scene: &Scene,
    registry: Result<&crate::blueprint::Registry, &str>,
) -> Result<Observation, String> {
    inspect_inputs(root, Some((path, scene)), Some(registry))
}
fn inspect_inputs(
    root: &Path,
    editor: Option<(&Path, &Scene)>,
    supplied_registry: Option<Result<&crate::blueprint::Registry, &str>>,
) -> Result<Observation, String> {
    let graph = artifact_dependencies::Graph::load(root)?;
    let mut observations = BTreeMap::new();
    let loaded_registry = (supplied_registry.is_none()
        && graph
            .nodes
            .keys()
            .any(|key| key == "audio-catalog" || key.starts_with("audio-selection:")))
    .then(|| crate::scripts::declaration_registry(root));
    // The editor supplies its registry only after its normal source refresh.
    // Headless build certification always rereads declarations from disk.
    let audio_registry = supplied_registry.or_else(|| {
        loaded_registry
            .as_ref()
            .map(|result| result.as_ref().map_err(String::as_str))
    });
    if graph.nodes.contains_key("audio-catalog") {
        observations.insert(
            "audio-catalog".into(),
            audio_registry
                .as_ref()
                .unwrap()
                .as_ref()
                .map(|registry| crate::audio::registry_selection_signature(registry))
                .map_err(|error| (*error).to_owned()),
        );
    }
    let audio_signature = |scene: &Scene| match &audio_registry {
        Some(Ok(registry)) => Ok(crate::audio::selection_signature(scene, Some(registry))),
        Some(Err(error)) => Err((*error).to_owned()),
        None => Ok(crate::audio::selection_signature(scene, None)),
    };
    let mut active = None;
    if let Some((path, scene)) = editor {
        let (key, signature) =
            Origin::Editor(path.to_owned(), signature(scene)).dependency(root, "")?;
        let audio_key = format!("audio-selection:{key}");
        if graph.nodes.contains_key(&audio_key) {
            observations.insert(audio_key, audio_signature(scene));
        }
        observations.insert(key.clone(), Ok(signature));
        active = Some(key);
    }
    if graph.nodes.contains_key("scene-render-settings")
        || graph.nodes.contains_key("display-settings")
    {
        let settings = crate::settings::rendering(root);
        if graph.nodes.contains_key("scene-render-settings") {
            observations.insert("scene-render-settings".into(), settings.clone().map(hash));
        }
        if graph.nodes.contains_key("display-settings") {
            observations.insert(
                "display-settings".into(),
                settings
                    .and_then(|settings| settings.header())
                    .map(|header| crate::assets::hash(header.as_bytes())),
            );
        }
    }
    if graph.nodes.contains_key("scene-registry") {
        observations.insert(
            "scene-registry".into(),
            crate::scene_bank::read(root).map(hash),
        );
    }
    if graph.nodes.contains_key("scene-inventory") {
        observations.insert(
            "scene-inventory".into(),
            crate::scene_bank::available(root).map(hash),
        );
    }
    if graph.nodes.contains_key("scene-transition-settings") {
        observations.insert(
            "scene-transition-settings".into(),
            crate::workspace::optional_manifest(root)
                .map(|m| hash(m.map(|m| m.transition).unwrap_or_default())),
        );
    }
    if graph.nodes.contains_key("scene-debug-settings") {
        observations.insert(
            "scene-debug-settings".into(),
            crate::settings::debug_hud(root).map(hash),
        );
    }
    if graph.nodes.contains_key("scene-play-settings") {
        observations.insert(
            "scene-play-settings".into(),
            crate::play::settings_signature(root),
        );
    }
    // Whole-scene and resource projections must use the same migrated document
    // snapshot, including when a file is edited while observation is running.
    let saved = graph
        .nodes
        .keys()
        .filter_map(|key| {
            key.strip_prefix("scene-file:")
                .or_else(|| key.strip_prefix("audio-selection:scene-file:"))
        })
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .map(|path| {
            (
                path,
                crate::assets::inside(root, path).and_then(|path| Input::load(&path)),
            )
        })
        .collect::<BTreeMap<_, _>>();
    for key in graph.nodes.keys() {
        if let Some(path) = key.strip_prefix("audio-selection:scene-file:") {
            observations.insert(
                key.clone(),
                saved[path]
                    .as_ref()
                    .map_err(Clone::clone)
                    .and_then(|input| audio_signature(&input.scene)),
            );
        } else if key.starts_with("audio-selection:scene-editor:")
            && active
                .as_ref()
                .is_some_and(|active| *key != format!("audio-selection:{active}"))
        {
            observations.insert(
                key.clone(),
                Err("Editor scene document is no longer open".into()),
            );
        } else if let Some(path) = key.strip_prefix("scene-file:") {
            observations.insert(
                key.clone(),
                saved[path]
                    .as_ref()
                    .map(|input| signature(&input.scene))
                    .map_err(Clone::clone),
            );
        } else if key.starts_with("scene-editor:")
            && active.as_ref().is_some_and(|active| key != active)
        {
            observations.insert(
                key.clone(),
                Err("Editor scene document is no longer open".into()),
            );
        }
    }
    // Already-observed inputs need no publication. In particular, repeatedly
    // invalidating a closed document rebuilds the graph's reverse edges every
    // poll even though its error and all affected consumers are unchanged.
    observations.retain(|key, value| match (graph.nodes.get(key), value) {
        (Some(node), Ok(signature)) => {
            node.signature.as_ref() != Some(signature) || !node.stale.is_empty()
        }
        (Some(node), Err(error)) => node.stale.get(key) != Some(error),
        _ => true,
    });
    let baseline = observations
        .keys()
        .map(|key| (key.clone(), graph.nodes.get(key).cloned()))
        .collect();
    Ok(Observation {
        values: observations,
        baseline,
    })
}
impl Observation {
    pub fn publish(self, root: &Path) -> Result<bool, String> {
        self.publish_observation(root, false)
    }
    /// Changing the active document invalidates old outputs, but is not an
    /// authored edit. Still report independent saved-file/settings changes.
    pub fn publish_navigation(self, root: &Path) -> Result<bool, String> {
        self.publish_observation(root, true)
    }
    fn publish_observation(self, root: &Path, navigation: bool) -> Result<bool, String> {
        if self.values.is_empty() {
            return Ok(false);
        }
        let mut affected = false;
        artifact_dependencies::transaction(root, |graph| {
            let mut changed = vec![];
            for (key, result) in self.values {
                // A build or another observer may have published after the read.
                // Never overwrite that newer provenance with a delayed snapshot.
                if graph.nodes.get(&key) != self.baseline[&key].as_ref() {
                    continue;
                }
                let before = graph.nodes.get(&key).cloned();
                match result {
                    Ok(signature) => graph.publish(&key, signature, Default::default()),
                    Err(error) => graph.invalidate(&key, &error),
                }
                let document_switch = navigation
                    && (key.starts_with("scene-editor:")
                        || key.starts_with("audio-selection:scene-editor:"));
                if before.as_ref() != graph.nodes.get(&key) && !document_switch {
                    changed.push(key);
                }
            }
            affected = [".epok/build", ".epok/build-blueprint-debug"]
                .iter()
                .any(|target| {
                    graph
                        .nodes
                        .get(&format!("generated-scene:{target}/scene.hh"))
                        .is_some_and(|node| changed.iter().any(|key| node.stale.contains_key(key)))
                });
        })?;
        Ok(affected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{artifact_dependencies::Graph, playback_staging::Batch};

    /// `signature` hashes the whole document except the derived lighting bake, so a
    /// map's own Blueprint is part of its provenance without a special case: editing
    /// the embedded graph makes everything cooked from that map stale.
    #[test]
    fn the_scene_signature_covers_the_embedded_scene_blueprint() {
        let mut scene = Scene::default();
        let plain = signature(&scene);
        assert!(scene.ensure_scene_script(None));
        let created = signature(&scene);
        assert_ne!(created, plain, "the map now carries a class of its own");

        let blueprint = &mut scene.scene_script.as_mut().unwrap().blueprint;
        blueprint.variables.push(crate::blueprint_asset::Variable {
            id: uuid::Uuid::new_v4().to_string(),
            name: "health".into(),
            value_type: crate::reflection_schema::Type::Fixed,
            default: serde_json::json!(3),
            editable: true,
            timeline_animatable: false,
        });
        assert_ne!(signature(&scene), created, "the graph is part of the map");
    }

    #[test]
    fn navigation_suppresses_document_switch_only_not_saved_source_changes() {
        let root = root();
        let path = root.join("assets/scenes/Main.epokmap");
        let mut scene = Scene::default();
        scene.save(&path).unwrap();
        let old = "scene-editor:assets/scenes/Old.epokmap";
        let saved = "scene-file:assets/scenes/Main.epokmap";
        let generated = "generated-scene:.epok/build/scene.hh";
        artifact_dependencies::transaction(&root, |graph| {
            graph.publish(old, "old-document".into(), Default::default());
            graph.publish(saved, signature(&scene), Default::default());
            graph.publish(generated, "build".into(), [old.into(), saved.into()].into());
        })
        .unwrap();
        let registry = crate::blueprint::Registry::new();
        assert!(
            !inspect_with_registry(&root, &path, &scene, Ok(&registry))
                .unwrap()
                .publish_navigation(&root)
                .unwrap()
        );
        assert!(
            !Graph::load(&root).unwrap().nodes[generated]
                .stale
                .is_empty()
        );
        // An unrelated real edit during navigation must still propagate.
        scene.actors[0].position[0] += 2.;
        scene.save(&path).unwrap();
        assert!(
            inspect_with_registry(&root, &path, &scene, Ok(&registry))
                .unwrap()
                .publish_navigation(&root)
                .unwrap()
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn delayed_observation_does_not_replace_newer_build_provenance() {
        let root = root();
        let path = root.join("assets/scenes/Main.epokmap");
        let scene = Scene::default();
        scene.save(&path).unwrap();
        let registry = crate::blueprint::Registry::new();
        let observation = inspect_with_registry(&root, &path, &scene, Ok(&registry)).unwrap();
        let key = Origin::Editor(path, signature(&scene))
            .dependency(&root, "")
            .unwrap()
            .0;
        artifact_dependencies::transaction(&root, |graph| {
            graph.publish(&key, "newer-editor-snapshot".into(), Default::default());
        })
        .unwrap();
        observation.publish(&root).unwrap();
        assert_eq!(
            Graph::load(&root).unwrap().nodes[&key].signature.as_deref(),
            Some("newer-editor-snapshot")
        );
    }
    fn root() -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("epok-scene-provenance-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(root.join("assets/scenes")).unwrap();
        root
    }
    #[test]
    fn retired_sidecars_invalidate_audio_observation_without_rewriting_sources() {
        let root = root();
        let path = root.join("assets/scripts/Retired.epokscript");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let original = b"{\"name\":\"Retired\",\"properties\":[]}";
        std::fs::write(&path, original).unwrap();
        assert!(
            crate::scripts::native_catalog(&root)
                .unwrap_err()
                .contains("retired script sidecars")
        );
        observe_saved(&root).unwrap();
        observe_catalog(&root, Err("retired script sidecars")).unwrap();
        assert!(
            !Graph::load(&root).unwrap().nodes["audio-catalog"]
                .stale
                .is_empty()
        );
        assert_eq!(std::fs::read(&path).unwrap(), original);
    }
    fn stage(root: &Path, target: &str, input: Input, additional: &[Input]) {
        let mut batch = Batch::new(root, &root.join(target)).unwrap();
        batch.scene_source(root, &input.origin).unwrap();
        let mut scenes = vec![input.scene];
        for input in additional {
            batch.scene_source(root, &input.origin).unwrap();
            scenes.push(input.scene.clone());
        }
        let header = crate::scene_bank::header(&scenes, &[]).unwrap();
        batch.scene_header(header.as_bytes());
        batch.publish(root).unwrap();
    }
    #[test]
    fn unsaved_scene_and_lighting_cache_do_not_relabel_saved_exports() {
        let root = root();
        let path = root.join("assets/scenes/Main.epokmap");
        let mut scene = Scene::default();
        scene.save(&path).unwrap();
        let saved_bytes = std::fs::read(&path).unwrap();
        stage(&root, "exports/saved", Input::load(&path).unwrap(), &[]);
        stage(
            &root,
            ".epok/build",
            Input::editor(path.clone(), scene.clone()),
            &[],
        );
        let baseline = Graph::load(&root).unwrap();
        let saved = baseline.nodes["generated-scene:exports/saved/scene.hh"].clone();
        let playback = baseline.nodes["stage-playback:.epok/build"].clone();
        scene.actors[1].position[0] += 2.;
        assert!(observe(&root, &path, &scene).unwrap());
        let changed = Graph::load(&root).unwrap();
        assert_eq!(
            changed.nodes["generated-scene:exports/saved/scene.hh"],
            saved
        );
        assert!(
            !changed.nodes["generated-scene:.epok/build/scene.hh"]
                .stale
                .is_empty()
        );
        assert!(
            !observe(&root, &path, &scene).unwrap(),
            "unchanged observations must not restart Play repeatedly"
        );
        stage(
            &root,
            ".epok/build",
            Input::editor(path.clone(), scene.clone()),
            &[],
        );
        scene.bake = Some(crate::lighting::bake(&scene).unwrap());
        scene.display_size = [640, 480];
        assert!(!observe(&root, &path, &scene).unwrap());
        let rebuilt = Graph::load(&root).unwrap();
        assert_eq!(rebuilt.nodes["stage-playback:.epok/build"], playback);
        assert!(
            rebuilt.nodes["generated-scene:.epok/build/scene.hh"]
                .stale
                .is_empty()
        );
        assert_eq!(std::fs::read(&path).unwrap(), saved_bytes);
    }
    #[test]
    fn registered_scene_edit_delete_and_parse_failure_keep_precise_stale_edges() {
        let root = root();
        let path = root.join("assets/scenes/Main.epokmap");
        let after_path = root.join("assets/scenes/After.epokmap");
        let scene = Scene::default();
        scene.save(&path).unwrap();
        let mut after = Scene {
            name: "After".into(),
            ..Scene::default()
        };
        after.save(&after_path).unwrap();
        stage(&root, "exports/unrelated", Input::load(&path).unwrap(), &[]);
        stage(
            &root,
            ".epok/build",
            Input::editor(path.clone(), scene.clone()),
            &[Input::load(&after_path).unwrap()],
        );
        let key = "scene-file:assets/scenes/After.epokmap";
        let before = Graph::load(&root).unwrap();
        after.actors[1].active = false;
        after.save(&after_path).unwrap();
        assert!(observe(&root, &path, &scene).unwrap());
        let changed = Graph::load(&root).unwrap();
        assert!(
            changed.nodes["generated-scene:.epok/build/scene.hh"]
                .stale
                .contains_key(key)
        );
        assert_eq!(
            changed.nodes["generated-scene:exports/unrelated/scene.hh"],
            before.nodes["generated-scene:exports/unrelated/scene.hh"]
        );
        stage(
            &root,
            ".epok/build",
            Input::editor(path.clone(), scene.clone()),
            &[Input::load(&after_path).unwrap()],
        );
        let last_valid = Graph::load(&root).unwrap().nodes["generated-scene:.epok/build/scene.hh"]
            .signature
            .clone();
        std::fs::remove_file(&after_path).unwrap();
        assert!(observe(&root, &path, &scene).unwrap());
        assert!(!observe(&root, &path, &scene).unwrap());
        std::fs::write(&after_path, b"{ broken scene").unwrap();
        assert!(observe(&root, &path, &scene).unwrap());
        assert!(Input::load(&after_path).is_err());
        after.save(&after_path).unwrap();
        assert!(observe(&root, &path, &scene).unwrap());
        let restored = Graph::load(&root).unwrap();
        let node = &restored.nodes["generated-scene:.epok/build/scene.hh"];
        assert_eq!(node.signature, last_valid);
        assert!(
            !node.stale.is_empty(),
            "restoring the input cannot certify an old output"
        );
    }
    #[test]
    fn switching_editor_documents_stales_only_the_former_unsaved_snapshot() {
        let root = root();
        let a = root.join("assets/scenes/A.epokmap");
        let b = root.join("assets/scenes/B.epokmap");
        let scene = Scene::default();
        scene.save(&a).unwrap();
        scene.save(&b).unwrap();
        stage(
            &root,
            ".epok/build",
            Input::editor(a.clone(), scene.clone()),
            &[],
        );
        stage(&root, "exports/saved", Input::load(&a).unwrap(), &[]);
        let saved =
            Graph::load(&root).unwrap().nodes["generated-scene:exports/saved/scene.hh"].clone();
        assert!(observe(&root, &b, &scene).unwrap());
        let graph = Graph::load(&root).unwrap();
        assert!(
            graph.nodes["generated-scene:.epok/build/scene.hh"]
                .stale
                .contains_key("scene-editor:assets/scenes/A.epokmap")
        );
        assert_eq!(graph.nodes["generated-scene:exports/saved/scene.hh"], saved);
        assert!(
            !serde_json::to_string(&graph)
                .unwrap()
                .contains("epok-scene-provenance-")
        );
    }
}
