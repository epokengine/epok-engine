use crate::{
    assets::{self, Kind},
    model_import::{self, Settings},
    skeletal::{self, Data, Model},
};
use std::{fs, path::PathBuf};
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
