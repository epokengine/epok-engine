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

pub const CATALOG: [Info; 3] = [
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
    /// What the selected flavor will actually write for this template.
    pub fn gameplay_note(&self, flavor: GameplayFlavor) -> &'static str {
        if self.template == Template::Basic {
            return "Basic generates no gameplay source. The choice is remembered as the style you \
prefer to start in; every project can use all three.";
        }
        match flavor {
            GameplayFlavor::Cpp => {
                "Generates project-owned C++ source under assets/scripts, compiled for the console \
with the rest of the project."
            }
            GameplayFlavor::Blueprint => {
                "Generates a project-owned visual graph under assets/Blueprints. The build backend \
turns the graph into native console code; what you edit stays visual."
            }
            GameplayFlavor::Lua => {
                "Generates project-owned Lua under assets/scripts, built with the project's Lua \
execution setting. It starts on the ahead-of-time native mode, which keeps the console build small."
            }
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
