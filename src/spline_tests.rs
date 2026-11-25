use super::*;
use crate::{mesh, scene::Actor};

#[test]
fn spline_loop_keeps_the_driving_surface_facing_inward() {
    let curve = Spline {
        points: Spline::vertical_loop(),
        ..Spline::default()
    };
    let frames = curve.frames().unwrap();
    assert!(frames[0].up[1] > 0.99);
    let top = frames
        .iter()
        .max_by(|a, b| a.center[1].total_cmp(&b.center[1]))
        .unwrap();
    assert!(top.up[1] < -0.95, "top normal {:?}", top.up);
    assert!(frames.last().unwrap().up[1] > 0.95);
    for (point, frame) in curve
        .points
        .iter()
        .zip(frames.iter().step_by(usize::from(curve.steps)))
    {
        assert_eq!(point.position, frame.center);
    }
    assert!(frames.windows(2).all(|w| dot(w[0].right, w[1].right) > 0.9));
    let mut doc = Document::default();
    curve.apply(&mut doc).unwrap();
    for (span, strip) in doc.faces.chunks_exact(2).enumerate() {
        for face in strip {
            assert!(dot(mesh::face_normal(doc.points(face)), frames[span].up) > 0.88);
        }
    }
}

#[test]
fn spline_banked_circuit_welds_the_seam_and_all_profiles_survive_a_cook() {
    let mut roll = Spline::default();
    roll.points[0].bank = 170.;
    roll.points[1].bank = -170.;
    assert_eq!(roll.midpoint(0).bank.abs(), 180.);
    assert!(roll.frames().unwrap().iter().all(|f| f.up[1] < -0.98));
    let mut curve = Spline {
        points: Spline::circuit(),
        closed: true,
        ..Spline::default()
    };
    curve.points[2].position[1] = 4.;
    curve.points[3].bank = 55.;
    let frames = curve.frames().unwrap();
    let last = frames.last().unwrap();
    assert_eq!((frames[0].center, frames[0].up), (last.center, last.up));
    assert!(dot(frames[frames.len() - 2].up, last.up) > 0.96);
    let spans = frames.len() - 1;
    for profile in [Profile::Road, Profile::Railing, Profile::Wall] {
        curve.profile = profile;
        (curve.width, curve.height) = profile.dimensions();
        let mut doc = Document::default();
        curve.clone().apply(&mut doc).unwrap();
        doc.validate().unwrap();
        let corners = if profile == Profile::Road { 2 } else { 4 };
        let strips = if profile == Profile::Road { 1 } else { 4 };
        let final_strip = &doc.faces[(spans - 1) * 2 * strips];
        assert_eq!(
            final_strip.vertices[1], 0,
            "last strip reuses the first ring"
        );
        let posts = if profile == Profile::Railing {
            (last.distance / curve.post_spacing).ceil() as usize
        } else {
            0
        };
        assert_eq!(doc.vertices.len(), spans * corners + 8 * posts);
        assert_eq!(doc.faces.len(), spans * 2 * strips + 6 * posts);
        let source = serde_json::to_value(&doc).unwrap();
        assert!(source.get("spline").is_some() && source.get("faces").is_none());
        let restored = Document::parse(&serde_json::to_vec(&doc).unwrap()).unwrap();
        assert_eq!(
            doc, restored,
            "stable generated geometry and face identifiers"
        );
        let mut actor = Actor::cube("Test spline".into());
        let mut component = mesh::Component::new(Uuid::new_v4());
        component.document = Some(std::sync::Arc::new(restored));
        actor.editable_mesh = Some(component);
        let header = crate::mesh_compile::header(&actor, 0).unwrap();
        assert!(header.contains("MeshGeometry") && header.contains("MeshQuad"));
    }
}

#[test]
fn spline_rejects_invalid_points_and_over_budget_posts_before_replacing_the_mesh() {
    let mut doc = Document::default();
    let mut curve = Spline::default();
    curve.clone().apply(&mut doc).unwrap();
    let original = doc.clone();
    curve.points[0] = curve.points[1];
    assert!(curve.clone().apply(&mut doc).is_err());
    curve.points = vec![[-120., 0., 0.].into(), [120., 0., 0.].into()];
    curve.profile = Profile::Railing;
    (curve.width, curve.height) = curve.profile.dimensions();
    curve.post_spacing = 0.1;
    assert!(curve.apply(&mut doc).unwrap_err().contains("faces"));
    assert_eq!(doc, original);
    let legacy = mesh::tests::shape("Box");
    let bytes = serde_json::to_vec(&legacy).unwrap();
    assert!(!String::from_utf8_lossy(&bytes).contains("spline"));
    assert_eq!(legacy, Document::parse(&bytes).unwrap());
}

#[test]
fn outgoing_surfaces_survive_an_edit_and_regeneration() {
    let mut doc = Document::default();
    let mut second = doc.materials[0].clone();
    second.id = Uuid::new_v4();
    second.name = "Boost".into();
    let surface = second.id;
    doc.materials.push(second);
    let mut curve = Spline {
        points: Spline::circuit(),
        closed: true,
        ..Default::default()
    };
    curve.points[2].surface = Some(surface);
    curve.points[2].tag = 5;
    let mid = curve.midpoint(2);
    assert_eq!((mid.surface, mid.tag), (Some(surface), 5));
    curve.points.insert(3, mid);
    curve.clone().apply(&mut doc).unwrap();
    let regenerated: Document = serde_json::from_slice(&serde_json::to_vec(&doc).unwrap()).unwrap();
    assert_eq!(doc, regenerated);
    for (i, faces) in doc
        .faces
        .chunks_exact(usize::from(curve.steps) * 2)
        .enumerate()
    {
        let expected = if i == 2 || i == 3 {
            surface
        } else {
            doc.materials[0].id
        };
        assert!(faces.iter().all(|f| f.material == expected));
    }
    curve.points[0].surface = Some(Uuid::new_v4());
    assert!(curve.apply(&mut doc).is_err());
}
