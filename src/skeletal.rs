//! Rigid skinning: bind positions live in their owning bone's local space.
use crate::{
    assets::{Index, Kind, Package},
    scene::{Material, Scene},
    transform::Matrix,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Arc};
use uuid::Uuid;
pub const MAX_BONES: usize = 64;
pub const MAX_VERTICES: usize = 512;
pub const MAX_TRIANGLES: usize = 1024;
pub const MAX_CLIP_BYTES: usize = 512 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Pose {
    pub translation: [i16; 3],
    pub rotation: [i16; 4],
    pub scale: [i16; 3],
}
impl Pose {
    pub fn matrix(&self) -> Matrix {
        let [x, y, z, w] = self.rotation.map(|v| v as f32 / 4096.);
        let s = self.scale.map(|v| v as f32 / 4096.);
        Matrix([
            [
                (1. - 2. * (y * y + z * z)) * s[0],
                2. * (x * y - z * w) * s[1],
                2. * (x * z + y * w) * s[2],
                self.translation[0] as f32 / 256.,
            ],
            [
                2. * (x * y + z * w) * s[0],
                (1. - 2. * (x * x + z * z)) * s[1],
                2. * (y * z - x * w) * s[2],
                self.translation[1] as f32 / 256.,
            ],
            [
                2. * (x * z - y * w) * s[0],
                2. * (y * z + x * w) * s[1],
                (1. - 2. * (x * x + y * y)) * s[2],
                self.translation[2] as f32 / 256.,
            ],
        ])
    }
    fn validate(&self) -> Result<(), String> {
        let len: f32 = self
            .rotation
            .iter()
            .map(|v| (*v as f32 / 4096.).powi(2))
            .sum();
        if (len - 1.).abs() > 0.002 || self.scale.contains(&0) {
            return Err("Invalid quantized bone pose".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Bone {
    pub name: String,
    pub parent: i16,
    pub bind: Pose,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Skeleton {
    pub bones: Vec<Bone>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Vertex {
    pub position: [i16; 3],
    pub bone: u8,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Triangle {
    pub indices: [u16; 3],
    pub material: u16,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Mesh {
    pub skeleton: Uuid,
    pub vertices: Vec<Vertex>,
    pub triangles: Vec<Triangle>,
    pub materials: Vec<Uuid>,
    pub clips: Vec<Uuid>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Clip {
    pub skeleton: Uuid,
    pub name: String,
    pub fps: u16,
    pub frames: u16,
    pub tracks: Vec<Vec<Pose>>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum Data {
    Skeleton(Skeleton),
    SkeletalMesh(Mesh),
    AnimationClip(Clip),
    Material(Material),
}
impl Data {
    pub fn kind(&self) -> Kind {
        match self {
            Self::Skeleton(_) => Kind::Skeleton,
            Self::SkeletalMesh(_) => Kind::SkeletalMesh,
            Self::AnimationClip(_) => Kind::AnimationClip,
            Self::Material(_) => Kind::Material,
        }
    }
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let data: Self = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        match &data {
            Self::Skeleton(s) => {
                if s.bones.is_empty() || s.bones.len() > MAX_BONES {
                    return Err(
                        "Skeleton must contain 1..64 bones (including transform helpers)".into(),
                    );
                }
                for (i, b) in s.bones.iter().enumerate() {
                    if b.parent < -1 || b.parent >= i as i16 || b.name.len() > 512 {
                        return Err("Invalid skeleton hierarchy".into());
                    }
                    b.bind.validate()?;
                }
            }
            Self::SkeletalMesh(m) => {
                if m.skeleton.is_nil()
                    || m.vertices.is_empty()
                    || m.vertices.len() > MAX_VERTICES
                    || m.triangles.is_empty()
                    || m.triangles.len() > MAX_TRIANGLES
                    || m.materials.is_empty()
                    || m.materials.len() > 64
                    || m.clips.len() > 16
                {
                    return Err("Skeletal mesh exceeds limits: 512 vertices / 1024 triangles / 64 materials / 16 clips".into());
                }
                if m.vertices.iter().any(|v| v.bone as usize >= MAX_BONES)
                    || m.triangles.iter().any(|t| {
                        t.indices.iter().any(|v| *v as usize >= m.vertices.len())
                            || t.material as usize >= m.materials.len()
                    })
                    || m.materials.iter().chain(&m.clips).any(Uuid::is_nil)
                {
                    return Err("Invalid skeletal mesh references or indices".into());
                }
            }
            Self::AnimationClip(c) => {
                if c.skeleton.is_nil()
                    || c.fps != 30
                    || c.frames == 0
                    || c.frames > 1801
                    || c.tracks.is_empty()
                    || c.tracks.len() > MAX_BONES
                    || c.name.len() > 512
                    || c.tracks
                        .iter()
                        .any(|t| t.len() != 1 && t.len() != c.frames as usize)
                    || c.tracks.iter().map(Vec::len).sum::<usize>() * 20 > MAX_CLIP_BYTES
                {
                    return Err(
                        "Animation exceeds 60 seconds / 64 tracks / 512 KiB or has invalid tracks"
                            .into(),
                    );
                }
                for p in c.tracks.iter().flatten() {
                    p.validate()?;
                }
            }
            Self::Material(m) => {
                if m.color
                    .iter()
                    .any(|v| !v.is_finite() || !(0. ..=1.).contains(v))
                {
                    return Err("Invalid material color".into());
                }
            }
        }
        Ok(data)
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Model {
    pub mesh: Mesh,
    pub skeleton: Skeleton,
    pub clips: Vec<(Uuid, Clip)>,
    pub materials: Vec<Material>,
}
fn load(index: &Index, id: Uuid, kind: Kind) -> Result<Data, String> {
    let record = index.resolve(id)?;
    if record.meta.kind != kind {
        return Err(format!(
            "{id}: expected {kind:?}, found {:?}",
            record.meta.kind
        ));
    }
    Data::parse(&Package::load(&record.path)?.source)
}
impl Model {
    pub fn load(index: &Index, id: Uuid) -> Result<Self, String> {
        let Data::SkeletalMesh(mesh) = load(index, id, Kind::SkeletalMesh)? else {
            unreachable!()
        };
        let Data::Skeleton(skeleton) = load(index, mesh.skeleton, Kind::Skeleton)? else {
            unreachable!()
        };
        if mesh
            .vertices
            .iter()
            .any(|v| v.bone as usize >= skeleton.bones.len())
        {
            return Err("Mesh references a bone outside its skeleton".into());
        }
        let mut clips = vec![];
        for &id in &mesh.clips {
            let Data::AnimationClip(c) = load(index, id, Kind::AnimationClip)? else {
                unreachable!()
            };
            if c.skeleton != mesh.skeleton || c.tracks.len() != skeleton.bones.len() {
                return Err("Animation / skeleton mismatch".into());
            }
            clips.push((id, c));
        }
        if clips
            .iter()
            .map(|(_, c)| c.tracks.iter().map(Vec::len).sum::<usize>() * 20)
            .sum::<usize>()
            > MAX_CLIP_BYTES
        {
            return Err("Combined model clips exceed 512 KiB PSX budget".into());
        }
        let mut materials = vec![];
        for &id in &mesh.materials {
            let Data::Material(m) = load(index, id, Kind::Material)? else {
                unreachable!()
            };
            materials.push(m);
        }
        Ok(Self {
            mesh,
            skeleton,
            clips,
            materials,
        })
    }
    pub fn bones(&self, clip: Option<Uuid>, seconds: f32, looping: bool) -> Vec<Matrix> {
        let clip = clip
            .and_then(|id| self.clips.iter().find(|(key, _)| *key == id))
            .map(|(_, c)| c);
        let frame = clip.map_or(0, |c| {
            let frame = (seconds.max(0.) * c.fps as f32 + 0.0001).floor() as usize;
            // Final sample is the endpoint; loops wrap before the duplicate endpoint.
            if looping {
                frame % (c.frames as usize - 1).max(1)
            } else {
                frame.min(c.frames as usize - 1)
            }
        });
        let mut matrices = Vec::<Matrix>::new();
        for (i, b) in self.skeleton.bones.iter().enumerate() {
            let pose = clip.map_or(&b.bind, |c| {
                &c.tracks[i][if c.tracks[i].len() == 1 { 0 } else { frame }]
            });
            let local = pose.matrix();
            matrices.push(if b.parent < 0 {
                local
            } else {
                matrices[b.parent as usize].compose(local)
            });
        }
        matrices
    }
    pub fn points(&self, clip: Option<Uuid>, seconds: f32, looping: bool) -> Vec<[f32; 3]> {
        let bones = self.bones(clip, seconds, looping);
        self.mesh
            .vertices
            .iter()
            .map(|v| bones[v.bone as usize].point(v.position.map(|x| x as f32 / 4096.)))
            .collect()
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Component {
    pub asset: Uuid,
    pub clip: Option<Uuid>,
    #[serde(default = "yes")]
    pub looping: bool,
    #[serde(default = "yes")]
    pub play_on_start: bool,
    #[serde(skip)]
    pub model: Option<Arc<Model>>,
    #[serde(skip)]
    pub time: f32,
    #[serde(skip)]
    pub error: Option<String>,
}
fn yes() -> bool {
    true
}
impl Component {
    pub fn new(asset: Uuid) -> Self {
        Self {
            asset,
            clip: None,
            looping: true,
            play_on_start: true,
            model: None,
            time: 0.,
            error: None,
        }
    }
}
pub fn validate(scene: &Scene) -> Result<(), String> {
    for e in &scene.actors {
        if let Some(c) = &e.skeletal_mesh
            && (e.kind != "Mesh"
                || e.editable_mesh.is_some()
                || c.asset.is_nil()
                || c.clip.is_some_and(|v| v.is_nil())
                || e.lighting.static_geometry
                || e.lighting.receive == crate::lighting::Receive::Baked)
        {
            return Err(format!(
                "{}: skeletal meshes require a valid asset, a Mesh entity and dynamic lighting",
                e.name
            ));
        }
    }
    Ok(())
}
pub fn resolve(scene: &mut Scene, index: &Index) -> Result<(), String> {
    let mut cache = BTreeMap::new();
    let mut errors = vec![];
    for e in &mut scene.actors {
        if let Some(c) = &mut e.skeletal_mesh {
            let result = cache
                .entry(c.asset)
                .or_insert_with(|| Model::load(index, c.asset).map(Arc::new));
            match result {
                Ok(m) => {
                    c.model = Some(m.clone());
                    c.error = None;
                    if c.clip
                        .is_some_and(|id| !m.clips.iter().any(|(key, _)| *key == id))
                    {
                        let error =
                            "Selected animation is missing from the model; choose another clip"
                                .to_string();
                        c.error = Some(error.clone());
                        errors.push(format!("{}: {error}", e.name));
                    }
                }
                other => {
                    let error =
                        other.as_ref().err().cloned().unwrap_or_else(|| {
                            "Selected animation is missing from the model".into()
                        });
                    c.model = None;
                    c.error = Some(error.clone());
                    errors.push(format!("{}: {error}", e.name));
                }
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}
pub fn quads(c: &Component) -> Vec<crate::lighting::Quad> {
    let Some(m) = &c.model else {
        return vec![];
    };
    let vertices = m.points(c.clip, c.time, c.looping);
    m.mesh
        .triangles
        .iter()
        .map(|t| {
            let [a, b, d] = t.indices.map(|v| vertices[v as usize]);
            let points = [a, b, d, d];
            crate::lighting::Quad {
                uv: [[0., 0.], [1., 0.], [1., 1.], [1., 1.]],
                face: 0,
                normal: crate::mesh::face_normal(points),
                points,
                material: m.materials[t.material as usize].clone(),
                id: None,
            }
        })
        .collect()
}
