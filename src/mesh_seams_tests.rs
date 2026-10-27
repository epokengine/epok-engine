use super::*;
fn quad(points: [[f32; 3]; 4]) -> Quad {
    Quad {
        points,
        normal: crate::mesh::face_normal(points),
        uv: [[0., 1.], [1., 1.], [1., 0.], [0., 0.]],
        vertex_colors: Some([[128, 64, 32], [64, 128, 32], [32, 64, 128], [128, 128, 128]]),
        material: Default::default(),
        face: 0,
        id: None,
    }
}
fn wall(z: [f32; 2], y: [f32; 2]) -> Quad {
    quad([
        [0., y[0], z[0]],
        [0., y[0], z[1]],
        [0., y[1], z[1]],
        [0., y[1], z[0]],
    ])
}
fn unpack(chunks: &[Chunk], face: usize) -> [[f32; 3]; 4] {
    let (chunk, indices) = chunks
        .iter()
        .find_map(|c| {
            c.faces
                .iter()
                .find(|(_, f)| *f == face)
                .map(|(v, _)| (c, v))
        })
        .unwrap();
    indices.map(|v| {
        std::array::from_fn(|a| chunk.origin[a] + chunk.vertices[usize::from(v)][a] as f32 / 4096.)
    })
}
fn verify_bounds(chunks: &[Chunk], faces: usize) {
    assert_eq!(chunks.iter().map(|c| c.faces.len()).sum::<usize>(), faces);
    for c in chunks {
        assert!(c.vertices.len() <= 512);
        for v in &c.vertices {
            for (a, &value) in v.iter().enumerate() {
                assert!(
                    (i64::from(value) - i64::from(c.center[a])).abs() <= i64::from(c.extent[a])
                );
            }
        }
    }
}
#[test]
fn seals_only_the_longer_coplanar_wall_without_changing_bake_layout() {
    let qs = vec![wall([0., 3.], [0., 6.]), wall([3., 6.], [0., 7.5])];
    let original = qs.clone();
    let chunks = crate::mesh_compile::chunks(&qs).unwrap();
    assert_eq!(unpack(&chunks, 0), qs[0].points);
    let mut expected = qs[1].points;
    expected[0][2] -= 0.5;
    expected[3][2] -= 0.5;
    assert_eq!(unpack(&chunks, 1), expected);
    for (a, b) in qs.iter().zip(original) {
        assert_eq!(a.points, b.points);
        assert_eq!(a.uv, b.uv);
        assert_eq!(a.vertex_colors, b.vertex_colors);
        assert_eq!(a.normal, b.normal);
        assert_eq!(a.material, b.material);
    }
    verify_bounds(&chunks, qs.len());
}
#[test]
fn leaves_matching_panels_open_edges_transparency_and_scrolling_unchanged() {
    let equal = vec![wall([0., 3.], [0., 6.]), wall([3., 6.], [0., 6.])];
    let mut blended = equal.clone();
    blended[1].points[2][1] = 7.5;
    blended[1].points[3][1] = 7.5;
    blended[1].material.blend = crate::texture::BlendMode::Add;
    let mut scroll = blended.clone();
    scroll[1].material.blend = crate::texture::BlendMode::Cutout;
    scroll[1].material.uv_scroll = [1., 0.];
    for qs in [equal, blended, scroll, vec![wall([0., 3.], [0., 6.])]] {
        let chunks = crate::mesh_compile::chunks(&qs).unwrap();
        for (i, q) in qs.iter().enumerate() {
            assert_eq!(unpack(&chunks, i), q.points);
        }
        verify_bounds(&chunks, qs.len());
    }
}
#[test]
fn closes_convex_floor_corners_without_growing_free_standing_props() {
    let a = wall([0., 3.], [0., 6.]);
    let b = quad([[0., 0., 3.], [3., 0., 3.], [3., 6., 3.], [0., 6., 3.]]);
    let floor = quad([[-3., 0., 3.], [-3., 0., 6.], [0., 0., 6.], [0., 0., 3.]]);
    let untouched = crate::mesh_compile::chunks(&[a.clone(), b.clone()]).unwrap();
    assert_eq!(unpack(&untouched, 0), a.points);
    assert_eq!(unpack(&untouched, 1), b.points);
    let qs = [a, b, floor];
    let chunks = crate::mesh_compile::chunks(&qs).unwrap();
    assert_eq!(unpack(&chunks, 0)[1][2], 3.125);
    assert_eq!(unpack(&chunks, 1)[0][0], -0.125);
    let mut expected = qs[2].points;
    expected[3] = [0.125, 0., 2.875];
    assert_eq!(unpack(&chunks, 2), expected);
    verify_bounds(&chunks, qs.len());
}
#[test]
fn keeps_all_floor_copies_of_a_join_coincident_and_excludes_inside_floors() {
    let a = wall([0., 3.], [0., 6.]);
    let b = quad([[0., 0., 3.], [3., 0., 3.], [3., 6., 3.], [0., 6., 3.]]);
    let outside = quad([[-3., 0., 3.], [-3., 0., 6.], [0., 0., 6.], [0., 0., 3.]]);
    let adjacent = quad([[0., 0., 3.], [0., 0., 6.], [3., 0., 6.], [3., 0., 3.]]);
    let qs = [a.clone(), b.clone(), outside, adjacent];
    let chunks = crate::mesh_compile::chunks(&qs).unwrap();
    assert_eq!(unpack(&chunks, 2)[3], unpack(&chunks, 3)[0]);
    assert_eq!(unpack(&chunks, 2)[3], [0.125, 0., 2.875]);
    verify_bounds(&chunks, qs.len());
    // A bottom face enclosed behind both outward walls is not an external
    // floor junction and must not grow the prop's silhouette.
    let inside = quad([[0., 0., 0.], [0., 0., 3.], [3., 0., 3.], [3., 0., 0.]]);
    let qs = [a, b, inside];
    let chunks = crate::mesh_compile::chunks(&qs).unwrap();
    for (i, q) in qs.iter().enumerate() {
        assert_eq!(unpack(&chunks, i), q.points);
    }
}

#[test]
fn caps_coplanar_overlap_to_a_quarter_of_narrow_panels() {
    let qs = [wall([0., 0.5], [0., 6.]), wall([0.5, 1.], [0., 7.5])];
    let chunks = crate::mesh_compile::chunks(&qs).unwrap();
    assert_eq!(unpack(&chunks, 0), qs[0].points);
    let mut expected = qs[1].points;
    expected[0][2] -= 0.125;
    expected[3][2] -= 0.125;
    assert_eq!(unpack(&chunks, 1), expected);
    verify_bounds(&chunks, qs.len());
}
