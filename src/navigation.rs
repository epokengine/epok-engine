//! NavLite bakes collision geometry on the PC. PSX receives only a bounded graph.
use crate::{
    collision::{Aabb, Collider, world_bounds},
    scene::Scene,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
pub const VOLUME: &str = "e56c5741-4b0f-4861-a732-e91430c72a01";
pub const AGENT: &str = "e56c5741-4b0f-4861-a732-e91430c72a02";
const EPS: f32 = 1. / 256.;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bake {
    pub fingerprint: String,
    pub nodes: Vec<Node>,
    pub spacing: f32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node {
    pub position: [i16; 3],
    pub links: [u16; 4],
}
#[derive(Clone, Debug, Serialize)]
struct Profile {
    spacing: f32,
    radius: f32,
    height: f32,
    mask: u32,
}
fn component<'a>(
    a: &'a crate::scene::Actor,
    id: &str,
) -> Option<&'a crate::actor_document::ComponentInstance> {
    a.components
        .iter()
        .find(|c| c.class.class_id.as_deref() == Some(id))
}
fn inputs(scene: &Scene) -> Result<(Vec<Aabb>, Profile, Vec<(Aabb, Collider)>), String> {
    let mut volumes = Vec::new();
    let mut profile = None;
    for (i, a) in scene
        .actors
        .iter()
        .enumerate()
        .filter(|(i, _)| scene.is_active(*i))
    {
        if let Some(c) = component(a, VOLUME) {
            let number = |key: &str, default: f32| -> Result<f32, String> {
                c.properties.get(key).map_or(Ok(default), |v| {
                    v.as_f64()
                        .map(|v| v as f32)
                        .ok_or_else(|| format!("Navigation: invalid {key}"))
                })
            };
            let p = Profile {
                spacing: number("spacing", 0.5)?,
                radius: number("radius", 0.18)?,
                height: number("height", 0.5)?,
                mask: c
                    .properties
                    .get("collision_mask")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1) as u32,
            };
            if ![p.spacing, p.radius, p.height]
                .iter()
                .all(|v| v.is_finite())
                || !(0.125..=4.).contains(&p.spacing)
                || !(EPS..=2.).contains(&p.radius)
                || !(0.125..=8.).contains(&p.height)
            {
                return Err(
                    "Navigation profile: spacing 0.125–4, radius 1/256–2, height 0.125–8".into(),
                );
            }
            if let Some(old) = &profile {
                if serde_json::to_value(old).unwrap() != serde_json::to_value(&p).unwrap() {
                    return Err(
                        "NavLite volumes in one map must share the same agent profile".into(),
                    );
                }
            }
            profile = Some(p);
            let bounds = world_bounds(&Collider::default(), scene.world_matrix(i));
            if bounds
                .min
                .iter()
                .chain(&bounds.max)
                .any(|v| !v.is_finite() || v.abs() > 127.)
            {
                return Err("Navigation volume must fit Q8 coordinates ±127".into());
            }
            volumes.push(bounds);
        }
    }
    let p = profile.unwrap_or(Profile {
        spacing: 0.5,
        radius: 0.18,
        height: 0.5,
        mask: 1,
    });
    let mut solids = Vec::new();
    if volumes.is_empty() {
        return Ok((volumes, p, solids));
    }
    for (i, a) in scene.actors.iter().enumerate() {
        if !scene.is_active(i) || component(a, AGENT).is_some() {
            continue;
        }
        if let Some(c) = &a.collider {
            if c.enabled && !c.trigger && c.layer & p.mask != 0 {
                crate::collision::validate_component(c)?;
                let b = world_bounds(c, scene.world_matrix(i));
                if b.min.iter().chain(&b.max).any(|v| !v.is_finite()) {
                    return Err(format!(
                        "Navigation: '{}' has a nonfinite collider transform",
                        a.name
                    ));
                }
                if !volumes
                    .iter()
                    .any(|v| (0..3).all(|k| b.max[k] >= v.min[k] && b.min[k] <= v.max[k]))
                {
                    continue;
                }
                if c.slope_rise != 0. {
                    return Err(format!(
                        "NavLite v1: ramp '{}' is inside the volume. Use a flat navigation area; ramps are not supported yet",
                        a.name
                    ));
                }
                solids.push((b, c.clone()));
            }
        }
    }
    Ok((volumes, p, solids))
}
pub fn fingerprint(scene: &Scene) -> Result<String, String> {
    let (v, p, s) = inputs(scene)?;
    Ok(crate::scene_dependencies::hash((
        v.iter().map(|b| (b.min, b.max)).collect::<Vec<_>>(),
        p,
        s.iter().map(|(b, _)| (b.min, b.max)).collect::<Vec<_>>(),
    )))
}
pub fn bake(scene: &Scene) -> Result<Bake, String> {
    let (volumes, p, solids) = inputs(scene)?;
    let mut nodes = Vec::<Node>::new();
    let mut cells = BTreeMap::<(i32, i32), Vec<usize>>::new();
    let mut probes = 0usize;
    for v in volumes {
        let x0 = ((v.min[0] + p.radius) / p.spacing).ceil() as i32;
        let x1 = ((v.max[0] - p.radius) / p.spacing).floor() as i32;
        let z0 = ((v.min[2] + p.radius) / p.spacing).ceil() as i32;
        let z1 = ((v.max[2] - p.radius) / p.spacing).floor() as i32;
        probes +=
            ((x1 - x0 + 1).max(0) as usize) * ((z1 - z0 + 1).max(0) as usize) * solids.len().max(1);
        if probes > 131072 {
            return Err(
                "Navigation bake exceeds 131072 surface probes: shrink volume or increase spacing"
                    .into(),
            );
        }
        for x in x0..=x1 {
            for z in z0..=z1 {
                let px = (x as f32 * p.spacing * 256.).round() / 256.;
                let pz = (z as f32 * p.spacing * 256.).round() / 256.;
                for (floor, _) in &solids {
                    let y = (floor.max[1] * 256.).ceil() / 256.;
                    let pos = [px, y, pz];
                    if y < v.min[1] || y + p.height > v.max[1] || !supported(floor, pos, p.radius) {
                        continue;
                    }
                    if solids.iter().any(|(b, _)| blocked(b, pos, pos, &p)) {
                        continue;
                    }
                    let q = pos.map(|v| (v * 256.).round() as i16);
                    let cell = cells.entry((x, z)).or_default();
                    if cell.iter().any(|&i| nodes[i].position == q) {
                        continue;
                    }
                    if nodes.len() == 512 {
                        return Err(
                            "NavLite exceeds 512 nodes: increase spacing or shrink bake volumes"
                                .into(),
                        );
                    }
                    cell.push(nodes.len());
                    nodes.push(Node {
                        position: q,
                        links: [u16::MAX; 4],
                    });
                }
            }
        }
    }
    for (&(x, z), indices) in &cells {
        for &i in indices {
            for (dir, (dx, dz)) in [(1, 0), (-1, 0), (0, 1), (0, -1)].into_iter().enumerate() {
                if let Some(others) = cells.get(&(x + dx, z + dz)) {
                    let a = nodes[i].position.map(|v| v as f32 / 256.);
                    if let Some(&j) = others.iter().find(|&&j| {
                        let b = nodes[j].position.map(|v| v as f32 / 256.);
                        (a[1] - b[1]).abs() <= EPS
                            && solids.iter().any(|(s, _)| {
                                supported(s, a, p.radius)
                                    && supported(s, b, p.radius)
                                    && (s.max[1] - a[1]).abs() <= EPS
                            })
                            && !solids.iter().any(|(s, _)| blocked(s, a, b, &p))
                    }) {
                        nodes[i].links[dir] = j as u16;
                    }
                }
            }
        }
    }
    Ok(Bake {
        fingerprint: fingerprint(scene)?,
        nodes,
        spacing: p.spacing,
    })
}
fn supported(b: &Aabb, p: [f32; 3], r: f32) -> bool {
    [0, 2]
        .into_iter()
        .all(|k| p[k] - r >= b.min[k] && p[k] + r <= b.max[k])
}
fn blocked(b: &Aabb, a: [f32; 3], z: [f32; 3], p: &Profile) -> bool {
    if b.max[1] <= a[1].min(z[1]) + EPS || b.min[1] >= a[1].max(z[1]) + p.height {
        return false;
    }
    // Cardinal swept footprint: also catches walls thinner than the grid.
    [0, 2]
        .into_iter()
        .all(|k| a[k].max(z[k]) + p.radius > b.min[k] && a[k].min(z[k]) - p.radius < b.max[k])
}
pub fn cpp(scene: &Scene) -> Result<(String, String), String> {
    let bake = match &scene.navigation {
        Some(b) if b.fingerprint == fingerprint(scene)? => b.clone(),
        _ => bake(scene)?,
    };
    if bake.nodes.is_empty() {
        return Ok((String::new(), "::epok::nav::world.reset({});\n".into()));
    }
    use std::fmt::Write;
    let mut out = String::from("inline constexpr ::epok::nav::Node navigation_nodes[]={\n");
    for n in &bake.nodes {
        writeln!(
            out,
            "{{{},{},{},{{{},{},{},{}}}}},",
            n.position[0],
            n.position[1],
            n.position[2],
            n.links[0],
            n.links[1],
            n.links[2],
            n.links[3]
        )
        .unwrap();
    }
    out.push_str("};\n");
    Ok((
        out,
        format!(
            "::epok::nav::world.reset({{navigation_nodes,{}, {}}});\n",
            bake.nodes.len(),
            (bake.spacing * 256.) as u16
        ),
    ))
}
pub fn editor_bake(e: &mut crate::editor::Editor) {
    if e.playing || e.critical_busy() {
        return;
    }
    match bake(&e.scene) {
        Ok(b) => {
            let count = b.nodes.len();
            e.scene.navigation = Some(b);
            e.changed();
            e.log(format!(
                "Navigation baked: {count}/512 nodes, {} graph bytes. Save Scene to persist.",
                count * 14
            ));
        }
        Err(err) => e.log(err),
    }
}
pub fn debug_lines(scene: &Scene, selected: Option<usize>) -> Vec<([f32; 3], [f32; 3], [u8; 3])> {
    let mut out = Vec::new();
    if let Some(i) = selected.filter(|&i| {
        scene
            .actors
            .get(i)
            .is_some_and(|a| component(a, VOLUME).is_some())
    }) {
        for (a, b) in world_bounds(&Collider::default(), scene.world_matrix(i)).edges() {
            out.push((a, b, [60, 200, 240]));
        }
        if let Some(bake) = &scene.navigation {
            let color = if fingerprint(scene).ok().as_ref() == Some(&bake.fingerprint) {
                [60, 230, 130]
            } else {
                [240, 150, 30]
            };
            for (i, n) in bake.nodes.iter().enumerate() {
                for &j in &n.links {
                    if (j as usize) > i && (j as usize) < bake.nodes.len() {
                        let point = |p: [i16; 3]| {
                            let mut q = p.map(|v| v as f32 / 256.);
                            q[1] += 0.025;
                            q
                        };
                        out.push((
                            point(n.position),
                            point(bake.nodes[j as usize].position),
                            color,
                        ));
                    }
                }
            }
        }
    }
    out
}
pub fn create_volume(e: &mut crate::editor::Editor) {
    use crate::actor_document::{ClassReference, ComponentInstance};
    if e.playing || e.critical_busy() {
        return;
    }
    let mut a = crate::scene::Actor::cube("Navigation Bake Volume".into());
    a.kind = "Empty".into();
    a.position = [0., 1., 0.];
    a.scale = [8., 3., 8.];
    crate::actor_components::sync(&mut a);
    a.components.push(ComponentInstance::new(
        uuid::Uuid::new_v4(),
        ClassReference::new("epok::NavigationBakeVolumeComponent", VOLUME),
        "Navigation Bake Volume",
    ));
    e.scene.actors.push(a);
    e.selected = Some(e.scene.actors.len() - 1);
    e.changed();
}
#[cfg(test)]
mod tests {
    use super::*;
    fn scene() -> Scene {
        let mut s = Scene::default();
        s.actors.clear();
        let mut floor = crate::scene::Actor::cube("floor".into());
        floor.position = [0., -0.25, 0.];
        floor.scale = [6., 0.5, 6.];
        floor.collider = Some(Collider::default());
        s.actors.push(floor);
        let mut volume = crate::scene::Actor::cube("nav".into());
        volume.kind = "Empty".into();
        volume.position = [0., 1., 0.];
        volume.scale = [6., 3., 6.];
        volume
            .components
            .push(crate::actor_document::ComponentInstance::new(
                uuid::Uuid::new_v4(),
                crate::actor_document::ClassReference::new(
                    "epok::NavigationBakeVolumeComponent",
                    VOLUME,
                ),
                "nav",
            ));
        s.actors.push(volume);
        s
    }
    #[test]
    fn obstacle_clearance_and_thin_wall_edges() {
        let mut s = scene();
        let mut wall = crate::scene::Actor::cube("wall".into());
        wall.position = [0.25, 0.5, 0.];
        wall.scale = [0.01, 1., 6.];
        wall.collider = Some(Collider::default());
        s.actors.push(wall);
        let b = bake(&s).unwrap();
        assert!(!b.nodes.is_empty());
        for n in &b.nodes {
            for &j in &n.links {
                if j != u16::MAX {
                    let a = n.position[0] as f32 / 256.;
                    let z = b.nodes[j as usize].position[0] as f32 / 256.;
                    assert!(!(a < 0.25 && z > 0.25 || z < 0.25 && a > 0.25));
                }
            }
        }
    }
    #[test]
    fn fingerprint_and_roundtrip() {
        let mut s = scene();
        s.navigation = Some(bake(&s).unwrap());
        let old = fingerprint(&s).unwrap();
        s.actors[0].position[0] += 1.;
        assert_ne!(old, fingerprint(&s).unwrap());
        let decoded: Scene = serde_json::from_value(serde_json::to_value(&s).unwrap()).unwrap();
        assert!(decoded.navigation.is_some());
    }
    #[test]
    fn empty_and_capacity() {
        assert!(bake(&Scene::default()).unwrap().nodes.is_empty());
        let mut s = scene();
        s.actors[0].scale = [50., 0.5, 50.];
        s.actors[1].scale = [50., 3., 50.];
        assert!(bake(&s).unwrap_err().contains("512"));
    }
    #[test]
    fn low_ceiling_and_two_floors_do_not_link_vertically() {
        let mut s = scene();
        let mut upper = crate::scene::Actor::cube("upper".into());
        upper.position = [0., 1., 0.];
        upper.scale = [2., 0.2, 2.];
        upper.collider = Some(Collider::default());
        s.actors.push(upper);
        let b = bake(&s).unwrap();
        assert!(b.nodes.iter().any(|n| n.position[1] > 200));
        for n in &b.nodes {
            for &j in &n.links {
                if j != u16::MAX {
                    assert_eq!(n.position[1], b.nodes[j as usize].position[1]);
                }
            }
        }
        s.actors[2].position[1] = 0.35;
        let b = bake(&s).unwrap();
        assert!(
            !b.nodes.iter().any(|n| n.position[1] == 0
                && n.position[0].abs() < 256
                && n.position[2].abs() < 256)
        );
    }
    #[test]
    fn triggers_and_agents_are_not_baked_as_static_obstacles() {
        let mut s = scene();
        let count = bake(&s).unwrap().nodes.len();
        let mut trigger = crate::scene::Actor::cube("trigger".into());
        trigger.collider = Some(Collider {
            trigger: true,
            ..Default::default()
        });
        s.actors.push(trigger);
        assert_eq!(bake(&s).unwrap().nodes.len(), count);
        s.actors[2].collider.as_mut().unwrap().trigger = false;
        s.actors[2]
            .components
            .push(crate::actor_document::ComponentInstance::new(
                uuid::Uuid::new_v4(),
                crate::actor_document::ClassReference::new("epok::NavigationAgentComponent", AGENT),
                "agent",
            ));
        assert_eq!(bake(&s).unwrap().nodes.len(), count);
    }
}
