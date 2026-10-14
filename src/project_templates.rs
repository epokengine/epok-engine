//! What the Hub shows about each starter template. Display text and preview
//! artwork live here so the New Project view stays a layout, and so `workspace`
//! and the template generators never depend on the UI toolkit.
use crate::workspace::{GameplayFlavor, Template};

pub struct Info {
    pub template: Template,
    pub title: &'static str,
    /// One line, shown on the card.
    pub summary: &'static str,
    /// A paragraph, shown beside the large preview.
    pub description: &'static str,
    pub features: &'static [&'static str],
    preview: &'static [u8],
}

pub const CATALOG: [Info; 5] = [
    Info {
        template: Template::Basic,
        title: "Basic",
        summary: "An empty scene and a camera.",
        description: "One scene, one camera, nothing else. Use it when the game is yours from \
the first actor.",
        features: &[
            "One empty scene with a camera",
            "Console rendering defaults ready to build",
            "No starter gameplay in any language",
        ],
        preview: include_bytes!("../resources/templates/previews/basic.png"),
    },
    Info {
        template: Template::Sample,
        title: "Sample",
        summary: "A small scene with one behaviour.",
        description: "A ground plane, two cubes and a camera, with a Spinner behaviour on one \
cube: the shortest path to a class of your own running on the console.",
        features: &[
            "Ground, two cubes and a camera",
            "A Spinner behaviour with an editable speed",
            "The same behaviour in C++, Blueprint or Lua",
        ],
        preview: include_bytes!("../resources/templates/previews/sample.png"),
    },
    Info {
        template: Template::ThirdPerson,
        title: "Third Person",
        summary: "Animated player and orbit camera.",
        description: "A playable arena with an animated character, camera-relative movement, \
jumping, collision and an orbit camera that avoids level geometry. No combat and no audio: a \
starting point, not a demo.",
        features: &[
            "Optimized arena within the console geometry budget",
            "Idle, walk, run, jump, fall and land animation",
            "Camera-relative movement, jumping and step handling",
            "Orbit camera with collision-aware boom",
        ],
        preview: include_bytes!("../resources/templates/previews/third-person.png"),
    },
    Info {
        template: Template::Platformer2D,
        title: "Platformer 2D",
        summary: "A side-scroller built from 2D nodes.",
        description: "A playable 2D level at 320 x 240: an animated character that runs and \
jumps with a following camera, solid ground and crates, one-way platforms and spinning coins \
to collect. Every piece is an ordinary 2D node you can select, move and edit in the 2D view.",
        features: &[
            "CharacterBody2D player with idle and run animations",
            "Camera2D with smoothing and level limits",
            "Solid ground, crates and one-way platforms",
            "Spinning coins collected through trigger shapes",
        ],
        preview: include_bytes!("../resources/templates/previews/platformer-2d.png"),
    },
    Info {
        template: Template::Dungeon,
        title: "Dungeon",
        summary: "Third-person hero in a generated dungeon.",
        description: "The Third Person character and camera in a generated sci-fi dungeon: rooms \
in different finishes with columns along the walls, a main path to the goal, side branches with \
treasure, one-way shortcuts and a locked door whose key waits elsewhere. Press New seed in the \
Inspector for another dungeon, or copy the recipe and style to make it yours.",
        features: &[
            "Dungeon Generator with the built-in recipe and sci-fi style",
            "A working key and locked door in the chosen flavor",
            "Walls, ceiling and doors sized to the character",
            "Animated player, orbit camera and console-ready lighting",
        ],
        preview: include_bytes!("../resources/templates/previews/dungeon.png"),
    },
];

pub fn info(template: Template) -> &'static Info {
    CATALOG
        .iter()
        .find(|entry| entry.template == template)
        .expect("every template is catalogued")
}

impl Info {
    /// RGBA pixels and dimensions, or `None` when the artwork cannot be decoded.
    /// A failed preview is a missing picture, never a failed engine start.
    pub fn preview(&self) -> Option<(Vec<u8>, u32, u32)> {
        crate::branding::decode(self.preview).ok()
    }
    /// Every template accepts every flavor today. The check exists so an
    /// unimplemented pair can be disabled in one place instead of silently
    /// generating something the author did not choose.
    pub fn supports(&self, _flavor: GameplayFlavor) -> bool {
        true
    }
    /// What the selected flavor will actually write for this template. One
    /// short line: it sits under the two lists, not in place of the manual.
    pub fn gameplay_note(&self, flavor: GameplayFlavor) -> &'static str {
        if self.template == Template::Basic {
            return "Basic writes no gameplay. Every project can use all three.";
        }
        match flavor {
            GameplayFlavor::Cpp => "Writes project-owned C++ under assets/scripts.",
            GameplayFlavor::Blueprint => "Writes a project-owned graph under assets/Blueprints.",
            GameplayFlavor::Lua => "Writes project-owned Lua, built ahead of time by default.",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_template_is_described_and_previewable() {
        for template in Template::ALL {
            let info = info(template);
            assert!(!info.title.is_empty());
            assert!(!info.summary.is_empty());
            assert!(info.description.len() > 40);
            assert!(!info.features.is_empty());
            let (pixels, width, height) = info.preview().expect("preview decodes");
            assert_eq!(pixels.len(), (width * height * 4) as usize);
            // 16:9 keeps the details panel's aspect stable at every Hub size.
            assert_eq!(width * 9, height * 16, "{} preview is not 16:9", info.title);
            for flavor in GameplayFlavor::ALL {
                assert!(!info.gameplay_note(flavor).is_empty());
                assert!(info.supports(flavor));
            }
        }
    }
}
