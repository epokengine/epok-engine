//! File selection has its own Inspector and isolated preview scene/camera.
use crate::{
    assets::{self, Index, Kind},
    content_preview::{Cache, Preview},
    editor::Editor,
    scene::{Actor, Scene},
    skeletal::{self, Data, Model},
    viewport::View,
};
use imgui::Ui;
use std::{
    path::{Path, PathBuf},
    sync::{Arc, mpsc},
    time::{Instant, SystemTime},
};

#[derive(Clone, PartialEq)]
struct Key {
    path: PathBuf,
    modified: Option<SystemTime>,
    size: u64,
    revision: String,
}
pub struct Details {
    pub fields: Vec<(String, String)>,
    pub scene: Option<Scene>,
    pub bounds: Option<([f32; 3], [f32; 3])>,
    pub text: Option<String>,
    pub kind: String,
}
pub struct State {
    key: Option<Key>,
    checked: Option<Instant>,
    pending: Option<mpsc::Receiver<(Key, Result<Details, String>)>>,
    pub details: Option<Details>,
    error: Option<String>,
    pub camera: View,
    pub texture: Option<imgui::TextureId>,
    pub visible: bool,
    pub model_dirty: bool,
    pub media: Cache,
    height: f32,
    collapsed: bool,
    image_zoom: f32,
}
impl Default for State {
    fn default() -> Self {
        Self {
            key: None,
            checked: None,
            pending: None,
            details: None,
            error: None,
            camera: View::default(),
            texture: None,
            visible: false,
            model_dirty: true,
            media: Cache::inspector(),
            height: 260.,
            collapsed: false,
            image_zoom: 1.,
        }
    }
}
impl State {
    fn update(&mut self, root: &Path, path: &Path, index: &Index) {
        let changed = self.key.as_ref().is_none_or(|key| key.path != path);
        if changed || self.checked.is_none_or(|t| t.elapsed().as_secs() >= 2) {
            self.checked = Some(Instant::now());
            let metadata = path.metadata();
            let key = Key {
                path: path.into(),
                modified: metadata.as_ref().ok().and_then(|m| m.modified().ok()),
                size: metadata.as_ref().map_or(0, |m| m.len()),
                revision: format!(
                    "{}:{:?}",
                    index.fingerprint(),
                    crate::workspace::optional_manifest(root)
                        .map(|m| m.and_then(|m| m.default_sound_bank))
                ),
            };
            if self.key.as_ref() != Some(&key) {
                self.key = Some(key);
                self.details = None;
                self.error = None;
                if changed {
                    self.camera = View::default();
                    self.image_zoom = 1.;
                }
            }
        }
        if let Some(result) = self.pending.as_ref().and_then(|rx| match rx.try_recv() {
            Ok(r) => Some(r),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => Some((
                self.key.clone().unwrap(),
                Err("Preview worker stopped".into()),
            )),
        }) {
            self.pending = None;
            if self.key.as_ref() == Some(&result.0) {
                match result.1 {
                    Ok(details) => {
                        self.model_dirty = true;
                        if let Some((lo, hi)) = details.bounds {
                            self.camera.frame_bounds(lo, hi);
                        }
                        self.details = Some(details);
                    }
                    Err(error) => self.error = Some(error),
                }
            }
        }
        if self.details.is_none() && self.error.is_none() && self.pending.is_none() {
            let key = self.key.clone().unwrap();
            let index = index.clone();
            let root = root.to_owned();
            let (send, receive) = mpsc::sync_channel(1);
            std::thread::spawn(move || {
                let result = load(&root, &key.path, &index);
                let _ = send.send((key, result));
            });
            self.pending = Some(receive);
        }
    }
}

pub fn draw(ui: &Ui, e: &mut Editor) {
    let Some(path) = e.selected_asset.clone() else {
        return;
    };
    let mut state = std::mem::take(&mut e.asset_inspector);
    state.update(&e.root, &path, &e.assets.index);
    let available = ui.content_region_avail();
    let preview_height = if state.collapsed {
        30.
    } else {
        state.height.min((available[1] - 120.).max(80.))
    };
    ui.child_window("file-details")
        .size([0., (available[1] - preview_height - 7.).max(60.)])
        .build(|| {
            ui.text_wrapped(path.file_name().unwrap_or_default().to_string_lossy());
            ui.text_disabled("File Inspector");
            ui.separator();
            ui.text_wrapped(assets::path_string(&e.root, &path));
            if let Some(details) = &state.details {
                for (label, value) in &details.fields {
                    ui.spacing();
                    ui.text_disabled(label);
                    ui.text_wrapped(value);
                }
            } else if let Some(error) = &state.error {
                ui.text_wrapped(error);
            } else {
                ui.text_disabled("Loading file details...");
            }
            ui.spacing();
            let audio_record = e
                .assets
                .index
                .usable()
                .find(|r| {
                    r.path == path
                        && matches!(
                            r.meta.kind,
                            Kind::AudioClip | Kind::MusicSequence | Kind::SoundBank
                        )
                })
                .cloned();
            if let Some(record) = audio_record {
                if ui.button("Edit audio import settings") {
                    e.assets.begin_reimport(&record, true);
                }
                ui.text_disabled("Uses the stored source; keeps this asset's UUID.");
            }
            if ui.button("Open file editor") {
                crate::project_browser::open_path(e, &path);
            }
            if let Some(text) = state.details.as_ref().and_then(|d| d.text.as_ref()) {
                ui.separator();
                ui.text_wrapped(text);
            }
        });
    let divider = ui.cursor_screen_pos();
    ui.invisible_button("resize-file-preview", [available[0].max(1.), 6.]);
    #[cfg(test)]
    track(ui, "resize");
    if ui.is_item_hovered() || ui.is_item_active() {
        ui.set_mouse_cursor(Some(imgui::MouseCursor::ResizeNS));
    }
    if ui.is_item_active() {
        state.height =
            (state.height - ui.io().mouse_delta[1]).clamp(80., (available[1] - 120.).max(80.));
    }
    ui.get_window_draw_list()
        .add_line(
            [divider[0], divider[1] + 3.],
            [divider[0] + available[0], divider[1] + 3.],
            crate::gui::gray(65),
        )
        .build();
    ui.child_window("file-preview")
        .size([0., 0.])
        .scroll_bar(false)
        .build(|| {
            if ui.small_button(if state.collapsed {
                "> Preview"
            } else {
                "v Preview"
            }) {
                state.collapsed = !state.collapsed;
            }
            if state.collapsed {
                return;
            }
            ui.same_line();
            if ui.small_button("Reset view") {
                state.model_dirty = true;
                state.camera = View::default();
                state.image_zoom = 1.;
                if let Some((lo, hi)) = state.details.as_ref().and_then(|d| d.bounds) {
                    state.camera.frame_bounds(lo, hi);
                }
            }
            let kind = state.details.as_ref().map_or("", |d| d.kind.as_str());
            if kind == "Audio" {
                e.project_browser.previews.mode_controls(ui);
                state.media.set_mode(e.project_browser.previews.mode());
                ui.same_line();
                if ui.small_button(if e.project_browser.previews.active(&path) {
                    "Stop"
                } else {
                    "Play"
                }) {
                    e.project_browser.previews.toggle(&path);
                }
                #[cfg(test)]
                track(ui, "play");
                if let Some(report) = &e.project_browser.previews.report {
                    ui.text_wrapped(report);
                }
            }
            let origin = ui.cursor_screen_pos();
            let size = ui.content_region_avail().map(|v| v.max(1.));
            ui.invisible_button("asset-preview-view", size);
            #[cfg(test)]
            track(ui, "view");
            let hovered = ui.is_item_hovered();
            state.visible = ui.is_item_visible();
            let bottom = [origin[0] + size[0], origin[1] + size[1]];
            let draw = ui.get_window_draw_list();
            draw.add_rect(origin, bottom, [0.10, 0.11, 0.12, 1.])
                .filled(true)
                .build();
            if state.details.as_ref().is_some_and(|d| d.scene.is_some()) {
                if ui.is_item_active() && ui.io().mouse_delta != [0.; 2] {
                    state.camera.look(ui.io().mouse_delta, true);
                    state.model_dirty = true;
                }
                if hovered && ui.io().mouse_wheel != 0. {
                    state.camera.zoom_orbit(ui.io().mouse_wheel);
                    state.model_dirty = true;
                }
                if let Some(texture) = state.texture {
                    let (lo, hi) = fit(origin, size, [960., 600.], 1.);
                    draw.add_image(texture, lo, hi).build();
                }
                if hovered {
                    ui.tooltip_text(
                        "Drag to orbit · Wheel to zoom · Reset view to frame the model",
                    );
                }
            } else if kind == "Texture" || kind == "Font" {
                let revision = crate::content_preview::revision(&e.root, &e.assets.index, &path);
                state.media.request(&path, revision);
                if hovered {
                    state.image_zoom =
                        (state.image_zoom * 1.2f32.powf(ui.io().mouse_wheel)).clamp(0.25, 8.);
                }
                if let Some(Preview::Image {
                    width,
                    height,
                    texture: Some(texture),
                    ..
                }) = state.media.get(&path)
                {
                    let (lo, hi) = fit(
                        origin,
                        size,
                        [*width as f32, *height as f32],
                        state.image_zoom,
                    );
                    draw.with_clip_rect(origin, bottom, || {
                        for y in 0..(size[1] / 12.).ceil() as usize {
                            for x in 0..(size[0] / 12.).ceil() as usize {
                                let p = [origin[0] + x as f32 * 12., origin[1] + y as f32 * 12.];
                                draw.add_rect(
                                    p,
                                    [p[0] + 12., p[1] + 12.],
                                    crate::gui::gray(if (x + y) % 2 == 0 { 40 } else { 49 }),
                                )
                                .filled(true)
                                .build();
                            }
                        }
                        draw.add_image(*texture, lo, hi).build();
                    });
                } else {
                    draw.add_text(
                        [origin[0] + 8., origin[1] + 8.],
                        [0.7, 0.7, 0.7, 1.],
                        state.media.failure(&path).unwrap_or("Loading image..."),
                    );
                }
            } else if kind == "Audio" {
                e.project_browser
                    .previews
                    .ensure_project(&e.root, &e.assets.index);
                e.project_browser.previews.request(
                    &path,
                    crate::content_preview::revision(&e.root, &e.assets.index, &path),
                );
                if let Some(Preview::Audio { peaks, duration }) =
                    e.project_browser.previews.get(&path)
                {
                    let center = origin[1] + size[1] * 0.5;
                    for (i, peak) in peaks.iter().enumerate() {
                        let x = origin[0] + 8. + i as f32 * (size[0] - 16.) / peaks.len() as f32;
                        let h = (peak * size[1] * 0.35).max(0.5);
                        draw.add_line([x, center - h], [x, center + h], [0.73, 0.48, 0.87, 1.])
                            .thickness(2.)
                            .build();
                    }
                    let progress = e.project_browser.previews.progress(&path);
                    draw.add_text(
                        [origin[0] + 8., origin[1] + 8.],
                        [0.8, 0.8, 0.8, 1.],
                        format!("{:.1} / {:.1} s", duration * progress, duration),
                    );
                    draw.add_line(
                        [origin[0] + size[0] * progress, origin[1]],
                        [origin[0] + size[0] * progress, bottom[1]],
                        [1.; 4],
                    )
                    .build();
                } else {
                    draw.add_text(
                        [origin[0] + 8., origin[1] + 8.],
                        [0.7, 0.7, 0.7, 1.],
                        e.project_browser
                            .previews
                            .failure(&path)
                            .unwrap_or("Loading waveform..."),
                    );
                }
            } else {
                draw.add_text(
                    [origin[0] + 8., origin[1] + 8.],
                    [0.7, 0.7, 0.7, 1.],
                    "No visual preview for this file type",
                );
            }
        });
    e.asset_inspector = state;
}
fn fit(origin: [f32; 2], size: [f32; 2], image: [f32; 2], zoom: f32) -> ([f32; 2], [f32; 2]) {
    let scale = (size[0] / image[0]).min(size[1] / image[1]) * zoom;
    let wh = image.map(|v| v * scale);
    let lo = [
        origin[0] + (size[0] - wh[0]) * 0.5,
        origin[1] + (size[1] - wh[1]) * 0.5,
    ];
    (lo, [lo[0] + wh[0], lo[1] + wh[1]])
}

fn load(root: &Path, path: &Path, index: &Index) -> Result<Details, String> {
    let meta = path.metadata().map_err(|e| e.to_string())?;
    let record = index.usable().find(|r| r.path == path);
    let mut details = Details {
        fields: vec![],
        scene: None,
        bounds: None,
        text: None,
        kind: if meta.is_dir() {
            "Folder".into()
        } else {
            crate::content_preview::source_kind(path)
                .unwrap_or_else(|| {
                    let name = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_ascii_lowercase();
                    if name.ends_with(".epokbp") {
                        "Blueprint"
                    } else if name.ends_with(".epokmap") {
                        "Scene"
                    } else if name.ends_with(".timeline.json") {
                        "Timeline"
                    } else if name.ends_with(".particle-effect.json") {
                        "Particle Effect"
                    } else if name.ends_with(".epokasset") {
                        "Epok Asset"
                    } else if [".hpp", ".cpp", ".h", ".c"]
                        .iter()
                        .any(|ext| name.ends_with(ext))
                    {
                        "C++ Script"
                    } else {
                        "File"
                    }
                })
                .into()
        },
    };
    if meta.is_file() {
        details
            .fields
            .push(("File size".into(), format!("{} bytes", meta.len())));
    }
    if let Ok(elapsed) = meta
        .modified()
        .and_then(|t| t.elapsed().map_err(std::io::Error::other))
    {
        let seconds = elapsed.as_secs();
        details.fields.push((
            "Modified".into(),
            if seconds < 60 {
                "Just now".into()
            } else if seconds < 3600 {
                format!("{} minutes ago", seconds / 60)
            } else if seconds < 86400 {
                format!("{} hours ago", seconds / 3600)
            } else {
                format!("{} days ago", seconds / 86400)
            },
        ));
    }
    if record.is_none() && matches!(details.kind.as_str(), "Audio" | "SoundBank") {
        let bytes = assets::read_bounded(path)?;
        match crate::content_preview::audio_content_kind(&bytes) {
            Some("Sequence" | "Audio") => details.kind = "Audio".into(),
            Some("SoundBank") => details.kind = "SoundBank".into(),
            _ => {} // Unidentified raw VB remains a companion candidate, never a detected format.
        }
    }
    if let Some(r) = record {
        details.kind = match r.meta.kind {
            Kind::Texture => "Texture",
            Kind::AudioClip | Kind::MusicSequence => "Audio",
            Kind::SoundBank => "SoundBank",
            Kind::EditableMesh | Kind::SkeletalMesh | Kind::ModelSource => "Mesh",
            Kind::Terrain => "Terrain",
            Kind::Skeleton => "Skeleton",
            Kind::AnimationClip => "Animation",
            Kind::Material => "Material",
            Kind::Font => "Font",
        }
        .into();
        details.fields.extend([
            ("Asset type".into(), format!("{:?}", r.meta.kind)),
            ("Asset ID".into(), r.meta.id.to_string()),
            ("Source".into(), r.meta.source.clone()),
        ]);
        if matches!(r.meta.kind, Kind::MusicSequence | Kind::SoundBank) {
            details.fields.push(("Target".into(), "PSX".into()));
            let name = if r.meta.kind == Kind::MusicSequence {
                "Sequence.epokcache"
            } else {
                "Bank.epokcache"
            };
            let summary = root
                .join(".epok/imported")
                .join(assets::cache_key(&r.meta))
                .join(name);
            if let Ok(bytes) = assets::read_bounded(&summary)
                && let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes)
            {
                let fresh = value["inputs"].as_object().is_some_and(|inputs| {
                    inputs.iter().all(|(key, hash)| {
                        if key == "default-sound-bank" {
                            return crate::workspace::optional_manifest(root)
                                .ok()
                                .flatten()
                                .and_then(|m| m.default_sound_bank)
                                .is_some_and(|id| hash.as_str() == Some(id.to_string().as_str()));
                        }
                        key.strip_prefix("asset:")
                            .and_then(|id| id.parse().ok())
                            .and_then(|id| index.resolve(id).ok())
                            .is_some_and(|record| {
                                hash.as_str() == Some(assets::cache_key(&record.meta).as_str())
                            })
                    })
                });
                if fresh {
                    let report = value.get("report").unwrap_or(&value);
                    if let Some(profile) = report["profile"].as_str() {
                        details
                            .fields
                            .push(("Last cook: PSX profile".into(), profile.into()));
                    }
                    for (key, label) in [
                        ("sequence_bytes", "Last cook: sequence RAM bytes"),
                        (
                            "bank_bytes",
                            "Last cook: bank RAM bytes (includes sample source)",
                        ),
                        ("spu_ram_bytes", "Last cook: SPU bytes"),
                        (
                            "package_bytes",
                            "Authoring packages with dependencies, bytes",
                        ),
                        ("voice_limit", "PSX voice ceiling"),
                        ("peak_polyphony", "Analyzed peak polyphony"),
                    ] {
                        if let Some(value) = report.get(key) {
                            details.fields.push((label.into(), value.to_string()));
                        }
                    }
                    if let Some(warnings) = report["warnings"].as_array() {
                        for warning in warnings.iter().filter_map(|v| v.as_str()) {
                            details
                                .fields
                                .push(("PSX cook note".into(), warning.into()));
                        }
                    }
                    details.fields.push(("PSX media / runtime".into(), "Sequence and bank ship inside the EXE; no continuous sequence CD reads. Linked runtime RAM is measured in the build report; hardware timing/stack require profiling.".into()));
                } else {
                    details.fields.push(("Target cost".into(), "Previous cook is obsolete after a dependency change. Build or request PSX Target Preview.".into()));
                }
            }
        }
        if let Ok(bank) = r.meta.settings.sound_bank() {
            if bank.library.is_some() {
                details.fields.extend(crate::soundfont_asset::inspect(
                    &assets::Package::load(&r.path)?.source,
                )?);
                details
                    .fields
                    .push(("Provenance / license".into(), bank.provenance.clone()));
                details
                    .fields
                    .push(("Sample Load Mode".into(), format!("{:?}", bank.load_mode)));
            } else if bank.imported.is_some() {
                details
                    .fields
                    .extend(crate::bank_compat::inspect(&assets::Package::load(
                        &r.path,
                    )?)?);
            } else {
                details.fields.extend([
                ("Sample Load Mode".into(), format!("{:?} (Auto resolves to Resident)", bank.load_mode)),
                ("Bank mappings".into(), format!("{} programs/drum mappings; {} zones; {} unique sample references; one tone per matched note", bank.programs.len(), bank.programs.iter().map(|p| p.zones.len()).sum::<usize>(), bank.dependencies().len())),
                ("Provenance / license".into(), bank.provenance.clone()),
                ("Target cost scope".into(), "Last-cook values are shown above when available; other costs remain unknown. Original samples are preserved.".into()),
            ]);
                for id in bank.dependencies() {
                    details.fields.push((
                        "Sample dependency".into(),
                        index
                            .resolve(id)
                            .map(|r| assets::path_string(root, &r.path))
                            .unwrap_or_else(|e| e),
                    ));
                }
            }
        }
        if let crate::import_settings::Settings::Audio(settings) = &r.meta.settings {
            let result = settings.target_result();
            details.fields.extend([
                ("Role".into(), format!("{:?}", settings.role)),
                (
                    "Load Mode".into(),
                    format!("{:?} -> {:?}", settings.load_mode, result.resolved_mode),
                ),
                (
                    "Target / profile".into(),
                    format!("{} / {}", result.target, result.profile),
                ),
                (
                    "Resolved representation".into(),
                    result.representation.into(),
                ),
                (
                    "Compatibility".into(),
                    result.error.unwrap_or_else(|| {
                        "Supported; aggregate budget checked during build".into()
                    }),
                ),
                (
                    "Sample voices per play".into(),
                    result.sample_voices.to_string(),
                ),
                (
                    "Memory cost".into(),
                    "Unknown until requested target cook/build".into(),
                ),
            ]);
            details.fields.push((
                "Audio settings".into(),
                format!(
                    "{:?} · {} Hz · {} channel(s)\nTrim: {:.3}s — {}\nNormalize: {} · Loop: {}",
                    settings.role,
                    settings.rate(),
                    settings.channels,
                    settings.trim_start,
                    settings
                        .trim_end
                        .map_or("End".into(), |v| format!("{v:.3}s")),
                    settings.normalize,
                    settings.looping
                ),
            ));
            let report = root
                .join(".epok/imported")
                .join(assets::cache_key(&r.meta))
                .join("Import.epokcache");
            if let Ok(bytes) = std::fs::read(report)
                && let Ok(report) = crate::document::from_slice::<serde_json::Value>(&bytes)
            {
                for (label, key) in [
                    ("Last cook: encoded bytes", "encoded_bytes"),
                    ("Last cook: main RAM sample copy", "main_ram_bytes"),
                    ("Last cook: SPU samples", "spu_ram_bytes"),
                ] {
                    let value = report["target"][key]
                        .as_u64()
                        .map_or_else(|| "Unknown".into(), |v| format!("{v} bytes"));
                    details.fields.push((label.into(), value));
                }
            }
        }
    } else {
        details.fields.push(("Type".into(), details.kind.clone()));
    }
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let mut entity = None;
    let mut points = vec![];
    if record.is_some_and(|r| r.meta.kind == Kind::EditableMesh) || extension == "obj" {
        let doc = if let Some(r) = record {
            crate::mesh::document(r)?
        } else {
            crate::obj_import::parse(
                &String::from_utf8(assets::read_bounded(path)?).map_err(|e| e.to_string())?,
                1.,
                &Default::default(),
            )?
        };
        points = doc.vertices.clone();
        details.fields.push((
            "Geometry".into(),
            format!(
                "{} vertices · {} faces · {} materials",
                doc.vertices.len(),
                doc.faces.len(),
                doc.materials.len()
            ),
        ));
        let mut e = Actor::cube("Asset preview".into());
        e.position = [0.; 3];
        let mut c =
            crate::mesh::Component::new(record.map_or_else(uuid::Uuid::new_v4, |r| r.meta.id));
        c.document = Some(Arc::new(doc));
        e.editable_mesh = Some(c);
        entity = Some(e);
    } else if record.is_some_and(|r| r.meta.kind == Kind::Terrain) {
        let doc = crate::terrain::document(record.unwrap())?;
        let cells = doc.cells();
        let span = doc.span();
        details.fields.push((
            "Grid".into(),
            format!(
                "{} x {} cells of {:.2} units · {:.1} x {:.1} units · {} quads",
                cells[0],
                cells[1],
                doc.cell_size(),
                span[0],
                span[1],
                doc.quad_count()
            ),
        ));
        details.fields.push((
            "Height range".into(),
            format!("{:.2} to {:.2}", doc.lowest(), doc.highest()),
        ));
        let mut e = Actor::cube("Asset preview".into());
        e.position = [0.; 3];
        let mut c =
            crate::terrain::Component::new(record.map_or_else(uuid::Uuid::new_v4, |r| r.meta.id));
        c.document = Some(Arc::new(doc));
        e.terrain = Some(c);
        points = crate::lighting::quads(&e)
            .into_iter()
            .flat_map(|q| q.points)
            .collect();
        entity = Some(e);
    } else if extension == "fbx"
        || record.is_some_and(|r| {
            matches!(
                r.meta.kind,
                Kind::SkeletalMesh | Kind::ModelSource | Kind::Skeleton | Kind::AnimationClip
            )
        })
    {
        let model = if extension == "fbx" {
            raw_model(&assets::read_bounded(path)?)?
        } else {
            let r = record.unwrap();
            let id = if r.meta.kind == Kind::SkeletalMesh {
                r.meta.id
            } else if let crate::import_settings::Settings::Fbx(settings) = &r.meta.settings {
                settings
                    .outputs
                    .get("SkeletalMesh/main")
                    .ok_or("Model has no mesh output")?
                    .id
            } else {
                index
                    .usable()
                    .filter(|r| r.meta.kind == Kind::SkeletalMesh)
                    .find_map(|m| {
                        Model::load(index, m.meta.id)
                            .ok()
                            .filter(|m| {
                                m.mesh.skeleton == r.meta.id || m.mesh.clips.contains(&r.meta.id)
                            })
                            .map(|_| m.meta.id)
                    })
                    .ok_or("No mesh references this asset")?
            };
            Model::load(index, id)?
        };
        points = model.points(None, 0., false);
        details.fields.push((
            "Geometry".into(),
            format!(
                "{} vertices · {} triangles · {} bones",
                points.len(),
                model.mesh.triangles.len(),
                model.skeleton.bones.len()
            ),
        ));
        let mut e = Actor::cube("Model preview".into());
        e.position = [0.; 3];
        let mut c = skeletal::Component::new(uuid::Uuid::new_v4());
        c.model = Some(Arc::new(model));
        e.skeletal_mesh = Some(c);
        entity = Some(e);
    } else if details.kind == "Texture" {
        let bytes = if record.is_some() {
            assets::Package::load(path)?.source
        } else {
            assets::read_bounded(path)?
        };
        let decoder = png::Decoder::new(std::io::Cursor::new(bytes));
        let reader = decoder.read_info().map_err(|e| e.to_string())?;
        details.fields.push((
            "Dimensions".into(),
            format!("{} × {}", reader.info().width, reader.info().height),
        ));
    } else if details.kind == "Font" {
        if let Some(r) = record {
            let package = assets::Package::load(path)?;
            let data = crate::font_asset::decode(&package.source, r.meta.settings.font()?)?;
            details.fields.extend([
                ("Atlas".into(), format!("{} × {}", data.width, data.height)),
                ("Glyphs".into(), data.metrics.len().to_string()),
                ("VRAM".into(), format!("{} bytes", data.vram_bytes())),
            ]);
        }
    } else if details.kind == "Audio" {
        let bytes = if record.is_some() {
            assets::Package::load(path)?.source
        } else {
            assets::read_bounded(path)?
        };
        if bytes.get(..4) == Some(b"MThd")
            || bytes.starts_with(b"pQES")
            || record.is_some_and(|r| r.meta.kind == Kind::MusicSequence)
            || crate::sequence::catalog_source(&bytes, None).is_ok()
        {
            let defaults = crate::sequence::Settings::default();
            let settings = record
                .and_then(|r| r.meta.settings.sequence().ok())
                .unwrap_or(&defaults);
            if bytes.starts_with(b"MThd") {
                let header = crate::midi::probe(&bytes)?;
                details.fields.push((
                    "MIDI source".into(),
                    format!(
                        "SMF {} · {} tracks · {} PPQN",
                        header.format, header.tracks, header.ppqn
                    ),
                ));
            } else {
                let catalog = crate::sequence::catalog_source(
                    &bytes,
                    settings.source_selection.as_ref().map(|s| s.profile),
                )?;
                details.fields.push((
                    "Sequence source profile".into(),
                    catalog.profile.map_or("MIDI", |p| p.id()).into(),
                ));
                for song in &catalog.songs {
                    details.fields.push((
                        "Source song".into(),
                        format!(
                            "ID {} · ordinal {} · {:.3} s · {} events · {} blockers · SHA-256 {}",
                            song.id,
                            song.ordinal,
                            song.duration_micros as f64 / 1_000_000.,
                            song.events,
                            song.playback_blockers.len(),
                            song.record_hash
                        ),
                    ));
                }
                if settings.source_selection.is_none() {
                    details.fields.push(("Selection required".into(), "Import this source and explicitly select its profile and song. The catalog is an analysis view; no song is auditioned automatically.".into()));
                    return Ok(details);
                }
            }
            let sequence = crate::sequence::decode_source(&bytes, settings)?;
            details.fields.push((
                "Role / event Load Mode".into(),
                format!(
                    "{:?} / {:?} (Auto resolves to Resident)",
                    settings.role, settings.load_mode
                ),
            ));
            details.fields.push((
                "Loop / voice ceiling".into(),
                format!("{:?} / {} voices", settings.loop_mode, settings.voices()),
            ));
            let bank = crate::sequence::resolve_bank(root, settings, index);
            details.fields.push((
                "SoundBank".into(),
                match &bank {
                    Ok(bank) => format!(
                        "{} ({})",
                        assets::path_string(root, &bank.path),
                        bank.meta.id
                    ),
                    Err(error) => error.clone(),
                },
            ));
            if let Ok(bank) = bank {
                if bank.meta.settings.sound_bank()?.library.is_some() {
                    let package = assets::Package::load(&bank.path)?;
                    let library = crate::soundfont_asset::decode(&package)?;
                    let coverage = crate::instrument_selection::resolve(
                        &sequence,
                        &library,
                        &settings.instrument_mappings,
                        &std::sync::atomic::AtomicBool::new(false),
                    )?;
                    details.fields.push(("Instrument coverage".into(), coverage.require_complete().err().unwrap_or_else(|| format!(
                        "All {} note events resolve to {} regions and {} samples; up to {} layers per note. Conversion determines physical voice and RAM cost.",
                        coverage.note_on_events, coverage.regions.len(), coverage.samples.len(), coverage.peak_layers_per_note))));
                } else {
                    details.fields.push((
                        "Instrument compatibility".into(),
                        bank.meta
                            .settings
                            .sound_bank()?
                            .validate_sequence(&sequence, index)
                            .err()
                            .unwrap_or_else(|| {
                                "All used program/key/velocity mappings resolve".into()
                            }),
                    ));
                }
            }
            if let Err(error) = settings.validate_playback(&sequence) {
                details.fields.push(("Playback settings".into(), error));
            }
            match crate::psx_sequence::sequence_payload(&sequence, settings, uuid::Uuid::nil()) {
                Err(error) => details.fields.push(("PSX compatibility".into(), error)),
                Ok((_, bytes)) => details.fields.push((
                    "PSX sequence payload RAM".into(),
                    format!(
                        "{} bytes; SoundBank and runtime state are additional",
                        bytes.len()
                    ),
                )),
            }
            details.fields.push((
                "Tempo map".into(),
                sequence
                    .tempo_map
                    .iter()
                    .map(|t| {
                        format!(
                            "tick {}: {:.3} BPM",
                            t.tick,
                            60_000_000. / t.micros_per_quarter as f64
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("; "),
            ));
            details.fields.push((
                "Sequence analysis".into(),
                format!(
                    "{:.3} s · {} events · peak {} logical voices",
                    sequence.duration_micros as f64 / 1_000_000.,
                    sequence.events.len(),
                    sequence.peak_polyphony
                ),
            ));
            details.fields.push((
                if sequence.source_profile.is_some() {
                    "Program/key requirements (source channel policy may be blocked)"
                } else {
                    "Programs / drum keys"
                }
                .into(),
                sequence
                    .instruments
                    .iter()
                    .map(|v| match v.drum_key {
                        Some(key) => format!("program {} / drum key {key}", v.program),
                        None => format!("program {}", v.program),
                    })
                    .collect::<Vec<_>>()
                    .join(", "),
            ));
            if let Some([start, end]) = sequence.loop_region {
                details.fields.push((
                    "MIDI loop markers".into(),
                    format!(
                        "{:.3}–{:.3} s",
                        sequence.micros_at(start) as f64 / 1_000_000.,
                        sequence.micros_at(end) as f64 / 1_000_000.
                    ),
                ));
            }
            for diagnostic in &sequence.diagnostics {
                details.fields.push((
                    if diagnostic.unsupported {
                        "Unsupported MIDI event"
                    } else {
                        "MIDI report"
                    }
                    .into(),
                    format!(
                        "Track {}, tick {}: {}",
                        diagnostic.track + 1,
                        diagnostic.tick,
                        diagnostic.message
                    ),
                ));
            }
        } else {
            let (info, _) = crate::audio_decode::decode(&bytes)?;
            details.fields.push((
                "Source audio".into(),
                format!(
                    "{} Hz · {} channel(s) · {:.2} s",
                    info.sample_rate,
                    info.channels,
                    info.frames as f32 / info.sample_rate as f32
                ),
            ));
        }
    } else if details.kind == "SoundBank" && record.is_none() {
        let bytes = assets::read_bounded(path)?;
        if crate::sf2::has_header(&bytes) {
            details
                .fields
                .extend(crate::soundfont_asset::inspect(&bytes)?);
            return Ok(details);
        }
        let source_name = assets::path_string(root, path);
        match crate::vab_import::parse(crate::vab_import::Input { bytes: &bytes, label: &source_name,
            rights: "Source ownership/license not established by inspection" }, None) {
            Ok(bank) => {
                details.fields.push(("Detected bank source".into(), format!("{} · {} programs · {} tones · {} samples · SHA-256 {}",
                    bank.profile, bank.header.program_count, bank.header.tone_count, bank.header.sample_count, assets::hash(&bytes))));
                details.fields.push(("Playback compatibility".into(), crate::bank_compat::PLAYBACK_BLOCKER.into()));
                for p in &bank.programs {
                    details.fields.push((format!("Program {}", p.id), format!("{} source tones; overlapping pairs {:?}", p.tones.len(), p.overlapping_tone_pairs)));
                    for t in &p.tones { details.fields.push((format!("Tone {}:{}", p.id, t.slot), format!("sample {}; keys {:?}; root {}; raw tuning {}; gain/pan {}/{}; native ADSR {:04x}/{:04x}; bend {:?}; mode {}; offset 0x{:x}",
                        t.sample_id, t.keys, t.root_key, t.tuning_raw, t.gain, t.pan, t.adsr_words[0], t.adsr_words[1], t.bend_down_up, t.mode, t.at.offset))); }
                }
                for s in &bank.samples { details.fields.push((format!("Sample {}", s.id), format!("{} encoded bytes; {} decoded frames; original Hz unknown; loop {:?}; SHA-256 {}",
                    s.encoded.length, s.decoded.pcm.len(), s.decoded.loop_region, s.encoded_sha256))); }
                for d in crate::vab_import::assess_current_sound_bank(&bank) { details.fields.push((format!("Compatibility: {}", d.code), format!("part {} offset 0x{:x}: {}", d.at.part, d.at.offset, d.reason))); }
            }
            Err(error) => details.fields.push(("Bank source / companion required".into(), format!("{error}. A standalone VB has no format identity; select its VH explicitly in Import. A VH requires its complete matching VB. Corrupt/unsupported pairs remain rejected."))),
        }
    } else if matches!(extension.as_str(), "epokbp" | "epokmap" | "json") {
        if let Ok(value) =
            crate::document::from_slice::<serde_json::Value>(&assets::read_bounded(path)?)
        {
            for key in ["name", "id", "version", "parent"] {
                if let Some(v) = value.get(key) {
                    details.fields.push((
                        key.into(),
                        v.as_str().map_or_else(|| v.to_string(), str::to_owned),
                    ));
                }
            }
        }
    } else if matches!(
        extension.as_str(),
        "hpp" | "cpp" | "h" | "c" | "txt" | "md" | "rs"
    ) && meta.len() <= 65536
    {
        details.text = String::from_utf8(assets::read_bounded(path)?).ok();
    }
    if let Some(entity) = entity {
        if !points.is_empty() {
            details.bounds = Some((
                std::array::from_fn(|a| points.iter().map(|p| p[a]).fold(f32::INFINITY, f32::min)),
                std::array::from_fn(|a| {
                    points
                        .iter()
                        .map(|p| p[a])
                        .fold(f32::NEG_INFINITY, f32::max)
                }),
            ));
        }
        let mut scene = Scene {
            actors: vec![entity],
            ..Default::default()
        };
        crate::texture::resolve(&mut scene, index)?;
        crate::hud::resolve_fonts(&mut scene, index)?;
        details.scene = Some(scene);
    }
    Ok(details)
}
fn raw_model(bytes: &[u8]) -> Result<Model, String> {
    let mut settings = crate::model_import::Settings::default();
    let imported = crate::model_import::decode(bytes, &mut settings)?;
    let data = imported
        .outputs
        .into_iter()
        .map(|(key, data)| (settings.outputs[&key].id, data))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mesh = data
        .values()
        .find_map(|d| {
            if let Data::SkeletalMesh(m) = d {
                Some(m.clone())
            } else {
                None
            }
        })
        .ok_or("FBX contains no mesh")?;
    let Some(Data::Skeleton(skeleton)) = data.get(&mesh.skeleton) else {
        return Err("FBX has no skeleton".into());
    };
    let materials = mesh
        .materials
        .iter()
        .map(|id| match data.get(id) {
            Some(Data::Material(m)) => Ok(m.clone()),
            _ => Err("Missing FBX material".to_string()),
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Model {
        mesh,
        skeleton: skeleton.clone(),
        materials,
        clips: vec![],
    })
}

#[cfg(test)]
thread_local! { static CONTROLS: std::cell::RefCell<std::collections::BTreeMap<String,[f32;2]>> = const {std::cell::RefCell::new(std::collections::BTreeMap::new())}; }
#[cfg(test)]
fn track(ui: &Ui, label: &str) {
    let a = ui.item_rect_min();
    let b = ui.item_rect_max();
    CONTROLS.with(|c| {
        c.borrow_mut()
            .insert(label.into(), [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5])
    });
}

#[cfg(test)]
pub fn verify_interactions(ctx: &mut imgui::Context) {
    let root = std::env::temp_dir().join(format!("epok-inspector-input-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(root.join("assets")).unwrap();
    let path = root.join("assets/Box.obj");
    std::fs::write(&path, "v -1 -1 0\nv 1 -1 0\nv 0 1 0\nf 1 2 3\n").unwrap();
    let mut e = Editor::new(root.clone());
    e.selected = Some(0);
    e.selected_asset = Some(path.clone());
    let original = e.scene.clone();
    let dirty = e.dirty;
    let camera = e.view;
    e.asset_inspector.key = Some(Key {
        path,
        modified: None,
        size: 0,
        revision: String::new(),
    });
    let details = load(&root, &root.join("assets/Box.obj"), &Index::default()).unwrap();
    e.asset_inspector
        .camera
        .frame_bounds(details.bounds.unwrap().0, details.bounds.unwrap().1);
    e.asset_inspector.details = Some(details);
    e.asset_inspector.checked = Some(Instant::now());
    fn frame(ctx: &mut imgui::Context, e: &mut Editor) {
        let ui = ctx.frame();
        unsafe {
            imgui::sys::igSetNextWindowDockID(0, imgui::sys::ImGuiCond_Always as i32);
            imgui::sys::igSetNextWindowPos(
                imgui::sys::ImVec2 { x: 10., y: 10. },
                imgui::sys::ImGuiCond_Always as i32,
                imgui::sys::ImVec2 { x: 0., y: 0. },
            );
            imgui::sys::igSetNextWindowSize(
                imgui::sys::ImVec2 { x: 380., y: 800. },
                imgui::sys::ImGuiCond_Always as i32,
            );
        }
        crate::gui::inspector(ui, e);
        ctx.render();
    }
    frame(ctx, &mut e);
    frame(ctx, &mut e);
    e.asset_inspector.model_dirty = false;
    frame(ctx, &mut e);
    assert!(
        !e.asset_inspector.model_dirty,
        "Idle Inspector must reuse its rendered model"
    );
    let point = CONTROLS.with(|c| c.borrow()["view"]);
    let yaw = e.asset_inspector.camera.yaw;
    ctx.io_mut().add_mouse_pos_event(point);
    frame(ctx, &mut e);
    ctx.io_mut()
        .add_mouse_button_event(imgui::MouseButton::Left, true);
    frame(ctx, &mut e);
    ctx.io_mut()
        .add_mouse_pos_event([point[0] + 45., point[1] + 20.]);
    frame(ctx, &mut e);
    ctx.io_mut()
        .add_mouse_button_event(imgui::MouseButton::Left, false);
    frame(ctx, &mut e);
    assert_ne!(yaw, e.asset_inspector.camera.yaw);
    assert!(e.asset_inspector.model_dirty);
    e.asset_inspector.model_dirty = false;
    let distance = e.asset_inspector.camera.distance;
    ctx.io_mut().add_mouse_wheel_event([0., 1.]);
    frame(ctx, &mut e);
    assert!(e.asset_inspector.camera.distance < distance);
    assert!(e.asset_inspector.model_dirty);
    let point = CONTROLS.with(|c| c.borrow()["resize"]);
    let height = e.asset_inspector.height;
    ctx.io_mut().add_mouse_pos_event(point);
    frame(ctx, &mut e);
    ctx.io_mut()
        .add_mouse_button_event(imgui::MouseButton::Left, true);
    frame(ctx, &mut e);
    ctx.io_mut().add_mouse_pos_event([point[0], point[1] - 60.]);
    frame(ctx, &mut e);
    ctx.io_mut()
        .add_mouse_button_event(imgui::MouseButton::Left, false);
    frame(ctx, &mut e);
    assert!(e.asset_inspector.height > height);
    assert_eq!(e.scene, original);
    assert_eq!(e.view.yaw, camera.yaw);
    assert_eq!(e.view.distance, camera.distance);
    assert_eq!(e.dirty, dirty);
    e.begin_rename(0);
    assert!(
        e.selected_asset.is_none(),
        "Actor selection restores the component Inspector"
    );
    drop(e);
    std::fs::remove_dir_all(root).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn isolated_model_load_supports_native_mesh_and_raw_fbx_without_writes() {
        let root = std::env::temp_dir().join(format!("epok-file-preview-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(root.join("assets")).unwrap();
        let mut doc = crate::mesh::Document::default();
        doc.primitive(
            "Box",
            [0.; 3],
            [2.; 3],
            1,
            doc.groups[0].id,
            doc.materials[0].id,
        );
        crate::mesh::create(&root, "assets/Box.epokasset", &doc).unwrap();
        let index = assets::scan(&root, &mut Default::default());
        let before = std::fs::read(root.join("assets/Box.epokasset")).unwrap();
        let details = load(&root, &root.join("assets/Box.epokasset"), &index).unwrap();
        assert_eq!(details.kind, "Mesh");
        assert!(details.bounds.is_some());
        assert_eq!(details.scene.unwrap().actors.len(), 1);
        assert_eq!(
            before,
            std::fs::read(root.join("assets/Box.epokasset")).unwrap()
        );
        let model = raw_model(include_bytes!("../resources/models/EpokMannequin.fbx")).unwrap();
        assert!(!model.mesh.triangles.is_empty());
        assert!(!model.points(None, 0., false).is_empty());
        assert!(raw_model(b"broken fbx").is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn aspect_fit_and_metadata_are_independent_of_source_availability() {
        let (lo, hi) = fit([10., 20.], [400., 300.], [800., 400.], 1.);
        assert_eq!(lo, [10., 70.]);
        assert_eq!(hi, [410., 270.]);
        let root =
            std::env::temp_dir().join(format!("epok-inspector-meta-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(root.join("assets")).unwrap();
        std::fs::write(
            root.join("assets/tone.wav"),
            crate::audio_import::test_wav(),
        )
        .unwrap();
        assets::commit(
            assets::prepare(
                &root,
                "assets/tone.wav",
                "assets/tone.epokasset",
                Default::default(),
                None,
                false,
            )
            .unwrap(),
        )
        .unwrap();
        std::fs::remove_file(root.join("assets/tone.wav")).unwrap();
        let details = load(
            &root,
            &root.join("assets/tone.epokasset"),
            &assets::scan(&root, &mut Default::default()),
        )
        .unwrap();
        assert_eq!(details.kind, "Audio");
        assert!(
            details
                .fields
                .iter()
                .any(|(label, _)| label == "Source audio")
        );
        assert!(load(&root, &root.join("missing"), &Index::default()).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
