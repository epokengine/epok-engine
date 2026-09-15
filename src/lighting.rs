//! Offline vertex lighting and the bounded lighting model used by the PSX runtime.
use crate::{
    scene::{Actor, Scene},
    transform::Matrix,
};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum LightType {
    #[default]
    Directional,
    Point,
}
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum LightMode {
    Baked,
    #[default]
    Realtime,
    Mixed,
}
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum Receive {
    Baked,
    #[default]
    Realtime,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Light {
    pub enabled: bool,
    pub kind: LightType,
    pub mode: LightMode,
    pub color: [f32; 3],
    pub intensity: f32,
    pub range: f32,
    pub priority: i32,
    pub shadows: bool,
}
impl Default for Light {
    fn default() -> Self {
        Self {
            enabled: true,
            kind: LightType::Directional,
            mode: LightMode::Realtime,
            color: [1.; 3],
            intensity: 0.8,
            range: 8.,
            priority: 0,
            shadows: true,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct MeshLighting {
    pub receive: Receive,
    pub static_geometry: bool,
    pub cast_shadows: bool,
    pub subdivisions: u8,
}
impl Default for MeshLighting {
    fn default() -> Self {
        Self {
            receive: Receive::Realtime,
            static_geometry: false,
            cast_shadows: true,
            subdivisions: 1,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Settings {
    pub ambient: [f32; 3],
    pub ao_strength: f32,
    pub ao_distance: f32,
    pub point_lights: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            ambient: [0.3; 3],
            ao_strength: 0.,
            ao_distance: 1.,
            point_lights: true,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Bake {
    pub fingerprint: u64,
    pub colors: Vec<Vec<[u8; 3]>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actor_ids: Vec<uuid::Uuid>,
}
impl Bake {
    /// Stale colors are intentionally retained until an explicit build. Match
    /// by identity so removing/reordering actors cannot transfer their lighting.
    pub fn preview_colors(&self, actor: uuid::Uuid, vertices: usize) -> Option<&[[u8; 3]]> {
        let index = self.actor_ids.iter().position(|id| *id == actor)?;
        self.colors
            .get(index)
            .filter(|colors| colors.len() == vertices)
            .map(Vec::as_slice)
    }
}
pub const CORNERS: [[f32; 3]; 8] = [
    [-0.5, -0.5, -0.5],
    [0.5, -0.5, -0.5],
    [0.5, 0.5, -0.5],
    [-0.5, 0.5, -0.5],
    [-0.5, -0.5, 0.5],
    [0.5, -0.5, 0.5],
    [0.5, 0.5, 0.5],
    [-0.5, 0.5, 0.5],
];
pub const FACES: [[usize; 4]; 6] = [
    [0, 1, 2, 3],
    [5, 4, 7, 6],
    [4, 0, 3, 7],
    [1, 5, 6, 2],
    [3, 2, 6, 7],
    [4, 5, 1, 0],
];
pub const NORMALS: [[f32; 3]; 6] = [
    [0., 0., -1.],
    [0., 0., 1.],
    [-1., 0., 0.],
    [1., 0., 0.],
    [0., 1., 0.],
    [0., -1., 0.],
];
pub fn tiled(e: &Actor) -> bool {
    e.skeletal_mesh.is_none()
        && e.editable_mesh.is_none()
        && e.kind == "Mesh"
        && e.scale[1] <= 0.25
        && e.scale[0] >= 2.
        && e.scale[2] >= 2.
}
pub fn face_steps(e: &Actor, f: usize) -> usize {
    if tiled(e) {
        if f == 5 {
            0
        } else if f == 4 {
            10.max(e.lighting.subdivisions as usize)
        } else {
            e.lighting.subdivisions as usize
        }
    } else {
        e.lighting.subdivisions as usize
    }
}
pub fn quad_count(e: &Actor) -> usize {
    if e.kind != "Mesh" {
        return 0;
    }
    if e.editable_mesh.is_some() || e.skeletal_mesh.is_some() {
        return quads(e).len();
    }
    (0..6).map(|f| face_steps(e, f).pow(2)).sum()
}
#[derive(Clone)]
pub struct Quad {
    pub uv: [[f32; 2]; 4],
    pub face: usize,
    pub points: [[f32; 3]; 4],
    pub normal: [f32; 3],
    pub material: crate::scene::Material,
    pub id: Option<uuid::Uuid>,
}
pub fn quads(e: &Actor) -> Vec<Quad> {
    if e.kind != "Mesh" {
        return vec![];
    }
    if let Some(mesh) = &e.skeletal_mesh {
        return crate::skeletal::quads(mesh);
    }
    if let Some(mesh) = &e.editable_mesh {
        return crate::mesh_compile::quads(e, mesh);
    }
    let mut out = Vec::new();
    for (face, indices) in FACES.iter().enumerate() {
        let n = face_steps(e, face);
        let p = indices.map(|i| CORNERS[i]);
        for y in 0..n {
            for x in 0..n {
                let points = [(x, y), (x + 1, y), (x + 1, y + 1), (x, y + 1)].map(|(x, y)| {
                    // Same Q12 integer grid as the target, including negative coordinates.
                    std::array::from_fn(|c| {
                        let a = (p[0][c] * 4096.) as i32;
                        let u = ((p[1][c] - p[0][c]) * 4096.) as i32;
                        let v = ((p[3][c] - p[0][c]) * 4096.) as i32;
                        (a + u * x as i32 / n as i32 + v * y as i32 / n as i32) as f32 / 4096.
                    })
                });
                out.push(Quad {
                    uv: [(x, y), (x + 1, y), (x + 1, y + 1), (x, y + 1)]
                        .map(|(x, y)| [x as f32 / n as f32, y as f32 / n as f32]),
                    face,
                    points,
                    normal: NORMALS[face],
                    material: e.material.clone(),
                    id: None,
                });
            }
        }
    }
    out
}
pub fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a.into_iter().zip(b).map(|(a, b)| a * b).sum()
}
pub fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}
pub fn unit(v: [f32; 3]) -> [f32; 3] {
    let len = dot(v, v).sqrt();
    if len < 1e-9 {
        [0.; 3]
    } else {
        v.map(|x| x / len)
    }
}
pub fn normal(world: Matrix, face: usize) -> [f32; 3] {
    transform_normal(world, NORMALS[face])
}
pub fn transform_normal(world: Matrix, normal: [f32; 3]) -> [f32; 3] {
    let inverse = world.inverse().unwrap_or(Matrix::IDENTITY);
    unit(std::array::from_fn(|r| {
        (0..3).map(|c| inverse.0[c][r] * normal[c]).sum()
    }))
}
pub fn fingerprint(scene: &Scene) -> u64 {
    // Stable FNV-1a over authored bake inputs. Excludes cached output and UI/camera edits.
    struct Fnv(u64);
    impl Hasher for Fnv {
        fn finish(&self) -> u64 {
            self.0
        }
        fn write(&mut self, b: &[u8]) {
            for v in b {
                self.0 = (self.0 ^ u64::from(*v)).wrapping_mul(1099511628211);
            }
        }
    }
    let meshes: Vec<_> = scene
        .actors
        .iter()
        .enumerate()
        .filter(|(_, e)| e.kind == "Mesh")
        .map(|(i, e)| {
            (
                i,
                scene.world_matrix(i).0,
                &e.lighting,
                &e.material,
                &e.editable_mesh,
                e.editable_mesh.as_ref().and_then(|m| m.document.as_deref()),
            )
        })
        .collect();
    let lights: Vec<_> = scene
        .actors
        .iter()
        .enumerate()
        .filter_map(|(i, e)| {
            e.light
                .as_ref()
                .filter(|l| l.mode != LightMode::Realtime)
                .map(|l| (i, scene.world_matrix(i).0, l))
        })
        .collect();
    let mut hash = Fnv(14695981039346656037);
    serde_json::to_vec(&(2, &scene.environment, scene.actors.len(), meshes, lights))
        .unwrap_or_default()
        .hash(&mut hash);
    hash.finish()
}
pub fn valid_bake(scene: &Scene) -> bool {
    scene
        .bake
        .as_ref()
        .is_some_and(|b| bake_matches(scene, b, fingerprint(scene)))
}
fn bake_matches(scene: &Scene, bake: &Bake, fingerprint: u64) -> bool {
    bake.fingerprint == fingerprint
        && bake.colors.len() == scene.actors.len()
        && (bake.actor_ids.is_empty()
            || bake
                .actor_ids
                .iter()
                .copied()
                .eq(scene.actors.iter().map(|e| e.id)))
        && bake
            .colors
            .iter()
            .zip(&scene.actors)
            .all(|(c, e)| c.len() == if baked(e) { quad_count(e) * 4 } else { 0 })
}
pub fn baked(e: &Actor) -> bool {
    e.kind == "Mesh"
        && (e.editable_mesh.is_some() || !e.material.unlit)
        && e.lighting.receive == Receive::Baked
}
pub fn validate(scene: &Scene) -> Result<(), String> {
    let s = &scene.environment;
    if s.ambient
        .iter()
        .any(|x| !x.is_finite() || !(0. ..=1.).contains(x))
        || !s.ao_strength.is_finite()
        || !(0. ..=1.).contains(&s.ao_strength)
        || !s.ao_distance.is_finite()
        || !(0.01..=32.).contains(&s.ao_distance)
    {
        return Err("Invalid ambient/AO settings".into());
    }
    // Editable meshes can form a world larger than the per-frame GPU budget:
    // their immutable payload may live on CD. Keep each compiled mesh bounded
    // and retain the existing aggregate limit for resources that stay resident.
    if scene.actors.iter().any(|entity| quad_count(entity) > 3500)
        || scene
            .actors
            .iter()
            .filter(|entity| entity.editable_mesh.is_none())
            .map(quad_count)
            .sum::<usize>()
            > 3500
    {
        return Err("Lighting geometry budget: at most 7000 triangles per mesh and across resident primitive/skeletal meshes; reduce subdivisions".into());
    }
    if scene.actors.iter().filter(|e| e.light.is_some()).count() > 32 {
        return Err("Lighting budget: at most 32 authored lights".into());
    }
    for e in &scene.actors {
        if !(1..=8).contains(&e.lighting.subdivisions) {
            return Err("Mesh subdivisions must be 1–8".into());
        }
        if baked(e) && !e.lighting.static_geometry {
            return Err("Baked receivers must be marked Static".into());
        }
        if let Some(l) = &e.light
            && (l
                .color
                .iter()
                .any(|x| !x.is_finite() || !(0. ..=1.).contains(x))
                || !l.intensity.is_finite()
                || !(0. ..=2.).contains(&l.intensity)
                || !l.range.is_finite()
                || !(0.01..=128.).contains(&l.range)
                || !(-100..=100).contains(&l.priority))
        {
            return Err("Invalid Light component values".into());
        }
    }
    Ok(())
}
struct Source {
    light: Light,
    position: [f32; 3],
    toward: [f32; 3],
}
pub struct Lighting {
    sources: Vec<Source>,
    casters: Vec<Caster>,
    environment: Settings,
}
struct Caster {
    index: usize,
    inverse: Matrix,
    triangles: Option<Vec<[[f32; 3]; 3]>>,
}
impl Lighting {
    pub fn new(scene: &Scene) -> Self {
        Self::with_shadows(scene, true)
    }
    /// Interactive shading never rebuilds static shadows or their ray geometry.
    pub fn unshadowed(scene: &Scene) -> Self {
        Self::with_shadows(scene, false)
    }
    fn with_shadows(scene: &Scene, shadows: bool) -> Self {
        Self {
            sources: scene
                .actors
                .iter()
                .enumerate()
                .filter_map(|(i, e)| {
                    e.light.as_ref().filter(|l| l.enabled).map(|l| {
                        let world = scene.world_matrix(i);
                        Source {
                            light: l.clone(),
                            position: world.point([0.; 3]),
                            toward: unit(world.vector([0., 0., -1.])),
                        }
                    })
                })
                .collect(),
            casters: scene
                .actors
                .iter()
                .enumerate()
                .filter(|(_, e)| {
                    shadows
                        && e.kind == "Mesh"
                        && e.lighting.static_geometry
                        && e.lighting.cast_shadows
                })
                .filter_map(|(i, e)| {
                    scene.world_matrix(i).inverse().ok().map(|m| Caster {
                        index: i,
                        inverse: m,
                        triangles: e.editable_mesh.as_ref().map(|_| {
                            quads(e)
                                .into_iter()
                                .flat_map(|q| {
                                    [
                                        [q.points[0], q.points[1], q.points[2]],
                                        [q.points[0], q.points[2], q.points[3]],
                                    ]
                                })
                                .collect()
                        }),
                    })
                })
                .collect(),
            environment: scene.environment.clone(),
        }
    }
    fn blocked(&self, p: [f32; 3], direction: [f32; 3], distance: f32, exclude: usize) -> bool {
        self.casters.iter().any(|caster| {
            if caster.index == exclude && caster.triangles.is_none() {
                return false;
            }
            let o = caster.inverse.point(p);
            let d = caster.inverse.vector(direction);
            if let Some(triangles) = &caster.triangles {
                return triangles.iter().any(|t| {
                    crate::mesh::hit(o, d, *t).is_some_and(|t| t > 0.002 && t < distance)
                });
            }
            let mut near = 0.002_f32;
            let mut far = distance;
            for c in 0..3 {
                if d[c].abs() < 1e-8 {
                    if o[c] < -0.5 || o[c] > 0.5 {
                        return false;
                    }
                } else {
                    let a = (-0.5 - o[c]) / d[c];
                    let b = (0.5 - o[c]) / d[c];
                    near = near.max(a.min(b));
                    far = far.min(a.max(b));
                }
            }
            far > near
        })
    }
    fn contribution(source: &Source, p: [f32; 3]) -> ([f32; 3], f32, f32) {
        if source.light.kind == LightType::Directional {
            (source.toward, source.light.intensity, 512.)
        } else {
            let delta = sub(source.position, p);
            let distance = dot(delta, delta).sqrt();
            let attenuation = (1. - distance / source.light.range).clamp(0., 1.);
            (
                unit(delta),
                source.light.intensity * attenuation * attenuation,
                distance,
            )
        }
    }
    pub fn sample(
        &self,
        p: [f32; 3],
        n: [f32; 3],
        object: usize,
        offline: bool,
        shadows: bool,
    ) -> [u8; 3] {
        let origin = std::array::from_fn(|c| p[c] + n[c] * 0.003);
        let mut ao = 1.;
        if offline && shadows && self.environment.ao_strength > 0. {
            let rays = [
                [1., 1., 1.],
                [-1., 1., 1.],
                [1., -1., 1.],
                [-1., -1., 1.],
                [1., 1., -1.],
                [-1., 1., -1.],
                [1., -1., -1.],
                [-1., -1., -1.],
            ];
            let mut blocked = 0;
            let mut total = 0;
            for ray in rays {
                let d = unit(ray);
                if dot(d, n) > 0. {
                    total += 1;
                    if self.blocked(origin, d, self.environment.ao_distance, object) {
                        blocked += 1;
                    }
                }
            }
            if total > 0 {
                ao -= self.environment.ao_strength * blocked as f32 / total as f32;
            }
        }
        let mut rgb = self.environment.ambient.map(|c| c * ao);
        let mut selected = [None, None];
        let mut scores = [f32::NEG_INFINITY; 2];
        if !offline {
            for (i, s) in self.sources.iter().enumerate() {
                if s.light.mode == LightMode::Baked {
                    continue;
                }
                let slot = usize::from(s.light.kind == LightType::Point);
                if slot == 1 && !self.environment.point_lights {
                    continue;
                }
                let (_, power, _) = Self::contribution(s, p);
                if power <= 0. {
                    continue;
                }
                let score = s.light.priority as f32 * 4. + power;
                if score > scores[slot] {
                    scores[slot] = score;
                    selected[slot] = Some(i);
                }
            }
        }
        for (i, s) in self.sources.iter().enumerate() {
            if if offline {
                s.light.mode == LightMode::Realtime
            } else {
                !selected.contains(&Some(i))
            } {
                continue;
            }
            let (d, power, distance) = Self::contribution(s, p);
            let lambert = dot(n, d).max(0.);
            if power <= 0.
                || lambert <= 0.
                || (offline
                    && shadows
                    && s.light.shadows
                    && self.blocked(origin, d, distance - 0.004, object))
            {
                continue;
            }
            for (c, out) in rgb.iter_mut().enumerate() {
                *out += s.light.color[c] * power * lambert;
            }
        }
        rgb.map(|v| (v.clamp(0., 1.) * 255.).round() as u8)
    }
}
pub fn bake(scene: &Scene) -> Result<Bake, String> {
    scene.validate()?;
    Ok(sample_bake(scene, fingerprint(scene)))
}
fn sample_bake(scene: &Scene, fingerprint: u64) -> Bake {
    let lighting = Lighting::new(scene);
    let mut colors = vec![Vec::new(); scene.actors.len()];
    for (i, e) in scene.actors.iter().enumerate().filter(|(_, e)| baked(e)) {
        let world = scene.world_matrix(i);
        for q in quads(e) {
            let n = transform_normal(world, q.normal);
            for p in q.points {
                colors[i].push(lighting.sample(world.point(p), n, i, true, true));
            }
        }
    }
    Bake {
        fingerprint,
        colors,
        actor_ids: scene.actors.iter().map(|e| e.id).collect(),
    }
}
pub fn modulate(light: [u8; 3], color: [f32; 3]) -> [u8; 3] {
    std::array::from_fn(|c| {
        (u16::from(light[c]) * (color[c].clamp(0., 1.) * 255.).round() as u16 / 255) as u8
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn shadow_scene() -> Scene {
        let mut scene = Scene::default();
        scene.environment.ambient = [0.1; 3];
        for i in 1..=3 {
            scene.actors[i].lighting.static_geometry = true;
            scene.actors[i].lighting.receive = Receive::Baked;
        }
        scene.actors[1].position = [-1.4, 0.5, 0.];
        scene.actors[2].position = [1.4, 0.5, 0.];
        let mut sun = Actor::cube("Sun".into());
        sun.kind = "Empty".into();
        sun.rotation = [90., 0., 0.];
        sun.light = Some(Light {
            mode: LightMode::Mixed,
            ..Default::default()
        });
        scene.actors.push(sun);
        scene.bake = Some(bake(&scene).unwrap());
        scene
    }

    #[test]
    fn preview_retains_baked_shadows_until_an_explicit_rebuild() {
        let mut scene = shadow_scene();
        let original = scene.bake.clone().unwrap();
        let floor = scene.actors[3].id;
        let vertices = quad_count(&scene.actors[3]) * 4;
        scene.actors[1].lighting.static_geometry = false;
        scene.actors[1].lighting.receive = Receive::Realtime;
        assert!(!valid_bake(&scene));
        let cached = scene
            .bake
            .as_ref()
            .unwrap()
            .preview_colors(floor, vertices)
            .unwrap();
        assert_eq!(cached, original.colors[3]);
        assert_ne!(
            cached,
            bake(&scene).unwrap().colors[3],
            "Preview must not recompute shadows after an edit"
        );
        assert!(
            Lighting::unshadowed(&scene).casters.is_empty(),
            "Interactive rendering must not build ray geometry"
        );
        assert_eq!(scene.bake.as_ref().unwrap(), &original);
        scene.bake = Some(bake(&scene).unwrap());
        assert!(valid_bake(&scene));
        assert_ne!(scene.bake.as_ref().unwrap().colors[3], original.colors[3]);
    }

    #[test]
    fn stale_colors_follow_actor_identity_and_require_matching_vertex_counts() {
        let mut scene = shadow_scene();
        let original = scene.bake.clone().unwrap();
        let floor = scene.actors[3].id;
        let vertices = quad_count(&scene.actors[3]) * 4;
        scene.actors.swap(1, 3);
        scene.actors.remove(2);
        scene.actors.push(Actor::cube("New cube".into()));
        assert!(!valid_bake(&scene));
        let cache = scene.bake.as_ref().unwrap();
        assert_eq!(
            cache.preview_colors(floor, vertices).unwrap(),
            original.colors[3]
        );
        assert!(
            cache
                .preview_colors(scene.actors.last().unwrap().id, 24)
                .is_none()
        );
        assert!(cache.preview_colors(floor, vertices + 4).is_none());
        scene.bake.as_mut().unwrap().colors[3].pop();
        assert!(
            scene
                .bake
                .as_ref()
                .unwrap()
                .preview_colors(floor, vertices)
                .is_none()
        );
    }

    #[test]
    fn legacy_bake_positions_are_bound_once_when_loading_the_scene() {
        let scene = shadow_scene();
        let mut document = serde_json::to_value(&scene).unwrap();
        document["bake"]
            .as_object_mut()
            .unwrap()
            .remove("actor_ids");
        let mut copy: Scene = serde_json::from_value(document).unwrap();
        assert!(valid_bake(&copy));
        let floor = copy.actors[3].id;
        copy.actors.swap(1, 3);
        let vertices = quad_count(&copy.actors[1]) * 4;
        assert_eq!(
            copy.bake
                .as_ref()
                .unwrap()
                .preview_colors(floor, vertices)
                .unwrap(),
            scene.bake.as_ref().unwrap().colors[3]
        );
        let reloaded: Scene = serde_json::from_value(serde_json::to_value(&copy).unwrap()).unwrap();
        assert_eq!(reloaded.bake, copy.bake);
    }
    #[test]
    fn baked_point_gradient_and_parent_invalidation() {
        let mut s = Scene::default();
        s.actors[1].lighting = MeshLighting {
            receive: Receive::Baked,
            static_geometry: true,
            subdivisions: 4,
            ..Default::default()
        };
        s.actors[2].kind = "Empty".into();
        s.actors[3].kind = "Empty".into();
        s.actors[2].light = Some(Light {
            kind: LightType::Point,
            mode: LightMode::Baked,
            intensity: 1.,
            range: 3.,
            ..Default::default()
        });
        s.actors[2].position = [0.5, 1., -1.];
        s.reparent(2, Some(3), false).unwrap();
        s.actors[3].position = [0.; 3];
        s.actors[3].scale = [1.; 3];
        s.bake = Some(bake(&s).unwrap());
        let colors = &s.bake.as_ref().unwrap().colors[1];
        assert!(colors.iter().map(|c| c[0]).max() > colors.iter().map(|c| c[0]).min());
        s.actors[3].position[0] += 1.;
        assert!(!valid_bake(&s));
        s.bake = Some(bake(&s).unwrap());
        assert!(valid_bake(&s));
        s.bake.as_mut().unwrap().colors[1].pop();
        assert!(!valid_bake(&s));
    }
    #[test]
    fn bake_shadows_invalidation_and_serialization() {
        let mut s = Scene::default();
        s.environment.ambient = [0.1; 3];
        s.actors[3].lighting.static_geometry = true;
        s.actors[3].lighting.receive = Receive::Baked;
        s.actors[1].lighting.static_geometry = true;
        s.actors[2].kind = "Empty".into();
        let mut l = Actor::cube("Sun".into());
        l.kind = "Empty".into();
        l.rotation = [90., 0., 0.];
        l.light = Some(Light {
            mode: LightMode::Mixed,
            ..Light::default()
        });
        s.actors.push(l);
        s.bake = Some(bake(&s).unwrap());
        assert!(valid_bake(&s));
        let colors = &s.bake.as_ref().unwrap().colors[3];
        assert!(colors.iter().any(|c| c[0] > 180));
        assert!(colors.iter().any(|c| c[0] < 40));
        let copy: Scene = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
        assert!(valid_bake(&copy));
        s.actors[0].position[0] += 1.;
        assert!(valid_bake(&s));
        s.actors[1].position[0] += 1.;
        assert!(!valid_bake(&s));
    }
    #[test]
    fn realtime_budget_unlit_and_nonuniform_normals() {
        let mut s = Scene::default();
        s.environment.ambient = [0.; 3];
        for _ in 0..4 {
            let mut l = Actor::cube("Light".into());
            l.kind = "Empty".into();
            l.light = Some(Light {
                intensity: 0.25,
                ..Light::default()
            });
            s.actors.push(l);
        }
        let rgb = Lighting::new(&s).sample([0.; 3], [0., 0., -1.], 1, false, false);
        assert_eq!(rgb, [64; 3]);
        let m = Matrix::trs([0.; 3], [10., 30., 20.], [2., 1., 3.]);
        let n = normal(m, 4);
        assert!(dot(n, m.vector([1., 0., 0.])).abs() < 0.0001);
        s.actors[1].lighting.subdivisions = 9;
        assert!(s.validate().is_err());
    }
}
#[test]
fn blockout_walls_shadow_floors_in_the_same_entity() {
    let mut scene = Scene::default();
    scene.actors.truncate(2);
    let mut doc = crate::mesh::Document::default();
    let g = doc.groups[0].id;
    let m = doc.materials[0].id;
    doc.primitive("Plane", [0., 0., 0.], [4., 1., 4.], 1, g, m);
    doc.primitive("Plane", [0., 2., 0.], [4., 1., 4.], 1, g, m);
    let mut component = crate::mesh::Component::new(uuid::Uuid::new_v4());
    component.document = Some(std::sync::Arc::new(doc));
    scene.actors[1].position = [0.; 3];
    scene.actors[1].editable_mesh = Some(component);
    scene.actors[1].lighting.static_geometry = true;
    let lighting = Lighting::new(&scene);
    assert!(lighting.blocked([0., 0.003, 0.], [0., 1., 0.], 10., 1));
    assert!(!lighting.blocked([0., 2.003, 0.], [0., 1., 0.], 10., 1));
}
