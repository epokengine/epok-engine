//! Authored camera paths for visibility tracks. The author places camera keys
//! (pose, field of view, time) in the editor; the track's samples are the
//! path evaluated every `sample_ms`, ready for the visibility bake. Positions,
//! angles and field of view follow smooth Hermite curves through the keys,
//! easing in and out at the ends; a cut key jumps to its pose.
use crate::scene::Scene;
use crate::visibility_track::{Sample, Track};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Key {
    /// Seconds from the start of the path; strictly increasing.
    pub time: f32,
    pub position: [f32; 3],
    /// Euler degrees, as actor rotations.
    pub rotation: [f32; 3],
    /// Horizontal field of view in degrees, as track samples.
    pub fov: f32,
    /// The camera jumps to this key instead of travelling to it.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub cut: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub fov: f32,
}
impl From<&Key> for Pose {
    fn from(k: &Key) -> Self {
        Self::from(*k)
    }
}
impl From<Key> for Pose {
    fn from(k: Key) -> Self {
        Self {
            position: k.position,
            rotation: k.rotation,
            fov: k.fov,
        }
    }
}

pub const MAX_SAMPLES: usize = 65535;

pub fn validate(keys: &[Key], sample_ms: u32) -> Result<(), String> {
    if keys.is_empty() {
        return Err("A camera path needs at least one key".into());
    }
    if keys.iter().any(|k| {
        !k.time.is_finite()
            || k.position
                .iter()
                .chain(&k.rotation)
                .any(|v| !v.is_finite() || v.abs() > 32767.)
            || !(25. ..=120.).contains(&k.fov)
    }) {
        return Err("A camera path key has an invalid time, pose or field of view".into());
    }
    if keys[0].time != 0. || keys.windows(2).any(|w| w[1].time <= w[0].time) {
        return Err("Camera path keys must start at 0 s and increase in time".into());
    }
    if sample_count(keys, sample_ms) > MAX_SAMPLES {
        return Err(format!(
            "The camera path is too long for {sample_ms} ms samples (more than {MAX_SAMPLES})"
        ));
    }
    Ok(())
}

pub fn duration(keys: &[Key]) -> f32 {
    keys.last().map_or(0., |k| k.time)
}
pub fn sample_count(keys: &[Key], sample_ms: u32) -> usize {
    // A tenth of a millisecond of slack, as `sample_time`.
    ((f64::from(duration(keys)) + 1e-4) * 1000. / f64::from(sample_ms.max(1))).floor() as usize + 1
}

fn wrap(degrees: f32) -> f32 {
    (degrees + 180.).rem_euclid(360.) - 180.
}

/// The pose at `time` seconds (clamped to the path).
pub fn pose_at(keys: &[Key], time: f32) -> Pose {
    let Some(first) = keys.first() else {
        return Pose {
            position: [0.; 3],
            rotation: [0.; 3],
            fov: 60.,
        };
    };
    let i = keys.partition_point(|k| k.time <= time);
    if i == 0 {
        return first.into();
    }
    if i == keys.len() {
        return keys[i - 1].into();
    }
    let (a, b) = (&keys[i - 1], &keys[i]);
    if b.cut {
        return a.into();
    }
    // Neighbours across a cut do not shape the curve.
    let before = (i >= 2 && !a.cut).then(|| &keys[i - 2]);
    let after = keys.get(i + 1).filter(|k| !k.cut);
    let h = b.time - a.time;
    let s = (time - a.time) / h;
    let (s2, s3) = (s * s, s * s * s);
    let (h00, h10, h01, h11) = (
        2. * s3 - 3. * s2 + 1.,
        s3 - 2. * s2 + s,
        -2. * s3 + 3. * s2,
        s3 - s2,
    );
    // Hermite with Catmull-Rom tangents per second; zero at the path's ends.
    let curve = |p0: Option<(f32, f32)>, p1: f32, p2: f32, p3: Option<(f32, f32)>| {
        let m1 = p0.map_or(0., |(t, v)| (p2 - v) / (b.time - t));
        let m2 = p3.map_or(0., |(t, v)| (v - p1) / (t - a.time));
        h00 * p1 + h10 * h * m1 + h01 * p2 + h11 * h * m2
    };
    let position = std::array::from_fn(|c| {
        curve(
            before.map(|k| (k.time, k.position[c])),
            a.position[c],
            b.position[c],
            after.map(|k| (k.time, k.position[c])),
        )
    });
    // Angles unwrapped around the segment's start: the short way round.
    let rotation = std::array::from_fn(|c| {
        let start = a.rotation[c];
        let end = start + wrap(b.rotation[c] - start);
        curve(
            before.map(|k| (k.time, start - wrap(start - k.rotation[c]))),
            start,
            end,
            after.map(|k| (k.time, end + wrap(k.rotation[c] - end))),
        )
    });
    let fov = curve(
        before.map(|k| (k.time, k.fov)),
        a.fov,
        b.fov,
        after.map(|k| (k.time, k.fov)),
    )
    .clamp(25., 120.);
    Pose {
        position,
        rotation: rotation.map(wrap),
        fov,
    }
}

/// The path time of sample `n`. A key's time, typed or converted from a
/// sample, may round a little either side of the sample's; samples are taken
/// a tenth of a millisecond late, so a sample at a key always reaches it (a
/// cut sample shows the pose after the cut).
fn sample_time(n: usize, sample_ms: u32) -> f32 {
    (n as f64 * f64::from(sample_ms) / 1000. + 1e-4) as f32
}

/// Track samples along the path, every `sample_ms`, with empty face lists.
/// A sample is a cut where a cut key falls since the previous sample; the
/// first sample is a cut too, so the track does not travel from its end back
/// to its start.
pub fn samples(keys: &[Key], sample_ms: u32) -> Vec<Sample> {
    (0..sample_count(keys, sample_ms))
        .map(|n| {
            let time = sample_time(n, sample_ms);
            let previous = if n == 0 {
                f32::NEG_INFINITY
            } else {
                sample_time(n - 1, sample_ms)
            };
            let pose = pose_at(keys, time);
            Sample {
                position: pose.position,
                rotation: pose.rotation,
                fov: pose.fov,
                visible: Vec::new(),
                cut: n == 0
                    || keys
                        .iter()
                        .any(|k| k.cut && k.time > previous && k.time <= time),
            }
        })
        .collect()
}

/// Whether the track's samples are the path's (ignoring the baked lists).
pub fn samples_current(track: &Track) -> bool {
    let generated = samples(&track.path, track.sample_ms);
    generated.len() == track.samples.len()
        && generated.iter().zip(&track.samples).all(|(g, s)| {
            g.cut == s.cut && g.fov == s.fov && g.position == s.position && g.rotation == s.rotation
        })
}

/// Replaces the track's samples with the path's. Baked lists no longer
/// match the samples, so they are cleared, and so is a prepared packet order.
pub fn apply(scene: &mut Scene) -> Result<(), String> {
    let track = scene
        .visibility_track
        .as_mut()
        .ok_or("The scene has no visibility track")?;
    if track.navigation.is_some() {
        return Err(
            "This track's samples come from a route graph; its path cannot be edited".into(),
        );
    }
    validate(&track.path, track.sample_ms)?;
    track.samples = samples(&track.path, track.sample_ms);
    scene.ordered_draw = None;
    Ok(())
}

/// Faces of every mesh a visibility track can cover: ordinary editable
/// meshes, all of their faces.
pub fn trackable_faces(scene: &Scene) -> Vec<Uuid> {
    scene
        .actors
        .iter()
        .filter(|e| e.kind == "Mesh" && e.skeletal_mesh.is_none() && e.terrain.is_none())
        .filter_map(|e| e.editable_mesh.as_ref()?.document.as_ref())
        .flat_map(|doc| doc.faces.iter().map(|f| f.id))
        .collect()
}

/// Rebuilds the face table from the scene's meshes, keeping baked lists of
/// faces that still exist.
pub fn refresh_faces(scene: &mut Scene) {
    let faces = trackable_faces(scene);
    let Some(track) = scene.visibility_track.as_mut() else {
        return;
    };
    let index: std::collections::HashMap<Uuid, u16> = faces
        .iter()
        .enumerate()
        .map(|(i, f)| (*f, i as u16))
        .collect();
    for sample in &mut track.samples {
        sample.visible = sample
            .visible
            .iter()
            .filter_map(|&i| index.get(track.faces.get(usize::from(i))?).copied())
            .collect();
        sample.visible.sort_unstable();
    }
    track.faces = faces;
}

/// Root camera actors a track can drive.
pub fn cameras(scene: &Scene) -> Vec<usize> {
    (0..scene.actors.len())
        .filter(|&i| scene.actors[i].kind == "Camera" && scene.actors[i].parent.is_none())
        .collect()
}

/// A new track for a root camera, covering every trackable mesh, with a path
/// of one key at the camera's pose.
pub fn create(scene: &mut Scene, camera: usize) -> Result<(), String> {
    let actor = scene.actors.get(camera).ok_or("No such camera")?;
    if actor.kind != "Camera" || actor.parent.is_some() {
        return Err("A visibility track needs a root camera actor".into());
    }
    let key = Key {
        time: 0.,
        position: actor.position,
        rotation: actor.rotation.map(wrap),
        fov: actor.camera_fov.clamp(25., 120.),
        cut: false,
    };
    let faces = trackable_faces(scene);
    if faces.len() > 65535 {
        return Err("The scene has more than 65535 mesh faces for one track".into());
    }
    scene.visibility_track = Some(Track {
        camera: actor.id,
        sample_ms: 100,
        cull_backfaces: true,
        faces,
        samples: samples(&[key], 100),
        navigation: None,
        static_world: false,
        path: vec![key],
    });
    Ok(())
}

/// How far a converted path may stray from the samples it replaces.
#[derive(Clone, Copy, Debug)]
pub struct Tolerance {
    /// World units.
    pub position: f32,
    /// Degrees, for rotation and field of view.
    pub angle: f32,
}

/// Largest deviation of a path from samples, as a fraction of the tolerance.
fn deviation(pose: Pose, sample: &Sample, tolerance: Tolerance) -> f32 {
    let position = (0..3)
        .map(|c| (pose.position[c] - sample.position[c]).powi(2))
        .sum::<f32>()
        .sqrt()
        / tolerance.position.max(1e-6);
    let angle = (0..3)
        .map(|c| wrap(pose.rotation[c] - sample.rotation[c]).abs())
        .chain(std::iter::once((pose.fov - sample.fov).abs()))
        .fold(0., f32::max)
        / tolerance.angle.max(1e-6);
    position.max(angle)
}

impl Tolerance {
    /// A tenth of a runtime camera unit (the near plane is a quarter) and a
    /// quarter of a degree.
    pub fn for_track(scene: &Scene, track: &Track) -> Self {
        Self {
            position: 0.1 * crate::visibility_bake::camera_depth_scale(scene, track),
            angle: 0.25,
        }
    }
}

/// Keys from a track's existing samples, so imported tracks can be edited as
/// paths: one every `every` seconds and at every cut, then more wherever the
/// path strays from the samples by more than the tolerance.
pub fn keys_from_samples(track: &Track, every: f32, tolerance: Tolerance) -> Vec<Key> {
    let step = track.sample_ms as f32 / 1000.;
    let ms = track.sample_ms;
    let stride = ((every / step).round() as usize).max(1);
    let last = track.samples.len().saturating_sub(1);
    let key = |n: usize| {
        let s = &track.samples[n];
        Key {
            time: (n as f64 * f64::from(ms) / 1000.) as f32,
            position: s.position,
            rotation: s.rotation.map(wrap),
            fov: s.fov,
            cut: s.cut && n > 0,
        }
    };
    let mut chosen: std::collections::BTreeSet<usize> = (0..track.samples.len())
        .filter(|&n| {
            let next_cut = track.samples.get(n + 1).is_some_and(|s| s.cut);
            n % stride == 0 || track.samples[n].cut || next_cut || n == last
        })
        .collect();
    loop {
        let keys: Vec<Key> = chosen.iter().map(|&n| key(n)).collect();
        // The worst sample of each segment between keys that strays too far.
        let worst: Vec<usize> = chosen
            .iter()
            .zip(chosen.iter().skip(1))
            .filter_map(|(&a, &b)| {
                (a + 1..b)
                    .map(|n| {
                        (
                            n,
                            deviation(
                                pose_at(&keys, sample_time(n, ms)),
                                &track.samples[n],
                                tolerance,
                            ),
                        )
                    })
                    .filter(|&(_, d)| d > 1.)
                    .max_by(|x, y| x.1.total_cmp(&y.1))
                    .map(|(n, _)| n)
            })
            .collect();
        if worst.is_empty() {
            return keys;
        }
        chosen.extend(worst);
    }
}

/// Largest deviation of the path from the track's samples: world units and
/// degrees (rotation or field of view).
pub fn path_error(track: &Track) -> (f32, f32) {
    track
        .samples
        .iter()
        .enumerate()
        .fold((0., 0.), |(p, a), (n, s)| {
            let pose = pose_at(&track.path, sample_time(n, track.sample_ms));
            let position = deviation(
                pose,
                s,
                Tolerance {
                    position: 1.,
                    angle: f32::INFINITY,
                },
            );
            let angle = deviation(
                pose,
                s,
                Tolerance {
                    position: f32::INFINITY,
                    angle: 1.,
                },
            );
            (p.max(position), a.max(angle))
        })
}

/// The editor view's camera as a path pose: the view looks along an actor's
/// +Z with rotation [pitch, yaw, 0]; the lens maps to a horizontal FOV.
pub fn pose_from_view(view: &crate::viewport::View) -> Pose {
    let forward = view.basis()[2];
    Pose {
        position: std::array::from_fn(|i| view.center[i] - forward[i] * view.distance),
        rotation: [
            wrap(view.pitch.to_degrees()),
            wrap(view.yaw.to_degrees()),
            0.,
        ],
        fov: (2. * (480. / (870. * view.zoom)).atan())
            .to_degrees()
            .clamp(25., 120.),
    }
}
/// Moves the editor view to look through a pose (roll is not shown).
pub fn view_through(view: &mut crate::viewport::View, pose: Pose) {
    view.yaw = pose.rotation[1]
        .to_radians()
        .rem_euclid(std::f32::consts::TAU);
    view.pitch = pose.rotation[0].to_radians().clamp(-1.54, 1.54);
    view.zoom = 480. / (870. * (pose.fov.clamp(25., 120.).to_radians() / 2.).tan());
    view.mirror_x = false;
    let forward = view.basis()[2];
    view.center = std::array::from_fn(|i| pose.position[i] + forward[i] * view.distance);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(time: f32, x: f32, yaw: f32) -> Key {
        Key {
            time,
            position: [x, 1., 0.],
            rotation: [0., yaw, 0.],
            fov: 60.,
            cut: false,
        }
    }

    #[test]
    fn path_passes_through_its_keys_smoothly_and_turns_the_short_way() {
        let keys = [key(0., 0., 170.), key(1., 10., -170.), key(3., 20., -150.)];
        validate(&keys, 100).unwrap();
        for k in &keys {
            let p = pose_at(&keys, k.time);
            assert!((p.position[0] - k.position[0]).abs() < 1e-4);
            assert!(wrap(p.rotation[1] - k.rotation[1]).abs() < 1e-3);
        }
        // Halfway between 170 and -170 is 180, not 0.
        assert!(wrap(pose_at(&keys, 0.5).rotation[1] - 180.).abs() < 5.);
        // Eases out of the first key and keeps moving through the middle one.
        assert!(pose_at(&keys, 0.05).position[0] < 0.5);
        assert!(pose_at(&keys, 1.05).position[0] > pose_at(&keys, 1.).position[0]);
        let samples = samples(&keys, 100);
        assert_eq!(samples.len(), 31);
        assert!(samples[0].cut && samples[1..].iter().all(|s| !s.cut));
        assert!((samples[10].position[0] - 10.).abs() < 1e-3);
    }

    #[test]
    fn cut_keys_hold_then_jump_and_mark_the_sample() {
        let mut keys = [
            key(0., 0., 0.),
            key(1., 10., 0.),
            key(1.25, 50., 0.),
            key(2., 60., 0.),
        ];
        keys[2].cut = true;
        assert_eq!(pose_at(&keys, 1.2).position[0], 10.);
        assert_eq!(pose_at(&keys, 1.25).position[0], 50.);
        let samples = samples(&keys, 100);
        let cuts: Vec<_> = samples
            .iter()
            .enumerate()
            .filter(|(_, s)| s.cut)
            .map(|(i, _)| i)
            .collect();
        assert_eq!(cuts, [0, 13]);
        assert!(samples[13].position[0] > 50.);
        // The segment after a cut does not bend towards the key before it.
        assert!(pose_at(&keys, 1.3).position[0] >= 50.);
        let back = keys_from_samples(
            &Track {
                camera: Uuid::nil(),
                sample_ms: 100,
                cull_backfaces: true,
                faces: Vec::new(),
                samples,
                navigation: None,
                static_world: false,
                path: Vec::new(),
            },
            0.5,
            Tolerance {
                position: 1e-3,
                angle: 1e-3,
            },
        );
        assert!(back.iter().any(|k| k.cut && (k.time - 1.3).abs() < 1e-4));
        validate(&back, 100).unwrap();
    }

    #[test]
    fn converted_tracks_resample_to_their_samples_at_every_cut() {
        // Cuts at sample times that do not round-trip through f32 seconds.
        let samples: Vec<Sample> = (0..1650)
            .map(|n| Sample {
                position: [
                    if n >= 46 { 100. } else { 0. } + n as f32 * 0.1,
                    0.,
                    if n >= 293 { 50. } else { 0. },
                ],
                rotation: [0., (n as f32 * 1.7).sin() * 30., 0.],
                fov: 70.,
                visible: Vec::new(),
                cut: n == 0 || n == 46 || n == 293,
            })
            .collect();
        let mut track = Track {
            camera: Uuid::nil(),
            sample_ms: 100,
            cull_backfaces: true,
            faces: Vec::new(),
            samples,
            navigation: None,
            static_world: false,
            path: Vec::new(),
        };
        let tolerance = Tolerance {
            position: 0.01,
            angle: 0.05,
        };
        track.path = keys_from_samples(&track, 1., tolerance);
        let (position, angle) = path_error(&track);
        assert!(position <= 0.01 && angle <= 0.05, "{position} {angle}");
        assert_eq!(sample_count(&track.path, 100), 1650);
        let resampled = super::samples(&track.path, 100);
        assert_eq!(resampled.len(), track.samples.len());
        for (a, b) in resampled.iter().zip(&track.samples) {
            assert_eq!(a.cut, b.cut);
            assert!((a.position[0] - b.position[0]).abs() <= 0.01);
        }
    }

    #[test]
    fn invalid_keys_are_rejected() {
        assert!(validate(&[], 100).is_err());
        assert!(validate(&[key(0.5, 0., 0.)], 100).is_err());
        assert!(validate(&[key(0., 0., 0.), key(0., 1., 0.)], 100).is_err());
        assert!(validate(&[key(0., 0., 0.), key(10000., 1., 0.)], 100).is_err());
    }

    #[test]
    fn the_view_and_a_pose_agree_with_an_actor_transform() {
        let mut view = crate::viewport::View {
            yaw: 0.8,
            pitch: -0.3,
            center: [3., 2., -1.],
            ..Default::default()
        };
        let pose = pose_from_view(&view);
        let m = crate::transform::Matrix::trs(pose.position, pose.rotation, [1.; 3]);
        let [_, up, forward] = view.basis();
        for (row, (up, forward)) in m.0.iter().zip(up.iter().zip(forward)) {
            assert!((row[2] - forward).abs() < 1e-5);
            assert!((row[1] - up).abs() < 1e-5);
        }
        let before = (view.center, view.yaw, view.pitch, view.zoom);
        view.center = [0.; 3];
        view.zoom = 3.;
        view_through(&mut view, pose);
        assert!((view.yaw - before.1).abs() < 1e-5 && (view.pitch - before.2).abs() < 1e-5);
        assert!((view.zoom - before.3).abs() < 1e-4);
        for i in 0..3 {
            assert!((view.center[i] - before.0[i]).abs() < 1e-4);
        }
    }
}
