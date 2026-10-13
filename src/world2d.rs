//! World2D components on the editor side: their document shape and defaults, the
//! 2D "node" presets the editor creates, the textures and TileSets they reference and
//! the C++ they cook into a scene bank (see `runtime/world2d_components.hpp` and
//! `runtime/world2d_tiles.hpp`).
//!
//! Units match the runtime: pixels, X right, Y down, degrees. Each component stores
//! one typed record under a single property key, the way the 3D built-ins do.
use crate::{
    actor_document::{ActorInstance, ClassReference, ComponentInstance},
    object_model as om,
    texture::BlendMode,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
use uuid::Uuid;

pub const SPRITE2D: &str = om::SPRITE2D_COMPONENT_ID;
pub const CAMERA2D: &str = om::CAMERA2D_COMPONENT_ID;
pub const COLLIDER2D: &str = om::COLLIDER2D_COMPONENT_ID;
pub const CHARACTER_BODY2D: &str = om::CHARACTER_BODY2D_COMPONENT_ID;
pub const TILEMAP_LAYER: &str = om::TILEMAP_LAYER_COMPONENT_ID;
pub const PARALLAX2D: &str = om::PARALLAX2D_COMPONENT_ID;
pub const CANVAS_LAYER2D: &str = om::CANVAS_LAYER2D_COMPONENT_ID;
pub const CANVAS_MODULATE2D: &str = om::CANVAS_MODULATE2D_COMPONENT_ID;
pub const PATH2D: &str = om::PATH2D_COMPONENT_ID;
pub const PATH_FOLLOW2D: &str = om::PATH_FOLLOW2D_COMPONENT_ID;
pub const LINE2D: &str = om::LINE2D_COMPONENT_ID;
pub const POLYGON2D: &str = om::POLYGON2D_COMPONENT_ID;
pub const PARTICLES2D: &str = om::PARTICLES2D_COMPONENT_ID;
pub const AUDIO_PLAYER2D: &str = om::AUDIO_PLAYER2D_COMPONENT_ID;
pub const NOTIFIER2D: &str = om::NOTIFIER2D_COMPONENT_ID;
pub const REMOTE_TRANSFORM2D: &str = om::REMOTE_TRANSFORM2D_COMPONENT_ID;
pub const SKELETON2D: &str = om::SKELETON2D_COMPONENT_ID;
pub const NAVIGATION_REGION2D: &str = om::NAVIGATION_REGION2D_COMPONENT_ID;
pub const NAVIGATION_AGENT2D: &str = om::NAVIGATION_AGENT2D_COMPONENT_ID;

/// Every engine 2D component the editor authors and cooks itself: its class id, the
/// property key holding its record, and its C++ class.
const BUILTINS: [(&str, &str, &str); 19] = [
    (SPRITE2D, "sprite2d", "epok::Sprite2DComponent"),
    (CAMERA2D, "camera2d", "epok::Camera2DComponent"),
    (COLLIDER2D, "collider2d", "epok::Collider2DComponent"),
    (CHARACTER_BODY2D, "body2d", "epok::CharacterBody2DComponent"),
    (TILEMAP_LAYER, "tilemap2d", "epok::TileMapLayerComponent"),
    (PARALLAX2D, "parallax2d", "epok::Parallax2DComponent"),
    (
        CANVAS_LAYER2D,
        "canvas_layer2d",
        "epok::CanvasLayer2DComponent",
    ),
    (
        CANVAS_MODULATE2D,
        "canvas_modulate2d",
        "epok::CanvasModulate2DComponent",
    ),
    (PATH2D, "path2d", "epok::Path2DComponent"),
    (
        PATH_FOLLOW2D,
        "path_follow2d",
        "epok::PathFollow2DComponent",
    ),
    (LINE2D, "line2d", "epok::Line2DComponent"),
    (POLYGON2D, "polygon2d", "epok::Polygon2DComponent"),
    (PARTICLES2D, "particles2d", "epok::Particles2DComponent"),
    (
        AUDIO_PLAYER2D,
        "audio_player2d",
        "epok::AudioPlayer2DComponent",
    ),
    (
        NOTIFIER2D,
        "notifier2d",
        "epok::VisibleOnScreenNotifier2DComponent",
    ),
    (
        REMOTE_TRANSFORM2D,
        "remote_transform2d",
        "epok::RemoteTransform2DComponent",
    ),
    (SKELETON2D, "skeleton2d", "epok::Skeleton2DComponent"),
    (
        NAVIGATION_REGION2D,
        "navigation_region2d",
        "epok::NavigationRegion2DComponent",
    ),
    (
        NAVIGATION_AGENT2D,
        "navigation_agent2d",
        "epok::NavigationAgent2DComponent",
    ),
];
pub fn is_builtin(class_id: &str) -> bool {
    BUILTINS.iter().any(|b| b.0 == class_id)
}
pub fn key(class_id: &str) -> Option<&'static str> {
    BUILTINS.iter().find(|b| b.0 == class_id).map(|b| b.1)
}
pub fn class_name(class_id: &str) -> Option<&'static str> {
    BUILTINS.iter().find(|b| b.0 == class_id).map(|b| b.2)
}

// ---- document records ------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Frame2D {
    /// Texel rectangle x, y, width, height.
    pub region: [u16; 4],
    /// Relative duration: the frame lasts `duration / fps` seconds, so an animation's speed scales all of its frames.
    pub duration: f32,
    /// Event id reported when the frame is entered; 0 = none.
    pub event: u16,
}
impl Default for Frame2D {
    fn default() -> Self {
        Self {
            region: [0, 0, 16, 16],
            duration: 1.,
            event: 0,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Animation2D {
    pub name: String,
    pub fps: f32,
    pub looping: bool,
    pub frames: Vec<Frame2D>,
}
impl Default for Animation2D {
    fn default() -> Self {
        Self {
            name: "default".into(),
            fps: 10.,
            looping: true,
            frames: Vec::new(),
        }
    }
}
impl Animation2D {
    pub fn frame_seconds(&self, frame: &Frame2D) -> f32 {
        (frame.duration.max(0.01) / self.fps.max(0.01)).max(1. / 60.)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Sprite2D {
    pub texture: Option<Uuid>,
    /// Texel rectangle x, y, width, height; zero width/height use the rest of the texture.
    pub region: [u16; 4],
    pub hframes: u16,
    pub vframes: u16,
    pub frame: u16,
    pub offset: [f32; 2],
    pub centered: bool,
    pub flip_h: bool,
    pub flip_v: bool,
    pub visible: bool,
    pub y_sort: bool,
    pub modulate: [u8; 3],
    pub blend: BlendMode,
    pub z_index: i16,
    pub animations: Vec<Animation2D>,
    /// Animation shown first and, with `autoplay`, played on load.
    pub animation: u16,
    pub autoplay: bool,
    pub speed_scale: f32,
}
impl Default for Sprite2D {
    fn default() -> Self {
        Self {
            texture: None,
            region: [0; 4],
            hframes: 1,
            vframes: 1,
            frame: 0,
            offset: [0.; 2],
            centered: true,
            flip_h: false,
            flip_v: false,
            visible: true,
            y_sort: false,
            modulate: [255; 3],
            blend: BlendMode::Cutout,
            z_index: 0,
            animations: Vec::new(),
            animation: 0,
            autoplay: true,
            speed_scale: 1.,
        }
    }
}
impl Sprite2D {
    /// The texel rectangle drawn in the editor for a `texture` of `size`, mirroring
    /// `Sprite2DComponent::source_rect`.
    pub fn source_rect(&self, size: [u16; 2]) -> Option<[u16; 4]> {
        let mut out;
        if let Some(frame) = self
            .animations
            .get(usize::from(self.animation))
            .and_then(|a| a.frames.first())
        {
            out = frame.region;
        } else {
            out = self.region;
            if out[2] == 0 {
                out[2] = size[0].saturating_sub(out[0]);
            }
            if out[3] == 0 {
                out[3] = size[1].saturating_sub(out[1]);
            }
            let (columns, rows) = (self.hframes.max(1), self.vframes.max(1));
            if columns > 1 || rows > 1 {
                let cell = self.frame % (columns * rows);
                out[2] /= columns;
                out[3] /= rows;
                out[0] += (cell % columns) * out[2];
                out[1] += (cell / columns) * out[3];
            }
        }
        (out[2] > 0
            && out[3] > 0
            && u32::from(out[0]) + u32::from(out[2]) <= u32::from(size[0])
            && u32::from(out[1]) + u32::from(out[3]) <= u32::from(size[1]))
        .then_some(out)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Camera2D {
    pub current: bool,
    pub zoom: f32,
    pub offset: [f32; 2],
    pub rotating: bool,
    pub smoothing: bool,
    pub smoothing_speed: f32,
    pub limits_enabled: bool,
    /// Left, top, right, bottom world pixels.
    pub limits: [f32; 4],
    pub drag_enabled: bool,
    /// Left, top, right, bottom fractions of the half view.
    pub drag_margins: [f32; 4],
    pub background: [u8; 3],
}
impl Default for Camera2D {
    fn default() -> Self {
        Self {
            current: true,
            zoom: 1.,
            offset: [0.; 2],
            rotating: false,
            smoothing: false,
            smoothing_speed: 5.,
            limits_enabled: false,
            limits: [-10000., -10000., 10000., 10000.],
            drag_enabled: false,
            drag_margins: [0.2; 4],
            background: [33, 40, 52],
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum Shape2D {
    #[default]
    Box,
    Circle,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Collider2D {
    pub shape: Shape2D,
    /// Full box size in pixels.
    pub size: [f32; 2],
    pub radius: f32,
    pub offset: [f32; 2],
    pub layer: u32,
    pub mask: u32,
    pub trigger: bool,
    pub one_way: bool,
    pub one_way_margin: f32,
    pub enabled: bool,
}
impl Default for Collider2D {
    fn default() -> Self {
        Self {
            shape: Shape2D::Box,
            size: [16., 16.],
            radius: 8.,
            offset: [0.; 2],
            layer: 1,
            mask: u32::MAX,
            trigger: false,
            one_way: false,
            one_way_margin: 1.,
            enabled: true,
        }
    }
}
impl Collider2D {
    /// Local axis-aligned bounds (min, max) around the owner, before its transform.
    pub fn bounds(&self) -> ([f32; 2], [f32; 2]) {
        let half = match self.shape {
            Shape2D::Box => [self.size[0].abs() * 0.5, self.size[1].abs() * 0.5],
            Shape2D::Circle => [self.radius.abs(); 2],
        };
        (
            [self.offset[0] - half[0], self.offset[1] - half[1]],
            [self.offset[0] + half[0], self.offset[1] + half[1]],
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct CharacterBody2D {
    pub up_direction: [f32; 2],
    pub floor_max_angle: f32,
    pub floor_snap_length: f32,
    pub collision_mask: u32,
    pub floating: bool,
}
impl Default for CharacterBody2D {
    fn default() -> Self {
        Self {
            up_direction: [0., -1.],
            floor_max_angle: 45.,
            floor_snap_length: 4.,
            collision_mask: u32::MAX,
            floating: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct TileMapLayer {
    pub tile_set: Option<Uuid>,
    /// Painted cells, encoded by `tilemap::encode`.
    pub cells: String,
    pub z_index: i16,
    pub modulate: [u8; 3],
    pub blend: BlendMode,
    pub visible: bool,
    pub collision_enabled: bool,
    pub collision_layer: u32,
}
impl Default for TileMapLayer {
    fn default() -> Self {
        Self {
            tile_set: None,
            cells: String::new(),
            z_index: 0,
            modulate: [255; 3],
            blend: BlendMode::Cutout,
            visible: true,
            collision_enabled: true,
            collision_layer: 1,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Parallax2D {
    /// 1 moves with the world, 0 stays on screen.
    pub scroll_scale: [f32; 2],
    /// Pixels between repeated copies on each axis; 0 does not repeat.
    pub repeat_size: [f32; 2],
    /// Drift in pixels per second.
    pub autoscroll: [f32; 2],
}
impl Default for Parallax2D {
    fn default() -> Self {
        Self {
            scroll_scale: [0.5, 0.5],
            repeat_size: [0.; 2],
            autoscroll: [0.; 2],
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct CanvasLayer2D {
    pub layer: i8,
    pub offset: [f32; 2],
    pub follow_viewport: bool,
    pub visible: bool,
}
impl Default for CanvasLayer2D {
    fn default() -> Self {
        Self {
            layer: 1,
            offset: [0.; 2],
            follow_viewport: false,
            visible: true,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct CanvasModulate2D {
    pub color: [u8; 3],
}
impl Default for CanvasModulate2D {
    fn default() -> Self {
        Self { color: [255; 3] }
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PathPoint {
    pub position: [f32; 2],
    /// Curve handles, relative to the point; zero handles make straight segments.
    pub handle_in: [f32; 2],
    pub handle_out: [f32; 2],
}
impl PathPoint {
    pub fn at(position: [f32; 2]) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Path2D {
    pub points: Vec<PathPoint>,
    pub closed: bool,
}
impl Default for Path2D {
    fn default() -> Self {
        Self {
            points: vec![PathPoint::at([0., 0.]), PathPoint::at([96., 0.])],
            closed: false,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PathFollow2D {
    /// The actor whose Path2D to follow; none follows the actor this one is attached to.
    pub path: Option<Uuid>,
    pub progress: f32,
    pub speed: f32,
    /// Along and across the path.
    pub offset: [f32; 2],
    pub looping: bool,
    pub ping_pong: bool,
    pub rotates: bool,
    pub moving: bool,
}
impl Default for PathFollow2D {
    fn default() -> Self {
        Self {
            path: None,
            progress: 0.,
            speed: 48.,
            offset: [0.; 2],
            looping: true,
            ping_pong: false,
            rotates: false,
            moving: true,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Line2D {
    pub points: Vec<[f32; 2]>,
    pub width: f32,
    pub begin_color: [u8; 3],
    pub end_color: [u8; 3],
    pub closed: bool,
    pub blend: BlendMode,
    pub z_index: i16,
    pub visible: bool,
}
impl Default for Line2D {
    fn default() -> Self {
        Self {
            points: vec![[0., 0.], [64., 0.]],
            width: 2.,
            begin_color: [255; 3],
            end_color: [255; 3],
            closed: false,
            blend: BlendMode::Cutout,
            z_index: 0,
            visible: true,
        }
    }
}
/// Line2D points the console stores (`line2d_max_points` in world2d_content.hpp).
pub const LINE_MAX_POINTS: usize = 32;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Polygon2D {
    pub points: Vec<[f32; 2]>,
    pub color: [u8; 3],
    pub texture: Option<Uuid>,
    /// Texel drawn at the polygon's local origin; the texture is not scaled.
    pub texture_offset: [f32; 2],
    pub blend: BlendMode,
    pub z_index: i16,
    pub visible: bool,
}
impl Default for Polygon2D {
    fn default() -> Self {
        Self {
            points: vec![[-16., -16.], [16., -16.], [16., 16.], [-16., 16.]],
            color: [255; 3],
            texture: None,
            texture_offset: [0.; 2],
            blend: BlendMode::Cutout,
            z_index: 0,
            visible: true,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum EmissionShape {
    #[default]
    Point,
    Rectangle,
    Circle,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Particles2D {
    pub emitting: bool,
    pub one_shot: bool,
    pub local_coords: bool,
    pub visible: bool,
    pub amount: u16,
    pub lifetime: f32,
    /// 0 is a steady stream, 1 releases the whole amount at once.
    pub explosiveness: f32,
    /// Degrees: 0 right, 90 down, -90 up.
    pub direction: f32,
    pub spread: f32,
    /// Minimum and maximum launch speed, pixels per second.
    pub speed: [f32; 2],
    pub gravity: [f32; 2],
    pub damping: f32,
    /// Start and end scale.
    pub scale: [f32; 2],
    pub start_color: [u8; 3],
    pub end_color: [u8; 3],
    pub shape: EmissionShape,
    /// Rectangle half size, or the circle radius in the first value.
    pub extents: [f32; 2],
    pub texture: Option<Uuid>,
    pub region: [u16; 4],
    /// Frames across the region, played over each particle's life.
    pub hframes: u16,
    pub blend: BlendMode,
    pub z_index: i16,
    pub seed: u32,
}
impl Default for Particles2D {
    fn default() -> Self {
        Self {
            emitting: true,
            one_shot: false,
            local_coords: false,
            visible: true,
            amount: 16,
            lifetime: 1.,
            explosiveness: 0.,
            direction: -90.,
            spread: 30.,
            speed: [40., 80.],
            gravity: [0., 98.],
            damping: 0.,
            scale: [1., 1.],
            start_color: [255; 3],
            end_color: [255; 3],
            shape: EmissionShape::Point,
            extents: [8.; 2],
            texture: None,
            region: [0; 4],
            hframes: 1,
            blend: BlendMode::Add,
            z_index: 0,
            seed: 1,
        }
    }
}
/// The 2D particle budget a build cooks at most, shared by every emitter.
pub const MAX_PARTICLES: usize = 512;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct AudioPlayer2D {
    pub audio: crate::audio::AudioSource,
    pub max_distance: f32,
    pub attenuation: f32,
    pub panning: f32,
}
impl Default for AudioPlayer2D {
    fn default() -> Self {
        Self {
            audio: crate::audio::AudioSource {
                play_on_start: false,
                ..Default::default()
            },
            max_distance: 400.,
            attenuation: 1.,
            panning: 1.,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Notifier2D {
    /// x, y, width, height around the owner.
    pub rect: [f32; 4],
    pub disable_when_hidden: bool,
}
impl Default for Notifier2D {
    fn default() -> Self {
        Self {
            rect: [-10., -10., 20., 20.],
            disable_when_hidden: false,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct RemoteTransform2D {
    pub target: Option<Uuid>,
    pub update_position: bool,
    pub update_rotation: bool,
    pub update_scale: bool,
}
impl Default for RemoteTransform2D {
    fn default() -> Self {
        Self {
            target: None,
            update_position: true,
            update_rotation: true,
            update_scale: true,
        }
    }
}

// ---- document access -------------------------------------------------------------------

pub fn read<T: DeserializeOwned + Default>(component: &ComponentInstance) -> T {
    key(component.class.class_id.as_deref().unwrap_or(""))
        .and_then(|key| component.properties.get(key))
        .and_then(|value| serde_json::from_value(value.clone()).ok())
        .unwrap_or_default()
}
pub fn write<T: Serialize>(component: &mut ComponentInstance, value: &T) {
    if let Some(key) = key(component.class.class_id.as_deref().unwrap_or("")) {
        component.properties.insert(
            key.into(),
            serde_json::to_value(value).expect("World2D record"),
        );
        component.overrides.insert(key.into());
    }
}
/// Validation used when a document is loaded: the record must decode.
pub fn validate(component: &ComponentInstance) -> Result<(), String> {
    let id = component.class.class_id.as_deref().unwrap_or("");
    let Some(key) = key(id) else { return Ok(()) };
    let Some(value) = component.properties.get(key).filter(|v| !v.is_null()) else {
        return Ok(());
    };
    fn decode<T: DeserializeOwned>(value: &serde_json::Value) -> Result<T, String> {
        serde_json::from_value(value.clone()).map_err(|e| e.to_string())
    }
    let result = match id {
        SPRITE2D => decode::<Sprite2D>(value).map(|_| ()),
        CAMERA2D => decode::<Camera2D>(value).map(|_| ()),
        COLLIDER2D => decode::<Collider2D>(value).map(|_| ()),
        TILEMAP_LAYER => decode::<TileMapLayer>(value)
            .and_then(|layer| crate::tilemap::decode(&layer.cells).map(|_| ())),
        PARALLAX2D => decode::<Parallax2D>(value).map(|_| ()),
        CANVAS_LAYER2D => decode::<CanvasLayer2D>(value).map(|_| ()),
        CANVAS_MODULATE2D => decode::<CanvasModulate2D>(value).map(|_| ()),
        PATH2D => decode::<Path2D>(value).map(|_| ()),
        PATH_FOLLOW2D => decode::<PathFollow2D>(value).map(|_| ()),
        LINE2D => decode::<Line2D>(value).and_then(|line| {
            if line.points.len() > LINE_MAX_POINTS {
                Err(format!("a Line2D holds at most {LINE_MAX_POINTS} points"))
            } else {
                Ok(())
            }
        }),
        POLYGON2D => decode::<Polygon2D>(value).map(|_| ()),
        PARTICLES2D => decode::<Particles2D>(value).map(|_| ()),
        AUDIO_PLAYER2D => decode::<AudioPlayer2D>(value).map(|_| ()),
        NOTIFIER2D => decode::<Notifier2D>(value).map(|_| ()),
        REMOTE_TRANSFORM2D => decode::<RemoteTransform2D>(value).map(|_| ()),
        SKELETON2D => decode::<crate::skeleton2d::Skeleton2D>(value).and_then(|s| s.validate()),
        NAVIGATION_REGION2D => {
            decode::<crate::navigation2d::NavigationRegion2D>(value).and_then(|r| {
                if r.outline.len() > crate::navigation2d::MAX_OUTLINE {
                    Err(format!(
                        "a navigation outline holds at most {} points",
                        crate::navigation2d::MAX_OUTLINE
                    ))
                } else {
                    Ok(())
                }
            })
        }
        NAVIGATION_AGENT2D => decode::<crate::navigation2d::NavigationAgent2D>(value).map(|_| ()),
        _ => decode::<CharacterBody2D>(value).map(|_| ()),
    };
    result.map_err(|e| format!("{key}: {e}"))
}
/// The default record of a built-in component, as stored in a document.
fn default_record(class_id: &str) -> serde_json::Value {
    fn value<T: Serialize + Default>() -> serde_json::Value {
        serde_json::to_value(T::default()).expect("World2D record")
    }
    match class_id {
        SPRITE2D => value::<Sprite2D>(),
        CAMERA2D => value::<Camera2D>(),
        COLLIDER2D => value::<Collider2D>(),
        TILEMAP_LAYER => value::<TileMapLayer>(),
        PARALLAX2D => value::<Parallax2D>(),
        CANVAS_LAYER2D => value::<CanvasLayer2D>(),
        CANVAS_MODULATE2D => value::<CanvasModulate2D>(),
        PATH2D => value::<Path2D>(),
        PATH_FOLLOW2D => value::<PathFollow2D>(),
        LINE2D => value::<Line2D>(),
        POLYGON2D => value::<Polygon2D>(),
        PARTICLES2D => value::<Particles2D>(),
        AUDIO_PLAYER2D => value::<AudioPlayer2D>(),
        NOTIFIER2D => value::<Notifier2D>(),
        REMOTE_TRANSFORM2D => value::<RemoteTransform2D>(),
        SKELETON2D => value::<crate::skeleton2d::Skeleton2D>(),
        NAVIGATION_REGION2D => value::<crate::navigation2d::NavigationRegion2D>(),
        NAVIGATION_AGENT2D => value::<crate::navigation2d::NavigationAgent2D>(),
        _ => value::<CharacterBody2D>(),
    }
}
pub fn component<'a>(actor: &'a ActorInstance, class_id: &str) -> Option<&'a ComponentInstance> {
    actor
        .components
        .iter()
        .find(|c| c.class.class_id.as_deref() == Some(class_id))
}
pub fn new_component(actor: &ActorInstance, class_id: &str) -> Option<ComponentInstance> {
    let class = class_name(class_id)?;
    let base = crate::actor_document::short_class_name(class).trim_end_matches("Component");
    let mut component = ComponentInstance::new(
        Uuid::new_v4(),
        ClassReference::new(class, class_id),
        &crate::actor_document::unique_component_name(actor, base),
    );
    write(&mut component, &default_record(class_id));
    Some(component)
}
/// Textures the actor's 2D components draw: sprites directly, tile layers through their
/// TileSet. They join the bank's texture set exactly like a 3D material's texture.
pub fn texture_ids<'a>(
    actor: &'a ActorInstance,
    tilesets: &'a BTreeMap<Uuid, Arc<crate::tileset::TileSet>>,
) -> impl Iterator<Item = Uuid> + 'a {
    actor
        .components
        .iter()
        .filter_map(move |c| match c.class.class_id.as_deref() {
            Some(SPRITE2D) => read::<Sprite2D>(c).texture,
            Some(TILEMAP_LAYER) => read::<TileMapLayer>(c)
                .tile_set
                .and_then(|id| tilesets.get(&id))
                .and_then(|set| set.texture),
            Some(POLYGON2D) => read::<Polygon2D>(c).texture,
            Some(PARTICLES2D) => read::<Particles2D>(c).texture,
            _ => None,
        })
}
/// TileSets the actor's tile layers reference.
pub fn tileset_ids(actor: &ActorInstance) -> impl Iterator<Item = Uuid> + '_ {
    actor
        .components
        .iter()
        .filter(|c| c.class.class_id.as_deref() == Some(TILEMAP_LAYER))
        .filter_map(|c| read::<TileMapLayer>(c).tile_set)
}
/// Whether a scene holds anything the World2D service must run for.
pub fn scene_uses_world2d(scene: &crate::scene::Scene) -> bool {
    scene.actors.iter().any(|actor| {
        actor.class.class_id.as_deref() == Some(om::ACTOR2D_ID)
            || actor.components.iter().any(|c| {
                c.class
                    .class_id
                    .as_deref()
                    .is_some_and(|id| is_builtin(id) || id == om::SCENE_COMPONENT2D_ID)
            })
    })
}

/// Engine classes of the 2D world. A project class of the 2D domain is the project's
/// own 2D content even when no map places it yet (code may spawn it).
fn engine_2d_class(id: &str) -> bool {
    id == om::ACTOR2D_ID || id == om::SCENE_COMPONENT2D_ID || is_builtin(id)
}
/// Whether a build needs the World2D service: a map or Blueprint template holds 2D
/// content, or the project declares a class of the 2D domain. Without it the build
/// compiles no 2D service and gives the engine 2D classes no object storage.
pub fn project_uses_world2d<'a>(
    scenes: impl IntoIterator<Item = &'a crate::scene::Scene>,
    registry: &crate::blueprint::Registry,
) -> bool {
    scenes.into_iter().any(scene_uses_world2d)
        || registry.classes.values().any(|class| {
            class.domain == Some(crate::reflection_schema::Domain::World2D)
                && !engine_2d_class(&class.id)
        })
}
/// `#define`s that switch the World2D service on for this build and size it, or nothing
/// when the build has no 2D content (the service then costs no RAM or code).
/// Capacities cover the largest bank plus headroom for actors spawned at runtime.
pub fn capacity_defines<'a>(
    enabled: bool,
    scenes: &'a [crate::scene::Scene],
    templates: impl Iterator<Item = &'a crate::scene::Scene>,
) -> String {
    let templates: Vec<_> = templates.collect();
    if !enabled {
        return String::new();
    }
    let count = |scene: &crate::scene::Scene, id: &str| {
        scene
            .actors
            .iter()
            .flat_map(|a| a.components.iter())
            .filter(|c| c.class.class_id.as_deref() == Some(id))
            .count()
    };
    let dynamic = |id: &str| templates.iter().map(|t| count(t, id)).sum::<usize>() * 4;
    let largest = |id: &str| scenes.iter().map(|s| count(s, id)).max().unwrap_or(0) + dynamic(id);
    // The collision world adds the tile probes to this; it stays within 1024 entries.
    let colliders = (largest(COLLIDER2D) + 16).clamp(16, 960);
    let layers = largest(TILEMAP_LAYER);
    let parallaxes = largest(PARALLAX2D);
    let shapes = largest(LINE2D) + largest(POLYGON2D) + largest(PARTICLES2D);
    // Parallax copies fill the screen once per repeating drawable.
    let sprites = (largest(SPRITE2D) + layers + shapes + parallaxes * 24 + 32).clamp(32, 1024);
    // Particles, line segments and polygon triangles, per scene.
    let sum = |scene: &crate::scene::Scene, measure: &dyn Fn(&ComponentInstance) -> usize| {
        scene
            .actors
            .iter()
            .flat_map(|a| a.components.iter())
            .map(measure)
            .sum::<usize>()
    };
    let amount = |c: &ComponentInstance| {
        if c.class.class_id.as_deref() == Some(PARTICLES2D) {
            usize::from(read::<Particles2D>(c).amount.max(1))
        } else {
            0
        }
    };
    let pieces = |c: &ComponentInstance| match c.class.class_id.as_deref() {
        Some(LINE2D) => read::<Line2D>(c).points.len() * 40,
        Some(POLYGON2D) => read::<Polygon2D>(c).points.len().saturating_sub(2) * 32,
        _ => 0,
    };
    let particles = scenes.iter().map(|s| sum(s, &amount)).max().unwrap_or(0)
        + templates.iter().map(|t| sum(t, &amount)).sum::<usize>() * 4;
    let shape_bytes = scenes.iter().map(|s| sum(s, &pieces)).max().unwrap_or(0)
        + templates.iter().map(|t| sum(t, &pieces)).sum::<usize>() * 4;
    let particles = particles.clamp(1, MAX_PARTICLES);
    // A textured quad fragment is 40 bytes, a rectangle 20 (16 for 8 and 16 pixel tiles)
    // and a page change 8. Tile layers draw at most a screen of cells each.
    let tiles: usize = scenes
        .iter()
        .chain(templates.iter().copied())
        .map(|scene| {
            let display = scene.display_size.map(usize::from);
            scene
                .actors
                .iter()
                .flat_map(|a| a.components.iter())
                .filter(|c| c.class.class_id.as_deref() == Some(TILEMAP_LAYER))
                .map(|c| {
                    let size = read::<TileMapLayer>(c)
                        .tile_set
                        .and_then(|id| scene.tilesets.get(&id))
                        .map_or([16, 16], |set| set.tile_size.map(|v| usize::from(v.max(1))));
                    (display[0] / size[0] + 2) * (display[1] / size[1] + 2)
                })
                .sum::<usize>()
        })
        .max()
        .unwrap_or(0);
    let arena = (sprites * 48
        + tiles * 20
        + layers * 16
        + shape_bytes
        + if particles > 1 { particles * 40 } else { 0 })
    .clamp(2048, 262144);
    let (probes, edits) = if layers > 0 { (48, 128) } else { (1, 1) };
    // The A* scratch covers the largest navigation grid of any map.
    let nav_cells = scenes
        .iter()
        .flat_map(|scene| {
            scene
                .actors
                .iter()
                .enumerate()
                .flat_map(move |(index, actor)| {
                    actor
                        .components
                        .iter()
                        .filter(|c| c.class.class_id.as_deref() == Some(NAVIGATION_REGION2D))
                        .filter_map(move |c| {
                            let r: crate::navigation2d::NavigationRegion2D = read(c);
                            (r.mode == crate::navigation2d::NavMode::Grid).then(|| {
                                crate::navigation2d::build_grid(
                                    &r,
                                    world_point(scene, index, [0., 0.]),
                                    scene,
                                    &scene.tilesets,
                                    Some(actor.id),
                                )
                                .map_or(0, |g| g.columns * g.rows)
                            })
                        })
                })
        })
        .max()
        .unwrap_or(0);
    let nav_open = if nav_cells > 0 {
        (nav_cells * 2).clamp(64, 4096)
    } else {
        1
    };
    format!(
        "#define EPOK_WORLD2D 1\n#define EPOK_WORLD2D_COLLIDERS {colliders}\n#define EPOK_WORLD2D_TILE_PROBES {probes}\n#define EPOK_WORLD2D_TILE_EDITS {edits}\n#define EPOK_WORLD2D_PARTICLES {particles}\n#define EPOK_WORLD2D_TRIGGER_PAIRS {}\n#define EPOK_WORLD2D_DRAW_ITEMS {sprites}\n#define EPOK_WORLD2D_ARENA_BYTES {arena}\n#define EPOK_WORLD2D_NAV_CELLS {}\n#define EPOK_WORLD2D_NAV_OPEN {nav_open}\n",
        (colliders * 2).clamp(32, 512),
        nav_cells.max(1)
    )
}

/// World position of a 2D actor through its spatial attachments (depth-bounded).
pub fn world_position(scene: &crate::scene::Scene, index: usize) -> [f32; 2] {
    world_point(scene, index, [0., 0.])
}
/// A point in a 2D actor's local pixels, in world pixels.
pub fn world_point(scene: &crate::scene::Scene, index: usize, local: [f32; 2]) -> [f32; 2] {
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
    // Compose root-first: p_world = parent(... child(p)).
    let mut point = local;
    for &i in &chain {
        let t = scene.actors[i]
            .root()
            .map(|c| crate::scene_view_mode::read_transform_2d(&c.properties))
            .unwrap_or_default();
        let (sin, cos) = t.rotation.to_radians().sin_cos();
        let scaled = [point[0] * t.scale[0], point[1] * t.scale[1]];
        point = [
            cos * scaled[0] - sin * scaled[1] + t.position[0],
            sin * scaled[0] + cos * scaled[1] + t.position[1],
        ];
    }
    point
}
/// The centre of the view the game starts on: the first current Camera2D's position after
/// its limits, plus its offset, or the origin without a camera. Parallax layers are
/// authored for this view.
pub fn start_view(scene: &crate::scene::Scene) -> [f32; 2] {
    let display = scene.display_size.map(f32::from);
    for (index, actor) in scene.actors.iter().enumerate() {
        if !actor.active {
            continue;
        }
        let Some(camera) = component(actor, CAMERA2D).map(read::<Camera2D>) else {
            continue;
        };
        if !camera.current {
            continue;
        }
        let mut centre = world_position(scene, index);
        let zoom = camera.zoom.clamp(0.0625, 64.);
        if camera.limits_enabled {
            for k in 0..2 {
                let half = display[k] * 0.5 / zoom;
                let (lo, hi) = (camera.limits[k] + half, camera.limits[k + 2] - half);
                centre[k] = if lo > hi {
                    (camera.limits[k] + camera.limits[k + 2]) * 0.5
                } else {
                    centre[k].clamp(lo, hi)
                };
            }
        }
        return [centre[0] + camera.offset[0], centre[1] + camera.offset[1]];
    }
    [0., 0.]
}

// ---- 2D node presets ---------------------------------------------------------------------

/// The ready-made 2D node types the editor offers. Each is an `Actor2D` with a set of
/// engine components; the user can add or remove components afterwards.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Preset {
    Node2D,
    Sprite2D,
    AnimatedSprite2D,
    TileMapLayer,
    Camera2D,
    StaticBody2D,
    Area2D,
    CharacterBody2D,
    Parallax2D,
    CanvasLayer2D,
    CanvasModulate2D,
    Path2D,
    PathFollow2D,
    Line2D,
    Polygon2D,
    Particles2D,
    PointLight2D,
    AudioPlayer2D,
    VisibleOnScreenNotifier2D,
    RemoteTransform2D,
    Skeleton2D,
    NavigationRegion2D,
    NavigationAgent2D,
}
impl Preset {
    pub const ALL: [Preset; 23] = [
        Preset::Node2D,
        Preset::Sprite2D,
        Preset::AnimatedSprite2D,
        Preset::Camera2D,
        Preset::CharacterBody2D,
        Preset::StaticBody2D,
        Preset::Area2D,
        Preset::TileMapLayer,
        Preset::Parallax2D,
        Preset::Path2D,
        Preset::PathFollow2D,
        Preset::Line2D,
        Preset::Polygon2D,
        Preset::Particles2D,
        Preset::PointLight2D,
        Preset::Skeleton2D,
        Preset::NavigationRegion2D,
        Preset::NavigationAgent2D,
        Preset::CanvasLayer2D,
        Preset::CanvasModulate2D,
        Preset::AudioPlayer2D,
        Preset::VisibleOnScreenNotifier2D,
        Preset::RemoteTransform2D,
    ];
    /// Menu groups, in the order of `ALL`.
    pub fn category(self) -> &'static str {
        match self {
            Preset::Node2D | Preset::Sprite2D | Preset::AnimatedSprite2D | Preset::Camera2D => {
                "Basics"
            }
            Preset::CharacterBody2D | Preset::StaticBody2D | Preset::Area2D => "Physics",
            Preset::TileMapLayer | Preset::Parallax2D | Preset::Path2D | Preset::PathFollow2D => {
                "Level"
            }
            Preset::Line2D | Preset::Polygon2D | Preset::Particles2D | Preset::PointLight2D => {
                "Drawing and effects"
            }
            Preset::Skeleton2D => "Animation",
            Preset::NavigationRegion2D | Preset::NavigationAgent2D => "Navigation",
            Preset::CanvasLayer2D | Preset::CanvasModulate2D => "Canvas",
            Preset::AudioPlayer2D
            | Preset::VisibleOnScreenNotifier2D
            | Preset::RemoteTransform2D => "Audio and helpers",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Preset::Node2D => "Node2D",
            Preset::Sprite2D => "Sprite2D",
            Preset::AnimatedSprite2D => "AnimatedSprite2D",
            Preset::TileMapLayer => "TileMapLayer",
            Preset::Camera2D => "Camera2D",
            Preset::StaticBody2D => "StaticBody2D",
            Preset::Area2D => "Area2D",
            Preset::CharacterBody2D => "CharacterBody2D",
            Preset::Parallax2D => "Parallax2D",
            Preset::CanvasLayer2D => "CanvasLayer2D",
            Preset::CanvasModulate2D => "CanvasModulate2D",
            Preset::Path2D => "Path2D",
            Preset::PathFollow2D => "PathFollow2D",
            Preset::Line2D => "Line2D",
            Preset::Polygon2D => "Polygon2D",
            Preset::Particles2D => "Particles2D",
            Preset::PointLight2D => "PointLight2D",
            Preset::AudioPlayer2D => "AudioPlayer2D",
            Preset::VisibleOnScreenNotifier2D => "VisibleOnScreenNotifier2D",
            Preset::RemoteTransform2D => "RemoteTransform2D",
            Preset::Skeleton2D => "Skeleton2D",
            Preset::NavigationRegion2D => "NavigationRegion2D",
            Preset::NavigationAgent2D => "NavigationAgent2D",
        }
    }
    pub fn description(self) -> &'static str {
        match self {
            Preset::Node2D => "An empty 2D actor: a transform to group or position others.",
            Preset::Sprite2D => "Draws a texture or one cell of a sprite sheet.",
            Preset::AnimatedSprite2D => "Plays named frame animations from a sprite sheet.",
            Preset::TileMapLayer => "A grid of tiles painted from a TileSet, with tile collision.",
            Preset::Camera2D => "Follows its actor; smoothing, limits and drag margins.",
            Preset::StaticBody2D => "A solid box that blocks characters: floors, walls, platforms.",
            Preset::Area2D => "A trigger region that reports Enter / Stay / Exit overlaps.",
            Preset::CharacterBody2D => "A player or enemy moved with move_and_slide().",
            Preset::Parallax2D => {
                "Scrolls attached nodes slower or faster than the camera; can repeat."
            }
            Preset::CanvasLayer2D => "Draws attached nodes on their own layer, in screen pixels.",
            Preset::CanvasModulate2D => "Tints everything on its canvas: night, flashes, fades.",
            Preset::Path2D => "A curve to move things along: patrols, moving platforms, rails.",
            Preset::PathFollow2D => {
                "Moves its actor along a Path2D; carries bodies standing on it."
            }
            Preset::Line2D => "A polyline with width and a colour gradient: ropes, lasers, trails.",
            Preset::Polygon2D => "A filled, optionally textured shape: terrain, shadows, decals.",
            Preset::Particles2D => "Sparks, smoke, dust and rain from a CPU particle emitter.",
            Preset::PointLight2D => "A soft additive glow: lamps, torches, magic.",
            Preset::AudioPlayer2D => "Plays a sound that fades and pans with the camera.",
            Preset::VisibleOnScreenNotifier2D => {
                "Reports entering and leaving the view; can pause its node."
            }
            Preset::RemoteTransform2D => "Pushes this node's transform onto another node.",
            Preset::Skeleton2D => {
                "Animates a cut-out character: attached parts are bones; pose keys and IK."
            }
            Preset::NavigationRegion2D => {
                "Where agents can walk: a grid from tiles and static bodies, or a navmesh."
            }
            Preset::NavigationAgent2D => {
                "Finds a path to a target and walks it: enemies, NPCs, followers."
            }
        }
    }
    pub fn components(self) -> &'static [&'static str] {
        match self {
            Preset::Node2D => &[],
            Preset::Sprite2D | Preset::AnimatedSprite2D | Preset::PointLight2D => &[SPRITE2D],
            Preset::TileMapLayer => &[TILEMAP_LAYER],
            Preset::Camera2D => &[CAMERA2D],
            Preset::StaticBody2D => &[COLLIDER2D],
            Preset::Area2D => &[COLLIDER2D],
            Preset::CharacterBody2D => &[SPRITE2D, COLLIDER2D, CHARACTER_BODY2D],
            Preset::Parallax2D => &[PARALLAX2D],
            Preset::CanvasLayer2D => &[CANVAS_LAYER2D],
            Preset::CanvasModulate2D => &[CANVAS_MODULATE2D],
            Preset::Path2D => &[PATH2D],
            Preset::PathFollow2D => &[PATH_FOLLOW2D],
            Preset::Line2D => &[LINE2D],
            Preset::Polygon2D => &[POLYGON2D],
            Preset::Particles2D => &[PARTICLES2D],
            Preset::AudioPlayer2D => &[AUDIO_PLAYER2D],
            Preset::VisibleOnScreenNotifier2D => &[NOTIFIER2D],
            Preset::RemoteTransform2D => &[REMOTE_TRANSFORM2D],
            Preset::Skeleton2D => &[SKELETON2D],
            Preset::NavigationRegion2D => &[NAVIGATION_REGION2D],
            Preset::NavigationAgent2D => &[NAVIGATION_AGENT2D],
        }
    }
}
/// A new Actor2D for `preset` at `position`, with its root transform and components.
pub fn create(preset: Preset, name: &str, position: [f32; 2]) -> ActorInstance {
    let mut actor = ActorInstance::new(
        Uuid::new_v4(),
        ClassReference::new("epok::Actor2D", om::ACTOR2D_ID),
        name,
    );
    let mut root = ComponentInstance::new(
        Uuid::new_v4(),
        ClassReference::new("epok::SceneComponent2D", om::SCENE_COMPONENT2D_ID),
        "Transform 2D",
    );
    root.root = true;
    for (key, value) in [
        ("position", serde_json::json!([position[0], position[1]])),
        ("rotation", serde_json::json!(0.)),
        ("scale", serde_json::json!([1., 1.])),
    ] {
        root.properties.insert(key.into(), value);
        root.overrides.insert(key.into());
    }
    actor.components.push(root);
    for id in preset.components() {
        if let Some(mut component) = new_component(&actor, id) {
            match (preset, *id) {
                (Preset::Area2D, COLLIDER2D) => {
                    let mut c = Collider2D {
                        trigger: true,
                        size: [32., 32.],
                        ..Default::default()
                    };
                    c.mask = u32::MAX;
                    write(&mut component, &c);
                }
                (Preset::StaticBody2D, COLLIDER2D) => write(
                    &mut component,
                    &Collider2D {
                        size: [64., 16.],
                        ..Default::default()
                    },
                ),
                (Preset::CharacterBody2D, COLLIDER2D) => write(
                    &mut component,
                    &Collider2D {
                        size: [14., 24.],
                        ..Default::default()
                    },
                ),
                // A glow adds light; the editor gives it the generated light texture.
                (Preset::PointLight2D, SPRITE2D) => write(
                    &mut component,
                    &Sprite2D {
                        blend: BlendMode::Add,
                        modulate: [200, 180, 140],
                        ..Default::default()
                    },
                ),
                _ => {}
            }
            actor.components.push(component);
        }
    }
    actor.refresh_components();
    actor
}

// ---- cooking ---------------------------------------------------------------------------

fn fixed(value: f32) -> String {
    let raw = (f64::from(value) * 4096.)
        .round()
        .clamp(f64::from(i32::MIN), f64::from(i32::MAX));
    format!("epok::Fixed({},epok::Fixed::RAW)", raw as i32)
}
fn blend(mode: BlendMode) -> &'static str {
    match mode {
        BlendMode::Cutout => "epok::BlendMode::Cutout",
        BlendMode::Average => "epok::BlendMode::Average",
        BlendMode::Add => "epok::BlendMode::Add",
        BlendMode::Subtract => "epok::BlendMode::Subtract",
        BlendMode::AddQuarter => "epok::BlendMode::AddQuarter",
    }
}
/// Output of cooking one actor's 2D components: setter statements for the actor's
/// generated `actor_apply_N`, and namespace-scope constants (frame clips, tile tables).
#[derive(Default)]
pub struct Cooked {
    pub statics: String,
    pub setters: String,
}
/// What cooking needs from the bank: the textures it can address, the TileSets its tile
/// layers use, the TileSets whose tables it already emitted, and the starting view.
pub struct CookContext<'a> {
    pub textures: &'a dyn Fn(Uuid) -> bool,
    pub tilesets: &'a BTreeMap<Uuid, Arc<crate::tileset::TileSet>>,
    pub emitted: BTreeSet<Uuid>,
    pub start_view: [f32; 2],
    /// The actor table slot of a map actor (`actors[slot]` in the generated setters).
    pub actor_slot: Option<&'a dyn Fn(Uuid) -> Option<usize>>,
    /// The bank index of an audio clip.
    pub clip: Option<&'a dyn Fn(Uuid) -> Option<usize>>,
    /// The map being cooked, for components that read other actors (navigation).
    pub scene: Option<&'a crate::scene::Scene>,
}
impl<'a> CookContext<'a> {
    pub fn new(
        textures: &'a dyn Fn(Uuid) -> bool,
        tilesets: &'a BTreeMap<Uuid, Arc<crate::tileset::TileSet>>,
        start_view: [f32; 2],
    ) -> Self {
        Self {
            textures,
            tilesets,
            emitted: BTreeSet::new(),
            start_view,
            actor_slot: None,
            clip: None,
            scene: None,
        }
    }
    pub fn with_scene(mut self, scene: &'a crate::scene::Scene) -> Self {
        self.scene = Some(scene);
        self
    }
    pub fn with_actors(mut self, slot: &'a dyn Fn(Uuid) -> Option<usize>) -> Self {
        self.actor_slot = Some(slot);
        self
    }
    pub fn with_clips(mut self, clip: &'a dyn Fn(Uuid) -> Option<usize>) -> Self {
        self.clip = Some(clip);
        self
    }
    fn actor(&self, id: Option<Uuid>) -> Option<usize> {
        self.actor_slot.and_then(|slot| id.and_then(slot))
    }
}
fn fixed_list(values: impl Iterator<Item = f32>) -> String {
    values.map(fixed).collect::<Vec<_>>().join(",")
}
fn colour(field: &str, value: [u8; 3]) -> String {
    format!(
        "c->{field}[0]={};c->{field}[1]={};c->{field}[2]={};",
        value[0], value[1], value[2]
    )
}
fn tileset_symbol(id: Uuid) -> String {
    format!("w2d_tileset_{}", id.simple())
}
/// The constant tables of one TileSet (tile infos, custom data, the TileSetData).
fn cook_tileset(id: Uuid, set: &crate::tileset::TileSet) -> String {
    use crate::tileset::Collision;
    let name = tileset_symbol(id);
    let mut out = String::new();
    let tiles = set
        .tiles
        .iter()
        .map(|t| {
            let texel = set.texel([u16::from(t.atlas[0]), u16::from(t.atlas[1])]);
            let collision = match t.collision {
                Collision::None => "None",
                Collision::Full => "Full",
                Collision::Rect => "Rect",
            };
            let seconds = if t.frames > 1 && t.fps > 0. {
                1. / t.fps
            } else {
                0.
            };
            format!(
                "{{{{{},{}}},{},{},epok::TileCollision::{collision},{},{{{},{},{},{}}},{},{},{{{},{}}},{}}}",
                t.atlas[0],
                t.atlas[1],
                texel[0].min(65535),
                texel[1].min(65535),
                u8::from(t.one_way),
                t.rect[0],
                t.rect[1],
                t.rect[2],
                t.rect[3],
                t.frames.max(1),
                t.columns,
                set.tile_size[0] + set.separation[0],
                set.tile_size[1] + set.separation[1],
                fixed(seconds)
            )
        })
        .collect::<Vec<_>>();
    if !tiles.is_empty() {
        out += &format!(
            "inline constexpr epok::TileInfo {name}_tiles[]={{{}}};\n",
            tiles.join(",")
        );
    }
    let layers = set.custom_layers.len();
    if layers > 0 && !set.tiles.is_empty() {
        out += &format!(
            "inline constexpr int32_t {name}_custom[]={{{}}};\n",
            set.tiles
                .iter()
                .flat_map(|t| (0..layers).map(|k| t.custom.get(k).copied().unwrap_or(0)))
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );
    }
    out += &format!(
        "inline constexpr epok::TileSetData {name}={{{},{},{},{},{},{}}};\n",
        set.tile_size[0],
        set.tile_size[1],
        set.tiles.len(),
        if tiles.is_empty() {
            "nullptr".to_string()
        } else {
            format!("{name}_tiles")
        },
        if layers > 0 && !tiles.is_empty() {
            layers
        } else {
            0
        },
        if layers > 0 && !tiles.is_empty() {
            format!("{name}_custom")
        } else {
            "nullptr".into()
        }
    );
    out
}
/// Cooks the 2D component at `position` in its actor. `symbol` names the unique prefix
/// for its constants.
pub fn cook(
    component: &ComponentInstance,
    position: usize,
    symbol: &str,
    context: &mut CookContext,
    label: &str,
) -> Result<Cooked, String> {
    let textures = context.textures;
    let id = component.class.class_id.as_deref().unwrap_or("");
    let mut out = Cooked::default();
    let mut body = String::new();
    let class = class_name(id).ok_or_else(|| format!("{label}: not a World2D component"))?;
    match id {
        SPRITE2D => {
            let s: Sprite2D = read(component);
            match s.texture {
                Some(texture) if textures(texture) => {
                    body += &format!("c->texture={};", crate::texture::symbol(texture))
                }
                Some(texture) => {
                    return Err(format!(
                        "{label}: texture {texture} is missing from the bank's resources"
                    ));
                }
                None => body += "c->texture=-1;",
            }
            body += &format!(
                "c->region[0]={};c->region[1]={};c->region[2]={};c->region[3]={};c->hframes={};c->vframes={};c->frame={};",
                s.region[0],
                s.region[1],
                s.region[2],
                s.region[3],
                s.hframes.max(1),
                s.vframes.max(1),
                s.frame
            );
            body += &format!(
                "c->offset[0]={};c->offset[1]={};c->modulate[0]={};c->modulate[1]={};c->modulate[2]={};c->blend={};c->z_index={};",
                fixed(s.offset[0]),
                fixed(s.offset[1]),
                s.modulate[0],
                s.modulate[1],
                s.modulate[2],
                blend(s.blend),
                s.z_index.clamp(-4096, 4096)
            );
            body += &format!(
                "c->centered={};c->flip_h={};c->flip_v={};c->visible={};c->y_sort={};c->speed_scale={};",
                s.centered,
                s.flip_h,
                s.flip_v,
                s.visible,
                s.y_sort,
                fixed(s.speed_scale.max(0.))
            );
            let clips: Vec<_> = s
                .animations
                .iter()
                .filter(|a| !a.frames.is_empty())
                .collect();
            if s.animations.len() > 64 {
                return Err(format!("{label}: at most 64 animations per sprite"));
            }
            if !clips.is_empty() {
                let mut rows = Vec::new();
                for (k, animation) in s.animations.iter().enumerate() {
                    if animation.frames.len() > 256 {
                        return Err(format!(
                            "{label}: animation {} has more than 256 frames",
                            animation.name
                        ));
                    }
                    let frames = if animation.frames.is_empty() {
                        "nullptr".to_string()
                    } else {
                        let name = format!("{symbol}_frames_{k}");
                        out.statics += &format!(
                            "inline constexpr epok::SpriteFrame {name}[]={{{}}};\n",
                            animation
                                .frames
                                .iter()
                                .map(|f| format!(
                                    "{{{{{},{},{},{}}},{},{}}}",
                                    f.region[0],
                                    f.region[1],
                                    f.region[2],
                                    f.region[3],
                                    fixed(animation.frame_seconds(f)),
                                    f.event
                                ))
                                .collect::<Vec<_>>()
                                .join(",")
                        );
                        name
                    };
                    rows.push(format!(
                        "{{{frames},{},{},{}}}",
                        animation.frames.len(),
                        animation.looping,
                        serde_json::to_string(&animation.name).unwrap()
                    ));
                }
                out.statics += &format!(
                    "inline constexpr epok::SpriteClip {symbol}_clips[]={{{}}};\n",
                    rows.join(",")
                );
                body += &format!(
                    "c->clips={symbol}_clips;c->clip_count={};c->animation={};c->frame=0;",
                    s.animations.len(),
                    usize::from(s.animation).min(s.animations.len() - 1)
                );
                if s.autoplay {
                    body += &format!(
                        "c->play({}u);",
                        usize::from(s.animation).min(s.animations.len() - 1)
                    );
                }
            }
        }
        CAMERA2D => {
            let c: Camera2D = read(component);
            body += &format!(
                "c->current={};c->zoom={};c->offset[0]={};c->offset[1]={};c->rotating={};c->smoothing={};c->smoothing_speed={};",
                c.current,
                fixed(c.zoom.clamp(0.0625, 64.)),
                fixed(c.offset[0]),
                fixed(c.offset[1]),
                c.rotating,
                c.smoothing,
                fixed(c.smoothing_speed.max(0.))
            );
            body += &format!(
                "c->limits_enabled={};c->drag_enabled={};",
                c.limits_enabled, c.drag_enabled
            );
            for k in 0..4 {
                body += &format!(
                    "c->limits[{k}]={};c->drag_margins[{k}]={};",
                    fixed(c.limits[k].clamp(-65536., 65536.)),
                    fixed(c.drag_margins[k].clamp(0., 1.))
                );
            }
            body += &format!(
                "c->background[0]={};c->background[1]={};c->background[2]={};c->reset_smoothing();",
                c.background[0], c.background[1], c.background[2]
            );
        }
        COLLIDER2D => {
            let c: Collider2D = read(component);
            body += &format!(
                "c->shape.shape={};c->shape.half_extents[0]={};c->shape.half_extents[1]={};c->shape.radius={};c->shape.offset[0]={};c->shape.offset[1]={};",
                if c.shape == Shape2D::Circle {
                    "epok::Collider2D::Circle"
                } else {
                    "epok::Collider2D::Box"
                },
                fixed(c.size[0].abs() * 0.5),
                fixed(c.size[1].abs() * 0.5),
                fixed(c.radius.abs()),
                fixed(c.offset[0]),
                fixed(c.offset[1])
            );
            body += &format!(
                "c->shape.layer={}u;c->shape.layer_mask={}u;c->shape.trigger={};c->shape.one_way={};c->shape.one_way_margin={};c->shape.enabled={};",
                c.layer,
                c.mask,
                c.trigger,
                c.one_way,
                fixed(c.one_way_margin.max(0.)),
                c.enabled
            );
        }
        TILEMAP_LAYER => {
            let layer: TileMapLayer = read(component);
            body += &format!(
                "c->modulate[0]={};c->modulate[1]={};c->modulate[2]={};c->blend={};c->z_index={};c->visible={};c->collision_enabled={};c->collision_layer={}u;",
                layer.modulate[0],
                layer.modulate[1],
                layer.modulate[2],
                blend(layer.blend),
                layer.z_index.clamp(-4096, 4096),
                layer.visible,
                layer.collision_enabled,
                layer.collision_layer
            );
            match layer.tile_set {
                None => body += "c->tileset=nullptr;c->cells=nullptr;c->grid=nullptr;",
                Some(id) => {
                    let set = context.tilesets.get(&id).ok_or_else(|| {
                        format!("{label}: TileSet {id} is missing; restore it or pick another")
                    })?;
                    let texture = set
                        .texture
                        .ok_or_else(|| format!("{label}: its TileSet has no texture"))?;
                    if !textures(texture) {
                        return Err(format!(
                            "{label}: texture {texture} is missing from the bank's resources"
                        ));
                    }
                    if context.emitted.insert(id) {
                        out.statics += &cook_tileset(id, set);
                    }
                    let cells = crate::tilemap::decode(&layer.cells)
                        .map_err(|e| format!("{label}: {e}"))?;
                    let cooked =
                        crate::tilemap::cook(&cells, set).map_err(|e| format!("{label}: {e}"))?;
                    body += &format!(
                        "c->tileset=&{};c->texture={};",
                        tileset_symbol(id),
                        crate::texture::symbol(texture)
                    );
                    if cooked.cells.is_empty() {
                        body += "c->cells=nullptr;c->grid=nullptr;";
                    } else {
                        out.statics += &format!(
                            "inline constexpr uint16_t {symbol}_cells[]={{{}}};\ninline constexpr int16_t {symbol}_grid[]={{{}}};\n",
                            cooked
                                .cells
                                .iter()
                                .map(|v| v.to_string())
                                .collect::<Vec<_>>()
                                .join(","),
                            cooked
                                .grid
                                .iter()
                                .map(|v| v.to_string())
                                .collect::<Vec<_>>()
                                .join(",")
                        );
                        body += &format!(
                            "c->cells={symbol}_cells;c->grid={symbol}_grid;c->grid_origin[0]={};c->grid_origin[1]={};c->grid_size[0]={};c->grid_size[1]={};",
                            cooked.grid_origin[0],
                            cooked.grid_origin[1],
                            cooked.grid_size[0],
                            cooked.grid_size[1]
                        );
                    }
                }
            }
        }
        PARALLAX2D => {
            let p: Parallax2D = read(component);
            for k in 0..2 {
                body += &format!(
                    "c->scroll_scale[{k}]={};c->repeat_size[{k}]={};c->autoscroll[{k}]={};c->reference[{k}]={};",
                    fixed(p.scroll_scale[k].clamp(-16., 16.)),
                    fixed(p.repeat_size[k].clamp(0., 65536.)),
                    fixed(p.autoscroll[k].clamp(-4096., 4096.)),
                    fixed(context.start_view[k])
                );
            }
        }
        CANVAS_LAYER2D => {
            let c: CanvasLayer2D = read(component);
            body += &format!(
                "c->layer={};c->offset[0]={};c->offset[1]={};c->follow_viewport={};c->visible={};",
                c.layer.clamp(-64, 64),
                fixed(c.offset[0]),
                fixed(c.offset[1]),
                c.follow_viewport,
                c.visible
            );
        }
        CANVAS_MODULATE2D => {
            let m: CanvasModulate2D = read(component);
            body += &format!(
                "c->color[0]={};c->color[1]={};c->color[2]={};",
                m.color[0], m.color[1], m.color[2]
            );
        }
        PATH2D => {
            let path: Path2D = read(component);
            let points = crate::shape2d::sample_path(&path);
            if points.len() > 2048 {
                return Err(format!(
                    "{label}: the path samples to more than 2048 points"
                ));
            }
            let lengths = crate::shape2d::cumulative_lengths(&points);
            if points.is_empty() {
                body += "c->points=nullptr;c->lengths=nullptr;c->point_count=0;";
            } else {
                out.statics += &format!(
                    "inline constexpr epok::Fixed {symbol}_points[]={{{}}};\ninline constexpr epok::Fixed {symbol}_lengths[]={{{}}};\n",
                    fixed_list(points.iter().flat_map(|p| [p[0], p[1]])),
                    fixed_list(lengths.iter().copied())
                );
                body += &format!(
                    "c->points={symbol}_points;c->lengths={symbol}_lengths;c->point_count={};",
                    points.len()
                );
            }
            body += &format!("c->closed={};", path.closed && path.points.len() > 2);
        }
        PATH_FOLLOW2D => {
            let f: PathFollow2D = read(component);
            if let Some(slot) = context.actor(f.path) {
                body += &format!("c->path=actors[{slot}];");
            }
            body += &format!(
                "c->progress={};c->speed={};c->offset[0]={};c->offset[1]={};c->looping={};c->ping_pong={};c->rotates={};c->moving={};",
                fixed(f.progress),
                fixed(f.speed.clamp(-8192., 8192.)),
                fixed(f.offset[0]),
                fixed(f.offset[1]),
                f.looping,
                f.ping_pong,
                f.rotates,
                f.moving
            );
        }
        LINE2D => {
            let l: Line2D = read(component);
            if l.points.len() > LINE_MAX_POINTS {
                return Err(format!(
                    "{label}: a Line2D holds at most {LINE_MAX_POINTS} points"
                ));
            }
            for (i, p) in l.points.iter().enumerate() {
                body += &format!(
                    "c->points[{i}][0]={};c->points[{i}][1]={};",
                    fixed(p[0]),
                    fixed(p[1])
                );
            }
            body += &format!(
                "c->point_count={};c->width={};c->closed={};c->blend={};c->z_index={};c->visible={};",
                l.points.len(),
                fixed(l.width.clamp(0., 256.)),
                l.closed && l.points.len() > 2,
                blend(l.blend),
                l.z_index.clamp(-4096, 4096),
                l.visible
            );
            body += &colour("begin_color", l.begin_color);
            body += &colour("end_color", l.end_color);
        }
        POLYGON2D => {
            let poly: Polygon2D = read(component);
            let triangles =
                crate::shape2d::triangulate(&poly.points).map_err(|e| format!("{label}: {e}"))?;
            out.statics += &format!(
                "inline constexpr epok::Fixed {symbol}_vertices[]={{{}}};\ninline constexpr uint16_t {symbol}_triangles[]={{{}}};\n",
                fixed_list(poly.points.iter().flat_map(|p| [p[0], p[1]])),
                triangles
                    .iter()
                    .flatten()
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            );
            body += &format!(
                "c->vertices={symbol}_vertices;c->triangles={symbol}_triangles;c->vertex_count={};c->triangle_count={};c->blend={};c->z_index={};c->visible={};",
                poly.points.len(),
                triangles.len(),
                blend(poly.blend),
                poly.z_index.clamp(-4096, 4096),
                poly.visible
            );
            body += &colour("color", poly.color);
            match poly.texture {
                Some(texture) if textures(texture) => {
                    out.statics += &format!(
                        "inline constexpr uint16_t {symbol}_texels[]={{{}}};\n",
                        poly.points
                            .iter()
                            .flat_map(|p| {
                                [0, 1].map(|k| {
                                    ((p[k] + poly.texture_offset[k]).round().clamp(0., 255.) as u16)
                                        .to_string()
                                })
                            })
                            .collect::<Vec<_>>()
                            .join(",")
                    );
                    body += &format!(
                        "c->texels={symbol}_texels;c->texture={};",
                        crate::texture::symbol(texture)
                    );
                }
                Some(texture) => {
                    return Err(format!(
                        "{label}: texture {texture} is missing from the bank's resources"
                    ));
                }
                None => body += "c->texels=nullptr;c->texture=-1;",
            }
        }
        PARTICLES2D => {
            let p: Particles2D = read(component);
            let texture = match p.texture {
                Some(texture) if textures(texture) => crate::texture::symbol(texture),
                Some(texture) => {
                    return Err(format!(
                        "{label}: texture {texture} is missing from the bank's resources"
                    ));
                }
                None => "-1".into(),
            };
            body += &format!(
                "c->emitting={};c->one_shot={};c->local_coords={};c->visible={};c->amount={};c->lifetime={};c->explosiveness={};",
                p.emitting,
                p.one_shot,
                p.local_coords,
                p.visible,
                p.amount.clamp(1, MAX_PARTICLES as u16),
                fixed(p.lifetime.clamp(0.05, 60.)),
                fixed(p.explosiveness.clamp(0., 1.))
            );
            body += &format!(
                "c->direction={};c->spread={};c->speed[0]={};c->speed[1]={};c->gravity[0]={};c->gravity[1]={};c->damping={};c->scale[0]={};c->scale[1]={};",
                fixed(p.direction),
                fixed(p.spread.clamp(0., 180.)),
                fixed(p.speed[0]),
                fixed(p.speed[1]),
                fixed(p.gravity[0]),
                fixed(p.gravity[1]),
                fixed(p.damping.max(0.)),
                fixed(p.scale[0].clamp(0., 16.)),
                fixed(p.scale[1].clamp(0., 16.))
            );
            body += &colour("start_color", p.start_color);
            body += &colour("end_color", p.end_color);
            body += &format!(
                "c->shape=epok::EmissionShape2D::{};c->extents[0]={};c->extents[1]={};c->texture={texture};c->region[0]={};c->region[1]={};c->region[2]={};c->region[3]={};c->hframes={};c->blend={};c->z_index={};c->seed={}u;",
                match p.shape {
                    EmissionShape::Point => "Point",
                    EmissionShape::Rectangle => "Rectangle",
                    EmissionShape::Circle => "Circle",
                },
                fixed(p.extents[0].abs()),
                fixed(p.extents[1].abs()),
                p.region[0],
                p.region[1],
                p.region[2],
                p.region[3],
                p.hframes.max(1),
                blend(p.blend),
                p.z_index.clamp(-4096, 4096),
                p.seed.max(1)
            );
        }
        AUDIO_PLAYER2D => {
            let a: AudioPlayer2D = read(component);
            let clip = match a.audio.clip {
                Some(id) => context
                    .clip
                    .and_then(|clip| clip(id))
                    .map(|i| i as i32)
                    .ok_or_else(|| {
                        format!("{label}: audio clip {id} is missing from the resource bank")
                    })?,
                None => -1,
            };
            body += &format!(
                "c->source.enabled={};c->source.clip={clip};c->source.pitch={};c->source.priority={};c->volume={};c->autoplay={};c->max_distance={};c->attenuation={};c->panning={};",
                clip >= 0,
                fixed(a.audio.pitch.clamp(0.25, 4.)),
                a.audio.priority,
                fixed(a.audio.volume.clamp(0., 1.)),
                a.audio.play_on_start,
                fixed(a.max_distance.max(0.)),
                fixed(a.attenuation.clamp(1., 4.)),
                fixed(a.panning.clamp(0., 1.))
            );
        }
        NOTIFIER2D => {
            let n: Notifier2D = read(component);
            for k in 0..4 {
                body += &format!("c->rect[{k}]={};", fixed(n.rect[k]));
            }
            body += &format!("c->disable_when_hidden={};", n.disable_when_hidden);
        }
        NAVIGATION_REGION2D => {
            let r: crate::navigation2d::NavigationRegion2D = read(component);
            let scene = context
                .scene
                .ok_or_else(|| format!("{label}: navigation needs its map to cook"))?;
            let index = scene
                .actors
                .iter()
                .position(|a| a.components.iter().any(|c| c.id == component.id))
                .ok_or_else(|| format!("{label}: the region is not in its map"))?;
            match r.mode {
                crate::navigation2d::NavMode::Grid => {
                    let grid = crate::navigation2d::build_grid(
                        &r,
                        world_point(scene, index, [0., 0.]),
                        scene,
                        context.tilesets,
                        Some(scene.actors[index].id),
                    )
                    .map_err(|e| format!("{label}: {e}"))?;
                    let mut bits = vec![0u8; grid.blocked.len().div_ceil(8)];
                    for (i, blocked) in grid.blocked.iter().enumerate() {
                        if *blocked {
                            bits[i / 8] |= 1 << (i % 8);
                        }
                    }
                    out.statics += &format!(
                        "inline constexpr uint8_t {symbol}_blocked[]={{{}}};\n",
                        bits.iter().map(u8::to_string).collect::<Vec<_>>().join(",")
                    );
                    body += &format!(
                        "c->is_mesh=false;c->grid.blocked={symbol}_blocked;c->grid.columns={};c->grid.rows={};c->grid.origin[0]={};c->grid.origin[1]={};c->grid.cell[0]={};c->grid.cell[1]={};c->grid.diagonal={};",
                        grid.columns,
                        grid.rows,
                        fixed(grid.origin[0]),
                        fixed(grid.origin[1]),
                        fixed(grid.cell[0]),
                        fixed(grid.cell[1]),
                        grid.diagonal
                    );
                }
                crate::navigation2d::NavMode::Polygon => {
                    let outline: Vec<[f32; 2]> = r
                        .outline
                        .iter()
                        .map(|p| world_point(scene, index, *p))
                        .collect();
                    let mesh = crate::navigation2d::build_mesh(&outline)
                        .map_err(|e| format!("{label}: {e}"))?;
                    out.statics += &format!(
                        "inline constexpr epok::Fixed {symbol}_vertices[]={{{}}};\ninline constexpr uint8_t {symbol}_triangles[]={{{}}};\ninline constexpr int8_t {symbol}_neighbours[]={{{}}};\n",
                        fixed_list(mesh.vertices.iter().flat_map(|v| *v)),
                        mesh.triangles
                            .iter()
                            .flat_map(|t| *t)
                            .map(|v| v.to_string())
                            .collect::<Vec<_>>()
                            .join(","),
                        mesh.neighbours
                            .iter()
                            .flat_map(|n| *n)
                            .map(|v| v.to_string())
                            .collect::<Vec<_>>()
                            .join(",")
                    );
                    body += &format!(
                        "c->is_mesh=true;c->mesh.vertices={symbol}_vertices;c->mesh.triangles={symbol}_triangles;c->mesh.neighbours={symbol}_neighbours;c->mesh.triangle_count={};",
                        mesh.triangles.len()
                    );
                }
            }
        }
        NAVIGATION_AGENT2D => {
            let a: crate::navigation2d::NavigationAgent2D = read(component);
            if let Some(slot) = context.actor(a.target) {
                body += &format!(
                    "c->target_actor=actors[{slot}];c->has_target=true;c->dirty=true;c->finished=false;"
                );
            }
            body += &format!(
                "c->max_speed={};c->arrive_distance={};c->repath_interval={};c->drives_body={};",
                fixed(a.max_speed.clamp(0., 4096.)),
                fixed(a.arrive_distance.clamp(0.5, 256.)),
                fixed(a.repath_interval.clamp(0., 60.)),
                a.drives_body
            );
        }
        SKELETON2D => {
            let s: crate::skeleton2d::Skeleton2D = read(component);
            let (statics, setters) =
                crate::skeleton2d::cook(&s, symbol, &|id| context.actor(id), &fixed, label)?;
            out.statics += &statics;
            body += &setters;
        }
        REMOTE_TRANSFORM2D => {
            let r: RemoteTransform2D = read(component);
            if let Some(slot) = context.actor(r.target) {
                body += &format!("c->target=actors[{slot}];");
            }
            body += &format!(
                "c->update_position={};c->update_rotation={};c->update_scale={};",
                r.update_position, r.update_rotation, r.update_scale
            );
        }
        _ => {
            let b: CharacterBody2D = read(component);
            body += &format!(
                "c->up_direction[0]={};c->up_direction[1]={};c->floor_max_angle={};c->floor_snap_length={};c->collision_mask={}u;c->floating={};",
                fixed(b.up_direction[0]),
                fixed(b.up_direction[1]),
                fixed(b.floor_max_angle.clamp(0., 180.)),
                fixed(b.floor_snap_length.max(0.)),
                b.collision_mask,
                b.floating
            );
        }
    }
    out.setters =
        format!("if(auto* c=registry.resolve<{class}>(components[{position}])){{{body}}}\n");
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn set_record<T: Serialize>(actor: &mut ActorInstance, class: &str, value: &T) {
        let c = actor
            .components
            .iter_mut()
            .find(|c| c.class.class_id.as_deref() == Some(class))
            .unwrap();
        write(c, value);
    }
    #[test]
    fn presets_build_actor2d_documents_that_round_trip() {
        for preset in Preset::ALL {
            let actor = create(preset, preset.label(), [12., -4.]);
            assert_eq!(actor.class.class_id.as_deref(), Some(om::ACTOR2D_ID));
            let root = actor.root().unwrap();
            assert_eq!(
                root.class.class_id.as_deref(),
                Some(om::SCENE_COMPONENT2D_ID)
            );
            assert_eq!(
                actor.components.len(),
                1 + preset.components().len(),
                "{preset:?}"
            );
            let text = serde_yaml_ng::to_string(&actor).unwrap();
            let back: ActorInstance = serde_yaml_ng::from_str(&text).unwrap();
            assert_eq!(back.components.len(), actor.components.len());
            for component in &back.components {
                validate(component).unwrap();
            }
        }
        let area = create(Preset::Area2D, "Area", [0.; 2]);
        let collider: Collider2D = read(component(&area, COLLIDER2D).unwrap());
        assert!(collider.trigger);
    }
    #[test]
    fn sprite_source_rect_follows_animation_then_grid_then_region() {
        let mut sprite = Sprite2D {
            hframes: 4,
            vframes: 2,
            frame: 5,
            ..Default::default()
        };
        assert_eq!(sprite.source_rect([64, 32]), Some([16, 16, 16, 16]));
        sprite.hframes = 1;
        sprite.vframes = 1;
        sprite.region = [8, 0, 0, 0];
        assert_eq!(sprite.source_rect([64, 32]), Some([8, 0, 56, 32]));
        sprite.animations.push(Animation2D {
            frames: vec![Frame2D {
                region: [0, 16, 8, 8],
                ..Default::default()
            }],
            ..Default::default()
        });
        assert_eq!(sprite.source_rect([64, 32]), Some([0, 16, 8, 8]));
        sprite.animations[0].frames[0].region = [60, 0, 8, 8];
        assert_eq!(sprite.source_rect([64, 32]), None);
    }
    #[test]
    fn cooking_emits_clips_texture_symbols_and_typed_setters() {
        let mut actor = create(Preset::AnimatedSprite2D, "Hero", [0.; 2]);
        let texture = Uuid::from_u128(7);
        let mut sprite = Sprite2D {
            texture: Some(texture),
            ..Default::default()
        };
        sprite.animations = vec![
            Animation2D {
                name: "idle".into(),
                fps: 4.,
                frames: vec![Frame2D::default(); 2],
                ..Default::default()
            },
            Animation2D {
                name: "run".into(),
                fps: 12.,
                looping: false,
                frames: vec![Frame2D {
                    event: 3,
                    ..Default::default()
                }],
            },
        ];
        sprite.animation = 1;
        let index = actor
            .components
            .iter()
            .position(|c| c.class.class_id.as_deref() == Some(SPRITE2D))
            .unwrap();
        write(&mut actor.components[index], &sprite);
        let tilesets = BTreeMap::new();
        let textures = |id| id == texture;
        let mut context = CookContext::new(&textures, &tilesets, [0., 0.]);
        let cooked = cook(
            &actor.components[index],
            index,
            "w2d_0_1",
            &mut context,
            "Hero",
        )
        .unwrap();
        assert!(
            cooked
                .statics
                .contains("inline constexpr epok::SpriteFrame w2d_0_1_frames_0[]=")
        );
        assert!(
            cooked
                .statics
                .contains("{w2d_0_1_frames_1,1,false,\"run\"}")
        );
        // 1/4 second frames at 4 fps.
        assert!(
            cooked
                .statics
                .contains("epok::Fixed(1024,epok::Fixed::RAW)")
        );
        assert!(cooked.setters.contains(&crate::texture::symbol(texture)));
        assert!(cooked.setters.contains("c->play(1u);"));
        assert!(
            cooked
                .setters
                .starts_with("if(auto* c=registry.resolve<epok::Sprite2DComponent>")
        );
        let none = |_| false;
        let mut context = CookContext::new(&none, &tilesets, [0., 0.]);
        assert!(cook(&actor.components[index], index, "x", &mut context, "Hero").is_err());
    }

    fn tile_layer(tileset: Uuid, cells: &crate::tilemap::Cells) -> ActorInstance {
        let mut actor = create(Preset::TileMapLayer, "Ground", [0., 0.]);
        let index = actor
            .components
            .iter()
            .position(|c| c.class.class_id.as_deref() == Some(TILEMAP_LAYER))
            .unwrap();
        write(
            &mut actor.components[index],
            &TileMapLayer {
                tile_set: Some(tileset),
                cells: crate::tilemap::encode(cells),
                z_index: -2,
                ..Default::default()
            },
        );
        actor
    }

    #[test]
    fn tile_layers_cook_one_shared_tileset_table_and_their_chunks() {
        use crate::tileset::{Collision, Tile, TileSet};
        let texture = Uuid::from_u128(11);
        let set_id = Uuid::from_u128(12);
        let mut solid = Tile::new([0, 0]);
        solid.collision = Collision::Full;
        let mut ledge = Tile::new([1, 0]);
        ledge.collision = Collision::Rect;
        ledge.rect = [0, 0, 16, 4];
        ledge.one_way = true;
        let mut flower = Tile::new([2, 0]);
        flower.frames = 2;
        flower.fps = 4.;
        let set = TileSet {
            texture: Some(texture),
            separation: [1, 0],
            tiles: vec![solid, ledge, flower],
            custom_layers: vec![crate::tileset::CustomLayer {
                name: "damage".into(),
                kind: crate::tileset::CustomKind::Int,
            }],
            ..Default::default()
        };
        let mut set = set;
        set.normalize();
        set.tiles[0].custom = vec![5];
        let tilesets: BTreeMap<_, _> = [(set_id, Arc::new(set))].into_iter().collect();
        let mut cells = crate::tilemap::Cells::new();
        cells.insert([0, 0], crate::tilemap::Cell::new([0, 0]));
        cells.insert([-1, 3], crate::tilemap::Cell::new([1, 0]));
        let first = tile_layer(set_id, &cells);
        let second = tile_layer(set_id, &cells);
        let textures = |id| id == texture;
        let mut context = CookContext::new(&textures, &tilesets, [0., 0.]);
        let a = cook(&first.components[1], 1, "w2d_0_1", &mut context, "Ground").unwrap();
        let b = cook(&second.components[1], 1, "w2d_1_1", &mut context, "Details").unwrap();
        let name = format!("w2d_tileset_{}", set_id.simple());
        // The TileSet's tables are emitted once per bank, its cells per layer.
        assert!(a.statics.contains(&format!(
            "inline constexpr epok::TileSetData {name}={{16,16,3,{name}_tiles,1,{name}_custom}};"
        )));
        assert!(!b.statics.contains("TileSetData"));
        assert!(a.statics.contains(&format!(
            "inline constexpr int32_t {name}_custom[]={{5,0,0}};"
        )));
        // The ledge is one-way with its rectangle; the flower animates at a quarter
        // second per frame and steps 17 texels (tile plus separation).
        assert!(
            a.statics
                .contains("{{1,0},17,0,epok::TileCollision::Rect,1,{0,0,16,4},1,0,{17,16}")
        );
        assert!(a.statics.contains("{{2,0},34,0,epok::TileCollision::None,0,{0,0,16,16},2,0,{17,16},epok::Fixed(1024,epok::Fixed::RAW)}"));
        assert!(
            a.statics
                .contains("inline constexpr uint16_t w2d_0_1_cells[]=")
        );
        assert!(
            b.statics
                .contains("inline constexpr int16_t w2d_1_1_grid[]={0,1};")
        );
        assert!(a.setters.contains(&format!(
            "c->tileset=&{name};c->texture={};",
            crate::texture::symbol(texture)
        )));
        assert!(a.setters.contains(
            "c->grid_origin[0]=-1;c->grid_origin[1]=0;c->grid_size[0]=2;c->grid_size[1]=1;"
        ));
        assert!(a.setters.contains("c->z_index=-2;"));
        // A layer whose TileSet is not loaded, or whose texture the bank lacks, fails.
        let empty = BTreeMap::new();
        let mut missing = CookContext::new(&textures, &empty, [0., 0.]);
        assert!(cook(&first.components[1], 1, "x", &mut missing, "Ground").is_err());
        let none = |_| false;
        let mut no_texture = CookContext::new(&none, &tilesets, [0., 0.]);
        assert!(cook(&first.components[1], 1, "x", &mut no_texture, "Ground").is_err());
        // The layer's TileSet texture joins the bank, and so does its TileSet.
        assert_eq!(
            texture_ids(&first, &tilesets).collect::<Vec<_>>(),
            vec![texture]
        );
        assert_eq!(tileset_ids(&first).collect::<Vec<_>>(), vec![set_id]);
    }

    #[test]
    fn parallax_and_canvas_records_cook_with_the_starting_view() {
        let textures = |_| true;
        let tilesets = BTreeMap::new();
        let mut context = CookContext::new(&textures, &tilesets, [100., -20.]);
        let mut hills = create(Preset::Parallax2D, "Hills", [0., 0.]);
        write(
            &mut hills.components[1],
            &Parallax2D {
                scroll_scale: [0.5, 1.],
                repeat_size: [256., 0.],
                autoscroll: [-6., 0.],
            },
        );
        let p = cook(&hills.components[1], 1, "p", &mut context, "Hills").unwrap();
        assert!(p.setters.contains("c->scroll_scale[0]=epok::Fixed(2048,epok::Fixed::RAW);c->repeat_size[0]=epok::Fixed(1048576,epok::Fixed::RAW);c->autoscroll[0]=epok::Fixed(-24576,epok::Fixed::RAW);c->reference[0]=epok::Fixed(409600,epok::Fixed::RAW);"));
        assert!(
            p.setters
                .contains("c->reference[1]=epok::Fixed(-81920,epok::Fixed::RAW);")
        );
        let hud = create(Preset::CanvasLayer2D, "HUD", [0., 0.]);
        let c = cook(&hud.components[1], 1, "c", &mut context, "HUD").unwrap();
        assert!(
            c.setters.contains("c->layer=1;") && c.setters.contains("c->follow_viewport=false;")
        );
        let mut night = create(Preset::CanvasModulate2D, "Night", [0., 0.]);
        write(
            &mut night.components[1],
            &CanvasModulate2D {
                color: [110, 120, 190],
            },
        );
        let m = cook(&night.components[1], 1, "m", &mut context, "Night").unwrap();
        assert!(
            m.setters
                .contains("c->color[0]=110;c->color[1]=120;c->color[2]=190;")
        );
    }

    #[test]
    fn the_starting_view_follows_the_current_camera_and_its_limits() {
        let mut scene = crate::scene::Scene {
            display_size: [320, 240],
            ..Default::default()
        };
        assert_eq!(start_view(&scene), [0., 0.]);
        let mut player = create(Preset::Camera2D, "Player", [-400., 60.]);
        write(
            &mut player.components[1],
            &Camera2D {
                limits_enabled: true,
                limits: [-448., -240., 448., 128.],
                offset: [0., -8.],
                ..Default::default()
            },
        );
        scene.actors.push(player);
        // The view's left edge stops at -448 and its bottom at 128.
        assert_eq!(start_view(&scene), [-448. + 160., 128. - 120. - 8.]);
        // An attached child is positioned through its parent.
        let parent = create(Preset::Node2D, "Rig", [10., 20.]);
        let mut child = create(Preset::Node2D, "Child", [5., 0.]);
        child.attach = Some(crate::actor_document::Attachment {
            actor: parent.id,
            component: None,
        });
        let child_id = child.id;
        scene.actors.push(parent);
        scene.actors.push(child);
        let index = scene.actor_index(child_id).unwrap();
        assert_eq!(world_position(&scene, index), [15., 20.]);
    }

    #[test]
    fn capacities_size_the_tile_probes_edits_and_arena_from_the_maps() {
        let mut scene = crate::scene::Scene {
            display_size: [320, 240],
            ..Default::default()
        };
        scene
            .actors
            .push(create(Preset::Sprite2D, "Sprite", [0., 0.]));
        let sprites_only = capacity_defines(true, std::slice::from_ref(&scene), std::iter::empty());
        assert!(sprites_only.contains("#define EPOK_WORLD2D_TILE_PROBES 1\n"));
        assert!(sprites_only.contains("#define EPOK_WORLD2D_TILE_EDITS 1\n"));
        scene
            .actors
            .push(create(Preset::TileMapLayer, "Ground", [0., 0.]));
        let tiled = capacity_defines(true, std::slice::from_ref(&scene), std::iter::empty());
        assert!(tiled.contains("#define EPOK_WORLD2D_TILE_PROBES 48\n"));
        assert!(tiled.contains("#define EPOK_WORLD2D_TILE_EDITS 128\n"));
        // A screen of 16-pixel tiles (22 x 17) adds its packets to the arena, beside
        // the sprite budget (34 draw items) and one page change for the layer.
        let arena = |text: &str| {
            text.lines()
                .find_map(|l| l.strip_prefix("#define EPOK_WORLD2D_ARENA_BYTES "))
                .unwrap()
                .parse::<usize>()
                .unwrap()
        };
        assert_eq!(arena(&sprites_only), 2048);
        assert_eq!(arena(&tiled), 34 * 48 + 22 * 17 * 20 + 16);
        assert_eq!(
            capacity_defines(false, std::slice::from_ref(&scene), std::iter::empty()),
            ""
        );
    }

    #[test]
    fn navigation_cooks_grids_meshes_agents_and_its_scratch() {
        let mut scene = crate::scene::Scene {
            actors: Vec::new(),
            ..Default::default()
        };
        let mut region = create(Preset::NavigationRegion2D, "Floor", [16., 32.]);
        set_record(
            &mut region,
            NAVIGATION_REGION2D,
            &crate::navigation2d::NavigationRegion2D {
                size: [64., 32.],
                ..Default::default()
            },
        );
        scene.actors.push(region);
        let mut wall = create(Preset::StaticBody2D, "Wall", [40., 40.]);
        set_record(
            &mut wall,
            COLLIDER2D,
            &Collider2D {
                size: [16., 16.],
                ..Default::default()
            },
        );
        scene.actors.push(wall);
        let target = Uuid::from_u128(5);
        let agent = create(Preset::NavigationAgent2D, "Walker", [0., 0.]);
        scene.actors.push(agent);
        let tilesets = BTreeMap::new();
        let textures = |_| false;
        let slot = |id: Uuid| (id == target).then_some(7);
        let mut context = CookContext::new(&textures, &tilesets, [0., 0.])
            .with_actors(&slot)
            .with_scene(&scene);
        let grid = cook(
            &scene.actors[0].components[1],
            1,
            "g",
            &mut context,
            "Floor",
        )
        .unwrap();
        // 4 x 2 cells of 16 px from (16, 32); the wall covers cell (1, 0): bit 1.
        assert!(
            grid.statics
                .contains("inline constexpr uint8_t g_blocked[]={2};")
        );
        assert!(grid.setters.contains("c->grid.columns=4;c->grid.rows=2;c->grid.origin[0]=epok::Fixed(65536,epok::Fixed::RAW);"));
        let mut mesh_region = scene.actors[0].clone();
        set_record(
            &mut mesh_region,
            NAVIGATION_REGION2D,
            &crate::navigation2d::NavigationRegion2D {
                mode: crate::navigation2d::NavMode::Polygon,
                ..Default::default()
            },
        );
        scene.actors[0] = mesh_region;
        let mut context = CookContext::new(&textures, &tilesets, [0., 0.])
            .with_actors(&slot)
            .with_scene(&scene);
        let mesh = cook(
            &scene.actors[0].components[1],
            1,
            "m",
            &mut context,
            "Floor",
        )
        .unwrap();
        assert!(
            mesh.setters.contains("c->is_mesh=true;")
                && mesh.setters.contains("c->mesh.triangle_count=2;")
        );
        assert!(mesh.statics.contains("m_vertices[]={epok::Fixed(65536,epok::Fixed::RAW),epok::Fixed(131072,epok::Fixed::RAW),"));
        let mut walker = scene.actors[2].clone();
        set_record(
            &mut walker,
            NAVIGATION_AGENT2D,
            &crate::navigation2d::NavigationAgent2D {
                target: Some(target),
                ..Default::default()
            },
        );
        let agent = cook(&walker.components[1], 1, "a", &mut context, "Walker").unwrap();
        assert!(
            agent
                .setters
                .contains("c->target_actor=actors[7];c->has_target=true;")
        );
        // A map with a grid sizes the A* scratch; others keep one entry.
        let mut grid_scene = scene.clone();
        grid_scene.actors[0] = create(Preset::NavigationRegion2D, "Floor", [0., 0.]);
        assert!(
            capacity_defines(true, std::slice::from_ref(&grid_scene), std::iter::empty()).contains(
                "#define EPOK_WORLD2D_NAV_CELLS 300\n#define EPOK_WORLD2D_NAV_OPEN 600\n"
            )
        );
        assert!(
            capacity_defines(true, &[crate::scene::Scene::default()], std::iter::empty())
                .contains("#define EPOK_WORLD2D_NAV_CELLS 1\n")
        );
    }

    #[test]
    fn content_nodes_cook_shapes_references_and_clips() {
        let tilesets = BTreeMap::new();
        let textures = |_| false;
        let target = Uuid::from_u128(40);
        let clip = Uuid::from_u128(41);
        let slot = |id: Uuid| (id == target).then_some(3);
        let clips = |id: Uuid| (id == clip).then_some(4);
        let mut context = CookContext::new(&textures, &tilesets, [0., 0.])
            .with_actors(&slot)
            .with_clips(&clips);
        let mut cook_one = |preset: Preset, edit: &dyn Fn(&mut ComponentInstance)| {
            let mut actor = create(preset, "Node", [0., 0.]);
            edit(&mut actor.components[1]);
            cook(&actor.components[1], 1, "n", &mut context, "Node")
        };
        let path = cook_one(Preset::Path2D, &|_| {}).unwrap();
        assert!(path.statics.contains("inline constexpr epok::Fixed n_lengths[]={epok::Fixed(0,epok::Fixed::RAW),epok::Fixed(393216,epok::Fixed::RAW)};"));
        assert!(path.setters.contains("c->point_count=2;c->closed=false;"));
        let follow = cook_one(Preset::PathFollow2D, &|c| {
            write(
                c,
                &PathFollow2D {
                    path: Some(target),
                    ping_pong: true,
                    ..Default::default()
                },
            )
        })
        .unwrap();
        assert!(
            follow.setters.contains("c->path=actors[3];")
                && follow.setters.contains("c->ping_pong=true;")
        );
        let line = cook_one(Preset::Line2D, &|_| {}).unwrap();
        assert!(line.setters.contains("c->points[1][0]=epok::Fixed(262144,epok::Fixed::RAW);c->points[1][1]=epok::Fixed(0,epok::Fixed::RAW);c->point_count=2;"));
        let polygon = cook_one(Preset::Polygon2D, &|_| {}).unwrap();
        assert!(
            polygon
                .setters
                .contains("c->vertex_count=4;c->triangle_count=2;")
                && polygon.setters.contains("c->texels=nullptr;")
        );
        assert!(
            cook_one(Preset::Polygon2D, &|c| {
                write(
                    c,
                    &Polygon2D {
                        points: vec![[0., 0.], [10., 10.], [10., 0.], [0., 10.]],
                        ..Default::default()
                    },
                )
            })
            .is_err()
        );
        let particles = cook_one(Preset::Particles2D, &|_| {}).unwrap();
        assert!(
            particles.setters.contains("c->texture=-1;")
                && particles
                    .setters
                    .contains("c->shape=epok::EmissionShape2D::Point;")
        );
        assert!(
            cook_one(Preset::Particles2D, &|c| {
                write(
                    c,
                    &Particles2D {
                        texture: Some(Uuid::from_u128(9)),
                        ..Default::default()
                    },
                )
            })
            .is_err()
        );
        let mut sound = AudioPlayer2D::default();
        sound.audio.clip = Some(clip);
        let audio = cook_one(Preset::AudioPlayer2D, &|c| write(c, &sound)).unwrap();
        assert!(
            audio
                .setters
                .contains("c->source.enabled=true;c->source.clip=4;")
        );
        let mut missing = AudioPlayer2D::default();
        missing.audio.clip = Some(Uuid::from_u128(77));
        assert!(cook_one(Preset::AudioPlayer2D, &|c| write(c, &missing)).is_err());
        let remote = cook_one(Preset::RemoteTransform2D, &|c| {
            write(
                c,
                &RemoteTransform2D {
                    target: Some(target),
                    update_scale: false,
                    ..Default::default()
                },
            )
        })
        .unwrap();
        assert!(
            remote.setters.contains("c->target=actors[3];")
                && remote.setters.contains("c->update_scale=false;")
        );
        let skeleton = cook_one(Preset::Skeleton2D, &|c| {
            let mut s = crate::skeleton2d::Skeleton2D {
                bones: vec![crate::skeleton2d::Bone {
                    actor: Some(target),
                    rest: [0., 4., 0.],
                }],
                animations: vec![crate::skeleton2d::Animation::default()],
                autoplay: Some(0),
                ..Default::default()
            };
            s.key_pose(0, 0., vec![[30., 4., 0.]]);
            write(c, &s)
        })
        .unwrap();
        assert!(skeleton.setters.starts_with("if(auto* c=registry.resolve<epok::Skeleton2DComponent>(components[1])){c->bones[0]=actors[3];c->bone_count=1;c->rest=n_rest;"));
        assert!(skeleton.statics.contains("inline constexpr epok::SkeletonClip2D n_clips[]={epok::SkeletonClip2D{n_a0_times,n_a0_poses,1,true,epok::Fixed(4096,epok::Fixed::RAW)}};"));
        assert!(skeleton.setters.contains("c->clip=0;c->playing=true;"));
        // A bone outside the map fails the build rather than animating nothing.
        assert!(
            cook_one(Preset::Skeleton2D, &|c| write(
                c,
                &crate::skeleton2d::Skeleton2D {
                    bones: vec![crate::skeleton2d::Bone::default()],
                    ..Default::default()
                }
            ))
            .is_err()
        );
        let notifier = cook_one(Preset::VisibleOnScreenNotifier2D, &|_| {}).unwrap();
        assert!(
            notifier
                .setters
                .contains("c->rect[2]=epok::Fixed(81920,epok::Fixed::RAW);")
        );
        // Particles size the shared pool; content-free builds keep a single slot.
        let mut scene = crate::scene::Scene {
            display_size: [320, 240],
            ..Default::default()
        };
        scene
            .actors
            .push(create(Preset::Particles2D, "Sparks", [0., 0.]));
        assert!(
            capacity_defines(true, std::slice::from_ref(&scene), std::iter::empty())
                .contains("#define EPOK_WORLD2D_PARTICLES 16\n")
        );
        scene.actors.clear();
        assert!(
            capacity_defines(true, std::slice::from_ref(&scene), std::iter::empty())
                .contains("#define EPOK_WORLD2D_PARTICLES 1\n")
        );
    }
}
