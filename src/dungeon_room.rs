//! Dungeon Rooms: rooms built by hand once, then placed by the Dungeon Generator wherever
//! a recipe's Handmade rooms step asks, turned in quarter turns and joined by its
//! corridors at their doors.
//!
//! A room is an editor-only asset holding a captured actor branch in the room's own
//! space (x from 0 to its width in cells, z from 0 to its depth, floor at y 0), its
//! footprint, its doors and where keys and items go. When a dungeon is generated, its
//! plain static geometry merges into the dungeon's mesh sections and its solid box
//! colliders into the dungeon's one compound collider, so a room of many props costs
//! quads rather than actors; lights and anything with behaviour stay actors.
use crate::{
    assets,
    dungeon::Side,
    scene::{Actor, Material, Scene},
    transform::Matrix,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

pub const FOLDER: &str = "assets/Dungeons/Rooms";
pub const VERSION: u32 = 1;
/// Footprint sides in cells.
pub const SIZE_LIMITS: (u16, u16) = (2, 16);

/// An opening on the footprint's edge: the floor cell inside and the side it faces.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Door {
    pub cell: [u16; 2],
    pub side: Side,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Room {
    pub version: u32,
    /// The grid cell size the room was built for, in world units.
    pub cell: f32,
    /// The footprint in grid cells: across (x) and deep (z).
    pub size: [u16; 2],
    /// Footprint cells that are not floor.
    pub closed: Vec<[u16; 2]>,
    pub doors: Vec<Door>,
    /// Floor cells where the recipe's keys and items go, in order of preference.
    pub spots: Vec<[u16; 2]>,
    /// The room's content in its own space.
    pub actors: Vec<Actor>,
}
impl Default for Room {
    fn default() -> Self {
        Self {
            version: VERSION,
            cell: 3.,
            size: [5, 5],
            closed: Vec::new(),
            doors: Vec::new(),
            spots: Vec::new(),
            actors: Vec::new(),
        }
    }
}

impl Room {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let room: Self = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        room.validate()?;
        Ok(room)
    }
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        serde_json::to_vec_pretty(self).map_err(|e| e.to_string())
    }
    fn inside(&self, cell: [u16; 2]) -> bool {
        cell[0] < self.size[0] && cell[1] < self.size[1]
    }
    pub fn floor(&self, cell: [u16; 2]) -> bool {
        self.inside(cell) && !self.closed.contains(&cell)
    }
    /// Whether a door on `side` of `cell` sits on the footprint's edge.
    pub fn on_edge(&self, cell: [u16; 2], side: Side) -> bool {
        match side {
            Side::East => cell[0] + 1 == self.size[0],
            Side::West => cell[0] == 0,
            Side::South => cell[1] + 1 == self.size[1],
            Side::North => cell[1] == 0,
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version > VERSION {
            return Err(format!(
                "Dungeon Room version {} needs a newer editor",
                self.version
            ));
        }
        let (lo, hi) = SIZE_LIMITS;
        if self.size.iter().any(|s| !(lo..=hi).contains(s)) {
            return Err(format!("A room is {lo}-{hi} cells across and deep"));
        }
        if !(0.5..=8.).contains(&self.cell) {
            return Err("A room's cell size is 0.5-8 units".into());
        }
        for cell in &self.closed {
            if !self.inside(*cell) {
                return Err(format!("Closed cell {cell:?} is outside the room"));
            }
        }
        if self.doors.is_empty() {
            return Err("Give the room at least one door on its edge".into());
        }
        for (i, door) in self.doors.iter().enumerate() {
            if !self.floor(door.cell) || !self.on_edge(door.cell, door.side) {
                return Err(format!(
                    "Door {} must be a floor cell on the room's {:?} edge",
                    i + 1,
                    door.side
                ));
            }
            if self.doors[..i].contains(door) {
                return Err(format!("Door {} is listed twice", i + 1));
            }
        }
        for spot in &self.spots {
            if !self.floor(*spot) {
                return Err(format!("Spot {spot:?} must be a floor cell of the room"));
            }
        }
        Ok(())
    }
    /// The footprint and openings the layout places.
    pub fn shape(&self) -> Shape {
        let [w, h] = self.size.map(i32::from);
        Shape {
            size: [w, h],
            floor: (0..h)
                .flat_map(|y| (0..w).map(move |x| [x as u16, y as u16]))
                .map(|p| self.floor(p))
                .collect(),
            doors: self
                .doors
                .iter()
                .map(|d| (d.cell.map(i32::from), d.side))
                .collect(),
            spots: self.spots.iter().map(|s| s.map(i32::from)).collect(),
        }
    }
}

/// A room's footprint and openings as the layout sees them, turned by quarter turns.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shape {
    pub size: [i32; 2],
    /// Row by row, whether each footprint cell is floor.
    pub floor: Vec<bool>,
    pub doors: Vec<([i32; 2], Side)>,
    pub spots: Vec<[i32; 2]>,
}
impl Shape {
    pub fn is_floor(&self, p: [i32; 2]) -> bool {
        p[0] >= 0
            && p[1] >= 0
            && p[0] < self.size[0]
            && p[1] < self.size[1]
            && self.floor[(p[1] * self.size[0] + p[0]) as usize]
    }
    /// The shape turned clockwise (seen from above, grid y growing south) `turns` times.
    pub fn turned(&self, turns: u8) -> Shape {
        let mut shape = self.clone();
        for _ in 0..turns % 4 {
            let [w, h] = shape.size;
            let turn = |p: [i32; 2]| [h - 1 - p[1], p[0]];
            let mut floor = vec![false; shape.floor.len()];
            for y in 0..h {
                for x in 0..w {
                    let q = turn([x, y]);
                    floor[(q[1] * h + q[0]) as usize] = shape.floor[(y * w + x) as usize];
                }
            }
            shape = Shape {
                size: [h, w],
                floor,
                doors: shape
                    .doors
                    .iter()
                    .map(|(c, s)| (turn(*c), s.next()))
                    .collect(),
                spots: shape.spots.iter().map(|c| turn(*c)).collect(),
            };
        }
        shape
    }
}

/// Where a room turned `turns` quarter turns lands: room space to the generator's space,
/// with the turned footprint's smallest corner at `corner` (x, floor height, z), scaled
/// from the cell size it was built for to `cell`.
pub fn placement(room: &Room, turns: u8, corner: [f32; 3], cell: f32) -> Matrix {
    let scale = cell / room.cell;
    let mut m = Matrix([
        [scale, 0., 0., 0.],
        [0., scale, 0., 0.],
        [0., 0., scale, 0.],
    ]);
    let (mut w, mut h) = (
        f32::from(room.size[0]) * cell,
        f32::from(room.size[1]) * cell,
    );
    for _ in 0..turns % 4 {
        // (x, z) -> (h - z, x): the grid's clockwise quarter turn.
        let quarter = Matrix([[0., 0., -1., h], [0., 1., 0., 0.], [1., 0., 0., 0.]]);
        m = quarter.compose(m);
        std::mem::swap(&mut w, &mut h);
    }
    let mut shift = Matrix::IDENTITY;
    for (row, value) in corner.into_iter().enumerate() {
        shift.0[row][3] = value;
    }
    shift.compose(m)
}

/// The rooms `ids` name, from the project's assets; ids that do not resolve are left out
/// and reported.
pub fn load(index: &assets::Index, ids: &[Uuid]) -> (BTreeMap<Uuid, Room>, Vec<String>) {
    let mut rooms = BTreeMap::new();
    let mut errors = Vec::new();
    for id in ids {
        match index
            .resolve(*id)
            .map_err(|e| e.to_string())
            .and_then(document)
        {
            Ok(room) => {
                rooms.insert(*id, room);
            }
            Err(error) => errors.push(format!("Dungeon Room {id}: {error}")),
        }
    }
    (rooms, errors)
}

// ---- storage ---------------------------------------------------------------------------------

pub fn fresh_path(root: &std::path::Path, name: &str) -> String {
    let stem: String = name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect();
    let stem = if stem.is_empty() {
        "DungeonRoom".into()
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
pub fn create(root: &std::path::Path, path: &str, room: &Room) -> Result<Uuid, String> {
    let id = Uuid::new_v4();
    let source = room.encode()?;
    let package = assets::Package {
        meta: assets::Metadata {
            version: 1,
            id,
            kind: assets::Kind::DungeonRoom,
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
pub fn document(record: &assets::Record) -> Result<Room, String> {
    if record.meta.kind != assets::Kind::DungeonRoom {
        return Err("Select a Dungeon Room asset".into());
    }
    Room::parse(&assets::Package::load(&record.path)?.source)
}
pub fn save(record: &assets::Record, room: &Room) -> Result<String, String> {
    let mut package = assets::Package::load(&record.path)?;
    if package.meta.kind != assets::Kind::DungeonRoom || package.meta.id != record.meta.id {
        return Err("The Dungeon Room was replaced; reopen it before editing".into());
    }
    package.source = room.encode()?;
    package.meta.source_hash = assets::hash(&package.source);
    let bytes = package.bytes()?;
    assets::atomic_write(&record.path, &bytes, Some(&record.revision))?;
    Ok(assets::hash(&bytes))
}
/// What a room's content looks like to the generator, for staleness.
pub fn digest(room: &Room) -> String {
    use sha2::{Digest, Sha256};
    let bytes = serde_json::to_vec(room).unwrap_or_default();
    Sha256::digest(bytes)[..8]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

// ---- capture ---------------------------------------------------------------------------------

/// Whether an actor marks a door when a room is captured: an empty named Door.
pub fn door_marker(actor: &Actor) -> bool {
    actor.kind == "Empty" && actor.name.to_lowercase().starts_with("door")
}

/// Captures the branch under `root` as a room: its descendants become the content, in
/// the root's space; empties named Door on the footprint's edge become its doors. The
/// footprint covers the content from the root's origin towards +x and +z.
pub fn capture(
    scene: &Scene,
    root: Uuid,
    cell: f32,
    index: &assets::Index,
) -> Result<Room, String> {
    let root_index = scene
        .actor_index(root)
        .ok_or("Select the room's root actor")?;
    let to_room = scene.world_matrix(root_index).inverse()?;
    let branch: Vec<usize> = scene
        .actor_branch(root)
        .into_iter()
        .filter(|id| *id != root)
        .filter_map(|id| scene.actor_index(id))
        .collect();
    if branch.is_empty() {
        return Err("Put the room's content under one actor and select it".into());
    }
    let local = |i: usize| to_room.compose(scene.world_matrix(i));
    // The content's extent on the floor plan.
    let mut hi = [0f32; 2];
    let mut lo = [f32::MAX; 2];
    let mut extend = |p: [f32; 3]| {
        lo[0] = lo[0].min(p[0]);
        lo[1] = lo[1].min(p[2]);
        hi[0] = hi[0].max(p[0]);
        hi[1] = hi[1].max(p[2]);
    };
    for &i in &branch {
        let actor = &scene.actors[i];
        let m = local(i);
        let mut corners = |a: [f32; 3], b: [f32; 3]| {
            for k in 0..8 {
                extend(m.point(std::array::from_fn(|axis| {
                    if k >> axis & 1 == 0 { a[axis] } else { b[axis] }
                })));
            }
        };
        if let Some(mesh) = &actor.editable_mesh
            && let Ok(record) = index.resolve(mesh.asset)
            && let Ok(document) = crate::mesh::document(record)
            && !document.vertices.is_empty()
        {
            let a = [0, 1, 2].map(|k| {
                document
                    .vertices
                    .iter()
                    .map(|v| v[k])
                    .fold(f32::MAX, f32::min)
            });
            let b = [0, 1, 2].map(|k| {
                document
                    .vertices
                    .iter()
                    .map(|v| v[k])
                    .fold(f32::MIN, f32::max)
            });
            corners(a, b);
        } else if actor.kind == "Mesh" {
            corners([-0.5; 3], [0.5; 3]);
        } else {
            extend(m.point([0.; 3]));
        }
    }
    if lo[0] < -0.01 || lo[1] < -0.01 {
        return Err(
            "Build the room on the root's +X and +Z side: its origin is the footprint's corner"
                .into(),
        );
    }
    let size =
        hi.map(|v| ((v / cell - 1e-3).ceil().max(1.) as u16).clamp(SIZE_LIMITS.0, SIZE_LIMITS.1));
    let mut room = Room {
        cell,
        size,
        ..Default::default()
    };
    let in_root: Vec<Uuid> = branch.iter().map(|i| scene.actors[*i].id).collect();
    for &i in &branch {
        let actor = &scene.actors[i];
        if door_marker(actor) {
            let p = local(i).point([0.; 3]);
            let cell_of = |v: f32, n: u16| ((v / cell).floor().max(0.) as u16).min(n - 1);
            let c = [cell_of(p[0], size[0]), cell_of(p[2], size[1])];
            // The nearest edge of the footprint.
            let edges = [
                (p[0], Side::West),
                (f32::from(size[0]) * cell - p[0], Side::East),
                (p[2], Side::North),
                (f32::from(size[1]) * cell - p[2], Side::South),
            ];
            let side = edges
                .iter()
                .min_by(|a, b| a.0.total_cmp(&b.0))
                .map(|e| e.1)
                .unwrap_or(Side::North);
            let door = Door { cell: c, side };
            if !room.doors.contains(&door) {
                room.doors.push(door);
            }
            continue;
        }
        let mut copy = actor.clone();
        // Children of the root hang from the room itself.
        if copy.logical_parent.is_some_and(|p| !in_root.contains(&p)) {
            copy.logical_parent = None;
        }
        if copy
            .attach
            .as_ref()
            .is_some_and(|a| !in_root.contains(&a.actor))
        {
            copy.attach = None;
            local(i).apply_trs(&mut copy)?;
        }
        room.actors.push(copy);
    }
    Ok(room)
}

// ---- baking ----------------------------------------------------------------------------------

/// Static geometry a room adds to the dungeon's mesh sections.
#[derive(Clone, Debug)]
pub enum Geometry {
    /// An EditableMesh, with its own materials.
    Mesh { asset: Uuid, transform: Matrix },
    /// A primitive cube with its actor's material.
    Cube {
        transform: Matrix,
        material: Material,
    },
}

/// One placed room, in the generator's space.
#[derive(Clone, Debug, Default)]
pub struct Baked {
    pub geometry: Vec<Geometry>,
    /// Solid boxes for the compound collider: `[min x, y, z, max x, y, z]`.
    pub boxes: Vec<[f32; 6]>,
    /// Lights and actors with behaviour, under the generator.
    pub actors: Vec<Actor>,
}

/// Whether an actor is plain static geometry or collision: an Actor3D with nothing but
/// its transform, a mesh and a solid collider. Those merge; everything else is copied.
pub fn is_static(actor: &Actor) -> bool {
    use crate::object_model as om;
    let plain_class = actor
        .class
        .class_id
        .as_deref()
        .is_none_or(|id| id == om::ACTOR3D_ID);
    let plain_components = actor.components.iter().all(|c| {
        matches!(
            c.class.class_id.as_deref(),
            Some(om::SCENE_COMPONENT3D_ID | om::MESH3D_COMPONENT_ID | om::COLLIDER3D_COMPONENT_ID)
        )
    });
    let solid = actor
        .collider
        .as_ref()
        .is_none_or(|c| c.enabled && !c.trigger && c.slope_rise == 0.);
    plain_class
        && plain_components
        && solid
        && matches!(actor.kind.as_str(), "Mesh" | "Empty")
        && actor.light.is_none()
        && actor.audio.is_none()
        && actor.skeletal_mesh.is_none()
        && actor.sprite.is_none()
        && actor.particle_emitter.is_none()
        && actor.particle_effect.is_none()
        && actor.timeline.is_none()
        && actor.blueprint_instance.is_none()
        && actor.active
}

/// Places `room` with `placement` (room space to the generator's space). Copied actors get
/// identities derived from `role` and hang from `parent`.
pub fn bake(room: &Room, placement: Matrix, role: &dyn Fn(Uuid) -> Uuid, parent: Uuid) -> Baked {
    let ids: BTreeMap<Uuid, usize> = room
        .actors
        .iter()
        .enumerate()
        .map(|(i, a)| (a.id, i))
        .collect();
    // An actor's room-space matrix, through its parents inside the room.
    fn matrix(room: &Room, ids: &BTreeMap<Uuid, usize>, index: usize, depth: usize) -> Matrix {
        let actor = &room.actors[index];
        let local = Matrix::trs(actor.position, actor.rotation, actor.scale);
        match actor.attach.as_ref().and_then(|a| ids.get(&a.actor)) {
            Some(&parent) if depth < 32 => matrix(room, ids, parent, depth + 1).compose(local),
            _ => local,
        }
    }
    let statics: Vec<bool> = room.actors.iter().map(is_static).collect();
    let mut baked = Baked::default();
    let mut map = crate::actor_document::IdentityMap::new();
    for actor in &room.actors {
        let mut synced = actor.clone();
        crate::actor_components::sync(&mut synced);
        map.insert(synced.id, role(synced.id));
        for component in &synced.components {
            map.insert(component.id, role(component.id));
        }
    }
    for (index, actor) in room.actors.iter().enumerate() {
        let m = placement.compose(matrix(room, &ids, index, 0));
        if statics[index] {
            if actor.kind == "Mesh" {
                match actor.editable_mesh.as_ref().map(|mesh| mesh.asset) {
                    Some(asset) => baked.geometry.push(Geometry::Mesh {
                        asset,
                        transform: m,
                    }),
                    None => baked.geometry.push(Geometry::Cube {
                        transform: m,
                        material: actor.material.clone(),
                    }),
                }
            }
            if let Some(collider) = &actor.collider {
                let parts: Vec<[f32; 6]> = if collider.boxes.is_empty() {
                    let (c, h) = (collider.center, collider.half_extents);
                    vec![[
                        c[0] - h[0],
                        c[1] - h[1],
                        c[2] - h[2],
                        c[0] + h[0],
                        c[1] + h[1],
                        c[2] + h[2],
                    ]]
                } else {
                    collider.boxes.clone()
                };
                for part in parts {
                    let mut lo = [f32::MAX; 3];
                    let mut hi = [f32::MIN; 3];
                    for k in 0..8 {
                        let p = m.point(std::array::from_fn(|axis| {
                            if k >> axis & 1 == 0 {
                                part[axis]
                            } else {
                                part[axis + 3]
                            }
                        }));
                        for axis in 0..3 {
                            lo[axis] = lo[axis].min(p[axis]);
                            hi[axis] = hi[axis].max(p[axis]);
                        }
                    }
                    baked.boxes.push([lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]]);
                }
            }
            continue;
        }
        let mut copy = actor.clone();
        crate::actor_document::remap_actor(&mut copy, &map);
        // Actors hanging from a merged one, or from the room, hang from the generator.
        let inside = actor
            .attach
            .as_ref()
            .and_then(|a| ids.get(&a.actor))
            .is_some_and(|p| !statics[*p]);
        if !inside {
            copy.logical_parent = Some(parent);
            copy.attach = Some(crate::actor_document::Attachment {
                actor: parent,
                component: None,
            });
            if m.apply_trs(&mut copy).is_err() {
                copy.position = m.point([0.; 3]);
            }
        }
        baked.actors.push(copy);
    }
    baked
}

#[cfg(test)]
mod tests {
    use super::*;

    fn room() -> Room {
        Room {
            size: [4, 3],
            closed: vec![[3, 2]],
            doors: vec![
                Door {
                    cell: [0, 1],
                    side: Side::West,
                },
                Door {
                    cell: [2, 0],
                    side: Side::North,
                },
            ],
            spots: vec![[1, 1]],
            ..Default::default()
        }
    }

    /// Turning a footprint four times gives it back; one turn moves doors, floor and
    /// spots as the placement moves the content.
    #[test]
    fn quarter_turns_move_cells_doors_and_content_together() {
        let room = room();
        room.validate().unwrap();
        let shape = room.shape();
        assert_eq!(shape.turned(4), shape);
        for turns in 0..4u8 {
            let turned = shape.turned(turns);
            let m = placement(&room, turns, [0.; 3], room.cell);
            for y in 0..shape.size[1] {
                for x in 0..shape.size[0] {
                    // The centre of each cell lands in the centre of its turned cell.
                    let c = m.point([
                        (x as f32 + 0.5) * room.cell,
                        0.,
                        (y as f32 + 0.5) * room.cell,
                    ]);
                    let q = [
                        (c[0] / room.cell).floor() as i32,
                        (c[2] / room.cell).floor() as i32,
                    ];
                    assert!(
                        q[0] >= 0 && q[1] >= 0 && q[0] < turned.size[0] && q[1] < turned.size[1]
                    );
                    assert_eq!(shape.is_floor([x, y]), turned.is_floor(q), "turns {turns}");
                }
            }
            // Doors stay on the turned footprint's edge, facing out.
            for (cell, side) in &turned.doors {
                let d = side.delta();
                let out = [cell[0] + d[0], cell[1] + d[1]];
                assert!(turned.is_floor(*cell));
                assert!(
                    out[0] < 0
                        || out[1] < 0
                        || out[0] >= turned.size[0]
                        || out[1] >= turned.size[1],
                    "turns {turns}: door {cell:?} {side:?} is inside"
                );
            }
        }
    }

    #[test]
    fn invalid_rooms_explain_themselves() {
        let mut room = room();
        room.doors.push(Door {
            cell: [1, 1],
            side: Side::East,
        });
        assert!(room.validate().unwrap_err().contains("edge"));
        let mut room = self::room();
        room.doors.clear();
        assert!(room.validate().unwrap_err().contains("at least one door"));
        let mut room = self::room();
        room.size = [1, 3];
        assert!(room.validate().is_err());
        let mut room = self::room();
        room.spots.push([3, 2]);
        assert!(room.validate().unwrap_err().contains("floor"));
    }
}
