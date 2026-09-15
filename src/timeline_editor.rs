//! Transactional TimelineAsset editing in the existing ImGui context.
use crate::{
    blueprint::Registry,
    timeline::{self, TimelineAsset},
    timeline_compile::{self, PreviewCache},
};
use std::{
    fs,
    path::{Path, PathBuf},
};
use uuid::Uuid;

/// The document owns exactly one TimelineAsset, standalone or embedded. Undo
/// snapshots include all effect layers and the shared timeline atomically.
#[derive(Clone, Debug, PartialEq)]
pub enum Document {
    Timeline(TimelineAsset),
    Effect(Box<crate::particle_effect::ParticleEffect>),
}
impl std::ops::Deref for Document {
    type Target = TimelineAsset;
    fn deref(&self) -> &TimelineAsset {
        match self {
            Self::Timeline(asset) => asset,
            Self::Effect(effect) => &effect.timeline,
        }
    }
}
impl std::ops::DerefMut for Document {
    fn deref_mut(&mut self) -> &mut TimelineAsset {
        match self {
            Self::Timeline(asset) => asset,
            Self::Effect(effect) => &mut effect.timeline,
        }
    }
}

#[derive(Default)]
pub struct TimelineEditor {
    pub open: bool,
    pub focused: bool,
    pub asset: Option<Document>,
    path: Option<PathBuf>,
    revision: String,
    saved: Vec<u8>,
    undo: Vec<Document>,
    redo: Vec<Document>,
    pub cache: Option<PreviewCache>,
    pub registry_error: Option<String>,
    pub catalog_error: Option<String>,
    source_error: Option<String>,
    checked_at: Option<std::time::Instant>,
    pub message: String,
    pub bindings: timeline::Bindings,
    pub effect_preview: crate::particle_effect_preview::View,
    pub scene_preview: crate::timeline_scene_preview::Preview,
    tick: i32,
    pub sequencer: crate::sequencer::View,
    pub layout: bool,
    pub font: Option<imgui::FontId>,
    details: bool,
    gesture_before: Option<Document>,
}
fn bytes(a: &Document) -> Vec<u8> {
    match a {
        Document::Timeline(asset) => serde_json::to_vec_pretty(asset),
        Document::Effect(effect) => serde_json::to_vec_pretty(effect),
    }
    .expect("serializable timeline/effect document")
}
pub(crate) fn button(ui: &imgui::Ui, label: &str) -> bool {
    let pressed = ui.button(label);
    #[cfg(test)]
    record_control(ui, label);
    pressed
}
#[cfg(test)]
pub(crate) fn record_control(ui: &imgui::Ui, label: &str) {
    CONTROLS.with(|controls| {
        let a = ui.item_rect_min();
        let b = ui.item_rect_max();
        controls
            .borrow_mut()
            .insert(label.into(), [(a[0] + b[0]) / 2., (a[1] + b[1]) / 2.]);
    });
}
fn event_key(function: &crate::reflection_schema::Function, tick: i32) -> timeline::EventKey {
    timeline::EventKey {
        id: Uuid::new_v4(),
        tick,
        arguments: function
            .parameters
            .iter()
            .map(|p| {
            let arg = if matches!(
                p.value_type,
                crate::reflection_schema::Type::ObjectRef { .. }
            ) || matches!(&p.value_type,crate::reflection_schema::Type::Record{cpp_name,..} if cpp_name=="epok::DataHandle") {
                    timeline::Argument::Slot { slot: Uuid::nil() }
                } else {
                    timeline::Argument::Literal {
                        value_type: p.value_type.clone(),
                        value: crate::script_values::default_value(&p.value_type),
                    }
                };
                (p.name.clone(), arg)
            })
            .collect(),
        extra: Default::default(),
    }
}
fn event_tracks(
    ui: &imgui::Ui,
    asset: &mut TimelineAsset,
    registry: &Registry,
    scene: &crate::scene::Scene,
    index: &crate::assets::Index,
) {
    if asset.events.is_empty() {
        return;
    }
    ui.child_window("Timeline events")
        .size([0., 210.])
        .border(true)
        .build(|| {
            let mut remove = None;
            let key_count: usize = asset.events.iter().map(|t| t.keys.len()).sum();
            for (i, track) in asset.events.iter_mut().enumerate() {
                let _id = ui.push_id(track.id.to_string());
                ui.input_text("Event track", &mut track.name).build();
                ui.text_disabled(format!("Function {}", track.function));
                if let Some(_combo) = ui.begin_combo("Event target", "Select slot") {
                    for s in &asset.slots {
                        if ui.selectable(format!("{}##{}", s.name, s.id)) {
                            track.slot = s.id;
                        }
                    }
                }
                let functions = asset
                    .slots
                    .iter()
                    .find(|s| s.id == track.slot)
                    .and_then(|s| s.class_id().and_then(|id| registry.classes.get(id)))
                    .map(|c| {
                        registry
                            .ancestry(&c.cpp_name)
                            .into_iter()
                            .rev()
                            .flat_map(|c| &c.functions)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                if let Some(_combo) = ui.begin_combo("Reassign function", "Select exposed function")
                {
                    for f in functions.iter().filter(|f| f.timeline.is_some()) {
                        if ui.selectable(format!("{}##{}", f.name, f.id)) {
                            track.function = f.id.clone();
                        }
                    }
                }
                let function = functions
                    .into_iter()
                    .find(|f| f.id == track.function || f.overrides.contains(&track.function));
                let mut delete = None;
                for (k, key) in track.keys.iter_mut().enumerate() {
                    let _key = ui.push_id(key.id.to_string());
                    crate::gui::Drag::new("Event tick")
                        .speed(1.)
                        .build(ui, &mut key.tick);
                    for (name, argument) in &mut key.arguments {
                        let _arg = ui.push_id(name);
                        match argument {
                            timeline::Argument::Literal { value_type, value } => {
                                crate::blueprint_refs::inspector(
                                    ui, name, value, value_type, scene, registry, index,
                                );
                            }
                            timeline::Argument::Slot { slot } => {
                                let label = asset
                                    .slots
                                    .iter()
                                    .find(|s| s.id == *slot)
                                    .map_or("Missing slot", |s| s.name.as_str());
                                if let Some(_combo) = ui.begin_combo(name, label) {
                                    for s in &asset.slots {
                                        if ui.selectable(format!("{}##{}", s.name, s.id)) {
                                            *slot = s.id;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if let Some(f) = function
                        && ui.small_button("Reset arguments to signature")
                    {
                        key.arguments = event_key(f, key.tick).arguments;
                    }
                    if ui.small_button("Remove event key") {
                        delete = Some(k);
                    }
                }
                if let Some(k) = delete {
                    track.keys.remove(k);
                }
                if let Some(f) = function
                    && ui.small_button("Add event key")
                    && key_count < timeline::EVENT_LIMIT
                {
                    track.keys.push(event_key(f, asset.duration_ticks / 2));
                }
                if ui.small_button("Remove event track") {
                    remove = Some(i);
                }
                ui.separator();
            }
            if let Some(i) = remove {
                asset.events.remove(i);
            }
        });
}
fn plot(ui: &imgui::Ui, compiled: &timeline_compile::Compiled, tick: i32) {
    let origin = ui.cursor_screen_pos();
    let size = [ui.content_region_avail()[0].max(1.), 64.];
    let draw = ui.get_window_draw_list();
    draw.add_rect(
        origin,
        [origin[0] + size[0], origin[1] + size[1]],
        [0.07, 0.08, 0.10, 1.],
    )
    .filled(true)
    .build();
    let x = |time: i32| origin[0] + size[0] * time as f32 / compiled.duration_ticks.max(1) as f32;
    for track in &compiled.tracks {
        let unsigned = matches!(track.value_type, crate::reflection_schema::Type::UInt32);
        let numeric = |value: i32| {
            if unsigned {
                i64::from(value as u32)
            } else {
                i64::from(value)
            }
        };
        for keys in &track.channels {
            let min = keys.iter().map(|k| numeric(k.1)).min().unwrap_or(0);
            let max = keys.iter().map(|k| numeric(k.1)).max().unwrap_or(1);
            let span = (max - min).max(1) as f64;
            let mut previous = None;
            for i in 0..=64 {
                let time = (i64::from(compiled.duration_ticks) * i / 64) as i32;
                let value = crate::timeline_curve::sample_mode(
                    keys,
                    time,
                    track.interpolation as u8,
                    unsigned,
                );
                let p = [
                    x(time),
                    origin[1] + size[1]
                        - 5.
                        - ((numeric(value) - min) as f64 / span) as f32 * (size[1] - 10.),
                ];
                if let Some(previous) = previous {
                    draw.add_line(previous, p, [0.30, 0.77, 0.95, 1.])
                        .thickness(2.)
                        .build();
                }
                previous = Some(p);
            }
        }
    }
    for (time, _) in &compiled.markers {
        draw.add_line(
            [x(*time), origin[1]],
            [x(*time), origin[1] + size[1]],
            [0.95, 0.68, 0.22, 0.7],
        )
        .build();
    }
    draw.add_line(
        [x(tick), origin[1]],
        [x(tick), origin[1] + size[1]],
        [0.9, 0.95, 1., 1.],
    )
    .thickness(2.)
    .build();
    ui.dummy(size);
}
#[cfg(test)]
thread_local! {static CONTROLS:std::cell::RefCell<std::collections::BTreeMap<String,[f32;2]>>=const{std::cell::RefCell::new(std::collections::BTreeMap::new())};}
impl TimelineEditor {
    pub(crate) fn document_path(&self) -> Option<&Path> {
        self.path.as_deref()
    }
    pub fn dirty(&self) -> bool {
        self.asset.as_ref().is_some_and(|a| bytes(a) != self.saved)
    }
    pub fn open(&mut self, path: &Path) -> Result<(), String> {
        if self.dirty() {
            return Err("Save or discard the open timeline before opening another asset".into());
        }
        let raw = fs::read(path).map_err(|e| e.to_string())?;
        let asset = if path.to_string_lossy().ends_with(".particle-effect.json") {
            Document::Effect(Box::new(crate::particle_effect::load(path)?))
        } else {
            Document::Timeline(timeline::load(path)?)
        };
        let registry_error = self.registry_error.take();
        let font = self.font;
        let layout = self.layout;
        *self = Self {
            registry_error,
            open: true,
            path: Some(path.into()),
            revision: crate::assets::hash(&raw),
            saved: bytes(&asset),
            asset: Some(asset),
            font,
            layout,
            ..Default::default()
        };
        Ok(())
    }
    pub fn create(&mut self, root: &Path) -> Result<(), String> {
        if self.dirty() {
            return Err("Save or discard the open timeline first".into());
        }
        for n in 1..=999 {
            let name = format!("Timeline{n:03}");
            if !root
                .join(format!("assets/Timelines/{name}.timeline.json"))
                .exists()
            {
                return self.open(&timeline::create(root, &name)?);
            }
        }
        Err("Choose an unused timeline filename".into())
    }
    pub fn save(&mut self) -> Result<(), String> {
        let data = bytes(self.asset.as_ref().ok_or("No timeline is open")?);
        crate::assets::atomic_write(
            self.path.as_ref().ok_or("No timeline path")?,
            &data,
            Some(&self.revision),
        )?;
        self.revision = crate::assets::hash(&data);
        self.saved = data;
        self.source_error = None;
        Ok(())
    }
    pub fn create_effect(&mut self, root: &Path) -> Result<(), String> {
        if self.dirty() {
            return Err("Save or discard the open asset first".into());
        }
        for n in 1..=999 {
            let name = format!("Effect{n:03}");
            if !root
                .join(format!("assets/Effects/{name}.particle-effect.json"))
                .exists()
            {
                return self.open(&crate::particle_effect::create(root, &name)?);
            }
        }
        Err("Choose an unused effect filename".into())
    }
    pub fn discard(&mut self) {
        let registry_error = self.registry_error.take();
        *self = Self {
            registry_error,
            ..Default::default()
        };
    }
    #[cfg(test)]
    pub fn edit(&mut self, change: impl FnOnce(&mut TimelineAsset)) {
        let Some(before) = self.asset.clone() else {
            return;
        };
        change(self.asset.as_mut().unwrap());
        self.checkpoint(before);
    }
    fn checkpoint(&mut self, before: Document) {
        if self.asset.as_ref() != Some(&before) {
            if self.undo.len() == 64 {
                self.undo.remove(0);
            }
            self.undo.push(before);
            self.redo.clear();
            self.sequencer.playing = false;
            self.tick = self
                .tick
                .min(self.asset.as_ref().map_or(0, |a| a.duration_ticks));
            if let Some(cache) = &mut self.cache {
                cache.stale = true;
            }
        }
    }
    pub fn undo(&mut self) {
        if let Some(previous) = self.undo.pop() {
            if let Some(current) = self.asset.replace(previous) {
                self.redo.push(current);
            }
            self.sequencer.playing = false;
            self.tick = self
                .tick
                .min(self.asset.as_ref().map_or(0, |a| a.duration_ticks));
            if let Some(c) = &mut self.cache {
                c.stale = true;
            }
        }
    }
    pub fn redo(&mut self) {
        if let Some(next) = self.redo.pop() {
            if let Some(current) = self.asset.replace(next) {
                self.undo.push(current);
            }
            self.sequencer.playing = false;
            self.tick = self
                .tick
                .min(self.asset.as_ref().map_or(0, |a| a.duration_ticks));
            if let Some(c) = &mut self.cache {
                c.stale = true;
            }
        }
    }
    pub fn validate(&mut self, root: &Path, registry: &Registry) {
        if let Some(error) = self
            .registry_error
            .as_ref()
            .or(self.catalog_error.as_ref())
            .or(self.source_error.as_ref())
        {
            self.message = format!("Stale timeline preview: {error}");
            if let Some(cache) = &mut self.cache {
                cache.stale = true;
            }
            if self.registry_error.is_some() {
                let _ = timeline_compile::invalidate_all(root, error);
            } else if let Some(asset) = &self.asset {
                let _ = timeline_compile::invalidate(root, asset.id, error);
            }
            return;
        }
        let Some(asset) = &self.asset else { return };
        if let Document::Effect(effect) = asset {
            let errors = effect.validate(registry);
            if !errors.is_empty() {
                self.message = errors
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("\n");
                if let Some(cache) = &mut self.cache {
                    cache.stale = true;
                }
                let _ = timeline_compile::invalidate(root, effect.timeline.id, &self.message);
                return;
            }
        }
        match timeline_compile::refresh(root, asset, registry) {
            Ok(cache) => {
                self.message = if cache.stale {
                    "Timeline has errors; last valid preview is stale".into()
                } else {
                    "Timeline validation passed".into()
                };
                self.cache = Some(cache);
            }
            Err(error) => self.message = error,
        }
    }
    pub fn draw(
        &mut self,
        ui: &imgui::Ui,
        root: &Path,
        registry: &Registry,
        scene: &crate::scene::Scene,
        index: &crate::assets::Index,
    ) {
        if matches!(self.asset, Some(Document::Effect(_))) {
            self.draw_details(ui, root, registry, scene, index);
            return;
        }
        self.focused = false;
        if !self.open {
            self.scene_preview.clear();
            return;
        }
        if self
            .checked_at
            .is_none_or(|time| time.elapsed().as_secs_f32() >= 1.)
        {
            self.checked_at = Some(std::time::Instant::now());
            self.source_error=self.path.as_ref().and_then(|path|match fs::read(path) {
                Ok(bytes) if crate::assets::hash(&bytes)==self.revision=>None,
                _=>Some("Source changed externally or is missing. Save is guarded; reload to use current disk data.".into()),
            });
        }
        let mut visible = true;
        let title = format!(
            "\u{f008}  Sequencer{}###TimelineEditor",
            if self.dirty() { " *" } else { "" }
        );
        let display = ui.io().display_size;
        let _padding = ui.push_style_var(imgui::StyleVar::WindowPadding([0., 0.]));
        let _round = ui.push_style_var(imgui::StyleVar::WindowRounding(0.));
        let _bg = ui.push_style_color(imgui::StyleColor::WindowBg, [0.115, 0.112, 0.112, 1.]);
        let _title =
            ui.push_style_color(imgui::StyleColor::TitleBgActive, [0.16, 0.155, 0.155, 1.]);
        let _font = self.font.map(|font| ui.push_font(font));
        let before = self.asset.clone();
        let mut actions = crate::sequencer::Actions::default();
        if let Some(asset) = &self.asset {
            crate::timeline_scene_preview::resolve_bindings(asset, scene, registry, &mut self.bindings);
        }
        ui.window(title)
            .opened(&mut visible)
            .position(
                [0., display[1] * 0.537],
                if self.layout {
                    imgui::Condition::Always
                } else {
                    imgui::Condition::FirstUseEver
                },
            )
            .size(
                [display[0], display[1] * 0.463],
                if self.layout {
                    imgui::Condition::Always
                } else {
                    imgui::Condition::FirstUseEver
                },
            )
            .movable(!self.layout)
            .resizable(!self.layout)
            .scroll_bar(false)
            .scrollable(false)
            .build(|| {
                ui.set_window_font_scale(if self.font.is_some() { 1. } else { 0.87 });
                self.focused = ui.is_window_focused_with_flags(
                    imgui::WindowFocusedFlags::ROOT_AND_CHILD_WINDOWS,
                );
                if let Some(a) = self.asset.as_mut() {
                    actions = self.sequencer.draw(ui, a, registry, &mut self.tick, scene, &mut self.bindings);
                }
            });
        if self.sequencer.gesture {
            if self.gesture_before.is_none() {
                self.gesture_before = before;
            }
        } else if let Some(before) = self.gesture_before.take().or(before) {
            self.checkpoint(before);
        }
        if actions.save {
            self.message = match self.save() {
                Ok(()) => "Asset saved".into(),
                Err(e) => e,
            };
        }
        if actions.undo {
            self.undo();
        }
        if actions.redo {
            self.redo();
        }
        if actions.validate {
            self.validate(root, registry);
        }
        if actions.details {
            self.details = !self.details;
        }
        if !self.message.is_empty() || self.source_error.is_some() {
            // Status is available without consuming a row in the editing canvas.
            if ui.is_window_hovered() && ui.io().key_alt {
                ui.tooltip_text(self.source_error.as_deref().unwrap_or(&self.message));
            }
        }
        if self.details {
            let focused = self.focused;
            self.draw_details(ui, root, registry, scene, index);
            self.focused |= focused;
        }
        if !visible {
            if self.dirty() {
                self.message = "Save or Discard / Reload the timeline before closing".into();
                self.details = true;
            } else {
                self.open = false;
                self.focused = false;
            }
        }
        self.refresh_scene_preview(scene, registry);
    }
    pub fn refresh_scene_preview(&mut self, scene: &crate::scene::Scene, registry: &Registry) {
        self.scene_preview.clear();
        if !self.open || !matches!(self.asset, Some(Document::Timeline(_))) {
            return;
        }
        if !self.sequencer.preview_scene {
            self.sequencer.status = "Scene preview off · Authored scene restored".into();
            return;
        }
        if let Some(error) = self.registry_error.as_ref().or(self.catalog_error.as_ref()).or(self.source_error.as_ref()) {
            self.sequencer.status = format!("Preview unavailable: {error}");
            self.sequencer.playing = false;
            return;
        }
        let asset = self.asset.as_ref().unwrap();
        self.scene_preview = crate::timeline_scene_preview::evaluate(asset, &self.bindings, scene, registry, self.tick);
        self.sequencer.status = self.scene_preview.message.clone();
    }
    fn draw_details(
        &mut self,
        ui: &imgui::Ui,
        root: &Path,
        registry: &Registry,
        scene: &crate::scene::Scene,
        index: &crate::assets::Index,
    ) {
        self.focused = false;
        if !self.open {
            self.effect_preview.clear();
            return;
        }
        if self
            .checked_at
            .is_none_or(|time| time.elapsed().as_secs_f32() >= 1.)
        {
            self.checked_at = Some(std::time::Instant::now());
            self.source_error=self.path.as_ref().and_then(|path|match fs::read(path) {
                Ok(bytes) if crate::assets::hash(&bytes)==self.revision=>None,
                _=>Some("Source changed externally or is missing. Save is guarded; reload to use current disk data.".into()),
            });
        }
        let mut visible = true;
        let title = format!(
            "{}{}###{}",
            if matches!(self.asset, Some(Document::Effect(_))) {
                "Particle Effect"
            } else {
                "Sequence Details"
            },
            if self.dirty() { " *" } else { "" },
            if matches!(self.asset, Some(Document::Effect(_))) {
                "TimelineEditor"
            } else {
                "TimelineDetails"
            }
        );
        ui.window(title).opened(&mut visible).size([900.,760.],imgui::Condition::FirstUseEver).build(||{
            self.focused=ui.is_window_focused_with_flags(imgui::WindowFocusedFlags::ROOT_AND_CHILD_WINDOWS);
            if button(ui,"Save") {self.message=match self.save(){Ok(())=>"Asset saved".into(),Err(e)=>e};}
            ui.same_line();if button(ui,"Undo"){self.undo();}
            ui.same_line();if button(ui,"Redo"){self.redo();}
            ui.same_line();if button(ui,"Validate"){self.validate(root,registry);}
            ui.same_line();if button(ui,"Discard / Reload") && let Some(path)=self.path.clone(){
                self.discard();if let Err(error)=self.open(&path){self.message=error;}
            }
            if self.focused && ui.io().key_ctrl && !ui.io().want_text_input {
                if ui.is_key_pressed(imgui::Key::Z){self.undo();}
                if ui.is_key_pressed(imgui::Key::Y){self.redo();}
            }
            ui.text_wrapped(&self.message);
            if let Some(Document::Effect(effect)) = &self.asset {
                self.effect_preview.draw(ui, effect, registry, index,
                    self.registry_error.as_deref().or(self.catalog_error.as_deref()).or(self.source_error.as_deref()));
            } else { self.effect_preview.clear(); }
            ui.child_window("Timeline and effect authoring").size([0.,0.]).build(|| {
            ui.text_disabled("Scrub or play to preview camera / transform tracks in Scene. Events run in Game.");
            let Some(before)=self.asset.clone() else{return};
            let document=self.asset.as_mut().unwrap();
            if let Document::Effect(effect)=document {crate::particle_effect_editor::layers(ui,effect,registry,scene,index);}
            let a:&mut TimelineAsset=document;
            ui.input_text("Name",&mut a.name).build();
            let mut duration=a.duration_ticks as f32/4096.;
            if crate::gui::Drag::new("Duration (seconds)").speed(0.01).build(ui,&mut duration){a.duration_ticks=(duration*4096.).round() as i32;}
            let mut repeat=a.loop_mode==timeline::LoopMode::Repeat;
            if ui.checkbox("Repeat",&mut repeat){a.loop_mode=if repeat{timeline::LoopMode::Repeat}else{timeline::LoopMode::Once};}
            ui.text_disabled(format!("Asset {} | {} / 16 tracks | {} / 64 markers",a.id,a.tracks.len()+a.events.len(),a.markers.len()));
            if self.registry_error.is_none() && self.catalog_error.is_none() && self.source_error.is_none() && let Ok(compiled)=timeline_compile::compile_index(a,registry,index){plot(ui,&compiled,self.tick);}
            if button(ui,"Add binding slot") && a.slots.len()<timeline::SLOT_LIMIT {
                let class=registry.classes.values().filter(|c|c.id!=crate::particle_effect::LAYER_CLASS_ID).find(|c|registry.properties(&c.cpp_name).iter().any(|p|p.timeline.is_some())).map(|c|c.id.clone());
                a.slots.push(timeline::Slot{id:Uuid::new_v4(),name:"Target".into(),target:crate::reflection_schema::Type::ObjectRef{class},required:true,extra:Default::default()});
            }
            ui.child_window("Timeline slots").size([0.,175.]).border(true).build(||{
                let mut remove=None;
                for (i,s) in a.slots.iter_mut().enumerate(){
                    let _id=ui.push_id(s.id.to_string());
                    ui.input_text("Slot",&mut s.name).build();ui.same_line();ui.checkbox("Required",&mut s.required);
                    if let crate::reflection_schema::Type::ObjectRef{class}=&mut s.target {
                        let selected=class.as_ref().and_then(|id|registry.classes.get(id)).map_or("Missing class",|c|c.cpp_name.as_str());
                        if let Some(_combo)=ui.begin_combo("Class",selected){
                            for c in registry.classes.values().filter(|c|c.id!=crate::particle_effect::LAYER_CLASS_ID){if ui.selectable(&c.cpp_name){*class=Some(c.id.clone());}}
                        }
                    }
                        if let Some(c)=s.class_id().and_then(|id|registry.classes.get(id))
                            && let Some(_combo)=ui.begin_combo("Add property track","Select animatable property"){
                                for p in registry.properties(&c.cpp_name).into_iter().filter(|p|p.timeline.is_some()){
                                    if ui.selectable(&p.name) && a.tracks.len()+a.events.len()<timeline::TRACK_LIMIT {
                                        a.tracks.push(timeline::Track{sections:vec![],id:Uuid::new_v4(),name:p.name.clone(),slot:s.id,property:p.id.clone(),value_type:p.value_type.clone(),priority:0,
                                            blend:timeline::Blend::Absolute,restore:timeline::Restore::LeaveFinal,interpolation:if matches!(p.value_type,crate::reflection_schema::Type::Bool|crate::reflection_schema::Type::Enum{..}){timeline::Interpolation::Step}else{timeline::Interpolation::Linear},
                                            keys:vec![timeline::Key{id:Uuid::new_v4(),tick:0,value:p.default.clone(),extra:Default::default()},timeline::Key{id:Uuid::new_v4(),tick:a.duration_ticks,value:p.default.clone(),extra:Default::default()}],extra:Default::default()});
                                    }
                                }
                        }
                        if let Some(c)=s.class_id().and_then(|id|registry.classes.get(id))
                            && let Some(_combo)=ui.begin_combo("Add event track","Select timeline function"){
                            let mut seen=std::collections::BTreeSet::new();
                            for f in registry.ancestry(&c.cpp_name).into_iter().rev().flat_map(|c|&c.functions).filter(|f|f.timeline.is_some()){
                                if seen.insert(f.name.clone()) && ui.selectable(format!("{}##{}",f.name,f.id)) && a.tracks.len()+a.events.len()<timeline::TRACK_LIMIT{
                                    a.events.push(timeline::EventTrack{id:Uuid::new_v4(),name:f.name.clone(),slot:s.id,function:f.id.clone(),keys:vec![event_key(f,0)],extra:Default::default()});
                                }
                            }
                        }
                    if matches!(s.target,crate::reflection_schema::Type::ObjectRef{..}|crate::reflection_schema::Type::ActorRef{..}|crate::reflection_schema::Type::ComponentRef{..}) {
                    let mut value = self.bindings.get(&s.id).copied().flatten().map_or(serde_json::Value::Null, |id| serde_json::json!(id));
                    if crate::blueprint_refs::inspector(ui,"Scene preview target",&mut value,&s.target,scene,registry,index) {
                        self.bindings.insert(s.id,value.as_str().and_then(|id|Uuid::parse_str(id).ok()));
                    }
                    ui.text_disabled("Test bindings are preview-only and are not saved to the reusable asset.");
                    if ui.small_button("Remove slot"){remove=Some(i);}
                    } else {ui.text_disabled("Internal effect layer binding; remove it through the layer controls.");}
                    ui.separator();
                }
                if let Some(i)=remove{a.slots.remove(i);}
            });
            ui.child_window("Timeline tracks").size([0.,200.]).border(true).build(||{
                let mut remove=None;
                for (i,t) in a.tracks.iter_mut().enumerate(){
                    let _id=ui.push_id(t.id.to_string());
                    ui.input_text("Track",&mut t.name).build();
                    ui.text_disabled(format!("Property {}",t.property));
                    let slot_name=a.slots.iter().find(|s|s.id==t.slot).map_or("Missing slot",|s|s.name.as_str());
                    if let Some(_combo)=ui.begin_combo("Binding slot",slot_name){
                        for s in &a.slots{if ui.selectable(format!("{}##{}",s.name,s.id)){t.slot=s.id;}}
                    }
                    if let Some(id)=a.slots.iter().find(|s|s.id==t.slot).and_then(|s|s.class_id())
                        && let Some(c)=registry.classes.get(id)
                        && let Some(_combo)=ui.begin_combo("Reassign property","Select animatable property"){
                        for p in registry.properties(&c.cpp_name).into_iter().filter(|p|p.timeline.is_some()){
                            if ui.selectable(&p.name){t.property=p.id.clone();t.value_type=p.value_type.clone();}
                        }
                    }
                    let mut priority=i32::from(t.priority);if crate::gui::Drag::new("Priority").speed(1.).build(ui, &mut priority){t.priority=priority.clamp(i16::MIN.into(),i16::MAX.into()) as i16;}
                    let mut restore=t.restore==timeline::Restore::RestoreInitial;if ui.checkbox("Restore initial value",&mut restore){t.restore=if restore{timeline::Restore::RestoreInitial}else{timeline::Restore::LeaveFinal};}
                    if let Some(_combo)=ui.begin_combo("Interpolation",format!("{:?}",t.interpolation)){
                        for mode in [timeline::Interpolation::Linear,timeline::Interpolation::Step,timeline::Interpolation::Smoothstep,timeline::Interpolation::EaseIn,timeline::Interpolation::EaseOut]{
                            if ui.selectable(format!("{mode:?}")){t.interpolation=mode;}
                        }
                    }
                    let mut additive=t.blend==timeline::Blend::Additive;
                    if ui.checkbox("Additive",&mut additive){t.blend=if additive{timeline::Blend::Additive}else{timeline::Blend::Absolute};}
                    if t.sections.is_empty() {
                    let mut delete=None;
                    for (k,key) in t.keys.iter_mut().enumerate(){
                        let _key=ui.push_id(key.id.to_string());
                        let mut time=key.tick as f32/4096.;
                        ui.set_next_item_width(110.);
                        if crate::gui::Drag::new("Time").speed(0.01).build(ui,&mut time){key.tick=(time*4096.).round() as i32;}
                        ui.same_line();
                        ui.set_next_item_width(160.);
                        crate::script_values::inspector(ui,"Value",&mut key.value,&t.value_type);
                        if !crate::script_values::valid(&key.value,&t.value_type) && ui.small_button("Reset incompatible value") {key.value=crate::script_values::default_value(&t.value_type);}
                        ui.same_line();if ui.small_button("Remove key"){delete=Some(k);}
                    }
                    if let Some(k)=delete{t.keys.remove(k);}
                    if ui.small_button("Add key") && t.keys.len()<timeline::KEY_LIMIT{t.keys.push(timeline::Key{id:Uuid::new_v4(),tick:a.duration_ticks/2,value:crate::script_values::default_value(&t.value_type),extra:Default::default()});}
                    } else {
                        ui.text_disabled(format!("{} section(s), {} independent channel(s). Edit them in the Sequencer section panel.",t.sections.len(),t.sections.iter().map(|s|s.channels.len()).sum::<usize>()));
                    }
                    ui.same_line();if ui.small_button("Remove track"){remove=Some(i);}
                    ui.separator();
                }
                if let Some(i)=remove{a.tracks.remove(i);}
            });
            event_tracks(ui,a,registry,scene,index);
            if button(ui,"Add marker") && a.markers.len()<timeline::MARKER_LIMIT{a.markers.push(timeline::Marker{id:Uuid::new_v4(),name:"Impact".into(),tick:a.duration_ticks/2,extra:Default::default()});}
            let mut delete=None;
            for (i,m) in a.markers.iter_mut().enumerate(){
                let _id=ui.push_id(m.id.to_string());
                ui.set_next_item_width(220.);
                ui.input_text("Marker",&mut m.name).build();ui.same_line();
                ui.set_next_item_width(110.);
                let mut seconds=m.tick as f32/4096.;if crate::gui::Drag::new("Time").speed(0.01).build(ui,&mut seconds){m.tick=(seconds*4096.).round() as i32;}
                ui.same_line();if ui.small_button("Remove marker"){delete=Some(i);}
            }
            if let Some(i)=delete{a.markers.remove(i);}
            let duration=a.duration_ticks.max(1);
            self.checkpoint(before);
            let Some(a)=self.asset.as_ref() else{return};
            let compiled=if let Some(error)=self.registry_error.as_ref().or(self.catalog_error.as_ref()).or(self.source_error.as_ref()) {
                Err(vec![timeline::Diagnostic{asset:a.id,item:a.id,required:true,message:format!("Stale preview: {error}")}])
            } else if let Document::Effect(effect)=a {
                let errors=effect.validate(registry);
                if errors.is_empty(){timeline_compile::compile_index(a,registry,index)}else{Err(errors)}
            } else {timeline_compile::compile_index(a,registry,index)};
            let signature=compiled.as_ref().ok().map(|c|&c.signature);
            if let Some(cache)=&mut self.cache{cache.stale=signature.is_none_or(|s|cache.compiled.as_ref().is_none_or(|c|&c.signature!=s));}
            ui.slider("Preview tick",0,duration,&mut self.tick);
            match &compiled {
                Ok(c)=>for (id,values) in c.sample_values(self.tick){
                    let Some(t)=c.tracks.iter().find(|t|t.id==id)else{continue;};
                    let values=values[..t.channels.len()].iter().map(|value|match &t.value_type{
                        crate::reflection_schema::Type::Fixed|crate::reflection_schema::Type::Vector{..}=>format!("{:.4}",f64::from(*value)/4096.),
                        crate::reflection_schema::Type::UInt32=>(*value as u32).to_string(),
                        crate::reflection_schema::Type::Bool=>(*value!=0).to_string(),
                        _=>value.to_string(),
                    }).collect::<Vec<_>>().join(", ");
                    ui.text(format!("{id}: {values}"));
                },
                Err(errors)=>for error in errors{ui.text_colored([1.,0.5,0.3,1.],error.to_string());},
            }
            let mut external=(**a).clone();external.slots.retain(|s|matches!(s.target,crate::reflection_schema::Type::ObjectRef{..}));
            for d in external.validate_bindings(&self.bindings,scene,registry){ui.text_wrapped(format!("{}: {d}",if d.required{"Required binding error"}else{"Optional / inactive binding"}));}
            if self.cache.as_ref().is_some_and(|c|c.stale){ui.text_colored([1.,0.7,0.3,1.],"Cached preview is stale; cooking requires fresh validation.");}
            });
        });
        if !visible {
            if !matches!(self.asset, Some(Document::Effect(_))) {
                self.details = false;
                return;
            }
            if self.dirty() {
                self.message = "Save or Discard / Reload the timeline before closing".into();
            } else {
                self.open = false;
                self.focused = false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "Owns an ImGui context; run explicitly and serially"]
    fn sequencer_preserves_layout_and_previews_camera_without_editing_the_map() {
        let root = std::env::temp_dir().join(format!("epok-camera-ui-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let (asset, registry, scene) = crate::timeline_scene_preview::tests::fixture();
        let path = root.join("Camera.timeline.json");
        fs::write(&path, serde_json::to_vec_pretty(&asset).unwrap()).unwrap();
        let saved_scene = serde_json::to_vec(&scene).unwrap();
        let mut editor = TimelineEditor::default();
        editor.open(&path).unwrap();
        assert!(!editor.layout, "Opening an asset must not activate the exclusive layout");
        let mut ctx = crate::gui::tests::imgui_context();
        ctx.set_ini_filename(None);
        ctx.io_mut().display_size = [1440., 1100.];
        ctx.io_mut().delta_time = 1. / 60.;
        ctx.fonts().add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
        ctx.fonts().build_rgba32_texture();
        let frame = |ctx: &mut imgui::Context, editor: &mut TimelineEditor| {
            editor.draw(ctx.frame(), &root, &registry, &scene, &Default::default());
            ctx.render();
        };
        let click = |ctx: &mut imgui::Context, editor: &mut TimelineEditor, label: &str| {
            let point = CONTROLS.with(|c| c.borrow()[label]);
            ctx.io_mut().add_mouse_pos_event(point);
            frame(ctx, editor);
            ctx.io_mut().add_mouse_button_event(imgui::MouseButton::Left, true);
            frame(ctx, editor);
            ctx.io_mut().add_mouse_button_event(imgui::MouseButton::Left, false);
            frame(ctx, editor);
        };
        frame(&mut ctx, &mut editor);
        frame(&mut ctx, &mut editor);
        for lane in 0..3 {
            assert!(CONTROLS.with(|c| c.borrow().contains_key(&format!("Value:{}:Some({lane})", asset.tracks[0].id))));
        }
        click(&mut ctx, &mut editor, "End");
        let preview = editor.scene_preview.scene.as_ref().unwrap();
        assert_eq!(preview.actors[0].position, [4., 5., -2.]);
        assert_eq!(preview.actors[0].rotation, [0., 90., 0.]);
        assert_eq!(preview.actors[0].camera_fov, 60.);
        click(&mut ctx, &mut editor, "Play");
        frame(&mut ctx, &mut editor);
        assert!(editor.sequencer.playing);
        assert!(editor.tick > 0 && editor.tick < asset.duration_ticks);
        assert_ne!(editor.scene_preview.scene.as_ref().unwrap().actors[0].position, [4., 5., -2.]);
        click(&mut ctx, &mut editor, "Play");
        assert!(!editor.sequencer.playing);
        click(&mut ctx, &mut editor, &format!("Key:1:{}:Some(0)", asset.tracks[0].id));
        assert_eq!(editor.tick, asset.duration_ticks);
        let point = CONTROLS.with(|c| c.borrow()[&format!("Value:{}:Some(0)", asset.tracks[0].id)]);
        ctx.io_mut().add_mouse_pos_event(point);
        frame(&mut ctx, &mut editor);
        ctx.io_mut().add_mouse_button_event(imgui::MouseButton::Left, true);
        frame(&mut ctx, &mut editor);
        ctx.io_mut().add_mouse_pos_event([point[0] + 30., point[1]]);
        frame(&mut ctx, &mut editor);
        ctx.io_mut().add_mouse_pos_event([point[0] + 60., point[1]]);
        frame(&mut ctx, &mut editor);
        ctx.io_mut().add_mouse_button_event(imgui::MouseButton::Left, false);
        frame(&mut ctx, &mut editor);
        assert!(editor.dirty(), "The channel value must be editable in the track row");
        assert_ne!(editor.scene_preview.scene.as_ref().unwrap().actors[0].position[0], 4.);
        click(&mut ctx, &mut editor, "Undo");
        assert_eq!(editor.scene_preview.scene.as_ref().unwrap().actors[0].position[0], 4.);
        click(&mut ctx, &mut editor, "Scene preview");
        assert!(editor.scene_preview.scene.is_none());
        assert_eq!(serde_json::to_vec(&scene).unwrap(), saved_scene);
        assert!(!editor.dirty());
        editor.open = false;
        frame(&mut ctx, &mut editor);
        assert!(editor.scene_preview.scene.is_none());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[ignore = "Owns an ImGui context; run explicitly and serially"]
    fn effect_controls_undo_the_whole_document_and_save_embedded_timeline() {
        let root = std::env::temp_dir().join(format!("epok-effect-ui-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let mut editor = TimelineEditor::default();
        editor.create_effect(&root).unwrap();
        let path = editor.path.clone().unwrap();
        let original = crate::particle_effect::load(&path).unwrap();
        let mut ctx = crate::gui::tests::imgui_context();
        ctx.set_ini_filename(None);
        ctx.io_mut().display_size = [1280., 1000.];
        ctx.io_mut().delta_time = 1. / 60.;
        ctx.fonts()
            .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
        ctx.fonts().build_rgba32_texture();
        let frame = |ctx: &mut imgui::Context, editor: &mut TimelineEditor| {
            editor.draw(
                ctx.frame(),
                &root,
                &Registry::new(),
                &crate::scene::Scene::default(),
                &Default::default(),
            );
            ctx.render();
        };
        let click = |ctx: &mut imgui::Context, editor: &mut TimelineEditor, label: &str| {
            let point = CONTROLS.with(|c| c.borrow()[label]);
            ctx.io_mut().add_mouse_pos_event(point);
            frame(ctx, editor);
            ctx.io_mut()
                .add_mouse_button_event(imgui::MouseButton::Left, true);
            frame(ctx, editor);
            ctx.io_mut()
                .add_mouse_button_event(imgui::MouseButton::Left, false);
            frame(ctx, editor);
        };
        frame(&mut ctx, &mut editor);
        frame(&mut ctx, &mut editor);
        click(&mut ctx, &mut editor, "Add Fire layer");
        let Document::Effect(added) = editor.asset.as_ref().unwrap() else {
            panic!("Effect document required")
        };
        let added = added.clone();
        assert_eq!(added.layers.len(), 2);
        assert_eq!(added.timeline.tracks.len(), 1);
        click(&mut ctx, &mut editor, "Undo");
        assert!(!editor.dirty());
        assert_eq!(
            editor.asset,
            Some(Document::Effect(Box::new(original.clone())))
        );
        click(&mut ctx, &mut editor, "Redo");
        assert_eq!(editor.asset, Some(Document::Effect(added.clone())));
        click(&mut ctx, &mut editor, "Save");
        assert!(!editor.dirty());
        assert_eq!(crate::particle_effect::load(&path).unwrap(), *added);
        // External source edits must not be overwritten by effect document save.
        fs::write(&path, serde_json::to_vec_pretty(&original).unwrap()).unwrap();
        assert!(editor.save().is_err());
        assert_eq!(crate::particle_effect::load(&path).unwrap(), original);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    #[ignore = "Owns an ImGui context; run explicitly and serially"]
    fn timeline_controls_add_undo_redo_validate_and_save() {
        let root = std::env::temp_dir().join(format!("epok-timeline-ui-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let mut editor = TimelineEditor::default();
        editor.create(&root).unwrap();
        let mut ctx = crate::gui::tests::imgui_context();
        ctx.set_ini_filename(None);
        ctx.io_mut().display_size = [1280., 1000.];
        ctx.io_mut().delta_time = 1. / 60.;
        ctx.fonts()
            .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
        ctx.fonts().build_rgba32_texture();
        let registry = Registry::new();
        let scene = crate::scene::Scene::default();
        let frame = |ctx: &mut imgui::Context, editor: &mut TimelineEditor| {
            editor.draw(ctx.frame(), &root, &registry, &scene, &Default::default());
            ctx.render();
        };
        let click = |ctx: &mut imgui::Context, editor: &mut TimelineEditor, label: &str| {
            let point = CONTROLS.with(|c| c.borrow()[label]);
            ctx.io_mut().add_mouse_pos_event(point);
            frame(ctx, editor);
            ctx.io_mut()
                .add_mouse_button_event(imgui::MouseButton::Left, true);
            frame(ctx, editor);
            ctx.io_mut()
                .add_mouse_button_event(imgui::MouseButton::Left, false);
            frame(ctx, editor);
        };
        frame(&mut ctx, &mut editor);
        frame(&mut ctx, &mut editor);
        click(&mut ctx, &mut editor, "Add marker");
        assert_eq!(editor.asset.as_ref().unwrap().markers.len(), 1);
        let id = editor.asset.as_ref().unwrap().markers[0].id;
        click(&mut ctx, &mut editor, "Undo");
        assert!(!editor.dirty());
        click(&mut ctx, &mut editor, "Redo");
        assert_eq!(editor.asset.as_ref().unwrap().markers[0].id, id);
        click(&mut ctx, &mut editor, "Validate");
        assert!(!editor.cache.as_ref().unwrap().stale);
        click(&mut ctx, &mut editor, "Save");
        assert!(!editor.dirty());
        assert_eq!(
            timeline::load(editor.path.as_ref().unwrap())
                .unwrap()
                .markers[0]
                .id,
            id
        );
        fs::remove_dir_all(root).unwrap();
    }
}
