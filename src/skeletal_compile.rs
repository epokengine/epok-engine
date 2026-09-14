//! Compile portable UUID-linked assets into immutable PSX tables.
use crate::{
    scene::Actor,
    skeletal::{Model, Pose},
};
use std::fmt::Write;
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
    let mut s = String::new();
    writeln!(
        s,
        "inline constexpr int16_t skin_vertices_{id}[{}][3]={{{}}};",
        m.mesh.vertices.len(),
        m.mesh
            .vertices
            .iter()
            .map(|v| format!("{{{}}}", array(&v.position)))
            .collect::<Vec<_>>()
            .join(",")
    )
    .unwrap();
    writeln!(
        s,
        "inline constexpr uint8_t skin_weights_{id}[]={{{}}};",
        m.mesh
            .vertices
            .iter()
            .map(|v| v.bone.to_string())
            .collect::<Vec<_>>()
            .join(",")
    )
    .unwrap();
    writeln!(
        s,
        "inline constexpr MeshQuad skin_faces_{id}[]={{{}}};",
        m.mesh
            .triangles
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
    writeln!(s,"inline constexpr MeshGeometry skin_geometry_{id}={{skin_vertices_{id},{},skin_faces_{id},{},true}};",m.mesh.vertices.len(),m.mesh.triangles.len()).unwrap();
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
    if !m.clips.is_empty() {
        writeln!(
            s,
            "inline constexpr AnimationClip skin_clips_{id}[]={{{}}};",
            m.clips
                .iter()
                .enumerate()
                .map(|(i, (_, c))| format!(
                    "{{skin_tracks_{id}_{i},{},\"{}\"}}",
                    c.frames,
                    c.name
                        .bytes()
                        .map(|b| format!("\\{b:03o}"))
                        .collect::<String>()
                ))
                .collect::<Vec<_>>()
                .join(",")
        )
        .unwrap();
    }
    writeln!(s,"inline constexpr SkeletalMesh skin_{id}={{&skin_geometry_{id},skin_weights_{id},skin_bones_{id},{},{},{}}};",m.skeleton.bones.len(),if m.clips.is_empty(){"nullptr".into()}else{format!("skin_clips_{id}")},m.clips.len()).unwrap();
    Ok(s)
}
