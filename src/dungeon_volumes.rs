//! Generation volumes: rectangles on a generator's grid that shape its layout, to fit a
//! dungeon around art made by hand or into a given area. They live in the generator's
//! settings, so nothing of them reaches the console.
//!
//! - Keep out: no room or corridor uses its cells.
//! - Only here: with any of these, rooms and corridors stay inside them.
//! - Room here: a room fills it, besides the rooms the settings scatter.
//! - Mirror: rooms scattered in its first half are repeated in its second half.
//!
//! Recipe layouts follow Keep out and Only here: a lattice slot that touches a blocked
//! cell holds no room and no corridor. The grid is the generator's whole grid; a recipe
//! layout with volumes is not cropped, so volumes stay where they were drawn.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    #[default]
    KeepOut,
    OnlyHere,
    Room,
    /// Repeats rooms left to right.
    MirrorX,
    /// Repeats rooms front to back (top to bottom in 2D).
    MirrorY,
}
impl Kind {
    pub const ALL: [Kind; 5] = [
        Kind::KeepOut,
        Kind::OnlyHere,
        Kind::Room,
        Kind::MirrorX,
        Kind::MirrorY,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Kind::KeepOut => "Keep out",
            Kind::OnlyHere => "Only here",
            Kind::Room => "Room here",
            Kind::MirrorX => "Mirror left-right",
            Kind::MirrorY => "Mirror front-back",
        }
    }
    pub fn tip(self) -> &'static str {
        match self {
            Kind::KeepOut => "No room or corridor uses these cells: room for art made by hand.",
            Kind::OnlyHere => "Rooms and corridors stay inside the Only here volumes.",
            Kind::Room => {
                "A room fills this rectangle, besides the rooms the settings scatter. Rooms and corridors layouts only."
            }
            Kind::MirrorX => {
                "Rooms scattered in the left half are repeated in the right half. Rooms and corridors layouts only."
            }
            Kind::MirrorY => {
                "Rooms scattered in the front half are repeated in the back half. Rooms and corridors layouts only."
            }
        }
    }
    /// Recipes follow only the volumes that block cells.
    pub fn shapes_recipes(self) -> bool {
        matches!(self, Kind::KeepOut | Kind::OnlyHere)
    }
    pub fn color(self) -> [f32; 4] {
        match self {
            Kind::KeepOut => [1., 0.4, 0.35, 1.],
            Kind::OnlyHere => [0.4, 0.85, 1., 1.],
            Kind::Room => [0.5, 0.9, 0.45, 1.],
            Kind::MirrorX | Kind::MirrorY => [0.85, 0.6, 1., 1.],
        }
    }
}

/// A rectangle of grid cells: from `min`, `size` cells across and deep.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Volume {
    pub kind: Kind,
    pub min: [i32; 2],
    pub size: [i32; 2],
}
impl Volume {
    pub fn contains(&self, p: [i32; 2]) -> bool {
        (0..2).all(|k| p[k] >= self.min[k] && p[k] < self.min[k] + self.size[k])
    }
    pub fn max(&self) -> [i32; 2] {
        [self.min[0] + self.size[0], self.min[1] + self.size[1]]
    }
    /// The same rectangle with its corners in order and at least one cell a side.
    pub fn normalized(a: [i32; 2], b: [i32; 2], kind: Kind) -> Volume {
        let min = [a[0].min(b[0]), a[1].min(b[1])];
        Volume {
            kind,
            min,
            size: [(a[0] - b[0]).abs() + 1, (a[1] - b[1]).abs() + 1],
        }
    }
}

/// Whether each cell of a `size` grid is closed to rooms and corridors: inside a Keep
/// out volume, or outside every Only here volume when there is one. Room here volumes
/// stay open.
pub fn blocked(volumes: &[Volume], size: [i32; 2]) -> Vec<bool> {
    let only: Vec<&Volume> = volumes
        .iter()
        .filter(|v| v.kind == Kind::OnlyHere)
        .collect();
    let mut cells = vec![false; (size[0].max(0) * size[1].max(0)) as usize];
    for y in 0..size[1] {
        for x in 0..size[0] {
            let p = [x, y];
            let out = volumes
                .iter()
                .any(|v| v.kind == Kind::KeepOut && v.contains(p))
                || (!only.is_empty() && !only.iter().any(|v| v.contains(p)));
            let room = volumes
                .iter()
                .any(|v| v.kind == Kind::Room && v.contains(p));
            cells[(y * size[0] + x) as usize] = out && !room;
        }
    }
    cells
}

/// Volumes must lie on the grid and leave something to build on.
pub fn validate(volumes: &[Volume], size: [i32; 2]) -> Result<(), String> {
    if volumes.len() > 32 {
        return Err("A generator holds up to 32 volumes".into());
    }
    for (i, v) in volumes.iter().enumerate() {
        if v.size[0] < 1 || v.size[1] < 1 {
            return Err(format!("Volume {} is empty", i + 1));
        }
        let max = v.max();
        if v.min[0] < 0 || v.min[1] < 0 || max[0] > size[0] || max[1] > size[1] {
            return Err(format!(
                "Volume {} ({}) reaches past the {}x{} grid",
                i + 1,
                v.kind.label(),
                size[0],
                size[1]
            ));
        }
        if v.kind == Kind::Room && (v.size[0] < 2 || v.size[1] < 2) {
            return Err(format!(
                "Room here volume {} is smaller than 2x2 cells",
                i + 1
            ));
        }
    }
    if blocked(volumes, size).iter().all(|b| *b) {
        return Err("The volumes leave no cell to build on".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dungeon::{Cell, Layout, RoomSettings};

    fn walkable(layout: &Layout) -> Vec<[i32; 2]> {
        (0..layout.size[1])
            .flat_map(|y| (0..layout.size[0]).map(move |x| [x, y]))
            .filter(|p| layout.at(*p).walkable())
            .collect()
    }

    /// Rooms and corridors: nothing enters a Keep out volume, everything stays in the
    /// Only here ones, a Room here volume holds a room, and a Mirror volume repeats the
    /// rooms of its first half in its second.
    #[test]
    fn rooms_and_corridors_follow_every_kind() {
        let settings = RoomSettings {
            size: [48, 40],
            rooms: 10,
            ..Default::default()
        };
        let out = Volume {
            kind: Kind::KeepOut,
            min: [14, 10],
            size: [16, 16],
        };
        let only = Volume {
            kind: Kind::OnlyHere,
            min: [0, 0],
            size: [30, 40],
        };
        let room = Volume {
            kind: Kind::Room,
            min: [36, 4],
            size: [6, 5],
        };
        for seed in 1..=10 {
            let plain = crate::dungeon_rooms::generate(&settings, seed, &[]).unwrap();
            let layout = crate::dungeon_rooms::generate(&settings, seed, &[out]).unwrap();
            assert!(
                walkable(&layout).iter().all(|p| !out.contains(*p)),
                "seed {seed}"
            );
            assert!(walkable(&plain).iter().any(|p| out.contains(*p)) || seed > 1);
            let layout = crate::dungeon_rooms::generate(&settings, seed, &[only]).unwrap();
            assert!(
                walkable(&layout).iter().all(|p| only.contains(*p)),
                "seed {seed}"
            );
            let layout = crate::dungeon_rooms::generate(&settings, seed, &[room]).unwrap();
            assert!(
                layout
                    .rooms
                    .iter()
                    .any(|r| r.min == room.min && r.size == room.size),
                "seed {seed}: no room fills the volume"
            );
            // Mirrored rooms come in pairs across the middle of the whole grid.
            let mirror = Volume {
                kind: Kind::MirrorX,
                min: [0, 0],
                size: [48, 40],
            };
            let layout = crate::dungeon_rooms::generate(&settings, seed, &[mirror]).unwrap();
            let pairs = layout
                .rooms
                .iter()
                .filter(|r| r.min[0] + r.size[0] <= 24)
                .filter(|r| {
                    layout.rooms.iter().any(|m| {
                        m.min == [48 - (r.min[0] + r.size[0]), r.min[1]] && m.size == r.size
                    })
                })
                .count();
            assert!(pairs >= 1, "seed {seed}: no mirrored pair");
            assert!(
                layout.rooms.iter().all(|r| r.min[0] + r.size[0] <= 24
                    || layout
                        .rooms
                        .iter()
                        .any(|m| m.min == [48 - (r.min[0] + r.size[0]), r.min[1]])),
                "seed {seed}: a room in the second half has no first-half twin"
            );
        }
    }

    /// Recipes: slots touching a Keep out volume hold nothing, the layout keeps the
    /// whole grid so volumes stay put, and with no volume nothing changes.
    #[test]
    fn recipes_leave_closed_slots_empty_on_the_whole_grid() {
        let recipe = crate::dungeon_recipe::builtin();
        let size = recipe.grid_size();
        let (lattice, span) = recipe.lattice();
        // One slot in the middle of the lattice, and a corner slot.
        let slot = |x: i32, y: i32| Volume {
            kind: Kind::KeepOut,
            min: [1 + x * span + 2, 1 + y * span + 2],
            size: [2, 2],
        };
        let volumes = [slot(lattice[0] / 2, lattice[1] / 2), slot(0, 0)];
        let shapes = Default::default();
        for seed in 1..=12 {
            let layout =
                crate::dungeon_recipe::layout_with(&recipe, seed, &shapes, &volumes).unwrap();
            assert_eq!(layout.size, size, "seed {seed}: cropped");
            for v in &volumes {
                let sx = (v.min[0] - 1) / span;
                let sy = (v.min[1] - 1) / span;
                for y in 1 + sy * span..1 + (sy + 1) * span {
                    for x in 1 + sx * span..1 + (sx + 1) * span {
                        assert_eq!(layout.at([x, y]), Cell::Empty, "seed {seed}: {x},{y}");
                    }
                }
            }
            let plain = crate::dungeon_recipe::layout_with(&recipe, seed, &shapes, &[]).unwrap();
            assert_eq!(plain, crate::dungeon_recipe::layout(&recipe, seed).unwrap());
        }
    }

    #[test]
    fn keep_out_and_only_here_block_cells_and_rooms_stay_open() {
        let size = [10, 8];
        let at = |cells: &[bool], p: [i32; 2]| cells[(p[1] * size[0] + p[0]) as usize];
        assert!(blocked(&[], size).iter().all(|b| !b));
        let out = Volume {
            kind: Kind::KeepOut,
            min: [2, 2],
            size: [3, 2],
        };
        let cells = blocked(&[out], size);
        assert!(at(&cells, [2, 2]) && at(&cells, [4, 3]) && !at(&cells, [5, 3]));
        let only = Volume {
            kind: Kind::OnlyHere,
            min: [0, 0],
            size: [5, 8],
        };
        let room = Volume {
            kind: Kind::Room,
            min: [7, 1],
            size: [2, 2],
        };
        let cells = blocked(&[only, room], size);
        assert!(!at(&cells, [0, 0]) && at(&cells, [6, 0]) && !at(&cells, [7, 1]));
        assert!(validate(&[only, room], size).is_ok());
        assert!(
            validate(
                &[Volume {
                    kind: Kind::KeepOut,
                    min: [8, 0],
                    size: [3, 1]
                }],
                size
            )
            .is_err()
        );
        assert!(
            validate(
                &[Volume {
                    kind: Kind::KeepOut,
                    min: [0, 0],
                    size
                }],
                size
            )
            .is_err()
        );
        assert_eq!(
            Volume::normalized([4, 1], [2, 3], Kind::Room),
            Volume {
                kind: Kind::Room,
                min: [2, 1],
                size: [3, 3]
            }
        );
    }
}
