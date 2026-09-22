//! Geometry queries used by the navigation baker. Mesh faces are opt-in.
use crate::{
    collision::Aabb,
    mesh::{cross, face_normal},
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Triangle {
    pub points: [[f32; 3]; 3],
}
impl Triangle {
    pub fn normal(&self) -> [f32; 3] {
        face_normal([
            self.points[0],
            self.points[1],
            self.points[2],
            self.points[2],
        ])
    }
    pub fn height(&self, x: f32, z: f32) -> Option<f32> {
        let [a, b, c] = self.points;
        let d = (b[2] - c[2]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[2] - c[2]);
        if d.abs() < 1e-8 {
            return None;
        }
        let u = ((b[2] - c[2]) * (x - c[0]) + (c[0] - b[0]) * (z - c[2])) / d;
        let v = ((c[2] - a[2]) * (x - c[0]) + (a[0] - c[0]) * (z - c[2])) / d;
        (u >= -1e-5 && v >= -1e-5 && u + v <= 1.00001)
            .then_some(u * a[1] + v * b[1] + (1. - u - v) * c[1])
    }
    // Separating-axis triangle/AABB test; catches vertical and thin mesh walls.
    pub fn intersects(&self, b: &Aabb) -> bool {
        let center = std::array::from_fn::<_, 3, _>(|k| (b.min[k] + b.max[k]) * 0.5);
        let half = std::array::from_fn::<_, 3, _>(|k| (b.max[k] - b.min[k]) * 0.5);
        let p = self
            .points
            .map(|p| std::array::from_fn::<_, 3, _>(|k| p[k] - center[k]));
        let edges = [sub(p[1], p[0]), sub(p[2], p[1]), sub(p[0], p[2])];
        let axes = [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]];
        let separated = |a: [f32; 3]| {
            let r = (0..3).map(|k| a[k].abs() * half[k]).sum::<f32>();
            let d = p.map(|p| dot(p, a));
            d.into_iter().fold(f32::INFINITY, f32::min) > r + 1e-5
                || d.into_iter().fold(f32::NEG_INFINITY, f32::max) < -r - 1e-5
        };
        !axes
            .into_iter()
            .chain([cross(edges[0], edges[1])])
            .chain(edges.into_iter().flat_map(|e| axes.map(|a| cross(e, a))))
            .any(separated)
    }
}
fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|k| a[k] - b[k])
}
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    (0..3).map(|k| a[k] * b[k]).sum()
}

pub fn quad(p: [[f32; 3]; 4], out: &mut Vec<Triangle>) {
    out.push(Triangle {
        points: [p[0], p[1], p[2]],
    });
    if p[2] != p[3] {
        out.push(Triangle {
            points: [p[0], p[2], p[3]],
        });
    }
}
