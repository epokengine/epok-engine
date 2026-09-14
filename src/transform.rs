//! Affine transforms preserve inherited non-uniform scale, including shear.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Matrix(pub [[f32; 4]; 3]);
impl Matrix {
    pub const IDENTITY: Self = Self([[1., 0., 0., 0.], [0., 1., 0., 0.], [0., 0., 1., 0.]]);
    pub fn trs(position: [f32; 3], rotation: [f32; 3], scale: [f32; 3]) -> Self {
        let mut m = Self::IDENTITY;
        for axis in 0..3 {
            let mut v = [0.; 3];
            v[axis] = scale[axis];
            let v = crate::viewport::rotate(v, rotation);
            for (row, value) in v.into_iter().enumerate() {
                m.0[row][axis] = value;
            }
        }
        for (row, value) in position.into_iter().enumerate() {
            m.0[row][3] = value;
        }
        m
    }
    pub fn point(self, p: [f32; 3]) -> [f32; 3] {
        std::array::from_fn(|r| self.0[r][3] + (0..3).map(|c| self.0[r][c] * p[c]).sum::<f32>())
    }
    pub fn vector(self, p: [f32; 3]) -> [f32; 3] {
        std::array::from_fn(|r| (0..3).map(|c| self.0[r][c] * p[c]).sum())
    }
    pub fn compose(self, b: Self) -> Self {
        let mut out = Self::IDENTITY;
        for r in 0..3 {
            for c in 0..4 {
                out.0[r][c] = (0..3).map(|k| self.0[r][k] * b.0[k][c]).sum::<f32>()
                    + if c == 3 { self.0[r][3] } else { 0. };
            }
        }
        out
    }
    pub fn inverse(self) -> Result<Self, String> {
        let a = self.0;
        let det = a[0][0] * (a[1][1] * a[2][2] - a[1][2] * a[2][1])
            - a[0][1] * (a[1][0] * a[2][2] - a[1][2] * a[2][0])
            + a[0][2] * (a[1][0] * a[2][1] - a[1][1] * a[2][0]);
        if !det.is_finite() || det.abs() < 1e-12 {
            return Err("Parent transform cannot be inverted".into());
        }
        let mut m = Self::IDENTITY;
        for r in 0..3 {
            for c in 0..3 {
                m.0[r][c] = (a[(c + 1) % 3][(r + 1) % 3] * a[(c + 2) % 3][(r + 2) % 3]
                    - a[(c + 1) % 3][(r + 2) % 3] * a[(c + 2) % 3][(r + 1) % 3])
                    / det;
            }
        }
        let t = m.vector([-a[0][3], -a[1][3], -a[2][3]]);
        for (r, v) in t.into_iter().enumerate() {
            m.0[r][3] = v;
        }
        Ok(m)
    }
    pub fn apply_trs(self, e: &mut crate::scene::Actor) -> Result<(), String> {
        let scale: [f32; 3] =
            std::array::from_fn(|c| (0..3).map(|r| self.0[r][c].powi(2)).sum::<f32>().sqrt());
        if scale.iter().any(|s| !s.is_finite() || *s < 1e-6) {
            return Err("Transform scale is too small".into());
        }
        let r: [[f32; 3]; 3] =
            std::array::from_fn(|row| std::array::from_fn(|c| self.0[row][c] / scale[c]));
        // Rz * Ry * Rx, matching the existing editor and PsyQo runtime.
        let y = (-r[2][0]).clamp(-1., 1.).asin();
        let (x, z) = if y.cos().abs() > 1e-5 {
            (r[2][1].atan2(r[2][2]), r[1][0].atan2(r[0][0]))
        } else {
            ((-r[1][2]).atan2(r[1][1]), 0.)
        };
        let p = [self.0[0][3], self.0[1][3], self.0[2][3]];
        let angles = [x.to_degrees(), y.to_degrees(), z.to_degrees()];
        let rebuilt = Self::trs(p, angles, scale);
        if (0..3).any(|row| {
            (0..3).any(|c| (rebuilt.0[row][c] - self.0[row][c]).abs() > 0.0002 * scale[c].max(1.))
        }) {
            return Err("Cannot preserve world transform with this non-uniform scale and rotation. Use Parent > Keep Local or a uniformly scaled parent.".into());
        }
        e.position = p;
        e.rotation = angles;
        e.scale = scale;
        Ok(())
    }
}
