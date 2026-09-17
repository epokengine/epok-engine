use crate::{
    assets::{self, Kind},
    model_import::{self, Settings},
    skeletal::{self, Data, Model},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
};
use uuid::Uuid;
const FBX: &[u8] = include_bytes!("../resources/models/EpokMannequin.fbx");
fn decoded() -> Model {
    let mut settings = Settings::default();
    let imported = model_import::decode(FBX, &mut settings).unwrap();
    let skeleton = imported
        .outputs
        .iter()
        .find_map(|(_, d)| {
            if let Data::Skeleton(v) = d {
                Some(v.clone())
            } else {
                None
            }
        })
        .unwrap();
    let mesh = imported
        .outputs
        .iter()
        .find_map(|(_, d)| {
            if let Data::SkeletalMesh(v) = d {
                Some(v.clone())
            } else {
                None
            }
        })
        .unwrap();
    let clips = imported
        .outputs
        .iter()
        .filter_map(|(key, d)| {
            if let Data::AnimationClip(v) = d {
                Some((settings.outputs[key].id, v.clone()))
            } else {
                None
            }
        })
        .collect();
    Model {
        mesh,
        skeleton,
        clips,
        materials: vec![],
    }
}

#[test]
fn frame_selected_uses_animated_skeletal_bounds_and_parent_transform() {
    let model = std::sync::Arc::new(decoded());
    let mut editor = crate::editor::Editor::new(
        std::env::temp_dir().join(format!("epok-frame-skin-{}", Uuid::new_v4())),
    );
    let mut parent = crate::scene::Actor::cube("Parent".into());
    parent.kind = "Empty".into();
    parent.position = [4., 2., -3.];
    parent.rotation = [15., 35., -10.];
    parent.scale = [1.3, 0.7, 1.1];
    let mut actor = crate::scene::Actor::cube("Character".into());
    actor.parent = Some(0);
    actor.attach = Some(crate::actor_document::Attachment {
        actor: parent.id,
        component: None,
    });
    actor.position = [2., 1., 0.];
    actor.rotation = [-5., 60., 12.];
    actor.scale = [0.8, 1.7, 0.6];
    let mut component = skeletal::Component::new(Uuid::new_v4());
    component.model = Some(model.clone());
    actor.skeletal_mesh = Some(component);
    editor.scene.actors = vec![parent, actor];
    editor.selected = Some(1);
    for clip in [None, Some(model.clips[0].0)] {
        for time in [0., 0.25, 0.75] {
            let component = editor.scene.actors[1].skeletal_mesh.as_mut().unwrap();
            component.clip = clip;
            component.time = time;
            let world = editor.scene.world_matrix(1);
            let points: Vec<_> = model
                .points(clip, time, true)
                .into_iter()
                .map(|p| world.point(p))
                .collect();
            let expected: [f32; 3] = std::array::from_fn(|axis| {
                (points.iter().map(|p| p[axis]).fold(f32::INFINITY, f32::min)
                    + points
                        .iter()
                        .map(|p| p[axis])
                        .fold(f32::NEG_INFINITY, f32::max))
                    * 0.5
            });
            let scene = editor.scene.clone();
            for zoom in [0.3, 0.85, 3.5] {
                editor.view.zoom = zoom;
                editor.view.center = [-10., -10., -10.];
                editor.view.yaw = 0.6;
                editor.view.pitch = 0.3;
                editor.action("frame-selected");
                assert_eq!(editor.view.center, expected);
                assert_eq!(editor.view.zoom, zoom);
                assert_eq!(editor.view.yaw, 0.6);
                assert_eq!(editor.view.pitch, 0.3);
                for &point in &points {
                    let pixel = crate::viewport::project(&editor.view, point);
                    assert!(pixel[2] > 1.);
                    assert!((0. ..960.).contains(&pixel[0]), "{pixel:?}");
                    assert!((0. ..600.).contains(&pixel[1]), "{pixel:?}");
                }
                assert_eq!(
                    editor.scene, scene,
                    "Framing must not edit the actor or pose"
                );
            }
        }
    }
}

#[test]
fn fbx_quantized_skinning_matches_independent_ufbx_evaluation() {
    let model = decoded();
    assert_eq!(model.mesh.vertices.len(), 96);
    assert_eq!(model.clips.len(), 2);
    let bind = model.points(None, 0., false);
    let center: [f32; 3] =
        std::array::from_fn(|c| bind[..8].iter().map(|p| p[c]).sum::<f32>() / 8.);
    for triangle in &model.mesh.triangles[..12] {
        let [a, b, c] = triangle.indices.map(|i| bind[i as usize]);
        let normal = crate::mesh::face_normal([a, b, c, c]);
        assert!(
            crate::lighting::dot(normal, crate::lighting::sub(a, center)) > 0.,
            "FBX triangles must face outward after handedness conversion"
        );
    }
    let source = ufbx::load_memory(
        FBX,
        ufbx::LoadOpts {
            target_axes: ufbx::axes_left_handed_y_up(),
            target_unit_meters: 1.,
            space_conversion: ufbx::SpaceConversion::TransformRoot,
            inherit_mode_handling: ufbx::InheritModeHandling::HelperNodes,
            ..Default::default()
        },
    )
    .unwrap();
    for (id, clip) in &model.clips {
        let stack = source
            .anim_stacks
            .iter()
            .find(|s| s.element.name.as_ref() == clip.name)
            .unwrap();
        for time in [0., 0.25, 0.5, 1., 1.5, 2.] {
            let time = (time * 30_f32).floor() / 30.;
            let evaluated = ufbx::evaluate_scene(
                &source,
                &stack.anim,
                stack.time_begin + time as f64,
                ufbx::EvaluateOpts {
                    evaluate_skinning: true,
                    ..Default::default()
                },
            )
            .unwrap();
            let mesh = &evaluated.meshes[0];
            let points = model.points(Some(*id), time + 0.000001, false);
            for (i, p) in points.iter().enumerate() {
                let mut reference = ufbx::get_vertex_vec3(
                    &mesh.skinned_position,
                    mesh.vertex_first_index[i] as usize,
                );
                if mesh.skinned_is_local {
                    reference = ufbx::transform_position(
                        &mesh.element.instances[0].geometry_to_world,
                        reference,
                    );
                }
                for (axis, expected) in [reference.x, reference.y, reference.z].iter().enumerate() {
                    assert!(
                        (p[axis] as f64 - expected).abs() < 0.012,
                        "{} @ {time}, vertex {i}: {p:?} != {reference:?}",
                        clip.name
                    );
                }
            }
            let height = points
                .iter()
                .map(|p| p[1])
                .fold(f32::NEG_INFINITY, f32::max);
            assert!(
                (1.7..2.).contains(&height),
                "Unexpected character height {height}; source axes {:?}, units {}; first {:?}",
                source.settings.axes,
                source.settings.unit_meters,
                points[0]
            );
        }
    }
    let walk = model
        .clips
        .iter()
        .find(|(_, c)| c.name.contains("Walk"))
        .unwrap()
        .0;
    assert_ne!(
        model.points(Some(walk), 0., false),
        model.points(Some(walk), 0.5, false)
    );
    assert_eq!(
        model.points(Some(walk), 0., true),
        model.points(Some(walk), 2., true)
    );
}
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!("epok-skeletal-{}", Uuid::new_v4()));
        fs::create_dir_all(p.join("assets")).unwrap();
        fs::write(p.join("assets/Hero.fbx"), FBX).unwrap();
        Self(p)
    }
    fn index(&self) -> assets::Index {
        assets::scan(&self.0, &mut Default::default())
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        assert!(
            self.0.starts_with(std::env::temp_dir())
                && self
                    .0
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("epok-skeletal-")
        );
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn model_reimport_is_portable_preserves_ids_materials_and_rejects_stale_edits() {
    let f = Fixture::new();
    let dest = model_import::destination("assets/Hero.fbx");
    let id = model_import::commit(
        model_import::prepare(&f.0, "assets/Hero.fbx", &dest, None, false, None).unwrap(),
    )
    .unwrap();
    let before = f.index();
    assert!(before.problems.is_empty(), "{:?}", before.problems);
    let source = before.resolve(id).unwrap();
    let material = before
        .usable()
        .find(|r| r.meta.kind == Kind::Material)
        .unwrap();
    let mut package = assets::Package::load(&material.path).unwrap();
    package.source = serde_json::to_vec(&Data::Material(crate::scene::Material {
        color: [1., 0., 0.],
        unlit: true,
        ..Default::default()
    }))
    .unwrap();
    package.meta.source_hash = assets::hash(&package.source);
    assets::atomic_write(
        &material.path,
        &package.bytes().unwrap(),
        Some(&material.revision),
    )
    .unwrap();
    let material_bytes = fs::read(&material.path).unwrap();
    let prepared =
        model_import::prepare(&f.0, "assets/Hero.fbx", &dest, Some(source), false, None).unwrap();
    // A stale edit to any subasset aborts the entire replacement.
    fs::write(&material.path, b"changed during import").unwrap();
    assert!(model_import::commit(prepared).is_err());
    assert_eq!(
        assets::hash(&fs::read(&source.path).unwrap()),
        source.revision
    );
    fs::write(&material.path, &material_bytes).unwrap();
    fs::remove_file(f.0.join("assets/Hero.fbx")).unwrap();
    model_import::commit(
        model_import::prepare(&f.0, "assets/Hero.fbx", &dest, Some(source), true, None).unwrap(),
    )
    .unwrap();
    let after = f.index();
    assert_eq!(
        before.assets.keys().collect::<Vec<_>>(),
        after.assets.keys().collect::<Vec<_>>()
    );
    assert_eq!(fs::read(&material.path).unwrap(), material_bytes);
    let mesh = after
        .usable()
        .find(|r| r.meta.kind == Kind::SkeletalMesh)
        .unwrap();
    let model = Model::load(&after, mesh.meta.id).unwrap();
    assert!(
        !assets::dependencies(&f.0, model.mesh.skeleton)
            .unwrap()
            .is_empty()
    );
    assert!(assets::trash(&f.0, after.resolve(model.mesh.skeleton).unwrap()).is_err());
    let mut entity = crate::scene::Actor::cube("Hero".into());
    entity.skeletal_mesh = Some(skeletal::Component::new(mesh.meta.id));
    let mut scene = crate::scene::Scene::default();
    scene.actors.push(entity);
    skeletal::resolve(&mut scene, &after).unwrap();
    scene.actors.push(scene.actors.last().unwrap().clone());
    let actor = scene.actors.last_mut().unwrap();
    let ids = crate::actor_document::fresh_identities(std::slice::from_ref(actor));
    crate::actor_document::remap_actor(actor, &ids);
    let header = crate::project::scene_header(&scene, &[]).unwrap();
    assert!(header.contains("inline constexpr SkeletalMesh"));
    assert!(header.contains("skin_bone_vertices_"));
    assert!(header.contains("SkeletalStorage::RigidGte"));
    assert_eq!(
        header.matches("inline constexpr SkeletalMesh").count(),
        1,
        "Characters must share compiled model data"
    );
    assert!(
        !header
            .split("inline void initialize_components()")
            .nth(1)
            .unwrap()
            .contains("inline constexpr SkeletalMesh")
    );
    assert!(
        !header.contains(&mesh.meta.id.to_string()),
        "UUID leaked to target tables"
    );
    std::sync::Arc::make_mut(
        scene
            .actors
            .last_mut()
            .unwrap()
            .skeletal_mesh
            .as_mut()
            .unwrap()
            .model
            .as_mut()
            .unwrap(),
    )
    .mesh
    .animation_storage = skeletal::AnimationStorage::BakedVertices;
    let baked = crate::skeletal_compile::header(scene.actors.last().unwrap(), 99).unwrap();
    assert!(baked.contains("skin_vertex_frames_99_"));
    assert!(baked.contains("skin_vertex_data_99_"));
    assert!(baked.contains("SkeletalStorage::BakedVertices"));
    assert!(!baked.contains("skin_pose_99_"));
    // Runtime state is not persisted into scenes.
    let serialized = serde_json::to_vec(&scene).unwrap();
    scene
        .actors
        .last_mut()
        .unwrap()
        .skeletal_mesh
        .as_mut()
        .unwrap()
        .time = 1.;
    assert_eq!(serialized, serde_json::to_vec(&scene).unwrap());
}
#[test]
fn malformed_and_incompatible_skeletal_assets_fail_before_publication() {
    assert!(model_import::decode(b"not an fbx", &mut Default::default()).is_err());
    let m = decoded();
    let mut legacy = serde_json::to_value(&m.mesh).unwrap();
    legacy.as_object_mut().unwrap().remove("animation_storage");
    let legacy: skeletal::Mesh = serde_json::from_value(legacy).unwrap();
    assert_eq!(
        legacy.animation_storage,
        skeletal::AnimationStorage::RigidGte
    );
    let mut bad = m.skeleton.clone();
    bad.bones[0].parent = 0;
    assert!(Data::parse(&serde_json::to_vec(&Data::Skeleton(bad)).unwrap()).is_err());
    let mut bad = m.mesh.clone();
    bad.triangles[0].indices[0] = 65535;
    assert!(Data::parse(&serde_json::to_vec(&Data::SkeletalMesh(bad)).unwrap()).is_err());
    let mut c = m.clips[0].1.clone();
    c.tracks[0].clear();
    assert!(Data::parse(&serde_json::to_vec(&Data::AnimationClip(c)).unwrap()).is_err());
}

#[test]
fn fbx_import_inbox_worker_publishes_subassets_and_opens_a_usable_model() {
    use std::time::{Duration, Instant};
    let f = Fixture::new();
    let mut manager = crate::asset_manager::Manager::new(f.0.clone());
    let deadline = Instant::now() + Duration::from_secs(8);
    while manager.pending.is_empty() {
        manager.tick();
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(10));
    }
    let item = manager.pending[0].clone();
    manager.begin_pending(&item);
    assert!(manager.form.as_ref().unwrap().model);
    manager.start_import();
    while manager.busy || manager.selected.is_none() {
        manager.tick();
        assert!(Instant::now() < deadline, "{:?}", manager.error);
        std::thread::sleep(Duration::from_millis(10));
    }
    let index = f.index();
    assert_eq!(
        index.resolve(manager.selected.unwrap()).unwrap().meta.kind,
        Kind::ModelSource
    );
    let mesh = index
        .usable()
        .find(|r| r.meta.kind == Kind::SkeletalMesh)
        .unwrap();
    let mut scene = crate::scene::Scene::default();
    let mut entity = crate::scene::Actor::cube("Missing clip".into());
    let mut c = skeletal::Component::new(mesh.meta.id);
    c.clip = Some(Uuid::new_v4());
    entity.skeletal_mesh = Some(c);
    scene.actors.push(entity);
    assert!(skeletal::resolve(&mut scene, &index).is_err());
    assert!(
        scene
            .actors
            .last()
            .unwrap()
            .skeletal_mesh
            .as_ref()
            .unwrap()
            .model
            .is_some(),
        "Inspector must retain the model to offer replacement clips"
    );
}

fn identity_pose() -> skeletal::Pose {
    skeletal::Pose {
        translation: [0; 3],
        rotation: [0, 0, 0, 4096],
        scale: [4096; 3],
    }
}
/// Two triangles that share positions but carry different corner coordinates,
/// the case a texture seam produces. Positions must never be duplicated for it.
fn seam_model(uv: bool, texture: Option<Uuid>) -> Model {
    let corners: [[f32; 2]; 3] = [[0., 0.], [0.5, 0.], [0.5, 0.5]];
    let second: [[f32; 2]; 3] = [[1., 0.5], [0.5, 1.], [1., 1.]];
    Model {
        mesh: skeletal::Mesh {
            skeleton: Uuid::from_u128(1),
            vertices: [
                [-2048, -2048, 0],
                [2048, -2048, 0],
                [2048, 2048, 0],
                [-2048, 2048, 0],
            ]
            .into_iter()
            .map(|position| skeletal::Vertex { position, bone: 0 })
            .collect(),
            triangles: vec![
                skeletal::Triangle {
                    indices: [0, 1, 2],
                    material: 0,
                    uv: uv.then_some(corners),
                },
                skeletal::Triangle {
                    indices: [2, 1, 3],
                    material: 0,
                    uv: uv.then_some(second),
                },
            ],
            materials: vec![Uuid::from_u128(2)],
            clips: vec![],
            animation_storage: skeletal::AnimationStorage::RigidGte,
        },
        skeleton: skeletal::Skeleton {
            bones: vec![skeletal::Bone {
                name: "root".into(),
                parent: -1,
                bind: identity_pose(),
            }],
        },
        clips: vec![],
        materials: vec![crate::scene::Material {
            unlit: true,
            texture,
            ..Default::default()
        }],
    }
}
fn seam_actor(model: Model) -> crate::scene::Actor {
    let mut actor = crate::scene::Actor::cube("Seam".into());
    let mut component = skeletal::Component::new(Uuid::from_u128(3));
    component.model = Some(std::sync::Arc::new(model));
    actor.skeletal_mesh = Some(component);
    actor
}

fn animated_seam_model(storage: skeletal::AnimationStorage) -> Model {
    let mut model = seam_model(false, None);
    let id = Uuid::from_u128(4);
    let mut moved = identity_pose();
    moved.translation[0] = 128;
    model.mesh.animation_storage = storage;
    model.mesh.clips.push(id);
    model.clips.push((
        id,
        skeletal::Clip {
            skeleton: model.mesh.skeleton,
            name: "Move".into(),
            fps: 30,
            frames: 2,
            tracks: vec![vec![identity_pose(), moved]],
        },
    ));
    model
}

#[test]
fn optional_skeletal_query_tables_follow_the_operation_demand() {
    use crate::skeletal_compile::{QueryDemand, header_with_pages_for};

    let render_only = header_with_pages_for(
        &seam_actor(animated_seam_model(
            skeletal::AnimationStorage::BakedVertices,
        )),
        7,
        &|_| None,
        QueryDemand::NONE,
    )
    .unwrap();
    assert!(render_only.contains("skin_vertex_frames_7_0"));
    assert!(render_only.contains("skin_vertex_data_7_0"));
    assert!(!render_only.contains("skin_vertex_seek_7_"));
    assert!(!render_only.contains("skin_bones_7"));
    assert!(!render_only.contains("skin_tracks_7_"));
    assert!(render_only.contains("nullptr,0,skin_clips_7,1,SkeletalStorage::BakedVertices"));

    let vertices = header_with_pages_for(
        &seam_actor(animated_seam_model(
            skeletal::AnimationStorage::BakedVertices,
        )),
        8,
        &|_| None,
        QueryDemand {
            vertices: true,
            bones: false,
        },
    )
    .unwrap();
    assert!(vertices.contains("skin_vertex_seek_8_"));
    assert!(!vertices.contains("skin_bones_8"));
    assert!(!vertices.contains("skin_tracks_8_"));

    let bones = header_with_pages_for(
        &seam_actor(animated_seam_model(
            skeletal::AnimationStorage::BakedVertices,
        )),
        9,
        &|_| None,
        QueryDemand {
            vertices: false,
            bones: true,
        },
    )
    .unwrap();
    assert!(!bones.contains("skin_vertex_seek_9_"));
    assert!(bones.contains("skin_bones_9"));
    assert!(bones.contains("skin_tracks_9_0"));

    let rigid = header_with_pages_for(
        &seam_actor(animated_seam_model(skeletal::AnimationStorage::RigidGte)),
        10,
        &|_| None,
        QueryDemand::NONE,
    )
    .unwrap();
    assert!(!rigid.contains("skin_portable_to_cooked_10"));
    assert!(rigid.contains("skin_bones_10"));
    assert!(rigid.contains("skin_tracks_10_0"));
}

#[test]
fn skeletal_picking_follows_pose_surfaces_activity_and_inherited_transforms() {
    let mut model = seam_model(false, None);
    model.mesh.triangles[0].indices = [0, 2, 1];
    model.mesh.triangles[1].indices = [0, 3, 2];
    let clip = Uuid::new_v4();
    let mut pose = identity_pose();
    pose.translation[1] = 512;
    let mut later = pose.clone();
    later.translation[0] = 768;
    model.clips.push((
        clip,
        skeletal::Clip {
            skeleton: model.mesh.skeleton,
            name: "Move".into(),
            fps: 30,
            frames: 2,
            tracks: vec![vec![pose, later]],
        },
    ));
    model.mesh.clips = vec![clip];
    let mut parent = crate::scene::Actor::cube("Parent".into());
    parent.kind = "Empty".into();
    parent.position = [2., 0., 1.];
    parent.rotation = [0., 25., 0.];
    parent.scale = [1.5, 0.8, 1.];
    let mut actor = seam_actor(model);
    actor.parent = Some(0);
    actor.position = [0.; 3];
    actor.attach = Some(crate::actor_document::Attachment {
        actor: parent.id,
        component: None,
    });
    let c = actor.skeletal_mesh.as_mut().unwrap();
    c.clip = Some(clip);
    c.looping = false;
    let mut scene = crate::scene::Scene {
        actors: vec![parent, actor],
        ..Default::default()
    };
    let view = crate::viewport::View {
        yaw: 0.,
        pitch: 0.,
        ..Default::default()
    };
    let pixel = |point| {
        let p = crate::viewport::project(&view, point);
        [p[0], p[1]]
    };
    let world = scene.world_matrix(1);
    let first = pixel(world.point([0., 2., 0.]));
    let later = pixel(world.point([3., 2., 0.]));
    assert_eq!(crate::picking::pick(&scene, &view, first), Some(1));
    assert_eq!(
        crate::picking::pick(&scene, &view, pixel(world.point([0.; 3]))),
        None,
        "No invisible pivot cube"
    );
    scene.actors[1].skeletal_mesh.as_mut().unwrap().time = 1. / 30.;
    assert_eq!(crate::picking::pick(&scene, &view, first), None);
    assert_eq!(crate::picking::pick(&scene, &view, later), Some(1));
    scene.actors[0].active = false;
    assert_eq!(crate::picking::pick(&scene, &view, later), None);
    scene.actors[0].active = true;
    scene.actors[1].skeletal_mesh.as_mut().unwrap().model = None;
    assert_eq!(crate::picking::pick(&scene, &view, later), None);
    assert_eq!(
        crate::picking::pick(&scene, &view, pixel(world.point([0.; 3]))),
        None
    );
}

#[test]
fn frame_selected_includes_skeletal_child_when_controller_root_is_selected() {
    let mut model = seam_model(false, None);
    model.skeleton.bones[0].bind.translation[1] = 512;
    let mut root = crate::scene::Actor::cube("Controller".into());
    root.kind = "Empty".into();
    root.position = [4., 1., -2.];
    let mut actor = seam_actor(model);
    actor.parent = Some(0);
    actor.position = [0.; 3];
    actor.attach = Some(crate::actor_document::Attachment {
        actor: root.id,
        component: None,
    });
    let mut editor = crate::editor::Editor::new(
        std::env::temp_dir().join(format!("epok-frame-root-{}", Uuid::new_v4())),
    );
    editor.scene.actors = vec![root, actor];
    editor.selected = Some(0);
    editor.scene_panel_size = [240., 900.];
    editor.action("frame-selected");
    assert_eq!(editor.view.center, [4., 3., -2.]);
}

#[test]
fn triangle_texture_coordinates_round_trip_and_reject_invalid_values() {
    let mut mesh = seam_model(true, None).mesh;
    let encoded = serde_json::to_value(&mesh).unwrap();
    // Unmapped triangles must stay absent from the payload so older assets and
    // newly written ones remain byte-comparable where nothing is mapped.
    let plain = serde_json::to_value(&seam_model(false, None).mesh).unwrap();
    assert!(
        !plain["triangles"][0]
            .as_object()
            .unwrap()
            .contains_key("uv")
    );
    let decoded: skeletal::Mesh = serde_json::from_value(encoded).unwrap();
    assert_eq!(decoded, mesh);
    assert_eq!(
        decoded.triangles[0].uv.unwrap(),
        [[0., 0.], [0.5, 0.], [0.5, 0.5]]
    );
    let legacy: skeletal::Mesh = serde_json::from_value(plain).unwrap();
    assert!(legacy.triangles.iter().all(|t| t.uv.is_none()));
    // The origin of the atlas is a real coordinate, not "unmapped".
    let mut zeroed = mesh.clone();
    zeroed.triangles[0].uv = Some([[0.; 2]; 3]);
    assert_ne!(zeroed.triangles[0].uv, legacy.triangles[0].uv);
    assert_ne!(
        serde_json::to_value(&zeroed).unwrap(),
        serde_json::to_value(&legacy).unwrap()
    );
    for bad in [f32::NAN, -0.01, 1.01, f32::INFINITY] {
        let mut broken = mesh.clone();
        broken.triangles[1].uv.as_mut().unwrap()[2][1] = bad;
        assert!(
            Data::parse(&serde_json::to_vec(&Data::SkeletalMesh(broken)).unwrap()).is_err(),
            "{bad} must be rejected"
        );
    }
    mesh.triangles[0].uv.as_mut().unwrap()[1] = [1., 1.];
    assert!(Data::parse(&serde_json::to_vec(&Data::SkeletalMesh(mesh)).unwrap()).is_ok());
}

#[test]
fn textured_material_without_coordinates_fails_when_the_model_resolves() {
    let f = Fixture::new();
    let dest = model_import::destination("assets/Hero.fbx");
    model_import::commit(
        model_import::prepare(&f.0, "assets/Hero.fbx", &dest, None, false, None).unwrap(),
    )
    .unwrap();
    let index = f.index();
    let mesh = index
        .usable()
        .find(|r| r.meta.kind == Kind::SkeletalMesh)
        .unwrap()
        .clone();
    let model = Model::load(&index, mesh.meta.id).unwrap();
    assert!(
        model.mesh.triangles.iter().all(|t| t.uv.is_none()),
        "the sample character carries no UV set"
    );
    let slots = model
        .mesh
        .materials
        .iter()
        .map(|id| index.resolve(*id).unwrap().clone())
        .collect::<Vec<_>>();
    let texture = Uuid::from_u128(77);
    for slot in &slots {
        crate::skeletal_ui::edit_material(&slot.path, &slot.revision, |m| {
            m.texture = Some(texture)
        })
        .unwrap();
    }
    let index = f.index();
    let error = Model::load(&index, mesh.meta.id).unwrap_err();
    assert!(error.contains("texture coordinates"), "{error}");
    // Untextured slots may legitimately stay unmapped.
    for slot in &slots {
        let slot = index.resolve(slot.meta.id).unwrap().clone();
        crate::skeletal_ui::edit_material(&slot.path, &slot.revision, |m| m.texture = None)
            .unwrap();
    }
    Model::load(&f.index(), mesh.meta.id).unwrap();
}

#[test]
fn material_edits_preserve_every_field_the_model_window_does_not_show() {
    let f = Fixture::new();
    let dest = model_import::destination("assets/Hero.fbx");
    model_import::commit(
        model_import::prepare(&f.0, "assets/Hero.fbx", &dest, None, false, None).unwrap(),
    )
    .unwrap();
    let index = f.index();
    let slot = index
        .usable()
        .find(|r| r.meta.kind == Kind::Material)
        .unwrap()
        .clone();
    let texture = Uuid::from_u128(9);
    let (_, revision, written) =
        crate::skeletal_ui::edit_material(&slot.path, &slot.revision, |m| {
            m.texture = Some(texture);
            m.blend = crate::texture::BlendMode::Add;
            m.depth_bias = -37;
            m.uv_scroll = [0.25, -0.5];
            m.unlit = false;
        })
        .unwrap();
    assert_eq!(written.texture, Some(texture));
    let (_, _, edited) =
        crate::skeletal_ui::edit_material(&slot.path, &revision, |m| m.color = [1., 0., 0.])
            .unwrap();
    assert_eq!(edited.color, [1., 0., 0.]);
    assert_eq!(edited.texture, Some(texture));
    assert_eq!(edited.blend, crate::texture::BlendMode::Add);
    assert_eq!(edited.depth_bias, -37);
    assert_eq!(edited.uv_scroll, [0.25, -0.5]);
    assert!(!edited.unlit);
    // Stale revisions are still refused.
    assert!(
        crate::skeletal_ui::edit_material(&slot.path, &slot.revision, |m| m.color = [0.; 3])
            .is_err()
    );
}

#[test]
fn seam_coordinates_survive_rigid_reordering_without_duplicating_positions() {
    let plain = crate::skeletal_compile::header(&seam_actor(seam_model(false, None)), 7).unwrap();
    let mapped = crate::skeletal_compile::header(&seam_actor(seam_model(true, None)), 7).unwrap();
    let positions = |text: &str| {
        text.split("skin_vertices_7[")
            .nth(1)
            .unwrap()
            .split(']')
            .next()
            .unwrap()
            .to_string()
    };
    assert_eq!(positions(&plain), positions(&mapped));
    assert_eq!(positions(&mapped), "4");
    assert!(plain.contains("{{0,0},{4096,0},{4096,4096},{4096,4096}}"));
    // Both faces keep their own corners although they share two positions.
    assert!(mapped.contains("{{0,0},{2048,0},{2048,2048},{2048,2048}}"));
    assert!(mapped.contains("{{4096,2048},{2048,4096},{4096,4096},{4096,4096}}"));
    let faces = mapped
        .split("skin_faces_7[]={")
        .nth(1)
        .unwrap()
        .split("};")
        .next()
        .unwrap();
    assert!(faces.contains("{{0,1,2,2}"), "{faces}");
    assert!(faces.contains("{{2,1,3,3}"), "{faces}");
    assert!(mapped.contains("SkeletalStorage::RigidGte"));
}

#[test]
fn page_coordinates_match_the_shared_mapping_in_both_storage_modes() {
    let page = (64_u16, 32_u16, 256_u16);
    let texture = Uuid::from_u128(5);
    let lookup = |id: Uuid| (id == texture).then_some(page);
    let expected = [[0., 0.], [0.5, 0.], [0.5, 0.5], [0.5, 0.5]]
        .map(|uv| crate::mesh_compile::packed_uv(uv, page).to_string())
        .join(",");
    for storage in [
        skeletal::AnimationStorage::RigidGte,
        skeletal::AnimationStorage::BakedVertices,
    ] {
        let mut model = seam_model(true, Some(texture));
        model.mesh.animation_storage = storage;
        let actor = seam_actor(model);
        let header = crate::skeletal_compile::header_with_pages(&actor, 7, &lookup).unwrap();
        assert!(header.contains(&format!("true,{{{expected}}}")), "{header}");
        assert!(header.contains("{{0,0},{2048,0},{2048,2048},{2048,2048}}"));
        assert!(header.contains("{{4096,2048},{2048,4096},{4096,4096},{4096,4096}}"));
        // Without a resident page the quad falls back to scaled Q12 corners.
        let plain = crate::skeletal_compile::header(&actor, 7).unwrap();
        let faces = plain
            .split("skin_faces_7[]={")
            .nth(1)
            .unwrap()
            .split("};")
            .next()
            .unwrap();
        assert_eq!(faces.matches("false,{0,0,0,0}").count(), 2);
        assert!(!faces.contains("true,{"), "{faces}");
    }
    // A textured but fully unlit model still takes the GTE path.
    let header = crate::skeletal_compile::header_with_pages(
        &seam_actor(seam_model(true, Some(texture))),
        7,
        &lookup,
    )
    .unwrap();
    assert!(header.contains("SkeletalStorage::RigidGte"));
}

#[test]
fn fbx_import_reads_corner_coordinates_and_flips_the_vertical_axis() {
    const SEAM: &[u8] = include_bytes!("../resources/models/EpokSeamCharacter.fbx");
    let imported = model_import::decode(SEAM, &mut Settings::default()).unwrap();
    let mesh = imported
        .outputs
        .iter()
        .find_map(|(_, d)| match d {
            Data::SkeletalMesh(v) => Some(v.clone()),
            _ => None,
        })
        .unwrap();
    assert!(mesh.triangles.iter().all(|t| t.uv.is_some()));
    assert!(
        mesh.triangles
            .iter()
            .flat_map(|t| t.uv.unwrap())
            .flatten()
            .all(|v| (0. ..=1.).contains(&v))
    );
    // One position carrying two different corner coordinates is the seam case;
    // it must not have been resolved by duplicating the position.
    let mut per_vertex = BTreeMap::<u16, BTreeSet<[u32; 2]>>::new();
    for t in &mesh.triangles {
        for (index, uv) in t.indices.iter().zip(t.uv.unwrap()) {
            per_vertex
                .entry(*index)
                .or_default()
                .insert(uv.map(f32::to_bits));
        }
    }
    assert!(
        per_vertex.values().any(|set| set.len() > 1),
        "the fixture is built around a texture seam"
    );
    // Cross-check the vertical convention against the file itself: the source
    // stores v from the bottom edge, the mesh stores it from the top row.
    let source = ufbx::load_memory(
        SEAM,
        ufbx::LoadOpts {
            target_axes: ufbx::axes_left_handed_y_up(),
            target_unit_meters: 1.,
            space_conversion: ufbx::SpaceConversion::TransformRoot,
            inherit_mode_handling: ufbx::InheritModeHandling::HelperNodes,
            ..Default::default()
        },
    )
    .unwrap();
    let source_mesh = &source.meshes[0];
    assert!(source_mesh.vertex_uv.exists);
    let stored = mesh
        .triangles
        .iter()
        .flat_map(|t| t.uv.unwrap())
        .map(|uv| uv.map(|v| (v * 4096.).round() as i32))
        .collect::<BTreeSet<_>>();
    let expected = (0..source_mesh.vertex_uv.indices.len())
        .map(|corner| {
            let uv = ufbx::get_vertex_vec2(&source_mesh.vertex_uv, corner);
            [uv.x as f32, 1. - uv.y as f32].map(|v| (v * 4096.).round() as i32)
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(stored, expected);
    assert!(
        expected
            .iter()
            .any(|uv| uv[1] != 4096 - uv[1] && !stored.contains(&[uv[0], 4096 - uv[1]])),
        "the fixture must contain at least one asymmetric coordinate so the flip is observable"
    );
}

const MANY_SEQUENCES: &[u8] = include_bytes!("../resources/models/EpokManySequences.fbx");

/// Resolve an import the way `Model::load` does: clips and materials follow the
/// mesh's own reference order, which is the order the generated tables and the
/// runtime clip indices use.
fn decoded_model(bytes: &[u8]) -> Model {
    let mut settings = Settings::default();
    let imported = model_import::decode(bytes, &mut settings).unwrap();
    let pick = |wanted: Uuid| {
        imported
            .outputs
            .iter()
            .find(|(key, _)| settings.outputs[key].id == wanted)
            .map(|(_, d)| d.clone())
            .unwrap()
    };
    let skeleton = imported
        .outputs
        .iter()
        .find_map(|(_, d)| match d {
            Data::Skeleton(v) => Some(v.clone()),
            _ => None,
        })
        .unwrap();
    let mesh = imported
        .outputs
        .iter()
        .find_map(|(_, d)| match d {
            Data::SkeletalMesh(v) => Some(v.clone()),
            _ => None,
        })
        .unwrap();
    let clips = mesh
        .clips
        .iter()
        .map(|id| match pick(*id) {
            Data::AnimationClip(c) => (*id, c),
            _ => unreachable!(),
        })
        .collect();
    let materials = mesh
        .materials
        .iter()
        .map(|id| match pick(*id) {
            Data::Material(m) => m,
            _ => unreachable!(),
        })
        .collect();
    Model {
        mesh,
        skeleton,
        clips,
        materials,
    }
}

fn mesh_with_clips(count: usize) -> skeletal::Mesh {
    let mut mesh = seam_model(false, None).mesh;
    mesh.clips = (0..count)
        .map(|i| Uuid::from_u128(100 + i as u128))
        .collect();
    mesh
}

#[test]
fn skeletal_mesh_payload_accepts_the_clip_bound_and_names_it_when_exceeded() {
    let parse = |count: usize| {
        skeletal::Data::parse(
            &serde_json::to_vec(&Data::SkeletalMesh(mesh_with_clips(count))).unwrap(),
        )
    };
    assert_eq!(skeletal::MAX_CLIPS, 32);
    parse(skeletal::MAX_CLIPS).unwrap();
    let error = parse(skeletal::MAX_CLIPS + 1).unwrap_err();
    assert!(
        error.contains(&format!("{} clips", skeletal::MAX_CLIPS)),
        "{error}"
    );
    // The retired bound must not survive anywhere in the message.
    assert!(!error.contains("16 clips"), "{error}");
}

#[test]
fn importer_reports_the_clip_bound_and_keeps_many_sequences_addressable() {
    // The generated bound and the importer's bound are the same constant.
    assert!(
        model_import::decode(MANY_SEQUENCES, &mut Settings::default())
            .unwrap()
            .outputs
            .iter()
            .filter(|(_, d)| matches!(d, Data::AnimationClip(_)))
            .count()
            > 16
    );
    let model = decoded_model(MANY_SEQUENCES);
    assert_eq!(model.clips.len(), 30);
    let names = model
        .clips
        .iter()
        .map(|(_, c)| c.name.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(names.len(), 30, "clip names must stay distinct identities");
    for (index, expected) in [
        (0_usize, "Clip00"),
        (15, "Clip15"),
        (16, "Clip16"),
        (28, "Clip28"),
        (29, "Clip29"),
    ] {
        let (id, clip) = &model.clips[index];
        assert_eq!(clip.name, expected);
        // The scene compiler turns a selected clip UUID into this index; the
        // round trip must agree for clips past the retired bound of 16.
        assert_eq!(
            model.clips.iter().position(|(key, _)| key == id),
            Some(index)
        );
    }
    // Clips beyond the old bound must be independently posable, not aliases.
    let sampled = [0_usize, 15, 16, 28, 29]
        .map(|index| {
            let (id, _) = &model.clips[index];
            model
                .points(Some(*id), 0.5, false)
                .iter()
                .flatten()
                .map(|v| (v * 4096.).round() as i32)
                .collect::<Vec<_>>()
        })
        .to_vec();
    for (a, first) in sampled.iter().enumerate() {
        for second in sampled.iter().skip(a + 1) {
            assert_ne!(first, second, "clips must pose the model differently");
        }
    }
}

#[test]
fn generated_tables_cover_every_clip_with_one_shared_geometry_table() {
    let model = decoded_model(MANY_SEQUENCES);
    let header = crate::skeletal_compile::header(&seam_actor(model), 7).unwrap();
    let clips = header
        .split("AnimationClip skin_clips_7[]={")
        .nth(1)
        .unwrap()
        .split("};")
        .next()
        .unwrap();
    assert_eq!(clips.matches("skin_tracks_7_").count(), 30);
    for name in ["Clip00", "Clip15", "Clip16", "Clip29"] {
        let octal = name
            .bytes()
            .map(|b| format!("\\{b:03o}"))
            .collect::<String>();
        assert!(clips.contains(&octal), "{name} missing from {clips}");
    }
    assert_eq!(header.matches("MeshGeometry skin_geometry_7=").count(), 1);
    assert_eq!(header.matches("int16_t skin_vertices_7[").count(), 1);
    assert_eq!(header.matches("SkeletalMesh skin_7=").count(), 1);
    assert!(header.contains(",30,SkeletalStorage::"));
    assert!(header.starts_with("// skeletal budget: "));
}

#[test]
fn model_budget_uses_the_target_layout_and_reports_a_breakdown_on_overflow() {
    use crate::skeletal_compile as compile;
    let model = decoded_model(MANY_SEQUENCES);
    let bones = model.skeleton.bones.len();
    let clips = model.clips.len();
    let poses = model
        .clips
        .iter()
        .map(|(_, c)| c.tracks.iter().map(Vec::len).sum::<usize>())
        .sum::<usize>();
    let budget = compile::budget(&model).unwrap();
    assert_eq!(budget.host_track_bytes, poses * compile::HOST_POSE_BYTES);
    assert_eq!(budget.rigid_pose_bytes, poses * compile::BONE_POSE_BYTES);
    assert_eq!(
        budget.rigid_descriptor_bytes,
        bones * compile::BONE_BYTES
            + clips * bones * compile::BONE_TRACK_BYTES
            + clips * compile::ANIMATION_CLIP_BYTES
    );
    assert_eq!(budget.baked_frame_bytes, 0);
    assert_eq!(budget.baked_descriptor_bytes, 0);
    assert_eq!(
        budget.geometry_bytes,
        model.mesh.vertices.len() * compile::VERTEX_BYTES
            + model.mesh.triangles.len() * compile::MESH_QUAD_BYTES
            + compile::MESH_GEOMETRY_BYTES
            + compile::SKELETAL_MESH_BYTES
            + model.mesh.vertices.len()
            + (bones + 1) * 2
    );
    assert_eq!(budget.animator_bytes, compile::ANIMATOR_BYTES);
    assert_eq!(
        budget.scratch_bytes,
        skeletal::MAX_BONES * compile::AFFINE_BYTES + skeletal::MAX_VERTICES * 6 + 72
    );
    assert!(budget.target_animation_bytes() <= skeletal::MAX_CLIP_BYTES);

    // Baked storage moves the same animation into frame payload and frame
    // descriptors, and the generated header states the breakdown it used.
    let mut baked = decoded_model(MANY_SEQUENCES);
    baked.mesh.animation_storage = skeletal::AnimationStorage::BakedVertices;
    let frames = baked
        .clips
        .iter()
        .map(|(_, c)| c.frames as usize)
        .sum::<usize>();
    let baked_budget = compile::budget(&baked).unwrap();
    assert_eq!(baked_budget.rigid_pose_bytes, 0);
    assert_eq!(baked_budget.rigid_descriptor_bytes, 0);
    assert!(baked_budget.baked_frame_bytes > 0);
    assert_eq!(
        baked_budget.baked_descriptor_bytes,
        frames * compile::VERTEX_FRAME_BYTES + clips * compile::ANIMATION_CLIP_BYTES
    );
    let header = crate::skeletal_compile::header(&seam_actor(baked), 7).unwrap();
    assert!(
        header.starts_with(&format!(
            "// skeletal budget: {}\n",
            baked_budget.breakdown()
        )),
        "{}",
        header.lines().next().unwrap()
    );

    // An oversized set of tracks fails with the same breakdown and the options.
    let mut oversized = decoded_model(MANY_SEQUENCES);
    let (_, clip) = &mut oversized.clips[0];
    clip.tracks[0] = vec![identity_pose(); 40_000];
    let oversized_bytes = oversized
        .clips
        .iter()
        .map(|(_, c)| c.tracks.iter().map(Vec::len).sum::<usize>())
        .sum::<usize>()
        * compile::HOST_POSE_BYTES;
    assert!(oversized_bytes > skeletal::MAX_CLIP_BYTES);
    let error = compile::budget(&oversized).unwrap_err();
    assert!(
        error.contains(&format!("host pose tracks {oversized_bytes} B")),
        "{error}"
    );
    assert!(error.contains("geometry "), "{error}");
    assert!(error.contains("shared scratch "), "{error}");
    assert!(
        error.contains(&format!("{} B", skeletal::MAX_CLIP_BYTES)),
        "{error}"
    );
    assert!(error.contains("Remove unused clips"), "{error}");
    assert!(
        error.contains("switch the animation storage mode"),
        "{error}"
    );
}
