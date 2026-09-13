//! Host-only FBX conversion and transactional publication of separate subassets.
use crate::{
    assets::{self, Kind, Metadata, Package, Record},
    import_settings,
    skeletal::{self, Bone, Clip, Data, Mesh, Pose, Skeleton, Triangle, Vertex},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Output {
    pub id: Uuid,
    pub file: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    pub version: u32,
    pub outputs: BTreeMap<String, Output>,
    pub warnings: Vec<String>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            version: 1,
            outputs: BTreeMap::new(),
            warnings: vec![],
        }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 || self.outputs.len() > 128 || self.warnings.len() > 64 {
            return Err("Unsupported FBX settings".into());
        }
        let mut ids = BTreeSet::new();
        let mut files = BTreeSet::new();
        for o in self.outputs.values() {
            if o.id.is_nil()
                || !ids.insert(o.id)
                || !files.insert(&o.file)
                || Path::new(&o.file).components().count() != 1
                || !o.file.ends_with(".epokasset")
                || o.file.contains(['/', '\\', ':'])
                || o.file == "Model.epokasset"
            {
                return Err("Invalid model output manifest".into());
            }
        }
        Ok(())
    }
}
fn q(value: f64, scale: f64, label: &str) -> Result<i16, String> {
    let value = (value * scale).round();
    if !value.is_finite() || value < i16::MIN as f64 || value > i16::MAX as f64 {
        return Err(format!(
            "{label} exceeds PSX fixed-point range. Apply transforms and export in meters."
        ));
    }
    Ok(value as i16)
}
fn pose(m: &ufbx::Matrix) -> Result<Pose, String> {
    let t = ufbx::matrix_to_transform(m);
    let rebuilt = ufbx::transform_to_matrix(&t);
    let a = [
        m.m00, m.m01, m.m02, m.m10, m.m11, m.m12, m.m20, m.m21, m.m22,
    ];
    let b = [
        rebuilt.m00,
        rebuilt.m01,
        rebuilt.m02,
        rebuilt.m10,
        rebuilt.m11,
        rebuilt.m12,
        rebuilt.m20,
        rebuilt.m21,
        rebuilt.m22,
    ];
    if a.iter()
        .zip(b)
        .any(|(a, b)| (*a - b).abs() > 1e-5 * a.abs().max(1.))
    {
        return Err(
            "Bone transform contains shear; bake/apply armature transforms before export".into(),
        );
    }
    Ok(Pose {
        translation: [
            q(t.translation.x, 256., "Bone translation")?,
            q(t.translation.y, 256., "Bone translation")?,
            q(t.translation.z, 256., "Bone translation")?,
        ],
        rotation: [
            q(t.rotation.x, 4096., "Quaternion")?,
            q(t.rotation.y, 4096., "Quaternion")?,
            q(t.rotation.z, 4096., "Quaternion")?,
            q(t.rotation.w, 4096., "Quaternion")?,
        ],
        scale: [
            q(t.scale.x, 4096., "Bone scale")?,
            q(t.scale.y, 4096., "Bone scale")?,
            q(t.scale.z, 4096., "Bone scale")?,
        ],
    })
}
// Normalize reference scale into rigid bind vertices. FBX armatures commonly carry
// a scale of 100 even when the character is correctly authored in meters.
fn normalized_world(n: &ufbx::Node, scales: &BTreeMap<u32, [f64; 3]>) -> ufbx::Matrix {
    let mut m = n.node_to_world;
    let s = scales[&n.element.typed_id];
    m.m00 /= s[0];
    m.m10 /= s[0];
    m.m20 /= s[0];
    m.m01 /= s[1];
    m.m11 /= s[1];
    m.m21 /= s[1];
    m.m02 /= s[2];
    m.m12 /= s[2];
    m.m22 /= s[2];
    m
}
fn local_pose(n: &ufbx::Node, scales: &BTreeMap<u32, [f64; 3]>) -> Result<Pose, String> {
    let world = normalized_world(n, scales);
    let local = if let Some(parent) = &n.parent {
        ufbx::matrix_mul(
            &ufbx::matrix_invert(&normalized_world(parent, scales)),
            &world,
        )
    } else {
        world
    };
    pose(&local).map_err(|e| format!("Bone '{}': {e}", n.element.name))
}
pub struct Imported {
    pub outputs: Vec<(String, Data)>,
}
// Names are identities, not array positions. Ambiguous names fail instead of silently rebinding.
fn output_id(settings: &mut Settings, key: &str) -> Uuid {
    let kind = key.split('/').next().unwrap_or("Asset");
    let label = key
        .rsplit('/')
        .next()
        .unwrap_or("Asset")
        .rsplit('|')
        .next()
        .unwrap_or("Asset")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .take(64)
        .collect::<String>();
    let file = if label == "main" {
        format!("{kind}.epokasset")
    } else {
        format!("{label}.{kind}.epokasset")
    };
    let occupied = settings.outputs.values().any(|o| o.file == file);
    settings
        .outputs
        .entry(key.into())
        .or_insert_with(|| {
            let id = Uuid::new_v4();
            Output {
                id,
                file: if occupied {
                    format!("{label}-{id}.{kind}.epokasset")
                } else {
                    file
                },
            }
        })
        .id
}
pub fn decode(bytes: &[u8], settings: &mut Settings) -> Result<Imported, String> {
    settings.validate()?;
    let scene = ufbx::load_memory(
        bytes,
        ufbx::LoadOpts {
            target_axes: ufbx::axes_left_handed_y_up(),
            target_unit_meters: 1.,
            space_conversion: ufbx::SpaceConversion::TransformRoot,
            inherit_mode_handling: ufbx::InheritModeHandling::HelperNodes,
            geometry_transform_handling: ufbx::GeometryTransformHandling::Preserve,
            load_external_files: false,
            ignore_embedded: true,
            clean_skin_weights: true,
            node_depth_limit: 64,
            temp_allocator: ufbx::AllocatorOpts {
                memory_limit: 256 * 1024 * 1024,
                ..Default::default()
            },
            result_allocator: ufbx::AllocatorOpts {
                memory_limit: 256 * 1024 * 1024,
                ..Default::default()
            },
            file_format: ufbx::FileFormat::Fbx,
            ..Default::default()
        },
    )
    .map_err(|e| format!("FBX: {e:?}"))?;
    let meshes = scene
        .meshes
        .iter()
        .filter(|m| !m.skin_deformers.is_empty())
        .collect::<Vec<_>>();
    if meshes.is_empty() {
        return Err(
            "FBX has no skinned meshes. Export the mesh and its armature with skin weights.".into(),
        );
    }
    if meshes.iter().map(|m| m.num_vertices).sum::<usize>() > skeletal::MAX_VERTICES
        || meshes.iter().map(|m| m.num_triangles).sum::<usize>() > skeletal::MAX_TRIANGLES
    {
        return Err("PSX rigid model limit: 512 vertices and 1024 triangles across all skinned meshes. Reduce the model before importing.".into());
    }
    let mut needed = BTreeSet::new();
    let mut warnings = vec![];
    for m in &meshes {
        if m.skin_deformers.len() != 1
            || !m.blend_deformers.is_empty()
            || !m.cache_deformers.is_empty()
        {
            return Err("Use one skin deformer per mesh, without blend shapes or vertex caches in this first importer".into());
        }
        let skin = &m.skin_deformers[0];
        if !matches!(
            skin.skinning_method,
            ufbx::SkinningMethod::Linear | ufbx::SkinningMethod::Rigid
        ) {
            return Err("Dual quaternion skinning is not supported; export linear skinning".into());
        }
        for cluster in &skin.clusters {
            let mut n = Some(
                cluster
                    .bone_node
                    .as_ref()
                    .ok_or("Skin cluster has no bone")?,
            );
            while let Some(node) = n {
                needed.insert(node.element.typed_id);
                n = node.parent.as_ref();
            }
        }
    }
    let mut nodes = scene
        .nodes
        .iter()
        .filter(|n| needed.contains(&n.element.typed_id))
        .collect::<Vec<_>>();
    nodes.sort_by_key(|n| (n.node_depth, n.element.typed_id));
    if nodes.len() > skeletal::MAX_BONES {
        return Err(format!(
            "{} bones/helpers exceed the 64 bone PSX limit",
            nodes.len()
        ));
    }
    let bone_indices = nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.element.typed_id, i))
        .collect::<BTreeMap<_, _>>();
    let mut scales = BTreeMap::new();
    for n in &nodes {
        let m = n.node_to_world;
        let scale = [
            (m.m00 * m.m00 + m.m10 * m.m10 + m.m20 * m.m20).sqrt(),
            (m.m01 * m.m01 + m.m11 * m.m11 + m.m21 * m.m21).sqrt(),
            (m.m02 * m.m02 + m.m12 * m.m12 + m.m22 * m.m22).sqrt(),
        ];
        if scale.iter().any(|s| !s.is_finite() || *s < 1e-9) {
            return Err("Singular bone transform".into());
        }
        scales.insert(n.element.typed_id, scale);
    }
    let skeleton_id = output_id(settings, "Skeleton/main");
    let skeleton = Skeleton {
        bones: nodes
            .iter()
            .map(|n| {
                Ok(Bone {
                    name: n.element.name.to_string(),
                    parent: n
                        .parent
                        .as_ref()
                        .and_then(|p| bone_indices.get(&p.element.typed_id))
                        .map_or(-1, |v| *v as i16),
                    bind: local_pose(n, &scales)?,
                })
            })
            .collect::<Result<_, String>>()?,
    };
    let mut outputs = vec![("Skeleton/main".into(), Data::Skeleton(skeleton))];
    let mut mesh = Mesh {
        skeleton: skeleton_id,
        vertices: vec![],
        triangles: vec![],
        materials: vec![],
        clips: vec![],
    };
    let mut material_keys = BTreeMap::<u32, usize>::new();
    let mut names = BTreeSet::new();
    // Only referenced materials become assets.
    for m in &meshes {
        for mat in &m.materials {
            if material_keys.contains_key(&mat.element.typed_id) {
                continue;
            }
            let key = format!("Material/{}", mat.element.name);
            if key == "Material/__default" {
                return Err("Material name '__default' is reserved".into());
            }
            if !names.insert(key.clone()) {
                return Err(format!(
                    "Duplicate material name '{}': give materials unique names",
                    mat.element.name
                ));
            }
            material_keys.insert(mat.element.typed_id, mesh.materials.len());
            mesh.materials.push(output_id(settings, &key));
            let c = mat.fbx.diffuse_color.value_vec4;
            let color = if mat.fbx.diffuse_color.has_value {
                [c.x as f32, c.y as f32, c.z as f32].map(|v| v.clamp(0., 1.))
            } else {
                [0.7; 3]
            };
            outputs.push((
                key,
                Data::Material(crate::scene::Material {
                    color,
                    unlit: true,
                    ..Default::default()
                }),
            ));
        }
    }
    let default_index = mesh.materials.len();
    mesh.materials
        .push(output_id(settings, "Material/__default"));
    outputs.push((
        "Material/__default".into(),
        Data::Material(crate::scene::Material {
            color: [0.65, 0.72, 0.8],
            unlit: true,
            ..Default::default()
        }),
    ));
    let mut reduced = 0;
    let mut tri_indices = vec![];
    for m in &meshes {
        let offset = mesh.vertices.len();
        let skin = &m.skin_deformers[0];
        for (i, v) in m.vertices.iter().enumerate() {
            let sv = skin.vertices.get(i).ok_or("Missing vertex skin data")?;
            let weights = &skin.weights.as_ref()
                [sv.weight_begin as usize..(sv.weight_begin + sv.num_weights) as usize];
            let weight = weights
                .iter()
                .filter(|w| w.weight.is_finite() && w.weight > 0.)
                .max_by(|a, b| a.weight.total_cmp(&b.weight))
                .ok_or("Unweighted vertex: assign all vertices to bones before exporting")?;
            if weights.iter().filter(|w| w.weight > 0.).count() > 1 {
                reduced += 1;
            }
            let cluster = skin
                .clusters
                .get(weight.cluster_index as usize)
                .ok_or("Invalid skin cluster index")?;
            let node = cluster.bone_node.as_ref().ok_or("Missing skin bone")?;
            let local = ufbx::transform_position(&cluster.geometry_to_bone, *v);
            let scale = scales[&node.element.typed_id];
            mesh.vertices.push(Vertex {
                position: [
                    q(local.x * scale[0], 4096., "Bone-local vertex")?,
                    q(local.y * scale[1], 4096., "Bone-local vertex")?,
                    q(local.z * scale[2], 4096., "Bone-local vertex")?,
                ],
                bone: bone_indices[&node.element.typed_id] as u8,
            });
        }
        for (f, face) in m.faces.iter().enumerate() {
            let count = ufbx::triangulate_face_vec(&mut tri_indices, m, *face) as usize;
            let material = m
                .face_material
                .get(f)
                .and_then(|i| m.materials.get(*i as usize))
                .and_then(|m| material_keys.get(&m.element.typed_id))
                .copied()
                .unwrap_or(default_index) as u16;
            for t in tri_indices[..count * 3].chunks_exact(3) {
                // FBX winding converted to the renderer's clockwise front faces.
                let indices = [t[0], t[2], t[1]]
                    .map(|i| (offset + m.vertex_indices[i as usize] as usize) as u16);
                mesh.triangles.push(Triangle { indices, material });
            }
        }
    }
    if reduced > 0 {
        warnings.push(format!(
            "{reduced} vertices had multiple weights; retained the strongest bone (rigid skinning)."
        ));
    }
    if scene.meshes.len() != meshes.len() {
        warnings.push("Unskinned meshes are excluded from this skeletal model.".into());
    }
    if !scene.textures.is_empty() {
        warnings.push(
            "Texture references are not imported yet; materials use flat diffuse colors.".into(),
        );
    }
    if scene.anim_stacks.len() > 16 {
        return Err("Maximum 16 animation clips per model".into());
    }
    let mut clip_names = BTreeSet::new();
    let mut pose_bytes = 0;
    for stack in &scene.anim_stacks {
        let name = stack.element.name.to_string();
        if !clip_names.insert(name.clone()) {
            return Err(format!("Duplicate animation name '{name}'"));
        }
        let duration = stack.time_end - stack.time_begin;
        if !duration.is_finite() || !(0. ..=60.).contains(&duration) {
            return Err(format!("Animation '{name}' must be at most 60 seconds"));
        }
        let frames = (duration * 30.).ceil() as u16 + 1;
        let mut tracks = vec![vec![]; nodes.len()];
        for frame in 0..frames {
            let time = stack.time_begin + (frame as f64 / 30.).min(duration);
            let evaluated = ufbx::evaluate_scene(
                &scene,
                &stack.anim,
                time,
                ufbx::EvaluateOpts {
                    evaluate_skinning: false,
                    ..Default::default()
                },
            )
            .map_err(|e| format!("Animation '{name}': {e:?}"))?;
            for (i, n) in nodes.iter().enumerate() {
                tracks[i].push(local_pose(
                    &evaluated.nodes[n.element.typed_id as usize],
                    &scales,
                )?);
            }
        }
        for track in &mut tracks {
            if track.iter().all(|p| p == &track[0]) {
                track.truncate(1);
            }
        }
        pose_bytes += tracks.iter().map(Vec::len).sum::<usize>() * 20;
        if pose_bytes > skeletal::MAX_CLIP_BYTES {
            return Err("Combined animation samples exceed 512 KiB PSX budget".into());
        }
        let key = format!("AnimationClip/{name}");
        mesh.clips.push(output_id(settings, &key));
        outputs.push((
            key,
            Data::AnimationClip(Clip {
                skeleton: skeleton_id,
                name,
                fps: 30,
                frames,
                tracks,
            }),
        ));
    }
    outputs.push(("SkeletalMesh/main".into(), Data::SkeletalMesh(mesh)));
    output_id(settings, "SkeletalMesh/main");
    // Validate every converted payload before any project asset is modified.
    for (_, data) in &outputs {
        Data::parse(&serde_json::to_vec(data).map_err(|e| e.to_string())?)?;
    }
    if !scene.metadata.warnings.is_empty() {
        warnings.push(format!(
            "ufbx reported {} source warnings; review the character preview.",
            scene.metadata.warnings.len()
        ));
    }
    settings.warnings = warnings;
    let converted_mesh = outputs
        .iter()
        .find_map(|(_, d)| {
            if let Data::SkeletalMesh(m) = d {
                Some(m.clone())
            } else {
                None
            }
        })
        .unwrap();
    let converted_skeleton = outputs
        .iter()
        .find_map(|(_, d)| {
            if let Data::Skeleton(s) = d {
                Some(s.clone())
            } else {
                None
            }
        })
        .unwrap();
    let converted_clips = outputs
        .iter()
        .filter_map(|(key, d)| {
            if let Data::AnimationClip(c) = d {
                Some((settings.outputs[key].id, c.clone()))
            } else {
                None
            }
        })
        .collect();
    let model = skeletal::Model {
        mesh: converted_mesh,
        skeleton: converted_skeleton,
        clips: converted_clips,
        materials: vec![],
    };
    crate::skeletal_compile::validate_bounds(&model)?;
    Ok(Imported { outputs })
}
pub struct Candidate {
    root: PathBuf,
    destination: PathBuf,
    source: Option<PathBuf>,
    source_hash: String,
    expected: BTreeMap<PathBuf, String>,
    files: Vec<(String, Vec<u8>)>,
    pub id: Uuid,
}
pub fn destination(source: &str) -> String {
    Path::new(source)
        .with_extension("imported")
        .join("Model.epokasset")
        .to_string_lossy()
        .replace('\\', "/")
}
pub fn prepare(
    root: &Path,
    source: &str,
    destination: &str,
    existing: Option<&Record>,
    snapshot: bool,
) -> Result<Candidate, String> {
    let path = assets::inside(root, destination)?;
    if path.file_name().is_none_or(|v| v != "Model.epokasset")
        || path
            .parent()
            .and_then(Path::extension)
            .is_none_or(|v| v != "imported")
    {
        return Err("FBX destination must be assets/<name>.imported/Model.epokasset".into());
    }
    let old = existing.map(|r| Package::load(&r.path)).transpose()?;
    let mut settings = match &old {
        Some(p) => match &p.meta.settings {
            import_settings::Settings::Fbx(s) => s.clone(),
            _ => return Err("Select a ModelSource asset to reimport".into()),
        },
        None => Settings::default(),
    };
    if let Some(r) = existing
        && r.path != path
    {
        return Err("Reimport must use the original model directory".into());
    }
    let source_path = if snapshot {
        None
    } else {
        Some(assets::inside(root, source)?)
    };
    let bytes = if let Some(p) = &source_path {
        assets::read_bounded(p)?
    } else {
        old.as_ref().ok_or("No FBX snapshot")?.source.clone()
    };
    let source_hash = assets::hash(&bytes);
    let id = existing.map_or_else(Uuid::new_v4, |r| r.meta.id);
    let mut expected = BTreeMap::new();
    let directory = path.parent().unwrap();
    if existing.is_some() {
        for entry in fs::read_dir(directory).map_err(|e| e.to_string())? {
            let p = entry.map_err(|e| e.to_string())?.path();
            if !p.is_file() {
                return Err("Model import directory must contain only its generated files".into());
            }
            expected.insert(p.clone(), assets::hash(&assets::read_bounded(&p)?));
        }
        if expected.get(&path) != existing.map(|r| &r.revision) {
            return Err("Model changed before import; refresh and retry".into());
        }
        for o in settings.outputs.values() {
            if !expected.contains_key(&directory.join(&o.file)) {
                return Err(
                    "A generated model asset was moved/deleted. Restore it before reimporting."
                        .into(),
                );
            }
        }
    } else if directory.exists() {
        return Err("Import directory already exists; select its ModelSource and Reimport".into());
    }
    let imported = decode(&bytes, &mut settings)?;
    let mut files = vec![];
    // Preserve material edits and retired subassets: existing scenes never lose identities.
    for p in expected.keys() {
        if p != &path {
            files.push((
                p.file_name().unwrap().to_string_lossy().into_owned(),
                assets::read_bounded(p)?,
            ));
        }
    }
    let source_name = if snapshot {
        old.as_ref().unwrap().meta.source.clone()
    } else {
        source.replace('\\', "/")
    };
    for (key, data) in imported.outputs {
        let output = &settings.outputs[&key];
        if matches!(data, Data::Material(_)) && expected.contains_key(&directory.join(&output.file))
        {
            continue;
        }
        let payload = serde_json::to_vec(&data).map_err(|e| e.to_string())?;
        let package = Package {
            meta: Metadata {
                version: 2,
                id: output.id,
                kind: data.kind(),
                importer_version: 1,
                source: assets::path_string(root, &path),
                source_hash: assets::hash(&payload),
                settings: import_settings::Settings::Derived,
                extra: Default::default(),
            },
            source: payload,
        };
        files.retain(|(file, _)| file != &output.file);
        files.push((output.file.clone(), package.bytes()?));
    }
    let package = Package {
        meta: Metadata {
            version: 2,
            id,
            kind: Kind::ModelSource,
            importer_version: 1,
            source: source_name,
            source_hash: source_hash.clone(),
            settings: import_settings::Settings::Fbx(settings),
            extra: Default::default(),
        },
        source: bytes,
    };
    files.push(("Model.epokasset".into(), package.bytes()?));
    Ok(Candidate {
        root: root.into(),
        destination: path,
        source: source_path,
        source_hash,
        expected,
        files,
        id,
    })
}
pub fn commit(c: Candidate) -> Result<Uuid, String> {
    if let Some(p) = &c.source
        && assets::hash(&assets::read_bounded(p)?) != c.source_hash
    {
        return Err("FBX changed during conversion; retry. Existing assets are intact.".into());
    }
    let directory = c.destination.parent().unwrap();
    if !c.expected.is_empty() {
        let mut current = BTreeMap::new();
        for entry in fs::read_dir(directory).map_err(|e| e.to_string())? {
            let p = entry.map_err(|e| e.to_string())?.path();
            current.insert(p.clone(), assets::hash(&assets::read_bounded(&p)?));
        }
        if current != c.expected {
            return Err(
                "Model assets changed during conversion; retry. Existing edits are intact.".into(),
            );
        }
    } else if directory.exists() {
        return Err("Import destination appeared during conversion".into());
    }
    let staging = c
        .root
        .join(".epok/model-staging")
        .join(Uuid::new_v4().to_string());
    fs::create_dir_all(&staging).map_err(|e| e.to_string())?;
    for (file, bytes) in &c.files {
        assets::atomic_write(&staging.join(file), bytes, None)?;
    }
    fs::create_dir_all(directory.parent().unwrap()).map_err(|e| e.to_string())?;
    if c.expected.is_empty() {
        fs::rename(&staging, directory).map_err(|e| e.to_string())?;
    } else {
        let backup =
            c.root
                .join(".epok/model-backups")
                .join(format!("{}-{}", c.id, Uuid::new_v4()));
        fs::create_dir_all(backup.parent().unwrap()).map_err(|e| e.to_string())?;
        fs::rename(directory, &backup).map_err(|e| format!("Cannot back up model assets: {e}"))?;
        if let Err(e) = fs::rename(&staging, directory) {
            fs::rename(&backup, directory).map_err(|rollback| {
                format!(
                    "Publish failed: {e}. Restore {} manually: {rollback}",
                    backup.display()
                )
            })?;
            return Err(e.to_string());
        }
    }
    Ok(c.id)
}
