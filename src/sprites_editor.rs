//! Inspector editing for reusable sprite clips and bounded particle emitters.
use crate::{
    editor::Editor,
    scene::Actor,
    sprites::{Orientation, Sprite},
    texture::BlendMode,
};
pub fn sprite(ui: &imgui::Ui, index: &crate::assets::Index, s: &mut Sprite) {
    ui.checkbox("Enabled", &mut s.enabled);
    let label = s
        .texture
        .and_then(|id| index.resolve(id).ok())
        .map(|r| r.meta.source.clone())
        .unwrap_or_else(|| "None / missing".into());
    if let Some(_combo) = ui.begin_combo("Texture", label) {
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
    crate::gui::Drag::new("Size")
        .range(0.001, 128.)
        .speed(0.01)
        .build_array(ui, &mut s.size);
    crate::gui::Drag::new("Pivot")
        .range(0., 1.)
        .speed(0.01)
        .build_array(ui, &mut s.pivot);
    crate::gui::Drag::new("Atlas x/y/w/h")
        .range(0, 256)
        .build_array(ui, &mut s.region);
    ui.text_disabled("Zero width/height uses the whole texture.");
    ui.checkbox("Flip X", &mut s.flip_x);
    ui.same_line();
    ui.checkbox("Flip Y", &mut s.flip_y);
    let mut orientation = match s.orientation {
        Orientation::Fixed => 0,
        Orientation::Upright => 1,
        Orientation::Spherical => 2,
    };
    if ui.combo_simple_string(
        "Orientation",
        &mut orientation,
        &["Fixed plane", "Upright billboard", "Spherical billboard"],
    ) {
        s.orientation = [
            Orientation::Fixed,
            Orientation::Upright,
            Orientation::Spherical,
        ][orientation];
    }
    ui.color_edit3("Tint", &mut s.color);
    ui.checkbox("Unlit", &mut s.unlit);
    let modes = [
        BlendMode::Cutout,
        BlendMode::Average,
        BlendMode::Add,
        BlendMode::Subtract,
        BlendMode::AddQuarter,
    ];
    let mut mode = modes.iter().position(|m| *m == s.blend).unwrap_or(0);
    if ui.combo_simple_string(
        "Blend",
        &mut mode,
        &["Cutout", "Average", "Add", "Subtract", "Add quarter"],
    ) {
        s.blend = modes[mode];
    }
    crate::gui::Drag::new("Depth bias")
        .range(-511, 511)
        .build(ui, &mut s.depth_bias);
    ui.text_disabled("Positive bias draws farther away (1 step = 0.25 world units).");
}
pub fn inspector(ui: &imgui::Ui, editor: &Editor, e: &mut Actor) {
    if let Some(s) = &mut e.sprite
        && crate::gui::heading(ui, "Sprite")
    {
        let _id = ui.push_id("sprite");
        sprite(ui, &editor.assets.index, s);
        if ui.small_button("Remove Sprite") {
            e.sprite = None;
            e.sprite_animator = None;
        }
    }
    if let Some(a) = &mut e.data.sprite_animator
        && crate::gui::heading(ui, "Sprite Animator")
    {
        let _id = ui.push_id("sprite animator");
        ui.checkbox("Play on start", &mut a.playing);
        let names = a
            .sheet
            .clips
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>();
        ui.combo_simple_string("Clip", &mut a.clip, &names);
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
            ui.input_text("Name", &mut c.name).build();
            ui.checkbox("Loop", &mut c.looping);
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
                    crate::gui::Drag::new("Rectangle")
                        .range(0, 256)
                        .build_array(ui, &mut f.region);
                    crate::gui::Drag::new("Seconds")
                        .range(1. / 60., 60.)
                        .speed(0.01)
                        .build(ui, &mut f.duration);
                    crate::gui::Drag::new("Event ID (0 = none)")
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
        if ui.small_button("Remove Sprite Animator") {
            e.sprite_animator = None;
        }
    }
    if let Some(p) = &mut e.particle_emitter
        && crate::gui::heading(ui, "Particle Emitter")
    {
        emitter(ui, &editor.assets.index, p);
        if ui.small_button("Remove Particle Emitter") {
            e.particle_emitter = None;
        }
    }
}
pub fn emitter(ui: &imgui::Ui, index: &crate::assets::Index, p: &mut crate::particles::Emitter) {
    let _id = ui.push_id("particles");
    ui.checkbox("Enabled", &mut p.enabled);
    ui.checkbox("Play on start", &mut p.play_on_start);
    ui.checkbox("Continuous", &mut p.continuous);
    ui.checkbox("Local space", &mut p.local_space);
    crate::gui::Drag::new("Particles / second")
        .range(0., 512.)
        .speed(0.1)
        .build(ui, &mut p.rate);
    crate::gui::Drag::new("Burst count")
        .range(0, 128)
        .build(ui, &mut p.burst);
    crate::gui::Drag::new("Per-emitter limit")
        .range(1, 128)
        .build(ui, &mut p.max_particles);
    crate::gui::Drag::new("Lifetime seconds")
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
    crate::gui::Drag::new("Start size")
        .range(0., 32.)
        .speed(0.01)
        .build(ui, &mut p.start_size);
    crate::gui::Drag::new("End size")
        .range(0., 32.)
        .speed(0.01)
        .build(ui, &mut p.end_size);
    ui.color_edit3("Start color", &mut p.start_color);
    ui.color_edit3("End color", &mut p.end_color);
    crate::gui::Drag::new("Seed").build(ui, &mut p.seed);
    crate::gui::Drag::new("Flipbook frames")
        .range(1, 256)
        .build(ui, &mut p.frames);
    crate::gui::Drag::new("Flipbook columns")
        .range(1, 256)
        .build(ui, &mut p.frame_columns);
    crate::gui::Drag::new("Seconds per cell")
        .range(1. / 60., 60.)
        .speed(0.01)
        .build(ui, &mut p.frame_duration);
    if ui.collapsing_header("Particle sprite", imgui::TreeNodeFlags::empty()) {
        let _id = ui.push_id("particle sprite");
        sprite(ui, index, &mut p.sprite);
    }
    ui.text_wrapped("256 particles globally; 128 per emitter. Full pools drop new particles. Preview repeats every eight seconds.");
}
