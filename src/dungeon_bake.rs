//! Bakes a generated layout into ordinary map content.
//!
//! 3D: walls are the empty cells around every floor, so each room and corridor is
//! enclosed by solid blocks. Floors, wall faces, wall tops and doorway lintels are
//! merged into large faces inside one EditableMesh per section; collision is one
//! compound collider; navigation, spawn and goal are separate actors. The output stays
//! a handful of actors whatever the size of the dungeon.
//!
//! 2D: floors and walls are terrains of a TileSet, painted into one TileMapLayer and
//! solved with the TileSet's own terrain rules.
use crate::actor_document::Attachment;
use crate::assets;
use crate::dungeon::{self, Cell, Generator, Layout, Side, SpotKind};
use crate::mesh::{Document, Face, Group, Slot};
use crate::reflection_schema::Domain;
use crate::scene::{Actor, Material, Scene};
use std::collections::{BTreeSet, HashMap};
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

/// An empty cell next to a floor, including diagonally: the solid ring around rooms
/// and corridors.
pub fn wall_cell(layout: &Layout, p: [i32; 2]) -> bool {
    layout.at(p) == Cell::Empty
        && (-1..=1).any(|dy| (-1..=1).any(|dx| layout.at([p[0] + dx, p[1] + dy]).walkable()))
}

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

pub struct Plan3d {
    pub sections: Vec<Section>,
    pub boxes: Vec<[f32; 6]>,
    pub spawn: [f32; 3],
    pub goal: [f32; 3],
    /// None when the floor is too large for the node budget.
    pub navigation: Option<Navigation>,
    pub quads: usize,
}

/// How far wall faces continue below the floor.
const WALL_SKIRT: f32 = 0.08;
/// How far merged wall tops reach past their cells.
const CAP_OVERLAP: f32 = 0.03;
const SLOT_FLOOR: usize = 0;
const SLOT_CORRIDOR: usize = 1;
const SLOT_WALL: usize = 2;
const SLOT_TOP: usize = 3;
const SLOT_DOORWAY: usize = 4;
const SLOTS: [(&str, [f32; 3]); 5] = [
    ("Floor", [0.46, 0.43, 0.40]),
    ("Corridor Floor", [0.36, 0.34, 0.33]),
    ("Wall", [0.66, 0.61, 0.54]),
    ("Wall Top", [0.30, 0.28, 0.30]),
    ("Doorway", [0.48, 0.33, 0.21]),
];
const GROUPS: [&str; 3] = ["Floors", "Walls", "Doorways"];
/// A face's group and material slot.
type Paint = (usize, usize);

struct Builder {
    document: Document,
    vertices: HashMap<[u32; 3], u32>,
    seed: u128,
    origin: [f32; 3],
    /// World units per texture repeat.
    cell: f32,
    quads: usize,
}
impl Builder {
    fn new(owner: Uuid, origin: [f32; 3], cell: f32) -> Self {
        let id = |role: &str| dungeon::derived_id(owner, role);
        Self {
            document: Document {
                version: 1,
                vertices: Vec::new(),
                faces: Vec::new(),
                groups: GROUPS
                    .iter()
                    .map(|name| Group {
                        id: id(&format!("group-{name}")),
                        name: (*name).into(),
                        parent: None,
                    })
                    .collect(),
                materials: SLOTS
                    .iter()
                    .map(|(name, color)| Slot {
                        id: id(&format!("slot-{name}")),
                        name: (*name).into(),
                        material: Material {
                            color: *color,
                            ..Material::default()
                        },
                    })
                    .collect(),
            },
            vertices: HashMap::new(),
            seed: owner.as_u128(),
            origin,
            cell,
            quads: 0,
        }
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
    /// A quad given counter-clockwise when seen from its front.
    fn quad(&mut self, p: [[f32; 3]; 4], uv: [[f32; 2]; 4], paint: Paint, size: [f32; 2]) {
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
            vertex_colors: None,
        });
        self.quads += compiled(size[0], size[1]);
    }
    /// A horizontal rectangle facing up (or down) at height `y`.
    fn horizontal(&mut self, x: [f32; 2], z: [f32; 2], y: f32, up: bool, paint: Paint) {
        let cell = self.cell;
        let mut p = [
            [x[0], y, z[0]],
            [x[0], y, z[1]],
            [x[1], y, z[1]],
            [x[1], y, z[0]],
        ];
        if !up {
            p.reverse();
        }
        let uv = p.map(|v| [v[0] / cell, v[2] / cell]);
        self.quad(p, uv, paint, [x[1] - x[0], z[1] - z[0]]);
    }
    /// A vertical rectangle along an edge from `a` to `b` (x, z), between heights `y`,
    /// facing the side to the left of a→b.
    fn vertical(&mut self, a: [f32; 2], b: [f32; 2], y: [f32; 2], paint: Paint) {
        let cell = self.cell;
        let p = [
            [a[0], y[0], a[1]],
            [b[0], y[0], b[1]],
            [b[0], y[1], b[1]],
            [a[0], y[1], a[1]],
        ];
        let length = (b[0] - a[0]).abs() + (b[1] - a[1]).abs();
        let along = |v: [f32; 3]| (v[0] - a[0]).abs() + (v[2] - a[1]).abs();
        let uv = p.map(|v| [along(v) / cell, -v[1] / cell]);
        self.quad(p, uv, paint, [length, y[1] - y[0]]);
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

fn build_section(
    g: &Generator,
    layout: &Layout,
    frame: &Frame,
    region: [i32; 4],
    doorways: &BTreeSet<[i32; 2]>,
    owner: Uuid,
) -> Section {
    let c = frame.cell;
    let h = g.wall_height;
    let origin = [
        (frame.x(region[0]) + frame.x(region[2])) * 0.5,
        0.,
        (frame.z(region[1]) + frame.z(region[3])) * 0.5,
    ];
    let mut b = Builder::new(owner, origin, c);
    // Floors and walls are one face per cell or cell side. Every vertex then sits on
    // the grid lattice, so no edge ends in the middle of a neighbour's edge: on the PSX
    // such T-junctions show one-pixel cracks. One face per cell also gives baked
    // lighting a vertex per cell corner.
    for y in region[1]..region[3] {
        for x in region[0]..region[2] {
            let cell = layout.at([x, y]);
            if !cell.walkable() {
                continue;
            }
            let slot = if cell == Cell::Corridor {
                SLOT_CORRIDOR
            } else {
                SLOT_FLOOR
            };
            b.horizontal(
                [frame.x(x), frame.x(x + 1)],
                [frame.z(y), frame.z(y + 1)],
                0.,
                true,
                (0, slot),
            );
            for side in Side::ALL {
                if layout.open([x, y], side) || layout.door([x, y], side) {
                    continue;
                }
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
                // Walls also reach a little below the floor, covering the seam.
                b.vertical(a, bb, [-WALL_SKIRT, h], (1, SLOT_WALL));
            }
        }
    }
    // Wall tops over the solid ring and over doorways, merged into rectangles that
    // overlap their neighbours slightly so the top edges cannot crack either.
    for [x0, y0, x1, y1] in rectangles(region, &|p| wall_cell(layout, p) || doorways.contains(&p)) {
        b.horizontal(
            [frame.x(x0) - CAP_OVERLAP, frame.x(x1) + CAP_OVERLAP],
            [frame.z(y0) - CAP_OVERLAP, frame.z(y1) + CAP_OVERLAP],
            h,
            true,
            (1, SLOT_TOP),
        );
    }
    // Doorway lintels: the corridor cell just outside each door gets a beam between
    // door height and the wall top, open underneath.
    if g.door_height < h - 0.01 {
        for p in doorways.iter().filter(|p| {
            p[0] >= region[0] && p[0] < region[2] && p[1] >= region[1] && p[1] < region[3]
        }) {
            let (x0, x1, z0, z1) = (
                frame.x(p[0]),
                frame.x(p[0] + 1),
                frame.z(p[1]),
                frame.z(p[1] + 1),
            );
            b.horizontal([x0, x1], [z0, z1], g.door_height, false, (2, SLOT_DOORWAY));
            for side in Side::ALL {
                // Faces towards the open sides of the doorway cell.
                if !layout.open(*p, side) {
                    continue;
                }
                let (a, bb) = match side {
                    Side::North => ([x0, z0], [x1, z0]),
                    Side::South => ([x1, z1], [x0, z1]),
                    Side::West => ([x0, z1], [x0, z0]),
                    Side::East => ([x1, z0], [x1, z1]),
                };
                // Seen from the neighbour, so the winding is reversed.
                b.vertical(bb, a, [g.door_height, h], (2, SLOT_DOORWAY));
            }
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
fn sections(g: &Generator, layout: &Layout, frame: &Frame, owner: Uuid) -> Vec<Section> {
    let doorways = doorway_cells(layout);
    let mut pending = vec![[0, 0, layout.size[0], layout.size[1]]];
    let mut out = Vec::new();
    while let Some(region) = pending.pop() {
        let section = build_section(
            g,
            layout,
            frame,
            region,
            &doorways,
            dungeon::derived_id(
                owner,
                &format!(
                    "section-{}-{}-{}-{}",
                    region[0], region[1], region[2], region[3]
                ),
            ),
        );
        let span = (region[2] - region[0]).max(region[3] - region[1]) as f32 * frame.cell;
        let too_big = section.document.faces.len() > SECTION_FACES
            || section.document.vertices.len() > 16000
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

pub fn plan_3d(g: &Generator, layout: &Layout) -> Result<Plan3d, String> {
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
    let sections = sections(g, layout, &frame, g.id);
    let h = g.wall_height;
    let all = [0, 0, layout.size[0], layout.size[1]];
    let mut boxes = Vec::new();
    for [x0, y0, x1, y1] in rectangles(all, &|p| layout.at(p).walkable()) {
        boxes.push([frame.x(x0), -0.5, frame.z(y0), frame.x(x1), 0., frame.z(y1)]);
    }
    for [x0, y0, x1, y1] in rectangles(all, &|p| wall_cell(layout, p)) {
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
    Ok(Plan3d {
        sections,
        boxes,
        spawn: frame.center(layout.rooms[layout.spawn].center()),
        goal: frame.center(layout.rooms[layout.goal].center()),
        navigation,
        quads,
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

/// The terrain set and the Floor and Wall terrain indices a TileSet offers.
pub fn terrains(set: &crate::tileset::TileSet) -> Option<(usize, u8, u8)> {
    set.terrain_sets.iter().enumerate().find_map(|(index, s)| {
        let find = |name: &str| {
            s.terrains
                .iter()
                .position(|t| t.name.eq_ignore_ascii_case(name))
                .map(|t| t as u8)
        };
        Some((index, find(FLOOR_TERRAIN)?, find(WALL_TERRAIN)?))
    })
}

pub struct Plan2d {
    pub cells: crate::tilemap::Cells,
    pub spawn: [f32; 2],
    pub goal: [f32; 2],
}

pub fn plan_2d(layout: &Layout, set: &crate::tileset::TileSet) -> Result<Plan2d, String> {
    let (terrain_set, floor, wall) = terrains(set).ok_or(
        "The TileSet needs a terrain set with terrains named Floor and Wall; choose the built-in dungeon tiles or add them in the TileSet editor",
    )?;
    let offset = [layout.size[0] / 2, layout.size[1] / 2];
    let painted: BTreeSet<[i32; 2]> = (0..layout.size[1])
        .flat_map(|y| (0..layout.size[0]).map(move |x| [x, y]))
        .filter(|p| layout.at(*p).walkable() || wall_cell(layout, *p))
        .collect();
    // Everything beyond the painted cells reads as wall, so outer wall tiles show no
    // edge towards the void.
    let terrain_of = |p: [i32; 2]| {
        Some(
            if layout.at([p[0] + offset[0], p[1] + offset[1]]).walkable() {
                floor
            } else {
                wall
            },
        )
    };
    let shifted: BTreeSet<[i32; 2]> = painted
        .iter()
        .map(|p| [p[0] - offset[0], p[1] - offset[1]])
        .collect();
    let solved = crate::tileset::solve(set, terrain_set, &shifted, &terrain_of);
    let cells = solved
        .into_iter()
        .filter(|(p, _)| shifted.contains(p))
        .map(|(p, atlas)| (p, crate::tilemap::Cell::new(atlas)))
        .collect();
    let tile = [f32::from(set.tile_size[0]), f32::from(set.tile_size[1])];
    let center = |room: usize| {
        let c = layout.rooms[room].center();
        [
            ((c[0] - offset[0]) as f32 + 0.5) * tile[0],
            ((c[1] - offset[1]) as f32 + 0.5) * tile[1],
        ]
    };
    Ok(Plan2d {
        cells,
        spawn: center(layout.spawn),
        goal: center(layout.goal),
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

/// Generates the layout of generator `id` and replaces its previous output in `scene`.
/// On error the scene may be partly changed; callers keep a copy to restore.
pub fn generate(root: &Path, scene: &mut Scene, id: Uuid) -> Result<Report, String> {
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
    scene.sync_actor_components();
    for actor in &g.output.actors {
        scene.delete_actor_branch(*actor);
    }
    let root_name = scene.actors[root_index].name.clone();
    let mut actors: Vec<Actor> = Vec::new();
    let mut assets_written = Vec::new();
    let mut report = Report {
        summary: dungeon::summary(&layout),
        ..Default::default()
    };
    if two_d {
        let (set_id, set) = match g.tile_set {
            Some(set) => {
                let index = assets::scan(root, &mut Default::default());
                let record = index.resolve(set)?;
                (set, crate::tileset::document(record)?)
            }
            None => builtin_tiles(root)?,
        };
        g.tile_set = Some(set_id);
        let plan = plan_2d(&layout, &set)?;
        let mut layer =
            crate::world2d::create(crate::world2d::Preset::TileMapLayer, "Tiles", [0., 0.]);
        if let Some(component) = layer.components.iter_mut().find(|c| {
            c.class.class_id.as_deref() == Some(crate::object_model::TILEMAP_LAYER_COMPONENT_ID)
        }) {
            crate::world2d::write(
                component,
                &crate::world2d::TileMapLayer {
                    tile_set: Some(set_id),
                    cells: crate::tilemap::encode(&plan.cells),
                    ..Default::default()
                },
            );
        }
        let layer = child(
            scene,
            layer,
            g.derived_id("tiles"),
            &format!("{root_name} Tiles"),
            g.root,
        );
        actors.push(layer);
        for (role, label, position) in [("spawn", "Spawn", plan.spawn), ("goal", "Goal", plan.goal)]
        {
            let marker = crate::world2d::create(crate::world2d::Preset::Node2D, label, position);
            actors.push(child(scene, marker, g.derived_id(role), label, g.root));
        }
        scene.tilesets.insert(set_id, Arc::new(set));
        report.notes.push(format!("{} tiles", plan.cells.len()));
    } else {
        let plan = plan_3d(&g, &layout)?;
        let asset_index = assets::scan(root, &mut Default::default());
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
        // A baked map without any light renders at ambient level only.
        let generated: BTreeSet<Uuid> = actors.iter().map(|a| a.id).collect();
        if !scene
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
    report.actors = actors.len();
    g.output = dungeon::Output {
        actors: actors.iter().map(|a| a.id).collect(),
        assets: assets_written,
        fingerprint: g.fingerprint(),
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
