//! SubViewport2D and SplitScreen2D editing (World2D initiative, phase 4): their Inspector
//! bands, the frame each camera covers drawn in the 2D view, and the stand-in drawn for a
//! sprite that shows a picture (the picture itself exists only while the game runs).
use crate::{
    editor::Editor,
    viewport2d::{SplitScreen2D, SubViewport2D},
    world2d,
    world2d_editor::View,
};
use imgui::Ui;
use uuid::Uuid;

pub const ICON_PICTURE: &str = "\u{f2d2}";
pub const ICON_SPLIT: &str = "\u{f0db}";
const FRAME: [f32; 4] = [0.55, 0.80, 1.0, 0.95];

fn record<T: serde::de::DeserializeOwned + serde::Serialize + Default + PartialEq + Clone>(
    e: &mut Editor,
    actor: Uuid,
    component: &crate::actor_document::ComponentInstance,
    edit: impl FnOnce(&mut Editor, &mut T),
) {
    let mut value: T = world2d::read(component);
    let before = value.clone();
    edit(e, &mut value);
    if value != before
        && let Some(index) = e.scene.actor_index(actor)
        && let Some(c) = e.scene.actors[index]
            .components
            .iter_mut()
            .find(|c| c.id == component.id)
    {
        world2d::write(c, &value);
        e.scene.actors[index].refresh_components();
        e.changed_coalesced("world2d-inspector");
    }
}
fn colour(ui: &Ui, label: &str, value: &mut [u8; 3]) {
    let mut c = value.map(|v| f32::from(v) / 255.);
    if ui
        .color_edit3_config(crate::gui::field(ui, label), &mut c)
        .build()
    {
        *value = c.map(|v| (v.clamp(0., 1.) * 255.).round() as u8);
    }
}
fn is_camera(a: &crate::scene::Actor) -> bool {
    world2d::component(a, world2d::CAMERA2D).is_some()
}
/// Canvas layers the map's CanvasLayer2D nodes use, plus the world canvas.
fn layers(e: &Editor) -> Vec<i8> {
    let mut out: Vec<i8> = vec![0];
    for a in &e.scene.actors {
        if let Some(c) = world2d::component(a, world2d::CANVAS_LAYER2D) {
            let layer = world2d::read::<world2d::CanvasLayer2D>(c).layer;
            if !out.contains(&layer) {
                out.push(layer);
            }
        }
    }
    out.sort_unstable();
    out
}

pub fn band(
    ui: &Ui,
    e: &mut Editor,
    actor: Uuid,
    component: &crate::actor_document::ComponentInstance,
) -> bool {
    let id = component.class.class_id.as_deref().unwrap_or("");
    let (glyph, title) = match id {
        world2d::SUBVIEWPORT2D => (ICON_PICTURE, "SubViewport2D"),
        world2d::SPLIT_SCREEN2D => (ICON_SPLIT, "SplitScreen2D"),
        _ => return false,
    };
    let (mut remove, mut reset) = (false, false);
    let open = crate::gui::section(ui, &format!("{glyph} {title}"), || {
        if crate::gui::script_menu_item(ui, "Reset to Defaults") {
            reset = true;
        }
        if crate::gui::script_menu_item(ui, "Remove Component") {
            remove = true;
        }
    });
    if remove {
        e.remove_actor_component(actor, component.id);
        return true;
    }
    if reset {
        if id == world2d::SUBVIEWPORT2D {
            record::<SubViewport2D>(e, actor, component, |_, v| *v = Default::default());
        } else {
            record::<SplitScreen2D>(e, actor, component, |_, v| *v = Default::default());
        }
        return true;
    }
    if !open {
        return true;
    }
    if id == world2d::SUBVIEWPORT2D {
        picture_band(ui, e, actor, component);
    } else {
        split_band(ui, e, actor, component);
    }
    true
}
fn picture_band(
    ui: &Ui,
    e: &mut Editor,
    actor: Uuid,
    component: &crate::actor_document::ComponentInstance,
) {
    let layers = layers(e);
    record::<SubViewport2D>(e, actor, component, |e, v| {
        crate::world2d_shapes_editor::actor_combo(
            ui,
            e,
            "Camera",
            "None: the active camera",
            actor,
            &is_camera,
            &mut v.camera,
        );
        let mut size = v.size.map(i32::from);
        if crate::gui::Drag::new(crate::gui::field(ui, "Size (px)"))
            .speed(1.)
            .range(8, 256)
            .build_array(ui, &mut size)
        {
            v.size = size.map(|s| s.clamp(8, 256) as u16);
        }
        let [w, h] = v.picture();
        ui.text_disabled(format!(
            "{w} x {h} picture, {:.1} KiB of VRAM (15-bit colour; width rounds to 16)",
            v.vram_bytes() as f32 / 1024.
        ));
        colour(ui, "Background", &mut v.background);
        let mut interval = i32::from(v.interval);
        if ui
            .input_int(
                crate::gui::field(ui, "Redraw every (frames)"),
                &mut interval,
            )
            .build()
        {
            v.interval = interval.clamp(1, 255) as u8;
        }
        let names: Vec<String> = std::iter::once("Every world layer".to_string())
            .chain(layers.iter().map(|l| format!("Canvas layer {l}")))
            .collect();
        let mut selected = v
            .layer
            .and_then(|l| layers.iter().position(|x| *x == l))
            .map_or(0, |i| i + 1);
        if ui.combo_simple_string(crate::gui::field(ui, "Draws"), &mut selected, &names) {
            v.layer = (selected > 0).then(|| layers[selected - 1]);
        }
        if v.layer.is_some() {
            crate::gui::toggle(ui, "Hide layer elsewhere", &mut v.hide_layer);
            if ui.is_item_hovered() {
                ui.tooltip_text("Only this picture shows the layer: a sprite can then draw it as one image, faded or tinted as a whole");
            }
        }
        crate::gui::toggle(ui, "Enabled", &mut v.enabled);
    });
    let shown = e.scene.actors.iter().any(|a| {
        world2d::component(a, world2d::SPRITE2D)
            .is_some_and(|c| world2d::read::<world2d::Sprite2D>(c).viewport == Some(actor))
    });
    if shown {
        crate::gui::muted(
            ui,
            "Shown by a Sprite2D. The picture is drawn while the game runs.",
        );
    } else {
        ui.text_colored(
            [1., 0.75, 0.35, 1.],
            "No sprite shows it yet: pick this node as a Sprite2D's Viewport.",
        );
    }
}
fn split_band(
    ui: &Ui,
    e: &mut Editor,
    actor: Uuid,
    component: &crate::actor_document::ComponentInstance,
) {
    record::<SplitScreen2D>(e, actor, component, |e, v| {
        let [mut a, mut b] = v.cameras;
        let labels = if v.side_by_side {
            ["Left camera", "Right camera"]
        } else {
            ["Top camera", "Bottom camera"]
        };
        crate::world2d_shapes_editor::actor_combo(
            ui, e, labels[0], "None", actor, &is_camera, &mut a,
        );
        crate::world2d_shapes_editor::actor_combo(
            ui, e, labels[1], "None", actor, &is_camera, &mut b,
        );
        v.cameras = [a, b];
        let mut layout = usize::from(!v.side_by_side);
        if crate::world2d_editor::segmented(ui, "Layout", &["Side by side", "Stacked"], &mut layout)
        {
            v.side_by_side = layout == 0;
        }
        let mut divider = i32::from(v.divider);
        if ui
            .input_int(crate::gui::field(ui, "Divider (px)"), &mut divider)
            .build()
        {
            v.divider = divider.clamp(0, 32) as u8;
        }
        colour(ui, "Divider colour", &mut v.divider_color);
        crate::gui::toggle(ui, "Enabled", &mut v.enabled);
    });
    let split: SplitScreen2D = world2d::read(component);
    if split.cameras.iter().any(Option::is_none) {
        ui.text_colored(
            [1., 0.75, 0.35, 1.],
            "Pick both cameras: each Camera2D follows its own node (one per player).",
        );
    } else {
        crate::gui::muted(
            ui,
            "Each half follows its camera; screen canvases (the HUD) draw once over both. The view shows what each camera frames.",
        );
    }
}

// ---- 2D view -----------------------------------------------------------------------------

/// The world rectangle a camera frames through a `size` viewport.
fn frame_of(e: &Editor, camera: Uuid, size: [f32; 2]) -> Option<[[f32; 2]; 2]> {
    let index = e.scene.actor_index(camera)?;
    let c: world2d::Camera2D = world2d::read(world2d::component(
        &e.scene.actors[index],
        world2d::CAMERA2D,
    )?);
    let at = world2d::world_position(&e.scene, index);
    let zoom = c.zoom.max(0.0625);
    let half = [size[0] * 0.5 / zoom, size[1] * 0.5 / zoom];
    let centre = [at[0] + c.offset[0], at[1] + c.offset[1]];
    Some([
        [centre[0] - half[0], centre[1] - half[1]],
        [centre[0] + half[0], centre[1] + half[1]],
    ])
}
fn draw_frame(
    draw: &imgui::DrawListMut,
    frame: [[f32; 2]; 2],
    to_screen: &dyn Fn([f32; 2]) -> [f32; 2],
    label: &str,
) {
    let (a, b) = (to_screen(frame[0]), to_screen(frame[1]));
    draw.add_rect(a, b, FRAME).thickness(1.5).build();
    draw.add_text([a[0] + 3., a[1] + 2.], FRAME, label);
}
/// The selected picture's or split screen's camera frames.
pub fn overlay(e: &Editor, draw: &imgui::DrawListMut, to_screen: &dyn Fn([f32; 2]) -> [f32; 2]) {
    let Some(selected) = e.selected_actor else {
        return;
    };
    let Some(index) = e.scene.actor_index(selected) else {
        return;
    };
    let actor = &e.scene.actors[index];
    if let Some(c) = world2d::component(actor, world2d::SUBVIEWPORT2D) {
        let v: SubViewport2D = world2d::read(c);
        let [w, h] = v.picture().map(f32::from);
        let camera = v.camera.or_else(|| {
            e.scene
                .actors
                .iter()
                .find(|a| {
                    world2d::component(a, world2d::CAMERA2D)
                        .is_some_and(|c| world2d::read::<world2d::Camera2D>(c).current)
                })
                .map(|a| a.id)
        });
        if let Some(frame) = camera.and_then(|camera| frame_of(e, camera, [w, h])) {
            draw_frame(
                draw,
                frame,
                to_screen,
                &format!("{ICON_PICTURE} {}", actor.name),
            );
        }
    }
    if let Some(c) = world2d::component(actor, world2d::SPLIT_SCREEN2D) {
        let split: SplitScreen2D = world2d::read(c);
        for (i, camera) in split.cameras.iter().enumerate() {
            let half = crate::viewport2d::half(&split, e.scene.display_size, i);
            let size = [half[2] as f32, half[3] as f32];
            if let Some(frame) = camera.and_then(|camera| frame_of(e, camera, size)) {
                draw_frame(draw, frame, to_screen, &format!("{ICON_SPLIT} {}", i + 1));
            }
        }
    }
}
/// The local rectangle a sprite showing a picture covers, as the runtime draws it.
pub fn picture_rect(e: &Editor, sprite: &world2d::Sprite2D) -> Option<([f32; 2], [f32; 2])> {
    let index = e.scene.actor_index(sprite.viewport?)?;
    let v: SubViewport2D = world2d::read(world2d::component(
        &e.scene.actors[index],
        world2d::SUBVIEWPORT2D,
    )?);
    let [w, h] = v.picture().map(f32::from);
    let left = if sprite.centered { -w * 0.5 } else { 0. } + sprite.offset[0];
    let top = if sprite.centered { -h * 0.5 } else { 0. } + sprite.offset[1];
    Some(([left, top], [left + w, top + h]))
}
/// Stand-ins for sprites that show a picture: the picture's frame, its background colour
/// and the name of its node.
pub fn draw_pictures(
    e: &Editor,
    draw: &imgui::DrawListMut,
    views: &[View],
    to_screen: &dyn Fn([f32; 2]) -> [f32; 2],
) {
    for view in views {
        let Some((_, sprite)) = &view.sprite else {
            continue;
        };
        let Some(source) = sprite.viewport else {
            continue;
        };
        let Some((lo, hi)) = picture_rect(e, sprite) else {
            continue;
        };
        let Some(index) = e.scene.actor_index(source) else {
            continue;
        };
        let v: SubViewport2D = world2d::read(
            match world2d::component(&e.scene.actors[index], world2d::SUBVIEWPORT2D) {
                Some(c) => c,
                None => continue,
            },
        );
        let corners = [
            [lo[0], lo[1]],
            [hi[0], lo[1]],
            [hi[0], hi[1]],
            [lo[0], hi[1]],
        ]
        .map(|p| to_screen(view.world.point(p)));
        let bg = v.background.map(|c| f32::from(c) / 255.);
        draw.add_polyline(corners.to_vec(), [bg[0], bg[1], bg[2], 0.9])
            .filled(true)
            .build();
        draw.add_polyline(corners.to_vec(), FRAME)
            .thickness(1.5)
            .build();
        let name = &e.scene.actors[index].name;
        draw.add_text(
            [corners[0][0] + 4., corners[0][1] + 3.],
            FRAME,
            format!("{ICON_PICTURE} {name}"),
        );
    }
}
