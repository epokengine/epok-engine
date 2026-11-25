//! Geometry and gameplay consume the same banked rings, including the closing span.
use crate::{
    lighting::{dot, sub, unit},
    mesh::cross,
    scene::Scene,
};
use std::{collections::BTreeSet, fmt::Write};

fn wrap(a: f32) -> f32 {
    (a + 180.).rem_euclid(360.) - 180.
}

/// Choose the equivalent Euler triple closest to the previous ring. A yaw passing
/// 90 degrees must not make the craft roll through 180 degrees between samples.
pub fn rotation(right: [f32; 3], up: [f32; 3], forward: [f32; 3], previous: [f32; 3]) -> [f32; 3] {
    let yaw = (-right[2]).clamp(-1., 1.).asin();
    let (x, z) = if yaw.cos().abs() > 1e-4 {
        (
            up[2].atan2(forward[2]).to_degrees(),
            right[1].atan2(right[0]).to_degrees(),
        )
    } else {
        let roll = previous[2];
        (
            (-forward[1]).atan2(up[1]).to_degrees() + yaw.signum() * roll,
            roll,
        )
    };
    let first = [x, yaw.to_degrees(), z].map(wrap);
    let other = [x + 180., 180. - first[1], z + 180.].map(wrap);
    let difference = |v: [f32; 3]| {
        (0..3)
            .map(|i| wrap(v[i] - previous[i]).powi(2))
            .sum::<f32>()
    };
    if difference(first) <= difference(other) {
        first
    } else {
        other
    }
}

pub fn cpp(scene: &Scene) -> Result<(String, String), String> {
    let mut tables = String::new();
    let mut paths = String::new();
    let mut used = BTreeSet::new();
    let q = |v: f32| (v * 4096.).round() as i32;
    let triple = |v: [i32; 3]| format!("{{{},{},{}}}", v[0], v[1], v[2]);
    for (index, actor) in scene.actors.iter().enumerate() {
        let Some(spline) = actor
            .editable_mesh
            .as_ref()
            .and_then(|m| m.document.as_ref())
            .and_then(|d| d.spline.as_ref())
        else {
            continue;
        };
        let Some(channel) = spline.gameplay_channel else {
            continue;
        };
        if !used.insert(channel) {
            return Err(format!(
                "{}: spline gameplay channel {channel} is used twice in this map",
                actor.name
            ));
        }
        let world = scene.world_matrix(index);
        let frames = spline.frames()?;
        let mut last = world.point(frames[0].center);
        let mut distance = 0.;
        let mut last_q = -1;
        let mut angles = [0.; 3];
        writeln!(
            tables,
            "constexpr ::epok::spline_path::Ring spline_rings_{channel}[]={{"
        )
        .unwrap();
        for (i, frame) in frames.iter().enumerate() {
            let center = world.point(frame.center);
            let right = world.vector(frame.right);
            let side_length = dot(right, right).sqrt();
            let forward = unit(world.vector(frame.tangent));
            let up = unit(cross(forward, right));
            let side = unit(cross(up, forward));
            // Measure lateral offsets in world units across the actual ribbon.
            let edge = unit(right);
            let height = unit(world.vector(frame.up));
            let step = sub(center, last);
            distance += dot(step, step).sqrt();
            let d = q(distance);
            if d <= last_q || side_length < 1e-6 || dot(up, up) < 0.9 {
                return Err(format!(
                    "{}: degenerate gameplay spline after applying the world transform",
                    actor.name
                ));
            }
            angles = rotation(side, up, forward, angles);
            let span = (i / usize::from(spline.steps)).min(spline.spans() - 1);
            let bend = frames.get(i + 1).map_or(0, |next| {
                let chord = sub(world.point(next.center), center);
                let change = sub(unit(world.vector(next.tangent)), forward);
                q(dot(change, side) / dot(chord, chord).sqrt().max(0.001)).clamp(-32768, 32767)
            });
            writeln!(
                tables,
                "{{{d},{},{},{},{},{},{bend},{}}},",
                triple(center.map(q)),
                q(spline.width * 0.5 * side_length),
                triple(edge.map(q)),
                triple(height.map(q)),
                triple(angles.map(|v| (v * 64.).round() as i32)),
                spline.points[span].tag
            )
            .unwrap();
            last = center;
            last_q = d;
        }
        tables.push_str("};\n");
        writeln!(
            paths,
            "{{spline_rings_{channel},{},{channel},{}}},",
            frames.len(),
            spline.closed
        )
        .unwrap();
    }
    if used.is_empty() {
        return Ok((tables, "::epok::spline_path::bind(nullptr,0);\n".into()));
    }
    writeln!(
        tables,
        "constexpr ::epok::spline_path::Path spline_paths[]={{\n{paths}}};"
    )
    .unwrap();
    Ok((
        tables,
        format!("::epok::spline_path::bind(spline_paths,{});\n", used.len()),
    ))
}
