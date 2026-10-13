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
const NAVIGATION_VOLUMES: usize = 6;

/// An actor a style piece places, relative to the generator root.
#[derive(Clone, Debug)]
pub struct Spawn {
    pub node: Uuid,
    pub transform: Matrix,
    pub piece: Piece,
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
const GROUPS: [&str; 4] = ["Floors", "Walls", "Doorways", "Pieces"];
const GROUP_FLOORS: usize = 0;
const GROUP_WALLS: usize = 1;
const GROUP_DOORWAYS: usize = 2;
const GROUP_PIECES: usize = 3;

struct Builder {
    document: Document,
    vertices: HashMap<[u32; 3], u32>,
    slots: HashMap<String, usize>,
    owner: Uuid,
    seed: u128,
    origin: [f32; 3],
    /// World units per texture repeat.
    cell: f32,
    quads: usize,
}
impl Builder {
    fn new(owner: Uuid, origin: [f32; 3], cell: f32) -> Self {
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
            cell,
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
        let mut p = [
            [x[0], y, z[0]],
            [x[0], y, z[1]],
            [x[1], y, z[1]],
            [x[1], y, z[0]],
        ];
        if !up {
            p.reverse();
        }
        let cell = self.cell;
        let uv = p.map(|v| [v[0] / cell, v[2] / cell]);
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
        let along = |v: [f32; 3]| (v[0] - a[0]).abs() + (v[2] - a[1]).abs();
        let cell = self.cell;
        let uv = p.map(|v| [along(v) / cell, -v[1] / cell]);
        self.face(p, uv, None, paint);
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
    let mut b = Builder::new(owner, origin, c);
    // Wall tops are merged per material once every solid cell is known.
    let mut tops: Vec<(usize, BTreeSet<[i32; 2]>)> = Vec::new();
    for placement in cx.placements.iter().filter(|p| inside(p.spot.cell)) {
        let spot = &placement.spot;
        let [x, y] = spot.cell;
        match &placement.visual.piece {
            Piece::Blockout { color, texture } => {
                let slot = b.blockout(spot.kind, *color, *texture);
                match (spot.kind, spot.side) {
                    // Floors and walls are one face per cell or cell side. Every vertex
                    // then sits on the grid lattice, so no edge ends in the middle of a
                    // neighbour's edge: on the PSX such T-junctions show one-pixel
                    // cracks. One face per cell also gives baked lighting a vertex per
                    // cell corner.
                    (SpotKind::Floor, _) => b.horizontal(
                        [frame.x(x), frame.x(x + 1)],
                        [frame.z(y), frame.z(y + 1)],
                        0.,
                        true,
                        (GROUP_FLOORS, slot),
                    ),
                    (SpotKind::Wall, Some(side)) => {
                        // Edge endpoints ordered so the face looks into the cell.
                        let (a, bb) = match side {
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
                        };
                        // Walls reach a little below the floor, covering the seam.
                        b.vertical(a, bb, [-WALL_SKIRT, h], (GROUP_WALLS, slot));
                    }
                    (SpotKind::Solid, _) => match tops.iter_mut().find(|(s, _)| *s == slot) {
                        Some((_, cells)) => {
                            cells.insert(spot.cell);
                        }
                        None => tops.push((slot, BTreeSet::from([spot.cell]))),
                    },
                    (SpotKind::Doorway, _) if g.door_height < h - 0.01 => {
                        let (x0, x1, z0, z1) =
                            (frame.x(x), frame.x(x + 1), frame.z(y), frame.z(y + 1));
                        b.horizontal(
                            [x0, x1],
                            [z0, z1],
                            g.door_height,
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
                            b.vertical(bb, a, [g.door_height, h], (GROUP_DOORWAYS, slot));
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
                        b.horizontal([x0, x1], [z0, z1], h, true, (GROUP_WALLS, slot));
                        let inner_x = if sx > 0. { x1 } else { x0 };
                        let inner_z = if sz > 0. { z1 } else { z0 };
                        // The face parallel to X, at inner_z, facing ±Z.
                        if sz > 0. {
                            b.vertical(
                                [x1, inner_z],
                                [x0, inner_z],
                                [-WALL_SKIRT, h],
                                (GROUP_WALLS, slot),
                            );
                        } else {
                            b.vertical(
                                [x0, inner_z],
                                [x1, inner_z],
                                [-WALL_SKIRT, h],
                                (GROUP_WALLS, slot),
                            );
                        }
                        // The face parallel to Z, at inner_x, facing ±X.
                        if sx > 0. {
                            b.vertical(
                                [inner_x, z0],
                                [inner_x, z1],
                                [-WALL_SKIRT, h],
                                (GROUP_WALLS, slot),
                            );
                        } else {
                            b.vertical(
                                [inner_x, z1],
                                [inner_x, z0],
                                [-WALL_SKIRT, h],
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
    for (slot, cells) in tops {
        for [x0, y0, x1, y1] in rectangles(region, &|p| cells.contains(&p)) {
            b.horizontal(
                [frame.x(x0) - CAP_OVERLAP, frame.x(x1) + CAP_OVERLAP],
                [frame.z(y0) - CAP_OVERLAP, frame.z(y1) + CAP_OVERLAP],
                h,
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
    if g.half_extent() > 127. {
        return Err(format!(
            "The dungeon spans {:.0} units; keep grid size × cell size within 254 units (reduce the cell size or the grid)",
            g.half_extent() * 2.
        ));
    }
    let frame = Frame {
        size: layout.size,
        cell: g.cell_size,
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
    let mut boxes = Vec::new();
    for [x0, y0, x1, y1] in rectangles(all, &|p| layout.at(p).walkable()) {
        boxes.push([frame.x(x0), -0.5, frame.z(y0), frame.x(x1), 0., frame.z(y1)]);
    }
    for [x0, y0, x1, y1] in rectangles(all, &|p| layout.solid(p)) {
        boxes.push([frame.x(x0), 0., frame.z(y0), frame.x(x1), h, frame.z(y1)]);
    }
    if g.door_height < h - 0.01 {
        let doorways = doorway_cells(layout);
        for [x0, y0, x1, y1] in rectangles(all, &|p| doorways.contains(&p)) {
            boxes.push([
                frame.x(x0),
                g.door_height,
                frame.z(y0),
                frame.x(x1),
                h,
                frame.z(y1),
            ]);
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
                Piece::Actor { class: Some(_) } | Piece::Light { .. }
            )
        })
        .map(|p| Spawn {
            node: p.node,
            transform: dungeon_style::transform(p, &frame),
            piece: p.visual.piece.clone(),
        })
        .collect();
    Ok(Plan3d {
        sections,
        boxes,
        spawn: frame.center(layout.rooms[layout.spawn].center()),
        goal: frame.center(layout.rooms[layout.goal].center()),
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
    let (bottom, top) = (-1., g.wall_height - 0.5);
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
    pub class: String,
}

pub struct Plan2d {
    /// The TileSet the floor and solid cells are painted with, if any terrain is used.
    pub tile_set: Option<Uuid>,
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
            decorations[index]
                .1
                .entry(shift(dungeon_style::tile_cell(&p.spot)))
                .or_insert(crate::tilemap::Cell::new(*atlas));
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
        .filter_map(|p| match &p.visual.piece {
            Piece::Actor { class: Some(class) } => Some(Spawn2d {
                node: p.node,
                position: pixel(p.spot.cell, p.visual.offset),
                class: class.clone(),
            }),
            _ => None,
        })
        .collect();
    Ok(Plan2d {
        tile_set: terrain_set,
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
    if let Some(record) = existing {
        return Ok((record.meta.id, crate::tileset::document(&record)?));
    }
    crate::project::write_changed(&assets::inside(root, TILES_IMAGE)?, &art::tiles()?)?;
    let texture = assets::commit(crate::texture::prepare(
        root,
        TILES_IMAGE,
        "assets/Dungeons/DungeonTilesTexture.epokasset",
        None,
        false,
    )?)?;
    let set = art::tileset(texture);
    let id = crate::tileset::create(root, TILES_PATH, &set)?;
    Ok((id, set))
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
        None => dungeon_style::builtin(two_d, None),
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
    let layout = dungeon::layout(&g)?;
    let two_d = is_2d(scene, g.root);
    let asset_index = assets::scan(root, &mut Default::default());
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
        for (n, spawn) in plan.spawns.iter().enumerate() {
            let classes = classes.expect("checked above");
            let (placed, ids) = place_piece(
                root,
                scene,
                classes,
                &spawn.class,
                g.id,
                &format!("piece-{}-{n}", spawn.node),
                g.root,
            )?;
            if let Some(component) = scene.actors[placed].components.iter_mut().find(|c| c.root) {
                component
                    .properties
                    .insert("position".into(), serde_json::json!(spawn.position));
                component.overrides.insert("position".into());
            }
            placed_ids.extend(ids);
        }
        for (set_id, set) in sets {
            scene.tilesets.insert(set_id, set);
        }
        if g.tile_set.is_none() && style.references().is_empty() {
            g.tile_set = Some(default_set);
        }
    } else {
        let meshes = piece_meshes(&asset_index, &placements)?;
        let plan = plan_3d(&g, &layout, &placements, &meshes)?;
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
        let style_lights = plan
            .spawns
            .iter()
            .any(|s| matches!(s.piece, Piece::Light { .. }));
        // A baked map without any light renders at ambient level only.
        let generated: BTreeSet<Uuid> = actors.iter().map(|a| a.id).collect();
        if !style_lights
            && !scene
                .actors
                .iter()
                .any(|a| a.light.is_some() && !generated.contains(&a.id))
        {
            let mut sun = Actor::cube("Light".into());
            sun.kind = "Empty".into();
            sun.rotation = [58., -32., 0.];
            sun.light = Some(crate::lighting::Light {
                mode: crate::lighting::LightMode::Mixed,
                intensity: 0.65,
                ..Default::default()
            });
            actors.push(child(scene, sun, g.derived_id("light"), "Light", g.root));
        }
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
                Piece::Actor { class: Some(class) } => {
                    let classes = classes.expect("checked above");
                    let (placed, ids) =
                        place_piece(root, scene, classes, class, g.id, &role, g.root)?;
                    let actor = &mut scene.actors[placed];
                    actor.position = position;
                    actor.rotation[1] += yaw;
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
