use crate::scene::Scene;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Budget {
    pub layouts: usize,
    pub rectangles: usize,
    pub texts: usize,
    pub glyphs: usize,
    /// Quads emitted by rotated elements. They sit in their own double-buffered
    /// polygon pools, so an unrotated HUD pays nothing for this.
    pub rotated: usize,
}
impl Default for Budget {
    fn default() -> Self {
        Self {
            layouts: 128,
            rectangles: 256,
            texts: 64,
            glyphs: 1024,
            rotated: 128,
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
            || self.rotated == 0
            || self.rotated > 512
        {
            return Err(
                "HUD budgets: layouts 1..128, rectangles 1..512, texts 1..64, glyphs 1..2048, rotated 1..512"
                    .into(),
            );
        }
        Ok(())
    }
    pub fn header(&self) -> Result<String, String> {
        self.validate()?;
        Ok(format!(
            "#pragma once\nnamespace epok {{\ninline constexpr unsigned hud_layout_budget={};\ninline constexpr unsigned hud_rectangle_budget={};\ninline constexpr unsigned hud_text_budget={};\ninline constexpr unsigned hud_glyph_budget={};\ninline constexpr unsigned hud_rotated_budget={};\n}}\n",
            self.layouts, self.rectangles, self.texts, self.glyphs, self.rotated
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
    /// The focused element, as an actor index, or -1. The authored initial focus
    /// and the runtime's current focus are this one field.
    pub focused: i32,
}
impl Default for Canvas {
    fn default() -> Self {
        Self {
            enabled: true,
            focused: -1,
        }
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
    /// Degrees about the pivot. Layout stays axis-aligned; only the emitted
    /// primitives turn, and zero emits exactly what an unrotated element emits.
    #[serde(default)]
    pub rotation: f32,
}
impl Default for RectTransform {
    fn default() -> Self {
        Self {
            anchor_min: [0.5; 2],
            anchor_max: [0.5; 2],
            pivot: [0.5; 2],
            position: [0.; 2],
            size: [100., 32.],
            rotation: 0.,
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
    /// How the source region fills the rect. With nine-slice borders only the
    /// centre piece tiles; the corners and edges keep their stretch.
    pub tiling: ImageTiling,
}
impl Default for Image {
    fn default() -> Self {
        Self {
            color: [0.2, 0.4, 0.65],
            enabled: true,
            texture: None,
            region: [0; 4],
            borders: [0; 4],
            tiling: ImageTiling::None,
        }
    }
}
/// Mirrors `epok::ImageTiling`. The numbering is pinned: saved Blueprint graphs
/// store it, so the type only ever grows at the end.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum ImageTiling {
    #[default]
    None = 0,
    Tile = 1,
    TileFit = 2,
}
impl From<u8> for ImageTiling {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Tile,
            2 => Self::TileFit,
            _ => Self::None,
        }
    }
}
impl ImageTiling {
    pub const ALL: [Self; 3] = [Self::None, Self::Tile, Self::TileFit];
    pub fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Tile => "Tile",
            Self::TileFit => "Tile Fit",
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
/// Per-child focus and D-pad navigation, read by the runtime's focus pass.
///
/// `neighbors` holds actor indices rather than actor UUIDs. No other
/// `BuiltinData` component references another actor — the UUID references in a
/// document are `logical_parent` and `attach`, both on the actor itself — so
/// there is no remapping precedent to follow here, and the runtime table is
/// indices either way. Reordering actors therefore rewrites these by hand.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Focusable {
    pub enabled: bool,
    /// Left, right, up, down; -1 for no neighbour that way.
    pub neighbors: [i32; 4],
    pub order: u8,
    /// Multiplied into the image and fill colours while this element is focused.
    pub highlight: [f32; 3],
}
impl Default for Focusable {
    fn default() -> Self {
        Self {
            enabled: true,
            neighbors: [-1; 4],
            order: 0,
            highlight: [1.; 3],
        }
    }
}
pub type Rect = [f32; 4]; // bottom-left x/y, width/height; positive Y points upward.
/// Where one element sits: the axis-aligned rect the layout pass decided, and
/// the four corners it really occupies once rotation is applied — top-left,
/// top-right, bottom-left, bottom-right, in HUD space with +Y up. Without
/// rotation the corners are simply the rect's own.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placement {
    pub rect: Rect,
    pub corners: [[f32; 2]; 4],
}
#[allow(dead_code)]
pub fn resolve(parent: Rect, r: &RectTransform) -> Rect {
    crate::hud_native::resolve(parent, r)
}
/// Every actor's resolved HUD rect, measured and arranged by the shared runtime
/// core in one call. Containers make a rect depend on its siblings, so the whole
/// scene is laid out at once and the viewport indexes the result.
pub fn layouts(scene: &Scene) -> Vec<Option<Placement>> {
    crate::hud_native::layouts(scene)
}
/// One actor's rect. Laying a scene out costs one pass whatever is asked of it,
/// so production code calls `layouts` once and indexes the result; this is the
/// readable form for the tests that want a single rectangle.
#[cfg(test)]
pub fn layout(scene: &Scene, index: usize) -> Option<Rect> {
    layouts(scene).get(index).copied().flatten().map(|p| p.rect)
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
            if !r.rotation.is_finite()
                || r.rotation.abs() > 3600.
                || r.position
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
                    "Invalid RectTransform: anchors/pivot 0..1, position/size within ±1024, rotation within ±3600 degrees"
                        .into(),
                );
            }
        }
        if (e.image.is_some()
            || e.text.is_some()
            || e.progress.is_some()
            || e.layout_element.is_some()
            || e.layout_container.is_some()
            || e.focusable.is_some())
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
        if let Some(c) = &e.focusable {
            let count = scene.actors.len() as i32;
            if c.neighbors.iter().any(|n| *n < -1 || *n >= count)
                || c.highlight
                    .iter()
                    .any(|v| !v.is_finite() || !(0. ..=1.).contains(v))
            {
                return Err(
                    "Invalid Focusable: neighbours are actor indices or -1, highlight 0..1".into(),
                );
            }
        }
        if let Some(c) = &e.canvas
            && (c.focused < -1 || c.focused >= scene.actors.len() as i32)
        {
            return Err("Canvas initial focus must be an actor index or -1".into());
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
    // Tiling multiplies one image into many, and a rotated element draws from
    // the rotated pool instead of the rectangle and glyph pools, so the three
    // counts are accumulated together over the laid-out rects the runtime sees.
    let places = layouts(scene);
    let (mut glyphs, mut rectangles, mut rotated) = (0usize, 0usize, 0usize);
    for (i, e) in scene.actors.iter().enumerate() {
        let size = places
            .get(i)
            .copied()
            .flatten()
            .map_or([0., 0.], |p| [p.rect[2], p.rect[3]]);
        let pictures = e.image.as_ref().filter(|v| v.enabled).map_or(0, |v| {
            let dimensions = v
                .texture
                .and_then(|id| scene.textures.get(&id))
                .map_or([256, 256], |t| [t.width, t.height]);
            image_primitives(v, dimensions, size)
        }) + 2 * usize::from(e.progress.as_ref().is_some_and(|v| v.enabled));
        let letters = e
            .text
            .as_ref()
            .filter(|t| t.enabled)
            .map_or(0, |t| t.text.chars().filter(|c| *c != '\n').count());
        if e.rect.as_ref().is_some_and(|r| r.rotation != 0.) {
            rotated += pictures + letters;
        } else {
            rectangles += pictures;
            glyphs += letters;
        }
    }
    if texts > scene.hud_budget.texts
        || scene.actors.iter().filter(|e| e.rect.is_some()).count() > scene.hud_budget.layouts
        || glyphs > scene.hud_budget.glyphs
        || rectangles > scene.hud_budget.rectangles
        || rotated > scene.hud_budget.rotated
    {
        return Err(
            "Scene exceeds its configured HUD layout/text/glyph/rectangle/rotated budget".into(),
        );
    }
    Ok(())
}
/// How many textured quads one Image draws, the way `hud_core::picture` counts
/// them: one per nine-slice piece, and one per tile of the piece that tiles.
/// `size` is the resolved rect in HUD pixels; it is only an upper bound for a
/// nine-sliced centre, which is smaller than the whole rect.
fn image_primitives(image: &Image, dimensions: [u16; 2], size: [f32; 2]) -> usize {
    let source = [0usize, 1].map(|i| {
        if image.region[i + 2] > 0 {
            u32::from(image.region[i + 2])
        } else {
            u32::from(dimensions[i].saturating_sub(image.region[i]))
        }
    });
    let tiles = if image.tiling == ImageTiling::None {
        1
    } else {
        [0usize, 1]
            .map(|i| {
                let extent = size[i].max(0.).round() as u32;
                if source[i] == 0 {
                    return 1;
                }
                match image.tiling {
                    ImageTiling::TileFit => (extent + source[i] / 2) / source[i],
                    _ => extent.div_ceil(source[i]),
                }
                .max(1) as usize
            })
            .iter()
            .product()
    };
    if image.borders.iter().any(|b| *b > 0) {
        8 + tiles
    } else {
        tiles
    }
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
            ..Default::default()
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
            ..Default::default()
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
        let boxes: Vec<Option<Rect>> = layouts(&s).iter().map(|p| p.map(|p| p.rect)).collect();
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
        let boxes: Vec<Option<Rect>> = layouts(&s).iter().map(|p| p.map(|p| p.rect)).collect();
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
    fn tiling_and_focus_round_trip_through_serde_and_their_pinned_numbering() {
        for (tiling, name, value) in [
            (ImageTiling::None, "none", 0u8),
            (ImageTiling::Tile, "tile", 1),
            (ImageTiling::TileFit, "tile_fit", 2),
        ] {
            assert_eq!(tiling as u8, value);
            assert_eq!(ImageTiling::from(value), tiling);
            let text = serde_json::to_string(&tiling).unwrap();
            assert_eq!(text, format!("\"{name}\""));
            assert_eq!(serde_json::from_str::<ImageTiling>(&text).unwrap(), tiling);
        }
        assert_eq!(ImageTiling::from(9), ImageTiling::None);
        assert_eq!(ImageTiling::ALL.len(), 3);
        let focus = Focusable {
            enabled: true,
            neighbors: [3, -1, 0, 7],
            order: 4,
            highlight: [0.5, 1., 0.25],
        };
        let text = serde_json::to_string(&focus).unwrap();
        assert_eq!(serde_json::from_str::<Focusable>(&text).unwrap(), focus);
        // Every field carries a default, so a document written before this
        // component existed still loads.
        assert_eq!(
            serde_json::from_str::<Focusable>("{}").unwrap(),
            Focusable::default()
        );
        assert_eq!(
            serde_json::from_str::<Canvas>("{\"enabled\":true}")
                .unwrap()
                .focused,
            -1
        );
        assert_eq!(
            serde_json::from_str::<RectTransform>("{}")
                .unwrap()
                .rotation,
            0.
        );
    }
    #[test]
    fn tiles_and_rotated_elements_are_counted_against_their_own_budgets() {
        let mut s = fixture();
        let id = uuid::Uuid::new_v4();
        s.textures.insert(
            id,
            std::sync::Arc::new(crate::texture::Data {
                width: 64,
                height: 64,
                words: vec![0; 2048],
                palette: vec![0; 256],
                rgba: vec![255; 64 * 64 * 4],
            }),
        );
        let panel = s.actors[1].rect.as_mut().unwrap();
        panel.size = [200., 100.];
        let image = s.actors[1].image.as_mut().unwrap();
        image.texture = Some(id);
        image.tiling = ImageTiling::Tile;
        s.hud_budget.rectangles = 7;
        assert!(s.validate().unwrap_err().contains("budget"));
        // Four columns by two rows over a 64x64 source, exactly as picture() tiles it.
        s.hud_budget.rectangles = 8;
        s.validate().unwrap();
        s.actors[1].image.as_mut().unwrap().tiling = ImageTiling::TileFit;
        s.hud_budget.rectangles = 6;
        s.validate().unwrap();
        // Rotating the element moves its cost out of the rectangle pool entirely.
        s.actors[1].rect.as_mut().unwrap().rotation = 30.;
        s.hud_budget.rectangles = 1;
        s.hud_budget.rotated = 5;
        assert!(s.validate().unwrap_err().contains("budget"));
        s.hud_budget.rotated = 6;
        s.validate().unwrap();
        s.actors[1].rect.as_mut().unwrap().rotation = f32::NAN;
        assert!(s.validate().unwrap_err().contains("rotation"));
    }
    #[test]
    fn focus_links_are_actor_indices_the_scene_still_has() {
        let mut s = fixture();
        s.actors[1].focusable = Some(Focusable {
            neighbors: [1, -1, -1, -1],
            ..Default::default()
        });
        s.sync_actor_components();
        s.validate().unwrap();
        s.actors[1].focusable.as_mut().unwrap().neighbors[0] = 9;
        assert!(s.validate().unwrap_err().contains("Focusable"));
        s.actors[1].focusable.as_mut().unwrap().neighbors[0] = -1;
        s.actors[0].canvas.as_mut().unwrap().focused = 1;
        s.validate().unwrap();
        s.actors[0].canvas.as_mut().unwrap().focused = 5;
        assert!(s.validate().unwrap_err().contains("focus"));
    }
    #[test]
    fn a_rotated_element_reports_turned_corners_and_an_unturned_rect() {
        let mut s = fixture();
        let before = layouts(&s)[1].unwrap();
        assert_eq!(
            before.corners,
            [
                [before.rect[0], before.rect[1] + before.rect[3]],
                [
                    before.rect[0] + before.rect[2],
                    before.rect[1] + before.rect[3]
                ],
                [before.rect[0], before.rect[1]],
                [before.rect[0] + before.rect[2], before.rect[1]],
            ]
        );
        s.actors[1].rect.as_mut().unwrap().rotation = 90.;
        let after = layouts(&s)[1].unwrap();
        assert_eq!(after.rect, before.rect);
        // The fixture panel pivots on its top-left corner, and that is what the
        // turn is about: the pivot is the one point rotation leaves alone.
        let p = s.actors[1].rect.as_ref().unwrap().pivot;
        let pivot = [
            before.rect[0] + before.rect[2] * p[0],
            before.rect[1] + before.rect[3] * p[1],
        ];
        for (plain, turned) in before.corners.iter().zip(&after.corners) {
            let expected = [
                pivot[0] - (plain[1] - pivot[1]),
                pivot[1] + (plain[0] - pivot[0]),
            ];
            assert!(
                (turned[0] - expected[0]).abs() < 0.05 && (turned[1] - expected[1]).abs() < 0.05,
                "{turned:?} vs {expected:?}"
            );
        }
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
            ..Default::default()
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
