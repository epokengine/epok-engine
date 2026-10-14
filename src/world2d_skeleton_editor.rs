//! Skeleton2D editing (World2D initiative, phase 4): the Inspector band of a skeleton,
//! the compact band shown on each of its bones, the pose-key timeline, IK chains, the
//! bone overlay in the 2D view and the animation preview.
//!
//! Bones are ordinary attached actors, so posing is the editor's own move and rotate
//! tools. Scrubbing the timeline poses the bones (an undoable edit, like any transform
//! change); the preview plays the animation without touching the undo history and puts
//! the pose back when it stops.
use crate::{
    editor::Editor,
    skeleton2d::{self, Animation, Bone, Ik, Skeleton2D},
    world2d,
    world2d_editor::{Affine, View},
};
use imgui::Ui;
use uuid::Uuid;

pub const ICON_SKELETON: &str = "\u{f5d7}";
const BONE: [f32; 4] = [0.96, 0.76, 0.33, 1.];
const BONE_SELECTED: [f32; 4] = [1., 0.93, 0.62, 1.];
const IK_COLOUR: [f32; 4] = [0.45, 0.80, 1., 1.];
const KEY_COLOUR: [f32; 4] = [0.96, 0.76, 0.33, 1.];

#[derive(Default)]
pub struct State {
    /// Timeline position and animation, shared by the skeleton and its bones' bands.
    pub time: f32,
    pub animation: usize,
    pub key: Option<usize>,
    playing: Option<Playback>,
}
struct Playback {
    actor: Uuid,
    component: Uuid,
    started: f32,
    last: std::time::Instant,
    /// The bones' transforms before the preview, put back when it stops.
    saved: Vec<(Uuid, crate::scene_view_mode::Transform2D)>,
}

// ---- document access -------------------------------------------------------------------

/// The skeleton that owns `actor` or names it as a bone: (owner actor, component id).
pub fn skeleton_of(e: &Editor, actor: Uuid) -> Option<(Uuid, Uuid)> {
    let own = e.scene.actor_index(actor).and_then(|i| {
        world2d::component(&e.scene.actors[i], world2d::SKELETON2D).map(|c| (actor, c.id))
    });
    own.or_else(|| {
        e.scene.actors.iter().find_map(|a| {
            let c = world2d::component(a, world2d::SKELETON2D)?;
            let s: Skeleton2D = world2d::read(c);
            s.bones
                .iter()
                .any(|b| b.actor == Some(actor))
                .then_some((a.id, c.id))
        })
    })
}
fn read(e: &Editor, actor: Uuid, component: Uuid) -> Option<Skeleton2D> {
    let index = e.scene.actor_index(actor)?;
    let c = e.scene.actors[index]
        .components
        .iter()
        .find(|c| c.id == component)?;
    Some(world2d::read(c))
}
fn store(e: &mut Editor, actor: Uuid, component: Uuid, skeleton: &Skeleton2D) {
    if let Some(index) = e.scene.actor_index(actor)
        && let Some(c) = e.scene.actors[index]
            .components
            .iter_mut()
            .find(|c| c.id == component)
    {
        let mut value = skeleton.clone();
        value.normalize();
        world2d::write(c, &value);
        e.scene.actors[index].refresh_components();
        e.changed_coalesced("skeleton2d-inspector");
    }
}
fn name_of(e: &Editor, actor: Option<Uuid>) -> String {
    actor
        .and_then(|id| e.scene.actors.iter().find(|a| a.id == id))
        .map_or_else(|| "(missing)".into(), |a| a.name.clone())
}
/// The bones' current transforms as a pose (rotation, x, y); a missing actor keeps its rest.
fn current_pose(e: &Editor, skeleton: &Skeleton2D) -> Vec<[f32; 3]> {
    skeleton
        .bones
        .iter()
        .map(|b| {
            b.actor
                .and_then(|id| e.scene.actor_index(id))
                .map_or(b.rest, |i| {
                    let t = crate::world2d_editor::root_transform(&e.scene.actors[i]);
                    [t.rotation, t.position[0], t.position[1]]
                })
        })
        .collect()
}
/// Writes `pose` into the bones' transforms. The caller records (or not) the change.
fn set_pose(e: &mut Editor, skeleton: &Skeleton2D, pose: &[[f32; 3]]) {
    for (bone, p) in skeleton.bones.iter().zip(pose) {
        if let Some(id) = bone.actor {
            crate::world2d_editor::edit_transform(e, id, |t| {
                t.rotation = p[0];
                t.position = [p[1], p[2]];
            });
        }
    }
}
/// Actors attached below `root`, depth first, in map order.
fn attached_parts(e: &Editor, root: Uuid) -> Vec<Uuid> {
    let mut out = Vec::new();
    let mut stack = vec![root];
    while let Some(parent) = stack.pop() {
        let children: Vec<Uuid> = e
            .scene
            .actors
            .iter()
            .filter(|a| a.attach.as_ref().is_some_and(|at| at.actor == parent))
            .map(|a| a.id)
            .collect();
        for child in children.iter().rev() {
            if !out.contains(child) && *child != root {
                stack.push(*child);
            }
        }
        if parent != root {
            out.push(parent);
        }
        if out.len() > skeleton2d::MAX_BONES * 4 {
            break;
        }
    }
    out
}
fn parent_of(e: &Editor, actor: Uuid) -> Option<Uuid> {
    let i = e.scene.actor_index(actor)?;
    e.scene.actors[i].attach.as_ref().map(|a| a.actor)
}
fn rest_of(e: &Editor, actor: Uuid) -> [f32; 3] {
    e.scene.actor_index(actor).map_or([0.; 3], |i| {
        let t = crate::world2d_editor::root_transform(&e.scene.actors[i]);
        [t.rotation, t.position[0], t.position[1]]
    })
}

// ---- preview ---------------------------------------------------------------------------

fn stop_preview(e: &mut Editor) {
    if let Some(playback) = e.world2d.skeleton.playing.take() {
        for (id, saved) in playback.saved {
            crate::world2d_editor::edit_transform(e, id, |t| *t = saved);
        }
        e.world2d.skeleton.time = playback.started;
        e.view_dirty = true;
    }
}
fn start_preview(e: &mut Editor, actor: Uuid, component: Uuid, skeleton: &Skeleton2D) {
    stop_preview(e);
    let saved = skeleton
        .bones
        .iter()
        .filter_map(|b| {
            let id = b.actor?;
            let i = e.scene.actor_index(id)?;
            Some((
                id,
                crate::world2d_editor::root_transform(&e.scene.actors[i]),
            ))
        })
        .collect();
    e.world2d.skeleton.playing = Some(Playback {
        actor,
        component,
        started: e.world2d.skeleton.time,
        last: std::time::Instant::now(),
        saved,
    });
}
/// Advances the preview; called by the 2D view every frame.
pub fn tick(e: &mut Editor) {
    let Some(playback) = &mut e.world2d.skeleton.playing else {
        return;
    };
    let now = std::time::Instant::now();
    let dt = now.duration_since(playback.last).as_secs_f32().min(0.1);
    playback.last = now;
    let (actor, component) = (playback.actor, playback.component);
    let Some(skeleton) = read(e, actor, component).filter(|_| !e.playing) else {
        stop_preview(e);
        return;
    };
    let index = e.world2d.skeleton.animation;
    let Some(animation) = skeleton.animations.get(index) else {
        stop_preview(e);
        return;
    };
    let mut time = e.world2d.skeleton.time + dt * skeleton.speed;
    if animation.looping {
        time = time.rem_euclid(animation.length);
    } else if time >= animation.length {
        time = animation.length;
    }
    e.world2d.skeleton.time = time;
    if let Some(pose) = skeleton.sample(index, time) {
        set_pose(e, &skeleton, &pose);
        e.view_dirty = true;
    }
    // A one-shot animation ends its preview on its last pose.
    if !animation.looping && time >= animation.length {
        stop_preview(e);
    }
}
pub fn previewing(e: &Editor) -> bool {
    e.world2d.skeleton.playing.is_some()
}

// ---- Inspector -------------------------------------------------------------------------

/// The Skeleton2D band of the skeleton's own actor. False for other components.
pub fn band(
    ui: &Ui,
    e: &mut Editor,
    actor: Uuid,
    component: &crate::actor_document::ComponentInstance,
) -> bool {
    if component.class.class_id.as_deref() != Some(world2d::SKELETON2D) {
        return false;
    }
    let (mut remove, mut reset) = (false, false);
    let open = crate::gui::section(ui, &format!("{ICON_SKELETON} Skeleton2D"), || {
        if crate::gui::script_menu_item(ui, "Reset to Defaults") {
            reset = true;
        }
        if crate::gui::script_menu_item(ui, "Remove Component") {
            remove = true;
        }
    });
    if remove {
        stop_preview(e);
        e.remove_actor_component(actor, component.id);
        return true;
    }
    if reset {
        stop_preview(e);
        store(e, actor, component.id, &Skeleton2D::default());
        e.changed();
        return true;
    }
    if !open {
        return true;
    }
    let mut s: Skeleton2D = world2d::read(component);
    let before = s.clone();
    bones_section(ui, e, actor, &mut s);
    ui.spacing();
    animations_section(ui, e, actor, component.id, &mut s);
    ui.spacing();
    ik_section(ui, e, actor, component.id, &mut s);
    if s != before {
        store(e, actor, component.id, &s);
    }
    true
}
/// The compact band on an actor that is a bone of some skeleton: the timeline and the
/// key button, so a pose is keyed right where the part was rotated.
pub fn bone_band(ui: &Ui, e: &mut Editor, actor: Uuid) {
    let Some((owner, component)) = skeleton_of(e, actor).filter(|(owner, _)| *owner != actor)
    else {
        return;
    };
    let Some(mut s) = read(e, owner, component) else {
        return;
    };
    let open = crate::gui::section(
        ui,
        &format!("{ICON_SKELETON} Bone of {}", name_of(e, Some(owner))),
        || {},
    );
    if !open {
        return;
    }
    if ui.small_button("Select skeleton") {
        e.select_actor(Some(owner));
        return;
    }
    let before = s.clone();
    timeline(ui, e, owner, component, &mut s);
    if s != before {
        store(e, owner, component, &s);
    }
}

fn bones_section(ui: &Ui, e: &mut Editor, actor: Uuid, s: &mut Skeleton2D) {
    ui.text(format!(
        "Bones ({}/{})",
        s.bones.len(),
        skeleton2d::MAX_BONES
    ));
    let parts = attached_parts(e, actor);
    // IK tips and targets are parts too, but they are not animated as bones.
    let missing: Vec<Uuid> = parts
        .iter()
        .copied()
        .filter(|id| !s.bones.iter().any(|b| b.actor == Some(*id)))
        .filter(|id| {
            !s.ik
                .iter()
                .any(|c| c.tip == Some(*id) || c.target == Some(*id))
        })
        .collect();
    if s.bones.is_empty() {
        crate::gui::muted(
            ui,
            "Bones are this actor's attached parts: add Sprite2D nodes (head, arms, legs), attach them to each other in the Hierarchy, then use them as bones.",
        );
    }
    if !missing.is_empty() {
        let label = match (s.bones.is_empty(), missing.len()) {
            (true, 1) => "Use the attached part as a bone".to_string(),
            (true, n) => format!("Use the {n} attached parts as bones"),
            (false, 1) => format!("Add {} as a bone", name_of(e, missing.first().copied())),
            (false, n) => format!("Add {n} new attached parts as bones"),
        };
        if ui.button(label) {
            for id in missing {
                if s.bones.len() >= skeleton2d::MAX_BONES {
                    break;
                }
                s.bones.push(Bone {
                    actor: Some(id),
                    rest: rest_of(e, id),
                });
            }
            s.normalize();
        }
    }
    let selected = e.selected_actor;
    let mut remove = None;
    let mut select = None;
    for (i, bone) in s.bones.iter().enumerate() {
        let _id = ui.push_id_usize(i);
        let depth = {
            let mut d = 0;
            let mut cur = bone.actor;
            while let Some(p) = cur.and_then(|id| parent_of(e, id)) {
                if p == actor || d > 16 {
                    break;
                }
                d += 1;
                cur = Some(p);
            }
            d
        };
        ui.indent_by(8. + depth as f32 * 10.);
        if ui
            .selectable_config(format!("{} {}", ICON_SKELETON, name_of(e, bone.actor)))
            .selected(bone.actor.is_some() && bone.actor == selected)
            .size([ui.content_region_avail()[0] - 26., 0.])
            .build()
        {
            select = bone.actor;
        }
        if ui.is_item_hovered() {
            ui.tooltip_text("Select this part to move or rotate it");
        }
        ui.same_line();
        if ui.small_button("x") {
            remove = Some(i);
        }
        ui.unindent_by(8. + depth as f32 * 10.);
    }
    if let Some(id) = select {
        e.select_actor(Some(id));
    }
    if let Some(i) = remove {
        s.remove_bone(i);
    }
    if !s.bones.is_empty() {
        if ui.small_button("Set rest pose") {
            let pose = current_pose(e, s);
            for (bone, p) in s.bones.iter_mut().zip(pose) {
                bone.rest = p;
            }
        }
        if ui.is_item_hovered() {
            ui.tooltip_text("The bones' current transforms become the rest pose");
        }
        ui.same_line();
        if ui.small_button("Go to rest pose") {
            stop_preview(e);
            let rest: Vec<[f32; 3]> = s.bones.iter().map(|b| b.rest).collect();
            set_pose(e, s, &rest);
            e.changed();
        }
    }
}

fn animations_section(ui: &Ui, e: &mut Editor, actor: Uuid, component: Uuid, s: &mut Skeleton2D) {
    ui.text("Animations");
    let state = &mut e.world2d.skeleton;
    if state.animation >= s.animations.len() {
        state.animation = s.animations.len().saturating_sub(1);
    }
    let mut current = state.animation;
    let names: Vec<String> = s.animations.iter().map(|a| a.name.clone()).collect();
    if !names.is_empty() {
        ui.set_next_item_width(ui.content_region_avail()[0] - 60.);
        if let Some(_combo) = ui.begin_combo("##animation", &names[current]) {
            for (i, name) in names.iter().enumerate() {
                if ui
                    .selectable_config(format!("{name}##{i}"))
                    .selected(i == current)
                    .build()
                {
                    current = i;
                }
            }
        }
        ui.same_line();
    }
    if ui.small_button("+") && s.animations.len() < skeleton2d::MAX_ANIMATIONS {
        let mut n = s.animations.len() + 1;
        while s
            .animations
            .iter()
            .any(|a| a.name == format!("animation {n}"))
        {
            n += 1;
        }
        s.animations.push(Animation {
            name: if s.animations.is_empty() {
                "idle".into()
            } else {
                format!("animation {n}")
            },
            ..Default::default()
        });
        current = s.animations.len() - 1;
    }
    if ui.is_item_hovered() {
        ui.tooltip_text("New animation");
    }
    if !names.is_empty() {
        ui.same_line();
        if ui.small_button("x") {
            stop_preview(e);
            s.animations.remove(current);
            s.autoplay = match s.autoplay {
                Some(a) if a == current => None,
                Some(a) if a > current => Some(a - 1),
                other => other,
            };
            current = current.saturating_sub(1);
        }
        if ui.is_item_hovered() {
            ui.tooltip_text("Delete this animation");
        }
    }
    if current != e.world2d.skeleton.animation {
        stop_preview(e);
        e.world2d.skeleton.animation = current;
        e.world2d.skeleton.key = None;
        e.world2d.skeleton.time = 0.;
    }
    if s.animations.is_empty() {
        crate::gui::muted(
            ui,
            "Add an animation, then pose the bones and key them on its timeline.",
        );
        return;
    }
    let a = &mut s.animations[current];
    ui.input_text(crate::gui::field(ui, "Name"), &mut a.name)
        .build();
    crate::gui::Drag::new(crate::gui::field(ui, "Length (s)"))
        .speed(0.01)
        .range(0.05, 600.)
        .display_format("%.2f")
        .build(ui, &mut a.length);
    crate::gui::toggle(ui, "Loop", &mut a.looping);
    let mut autoplay = s.autoplay == Some(current);
    if crate::gui::toggle(ui, "Plays on start", &mut autoplay) {
        s.autoplay = autoplay.then_some(current);
    }
    crate::gui::Drag::new(crate::gui::field(ui, "Speed"))
        .speed(0.01)
        .range(-8., 8.)
        .display_format("%.2fx")
        .build(ui, &mut s.speed);
    timeline(ui, e, actor, component, s);
}

/// The key strip, the scrubber and the key buttons of the current animation.
fn timeline(ui: &Ui, e: &mut Editor, actor: Uuid, component: Uuid, s: &mut Skeleton2D) {
    let index = e.world2d.skeleton.animation;
    if index >= s.animations.len() {
        return;
    }
    let length = s.animations[index].length.max(0.05);
    let width = ui.content_region_avail()[0].max(80.);
    let height = 30.;
    let origin = ui.cursor_screen_pos();
    ui.invisible_button("##skeleton-timeline", [width, height]);
    let hovered = ui.is_item_hovered();
    let active = ui.is_item_active();
    let draw = ui.get_window_draw_list();
    draw.add_rect(
        origin,
        [origin[0] + width, origin[1] + height],
        [0.10, 0.10, 0.12, 1.],
    )
    .filled(true)
    .rounding(3.)
    .build();
    // Ticks every tenth of the length, labels at the ends.
    for i in 0..=10 {
        let x = origin[0] + 7. + (width - 14.) * i as f32 / 10.;
        let tall = if i % 5 == 0 { 8. } else { 4. };
        draw.add_line([x, origin[1]], [x, origin[1] + tall], [1., 1., 1., 0.25])
            .build();
    }
    // Keys at either end stay whole inside the strip.
    let inset = 7.;
    let x_of = |t: f32| origin[0] + inset + (t / length).clamp(0., 1.) * (width - 2. * inset);
    let middle = origin[1] + height * 0.55;
    let selected_key = e.world2d.skeleton.key;
    for (k, key) in s.animations[index].keys.iter().enumerate() {
        let x = x_of(key.time);
        let r = if selected_key == Some(k) { 6. } else { 5. };
        let points = [
            [x, middle - r],
            [x + r, middle],
            [x, middle + r],
            [x - r, middle],
        ];
        draw.add_polyline(points.to_vec(), KEY_COLOUR)
            .filled(true)
            .build();
        if selected_key == Some(k) {
            draw.add_polyline(points.to_vec(), [1., 1., 1., 1.])
                .thickness(1.5)
                .build();
        }
    }
    let now = e.world2d.skeleton.time.clamp(0., length);
    let cx = x_of(now);
    draw.add_line(
        [cx, origin[1]],
        [cx, origin[1] + height],
        [0.95, 0.35, 0.35, 1.],
    )
    .thickness(2.)
    .build();
    drop(draw);
    // Click a diamond to select its key; click or drag elsewhere to scrub.
    if (active || (hovered && ui.is_mouse_clicked(imgui::MouseButton::Left))) && !previewing(e) {
        let mouse = ui.io().mouse_pos;
        let picked = s.animations[index]
            .keys
            .iter()
            .position(|k| (x_of(k.time) - mouse[0]).abs() <= 6.);
        let time = if ui.is_mouse_clicked(imgui::MouseButton::Left) && picked.is_some() {
            e.world2d.skeleton.key = picked;
            s.animations[index].keys[picked.unwrap_or(0)].time
        } else {
            if ui.is_mouse_clicked(imgui::MouseButton::Left) {
                e.world2d.skeleton.key = None;
            }
            ((mouse[0] - origin[0] - inset) / (width - 2. * inset)).clamp(0., 1.) * length
        };
        if (time - e.world2d.skeleton.time).abs() > 1e-5
            || ui.is_mouse_clicked(imgui::MouseButton::Left)
        {
            e.world2d.skeleton.time = time;
            if let Some(pose) = s.sample(index, time) {
                set_pose(e, s, &pose);
                e.changed_coalesced("skeleton2d-scrub");
            }
        }
    }
    if hovered {
        ui.tooltip_text(
            "Click a key to go to it; click or drag to scrub. The bones take the pose.",
        );
    }
    // Buttons.
    let playing = previewing(e);
    if ui.button(if playing { "Stop" } else { "Play" }) {
        if playing {
            stop_preview(e);
        } else if s.animations[index].keys.is_empty() {
            // Nothing to play yet.
        } else {
            start_preview(e, actor, component, s);
        }
    }
    ui.same_line();
    if ui.button("Key pose") && !playing {
        let pose = current_pose(e, s);
        let time = e.world2d.skeleton.time.clamp(0., length);
        if let Some(k) = s.key_pose(index, time, pose) {
            e.world2d.skeleton.key = Some(k);
        }
    }
    if ui.is_item_hovered() {
        ui.tooltip_text("Records every bone's current rotation and position at this time");
    }
    if let Some(k) = e
        .world2d
        .skeleton
        .key
        .filter(|k| *k < s.animations[index].keys.len())
    {
        ui.same_line();
        if ui.button("Delete key") && !playing {
            s.animations[index].keys.remove(k);
            e.world2d.skeleton.key = None;
        }
    }
    ui.same_line();
    ui.text_disabled(format!(
        "{:.2} / {:.2} s  ·  {} keys",
        now,
        length,
        s.animations[index].keys.len()
    ));
    if let Some(k) = e
        .world2d
        .skeleton
        .key
        .filter(|k| *k < s.animations[index].keys.len())
    {
        let mut time = s.animations[index].keys[k].time;
        if crate::gui::Drag::new(crate::gui::field(ui, "Key time (s)"))
            .speed(0.01)
            .range(0., length)
            .display_format("%.2f")
            .build(ui, &mut time)
        {
            s.animations[index].keys[k].time = time;
            e.world2d.skeleton.time = time;
            s.normalize();
            e.world2d.skeleton.key = s.animations[index]
                .keys
                .iter()
                .position(|key| (key.time - time).abs() < 1e-4);
        }
    }
}

fn ik_section(ui: &Ui, e: &mut Editor, actor: Uuid, _component: Uuid, s: &mut Skeleton2D) {
    ui.text(format!("IK chains ({}/{})", s.ik.len(), skeleton2d::MAX_IK));
    let names: Vec<String> = s.bones.iter().map(|b| name_of(e, b.actor)).collect();
    let bone_combo = |ui: &Ui, label: &str, value: &mut usize| {
        if names.is_empty() {
            return;
        }
        let shown = names.get(*value).cloned().unwrap_or_default();
        if let Some(_c) = ui.begin_combo(crate::gui::field(ui, label), shown) {
            for (i, n) in names.iter().enumerate() {
                if ui
                    .selectable_config(format!("{n}##{i}"))
                    .selected(i == *value)
                    .build()
                {
                    *value = i;
                }
            }
        }
    };
    let bones: Vec<Option<Uuid>> = s.bones.iter().map(|b| b.actor).collect();
    let mut remove = None;
    let mut reach = None;
    for (n, chain) in s.ik.iter_mut().enumerate() {
        let _id = ui.push_id_usize(n);
        ui.separator();
        crate::gui::toggle(ui, &format!("Chain {}", n + 1), &mut chain.enabled);
        ui.same_line();
        if ui.small_button("x") {
            remove = Some(n);
        }
        bone_combo(ui, "Upper bone", &mut chain.upper);
        bone_combo(ui, "Lower bone", &mut chain.lower);
        let lower = bones.get(chain.lower).copied().flatten();
        let below = |a: &crate::scene::Actor| {
            lower.is_some() && a.attach.as_ref().map(|at| at.actor) == lower
        };
        crate::world2d_shapes_editor::actor_combo(
            ui,
            e,
            "Tip",
            "None",
            actor,
            &|a| below(a) || world2d::component(a, world2d::SPRITE2D).is_some(),
            &mut chain.tip,
        );
        crate::world2d_shapes_editor::actor_combo(
            ui,
            e,
            "Target",
            "None",
            actor,
            &|a| !bones.contains(&Some(a.id)),
            &mut chain.target,
        );
        let mut side = usize::from(!chain.bend_positive);
        if crate::world2d_editor::segmented(ui, "Elbow side", &["Right", "Left"], &mut side) {
            chain.bend_positive = side == 0;
        }
        if chain.tip.is_some() && chain.target.is_some() {
            if ui.small_button("Reach target") {
                reach = Some(n);
            }
            if ui.is_item_hovered() {
                ui.tooltip_text(
                    "Poses the two bones so the tip touches the target, here in the editor",
                );
            }
        } else {
            crate::gui::muted(
                ui,
                "Pick the tip (the part at the end of the lower bone) and a target actor.",
            );
        }
    }
    if let Some(n) = remove {
        s.ik.remove(n);
    }
    if s.ik.len() < skeleton2d::MAX_IK && s.bones.len() >= 2 && ui.button("Add IK chain") {
        s.ik.push(suggest_chain(e, s));
    }
    if !s.ik.is_empty() {
        crate::gui::muted(
            ui,
            "At runtime the chain turns the upper and lower bones after the animation so the tip reaches the target; move the target from gameplay (or set_ik_target) to aim arms and plant feet.",
        );
    }
    if let Some(n) = reach {
        reach_target(e, s, n);
    }
}
/// A chain over the first bone pair (parent, child) whose child has a part attached.
fn suggest_chain(e: &Editor, s: &Skeleton2D) -> Ik {
    for (lower, bone) in s.bones.iter().enumerate() {
        let Some(id) = bone.actor else { continue };
        let Some(parent) = parent_of(e, id) else {
            continue;
        };
        let Some(upper) = s.bones.iter().position(|b| b.actor == Some(parent)) else {
            continue;
        };
        if s.ik.iter().any(|c| c.lower == lower) {
            continue;
        }
        let tip = e
            .scene
            .actors
            .iter()
            .find(|a| a.attach.as_ref().is_some_and(|at| at.actor == id))
            .map(|a| a.id);
        if tip.is_some() {
            return Ik {
                upper,
                lower,
                tip,
                ..Default::default()
            };
        }
    }
    Ik {
        upper: 0,
        lower: 1.min(s.bones.len().saturating_sub(1)),
        ..Default::default()
    }
}
fn world_point(e: &Editor, actor: Option<Uuid>) -> Option<[f32; 2]> {
    let i = e.scene.actor_index(actor?)?;
    Some(crate::world2d_editor::world_of_index(&e.scene, i).1.t)
}
fn reach_target(e: &mut Editor, s: &Skeleton2D, chain: usize) {
    let Some(c) = s.ik.get(chain) else { return };
    let (upper, lower) = (
        s.bones.get(c.upper).and_then(|b| b.actor),
        s.bones.get(c.lower).and_then(|b| b.actor),
    );
    let (Some(u), Some(l), Some(t), Some(g)) = (
        world_point(e, upper),
        world_point(e, lower),
        world_point(e, c.tip),
        world_point(e, c.target),
    ) else {
        return;
    };
    let Some((first, second)) = skeleton2d::ik_turns(u, l, t, g, c.bend_positive) else {
        return;
    };
    stop_preview(e);
    if let Some(id) = upper {
        crate::world2d_editor::edit_transform(e, id, |tr| {
            tr.rotation = skeleton2d::wrap_degrees(tr.rotation + first)
        });
    }
    if let Some(id) = lower {
        crate::world2d_editor::edit_transform(e, id, |tr| {
            tr.rotation = skeleton2d::wrap_degrees(tr.rotation + second)
        });
    }
    e.changed();
}

// ---- 2D view -----------------------------------------------------------------------------

/// Bones as tapered shapes from each joint to its child bones' joints, the IK chains and
/// their targets, for the skeleton of the selected actor (or of the selected bone).
pub fn overlay(
    e: &Editor,
    draw: &imgui::DrawListMut,
    views: &[View],
    to_screen: &dyn Fn([f32; 2]) -> [f32; 2],
) {
    let Some(selected) = e.selected_actor else {
        return;
    };
    let Some((owner, component)) = skeleton_of(e, selected) else {
        return;
    };
    let Some(s) = read(e, owner, component) else {
        return;
    };
    let world = |id: Option<Uuid>| -> Option<Affine> {
        let id = id?;
        views.iter().find(|v| v.id == id).map(|v| v.world)
    };
    for bone in &s.bones {
        let Some(w) = world(bone.actor) else { continue };
        let from = to_screen(w.t);
        let lit = bone.actor == Some(selected);
        let colour = if lit { BONE_SELECTED } else { BONE };
        let children: Vec<[f32; 2]> = s
            .bones
            .iter()
            .filter(|b| b.actor.is_some_and(|id| parent_of(e, id) == bone.actor))
            .filter_map(|b| world(b.actor).map(|w| to_screen(w.t)))
            .collect();
        for to in &children {
            let d = [to[0] - from[0], to[1] - from[1]];
            let length = d[0].hypot(d[1]);
            if length < 2. {
                continue;
            }
            let u = [d[0] / length, d[1] / length];
            let n = [-u[1], u[0]];
            let half = (length * 0.12).clamp(2.5, 9.);
            let shoulder = [from[0] + u[0] * length * 0.2, from[1] + u[1] * length * 0.2];
            let points = vec![
                from,
                [shoulder[0] + n[0] * half, shoulder[1] + n[1] * half],
                *to,
                [shoulder[0] - n[0] * half, shoulder[1] - n[1] * half],
            ];
            draw.add_polyline(
                points.clone(),
                [
                    colour[0],
                    colour[1],
                    colour[2],
                    if lit { 0.55 } else { 0.32 },
                ],
            )
            .filled(true)
            .build();
            draw.add_polyline(points, colour).thickness(1.2).build();
        }
        draw.add_circle(from, if lit { 5. } else { 4. }, [0.12, 0.12, 0.14, 1.])
            .filled(true)
            .build();
        draw.add_circle(from, if lit { 5. } else { 4. }, colour)
            .thickness(1.5)
            .build();
    }
    for chain in s.ik.iter().filter(|c| c.enabled) {
        let (Some(tip), Some(target)) = (world(chain.tip), world(chain.target)) else {
            continue;
        };
        let (a, b) = (to_screen(tip.t), to_screen(target.t));
        let d = [b[0] - a[0], b[1] - a[1]];
        let length = d[0].hypot(d[1]);
        let dashes = (length / 6.).floor() as usize;
        for i in (0..dashes).step_by(2) {
            let (t0, t1) = (
                i as f32 / dashes as f32,
                ((i + 1) as f32 / dashes as f32).min(1.),
            );
            draw.add_line(
                [a[0] + d[0] * t0, a[1] + d[1] * t0],
                [a[0] + d[0] * t1, a[1] + d[1] * t1],
                IK_COLOUR,
            )
            .thickness(1.5)
            .build();
        }
        draw.add_circle(b, 7., IK_COLOUR).thickness(1.5).build();
        draw.add_line([b[0] - 10., b[1]], [b[0] + 10., b[1]], IK_COLOUR)
            .build();
        draw.add_line([b[0], b[1] - 10.], [b[0], b[1] + 10.], IK_COLOUR)
            .build();
    }
}
