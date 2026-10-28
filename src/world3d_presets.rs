//! Ready-made 3D actors for the creation menus. Each preset is an actor whose
//! components are already set up, so an author never assembles a light, a
//! camera or an emitter from an empty actor and Add Component.

use crate::editor::Editor;
use crate::scene_view_mode::SceneViewMode;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Preset {
    Empty,
    Cube,
    BlockoutMesh,
    Sprite,
    Terrain,
    DungeonGenerator,
    DirectionalLight,
    PointLight,
    AudioSource,
    ParticleEmitter,
    ParticleEffect,
    BlobShadow,
    BoxCollider,
    TriggerVolume,
    Camera,
}

impl Preset {
    /// Every preset, in menu order.
    pub const ALL: [Preset; 15] = [
        Preset::Empty,
        Preset::Cube,
        Preset::BlockoutMesh,
        Preset::Sprite,
        Preset::Terrain,
        Preset::DungeonGenerator,
        Preset::DirectionalLight,
        Preset::PointLight,
        Preset::AudioSource,
        Preset::ParticleEmitter,
        Preset::ParticleEffect,
        Preset::BlobShadow,
        Preset::BoxCollider,
        Preset::TriggerVolume,
        Preset::Camera,
    ];
    /// The creation-menu submenu that lists the preset; `None` lists it on the
    /// menu itself.
    pub fn category(self) -> Option<&'static str> {
        match self {
            Preset::Empty | Preset::Camera => None,
            Preset::Cube
            | Preset::BlockoutMesh
            | Preset::Sprite
            | Preset::Terrain
            | Preset::DungeonGenerator => Some("3D Object"),
            Preset::DirectionalLight | Preset::PointLight => Some("Light"),
            Preset::AudioSource => Some("Audio"),
            Preset::ParticleEmitter | Preset::ParticleEffect | Preset::BlobShadow => {
                Some("Effects")
            }
            Preset::BoxCollider | Preset::TriggerVolume => Some("Physics"),
        }
    }
    /// The menu entry.
    pub fn label(self) -> &'static str {
        match self {
            Preset::Empty => "Create Empty",
            Preset::Cube => "Cube",
            Preset::BlockoutMesh => "Blockout Mesh",
            Preset::Sprite => "Sprite (Billboard)",
            Preset::Terrain => "Terrain",
            Preset::DungeonGenerator => "Dungeon Generator",
            Preset::DirectionalLight => "Directional Light",
            Preset::PointLight => "Point Light",
            Preset::AudioSource => "Audio Source",
            Preset::ParticleEmitter => "Particle Emitter",
            Preset::ParticleEffect => "Particle Effect",
            Preset::BlobShadow => "Blob Shadow",
            Preset::BoxCollider => "Box Collider",
            Preset::TriggerVolume => "Trigger Volume",
            Preset::Camera => "Camera",
        }
    }
    /// What the preset creates, shown as the menu entry's tooltip.
    pub fn description(self) -> &'static str {
        match self {
            Preset::Empty => "An actor with only a transform, to group or place other actors.",
            Preset::Cube => "A unit cube with a Mesh Renderer.",
            Preset::BlockoutMesh => {
                "A cube with its own editable mesh under assets/Meshes, opened for Edit geometry."
            }
            Preset::Sprite => {
                "A textured quad that turns to face the camera. Choose its texture in the Inspector."
            }
            Preset::Terrain => {
                "A paintable heightfield with the built-in ground atlas. A map without a light also gets a sun."
            }
            Preset::DungeonGenerator => {
                "Rooms, corridors and walls generated from a seed; a first layout is generated at once."
            }
            Preset::DirectionalLight => {
                "Parallel light along the actor's forward axis, like the sun."
            }
            Preset::PointLight => "Light that fades with distance from the actor.",
            Preset::AudioSource => {
                "Plays a sound clip when the game starts. Choose the clip in the Inspector."
            }
            Preset::ParticleEmitter => "A continuous stream of camera-facing particles.",
            Preset::ParticleEffect => {
                "Plays a particle effect asset. Choose the effect in the Inspector."
            }
            Preset::BlobShadow => "A soft round shadow on the ground below the actor.",
            Preset::BoxCollider => "A solid box that moving actors cannot pass through.",
            Preset::TriggerVolume => "A box that detects what enters it without blocking anything.",
            Preset::Camera => "A camera the game can render through.",
        }
    }
    /// The editor command that creates the preset (`Editor::action`).
    pub fn command(self) -> &'static str {
        match self {
            Preset::Empty => "create-3d-empty",
            Preset::Cube => "create-3d-cube",
            Preset::BlockoutMesh => "create-3d-blockout-mesh",
            Preset::Sprite => "create-3d-sprite",
            Preset::Terrain => "create-3d-terrain",
            Preset::DungeonGenerator => "create-3d-dungeon-generator",
            Preset::DirectionalLight => "create-3d-directional-light",
            Preset::PointLight => "create-3d-point-light",
            Preset::AudioSource => "create-3d-audio-source",
            Preset::ParticleEmitter => "create-3d-particle-emitter",
            Preset::ParticleEffect => "create-3d-particle-effect",
            Preset::BlobShadow => "create-3d-blob-shadow",
            Preset::BoxCollider => "create-3d-box-collider",
            Preset::TriggerVolume => "create-3d-trigger-volume",
            Preset::Camera => "create-3d-camera",
        }
    }
    pub fn from_command(command: &str) -> Option<Preset> {
        Preset::ALL.into_iter().find(|p| p.command() == command)
    }
    /// The new actor's name before it is made unique in the map.
    fn name(self) -> &'static str {
        match self {
            Preset::Empty => "Empty",
            Preset::Sprite => "Sprite",
            other => other.label(),
        }
    }
}

/// Creates `preset` under `parent` (`None` is the map root), selects it and
/// shows the 3D view, as one undo step. A root actor stands at the Scene view's
/// pivot; a parent of the same domain carries a child at its own origin.
pub fn create(e: &mut Editor, preset: Preset, parent: Option<Uuid>) -> Option<Uuid> {
    if e.playing {
        e.log("Stop the emulator before editing.");
        return None;
    }
    if e.critical_busy() {
        e.log("Editing is locked until the current operation finishes.");
        return None;
    }
    e.set_scene_view_mode(SceneViewMode::ThreeD);
    // Blockout, terrain and dungeons create assets and generate content; their
    // own recipes do that, given the parent before they record the step.
    let id = match preset {
        Preset::BlockoutMesh => crate::mesh_editor::allocate_actor_data_under(e, parent)?,
        Preset::Terrain => match crate::terrain_editor::create_under(e, parent) {
            Ok(id) => id,
            Err(error) => {
                e.log(error);
                return None;
            }
        },
        Preset::DungeonGenerator => crate::dungeon_editor::create_3d_under(e, parent),
        _ => return place(e, preset, parent),
    };
    e.select_actor(Some(id));
    Some(id)
}

/// The presets that are one actor with configured components.
fn place(e: &mut Editor, preset: Preset, parent: Option<Uuid>) -> Option<Uuid> {
    if e.scene.actors.len() >= 512 {
        e.log("Editor limit: 512 objects.");
        return None;
    }
    let original = e.scene.clone();
    let mut actor = crate::scene::Actor::cube(e.scene.unique_actor_name(preset.name()));
    let height = configure(&mut actor, preset);
    let id = actor.id;
    let name = actor.name.clone();
    e.scene.actors.push(actor);
    let origin = if e.parent_new_actor(id, parent) {
        [0.; 3]
    } else {
        e.view.center
    };
    let actor = e.scene.actors.last_mut().expect("the actor was just added");
    actor.position = [origin[0], origin[1] + height, origin[2]];
    crate::actor_components::sync(actor);
    let model = e.object_model();
    if let Err(error) = e.scene.validate_with_model(model.as_deref()) {
        e.scene = original;
        e.last_error = Some(error.clone());
        e.log(error);
        return None;
    }
    e.last_error = None;
    e.select_actor(Some(id));
    e.search.clear();
    e.changed();
    e.log(format!("Created {name} ({}).", preset.label()));
    Some(id)
}

/// Sets up the components of `preset` on a fresh cube actor and returns how far
/// above its origin the actor stands, so a box rests on the ground and a light
/// hangs above it.
fn configure(actor: &mut crate::scene::Actor, preset: Preset) -> f32 {
    use crate::lighting::{Light, LightType};
    if preset != Preset::Cube {
        actor.kind = "Empty".into();
    }
    match preset {
        Preset::Cube => 0.5,
        Preset::Sprite => {
            actor.sprite = Some(Default::default());
            0.
        }
        Preset::DirectionalLight => {
            actor.light = Some(Light::default());
            actor.rotation = [50., -30., 0.];
            3.
        }
        Preset::PointLight => {
            actor.light = Some(Light {
                kind: LightType::Point,
                ..Light::default()
            });
            3.
        }
        Preset::AudioSource => {
            actor.audio = Some(Default::default());
            0.
        }
        Preset::ParticleEmitter => {
            actor.particle_emitter = Some(Default::default());
            0.
        }
        Preset::ParticleEffect => {
            actor.particle_effect = Some(Default::default());
            0.
        }
        Preset::BlobShadow => {
            actor.blob_shadow = Some(Default::default());
            0.
        }
        Preset::BoxCollider | Preset::TriggerVolume => {
            actor.collider = Some(crate::collision::Collider {
                trigger: preset == Preset::TriggerVolume,
                ..Default::default()
            });
            0.5
        }
        Preset::Camera => {
            actor.kind = "Camera".into();
            1.5
        }
        Preset::Empty | Preset::BlockoutMesh | Preset::Terrain | Preset::DungeonGenerator => 0.,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every preset creates a configured, selected actor in one undo step, at
    /// the view's pivot from the map root and at its parent's origin from an
    /// actor's menu.
    #[test]
    fn every_preset_creates_its_configured_actor_in_one_step() {
        let root = crate::workspace::tests::temp("world3d-presets");
        let project =
            crate::workspace::create(&root, "Presets", crate::workspace::Template::Basic).unwrap();
        let mut editor = Editor::open(project).unwrap();
        editor.auto_build = false;
        let has = |e: &Editor, id: Uuid, component: &str| {
            let actor = &e.scene.actors[e.scene.actor_index(id).unwrap()];
            actor.components.iter().any(|c| c.class.name == component)
        };
        for preset in Preset::ALL {
            // Terrain and dungeons frame what they create, moving the pivot.
            editor.view.center = [4., 0., -2.];
            editor.set_scene_view_mode(SceneViewMode::UI);
            let before = editor.scene.clone();
            editor.action(preset.command());
            assert!(
                editor.last_error.is_none(),
                "{preset:?}: {:?}",
                editor.last_error
            );
            let id = editor.selected_actor.expect("the new actor is selected");
            assert!(
                before.actor_index(id).is_none(),
                "{preset:?} selected an old actor"
            );
            assert_eq!(editor.selected, editor.scene.actor_index(id));
            assert_eq!(editor.scene_view_mode, SceneViewMode::ThreeD, "{preset:?}");
            let actor = editor.scene.actors[editor.scene.actor_index(id).unwrap()].clone();
            assert_eq!(actor.class.name, "epok::Actor3D", "{preset:?}");
            assert_eq!(actor.logical_parent, None);
            match preset {
                Preset::Empty => assert_eq!(actor.components.len(), 1, "only the transform"),
                Preset::Cube => {
                    assert!(has(&editor, id, "epok::Mesh3DComponent"));
                    assert_eq!(actor.position, [4., 0.5, -2.], "a root stands at the pivot");
                }
                Preset::BlockoutMesh => assert!(actor.editable_mesh.is_some()),
                Preset::Sprite => assert!(has(&editor, id, "epok::Sprite3DComponent")),
                Preset::Terrain => assert!(actor.terrain.is_some()),
                Preset::DungeonGenerator => {
                    assert!(editor.scene.dungeons.iter().any(|g| g.root == id))
                }
                Preset::DirectionalLight | Preset::PointLight => {
                    assert!(has(&editor, id, "epok::Light3DComponent"));
                    let point =
                        actor.light.as_ref().unwrap().kind == crate::lighting::LightType::Point;
                    assert_eq!(point, preset == Preset::PointLight);
                    assert_eq!(
                        actor.position,
                        [4., 3., -2.],
                        "a light hangs above the pivot"
                    );
                }
                Preset::AudioSource => assert!(has(&editor, id, "epok::AudioComponent")),
                Preset::ParticleEmitter => {
                    assert!(has(&editor, id, "epok::ParticleEmitterComponent"))
                }
                Preset::ParticleEffect => {
                    assert!(has(&editor, id, "epok::ParticleEffectComponent"))
                }
                Preset::BlobShadow => assert!(has(&editor, id, "epok::BlobShadowComponent")),
                Preset::BoxCollider | Preset::TriggerVolume => {
                    assert!(has(&editor, id, "epok::Collider3DComponent"));
                    let trigger = actor.collider.as_ref().unwrap().trigger;
                    assert_eq!(trigger, preset == Preset::TriggerVolume);
                }
                Preset::Camera => assert!(has(&editor, id, "epok::Camera3DComponent")),
            }
            // A dungeon's first generated layout is a step of its own.
            if preset == Preset::DungeonGenerator {
                editor.undo_scene().unwrap();
            }
            editor.undo_scene().unwrap();
            assert!(
                editor.scene.actor_index(id).is_none(),
                "{preset:?} is one undo step"
            );
        }

        // From an actor's menu the preset becomes that actor's child, carried at
        // its origin, in one step; recipes with assets are parented too.
        editor.action(Preset::Cube.command());
        let hero = editor.selected_actor.unwrap();
        for preset in [Preset::PointLight, Preset::Terrain, Preset::BlockoutMesh] {
            editor.create_child(preset.command(), hero);
            assert!(
                editor.last_error.is_none(),
                "{preset:?}: {:?}",
                editor.last_error
            );
            let child = editor.selected_actor.unwrap();
            assert_ne!(child, hero);
            let actor = &editor.scene.actors[editor.scene.actor_index(child).unwrap()];
            assert_eq!(actor.logical_parent, Some(hero), "{preset:?}");
            assert_eq!(
                actor.attach.as_ref().map(|a| a.actor),
                Some(hero),
                "{preset:?}"
            );
            if preset == Preset::PointLight {
                assert_eq!(actor.position, [0., 3., 0.], "local to the parent");
            }
            editor.undo_scene().unwrap();
            assert!(editor.scene.actor_index(child).is_none(), "{preset:?}");
        }
        drop(editor);
        if root.exists() {
            std::fs::remove_dir_all(root).unwrap();
        }
    }
}
