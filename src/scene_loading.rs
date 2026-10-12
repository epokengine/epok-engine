//! Scene I/O never borrows the active editor or runs on its frame thread.
use crate::{assets, blueprint, scene::Scene};
use std::{
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver},
    time::Instant,
};

pub struct Message {
    pub time: String,
    pub text: String,
}
impl Message {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            time: chrono::Local::now()
                .format("%Y-%m-%d %H:%M:%S%.3f")
                .to_string(),
            text: text.into(),
        }
    }
}

/// Capture wall time at the producer, and durations with a monotonic clock.
pub struct Progress {
    started: Instant,
    previous: Option<(&'static str, Instant)>,
}
impl Progress {
    pub fn new() -> Self {
        Self {
            started: Instant::now(),
            previous: None,
        }
    }
    pub fn stage(&mut self, label: &'static str, emit: &mut impl FnMut(Message)) {
        self.complete_stage(emit);
        self.previous = Some((label, Instant::now()));
        emit(Message::new(label));
    }
    fn complete_stage(&mut self, emit: &mut impl FnMut(Message)) {
        if let Some((label, started)) = self.previous.take() {
            emit(Message::new(format!(
                "{label}: {:.1} ms",
                started.elapsed().as_secs_f64() * 1000.
            )));
        }
    }
    pub fn finish(&mut self, label: &str, emit: &mut impl FnMut(Message)) {
        self.complete_stage(emit);
        emit(Message::new(format!(
            "{label}: {:.1} ms total",
            self.started.elapsed().as_secs_f64() * 1000.
        )));
    }
}

pub struct Prepared {
    pub path: PathBuf,
    pub scene: Scene,
    pub assets: assets::Index,
    pub cache: assets::ScanCache,
    pub dirty: bool,
    pub bake_current: bool,
}
impl Prepared {
    pub fn load(
        root: &Path,
        path: PathBuf,
        display_size: [u16; 2],
        registry: &blueprint::Registry,
        mut cache: assets::ScanCache,
        mut emit: impl FnMut(Message),
    ) -> Result<Self, String> {
        let mut progress = Progress::new();
        progress.stage("Reading scene document", &mut emit);
        let mut scene = Scene::load_unresolved(&path)?;
        scene.display_size = display_size;
        progress.stage("Refreshing scene bindings", &mut emit);
        // Scripts belong to the project, not the scene. The source watcher owns
        // catalog refresh; navigation must not rerun the native extractor.
        let mut dirty = false;
        if scene.actors.iter().any(|a| a.blueprint_instance.is_some()) {
            let previous = scene.clone();
            if let Err(error) = crate::blueprint_asset::load_all(root).and_then(|files| {
                crate::blueprint_templates::refresh_instances(&mut scene, &files, registry)
            }) {
                emit(Message::new(format!(
                    "Blueprint instances preserved: {error}"
                )));
            }
            dirty = scene != previous;
        }
        progress.stage("Indexing scene assets", &mut emit);
        let assets = assets::scan(root, &mut cache);
        progress.stage("Resolving scene meshes", &mut emit);
        if let Err(error) = crate::mesh::resolve(&mut scene, &assets) {
            emit(Message::new(error));
        }
        if let Err(error) = crate::terrain::resolve(&mut scene, &assets) {
            emit(Message::new(error));
        }
        progress.stage("Resolving skeletal meshes", &mut emit);
        if let Err(error) = crate::skeletal::resolve(&mut scene, &assets) {
            emit(Message::new(error));
        }
        progress.stage("Resolving scene textures", &mut emit);
        if let Err(error) = crate::texture::resolve(&mut scene, &assets) {
            emit(Message::new(error));
        }
        progress.stage("Checking saved lighting", &mut emit);
        let bake_current = crate::lighting::valid_bake(&scene);
        progress.finish("Scene resources ready", &mut emit);
        Ok(Self {
            path,
            scene,
            assets,
            cache,
            dirty,
            bake_current,
        })
    }
}

pub enum Event {
    Message(Message),
    Ready(Box<Result<Prepared, String>>),
}
pub struct Loading {
    pub events: Receiver<Event>,
    worker: Option<std::thread::JoinHandle<()>>,
    pub stage: String,
    pub started: Instant,
}
impl Loading {
    pub fn start(
        root: PathBuf,
        path: PathBuf,
        display_size: [u16; 2],
        registry: blueprint::Registry,
        cache: assets::ScanCache,
    ) -> Result<Self, String> {
        let (tx, events) = mpsc::channel();
        let started = Instant::now();
        let worker = std::thread::Builder::new()
            .name("epok-scene-loading".into())
            .spawn(move || {
                let result =
                    Prepared::load(&root, path, display_size, &registry, cache, |message| {
                        let _ = tx.send(Event::Message(message));
                    });
                let _ = tx.send(Event::Ready(Box::new(result)));
            })
            .map_err(|error| format!("Cannot start scene loader: {error}"))?;
        Ok(Self {
            events,
            worker: Some(worker),
            stage: "Opening scene".into(),
            started,
        })
    }
}
impl Drop for Loading {
    fn drop(&mut self) {
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh;
    use std::{fs, sync::Arc};

    #[test]
    fn scene_switch_reuses_meshes_and_tracks_live_asset_changes() {
        let root = crate::workspace::tests::temp("scene-resource-cache");
        fs::create_dir_all(root.join("assets/scenes")).unwrap();
        let mut doc = mesh::Document::default();
        doc.primitive(
            "Plane",
            [0.; 3],
            [2.; 3],
            1,
            doc.groups[0].id,
            doc.materials[0].id,
        );
        let asset = mesh::create(&root, "assets/mesh.epokasset", &doc).unwrap();
        let path = root.join("assets/scenes/Map.epokmap");
        let mut scene = Scene::default();
        scene.actors[1].editable_mesh = Some(mesh::Component::new(asset));
        scene.save(&path).unwrap();
        let source = fs::read(&path).unwrap();
        let registry = blueprint::Registry::new();
        let first = Prepared::load(
            &root,
            path.clone(),
            [320, 240],
            &registry,
            Default::default(),
            |_| {},
        )
        .unwrap();
        let second = Prepared::load(
            &root,
            path.clone(),
            [320, 240],
            &registry,
            first.cache.clone(),
            |_| {},
        )
        .unwrap();
        let a = first.scene.actors[1]
            .editable_mesh
            .as_ref()
            .unwrap()
            .document
            .as_ref()
            .unwrap();
        let b = second.scene.actors[1]
            .editable_mesh
            .as_ref()
            .unwrap()
            .document
            .as_ref()
            .unwrap();
        assert!(
            Arc::ptr_eq(a, b),
            "unchanged resources shared across scene switches"
        );
        assert!(!second.dirty);
        assert_eq!(fs::read(&path).unwrap(), source);
        let mut edited = (**a).clone();
        for vertex in &mut edited.vertices {
            vertex[1] += 0.5;
        }
        mesh::save(first.assets.resolve(asset).unwrap(), &edited).unwrap();
        let third = Prepared::load(
            &root,
            path.clone(),
            [320, 240],
            &registry,
            second.cache,
            |_| {},
        )
        .unwrap();
        assert_eq!(
            third.scene.actors[1]
                .editable_mesh
                .as_ref()
                .unwrap()
                .document
                .as_deref(),
            Some(&edited)
        );
        assert_eq!(a.vertices[0][1], 0., "existing snapshots are immutable");
        fs::write(&path, "invalid: [").unwrap();
        assert!(Prepared::load(&root, path, [320, 240], &registry, third.cache, |_| {}).is_err());
        fs::remove_dir_all(root).unwrap();
    }
}
