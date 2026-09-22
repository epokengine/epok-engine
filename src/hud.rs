use crate::scene::Scene;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Budget {
    pub layouts: usize,
    pub rectangles: usize,
    pub texts: usize,
    pub glyphs: usize,
}
impl Default for Budget {
    fn default() -> Self {
        Self {
            layouts: 128,
            rectangles: 256,
            texts: 64,
            glyphs: 1024,
        }
    }
}
impl Budget {
    pub fn validate(&self) -> Result<(), String> {
        if self.layouts == 0
            || self.layouts > 128
            || self.rectangles == 0
            || self.rectangles > 512
            || self.texts == 0
            || self.texts > 64
            || self.glyphs == 0
            || self.glyphs > 2048
        {
            return Err(
                "HUD budgets: layouts 1..128, rectangles 1..512, texts 1..64, glyphs 1..2048"
                    .into(),
            );
        }
        Ok(())
    }
    pub fn header(&self) -> Result<String, String> {
        self.validate()?;
        Ok(format!(
            "#pragma once\nnamespace epok {{\ninline constexpr unsigned hud_layout_budget={};\ninline constexpr unsigned hud_rectangle_budget={};\ninline constexpr unsigned hud_text_budget={};\ninline constexpr unsigned hud_glyph_budget={};\n}}\n",
            self.layouts, self.rectangles, self.texts, self.glyphs
        ))
    }
}
pub fn stage(build: &std::path::Path, budget: &Budget) -> Result<(), String> {
    crate::project::write_changed(&build.join("hud-config.hh"), budget.header()?.as_bytes())?;
    crate::project::write_changed(
        &build.join("hud-font.hh"),
        crate::bitmap_font::header().as_bytes(),
    )?;
    crate::project::write_changed(
        &build.join("text.hpp"),
        include_bytes!("../runtime/text.hpp"),
    )
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Canvas {
    pub enabled: bool,
}
impl Default for Canvas {
    fn default() -> Self {
        Self { enabled: true }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct RectTransform {
    pub anchor_min: [f32; 2],
    pub anchor_max: [f32; 2],
    pub pivot: [f32; 2],
    pub position: [f32; 2],
    pub size: [f32; 2],
}
impl Default for RectTransform {
    fn default() -> Self {
        Self {
            anchor_min: [0.5; 2],
            anchor_max: [0.5; 2],
            pivot: [0.5; 2],
            position: [0.; 2],
            size: [100., 32.],
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Image {
    pub color: [f32; 3],
    pub enabled: bool,
    pub texture: Option<uuid::Uuid>,
    /// Atlas pixel rectangle. Zero width and height select the entire texture.
    pub region: [u16; 4],
    /// Nine-slice borders in source pixels: left, top, right, bottom.
    pub borders: [u16; 4],
}
impl Default for Image {
    fn default() -> Self {
        Self {
            color: [0.2, 0.4, 0.65],
            enabled: true,
            texture: None,
            region: [0; 4],
            borders: [0; 4],
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Text {
    pub text: String,
    pub color: [f32; 3],
    pub enabled: bool,
    pub wrap: bool,
}
impl Default for Text {
    fn default() -> Self {
        Self {
            text: "New Text".into(),
            color: [1.; 3],
            enabled: true,
            wrap: true,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ProgressBar {
    pub value: f32,
    pub color: [f32; 3],
    pub background: [f32; 3],
    pub enabled: bool,
}
impl Default for ProgressBar {
    fn default() -> Self {
        Self {
            value: 0.75,
            color: [0.25, 0.85, 0.3],
            background: [0.12; 3],
            enabled: true,
        }
    }
}
/// Mirrors `epok::LayoutKind`. The numbering is pinned: saved Blueprint graphs
/// store it, so the type only ever grows at the end.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum LayoutKind {
    #[default]
    None = 0,
    Horizontal = 1,
    Vertical = 2,
    Grid = 3,
    Margin = 4,
    Center = 5,
}
impl From<u8> for LayoutKind {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Horizontal,
            2 => Self::Vertical,
            3 => Self::Grid,
            4 => Self::Margin,
            5 => Self::Center,
            _ => Self::None,
        }
    }
}
impl LayoutKind {
    pub const ALL: [Self; 6] = [
        Self::None,
        Self::Horizontal,
        Self::Vertical,
        Self::Grid,
        Self::Margin,
        Self::Center,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Horizontal => "Horizontal",
            Self::Vertical => "Vertical",
            Self::Grid => "Grid",
            Self::Margin => "Margin",
            Self::Center => "Center",
        }
    }
}
/// Per-child layout hints read only when the parent has a `LayoutContainer`.
/// `horizontal`/`vertical` are bitfields: 1 Fill, 2 Expand, 4 Shrink Center,
/// 8 Shrink End.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct LayoutElement {
    pub enabled: bool,
    pub horizontal: u8,
    pub vertical: u8,
    pub minimum: [f32; 2],
    pub stretch: f32,
}
impl Default for LayoutElement {
    fn default() -> Self {
        Self {
            enabled: true,
            horizontal: 1,
            vertical: 1,
            minimum: [0.; 2],
            stretch: 1.,
        }
    }
}
/// Automatic placement for the children of this rect. `padding` is left, top,
/// right, bottom, the order `Image::borders` uses.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct LayoutContainer {
    pub enabled: bool,
    pub kind: LayoutKind,
    pub spacing: [f32; 2],
    pub padding: [f32; 4],
    pub columns: u8,
}
impl Default for LayoutContainer {
    fn default() -> Self {
        Self {
            enabled: true,
            kind: LayoutKind::None,
            spacing: [0.; 2],
            padding: [0.; 4],
            columns: 1,
        }
    }
}
pub type Rect = [f32; 4]; // bottom-left x/y, width/height; positive Y points upward.
#[allow(dead_code)]
pub fn resolve(parent: Rect, r: &RectTransform) -> Rect {
    crate::hud_native::resolve(parent, r)
}
/// Every actor's resolved HUD rect, measured and arranged by the shared runtime
/// core in one call. Containers make a rect depend on its siblings, so the whole
/// scene is laid out at once and the viewport indexes the result.
pub fn layouts(scene: &Scene) -> Vec<Option<Rect>> {
    crate::hud_native::layouts(scene)
}
/// One actor's rect. Laying a scene out costs one pass whatever is asked of it,
/// so production code calls `layouts` once and indexes the result; this is the
/// readable form for the tests that want a single rectangle.
#[cfg(test)]
pub fn layout(scene: &Scene, index: usize) -> Option<Rect> {
    layouts(scene).get(index).copied().flatten()
}
pub fn order(scene: &Scene) -> Vec<usize> {
    fn visit(s: &Scene, i: usize, out: &mut Vec<usize>) {
        if !s.is_active(i) {
            return;
        }
        out.push(i);
        for (child, e) in s.actors.iter().enumerate() {
            if s.spatial_parent(child) == Some(i) && e.rect.is_some() {
                visit(s, child, out);
            }
        }
    }
    let mut out = Vec::new();
    for (i, e) in scene.actors.iter().enumerate() {
        if e.canvas.as_ref().is_some_and(|c| c.enabled)
            || (e.rect.is_some() && scene.spatial_parent(i).is_none())
        {
            visit(scene, i, &mut out);
        }
    }
    out
}
pub fn validate(scene: &Scene) -> Result<(), String> {
    scene.hud_budget.validate()?;
    let mut texts = 0;
    for e in &scene.actors {
        if let Some(image) = &e.image {
            let dimensions = image
                .texture
                .and_then(|id| scene.textures.get(&id))
                .map_or([256, 256], |t| [t.width, t.height]);
            let w = if image.region[2] > 0 {
                image.region[2]
            } else {
                dimensions[0].saturating_sub(image.region[0])
            };
            let h = if image.region[3] > 0 {
                image.region[3]
            } else {
                dimensions[1].saturating_sub(image.region[1])
            };
            if u32::from(image.borders[0]) + u32::from(image.borders[2]) > u32::from(w)
                || u32::from(image.borders[1]) + u32::from(image.borders[3]) > u32::from(h)
            {
                return Err("Image nine-slice borders exceed the source atlas region".into());
            }
        }
        if e.canvas.is_some() && e.kind != "Empty" {
            return Err("Canvas belongs to a UIActor without a 3D mesh".into());
        }
        if let Some(r) = &e.rect {
            if e.kind != "Empty" {
                return Err(
                    "RectTransform needs a Canvas or RectTransform parent and no 3D mesh".into(),
                );
            }
            if r.position
                .iter()
                .chain(&r.size)
                .any(|v| !v.is_finite() || v.abs() > 1024.)
                || (0..2).any(|i| {
                    !r.anchor_min[i].is_finite()
                        || !r.anchor_max[i].is_finite()
                        || !r.pivot[i].is_finite()
                        || r.anchor_min[i] < 0.
                        || r.anchor_max[i] > 1.
                        || r.anchor_min[i] > r.anchor_max[i]
                        || !(0. ..=1.).contains(&r.pivot[i])
                })
            {
                return Err(
                    "Invalid RectTransform: anchors/pivot 0..1, position/size within ±1024".into(),
                );
            }
        }
        if (e.image.is_some()
            || e.text.is_some()
            || e.progress.is_some()
            || e.layout_element.is_some()
            || e.layout_container.is_some())
            && e.rect.is_none()
        {
            return Err("HUD graphics require RectTransform".into());
        }
        if let Some(c) = &e.layout_container
            && (c.columns < 1
                || c.spacing
                    .iter()
                    .chain(&c.padding)
                    .any(|v| !v.is_finite() || v.abs() > 1024.))
        {
            return Err(
                "Invalid Layout Container: columns at least 1, spacing/padding within ±1024".into(),
            );
        }
        if let Some(c) = &e.layout_element
            && (!c.stretch.is_finite()
                || c.stretch < 0.
                || c.minimum
                    .iter()
                    .any(|v| !v.is_finite() || *v < 0. || *v > 1024.))
        {
            return Err(
                "Invalid Layout Element: minimum 0..1024 and stretch zero or greater".into(),
            );
        }
        let valid_color = |c: &[f32; 3]| c.iter().all(|v| v.is_finite() && (0. ..=1.).contains(v));
        if e.image.as_ref().is_some_and(|c| !valid_color(&c.color))
            || e.text.as_ref().is_some_and(|c| !valid_color(&c.color))
            || e.progress.as_ref().is_some_and(|c| {
                !valid_color(&c.color)
                    || !valid_color(&c.background)
                    || !c.value.is_finite()
                    || !(0. ..=1.).contains(&c.value)
            })
        {
            return Err("HUD colors and fill must be between 0 and 1".into());
        }
        if let Some(t) = &e.text {
            texts += 1;
            crate::bitmap_font::validate(&t.text)?;
        }
        if let Some(i) = &e.image {
            crate::texture::validate_region(i.texture, i.region, scene)?;
        }
    }
    let glyphs: usize = scene
        .actors
        .iter()
        .filter_map(|e| e.text.as_ref())
        .filter(|t| t.enabled)
        .map(|t| t.text.chars().filter(|c| *c != '\n').count())
        .sum();
    let rectangles: usize = scene
        .actors
        .iter()
        .map(|e| {
            e.image.as_ref().filter(|v| v.enabled).map_or(0, |v| {
                if v.borders.iter().any(|b| *b > 0) {
                    9
                } else {
                    1
                }
            }) + 2 * usize::from(e.progress.as_ref().is_some_and(|v| v.enabled))
        })
        .sum();
    if texts > scene.hud_budget.texts
        || scene.actors.iter().filter(|e| e.rect.is_some()).count() > scene.hud_budget.layouts
        || glyphs > scene.hud_budget.glyphs
        || rectangles > scene.hud_budget.rectangles
    {
        return Err("Scene exceeds its configured HUD layout/text/glyph/rectangle budget".into());
    }
    Ok(())
}
#[cfg(test)]
pub fn render(scene: &Scene) -> Vec<u8> {
    render_at(scene, 0.)
}
pub fn render_at(scene: &Scene, seconds: f32) -> Vec<u8> {
    let (commands, _) = crate::hud_native::compile(scene);
    crate::hud_native::render(scene, &commands, seconds, 0)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::Actor;
    #[test]
    fn nine_slice_preserves_borders_and_checks_budget() {
        let mut scene = fixture();
        scene.actors[1].image.as_mut().unwrap().borders = [4; 4];
        scene.hud_budget.rectangles = 8;
        assert!(scene.validate().unwrap_err().contains("budget"));
        scene.hud_budget.rectangles = 9;
        scene.validate().unwrap();
        scene.actors[1].image.as_mut().unwrap().borders = [200; 4];
        assert!(scene.validate().unwrap_err().contains("borders"));
    }
    #[test]
    fn native_nine_slice_keeps_corner_pixels_and_shrinks_borders() {
        let mut scene = fixture();
        let id = uuid::Uuid::new_v4();
        scene.textures.insert(
            id,
            std::sync::Arc::new(crate::texture::Data {
                width: 16,
                height: 16,
                words: vec![0; 128],
                palette: vec![0; 256],
                rgba: vec![255; 16 * 16 * 4],
            }),
        );
        let image = scene.actors[1].image.as_mut().unwrap();
        image.texture = Some(id);
        image.borders = [4; 4];
        let (commands, stats) = crate::hud_native::compile(&scene);
        assert_eq!(stats[3], 9);
        assert_eq!(&commands[0].0[3..11], &[12, 12, 16, 16, 0, 0, 3, 3]);
        scene.actors[1].rect.as_mut().unwrap().size = [4., 4.];
        let (commands, stats) = crate::hud_native::compile(&scene);
        assert_eq!(stats[3], 4);
        assert_eq!(&commands[0].0[3..11], &[12, 12, 14, 14, 0, 0, 3, 3]);
    }
    fn fixture() -> Scene {
        let mut c = Actor::cube("Canvas".into());
        c.kind = "Empty".into();
        c.canvas = Some(Default::default());
        let mut p = Actor::cube("Panel".into());
        p.kind = "Empty".into();
        p.parent = Some(0);
        p.rect = Some(RectTransform {
            anchor_min: [0., 1.],
            anchor_max: [0., 1.],
            pivot: [0., 1.],
            position: [12., -12.],
            size: [180., 64.],
        });
        p.image = Some(Image {
            color: [0., 0., 1.],
            enabled: true,
            ..Default::default()
        });
        let mut scene = Scene {
            version: crate::actor_document::SCENE_VERSION,
            name: "HUD".into(),
            actors: vec![c, p],
            ..Scene::default()
        };
        scene.sync_actor_components();
        scene
    }
    #[test]
    fn anchors_pivots_nested_layout_and_reparent_preserve_screen_rect() {
        let mut s = fixture();
        s.validate().unwrap();
        assert_eq!(layout(&s, 1), Some([12., 164., 180., 64.]));
        let mut child = s.actors[1].clone();
        let ids = crate::actor_document::fresh_identities(std::slice::from_ref(&child));
        crate::actor_document::remap_actor(&mut child, &ids);
        child.logical_parent = None;
        child.attach = None;
        child.parent = Some(1);
        child.rect = Some(RectTransform {
            anchor_min: [0.; 2],
            anchor_max: [1.; 2],
            pivot: [0.5; 2],
            position: [0.; 2],
            size: [-16., -16.],
        });
        s.actors.push(child);
        s.sync_actor_components();
        assert_eq!(layout(&s, 2), Some([20., 172., 164., 48.]));
        let before = layout(&s, 2);
        s.reparent(2, Some(0), true).unwrap();
        assert_eq!(before, layout(&s, 2));
        let decoded: Scene = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
        assert_eq!(s, decoded);
    }
    #[test]
    fn a_vertical_box_stacks_its_children_downward_from_the_top() {
        let mut s = fixture();
        let container = s.actors[1].rect.as_mut().unwrap();
        container.anchor_min = [0., 1.];
        container.anchor_max = [0., 1.];
        container.pivot = [0., 1.];
        container.position = [0., 0.];
        container.size = [200., 120.];
        s.actors[1].image = None;
        s.actors[1].layout_container = Some(LayoutContainer {
            kind: LayoutKind::Vertical,
            spacing: [0., 10.],
            ..Default::default()
        });
        for name in ["First", "Second", "Third"] {
            let mut label = Actor::cube(name.into());
            label.kind = "Empty".into();
            label.parent = Some(1);
            label.rect = Some(RectTransform {
                size: [50., 20.],
                ..Default::default()
            });
            label.text = Some(Text {
                text: name.into(),
                wrap: false,
                ..Default::default()
            });
            s.actors.push(label);
        }
        s.sync_actor_components();
        s.validate().unwrap();
        // The canvas is 320x240 and the box hangs from its top-left corner.
        let boxes = layouts(&s);
        assert_eq!(boxes[1], Some([0., 120., 200., 120.]));
        // Each label measures 50x20 from its rect, wider and taller than the 48x16
        // the unwrapped text needs; Fill is the default on both axes.
        assert_eq!(boxes[2], Some([0., 220., 200., 20.]));
        assert_eq!(boxes[3], Some([0., 190., 200., 20.]));
        assert_eq!(boxes[4], Some([0., 160., 200., 20.]));
        assert!(boxes[2].unwrap()[1] > boxes[3].unwrap()[1]);
        assert_eq!(layout(&s, 4), boxes[4]);
        // Hiding the middle label closes the list up instead of leaving its gap.
        s.actors[3].active = false;
        let boxes = layouts(&s);
        assert_eq!(boxes[2], Some([0., 220., 200., 20.]));
        assert_eq!(boxes[3], None);
        assert_eq!(boxes[4], Some([0., 190., 200., 20.]));
    }
    #[test]
    fn layout_kind_round_trips_through_serde_and_its_pinned_numbering() {
        for (kind, name, value) in [
            (LayoutKind::None, "none", 0u8),
            (LayoutKind::Horizontal, "horizontal", 1),
            (LayoutKind::Vertical, "vertical", 2),
            (LayoutKind::Grid, "grid", 3),
            (LayoutKind::Margin, "margin", 4),
            (LayoutKind::Center, "center", 5),
        ] {
            assert_eq!(kind as u8, value);
            assert_eq!(LayoutKind::from(value), kind);
            let text = serde_json::to_string(&kind).unwrap();
            assert_eq!(text, format!("\"{name}\""));
            assert_eq!(serde_json::from_str::<LayoutKind>(&text).unwrap(), kind);
        }
        assert_eq!(LayoutKind::from(9), LayoutKind::None);
        assert_eq!(LayoutKind::ALL.len(), 6);
    }
    #[test]
    fn layout_components_are_validated_against_their_budgets() {
        let mut s = fixture();
        s.actors[1].layout_container = Some(Default::default());
        s.actors[1].layout_element = Some(Default::default());
        s.validate().unwrap();
        s.actors[1].layout_container.as_mut().unwrap().columns = 0;
        assert!(s.validate().unwrap_err().contains("Layout Container"));
        s.actors[1].layout_container.as_mut().unwrap().columns = 1;
        s.actors[1].layout_container.as_mut().unwrap().padding[2] = 4096.;
        assert!(s.validate().unwrap_err().contains("Layout Container"));
        s.actors[1].layout_container.as_mut().unwrap().padding[2] = 0.;
        s.actors[1].layout_element.as_mut().unwrap().stretch = -1.;
        assert!(s.validate().unwrap_err().contains("Layout Element"));
        s.actors[1].layout_element.as_mut().unwrap().stretch = 1.;
        s.actors[1].rect = None;
        assert!(s.validate().unwrap_err().contains("RectTransform"));
    }
    #[test]
    fn invalid_hud_dependencies_text_and_cycles_are_rejected() {
        let mut s = fixture();
        s.reparent(1, None, true).unwrap();
        s.validate().unwrap();
        s.reparent(1, Some(0), true).unwrap();
        s.actors[1].text = Some(Text {
            text: "漢".into(),
            ..Default::default()
        });
        assert!(s.validate().is_err());
        s.actors[1].text.as_mut().unwrap().text = "HP 075".into();
        s.validate().unwrap();
        assert!(s.reparent(0, Some(1), false).is_err());
        s.actors[1].rect.as_mut().unwrap().anchor_min = [1., 1.];
        s.actors[1].rect.as_mut().unwrap().anchor_max = [0., 0.];
        assert!(s.validate().is_err());
    }
    #[test]
    fn animated_palette_matches_texture_indices_and_preserves_cutout() {
        let mut s = fixture();
        let id = uuid::Uuid::new_v4();
        s.actors[1].rect = Some(RectTransform {
            anchor_min: [0., 1.],
            anchor_max: [0., 1.],
            pivot: [0., 1.],
            position: [0.; 2],
            size: [3., 1.],
        });
        s.actors[1].image = Some(Image {
            texture: Some(id),
            color: [1.; 3],
            ..Default::default()
        });
        s.actors[1].palette_animator = Some(crate::palette::Animator {
            texture: Some(id),
            speed: 1.,
            ..Default::default()
        });
        let mut palette = vec![0; 256];
        palette[1] = 0x801f;
        palette[2] = 0x83e0;
        s.textures.insert(
            id,
            std::sync::Arc::new(crate::texture::Data {
                width: 3,
                height: 1,
                words: vec![0x0100, 0x0002],
                palette,
                rgba: vec![0, 0, 0, 0, 255, 0, 0, 255, 0, 255, 0, 255],
            }),
        );
        assert_eq!(
            &render_at(&s, 0.)[..12],
            &[33, 40, 52, 255, 255, 0, 0, 255, 0, 255, 0, 255]
        );
        assert_eq!(
            &render_at(&s, 1.)[..12],
            &[33, 40, 52, 255, 0, 255, 0, 255, 255, 0, 0, 255]
        );
        s.actors[1].palette_animator.as_mut().unwrap().enabled = false;
        assert_eq!(render_at(&s, 1.), render(&s));
    }
    #[test]
    fn disabled_canvas_hides_all_graphics_and_text_uses_psx_bitmap() {
        let mut s = fixture();
        s.actors[1].text = Some(Text {
            text: "HUD".into(),
            ..Default::default()
        });
        let rendered = render(&s);
        assert!(rendered.chunks_exact(4).any(|p| p == [0, 0, 255, 255]));
        assert!(rendered.chunks_exact(4).any(|p| p == [255; 4]));
        s.actors[0].canvas.as_mut().unwrap().enabled = false;
        assert!(render(&s).chunks_exact(4).all(|p| p == [33, 40, 52, 255]));
    }
}
