//! Editable 3D paths and the mesh swept along them. All geometry is cooked from
//! local-space control points, including the return to the first ring on a circuit.
use crate::{
    lighting::{dot, sub, unit},
    mesh::{Document, Face, cross},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

type Vec3 = [f32; 3];
#[cfg(test)]
#[path = "spline_tests.rs"]
mod tests;
pub const MAX_POINTS: usize = 128;
pub const MAX_STEPS: u8 = 24;
const MAX_FACES: usize = 3500;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub position: Vec3,
    #[serde(default)]
    pub bank: f32,
    /// Material and gameplay marker for the span starting here.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub surface: Option<Uuid>,
    #[serde(default)]
    pub tag: u8,
}
impl From<Vec3> for Point {
    fn from(position: Vec3) -> Self {
        Self {
            position,
            bank: 0.,
            surface: None,
            tag: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Profile {
    Road,
    Railing,
    Wall,
}
impl Profile {
    pub fn dimensions(self) -> (f32, f32) {
        match self {
            Self::Road => (3., 0.),
            Self::Railing => (0.18, 1.2),
            Self::Wall => (0.35, 2.5),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Spline {
    pub points: Vec<Point>,
    pub closed: bool,
    pub profile: Profile,
    pub width: f32,
    pub height: f32,
    pub post_spacing: f32,
    pub steps: u8,
    /// Unique within the map. Opt in to cooking a gameplay sampling table.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gameplay_channel: Option<u8>,
}

impl Default for Spline {
    fn default() -> Self {
        Self {
            points: vec![[0., 0., -3.].into(), [0., 0., 3.].into()],
            closed: false,
            profile: Profile::Road,
            width: Profile::Road.dimensions().0,
            height: 0.,
            post_spacing: 2.,
            steps: 6,
            gameplay_channel: None,
        }
    }
}

fn add(a: Vec3, b: Vec3) -> Vec3 {
    std::array::from_fn(|i| a[i] + b[i])
}
fn scale(v: Vec3, s: f32) -> Vec3 {
    v.map(|x| x * s)
}
fn length(v: Vec3) -> f32 {
    dot(v, v).sqrt()
}
fn lerp(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    add(a, scale(sub(b, a), t))
}
fn signed_angle(from: Vec3, to: Vec3, axis: Vec3) -> f32 {
    dot(axis, cross(from, to)).atan2(dot(from, to))
}
fn bank_at(from: f32, to: f32, t: f32) -> f32 {
    let mut delta = to - from;
    if delta.abs() > 180. {
        delta -= 360. * delta.signum();
    }
    let angle = from + t * delta;
    if angle.abs() > 180. {
        angle - 360. * angle.signum()
    } else {
        angle
    }
}
fn rotate(v: Vec3, axis: Vec3, angle: f32) -> Vec3 {
    let (s, c) = angle.sin_cos();
    add(
        add(scale(v, c), scale(cross(axis, v), s)),
        scale(axis, dot(axis, v) * (1. - c)),
    )
}

#[derive(Clone, Copy)]
pub struct Frame {
    pub center: Vec3,
    pub tangent: Vec3,
    pub right: Vec3,
    pub up: Vec3,
    pub distance: f32,
}
impl Frame {
    fn point(self, x: f32, y: f32, z: f32) -> Vec3 {
        add(
            self.center,
            add(
                scale(self.right, x),
                add(scale(self.up, y), scale(self.tangent, z)),
            ),
        )
    }
}

impl Spline {
    pub fn circuit() -> Vec<Point> {
        (0..8)
            .map(|n| {
                let a = std::f32::consts::TAU * n as f32 / 8.;
                [8. * a.sin(), 0., -12. * a.cos()].into()
            })
            .collect()
    }

    /// The entry and exit lanes are offset so they can pass beside each other.
    pub fn vertical_loop() -> Vec<Point> {
        let mut points = vec![[0., 0., -7.].into()];
        for n in 0..=8 {
            let t = n as f32 / 8.;
            let a = t * std::f32::consts::TAU;
            points.push(
                [
                    5. * t * t * (3. - 2. * t),
                    5. * (1. - a.cos()),
                    5. * a.sin(),
                ]
                .into(),
            );
        }
        points.push([5., 0., 7.].into());
        // The lateral offset introduces torsion. Bank the starter's control
        // points so the ribbon faces inward and the exit returns to level.
        let mut curve = Self {
            points,
            ..Self::default()
        };
        let frames = curve
            .frames()
            .expect("the built-in loop has distinct control points");
        for (point, frame) in curve
            .points
            .iter_mut()
            .zip(frames.iter().step_by(usize::from(curve.steps)))
        {
            let lateral = sub([1., 0., 0.], scale(frame.tangent, frame.tangent[0]));
            point.bank = signed_angle(frame.right, lateral, frame.tangent).to_degrees();
        }
        curve.points
    }

    pub fn spans(&self) -> usize {
        if self.closed {
            self.points.len()
        } else {
            self.points.len().saturating_sub(1)
        }
    }

    fn validate(&self) -> Result<(), String> {
        let min = if self.closed { 3 } else { 2 };
        if !(min..=MAX_POINTS).contains(&self.points.len()) {
            return Err(format!(
                "A spline needs {min} to {MAX_POINTS} control points."
            ));
        }
        if !(1..=MAX_STEPS).contains(&self.steps)
            || !(0.04..=64.).contains(&self.width)
            || !(0.0..=64.).contains(&self.height)
            || (self.profile != Profile::Road && self.height < 0.08)
            || !(0.1..=128.).contains(&self.post_spacing)
        {
            return Err("Check the spline dimensions, post spacing and subdivisions.".into());
        }
        for (i, p) in self.points.iter().enumerate() {
            if p.position.iter().any(|c| !c.is_finite() || c.abs() > 128.)
                || !(-180. ..=180.).contains(&p.bank)
            {
                return Err(format!(
                    "Point {} exceeds local ±128 units or ±180 degrees of bank.",
                    i + 1
                ));
            }
            if i < self.spans()
                && length(sub(
                    self.points[(i + 1) % self.points.len()].position,
                    p.position,
                )) < 0.02
            {
                return Err(format!("Separate the adjacent points at point {}.", i + 1));
            }
        }
        Ok(())
    }

    /// Cubic Hermite with centripetal Catmull–Rom tangents. Extrapolating the
    /// missing neighbours gives an open path a usable tangent at both ends.
    fn segment(&self, i: usize) -> (Vec3, Vec3, Vec3, Vec3) {
        let n = self.points.len();
        let p = |k: usize| self.points[k % n].position;
        let (a, b) = (p(i), p(i + 1));
        let before = if i == 0 && !self.closed {
            sub(scale(a, 2.), b)
        } else {
            p(i + n - 1)
        };
        let after = if i + 2 == n && !self.closed {
            sub(scale(b, 2.), a)
        } else {
            p(i + 2)
        };
        let tangent = |prev, at, next| {
            let l = length(sub(at, prev)).sqrt();
            let r = length(sub(next, at)).sqrt();
            add(
                scale(sub(at, prev), r / (l * (l + r))),
                scale(sub(next, at), l / (r * (l + r))),
            )
        };
        let duration = length(sub(b, a)).sqrt();
        (
            a,
            b,
            scale(tangent(before, a, b), duration),
            scale(tangent(a, b, after), duration),
        )
    }

    pub fn midpoint(&self, span: usize) -> Point {
        let (a, b, m, n) = self.segment(span);
        Point {
            position: add(lerp(a, b, 0.5), scale(sub(m, n), 0.125)),
            bank: bank_at(
                self.points[span].bank,
                self.points[(span + 1) % self.points.len()].bank,
                0.5,
            ),
            ..self.points[span]
        }
    }

    pub fn frames(&self) -> Result<Vec<Frame>, String> {
        self.validate()?;
        let steps = usize::from(self.steps);
        let mut frames: Vec<Frame> = Vec::with_capacity(self.spans() * steps + 1);
        let mut banks = Vec::with_capacity(frames.capacity());
        for i in 0..self.spans() {
            let (a, b, m, n) = self.segment(i);
            let delta = sub(b, a);
            let c = sub(scale(delta, 3.), add(scale(m, 2.), n));
            let d = sub(add(m, n), scale(delta, 2.));
            for k in usize::from(i != 0)..=steps {
                let t = k as f32 / steps as f32;
                let center = if k == steps {
                    b
                } else {
                    add(a, scale(add(m, scale(add(c, scale(d, t)), t)), t))
                };
                let tangent = unit(if k == steps {
                    n
                } else {
                    add(m, scale(add(scale(c, 2.), scale(d, 3. * t)), t))
                });
                if length(tangent) < 0.5 {
                    return Err(format!(
                        "The spline reverses at span {}. Add a point around the turn.",
                        i + 1
                    ));
                }
                let (right, distance) = if let Some(prev) = frames.last() {
                    let axis = cross(prev.tangent, tangent);
                    let sine = length(axis);
                    let cosine = dot(prev.tangent, tangent);
                    if cosine < -0.9999 {
                        return Err(format!(
                            "Increase subdivisions or soften the turn at span {}.",
                            i + 1
                        ));
                    }
                    // Parallel transport keeps a horizontal track upright through
                    // a vertical tangent and upside down at the top of a loop.
                    let rotated = if sine > 1e-6 {
                        rotate(prev.right, scale(axis, 1. / sine), sine.atan2(cosine))
                    } else {
                        prev.right
                    };
                    (
                        unit(sub(rotated, scale(tangent, dot(tangent, rotated)))),
                        prev.distance + length(sub(center, prev.center)),
                    )
                } else {
                    let reference = if tangent[1].abs() > 0.999 {
                        [0., 0., 1.]
                    } else {
                        [0., 1., 0.]
                    };
                    (unit(cross(reference, tangent)), 0.)
                };
                frames.push(Frame {
                    center,
                    tangent,
                    right,
                    up: [0.; 3],
                    distance,
                });
                banks.push(
                    bank_at(
                        self.points[i].bank,
                        self.points[(i + 1) % self.points.len()].bank,
                        t,
                    )
                    .to_radians(),
                );
            }
        }
        let first = frames[0];
        let last = *frames.last().unwrap();
        // In 3D the transported axes can return rotated. Distribute the
        // residual roll by travelled distance instead of twisting the seam.
        let twist = if self.closed {
            signed_angle(last.right, first.right, first.tangent)
        } else {
            0.
        };
        for (frame, bank) in frames.iter_mut().zip(banks) {
            frame.right = rotate(
                frame.right,
                frame.tangent,
                bank + twist * frame.distance / last.distance,
            );
            frame.up = unit(cross(frame.tangent, frame.right));
        }
        if self.closed {
            let first = frames[0];
            *frames.last_mut().unwrap() = Frame {
                distance: last.distance,
                ..first
            };
        }
        Ok(frames)
    }

    /// Replace only the derived geometry. The group and material identities
    /// belong to the asset and remain valid for per-instance material overrides.
    pub fn apply(self, doc: &mut Document) -> Result<(), String> {
        let group = doc.groups.first().ok_or("A spline needs a mesh group.")?.id;
        let slot = doc
            .materials
            .first()
            .ok_or("A spline needs a material slot.")?
            .id;
        if self
            .points
            .iter()
            .filter_map(|p| p.surface)
            .any(|id| !doc.materials.iter().any(|s| s.id == id))
        {
            return Err("Spline span references a missing material slot.".into());
        }
        let frames = self.frames()?;
        let spans = frames.len() - 1;
        let distance = frames[spans].distance;
        let intervals = (distance / self.post_spacing).ceil().max(1.) as usize;
        let posts = if self.profile == Profile::Railing {
            intervals + usize::from(!self.closed)
        } else {
            0
        };
        let solid = self.profile != Profile::Road;
        let face_count = spans * (if solid { 8 } else { 2 })
            + 2 * usize::from(solid && !self.closed)
            + posts * 6;
        if face_count > MAX_FACES {
            return Err(format!(
                "Spline needs {face_count} faces (limit {MAX_FACES}). Reduce subdivisions or increase post spacing."
            ));
        }
        let mut mesh = Sweep {
            group,
            slot,
            vertices: Vec::new(),
            faces: Vec::with_capacity(face_count),
        };
        let half = self.width * 0.5;
        let (low, high) = match self.profile {
            Profile::Road => (0., 0.),
            Profile::Wall => (0., self.height),
            Profile::Railing => (self.height - self.width.min(self.height * 0.3), self.height),
        };
        let corners = if solid { 4 } else { 2 };
        let rings = frames.len() - usize::from(self.closed);
        for frame in &frames[..rings] {
            for (x, y) in [(-half, high), (half, high), (half, low), (-half, low)]
                .into_iter()
                .take(corners)
            {
                mesh.vertices.push(frame.point(x, y, 0.));
            }
        }
        for i in 0..spans {
            mesh.slot = self.points[i / usize::from(self.steps)]
                .surface
                .unwrap_or(slot);
            let a = i * corners;
            let b = (i + 1) % rings * corners;
            for j in 0..if solid { 4 } else { 1 } {
                let next = (j + 1) % corners;
                // Four corners spanning two banked sections need not be coplanar.
                mesh.strip([a + j, b + j, b + next, a + next]);
            }
        }
        if solid && !self.closed {
            mesh.slot = self.points[0].surface.unwrap_or(slot);
            mesh.quad([0, 1, 2, 3]);
            mesh.slot = self.points[self.spans() - 1].surface.unwrap_or(slot);
            let end = spans * corners;
            mesh.quad([end + 3, end + 2, end + 1, end]);
        }
        for p in 0..posts {
            let at = distance * (p as f32 / intervals as f32);
            let end = frames
                .partition_point(|frame| frame.distance < at)
                .clamp(1, spans);
            let (a, b) = (frames[end - 1], frames[end]);
            let t = (at - a.distance) / (b.distance - a.distance);
            let tangent = unit(lerp(a.tangent, b.tangent, t));
            let up = unit(cross(tangent, lerp(a.right, b.right, t)));
            let frame = Frame {
                center: lerp(a.center, b.center, t),
                tangent,
                up,
                right: cross(up, tangent),
                distance: at,
            };
            mesh.slot = self.points[(end - 1) / usize::from(self.steps)]
                .surface
                .unwrap_or(slot);
            mesh.post(frame, half * 0.8, low);
        }
        doc.vertices = mesh.vertices;
        doc.faces = mesh.faces;
        doc.spline = Some(self);
        Ok(())
    }
}

struct Sweep {
    group: Uuid,
    slot: Uuid,
    vertices: Vec<Vec3>,
    faces: Vec<Face>,
}
impl Sweep {
    fn face(&mut self, corners: [usize; 4], uv: [[f32; 2]; 4]) {
        self.faces.push(Face {
            id: crate::dungeon::derived_id(
                self.group,
                &format!("spline-face-{}", self.faces.len()),
            ),
            vertices: corners.map(|i| i as u32),
            group: self.group,
            material: self.slot,
            uv,
            vertex_colors: None,
        });
    }
    fn strip(&mut self, [a, b, c, d]: [usize; 4]) {
        self.face([a, b, c, c], [[0., 0.], [0., 1.], [1., 1.], [1., 1.]]);
        self.face([a, c, d, d], [[0., 0.], [1., 1.], [1., 0.], [1., 0.]]);
    }
    fn quad(&mut self, corners: [usize; 4]) {
        self.face(corners, [[0., 0.], [0., 1.], [1., 1.], [1., 0.]]);
    }
    fn post(&mut self, frame: Frame, half: f32, high: f32) {
        let base = self.vertices.len();
        for z in [-half, half] {
            for (x, y) in [(-half, high), (half, high), (half, 0.), (-half, 0.)] {
                self.vertices.push(frame.point(x, y, z));
            }
        }
        self.quad([base, base + 1, base + 2, base + 3]);
        self.quad([base + 7, base + 6, base + 5, base + 4]);
        for i in 0..4 {
            let j = (i + 1) % 4;
            self.quad([base + i, base + 4 + i, base + 4 + j, base + j]);
        }
    }
}
