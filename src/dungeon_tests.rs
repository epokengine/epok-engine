//! End-to-end checks of the Dungeon Generator: generate into a real project, then cook,
//! bake navigation and regenerate.
use crate::dungeon::{Generator, SpotKind};
use crate::dungeon_bake::{self, SCENE_QUADS};
use crate::scene::{Actor, Scene};
use std::path::PathBuf;
use uuid::Uuid;

fn project(name: &str) -> (PathBuf, PathBuf) {
    let template = crate::workspace::Template::Basic;
    let parent = std::env::temp_dir().join(format!("epok-dungeon-{name}-{}", Uuid::new_v4()));
    let root = parent.join("Dungeon Test");
    crate::workspace::create_with_options(
        &root,
        "Dungeon Test",
        crate::workspace::CreateOptions::new(template),
    )
    .unwrap();
    (parent, root)
}

/// A map with one 3D generator; returns the generator id.
fn scene_3d(seed: u32) -> (Scene, Uuid) {
    let mut scene = Scene::default();
    scene.actors.clear();
    let mut root = Actor::cube("Dungeon".into());
    root.kind = "Empty".into();
    root.position = [0.; 3];
    let id = root.id;
    scene.actors.push(root);
    let mut generator = Generator::new(id);
    generator.id = Uuid::from_u128(0xd00d);
    generator.seed = seed;
    let gid = generator.id;
    scene.dungeons.push(generator);
    scene.refresh_actor_hierarchy();
    (scene, gid)
}

#[test]
fn a_3d_dungeon_bakes_into_a_few_actors_that_cook() {
    let (parent, root) = project("3d");
    let (mut scene, id) = scene_3d(3);
    let report = dungeon_bake::generate(&root, &mut scene, id, None).unwrap();
    let generator = scene.dungeons[0].clone();
    // Geometry, collision, navigation volumes, spawn, goal and a light: far below the
    // 64 actors a map may hold.
    assert!(report.actors <= 12, "{report:?}");
    let branch = scene.actor_branch(generator.root);
    assert_eq!(branch.len() - 1, report.actors);
    assert!(
        report.quads > 100 && report.quads < SCENE_QUADS,
        "{report:?}"
    );
    assert!(
        report.boxes > 10 && report.navigation_nodes > 0,
        "{report:?}"
    );
    assert!(!generator.stale());
    let collider = scene
        .actors
        .iter()
        .find_map(|a| a.collider.as_ref().filter(|c| !c.boxes.is_empty()))
        .expect("compound collider");
    crate::collision::validate_component(collider).unwrap();
    // The generated assets are real EditableMesh packages.
    let index = crate::assets::scan(&root, &mut Default::default());
    for asset in &generator.output.assets {
        let record = index.resolve(*asset).unwrap();
        crate::mesh::document(record).unwrap();
        assert!(record.path.starts_with(root.join("assets/Dungeons")));
    }
    // Navigation bakes over the compound floor, inside the node budget.
    let bake = crate::navigation::bake(&scene).unwrap();
    assert!(
        !bake.nodes.is_empty() && bake.nodes.len() <= 512,
        "{}",
        bake.nodes.len()
    );
    // The cooker accepts the map, compound collider included. Unit tests run without
    // the reflected engine headers, so the bake volumes' reflected component is left
    // to the editor build check.
    let mut cooked = scene.clone();
    for actor in &mut cooked.actors {
        actor
            .components
            .retain(|c| c.class.class_id.as_deref() != Some(crate::navigation::VOLUME));
    }
    let catalog = crate::scripts::catalog(&root).unwrap();
    let header = crate::project::scene_header(&cooked, &catalog).unwrap();
    assert!(header.contains("collider_boxes_"));
    std::fs::remove_dir_all(parent).unwrap();
}

#[test]
fn regenerating_replaces_the_output_and_is_deterministic() {
    let (parent, root) = project("regen");
    let (mut scene, id) = scene_3d(8);
    dungeon_bake::generate(&root, &mut scene, id, None).unwrap();
    let first = serde_json::to_value(&scene).unwrap();
    let count = scene.actors.len();
    // Same seed again: byte-identical document.
    dungeon_bake::generate(&root, &mut scene, id, None).unwrap();
    assert_eq!(first, serde_json::to_value(&scene).unwrap());
    // A new seed replaces the output instead of adding to it.
    scene.dungeons[0].seed = 9;
    assert!(scene.dungeons[0].stale());
    dungeon_bake::generate(&root, &mut scene, id, None).unwrap();
    assert_eq!(scene.actors.len(), count);
    assert_ne!(first, serde_json::to_value(&scene).unwrap());
    // Clearing removes exactly the output and keeps the generator.
    dungeon_bake::clear(&mut scene, id);
    assert_eq!(scene.actors.len(), 1);
    assert!(scene.dungeons[0].output.is_empty());
    std::fs::remove_dir_all(parent).unwrap();
}

#[test]
fn spawn_actor_moves_to_the_spawn_room() {
    let (parent, root) = project("spawn");
    let (mut scene, id) = scene_3d(4);
    let mut player = Actor::cube("Player".into());
    player.position = [50., 0., 50.];
    let player_id = player.id;
    scene.actors.push(player);
    scene.dungeons[0].spawn_actor = Some(player_id);
    scene.actors[0].position = [10., 0., -4.];
    dungeon_bake::generate(&root, &mut scene, id, None).unwrap();
    let layout = crate::dungeon::layout(&scene.dungeons[0]).unwrap();
    let spawn = crate::dungeon::spots(&layout)
        .into_iter()
        .find(|s| s.kind == SpotKind::Spawn)
        .unwrap();
    let frame = dungeon_bake::Frame {
        size: layout.size,
        cell: scene.dungeons[0].cell_size,
    };
    let expected = frame.center(spawn.cell);
    let player = scene.actors.iter().find(|a| a.id == player_id).unwrap();
    assert!((player.position[0] - (expected[0] + 10.)).abs() < 1e-4);
    assert!((player.position[2] - (expected[2] - 4.)).abs() < 1e-4);
    std::fs::remove_dir_all(parent).unwrap();
}

#[test]
fn rotated_generators_and_oversized_grids_are_refused_with_a_reason() {
    let (parent, root) = project("refuse");
    let (mut scene, id) = scene_3d(1);
    scene.dungeons[0].rooms.size = [128, 128];
    scene.dungeons[0].cell_size = 4.;
    let error = dungeon_bake::generate(&root, &mut scene, id, None).unwrap_err();
    assert!(error.contains("254 units"), "{error}");
    let (mut scene, id) = scene_3d(1);
    scene.actors[0].rotation = [0., 30., 0.];
    let error = dungeon_bake::generate(&root, &mut scene, id, None).unwrap_err();
    assert!(error.contains("must not rotate or scale"), "{error}");
    std::fs::remove_dir_all(parent).unwrap();
}

#[test]
fn a_2d_dungeon_paints_one_tile_layer_with_solid_walls() {
    let (parent, root) = project("2d");
    let mut scene = Scene::default();
    scene.actors.clear();
    let actor = crate::world2d::create(crate::world2d::Preset::Node2D, "Dungeon", [0., 0.]);
    let root_id = actor.id;
    scene.actors.push(actor);
    let mut generator = Generator::new(root_id);
    generator.seed = 12;
    let id = generator.id;
    scene.dungeons.push(generator);
    scene.refresh_actor_hierarchy();
    let report = dungeon_bake::generate(&root, &mut scene, id, None).unwrap();
    assert_eq!(report.actors, 3, "{report:?}");
    let set_id = scene.dungeons[0].tile_set.expect("built-in tiles chosen");
    let set = scene.tilesets.get(&set_id).unwrap().clone();
    let layer = scene
        .actors
        .iter()
        .find_map(|a| crate::world2d::component(a, crate::object_model::TILEMAP_LAYER_COMPONENT_ID))
        .map(crate::world2d::read::<crate::world2d::TileMapLayer>)
        .unwrap();
    let cells = crate::tilemap::decode(&layer.cells).unwrap();
    let layout = crate::dungeon::layout(&scene.dungeons[0]).unwrap();
    let offset = [layout.size[0] / 2, layout.size[1] / 2];
    for (p, cell) in &cells {
        let tile = set.tile(cell.atlas).unwrap();
        let walkable = layout.at([p[0] + offset[0], p[1] + offset[1]]).walkable();
        assert_eq!(tile.terrain, Some(if walkable { 0 } else { 1 }));
        assert_eq!(
            tile.collision == crate::tileset::Collision::Full,
            !walkable,
            "{p:?}"
        );
    }
    assert_eq!(
        cells.len(),
        layout.walkable_cells()
            + (0..layout.size[1])
                .flat_map(|y| (0..layout.size[0]).map(move |x| [x, y]))
                .filter(|p| layout.solid(*p))
                .count()
    );
    // A second 2D generation reuses the same built-in TileSet.
    scene.dungeons[0].tile_set = None;
    dungeon_bake::generate(&root, &mut scene, id, None).unwrap();
    assert_eq!(scene.dungeons[0].tile_set, Some(set_id));
    // The layer cooks into the runtime's chunked tile tables.
    crate::tilemap::cook(&cells, &set).unwrap();
    let index = crate::assets::scan(&root, &mut Default::default());
    crate::tileset::resolve(&mut scene, &index).unwrap();
    crate::texture::resolve(&mut scene, &index).unwrap();
    std::fs::remove_dir_all(parent).unwrap();
}

/// A 3D style: the built-in blockout plus a mesh pillar in every corner and three
/// baked lights spread over room floors.
fn pillar_style(root: &std::path::Path) -> (Uuid, Uuid) {
    use crate::dungeon_style::{self, Filter, Node, NodeKind, Piece, Visual};
    let mut pillar = crate::mesh::Document::default();
    let (group, slot) = (pillar.groups[0].id, pillar.materials[0].id);
    pillar.primitive("Box", [0., 1., 0.], [0.4, 2., 0.4], 1, group, slot);
    let mesh = crate::mesh::create(root, "assets/Meshes/Pillar.epokasset", &pillar).unwrap();
    let mut style = dungeon_style::builtin(false, None);
    let corner = Uuid::from_u128(0xc0);
    style.nodes.push(Node {
        id: corner,
        position: [0., 800.],
        kind: NodeKind::Spot {
            spot: SpotKind::Corner,
        },
    });
    style.nodes.push(Node {
        id: Uuid::from_u128(0xc1),
        position: [300., 800.],
        kind: NodeKind::Piece {
            parent: corner,
            visual: Visual {
                piece: Piece::Mesh { asset: Some(mesh) },
                ..Visual::default()
            },
        },
    });
    let floor = style.spot_node(SpotKind::Floor).unwrap().id;
    style.nodes.push(Node {
        id: Uuid::from_u128(0xc2),
        position: [-300., 0.],
        kind: NodeKind::Piece {
            parent: floor,
            visual: Visual {
                piece: Piece::Light {
                    color: [1., 0.7, 0.4],
                    intensity: 1.,
                    range: 6.,
                },
                consume: false,
                filter: Filter::Rooms,
                limit: 3,
                ..Visual::default()
            },
        },
    });
    let path = dungeon_style::fresh_path(root, "Crypt Style");
    (dungeon_style::create(root, &path, &style).unwrap(), mesh)
}

#[test]
fn styles_merge_meshes_place_lights_and_mark_the_output_stale() {
    let (parent, root) = project("style");
    let (style_id, _) = pillar_style(&root);
    let (mut scene, id) = scene_3d(21);
    let plain = dungeon_bake::generate(&root, &mut scene, id, None).unwrap();
    scene.dungeons[0].style = Some(style_id);
    assert!(
        scene.dungeons[0].stale(),
        "choosing a style is a settings change"
    );
    let styled = dungeon_bake::generate(&root, &mut scene, id, None).unwrap();
    // Corner pillars are merged geometry: more quads, the same section count.
    assert!(styled.quads > plain.quads, "{plain:?} {styled:?}");
    // Three baked point lights replace the default sun.
    let lights: Vec<_> = scene
        .actors
        .iter()
        .filter_map(|a| a.light.as_ref())
        .collect();
    assert_eq!(lights.len(), 3);
    assert!(
        lights
            .iter()
            .all(|l| l.kind == crate::lighting::LightType::Point
                && l.mode == crate::lighting::LightMode::Baked)
    );
    let index = crate::assets::scan(&root, &mut Default::default());
    let section = index.resolve(scene.dungeons[0].output.assets[0]).unwrap();
    let document = crate::mesh::document(section).unwrap();
    assert!(document.groups.iter().any(|g| g.name == "Pieces"));
    // Editing the style marks the output stale until it is regenerated.
    let (style, digest) = dungeon_bake::style_of(&index, &scene.dungeons[0], false).unwrap();
    assert!(!scene.dungeons[0].style_stale(&digest));
    let mut edited = style.clone();
    if let crate::dungeon_style::NodeKind::Piece { visual, .. } = &mut edited
        .nodes
        .iter_mut()
        .find(|n| n.id == Uuid::from_u128(0xc2))
        .unwrap()
        .kind
    {
        visual.limit = 1;
    }
    crate::dungeon_style::save(index.resolve(style_id).unwrap(), &edited).unwrap();
    let index = crate::assets::scan(&root, &mut Default::default());
    let (_, digest) = dungeon_bake::style_of(&index, &scene.dungeons[0], false).unwrap();
    assert!(scene.dungeons[0].style_stale(&digest));
    dungeon_bake::generate(&root, &mut scene, id, None).unwrap();
    assert_eq!(scene.actors.iter().filter(|a| a.light.is_some()).count(), 1);
    // A 3D style cannot dress a 2D generator.
    let actor = crate::world2d::create(crate::world2d::Preset::Node2D, "Flat", [0., 0.]);
    let mut flat = Generator::new(actor.id);
    flat.style = Some(style_id);
    let flat_id = flat.id;
    scene.actors.push(actor);
    scene.dungeons.push(flat);
    let error = dungeon_bake::generate(&root, &mut scene, flat_id, None).unwrap_err();
    assert!(error.contains("for 3D dungeons"), "{error}");
    std::fs::remove_dir_all(parent).unwrap();
}

#[test]
fn actor_pieces_need_classes_and_2d_tiles_fill_a_decoration_layer() {
    use crate::dungeon_style::{self, Node, NodeKind, Piece, Visual};
    let (parent, root) = project("pieces");
    // Actor pieces without a class model are refused before anything changes.
    let mut style = dungeon_style::builtin(false, None);
    let goal = Uuid::from_u128(0xa0);
    style.nodes.push(Node {
        id: goal,
        position: [0., 900.],
        kind: NodeKind::Spot {
            spot: SpotKind::Goal,
        },
    });
    style.nodes.push(Node {
        id: Uuid::from_u128(0xa1),
        position: [300., 900.],
        kind: NodeKind::Piece {
            parent: goal,
            visual: Visual {
                piece: Piece::Actor {
                    class: Some("epok::Actor3D".into()),
                },
                ..Visual::default()
            },
        },
    });
    let path = dungeon_style::fresh_path(&root, "With Actors");
    let style_id = dungeon_style::create(&root, &path, &style).unwrap();
    let (mut scene, id) = scene_3d(4);
    scene.dungeons[0].style = Some(style_id);
    let before = scene.clone();
    let error = dungeon_bake::generate(&root, &mut scene, id, None).unwrap_err();
    assert!(error.contains("class model"), "{error}");
    assert_eq!(scene.actors.len(), before.actors.len());
    // 2D: bones on a fifth of the room floors, on their own layer above the tiles.
    let mut flat = dungeon_style::builtin(true, None);
    let floor = flat.spot_node(SpotKind::Floor).unwrap().id;
    flat.nodes.push(Node {
        id: Uuid::from_u128(0xb1),
        position: [600., 0.],
        kind: NodeKind::Piece {
            parent: floor,
            visual: Visual {
                piece: Piece::Tile {
                    tile_set: None,
                    atlas: [3, 0],
                },
                chance: 20,
                consume: false,
                filter: crate::dungeon_style::Filter::Rooms,
                ..Visual::default()
            },
        },
    });
    let path = dungeon_style::fresh_path(&root, "Bones");
    let flat_id = dungeon_style::create(&root, &path, &flat).unwrap();
    let mut scene = Scene::default();
    scene.actors.clear();
    let actor = crate::world2d::create(crate::world2d::Preset::Node2D, "Dungeon", [0., 0.]);
    let mut generator = Generator::new(actor.id);
    generator.style = Some(flat_id);
    let gid = generator.id;
    scene.actors.push(actor);
    scene.dungeons.push(generator);
    scene.refresh_actor_hierarchy();
    let report = dungeon_bake::generate(&root, &mut scene, gid, None).unwrap();
    assert_eq!(report.actors, 4, "{report:?}");
    let layers: Vec<crate::world2d::TileMapLayer> = scene
        .actors
        .iter()
        .filter_map(|a| {
            crate::world2d::component(a, crate::object_model::TILEMAP_LAYER_COMPONENT_ID)
        })
        .map(crate::world2d::read)
        .collect();
    assert_eq!(layers.len(), 2);
    let details = layers.iter().find(|l| !l.collision_enabled).unwrap();
    let floor_tiles =
        crate::tilemap::decode(&layers.iter().find(|l| l.collision_enabled).unwrap().cells)
            .unwrap()
            .len();
    let bones = crate::tilemap::decode(&details.cells).unwrap().len();
    assert!(
        bones > 0 && bones * 3 < floor_tiles,
        "{bones} of {floor_tiles}"
    );
    assert!(details.z_index > 0);
    std::fs::remove_dir_all(parent).unwrap();
}

/// Writes a Third Person project with a styled Dungeon Generator for screenshots:
/// `EPOK_DUNGEON_FIXTURE=/tmp/Crypt cargo test dungeon_visual_fixture -- --ignored`,
/// then `epok-editor --project /tmp/Crypt --generate-dungeon Dungeon`, which places the
/// style's Actor piece with the project's class model.
#[test]
#[ignore]
fn dungeon_visual_fixture() {
    use crate::dungeon_style::{self, Filter, Node, NodeKind, Piece, Visual};
    let Ok(root) = std::env::var("EPOK_DUNGEON_FIXTURE") else {
        return;
    };
    let root = PathBuf::from(root);
    crate::workspace::create_with_options(
        &root,
        "Crypt",
        crate::workspace::CreateOptions::new(crate::workspace::Template::ThirdPerson),
    )
    .unwrap();
    let path = crate::workspace::startup_scene(&root).unwrap();
    let mut scene = Scene::load(&path).unwrap();
    // The dungeon replaces the arena; inactive actors would still be cooked.
    scene.sync_actor_components();
    if let Some(environment) = scene
        .actors
        .iter()
        .find(|a| a.name == "Environment")
        .map(|a| a.id)
    {
        scene.delete_actor_branch(environment);
    }
    let box_mesh = |name: &str, size: [f32; 3], center: [f32; 3], color: [f32; 3]| {
        let mut doc = crate::mesh::Document::default();
        doc.materials[0].material.color = color;
        let (group, slot) = (doc.groups[0].id, doc.materials[0].id);
        doc.primitive("Box", center, size, 1, group, slot);
        crate::mesh::create(&root, &format!("assets/Meshes/{name}.epokasset"), &doc).unwrap()
    };
    let crate_mesh = box_mesh("Crate", [0.8, 0.8, 0.8], [0., 0.4, 0.], [0.55, 0.38, 0.22]);
    let banner = box_mesh(
        "Banner",
        [0.9, 1.4, 0.06],
        [0., 1.6, 0.04],
        [0.55, 0.12, 0.14],
    );
    let mut style = dungeon_style::builtin(false, None);
    let piece = |id: u128, parent: Uuid, x: f32, y: f32, visual: Visual| Node {
        id: Uuid::from_u128(id),
        position: [x, y],
        kind: NodeKind::Piece { parent, visual },
    };
    let floor = style.spot_node(SpotKind::Floor).unwrap().id;
    let wall = style.spot_node(SpotKind::Wall).unwrap().id;
    let corner = Uuid::from_u128(0x100);
    style.nodes.push(Node {
        id: corner,
        position: [0., 560.],
        kind: NodeKind::Spot {
            spot: SpotKind::Corner,
        },
    });
    style.nodes.push(piece(
        0x101,
        corner,
        260.,
        560.,
        Visual {
            piece: Piece::Blockout {
                color: [0.42, 0.38, 0.36],
                texture: None,
            },
            ..Visual::default()
        },
    ));
    let room = Uuid::from_u128(0x110);
    style.nodes.push(Node {
        id: room,
        position: [0., 700.],
        kind: NodeKind::Spot {
            spot: SpotKind::Room,
        },
    });
    style.nodes.push(piece(
        0x102,
        room,
        260.,
        700.,
        Visual {
            piece: Piece::Light {
                color: [1., 0.66, 0.36],
                intensity: 1.6,
                range: 9.,
            },
            offset: [0., 2.2, 0.],
            ..Visual::default()
        },
    ));
    style.nodes.push(piece(
        0x103,
        floor,
        720.,
        0.,
        Visual {
            piece: Piece::Mesh {
                asset: Some(crate_mesh),
            },
            chance: 5,
            filter: Filter::Rooms,
            rotation: [0., 20., 0.],
            ..Visual::default()
        },
    ));
    style.nodes.push(piece(
        0x104,
        wall,
        490.,
        140.,
        Visual {
            piece: Piece::Mesh {
                asset: Some(banner),
            },
            chance: 12,
            consume: false,
            filter: Filter::Rooms,
            ..Visual::default()
        },
    ));
    let goal = Uuid::from_u128(0x120);
    style.nodes.push(Node {
        id: goal,
        position: [0., 840.],
        kind: NodeKind::Spot {
            spot: SpotKind::Goal,
        },
    });
    style.nodes.push(piece(
        0x121,
        goal,
        260.,
        840.,
        Visual {
            piece: Piece::Actor {
                class: Some("epok::Actor3D".into()),
            },
            ..Visual::default()
        },
    ));
    let style_path = dungeon_style::fresh_path(&root, "Crypt");
    let style_id = dungeon_style::create(&root, &style_path, &style).unwrap();
    let mut actor = crate::scene::Actor::cube("Dungeon".into());
    actor.kind = "Empty".into();
    actor.position = [0.; 3];
    let actor_id = actor.id;
    scene.actors.push(actor);
    let mut generator = Generator::new(actor_id);
    generator.seed = 1207;
    generator.style = Some(style_id);
    generator.spawn_actor = scene
        .actors
        .iter()
        .find(|a| a.name == "Player")
        .map(|a| a.id);
    scene.dungeons.push(generator);
    scene.refresh_actor_hierarchy();
    scene.save(&path).unwrap();
}
