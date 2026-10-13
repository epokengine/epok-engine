//! The Platformer 2D starter: a side-scrolling level built entirely from World2D
//! nodes - an animated CharacterBody2D player with a following Camera2D, a tile map
//! painted from a TileSet (autotiled ground, crates, one-way platforms, animated
//! flowers), a lift riding a path, fireflies, a hanging vine, parallax mountains, hills
//! and clouds, spinning coins and a screen-space coin icon -
//! with its gameplay written in the selected flavor. The artwork is original pixel
//! art generated here at creation time, so the template ships no binary files.
use crate::{
    actor_document::{ClassReference, ComponentInstance},
    blueprint_asset::{BinaryOp, BlueprintAsset, Builtin, Graph, Input, Node, NodeKind, Variable},
    reflection_schema::{self as schema, Type},
    scene::{Actor, Scene},
    tilemap::{Cell, Cells},
    tileset::{self, PatternCell, TileSet},
    workspace::GameplayFlavor,
    world2d::{
        self, Animation2D, Camera2D, CanvasLayer2D, Collider2D, EmissionShape, Frame2D, Line2D,
        Parallax2D, Particles2D, Path2D, PathFollow2D, PathPoint, Polygon2D, Preset, Sprite2D,
        TileMapLayer,
    },
};
use serde_json::json;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    sync::Arc,
};
use uuid::Uuid;

pub const CONTROLLER: &str = "PlatformerController";
pub const COIN: &str = "CoinPickup";
pub const CPP_CONTROLLER_ID: &str = "ca7559df-c335-44a1-ae8c-5268c5b68ace";
pub const CPP_COIN_ID: &str = "6755cf22-df21-4f8f-81b5-21c85f6872fa";
pub const LUA_CONTROLLER_ID: &str = "4919923d-00cf-47ed-bcb8-393562762177";
pub const LUA_COIN_ID: &str = "39caab97-d716-49fc-8029-a1fb3caf7380";
pub const BLUEPRINT_CONTROLLER_ID: &str = "31feac4e-5004-460b-9de6-2648b95604d5";
pub const BLUEPRINT_COIN_ID: &str = "7d16f1c2-90f8-48db-88c4-de1f48183edf";
const LUA_CONTROLLER_PATH: &str = "assets/scripts/PlatformerController.lua";
const LUA_COIN_PATH: &str = "assets/scripts/CoinPickup.lua";
const BLUEPRINT_CONTROLLER_PATH: &str = "assets/Blueprints/PlatformerController.epokbp";
const BLUEPRINT_COIN_PATH: &str = "assets/Blueprints/CoinPickup.epokbp";

/// Ground top edge, in world pixels (Y down).
const GROUND: f32 = 96.;
/// The ground's first tile row (16-pixel tiles).
const GROUND_ROW: i32 = 6;

/// The template's TileSet: 16 ground tiles (one per combination of connected sides, a
/// "Sides" terrain) on rows 0-1, then a 2 x 2 crate, a three-piece one-way platform,
/// an animated flower and two plants.
fn tileset(texture: Uuid) -> TileSet {
    let mut set = TileSet {
        texture: Some(texture),
        terrain_sets: vec![tileset::TerrainSet {
            name: "Ground".into(),
            mode: tileset::TerrainMode::Sides,
            terrains: vec![tileset::Terrain {
                name: "Grass".into(),
                color: [104, 198, 76],
            }],
        }],
        ..Default::default()
    };
    for mask in 0u8..16 {
        let mut tile = tileset::Tile::new([mask % 8, mask / 8]);
        tile.collision = tileset::Collision::Full;
        tile.terrain_set = Some(0);
        tile.terrain = Some(0);
        for side in 0..4 {
            if mask & (1 << side) != 0 {
                tile.peering[side * 2] = Some(0);
            }
        }
        set.tiles.push(tile);
    }
    for atlas in [[0, 2], [1, 2], [0, 3], [1, 3]] {
        let mut tile = tileset::Tile::new(atlas);
        tile.collision = tileset::Collision::Full;
        set.tiles.push(tile);
    }
    for column in 2..5 {
        let mut tile = tileset::Tile::new([column, 2]);
        tile.collision = tileset::Collision::Rect;
        tile.rect = [0, 0, 16, 6];
        tile.one_way = true;
        set.tiles.push(tile);
    }
    let mut flower = tileset::Tile::new([5, 2]);
    flower.frames = 2;
    flower.fps = 3.;
    set.tiles.push(flower);
    set.tiles.push(tileset::Tile::new([2, 3]));
    set.tiles.push(tileset::Tile::new([3, 3]));
    set.patterns.push(tileset::Pattern {
        name: "Crate".into(),
        cells: [[0, 0], [1, 0], [0, 1], [1, 1]]
            .map(|o| PatternCell {
                offset: o,
                atlas: [o[0], o[1] + 2],
                flags: 0,
            })
            .to_vec(),
    });
    set.patterns.push(tileset::Pattern {
        name: "Platform".into(),
        cells: (0..3)
            .map(|x| PatternCell {
                offset: [x, 0],
                atlas: [x + 2, 2],
                flags: 0,
            })
            .collect(),
    });
    set.sort();
    set
}
/// Paints the level: autotiled ground, crates and one-way platforms on the solid layer,
/// plants on the decoration layer.
fn paint_level(set: &TileSet) -> (Cells, Cells) {
    let mut solid = Cells::new();
    let ground: BTreeSet<[i32; 2]> = (-28..28)
        .flat_map(|x| (GROUND_ROW..GROUND_ROW + 2).map(move |y| [x, y]))
        .collect();
    let terrain_of = |p: [i32; 2]| ground.contains(&p).then_some(0u8);
    for (p, atlas) in tileset::solve(set, 0, &ground, &terrain_of) {
        solid.insert(p, Cell::new(atlas));
    }
    let stamp = |cells: &mut Cells, pattern: usize, at: [i32; 2]| {
        for c in &set.patterns[pattern].cells {
            cells.insert(
                [
                    at[0] + i32::from(c.offset[0]),
                    at[1] + i32::from(c.offset[1]),
                ],
                Cell::new(c.atlas),
            );
        }
    };
    for at in [[-28, 4], [-28, 2], [15, 4]] {
        stamp(&mut solid, 0, at);
    }
    for at in [[-13, 2], [-2, 0], [8, -2], [20, 2]] {
        stamp(&mut solid, 1, at);
    }
    let mut decor = Cells::new();
    for (x, atlas) in [
        (-22, [5, 2]),
        (-19, [2, 3]),
        (-9, [3, 3]),
        (-5, [5, 2]),
        (3, [2, 3]),
        (11, [5, 2]),
        (18, [3, 3]),
        (24, [5, 2]),
    ] {
        decor.insert([x, GROUND_ROW - 1], Cell::new(atlas));
    }
    (solid, decor)
}

pub fn create(root: &Path, flavor: GameplayFlavor) -> Result<Scene, String> {
    let art = art::install(root)?;
    let mut scene = Scene {
        name: "Main".into(),
        actors: Vec::new(),
        ..Scene::default()
    };
    let sprite = |texture: Uuid| Sprite2D {
        texture: Some(texture),
        ..Default::default()
    };
    // Grouping nodes: logical parents only, so they tidy the Hierarchy without
    // moving their children.
    let group = |scene: &mut Scene, name: &str| {
        let actor = world2d::create(Preset::Node2D, name, [0., 0.]);
        let id = actor.id;
        scene.actors.push(actor);
        id
    };
    // The Hierarchy reads top to bottom like the screen is built: scenery behind, the
    // level, who lives in it, props, coins, effects, then the HUD.
    let scenery = group(&mut scene, "Scenery");
    let level = group(&mut scene, "Level");
    let characters = group(&mut scene, "Characters");
    let props_group = group(&mut scene, "Props");
    let collectibles = group(&mut scene, "Coins");
    let effects = group(&mut scene, "Effects");
    // Parallax backgrounds: hills scroll at half the camera's speed and repeat every
    // 256 pixels; clouds drift slowly and repeat every 768.
    let mut hills = world2d::create(Preset::Parallax2D, "Hills", [0., GROUND - 32.]);
    set(
        &mut hills,
        world2d::PARALLAX2D,
        &Parallax2D {
            scroll_scale: [0.5, 1.],
            repeat_size: [256., 0.],
            ..Default::default()
        },
    );
    add(
        &mut hills,
        world2d::SPRITE2D,
        &Sprite2D {
            z_index: -10,
            ..sprite(art.hills)
        },
    );
    let hills_id = hills.id;
    scene.actors.push(hills);
    // Distant mountains: a polygon on the hills' parallax, so it scrolls and repeats with them.
    let mut mountains = world2d::create(Preset::Polygon2D, "Mountains", [0., 0.]);
    set(
        &mut mountains,
        world2d::POLYGON2D,
        &Polygon2D {
            points: vec![
                [-128., 0.],
                [-92., -44.],
                [-60., -26.],
                [-16., -70.],
                [36., -18.],
                [78., -50.],
                [128., 0.],
            ],
            color: [150, 176, 214],
            z_index: -11,
            ..Default::default()
        },
    );
    mountains.logical_parent = Some(hills_id);
    mountains.attach = Some(crate::actor_document::Attachment {
        actor: hills_id,
        component: None,
    });
    scene.actors.push(mountains);
    let sky = world2d::create(Preset::Parallax2D, "Clouds", [0., 0.]);
    let sky_id = sky.id;
    scene.actors.push(sky);
    let clouds = scene.actors.len() - 1;
    set(
        &mut scene.actors[clouds],
        world2d::PARALLAX2D,
        &Parallax2D {
            scroll_scale: [0.25, 0.5],
            repeat_size: [768., 0.],
            autoscroll: [-6., 0.],
        },
    );
    for (i, [x, y]) in [[-300., -70.], [-40., -96.], [180., -60.], [400., -88.]]
        .into_iter()
        .enumerate()
    {
        let mut cloud = world2d::create(Preset::Sprite2D, &format!("Cloud{i}"), [x, y]);
        set(
            &mut cloud,
            world2d::SPRITE2D,
            &Sprite2D {
                z_index: -9,
                ..sprite(art.cloud)
            },
        );
        cloud.logical_parent = Some(sky_id);
        cloud.attach = Some(crate::actor_document::Attachment {
            actor: sky_id,
            component: None,
        });
        scene.actors.push(cloud);
    }
    // The level: a solid tile layer and a decoration layer painted from one TileSet.
    let tiles = tileset(art.tiles);
    let set_id = tileset::create(root, "assets/TileSets/Tiles.epokasset", &tiles)?;
    let (solid, decor) = paint_level(&tiles);
    for (name, cells, z, collision) in [("Ground", &solid, 0, true), ("Details", &decor, -1, false)]
    {
        let mut layer = world2d::create(Preset::TileMapLayer, name, [0., 0.]);
        set(
            &mut layer,
            world2d::TILEMAP_LAYER,
            &TileMapLayer {
                tile_set: Some(set_id),
                cells: crate::tilemap::encode(cells),
                z_index: z,
                collision_enabled: collision,
                ..Default::default()
            },
        );
        layer.logical_parent = Some(level);
        scene.actors.push(layer);
    }
    scene.tilesets.insert(set_id, Arc::new(tiles));
    // A lift between the middle platforms: a Path2D and a one-way platform that follows it,
    // carrying the player.
    let mut rail = world2d::create(Preset::Path2D, "LiftPath", [24., -8.]);
    set(
        &mut rail,
        world2d::PATH2D,
        &Path2D {
            points: vec![PathPoint::at([0., 0.]), PathPoint::at([88., -24.])],
            closed: false,
        },
    );
    rail.logical_parent = Some(level);
    let rail_id = rail.id;
    scene.actors.push(rail);
    let mut lift = world2d::create(Preset::PathFollow2D, "Lift", [24., -8.]);
    set(
        &mut lift,
        world2d::PATH_FOLLOW2D,
        &PathFollow2D {
            speed: 28.,
            ping_pong: true,
            looping: false,
            ..Default::default()
        },
    );
    add(
        &mut lift,
        world2d::SPRITE2D,
        &Sprite2D {
            region: [32, 32, 48, 12],
            centered: false,
            offset: [-24., -3.],
            ..sprite(art.tiles)
        },
    );
    add(
        &mut lift,
        world2d::COLLIDER2D,
        &Collider2D {
            size: [48., 6.],
            one_way: true,
            one_way_margin: 3.,
            ..Default::default()
        },
    );
    lift.logical_parent = Some(rail_id);
    lift.attach = Some(crate::actor_document::Attachment {
        actor: rail_id,
        component: None,
    });
    scene.actors.push(lift);
    // A vine hanging under the left platform, and fireflies drifting over the meadow.
    let mut vine = world2d::create(Preset::Line2D, "Vine", [-196., 38.]);
    set(
        &mut vine,
        world2d::LINE2D,
        &Line2D {
            points: vec![[0., 0.], [2., 9.], [-1., 18.], [1., 27.]],
            width: 2.,
            begin_color: [62, 140, 56],
            end_color: [124, 214, 88],
            z_index: -1,
            ..Default::default()
        },
    );
    vine.logical_parent = Some(level);
    scene.actors.push(vine);
    let mut fireflies = world2d::create(Preset::Particles2D, "Fireflies", [-120., 70.]);
    set(
        &mut fireflies,
        world2d::PARTICLES2D,
        &Particles2D {
            amount: 10,
            lifetime: 3.,
            direction: -90.,
            spread: 70.,
            speed: [4., 12.],
            gravity: [0., -2.],
            shape: EmissionShape::Rectangle,
            extents: [96., 10.],
            start_color: [255, 240, 120],
            end_color: [120, 200, 80],
            texture: Some(art.spark),
            blend: crate::texture::BlendMode::Add,
            z_index: 3,
            ..Default::default()
        },
    );
    fireflies.logical_parent = Some(level);
    scene.actors.push(fireflies);
    // A screen-space coin icon in the top-left corner, drawn over the world.
    let mut hud = world2d::create(Preset::CanvasLayer2D, "HUD", [0., 0.]);
    set(&mut hud, world2d::CANVAS_LAYER2D, &CanvasLayer2D::default());
    let hud_id = hud.id;
    scene.actors.push(hud);
    let spin = Animation2D {
        name: "spin".into(),
        fps: 10.,
        looping: true,
        frames: (0..4)
            .map(|i| Frame2D {
                region: [i * 16, 0, 16, 16],
                ..Default::default()
            })
            .collect(),
    };
    let mut icon = world2d::create(Preset::AnimatedSprite2D, "CoinIcon", [14., 14.]);
    set(
        &mut icon,
        world2d::SPRITE2D,
        &Sprite2D {
            animations: vec![spin.clone()],
            ..sprite(art.coin)
        },
    );
    icon.logical_parent = Some(hud_id);
    icon.attach = Some(crate::actor_document::Attachment {
        actor: hud_id,
        component: None,
    });
    scene.actors.push(icon);
    let mut coins = Vec::new();
    for (i, [x, y]) in [
        [-200., 16.],
        [-168., 16.],
        [-8., -16.],
        [152., -48.],
        [96., 72.],
        [128., 72.],
        [344., 16.],
        [-300., 72.],
    ]
    .into_iter()
    .enumerate()
    {
        let mut coin = world2d::create(Preset::Area2D, &format!("Coin{i}"), [x, y]);
        set(
            &mut coin,
            world2d::COLLIDER2D,
            &Collider2D {
                size: [12., 12.],
                trigger: true,
                ..Default::default()
            },
        );
        add(
            &mut coin,
            world2d::SPRITE2D,
            &Sprite2D {
                animations: vec![spin.clone()],
                z_index: 1,
                ..sprite(art.coin)
            },
        );
        coin.logical_parent = Some(collectibles);
        coins.push(scene.actors.len());
        scene.actors.push(coin);
    }
    // The player: an animated body that the camera follows inside the level.
    let mut player = world2d::create(Preset::CharacterBody2D, "Player", [-256., GROUND - 12.]);
    let frames = |indices: &[u16]| {
        indices
            .iter()
            .map(|i| Frame2D {
                region: [i * 16, 0, 16, 24],
                ..Default::default()
            })
            .collect()
    };
    set(
        &mut player,
        world2d::SPRITE2D,
        &Sprite2D {
            animations: vec![
                Animation2D {
                    name: "idle".into(),
                    fps: 3.,
                    looping: true,
                    frames: frames(&[0, 1]),
                },
                Animation2D {
                    name: "run".into(),
                    fps: 12.,
                    looping: true,
                    frames: frames(&[2, 3, 4, 5]),
                },
            ],
            z_index: 2,
            ..sprite(art.player)
        },
    );
    set(
        &mut player,
        world2d::COLLIDER2D,
        &Collider2D {
            size: [10., 22.],
            offset: [0., 1.],
            ..Default::default()
        },
    );
    add(
        &mut player,
        world2d::CAMERA2D,
        &Camera2D {
            smoothing: true,
            smoothing_speed: 8.,
            limits_enabled: true,
            limits: [-448., -240., 448., 128.],
            background: [96, 156, 228],
            ..Default::default()
        },
    );
    let body = component_id(&player, world2d::CHARACTER_BODY2D)?;
    let sprite_component = component_id(&player, world2d::SPRITE2D)?;
    let mut controller = ComponentInstance::new(
        Uuid::new_v4(),
        ClassReference::new(CONTROLLER, controller_id(flavor)),
        "Controller",
    );
    if flavor != GameplayFlavor::Cpp {
        // Typed references, resolved once here: renaming a component never breaks them.
        controller
            .properties
            .insert("body".into(), json!(body.to_string()));
        controller
            .properties
            .insert("sprite".into(), json!(sprite_component.to_string()));
        controller
            .overrides
            .extend(controller.properties.keys().cloned());
    }
    player.components.push(controller);
    player.refresh_components();
    let player_id = player.id;
    scene.actors.push(player);
    guide(&mut scene, art.guide, player_id, characters);
    wisp(&mut scene, art.spark, player_id, characters);
    props(&mut scene, art.tiles, art.guide, props_group);
    scenery_layers(&mut scene, &art, scenery);
    ambience(&mut scene, art.spark, effects);
    controls_hint(&mut scene);
    // Top-level nodes join their group; attached nodes stay under what they hang from.
    for actor in &mut scene.actors {
        if actor.logical_parent.is_some() || actor.attach.is_some() {
            continue;
        }
        actor.logical_parent = match actor.name.as_str() {
            "Hills" | "Clouds" => Some(scenery),
            "Player" => Some(characters),
            "Fireflies" => Some(effects),
            _ => None,
        };
    }
    for actor in &mut scene.actors {
        if actor.name == "Fireflies" || actor.name == "WispPaths" {
            actor.logical_parent = Some(if actor.name == "Fireflies" {
                effects
            } else {
                level
            });
        }
    }
    for index in coins {
        let pickup = ComponentInstance::new(
            Uuid::new_v4(),
            ClassReference::new(COIN, coin_id(flavor)),
            "Pickup",
        );
        scene.actors[index].components.push(pickup);
        scene.actors[index].refresh_components();
    }
    install_gameplay(root, flavor)?;
    Ok(scene)
}

/// A cut-out guide near the start: a Skeleton2D whose robe breathes and head nods on a
/// looping animation while a two-bone IK arm points at the player.
fn guide(scene: &mut Scene, texture: Uuid, player: Uuid, level: Uuid) {
    let mut root = world2d::create(Preset::Skeleton2D, "Guide", [-208., GROUND - 8.]);
    root.logical_parent = Some(level);
    let root_id = root.id;
    scene.actors.push(root);
    let mut part = |name: &str, parent: Uuid, at: [f32; 2], region: [u16; 4], z_index: i16| {
        let centred = z_index != 3;
        let mut actor = world2d::create(Preset::Sprite2D, name, at);
        set(
            &mut actor,
            world2d::SPRITE2D,
            &Sprite2D {
                texture: Some(texture),
                region,
                centered: centred,
                // Limbs hang off their joint along +X.
                offset: if centred { [0., 0.] } else { [0., -2.] },
                z_index,
                ..Default::default()
            },
        );
        actor.logical_parent = Some(parent);
        actor.attach = Some(crate::actor_document::Attachment {
            actor: parent,
            component: None,
        });
        let id = actor.id;
        scene.actors.push(actor);
        id
    };
    // z 3 marks the limbs, drawn from their joint; the hand sits on top of the forearm.
    let torso = part("Robe", root_id, [0., 0.], [0, 0, 12, 16], 1);
    let head = part("Head", torso, [0., -13.], [16, 0, 12, 12], 2);
    let upper = part("UpperArm", torso, [3., -5.], [32, 0, 10, 4], 3);
    let fore = part("Forearm", upper, [9., 0.], [32, 6, 10, 4], 3);
    let hand = part("Hand", fore, [9., 0.], [48, 0, 4, 4], 4);
    let bones = [
        (torso, [0., 0., 0.]),
        (head, [0., 0., -13.]),
        (upper, [0., 3., -5.]),
        (fore, [0., 9., 0.]),
    ];
    let mut skeleton = crate::skeleton2d::Skeleton2D {
        bones: bones
            .iter()
            .map(|(id, rest)| crate::skeleton2d::Bone {
                actor: Some(*id),
                rest: *rest,
            })
            .collect(),
        animations: vec![crate::skeleton2d::Animation {
            name: "idle".into(),
            length: 2.,
            looping: true,
            keys: Vec::new(),
        }],
        autoplay: Some(0),
        ik: vec![crate::skeleton2d::Ik {
            upper: 2,
            lower: 3,
            tip: Some(hand),
            target: Some(player),
            ..Default::default()
        }],
        ..Default::default()
    };
    let rest: Vec<[f32; 3]> = bones.iter().map(|(_, r)| *r).collect();
    let mut breathe = rest.clone();
    breathe[0] = [-3., 0., 1.];
    breathe[1] = [8., 0., -12.];
    skeleton.key_pose(0, 0., rest);
    skeleton.key_pose(0, 1., breathe);
    if let Some(index) = scene.actor_index(root_id) {
        set(&mut scene.actors[index], world2d::SKELETON2D, &skeleton);
    }
}

/// A glowing wisp that keeps near the player, flying round the level's solid tiles: a
/// NavigationRegion2D grid over the Ground layer and a NavigationAgent2D following the
/// player.
fn wisp(scene: &mut Scene, spark: Uuid, player: Uuid, level: Uuid) {
    let Some(ground) = scene
        .actors
        .iter()
        .find(|a| a.name == "Ground")
        .map(|a| a.id)
    else {
        return;
    };
    let mut region = world2d::create(Preset::NavigationRegion2D, "WispPaths", [0., 0.]);
    set(
        &mut region,
        world2d::NAVIGATION_REGION2D,
        &crate::navigation2d::NavigationRegion2D {
            tile_layer: Some(ground),
            margin: 3,
            agent_radius: 4.,
            ..Default::default()
        },
    );
    region.logical_parent = Some(level);
    scene.actors.push(region);
    let mut wisp = world2d::create(Preset::Particles2D, "Wisp", [-236., 40.]);
    set(
        &mut wisp,
        world2d::PARTICLES2D,
        &Particles2D {
            amount: 12,
            lifetime: 0.8,
            spread: 180.,
            speed: [2., 6.],
            gravity: [0., -6.],
            start_color: [255, 250, 200],
            end_color: [90, 150, 255],
            texture: Some(spark),
            blend: crate::texture::BlendMode::Add,
            z_index: 4,
            ..Default::default()
        },
    );
    add(
        &mut wisp,
        world2d::SPRITE2D,
        &Sprite2D {
            texture: Some(spark),
            blend: crate::texture::BlendMode::Add,
            modulate: [255, 255, 220],
            z_index: 5,
            ..Default::default()
        },
    );
    add(
        &mut wisp,
        world2d::NAVIGATION_AGENT2D,
        &crate::navigation2d::NavigationAgent2D {
            target: Some(player),
            max_speed: 80.,
            arrive_distance: 20.,
            repath_interval: 0.3,
            ..Default::default()
        },
    );
    wisp.logical_parent = Some(level);
    scene.actors.push(wisp);
}

/// Physics props near the start: a crate the player can push, and a garland of light
/// bulbs on rope joints, its cord a Line2D through the anchors and bulbs, swaying.
fn props(scene: &mut Scene, tiles: Uuid, sheet: Uuid, level: Uuid) {
    let mut crate_box = world2d::create(Preset::RigidBody2D, "PushCrate", [-164., GROUND - 16.]);
    set(
        &mut crate_box,
        world2d::SPRITE2D,
        &Sprite2D {
            texture: Some(tiles),
            region: [0, 32, 32, 32],
            z_index: 1,
            ..Default::default()
        },
    );
    set(
        &mut crate_box,
        world2d::COLLIDER2D,
        &Collider2D {
            size: [32., 32.],
            ..Default::default()
        },
    );
    set(
        &mut crate_box,
        world2d::RIGID_BODY2D,
        &crate::physics2d::RigidBody2D {
            mass: 2.,
            bounce: 0.,
            friction: 0.6,
            ..Default::default()
        },
    );
    crate_box.logical_parent = Some(level);
    scene.actors.push(crate_box);

    let garland = world2d::create(Preset::Node2D, "Garland", [0., 0.]);
    let garland_id = garland.id;
    scene.actors.push(garland);
    let node = |scene: &mut Scene, preset: Preset, name: &str, at: [f32; 2]| {
        let mut actor = world2d::create(preset, name, at);
        actor.logical_parent = Some(garland_id);
        let id = actor.id;
        scene.actors.push(actor);
        id
    };
    let (left, right) = ([-316., -6.], [-196., -6.]);
    let left_id = node(scene, Preset::Node2D, "HookLeft", left);
    let colours = [
        [255, 60, 60],
        [255, 220, 70],
        [90, 255, 110],
        [90, 160, 255],
        [255, 110, 220],
    ];
    let mut previous = left_id;
    let mut chain = vec![left_id];
    for (i, colour) in colours.iter().enumerate() {
        let t = (i as f32 + 1.) / (colours.len() as f32 + 1.);
        let x = left[0] + (right[0] - left[0]) * t;
        let sag = 24. * (1. - (2. * t - 1.).powi(2));
        let id = node(
            scene,
            Preset::Sprite2D,
            &format!("Bulb{i}"),
            [x, left[1] + sag],
        );
        let index = scene.actors.len() - 1;
        let bulb = &mut scene.actors[index];
        set(
            bulb,
            world2d::SPRITE2D,
            &Sprite2D {
                texture: Some(sheet),
                region: [56, 0, 6, 8],
                offset: [0., 3.],
                modulate: *colour,
                z_index: 2,
                ..Default::default()
            },
        );
        add(
            bulb,
            world2d::RIGID_BODY2D,
            &crate::physics2d::RigidBody2D {
                mass: 0.3,
                can_sleep: false,
                // The middle bulb starts the garland swaying.
                velocity: if i == 2 { [40., 0.] } else { [0., 0.] },
                ..Default::default()
            },
        );
        add(
            bulb,
            world2d::JOINT2D,
            &crate::physics2d::Joint2D {
                body_a: Some(previous),
                body_b: Some(id),
                kind: crate::physics2d::JointKind::Rope,
                ..Default::default()
            },
        );
        previous = id;
        chain.push(id);
    }
    let right_id = node(scene, Preset::Joint2D, "HookRight", right);
    let index = scene.actors.len() - 1;
    set(
        &mut scene.actors[index],
        world2d::JOINT2D,
        &crate::physics2d::Joint2D {
            body_b: Some(previous),
            kind: crate::physics2d::JointKind::Rope,
            ..Default::default()
        },
    );
    chain.push(right_id);
    // The cord, drawn through the hooks and bulbs as they move.
    if let Some(index) = scene.actor_index(left_id) {
        add(
            &mut scene.actors[index],
            world2d::LINE2D,
            &Line2D {
                follow: chain,
                width: 1.,
                begin_color: [40, 64, 40],
                end_color: [40, 64, 40],
                z_index: 1,
                ..Default::default()
            },
        );
    }
    if let Some(index) = scene.actor_index(garland_id) {
        scene.actors[index].logical_parent = Some(level);
    }
}

/// The sky behind every canvas (a gradient stretched over the screen and the sun), a
/// tree line between the hills and the level, and dark bushes in front of everything,
/// scrolling faster than the camera.
fn scenery_layers(scene: &mut Scene, art: &art::Textures, scenery: Uuid) {
    let sprite = |texture: Uuid| Sprite2D {
        texture: Some(texture),
        ..Default::default()
    };
    let push = |scene: &mut Scene, mut actor: Actor, parent: Uuid, attach: bool| {
        actor.logical_parent = Some(parent);
        if attach {
            actor.attach = Some(crate::actor_document::Attachment {
                actor: parent,
                component: None,
            });
        }
        let id = actor.id;
        scene.actors.push(actor);
        id
    };
    let mut sky_layer = world2d::create(Preset::CanvasLayer2D, "SkyLayer", [0., 0.]);
    set(
        &mut sky_layer,
        world2d::CANVAS_LAYER2D,
        &CanvasLayer2D {
            layer: -20,
            ..Default::default()
        },
    );
    let sky_layer = push(scene, sky_layer, scenery, false);
    // Stretched to cover any display up to 640 x 480.
    let mut sky = world2d::create(Preset::Sprite2D, "Sky", [0., 0.]);
    set(
        &mut sky,
        world2d::SPRITE2D,
        &Sprite2D {
            centered: false,
            ..sprite(art.sky)
        },
    );
    if let Some(root) = sky.components.iter_mut().find(|c| c.root) {
        root.properties.insert("scale".into(), json!([40., 7.5]));
    }
    push(scene, sky, sky_layer, true);
    let mut sun = world2d::create(Preset::Sprite2D, "Sun", [252., 46.]);
    set(
        &mut sun,
        world2d::SPRITE2D,
        &Sprite2D {
            blend: crate::texture::BlendMode::Add,
            z_index: 1,
            ..sprite(art.sun)
        },
    );
    push(scene, sun, sky_layer, true);
    let mut trees = world2d::create(Preset::Parallax2D, "Treeline", [0., GROUND - 20.]);
    set(
        &mut trees,
        world2d::PARALLAX2D,
        &Parallax2D {
            scroll_scale: [0.7, 1.],
            repeat_size: [256., 0.],
            ..Default::default()
        },
    );
    add(
        &mut trees,
        world2d::SPRITE2D,
        &Sprite2D {
            z_index: -8,
            ..sprite(art.trees)
        },
    );
    push(scene, trees, scenery, false);
    let mut front = world2d::create(Preset::Parallax2D, "Foreground", [0., GROUND + 14.]);
    set(
        &mut front,
        world2d::PARALLAX2D,
        &Parallax2D {
            scroll_scale: [1.25, 1.],
            repeat_size: [128., 0.],
            ..Default::default()
        },
    );
    add(
        &mut front,
        world2d::SPRITE2D,
        &Sprite2D {
            z_index: 20,
            ..sprite(art.bushes)
        },
    );
    push(scene, front, scenery, false);
}
/// Petals drifting down over the screen, on a canvas of their own in front of the world.
fn ambience(scene: &mut Scene, spark: Uuid, effects: Uuid) {
    let mut layer = world2d::create(Preset::CanvasLayer2D, "Ambience", [0., 0.]);
    set(
        &mut layer,
        world2d::CANVAS_LAYER2D,
        &CanvasLayer2D {
            layer: 2,
            ..Default::default()
        },
    );
    layer.logical_parent = Some(effects);
    let layer_id = layer.id;
    scene.actors.push(layer);
    let mut petals = world2d::create(Preset::Particles2D, "Petals", [160., -8.]);
    set(
        &mut petals,
        world2d::PARTICLES2D,
        &Particles2D {
            amount: 12,
            lifetime: 7.,
            direction: 90.,
            spread: 30.,
            speed: [6., 14.],
            gravity: [5., 3.],
            shape: EmissionShape::Rectangle,
            extents: [176., 4.],
            start_color: [255, 170, 210],
            end_color: [255, 240, 250],
            scale: [1., 0.6],
            texture: Some(spark),
            ..Default::default()
        },
    );
    petals.logical_parent = Some(layer_id);
    petals.attach = Some(crate::actor_document::Attachment {
        actor: layer_id,
        component: None,
    });
    scene.actors.push(petals);
}
/// A line of help at the top of the screen: a HUD canvas and its text.
fn controls_hint(scene: &mut Scene) {
    let mut canvas = Actor::cube("HelpCanvas".into());
    canvas.kind = "Empty".into();
    canvas.data.canvas = Some(Default::default());
    let parent = scene.actors.len();
    scene.actors.push(canvas);
    let mut hint = Actor::cube("ControlsHint".into());
    hint.kind = "Empty".into();
    hint.data.parent = Some(parent);
    hint.data.rect = Some(crate::hud::RectTransform {
        anchor_min: [0.5, 1.],
        anchor_max: [0.5, 1.],
        pivot: [0.5, 1.],
        position: [0., -6.],
        size: [240., 16.],
        ..Default::default()
    });
    hint.data.text = Some(crate::hud::Text {
        text: "LEFT/RIGHT MOVE   X JUMP".into(),
        color: [1., 0.96, 0.86],
        wrap: false,
        align: crate::hud::TextAlign::Center,
        ..Default::default()
    });
    scene.actors.push(hint);
    scene.sync_actor_components();
}

pub fn controller_id(flavor: GameplayFlavor) -> &'static str {
    match flavor {
        GameplayFlavor::Cpp => CPP_CONTROLLER_ID,
        GameplayFlavor::Blueprint => BLUEPRINT_CONTROLLER_ID,
        GameplayFlavor::Lua => LUA_CONTROLLER_ID,
    }
}
pub fn coin_id(flavor: GameplayFlavor) -> &'static str {
    match flavor {
        GameplayFlavor::Cpp => CPP_COIN_ID,
        GameplayFlavor::Blueprint => BLUEPRINT_COIN_ID,
        GameplayFlavor::Lua => LUA_COIN_ID,
    }
}
fn component_id(actor: &Actor, class: &str) -> Result<Uuid, String> {
    world2d::component(actor, class)
        .map(|c| c.id)
        .ok_or_else(|| {
            format!(
                "Platformer template {} has no {class} component",
                actor.name
            )
        })
}
fn set<T: serde::Serialize>(actor: &mut Actor, class: &str, value: &T) {
    if let Some(component) = actor
        .components
        .iter_mut()
        .find(|c| c.class.class_id.as_deref() == Some(class))
    {
        world2d::write(component, value);
    }
    actor.refresh_components();
}
fn add<T: serde::Serialize>(actor: &mut Actor, class: &str, value: &T) {
    if let Some(mut component) = world2d::new_component(actor, class) {
        world2d::write(&mut component, value);
        actor.components.push(component);
    }
    actor.refresh_components();
}

fn install_gameplay(root: &Path, flavor: GameplayFlavor) -> Result<(), String> {
    let write = |path: &str, bytes: &[u8]| {
        crate::project::write_changed(&crate::assets::inside(root, path)?, bytes)
    };
    match flavor {
        GameplayFlavor::Cpp => {
            write(
                "assets/scripts/PlatformerController.hpp",
                include_bytes!("../templates/PlatformerController.hpp"),
            )?;
            write(
                "assets/scripts/PlatformerController.cpp",
                include_bytes!("../templates/PlatformerController.cpp"),
            )?;
            write(
                "assets/scripts/CoinPickup.hpp",
                include_bytes!("../templates/CoinPickup.hpp"),
            )?;
            write(
                "assets/scripts/CoinPickup.cpp",
                include_bytes!("../templates/CoinPickup.cpp"),
            )?;
        }
        GameplayFlavor::Lua => {
            write(
                LUA_CONTROLLER_PATH,
                include_bytes!("../templates/PlatformerController.lua"),
            )?;
            write(LUA_COIN_PATH, include_bytes!("../templates/CoinPickup.lua"))?;
            let mut registry = crate::lua_identity::Registry::default();
            registry
                .classes
                .insert(LUA_CONTROLLER_PATH.into(), LUA_CONTROLLER_ID.into());
            registry
                .classes
                .insert(LUA_COIN_PATH.into(), LUA_COIN_ID.into());
            registry.save(root)?;
        }
        GameplayFlavor::Blueprint => {
            std::fs::create_dir_all(root.join("assets/Blueprints")).map_err(|e| e.to_string())?;
            crate::blueprint_asset::create(
                &crate::assets::inside(root, BLUEPRINT_CONTROLLER_PATH)?,
                &blueprint::controller(),
            )?;
            crate::blueprint_asset::create(
                &crate::assets::inside(root, BLUEPRINT_COIN_PATH)?,
                &blueprint::coin(),
            )?;
        }
    }
    Ok(())
}

// ---- artwork ---------------------------------------------------------------------------
mod art {
    use std::path::Path;
    use uuid::Uuid;

    pub struct Textures {
        pub player: Uuid,
        pub tiles: Uuid,
        pub coin: Uuid,
        pub hills: Uuid,
        pub cloud: Uuid,
        pub spark: Uuid,
        pub guide: Uuid,
        pub sky: Uuid,
        pub sun: Uuid,
        pub trees: Uuid,
        pub bushes: Uuid,
    }
    type Rgba = [u8; 4];
    const CLEAR: Rgba = [0, 0, 0, 0];
    const OUTLINE: Rgba = [34, 30, 52, 255];

    struct Canvas {
        width: usize,
        height: usize,
        pixels: Vec<Rgba>,
    }
    impl Canvas {
        fn new(width: usize, height: usize) -> Self {
            Self {
                width,
                height,
                pixels: vec![CLEAR; width * height],
            }
        }
        fn set(&mut self, x: i32, y: i32, c: Rgba) {
            if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
                self.pixels[y as usize * self.width + x as usize] = c;
            }
        }
        fn get(&self, x: i32, y: i32) -> Rgba {
            if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
                self.pixels[y as usize * self.width + x as usize]
            } else {
                CLEAR
            }
        }
        fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: Rgba) {
            for j in y..y + h {
                for i in x..x + w {
                    self.set(i, j, c);
                }
            }
        }
        /// A dark outline around every opaque shape inside one cell.
        fn outline(&mut self, x0: i32, y0: i32, w: i32, h: i32) {
            let mut edge = Vec::new();
            for y in y0..y0 + h {
                for x in x0..x0 + w {
                    if self.get(x, y)[3] != 0 {
                        continue;
                    }
                    let near = [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|(dx, dy)| {
                        let (nx, ny) = (x + dx, y + dy);
                        nx >= x0
                            && ny >= y0
                            && nx < x0 + w
                            && ny < y0 + h
                            && self.get(nx, ny)[3] != 0
                            && self.get(nx, ny) != OUTLINE
                    });
                    if near {
                        edge.push((x, y));
                    }
                }
            }
            for (x, y) in edge {
                self.set(x, y, OUTLINE);
            }
        }
        fn png(&self) -> Result<Vec<u8>, String> {
            let mut bytes = Vec::new();
            {
                let mut encoder =
                    png::Encoder::new(&mut bytes, self.width as u32, self.height as u32);
                encoder.set_color(png::ColorType::Rgba);
                encoder.set_depth(png::BitDepth::Eight);
                let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
                let data: Vec<u8> = self.pixels.iter().flatten().copied().collect();
                writer.write_image_data(&data).map_err(|e| e.to_string())?;
            }
            Ok(bytes)
        }
    }
    /// Deterministic noise so the generated art is identical on every machine.
    fn noise(x: i32, y: i32, seed: u32) -> u32 {
        let mut h = (x as u32).wrapping_mul(374_761_393)
            ^ (y as u32).wrapping_mul(668_265_263)
            ^ seed.wrapping_mul(2_246_822_519);
        h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
        h ^ (h >> 16)
    }

    fn player() -> Canvas {
        const SKIN: Rgba = [255, 206, 170, 255];
        const HAIR: Rgba = [128, 74, 40, 255];
        const SHIRT: Rgba = [62, 168, 222, 255];
        const SHIRT_DARK: Rgba = [40, 120, 180, 255];
        const PANTS: Rgba = [54, 66, 142, 255];
        const SHOES: Rgba = [46, 42, 58, 255];
        const EYE: Rgba = [24, 22, 36, 255];
        let mut c = Canvas::new(96, 24);
        // 0-1 idle (breathing), 2-5 run (legs and arms swing).
        let poses: [(i32, i32, i32, i32); 6] = [
            (0, 0, 0, 0),
            (1, 0, 0, 0),
            (0, 2, -2, 1),
            (1, 0, 0, 0),
            (0, -2, 2, -1),
            (1, 0, 0, 0),
        ];
        for (frame, (bob, front, back, arm)) in poses.into_iter().enumerate() {
            let x = frame as i32 * 16;
            // Legs (drawn first so the body overlaps their tops).
            c.rect(x + 5 + back, 17, 3, 4, PANTS);
            c.rect(x + 8 + front, 17, 3, 4, PANTS);
            c.rect(x + 4 + back, 21, 4, 2, SHOES);
            c.rect(x + 8 + front, 21, 4, 2, SHOES);
            // Body and arms.
            c.rect(x + 5, 10 + bob, 6, 7 - bob, SHIRT);
            c.rect(x + 5, 15, 6, 2, SHIRT_DARK);
            c.rect(x + 3 - arm.max(0), 11 + bob, 2, 4, SHIRT_DARK);
            c.rect(x + 3 - arm.max(0), 15 + bob, 2, 1, SKIN);
            c.rect(x + 11 + arm.min(0).abs(), 11 + bob, 2, 4, SHIRT);
            c.rect(x + 11 + arm.min(0).abs(), 15 + bob, 2, 1, SKIN);
            // Head, hair and a forward-looking eye.
            c.rect(x + 4, 2 + bob, 8, 8, SKIN);
            c.rect(x + 4, 1 + bob, 8, 3, HAIR);
            c.rect(x + 4, 4 + bob, 2, 3, HAIR);
            c.set(x + 9, 5 + bob, EYE);
            c.set(x + 9, 6 + bob, EYE);
            c.set(x + 11, 8 + bob, [236, 150, 130, 255]);
            c.outline(x, 0, 16, 24);
        }
        c
    }
    const GRASS_LIGHT: Rgba = [124, 214, 88, 255];
    const GRASS: Rgba = [104, 198, 76, 255];
    const GRASS_SHADE: Rgba = [96, 188, 72, 255];
    const GRASS_DARK: Rgba = [62, 140, 56, 255];
    const DIRT: Rgba = [136, 90, 56, 255];
    const DIRT_LIGHT: Rgba = [154, 104, 66, 255];
    const DIRT_DARK: Rgba = [92, 60, 40, 255];
    /// The 128 x 64 tile atlas (16-pixel tiles); see `super::tileset` for the layout.
    fn tiles() -> Canvas {
        let mut c = Canvas::new(128, 64);
        for mask in 0..16i32 {
            let (ox, oy) = ((mask % 8) * 16, (mask / 8) * 16);
            let (right, bottom, left, top) =
                (mask & 1 != 0, mask & 2 != 0, mask & 4 != 0, mask & 8 != 0);
            for y in 0..16 {
                for x in 0..16 {
                    let n = noise(ox + x, oy + y, 7) % 100;
                    let mut colour = if n < 6 {
                        DIRT_DARK
                    } else if n < 14 {
                        DIRT_LIGHT
                    } else {
                        DIRT
                    };
                    let edge = (!left && x == 0) || (!right && x == 15) || (!bottom && y == 15);
                    if edge {
                        colour = DIRT_DARK;
                    }
                    if !top {
                        colour = match y {
                            0 => GRASS_LIGHT,
                            1..=4 if n < 18 => GRASS_SHADE,
                            1..=4 => GRASS,
                            5 => GRASS_DARK,
                            _ => colour,
                        };
                        if y < 6 && ((!left && x == 0) || (!right && x == 15)) {
                            colour = GRASS_DARK;
                        }
                    }
                    c.set(ox + x, oy + y, colour);
                }
            }
            if !top {
                // Grass blades hanging over the dirt, continuous across tiles.
                for x in 0..16 {
                    if (ox + x) % 7 == 3 && (left || x > 0) && (right || x < 15) {
                        c.set(ox + x, oy + 6, GRASS_DARK);
                    }
                }
            }
        }
        // A 32 x 32 crate as four tiles at (0, 2).
        let crate_ = crate_box();
        for y in 0..32 {
            for x in 0..32 {
                c.set(x, 32 + y, crate_.get(x, y));
            }
        }
        // One-way platform: left cap, middle, right cap at (2..4, 2).
        for piece in 0..3 {
            let ox = 32 + piece * 16;
            c.rect(ox, 32, 16, 6, [206, 170, 108, 255]);
            c.rect(ox, 32, 16, 2, [232, 204, 144, 255]);
            c.rect(ox, 36, 16, 2, [156, 118, 70, 255]);
            if piece == 1 {
                c.rect(ox + 7, 34, 1, 2, [156, 118, 70, 255]);
            }
            if piece == 0 {
                c.rect(ox, 32, 1, 6, CLEAR);
            }
            if piece == 2 {
                c.rect(ox + 15, 32, 1, 6, CLEAR);
            }
            // Support posts under the caps.
            if piece != 1 {
                c.rect(ox + 6, 38, 3, 6, [156, 118, 70, 255]);
            }
        }
        c.outline(32, 32, 48, 16);
        // A flower with two frames (5, 2) and (6, 2): the head sways.
        for (frame, sway) in [(0, 0), (1, 1)] {
            let ox = 80 + frame * 16;
            c.rect(ox + 7, 40, 2, 8, GRASS_DARK);
            c.rect(ox + 5, 44, 2, 2, GRASS);
            c.rect(ox + 5 + sway, 35, 6, 6, [240, 96, 120, 255]);
            c.rect(ox + 7 + sway, 37, 2, 2, [255, 226, 110, 255]);
            c.outline(ox, 32, 16, 16);
        }
        // A bush (2, 3) and a grass tuft (3, 3).
        for y in 0..8 {
            for x in 0..14 {
                let (dx, dy) = (x as f32 - 6.5, y as f32 - 7.);
                if dx * dx / 49. + dy * dy / 64. <= 1. {
                    c.set(
                        33 + x,
                        56 + y,
                        if noise(x, y, 3).is_multiple_of(5) {
                            GRASS_SHADE
                        } else {
                            GRASS
                        },
                    );
                }
            }
        }
        c.outline(32, 48, 16, 16);
        for (x, h) in [(52, 5), (54, 7), (56, 4), (58, 6), (60, 3)] {
            c.rect(x, 64 - h, 1, h, GRASS_DARK);
        }
        c
    }
    /// The guide's cut-out parts on one 64 x 16 sheet: robe (0,0 12x16), head (16,0
    /// 12x12), upper arm (32,0 10x4) and forearm (32,6 10x4) drawn along +X from their
    /// joint, a hand (48,0 4x4), and the garland's light bulb (56,0 6x8).
    fn guide() -> Canvas {
        const ROBE: Rgba = [124, 84, 178, 255];
        const ROBE_DARK: Rgba = [92, 60, 140, 255];
        const BELT: Rgba = [232, 196, 92, 255];
        const SKIN: Rgba = [255, 206, 170, 255];
        const BEARD: Rgba = [236, 236, 244, 255];
        const EYE: Rgba = [24, 22, 36, 255];
        let mut c = Canvas::new(64, 16);
        // Robe, widening towards the hem.
        c.rect(3, 1, 6, 6, ROBE);
        c.rect(2, 7, 8, 5, ROBE);
        c.rect(1, 12, 10, 3, ROBE_DARK);
        c.rect(2, 8, 8, 1, BELT);
        c.outline(0, 0, 12, 16);
        // Head under a pointed hat, with a white beard.
        c.rect(19, 5, 6, 5, SKIN);
        c.rect(19, 9, 6, 2, BEARD);
        c.rect(20, 11, 4, 1, BEARD);
        c.set(23, 7, EYE);
        c.rect(18, 4, 8, 1, ROBE_DARK);
        c.rect(19, 2, 5, 2, ROBE);
        c.rect(20, 1, 3, 1, ROBE);
        c.set(21, 0, BELT);
        c.outline(16, 0, 12, 12);
        // Sleeves.
        c.rect(32, 1, 9, 2, ROBE);
        c.outline(32, 0, 10, 4);
        c.rect(32, 7, 9, 2, ROBE_DARK);
        c.outline(32, 6, 10, 4);
        // Hand.
        c.rect(49, 1, 2, 2, SKIN);
        c.outline(48, 0, 4, 4);
        // A light bulb (56,0 6x8), pale so a sprite's modulate colours it.
        c.rect(58, 0, 2, 2, [70, 70, 80, 255]);
        c.rect(57, 2, 4, 4, [236, 236, 236, 255]);
        c.rect(58, 6, 2, 1, [236, 236, 236, 255]);
        c.set(57, 2, [255, 255, 255, 255]);
        c.outline(56, 0, 6, 8);
        c
    }
    /// A soft 4 x 4 dot for particles (additive: dark texels add little).
    fn spark() -> Canvas {
        let mut c = Canvas::new(4, 4);
        for (x, y, v) in [
            (1, 1, 255),
            (2, 1, 255),
            (1, 2, 255),
            (2, 2, 255),
            (1, 0, 120),
            (2, 0, 120),
            (1, 3, 120),
            (2, 3, 120),
            (0, 1, 120),
            (0, 2, 120),
            (3, 1, 120),
            (3, 2, 120),
        ] {
            c.set(x, y, [v, v, v, 255]);
        }
        c
    }
    fn crate_box() -> Canvas {
        let mut c = Canvas::new(32, 32);
        c.rect(0, 0, 32, 32, [168, 112, 60, 255]);
        for i in 0..32 {
            c.set(i, i, [120, 78, 40, 255]);
            c.set(31 - i, i, [120, 78, 40, 255]);
        }
        c.rect(0, 0, 32, 3, [196, 142, 82, 255]);
        c.rect(0, 29, 32, 3, [110, 70, 36, 255]);
        c.rect(0, 0, 3, 32, [196, 142, 82, 255]);
        c.rect(29, 0, 3, 32, [110, 70, 36, 255]);
        for (x, y) in [(1, 1), (30, 1), (1, 30), (30, 30)] {
            c.set(x, y, [70, 46, 26, 255]);
        }
        c
    }
    fn coin() -> Canvas {
        let mut c = Canvas::new(64, 16);
        for (frame, half) in [5_i32, 3, 1, 3].into_iter().enumerate() {
            let cx = frame as i32 * 16 + 8;
            for y in 2..14 {
                let dy = (y as f32 - 7.5) / 6.;
                let width = ((1. - dy * dy).max(0.).sqrt() * half as f32).round() as i32;
                for x in cx - width - 1..=cx + width {
                    let edge = x == cx - width - 1 || x == cx + width;
                    c.set(
                        x,
                        y,
                        if edge {
                            [196, 132, 20, 255]
                        } else {
                            [252, 212, 64, 255]
                        },
                    );
                }
            }
            if half > 1 {
                c.rect(cx - 1, 5, 1, 5, [255, 244, 170, 255]);
            }
            c.outline(frame as i32 * 16, 0, 16, 16);
        }
        c
    }
    fn hills() -> Canvas {
        let mut c = Canvas::new(256, 64);
        for x in 0..256 {
            let t = x as f32 / 256. * std::f32::consts::TAU;
            let far = (28. + 10. * (t * 2.).sin() + 4. * (t * 5.).cos()) as i32;
            let near = (44. + 8. * (t * 3. + 1.).sin()) as i32;
            for y in 0..64 {
                if y >= 64 - near {
                    c.set(x, y, [70, 150, 96, 255]);
                } else if y >= 64 - far - 14 {
                    c.set(x, y, [120, 186, 160, 255]);
                }
            }
        }
        c
    }
    /// A 16 x 64 vertical gradient from deep blue at the top to a pale horizon,
    /// stretched over the whole screen behind everything.
    fn sky() -> Canvas {
        let mut c = Canvas::new(16, 64);
        let (top, horizon) = ([58., 112., 206.], [176., 218., 246.]);
        for y in 0..64 {
            let t = (y as f32 / 63.).powf(1.6);
            let colour = [0, 1, 2].map(|k| (top[k] + (horizon[k] - top[k]) * t) as u8);
            c.rect(0, y, 16, 1, [colour[0], colour[1], colour[2], 255]);
        }
        c
    }
    /// A 32 x 32 sun: a bright disc inside a soft ring, drawn additively.
    fn sun() -> Canvas {
        let mut c = Canvas::new(32, 32);
        for y in 0..32 {
            for x in 0..32 {
                let (dx, dy) = (x as f32 - 15.5, y as f32 - 15.5);
                let d = (dx * dx + dy * dy).sqrt();
                let colour = if d < 7. {
                    [255, 244, 196]
                } else if d < 15.5 {
                    // A glow that fades with distance; dark texels add little.
                    let t = 1. - (d - 7.) / 8.5;
                    [(200. * t) as u8, (150. * t) as u8, (70. * t) as u8]
                } else {
                    continue;
                };
                if colour.iter().all(|v| *v < 8) {
                    continue;
                }
                c.set(x, y, [colour[0], colour[1], colour[2], 255]);
            }
        }
        c
    }
    /// A 256 x 48 row of tree crowns and trunks, darker than the hills behind it.
    fn trees() -> Canvas {
        const CROWN: Rgba = [52, 120, 80, 255];
        const CROWN_LIGHT: Rgba = [70, 140, 92, 255];
        const TRUNK: Rgba = [70, 54, 44, 255];
        let mut c = Canvas::new(256, 48);
        for (i, (cx, r)) in [
            (14, 13),
            (40, 16),
            (70, 11),
            (96, 15),
            (128, 12),
            (156, 17),
            (190, 13),
            (220, 15),
            (246, 11),
        ]
        .into_iter()
        .enumerate()
        {
            let base = 48 - 8;
            c.rect(cx - 1, base - 6, 3, 14, TRUNK);
            let cy = base - 6 - r + 2;
            for y in (cy - r).max(0)..=(cy + r).min(47) {
                for x in cx - r..=cx + r {
                    let (dx, dy) = (x - cx, y - cy);
                    if dx * dx + dy * dy <= r * r {
                        let lit = dx + dy < -r / 2 && (x + y + i as i32) % 3 != 0;
                        c.set(x.rem_euclid(256), y, if lit { CROWN_LIGHT } else { CROWN });
                    }
                }
            }
        }
        // A ground strip, so the row sits on the hills without a gap.
        c.rect(0, 40, 256, 8, CROWN);
        c
    }
    /// A 128 x 24 strip of dark foreground bushes and grass blades.
    fn bushes() -> Canvas {
        const DARK: Rgba = [30, 74, 46, 255];
        const BLADE: Rgba = [40, 92, 54, 255];
        let mut c = Canvas::new(128, 24);
        for (cx, r) in [
            (10i32, 9i32),
            (26, 7),
            (60, 10),
            (78, 6),
            (104, 8),
            (118, 7),
        ] {
            for y in 0..24 {
                for x in cx - r..=cx + r {
                    let (dx, dy) = (x - cx, y - 24);
                    if dx * dx + dy * dy <= r * r * 2 {
                        c.set(x.rem_euclid(128), y, DARK);
                    }
                }
            }
        }
        for x in (2..128).step_by(5) {
            let h = 6 + (noise(x, 3, 7) % 7) as i32;
            for y in 24 - h..24 {
                c.set(x, y, BLADE);
            }
        }
        c
    }
    fn cloud() -> Canvas {
        let mut c = Canvas::new(64, 24);
        for (cx, cy, r) in [
            (16., 14., 9.),
            (30., 10., 11.),
            (46., 14., 9.),
            (30., 16., 9.),
        ] {
            for y in 0..24 {
                for x in 0..64 {
                    let (dx, dy) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
                    if dx * dx + dy * dy <= r * r {
                        c.set(
                            x,
                            y,
                            if dy > r * 0.45 {
                                [214, 230, 248, 255]
                            } else {
                                [250, 252, 255, 255]
                            },
                        );
                    }
                }
            }
        }
        c
    }

    pub fn install(root: &Path) -> Result<Textures, String> {
        std::fs::create_dir_all(root.join("assets/Sprites")).map_err(|e| e.to_string())?;
        let import = |name: &str, canvas: Canvas| -> Result<Uuid, String> {
            let source = format!("assets/Sprites/{name}.png");
            crate::project::write_changed(&root.join(&source), &canvas.png()?)?;
            crate::assets::commit(crate::texture::prepare(
                root,
                &source,
                &format!("assets/Sprites/{name}.epokasset"),
                None,
                false,
            )?)
        };
        Ok(Textures {
            player: import("Player", player())?,
            tiles: import("Tiles", tiles())?,
            coin: import("Coin", coin())?,
            hills: import("Hills", hills())?,
            cloud: import("Cloud", cloud())?,
            spark: import("Spark", spark())?,
            guide: import("Guide", guide())?,
            sky: import("Sky", sky())?,
            sun: import("Sun", sun())?,
            trees: import("Trees", trees())?,
            bushes: import("Bushes", bushes())?,
        })
    }
}

// ---- Blueprint flavor ------------------------------------------------------------------
mod blueprint {
    use super::*;

    mod op {
        pub const SET_VELOCITY: &str = "8cbd83fc-9005-4378-b295-17b7db2423c2";
        pub const VELOCITY_Y: &str = "a9dd10c9-5330-432b-a1a8-e045f3143111";
        pub const IS_ON_FLOOR: &str = "33d90de2-b12c-4a7d-8382-ef868816781d";
        pub const MOVE_AND_SLIDE: &str = "e8bdb3c0-41ed-42c9-9f8b-d994db924f9a";
        pub const PLAY: &str = "232abd11-a1eb-4041-ada7-20a45d58b72e";
        pub const SET_FLIP_H: &str = "480f44c0-403e-4090-b367-9e7f848cedd0";
    }
    const TICK: &str =
        "cpp:c:@N@epok@S@ActorComponent@F@tick#$@N@psyqo@S@FixedPoint>#Vi12#I#Vi4096#";
    const TRIGGER_EVENT: &str = "45c82d77-d076-43fe-b947-a4eed670f46b";
    const RIGHT: u64 = 5;
    const LEFT: u64 = 7;
    const CROSS: u64 = 14;

    fn ident(namespace: &str, key: &str) -> String {
        use sha2::{Digest, Sha256};
        let digest =
            Sha256::digest(format!("epok:platformer-2d-blueprint:{namespace}:{key}").as_bytes());
        let mut bytes: [u8; 16] = digest[..16].try_into().expect("a digest covers a UUID");
        bytes[6] = (bytes[6] & 15) | 0x50;
        bytes[8] = (bytes[8] & 63) | 0x80;
        Uuid::from_bytes(bytes).to_string()
    }
    #[derive(Clone)]
    enum Src {
        Pin(String),
        Lit(Type, serde_json::Value),
        Param(String),
    }
    fn out(node: &str) -> Src {
        Src::Pin(node.into())
    }
    fn fixed(value: f64) -> Src {
        Src::Lit(Type::Fixed, json!(value))
    }
    fn count(value: u64) -> Src {
        Src::Lit(Type::UInt32, json!(value))
    }
    fn flag(value: bool) -> Src {
        Src::Lit(Type::Bool, json!(value))
    }
    /// A compact graph builder: execution steps run left to right along a lane and
    /// pure data is stacked above the step that consumes it.
    struct Build {
        namespace: &'static str,
        graph: String,
        nodes: Vec<Node>,
        positions: BTreeMap<String, [f32; 2]>,
        column: i32,
        row: i32,
        stacked: BTreeMap<i32, i32>,
    }
    impl Build {
        fn new(namespace: &'static str, graph: &str) -> Self {
            Self {
                namespace,
                graph: graph.into(),
                nodes: vec![],
                positions: BTreeMap::new(),
                column: 0,
                row: 0,
                stacked: BTreeMap::new(),
            }
        }
        fn place(&mut self, key: &str, kind: NodeKind, column: i32, row: i32) -> String {
            let id = ident(self.namespace, &format!("{}/{key}", self.graph));
            self.positions
                .insert(id.clone(), [column as f32 * 240., row as f32 * 150.]);
            self.nodes.push(Node {
                id: id.clone(),
                kind,
                inputs: BTreeMap::new(),
                outputs: BTreeMap::new(),
            });
            id
        }
        fn step(&mut self, key: &str, kind: NodeKind) -> String {
            let (column, row) = (self.column, self.row);
            self.column += 1;
            self.place(key, kind, column, row)
        }
        fn data(&mut self, key: &str, kind: NodeKind) -> String {
            let depth = self.stacked.entry(self.column).or_default();
            let (offset, slot) = (*depth / 5, *depth % 5);
            *depth += 1;
            self.place(key, kind, self.column - offset, self.row - 1 - slot)
        }
        fn lane(&mut self, column: i32, row: i32) {
            self.column = column;
            self.row = row;
        }
        fn feed(&mut self, node: &str, pin: &str, src: Src) {
            let input = match src {
                Src::Pin(node) => Input::Link {
                    node,
                    pin: "value".into(),
                },
                Src::Lit(value_type, value) => Input::Literal { value_type, value },
                Src::Param(name) => Input::Parameter { name },
            };
            self.nodes
                .iter_mut()
                .find(|n| n.id == node)
                .expect("node of this graph")
                .inputs
                .insert(pin.into(), input);
        }
        fn exec(&mut self, from: &str, pin: &str, to: &str) {
            self.nodes
                .iter_mut()
                .find(|n| n.id == from)
                .expect("node of this graph")
                .outputs
                .insert(pin.into(), vec![to.into()]);
        }
        fn chain(&mut self, steps: &[String]) {
            for pair in steps.windows(2) {
                self.exec(&pair[0], "next", &pair[1]);
            }
        }
        fn read(&mut self, key: &str, variable: &str) -> String {
            self.data(
                key,
                NodeKind::GetVariable {
                    member: ident("variable", variable),
                },
            )
        }
        fn binary(&mut self, key: &str, op: BinaryOp, a: Src, b: Src) -> String {
            let node = self.data(key, NodeKind::Binary { op });
            self.feed(&node, "a", a);
            self.feed(&node, "b", b);
            node
        }
        fn input(&mut self, key: &str, operation: Builtin, button: u64) -> String {
            let node = self.data(key, NodeKind::Builtin { operation });
            self.feed(&node, "button", count(button));
            self.feed(&node, "port", count(0));
            node
        }
        fn query(&mut self, key: &str, operation: &str, target: Src) -> String {
            let node = self.data(
                key,
                NodeKind::Operation {
                    operation: operation.into(),
                },
            );
            self.feed(&node, "__target", target);
            node
        }
        fn perform_on(&mut self, key: &str, operation: &str, target: Src, args: &[Src]) -> String {
            let node = self.step(
                key,
                NodeKind::Operation {
                    operation: operation.into(),
                },
            );
            self.feed(&node, "__target", target);
            for (index, value) in args.iter().enumerate() {
                self.feed(
                    &node,
                    &format!("{operation}:parameter:{index}"),
                    value.clone(),
                );
            }
            node
        }
        fn call(&mut self, key: &str, graph: &str, args: &[(&str, Src)]) -> String {
            let node = self.step(
                key,
                NodeKind::Call {
                    function: ident("graph", graph),
                },
            );
            for (pin, value) in args {
                self.feed(&node, pin, value.clone());
            }
            node
        }
        fn finish(
            self,
            name: &str,
            override_id: Option<&str>,
            parameters: Vec<schema::Parameter>,
            returns: Type,
        ) -> (Graph, BTreeMap<String, [f32; 2]>) {
            (
                Graph {
                    id: ident("graph", name),
                    name: name.into(),
                    override_id: override_id.map(str::to_owned),
                    timeline: None,
                    parameters,
                    returns,
                    entry: ident(self.namespace, &format!("{}/entry", self.graph)),
                    nodes: self.nodes,
                },
                self.positions,
            )
        }
    }
    fn parameter(name: &str, value_type: Type) -> schema::Parameter {
        schema::Parameter {
            name: name.into(),
            value_type,
            direction: schema::Direction::Value,
        }
    }
    fn variable(
        name: &str,
        value_type: Type,
        default: serde_json::Value,
        editable: bool,
    ) -> Variable {
        Variable {
            id: ident("variable", name),
            name: name.into(),
            value_type,
            default,
            editable,
            timeline_animatable: false,
        }
    }
    fn component_asset(name: &str, id: &str) -> BlueprintAsset {
        let mut asset =
            BlueprintAsset::new(name.into(), crate::object_model::ACTOR_COMPONENT_ID.into());
        asset.id = id.into();
        asset.family = Some(schema::ClassFamily::Component);
        asset.component = Some(schema::ComponentContract {
            owners: [schema::Domain::World2D].into_iter().collect(),
            ..Default::default()
        });
        asset
    }

    /// The PlatformerController graph: the same movement as the C++ and Lua flavors.
    pub fn controller() -> BlueprintAsset {
        let mut asset = component_asset(CONTROLLER, BLUEPRINT_CONTROLLER_ID);
        let reference = |class: &str| Type::ComponentRef {
            class: Some(class.into()),
        };
        asset.variables = vec![
            variable(
                "body",
                reference(world2d::CHARACTER_BODY2D),
                json!(null),
                true,
            ),
            variable("sprite", reference(world2d::SPRITE2D), json!(null), true),
            variable("speed", Type::Fixed, json!(96.0), true),
            variable("jump_speed", Type::Fixed, json!(264.0), true),
            variable("gravity", Type::Fixed, json!(720.0), true),
            variable("idle_animation", Type::UInt32, json!(0), true),
            variable("run_animation", Type::UInt32, json!(1), true),
        ];
        // pick(condition, when_true, when_false): a conditional value.
        let mut b = Build::new("controller", "pick");
        let entry = b.place("entry", NodeKind::Entry, 0, 0);
        b.lane(1, 0);
        let branch = b.step("branch", NodeKind::Branch);
        b.feed(&branch, "condition", Src::Param("condition".into()));
        b.exec(&entry, "next", &branch);
        b.lane(2, 1);
        let yes = b.step("yes", NodeKind::Return);
        b.feed(&yes, "value", Src::Param("when_true".into()));
        b.lane(2, 2);
        let no = b.step("no", NodeKind::Return);
        b.feed(&no, "value", Src::Param("when_false".into()));
        b.exec(&branch, "true", &yes);
        b.exec(&branch, "false", &no);
        let (pick, pick_layout) = b.finish(
            "pick",
            None,
            vec![
                parameter("condition", Type::Bool),
                parameter("when_true", Type::Fixed),
                parameter("when_false", Type::Fixed),
            ],
            Type::Fixed,
        );
        // tick: input to velocity, move_and_slide, then face and animate.
        let mut b = Build::new("controller", "tick");
        let entry = b.place("entry", NodeKind::Entry, 0, 0);
        b.lane(1, 0);
        let speed = b.read("speed", "speed");
        let held_right = b.input("held_right", Builtin::InputHeld, RIGHT);
        let right = b.call(
            "right",
            "pick",
            &[
                ("condition", out(&held_right)),
                ("when_true", out(&speed)),
                ("when_false", fixed(0.)),
            ],
        );
        let speed_left = b.read("speed_left", "speed");
        let held_left = b.input("held_left", Builtin::InputHeld, LEFT);
        let left = b.call(
            "left",
            "pick",
            &[
                ("condition", out(&held_left)),
                ("when_true", out(&speed_left)),
                ("when_false", fixed(0.)),
            ],
        );
        let body = b.read("body", "body");
        let velocity_y = b.query("velocity_y", op::VELOCITY_Y, out(&body));
        let gravity = b.read("gravity", "gravity");
        let pull = b.binary(
            "pull",
            BinaryOp::Multiply,
            out(&gravity),
            Src::Param("delta_seconds".into()),
        );
        let fall = b.binary("fall", BinaryOp::Add, out(&velocity_y), out(&pull));
        let floor = b.query("on_floor", op::IS_ON_FLOOR, out(&body));
        let pressed = b.input("jump_pressed", Builtin::InputPressed, CROSS);
        let jumping = b.binary("jumping", BinaryOp::And, out(&floor), out(&pressed));
        let jump_speed = b.read("jump_speed", "jump_speed");
        let launch = b.binary("launch", BinaryOp::Subtract, fixed(0.), out(&jump_speed));
        let vertical = b.call(
            "vertical",
            "pick",
            &[
                ("condition", out(&jumping)),
                ("when_true", out(&launch)),
                ("when_false", out(&fall)),
            ],
        );
        let body_set = b.read("body_set", "body");
        let moving = b.binary("move", BinaryOp::Subtract, out(&right), out(&left));
        let set_velocity = b.perform_on(
            "set_velocity",
            op::SET_VELOCITY,
            out(&body_set),
            &[out(&moving), out(&vertical)],
        );
        let body_slide = b.read("body_slide", "body");
        let slide = b.perform_on("slide", op::MOVE_AND_SLIDE, out(&body_slide), &[]);
        let has_move = b.binary("has_move", BinaryOp::NotEqual, out(&moving), fixed(0.));
        let branch = b.step("moving", NodeKind::Branch);
        b.feed(&branch, "condition", out(&has_move));
        b.chain(&[
            entry,
            right,
            left,
            vertical,
            set_velocity,
            slide,
            branch.clone(),
        ]);
        b.lane(8, 1);
        let sprite = b.read("sprite_face", "sprite");
        let facing_left = b.binary("facing_left", BinaryOp::Less, out(&moving), fixed(0.));
        let face = b.perform_on("face", op::SET_FLIP_H, out(&sprite), &[out(&facing_left)]);
        let sprite_run = b.read("sprite_run", "sprite");
        let run_animation = b.read("run_animation", "run_animation");
        let run = b.perform_on("run", op::PLAY, out(&sprite_run), &[out(&run_animation)]);
        b.exec(&branch, "true", &face);
        b.exec(&face, "next", &run);
        b.lane(8, 3);
        let sprite_idle = b.read("sprite_idle", "sprite");
        let idle_animation = b.read("idle_animation", "idle_animation");
        let idle = b.perform_on("idle", op::PLAY, out(&sprite_idle), &[out(&idle_animation)]);
        b.exec(&branch, "false", &idle);
        let (tick, tick_layout) = b.finish(
            "tick",
            Some(TICK),
            vec![parameter("delta_seconds", Type::Fixed)],
            Type::Void,
        );
        for (graph, layout) in [(tick, tick_layout), (pick, pick_layout)] {
            asset.layout.positions.extend(layout);
            asset.functions.push(graph);
        }
        asset
    }

    /// CoinPickup: deactivate the owner the first time something enters the trigger.
    pub fn coin() -> BlueprintAsset {
        let mut asset = component_asset(COIN, BLUEPRINT_COIN_ID);
        let phase = Type::Enum {
            cpp_name: "epok::TriggerPhase".into(),
            variants: [("Enter", 0), ("Stay", 1), ("Exit", 2)]
                .into_iter()
                .map(|(n, v)| (n.to_string(), v))
                .collect(),
        };
        let mut b = Build::new("coin", "trigger_event");
        let entry = b.place("entry", NodeKind::Entry, 0, 0);
        b.lane(1, 0);
        let entered = b.binary(
            "entered",
            BinaryOp::Equal,
            Src::Param("phase".into()),
            Src::Lit(phase.clone(), json!(0)),
        );
        let branch = b.step("branch", NodeKind::Branch);
        b.feed(&branch, "condition", out(&entered));
        b.exec(&entry, "next", &branch);
        b.lane(2, 1);
        let owner = b.data(
            "owner",
            NodeKind::Builtin {
                operation: Builtin::GetOwner,
            },
        );
        let hide = b.step(
            "hide",
            NodeKind::Builtin {
                operation: Builtin::SetActive,
            },
        );
        b.feed(&hide, "target", out(&owner));
        b.feed(&hide, "active", flag(false));
        b.exec(&branch, "true", &hide);
        let (graph, layout) = b.finish(
            "trigger_event",
            Some(TRIGGER_EVENT),
            vec![
                parameter("other", Type::ObjectRef { class: None }),
                parameter("phase", phase),
            ],
            Type::Void,
        );
        asset.layout.positions.extend(layout);
        asset.functions.push(graph);
        asset
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_flavor_creates_a_valid_platformer_with_its_gameplay() {
        for flavor in [
            GameplayFlavor::Cpp,
            GameplayFlavor::Lua,
            GameplayFlavor::Blueprint,
        ] {
            let root =
                std::env::temp_dir().join(format!("epok-platformer-{flavor:?}-{}", Uuid::new_v4()));
            std::fs::create_dir_all(root.join("assets/scenes")).unwrap();
            std::fs::create_dir_all(root.join("assets/scripts")).unwrap();
            let scene = create(&root, flavor).unwrap();
            scene.validate().unwrap();
            let player = scene.actors.iter().find(|a| a.name == "Player").unwrap();
            for class in [
                world2d::SPRITE2D,
                world2d::COLLIDER2D,
                world2d::CHARACTER_BODY2D,
                world2d::CAMERA2D,
            ] {
                assert!(
                    world2d::component(player, class).is_some(),
                    "{flavor:?} player lacks {class}"
                );
            }
            let controller = player
                .components
                .iter()
                .find(|c| c.class.name == CONTROLLER)
                .unwrap();
            assert_eq!(
                controller.class.class_id.as_deref(),
                Some(controller_id(flavor))
            );
            assert_eq!(
                controller.properties.contains_key("body"),
                flavor != GameplayFlavor::Cpp
            );
            assert_eq!(
                scene
                    .actors
                    .iter()
                    .filter(|a| a.components.iter().any(|c| c.class.name == COIN))
                    .count(),
                8
            );
            // Every sprite points at an imported texture of the project.
            let index = crate::assets::scan(&root, &mut Default::default());
            for actor in &scene.actors {
                for id in world2d::texture_ids(actor, &scene.tilesets) {
                    assert_eq!(
                        index.resolve(id).unwrap().meta.kind,
                        crate::assets::Kind::Texture
                    );
                }
            }
            // A new project opens without a warning sign anywhere in its Hierarchy.
            for i in 0..scene.actors.len() {
                let warnings = crate::world2d_warnings::of(&scene, i);
                assert!(
                    warnings.is_empty(),
                    "{flavor:?}: {} warns {warnings:?}",
                    scene.actors[i].name
                );
            }
            let expected = match flavor {
                GameplayFlavor::Cpp => "assets/scripts/PlatformerController.hpp",
                GameplayFlavor::Lua => LUA_CONTROLLER_PATH,
                GameplayFlavor::Blueprint => BLUEPRINT_CONTROLLER_PATH,
            };
            assert!(
                root.join(expected).is_file(),
                "{flavor:?} wrote no {expected}"
            );
            std::fs::remove_dir_all(&root).ok();
        }
    }
    #[test]
    fn generated_blueprints_are_stable() {
        let a = serde_json::to_string(&blueprint::controller()).unwrap();
        let b = serde_json::to_string(&blueprint::controller()).unwrap();
        assert_eq!(a, b);
        assert_eq!(
            serde_json::to_string(&blueprint::coin()).unwrap(),
            serde_json::to_string(&blueprint::coin()).unwrap()
        );
    }
}
