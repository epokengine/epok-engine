use crate::{
    assets::{self, Kind, Record},
    editor::Editor,
    skeletal::{self, Component, Model},
};
use imgui::{Condition, Ui};
use std::{sync::Arc, time::Instant};
#[derive(Default)]
pub struct State {
    pub open: bool,
    record: Option<Record>,
    preview: Option<Component>,
    pub error: Option<String>,
    playing: bool,
    pub animate_scene: bool,
    show_bones: bool,
    selected_vertex: i32,
    selected_bone: i32,
    yaw: f32,
    last: Option<Instant>,
    scene_accumulator: f32,
}
pub fn open(e: &mut Editor, record: Record) {
    let mesh_id = if record.meta.kind == Kind::SkeletalMesh {
        Some(record.meta.id)
    } else if record.meta.kind == Kind::ModelSource {
        if let crate::import_settings::Settings::Fbx(s) = &record.meta.settings {
            s.outputs.get("SkeletalMesh/main").map(|o| o.id)
        } else {
            None
        }
    } else {
        e.assets
            .index
            .usable()
            .filter(|r| r.meta.kind == Kind::SkeletalMesh)
            .find_map(|r| {
                Model::load(&e.assets.index, r.meta.id)
                    .ok()
                    .filter(|m| {
                        m.mesh.skeleton == record.meta.id
                            || m.mesh.clips.contains(&record.meta.id)
                            || m.mesh.materials.contains(&record.meta.id)
                    })
                    .map(|_| r.meta.id)
            })
    };
    let mut state = State {
        open: true,
        record: Some(record.clone()),
        playing: false,
        show_bones: true,
        yaw: -0.4,
        ..Default::default()
    };
    let args = std::env::args().collect::<Vec<_>>();
    if let Some(pair) = args.windows(2).find(|v| v[0] == "--preview-model-yaw") {
        if let Ok(yaw) = pair[1].parse::<f32>() {
            if yaw.is_finite() {
                state.yaw = yaw;
            }
        }
    }
    if args.iter().any(|v| v == "--preview-model-no-bones") {
        state.show_bones = false;
    }
    match mesh_id
        .ok_or("No skeletal mesh references this asset".into())
        .and_then(|id| Model::load(&e.assets.index, id).map(|model| (id, model)))
    {
        Ok((id, model)) => {
            let mut c = Component::new(id);
            if let Some(pair) = args.windows(2).find(|v| v[0] == "--preview-model-time") {
                if let Ok(time) = pair[1].parse::<f32>() {
                    if time.is_finite() && time >= 0.0 {
                        c.time = time;
                    }
                }
            }
            c.clip = if model.mesh.clips.contains(&record.meta.id) {
                Some(record.meta.id)
            } else {
                model.mesh.clips.first().copied()
            };
            c.model = Some(Arc::new(model));
            state.preview = Some(c);
        }
        Err(err) => state.error = Some(err),
    }
    e.skeletal_ui = state;
}
pub fn tick(e: &mut Editor) {
    let now = Instant::now();
    let s = &mut e.skeletal_ui;
    let dt = s
        .last
        .replace(now)
        .map_or(0., |t| now.duration_since(t).as_secs_f32().min(0.1));
    if s.open
        && s.playing
        && let Some(c) = &mut s.preview
    {
        c.time += dt;
    }
    s.scene_accumulator += dt;
    let steps = (s.scene_accumulator * 30.).floor();
    if steps > 0. {
        s.scene_accumulator -= steps / 30.;
    }
    if s.animate_scene && steps > 0. {
        for entity in &mut e.scene.actors {
            if let Some(c) = &mut entity.skeletal_mesh {
                c.time = ((c.time * 30.).round() + steps) / 30.;
                e.view_dirty = true;
            }
        }
    }
}
pub fn synchronize(e: &mut Editor) {
    let s = &mut e.skeletal_ui;
    if let Some(record) = &s.record {
        match e.assets.index.resolve(record.meta.id) {
            Ok(record) => s.record = Some(record.clone()),
            Err(error) => {
                s.error = Some(error);
                return;
            }
        }
    }
    if let Some(c) = &mut s.preview {
        match Model::load(&e.assets.index, c.asset) {
            Ok(model) => {
                c.model = Some(Arc::new(model));
                s.error = None;
            }
            Err(error) => {
                c.model = None;
                s.error = Some(error);
            }
        }
    }
}
pub fn sample(e: &mut Editor) -> Result<(), String> {
    let source = "assets/EpokMannequin.fbx";
    let path = assets::inside(&e.root, source)?;
    if !path.exists() {
        assets::atomic_write(
            &path,
            include_bytes!("../resources/models/EpokMannequin.fbx"),
            None,
        )?;
    }
    let destination = crate::model_import::destination(source);
    let model_path = assets::inside(&e.root, &destination)?;
    if model_path.exists() {
        let package = assets::Package::load(&model_path)?;
        e.assets.index = assets::scan(&e.root, &mut Default::default());
        let record = e.assets.index.resolve(package.meta.id)?.clone();
        open(e, record);
    } else {
        let item = crate::asset_manager::Pending {
            source: source.into(),
            hash: assets::hash(&assets::read_bounded(&path)?),
            existing: None,
            status: Default::default(),
        };
        e.assets.begin_pending(&item);
    }
    Ok(())
}
/// Rewrite one Material asset in place. The stored package is the source of
/// truth, so every field the model window does not expose (texture, blend,
/// depth bias, UV scroll, unlit) survives an edit. The write keeps the caller's
/// stale-revision protection and the payload is validated before it lands.
pub fn edit_material(
    path: &std::path::Path,
    revision: &str,
    edit: impl FnOnce(&mut crate::scene::Material),
) -> Result<(assets::Metadata, String, crate::scene::Material), String> {
    let mut package = assets::Package::load(path)?;
    let skeletal::Data::Material(mut material) = skeletal::Data::parse(&package.source)? else {
        return Err(format!(
            "{} is not a Material asset",
            path.file_name().unwrap_or_default().to_string_lossy()
        ));
    };
    edit(&mut material);
    crate::texture::validate_material(&material)?;
    package.source = serde_json::to_vec(&skeletal::Data::Material(material))
        .map_err(|e: serde_json::Error| e.to_string())?;
    let skeletal::Data::Material(material) = skeletal::Data::parse(&package.source)? else {
        unreachable!()
    };
    package.meta.source_hash = assets::hash(&package.source);
    let bytes = package.bytes()?;
    assets::atomic_write(path, &bytes, Some(revision))?;
    Ok((package.meta, assets::hash(&bytes), material))
}
fn playback(ui: &Ui, c: &mut Component) {
    let Some(model) = &c.model else {
        return;
    };
    let label = c
        .clip
        .and_then(|id| model.clips.iter().find(|(key, _)| *key == id))
        .map_or("Bind pose", |(_, clip)| clip.name.as_str());
    if let Some(_combo) = ui.begin_combo(crate::gui::field(ui, "Animation clip"), label) {
        if ui.selectable("Bind pose") {
            c.clip = None;
            c.time = 0.;
            c.error = None;
        }
        for (id, clip) in &model.clips {
            if ui
                .selectable_config(&clip.name)
                .selected(c.clip == Some(*id))
                .build()
            {
                c.clip = Some(*id);
                c.time = 0.;
                c.error = None;
            }
        }
    }
    if let Some((_, clip)) = model.clips.iter().find(|(id, _)| Some(*id) == c.clip) {
        let duration = (clip.frames - 1) as f32 / 30.;
        let mut t = if c.looping && duration > 0. {
            c.time % duration
        } else {
            c.time.min(duration)
        };
        if ui.slider(
            crate::gui::field(ui, "Time (s)"),
            0.,
            duration.max(0.001),
            &mut t,
        ) {
            c.time = t;
        }
    }
    crate::gui::toggle(ui, "Loop", &mut c.looping);
}
pub fn window(ui: &Ui, e: &mut Editor) {
    if !e.skeletal_ui.open {
        return;
    }
    let mut s = std::mem::take(&mut e.skeletal_ui);
    let mut open = s.open;
    ui.window("Skeletal Model")
        .opened(&mut open)
        .position(
            [ui.io().display_size[0] * 0.5, ui.io().display_size[1] * 0.5],
            Condition::Appearing,
        )
        .position_pivot([0.5, 0.5])
        .size(
            [940., (ui.io().display_size[1] - 90.).min(780.)],
            Condition::FirstUseEver,
        )
        .size_constraints([600., 540.], [2000., 1600.])
        .build(|| {
            if let Some(record) = &s.record {
                ui.text_wrapped(assets::path_string(&e.root, &record.path));
                if record.meta.kind == Kind::ModelSource {
                    if ui.button("Reimport FBX...") {
                        e.assets.begin_reimport(record, false);
                    }
                    crate::gui::inline(ui, "Rebuild from stored FBX...");
                    if ui.button("Rebuild from stored FBX...") {
                        e.assets.begin_reimport(record, true);
                    }
                    if let crate::import_settings::Settings::Fbx(options) = &record.meta.settings {
                        for warning in &options.warnings {
                            ui.text_colored([1., 0.72, 0.38, 1.], warning);
                        }
                    }
                }
            }
            if let Some(error) = &s.error {
                ui.text_wrapped(error);
            }
            let Some(c) = &mut s.preview else {
                return;
            };
            let Some(m) = c.model.clone() else {
                return;
            };
            crate::gui::muted(
                ui,
                format!(
                    "{} vertices   {} triangles   {} bones/helpers   {} clips",
                    m.mesh.vertices.len(),
                    m.mesh.triangles.len(),
                    m.skeleton.bones.len(),
                    m.clips.len()
                ),
            );
            crate::gui::muted(
                ui,
                "PSX preview: rigid weights, quantized poses, 30 Hz samples, assigned textures",
            );
            if ui.button(if s.playing {
                "Pause preview"
            } else {
                "Play preview"
            }) {
                s.playing = !s.playing;
            }
            crate::gui::inline(ui, "Restart");
            if ui.button("Restart") {
                c.time = 0.;
            }
            crate::gui::inline(ui, "Show skeleton");
            ui.checkbox("Show skeleton", &mut s.show_bones);
            playback(ui, c);
            if ui.button("Add character to scene") {
                let mut entity = crate::scene::Actor::cube("Character".into());
                entity.position = [0.; 3];
                let mut component = c.clone();
                component.time = 0.;
                entity.skeletal_mesh = Some(component);
                e.scene.actors.push(entity);
                e.selected = Some(e.scene.actors.len() - 1);
                e.changed();
            }
            ui.same_line();
            crate::gui::muted(ui, "Drag preview to orbit");
            // Host preview textures come from the shared asset preview cache,
            // which the editor loop uploads to the imgui renderer each frame.
            e.project_browser
                .previews
                .ensure_project(&e.root, &e.assets.index);
            let mut slot_textures = vec![None; m.materials.len()];
            for (slot, material) in m.materials.iter().enumerate() {
                let Some(texture) = material.texture else {
                    continue;
                };
                let Some((path, revision)) = e
                    .assets
                    .index
                    .resolve(texture)
                    .ok()
                    .map(|r| (r.path.clone(), r.revision.clone()))
                else {
                    continue;
                };
                e.project_browser.previews.request(&path, &revision);
                if let Some(crate::content_preview::Preview::Image {
                    texture: Some(handle),
                    ..
                }) = e.project_browser.previews.get(&path)
                {
                    slot_textures[slot] = Some(*handle);
                }
            }
            let origin = ui.cursor_screen_pos();
            let size = [
                ui.content_region_avail()[0].max(1.),
                (ui.content_region_avail()[1] - 70.).max(160.),
            ];
            ui.invisible_button("model-preview", size);
            if ui.is_item_active() {
                s.yaw += ui.io().mouse_delta[0] * 0.01;
            }
            let dl = ui.get_window_draw_list();
            dl.add_rect(
                origin,
                [origin[0] + size[0], origin[1] + size[1]],
                [0.07, 0.08, 0.10, 1.],
            )
            .filled(true)
            .build();
            let points = m.points(c.clip, c.time, c.looping);
            let bind = m.points(None, 0., false);
            let lo: [f32; 3] =
                std::array::from_fn(|a| bind.iter().map(|p| p[a]).fold(f32::INFINITY, f32::min));
            let hi: [f32; 3] = std::array::from_fn(|a| {
                bind.iter().map(|p| p[a]).fold(f32::NEG_INFINITY, f32::max)
            });
            let center = std::array::from_fn::<_, 3, _>(|a| (lo[a] + hi[a]) * 0.5);
            let extent = (0..3).map(|a| hi[a] - lo[a]).fold(0.1_f32, f32::max);
            let scale = size[1].min(size[0]) * 0.72 / extent;
            let project = |p: [f32; 3]| {
                let p = crate::lighting::sub(p, center);
                let (sin, cos) = s.yaw.sin_cos();
                let x = p[0] * cos - p[2] * sin;
                let z = p[0] * sin + p[2] * cos;
                (
                    [
                        origin[0] + size[0] * 0.5 + x * scale,
                        origin[1] + size[1] * 0.52 - (p[1] * 0.96 - z * 0.28) * scale,
                    ],
                    z,
                )
            };
            dl.with_clip_rect(origin, [origin[0] + size[0], origin[1] + size[1]], || {
                let mut faces = m
                    .mesh
                    .triangles
                    .iter()
                    .map(|t| {
                        let p = t.indices.map(|v| project(points[v as usize]));
                        (p.iter().map(|v| v.1).sum::<f32>(), t, p)
                    })
                    .collect::<Vec<_>>();
                faces.sort_by(|a, b| b.0.total_cmp(&a.0));
                for (_, t, p) in faces {
                    let mat = &m.materials[t.material as usize];
                    let color = [mat.color[0], mat.color[1], mat.color[2], 1.];
                    match slot_textures[t.material as usize] {
                        // The fourth corner repeats the third, so the quad's
                        // second triangle is degenerate and only the authored
                        // corner coordinates are sampled.
                        Some(handle) => {
                            let uv = crate::skeletal::corner_uv(t);
                            dl.add_image_quad(handle, p[0].0, p[1].0, p[2].0, p[2].0)
                                .uv(uv[0], uv[1], uv[2], uv[2])
                                .col(color)
                                .build();
                        }
                        None => {
                            dl.add_triangle(p[0].0, p[1].0, p[2].0, color)
                                .filled(true)
                                .build();
                        }
                    }
                    dl.add_triangle(p[0].0, p[1].0, p[2].0, [0.1, 0.12, 0.15, 0.5])
                        .build();
                }
                if s.show_bones {
                    let bones = m.bones(c.clip, c.time, c.looping);
                    for (i, b) in m.skeleton.bones.iter().enumerate() {
                        let a = project(bones[i].point([0.; 3])).0;
                        if b.parent >= 0 {
                            let parent = project(bones[b.parent as usize].point([0.; 3])).0;
                            dl.add_line(a, parent, [0.3, 0.85, 1., 1.])
                                .thickness(2.)
                                .build();
                        }
                        dl.add_circle(a, 3., [1., 0.8, 0.2, 1.])
                            .filled(true)
                            .build();
                    }
                }
            });
            if let Some(_tree) = ui.tree_node("Bones and parents") {
                for (i, b) in m.skeleton.bones.iter().enumerate() {
                    ui.text(format!("{i}: {}  (parent {})", b.name, b.parent));
                }
            }
            if let Some(_tree) = ui.tree_node("Gameplay query references") {
                crate::gui::muted(
                    ui,
                    "Portable indices are stable across PSX storage formats for this imported topology.",
                );
                let vertex_max = m.mesh.vertices.len().saturating_sub(1) as i32;
                if ui.input_int("Portable vertex", &mut s.selected_vertex).build() {
                    s.selected_vertex = s.selected_vertex.clamp(0, vertex_max);
                }
                s.selected_vertex = s.selected_vertex.clamp(0, vertex_max);
                if let Some(vertex) = m.mesh.vertices.get(s.selected_vertex as usize) {
                    ui.text(format!(
                        "Bind Q12: [{}, {}, {}]   strongest bone: {}",
                        vertex.position[0], vertex.position[1], vertex.position[2], vertex.bone
                    ));
                    if ui.button("Copy vertex index") {
                        ui.set_clipboard_text(s.selected_vertex.to_string());
                    }
                }
                let bone_max = m.skeleton.bones.len().saturating_sub(1) as i32;
                if ui.input_int("Bone", &mut s.selected_bone).build() {
                    s.selected_bone = s.selected_bone.clamp(0, bone_max);
                }
                s.selected_bone = s.selected_bone.clamp(0, bone_max);
                if let Some(bone) = m.skeleton.bones.get(s.selected_bone as usize) {
                    ui.text(format!("{}   parent {}", bone.name, bone.parent));
                    if ui.button("Copy bone index") {
                        ui.set_clipboard_text(s.selected_bone.to_string());
                    }
                }
                ui.text(format!("Topology identity: {}", c.asset));
            }
            if let Some(_tree) = ui.tree_node("Materials") {
                crate::gui::muted(
                    ui,
                    "Texture images are assigned here; imported coordinates stay with the mesh.",
                );
                for (slot, id) in m.mesh.materials.iter().enumerate() {
                    let _id = ui.push_id_usize(slot);
                    ui.text(format!("Slot {slot}"));
                    let mut edited = m.materials[slot].clone();
                    let mut changed = ui.color_edit3("Color", &mut edited.color);
                    changed |= crate::texture::picker(ui, &e.assets.index, &mut edited);
                    if !changed {
                        continue;
                    }
                    let Ok(record) = e.assets.index.resolve(*id).cloned() else {
                        continue;
                    };
                    match edit_material(&record.path, &record.revision, |material| {
                        material.color = edited.color;
                        material.texture = edited.texture;
                        material.blend = edited.blend;
                        material.depth_bias = edited.depth_bias;
                        material.uv_scroll = edited.uv_scroll;
                    }) {
                        Ok((meta, revision, material)) => {
                            if let Some(records) = e.assets.index.assets.get_mut(id) {
                                for r in records {
                                    if r.path == record.path {
                                        r.meta = meta.clone();
                                        r.revision = revision.clone();
                                    }
                                }
                            }
                            e.assets.refresh();
                            s.error = None;
                            let mut updated = (*m).clone();
                            updated.materials[slot] = material;
                            c.model = Some(Arc::new(updated));
                        }
                        Err(error) => s.error = Some(error),
                    }
                }
            }
        });
    s.open = open;
    e.skeletal_ui = s;
}
pub fn component(ui: &Ui, e: &mut Editor, entity: &mut crate::scene::Actor) {
    if entity.skeletal_mesh.is_none() {
        return;
    }
    ui.separator();
    let mut remove = false;
    let expanded = crate::gui::section(ui, "Skeletal Mesh / Animator", || {
        remove = ui.menu_item("Remove Skeletal Mesh");
    });
    if remove {
        entity.skeletal_mesh = None;
        entity.kind = "Empty".into();
        return;
    }
    if !expanded {
        return;
    }
    crate::mesh_editor::selector(ui, e, entity);
    let Some(c) = entity.skeletal_mesh.as_mut() else {
        return;
    };
    if let Some(error) = &c.error {
        ui.text_wrapped(error);
    }
    playback(ui, c);
    crate::gui::toggle(ui, "Play on start (PSX)", &mut c.play_on_start);
    crate::gui::toggle(
        ui,
        "Animate scene preview",
        &mut e.skeletal_ui.animate_scene,
    );
    if ui.button("Open model preview")
        && let Ok(record) = e.assets.index.resolve(c.asset).cloned()
    {
        open(e, record);
    }
}
