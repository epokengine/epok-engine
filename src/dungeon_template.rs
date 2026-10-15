//! The Dungeon starter: the Third Person character and camera in a generated dungeon.
//!
//! The Third Person template provides the character, its controller in the chosen
//! flavor, the camera and the lighting setup; its arena is replaced by a Dungeon
//! Generator following the built-in recipe and style, and the key and door are
//! installed in the same flavor as the controller.
use crate::{
    dungeon::{Generator, Source},
    scene::{Actor, Scene},
    workspace::GameplayFlavor,
};
use std::path::Path;

/// A seed whose built-in recipe gives a varied first dungeon within the budget.
pub const SEED: u32 = 1207;
/// The map the sample handmade room is built in; outside assets/scenes, so it is never
/// cooked or played.
pub const WORKBENCH: &str = "assets/Dungeons/Rooms/Workbench.epokmap";
const REACTOR_MESH: &str = "assets/Dungeons/Rooms/ReactorShell.epokasset";
const REACTOR_ROOM: &str = "assets/Dungeons/Rooms/Reactor.epokasset";
const RECIPE: &str = "assets/Dungeons/Recipes/Dungeon.epokasset";

pub fn create(root: &Path, flavor: GameplayFlavor) -> Result<Scene, String> {
    let mut scene = crate::third_person::create(root, flavor)?;
    scene.sync_actor_components();
    // The arena, its props and its sun give way to the dungeon, which brings its own
    // light. Inactive actors would still be cooked, so they are deleted.
    let environment = scene
        .actors
        .iter()
        .find(|a| a.name == "Environment")
        .map(|a| a.id)
        .ok_or("The Third Person scene has no Environment to replace")?;
    scene.delete_actor_branch(environment);
    let world = scene
        .actors
        .iter()
        .find(|a| a.name == "World")
        .map(|a| a.id);
    let player = scene
        .actors
        .iter()
        .find(|a| a.name == "Player")
        .map(|a| a.id)
        .ok_or("The Third Person scene has no Player")?;

    let mut root_actor = Actor::cube("Dungeon".into());
    root_actor.kind = "Empty".into();
    root_actor.position = [0.; 3];
    root_actor.logical_parent = world;
    let mut generator = Generator::new(root_actor.id);
    generator.seed = SEED;
    generator.source = Source::Recipe;
    generator.spawn_actor = Some(player);
    // The built-in recipe, with one room of the loop made from the sample handmade room:
    // its light is an actor, and the console's RAM keeps room for the gameplay flavors.
    let reactor = reactor(root)?;
    let mut recipe = crate::dungeon_recipe::builtin();
    recipe.steps.insert(
        recipe.steps.len() - 1,
        crate::dungeon_recipe::Step::HandmadeRooms {
            paths: vec!["loop".into()],
            rooms: vec![reactor],
            most: 1,
        },
    );
    generator.recipe = Some(crate::dungeon_recipe::create(root, RECIPE, &recipe)?);
    let id = generator.id;
    scene.actors.push(root_actor);
    scene.dungeons.push(generator);
    scene.refresh_actor_hierarchy();

    crate::dungeon_gameplay::install(root, flavor)?;
    crate::dungeon_bake::generate(root, &mut scene, id, None)?;
    scene.sync_actor_components();
    scene.bake = Some(crate::lighting::bake(&scene)?);
    scene.validate()?;

    crate::project::write_changed(
        &root.join("UserSettings/SceneView.epokprefs"),
        &crate::document::to_vec(&overview()).map_err(|e| e.to_string())?,
    )?;
    crate::project::write_changed(&root.join("README.md"), readme(flavor).as_bytes())?;
    Ok(scene)
}

/// The sample handmade room, built in the workbench map the way a designer would: an
/// EditableMesh shell (grid floor, pipe walls with a doorway on every side, ceiling and
/// a reactor core) and four console cubes with colliders, a light, and empties named
/// Door at the doorways. It is captured into a Dungeon Room the template's recipe uses.
fn reactor(root: &Path) -> Result<uuid::Uuid, String> {
    use crate::actor_document::Attachment;
    use crate::dungeon_bake::BuiltinTexture as Texture;
    use crate::mesh::{Document, Face, Group, Slot};
    const CELL: f32 = 3.;
    const CELLS: i32 = 5;
    const HEIGHT: f32 = 6.;
    const DOOR: f32 = 3.6;
    let span = CELL * CELLS as f32;
    let id = |what: &str| crate::dungeon::derived_id(uuid::Uuid::nil(), &format!("reactor-{what}"));
    let mut doc = Document {
        groups: vec![Group {
            id: id("group"),
            name: "Reactor".into(),
            parent: None,
        }],
        ..Default::default()
    };
    let slot = |name: &str, texture: Texture| Slot {
        id: id(name),
        name: name.into(),
        material: crate::scene::Material {
            texture: Some(texture.id()),
            ..Default::default()
        },
    };
    doc.materials = vec![
        slot("Floor", Texture::FloorGrid),
        slot("Walls", Texture::WallPipes),
        slot("Ceiling", Texture::Ceiling),
        slot("Core", Texture::Column),
    ];
    // Corners, their UVs and the material slot.
    type Quad = ([[f32; 3]; 4], [[f32; 2]; 4], usize);
    let mut quads: Vec<Quad> = Vec::new();
    // A vertical face along a -> b (x, z) looks to the left of a -> b.
    let wall = |a: [f32; 2], b: [f32; 2], y: [f32; 2], top: f32, material: usize| {
        let p = [
            [a[0], y[0], a[1]],
            [b[0], y[0], b[1]],
            [b[0], y[1], b[1]],
            [a[0], y[1], a[1]],
        ];
        let length = (b[0] - a[0]).abs() + (b[1] - a[1]).abs();
        let uv = p.map(|v| {
            let along = ((v[0] - a[0]).abs() + (v[2] - a[1]).abs()) / length;
            [along, ((top - v[1]) / top).clamp(0., 1.)]
        });
        (p, uv, material)
    };
    let doorway = |x: i32, z: i32, side: usize| match side {
        0 | 1 => z == CELLS / 2,
        _ => x == CELLS / 2,
    };
    for z in 0..CELLS {
        for x in 0..CELLS {
            let (x0, x1, z0, z1) = (
                x as f32 * CELL,
                (x + 1) as f32 * CELL,
                z as f32 * CELL,
                (z + 1) as f32 * CELL,
            );
            // West, east, north and south edges of the footprint, looking in.
            let edges = [
                (x == 0, [x0, z1], [x0, z0]),
                (x == CELLS - 1, [x1, z0], [x1, z1]),
                (z == 0, [x0, z0], [x1, z0]),
                (z == CELLS - 1, [x1, z1], [x0, z1]),
            ];
            for (side, (on_edge, a, b)) in edges.into_iter().enumerate() {
                if !on_edge {
                    continue;
                }
                // Walls beside a doorway are cut at the door height, so they meet its
                // lintel edge to edge; the others are one face.
                let beside = match side {
                    0 | 1 => (z - CELLS / 2).abs() == 1,
                    _ => (x - CELLS / 2).abs() == 1,
                };
                if doorway(x, z, side) {
                    quads.push(wall(a, b, [DOOR, HEIGHT], HEIGHT, 1));
                } else if beside {
                    quads.push(wall(a, b, [0., DOOR], HEIGHT, 1));
                    quads.push(wall(a, b, [DOOR, HEIGHT], HEIGHT, 1));
                } else {
                    quads.push(wall(a, b, [0., HEIGHT], HEIGHT, 1));
                }
            }
            let floor = [[x0, 0., z0], [x0, 0., z1], [x1, 0., z1], [x1, 0., z0]];
            let uv = [[0., 0.], [0., 1.], [1., 1.], [1., 0.]];
            quads.push((floor, uv, 0));
            let mut ceiling = floor.map(|v| [v[0], HEIGHT, v[2]]);
            ceiling.reverse();
            let mut ceiling_uv = uv;
            ceiling_uv.reverse();
            quads.push((ceiling, ceiling_uv, 2));
        }
    }
    // The reactor core, floor to ceiling, its faces looking out.
    let (lo, hi) = (span * 0.5 - 1., span * 0.5 + 1.);
    for (a, b) in [
        ([hi, lo], [lo, lo]),
        ([lo, hi], [hi, hi]),
        ([lo, lo], [lo, hi]),
        ([hi, hi], [hi, lo]),
    ] {
        quads.push(wall(a, b, [0., HEIGHT], HEIGHT, 3));
    }
    let mut vertices: Vec<[f32; 3]> = Vec::new();
    for (n, (points, uv, material)) in quads.into_iter().enumerate() {
        let corners = points.map(|p| {
            let found = vertices
                .iter()
                .position(|v| v.map(f32::to_bits) == p.map(f32::to_bits));
            found.unwrap_or_else(|| {
                vertices.push(p);
                vertices.len() - 1
            }) as u32
        });
        doc.faces.push(Face {
            id: id(&format!("face-{n}")),
            vertices: corners,
            group: doc.groups[0].id,
            material: doc.materials[material].id,
            uv,
            vertex_colors: None,
        });
    }
    doc.vertices = vertices;
    doc.validate()?;
    let source = serde_json::to_vec(&doc).map_err(|e| e.to_string())?;
    let mesh_id = uuid::Uuid::new_v4();
    let package = crate::assets::Package {
        meta: crate::assets::Metadata {
            version: 1,
            id: mesh_id,
            kind: crate::assets::Kind::EditableMesh,
            importer_version: 1,
            source: REACTOR_MESH.into(),
            source_hash: crate::assets::hash(&source),
            settings: crate::import_settings::Settings::Authored,
            extra: Default::default(),
        },
        source,
    };
    crate::assets::atomic_write(
        &crate::assets::inside(root, REACTOR_MESH)?,
        &package.bytes()?,
        None,
    )?;

    // The workbench map: the room under one actor.
    let mut map = Scene {
        name: "Room workbench".into(),
        actors: Vec::new(),
        ..Default::default()
    };
    let mut room = Actor::cube("Reactor".into());
    room.kind = "Empty".into();
    room.position = [0.; 3];
    let room_id = room.id;
    map.actors.push(room);
    let child = |mut actor: Actor, position: [f32; 3], scale: [f32; 3]| {
        actor.position = position;
        actor.scale = scale;
        actor.logical_parent = Some(room_id);
        actor.attach = Some(Attachment {
            actor: room_id,
            component: None,
        });
        actor
    };
    let mut shell = child(Actor::cube("Reactor shell".into()), [0.; 3], [1.; 3]);
    shell.editable_mesh = Some(crate::mesh::Component::new(mesh_id));
    // The shell's collider is the core; the generator walls and floors the room.
    shell.collider = Some(crate::collision::Collider {
        center: [span * 0.5, HEIGHT * 0.5, span * 0.5],
        half_extents: [1., HEIGHT * 0.5, 1.],
        ..Default::default()
    });
    map.actors.push(shell);
    for (n, (x, z)) in [(4., 4.), (11., 4.), (4., 11.), (11., 11.)]
        .into_iter()
        .enumerate()
    {
        let mut console = child(
            Actor::cube(format!("Console {}", n + 1)),
            [x, 0.5, z],
            [1.6, 1., 0.8],
        );
        console.material.texture = Some(Texture::WallRibbed.id());
        console.collider = Some(crate::collision::Collider::default());
        map.actors.push(console);
    }
    let mut glow = child(
        Actor::cube("Reactor light".into()),
        [span * 0.5, 4.6, span * 0.5],
        [1.; 3],
    );
    glow.kind = "Empty".into();
    glow.light = Some(crate::lighting::Light {
        kind: crate::lighting::LightType::Point,
        mode: crate::lighting::LightMode::Baked,
        color: [0.55, 0.85, 1.],
        intensity: 1.3,
        range: 11.,
        ..Default::default()
    });
    map.actors.push(glow);
    for (n, position) in [
        [0., 0., span * 0.5],
        [span, 0., span * 0.5],
        [span * 0.5, 0., 0.],
        [span * 0.5, 0., span],
    ]
    .into_iter()
    .enumerate()
    {
        let name = if n == 0 {
            "Door".to_owned()
        } else {
            format!("Door {}", n + 1)
        };
        let mut door = child(Actor::cube(name), position, [1.; 3]);
        door.kind = "Empty".into();
        map.actors.push(door);
    }
    map.refresh_actor_hierarchy();
    for actor in &mut map.actors {
        crate::actor_components::sync(actor);
    }
    map.save(&crate::assets::inside(root, WORKBENCH)?)?;
    let index = crate::assets::scan(root, &mut Default::default());
    let mut captured = crate::dungeon_room::capture(&map, room_id, CELL, &index)?;
    // Keys and items go beside the core, clear of the consoles and the doorways.
    captured.spots = vec![[1, 2], [3, 2]];
    crate::dungeon_room::create(root, REACTOR_ROOM, &captured)
}

/// The Scene view the project opens with: the whole dungeon from above, at an angle.
fn overview() -> crate::viewport::View {
    let mut view = crate::viewport::View {
        yaw: 0.,
        pitch: 0.9,
        zoom: 0.76,
        ..Default::default()
    };
    let forward = view.basis()[2];
    view.center = std::array::from_fn(|i| -forward[i] * 150.);
    view
}

/// The README names the sources this flavor actually wrote.
fn readme(flavor: GameplayFlavor) -> String {
    let source = match flavor {
        GameplayFlavor::Cpp => concat!(
            "The character controller is project-owned C++ in\n",
            "`assets/scripts/ThirdPersonController.hpp` and `.cpp`. The key and the door are\n",
            "`assets/scripts/DungeonKey.hpp` and `assets/scripts/DungeonDoor.hpp`: a key\n",
            "deactivates when touched, a door when the key it names is gone."
        ),
        GameplayFlavor::Blueprint => concat!(
            "The character controller, the key and the door are project-owned visual graphs in\n",
            "`assets/Blueprints/`: `ThirdPersonController`, `DungeonKey` (deactivates when\n",
            "touched) and `DungeonDoor` (deactivates when the key it names is gone). The build\n",
            "backend turns the graphs into native console code."
        ),
        GameplayFlavor::Lua => concat!(
            "The character controller, the key and the door are project-owned Lua in\n",
            "`assets/scripts/`: `ThirdPersonController.lua`, `DungeonKey.lua` (deactivates when\n",
            "touched) and `DungeonDoor.lua` (deactivates when the key it names is gone). They\n",
            "are compiled with this project's Lua execution setting."
        ),
    };
    include_str!("../resources/templates/dungeon/README.md").replace("<!--GAMEPLAY-->", source)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::{CreateOptions, Template};

    /// Every flavor creates a dungeon the character starts in, with a working key and
    /// door of that flavor, inside the console budget, and whose classes compile.
    #[test]
    fn every_flavor_creates_a_playable_dungeon() {
        for flavor in GameplayFlavor::ALL {
            let parent = std::env::temp_dir()
                .join(format!("epok-dungeon-template-{}", uuid::Uuid::new_v4()));
            let root = parent.join("Dungeon Game");
            crate::workspace::create_with_options(
                &root,
                "Dungeon Game",
                CreateOptions::new(Template::Dungeon).with_gameplay(flavor),
            )
            .unwrap_or_else(|e| panic!("{flavor:?} project creation: {e}"));
            let scene = Scene::load(&crate::workspace::startup_scene(&root).unwrap()).unwrap();
            assert!(!scene.actors.iter().any(|a| a.name == "Environment"));
            let g = &scene.dungeons[0];
            assert!(!g.output.is_empty() && !g.stale(), "{flavor:?}");
            // The character stands in the spawn room.
            let player = scene.actors.iter().find(|a| a.name == "Player").unwrap();
            let spawn = scene.actors.iter().find(|a| a.name == "Spawn").unwrap();
            for k in 0..3 {
                assert!(
                    (player.position[k] - spawn.position[k]).abs() < 1e-3,
                    "{flavor:?}: player at {:?}, spawn at {:?}",
                    player.position,
                    spawn.position
                );
            }
            // The key and the door are this flavor's, and the door names the key.
            let classes = crate::dungeon_gameplay::classes(flavor);
            let with = |class: &str| {
                scene
                    .actors
                    .iter()
                    .filter(|a| {
                        a.components
                            .iter()
                            .any(|c| c.class.class_id.as_deref() == Some(class))
                    })
                    .collect::<Vec<_>>()
            };
            let (keys, doors) = (with(classes.key.1), with(classes.door.1));
            assert_eq!(keys.len(), 1, "{flavor:?}");
            // Only the character picks the key up.
            let named = keys[0]
                .components
                .iter()
                .find_map(|c| c.properties.get(crate::dungeon_gameplay::PLAYER))
                .and_then(|v| v.as_str());
            assert_eq!(named, Some(player.id.to_string().as_str()), "{flavor:?}");
            assert!(!doors.is_empty(), "{flavor:?}");
            for door in doors {
                let named = door
                    .components
                    .iter()
                    .find_map(|c| c.properties.get(crate::dungeon_gameplay::KEY_REFERENCE))
                    .and_then(|v| v.as_str());
                assert_eq!(named, Some(keys[0].id.to_string().as_str()));
            }
            // Within the console budget, lit and ready.
            assert!(
                scene.actors.len() <= 64,
                "{flavor:?}: {} actors",
                scene.actors.len()
            );
            let quads: usize = scene.actors.iter().map(crate::lighting::quad_count).sum();
            assert!(
                quads <= crate::dungeon_bake::SCENE_QUADS,
                "{flavor:?}: {quads} quads"
            );
            assert!(crate::lighting::valid_bake(&scene), "{flavor:?}");
            // Every class the scene binds resolves.
            let catalog = crate::scripts::catalog(&root)
                .unwrap_or_else(|e| panic!("{flavor:?} script catalog: {e}"));
            for name in ["ThirdPersonController", classes.key.0, classes.door.0] {
                assert!(
                    catalog
                        .iter()
                        .flat_map(|s| s.classes.iter())
                        .any(|c| c.cpp_name == name),
                    "{flavor:?} catalog has no {name}"
                );
            }
            // One room of the loop is the sample handmade room, built in its workbench map.
            assert!(root.join(WORKBENCH).is_file());
            let reactors = scene
                .actors
                .iter()
                .filter(|a| a.name.starts_with("Reactor light"))
                .count();
            assert_eq!(reactors, 1, "{flavor:?}: {reactors} reactor rooms");
            let readme = std::fs::read_to_string(root.join("README.md")).unwrap();
            assert!(readme.contains("DungeonDoor") && !readme.contains("<!--"));
            std::fs::remove_dir_all(parent).unwrap();
        }
    }
}
