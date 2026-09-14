//! Bounded distance fog and UV scrolling with explicit wrap-seam subdivision.
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Fog {
    pub enabled: bool,
    pub start: f32,
    pub end: f32,
    pub color: [f32; 3],
}
impl Default for Fog {
    fn default() -> Self {
        Self {
            enabled: false,
            start: 12.,
            end: 40.,
            color: [0.25, 0.3, 0.4],
        }
    }
}
pub fn validate(scene: &crate::scene::Scene) -> Result<(), String> {
    let f = &scene.fog;
    if !f.start.is_finite()
        || !f.end.is_finite()
        || f.start < 0.
        || f.end > 128.
        || f.end - f.start < 1. / 4096.
        || f.color
            .iter()
            .any(|v| !v.is_finite() || !(0. ..=1.).contains(v))
    {
        return Err("Fog requires 0 ≤ start < end ≤ 128 and color 0..1".into());
    }
    for e in &scene.actors {
        crate::texture::validate_material(&e.material)?;
        if let Some(m) = &e.editable_mesh {
            for material in m.materials.values() {
                crate::texture::validate_material(material)?;
            }
            if let Some(d) = &m.document {
                for slot in &d.materials {
                    crate::texture::validate_material(&slot.material)?;
                }
            }
        }
    }
    Ok(())
}
pub fn fog_color(color: [u8; 3], depth: f32, fog: &Fog) -> [u8; 3] {
    if !fog.enabled {
        return color;
    }
    let amount = ((depth - fog.start) / (fog.end - fog.start)).clamp(0., 1.);
    std::array::from_fn(|i| {
        (color[i] as f32 * (1. - amount) + fog.color[i] * 255. * amount)
            .round()
            .clamp(0., 255.) as u8
    })
}
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub point: [f32; 3],
    pub color: [u8; 3],
    pub uv: [f32; 2],
}
pub fn scroll_triangle(vertices: [Vertex; 3], scroll: [f32; 2], seconds: f32) -> Vec<[Vertex; 3]> {
    if scroll == [0.; 2] {
        return vec![vertices];
    }
    let offset = scroll.map(|v| (v * seconds).rem_euclid(1.));
    let mut base = vertices;
    for v in &mut base {
        for (uv, offset) in v.uv.iter_mut().zip(offset) {
            *uv = uv.clamp(0., 1.) + offset;
        }
    }
    let mut output = vec![];
    for x in 0..=usize::from(offset[0] > 0.) {
        for y in 0..=usize::from(offset[1] > 0.) {
            let origin = [x as f32, y as f32];
            let mut polygon = base.to_vec();
            for plane in 0..4 {
                if polygon.is_empty() {
                    break;
                }
                let axis = plane / 2;
                let distance = |v: Vertex| {
                    if plane % 2 == 0 {
                        v.uv[axis] - origin[axis]
                    } else {
                        origin[axis] + 1. - v.uv[axis]
                    }
                };
                let mut next = vec![];
                let mut previous = *polygon.last().unwrap();
                let mut pd = distance(previous);
                for current in polygon {
                    let cd = distance(current);
                    if (pd >= 0.) != (cd >= 0.) {
                        let t = pd / (pd - cd);
                        next.push(Vertex {
                            point: std::array::from_fn(|c| {
                                previous.point[c] + (current.point[c] - previous.point[c]) * t
                            }),
                            color: std::array::from_fn(|c| {
                                (previous.color[c] as f32
                                    + (current.color[c] as f32 - previous.color[c] as f32) * t)
                                    .round() as u8
                            }),
                            uv: std::array::from_fn(|c| {
                                previous.uv[c] + (current.uv[c] - previous.uv[c]) * t
                            }),
                        });
                    }
                    if cd >= 0. {
                        next.push(current);
                    }
                    previous = current;
                    pd = cd;
                }
                polygon = next;
            }
            for v in &mut polygon {
                for (uv, origin) in v.uv.iter_mut().zip(origin) {
                    *uv = (*uv - origin).clamp(0., 1.);
                }
            }
            for i in 1..polygon.len().saturating_sub(1) {
                output.push([polygon[0], polygon[i], polygon[i + 1]]);
            }
        }
    }
    output
}
pub fn animated(scene: &crate::scene::Scene) -> bool {
    scene.actors.iter().any(|e| {
        e.material.uv_scroll != [0.; 2]
            || e.editable_mesh.as_ref().is_some_and(|m| {
                m.materials.values().any(|m| m.uv_scroll != [0.; 2])
                    || m.document.as_ref().is_some_and(|d| {
                        d.materials.iter().any(|s| s.material.uv_scroll != [0.; 2])
                    })
            })
    })
}
pub fn cpp_setup(scene: &crate::scene::Scene) -> String {
    let f = &scene.fog;
    format!(
        "fog_environment=FogEnvironment{{{},{},{},{{{}}}}};\n",
        f.enabled,
        (f.start * 4096.).round() as i32,
        (f.end * 4096.).round() as i32,
        f.color
            .map(|v| ((v * 255.).round() as u8).to_string())
            .join(",")
    )
}
pub fn inspector(ui: &imgui::Ui, fog: &mut Fog) {
    if crate::gui::heading(ui, "Distance Fog") {
        ui.checkbox("Enabled##fog", &mut fog.enabled);
        crate::gui::Drag::new("Start distance##fog")
            .range(0., 127.)
            .speed(0.1)
            .build(ui, &mut fog.start);
        crate::gui::Drag::new("End distance##fog")
            .range(fog.start + 0.01, 128.)
            .speed(0.1)
            .build(ui, &mut fog.end);
        ui.color_edit3("Fog tint", &mut fog.color);
        ui.text_wrapped("Vertex depth cue before texture modulation; unlit world sprites and meshes also receive fog.");
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wrapped_uvs_preserve_coverage_without_stretched_seam() {
        let p = [
            Vertex {
                point: [0., 0., 0.],
                color: [255, 0, 0],
                uv: [0., 0.],
            },
            Vertex {
                point: [1., 0., 0.],
                color: [0, 255, 0],
                uv: [1., 0.],
            },
            Vertex {
                point: [0., 1., 0.],
                color: [0, 0, 255],
                uv: [0., 1.],
            },
        ];
        for speed in [[0.25, 0.], [0.25, 0.4], [-0.3, -0.5]] {
            let result = scroll_triangle(p, speed, 1.);
            let area = result
                .iter()
                .map(|v| {
                    let a = v[0].point;
                    let b = v[1].point;
                    let c = v[2].point;
                    ((b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])).abs() / 2.
                })
                .sum::<f32>();
            assert!((area - 0.5).abs() < 0.0001);
            assert!(result.len() <= 10);
            assert!(
                result
                    .iter()
                    .flatten()
                    .flat_map(|v| v.uv)
                    .all(|v| (0. ..=1.).contains(&v))
            );
        }
    }
    #[test]
    fn fog_endpoints_and_disabled_are_predictable() {
        let fog = Fog {
            enabled: true,
            start: 2.,
            end: 10.,
            color: [1., 0., 0.],
        };
        assert_eq!(fog_color([0, 100, 200], 2., &fog), [0, 100, 200]);
        assert_eq!(fog_color([0, 100, 200], 10., &fog), [255, 0, 0]);
        assert_eq!(fog_color([0, 100, 200], 6., &fog), [128, 50, 100]);
    }
}
