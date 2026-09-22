//! Built-in component data and the editor's spatial projection of an Actor.
//! The projection is owned by that Actor; only the component document is saved.
use crate::{
    actor_document::{ActorInstance, ClassReference, ComponentInstance},
    object_model as om,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub const PARTICLES: &str = "1d067605-c408-40b8-b2c2-718b8cf0c601";
pub const TIMELINE: &str = "1d067605-c408-40b8-b2c2-718b8cf0c602";
pub const EFFECT: &str = "1d067605-c408-40b8-b2c2-718b8cf0c603";
pub const PALETTE: &str = "1d067605-c408-40b8-b2c2-718b8cf0c604";
pub const SHADOW: &str = "1d067605-c408-40b8-b2c2-718b8cf0c605";

pub fn domain(actor: &ActorInstance) -> crate::reflection_schema::Domain {
    use crate::reflection_schema::Domain;
    if actor.rect.is_some()
        || actor.canvas.is_some()
        || actor
            .root()
            .is_some_and(|r| r.class.class_id.as_deref() == Some(om::RECT_TRANSFORM_COMPONENT_ID))
    {
        Domain::UI
    } else if actor
        .root()
        .is_some_and(|r| r.class.class_id.as_deref() == Some(om::SCENE_COMPONENT2D_ID))
    {
        Domain::World2D
    } else if actor.root().is_some() {
        Domain::World3D
    } else {
        Domain::None
    }
}

pub fn native(id: &str) -> bool {
    matches!(
        id,
        om::SCENE_COMPONENT3D_ID
            | om::SCENE_COMPONENT2D_ID
            | om::RECT_TRANSFORM_COMPONENT_ID
            | om::MESH3D_COMPONENT_ID
            | om::CAMERA3D_COMPONENT_ID
            | om::SPRITE3D_COMPONENT_ID
            | om::LIGHT3D_COMPONENT_ID
            | om::COLLIDER3D_COMPONENT_ID
            | om::CANVAS_COMPONENT_ID
            | om::IMAGE_COMPONENT_ID
            | om::TEXT_COMPONENT_ID
            | om::PROGRESS_BAR_COMPONENT_ID
            | om::LAYOUT_ELEMENT_COMPONENT_ID
            | om::LAYOUT_CONTAINER_COMPONENT_ID
            | om::FOCUSABLE_COMPONENT_ID
            | om::AUDIO_COMPONENT_ID
            | PARTICLES
            | TIMELINE
            | EFFECT
            | PALETTE
            | SHADOW
    )
}

fn upsert(
    actor: &mut ActorInstance,
    id: &str,
    class: &str,
    name: &str,
    root: bool,
    properties: Value,
) {
    let index = actor.components.iter().position(|c| {
        if root {
            c.root
        } else {
            c.class.class_id.as_deref() == Some(id)
        }
    });
    let index = index.unwrap_or_else(|| {
        // Projection may be synchronized on a clone for saving or hashing. A
        // not-yet-materialized built-in therefore needs a stable identity.
        use sha2::{Digest, Sha256};
        let digest = Sha256::digest([actor.id.as_bytes().as_slice(), id.as_bytes()].concat());
        let mut bytes: [u8; 16] = digest[..16].try_into().unwrap();
        bytes[6] = (bytes[6] & 15) | 0x50;
        bytes[8] = (bytes[8] & 63) | 0x80;
        let mut component = ComponentInstance::new(
            uuid::Uuid::from_bytes(bytes),
            ClassReference::new(class, id),
            name,
        );
        component.root = root;
        actor.components.push(component);
        actor.components.len() - 1
    });
    let component = &mut actor.components[index];
    let values: BTreeMap<String, Value> =
        serde_json::from_value(properties).expect("native component property map");
    component.overrides.extend(values.keys().cloned());
    component.properties.extend(values);
}

pub fn sync(actor: &mut ActorInstance) {
    // Component controls and the viewport edit the same document through two
    // views. Merge only changed projection fields; a component edit must not
    // be overwritten by an unchanged viewport cache (or vice versa).
    let pending = serde_json::to_value(&actor.data).expect("built-in projection");
    let baseline = actor.projection_baseline.clone();
    let mut projected = actor.clone();
    read(&mut projected);
    let current = serde_json::to_value(&projected.data).expect("built-in component projection");
    macro_rules! merge {($($field:ident),*)=>{$(if pending[stringify!($field)]==baseline[stringify!($field)] && current[stringify!($field)]!=pending[stringify!($field)] {actor.data.$field=projected.data.$field.clone();})*};}
    merge!(
        kind,
        position,
        rotation,
        scale,
        material,
        lighting,
        camera_fov,
        camera_sky_color,
        editable_mesh,
        terrain,
        skeletal_mesh,
        sprite,
        sprite_animator,
        light,
        collider,
        audio,
        canvas,
        rect,
        image,
        text,
        progress,
        layout_element,
        layout_container,
        focusable,
        particle_emitter,
        palette_animator,
        blob_shadow,
        timeline,
        particle_effect
    );
    let data = actor.data.clone();
    let ui = data.rect.is_some()
        || data.canvas.is_some()
        || actor.class.class_id.as_deref() == Some(om::UI_ACTOR_ID);
    let two_d = actor.class.class_id.as_deref() == Some(om::ACTOR2D_ID)
        || actor
            .root()
            .is_some_and(|r| r.class.class_id.as_deref() == Some(om::SCENE_COMPONENT2D_ID));
    if ui && actor.class.class_id.as_deref() == Some(om::ACTOR3D_ID) {
        actor.class = ClassReference::new("epok::UIActor", om::UI_ACTOR_ID);
        if let Some(root) = actor.components.iter_mut().find(|c| c.root) {
            root.class = ClassReference::new(
                "epok::RectTransformComponent",
                om::RECT_TRANSFORM_COMPONENT_ID,
            );
        }
    }
    let mut wanted = Vec::new();
    if ui {
        wanted.push((
            om::RECT_TRANSFORM_COMPONENT_ID,
            "epok::RectTransformComponent",
            "Rect Transform",
            true,
            json!({"rect": data.rect.clone().unwrap_or_default()}),
        ));
    } else if two_d {
        wanted.push((om::SCENE_COMPONENT2D_ID,"epok::SceneComponent2D","Transform 2D",true,
            json!({"position":[data.position[0],data.position[1]],"rotation":data.rotation[2],"scale":[data.scale[0],data.scale[1]]})));
    } else if !two_d
        && (actor.root().is_some() || actor.class.class_id.as_deref() == Some(om::ACTOR3D_ID))
    {
        wanted.push((
            om::SCENE_COMPONENT3D_ID,
            "epok::SceneComponent3D",
            "Transform",
            true,
            json!({"position":data.position,"rotation":data.rotation,"scale":data.scale}),
        ));
    }
    if !ui && !two_d {
        if data.kind == "Mesh" {
            wanted.push((om::MESH3D_COMPONENT_ID,"epok::Mesh3DComponent","Mesh Renderer",false,
                json!({"material":data.material,"lighting":data.lighting,"editable_mesh":data.editable_mesh,"terrain":data.terrain,"skeletal_mesh":data.skeletal_mesh})));
        }
        if data.kind == "Camera" {
            wanted.push((
                om::CAMERA3D_COMPONENT_ID,
                "epok::Camera3DComponent",
                "Camera",
                false,
                json!({"camera_fov":data.camera_fov,"camera_sky_color":data.camera_sky_color}),
            ));
        }
    }
    macro_rules! optional {
        ($field:ident, $id:expr, $class:expr, $name:expr) => {
            if let Some(value) = &data.$field { wanted.push(($id,$class,$name,false,json!({stringify!($field):value}))); }
        }
    }
    if !ui && !two_d {
        if let Some(sprite) = &data.sprite {
            wanted.push((
                om::SPRITE3D_COMPONENT_ID,
                "epok::Sprite3DComponent",
                "Sprite",
                false,
                json!({"sprite":sprite,"sprite_animator":data.sprite_animator}),
            ));
        }
        optional!(
            light,
            om::LIGHT3D_COMPONENT_ID,
            "epok::Light3DComponent",
            "Light"
        );
        optional!(
            collider,
            om::COLLIDER3D_COMPONENT_ID,
            "epok::Collider3DComponent",
            "Collider"
        );
        optional!(
            particle_emitter,
            PARTICLES,
            "epok::ParticleEmitterComponent",
            "Particle Emitter"
        );
        optional!(
            palette_animator,
            PALETTE,
            "epok::PaletteAnimatorComponent",
            "Palette Animator"
        );
        optional!(
            blob_shadow,
            SHADOW,
            "epok::BlobShadowComponent",
            "Blob Shadow"
        );
    }
    if ui {
        optional!(
            canvas,
            om::CANVAS_COMPONENT_ID,
            "epok::CanvasComponent",
            "Canvas"
        );
        optional!(
            image,
            om::IMAGE_COMPONENT_ID,
            "epok::ImageComponent",
            "Image"
        );
        optional!(text, om::TEXT_COMPONENT_ID, "epok::TextComponent", "Text");
        optional!(
            progress,
            om::PROGRESS_BAR_COMPONENT_ID,
            "epok::ProgressBarComponent",
            "Progress Bar"
        );
        optional!(
            layout_element,
            om::LAYOUT_ELEMENT_COMPONENT_ID,
            "epok::LayoutElementComponent",
            "Layout Element"
        );
        optional!(
            layout_container,
            om::LAYOUT_CONTAINER_COMPONENT_ID,
            "epok::LayoutContainerComponent",
            "Layout Container"
        );
        optional!(
            focusable,
            om::FOCUSABLE_COMPONENT_ID,
            "epok::FocusableComponent",
            "Focusable"
        );
    }
    optional!(
        audio,
        om::AUDIO_COMPONENT_ID,
        "epok::AudioComponent",
        "Audio"
    );
    optional!(timeline, TIMELINE, "epok::TimelineComponent", "Timeline");
    optional!(
        particle_effect,
        EFFECT,
        "epok::ParticleEffectComponent",
        "Particle Effect"
    );
    actor.components.retain(|c| {
        c.root
            || !c.class.class_id.as_deref().is_some_and(native)
            || wanted
                .iter()
                .any(|(id, _, _, _, _)| c.class.class_id.as_deref() == Some(*id))
    });
    for (id, class, name, root, properties) in wanted {
        upsert(actor, id, class, name, root, properties);
    }
    actor.data.id = actor.id;
    actor.data.name = actor.name.clone();
    actor.data.active = actor.active;
    actor.projection_baseline = serde_json::to_value(&actor.data).expect("built-in projection");
}

pub fn read(actor: &mut ActorInstance) {
    let mut data = crate::scene::BuiltinData::cube(actor.name.clone());
    data.id = actor.id;
    data.active = actor.active;
    data.kind = "Empty".into();
    data.position = [0.; 3];
    data.parent = actor.data.parent;
    data.blueprint_instance = actor.data.blueprint_instance.clone();
    for component in &actor.components {
        let id = component.class.class_id.as_deref().unwrap_or("");
        let p = &component.properties;
        if component.root
            && !matches!(
                id,
                om::SCENE_COMPONENT2D_ID | om::RECT_TRANSFORM_COMPONENT_ID
            )
        {
            if let Some(value) = decoded(p, "position") {
                data.position = value;
            }
            if let Some(value) = decoded(p, "rotation") {
                data.rotation = value;
            }
            if let Some(value) = decoded(p, "scale") {
                data.scale = value;
            }
        }
        match id {
            om::SCENE_COMPONENT2D_ID => {
                let position: [f32; 2] = decoded(p, "position").unwrap_or([0.; 2]);
                let scale: [f32; 2] = decoded(p, "scale").unwrap_or([1.; 2]);
                data.position = [position[0], position[1], 0.];
                data.rotation = [0., 0., decoded(p, "rotation").unwrap_or(0.)];
                data.scale = [scale[0], scale[1], 1.];
            }
            om::MESH3D_COMPONENT_ID => {
                data.kind = "Mesh".into();
                if let Some(value) = decoded(p, "material") {
                    data.material = value;
                }
                if let Some(value) = decoded(p, "lighting") {
                    data.lighting = value;
                }
                data.editable_mesh = decoded(p, "editable_mesh");
                data.terrain = decoded(p, "terrain");
                data.skeletal_mesh = decoded(p, "skeletal_mesh");
            }
            om::CAMERA3D_COMPONENT_ID => {
                data.kind = "Camera".into();
                data.camera_fov = decoded(p, "camera_fov").unwrap_or(90.);
                data.camera_sky_color = decoded(p, "camera_sky_color")
                    .unwrap_or(crate::scene::DEFAULT_CAMERA_SKY_COLOR);
            }
            om::RECT_TRANSFORM_COMPONENT_ID => {
                data.rect = Some(decoded(p, "rect").unwrap_or_default())
            }
            om::SPRITE3D_COMPONENT_ID => {
                data.sprite = Some(decoded(p, "sprite").unwrap_or_default());
                data.sprite_animator = decoded(p, "sprite_animator");
            }
            om::LIGHT3D_COMPONENT_ID => data.light = Some(decoded(p, "light").unwrap_or_default()),
            om::COLLIDER3D_COMPONENT_ID => {
                data.collider = Some(decoded(p, "collider").unwrap_or_default())
            }
            om::AUDIO_COMPONENT_ID if data.audio.is_none() => {
                data.audio = Some(decoded(p, "audio").unwrap_or_default())
            }
            om::CANVAS_COMPONENT_ID => data.canvas = Some(decoded(p, "canvas").unwrap_or_default()),
            om::IMAGE_COMPONENT_ID => data.image = Some(decoded(p, "image").unwrap_or_default()),
            om::TEXT_COMPONENT_ID => data.text = Some(decoded(p, "text").unwrap_or_default()),
            om::PROGRESS_BAR_COMPONENT_ID => {
                data.progress = Some(decoded(p, "progress").unwrap_or_default())
            }
            om::LAYOUT_ELEMENT_COMPONENT_ID => {
                data.layout_element = Some(decoded(p, "layout_element").unwrap_or_default())
            }
            om::LAYOUT_CONTAINER_COMPONENT_ID => {
                data.layout_container = Some(decoded(p, "layout_container").unwrap_or_default())
            }
            om::FOCUSABLE_COMPONENT_ID => {
                data.focusable = Some(decoded(p, "focusable").unwrap_or_default())
            }
            PARTICLES => {
                data.particle_emitter = Some(decoded(p, "particle_emitter").unwrap_or_default())
            }
            PALETTE => {
                data.palette_animator = Some(decoded(p, "palette_animator").unwrap_or_default())
            }
            SHADOW => data.blob_shadow = Some(decoded(p, "blob_shadow").unwrap_or_default()),
            TIMELINE => data.timeline = decoded(p, "timeline"),
            EFFECT => data.particle_effect = decoded(p, "particle_effect"),
            _ => {}
        }
    }
    // Resolved mesh buffers are caches, excluded from the component document.
    // Keep them when a component edit or identity remap leaves the asset unchanged.
    macro_rules! retain_cache {($($field:ident),*)=>{$(if serde_json::to_value(&data.$field).ok()==serde_json::to_value(&actor.data.$field).ok(){data.$field=actor.data.$field.clone();})*};}
    retain_cache!(editable_mesh, terrain, skeletal_mesh);
    actor.data = data;
    actor.projection_baseline = serde_json::to_value(&actor.data).expect("built-in projection");
}
fn decoded<T: serde::de::DeserializeOwned>(
    properties: &BTreeMap<String, Value>,
    key: &str,
) -> Option<T> {
    properties
        .get(key)
        .filter(|v| !v.is_null())
        .and_then(|v| serde_json::from_value(v.clone()).ok())
}

/// Validate the persisted payload before constructing the editor projection.
/// An invalid authored value must never turn into a default on save.
pub fn validate(component: &ComponentInstance) -> Result<(), String> {
    fn field<T: serde::de::DeserializeOwned>(
        p: &BTreeMap<String, Value>,
        key: &str,
    ) -> Result<(), String> {
        if let Some(value) = p.get(key).filter(|v| !v.is_null()) {
            serde_json::from_value::<T>(value.clone()).map_err(|e| format!("{key}: {e}"))?;
        }
        Ok(())
    }
    let p = &component.properties;
    macro_rules! check {
        ($key:literal,$ty:ty) => {
            field::<$ty>(p, $key)?;
        };
    }
    match component.class.class_id.as_deref().unwrap_or("") {
        om::SCENE_COMPONENT3D_ID => {
            check!("position", [f32; 3]);
            check!("rotation", [f32; 3]);
            check!("scale", [f32; 3]);
        }
        om::SCENE_COMPONENT2D_ID => {
            check!("position", [f32; 2]);
            check!("rotation", f32);
            check!("scale", [f32; 2]);
        }
        om::RECT_TRANSFORM_COMPONENT_ID => {
            check!("rect", crate::hud::RectTransform);
        }
        om::MESH3D_COMPONENT_ID => {
            check!("material", crate::scene::Material);
            check!("lighting", crate::lighting::MeshLighting);
            check!("editable_mesh", crate::mesh::Component);
            check!("terrain", crate::terrain::Component);
            check!("skeletal_mesh", crate::skeletal::Component);
        }
        om::CAMERA3D_COMPONENT_ID => {
            check!("camera_fov", f32);
            check!("camera_sky_color", [f32; 3]);
        }
        om::SPRITE3D_COMPONENT_ID => {
            check!("sprite", crate::sprites::Sprite);
            check!("sprite_animator", crate::sprites::SpriteAnimator);
        }
        om::LIGHT3D_COMPONENT_ID => {
            check!("light", crate::lighting::Light);
        }
        om::COLLIDER3D_COMPONENT_ID => {
            check!("collider", crate::collision::Collider);
        }
        om::AUDIO_COMPONENT_ID => {
            check!("audio", crate::audio::AudioSource);
        }
        om::CANVAS_COMPONENT_ID => {
            check!("canvas", crate::hud::Canvas);
        }
        om::IMAGE_COMPONENT_ID => {
            check!("image", crate::hud::Image);
        }
        om::TEXT_COMPONENT_ID => {
            check!("text", crate::hud::Text);
        }
        om::PROGRESS_BAR_COMPONENT_ID => {
            check!("progress", crate::hud::ProgressBar);
        }
        om::LAYOUT_ELEMENT_COMPONENT_ID => {
            check!("layout_element", crate::hud::LayoutElement);
        }
        om::LAYOUT_CONTAINER_COMPONENT_ID => {
            check!("layout_container", crate::hud::LayoutContainer);
        }
        om::FOCUSABLE_COMPONENT_ID => {
            check!("focusable", crate::hud::Focusable);
        }
        PARTICLES => {
            check!("particle_emitter", crate::particles::Emitter);
        }
        TIMELINE => {
            check!("timeline", crate::timeline_scene::Component);
        }
        EFFECT => {
            check!("particle_effect", crate::particle_effect_scene::Component);
        }
        PALETTE => {
            check!("palette_animator", crate::palette::Animator);
        }
        SHADOW => {
            check!("blob_shadow", crate::shadows::BlobShadow);
        }
        _ => {}
    }
    Ok(())
}
