//! Editable greybox arena with platforms, ramps and a static mannequin.
//! Layout, dimensions and PSX rendering details: docs/third-person.md.
use crate::{
    lighting, mesh,
    scene::{Actor, Material, Scene},
    viewport::View,
};
use std::{path::Path, sync::Arc};
use uuid::Uuid;

const CONCRETE: [f32; 3] = [0.64, 0.65, 0.67];
const PLATFORM: [f32; 3] = [0.43, 0.44, 0.46];
const BLUE: [f32; 3] = [0.015, 0.36, 0.85];

struct Builder {
    doc: mesh::Document,
    group: Uuid,
    surface: Uuid,
    seam: Uuid,
}
impl Builder {
    fn new(name: &str, color: [f32; 3]) -> Self {
        let mut doc = mesh::Document::default();
        doc.groups[0].name = name.into();
        doc.materials[0].name = "Surface".into();
        doc.materials[0].material.color = color;
        let seam = Uuid::new_v4();
        doc.materials.push(mesh::Slot {
            id: seam,
            name: "Grid joints".into(),
            material: Material {
                color: color.map(|v| v * 0.68),
                unlit: false,
                ..Default::default()
            },
        });
        Self {
            group: doc.groups[0].id,
            surface: doc.materials[0].id,
            seam,
            doc,
        }
    }
    fn group(&mut self, name: &str) {
        self.group = Uuid::new_v4();
        self.doc.groups.push(mesh::Group {
            id: self.group,
            name: name.into(),
            parent: Some(self.doc.groups[0].id),
        });
    }
    fn face(&mut self, points: [[f32; 3]; 4]) {
        self.doc.add_face(points, self.group, self.surface);
    }
    // Grid lines are disjoint colored faces, not overlapping decals or texture assets.
    fn grid(&mut self, p: [[f32; 3]; 4], spacing: f32) {
        let distance = |a: [f32; 3], b: [f32; 3]| {
            a.iter()
                .zip(b)
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f32>()
                .sqrt()
        };
        let nu = (distance(p[0], p[1]) / spacing).ceil().max(1.) as usize;
        let nv = (distance(p[0], p[3]) / spacing).ceil().max(1.) as usize;
        let at = |u: f32, v: f32| {
            std::array::from_fn(|c| p[0][c] + (p[1][c] - p[0][c]) * u + (p[3][c] - p[0][c]) * v)
        };
        for j in 0..nv {
            for i in 0..nu {
                let (u, v) = (i as f32 / nu as f32, j as f32 / nv as f32);
                let (u1, v1) = ((i + 1) as f32 / nu as f32, (j + 1) as f32 / nv as f32);
                let (us, vs) = (u + 0.10 / nu as f32, v + 0.10 / nv as f32);
                for (a, b, c, d, slot) in [
                    (u, v, us, v1, self.seam),
                    (us, v, u1, vs, self.seam),
                    (us, vs, u1, v1, self.surface),
                ] {
                    self.doc
                        .add_face([at(a, b), at(c, b), at(c, d), at(a, d)], self.group, slot);
                }
            }
        }
    }
    fn primitive(&mut self, kind: &str, center: [f32; 3], size: [f32; 3]) {
        self.doc
            .primitive(kind, center, size, 1, self.group, self.surface);
    }
    // Extrude a counter-clockwise footprint in x/z. Curves are authored as a few flat segments.
    fn prism(&mut self, polygon: &[[f32; 2]], height: f32) {
        self.sloped_prism(polygon, |_| height);
    }
    fn sloped_prism(&mut self, polygon: &[[f32; 2]], height: impl Fn([f32; 2]) -> f32) {
        let n = polygon.len();
        for i in 0..n {
            let a = polygon[i];
            let b = polygon[(i + 1) % n];
            self.face([
                [a[0], 0., a[1]],
                [a[0], height(a), a[1]],
                [b[0], height(b), b[1]],
                [b[0], 0., b[1]],
            ]);
        }
        // Fan around the first point; reversed winding makes the roof face upwards.
        for i in 1..n - 1 {
            self.face([
                [polygon[0][0], height(polygon[0]), polygon[0][1]],
                [polygon[i + 1][0], height(polygon[i + 1]), polygon[i + 1][1]],
                [polygon[i][0], height(polygon[i]), polygon[i][1]],
                [polygon[i][0], height(polygon[i]), polygon[i][1]],
            ]);
        }
    }
    fn finish(mut self, root: &Path, name: &str) -> Result<Actor, String> {
        self.doc.compact();
        self.doc.validate()?;
        let asset = mesh::create(root, &format!("assets/Meshes/{name}.epokasset"), &self.doc)?;
        let mut e = Actor::cube(name.into());
        e.position = [0.; 3];
        e.editable_mesh = Some(mesh::Component {
            document: Some(Arc::new(self.doc)),
            ..mesh::Component::new(asset)
        });
        e.lighting.receive = lighting::Receive::Baked;
        e.lighting.static_geometry = true;
        Ok(e)
    }
}

pub fn overview() -> View {
    let mut v = View {
        yaw: 0.,
        pitch: 0.58,
        zoom: 0.76,
        ..View::default()
    };
    let forward = v.basis()[2];
    v.center = std::array::from_fn(|i| [0., -3., 0.][i] - forward[i] * 29.);
    v
}

pub fn create(root: &Path) -> Result<Scene, String> {
    // The controller drives this every frame; the authored transform is only the
    // pose the editor viewport and the first frame start from.
    let mut camera = Actor::cube("Follow Camera".into());
    camera.kind = "Camera".into();
    camera.position = [-7., 4.5, -13.5];
    camera.rotation = [20., 0., 0.];
    let mut scene = Scene {
        name: "ThirdPersonArena".into(),
        actors: vec![camera],
        ..Scene::default()
    };
    scene.environment.ambient = [0.53, 0.55, 0.59];
    scene.environment.point_lights = false;

    let mut floor = Builder::new("Floor", CONCRETE);
    floor.grid(
        [
            [-16., 0., -16.],
            [-16., 0., 16.],
            [16., 0., 16.],
            [16., 0., -16.],
        ],
        4.,
    );
    let mut floor_entity = floor.finish(root, "Arena Floor")?;
    // Collision is axis-aligned boxes only, so the ground is one slab sitting
    // just under the surface the grid draws.
    floor_entity.collider = Some(crate::collision::Collider {
        center: [0., -0.5, 0.],
        half_extents: [16., 0.5, 16.],
        ..Default::default()
    });
    scene.actors.push(floor_entity);

    let mut walls = Builder::new("Perimeter", CONCRETE);
    for (name, a, b, normal) in [
        ("North wall", [-16., 16.], [16., 16.], [0., -1.]),
        ("East wall", [16., 16.], [16., -16.], [-1., 0.]),
        ("South wall", [16., -16.], [-16., -16.], [0., 1.]),
        ("West wall", [-16., -16.], [-16., 16.], [1., 0.]),
    ] {
        walls.group(name);
        walls.grid(
            [
                [a[0], 0., a[1]],
                [a[0], 4., a[1]],
                [b[0], 4., b[1]],
                [b[0], 0., b[1]],
            ],
            4.,
        );
        let ao = [a[0] - normal[0] * 0.5, a[1] - normal[1] * 0.5];
        let bo = [b[0] - normal[0] * 0.5, b[1] - normal[1] * 0.5];
        walls.grid(
            [
                [bo[0], -0.5, bo[1]],
                [bo[0], 4., bo[1]],
                [ao[0], 4., ao[1]],
                [ao[0], -0.5, ao[1]],
            ],
            4.,
        );
        walls.face([
            [a[0], 4., a[1]],
            [ao[0], 4., ao[1]],
            [bo[0], 4., bo[1]],
            [b[0], 4., b[1]],
        ]);
    }
    scene.actors.push(walls.finish(root, "Arena Walls")?);
    // One entity carries one box, so the perimeter needs four of them. They are
    // invisible: the visible wall geometry is the single mesh pushed above.
    for (name, center, half_extents) in [
        ("North wall collider", [0., 2., 16.], [16.5, 2., 0.5]),
        ("East wall collider", [16., 2., 0.], [0.5, 2., 16.5]),
        ("South wall collider", [0., 2., -16.], [16.5, 2., 0.5]),
        ("West wall collider", [-16., 2., 0.], [0.5, 2., 16.5]),
    ] {
        let mut bound = Actor::cube(name.into());
        bound.kind = "Empty".into();
        bound.collider = Some(crate::collision::Collider {
            center: [0., 0., 0.],
            half_extents,
            ..Default::default()
        });
        bound.position = center;
        scene.actors.push(bound);
    }
    // Platform collision, as invisible boxes. Sloped approaches carry a
    // slope_rise, so the solver lifts the character along the surface instead of
    // stopping it at a wall; the rise matches the geometry the mesh draws.
    for (name, center, half_extents, slope) in [
        // Flat decks.
        (
            "Central deck collider",
            [2.25, 1., 3.],
            [3.25, 1., 3.],
            None,
        ),
        (
            "Long platform collider",
            [10., 1.25, 6.],
            [1.75, 1.25, 6.],
            None,
        ),
        (
            "Round platform collider",
            [-10., 0.625, 9.],
            [2., 0.625, 2.],
            None,
        ),
        // Ramps. The Ramp primitive rises toward +Z, so does the collider.
        (
            "Small ramp collider",
            [-10., 0.22, 1.5],
            [1., 0.22, 2.],
            Some((0.44, 2)),
        ),
        (
            "West ramp collider",
            [-2.5, 1., 3.],
            [1.5, 1., 3.],
            Some((2., 2)),
        ),
        (
            "Rear wedge collider",
            [1., 1., 7.],
            [2., 1., 1.],
            Some((2., 2)),
        ),
        (
            "Low ramp collider",
            [9.5, 0.75, -9.],
            [2.5, 0.75, 2.5],
            Some((1.42, 2)),
        ),
    ] {
        let (slope_rise, slope_axis) = slope.unwrap_or((0., 0));
        let mut solid = Actor::cube(name.into());
        solid.kind = "Empty".into();
        solid.collider = Some(crate::collision::Collider {
            center: [0., 0., 0.],
            half_extents,
            slope_rise,
            slope_axis,
            ..Default::default()
        });
        solid.position = center;
        scene.actors.push(solid);
    }

    let mut central = Builder::new("Central platform", PLATFORM);
    // Rectangular deck with the recognizable rounded front-right corner.
    central.prism(
        &[
            [-1., 0.],
            [4., 0.],
            [4.57, 0.11],
            [5.06, 0.44],
            [5.39, 0.93],
            [5.5, 1.5],
            [5.5, 6.],
            [-1., 6.],
        ],
        2.,
    );
    central.group("West access ramp");
    central.primitive("Ramp", [-2.5, 1., 3.], [3., 2., 6.]);
    central.group("Rear wedge");
    central.primitive("Ramp", [1., 1., 7.], [4., 2., 2.]);
    scene
        .actors
        .push(central.finish(root, "Central Platform and Ramps")?);

    let mut deck = Builder::new("Long east platform", PLATFORM);
    deck.primitive("Box", [10., 1.25, 6.], [3.5, 2.5, 12.]);
    scene.actors.push(deck.finish(root, "Long Platform")?);

    let mut ramp = Builder::new("South east ramp", PLATFORM);
    ramp.sloped_prism(
        &[
            [7., -11.5],
            [11., -11.5],
            [11.38, -11.42],
            [11.71, -11.21],
            [11.92, -10.88],
            [12., -10.5],
            [12., -6.5],
            [7., -6.5],
        ],
        |p| 0.08 + (p[1] + 11.5) / 5. * 1.42,
    );
    scene.actors.push(ramp.finish(root, "Low Ramp")?);

    let mut cylinder = Builder::new("Twelve sided cylinder", PLATFORM);
    let points: Vec<_> = (0..12)
        .map(|i| {
            let a = i as f32 * std::f32::consts::TAU / 12.;
            [-10. + a.cos() * 2., 9. + a.sin() * 2.]
        })
        .collect();
    cylinder.prism(&points, 1.25);
    scene.actors.push(cylinder.finish(root, "Round Platform")?);

    let mut small = Builder::new("West low wedge", PLATFORM);
    small.primitive("Ramp", [-10., 0.22, 1.5], [2., 0.44, 4.]);
    scene.actors.push(small.finish(root, "Small Ramp")?);

    for (i, p, angle) in [
        (0, [3., 0.65, -1.5], 0.),
        (1, [10., 0.65, -3.5], 18.),
        (2, [5.5, 0.65, -11.], -20.),
    ] {
        let mut cube = Actor::cube(format!("Blue Cube {} - static placeholder", i + 1));
        cube.position = p;
        cube.rotation[1] = angle;
        cube.scale = [1.3; 3];
        cube.material.color = BLUE;
        cube.lighting.receive = lighting::Receive::Baked;
        cube.lighting.static_geometry = true;
        scene.actors.push(cube);
    }
    let mut start = Actor::cube("Player".into());
    start.kind = "Empty".into();
    start.position = [-7., 0., -6.];
    // move_and_slide and query_ground both need a body to sweep; the box wraps
    // the placeholder limbs parented below.
    start.collider = Some(crate::collision::Collider {
        center: [0., 0.9, 0.],
        half_extents: [0.32, 0.9, 0.32],
        ..Default::default()
    });
    start
        .components
        .push(crate::actor_document::ComponentInstance::new(
            Uuid::new_v4(),
            crate::actor_document::ClassReference::new(
                "ThirdPersonController",
                "9233f481-d27e-4765-a2d3-4dcfcb0cc910",
            ),
            "Third Person Controller",
        ));
    let parent = scene.actors.len();
    scene.actors.push(start);
    for (name, position, scale) in [
        ("Head", [0., 1.62, 0.], [0.3, 0.34, 0.3]),
        ("Torso", [0., 1.16, 0.], [0.48, 0.6, 0.28]),
        ("Left leg", [-0.15, 0.43, 0.], [0.18, 0.84, 0.22]),
        ("Right leg", [0.15, 0.43, 0.], [0.18, 0.84, 0.22]),
        ("Left arm", [-0.36, 1.08, 0.], [0.16, 0.64, 0.18]),
        ("Right arm", [0.36, 1.08, 0.], [0.16, 0.64, 0.18]),
    ] {
        let mut e = Actor::cube(format!("Placeholder {name}"));
        e.parent = Some(parent);
        e.position = position;
        e.scale = scale;
        e.material.color = [0.75, 0.77, 0.8];
        scene.actors.push(e);
    }
    let mut sun = Actor::cube("Sun".into());
    sun.kind = "Empty".into();
    sun.rotation = [58., -32., 0.];
    sun.light = Some(lighting::Light {
        mode: lighting::LightMode::Mixed,
        intensity: 0.65,
        ..Default::default()
    });
    scene.actors.push(sun);
    scene.sync_actor_components();
    scene.bake = Some(lighting::bake(&scene)?);
    scene.validate()?;
    std::fs::create_dir_all(root.join("UserSettings")).map_err(|e| e.to_string())?;
    crate::project::write_changed(
        &root.join("UserSettings/SceneView.epokprefs"),
        &crate::document::to_vec(&overview()).map_err(|e| e.to_string())?,
    )?;
    crate::project::write_changed(
        &root.join("README.md"),
        include_bytes!("../resources/templates/third-person/README.md"),
    )?;
    for (name, source) in [
        (
            "assets/scripts/ThirdPersonController.hpp",
            include_str!("../templates/ThirdPersonController.hpp"),
        ),
        (
            "assets/scripts/ThirdPersonController.cpp",
            include_str!("../templates/ThirdPersonController.cpp"),
        ),
    ] {
        let path = crate::assets::inside(root, name)?;
        crate::project::write_changed(&path, source.as_bytes())?;
    }
    Ok(scene)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn arena_template_resolves_compiles_and_keeps_editable_assets() {
        let parent = std::env::temp_dir().join(format!("epok-third-person-{}", Uuid::new_v4()));
        let root = parent.join("Third Person Game");
        let project = crate::workspace::create(
            &root,
            "Third Person Game",
            crate::workspace::Template::ThirdPerson,
        )
        .unwrap();
        let scene = Scene::load(&crate::workspace::startup_scene(&root).unwrap()).unwrap();
        assert!(lighting::valid_bake(&scene));
        assert_eq!(
            scene
                .actors
                .iter()
                .filter(|e| e.editable_mesh.is_some())
                .count(),
            7
        );
        let triangles: usize = scene
            .actors
            .iter()
            .map(|e| lighting::quad_count(e) * 2)
            .sum();
        assert!(
            triangles < 7000,
            "Template exceeds the PSX geometry limit: {triangles}"
        );
        for e in &scene.actors {
            if let Some(mesh) = &e.editable_mesh {
                assert!(mesh.error.is_none());
                mesh.document.as_ref().unwrap().validate().unwrap();
                crate::mesh_compile::chunks(&lighting::quads(e)).unwrap();
            }
        }
        // The template now ships a controller, so the header needs the real
        // catalog to resolve the Player's script binding.
        let catalog = crate::scripts::catalog(&root).unwrap();
        let header = crate::project::scene_header(&scene, &catalog).unwrap();
        assert!(header.contains("MeshGeometry"));
        assert!(root.join("UserSettings/SceneView.epokprefs").is_file());
        let editor = crate::editor::Editor::open(project).unwrap();
        assert_eq!(editor.view.center, overview().center);
        assert!(editor.scene.actors.iter().any(|e| e.name == "Player"));
        // Opening a project creates the default map Blueprint only when host
        // reflection resolves its SDK parent. The editor deliberately leaves a map
        // unchanged when those optional host tools are unavailable.
        if let Some(scene_script) = &editor.scene.scene_script {
            assert_eq!(
                scene_script.parent.class_id.as_deref(),
                Some(crate::object_model::SCENE_SCRIPT_ACTOR_ID)
            );
            crate::project::stage(&root, &editor.scene).unwrap();
            let cooked = std::fs::read_to_string(root.join(".epok/build/scene.hh")).unwrap();
            let scene_script =
                crate::blueprint_refs::compact_id(crate::object_model::SCENE_SCRIPT_ACTOR_ID);
            assert!(cooked.contains(&format!(
                "inline constexpr uint64_t scene_script_class=UINT64_C({scene_script});"
            )));
            assert!(cooked.contains("ObjectPool<epok::SceneScriptActor,4>::storage_bytes"));
        }
        println!(
            "Third Person: {} actors, {triangles} compiled triangle slots",
            scene.actors.len()
        );
        drop(editor);
        std::fs::remove_dir_all(parent).unwrap();
    }
}
