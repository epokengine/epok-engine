//! Compile portable UUID-linked assets into immutable PSX tables.
use crate::{
    scene::Actor,
    skeletal::{AnimationStorage, MAX_CLIP_BYTES, Model, Pose, Triangle, Vertex},
};
use std::{collections::BTreeMap, fmt::Write};

fn array<T: ToString>(v: &[T]) -> String {
    v.iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
fn pose(p: &Pose) -> String {
    format!(
        "{{{{{}}},{{{}}},{{{}}}}}",
        array(&p.translation),
        array(&p.rotation),
        array(&p.scale)
    )
}
fn name(value: &str) -> String {
    value
        .bytes()
        .map(|b| format!("\\{b:03o}"))
        .collect::<String>()
}

fn q12(points: &[[f32; 3]]) -> Vec<[i16; 3]> {
    points
        .iter()
        .map(|p| p.map(|v| (v as f64 * 4096.).round() as i16))
        .collect()
}

type FixedMatrix = [[i32; 4]; 3];
fn fixed_mul(a: i32, b: i32) -> i32 {
    (i64::from(a) * i64::from(b) / 4096) as i32
}
fn fixed_pose(p: &Pose) -> FixedMatrix {
    let [x, y, z, w] = p.rotation.map(i32::from);
    let two = 8192;
    let mut matrix = [
        [
            4096 - fixed_mul(two, fixed_mul(y, y) + fixed_mul(z, z)),
            fixed_mul(two, fixed_mul(x, y) - fixed_mul(z, w)),
            fixed_mul(two, fixed_mul(x, z) + fixed_mul(y, w)),
            i32::from(p.translation[0]) * 16,
        ],
        [
            fixed_mul(two, fixed_mul(x, y) + fixed_mul(z, w)),
            4096 - fixed_mul(two, fixed_mul(x, x) + fixed_mul(z, z)),
            fixed_mul(two, fixed_mul(y, z) - fixed_mul(x, w)),
            i32::from(p.translation[1]) * 16,
        ],
        [
            fixed_mul(two, fixed_mul(x, z) - fixed_mul(y, w)),
            fixed_mul(two, fixed_mul(y, z) + fixed_mul(x, w)),
            4096 - fixed_mul(two, fixed_mul(x, x) + fixed_mul(y, y)),
            i32::from(p.translation[2]) * 16,
        ],
    ];
    for row in &mut matrix {
        for column in 0..3 {
            row[column] = fixed_mul(row[column], i32::from(p.scale[column]));
        }
    }
    matrix
}
fn fixed_compose(a: FixedMatrix, b: FixedMatrix) -> FixedMatrix {
    std::array::from_fn(|row| {
        std::array::from_fn(|column| {
            let translation = if column == 3 { a[row][3] } else { 0 };
            translation
                + (0..3)
                    .map(|k| fixed_mul(a[row][k], b[k][column]))
                    .sum::<i32>()
        })
    })
}
fn fixed_points(m: &Model, clip: Option<&crate::skeletal::Clip>, frame: usize) -> Vec<[i32; 3]> {
    let mut bones = Vec::<FixedMatrix>::with_capacity(m.skeleton.bones.len());
    for (index, bone) in m.skeleton.bones.iter().enumerate() {
        let pose = clip.map_or(&bone.bind, |clip| {
            &clip.tracks[index][if clip.tracks[index].len() == 1 {
                0
            } else {
                frame
            }]
        });
        let local = fixed_pose(pose);
        bones.push(if bone.parent < 0 {
            local
        } else {
            fixed_compose(bones[bone.parent as usize], local)
        });
    }
    m.mesh
        .vertices
        .iter()
        .map(|vertex| {
            let matrix = bones[vertex.bone as usize];
            std::array::from_fn(|row| {
                matrix[row][3]
                    + (0..3)
                        .map(|column| {
                            fixed_mul(matrix[row][column], i32::from(vertex.position[column]))
                        })
                        .sum::<i32>()
            })
        })
        .collect()
}

/// Conservative model-local bounds over bind pose and every quantized sample.
/// They allow the target to reject an off-screen character before evaluating
/// bones or decoding a baked vertex frame.
fn animated_bounds(m: &Model) -> ([i32; 3], [i32; 3]) {
    let mut low = [i32::MAX; 3];
    let mut high = [i32::MIN; 3];
    let mut include = |points: Vec<[i32; 3]>| {
        for p in points {
            for c in 0..3 {
                low[c] = low[c].min(p[c]);
                high[c] = high[c].max(p[c]);
            }
        }
    };
    let baked = m.mesh.animation_storage == AnimationStorage::BakedVertices;
    if baked {
        let quantized = |points: Vec<[f32; 3]>| {
            q12(&points)
                .into_iter()
                .map(|point| point.map(i32::from))
                .collect()
        };
        include(quantized(m.points(None, 0., false)));
        for (id, clip) in &m.clips {
            for frame in 0..clip.frames {
                include(quantized(m.points(
                    Some(*id),
                    frame as f32 / 30. + 0.00001,
                    false,
                )));
            }
        }
    } else {
        include(fixed_points(m, None, 0));
        for (_, clip) in &m.clips {
            for frame in 0..clip.frames as usize {
                include(fixed_points(m, Some(clip), frame));
            }
        }
    }
    let center = std::array::from_fn(|c| (low[c] + high[c]) / 2);
    // Baked Q8 deltas add at most 8 Q12 units; the remaining guard covers GTE
    // input truncation and fixed compose association with the object matrix.
    let guard = if baked { 24 } else { 16 };
    let extent = std::array::from_fn(|c| (high[c] - low[c] + 1) / 2 + guard);
    (center, extent)
}

struct RigidOrder {
    vertices: Vec<Vertex>,
    triangles: Vec<Triangle>,
    bone_offsets: Vec<u16>,
}

/// The GTE path changes its matrix once per contiguous bone range. Reorder the
/// generated target vertices without changing the portable asset identity.
fn rigid_order(m: &Model) -> RigidOrder {
    let mut order = (0..m.mesh.vertices.len()).collect::<Vec<_>>();
    order.sort_by_key(|&i| (m.mesh.vertices[i].bone, i));
    let mut remap = vec![0_u16; order.len()];
    let vertices = order
        .iter()
        .enumerate()
        .map(|(new, &old)| {
            remap[old] = new as u16;
            m.mesh.vertices[old].clone()
        })
        .collect::<Vec<_>>();
    let triangles = m
        .mesh
        .triangles
        .iter()
        .map(|t| Triangle {
            indices: t.indices.map(|i| remap[i as usize]),
            material: t.material,
        })
        .collect();
    let mut bone_offsets = vec![0_u16; m.skeleton.bones.len() + 1];
    let mut cursor = 0;
    for bone in 0..m.skeleton.bones.len() {
        bone_offsets[bone] = cursor as u16;
        while cursor < vertices.len() && vertices[cursor].bone as usize == bone {
            cursor += 1;
        }
    }
    *bone_offsets.last_mut().unwrap() = cursor as u16;
    RigidOrder {
        vertices,
        triangles,
        bone_offsets,
    }
}

#[derive(Clone, Debug, PartialEq)]
struct EncodedFrame {
    offset: u32,
    raw: bool,
}
#[derive(Clone, Debug, PartialEq)]
struct EncodedVertexClip {
    frames: Vec<EncodedFrame>,
    data: Vec<u8>,
}

/// Independent frames support random clip changes and shared scratch. A compact
/// coordinate is a signed Q8 delta from bind pose; 0x80 escapes to an absolute
/// little-endian Q12 i16. If escapes make a frame larger, store raw i16s.
fn encode_vertex_clip(base: &[[i16; 3]], frames: &[Vec<[i16; 3]>]) -> EncodedVertexClip {
    let mut output = EncodedVertexClip {
        frames: Vec::with_capacity(frames.len()),
        data: vec![],
    };
    let mut known = BTreeMap::<(bool, Vec<u8>), u32>::new();
    for points in frames {
        let mut compact = Vec::with_capacity(points.len() * 3);
        let mut raw = Vec::with_capacity(points.len() * 6);
        for (bind, point) in base.iter().zip(points) {
            for c in 0..3 {
                raw.extend_from_slice(&point[c].to_le_bytes());
                let difference = i32::from(point[c]) - i32::from(bind[c]);
                let delta = if difference >= 0 {
                    (difference + 8) / 16
                } else {
                    (difference - 8) / 16
                };
                let reconstructed = i32::from(bind[c]) + delta * 16;
                if (-127..=127).contains(&delta)
                    && (i16::MIN as i32..=i16::MAX as i32).contains(&reconstructed)
                {
                    compact.push((delta as i8) as u8);
                } else {
                    compact.push(0x80);
                    compact.extend_from_slice(&point[c].to_le_bytes());
                }
            }
        }
        let (raw_frame, bytes) = if compact.len() < raw.len() {
            (false, compact)
        } else {
            (true, raw)
        };
        let key = (raw_frame, bytes);
        let offset = if let Some(offset) = known.get(&key) {
            *offset
        } else {
            let offset = output.data.len() as u32;
            output.data.extend_from_slice(&key.1);
            known.insert(key, offset);
            offset
        };
        output.frames.push(EncodedFrame {
            offset,
            raw: raw_frame,
        });
    }
    output
}

pub fn validate_bounds(m: &Model) -> Result<(), String> {
    let validate = |clip, time| -> Result<(), String> {
        let bones = m.bones(clip, time, false);
        if bones.iter().any(|b| {
            b.0.iter()
                .any(|row| row[3].abs() > 64. || row[..3].iter().any(|v| v.abs() > 8.))
        }) {
            return Err("Animated bone hierarchy exceeds PSX basis/translation limits. Apply armature scale before export.".into());
        }
        // Leave rounding margin for fixed-point hierarchy composition on the target.
        if m.points(clip, time, false)
            .iter()
            .flatten()
            .any(|v| !v.is_finite() || v.abs() > 7.5)
        {
            return Err("Animated model exceeds +/-7.5 meters in local space. Resize/recenter it before export.".into());
        }
        Ok(())
    };
    validate(None, 0.)?;
    for (id, c) in &m.clips {
        for f in 0..c.frames {
            validate(Some(*id), f as f32 / 30. + 0.00001)?;
        }
    }
    Ok(())
}

pub fn header(e: &Actor, id: usize) -> Result<String, String> {
    let c = e.skeletal_mesh.as_ref().ok_or("No skeletal component")?;
    let m = c.model.as_ref().ok_or_else(|| {
        c.error
            .clone()
            .unwrap_or("Skeletal asset unresolved".into())
    })?;
    validate_bounds(m)?;
    let (center, extent) = animated_bounds(m);
    let baked = m.mesh.animation_storage == AnimationStorage::BakedVertices;
    let rigid = (!baked).then(|| rigid_order(m));
    let base = if baked {
        q12(&m.points(None, 0., false))
    } else {
        rigid
            .as_ref()
            .unwrap()
            .vertices
            .iter()
            .map(|v| v.position)
            .collect()
    };
    let triangles = if baked {
        &m.mesh.triangles
    } else {
        &rigid.as_ref().unwrap().triangles
    };
    let mut s = String::new();
    writeln!(
        s,
        "inline constexpr int16_t skin_vertices_{id}[{}][3]={{{}}};",
        base.len(),
        base.iter()
            .map(|v| format!("{{{}}}", array(v)))
            .collect::<Vec<_>>()
            .join(",")
    )
    .unwrap();
    if let Some(rigid) = &rigid {
        writeln!(
            s,
            "inline constexpr uint8_t skin_weights_{id}[]={{{}}};",
            rigid
                .vertices
                .iter()
                .map(|v| v.bone.to_string())
                .collect::<Vec<_>>()
                .join(",")
        )
        .unwrap();
        writeln!(
            s,
            "inline constexpr uint16_t skin_bone_vertices_{id}[]={{{}}};",
            array(&rigid.bone_offsets)
        )
        .unwrap();
    }
    writeln!(
        s,
        "inline constexpr MeshQuad skin_faces_{id}[]={{{}}};",
        triangles
            .iter()
            .map(|t| {
                let mat = &m.materials[t.material as usize];
                let [a, b, c] = t.indices;
                format!(
                    "{{{{{a},{b},{c},{c}}},0,{{0,0,0}},{},0}}",
                    crate::texture::material_cpp(mat)
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
    .unwrap();
    writeln!(s,"inline constexpr MeshGeometry skin_geometry_{id}={{skin_vertices_{id},{},skin_faces_{id},{},true,{{0,0,0}},{{{}}},{{{}}}}};",base.len(),triangles.len(),array(&center),array(&extent)).unwrap();

    if baked {
        let mut total = 0_usize;
        for (clip_index, (clip_id, clip)) in m.clips.iter().enumerate() {
            let frames = (0..clip.frames)
                .map(|frame| q12(&m.points(Some(*clip_id), frame as f32 / 30. + 0.00001, false)))
                .collect::<Vec<_>>();
            let encoded = encode_vertex_clip(&base, &frames);
            total +=
                encoded.data.len() + encoded.frames.len() * std::mem::size_of::<EncodedFrame>();
            if total > MAX_CLIP_BYTES {
                return Err(format!(
                    "Baked vertex animation exceeds the 512 KiB PSX budget after clip '{}'. Use Rigid GTE or shorten/reduce the model.",
                    clip.name
                ));
            }
            writeln!(
                s,
                "inline constexpr VertexFrame skin_vertex_frames_{id}_{clip_index}[]={{{}}};",
                encoded
                    .frames
                    .iter()
                    .map(|f| format!("{{{},{}}}", f.offset, f.raw))
                    .collect::<Vec<_>>()
                    .join(",")
            )
            .unwrap();
            writeln!(
                s,
                "inline constexpr uint8_t skin_vertex_data_{id}_{clip_index}[]={{{}}};",
                array(&encoded.data)
            )
            .unwrap();
        }
    } else {
        writeln!(
            s,
            "inline constexpr Bone skin_bones_{id}[]={{{}}};",
            m.skeleton
                .bones
                .iter()
                .map(|b| format!("{{{},{}}}", b.parent, pose(&b.bind)))
                .collect::<Vec<_>>()
                .join(",")
        )
        .unwrap();
        for (clip, (_, c)) in m.clips.iter().enumerate() {
            for (track, t) in c.tracks.iter().enumerate() {
                writeln!(
                    s,
                    "inline constexpr BonePose skin_pose_{id}_{clip}_{track}[]={{{}}};",
                    t.iter().map(pose).collect::<Vec<_>>().join(",")
                )
                .unwrap();
            }
            writeln!(
                s,
                "inline constexpr BoneTrack skin_tracks_{id}_{clip}[]={{{}}};",
                c.tracks
                    .iter()
                    .enumerate()
                    .map(|(t, p)| format!("{{skin_pose_{id}_{clip}_{t},{}}}", p.len() == 1))
                    .collect::<Vec<_>>()
                    .join(",")
            )
            .unwrap();
        }
    }
    if !m.clips.is_empty() {
        writeln!(
            s,
            "inline constexpr AnimationClip skin_clips_{id}[]={{{}}};",
            m.clips
                .iter()
                .enumerate()
                .map(|(i, (_, c))| if baked {
                    format!("{{nullptr,skin_vertex_frames_{id}_{i},skin_vertex_data_{id}_{i},{},\"{}\"}}",c.frames,name(&c.name))
                } else {
                    format!("{{skin_tracks_{id}_{i},nullptr,nullptr,{},\"{}\"}}",c.frames,name(&c.name))
                })
                .collect::<Vec<_>>()
                .join(",")
        )
        .unwrap();
    }
    let clips = if m.clips.is_empty() {
        "nullptr".into()
    } else {
        format!("skin_clips_{id}")
    };
    if baked {
        writeln!(s,"inline constexpr SkeletalMesh skin_{id}={{&skin_geometry_{id},nullptr,nullptr,nullptr,0,{clips},{},SkeletalStorage::BakedVertices}};",m.clips.len()).unwrap();
    } else {
        // Dynamic lit triangles need posed model-space normals and keep the
        // compatible CPU path. Imported materials are unlit by default.
        let mode = if m.materials.iter().all(|material| material.unlit) {
            "RigidGte"
        } else {
            "CpuRigid"
        };
        writeln!(s,"inline constexpr SkeletalMesh skin_{id}={{&skin_geometry_{id},skin_weights_{id},skin_bone_vertices_{id},skin_bones_{id},{},{clips},{},SkeletalStorage::{mode}}};",m.skeleton.bones.len(),m.clips.len()).unwrap();
    }
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::encode_vertex_clip;

    #[test]
    fn baked_vertex_frames_are_independent_compact_and_bounded() {
        let base = vec![[100, -200, 300], [32000, -32000, 0]];
        let frames = vec![base.clone(), vec![[116, -232, 308], [-30000, 30000, 4096]]];
        let encoded = encode_vertex_clip(&base, &frames);
        assert_eq!(encoded.frames.len(), 2);
        assert!(!encoded.frames[0].raw);
        assert!(encoded.data.len() < frames.len() * frames[0].len() * 3 * 2);
        for (frame, expected) in encoded.frames.iter().zip(frames) {
            let mut cursor = frame.offset as usize;
            for (bind, expected) in base.iter().zip(expected) {
                for c in 0..3 {
                    let actual = if frame.raw {
                        let value =
                            i16::from_le_bytes([encoded.data[cursor], encoded.data[cursor + 1]]);
                        cursor += 2;
                        value
                    } else {
                        let delta = encoded.data[cursor] as i8;
                        cursor += 1;
                        if delta == -128 {
                            let value = i16::from_le_bytes([
                                encoded.data[cursor],
                                encoded.data[cursor + 1],
                            ]);
                            cursor += 2;
                            value
                        } else {
                            (i32::from(bind[c]) + i32::from(delta) * 16) as i16
                        }
                    };
                    assert!((i32::from(actual) - i32::from(expected[c])).abs() <= 8);
                }
            }
        }
    }
}
