//! Record the exact successful writes of a synchronous staging operation.
//! This is an output manifest, not an asset loader or a filesystem inventory.
use std::{
    cell::RefCell,
    collections::BTreeMap,
    marker::PhantomData,
    path::{Component, Path, PathBuf},
    rc::Rc,
};
pub type Files = BTreeMap<String, String>;
struct Active {
    destination: PathBuf,
    files: Files,
    inputs: Files,
}
thread_local! {
    static ACTIVE: RefCell<Option<Active>> = const { RefCell::new(None) };
}
pub struct Capture {
    _thread: PhantomData<Rc<()>>,
}
impl Capture {
    pub fn begin(destination: &Path) -> Result<Self, String> {
        ACTIVE.with(|active| {
            let mut active = active.borrow_mut();
            if active.is_some() {
                return Err("A staging write capture is already active on this worker".into());
            }
            *active = Some(Active {
                destination: destination.to_owned(),
                files: Files::new(),
                inputs: Files::new(),
            });
            Ok(Self {
                _thread: PhantomData,
            })
        })
    }
    pub fn inputs(&self) -> Files {
        ACTIVE.with(|active| {
            active
                .borrow()
                .as_ref()
                .expect("Active staging capture")
                .inputs
                .clone()
        })
    }
    pub fn finish(self) -> Files {
        ACTIVE.with(|active| {
            active
                .borrow_mut()
                .take()
                .expect("Active staging capture")
                .files
        })
    }
}
impl Drop for Capture {
    fn drop(&mut self) {
        ACTIVE.with(|active| {
            active.borrow_mut().take();
        });
    }
}
/// Record snapshots actually read by existing metadata loaders. Repeated reads
/// in one staging operation must agree, even if the resulting declarations match.
pub fn consumed(inputs: Files) -> Result<(), String> {
    ACTIVE.with(|active| {
        let mut active = active.borrow_mut();
        let Some(active) = active.as_mut() else {
            return Ok(());
        };
        for (key, signature) in inputs {
            if active.inputs.get(&key).is_some_and(|old| old != &signature) {
                return Err(format!(
                    "Native metadata input {key} changed during staging"
                ));
            }
            active.inputs.insert(key, signature);
        }
        Ok(())
    })
}
pub fn written(path: &Path, bytes: &[u8]) -> Result<(), String> {
    ACTIVE.with(|active| {
        let mut active = active.borrow_mut();
        let Some(active) = active.as_mut() else {
            return Ok(());
        };
        let Ok(path) = path.strip_prefix(&active.destination) else {
            return Ok(());
        };
        if path.as_os_str().is_empty()
            || path
                .components()
                .any(|c| !matches!(c, Component::Normal(_)))
        {
            return Err("Staged output paths must remain inside the selected destination".into());
        }
        active.files.insert(
            path.to_string_lossy().replace('\\', "/"),
            crate::assets::hash(bytes),
        );
        Ok(())
    })
}

pub fn native_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    Ok(crate::reflection::script_files(root)?
        .into_iter()
        .filter(|path| {
            path.extension().is_some_and(|e| {
                ["cpp", "cc", "hpp", "hh", "h", "inl"]
                    .iter()
                    .any(|v| e == *v)
            })
        })
        .collect())
}
pub fn native_key(root: &Path, path: &Path) -> Result<String, String> {
    Ok(format!(
        "native-file:{}",
        path.strip_prefix(root)
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .replace('\\', "/")
    ))
}
pub fn native_set(root: &Path, files: &[PathBuf]) -> Result<String, String> {
    let keys = files
        .iter()
        .map(|p| native_key(root, p))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(crate::assets::hash(
        &serde_json::to_vec(&keys).expect("Serializable native file set"),
    ))
}
/// Observe bytes, not C++ syntax. The existing reflection/compiler owns syntax
/// validation; body-only edits must still reach staged files and executables.
pub fn observe_native(root: &Path) -> Result<(), String> {
    let files = native_files(root)?;
    let mut inputs = BTreeMap::from([("native-sources".into(), Ok(native_set(root, &files)?))]);
    for path in files {
        inputs.insert(
            native_key(root, &path)?,
            std::fs::read(path)
                .map(|b| crate::assets::hash(&b))
                .map_err(|e| e.to_string()),
        );
    }
    crate::artifact_dependencies::transaction(root, |graph| {
        let removed = graph
            .nodes
            .keys()
            .filter(|key| key.starts_with("native-file:") && !inputs.contains_key(*key))
            .cloned()
            .collect::<Vec<_>>();
        for key in removed {
            graph.invalidate(&key, "Native source file was removed");
        }
        for (key, value) in inputs {
            match value {
                Ok(signature) => graph.publish(&key, signature, Default::default()),
                Err(error) => graph.invalidate(&key, &error),
            }
        }
    })?;
    Ok(())
}

fn checked_output(destination: &Path, path: &str) -> Result<PathBuf, String> {
    let path = Path::new(path);
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err("Invalid staged file identity".into());
    }
    let path = destination.join(path);
    let canonical = std::fs::canonicalize(&path).map_err(|e| e.to_string())?;
    if !canonical.starts_with(std::fs::canonicalize(destination).map_err(|e| e.to_string())?) {
        return Err("Staged files cannot follow links outside their destination".into());
    }
    Ok(path)
}
fn manifest(graph: &crate::artifact_dependencies::Graph, target: &str) -> Result<Files, String> {
    let stage = graph
        .nodes
        .get(&format!("stage:{target}"))
        .ok_or("Missing staged input manifest")?;
    if stage.signature.is_none() || !stage.stale.is_empty() {
        return Err("Staged inputs are stale; prepare this target again".into());
    }
    if !graph.depends_on_any(
        &format!("stage:{target}"),
        &["blueprint-sources".into()].into_iter().collect(),
    ) {
        return Err(
            "Staged inputs lack Blueprint source-set provenance; prepare this target again".into(),
        );
    }
    if graph
        .nodes
        .contains_key(&format!("generated-scene:{target}/scene.hh"))
        && !graph.depends_on_any(
            &format!("stage:{target}"),
            &[crate::native_metadata::FILES.into()].into_iter().collect(),
        )
    {
        return Err(
            "Staged scene lacks native metadata provenance; prepare this target again".into(),
        );
    }
    let prefix = format!("staged-file:{target}/");
    stage
        .dependencies
        .iter()
        .map(|key| {
            let path = key
                .strip_prefix(&prefix)
                .ok_or("Unexpected staged manifest dependency")?;
            let node = graph
                .nodes
                .get(key)
                .ok_or("Missing staged file provenance")?;
            if !node.stale.is_empty() {
                return Err(format!("Staged file {path} is stale"));
            }
            Ok((
                path.to_owned(),
                node.signature
                    .clone()
                    .ok_or("Missing staged file signature")?,
            ))
        })
        .collect()
}
fn verify(destination: &Path, files: &Files) -> Result<(), String> {
    for (path, expected) in files {
        let bytes = std::fs::read(checked_output(destination, path)?).map_err(|e| e.to_string())?;
        if crate::assets::hash(&bytes) != *expected {
            return Err(format!(
                "Staged file {path} changed after generation; prepare this target again"
            ));
        }
    }
    Ok(())
}
fn invalidate_unchanged_stage(
    root: &Path,
    key: &str,
    expected: &crate::artifact_dependencies::Node,
    error: &str,
) {
    let _ = crate::artifact_dependencies::transaction(root, |graph| {
        if graph.nodes.get(key) == Some(expected) {
            graph.invalidate(key, error);
        }
    });
}

/// Explicit debugger/disc preparation writes extend the same stage manifest.
/// They cannot certify unrelated files or revive a source-invalidated stage.
pub fn patch(
    root: &Path,
    destination: &Path,
    patches: Files,
    options: String,
) -> Result<(), String> {
    let target = crate::playback_staging::target(root, destination)?;
    let mut failure = None;
    crate::artifact_dependencies::transaction(root, |graph| {
        let result = (|| {
            let mut files = manifest(graph, &target)?;
            verify(
                destination,
                &files
                    .iter()
                    .filter(|(path, _)| !patches.contains_key(*path))
                    .map(|(p, s)| (p.clone(), s.clone()))
                    .collect(),
            )?;
            verify(destination, &patches)?;
            let option_key = format!("build-options:{target}");
            graph.publish(&option_key, options, Default::default());
            for (path, signature) in patches {
                let key = format!("staged-file:{target}/{path}");
                let mut dependencies = graph
                    .nodes
                    .get(&key)
                    .map(|n| n.dependencies.clone())
                    .unwrap_or_default();
                dependencies.insert(option_key.clone());
                graph.publish(&key, signature.clone(), dependencies);
                files.insert(path, signature);
            }
            graph.publish(
                &format!("stage:{target}"),
                crate::scene_dependencies::hash(&files),
                files
                    .keys()
                    .map(|p| format!("staged-file:{target}/{p}"))
                    .collect(),
            );
            Ok::<_, String>(())
        })();
        failure = result.err();
        if let Some(error) = &failure {
            graph.invalidate(&format!("stage:{target}"), error);
        }
    })?;
    failure.map_or(Ok(()), Err)
}

#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BuildTicket {
    target: String,
    stage: crate::artifact_dependencies::Node,
    options: crate::artifact_dependencies::Node,
    native: Option<crate::artifact_dependencies::Node>,
    files: Files,
}

fn observe_build_sources(root: &Path) -> Result<(), String> {
    let mut last = std::time::Instant::now();
    let mut measured = |label| {
        if std::env::var_os("EPOK_PROFILE_BUILD").is_some() {
            eprintln!(
                "[build-validation] {label}: {:.1} ms",
                last.elapsed().as_secs_f64() * 1000.
            );
        }
        last = std::time::Instant::now();
    };
    observe_native(root)?;
    measured("native sources");
    crate::native_metadata::observe(root)?;
    measured("native metadata inputs");
    crate::build_inputs::observe(root)?;
    measured("compiler configuration");
    crate::timeline_compile::observe_sources(root)?;
    measured("timeline sources");
    crate::blueprint_dependencies::observe_current(root)?;
    measured("Blueprint sources and reflection");
    crate::scene_dependencies::observe_saved(root)?;
    measured("saved scene dependencies");
    // A fresh scan validates package bytes and checksums, independent of the
    // editor's timestamp cache. Only recorded asset consumers are invalidated.
    let index = crate::assets::scan(root, &mut Default::default());
    measured("asset packages");
    let result = crate::timeline_compile::observe_resources(root, &index);
    measured("resource dependencies");
    result
}

impl BuildTicket {
    pub fn begin(root: &Path, destination: &Path) -> Result<Self, String> {
        observe_build_sources(root)?;
        let target = crate::playback_staging::target(root, destination)?;
        let graph = crate::artifact_dependencies::Graph::load(root)?;
        let stage_key = format!("stage:{target}");
        let stage = graph
            .nodes
            .get(&stage_key)
            .ok_or("Missing staged input manifest")?
            .clone();
        let files = manifest(&graph, &target)
            .and_then(|files| {
                verify(destination, &files)?;
                Ok(files)
            })
            .inspect_err(|error| invalidate_unchanged_stage(root, &stage_key, &stage, error))?;
        let options = graph
            .nodes
            .get(&format!("build-options:{target}"))
            .filter(|node| node.signature.is_some() && node.stale.is_empty())
            .ok_or("Missing current build options")?
            .clone();
        let native = graph.nodes.get(&format!("native-build:{target}")).cloned();
        if native
            .as_ref()
            .is_some_and(|node| node.signature.is_none() || !node.stale.is_empty())
        {
            return Err("Native build inputs are stale; prepare the compiler inputs again".into());
        }
        crate::artifact_dependencies::transaction(root, |graph| {
            graph.invalidate(
                &format!("executable:{target}/epok.ps-exe"),
                "Compilation has not completed for this build request",
            );
        })?;
        Ok(Self {
            target,
            stage,
            options,
            native,
            files,
        })
    }
    pub fn begin_native(root: &Path, destination: &Path) -> Result<Self, String> {
        let ticket = Self::begin(root, destination)?;
        if ticket.native.is_none() {
            return Err("Missing native compiler input provenance".into());
        }
        Ok(ticket)
    }
    pub fn complete(self, root: &Path, destination: &Path, bytes: &[u8]) -> Result<(), String> {
        observe_build_sources(root)?;
        self.publish_verified(root, destination, bytes)
    }
    /// Reuse is allowed only after a fresh begin_native (including content and
    /// manifest validation) and a fresh compiler dependency capture. The ticket
    /// must be identical to the one that certified the cached executable.
    pub fn reuse(
        self,
        previous: &Self,
        root: &Path,
        destination: &Path,
        bytes: &[u8],
    ) -> Result<(), String> {
        if &self != previous {
            return Err("Cached build inputs no longer match their certificate".into());
        }
        self.publish_verified(root, destination, bytes)
    }
    fn publish_verified(self, root: &Path, destination: &Path, bytes: &[u8]) -> Result<(), String> {
        let current = crate::artifact_dependencies::Graph::load(root)?;
        let stage_key = format!("stage:{}", self.target);
        if current.nodes.get(&stage_key) != Some(&self.stage)
            || current.nodes.get(&format!("build-options:{}", self.target)) != Some(&self.options)
            || current.nodes.get(&format!("native-build:{}", self.target)) != self.native.as_ref()
        {
            return Err("Build inputs changed during compilation; no executable launched".into());
        }
        verify(destination, &self.files).inspect_err(|error| {
            invalidate_unchanged_stage(root, &stage_key, &self.stage, error)
        })?;
        let mut failure = None;
        crate::artifact_dependencies::transaction(root, |graph| {
            let stage = format!("stage:{}", self.target);
            if graph.nodes.get(&stage) != Some(&self.stage)
                || graph.nodes.get(&format!("build-options:{}", self.target)) != Some(&self.options)
                || graph.nodes.get(&format!("native-build:{}", self.target)) != self.native.as_ref()
            {
                failure =
                    Some("Build inputs changed during compilation; no executable launched".into());
                return;
            }
            let mut dependencies =
                std::collections::BTreeSet::from([stage, format!("build-options:{}", self.target)]);
            if self.native.is_some() {
                dependencies.insert(format!("native-build:{}", self.target));
            }
            graph.publish(
                &format!("executable:{}/epok.ps-exe", self.target),
                crate::assets::hash(bytes),
                dependencies,
            );
        })?;
        failure.map_or(Ok(()), Err)
    }
}

pub fn publish_export(root: &Path, destination: &Path, documents: Files) -> Result<(), String> {
    observe_build_sources(root)?;
    let target = crate::playback_staging::target(root, destination)?;
    let mut failure = None;
    crate::artifact_dependencies::transaction(root, |graph| {
        let result = (|| {
            let mut files = manifest(graph, &target)?;
            verify(destination, &files)?;
            verify(destination, &documents)?;
            let mut dependencies = std::collections::BTreeSet::from([format!("stage:{target}")]);
            for (path, signature) in documents {
                let source = format!("export-document:{path}");
                graph.publish(&source, signature.clone(), Default::default());
                let key = format!("export-file:{target}/{path}");
                graph.publish(&key, signature.clone(), [source].into_iter().collect());
                dependencies.insert(key);
                files.insert(path, signature);
            }
            graph.publish(
                &format!("export:{target}"),
                crate::scene_dependencies::hash(files),
                dependencies,
            );
            Ok::<_, String>(())
        })();
        failure = result.err();
    })?;
    failure.map_or(Ok(()), Err)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{artifact_dependencies::Graph, playback_staging::Batch};
    #[test]
    #[ignore = "requires a disposable project copy in EPOK_IDLE_PROJECT"]
    fn profile_build_validation() {
        let root = PathBuf::from(std::env::var_os("EPOK_IDLE_PROJECT").unwrap());
        let bytes = vec![42; 64 * 1024 * 1024];
        let started = std::time::Instant::now();
        std::hint::black_box(crate::assets::hash(&bytes));
        eprintln!("Hash 64 MiB: {:?}", started.elapsed());
        for _ in 0..2 {
            let started = std::time::Instant::now();
            observe_build_sources(&root).unwrap();
            eprintln!("Build validation total: {:?}", started.elapsed());
        }
    }
    fn fixture() -> PathBuf {
        let root = std::env::temp_dir().join(format!("epok-stage-files-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(root.join("assets/scripts")).unwrap();
        std::fs::write(root.join("assets/scripts/Probe.cpp"), b"int probe=1;\n").unwrap();
        std::fs::write(
            root.join("assets/scripts/Independent.cpp"),
            b"int independent=3;\n",
        )
        .unwrap();
        root
    }
    fn stage(root: &Path, name: &str) -> PathBuf {
        let destination = root.join(name);
        let capture = Capture::begin(&destination).unwrap();
        let artifacts = crate::script_backend::prepare_all(root, &[]).unwrap();
        artifacts.stage(root, &destination).unwrap();
        let mut batch = Batch::new(root, &destination).unwrap();
        batch.scripts(&artifacts).unwrap();
        batch.files(capture.finish()).unwrap();
        batch.publish(root).unwrap();
        patch(root, &destination, Files::new(), "release".into()).unwrap();
        destination
    }
    #[test]
    fn certification_rechecks_saved_scenes_and_settings_without_replacing_editor_inputs() {
        use crate::scene_dependencies::Input;
        let root = fixture();
        let path = root.join("assets/scenes/Main.epokmap");
        let mut saved = crate::scene::Scene::default();
        saved.save(&path).unwrap();
        let mut editor = saved.clone();
        editor.actors[1].position[0] += 3.;
        let stage_scene = |target: &str, input: Input| {
            let destination = root.join(target);
            crate::project::stage_with_origin(&root, &input.scene, &destination, &input.origin)
                .unwrap();
            patch(&root, &destination, Files::new(), "release".into()).unwrap();
            destination
        };
        let build = stage_scene(".epok/build", Input::editor(path.clone(), editor.clone()));
        let export = stage_scene("exports/saved", Input::load(&path).unwrap());
        let baseline = Graph::load(&root).unwrap();
        let pending = BuildTicket::begin(&root, &build).unwrap();
        let saved_pending = BuildTicket::begin(&root, &export).unwrap();
        saved.actors[1].active = false;
        saved.save(&path).unwrap();
        pending
            .complete(&root, &build, b"submitted editor scene")
            .unwrap();
        assert!(
            saved_pending
                .complete(&root, &export, b"old saved scene")
                .is_err()
        );
        assert!(publish_export(&root, &export, Files::new()).is_err());
        let graph = Graph::load(&root).unwrap();
        for key in [
            "scene-editor:assets/scenes/Main.epokmap",
            "audio-selection:scene-editor:assets/scenes/Main.epokmap",
        ] {
            assert_eq!(graph.nodes[key], baseline.nodes[key]);
        }
        assert!(
            graph.nodes["stage:exports/saved"]
                .stale
                .contains_key("scene-file:assets/scenes/Main.epokmap")
        );
        // Deletion and repair retain stale output until this target is restaged.
        stage_scene("exports/saved", Input::load(&path).unwrap());
        std::fs::remove_file(&path).unwrap();
        assert!(BuildTicket::begin(&root, &export).is_err());
        saved.save(&path).unwrap();
        assert!(BuildTicket::begin(&root, &export).is_err());
        stage_scene("exports/saved", Input::load(&path).unwrap());
        publish_export(&root, &export, Files::new()).unwrap();

        let after_path = root.join("assets/scenes/After.epokmap");
        let mut after = crate::scene::Scene {
            name: "After".into(),
            ..Default::default()
        };
        after.save(&after_path).unwrap();
        let pending = BuildTicket::begin(&root, &build).unwrap();
        crate::scene_bank::Registry {
            scenes: vec!["assets/scenes/After.epokmap".into()],
        }
        .save(&root)
        .unwrap();
        assert!(
            pending
                .complete(&root, &build, b"old scene registry")
                .is_err()
        );
        assert!(
            Graph::load(&root).unwrap().nodes["stage:.epok/build"]
                .stale
                .contains_key("scene-registry")
        );
        stage_scene(".epok/build", Input::editor(path.clone(), editor.clone()));
        let pending = BuildTicket::begin(&root, &build).unwrap();
        after.actors[1].position[2] += 2.;
        after.save(&after_path).unwrap();
        assert!(
            pending
                .complete(&root, &build, b"old registered scene")
                .is_err()
        );
        stage_scene(".epok/build", Input::editor(path.clone(), editor.clone()));
        std::fs::write(&after_path, b"{").unwrap();
        assert!(BuildTicket::begin(&root, &build).is_err());
        after.save(&after_path).unwrap();
        assert!(BuildTicket::begin(&root, &build).is_err());
        stage_scene(".epok/build", Input::editor(path.clone(), editor.clone()));

        let pending = BuildTicket::begin(&root, &build).unwrap();
        let manifest = crate::workspace::Manifest {
            format_version: crate::workspace::FORMAT,
            editor_version: env!("CARGO_PKG_VERSION").into(),
            name: "Certification".into(),
            debug: Default::default(),
            controls: Default::default(),
            lua_execution: Default::default(),
            lua_profile: Default::default(),
            play: Default::default(),
            transition: Default::default(),
            startup_scene: "assets/scenes/Main.epokmap".into(),
            default_sound_bank: None,
            default_scene_script_parent: None,
            auto_build: false,
            build: Default::default(),
            rendering: crate::settings::Rendering {
                width: 320,
                height: 240,
                ..Default::default()
            },
        };
        let descriptor = root.join("Certification.epokproject");
        std::fs::write(&descriptor, serde_json::to_vec(&manifest).unwrap()).unwrap();
        assert!(
            pending
                .complete(&root, &build, b"old rendering settings")
                .is_err()
        );
        let graph = Graph::load(&root).unwrap();
        assert!(
            graph.nodes["stage:.epok/build"]
                .stale
                .contains_key("scene-render-settings")
        );
        assert!(
            graph.nodes["generated-resource:.epok/build/display.hh"]
                .stale
                .contains_key("display-settings")
        );
        stage_scene(".epok/build", Input::editor(path.clone(), editor.clone()));
        std::fs::write(&descriptor, b"{").unwrap();
        assert!(publish_export(&root, &build, Files::new()).is_err());
        std::fs::write(&descriptor, serde_json::to_vec(&manifest).unwrap()).unwrap();
        assert!(BuildTicket::begin(&root, &build).is_err());
        stage_scene(".epok/build", Input::editor(path, editor));
        BuildTicket::begin(&root, &build)
            .unwrap()
            .complete(&root, &build, b"current")
            .unwrap();
    }

    #[test]
    fn playback_changes_during_staging_or_compilation_cannot_certify_outputs() {
        let root = fixture();
        std::fs::create_dir_all(root.join("assets/Timelines")).unwrap();
        let mut source = crate::timeline::TimelineAsset::new("Used".into());
        let path = root.join("assets/Timelines/Used.timeline.json");
        std::fs::write(&path, serde_json::to_vec(&source).unwrap()).unwrap();
        let stage_timeline = |target: &str, source: &crate::timeline::TimelineAsset| {
            let destination = root.join(target);
            let capture = Capture::begin(&destination).unwrap();
            let artifacts = crate::script_backend::prepare_all(&root, &[]).unwrap();
            artifacts.stage(&root, &destination).unwrap();
            let registry = crate::blueprint::Registry::new();
            let compiled = crate::timeline_compile::compile(source, &registry).unwrap();
            let header = crate::timeline_runtime::header(&compiled, &registry).unwrap();
            let mut batch = Batch::new(&root, &destination).unwrap();
            batch.scripts(&artifacts).unwrap();
            batch.timeline(&compiled, header.as_bytes()).unwrap();
            crate::project::write_changed(
                &destination.join(format!("timelines/{}.hh", source.id)),
                header.as_bytes(),
            )
            .unwrap();
            batch.files(capture.finish()).unwrap();
            batch.publish(&root).unwrap();
            patch(&root, &destination, Files::new(), "release".into()).unwrap();
            destination
        };
        let build = stage_timeline(".epok/build", &source);
        let ticket = BuildTicket::begin(&root, &build).unwrap();
        std::fs::write(
            root.join("assets/Timelines/UnusedBroken.timeline.json"),
            b"{",
        )
        .unwrap();
        ticket
            .complete(&root, &build, b"current executable")
            .unwrap();
        let ticket = BuildTicket::begin(&root, &build).unwrap();
        source.duration_ticks += 68;
        std::fs::write(&path, serde_json::to_vec(&source).unwrap()).unwrap();
        assert!(
            ticket
                .complete(&root, &build, b"obsolete executable")
                .is_err()
        );
        let graph = Graph::load(&root).unwrap();
        assert!(
            graph.nodes["executable:.epok/build/epok.ps-exe"]
                .stale
                .contains_key(&format!("timeline:{}", source.id))
        );
        stage_timeline(".epok/build", &source);
        std::fs::write(&path, b"{").unwrap();
        assert!(BuildTicket::begin(&root, &build).is_err());
        std::fs::write(&path, serde_json::to_vec(&source).unwrap()).unwrap();
        let export = stage_timeline("exports/test", &source);
        source.duration_ticks += 68;
        std::fs::write(&path, serde_json::to_vec(&source).unwrap()).unwrap();
        assert!(publish_export(&root, &export, Files::new()).is_err());
        assert!(
            !Graph::load(&root).unwrap().nodes["stage:exports/test"]
                .stale
                .is_empty()
        );
    }
    #[test]
    fn first_blueprint_and_legacy_manifest_require_fresh_staging() {
        let root = fixture();
        let build = stage(&root, ".epok/build");
        let pending = BuildTicket::begin(&root, &build).unwrap();
        let source = crate::blueprint_asset::BlueprintAsset::new(
            "Added".into(),
            uuid::Uuid::new_v4().to_string(),
        );
        let path = root.join("assets/Added.epokbp");
        std::fs::write(&path, serde_json::to_vec(&source).unwrap()).unwrap();
        assert!(
            pending
                .complete(&root, &build, b"old empty project")
                .unwrap_err()
                .contains("changed during compilation")
        );
        assert!(
            Graph::load(&root).unwrap().nodes["stage:.epok/build"]
                .stale
                .contains_key("blueprint-sources")
        );
        std::fs::remove_file(path).unwrap();
        assert!(BuildTicket::begin(&root, &build).is_err());
        stage(&root, ".epok/build");
        // Simulate an earlier provenance manifest. Missing coverage cannot be
        // repaired by merely observing today's sources against old output.
        crate::artifact_dependencies::transaction(&root, |graph| {
            graph.nodes.remove("blueprint-sources");
            for node in graph.nodes.values_mut() {
                node.dependencies.remove("blueprint-sources");
            }
        })
        .unwrap();
        assert!(
            BuildTicket::begin(&root, &build)
                .err()
                .unwrap()
                .contains("lack Blueprint source-set provenance")
        );
        stage(&root, ".epok/build");
        BuildTicket::begin(&root, &build)
            .unwrap()
            .complete(&root, &build, b"current")
            .unwrap();
    }

    #[test]
    fn native_body_changes_reach_executable_and_export_without_touching_other_sources() {
        let root = fixture();
        let build = stage(&root, ".epok/build");
        let export = stage(&root, "exports/test");
        BuildTicket::begin(&root, &build)
            .unwrap()
            .complete(&root, &build, b"linked original")
            .unwrap();
        publish_export(&root, &export, Files::new()).unwrap();
        let before = Graph::load(&root).unwrap();
        std::fs::write(root.join("assets/scripts/Probe.cpp"), b"int probe=2;\n").unwrap();
        observe_native(&root).unwrap();
        let changed = Graph::load(&root).unwrap();
        let source = "native-file:assets/scripts/Probe.cpp";
        for key in [
            "stage:.epok/build",
            "executable:.epok/build/epok.ps-exe",
            "export:exports/test",
        ] {
            assert!(changed.nodes[key].stale.contains_key(source), "{key}");
            assert_eq!(changed.nodes[key].signature, before.nodes[key].signature);
        }
        let independent = "generated-script:.epok/build/scripts/Independent.cpp";
        assert_eq!(changed.nodes[independent], before.nodes[independent]);
        stage(&root, ".epok/build");
        BuildTicket::begin(&root, &build)
            .unwrap()
            .complete(&root, &build, b"linked updated")
            .unwrap();
        let rebuilt = Graph::load(&root).unwrap();
        assert!(
            rebuilt.nodes["executable:.epok/build/epok.ps-exe"]
                .stale
                .is_empty()
        );
        assert!(!rebuilt.nodes["export:exports/test"].stale.is_empty());
    }
    #[test]
    fn manifests_record_unchanged_writes_and_exclude_old_or_outside_files() {
        let root = fixture();
        let build = stage(&root, ".epok/build");
        let old = build.join("scripts/Old.cpp");
        std::fs::write(&old, b"untracked old output").unwrap();
        let capture = Capture::begin(&build).unwrap();
        let current = build.join("scripts/Probe.cpp");
        let bytes = std::fs::read(&current).unwrap();
        crate::project::write_changed(&current, &bytes).unwrap();
        crate::project::write_changed(&root.join("outside.txt"), b"outside").unwrap();
        assert_eq!(
            capture.finish(),
            Files::from([("scripts/Probe.cpp".into(), crate::assets::hash(&bytes))])
        );
        {
            let _aborted = Capture::begin(&build).unwrap();
        }
        assert!(Capture::begin(&build).unwrap().finish().is_empty());
        std::fs::remove_file(root.join("assets/scripts/Probe.cpp")).unwrap();
        stage(&root, ".epok/build");
        let graph = Graph::load(&root).unwrap();
        assert!(
            !graph.nodes["stage:.epok/build"]
                .dependencies
                .iter()
                .any(|id| id.ends_with("/Probe.cpp") || id.ends_with("/Old.cpp"))
        );
        assert!(
            !graph.nodes["staged-file:.epok/build/scripts/Probe.cpp"]
                .stale
                .is_empty()
        );
        assert_eq!(std::fs::read(&old).unwrap(), b"untracked old output");
        assert!(
            current.exists(),
            "old files may remain for navigation but cannot enter the manifest"
        );
    }
    #[test]
    fn stale_compilation_and_tampered_inputs_cannot_certify_an_executable() {
        let root = fixture();
        let build = stage(&root, ".epok/build");
        BuildTicket::begin(&root, &build)
            .unwrap()
            .complete(&root, &build, b"original")
            .unwrap();
        std::fs::write(build.join("scripts/Probe.cpp"), b"unexpected staged edit").unwrap();
        assert!(
            BuildTicket::begin(&root, &build)
                .err()
                .unwrap()
                .contains("changed after generation")
        );
        assert!(
            !Graph::load(&root).unwrap().nodes["executable:.epok/build/epok.ps-exe"]
                .stale
                .is_empty()
        );
        stage(&root, ".epok/build");
        let old = BuildTicket::begin(&root, &build).unwrap();
        patch(
            &root,
            &build,
            Files::new(),
            "different compiler options".into(),
        )
        .unwrap();
        BuildTicket::begin(&root, &build)
            .unwrap()
            .complete(&root, &build, b"newer build")
            .unwrap();
        assert!(
            old.complete(&root, &build, b"old candidate")
                .unwrap_err()
                .contains("changed during compilation")
        );
        let graph = Graph::load(&root).unwrap();
        let exe = &graph.nodes["executable:.epok/build/epok.ps-exe"];
        assert!(exe.stale.is_empty());
        assert_eq!(exe.signature, Some(crate::assets::hash(b"newer build")));
        let pending = BuildTicket::begin(&root, &build).unwrap();
        std::fs::write(root.join("assets/scripts/Probe.cpp"), b"int probe=4;\n").unwrap();
        assert!(
            pending
                .complete(&root, &build, b"obsolete compile")
                .is_err()
        );
        assert!(
            !Graph::load(&root).unwrap().nodes["executable:.epok/build/epok.ps-exe"]
                .stale
                .is_empty()
        );
    }

    /// Actor overrides and the scene script are part of the scene document, so they are
    /// part of the staged build inputs: changing one invalidates an in-flight build the
    /// same way an entity edit does, and a byte-identical re-save does not.
    #[test]
    fn actor_overrides_and_the_scene_script_are_staged_build_inputs() {
        use crate::actor_document::{ActorInstance, ClassReference, SceneScript};
        use crate::object_model as om;
        use crate::scene_dependencies::Input;
        let root = fixture();
        let path = root.join("assets/scenes/Actors.epokmap");
        let scene = crate::scene::Scene {
            name: "Actors".into(),
            ..Default::default()
        };
        scene.save(&path).unwrap();
        let build = root.join(".epok/build");
        crate::project::stage_with_origin(
            &root,
            &scene,
            &build,
            &Input::load(&path).unwrap().origin,
        )
        .unwrap();
        patch(&root, &build, Files::new(), "release".into()).unwrap();

        // A byte-identical re-save changes nothing: an unrelated touch is not staleness.
        let pending = BuildTicket::begin(&root, &build).unwrap();
        scene.save(&path).unwrap();
        pending.complete(&root, &build, b"unchanged").unwrap();

        // An actor property override is a document change and invalidates the build.
        let mut actor = ActorInstance::new(
            uuid::Uuid::parse_str("7d0f1c02-9f3a-4c0e-9d3a-6b1c2f7d4a11").unwrap(),
            ClassReference::new("epok::Actor3D", om::ACTOR3D_ID),
            "Hero",
        );
        actor
            .properties
            .insert("speed".into(), serde_json::json!(1.0));
        actor.overrides.insert("speed".into());
        let mut edited = scene.clone();
        edited.actors = vec![actor.clone()];
        let pending = BuildTicket::begin(&root, &build).unwrap();
        edited.save(&path).unwrap();
        assert!(
            pending
                .complete(&root, &build, b"stale actor override")
                .is_err()
        );
        assert!(
            Graph::load(&root).unwrap().nodes["stage:.epok/build"]
                .stale
                .contains_key("scene-file:assets/scenes/Actors.epokmap")
        );

        // An actor whose class the reflected catalog does not know is a cook
        // diagnostic, not a silently empty table: this fixture has no object model.
        crate::project::stage_with_origin(
            &root,
            &edited,
            &build,
            &Input::load(&path).unwrap().origin,
        )
        .unwrap_err();
        // The scene script is part of the document signature too.
        let mut scripted = scene.clone();
        scripted.scene_script = Some(SceneScript {
            parent: ClassReference::new("epok::SceneScriptActor", om::SCENE_SCRIPT_ACTOR_ID),
            blueprint: crate::blueprint_asset::BlueprintAsset::new(
                "MapScript".into(),
                om::SCENE_SCRIPT_ACTOR_ID.into(),
            ),
        });
        assert_ne!(
            crate::scene_dependencies::signature(&scripted),
            crate::scene_dependencies::signature(&scene)
        );
        assert_ne!(
            crate::scene_dependencies::signature(&edited),
            crate::scene_dependencies::signature(&scene)
        );
        // The lighting bake is derived, so it still never participates.
        let mut baked = scene.clone();
        baked.bake = Some(crate::lighting::bake(&baked).unwrap());
        assert_eq!(
            crate::scene_dependencies::signature(&baked),
            crate::scene_dependencies::signature(&scene)
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
