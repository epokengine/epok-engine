//! Meridian Circuit. The two project-owned Lua components run the race and boost drive.
use crate::{
    actor_document::{ClassReference, ComponentInstance},
    assets,
    audio::AudioSource,
    audio_import,
    effects::Fog,
    hud,
    lighting::{dot, sub, unit},
    mesh::{self, Document, Slot, cross},
    scene::{Actor, Material, Scene, SkyGradient},
    spline::{Point, Spline},
    workspace::{GameplayFlavor, Template},
};
use std::{path::Path, sync::Arc};
use uuid::Uuid;
const SCRIPT: &str = "assets/scripts/RacePilot.lua";
const BOOST_SCRIPT: &str = "assets/scripts/RaceBoost.lua";
const BOOST_CLASS: &str = "ee91a503-23a8-42c6-8932-276461ff039c";
const CLASS: &str = "7fa1dc74-9e15-4f8e-a97a-5b46b1896692";

macro_rules! art {
    ($name:literal) => {
        include_bytes!(concat!("../resources/templates/race/", $name)).as_slice()
    };
}
const FINISHES: [(&str, &[u8]); 6] = [
    ("alloy", art!("alloy.png")),
    ("induction", art!("induction.png")),
    ("ceramic", art!("ceramic.png")),
    ("grid", art!("grid.png")),
    ("boost-left", art!("boost-left.png")),
    ("boost-right", art!("boost-right.png")),
];

struct Knot {
    point: Point,
    surface: usize,
    right: Option<[f32; 3]>,
    bank: f32,
}
fn knot(p: [f32; 3], surface: usize, bank: f32) -> Knot {
    Knot {
        point: p.into(),
        surface,
        right: None,
        bank,
    }
}
fn loop_knots(knots: &mut Vec<Knot>, x: f32, z: f32, sign: f32, radius: f32) {
    for i in 0..=10 {
        let t = i as f32 / 10.;
        let a = std::f32::consts::TAU * t;
        let offset = 10. * t * t * (3. - 2. * t);
        let mut k = knot(
            [
                x + sign * offset,
                radius * (1. - a.cos()),
                z + sign * radius * a.sin(),
            ],
            if (3..7).contains(&i) { 2 } else { 1 },
            0.,
        );
        k.right = Some([sign, 0., 0.]);
        knots.push(k);
    }
}
fn track(slots: &[Slot]) -> Result<Spline, String> {
    let mut knots = vec![
        knot([-44., 0., -54.], 3, 0.),
        knot([-44., 0., -49.], 0, 0.),
        knot([-44., 0., -37.], 4, 0.),
        knot([-44., 0., -29.], 0, 0.),
    ];
    loop_knots(&mut knots, -44., -14., 1., 11.);
    knots.extend([
        knot([-34., 0., 8.], 0, 0.),
        knot([-32., 2., 28.], 2, -12.),
        knot([-16., 5., 46.], 1, -24.),
        knot([8., 6., 52.], 5, -12.),
        knot([25., 4., 49.], 0, -18.),
        knot([44., 1., 34.], 2, -25.),
        knot([48., 0., 16.], 0, -5.),
        knot([48., 0., 8.], 1, 0.),
    ]);
    loop_knots(&mut knots, 48., -6., -1., 9.);
    knots.extend([
        knot([38., 0., -22.], 0, 0.),
        knot([38., 1., -31.], 5, -8.),
        knot([34., 2., -43.], 2, -24.),
        knot([22., 4., -60.], 1, -24.),
        knot([1., 5., -70.], 0, -20.),
        knot([-19., 3., -72.], 4, -12.),
        knot([-30., 1., -69.], 0, -24.),
        knot([-41., 0., -62.], 2, -15.),
    ]);
    let mut curve = Spline {
        points: knots.iter().map(|k| k.point).collect(),
        closed: true,
        width: 6.8,
        steps: 5,
        gameplay_channel: Some(0),
        ..Default::default()
    };
    let frames = curve.frames()?;
    for (i, k) in knots.iter().enumerate() {
        let f = frames[i * usize::from(curve.steps)];
        let reference = k.right.unwrap_or_else(|| cross([0., 1., 0.], f.tangent));
        let lateral = unit(sub(
            reference,
            f.tangent.map(|t| t * dot(reference, f.tangent)),
        ));
        let angle = dot(f.tangent, cross(f.right, lateral))
            .atan2(dot(f.right, lateral))
            .to_degrees()
            + k.bank;
        curve.points[i].bank = (angle + 180.).rem_euclid(360.) - 180.;
        curve.points[i].surface = Some(slots[k.surface].id);
        curve.points[i].tag = match k.surface {
            4 => 1,
            5 => 2,
            _ => 0,
        };
    }
    Ok(curve)
}
fn slot(doc: &mut Document, name: &str, color: [f32; 3]) -> Uuid {
    let id = Uuid::new_v4();
    doc.materials.push(Slot {
        id,
        name: name.into(),
        material: Material {
            color,
            unlit: true,
            double_sided: true,
            ..Default::default()
        },
    });
    id
}
fn imported(root: &Path, name: &str, bytes: &[u8], audio: bool) -> Result<Uuid, String> {
    let ext = if audio { "wav" } else { "png" };
    let source = format!("assets/Race/Source/{name}.{ext}");
    let destination = format!("assets/Race/{name}.epokasset");
    crate::project::write_changed(&root.join(&source), bytes)?;
    let candidate = if audio {
        assets::prepare(
            root,
            &source,
            &destination,
            audio_import::Settings {
                looping: name == "motor",
                ..Default::default()
            },
            None,
            false,
        )?
    } else {
        crate::texture::prepare(root, &source, &destination, None, false)?
    };
    assets::commit(candidate)
}
fn mesh_actor(root: &Path, name: &str, file: &str, doc: Document) -> Result<Actor, String> {
    let asset = mesh::create(root, &format!("assets/Race/{file}.epokasset"), &doc)?;
    let mut actor = Actor::cube(name.into());
    actor.position = [0.; 3];
    let mut component = mesh::Component::new(asset);
    component.document = Some(Arc::new(doc));
    actor.editable_mesh = Some(component);
    Ok(actor)
}
fn textured_slot(doc: &mut Document, name: &str, texture: Uuid, tint: [f32; 3]) -> Uuid {
    let id = slot(doc, name, tint);
    doc.materials.last_mut().unwrap().material.texture = Some(texture);
    id
}

/// Triangulate skin and ribbon panels whose four corners can lie on different planes.
fn panel(d: &mut Document, [a, b, c, end]: [[f32; 3]; 4], group: Uuid, material: Uuid) {
    d.add_face([a, b, c, c], group, material);
    d.faces.last_mut().unwrap().uv = [[0., 0.], [1., 0.], [1., 1.], [1., 1.]];
    if c != end {
        d.add_face([a, c, end, end], group, material);
        d.faces.last_mut().unwrap().uv = [[0., 0.], [1., 1.], [0., 1.], [0., 1.]];
    }
}

fn craft(livery: Uuid, nozzle: Uuid) -> Document {
    let mut d = Document::default();
    d.materials.clear();
    let paint = textured_slot(
        &mut d,
        "Kestrel 07 | lacquer and race stripe",
        livery,
        [1.; 3],
    );
    let ivory = slot(&mut d, "Ceramic leading edge", [0.86, 0.85, 0.70]);
    let navy = slot(&mut d, "Carbon keel", [0.065, 0.12, 0.20]);
    let blue = slot(&mut d, "Cobalt engine cowling", [0.12, 0.32, 0.52]);
    let amber = slot(&mut d, "Rescue orange", [1.0, 0.43, 0.13]);
    let glass = slot(&mut d, "Smoked canopy", [0.045, 0.22, 0.33]);
    let glint = slot(&mut d, "Reflected sky", [0.29, 0.73, 0.76]);
    let turbine = textured_slot(&mut d, "Turbine throat and emissive core", nozzle, [1.; 3]);
    let g = d.groups[0].id;
    let mut face = |p, m| panel(&mut d, p, g, m);
    let nose = [0., -0.01, 1.24];
    let ridge = [0., 0.13, 0.42];
    let roof = [0., 0.35, -0.22];
    let tail = [0., 0.13, -0.82];
    for s in [-1., 1.] {
        let chin = [s * 0.22, -0.12, 0.25];
        let bow = [s * 0.24, 0.08, 0.34];
        let sill = [s * 0.31, 0.10, -0.30];
        let aft = [s * 0.29, 0.03, -0.82];
        let bottom = [s * 0.29, -0.18, -0.70];
        face([nose, bow, ridge, ridge], ivory);
        face([nose, chin, bow, bow], blue);
        face([bow, sill, bottom, chin], navy);
        face([bow, ridge, roof, sill], if s < 0. { glass } else { glint });
        face([sill, roof, tail, aft], paint);
        face([aft, sill, bottom, bottom], blue);
        face([tail, aft, bottom, [0., -0.14, -0.75]], navy);
        // The angular nacelles have readable back plates in the chase camera.
        let front_i = [s * 0.29, 0.03, 0.47];
        let front_o = [s * 0.46, 0.01, 0.34];
        let rear_i = [s * 0.34, 0.15, -0.91];
        let rear_o = [s * 0.78, 0.12, -0.91];
        let rear_lo = [s * 0.78, -0.15, -0.91];
        let rear_li = [s * 0.34, -0.15, -0.91];
        let tip = [s * 0.94, -0.035, -0.60];
        face([front_i, front_o, rear_o, rear_i], paint);
        face([front_o, tip, rear_o, rear_o], amber);
        face([tip, rear_lo, rear_o, rear_o], blue);
        face([front_i, rear_i, rear_li, chin], blue);
        face([front_o, front_i, rear_li, rear_lo], navy);
        face([front_o, rear_lo, tip, tip], navy);
        face([rear_i, rear_o, rear_lo, rear_li], turbine);
        let fin = [s * 0.73, 0.38, -0.80];
        face(
            [
                [s * 0.70, 0.10, -0.81],
                fin,
                [s * 0.60, 0.065, -0.19],
                [s * 0.60, 0.065, -0.19],
            ],
            blue,
        );
        face(
            [
                fin,
                [s * 0.74, 0.30, -0.84],
                [s * 0.66, 0.16, -0.47],
                [s * 0.66, 0.16, -0.47],
            ],
            amber,
        );
    }
    // Keep the low hull above long road panels in the PSX ordering table.
    for slot in &mut d.materials {
        slot.material.depth_bias = -4;
    }
    d
}

fn exhaust() -> Document {
    let mut d = Document::default();
    d.materials.clear();
    let sheath = slot(&mut d, "Ion envelope", [1.; 3]);
    let envelope = &mut d.materials.last_mut().unwrap().material;
    envelope.blend = crate::texture::BlendMode::Add;
    envelope.depth_bias = -4;
    let hot = slot(&mut d, "Plasma core", [0.73, 1.0, 0.91]);
    d.materials.last_mut().unwrap().material.depth_bias = -5;
    let group = d.groups[0].id;
    for x in [-0.55, 0.55] {
        let tip = [x, -0.025, -0.70];
        for (a, b) in [
            ([-0.14, 0.], [0., 0.072]),
            ([0., 0.072], [0.14, 0.]),
            ([0.14, 0.], [0., -0.072]),
            ([0., -0.072], [-0.14, 0.]),
        ] {
            d.add_face(
                [[x + a[0], a[1], 0.], [x + b[0], b[1], 0.], tip, tip],
                group,
                sheath,
            );
            d.faces.last_mut().unwrap().vertex_colors =
                Some([[93, 242, 238], [84, 215, 255], [14, 63, 157], [14, 63, 157]]);
        }
        let end = [x, 0.008, -0.47];
        d.add_face(
            [[x - 0.085, 0.028, 0.], [x + 0.085, 0.028, 0.], end, end],
            group,
            hot,
        );
    }
    d
}
fn scenery(curve: &Spline, windows: Uuid, barriers: Uuid) -> Result<Document, String> {
    let mut d = Document::default();
    d.materials.clear();
    let steel = slot(&mut d, "Midnight steel", [0.12, 0.19, 0.27]);
    let metal = slot(&mut d, "Galvanized pipe", [0.41, 0.55, 0.59]);
    let hazard = textured_slot(&mut d, "Dockside hazard boards", barriers, [1.; 3]);
    let face = textured_slot(&mut d, "Port office windows", windows, [1.; 3]);
    d.materials.last_mut().unwrap().material.double_sided = false;
    let stripe = slot(&mut d, "Approach lights", [0.18, 0.89, 0.85]);
    let red = slot(&mut d, "Signal orange", [0.98, 0.32, 0.13]);
    let g = d.groups[0].id;
    // Buildings are deliberately clustered, leaving the silhouettes of the loops legible.
    for (i, (x, z)) in [
        (-62., -35.),
        (-66., -7.),
        (-58., 21.),
        (-67., 40.),
        (-47., 54.),
        (-18., 71.),
        (8., 78.),
        (41., 69.),
        (62., 47.),
        (70., 24.),
        (66., -7.),
        (61., -37.),
        (48., -65.),
        (20., -82.),
        (-16., -88.),
        (-51., -73.),
        (-9., 13.),
        (11., 18.),
        (19., -12.),
        (-7., -22.),
    ]
    .into_iter()
    .enumerate()
    {
        let h = 24. + ((i * 7) % 22) as f32;
        let w = 5. + (i % 4) as f32;
        let y = -24. + h * 0.5;
        let begin = d.faces.len();
        d.primitive("Box", [x, y, z], [w, h, w * 0.84], 1, g, face);
        for wall in &mut d.faces[begin..] {
            let points = wall.vertices.map(|v| d.vertices[v as usize]);
            if points.iter().all(|v| v[1] == points[0][1]) {
                wall.material = steel;
            } else {
                let axis = if points.iter().all(|v| v[0] == points[0][0]) {
                    2
                } else {
                    0
                };
                let center = if axis == 2 { z } else { x };
                let width = if axis == 2 { w * 0.84 } else { w };
                wall.uv = points.map(|v| {
                    [
                        ((v[axis] - center) / width + 0.5).clamp(0., 1.),
                        (1.0 - (v[1] + 24.) / h).clamp(0., 1.),
                    ]
                });
            }
        }
        if i % 4 == 0 {
            d.primitive(
                "Box",
                [x, y + h * 0.5 - 0.48, z],
                [w + 0.08, 0.34, w * 0.84 + 0.08],
                1,
                g,
                if i % 4 == 0 { red } else { stripe },
            );
        }
    }
    // Slim luminous pylons follow the ribbon, including through both inversions.
    let frames = curve.frames()?;
    let bar = |doc: &mut Document, frame: crate::spline::Frame, x: f32, h: f32, m: Uuid| {
        let p = |dx, dy, dz| {
            std::array::from_fn(|k| {
                frame.center[k]
                    + frame.right[k] * (x + dx)
                    + frame.up[k] * dy
                    + frame.tangent[k] * dz
            })
        };
        doc.add_face(
            [
                p(-0.09, 0., 0.),
                p(0.09, 0., 0.),
                p(0.09, h, 0.),
                p(-0.09, h, 0.),
            ],
            g,
            m,
        );
    };
    for (i, f) in frames[..frames.len() - 1].iter().enumerate().step_by(5) {
        for s in [-1., 1.] {
            bar(
                &mut d,
                *f,
                s * 3.55,
                if i % 15 == 0 { 1.25 } else { 0.48 },
                if i % 15 == 0 { red } else { stripe },
            );
        }
    }
    // Short barrier panels follow the spline frame and keep the banked curves readable.
    let at = |f: crate::spline::Frame, x: f32, y: f32| -> [f32; 3] {
        std::array::from_fn(|i| f.center[i] + f.right[i] * x + f.up[i] * y)
    };
    for (start, end) in [(1, 18), (74, 98), (157, 172), (183, 203)] {
        for i in (start..end).step_by(2) {
            let a = frames[i];
            let b = frames[i + 2];
            for side in [-3.54, 3.54] {
                panel(
                    &mut d,
                    [
                        at(a, side, 0.46),
                        at(b, side, 0.46),
                        at(b, side, -0.09),
                        at(a, side, -0.09),
                    ],
                    g,
                    hazard,
                );
            }
        }
    }
    // The pair of service galleries runs alongside the first boost. Gaps in their
    // lattice let the driver see the loop and make the speed of passing bays clear.
    for (side, height, bays) in [(-1., 2.90, 5), (1., 1.75, 3)] {
        let x = -44. + side * 4.75;
        for i in 0..bays {
            let z = -46. + i as f32 * 5.0;
            let p = |y, z| [x, y, z];
            d.add_face(
                [
                    p(height, z),
                    p(height, z + 5.),
                    p(height - 0.24, z + 5.),
                    p(height - 0.24, z),
                ],
                g,
                steel,
            );
            d.add_face(
                [p(0.12, z), p(0.12, z + 5.), p(-0.45, z + 5.), p(-0.45, z)],
                g,
                hazard,
            );
            d.add_face(
                [
                    p(0.16, z + 0.14),
                    p(height, z + 4.86),
                    p(height, z + 4.44),
                    p(0.16, z + 0.06),
                ],
                g,
                red,
            );
            d.add_face(
                [
                    p(height, z),
                    p(height, z + 0.20),
                    p(-6.5, z + 0.20),
                    p(-6.5, z),
                ],
                g,
                steel,
            );
        }
        d.primitive(
            "Box",
            [x, height + 0.08, -46. + 2.5 * bays as f32],
            [0.38, 0.20, 5. * bays as f32],
            1,
            g,
            metal,
        );
    }
    // Cargo cranes are off the racing line. The near gantry frames the first loop.
    for (x, z, span, h) in [(-57., -28., 11., 10.), (58., 13., -10., 12.)] {
        d.primitive("Box", [x, h * 0.5 - 3., z], [0.9, h + 6., 1.3], 1, g, steel);
        d.primitive(
            "Box",
            [x + span * 0.35, h - 1.4, z],
            [span.abs() * 0.88, 1.25, 1.2],
            1,
            g,
            face,
        );
        d.primitive(
            "Box",
            [x + span * 0.37, h, z],
            [span.abs() * 1.18, 0.38, 0.59],
            1,
            g,
            red,
        );
        for i in 0..5 {
            let a = x + span * (-0.18 + i as f32 * 0.22);
            let b = a + span * 0.21;
            d.add_face(
                [
                    [a, h - 1.0, z - 0.31],
                    [b, h - 0.06, z - 0.31],
                    [b - 0.20, h - 0.06, z - 0.31],
                    [a - 0.20, h - 1.0, z - 0.31],
                ],
                g,
                red,
            );
        }
        let tip = x + span * 0.9;
        d.primitive("Box", [tip, h - 1.1, z], [0.10, 2.2, 0.11], 1, g, steel);
        d.primitive("Box", [tip, h - 2.30, z], [0.80, 0.42, 0.53], 1, g, red);
    }
    // Two towers and an illuminated bridge frame the start/finish grid.
    for s in [-1., 1.] {
        d.primitive(
            "Box",
            [-44. + s * 4.4, 2.9, -53.],
            [0.54, 6.0, 0.8],
            1,
            g,
            face,
        );
    }
    d.primitive("Box", [-44., 5.8, -53.], [9.25, 0.55, 0.8], 1, g, steel);
    d.primitive("Box", [-44., 5.7, -53.45], [7.8, 0.12, 0.12], 1, g, stripe);
    Ok(d)
}
fn empty(name: &str) -> Actor {
    let mut a = Actor::cube(name.into());
    a.kind = "Empty".into();
    a.position = [0.; 3];
    a
}
fn component(actor: &mut Actor, id: &str) -> Result<Uuid, String> {
    crate::actor_components::sync(actor);
    actor
        .components
        .iter()
        .find(|c| c.class.class_id.as_deref() == Some(id))
        .map(|c| c.id)
        .ok_or_else(|| format!("{} lacks component {id}", actor.name))
}
fn label(
    scene: &mut Scene,
    parent: usize,
    name: &str,
    value: &str,
    x: f32,
    y: f32,
    width: f32,
    color: [f32; 3],
) -> Result<Uuid, String> {
    let mut a = empty(name);
    a.parent = Some(parent);
    a.rect = Some(hud::RectTransform {
        anchor_min: [0.; 2],
        anchor_max: [0.; 2],
        pivot: [0.; 2],
        position: [x, y],
        size: [width, 18.],
        ..Default::default()
    });
    a.text = Some(hud::Text {
        text: value.into(),
        color,
        wrap: false,
        ..Default::default()
    });
    let id = component(&mut a, crate::object_model::TEXT_COMPONENT_ID)?;
    scene.actors.push(a);
    Ok(id)
}
pub fn create(root: &Path) -> Result<Scene, String> {
    std::fs::create_dir_all(root.join("assets/Race/Source")).map_err(|e| e.to_string())?;
    let mut road = Document::default();
    road.materials.clear();
    for (name, bytes) in FINISHES {
        slot(&mut road, name, [1.; 3]);
        road.materials.last_mut().unwrap().material.texture =
            Some(imported(root, name, bytes, false)?);
    }
    let curve = track(&road.materials)?;
    let first = curve.frames()?[0];
    let livery = imported(root, "kestrel-livery", art!("kestrel-livery.png"), false)?;
    let nozzle = imported(root, "kestrel-nozzle", art!("kestrel-nozzle.png"), false)?;
    let windows = imported(root, "port-windows", art!("port-windows.png"), false)?;
    let barriers = imported(root, "port-hazard", art!("port-hazard.png"), false)?;
    let spark = imported(root, "ion-spark", art!("ion-spark.png"), false)?;
    let buildings = scenery(&curve, windows, barriers)?;
    curve.apply(&mut road)?;
    let mut scene = Scene {
        name: "MeridianCircuit".into(),
        display_size: [320, 240],
        spawn_reserve: Some(0),
        ..Default::default()
    };
    scene.hud_budget = hud::Budget {
        layouts: 16,
        rectangles: 8,
        texts: 12,
        glyphs: 128,
        rotated: 1,
    };
    scene.actors.truncate(1);
    scene.fog = Fog {
        enabled: true,
        start: 28.,
        end: 72.,
        color: [0.30, 0.31, 0.42],
    };
    let camera = &mut scene.actors[0];
    camera.name = "Chase Camera".into();
    camera.camera_fov = 78.;
    camera.camera_sky_color = [0.055, 0.09, 0.20];
    camera.camera_sky_gradient = Some(SkyGradient {
        horizon: [0.66, 0.41, 0.45],
        ground: [0.055, 0.09, 0.16],
    });
    camera.position =
        std::array::from_fn(|i| first.center[i] - first.tangent[i] * 3.0 + first.up[i] * 1.42);
    camera.rotation = crate::spline_path::rotation(first.right, first.up, first.tangent, [0.; 3]);
    crate::actor_components::sync(camera);
    let lens = component(camera, crate::object_model::CAMERA3D_COMPONENT_ID)?;
    let cam = camera.root().ok_or("Race camera lacks transform")?.id;
    scene.actors.push(mesh_actor(
        root,
        "Meridian | racing spline",
        "MeridianTrack",
        road,
    )?);
    scene.actors.push(mesh_actor(
        root,
        "Port Meridian | skyline and course lights",
        "PortMeridian",
        buildings,
    )?);
    let mut ship = mesh_actor(
        root,
        "Kestrel | hovercraft",
        "Kestrel",
        craft(livery, nozzle),
    )?;
    ship.position = std::array::from_fn(|i| first.center[i] + first.up[i] * 0.31);
    ship.rotation = scene.actors[0].rotation;
    let pilot_index = scene.actors.len();
    scene.actors.push(ship);
    let mut jets = empty("Kestrel | ion wake");
    jets.parent = Some(pilot_index);
    jets.position = [0., 0., -0.84];
    jets.particle_emitter = Some(crate::particles::Emitter {
        rate: 12.,
        burst: 0,
        max_particles: 12,
        lifetime: 0.16,
        velocity: [0., 0., -3.],
        spread: [0.19, 0.10, 0.3],
        gravity: [0.; 3],
        start_size: 0.19,
        end_size: 0.06,
        start_color: [0.57, 1., 0.92],
        end_color: [0.12, 0.34, 1.],
        sprite: crate::sprites::Sprite {
            texture: Some(spark),
            pivot: [0.5; 2],
            unlit: true,
            blend: crate::texture::BlendMode::Add,
            ..Default::default()
        },
        ..Default::default()
    });
    let wake = component(&mut jets, crate::actor_components::PARTICLES)?;
    scene.actors.push(jets);
    let mut bindings =
        std::collections::BTreeMap::from([("camera".to_owned(), serde_json::json!(cam))]);
    for (key, name, bytes, volume) in [
        ("motor", "motor", art!("motor.wav"), 0.24),
        ("surge", "surge", art!("surge.wav"), 0.70),
        ("scrape", "edge", art!("edge.wav"), 0.50),
        ("chime", "chime", art!("chime.wav"), 0.58),
    ] {
        let mut sound = empty(name);
        sound.audio = Some(AudioSource {
            clip: Some(imported(root, name, bytes, true)?),
            volume,
            play_on_start: false,
            ..Default::default()
        });
        let id = component(&mut sound, crate::object_model::AUDIO_COMPONENT_ID)?;
        bindings.insert(key.into(), serde_json::json!(id));
        scene.actors.push(sound);
    }
    let canvas = scene.actors.len();
    let mut overlay = empty("Race HUD");
    overlay.canvas = Some(hud::Canvas {
        screen_overlay: true,
        ..Default::default()
    });
    scene.actors.push(overlay);
    let white = [0.96, 0.98, 0.91];
    let aqua = [0.36, 0.95, 0.87];
    let amber = [1., 0.77, 0.35];
    label(&mut scene, canvas, "Lap caption", "LAP", 8., 6., 32., aqua)?;
    label(&mut scene, canvas, "Lap total", "/3", 49., 6., 24., white)?;
    label(
        &mut scene,
        canvas,
        "Time caption",
        "TIME",
        206.,
        6.,
        36.,
        aqua,
    )?;
    label(
        &mut scene,
        canvas,
        "Speed caption",
        "SPD",
        249.,
        215.,
        28.,
        aqua,
    )?;
    for (key, name, value, x, y, w, color) in [
        ("lap_text", "Current lap", "1", 40., 6., 9., white),
        ("timer", "Race time", "00:00.00", 246., 6., 70., white),
        ("speed_text", "Ship speed", "0", 282., 215., 32., white),
        (
            "ready",
            "Start prompt",
            "X / UP: THROTTLE    DOWN: BRAKE",
            37.,
            164.,
            252.,
            amber,
        ),
        (
            "finish",
            "Finish prompt",
            "FINISH   START TO RACE AGAIN",
            49.,
            150.,
            226.,
            amber,
        ),
    ] {
        let id = label(&mut scene, canvas, name, value, x, y, w, color)?;
        bindings.insert(key.into(), serde_json::json!(id));
    }
    scene
        .actors
        .last_mut()
        .unwrap()
        .text
        .as_mut()
        .unwrap()
        .enabled = false;
    let mut plume = mesh_actor(root, "Kestrel | twin exhaust", "KestrelExhaust", exhaust())?;
    plume.parent = Some(pilot_index);
    plume.position = [0., -0.015, -0.96];
    plume.scale = [1., 1., 0.26];
    crate::actor_components::sync(&mut plume);
    let plume_id = plume.root().ok_or("Race exhaust lacks a transform")?.id;
    scene.actors.push(plume);

    let mut charge = empty("charge");
    charge.audio = Some(AudioSource {
        clip: Some(imported(root, "charge", art!("charge.wav"), true)?),
        volume: 0.74,
        play_on_start: false,
        ..Default::default()
    });
    let charge_id = component(&mut charge, crate::object_model::AUDIO_COMPONENT_ID)?;
    scene.actors.push(charge);

    label(
        &mut scene,
        canvas,
        "Boost caption",
        "BOOST",
        217.,
        196.,
        43.,
        aqua,
    )?;
    let mut gauge = empty("Boost charge");
    gauge.parent = Some(canvas);
    gauge.rect = Some(hud::RectTransform {
        anchor_min: [0.; 2],
        anchor_max: [0.; 2],
        pivot: [0.; 2],
        position: [264., 199.],
        size: [47., 5.],
        ..Default::default()
    });
    gauge.progress = Some(hud::ProgressBar {
        value: 0.,
        color: [1., 0.68, 0.24],
        background: [0.12, 0.18, 0.25],
        ..Default::default()
    });
    let gauge_id = component(&mut gauge, crate::object_model::PROGRESS_BAR_COMPONENT_ID)?;
    scene.actors.push(gauge);

    let boost_class = ClassReference::new("RaceBoost", BOOST_CLASS);
    let mut boost = ComponentInstance::new(Uuid::new_v4(), boost_class.clone(), "Boost drive");
    for (key, id) in [
        ("charge_sound", charge_id),
        ("plume", plume_id),
        ("wake", wake),
        ("lens", lens),
        ("gauge", gauge_id),
    ] {
        boost.properties.insert(key.into(), serde_json::json!(id));
    }
    boost
        .properties
        .insert("release_sound".into(), bindings.remove("surge").unwrap());
    boost.overrides.extend(boost.properties.keys().cloned());
    bindings.insert("booster".into(), serde_json::json!(boost.id));
    scene.actors[pilot_index].components.push(boost);
    let class = ClassReference::new("RacePilot", CLASS);
    let mut pilot = ComponentInstance::new(Uuid::new_v4(), class.clone(), "Race Pilot");
    pilot.overrides.extend(bindings.keys().cloned());
    pilot.properties = bindings;
    scene.actors[pilot_index].components.push(pilot);
    for a in &mut scene.actors {
        crate::actor_components::sync(a);
    }
    crate::project::write_changed(
        &root.join(SCRIPT),
        include_bytes!("../templates/RacePilot.lua"),
    )?;
    crate::project::write_changed(
        &root.join(BOOST_SCRIPT),
        include_bytes!("../templates/RaceBoost.lua"),
    )?;
    let mut ids = crate::lua_identity::Registry::default();
    ids.classes.insert(BOOST_SCRIPT.into(), BOOST_CLASS.into());
    ids.classes.insert(SCRIPT.into(), CLASS.into());
    ids.save(root)?;
    crate::template_install::write(
        root,
        Template::Race,
        GameplayFlavor::Lua,
        [("pilot", class), ("boost", boost_class)],
    )?;
    Ok(scene)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fresh_race_exports_the_bound_boost_controller_and_spline() {
        let root = crate::workspace::tests::temp("race-export");
        crate::workspace::create(&root, "MeridianRace", Template::Race).unwrap();
        let map = crate::workspace::startup_scene(&root).unwrap();
        let input = crate::scene_dependencies::Input::load(&map).unwrap();
        let (ship, pilot) = input
            .scene
            .actors
            .iter()
            .find_map(|actor| {
                actor
                    .components
                    .iter()
                    .find(|c| c.class.class_id.as_deref() == Some(CLASS))
                    .map(|pilot| (actor, pilot))
            })
            .unwrap();
        let boost = ship
            .components
            .iter()
            .find(|c| c.class.class_id.as_deref() == Some(BOOST_CLASS))
            .unwrap();
        assert_eq!(
            pilot.properties.get("booster"),
            Some(&serde_json::json!(boost.id))
        );
        let export = crate::export::export_project(&root, &input).unwrap();
        let header = std::fs::read_to_string(export.join("scene.hh")).unwrap();
        assert!(header.contains("::epok::spline_path::bind(spline_paths,1)"));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn race_course_has_two_inversions_and_the_painted_lanes_match_the_markers() {
        let mut doc = Document::default();
        doc.materials.clear();
        for (name, _) in FINISHES {
            slot(&mut doc, name, [1.; 3]);
        }
        let curve = track(&doc.materials).unwrap();
        let frames = curve.frames().unwrap();
        assert_eq!(
            frames
                .windows(2)
                .filter(|w| w[0].up[1] >= 0. && w[1].up[1] < 0.)
                .count(),
            2
        );
        assert!(frames.last().unwrap().distance > 360.);
        let mut angles = [0.; 3];
        for f in &frames {
            angles = crate::spline_path::rotation(f.right, f.up, f.tangent, angles);
            let m = crate::transform::Matrix::trs(f.center, angles, [1.; 3]);
            for (axis, expected) in [
                ([1., 0., 0.], f.right),
                ([0., 1., 0.], f.up),
                ([0., 0., 1.], f.tangent),
            ] {
                assert!(dot(m.vector(axis), expected) > 0.999);
            }
        }
        for (slot, tag) in [(4, 1), (5, 2)] {
            assert!(curve.points.iter().filter(|p| p.tag == tag).count() >= 2);
            assert!(
                curve
                    .points
                    .iter()
                    .all(|p| (p.tag == tag) == (p.surface == Some(doc.materials[slot].id)))
            );
        }
    }
}
