//! Portable, explicitly static world-space display lists. Authoring still uses
//! ordinary editable mesh faces; this contract only describes packet grouping,
//! corner order and per-camera submission order for the offline cooker.
use crate::{lighting::Quad, scene::Scene};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FaceRef {
    pub actor: Uuid,
    pub face: Uuid,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Primitive {
    pub first: FaceRef,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub second: Option<FaceRef>,
    /// [source triangle (0/1), corner (0..2)] in GPU packet order.
    /// The first triangle supplies corners 0,1,2; the other supplies corner 3.
    pub corners: [[u8; 2]; 4],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Program {
    pub version: u32,
    /// Explicit opt-in: actor transforms and these materials are baked, not
    /// dynamic inputs. Ordinary meshes remain the editable/query source.
    pub static_world: bool,
    pub primitives: Vec<Primitive>,
    /// Opaque primitive index << 2 | part: 0 triangle, 1/2 halves, 3 quad.
    /// Stored order, traversed in reverse by the prepared renderer.
    pub samples: Vec<Vec<u32>>,
}

impl Program {
    /// Does not resolve assets: scene loading calls this before hydration.
    pub fn validate(&self, scene: &Scene) -> Result<(), String> {
        let track = scene
            .visibility_track
            .as_ref()
            .ok_or("Ordered draw requires a visibility track")?;
        track.validate(scene)?;
        if self.version != 1 || !self.static_world || scene.fog.enabled || track.cull_backfaces {
            return Err("Ordered draw v1 requires explicit static world space, no fog and prepared facing selection".into());
        }
        if self.primitives.is_empty()
            || self.primitives.len() > 65536
            || self.samples.len() != track.samples.len()
        {
            return Err("Ordered draw primitive/sample count is invalid".into());
        }
        let actors: BTreeMap<_, _> = scene.actors.iter().map(|a| (a.id, a)).collect();
        let mut covered = BTreeSet::new();
        for primitive in &self.primitives {
            let corners = primitive.corners;
            if corners[..3]
                .iter()
                .any(|&[side, corner]| side != 0 || corner > 2)
                || corners[..3].iter().collect::<BTreeSet<_>>().len() != 3
                || if primitive.second.is_some() {
                    corners[3][0] != 1 || corners[3][1] > 2
                } else {
                    corners[3] != corners[2]
                }
            {
                return Err("Ordered draw packet has invalid corner references".into());
            }
            for reference in std::iter::once(primitive.first).chain(primitive.second) {
                let actor = actors
                    .get(&reference.actor)
                    .ok_or("Ordered draw references a missing actor")?;
                if actor.kind != "Mesh"
                    || actor.editable_mesh.is_none()
                    || actor.skeletal_mesh.is_some()
                    || actor.terrain.is_some()
                    || actor.parent.is_some()
                    || actor.attach.is_some()
                {
                    return Err("Ordered draw v1 requires root ordinary editable meshes".into());
                }
                // The existing visibility namespace is by face UUID. Do not
                // silently expand one referenced instance to every instance.
                if !covered.insert(reference.face) {
                    return Err("Ordered draw contains repeated source faces".into());
                }
            }
        }
        if covered != track.faces.iter().copied().collect() {
            return Err("Ordered draw must cover exactly the visibility track faces".into());
        }
        for (index, (tokens, sample)) in self.samples.iter().zip(&track.samples).enumerate() {
            if tokens.len() > 65535 {
                return Err("Ordered draw sample exceeds 65535 packets".into());
            }
            let mut faces = BTreeSet::new();
            for &token in tokens {
                let primitive = self
                    .primitives
                    .get((token >> 2) as usize)
                    .ok_or("Ordered draw token references a missing primitive")?;
                let part = token & 3;
                if (part == 0) != primitive.second.is_none() {
                    return Err("Ordered draw part disagrees with primitive topology".into());
                }
                let selected = [
                    (part != 2).then_some(primitive.first.face),
                    if part >= 2 {
                        primitive.second.map(|f| f.face)
                    } else {
                        None
                    },
                ];
                for face in selected.into_iter().flatten() {
                    if !faces.insert(face) {
                        return Err("Ordered draw sample draws a source face twice".into());
                    }
                }
            }
            let expected: BTreeSet<_> = sample
                .visible
                .iter()
                .map(|&i| {
                    track
                        .faces
                        .get(usize::from(i))
                        .copied()
                        .ok_or("Ordered draw visibility index is out of range")
                })
                .collect::<Result<_, _>>()?;
            if faces != expected || expected.len() != sample.visible.len() {
                return Err(format!(
                    "Ordered draw sample {index} differs from baked visibility"
                ));
            }
        }
        Ok(())
    }

    pub fn remap_actors(&mut self, identities: &BTreeMap<Uuid, Uuid>) {
        for primitive in &mut self.primitives {
            for reference in std::iter::once(&mut primitive.first).chain(primitive.second.as_mut())
            {
                if let Some(&actor) = identities.get(&reference.actor) {
                    reference.actor = actor;
                }
            }
        }
    }
}

pub struct Resolved {
    pub quads: Vec<Quad>,
    pub points: Vec<[[i32; 3]; 4]>,
    pub actors: BTreeSet<usize>,
}

#[derive(Clone)]
struct SourceFace {
    quad: Quad,
    points: [[i32; 3]; 4],
}

/// Resolve editable source data into native packet corners, with no source
/// executable formats or identifiers in the engine. Shared corners must agree
/// in UV/RGB and within one world Q12 unit; one canonical point closes the seam.
pub fn resolve(scene: &Scene, program: &Program) -> Result<Resolved, String> {
    program.validate(scene)?;
    let wanted: BTreeSet<_> = program
        .primitives
        .iter()
        .flat_map(|p| std::iter::once(p.first).chain(p.second))
        .collect();
    let mut source = BTreeMap::new();
    let mut actors = BTreeSet::new();
    let tracked_ids: BTreeSet<_> = wanted.iter().map(|r| r.face).collect();
    for (index, actor) in scene.actors.iter().enumerate() {
        if !wanted.iter().any(|r| r.actor == actor.id) {
            if actor
                .editable_mesh
                .as_ref()
                .and_then(|m| m.document.as_ref())
                .is_some_and(|d| d.faces.iter().any(|f| tracked_ids.contains(&f.id)))
            {
                return Err(
                    "Ordered draw cannot ambiguously share tracked face IDs across actors".into(),
                );
            }
            continue;
        }
        let doc = actor
            .editable_mesh
            .as_ref()
            .and_then(|m| m.document.as_ref())
            .ok_or("Ordered draw mesh asset is unresolved")?;
        if !scene.is_active(index) || actor.material.color != [1.; 3] {
            return Err("Ordered draw requires active, untinted static actors".into());
        }
        actors.insert(index);
        let transform = scene.world_matrix(index);
        for face in &doc.faces {
            let reference = FaceRef {
                actor: actor.id,
                face: face.id,
            };
            if !wanted.contains(&reference) {
                return Err("An ordered draw actor must include every mesh face".into());
            }
            if face.vertices[2] != face.vertices[3] {
                return Err("Ordered draw source faces must be triangles".into());
            }
            let material = crate::mesh::material(actor, face.material);
            let raw = doc.points(face);
            let mut points = [[0; 3]; 4];
            for (v, point) in raw.iter().enumerate() {
                points[v] = crate::mesh_compile::world_q12(&transform, *point)
                    .map_err(|_| "Ordered draw world coordinate exceeds Q12 range")?;
            }
            let quad = Quad {
                uv: face.uv,
                vertex_colors: face.vertex_colors,
                face: 0,
                points: points.map(|p| p.map(|v| v as f32 / 4096.)),
                normal: [0.; 3],
                material,
                id: Some(face.id),
            };
            if !crate::baked_draw::eligible(actor, std::slice::from_ref(&quad)) {
                return Err(
                    "Ordered draw requires immutable unlit, non-scrolling materials".into(),
                );
            }
            if source
                .insert(reference, SourceFace { quad, points })
                .is_some()
            {
                return Err("Ordered draw has duplicate mesh face IDs".into());
            }
        }
    }
    if source.len() != wanted.len() {
        return Err("Ordered draw references a missing mesh face".into());
    }
    let mut quads = Vec::with_capacity(program.primitives.len());
    let mut points = Vec::with_capacity(program.primitives.len());
    for primitive in &program.primitives {
        let first = source
            .get(&primitive.first)
            .ok_or("Missing ordered draw first face")?;
        let second = primitive
            .second
            .map(|r| source.get(&r).ok_or("Missing ordered draw second face"))
            .transpose()?;
        if let Some(second) = second {
            if first.quad.material != second.quad.material
                || first.quad.vertex_colors.is_some() != second.quad.vertex_colors.is_some()
            {
                return Err(
                    "Ordered draw halves have incompatible materials or color domains".into(),
                );
            }
        }
        let faces = [first, second.unwrap_or(first)];
        let mut quad = first.quad.clone();
        let corners = primitive
            .corners
            .map(|[side, corner]| (faces[usize::from(side)], usize::from(corner)));
        quad.points = corners.map(|(f, c)| f.points[c].map(|v| v as f32 / 4096.));
        points.push(corners.map(|(f, c)| f.points[c]));
        quad.uv = corners.map(|(f, c)| f.quad.uv[c]);
        quad.vertex_colors = first
            .quad
            .vertex_colors
            .map(|_| corners.map(|(f, c)| f.quad.vertex_colors.unwrap()[c]));
        if let Some(second) = second {
            // Native second half is packet corners 1,2,3, never 0,2,3.
            // Require a bijection, not three independent approximate matches.
            let same = |source_corner: usize, packet_corner: usize| {
                let (other, corner) = corners[packet_corner];
                (0..3).all(|axis| {
                    (i64::from(second.points[source_corner][axis])
                        - i64::from(other.points[corner][axis]))
                    .abs()
                        <= 1
                }) && second.quad.uv[source_corner] == other.quad.uv[corner]
                    && second.quad.vertex_colors.map(|v| v[source_corner])
                        == other.quad.vertex_colors.map(|v| v[corner])
            };
            if ![
                [1, 2, 3],
                [1, 3, 2],
                [2, 1, 3],
                [2, 3, 1],
                [3, 1, 2],
                [3, 2, 1],
            ]
            .iter()
            .any(|order| (0..3).all(|i| same(i, order[i])))
            {
                return Err("Ordered draw halves disagree in shared geometry, UV or RGB".into());
            }
        }
        quads.push(quad);
    }
    Ok(Resolved {
        quads,
        points,
        actors,
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    pub fn fixture() -> Scene {
        let mut scene = Scene::default();
        scene.actors.truncate(2);
        let actor = &mut scene.actors[1];
        actor.kind = "Mesh".into();
        actor.position = [17., 3., -4.];
        actor.scale = [10.; 3];
        let mut doc = crate::mesh::Document::default();
        doc.materials[0].material.unlit = true;
        doc.vertices = vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.], [1., 1., 0.]];
        let colors = [[1, 2, 3], [128, 129, 130], [254, 255, 0], [64, 65, 66]];
        let uv = [[0., 0.], [1., 0.], [0., 1.], [1., 1.]];
        for vertices in [[0, 1, 2, 2], [1, 2, 3, 3]] {
            doc.faces.push(crate::mesh::Face {
                id: Uuid::new_v4(),
                vertices,
                group: doc.groups[0].id,
                material: doc.materials[0].id,
                uv: vertices.map(|i| uv[i as usize]),
                vertex_colors: Some(vertices.map(|i| colors[i as usize])),
            });
        }
        let faces: Vec<_> = doc.faces.iter().map(|f| f.id).collect();
        let first = FaceRef {
            actor: actor.id,
            face: faces[0],
        };
        let second = FaceRef {
            actor: actor.id,
            face: faces[1],
        };
        let mut component = crate::mesh::Component::new(Uuid::new_v4());
        component.document = Some(std::sync::Arc::new(doc));
        actor.editable_mesh = Some(component);
        scene.visibility_track = Some(crate::visibility_track::Track {
            camera: scene.actors[0].id,
            sample_ms: 100,
            cull_backfaces: false, static_world: false,
            faces,
            samples: vec![vec![0, 1], vec![1], vec![0], vec![]]
                .into_iter()
                .map(|visible| crate::visibility_track::Sample {
                    position: [0.; 3],
                    rotation: [0.; 3],
                    fov: 90.,
                    visible,
                    cut: false,
                })
                .collect(),
            navigation: None,
        });
        scene.ordered_draw = Some(Program {
            version: 1,
            static_world: true,
            primitives: vec![Primitive {
                first,
                second: Some(second),
                corners: [[0, 0], [0, 1], [0, 2], [1, 2]],
            }],
            samples: vec![vec![3], vec![2], vec![1], vec![]],
        });
        scene
    }
    #[test]
    fn resolves_native_corners_and_world_coordinates_without_changing_meshes() {
        let scene = fixture();
        let program = scene.ordered_draw.as_ref().unwrap();
        let source = scene.actors[1]
            .editable_mesh
            .as_ref()
            .unwrap()
            .document
            .clone();
        let resolved = resolve(&scene, program).unwrap();
        assert_eq!(
            resolved.points[0],
            [
                [69632, 12288, -16384],
                [110592, 12288, -16384],
                [69632, 53248, -16384],
                [110592, 53248, -16384]
            ]
        );
        assert_eq!(resolved.quads[0].vertex_colors.unwrap()[3], [64, 65, 66]);
        assert_eq!(
            scene.actors[1].editable_mesh.as_ref().unwrap().document,
            source
        );
        assert_eq!(resolved.actors, BTreeSet::from([1]));
    }
    #[test]
    fn membership_and_topology_fail_closed() {
        let scene = fixture();
        let good = scene.ordered_draw.as_ref().unwrap();
        for tokens in [vec![0], vec![7], vec![1, 3], vec![3, 3], vec![]] {
            let mut program = good.clone();
            program.samples[0] = tokens;
            assert!(program.validate(&scene).is_err());
        }
        let mut program = good.clone();
        program.primitives[0].corners[0] = [1, 0];
        assert!(program.validate(&scene).is_err());
        let mut program = good.clone();
        program.primitives[0].second = None;
        assert!(program.validate(&scene).is_err());
        let mut program = good.clone();
        program.static_world = false;
        assert!(program.validate(&scene).is_err());
        let mut scene = scene;
        scene.visibility_track.as_mut().unwrap().samples[0]
            .visible
            .push(0);
        assert!(
            scene
                .ordered_draw
                .as_ref()
                .unwrap()
                .validate(&scene)
                .is_err()
        );
    }
    #[test]
    fn changed_uv_rgb_geometry_or_material_cannot_merge_silently() {
        for mode in 0..4 {
            let mut scene = fixture();
            let doc = std::sync::Arc::make_mut(
                scene.actors[1]
                    .editable_mesh
                    .as_mut()
                    .unwrap()
                    .document
                    .as_mut()
                    .unwrap(),
            );
            match mode {
                0 => doc.faces[1].uv[0][0] = 0.25,
                1 => doc.faces[1].vertex_colors.as_mut().unwrap()[0][0] = 42,
                2 => {
                    doc.vertices.push([1.1, 0., 0.]);
                    doc.faces[1].vertices[0] = 4;
                }
                _ => {
                    let mut slot = doc.materials[0].clone();
                    slot.id = Uuid::new_v4();
                    slot.material.depth_bias = 1;
                    doc.faces[1].material = slot.id;
                    doc.materials.push(slot);
                }
            }
            assert!(resolve(&scene, scene.ordered_draw.as_ref().unwrap()).is_err());
        }
    }
    #[test]
    fn optional_scene_document_roundtrip_and_identity_remapping() {
        let mut scene = fixture();
        let before = scene.ordered_draw.clone();
        let encoded = serde_json::to_vec(&scene).unwrap();
        let restored: Scene = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(restored.ordered_draw, before);
        scene.duplicate_identities();
        assert_ne!(scene.ordered_draw, before);
        scene
            .ordered_draw
            .as_ref()
            .unwrap()
            .validate(&scene)
            .unwrap();
        assert_eq!(
            scene.ordered_draw.as_ref().unwrap().primitives[0]
                .first
                .actor,
            scene.actors[1].id
        );
        let mut plain = Scene::default();
        plain.ordered_draw = None;
        let json = serde_json::to_value(plain).unwrap();
        assert!(json.get("ordered_draw").is_none());
    }
    #[test]
    fn ambiguous_instances_do_not_leak_into_the_legacy_render_pass() {
        let mut scene = fixture();
        let mut instance = scene.actors[1].clone();
        instance.id = Uuid::new_v4();
        scene.actors.push(instance);
        assert!(
            resolve(&scene, scene.ordered_draw.as_ref().unwrap())
                .err().unwrap()
                .contains("ambiguously")
        );
    }
}
