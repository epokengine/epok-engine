//! The dungeon minimap: what a generation bakes for it (the walkable cells of the
//! dungeon's plane as rectangles of rooms and corridors), the generator's settings for
//! it, and how a map cooks both for `runtime/dungeon_minimap.hpp`. On the console the
//! map fills in as the player explores: a room shows once entered, with dim stubs of
//! the corridors leading out of it, and a corridor once walked.
use crate::dungeon::{Cell, Layout, Side};
use serde::{Deserialize, Serialize};

/// The most rectangles a minimap keeps; the runtime marks each explored with one bit.
pub const MAX_RECTS: usize = 255;
/// Grid cells a side; rectangles store their cells in bytes.
pub const MAX_CELLS: i32 = 255;

/// One rectangle of walkable cells, in cells from the minimap grid's corner.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Rect {
    pub x: u8,
    pub y: u8,
    pub w: u8,
    pub h: u8,
    /// The room it belongs to; -1 for corridors.
    pub room: i8,
    /// Rooms a corridor rectangle opens into; -1 when none.
    pub near: [i8; 2],
}

/// What a generation bakes for the minimap.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Baked {
    /// Cells across and deep.
    pub size: [u16; 2],
    /// The grid's corner and one cell's step on the dungeon's plane, in the generator
    /// root's space: (x, z) in 3D, pixels (x, y) in 2D.
    pub corner: [f32; 2],
    pub step: [f32; 2],
    pub rects: Vec<Rect>,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum Corner {
    TopLeft,
    #[default]
    TopRight,
    BottomLeft,
    BottomRight,
}
impl Corner {
    pub const ALL: [Corner; 4] = [
        Corner::TopLeft,
        Corner::TopRight,
        Corner::BottomLeft,
        Corner::BottomRight,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Corner::TopLeft => "Top left",
            Corner::TopRight => "Top right",
            Corner::BottomLeft => "Bottom left",
            Corner::BottomRight => "Bottom right",
        }
    }
}

/// The colours a minimap draws with, in the runtime's `MinimapColor` order.
pub const COLOR_NAMES: [&str; 8] = [
    "Rooms",
    "Corridors",
    "Seen",
    "Backdrop",
    "Player",
    "Keys",
    "Locked doors",
    "Goal",
];

/// A generator's minimap: whether the map has one and how it draws.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Settings {
    pub enabled: bool,
    /// Shown when the map starts; game code shows and hides it.
    pub visible: bool,
    pub corner: Corner,
    /// Pixels from the screen's edges to the box.
    pub margin: [u16; 2],
    /// The box in pixels.
    pub size: [u16; 2],
    /// Keep the player in the box's middle at `cell` pixels a cell, instead of fitting
    /// the whole dungeon in the box.
    pub follow: bool,
    pub cell: u8,
    /// Show the whole dungeon from the start instead of what the player explored.
    pub everything: bool,
    pub colors: [[u8; 3]; 8],
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: false,
            visible: true,
            corner: Corner::TopRight,
            margin: [12, 12],
            size: [96, 72],
            follow: false,
            cell: 3,
            everything: false,
            colors: [
                [96, 200, 210],
                [70, 140, 150],
                [40, 70, 80],
                [16, 16, 24],
                [255, 255, 255],
                [255, 210, 40],
                [230, 50, 50],
                [90, 230, 110],
            ],
        }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if !(16..=240).contains(&self.size[0]) || !(16..=200).contains(&self.size[1]) {
            return Err("The minimap box is 16-240 pixels wide and 16-200 high".into());
        }
        if !(1..=8).contains(&self.cell) {
            return Err("The minimap draws 1-8 pixels a cell".into());
        }
        if self.margin[0] > 200 || self.margin[1] > 200 {
            return Err("The minimap's margin is at most 200 pixels".into());
        }
        Ok(())
    }
    /// The runtime's `MinimapFlag` bits.
    pub fn flags(&self) -> u8 {
        let mut flags = 0;
        if self.visible {
            flags |= 1;
        }
        if self.follow {
            flags |= 2;
        }
        if self.everything {
            flags |= 4;
        }
        if matches!(self.corner, Corner::TopRight | Corner::BottomRight) {
            flags |= 16;
        }
        if matches!(self.corner, Corner::BottomLeft | Corner::BottomRight) {
            flags |= 32;
        }
        flags
    }
    /// The box's pixels a cell when it fits the whole dungeon.
    pub fn fit(&self, baked: &Baked) -> u32 {
        let across = u32::from(self.size[0]) / u32::from(baked.size[0].max(1));
        let deep = u32::from(self.size[1]) / u32::from(baked.size[1].max(1));
        across.min(deep).max(1)
    }
}

/// Bakes the minimap of a layout: the bounding box of its walkable cells, each room's
/// cells (rooms from `rooms` on count as corridors) and the corridors' cells as
/// rectangles. `corner` is a grid lattice point on the dungeon's plane.
/// `up` puts the grid's last row at the top of the map: in 3D, where the grid's rows
/// run towards +Z, the direction the camera and the Scene view look up the screen.
pub fn build(
    layout: &Layout,
    rooms: usize,
    corner: &dyn Fn(i32, i32) -> [f32; 2],
    up: bool,
) -> (Baked, Vec<String>) {
    let mut notes = Vec::new();
    let walkable: Vec<[i32; 2]> = (0..layout.size[1])
        .flat_map(|y| (0..layout.size[0]).map(move |x| [x, y]))
        .filter(|p| layout.at(*p).walkable())
        .collect();
    let Some(lo) = walkable
        .iter()
        .copied()
        .reduce(|a, b| [a[0].min(b[0]), a[1].min(b[1])])
    else {
        return (Baked::default(), notes);
    };
    let hi = walkable
        .iter()
        .copied()
        .fold(lo, |a, b| [a[0].max(b[0] + 1), a[1].max(b[1] + 1)]);
    let size = [hi[0] - lo[0], hi[1] - lo[1]];
    if size[0] > MAX_CELLS || size[1] > MAX_CELLS {
        notes.push(format!(
            "No minimap: the dungeon spans {}x{} cells, more than {MAX_CELLS}",
            size[0], size[1]
        ));
        return (Baked::default(), notes);
    }
    let room_of = |p: [i32; 2]| match layout.at(p) {
        Cell::Room(r) if usize::from(r) < rooms => Some(usize::from(r)),
        _ => None,
    };
    let region = [lo[0], lo[1], hi[0], hi[1]];
    let mut rects = Vec::new();
    let mut push = |r: [i32; 4], room: i8, near: [i8; 2]| {
        rects.push(Rect {
            x: (r[0] - lo[0]) as u8,
            y: if up { hi[1] - r[3] } else { r[1] - lo[1] } as u8,
            w: (r[2] - r[0]) as u8,
            h: (r[3] - r[1]) as u8,
            room,
            near,
        });
    };
    for room in 0..rooms.min(layout.rooms.len()) {
        for r in crate::dungeon_bake::rectangles(region, &|p| room_of(p) == Some(room)) {
            push(r, room as i8, [-1, -1]);
        }
    }
    let corridor = |p: [i32; 2]| layout.at(p).walkable() && room_of(p).is_none();
    for r in crate::dungeon_bake::rectangles(region, &corridor) {
        // The rooms its edge cells open into.
        let mut near = [-1i8; 2];
        let mut found = 0;
        for y in r[1]..r[3] {
            for x in r[0]..r[2] {
                for side in Side::ALL {
                    let d = side.delta();
                    let q = [x + d[0], y + d[1]];
                    if let Some(room) = room_of(q)
                        && layout.open([x, y], side)
                        && !near.contains(&(room as i8))
                        && found < 2
                    {
                        near[found] = room as i8;
                        found += 1;
                    }
                }
            }
        }
        push(r, -1, near);
    }
    if rects.len() > MAX_RECTS {
        notes.push(format!(
            "The minimap keeps {MAX_RECTS} of {} rectangles; caves and odd rooms take many",
            rects.len()
        ));
        rects.truncate(MAX_RECTS);
    }
    // The map's first row: the grid's first, or its last when the map looks up the grid.
    let (origin, next) = if up {
        (corner(lo[0], hi[1]), corner(lo[0] + 1, hi[1] - 1))
    } else {
        (corner(lo[0], lo[1]), corner(lo[0] + 1, lo[1] + 1))
    };
    (
        Baked {
            size: [size[0] as u16, size[1] as u16],
            corner: origin,
            step: [next[0] - origin[0], next[1] - origin[1]],
            rects,
        },
        notes,
    )
}

/// Bytes the console spends on a minimap of `rects` rectangles: the table, the explored
/// bits, the double-buffered packets for rectangles and icons, and the drawing code.
pub fn ram_bytes(rects: usize) -> usize {
    if rects == 0 {
        return 0;
    }
    rects * 8 + rects.div_ceil(32) * 4 + 2 * (rects + 41) * 16 + 6 * 1024
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every walkable cell lies in exactly one rectangle, rooms keep their index, and
    /// the corridor rectangles touching a room's doorway name it.
    #[test]
    fn rectangles_cover_the_walkable_cells_once() {
        let recipe = crate::dungeon_recipe::builtin();
        for seed in 1..=8 {
            let layout = crate::dungeon_recipe::layout(&recipe, seed).unwrap();
            let (baked, notes) = build(
                &layout,
                layout.rooms.len(),
                &|x, y| [x as f32 * 3., y as f32 * 3.],
                false,
            );
            assert!(notes.is_empty(), "seed {seed}: {notes:?}");
            let lo = [baked.corner[0] as i32 / 3, baked.corner[1] as i32 / 3];
            assert_eq!(baked.step, [3., 3.]);
            let mut covered = std::collections::BTreeMap::new();
            for (i, r) in baked.rects.iter().enumerate() {
                for y in r.y..r.y + r.h {
                    for x in r.x..r.x + r.w {
                        let p = [lo[0] + i32::from(x), lo[1] + i32::from(y)];
                        assert!(covered.insert(p, i).is_none(), "seed {seed}: {p:?} twice");
                        match layout.at(p) {
                            Cell::Room(room) => assert_eq!(i32::from(r.room), i32::from(room)),
                            Cell::Corridor => assert_eq!(r.room, -1),
                            Cell::Empty => panic!("seed {seed}: {p:?} is rock"),
                        }
                    }
                }
            }
            assert_eq!(covered.len(), layout.walkable_cells(), "seed {seed}");
            // Corridors leaving rooms name them.
            assert!(
                baked.rects.iter().any(|r| r.room < 0 && r.near[0] >= 0),
                "seed {seed}"
            );
            assert!(baked.rects.len() <= MAX_RECTS);
            // Looking up the grid, the rows turn over and the step points back.
            let (up, _) = build(
                &layout,
                layout.rooms.len(),
                &|x, y| [x as f32 * 3., y as f32 * 3.],
                true,
            );
            assert_eq!(up.step, [3., -3.]);
            assert_eq!(up.corner[1], (lo[1] + i32::from(baked.size[1])) as f32 * 3.);
            for (a, b) in baked.rects.iter().zip(&up.rects) {
                assert_eq!(
                    i32::from(b.y),
                    i32::from(baked.size[1]) - i32::from(a.y) - i32::from(a.h)
                );
                assert_eq!((a.x, a.w, a.h, a.room), (b.x, b.w, b.h, b.room));
            }
        }
    }

    #[test]
    fn settings_flags_follow_the_corner() {
        let mut s = Settings::default();
        assert_eq!(s.flags(), 1 | 16);
        s.corner = Corner::BottomLeft;
        s.follow = true;
        s.everything = true;
        s.visible = false;
        assert_eq!(s.flags(), 2 | 4 | 32);
        assert!(s.validate().is_ok());
        s.cell = 0;
        assert!(s.validate().is_err());
    }
}
