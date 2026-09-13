//! OS notifications wake content validation; they never certify build inputs.
//! Callbacks only coalesce hints, so reading/hashing cannot block the OS thread.
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

const QUIET: Duration = Duration::from_millis(150);
const MAX_BATCH: Duration = Duration::from_secs(1);
// Also reconciles silent backends (e.g. some network shares). This is a safety
// net, not the primary change detector. Play/build still validates independently.
const RECONCILE: Duration = Duration::from_secs(60);

#[derive(Clone, PartialEq, Eq)]
enum Scope {
    Project(PathBuf),
    Files(BTreeSet<PathBuf>),
}

/// Resolve symlinked ancestors without requiring the leaf to exist.
///
/// macOS reports FSEvents paths through `/private`, and both `/tmp` and the
/// per-user temporary folders are symlinks, so a watch root and the events it
/// receives would otherwise never compare equal. Renamed and deleted leaves
/// keep their own names, since only existing ancestors can be resolved.
fn resolved(path: &Path) -> PathBuf {
    let mut trailing = Vec::new();
    let mut current = path;
    loop {
        if let Ok(mut real) = fs::canonicalize(current) {
            real.extend(trailing.iter().rev());
            return real;
        }
        match (current.parent(), current.file_name()) {
            (Some(parent), Some(name)) => {
                trailing.push(name.to_owned());
                current = parent;
            }
            _ => return path.to_owned(),
        }
    }
}

fn normalized(path: &Path) -> PathBuf {
    let path = resolved(path).to_string_lossy().replace('\\', "/");
    #[cfg(windows)]
    let path = {
        let path = path.to_ascii_lowercase();
        if let Some(unc) = path.strip_prefix("//?/unc/") {
            format!("//{unc}")
        } else {
            path.strip_prefix("//?/").unwrap_or(&path).to_owned()
        }
    };
    PathBuf::from(path)
}

impl Scope {
    fn ancestor_event(&self, path: &Path) -> bool {
        let path = normalized(path);
        match self {
            Self::Project(root) => root == &path,
            Self::Files(files) => files
                .iter()
                .any(|file| file != &path && file.starts_with(&path)),
        }
    }
    fn accepts(&self, path: &Path) -> bool {
        let path = normalized(path);
        match self {
            Self::Project(root) => {
                let Ok(relative) = path.strip_prefix(root) else {
                    return false;
                };
                // Generated outputs/preferences must not feed the observer back
                // into itself. Access-only events are filtered separately.
                !relative.components().any(|part| {
                    let name = part.as_os_str().to_string_lossy().to_ascii_lowercase();
                    name.starts_with('.')
                        || matches!(name.as_str(), "target" | "exports" | "usersettings")
                })
            }
            // Keep both sides of renames and ancestor removals. Do not require
            // existence/canonicalization: deleted and atomically replaced files
            // are precisely the cases the observer must be able to report.
            Self::Files(files) => files
                .iter()
                .any(|file| file == &path || file.starts_with(&path)),
        }
    }
    fn roots(&self) -> BTreeMap<PathBuf, RecursiveMode> {
        let candidates: BTreeSet<_> = match self {
            Self::Project(root) => [root.clone()].into(),
            Self::Files(files) => files
                .iter()
                .filter_map(|file| file.parent().map(Path::to_path_buf))
                .collect(),
        };
        let candidates: BTreeSet<_> = if matches!(self, Self::Project(_)) {
            candidates // Do not recursively watch all Temp for an absent project.
        } else {
            candidates
                .into_iter()
                .filter_map(|path| {
                    path.ancestors()
                        .find(|ancestor| ancestor.is_dir())
                        .map(Path::to_path_buf)
                })
                .collect()
        };
        let recursive: BTreeSet<_> = candidates
            .iter()
            .filter(|path| {
                !candidates
                    .iter()
                    .any(|other| other != *path && path.starts_with(other))
            })
            .cloned()
            .collect();
        let mut roots = BTreeMap::new();
        for path in &recursive {
            roots.insert(path.clone(), RecursiveMode::Recursive);
            // A shallow parent watch catches replacement of a watched folder
            // without recursively subscribing to all siblings (e.g. all Temp).
            if let Some(parent) = path.parent()
                && !recursive.iter().any(|root| parent.starts_with(root))
            {
                roots
                    .entry(parent.into())
                    .or_insert(RecursiveMode::NonRecursive);
            }
        }
        roots
    }
}

#[derive(Default)]
struct Pending {
    first: Option<Instant>,
    last: Option<Instant>,
    rescan: bool,
    warning: Option<String>,
}
impl Pending {
    fn event(&mut self, scope: &Scope, result: notify::Result<Event>, now: Instant) {
        let relevant = match result {
            Err(error) => {
                self.rescan = true;
                self.warning = Some(format!("File notifications need recovery: {error}"));
                true
            }
            Ok(event) if event.need_rescan() => {
                self.rescan = true;
                true
            }
            Ok(event) => {
                if matches!(
                    event.kind,
                    EventKind::Create(_)
                        | EventKind::Remove(_)
                        | EventKind::Modify(notify::event::ModifyKind::Name(_))
                ) && event.paths.iter().any(|path| scope.ancestor_event(path))
                {
                    self.rescan = true; // Reattach after watched-directory replacement.
                }
                !matches!(event.kind, EventKind::Access(_))
                    && (event.paths.is_empty()
                        || event.paths.iter().any(|path| scope.accepts(path)))
            }
        };
        if relevant {
            self.first.get_or_insert(now);
            self.last = Some(now);
        }
    }
    fn take(&mut self, now: Instant) -> bool {
        let ready = self.rescan
            || self
                .last
                .is_some_and(|last| now.duration_since(last) >= QUIET)
            || self
                .first
                .is_some_and(|first| now.duration_since(first) >= MAX_BATCH);
        if ready {
            self.first = None;
            self.last = None;
            self.rescan = false;
        }
        ready
    }
}

pub struct Watch {
    scope: Scope,
    pending: Arc<Mutex<Pending>>,
    watcher: Option<RecommendedWatcher>,
    reconciled: Instant,
    warning: Option<String>,
}
impl Watch {
    pub fn project(root: &Path) -> Self {
        Self::new(Scope::Project(normalized(root)))
    }
    pub fn files(files: impl IntoIterator<Item = PathBuf>) -> Self {
        Self::new(Scope::Files(
            files.into_iter().map(|path| normalized(&path)).collect(),
        ))
    }
    fn new(scope: Scope) -> Self {
        let mut watch = Self {
            scope,
            pending: Default::default(),
            watcher: None,
            reconciled: Instant::now(),
            warning: None,
        };
        watch.arm();
        watch
    }
    fn arm(&mut self) {
        let pending = self.pending.clone();
        let scope = self.scope.clone();
        let result = notify::recommended_watcher(move |event| {
            if let Ok(mut pending) = pending.lock() {
                pending.event(&scope, event, Instant::now());
            }
        })
        .and_then(|mut watcher| {
            for (root, mode) in self.scope.roots() {
                watcher.watch(&root, mode)?;
            }
            Ok(watcher)
        });
        match result {
            Ok(watcher) => self.watcher = Some(watcher),
            Err(error) => {
                self.watcher = None;
                self.warning = Some(format!(
                    "File notifications unavailable; using 60-second recovery scans: {error}"
                ));
            }
        }
    }
    /// Re-arm only when compiler/reflection dependency membership changes.
    /// Returns true if a new subscription needs an initial content check.
    pub fn set_files(&mut self, files: impl IntoIterator<Item = PathBuf>) -> bool {
        let scope = Scope::Files(files.into_iter().map(|path| normalized(&path)).collect());
        if self.scope == scope {
            return false;
        }
        self.scope = scope;
        self.arm();
        true
    }
    pub fn poll(&mut self) -> bool {
        let now = Instant::now();
        let recovery = now.duration_since(self.reconciled) >= RECONCILE;
        let (event, rearm) = {
            let mut pending = self.pending.lock().unwrap();
            let rearm = pending.rescan;
            if let Some(warning) = pending.warning.take() {
                self.warning = Some(warning);
            }
            (pending.take(now), rearm)
        };
        if recovery || rearm {
            self.reconciled = now;
            self.arm();
        }
        event || recovery
    }
    pub fn take_warning(&mut self) -> Option<String> {
        self.warning.take()
    }
    #[cfg(test)]
    fn native_active(&self) -> bool {
        self.watcher.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use notify::event::{AccessKind, CreateKind, Flag, ModifyKind, RenameMode};

    #[test]
    fn recovery_scan_is_infrequent_and_folder_replacement_rearms() {
        let root =
            std::env::temp_dir().join(format!("epok-watch-recovery-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let mut watch = Watch::files([root.join("Shared.hpp")]);
        assert!(!watch.poll());
        watch.reconciled = Instant::now() - RECONCILE;
        assert!(watch.poll());
        assert!(!watch.poll());
        watch.pending.lock().unwrap().event(
            &watch.scope,
            Ok(
                Event::new(EventKind::Modify(ModifyKind::Name(RenameMode::From)))
                    .add_path(root.clone()),
            ),
            Instant::now(),
        );
        assert!(watch.pending.lock().unwrap().rescan);
        assert!(watch.poll());
        assert!(!watch.poll());
        drop(watch);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn windows_extended_paths_match_native_event_paths_including_unc() {
        assert_eq!(
            normalized(Path::new(r"\\?\D:\Game\assets\Test.cpp")),
            normalized(Path::new(r"d:\game\assets\test.cpp"))
        );
        assert_eq!(
            normalized(Path::new(r"\\?\UNC\Server\Share\Game")),
            normalized(Path::new(r"\\server\share\game"))
        );
    }

    #[test]
    fn batches_renames_ignores_reads_and_outputs_and_recovers_overflow() {
        let root = std::env::temp_dir().join("epok-event-filter");
        let scope = Scope::Project(normalized(&root));
        let mut pending = Pending::default();
        let now = Instant::now();
        for path in [
            ".epok/Build.log",
            ".git/index",
            "exports/game/scene.hh",
            "UserSettings/ContentBrowser.epokprefs",
        ] {
            pending.event(
                &scope,
                Ok(Event::new(EventKind::Create(CreateKind::File)).add_path(root.join(path))),
                now,
            );
        }
        pending.event(
            &scope,
            Ok(Event::new(EventKind::Access(AccessKind::Read))
                .add_path(root.join("assets/scene.epokmap"))),
            now,
        );
        assert!(!pending.take(now + QUIET));
        let renamed = Event::new(EventKind::Modify(ModifyKind::Name(RenameMode::Both)))
            .add_path(root.join(".temporary"))
            .add_path(root.join("assets/scene.epokmap"));
        pending.event(&scope, Ok(renamed.clone()), now);
        pending.event(&scope, Ok(renamed), now + QUIET / 2);
        assert!(!pending.take(now + QUIET));
        assert!(pending.take(now + QUIET * 2));
        assert!(!pending.take(now + QUIET * 3));
        pending.event(
            &scope,
            Ok(Event::new(EventKind::Other).set_flag(Flag::Rescan)),
            now,
        );
        assert!(pending.take(now));
    }

    #[test]
    fn native_events_detect_create_rename_delete_and_preserved_timestamp() {
        let root =
            std::env::temp_dir().join(format!("epok-native-events-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(root.join("assets")).unwrap();
        let mut watch = Watch::project(&root);
        assert!(watch.native_active(), "{:?}", watch.take_warning());
        let wait = |watch: &mut Watch| {
            let started = Instant::now();
            while !watch.poll() {
                assert!(
                    started.elapsed() < Duration::from_secs(5),
                    "Native event was lost"
                );
                std::thread::sleep(Duration::from_millis(10));
            }
        };
        let path = root.join("assets/test.cpp");
        std::fs::write(&path, b"old").unwrap();
        wait(&mut watch);
        let modified = std::fs::metadata(&path).unwrap().modified().unwrap();
        std::fs::write(&path, b"new").unwrap();
        std::fs::File::options()
            .write(true)
            .open(&path)
            .unwrap()
            .set_times(std::fs::FileTimes::new().set_modified(modified))
            .unwrap();
        wait(&mut watch);
        let renamed = root.join("assets/renamed.cpp");
        std::fs::rename(&path, &renamed).unwrap();
        wait(&mut watch);
        std::fs::remove_file(&renamed).unwrap();
        wait(&mut watch);
        for _ in 0..20 {
            assert!(!watch.poll());
            std::thread::sleep(Duration::from_millis(10));
        }
        drop(watch);
        std::fs::remove_dir_all(root).unwrap();
    }
}
