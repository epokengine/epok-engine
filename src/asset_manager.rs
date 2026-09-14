use crate::{
    assets::{self, Candidate, Index, Record, ScanCache},
    audio_import::Settings,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    sync::mpsc::{self, Receiver},
    time::{Duration, Instant},
};
use uuid::Uuid;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub enum Status {
    #[default]
    Pending,
    Omitted,
    Error(String),
}
#[derive(Clone, Debug)]
pub struct Pending {
    pub source: String,
    pub hash: String,
    pub existing: Option<Uuid>,
    pub status: Status,
}
impl Pending {
    pub fn key(&self) -> String {
        format!(
            "{}:{}:{}",
            self.source,
            self.hash,
            self.existing.map(|id| id.to_string()).unwrap_or_default()
        )
    }
}
#[derive(Default, Serialize, Deserialize)]
struct Decisions {
    entries: BTreeMap<String, Status>,
}
pub struct ImportForm {
    pub model: bool,
    pub texture: bool,
    pub source: String,
    pub destination: String,
    pub settings: Settings,
    pub sequence: Option<crate::sequence::Settings>,
    pub bank: Option<crate::sound_bank::Settings>,
    pub bank_companion: String,
    pub sequence_catalog: Option<Result<crate::sequence::SourceCatalog, String>>,
    pub existing: Option<Record>,
    pub snapshot: bool,
    pub queue_key: Option<String>,
}

enum Prepared {
    Audio(Candidate),
    ReferenceBank(Candidate),
    SonyBank(crate::bank_compat::Candidate),
    Starter(Vec<Candidate>),
    Model(crate::model_import::Candidate),
}
impl Prepared {
    fn commit(self) -> Result<uuid::Uuid, String> {
        match self {
            Self::Audio(c) | Self::ReferenceBank(c) => assets::commit(c),
            Self::SonyBank(c) => crate::bank_compat::commit(c),
            Self::Starter(candidates) => crate::sound_bank::publish_starter(candidates),
            Self::Model(c) => crate::model_import::commit(c),
        }
    }
}
type ScanResult = (u64, Index, ScanCache, Vec<Candidate>);
pub struct Manager {
    pub root: PathBuf,
    pub index: Index,
    pub pending: Vec<Pending>,
    pub window: bool,
    pub notification: bool,
    pub form: Option<ImportForm>,
    pub open_dialog: bool,
    pub selected: Option<Uuid>,
    pub focus_tab: Option<usize>,
    pub operation_path: String,
    pub folder: String,
    pub error: Option<String>,
    pub messages: Vec<String>,
    pub music_conversion: crate::music_conversion_ui::State,
    pub refresh_editor: bool,
    pub busy: bool,
    pub last_deleted: Option<PathBuf>,
    decisions: Decisions,
    announced: BTreeSet<String>,
    observed: BTreeMap<String, (String, Instant)>,
    cache: ScanCache,
    scan: Option<Receiver<ScanResult>>,
    worker: Option<Receiver<Result<Prepared, String>>>,
    worker_key: Option<String>,
    generation: u64,
    watch: crate::file_watch::Watch,
    pub revision: u64,
    scan_requested: bool,
    settle_at: Option<Instant>,
}
impl Manager {
    pub fn new(root: PathBuf) -> Self {
        let decisions = std::fs::read(root.join("UserSettings/ImportState.epokprefs"))
            .map_err(|e| e.to_string())
            .and_then(|b| crate::document::from_slice(&b).map_err(|e| e.to_string()));
        let error = if root.join("UserSettings/ImportState.epokprefs").exists() {
            decisions.as_ref().err().cloned()
        } else {
            None
        };
        Self {
            watch: crate::file_watch::Watch::project(&root),
            root,
            index: Index::default(),
            pending: vec![],
            window: false,
            notification: false,
            form: None,
            open_dialog: false,
            selected: None,
            focus_tab: None,
            operation_path: String::new(),
            folder: "assets".into(),
            error,
            messages: vec![],
            music_conversion: Default::default(),
            refresh_editor: false,
            busy: false,
            last_deleted: None,
            decisions: decisions.unwrap_or_default(),
            announced: BTreeSet::new(),
            observed: BTreeMap::new(),
            cache: ScanCache::default(),
            scan: None,
            worker: None,
            worker_key: None,
            generation: 0,
            revision: 0,
            scan_requested: true,
            settle_at: None,
        }
    }
    /// Adopt a worker-prepared index and retain its package scan cache. Any old
    /// in-flight scan is ignored by the generation check.
    pub fn adopt(&mut self, index: Index, cache: ScanCache) {
        self.generation += 1;
        self.index = index;
        self.cache = cache;
        self.scan_requested = false;
        self.refresh_editor = false;
        self.update_pending();
    }
    pub fn scan_cache(&self) -> ScanCache {
        self.cache.clone()
    }
    pub fn refresh(&mut self) {
        self.generation += 1;
        self.cache = ScanCache::default();
        self.revision = self.revision.wrapping_add(1);
        self.scan_requested = true;
    }
    pub fn tick(&mut self) {
        self.messages.append(&mut self.music_conversion.messages);
        if self.watch.poll() {
            // An OS event can report a write with preserved size/mtime. Do not
            // let the stat-based package cache hide it, or adopt an older scan.
            self.refresh();
        }
        if let Some(warning) = self.watch.take_warning() {
            self.messages.push(warning);
        }
        if self.settle_at.is_some_and(|at| Instant::now() >= at) {
            self.update_pending(); // Import debounce needs no further disk scan.
        }
        if let Some(worker) = &self.worker {
            match worker.try_recv() {
                Ok(result) => {
                    self.worker = None;
                    self.busy = false;
                    let reference = matches!(&result, Ok(Prepared::ReferenceBank(_)));
                    let result = result.and_then(Prepared::commit);
                    match result {
                        Ok(id) => {
                            self.selected = Some(id);
                            self.focus_tab = Some(2);
                            self.messages.push(format!("Imported asset {id}"));
                            if let Some(key) = self.worker_key.take() {
                                self.decisions.entries.remove(&key);
                            }
                            self.save_decisions();
                            if reference {
                                if let Some(settings) =
                                    self.form.as_mut().and_then(|f| f.sequence.as_mut())
                                {
                                    settings.sound_bank = Some(id);
                                }
                            } else {
                                self.music_conversion.cancel();
                                self.form = None;
                            }
                            self.error = None;
                            self.refresh_editor = true;
                            self.refresh();
                        }
                        Err(error) => {
                            if let Some(key) = self.worker_key.take() {
                                self.decisions
                                    .entries
                                    .insert(key, Status::Error(error.clone()));
                                self.save_decisions();
                            }
                            self.error = Some(error);
                            self.window = true;
                            self.refresh();
                        }
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.worker = None;
                    self.busy = false;
                    self.error = Some("Importer worker terminated; no asset was published.".into());
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
        if let Some(scan) = &self.scan {
            match scan.try_recv() {
                Ok((generation, index, cache, relinks)) => {
                    self.scan = None;
                    if generation == self.generation {
                        self.cache = cache;
                        self.index = index;
                        self.update_pending();
                        self.refresh_editor = true;
                        if !relinks.is_empty() && !self.busy {
                            for candidate in relinks {
                                match assets::commit(candidate) {
                                    Ok(id) => self
                                        .messages
                                        .push(format!("Recovered moved source for {id}")),
                                    Err(error) => self.error = Some(error),
                                }
                            }
                            self.refresh();
                        }
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.scan = None;
                    self.error = Some("Asset scan failed. Refresh to retry.".into());
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
        if self.scan.is_none() && self.scan_requested {
            self.scan_requested = false;
            let root = self.root.clone();
            let generation = self.generation;
            let mut cache = std::mem::take(&mut self.cache);
            let (tx, rx) = mpsc::channel();
            std::thread::spawn(move || {
                let index = assets::scan(&root, &mut cache);
                let relinks = assets::source_relinks(&root, &index);
                let _ = tx.send((generation, index, cache, relinks));
            });
            self.scan = Some(rx);
        }
    }
    fn update_pending(&mut self) {
        let now = Instant::now();
        let mut pending = Vec::new();
        for source in self.index.sources.values() {
            let observed = self
                .observed
                .entry(source.path.clone())
                .or_insert_with(|| (source.hash.clone(), now));
            if observed.0 != source.hash {
                *observed = (source.hash.clone(), now);
            }
            if now.duration_since(observed.1) < Duration::from_millis(750) {
                continue;
            }
            if self.index.usable().any(|r| {
                crate::bank_compat::parts(r)
                    .is_some_and(|p| p.iter().any(|p| p.path == source.path))
            }) {
                continue;
            }
            let linked = self
                .index
                .assets
                .values()
                .flatten()
                .filter(|r| {
                    self.index
                        .linked_source(r)
                        .ok()
                        .flatten()
                        .is_some_and(|s| s.path == source.path)
                })
                .collect::<Vec<_>>();
            if linked.is_empty() {
                pending.push(Pending {
                    source: source.path.clone(),
                    hash: source.hash.clone(),
                    existing: None,
                    status: Status::Pending,
                });
            } else {
                for record in linked {
                    if record.meta.source_hash != source.hash {
                        pending.push(Pending {
                            source: source.path.clone(),
                            hash: source.hash.clone(),
                            existing: Some(record.meta.id),
                            status: Status::Pending,
                        });
                    }
                }
            }
        }
        for record in self.index.usable() {
            let Some(parts) = crate::bank_compat::parts(record) else {
                continue;
            };
            let changed = parts.iter().any(|p| {
                self.index
                    .sources
                    .get(&p.path)
                    .is_some_and(|s| s.hash != p.sha256)
            });
            let stable = parts.iter().all(|p| {
                self.observed.get(&p.path).is_some_and(|(_, time)| {
                    now.duration_since(*time) >= Duration::from_millis(750)
                })
            });
            if changed && stable {
                let revision = parts
                    .iter()
                    .map(|p| {
                        self.index
                            .sources
                            .get(&p.path)
                            .map_or("missing", |s| s.hash.as_str())
                    })
                    .collect::<Vec<_>>()
                    .join(":");
                pending.push(Pending {
                    source: record.meta.source.clone(),
                    hash: assets::hash(revision.as_bytes()),
                    existing: Some(record.meta.id),
                    status: Status::Pending,
                });
            }
        }
        self.observed
            .retain(|path, _| self.index.sources.contains_key(path));
        self.settle_at = self
            .observed
            .values()
            .map(|(_, time)| *time + Duration::from_millis(750))
            .filter(|at| *at > now)
            .min();
        let mut keys = BTreeSet::new();
        pending.retain(|item| keys.insert(item.key()));
        for item in &mut pending {
            let key = item.key();
            item.status = self
                .decisions
                .entries
                .get(&key)
                .cloned()
                .unwrap_or_default();
            if item.status == Status::Pending && self.announced.insert(key) {
                self.notification = true;
            }
        }
        if !pending.iter().any(|item| item.status == Status::Pending) {
            self.notification = false;
        }
        self.pending = pending;
    }
    pub fn omit(&mut self, key: &str) {
        self.decisions.entries.insert(key.into(), Status::Omitted);
        self.save_decisions();
        self.update_pending();
    }
    fn save_decisions(&mut self) {
        let result = crate::document::to_vec(&self.decisions)
            .map_err(|e| e.to_string())
            .and_then(|bytes| {
                let path = self.root.join("UserSettings/ImportState.epokprefs");
                let previous = std::fs::read(&path).ok().map(|b| assets::hash(&b));
                assets::atomic_write(&path, &bytes, previous.as_deref())
            });
        if let Err(error) = result {
            self.error = Some(format!("Import preferences: {error}"));
        }
    }
    pub fn begin_pending(&mut self, item: &Pending) {
        let existing = match item.existing {
            Some(id) => match self.index.resolve(id) {
                Ok(r) => Some(r.clone()),
                Err(e) => {
                    self.error = Some(e);
                    return;
                }
            },
            None => None,
        };
        let model = item.source.to_ascii_lowercase().ends_with(".fbx");
        let detected = assets::audio_source_kind(&self.root, &item.source);
        let soundfont = crate::soundfont_asset::has_source_header(&self.root.join(&item.source));
        let destination = existing
            .as_ref()
            .map(|r| assets::path_string(&self.root, &r.path))
            .unwrap_or_else(|| {
                if model {
                    return crate::model_import::destination(&item.source);
                }
                std::path::Path::new(&item.source)
                    .with_extension("epokasset")
                    .to_string_lossy()
                    .into_owned()
            });
        self.form = Some(ImportForm {
            model,
            texture: item.source.to_ascii_lowercase().ends_with(".png"),
            source: item.source.clone(),
            destination,
            settings: existing
                .as_ref()
                .map(|r| r.meta.settings.audio().cloned().unwrap_or_default())
                .unwrap_or_default(),
            sequence: (detected == Some(assets::Kind::MusicSequence)
                || (detected.is_none()
                    && std::path::Path::new(&item.source)
                        .extension()
                        .is_some_and(|e| {
                            matches!(
                                e.to_ascii_lowercase().to_str(),
                                Some("mid" | "midi" | "seq" | "sep")
                            )
                        })))
            .then(|| {
                existing
                    .as_ref()
                    .and_then(|r| r.meta.settings.sequence().ok())
                    .cloned()
                    .unwrap_or_default()
            }),
            bank: existing
                .as_ref()
                .and_then(|r| r.meta.settings.sound_bank().ok())
                .cloned()
                .or_else(|| {
                    if soundfont {
                        Some(crate::sound_bank::Settings {
                            schema_version: 2,
                            library: Some(crate::soundfont_asset::Definition::detected(
                                crate::sf2::SourceFormat::Sf2Pcm16,
                            )),
                            ..Default::default()
                        })
                    } else {
                        (detected == Some(assets::Kind::SoundBank)
                            || (detected.is_none()
                                && crate::bank_compat::source_candidate(std::path::Path::new(
                                    &item.source,
                                ))))
                        .then(|| crate::sound_bank::Settings {
                            imported: Some(Default::default()),
                            ..Default::default()
                        })
                    }
                }),
            bank_companion: existing
                .as_ref()
                .and_then(crate::bank_compat::parts)
                .and_then(|p| p.get(1))
                .map(|p| p.path.clone())
                .unwrap_or_default(),
            sequence_catalog: None,
            existing,
            snapshot: false,
            queue_key: Some(item.key()),
        });
        self.window = true;
        self.notification = false;
        self.open_dialog = true;
    }
    pub fn begin_reimport(&mut self, record: &Record, snapshot: bool) {
        let source = self
            .index
            .linked_source(record)
            .ok()
            .flatten()
            .map_or_else(|| record.meta.source.clone(), |s| s.path.clone());
        self.form = Some(ImportForm {
            model: record.meta.kind == assets::Kind::ModelSource,
            texture: record.meta.kind == assets::Kind::Texture,
            source,
            destination: assets::path_string(&self.root, &record.path),
            settings: record.meta.settings.audio().cloned().unwrap_or_default(),
            sequence: record.meta.settings.sequence().ok().cloned(),
            bank: record.meta.settings.sound_bank().ok().cloned(),
            bank_companion: crate::bank_compat::parts(record)
                .and_then(|p| p.get(1))
                .map(|p| p.path.clone())
                .unwrap_or_default(),
            sequence_catalog: None,
            existing: Some(record.clone()),
            snapshot,
            queue_key: None,
        });
        self.window = true;
        self.open_dialog = true;
    }
    pub fn begin_new_bank(&mut self) {
        if self.busy || self.form.is_some() {
            self.error = Some("Finish the current import before creating a SoundBank".into());
            return;
        }
        self.form = Some(ImportForm {
            model: false,
            texture: false,
            source: String::new(),
            destination: format!("{}/New SoundBank.epokasset", self.folder),
            settings: Default::default(),
            sequence: None,
            bank: Some(Default::default()),
            bank_companion: String::new(),
            sequence_catalog: None,
            existing: None,
            snapshot: false,
            queue_key: None,
        });
        self.window = true;
        self.open_dialog = true;
    }
    pub fn start_starter_bank(&mut self) {
        if self.busy || self.form.is_some() {
            self.error = Some("Finish the current import before creating a starter bank".into());
            return;
        }
        let root = self.root.clone();
        let destination = format!("{}/Retro Starter.epokasset", self.folder);
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(
                crate::sound_bank::starter_candidates(&root, &destination).map(Prepared::Starter),
            );
        });
        self.worker = Some(rx);
        self.worker_key = None;
        self.busy = true;
        self.error = None;
        self.window = true;
    }
    pub fn start_reference_bank(&mut self) {
        if self.busy {
            return;
        }
        if let Some(record) = self.index.usable().find(|r| {
            r.meta.source_hash == crate::soundfont_asset::REFERENCE_HASH
                && r.meta
                    .settings
                    .sound_bank()
                    .is_ok_and(|s| s.library.is_some())
        }) {
            if let Some(settings) = self.form.as_mut().and_then(|f| f.sequence.as_mut()) {
                settings.sound_bank = Some(record.meta.id);
            }
            return;
        }
        let root = self.root.clone();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(
                crate::music_conversion_ui::reference_candidate(&root).map(Prepared::ReferenceBank),
            );
        });
        self.worker = Some(rx);
        self.worker_key = None;
        self.busy = true;
        self.error = None;
    }
    pub fn start_import(&mut self) {
        if self.busy {
            return;
        }
        let Some(form) = &self.form else {
            return;
        };
        if let Some(record) = &form.existing
            && self.index.resolve(record.meta.id).is_err()
        {
            self.error = Some("Resolve duplicate UUIDs before reimporting.".into());
            return;
        }
        let (root, source, destination, settings, existing, snapshot) = (
            self.root.clone(),
            form.source.clone(),
            form.destination.clone(),
            form.settings.clone(),
            form.existing.clone(),
            form.snapshot,
        );
        let model = form.model;
        let texture = form.texture;
        let sequence = form.sequence.clone();
        let bank = form.bank.clone();
        let bank_companion = form.bank_companion.clone();
        self.worker_key = form.queue_key.clone();
        let (tx, rx) = mpsc::channel();
        // Worker only prepares a candidate/cache. The owning session alone can publish it.
        std::thread::spawn(move || {
            let _ = tx.send(if texture {
                crate::texture::prepare(&root, &source, &destination, existing.as_ref(), snapshot)
                    .map(Prepared::Audio)
            } else if model {
                crate::model_import::prepare(
                    &root,
                    &source,
                    &destination,
                    existing.as_ref(),
                    snapshot,
                )
                .map(Prepared::Model)
            } else if let Some(settings) = sequence {
                crate::sequence::prepare(
                    &root,
                    &source,
                    &destination,
                    settings,
                    existing.as_ref(),
                    snapshot,
                )
                .map(Prepared::Audio)
            } else if let Some(settings) = bank {
                if settings.library.is_some()
                    || crate::soundfont_asset::has_source_header(&root.join(&source))
                {
                    crate::soundfont_asset::prepare(
                        &root,
                        &source,
                        &destination,
                        settings,
                        existing.as_ref(),
                        snapshot,
                    )
                    .map(Prepared::Audio)
                } else if settings.imported.is_some() {
                    crate::bank_compat::prepare(
                        &root,
                        &source,
                        Some(&bank_companion),
                        &destination,
                        settings,
                        existing.as_ref(),
                        snapshot,
                    )
                    .map(Prepared::SonyBank)
                } else if existing.is_none() && source.is_empty() {
                    crate::sound_bank::prepare_new(&root, &destination, settings)
                        .map(Prepared::Audio)
                } else {
                    assets::prepare_portable(
                        &root,
                        &source,
                        &destination,
                        crate::import_settings::Settings::SoundBank(settings),
                        existing.as_ref(),
                        snapshot,
                    )
                    .map(Prepared::Audio)
                }
            } else {
                assets::prepare(
                    &root,
                    &source,
                    &destination,
                    settings,
                    existing.as_ref(),
                    snapshot,
                )
                .map(Prepared::Audio)
            });
        });
        self.worker = Some(rx);
        self.busy = true;
        self.error = None;
    }
    pub fn report<T>(&mut self, result: Result<T, String>) {
        match result {
            Ok(_) => {
                self.error = None;
                self.refresh_editor = true;
                self.refresh();
            }
            Err(e) => self.error = Some(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sony_pair_watcher_groups_vb_changes_and_preserves_unknown_metadata() {
        let root = crate::workspace::tests::temp("sony-pair-watcher");
        std::fs::create_dir_all(root.join("assets")).unwrap();
        let bytes = crate::bank_compat::tests::fixture();
        std::fs::write(root.join("assets/source.vh"), &bytes[..0xc20]).unwrap();
        std::fs::write(root.join("assets/source.vb"), &bytes[0xc20..]).unwrap();
        let id = crate::bank_compat::commit(
            crate::bank_compat::prepare(
                &root,
                "assets/source.vh",
                Some("assets/source.vb"),
                "assets/bank.epokasset",
                Default::default(),
                None,
                false,
            )
            .unwrap(),
        )
        .unwrap();
        let mut manager = Manager::new(root.clone());
        manager.index = assets::scan(&root, &mut Default::default());
        for source in manager.index.sources.values() {
            manager.observed.insert(
                source.path.clone(),
                (source.hash.clone(), Instant::now() - Duration::from_secs(1)),
            );
        }
        manager.update_pending();
        assert!(manager.pending.is_empty());
        let mut changed = bytes[0xc20..].to_vec();
        changed[2] ^= 1;
        std::fs::write(root.join("assets/source.vb"), changed).unwrap();
        manager.index = assets::scan(&root, &mut Default::default());
        for source in manager.index.sources.values() {
            manager.observed.insert(
                source.path.clone(),
                (source.hash.clone(), Instant::now() - Duration::from_secs(1)),
            );
        }
        manager.update_pending();
        assert_eq!(manager.pending.len(), 1);
        assert_eq!(manager.pending[0].existing, Some(id));
        assert_eq!(manager.pending[0].source, "assets/source.vh");
        let record = manager.index.resolve(id).unwrap();
        let mut settings = record.meta.settings.sound_bank().unwrap().clone();
        let definition = settings.imported.as_mut().unwrap();
        definition.extra.insert("future".into(), 42.into());
        definition.parts[1]
            .extra
            .insert("future_part".into(), 17.into());
        crate::bank_compat::commit(
            crate::bank_compat::prepare(
                &root,
                "assets/source.vh",
                Some("assets/source.vb"),
                "assets/bank.epokasset",
                settings,
                Some(record),
                false,
            )
            .unwrap(),
        )
        .unwrap();
        let package = assets::Package::load(&record.path).unwrap();
        let definition = package
            .meta
            .settings
            .sound_bank()
            .unwrap()
            .imported
            .as_ref()
            .unwrap();
        assert_eq!(definition.extra["future"], 42);
        assert_eq!(definition.parts[1].extra["future_part"], 17);
        assert!(assets::source_relinks(&root, &manager.index).is_empty());
        drop(manager);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn detection_omission_reopen_and_worker_publication() {
        let root = std::env::temp_dir().join(format!("epok-import-manager-{}", Uuid::new_v4()));
        std::fs::create_dir_all(root.join("assets")).unwrap();
        std::fs::write(
            root.join("assets/tone.wav"),
            crate::audio_import::test_wav(),
        )
        .unwrap();
        let wait = |manager: &mut Manager, predicate: fn(&Manager) -> bool| {
            let deadline = Instant::now() + Duration::from_secs(8);
            while !predicate(manager) {
                manager.tick();
                assert!(Instant::now() < deadline, "{:?}", manager.error);
                std::thread::sleep(Duration::from_millis(10));
            }
        };
        let mut manager = Manager::new(root.clone());
        wait(&mut manager, |m| !m.pending.is_empty());
        assert!(manager.notification);
        assert_eq!(manager.index.usable().count(), 0);
        let key = manager.pending[0].key();
        manager.omit(&key);
        drop(manager);
        let mut manager = Manager::new(root.clone());
        wait(&mut manager, |m| !m.pending.is_empty());
        assert_eq!(manager.pending[0].status, Status::Omitted);
        assert!(!manager.notification);
        let item = manager.pending[0].clone();
        manager.begin_pending(&item);
        manager.start_import();
        wait(&mut manager, |m| m.selected.is_some() && !m.busy);
        let id = manager.selected.unwrap();
        manager.refresh();
        wait(&mut manager, |m| {
            m.index.resolve(m.selected.unwrap()).is_ok() && m.pending.is_empty()
        });
        assert_eq!(
            manager.index.resolve(id).unwrap().meta.source,
            "assets/tone.wav"
        );
        // A move between sessions is reconciled and written back to the portable package.
        drop(manager);
        std::fs::rename(root.join("assets/tone.wav"), root.join("assets/moved.wav")).unwrap();
        let mut manager = Manager::new(root.clone());
        wait(&mut manager, |m| {
            m.index
                .usable()
                .any(|r| r.meta.source == "assets/moved.wav")
        });
        assert_eq!(
            manager.index.resolve(id).unwrap().meta.source,
            "assets/moved.wav"
        );
        drop(manager);
        assert!(
            root.starts_with(std::env::temp_dir())
                && root
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("epok-import-manager-")
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
