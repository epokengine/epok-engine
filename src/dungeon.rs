//! Dungeon Generator: seeded layouts baked into ordinary map content.
//!
//! A generator lives in the map document next to the actor that places it. Its layout
//! is a grid of cells grouped into rooms and corridors; `spots` names every place where
//! content belongs (floors, walls, doors, spawn, goal). `dungeon_bake` turns spots into
//! EditableMesh sections, a compound collider and marker actors in 3D, or TileMapLayer
//! cells in 2D. Decisions use integers and `dungeon_random`, so a seed always produces
//! the same dungeon.
use serde::{Deserialize, Serialize};
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
            return Err(format!("Grid size must be {lo}–{hi} cells per side"));
        }
        if !(ROOM_LIMITS.0..=ROOM_LIMITS.1).contains(&self.rooms) {
            return Err(format!(
                "Room count must be {}–{}",
                ROOM_LIMITS.0, ROOM_LIMITS.1
            ));
        }
        let (min, max) = ROOM_SIZE_LIMITS;
        if !(min..=max).contains(&self.room_min)
            || !(min..=max).contains(&self.room_max)
            || self.room_min > self.room_max
        {
            return Err(format!(
                "Room sizes must be {min}–{max} cells, smallest first"
            ));
        }
        if usize::from(self.room_max) + 2 > usize::from(*self.size.iter().min().unwrap()) {
            return Err("The largest room must fit inside the grid".into());
        }
        if !(SPACING_LIMITS.0..=SPACING_LIMITS.1).contains(&self.spacing) {
            return Err(format!(
                "Room spacing must be {}–{} cells",
                SPACING_LIMITS.0, SPACING_LIMITS.1
            ));
        }
        if !(CORRIDOR_LIMITS.0..=CORRIDOR_LIMITS.1).contains(&self.corridor_width) {
            return Err(format!(
                "Corridor width must be {}–{} cells",
                CORRIDOR_LIMITS.0, CORRIDOR_LIMITS.1
            ));
        }
        if self.loops > 100 {
            return Err("Loops is a percentage, 0–100".into());
        }
        Ok(())
    }
}

/// What a generation wrote into the map, so regenerating replaces exactly that.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Output {
    pub actors: Vec<Uuid>,
    pub assets: Vec<Uuid>,
    /// `fingerprint` of the settings that produced this output.
    pub fingerprint: String,
    /// Digest of the Dungeon Style it was generated with.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub style: String,
}
impl Output {
    pub fn is_empty(&self) -> bool {
        self.actors.is_empty() && self.assets.is_empty()
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
    pub rooms: RoomSettings,
    /// 3D world units per grid cell.
    pub cell_size: f32,
    pub wall_height: f32,
    pub door_height: f32,
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
            rooms: RoomSettings::default(),
            cell_size: 2.,
            wall_height: 3.,
            door_height: 2.25,
            navigation: true,
            spawn_actor: None,
            tile_set: None,
            style: None,
            output: Output::default(),
        }
    }
}
pub const CELL_SIZE_LIMITS: (f32, f32) = (0.5, 8.);
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
        self.rooms.validate()?;
        let (lo, hi) = CELL_SIZE_LIMITS;
        if !(lo..=hi).contains(&self.cell_size) {
            return Err(format!("Cell size must be {lo}–{hi} units"));
        }
        let (lo, hi) = WALL_HEIGHT_LIMITS;
        if !(lo..=hi).contains(&self.wall_height) {
            return Err(format!("Wall height must be {lo}–{hi} units"));
        }
        if !(0.5..=self.wall_height).contains(&self.door_height) {
            return Err("Door height must be at least 0.5 and at most the wall height".into());
        }
        Ok(())
    }
    /// Half the 3D footprint along the longer axis. Compound collider parts and mesh
    /// vertices are local to the root and must stay within ±128 units.
    pub fn half_extent(&self) -> f32 {
        f32::from(*self.rooms.size.iter().max().unwrap()) * self.cell_size * 0.5
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

#[derive(Clone, Debug, PartialEq, Eq)]
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
}
impl Layout {
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
    /// An inner corner where two closed sides of a walkable cell meet.
    Corner,
    /// The centre of every room.
    Room,
    /// The centre of the spawn room.
    Spawn,
    /// The centre of the goal room.
    Goal,
}
impl SpotKind {
    pub const ALL: [SpotKind; 9] = [
        SpotKind::Floor,
        SpotKind::Wall,
        SpotKind::Door,
        SpotKind::Doorway,
        SpotKind::Solid,
        SpotKind::Corner,
        SpotKind::Room,
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
            SpotKind::Corner => "Corner",
            SpotKind::Room => "Room",
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
            SpotKind::Corner => "Each inner corner where two walls of a floor cell meet.",
            SpotKind::Room => "The centre of every room, once per room.",
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
}

/// Every spot of a layout: per cell in grid order its floor, walls, doors and corners,
/// or its solid block; then doorways, the spawn and the goal.
pub fn spots(layout: &Layout) -> Vec<Spot> {
    let mut out = Vec::new();
    let mut doorways = Vec::new();
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
            };
            out.push(spot(SpotKind::Floor, None));
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
                    });
                } else if closed(side) {
                    out.push(spot(SpotKind::Wall, Some(side)));
                }
            }
            for side in Side::ALL {
                if closed(side) && closed(side.next()) {
                    out.push(spot(SpotKind::Corner, Some(side)));
                }
            }
        }
    }
    out.extend(doorways);
    for (index, room) in layout.rooms.iter().enumerate() {
        out.push(Spot {
            kind: SpotKind::Room,
            cell: room.center(),
            side: None,
            room: Some(index),
            corridor: false,
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
            });
        }
    }
    out
}

/// Generates the layout a generator describes.
pub fn layout(generator: &Generator) -> Result<Layout, String> {
    generator.rooms.validate()?;
    crate::dungeon_rooms::generate(&generator.rooms, generator.seed)
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
        let a = layout(&generator(5)).unwrap();
        assert_eq!(a, layout(&generator(5)).unwrap());
        assert_ne!(a.cells, layout(&generator(6)).unwrap().cells);
    }

    #[test]
    fn every_walkable_cell_is_reachable_from_the_spawn() {
        for seed in 0..120 {
            let layout = layout(&generator(seed)).unwrap();
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
            let layout = layout(&generator(seed)).unwrap();
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
        let (none, all) = (layout(&none).unwrap(), layout(&all).unwrap());
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
            layout(&crowded)
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
        let layout = layout(&generator(1)).unwrap();
        println!("{}\n{}", summary(&layout), picture(&layout));
    }
}
