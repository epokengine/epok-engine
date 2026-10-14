//! Dungeon Recipes: designer-authored steps that grow a room graph, then lay it out.
//!
//! A recipe runs its steps in order over a lattice of possible rooms. Paths grow as
//! self-avoiding walks: the main path from the spawn to the goal, then branches that
//! end in dead ends or loop back. Items land by depth along their paths; a key and
//! lock pair locks a link so that the rooms behind it need the key, while the key stays
//! reachable without it. A simulation that walks from the spawn collecting keys checks
//! every placement, so a generated dungeon is always solvable. One-way doors turn
//! links that lead back towards the spawn into shortcuts. When a step cannot be
//! satisfied, the whole recipe runs again with the next attempt's random stream.
//!
//! The finished graph becomes an ordinary `Layout`: one room per node inside its
//! lattice cell, corridors between linked nodes, and the locks, one-way doors and items
//! as spots for the dungeon's style.
use crate::assets;
use crate::dungeon::{Cell, GuardedDoor, Item, Layout, Room, Side};
use crate::dungeon_random::Random;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use uuid::Uuid;

pub const VERSION: u32 = 1;
pub const MAX_STEPS: usize = 64;
pub const LATTICE_LIMITS: (u16, u16) = (2, 16);
pub const ATTEMPTS: u32 = 64;
const STAGE_RECIPE: u32 = 20;

/// A path a step grows or reads, by name.
pub const MAIN: &str = "main";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "step")]
pub enum Step {
    /// The lattice of possible rooms, and how many grid cells each lattice cell spans.
    Lattice { size: [u16; 2], cell: u16 },
    /// The path from the spawn (its first room) to the goal (its last).
    MainPath { length: [u16; 2] },
    /// Paths starting next to rooms of `from`. With `to` paths they must end next to one
    /// of their rooms and connect to it, closing a loop; otherwise they are dead ends.
    Branch {
        name: String,
        from: Vec<String>,
        #[serde(default)]
        to: Vec<String>,
        count: u16,
        length: [u16; 2],
    },
    /// Named items in rooms of `paths`, between `depth` percent of each path's length,
    /// placed in their room as `place` says.
    Items {
        name: String,
        paths: Vec<String>,
        count: [u16; 2],
        depth: [u8; 2],
        #[serde(default)]
        place: Place,
    },
    /// A key in a room of `key_paths` and the lock it opens on a link into a room of
    /// `lock_paths`.
    KeyLock {
        key_paths: Vec<String>,
        lock_paths: Vec<String>,
    },
    /// Links that lead back towards the spawn become one-way, with this chance.
    OneWay { chance: u8 },
    /// Rooms of `paths` become caves: rock fills `density` percent of the room, then
    /// `smoothing` passes of cellular automaton round it into open chambers.
    Caves {
        paths: Vec<String>,
        density: u8,
        smoothing: u8,
    },
    /// Rooms of `paths` are made by hand: each takes one of these Dungeon Rooms that fits
    /// its slot and has doors towards its links, turned as needed. `most` caps how many
    /// rooms the step makes (0: every room of the paths).
    HandmadeRooms {
        paths: Vec<String>,
        rooms: Vec<Uuid>,
        #[serde(default)]
        most: u8,
    },
    /// Rooms climb and drop between floor levels: each link changes level by one with
    /// `chance` percent, between level 0 (the spawn's) and `highest`, and its corridor
    /// gets one stair row per level it climbs.
    Levels { chance: u8, highest: u8 },
    /// How rooms fill their lattice cells, whether their corners are cut, and the
    /// corridor width.
    Rooms {
        fill: [u8; 2],
        corridor_width: u16,
        #[serde(default)]
        rounded: bool,
    },
}

/// Where in its room an item goes.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum Place {
    /// Any free floor cell, away from the doors.
    #[default]
    Anywhere,
    /// The room's centre, or the free cell nearest to it.
    Centre,
    /// A free floor cell against a wall.
    Edge,
}
impl Place {
    pub const ALL: [Place; 3] = [Place::Anywhere, Place::Centre, Place::Edge];
    pub fn label(self) -> &'static str {
        match self {
            Place::Anywhere => "Anywhere",
            Place::Centre => "Centre",
            Place::Edge => "Against a wall",
        }
    }
}
impl Step {
    pub fn label(&self) -> &'static str {
        match self {
            Step::Lattice { .. } => "Lattice",
            Step::MainPath { .. } => "Main path",
            Step::Branch { .. } => "Branch",
            Step::Items { .. } => "Items",
            Step::KeyLock { .. } => "Key and lock",
            Step::OneWay { .. } => "One-way doors",
            Step::Caves { .. } => "Caves",
            Step::Levels { .. } => "Levels",
            Step::HandmadeRooms { .. } => "Handmade rooms",
            Step::Rooms { .. } => "Rooms",
        }
    }
    pub fn description(&self) -> &'static str {
        match self {
            Step::Lattice { .. } => {
                "The grid of possible rooms. Each lattice cell holds at most one room."
            }
            Step::MainPath { .. } => "The path from the spawn room to the goal room.",
            Step::Branch { .. } => {
                "Side paths off other paths: dead ends, or loops when they lead back."
            }
            Step::Items { .. } => "Named items in rooms of some paths, by depth along them.",
            Step::KeyLock { .. } => "A key and the door it opens, always solvable.",
            Step::OneWay { .. } => "Shortcuts back towards the spawn that open from one side only.",
            Step::Caves { .. } => "Rooms of some paths become organic caves instead of rectangles.",
            Step::Levels { .. } => {
                "Rooms on different floor levels, joined by stairs in their corridors."
            }
            Step::HandmadeRooms { .. } => {
                "Rooms of some paths are Dungeon Rooms made by hand, turned to fit their doors."
            }
            Step::Rooms { .. } => {
                "How much of its lattice cell each room fills, and corridor width."
            }
        }
    }
    /// One step of each kind, as the editor adds them.
    pub fn defaults() -> Vec<Step> {
        vec![
            Step::Lattice {
                size: [6, 5],
                cell: 9,
            },
            Step::MainPath { length: [6, 9] },
            Step::Branch {
                name: "branch".into(),
                from: vec![MAIN.into()],
                to: Vec::new(),
                count: 1,
                length: [2, 3],
            },
            Step::Items {
                name: "Treasure".into(),
                paths: vec![MAIN.into()],
                count: [1, 2],
                depth: [60, 100],
                place: Place::Anywhere,
            },
            Step::KeyLock {
                key_paths: vec![MAIN.into()],
                lock_paths: vec![MAIN.into()],
            },
            Step::OneWay { chance: 50 },
            Step::Caves {
                paths: vec![MAIN.into()],
                density: 45,
                smoothing: 3,
            },
            Step::Levels {
                chance: 40,
                highest: 2,
            },
            Step::HandmadeRooms {
                paths: vec![MAIN.into()],
                rooms: Vec::new(),
                most: 0,
            },
            Step::Rooms {
                fill: [60, 100],
                corridor_width: 1,
                rounded: false,
            },
        ]
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Recipe {
    pub version: u32,
    pub steps: Vec<Step>,
}
impl Default for Recipe {
    fn default() -> Self {
        Self {
            version: VERSION,
            steps: Vec::new(),
        }
    }
}

/// The recipe a generator uses when it has none: a main path, two dead-end cave
/// branches with treasure, a loop, a key on a branch for a lock on the main path,
/// one-way shortcuts back, and rooms with cut corners.
pub fn builtin() -> Recipe {
    Recipe {
        version: VERSION,
        steps: vec![
            Step::Lattice {
                size: [6, 5],
                cell: 10,
            },
            Step::MainPath { length: [6, 9] },
            Step::Branch {
                name: "branch".into(),
                from: vec![MAIN.into()],
                to: Vec::new(),
                count: 2,
                length: [2, 3],
            },
            Step::Branch {
                name: "loop".into(),
                from: vec![MAIN.into()],
                to: vec![MAIN.into()],
                count: 1,
                length: [2, 4],
            },
            Step::Items {
                name: "Treasure".into(),
                paths: vec!["branch".into()],
                count: [1, 2],
                depth: [60, 100],
                place: Place::Anywhere,
            },
            Step::KeyLock {
                key_paths: vec!["branch".into()],
                lock_paths: vec![MAIN.into()],
            },
            Step::OneWay { chance: 50 },
            Step::Caves {
                paths: vec!["branch".into()],
                density: 42,
                smoothing: 3,
            },
            Step::Levels {
                chance: 45,
                highest: 2,
            },
            Step::Rooms {
                fill: [60, 100],
                corridor_width: 1,
                rounded: true,
            },
        ],
    }
}

impl Recipe {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let recipe: Self = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        recipe.validate()?;
        Ok(recipe)
    }
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        serde_json::to_vec_pretty(self).map_err(|e| e.to_string())
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != VERSION {
            return Err(format!("Dungeon Recipe version {VERSION} required"));
        }
        if self.steps.len() > MAX_STEPS {
            return Err(format!("A Dungeon Recipe holds at most {MAX_STEPS} steps"));
        }
        let lattices = self
            .steps
            .iter()
            .filter(|s| matches!(s, Step::Lattice { .. }))
            .count();
        if lattices > 1 {
            return Err("A recipe has one Lattice step".into());
        }
        if self
            .steps
            .iter()
            .filter(|s| matches!(s, Step::MainPath { .. }))
            .count()
            != 1
        {
            return Err("A recipe needs exactly one Main path step".into());
        }
        let mut known: BTreeSet<&str> = BTreeSet::new();
        for (index, step) in self.steps.iter().enumerate() {
            let at = |message: String| format!("Step {} ({}): {message}", index + 1, step.label());
            let range = |r: [u16; 2], lo: u16, hi: u16, what: &str| {
                if r[0] < lo || r[1] > hi || r[0] > r[1] {
                    Err(at(format!("{what} must be {lo}-{hi}, smallest first")))
                } else {
                    Ok(())
                }
            };
            let paths = |names: &[String], known: &BTreeSet<&str>| -> Result<(), String> {
                if names.is_empty() {
                    return Err(at("name at least one path".into()));
                }
                for name in names {
                    if !known.contains(name.as_str()) {
                        return Err(at(format!("no earlier step grows a path named \"{name}\"")));
                    }
                }
                Ok(())
            };
            match step {
                Step::Lattice { size, cell } => {
                    let (lo, hi) = LATTICE_LIMITS;
                    if size.iter().any(|s| !(lo..=hi).contains(s)) {
                        return Err(at(format!("the lattice is {lo}-{hi} cells per side")));
                    }
                    if !(min_cell(1)..=16).contains(cell) {
                        return Err(at(format!(
                            "each lattice cell spans {}-16 grid cells",
                            min_cell(1)
                        )));
                    }
                    if usize::from(size[0].max(size[1])) * usize::from(*cell) + 2 > 128 {
                        return Err(at("the lattice spans more than 128 grid cells".into()));
                    }
                }
                Step::MainPath { length } => {
                    range(*length, 2, 64, "Length")?;
                    known.insert(MAIN);
                }
                Step::Branch {
                    name,
                    from,
                    to,
                    count,
                    length,
                } => {
                    if name.trim().is_empty() || name.len() > 32 {
                        return Err(at("name the branch (up to 32 characters)".into()));
                    }
                    paths(from, &known)?;
                    for target in to {
                        if !known.contains(target.as_str()) && target != name {
                            return Err(at(format!(
                                "no earlier step grows a path named \"{target}\""
                            )));
                        }
                    }
                    if !(1..=16).contains(count) {
                        return Err(at("Count must be 1-16".into()));
                    }
                    range(*length, 1, 32, "Length")?;
                    known.insert(name.as_str());
                }
                Step::Items {
                    name,
                    paths: names,
                    count,
                    depth,
                    ..
                } => {
                    if name.trim().is_empty() || name.len() > 32 {
                        return Err(at("name the items (up to 32 characters)".into()));
                    }
                    paths(names, &known)?;
                    range(*count, 1, 32, "Count")?;
                    if depth[0] > depth[1] || depth[1] > 100 {
                        return Err(at("Depth is a percentage range, 0-100".into()));
                    }
                }
                Step::KeyLock {
                    key_paths,
                    lock_paths,
                } => {
                    paths(key_paths, &known)?;
                    paths(lock_paths, &known)?;
                }
                Step::OneWay { chance } => {
                    if *chance > 100 {
                        return Err(at("Chance is a percentage, 0-100".into()));
                    }
                }
                Step::Caves {
                    paths: names,
                    density,
                    smoothing,
                } => {
                    paths(names, &known)?;
                    if !(20..=70).contains(density) {
                        return Err(at("Rock is 20-70 percent".into()));
                    }
                    if !(1..=8).contains(smoothing) {
                        return Err(at("Smoothing is 1-8 passes".into()));
                    }
                }
                Step::HandmadeRooms {
                    paths: names,
                    rooms,
                    most,
                } => {
                    paths(names, &known)?;
                    if rooms.len() > 32 {
                        return Err(at("Up to 32 rooms per step".into()));
                    }
                    if *most > 16 {
                        return Err(at(
                            "At most 16 handmade rooms per step, or 0 for every room".into(),
                        ));
                    }
                }
                Step::Levels { chance, highest } => {
                    if *chance > 100 {
                        return Err(at("Chance is a percentage, 0-100".into()));
                    }
                    if !(1..=4).contains(highest) {
                        return Err(at("Highest level is 1-4".into()));
                    }
                }
                Step::Rooms {
                    fill,
                    corridor_width,
                    ..
                } => {
                    if fill[0] < 20 || fill[1] > 100 || fill[0] > fill[1] {
                        return Err(at("Fill is 20-100 percent, smallest first".into()));
                    }
                    if !(1..=3).contains(corridor_width) {
                        return Err(at("Corridor width is 1-3 cells".into()));
                    }
                }
            }
        }
        let ((_, span), (_, width, _)) = (self.lattice(), self.rooms());
        if span < i32::from(min_cell(width as u16)) {
            return Err(format!(
                "Corridors {width} wide need lattice cells of at least {} grid cells",
                min_cell(width as u16)
            ));
        }
        Ok(())
    }
    fn lattice(&self) -> ([i32; 2], i32) {
        self.steps
            .iter()
            .find_map(|s| match s {
                Step::Lattice { size, cell } => {
                    Some(([i32::from(size[0]), i32::from(size[1])], i32::from(*cell)))
                }
                _ => None,
            })
            .unwrap_or(([6, 5], 9))
    }
    /// Room fill, corridor width and whether corners are cut.
    fn rooms(&self) -> ([u8; 2], i32, bool) {
        self.steps
            .iter()
            .find_map(|s| match s {
                Step::Rooms {
                    fill,
                    corridor_width,
                    rounded,
                } => Some((*fill, i32::from(*corridor_width), *rounded)),
                _ => None,
            })
            .unwrap_or(([55, 90], 1, false))
    }
    /// The Dungeon Rooms each path's rooms are made from, with the step that says so and
    /// how many rooms that step makes at most (0: every one); later steps win.
    pub fn handmade(&self) -> BTreeMap<String, (usize, u8, Vec<Uuid>)> {
        let mut rooms = BTreeMap::new();
        for (index, step) in self.steps.iter().enumerate() {
            if let Step::HandmadeRooms {
                paths,
                rooms: ids,
                most,
            } = step
                && !ids.is_empty()
            {
                for path in paths {
                    rooms.insert(path.clone(), (index, *most, ids.clone()));
                }
            }
        }
        rooms
    }
    /// Every Dungeon Room the recipe uses, once.
    pub fn rooms_used(&self) -> Vec<Uuid> {
        let mut ids: Vec<Uuid> = self
            .handmade()
            .into_values()
            .flat_map(|(_, _, ids)| ids)
            .collect();
        ids.sort_unstable();
        ids.dedup();
        ids
    }
    /// The Levels step's chance and highest level, if the recipe has one.
    pub fn levels(&self) -> Option<(u8, u8)> {
        self.steps.iter().rev().find_map(|s| match s {
            Step::Levels { chance, highest } => Some((*chance, *highest)),
            _ => None,
        })
    }
    /// Path names in recipe order: the main path, then each branch step's paths.
    pub fn path_names(&self) -> Vec<String> {
        let mut names = vec![MAIN.to_owned()];
        for step in &self.steps {
            if let Step::Branch { name, .. } = step
                && !names.contains(name)
            {
                names.push(name.clone());
            }
        }
        names
    }
    /// The cave settings for each path name a Caves step names; later steps win.
    pub fn caves(&self) -> BTreeMap<String, (u8, u8)> {
        let mut caves = BTreeMap::new();
        for step in &self.steps {
            if let Step::Caves {
                paths,
                density,
                smoothing,
            } = step
            {
                for path in paths {
                    caves.insert(path.clone(), (*density, *smoothing));
                }
            }
        }
        caves
    }
}

/// The recipe a generator follows, and its digest.
pub fn of(
    index: &assets::Index,
    g: &crate::dungeon::Generator,
) -> Result<(Recipe, String), String> {
    let recipe = match g.recipe {
        Some(id) => document(
            index
                .resolve(id)
                .map_err(|e| format!("The Dungeon Recipe is missing: {e}"))?,
        )?,
        None => builtin(),
    };
    // A dungeon follows its rooms as well as its recipe.
    let mut digest = digest(&recipe);
    let (rooms, _) = crate::dungeon_room::load(index, &recipe.rooms_used());
    for room in rooms.values() {
        digest.push_str(&crate::dungeon_room::digest(room));
    }
    Ok((recipe, digest))
}

pub fn digest(recipe: &Recipe) -> String {
    use sha2::{Digest, Sha256};
    let bytes = serde_json::to_vec(recipe).expect("recipe serializes");
    Sha256::digest(bytes)[..8]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

// ---- room graph ----------------------------------------------------------------------------

/// A link between two lattice nodes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Link {
    pub a: usize,
    pub b: usize,
    /// The key that opens it (1 and up), and the node it guards.
    pub lock: Option<(u16, usize)>,
    /// Passable only from `a` to `b`.
    pub one_way: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Placed {
    pub node: usize,
    pub name: String,
    /// For keys, the lock they open.
    pub lock: u16,
    pub place: Place,
}

/// The room graph a recipe grew.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Graph {
    pub size: [i32; 2],
    /// The path each node belongs to, if any.
    pub path: Vec<Option<usize>>,
    /// Each node's position along its path, from 0.
    pub depth: Vec<usize>,
    /// How far into the dungeon each node is along the intended route: its main path
    /// position, or for a branch its anchor's plus its own position along the branch.
    pub progress: Vec<usize>,
    pub paths: Vec<(String, Vec<usize>)>,
    pub links: Vec<Link>,
    pub items: Vec<Placed>,
    pub spawn: usize,
    pub goal: usize,
    pub keys: u16,
}

impl Graph {
    fn new(size: [i32; 2]) -> Self {
        let count = (size[0] * size[1]) as usize;
        Self {
            size,
            path: vec![None; count],
            depth: vec![0; count],
            progress: vec![0; count],
            paths: Vec::new(),
            links: Vec::new(),
            items: Vec::new(),
            spawn: 0,
            goal: 0,
            keys: 0,
        }
    }
    pub fn point(&self, node: usize) -> [i32; 2] {
        [node as i32 % self.size[0], node as i32 / self.size[0]]
    }
    fn neighbours(&self, node: usize) -> Vec<usize> {
        let p = self.point(node);
        [[1, 0], [0, 1], [-1, 0], [0, -1]]
            .iter()
            .filter_map(|d| {
                let q = [p[0] + d[0], p[1] + d[1]];
                (q[0] >= 0 && q[1] >= 0 && q[0] < self.size[0] && q[1] < self.size[1])
                    .then(|| (q[1] * self.size[0] + q[0]) as usize)
            })
            .collect()
    }
    pub fn active(&self, node: usize) -> bool {
        self.path[node].is_some()
    }
    fn nodes_of(&self, names: &[String]) -> Vec<usize> {
        self.paths
            .iter()
            .filter(|(name, _)| names.iter().any(|n| n == name))
            .flat_map(|(_, nodes)| nodes.iter().copied())
            .collect()
    }
    /// Records `nodes` as part of path `name`, `start` steps into the dungeon.
    fn add_path(&mut self, name: &str, nodes: &[usize], start: usize) {
        let index = match self.paths.iter().position(|(n, _)| n == name) {
            Some(index) => index,
            None => {
                self.paths.push((name.to_owned(), Vec::new()));
                self.paths.len() - 1
            }
        };
        for (depth, node) in nodes.iter().enumerate() {
            self.path[*node] = Some(index);
            self.depth[*node] = depth;
            self.progress[*node] = start + depth;
        }
        self.paths[index].1.extend_from_slice(nodes);
    }
    fn link(&mut self, a: usize, b: usize) {
        if !self
            .links
            .iter()
            .any(|l| (l.a == a && l.b == b) || (l.a == b && l.b == a))
        {
            self.links.push(Link {
                a,
                b,
                lock: None,
                one_way: false,
            });
        }
    }
    /// Walks from the spawn through open links, collecting keys and opening the locks
    /// they fit, until nothing new is reached. Returns the reachable nodes.
    pub fn reachable(&self, ignore_key: Option<u16>) -> Vec<bool> {
        let mut seen = vec![false; self.path.len()];
        let mut keys: BTreeSet<u16> = BTreeSet::new();
        loop {
            seen.iter_mut().for_each(|s| *s = false);
            seen[self.spawn] = true;
            let mut queue = VecDeque::from([self.spawn]);
            while let Some(node) = queue.pop_front() {
                for item in self.items.iter().filter(|i| i.node == node && i.lock > 0) {
                    if Some(item.lock) != ignore_key {
                        keys.insert(item.lock);
                    }
                }
                for link in &self.links {
                    let next = if link.a == node {
                        link.b
                    } else if link.b == node && !link.one_way {
                        link.a
                    } else {
                        continue;
                    };
                    if seen[next] {
                        continue;
                    }
                    if let Some((key, _)) = link.lock
                        && !keys.contains(&key)
                    {
                        continue;
                    }
                    seen[next] = true;
                    queue.push_back(next);
                }
            }
            let collectable = self.items.iter().any(|i| {
                i.lock > 0 && Some(i.lock) != ignore_key && seen[i.node] && !keys.contains(&i.lock)
            });
            if !collectable {
                // Keys picked up on this pass may open more; walk once more if so.
                let before = keys.len();
                for item in self.items.iter().filter(|i| i.lock > 0 && seen[i.node]) {
                    if Some(item.lock) != ignore_key {
                        keys.insert(item.lock);
                    }
                }
                if keys.len() == before {
                    return seen;
                }
            }
        }
    }
    /// Every active node is reachable from the spawn, collecting keys on the way.
    pub fn solvable(&self) -> bool {
        let seen = self.reachable(None);
        (0..self.path.len()).all(|n| !self.active(n) || seen[n])
    }
}

/// A self-avoiding walk over free nodes, `length` long, starting at `start` (which may
/// already be taken when `start_taken`), bounded in effort.
fn walk(
    graph: &Graph,
    start: usize,
    length: usize,
    random: &mut Random,
    end_near: Option<&[usize]>,
) -> Option<Vec<usize>> {
    let mut path = vec![start];
    let mut options: Vec<Vec<usize>> = vec![Vec::new()];
    let mut budget = 4000;
    let fresh = |graph: &Graph, path: &[usize], node: usize| -> Vec<usize> {
        let mut next: Vec<usize> = graph
            .neighbours(node)
            .into_iter()
            .filter(|n| !graph.active(*n) && !path.contains(n))
            .collect();
        random_order(&mut next, node as u32);
        next
    };
    options[0] = fresh(graph, &path, start);
    let mut order = random.next_u32();
    loop {
        budget -= 1;
        if budget == 0 {
            return None;
        }
        if path.len() == length {
            let last = *path.last().unwrap();
            let ends_well = match end_near {
                None => true,
                Some(targets) => graph
                    .neighbours(last)
                    .iter()
                    .any(|n| targets.contains(n) && Some(n) != path.first()),
            };
            if ends_well {
                return Some(path);
            }
            path.pop();
            options.pop();
            if path.is_empty() {
                return None;
            }
            continue;
        }
        let top = options.len() - 1;
        if let Some(next) = options[top].pop() {
            path.push(next);
            order = order.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let mut choices = fresh(graph, &path, next);
            random_order(&mut choices, order);
            options.push(choices);
        } else {
            path.pop();
            options.pop();
            if path.is_empty() {
                return None;
            }
        }
    }
}

/// A deterministic shuffle seeded by `salt`, used inside walks.
fn random_order(items: &mut [usize], salt: u32) {
    let mut random = Random::new(salt);
    for i in (1..items.len()).rev() {
        let j = random.below(i as u32 + 1) as usize;
        items.swap(i, j);
    }
}

fn shuffled<T: Copy>(items: &[T], random: &mut Random) -> Vec<T> {
    let mut out = items.to_vec();
    for i in (1..out.len()).rev() {
        let j = random.below(i as u32 + 1) as usize;
        out.swap(i, j);
    }
    out
}

/// Runs every step once with `random`; Err names the step that could not be satisfied.
fn grow(recipe: &Recipe, random: &mut Random) -> Result<Graph, String> {
    let (size, _) = recipe.lattice();
    let mut graph = Graph::new(size);
    let count = graph.path.len();
    for (index, step) in recipe.steps.iter().enumerate() {
        let fail = |message: &str| format!("Step {} ({}): {message}", index + 1, step.label());
        match step {
            Step::Lattice { .. }
            | Step::Rooms { .. }
            | Step::Caves { .. }
            | Step::HandmadeRooms { .. }
            | Step::Levels { .. } => {}
            Step::MainPath { length } => {
                let length = random.range(i32::from(length[0]), i32::from(length[1])) as usize;
                if length > count {
                    return Err(fail("the path is longer than the lattice has rooms"));
                }
                let start = random.below(count as u32) as usize;
                let path = walk(&graph, start, length, random, None)
                    .ok_or_else(|| fail("no path of that length fits"))?;
                graph.spawn = path[0];
                graph.goal = *path.last().unwrap();
                for pair in path.windows(2) {
                    graph.link(pair[0], pair[1]);
                }
                graph.add_path(MAIN, &path, 0);
            }
            Step::Branch {
                name,
                from,
                to,
                count: branches,
                length,
            } => {
                for _ in 0..*branches {
                    let length = random.range(i32::from(length[0]), i32::from(length[1])) as usize;
                    let anchors = shuffled(&graph.nodes_of(from), random);
                    let targets = graph.nodes_of(to);
                    let mut grown = None;
                    'anchors: for anchor in anchors {
                        for first in shuffled(&graph.neighbours(anchor), random) {
                            if graph.active(first) {
                                continue;
                            }
                            let end = (!to.is_empty()).then_some(targets.as_slice());
                            if let Some(path) = walk(&graph, first, length, random, end) {
                                grown = Some((anchor, path));
                                break 'anchors;
                            }
                        }
                    }
                    let (anchor, path) =
                        grown.ok_or_else(|| fail("no branch of that length fits"))?;
                    graph.link(anchor, path[0]);
                    for pair in path.windows(2) {
                        graph.link(pair[0], pair[1]);
                    }
                    if !to.is_empty() {
                        let last = *path.last().unwrap();
                        let close = graph
                            .neighbours(last)
                            .into_iter()
                            .filter(|n| targets.contains(n) && *n != anchor)
                            .min()
                            .or_else(|| {
                                graph
                                    .neighbours(last)
                                    .into_iter()
                                    .find(|n| targets.contains(n))
                            })
                            .ok_or_else(|| fail("the branch cannot close its loop"))?;
                        graph.link(last, close);
                    }
                    let start = graph.progress[anchor] + 1;
                    graph.add_path(name, &path, start);
                }
            }
            Step::Items {
                name,
                paths,
                count: amount,
                depth,
                place,
            } => {
                let amount = random.range(i32::from(amount[0]), i32::from(amount[1])) as usize;
                let mut candidates: Vec<usize> = Vec::new();
                for (path_name, nodes) in &graph.paths {
                    if !paths.iter().any(|p| p == path_name) {
                        continue;
                    }
                    let last = nodes.len().saturating_sub(1).max(1);
                    for (position, node) in nodes.iter().enumerate() {
                        let percent = position * 100 / last;
                        if percent >= usize::from(depth[0]) && percent <= usize::from(depth[1]) {
                            candidates.push(*node);
                        }
                    }
                }
                let candidates = shuffled(&candidates, random);
                if candidates.is_empty() {
                    return Err(fail("no room of those paths lies in that depth range"));
                }
                for i in 0..amount {
                    graph.items.push(Placed {
                        node: candidates[i % candidates.len()],
                        name: name.clone(),
                        lock: 0,
                        place: *place,
                    });
                }
            }
            Step::KeyLock {
                key_paths,
                lock_paths,
            } => {
                let key = graph.keys + 1;
                let keys: Vec<usize> = shuffled(&graph.nodes_of(key_paths), random)
                    .into_iter()
                    .filter(|n| *n != graph.spawn)
                    .collect();
                let locks: Vec<usize> = shuffled(&graph.nodes_of(lock_paths), random)
                    .into_iter()
                    .filter(|n| *n != graph.spawn)
                    .collect();
                let mut placed = false;
                'search: for &key_node in &keys {
                    for &lock_node in &locks {
                        if lock_node == key_node {
                            continue;
                        }
                        let candidates: Vec<usize> = (0..graph.links.len())
                            .filter(|&l| {
                                let link = graph.links[l];
                                link.lock.is_none() && (link.a == lock_node || link.b == lock_node)
                            })
                            .collect();
                        for l in shuffled(&candidates, random) {
                            let mut trial = graph.clone();
                            trial.links[l].lock = Some((key, lock_node));
                            trial.items.push(Placed {
                                node: key_node,
                                name: "Key".into(),
                                lock: key,
                                place: Place::Centre,
                            });
                            trial.keys = key;
                            // Solvable with every key, and the lock really keeps the
                            // guarded room out of reach without its own key.
                            let without = trial.reachable(Some(key));
                            if trial.solvable() && !without[lock_node] {
                                graph = trial;
                                placed = true;
                                break 'search;
                            }
                        }
                    }
                }
                if !placed {
                    return Err(fail("no key and lock placement is solvable"));
                }
            }
            Step::OneWay { chance } => {
                // Links that lead back at least two rooms along the route are
                // shortcuts; they open only from the far side.
                for l in 0..graph.links.len() {
                    let link = graph.links[l];
                    let roll = random.chance(u32::from(*chance));
                    let (pa, pb) = (graph.progress[link.a], graph.progress[link.b]);
                    if link.lock.is_some() || link.one_way || pa.abs_diff(pb) < 2 || !roll {
                        continue;
                    }
                    let (from, to) = if pa > pb {
                        (link.a, link.b)
                    } else {
                        (link.b, link.a)
                    };
                    let mut trial = graph.clone();
                    trial.links[l] = Link {
                        a: from,
                        b: to,
                        lock: None,
                        one_way: true,
                    };
                    if trial.solvable() {
                        graph = trial;
                    }
                }
            }
        }
    }
    if !graph.solvable() {
        return Err("The grown graph is not solvable".into());
    }
    Ok(graph)
}

/// Grows the room graph, retrying with fresh randomness until every step succeeds.
pub fn graph(recipe: &Recipe, seed: u32) -> Result<Graph, String> {
    graph_counted(recipe, seed).map(|(graph, _)| graph)
}

/// The room graph and how many attempts it took, from 1 to `ATTEMPTS`.
pub fn graph_counted(recipe: &Recipe, seed: u32) -> Result<(Graph, u32), String> {
    recipe.validate()?;
    let mut last = String::new();
    for attempt in 0..ATTEMPTS {
        let mut random = Random::stream(seed, STAGE_RECIPE + attempt);
        match grow(recipe, &mut random) {
            Ok(graph) => return Ok((graph, attempt + 1)),
            Err(error) => last = error,
        }
    }
    Err(format!(
        "{last} (after {ATTEMPTS} attempts; loosen the step's settings)"
    ))
}

// ---- layout ----------------------------------------------------------------------------------

/// Empty cells between a lattice cell's edge and its room. Corridors jog inside this
/// margin, one cell clear of everything else, so corridors of different links never
/// touch: a locked door cannot be walked around.
fn margin(width: i32) -> i32 {
    if width >= 3 { 3 } else { 2 }
}

/// The smallest lattice cell, in grid cells, that holds a room with its margins.
pub fn min_cell(width: u16) -> u16 {
    2 * margin(i32::from(width)) as u16 + 3
}

/// One cross-section of a corridor, in order from its first room to its second: the
/// cells across it, and whether a flight of stairs may climb through it along the
/// corridor's axis (straight stretches only, never at a door or next to a bend).
#[derive(Clone, Debug)]
struct Slice {
    cells: Vec<[i32; 2]>,
    stairs: bool,
}

/// Carves the corridor between `a` and the room after it along `axis` (0: east, 1:
/// south), jogging inside the gap that starts at `gap`. Returns the door cells on each
/// room's side and the corridor's cross-sections from `a` to `b`.
#[allow(clippy::too_many_arguments)]
fn corridor(
    cells: &mut [Cell],
    size: [i32; 2],
    a: &Room,
    b: &Room,
    axis: usize,
    gap: i32,
    width: i32,
    fixed: [Option<i32>; 2],
    random: &mut Random,
) -> ([Vec<GuardedDoor>; 2], Vec<Slice>) {
    let across = 1 - axis;
    let at = |along: i32, side: i32| -> [i32; 2] {
        let mut p = [0; 2];
        p[axis] = along;
        p[across] = side;
        p
    };
    let mut carve = |p: [i32; 2]| {
        let i = (p[1] * size[0] + p[0]) as usize;
        if cells[i] == Cell::Empty {
            cells[i] = Cell::Corridor;
        }
    };
    let range = |r: &Room| (r.min[across], r.min[across] + r.size[across] - width);
    let ((a0, a1), (b0, b1)) = (range(a), range(b));
    let (lo, hi) = (a0.max(b0), a1.min(b1));
    // A handmade room's door fixes where the corridor meets it; the other end lines up
    // with it when it can, for a straight corridor.
    let (ya, yb) = match fixed {
        [Some(ya), Some(yb)] => (ya, yb),
        [Some(ya), None] if (b0..=b1).contains(&ya) => (ya, ya),
        [Some(ya), None] => (ya, random.range(b0, b1)),
        [None, Some(yb)] if (a0..=a1).contains(&yb) => (yb, yb),
        [None, Some(yb)] => (random.range(a0, a1), yb),
        [None, None] if lo <= hi => {
            let y = random.range(lo, hi);
            (y, y)
        }
        [None, None] => (random.range(a0, a1), random.range(b0, b1)),
    };
    let a_end = a.min[axis] + a.size[axis];
    let b_start = b.min[axis];
    let jog = gap + random.range(1, 2 * margin(width) - width - 1);
    let row = |along: i32, side: i32, stairs: bool| Slice {
        cells: (0..width).map(|k| at(along, side + k)).collect(),
        stairs,
    };
    let mut slices = Vec::new();
    if ya == yb {
        for along in a_end..b_start {
            slices.push(row(along, ya, along > a_end && along < b_start - 1));
        }
    } else {
        for along in a_end..jog + width {
            slices.push(row(along, ya, along > a_end && along < jog - 1));
        }
        let sides: Vec<i32> = if ya < yb {
            (ya + width..yb).collect()
        } else {
            (yb + width..ya).rev().collect()
        };
        for side in sides {
            slices.push(Slice {
                cells: (0..width).map(|k| at(jog + k, side)).collect(),
                stairs: false,
            });
        }
        for along in jog..b_start {
            slices.push(row(along, yb, along > jog + width && along < b_start - 1));
        }
    }
    for k in 0..width {
        if ya == yb {
            for along in a_end..b_start {
                carve(at(along, ya + k));
            }
            continue;
        }
        for along in a_end..jog + width {
            carve(at(along, ya + k));
        }
        for side in ya.min(yb)..ya.max(yb) + width {
            carve(at(jog + k, side));
        }
        for along in jog..b_start {
            carve(at(along, yb + k));
        }
    }
    let (out, back) = if axis == 0 {
        (Side::East, Side::West)
    } else {
        (Side::South, Side::North)
    };
    let doors = |along: i32, y: i32, side: Side| {
        (0..width)
            .map(|k| GuardedDoor {
                cell: at(along, y + k),
                side,
                lock: 0,
            })
            .collect()
    };
    (
        [doors(a_end - 1, ya, out), doors(b_start, yb, back)],
        slices,
    )
}

/// A link's corridor as the level step sees it: its rooms in corridor order and the
/// direction its slices run in.
struct Passage {
    first: usize,
    second: usize,
    axis: usize,
    slices: Vec<Slice>,
}
impl Passage {
    fn capacity(&self) -> i32 {
        self.slices.iter().filter(|s| s.stairs).count() as i32
    }
}

/// Floor levels for every node and the cells of their rooms and corridors. Levels grow
/// from the spawn (level 0) over the graph's links: each link climbs or drops one level
/// with `chance` percent, staying within `0..=highest` and within the stairs its
/// corridor has room for. A link that cannot keep within its corridor flattens the
/// whole dungeon rather than leaving a step nobody can climb.
#[allow(clippy::type_complexity)]
fn levels(
    graph: &Graph,
    passages: &[Passage],
    room_of: &[usize],
    cells: &[Cell],
    size: [i32; 2],
    (chance, highest): (u8, u8),
    random: &mut Random,
) -> (BTreeMap<[i32; 2], u8>, BTreeMap<[i32; 2], Side>) {
    let count = graph.path.len();
    let mut level: Vec<Option<i32>> = vec![None; count];
    let mut adjacency: Vec<Vec<(usize, i32)>> = vec![Vec::new(); count];
    for p in passages {
        adjacency[p.first].push((p.second, p.capacity()));
        adjacency[p.second].push((p.first, p.capacity()));
    }
    level[graph.spawn] = Some(0);
    let mut queue = std::collections::VecDeque::from([graph.spawn]);
    while let Some(node) = queue.pop_front() {
        let here = level[node].unwrap_or(0);
        for &(next, _) in &adjacency[node] {
            if level[next].is_some() {
                continue;
            }
            // Every assigned neighbour bounds the level by its corridor's stairs.
            let (mut lo, mut hi) = (0, i32::from(highest));
            for &(other, capacity) in &adjacency[next] {
                if let Some(l) = level[other] {
                    lo = lo.max(l - capacity);
                    hi = hi.min(l + capacity);
                }
            }
            let mut wanted = here;
            if random.chance(u32::from(chance)) {
                let step = if random.chance(50) { 1 } else { -1 };
                wanted = if (lo..=hi).contains(&(here + step)) {
                    here + step
                } else {
                    here - step
                };
            }
            level[next] = Some(if lo > hi { here } else { wanted.clamp(lo, hi) });
            queue.push_back(next);
        }
    }
    let at = |p: [i32; 2]| cells[(p[1] * size[0] + p[0]) as usize];
    let fits = passages.iter().all(|p| {
        let difference = level[p.second].unwrap_or(0) - level[p.first].unwrap_or(0);
        difference.abs() <= p.capacity()
    });
    let mut levels = BTreeMap::new();
    let mut stairs = BTreeMap::new();
    if !fits || level.iter().all(|l| l.unwrap_or(0) == 0) {
        return (levels, stairs);
    }
    for (index, cell) in cells.iter().enumerate() {
        if let Cell::Room(r) = cell {
            let node = room_of.iter().position(|&room| room == usize::from(*r));
            let l = node.and_then(|n| level[n]).unwrap_or(0);
            if l > 0 {
                let p = [index as i32 % size[0], index as i32 / size[0]];
                levels.insert(p, l as u8);
            }
        }
    }
    for p in passages {
        let (from, to) = (level[p.first].unwrap_or(0), level[p.second].unwrap_or(0));
        let climb = to - from;
        let eligible: Vec<usize> = (0..p.slices.len())
            .filter(|i| p.slices[*i].stairs)
            .collect();
        // Flights spread evenly along the corridor's straight stretches.
        let n = climb.unsigned_abs() as usize;
        let flights: BTreeSet<usize> = (0..n)
            .map(|i| eligible[(2 * i + 1) * eligible.len() / (2 * n)])
            .collect();
        let toward = match (p.axis, climb > 0) {
            (0, true) => Side::East,
            (0, false) => Side::West,
            (_, true) => Side::South,
            (_, false) => Side::North,
        };
        let mut current = from;
        let mut seen = BTreeSet::new();
        for (index, slice) in p.slices.iter().enumerate() {
            let flight = flights.contains(&index);
            let low = if flight {
                current.min(current + climb.signum())
            } else {
                current
            };
            for &cell in &slice.cells {
                if at(cell) != Cell::Corridor || !seen.insert(cell) {
                    continue;
                }
                if low > 0 {
                    levels.insert(cell, low as u8);
                }
                if flight {
                    stairs.insert(cell, toward);
                }
            }
            if flight {
                current += climb.signum();
            }
        }
    }
    (levels, stairs)
}

/// Lays the graph out on the grid without handmade rooms' shapes: rooms that should be
/// handmade are generated instead.
#[cfg(test)]
pub fn layout(recipe: &Recipe, seed: u32) -> Result<Layout, String> {
    layout_with(recipe, seed, &BTreeMap::new())
}

/// Across coordinates (along the side, from the footprint's corner) where `width`
/// neighbouring doors open on `side`: where a corridor that wide can meet the room.
fn door_starts(shape: &crate::dungeon_room::Shape, side: Side, width: i32) -> Vec<i32> {
    let across = |c: [i32; 2]| match side {
        Side::East | Side::West => c[1],
        Side::North | Side::South => c[0],
    };
    let doors: BTreeSet<i32> = shape
        .doors
        .iter()
        .filter(|(_, s)| *s == side)
        .map(|(c, _)| across(*c))
        .collect();
    doors
        .iter()
        .copied()
        .filter(|start| (0..width).all(|k| doors.contains(&(start + k))))
        .collect()
}

/// Lays the graph out on the grid: one room per node inside its lattice cell, a
/// corridor per link, and the recipe's doors and items. Rooms of handmade paths take
/// the footprint and doors of one of `shapes` that fits.
pub fn layout_with(
    recipe: &Recipe,
    seed: u32,
    shapes: &BTreeMap<Uuid, crate::dungeon_room::Shape>,
) -> Result<Layout, String> {
    let (graph, attempts) = graph_counted(recipe, seed)?;
    let (lattice, span) = recipe.lattice();
    let (fill, width, rounded) = recipe.rooms();
    let margin = margin(width);
    let inner = span - 2 * margin;
    if inner < 3 {
        return Err(format!(
            "Lattice cells must span at least {} grid cells for corridors {width} wide",
            min_cell(width as u16)
        ));
    }
    let size = [lattice[0] * span + 2, lattice[1] * span + 2];
    let mut random = Random::stream(seed, STAGE_RECIPE + ATTEMPTS + 1);
    let nodes: Vec<usize> = (0..graph.path.len()).filter(|n| graph.active(*n)).collect();
    let mut room_of = vec![usize::MAX; graph.path.len()];
    let mut rooms = Vec::new();
    let lo = (inner * i32::from(fill[0]) / 100).clamp(3.max(width), inner);
    let hi = (inner * i32::from(fill[1]) / 100).clamp(lo, inner);
    let caves = recipe.caves();
    let is_cave =
        |node: usize| graph.path[node].is_some_and(|p| caves.contains_key(&graph.paths[p].0));
    // The side of a node towards each node it links to.
    let sides_of = |node: usize| -> BTreeSet<Side> {
        graph
            .links
            .iter()
            .filter_map(|l| match (l.a == node, l.b == node) {
                (true, _) => Some(l.b),
                (_, true) => Some(l.a),
                _ => None,
            })
            .map(|other| {
                let (p, q) = (graph.point(node), graph.point(other));
                match (q[0] - p[0], q[1] - p[1]) {
                    (d, _) if d > 0 => Side::East,
                    (d, _) if d < 0 => Side::West,
                    (_, d) if d > 0 => Side::South,
                    _ => Side::North,
                }
            })
            .collect()
    };
    let handmade_paths = recipe.handmade();
    let mut made: Vec<Option<(crate::dungeon::Handmade, crate::dungeon_room::Shape)>> = Vec::new();
    let mut used: BTreeMap<Uuid, usize> = BTreeMap::new();
    // Rooms each Handmade rooms step made so far, for its cap.
    let mut made_by: BTreeMap<usize, u8> = BTreeMap::new();
    let mut notes = Vec::new();
    for &node in &nodes {
        let p = graph.point(node);
        let path = graph.path[node].map(|i| graph.paths[i].0.clone());
        let wanted = path
            .as_ref()
            .and_then(|name| handmade_paths.get(name))
            .filter(|(step, most, _)| {
                *most == 0 || made_by.get(step).copied().unwrap_or(0) < *most
            });
        // A Dungeon Room for this room: one that fits the slot with doors towards every
        // link, the least used so far, turned any way that fits.
        let chosen = wanted.and_then(|(_, _, ids)| {
            let sides = sides_of(node);
            let mut options = Vec::new();
            for id in ids {
                let Some(shape) = shapes.get(id) else {
                    continue;
                };
                for turns in 0..4u8 {
                    let turned = shape.turned(turns);
                    if turned.size[0] <= inner
                        && turned.size[1] <= inner
                        && sides
                            .iter()
                            .all(|s| !door_starts(&turned, *s, width).is_empty())
                    {
                        options.push((*id, turns, turned));
                    }
                }
            }
            let least = options
                .iter()
                .map(|o| used.get(&o.0).copied().unwrap_or(0))
                .min()?;
            let pool: Vec<_> = options
                .into_iter()
                .filter(|o| used.get(&o.0).copied().unwrap_or(0) == least)
                .collect();
            Some(pool[random.below(pool.len() as u32) as usize].clone())
        });
        if wanted.is_some() && chosen.is_none() {
            notes.push(format!(
                "No Dungeon Room of the {} path fits a slot of {inner} cells with doors {}; a generated room stands there",
                path.as_deref().unwrap_or(MAIN),
                sides_of(node)
                    .iter()
                    .map(|s| format!("{s:?}").to_lowercase())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        let (w, h) = match &chosen {
            Some((_, _, shape)) => (shape.size[0], shape.size[1]),
            // Caves take their whole slot; their shape comes later.
            None if is_cave(node) => {
                let _ = (random.range(lo, hi), random.range(lo, hi));
                (inner, inner)
            }
            None => (random.range(lo, hi), random.range(lo, hi)),
        };
        if let Some((id, turns, shape)) = chosen {
            *used.entry(id).or_default() += 1;
            if let Some((step, _, _)) = wanted {
                *made_by.entry(*step).or_default() += 1;
            }
            made.push(Some((crate::dungeon::Handmade { room: id, turns }, shape)));
        } else {
            made.push(None);
        }
        let base = [1 + p[0] * span + margin, 1 + p[1] * span + margin];
        room_of[node] = rooms.len();
        rooms.push(Room {
            min: [
                base[0] + random.range(0, inner - w),
                base[1] + random.range(0, inner - h),
            ],
            size: [w, h],
        });
    }
    let mut cells = vec![Cell::Empty; (size[0] * size[1]) as usize];
    for (index, room) in rooms.iter().enumerate() {
        for y in room.min[1]..room.min[1] + room.size[1] {
            for x in room.min[0]..room.min[0] + room.size[0] {
                cells[(y * size[0] + x) as usize] = Cell::Room(index as u16);
            }
        }
    }
    // Each link's door cells, keyed by its two nodes, and its corridor.
    let mut doors: Vec<[Vec<GuardedDoor>; 2]> = Vec::new();
    let mut passages: Vec<Passage> = Vec::new();
    for link in &graph.links {
        let (pa, pb) = (graph.point(link.a), graph.point(link.b));
        let axis = usize::from(pa[0] == pb[0]);
        let (first, second) = if pa[axis] < pb[axis] {
            (link.a, link.b)
        } else {
            (link.b, link.a)
        };
        let gap = 1 + (graph.point(second)[axis]) * span - margin;
        // Handmade rooms are met at one of their doors on the facing side.
        let mut fixed = [None, None];
        for (k, (node, side)) in [
            (first, [Side::East, Side::South][axis]),
            (second, [Side::West, Side::North][axis]),
        ]
        .into_iter()
        .enumerate()
        {
            if let Some((_, shape)) = &made[room_of[node]] {
                let starts = door_starts(shape, side, width);
                if !starts.is_empty() {
                    let start = starts[random.below(starts.len() as u32) as usize];
                    fixed[k] = Some(rooms[room_of[node]].min[1 - axis] + start);
                }
            }
        }
        let ([d1, d2], slices) = corridor(
            &mut cells,
            size,
            &rooms[room_of[first]],
            &rooms[room_of[second]],
            axis,
            gap,
            width,
            fixed,
            &mut random,
        );
        doors.push(if first == link.a { [d1, d2] } else { [d2, d1] });
        passages.push(Passage {
            first,
            second,
            axis,
            slices,
        });
    }
    let mut locks = Vec::new();
    let mut one_ways = Vec::new();
    for (link, [at_a, at_b]) in graph.links.iter().zip(&doors) {
        if let Some((key, guarded)) = link.lock {
            let side = if guarded == link.a { at_a } else { at_b };
            locks.extend(side.iter().map(|d| GuardedDoor { lock: key, ..*d }));
        }
        if link.one_way {
            one_ways.extend(at_b.iter().copied());
        }
    }
    // Shape the rooms: caves and cut corners keep the centre, every door and a way
    // between them.
    let mut room_doors: Vec<Vec<GuardedDoor>> = vec![Vec::new(); rooms.len()];
    for (link, [at_a, at_b]) in graph.links.iter().zip(&doors) {
        room_doors[room_of[link.a]].extend(at_a.iter().copied());
        room_doors[room_of[link.b]].extend(at_b.iter().copied());
    }
    for &node in &nodes {
        let index = room_of[node];
        if let Some((_, shape)) = &made[index] {
            // A handmade room's footprint: cells it leaves out are rock.
            let room = rooms[index];
            for y in 0..room.size[1] {
                for x in 0..room.size[0] {
                    if !shape.is_floor([x, y]) {
                        cells[((room.min[1] + y) * size[0] + room.min[0] + x) as usize] =
                            Cell::Empty;
                    }
                }
            }
            continue;
        }
        let path = graph.path[node].map(|p| graph.paths[p].0.as_str());
        let shape = match path.and_then(|p| caves.get(p)) {
            Some(&(density, smoothing)) => Shape::Cave { density, smoothing },
            None if rounded => Shape::Rounded,
            None => Shape::Rectangle,
        };
        shape_room(
            &mut cells,
            size,
            &rooms[index],
            index as u16,
            &room_doors[index],
            shape,
            &mut random,
        );
    }
    // Items on free floor cells of their room, as their step places them; keys at
    // the centre.
    let mut taken: BTreeSet<[i32; 2]> = BTreeSet::new();
    let mut items = Vec::new();
    let at = |p: [i32; 2]| cells[(p[1] * size[0] + p[0]) as usize];
    for placed in &graph.items {
        let index = room_of[placed.node];
        let room = &rooms[index];
        let centre = room.center();
        let doors: BTreeSet<[i32; 2]> = room_doors[index].iter().map(|d| d.cell).collect();
        let floor: Vec<[i32; 2]> = (room.min[1]..room.min[1] + room.size[1])
            .flat_map(|y| (room.min[0]..room.min[0] + room.size[0]).map(move |x| [x, y]))
            .filter(|p| at(*p) == Cell::Room(index as u16) && !taken.contains(p))
            .collect();
        let near_door = |p: &[i32; 2]| {
            doors
                .iter()
                .any(|d| (d[0] - p[0]).abs() + (d[1] - p[1]).abs() <= 1)
        };
        let against_wall = |p: &[i32; 2]| {
            Side::ALL.iter().any(|s| {
                let d = s.delta();
                at([p[0] + d[0], p[1] + d[1]]) == Cell::Empty
            })
        };
        let pick = |pool: Vec<[i32; 2]>, random: &mut Random| {
            (!pool.is_empty()).then(|| pool[random.below(pool.len() as u32) as usize])
        };
        let open: Vec<[i32; 2]> = floor
            .iter()
            .copied()
            .filter(|p| *p != centre && !near_door(p))
            .collect();
        let chosen = match placed.place {
            Place::Centre => floor.iter().copied().min_by_key(|p| {
                (
                    (p[0] - centre[0]).abs() + (p[1] - centre[1]).abs(),
                    p[1],
                    p[0],
                )
            }),
            Place::Edge => pick(
                open.iter().copied().filter(|p| against_wall(p)).collect(),
                &mut random,
            )
            .or_else(|| pick(open.clone(), &mut random)),
            Place::Anywhere => {
                let inner: Vec<[i32; 2]> =
                    open.iter().copied().filter(|p| !against_wall(p)).collect();
                pick(
                    if inner.is_empty() {
                        open.clone()
                    } else {
                        inner
                    },
                    &mut random,
                )
            }
        };
        // A handmade room puts keys and items on its own spots first.
        let authored = made[index].as_ref().and_then(|(_, shape)| {
            shape
                .spots
                .iter()
                .map(|s| [room.min[0] + s[0], room.min[1] + s[1]])
                .find(|p| !taken.contains(p))
        });
        let cell = authored
            .or(chosen)
            .or_else(|| pick(floor.clone(), &mut random))
            .unwrap_or(centre);
        taken.insert(cell);
        items.push(Item {
            cell,
            name: placed.name.clone(),
            lock: placed.lock,
        });
    }
    let links: Vec<[usize; 2]> = graph
        .links
        .iter()
        .map(|l| {
            let (a, b) = (room_of[l.a], room_of[l.b]);
            [a.min(b), a.max(b)]
        })
        .collect();
    let loops = (links.len() + 1).saturating_sub(rooms.len());
    // Doors of handmade rooms no corridor came to are closed.
    let mut caps = Vec::new();
    for (index, entry) in made.iter().enumerate() {
        let Some((_, shape)) = entry else { continue };
        for (cell, side) in &shape.doors {
            let cell = [rooms[index].min[0] + cell[0], rooms[index].min[1] + cell[1]];
            let used = room_doors[index]
                .iter()
                .any(|d| d.cell == cell && d.side == *side);
            if !used {
                caps.push(GuardedDoor {
                    cell,
                    side: *side,
                    lock: 0,
                });
            }
        }
    }
    let (levels, stairs) = match recipe.levels() {
        Some(settings) => levels(
            &graph,
            &passages,
            &room_of,
            &cells,
            size,
            settings,
            &mut Random::stream(seed, STAGE_RECIPE + ATTEMPTS + 2),
        ),
        None => (BTreeMap::new(), BTreeMap::new()),
    };
    let room_paths = nodes
        .iter()
        .map(|n| {
            graph.path[*n]
                .map(|p| graph.paths[p].0.clone())
                .unwrap_or_default()
        })
        .collect();
    Ok(crop(Layout {
        size,
        cells,
        requested: rooms.len(),
        rooms,
        links,
        spawn: room_of[graph.spawn],
        goal: room_of[graph.goal],
        loops,
        attempts,
        locks,
        one_ways,
        items,
        room_paths,
        levels,
        stairs,
        handmade: made.into_iter().map(|m| m.map(|(h, _)| h)).collect(),
        caps,
        notes,
    }))
}

/// The outline of a room inside its rectangle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Shape {
    Rectangle,
    /// Corners cut by one cell, or by a three-cell triangle in larger rooms.
    Rounded,
    /// Rock scattered at `density` percent, then `smoothing` cellular automaton passes.
    Cave {
        density: u8,
        smoothing: u8,
    },
}

/// Turns the floor of room `index` into `shape`, always keeping its centre, its door
/// cells and a way from each door to the centre, then dropping floor the centre
/// cannot reach.
#[allow(clippy::too_many_arguments)]
fn shape_room(
    cells: &mut [Cell],
    size: [i32; 2],
    room: &Room,
    index: u16,
    doors: &[GuardedDoor],
    shape: Shape,
    random: &mut Random,
) {
    let [w, h] = room.size;
    let local = |x: i32, y: i32| (y * w + x) as usize;
    let mut floor = vec![true; (w * h) as usize];
    match shape {
        Shape::Rectangle => return,
        Shape::Rounded => {
            let cut = match w.min(h) {
                n if n >= 8 => 2,
                n if n >= 5 => 1,
                _ => 0,
            };
            for y in 0..h {
                for x in 0..w {
                    let (dx, dy) = (x.min(w - 1 - x), y.min(h - 1 - y));
                    floor[local(x, y)] = dx + dy >= cut;
                }
            }
        }
        Shape::Cave { density, smoothing } => {
            // Rock grows likelier towards the walls, so the middle stays open.
            for y in 0..h {
                for x in 0..w {
                    let edge = |v: i32, n: i32| {
                        let half = (n - 1).max(1) as f32 * 0.5;
                        ((v as f32 - half).abs() / half).min(1.)
                    };
                    let towards = edge(x, w).max(edge(y, h));
                    let chance = (f32::from(density) * 2. * towards).round() as u32;
                    floor[local(x, y)] = !random.chance(chance);
                }
            }
            for _ in 0..smoothing {
                let before = floor.clone();
                for y in 0..h {
                    for x in 0..w {
                        // Outside the room counts as rock, so caves round off their edges.
                        let rock = (-1..=1)
                            .flat_map(|dy| (-1..=1).map(move |dx| (dx, dy)))
                            .filter(|&(dx, dy)| (dx, dy) != (0, 0))
                            .filter(|&(dx, dy)| {
                                let (nx, ny) = (x + dx, y + dy);
                                nx < 0 || ny < 0 || nx >= w || ny >= h || !before[local(nx, ny)]
                            })
                            .count();
                        let was_rock = !before[local(x, y)];
                        floor[local(x, y)] = !(rock >= 6 || (was_rock && rock >= 4));
                    }
                }
            }
        }
    }
    let centre = [
        room.center()[0] - room.min[0],
        room.center()[1] - room.min[1],
    ];
    floor[local(centre[0], centre[1])] = true;
    for door in doors {
        // From the door straight into the room up to the centre's row or column, then
        // across to the centre.
        let mut p = [door.cell[0] - room.min[0], door.cell[1] - room.min[1]];
        let inward = door.side.opposite().delta();
        let axis = usize::from(inward[0] == 0);
        floor[local(p[0], p[1])] = true;
        while p[axis] != centre[axis] {
            p[axis] += inward[axis];
            floor[local(p[0], p[1])] = true;
        }
        let other = 1 - axis;
        while p[other] != centre[other] {
            p[other] += (centre[other] - p[other]).signum();
            floor[local(p[0], p[1])] = true;
        }
    }
    let mut reached = vec![false; floor.len()];
    reached[local(centre[0], centre[1])] = true;
    let mut stack = vec![centre];
    while let Some(p) = stack.pop() {
        for side in Side::ALL {
            let d = side.delta();
            let q = [p[0] + d[0], p[1] + d[1]];
            if q[0] >= 0 && q[1] >= 0 && q[0] < w && q[1] < h {
                let i = local(q[0], q[1]);
                if floor[i] && !reached[i] {
                    reached[i] = true;
                    stack.push(q);
                }
            }
        }
    }
    for y in 0..h {
        for x in 0..w {
            if !reached[local(x, y)] {
                let p = [room.min[0] + x, room.min[1] + y];
                let i = (p[1] * size[0] + p[0]) as usize;
                debug_assert_eq!(cells[i], Cell::Room(index));
                cells[i] = Cell::Empty;
            }
        }
    }
}

/// Trims the lattice cells no path reached, keeping a one-cell wall margin, so the
/// dungeon stays centred on its generator and as small as it can be.
fn crop(mut layout: Layout) -> Layout {
    let walkable: Vec<[i32; 2]> = (0..layout.size[1])
        .flat_map(|y| (0..layout.size[0]).map(move |x| [x, y]))
        .filter(|p| layout.at(*p).walkable())
        .collect();
    let lo = [0, 1].map(|k| walkable.iter().map(|p| p[k]).min().unwrap_or(1) - 1);
    let hi = [0, 1].map(|k| walkable.iter().map(|p| p[k]).max().unwrap_or(0) + 2);
    let size = [hi[0] - lo[0], hi[1] - lo[1]];
    let mut cells = vec![Cell::Empty; (size[0] * size[1]) as usize];
    for y in 0..size[1] {
        for x in 0..size[0] {
            cells[(y * size[0] + x) as usize] = layout.at([x + lo[0], y + lo[1]]);
        }
    }
    let shift = |p: &mut [i32; 2]| {
        p[0] -= lo[0];
        p[1] -= lo[1];
    };
    layout.rooms.iter_mut().for_each(|r| shift(&mut r.min));
    layout.locks.iter_mut().for_each(|d| shift(&mut d.cell));
    layout.one_ways.iter_mut().for_each(|d| shift(&mut d.cell));
    layout.items.iter_mut().for_each(|i| shift(&mut i.cell));
    layout.caps.iter_mut().for_each(|d| shift(&mut d.cell));
    layout.levels = std::mem::take(&mut layout.levels)
        .into_iter()
        .map(|(mut p, level)| {
            shift(&mut p);
            (p, level)
        })
        .collect();
    layout.stairs = std::mem::take(&mut layout.stairs)
        .into_iter()
        .map(|(mut p, side)| {
            shift(&mut p);
            (p, side)
        })
        .collect();
    layout.size = size;
    layout.cells = cells;
    layout
}

// ---- asset storage -----------------------------------------------------------------------------

pub const FOLDER: &str = "assets/Dungeons/Recipes";

pub fn fresh_path(root: &std::path::Path, name: &str) -> String {
    let stem: String = name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect();
    let stem = if stem.is_empty() {
        "DungeonRecipe".into()
    } else {
        stem
    };
    let mut path = format!("{FOLDER}/{stem}.epokasset");
    let mut n = 2;
    while root.join(&path).exists() {
        path = format!("{FOLDER}/{stem}{n}.epokasset");
        n += 1;
    }
    path
}
pub fn create(root: &std::path::Path, path: &str, recipe: &Recipe) -> Result<Uuid, String> {
    let id = Uuid::new_v4();
    let source = recipe.encode()?;
    let package = assets::Package {
        meta: assets::Metadata {
            version: 1,
            id,
            kind: assets::Kind::DungeonRecipe,
            importer_version: 1,
            source: path.into(),
            source_hash: assets::hash(&source),
            settings: crate::import_settings::Settings::Authored,
            extra: Default::default(),
        },
        source,
    };
    assets::atomic_write(&assets::inside(root, path)?, &package.bytes()?, None)?;
    Ok(id)
}
pub fn document(record: &assets::Record) -> Result<Recipe, String> {
    if record.meta.kind != assets::Kind::DungeonRecipe {
        return Err("Select a Dungeon Recipe asset".into());
    }
    Recipe::parse(&assets::Package::load(&record.path)?.source)
}
pub fn save(record: &assets::Record, recipe: &Recipe) -> Result<String, String> {
    let mut package = assets::Package::load(&record.path)?;
    if package.meta.kind != assets::Kind::DungeonRecipe || package.meta.id != record.meta.id {
        return Err("The Dungeon Recipe was replaced; reopen it before editing".into());
    }
    package.source = recipe.encode()?;
    package.meta.source_hash = assets::hash(&package.source);
    let bytes = package.bytes()?;
    assets::atomic_write(&record.path, &bytes, Some(&record.revision))?;
    Ok(assets::hash(&bytes))
}

#[cfg(test)]
mod tests {
    /// Handmade rooms take a Dungeon Room's turned footprint, are met by corridors at
    /// their doors, wall up the doors nobody uses, get no generated floor or walls, and
    /// leave every room reachable.
    #[test]
    fn handmade_rooms_take_their_footprint_and_doors() {
        use crate::dungeon_room::{Door, Room};
        let cross = Room {
            size: [5, 5],
            doors: vec![
                Door {
                    cell: [4, 2],
                    side: Side::East,
                },
                Door {
                    cell: [0, 2],
                    side: Side::West,
                },
                Door {
                    cell: [2, 0],
                    side: Side::North,
                },
                Door {
                    cell: [2, 4],
                    side: Side::South,
                },
            ],
            spots: vec![[1, 1]],
            ..Default::default()
        };
        let corner = Room {
            size: [6, 4],
            closed: vec![[4, 2], [5, 2], [4, 3], [5, 3]],
            doors: vec![
                Door {
                    cell: [5, 1],
                    side: Side::East,
                },
                Door {
                    cell: [1, 3],
                    side: Side::South,
                },
            ],
            ..Default::default()
        };
        let ids = [Uuid::from_u128(1), Uuid::from_u128(2)];
        let shapes: BTreeMap<Uuid, crate::dungeon_room::Shape> = ids
            .iter()
            .zip([&cross, &corner])
            .map(|(id, r)| (*id, r.shape()))
            .collect();
        let mut recipe = builtin();
        recipe.steps.insert(
            recipe.steps.len() - 1,
            Step::HandmadeRooms {
                paths: vec![MAIN.into()],
                rooms: ids.to_vec(),
                most: 0,
            },
        );
        recipe.validate().unwrap();
        let mut handmade_total = 0;
        for seed in 1..=12 {
            let layout = layout_with(&recipe, seed, &shapes).unwrap();
            let spots = crate::dungeon::spots(&layout);
            for (index, room) in layout.rooms.iter().enumerate() {
                let Some(made) = layout.handmade_room(index) else {
                    continue;
                };
                handmade_total += 1;
                assert_eq!(layout.room_paths[index], MAIN, "seed {seed}");
                let shape = shapes[&made.room].turned(made.turns);
                assert_eq!(room.size, shape.size, "seed {seed}: room {index}");
                for y in 0..room.size[1] {
                    for x in 0..room.size[0] {
                        let p = [room.min[0] + x, room.min[1] + y];
                        assert_eq!(
                            layout.at(p) == Cell::Room(index as u16),
                            shape.is_floor([x, y]),
                            "seed {seed}: room {index} cell {p:?}"
                        );
                    }
                }
                // Every opening of the room is one of its doors, used or walled up.
                let doors: BTreeSet<([i32; 2], Side)> = shape
                    .doors
                    .iter()
                    .map(|(c, s)| ([room.min[0] + c[0], room.min[1] + c[1]], *s))
                    .collect();
                for y in 0..room.size[1] {
                    for x in 0..room.size[0] {
                        let p = [room.min[0] + x, room.min[1] + y];
                        for side in Side::ALL {
                            if layout.door(p, side) {
                                assert!(doors.contains(&(p, side)), "seed {seed}: {p:?} {side:?}");
                            }
                        }
                    }
                }
                for cap in &layout.caps {
                    assert!(!layout.door(cap.cell, cap.side));
                }
                let capped = layout
                    .caps
                    .iter()
                    .filter(|c| doors.contains(&(c.cell, c.side)))
                    .count();
                let open = doors.iter().filter(|(c, s)| layout.door(*c, *s)).count();
                assert_eq!(capped + open, doors.len(), "seed {seed}");
                // The generator draws nothing inside: no floor, ceiling, wall, corner or
                // column, except the walls that close unused doors.
                for spot in spots.iter().filter(|s| layout.handmade_cell(s.cell)) {
                    match spot.kind {
                        crate::dungeon::SpotKind::Wall => assert!(
                            layout
                                .caps
                                .iter()
                                .any(|c| c.cell == spot.cell && Some(c.side) == spot.side)
                        ),
                        kind => assert!(
                            !matches!(
                                kind,
                                crate::dungeon::SpotKind::Floor
                                    | crate::dungeon::SpotKind::Ceiling
                                    | crate::dungeon::SpotKind::Corner
                                    | crate::dungeon::SpotKind::Column
                            ),
                            "seed {seed}: {kind:?} in a handmade room"
                        ),
                    }
                }
            }
            // Every walkable cell is reached from the spawn.
            let start = layout.room_floor_center(layout.spawn);
            let mut seen = BTreeSet::from([start]);
            let mut queue = VecDeque::from([start]);
            while let Some(p) = queue.pop_front() {
                for side in Side::ALL {
                    let d = side.delta();
                    let q = [p[0] + d[0], p[1] + d[1]];
                    if layout.open(p, side) && seen.insert(q) {
                        queue.push_back(q);
                    }
                }
            }
            assert_eq!(seen.len(), layout.walkable_cells(), "seed {seed}");
        }
        assert!(
            handmade_total >= 24,
            "only {handmade_total} handmade rooms in 12 dungeons"
        );
    }

    /// A Handmade rooms step with a cap makes no more rooms than it, and still makes them.
    #[test]
    fn handmade_rooms_respect_their_cap() {
        use crate::dungeon_room::{Door, Room};
        let cross = Room {
            size: [5, 5],
            doors: Side::ALL
                .into_iter()
                .map(|side| Door {
                    cell: match side {
                        Side::East => [4, 2],
                        Side::West => [0, 2],
                        Side::North => [2, 0],
                        Side::South => [2, 4],
                    },
                    side,
                })
                .collect(),
            ..Default::default()
        };
        let id = Uuid::from_u128(1);
        let shapes = BTreeMap::from([(id, cross.shape())]);
        let mut recipe = builtin();
        recipe.steps.insert(
            recipe.steps.len() - 1,
            Step::HandmadeRooms {
                paths: vec![MAIN.into()],
                rooms: vec![id],
                most: 1,
            },
        );
        recipe.validate().unwrap();
        let mut total = 0;
        for seed in 1..=12 {
            let layout = layout_with(&recipe, seed, &shapes).unwrap();
            let made = (0..layout.rooms.len())
                .filter(|&i| layout.handmade_room(i).is_some())
                .count();
            assert!(made <= 1, "seed {seed}: {made} handmade rooms");
            total += made;
        }
        assert!(total >= 10, "only {total} handmade rooms in 12 dungeons");
        if let Some(Step::HandmadeRooms { most, .. }) = recipe
            .steps
            .iter_mut()
            .find(|s| matches!(s, Step::HandmadeRooms { .. }))
        {
            *most = 17;
        }
        assert!(recipe.validate().is_err());
    }

    /// Floors meet across every open edge: a flat cell at its floor, a stair cell at its
    /// low end on the side it climbs from and its top towards the side it climbs to.
    /// Rooms stay flat, stairs stand in corridors, and the built-in recipe climbs.
    #[test]
    fn levels_meet_across_every_open_edge() {
        let recipe = builtin();
        let mut climbed = 0;
        for seed in 1..=24 {
            let layout = layout(&recipe, seed).unwrap();
            let edge = |p: [i32; 2], side: Side| match layout.stair(p) {
                Some(up) if up == side => layout.top(p),
                Some(up) if up == side.opposite() => layout.floor(p),
                _ => layout.floor(p),
            };
            for y in 0..layout.size[1] {
                for x in 0..layout.size[0] {
                    let p = [x, y];
                    if !layout.at(p).walkable() {
                        assert_eq!(layout.level(p), 0);
                        assert!(layout.stair(p).is_none());
                        continue;
                    }
                    if let Some(up) = layout.stair(p) {
                        assert_eq!(
                            layout.at(p),
                            Cell::Corridor,
                            "seed {seed}: stairs in a room"
                        );
                        // A flight runs between open cells along its axis.
                        assert!(layout.open(p, up) && layout.open(p, up.opposite()));
                    }
                    for side in [Side::East, Side::South] {
                        if !layout.open(p, side) {
                            continue;
                        }
                        let d = side.delta();
                        let q = [x + d[0], y + d[1]];
                        // Across a flight's width its lanes climb together.
                        let across = layout
                            .stair(p)
                            .is_some_and(|up| up != side && up != side.opposite());
                        if across && layout.stair(p) == layout.stair(q) {
                            assert_eq!(layout.level(p), layout.level(q));
                            continue;
                        }
                        assert!(
                            (edge(p, side) - edge(q, side.opposite())).abs() < 1e-4,
                            "seed {seed}: {p:?} and {q:?} do not meet"
                        );
                    }
                }
            }
            for (index, room) in layout.rooms.iter().enumerate() {
                let levels: BTreeSet<u8> = (room.min[1]..room.min[1] + room.size[1])
                    .flat_map(|y| (room.min[0]..room.min[0] + room.size[0]).map(move |x| [x, y]))
                    .filter(|p| layout.at(*p) == Cell::Room(index as u16))
                    .map(|p| layout.level(p))
                    .collect();
                assert!(levels.len() <= 1, "seed {seed}: room {index} is not flat");
            }
            assert_eq!(layout.level(layout.rooms[layout.spawn].center()), 0);
            climbed += usize::from(!layout.stairs.is_empty());
            assert!(layout.highest_level() <= 2);
        }
        assert!(climbed >= 12, "only {climbed} of 24 dungeons climb");
    }

    use super::*;

    #[test]
    fn the_builtin_recipe_is_valid_and_round_trips() {
        let recipe = builtin();
        assert_eq!(Recipe::parse(&recipe.encode().unwrap()).unwrap(), recipe);
    }

    #[test]
    fn graphs_are_reproducible_connected_and_always_solvable() {
        let recipe = builtin();
        for seed in 0..80 {
            let graph = graph(&recipe, seed).unwrap_or_else(|e| panic!("seed {seed}: {e}"));
            assert_eq!(graph, super::graph(&recipe, seed).unwrap());
            assert!(graph.solvable(), "seed {seed}");
            // One key and one lock, and the lock guards something.
            let locks: Vec<_> = graph.links.iter().filter_map(|l| l.lock).collect();
            assert_eq!(locks.len(), 1, "seed {seed}");
            let (key, guarded) = locks[0];
            assert!(!graph.reachable(Some(key))[guarded], "seed {seed}");
            assert!(graph.items.iter().any(|i| i.lock == key));
            // The main path runs from spawn to goal through links.
            let main = &graph.paths[0].1;
            assert_eq!(main[0], graph.spawn);
            assert_eq!(*main.last().unwrap(), graph.goal);
            // Treasure lies on branches.
            let branch = graph.paths.iter().find(|(n, _)| n == "branch").unwrap();
            for item in graph.items.iter().filter(|i| i.name == "Treasure") {
                assert!(branch.1.contains(&item.node));
            }
        }
    }

    #[test]
    fn several_keys_never_deadlock() {
        let mut recipe = builtin();
        recipe.steps.insert(
            5,
            Step::KeyLock {
                key_paths: vec![MAIN.into(), "branch".into()],
                lock_paths: vec![MAIN.into(), "branch".into()],
            },
        );
        recipe.steps.insert(
            5,
            Step::KeyLock {
                key_paths: vec!["branch".into()],
                lock_paths: vec!["branch".into(), MAIN.into()],
            },
        );
        for seed in 0..60 {
            let graph = graph(&recipe, seed).unwrap_or_else(|e| panic!("seed {seed}: {e}"));
            assert_eq!(graph.links.iter().filter(|l| l.lock.is_some()).count(), 3);
            assert!(graph.solvable(), "seed {seed}");
        }
    }

    #[test]
    fn loops_close_onto_their_target_path() {
        let mut recipe = builtin();
        recipe.steps.retain(|s| {
            matches!(
                s,
                Step::Lattice { .. } | Step::MainPath { .. } | Step::Rooms { .. }
            ) || matches!(s, Step::Branch { name, .. } if name == "loop")
        });
        for seed in 0..20 {
            let graph = graph(&recipe, seed).unwrap();
            // A loop adds one more link than nodes it adds.
            let looped = graph
                .paths
                .iter()
                .find(|(n, _)| n == "loop")
                .unwrap()
                .1
                .len();
            let main = graph.paths[0].1.len();
            assert_eq!(graph.links.len(), (main - 1) + looped + 1, "seed {seed}");
        }
    }

    #[test]
    fn layouts_carry_locks_one_ways_and_items_to_spots() {
        let recipe = builtin();
        let mut found = (0, 0);
        for seed in 0..40 {
            let layout = layout(&recipe, seed).unwrap_or_else(|e| panic!("seed {seed}: {e}"));
            assert_eq!(layout.locks.len(), 1);
            let lock = layout.locks[0];
            assert!(matches!(layout.at(lock.cell), Cell::Room(_)));
            let d = lock.side.delta();
            assert_eq!(
                layout.at([lock.cell[0] + d[0], lock.cell[1] + d[1]]),
                Cell::Corridor
            );
            let spots = crate::dungeon::spots(&layout);
            assert!(
                spots
                    .iter()
                    .any(|s| s.kind == crate::dungeon::SpotKind::Key && s.tag == lock.lock)
            );
            assert!(
                spots
                    .iter()
                    .any(|s| s.kind == crate::dungeon::SpotKind::Lock && s.tag == lock.lock)
            );
            found.0 += layout.one_ways.len();
            found.1 += layout.items.iter().filter(|i| i.name == "Treasure").count();
            assert_eq!(layout.room_paths.len(), layout.rooms.len());
            // Everything walkable is reachable when doors are ignored.
            let start = layout.rooms[layout.spawn].center();
            let mut seen = std::collections::BTreeSet::from([start]);
            let mut stack = vec![start];
            while let Some(p) = stack.pop() {
                for side in Side::ALL {
                    let d = side.delta();
                    let q = [p[0] + d[0], p[1] + d[1]];
                    if layout.open(p, side) && seen.insert(q) {
                        stack.push(q);
                    }
                }
            }
            assert_eq!(seen.len(), layout.walkable_cells(), "seed {seed}");
        }
        assert!(found.0 > 0 && found.1 > 0, "{found:?}");
    }

    #[test]
    fn a_locked_door_cannot_be_walked_around_on_the_grid() {
        let recipe = builtin();
        for seed in 0..40 {
            let layout = layout(&recipe, seed).unwrap();
            for lock in &layout.locks {
                let Cell::Room(guarded) = layout.at(lock.cell) else {
                    panic!("locks sit on room cells");
                };
                let blocked = |p: [i32; 2], side: Side| {
                    let d = side.delta();
                    let q = [p[0] + d[0], p[1] + d[1]];
                    layout.locks.iter().any(|l| {
                        l.lock == lock.lock
                            && ((l.cell == p && l.side == side)
                                || (l.cell == q && l.side == side.opposite()))
                    })
                };
                let start = layout.rooms[layout.spawn].center();
                let mut seen = std::collections::BTreeSet::from([start]);
                let mut stack = vec![start];
                while let Some(p) = stack.pop() {
                    for side in Side::ALL {
                        let d = side.delta();
                        let q = [p[0] + d[0], p[1] + d[1]];
                        if layout.open(p, side) && !blocked(p, side) && seen.insert(q) {
                            stack.push(q);
                        }
                    }
                }
                let inside = layout.rooms[usize::from(guarded)].center();
                assert!(
                    !seen.contains(&inside),
                    "seed {seed}: lock {} bypassed",
                    lock.lock
                );
            }
        }
    }

    #[test]
    fn caves_and_cut_corners_shape_rooms_around_their_doors() {
        let recipe = builtin();
        let (mut caves, mut cut) = (0, 0);
        for seed in 0..30 {
            let layout = layout(&recipe, seed).unwrap();
            for (index, room) in layout.rooms.iter().enumerate() {
                let own = Cell::Room(index as u16);
                assert_eq!(layout.at(room.center()), own, "seed {seed}");
                let area = (room.size[0] * room.size[1]) as usize;
                let floor = (room.min[1]..room.min[1] + room.size[1])
                    .flat_map(|y| (room.min[0]..room.min[0] + room.size[0]).map(move |x| [x, y]))
                    .filter(|p| layout.at(*p) == own)
                    .count();
                let corners = [
                    room.min,
                    [room.min[0] + room.size[0] - 1, room.min[1]],
                    [room.min[0], room.min[1] + room.size[1] - 1],
                    [
                        room.min[0] + room.size[0] - 1,
                        room.min[1] + room.size[1] - 1,
                    ],
                ];
                if layout.room_paths[index] == "branch" {
                    // Caves: organic, and never more than their slot.
                    assert!(floor < area, "seed {seed}: cave {index} is a rectangle");
                    caves += 1;
                } else if room.size[0].min(room.size[1]) >= 5 {
                    // Cut corners, unless a door needs the cell.
                    let doors: Vec<[i32; 2]> = crate::dungeon::spots(&layout)
                        .iter()
                        .filter(|s| s.kind == crate::dungeon::SpotKind::Door)
                        .map(|s| s.cell)
                        .collect();
                    for corner in corners {
                        assert!(
                            layout.at(corner) != own || doors.contains(&corner),
                            "seed {seed}: room {index} keeps corner {corner:?}"
                        );
                    }
                    cut += 1;
                }
            }
        }
        assert!(caves > 30 && cut > 30, "{caves} caves, {cut} cut rooms");
    }

    #[test]
    fn items_go_where_their_step_places_them() {
        let mut recipe = builtin();
        recipe.steps.retain(|s| !matches!(s, Step::Caves { .. }));
        for (place, name) in [(Place::Centre, "Altar"), (Place::Edge, "Torch")] {
            recipe.steps.insert(
                4,
                Step::Items {
                    name: name.into(),
                    paths: vec![MAIN.into()],
                    count: [3, 3],
                    depth: [0, 100],
                    place,
                },
            );
        }
        for seed in 0..20 {
            let layout = layout(&recipe, seed).unwrap();
            let room_of = |p: [i32; 2]| match layout.at(p) {
                Cell::Room(r) => usize::from(r),
                other => panic!("seed {seed}: item on {other:?}"),
            };
            let mut cells = std::collections::BTreeSet::new();
            for item in &layout.items {
                assert!(
                    cells.insert(item.cell),
                    "seed {seed}: two items share a cell"
                );
                let room = &layout.rooms[room_of(item.cell)];
                match item.name.as_str() {
                    "Torch" => assert!(
                        Side::ALL.iter().any(|s| {
                            let d = s.delta();
                            layout.at([item.cell[0] + d[0], item.cell[1] + d[1]]) == Cell::Empty
                        }),
                        "seed {seed}: torch away from the walls"
                    ),
                    "Altar" | "Key" => {
                        let c = room.center();
                        let distance = (item.cell[0] - c[0]).abs() + (item.cell[1] - c[1]).abs();
                        assert!(
                            distance <= 2,
                            "seed {seed}: {} {distance} from the centre",
                            item.name
                        );
                    }
                    _ => {}
                }
            }
        }
    }

    #[test]
    #[ignore]
    fn print_a_recipe_layout() {
        let seed = std::env::var("EPOK_RECIPE_SEED")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(7);
        let layout = layout(&builtin(), seed).unwrap();
        for (i, room) in layout.rooms.iter().enumerate() {
            let world = |c: i32, k: usize| (c as f32 - layout.size[k] as f32 * 0.5) * 2.;
            println!(
                "room {i} ({}): x {}..{} z {}..{}",
                layout.room_paths[i],
                world(room.min[0], 0),
                world(room.min[0] + room.size[0], 0),
                world(room.min[1], 1),
                world(room.min[1] + room.size[1], 1)
            );
        }
        println!("{}", crate::dungeon::picture(&layout));
        println!("{}", crate::dungeon::summary(&layout));
        for lock in &layout.locks {
            println!(
                "lock {} at {:?} facing {:?}",
                lock.lock, lock.cell, lock.side
            );
        }
    }

    #[test]
    fn invalid_recipes_name_the_step() {
        let mut recipe = builtin();
        recipe.steps.insert(
            2,
            Step::Items {
                name: "Gold".into(),
                paths: vec!["nowhere".into()],
                count: [1, 1],
                depth: [0, 100],
                place: Place::Anywhere,
            },
        );
        let error = recipe.validate().unwrap_err();
        assert!(
            error.contains("Step 3 (Items)") && error.contains("nowhere"),
            "{error}"
        );
        let mut impossible = builtin();
        impossible.steps[1] = Step::MainPath { length: [40, 40] };
        let error = graph(&impossible, 1).unwrap_err();
        assert!(error.contains("Main path"), "{error}");
    }
}
