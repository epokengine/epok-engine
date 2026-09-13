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
        for binding in scene.entities.iter_mut().filter_map(|e| e.script.as_mut()) {
            registry.upgrade_binding(binding);
        }
        let previous = scene.clone();
        if let Err(error) = crate::blueprint_asset::load_all(root).and_then(|files| {
            crate::blueprint_templates::refresh_instances(&mut scene, &files, registry)
        }) {
            emit(Message::new(format!(
                "Blueprint instances preserved: {error}"
            )));
        }
        let dirty = scene != previous;
        progress.stage("Indexing scene assets", &mut emit);
        let assets = assets::scan(root, &mut cache);
        progress.stage("Resolving scene meshes", &mut emit);
        if let Err(error) = crate::mesh::resolve(&mut scene, &assets) {
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
        progress.finish("Scene resources ready", &mut emit);
        Ok(Self {
            path,
            scene,
            assets,
            cache,
            dirty,
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
                let result = Prepared::load(&root, path, display_size, &registry, cache, |message| {
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
