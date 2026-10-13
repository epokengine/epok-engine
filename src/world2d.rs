//! World2D components on the editor side: their document shape and defaults, the
//! 2D "node" presets the editor creates, the textures they reference and the C++
//! they cook into a scene bank (see `runtime/world2d_components.hpp`).
//!
//! Units match the runtime: pixels, X right, Y down, degrees. Each component stores
//! one typed record under a single property key, the way the 3D built-ins do.
use crate::{
    actor_document::{ActorInstance, ClassReference, ComponentInstance},
    object_model as om,
    texture::BlendMode,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use uuid::Uuid;

pub const SPRITE2D: &str = om::SPRITE2D_COMPONENT_ID;
pub const CAMERA2D: &str = om::CAMERA2D_COMPONENT_ID;
pub const COLLIDER2D: &str = om::COLLIDER2D_COMPONENT_ID;
pub const CHARACTER_BODY2D: &str = om::CHARACTER_BODY2D_COMPONENT_ID;

pub const SPRITE_KEY: &str = "sprite2d";
pub const CAMERA_KEY: &str = "camera2d";
pub const COLLIDER_KEY: &str = "collider2d";
pub const BODY_KEY: &str = "body2d";

/// Every engine 2D component the editor authors and cooks itself.
pub fn is_builtin(class_id: &str) -> bool {
    matches!(
        class_id,
        SPRITE2D | CAMERA2D | COLLIDER2D | CHARACTER_BODY2D
    )
}
pub fn key(class_id: &str) -> Option<&'static str> {
    match class_id {
        SPRITE2D => Some(SPRITE_KEY),
        CAMERA2D => Some(CAMERA_KEY),
        COLLIDER2D => Some(COLLIDER_KEY),
        CHARACTER_BODY2D => Some(BODY_KEY),
        _ => None,
    }
}
pub fn class_name(class_id: &str) -> Option<&'static str> {
    match class_id {
        SPRITE2D => Some("epok::Sprite2DComponent"),
        CAMERA2D => Some("epok::Camera2DComponent"),
        COLLIDER2D => Some("epok::Collider2DComponent"),
        CHARACTER_BODY2D => Some("epok::CharacterBody2DComponent"),
        _ => None,
    }
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
    let result = match id {
        SPRITE2D => serde_json::from_value::<Sprite2D>(value.clone()).map(|_| ()),
        CAMERA2D => serde_json::from_value::<Camera2D>(value.clone()).map(|_| ()),
        COLLIDER2D => serde_json::from_value::<Collider2D>(value.clone()).map(|_| ()),
        _ => serde_json::from_value::<CharacterBody2D>(value.clone()).map(|_| ()),
    };
    result.map_err(|e| format!("{key}: {e}"))
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
    match class_id {
        SPRITE2D => write(&mut component, &Sprite2D::default()),
        CAMERA2D => write(&mut component, &Camera2D::default()),
        COLLIDER2D => write(&mut component, &Collider2D::default()),
        _ => write(&mut component, &CharacterBody2D::default()),
    }
    Some(component)
}
/// Textures the actor's 2D components draw. They join the bank's texture set exactly
/// like a 3D material's texture.
pub fn texture_ids(actor: &ActorInstance) -> impl Iterator<Item = Uuid> + '_ {
    actor
        .components
        .iter()
        .filter(|c| c.class.class_id.as_deref() == Some(SPRITE2D))
        .filter_map(|c| read::<Sprite2D>(c).texture)
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
const ENGINE_2D_CLASSES: [&str; 6] = [
    om::ACTOR2D_ID,
    om::SCENE_COMPONENT2D_ID,
    SPRITE2D,
    CAMERA2D,
    COLLIDER2D,
    CHARACTER_BODY2D,
];
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
                && !ENGINE_2D_CLASSES.contains(&class.id.as_str())
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
    let colliders = (largest(COLLIDER2D) + 16).clamp(16, 1024);
    let sprites = (largest(SPRITE2D) + 32).clamp(32, 1024);
    // A textured quad fragment is 40 bytes, a rectangle 20 and a page change 8.
    let arena = (sprites * 48).clamp(2048, 65536);
    format!(
        "#define EPOK_WORLD2D 1\n#define EPOK_WORLD2D_COLLIDERS {colliders}\n#define EPOK_WORLD2D_TRIGGER_PAIRS {}\n#define EPOK_WORLD2D_DRAW_ITEMS {sprites}\n#define EPOK_WORLD2D_ARENA_BYTES {arena}\n",
        (colliders * 2).clamp(32, 512)
    )
}

// ---- 2D node presets ---------------------------------------------------------------------

/// The ready-made 2D node types the editor offers. Each is an `Actor2D` with a set of
/// engine components; the user can add or remove components afterwards.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Preset {
    Node2D,
    Sprite2D,
    AnimatedSprite2D,
    Camera2D,
    StaticBody2D,
    Area2D,
    CharacterBody2D,
}
impl Preset {
    pub const ALL: [Preset; 7] = [
        Preset::Node2D,
        Preset::Sprite2D,
        Preset::AnimatedSprite2D,
        Preset::Camera2D,
        Preset::CharacterBody2D,
        Preset::StaticBody2D,
        Preset::Area2D,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Preset::Node2D => "Node2D",
            Preset::Sprite2D => "Sprite2D",
            Preset::AnimatedSprite2D => "AnimatedSprite2D",
            Preset::Camera2D => "Camera2D",
            Preset::StaticBody2D => "StaticBody2D",
            Preset::Area2D => "Area2D",
            Preset::CharacterBody2D => "CharacterBody2D",
        }
    }
    pub fn description(self) -> &'static str {
        match self {
            Preset::Node2D => "An empty 2D actor: a transform to group or position others.",
            Preset::Sprite2D => "Draws a texture or one cell of a sprite sheet.",
            Preset::AnimatedSprite2D => "Plays named frame animations from a sprite sheet.",
            Preset::Camera2D => "Follows its actor; smoothing, limits and drag margins.",
            Preset::StaticBody2D => "A solid box that blocks characters: floors, walls, platforms.",
            Preset::Area2D => "A trigger region that reports Enter / Stay / Exit overlaps.",
            Preset::CharacterBody2D => "A player or enemy moved with move_and_slide().",
        }
    }
    pub fn components(self) -> &'static [&'static str] {
        match self {
            Preset::Node2D => &[],
            Preset::Sprite2D | Preset::AnimatedSprite2D => &[SPRITE2D],
            Preset::Camera2D => &[CAMERA2D],
            Preset::StaticBody2D => &[COLLIDER2D],
            Preset::Area2D => &[COLLIDER2D],
            Preset::CharacterBody2D => &[SPRITE2D, COLLIDER2D, CHARACTER_BODY2D],
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
/// generated `actor_apply_N`, and namespace-scope constants (frame clips).
#[derive(Default)]
pub struct Cooked {
    pub statics: String,
    pub setters: String,
}
/// Cooks the 2D component at `position` in its actor. `symbol` names the unique prefix
/// for its constants; `textures` lists the textures this bank can address.
pub fn cook(
    component: &ComponentInstance,
    position: usize,
    symbol: &str,
    textures: &dyn Fn(Uuid) -> bool,
    label: &str,
) -> Result<Cooked, String> {
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
        let cooked = cook(
            &actor.components[index],
            index,
            "w2d_0_1",
            &|id| id == texture,
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
        assert!(cook(&actor.components[index], index, "x", &|_| false, "Hero").is_err());
    }
}
