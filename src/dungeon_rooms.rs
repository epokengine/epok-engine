//! Rooms-and-corridors layouts.
//!
//! Rectangular rooms are scattered on the grid with a minimum spacing. Rooms whose
//! centres see each other across an empty circle (the Gabriel graph, which always
//! contains the shortest spanning tree) are candidate connections: a spanning tree of
//! them guarantees every room is reachable, and each remaining candidate becomes a loop
//! with the configured chance. Corridors are routed with A* that keeps one cell away
//! from every room other than the two it joins, prefers straight runs and reuses
//! existing corridors.
use crate::dungeon::{Cell, Layout, Room, RoomSettings};
use crate::dungeon_random::Random;
use std::cmp::Reverse;
use std::collections::{BTreeSet, BinaryHeap};

const STAGE_PLACE: u32 = 1;
const STAGE_CONNECT: u32 = 2;
const STAGE_ROUTE: u32 = 3;
const STAGE_ENDS: u32 = 4;

pub fn generate(settings: &RoomSettings, seed: u32) -> Result<Layout, String> {
    let size = [i32::from(settings.size[0]), i32::from(settings.size[1])];
    let mut rooms = place(settings, size, &mut Random::stream(seed, STAGE_PLACE));
    if rooms.len() < 2 {
        return Err(
            "Fewer than two rooms fit: enlarge the grid, shrink the rooms or reduce the spacing"
                .into(),
        );
    }
    let mut grid = Grid::new(size, &rooms);
    let candidates = gabriel(&rooms);
    let (tree, extra) = spanning_tree(rooms.len(), &candidates);
    let mut connect = Random::stream(seed, STAGE_CONNECT);
    let mut links = tree;
    for edge in extra {
        if connect.chance(u32::from(settings.loops)) {
            links.push(edge);
        }
    }
    let mut route = Random::stream(seed, STAGE_ROUTE);
    let width = i32::from(settings.corridor_width);
    for &[a, b] in &links {
        grid.route(&rooms, a, b, width, &mut route);
    }
    // A blocked tree link can leave rooms apart; join the nearest rooms of different
    // groups until everything connects or nothing else can be routed.
    let mut tried = BTreeSet::new();
    loop {
        let groups = grid.groups(&rooms);
        if groups.iter().all(|g| *g == groups[0]) {
            break;
        }
        let mut pairs: Vec<(i64, usize, usize)> = (0..rooms.len())
            .flat_map(|a| (a + 1..rooms.len()).map(move |b| (a, b)))
            .filter(|&(a, b)| groups[a] != groups[b] && !tried.contains(&(a, b)))
            .map(|(a, b)| (distance2(&rooms[a], &rooms[b]), a, b))
            .collect();
        pairs.sort();
        let Some(&(_, a, b)) = pairs.first() else {
            break;
        };
        tried.insert((a, b));
        grid.route(&rooms, a, b, width, &mut route);
    }
    // Rooms that still cannot be reached are left out of the dungeon.
    let groups = grid.groups(&rooms);
    let mut counts = std::collections::BTreeMap::new();
    for g in &groups {
        *counts.entry(*g).or_insert(0usize) += 1;
    }
    let main = counts
        .iter()
        .max_by_key(|(g, n)| (**n, Reverse(**g)))
        .map(|(g, _)| *g)
        .unwrap_or(0);
    if groups.iter().any(|g| *g != main) {
        grid.keep(&rooms, &groups, main);
        let kept: Vec<usize> = (0..rooms.len()).filter(|r| groups[*r] == main).collect();
        rooms = kept.iter().map(|r| rooms[*r]).collect();
        grid.renumber(&rooms);
        if rooms.len() < 2 {
            return Err(
                "Corridors could not connect the rooms: reduce the spacing or the room count"
                    .into(),
            );
        }
    }
    let mut layout = Layout {
        size,
        cells: grid.cells,
        rooms,
        links: Vec::new(),
        spawn: 0,
        goal: 0,
        requested: usize::from(settings.rooms),
        loops: 0,
        locks: Vec::new(),
        one_ways: Vec::new(),
        items: Vec::new(),
        room_paths: Vec::new(),
    };
    let (links, loops) = connections(&layout);
    layout.links = links;
    layout.loops = loops;
    let (spawn, goal) = ends(&layout, &mut Random::stream(seed, STAGE_ENDS));
    layout.spawn = spawn;
    layout.goal = goal;
    Ok(layout)
}

fn place(settings: &RoomSettings, size: [i32; 2], random: &mut Random) -> Vec<Room> {
    let wanted = usize::from(settings.rooms);
    let (min, max) = (i32::from(settings.room_min), i32::from(settings.room_max));
    let spacing = i32::from(settings.spacing);
    let mut rooms: Vec<Room> = Vec::new();
    for _ in 0..wanted * 60 {
        if rooms.len() == wanted {
            break;
        }
        let w = random.range(min, max);
        let h = random.range(min, max);
        let x = random.range(1, size[0] - 1 - w);
        let y = random.range(1, size[1] - 1 - h);
        // Keep rooms from becoming corridors themselves.
        if w > h * 2 || h > w * 2 || x + w > size[0] - 1 || y + h > size[1] - 1 {
            continue;
        }
        let room = Room {
            min: [x, y],
            size: [w, h],
        };
        let apart = |r: &Room| {
            (0..2).any(|k| {
                r.min[k] + r.size[k] + spacing <= room.min[k]
                    || room.min[k] + room.size[k] + spacing <= r.min[k]
            })
        };
        if rooms.iter().all(apart) {
            rooms.push(room);
        }
    }
    rooms
}

fn distance2(a: &Room, b: &Room) -> i64 {
    let (p, q) = (a.center(), b.center());
    (0..2).map(|k| i64::from(p[k] - q[k]).pow(2)).sum::<i64>()
}

/// Gabriel graph edges, shortest first: a and b connect when no other centre lies
/// inside the circle whose diameter is the segment between them.
fn gabriel(rooms: &[Room]) -> Vec<[usize; 2]> {
    let centers: Vec<[i64; 2]> = rooms.iter().map(|r| r.center().map(i64::from)).collect();
    let mut edges = Vec::new();
    for a in 0..rooms.len() {
        for b in a + 1..rooms.len() {
            let blocked = (0..rooms.len()).any(|c| {
                c != a
                    && c != b
                    && (0..2)
                        .map(|k| (centers[a][k] - centers[c][k]) * (centers[b][k] - centers[c][k]))
                        .sum::<i64>()
                        < 0
            });
            if !blocked {
                edges.push((distance2(&rooms[a], &rooms[b]), a, b));
            }
        }
    }
    edges.sort();
    edges.into_iter().map(|(_, a, b)| [a, b]).collect()
}

fn find(parent: &mut [usize], mut x: usize) -> usize {
    while parent[x] != x {
        parent[x] = parent[parent[x]];
        x = parent[x];
    }
    x
}

/// Kruskal over the candidate edges (already shortest first): the tree, then the rest.
fn spanning_tree(count: usize, edges: &[[usize; 2]]) -> (Vec<[usize; 2]>, Vec<[usize; 2]>) {
    let mut parent: Vec<usize> = (0..count).collect();
    let (mut tree, mut rest) = (Vec::new(), Vec::new());
    for &[a, b] in edges {
        let (ra, rb) = (find(&mut parent, a), find(&mut parent, b));
        if ra == rb {
            rest.push([a, b]);
        } else {
            parent[ra] = rb;
            tree.push([a, b]);
        }
    }
    (tree, rest)
}

pub(crate) struct Grid {
    size: [i32; 2],
    pub(crate) cells: Vec<Cell>,
    /// For each cell, the rooms whose one-cell ring covers it (rooms have at most two
    /// rings overlapping thanks to the spacing, but corners can see more).
    rings: Vec<Vec<u16>>,
}

impl Grid {
    pub(crate) fn new(size: [i32; 2], rooms: &[Room]) -> Self {
        let count = (size[0] * size[1]) as usize;
        let mut grid = Self {
            size,
            cells: vec![Cell::Empty; count],
            rings: vec![Vec::new(); count],
        };
        for (index, room) in rooms.iter().enumerate() {
            for y in room.min[1] - 1..=room.min[1] + room.size[1] {
                for x in room.min[0] - 1..=room.min[0] + room.size[0] {
                    if let Some(i) = grid.index([x, y]) {
                        if room.contains([x, y]) {
                            grid.cells[i] = Cell::Room(index as u16);
                        } else {
                            grid.rings[i].push(index as u16);
                        }
                    }
                }
            }
        }
        grid
    }
    fn index(&self, p: [i32; 2]) -> Option<usize> {
        (p[0] >= 0 && p[1] >= 0 && p[0] < self.size[0] && p[1] < self.size[1])
            .then(|| (p[1] * self.size[0] + p[0]) as usize)
    }
    pub(crate) fn point(&self, index: usize) -> [i32; 2] {
        [index as i32 % self.size[0], index as i32 / self.size[0]]
    }
    /// The step cost of entering a cell on a corridor between rooms `a` and `b`, or None
    /// where a corridor may not go.
    fn cost(&self, index: usize, a: u16, b: u16) -> Option<u32> {
        let p = self.point(index);
        match self.cells[index] {
            Cell::Room(r) if r == a || r == b => return Some(2),
            Cell::Room(_) => return None,
            Cell::Corridor => return Some(4),
            Cell::Empty => {}
        }
        if p[0] == 0 || p[1] == 0 || p[0] == self.size[0] - 1 || p[1] == self.size[1] - 1 {
            return None;
        }
        let ring = &self.rings[index];
        if ring.iter().any(|r| *r != a && *r != b) {
            return None;
        }
        // Crossing an end room's ring is fine; running along it would open a long
        // side of the room onto the corridor.
        let mut cost = if ring.is_empty() { 10 } else { 30 };
        // Discourage corridors that hug another corridor without joining it.
        let beside = [[1, 0], [-1, 0], [0, 1], [0, -1]].iter().any(|d| {
            self.index([p[0] + d[0], p[1] + d[1]])
                .is_some_and(|n| self.cells[n] == Cell::Corridor)
        });
        if beside {
            cost += 6;
        }
        Some(cost)
    }
    /// Routes and carves a corridor between rooms `a` and `b`, returning the route's
    /// cells from inside `a` to inside `b`; None when blocked.
    pub(crate) fn route(
        &mut self,
        rooms: &[Room],
        a: usize,
        b: usize,
        width: i32,
        random: &mut Random,
    ) -> Option<Vec<usize>> {
        const DIRECTIONS: [[i32; 2]; 4] = [[1, 0], [0, 1], [-1, 0], [0, -1]];
        let (ra, rb) = (a as u16, b as u16);
        let start = self.index(rooms[a].center())?;
        let target = rooms[b].center();
        // A random preferred first axis varies otherwise identical L-shaped choices.
        let bias = random.below(2) as usize;
        let count = self.cells.len();
        let states = count * 5;
        let mut best = vec![u32::MAX; states];
        let mut from = vec![usize::MAX; states];
        let mut heap = BinaryHeap::new();
        let state = |cell: usize, dir: usize| cell * 5 + dir;
        best[state(start, 4)] = 0;
        let mut order = 0u32;
        heap.push(Reverse((0u32, order, state(start, 4))));
        let heuristic =
            |p: [i32; 2]| ((p[0] - target[0]).abs() + (p[1] - target[1]).abs()) as u32 * 2;
        let mut end = None;
        while let Some(Reverse((_, _, current))) = heap.pop() {
            let (cell, dir) = (current / 5, current % 5);
            if self.cells[cell] == Cell::Room(rb) {
                end = Some(current);
                break;
            }
            let g = best[current];
            let p = self.point(cell);
            for k in 0..4 {
                let turn = (k + bias) % 4;
                let d = DIRECTIONS[turn];
                let Some(next) = self.index([p[0] + d[0], p[1] + d[1]]) else {
                    continue;
                };
                let Some(step) = self.cost(next, ra, rb) else {
                    continue;
                };
                let penalty = if dir != 4 && dir != turn { 5 } else { 0 };
                let total = g + step + penalty;
                let key = state(next, turn);
                if total < best[key] {
                    best[key] = total;
                    from[key] = current;
                    order += 1;
                    heap.push(Reverse((total + heuristic(self.point(next)), order, key)));
                }
            }
        }
        let mut current = end?;
        let mut path = Vec::new();
        while current != usize::MAX {
            path.push(current / 5);
            current = from[current];
        }
        path.reverse();
        for &cell in &path {
            if self.cells[cell] != Cell::Empty {
                continue;
            }
            let p = self.point(cell);
            for dy in 0..width {
                for dx in 0..width {
                    if let Some(i) = self.index([p[0] + dx, p[1] + dy])
                        && self.cells[i] == Cell::Empty
                        && self.cost(i, ra, rb).is_some()
                    {
                        self.cells[i] = Cell::Corridor;
                    }
                }
            }
        }
        Some(path)
    }
    /// A group number per room: rooms reachable from each other share one.
    fn groups(&self, rooms: &[Room]) -> Vec<usize> {
        let reached = self.flood_all();
        rooms
            .iter()
            .map(|r| reached[self.index(r.center()).expect("room inside grid")])
            .collect()
    }
    fn flood_all(&self) -> Vec<usize> {
        let mut group = vec![usize::MAX; self.cells.len()];
        let mut next = 0;
        for start in 0..self.cells.len() {
            if self.cells[start] == Cell::Empty || group[start] != usize::MAX {
                continue;
            }
            let mut stack = vec![start];
            group[start] = next;
            while let Some(cell) = stack.pop() {
                let p = self.point(cell);
                for d in [[1, 0], [0, 1], [-1, 0], [0, -1]] {
                    let Some(n) = self.index([p[0] + d[0], p[1] + d[1]]) else {
                        continue;
                    };
                    if group[n] != usize::MAX || !joined(self.cells[cell], self.cells[n]) {
                        continue;
                    }
                    group[n] = next;
                    stack.push(n);
                }
            }
            next += 1;
        }
        group
    }
    /// Clears every walkable cell outside group `main` of the rooms' flood.
    fn keep(&mut self, rooms: &[Room], groups: &[usize], main: usize) {
        let reached = self.flood_all();
        let main_group = rooms
            .iter()
            .zip(groups)
            .find(|(_, g)| **g == main)
            .map(|(r, _)| reached[self.index(r.center()).expect("room inside grid")]);
        for (cell, g) in self.cells.iter_mut().zip(reached) {
            if Some(g) != main_group {
                *cell = Cell::Empty;
            }
        }
    }
    fn renumber(&mut self, rooms: &[Room]) {
        for i in 0..self.cells.len() {
            if let Cell::Room(_) = self.cells[i] {
                let p = self.point(i);
                let index = rooms.iter().position(|r| r.contains(p));
                self.cells[i] = index.map_or(Cell::Empty, |r| Cell::Room(r as u16));
            }
        }
    }
}

fn joined(a: Cell, b: Cell) -> bool {
    match (a, b) {
        (Cell::Empty, _) | (_, Cell::Empty) => false,
        (Cell::Room(x), Cell::Room(y)) => x == y,
        _ => true,
    }
}

/// Room pairs joined by a corridor network, and the number of independent loops.
fn connections(layout: &Layout) -> (Vec<[usize; 2]>, usize) {
    let [w, h] = layout.size;
    let mut network = vec![usize::MAX; layout.cells.len()];
    let mut touched: Vec<BTreeSet<usize>> = Vec::new();
    for start in 0..layout.cells.len() {
        if layout.cells[start] != Cell::Corridor || network[start] != usize::MAX {
            continue;
        }
        let id = touched.len();
        touched.push(BTreeSet::new());
        network[start] = id;
        let mut stack = vec![start];
        while let Some(cell) = stack.pop() {
            let p = [cell as i32 % w, cell as i32 / w];
            for d in [[1, 0], [0, 1], [-1, 0], [0, -1]] {
                let q = [p[0] + d[0], p[1] + d[1]];
                if q[0] < 0 || q[1] < 0 || q[0] >= w || q[1] >= h {
                    continue;
                }
                let n = (q[1] * w + q[0]) as usize;
                match layout.cells[n] {
                    Cell::Corridor if network[n] == usize::MAX => {
                        network[n] = id;
                        stack.push(n);
                    }
                    Cell::Room(r) => {
                        touched[id].insert(usize::from(r));
                    }
                    _ => {}
                }
            }
        }
    }
    let mut links = BTreeSet::new();
    let mut incidences = 0;
    for rooms in &touched {
        incidences += rooms.len();
        let list: Vec<usize> = rooms.iter().copied().collect();
        for (i, a) in list.iter().enumerate() {
            for b in &list[i + 1..] {
                links.insert([*a, *b]);
            }
        }
    }
    // Rooms and networks form a connected bipartite graph; its cycle rank counts loops.
    let vertices = layout.rooms.len() + touched.iter().filter(|t| !t.is_empty()).count();
    let loops = (incidences + 1).saturating_sub(vertices);
    (links.into_iter().collect(), loops)
}

/// The two rooms furthest apart along the connections, spawn first.
fn ends(layout: &Layout, random: &mut Random) -> (usize, usize) {
    let count = layout.rooms.len();
    let mut adjacent = vec![Vec::new(); count];
    for &[a, b] in &layout.links {
        adjacent[a].push(b);
        adjacent[b].push(a);
    }
    let mut best = (0usize, 0i64, 0usize, 0usize);
    for start in 0..count {
        let mut hops = vec![usize::MAX; count];
        hops[start] = 0;
        let mut queue = std::collections::VecDeque::from([start]);
        while let Some(room) = queue.pop_front() {
            for &next in &adjacent[room] {
                if hops[next] == usize::MAX {
                    hops[next] = hops[room] + 1;
                    queue.push_back(next);
                }
            }
        }
        for (end, &hop) in hops.iter().enumerate().skip(start + 1) {
            if hop == usize::MAX {
                continue;
            }
            let score = (hop, distance2(&layout.rooms[start], &layout.rooms[end]));
            if score > (best.0, best.1) {
                best = (score.0, score.1, start, end);
            }
        }
    }
    let (a, b) = (best.2, best.3);
    if random.chance(50) { (a, b) } else { (b, a) }
}
