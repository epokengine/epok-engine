//! Geometry of the World2D content nodes (World2D initiative, phase 3): Path2D curve
//! sampling and lengths, Polygon2D triangulation, and the stateless particle preview the
//! 2D view draws. The console receives the sampled polylines and the triangles; the editor
//! draws exactly the same shapes.
use crate::world2d::{EmissionShape, Particles2D, Path2D};

/// Segments a curved path segment is sampled into.
pub const CURVE_STEPS: usize = 12;

fn cubic(p0: [f32; 2], p1: [f32; 2], p2: [f32; 2], p3: [f32; 2], t: f32) -> [f32; 2] {
    let u = 1. - t;
    [0, 1].map(|k| {
        u * u * u * p0[k] + 3. * u * u * t * p1[k] + 3. * u * t * t * p2[k] + t * t * t * p3[k]
    })
}
/// The path as a polyline in local pixels: straight segments stay two points, curved
/// ones are sampled. A closed path ends where it started.
pub fn sample_path(path: &Path2D) -> Vec<[f32; 2]> {
    let mut out = Vec::new();
    let count = path.points.len();
    if count == 0 {
        return out;
    }
    out.push(path.points[0].position);
    let segments = if path.closed && count > 2 {
        count
    } else {
        count - 1
    };
    for i in 0..segments {
        let a = &path.points[i];
        let b = &path.points[(i + 1) % count];
        if a.handle_out == [0., 0.] && b.handle_in == [0., 0.] {
            out.push(b.position);
            continue;
        }
        let c1 = [
            a.position[0] + a.handle_out[0],
            a.position[1] + a.handle_out[1],
        ];
        let c2 = [
            b.position[0] + b.handle_in[0],
            b.position[1] + b.handle_in[1],
        ];
        for step in 1..=CURVE_STEPS {
            out.push(cubic(
                a.position,
                c1,
                c2,
                b.position,
                step as f32 / CURVE_STEPS as f32,
            ));
        }
    }
    out
}
/// Distance from the first point to every point of a polyline.
pub fn cumulative_lengths(points: &[[f32; 2]]) -> Vec<f32> {
    let mut total = 0.;
    let mut out = Vec::with_capacity(points.len());
    for (i, p) in points.iter().enumerate() {
        if i > 0 {
            let q = points[i - 1];
            total += ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2)).sqrt();
        }
        out.push(total);
    }
    out
}
/// The point and unit direction at `distance` along a polyline (clamped, or wrapped when
/// `closed`), mirroring `Path2DComponent::sample_local`.
pub fn sample_at(
    points: &[[f32; 2]],
    lengths: &[f32],
    distance: f32,
    closed: bool,
) -> ([f32; 2], [f32; 2]) {
    if points.is_empty() {
        return ([0.; 2], [1., 0.]);
    }
    let total = *lengths.last().unwrap_or(&0.);
    if points.len() == 1 || total <= 0. {
        return (points[0], [1., 0.]);
    }
    let d = if closed {
        distance.rem_euclid(total)
    } else {
        distance.clamp(0., total)
    };
    let mut i = lengths
        .partition_point(|l| *l <= d)
        .max(1)
        .min(points.len() - 1);
    while i > 1 && lengths[i - 1] > d {
        i -= 1;
    }
    let (a, b) = (points[i - 1], points[i]);
    let span = lengths[i] - lengths[i - 1];
    let t = if span > 0. {
        (d - lengths[i - 1]) / span
    } else {
        0.
    };
    let dir = [b[0] - a[0], b[1] - a[1]];
    let length = (dir[0] * dir[0] + dir[1] * dir[1]).sqrt();
    (
        [a[0] + dir[0] * t, a[1] + dir[1] * t],
        if length > 0. {
            [dir[0] / length, dir[1] / length]
        } else {
            [1., 0.]
        },
    )
}
/// Automatic curve handles through every point (a Catmull-Rom style tangent a third of
/// the way to the neighbours), for "Smooth curve".
pub fn smooth(path: &mut Path2D) {
    let count = path.points.len();
    for i in 0..count {
        let prev = if i > 0 {
            Some(path.points[i - 1].position)
        } else if path.closed && count > 2 {
            Some(path.points[count - 1].position)
        } else {
            None
        };
        let next = if i + 1 < count {
            Some(path.points[i + 1].position)
        } else if path.closed && count > 2 {
            Some(path.points[0].position)
        } else {
            None
        };
        let here = path.points[i].position;
        let (Some(prev), Some(next)) = (prev.or(Some(here)), next.or(Some(here))) else {
            continue;
        };
        let tangent = [(next[0] - prev[0]) / 6., (next[1] - prev[1]) / 6.];
        path.points[i].handle_out = tangent;
        path.points[i].handle_in = [-tangent[0], -tangent[1]];
    }
}

/// Twice the signed area: positive for counter-clockwise points in a Y-up frame, so
/// positive here (Y down) means clockwise on screen.
fn signed_area(points: &[[f32; 2]]) -> f32 {
    let n = points.len();
    (0..n)
        .map(|i| {
            let (a, b) = (points[i], points[(i + 1) % n]);
            a[0] * b[1] - b[0] * a[1]
        })
        .sum()
}
fn cross(o: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])
}
fn inside_triangle(p: [f32; 2], a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> bool {
    let (d1, d2, d3) = (cross(a, b, p), cross(b, c, p), cross(c, a, p));
    let negative = d1 < 0. || d2 < 0. || d3 < 0.;
    let positive = d1 > 0. || d2 > 0. || d3 > 0.;
    !(negative && positive)
}
/// Whether two edges that share no corner intersect.
fn crosses_itself(points: &[[f32; 2]]) -> bool {
    let n = points.len();
    let edge = |i: usize| (points[i], points[(i + 1) % n]);
    for i in 0..n {
        for j in i + 1..n {
            if j == i + 1 || (i == 0 && j == n - 1) {
                continue; // neighbours share a corner
            }
            let ((a, b), (c, d)) = (edge(i), edge(j));
            let (d1, d2) = (cross(a, b, c), cross(a, b, d));
            let (d3, d4) = (cross(c, d, a), cross(c, d, b));
            if ((d1 > 0. && d2 < 0.) || (d1 < 0. && d2 > 0.))
                && ((d3 > 0. && d4 < 0.) || (d3 < 0. && d4 > 0.))
            {
                return true;
            }
        }
    }
    false
}
/// Ear-clipping triangulation of a simple polygon (either winding). Fails for fewer than
/// three points or a self-intersecting outline.
pub fn triangulate(points: &[[f32; 2]]) -> Result<Vec<[u16; 3]>, String> {
    let n = points.len();
    if n < 3 {
        return Err("a polygon needs at least three points".into());
    }
    if n > 256 {
        return Err("a polygon holds at most 256 points".into());
    }
    let area = signed_area(points);
    if area.abs() < 1e-6 {
        return Err("the polygon has no area".into());
    }
    if crosses_itself(points) {
        return Err("the polygon's outline crosses itself".into());
    }
    let winding = area.signum();
    let mut remaining: Vec<usize> = (0..n).collect();
    let mut out = Vec::new();
    let mut guard = 0;
    while remaining.len() > 3 {
        guard += 1;
        if guard > n * n {
            return Err("the polygon's outline crosses itself".into());
        }
        let m = remaining.len();
        let mut clipped = false;
        for i in 0..m {
            let (ia, ib, ic) = (
                remaining[(i + m - 1) % m],
                remaining[i],
                remaining[(i + 1) % m],
            );
            let (a, b, c) = (points[ia], points[ib], points[ic]);
            if cross(a, b, c) * winding <= 0. {
                continue; // reflex or degenerate corner
            }
            let ear = remaining
                .iter()
                .filter(|j| ![ia, ib, ic].contains(j))
                .all(|j| !inside_triangle(points[*j], a, b, c));
            if ear {
                out.push([ia as u16, ib as u16, ic as u16]);
                remaining.remove(i);
                clipped = true;
                break;
            }
        }
        if !clipped {
            return Err("the polygon's outline crosses itself".into());
        }
    }
    out.push([
        remaining[0] as u16,
        remaining[1] as u16,
        remaining[2] as u16,
    ]);
    Ok(out)
}

/// One particle of the editor preview: emitter-local position, age fraction.
pub struct PreviewParticle {
    pub position: [f32; 2],
    pub age: f32,
}
fn hash(a: u32, b: u32, salt: u32) -> f32 {
    let mut h =
        a.wrapping_mul(0x9e37_79b1) ^ b.wrapping_mul(0x85eb_ca77) ^ salt.wrapping_mul(0xc2b2_ae3d);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    (h & 0xffff) as f32 / 65536.
}
/// The particles alive at `seconds` of an endlessly repeating emitter, computed without
/// state so the 2D view can preview any frame (damping is approximated).
pub fn preview(p: &Particles2D, seconds: f32) -> Vec<PreviewParticle> {
    let amount = usize::from(p.amount.max(1)).min(crate::world2d::MAX_PARTICLES);
    let life = p.lifetime.max(0.05);
    let burst = p.explosiveness >= 1.;
    let mut out = Vec::new();
    for k in 0..amount {
        let offset = if burst {
            0.
        } else {
            life * k as f32 / amount as f32
        };
        let local = seconds - offset;
        if local < 0. {
            continue;
        }
        let cycle = (local / life).floor();
        if p.one_shot && cycle >= 1. {
            continue;
        }
        let age = local - cycle * life;
        let seed = cycle as u32;
        let r = |salt: u32| hash(k as u32, seed, salt ^ p.seed);
        let mut start = [0., 0.];
        match p.shape {
            EmissionShape::Point => {}
            EmissionShape::Rectangle => {
                start = [
                    (r(1) * 2. - 1.) * p.extents[0],
                    (r(2) * 2. - 1.) * p.extents[1],
                ];
            }
            EmissionShape::Circle => {
                let (s, c) = (r(3) * std::f32::consts::TAU).sin_cos();
                let radius = r(4) * p.extents[0];
                start = [c * radius, s * radius];
            }
        }
        let angle = (p.direction + (r(5) * 2. - 1.) * p.spread).to_radians();
        let speed = p.speed[0] + (p.speed[1] - p.speed[0]) * r(6);
        let keep = (1. - p.damping * age * 0.5).max(0.);
        let velocity = [angle.cos() * speed * keep, angle.sin() * speed * keep];
        out.push(PreviewParticle {
            position: [
                start[0] + velocity[0] * age + 0.5 * p.gravity[0] * age * age,
                start[1] + velocity[1] * age + 0.5 * p.gravity[1] * age * age,
            ],
            age: age / life,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world2d::PathPoint;

    #[test]
    fn straight_and_curved_paths_sample_and_measure() {
        let mut path = Path2D {
            points: vec![
                PathPoint::at([0., 0.]),
                PathPoint::at([30., 40.]),
                PathPoint::at([30., 100.]),
            ],
            closed: false,
        };
        let points = sample_path(&path);
        assert_eq!(points.len(), 3);
        let lengths = cumulative_lengths(&points);
        assert_eq!(lengths, vec![0., 50., 110.]);
        let (p, d) = sample_at(&points, &lengths, 80., false);
        assert_eq!(p, [30., 70.]);
        assert_eq!(d, [0., 1.]);
        assert_eq!(sample_at(&points, &lengths, 500., false).0, [30., 100.]);
        // A closed path returns to its start and wraps distances.
        path.closed = true;
        let closed = sample_path(&path);
        assert_eq!(closed.last(), Some(&[0., 0.]));
        let total = *cumulative_lengths(&closed).last().unwrap();
        assert!(
            (sample_at(&closed, &cumulative_lengths(&closed), total + 25., true).0[1] - 20.).abs()
                < 1e-4
        );
        // Handles bend a segment into a curve that still ends on its points.
        path.closed = false;
        smooth(&mut path);
        let curved = sample_path(&path);
        assert_eq!(curved.len(), 1 + 2 * CURVE_STEPS);
        assert_eq!(curved.last(), Some(&[30., 100.]));
        assert!(curved.iter().any(|p| p[0] > 30.5));
    }

    #[test]
    fn polygons_triangulate_in_either_winding_and_reject_crossings() {
        let square = [[0., 0.], [10., 0.], [10., 10.], [0., 10.]];
        assert_eq!(triangulate(&square).unwrap().len(), 2);
        let mut reversed = square;
        reversed.reverse();
        assert_eq!(triangulate(&reversed).unwrap().len(), 2);
        // A concave "L" needs four triangles, none covering the notch.
        let l = [
            [0., 0.],
            [20., 0.],
            [20., 10.],
            [10., 10.],
            [10., 20.],
            [0., 20.],
        ];
        let triangles = triangulate(&l).unwrap();
        assert_eq!(triangles.len(), 4);
        let area: f32 = triangles
            .iter()
            .map(|t| cross(l[t[0] as usize], l[t[1] as usize], l[t[2] as usize]).abs() / 2.)
            .sum();
        assert!((area - 300.).abs() < 1e-3);
        let bowtie = [[0., 0.], [10., 10.], [10., 0.], [0., 10.]];
        assert!(triangulate(&bowtie).is_err());
        assert!(triangulate(&square[..2]).is_err());
    }

    #[test]
    fn the_particle_preview_streams_bursts_and_ends_one_shots() {
        let stream = Particles2D {
            amount: 10,
            lifetime: 1.,
            gravity: [0., 0.],
            ..Default::default()
        };
        assert_eq!(preview(&stream, 0.55).len(), 6);
        assert_eq!(preview(&stream, 5.).len(), 10);
        let burst = Particles2D {
            explosiveness: 1.,
            one_shot: true,
            ..stream.clone()
        };
        assert_eq!(preview(&burst, 0.2).len(), 10);
        assert!(preview(&burst, 1.5).is_empty());
        // Launched upwards (direction -90) without gravity: every particle rises.
        assert!(preview(&stream, 0.5).iter().all(|p| p.position[1] <= 0.));
    }
}
