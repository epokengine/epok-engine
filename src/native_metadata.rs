//! Host provenance for reads made by the existing native catalog and extractor.
//! These records never resolve classes, parse C++, or replace reflection caches.
use crate::{artifact_dependencies, staging_files::Files};
use std::path::{Component, Path, PathBuf};

pub const FILES: &str = "native-metadata:files";
const OPTIONS: &str = "native-metadata:reflection-options";
const FILE: &str = "native-metadata:file:";

pub(crate) fn file_key(root: &Path, path: &Path) -> Result<String, String> {
    let root = std::fs::canonicalize(root).map_err(|e| e.to_string())?;
    let path = std::fs::canonicalize(path).map_err(|e| e.to_string())?;
    let location = match path.strip_prefix(&root) {
        Ok(relative) => format!("project:{}", relative.to_string_lossy().replace('\\', "/")),
        Err(_) => format!("external:{}", path.to_string_lossy().replace('\\', "/")),
    };
    Ok(format!("{FILE}{location}"))
}

pub fn file(root: &Path, path: &Path, signature: String) -> Result<(), String> {
    crate::staging_files::consumed(Files::from([(file_key(root, path)?, signature)]))
}

fn metadata_set(root: &Path, files: &[PathBuf]) -> Result<String, String> {
    let mut keys = crate::scripts::metadata_files(files)
        .map(|path| file_key(root, path))
        .collect::<Result<Vec<_>, _>>()?;
    keys.sort();
    Ok(crate::scene_dependencies::hash(keys))
}

pub fn catalog_files(root: &Path, files: &[PathBuf]) -> Result<(), String> {
    crate::staging_files::consumed(Files::from([(FILES.into(), metadata_set(root, files)?)]))
}

fn options(config: &crate::project::Config) -> String {
    crate::scene_dependencies::hash((
        &config.nugget,
        &config.toolchain_bin,
        &config.libclang,
        crate::reflection_schema::SCHEMA_VERSION,
        crate::reflection_schema::CLANG_VERSION,
    ))
}

pub fn reflection(
    root: &Path,
    config: &crate::project::Config,
    inputs: impl IntoIterator<Item = (PathBuf, String)>,
) -> Result<(), String> {
    let mut captured = Files::from([(OPTIONS.into(), options(config))]);
    for (path, signature) in inputs {
        let key = file_key(root, &path)?;
        if captured.get(&key).is_some_and(|old| old != &signature) {
            return Err(format!("Reflection input {key} changed during extraction"));
        }
        captured.insert(key, signature);
    }
    crate::staging_files::consumed(captured)
}

fn input_path(root: &Path, location: &str) -> Result<PathBuf, String> {
    if let Some(path) = location.strip_prefix("project:") {
        let path = Path::new(path);
        if path.as_os_str().is_empty()
            || path
                .components()
                .any(|p| !matches!(p, Component::Normal(_)))
        {
            return Err("Invalid project metadata path".into());
        }
        let path = std::fs::canonicalize(root.join(path)).map_err(|e| e.to_string())?;
        if !path.starts_with(std::fs::canonicalize(root).map_err(|e| e.to_string())?) {
            return Err("Project metadata input now points outside the project".into());
        }
        Ok(path)
    } else if let Some(path) = location.strip_prefix("external:") {
        let path = PathBuf::from(path);
        if !path.is_absolute() {
            return Err("External metadata input requires an absolute host path".into());
        }
        Ok(path)
    } else {
        Err("Unknown native metadata input location".into())
    }
}

pub fn observe(root: &Path) -> Result<(), String> {
    let graph = artifact_dependencies::Graph::load(root)?;
    let mut observed = std::collections::BTreeMap::new();
    for key in graph.nodes.keys() {
        let value = if key == FILES {
            crate::reflection::script_files(root).and_then(|files| metadata_set(root, &files))
        } else if key == OPTIONS {
            crate::project::Config::load(root).map(|config| options(&config))
        } else if let Some(location) = key.strip_prefix(FILE) {
            input_path(root, location).and_then(|path| {
                std::fs::read(path)
                    .map(|bytes| crate::assets::hash(&bytes))
                    .map_err(|e| e.to_string())
            })
        } else {
            continue;
        };
        observed.insert(key.clone(), value);
    }
    artifact_dependencies::transaction(root, |graph| {
        for (key, result) in observed {
            match result {
                Ok(signature) => graph.publish(&key, signature, Default::default()),
                Err(error) => graph.invalidate(&key, &error),
            }
        }
    })?;
    Ok(())
}

type ExternalInputs =
    std::collections::BTreeMap<String, (artifact_dependencies::Node, Result<String, String>)>;

/// Read the compiler/extractor's recorded external inputs, never scan or parse
/// another source tree. Large SDK and tool binaries are hashed off the UI thread.
fn external_snapshot(root: &Path) -> Result<ExternalInputs, String> {
    artifact_dependencies::Graph::load(root)?
        .nodes
        .into_iter()
        .filter(|(key, _)| key.starts_with(&format!("{FILE}external:")))
        .map(|(key, node)| {
            let value = input_path(root, key.strip_prefix(FILE).unwrap()).and_then(|path| {
                std::fs::read(&path)
                    .map(|bytes| crate::assets::hash(&bytes))
                    .map_err(|error| format!("{}: {error}", path.display()))
            });
            Ok((key, (node, value)))
        })
        .collect()
}

#[derive(Default)]
pub struct ExternalWatch {
    previous: std::collections::BTreeMap<String, Result<String, String>>,
    pending: Option<std::sync::mpsc::Receiver<Result<ExternalInputs, String>>>,
    last_started: Option<std::time::Instant>,
    watch: Option<crate::file_watch::Watch>,
    refresh_requested: bool,
}

fn watched_paths<'a>(keys: impl Iterator<Item = &'a String>) -> Vec<PathBuf> {
    let mut paths: Vec<_> = keys
        .filter_map(|key| {
            key.strip_prefix(&format!("{FILE}external:"))
                .map(PathBuf::from)
        })
        .collect();
    // Host configuration participates in the project fingerprint but lives
    // outside its folder. Include absent files to catch creation/replacement.
    let home = crate::workspace::editor_home();
    paths.extend([
        home.join("Editor.epokconfig"),
        home.join("Local.epokconfig"),
    ]);
    paths
}

impl ExternalWatch {
    /// Opening a project establishes an observation baseline. Cached provenance
    /// may be stale, but it is not a new edit made during this editor session.
    pub fn primed(root: &Path) -> Result<Self, String> {
        let graph = artifact_dependencies::Graph::load(root)?;
        let watch = crate::file_watch::Watch::files(watched_paths(graph.nodes.keys()));
        Ok(Self {
            watch: Some(watch),
            previous: external_snapshot(root)?
                .into_iter()
                .map(|(key, (_, value))| (key, value))
                .collect(),
            ..Default::default()
        })
    }

    pub fn request_refresh(&mut self) {
        self.refresh_requested = true;
    }

    #[cfg(test)]
    pub fn last_started_for_test(&self) -> Option<std::time::Instant> {
        self.last_started
    }

    pub fn take_warning(&mut self) -> Option<String> {
        self.watch
            .as_mut()
            .and_then(crate::file_watch::Watch::take_warning)
    }

    fn accept(
        &mut self,
        root: &Path,
        target: &str,
        inputs: ExternalInputs,
    ) -> Result<bool, String> {
        let mut changed = std::collections::BTreeSet::new();
        let graph = artifact_dependencies::transaction(root, |graph| {
            for (key, (baseline, value)) in inputs {
                let different = self.previous.get(&key).map_or_else(
                    || match &value {
                        Ok(signature) => baseline.signature.as_ref() != Some(signature),
                        Err(_) => baseline.stale.is_empty(),
                    },
                    |previous| previous != &value,
                );
                if different {
                    changed.insert(key.clone());
                }
                self.previous.insert(key.clone(), value.clone());
                // A build or another observation may have published while the
                // worker read files. Never replace that newer provenance with
                // this snapshot; the next observation will read it again.
                if graph.nodes.get(&key) != Some(&baseline) {
                    continue;
                }
                match value {
                    Ok(signature) => graph.publish(&key, signature, Default::default()),
                    Err(error) => graph.invalidate(&key, &error),
                }
            }
        })?;
        Ok([
            format!("stage:{target}"),
            format!("native-build:{target}"),
            format!("executable:{target}/epok.ps-exe"),
        ]
        .iter()
        .any(|consumer| graph.depends_on_any(consumer, &changed)))
    }

    /// At most one background reader per editor. Completion is polled without
    /// waiting; repeated missing inputs only report a change once, then on repair.
    pub fn poll(&mut self, root: &Path, target: &str) -> Result<Option<bool>, String> {
        let mut affected = None;
        if let Some(receiver) = &self.pending {
            match receiver.try_recv() {
                Ok(snapshot) => {
                    self.pending = None;
                    let snapshot = snapshot?;
                    if let Some(watch) = &mut self.watch {
                        // Close the read/subscription race with one follow-up
                        // read after installing newly discovered dependencies.
                        self.refresh_requested |= watch.set_files(watched_paths(snapshot.keys()));
                    }
                    affected = Some(self.accept(root, target, snapshot)?);
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => return Ok(None),
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.pending = None;
                    return Err("External native input observation worker stopped".into());
                }
            }
        }
        if self.watch.is_none() {
            let graph = artifact_dependencies::Graph::load(root)?;
            self.watch = Some(crate::file_watch::Watch::files(watched_paths(
                graph.nodes.keys(),
            )));
            self.refresh_requested |= self.last_started.is_none();
        }
        if self.watch.as_mut().is_some_and(|watch| watch.poll()) {
            self.refresh_requested = true;
        }
        if self.refresh_requested
            && self
                .last_started
                .is_none_or(|time| time.elapsed() >= std::time::Duration::from_millis(300))
        {
            let root = root.to_path_buf();
            let (sender, receiver) = std::sync::mpsc::channel();
            std::thread::Builder::new()
                .name("epok-external-inputs".into())
                .spawn(move || {
                    let _ = sender.send(external_snapshot(&root));
                })
                .map_err(|error| format!("Cannot observe external native inputs: {error}"))?;
            self.pending = Some(receiver);
            self.last_started = Some(std::time::Instant::now());
            self.refresh_requested = false;
        }
        Ok(affected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        artifact_dependencies::Graph,
        staging_files::{BuildTicket, Capture},
    };

    #[test]
    fn external_native_event_wakes_validation_and_idle_does_not_hash_again() {
        let root = fixture();
        let external_root = fixture();
        let path = external_root.join("Shared.hpp");
        std::fs::write(&path, b"old").unwrap();
        let key = file_key(&root, &path).unwrap();
        artifact_dependencies::transaction(&root, |graph| {
            graph.publish(&key, crate::assets::hash(b"old"), Default::default());
            graph.publish("native-build:used", "build".into(), [key].into());
        })
        .unwrap();
        let mut watch = ExternalWatch::primed(&root).unwrap();
        for _ in 0..5 {
            assert_eq!(watch.poll(&root, "used").unwrap(), None);
        }
        assert!(watch.last_started.is_none());
        std::fs::write(&path, b"new").unwrap();
        let started = std::time::Instant::now();
        loop {
            if watch.poll(&root, "used").unwrap() == Some(true) {
                break;
            }
            assert!(
                started.elapsed().as_secs() < 8,
                "External file event was not processed"
            );
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let read = watch.last_started;
        for _ in 0..110 {
            assert_eq!(watch.poll(&root, "used").unwrap(), None);
            assert_eq!(watch.last_started, read);
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        drop(watch);
        std::fs::remove_dir_all(root).unwrap();
        std::fs::remove_dir_all(external_root).unwrap();
    }

    fn fixture() -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("epok-native-metadata-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(root.join("assets/scripts")).unwrap();
        root
    }
    #[test]
    fn opening_external_inputs_baselines_old_cache_without_hiding_later_edits() {
        let root = fixture();
        let external = fixture().join("Shared.hpp");
        std::fs::write(&external, b"current").unwrap();
        let key = file_key(&root, &external).unwrap();
        artifact_dependencies::transaction(&root, |graph| {
            graph.publish(
                &key,
                crate::assets::hash(b"previous-session"),
                Default::default(),
            );
            graph.publish("native-build:used", "build".into(), [key.clone()].into());
        })
        .unwrap();
        let mut watch = ExternalWatch::primed(&root).unwrap();
        assert!(
            !watch
                .accept(&root, "used", external_snapshot(&root).unwrap())
                .unwrap()
        );
        assert!(
            !Graph::load(&root).unwrap().nodes["native-build:used"]
                .stale
                .is_empty()
        );
        std::fs::write(&external, b"edited after opening").unwrap();
        assert!(
            watch
                .accept(&root, "used", external_snapshot(&root).unwrap())
                .unwrap()
        );
        std::fs::remove_file(external).unwrap();
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn external_watch_pending_read_is_not_a_successful_recovery() {
        let root = fixture();
        let (sender, receiver) = std::sync::mpsc::channel();
        let mut watch = ExternalWatch {
            pending: Some(receiver),
            last_started: Some(std::time::Instant::now()),
            ..Default::default()
        };
        assert_eq!(watch.poll(&root, "used").unwrap(), None);
        sender
            .send(Err("Cannot read dependency graph".into()))
            .unwrap();
        assert_eq!(
            watch.poll(&root, "used").unwrap_err(),
            "Cannot read dependency graph"
        );
        assert_eq!(watch.poll(&root, "used").unwrap(), None);
        let (sender, receiver) = std::sync::mpsc::channel();
        watch.pending = Some(receiver);
        sender.send(Ok(Default::default())).unwrap();
        assert_eq!(watch.poll(&root, "used").unwrap(), Some(false));
    }
    #[test]
    fn external_watch_follows_consumers_and_detects_deletion_repair_and_preserved_times() {
        let root = fixture();
        let external = fixture().join("Shared.hpp");
        std::fs::write(&external, b"old").unwrap();
        let key = file_key(&root, &external).unwrap();
        assert!(key.starts_with(&format!("{FILE}external:")));
        artifact_dependencies::transaction(&root, |graph| {
            graph.publish(&key, crate::assets::hash(b"old"), Default::default());
            graph.publish("native-build:used", "build".into(), [key.clone()].into());
            graph.publish("native-build:other", "other".into(), Default::default());
        })
        .unwrap();
        let mut watch = ExternalWatch::default();
        assert!(
            !watch
                .accept(&root, "used", external_snapshot(&root).unwrap())
                .unwrap()
        );
        let modified = std::fs::metadata(&external).unwrap().modified().unwrap();
        std::fs::write(&external, b"new").unwrap();
        std::fs::File::options()
            .write(true)
            .open(&external)
            .unwrap()
            .set_times(std::fs::FileTimes::new().set_modified(modified))
            .unwrap();
        let snapshot = external_snapshot(&root).unwrap();
        let mut other = ExternalWatch::default();
        assert!(!other.accept(&root, "other", snapshot.clone()).unwrap());
        // Another observer already published: raw observation history must
        // still stop the old running consumer.
        assert!(watch.accept(&root, "used", snapshot).unwrap());
        assert!(
            !watch
                .accept(&root, "used", external_snapshot(&root).unwrap())
                .unwrap()
        );
        let graph = Graph::load(&root).unwrap();
        assert!(!graph.nodes["native-build:used"].stale.is_empty());
        assert!(graph.nodes["native-build:other"].stale.is_empty());
        std::fs::remove_file(&external).unwrap();
        assert!(
            watch
                .accept(&root, "used", external_snapshot(&root).unwrap())
                .unwrap()
        );
        assert!(
            !watch
                .accept(&root, "used", external_snapshot(&root).unwrap())
                .unwrap()
        );
        std::fs::write(&external, b"new").unwrap();
        assert!(
            watch
                .accept(&root, "used", external_snapshot(&root).unwrap())
                .unwrap()
        );
        assert!(Graph::load(&root).unwrap().nodes[&key].stale.is_empty());
        assert!(
            !Graph::load(&root).unwrap().nodes["native-build:used"]
                .stale
                .is_empty()
        );

        let stale_snapshot = external_snapshot(&root).unwrap();
        artifact_dependencies::transaction(&root, |graph| {
            graph.publish(&key, "newer-build-input".into(), Default::default());
        })
        .unwrap();
        watch.accept(&root, "used", stale_snapshot).unwrap();
        assert_eq!(
            Graph::load(&root).unwrap().nodes[&key].signature.as_deref(),
            Some("newer-build-input")
        );
        // A fresh watcher also catches edits that precede its first observation.
        assert!(
            ExternalWatch::default()
                .accept(&root, "used", external_snapshot(&root).unwrap())
                .unwrap()
        );
    }
    fn stage(root: &Path, target: &str) -> PathBuf {
        let destination = root.join(target);
        let path = root.join("assets/scenes/Main.epokmap");
        if !path.exists() {
            crate::scene::Scene::default().save(&path).unwrap();
        }
        let input = crate::scene_dependencies::Input::load(&path).unwrap();
        crate::project::stage_with_origin(root, &input.scene, &destination, &input.origin).unwrap();
        crate::staging_files::patch(root, &destination, Files::new(), "release".into()).unwrap();
        destination
    }
    #[test]
    fn native_defaults_and_class_membership_require_fresh_scene_staging() {
        let root = fixture();
        let path = root.join("assets/scripts/Probe.hpp");
        let write = |name: &str, id: &str, value: i32| {
            std::fs::write(root.join(format!("assets/scripts/{name}.hpp")),format!("#pragma once\n#include \"epok.hpp\"\nclass EPOK_CLASS(Blueprintable,Id=\"{id}\") {name}:public epok::ActorComponent {{public:EPOK_PROPERTY(EditAnywhere,Id=\"dbd7f48f-c5ea-4d84-902b-e00fd56acb22\") int32_t health={value};}};\n")).unwrap();
        };
        let probe = "11f9d403-d033-4e73-8b37-938c9aca27cc";
        write("Probe", probe, 50);
        let build = stage(&root, ".epok/build");
        let export = stage(&root, "exports/metadata");
        let pending = BuildTicket::begin(&root, &build).unwrap();
        write("Probe", probe, 75);
        assert!(pending.complete(&root, &build, b"old defaults").is_err());
        assert!(crate::staging_files::publish_export(&root, &export, Files::new()).is_err());
        stage(&root, ".epok/build");
        let pending = BuildTicket::begin(&root, &build).unwrap();
        std::fs::write(root.join("assets/scripts/Extra.hpp"),"#pragma once\n#include \"epok.hpp\"\nclass EPOK_CLASS(Blueprintable,Id=\"eb0b8687-2a9d-4e11-ace1-57e8f694b4f3\") Extra:public epok::ActorComponent {};\n").unwrap();
        assert!(pending.complete(&root, &build, b"old membership").is_err());
        stage(&root, ".epok/build");
        std::fs::remove_file(root.join("assets/scripts/Extra.hpp")).unwrap();
        assert!(BuildTicket::begin(&root, &build).is_err());
        stage(&root, ".epok/build");
        std::fs::write(&path, "invalid C++ {").unwrap();
        assert!(BuildTicket::begin(&root, &build).is_err());
        write("Probe", probe, 100);
        assert!(BuildTicket::begin(&root, &build).is_err());
        stage(&root, ".epok/build");
        let capture = Capture::begin(&root.join("exports/conflict")).unwrap();
        crate::scripts::native_catalog(&root).unwrap();
        write("Probe", probe, 120);
        assert!(crate::scripts::native_catalog(&root).is_err());
        drop(capture);
        stage(&root, ".epok/build");
        BuildTicket::begin(&root, &build)
            .unwrap()
            .complete(&root, &build, b"current")
            .unwrap();
        assert!(
            !Graph::load(&root).unwrap().nodes["stage:exports/metadata"]
                .stale
                .is_empty()
        );
        stage(&root, "exports/metadata");
        crate::staging_files::publish_export(&root, &export, Files::new()).unwrap();
    }

    #[test]
    #[ignore = "requires the pinned host extractor and PSX SDK; does not launch the emulator"]
    fn reflection_capture_rechecks_external_includes_tools_and_options() {
        let parent = fixture();
        let root = parent.join("Game");
        std::fs::create_dir_all(root.join("assets/scripts")).unwrap();
        let include = parent.join("ExternalDefaults.inc");
        std::fs::write(&include, b"#define EPOK_TEST_HEALTH 25\n").unwrap();
        let header = format!(
            "#pragma once\n#include \"epok.hpp\"\n#include \"{}\"\nclass EPOK_CLASS(Blueprintable,Id=\"{}\") Probe:public epok::ActorComponent {{\npublic:\n EPOK_PROPERTY(EditAnywhere,Id=\"{}\") uint32_t health=EPOK_TEST_HEALTH;\n void tick(epok::Fixed) override {{}}\n}};\n",
            include.to_string_lossy().replace('\\', "/"),
            uuid::Uuid::new_v4(),
            uuid::Uuid::new_v4()
        );
        std::fs::write(root.join("assets/scripts/Probe.hpp"), header).unwrap();
        std::fs::write(
            root.join("assets/scripts/Probe.cpp"),
            b"#include \"Probe.hpp\"\n",
        )
        .unwrap();
        let build = stage(&root, ".epok/build");
        let export = stage(&root, "exports/reflection");
        let key = file_key(&root, &include).unwrap();
        let initial = Graph::load(&root).unwrap();
        assert!(initial.depends_on_any("stage:.epok/build", &[key.clone()].into_iter().collect()));
        for suffix in [
            "common.mk",
            "psyqo.mk",
            "epok-header-tool.exe",
            "mipsel-none-elf-g++.exe",
            "libclang.dll",
        ] {
            if cfg!(windows) || !suffix.ends_with(".exe") && suffix != "libclang.dll" {
                assert!(
                    initial
                        .nodes
                        .keys()
                        .any(|key| key.starts_with(FILE) && key.ends_with(suffix)),
                    "{suffix}"
                );
            }
        }
        let pending = BuildTicket::begin(&root, &build).unwrap();
        std::fs::write(&include, b"#define EPOK_TEST_HEALTH 50\n").unwrap();
        assert!(
            pending
                .complete(&root, &build, b"old included default")
                .is_err()
        );
        assert!(
            Graph::load(&root).unwrap().nodes["stage:.epok/build"]
                .stale
                .contains_key(&key)
        );
        assert!(crate::staging_files::publish_export(&root, &export, Files::new()).is_err());
        stage(&root, ".epok/build");
        // A valid extractor cache hit must record the same included-file inputs.
        let stable = Graph::load(&root).unwrap().nodes["stage:.epok/build"].clone();
        stage(&root, ".epok/build");
        assert_eq!(
            Graph::load(&root).unwrap().nodes["stage:.epok/build"],
            stable
        );
        let pending = BuildTicket::begin(&root, &build).unwrap();
        std::fs::remove_file(&include).unwrap();
        assert!(pending.complete(&root, &build, b"missing include").is_err());
        std::fs::write(&include, b"#define EPOK_TEST_HEALTH 50\n").unwrap();
        assert!(BuildTicket::begin(&root, &build).is_err());
        stage(&root, ".epok/build");
        let pending = BuildTicket::begin(&root, &build).unwrap();
        let mut config = crate::project::Config::load(&root).unwrap();
        config.nugget = parent.join("DifferentSdk").to_string_lossy().into();
        let local = root.join("Local.epokconfig");
        std::fs::write(&local, serde_json::to_vec(&config).unwrap()).unwrap();
        assert!(
            pending
                .complete(&root, &build, b"old SDK selection")
                .is_err()
        );
        assert!(
            Graph::load(&root).unwrap().nodes["stage:.epok/build"]
                .stale
                .contains_key(OPTIONS)
        );
        std::fs::remove_file(local).unwrap();
        assert!(BuildTicket::begin(&root, &build).is_err());
        stage(&root, ".epok/build");
        BuildTicket::begin(&root, &build)
            .unwrap()
            .complete(&root, &build, b"current metadata")
            .unwrap();
    }
}
