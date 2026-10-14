//! 2D navigation records and algorithms (World2D initiative, phase 4): NavigationRegion2D
//! (a walkable grid built from tile collision and static bodies, or a polygon navmesh)
//! and NavigationAgent2D. Mirrors `runtime/world2d_navigation.hpp`: the editor builds
//! and cooks the regions and uses the same searches to preview agents' paths.
//!
//! Grid search: A* over cells with integer costs 2 (straight) and 3 (diagonal), no
//! corner cutting, then line-of-sight pruning through cell centres. Navmesh search: A*
//! over triangles, then the funnel algorithm through the shared edges.
use crate::{scene::Scene, tileset::TileSet, world2d};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Reverse,
    collections::{BTreeMap, BinaryHeap},
    sync::Arc,
};
use uuid::Uuid;

pub const MAX_GRID_CELLS: usize = 16384;
pub const MAX_OUTLINE: usize = 64;
pub const MAX_PATH: usize = 32;

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum NavMode {
    #[default]
    Grid,
    Polygon,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct NavigationRegion2D {
    pub mode: NavMode,
    /// Grid: follow this TileMapLayer's cells; its solid tiles block.
    pub tile_layer: Option<Uuid>,
    /// Grid without a tile layer: cell size and covered area from the node's position.
    pub cell_size: f32,
    pub size: [f32; 2],
    /// Grid on a tile layer: free cells kept around the painted area.
    pub margin: i32,
    /// Grid: solid colliders of nodes without a CharacterBody2D block their cells.
    pub static_bodies: bool,
    /// Grid: cells closer than this to a blocked cell block too.
    pub agent_radius: f32,
    pub diagonal: bool,
    /// Polygon: the walkable outline in the node's local pixels.
    pub outline: Vec<[f32; 2]>,
}
impl Default for NavigationRegion2D {
    fn default() -> Self {
        Self {
            mode: NavMode::Grid,
            tile_layer: None,
            cell_size: 16.,
            size: [320., 240.],
            margin: 1,
            static_bodies: true,
            agent_radius: 0.,
            diagonal: true,
            outline: vec![[0., 0.], [160., 0.], [160., 96.], [0., 96.]],
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct NavigationAgent2D {
    /// Follow this node; without one, gameplay sets a target position.
    pub target: Option<Uuid>,
    pub max_speed: f32,
    /// How close counts as having reached a waypoint and the target.
    pub arrive_distance: f32,
    /// Seconds between new paths towards a moving target.
    pub repath_interval: f32,
    /// Move the node itself: through its CharacterBody2D's move_and_slide, or directly.
    pub drives_body: bool,
}
impl Default for NavigationAgent2D {
    fn default() -> Self {
        Self {
            target: None,
            max_speed: 64.,
            arrive_distance: 4.,
            repath_interval: 0.5,
            drives_body: true,
        }
    }
}

// ---- grids -------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct Grid {
    /// World position of cell (0, 0)'s top-left corner.
    pub origin: [f32; 2],
    pub cell: [f32; 2],
    pub columns: usize,
    pub rows: usize,
    pub blocked: Vec<bool>,
    pub diagonal: bool,
}
impl Grid {
    pub fn cell_of(&self, p: [f32; 2]) -> [i32; 2] {
        [0, 1].map(|k| ((p[k] - self.origin[k]) / self.cell[k]).floor() as i32)
    }
    pub fn centre(&self, c: [i32; 2]) -> [f32; 2] {
        [0, 1].map(|k| self.origin[k] + (c[k] as f32 + 0.5) * self.cell[k])
    }
    pub fn inside(&self, c: [i32; 2]) -> bool {
        c[0] >= 0 && c[1] >= 0 && (c[0] as usize) < self.columns && (c[1] as usize) < self.rows
    }
    pub fn walkable(&self, c: [i32; 2]) -> bool {
        self.inside(c) && !self.blocked[c[1] as usize * self.columns + c[0] as usize]
    }
    fn block_rect(&mut self, lo: [f32; 2], hi: [f32; 2]) {
        // Cells the box overlaps by more than a hair: touching edges stay free.
        let a = self.cell_of([lo[0] + 0.01, lo[1] + 0.01]);
        let b = self.cell_of([hi[0] - 0.01, hi[1] - 0.01]);
        for y in a[1].max(0)..=b[1].min(self.rows as i32 - 1) {
            for x in a[0].max(0)..=b[0].min(self.columns as i32 - 1) {
                self.blocked[y as usize * self.columns + x as usize] = true;
            }
        }
    }
    /// Blocks every cell within `cells` (Chebyshev distance) of a blocked cell.
    fn dilate(&mut self, cells: i32) {
        if cells <= 0 {
            return;
        }
        let source = self.blocked.clone();
        for y in 0..self.rows as i32 {
            for x in 0..self.columns as i32 {
                if !source[y as usize * self.columns + x as usize] {
                    continue;
                }
                for dy in -cells..=cells {
                    for dx in -cells..=cells {
                        let c = [x + dx, y + dy];
                        if self.inside(c) {
                            self.blocked[c[1] as usize * self.columns + c[0] as usize] = true;
                        }
                    }
                }
            }
        }
    }
    /// Whether the straight line between two cell centres only crosses walkable cells,
    /// squeezing past no corner (both cells beside a diagonal step must be free).
    pub fn line_of_sight(&self, a: [i32; 2], b: [i32; 2]) -> bool {
        let (dx, dy) = ((b[0] - a[0]).abs(), (b[1] - a[1]).abs());
        let (sx, sy) = ((b[0] - a[0]).signum(), (b[1] - a[1]).signum());
        let (mut x, mut y, mut ix, mut iy) = (a[0], a[1], 0, 0);
        while ix < dx || iy < dy {
            let decision = (1 + 2 * ix) * dy - (1 + 2 * iy) * dx;
            if decision == 0 {
                if !self.walkable([x + sx, y]) || !self.walkable([x, y + sy]) {
                    return false;
                }
                x += sx;
                y += sy;
                ix += 1;
                iy += 1;
            } else if decision < 0 {
                x += sx;
                ix += 1;
            } else {
                y += sy;
                iy += 1;
            }
            if !self.walkable([x, y]) {
                return false;
            }
        }
        true
    }
}

/// A region's grid in world pixels. `origin` is the region node's world position.
pub fn build_grid(
    region: &NavigationRegion2D,
    origin: [f32; 2],
    scene: &Scene,
    tilesets: &BTreeMap<Uuid, Arc<TileSet>>,
    exclude: Option<Uuid>,
) -> Result<Grid, String> {
    let mut grid;
    let mut solid_tiles: Vec<[i32; 2]> = Vec::new();
    if let Some(layer_id) = region.tile_layer {
        let index = scene
            .actor_index(layer_id)
            .ok_or("the region's tile layer is not in this map")?;
        let actor = &scene.actors[index];
        let component = world2d::component(actor, world2d::TILEMAP_LAYER)
            .ok_or("the region's tile layer node has no TileMapLayer")?;
        let layer: world2d::TileMapLayer = world2d::read(component);
        let set = layer
            .tile_set
            .and_then(|id| tilesets.get(&id))
            .ok_or("the region's tile layer has no TileSet")?;
        let t = crate::world2d_editor::root_transform(actor);
        let parented = actor.attach.is_some();
        if t.rotation.abs() > 1e-3
            || (t.scale[0] - 1.).abs() > 1e-3
            || (t.scale[1] - 1.).abs() > 1e-3
            || parented
        {
            return Err(
                "navigation follows a tile layer that is neither rotated, scaled nor attached"
                    .into(),
            );
        }
        let cells = crate::tilemap::decode(&layer.cells)?;
        let Some((lo, hi)) = crate::tilemap::bounds(&cells) else {
            return Err("the region's tile layer has no tiles".into());
        };
        let margin = region.margin.clamp(0, 64);
        let size = [f32::from(set.tile_size[0]), f32::from(set.tile_size[1])];
        let first = [lo[0] - margin, lo[1] - margin];
        let columns = (hi[0] - lo[0] + 1 + 2 * margin) as usize;
        let rows = (hi[1] - lo[1] + 1 + 2 * margin) as usize;
        if columns * rows > MAX_GRID_CELLS {
            return Err(format!(
                "the navigation grid would have {} cells; at most {MAX_GRID_CELLS}",
                columns * rows
            ));
        }
        grid = Grid {
            origin: [
                t.position[0] + first[0] as f32 * size[0],
                t.position[1] + first[1] as f32 * size[1],
            ],
            cell: size,
            columns,
            rows,
            blocked: vec![false; columns * rows],
            diagonal: region.diagonal,
        };
        if layer.collision_enabled {
            for (at, cell) in &cells {
                let solid = set.tile(cell.atlas).is_some_and(|tile| {
                    tile.collision != crate::tileset::Collision::None && !tile.one_way
                });
                if solid {
                    solid_tiles.push([at[0] - first[0], at[1] - first[1]]);
                }
            }
        }
    } else {
        let cell = region.cell_size.clamp(4., 128.);
        let columns = (region.size[0].max(cell) / cell).ceil() as usize;
        let rows = (region.size[1].max(cell) / cell).ceil() as usize;
        if columns * rows > MAX_GRID_CELLS {
            return Err(format!(
                "the navigation grid would have {} cells; at most {MAX_GRID_CELLS} (raise the cell size)",
                columns * rows
            ));
        }
        grid = Grid {
            origin,
            cell: [cell; 2],
            columns,
            rows,
            blocked: vec![false; columns * rows],
            diagonal: region.diagonal,
        };
    }
    for c in solid_tiles {
        if grid.inside(c) {
            grid.blocked[c[1] as usize * grid.columns + c[0] as usize] = true;
        }
    }
    if region.static_bodies {
        for (index, actor) in scene.actors.iter().enumerate() {
            if Some(actor.id) == exclude
                || !actor.active
                || world2d::component(actor, world2d::CHARACTER_BODY2D).is_some()
            {
                continue;
            }
            for component in actor
                .components
                .iter()
                .filter(|c| c.class.class_id.as_deref() == Some(world2d::COLLIDER2D))
            {
                let collider: world2d::Collider2D = world2d::read(component);
                if !collider.enabled || collider.trigger || collider.one_way {
                    continue;
                }
                let (lo, hi) = collider.bounds();
                let corners = [
                    [lo[0], lo[1]],
                    [hi[0], lo[1]],
                    [lo[0], hi[1]],
                    [hi[0], hi[1]],
                ]
                .map(|p| world2d::world_point(scene, index, p));
                let min = [0, 1].map(|k| corners.iter().map(|c| c[k]).fold(f32::MAX, f32::min));
                let max = [0, 1].map(|k| corners.iter().map(|c| c[k]).fold(f32::MIN, f32::max));
                grid.block_rect(min, max);
            }
        }
    }
    let radius = region.agent_radius.max(0.);
    if radius > 0. {
        grid.dilate((radius / grid.cell[0].min(grid.cell[1])).ceil() as i32);
    }
    Ok(grid)
}

const NEIGHBOURS: [[i32; 2]; 8] = [
    [1, 0],
    [0, 1],
    [-1, 0],
    [0, -1],
    [1, 1],
    [-1, 1],
    [-1, -1],
    [1, -1],
];
fn octile(a: [i32; 2], b: [i32; 2]) -> u32 {
    let (dx, dy) = ((a[0] - b[0]).unsigned_abs(), (a[1] - b[1]).unsigned_abs());
    2 * dx.max(dy) + dx.min(dy)
}
/// The nearest walkable cell to `c` within three rings, or None.
fn nearest_walkable(grid: &Grid, c: [i32; 2]) -> Option<[i32; 2]> {
    if grid.walkable(c) {
        return Some(c);
    }
    for ring in 1..=3i32 {
        for dy in -ring..=ring {
            for dx in -ring..=ring {
                if dx.abs() == ring || dy.abs() == ring {
                    let n = [c[0] + dx, c[1] + dy];
                    if grid.walkable(n) {
                        return Some(n);
                    }
                }
            }
        }
    }
    None
}
/// Waypoints from `from` towards `to` (the start excluded) and whether `to` itself is
/// reached; an unreachable target gives the path to the closest reachable cell.
pub fn grid_path(grid: &Grid, from: [f32; 2], to: [f32; 2]) -> Option<(Vec<[f32; 2]>, bool)> {
    let start = nearest_walkable(grid, grid.cell_of(from))?;
    let raw_goal = grid.cell_of(to);
    let goal = [
        raw_goal[0].clamp(0, grid.columns as i32 - 1),
        raw_goal[1].clamp(0, grid.rows as i32 - 1),
    ];
    let index = |c: [i32; 2]| c[1] as usize * grid.columns + c[0] as usize;
    let cells = grid.columns * grid.rows;
    let mut g = vec![u32::MAX; cells];
    let mut parent = vec![u8::MAX; cells];
    let mut closed = vec![false; cells];
    let mut open = BinaryHeap::new();
    g[index(start)] = 0;
    open.push(Reverse((octile(start, goal), 0u32, start)));
    let mut best = (octile(start, goal), start);
    let mut reached = false;
    while let Some(Reverse((_, cost, c))) = open.pop() {
        if closed[index(c)] || cost > g[index(c)] {
            continue;
        }
        closed[index(c)] = true;
        let h = octile(c, goal);
        if h < best.0 {
            best = (h, c);
        }
        if c == goal {
            reached = true;
            break;
        }
        for (d, step) in NEIGHBOURS.iter().enumerate() {
            let diagonal = d >= 4;
            if diagonal
                && (!grid.diagonal
                    || !grid.walkable([c[0] + step[0], c[1]])
                    || !grid.walkable([c[0], c[1] + step[1]]))
            {
                continue;
            }
            let n = [c[0] + step[0], c[1] + step[1]];
            if !grid.walkable(n) || closed[index(n)] {
                continue;
            }
            let next = cost + if diagonal { 3 } else { 2 };
            if next < g[index(n)] {
                g[index(n)] = next;
                parent[index(n)] = d as u8;
                open.push(Reverse((next + octile(n, goal), next, n)));
            }
        }
    }
    let end = if reached { goal } else { best.1 };
    let mut cells_path = vec![end];
    let mut c = end;
    while c != start {
        let d = parent[index(c)];
        if d == u8::MAX {
            break;
        }
        let step = NEIGHBOURS[d as usize];
        c = [c[0] - step[0], c[1] - step[1]];
        cells_path.push(c);
    }
    cells_path.reverse();
    // Line-of-sight pruning: from each waypoint, jump to the farthest visible cell.
    let mut points = Vec::new();
    let mut at = 0;
    while at + 1 < cells_path.len() && points.len() < MAX_PATH {
        let mut next = at + 1;
        let window = (at + 64).min(cells_path.len() - 1);
        for k in (at + 2..=window).rev() {
            if grid.line_of_sight(cells_path[at], cells_path[k]) {
                next = k;
                break;
            }
        }
        points.push(grid.centre(cells_path[next]));
        at = next;
    }
    let reached = reached && grid.walkable(raw_goal);
    if reached {
        match points.last_mut() {
            Some(last) => *last = to,
            None => points.push(to),
        }
    }
    Some((points, reached))
}

// ---- navmeshes ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct Mesh {
    pub vertices: Vec<[f32; 2]>,
    /// Vertex indices, every triangle wound the same way (positive `area2`).
    pub triangles: Vec<[usize; 3]>,
    /// The triangle across edge (v0 v1), (v1 v2) and (v2 v0), or -1.
    pub neighbours: Vec<[i32; 3]>,
}
fn area2(a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> f32 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}
pub fn build_mesh(outline: &[[f32; 2]]) -> Result<Mesh, String> {
    if outline.len() > MAX_OUTLINE {
        return Err(format!(
            "a navigation outline holds at most {MAX_OUTLINE} points"
        ));
    }
    let triangles: Vec<[usize; 3]> = crate::shape2d::triangulate(outline)?
        .into_iter()
        .map(|t| {
            let t = t.map(usize::from);
            if area2(outline[t[0]], outline[t[1]], outline[t[2]]) < 0. {
                [t[0], t[2], t[1]]
            } else {
                t
            }
        })
        .collect();
    let mut edges: BTreeMap<(usize, usize), (usize, usize)> = BTreeMap::new();
    let mut neighbours = vec![[-1; 3]; triangles.len()];
    for (i, t) in triangles.iter().enumerate() {
        for e in 0..3 {
            let (a, b) = (t[e], t[(e + 1) % 3]);
            let key = (a.min(b), a.max(b));
            if let Some((j, f)) = edges.get(&key) {
                neighbours[i][e] = *j as i32;
                neighbours[*j][*f] = i as i32;
            } else {
                edges.insert(key, (i, e));
            }
        }
    }
    Ok(Mesh {
        vertices: outline.to_vec(),
        triangles,
        neighbours,
    })
}
fn closest_on_segment(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    let d = [b[0] - a[0], b[1] - a[1]];
    let length = d[0] * d[0] + d[1] * d[1];
    let t = if length > 0. {
        (((p[0] - a[0]) * d[0] + (p[1] - a[1]) * d[1]) / length).clamp(0., 1.)
    } else {
        0.
    };
    [a[0] + d[0] * t, a[1] + d[1] * t]
}
impl Mesh {
    fn corners(&self, t: usize) -> [[f32; 2]; 3] {
        self.triangles[t].map(|v| self.vertices[v])
    }
    fn centroid(&self, t: usize) -> [f32; 2] {
        let c = self.corners(t);
        [0, 1].map(|k| (c[0][k] + c[1][k] + c[2][k]) / 3.)
    }
    pub fn contains(&self, t: usize, p: [f32; 2]) -> bool {
        let c = self.corners(t);
        (0..3).all(|e| area2(c[e], c[(e + 1) % 3], p) >= -1e-3)
    }
    /// The triangle holding `p`, or the closest triangle and the closest point on it.
    pub fn locate(&self, p: [f32; 2]) -> Option<(usize, [f32; 2], bool)> {
        if let Some(t) = (0..self.triangles.len()).find(|t| self.contains(*t, p)) {
            return Some((t, p, true));
        }
        let mut best: Option<(f32, usize, [f32; 2])> = None;
        for t in 0..self.triangles.len() {
            let c = self.corners(t);
            for e in 0..3 {
                let q = closest_on_segment(p, c[e], c[(e + 1) % 3]);
                let d = (q[0] - p[0]).hypot(q[1] - p[1]);
                if best.is_none_or(|b| d < b.0) {
                    best = Some((d, t, q));
                }
            }
        }
        best.map(|(_, t, q)| (t, q, false))
    }
}
/// Waypoints from `from` towards `to` (the start excluded) and whether `to` lies on the
/// mesh; a target off the mesh is replaced by the closest point on it.
pub fn mesh_path(mesh: &Mesh, from: [f32; 2], to: [f32; 2]) -> Option<(Vec<[f32; 2]>, bool)> {
    let (start_t, start, _) = mesh.locate(from)?;
    let (goal_t, goal, on_mesh) = mesh.locate(to)?;
    let n = mesh.triangles.len();
    let mut g = vec![f32::MAX; n];
    let mut parent = vec![usize::MAX; n];
    let mut closed = vec![false; n];
    g[start_t] = 0.;
    let distance = |a: [f32; 2], b: [f32; 2]| (b[0] - a[0]).hypot(b[1] - a[1]);
    let mut found = start_t == goal_t;
    while !found {
        let Some(t) = (0..n)
            .filter(|t| !closed[*t] && g[*t] < f32::MAX)
            .min_by(|a, b| {
                let fa = g[*a] + distance(mesh.centroid(*a), goal);
                let fb = g[*b] + distance(mesh.centroid(*b), goal);
                fa.total_cmp(&fb)
            })
        else {
            break;
        };
        closed[t] = true;
        if t == goal_t {
            found = true;
            break;
        }
        for &next in &mesh.neighbours[t] {
            if next < 0 || closed[next as usize] {
                continue;
            }
            let next = next as usize;
            let cost = g[t] + distance(mesh.centroid(t), mesh.centroid(next));
            if cost < g[next] {
                g[next] = cost;
                parent[next] = t;
            }
        }
    }
    if !found {
        return None;
    }
    let mut corridor = vec![goal_t];
    while *corridor.last()? != start_t {
        corridor.push(parent[*corridor.last()?]);
    }
    corridor.reverse();
    // Portals: the shared edge of each consecutive pair, (left, right) as seen walking.
    let mut portals = vec![(start, start)];
    for pair in corridor.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let e = (0..3).find(|e| mesh.neighbours[a][*e] == b as i32)?;
        let t = mesh.triangles[a];
        let (u, v) = (mesh.vertices[t[e]], mesh.vertices[t[(e + 1) % 3]]);
        // Triangles wind with positive area2, so the edge's second vertex is on the left.
        portals.push((v, u));
    }
    portals.push((goal, goal));
    Some((funnel(&portals), on_mesh))
}
/// The string-pulled path through `portals` (left, right), the start excluded.
fn funnel(portals: &[([f32; 2], [f32; 2])]) -> Vec<[f32; 2]> {
    let same = |a: [f32; 2], b: [f32; 2]| (a[0] - b[0]).abs() < 1e-4 && (a[1] - b[1]).abs() < 1e-4;
    let mut out = Vec::new();
    let mut apex = portals[0].0;
    let (mut left, mut right) = portals[0];
    let (mut left_i, mut right_i) = (0, 0);
    let mut i = 1;
    while i < portals.len() {
        let (l, r) = portals[i];
        // Tighten the right side.
        if area2(apex, right, r) >= 0. {
            if same(apex, right) || area2(apex, left, r) < 0. {
                right = r;
                right_i = i;
            } else {
                if out.last().is_none_or(|p| !same(*p, left)) {
                    out.push(left);
                }
                // The funnel restarts from its new apex.
                let apex_i = left_i;
                apex = left;
                right = apex;
                left_i = apex_i;
                right_i = apex_i;
                i = apex_i + 1;
                continue;
            }
        }
        // Tighten the left side.
        if area2(apex, left, l) <= 0. {
            if same(apex, left) || area2(apex, right, l) > 0. {
                left = l;
                left_i = i;
            } else {
                if out.last().is_none_or(|p| !same(*p, right)) {
                    out.push(right);
                }
                // The funnel restarts from its new apex.
                let apex_i = right_i;
                apex = right;
                left = apex;
                left_i = apex_i;
                right_i = apex_i;
                i = apex_i + 1;
                continue;
            }
        }
        i += 1;
    }
    let goal = portals[portals.len() - 1].0;
    if out.last().is_none_or(|p| !same(*p, goal)) {
        out.push(goal);
    }
    out.truncate(MAX_PATH);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open_grid(columns: usize, rows: usize) -> Grid {
        Grid {
            origin: [0., 0.],
            cell: [16., 16.],
            columns,
            rows,
            blocked: vec![false; columns * rows],
            diagonal: true,
        }
    }

    #[test]
    fn an_open_grid_goes_straight_to_the_target() {
        let grid = open_grid(10, 10);
        let (points, reached) = grid_path(&grid, [8., 8.], [150., 150.]).unwrap();
        assert!(reached);
        assert_eq!(points, vec![[150., 150.]]);
    }

    #[test]
    fn walls_bend_the_path_around_their_end() {
        // A wall at column 4 from row 0 to row 7; the gap is at rows 8-9.
        let mut grid = open_grid(10, 10);
        for y in 0..8 {
            grid.blocked[y * 10 + 4] = true;
        }
        let (points, reached) = grid_path(&grid, [8., 8.], [8. + 16. * 8., 8.]).unwrap();
        assert!(reached);
        // Through the gap: some waypoint below the wall's end.
        assert!(points.iter().any(|p| p[1] > 16. * 8.), "{points:?}");
        // Every leg is in line of sight.
        let mut at = grid.cell_of([8., 8.]);
        for p in &points {
            let c = grid.cell_of(*p);
            assert!(grid.line_of_sight(at, c), "{at:?} -> {c:?}");
            at = c;
        }
    }

    #[test]
    fn diagonal_steps_never_cut_corners() {
        let mut grid = open_grid(3, 3);
        grid.blocked[1] = true; // (1, 0)
        grid.blocked[3] = true; // (0, 1)
        // (0,0) to (1,1) would squeeze between two blocked cells.
        assert!(!grid.line_of_sight([0, 0], [1, 1]));
        let (points, reached) = grid_path(&grid, [8., 8.], [24., 24.]).unwrap();
        assert!(!reached && points.is_empty());
    }

    #[test]
    fn an_unreachable_target_leads_to_the_closest_cell() {
        let mut grid = open_grid(10, 3);
        for y in 0..3 {
            grid.blocked[y * 10 + 5] = true;
        }
        let (points, reached) = grid_path(&grid, [8., 24.], [150., 24.]).unwrap();
        assert!(!reached);
        assert_eq!(grid.cell_of(*points.last().unwrap()), [4, 1]);
    }

    #[test]
    fn grids_follow_tile_collision_static_bodies_and_the_agent_radius() {
        let mut scene = Scene {
            actors: Vec::new(),
            ..Default::default()
        };
        let mut set = TileSet::default();
        let mut wall = crate::tileset::Tile::new([0, 0]);
        wall.collision = crate::tileset::Collision::Full;
        set.tiles.push(wall);
        let mut ledge = crate::tileset::Tile::new([1, 0]);
        ledge.collision = crate::tileset::Collision::Full;
        ledge.one_way = true;
        set.tiles.push(ledge);
        let set_id = Uuid::from_u128(9);
        let tilesets = BTreeMap::from([(set_id, Arc::new(set))]);
        let mut cells = crate::tilemap::Cells::new();
        cells.insert([0, 0], crate::tilemap::Cell::new([0, 0]));
        cells.insert([5, 3], crate::tilemap::Cell::new([1, 0]));
        let mut layer = world2d::create(world2d::Preset::TileMapLayer, "Ground", [0., 0.]);
        let c = layer
            .components
            .iter_mut()
            .find(|c| c.class.class_id.as_deref() == Some(world2d::TILEMAP_LAYER))
            .unwrap();
        world2d::write(
            c,
            &world2d::TileMapLayer {
                tile_set: Some(set_id),
                cells: crate::tilemap::encode(&cells),
                ..Default::default()
            },
        );
        let layer_id = layer.id;
        scene.actors.push(layer);
        let mut crate_box = world2d::create(world2d::Preset::StaticBody2D, "Crate", [40., 40.]);
        let c = crate_box
            .components
            .iter_mut()
            .find(|c| c.class.class_id.as_deref() == Some(world2d::COLLIDER2D))
            .unwrap();
        world2d::write(
            c,
            &world2d::Collider2D {
                size: [16., 16.],
                ..Default::default()
            },
        );
        scene.actors.push(crate_box);
        let region = NavigationRegion2D {
            tile_layer: Some(layer_id),
            margin: 1,
            ..Default::default()
        };
        let grid = build_grid(&region, [0., 0.], &scene, &tilesets, None).unwrap();
        // Cells -1..=6 by -1..=4.
        assert_eq!((grid.columns, grid.rows), (8, 6));
        assert_eq!(grid.origin, [-16., -16.]);
        assert!(!grid.walkable(grid.cell_of([8., 8.])), "solid tile");
        assert!(
            grid.walkable(grid.cell_of([88., 56.])),
            "one-way tiles do not block"
        );
        // The crate covers 32..48 x 32..48: one cell.
        assert!(!grid.walkable(grid.cell_of([40., 40.])));
        assert!(grid.walkable(grid.cell_of([56., 40.])));
        let wide = NavigationRegion2D {
            agent_radius: 8.,
            ..region
        };
        let grid = build_grid(&wide, [0., 0.], &scene, &tilesets, None).unwrap();
        assert!(
            !grid.walkable(grid.cell_of([56., 40.])),
            "dilated by the radius"
        );
    }

    #[test]
    fn navmesh_paths_turn_at_the_inner_corner() {
        // An L: across the top, then down the right side.
        let outline = [
            [0., 0.],
            [100., 0.],
            [100., 100.],
            [70., 100.],
            [70., 30.],
            [0., 30.],
        ];
        for reversed in [false, true] {
            let mut points = outline.to_vec();
            if reversed {
                points.reverse();
            }
            let mesh = build_mesh(&points).unwrap();
            let (path, on_mesh) = mesh_path(&mesh, [10., 15.], [85., 90.]).unwrap();
            assert!(on_mesh);
            assert_eq!(path.len(), 2, "{path:?}");
            assert_eq!(path[0], [70., 30.]);
            assert_eq!(path[1], [85., 90.]);
        }
    }

    #[test]
    fn navmesh_targets_off_the_mesh_clamp_to_its_edge() {
        let mesh = build_mesh(&[[0., 0.], [100., 0.], [100., 50.], [0., 50.]]).unwrap();
        let (path, on_mesh) = mesh_path(&mesh, [10., 10.], [50., 90.]).unwrap();
        assert!(!on_mesh);
        assert_eq!(path, vec![[50., 50.]]);
        assert!(build_mesh(&[[0., 0.]; 70]).is_err());
    }
}
