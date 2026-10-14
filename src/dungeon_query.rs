//! The Dungeon gameplay queries: what a generation bakes about its dungeon for game code
//! (rooms and their links, keys, locks, items, free spots and the seed) and how a map
//! cooks it for `runtime/dungeon_map.hpp`. Positions stay in the generator root's space
//! until the map is cooked, so moving the generator keeps them right.
use crate::{
    dungeon::{Cell, Layout, Side},
    scene::Scene,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// The most rooms the runtime walks; rooms past it are left out of the queries.
pub const MAX_ROOMS: usize = 64;
/// Free spots kept per room.
pub const SPOTS: usize = 8;

pub const SPAWN: u8 = 1;
pub const GOAL: u8 = 2;
pub const KEY: u8 = 4;
pub const ITEM: u8 = 8;
pub const LOCKED: u8 = 16;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Map {
    pub two_d: bool,
    pub seed: u32,
    pub spawn: i16,
    pub goal: i16,
    pub rooms: Vec<Room>,
    /// Keys (lock above 0) and other items.
    pub items: Vec<Point>,
    /// One doorway per lock, in lock order.
    pub locks: Vec<Point>,
    /// The recipe's path names, for the editor; game code sees the indices.
    pub paths: Vec<String>,
    /// What the minimap draws, cooked when the generator's minimap is on.
    pub minimap: crate::dungeon_minimap::Baked,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Room {
    pub minimum: [f32; 3],
    pub maximum: [f32; 3],
    pub center: [f32; 3],
    pub level: i16,
    /// The recipe path, in recipe order; 0 is the main path.
    pub path: u16,
    pub flags: u8,
    /// Rooms from the spawn room, and to the goal room.
    pub depth: u8,
    pub to_goal: u8,
    pub neighbours: Vec<u8>,
    pub spots: Vec<[f32; 3]>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Point {
    pub position: [f32; 3],
    pub room: i16,
    pub lock: u16,
    /// The generated actor standing for it: a key's, a locked door's.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor: Option<uuid::Uuid>,
}

/// Where the grid lands in the generator root's space.
pub struct Grid<'a> {
    pub two_d: bool,
    /// A grid lattice point on the dungeon's plane: (x, z) in 3D, pixels (x, y) in 2D.
    pub corner: &'a dyn Fn(i32, i32) -> [f32; 2],
    /// A walkable cell's floor height in 3D.
    pub floor: &'a dyn Fn([i32; 2]) -> f32,
    /// How far above its floor a room reaches in 3D.
    pub height: f32,
}
impl Grid<'_> {
    fn point(&self, plane: [f32; 2], y: f32) -> [f32; 3] {
        if self.two_d {
            [plane[0], plane[1], 0.]
        } else {
            [plane[0], y, plane[1]]
        }
    }
    fn center(&self, p: [i32; 2]) -> [f32; 3] {
        let (a, b) = ((self.corner)(p[0], p[1]), (self.corner)(p[0] + 1, p[1] + 1));
        self.point([(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5], (self.floor)(p))
    }
    /// The middle of the edge between a cell and its neighbour across `side`.
    fn edge(&self, p: [i32; 2], side: Side) -> [f32; 3] {
        let d = side.delta();
        let (a, b) = (self.center(p), self.center([p[0] + d[0], p[1] + d[1]]));
        let mut mid = [0.; 3];
        for k in 0..3 {
            mid[k] = (a[k] + b[k]) * 0.5;
        }
        if !self.two_d {
            mid[1] = a[1];
        }
        mid
    }
}

/// Rooms reached from `from` in each number of links, over undirected links.
fn hops(adjacency: &[Vec<usize>], from: usize) -> Vec<Option<usize>> {
    let mut steps = vec![None; adjacency.len()];
    if from >= adjacency.len() {
        return steps;
    }
    steps[from] = Some(0);
    let mut queue = VecDeque::from([from]);
    while let Some(at) = queue.pop_front() {
        for &next in &adjacency[at] {
            if steps[next].is_none() {
                steps[next] = steps[at].map(|s| s + 1);
                queue.push_back(next);
            }
        }
    }
    steps
}

/// Builds the queries for one layout. `paths` names the recipe's paths in order; rooms
/// on other paths, and every room of a rooms-and-corridors layout, are on path 0.
pub fn build(layout: &Layout, seed: u32, paths: &[String], grid: &Grid) -> (Map, Vec<String>) {
    let mut notes = Vec::new();
    let count = layout.rooms.len().min(MAX_ROOMS);
    if layout.rooms.len() > MAX_ROOMS {
        notes.push(format!(
            "Gameplay queries cover the first {MAX_ROOMS} of {} rooms",
            layout.rooms.len()
        ));
    }
    let room_of = |p: [i32; 2]| match layout.at(p) {
        Cell::Room(i) if usize::from(i) < count => Some(usize::from(i)),
        _ => None,
    };
    let mut adjacency = vec![Vec::new(); count];
    for &[a, b] in &layout.links {
        if a < count && b < count && a != b {
            adjacency[a].push(b);
            adjacency[b].push(a);
        }
    }
    for list in &mut adjacency {
        list.sort_unstable();
        list.dedup();
    }
    let (from_spawn, from_goal) = (
        hops(&adjacency, layout.spawn),
        hops(&adjacency, layout.goal),
    );
    let mut flags = vec![0u8; count];
    if layout.spawn < count {
        flags[layout.spawn] |= SPAWN;
    }
    if layout.goal < count {
        flags[layout.goal] |= GOAL;
    }
    let items: Vec<Point> = layout
        .items
        .iter()
        .map(|item| {
            let room = room_of(item.cell);
            if let Some(r) = room {
                flags[r] |= if item.lock > 0 { KEY } else { ITEM };
            }
            Point {
                position: grid.center(item.cell),
                room: room.map_or(-1, |r| r as i16),
                lock: item.lock,
                actor: None,
            }
        })
        .collect();
    let mut by_lock: BTreeMap<u16, Vec<_>> = BTreeMap::new();
    for door in &layout.locks {
        by_lock.entry(door.lock).or_default().push(*door);
    }
    let locks: Vec<Point> = by_lock
        .into_iter()
        .map(|(lock, doors)| {
            let door = doors[doors.len() / 2];
            let room = room_of(door.cell);
            if let Some(r) = room {
                flags[r] |= LOCKED;
            }
            Point {
                position: grid.edge(door.cell, door.side),
                room: room.map_or(-1, |r| r as i16),
                lock,
                actor: None,
            }
        })
        .collect();
    let taken: BTreeSet<[i32; 2]> = layout.items.iter().map(|i| i.cell).collect();
    let rooms = (0..count)
        .map(|index| {
            let room = &layout.rooms[index];
            let cells: Vec<[i32; 2]> = (room.min[1]..room.min[1] + room.size[1])
                .flat_map(|y| (room.min[0]..room.min[0] + room.size[0]).map(move |x| [x, y]))
                .filter(|p| room_of(*p) == Some(index))
                .collect();
            let doors: Vec<[i32; 2]> = cells
                .iter()
                .copied()
                .filter(|p| Side::ALL.iter().any(|s| layout.door(*p, *s)))
                .collect();
            let (lo, hi) = (
                (grid.corner)(room.min[0], room.min[1]),
                (grid.corner)(room.min[0] + room.size[0], room.min[1] + room.size[1]),
            );
            let floors = cells.iter().map(|p| (grid.floor)(*p));
            let low = floors.clone().reduce(f32::min).unwrap_or(0.);
            let high = floors.reduce(f32::max).unwrap_or(0.);
            let (a, b) = (grid.point(lo, low), grid.point(hi, high + grid.height));
            let level = if grid.two_d {
                0
            } else {
                (low / LEVEL_UNIT).round() as i16
            };
            Room {
                minimum: std::array::from_fn(|k| a[k].min(b[k])),
                maximum: std::array::from_fn(|k| a[k].max(b[k])),
                center: grid.center(room.center()),
                level,
                path: layout
                    .room_paths
                    .get(index)
                    .and_then(|name| paths.iter().position(|p| p == name))
                    .unwrap_or(0) as u16,
                flags: flags[index],
                depth: from_spawn[index].map_or(u8::MAX, |s| s.min(254) as u8),
                to_goal: from_goal[index].map_or(u8::MAX, |s| s.min(254) as u8),
                neighbours: adjacency[index].iter().map(|n| *n as u8).collect(),
                spots: spots(layout, &cells, &doors, &taken)
                    .into_iter()
                    .map(|p| grid.center(p))
                    .collect(),
            }
        })
        .collect();
    let spawn = if layout.spawn < count {
        layout.spawn as i16
    } else {
        -1
    };
    let goal = if layout.goal < count {
        layout.goal as i16
    } else {
        -1
    };
    let (minimap, minimap_notes) =
        crate::dungeon_minimap::build(layout, count, grid.corner, !grid.two_d);
    notes.extend(minimap_notes);
    (
        Map {
            two_d: grid.two_d,
            seed,
            spawn,
            goal,
            rooms,
            items,
            locks,
            paths: paths.to_vec(),
            minimap,
        },
        notes,
    )
}

/// Names the generated actors standing for keys and locked doors: `keys` and `doors`
/// by lock number.
pub fn link_actors(
    map: &mut Map,
    keys: &BTreeMap<u16, uuid::Uuid>,
    doors: &BTreeMap<u16, uuid::Uuid>,
) {
    for item in map.items.iter_mut().filter(|i| i.lock > 0) {
        item.actor = keys.get(&item.lock).copied();
    }
    for lock in &mut map.locks {
        lock.actor = doors.get(&lock.lock).copied();
    }
}

/// One floor level's height in world units, for room levels.
pub const LEVEL_UNIT: f32 = 1.6;

/// Up to `SPOTS` free floor cells of a room, spread out: away from its doors, keys and
/// items, and off its walls when the room has an inside.
fn spots(
    layout: &Layout,
    cells: &[[i32; 2]],
    doors: &[[i32; 2]],
    taken: &BTreeSet<[i32; 2]>,
) -> Vec<[i32; 2]> {
    let manhattan = |a: [i32; 2], b: [i32; 2]| (a[0] - b[0]).abs() + (a[1] - b[1]).abs();
    let free: Vec<[i32; 2]> = cells
        .iter()
        .copied()
        .filter(|p| !taken.contains(p) && doors.iter().all(|d| manhattan(*d, *p) > 1))
        .collect();
    let inside: Vec<[i32; 2]> = free
        .iter()
        .copied()
        .filter(|p| {
            Side::ALL.iter().all(|s| {
                let d = s.delta();
                layout.at([p[0] + d[0], p[1] + d[1]]).walkable()
            })
        })
        .collect();
    // A small room may have every cell next to a door: then any free floor will do,
    // and the floor itself before that, so every room has somewhere to put things.
    let pool = if !inside.is_empty() {
        inside
    } else if !free.is_empty() {
        free
    } else {
        let open: Vec<[i32; 2]> = cells
            .iter()
            .copied()
            .filter(|p| !taken.contains(p))
            .collect();
        if open.is_empty() {
            cells.to_vec()
        } else {
            open
        }
    };
    // Farthest-point sampling: the cell farthest from the doors first, then each next
    // one farthest from those already chosen. Ties go to the first cell in grid order.
    let mut chosen: Vec<[i32; 2]> = Vec::new();
    while chosen.len() < SPOTS.min(pool.len()) {
        let anchors: &[[i32; 2]] = if chosen.is_empty() { doors } else { &chosen };
        let best = pool
            .iter()
            .copied()
            .filter(|p| !chosen.contains(p))
            .max_by_key(|p| {
                let distance = anchors.iter().map(|a| manhattan(*a, *p)).min().unwrap_or(0);
                (distance, std::cmp::Reverse((p[1], p[0])))
            });
        match best {
            Some(p) => chosen.push(p),
            None => break,
        }
    }
    chosen
}

/// The first generated dungeon's queries in world space, as constant tables and the
/// statement that makes them the map's; an empty reset when the map has none.
pub fn cpp(scene: &Scene) -> (String, String) {
    const NONE: &str = "inline constexpr size_t dungeon_minimap_capacity=0;\n";
    let found = scene.dungeons.iter().find_map(|g| {
        let map = g.output.queries.as_ref()?;
        Some((g, map, scene.actor_index(g.root)?))
    });
    let Some((generator, map, root)) = found.filter(|(_, map, _)| !map.rooms.is_empty()) else {
        return (
            NONE.into(),
            "::epok::dungeon::reset({});\n::epok::dungeon::reset_minimap({});\n".into(),
        );
    };
    let world = |p: [f32; 3]| -> [f32; 3] {
        if map.two_d {
            let q = crate::world2d::world_point(scene, root, [p[0], p[1]]);
            [q[0], q[1], 0.]
        } else {
            scene.world_matrix(root).point(p)
        }
    };
    let bounds = |lo: [f32; 3], hi: [f32; 3]| {
        let mut min = [f32::MAX; 3];
        let mut max = [f32::MIN; 3];
        for corner in 0..8 {
            let p = world(std::array::from_fn(|k| {
                if corner >> k & 1 == 0 { lo[k] } else { hi[k] }
            }));
            for k in 0..3 {
                min[k] = min[k].min(p[k]);
                max[k] = max[k].max(p[k]);
            }
        }
        (min, max)
    };
    let raw = |p: [f32; 3]| {
        p.iter()
            .map(|v| ((v * 4096.).round() as i32).to_string())
            .collect::<Vec<_>>()
            .join(",")
    };
    let point = |p: &Point| format!("{{{{{}}},{},{}}},", raw(world(p.position)), p.room, p.lock);
    use std::fmt::Write;
    let mut out = String::from("inline constexpr ::epok::dungeon::RoomData dungeon_rooms[]={\n");
    let (mut neighbours, mut spots) = (Vec::new(), Vec::new());
    for (index, r) in map.rooms.iter().enumerate() {
        let (min, max) = bounds(r.minimum, r.maximum);
        writeln!(
            out,
            "{{{{{}}},{{{}}},{{{}}},{},{},{},{},{},{},{},{},{}}},",
            raw(min),
            raw(max),
            raw(world(r.center)),
            r.level,
            r.path,
            neighbours.len(),
            spots.len(),
            r.flags,
            r.depth,
            r.to_goal,
            r.neighbours.len(),
            r.spots.len()
        )
        .unwrap();
        neighbours.extend(r.neighbours.iter().copied());
        spots.extend(r.spots.iter().map(|p| Point {
            position: *p,
            room: index as i16,
            lock: 0,
            actor: None,
        }));
    }
    out.push_str("};\n");
    let mut table = |name: &str, kind: &str, rows: Vec<String>| -> String {
        if rows.is_empty() {
            return "nullptr".into();
        }
        writeln!(
            out,
            "inline constexpr {kind} {name}[]={{\n{}\n}};",
            rows.join("\n")
        )
        .unwrap();
        name.into()
    };
    let neighbours_name = table(
        "dungeon_neighbours",
        "uint8_t",
        neighbours.iter().map(|n| format!("{n},")).collect(),
    );
    let points = "::epok::dungeon::PointData";
    let spots_name = table("dungeon_spots", points, spots.iter().map(point).collect());
    let items_name = table(
        "dungeon_items",
        points,
        map.items.iter().map(point).collect(),
    );
    let locks_name = table(
        "dungeon_locks",
        points,
        map.locks.iter().map(point).collect(),
    );
    let (minimap_table, minimap_init) = minimap_cpp(scene, generator, map, &world);
    out.push_str(&minimap_table);
    let init = format!(
        "{minimap_init}::epok::dungeon::reset({{dungeon_rooms,{},{neighbours_name},{spots_name},{items_name},{},{locks_name},{},{},{},{}u,{}}});\n",
        map.rooms.len(),
        map.items.len(),
        map.locks.len(),
        map.spawn,
        map.goal,
        map.seed,
        map.two_d
    );
    (out, init)
}

/// The minimap's tables and the statement that makes them the map's: its rectangles,
/// where its grid lies in the world, the actors it watches and how it draws; an empty
/// reset when the generator has no minimap.
fn minimap_cpp(
    scene: &Scene,
    generator: &crate::dungeon::Generator,
    map: &Map,
    world: &dyn Fn([f32; 3]) -> [f32; 3],
) -> (String, String) {
    let settings = &generator.minimap;
    let baked = &map.minimap;
    if !settings.enabled || baked.rects.is_empty() {
        return (
            "inline constexpr size_t dungeon_minimap_capacity=0;\n".into(),
            "::epok::dungeon::reset_minimap({});\n".into(),
        );
    }
    let plane = |p: [f32; 2]| {
        let q = if map.two_d {
            world([p[0], p[1], 0.])
        } else {
            world([p[0], 0., p[1]])
        };
        if map.two_d {
            [q[0], q[1]]
        } else {
            [q[0], q[2]]
        }
    };
    let corner = plane(baked.corner);
    let next = plane([
        baked.corner[0] + baked.step[0],
        baked.corner[1] + baked.step[1],
    ]);
    let raw = |v: f32| (v * 4096.).round() as i32;
    let slot = |id: Option<uuid::Uuid>| {
        id.and_then(|id| scene.actor_index(id))
            .map_or(-1, |i| i as i32)
    };
    let count = baked.rects.len();
    let mut out = format!(
        "inline constexpr size_t dungeon_minimap_capacity={count};\ninline constexpr ::epok::dungeon::MapRect dungeon_map_rects[]={{\n"
    );
    for r in &baked.rects {
        out.push_str(&format!(
            "{{{},{},{},{},{},{{{},{}}},0}},\n",
            r.x, r.y, r.w, r.h, r.room, r.near[0], r.near[1]
        ));
    }
    out.push_str("};\n");
    let actors = |name: &str, points: &[Point]| -> (String, String) {
        if points.is_empty() {
            return ("nullptr".into(), String::new());
        }
        out_table(name, points.iter().map(|p| slot(p.actor)).collect())
    };
    let items = actors("dungeon_item_actors", &map.items);
    let locks = actors("dungeon_lock_actors", &map.locks);
    out.push_str(&items.1);
    out.push_str(&locks.1);
    out.push_str(&format!(
        "inline uint32_t dungeon_map_explored[{}];\n",
        count.div_ceil(32)
    ));
    let colors = settings
        .colors
        .iter()
        .map(|c| format!("{{{},{},{}}}", c[0], c[1], c[2]))
        .collect::<Vec<_>>()
        .join(",");
    let init = format!(
        "::epok::dungeon::reset_minimap({{dungeon_map_rects,{count},{},{},{{{},{}}},{{{},{}}},{},{},{},dungeon_map_explored,{},{},{},{},{},{},{{{colors}}}}});\n",
        baked.size[0],
        baked.size[1],
        raw(corner[0]),
        raw(corner[1]),
        raw(next[0] - corner[0]),
        raw(next[1] - corner[1]),
        slot(generator.spawn_actor),
        items.0,
        locks.0,
        settings.margin[0],
        settings.margin[1],
        settings.size[0],
        settings.size[1],
        settings.cell,
        settings.flags(),
    );
    (out, init)
}

/// A constant table of runtime object slots: its name and its definition.
fn out_table(name: &str, slots: Vec<i32>) -> (String, String) {
    (
        name.into(),
        format!(
            "inline constexpr int16_t {name}[]={{{}}};\n",
            slots
                .iter()
                .map(i32::to_string)
                .collect::<Vec<_>>()
                .join(",")
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dungeon_bake::Frame;

    fn built(seed: u32) -> (Layout, Map) {
        let recipe = crate::dungeon_recipe::builtin();
        let layout = crate::dungeon_recipe::layout(&recipe, seed).unwrap();
        let frame = Frame {
            size: layout.size,
            cell: 3.,
            height: 6.,
        };
        let corner = |x: i32, y: i32| [frame.x(x), frame.z(y)];
        let (map, notes) = build(
            &layout,
            seed,
            &recipe.path_names(),
            &Grid {
                two_d: false,
                corner: &corner,
                floor: &|_| 0.,
                height: 6.,
            },
        );
        assert!(notes.is_empty(), "{notes:?}");
        (layout, map)
    }

    /// The baked rooms, links, keys, locks and spots agree with the layout they came
    /// from, for several seeds of the built-in recipe.
    #[test]
    fn rooms_links_keys_and_spots_describe_the_layout() {
        for seed in [1, 7, 21, 1207] {
            let (layout, map) = built(seed);
            assert_eq!(map.rooms.len(), layout.rooms.len());
            let (spawn, goal) = (map.spawn as usize, map.goal as usize);
            assert_eq!((spawn, goal), (layout.spawn, layout.goal));
            assert!(map.rooms[spawn].flags & SPAWN != 0 && map.rooms[spawn].depth == 0);
            assert!(map.rooms[goal].flags & GOAL != 0 && map.rooms[goal].to_goal == 0);
            // Every room is reached, links go both ways, and a link changes the depth
            // from the spawn by at most one room.
            for (index, room) in map.rooms.iter().enumerate() {
                assert!(
                    room.depth < u8::MAX && room.to_goal < u8::MAX,
                    "seed {seed}"
                );
                for &n in &room.neighbours {
                    let other = &map.rooms[usize::from(n)];
                    assert!(other.neighbours.contains(&(index as u8)));
                    assert!(room.depth.abs_diff(other.depth) <= 1);
                }
                let inside = |p: [f32; 3]| {
                    (0..3).all(|k| p[k] >= room.minimum[k] - 1e-4 && p[k] <= room.maximum[k] + 1e-4)
                };
                assert!(inside(room.center));
                // Spots are distinct floor points of the room, off its doors and items.
                assert!(
                    room.spots.len() <= SPOTS && !room.spots.is_empty(),
                    "seed {seed}"
                );
                for (i, spot) in room.spots.iter().enumerate() {
                    assert!(inside(*spot));
                    assert!(!room.spots[..i].contains(spot));
                    assert!(map.items.iter().all(|item| item.position != *spot));
                }
            }
            // The built-in recipe has branches: some rooms are off the main path.
            assert!(map.rooms.iter().any(|r| r.path == 0));
            assert!(map.rooms.iter().any(|r| r.path > 0), "seed {seed}");
            // Keys sit in rooms flagged as key rooms; each lock guards a locked room.
            for item in &map.items {
                let room = &map.rooms[item.room as usize];
                assert_ne!(room.flags & if item.lock > 0 { KEY } else { ITEM }, 0);
            }
            let locks: BTreeSet<u16> = layout.locks.iter().map(|l| l.lock).collect();
            assert_eq!(map.locks.len(), locks.len());
            for lock in &map.locks {
                assert_ne!(map.rooms[lock.room as usize].flags & LOCKED, 0);
                assert!(map.items.iter().any(|i| i.lock == lock.lock));
            }
        }
    }

    /// The cook moves the tables with the generator and resets the map's queries when
    /// it has no dungeon, so a previous map's rooms never leak into the next one.
    #[test]
    fn the_cook_writes_world_tables_or_an_empty_reset() {
        let (_, map) = built(1207);
        let mut scene = Scene::default();
        scene.actors.clear();
        assert_eq!(
            cpp(&scene),
            (
                "inline constexpr size_t dungeon_minimap_capacity=0;\n".into(),
                "::epok::dungeon::reset({});\n::epok::dungeon::reset_minimap({});\n".into()
            )
        );
        let mut root = crate::scene::Actor::cube("Dungeon".into());
        root.kind = "Empty".into();
        root.position = [10., 0., -4.];
        let mut generator = crate::dungeon::Generator::new(root.id);
        generator.output.queries = Some(map.clone());
        scene.actors.push(root);
        scene.dungeons.push(generator);
        scene.refresh_actor_hierarchy();
        let (tables, init) = cpp(&scene);
        let first = &map.rooms[0];
        let raw = |v: f32| ((v * 4096.).round() as i32).to_string();
        let expected = format!(
            "{{{{{},{},{}}},",
            raw(first.minimum[0] + 10.),
            raw(first.minimum[1]),
            raw(first.minimum[2] - 4.)
        );
        assert!(tables.contains(&expected), "{expected} not in {tables}");
        assert!(
            tables.matches('\n').count() > map.rooms.len(),
            "one row per room"
        );
        assert!(init.contains(&format!(
            "::epok::dungeon::reset({{dungeon_rooms,{},dungeon_neighbours,dungeon_spots,",
            map.rooms.len()
        )));
        assert!(init.contains(&format!(",{},{},1207u,false}});", map.spawn, map.goal)));
        // Without its minimap on, the map cooks none and clears the last one.
        assert!(tables.contains("dungeon_minimap_capacity=0;"));
        assert!(init.starts_with("::epok::dungeon::reset_minimap({});"));
        // With it on, the rectangles, the grid moved with the generator, the player's
        // slot and the box settings.
        let mut player = crate::scene::Actor::cube("Player".into());
        player.kind = "Empty".into();
        let player_id = player.id;
        scene.actors.push(player);
        scene.dungeons[0].spawn_actor = Some(player_id);
        scene.dungeons[0].minimap.enabled = true;
        let (tables, init) = cpp(&scene);
        let baked = &map.minimap;
        assert!(tables.contains(&format!("dungeon_minimap_capacity={};", baked.rects.len())));
        assert_eq!(
            tables.matches("},0},\n").count(),
            baked.rects.len(),
            "one row per rectangle"
        );
        assert!(init.starts_with(&format!(
            "::epok::dungeon::reset_minimap({{dungeon_map_rects,{},{},{},{{{},{}}},{{12288,-12288}},1,",
            baked.rects.len(),
            baked.size[0],
            baked.size[1],
            raw(baked.corner[0] + 10.),
            raw(baked.corner[1] - 4.),
        )), "{init}");
        assert!(
            init.contains("dungeon_map_explored,12,12,96,72,3,17,"),
            "{init}"
        );
    }

    /// Blueprint and Lua reach every query through the real native catalog: a Lua class
    /// and a Blueprint graph calling them compile, as the C++ flavor calls the library.
    #[test]
    fn blueprint_and_lua_compile_calls_to_the_dungeon_queries() {
        use crate::blueprint_asset as bp;
        let parent =
            std::env::temp_dir().join(format!("epok-dungeon-queries-{}", uuid::Uuid::new_v4()));
        let root = parent.join("Queries");
        crate::workspace::create_with_options(
            &root,
            "Queries",
            crate::workspace::CreateOptions::new(crate::workspace::Template::Basic),
        )
        .unwrap();
        let lua = "assets/scripts/DungeonProbe.lua";
        crate::project::write_changed(
            &root.join(lua),
            br#"---@class DungeonProbe : epok.ActorComponent
local DungeonProbe = epok.ActorComponent:extend()

DungeonProbe.here = epok.Int32(-1)

---@override
function DungeonProbe:tick(delta_seconds)
    local here = epok.dungeon.room_of(epok.owner())
    local room = epok.dungeon.room(here)
    if room.valid and epok.dungeon.next_room(here, epok.dungeon.goal_room()) >= 0 then
        self.here = here
    end
    local spot = epok.dungeon.spot(here, epok.dungeon.random(1, 0, 8))
    if epok.dungeon.room_at(spot.position) == here and epok.dungeon.distance(here, epok.dungeon.spawn_room()) > 0 then
        self.here = epok.dungeon.neighbour(here, epok.dungeon.room_count() - 1)
    end
    if epok.dungeon.item(0).lock == epok.dungeon.lock(0).lock and epok.dungeon.item_count() > epok.dungeon.lock_count() then
        self.here = epok.to_int(epok.dungeon.room(epok.dungeon.lock(0).room).center.x)
    end
    if epok.dungeon.seed() == 0 then
        self.here = -1
    end
    if not epok.dungeon.room_explored(here) then
        epok.dungeon.explore_room(here)
    end
    if epok.dungeon.minimap_shown() and not epok.dungeon.full_map_shown() then
        epok.dungeon.show_full_map(true)
        epok.dungeon.show_minimap(true)
    end
    epok.dungeon.show_full_map(epok.input.held(0, 0))
    if self.here < -1 then
        epok.dungeon.explore_everything()
        epok.dungeon.forget_explored()
    end
end

return DungeonProbe
"#,
        )
        .unwrap();
        let mut identities = crate::lua_identity::read(&root).unwrap_or_default();
        identities
            .classes
            .insert(lua.into(), "5bd1c0d6-3c49-4b6e-9c55-2c7f3a8d1e64".into());
        identities.save(&root).unwrap();

        // The Blueprint: probe() returns the room after the spawn towards the goal.
        let mut asset = bp::BlueprintAsset::new(
            "DungeonProbeGraph".into(),
            crate::object_model::ACTOR_COMPONENT_ID.into(),
        );
        asset.id = "3f0c6a77-0f1e-4d43-9a4f-5a2b9e8c7d10".into();
        let node = |id: &str, kind: bp::NodeKind| bp::Node {
            id: id.into(),
            kind,
            inputs: Default::default(),
            outputs: Default::default(),
        };
        let op = |name: &str| bp::NodeKind::Operation {
            operation: match name {
                "spawn_room" => "b98ef46d-b84c-4629-83c9-46f76e4f17e6",
                "goal_room" => "b22991ec-2a6a-449c-9c09-5177f0bc0c7d",
                _ => "5d8c296c-def1-4e2c-97d6-8b08da38a060",
            }
            .into(),
        };
        let link = |node: &str| bp::Input::Link {
            node: node.into(),
            pin: "value".into(),
        };
        let mut entry = node("0a6f3c1e-5d4b-4f2a-8e9d-1b2c3d4e5f60", bp::NodeKind::Entry);
        entry.outputs.insert(
            "next".into(),
            vec!["2c8f5e30-7f6d-4b4c-8abf-3d4e5f607182".into()],
        );
        let mut next = node("1b7e4d2f-6e5c-4a3b-9fae-2c3d4e5f6071", op("next_room"));
        next.inputs.insert(
            "5d8c296c-def1-4e2c-97d6-8b08da38a060:parameter:0".into(),
            link("3d906f41-8a7e-4c5d-9bc0-4e5f60718293"),
        );
        next.inputs.insert(
            "5d8c296c-def1-4e2c-97d6-8b08da38a060:parameter:1".into(),
            link("4ea17052-9b8f-4d6e-8cd1-5f60718293a4"),
        );
        let mut ret = node("2c8f5e30-7f6d-4b4c-8abf-3d4e5f607182", bp::NodeKind::Return);
        ret.inputs
            .insert("value".into(), link("1b7e4d2f-6e5c-4a3b-9fae-2c3d4e5f6071"));
        asset.functions.push(bp::Graph {
            id: "8d6f2e1a-4c3b-4a59-9e7d-1f2a3b4c5d6e".into(),
            name: "probe".into(),
            override_id: None,
            timeline: None,
            parameters: Vec::new(),
            returns: crate::reflection_schema::Type::Int32,
            entry: "0a6f3c1e-5d4b-4f2a-8e9d-1b2c3d4e5f60".into(),
            nodes: vec![
                entry,
                node("3d906f41-8a7e-4c5d-9bc0-4e5f60718293", op("spawn_room")),
                node("4ea17052-9b8f-4d6e-8cd1-5f60718293a4", op("goal_room")),
                next,
                ret,
            ],
        });
        // reveal() opens the full map: a callable library node on the exec chain.
        let full_map = "52559c3a-438b-4823-b6c8-2ee940a062b4";
        let mut entry = node("5fb28163-ac90-4e7f-9de2-60718293a4b5", bp::NodeKind::Entry);
        entry.outputs.insert(
            "next".into(),
            vec!["60c39274-bda1-4f80-8ef3-718293a4b5c6".into()],
        );
        let mut show = node(
            "60c39274-bda1-4f80-8ef3-718293a4b5c6",
            bp::NodeKind::Operation {
                operation: full_map.into(),
            },
        );
        show.inputs.insert(
            format!("{full_map}:parameter:0"),
            bp::Input::Literal {
                value_type: crate::reflection_schema::Type::Bool,
                value: serde_json::json!(true),
            },
        );
        show.outputs.insert(
            "next".into(),
            vec!["71d4a385-ceb2-4091-9f04-8293a4b5c6d7".into()],
        );
        asset.functions.push(bp::Graph {
            id: "9e7f3f2b-5d4c-4b6a-8f8e-2a3b4c5d6e7f".into(),
            name: "reveal".into(),
            override_id: None,
            timeline: None,
            parameters: Vec::new(),
            returns: crate::reflection_schema::Type::Void,
            entry: "5fb28163-ac90-4e7f-9de2-60718293a4b5".into(),
            nodes: vec![
                entry,
                show,
                node("71d4a385-ceb2-4091-9f04-8293a4b5c6d7", bp::NodeKind::Return),
            ],
        });
        let path = root.join("assets/Blueprints/DungeonProbeGraph.epokbp");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        bp::create(&path, &asset).unwrap();

        let native = crate::scripts::native_catalog(&root).unwrap();
        crate::scripts::compile_lua(&root, &native)
            .unwrap_or_else(|e| panic!("Lua: {e}"))
            .expect("one Lua class");
        let compiled = crate::scripts::compile_blueprints(&root, &native)
            .unwrap_or_else(|e| panic!("Blueprint: {e}"))
            .expect("one Blueprint class");
        let text: String = compiled
            .artifacts
            .files
            .values()
            .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
            .collect();
        assert!(
            text.contains("epok::DungeonLibrary::next_room"),
            "the graph lowers to the library"
        );
        assert!(
            text.contains("epok::DungeonLibrary::show_full_map"),
            "the exec chain lowers to the library"
        );
        std::fs::remove_dir_all(parent).unwrap();
    }
}
