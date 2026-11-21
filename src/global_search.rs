//! Global search (Ctrl+P or the toolbar field): one fuzzy-ranked index over the
//! project's scripts, Blueprints, scenes, assets and folders, the open scene's
//! actors, the settings pages and a few editor commands.
//!
//! The file list comes from a walk of `assets/` on a worker thread, classified
//! with the asset index the editor already keeps, and is rebuilt only when the
//! project changes. Ranking runs when the query changes, never per frame, and a
//! query that extends the previous one only re-ranks the previous matches.
use crate::{assets, editor::Editor};
use imgui::{Key, MouseButton, StyleColor as C, StyleVar as V, Ui};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::Path,
    sync::mpsc::{self, Receiver, TryRecvError},
};

const POPUP: &str = "##global-search";
const PREFS: &str = "UserSettings/GlobalSearch.epokprefs";
const MAX_RECENT: usize = 24;
const ROW_HEIGHT: f32 = 30.;
const ACCENT: [f32; 4] = [0.075, 0.28, 0.48, 1.];
const MATCHED: [f32; 4] = [1.0, 0.78, 0.38, 1.];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Category {
    Script,
    Blueprint,
    Scene,
    Asset,
    Actor,
    Setting,
    Command,
}
impl Category {
    /// A small preference among near-equal matches: documents you open over
    /// raw assets, and those over settings pages and commands.
    fn bias(self) -> i32 {
        match self {
            Category::Scene | Category::Blueprint | Category::Script => 4,
            Category::Actor => 2,
            Category::Asset => 0,
            Category::Setting | Category::Command => -2,
        }
    }
    fn filter(self) -> usize {
        FILTERS
            .iter()
            .position(|(category, _)| *category == Some(self))
            .unwrap_or(0)
    }
}

/// Filter chips in display order; `None` shows every category.
pub const FILTERS: [(Option<Category>, &str); 8] = [
    (None, "All"),
    (Some(Category::Script), "Scripts"),
    (Some(Category::Blueprint), "Blueprints"),
    (Some(Category::Scene), "Scenes"),
    (Some(Category::Asset), "Assets"),
    (Some(Category::Actor), "Actors"),
    (Some(Category::Setting), "Settings"),
    (Some(Category::Command), "Commands"),
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    /// Project-relative file path with forward slashes.
    File(String),
    Folder(String),
    Actor(uuid::Uuid),
    Setting {
        preferences: bool,
        page: usize,
    },
    Command(&'static str),
}
impl Target {
    fn key(&self) -> String {
        match self {
            Target::File(path) => format!("file:{path}"),
            Target::Folder(path) => format!("folder:{path}"),
            Target::Actor(id) => format!("actor:{id}"),
            Target::Setting { preferences, page } => format!(
                "setting:{}:{page}",
                if *preferences { "editor" } else { "project" }
            ),
            Target::Command(id) => format!("command:{id}"),
        }
    }
    fn path(&self) -> Option<&str> {
        match self {
            Target::File(path) | Target::Folder(path) => Some(path),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Item {
    pub name: String,
    /// Dimmed context: the folder, the actor's class or the settings window.
    pub detail: String,
    /// Type tag shown at the right; it also chooses the icon.
    pub kind: &'static str,
    pub category: Category,
    pub target: Target,
    name_chars: Box<[char]>,
    /// Secondary text a query may match: the full path, class or keywords.
    extra_chars: Box<[char]>,
}
impl Item {
    pub fn new(
        name: &str,
        detail: &str,
        extra: &str,
        kind: &'static str,
        category: Category,
        target: Target,
    ) -> Self {
        Self {
            name: name.into(),
            detail: detail.into(),
            kind,
            category,
            target,
            name_chars: name.chars().collect(),
            extra_chars: extra.chars().collect(),
        }
    }
    fn rename(&mut self, name: &str) {
        self.name = name.into();
        self.name_chars = name.chars().collect();
    }
}

// ------------------------------------------------------------------ ranking --

const MATCH: i32 = 16;
const GAP_START: i32 = 3;
const GAP_EXTEND: i32 = 1;
const CONSECUTIVE: i32 = 5;
const FIRST_MULTIPLIER: i32 = 2;
const NONE: i32 = i32::MIN / 4;

fn fold(c: char) -> char {
    if c.is_ascii() {
        c.to_ascii_lowercase()
    } else {
        c.to_lowercase().next().unwrap_or(c)
    }
}
fn separator(c: char) -> bool {
    matches!(
        c,
        ' ' | '_' | '-' | '/' | '\\' | '.' | ':' | '(' | ')' | '[' | ']' | ','
    )
}
/// How much a match at `c` (after `prev`) looks like the start of a word.
fn boundary(prev: Option<char>, c: char) -> i32 {
    match prev {
        None => 10,
        Some(p) if separator(p) => 9,
        Some(p) if p.is_lowercase() && c.is_uppercase() => 8,
        Some(p) if !p.is_alphanumeric() && c.is_alphanumeric() => 7,
        Some(p) if !p.is_ascii_digit() && c.is_ascii_digit() => 6,
        _ => 0,
    }
}

/// Reusable buffers so ranking a large index allocates once.
#[derive(Default)]
pub struct Scratch {
    score: Vec<i32>,
    from: Vec<u32>,
    /// Bonus of the run a cell continues, as fzf carries a word start's bonus
    /// through the letters that follow it.
    chunk: Vec<i32>,
    bonus: Vec<i32>,
    positions: Vec<usize>,
}

/// Best alignment of `pattern` (already lowercase) inside `text`, scored like
/// fzf: word starts, camelCase humps and runs of consecutive letters earn
/// bonuses, gaps cost. `None` when `pattern` is not a subsequence of `text`.
/// With `positions`, also writes the matched character indices.
pub fn fuzzy(
    pattern: &[char],
    text: &[char],
    scratch: &mut Scratch,
    positions: Option<&mut Vec<usize>>,
) -> Option<i32> {
    let (m, n) = (pattern.len(), text.len());
    if m == 0 {
        if let Some(positions) = positions {
            positions.clear();
        }
        return Some(0);
    }
    if m > n {
        return None;
    }
    // Cheap rejection before the quadratic pass.
    let mut next = 0;
    for &c in text {
        if fold(c) == pattern[next] {
            next += 1;
            if next == m {
                break;
            }
        }
    }
    if next < m {
        return None;
    }
    let Scratch {
        score,
        from,
        chunk,
        bonus,
        ..
    } = scratch;
    bonus.clear();
    bonus.extend((0..n).map(|j| boundary(j.checked_sub(1).map(|k| text[k]), text[j])));
    score.clear();
    score.resize(m * n, NONE);
    from.clear();
    from.resize(m * n, u32::MAX);
    chunk.clear();
    chunk.resize(m * n, 0);
    for j in 0..n {
        if fold(text[j]) == pattern[0] {
            // A light lean towards earlier matches; word starts matter more.
            score[j] = MATCH + bonus[j] * FIRST_MULTIPLIER - (j as i32).min(4);
            chunk[j] = bonus[j];
        }
    }
    for (i, &wanted) in pattern.iter().enumerate().skip(1) {
        let (previous, row) = ((i - 1) * n, i * n);
        // Best predecessor at least two characters back, already charged for
        // the gap it leaves before `j`.
        let mut gap = NONE;
        let mut gap_from = u32::MAX;
        for j in i..n {
            if j >= 2 {
                if gap > NONE {
                    gap -= GAP_EXTEND;
                }
                let candidate = score[previous + j - 2];
                if candidate > NONE && candidate - GAP_START > gap {
                    gap = candidate - GAP_START;
                    gap_from = (j - 2) as u32;
                }
            }
            if fold(text[j]) != wanted {
                continue;
            }
            let mut best = NONE;
            let mut origin = u32::MAX;
            let mut run = bonus[j];
            let adjacent = score[previous + j - 1];
            if adjacent > NONE {
                let carried = chunk[previous + j - 1].max(bonus[j]).max(CONSECUTIVE);
                best = adjacent + MATCH + carried;
                origin = (j - 1) as u32;
                run = carried;
            }
            if gap > NONE && gap + MATCH + bonus[j] > best {
                best = gap + MATCH + bonus[j];
                origin = gap_from;
                run = bonus[j];
            }
            score[row + j] = best;
            from[row + j] = origin;
            chunk[row + j] = run;
        }
    }
    let last = (m - 1) * n;
    let (end, best) = (0..n)
        .map(|j| (j, score[last + j]))
        .max_by_key(|&(j, s)| (s, std::cmp::Reverse(j)))?;
    if best <= NONE {
        return None;
    }
    if let Some(positions) = positions {
        positions.clear();
        positions.resize(m, 0);
        let mut j = end;
        for i in (0..m).rev() {
            positions[i] = j;
            if i > 0 {
                j = from[i * n + j] as usize;
            }
        }
    }
    Some(best)
}

/// Query terms: whitespace-separated, lowercase. Every term must match.
pub fn terms(query: &str) -> Vec<Vec<char>> {
    query
        .split_whitespace()
        .map(|term| term.chars().map(fold).collect())
        .collect()
}

/// A term matches secondary text (a path, a class, keyword lists) only as a
/// substring or within one word that it starts: scattered letters across a
/// long keyword list would otherwise match nearly any short query.
fn secondary(term: &[char], text: &[char], scratch: &mut Scratch) -> Option<i32> {
    let m = term.len();
    if m == 0 {
        return Some(0);
    }
    let mut best: Option<i32> = None;
    if m <= text.len() {
        for start in 0..=text.len() - m {
            if text[start..start + m]
                .iter()
                .zip(term)
                .all(|(&a, &b)| fold(a) == b)
            {
                let lead = boundary(start.checked_sub(1).map(|k| text[k]), text[start]);
                let value =
                    MATCH * m as i32 + lead * FIRST_MULTIPLIER + CONSECUTIVE * (m as i32 - 1);
                best = best.max(Some(value));
            }
        }
    }
    if best.is_some() {
        return best;
    }
    let mut start = 0;
    while start < text.len() {
        let end = text[start..]
            .iter()
            .position(|c| !c.is_alphanumeric())
            .map_or(text.len(), |offset| start + offset);
        if end > start && fold(text[start]) == term[0] {
            best = best.max(fuzzy(term, &text[start..end], scratch, None));
        }
        start = end + 1;
    }
    best
}

/// Exact names and prefixes beat scattered matches of the same letters.
fn name_bonus(term: &[char], name: &[char]) -> i32 {
    let stem = name
        .iter()
        .rposition(|&c| c == '.')
        .filter(|&i| i > 0)
        .unwrap_or(name.len());
    let equal =
        |len: usize| len == term.len() && name[..len].iter().zip(term).all(|(&a, &b)| fold(a) == b);
    if equal(name.len()) || equal(stem) {
        60
    } else if term.len() <= name.len() && name.iter().zip(term).all(|(&a, &b)| fold(a) == b) {
        25
    } else {
        0
    }
}

/// Score of `item` for all `terms`, or `None` when a term matches neither its
/// name nor its secondary text. Name matches count fully and are highlighted;
/// path, class and keyword matches count half.
pub fn score(
    item: &Item,
    terms: &[Vec<char>],
    scratch: &mut Scratch,
    mut highlights: Option<&mut Vec<usize>>,
) -> Option<i32> {
    if let Some(highlights) = highlights.as_deref_mut() {
        highlights.clear();
    }
    let mut positions = std::mem::take(&mut scratch.positions);
    let mut total = 0;
    let mut matched = true;
    for term in terms {
        let wants = highlights.is_some();
        if let Some(value) = fuzzy(
            term,
            &item.name_chars,
            scratch,
            wants.then_some(&mut positions),
        ) {
            total += value + name_bonus(term, &item.name_chars);
            if let Some(highlights) = highlights.as_deref_mut() {
                highlights.extend_from_slice(&positions);
            }
        } else if let Some(value) = secondary(term, &item.extra_chars, scratch) {
            total += value / 2 - 8;
        } else {
            matched = false;
            break;
        }
    }
    scratch.positions = positions;
    matched.then_some(total)
}

// -------------------------------------------------------------------- index --

/// One walked entry under `assets/`: project-relative path and whether it is a folder.
pub type Entry = (String, bool);

/// Every file and folder under `assets/`, sorted by path. Hidden names and
/// links are skipped, as in the Content Browser. Runs on a worker thread.
pub fn walk(root: &Path) -> Vec<Entry> {
    fn visit(root: &Path, dir: &Path, out: &mut Vec<Entry>) {
        let Ok(items) = fs::read_dir(dir) else {
            return;
        };
        for item in items.flatten() {
            let name = item.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') {
                continue;
            }
            let Ok(meta) = fs::symlink_metadata(item.path()) else {
                continue;
            };
            if meta.file_type().is_symlink() {
                continue;
            }
            #[cfg(windows)]
            {
                use std::os::windows::fs::MetadataExt;
                if meta.file_attributes() & 0x400 != 0 {
                    continue;
                }
            }
            out.push((assets::path_string(root, &item.path()), meta.is_dir()));
            if meta.is_dir() {
                visit(root, &item.path(), out);
            }
        }
    }
    let mut out = Vec::new();
    visit(root, &root.join("assets"), &mut out);
    out.sort();
    out
}

/// Class compositions are stored under their class UUID; their readable name
/// is the root entity's class in the template. Read on the walk's thread.
pub fn class_names(root: &Path, entries: &[Entry]) -> HashMap<String, String> {
    entries
        .iter()
        .filter(|(path, folder)| !folder && path.ends_with(".epokclass"))
        .filter_map(|(path, _)| {
            let bytes = assets::read_bounded(&root.join(path)).ok()?;
            let document = crate::class_composition::Document::parse(&bytes).ok()?;
            let name = document
                .template
                .actors
                .iter()
                .map(|actor| &actor.entity.class)
                .find(|class| class.class_id.as_deref() == Some(document.class.as_str()))?
                .name
                .clone();
            (!name.is_empty()).then(|| (path.clone(), name))
        })
        .collect()
}

/// What a walked path is to the search, or `None` for files it does not list
/// (raw import sources, script descriptors, notes). `kinds` maps imported
/// `.epokasset` paths to their asset kind.
pub fn file_item(path: &str, folder: bool, kinds: &HashMap<String, assets::Kind>) -> Option<Item> {
    let name = path.rsplit('/').next().filter(|n| !n.is_empty())?;
    let parent = path.rsplit_once('/').map_or("", |(parent, _)| parent);
    if folder {
        return Some(Item::new(
            name,
            parent,
            &format!("{path} folder"),
            "Folder",
            Category::Asset,
            Target::Folder(path.into()),
        ));
    }
    let lower = name.to_ascii_lowercase();
    let extension = lower.rsplit_once('.').map_or("", |(_, e)| e);
    let (kind, category) = if extension == "epokasset" {
        (
            kinds
                .get(path)
                .map_or("Asset", crate::project_browser::kind_label),
            Category::Asset,
        )
    } else if extension == "epokbp" {
        ("Blueprint", Category::Blueprint)
    } else if extension == "epokclass" {
        ("UI Class", Category::Blueprint)
    } else if extension == "epokmap" {
        ("Scene", Category::Scene)
    } else if lower.ends_with(".timeline.json") {
        ("Timeline", Category::Asset)
    } else if lower.ends_with(".particle-effect.json") {
        ("Particle Effect", Category::Asset)
    } else if extension == "lua" {
        ("Lua", Category::Script)
    } else if matches!(
        extension,
        "cpp" | "hpp" | "h" | "c" | "cc" | "cxx" | "hxx" | "inl"
    ) {
        ("C++", Category::Script)
    } else {
        return None;
    };
    // Scripts keep their extension: Spinner.hpp and Spinner.cpp are two results.
    let display = if category == Category::Script {
        name
    } else {
        crate::project_browser::display_file_name(name)
    };
    Some(Item::new(
        display,
        parent,
        &format!("{path} {kind}"),
        kind,
        category,
        Target::File(path.into()),
    ))
}

/// Asset kinds by project-relative `.epokasset` path.
pub fn asset_kinds(root: &Path, index: &assets::Index) -> HashMap<String, assets::Kind> {
    index
        .assets
        .values()
        .flatten()
        .map(|record| {
            (
                assets::path_string(root, &record.path),
                record.meta.kind.clone(),
            )
        })
        .collect()
}

pub fn actor_items(scene: &crate::scene::Scene) -> Vec<Item> {
    scene
        .actors
        .iter()
        .map(|actor| {
            let class = actor
                .class
                .name
                .rsplit("::")
                .next()
                .unwrap_or(&actor.class.name);
            Item::new(
                &actor.name,
                class,
                &format!("{} actor {}", actor.class.name, scene.name),
                "Actor",
                Category::Actor,
                Target::Actor(actor.id),
            )
        })
        .collect()
}

/// Commands: label, id and the words a query may use to find them.
const COMMANDS: &[(&str, &str, &str)] = &[
    ("Build", "build", "compile game native psx"),
    ("Package Disc", "export-disc", "export iso bin cue cd image"),
    ("Play / Stop", "play", "run game start test"),
    ("Save All", "save-all", "Ctrl+S save scene documents"),
    ("Reload Scene", "reload", "revert map"),
    ("New C++ Script", "new-script", "create class cpp code"),
    ("New Lua Class", "new-lua", "create script lua code"),
    ("Reset Layout", "reset-layout", "window dock panels default"),
    (
        "Open Code Editor",
        "code-editor",
        "scripts source workspace",
    ),
    ("Lighting", "lighting", "lights bake window"),
    (
        "Strings (Localization)",
        "strings",
        "localization language text",
    ),
    ("PocketStation", "pocketstation", "pocket station minigame"),
    (
        "Artifact Dependencies",
        "artifact-dependencies",
        "graph build outputs",
    ),
    ("Memory Report", "memory-report", "ram vram budget size"),
    ("Projects (Hub)", "hub", "open new close project"),
];

pub fn static_items() -> Vec<Item> {
    let mut items = Vec::new();
    for (preferences, window, pages) in [
        (
            true,
            "Editor Preferences",
            crate::settings_ui::PREFERENCE_PAGES,
        ),
        (false, "Project Settings", crate::settings_ui::PROJECT_PAGES),
    ] {
        for (page, (label, keywords)) in pages.iter().enumerate() {
            items.push(Item::new(
                label,
                window,
                &format!("{window} settings {keywords}"),
                "Settings",
                Category::Setting,
                Target::Setting { preferences, page },
            ));
        }
    }
    for (label, id, keywords) in COMMANDS {
        let detail = if *id == "save-all" { "Ctrl+S" } else { "" };
        items.push(Item::new(
            label,
            detail,
            &format!("command {keywords}"),
            "Command",
            Category::Command,
            Target::Command(id),
        ));
    }
    items
}

/// Where each kind's icon comes from: the Content Browser's set, plus search-only kinds.
fn icon(kind: &str) -> (&'static str, [f32; 4]) {
    match kind {
        "C++" => ("\u{f121}", [0.38, 0.62, 0.92, 1.]),
        "Lua" => crate::project_browser::type_icon("Lua Class"),
        "UI Class" => ("\u{f2d2}", [0.36, 0.70, 0.88, 1.]),
        "Actor" => ("\u{eb29}", [0.74, 0.76, 0.80, 1.]),
        "Settings" => ("\u{f013}", [0.66, 0.68, 0.71, 1.]),
        "Command" => ("\u{f0e7}", [0.95, 0.76, 0.32, 1.]),
        "Folder" => crate::project_browser::type_icon("Folder"),
        "Audio" | "SoundBank" => ("\u{f001}", crate::project_browser::type_icon(kind).1),
        "Data Table" | "String Table" => ("\u{f0ce}", [0.45, 0.76, 0.66, 1.]),
        "TileSet" => ("\u{f00a}", [0.55, 0.74, 0.45, 1.]),
        "Dialogue" | "Dialogue Style" => ("\u{f15b}", [0.80, 0.70, 0.52, 1.]),
        "Dungeon Room" => ("\u{f6d9}", [0.72, 0.62, 0.48, 1.]),
        _ => crate::project_browser::type_icon(kind),
    }
}

// -------------------------------------------------------------------- state --

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct Prefs {
    recent: Vec<String>,
}

/// A finished walk: entries and the readable names of class compositions.
type Walked = (Vec<Entry>, HashMap<String, String>);

/// Inputs the file index was built from: asset rescan requests, adopted
/// indexes, and the index's size (some tools replace it directly).
type Stamp = (u64, u64, usize, usize);

#[derive(Default)]
pub struct State {
    open: bool,
    request_open: bool,
    /// Focus the query on the next frame, selecting a prefilled query.
    focus_input: bool,
    keep_caret: bool,
    pub query: String,
    filter: usize,
    selected: usize,
    scroll_to_selected: bool,
    scroll_top: bool,
    files: Vec<Item>,
    statics: Vec<Item>,
    actors: Vec<Item>,
    entries: Vec<Entry>,
    walk: Option<(Receiver<Walked>, Stamp)>,
    class_names: HashMap<String, String>,
    walked: Option<Stamp>,
    entries_fresh: bool,
    classified: Option<Stamp>,
    actor_stamp: Option<(String, usize)>,
    items_revision: u64,
    ranked: Option<(String, u64)>,
    terms: Vec<Vec<char>>,
    /// Every match regardless of the filter chip, best first: (item, score).
    matches: Vec<(u32, i32)>,
    hits: Vec<u32>,
    counts: [usize; FILTERS.len()],
    recent_count: usize,
    filtered: Option<usize>,
    scratch: Scratch,
    highlights: Vec<usize>,
    prefs: Prefs,
    prefs_loaded: bool,
}

impl State {
    /// Open the search with `query` prefilled (empty shows recent items).
    pub fn open_with(&mut self, query: &str) {
        self.request_open = true;
        self.focus_input = true;
        self.query = query.into();
        self.selected = 0;
        self.scroll_top = true;
    }
    pub fn is_open(&self) -> bool {
        self.open || self.request_open
    }
    pub fn len(&self) -> usize {
        self.files.len() + self.statics.len() + self.actors.len()
    }
    fn item(&self, index: u32) -> &Item {
        let mut i = index as usize;
        if i < self.files.len() {
            return &self.files[i];
        }
        i -= self.files.len();
        if i < self.statics.len() {
            return &self.statics[i];
        }
        &self.actors[i - self.statics.len()]
    }
    fn recent_rank(&self, target: &Target) -> Option<usize> {
        let key = target.key();
        self.prefs.recent.iter().position(|recent| *recent == key)
    }
    fn indexing(&self) -> bool {
        self.walk.is_some()
    }

    /// Keep the index current. A walk runs once per project change, and only
    /// while the search is open (or once, to warm the first open).
    pub fn refresh(
        &mut self,
        root: &Path,
        manager: &crate::asset_manager::Manager,
        scene: &crate::scene::Scene,
    ) {
        if !self.prefs_loaded {
            self.prefs_loaded = true;
            if let Ok(bytes) = fs::read(root.join(PREFS)) {
                self.prefs = crate::document::from_slice(&bytes).unwrap_or_default();
            }
        }
        if self.statics.is_empty() {
            self.statics = static_items();
            self.items_revision += 1;
        }
        if let Some((receiver, stamp)) = &self.walk {
            match receiver.try_recv() {
                Ok((entries, class_names)) => {
                    self.walked = Some(*stamp);
                    self.entries = entries;
                    self.class_names = class_names;
                    self.entries_fresh = true;
                    self.walk = None;
                }
                Err(TryRecvError::Disconnected) => {
                    self.walked = Some(*stamp);
                    self.walk = None;
                }
                Err(TryRecvError::Empty) => {}
            }
        }
        let stamp = (
            manager.revision,
            manager.index_revision,
            manager.index.assets.len(),
            manager.index.native.len(),
        );
        if self.walk.is_none()
            && self.walked != Some(stamp)
            && (self.is_open() || self.walked.is_none())
        {
            let (sender, receiver) = mpsc::channel();
            let root = root.to_path_buf();
            let spawned = std::thread::Builder::new()
                .name("epok-global-search".into())
                .spawn(move || {
                    let entries = walk(&root);
                    let names = class_names(&root, &entries);
                    let _ = sender.send((entries, names));
                });
            if spawned.is_ok() {
                self.walk = Some((receiver, stamp));
            } else {
                self.walked = Some(stamp);
            }
        }
        // New entries are classified at once; a changed asset index (kinds
        // known only after its scan) is applied while the search is open,
        // without waiting for the next walk of the same files.
        if self.walked.is_some()
            && (self.entries_fresh || (self.is_open() && self.classified != Some(stamp)))
        {
            let kinds = asset_kinds(root, &manager.index);
            self.files = self
                .entries
                .iter()
                .filter_map(|(path, folder)| {
                    let mut item = file_item(path, *folder, &kinds)?;
                    if let Some(name) = self.class_names.get(path) {
                        item.rename(name);
                    }
                    Some(item)
                })
                .collect();
            self.classified = Some(stamp);
            self.entries_fresh = false;
            self.items_revision += 1;
        }
        let actors_changed = self
            .actor_stamp
            .as_ref()
            .is_none_or(|(name, count)| *name != scene.name || *count != scene.actors.len());
        if (self.request_open || actors_changed) && (self.is_open() || self.actor_stamp.is_none()) {
            self.actors = actor_items(scene);
            self.actor_stamp = Some((scene.name.clone(), scene.actors.len()));
            self.items_revision += 1;
        }
    }

    /// Rank every item for the current query (only when it or the index changed).
    pub fn rank(&mut self) {
        let key = (self.query.clone(), self.items_revision);
        if self.ranked.as_ref() == Some(&key) {
            return;
        }
        let terms = terms(&self.query);
        let mut scratch = std::mem::take(&mut self.scratch);
        let mut matches = Vec::new();
        self.recent_count = 0;
        if terms.is_empty() {
            // Browse: recent items first, then everything in index order.
            let mut recent = Vec::new();
            for key in &self.prefs.recent {
                if let Some(index) = self.find(key) {
                    recent.push(index);
                }
            }
            self.recent_count = recent.len();
            let skip: std::collections::HashSet<u32> = recent.iter().copied().collect();
            matches.extend(recent.into_iter().map(|i| (i, 0)));
            matches.extend(
                (0..self.len() as u32)
                    .filter(|i| !skip.contains(i))
                    .map(|i| (i, 0)),
            );
        } else {
            // A longer query can only drop matches of the shorter one.
            let narrowing = self.ranked.as_ref().is_some_and(|(previous, revision)| {
                *revision == self.items_revision
                    && !previous.trim().is_empty()
                    && self.query.starts_with(previous.as_str())
            });
            let candidates: Vec<u32> = if narrowing {
                self.matches.iter().map(|(i, _)| *i).collect()
            } else {
                (0..self.len() as u32).collect()
            };
            for index in candidates {
                let item = self.item(index);
                if let Some(value) = score(item, &terms, &mut scratch, None) {
                    let recency = self
                        .recent_rank(&item.target)
                        .map_or(0, |rank| (MAX_RECENT - rank.min(MAX_RECENT)) as i32);
                    matches.push((index, value + item.category.bias() + recency));
                }
            }
            matches.sort_by(|a, b| {
                let (x, y) = (self.item(a.0), self.item(b.0));
                b.1.cmp(&a.1)
                    .then_with(|| x.name.len().cmp(&y.name.len()))
                    .then_with(|| x.name.cmp(&y.name))
                    .then_with(|| x.detail.cmp(&y.detail))
            });
        }
        self.scratch = scratch;
        self.terms = terms;
        self.matches = matches;
        self.ranked = Some(key);
        self.filtered = None;
        self.selected = 0;
        self.scroll_top = true;
    }

    /// Apply the filter chip and count matches per category.
    pub fn refilter(&mut self) {
        if self.filtered == Some(self.filter) {
            return;
        }
        self.counts = [0; FILTERS.len()];
        let wanted = FILTERS[self.filter].0;
        let mut hits = Vec::new();
        for &(index, _) in &self.matches {
            let category = self.item(index).category;
            self.counts[0] += 1;
            self.counts[category.filter()] += 1;
            if wanted.is_none_or(|c| c == category) {
                hits.push(index);
            }
        }
        self.hits = hits;
        self.filtered = Some(self.filter);
        self.selected = self.selected.min(self.hits.len().saturating_sub(1));
    }

    /// The item a recent-list key names, if it still exists.
    fn find(&self, key: &str) -> Option<u32> {
        let (kind, value) = key.split_once(':')?;
        match kind {
            "file" | "folder" => self
                .files
                .binary_search_by(|item| item.target.path().unwrap_or_default().cmp(value))
                .ok()
                .map(|i| i as u32),
            "actor" => {
                let id = uuid::Uuid::parse_str(value).ok()?;
                let i = self
                    .actors
                    .iter()
                    .position(|item| item.target == Target::Actor(id))?;
                Some((self.files.len() + self.statics.len() + i) as u32)
            }
            _ => {
                let i = self
                    .statics
                    .iter()
                    .position(|item| item.target.key() == key)?;
                Some((self.files.len() + i) as u32)
            }
        }
    }

    fn remember(&mut self, root: &Path, target: &Target) {
        let key = target.key();
        self.prefs.recent.retain(|recent| *recent != key);
        self.prefs.recent.insert(0, key);
        self.prefs.recent.truncate(MAX_RECENT);
        self.ranked = None;
        let result = crate::document::to_vec(&self.prefs)
            .map_err(|e| e.to_string())
            .and_then(|bytes| {
                fs::create_dir_all(root.join("UserSettings")).map_err(|e| e.to_string())?;
                fs::write(root.join(PREFS), bytes).map_err(|e| e.to_string())
            });
        if let Err(error) = result {
            eprintln!("Global search history: {error}");
        }
    }

    #[cfg(test)]
    pub fn hits(&self) -> impl Iterator<Item = &Item> {
        self.hits.iter().map(|&i| self.item(i))
    }
}

// ---------------------------------------------------------------- activation --

/// Do what the result names. Documents open by default and are revealed in
/// the Content Browser with Ctrl; assets are revealed by default and open their
/// editor with Ctrl.
pub fn activate(e: &mut Editor, target: &Target, alternate: bool) {
    match target {
        Target::File(path) => {
            let absolute = e.root.join(path);
            let asset = path.ends_with(".epokasset")
                || path.ends_with(".timeline.json")
                || path.ends_with(".particle-effect.json");
            if asset != alternate {
                crate::project_browser::State::select_asset(e, &absolute);
                return;
            }
            let before = e.project_browser.error().map(str::to_owned);
            crate::project_browser::open_path(e, &absolute);
            if let Some(error) = e
                .project_browser
                .error()
                .filter(|error| before.as_deref() != Some(*error))
                .map(str::to_owned)
            {
                e.last_error = Some(error.clone());
                e.log(error);
            } else if path.ends_with(".epokmap") && e.dirty && absolute != e.scene_path() {
                // The "save changes?" prompt lives in the Content Browser.
                e.focus_project = true;
            }
        }
        Target::Folder(path) => crate::project_browser::State::reveal_folder(e, path),
        Target::Actor(id) => {
            if e.scene.actor_index(*id).is_none() {
                e.log("That actor is no longer in the open scene.");
                return;
            }
            e.select_actor(Some(*id));
            e.reveal_selected = true;
            e.search.clear();
            e.view_dirty = true;
            if !alternate {
                e.focus_scene = true;
                e.action("frame-selected");
            }
        }
        Target::Setting { preferences, page } => {
            crate::settings_ui::open_page(e, *preferences, *page)
        }
        Target::Command(id) => match *id {
            "save-all" => {
                e.save_all();
            }
            "reset-layout" => e.reset_layout = true,
            "code-editor" => e.open_code(&e.root.join("assets/scripts"), None),
            "lighting" => e.lighting_window = true,
            "strings" => crate::localization_editor::open(e),
            "pocketstation" => crate::pocketstation_ui::open(e),
            "artifact-dependencies" => e.artifact_dependencies.open = true,
            "memory-report" => e.show_memory_report(),
            "hub" => e.hub_requested = true,
            other => e.action(other),
        },
    }
}

// ----------------------------------------------------------------------- UI --

/// Query field callbacks: Tab cycles the filter instead of moving focus, and
/// a field refocused after a click keeps its text instead of selecting it all.
struct QueryKeys<'a> {
    tab: &'a mut bool,
    keep_caret: &'a mut bool,
}
impl imgui::InputTextCallbackHandler for QueryKeys<'_> {
    fn on_completion(&mut self, _: imgui::TextCallbackData) {
        *self.tab = true;
    }
    fn on_always(&mut self, mut data: imgui::TextCallbackData) {
        if std::mem::take(self.keep_caret) {
            let end = data.str().len();
            data.clear_selection();
            data.set_cursor_pos(end);
        }
    }
}

/// Keep the end of `text` (paths read best from the file end) within `width`.
fn fit_end(ui: &Ui, text: &str, width: f32) -> String {
    if width <= 0. {
        return String::new();
    }
    if ui.calc_text_size(text)[0] <= width {
        return text.into();
    }
    let chars: Vec<char> = text.chars().collect();
    let mut start = 0;
    while start < chars.len() {
        let candidate: String = std::iter::once('\u{2026}')
            .chain(chars[start..].iter().copied())
            .collect();
        if ui.calc_text_size(&candidate)[0] <= width {
            return candidate.replace('\u{2026}', "...");
        }
        start += 1;
    }
    String::new()
}

/// Draw the search when it is open. Call once per frame, after the workspace.
pub fn draw(ui: &Ui, e: &mut Editor) {
    let mut s = std::mem::take(&mut e.global_search);
    s.refresh(&e.root, &e.assets, &e.scene);
    if std::mem::take(&mut s.request_open) {
        ui.open_popup(POPUP);
        s.open = true;
        s.filter = 0;
        s.filtered = None;
    }
    let mut chosen = None;
    if s.open {
        s.rank();
        s.refilter();
        let shown = modal(ui, &mut s, &mut chosen);
        if !shown {
            s.open = false;
        }
    }
    e.global_search = s;
    if let Some((target, alternate)) = chosen {
        let root = e.root.clone();
        e.global_search.remember(&root, &target);
        activate(e, &target, alternate);
    }
}

fn modal(ui: &Ui, s: &mut State, chosen: &mut Option<(Target, bool)>) -> bool {
    let display = ui.io().display_size;
    let width = (display[0] - 40.).clamp(320., 760.);
    let height = (display[1] * 0.64).clamp(240., 580.);
    unsafe {
        imgui::sys::igSetNextWindowPos(
            [display[0] * 0.5, (display[1] * 0.1).max(56.)].into(),
            imgui::sys::ImGuiCond_Always as i32,
            [0.5, 0.].into(),
        );
        imgui::sys::igSetNextWindowSize(
            [width, height].into(),
            imgui::sys::ImGuiCond_Always as i32,
        );
    }
    let _colors = [
        (C::PopupBg, crate::gui::gray(30)),
        (C::Border, crate::gui::gray(62)),
        (C::ModalWindowDimBg, [0., 0., 0., 0.45]),
        (C::FrameBg, crate::gui::gray(19)),
        (C::FrameBgHovered, crate::gui::gray(22)),
        (C::FrameBgActive, crate::gui::gray(19)),
    ]
    .map(|(c, v)| ui.push_style_color(c, v));
    let _vars = [
        ui.push_style_var(V::WindowPadding([0., 0.])),
        ui.push_style_var(V::WindowRounding(8.)),
        ui.push_style_var(V::WindowBorderSize(1.)),
        ui.push_style_var(V::ItemSpacing([6., 0.])),
    ];
    ui.modal_popup_config(POPUP)
        .title_bar(false)
        .resizable(false)
        .movable(false)
        .scroll_bar(false)
        .save_settings(false)
        .build(|| contents(ui, s, chosen, width, height))
        .is_some()
}

fn contents(ui: &Ui, s: &mut State, chosen: &mut Option<(Target, bool)>, width: f32, height: f32) {
    let pad = 14.;
    let io = ui.io();
    // Query field with an inset magnifier.
    ui.set_cursor_pos([pad, pad]);
    let field = ui.cursor_screen_pos();
    let mut tab = false;
    {
        let _vars = [
            ui.push_style_var(V::FramePadding([38., 10.])),
            ui.push_style_var(V::FrameRounding(6.)),
            ui.push_style_var(V::FrameBorderSize(1.)),
        ];
        let _border = ui.push_style_color(C::Border, [0.2, 0.52, 0.85, 1.]);
        if s.focus_input || (!ui.is_any_item_active() && !ui.is_mouse_down(MouseButton::Left)) {
            ui.set_keyboard_focus_here();
            // Opening selects a prefilled query; later refocusing must not.
            s.keep_caret = !std::mem::take(&mut s.focus_input);
        }
        ui.set_next_item_width(width - pad * 2.);
        ui.input_text("##global-search-query", &mut s.query)
            .hint("Search scripts, Blueprints, scenes, assets, actors, settings...")
            .callback(
                imgui::InputTextCallback::COMPLETION | imgui::InputTextCallback::ALWAYS,
                QueryKeys {
                    tab: &mut tab,
                    keep_caret: &mut s.keep_caret,
                },
            )
            .build();
        #[cfg(test)]
        crate::gui::record_script_control(ui, "Global search query");
    }
    let field_height = ui.item_rect_size()[1];
    let draw = ui.get_window_draw_list();
    draw.add_text(
        [
            field[0] + 14.,
            field[1] + (field_height - ui.text_line_height()) * 0.5,
        ],
        [0.55, 0.58, 0.62, 1.],
        "\u{f002}",
    );
    // A changed query re-ranks before the rows below are drawn.
    s.rank();
    // Filter chips with live counts.
    let mut x = pad;
    let mut y = pad + field_height + 10.;
    let chip_height = 24.;
    for (i, (_, label)) in FILTERS.iter().enumerate() {
        let count = s.counts[i];
        let text = format!("{label}  {count}");
        let w = ui.calc_text_size(&text)[0] + 20.;
        if x + w > width - pad {
            x = pad;
            y += chip_height + 6.;
        }
        ui.set_cursor_pos([x, y]);
        let p = ui.cursor_screen_pos();
        if ui.invisible_button(format!("##chip-{label}"), [w, chip_height]) {
            s.filter = i;
        }
        #[cfg(test)]
        crate::gui::record_script_control(ui, &format!("Global search filter {label}"));
        let active = s.filter == i;
        let hovered = ui.is_item_hovered();
        let fill = if active {
            ACCENT
        } else if hovered {
            crate::gui::gray(52)
        } else {
            crate::gui::gray(40)
        };
        draw.add_rect(p, [p[0] + w, p[1] + chip_height], fill)
            .filled(true)
            .rounding(chip_height * 0.5)
            .build();
        let label_width = ui.calc_text_size(label)[0];
        let text_y = p[1] + (chip_height - ui.text_line_height()) * 0.5;
        let strong = if count == 0 && !active {
            crate::gui::gray(120)
        } else {
            crate::gui::gray(230)
        };
        draw.add_text([p[0] + 10., text_y], strong, label);
        draw.add_text(
            [
                p[0] + 10. + label_width + ui.calc_text_size("  ")[0],
                text_y,
            ],
            if active {
                [0.72, 0.84, 0.96, 1.]
            } else {
                crate::gui::gray(130)
            },
            count.to_string(),
        );
        x += w + 6.;
    }
    if tab {
        let n = FILTERS.len();
        s.filter = if io.key_shift {
            (s.filter + n - 1) % n
        } else {
            (s.filter + 1) % n
        };
    }
    s.refilter();
    // Keyboard navigation over the filtered results.
    let page = ((height - y - 90.) / ROW_HEIGHT).max(1.) as usize;
    let last = s.hits.len().saturating_sub(1);
    let previous = s.selected;
    if ui.is_key_pressed(Key::DownArrow) {
        s.selected = (s.selected + 1).min(last);
    }
    if ui.is_key_pressed(Key::UpArrow) {
        s.selected = s.selected.saturating_sub(1);
    }
    if ui.is_key_pressed(Key::PageDown) {
        s.selected = (s.selected + page).min(last);
    }
    if ui.is_key_pressed(Key::PageUp) {
        s.selected = s.selected.saturating_sub(page);
    }
    if s.selected != previous {
        s.scroll_to_selected = true;
    }
    let enter = ui.is_key_pressed(Key::Enter) || ui.is_key_pressed(Key::KeypadEnter);
    if enter && let Some(&index) = s.hits.get(s.selected) {
        *chosen = Some((s.item(index).target.clone(), io.key_ctrl));
    }
    // Results.
    let list_top = y + chip_height + 10.;
    let footer = 34.;
    ui.set_cursor_pos([0., list_top]);
    draw.add_line(
        [ui.window_pos()[0], ui.window_pos()[1] + list_top - 1.],
        [
            ui.window_pos()[0] + width,
            ui.window_pos()[1] + list_top - 1.,
        ],
        crate::gui::gray(44),
    )
    .build();
    let list_height = (height - list_top - footer).max(ROW_HEIGHT);
    // ImGui-rs lends one window draw list at a time; the list draws its own.
    drop(draw);
    {
        let _scrollbar = ui.push_style_var(V::ScrollbarSize(10.));
        ui.child_window("##global-search-results")
            .size([width, list_height])
            .build(|| results(ui, s, chosen, width));
    }
    let draw = ui.get_window_draw_list();
    // Footer: key hints and totals.
    let window = ui.window_pos();
    let footer_top = window[1] + height - footer;
    draw.add_line(
        [window[0], footer_top],
        [window[0] + width, footer_top],
        crate::gui::gray(44),
    )
    .build();
    let alternate =
        s.hits
            .get(s.selected)
            .map(|&i| s.item(i))
            .map_or("Show in Content Browser", |item| match &item.target {
                Target::File(path)
                    if path.ends_with(".epokasset")
                        || path.ends_with(".timeline.json")
                        || path.ends_with(".particle-effect.json") =>
                {
                    "Open editor"
                }
                Target::File(_) => "Show in Content Browser",
                Target::Actor(_) => "Select without framing",
                _ => "",
            });
    let mut hints = vec![("\u{f062}\u{f063}", "Navigate"), ("Enter", "Open")];
    if !alternate.is_empty() {
        hints.push(("Ctrl+Enter", alternate));
    }
    hints.extend([("Tab", "Filter"), ("Esc", "Close")]);
    let mut hx = window[0] + pad;
    let hy = footer_top + (footer - ui.text_line_height()) * 0.5;
    for (key, action) in hints {
        let key_width = ui.calc_text_size(key)[0] + 10.;
        draw.add_rect(
            [hx, hy - 2.],
            [hx + key_width, hy + ui.text_line_height() + 2.],
            crate::gui::gray(44),
        )
        .filled(true)
        .rounding(3.)
        .build();
        draw.add_text([hx + 5., hy], crate::gui::gray(200), key);
        hx += key_width + 6.;
        draw.add_text([hx, hy], crate::gui::gray(140), action);
        hx += ui.calc_text_size(action)[0] + 14.;
    }
    let total = if s.indexing() {
        format!("{} results  |  indexing...", s.hits.len())
    } else {
        format!("{} results  |  {} indexed", s.hits.len(), s.len())
    };
    let total_width = ui.calc_text_size(&total)[0];
    if hx < window[0] + width - pad - total_width {
        draw.add_text(
            [window[0] + width - pad - total_width, hy],
            crate::gui::gray(120),
            &total,
        );
    }
    // Close on Escape or a click outside.
    let outside = ui.is_mouse_clicked(MouseButton::Left)
        && !ui.is_window_hovered_with_flags(
            imgui::WindowHoveredFlags::ROOT_AND_CHILD_WINDOWS
                | imgui::WindowHoveredFlags::ALLOW_WHEN_BLOCKED_BY_ACTIVE_ITEM,
        );
    if chosen.is_some() || ui.is_key_pressed(Key::Escape) || outside {
        ui.close_current_popup();
        s.open = false;
    }
}

fn results(ui: &Ui, s: &mut State, chosen: &mut Option<(Target, bool)>, width: f32) {
    if std::mem::take(&mut s.scroll_top) {
        ui.set_scroll_y(0.);
    }
    if std::mem::take(&mut s.scroll_to_selected) {
        let top = s.selected as f32 * ROW_HEIGHT + 6.;
        let view = ui.window_size()[1];
        let scroll = ui.scroll_y();
        if top < scroll {
            ui.set_scroll_y(top - 6.);
        } else if top + ROW_HEIGHT > scroll + view {
            ui.set_scroll_y(top + ROW_HEIGHT - view + 6.);
        }
    }
    if s.hits.is_empty() {
        let message = if s.indexing() && s.files.is_empty() {
            "Indexing the project...".to_string()
        } else if s.query.trim().is_empty() {
            "Nothing to show in this filter.".to_string()
        } else {
            format!("No results for \"{}\"", s.query.trim())
        };
        let size = ui.calc_text_size(&message);
        ui.set_cursor_pos([(width - size[0]) * 0.5, 36.]);
        ui.text_disabled(&message);
        return;
    }
    ui.set_cursor_pos([0., 6.]);
    let draw = ui.get_window_draw_list();
    let mouse_moved = ui.io().mouse_delta != [0., 0.];
    let scrollbar = if ui.scroll_max_y() > 0. { 10. } else { 0. };
    let row_width = width - scrollbar;
    let mut clipper = imgui::ListClipper::new(s.hits.len() as i32)
        .items_height(ROW_HEIGHT)
        .begin(ui);
    let mut scratch = std::mem::take(&mut s.scratch);
    let mut highlights = std::mem::take(&mut s.highlights);
    while clipper.step() {
        for row in clipper.display_start()..clipper.display_end() {
            let row = row as usize;
            let index = s.hits[row];
            ui.set_cursor_pos([0., 6. + row as f32 * ROW_HEIGHT]);
            let p = ui.cursor_screen_pos();
            let clicked = ui.invisible_button(format!("##hit-{row}"), [row_width, ROW_HEIGHT]);
            let hovered = ui.is_item_hovered();
            if hovered && mouse_moved {
                s.selected = row;
            }
            let item = s.item(index);
            #[cfg(test)]
            crate::gui::record_script_control(ui, &format!("Global search hit {}", item.name));
            if clicked {
                *chosen = Some((item.target.clone(), ui.io().key_ctrl));
            }
            let selected = s.selected == row;
            let (x0, x1) = (p[0] + 8., p[0] + row_width - 8.);
            if selected {
                draw.add_rect([x0, p[1] + 1.], [x1, p[1] + ROW_HEIGHT - 1.], ACCENT)
                    .filled(true)
                    .rounding(5.)
                    .build();
            } else if hovered {
                draw.add_rect(
                    [x0, p[1] + 1.],
                    [x1, p[1] + ROW_HEIGHT - 1.],
                    crate::gui::gray(40),
                )
                .filled(true)
                .rounding(5.)
                .build();
            }
            let text_y = p[1] + (ROW_HEIGHT - ui.text_line_height()) * 0.5;
            let (glyph, color) = icon(item.kind);
            let glyph_width = ui.calc_text_size(glyph)[0];
            draw.add_text([x0 + 10. + (18. - glyph_width) * 0.5, text_y], color, glyph);
            // Type tag at the right.
            let tag_width = ui.calc_text_size(item.kind)[0] + 14.;
            let tag_x = x1 - 8. - tag_width;
            draw.add_rect(
                [tag_x, p[1] + 6.],
                [tag_x + tag_width, p[1] + ROW_HEIGHT - 6.],
                if selected {
                    [0.05, 0.2, 0.36, 1.]
                } else {
                    crate::gui::gray(42)
                },
            )
            .filled(true)
            .rounding(4.)
            .build();
            draw.add_text(
                [tag_x + 7., text_y],
                if selected {
                    [0.78, 0.87, 0.97, 1.]
                } else {
                    crate::gui::gray(165)
                },
                item.kind,
            );
            // Name with matched letters highlighted, then the dimmed detail.
            let recent = s.query.trim().is_empty() && row < s.recent_count;
            if !s.terms.is_empty() {
                score(item, &s.terms, &mut scratch, Some(&mut highlights));
            } else {
                highlights.clear();
            }
            let mut x = x0 + 36.;
            let name_limit = tag_x - 12.;
            let mut run = String::new();
            let mut run_matched = false;
            let flush = |run: &mut String, matched: bool, x: &mut f32| {
                if run.is_empty() {
                    return;
                }
                let w = ui.calc_text_size(&*run)[0];
                if *x + w <= name_limit {
                    draw.add_text(
                        [*x, text_y],
                        if matched {
                            MATCHED
                        } else {
                            crate::gui::gray(235)
                        },
                        &*run,
                    );
                }
                *x += w;
                run.clear();
            };
            for (i, c) in item.name.chars().enumerate() {
                let matched = highlights.contains(&i);
                if matched != run_matched {
                    flush(&mut run, run_matched, &mut x);
                    run_matched = matched;
                }
                run.push(c);
            }
            flush(&mut run, run_matched, &mut x);
            let mut detail = item.detail.clone();
            if recent {
                detail = if detail.is_empty() {
                    "recent".into()
                } else {
                    format!("{detail}  |  recent")
                };
            }
            let detail = fit_end(ui, &detail, name_limit - x - 12.);
            if !detail.is_empty() {
                draw.add_text(
                    [x + 12., text_y],
                    if selected {
                        [0.70, 0.80, 0.90, 1.]
                    } else {
                        crate::gui::gray(128)
                    },
                    &detail,
                );
            }
        }
    }
    s.scratch = scratch;
    s.highlights = highlights;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chars(s: &str) -> Vec<char> {
        s.chars().collect()
    }
    fn fz(pattern: &str, text: &str) -> Option<i32> {
        let pattern = terms(pattern).concat();
        fuzzy(&pattern, &chars(text), &mut Scratch::default(), None)
    }
    fn positions(pattern: &str, text: &str) -> Vec<usize> {
        let mut out = Vec::new();
        fuzzy(
            &chars(pattern),
            &chars(text),
            &mut Scratch::default(),
            Some(&mut out),
        )
        .unwrap();
        out
    }
    fn kinds() -> HashMap<String, assets::Kind> {
        HashMap::from([
            (
                "assets/Textures/Brick.epokasset".into(),
                assets::Kind::Texture,
            ),
            (
                "assets/Audio/Theme.epokasset".into(),
                assets::Kind::MusicSequence,
            ),
            (
                "assets/Meshes/Crate.epokasset".into(),
                assets::Kind::EditableMesh,
            ),
        ])
    }
    fn index(paths: &[(&str, bool)]) -> State {
        let kinds = kinds();
        let mut state = State {
            files: paths
                .iter()
                .filter_map(|(path, folder)| file_item(path, *folder, &kinds))
                .collect(),
            statics: static_items(),
            ..Default::default()
        };
        state
            .files
            .sort_by(|a, b| a.target.path().cmp(&b.target.path()));
        state.items_revision = 1;
        state
    }
    fn ranked(state: &mut State, query: &str) -> Vec<String> {
        state.query = query.into();
        state.rank();
        state.refilter();
        state.hits().map(|item| item.name.clone()).collect()
    }

    #[test]
    fn fuzzy_requires_an_ordered_subsequence_and_ignores_case() {
        assert!(fz("tpc", "ThirdPersonController").is_some());
        assert!(fz("TPC", "thirdpersoncontroller").is_some());
        assert!(fz("ctp", "ThirdPersonController").is_none());
        assert!(fz("spinnerx", "Spinner").is_none());
        assert_eq!(fz("", "anything"), Some(0));
    }

    #[test]
    fn fuzzy_prefers_word_starts_camel_humps_and_runs() {
        // camelCase initials over the same letters scattered mid-word.
        assert!(fz("tpc", "ThirdPersonController") > fz("tpc", "outputconfig"));
        // A contiguous run beats the same letters spread out.
        assert!(fz("door", "DungeonDoor") > fz("door", "DrawOutOfRange"));
        // A word start beats a match inside a word.
        assert!(fz("key", "DungeonKey") > fz("key", "Monkey"));
        // Earlier is better for otherwise equal matches.
        assert!(fz("map", "MapSettings") > fz("map", "WorldMapSettings"));
    }

    #[test]
    fn fuzzy_reports_the_best_alignment_not_the_first() {
        // Greedy matching would take the "l" of "Player"; the best alignment
        // uses the word starts of Light and Controller.
        assert_eq!(positions("lc", "PlayerLightController"), vec![6, 11]);
        assert!(fz("plc", "PlayerLightController").is_some());
        assert_eq!(positions("sc", "SpinnerScript"), vec![7, 8]);
        assert_eq!(positions("door", "Door 0 Left"), vec![0, 1, 2, 3]);
    }

    #[test]
    fn files_are_classified_like_the_content_browser() {
        let kinds = kinds();
        let item = |path: &str| file_item(path, false, &kinds);
        let cpp = item("assets/scripts/Spinner.cpp").unwrap();
        assert_eq!(
            (cpp.name.as_str(), cpp.kind, cpp.category),
            ("Spinner.cpp", "C++", Category::Script)
        );
        assert_eq!(cpp.detail, "assets/scripts");
        assert_eq!(item("assets/scripts/Spinner.hpp").unwrap().kind, "C++");
        let lua = item("assets/scripts/Guard.lua").unwrap();
        assert_eq!((lua.kind, lua.category), ("Lua", Category::Script));
        let bp = item("assets/blueprints/BP_Door.epokbp").unwrap();
        assert_eq!(
            (bp.name.as_str(), bp.kind, bp.category),
            ("BP_Door", "Blueprint", Category::Blueprint)
        );
        assert_eq!(item("assets/ui/Hud.epokclass").unwrap().kind, "UI Class");
        let scene = item("assets/scenes/Main.epokmap").unwrap();
        assert_eq!(
            (scene.name.as_str(), scene.category),
            ("Main", Category::Scene)
        );
        assert_eq!(
            scene.target,
            Target::File("assets/scenes/Main.epokmap".into())
        );
        assert_eq!(
            item("assets/Textures/Brick.epokasset").unwrap().kind,
            "Texture"
        );
        assert_eq!(item("assets/Audio/Theme.epokasset").unwrap().kind, "Audio");
        assert_eq!(item("assets/Meshes/Crate.epokasset").unwrap().kind, "Mesh");
        assert_eq!(item("assets/Unknown/Lost.epokasset").unwrap().kind, "Asset");
        assert_eq!(
            item("assets/fx/Spark.particle-effect.json").unwrap().name,
            "Spark"
        );
        assert_eq!(
            item("assets/seq/Intro.timeline.json").unwrap().kind,
            "Timeline"
        );
        // Raw import sources and sidecars stay out, as the browser hides them.
        for hidden in [
            "assets/Textures/brick.png",
            "assets/Audio/theme.wav",
            "assets/scripts/Spinner.epokscript",
            "assets/notes.md",
        ] {
            assert!(item(hidden).is_none(), "{hidden}");
        }
        let folder = file_item("assets/Characters", true, &kinds).unwrap();
        assert_eq!(
            (folder.kind, folder.target),
            ("Folder", Target::Folder("assets/Characters".into()))
        );
    }

    #[test]
    fn walk_lists_project_files_and_folders_but_not_hidden_ones() {
        let root =
            std::env::temp_dir().join(format!("epok-global-search-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(root.join("assets/scripts")).unwrap();
        fs::create_dir_all(root.join("assets/.cache")).unwrap();
        fs::write(root.join("assets/scripts/Guard.lua"), "").unwrap();
        fs::write(root.join("assets/.cache/x.epokasset"), "").unwrap();
        fs::write(root.join("assets/.hidden.lua"), "").unwrap();
        let entries = walk(&root);
        assert_eq!(
            entries,
            vec![
                ("assets/scripts".to_string(), true),
                ("assets/scripts/Guard.lua".to_string(), false),
            ]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn ranking_puts_the_intended_result_first() {
        let mut state = index(&[
            ("assets/scripts/ThirdPersonController.cpp", false),
            ("assets/scripts/ThirdPersonController.hpp", false),
            ("assets/scripts/TopicConfig.lua", false),
            ("assets/scripts/Spinner.cpp", false),
            ("assets/blueprints/BP_Spinner.epokbp", false),
            ("assets/scenes/Main.epokmap", false),
            ("assets/scenes/MainMenu.epokmap", false),
            ("assets/Textures/Brick.epokasset", false),
            ("assets/Textures", true),
        ]);
        assert!(ranked(&mut state, "tpc")[0].starts_with("ThirdPersonController"));
        // The exact name wins over longer names containing it.
        assert_eq!(ranked(&mut state, "main")[0], "Main");
        assert_eq!(ranked(&mut state, "spinner")[..2].len(), 2);
        assert!(ranked(&mut state, "spinner")[0] == "Spinner.cpp");
        // Every term must match: name or path.
        assert_eq!(ranked(&mut state, "spin bluep"), vec!["BP_Spinner"]);
        assert_eq!(ranked(&mut state, "textures brick"), vec!["Brick"]);
        assert!(ranked(&mut state, "zzz").is_empty());
        // Settings pages and commands are searchable by their keywords.
        assert!(ranked(&mut state, "dithering").contains(&"Rendering".to_string()));
        assert_eq!(ranked(&mut state, "package disc")[0], "Package Disc");
    }

    #[test]
    fn filters_count_and_restrict_categories() {
        let mut state = index(&[
            ("assets/scripts/Door.cpp", false),
            ("assets/blueprints/BP_Door.epokbp", false),
            ("assets/scenes/DoorTest.epokmap", false),
        ]);
        ranked(&mut state, "door");
        assert_eq!(state.counts[0], 3);
        assert_eq!(state.counts[Category::Script.filter()], 1);
        assert_eq!(state.counts[Category::Blueprint.filter()], 1);
        assert_eq!(state.counts[Category::Scene.filter()], 1);
        state.filter = Category::Scene.filter();
        state.refilter();
        assert_eq!(
            state.hits().map(|i| i.name.as_str()).collect::<Vec<_>>(),
            ["DoorTest"]
        );
    }

    #[test]
    fn narrowing_queries_reuse_previous_matches_and_stay_correct() {
        let mut state = index(&[
            ("assets/scripts/Door.cpp", false),
            ("assets/scripts/DoorLock.cpp", false),
            ("assets/scripts/Window.cpp", false),
        ]);
        assert_eq!(ranked(&mut state, "door"), vec!["Door.cpp", "DoorLock.cpp"]);
        assert_eq!(ranked(&mut state, "doorl"), vec!["DoorLock.cpp"]);
        // Shortening the query ranks everything again.
        let window = ranked(&mut state, "win");
        assert_eq!(window[0], "Window.cpp");
        assert!(!window.contains(&"DoorLock.cpp".to_string()));
    }

    #[test]
    fn empty_query_lists_recent_items_first() {
        let mut state = index(&[
            ("assets/scripts/A.cpp", false),
            ("assets/scripts/B.cpp", false),
            ("assets/scenes/Level.epokmap", false),
        ]);
        state.prefs.recent = vec![
            "file:assets/scenes/Level.epokmap".into(),
            "command:build".into(),
            "file:assets/missing.epokbp".into(),
        ];
        let names = ranked(&mut state, "");
        assert_eq!(&names[..2], ["Level", "Build"]);
        assert_eq!(state.recent_count, 2);
        assert_eq!(names.len(), state.len());
        // Recency also breaks ties for a typed query.
        state.prefs.recent = vec!["file:assets/scripts/B.cpp".into()];
        assert_eq!(ranked(&mut state, "cpp")[0], "B.cpp");
    }

    #[test]
    fn ranking_a_large_project_stays_interactive() {
        let mut paths = Vec::new();
        for folder in 0..250 {
            for file in 0..200 {
                paths.push(format!(
                    "assets/Area{folder:03}/Prop{file:03}_{}.epokasset",
                    ["Crate", "Barrel", "Torch", "Door", "Lever"][file % 5]
                ));
            }
        }
        paths.push("assets/scripts/ThirdPersonController.cpp".into());
        let kinds = HashMap::new();
        let mut state = State {
            files: paths
                .iter()
                .filter_map(|p| file_item(p, false, &kinds))
                .collect(),
            statics: static_items(),
            items_revision: 1,
            ..Default::default()
        };
        assert_eq!(state.files.len(), 50_001);
        let started = std::time::Instant::now();
        for query in ["t", "th", "thi", "thir", "third", "thirdp", "thirdpc"] {
            ranked(&mut state, query);
        }
        let elapsed = started.elapsed();
        assert_eq!(
            state.hits().next().unwrap().name,
            "ThirdPersonController.cpp"
        );
        eprintln!("7 keystrokes over 50k items: {elapsed:?}");
        // Generous for unoptimized test builds; release is far below a frame.
        assert!(elapsed.as_secs_f64() < 20., "{elapsed:?}");
    }

    #[test]
    fn each_kind_of_result_does_what_its_editor_would() {
        use crate::workspace::{CreateOptions, GameplayFlavor, Template};
        let root =
            std::env::temp_dir().join(format!("epok-global-search-open-{}", uuid::Uuid::new_v4()));
        let project = crate::workspace::create_with_options(
            &root,
            "Search Activate",
            CreateOptions::new(Template::Sample).with_gameplay(GameplayFlavor::Blueprint),
        )
        .unwrap();
        fs::copy(
            root.join("assets/scenes/SampleScene.epokmap"),
            root.join("assets/scenes/Other.epokmap"),
        )
        .unwrap();
        fs::create_dir_all(root.join("assets/Textures")).unwrap();
        fs::write(root.join("assets/Textures/Brick.epokasset"), b"placeholder").unwrap();
        let mut e = Editor::open(project).unwrap();
        e.auto_build = false;
        let file = |path: &str| Target::File(path.into());

        // Blueprints open in the Blueprint editor; Ctrl reveals them instead.
        activate(&mut e, &file("assets/Blueprints/Spinner.epokbp"), false);
        assert!(e.blueprint_editor.open);
        assert!(
            e.blueprint_editor
                .path
                .as_ref()
                .unwrap()
                .ends_with("Spinner.epokbp")
        );
        activate(&mut e, &file("assets/Blueprints/Spinner.epokbp"), true);
        assert_eq!(
            e.selected_asset,
            Some(e.root.join("assets/Blueprints/Spinner.epokbp"))
        );

        // Assets are revealed in the Content Browser, in their folder.
        e.focus_project = false;
        activate(&mut e, &file("assets/Textures/Brick.epokasset"), false);
        assert_eq!(
            e.selected_asset,
            Some(e.root.join("assets/Textures/Brick.epokasset"))
        );
        assert_eq!(e.assets.folder, "assets/Textures");
        assert!(e.focus_project);
        activate(&mut e, &Target::Folder("assets/scenes".into()), false);
        assert_eq!(e.assets.folder, "assets/scenes");

        // The open scene is focused; another one loads, or asks to save first.
        e.focus_scene = false;
        activate(&mut e, &file("assets/scenes/SampleScene.epokmap"), false);
        assert!(e.focus_scene && e.scene_loading.is_none());
        e.dirty = true;
        e.focus_project = false;
        activate(&mut e, &file("assets/scenes/Other.epokmap"), false);
        assert!(
            e.scene_loading.is_none(),
            "unsaved edits are never replaced silently"
        );
        assert!(e.focus_project, "the save prompt is in the Content Browser");
        e.dirty = false;
        e.project_browser = Default::default();
        activate(&mut e, &file("assets/scenes/Other.epokmap"), false);
        assert!(
            e.scene_loading.is_some(),
            "a clean editor starts loading the scene"
        );
        let started = std::time::Instant::now();
        while e.scene_loading.is_some() {
            assert!(started.elapsed().as_secs() < 10, "scene loading timed out");
            e.tick();
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(e.scene_path().ends_with("Other.epokmap"));

        // Actors are selected and revealed in the Hierarchy.
        let actor = e
            .scene
            .actors
            .first()
            .expect("the sample scene has actors")
            .id;
        e.search = "filter".into();
        activate(&mut e, &Target::Actor(actor), true);
        assert_eq!(e.selected_actor, Some(actor));
        assert!(e.reveal_selected && e.search.is_empty());

        // Settings pages and commands.
        activate(
            &mut e,
            &Target::Setting {
                preferences: false,
                page: 2,
            },
            false,
        );
        assert!(e.settings.project_open);
        activate(
            &mut e,
            &Target::Setting {
                preferences: true,
                page: 0,
            },
            false,
        );
        assert!(e.settings.preferences_open);
        activate(&mut e, &Target::Command("lighting"), false);
        assert!(e.lighting_window);
        drop(e);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn modal_opens_types_navigates_and_opens_a_script() {
        use imgui::Key;
        let root =
            std::env::temp_dir().join(format!("epok-global-search-ui-{}", uuid::Uuid::new_v4()));
        let project =
            crate::workspace::create(&root, "Search Input", crate::workspace::Template::Sample)
                .unwrap();
        let mut e = Editor::open(project).unwrap();
        e.auto_build = false;
        let mut ctx = crate::gui::tests::imgui_context();
        crate::gui::configure_input(ctx.io_mut());
        ctx.io_mut().display_size = [1280., 800.];
        ctx.io_mut().delta_time = 1. / 60.;
        ctx.fonts()
            .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
        ctx.fonts().build_rgba32_texture();
        let frame = |ctx: &mut imgui::Context, e: &mut Editor| {
            let ui = ctx.frame();
            draw(ui, e);
            ctx.render();
        };
        let key = |ctx: &mut imgui::Context, e: &mut Editor, key: Key| {
            ctx.io_mut().add_key_event(key, true);
            frame(ctx, e);
            ctx.io_mut().add_key_event(key, false);
            frame(ctx, e);
        };
        e.global_search.open_with("");
        // Wait for the worker walk of the sample project.
        let started = std::time::Instant::now();
        while e.global_search.walk.is_some() || e.global_search.walked.is_none() {
            assert!(started.elapsed().as_secs() < 10, "index walk timed out");
            frame(&mut ctx, &mut e);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        frame(&mut ctx, &mut e);
        assert!(e.global_search.is_open());
        assert!(
            e.global_search
                .files
                .iter()
                .any(|i| i.name == "Spinner.cpp")
        );
        for c in "spinner".chars() {
            ctx.io_mut().add_input_character(c);
            frame(&mut ctx, &mut e);
        }
        frame(&mut ctx, &mut e);
        let names = e
            .global_search
            .hits()
            .map(|i| i.name.clone())
            .collect::<Vec<_>>();
        assert!(names.len() >= 2, "{names:?}");
        assert!(names[..2].contains(&"Spinner.cpp".to_string()), "{names:?}");
        // Down moves the selection; Enter opens the selected script.
        let first = names[0].clone();
        key(&mut ctx, &mut e, Key::DownArrow);
        assert_eq!(e.global_search.selected, 1);
        key(&mut ctx, &mut e, Key::UpArrow);
        assert_eq!(e.global_search.selected, 0);
        assert!(!e.script_editor.open);
        key(&mut ctx, &mut e, Key::Enter);
        assert!(!e.global_search.is_open(), "Enter closes the search");
        assert!(
            e.script_editor.open,
            "Enter opened {first} in the code editor"
        );
        assert!(e.global_search.prefs.recent[0].ends_with(&first));
        // Tab cycles the filter; Escape closes without acting.
        e.global_search.open_with("main");
        frame(&mut ctx, &mut e);
        frame(&mut ctx, &mut e);
        key(&mut ctx, &mut e, Key::Tab);
        assert_eq!(e.global_search.filter, 1);
        assert!(e.global_search.is_open(), "Tab stays in the search");
        // Clicking a filter chip keeps typing into the same query.
        let chip = crate::gui::script_control("Global search filter Scenes").unwrap();
        ctx.io_mut().add_mouse_pos_event(chip);
        frame(&mut ctx, &mut e);
        ctx.io_mut()
            .add_mouse_button_event(imgui::MouseButton::Left, true);
        frame(&mut ctx, &mut e);
        ctx.io_mut()
            .add_mouse_button_event(imgui::MouseButton::Left, false);
        for _ in 0..3 {
            frame(&mut ctx, &mut e);
        }
        assert_eq!(e.global_search.filter, Category::Scene.filter());
        ctx.io_mut().add_input_character('x');
        frame(&mut ctx, &mut e);
        assert_eq!(e.global_search.query, "mainx");
        key(&mut ctx, &mut e, Key::Escape);
        assert!(!e.global_search.is_open());
        drop(ctx);
        let _ = fs::remove_dir_all(root);
    }
}
