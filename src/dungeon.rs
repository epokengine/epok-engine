//! Dungeon Generator: seeded layouts baked into ordinary map content.
//!
//! A generator lives in the map document next to the actor that places it. Its layout
//! is a grid of cells grouped into rooms and corridors; `spots` names every place where
//! content belongs (floors, walls, doors, spawn, goal). `dungeon_bake` turns spots into
//! EditableMesh sections, a compound collider and marker actors in 3D, or TileMapLayer
//! cells in 2D. Decisions use integers and `dungeon_random`, so a seed always produces
//! the same dungeon.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

/// Settings of the rooms-and-corridors layout, in grid cells.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct RoomSettings {
    /// Grid width and depth.
    pub size: [u16; 2],
    /// How many rooms to place; fewer fit when the grid is crowded.
    pub rooms: u16,
    pub room_min: u16,
    pub room_max: u16,
    /// Empty cells kept between rooms.
    pub spacing: u16,
    pub corridor_width: u16,
    /// Percentage of optional extra connections that become loops.
    pub loops: u16,
}
impl Default for RoomSettings {
    fn default() -> Self {
        Self {
            size: [40, 40],
            rooms: 10,
            room_min: 4,
            room_max: 8,
            spacing: 3,
            corridor_width: 1,
            loops: 25,
        }
    }
}
pub const GRID_LIMITS: (u16, u16) = (12, 128);
pub const ROOM_LIMITS: (u16, u16) = (2, 64);
pub const ROOM_SIZE_LIMITS: (u16, u16) = (3, 24);
pub const SPACING_LIMITS: (u16, u16) = (1, 8);
pub const CORRIDOR_LIMITS: (u16, u16) = (1, 3);

impl RoomSettings {
    pub fn validate(&self) -> Result<(), String> {
        let (lo, hi) = GRID_LIMITS;
        if self.size.iter().any(|s| !(lo..=hi).contains(s)) {
            return Err(format!("Grid size must be {lo}-{hi} cells per side"));
        }
        if !(ROOM_LIMITS.0..=ROOM_LIMITS.1).contains(&self.rooms) {
            return Err(format!(
                "Room count must be {}-{}",
                ROOM_LIMITS.0, ROOM_LIMITS.1
            ));
        }
        let (min, max) = ROOM_SIZE_LIMITS;
        if !(min..=max).contains(&self.room_min)
            || !(min..=max).contains(&self.room_max)
            || self.room_min > self.room_max
        {
            return Err(format!(
                "Room sizes must be {min}-{max} cells, smallest first"
            ));
        }
        if usize::from(self.room_max) + 2 > usize::from(*self.size.iter().min().unwrap()) {
            return Err("The largest room must fit inside the grid".into());
        }
        if !(SPACING_LIMITS.0..=SPACING_LIMITS.1).contains(&self.spacing) {
            return Err(format!(
                "Room spacing must be {}-{} cells",
                SPACING_LIMITS.0, SPACING_LIMITS.1
            ));
        }
        if !(CORRIDOR_LIMITS.0..=CORRIDOR_LIMITS.1).contains(&self.corridor_width) {
            return Err(format!(
                "Corridor width must be {}-{} cells",
                CORRIDOR_LIMITS.0, CORRIDOR_LIMITS.1
            ));
        }
        if self.loops > 100 {
            return Err("Loops is a percentage, 0-100".into());
        }
        Ok(())
    }
}

/// What a generation wrote into the map, so regenerating replaces exactly that.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Output {
    pub actors: Vec<Uuid>,
    pub assets: Vec<Uuid>,
    /// `fingerprint` of the settings that produced this output.
    pub fingerprint: String,
    /// Digest of the Dungeon Style it was generated with.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub style: String,
    /// Digest of the Dungeon Recipe it was generated with; empty for rooms layouts.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub recipe: String,
    /// What gameplay can ask about the generated dungeon.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queries: Option<crate::dungeon_query::Map>,
}
impl Output {
    pub fn is_empty(&self) -> bool {
        self.actors.is_empty() && self.assets.is_empty()
    }
}

/// Where a generator's rooms come from.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum Source {
    /// Rooms scattered over the grid and joined by corridors.
    #[default]
    Rooms,
    /// A Dungeon Recipe: a main path, branches, loops, items, keys and locks.
    Recipe,
}
impl Source {
    pub const ALL: [Source; 2] = [Source::Rooms, Source::Recipe];
    pub fn label(self) -> &'static str {
        match self {
            Source::Rooms => "Rooms and corridors",
            Source::Recipe => "Recipe",
        }
    }
}

/// One generator in a map. The root actor places it; its 2D or 3D domain decides the
/// kind of output.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Generator {
    pub id: Uuid,
    pub root: Uuid,
    pub seed: u32,
    pub source: Source,
    pub rooms: RoomSettings,
    /// The Dungeon Recipe a `Source::Recipe` generator follows; None uses the built-in one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipe: Option<Uuid>,
    /// 3D world units per grid cell.
    pub cell_size: f32,
    pub wall_height: f32,
    pub door_height: f32,
    /// 3D: close the dungeon from above at the wall height (off for top-down cameras).
    pub ceiling: bool,
    /// Add a navigation bake volume sized to stay inside the node budget.
    pub navigation: bool,
    /// An actor moved to the spawn point after generating (usually the player).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spawn_actor: Option<Uuid>,
    /// 2D: the TileSet a style's terrains and tiles use when they name none; None uses
    /// the built-in dungeon tiles.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tile_set: Option<Uuid>,
    /// The Dungeon Style deciding what each spot becomes; None uses the built-in one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<Uuid>,
    pub output: Output,
}
impl Default for Generator {
    fn default() -> Self {
        Self {
            id: Uuid::nil(),
            root: Uuid::nil(),
            seed: 1,
            source: Source::Rooms,
            rooms: RoomSettings::default(),
            recipe: None,
            cell_size: 3.,
            wall_height: 6.,
            door_height: 3.6,
            ceiling: true,
            navigation: true,
            spawn_actor: None,
            tile_set: None,
            style: None,
            output: Output::default(),
        }
    }
}
pub const CELL_SIZE_LIMITS: (f32, f32) = (0.5, 8.);
/// Proportions of a character `height` units tall that keep a dungeon roomy around
/// it, as in third-person console games: corridors a little wider than the character
/// is tall, doors twice its height, walls more than three times.
pub fn sizes_for(height: f32) -> (f32, f32, f32) {
    let round = |v: f32| (v * 10.).round() / 10.;
    let cell = round((height * 1.63).clamp(CELL_SIZE_LIMITS.0, CELL_SIZE_LIMITS.1));
    let wall = round((height * 3.26).clamp(WALL_HEIGHT_LIMITS.0, WALL_HEIGHT_LIMITS.1));
    let door = round((height * 1.96).clamp(0.5, wall));
    (cell, wall, door)
}
pub const WALL_HEIGHT_LIMITS: (f32, f32) = (0.5, 12.);

impl Generator {
    pub fn new(root: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            root,
            ..Self::default()
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.source == Source::Rooms {
            self.rooms.validate()?;
        }
        let (lo, hi) = CELL_SIZE_LIMITS;
        if !(lo..=hi).contains(&self.cell_size) {
            return Err(format!("Cell size must be {lo}-{hi} units"));
        }
        let (lo, hi) = WALL_HEIGHT_LIMITS;
        if !(lo..=hi).contains(&self.wall_height) {
            return Err(format!("Wall height must be {lo}-{hi} units"));
        }
        if !(0.5..=self.wall_height).contains(&self.door_height) {
            return Err("Door height must be at least 0.5 and at most the wall height".into());
        }
        Ok(())
    }
    /// Half the 3D footprint of a layout of `size` cells along the longer axis.
    /// Compound collider parts and mesh vertices are local to the root and must stay
    /// within ±128 units.
    pub fn half_extent(&self, size: [i32; 2]) -> f32 {
        size[0].max(size[1]) as f32 * self.cell_size * 0.5
    }
    /// A short stable digest of everything that shapes the output.
    pub fn fingerprint(&self) -> String {
        use sha2::{Digest, Sha256};
        let mut settings = self.clone();
        settings.output = Output::default();
        let bytes = serde_json::to_vec(&settings).expect("generator settings serialize");
        Sha256::digest(bytes)[..8]
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }
    /// The output exists and was produced by different settings.
    pub fn stale(&self) -> bool {
        !self.output.is_empty() && self.output.fingerprint != self.fingerprint()
    }
    /// The output exists and was produced by a different version of its style.
    pub fn style_stale(&self, digest: &str) -> bool {
        !self.output.is_empty() && self.output.style != digest
    }
    /// The output exists and was produced by a different version of its recipe.
    pub fn recipe_stale(&self, digest: &str) -> bool {
        !self.output.is_empty() && self.output.recipe != digest
    }
    /// A stable identity for generated content with this role.
    pub fn derived_id(&self, role: &str) -> Uuid {
        derived_id(self.id, role)
    }
}
pub fn derived_id(owner: Uuid, role: &str) -> Uuid {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest([owner.as_bytes().as_slice(), role.as_bytes()].concat());
    let mut bytes: [u8; 16] = digest[..16].try_into().expect("digest is 32 bytes");
    bytes[6] = (bytes[6] & 15) | 0x50;
    bytes[8] = (bytes[8] & 63) | 0x80;
    Uuid::from_bytes(bytes)
}

// ---- layout ------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Empty,
    Room(u16),
    Corridor,
}
impl Cell {
    pub fn walkable(self) -> bool {
        self != Cell::Empty
    }
}

/// The four sides of a cell. Grid X grows east; grid Y grows south in 2D and towards
/// +Z in 3D.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Side {
    East,
    South,
    West,
    North,
}
impl Side {
    pub const ALL: [Side; 4] = [Side::East, Side::South, Side::West, Side::North];
    pub fn delta(self) -> [i32; 2] {
        match self {
            Side::East => [1, 0],
            Side::South => [0, 1],
            Side::West => [-1, 0],
            Side::North => [0, -1],
        }
    }
    /// The next side clockwise (east, south, west, north).
    pub fn next(self) -> Side {
        match self {
            Side::East => Side::South,
            Side::South => Side::West,
            Side::West => Side::North,
            Side::North => Side::East,
        }
    }
    pub fn opposite(self) -> Side {
        self.next().next()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Room {
    pub min: [i32; 2],
    pub size: [i32; 2],
}
impl Room {
    pub fn center(&self) -> [i32; 2] {
        [
            self.min[0] + self.size[0] / 2,
            self.min[1] + self.size[1] / 2,
        ]
    }
    pub fn contains(&self, p: [i32; 2]) -> bool {
        (0..2).all(|k| p[k] >= self.min[k] && p[k] < self.min[k] + self.size[k])
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Layout {
    pub size: [i32; 2],
    pub cells: Vec<Cell>,
    pub rooms: Vec<Room>,
    /// Rooms joined through a corridor network, each pair once, lower index first.
    pub links: Vec<[usize; 2]>,
    pub spawn: usize,
    pub goal: usize,
    /// Rooms the settings asked for; `rooms.len()` is how many fit.
    pub requested: usize,
    /// Connections beyond a spanning tree.
    pub loops: usize,
    /// Doors that need a key, on the side of the room they guard.
    pub locks: Vec<GuardedDoor>,
    /// Doors passable only into their room.
    pub one_ways: Vec<GuardedDoor>,
    /// Keys and other items placed on floor cells.
    pub items: Vec<Item>,
    /// The recipe path each room belongs to; empty for rooms-and-corridors layouts.
    pub room_paths: Vec<String>,
    /// Floor levels above the lowest, by walkable cell; cells not listed are level 0.
    pub levels: BTreeMap<[i32; 2], u8>,
    /// Stair cells: each climbs one level towards its side, from the cell's level.
    pub stairs: BTreeMap<[i32; 2], Side>,
}

/// A door on a room's side: `cell` is the room cell, `side` faces the corridor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GuardedDoor {
    pub cell: [i32; 2],
    pub side: Side,
    /// The key that opens it, for locked doors; 0 otherwise.
    pub lock: u16,
}

/// Something a recipe places in a room: a key (with the lock it opens) or a named item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub cell: [i32; 2],
    pub name: String,
    /// The lock this key opens, 1 and up; 0 for other items.
    pub lock: u16,
}
/// How far one stair step rises: below the character's step-up and navigation's climb.
pub const STEP_RISE: f32 = 0.4;
/// Steps in one stair cell.
pub const STEPS_PER_LEVEL: usize = 4;
/// The height between floor levels: one stair cell.
pub const LEVEL_HEIGHT: f32 = STEP_RISE * STEPS_PER_LEVEL as f32;

impl Layout {
    /// A walkable cell's floor level; 0 elsewhere.
    pub fn level(&self, p: [i32; 2]) -> u8 {
        self.levels.get(&p).copied().unwrap_or(0)
    }
    /// The side a stair cell climbs towards.
    pub fn stair(&self, p: [i32; 2]) -> Option<Side> {
        self.stairs.get(&p).copied()
    }
    /// The level at a cell's highest floor point: one above its level on stairs.
    pub fn top_level(&self, p: [i32; 2]) -> u8 {
        self.level(p) + u8::from(self.stair(p).is_some())
    }
    /// A cell's floor height, at its low end on stairs.
    pub fn floor(&self, p: [i32; 2]) -> f32 {
        f32::from(self.level(p)) * LEVEL_HEIGHT
    }
    /// A cell's highest floor point.
    pub fn top(&self, p: [i32; 2]) -> f32 {
        f32::from(self.top_level(p)) * LEVEL_HEIGHT
    }
    /// The highest floor level any cell reaches.
    pub fn highest_level(&self) -> u8 {
        self.levels
            .keys()
            .map(|p| self.top_level(*p))
            .chain(self.stairs.keys().map(|p| self.top_level(*p)))
            .max()
            .unwrap_or(0)
    }
    pub fn inside(&self, p: [i32; 2]) -> bool {
        p[0] >= 0 && p[1] >= 0 && p[0] < self.size[0] && p[1] < self.size[1]
    }
    pub fn at(&self, p: [i32; 2]) -> Cell {
        if self.inside(p) {
            self.cells[(p[1] * self.size[0] + p[0]) as usize]
        } else {
            Cell::Empty
        }
    }
    /// Whether one can walk from cell `p` through its side `side`.
    pub fn open(&self, p: [i32; 2], side: Side) -> bool {
        let d = side.delta();
        let (a, b) = (self.at(p), self.at([p[0] + d[0], p[1] + d[1]]));
        match (a, b) {
            (Cell::Empty, _) | (_, Cell::Empty) => false,
            (Cell::Room(x), Cell::Room(y)) => x == y,
            _ => true,
        }
    }
    /// A room side that opens onto a corridor.
    pub fn door(&self, p: [i32; 2], side: Side) -> bool {
        let d = side.delta();
        matches!(
            (self.at(p), self.at([p[0] + d[0], p[1] + d[1]])),
            (Cell::Room(_), Cell::Corridor)
        )
    }
    /// An empty cell next to a floor, including diagonally: the solid ring around
    /// rooms and corridors.
    pub fn solid(&self, p: [i32; 2]) -> bool {
        self.at(p) == Cell::Empty
            && (-1..=1).any(|dy| (-1..=1).any(|dx| self.at([p[0] + dx, p[1] + dy]).walkable()))
    }
    pub fn walkable_cells(&self) -> usize {
        self.cells.iter().filter(|c| c.walkable()).count()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SpotKind {
    /// Every walkable cell.
    Floor,
    /// A closed side of a walkable cell, facing into it.
    Wall,
    /// A room side that opens onto a corridor.
    Door,
    /// The corridor cell just outside a door, facing the room.
    Doorway,
    /// A solid cell of the ring around rooms and corridors.
    Solid,
    /// Every walkable cell again, at the wall height (3D), facing down.
    Ceiling,
    /// An inner corner where two closed sides of a walkable cell meet.
    Corner,
    /// A point along a straight run of room wall where a column stands, at the start
    /// of its cell side: about one every two cells, evenly spaced, facing into the
    /// room.
    Column,
    /// The centre of every room.
    Room,
    /// A key a recipe placed; the lock it opens is the spot's tag.
    Key,
    /// A door that needs a key, on the guarded room's side, facing into it.
    Lock,
    /// A door passable only into its room, facing into it.
    OneWay,
    /// A named item a recipe placed.
    Item,
    /// The centre of the spawn room.
    Spawn,
    /// The centre of the goal room.
    Goal,
}
impl SpotKind {
    pub const ALL: [SpotKind; 15] = [
        SpotKind::Floor,
        SpotKind::Wall,
        SpotKind::Door,
        SpotKind::Doorway,
        SpotKind::Solid,
        SpotKind::Ceiling,
        SpotKind::Corner,
        SpotKind::Column,
        SpotKind::Room,
        SpotKind::Key,
        SpotKind::Lock,
        SpotKind::OneWay,
        SpotKind::Item,
        SpotKind::Spawn,
        SpotKind::Goal,
    ];
    pub fn label(self) -> &'static str {
        match self {
            SpotKind::Floor => "Floor",
            SpotKind::Wall => "Wall",
            SpotKind::Door => "Door",
            SpotKind::Doorway => "Doorway",
            SpotKind::Solid => "Solid",
            SpotKind::Ceiling => "Ceiling",
            SpotKind::Corner => "Corner",
            SpotKind::Column => "Column",
            SpotKind::Room => "Room",
            SpotKind::Key => "Key",
            SpotKind::Lock => "Locked door",
            SpotKind::OneWay => "One-way door",
            SpotKind::Item => "Item",
            SpotKind::Spawn => "Spawn",
            SpotKind::Goal => "Goal",
        }
    }
    pub fn description(self) -> &'static str {
        match self {
            SpotKind::Floor => "Every floor cell of rooms and corridors.",
            SpotKind::Wall => "Each closed side of a floor cell, facing into the cell.",
            SpotKind::Door => "Each room side that opens onto a corridor, facing into the room.",
            SpotKind::Doorway => "The corridor cell just outside each door, facing the room.",
            SpotKind::Solid => "Each solid cell of the ring that encloses the floor.",
            SpotKind::Ceiling => {
                "Each floor cell again, at the wall height and facing down (3D, when the generator has a ceiling)."
            }
            SpotKind::Corner => "Each inner corner where two walls of a floor cell meet.",
            SpotKind::Column => {
                "Evenly spaced points along the straight walls of rooms, about one every two cells."
            }
            SpotKind::Room => "The centre of every room, once per room.",
            SpotKind::Key => "Each key a recipe placed; actors here get the lock number it opens.",
            SpotKind::Lock => {
                "Each door that needs a key, facing into the room it guards; actors here get the lock number."
            }
            SpotKind::OneWay => "Each door passable only into its room, facing into it.",
            SpotKind::Item => "Each item a recipe placed; pieces can pick items by name.",
            SpotKind::Spawn => "The centre of the spawn room.",
            SpotKind::Goal => "The centre of the goal room, the furthest from the spawn.",
        }
    }
}

/// A named place in the layout where content belongs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spot {
    pub kind: SpotKind,
    pub cell: [i32; 2],
    /// The cell side for walls and doors.
    pub side: Option<Side>,
    pub room: Option<usize>,
    /// Corridor floor rather than room floor.
    pub corridor: bool,
    /// Keys and locked doors: the lock number. Items: the index in `Layout::items`.
    pub tag: u16,
    /// The floor level the spot stands on; for ceilings, the level under the ceiling's
    /// highest point. Solid cells take their highest walkable neighbour.
    pub level: u8,
}

/// Column spots for each straight run of room wall: `(L - 1) / 2` columns spread
/// evenly over a run of `L` cells, none on runs shorter than three.
fn columns(walls: &[(u8, i32, i32, usize)]) -> Vec<Spot> {
    let mut sorted = walls.to_vec();
    sorted.sort_unstable();
    let mut out = Vec::new();
    let mut start = 0;
    while start < sorted.len() {
        let (side, line, first, room) = sorted[start];
        let mut end = start + 1;
        while end < sorted.len()
            && sorted[end].0 == side
            && sorted[end].1 == line
            && sorted[end].2 == sorted[end - 1].2 + 1
        {
            end += 1;
        }
        let length = (end - start) as i32;
        let count = (length - 1) / 2;
        for j in 1..=count {
            let along = first + (length * j + (count + 1) / 2) / (count + 1);
            let side = Side::ALL[usize::from(side)];
            let cell = match side {
                Side::North | Side::South => [along, line],
                Side::East | Side::West => [line, along],
            };
            out.push(Spot {
                kind: SpotKind::Column,
                cell,
                side: Some(side),
                room: Some(room),
                corridor: false,
                level: 0,
                tag: 0,
            });
        }
        start = end;
    }
    out
}

/// Every spot of a layout: per cell in grid order its floor, walls, doors and corners,
/// or its solid block; then doorways, the spawn and the goal.
pub fn spots(layout: &Layout) -> Vec<Spot> {
    let mut out = Vec::new();
    let mut doorways = Vec::new();
    // Room wall sides as (side, line, position along the line, room).
    let mut walls: Vec<(u8, i32, i32, usize)> = Vec::new();
    for y in 0..layout.size[1] {
        for x in 0..layout.size[0] {
            let cell = layout.at([x, y]);
            if !cell.walkable() {
                if layout.solid([x, y]) {
                    out.push(Spot {
                        kind: SpotKind::Solid,
                        cell: [x, y],
                        side: None,
                        room: None,
                        corridor: false,
                        level: 0,
                        tag: 0,
                    });
                }
                continue;
            }
            let room = match cell {
                Cell::Room(r) => Some(usize::from(r)),
                _ => None,
            };
            let spot = |kind, side| Spot {
                kind,
                cell: [x, y],
                side,
                room,
                corridor: cell == Cell::Corridor,
                level: 0,
                tag: 0,
            };
            out.push(spot(SpotKind::Floor, None));
            out.push(spot(SpotKind::Ceiling, None));
            let closed = |side| !layout.open([x, y], side) && !layout.door([x, y], side);
            for side in Side::ALL {
                if layout.door([x, y], side) {
                    out.push(spot(SpotKind::Door, Some(side)));
                    let d = side.delta();
                    doorways.push(Spot {
                        kind: SpotKind::Doorway,
                        cell: [x + d[0], y + d[1]],
                        side: Some(side.opposite()),
                        room: None,
                        corridor: true,
                        level: 0,
                        tag: 0,
                    });
                } else if closed(side) {
                    out.push(spot(SpotKind::Wall, Some(side)));
                }
            }
            for side in Side::ALL {
                if closed(side) && closed(side.next()) {
                    out.push(spot(SpotKind::Corner, Some(side)));
                }
                if let Some(room) = room
                    && closed(side)
                {
                    let (line, along) = match side {
                        Side::North | Side::South => (y, x),
                        Side::East | Side::West => (x, y),
                    };
                    walls.push((side as u8, line, along, room));
                }
            }
        }
    }
    out.extend(columns(&walls));
    out.extend(doorways);
    let room_of = |p: [i32; 2]| match layout.at(p) {
        Cell::Room(r) => Some(usize::from(r)),
        _ => None,
    };
    for (index, room) in layout.rooms.iter().enumerate() {
        out.push(Spot {
            kind: SpotKind::Room,
            cell: room.center(),
            side: None,
            room: Some(index),
            corridor: false,
            level: 0,
            tag: 0,
        });
    }
    for (kind, doors) in [
        (SpotKind::Lock, &layout.locks),
        (SpotKind::OneWay, &layout.one_ways),
    ] {
        for door in doors {
            out.push(Spot {
                kind,
                cell: door.cell,
                side: Some(door.side),
                room: room_of(door.cell),
                corridor: false,
                level: 0,
                tag: door.lock,
            });
        }
    }
    for (index, item) in layout.items.iter().enumerate() {
        out.push(Spot {
            kind: if item.lock > 0 {
                SpotKind::Key
            } else {
                SpotKind::Item
            },
            cell: item.cell,
            side: None,
            room: room_of(item.cell),
            corridor: layout.at(item.cell) == Cell::Corridor,
            level: 0,
            tag: if item.lock > 0 {
                item.lock
            } else {
                index as u16
            },
        });
    }
    for (kind, room) in [
        (SpotKind::Spawn, layout.spawn),
        (SpotKind::Goal, layout.goal),
    ] {
        if let Some(r) = layout.rooms.get(room) {
            out.push(Spot {
                kind,
                cell: r.center(),
                side: None,
                room: Some(room),
                corridor: false,
                level: 0,
                tag: 0,
            });
        }
    }
    if layout.levels.is_empty() && layout.stairs.is_empty() {
        return out;
    }
    for spot in &mut out {
        let [x, y] = spot.cell;
        spot.level = match spot.kind {
            SpotKind::Ceiling => layout.top_level(spot.cell),
            // A solid block reaches as high as the floors around it.
            SpotKind::Solid => (-1..=1)
                .flat_map(|dy| (-1..=1).map(move |dx| [x + dx, y + dy]))
                .filter(|p| layout.at(*p).walkable())
                .map(|p| layout.top_level(p))
                .max()
                .unwrap_or(0),
            _ => layout.level(spot.cell),
        };
    }
    out
}

/// Generates the layout a generator describes.
/// The layout of a generator's settings. Recipe layouts find their recipe in `index`.
pub fn layout(generator: &Generator, index: &crate::assets::Index) -> Result<Layout, String> {
    match generator.source {
        Source::Rooms => {
            generator.rooms.validate()?;
            crate::dungeon_rooms::generate(&generator.rooms, generator.seed)
        }
        Source::Recipe => {
            let (recipe, _) = crate::dungeon_recipe::of(index, generator)?;
            crate::dungeon_recipe::layout(&recipe, generator.seed)
        }
    }
}

/// One sentence describing a layout for the Inspector and logs.
pub fn summary(layout: &Layout) -> String {
    let rooms = layout.rooms.len();
    let mut text = format!(
        "{rooms} room{} · {} connection{} · {} loop{}",
        if rooms == 1 { "" } else { "s" },
        layout.links.len(),
        if layout.links.len() == 1 { "" } else { "s" },
        layout.loops,
        if layout.loops == 1 { "" } else { "s" },
    );
    if rooms < layout.requested {
        text.push_str(&format!(" · {} of {} rooms fit", rooms, layout.requested));
    }
    let count =
        |n: usize, one: &str, many: &str| format!(" · {n} {}", if n == 1 { one } else { many });
    let keys: BTreeSet<u16> = layout.locks.iter().map(|l| l.lock).collect();
    if !keys.is_empty() {
        text.push_str(&count(keys.len(), "key and lock", "keys and locks"));
    }
    // Doors wider than a cell hold one entry per cell; count each run once.
    let one_ways = layout
        .one_ways
        .iter()
        .filter(|d| {
            let along = d.side.next().delta();
            let before = [d.cell[0] - along[0], d.cell[1] - along[1]];
            !layout
                .one_ways
                .iter()
                .any(|o| o.side == d.side && o.cell == before)
        })
        .count();
    if one_ways > 0 {
        text.push_str(&count(one_ways, "one-way door", "one-way doors"));
    }
    let items = layout.items.iter().filter(|i| i.lock == 0).count();
    if items > 0 {
        text.push_str(&count(items, "item", "items"));
    }
    // A flight across a wide corridor holds one stair cell per lane; count it once.
    let flights = layout
        .stairs
        .iter()
        .filter(|(cell, up)| {
            let across = up.next().delta();
            let before = [cell[0] - across[0], cell[1] - across[1]];
            layout.stair(before) != Some(**up)
        })
        .count();
    if flights > 0 {
        text.push_str(&count(
            usize::from(layout.highest_level()) + 1,
            "level",
            "levels",
        ));
        text.push_str(&count(flights, "flight of stairs", "flights of stairs"));
    }
    text
}

#[cfg(test)]
/// A character picture of a layout for tests: `#` room, `+` corridor, `S`/`G`
/// spawn and goal, `.` empty.
pub fn picture(layout: &Layout) -> String {
    let mut text = String::new();
    let spawn = layout.rooms.get(layout.spawn).map(|r| r.center());
    let goal = layout.rooms.get(layout.goal).map(|r| r.center());
    for y in 0..layout.size[1] {
        for x in 0..layout.size[0] {
            text.push(if Some([x, y]) == spawn {
                'S'
            } else if Some([x, y]) == goal {
                'G'
            } else {
                match layout.at([x, y]) {
                    Cell::Empty => '.',
                    Cell::Room(_) => '#',
                    Cell::Corridor => '+',
                }
            });
        }
        text.push('\n');
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generator(seed: u32) -> Generator {
        Generator {
            id: Uuid::from_u128(7),
            seed,
            ..Generator::default()
        }
    }

    fn reachable(layout: &Layout) -> usize {
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
        seen.len()
    }

    #[test]
    fn seeds_are_reproducible_and_distinct() {
        let a = layout(&generator(5), &Default::default()).unwrap();
        assert_eq!(a, layout(&generator(5), &Default::default()).unwrap());
        assert_ne!(
            a.cells,
            layout(&generator(6), &Default::default()).unwrap().cells
        );
    }

    #[test]
    fn every_walkable_cell_is_reachable_from_the_spawn() {
        for seed in 0..120 {
            let layout = layout(&generator(seed), &Default::default()).unwrap();
            assert!(layout.rooms.len() >= 2, "seed {seed}");
            assert_ne!(layout.spawn, layout.goal);
            assert_eq!(
                reachable(&layout),
                layout.walkable_cells(),
                "seed {seed}\n{}",
                picture(&layout)
            );
            // Every room has a way in.
            let spots = spots(&layout);
            for room in 0..layout.rooms.len() {
                assert!(
                    spots
                        .iter()
                        .any(|s| s.kind == SpotKind::Door && s.room == Some(room)),
                    "seed {seed} room {room}\n{}",
                    picture(&layout)
                );
            }
        }
    }

    #[test]
    fn corridors_keep_a_wall_cell_from_rooms_they_do_not_enter() {
        // A corridor cell touching a room (even diagonally) must open into it through a
        // door somewhere along that room's side, so 2D maps always have a wall between
        // a room and a corridor that merely passes by.
        for seed in 0..60 {
            let layout = layout(&generator(seed), &Default::default()).unwrap();
            for y in 0..layout.size[1] {
                for x in 0..layout.size[0] {
                    if layout.at([x, y]) != Cell::Corridor {
                        continue;
                    }
                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            if let Cell::Room(r) = layout.at([x + dx, y + dy]) {
                                let room = usize::from(r);
                                assert!(
                                    layout.links.iter().any(|l| l.contains(&room)),
                                    "seed {seed}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn loops_follow_the_setting() {
        let mut none = generator(11);
        none.rooms.loops = 0;
        let mut all = generator(11);
        all.rooms.loops = 100;
        let (none, all) = (
            layout(&none, &Default::default()).unwrap(),
            layout(&all, &Default::default()).unwrap(),
        );
        assert!(all.loops > none.loops, "{} vs {}", all.loops, none.loops);
    }

    #[test]
    fn settings_are_validated_with_actionable_messages() {
        let mut g = generator(1);
        g.rooms.room_min = 9;
        g.rooms.room_max = 4;
        assert!(g.validate().unwrap_err().contains("smallest first"));
        let mut g = generator(1);
        g.rooms.size = [8, 40];
        assert!(g.validate().is_err());
        let mut g = generator(1);
        g.door_height = 9.;
        assert!(g.validate().unwrap_err().contains("Door height"));
        let mut crowded = generator(1);
        crowded.rooms.size = [12, 12];
        crowded.rooms.room_min = 9;
        crowded.rooms.room_max = 9;
        assert!(
            layout(&crowded, &Default::default())
                .unwrap_err()
                .contains("Fewer than two rooms")
        );
    }

    #[test]
    fn fingerprints_track_settings_but_not_output() {
        let mut g = generator(3);
        let before = g.fingerprint();
        g.output.actors.push(Uuid::from_u128(1));
        assert_eq!(before, g.fingerprint());
        g.output.fingerprint = before.clone();
        assert!(!g.stale());
        g.seed = 4;
        assert!(g.stale());
        assert_eq!(g.derived_id("a"), g.derived_id("a"));
        assert_ne!(g.derived_id("a"), g.derived_id("b"));
    }

    #[test]
    #[ignore]
    fn print_a_layout() {
        let layout = layout(&generator(1), &Default::default()).unwrap();
        println!("{}\n{}", summary(&layout), picture(&layout));
    }
}
