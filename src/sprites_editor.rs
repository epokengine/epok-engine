//! Inspector editing for reusable sprite clips and bounded particle emitters.
use crate::{
    editor::Editor,
    scene::Actor,
    sprites::{Orientation, Sprite},
    texture::BlendMode,
};
pub fn sprite(ui: &imgui::Ui, index: &crate::assets::Index, s: &mut Sprite) {
    crate::gui::toggle(ui, "Enabled", &mut s.enabled);
    let label = s
        .texture
        .and_then(|id| index.resolve(id).ok())
        .map(|r| r.meta.source.clone())
        .unwrap_or_else(|| "None / missing".into());
    if let Some(_combo) = ui.begin_combo(crate::gui::field(ui, "Texture"), label) {
        if ui.selectable("None") {
            s.texture = None;
        }
        for r in index
            .usable()
            .filter(|r| r.meta.kind == crate::assets::Kind::Texture)
        {
            if ui.selectable(&r.meta.source) {
                s.texture = Some(r.meta.id);
            }
        }
    }
    crate::gui::Drag::new(crate::gui::field(ui, "Size"))
        .range(0.001, 128.)
        .speed(0.01)
        .build_array(ui, &mut s.size);
    crate::gui::Drag::new(crate::gui::field(ui, "Pivot"))
        .range(0., 1.)
        .speed(0.01)
        .build_array(ui, &mut s.pivot);
    crate::gui::Drag::new(crate::gui::field(ui, "Atlas x/y/w/h"))
        .range(0, 256)
        .build_array(ui, &mut s.region);
    ui.text_disabled("Zero width/height uses the whole texture.");
    crate::gui::toggle(ui, "Flip X", &mut s.flip_x);
    crate::gui::toggle(ui, "Flip Y", &mut s.flip_y);
    let mut orientation = match s.orientation {
        Orientation::Fixed => 0,
        Orientation::Upright => 1,
        Orientation::Spherical => 2,
    };
    if ui.combo_simple_string(
        crate::gui::field(ui, "Orientation"),
        &mut orientation,
        &["Fixed plane", "Upright billboard", "Spherical billboard"],
    ) {
        s.orientation = [
            Orientation::Fixed,
            Orientation::Upright,
            Orientation::Spherical,
        ][orientation];
    }
    ui.color_edit3(crate::gui::field(ui, "Tint"), &mut s.color);
    crate::gui::toggle(ui, "Unlit", &mut s.unlit);
    let modes = [
        BlendMode::Cutout,
        BlendMode::Average,
        BlendMode::Add,
        BlendMode::Subtract,
        BlendMode::AddQuarter,
    ];
    let mut mode = modes.iter().position(|m| *m == s.blend).unwrap_or(0);
    if ui.combo_simple_string(
        crate::gui::field(ui, "Blend"),
        &mut mode,
        &["Cutout", "Average", "Add", "Subtract", "Add quarter"],
    ) {
        s.blend = modes[mode];
    }
    crate::gui::Drag::new(crate::gui::field(ui, "Depth bias"))
        .range(-511, 511)
        .build(ui, &mut s.depth_bias);
    ui.text_disabled("Positive bias draws farther away (1 step = 0.25 world units).");
}
pub fn inspector(ui: &imgui::Ui, editor: &Editor, e: &mut Actor) {
    if e.sprite.is_some() {
        let mut remove = false;
        let open = crate::gui::section(ui, "Sprite", || {
            remove = ui.menu_item("Remove Sprite");
        });
        if open && let Some(s) = &mut e.sprite {
            let _id = ui.push_id("sprite");
            sprite(ui, &editor.assets.index, s);
        }
        if remove {
            e.sprite = None;
            e.sprite_animator = None;
        }
    }
    let mut remove_animator = false;
    let animator = e.data.sprite_animator.is_some()
        && crate::gui::section(ui, "Sprite Animator", || {
            remove_animator = ui.menu_item("Remove Sprite Animator");
        });
    if animator && let Some(a) = &mut e.data.sprite_animator {
        let _id = ui.push_id("sprite animator");
        crate::gui::toggle(ui, "Play on start", &mut a.playing);
        let names = a
            .sheet
            .clips
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>();
        ui.combo_simple_string(crate::gui::field(ui, "Clip"), &mut a.clip, &names);
        if ui.small_button("Add clip") && a.sheet.clips.len() < 32 {
            a.sheet.clips.push(crate::sprites::Clip {
                name: format!("Clip {}", a.sheet.clips.len() + 1),
                ..Default::default()
            });
            a.clip = a.sheet.clips.len() - 1;
        }
        if a.sheet.clips.len() > 1 {
            ui.same_line();
            if ui.small_button("Delete clip") {
                a.sheet.clips.remove(a.clip);
                a.clip = a.clip.min(a.sheet.clips.len() - 1);
            }
        }
        if let Some(c) = a.sheet.clips.get_mut(a.clip) {
            ui.input_text(crate::gui::field(ui, "Name"), &mut c.name)
                .build();
            crate::gui::toggle(ui, "Loop", &mut c.looping);
            if ui.small_button("Append frame") && c.frames.len() < 256 {
                c.frames.push(c.frames.last().cloned().unwrap_or_default());
            }
            ui.same_line();
            if ui.small_button("Slice horizontal row") {
                let base = e
                    .data
                    .sprite
                    .as_ref()
                    .map(|s| s.region)
                    .unwrap_or([0, 0, 16, 16]);
                if base[2] > 0 && base[3] > 0 {
                    let count = ((256 - base[0]) / base[2]).min(256);
                    c.frames = (0..count)
                        .map(|i| crate::sprites::Frame {
                            region: [base[0] + i * base[2], base[1], base[2], base[3]],
                            ..Default::default()
                        })
                        .collect();
                }
            }
            let mut remove = None;
            for (i, f) in c.frames.iter_mut().enumerate() {
                let _id = ui.push_id(i.to_string());
                if ui.collapsing_header(format!("Frame {}", i + 1), imgui::TreeNodeFlags::empty()) {
                    crate::gui::Drag::new(crate::gui::field(ui, "Rectangle"))
                        .range(0, 256)
                        .build_array(ui, &mut f.region);
                    crate::gui::Drag::new(crate::gui::field(ui, "Seconds"))
                        .range(1. / 60., 60.)
                        .speed(0.01)
                        .build(ui, &mut f.duration);
                    crate::gui::Drag::new(crate::gui::field(ui, "Event ID (0 = none)"))
                        .range(0, 65535)
                        .build(ui, &mut f.event);
                    if ui.small_button("Delete frame") {
                        remove = Some(i);
                    }
                }
            }
            if c.frames.len() > 1
                && let Some(i) = remove
            {
                c.frames.remove(i);
            }
        }
    }
    if remove_animator {
        e.sprite_animator = None;
    }
    if e.particle_emitter.is_some() {
        let mut remove = false;
        let open = crate::gui::section(ui, "Particle Emitter", || {
            remove = ui.menu_item("Remove Particle Emitter");
        });
        if open && let Some(p) = &mut e.particle_emitter {
            emitter(ui, &editor.assets.index, p);
        }
        if remove {
            e.particle_emitter = None;
        }
    }
}
pub fn emitter(ui: &imgui::Ui, index: &crate::assets::Index, p: &mut crate::particles::Emitter) {
    let _id = ui.push_id("particles");
    crate::gui::toggle(ui, "Enabled", &mut p.enabled);
    crate::gui::toggle(ui, "Play on start", &mut p.play_on_start);
    crate::gui::toggle(ui, "Continuous", &mut p.continuous);
    crate::gui::toggle(ui, "Local space", &mut p.local_space);
    crate::gui::Drag::new(crate::gui::field(ui, "Particles / second"))
        .range(0., 512.)
        .speed(0.1)
        .build(ui, &mut p.rate);
    crate::gui::Drag::new(crate::gui::field(ui, "Burst count"))
        .range(0, 128)
        .build(ui, &mut p.burst);
    crate::gui::Drag::new(crate::gui::field(ui, "Per-emitter limit"))
        .range(1, 128)
        .build(ui, &mut p.max_particles);
    crate::gui::Drag::new(crate::gui::field(ui, "Lifetime seconds"))
        .range(1. / 60., 60.)
        .speed(0.01)
        .build(ui, &mut p.lifetime);
    for (label, values) in [
        ("Velocity", &mut p.velocity),
        ("Velocity spread", &mut p.spread),
        ("Gravity", &mut p.gravity),
    ] {
        crate::gui::Drag::new(label)
            .range(
                if label == "Velocity spread" {
                    0.
                } else {
                    -128.
                },
                128.,
            )
            .speed(0.01)
            .build_array(ui, values);
    }
    crate::gui::Drag::new(crate::gui::field(ui, "Start size"))
        .range(0., 32.)
        .speed(0.01)
        .build(ui, &mut p.start_size);
    crate::gui::Drag::new(crate::gui::field(ui, "End size"))
        .range(0., 32.)
        .speed(0.01)
        .build(ui, &mut p.end_size);
    ui.color_edit3(crate::gui::field(ui, "Start color"), &mut p.start_color);
    ui.color_edit3(crate::gui::field(ui, "End color"), &mut p.end_color);
    crate::gui::Drag::new(crate::gui::field(ui, "Seed")).build(ui, &mut p.seed);
    crate::gui::Drag::new(crate::gui::field(ui, "Flipbook frames"))
        .range(1, 256)
        .build(ui, &mut p.frames);
    crate::gui::Drag::new(crate::gui::field(ui, "Flipbook columns"))
        .range(1, 256)
        .build(ui, &mut p.frame_columns);
    crate::gui::Drag::new(crate::gui::field(ui, "Seconds per cell"))
        .range(1. / 60., 60.)
        .speed(0.01)
        .build(ui, &mut p.frame_duration);
    if ui.collapsing_header("Particle sprite", imgui::TreeNodeFlags::empty()) {
        let _id = ui.push_id("particle sprite");
        sprite(ui, index, &mut p.sprite);
    }
    ui.text_wrapped("256 particles globally; 128 per emitter. Full pools drop new particles. Preview repeats every eight seconds.");
}
