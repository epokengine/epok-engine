//! Cheap moving shadows on horizontal, axis-aligned floor entities.
use crate::{lighting, scene::Scene};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct BlobShadow {
    pub enabled: bool,
    pub radius: f32,
    pub strength: f32,
    pub distance: f32,
}
impl Default for BlobShadow {
    fn default() -> Self {
        Self {
            enabled: true,
            radius: 0.6,
            strength: 0.25,
            distance: 3.,
        }
    }
}
pub struct Blob {
    pub points: [[f32; 3]; 9],
    pub color: [u8; 3],
}
pub fn blobs(scene: &Scene) -> Vec<Blob> {
    let mut out = Vec::new();
    for (i, e) in scene.entities.iter().enumerate() {
        let Some(b) = e.blob_shadow.as_ref().filter(|b| b.enabled) else {
            continue;
        };
        if out.len() >= 32 {
            break;
        }
        let center = scene.world_matrix(i).point([0.; 3]);
        let mut floor = None;
        let mut highest = f32::NEG_INFINITY;
        for (j, f) in scene.entities.iter().enumerate() {
            if i == j || !lighting::tiled(f) {
                continue;
            }
            let m = scene.world_matrix(j);
            if (0..3).any(|r| (0..3).any(|c| r != c && m.0[r][c].abs() > 1. / 4096.)) {
                continue;
            }
            let min = m.point([-0.5; 3]);
            let max = m.point([0.5; 3]);
            let y = max[1];
            if y > center[1]
                || center[1] - y > b.distance
                || y <= highest
                || center[0] < min[0]
                || center[0] > max[0]
                || center[2] < min[2]
                || center[2] > max[2]
            {
                continue;
            }
            highest = y;
            floor = Some((min, max));
        }
        if let Some((min, max)) = floor {
            let color = (255. * b.strength * (1. - (center[1] - highest) / b.distance))
                .clamp(0., 255.) as u8;
            let points = std::array::from_fn(|v| {
                if v == 0 {
                    [center[0], highest + 0.006, center[2]]
                } else {
                    let angle = (v - 1) as f32 * std::f32::consts::FRAC_PI_4;
                    [
                        (center[0] + angle.cos() * b.radius).clamp(min[0], max[0]),
                        highest + 0.006,
                        (center[2] + angle.sin() * b.radius).clamp(min[2], max[2]),
                    ]
                }
            });
            out.push(Blob {
                points,
                color: [color; 3],
            });
        }
    }
    out
}
pub fn inspector(ui: &imgui::Ui, e: &mut crate::scene::Entity) {
    if let Some(b) = &mut e.blob_shadow
        && crate::gui::heading(ui, "Blob Shadow")
    {
        ui.checkbox("Enabled##blob", &mut b.enabled);
        crate::gui::Drag::new(crate::gui::field(ui, "Radius##blob"))
            .speed(0.01)
            .range(0.01, 8.)
            .build(ui, &mut b.radius);
        crate::gui::Drag::new(crate::gui::field(ui, "Strength##blob"))
            .speed(0.01)
            .range(0., 1.)
            .build(ui, &mut b.strength);
        crate::gui::Drag::new(crate::gui::field(ui, "Max Distance##blob"))
            .speed(0.05)
            .range(0.01, 32.)
            .build(ui, &mut b.distance);
        ui.text_wrapped("Soft subtractive shadow on horizontal, axis-aligned Ground meshes. Maximum 32 visible blobs.");
        if ui.small_button("Remove Blob Shadow") {
            e.blob_shadow = None;
        }
    }
}
