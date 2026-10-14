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
    // 64 actors a map may hold (each costs runtime memory).
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
    // The built-in textures resolve like any imported texture.
    let textures = crate::assets::scan(&root, &mut Default::default());
    crate::mesh::resolve(&mut cooked, &textures).unwrap();
    crate::texture::resolve(&mut cooked, &textures).unwrap();
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
    let layout = crate::dungeon::layout(&scene.dungeons[0], &Default::default()).unwrap();
    let spawn = crate::dungeon::spots(&layout)
        .into_iter()
        .find(|s| s.kind == SpotKind::Spawn)
        .unwrap();
    let frame = dungeon_bake::Frame {
        size: layout.size,
        cell: scene.dungeons[0].cell_size,
        height: scene.dungeons[0].wall_height,
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
    let layout = crate::dungeon::layout(&scene.dungeons[0], &Default::default()).unwrap();
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
    // Only this style's own lights, not the built-in room lights.
    style.nodes.retain(|n| {
        !matches!(&n.kind, NodeKind::Piece { visual, .. } if matches!(visual.piece, Piece::Light { .. }))
    });
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
    // Three baked point lights, on top of the dungeon's own sky light.
    let lights: Vec<_> = scene
        .actors
        .iter()
        .filter_map(|a| a.light.as_ref())
        .filter(|l| l.kind != crate::lighting::LightType::Directional)
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
    assert_eq!(scene.actors.iter().filter(|a| a.light.is_some()).count(), 2);
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

/// The actors carrying component `class` and the lock number each was given.
fn gameplay_actors(scene: &Scene, class: (&str, &str)) -> Vec<(usize, u64)> {
    scene
        .actors
        .iter()
        .enumerate()
        .filter_map(|(i, a)| {
            let component = a
                .components
                .iter()
                .find(|c| c.class.class_id.as_deref() == Some(class.1))?;
            Some((
                i,
                component.properties[dungeon_bake::LOCK_PROPERTY].as_u64()?,
            ))
        })
        .collect()
}

#[test]
fn recipe_dungeons_bake_markers_within_budget_and_cook() {
    use crate::dungeon::Source;
    let (parent, root) = project("recipe");
    for seed in [1, 7, 21] {
        let (mut scene, id) = scene_3d(seed);
        scene.dungeons[0].source = Source::Recipe;
        let report = dungeon_bake::generate(&root, &mut scene, id, None).unwrap();
        // The first generation installs the key and door scripts, once.
        assert_eq!(
            report.notes.iter().any(|n| n.contains("DungeonKey")),
            seed == 1,
            "{report:?}"
        );
        for (path, bytes) in dungeon_bake::GAMEPLAY_SCRIPTS {
            assert_eq!(std::fs::read(root.join(path)).unwrap(), bytes);
        }
        // The built-in textures are imported under their fixed identities.
        let index = crate::assets::scan(&root, &mut Default::default());
        for texture in dungeon_bake::BuiltinTexture::ALL {
            let record = index.resolve(texture.id()).unwrap();
            assert_eq!(record.meta.kind, crate::assets::Kind::Texture);
        }
        assert!(report.summary.contains("key and lock"), "{report:?}");
        // A key and a door on top of the geometry: well inside 64.
        assert!(report.actors <= 20, "{report:?}");
        assert!(report.quads < SCENE_QUADS, "{report:?}");
        assert!(report.navigation_nodes > 0, "{report:?}");
        let g = &scene.dungeons[0];
        assert!(!g.output.recipe.is_empty());
        assert!(!g.recipe_stale(&crate::dungeon_recipe::digest(
            &crate::dungeon_recipe::builtin()
        )));
        // The built-in style marks key and item pedestals with their own materials.
        let index = crate::assets::scan(&root, &mut Default::default());
        let names: std::collections::BTreeSet<String> = g
            .output
            .assets
            .iter()
            .filter_map(|a| crate::mesh::document(index.resolve(*a).ok()?).ok())
            .flat_map(|d| d.materials.into_iter().map(|m| m.name))
            .collect();
        for marker in ["Key", "Item"] {
            assert!(names.contains(marker), "seed {seed}: {names:?}");
        }
        // Working keys and doors: a floating trigger and a solid slab per lock, each
        // carrying its lock number.
        let layout = crate::dungeon::layout(g, &Default::default()).unwrap();
        let keys = gameplay_actors(&scene, dungeon_bake::KEY_CLASS);
        let doors = gameplay_actors(&scene, dungeon_bake::DOOR_CLASS);
        let locks: Vec<u64> = layout.locks.iter().map(|l| u64::from(l.lock)).collect();
        assert_eq!(doors.iter().map(|d| d.1).collect::<Vec<_>>(), locks);
        assert_eq!(
            keys.iter().map(|k| k.1).collect::<Vec<_>>(),
            layout
                .items
                .iter()
                .filter(|i| i.lock > 0)
                .map(|i| u64::from(i.lock))
                .collect::<Vec<_>>()
        );
        for (index, _) in &keys {
            let key = &scene.actors[*index];
            let collider = key.collider.as_ref().unwrap();
            assert!(collider.trigger);
            let bottom = key.position[1] - collider.half_extents[1] * key.scale[1];
            assert!(
                bottom > 0.2,
                "the key's trigger touches the floor: {bottom}"
            );
            assert!(g.output.actors.contains(&key.id));
        }
        for (index, _) in &doors {
            let door = &scene.actors[*index];
            assert!(!door.collider.as_ref().unwrap().trigger);
            assert!((door.scale[0] - g.cell_size).abs() < 1e-4);
            assert!((door.scale[1] - g.door_height).abs() < 1e-4);
        }
    }
    // A recipe asset drives the layout, and editing it marks the output stale.
    let mut recipe = crate::dungeon_recipe::builtin();
    let path = crate::dungeon_recipe::fresh_path(&root, "Crypt Route");
    let recipe_id = crate::dungeon_recipe::create(&root, &path, &recipe).unwrap();
    let (mut scene, id) = scene_3d(5);
    scene.dungeons[0].source = Source::Recipe;
    scene.dungeons[0].recipe = Some(recipe_id);
    dungeon_bake::generate(&root, &mut scene, id, None).unwrap();
    recipe
        .steps
        .retain(|s| !matches!(s, crate::dungeon_recipe::Step::OneWay { .. }));
    let index = crate::assets::scan(&root, &mut Default::default());
    crate::dungeon_recipe::save(index.resolve(recipe_id).unwrap(), &recipe).unwrap();
    let index = crate::assets::scan(&root, &mut Default::default());
    let (_, digest) = crate::dungeon_recipe::of(&index, &scene.dungeons[0]).unwrap();
    assert!(scene.dungeons[0].recipe_stale(&digest));
    // The map cooks. Unit tests run without the header extractor, so the reflected
    // components (navigation volumes, keys and doors) are left to the editor build.
    let mut cooked = scene.clone();
    for actor in &mut cooked.actors {
        actor.components.retain(|c| {
            ![
                crate::navigation::VOLUME,
                dungeon_bake::KEY_CLASS.1,
                dungeon_bake::DOOR_CLASS.1,
            ]
            .contains(&c.class.class_id.as_deref().unwrap_or_default())
        });
    }
    for (path, _) in dungeon_bake::GAMEPLAY_SCRIPTS {
        std::fs::remove_file(root.join(path)).unwrap();
    }
    // The built-in textures resolve like any imported texture.
    let textures = crate::assets::scan(&root, &mut Default::default());
    crate::mesh::resolve(&mut cooked, &textures).unwrap();
    crate::texture::resolve(&mut cooked, &textures).unwrap();
    let catalog = crate::scripts::catalog(&root).unwrap();
    crate::project::scene_header(&cooked, &catalog).unwrap();
    // A missing recipe is reported, not replaced by the built-in one.
    scene.dungeons[0].recipe = Some(Uuid::from_u128(0xbad));
    let error = dungeon_bake::generate(&root, &mut scene, id, None).unwrap_err();
    assert!(error.contains("Dungeon Recipe is missing"), "{error}");
    std::fs::remove_dir_all(parent).unwrap();
}

#[test]
fn recipe_markers_turn_with_their_doors_in_2d() {
    let (parent, root) = project("recipe-2d");
    let mut scene = Scene::default();
    scene.actors.clear();
    let actor = crate::world2d::create(crate::world2d::Preset::Node2D, "Dungeon", [0., 0.]);
    let mut generator = Generator::new(actor.id);
    generator.source = crate::dungeon::Source::Recipe;
    generator.seed = 7;
    let gid = generator.id;
    scene.actors.push(actor);
    scene.dungeons.push(generator);
    scene.refresh_actor_hierarchy();
    dungeon_bake::generate(&root, &mut scene, gid, None).unwrap();
    // Generating pins the built-in tiles on the generator; its look stays current.
    let index = crate::assets::scan(&root, &mut Default::default());
    let (_, digest) = dungeon_bake::style_of(&index, &scene.dungeons[0], true).unwrap();
    assert!(scene.dungeons[0].tile_set.is_some());
    assert!(!scene.dungeons[0].style_stale(&digest));
    let layout = crate::dungeon::layout(&scene.dungeons[0], &Default::default()).unwrap();
    let details = scene
        .actors
        .iter()
        .filter_map(|a| {
            crate::world2d::component(a, crate::object_model::TILEMAP_LAYER_COMPONENT_ID)
        })
        .map(crate::world2d::read::<crate::world2d::TileMapLayer>)
        .find(|l| !l.collision_enabled)
        .expect("a marker layer");
    let cells = crate::tilemap::decode(&details.cells).unwrap();
    let offset = [layout.size[0] / 2, layout.size[1] / 2];
    let at = |p: [i32; 2]| cells.get(&[p[0] - offset[0], p[1] - offset[1]]);
    for item in layout.items.iter().filter(|i| i.lock == 0) {
        let cell = at(item.cell).expect("a marker on every item");
        assert_eq!(cell.atlas, [7, 0]);
    }
    // Keys and locked doors are working actors: a trigger with the key sprite and a
    // static body with the gate turned towards its corridor, both on their cells.
    let position = |index: usize| {
        let root = scene.actors[index]
            .components
            .iter()
            .find(|c| c.root)
            .unwrap();
        crate::scene_view_mode::read_transform_2d(&root.properties)
    };
    let pixel = |p: [i32; 2]| {
        [
            (p[0] - offset[0]) as f32 * 16. + 8.,
            (p[1] - offset[1]) as f32 * 16. + 8.,
        ]
    };
    let doors = gameplay_actors(&scene, dungeon_bake::DOOR_CLASS);
    assert_eq!(doors.len(), layout.locks.len());
    for ((index, lock), door) in doors.iter().zip(&layout.locks) {
        assert_eq!(*lock, u64::from(door.lock));
        let transform = position(*index);
        assert_eq!(transform.position, pixel(door.cell));
        let turn = match door.side {
            crate::dungeon::Side::North => 0.,
            crate::dungeon::Side::East => 90.,
            crate::dungeon::Side::South => 180.,
            crate::dungeon::Side::West => 270.,
        };
        assert_eq!(transform.rotation, turn);
    }
    let keys = gameplay_actors(&scene, dungeon_bake::KEY_CLASS);
    for ((index, lock), key) in keys.iter().zip(layout.items.iter().filter(|i| i.lock > 0)) {
        assert_eq!(*lock, u64::from(key.lock));
        assert_eq!(position(*index).position, pixel(key.cell));
        let collider: crate::world2d::Collider2D = crate::world2d::read(
            crate::world2d::component(&scene.actors[*index], crate::world2d::COLLIDER2D).unwrap(),
        );
        assert!(collider.trigger);
    }
    assert!(!keys.is_empty());
    // The project's built-in tiles hold the markers, and a new style keeps pointing at
    // them when its generator paints with another TileSet.
    let (tiles, set) = dungeon_bake::builtin_tiles(&root).unwrap();
    assert!(set.tile([6, 0]).is_some());
    let mut style = crate::dungeon_style::builtin(true, None);
    dungeon_bake::pin_markers(&root, &mut style).unwrap();
    assert!(style.references().contains(&tiles));
    std::fs::remove_dir_all(parent).unwrap();
}

/// The console clamps UVs, so every face of the built-in look maps its texture once;
/// the ceiling closes exactly the floor, with collision above it.
#[test]
fn textures_cover_each_face_once_and_the_ceiling_closes_the_floor() {
    let style = crate::dungeon_style::builtin(false, None);
    for ceiling in [true, false] {
        let g = Generator {
            seed: 4,
            source: crate::dungeon::Source::Recipe,
            ceiling,
            ..Generator::default()
        };
        let layout = crate::dungeon::layout(&g, &Default::default()).unwrap();
        let placements = crate::dungeon_style::evaluate(&style, &layout, g.seed);
        let plan = dungeon_bake::plan_3d(&g, &layout, &placements, &Default::default()).unwrap();
        let mut ceilings = 0;
        for section in &plan.sections {
            let d = &section.document;
            let group = |name: &str| d.groups.iter().find(|g| g.name == name).unwrap().id;
            for face in &d.faces {
                for uv in face.uv {
                    assert!(
                        uv.iter().all(|v| (-1e-4..=1.0001).contains(v)),
                        "uv {uv:?} outside 0..1"
                    );
                }
                if face.group == group("Ceilings") {
                    let y = d.vertices[face.vertices[0] as usize][1];
                    assert!((y - g.wall_height).abs() < 1e-4);
                    ceilings += 1;
                }
            }
        }
        assert_eq!(ceilings, if ceiling { layout.walkable_cells() } else { 0 });
        let above = plan
            .boxes
            .iter()
            .filter(|b| (b[1] - g.wall_height).abs() < 1e-4)
            .count();
        assert_eq!(above > 0, ceiling);
    }
    // The defaults fit the template character: 1.84 units tall.
    let g = Generator::default();
    assert_eq!(
        crate::dungeon::sizes_for(1.84),
        (g.cell_size, g.wall_height, g.door_height)
    );
}

/// Faces are drawn only from the front on the PSX: every vertical face of walls,
/// corner pillars and doorway beams must look into a walkable cell.
#[test]
fn wall_corner_and_doorway_faces_look_into_the_floor() {
    use crate::dungeon_style::{Node, NodeKind, Piece, Visual};
    let mut style = crate::dungeon_style::builtin(false, None);
    let corner = Uuid::from_u128(0xc0);
    style.nodes.push(Node {
        id: corner,
        position: [0., 900.],
        kind: NodeKind::Spot {
            spot: SpotKind::Corner,
        },
    });
    style.nodes.push(Node {
        id: Uuid::from_u128(0xc1),
        position: [300., 900.],
        kind: NodeKind::Piece {
            parent: corner,
            visual: Visual {
                piece: Piece::Blockout {
                    color: [0.4, 0.4, 0.4],
                    texture: None,
                },
                ..Visual::default()
            },
        },
    });
    for source in [
        crate::dungeon::Source::Rooms,
        crate::dungeon::Source::Recipe,
    ] {
        let mut g = Generator {
            seed: 11,
            source,
            ..Generator::default()
        };
        g.door_height = 2.;
        let layout = crate::dungeon::layout(&g, &Default::default()).unwrap();
        let placements = crate::dungeon_style::evaluate(&style, &layout, g.seed);
        let plan = dungeon_bake::plan_3d(&g, &layout, &placements, &Default::default()).unwrap();
        let frame = dungeon_bake::Frame {
            size: layout.size,
            cell: g.cell_size,
            height: g.wall_height,
        };
        let mut checked = 0;
        for section in &plan.sections {
            let d = &section.document;
            for face in &d.faces {
                let group = &d.groups.iter().find(|g| g.id == face.group).unwrap().name;
                if group != "Walls" && group != "Doorways" && group != "Pieces" {
                    continue;
                }
                let p = face.vertices.map(|v| {
                    let v = d.vertices[v as usize];
                    [v[0] + section.origin[0], v[1], v[2] + section.origin[2]]
                });
                let (a, b) = (
                    [p[1][0] - p[0][0], p[1][1] - p[0][1], p[1][2] - p[0][2]],
                    [p[2][0] - p[0][0], p[2][1] - p[0][1], p[2][2] - p[0][2]],
                );
                let n = [
                    a[1] * b[2] - a[2] * b[1],
                    a[2] * b[0] - a[0] * b[2],
                    a[0] * b[1] - a[1] * b[0],
                ];
                if n[1].abs() > 1e-4 {
                    continue; // tops and beams' undersides
                }
                let length = (n[0] * n[0] + n[2] * n[2]).sqrt();
                let centre = [0, 2].map(|k| p.iter().map(|v| v[k]).sum::<f32>() / 4.);
                if group == "Pieces" {
                    // Key and item pedestals stand in the middle of their cell and
                    // show their outside; gates are seen from both sides.
                    let cell = [
                        (centre[0] / frame.cell + layout.size[0] as f32 * 0.5).floor() as i32,
                        (centre[1] / frame.cell + layout.size[1] as f32 * 0.5).floor() as i32,
                    ];
                    let middle = frame.center(cell);
                    let (dx, dz) = (centre[0] - middle[0], centre[1] - middle[2]);
                    if dx.abs().max(dz.abs()) < frame.cell * 0.3 {
                        assert!(dx * n[0] + dz * n[2] > 0., "pedestal face looks inwards");
                        checked += 1;
                    }
                    continue;
                }
                let probe = [
                    centre[0] + n[0] / length * 0.05,
                    centre[1] + n[2] / length * 0.05,
                ];
                let cell = [
                    (probe[0] / frame.cell + layout.size[0] as f32 * 0.5).floor() as i32,
                    (probe[1] / frame.cell + layout.size[1] as f32 * 0.5).floor() as i32,
                ];
                assert!(
                    layout.at(cell).walkable(),
                    "{source:?} {group} face at {centre:?} looks into solid cell {cell:?}"
                );
                // And towards the middle of that cell, not into a pillar.
                let middle = frame.center(cell);
                let towards = (middle[0] - centre[0]) * n[0] + (middle[2] - centre[1]) * n[2];
                assert!(
                    towards > 0.,
                    "{source:?} {group} face at {centre:?} looks away from cell {cell:?}"
                );
                checked += 1;
            }
        }
        assert!(checked > 100, "{checked}");
    }
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
        [1.4, 2.6, 0.06],
        [0., 3., 0.04],
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
                color: [0.34, 0.38, 0.44],
                texture: None,
            },
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
    // Recipes: the built-in Key and Door pieces make working keys and doors; keys glow.
    let key = style.spot_node(SpotKind::Key).unwrap().id;
    style.nodes.push(piece(
        0x131,
        key,
        490.,
        style.spot_node(SpotKind::Key).unwrap().position[1] + 60.,
        Visual {
            piece: Piece::Light {
                color: [1., 0.82, 0.3],
                intensity: 1.4,
                range: 4.,
            },
            offset: [0., 1.2, 0.],
            consume: false,
            ..Visual::default()
        },
    ));
    // Caves: the built-in recipe's branches tint the same panels mossy and rusty.
    use dungeon_bake::BuiltinTexture;
    for (id, spot, color, texture) in [
        (0x140, floor, [0.55, 0.85, 0.55], BuiltinTexture::Floor),
        (0x141, wall, [0.9, 0.7, 0.55], BuiltinTexture::Wall),
    ] {
        let row = style.nodes.iter().find(|n| n.id == spot).unwrap().position[1];
        style.nodes.push(piece(
            id,
            spot,
            200.,
            row,
            Visual {
                piece: Piece::Blockout {
                    color,
                    texture: Some(texture.id()),
                },
                path: "branch".into(),
                ..Visual::default()
            },
        ));
    }
    let style_path = dungeon_style::fresh_path(&root, "Crypt");
    let style_id = dungeon_style::create(&root, &style_path, &style).unwrap();
    let recipe_path = crate::dungeon_recipe::fresh_path(&root, "Crypt Route");
    let recipe_id =
        crate::dungeon_recipe::create(&root, &recipe_path, &crate::dungeon_recipe::builtin())
            .unwrap();
    let mut actor = crate::scene::Actor::cube("Dungeon".into());
    actor.kind = "Empty".into();
    actor.position = [0.; 3];
    let actor_id = actor.id;
    scene.actors.push(actor);
    let mut generator = Generator::new(actor_id);
    generator.seed = 1207;
    generator.style = Some(style_id);
    generator.source = crate::dungeon::Source::Recipe;
    generator.recipe = Some(recipe_id);
    generator.spawn_actor = scene
        .actors
        .iter()
        .find(|a| a.name == "Player")
        .map(|a| a.id);
    scene.dungeons.push(generator);
    scene.refresh_actor_hierarchy();
    scene.save(&path).unwrap();
}
