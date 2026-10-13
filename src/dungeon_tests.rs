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
    let report = dungeon_bake::generate(&root, &mut scene, id).unwrap();
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
    dungeon_bake::generate(&root, &mut scene, id).unwrap();
    let first = serde_json::to_value(&scene).unwrap();
    let count = scene.actors.len();
    // Same seed again: byte-identical document.
    dungeon_bake::generate(&root, &mut scene, id).unwrap();
    assert_eq!(first, serde_json::to_value(&scene).unwrap());
    // A new seed replaces the output instead of adding to it.
    scene.dungeons[0].seed = 9;
    assert!(scene.dungeons[0].stale());
    dungeon_bake::generate(&root, &mut scene, id).unwrap();
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
    dungeon_bake::generate(&root, &mut scene, id).unwrap();
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
    let error = dungeon_bake::generate(&root, &mut scene, id).unwrap_err();
    assert!(error.contains("254 units"), "{error}");
    let (mut scene, id) = scene_3d(1);
    scene.actors[0].rotation = [0., 30., 0.];
    let error = dungeon_bake::generate(&root, &mut scene, id).unwrap_err();
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
    let report = dungeon_bake::generate(&root, &mut scene, id).unwrap();
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
                .filter(|p| dungeon_bake::wall_cell(&layout, *p))
                .count()
    );
    // A second 2D generation reuses the same built-in TileSet.
    scene.dungeons[0].tile_set = None;
    dungeon_bake::generate(&root, &mut scene, id).unwrap();
    assert_eq!(scene.dungeons[0].tile_set, Some(set_id));
    // The layer cooks into the runtime's chunked tile tables.
    crate::tilemap::cook(&cells, &set).unwrap();
    let index = crate::assets::scan(&root, &mut Default::default());
    crate::tileset::resolve(&mut scene, &index).unwrap();
    crate::texture::resolve(&mut scene, &index).unwrap();
    std::fs::remove_dir_all(parent).unwrap();
}
