//! Portable baked visibility tracks. Source face UUIDs survive mesh cooking;
//! the PSX receives XOR changes and two small masks, not a list of every face
//! for every camera sample. No game-specific identifiers enter the runtime.
use crate::scene::Scene;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sample {
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub fov: f32,
    pub visible: Vec<u16>,
    #[serde(default)]
    pub cut: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Track {
    pub camera: Uuid,
    pub sample_ms: u32,
    #[serde(default = "default_cull")]
    pub cull_backfaces: bool,
    pub faces: Vec<Uuid>,
    pub samples: Vec<Sample>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub navigation: Option<crate::rail_navigation::Navigation>,
}
fn default_cull() -> bool {
    true
}
impl Track {
    pub fn validate(&self, scene: &Scene) -> Result<(), String> {
        if !scene
            .actors
            .iter()
            .any(|a| a.id == self.camera && a.kind == "Camera" && a.parent.is_none())
        {
            return Err("Visibility track requires a root camera actor".into());
        }
        if self.samples.is_empty()
            || self.samples.len() > 65535
            || self.faces.len() > 65535
            || self.sample_ms < 16
            || self.sample_ms > 60000
        {
            return Err("Visibility track exceeds sample/face limits or has invalid timing".into());
        }
        if self.faces.iter().copied().collect::<BTreeSet<_>>().len() != self.faces.len() {
            return Err("Visibility track contains duplicate face IDs".into());
        }
        for sample in &self.samples {
            if sample
                .position
                .iter()
                .chain(&sample.rotation)
                .any(|v| !v.is_finite() || v.abs() > 32767.)
                || !sample.fov.is_finite()
                || !(25. ..=120.).contains(&sample.fov)
                || sample
                    .visible
                    .iter()
                    .any(|&i| usize::from(i) >= self.faces.len())
            {
                return Err(
                    "Visibility track sample has invalid coordinates or face indices".into(),
                );
            }
        }
        if let Some(nav) = &self.navigation {
            nav.validate(scene, self)?;
        }
        Ok(())
    }
    pub fn residency_pairs(&self) -> Vec<(usize, usize)> {
        if let Some(nav) = &self.navigation {
            return nav.pairs();
        }
        (0..self.samples.len())
            .filter_map(|i| {
                let next = (i + 1) % self.samples.len();
                (!self.samples[next].cut).then_some((i, next))
            })
            .collect()
    }
}

pub struct Cooked {
    pub chunks: BTreeMap<String, (usize, Vec<Uuid>)>,
    pub bits: Vec<BTreeSet<usize>>,
    pub words: usize,
}
fn encode_changes(bits: &BTreeSet<usize>, previous: &BTreeSet<usize>) -> Vec<u8> {
    fn varint(out: &mut Vec<u8>, mut n: usize) {
        while n >= 128 {
            out.push((n as u8 & 127) | 128);
            n >>= 7;
        }
        out.push(n as u8);
    }
    let delta: Vec<_> = bits.symmetric_difference(previous).copied().collect();
    let mut out = Vec::new();
    let mut cursor = 0;
    let mut i = 0;
    while i < delta.len() {
        let begin = delta[i];
        let mut end = i + 1;
        while end < delta.len() && delta[end] == begin + end - i {
            end += 1;
        }
        let count = end - i;
        varint(&mut out, ((begin - cursor) << 1) | usize::from(count > 1));
        if count > 1 {
            varint(&mut out, count - 2);
        }
        cursor = begin + count;
        i = end;
    }
    out
}
pub fn cook(scene: &Scene) -> Result<Option<Cooked>, String> {
    let Some(track) = &scene.visibility_track else {
        return Ok(None);
    };
    track.validate(scene)?;
    let wanted: BTreeSet<_> = track.faces.iter().copied().collect();
    let mut lookup: BTreeMap<Uuid, Vec<usize>> = BTreeMap::new();
    let mut chunks = BTreeMap::new();
    let mut words = 0;
    for (actor, e) in scene.actors.iter().enumerate() {
        let Some(mesh) = &e.editable_mesh else {
            continue;
        };
        let Some(doc) = &mesh.document else {
            return Err("Unresolved mesh in visibility track".into());
        };
        if !doc.faces.iter().any(|f| wanted.contains(&f.id)) {
            continue;
        }
        if e.kind != "Mesh" || e.skeletal_mesh.is_some() || e.terrain.is_some() {
            return Err("Visibility tracks require ordinary editable meshes".into());
        }
        if doc.faces.iter().any(|f| !wanted.contains(&f.id)) {
            return Err(
                "A tracked mesh must list every source face in the visibility track".into(),
            );
        }
        let qs = crate::lighting::quads(e);
        for (chunk_index, chunk) in crate::mesh_compile::chunks(&qs)?.iter().enumerate() {
            let ids: Vec<_> = chunk
                .faces
                .iter()
                .map(|(_, i)| qs[*i].id.unwrap())
                .collect();
            for (i, id) in ids.iter().enumerate() {
                lookup.entry(*id).or_default().push(words * 32 + i);
            }
            chunks.insert(format!("editable_{actor}_{chunk_index}"), (words, ids));
            words += chunk.faces.len().div_ceil(32);
        }
    }
    if words * 32 > 65536 || track.faces.iter().any(|id| !lookup.contains_key(id)) {
        return Err("Visibility track has missing mesh faces or exceeds 65536 cooked bits".into());
    }
    let bits = track
        .samples
        .iter()
        .map(|s| {
            s.visible
                .iter()
                .flat_map(|&i| lookup[&track.faces[usize::from(i)]].iter().copied())
                .collect()
        })
        .collect();
    Ok(Some(Cooked {
        chunks,
        bits,
        words,
    }))
}

pub fn rewrite(scene: &Scene, header: String) -> Result<String, String> {
    let Some(cooked) = cook(scene)? else {
        return Ok(header);
    };
    let track = scene.visibility_track.as_ref().unwrap();
    let camera = scene
        .actors
        .iter()
        .position(|a| a.id == track.camera)
        .unwrap();
    let mut changes = Vec::new();
    let mut samples = Vec::new();
    let values = track
        .samples
        .iter()
        .map(|s| {
            std::array::from_fn(|i| {
                ((if i < 3 {
                    s.position[i]
                } else if i < 6 {
                    s.rotation[i - 3]
                } else {
                    s.fov
                }) * 4096.)
                    .round() as i32
            })
        })
        .collect::<Vec<[i32; 7]>>();
    let packed = crate::visibility_codec::pack(&values).filter(|p| {
        p.samples.len() * 24 + p.origins.len() * 12 + p.scalars.len() * 4 < values.len() * 36
    });
    let mut previous = BTreeSet::new();
    for (index, (sample, bits)) in track.samples.iter().zip(&cooked.bits).enumerate() {
        if track
            .navigation
            .as_ref()
            .is_some_and(|nav| nav.routes.iter().any(|r| usize::from(r.first) == index))
        {
            previous.clear(); // Each route starts with a full checkpoint.
        }
        let delta = encode_changes(bits, &previous);
        if delta.len() > u16::MAX as usize {
            return Err("Visibility sample delta exceeds 65535 encoded bytes".into());
        }
        if let Some(packed) = &packed {
            let (p, origin, indices) = packed.samples[index];
            samples.push(format!(
                "{{{{{},{},{}}},{origin},{{{},{},{}}},{},{},{},{}}}",
                p[0],
                p[1],
                p[2],
                indices[0],
                indices[1],
                indices[2],
                indices[3],
                changes.len(),
                delta.len(),
                sample.cut
            ));
        } else {
            let position = &values[index][..3];
            let rotation = &values[index][3..6];
            samples.push(format!(
                "{{{{{},{},{}}},{{{},{},{}}},{},{},{},{}}}",
                position[0],
                position[1],
                position[2],
                rotation[0],
                rotation[1],
                rotation[2],
                values[index][6],
                changes.len(),
                delta.len(),
                sample.cut
            ));
        }
        changes.extend(delta);
        previous = bits.clone();
    }
    let (dictionary, sample_type, sample_ref, packed_ref) = if let Some(p) = &packed {
        let origins = p
            .origins
            .iter()
            .map(|v| format!("{{{},{},{}}}", v[0], v[1], v[2]))
            .collect::<Vec<_>>()
            .join(",");
        let scalars = p
            .scalars
            .iter()
            .map(i32::to_string)
            .collect::<Vec<_>>()
            .join(",");
        (
            format!(
                "inline constexpr int32_t track_origins[][3]={{{origins}}};\ninline constexpr int32_t track_scalars[]={{{scalars}}};\n"
            ),
            "PackedVisibilityTrackSample",
            "nullptr",
            "track_samples,track_origins,track_scalars",
        )
    } else {
        (
            String::new(),
            "VisibilityTrackSample",
            "track_samples",
            "nullptr,nullptr,nullptr",
        )
    };
    let data = format!(
        "{dictionary}{}inline uint32_t track_current[{}]={{}},track_next[{}]={{}};\ninline constexpr uint8_t track_changes[]={{{}}};\ninline constexpr {sample_type} track_samples[]={{{}}};\ninline constexpr VisibilityTrackData track_data={{{sample_ref},track_changes,track_current,track_next,{},{},{},{},{},{},{packed_ref}}};\n",
        track
            .navigation
            .as_ref()
            .map(|n| n.header(scene, cooked.words))
            .unwrap_or_default(),
        cooked.words.max(1),
        cooked.words.max(1),
        if changes.is_empty() {
            "0".into()
        } else {
            changes
                .iter()
                .map(u8::to_string)
                .collect::<Vec<_>>()
                .join(",")
        },
        samples.join(","),
        samples.len(),
        cooked.words,
        camera,
        track.sample_ms * 1000,
        track.cull_backfaces,
        if track.navigation.is_some() {
            "&rail_data"
        } else {
            "nullptr"
        }
    );
    let mut out = String::new();
    let mut inserted = false;
    for line in header.lines() {
        if !inserted && line == "namespace epok {" {
            out.push_str(line);
            out.push('\n');
            out.push_str(&data);
            inserted = true;
            continue;
        }
        let mut replacement = None;
        for (key, (word, _)) in &cooked.chunks {
            if line.starts_with(&format!("inline constexpr MeshGeometry {key}=")) {
                // Applied before the streaming rewrite, which inserts its three
                // transport fields immediately before this final pointer.
                let (span, count) = if let Some(prefix) = key.strip_suffix("_0") {
                    let prefix = format!("{prefix}_");
                    cooked.chunks.iter().filter(|(k, _)| k.starts_with(&prefix))
                        .fold((0, 0), |(words, count), (_, (_, ids))|
                            (words + ids.len().div_ceil(32), count + 1))
                } else { (0, 0) };
                replacement = Some(format!(
                    "{},0xffffffffu,0,0,track_current+{word},{span},{count}}};",
                    line.strip_suffix("};")
                        .ok_or("Unexpected geometry descriptor")?
                ));
                break;
            }
        }
        out.push_str(replacement.as_deref().unwrap_or(line));
        out.push('\n');
        if line == "epok::set_visibility_track(nullptr);" {
            out.push_str("epok::set_visibility_track(&track_data);\n");
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn visibility_change_encoding_handles_runs_gaps_and_last_bit() {
        let old = BTreeSet::new();
        assert_eq!(encode_changes(&old, &old), Vec::<u8>::new());
        assert_eq!(encode_changes(&BTreeSet::from([0, 33]), &old), [0, 64]);
        assert_eq!(
            encode_changes(&(0..65536).collect(), &old),
            [1, 254, 255, 3]
        );
        assert_eq!(
            encode_changes(&BTreeSet::from([65535]), &old),
            [254, 255, 7]
        );
        assert_eq!(
            encode_changes(&(0..32).collect(), &(1..33).collect()),
            [0, 62]
        );
    }
    #[test]
    fn visibility_track_preserves_large_source_faces_and_stream_bindings() {
        let mut scene = Scene::default();
        scene.actors.truncate(2);
        let mut doc = crate::mesh::Document::default();
        doc.primitive(
            "Cube",
            [0.; 3],
            [20.; 3],
            1,
            doc.groups[0].id,
            doc.materials[0].id,
        );
        let faces = doc.faces.iter().map(|f| f.id).collect::<Vec<_>>();
        scene.actors[1].editable_mesh = Some(crate::mesh::Component {
            asset: Uuid::new_v4(),
            materials: Default::default(),
            document: Some(std::sync::Arc::new(doc)),
            error: None,
        });
        scene.visibility_track = Some(Track {
            navigation: None,
            camera: scene.actors[0].id,
            sample_ms: 100,
            cull_backfaces: false,
            faces,
            samples: vec![
                Sample {
                    position: [0.; 3],
                    rotation: [0.; 3],
                    fov: 90.,
                    visible: vec![0],
                    cut: true,
                },
                Sample {
                    position: [1., 0., 0.],
                    rotation: [0.; 3],
                    fov: 90.,
                    visible: vec![1],
                    cut: false,
                },
            ],
        });
        let cooked = cook(&scene).unwrap().unwrap();
        assert_eq!(cooked.bits[0].len(), 1);
        assert_eq!(cooked.bits[1].len(), 1);
        assert!(cooked.bits[0].is_disjoint(&cooked.bits[1]));
        let header = crate::project::scene_header(&scene, &[]).unwrap();
        let archive = crate::streaming::compile(&[scene.clone()], 2).unwrap();
        let rewritten = archive.rewrite(0, &header).unwrap();
        assert!(rewritten.contains("MeshVertices{nullptr,12}"));
        let root = rewritten.lines().find(|l| l.starts_with("inline constexpr MeshGeometry editable_1_0=")).unwrap();
        assert!(root.ends_with(&format!("track_current+0,{},{}}};", cooked.words, cooked.chunks.len())));
        assert!(rewritten.contains("epok::set_visibility_track(&track_data);"));
        assert!(rewritten.lines().any(
            |l| l.starts_with("inline constexpr MeshGeometry editable_1_")
                && l.contains("nullptr")
                && l.contains("track_current+")
        ));
        let original_camera = scene.visibility_track.as_ref().unwrap().camera;
        scene.duplicate_identities();
        assert_ne!(
            scene.visibility_track.as_ref().unwrap().camera,
            original_camera
        );
        scene
            .visibility_track
            .as_ref()
            .unwrap()
            .validate(&scene)
            .unwrap();
        scene.visibility_track.as_mut().unwrap().samples[0].visible = vec![u16::MAX];
        assert!(cook(&scene).is_err());
    }
}
