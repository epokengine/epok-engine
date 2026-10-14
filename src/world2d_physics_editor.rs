//! RigidBody2D and Joint2D editing (World2D initiative, phase 4): their Inspector bands,
//! with the checks that explain why a body would not collide, and the joints drawn in
//! the 2D view (a solid rod, a dashed rope, a zigzag spring).
use crate::{
    editor::Editor,
    physics2d::{self, Joint2D, JointKind, RigidBody2D},
    world2d,
};
use imgui::Ui;
use uuid::Uuid;

pub const ICON_RIGID: &str = "\u{f1b2}";
pub const ICON_JOINT: &str = "\u{f0c1}";
const JOINT_COLOUR: [f32; 4] = [0.95, 0.80, 0.45, 1.];
const WARN: [f32; 4] = [1., 0.75, 0.35, 1.];

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
fn drag(ui: &Ui, label: &str, value: &mut f32, speed: f32, range: (f32, f32), format: &str) {
    crate::gui::Drag::new(crate::gui::field(ui, label))
        .speed(speed)
        .range(range.0, range.1)
        .display_format(format)
        .build(ui, value);
}
fn has(e: &Editor, actor: Uuid, class: &str) -> bool {
    e.scene
        .actor_index(actor)
        .is_some_and(|i| world2d::component(&e.scene.actors[i], class).is_some())
}

pub fn band(
    ui: &Ui,
    e: &mut Editor,
    actor: Uuid,
    component: &crate::actor_document::ComponentInstance,
) -> bool {
    let id = component.class.class_id.as_deref().unwrap_or("");
    let (glyph, title) = match id {
        world2d::RIGID_BODY2D => (ICON_RIGID, "RigidBody2D"),
        world2d::JOINT2D => (ICON_JOINT, "Joint2D"),
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
        if id == world2d::RIGID_BODY2D {
            record::<RigidBody2D>(e, actor, component, |_, r| *r = Default::default());
        } else {
            record::<Joint2D>(e, actor, component, |_, j| *j = Default::default());
        }
        return true;
    }
    if open {
        if id == world2d::RIGID_BODY2D {
            rigid_band(ui, e, actor, component);
        } else {
            joint_band(ui, e, actor, component);
        }
    }
    true
}
fn rigid_band(
    ui: &Ui,
    e: &mut Editor,
    actor: Uuid,
    component: &crate::actor_document::ComponentInstance,
) {
    let index = e.scene.actor_index(actor);
    let circle = index
        .and_then(|i| world2d::component(&e.scene.actors[i], world2d::COLLIDER2D))
        .map(world2d::read::<world2d::Collider2D>)
        .filter(|c| !c.trigger && c.enabled);
    record::<RigidBody2D>(e, actor, component, |_, r| {
        drag(ui, "Mass", &mut r.mass, 0.01, (0.01, 1000.), "%.2f");
        drag(ui, "Bounce", &mut r.bounce, 0.01, (0., 1.), "%.2f");
        drag(ui, "Friction", &mut r.friction, 0.01, (0., 1.), "%.2f");
        drag(
            ui,
            "Damping",
            &mut r.linear_damping,
            0.01,
            (0., 60.),
            "%.2f",
        );
        crate::gui::Drag::new(crate::gui::field(ui, "Gravity (px/s²)"))
            .speed(1.)
            .range(-8192., 8192.)
            .display_format("%.0f")
            .build_array(ui, &mut r.gravity);
        crate::gui::Drag::new(crate::gui::field(ui, "Start velocity"))
            .speed(1.)
            .range(-8192., 8192.)
            .display_format("%.0f")
            .build_array(ui, &mut r.velocity);
        crate::gui::toggle(ui, "Pushable", &mut r.pushable);
        if ui.is_item_hovered() {
            ui.tooltip_text("Characters walking into it push it; heavier bodies move slower");
        }
        crate::gui::toggle(ui, "Frozen", &mut r.frozen);
        if ui.is_item_hovered() {
            ui.tooltip_text(
                "Stays put and blocks like a static body until gameplay calls set_frozen(false)",
            );
        }
        crate::gui::toggle(ui, "Can sleep", &mut r.can_sleep);
        if circle
            .as_ref()
            .is_some_and(|c| c.shape == world2d::Shape2D::Circle)
        {
            crate::gui::toggle(ui, "Rolls", &mut r.rolls);
        }
    });
    match circle {
        None => ui.text_colored(
            WARN,
            "No solid Collider2D: the body falls through everything. Add one (box or circle).",
        ),
        Some(c) if c.shape == world2d::Shape2D::Box => {
            crate::gui::muted(ui, "Boxes slide and stack without tumbling; circles roll.")
        }
        Some(_) => {}
    }
    if let Some(i) = index
        && e.scene.actors[i].attach.is_some()
    {
        ui.text_colored(
            WARN,
            "Attached to another node: the body still moves in the world, and moving the parent moves it too.",
        );
    }
}
fn joint_band(
    ui: &Ui,
    e: &mut Editor,
    actor: Uuid,
    component: &crate::actor_document::ComponentInstance,
) {
    let index = e.scene.actor_index(actor);
    record::<Joint2D>(e, actor, component, |e, j| {
        let mut kind = j.kind as usize;
        if crate::world2d_editor::segmented(ui, "Kind", &["Rod", "Rope", "Spring"], &mut kind) {
            j.kind = [JointKind::Rod, JointKind::Rope, JointKind::Spring][kind];
        }
        let any = |_: &crate::scene::Actor| true;
        crate::world2d_shapes_editor::actor_combo(
            ui,
            e,
            "Body A",
            "None: this node is the fixed end",
            actor,
            &any,
            &mut j.body_a,
        );
        let rigid =
            |a: &crate::scene::Actor| world2d::component(a, world2d::RIGID_BODY2D).is_some();
        crate::world2d_shapes_editor::actor_combo(
            ui,
            e,
            "Body B",
            "None",
            actor,
            &rigid,
            &mut j.body_b,
        );
        drag(ui, "Length (px)", &mut j.length, 0.25, (0., 8192.), "%.1f");
        if j.length <= 0. {
            let current = index.map_or(0., |i| physics2d::cooked_length(&e.scene, i, j));
            ui.text_disabled(format!("0 keeps the distance in the map: {current:.1} px"));
        }
        if j.kind == JointKind::Spring {
            drag(
                ui,
                "Stiffness",
                &mut j.stiffness,
                0.5,
                (0., 100000.),
                "%.1f",
            );
            drag(ui, "Damping", &mut j.damping, 0.05, (0., 10000.), "%.2f");
        }
        crate::gui::toggle(ui, "Enabled", &mut j.enabled);
    });
    let joint: Joint2D = world2d::read(component);
    match joint.body_b {
        None => ui.text_colored(WARN, "Pick Body B: the rigid body the joint holds."),
        Some(b) if !has(e, b, world2d::RIGID_BODY2D) => {
            ui.text_colored(WARN, "Body B has no RigidBody2D.")
        }
        Some(_) => crate::gui::muted(
            ui,
            "A rod holds the distance, a rope at most the distance, a spring pulls back towards it. Chains are bodies joined one to the next; gameplay can set_length or break_joint.",
        ),
    }
}

// ---- 2D view -----------------------------------------------------------------------------

/// Every joint of the map: anchors and the link between them.
pub fn overlay(e: &Editor, draw: &imgui::DrawListMut, to_screen: &dyn Fn([f32; 2]) -> [f32; 2]) {
    for (index, actor) in e.scene.actors.iter().enumerate() {
        for component in actor
            .components
            .iter()
            .filter(|c| c.class.class_id.as_deref() == Some(world2d::JOINT2D))
        {
            let joint: Joint2D = world2d::read(component);
            let Some((a, b)) = physics2d::joint_ends(&e.scene, index, &joint) else {
                continue;
            };
            let selected = e.selected_actor.is_some_and(|s| {
                s == actor.id || Some(s) == joint.body_a || Some(s) == joint.body_b
            });
            let colour = [
                JOINT_COLOUR[0],
                JOINT_COLOUR[1],
                JOINT_COLOUR[2],
                if selected { 1. } else { 0.55 },
            ];
            let (sa, sb) = (to_screen(a), to_screen(b));
            let d = [sb[0] - sa[0], sb[1] - sa[1]];
            let length = d[0].hypot(d[1]).max(1e-3);
            match joint.kind {
                JointKind::Rod => {
                    draw.add_line(sa, sb, colour).thickness(2.).build();
                }
                JointKind::Rope => {
                    let dashes = (length / 5.).floor().max(1.) as usize;
                    for i in (0..dashes).step_by(2) {
                        let (t0, t1) = (
                            i as f32 / dashes as f32,
                            ((i + 1) as f32 / dashes as f32).min(1.),
                        );
                        draw.add_line(
                            [sa[0] + d[0] * t0, sa[1] + d[1] * t0],
                            [sa[0] + d[0] * t1, sa[1] + d[1] * t1],
                            colour,
                        )
                        .thickness(1.5)
                        .build();
                    }
                }
                JointKind::Spring => {
                    let n = [-d[1] / length * 4., d[0] / length * 4.];
                    let turns = 10;
                    let mut points = vec![sa];
                    for i in 1..turns {
                        let t = i as f32 / turns as f32;
                        let side = if i % 2 == 0 { 1. } else { -1. };
                        points.push([
                            sa[0] + d[0] * t + n[0] * side,
                            sa[1] + d[1] * t + n[1] * side,
                        ]);
                    }
                    points.push(sb);
                    draw.add_polyline(points, colour).thickness(1.5).build();
                }
            }
            let fixed = joint
                .body_a
                .is_none_or(|a| !has(e, a, world2d::RIGID_BODY2D));
            if fixed {
                draw.add_rect([sa[0] - 4., sa[1] - 4.], [sa[0] + 4., sa[1] + 4.], colour)
                    .filled(true)
                    .build();
            } else {
                draw.add_circle(sa, 3.5, colour).filled(true).build();
            }
            draw.add_circle(sb, 3.5, colour).filled(true).build();
        }
    }
}
