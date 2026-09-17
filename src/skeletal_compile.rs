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
    }
    // Bone query sidecars share the existing quantized hierarchy/tracks. The
    // baked renderer still consumes only vertex frames; gameplay bone queries
    // never attempt to reconstruct a pose from skinned positions.
    {
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
    portable_to_cooked: Vec<u16>,
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
            uv: t.uv,
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
        portable_to_cooked: remap,
    }
}

#[derive(Clone, Debug, PartialEq)]
struct EncodedFrame {
    offset: u32,
    raw: bool,
    seek: Vec<u32>,
}
#[derive(Clone, Debug, PartialEq)]
struct EncodedVertexClip {
    frames: Vec<EncodedFrame>,
    data: Vec<u8>,
}

/// Independent frames support random clip changes and shared scratch. A compact
/// coordinate is a signed Q8 delta from bind pose; 0x80 escapes to an absolute
/// little-endian Q12 i16. If escapes make a frame larger, store raw i16s.
fn encode_vertex_clip(
    base: &[[i16; 3]],
    frames: &[Vec<[i16; 3]>],
    indexed_queries: bool,
) -> EncodedVertexClip {
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
            known.insert(key.clone(), offset);
            offset
        };
        let mut seek = vec![];
        if indexed_queries && !raw_frame {
            let mut cursor = 0usize;
            for vertex in 0..base.len() {
                if vertex % 16 == 0 {
                    seek.push(offset + cursor as u32);
                }
                for _ in 0..3 {
                    let delta = key.1[cursor] as i8;
                    cursor += 1;
                    if delta == -128 {
                        cursor += 2;
                    }
                }
            }
        }
        output.frames.push(EncodedFrame {
            offset,
            raw: raw_frame,
            seek,
        });
    }
    output
}

// ---------------------------------------------------------------------------
// Explicit target-layout accounting.
//
// Sizes below are the MIPS 32-bit ABI layout of the runtime structs: 4-byte
// pointers, 4-byte `int`/`size_t`/scoped-enum, natural alignment and trailing
// padding to the struct's own alignment. They are written out instead of being
// derived from host `size_of`, whose pointers are 8 bytes. `runtime/skeletal.hpp`
// carries a matching `static_assert` for each one, so the PSX build fails if a
// runtime struct changes shape.
// ---------------------------------------------------------------------------

/// Host asset accounting for one stored pose (quantized translation, rotation
/// and scale); identical to the target `BonePose`.
pub const HOST_POSE_BYTES: usize = 20;
/// `BonePose { int16_t translation[3], rotation[4], scale[3]; }` — ten `int16_t`.
pub const BONE_POSE_BYTES: usize = 20;
/// `Bone { int16_t parent; BonePose bind; }` — alignment 2, so no tail padding.
pub const BONE_BYTES: usize = 22;
/// `BoneTrack { const BonePose* poses; bool constant; }` — 4 + 1 padded to 8.
pub const BONE_TRACK_BYTES: usize = 8;
/// `VertexFrame { uint32_t offset; const uint32_t* seek; bool raw; }` — 8 + 1 padded to 12.
pub const VERTEX_FRAME_BYTES: usize = 12;
/// `AnimationClip { const BoneTrack*; const VertexFrame*; const uint8_t*;
/// uint16_t frames; const char* name; }` — 12 + 2 + 2 padding + 4.
pub const ANIMATION_CLIP_BYTES: usize = 20;
/// `Material { uint8_t color[3]; bool unlit; int texture; BlendMode blend;
/// int16_t depth_bias; int32_t uv_scroll[2]; }`. Assumed layout: 3 + 1 colour
/// bytes, `int` texture at 4, the scoped enum `BlendMode` as a 4-byte `int` at
/// 8, `depth_bias` at 12 with 2 bytes of padding, and `uv_scroll` at 16.
pub const MATERIAL_BYTES: usize = 24;
/// `MeshQuad { uint16_t indices[4]; uint8_t face; int16_t normal[3]; Material
/// material; uint32_t color_offset; int16_t uv[4][2]; bool packed_uv;
/// uint16_t uvw[4]; }`. Assumed layout: indices at 0, `face` at 8 with one
/// byte of padding, `normal` at 10, `Material` at 16 (alignment 4),
/// `color_offset` at 40, the per-corner coordinates at 44, `packed_uv` at 60
/// with one byte of padding, `uvw` at 62, padded to 72 for alignment 4.
pub const MESH_QUAD_BYTES: usize = 72;
/// `MeshGeometry` — four words of pointers/counts, `editable` padded to a word,
/// three `int32_t[3]` boxes, two pointers, `stream_page` and two `uint16_t`.
pub const MESH_GEOMETRY_BYTES: usize = 72;
/// `SkeletalMesh` — eight words plus the `SkeletalStorage` byte padded to a word.
pub const SKELETAL_MESH_BYTES: usize = 36;
/// `Animator` — `enabled` padded to a word, model pointer, clip, ticks, and the
/// `playing`/`looping` pair padded to a word.
pub const ANIMATOR_BYTES: usize = 20;
/// One stored target position: `int16_t[3]`.
pub const VERTEX_BYTES: usize = 6;
/// One `Affine<Fixed>`: `Fixed values[3][4]`, `Fixed` being a 32-bit Q12 word.
pub const AFFINE_BYTES: usize = 48;
/// `skeletal_detail::Scratch` — the shared 64-matrix pose buffer, the shared
/// 512-position decode buffer and one `MeshGeometry` copy. Allocated once for
/// the whole executable, not per character.
pub const SCRATCH_BYTES: usize = crate::skeletal::MAX_BONES * AFFINE_BYTES
    + crate::skeletal::MAX_VERTICES * VERTEX_BYTES
    + MESH_GEOMETRY_BYTES;

/// Immutable query tables retained in addition to the selected render format.
/// These flags are computed from the shared operation demand set before scene
/// banks are emitted, so a project that does not query poses keeps no sidecar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct QueryDemand {
    pub vertices: bool,
    pub bones: bool,
}
impl QueryDemand {
    pub const NONE: Self = Self {
        vertices: false,
        bones: false,
    };
    pub const ALL: Self = Self {
        vertices: true,
        bones: true,
    };
}

const _: () = {
    // `Material` starts at offset 16 in `MeshQuad`; after it come the colour
    // offset word, the four Q12 coordinate pairs, the packed flag with one byte
    // of padding, the four packed corners and two bytes of tail padding.
    assert!(MESH_QUAD_BYTES == 16 + MATERIAL_BYTES + 4 + 16 + 2 + 8 + 2);
};

/// Explicit byte accounting for one compiled model, in the target's layout.
/// Every field is produced with checked arithmetic so a malformed or oversized
/// model reports a breakdown instead of wrapping.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Budget {
    /// Host asset bytes for the stored pose tracks (20 per stored pose).
    pub host_track_bytes: usize,
    /// Rigid-mode target pose payload.
    pub rigid_pose_bytes: usize,
    /// Rigid-mode bone metadata plus track and clip descriptors.
    pub rigid_descriptor_bytes: usize,
    /// Baked-mode frame payload after identical frames are shared.
    pub baked_frame_bytes: usize,
    /// Baked-mode frame and clip descriptors.
    pub baked_descriptor_bytes: usize,
    /// Shared immutable geometry: positions, faces (texture coordinates
    /// included), bone offsets and bone indices, and the two table headers.
    pub geometry_bytes: usize,
    /// Per placed character, not per model.
    pub animator_bytes: usize,
    /// Shared between all characters, counted once per executable.
    pub scratch_bytes: usize,
}
impl Budget {
    /// The part measured against the per-model animation limit.
    pub fn target_animation_bytes(&self) -> usize {
        self.rigid_pose_bytes
            + self.rigid_descriptor_bytes
            + self.baked_frame_bytes
            + self.baked_descriptor_bytes
    }
    /// Single-line breakdown, reused by the generated comment and by errors.
    pub fn breakdown(&self) -> String {
        format!(
            "host pose tracks {} B, target bone poses {} B, bone/track/clip descriptors {} B, baked frame payload {} B, baked frame descriptors {} B, geometry {} B, animator {} B per character, shared scratch {} B; animation total {} B of {} B",
            self.host_track_bytes,
            self.rigid_pose_bytes,
            self.rigid_descriptor_bytes,
            self.baked_frame_bytes,
            self.baked_descriptor_bytes,
            self.geometry_bytes,
            self.animator_bytes,
            self.scratch_bytes,
            self.target_animation_bytes(),
            MAX_CLIP_BYTES
        )
    }
    fn overflow(&self, what: &str) -> String {
        format!(
            "{what} exceeds the {MAX_CLIP_BYTES} byte animation budget: {}. Remove unused clips, shorten clips, switch the animation storage mode, or reduce the model's vertex and triangle counts.",
            self.breakdown()
        )
    }
}

fn overflowed() -> String {
    "Skeletal model size overflows the byte counters; reduce its clips and geometry".to_string()
}
fn checked(value: Option<usize>) -> Result<usize, String> {
    value.ok_or_else(overflowed)
}

/// Byte accounting for a model in its selected storage mode. Fails when the
/// host tracks or the generated target tables exceed the animation budget.
pub fn budget(m: &Model) -> Result<Budget, String> {
    budget_for(m, QueryDemand::NONE)
}

pub fn budget_for(m: &Model, demand: QueryDemand) -> Result<Budget, String> {
    measure(m, demand).map(|(budget, _)| budget)
}

fn measure(m: &Model, demand: QueryDemand) -> Result<(Budget, Vec<EncodedVertexClip>), String> {
    let baked = m.mesh.animation_storage == AnimationStorage::BakedVertices;
    let bones = m.skeleton.bones.len();
    let clips = m.clips.len();
    let poses = checked(m.clips.iter().try_fold(0_usize, |total, (_, c)| {
        total.checked_add(
            c.tracks
                .iter()
                .map(Vec::len)
                .try_fold(0_usize, |a, n| a.checked_add(n))?,
        )
    }))?;
    let mut budget = Budget {
        host_track_bytes: checked(poses.checked_mul(HOST_POSE_BYTES))?,
        animator_bytes: ANIMATOR_BYTES,
        scratch_bytes: SCRATCH_BYTES,
        ..Default::default()
    };
    let mut geometry = checked(m.mesh.vertices.len().checked_mul(VERTEX_BYTES))?;
    geometry = checked(
        m.mesh
            .triangles
            .len()
            .checked_mul(MESH_QUAD_BYTES)
            .and_then(|v| v.checked_add(geometry)),
    )?;
    geometry = checked(geometry.checked_add(MESH_GEOMETRY_BYTES + SKELETAL_MESH_BYTES))?;
    if !baked {
        // One bone index and one portable-to-cooked uint16 per vertex, plus
        // the per-bone range table (bones + 1).
        geometry = checked(geometry.checked_add(m.mesh.vertices.len()))?;
        if demand.vertices {
            geometry = checked(
                m.mesh
                    .vertices
                    .len()
                    .checked_mul(2)
                    .and_then(|v| v.checked_add(geometry)),
            )?;
        }
        geometry = checked(
            bones
                .checked_add(1)
                .and_then(|v| v.checked_mul(2))
                .and_then(|v| v.checked_add(geometry)),
        )?;
    }
    budget.geometry_bytes = geometry;
    let clip_descriptors = checked(clips.checked_mul(ANIMATION_CLIP_BYTES))?;
    if budget.host_track_bytes > MAX_CLIP_BYTES {
        return Err(budget.overflow("Combined model clips"));
    }
    if !baked {
        budget.rigid_pose_bytes = checked(poses.checked_mul(BONE_POSE_BYTES))?;
        let tracks = checked(
            clips
                .checked_mul(bones)
                .and_then(|v| v.checked_mul(BONE_TRACK_BYTES)),
        )?;
        let metadata = checked(bones.checked_mul(BONE_BYTES))?;
        budget.rigid_descriptor_bytes = checked(
            tracks
                .checked_add(metadata)
                .and_then(|v| v.checked_add(clip_descriptors)),
        )?;
        if budget.target_animation_bytes() > MAX_CLIP_BYTES {
            return Err(budget.overflow("Rigid bone animation"));
        }
        return Ok((budget, vec![]));
    }
    // Baked rendering always keeps its vertex payload. Quantized hierarchy and
    // tracks are a gameplay-query sidecar and are retained only when the
    // operation/native-use demand set requests bone sampling.
    budget.rigid_pose_bytes = if demand.bones {
        checked(poses.checked_mul(BONE_POSE_BYTES))?
    } else {
        0
    };
    let query_tracks = if demand.bones {
        checked(
            clips
                .checked_mul(bones)
                .and_then(|v| v.checked_mul(BONE_TRACK_BYTES)),
        )?
    } else {
        0
    };
    let query_bones = if demand.bones {
        checked(bones.checked_mul(BONE_BYTES))?
    } else {
        0
    };
    budget.baked_descriptor_bytes = checked(
        clip_descriptors
            .checked_add(query_tracks)
            .and_then(|v| v.checked_add(query_bones)),
    )?;
    let base = q12(&m.points(None, 0., false));
    let mut encoded = Vec::with_capacity(clips);
    for (clip_id, clip) in &m.clips {
        let frames = (0..clip.frames)
            .map(|frame| q12(&m.points(Some(*clip_id), frame as f32 / 30. + 0.00001, false)))
            .collect::<Vec<_>>();
        let clip_encoded = encode_vertex_clip(&base, &frames, demand.vertices);
        budget.baked_frame_bytes = checked(
            budget
                .baked_frame_bytes
                .checked_add(clip_encoded.data.len()),
        )?;
        budget.baked_descriptor_bytes = checked(
            clip_encoded
                .frames
                .len()
                .checked_mul(VERTEX_FRAME_BYTES)
                .and_then(|v| v.checked_add(budget.baked_descriptor_bytes)),
        )?;
        budget.baked_descriptor_bytes = checked(
            clip_encoded
                .frames
                .iter()
                .try_fold(0usize, |total, frame| {
                    frame
                        .seek
                        .len()
                        .checked_mul(4)
                        .and_then(|bytes| total.checked_add(bytes))
                })
                .and_then(|v| v.checked_add(budget.baked_descriptor_bytes)),
        )?;
        encoded.push(clip_encoded);
        // Stop before decoding further clips once the budget is already gone.
        if budget.target_animation_bytes() > MAX_CLIP_BYTES {
            return Err(budget.overflow(&format!("Baked vertex animation at clip '{}'", clip.name)));
        }
    }
    Ok((budget, encoded))
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

#[cfg(test)]
pub fn header(e: &Actor, id: usize) -> Result<String, String> {
    header_with_pages_for(e, id, &|_| None, QueryDemand::NONE)
}
pub fn header_with_pages(
    e: &Actor,
    id: usize,
    pages: crate::mesh_compile::PageLookup,
) -> Result<String, String> {
    header_with_pages_for(e, id, pages, QueryDemand::NONE)
}
pub fn header_with_pages_for(
    e: &Actor,
    id: usize,
    pages: crate::mesh_compile::PageLookup,
    demand: QueryDemand,
) -> Result<String, String> {
    let c = e.skeletal_mesh.as_ref().ok_or("No skeletal component")?;
    let m = c.model.as_ref().ok_or_else(|| {
        c.error
            .clone()
            .unwrap_or("Skeletal asset unresolved".into())
    })?;
    validate_bounds(m)?;
    let (budget, encoded_clips) = measure(m, demand)?;
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
    writeln!(s, "// skeletal budget: {}", budget.breakdown()).unwrap();
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
        if demand.vertices {
            writeln!(
                s,
                "inline constexpr uint16_t skin_portable_to_cooked_{id}[]={{{}}};",
                array(&rigid.portable_to_cooked)
            )
            .unwrap();
        }
    }
    writeln!(
        s,
        "inline constexpr MeshQuad skin_faces_{id}[]={{{}}};",
        triangles
            .iter()
            .map(|t| {
                let mat = &m.materials[t.material as usize];
                let [a, b, c] = t.indices;
                let uv = crate::skeletal::corner_uv(t);
                let packed =
                    mat.texture
                        .and_then(pages)
                        .map_or("false,{0,0,0,0}".to_string(), |page| {
                            format!(
                                "true,{{{}}}",
                                uv.iter()
                                    .map(|uv| crate::mesh_compile::packed_uv(*uv, page).to_string())
                                    .collect::<Vec<_>>()
                                    .join(",")
                            )
                        });
                format!(
                    "{{{{{a},{b},{c},{c}}},0,{{0,0,0}},{},0,{},{}}}",
                    crate::texture::material_cpp(mat),
                    crate::texture::uv_cpp(uv),
                    packed
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
    .unwrap();
    writeln!(s,"inline constexpr MeshGeometry skin_geometry_{id}={{skin_vertices_{id},{},skin_faces_{id},{},true,{{0,0,0}},{{{}}},{{{}}}}};",base.len(),triangles.len(),array(&center),array(&extent)).unwrap();

    // Rigid rendering consumes the hierarchy and tracks directly. Baked
    // rendering retains them only when a reachable bone query demands them.
    if !baked || demand.bones {
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
            for (track, values) in c.tracks.iter().enumerate() {
                writeln!(
                    s,
                    "inline constexpr BonePose skin_pose_{id}_{clip}_{track}[]={{{}}};",
                    values.iter().map(pose).collect::<Vec<_>>().join(",")
                )
                .unwrap();
            }
            writeln!(
                s,
                "inline constexpr BoneTrack skin_tracks_{id}_{clip}[]={{{}}};",
                c.tracks
                    .iter()
                    .enumerate()
                    .map(|(track, values)| format!(
                        "{{skin_pose_{id}_{clip}_{track},{}}}",
                        values.len() == 1
                    ))
                    .collect::<Vec<_>>()
                    .join(",")
            )
            .unwrap();
        }
    }

    if baked {
        for (clip_index, encoded) in encoded_clips.iter().enumerate() {
            for (frame_index, frame) in encoded.frames.iter().enumerate() {
                if !frame.seek.is_empty() {
                    writeln!(
                        s,
                        "inline constexpr uint32_t skin_vertex_seek_{id}_{clip_index}_{frame_index}[]={{{}}};",
                        array(&frame.seek)
                    )
                    .unwrap();
                }
            }
            writeln!(
                s,
                "inline constexpr VertexFrame skin_vertex_frames_{id}_{clip_index}[]={{{}}};",
                encoded
                    .frames
                    .iter()
                    .enumerate()
                    .map(|(frame_index, f)| format!(
                        "{{{},{},{}}}",
                        f.offset,
                        if f.seek.is_empty() {
                            "nullptr".to_string()
                        } else {
                            format!("skin_vertex_seek_{id}_{clip_index}_{frame_index}")
                        },
                        f.raw
                    ))
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
    }
    if !m.clips.is_empty() {
        writeln!(
            s,
            "inline constexpr AnimationClip skin_clips_{id}[]={{{}}};",
            m.clips
                .iter()
                .enumerate()
                .map(|(i, (_, c))| if baked {
                    let tracks=if demand.bones {format!("skin_tracks_{id}_{i}")} else {"nullptr".into()};
                    format!("{{{tracks},skin_vertex_frames_{id}_{i},skin_vertex_data_{id}_{i},{},\"{}\"}}",c.frames,name(&c.name))
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
        let bones = if demand.bones {
            format!("skin_bones_{id}")
        } else {
            "nullptr".into()
        };
        let bone_count = if demand.bones {
            m.skeleton.bones.len()
        } else {
            0
        };
        writeln!(s,"inline constexpr SkeletalMesh skin_{id}={{&skin_geometry_{id},nullptr,nullptr,nullptr,{bones},{bone_count},{clips},{},SkeletalStorage::BakedVertices}};",m.clips.len()).unwrap();
    } else {
        // Dynamic lit triangles need posed model-space normals and keep the
        // compatible CPU path. Imported materials are unlit by default.
        let mode = if m.materials.iter().all(|material| material.unlit) {
            "RigidGte"
        } else {
            "CpuRigid"
        };
        let remap = if demand.vertices {
            format!("skin_portable_to_cooked_{id}")
        } else {
            "nullptr".into()
        };
        writeln!(s,"inline constexpr SkeletalMesh skin_{id}={{&skin_geometry_{id},skin_weights_{id},skin_bone_vertices_{id},{remap},skin_bones_{id},{},{clips},{},SkeletalStorage::{mode}}};",m.skeleton.bones.len(),m.clips.len()).unwrap();
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
        let encoded = encode_vertex_clip(&base, &frames, true);
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
