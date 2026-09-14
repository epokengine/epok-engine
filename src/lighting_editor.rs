use crate::{editor::Editor, lighting::*, scene::Actor};

pub fn start_bake(e: &mut Editor) {
    if e.playing || e.bake_job.is_some() {
        return;
    }
    let scene = e.scene.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    e.bake_job = Some(rx);
    e.log("Baking vertex lighting and static shadows on PC...");
    std::thread::spawn(move || {
        let _ = tx.send(bake(&scene));
    });
}
pub fn poll(e: &mut Editor) {
    let result = e.bake_job.as_ref().and_then(|rx| match rx.try_recv() {
        Ok(r) => Some(r),
        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
            Some(Err("Lighting worker stopped".into()))
        }
        Err(_) => None,
    });
    if let Some(result) = result {
        e.bake_job = None;
        match result {
            Ok(b) if b.fingerprint == fingerprint(&e.scene) => {
                let count: usize = b.colors.iter().map(Vec::len).sum();
                e.scene.bake = Some(b);
                e.changed();
                e.log(format!(
                    "Lighting baked: {count} vertex colors ({} bytes). Save Scene to persist.",
                    count * 3
                ));
            }
            Ok(_) => e.log("Scene changed during Bake. Result discarded; bake again."),
            Err(error) => e.log(format!("Bake failed: {error}")),
        }
    }
}
pub fn create(e: &mut Editor, kind: LightType, child: bool) {
    if e.playing || e.scene.actors.len() >= 512 {
        return;
    }
    let base = if kind == LightType::Directional {
        "Directional Light"
    } else {
        "Point Light"
    };
    let mut entity = Actor::cube(base.into());
    entity.kind = "Empty".into();
    entity.position = [0., 3., 0.];
    entity.light = Some(Light {
        kind,
        ..Light::default()
    });
    if kind == LightType::Directional {
        entity.rotation = [50., -30., 0.];
    }
    if child {
        entity.parent = e.selected;
        entity.position = [0.; 3];
    }
    let mut n = 1;
    while e.scene.actors.iter().any(|v| v.name == entity.name) {
        entity.name = format!("{base}.{n:03}");
        n += 1;
    }
    e.scene.actors.push(entity);
    if let Err(error) = e.scene.validate() {
        e.scene.actors.pop();
        e.log(error);
        return;
    }
    e.selected = Some(e.scene.actors.len() - 1);
    e.reveal_selected = true;
    e.search.clear();
    e.set_scene_2d(false);
    e.changed();
}
pub fn inspector(ui: &imgui::Ui, entity: &mut Actor) {
    if let Some(light) = &mut entity.light {
        if crate::gui::heading(ui, "Light") {
            ui.checkbox("Enabled##light", &mut light.enabled);
            let mut kind = usize::from(light.kind == LightType::Point);
            if ui.combo_simple_string(
                crate::gui::field(ui, "Type"),
                &mut kind,
                &["Directional", "Point"],
            ) {
                light.kind = if kind == 0 {
                    LightType::Directional
                } else {
                    LightType::Point
                };
            }
            let mut mode = match light.mode {
                LightMode::Baked => 0,
                LightMode::Realtime => 1,
                LightMode::Mixed => 2,
            };
            if ui.combo_simple_string(
                crate::gui::field(ui, "Mode"),
                &mut mode,
                &["Baked", "Realtime", "Mixed"],
            ) {
                light.mode = [LightMode::Baked, LightMode::Realtime, LightMode::Mixed][mode];
            }
            ui.color_edit3(crate::gui::field(ui, "Light Color"), &mut light.color);
            crate::gui::Drag::new(crate::gui::field(ui, "Intensity"))
                .speed(0.01)
                .range(0., 2.)
                .build(ui, &mut light.intensity);
            if light.kind == LightType::Point {
                crate::gui::Drag::new(crate::gui::field(ui, "Range"))
                    .speed(0.1)
                    .range(0.01, 128.)
                    .build(ui, &mut light.range);
            }
            crate::gui::Drag::new(crate::gui::field(ui, "Priority"))
                .range(-100, 100)
                .build(ui, &mut light.priority);
            if light.mode != LightMode::Realtime {
                ui.checkbox("Bake Shadows", &mut light.shadows);
            }
            ui.text_wrapped(
                "Realtime: 1 directional + 1 local light per object. Shadows are baked only.",
            );
            if light.kind == LightType::Directional {
                crate::gui::muted(ui, "Transform +Z is the light's travel direction.");
            }
            if ui.small_button("Remove Light") {
                entity.light = None;
            }
        }
        ui.separator();
    }
}
pub fn mesh(ui: &imgui::Ui, entity: &mut Actor) {
    let mut mode = if entity.material.unlit {
        0
    } else if entity.lighting.receive == Receive::Baked {
        1
    } else {
        2
    };
    if ui.combo_simple_string(
        crate::gui::field(ui, "Receive Lighting"),
        &mut mode,
        &["Unlit", "Baked Vertex", "Realtime (GTE)"],
    ) {
        entity.material.unlit = mode == 0;
        entity.lighting.receive = if mode == 1 {
            Receive::Baked
        } else {
            Receive::Realtime
        };
        if mode == 1 {
            entity.lighting.static_geometry = true;
        }
    }
    ui.checkbox("Cast Baked Shadows", &mut entity.lighting.cast_shadows);
    let mut n = i32::from(entity.lighting.subdivisions);
    if crate::gui::Drag::new(crate::gui::field(ui, "Subdivisions"))
        .range(1, 8)
        .build(ui, &mut n)
    {
        entity.lighting.subdivisions = n.clamp(1, 8) as u8;
    }
    crate::gui::muted(ui, format!("{} triangles", quad_count(entity) * 2));
}
pub fn window(ui: &imgui::Ui, e: &mut Editor) {
    if !e.lighting_window {
        return;
    }
    let mut opened = true;
    ui.window("Lighting").opened(&mut opened).position([80.,90.],imgui::Condition::FirstUseEver).size([480.,530.],imgui::Condition::FirstUseEver).size_constraints([380.,400.],[1400.,1200.]).build(||{
        let old=e.scene.environment.clone();
        let old_fog=e.scene.fog.clone();
        ui.disabled(e.playing,||{
            ui.color_edit3(crate::gui::field(ui, "Ambient"),&mut e.scene.environment.ambient);
            crate::gui::Drag::new(crate::gui::field(ui, "Baked AO Strength")).speed(0.01).range(0.,1.).build(ui,&mut e.scene.environment.ao_strength);
            crate::gui::Drag::new(crate::gui::field(ui, "AO Distance")).speed(0.05).range(0.01,32.).build(ui,&mut e.scene.environment.ao_distance);
            ui.checkbox("Realtime Point Lights",&mut e.scene.environment.point_lights);
            crate::effects::inspector(ui,&mut e.scene.fog);
        });
        if old!=e.scene.environment{e.changed();}
        if old_fog!=e.scene.fog{e.changed();}
        ui.separator();
        let needs=e.scene.actors.iter().any(baked);
        let status=if e.bake_job.is_some(){"Baking in background..."}else if !needs{"No baked receivers"}else if e.bake_current{"Bake up to date"}else{"Bake outdated - preview has no baked shadows"};
        ui.text_wrapped(status);
        ui.disabled(e.playing || e.bake_job.is_some(),||{if ui.button("Bake Lighting"){start_bake(e);}});
        ui.text_wrapped("Play / Build automatically bake outdated lighting. Save Scene after a manual bake to keep the cache.");
        ui.separator();
        let count:usize=e.scene.actors.iter().map(quad_count).sum();
        let vertices:usize=e.scene.actors.iter().filter(|e|baked(e)).map(|e|quad_count(e)*4).sum();
        ui.text(format!("Geometry: {} / 7000 triangles",count*2));
        ui.text(format!("Baked colors: {} bytes",vertices*3));
        ui.text_wrapped("Dynamic budget: 1 directional + 1 point per object; 32 active lights. Static objects must stay fixed, including their parents.");
    });
    e.lighting_window = opened;
}
