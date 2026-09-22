use crate::{
    collision::{Aabb, Collider, world_bounds},
    navigation_geometry::{Triangle, quad},
    scene::Scene,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
pub const VOLUME: &str = "e56c5741-4b0f-4861-a732-e91430c72a01";
pub const AGENT: &str = "e56c5741-4b0f-4861-a732-e91430c72a02";
pub const SURFACE: &str = "e56c5741-4b0f-4861-a732-e91430c72a03";
pub const LINK: &str = "e56c5741-4b0f-4861-a732-e91430c72a04";
pub const OBSTACLE: &str = "e56c5741-4b0f-4861-a732-e91430c72a05";
const EPS: f32 = 1. / 256.;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bake {
    pub fingerprint: String,
    pub nodes: Vec<Node>,
    pub spacing: f32,
    #[serde(default)]
    pub surfaces: Vec<Triangle>,
    #[serde(default)]
    pub traversals: Vec<Traversal>,
    #[serde(default)]
    pub radius: f32,
    #[serde(default)]
    pub height: f32,
    #[serde(default)]
    pub step_height: f32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node {
    pub position: [i16; 3],
    pub links: Vec<u16>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Traversal {
    pub from: u16,
    pub to: u16,
    pub kind: u8,
    pub arc: f32,
    pub duration: f32,
}
#[derive(Clone, Debug, Serialize)]
struct Profile {
    spacing: f32,
    radius: f32,
    height: f32,
    step_height: f32,
    max_slope: f32,
    mask: u32,
}
#[derive(Serialize)]
struct Inputs {
    volumes: Vec<([f32; 3], [f32; 3])>,
    profile: Profile,
    solids: Vec<(Collider, [f32; 3], [f32; 3])>,
    triangles: Vec<Triangle>,
    links: Vec<LinkInput>,
}
#[derive(Serialize)]
struct LinkInput {
    start: [f32; 3],
    end: [f32; 3],
    kind: u8,
    arc: f32,
    duration: f32,
    both: bool,
}
fn component<'a>(
    a: &'a crate::scene::Actor,
    id: &str,
) -> Option<&'a crate::actor_document::ComponentInstance> {
    a.components
        .iter()
        .find(|c| c.class.class_id.as_deref() == Some(id))
}
fn number(
    c: &crate::actor_document::ComponentInstance,
    key: &str,
    default: f32,
) -> Result<f32, String> {
    c.properties
        .get(key)
        .map_or(Some(default), |v| v.as_f64().map(|v| v as f32))
        .filter(|n| n.is_finite())
        .ok_or_else(|| format!("Navigation: invalid {key}"))
}
fn inputs(scene: &Scene) -> Result<Inputs, String> {
    let mut out = Inputs {
        volumes: vec![],
        profile: Profile {
            spacing: 0.5,
            radius: 0.18,
            height: 0.5,
            step_height: 0.4,
            max_slope: 45.,
            mask: 1,
        },
        solids: vec![],
        triangles: vec![],
        links: vec![],
    };
    for (i, a) in scene
        .actors
        .iter()
        .enumerate()
        .filter(|(i, _)| scene.is_active(*i))
    {
        if let Some(c) = component(a, VOLUME) {
            let mask = c
                .properties
                .get("collision_mask")
                .map_or(Some(1), |v| v.as_u64())
                .filter(|v| *v <= u32::MAX as u64)
                .ok_or("Navigation: collision_mask must be an unsigned 32-bit integer")?
                as u32;
            let p = Profile {
                spacing: number(c, "spacing", 0.5)?,
                radius: number(c, "radius", 0.18)?,
                height: number(c, "height", 0.5)?,
                step_height: number(c, "step_height", 0.4)?,
                max_slope: number(c, "max_slope", 45.)?,
                mask,
            };
            if !(0.125..=4.).contains(&p.spacing)
                || !(EPS..=2.).contains(&p.radius)
                || !(0.125..=8.).contains(&p.height)
                || !(0.0..=2.).contains(&p.step_height)
                || !(0.0..=75.).contains(&p.max_slope)
            {
                return Err("Navigation profile: spacing 0.125–4, radius 1/256–2, height 0.125–8, step_height 0–2, max_slope 0–75 degrees".into());
            }
            if !out.volumes.is_empty()
                && serde_json::to_value(&out.profile).unwrap() != serde_json::to_value(&p).unwrap()
            {
                return Err("NavLite volumes in one map must share the same agent profile".into());
            }
            out.profile = p;
            let b = world_bounds(&Collider::default(), scene.world_matrix(i));
            if b.min
                .iter()
                .chain(&b.max)
                .any(|v| !v.is_finite() || v.abs() > 127.)
            {
                return Err("Navigation volume must fit Q8 coordinates ±127".into());
            }
            out.volumes.push((b.min, b.max));
        }
    }
    if out.volumes.is_empty() {
        return Ok(out);
    }
    for (i, a) in scene
        .actors
        .iter()
        .enumerate()
        .filter(|(i, _)| scene.is_active(*i))
    {
        if component(a, AGENT).is_some() || component(a, OBSTACLE).is_some() {
            continue;
        }
        if let Some(c) = component(a, LINK) {
            let m = scene.world_matrix(i);
            let kind = number(c, "kind", 1.)?;
            let arc = number(c, "arc_height", 1.)?;
            let duration = number(c, "duration", 1.)?;
            if ![1., 2.].contains(&kind)
                || !(0.0..=16.).contains(&arc)
                || !(0.1..=30.).contains(&duration)
            {
                return Err(format!(
                    "{}: link kind must be 1 (jump) or 2 (climb), arc_height 0–16 and duration 0.1–30",
                    a.name
                ));
            }
            out.links.push(LinkInput {
                start: m.point([0.; 3]),
                end: m.point([
                    number(c, "end_x", 0.)?,
                    number(c, "end_y", 0.)?,
                    number(c, "end_z", 2.)?,
                ]),
                kind: kind as u8,
                arc,
                duration,
                both: c
                    .properties
                    .get("bidirectional")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true),
            });
        }
        if component(a, SURFACE).is_some() {
            if a.terrain.is_some() {
                // Terrain walkability comes from the same quads the renderer
                // draws, so a merged cell contributes one wide triangle pair
                // rather than the cells it replaced. Slope filtering happens
                // downstream, exactly as for an authored surface.
                if a.terrain
                    .as_ref()
                    .and_then(|t| t.document.as_ref())
                    .is_none()
                {
                    return Err(format!(
                        "{}: Navigation Surface needs a resolved Terrain",
                        a.name
                    ));
                }
                for q in crate::lighting::quads(a) {
                    quad(
                        q.points.map(|p| scene.world_matrix(i).point(p)),
                        &mut out.triangles,
                    );
                }
            } else {
                let doc = a
                    .editable_mesh
                    .as_ref()
                    .and_then(|m| m.document.as_ref())
                    .ok_or_else(|| {
                        format!(
                            "{}: Navigation Surface needs a resolved EditableMesh",
                            a.name
                        )
                    })?;
                for f in &doc.faces {
                    quad(
                        doc.points(f).map(|p| scene.world_matrix(i).point(p)),
                        &mut out.triangles,
                    );
                }
            }
            // A terrain's own heightfield collider is a walkable surface, not
            // an obstacle, so it is exempt from the box-collider rule.
            let terrain_collision = a.terrain.as_ref().is_some_and(|t| t.collision);
            if !terrain_collision && a.collider.as_ref().is_some_and(|c| c.enabled && !c.trigger) {
                return Err(format!(
                    "{}: disable the box collider on a Navigation Surface; use separate obstacle colliders",
                    a.name
                ));
            }
        } else if let Some(c) = &a.collider {
            if c.enabled && !c.trigger && c.layer & out.profile.mask != 0 {
                crate::collision::validate_component(c)?;
                let b = world_bounds(c, scene.world_matrix(i));
                if b.min.iter().chain(&b.max).any(|v| !v.is_finite()) {
                    return Err(format!("{}: nonfinite collider", a.name));
                }
                if !out
                    .volumes
                    .iter()
                    .any(|(lo, hi)| (0..3).all(|k| b.max[k] >= lo[k] && b.min[k] <= hi[k]))
                {
                    continue;
                }
                if !c.slope_rise.is_finite()
                    || c.slope_rise < 0.
                    || c.slope_rise > b.max[1] - b.min[1] + EPS
                    || ![0, 2].contains(&c.slope_axis)
                {
                    return Err(format!("{}: invalid navigation ramp rise/axis", a.name));
                }
                out.solids.push((c.clone(), b.min, b.max));
            }
        }
    }
    if out.triangles.len() > 4096 {
        return Err("Navigation exceeds 4096 source triangles".into());
    }
    Ok(out)
}
fn surfaces(i: &Inputs) -> Vec<Triangle> {
    let mut all = i.triangles.clone();
    for (c, lo, hi) in &i.solids {
        let mut p = [
            [lo[0], hi[1], lo[2]],
            [lo[0], hi[1], hi[2]],
            [hi[0], hi[1], hi[2]],
            [hi[0], hi[1], lo[2]],
        ];
        for p in &mut p {
            p[1] = solid_top(c, *lo, *hi, p[0], p[2]);
        }
        quad(p, &mut all);
    }
    let min_normal = i.profile.max_slope.to_radians().cos();
    all.retain(|t| t.normal()[1] >= min_normal - 1e-5);
    all
}
fn solid_top(c: &Collider, lo: [f32; 3], hi: [f32; 3], x: f32, z: f32) -> f32 {
    if c.slope_rise == 0. {
        return hi[1];
    }
    let k = c.slope_axis as usize;
    let pos = [x, 0., z];
    hi[1] - c.slope_rise + c.slope_rise * ((pos[k] - lo[k]) / (hi[k] - lo[k])).clamp(0., 1.)
}
fn floor_at(s: &[Triangle], x: f32, z: f32, near: f32, reach: f32) -> Option<f32> {
    s.iter()
        .filter_map(|t| t.height(x, z))
        .filter(|y| (*y - near).abs() <= reach + EPS)
        .max_by(f32::total_cmp)
}
fn clear(i: &Inputs, p: [f32; 3], floor_slack: f32) -> bool {
    let r = i.profile.radius;
    let b = Aabb {
        min: [p[0] - r, p[1] + floor_slack + EPS, p[2] - r],
        max: [p[0] + r, p[1] + i.profile.height - EPS, p[2] + r],
    };
    if b.min[1] >= b.max[1] {
        return false;
    }
    for (c, lo, hi) in &i.solids {
        if [0, 2]
            .into_iter()
            .all(|k| b.max[k] > lo[k] + EPS && b.min[k] < hi[k] - EPS)
        {
            let top = if c.slope_rise == 0. {
                hi[1]
            } else {
                solid_top(c, *lo, *hi, b.min[0], b.min[2])
                    .max(solid_top(c, *lo, *hi, b.max[0], b.max[2]))
            };
            // Headroom is measured from the feet, including a ceiling whose
            // underside is below the permitted step/ramp clearance.
            if lo[1] > p[1] + EPS && lo[1] < b.max[1] {
                return false;
            }
            if [0, 2]
                .into_iter()
                .all(|k| p[k] > lo[k] + EPS && p[k] < hi[k] - EPS)
                && solid_top(c, *lo, *hi, p[0], p[2]) > p[1] + EPS
                && lo[1] < b.max[1]
            {
                return false;
            }
            if top > b.min[1] && lo[1] < b.max[1] {
                return false;
            }
        }
    }
    if i.triangles.iter().any(|t| {
        t.height(p[0], p[2])
            .is_some_and(|y| y > p[1] + EPS * 2. && y < b.max[1])
    }) {
        return false;
    }
    !i.triangles.iter().any(|t| t.intersects(&b))
}
fn supported(i: &Inputs, s: &[Triangle], p: [f32; 3]) -> bool {
    let r = i.profile.radius;
    let reach = i
        .profile
        .step_height
        .max(r * 2. * i.profile.max_slope.to_radians().tan());
    [-r, 0., r].into_iter().all(|dx| {
        [-r, 0., r]
            .into_iter()
            .all(|dz| floor_at(s, p[0] + dx, p[2] + dz, p[1], reach).is_some())
    })
}
fn walkable(i: &Inputs, s: &[Triangle], a: [f32; 3], b: [f32; 3]) -> bool {
    let horizontal = (a[0] - b[0]).abs() + (a[2] - b[2]).abs();
    let slope = i.profile.max_slope.to_radians().tan();
    if (a[1] - b[1]).abs() > i.profile.step_height + horizontal * slope + EPS {
        return false;
    }
    let count = (horizontal / (i.profile.radius * 0.5).min(0.0625))
        .ceil()
        .clamp(1., 512.) as usize;
    let mut previous = a[1];
    for n in 0..=count {
        let t = n as f32 / count as f32;
        let x = a[0] + (b[0] - a[0]) * t;
        let z = a[2] + (b[2] - a[2]) * t;
        let Some(y) = floor_at(
            s,
            x,
            z,
            previous,
            i.profile.step_height + horizontal / count as f32 * slope + EPS,
        ) else {
            return false;
        };
        let p = [x, y, z];
        if !supported(i, s, p) || !clear(i, p, i.profile.step_height.max(i.profile.radius * slope))
        {
            return false;
        }
        previous = y;
    }
    (previous - b[1]).abs() <= EPS * 2.
}
pub fn fingerprint(scene: &Scene) -> Result<String, String> {
    Ok(crate::scene_dependencies::hash(inputs(scene)?))
}
pub fn bake(scene: &Scene) -> Result<Bake, String> {
    let input = inputs(scene)?;
    let p = &input.profile;
    let surf = surfaces(&input);
    if surf.len() > 4096 {
        return Err("Navigation exceeds 4096 walkable triangles".into());
    }
    let mut nodes = Vec::<Node>::new();
    let mut cells = BTreeMap::<(i32, i32), Vec<usize>>::new();
    let mut probes = 0usize;
    for (lo, hi) in &input.volumes {
        let x0 = ((lo[0] + p.radius) / p.spacing).ceil() as i32;
        let x1 = ((hi[0] - p.radius) / p.spacing).floor() as i32;
        let z0 = ((lo[2] + p.radius) / p.spacing).ceil() as i32;
        let z1 = ((hi[2] - p.radius) / p.spacing).floor() as i32;
        probes +=
            ((x1 - x0 + 1).max(0) as usize) * ((z1 - z0 + 1).max(0) as usize) * surf.len().max(1);
        if probes > 4_000_000 {
            return Err(
                "Navigation exceeds 4000000 surface probes: shrink volume or increase spacing"
                    .into(),
            );
        }
        for x in x0..=x1 {
            for z in z0..=z1 {
                let px = (x as f32 * p.spacing * 256.).round() / 256.;
                let pz = (z as f32 * p.spacing * 256.).round() / 256.;
                for f in &surf {
                    let Some(y) = f.height(px, pz) else {
                        continue;
                    };
                    let y = (y * 256.).ceil() / 256.;
                    let pos = [px, y, pz];
                    if y < lo[1]
                        || y + p.height > hi[1]
                        || !supported(&input, &surf, pos)
                        || !clear(&input, pos, p.radius * p.max_slope.to_radians().tan())
                    {
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
                        links: vec![u16::MAX; 6],
                    });
                }
            }
        }
    }
    for (&(x, z), indices) in &cells {
        for &i in indices {
            for (dir, (dx, dz)) in [(1, 0), (-1, 0), (0, 1), (0, -1)].into_iter().enumerate() {
                if let Some(other) = cells.get(&(x + dx, z + dz)) {
                    let a = nodes[i].position.map(|v| v as f32 / 256.);
                    let mut candidates = other.clone();
                    candidates.sort_by_key(|&j| {
                        (nodes[j].position[1] as i32 - nodes[i].position[1] as i32).abs()
                    });
                    if let Some(j) = candidates.into_iter().find(|&j| {
                        walkable(&input, &surf, a, nodes[j].position.map(|v| v as f32 / 256.))
                    }) {
                        nodes[i].links[dir] = j as u16;
                    }
                }
            }
        }
    }
    let mut traversals = vec![];
    for l in &input.links {
        let nearest = |p: [f32; 3]| {
            nodes
                .iter()
                .enumerate()
                .map(|(i, n)| {
                    (
                        i,
                        (0..3)
                            .map(|k| (n.position[k] as f32 / 256. - p[k]).abs())
                            .sum::<f32>(),
                    )
                })
                .filter(|(_, d)| *d <= input.profile.spacing)
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .map(|(i, _)| i)
        };
        let from = nearest(l.start).ok_or("Navigation link start is off graph")?;
        let to = nearest(l.end).ok_or("Navigation link end is off graph")?;
        if from == to {
            return Err("Navigation link endpoints project onto the same node".into());
        }
        for (a, b) in [(from, to)].into_iter().chain(l.both.then_some((to, from))) {
            let start = nodes[a].position.map(|v| v as f32 / 256.);
            let end = nodes[b].position.map(|v| v as f32 / 256.);
            let distance = (0..3).map(|k| (end[k] - start[k]).abs()).sum::<f32>() + 4. * l.arc;
            let steps = (distance / (p.radius * 0.5).min(0.0625)).ceil().max(1.) as usize;
            if steps > 4096 {
                return Err("Navigation link is too long to validate".into());
            }
            for n in 1..steps {
                let t = n as f32 / steps as f32;
                let mut q = std::array::from_fn(|k| start[k] + (end[k] - start[k]) * t);
                if l.kind == 1 {
                    q[1] += 4. * l.arc * t * (1. - t);
                } else {
                    q = if t < 0.5 {
                        [start[0], start[1] + (end[1] - start[1]) * t * 2., start[2]]
                    } else {
                        [
                            start[0] + (end[0] - start[0]) * (t * 2. - 1.),
                            end[1],
                            start[2] + (end[2] - start[2]) * (t * 2. - 1.),
                        ]
                    };
                }
                if !clear(&input, q, EPS) {
                    return Err("Navigation jump/climb link intersects geometry; adjust its endpoints or arc".into());
                }
            }
            let slot = nodes[a].links[4..]
                .iter()
                .position(|v| *v == u16::MAX)
                .ok_or("Navigation node supports at most two authored traversal links")?
                + 4;
            if traversals.len() == 32 {
                return Err("Navigation exceeds 32 directed traversal links".into());
            }
            nodes[a].links[slot] = b as u16;
            traversals.push(Traversal {
                from: a as u16,
                to: b as u16,
                kind: l.kind,
                arc: l.arc,
                duration: l.duration,
            });
        }
    }
    Ok(Bake {
        fingerprint: fingerprint(scene)?,
        nodes,
        spacing: p.spacing,
        surfaces: surf,
        traversals,
        radius: p.radius,
        height: p.height,
        step_height: p.step_height,
    })
}
pub fn cpp(scene: &Scene) -> Result<(String, String), String> {
    let b = match &scene.navigation {
        Some(b) if b.fingerprint == fingerprint(scene)? => b.clone(),
        _ => bake(scene)?,
    };
    if b.nodes.is_empty() {
        return Ok((String::new(), "::epok::nav::world.reset({});\n".into()));
    }
    use std::fmt::Write;
    let mut out = String::from("inline constexpr ::epok::nav::Node navigation_nodes[]={\n");
    for n in &b.nodes {
        writeln!(
            out,
            "{{{},{},{},{{{}}}}},",
            n.position[0],
            n.position[1],
            n.position[2],
            n.links
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(",")
        )
        .unwrap();
    }
    out.push_str("};\n");
    out.push_str("inline constexpr ::epok::nav::Surface navigation_surfaces[]={\n");
    for t in &b.surfaces {
        writeln!(
            out,
            "{{{{{}}}}},",
            t.points
                .iter()
                .map(|p| format!(
                    "{{{}}}",
                    p.iter()
                        .map(|v| ((*v * 256.).round() as i32).to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                ))
                .collect::<Vec<_>>()
                .join(",")
        )
        .unwrap();
    }
    out.push_str("};\n");
    let ptr = if b.traversals.is_empty() {
        "nullptr"
    } else {
        out.push_str("inline constexpr ::epok::nav::Traversal navigation_traversals[]={\n");
        for t in &b.traversals {
            writeln!(
                out,
                "{{{},{},{},{},{}}},",
                t.from,
                t.to,
                t.kind,
                (t.arc * 256.).round() as i32,
                (t.duration * 4096.).round() as i32
            )
            .unwrap();
        }
        out.push_str("};\n");
        "navigation_traversals"
    };
    Ok((
        out,
        format!(
            "::epok::nav::world.reset({{navigation_nodes,{}, {},navigation_surfaces,{}, {ptr},{},{},{},{}}});\n",
            b.nodes.len(),
            (b.spacing * 256.).round() as u16,
            b.surfaces.len(),
            b.traversals.len(),
            (b.radius * 256.).ceil() as u16,
            (b.height * 256.).ceil() as u16,
            (b.step_height * 256.).ceil() as u16
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
                count * 18
            ));
        }
        Err(err) => e.log(err),
    }
}
/// Editor-only preview. Reuse across camera/animation redraws and never write
/// the automatically generated graph into the authored scene.
#[derive(Default)]
pub struct Preview {
    key: Option<(uuid::Uuid, String)>,
    pub lines: Vec<([f32; 3], [f32; 3], [u8; 3])>,
    pub patches: Vec<[[f32; 3]; 4]>,
    pub error: Option<String>,
    node_count: usize,
}
pub fn selected_volume(scene: &Scene, selected: Option<usize>) -> Option<usize> {
    selected.filter(|&i| {
        scene
            .actors
            .get(i)
            .is_some_and(|a| component(a, VOLUME).is_some())
            && scene.is_active(i)
    })
}
impl Preview {
    pub fn status(&self) -> Option<Result<usize, String>> {
        self.key
            .as_ref()
            .map(|_| self.error.clone().map_or(Ok(self.node_count), Err))
    }
    pub fn update(&mut self, scene: &Scene, selected: Option<usize>) {
        let Some(index) = selected_volume(scene, selected) else {
            *self = Self::default();
            return;
        };
        let signature = fingerprint(scene);
        let bounds = world_bounds(&Collider::default(), scene.world_matrix(index));
        let key = (
            scene.actors[index].id,
            signature
                .clone()
                .unwrap_or_else(|e| format!("error:{e}:{:?}:{:?}", bounds.min, bounds.max)),
        );
        if self.key.as_ref() == Some(&key) {
            return;
        }
        self.key = Some(key);
        self.lines.clear();
        self.patches.clear();
        self.error = None;
        self.node_count = 0;
        self.lines.extend(
            bounds
                .edges()
                .into_iter()
                .map(|(a, b)| (a, b, [60, 200, 240])),
        );
        let result = signature.and_then(|signature| match &scene.navigation {
            Some(b) if b.fingerprint == signature => Ok(b.clone()),
            _ => bake(scene),
        });
        let b = match result {
            Ok(b) => b,
            Err(e) => {
                self.error = Some(e);
                return;
            }
        };
        let input = match inputs(scene) {
            Ok(i) => i,
            Err(e) => {
                self.error = Some(e);
                return;
            }
        };
        let inside = |p: [f32; 3]| {
            (0..3).all(|k| p[k] >= bounds.min[k] - EPS && p[k] <= bounds.max[k] + EPS)
        };
        let lift = |mut p: [f32; 3]| {
            p[1] += 0.045;
            p
        };
        for (i, n) in b.nodes.iter().enumerate() {
            let center = n.position.map(|v| v as f32 / 256.);
            if !inside(center) {
                continue;
            }
            self.node_count += 1;
            let marker = (b.spacing * 0.09).clamp(0.065, 0.14);
            // A marker exists even when a node has no outgoing connections.
            for axis in 0..3 {
                let mut a = lift(center);
                let mut z = a;
                a[axis] -= marker;
                z[axis] += marker;
                self.lines.push((a, z, [240, 255, 180]));
            }
            for &j in &n.links {
                if let Some(other) = b.nodes.get(j as usize) {
                    let p = other.position.map(|v| v as f32 / 256.);
                    if inside(p) && (j as usize > i || !other.links.contains(&(i as u16))) {
                        self.lines.push((lift(center), lift(p), [25, 195, 85]));
                    }
                }
            }
            // Small surface-conforming tiles show sampled walkable coverage,
            // not the whole volume or the unfiltered source floor triangles.
            let half = b.spacing * 0.5;
            let tile = b.spacing / 4.;
            for x in 0..4 {
                for z in 0..4 {
                    let x0 = (center[0] - half + x as f32 * tile).max(bounds.min[0]);
                    let z0 = (center[2] - half + z as f32 * tile).max(bounds.min[2]);
                    let x1 = (center[0] - half + (x + 1) as f32 * tile).min(bounds.max[0]);
                    let z1 = (center[2] - half + (z + 1) as f32 * tile).min(bounds.max[2]);
                    if x0 >= x1 || z0 >= z1 {
                        continue;
                    }
                    let reach = input.profile.step_height
                        + b.spacing * input.profile.max_slope.to_radians().tan();
                    let sample = |x, z| -> Option<[f32; 3]> {
                        let y = floor_at(&b.surfaces, x, z, center[1], reach)?;
                        let p = [x, y, z];
                        (inside(p)
                            && supported(&input, &b.surfaces, p)
                            && clear(
                                &input,
                                p,
                                input.profile.radius * input.profile.max_slope.to_radians().tan(),
                            ))
                        .then_some(p)
                    };
                    if sample((x0 + x1) * 0.5, (z0 + z1) * 0.5).is_none() {
                        continue;
                    }
                    let points =
                        [[x0, z0], [x0, z1], [x1, z1], [x1, z0]].map(|[x, z]| sample(x, z));
                    if let [Some(a), Some(c), Some(d), Some(e)] = points {
                        // Do not draw a diagonal green sheet through a stair riser.
                        if (a[1] + d[1] - c[1] - e[1]).abs() <= EPS * 2. {
                            self.patches.push([a, c, d, e].map(lift));
                        }
                    }
                }
            }
        }
    }
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
    #[test]
    fn preview_without_saved_bake_is_cached_filtered_and_read_only() {
        let mut s = scene();
        let original = serde_json::to_value(&s).unwrap();
        let mut preview = Preview::default();
        preview.update(&s, Some(1));
        assert!(preview.error.is_none(), "{:?}", preview.error);
        assert!(!preview.patches.is_empty());
        assert!(preview.lines.len() > 12);
        assert_eq!(original, serde_json::to_value(&s).unwrap());
        let ptr = preview.patches.as_ptr();
        preview.update(&s, Some(1));
        assert_eq!(ptr, preview.patches.as_ptr());
        // Narrowing the selected volume refreshes and clips the overlay.
        s.actors[1].scale[0] = 2.;
        preview.update(&s, Some(1));
        assert!(
            preview
                .patches
                .iter()
                .flatten()
                .all(|p| p[0].abs() <= 1. + EPS)
        );
        preview.update(&s, None);
        assert!(preview.patches.is_empty() && preview.lines.is_empty());
    }
    #[test]
    fn preview_does_not_paint_obstacles_or_use_a_stale_bake() {
        let mut s = scene();
        s.navigation = Some(bake(&s).unwrap());
        let mut wall = crate::scene::Actor::cube("wall".into());
        wall.position = [0., 0.5, 0.];
        wall.scale = [1., 1., 6.];
        wall.collider = Some(Collider::default());
        s.actors.push(wall);
        let mut p = Preview::default();
        p.update(&s, Some(1));
        assert!(p.error.is_none());
        assert!(!p.patches.is_empty());
        assert!(
            p.patches
                .iter()
                .filter(|q| q[0][1] < 0.1)
                .flatten()
                .all(|v| v[0].abs() >= 0.5 + 0.18 - EPS)
        );
        // A bake/profile failure is retained for the editor, never an old green area.
        s.actors[1]
            .components
            .last_mut()
            .unwrap()
            .properties
            .insert("spacing".into(), 0.into());
        p.update(&s, Some(1));
        assert!(p.error.is_some() && p.patches.is_empty());
    }
    fn add_component<'a>(
        a: &'a mut crate::scene::Actor,
        id: &str,
    ) -> &'a mut crate::actor_document::ComponentInstance {
        a.components
            .push(crate::actor_document::ComponentInstance::new(
                uuid::Uuid::new_v4(),
                crate::actor_document::ClassReference::new("navigation", id),
                "navigation",
            ));
        a.components.last_mut().unwrap()
    }
    fn connected(b: &Bake, from: usize, to: usize) -> bool {
        let mut seen = vec![false; b.nodes.len()];
        let mut stack = vec![from];
        while let Some(i) = stack.pop() {
            if i == to {
                return true;
            }
            if seen[i] {
                continue;
            }
            seen[i] = true;
            for &j in &b.nodes[i].links {
                if (j as usize) < b.nodes.len() {
                    stack.push(j as usize);
                }
            }
        }
        false
    }
    #[test]
    fn ramp_steps_and_modular_floors() {
        let mut s = scene();
        s.actors[0].scale = [2., 2., 2.];
        s.actors[0].position = [0., 0., 0.];
        s.actors[0].collider.as_mut().unwrap().slope_rise = 1.;
        let b = bake(&s).unwrap();
        assert!(b.nodes.iter().any(|n| n.position[1] > 100));
        let lo = b
            .nodes
            .iter()
            .enumerate()
            .min_by_key(|(_, n)| n.position[1])
            .unwrap()
            .0;
        let hi = b
            .nodes
            .iter()
            .enumerate()
            .max_by_key(|(_, n)| n.position[1])
            .unwrap()
            .0;
        assert!(connected(&b, lo, hi));
        let old = fingerprint(&s).unwrap();
        s.actors[0].collider.as_mut().unwrap().slope_rise = 0.5;
        assert_ne!(old, fingerprint(&s).unwrap());
        s.actors[0].collider.as_mut().unwrap().slope_rise = 2.;
        s.actors[0].scale[0] = 0.5;
        assert!(bake(&s).unwrap().nodes.is_empty());
        let mut s = scene();
        s.actors[0].scale = [2., 0.5, 2.];
        s.actors[0].position[0] = -1.;
        let mut step = s.actors[0].clone();
        step.id = uuid::Uuid::new_v4();
        step.position = [1., 0.05, 0.];
        s.actors.push(step);
        let b = bake(&s).unwrap();
        let lo = b
            .nodes
            .iter()
            .position(|n| n.position == [-128, 0, 0])
            .unwrap();
        let hi = b
            .nodes
            .iter()
            .position(|n| n.position == [128, 77, 0])
            .unwrap();
        assert!(connected(&b, lo, hi));
        assert!(connected(&b, hi, lo));
        s.actors[2].position[1] = 0.75;
        let b = bake(&s).unwrap();
        assert!(!b.nodes.iter().enumerate().any(|(i, n)| {
            n.links.iter().any(|&j| {
                (j as usize) < b.nodes.len()
                    && b.nodes[j as usize].position[1] != b.nodes[i].position[1]
            })
        }));
    }
    #[test]
    fn mesh_terrain_and_gap() {
        let mut s = scene();
        s.actors[0].collider = None;
        s.actors[0].position = [0.; 3];
        s.actors[0].scale = [1.; 3];
        let mut d = crate::mesh::Document::default();
        let g = d.groups[0].id;
        let m = d.materials[0].id;
        d.add_face(
            [[-2., 0., -2.], [-2., 0., 2.], [2., 1., 2.], [2., 1., -2.]],
            g,
            m,
        );
        let mut mesh = crate::mesh::Component::new(uuid::Uuid::new_v4());
        mesh.document = Some(std::sync::Arc::new(d));
        s.actors[0].editable_mesh = Some(mesh);
        add_component(&mut s.actors[0], SURFACE);
        let b = bake(&s).unwrap();
        assert!(b.nodes.len() > 20);
        assert!(b.nodes.iter().any(|n| {
            n.links.iter().any(|&j| {
                (j as usize) < b.nodes.len() && b.nodes[j as usize].position[1] != n.position[1]
            })
        }));
        let mut s = scene();
        s.actors[0].scale = [2., 0.5, 2.];
        s.actors[0].position[0] = -1.1;
        let mut other = s.actors[0].clone();
        other.id = uuid::Uuid::new_v4();
        other.position[0] = 1.1;
        s.actors.push(other);
        let b = bake(&s).unwrap();
        let a = b.nodes.iter().position(|n| n.position[0] < 0).unwrap();
        let z = b.nodes.iter().position(|n| n.position[0] > 0).unwrap();
        assert!(!connected(&b, a, z));
    }
    #[test]
    fn jump_links_are_directed_and_checked() {
        let mut s = scene();
        s.actors[0].scale = [2., 0.5, 2.];
        s.actors[0].position[0] = -2.;
        let mut other = s.actors[0].clone();
        other.id = uuid::Uuid::new_v4();
        other.position[0] = 2.;
        s.actors.push(other);
        let mut link = crate::scene::Actor::cube("Jump".into());
        link.position = [-1.5, 0., 0.];
        link.kind = "Empty".into();
        let c = add_component(&mut link, LINK);
        c.properties.insert("end_x".into(), 3.0.into());
        c.properties.insert("end_z".into(), 0.0.into());
        c.properties.insert("bidirectional".into(), false.into());
        s.actors.push(link);
        let b = bake(&s).unwrap();
        assert_eq!(b.traversals.len(), 1);
        let t = &b.traversals[0];
        assert!(connected(&b, t.from as usize, t.to as usize));
        assert!(!connected(&b, t.to as usize, t.from as usize));
        let mut wall = crate::scene::Actor::cube("wall".into());
        wall.position = [0., 1., 0.];
        wall.scale = [0.1, 3., 4.];
        wall.collider = Some(Collider::default());
        s.actors.push(wall);
        assert!(bake(&s).unwrap_err().contains("intersects geometry"));
    }
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
