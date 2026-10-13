//! The Platformer 2D starter: a side-scrolling level built entirely from World2D
//! nodes - an animated CharacterBody2D player with a following Camera2D, solid
//! ground and crates, one-way platforms, spinning coins and decorative hills -
//! with its gameplay written in the selected flavor. The artwork is original pixel
//! art generated here at creation time, so the template ships no binary files.
use crate::{
    actor_document::{ClassReference, ComponentInstance},
    blueprint_asset::{BinaryOp, BlueprintAsset, Builtin, Graph, Input, Node, NodeKind, Variable},
    reflection_schema::{self as schema, Type},
    scene::{Actor, Scene},
    workspace::GameplayFlavor,
    world2d::{self, Animation2D, Camera2D, Collider2D, Frame2D, Preset, Sprite2D},
};
use serde_json::json;
use std::{collections::BTreeMap, path::Path};
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
    let decor = group(&mut scene, "Decor");
    let level = group(&mut scene, "Level");
    let collectibles = group(&mut scene, "Coins");
    // Decoration first: drawn behind everything at z -10.
    for (i, x) in [-384., -128., 128., 384.].into_iter().enumerate() {
        let mut hills = world2d::create(Preset::Sprite2D, &format!("Hills{i}"), [x, GROUND - 32.]);
        set(
            &mut hills,
            world2d::SPRITE2D,
            &Sprite2D {
                z_index: -10,
                ..sprite(art.hills)
            },
        );
        hills.logical_parent = Some(decor);
        scene.actors.push(hills);
    }
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
        cloud.logical_parent = Some(decor);
        scene.actors.push(cloud);
    }
    // Solid ground, seven 128-pixel strips from x = -448 to 448.
    for i in 0..7 {
        let x = -384. + 128. * i as f32;
        let mut ground = world2d::create(
            Preset::StaticBody2D,
            &format!("Ground{i}"),
            [x, GROUND + 16.],
        );
        set(
            &mut ground,
            world2d::COLLIDER2D,
            &Collider2D {
                size: [128., 32.],
                ..Default::default()
            },
        );
        add(&mut ground, world2d::SPRITE2D, &sprite(art.ground));
        ground.logical_parent = Some(level);
        scene.actors.push(ground);
    }
    for (i, [x, y]) in [
        [-424., GROUND - 16.],
        [-424., GROUND - 48.],
        [256., GROUND - 16.],
    ]
    .into_iter()
    .enumerate()
    {
        let mut crate_ = world2d::create(Preset::StaticBody2D, &format!("Crate{i}"), [x, y]);
        set(
            &mut crate_,
            world2d::COLLIDER2D,
            &Collider2D {
                size: [32., 32.],
                ..Default::default()
            },
        );
        add(&mut crate_, world2d::SPRITE2D, &sprite(art.crate_));
        crate_.logical_parent = Some(level);
        scene.actors.push(crate_);
    }
    let platforms = [[-176., 44.], [-16., 8.], [144., -20.], [336., 36.]];
    for (i, [x, y]) in platforms.into_iter().enumerate() {
        let mut platform = world2d::create(Preset::StaticBody2D, &format!("Platform{i}"), [x, y]);
        set(
            &mut platform,
            world2d::COLLIDER2D,
            &Collider2D {
                size: [48., 10.],
                one_way: true,
                one_way_margin: 3.,
                ..Default::default()
            },
        );
        add(&mut platform, world2d::SPRITE2D, &sprite(art.platform));
        platform.logical_parent = Some(level);
        scene.actors.push(platform);
    }
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
    let mut coins = Vec::new();
    for (i, [x, y]) in [
        [-192., 24.],
        [-160., 24.],
        [-16., -12.],
        [144., -40.],
        [96., 72.],
        [128., 72.],
        [336., 16.],
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
    scene.actors.push(player);
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
        pub ground: Uuid,
        pub crate_: Uuid,
        pub platform: Uuid,
        pub coin: Uuid,
        pub hills: Uuid,
        pub cloud: Uuid,
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
    fn ground() -> Canvas {
        let mut c = Canvas::new(128, 32);
        for y in 0..32 {
            for x in 0..128 {
                let n = noise(x, y, 7) % 100;
                let colour = match y {
                    0 => [124, 214, 88, 255],
                    1..=4 => {
                        if n < 18 {
                            [96, 188, 72, 255]
                        } else {
                            [104, 198, 76, 255]
                        }
                    }
                    5 => [62, 140, 56, 255],
                    _ if n < 6 => [92, 60, 40, 255],
                    _ if n < 14 => [154, 104, 66, 255],
                    _ => [136, 90, 56, 255],
                };
                c.set(x, y, colour);
            }
            // Grass blades hanging over the dirt.
        }
        for x in (3..128).step_by(7) {
            c.set(x, 6, [62, 140, 56, 255]);
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
    fn platform() -> Canvas {
        let mut c = Canvas::new(48, 10);
        c.rect(1, 1, 46, 8, [206, 170, 108, 255]);
        c.rect(1, 1, 46, 2, [232, 204, 144, 255]);
        c.rect(1, 7, 46, 2, [156, 118, 70, 255]);
        for x in [12, 24, 36] {
            c.rect(x, 3, 1, 4, [156, 118, 70, 255]);
        }
        c.outline(0, 0, 48, 10);
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
            ground: import("Ground", ground())?,
            crate_: import("Crate", crate_box())?,
            platform: import("Platform", platform())?,
            coin: import("Coin", coin())?,
            hills: import("Hills", hills())?,
            cloud: import("Cloud", cloud())?,
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
                for id in world2d::texture_ids(actor) {
                    assert_eq!(
                        index.resolve(id).unwrap().meta.kind,
                        crate::assets::Kind::Texture
                    );
                }
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
