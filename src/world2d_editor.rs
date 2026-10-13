//! The 2D world editor (World2D initiative): a 2D viewport with select,
//! move, rotate, scale and pan tools, grid snapping, rulers, real sprite and tile
//! previews, collider and camera gizmos, drag-and-drop sprite creation, plus the 2D
//! Inspector and its sprite region and sprite-sheet dialogs. Tile painting lives in
//! `tilemap_editor.rs` and the TileSet editor in `tileset_editor.rs`.
//!
//! Everything here edits the same Actor documents as the rest of the editor: the root
//! `SceneComponent2D` holds the transform and each engine 2D component one typed record
//! (see `world2d.rs`). Units are pixels with Y down, exactly what the PSX renders.
use crate::{
    editor::Editor,
    object_model as om,
    scene_view_mode::{SceneView2D, Transform2D},
    world2d::{
        self, Animation2D, Camera2D, CanvasLayer2D, CanvasModulate2D, CharacterBody2D, Collider2D,
        Frame2D, Parallax2D, Preset, Shape2D, Sprite2D,
    },
};
use imgui::{MouseButton, Ui};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

// ---- icons (FontAwesome solid; listed in platform::EDITOR_FA_GLYPHS) --------------------
pub const ICON_SELECT: &str = "\u{f245}";
pub const ICON_MOVE: &str = "\u{f0b2}";
pub const ICON_ROTATE: &str = "\u{f2f1}";
pub const ICON_SCALE: &str = "\u{f424}";
pub const ICON_PAN: &str = "\u{f256}";
pub const ICON_SNAP: &str = "\u{f076}";
pub const ICON_GRID: &str = "\u{f00a}";
pub const ICON_RULER: &str = "\u{f546}";
pub const ICON_SHAPES: &str = "\u{f5cb}";
pub const ICON_CAMERA: &str = "\u{f03d}";
pub const ICON_FRAME: &str = "\u{f065}";
pub const ICON_ZOOM_IN: &str = "\u{f00e}";
pub const ICON_ZOOM_OUT: &str = "\u{f010}";
pub const ICON_PLUS: &str = "\u{f067}";
pub const ICON_IMAGE: &str = "\u{f03e}";
pub const ICON_FILM: &str = "\u{f008}";
pub const ICON_RUN: &str = "\u{f70c}";
pub const ICON_BODY: &str = "\u{f45c}";
pub const ICON_AREA: &str = "\u{f140}";
pub const ICON_NODE: &str = "\u{f05b}";
pub const ICON_PLAY: &str = "\u{f04b}";
pub const ICON_TRASH: &str = "\u{f1f8}";
pub const ICON_CLONE: &str = "\u{f24d}";
pub const ICON_SLICE: &str = "\u{f0c4}";
pub const ICON_CROP: &str = "\u{f565}";
pub const ICON_FLIP_H: &str = "\u{f0ec}";
pub const ICON_FLIP_V: &str = "\u{f338}";
pub const ICON_INFO: &str = "\u{f05a}";
pub const ICON_WAND: &str = "\u{e2ca}";
pub const ICON_LEFT: &str = "\u{f048}";
pub const ICON_RIGHT: &str = "\u{f051}";
pub const ICON_PARALLAX: &str = "\u{f302}";
pub const ICON_CANVAS: &str = "\u{f5fd}";
pub const ICON_MODULATE: &str = "\u{f53f}";

pub fn preset_icon(preset: Preset) -> &'static str {
    match preset {
        Preset::Node2D => ICON_NODE,
        Preset::Sprite2D => ICON_IMAGE,
        Preset::AnimatedSprite2D => ICON_FILM,
        Preset::Camera2D => ICON_CAMERA,
        Preset::StaticBody2D => ICON_BODY,
        Preset::Area2D => ICON_AREA,
        Preset::CharacterBody2D => ICON_RUN,
        Preset::TileMapLayer => crate::tilemap_editor::ICON_TILEMAP,
        Preset::Parallax2D => ICON_PARALLAX,
        Preset::CanvasLayer2D => ICON_CANVAS,
        Preset::CanvasModulate2D => ICON_MODULATE,
        Preset::Path2D => crate::world2d_shapes_editor::ICON_PATH,
        Preset::PathFollow2D => crate::world2d_shapes_editor::ICON_FOLLOW,
        Preset::Line2D => crate::world2d_shapes_editor::ICON_LINE,
        Preset::Polygon2D => crate::world2d_shapes_editor::ICON_POLYGON,
        Preset::Particles2D => crate::world2d_shapes_editor::ICON_PARTICLES,
        Preset::PointLight2D => crate::world2d_shapes_editor::ICON_LIGHT,
        Preset::AudioPlayer2D => crate::world2d_shapes_editor::ICON_AUDIO,
        Preset::VisibleOnScreenNotifier2D => crate::world2d_shapes_editor::ICON_NOTIFIER,
        Preset::RemoteTransform2D => crate::world2d_shapes_editor::ICON_REMOTE,
    }
}

// ---- palette ---------------------------------------------------------------------------
const CANVAS: [f32; 4] = [0.118, 0.125, 0.145, 1.];
const GRID_MINOR: [f32; 4] = [1., 1., 1., 0.045];
const GRID_MAJOR: [f32; 4] = [1., 1., 1., 0.10];
const AXIS_X: [f32; 4] = [0.90, 0.36, 0.36, 0.85];
const AXIS_Y: [f32; 4] = [0.46, 0.82, 0.40, 0.85];
const SELECTION: [f32; 4] = [1.0, 0.62, 0.20, 1.];
const HOVER: [f32; 4] = [1., 1., 1., 0.55];
const SOLID: [f32; 4] = [0.36, 0.66, 1.0, 1.];
const TRIGGER: [f32; 4] = [0.42, 0.90, 0.52, 1.];
const ONE_WAY: [f32; 4] = [0.98, 0.84, 0.30, 1.];
const CAMERA_COLOUR: [f32; 4] = [0.78, 0.48, 1.0, 1.];
const RULER: [f32; 4] = [0.085, 0.09, 0.10, 0.96];

fn alpha(colour: [f32; 4], a: f32) -> [f32; 4] {
    [colour[0], colour[1], colour[2], colour[3] * a]
}

// ---- state -----------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tool {
    #[default]
    Select,
    Move,
    Rotate,
    Scale,
    Pan,
}
impl Tool {
    const ALL: [(Tool, &'static str, &'static str, &'static str); 5] = [
        (
            Tool::Select,
            ICON_SELECT,
            "Select (Q)",
            "Click to select, drag to move; handles scale and rotate",
        ),
        (
            Tool::Move,
            ICON_MOVE,
            "Move (W)",
            "Drag anywhere to move the selection",
        ),
        (
            Tool::Rotate,
            ICON_ROTATE,
            "Rotate (E)",
            "Drag around the pivot to rotate; Shift snaps",
        ),
        (
            Tool::Scale,
            ICON_SCALE,
            "Scale (S)",
            "Drag to scale from the pivot; Shift keeps the ratio",
        ),
        (Tool::Pan, ICON_PAN, "Pan (Space)", "Drag to pan the view"),
    ];
}

#[derive(Clone, Debug)]
enum DragKind {
    Move {
        grab: [f32; 2],
    },
    Rotate {
        start_angle: f32,
        start: f32,
    },
    Scale {
        axes: [bool; 2],
        start_local: [f32; 2],
        start: [f32; 2],
    },
    Collider {
        component: Uuid,
        side: usize,
        start: Collider2D,
    },
    Pan,
}
#[derive(Clone, Debug)]
struct Drag {
    actor: Option<Uuid>,
    kind: DragKind,
    moved: bool,
}

/// The region editor dialog for one Sprite2D.
#[derive(Clone, Debug)]
pub struct RegionEditor {
    actor: Uuid,
    component: Uuid,
    region: [u16; 4],
    zoom: f32,
    snap: u16,
    dragging: Option<[f32; 2]>,
}
/// The sprite-sheet slicer that adds frames to one animation.
#[derive(Clone, Debug)]
pub struct SheetSlicer {
    actor: Uuid,
    component: Uuid,
    animation: usize,
    columns: u16,
    rows: u16,
    offset: [u16; 2],
    separation: [u16; 2],
    selected: Vec<u16>,
    zoom: f32,
}

pub struct State {
    pub tool: Tool,
    pub snap: bool,
    pub grid_step: f32,
    pub rotation_step: f32,
    pub show_grid: bool,
    pub show_rulers: bool,
    pub show_colliders: bool,
    pub show_camera: bool,
    pub animate: bool,
    pub textures: TextureCache,
    drag: Option<Drag>,
    context: [f32; 2],
    hovered: Option<Uuid>,
    pub region_editor: Option<RegionEditor>,
    pub slicer: Option<SheetSlicer>,
    /// Selected animation in the Sprite2D inspector, per component.
    animation_selection: BTreeMap<Uuid, usize>,
    frame_selection: BTreeMap<Uuid, usize>,
    keep_ratio: bool,
    frame_request: bool,
    open_context: bool,
    right_dragged: bool,
    slicer_cell: [u16; 2],
    /// The map the view last framed automatically.
    framed: Option<String>,
    /// Tile painting and the palette under the view.
    pub tiles: crate::tilemap_editor::State,
    /// The TileSet editor window, when open.
    pub tileset_window: Option<crate::tileset_editor::Window>,
    /// Point editing of paths, lines and polygons.
    pub points: crate::world2d_shapes_editor::State,
}
impl Default for State {
    fn default() -> Self {
        Self {
            tool: Tool::Select,
            snap: true,
            grid_step: 8.,
            rotation_step: 15.,
            show_grid: true,
            show_rulers: true,
            show_colliders: true,
            show_camera: true,
            animate: true,
            textures: TextureCache::default(),
            drag: None,
            context: [0.; 2],
            hovered: None,
            region_editor: None,
            slicer: None,
            animation_selection: BTreeMap::new(),
            frame_selection: BTreeMap::new(),
            keep_ratio: false,
            frame_request: false,
            open_context: false,
            right_dragged: false,
            slicer_cell: [16, 16],
            framed: None,
            tiles: Default::default(),
            tileset_window: None,
            points: Default::default(),
        }
    }
}

// ---- texture previews ------------------------------------------------------------------
/// Uploads the PSX-quantized preview of each texture the 2D view or Inspector draws,
/// with nearest filtering so pixel art stays crisp at every zoom.
#[derive(Default)]
pub struct TextureCache {
    entries: BTreeMap<Uuid, (imgui::TextureId, [u16; 2], usize)>,
    wanted: BTreeSet<Uuid>,
}
impl TextureCache {
    /// The uploaded texture and its size, or None until the next upload pass.
    pub fn get(&mut self, id: Uuid) -> Option<(imgui::TextureId, [u16; 2])> {
        self.wanted.insert(id);
        self.entries
            .get(&id)
            .map(|(texture, size, _)| (*texture, *size))
    }
    pub fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        renderer: &mut imgui_wgpu::Renderer,
        textures: &BTreeMap<Uuid, std::sync::Arc<crate::texture::Data>>,
    ) {
        for id in std::mem::take(&mut self.wanted) {
            let Some(data) = textures.get(&id) else {
                continue;
            };
            let identity = std::sync::Arc::as_ptr(data) as usize;
            if self
                .entries
                .get(&id)
                .is_some_and(|(_, _, current)| *current == identity)
            {
                continue;
            }
            if data.rgba.len() < usize::from(data.width) * usize::from(data.height) * 4 {
                continue;
            }
            if let Some((old, _, _)) = self.entries.remove(&id) {
                renderer.textures.remove(old);
            }
            let image = imgui_wgpu::Texture::new(
                device,
                renderer,
                imgui_wgpu::TextureConfig {
                    size: wgpu::Extent3d {
                        width: u32::from(data.width),
                        height: u32::from(data.height),
                        depth_or_array_layers: 1,
                    },
                    format: Some(wgpu::TextureFormat::Rgba8UnormSrgb),
                    sampler_desc: wgpu::SamplerDescriptor {
                        mag_filter: wgpu::FilterMode::Nearest,
                        min_filter: wgpu::FilterMode::Nearest,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            );
            image.write(
                queue,
                &data.rgba,
                u32::from(data.width),
                u32::from(data.height),
            );
            let texture = renderer.textures.insert(image);
            self.entries
                .insert(id, (texture, [data.width, data.height], identity));
        }
    }
}

// ---- geometry --------------------------------------------------------------------------
/// A 2x3 affine transform in pixels, mirroring `epok::Affine2D`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Affine {
    pub m: [[f32; 2]; 2],
    pub t: [f32; 2],
}
impl Affine {
    pub const IDENTITY: Affine = Affine {
        m: [[1., 0.], [0., 1.]],
        t: [0., 0.],
    };
    pub fn local(transform: &Transform2D) -> Self {
        let (sin, cos) = transform.rotation.to_radians().sin_cos();
        Self {
            m: [
                [cos * transform.scale[0], -sin * transform.scale[1]],
                [sin * transform.scale[0], cos * transform.scale[1]],
            ],
            t: transform.position,
        }
    }
    pub fn compose(&self, child: &Affine) -> Affine {
        let m = &self.m;
        let c = &child.m;
        Affine {
            m: [
                [
                    m[0][0] * c[0][0] + m[0][1] * c[1][0],
                    m[0][0] * c[0][1] + m[0][1] * c[1][1],
                ],
                [
                    m[1][0] * c[0][0] + m[1][1] * c[1][0],
                    m[1][0] * c[0][1] + m[1][1] * c[1][1],
                ],
            ],
            t: self.point(child.t),
        }
    }
    pub fn point(&self, p: [f32; 2]) -> [f32; 2] {
        [
            self.m[0][0] * p[0] + self.m[0][1] * p[1] + self.t[0],
            self.m[1][0] * p[0] + self.m[1][1] * p[1] + self.t[1],
        ]
    }
    pub fn inverse(&self) -> Option<Affine> {
        let [[a, b], [c, d]] = self.m;
        let det = a * d - b * c;
        if det.abs() < 1e-9 {
            return None;
        }
        let m = [[d / det, -b / det], [-c / det, a / det]];
        let t = [
            -(m[0][0] * self.t[0] + m[0][1] * self.t[1]),
            -(m[1][0] * self.t[0] + m[1][1] * self.t[1]),
        ];
        Some(Affine { m, t })
    }
}

/// One World2D actor as the 2D view and its gizmos see it.
#[derive(Clone, Debug)]
pub struct View {
    pub id: Uuid,
    pub name: String,
    pub active: bool,
    pub local: Transform2D,
    /// Parent (attachment) world transform; identity at the map root.
    pub parent: Affine,
    pub world: Affine,
    pub sprite: Option<(Uuid, Sprite2D)>,
    pub colliders: Vec<(Uuid, Collider2D)>,
    pub camera: Option<Camera2D>,
    /// The first TileMapLayer of the actor.
    pub tiles: Option<TileLayerView>,
    pub parallax: Option<Parallax2D>,
    pub canvas_layer: Option<CanvasLayer2D>,
    pub modulate: Option<CanvasModulate2D>,
    /// The canvas the actor draws on and the repeat of its nearest Parallax2D.
    pub canvas: i8,
    pub repeat: [f32; 2],
    /// A grouping node: other actors name it as their logical parent.
    pub group: bool,
    /// Local bounds (min, max) used for picking and the selection box.
    pub bounds: ([f32; 2], [f32; 2]),
    pub order: (i32, f32, usize),
}

/// A tile layer as the view draws it: cells at the owner's world position.
#[derive(Clone)]
pub struct TileLayerView {
    pub component: Uuid,
    pub layer: world2d::TileMapLayer,
    pub cells: std::sync::Arc<crate::tilemap::Cells>,
    pub tileset: Option<std::sync::Arc<crate::tileset::TileSet>>,
    pub origin: [f32; 2],
}
impl std::fmt::Debug for TileLayerView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TileLayerView")
            .field("component", &self.component)
            .field("cells", &self.cells.len())
            .finish()
    }
}
impl TileLayerView {
    fn tile_size(&self) -> [f32; 2] {
        self.tileset
            .as_ref()
            .map_or([16., 16.], |t| t.tile_size.map(|v| f32::from(v.max(1))))
    }
    /// Whether a painted cell covers a world point.
    fn covers(&self, world: [f32; 2]) -> bool {
        let size = self.tile_size();
        let cell = [0, 1].map(|k| ((world[k] - self.origin[k]) / size[k]).floor() as i32);
        self.cells.contains_key(&cell)
    }
}
/// The canvas layer and parallax repeat an actor inherits through its attachments.
fn canvas_of(scene: &crate::scene::Scene, index: usize) -> (i8, [f32; 2]) {
    let mut current = index;
    let mut repeat = None;
    for _ in 0..32 {
        let actor = &scene.actors[current];
        if repeat.is_none()
            && let Some(p) = world2d::component(actor, world2d::PARALLAX2D)
        {
            repeat = Some(world2d::read::<Parallax2D>(p).repeat_size);
        }
        if let Some(c) = world2d::component(actor, world2d::CANVAS_LAYER2D) {
            return (
                world2d::read::<CanvasLayer2D>(c).layer,
                repeat.unwrap_or_default(),
            );
        }
        let Some(parent) = actor
            .attach
            .as_ref()
            .and_then(|a| scene.actor_index(a.actor))
        else {
            break;
        };
        current = parent;
    }
    (0, repeat.unwrap_or_default())
}

fn is_2d(actor: &crate::scene::Actor) -> bool {
    actor.class.class_id.as_deref() == Some(om::ACTOR2D_ID)
        || actor
            .root()
            .is_some_and(|r| r.class.class_id.as_deref() == Some(om::SCENE_COMPONENT2D_ID))
}
/// The Hierarchy icon of a 2D actor, or None for other domains.
pub fn hierarchy_icon(actor: &crate::scene::Actor) -> Option<&'static str> {
    is_2d(actor).then(|| node_icon(actor))
}
/// The command string a Hierarchy creation menu issues for a 2D node preset.
pub fn create_command(preset: Preset) -> &'static str {
    match preset {
        Preset::Node2D => "create-node2d-Node2D",
        Preset::Sprite2D => "create-node2d-Sprite2D",
        Preset::AnimatedSprite2D => "create-node2d-AnimatedSprite2D",
        Preset::Camera2D => "create-node2d-Camera2D",
        Preset::StaticBody2D => "create-node2d-StaticBody2D",
        Preset::Area2D => "create-node2d-Area2D",
        Preset::CharacterBody2D => "create-node2d-CharacterBody2D",
        Preset::TileMapLayer => "create-node2d-TileMapLayer",
        Preset::Parallax2D => "create-node2d-Parallax2D",
        Preset::CanvasLayer2D => "create-node2d-CanvasLayer2D",
        Preset::CanvasModulate2D => "create-node2d-CanvasModulate2D",
        Preset::Path2D => "create-node2d-Path2D",
        Preset::PathFollow2D => "create-node2d-PathFollow2D",
        Preset::Line2D => "create-node2d-Line2D",
        Preset::Polygon2D => "create-node2d-Polygon2D",
        Preset::Particles2D => "create-node2d-Particles2D",
        Preset::PointLight2D => "create-node2d-PointLight2D",
        Preset::AudioPlayer2D => "create-node2d-AudioPlayer2D",
        Preset::VisibleOnScreenNotifier2D => "create-node2d-VisibleOnScreenNotifier2D",
        Preset::RemoteTransform2D => "create-node2d-RemoteTransform2D",
    }
}
/// Whether the Inspector should show the 2D layout for this actor.
pub fn is_world2d_actor(e: &Editor, id: Uuid) -> bool {
    e.scene
        .actors
        .iter()
        .find(|a| a.id == id)
        .is_some_and(|actor| {
            is_2d(actor)
                || e.object_model().is_some_and(|model| {
                    actor
                        .class
                        .resolve(&model)
                        .is_some_and(|c| c.domain == crate::reflection_schema::Domain::World2D)
                })
        })
}
fn root_transform(actor: &crate::scene::Actor) -> Transform2D {
    actor
        .root()
        .map(|c| crate::scene_view_mode::read_transform_2d(&c.properties))
        .unwrap_or_default()
}
/// World transform of an actor through its spatial attachments (depth-bounded).
/// (parent, world) transforms of a scene actor, for other 2D editors.
pub(crate) fn world_of_index(scene: &crate::scene::Scene, index: usize) -> (Affine, Affine) {
    world_of(scene, index)
}
/// The local rectangle (min, max) the actor's sprite draws, when its texture is loaded.
pub(crate) fn sprite_bounds(e: &mut Editor, actor: Uuid) -> Option<([f32; 2], [f32; 2])> {
    let index = e.scene.actor_index(actor)?;
    let sprite: Sprite2D = world2d::read(world2d::component(
        &e.scene.actors[index],
        world2d::SPRITE2D,
    )?);
    let size = e.world2d.textures.get(sprite.texture?)?.1;
    let (size, _) = sprite_size(&sprite, Some(size))?;
    Some(sprite_local_rect(&sprite, size))
}
fn world_of(scene: &crate::scene::Scene, index: usize) -> (Affine, Affine) {
    let mut chain = vec![index];
    let mut current = index;
    for _ in 0..32 {
        let Some(parent) = scene.actors[current]
            .attach
            .as_ref()
            .and_then(|a| scene.actor_index(a.actor))
        else {
            break;
        };
        if chain.contains(&parent) {
            break;
        }
        chain.push(parent);
        current = parent;
    }
    let mut parent = Affine::IDENTITY;
    for &i in chain.iter().skip(1).rev() {
        parent = parent.compose(&Affine::local(&root_transform(&scene.actors[i])));
    }
    let world = parent.compose(&Affine::local(&root_transform(&scene.actors[index])));
    (parent, world)
}
fn sprite_size(sprite: &Sprite2D, cache_size: Option<[u16; 2]>) -> Option<([f32; 2], [u16; 4])> {
    let size = cache_size?;
    let rect = sprite.source_rect(size)?;
    Some(([f32::from(rect[2]), f32::from(rect[3])], rect))
}
fn sprite_local_rect(sprite: &Sprite2D, size: [f32; 2]) -> ([f32; 2], [f32; 2]) {
    let left = if sprite.centered { -size[0] * 0.5 } else { 0. } + sprite.offset[0];
    let top = if sprite.centered { -size[1] * 0.5 } else { 0. } + sprite.offset[1];
    ([left, top], [left + size[0], top + size[1]])
}
pub fn views(e: &mut Editor) -> Vec<View> {
    let mut out = Vec::new();
    for index in 0..e.scene.actors.len() {
        let actor = &e.scene.actors[index];
        if !is_2d(actor) {
            continue;
        }
        let (parent, world) = world_of(&e.scene, index);
        let mut sprite = None;
        let mut colliders = Vec::new();
        let mut camera = None;
        let mut tile_record = None;
        let (mut parallax, mut canvas_layer, mut modulate) = (None, None, None);
        for component in &actor.components {
            match component.class.class_id.as_deref() {
                Some(world2d::SPRITE2D) => {
                    sprite = Some((component.id, world2d::read::<Sprite2D>(component)))
                }
                Some(world2d::COLLIDER2D) => {
                    colliders.push((component.id, world2d::read::<Collider2D>(component)))
                }
                Some(world2d::CAMERA2D) => camera = Some(world2d::read::<Camera2D>(component)),
                Some(world2d::TILEMAP_LAYER) if tile_record.is_none() => {
                    tile_record = Some((
                        component.id,
                        world2d::read::<world2d::TileMapLayer>(component),
                    ))
                }
                Some(world2d::PARALLAX2D) => {
                    parallax = Some(world2d::read::<Parallax2D>(component))
                }
                Some(world2d::CANVAS_LAYER2D) => {
                    canvas_layer = Some(world2d::read::<CanvasLayer2D>(component))
                }
                Some(world2d::CANVAS_MODULATE2D) => {
                    modulate = Some(world2d::read::<CanvasModulate2D>(component))
                }
                _ => {}
            }
        }
        let (canvas, repeat) = canvas_of(&e.scene, index);
        let local = root_transform(actor);
        let (id, name, active) = (actor.id, actor.name.clone(), actor.active);
        let tiles = tile_record.map(|(component, layer)| {
            let cells =
                crate::tilemap_editor::cells_of(&mut e.world2d.tiles, component, &layer.cells);
            let tileset = layer
                .tile_set
                .and_then(|id| crate::tilemap_editor::tileset(e, id));
            TileLayerView {
                component,
                layer,
                cells,
                tileset,
                origin: world.t,
            }
        });
        let group = e.scene.actors.iter().any(|a| a.logical_parent == Some(id));
        let mut min = [f32::MAX; 2];
        let mut max = [f32::MIN; 2];
        let mut grow = |lo: [f32; 2], hi: [f32; 2]| {
            for k in 0..2 {
                min[k] = min[k].min(lo[k]);
                max[k] = max[k].max(hi[k]);
            }
        };
        let mut z = local.draw_order;
        if let Some(t) = &tiles {
            z += i32::from(t.layer.z_index);
        }
        if let Some((_, s)) = &sprite {
            z += i32::from(s.z_index);
            if let Some(texture) = s.texture
                && let Some((size, _)) =
                    sprite_size(s, e.world2d.textures.get(texture).map(|t| t.1))
            {
                let (lo, hi) = sprite_local_rect(s, size);
                grow(lo, hi);
            }
        }
        for (_, c) in &colliders {
            let (lo, hi) = c.bounds();
            grow(lo, hi);
        }
        if let Some((lo, hi)) = crate::world2d_shapes_editor::local_bounds(e, id) {
            grow(lo, hi);
        }
        // A tile layer's bounds are its painted cells, in the node's unrotated frame.
        if let Some(t) = &tiles
            && let Some((lo, hi)) = crate::tilemap::bounds(&t.cells)
            && let Some(inverse) = world.inverse()
        {
            let size = t.tile_size();
            let a = inverse.point([
                t.origin[0] + lo[0] as f32 * size[0],
                t.origin[1] + lo[1] as f32 * size[1],
            ]);
            let b = inverse.point([
                t.origin[0] + (hi[0] + 1) as f32 * size[0],
                t.origin[1] + (hi[1] + 1) as f32 * size[1],
            ]);
            grow(
                [a[0].min(b[0]), a[1].min(b[1])],
                [a[0].max(b[0]), a[1].max(b[1])],
            );
        }
        if min[0] > max[0] {
            min = [-8., -8.];
            max = [8., 8.];
        }
        let y_sort = sprite.as_ref().is_some_and(|(_, s)| s.y_sort);
        out.push(View {
            id,
            name,
            active,
            local,
            parent,
            world,
            sprite,
            colliders,
            camera,
            tiles,
            parallax,
            canvas_layer,
            modulate,
            canvas,
            repeat,
            group,
            bounds: (min, max),
            order: (
                z + i32::from(canvas) * 65536,
                if y_sort { world.t[1] } else { f32::MIN },
                index,
            ),
        });
    }
    out.sort_by(|a, b| {
        (a.order.0, a.order.1, a.order.2)
            .partial_cmp(&(b.order.0, b.order.1, b.order.2))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    out
}
fn corners(view: &View) -> [[f32; 2]; 4] {
    let (lo, hi) = view.bounds;
    [
        view.world.point(lo),
        view.world.point([hi[0], lo[1]]),
        view.world.point(hi),
        view.world.point([lo[0], hi[1]]),
    ]
}
fn contains(view: &View, world: [f32; 2]) -> bool {
    // A tile layer is picked by its painted cells, not by the box around them, so the
    // nodes standing on a large map stay clickable.
    if let Some(t) = &view.tiles
        && view.sprite.is_none()
        && view.colliders.is_empty()
        && !t.cells.is_empty()
    {
        return t.covers(world);
    }
    let Some(inverse) = view.world.inverse() else {
        return false;
    };
    let p = inverse.point(world);
    let (lo, hi) = view.bounds;
    p[0] >= lo[0] && p[0] <= hi[0] && p[1] >= lo[1] && p[1] <= hi[1]
}
fn snap_value(value: f32, step: f32) -> f32 {
    if step <= 0. {
        value
    } else {
        (value / step).round() * step
    }
}

// ---- document edits --------------------------------------------------------------------
/// Edits the root transform of one actor. Returns whether the actor exists.
fn edit_transform(e: &mut Editor, id: Uuid, edit: impl FnOnce(&mut Transform2D)) -> bool {
    let Some(index) = e.scene.actor_index(id) else {
        return false;
    };
    let actor = &mut e.scene.actors[index];
    let Some(root) = actor.components.iter_mut().find(|c| c.root) else {
        return false;
    };
    let mut transform = crate::scene_view_mode::read_transform_2d(&root.properties);
    edit(&mut transform);
    transform.position = transform.position.map(|v| v.clamp(-65536., 65536.));
    transform.scale = transform.scale.map(|v| {
        if v.abs() < 0.001 {
            0.001_f32.copysign(v)
        } else {
            v.clamp(-64., 64.)
        }
    });
    for (key, value) in [
        (
            "position",
            serde_json::json!([transform.position[0], transform.position[1]]),
        ),
        ("rotation", serde_json::json!(transform.rotation)),
        (
            "scale",
            serde_json::json!([transform.scale[0], transform.scale[1]]),
        ),
        (
            "draw_order",
            serde_json::json!(transform.draw_order.clamp(-4096, 4096)),
        ),
    ] {
        root.properties.remove("transform");
        root.properties.insert(key.into(), value);
        root.overrides.insert(key.into());
    }
    actor.refresh_components();
    true
}
/// Edits one 2D component record of an actor.
fn edit_component<T: serde::de::DeserializeOwned + serde::Serialize + Default>(
    e: &mut Editor,
    actor: Uuid,
    component: Uuid,
    edit: impl FnOnce(&mut T),
) {
    let Some(index) = e.scene.actor_index(actor) else {
        return;
    };
    let Some(c) = e.scene.actors[index]
        .components
        .iter_mut()
        .find(|c| c.id == component)
    else {
        return;
    };
    let mut value: T = world2d::read(c);
    edit(&mut value);
    world2d::write(c, &value);
    e.scene.actors[index].refresh_components();
}
fn unique_name(e: &Editor, base: &str) -> String {
    let mut name = base.to_owned();
    let mut suffix = 2;
    while e.scene.actors.iter().any(|a| a.name == name) {
        name = format!("{base}{suffix}");
        suffix += 1;
    }
    name
}
/// Creates a 2D node at a world position, selects it and records one undo step.
pub fn create_node(e: &mut Editor, preset: Preset, position: [f32; 2]) -> Option<Uuid> {
    if e.playing {
        e.log("Stop the emulator before editing.");
        return None;
    }
    let name = unique_name(e, preset.label());
    let actor = world2d::create(preset, &name, position.map(|v| v.round()));
    let id = actor.id;
    e.scene.actors.push(actor);
    let model = e.object_model();
    if let Err(error) = e.scene.validate_with_model(model.as_deref()) {
        e.scene.actors.retain(|a| a.id != id);
        e.last_error = Some(error.clone());
        e.log(error);
        return None;
    }
    if preset == Preset::PointLight2D {
        match crate::world2d_shapes_editor::light_texture(e) {
            Ok(texture) => {
                if let Some(index) = e.scene.actor_index(id)
                    && let Some(component) =
                        world2d::component(&e.scene.actors[index], world2d::SPRITE2D).map(|c| c.id)
                {
                    edit_component::<Sprite2D>(e, id, component, |s| s.texture = Some(texture));
                    let _ = crate::texture::resolve(&mut e.scene, &e.assets.index);
                }
            }
            Err(error) => e.log(format!("The light texture could not be created: {error}")),
        }
    }
    e.select_actor(Some(id));
    e.changed();
    e.log(format!("Created {name} ({}).", preset.label()));
    Some(id)
}
/// Creates a Sprite2D for a texture asset dropped on the view.
fn create_sprite_from_texture(e: &mut Editor, texture: Uuid, name: &str, position: [f32; 2]) {
    let base = name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect::<String>();
    let Some(id) = create_node(e, Preset::Sprite2D, position) else {
        return;
    };
    if let Some(index) = e.scene.actor_index(id) {
        let unique = unique_name(e, if base.is_empty() { "Sprite2D" } else { &base });
        e.scene.actors[index].name = unique;
        if let Some(component) =
            world2d::component(&e.scene.actors[index], world2d::SPRITE2D).map(|c| c.id)
        {
            edit_component::<Sprite2D>(e, id, component, |s| s.texture = Some(texture));
        }
        e.changed();
    }
}
/// The texture asset behind a Content Browser path: the `.epokasset` itself or the
/// imported source it came from.
fn texture_for_path(e: &Editor, path: &str) -> Option<Uuid> {
    let absolute = crate::assets::inside(&e.root, path).ok()?;
    e.assets
        .index
        .usable()
        .filter(|r| r.meta.kind == crate::assets::Kind::Texture)
        .find(|r| r.path == absolute || e.root.join(&r.meta.source) == absolute)
        .map(|r| r.meta.id)
}

// ---- viewport --------------------------------------------------------------------------
const HANDLE: f32 = 7.;
const RULER_SIZE: f32 = 18.;

fn tool_button(ui: &Ui, glyph: &str, id: &str, tip: &str, detail: &str, active: bool) -> bool {
    let pressed = crate::gui::icon(ui, glyph, id, "", active);
    if ui.is_item_hovered() {
        ui.tooltip(|| {
            ui.text(tip);
            if !detail.is_empty() {
                ui.text_disabled(detail);
            }
        });
    }
    pressed
}
fn toolbar(ui: &Ui, e: &mut Editor) {
    let _spacing = ui.push_style_var(imgui::StyleVar::ItemSpacing([2., 4.]));
    for (tool, glyph, tip, detail) in Tool::ALL {
        if tool_button(
            ui,
            glyph,
            &format!("tool-{tip}"),
            tip,
            detail,
            e.world2d.tool == tool,
        ) {
            e.world2d.tool = tool;
        }
        ui.same_line();
    }
    ui.same_line_with_spacing(0., 10.);
    if tool_button(
        ui,
        ICON_SNAP,
        "snap",
        "Snap (G)",
        "Snap moves to the grid and rotations to steps",
        e.world2d.snap,
    ) {
        e.world2d.snap = !e.world2d.snap;
    }
    ui.same_line();
    if ui.arrow_button("##snap-options", imgui::Direction::Down) {
        ui.open_popup("snap-options");
    }
    ui.popup("snap-options", || {
        ui.text("Snapping");
        ui.separator();
        ui.set_next_item_width(160.);
        crate::gui::Drag::new("Grid step (px)")
            .range(1., 256.)
            .speed(0.25)
            .display_format("%.0f")
            .build(ui, &mut e.world2d.grid_step);
        for step in [1., 2., 4., 8., 16., 32.] {
            if ui.small_button(format!("{step:.0}")) {
                e.world2d.grid_step = step;
            }
            ui.same_line();
        }
        ui.new_line();
        ui.set_next_item_width(160.);
        crate::gui::Drag::new("Rotation step (deg)")
            .range(1., 90.)
            .speed(0.25)
            .display_format("%.0f")
            .build(ui, &mut e.world2d.rotation_step);
        ui.text_disabled("Hold Alt while dragging to ignore snapping.");
    });
    ui.same_line_with_spacing(0., 10.);
    for (glyph, id, tip, value) in [
        (ICON_GRID, "grid", "Show grid", &mut e.world2d.show_grid),
        (
            ICON_RULER,
            "rulers",
            "Show rulers",
            &mut e.world2d.show_rulers,
        ),
        (
            ICON_SHAPES,
            "colliders",
            "Show collision shapes",
            &mut e.world2d.show_colliders,
        ),
        (
            ICON_CAMERA,
            "camera",
            "Show camera bounds",
            &mut e.world2d.show_camera,
        ),
        (
            ICON_PLAY,
            "animate",
            "Preview animations",
            &mut e.world2d.animate,
        ),
    ] {
        if tool_button(ui, glyph, id, tip, "", *value) {
            *value = !*value;
        }
        ui.same_line();
    }
    ui.same_line_with_spacing(0., 10.);
    let available = ui.content_region_avail();
    if available[0] > 230. {
        if tool_button(ui, ICON_ZOOM_OUT, "zoom-out", "Zoom out (-)", "", false) {
            let z = e.view_2d.zoom / 2.;
            e.view_2d.zoom = z.clamp(SceneView2D::MIN_ZOOM, SceneView2D::MAX_ZOOM);
        }
        ui.same_line();
        if ui.button_with_size(format!("{:.0}%##zoom", e.view_2d.zoom * 100.), [58., 0.]) {
            ui.open_popup("zoom-presets");
        }
        ui.popup("zoom-presets", || {
            for zoom in [0.125, 0.25, 0.5, 1., 2., 3., 4., 6., 8., 16.] {
                if ui.selectable(format!("{:.0}%", zoom * 100.)) {
                    e.view_2d.zoom = zoom;
                }
            }
        });
        ui.same_line();
        if tool_button(ui, ICON_ZOOM_IN, "zoom-in", "Zoom in (+)", "", false) {
            let z = e.view_2d.zoom * 2.;
            e.view_2d.zoom = z.clamp(SceneView2D::MIN_ZOOM, SceneView2D::MAX_ZOOM);
        }
        ui.same_line();
        if tool_button(
            ui,
            ICON_FRAME,
            "frame",
            "Frame selection (F)",
            "Frames the whole scene when nothing is selected",
            false,
        ) {
            e.world2d.frame_request = true;
        }
        ui.same_line_with_spacing(0., 10.);
    }
    if ui.button(format!("{ICON_PLUS} Add 2D Node")) {
        ui.open_popup("add-2d-node");
    }
    let centre = e.view_2d.centre;
    add_node_popup(ui, e, "add-2d-node", centre);
}
/// The 2D node palette: presets with a one-line description of each.
pub fn add_node_popup(ui: &Ui, e: &mut Editor, id: &str, position: [f32; 2]) {
    ui.popup(id, || {
        ui.text_disabled("Create 2D node");
        let mut category = "";
        for preset in Preset::ALL {
            if preset.category() != category {
                category = preset.category();
                ui.separator();
                ui.text_disabled(category);
            }
            let clicked = ui
                .selectable_config(format!(
                    "{}  {}##{}",
                    preset_icon(preset),
                    preset.label(),
                    preset.label()
                ))
                .size([250., 0.])
                .build();
            if ui.is_item_hovered() {
                ui.tooltip_text(preset.description());
            }
            if clicked {
                create_node(e, preset, snapped(e, position));
            }
        }
    });
}
fn snapped(e: &Editor, position: [f32; 2]) -> [f32; 2] {
    if e.world2d.snap {
        position.map(|v| snap_value(v, e.world2d.grid_step))
    } else {
        position.map(|v| v.round())
    }
}

/// The Scene window in 2D mode.
pub fn viewport(ui: &Ui, e: &mut Editor) {
    toolbar(ui, e);
    // A selected TileMapLayer gets the tile tools and, while painting, the palette.
    let painting = crate::tilemap_editor::painting(e);
    let layer_selected = e.selected_actor.is_some_and(|id| {
        e.scene.actor_index(id).is_some_and(|i| {
            world2d::component(&e.scene.actors[i], world2d::TILEMAP_LAYER).is_some()
        })
    });
    if layer_selected && !e.playing {
        crate::tilemap_editor::toolbar(ui, e);
    }
    if !e.playing
        && e.selected_actor
            .is_some_and(|id| crate::world2d_shapes_editor::shape_of(e, id).is_some())
    {
        crate::world2d_shapes_editor::toolbar(ui, e);
    }
    let origin = ui.cursor_screen_pos();
    let available = ui.content_region_avail();
    let status = ui.text_line_height_with_spacing() + 2.;
    let panel = if painting {
        e.world2d.tiles.panel_height = e
            .world2d
            .tiles
            .panel_height
            .clamp(120., (available[1] * 0.65).max(120.));
        e.world2d.tiles.panel_height + 8.
    } else {
        0.
    };
    let size = [
        available[0].max(1.),
        (available[1] - status - panel).max(1.),
    ];
    ui.invisible_button_flags(
        "##world2d-canvas",
        size,
        imgui::ButtonFlags::MOUSE_BUTTON_LEFT
            | imgui::ButtonFlags::MOUSE_BUTTON_RIGHT
            | imgui::ButtonFlags::MOUSE_BUTTON_MIDDLE,
    );
    let hovered = ui.is_item_hovered();
    let mouse = ui.io().mouse_pos;
    let world_mouse = e.view_2d.screen_to_world(mouse, origin, size);
    drop_target(ui, e, world_mouse);
    let before = views(e);
    // A map opened in this view starts on what its game camera shows, or on all
    // of its content, instead of on an arbitrary origin.
    let identity = format!("{}:{}", e.root.display(), e.scene.name);
    if e.world2d.framed.as_deref() != Some(identity.as_str()) && size[0] > 64. && size[1] > 64. {
        e.world2d.framed = Some(identity);
        frame_start(e, &before, size);
    }
    if std::mem::take(&mut e.world2d.frame_request) {
        frame(e, &before, size);
    }
    input(ui, e, &before, origin, size, hovered, world_mouse, painting);
    let after = views(e);
    draw(ui, e, &after, origin, size, world_mouse);
    if e.world2d.open_context {
        e.world2d.open_context = false;
        ui.open_popup("world2d-context");
    }
    context_menu(ui, e);
    if after.is_empty() {
        empty_state(ui, e, origin, size);
    }
    ui.set_cursor_screen_pos([origin[0], origin[1] + size[1] + 2.]);
    status_bar(ui, e, &after, world_mouse, hovered);
    if painting {
        // A splitter to resize the palette, then the palette itself.
        let start = ui.cursor_screen_pos();
        ui.invisible_button("##palette-splitter", [available[0].max(1.), 6.]);
        if ui.is_item_hovered() || ui.is_item_active() {
            ui.set_mouse_cursor(Some(imgui::MouseCursor::ResizeNS));
        }
        if ui.is_item_active() {
            e.world2d.tiles.panel_height -= ui.io().mouse_delta[1];
        }
        ui.get_window_draw_list()
            .add_line(
                [start[0] + available[0] * 0.5 - 18., start[1] + 3.],
                [start[0] + available[0] * 0.5 + 18., start[1] + 3.],
                [1., 1., 1., 0.25],
            )
            .thickness(2.)
            .build();
        let height = e.world2d.tiles.panel_height;
        crate::tilemap_editor::palette(ui, e, height);
    }
    if hovered || ui.is_window_focused() {
        let handled = painting && crate::tilemap_editor::shortcuts(ui, e);
        if !handled {
            let painting = crate::tilemap_editor::painting(e);
            shortcuts(ui, e, &after, size, painting);
        }
    }
    crate::gui::scene_shortcuts(ui, e);
}
fn frame(e: &mut Editor, views: &[View], size: [f32; 2]) {
    let selected: Vec<_> = views
        .iter()
        .filter(|v| Some(v.id) == e.selected_actor)
        .collect();
    let targets: Vec<&View> = if selected.is_empty() {
        views.iter().collect()
    } else {
        selected
    };
    if targets.is_empty() {
        e.view_2d.reset();
        return;
    }
    let mut lo = [f32::MAX; 2];
    let mut hi = [f32::MIN; 2];
    for view in targets {
        for p in corners(view) {
            for k in 0..2 {
                lo[k] = lo[k].min(p[k]);
                hi[k] = hi[k].max(p[k]);
            }
        }
    }
    e.view_2d.frame(lo, hi, size);
}
fn frame_start(e: &mut Editor, views: &[View], size: [f32; 2]) {
    let display = e.scene.display_size.map(f32::from);
    if let Some(v) = views
        .iter()
        .find(|v| v.active && v.camera.as_ref().is_some_and(|c| c.current))
    {
        let camera = v.camera.as_ref().unwrap();
        let half = [
            display[0] * 0.5 / camera.zoom.max(0.0625),
            display[1] * 0.5 / camera.zoom.max(0.0625),
        ];
        let mut centre = [
            v.world.t[0] + camera.offset[0],
            v.world.t[1] + camera.offset[1],
        ];
        if camera.limits_enabled {
            for k in 0..2 {
                let (lo, hi) = (camera.limits[k] + half[k], camera.limits[k + 2] - half[k]);
                centre[k] = if lo > hi {
                    (camera.limits[k] + camera.limits[k + 2]) * 0.5
                } else {
                    centre[k].clamp(lo, hi)
                };
            }
        }
        e.view_2d.frame(
            [centre[0] - half[0], centre[1] - half[1]],
            [centre[0] + half[0], centre[1] + half[1]],
            size,
        );
        // Framing fits with a margin; keep the screen at a crisp integer zoom.
        e.view_2d.zoom = e.view_2d.zoom.max(1.);
    } else if !views.is_empty() {
        let selected = e.selected_actor;
        e.selected_actor = None;
        frame(e, views, size);
        e.selected_actor = selected;
    }
}
fn drop_target(ui: &Ui, e: &mut Editor, world: [f32; 2]) {
    if let Some(target) = ui.drag_drop_target()
        && let Some(Ok(payload)) =
            target.accept_payload::<u8, _>("EPOK_CONTENT", imgui::DragDropFlags::empty())
        && payload.delivery
    {
        let paths = e.project_browser.drag.clone();
        let mut placed = 0;
        for (n, path) in paths.iter().enumerate() {
            if let Some(texture) = texture_for_path(e, path) {
                let name = std::path::Path::new(path)
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .split('.')
                    .next()
                    .unwrap_or("Sprite")
                    .to_owned();
                let offset = n as f32 * 16.;
                create_sprite_from_texture(
                    e,
                    texture,
                    &name,
                    snapped(e, [world[0] + offset, world[1] + offset]),
                );
                placed += 1;
            }
        }
        if placed == 0 && !paths.is_empty() {
            e.log("Drop an imported Texture to create a Sprite2D. Import PNG files first.");
        }
    }
}
struct Handles {
    corners: [[f32; 2]; 4],
    knob: [f32; 2],
    top: [f32; 2],
    edges: [[f32; 2]; 4],
}
fn handles(view: &View, e: &Editor, origin: [f32; 2], size: [f32; 2]) -> Handles {
    let c = corners(view).map(|p| e.view_2d.world_to_screen(p, origin, size));
    let mid = |a: [f32; 2], b: [f32; 2]| [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5];
    let edges = [
        mid(c[0], c[1]),
        mid(c[1], c[2]),
        mid(c[2], c[3]),
        mid(c[3], c[0]),
    ];
    let centre = mid(c[0], c[2]);
    let up = [edges[0][0] - centre[0], edges[0][1] - centre[1]];
    let length = (up[0] * up[0] + up[1] * up[1]).sqrt().max(1.);
    let knob = [
        edges[0][0] + up[0] / length * 26.,
        edges[0][1] + up[1] / length * 26.,
    ];
    Handles {
        corners: c,
        knob,
        top: edges[0],
        edges,
    }
}
fn near(a: [f32; 2], b: [f32; 2], radius: f32) -> bool {
    let d = [a[0] - b[0], a[1] - b[1]];
    d[0] * d[0] + d[1] * d[1] <= radius * radius
}
/// Collider resize handles in screen space: (component, side, position).
fn collider_handles(
    view: &View,
    e: &Editor,
    origin: [f32; 2],
    size: [f32; 2],
) -> Vec<(Uuid, usize, [f32; 2])> {
    let mut out = Vec::new();
    for (id, c) in &view.colliders {
        let (centre, half) = runtime_box(view, c);
        let points = match c.shape {
            Shape2D::Box => vec![
                (0, [centre[0] + half[0], centre[1]]),
                (1, [centre[0], centre[1] + half[1]]),
                (2, [centre[0] - half[0], centre[1]]),
                (3, [centre[0], centre[1] - half[1]]),
            ],
            Shape2D::Circle => vec![(0, [centre[0] + half[0], centre[1]])],
        };
        for (side, p) in points {
            out.push((*id, side, e.view_2d.world_to_screen(p, origin, size)));
        }
    }
    out
}
/// A collider exactly as the runtime resolves it: centred at the transformed offset,
/// axis-aligned, extents scaled by the owner's scale.
fn runtime_box(view: &View, c: &Collider2D) -> ([f32; 2], [f32; 2]) {
    let centre = view.world.point(c.offset);
    let scale = [
        (view.world.m[0][0].powi(2) + view.world.m[1][0].powi(2)).sqrt(),
        (view.world.m[0][1].powi(2) + view.world.m[1][1].powi(2)).sqrt(),
    ];
    let half = match c.shape {
        Shape2D::Box => [
            c.size[0].abs() * 0.5 * scale[0],
            c.size[1].abs() * 0.5 * scale[1],
        ],
        Shape2D::Circle => [c.radius.abs() * scale[0].max(scale[1]); 2],
    };
    (centre, half)
}
fn pick(views: &[View], world: [f32; 2]) -> Option<&View> {
    views.iter().rev().find(|view| contains(view, world))
}
fn rotation_frame(view: &View) -> Affine {
    // Parent * translate * rotate, without the actor's own scale.
    let unscaled = Transform2D {
        scale: [1., 1.],
        ..view.local
    };
    view.parent.compose(&Affine::local(&unscaled))
}
#[allow(clippy::too_many_arguments)]
fn input(
    ui: &Ui,
    e: &mut Editor,
    views: &[View],
    origin: [f32; 2],
    size: [f32; 2],
    hovered: bool,
    world: [f32; 2],
    painting: bool,
) {
    let io = ui.io();
    let mouse = io.mouse_pos;
    let typing = crate::gui::text_input_active(ui);
    if hovered && io.mouse_wheel != 0. {
        e.view_2d.zoom_at(io.mouse_wheel, mouse, origin, size);
    }
    let space = hovered && !typing && ui.is_key_down(imgui::Key::Space);
    if painting {
        // The right button erases while painting: pan with the middle button or Space.
        if (ui.is_mouse_dragging(MouseButton::Middle)
            || (ui.is_key_down(imgui::Key::Space) && ui.is_mouse_dragging(MouseButton::Left)))
            && (hovered
                || ui.is_mouse_down(MouseButton::Middle)
                || ui.is_mouse_down(MouseButton::Left))
        {
            e.view_2d.pan_pixels(io.mouse_delta);
        }
        e.world2d.hovered = None;
        crate::tilemap_editor::input(ui, e, hovered, world);
        return;
    }
    if (ui.is_mouse_dragging(MouseButton::Middle)
        || ui.is_mouse_dragging_with_threshold(MouseButton::Right, 4.))
        && (hovered
            || ui.is_mouse_down(MouseButton::Middle)
            || ui.is_mouse_down(MouseButton::Right))
        && e.world2d.drag.is_none()
    {
        e.view_2d.pan_pixels(io.mouse_delta);
        e.world2d.right_dragged = true;
    }
    // Points of the selected path, line or polygon take clicks before the node tools.
    if !e.playing
        && matches!(e.world2d.tool, Tool::Select | Tool::Move)
        && e.world2d.drag.is_none()
        && let Some(view) = views
            .iter()
            .find(|v| Some(v.id) == e.selected_actor)
            .cloned()
    {
        let camera = e.view_2d;
        let to_screen = move |p: [f32; 2]| camera.world_to_screen(p, origin, size);
        let (snap_on, step) = (e.world2d.snap, e.world2d.grid_step);
        let snap = move |p: [f32; 2]| {
            if snap_on {
                p.map(|v| snap_value(v, step))
            } else {
                p.map(|v| v.round())
            }
        };
        let right = hovered && ui.is_mouse_clicked(MouseButton::Right);
        if crate::world2d_shapes_editor::input(ui, e, &view, &to_screen, hovered, world, &snap) {
            if right {
                // The click deleted a point: no context menu on release.
                e.world2d.right_dragged = true;
            }
            return;
        }
    }
    if hovered && ui.is_mouse_clicked(MouseButton::Right) {
        e.world2d.right_dragged = false;
    }
    if hovered && ui.is_mouse_released(MouseButton::Right) && !e.world2d.right_dragged && !e.playing
    {
        e.world2d.context = world;
        if let Some(view) = pick(views, world) {
            e.select_actor(Some(view.id));
        }
        e.world2d.open_context = true;
    }
    e.world2d.hovered = if hovered {
        pick(views, world).map(|v| v.id)
    } else {
        None
    };
    if hovered && ui.is_mouse_clicked(MouseButton::Left) && !e.playing {
        let tool = if space { Tool::Pan } else { e.world2d.tool };
        let selected = views.iter().find(|v| Some(v.id) == e.selected_actor);
        let mut kind = None;
        let mut actor = None;
        if let Some(view) = selected
            && matches!(tool, Tool::Select)
        {
            let h = handles(view, e, origin, size);
            if near(mouse, h.knob, HANDLE + 2.) {
                let pivot = view.world.t;
                kind = Some(DragKind::Rotate {
                    start_angle: (world[1] - pivot[1])
                        .atan2(world[0] - pivot[0])
                        .to_degrees(),
                    start: view.local.rotation,
                });
            } else if let Some(corner) = h.corners.iter().position(|p| near(mouse, *p, HANDLE)) {
                let _ = corner;
                kind = scale_drag(view, world, [true, true]);
            } else if let Some(edge) = h.edges.iter().position(|p| near(mouse, *p, HANDLE)) {
                kind = scale_drag(
                    view,
                    world,
                    if edge % 2 == 0 {
                        [false, true]
                    } else {
                        [true, false]
                    },
                );
            } else if e.world2d.show_colliders
                && let Some((component, side, _)) = collider_handles(view, e, origin, size)
                    .into_iter()
                    .find(|(_, _, p)| near(mouse, *p, HANDLE))
            {
                let start = view
                    .colliders
                    .iter()
                    .find(|(id, _)| *id == component)
                    .map(|c| c.1.clone())
                    .unwrap_or_default();
                kind = Some(DragKind::Collider {
                    component,
                    side,
                    start,
                });
            }
            if kind.is_some() {
                actor = Some(view.id);
            }
        }
        if kind.is_none() {
            let hit = pick(views, world);
            match tool {
                Tool::Pan => kind = Some(DragKind::Pan),
                Tool::Select | Tool::Move => {
                    let target = hit.or(if tool == Tool::Move { selected } else { None });
                    match target {
                        Some(view) => {
                            e.select_actor(Some(view.id));
                            actor = Some(view.id);
                            kind = Some(DragKind::Move {
                                grab: [view.world.t[0] - world[0], view.world.t[1] - world[1]],
                            });
                        }
                        None => e.select_actor(None),
                    }
                }
                Tool::Rotate | Tool::Scale => {
                    if let Some(view) = hit.or(selected) {
                        e.select_actor(Some(view.id));
                        actor = Some(view.id);
                        kind = if tool == Tool::Rotate {
                            let pivot = view.world.t;
                            Some(DragKind::Rotate {
                                start_angle: (world[1] - pivot[1])
                                    .atan2(world[0] - pivot[0])
                                    .to_degrees(),
                                start: view.local.rotation,
                            })
                        } else {
                            scale_drag(view, world, [true, true])
                        };
                    }
                }
            }
        }
        e.world2d.drag = kind.map(|kind| Drag {
            actor,
            kind,
            moved: false,
        });
    }
    if let Some(drag) = e.world2d.drag.clone() {
        if !ui.is_mouse_down(MouseButton::Left) {
            e.world2d.drag = None;
            e.end_coalesced();
            return;
        }
        let dragging = ui.is_mouse_dragging_with_threshold(MouseButton::Left, 2.) || drag.moved;
        if !dragging {
            return;
        }
        if let Some(d) = &mut e.world2d.drag {
            d.moved = true;
        }
        let free = ui.io().key_alt;
        let shift = ui.io().key_shift;
        let Some(view) = drag
            .actor
            .and_then(|id| views.iter().find(|v| v.id == id))
            .cloned()
        else {
            if matches!(drag.kind, DragKind::Pan) {
                e.view_2d.pan_pixels(io.mouse_delta);
            }
            return;
        };
        let snap = e.world2d.snap && !free;
        let step = e.world2d.grid_step;
        match drag.kind {
            DragKind::Pan => e.view_2d.pan_pixels(io.mouse_delta),
            DragKind::Move { grab } => {
                let mut target = [world[0] + grab[0], world[1] + grab[1]];
                if snap {
                    target = target.map(|v| snap_value(v, step));
                }
                let local = view
                    .parent
                    .inverse()
                    .map(|i| i.point(target))
                    .unwrap_or(target);
                edit_transform(e, view.id, |t| t.position = local);
                e.changed_coalesced("world2d-move");
            }
            DragKind::Rotate { start_angle, start } => {
                let pivot = view.world.t;
                let angle = (world[1] - pivot[1])
                    .atan2(world[0] - pivot[0])
                    .to_degrees();
                let mut rotation = start + (angle - start_angle);
                if snap || shift {
                    rotation = snap_value(rotation, e.world2d.rotation_step);
                }
                let rotation = ((rotation + 180.).rem_euclid(360.)) - 180.;
                edit_transform(e, view.id, |t| t.rotation = rotation);
                e.changed_coalesced("world2d-rotate");
            }
            DragKind::Scale {
                axes,
                start_local,
                start,
            } => {
                let Some(inverse) = rotation_frame(&view).inverse() else {
                    return;
                };
                let local = inverse.point(world);
                let mut factor = [1., 1.];
                for k in 0..2 {
                    if axes[k] && start_local[k].abs() > 0.5 {
                        factor[k] = local[k] / start_local[k];
                    }
                }
                if shift
                    || e.world2d.keep_ratio
                    || (axes[0] && axes[1] && e.world2d.tool == Tool::Scale)
                {
                    let uniform = if axes[0] && axes[1] {
                        (factor[0] + factor[1]) * 0.5
                    } else if axes[0] {
                        factor[0]
                    } else {
                        factor[1]
                    };
                    factor = [uniform, uniform];
                }
                let mut scale = [start[0] * factor[0], start[1] * factor[1]];
                if snap {
                    scale = scale.map(|v| snap_value(v, 0.125));
                }
                edit_transform(e, view.id, |t| {
                    t.scale = scale.map(|v| {
                        if v.abs() < 0.0625 {
                            0.0625_f32.copysign(v)
                        } else {
                            v
                        }
                    })
                });
                e.changed_coalesced("world2d-scale");
            }
            DragKind::Collider {
                component,
                side,
                start,
            } => {
                let Some(inverse) = view.world.inverse() else {
                    return;
                };
                let p = inverse.point(world);
                let mut collider = start.clone();
                let round = |v: f32| {
                    if snap {
                        snap_value(v, (step / 2.).max(1.))
                    } else {
                        v.round()
                    }
                };
                match collider.shape {
                    Shape2D::Box => {
                        let axis = side % 2;
                        collider.size[axis] =
                            round(((p[axis] - start.offset[axis]).abs() * 2.).max(1.));
                    }
                    Shape2D::Circle => {
                        let d = [p[0] - start.offset[0], p[1] - start.offset[1]];
                        collider.radius = round((d[0] * d[0] + d[1] * d[1]).sqrt().max(1.));
                    }
                }
                edit_component::<Collider2D>(e, view.id, component, |c| *c = collider);
                e.changed_coalesced("world2d-collider");
            }
        }
    }
}
fn scale_drag(view: &View, world: [f32; 2], axes: [bool; 2]) -> Option<DragKind> {
    let inverse = rotation_frame(view).inverse()?;
    let local = inverse.point(world);
    Some(DragKind::Scale {
        axes,
        start_local: local,
        start: view.local.scale,
    })
}
fn shortcuts(ui: &Ui, e: &mut Editor, views: &[View], size: [f32; 2], painting: bool) {
    if crate::gui::text_input_active(ui) || e.playing {
        return;
    }
    let io = ui.io();
    let command = io.key_ctrl || io.key_super;
    if painting {
        // Painting owns the letter keys, Delete and the arrows; framing and zoom stay.
        if !command && ui.is_key_pressed_no_repeat(imgui::Key::F) {
            frame(e, views, size);
        }
        if ui.is_key_pressed(imgui::Key::Equal) || ui.is_key_pressed(imgui::Key::KeypadAdd) {
            e.view_2d.zoom = (e.view_2d.zoom * 2.).min(SceneView2D::MAX_ZOOM);
        }
        if ui.is_key_pressed(imgui::Key::Minus) || ui.is_key_pressed(imgui::Key::KeypadSubtract) {
            e.view_2d.zoom = (e.view_2d.zoom / 2.).max(SceneView2D::MIN_ZOOM);
        }
        return;
    }
    if !command {
        for (key, tool) in [
            (imgui::Key::Q, Tool::Select),
            (imgui::Key::W, Tool::Move),
            (imgui::Key::E, Tool::Rotate),
            (imgui::Key::S, Tool::Scale),
        ] {
            if ui.is_key_pressed_no_repeat(key) {
                e.world2d.tool = tool;
            }
        }
        if ui.is_key_pressed_no_repeat(imgui::Key::G) {
            e.world2d.snap = !e.world2d.snap;
        }
        if ui.is_key_pressed_no_repeat(imgui::Key::F) {
            frame(e, views, size);
        }
        if ui.is_key_pressed_no_repeat(imgui::Key::Escape) {
            e.select_actor(None);
        }
        if ui.is_key_pressed(imgui::Key::Equal) || ui.is_key_pressed(imgui::Key::KeypadAdd) {
            e.view_2d.zoom = (e.view_2d.zoom * 2.).min(SceneView2D::MAX_ZOOM);
        }
        if ui.is_key_pressed(imgui::Key::Minus) || ui.is_key_pressed(imgui::Key::KeypadSubtract) {
            e.view_2d.zoom = (e.view_2d.zoom / 2.).max(SceneView2D::MIN_ZOOM);
        }
    }
    let Some(selected) = e
        .selected_actor
        .filter(|id| views.iter().any(|v| v.id == *id))
    else {
        return;
    };
    if (ui.is_key_pressed_no_repeat(imgui::Key::Delete)
        || ui.is_key_pressed_no_repeat(imgui::Key::Backspace))
        && !command
    {
        // A selected path, line or polygon point goes first, then the node.
        if !crate::world2d_shapes_editor::delete_selected_point(e) {
            e.delete_actor(selected);
        }
        return;
    }
    if command && ui.is_key_pressed_no_repeat(imgui::Key::D) {
        e.duplicate_actor(selected);
        return;
    }
    let step = if io.key_shift {
        e.world2d.grid_step.max(1.)
    } else {
        1.
    };
    let mut delta = [0., 0.];
    for (key, d) in [
        (imgui::Key::LeftArrow, [-step, 0.]),
        (imgui::Key::RightArrow, [step, 0.]),
        (imgui::Key::UpArrow, [0., -step]),
        (imgui::Key::DownArrow, [0., step]),
    ] {
        if ui.is_key_pressed(key) {
            delta = d;
        }
    }
    if delta != [0., 0.] {
        edit_transform(e, selected, |t| {
            t.position[0] += delta[0];
            t.position[1] += delta[1];
        });
        e.changed_coalesced("world2d-nudge");
    }
}
fn context_menu(ui: &Ui, e: &mut Editor) {
    ui.popup("world2d-context", || {
        let position = e.world2d.context;
        if let Some(_menu) = ui.begin_menu(format!("{ICON_PLUS}  Add 2D Node Here")) {
            let mut category = "";
            for preset in Preset::ALL {
                if preset.category() != category {
                    if !category.is_empty() {
                        ui.separator();
                    }
                    category = preset.category();
                    ui.text_disabled(category);
                }
                if ui.menu_item(format!("{}  {}", preset_icon(preset), preset.label())) {
                    create_node(e, preset, snapped(e, position));
                }
            }
        }
        if let Some(selected) = e.selected_actor {
            ui.separator();
            if ui
                .menu_item_config(format!("{ICON_CLONE}  Duplicate"))
                .shortcut("Ctrl+D")
                .build()
            {
                e.duplicate_actor(selected);
            }
            if ui
                .menu_item_config(format!("{ICON_FRAME}  Frame Selection"))
                .shortcut("F")
                .build()
            {
                e.world2d.frame_request = true;
            }
            if ui
                .menu_item_config(format!("{ICON_TRASH}  Delete"))
                .shortcut("Del")
                .build()
            {
                e.delete_actor(selected);
            }
        }
        ui.separator();
        if ui.menu_item(format!("{ICON_FRAME}  Frame Whole Scene")) {
            e.select_actor(None);
            e.world2d.frame_request = true;
        }
    });
}
fn empty_state(ui: &Ui, e: &mut Editor, origin: [f32; 2], size: [f32; 2]) {
    let width = 420_f32.min(size[0] - 24.);
    let height = 196.;
    if width < 200. || size[1] < height + 20. {
        return;
    }
    let corner = [
        origin[0] + (size[0] - width) * 0.5,
        origin[1] + (size[1] - height) * 0.5,
    ];
    let draw = ui.get_window_draw_list();
    draw.add_rect(
        corner,
        [corner[0] + width, corner[1] + height],
        [0.16, 0.17, 0.20, 0.96],
    )
    .filled(true)
    .rounding(8.)
    .build();
    draw.add_rect(
        corner,
        [corner[0] + width, corner[1] + height],
        [1., 1., 1., 0.08],
    )
    .rounding(8.)
    .build();
    ui.set_cursor_screen_pos([corner[0] + 18., corner[1] + 16.]);
    ui.text("Start your 2D world");
    ui.set_cursor_screen_pos([corner[0] + 18., corner[1] + 38.]);
    ui.text_disabled("Add a node, or drag an imported texture here to create a sprite.");
    let presets = [
        Preset::Sprite2D,
        Preset::CharacterBody2D,
        Preset::StaticBody2D,
        Preset::Camera2D,
    ];
    let button = [(width - 36. - 3. * 8.) / 4., 92.];
    for (i, preset) in presets.into_iter().enumerate() {
        ui.set_cursor_screen_pos([
            corner[0] + 18. + i as f32 * (button[0] + 8.),
            corner[1] + 68.,
        ]);
        let _id = ui.push_id(preset.label());
        if ui.button_with_size("##preset", button) {
            let centre = e.view_2d.centre;
            create_node(e, preset, snapped(e, centre));
        }
        if ui.is_item_hovered() {
            ui.tooltip_text(preset.description());
        }
        let min = ui.item_rect_min();
        let icon = preset_icon(preset);
        let icon_size = ui.calc_text_size(icon);
        draw.add_text(
            [min[0] + (button[0] - icon_size[0]) * 0.5, min[1] + 22.],
            [0.98, 0.70, 0.35, 1.],
            icon,
        );
        let label = preset.label();
        let label_size = ui.calc_text_size(label);
        draw.add_text(
            [
                min[0] + (button[0] - label_size[0]).max(0.) * 0.5,
                min[1] + 56.,
            ],
            [0.9, 0.9, 0.92, 1.],
            label,
        );
    }
    ui.set_cursor_screen_pos([corner[0] + 18., corner[1] + 168.]);
    ui.text_disabled(format!(
        "{ICON_INFO}  Pixels, Y down: 1 unit = 1 screen pixel at 100%."
    ));
}
fn status_bar(ui: &Ui, e: &Editor, views: &[View], world: [f32; 2], hovered: bool) {
    let selected = views.iter().find(|v| Some(v.id) == e.selected_actor);
    let mut text = String::new();
    if hovered {
        text += &format!("X {:>6.0}  Y {:>6.0}   ", world[0], world[1]);
        if let Some(cell) = e.world2d.tiles.hovered_cell()
            && e.world2d.tiles.tool.is_some()
            && let Some(t) = selected.and_then(|v| v.tiles.as_ref())
        {
            text += &format!("Cell ({}, {})", cell[0], cell[1]);
            if let Some(c) = t.cells.get(&cell) {
                text += &format!(" tile ({}, {})", c.atlas[0], c.atlas[1]);
            }
            text += "   ";
        }
    }
    text += &format!(
        "Zoom {:.0}%   {} node{}",
        e.view_2d.zoom * 100.,
        views.len(),
        if views.len() == 1 { "" } else { "s" }
    );
    if let Some(view) = selected {
        text += &format!(
            "   Selected: {} at ({:.0}, {:.0})",
            view.name, view.local.position[0], view.local.position[1]
        );
    }
    if e.world2d.snap {
        text += &format!("   Snap {:.0} px", e.world2d.grid_step);
    }
    ui.text_disabled(text);
}

// ---- drawing ---------------------------------------------------------------------------
/// The frame an animated sprite shows at editor time `seconds`.
fn preview_rect(
    sprite: &Sprite2D,
    size: [u16; 2],
    seconds: f32,
    animate: bool,
) -> Option<[u16; 4]> {
    let animation = sprite.animations.get(usize::from(sprite.animation));
    if let Some(animation) = animation.filter(|a| !a.frames.is_empty()) {
        let mut index = 0;
        if animate {
            let total: f32 = animation
                .frames
                .iter()
                .map(|f| animation.frame_seconds(f))
                .sum();
            let mut t = if animation.looping {
                seconds % total.max(1e-3)
            } else {
                seconds.min(total)
            };
            for (i, frame) in animation.frames.iter().enumerate() {
                index = i;
                let d = animation.frame_seconds(frame);
                if t < d {
                    break;
                }
                t -= d;
            }
        }
        let r = animation.frames[index].region;
        return (r[2] > 0
            && r[3] > 0
            && u32::from(r[0]) + u32::from(r[2]) <= u32::from(size[0])
            && u32::from(r[1]) + u32::from(r[3]) <= u32::from(size[1]))
        .then_some(r);
    }
    sprite.source_rect(size)
}
pub(crate) fn dashed(
    draw: &imgui::DrawListMut,
    a: [f32; 2],
    b: [f32; 2],
    colour: [f32; 4],
    dash: f32,
) {
    let d = [b[0] - a[0], b[1] - a[1]];
    let length = (d[0] * d[0] + d[1] * d[1]).sqrt();
    if length < 1. {
        return;
    }
    let steps = (length / dash).ceil() as usize;
    for i in (0..steps).step_by(2) {
        let t0 = i as f32 * dash / length;
        let t1 = ((i + 1) as f32 * dash / length).min(1.);
        draw.add_line(
            [a[0] + d[0] * t0, a[1] + d[1] * t0],
            [a[0] + d[0] * t1, a[1] + d[1] * t1],
            colour,
        )
        .build();
    }
}
fn draw(
    ui: &Ui,
    e: &mut Editor,
    views: &[View],
    origin: [f32; 2],
    size: [f32; 2],
    world_mouse: [f32; 2],
) {
    let draw = ui.get_window_draw_list();
    let corner = [origin[0] + size[0], origin[1] + size[1]];
    let view = e.view_2d;
    let to_screen = |p: [f32; 2]| view.world_to_screen(p, origin, size);
    let seconds = ui.time() as f32;
    draw.with_clip_rect_intersect(origin, corner, || {
        draw.add_rect(origin, corner, CANVAS).filled(true).build();
        // Grid: the snap step when it is readable, doubled until it is.
        if e.world2d.show_grid {
            let mut minor = e.world2d.grid_step.max(1.);
            while minor * view.zoom < 7. {
                minor *= 2.;
            }
            let major = minor * 8.;
            let top_left = view.screen_to_world(origin, origin, size);
            let bottom_right = view.screen_to_world(corner, origin, size);
            for (step, colour) in [(minor, GRID_MINOR), (major, GRID_MAJOR)] {
                let mut x = (top_left[0] / step).floor() * step;
                while x <= bottom_right[0] {
                    let sx = to_screen([x, 0.])[0];
                    draw.add_line([sx, origin[1]], [sx, corner[1]], colour)
                        .build();
                    x += step;
                }
                let mut y = (top_left[1] / step).floor() * step;
                while y <= bottom_right[1] {
                    let sy = to_screen([0., y])[1];
                    draw.add_line([origin[0], sy], [corner[0], sy], colour)
                        .build();
                    y += step;
                }
            }
        }
        let zero = to_screen([0., 0.]);
        draw.add_line([origin[0], zero[1]], [corner[0], zero[1]], AXIS_X)
            .build();
        draw.add_line([zero[0], origin[1]], [zero[0], corner[1]], AXIS_Y)
            .build();
        // The game screen: the current camera's view, or the default view at the origin.
        let display = e.scene.display_size.map(f32::from);
        let current_camera = views
            .iter()
            .find(|v| v.camera.as_ref().is_some_and(|c| c.current) && v.active);
        if e.world2d.show_camera {
            match current_camera {
                Some(camera_view) => {
                    let camera = camera_view.camera.as_ref().unwrap();
                    let zoom = camera.zoom.max(0.0625);
                    let mut centre = [
                        camera_view.world.t[0] + camera.offset[0],
                        camera_view.world.t[1] + camera.offset[1],
                    ];
                    let half = [display[0] * 0.5 / zoom, display[1] * 0.5 / zoom];
                    if camera.limits_enabled {
                        for k in 0..2 {
                            let (lo, hi) =
                                (camera.limits[k] + half[k], camera.limits[k + 2] - half[k]);
                            centre[k] = if lo > hi {
                                (camera.limits[k] + camera.limits[k + 2]) * 0.5
                            } else {
                                centre[k].clamp(lo, hi)
                            };
                        }
                        let a = to_screen([camera.limits[0], camera.limits[1]]);
                        let b = to_screen([camera.limits[2], camera.limits[3]]);
                        let c = [[a[0], a[1]], [b[0], a[1]], [b[0], b[1]], [a[0], b[1]]];
                        for i in 0..4 {
                            dashed(&draw, c[i], c[(i + 1) % 4], alpha(CAMERA_COLOUR, 0.55), 6.);
                        }
                    }
                    let a = to_screen([centre[0] - half[0], centre[1] - half[1]]);
                    let b = to_screen([centre[0] + half[0], centre[1] + half[1]]);
                    // What the game clears to, so the view reads like the game screen.
                    let sky = camera.background.map(|c| f32::from(c) / 255.);
                    draw.add_rect(a, b, [sky[0], sky[1], sky[2], 0.55])
                        .filled(true)
                        .build();
                    draw.add_rect(a, b, alpha(CAMERA_COLOUR, 0.9))
                        .thickness(1.5)
                        .build();
                    draw.add_text(
                        [a[0] + 4., a[1] + 3.],
                        alpha(CAMERA_COLOUR, 0.9),
                        format!(
                            "{ICON_CAMERA} {} ({}x{})",
                            camera_view.name, display[0], display[1]
                        ),
                    );
                }
                None => {
                    let a = to_screen([-display[0] * 0.5, -display[1] * 0.5]);
                    let b = to_screen([display[0] * 0.5, display[1] * 0.5]);
                    draw.add_rect(a, b, [0.45, 0.62, 0.95, 0.55])
                        .thickness(1.)
                        .build();
                    draw.add_text(
                        [a[0] + 4., a[1] + 3.],
                        [0.45, 0.62, 0.95, 0.7],
                        format!("Default view {}x{}", display[0], display[1]),
                    );
                }
            }
        }
        // CanvasModulate2D tints, per canvas (the first one wins, as on the console).
        let mut tints: BTreeMap<i8, [f32; 3]> = BTreeMap::new();
        for v in views.iter().filter(|v| v.active) {
            if let Some(m) = &v.modulate {
                tints
                    .entry(v.canvas)
                    .or_insert(m.color.map(|c| f32::from(c) / 255.));
            }
        }
        let visible = (
            view.screen_to_world(origin, origin, size),
            view.screen_to_world(corner, origin, size),
        );
        let painting = crate::tilemap_editor::painting(e)
            .then_some(e.selected_actor)
            .flatten();
        // Screen canvases draw their nodes in screen pixels from the origin.
        if e.world2d.show_camera
            && views
                .iter()
                .any(|v| v.canvas_layer.as_ref().is_some_and(|c| !c.follow_viewport))
        {
            let a = to_screen([0., 0.]);
            let b = to_screen(display);
            let colour = [0.55, 0.85, 0.75, 0.55];
            for (p, q) in [
                (a, [b[0], a[1]]),
                ([b[0], a[1]], b),
                (b, [a[0], b[1]]),
                ([a[0], b[1]], a),
            ] {
                dashed(&draw, p, q, colour, 5.);
            }
            draw.add_text(
                [a[0] + 4., b[1] - 16.],
                colour,
                format!("{ICON_CANVAS} Screen canvas"),
            );
        }
        // Content, back to front.
        for v in views {
            let fade = if v.active { 1. } else { 0.35 };
            let tint = tints.get(&v.canvas).copied().unwrap_or([1.; 3]);
            let mut drew = false;
            // Parallax repeats: the original plus faded copies across the visible area.
            let mut shifts = vec![[0., 0.]];
            for k in 0..2 {
                let period = v.repeat[k];
                if period >= 4. {
                    let span = visible.1[k] - visible.0[k];
                    let copies = ((span / period).ceil() as i32 + 1).min(24);
                    let base: Vec<[f32; 2]> = shifts.clone();
                    for b in base {
                        for n in -copies..=copies {
                            if n != 0 {
                                let mut shifted = b;
                                shifted[k] += n as f32 * period;
                                shifts.push(shifted);
                            }
                        }
                    }
                }
            }
            if let Some(t) = &v.tiles
                && t.layer.visible
                && let Some(tileset) = &t.tileset
            {
                let texture = tileset.texture.and_then(|id| e.world2d.textures.get(id));
                let dim = if painting.is_some_and(|p| p != v.id) {
                    0.45
                } else {
                    1.
                };
                for (n, shift) in shifts.iter().enumerate() {
                    let origin_world = [t.origin[0] + shift[0], t.origin[1] + shift[1]];
                    let cells = crate::tilemap_editor::visible_cells(
                        &t.cells,
                        origin_world,
                        t.tile_size(),
                        visible,
                    );
                    let m = t.layer.modulate.map(|c| f32::from(c) / 255.);
                    let alpha = match t.layer.blend {
                        crate::texture::BlendMode::Cutout => 1.,
                        crate::texture::BlendMode::AddQuarter => 0.3,
                        _ => 0.6,
                    } * fade
                        * dim
                        * if n == 0 { 1. } else { 0.45 };
                    crate::tilemap_editor::draw_cells(
                        &draw,
                        texture,
                        tileset,
                        cells.into_iter(),
                        origin_world,
                        &to_screen,
                        visible,
                        [m[0] * tint[0], m[1] * tint[1], m[2] * tint[2], alpha],
                        if e.world2d.animate { seconds } else { 0. },
                    );
                }
                drew = true;
            }
            if let Some((_, sprite)) = &v.sprite
                && sprite.visible
                && let Some(texture) = sprite.texture
                && let Some((texture_id, texture_size)) = e.world2d.textures.get(texture)
                && let Some(rect) = preview_rect(sprite, texture_size, seconds, e.world2d.animate)
            {
                let (lo, hi) = sprite_local_rect(sprite, [f32::from(rect[2]), f32::from(rect[3])]);
                let (w, h) = (f32::from(texture_size[0]), f32::from(texture_size[1]));
                let (mut u0, mut u1) = (f32::from(rect[0]) / w, f32::from(rect[0] + rect[2]) / w);
                let (mut v0, mut v1) = (f32::from(rect[1]) / h, f32::from(rect[1] + rect[3]) / h);
                if sprite.flip_h {
                    std::mem::swap(&mut u0, &mut u1);
                }
                if sprite.flip_v {
                    std::mem::swap(&mut v0, &mut v1);
                }
                for (n, shift) in shifts.iter().enumerate() {
                    let quad = [lo, [hi[0], lo[1]], hi, [lo[0], hi[1]]].map(|p| {
                        let w = v.world.point(p);
                        to_screen([w[0] + shift[0], w[1] + shift[1]])
                    });
                    let colour = [
                        f32::from(sprite.modulate[0]) / 255. * tint[0],
                        f32::from(sprite.modulate[1]) / 255. * tint[1],
                        f32::from(sprite.modulate[2]) / 255. * tint[2],
                        match sprite.blend {
                            crate::texture::BlendMode::Cutout => 1.,
                            crate::texture::BlendMode::AddQuarter => 0.3,
                            _ => 0.6,
                        } * fade
                            * if n == 0 { 1. } else { 0.45 },
                    ];
                    draw.add_image_quad(texture_id, quad[0], quad[1], quad[2], quad[3])
                        .uv([u0, v0], [u1, v0], [u1, v1], [u0, v1])
                        .col(colour)
                        .build();
                }
                drew = true;
            } else if let Some((_, sprite)) = &v.sprite {
                // Missing or not yet uploaded texture: a placeholder card.
                let quad = corners(v).map(to_screen);
                draw.add_polyline(quad.to_vec(), [0.6, 0.6, 0.65, 0.25 * fade])
                    .filled(true)
                    .build();
                draw.add_polyline(quad.to_vec(), [0.8, 0.8, 0.85, 0.6 * fade])
                    .thickness(1.)
                    .build();
                let c = to_screen(v.world.t);
                draw.add_text(
                    [c[0] - 6., c[1] - 7.],
                    [0.9, 0.9, 0.9, 0.8 * fade],
                    if sprite.texture.is_some() {
                        ICON_IMAGE
                    } else {
                        "?"
                    },
                );
                drew = true;
            }
            if e.world2d.show_colliders {
                for (_, c) in &v.colliders {
                    let colour = if c.trigger {
                        TRIGGER
                    } else if c.one_way {
                        ONE_WAY
                    } else {
                        SOLID
                    };
                    let colour = alpha(colour, if c.enabled { fade } else { 0.35 * fade });
                    let (centre, half) = runtime_box(v, c);
                    match c.shape {
                        Shape2D::Box => {
                            let a = to_screen([centre[0] - half[0], centre[1] - half[1]]);
                            let b = to_screen([centre[0] + half[0], centre[1] + half[1]]);
                            draw.add_rect(a, b, alpha(colour, 0.16))
                                .filled(true)
                                .build();
                            draw.add_rect(a, b, alpha(colour, 0.95))
                                .thickness(1.5)
                                .build();
                            if c.one_way {
                                let mid = (a[0] + b[0]) * 0.5;
                                draw.add_line([mid, a[1]], [mid, a[1] - 14.], colour)
                                    .thickness(2.)
                                    .build();
                                draw.add_triangle(
                                    [mid - 5., a[1] - 10.],
                                    [mid + 5., a[1] - 10.],
                                    [mid, a[1] - 17.],
                                    colour,
                                )
                                .filled(true)
                                .build();
                            }
                        }
                        Shape2D::Circle => {
                            let c2 = to_screen(centre);
                            let r = half[0] * view.zoom;
                            draw.add_circle(c2, r, alpha(colour, 0.16))
                                .filled(true)
                                .num_segments(32)
                                .build();
                            draw.add_circle(c2, r, alpha(colour, 0.95))
                                .thickness(1.5)
                                .num_segments(32)
                                .build();
                        }
                    }
                    drew = true;
                }
            }
            let mut shapes = false;
            for (n, shift) in shifts.iter().enumerate() {
                shapes |= crate::world2d_shapes_editor::draw_view(
                    e,
                    &draw,
                    v,
                    &to_screen,
                    view.zoom,
                    seconds,
                    e.world2d.animate,
                    tint,
                    fade * if n == 0 { 1. } else { 0.45 },
                    n == 0 && Some(v.id) == e.selected_actor,
                    n == 0,
                    *shift,
                );
            }
            drew |= shapes;
            if v.parallax.is_some() && v.sprite.is_none() && v.tiles.is_none() {
                let c = to_screen(v.world.t);
                draw.add_text(
                    [c[0] - 7., c[1] - 7.],
                    [0.55, 0.85, 0.75, fade],
                    ICON_PARALLAX,
                );
                drew = true;
            }
            if v.camera.is_some() {
                let c = to_screen(v.world.t);
                draw.add_text(
                    [c[0] - 7., c[1] - 7.],
                    alpha(CAMERA_COLOUR, fade),
                    ICON_CAMERA,
                );
                drew = true;
            }
            if (!drew || (v.sprite.is_none() && v.tiles.is_none() && !shapes)) && !v.group {
                // Node2D marker: a small cross at the origin.
                let c = to_screen(v.world.t);
                draw.add_line(
                    [c[0] - 6., c[1]],
                    [c[0] + 6., c[1]],
                    [0.85, 0.85, 0.9, 0.7 * fade],
                )
                .thickness(1.5)
                .build();
                draw.add_line(
                    [c[0], c[1] - 6.],
                    [c[0], c[1] + 6.],
                    [0.85, 0.85, 0.9, 0.7 * fade],
                )
                .thickness(1.5)
                .build();
            }
        }
        if painting.is_some() {
            crate::tilemap_editor::overlay(ui, e, &draw, origin, size);
        } else if !e.playing
            && let Some(v) = views.iter().find(|v| Some(v.id) == e.selected_actor)
        {
            crate::world2d_shapes_editor::overlay(e, &draw, v, &to_screen);
        }
        // Hover and selection.
        if let Some(v) = e
            .world2d
            .hovered
            .and_then(|id| views.iter().find(|v| v.id == id))
            && Some(v.id) != e.selected_actor
        {
            draw.add_polyline(corners(v).map(to_screen).to_vec(), HOVER)
                .thickness(1.)
                .build();
            let c = corners(v).map(to_screen)[0];
            draw.add_text([c[0], c[1] - 16.], HOVER, &v.name);
        }
        if let Some(v) = views.iter().find(|v| Some(v.id) == e.selected_actor) {
            let h = handles(v, e, origin, size);
            let mut outline = h.corners.to_vec();
            outline.push(h.corners[0]);
            draw.add_polyline(outline, SELECTION).thickness(1.5).build();
            let pivot = to_screen(v.world.t);
            draw.add_circle(pivot, 4., SELECTION).thickness(1.5).build();
            draw.add_line(
                [pivot[0] - 7., pivot[1]],
                [pivot[0] + 7., pivot[1]],
                SELECTION,
            )
            .build();
            draw.add_line(
                [pivot[0], pivot[1] - 7.],
                [pivot[0], pivot[1] + 7.],
                SELECTION,
            )
            .build();
            if e.world2d.tool == Tool::Select && !e.playing && painting.is_none() {
                draw.add_line(h.top, h.knob, SELECTION).build();
                draw.add_circle(h.knob, HANDLE * 0.7, [0.12, 0.12, 0.14, 1.])
                    .filled(true)
                    .build();
                draw.add_circle(h.knob, HANDLE * 0.7, SELECTION)
                    .thickness(1.5)
                    .build();
                for p in h.corners.iter().chain(h.edges.iter()) {
                    let half = HANDLE * 0.55;
                    draw.add_rect(
                        [p[0] - half, p[1] - half],
                        [p[0] + half, p[1] + half],
                        [0.12, 0.12, 0.14, 1.],
                    )
                    .filled(true)
                    .build();
                    draw.add_rect(
                        [p[0] - half, p[1] - half],
                        [p[0] + half, p[1] + half],
                        SELECTION,
                    )
                    .build();
                }
                if e.world2d.show_colliders {
                    for (_, _, p) in collider_handles(v, e, origin, size) {
                        draw.add_circle(p, HANDLE * 0.55, [0.12, 0.12, 0.14, 1.])
                            .filled(true)
                            .build();
                        draw.add_circle(p, HANDLE * 0.55, SOLID)
                            .thickness(1.5)
                            .build();
                    }
                }
            }
            let label_at = h.corners[0];
            draw.add_text(
                [label_at[0], label_at[1] - 18.],
                SELECTION,
                format!(
                    "{}  ({:.0}, {:.0})",
                    v.name, v.local.position[0], v.local.position[1]
                ),
            );
        }
        if e.world2d.show_rulers {
            rulers(&draw, &view, origin, size, world_mouse);
        }
    });
}
fn rulers(
    draw: &imgui::DrawListMut,
    view: &SceneView2D,
    origin: [f32; 2],
    size: [f32; 2],
    mouse: [f32; 2],
) {
    let corner = [origin[0] + size[0], origin[1] + size[1]];
    draw.add_rect(origin, [corner[0], origin[1] + RULER_SIZE], RULER)
        .filled(true)
        .build();
    draw.add_rect(origin, [origin[0] + RULER_SIZE, corner[1]], RULER)
        .filled(true)
        .build();
    let mut major = 1.;
    while major * view.zoom < 64. {
        major *= 2.;
    }
    let minor = major / 8.;
    let top_left = view.screen_to_world(origin, origin, size);
    let bottom_right = view.screen_to_world(corner, origin, size);
    let tick = [0.55, 0.57, 0.62, 1.];
    let text = [0.70, 0.72, 0.76, 1.];
    let mut x = (top_left[0] / minor).floor() * minor;
    while x <= bottom_right[0] {
        let sx = view.world_to_screen([x, 0.], origin, size)[0];
        let is_major = (x / major).round() * major == x;
        if sx > origin[0] + RULER_SIZE {
            let length = if is_major { RULER_SIZE } else { 5. };
            draw.add_line(
                [sx, origin[1] + RULER_SIZE - length],
                [sx, origin[1] + RULER_SIZE],
                tick,
            )
            .build();
            if is_major {
                draw.add_text([sx + 3., origin[1] + 1.], text, format!("{x:.0}"));
            }
        }
        x += minor;
    }
    let mut y = (top_left[1] / minor).floor() * minor;
    while y <= bottom_right[1] {
        let sy = view.world_to_screen([0., y], origin, size)[1];
        let is_major = (y / major).round() * major == y;
        if sy > origin[1] + RULER_SIZE {
            let length = if is_major { RULER_SIZE } else { 5. };
            draw.add_line(
                [origin[0] + RULER_SIZE - length, sy],
                [origin[0] + RULER_SIZE, sy],
                tick,
            )
            .build();
            if is_major {
                let label = format!("{y:.0}");
                for (i, ch) in label.chars().enumerate() {
                    draw.add_text(
                        [origin[0] + 3., sy + 2. + i as f32 * 10.],
                        text,
                        ch.to_string(),
                    );
                }
            }
        }
        y += minor;
    }
    let m = view.world_to_screen(mouse, origin, size);
    draw.add_line([m[0], origin[1]], [m[0], origin[1] + RULER_SIZE], SELECTION)
        .thickness(1.5)
        .build();
    draw.add_line([origin[0], m[1]], [origin[0] + RULER_SIZE, m[1]], SELECTION)
        .thickness(1.5)
        .build();
    draw.add_rect(
        origin,
        [origin[0] + RULER_SIZE, origin[1] + RULER_SIZE],
        RULER,
    )
    .filled(true)
    .build();
}

// ---- inspector widgets -----------------------------------------------------------------
const X_COLOUR: [f32; 4] = [0.87, 0.40, 0.36, 1.];
const Y_COLOUR: [f32; 4] = [0.55, 0.78, 0.35, 1.];

/// A labelled X/Y pair, the 2D counterpart of the 3D Inspector's vector rows.
fn vec2(ui: &Ui, label: &str, values: &mut [f32; 2], speed: f32, format: &str) -> bool {
    let _id = ui.push_id(label);
    let left = ui.cursor_pos()[0];
    let width = ui.content_region_avail()[0];
    ui.align_text_to_frame_padding();
    ui.text(label);
    let column = if crate::inspector_theme::active() {
        crate::inspector_theme::label_column(width)
    } else {
        96.
    };
    let value_width = if width >= 170. {
        ui.same_line_with_pos(left + column);
        width - column
    } else {
        width
    };
    let field = ((value_width - 7. - 2. * (ui.calc_text_size("X")[0] + 2.)) / 2.).max(1.);
    let mut changed = false;
    for i in 0..2 {
        if i > 0 {
            ui.same_line();
        }
        ui.text_colored([X_COLOUR, Y_COLOUR][i], ["X", "Y"][i]);
        ui.same_line_with_spacing(0., 2.);
        ui.set_next_item_width(field);
        changed |= crate::gui::Drag::new(format!("##{i}"))
            .speed(speed)
            .display_format(format)
            .build(ui, &mut values[i]);
    }
    changed
}
/// Two or more mutually exclusive options as a row of buttons.
pub(crate) fn segmented(ui: &Ui, label: &str, options: &[&str], selected: &mut usize) -> bool {
    let id = crate::gui::field(ui, label);
    let _id = ui.push_id(&id);
    let width = ((ui.content_region_avail()[0] - (options.len() as f32 - 1.) * 2.)
        / options.len() as f32)
        .max(24.);
    let mut changed = false;
    let _spacing = ui.push_style_var(imgui::StyleVar::ItemSpacing([2., 4.]));
    for (i, option) in options.iter().enumerate() {
        if i > 0 {
            ui.same_line();
        }
        let active = *selected == i;
        let _colour = ui.push_style_color(
            imgui::StyleColor::Button,
            if active {
                [0.19, 0.37, 0.52, 1.]
            } else {
                [0.19, 0.19, 0.21, 1.]
            },
        );
        if ui.button_with_size(format!("{option}##{i}"), [width, 0.]) && !active {
            *selected = i;
            changed = true;
        }
    }
    changed
}
/// A 32-bit collision layer or mask as a grid of numbered cells.
pub(crate) fn layer_grid(ui: &Ui, label: &str, value: &mut u32) -> bool {
    let id = crate::gui::field(ui, label);
    let _id = ui.push_id(&id);
    let mut changed = false;
    let available = ui.content_region_avail()[0];
    let cell = ((available - 15. * 2. - 6.) / 16.).clamp(9., 16.);
    let draw = ui.get_window_draw_list();
    let start = ui.cursor_screen_pos();
    for bit in 0..32u32 {
        let row = bit / 16;
        let column = bit % 16;
        let gap = if column >= 8 { 6. } else { 0. };
        let min = [
            start[0] + column as f32 * (cell + 2.) + gap,
            start[1] + row as f32 * (cell + 2.),
        ];
        ui.set_cursor_screen_pos(min);
        let on = *value & (1 << bit) != 0;
        if ui.invisible_button(format!("##bit{bit}"), [cell, cell]) {
            *value ^= 1 << bit;
            changed = true;
        }
        let hovered = ui.is_item_hovered();
        if hovered {
            ui.tooltip_text(format!("{label} bit {} (value {})", bit + 1, 1u64 << bit));
        }
        let colour = if on {
            [0.30, 0.55, 0.85, 1.]
        } else if hovered {
            [0.30, 0.30, 0.33, 1.]
        } else {
            [0.20, 0.20, 0.22, 1.]
        };
        draw.add_rect(min, [min[0] + cell, min[1] + cell], colour)
            .filled(true)
            .rounding(2.)
            .build();
    }
    ui.set_cursor_screen_pos([start[0], start[1] + 2. * (cell + 2.)]);
    ui.dummy([0., 2.]);
    changed
}
fn colour_u8(ui: &Ui, label: &str, colour: &mut [u8; 3]) -> bool {
    let mut value = colour.map(|c| f32::from(c) / 255.);
    let id = crate::gui::field(ui, label);
    if ui.color_edit3(id, &mut value) {
        *colour = value.map(|c| (c * 255.).round().clamp(0., 255.) as u8);
        return true;
    }
    false
}
pub(crate) fn checker(draw: &imgui::DrawListMut, min: [f32; 2], max: [f32; 2], cell: f32) {
    draw.add_rect(min, max, [0.22, 0.22, 0.24, 1.])
        .filled(true)
        .build();
    let mut y = min[1];
    let mut row = 0;
    while y < max[1] {
        let mut x = min[0] + if row % 2 == 0 { 0. } else { cell };
        while x < max[0] {
            draw.add_rect(
                [x, y],
                [(x + cell).min(max[0]), (y + cell).min(max[1])],
                [0.30, 0.30, 0.32, 1.],
            )
            .filled(true)
            .build();
            x += cell * 2.;
        }
        y += cell;
        row += 1;
    }
}
/// A texture rectangle drawn into a box, aspect-correct, on a checkerboard.
fn thumbnail(
    ui: &Ui,
    cache: &mut TextureCache,
    texture: Option<Uuid>,
    rect: Option<[u16; 4]>,
    flip: [bool; 2],
    size: f32,
) {
    let start = ui.cursor_screen_pos();
    ui.dummy([size, size]);
    let draw = ui.get_window_draw_list();
    let max = [start[0] + size, start[1] + size];
    checker(&draw, start, max, 6.);
    let Some((id, texture_size)) = texture.and_then(|t| cache.get(t)) else {
        let label = if texture.is_some() {
            "loading"
        } else {
            "no texture"
        };
        let text = ui.calc_text_size(label);
        draw.add_text(
            [
                start[0] + (size - text[0]) * 0.5,
                start[1] + (size - text[1]) * 0.5,
            ],
            [0.7, 0.7, 0.72, 1.],
            label,
        );
        return;
    };
    let r = rect.unwrap_or([0, 0, texture_size[0], texture_size[1]]);
    if r[2] == 0 || r[3] == 0 {
        return;
    }
    let scale = (size - 4.) / f32::from(r[2].max(r[3]));
    let w = f32::from(r[2]) * scale;
    let h = f32::from(r[3]) * scale;
    let min = [start[0] + (size - w) * 0.5, start[1] + (size - h) * 0.5];
    let (tw, th) = (f32::from(texture_size[0]), f32::from(texture_size[1]));
    let (mut u0, mut u1) = (f32::from(r[0]) / tw, f32::from(r[0] + r[2]) / tw);
    let (mut v0, mut v1) = (f32::from(r[1]) / th, f32::from(r[1] + r[3]) / th);
    if flip[0] {
        std::mem::swap(&mut u0, &mut u1);
    }
    if flip[1] {
        std::mem::swap(&mut v0, &mut v1);
    }
    draw.add_image(id, min, [min[0] + w, min[1] + h])
        .uv_min([u0, v0])
        .uv_max([u1, v1])
        .build();
}
// ---- inspector -------------------------------------------------------------------------
pub fn node_icon(actor: &crate::scene::Actor) -> &'static str {
    let has = |id: &str| {
        actor
            .components
            .iter()
            .any(|c| c.class.class_id.as_deref() == Some(id))
    };
    if has(world2d::CHARACTER_BODY2D) {
        ICON_RUN
    } else if has(world2d::TILEMAP_LAYER) {
        crate::tilemap_editor::ICON_TILEMAP
    } else if has(world2d::PARALLAX2D) {
        ICON_PARALLAX
    } else if has(world2d::CANVAS_LAYER2D) {
        ICON_CANVAS
    } else if has(world2d::CANVAS_MODULATE2D) {
        ICON_MODULATE
    } else if has(world2d::PARTICLES2D) {
        crate::world2d_shapes_editor::ICON_PARTICLES
    } else if has(world2d::PATH_FOLLOW2D) {
        crate::world2d_shapes_editor::ICON_FOLLOW
    } else if has(world2d::PATH2D) {
        crate::world2d_shapes_editor::ICON_PATH
    } else if has(world2d::LINE2D) {
        crate::world2d_shapes_editor::ICON_LINE
    } else if has(world2d::POLYGON2D) {
        crate::world2d_shapes_editor::ICON_POLYGON
    } else if has(world2d::AUDIO_PLAYER2D) {
        crate::world2d_shapes_editor::ICON_AUDIO
    } else if has(world2d::NOTIFIER2D) {
        crate::world2d_shapes_editor::ICON_NOTIFIER
    } else if has(world2d::REMOTE_TRANSFORM2D) {
        crate::world2d_shapes_editor::ICON_REMOTE
    } else if has(world2d::CAMERA2D) {
        ICON_CAMERA
    } else if has(world2d::SPRITE2D) {
        ICON_IMAGE
    } else if has(world2d::COLLIDER2D) {
        ICON_BODY
    } else {
        ICON_NODE
    }
}
/// The Inspector for a World2D actor.
pub fn inspector(ui: &Ui, e: &mut Editor) {
    let Some(id) = e.selected_actor else { return };
    if e.scene.actor_index(id).is_none() {
        e.select_actor(None);
        return;
    }
    ui.disabled(e.playing, || inspector_body(ui, e, id));
    if e.playing {
        ui.spacing();
        ui.text_disabled("Stop Play to edit this node.");
    }
    if !ui.is_any_item_active() {
        e.end_coalesced();
    }
}
fn inspector_body(ui: &Ui, e: &mut Editor, id: Uuid) {
    let Some(index) = e.scene.actor_index(id) else {
        return;
    };
    let mut actor = e.scene.actors[index].clone();
    let glyph = node_icon(&actor);
    if crate::gui::actor_header(ui, &mut actor, glyph) {
        e.scene.actors[index] = actor;
        e.changed();
    }
    ui.separator();
    let class = e.scene.actors[index].class.clone();
    let cpp = class.name.clone();
    if crate::gui::heading(
        ui,
        &format!(
            "{ICON_NODE} {}",
            crate::actor_document::short_class_name(&cpp)
        ),
    ) {
        crate::gui::edit_class_button(ui, e, &class);
        let _ = crate::gui::field(ui, "Class");
        ui.text(&cpp);
        let _ = crate::gui::field(ui, "Domain");
        ui.text("2D world");
    }
    transform_band(ui, e, id);
    let components: Vec<_> = e.scene.actors[index]
        .components
        .iter()
        .filter(|c| !c.root)
        .cloned()
        .collect();
    for component in components {
        let _id = ui.push_id(component.id.to_string());
        match component.class.class_id.as_deref() {
            Some(world2d::SPRITE2D) => sprite_band(ui, e, id, &component),
            Some(world2d::COLLIDER2D) => collider_band(ui, e, id, &component),
            Some(world2d::CHARACTER_BODY2D) => body_band(ui, e, id, &component),
            Some(world2d::CAMERA2D) => camera_band(ui, e, id, &component),
            Some(world2d::TILEMAP_LAYER) => tilemap_band(ui, e, id, &component),
            Some(world2d::PARALLAX2D) => parallax_band(ui, e, id, &component),
            Some(world2d::CANVAS_LAYER2D) => canvas_layer_band(ui, e, id, &component),
            Some(world2d::CANVAS_MODULATE2D) => canvas_modulate_band(ui, e, id, &component),
            _ if crate::world2d_shapes_editor::band(ui, e, id, &component) => {}
            _ => generic_band(ui, e, id, &component),
        }
    }
    ui.dummy([0., 7.]);
    let width = ui.content_region_avail()[0];
    ui.set_cursor_pos([
        ui.cursor_pos()[0] + (width - 210.).max(0.) * 0.5,
        ui.cursor_pos()[1],
    ]);
    if ui.button_with_size(
        format!("{ICON_PLUS}  Add Component"),
        [210_f32.min(width), 23.],
    ) {
        ui.open_popup("add-2d-component");
    }
    add_component_popup(ui, e, id);
}
fn band(ui: &Ui, title: &str, remove: &mut bool, reset: &mut bool, removable: bool) -> bool {
    crate::gui::section(ui, title, || {
        if crate::gui::script_menu_item(ui, "Reset to Defaults") {
            *reset = true;
        }
        if removable && crate::gui::script_menu_item(ui, "Remove Component") {
            *remove = true;
        }
    })
}
fn finish_band(
    e: &mut Editor,
    actor: Uuid,
    component: &crate::actor_document::ComponentInstance,
    remove: bool,
    reset: bool,
) {
    if remove {
        e.remove_actor_component(actor, component.id);
    } else if reset {
        let class = component.class.class_id.clone().unwrap_or_default();
        if let Some(index) = e.scene.actor_index(actor)
            && let Some(c) = e.scene.actors[index]
                .components
                .iter_mut()
                .find(|c| c.id == component.id)
        {
            match class.as_str() {
                world2d::SPRITE2D => {
                    let texture = world2d::read::<Sprite2D>(c).texture;
                    world2d::write(
                        c,
                        &Sprite2D {
                            texture,
                            ..Default::default()
                        },
                    )
                }
                world2d::COLLIDER2D => world2d::write(c, &Collider2D::default()),
                world2d::CHARACTER_BODY2D => world2d::write(c, &CharacterBody2D::default()),
                world2d::CAMERA2D => world2d::write(c, &Camera2D::default()),
                world2d::TILEMAP_LAYER => {
                    // Reset keeps the painted map and its TileSet.
                    let old = world2d::read::<world2d::TileMapLayer>(c);
                    world2d::write(
                        c,
                        &world2d::TileMapLayer {
                            tile_set: old.tile_set,
                            cells: old.cells,
                            ..Default::default()
                        },
                    )
                }
                world2d::PARALLAX2D => world2d::write(c, &Parallax2D::default()),
                world2d::CANVAS_LAYER2D => world2d::write(c, &CanvasLayer2D::default()),
                world2d::CANVAS_MODULATE2D => world2d::write(c, &CanvasModulate2D::default()),
                _ => {}
            }
            e.scene.actors[index].refresh_components();
            e.changed();
        }
    }
}
fn commit<T: serde::Serialize>(e: &mut Editor, actor: Uuid, component: Uuid, value: &T) {
    if let Some(index) = e.scene.actor_index(actor)
        && let Some(c) = e.scene.actors[index]
            .components
            .iter_mut()
            .find(|c| c.id == component)
    {
        world2d::write(c, value);
        e.scene.actors[index].refresh_components();
        e.changed_coalesced("world2d-inspector");
    }
}
fn transform_band(ui: &Ui, e: &mut Editor, id: Uuid) {
    let Some(index) = e.scene.actor_index(id) else {
        return;
    };
    let mut reset = false;
    let open = crate::gui::section(ui, &format!("{ICON_MOVE} Transform 2D"), || {
        if crate::gui::script_menu_item(ui, "Reset Transform") {
            reset = true;
        }
    });
    if reset {
        edit_transform(e, id, |t| {
            *t = Transform2D {
                draw_order: t.draw_order,
                ..Default::default()
            }
        });
        e.changed();
    }
    if !open {
        return;
    }
    let mut t = root_transform(&e.scene.actors[index]);
    let before = t;
    vec2(ui, "Position", &mut t.position, 0.5, "%.1f");
    crate::gui::Drag::new(crate::gui::field(ui, "Rotation (deg)"))
        .speed(0.5)
        .display_format("%.1f")
        .build(ui, &mut t.rotation);
    let ratio = e.world2d.keep_ratio;
    let start_scale = t.scale;
    vec2(ui, "Scale", &mut t.scale, 0.01, "%.3f");
    if ratio && t.scale != start_scale {
        let k = if t.scale[0] != start_scale[0] { 0 } else { 1 };
        let factor = if start_scale[k].abs() > 1e-6 {
            t.scale[k] / start_scale[k]
        } else {
            1.
        };
        t.scale = [start_scale[0] * factor, start_scale[1] * factor];
    }
    crate::gui::toggle(ui, "Keep scale ratio", &mut e.world2d.keep_ratio);
    crate::gui::Drag::new(crate::gui::field(ui, "Z Index"))
        .range(-4096, 4096)
        .speed(0.2)
        .build(ui, &mut t.draw_order);
    if t != before {
        edit_transform(e, id, |x| *x = t);
        e.changed_coalesced("world2d-inspector");
    }
}
fn sprite_band(
    ui: &Ui,
    e: &mut Editor,
    actor: Uuid,
    component: &crate::actor_document::ComponentInstance,
) {
    let (mut remove, mut reset) = (false, false);
    let open = band(
        ui,
        &format!("{ICON_IMAGE} Sprite2D"),
        &mut remove,
        &mut reset,
        true,
    );
    if open {
        let mut s: Sprite2D = world2d::read(component);
        let before = s.clone();
        let size = s
            .texture
            .and_then(|t| e.world2d.textures.get(t))
            .map(|t| t.1);
        let rect =
            size.and_then(|size| preview_rect(&s, size, ui.time() as f32, e.world2d.animate));
        // Preview and texture picker side by side.
        let top = ui.cursor_screen_pos();
        thumbnail(
            ui,
            &mut e.world2d.textures,
            s.texture,
            rect,
            [s.flip_h, s.flip_v],
            84.,
        );
        let after_thumb = ui.cursor_screen_pos();
        ui.set_cursor_screen_pos([top[0] + 92., top[1]]);
        ui.group(|| {
            let width = ui.content_region_avail()[0];
            ui.set_next_item_width(width);
            let name = s
                .texture
                .and_then(|id| e.assets.index.resolve(id).ok())
                .map(|r| r.meta.source.clone())
                .unwrap_or_else(|| "Choose a texture".into());
            if let Some(_combo) = ui.begin_combo("##sprite-texture", name) {
                if ui.selectable("None") {
                    s.texture = None;
                }
                let mut textures: Vec<_> = e
                    .assets
                    .index
                    .usable()
                    .filter(|r| r.meta.kind == crate::assets::Kind::Texture)
                    .collect();
                textures.sort_by(|a, b| a.meta.source.cmp(&b.meta.source));
                for record in textures {
                    if ui
                        .selectable_config(&record.meta.source)
                        .selected(s.texture == Some(record.meta.id))
                        .build()
                    {
                        s.texture = Some(record.meta.id);
                    }
                }
            }
            match (s.texture, size) {
                (Some(_), Some(size)) => {
                    ui.text_disabled(format!("{} x {} texels", size[0], size[1]))
                }
                (Some(_), None) => ui.text_disabled("Loading preview..."),
                (None, _) => ui.text_disabled("Drop a texture asset here or pick one above."),
            }
            if let Some(r) = rect {
                ui.text_disabled(format!("Drawing {} x {} px", r[2], r[3]));
            }
            if ui.button(format!("{ICON_CROP}  Region...")) && s.texture.is_some() {
                e.world2d.region_editor = Some(RegionEditor {
                    actor,
                    component: component.id,
                    region: s.region,
                    zoom: 0.,
                    snap: 8,
                    dragging: None,
                });
            }
            if ui.is_item_hovered() {
                ui.tooltip_text("Pick the part of the texture this sprite draws");
            }
        });
        texture_drop(ui, e, &mut s.texture);
        ui.set_cursor_screen_pos([
            after_thumb[0],
            after_thumb[1].max(ui.cursor_screen_pos()[1]) + 4.,
        ]);
        let mut region = s.region.map(i32::from);
        if crate::gui::Drag::new(crate::gui::field(ui, "Region x/y/w/h"))
            .range(0, 256)
            .build_array(ui, &mut region)
        {
            s.region = region.map(|v| v.clamp(0, 256) as u16);
        }
        let mut grid = [i32::from(s.hframes), i32::from(s.vframes)];
        if crate::gui::Drag::new(crate::gui::field(ui, "Sheet columns/rows"))
            .range(1, 64)
            .speed(0.05)
            .build_array(ui, &mut grid)
        {
            s.hframes = grid[0].clamp(1, 64) as u16;
            s.vframes = grid[1].clamp(1, 64) as u16;
        }
        if s.hframes > 1 || s.vframes > 1 {
            let mut frame = i32::from(s.frame);
            let count = i32::from(s.hframes) * i32::from(s.vframes);
            if crate::gui::Drag::new(crate::gui::field(ui, "Frame"))
                .range(0, count - 1)
                .speed(0.05)
                .build(ui, &mut frame)
            {
                s.frame = frame.clamp(0, count - 1) as u16;
            }
        }
        vec2(ui, "Offset", &mut s.offset, 0.5, "%.0f");
        crate::gui::toggle(ui, "Centered", &mut s.centered);
        let flip_id = crate::gui::field(ui, "Flip");
        let _flip = ui.push_id(&flip_id);
        if crate::gui::icon(ui, ICON_FLIP_H, "flip-h", "Flip horizontally", s.flip_h) {
            s.flip_h = !s.flip_h;
        }
        ui.same_line();
        if crate::gui::icon(ui, ICON_FLIP_V, "flip-v", "Flip vertically", s.flip_v) {
            s.flip_v = !s.flip_v;
        }
        drop(_flip);
        colour_u8(ui, "Modulate", &mut s.modulate);
        let modes = [
            crate::texture::BlendMode::Cutout,
            crate::texture::BlendMode::Average,
            crate::texture::BlendMode::Add,
            crate::texture::BlendMode::Subtract,
            crate::texture::BlendMode::AddQuarter,
        ];
        let mut mode = modes.iter().position(|m| *m == s.blend).unwrap_or(0);
        if ui.combo_simple_string(
            crate::gui::field(ui, "Blend"),
            &mut mode,
            &[
                "Opaque (cutout)",
                "Average 50%",
                "Add",
                "Subtract",
                "Add 25%",
            ],
        ) {
            s.blend = modes[mode];
        }
        let mut z = i32::from(s.z_index);
        if crate::gui::Drag::new(crate::gui::field(ui, "Z Index"))
            .range(-4096, 4096)
            .speed(0.2)
            .build(ui, &mut z)
        {
            s.z_index = z.clamp(-4096, 4096) as i16;
        }
        crate::gui::toggle(ui, "Y-Sort", &mut s.y_sort);
        if ui.is_item_hovered() {
            ui.tooltip_text("Within its Z index, draw lower sprites (higher Y) in front");
        }
        crate::gui::toggle(ui, "Visible", &mut s.visible);
        animations(ui, e, actor, component.id, &mut s);
        if s != before {
            commit(e, actor, component.id, &s);
        }
    }
    finish_band(e, actor, component, remove, reset);
}
fn animations(ui: &Ui, e: &mut Editor, actor: Uuid, component: Uuid, s: &mut Sprite2D) {
    ui.spacing();
    if !crate::gui::heading(
        ui,
        &format!(
            "{ICON_FILM} Animations ({})##animations",
            s.animations.len()
        ),
    ) {
        return;
    }
    let selected = e.world2d.animation_selection.entry(component).or_insert(0);
    if *selected >= s.animations.len() {
        *selected = s.animations.len().saturating_sub(1);
    }
    let mut selection = *selected;
    if ui.small_button(format!("{ICON_PLUS} New")) && s.animations.len() < 64 {
        let mut name = "default".to_owned();
        let mut n = 2;
        while s.animations.iter().any(|a| a.name == name) {
            name = format!("anim{n}");
            n += 1;
        }
        s.animations.push(Animation2D {
            name,
            ..Default::default()
        });
        selection = s.animations.len() - 1;
    }
    ui.same_line();
    let has = selection < s.animations.len();
    ui.disabled(!has, || {
        if ui.small_button(format!("{ICON_CLONE} Duplicate")) && has && s.animations.len() < 64 {
            let mut copy = s.animations[selection].clone();
            copy.name = format!("{}_copy", copy.name);
            s.animations.insert(selection + 1, copy);
            selection += 1;
        }
        ui.same_line();
        if ui.small_button(format!("{ICON_TRASH} Delete")) && has {
            s.animations.remove(selection);
            if usize::from(s.animation) >= s.animations.len() {
                s.animation = s.animations.len().saturating_sub(1) as u16;
            }
            selection = selection.min(s.animations.len().saturating_sub(1));
        }
    });
    if s.animations.is_empty() {
        crate::gui::muted(
            ui,
            "No animations. Click New, then add frames from the sprite sheet.",
        );
        e.world2d.animation_selection.insert(component, selection);
        return;
    }
    let list_height =
        (s.animations.len() as f32 * ui.text_line_height_with_spacing() + 8.).min(120.);
    ui.child_window("##animation-list")
        .size([-1., list_height])
        .border(true)
        .build(|| {
            for (i, animation) in s.animations.iter().enumerate() {
                let star = if usize::from(s.animation) == i {
                    "\u{f005} "
                } else {
                    "    "
                };
                let label = format!(
                    "{star}{}  ·  {} frames · {:.0} fps{}##anim{i}",
                    animation.name,
                    animation.frames.len(),
                    animation.fps,
                    if animation.looping { " · loop" } else { "" }
                );
                if ui.selectable_config(label).selected(selection == i).build() {
                    selection = i;
                }
            }
        });
    e.world2d.animation_selection.insert(component, selection);
    let Some(animation) = s.animations.get_mut(selection) else {
        return;
    };
    ui.input_text(crate::gui::field(ui, "Name"), &mut animation.name)
        .build();
    crate::gui::Drag::new(crate::gui::field(ui, "Speed (FPS)"))
        .range(0.5, 60.)
        .speed(0.1)
        .display_format("%.1f")
        .build(ui, &mut animation.fps);
    crate::gui::toggle(ui, "Loop", &mut animation.looping);
    let is_default = usize::from(s.animation) == selection;
    let id = crate::gui::field(ui, "On load");
    let _on_load = ui.push_id(&id);
    let mut default = is_default;
    if ui.checkbox("Default animation", &mut default) && default {
        s.animation = selection as u16;
    }
    ui.same_line();
    ui.checkbox("Autoplay", &mut s.autoplay);
    drop(_on_load);
    // Frames strip.
    let animation = &mut s.animations[selection];
    let frame_key = e.world2d.frame_selection.entry(component).or_insert(0);
    if *frame_key >= animation.frames.len() {
        *frame_key = animation.frames.len().saturating_sub(1);
    }
    let mut current = *frame_key;
    ui.text_disabled(format!("Frames ({})", animation.frames.len()));
    let thumb = 44.;
    let texture = s.texture;
    let per_row = ((ui.content_region_avail()[0] + 4.) / (thumb + 4.))
        .floor()
        .max(1.) as usize;
    for (i, frame) in animation.frames.iter().enumerate() {
        if i % per_row != 0 {
            ui.same_line_with_spacing(0., 4.);
        }
        let start = ui.cursor_screen_pos();
        thumbnail(
            ui,
            &mut e.world2d.textures,
            texture,
            Some(frame.region),
            [false, false],
            thumb,
        );
        if ui.is_item_clicked() {
            current = i;
        }
        let draw = ui.get_window_draw_list();
        let end = [start[0] + thumb, start[1] + thumb];
        draw.add_rect(
            start,
            end,
            if i == current {
                SELECTION
            } else {
                [1., 1., 1., 0.12]
            },
        )
        .thickness(if i == current { 2. } else { 1. })
        .build();
        draw.add_text(
            [start[0] + 3., start[1] + 1.],
            [1., 1., 1., 0.85],
            format!("{i}"),
        );
    }
    if animation.frames.is_empty() {
        crate::gui::muted(ui, "Empty animation.");
    }
    if ui.button(format!("{ICON_SLICE}  Add Frames from Sheet...")) && texture.is_some() {
        e.world2d.slicer = Some(SheetSlicer {
            actor,
            component,
            animation: selection,
            columns: s
                .hframes
                .max(1)
                .max(if animation.frames.is_empty() { 4 } else { 1 }),
            rows: s.vframes.max(1),
            offset: [0, 0],
            separation: [0, 0],
            selected: Vec::new(),
            zoom: 0.,
        });
    }
    if ui.is_item_hovered() {
        ui.tooltip_text("Cut the texture into a grid and pick frames in order");
    }
    if current < animation.frames.len() {
        ui.same_line();
        if crate::gui::icon(ui, ICON_LEFT, "frame-left", "Move frame earlier", false) && current > 0
        {
            animation.frames.swap(current, current - 1);
            current -= 1;
        }
        ui.same_line();
        if crate::gui::icon(ui, ICON_RIGHT, "frame-right", "Move frame later", false)
            && current + 1 < animation.frames.len()
        {
            animation.frames.swap(current, current + 1);
            current += 1;
        }
        ui.same_line();
        if crate::gui::icon(ui, ICON_TRASH, "frame-delete", "Remove frame", false) {
            animation.frames.remove(current);
            current = current.saturating_sub(1);
        }
        if let Some(frame) = animation.frames.get_mut(current) {
            crate::gui::Drag::new(crate::gui::field(ui, "Frame duration"))
                .range(0.1, 16.)
                .speed(0.02)
                .display_format("%.2fx")
                .build(ui, &mut frame.duration);
            let mut event = i32::from(frame.event);
            if crate::gui::Drag::new(crate::gui::field(ui, "Frame event (0 = none)"))
                .range(0, 65535)
                .build(ui, &mut event)
            {
                frame.event = event.clamp(0, 65535) as u16;
            }
        }
    }
    e.world2d.frame_selection.insert(component, current);
}
fn collider_band(
    ui: &Ui,
    e: &mut Editor,
    actor: Uuid,
    component: &crate::actor_document::ComponentInstance,
) {
    let (mut remove, mut reset) = (false, false);
    let mut c: Collider2D = world2d::read(component);
    let title = format!(
        "{} Collider2D",
        if c.trigger { ICON_AREA } else { ICON_BODY }
    );
    let open = band(
        ui,
        &format!("{title}##{}", component.id),
        &mut remove,
        &mut reset,
        true,
    );
    if open {
        let before = c.clone();
        let mut shape = usize::from(c.shape == Shape2D::Circle);
        if segmented(ui, "Shape", &["Rectangle", "Circle"], &mut shape) {
            c.shape = if shape == 1 {
                Shape2D::Circle
            } else {
                Shape2D::Box
            };
        }
        match c.shape {
            Shape2D::Box => {
                vec2(ui, "Size", &mut c.size, 0.5, "%.0f");
                c.size = c.size.map(|v| v.abs().max(1.));
            }
            Shape2D::Circle => {
                crate::gui::Drag::new(crate::gui::field(ui, "Radius"))
                    .range(1., 4096.)
                    .speed(0.25)
                    .display_format("%.1f")
                    .build(ui, &mut c.radius);
            }
        }
        vec2(ui, "Offset", &mut c.offset, 0.5, "%.0f");
        let mut mode = if c.trigger {
            1
        } else if c.one_way {
            2
        } else {
            0
        };
        if segmented(ui, "Mode", &["Solid", "Trigger", "One-way"], &mut mode) {
            c.trigger = mode == 1;
            c.one_way = mode == 2;
        }
        if c.one_way {
            crate::gui::Drag::new(crate::gui::field(ui, "One-way margin"))
                .range(0., 64.)
                .speed(0.1)
                .display_format("%.1f px")
                .build(ui, &mut c.one_way_margin);
        }
        layer_grid(ui, "Layer", &mut c.layer);
        layer_grid(ui, "Mask", &mut c.mask);
        crate::gui::toggle(ui, "Enabled", &mut c.enabled);
        let sprite = e.scene.actor_index(actor).and_then(|i| {
            world2d::component(&e.scene.actors[i], world2d::SPRITE2D).map(world2d::read::<Sprite2D>)
        });
        if let Some(sprite) = sprite
            && let Some(texture) = sprite.texture
            && let Some(size) = e.world2d.textures.get(texture).map(|t| t.1)
            && let Some(rect) = sprite.source_rect(size)
        {
            if ui.button(format!("{ICON_WAND}  Fit to Sprite")) {
                let (lo, hi) = sprite_local_rect(&sprite, [f32::from(rect[2]), f32::from(rect[3])]);
                c.size = [hi[0] - lo[0], hi[1] - lo[1]];
                c.radius = c.size[0].min(c.size[1]) * 0.5;
                c.offset = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5];
            }
            if ui.is_item_hovered() {
                ui.tooltip_text("Match the shape to the sprite's drawn rectangle");
            }
            ui.same_line();
            if ui.button(format!("{ICON_WAND}  Fit to Visible Pixels"))
                && let Some(data) = e.scene.textures.get(&texture)
                && let Some((min, max)) = opaque_bounds(data, rect)
            {
                // The tight box of the frame's opaque texels, placed like the sprite draws it
                // (flips mirror the box inside the frame).
                let (lo, _) = sprite_local_rect(&sprite, [f32::from(rect[2]), f32::from(rect[3])]);
                let (w, h) = (f32::from(rect[2]), f32::from(rect[3]));
                let (mut x0, mut x1) = (min[0], max[0]);
                let (mut y0, mut y1) = (min[1], max[1]);
                if sprite.flip_h {
                    (x0, x1) = (w - x1, w - x0);
                }
                if sprite.flip_v {
                    (y0, y1) = (h - y1, h - y0);
                }
                c.shape = Shape2D::Box;
                c.size = [x1 - x0, y1 - y0];
                c.offset = [lo[0] + (x0 + x1) * 0.5, lo[1] + (y0 + y1) * 0.5];
            }
            if ui.is_item_hovered() {
                ui.tooltip_text(
                    "A box around the frame's visible pixels, ignoring its transparent margins",
                );
            }
        }
        crate::gui::muted(
            ui,
            if c.trigger {
                "Reports Enter / Stay / Exit through the actor's Overlap 2D event."
            } else {
                "Rotation does not turn the shape: boxes stay axis-aligned on PSX."
            },
        );
        if c != before {
            commit(e, actor, component.id, &c);
        }
    }
    finish_band(e, actor, component, remove, reset);
}
/// Bounds (min, max) of the texels with alpha inside `rect` (x, y, w, h), relative to
/// the rectangle, or None when the rectangle is fully transparent.
pub(crate) fn opaque_bounds(
    data: &crate::texture::Data,
    rect: [u16; 4],
) -> Option<([f32; 2], [f32; 2])> {
    let mut lo = [u32::MAX; 2];
    let mut hi = [0u32; 2];
    for y in 0..u32::from(rect[3]) {
        for x in 0..u32::from(rect[2]) {
            let (tx, ty) = (u32::from(rect[0]) + x, u32::from(rect[1]) + y);
            let at = ((ty * u32::from(data.width) + tx) * 4 + 3) as usize;
            if data.rgba.get(at).is_some_and(|a| *a > 0) {
                lo = [lo[0].min(x), lo[1].min(y)];
                hi = [hi[0].max(x + 1), hi[1].max(y + 1)];
            }
        }
    }
    (lo[0] <= hi[0]).then(|| (lo.map(|v| v as f32), hi.map(|v| v as f32)))
}
fn body_band(
    ui: &Ui,
    e: &mut Editor,
    actor: Uuid,
    component: &crate::actor_document::ComponentInstance,
) {
    let (mut remove, mut reset) = (false, false);
    let open = band(
        ui,
        &format!("{ICON_RUN} CharacterBody2D"),
        &mut remove,
        &mut reset,
        true,
    );
    if open {
        let mut b: CharacterBody2D = world2d::read(component);
        let before = b.clone();
        let mut mode = usize::from(b.floating);
        if segmented(ui, "Motion mode", &["Grounded", "Floating"], &mut mode) {
            b.floating = mode == 1;
        }
        if !b.floating {
            vec2(ui, "Up direction", &mut b.up_direction, 0.01, "%.2f");
            crate::gui::Drag::new(crate::gui::field(ui, "Floor max angle"))
                .range(0., 89.)
                .speed(0.25)
                .display_format("%.0f deg")
                .build(ui, &mut b.floor_max_angle);
            crate::gui::Drag::new(crate::gui::field(ui, "Floor snap length"))
                .range(0., 64.)
                .speed(0.1)
                .display_format("%.1f px")
                .build(ui, &mut b.floor_snap_length);
        }
        layer_grid(ui, "Collision mask", &mut b.collision_mask);
        let has_collider = e
            .scene
            .actor_index(actor)
            .is_some_and(|i| world2d::component(&e.scene.actors[i], world2d::COLLIDER2D).is_some());
        if !has_collider {
            ui.text_colored(
                [1., 0.72, 0.3, 1.],
                format!("{ICON_INFO}  Add a solid Collider2D: the body moves through it."),
            );
        }
        crate::gui::muted(
            ui,
            "Set the velocity (pixels/second) and call move_and_slide() from tick.",
        );
        if b != before {
            commit(e, actor, component.id, &b);
        }
    }
    finish_band(e, actor, component, remove, reset);
}
fn camera_band(
    ui: &Ui,
    e: &mut Editor,
    actor: Uuid,
    component: &crate::actor_document::ComponentInstance,
) {
    let (mut remove, mut reset) = (false, false);
    let open = band(
        ui,
        &format!("{ICON_CAMERA} Camera2D"),
        &mut remove,
        &mut reset,
        true,
    );
    if open {
        let mut c: Camera2D = world2d::read(component);
        let before = c.clone();
        crate::gui::toggle(ui, "Current", &mut c.current);
        if c.current && !before.current {
            // Only one current camera: clear the flag elsewhere in the map.
            let others: Vec<(Uuid, Uuid)> = e
                .scene
                .actors
                .iter()
                .flat_map(|a| {
                    a.components
                        .iter()
                        .filter(|x| {
                            x.class.class_id.as_deref() == Some(world2d::CAMERA2D)
                                && x.id != component.id
                        })
                        .map(move |x| (a.id, x.id))
                })
                .collect();
            for (a, x) in others {
                edit_component::<Camera2D>(e, a, x, |camera| camera.current = false);
            }
        }
        crate::gui::Drag::new(crate::gui::field(ui, "Zoom"))
            .range(0.0625, 64.)
            .speed(0.01)
            .display_format("%.2fx")
            .build(ui, &mut c.zoom);
        vec2(ui, "Offset", &mut c.offset, 0.5, "%.0f");
        colour_u8(ui, "Background", &mut c.background);
        crate::gui::toggle(ui, "Rotate with node", &mut c.rotating);
        crate::gui::toggle(ui, "Smoothing", &mut c.smoothing);
        if c.smoothing {
            crate::gui::Drag::new(crate::gui::field(ui, "Smoothing speed"))
                .range(0.1, 60.)
                .speed(0.05)
                .display_format("%.1f")
                .build(ui, &mut c.smoothing_speed);
        }
        crate::gui::toggle(ui, "Limits", &mut c.limits_enabled);
        if c.limits_enabled {
            let mut left_top = [c.limits[0], c.limits[1]];
            let mut right_bottom = [c.limits[2], c.limits[3]];
            vec2(ui, "Left / Top", &mut left_top, 1., "%.0f");
            vec2(ui, "Right / Bottom", &mut right_bottom, 1., "%.0f");
            c.limits = [left_top[0], left_top[1], right_bottom[0], right_bottom[1]];
            if ui.button(format!("{ICON_WAND}  Fit Limits to Scene")) {
                let views = views(e);
                let mut lo = [f32::MAX; 2];
                let mut hi = [f32::MIN; 2];
                for v in views
                    .iter()
                    .filter(|v| v.sprite.is_some() || !v.colliders.is_empty())
                {
                    for p in corners(v) {
                        for k in 0..2 {
                            lo[k] = lo[k].min(p[k]);
                            hi[k] = hi[k].max(p[k]);
                        }
                    }
                }
                if lo[0] <= hi[0] {
                    c.limits = [lo[0].floor(), lo[1].floor(), hi[0].ceil(), hi[1].ceil()];
                }
            }
        }
        crate::gui::toggle(ui, "Drag margins", &mut c.drag_enabled);
        if c.drag_enabled {
            let mut left_top = [c.drag_margins[0], c.drag_margins[1]];
            let mut right_bottom = [c.drag_margins[2], c.drag_margins[3]];
            vec2(ui, "Margin L / T", &mut left_top, 0.01, "%.2f");
            vec2(ui, "Margin R / B", &mut right_bottom, 0.01, "%.2f");
            c.drag_margins = [left_top[0], left_top[1], right_bottom[0], right_bottom[1]]
                .map(|v| v.clamp(0., 1.));
        }
        if c != before {
            commit(e, actor, component.id, &c);
        }
    }
    finish_band(e, actor, component, remove, reset);
}
fn tilemap_band(
    ui: &Ui,
    e: &mut Editor,
    actor: Uuid,
    component: &crate::actor_document::ComponentInstance,
) {
    let (mut remove, mut reset) = (false, false);
    let open = band(
        ui,
        &format!("{} TileMapLayer", crate::tilemap_editor::ICON_TILEMAP),
        &mut remove,
        &mut reset,
        true,
    );
    if open && crate::tilemap_editor::band(ui, e, actor, component) {
        e.changed_coalesced("world2d-inspector");
    }
    finish_band(e, actor, component, remove, reset);
}
fn parallax_band(
    ui: &Ui,
    e: &mut Editor,
    actor: Uuid,
    component: &crate::actor_document::ComponentInstance,
) {
    let (mut remove, mut reset) = (false, false);
    let open = band(
        ui,
        &format!("{ICON_PARALLAX} Parallax2D"),
        &mut remove,
        &mut reset,
        true,
    );
    if open {
        let mut p: Parallax2D = world2d::read(component);
        let before = p.clone();
        vec2(ui, "Scroll scale", &mut p.scroll_scale, 0.01, "%.2f");
        if ui.is_item_hovered() {
            ui.tooltip_text("1 moves with the world, 0.5 is a distant layer, 0 stays on screen");
        }
        let _ = crate::gui::field(ui, "Presets");
        for (label, value) in [("Far", 0.25), ("Mid", 0.5), ("Near", 0.75), ("Sky", 0.)] {
            if ui.small_button(label) {
                p.scroll_scale = [value, if value == 0. { 0. } else { 1. }];
            }
            ui.same_line();
        }
        ui.new_line();
        vec2(ui, "Repeat size", &mut p.repeat_size, 1., "%.0f");
        p.repeat_size = p.repeat_size.map(|v| v.max(0.));
        // The width of the actor's own sprite is the usual repeat.
        let sprite_size = e.scene.actor_index(actor).and_then(|i| {
            let s = world2d::component(&e.scene.actors[i], world2d::SPRITE2D)
                .map(world2d::read::<Sprite2D>)?;
            let size = e.world2d.textures.get(s.texture?)?.1;
            s.source_rect(size)
                .map(|r| [f32::from(r[2]), f32::from(r[3])])
        });
        if let Some(size) = sprite_size {
            if ui.button(format!("{ICON_WAND}  Repeat Sprite Horizontally")) {
                p.repeat_size = [size[0], 0.];
            }
        } else {
            ui.text_disabled("Attach sprites or tile layers to this node; with a repeat size they tile across the screen.");
        }
        vec2(ui, "Autoscroll (px/s)", &mut p.autoscroll, 0.5, "%.1f");
        crate::gui::muted(
            ui,
            "The layer sits where you placed it when the game camera starts here; it scrolls from there.",
        );
        if p != before {
            commit(e, actor, component.id, &p);
        }
    }
    finish_band(e, actor, component, remove, reset);
}
fn canvas_layer_band(
    ui: &Ui,
    e: &mut Editor,
    actor: Uuid,
    component: &crate::actor_document::ComponentInstance,
) {
    let (mut remove, mut reset) = (false, false);
    let open = band(
        ui,
        &format!("{ICON_CANVAS} CanvasLayer2D"),
        &mut remove,
        &mut reset,
        true,
    );
    if open {
        let mut c: CanvasLayer2D = world2d::read(component);
        let before = c.clone();
        let mut layer = i32::from(c.layer);
        if crate::gui::Drag::new(crate::gui::field(ui, "Layer"))
            .range(-64, 64)
            .speed(0.05)
            .build(ui, &mut layer)
        {
            c.layer = layer.clamp(-64, 64) as i8;
        }
        if ui.is_item_hovered() {
            ui.tooltip_text(
                "The world is layer 0: negative layers draw behind it, positive ones in front",
            );
        }
        let mut mode = usize::from(c.follow_viewport);
        if segmented(ui, "Space", &["Screen", "World"], &mut mode) {
            c.follow_viewport = mode == 1;
        }
        if !c.follow_viewport {
            vec2(ui, "Offset", &mut c.offset, 0.5, "%.0f");
        }
        crate::gui::toggle(ui, "Visible", &mut c.visible);
        crate::gui::muted(
            ui,
            if c.follow_viewport {
                "Attached nodes follow the camera but draw on this layer."
            } else {
                "Attached nodes are placed in screen pixels (0, 0 is the top-left corner) and ignore the camera."
            },
        );
        if c != before {
            commit(e, actor, component.id, &c);
        }
    }
    finish_band(e, actor, component, remove, reset);
}
fn canvas_modulate_band(
    ui: &Ui,
    e: &mut Editor,
    actor: Uuid,
    component: &crate::actor_document::ComponentInstance,
) {
    let (mut remove, mut reset) = (false, false);
    let open = band(
        ui,
        &format!("{ICON_MODULATE} CanvasModulate2D"),
        &mut remove,
        &mut reset,
        true,
    );
    if open {
        let mut m: CanvasModulate2D = world2d::read(component);
        let before = m.clone();
        colour_u8(ui, "Color", &mut m.color);
        let _ = crate::gui::field(ui, "Presets");
        for (label, colour) in [
            ("Day", [255, 255, 255]),
            ("Dusk", [255, 190, 150]),
            ("Night", [110, 120, 190]),
            ("Cave", [150, 140, 130]),
        ] {
            if ui.small_button(label) {
                m.color = colour;
            }
            ui.same_line();
        }
        ui.new_line();
        crate::gui::muted(
            ui,
            "Multiplies everything on this node's canvas; 255 leaves colours unchanged.",
        );
        if m != before {
            commit(e, actor, component.id, &m);
        }
    }
    finish_band(e, actor, component, remove, reset);
}
fn generic_band(
    ui: &Ui,
    e: &mut Editor,
    actor: Uuid,
    component: &crate::actor_document::ComponentInstance,
) {
    let mut remove = false;
    let open = crate::gui::section(ui, &format!("{}##{}", component.name, component.id), || {
        if !component.inherited && crate::gui::script_menu_item(ui, "Remove Component") {
            remove = true;
        }
    });
    if open {
        ui.text_disabled(crate::actor_document::short_class_name(
            &component.class.name,
        ));
        crate::gui::edit_class_button(ui, e, &component.class);
        if let Some(action) = crate::gui::reflected_property_table(
            ui,
            e,
            "component",
            component
                .class
                .class_id
                .as_deref()
                .unwrap_or(component.class.name.as_str()),
            &component.properties,
            &component.overrides,
        ) && let Some(index) = e.scene.actor_index(actor)
            && let Some(target) = e.scene.actors[index]
                .components
                .iter_mut()
                .find(|c| c.id == component.id)
        {
            let (mut properties, mut overrides) =
                (target.properties.clone(), target.overrides.clone());
            crate::gui::apply_property_action(action, &mut properties, &mut overrides);
            target.properties = properties;
            target.overrides = overrides;
            e.changed();
        }
    }
    if remove {
        e.remove_actor_component(actor, component.id);
    }
}
fn add_component_popup(ui: &Ui, e: &mut Editor, actor: Uuid) {
    ui.popup("add-2d-component", || {
        ui.text_disabled("2D components");
        ui.separator();
        let present: Vec<String> = e
            .scene
            .actor_index(actor)
            .map(|i| {
                e.scene.actors[i]
                    .components
                    .iter()
                    .filter_map(|c| c.class.class_id.clone())
                    .collect()
            })
            .unwrap_or_default();
        for (id, icon, label, detail, multiple) in [
            (
                world2d::SPRITE2D,
                ICON_IMAGE,
                "Sprite2D",
                "Texture, sprite sheet and frame animations",
                false,
            ),
            (
                world2d::COLLIDER2D,
                ICON_BODY,
                "Collider2D",
                "Solid, trigger or one-way shape",
                true,
            ),
            (
                world2d::CHARACTER_BODY2D,
                ICON_RUN,
                "CharacterBody2D",
                "move_and_slide() with floor detection",
                false,
            ),
            (
                world2d::CAMERA2D,
                ICON_CAMERA,
                "Camera2D",
                "Follow, smoothing, limits, drag margins",
                false,
            ),
            (
                world2d::TILEMAP_LAYER,
                crate::tilemap_editor::ICON_TILEMAP,
                "TileMapLayer",
                "A grid of tiles painted from a TileSet",
                true,
            ),
            (
                world2d::PARALLAX2D,
                ICON_PARALLAX,
                "Parallax2D",
                "Scroll attached nodes against the camera; repeat",
                false,
            ),
            (
                world2d::CANVAS_LAYER2D,
                ICON_CANVAS,
                "CanvasLayer2D",
                "A drawing layer, in screen pixels or following the camera",
                false,
            ),
            (
                world2d::CANVAS_MODULATE2D,
                ICON_MODULATE,
                "CanvasModulate2D",
                "Tint everything on this canvas",
                false,
            ),
        ] {
            let taken = !multiple && present.iter().any(|p| p == id);
            let clicked = ui
                .selectable_config(format!("{icon}  {label}"))
                .disabled(taken)
                .size([280., 0.])
                .build();
            if ui.is_item_hovered_with_flags(imgui::ItemHoveredFlags::ALLOW_WHEN_DISABLED) {
                ui.tooltip_text(if taken {
                    "Already on this node"
                } else {
                    detail
                });
            }
            if clicked
                && let Some(index) = e.scene.actor_index(actor)
                && let Some(component) = world2d::new_component(&e.scene.actors[index], id)
            {
                e.scene.actors[index].components.push(component);
                e.scene.actors[index].refresh_components();
                e.changed();
            }
        }
        let others: Vec<_> = e
            .addable_component_classes(actor)
            .into_iter()
            .filter(|(_, cpp, _)| {
                !matches!(
                    cpp.as_str(),
                    "epok::Sprite2DComponent"
                        | "epok::Collider2DComponent"
                        | "epok::CharacterBody2DComponent"
                        | "epok::Camera2DComponent"
                        | "epok::TileMapLayerComponent"
                        | "epok::Parallax2DComponent"
                        | "epok::CanvasLayer2DComponent"
                        | "epok::CanvasModulate2DComponent"
                )
            })
            .collect();
        if !others.is_empty() {
            ui.separator();
            ui.text_disabled("Scripts and shared components");
            for (_, cpp, label) in others {
                if ui.selectable(&label) {
                    e.add_actor_component(actor, &cpp);
                }
            }
        }
    });
}
/// Dropping a texture asset from the Content Browser on the previous item assigns it.
fn texture_drop(ui: &Ui, e: &Editor, texture: &mut Option<Uuid>) {
    if let Some(target) = ui.drag_drop_target()
        && let Some(Ok(payload)) =
            target.accept_payload::<u8, _>("EPOK_CONTENT", imgui::DragDropFlags::empty())
        && payload.delivery
        && let Some(path) = e.project_browser.drag.first()
        && let Some(id) = texture_for_path(e, path)
    {
        *texture = Some(id);
    }
}

// ---- dialogs ---------------------------------------------------------------------------
/// The region editor and sprite-sheet slicer windows. Call once per frame.
pub fn dialogs(ui: &Ui, e: &mut Editor) {
    region_editor(ui, e);
    sheet_slicer(ui, e);
    crate::tilemap_editor::dialogs(ui, e);
    crate::tileset_editor::window(ui, e);
}
fn texture_canvas(
    ui: &Ui,
    cache: &mut TextureCache,
    texture: Uuid,
    zoom: &mut f32,
    height: f32,
) -> Option<([f32; 2], f32, [u16; 2])> {
    let (id, size) = cache.get(texture)?;
    let available = [ui.content_region_avail()[0], height];
    if *zoom <= 0. {
        let fit = ((available[0] - 8.) / f32::from(size[0]))
            .min((available[1] - 8.) / f32::from(size[1]));
        *zoom = 2_f32.powf(fit.max(0.25).log2().floor()).clamp(0.25, 16.);
    }
    let canvas = ui.cursor_screen_pos();
    ui.invisible_button("##texture-canvas", available);
    if ui.is_item_hovered() && ui.io().mouse_wheel != 0. {
        *zoom = (*zoom * if ui.io().mouse_wheel > 0. { 2. } else { 0.5 }).clamp(0.25, 16.);
    }
    let draw = ui.get_window_draw_list();
    let image = [f32::from(size[0]) * *zoom, f32::from(size[1]) * *zoom];
    let min = [
        canvas[0] + ((available[0] - image[0]) * 0.5).max(4.),
        canvas[1] + ((available[1] - image[1]) * 0.5).max(4.),
    ];
    draw.with_clip_rect_intersect(
        canvas,
        [canvas[0] + available[0], canvas[1] + available[1]],
        || {
            draw.add_rect(
                canvas,
                [canvas[0] + available[0], canvas[1] + available[1]],
                CANVAS,
            )
            .filled(true)
            .build();
            checker(&draw, min, [min[0] + image[0], min[1] + image[1]], 8.);
            draw.add_image(id, min, [min[0] + image[0], min[1] + image[1]])
                .build();
            draw.add_rect(
                min,
                [min[0] + image[0], min[1] + image[1]],
                [1., 1., 1., 0.2],
            )
            .build();
        },
    );
    Some((min, *zoom, size))
}
fn region_editor(ui: &Ui, e: &mut Editor) {
    let Some(mut state) = e.world2d.region_editor.clone() else {
        return;
    };
    let texture = e
        .scene
        .actor_index(state.actor)
        .and_then(|i| {
            e.scene.actors[i]
                .components
                .iter()
                .find(|c| c.id == state.component)
        })
        .and_then(|c| world2d::read::<Sprite2D>(c).texture);
    let Some(texture) = texture else {
        e.world2d.region_editor = None;
        return;
    };
    let mut open = true;
    let mut apply = false;
    ui.window(format!("{ICON_CROP} Sprite Region###world2d-region"))
        .opened(&mut open)
        .size([620., 560.], imgui::Condition::FirstUseEver)
        .build(|| {
            ui.text_disabled("Drag on the texture to select the region this sprite draws.");
            ui.set_next_item_width(120.);
            let steps = [1u16, 4, 8, 16, 32];
            let mut snap = steps.iter().position(|v| *v == state.snap).unwrap_or(0);
            if ui.combo_simple_string(
                "Snap",
                &mut snap,
                &["Pixel", "4 px", "8 px", "16 px", "32 px"],
            ) {
                state.snap = steps[snap];
            }
            ui.same_line();
            ui.text_disabled(format!("Zoom {:.0}%  (wheel)", state.zoom * 100.));
            let height = (ui.content_region_avail()[1] - 70.).max(120.);
            if let Some((min, zoom, size)) = texture_canvas(
                ui,
                &mut e.world2d.textures,
                texture,
                &mut state.zoom,
                height,
            ) {
                let draw = ui.get_window_draw_list();
                let mouse = ui.io().mouse_pos;
                let texel = |p: [f32; 2], round_up: bool| {
                    let step = f32::from(state.snap.max(1));
                    let raw = [((p[0] - min[0]) / zoom), ((p[1] - min[1]) / zoom)];
                    raw.map(|v| {
                        let s = if round_up {
                            (v / step).ceil() * step
                        } else {
                            (v / step).floor() * step
                        };
                        s.max(0.)
                    })
                };
                if ui.is_item_clicked() {
                    state.dragging = Some(texel(mouse, false));
                }
                if let Some(start) = state.dragging {
                    if ui.is_mouse_down(MouseButton::Left) {
                        let end = texel(mouse, true);
                        let lo = [start[0].min(end[0]), start[1].min(end[1])];
                        let hi = [
                            start[0].max(end[0]).min(f32::from(size[0])),
                            start[1].max(end[1]).min(f32::from(size[1])),
                        ];
                        state.region = [
                            lo[0] as u16,
                            lo[1] as u16,
                            (hi[0] - lo[0]).max(1.) as u16,
                            (hi[1] - lo[1]).max(1.) as u16,
                        ];
                    } else {
                        state.dragging = None;
                    }
                }
                let r = state.region;
                let (w, h) = (
                    if r[2] == 0 { size[0] - r[0] } else { r[2] },
                    if r[3] == 0 { size[1] - r[1] } else { r[3] },
                );
                let a = [
                    min[0] + f32::from(r[0]) * zoom,
                    min[1] + f32::from(r[1]) * zoom,
                ];
                let b = [a[0] + f32::from(w) * zoom, a[1] + f32::from(h) * zoom];
                draw.add_rect(a, b, [1.0, 0.62, 0.2, 0.18])
                    .filled(true)
                    .build();
                draw.add_rect(a, b, SELECTION).thickness(2.).build();
            }
            let mut values = state.region.map(i32::from);
            ui.set_next_item_width(280.);
            if crate::gui::Drag::new("x / y / w / h")
                .range(0, 256)
                .build_array(ui, &mut values)
            {
                state.region = values.map(|v| v.clamp(0, 256) as u16);
            }
            ui.same_line();
            if ui.button("Whole texture") {
                state.region = [0; 4];
            }
            if ui.button_with_size("Apply", [100., 0.]) {
                apply = true;
            }
            ui.same_line();
            if ui.button_with_size("Cancel", [100., 0.]) {
                state.dragging = None;
                e.world2d.region_editor = None;
            }
        });
    if apply {
        let region = state.region;
        edit_component::<Sprite2D>(e, state.actor, state.component, |s| {
            s.region = region;
            s.hframes = 1;
            s.vframes = 1;
            s.frame = 0;
        });
        e.changed();
        e.world2d.region_editor = None;
        return;
    }
    if !open {
        e.world2d.region_editor = None;
    } else if e.world2d.region_editor.is_some() {
        e.world2d.region_editor = Some(state);
    }
}
fn sheet_slicer(ui: &Ui, e: &mut Editor) {
    let Some(mut state) = e.world2d.slicer.clone() else {
        return;
    };
    let sprite = e
        .scene
        .actor_index(state.actor)
        .and_then(|i| {
            e.scene.actors[i]
                .components
                .iter()
                .find(|c| c.id == state.component)
        })
        .map(world2d::read::<Sprite2D>);
    let Some(texture) = sprite.as_ref().and_then(|s| s.texture) else {
        e.world2d.slicer = None;
        return;
    };
    let animation_name = sprite
        .as_ref()
        .and_then(|s| s.animations.get(state.animation))
        .map(|a| a.name.clone())
        .unwrap_or_default();
    let mut open = true;
    let mut add = false;
    ui.window(format!(
        "{ICON_SLICE} Add Frames from Sprite Sheet###world2d-slicer"
    ))
    .opened(&mut open)
    .size([660., 600.], imgui::Condition::FirstUseEver)
    .build(|| {
        ui.text(format!("Animation: {animation_name}"));
        ui.text_disabled("Set the grid, then click cells in the order the frames should play.");
        let mut grid = [i32::from(state.columns), i32::from(state.rows)];
        ui.set_next_item_width(160.);
        if crate::gui::Drag::new("Columns / Rows")
            .range(1, 64)
            .speed(0.05)
            .build_array(ui, &mut grid)
        {
            state.columns = grid[0].clamp(1, 64) as u16;
            state.rows = grid[1].clamp(1, 64) as u16;
            state.selected.clear();
        }
        ui.same_line();
        let mut offset = state.offset.map(i32::from);
        ui.set_next_item_width(110.);
        if crate::gui::Drag::new("Offset")
            .range(0, 255)
            .build_array(ui, &mut offset)
        {
            state.offset = offset.map(|v| v.clamp(0, 255) as u16);
        }
        ui.same_line();
        let mut separation = state.separation.map(i32::from);
        ui.set_next_item_width(110.);
        if crate::gui::Drag::new("Separation")
            .range(0, 64)
            .build_array(ui, &mut separation)
        {
            state.separation = separation.map(|v| v.clamp(0, 64) as u16);
        }
        let height = (ui.content_region_avail()[1] - 64.).max(120.);
        let mut cell_size = [0u16; 2];
        if let Some((min, zoom, size)) = texture_canvas(
            ui,
            &mut e.world2d.textures,
            texture,
            &mut state.zoom,
            height,
        ) {
            let usable = [
                size[0]
                    .saturating_sub(state.offset[0])
                    .saturating_add(state.separation[0]),
                size[1]
                    .saturating_sub(state.offset[1])
                    .saturating_add(state.separation[1]),
            ];
            cell_size = [
                (usable[0] / state.columns.max(1))
                    .saturating_sub(state.separation[0])
                    .max(1),
                (usable[1] / state.rows.max(1))
                    .saturating_sub(state.separation[1])
                    .max(1),
            ];
            let draw = ui.get_window_draw_list();
            let mouse = ui.io().mouse_pos;
            let clicked = ui.is_item_clicked();
            for row in 0..state.rows {
                for column in 0..state.columns {
                    let cell = row * state.columns + column;
                    let x = state.offset[0] + column * (cell_size[0] + state.separation[0]);
                    let y = state.offset[1] + row * (cell_size[1] + state.separation[1]);
                    let a = [min[0] + f32::from(x) * zoom, min[1] + f32::from(y) * zoom];
                    let b = [
                        a[0] + f32::from(cell_size[0]) * zoom,
                        a[1] + f32::from(cell_size[1]) * zoom,
                    ];
                    let inside =
                        mouse[0] >= a[0] && mouse[0] < b[0] && mouse[1] >= a[1] && mouse[1] < b[1];
                    let order = state.selected.iter().position(|c| *c == cell);
                    if clicked && inside {
                        match order {
                            Some(i) => {
                                state.selected.remove(i);
                            }
                            None => state.selected.push(cell),
                        }
                    }
                    let colour = if order.is_some() {
                        SELECTION
                    } else if inside {
                        [1., 1., 1., 0.6]
                    } else {
                        [1., 1., 1., 0.22]
                    };
                    if order.is_some() {
                        draw.add_rect(a, b, [1.0, 0.62, 0.2, 0.2])
                            .filled(true)
                            .build();
                    }
                    draw.add_rect(a, b, colour).build();
                    if let Some(i) = state.selected.iter().position(|c| *c == cell) {
                        draw.add_rect(
                            [a[0] + 2., a[1] + 2.],
                            [a[0] + 20., a[1] + 16.],
                            [0., 0., 0., 0.7],
                        )
                        .filled(true)
                        .build();
                        draw.add_text([a[0] + 4., a[1] + 2.], SELECTION, format!("{}", i + 1));
                    }
                }
            }
        }
        ui.text_disabled(format!(
            "Cell {} x {} px  ·  {} selected",
            cell_size[0],
            cell_size[1],
            state.selected.len()
        ));
        if ui.button("Select all") {
            state.selected = (0..state.columns * state.rows).collect();
        }
        ui.same_line();
        if ui.button("Select row 1") {
            state.selected = (0..state.columns).collect();
        }
        ui.same_line();
        if ui.button("Clear") {
            state.selected.clear();
        }
        ui.same_line();
        ui.disabled(state.selected.is_empty(), || {
            if ui.button_with_size(
                format!(
                    "Add {} Frame{}",
                    state.selected.len(),
                    if state.selected.len() == 1 { "" } else { "s" }
                ),
                [140., 0.],
            ) {
                add = true;
            }
        });
        state.zoom = state.zoom.max(0.);
        let _ = cell_size;
        e.world2d.slicer_cell = cell_size;
    });
    if add {
        let cell = e.world2d.slicer_cell;
        let frames: Vec<Frame2D> = state
            .selected
            .iter()
            .map(|index| {
                let column = index % state.columns.max(1);
                let row = index / state.columns.max(1);
                Frame2D {
                    region: [
                        state.offset[0] + column * (cell[0] + state.separation[0]),
                        state.offset[1] + row * (cell[1] + state.separation[1]),
                        cell[0],
                        cell[1],
                    ],
                    ..Default::default()
                }
            })
            .collect();
        let animation = state.animation;
        edit_component::<Sprite2D>(e, state.actor, state.component, |s| {
            if let Some(a) = s.animations.get_mut(animation) {
                a.frames.extend(frames);
            }
        });
        e.changed();
        e.world2d.slicer = None;
        return;
    }
    if !open {
        e.world2d.slicer = None;
    } else if e.world2d.slicer.is_some() {
        e.world2d.slicer = Some(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opaque_bounds_trim_transparent_margins() {
        let mut rgba = vec![0u8; 8 * 4 * 4];
        for (x, y) in [(3, 1), (5, 2)] {
            rgba[(y * 8 + x) * 4 + 3] = 255;
        }
        let data = crate::texture::Data {
            width: 8,
            height: 4,
            rgba,
            words: Vec::new(),
            palette: Vec::new(),
        };
        assert_eq!(
            opaque_bounds(&data, [0, 0, 8, 4]),
            Some(([3., 1.], [6., 3.]))
        );
        // Relative to the frame rectangle, and empty for a transparent frame.
        assert_eq!(
            opaque_bounds(&data, [2, 0, 4, 4]),
            Some(([1., 1.], [4., 3.]))
        );
        assert_eq!(opaque_bounds(&data, [6, 0, 2, 4]), None);
    }
}
