use crate::scene::Scene;
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct View {
    pub yaw: f32,
    pub pitch: f32,
    pub center: [f32; 3],
    pub zoom: f32,
    /// Distance from the eye to the orbit pivot; independent of the lens zoom.
    pub distance: f32,
    pub phase: f32,
    pub fly_speed: f32,
}
impl Default for View {
    fn default() -> Self {
        Self {
            yaw: 0.65,
            pitch: 0.5576,
            center: [0.; 3],
            zoom: 0.85,
            distance: 12.,
            phase: 0.,
            fly_speed: 5.,
        }
    }
}

impl View {
    pub fn dolly(&mut self, steps: f32) {
        // Move the eye and its orbit pivot together, so the wheel can travel
        // past the old pivot without changing the lens or hitting a zoom limit.
        let forward = self.basis()[2];
        let travel = steps * self.fly_speed * 0.2;
        for (center, direction) in self.center.iter_mut().zip(forward) {
            *center += direction * travel;
        }
    }
    /// Zoom a framed asset around its pivot; Scene navigation uses free dolly.
    pub fn zoom_orbit(&mut self, steps: f32) {
        self.distance = (self.distance * 1.2_f32.powf(-steps)).clamp(1.05, 10000.);
    }
    /// Snap the authoring camera to look along one signed world axis.
    /// A small pole offset preserves a stable horizontal basis for subsequent orbiting.
    pub fn snap_to_axis(&mut self, axis: usize, direction: f32) {
        let sign = if direction.is_sign_negative() {
            -1.
        } else {
            1.
        };
        const POLE_OFFSET: f32 = 0.001;
        match axis {
            0 => {
                self.yaw = if sign > 0. {
                    std::f32::consts::FRAC_PI_2
                } else {
                    3. * std::f32::consts::FRAC_PI_2
                };
                self.pitch = 0.;
            }
            1 => {
                self.yaw = 0.;
                self.pitch = -sign * (std::f32::consts::FRAC_PI_2 - POLE_OFFSET);
            }
            2 => {
                self.yaw = if sign > 0. { 0. } else { std::f32::consts::PI };
                self.pitch = 0.;
            }
            _ => {}
        }
    }
    /// Return to a three-quarter authoring view without moving the orbit pivot.
    pub fn snap_to_isometric(&mut self) {
        let default = Self::default();
        self.yaw = default.yaw;
        self.pitch = default.pitch;
    }
    pub fn frame_bounds(&mut self, low: [f32; 3], high: [f32; 3]) {
        self.frame_bounds_in_panel(low, high, [960., 600.]);
    }
    pub fn frame_bounds_in_panel(&mut self, low: [f32; 3], high: [f32; 3], panel: [f32; 2]) {
        self.center = std::array::from_fn(|i| (low[i] + high[i]) * 0.5);
        let radius = (0..3)
            .map(|i| ((high[i] - low[i]) * 0.5).powi(2))
            .sum::<f32>()
            .sqrt()
            .max(0.1);
        // Fit a bounding sphere with padding and near-plane clearance.
        let panel = panel.map(|v| if v.is_finite() { v.max(1.) } else { 600. });
        let factor = (panel[0] / 960.).max(panel[1] / 600.);
        // Account for both the render texture and the panel's cropped region.
        let half_width = panel[0] / factor * 0.5;
        let lower_margin = panel[1] / factor * 0.5;
        let margin = half_width.min(lower_margin).max(1.) * 0.85;
        let cotangent = 870. * self.zoom / margin;
        self.distance = (radius * (1. + cotangent * cotangent).sqrt())
            .max(radius + 1.05)
            .clamp(1.05, 10000.);
    }
    pub fn basis(&self) -> [[f32; 3]; 3] {
        let (s, c) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        [[c, 0., -s], [s * sp, cp, c * sp], [s * cp, -sp, c * cp]]
    }
    pub fn pan(&mut self, delta: [f32; 2], factor: f32) {
        let [right, up, _] = self.basis();
        let units = self.distance / (870. * self.zoom * factor.max(0.01));
        for i in 0..3 {
            self.center[i] += (-right[i] * delta[0] + up[i] * delta[1]) * units;
        }
    }
    pub fn look(&mut self, delta: [f32; 2], orbit: bool) {
        let previous = self.basis()[2];
        self.yaw = (self.yaw + delta[0] * 0.003).rem_euclid(std::f32::consts::TAU);
        self.pitch = (self.pitch + delta[1] * 0.003).clamp(-1.54, 1.54);
        if !orbit {
            let next = self.basis()[2];
            for i in 0..3 {
                self.center[i] += (next[i] - previous[i]) * self.distance;
            }
        }
    }
    pub fn fly(&mut self, axes: [f32; 3], dt: f32, fast: bool) {
        let [right, _, forward] = self.basis();
        let length = axes.iter().map(|v| v * v).sum::<f32>().sqrt().max(1.);
        let step = dt.clamp(0., 0.1) * self.fly_speed * if fast { 4. } else { 1. } / length;
        for i in 0..3 {
            self.center[i] +=
                (right[i] * axes[0] + forward[i] * axes[2] + if i == 1 { axes[1] } else { 0. })
                    * step;
        }
    }
    pub fn unproject(&self, pixel: [f32; 2], depth: f32) -> [f32; 3] {
        let [right, up, forward] = self.basis();
        let x = (pixel[0] - 480.) * depth / (870. * self.zoom);
        let y = (300. - pixel[1]) * depth / (870. * self.zoom);
        std::array::from_fn(|i| {
            self.center[i] + right[i] * x + up[i] * y + forward[i] * (depth - self.distance)
        })
    }
}

struct Canvas {
    pixels: Vec<[u8; 3]>,
    depth: Vec<f32>,
    w: i32,
    h: i32,
}
impl Canvas {
    fn put_depth(&mut self, x: i32, y: i32, depth: f32, color: [u8; 3]) {
        if x >= 0 && x < self.w && y >= 0 && y < self.h && depth > 0. {
            let index = (y * self.w + x) as usize;
            if depth < self.depth[index] {
                self.depth[index] = depth;
                self.put(x, y, color);
            }
        }
    }
    fn put(&mut self, x: i32, y: i32, color: [u8; 3]) {
        if x >= 0 && x < self.w && y >= 0 && y < self.h {
            self.pixels[(y * self.w + x) as usize] = color;
        }
    }
    fn line(&mut self, a: [f32; 3], b: [f32; 3], color: [u8; 3]) {
        let steps = ((a[0] - b[0]).abs().max((a[1] - b[1]).abs()) as usize).clamp(1, 4000);
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            self.put_depth(
                (a[0] + (b[0] - a[0]) * t) as i32,
                (a[1] + (b[1] - a[1]) * t) as i32,
                1. / ((1. - t) / a[2] + t / b[2]) - 0.012,
                color,
            );
        }
    }
    fn triangle(&mut self, p: [[f32; 3]; 3], colors: [[u8; 3]; 3]) {
        let edge = |a: [f32; 3], b: [f32; 3], x: f32, y: f32| {
            (x - a[0]) * (b[1] - a[1]) - (y - a[1]) * (b[0] - a[0])
        };
        let area = edge(p[0], p[1], p[2][0], p[2][1]);
        if area.abs() < 0.01 {
            return;
        }
        let min_x = p
            .iter()
            .map(|v| v[0])
            .fold(f32::INFINITY, f32::min)
            .floor()
            .max(0.) as i32;
        let max_x = p
            .iter()
            .map(|v| v[0])
            .fold(f32::NEG_INFINITY, f32::max)
            .ceil()
            .min((self.w - 1) as f32) as i32;
        let min_y = p
            .iter()
            .map(|v| v[1])
            .fold(f32::INFINITY, f32::min)
            .floor()
            .max(0.) as i32;
        let max_y = p
            .iter()
            .map(|v| v[1])
            .fold(f32::NEG_INFINITY, f32::max)
            .ceil()
            .min((self.h - 1) as f32) as i32;
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let a = edge(p[0], p[1], x as f32 + 0.5, y as f32 + 0.5) / area;
                let b = edge(p[1], p[2], x as f32 + 0.5, y as f32 + 0.5) / area;
                let c = edge(p[2], p[0], x as f32 + 0.5, y as f32 + 0.5) / area;
                if a >= 0. && b >= 0. && c >= 0. {
                    let depth = 1. / (b / p[0][2] + c / p[1][2] + a / p[2][2]);
                    let color = std::array::from_fn(|i| {
                        (b * colors[0][i] as f32
                            + c * colors[1][i] as f32
                            + a * colors[2][i] as f32)
                            .clamp(0., 255.) as u8
                    });
                    self.put_depth(x, y, depth, color);
                }
            }
        }
    }
}

pub fn rotate(mut p: [f32; 3], angles: [f32; 3]) -> [f32; 3] {
    for (axis, degrees) in angles.into_iter().enumerate() {
        let (s, c) = degrees.to_radians().sin_cos();
        let a = (axis + 1) % 3;
        let b = (axis + 2) % 3;
        (p[a], p[b]) = (p[a] * c - p[b] * s, p[a] * s + p[b] * c);
    }
    p
}

pub fn project(view: &View, p: [f32; 3]) -> [f32; 3] {
    let (w, h) = (960, 600);
    let relative: [f32; 3] = std::array::from_fn(|i| p[i] - view.center[i]);
    let [right, up, forward] = view.basis();
    let dot = |v: [f32; 3]| (0..3).map(|i| v[i] * relative[i]).sum::<f32>();
    let x = dot(right);
    let y = dot(up);
    let depth = view.distance + dot(forward);
    let scale = h as f32 * 1.45 * view.zoom / depth.max(1.);
    [
        w as f32 * 0.5 + x * scale,
        h as f32 * 0.5 - y * scale,
        depth,
    ]
}

pub fn render(
    scene: &Scene,
    selected: Option<usize>,
    view: &View,
    grid: bool,
    wire: bool,
    game: bool,
) -> Image {
    let (w, h) = if game { (320, 240) } else { (960, 600) };
    let mut canvas = Canvas {
        pixels: vec![[0; 3]; (w * h) as usize],
        depth: vec![f32::INFINITY; (w * h) as usize],
        w: w as i32,
        h: h as i32,
    };
    for y in 0..h {
        for x in 0..w {
            let t = y as f32 / h as f32;
            canvas.put(
                x as i32,
                y as i32,
                [
                    (68. - t * 6.) as u8,
                    (68. - t * 6.) as u8,
                    (68. - t * 6.) as u8,
                ],
            );
        }
    }
    let project = |p| project(view, p);
    if grid && !game {
        for i in -10..=10 {
            canvas.line(
                project([i as f32, -0.26, -10.]),
                project([i as f32, -0.26, 10.]),
                [88, 88, 88],
            );
            canvas.line(
                project([-10., -0.26, i as f32]),
                project([10., -0.26, i as f32]),
                [88, 88, 88],
            );
        }
        canvas.line(
            project([-10., -0.25, 0.]),
            project([10., -0.25, 0.]),
            [127, 75, 75],
        );
        canvas.line(
            project([0., -0.25, -10.]),
            project([0., -0.25, 10.]),
            [71, 102, 139],
        );
    }
    let lighting = crate::lighting::Lighting::unshadowed(scene);
    let mut polygons = Vec::new();
    for (index, entity) in scene
        .actors
        .iter()
        .enumerate()
        .filter(|(i, e)| e.kind == "Mesh" && scene.is_active(*i))
    {
        let world = scene.world_matrix(index);
        let offline = crate::lighting::baked(entity);
        let quads = crate::lighting::quads(entity);
        let cached = scene
            .bake
            .as_ref()
            .and_then(|b| b.preview_colors(entity.id, quads.len() * 4));
        for (qi, q) in quads.iter().enumerate() {
            let world_points = q.points.map(|p| world.point(p));
            let p = world_points.map(project);
            if p.iter().any(|v| v[2] <= 1.) {
                continue;
            }
            if (entity.editable_mesh.is_some()
                || entity.terrain.is_some()
                || entity.skeletal_mesh.is_some())
                && !wire
            {
                let area = (p[1][0] - p[0][0]) * (p[2][1] - p[0][1])
                    - (p[1][1] - p[0][1]) * (p[2][0] - p[0][0]);
                if area <= 0. {
                    continue;
                }
            }
            let n = crate::lighting::transform_normal(world, q.normal);
            let colors: [[u8; 3]; 4] = std::array::from_fn(|v| {
                let light = if q.material.unlit {
                    [255; 3]
                } else if offline && let Some(colors) = cached {
                    colors[qi * 4 + v]
                } else if offline {
                    lighting.sample(world_points[v], n, index, true, false)
                } else {
                    lighting.sample(world.point([0.; 3]), n, index, false, false)
                };
                crate::lighting::modulate(light, q.material.color)
            });
            polygons.push((
                p.iter().map(|v| v[2]).sum::<f32>() / 4.,
                p,
                colors,
                selected == Some(index),
            ));
        }
    }
    polygons.sort_by(|a, b| b.0.total_cmp(&a.0));
    for (_, p, color, selected) in polygons {
        if !wire {
            canvas.triangle([p[0], p[1], p[2]], [color[0], color[1], color[2]]);
            canvas.triangle([p[0], p[2], p[3]], [color[0], color[2], color[3]]);
        }
        let outline = if selected && !game {
            [238, 165, 74]
        } else {
            [79, 94, 111]
        };
        if wire || (selected && !game) {
            for i in 0..4 {
                canvas.line(p[i], p[(i + 1) % 4], outline);
            }
        }
    }
    if !game
        && let Some(index) = selected
        && let Some(entity) = scene.actors.get(index)
        && let Some(collider) = &entity.collider
        && collider.enabled
    {
        let color = if collider.trigger {
            [240, 190, 60]
        } else {
            [60, 230, 130]
        };
        for (a, b) in crate::collision::world_bounds(collider, scene.world_matrix(index)).edges() {
            let a = project(a);
            let b = project(b);
            if a[2] > 1. && b[2] > 1. {
                canvas.line(a, b, color);
            }
        }
    }
    Image {
        width: w,
        height: h,
        pixels: canvas.pixels.into_iter().flatten().collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dolly_moves_along_the_view_past_the_pivot_without_changing_the_lens() {
        let mut view = View {
            center: [3., 2., -1.],
            ..Default::default()
        };
        let lens = view.zoom;
        for distance in [1.05, 8., 30.] {
            view.distance = distance;
            let pivot = view.center;
            let before = view.unproject([480., 342.], 0.);
            let forward = view.basis()[2];
            // Travel beyond the old pivot, including a camera previously saved
            // at the old zoom limit. The selected object's distance is irrelevant.
            view.dolly(40.);
            let eye = view.unproject([480., 342.], 0.);
            assert_eq!(view.zoom, lens);
            assert_eq!(view.distance, distance);
            for i in 0..3 {
                assert!((eye[i] - before[i] - forward[i] * 40.).abs() < 0.0001);
            }
            assert!(
                project(&view, pivot)[2] < 0.,
                "Wheel must pass the old pivot"
            );
            view.dolly(-40.);
            for i in 0..3 {
                assert!((view.center[i] - pivot[i]).abs() < 0.0001);
            }
        }
        let legacy: View = serde_json::from_value(serde_json::json!({"zoom": 1.5})).unwrap();
        assert_eq!(legacy.distance, 12.);
    }

    #[test]
    fn dolly_respects_speed_and_fractional_wheel_input() {
        let mut view = View {
            yaw: 0.,
            pitch: 0.,
            ..Default::default()
        };
        view.dolly(0.25);
        assert_eq!(view.center, [0., 0., 0.25]);
        view.fly_speed = 20.;
        view.dolly(0.25);
        assert_eq!(view.center, [0., 0., 1.25]);
        view.dolly(-0.25);
        assert_eq!(view.center, [0., 0., 0.25]);
    }

    #[test]
    fn orbit_preserves_the_pivot_and_free_look_preserves_the_eye_after_dolly() {
        let mut view = View::default();
        for distance in [1.05, 8., 30.] {
            view.distance = distance;
            view.dolly(10.);
            let pivot = view.center;
            view.look([80., -25.], true);
            assert_eq!(view.center, pivot);
            let p = project(&view, pivot);
            assert!((p[0] - 480.).abs() < 0.001 && (p[1] - 300.).abs() < 0.001);
            let eye = view.unproject([480., 342.], 0.);
            view.look([-50., 30.], false);
            for (a, b) in eye.into_iter().zip(view.unproject([480., 342.], 0.)) {
                assert!((a - b).abs() < 0.0001);
            }
        }
    }

    #[test]
    fn axis_snap_aligns_the_camera_forward_vector() {
        for (axis, direction) in [(0, 1.), (0, -1.), (1, 1.), (1, -1.), (2, 1.), (2, -1.)] {
            let mut view = View {
                center: [3., -2., 7.],
                distance: 42.,
                ..Default::default()
            };
            let pivot = view.center;
            let distance = view.distance;
            view.snap_to_axis(axis, direction);
            assert_eq!(view.center, pivot);
            assert_eq!(view.distance, distance);
            let forward = view.basis()[2];
            assert!(
                forward[axis] * direction > 0.999,
                "{axis} {direction}: {forward:?}"
            );
            for other in 0..3 {
                if other != axis {
                    assert!(
                        forward[other].abs() < 0.05,
                        "{axis} {direction}: {forward:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn isometric_snap_preserves_the_camera_target_and_lens() {
        let mut view = View {
            yaw: 4.2,
            pitch: -0.8,
            center: [3., -2., 7.],
            zoom: 1.7,
            distance: 42.,
            phase: 0.25,
            fly_speed: 12.,
        };
        let expected = View::default();
        view.snap_to_isometric();
        assert_eq!(view.yaw, expected.yaw);
        assert_eq!(view.pitch, expected.pitch);
        assert_eq!(view.center, [3., -2., 7.]);
        assert_eq!(view.zoom, 1.7);
        assert_eq!(view.distance, 42.);
        assert_eq!(view.phase, 0.25);
        assert_eq!(view.fly_speed, 12.);
    }

    #[test]
    fn framing_fits_bounds_without_changing_the_lens() {
        for zoom in [0.3, 0.85, 3.] {
            for size in [0.1, 1., 40.] {
                let mut view = View {
                    zoom,
                    ..Default::default()
                };
                let low = [-size, -size * 2., -size * 0.5];
                let high = [size, size * 2., size * 0.5];
                view.frame_bounds(low, high);
                assert_eq!(view.zoom, zoom);
                for yaw in [0., 0.7, 2.] {
                    view.yaw = yaw;
                    for mask in 0..8 {
                        let corner = std::array::from_fn(|i| {
                            if mask & (1 << i) == 0 {
                                low[i]
                            } else {
                                high[i]
                            }
                        });
                        let p = project(&view, corner);
                        assert!(
                            p[2] > 1. && (0. ..960.).contains(&p[0]) && (0. ..600.).contains(&p[1]),
                            "{p:?}"
                        );
                        let world = view.unproject([p[0], p[1]], p[2]);
                        for i in 0..3 {
                            assert!((world[i] - corner[i]).abs() < 0.001);
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn framing_respects_cropped_portrait_and_wide_panels() {
        for panel in [[240., 900.], [1500., 240.], [960., 600.], [1500., 100.]] {
            let factor = (panel[0] / 960_f32).max(panel[1] / 600.);
            let half = [panel[0] / factor * 0.5, panel[1] / factor * 0.5];
            for zoom in [0.3, 0.85, 3.5] {
                let mut view = View {
                    zoom,
                    ..Default::default()
                };
                view.frame_bounds_in_panel([-1., -2., -0.5], [1., 2., 0.5], panel);
                for yaw in [0., 1., 2.5] {
                    view.yaw = yaw;
                    for mask in 0..8 {
                        let point = [
                            if mask & 1 == 0 { -1. } else { 1. },
                            if mask & 2 == 0 { -2. } else { 2. },
                            if mask & 4 == 0 { -0.5 } else { 0.5 },
                        ];
                        let p = project(&view, point);
                        assert!(p[2] > 1.);
                        assert!(
                            (p[0] - 480.).abs() < half[0] && (p[1] - 300.).abs() < half[1],
                            "{panel:?}: {p:?}"
                        );
                    }
                }
            }
        }
    }
    #[test]
    fn navigation_and_picking_share_the_same_camera() {
        let mut view = View::default();
        let eye = view.unproject([480., 342.], 0.);
        view.look([35., -16.], false);
        for (a, b) in eye.into_iter().zip(view.unproject([480., 342.], 0.)) {
            assert!((a - b).abs() < 0.0001);
        }
        view.pan([24., -10.], 1.2);
        view.fly([1., 1., 1.], 0.02, true);
        let world = view.unproject([640., 270.], 15.);
        let p = project(&view, world);
        assert!(
            (p[0] - 640.).abs() < 0.001
                && (p[1] - 270.).abs() < 0.001
                && (p[2] - 15.).abs() < 0.001
        );
        let mut a = View::default();
        let mut b = a;
        for _ in 0..60 {
            a.fly([0., 0., 1.], 1. / 60., false);
        }
        for _ in 0..120 {
            b.fly([0., 0., 1.], 1. / 120., false);
        }
        for i in 0..3 {
            assert!((a.center[i] - b.center[i]).abs() < 0.0001);
        }
    }
    #[test]
    #[ignore = "timing baseline for the legacy CPU viewport"]
    fn profile_legacy_viewport() {
        let scene = Scene::load(std::path::Path::new(
            "examples/sample-game/assets/scenes/SampleScene.epokmap",
        ))
        .unwrap();
        let mut view = View::default();
        let mut raster = 0.;
        let mut convert = 0.;
        for _ in 0..40 {
            view.yaw += 0.008;
            let start = std::time::Instant::now();
            let image = render(&scene, Some(1), &view, true, false, false);
            raster += start.elapsed().as_secs_f64() * 1000.;
            let start = std::time::Instant::now();
            std::hint::black_box(
                image
                    .pixels
                    .chunks_exact(3)
                    .flat_map(|p| [p[0], p[1], p[2], 255])
                    .collect::<Vec<_>>(),
            );
            convert += start.elapsed().as_secs_f64() * 1000.;
        }
        println!(
            "Legacy CPU scene: raster {:.2} ms, conversion {:.2} ms, total {:.2} ms",
            raster / 40.,
            convert / 40.,
            (raster + convert) / 40.
        );
    }
    #[test]
    fn nearer_geometry_wins_regardless_of_submission_order() {
        let mut canvas = Canvas {
            pixels: vec![[0; 3]; 64],
            depth: vec![f32::INFINITY; 64],
            w: 8,
            h: 8,
        };
        let triangle = |z| [[0., 0., z], [7., 0., z], [0., 7., z]];
        canvas.triangle(triangle(2.), [[255, 0, 0]; 3]);
        canvas.triangle(triangle(10.), [[0, 0, 255]; 3]);
        assert_eq!(canvas.pixels[9], [255, 0, 0]);
        canvas.triangle(triangle(1.), [[0, 255, 0]; 3]);
        assert_eq!(canvas.pixels[9], [0, 255, 0]);
    }

    #[test]
    fn xyz_rotation_applies_to_mesh_vertices() {
        let p = rotate([1., 0., 0.], [0., 0., 90.]);
        assert!(p[0].abs() < 0.0001);
        assert!((p[1] - 1.).abs() < 0.0001);
    }
}
#[test]
fn free_look_rotates_in_place_and_directions_match_fps_controls() {
    let mut view = View {
        yaw: 0.,
        pitch: 0.,
        ..Default::default()
    };
    let eye = view.unproject([480., 342.], 0.);
    view.look([100., 50.], false);
    for (a, b) in eye.into_iter().zip(view.unproject([480., 342.], 0.)) {
        assert!((a - b).abs() < 0.00001);
    }
    let forward = view.basis()[2];
    assert!(forward[0] > 0. && forward[1] < 0.);
    let before = view.center;
    view.fly([0., 0., 1.], 0.1, false);
    for i in 0..3 {
        assert!((view.center[i] - before[i] - forward[i] * 0.5).abs() < 0.00001);
    }
    let before = view.center;
    view.fly([0., 1., 0.], 0.1, false);
    assert!((view.center[1] - before[1] - 0.5).abs() < 0.00001);
    assert_eq!(view.center[0], before[0]);
    let center = view.center;
    view.look([20., -10.], true);
    assert_eq!(view.center, center);
}
