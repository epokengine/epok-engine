//! Palette animation rotates quantized PSX CLUT entries, preserving index zero.
use crate::{assets, scene::Scene, texture::Data};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Animator {
    pub enabled: bool,
    pub texture: Option<Uuid>,
    pub first: u8,
    pub last: u8,
    pub speed: f32,
    pub reverse: bool,
}
impl Default for Animator {
    fn default() -> Self {
        Self {
            enabled: true,
            texture: None,
            first: 1,
            last: 2,
            speed: 8.,
            reverse: false,
        }
    }
}
pub fn validate(scene: &Scene) -> Result<(), String> {
    let mut used = std::collections::BTreeSet::new();
    for entity in &scene.actors {
        let Some(a) = &entity.palette_animator else {
            continue;
        };
        if a.first == 0
            || a.last <= a.first
            || !a.speed.is_finite()
            || !(0.01..=60.).contains(&a.speed)
            || a.texture.is_some_and(|id| id.is_nil())
        {
            return Err(format!(
                "{}: palette animation requires indices 1..255 with first < last and speed 0.01..60",
                entity.name
            ));
        }
        if let Some(id) = a.texture {
            if a.enabled && !used.insert(id) {
                return Err(
                    "Only one enabled Palette Animator can control a texture in a scene".into(),
                );
            }
            if let Some(data) = scene.textures.get(&id)
                && (a.first..=a.last).any(|index| {
                    data.palette
                        .get(usize::from(index))
                        .is_none_or(|color| *color == 0)
                })
            {
                return Err(format!(
                    "{}: palette cycle range includes unused/transparent colors",
                    entity.name
                ));
            }
        }
    }
    Ok(())
}
pub fn offset(animator: &Animator, seconds: f32) -> usize {
    if !animator.enabled
        || animator.first == 0
        || animator.last <= animator.first
        || !seconds.is_finite()
        || seconds <= 0.
    {
        return 0;
    }
    let ticks = (seconds * 60.).floor().min(u32::MAX as f32) as u64;
    let elapsed_raw = ticks * 4096 / 60;
    let speed_raw = (animator.speed * 4096.).round().max(0.) as u64;
    let count = u64::from(animator.last - animator.first) + 1;
    let step = (u128::from(elapsed_raw) * u128::from(speed_raw) / (4096 * 4096) % u128::from(count))
        as u64;
    if animator.reverse {
        ((count - step) % count) as usize
    } else {
        step as usize
    }
}
pub fn preview_rgba(scene: &Scene, id: Uuid, seconds: f32, data: &Data) -> Vec<u8> {
    let animator = scene.actors.iter().enumerate().find_map(|(index, e)| {
        e.palette_animator
            .as_ref()
            .filter(|a| scene.is_active(index) && a.enabled && a.texture == Some(id))
    });
    let Some(animator) = animator else {
        return data.rgba.clone();
    };
    let shift = offset(animator, seconds);
    if shift == 0 {
        return data.rgba.clone();
    }
    let row_words = usize::from(data.width).div_ceil(4) * 2;
    let mut output = Vec::with_capacity(data.rgba.len());
    for y in 0..usize::from(data.height) {
        for x in 0..usize::from(data.width) {
            let index = usize::from((data.words[y * row_words + x / 2] >> ((x % 2) * 8)) & 255);
            let source =
                if index >= usize::from(animator.first) && index <= usize::from(animator.last) {
                    usize::from(animator.first)
                        + (index - usize::from(animator.first) + shift)
                            % (usize::from(animator.last - animator.first) + 1)
                } else {
                    index
                };
            let color = data.palette[source];
            for channel in 0..3 {
                output.push((((color >> (channel * 5)) & 31) * 255 / 31) as u8);
            }
            output.push(if index == 0 { 0 } else { 255 });
        }
    }
    output
}
pub fn cpp_setup(scene: &Scene) -> String {
    use std::fmt::Write;
    let mut output = String::new();
    for (index, entity) in scene.actors.iter().enumerate() {
        if let Some(a) = &entity.palette_animator {
            writeln!(output, "objects[{index}].palette_animator.enabled={};objects[{index}].palette_animator.texture={};objects[{index}].palette_animator.first={};objects[{index}].palette_animator.last={};objects[{index}].palette_animator.speed=Fixed({},Fixed::RAW);objects[{index}].palette_animator.reverse={};", a.enabled, a.texture.map(crate::texture::symbol).unwrap_or("-1".into()), a.first, a.last, (a.speed * 4096.).round() as i32, a.reverse).unwrap();
        }
    }
    output
}
pub fn inspector(ui: &imgui::Ui, editor: &crate::editor::Editor, entity: &mut crate::scene::Actor) {
    if entity.palette_animator.is_none() {
        return;
    }
    let mut remove = false;
    let open = crate::gui::section(ui, "Palette Animator", || {
        remove = ui.menu_item("Remove Palette Animator");
    });
    if remove {
        entity.palette_animator = None;
        ui.separator();
        return;
    }
    if !open {
        return;
    }
    let a = entity.palette_animator.as_mut().unwrap();
    crate::gui::toggle(ui, "Enabled##palette", &mut a.enabled);
    let label = a
        .texture
        .and_then(|id| editor.assets.index.resolve(id).ok())
        .map(|r| r.meta.source.as_str())
        .unwrap_or("None");
    if let Some(_combo) = ui.begin_combo(crate::gui::field(ui, "Texture##palette"), label) {
        if ui.selectable("None") {
            a.texture = None;
        }
        for record in editor
            .assets
            .index
            .usable()
            .filter(|r| r.meta.kind == assets::Kind::Texture)
        {
            if ui.selectable(&record.meta.source) {
                a.texture = Some(record.meta.id);
            }
        }
    }
    let mut range = [i32::from(a.first), i32::from(a.last)];
    if crate::gui::Drag::new(crate::gui::field(ui, "First / Last##palette"))
        .range(1, 255)
        .build_array(ui, &mut range)
    {
        a.first = range[0].clamp(1, 254) as u8;
        a.last = range[1].clamp(i32::from(a.first) + 1, 255) as u8;
    }
    crate::gui::Drag::new(crate::gui::field(ui, "Steps per second##palette"))
        .range(0.01, 60.)
        .speed(0.1)
        .build(ui, &mut a.speed);
    crate::gui::toggle(ui, "Reverse##palette", &mut a.reverse);
    ui.text_wrapped("Cycles quantized colors for every use of this texture. Index 0 remains transparent. One enabled animator per texture.");
    if let Some(data) = a.texture.and_then(|id| editor.scene.textures.get(&id)) {
        let available = data
            .palette
            .iter()
            .skip(1)
            .take_while(|color| **color != 0)
            .count();
        ui.text(format!("Palette colors available: 1–{available}"));
    }
    ui.separator();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn q12_preview_phase_and_quantized_color_rotation() {
        let id = Uuid::new_v4();
        let mut scene = Scene::default();
        scene.actors[0].palette_animator = Some(Animator {
            texture: Some(id),
            speed: 1.,
            ..Default::default()
        });
        let mut palette = vec![0; 256];
        palette[1] = 0x801f;
        palette[2] = 0x83e0;
        let data = Data {
            width: 3,
            height: 1,
            words: vec![0x0100, 0x0002],
            palette,
            rgba: vec![0, 0, 0, 0, 255, 0, 0, 255, 0, 255, 0, 255],
        };
        assert_eq!(preview_rgba(&scene, id, 0.5, &data), data.rgba);
        assert_eq!(
            preview_rgba(&scene, id, 1., &data),
            [0, 0, 0, 0, 0, 255, 0, 255, 255, 0, 0, 255]
        );
        assert_eq!(
            offset(scene.actors[0].palette_animator.as_ref().unwrap(), 2.),
            0
        );
        let mut a = Animator {
            first: 1,
            last: 4,
            speed: 8.,
            ..Default::default()
        };
        assert_eq!(offset(&a, 0.25), 2);
        a.reverse = true;
        assert_eq!(offset(&a, 0.125), 0); // 7 fixed ticks are still below one step.
        assert_eq!(offset(&a, 0.15), 3);
    }
    #[test]
    fn serialization_validation_and_export() {
        let mut scene = Scene::default();
        let id = Uuid::new_v4();
        scene.actors[0].palette_animator = Some(Animator {
            texture: Some(id),
            ..Default::default()
        });
        let decoded: Scene = serde_json::from_str(&serde_json::to_string(&scene).unwrap()).unwrap();
        assert_eq!(scene, decoded);
        validate(&scene).unwrap();
        assert!(cpp_setup(&scene).contains("palette_animator.speed=Fixed(32768,Fixed::RAW)"));
        scene.actors[1].palette_animator = scene.actors[0].palette_animator.clone();
        assert!(validate(&scene).is_err());
        scene.actors[1].palette_animator = None;
        scene.actors[0].palette_animator.as_mut().unwrap().first = 0;
        assert!(validate(&scene).is_err());
    }
}
