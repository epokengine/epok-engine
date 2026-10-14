//! Bakes a generated layout into ordinary map content.
//!
//! A Dungeon Style decides what each spot becomes (see `dungeon_style`).
//!
//! 3D: walls are the solid cells around every floor, so each room and corridor is
//! enclosed by blocks. Blockout pieces and meshes are merged into one EditableMesh per
//! section; collision is one compound collider built from the layout; navigation,
//! spawn, goal, lights and actor pieces are separate actors. Without actor and light
//! pieces the output stays a handful of actors whatever the size of the dungeon.
//!
//! 2D: Terrain pieces paint floors and solid cells into one TileMapLayer, solved with
//! the TileSet's own terrain rules; Tile pieces fill decoration layers above it.
use crate::actor_document::Attachment;
use crate::assets;
use crate::dungeon::{self, Generator, Layout, Side, SpotKind};
use crate::dungeon_style::{self, Piece, Placement, Style};
use crate::mesh::{Document, Face, Group, Slot};
use crate::reflection_schema::Domain;
use crate::scene::{Actor, Material, Scene};
use crate::tileset::TileSet;
use crate::transform::Matrix;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

/// Resident geometry a scene may hold without streaming (`project.rs` refuses more).
pub const SCENE_QUADS: usize = 3500;
/// Faces one EditableMesh asset may hold.
const SECTION_FACES: usize = 3400;
/// Half the span a section's vertices may cover around its origin.
const SECTION_HALF_SPAN: f32 = 120.;
/// Navigation nodes the generated volume aims for, under the 512 budget.
const NAVIGATION_NODES: f32 = 420.;
pub const DUNGEON_FOLDER: &str = "assets/Dungeons";

/// What one generation produced, for the Inspector, the log and MCP replies.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Report {
    pub summary: String,
    pub actors: usize,
    pub sections: usize,
    /// Compiled quads of the generated geometry (surfaces split every 4 units).
    pub quads: usize,
    pub boxes: usize,
    /// Estimated navigation nodes; 0 without a navigation volume.
    pub navigation_nodes: usize,
    pub notes: Vec<String>,
}

pub fn is_2d(scene: &Scene, root: Uuid) -> bool {
    scene
        .actor_index(root)
        .is_some_and(|i| crate::actor_components::domain(&scene.actors[i]) == Domain::World2D)
}

// ---- grid helpers --------------------------------------------------------------------------

/// Cells just beyond a room door: the corridor cells that carry a lintel.
fn doorway_cells(layout: &Layout) -> BTreeSet<[i32; 2]> {
    dungeon::spots(layout)
        .into_iter()
        .filter(|s| s.kind == SpotKind::Door)
        .filter_map(|s| {
            let d = s.side?.delta();
            Some([s.cell[0] + d[0], s.cell[1] + d[1]])
        })
        .collect()
}

/// Greedy rectangles covering exactly the cells `inside` selects within `region`
/// (`[x0, y0, x1, y1)`), scanning rows then growing downwards.
fn rectangles(region: [i32; 4], inside: &dyn Fn([i32; 2]) -> bool) -> Vec<[i32; 4]> {
    let [x0, y0, x1, y1] = region;
    let width = (x1 - x0).max(0) as usize;
    let mut used = vec![false; width * (y1 - y0).max(0) as usize];
    let at = |x: i32, y: i32| (y - y0) as usize * width + (x - x0) as usize;
    let mut out = Vec::new();
    for y in y0..y1 {
        let mut x = x0;
        while x < x1 {
            if used[at(x, y)] || !inside([x, y]) {
                x += 1;
                continue;
            }
            let mut end = x + 1;
            while end < x1 && !used[at(end, y)] && inside([end, y]) {
                end += 1;
            }
            let mut bottom = y + 1;
            while bottom < y1 && (x..end).all(|i| !used[at(i, bottom)] && inside([i, bottom])) {
                bottom += 1;
            }
            for yy in y..bottom {
                for xx in x..end {
                    used[at(xx, yy)] = true;
                }
            }
            out.push([x, y, end, bottom]);
            x = end;
        }
    }
    out
}

/// Faces a surface of this size compiles into: the compiler splits anything over four
/// units.
fn compiled(width: f32, height: f32) -> usize {
    ((width / 4.).ceil().max(1.) * (height / 4.).ceil().max(1.)) as usize
}

// ---- 3D -------------------------------------------------------------------------------------

/// One mesh section: its origin relative to the generator root, and its geometry.
pub struct Section {
    pub origin: [f32; 3],
    pub document: Document,
    pub quads: usize,
}

/// Navigation bake volumes covering the floor. Each stays within the ±64 transform an
/// actor cooks to, and neighbours overlap so the bake joins them.
pub struct Navigation {
    pub spacing: f32,
    pub nodes: usize,
    /// Centre and size of each volume, relative to the generator root.
    pub volumes: Vec<([f32; 3], [f32; 3])>,
}
/// Volumes beyond this many cost more actors than navigation is worth here.
const NAVIGATION_VOLUMES: usize = 12;

/// An actor a style piece places, relative to the generator root.
#[derive(Clone, Debug)]
pub struct Spawn {
    pub node: Uuid,
    pub transform: Matrix,
    pub piece: Piece,
    /// The lock number at key and locked-door spots; 0 elsewhere.
    pub lock: u16,
    /// The floor level of the spot.
    pub level: u8,
}

pub struct Plan3d {
    pub sections: Vec<Section>,
    pub boxes: Vec<[f32; 6]>,
    pub spawn: [f32; 3],
    pub goal: [f32; 3],
    /// None when the floor is too large for the node budget.
    pub navigation: Option<Navigation>,
    pub quads: usize,
    pub spawns: Vec<Spawn>,
}

/// How far wall faces continue below the floor.
const WALL_SKIRT: f32 = 0.08;
/// How far merged wall tops reach past their cells.
const CAP_OVERLAP: f32 = 0.03;
const GROUPS: [&str; 5] = ["Floors", "Walls", "Doorways", "Pieces", "Ceilings"];
const GROUP_FLOORS: usize = 0;
const GROUP_WALLS: usize = 1;
const GROUP_DOORWAYS: usize = 2;
const GROUP_PIECES: usize = 3;
const GROUP_CEILINGS: usize = 4;

struct Builder {
    document: Document,
    vertices: HashMap<[u32; 3], u32>,
    slots: HashMap<String, usize>,
    owner: Uuid,
    seed: u128,
    origin: [f32; 3],
    quads: usize,
}
impl Builder {
    fn new(owner: Uuid, origin: [f32; 3]) -> Self {
        Self {
            document: Document {
                version: 1,
                vertices: Vec::new(),
                faces: Vec::new(),
                groups: GROUPS
                    .iter()
                    .map(|name| Group {
                        id: dungeon::derived_id(owner, &format!("group-{name}")),
                        name: (*name).into(),
                        parent: None,
                    })
                    .collect(),
                materials: Vec::new(),
            },
            vertices: HashMap::new(),
            slots: HashMap::new(),
            owner,
            seed: owner.as_u128(),
            origin,
            quads: 0,
        }
    }
    /// The slot for `material`, shared by every face with the same key.
    fn slot(&mut self, key: &str, name: &str, material: &Material) -> usize {
        if let Some(index) = self.slots.get(key) {
            return *index;
        }
        let index = self.document.materials.len();
        self.document.materials.push(Slot {
            id: dungeon::derived_id(self.owner, &format!("slot-{key}")),
            name: name.chars().take(120).collect(),
            material: material.clone(),
        });
        self.slots.insert(key.to_owned(), index);
        index
    }
    fn blockout(&mut self, spot: SpotKind, color: [f32; 3], texture: Option<Uuid>) -> usize {
        let key = format!("blockout-{color:?}-{texture:?}");
        let material = Material {
            color,
            texture,
            ..Material::default()
        };
        self.slot(&key, spot.label(), &material)
    }
    fn vertex(&mut self, p: [f32; 3]) -> u32 {
        let local: [f32; 3] = std::array::from_fn(|k| p[k] - self.origin[k]);
        let key = local.map(f32::to_bits);
        if let Some(index) = self.vertices.get(&key) {
            return *index;
        }
        let index = self.document.vertices.len() as u32;
        self.document.vertices.push(local);
        self.vertices.insert(key, index);
        index
    }
    /// A quad given counter-clockwise when seen from its front; the last two points
    /// may repeat for a triangle.
    fn face(
        &mut self,
        p: [[f32; 3]; 4],
        uv: [[f32; 2]; 4],
        colors: Option<crate::vertex_colors::Corners>,
        paint: (usize, usize),
    ) {
        let (group, slot) = paint;
        let vertices = p.map(|v| self.vertex(v));
        let n = self.document.faces.len() as u128 + 1;
        let id =
            Uuid::from_u128(self.seed ^ n.wrapping_mul(0x9e37_79b9_7f4a_7c15_f39c_c060_5ced_c835));
        self.document.faces.push(Face {
            id,
            vertices,
            group: self.document.groups[group].id,
            material: self.document.materials[slot].id,
            uv,
            vertex_colors: colors,
        });
        let length =
            |a: [f32; 3], b: [f32; 3]| (0..3).map(|k| (a[k] - b[k]).powi(2)).sum::<f32>().sqrt();
        self.quads += compiled(
            length(p[0], p[1]),
            length(p[0], p[3]).max(length(p[1], p[2])),
        );
    }
    /// A horizontal rectangle facing up (or down) at height `y`.
    fn horizontal(&mut self, x: [f32; 2], z: [f32; 2], y: f32, up: bool, paint: (usize, usize)) {
        self.horizontal_turned(x, z, y, up, false, paint);
    }
    /// A horizontal rectangle whose texture covers it once (the console clamps UVs),
    /// with U along X, or along Z when `turned`.
    fn horizontal_turned(
        &mut self,
        x: [f32; 2],
        z: [f32; 2],
        y: f32,
        up: bool,
        turned: bool,
        paint: (usize, usize),
    ) {
        let mut p = [
            [x[0], y, z[0]],
            [x[0], y, z[1]],
            [x[1], y, z[1]],
            [x[1], y, z[0]],
        ];
        if !up {
            p.reverse();
        }
        let fraction = |v: f32, r: [f32; 2]| (v - r[0]) / (r[1] - r[0]).max(1e-6);
        let uv = p.map(|v| {
            let (u, w) = (fraction(v[0], x), fraction(v[2], z));
            if turned { [w, u] } else { [u, w] }
        });
        self.face(p, uv, None, paint);
    }
    /// A vertical rectangle along an edge from `a` to `b` (x, z), between heights `y`,
    /// facing the side to the left of a→b.
    fn vertical(&mut self, a: [f32; 2], b: [f32; 2], y: [f32; 2], paint: (usize, usize)) {
        let p = [
            [a[0], y[0], a[1]],
            [b[0], y[0], b[1]],
            [b[0], y[1], b[1]],
            [a[0], y[1], a[1]],
        ];
        // The texture covers the face once: U across, V from the top down.
        let length = ((b[0] - a[0]).abs() + (b[1] - a[1]).abs()).max(1e-6);
        let along = |v: [f32; 3]| (v[0] - a[0]).abs() + (v[2] - a[1]).abs();
        let height = (y[1] - y[0]).abs().max(1e-6);
        let uv = p.map(|v| [along(v) / length, (y[1] - v[1]) / height]);
        self.face(p, uv, None, paint);
    }
    /// A vertical rectangle like `vertical`, whose edge at `a` has a vertex at every
    /// height in `at_a` and whose edge at `b` at every height in `at_b`: the faces of the
    /// walls beside it end there, and an edge stopping in the middle of a neighbour's
    /// edge shows as a crack on the console. The rectangle is filled with a ladder of
    /// quads and triangles between its two edges, carrying the texture as one face.
    fn vertical_matched(
        &mut self,
        a: [f32; 2],
        b: [f32; 2],
        y: [f32; 2],
        at_a: &[f32],
        at_b: &[f32],
        paint: (usize, usize),
    ) {
        let rail = |extra: &[f32]| {
            let mut heights: Vec<f32> = extra
                .iter()
                .copied()
                .filter(|v| *v > y[0] + 1e-4 && *v < y[1] - 1e-4)
                .collect();
            heights.push(y[0]);
            heights.push(y[1]);
            heights.sort_by(f32::total_cmp);
            heights.dedup_by(|p, q| (*p - *q).abs() < 1e-4);
            heights
        };
        let (left, right) = (rail(at_a), rail(at_b));
        let length = ((b[0] - a[0]).abs() + (b[1] - a[1]).abs()).max(1e-6);
        let along = |v: [f32; 3]| (v[0] - a[0]).abs() + (v[2] - a[1]).abs();
        let height = (y[1] - y[0]).abs().max(1e-6);
        let emit = |p: [[f32; 3]; 4], face: &mut Self| {
            let uv = p.map(|v| [along(v) / length, (y[1] - v[1]) / height]);
            face.face(p, uv, None, paint);
        };
        let at = |end: [f32; 2], h: f32| [end[0], h, end[1]];
        let (mut i, mut j) = (0, 0);
        while i + 1 < left.len() || j + 1 < right.len() {
            let next_left = left.get(i + 1).copied();
            let next_right = right.get(j + 1).copied();
            match (next_left, next_right) {
                (Some(l), Some(r)) if (l - r).abs() < 1e-4 => {
                    emit([at(a, left[i]), at(b, right[j]), at(b, r), at(a, l)], self);
                    i += 1;
                    j += 1;
                }
                (Some(l), Some(r)) if l < r => {
                    let p = [at(a, left[i]), at(b, right[j]), at(a, l), at(a, l)];
                    emit(p, self);
                    i += 1;
                }
                (_, Some(r)) => {
                    let p = [at(a, left[i]), at(b, right[j]), at(b, r), at(b, r)];
                    emit(p, self);
                    j += 1;
                }
                (Some(l), None) => {
                    let p = [at(a, left[i]), at(b, right[j]), at(a, l), at(a, l)];
                    emit(p, self);
                    i += 1;
                }
                (None, None) => break,
            }
        }
    }
    /// A vertical rectangle between `a` and `b` that looks towards `normal` (x, z).
    fn vertical_toward(
        &mut self,
        a: [f32; 2],
        b: [f32; 2],
        y: [f32; 2],
        normal: [f32; 2],
        paint: (usize, usize),
    ) {
        // `vertical` looks to the left of a->b: (-dz, dx) for a direction (dx, dz).
        let d = [b[0] - a[0], b[1] - a[1]];
        if -d[1] * normal[0] + d[0] * normal[1] >= 0. {
            self.vertical(a, b, y, paint);
        } else {
            self.vertical(b, a, y, paint);
        }
    }
    /// A flight of steps filling a cell and climbing one level towards `up` from `floor`:
    /// treads and the risers in front of them. The texture is laid from above, so it runs
    /// across the whole flight as it does across a floor.
    fn stairs(
        &mut self,
        frame: &Frame,
        cell: [i32; 2],
        up: Side,
        floor: f32,
        paint: (usize, usize),
    ) {
        let [x, y] = cell;
        let (x0, x1, z0, z1) = (frame.x(x), frame.x(x + 1), frame.z(y), frame.z(y + 1));
        let steps = crate::dungeon::STEPS_PER_LEVEL;
        let fraction = |v: f32, a: f32, b: f32| (v - a) / (b - a).max(1e-6);
        let toward = up.delta().map(|v| v as f32);
        for i in 0..steps {
            let (t0, t1) = (i as f32 / steps as f32, (i + 1) as f32 / steps as f32);
            let (low, high) = (
                floor + crate::dungeon::STEP_RISE * i as f32,
                floor + crate::dungeon::STEP_RISE * (i + 1) as f32,
            );
            // The step's footprint: from t0 to t1 of the way up the cell.
            let (xs, zs) = match up {
                Side::East => ([x0 + (x1 - x0) * t0, x0 + (x1 - x0) * t1], [z0, z1]),
                Side::West => ([x1 - (x1 - x0) * t1, x1 - (x1 - x0) * t0], [z0, z1]),
                Side::South => ([x0, x1], [z0 + (z1 - z0) * t0, z0 + (z1 - z0) * t1]),
                Side::North => ([x0, x1], [z1 - (z1 - z0) * t1, z1 - (z1 - z0) * t0]),
            };
            let tread = [
                [xs[0], high, zs[0]],
                [xs[0], high, zs[1]],
                [xs[1], high, zs[1]],
                [xs[1], high, zs[0]],
            ];
            let uv = tread.map(|v| [fraction(v[0], x0, x1), fraction(v[2], z0, z1)]);
            self.face(tread, uv, None, paint);
            // The riser stands on the step's low edge and faces down the flight.
            let (a, b) = match up {
                Side::East => ([xs[0], zs[0]], [xs[0], zs[1]]),
                Side::West => ([xs[1], zs[0]], [xs[1], zs[1]]),
                Side::South => ([xs[0], zs[0]], [xs[1], zs[0]]),
                Side::North => ([xs[0], zs[1]], [xs[1], zs[1]]),
            };
            self.vertical_toward(a, b, [low, high], [-toward[0], -toward[1]], paint);
        }
    }
    /// Every face of `source`, moved by `transform`, with its own materials.
    fn mesh(&mut self, source: &Document, asset: Uuid, transform: Matrix) {
        let m = transform.0;
        let determinant = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
            - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
        for face in &source.faces {
            let Some(material) = source.materials.iter().find(|s| s.id == face.material) else {
                continue;
            };
            let slot = self.slot(
                &format!("mesh-{asset}-{}", material.id),
                &material.name,
                &material.material,
            );
            let triangle = face.vertices[2] == face.vertices[3];
            // A mirroring transform reverses the winding; restore it.
            let order: [usize; 4] = match (determinant < 0., triangle) {
                (false, _) => [0, 1, 2, 3],
                (true, true) => [0, 2, 1, 1],
                (true, false) => [0, 3, 2, 1],
            };
            let points = source.points(face).map(|v| transform.point(v));
            self.face(
                order.map(|i| points[i]),
                order.map(|i| face.uv[i]),
                face.vertex_colors.map(|c| order.map(|i| c[i])),
                (GROUP_PIECES, slot),
            );
        }
    }
}

/// Converts grid positions to root-local world units, centred on the grid.
pub struct Frame {
    pub size: [i32; 2],
    pub cell: f32,
    /// The ceiling height: where Ceiling spots sit.
    pub height: f32,
}
impl Frame {
    pub fn x(&self, x: i32) -> f32 {
        (x as f32 - self.size[0] as f32 * 0.5) * self.cell
    }
    pub fn z(&self, y: i32) -> f32 {
        (y as f32 - self.size[1] as f32 * 0.5) * self.cell
    }
    pub fn center(&self, p: [i32; 2]) -> [f32; 3] {
        [
            self.x(p[0]) + self.cell * 0.5,
            0.,
            self.z(p[1]) + self.cell * 0.5,
        ]
    }
}

/// What every section of one generation shares.
struct Context<'a> {
    g: &'a Generator,
    layout: &'a Layout,
    frame: &'a Frame,
    placements: &'a [Placement],
    meshes: &'a HashMap<Uuid, Arc<Document>>,
}

/// Where a Column spot stands: the start of its cell side (the end with the smaller
/// coordinate), and the direction into the room.
pub fn column_base(frame: &Frame, cell: [i32; 2], side: Side) -> ([f32; 2], [f32; 2]) {
    let [x, y] = cell;
    let d = side.delta();
    let point = match side {
        Side::North => [frame.x(x), frame.z(y)],
        Side::South => [frame.x(x), frame.z(y + 1)],
        Side::West => [frame.x(x), frame.z(y)],
        Side::East => [frame.x(x + 1), frame.z(y)],
    };
    (point, [-d[0] as f32, -d[1] as f32])
}

/// A built-in column's footprint around its base: corner minimum and maximum (x, z).
fn column_box(frame: &Frame, cell: [i32; 2], side: Side) -> ([f32; 2], [f32; 2]) {
    let (point, into) = column_base(frame, cell, side);
    let half = (frame.cell * 0.12).min(0.35);
    let depth = (frame.cell * 0.12).min(0.4);
    let along = [into[1].abs(), into[0].abs()];
    let corners = [
        [point[0] - along[0] * half, point[1] - along[1] * half],
        [
            point[0] + along[0] * half + into[0] * depth,
            point[1] + along[1] * half + into[1] * depth,
        ],
    ];
    (
        [
            corners[0][0].min(corners[1][0]),
            corners[0][1].min(corners[1][1]),
        ],
        [
            corners[0][0].max(corners[1][0]),
            corners[0][1].max(corners[1][1]),
        ],
    )
}

/// The edge on `side` of `cell`, its endpoints ordered so a face along it looks into
/// the cell.
fn edge(frame: &Frame, cell: [i32; 2], side: Side) -> ([f32; 2], [f32; 2]) {
    let [x, y] = cell;
    match side {
        Side::North => ([frame.x(x), frame.z(y)], [frame.x(x + 1), frame.z(y)]),
        Side::South => (
            [frame.x(x + 1), frame.z(y + 1)],
            [frame.x(x), frame.z(y + 1)],
        ),
        Side::West => ([frame.x(x), frame.z(y + 1)], [frame.x(x), frame.z(y)]),
        Side::East => (
            [frame.x(x + 1), frame.z(y)],
            [frame.x(x + 1), frame.z(y + 1)],
        ),
    }
}

fn build_section(cx: &Context, region: [i32; 4], owner: Uuid) -> Section {
    let (layout, frame, g) = (cx.layout, cx.frame, cx.g);
    let c = frame.cell;
    let h = g.wall_height;
    let origin = [
        (frame.x(region[0]) + frame.x(region[2])) * 0.5,
        0.,
        (frame.z(region[1]) + frame.z(region[3])) * 0.5,
    ];
    let inside = |p: [i32; 2]| {
        p[0] >= region[0] && p[0] < region[2] && p[1] >= region[1] && p[1] < region[3]
    };
    let mut b = Builder::new(owner, origin);
    // Wall tops are merged per material and level once every solid cell is known.
    let mut tops: Vec<(usize, u8, BTreeSet<[i32; 2]>)> = Vec::new();
    for placement in cx.placements.iter().filter(|p| inside(p.spot.cell)) {
        let spot = &placement.spot;
        let [x, y] = spot.cell;
        // The floor under the spot; flights of stairs climb from it.
        let f = layout.floor(spot.cell);
        match &placement.visual.piece {
            Piece::Blockout { color, texture } => {
                let slot = b.blockout(spot.kind, *color, *texture);
                match (spot.kind, spot.side) {
                    // Floors and walls are one face per cell or cell side. Every vertex
                    // then sits on the grid lattice, so no edge ends in the middle of a
                    // neighbour's edge: on the PSX such T-junctions show one-pixel
                    // cracks. One face per cell also gives baked lighting a vertex per
                    // cell corner.
                    (SpotKind::Floor, _) => {
                        if let Some(up) = layout.stair(spot.cell) {
                            b.stairs(frame, spot.cell, up, f, (GROUP_FLOORS, slot));
                            continue;
                        }
                        // Corridor textures run along the corridor.
                        let east_west = spot.corridor
                            && (layout.open(spot.cell, Side::East)
                                || layout.open(spot.cell, Side::West))
                            && !(layout.open(spot.cell, Side::North)
                                || layout.open(spot.cell, Side::South));
                        b.horizontal_turned(
                            [frame.x(x), frame.x(x + 1)],
                            [frame.z(y), frame.z(y + 1)],
                            f,
                            true,
                            east_west,
                            (GROUP_FLOORS, slot),
                        )
                    }
                    (SpotKind::Ceiling, _) => {
                        let top = layout.top(spot.cell) + h;
                        b.horizontal(
                            [frame.x(x), frame.x(x + 1)],
                            [frame.z(y), frame.z(y + 1)],
                            top,
                            false,
                            (GROUP_CEILINGS, slot),
                        );
                        // Where a neighbour's ceiling is lower, the step between them
                        // faces this cell.
                        for side in Side::ALL {
                            if !layout.open(spot.cell, side) {
                                continue;
                            }
                            let d = side.delta();
                            let lower = layout.top([x + d[0], y + d[1]]) + h;
                            if lower < top - 1e-4 {
                                let (a, bb) = edge(frame, spot.cell, side);
                                b.vertical(a, bb, [lower, top], (GROUP_CEILINGS, slot));
                            }
                        }
                    }
                    (SpotKind::Column, Some(side)) => {
                        // A pilaster standing out of the wall: its front, two sides and,
                        // without a ceiling, its top. The face against the wall is never
                        // seen.
                        let (lo, hi) = column_box(frame, spot.cell, side);
                        let into = side.opposite().delta().map(|v| v as f32);
                        let paint = (GROUP_WALLS, slot);
                        let heights = [f - WALL_SKIRT, f + h];
                        for (a, bb, normal) in [
                            ([lo[0], hi[1]], [hi[0], hi[1]], [0., 1.]),
                            ([lo[0], lo[1]], [hi[0], lo[1]], [0., -1.]),
                            ([hi[0], lo[1]], [hi[0], hi[1]], [1., 0.]),
                            ([lo[0], lo[1]], [lo[0], hi[1]], [-1., 0.]),
                        ] {
                            if normal[0] * into[0] + normal[1] * into[1] >= -0.5 {
                                b.vertical_toward(a, bb, heights, normal, paint);
                            }
                        }
                        if !g.ceiling {
                            b.horizontal([lo[0], hi[0]], [lo[1], hi[1]], f + h, true, paint);
                        }
                    }
                    (SpotKind::Wall, Some(side)) => {
                        let (a, bb) = edge(frame, spot.cell, side);
                        // Walls reach a little below the floor, covering the seam, and up
                        // to the ceiling over the cell's highest point. Walls along the
                        // same line meet edge to edge: this one is cut where its
                        // neighbours start and end, so no edge stops in the middle of
                        // another, which the console draws as a crack.
                        let top = layout.top(spot.cell) + h;
                        // The wall beside each end, on the same line, and where it ends.
                        let beside = |end: [f32; 2], other: [f32; 2]| {
                            let step = [
                                (end[0] - other[0]).signum() as i32,
                                (end[1] - other[1]).signum() as i32,
                            ];
                            let q = [x + step[0], y + step[1]];
                            if layout.at(q).walkable()
                                && !layout.open(q, side)
                                && !layout.door(q, side)
                            {
                                vec![layout.floor(q) - WALL_SKIRT, layout.top(q) + h]
                            } else {
                                Vec::new()
                            }
                        };
                        let (at_a, at_b) = (beside(a, bb), beside(bb, a));
                        b.vertical_matched(
                            a,
                            bb,
                            [f - WALL_SKIRT, top],
                            &at_a,
                            &at_b,
                            (GROUP_WALLS, slot),
                        );
                    }
                    (SpotKind::Key | SpotKind::Item, _) => {
                        // A pedestal: tall and thin for keys, low and wide for items.
                        let (half, top) = if spot.kind == SpotKind::Key {
                            (c * 0.14, 0.7)
                        } else {
                            (c * 0.2, 0.45)
                        };
                        let centre = frame.center(spot.cell);
                        let (x0, x1) = (centre[0] - half, centre[0] + half);
                        let (z0, z1) = (centre[2] - half, centre[2] + half);
                        let paint = (GROUP_PIECES, slot);
                        let span = [f, f + top];
                        b.horizontal([x0, x1], [z0, z1], f + top, true, paint);
                        b.vertical([x0, z1], [x1, z1], span, paint);
                        b.vertical([x1, z0], [x0, z0], span, paint);
                        b.vertical([x1, z1], [x1, z0], span, paint);
                        b.vertical([x0, z0], [x0, z1], span, paint);
                    }
                    (SpotKind::Lock, Some(side)) => {
                        // A gate filling the door opening, seen from both sides.
                        let (a, bb) = edge(frame, spot.cell, side);
                        let top = f + g.door_height.min(h);
                        b.vertical(a, bb, [f, top], (GROUP_PIECES, slot));
                        b.vertical(bb, a, [f, top], (GROUP_PIECES, slot));
                    }
                    (SpotKind::OneWay, Some(side)) => {
                        // An arrow on the floor pointing from the door into the room.
                        let d = side.delta().map(|v| v as f32);
                        let across = [-d[1], d[0]];
                        let centre = frame.center(spot.cell);
                        let at = |forward: f32, side: f32| {
                            [
                                centre[0] + (d[0] * forward + across[0] * side) * c,
                                f + 0.03,
                                centre[2] + (d[1] * forward + across[1] * side) * c,
                            ]
                        };
                        let mut p = [at(0.42, -0.32), at(0.42, 0.32), at(-0.2, 0.)];
                        // Counter-clockwise seen from above.
                        let (u, v) = (
                            [p[1][0] - p[0][0], p[1][2] - p[0][2]],
                            [p[2][0] - p[0][0], p[2][2] - p[0][2]],
                        );
                        if u[1] * v[0] - u[0] * v[1] < 0. {
                            p.swap(1, 2);
                        }
                        // UVs within the arrow's own bounds; the console clamps them.
                        let lo = [0, 2].map(|k| p.iter().map(|q| q[k]).fold(f32::MAX, f32::min));
                        let hi = [0, 2].map(|k| p.iter().map(|q| q[k]).fold(f32::MIN, f32::max));
                        let uv = [p[0], p[1], p[2], p[2]].map(|q| {
                            [
                                (q[0] - lo[0]) / (hi[0] - lo[0]).max(1e-6),
                                (q[2] - lo[1]) / (hi[1] - lo[1]).max(1e-6),
                            ]
                        });
                        b.face([p[0], p[1], p[2], p[2]], uv, None, (GROUP_PIECES, slot));
                    }
                    (SpotKind::Solid, _) => {
                        match tops
                            .iter_mut()
                            .find(|(s, level, _)| *s == slot && *level == spot.level)
                        {
                            Some((_, _, cells)) => {
                                cells.insert(spot.cell);
                            }
                            None => tops.push((slot, spot.level, BTreeSet::from([spot.cell]))),
                        }
                    }
                    (SpotKind::Doorway, _) if g.door_height < h - 0.01 => {
                        let (x0, x1, z0, z1) =
                            (frame.x(x), frame.x(x + 1), frame.z(y), frame.z(y + 1));
                        b.horizontal(
                            [x0, x1],
                            [z0, z1],
                            f + g.door_height,
                            false,
                            (GROUP_DOORWAYS, slot),
                        );
                        for side in Side::ALL {
                            // Faces towards the open sides of the doorway cell.
                            if !layout.open(spot.cell, side) {
                                continue;
                            }
                            let (a, bb) = match side {
                                Side::North => ([x0, z0], [x1, z0]),
                                Side::South => ([x1, z1], [x0, z1]),
                                Side::West => ([x0, z1], [x0, z0]),
                                Side::East => ([x1, z0], [x1, z1]),
                            };
                            // Seen from the neighbour, so the winding is reversed.
                            b.vertical(bb, a, [f + g.door_height, f + h], (GROUP_DOORWAYS, slot));
                        }
                    }
                    (SpotKind::Corner, Some(side)) => {
                        // A square pillar in the corner, showing its two faces that
                        // look into the cell and its top.
                        let (da, db) = (side.delta(), side.next().delta());
                        let size = (c * 0.2).min(0.4);
                        let corner = [
                            frame.x(x) + c * 0.5 * (1 + da[0] + db[0]) as f32,
                            frame.z(y) + c * 0.5 * (1 + da[1] + db[1]) as f32,
                        ];
                        let sx = -(da[0] + db[0]) as f32;
                        let sz = -(da[1] + db[1]) as f32;
                        let (x0, x1) = (
                            corner[0].min(corner[0] + sx * size),
                            corner[0].max(corner[0] + sx * size),
                        );
                        let (z0, z1) = (
                            corner[1].min(corner[1] + sz * size),
                            corner[1].max(corner[1] + sz * size),
                        );
                        b.horizontal([x0, x1], [z0, z1], f + h, true, (GROUP_WALLS, slot));
                        let inner_x = if sx > 0. { x1 } else { x0 };
                        let inner_z = if sz > 0. { z1 } else { z0 };
                        // The face parallel to X, at inner_z, facing away from the
                        // corner (+Z when the pillar grows towards +Z).
                        if sz > 0. {
                            b.vertical(
                                [x0, inner_z],
                                [x1, inner_z],
                                [f - WALL_SKIRT, f + h],
                                (GROUP_WALLS, slot),
                            );
                        } else {
                            b.vertical(
                                [x1, inner_z],
                                [x0, inner_z],
                                [f - WALL_SKIRT, f + h],
                                (GROUP_WALLS, slot),
                            );
                        }
                        // The face parallel to Z, at inner_x, facing away from the corner.
                        if sx > 0. {
                            b.vertical(
                                [inner_x, z1],
                                [inner_x, z0],
                                [f - WALL_SKIRT, f + h],
                                (GROUP_WALLS, slot),
                            );
                        } else {
                            b.vertical(
                                [inner_x, z0],
                                [inner_x, z1],
                                [f - WALL_SKIRT, f + h],
                                (GROUP_WALLS, slot),
                            );
                        }
                    }
                    _ => {}
                }
            }
            Piece::Mesh { asset: Some(asset) } => {
                if let Some(source) = cx.meshes.get(asset) {
                    b.mesh(source, *asset, dungeon_style::transform(placement, frame));
                }
            }
            _ => {}
        }
    }
    // Wall tops merged into rectangles that overlap their neighbours slightly so the top
    // edges cannot crack either.
    for (slot, level, cells) in tops {
        for [x0, y0, x1, y1] in rectangles(region, &|p| cells.contains(&p)) {
            b.horizontal(
                [frame.x(x0) - CAP_OVERLAP, frame.x(x1) + CAP_OVERLAP],
                [frame.z(y0) - CAP_OVERLAP, frame.z(y1) + CAP_OVERLAP],
                f32::from(level) * crate::dungeon::LEVEL_HEIGHT + h,
                true,
                (GROUP_WALLS, slot),
            );
        }
    }
    let quads = b.quads;
    Section {
        origin,
        document: b.document,
        quads,
    }
}

/// Splits the grid until every section fits one EditableMesh.
fn sections(cx: &Context, owner: Uuid) -> Vec<Section> {
    let layout = cx.layout;
    let mut pending = vec![[0, 0, layout.size[0], layout.size[1]]];
    let mut out = Vec::new();
    while let Some(region) = pending.pop() {
        let section = build_section(
            cx,
            region,
            dungeon::derived_id(
                owner,
                &format!(
                    "section-{}-{}-{}-{}",
                    region[0], region[1], region[2], region[3]
                ),
            ),
        );
        let span = (region[2] - region[0]).max(region[3] - region[1]) as f32 * cx.frame.cell;
        let too_big = section.document.faces.len() > SECTION_FACES
            || section.document.vertices.len() > 16000
            || section.document.materials.len() > 64
            || span * 0.5 > SECTION_HALF_SPAN;
        if too_big && (region[2] - region[0] > 1 || region[3] - region[1] > 1) {
            let [x0, y0, x1, y1] = region;
            if x1 - x0 >= y1 - y0 {
                let mid = (x0 + x1) / 2;
                pending.push([mid, y0, x1, y1]);
                pending.push([x0, y0, mid, y1]);
            } else {
                let mid = (y0 + y1) / 2;
                pending.push([x0, mid, x1, y1]);
                pending.push([x0, y0, x1, mid]);
            }
            continue;
        }
        if !section.document.faces.is_empty() {
            out.push(section);
        }
    }
    out
}

pub fn plan_3d(
    g: &Generator,
    layout: &Layout,
    placements: &[Placement],
    meshes: &HashMap<Uuid, Arc<Document>>,
) -> Result<Plan3d, String> {
    g.validate()?;
    if g.half_extent(layout.size) > 127. {
        return Err(format!(
            "The dungeon spans {:.0} units; keep grid size × cell size within 254 units (reduce the cell size or the grid)",
            g.half_extent(layout.size) * 2.
        ));
    }
    let frame = Frame {
        size: layout.size,
        cell: g.cell_size,
        height: g.wall_height,
    };
    // Without a ceiling its spots place nothing.
    let kept: Vec<Placement>;
    let placements = if g.ceiling {
        placements
    } else {
        kept = placements
            .iter()
            .filter(|p| p.spot.kind != SpotKind::Ceiling)
            .cloned()
            .collect();
        &kept
    };
    let cx = Context {
        g,
        layout,
        frame: &frame,
        placements,
        meshes,
    };
    let sections = sections(&cx, g.id);
    let h = g.wall_height;
    let all = [0, 0, layout.size[0], layout.size[1]];
    let height = |level: u8| f32::from(level) * crate::dungeon::LEVEL_HEIGHT;
    let highest = layout.highest_level();
    // Solid everywhere but the air over each walkable cell: floors fill down to the
    // bottom, ceilings and the ring of walls up to the top, so no wall or ceiling top is
    // ever a floor for the character or for navigation, whatever the levels.
    let (bottom, top) = (-0.5, height(highest) + h + 0.5);
    let walkable = |p: [i32; 2]| layout.at(p).walkable();
    let mut boxes = Vec::new();
    for level in 0..=highest {
        for [x0, y0, x1, y1] in rectangles(all, &|p| {
            walkable(p) && layout.stair(p).is_none() && layout.level(p) == level
        }) {
            boxes.push([
                frame.x(x0),
                bottom,
                frame.z(y0),
                frame.x(x1),
                height(level),
                frame.z(y1),
            ]);
        }
    }
    // Every flight is a staircase of boxes, one per step, each up to its tread.
    for (&cell, &up) in &layout.stairs {
        let [x, y] = cell;
        let (x0, x1, z0, z1) = (frame.x(x), frame.x(x + 1), frame.z(y), frame.z(y + 1));
        let steps = crate::dungeon::STEPS_PER_LEVEL;
        for i in 0..steps {
            let (t0, t1) = (i as f32 / steps as f32, (i + 1) as f32 / steps as f32);
            let ([a, b], [c, d]) = match up {
                Side::East => ([x0 + (x1 - x0) * t0, x0 + (x1 - x0) * t1], [z0, z1]),
                Side::West => ([x1 - (x1 - x0) * t1, x1 - (x1 - x0) * t0], [z0, z1]),
                Side::South => ([x0, x1], [z0 + (z1 - z0) * t0, z0 + (z1 - z0) * t1]),
                Side::North => ([x0, x1], [z1 - (z1 - z0) * t1, z1 - (z1 - z0) * t0]),
            };
            let tread = layout.floor(cell) + crate::dungeon::STEP_RISE * (i + 1) as f32;
            boxes.push([a, bottom, c, b, tread, d]);
        }
    }
    for [x0, y0, x1, y1] in rectangles(all, &|p| layout.solid(p)) {
        boxes.push([
            frame.x(x0),
            bottom,
            frame.z(y0),
            frame.x(x1),
            top,
            frame.z(y1),
        ]);
    }
    if g.ceiling {
        for level in 0..=highest {
            for [x0, y0, x1, y1] in
                rectangles(all, &|p| walkable(p) && layout.top_level(p) == level)
            {
                boxes.push([
                    frame.x(x0),
                    height(level) + h,
                    frame.z(y0),
                    frame.x(x1),
                    top,
                    frame.z(y1),
                ]);
            }
        }
    }
    // Built-in columns stand out of the wall, so the player walks around them.
    for p in placements {
        if let (SpotKind::Column, Some(side), Piece::Blockout { .. }) =
            (p.spot.kind, p.spot.side, &p.visual.piece)
        {
            let (lo, hi) = column_box(&frame, p.spot.cell, side);
            let f = layout.floor(p.spot.cell);
            boxes.push([lo[0], f, lo[1], hi[0], f + h, hi[1]]);
        }
    }
    if g.door_height < h - 0.01 {
        let doorways = doorway_cells(layout);
        for level in 0..=highest {
            for [x0, y0, x1, y1] in
                rectangles(all, &|p| doorways.contains(&p) && layout.level(p) == level)
            {
                boxes.push([
                    frame.x(x0),
                    height(level) + g.door_height,
                    frame.z(y0),
                    frame.x(x1),
                    height(level) + h,
                    frame.z(y1),
                ]);
            }
        }
    }
    if boxes.len() > crate::collision::MAX_COMPOUND_BOXES {
        return Err(format!(
            "The dungeon needs {} collision boxes, above the {} one collider holds; use fewer or larger rooms",
            boxes.len(),
            crate::collision::MAX_COMPOUND_BOXES
        ));
    }
    let navigation = if g.navigation {
        navigation(g, layout, &frame)
    } else {
        None
    };
    let quads = sections.iter().map(|s| s.quads).sum();
    let spawns = placements
        .iter()
        .filter(|p| {
            matches!(
                p.visual.piece,
                Piece::Actor { class: Some(_) }
                    | Piece::Light { .. }
                    | Piece::Key { .. }
                    | Piece::Door { .. }
            )
        })
        .map(|p| Spawn {
            node: p.node,
            transform: dungeon_style::transform(p, &frame),
            piece: p.visual.piece.clone(),
            lock: lock_of(&p.spot),
            level: p.spot.level,
        })
        .collect();
    let on_floor = |p: [i32; 2]| {
        let mut centre = frame.center(p);
        centre[1] = layout.floor(p);
        centre
    };
    Ok(Plan3d {
        sections,
        boxes,
        spawn: on_floor(layout.rooms[layout.spawn].center()),
        goal: on_floor(layout.rooms[layout.goal].center()),
        navigation,
        quads,
        spawns,
    })
}

fn navigation(g: &Generator, layout: &Layout, frame: &Frame) -> Option<Navigation> {
    let c = frame.cell;
    let floor = layout.walkable_cells() as f32 * c * c;
    let spacing = ((floor / NAVIGATION_NODES).sqrt().max(0.5) * 8.).ceil() / 8.;
    if spacing > 4. {
        return None;
    }
    let walkable: Vec<[i32; 2]> = (0..layout.size[1])
        .flat_map(|y| (0..layout.size[0]).map(move |x| [x, y]))
        .filter(|p| layout.at(*p).walkable())
        .collect();
    let lo = [0, 1].map(|k| walkable.iter().map(|p| p[k]).min().unwrap_or(0));
    let hi = [0, 1].map(|k| walkable.iter().map(|p| p[k]).max().unwrap_or(0) + 1);
    // Samples within the agent radius of a volume edge are skipped, so neighbours
    // overlap by more than a spacing.
    let overlap = spacing + 0.25;
    let per_tile = (((60. - 2. * overlap) / c).floor() as i32).max(1);
    let highest = f32::from(layout.highest_level()) * crate::dungeon::LEVEL_HEIGHT;
    let (bottom, top) = (-1., highest + g.wall_height - 0.5);
    let mut volumes = Vec::new();
    let mut y = lo[1];
    while y < hi[1] {
        let y1 = (y + per_tile).min(hi[1]);
        let mut x = lo[0];
        while x < hi[0] {
            let x1 = (x + per_tile).min(hi[0]);
            if walkable
                .iter()
                .any(|p| p[0] >= x && p[0] < x1 && p[1] >= y && p[1] < y1)
            {
                let (x0w, x1w) = (frame.x(x) - overlap, frame.x(x1) + overlap);
                let (z0w, z1w) = (frame.z(y) - overlap, frame.z(y1) + overlap);
                volumes.push((
                    [(x0w + x1w) * 0.5, (bottom + top) * 0.5, (z0w + z1w) * 0.5],
                    [x1w - x0w, top - bottom, z1w - z0w],
                ));
            }
            x = x1;
        }
        y = y1;
    }
    (volumes.len() <= NAVIGATION_VOLUMES).then(|| Navigation {
        spacing,
        nodes: (floor / (spacing * spacing)).round() as usize,
        volumes,
    })
}

// ---- 2D -------------------------------------------------------------------------------------

pub const TILES_PATH: &str = "assets/Dungeons/DungeonTiles.epokasset";
const TILES_IMAGE: &str = "assets/Dungeons/DungeonTiles.png";
const TILES_TEXTURE: &str = "assets/Dungeons/DungeonTilesTexture.epokasset";
pub const FLOOR_TERRAIN: &str = "Floor";
pub const WALL_TERRAIN: &str = "Wall";

/// The terrain set holding every terrain name in `names`, and their indices in it.
pub fn terrain_indices(set: &TileSet, names: &[&str]) -> Option<(usize, Vec<u8>)> {
    set.terrain_sets.iter().enumerate().find_map(|(index, s)| {
        let found: Option<Vec<u8>> = names
            .iter()
            .map(|name| {
                s.terrains
                    .iter()
                    .position(|t| t.name.eq_ignore_ascii_case(name))
                    .map(|t| t as u8)
            })
            .collect();
        Some((index, found?))
    })
}

/// An actor a 2D style piece places, in pixels relative to the generator root.
#[derive(Clone, Debug)]
pub struct Spawn2d {
    pub node: Uuid,
    pub position: [f32; 2],
    /// An Actor piece (with a class), a Key or a Door.
    pub piece: Piece,
    /// The lock number at key and locked-door spots; 0 elsewhere.
    pub lock: u16,
    /// The door side of locked-door spots.
    pub side: Option<Side>,
}

fn lock_of(spot: &dungeon::Spot) -> u16 {
    if matches!(spot.kind, SpotKind::Key | SpotKind::Lock) {
        spot.tag
    } else {
        0
    }
}

/// Tile flags turning a marker drawn pointing south so it points from the door on
/// `side` into the room.
pub fn marker_flags(side: Side) -> u8 {
    use crate::tileset::{FLIP_V, TRANSPOSE};
    match side {
        Side::North => 0,
        Side::South => FLIP_V,
        Side::West => TRANSPOSE,
        Side::East => TRANSPOSE | FLIP_V,
    }
}

pub struct Plan2d {
    /// The TileSet the floor and solid cells are painted with, if any terrain is used.
    pub tile_set: Option<Uuid>,
    /// Pixels per grid cell.
    pub tile: [f32; 2],
    pub cells: crate::tilemap::Cells,
    /// Decoration layers, one per TileSet.
    pub decorations: Vec<(Uuid, crate::tilemap::Cells)>,
    pub spawn: [f32; 2],
    pub goal: [f32; 2],
    pub spawns: Vec<Spawn2d>,
}

/// Decoration layers one 2D dungeon may add.
const DECORATION_LAYERS: usize = 4;

pub fn plan_2d(
    layout: &Layout,
    placements: &[Placement],
    default_set: Uuid,
    sets: &BTreeMap<Uuid, Arc<TileSet>>,
) -> Result<Plan2d, String> {
    let offset = [layout.size[0] / 2, layout.size[1] / 2];
    let shift = |p: [i32; 2]| [p[0] - offset[0], p[1] - offset[1]];
    // Terrains: every Terrain piece paints with one TileSet.
    let mut terrain_set = None;
    let mut names: Vec<String> = Vec::new();
    let mut assigned: BTreeMap<[i32; 2], String> = BTreeMap::new();
    let mut defaults: BTreeMap<SpotKind, String> = BTreeMap::new();
    for p in placements {
        if let Piece::Terrain { tile_set, terrain } = &p.visual.piece {
            let set = tile_set.unwrap_or(default_set);
            if terrain_set.is_some_and(|t| t != set) {
                return Err(
                    "All Terrain pieces of a style must paint with the same TileSet".into(),
                );
            }
            terrain_set = Some(set);
            if !names.contains(terrain) {
                names.push(terrain.clone());
            }
            assigned
                .entry(p.spot.cell)
                .or_insert_with(|| terrain.clone());
            defaults
                .entry(p.spot.kind)
                .or_insert_with(|| terrain.clone());
        }
    }
    let mut cells = crate::tilemap::Cells::new();
    if let Some(set_id) = terrain_set {
        let set = sets
            .get(&set_id)
            .ok_or("A Terrain piece's TileSet is missing from the project")?;
        let refs: Vec<&str> = names.iter().map(String::as_str).collect();
        let (index, indices) = terrain_indices(set, &refs).ok_or_else(|| {
            format!(
                "The TileSet has no terrain set holding {}",
                names.join(", ")
            )
        })?;
        let number = |name: &str| indices[names.iter().position(|n| n == name).unwrap_or(0)];
        let floor = defaults.get(&SpotKind::Floor).map(|n| number(n));
        let solid = defaults.get(&SpotKind::Solid).map(|n| number(n));
        let numbered: BTreeMap<[i32; 2], u8> = assigned
            .iter()
            .map(|(p, name)| (shift(*p), number(name)))
            .collect();
        // Unpainted neighbours read as their kind's terrain, and everything beyond the
        // dungeon as solid, so outer walls show no edge towards the void.
        let terrain_of = |p: [i32; 2]| {
            numbered.get(&p).copied().or_else(|| {
                if layout.at([p[0] + offset[0], p[1] + offset[1]]).walkable() {
                    floor
                } else {
                    solid
                }
            })
        };
        let painted: BTreeSet<[i32; 2]> = numbered.keys().copied().collect();
        cells = crate::tileset::solve(set, index, &painted, &terrain_of)
            .into_iter()
            .filter(|(p, _)| painted.contains(p))
            .map(|(p, atlas)| (p, crate::tilemap::Cell::new(atlas)))
            .collect();
    }
    // Decorations: one layer per TileSet, first tile per cell.
    let mut decorations: Vec<(Uuid, crate::tilemap::Cells)> = Vec::new();
    for p in placements {
        if let Piece::Tile { tile_set, atlas } = &p.visual.piece {
            let set = tile_set.or(terrain_set).unwrap_or(default_set);
            let index = match decorations.iter().position(|(s, _)| *s == set) {
                Some(index) => index,
                None if decorations.len() < DECORATION_LAYERS => {
                    decorations.push((set, crate::tilemap::Cells::new()));
                    decorations.len() - 1
                }
                None => {
                    return Err(format!(
                        "Tile pieces use more than {DECORATION_LAYERS} TileSets"
                    ));
                }
            };
            let mut cell = crate::tilemap::Cell::new(*atlas);
            if let (SpotKind::Lock | SpotKind::OneWay, Some(side)) = (p.spot.kind, p.spot.side) {
                cell.flags = marker_flags(side);
            }
            decorations[index]
                .1
                .entry(shift(dungeon_style::tile_cell(&p.spot)))
                .or_insert(cell);
        }
    }
    let tile = sets
        .get(&terrain_set.unwrap_or(default_set))
        .map_or([16., 16.], |s| s.tile_size.map(f32::from));
    let pixel = |cell: [i32; 2], offset_px: [f32; 3]| {
        let p = shift(cell);
        [
            (p[0] as f32 + 0.5) * tile[0] + offset_px[0],
            (p[1] as f32 + 0.5) * tile[1] + offset_px[1],
        ]
    };
    let spawns = placements
        .iter()
        .filter(|p| {
            matches!(
                p.visual.piece,
                Piece::Actor { class: Some(_) } | Piece::Key { .. } | Piece::Door { .. }
            )
        })
        .map(|p| Spawn2d {
            node: p.node,
            position: pixel(p.spot.cell, p.visual.offset),
            piece: p.visual.piece.clone(),
            lock: lock_of(&p.spot),
            side: p.spot.side,
        })
        .collect();
    Ok(Plan2d {
        tile_set: terrain_set,
        tile,
        cells,
        decorations,
        spawn: pixel(layout.rooms[layout.spawn].center(), [0.; 3]),
        goal: pixel(layout.rooms[layout.goal].center(), [0.; 3]),
        spawns,
    })
}

/// The project's built-in dungeon TileSet, created on first use.
pub fn builtin_tiles(root: &Path) -> Result<(Uuid, crate::tileset::TileSet), String> {
    let index = assets::scan(root, &mut Default::default());
    let existing = index
        .assets
        .values()
        .flatten()
        .find(|r| {
            r.meta.kind == assets::Kind::TileSet
                && r.path == assets::inside(root, TILES_PATH).unwrap_or_default()
        })
        .cloned();
    let current = |set: &crate::tileset::TileSet| {
        dungeon_style::MARKER_TILES
            .iter()
            .all(|(_, atlas)| set.tile(*atlas).is_some())
    };
    if let Some(record) = &existing {
        let set = crate::tileset::document(record)?;
        if current(&set) {
            return Ok((record.meta.id, set));
        }
    }
    // First use, or tiles from before the recipe markers: draw them (again).
    crate::project::write_changed(&assets::inside(root, TILES_IMAGE)?, &art::tiles()?)?;
    let texture_path = assets::inside(root, TILES_TEXTURE)?;
    let texture_record = index
        .assets
        .values()
        .flatten()
        .find(|r| r.meta.kind == assets::Kind::Texture && r.path == texture_path)
        .cloned();
    let texture = assets::commit(crate::texture::prepare(
        root,
        TILES_IMAGE,
        TILES_TEXTURE,
        texture_record.as_ref(),
        false,
    )?)?;
    let set = art::tileset(texture);
    let id = match existing {
        Some(record) => {
            crate::tileset::save(&record, &set)?;
            record.meta.id
        }
        None => crate::tileset::create(root, TILES_PATH, &set)?,
    };
    Ok((id, set))
}

/// The built-in 3D surface textures, drawn by `tools/build_dungeon_textures.py`. Each
/// keeps one asset identity in every project, so the built-in style can name it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuiltinTexture {
    Floor,
    FloorGrid,
    FloorHazard,
    Corridor,
    Wall,
    WallRibbed,
    WallPipes,
    Column,
    Ceiling,
    Doorway,
}
impl BuiltinTexture {
    pub const ALL: [BuiltinTexture; 10] = [
        BuiltinTexture::Floor,
        BuiltinTexture::FloorGrid,
        BuiltinTexture::FloorHazard,
        BuiltinTexture::Corridor,
        BuiltinTexture::Wall,
        BuiltinTexture::WallRibbed,
        BuiltinTexture::WallPipes,
        BuiltinTexture::Column,
        BuiltinTexture::Ceiling,
        BuiltinTexture::Doorway,
    ];
    pub fn id(self) -> Uuid {
        Uuid::from_u128(match self {
            BuiltinTexture::Floor => 0x757152f8_9008_4e17_a9d7_01ee0ed85df7,
            BuiltinTexture::FloorGrid => 0x80863613_66d3_4ec2_ae6e_348097a6dda6,
            BuiltinTexture::FloorHazard => 0xababbe3a_73c5_453d_9054_d9a3da2a8237,
            BuiltinTexture::WallRibbed => 0x2d62e77e_5933_41d3_af51_cc41c1712698,
            BuiltinTexture::WallPipes => 0x7e173f83_6004_4b47_be89_823c107b67c0,
            BuiltinTexture::Column => 0xe3537af1_ed51_4840_965a_7bed9233b6fe,
            BuiltinTexture::Corridor => 0x39bfb74b_b144_43bc_83cb_f5a55beae6d2,
            BuiltinTexture::Wall => 0x370146fd_21ac_4c75_b903_abc81f236612,
            BuiltinTexture::Ceiling => 0x77fed04a_eadb_4a26_b11f_f3d36e5244bf,
            BuiltinTexture::Doorway => 0x0ceb1bf7_23fe_44e4_8029_0d6ce6b75313,
        })
    }
    fn name(self) -> &'static str {
        match self {
            BuiltinTexture::Floor => "DungeonFloor",
            BuiltinTexture::FloorGrid => "DungeonFloorGrid",
            BuiltinTexture::FloorHazard => "DungeonFloorHazard",
            BuiltinTexture::WallRibbed => "DungeonWallRibbed",
            BuiltinTexture::WallPipes => "DungeonWallPipes",
            BuiltinTexture::Column => "DungeonColumn",
            BuiltinTexture::Corridor => "DungeonCorridor",
            BuiltinTexture::Wall => "DungeonWall",
            BuiltinTexture::Ceiling => "DungeonCeiling",
            BuiltinTexture::Doorway => "DungeonDoorway",
        }
    }
    fn png(self) -> &'static [u8] {
        match self {
            BuiltinTexture::Floor => include_bytes!("../resources/dungeon/floor.png"),
            BuiltinTexture::FloorGrid => include_bytes!("../resources/dungeon/floor_grid.png"),
            BuiltinTexture::FloorHazard => include_bytes!("../resources/dungeon/floor_hazard.png"),
            BuiltinTexture::WallRibbed => include_bytes!("../resources/dungeon/wall_ribbed.png"),
            BuiltinTexture::WallPipes => include_bytes!("../resources/dungeon/wall_pipes.png"),
            BuiltinTexture::Column => include_bytes!("../resources/dungeon/column.png"),
            BuiltinTexture::Corridor => include_bytes!("../resources/dungeon/corridor.png"),
            BuiltinTexture::Wall => include_bytes!("../resources/dungeon/wall.png"),
            BuiltinTexture::Ceiling => include_bytes!("../resources/dungeon/ceiling.png"),
            BuiltinTexture::Doorway => include_bytes!("../resources/dungeon/doorway.png"),
        }
    }
}
const TEXTURES_FOLDER: &str = "assets/Dungeons/Textures";

/// Imports the built-in textures `style` names and the project lacks, under their
/// fixed identities, so every seed finds them. Returns whether it imported any.
pub fn builtin_textures(root: &Path, style: &Style) -> Result<bool, String> {
    let named = style.references();
    let wanted: Vec<BuiltinTexture> = BuiltinTexture::ALL
        .into_iter()
        .filter(|t| named.contains(&t.id()))
        .collect();
    if wanted.is_empty() {
        return Ok(false);
    }
    let index = assets::scan(root, &mut Default::default());
    let mut imported = false;
    for texture in wanted {
        // Present and current: nothing to do. Older built-in pictures are replaced in
        // place, keeping the identity every style and mesh refers to.
        let existing = index.resolve(texture.id()).ok().cloned();
        if existing
            .as_ref()
            .is_some_and(|r| r.meta.source_hash == assets::hash(texture.png()))
        {
            continue;
        }
        let image = format!("{TEXTURES_FOLDER}/{}.png", texture.name());
        let destination = match &existing {
            Some(record) => assets::path_string(root, &record.path),
            None => format!("{TEXTURES_FOLDER}/{}.epokasset", texture.name()),
        };
        crate::project::write_changed(&assets::inside(root, &image)?, texture.png())?;
        let mut candidate =
            crate::texture::prepare(root, &image, &destination, existing.as_ref(), false)?;
        candidate.package.meta.id = texture.id();
        assets::commit(candidate)?;
        imported = true;
    }
    Ok(imported)
}

/// Points a new 2D style's marker tiles at the project's built-in dungeon tiles,
/// creating them if needed.
pub fn pin_markers(root: &Path, style: &mut Style) -> Result<(), String> {
    let unpinned = style.two_d
        && style.nodes.iter().any(|n| {
            matches!(
                &n.kind,
                dungeon_style::NodeKind::Piece {
                    visual: dungeon_style::Visual {
                        piece: Piece::Tile { tile_set: None, .. },
                        ..
                    },
                    ..
                }
            )
        });
    if unpinned {
        let (tiles, _) = builtin_tiles(root)?;
        dungeon_style::pin_markers(style, tiles);
    }
    Ok(())
}

// ---- styles ---------------------------------------------------------------------------------

/// The style a generator uses and a digest of it, so a changed style marks the
/// output stale.
pub fn style_of(
    index: &assets::Index,
    g: &Generator,
    two_d: bool,
) -> Result<(Style, String), String> {
    let style = match g.style {
        Some(id) => {
            let record = index.resolve(id)?;
            let style = dungeon_style::document(record)?;
            if style.two_d != two_d {
                return Err(format!(
                    "The Dungeon Style is for {} dungeons; choose a {} style",
                    if style.two_d { "2D" } else { "3D" },
                    if two_d { "2D" } else { "3D" }
                ));
            }
            style
        }
        None => {
            let mut style = dungeon_style::builtin(two_d, None);
            // Marker tiles come from the built-in dungeon tiles, even when the
            // generator paints its floors and walls with another TileSet.
            if two_d && let Some(set) = g.tile_set {
                let tiles = index
                    .usable()
                    .find(|r| r.meta.kind == assets::Kind::TileSet && r.path.ends_with(TILES_PATH))
                    .map(|r| r.meta.id);
                match tiles {
                    Some(tiles) if tiles == set => {}
                    Some(tiles) => dungeon_style::pin_markers(&mut style, tiles),
                    None => dungeon_style::strip_markers(&mut style),
                }
            }
            style
        }
    };
    let digest = dungeon_style::digest(&style);
    Ok((style, digest))
}

/// The EditableMesh documents Mesh pieces merge in.
pub fn piece_meshes(
    index: &assets::Index,
    placements: &[Placement],
) -> Result<HashMap<Uuid, Arc<Document>>, String> {
    let mut meshes = HashMap::new();
    for p in placements {
        if let Piece::Mesh { asset: Some(asset) } = p.visual.piece
            && !meshes.contains_key(&asset)
        {
            let record = index
                .resolve(asset)
                .map_err(|e| format!("A Mesh piece's asset is missing: {e}"))?;
            meshes.insert(asset, Arc::new(crate::mesh::document(record)?));
        }
    }
    Ok(meshes)
}

/// The class model and registry that Actor pieces are placed with.
pub struct Classes<'a> {
    pub registry: &'a crate::blueprint::Registry,
    pub model: &'a crate::object_model::Model,
}

// ---- writing --------------------------------------------------------------------------------

fn write_mesh(
    root: &Path,
    index: &assets::Index,
    id: Uuid,
    fallback: &str,
    document: &Document,
) -> Result<(), String> {
    document.validate()?;
    // Regenerating rewrites the section where it is now, even if it was moved.
    let (path, revision) = match index.resolve(id) {
        Ok(record) => (record.path.clone(), Some(record.revision.clone())),
        Err(_) => (assets::inside(root, fallback)?, None),
    };
    let source = serde_json::to_vec(document).map_err(|e| e.to_string())?;
    let package = assets::Package {
        meta: assets::Metadata {
            version: 1,
            id,
            kind: assets::Kind::EditableMesh,
            importer_version: 1,
            source: fallback.into(),
            source_hash: assets::hash(&source),
            settings: crate::import_settings::Settings::Authored,
            extra: Default::default(),
        },
        source,
    };
    assets::atomic_write(&path, &package.bytes()?, revision.as_deref())
}

fn folder_name(name: &str) -> String {
    let stem: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if stem.trim_matches('_').is_empty() {
        "Dungeon".into()
    } else {
        stem
    }
}

fn child(scene: &Scene, mut actor: Actor, id: Uuid, name: &str, root: Uuid) -> Actor {
    actor.id = id;
    actor.data.id = id;
    actor.name = scene.unique_actor_name(name);
    actor.data.name = actor.name.clone();
    actor.logical_parent = Some(root);
    actor.attach = Some(Attachment {
        actor: root,
        component: None,
    });
    actor
}

/// The position in its parent's pixels that puts a 2D actor at `world`, or `None` when
/// the parent is scaled to nothing.
fn local_2d(scene: &Scene, index: usize, world: [f32; 2]) -> Option<[f32; 2]> {
    let Some(parent) = scene.actors[index]
        .attach
        .as_ref()
        .and_then(|a| scene.actor_index(a.actor))
    else {
        return Some(world);
    };
    let at = |p| crate::world2d::world_point(scene, parent, p);
    let (o, x, y) = (at([0., 0.]), at([1., 0.]), at([0., 1.]));
    let (ax, ay) = ([x[0] - o[0], x[1] - o[1]], [y[0] - o[0], y[1] - o[1]]);
    let det = ax[0] * ay[1] - ay[0] * ax[1];
    if det.abs() < 1e-6 {
        return None;
    }
    let d = [world[0] - o[0], world[1] - o[1]];
    Some([
        (d[0] * ay[1] - ay[0] * d[1]) / det,
        (ax[0] * d[1] - d[0] * ax[1]) / det,
    ])
}

/// Gives every component a stable identity derived from its actor.
fn stable_components(actor: &mut Actor) {
    for (i, component) in actor.components.iter_mut().enumerate() {
        component.id = dungeon::derived_id(actor.id, &format!("component-{i}"));
    }
}

/// Places a class instance under the generator and gives the placed branch stable
/// identities derived from `role`. Returns the identities of every placed actor.
fn place_piece(
    root: &Path,
    scene: &mut Scene,
    classes: &Classes,
    class: &str,
    owner: Uuid,
    role: &str,
    parent: Uuid,
) -> Result<(usize, Vec<Uuid>), String> {
    let first = scene.actors.len();
    let placed = crate::editor::place_class(scene, root, classes.registry, classes.model, class)?;
    // Template placements append their whole branch; remap it to stable identities.
    let mut actor_map = BTreeMap::new();
    let mut component_map = BTreeMap::new();
    for (k, actor) in scene.actors[first..].iter().enumerate() {
        let id = dungeon::derived_id(owner, &format!("{role}-{k}"));
        actor_map.insert(actor.id, id);
        for (j, component) in actor.components.iter().enumerate() {
            component_map.insert(
                component.id,
                dungeon::derived_id(id, &format!("component-{j}")),
            );
        }
    }
    for actor in &mut scene.actors[first..] {
        actor.id = actor_map[&actor.id];
        actor.data.id = actor.id;
        for component in &mut actor.components {
            component.id = component_map[&component.id];
        }
        if let Some(p) = actor.logical_parent.and_then(|p| actor_map.get(&p)) {
            actor.logical_parent = Some(*p);
        }
        if let Some(attach) = &mut actor.attach {
            if let Some(a) = actor_map.get(&attach.actor) {
                attach.actor = *a;
            }
            if let Some(c) = attach.component.and_then(|c| component_map.get(&c)) {
                attach.component = Some(*c);
            }
        }
    }
    let root_actor = &mut scene.actors[placed];
    root_actor.logical_parent = Some(parent);
    root_actor.attach = Some(Attachment {
        actor: parent,
        component: None,
    });
    let ids = scene.actors[first..].iter().map(|a| a.id).collect();
    Ok((placed, ids))
}

/// The property a key or locked-door class declares to receive its lock number.
pub const LOCK_PROPERTY: &str = "lock";

const NO_PLAYER_NOTE: &str =
    "Keys are picked up by anything that touches them; choose the Player so only it can";
const GAMEPLAY_NOTE: &str = "Added the DungeonKey and DungeonDoor scripts to assets/scripts; edit them to change how keys and doors behave";
/// 3D key: cube side, height of its centre and half the size of its pickup trigger.
const KEY_SIZE: f32 = 0.4;
const KEY_LIFT: f32 = 1.05;
const KEY_REACH: f32 = 0.5;
const DOOR_THICKNESS: f32 = 0.3;

fn needs_gameplay(placements: &[Placement]) -> bool {
    placements
        .iter()
        .any(|p| matches!(p.visual.piece, Piece::Key { .. } | Piece::Door { .. }))
}

/// The key and door classes the project uses, installing C++ ones when it has none,
/// and the player its keys open for: the generator's Player, when it exists.
fn gameplay(
    root: &Path,
    scene: &Scene,
    g: &Generator,
    placements: &[Placement],
    report: &mut Report,
) -> Result<Option<(crate::dungeon_gameplay::Classes, Option<Uuid>)>, String> {
    if !needs_gameplay(placements) {
        return Ok(None);
    }
    let (classes, written) = crate::dungeon_gameplay::ensure(root)?;
    if !written.added.is_empty() {
        report.notes.push(GAMEPLAY_NOTE.into());
    }
    if !written.updated.is_empty() {
        report.notes.push(format!(
            "Updated {}, unedited copies of an earlier version: keys now open only for the Player",
            written.updated.join(", ")
        ));
    }
    let player = g.spawn_actor.filter(|id| scene.actor_index(*id).is_some());
    if player.is_none()
        && placements
            .iter()
            .any(|p| matches!(p.visual.piece, Piece::Key { .. }))
    {
        report.notes.push(NO_PLAYER_NOTE.into());
    }
    Ok(Some((classes, player)))
}

/// The actor each lock's key becomes, so doors can name it.
fn key_actors<'a>(
    g: &Generator,
    spawns: impl Iterator<Item = (usize, Uuid, &'a Piece, u16)>,
) -> BTreeMap<u16, Uuid> {
    spawns
        .filter(|(_, _, piece, lock)| matches!(piece, Piece::Key { .. }) && *lock > 0)
        .map(|(n, node, _, lock)| (lock, g.derived_id(&format!("piece-{node}-{n}"))))
        .collect()
}

/// A key or door component carrying its lock number and the actor it refers to: a
/// key's player or a door's key.
fn gameplay_component(
    class: (&str, &str),
    label: &str,
    lock: u16,
    refers: Option<(&str, Uuid)>,
) -> crate::actor_document::ComponentInstance {
    let mut component = crate::actor_document::ComponentInstance::new(
        Uuid::nil(),
        crate::actor_document::ClassReference::new(class.0, class.1),
        label,
    );
    component
        .properties
        .insert(LOCK_PROPERTY.into(), serde_json::json!(lock));
    component.overrides.insert(LOCK_PROPERTY.into());
    if let Some((name, actor)) = refers {
        component
            .properties
            .insert(name.into(), serde_json::Value::String(actor.to_string()));
        component.overrides.insert(name.into());
    }
    component
}

/// A 2D key (a trigger with the key sprite) or locked door (a static body with the gate
/// sprite turned towards its corridor), drawn from the built-in dungeon tiles.
fn gameplay_2d(
    root: &Path,
    spawn: &Spawn2d,
    key: bool,
    color: [f32; 3],
    tile: [f32; 2],
    component: crate::actor_document::ComponentInstance,
) -> Result<Actor, String> {
    use crate::world2d::{self, Collider2D, Preset, Sprite2D};
    let (_, tiles) = builtin_tiles(root)?;
    let atlas = dungeon_style::MARKER_TILES
        .iter()
        .find(|(kind, _)| *kind == if key { SpotKind::Key } else { SpotKind::Lock })
        .map(|(_, atlas)| *atlas)
        .expect("markers include keys and locks");
    let preset = if key {
        Preset::Area2D
    } else {
        Preset::StaticBody2D
    };
    let label = if key { "Key" } else { "Locked door" };
    let mut actor = world2d::create(preset, label, spawn.position);
    let collider = Collider2D {
        size: if key { [12., 12.] } else { tile },
        trigger: key,
        ..Default::default()
    };
    match actor
        .components
        .iter_mut()
        .find(|c| c.class.class_id.as_deref() == Some(world2d::COLLIDER2D))
    {
        Some(component) => world2d::write(component, &collider),
        None => {
            if let Some(mut component) = world2d::new_component(&actor, world2d::COLLIDER2D) {
                world2d::write(&mut component, &collider);
                actor.components.push(component);
            }
        }
    }
    let size = 16;
    if let Some(mut component) = world2d::new_component(&actor, world2d::SPRITE2D) {
        world2d::write(
            &mut component,
            &Sprite2D {
                texture: tiles.texture,
                region: [
                    u16::from(atlas[0]) * size,
                    u16::from(atlas[1]) * size,
                    size,
                    size,
                ],
                modulate: color.map(|c| (c.clamp(0., 1.) * 255.).round() as u8),
                z_index: 2,
                ..Default::default()
            },
        );
        actor.components.push(component);
    }
    // The gate is drawn with its bars to the north; turn them towards the corridor.
    if let Some(side) = spawn.side.filter(|_| !key)
        && let Some(root) = actor.components.iter_mut().find(|c| c.root)
    {
        let degrees = match side {
            Side::North => 0.,
            Side::East => 90.,
            Side::South => 180.,
            Side::West => 270.,
        };
        root.properties
            .insert("rotation".into(), serde_json::json!(degrees));
        root.overrides.insert("rotation".into());
    }
    actor.components.push(component);
    actor.refresh_components();
    Ok(actor)
}

/// Gives the actors of a placed key or locked door their lock number, wherever their
/// classes or their components' classes declare an editable number named `lock`.
fn set_lock(scene: &mut Scene, classes: &Classes, ids: &[Uuid], lock: u16) {
    use crate::reflection_schema::Type;
    let declares = |class: &crate::actor_document::ClassReference| {
        let id = class.class_id.as_deref().unwrap_or(&class.name);
        classes.model.class(id).is_some_and(|model| {
            model.ancestry.iter().any(|ancestor| {
                classes.registry.classes.get(ancestor).is_some_and(|c| {
                    c.properties.iter().any(|p| {
                        p.name == LOCK_PROPERTY
                            && p.editable
                            && matches!(p.value_type, Type::Int32 | Type::UInt32 | Type::Fixed)
                    })
                })
            })
        })
    };
    for id in ids {
        let Some(index) = scene.actor_index(*id) else {
            continue;
        };
        let actor = &mut scene.actors[index];
        if declares(&actor.class) {
            actor
                .properties
                .insert(LOCK_PROPERTY.into(), serde_json::json!(lock));
            actor.overrides.insert(LOCK_PROPERTY.into());
        }
        for component in &mut actor.components {
            if declares(&component.class) {
                component
                    .properties
                    .insert(LOCK_PROPERTY.into(), serde_json::json!(lock));
                component.overrides.insert(LOCK_PROPERTY.into());
            }
        }
    }
}

/// Generates the layout of generator `id` and replaces its previous output in `scene`.
/// `classes` places Actor pieces; without it a style that uses them is refused. On
/// error the scene may be partly changed; callers keep a copy to restore.
pub fn generate(
    root: &Path,
    scene: &mut Scene,
    id: Uuid,
    classes: Option<&Classes>,
) -> Result<Report, String> {
    let index = scene
        .dungeons
        .iter()
        .position(|g| g.id == id)
        .ok_or("The Dungeon Generator was removed")?;
    let mut g = scene.dungeons[index].clone();
    g.validate()?;
    let root_index = scene
        .actor_index(g.root)
        .ok_or("The Dungeon Generator's actor was deleted")?;
    let two_d = is_2d(scene, g.root);
    let asset_index = assets::scan(root, &mut Default::default());
    let (recipe_digest, paths) = match g.source {
        dungeon::Source::Rooms => (String::new(), Vec::new()),
        dungeon::Source::Recipe => {
            let (recipe, digest) = crate::dungeon_recipe::of(&asset_index, &g)?;
            (digest, recipe.path_names())
        }
    };
    let layout = dungeon::layout(&g, &asset_index)?;
    let (style, style_digest) = style_of(&asset_index, &g, two_d)?;
    let placements = dungeon_style::evaluate(&style, &layout, g.seed);
    let needs_classes = placements
        .iter()
        .any(|p| matches!(p.visual.piece, Piece::Actor { class: Some(_) }));
    if needs_classes && classes.is_none() {
        return Err("The style places actors; their classes need the project's class model, which is unavailable".into());
    }
    scene.sync_actor_components();
    for actor in &g.output.actors {
        scene.delete_actor_branch(*actor);
    }
    let root_name = scene.actors[root_index].name.clone();
    let mut actors: Vec<Actor> = Vec::new();
    let mut placed_ids: Vec<Uuid> = Vec::new();
    let mut assets_written = Vec::new();
    let mut report = Report {
        summary: dungeon::summary(&layout),
        ..Default::default()
    };
    let queries;
    if two_d {
        let default_set = match g.tile_set {
            Some(set) => set,
            None => builtin_tiles(root)?.0,
        };
        let asset_index = assets::scan(root, &mut Default::default());
        let mut sets = BTreeMap::new();
        for id in style.references().into_iter().chain([default_set]) {
            if let Ok(record) = asset_index.resolve(id)
                && record.meta.kind == assets::Kind::TileSet
            {
                sets.insert(id, Arc::new(crate::tileset::document(record)?));
            }
        }
        let plan = plan_2d(&layout, &placements, default_set, &sets)?;
        let (offset, tile) = ([layout.size[0] / 2, layout.size[1] / 2], plan.tile);
        let corner = |x: i32, y: i32| {
            [
                (x - offset[0]) as f32 * tile[0],
                (y - offset[1]) as f32 * tile[1],
            ]
        };
        let (map, notes) = crate::dungeon_query::build(
            &layout,
            g.seed,
            &paths,
            &crate::dungeon_query::Grid {
                two_d: true,
                corner: &corner,
                floor: &|_| 0.,
                height: 0.,
            },
        );
        report.notes.extend(notes);
        queries = Some(map);
        let mut layers: Vec<(Uuid, crate::tilemap::Cells, &str, i16, bool)> = Vec::new();
        if let Some(set) = plan.tile_set {
            layers.push((set, plan.cells.clone(), "Tiles", 0, true));
        }
        for (set, cells) in &plan.decorations {
            layers.push((*set, cells.clone(), "Details", 1, false));
        }
        for (i, (set, cells, label, z, collision)) in layers.into_iter().enumerate() {
            let mut layer =
                crate::world2d::create(crate::world2d::Preset::TileMapLayer, label, [0., 0.]);
            if let Some(component) = layer.components.iter_mut().find(|c| {
                c.class.class_id.as_deref() == Some(crate::object_model::TILEMAP_LAYER_COMPONENT_ID)
            }) {
                crate::world2d::write(
                    component,
                    &crate::world2d::TileMapLayer {
                        tile_set: Some(set),
                        cells: crate::tilemap::encode(&cells),
                        z_index: z,
                        collision_enabled: collision,
                        ..Default::default()
                    },
                );
            }
            let role = if i == 0 && collision {
                "tiles".to_owned()
            } else {
                format!("details-{i}")
            };
            let layer = child(
                scene,
                layer,
                g.derived_id(&role),
                &format!("{root_name} {label}"),
                g.root,
            );
            actors.push(layer);
            report.notes.push(format!("{label}: {} tiles", cells.len()));
        }
        for (role, label, position) in [("spawn", "Spawn", plan.spawn), ("goal", "Goal", plan.goal)]
        {
            let marker = crate::world2d::create(crate::world2d::Preset::Node2D, label, position);
            actors.push(child(scene, marker, g.derived_id(role), label, g.root));
        }
        let gameplay = gameplay(root, scene, &g, &placements, &mut report)?;
        let keys = key_actors(
            &g,
            plan.spawns
                .iter()
                .enumerate()
                .map(|(n, s)| (n, s.node, &s.piece, s.lock)),
        );
        for (n, spawn) in plan.spawns.iter().enumerate() {
            let role = format!("piece-{}-{n}", spawn.node);
            let class = match &spawn.piece {
                Piece::Actor { class: Some(class) } => class,
                Piece::Key { color } | Piece::Door { color } => {
                    let key = matches!(spawn.piece, Piece::Key { .. });
                    let (classes, player) = gameplay.expect("checked above");
                    let component = if key {
                        let player = player.map(|p| (crate::dungeon_gameplay::PLAYER, p));
                        gameplay_component(classes.key, "Key", spawn.lock, player)
                    } else {
                        let opens = keys.get(&spawn.lock).copied();
                        let opens = opens.map(|k| (crate::dungeon_gameplay::KEY_REFERENCE, k));
                        gameplay_component(classes.door, "Door", spawn.lock, opens)
                    };
                    let actor = gameplay_2d(root, spawn, key, *color, plan.tile, component)?;
                    actors.push(child(
                        scene,
                        actor,
                        g.derived_id(&role),
                        if key { "Key" } else { "Locked door" },
                        g.root,
                    ));
                    continue;
                }
                _ => continue,
            };
            let classes = classes.expect("checked above");
            let (placed, ids) = place_piece(root, scene, classes, class, g.id, &role, g.root)?;
            if let Some(component) = scene.actors[placed].components.iter_mut().find(|c| c.root) {
                component
                    .properties
                    .insert("position".into(), serde_json::json!(spawn.position));
                component.overrides.insert("position".into());
            }
            if spawn.lock > 0 {
                set_lock(scene, classes, &ids, spawn.lock);
            }
            placed_ids.extend(ids);
        }
        for (set_id, set) in sets {
            scene.tilesets.insert(set_id, set);
        }
        if g.tile_set.is_none() && style.references().is_empty() {
            g.tile_set = Some(default_set);
        }
        if let Some(target) = g.spawn_actor
            && let Some(t) = scene.actor_index(target)
        {
            let world = crate::world2d::world_point(scene, root_index, plan.spawn);
            match local_2d(scene, t, world) {
                Some(local) => {
                    if let Some(component) = scene.actors[t].components.iter_mut().find(|c| c.root)
                    {
                        component
                            .properties
                            .insert("position".into(), serde_json::json!(local));
                        component.overrides.insert("position".into());
                    }
                }
                None => report.notes.push(format!(
                    "{} was not moved to the spawn: its parent cannot be inverted",
                    scene.actors[t].name
                )),
            }
        }
    } else {
        builtin_textures(root, &style)?;
        let meshes = piece_meshes(&asset_index, &placements)?;
        let plan = plan_3d(&g, &layout, &placements, &meshes)?;
        let frame = Frame {
            size: layout.size,
            cell: g.cell_size,
            height: g.wall_height,
        };
        let corner = |x: i32, y: i32| [frame.x(x), frame.z(y)];
        let (map, notes) = crate::dungeon_query::build(
            &layout,
            g.seed,
            &paths,
            &crate::dungeon_query::Grid {
                two_d: false,
                corner: &corner,
                floor: &|p| layout.floor(p),
                height: g.wall_height,
            },
        );
        report.notes.extend(notes);
        queries = Some(map);
        let folder = format!("{DUNGEON_FOLDER}/{}", folder_name(&root_name));
        let count = plan.sections.len();
        for (i, section) in plan.sections.iter().enumerate() {
            let asset = g.derived_id(&format!("mesh-{i}"));
            let fallback = format!("{folder}/Section{}.epokasset", i + 1);
            write_mesh(root, &asset_index, asset, &fallback, &section.document)?;
            assets_written.push(asset);
            let mut actor = Actor::cube("Geometry".into());
            actor.position = section.origin;
            actor.editable_mesh = Some(crate::mesh::Component {
                document: Some(Arc::new(section.document.clone())),
                ..crate::mesh::Component::new(asset)
            });
            actor.lighting.receive = crate::lighting::Receive::Baked;
            actor.lighting.static_geometry = true;
            // Under its own ceiling a dungeon would sit in shadow; letting the map's
            // light through keeps floors and walls lit and leaves the ceiling dark.
            actor.lighting.cast_shadows = !g.ceiling;
            let name = if count == 1 {
                "Geometry".to_owned()
            } else {
                format!("Geometry {}", i + 1)
            };
            actors.push(child(
                scene,
                actor,
                g.derived_id(&format!("geometry-{i}")),
                &name,
                g.root,
            ));
        }
        let mut collision = Actor::cube("Collision".into());
        collision.kind = "Empty".into();
        collision.position = [0.; 3];
        collision.collider = Some(crate::collision::Collider::compound(plan.boxes.clone()));
        actors.push(child(
            scene,
            collision,
            g.derived_id("collision"),
            "Collision",
            g.root,
        ));
        if let Some(navigation) = &plan.navigation {
            let count = navigation.volumes.len();
            for (i, (center, size)) in navigation.volumes.iter().enumerate() {
                let mut volume = Actor::cube("Navigation".into());
                volume.kind = "Empty".into();
                volume.position = *center;
                volume.scale = *size;
                let mut component = crate::actor_document::ComponentInstance::new(
                    Uuid::nil(),
                    crate::actor_document::ClassReference::new(
                        "epok::NavigationBakeVolumeComponent",
                        crate::navigation::VOLUME,
                    ),
                    "Navigation Bake Volume",
                );
                component
                    .properties
                    .insert("spacing".into(), serde_json::json!(navigation.spacing));
                component.overrides.insert("spacing".into());
                crate::actor_components::sync(&mut volume);
                volume.components.push(component);
                let name = if count == 1 {
                    "Navigation".to_owned()
                } else {
                    format!("Navigation {}", i + 1)
                };
                actors.push(child(
                    scene,
                    volume,
                    g.derived_id(&format!("navigation-{i}")),
                    &name,
                    g.root,
                ));
            }
            report.navigation_nodes = navigation.nodes;
        } else if g.navigation {
            report
                .notes
                .push("Navigation skipped: the floor is too large for 512 navigation nodes".into());
        }
        for (role, label, position) in [("spawn", "Spawn", plan.spawn), ("goal", "Goal", plan.goal)]
        {
            let mut marker = Actor::cube(label.into());
            marker.kind = "Empty".into();
            marker.position = position;
            actors.push(child(scene, marker, g.derived_id(role), label, g.root));
        }
        // Style lights are local accents; without a directional light of the map's own
        // the dungeon brings one, or a baked map renders at ambient level only.
        let generated: BTreeSet<Uuid> = actors.iter().map(|a| a.id).collect();
        if !scene.actors.iter().any(|a| {
            !generated.contains(&a.id)
                && a.light
                    .as_ref()
                    .is_some_and(|l| l.kind == crate::lighting::LightType::Directional)
        }) {
            let mut sun = Actor::cube("Light".into());
            sun.kind = "Empty".into();
            sun.rotation = [62., -32., 0.];
            sun.light = Some(crate::lighting::Light {
                mode: crate::lighting::LightMode::Mixed,
                intensity: 0.9,
                ..Default::default()
            });
            actors.push(child(scene, sun, g.derived_id("light"), "Light", g.root));
        }
        let gameplay = gameplay(root, scene, &g, &placements, &mut report)?;
        let keys = key_actors(
            &g,
            plan.spawns
                .iter()
                .enumerate()
                .map(|(n, s)| (n, s.node, &s.piece, s.lock)),
        );
        for (n, spawn) in plan.spawns.iter().enumerate() {
            let position = spawn.transform.point([0.; 3]);
            let forward = spawn.transform.vector([0., 0., 1.]);
            let yaw = forward[0].atan2(forward[2]).to_degrees();
            let role = format!("piece-{}-{n}", spawn.node);
            match &spawn.piece {
                Piece::Light {
                    color,
                    intensity,
                    range,
                } => {
                    let mut light = Actor::cube("Light".into());
                    light.kind = "Empty".into();
                    light.position = position;
                    // Under a ceiling a light above it would light nothing inside.
                    if g.ceiling {
                        let ceiling =
                            f32::from(spawn.level) * crate::dungeon::LEVEL_HEIGHT + g.wall_height;
                        light.position[1] = light.position[1].min(ceiling - 0.4);
                    }
                    light.light = Some(crate::lighting::Light {
                        kind: crate::lighting::LightType::Point,
                        mode: crate::lighting::LightMode::Baked,
                        color: *color,
                        intensity: *intensity,
                        range: *range,
                        ..Default::default()
                    });
                    actors.push(child(scene, light, g.derived_id(&role), "Light", g.root));
                }
                Piece::Key { color } => {
                    let mut key = Actor::cube("Key".into());
                    key.position = spawn.transform.point([0., KEY_LIFT, 0.]);
                    key.rotation = [0., yaw, 0.];
                    key.scale = [KEY_SIZE; 3];
                    key.material.color = *color;
                    // A trigger above the floor, so the dungeon's own boxes never touch it.
                    key.collider = Some(crate::collision::Collider {
                        trigger: true,
                        half_extents: [KEY_REACH / KEY_SIZE; 3],
                        ..Default::default()
                    });
                    crate::actor_components::sync(&mut key);
                    let (classes, player) = gameplay.expect("checked above");
                    let player = player.map(|p| (crate::dungeon_gameplay::PLAYER, p));
                    key.components
                        .push(gameplay_component(classes.key, "Key", spawn.lock, player));
                    actors.push(child(scene, key, g.derived_id(&role), "Key", g.root));
                }
                Piece::Door { color } => {
                    let height = g.door_height.min(g.wall_height);
                    let mut door = Actor::cube("Locked door".into());
                    door.position = spawn.transform.point([0., height * 0.5, 0.]);
                    door.rotation = [0., yaw, 0.];
                    door.scale = [g.cell_size, height, DOOR_THICKNESS];
                    door.material.color = *color;
                    door.collider = Some(crate::collision::Collider::default());
                    crate::actor_components::sync(&mut door);
                    let (classes, _) = gameplay.expect("checked above");
                    let opens = keys.get(&spawn.lock).copied();
                    let opens = opens.map(|k| (crate::dungeon_gameplay::KEY_REFERENCE, k));
                    door.components.push(gameplay_component(
                        classes.door,
                        "Door",
                        spawn.lock,
                        opens,
                    ));
                    actors.push(child(
                        scene,
                        door,
                        g.derived_id(&role),
                        "Locked door",
                        g.root,
                    ));
                }
                Piece::Actor { class: Some(class) } => {
                    let classes = classes.expect("checked above");
                    let (placed, ids) =
                        place_piece(root, scene, classes, class, g.id, &role, g.root)?;
                    let actor = &mut scene.actors[placed];
                    actor.position = position;
                    actor.rotation[1] += yaw;
                    if spawn.lock > 0 {
                        set_lock(scene, classes, &ids, spawn.lock);
                    }
                    placed_ids.extend(ids);
                }
                _ => {}
            }
        }
        report.sections = count;
        report.quads = plan.quads;
        report.boxes = plan.boxes.len();
        if let Some(target) = g.spawn_actor
            && let Some(t) = scene.actor_index(target)
        {
            let world = scene.world_matrix(root_index).point(plan.spawn);
            let local = match scene.actors[t].data.parent {
                Some(parent) => scene.world_matrix(parent).inverse().map(|m| m.point(world)),
                None => Ok(world),
            };
            match local {
                Ok(local) => scene.actors[t].position = local,
                Err(_) => report.notes.push(format!(
                    "{} was not moved to the spawn: its parent cannot be inverted",
                    scene.actors[t].name
                )),
            }
        }
    }
    for actor in &mut actors {
        crate::actor_components::sync(actor);
        stable_components(actor);
    }
    report.actors = actors.len() + placed_ids.len();
    let mut output_actors: Vec<Uuid> = actors.iter().map(|a| a.id).collect();
    // Placed branches are removed through their roots; listing every actor is harmless.
    output_actors.extend(placed_ids);
    g.output = dungeon::Output {
        actors: output_actors,
        assets: assets_written,
        fingerprint: g.fingerprint(),
        style: style_digest,
        recipe: recipe_digest,
        queries,
    };
    scene.actors.extend(actors);
    scene.refresh_actor_hierarchy();
    scene.dungeons[index] = g;
    scene.validate()?;
    Ok(report)
}

/// Removes a generator's output, keeping the generator itself.
pub fn clear(scene: &mut Scene, id: Uuid) {
    scene.sync_actor_components();
    if let Some(index) = scene.dungeons.iter().position(|g| g.id == id) {
        for actor in std::mem::take(&mut scene.dungeons[index].output.actors) {
            scene.delete_actor_branch(actor);
        }
        scene.dungeons[index].output = dungeon::Output::default();
    }
}

// ---- built-in art ----------------------------------------------------------------------------

mod art {
    use crate::tileset::{Collision, Terrain, TerrainMode, TerrainSet, Tile, TileSet};
    use uuid::Uuid;

    type Rgba = [u8; 4];
    const TILE: usize = 16;
    const COLUMNS: usize = 8;
    const ROWS: usize = 3;

    fn noise(x: usize, y: usize, seed: u32) -> u32 {
        let mut h = (x as u32).wrapping_mul(374_761_393)
            ^ (y as u32).wrapping_mul(668_265_263)
            ^ seed.wrapping_mul(2_246_822_519);
        h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
        h ^ (h >> 16)
    }
    fn shade(c: Rgba, amount: i32) -> Rgba {
        let f = |v: u8| (i32::from(v) + amount).clamp(0, 255) as u8;
        [f(c[0]), f(c[1]), f(c[2]), c[3]]
    }

    struct Canvas {
        pixels: Vec<Rgba>,
    }
    impl Canvas {
        fn set(&mut self, x: usize, y: usize, c: Rgba) {
            self.pixels[y * COLUMNS * TILE + x] = c;
        }
    }

    fn floor(c: &mut Canvas, column: usize, variant: u32) {
        const STONE: Rgba = [128, 116, 100, 255];
        const GROUT: Rgba = [84, 74, 66, 255];
        for y in 0..TILE {
            for x in 0..TILE {
                // Four slabs per tile, offset on alternate rows.
                let shift = if y < 8 { 0 } else { 4 };
                let grout = y % 8 == 7 || (x + shift) % 8 == 7;
                let n = noise(x + column * TILE, y, 11 + variant) % 23;
                let mut colour = if grout {
                    GROUT
                } else {
                    shade(STONE, n as i32 - 11)
                };
                if !grout && variant >= 2 && noise(x, y, variant * 31).is_multiple_of(29) {
                    colour = [70, 104, 66, 255]; // moss
                }
                if variant == 3 && !grout && (x + y == 12 || x + y == 13) && (4..12).contains(&x) {
                    colour = GROUT; // a crack across the slab
                }
                c.set(column * TILE + x, y, colour);
            }
        }
    }

    /// `mask` bit s set: the neighbour on side s (east, south, west, north) is wall too.
    fn wall(c: &mut Canvas, mask: usize) {
        const TOP: Rgba = [46, 40, 58, 255];
        const RIM: Rgba = [96, 88, 116, 255];
        const BRICK: Rgba = [116, 82, 70, 255];
        const MORTAR: Rgba = [62, 46, 46, 255];
        const OUTLINE: Rgba = [20, 16, 28, 255];
        let (column, row) = (mask % COLUMNS, 1 + mask / COLUMNS);
        let open = |side: usize| mask & (1 << side) == 0;
        for y in 0..TILE {
            for x in 0..TILE {
                let n = noise(x + mask * TILE, y, 5) % 11;
                let mut colour = shade(TOP, n as i32 - 5);
                // A brick face where the wall meets floor to the south (seen from above
                // and a little in front, as top-down maps draw walls).
                if open(1) && y >= 10 {
                    let row_offset = if ((y - 10) / 3).is_multiple_of(2) {
                        0
                    } else {
                        3
                    };
                    colour = if (y - 10) % 3 == 2 || (x + row_offset) % 6 == 5 {
                        MORTAR
                    } else {
                        shade(BRICK, n as i32 - 5)
                    };
                }
                if (open(0) && x == TILE - 1)
                    || (open(2) && x == 0)
                    || (open(3) && y == 0)
                    || (open(1) && y == TILE - 1)
                {
                    colour = OUTLINE;
                } else if (open(0) && x == TILE - 2)
                    || (open(2) && x == 1)
                    || (open(3) && y == 1)
                    || (open(1) && y == 9)
                {
                    colour = RIM;
                }
                c.set(column * TILE + x, row * TILE + y, colour);
            }
        }
    }

    /// Recipe markers in row 0, columns 4 to 7, on transparent backgrounds: a key, a
    /// locked gate, a one-way arrow and a chest. Directional ones point south.
    fn markers(c: &mut Canvas) {
        const GOLD: Rgba = [232, 188, 64, 255];
        const GOLD_DARK: Rgba = [150, 108, 30, 255];
        const IRON: Rgba = [70, 70, 84, 255];
        const IRON_LIGHT: Rgba = [130, 130, 150, 255];
        const RED: Rgba = [190, 48, 40, 255];
        const BLUE: Rgba = [70, 130, 220, 255];
        const BLUE_DARK: Rgba = [30, 62, 120, 255];
        const WOOD: Rgba = [130, 82, 44, 255];
        const WOOD_DARK: Rgba = [78, 46, 24, 255];
        let mut paint = |column: usize, f: &dyn Fn(i32, i32) -> Option<Rgba>| {
            for y in 0..TILE {
                for x in 0..TILE {
                    if let Some(colour) = f(x as i32, y as i32) {
                        c.set(column * TILE + x, y, colour);
                    }
                }
            }
        };
        paint(4, &|x, y| {
            // A key: a ring at the top, a shaft down the middle, teeth to the right.
            let (dx, dy) = (x - 7, y - 4);
            let ring = dx * dx + dy * dy;
            if (5..=13).contains(&ring) {
                return Some(if dy > 0 { GOLD_DARK } else { GOLD });
            }
            if (7..=8).contains(&x) && (7..=14).contains(&y) {
                return Some(if x == 8 { GOLD_DARK } else { GOLD });
            }
            if (9..=10).contains(&x) && (y == 11 || y == 13) {
                return Some(GOLD);
            }
            None
        });
        paint(5, &|x, y| {
            // A gate: bars along the north edge and a padlock below them.
            if y <= 5 && (x % 3 == 1 || y == 0 || y == 5) {
                return Some(if y == 0 { IRON_LIGHT } else { IRON });
            }
            let (dx, dy) = (x - 7, y - 8);
            if dy <= 0 && dx * dx + dy * dy <= 9 && dx * dx + dy * dy >= 4 && y >= 6 {
                return Some(IRON_LIGHT);
            }
            if (4..=11).contains(&x) && (9..=14).contains(&y) {
                return Some(if (7..=8).contains(&x) && (11..=12).contains(&y) {
                    IRON
                } else {
                    RED
                });
            }
            None
        });
        paint(6, &|x, y| {
            // Two chevrons pointing south.
            for top in [2, 8] {
                let dy = y - top;
                if (0..5).contains(&dy) && ((x - 7).abs() == dy || (x - 8).abs() == dy) {
                    return Some(BLUE);
                }
                if (0..5).contains(&dy) && ((x - 7).abs() == dy + 1 || (x - 8).abs() == dy + 1) {
                    return Some(BLUE_DARK);
                }
            }
            None
        });
        paint(7, &|x, y| {
            // A chest: a wooden box with a lid line and a gold clasp.
            if !(2..=13).contains(&x) || !(4..=13).contains(&y) {
                return None;
            }
            if x == 2 || x == 13 || y == 4 || y == 13 || y == 8 {
                return Some(WOOD_DARK);
            }
            if (7..=8).contains(&x) && (7..=10).contains(&y) {
                return Some(GOLD);
            }
            Some(WOOD)
        });
    }

    pub fn tiles() -> Result<Vec<u8>, String> {
        let (width, height) = (COLUMNS * TILE, ROWS * TILE);
        let mut canvas = Canvas {
            pixels: vec![[0, 0, 0, 0]; width * height],
        };
        for variant in 0..4 {
            floor(&mut canvas, variant as usize, variant);
        }
        for mask in 0..16 {
            wall(&mut canvas, mask);
        }
        markers(&mut canvas);
        let mut bytes = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut bytes, width as u32, height as u32);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
            let data: Vec<u8> = canvas.pixels.iter().flatten().copied().collect();
            writer.write_image_data(&data).map_err(|e| e.to_string())?;
        }
        Ok(bytes)
    }

    pub fn tileset(texture: Uuid) -> TileSet {
        let mut set = TileSet {
            texture: Some(texture),
            terrain_sets: vec![TerrainSet {
                name: "Dungeon".into(),
                mode: TerrainMode::Sides,
                terrains: vec![
                    Terrain {
                        name: super::FLOOR_TERRAIN.into(),
                        color: [150, 140, 160],
                    },
                    Terrain {
                        name: super::WALL_TERRAIN.into(),
                        color: [90, 70, 110],
                    },
                ],
            }],
            ..Default::default()
        };
        for (variant, weight) in [1.0, 0.5, 0.25, 0.12].into_iter().enumerate() {
            let mut tile = Tile::new([variant as u8, 0]);
            tile.terrain_set = Some(0);
            tile.terrain = Some(0);
            tile.probability = weight;
            for side in 0..4 {
                tile.peering[side * 2] = Some(0);
            }
            set.tiles.push(tile);
        }
        for (_, atlas) in crate::dungeon_style::MARKER_TILES {
            set.tiles.push(Tile::new(atlas));
        }
        for mask in 0..16usize {
            let mut tile = Tile::new([(mask % COLUMNS) as u8, (1 + mask / COLUMNS) as u8]);
            tile.collision = Collision::Full;
            tile.terrain_set = Some(0);
            tile.terrain = Some(1);
            for side in 0..4 {
                tile.peering[side * 2] = Some(if mask & (1 << side) != 0 { 1 } else { 0 });
            }
            set.tiles.push(tile);
        }
        set
    }
}
