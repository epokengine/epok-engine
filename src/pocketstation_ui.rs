//! The PocketStation window: write an app's code, draw its sprites and
//! icons, build it and play it on the emulated PocketStation beside the
//! editor, with the console, live variables, a debugger and a PlayStation
//! link tester.
use crate::{
    editor::Editor,
    pocketstation::{self, Built, Language, PocketApp, SpriteDoc, Template},
};
use epok_pocketstation::{self as ps, card, file, hw::button, link, plua};
use imgui::{
    Condition, InputTextCallbackHandler, InputTextMultilineCallback, Key, MouseButton,
    TextCallbackData, TextureId, Ui,
};
use std::{cell::Cell, time::Instant};
use uuid::Uuid;

thread_local! {
    static CODE_FONT: Cell<Option<imgui::FontId>> = const { Cell::new(None) };
}

/// Registers the monospaced font used by the code editor.
pub fn set_code_font(font: imgui::FontId) {
    CODE_FONT.with(|f| f.set(Some(font)));
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tab {
    #[default]
    Code,
    Sprites,
    Icons,
    Settings,
    Card,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Panel {
    #[default]
    Console,
    Problems,
    Variables,
    Debug,
    Link,
}

pub struct State {
    pub open: bool,
    pub app: Option<Uuid>,
    pub doc: Option<PocketApp>,
    dirty_since: Option<Instant>,
    pub tab: Tab,
    select_tab: Option<Tab>,
    pub panel: Panel,
    select_panel: Option<Panel>,
    file: usize,
    sprite: usize,
    frame: usize,
    icon_frame: usize,
    color_frame: usize,
    color_index: u8,
    paint: Option<bool>,
    pub built: Option<Built>,
    pub problems: Vec<plua::Diagnostic>,
    pub status: String,
    pub machine: Option<ps::Machine>,
    /// File name and app id of the program the emulator runs.
    running: Option<(Uuid, String)>,
    pub paused: bool,
    last_tick: Option<Instant>,
    mouse_buttons: u8,
    pub capture: bool,
    texture: Option<TextureId>,
    pixels: Option<Vec<u8>>,
    card_hash: u64,
    card_checked: Option<Instant>,
    cursor: usize,
    goto_line: Option<u32>,
    focus_code: bool,
    new_popup: bool,
    new_title: String,
    new_template: usize,
    speaker: u32,
    audio_since: u64,
    pub error: Option<String>,
    link_text: String,
    link_reply: String,
    link_index: i32,
    link_value: i32,
    confirm_format: bool,
    new_sprite_name: String,
    /// Docked automatically because a game in Play talks to the PocketStation.
    docked_by_play: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            open: false,
            app: None,
            doc: None,
            dirty_since: None,
            tab: Tab::Code,
            select_tab: None,
            panel: Panel::Console,
            select_panel: None,
            file: 0,
            sprite: 0,
            frame: 0,
            icon_frame: 0,
            color_frame: 0,
            color_index: 1,
            paint: None,
            built: None,
            problems: Vec::new(),
            status: String::new(),
            machine: None,
            running: None,
            paused: false,
            last_tick: None,
            mouse_buttons: 0,
            capture: false,
            texture: None,
            pixels: None,
            card_hash: 0,
            card_checked: None,
            cursor: 0,
            goto_line: None,
            focus_code: false,
            new_popup: false,
            new_title: "My Pocket Game".into(),
            new_template: 0,
            speaker: 0,
            audio_since: 0,
            error: None,
            link_text: "1 2 3".into(),
            link_reply: String::new(),
            link_index: 1,
            link_value: 0,
            confirm_format: false,
            new_sprite_name: "sprite".into(),
            docked_by_play: false,
        }
    }
}

/// Answers a game's PocketStation requests during Play (every editor frame,
/// whether or not the window is open). The first request docks the emulated
/// PocketStation into slot 1 and opens its window.
pub fn service_link(e: &mut Editor) {
    let bridge = e
        .job
        .as_ref()
        .and_then(|job| job.bridge.as_ref())
        .map(|b| b.state.clone());
    let Some(state) = bridge else {
        if e.pocketstation.docked_by_play {
            e.pocketstation.docked_by_play = false;
            if let Some(machine) = &mut e.pocketstation.machine {
                machine.set_docked(false);
            }
            save_card(e);
        }
        return;
    };
    let requests: Vec<crate::bridge::PocketRequest> =
        state.lock().unwrap().pocket_requests.drain(..).collect();
    if requests.is_empty() {
        if e.pocketstation.docked_by_play && !e.pocketstation.open {
            e.pocketstation.tick();
        }
        return;
    }
    ensure_machine(e);
    if !e.pocketstation.docked_by_play {
        e.pocketstation.docked_by_play = true;
        e.pocketstation.open = true;
        e.pocketstation.select_panel = Some(Panel::Link);
        e.log("The game is talking to the PocketStation: the editor's PocketStation is in memory card slot 1.");
    }
    let machine = e.pocketstation.machine.as_mut().unwrap();
    let replies = pocketstation::answer(machine, requests);
    state.lock().unwrap().pocket_replies.extend(replies);
}

/// PocketStation app assets: (id, display name).
pub fn apps(e: &Editor) -> Vec<(Uuid, String)> {
    let mut out: Vec<(Uuid, String)> = e
        .assets
        .index
        .assets
        .iter()
        .filter_map(|(id, records)| {
            let record = records.first()?;
            (record.meta.kind == crate::assets::Kind::PocketStationApp).then(|| {
                let name = record
                    .path
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default();
                let name = match name.rsplit_once('-') {
                    Some((stem, hash)) if hash.len() == 8 => stem.to_string(),
                    _ => name,
                };
                (*id, name)
            })
        })
        .collect();
    out.sort_by(|a, b| a.1.cmp(&b.1));
    out
}

pub fn open(e: &mut Editor) {
    e.pocketstation.open = true;
    ensure_machine(e);
    if e.pocketstation.app.is_none() {
        if let Some((id, _)) = apps(e).first().cloned() {
            open_app(e, id);
        }
    }
}

fn ensure_machine(e: &mut Editor) {
    if e.pocketstation.machine.is_none() {
        let mut machine = ps::Machine::new(pocketstation::load_card(&e.root));
        machine.set_date_time(pocketstation::now());
        e.pocketstation.card_hash = hash(&machine.hw.flash);
        e.pocketstation.machine = Some(machine);
        e.pocketstation.last_tick = None;
    }
}

pub fn open_app(e: &mut Editor, id: Uuid) {
    store(e);
    let record = match e.assets.index.resolve(id) {
        Ok(record) => record.clone(),
        Err(error) => {
            e.pocketstation.error = Some(error);
            return;
        }
    };
    match pocketstation::document(&record) {
        Ok(doc) => {
            let s = &mut e.pocketstation;
            s.app = Some(id);
            s.doc = Some(doc);
            s.file = 0;
            s.sprite = 0;
            s.frame = 0;
            s.built = None;
            s.problems.clear();
            s.error = None;
            s.open = true;
        }
        Err(error) => e.pocketstation.error = Some(error),
    }
    ensure_machine(e);
}

/// Creates an app from a template and opens it.
pub fn create(e: &mut Editor, template: Template, title: &str) -> Result<Uuid, String> {
    let app = PocketApp::template(template, title);
    let path = pocketstation::fresh_path(title);
    let id = pocketstation::create(&e.root, &path, &app)?;
    e.assets.index = crate::assets::scan(&e.root, &mut Default::default());
    e.assets.refresh();
    open_app(e, id);
    e.log(format!(
        "Created PocketStation app {path}. Press Build & Run (F5) to play it."
    ));
    Ok(id)
}

/// Writes pending edits to the asset.
pub fn store(e: &mut Editor) {
    let (Some(id), Some(doc)) = (e.pocketstation.app, e.pocketstation.doc.clone()) else {
        return;
    };
    if e.pocketstation.dirty_since.is_none() {
        return;
    }
    let record = match e.assets.index.resolve(id) {
        Ok(record) => record.clone(),
        Err(error) => {
            e.pocketstation.error = Some(error);
            return;
        }
    };
    match pocketstation::save(&record, &doc) {
        Ok(_) => {
            e.pocketstation.dirty_since = None;
            e.assets.index = crate::assets::scan(&e.root, &mut Default::default());
            e.assets.refresh();
        }
        Err(error) => e.pocketstation.error = Some(error),
    }
}

/// Saves the emulated card when its flash changed.
pub fn save_card(e: &mut Editor) {
    let root = e.root.clone();
    let s = &mut e.pocketstation;
    let Some(machine) = &s.machine else { return };
    let h = hash(&machine.hw.flash);
    if h != s.card_hash {
        match pocketstation::save_card(&root, &machine.hw.flash) {
            Ok(()) => s.card_hash = h,
            Err(error) => s.error = Some(format!("PocketStation card: {error}")),
        }
    }
}

fn hash(bytes: &[u8]) -> u64 {
    bytes.chunks(8).fold(0xcbf29ce484222325u64, |h, c| {
        let mut w = [0u8; 8];
        w[..c.len()].copy_from_slice(c);
        (h ^ u64::from_le_bytes(w)).wrapping_mul(0x100000001b3)
    })
}

fn mark_dirty(s: &mut State) {
    if s.dirty_since.is_none() {
        s.dirty_since = Some(Instant::now());
    }
}

/// Builds the open app, installs it on the emulated card and starts it.
pub fn build_and_run(e: &mut Editor) {
    store(e);
    ensure_machine(e);
    let s = &mut e.pocketstation;
    let (Some(doc), Some(id)) = (s.doc.clone(), s.app) else {
        return;
    };
    match pocketstation::build_app(&doc) {
        Ok(built) => {
            s.problems = built.warnings.clone();
            let machine = s.machine.as_mut().unwrap();
            match card::replace(&mut machine.hw.flash, &doc.file_name, &built.bytes) {
                Ok(block) => {
                    if let Err(error) = machine.launch(block, 0) {
                        s.error = Some(error);
                    }
                    s.running = Some((id, doc.file_name.clone()));
                    s.paused = false;
                    s.last_tick = None;
                    s.capture = true;
                    machine.tty.clear();
                    s.status = format!(
                        "Built {}: {} block{}, {} bytes of RAM for variables, {} for the stack",
                        doc.title,
                        built.blocks,
                        if built.blocks == 1 { "" } else { "s" },
                        built.program.ram_end.saturating_sub(0x200),
                        built.program.stack_bytes()
                    );
                    s.built = Some(built);
                    s.error = None;
                }
                Err(error) => {
                    s.error = Some(format!("Cannot install on the PocketStation card: {error}"))
                }
            }
            let message = s.status.clone();
            e.log(message);
            save_card(e);
        }
        Err(problems) => {
            let first = problems.first().cloned();
            s.problems = problems;
            s.select_panel = Some(Panel::Problems);
            s.status = format!(
                "Build failed with {} problem{}",
                s.problems.len(),
                if s.problems.len() == 1 { "" } else { "s" }
            );
            if let Some(p) = first {
                jump_to(s, &p.file, p.line);
            }
        }
    }
}

fn jump_to(s: &mut State, file_name: &str, line: u32) {
    if let Some(doc) = &s.doc {
        if let Some(index) = doc.files.iter().position(|f| f.name == file_name) {
            s.file = index;
        }
    }
    s.select_tab = Some(Tab::Code);
    s.goto_line = Some(line.max(1));
    s.focus_code = true;
}

impl State {
    /// Advances the emulator to the present.
    fn tick(&mut self) {
        let now = Instant::now();
        let elapsed = self
            .last_tick
            .map(|t| now.duration_since(t).as_secs_f64())
            .unwrap_or(0.0);
        self.last_tick = Some(now);
        let Some(machine) = &mut self.machine else {
            return;
        };
        machine.set_buttons(self.mouse_buttons);
        if !self.paused {
            let dt = elapsed.min(0.1);
            if dt > 0.0 {
                if let ps::Stop::Breakpoint(_) = machine.run(dt) {
                    self.paused = true;
                }
            }
        }
        let events = machine.hw.audio_events.len() as u32;
        self.speaker = if events != self.speaker && machine.hw.speaker_on() {
            events
        } else {
            0
        };
        let mut since = self.audio_since;
        let _ = machine.take_audio(8000, &mut since);
        self.audio_since = since;
        self.pixels = Some(pocketstation::screen_rgba(&machine.screen()));
    }

    /// Plays the running program for `seconds` with simulated input: Fire to
    /// start, then Left/Right (screenshot automation).
    pub fn autoplay(&mut self, seconds: f64) {
        let Some(machine) = &mut self.machine else {
            return;
        };
        machine.set_buttons(button::FIRE);
        machine.run(0.2);
        machine.set_buttons(0);
        machine.run(0.2);
        let mut t = 0.0;
        while t < seconds {
            machine.set_buttons(if (t as u32) % 2 == 0 {
                button::LEFT
            } else {
                button::RIGHT
            });
            machine.run(0.25);
            t += 0.25;
        }
        machine.set_buttons(0);
        self.last_tick = None;
    }

    /// Opens the New App dialog.
    pub fn new_app(&mut self) {
        self.new_popup = true;
    }

    /// Selects an editor tab (`code`, `sprites`, `icons`, `settings`, `card`)
    /// or a panel (`console`, `problems`, `variables`, `debug`, `link`).
    pub fn show_tab(&mut self, name: &str) {
        match name {
            "code" => self.select_tab = Some(Tab::Code),
            "sprites" => self.select_tab = Some(Tab::Sprites),
            "icons" => self.select_tab = Some(Tab::Icons),
            "settings" => self.select_tab = Some(Tab::Settings),
            "card" => self.select_tab = Some(Tab::Card),
            "console" => self.select_panel = Some(Panel::Console),
            "problems" => self.select_panel = Some(Panel::Problems),
            "variables" => self.select_panel = Some(Panel::Variables),
            "debug" => self.select_panel = Some(Panel::Debug),
            "link" => self.select_panel = Some(Panel::Link),
            _ => {}
        }
    }

    pub fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        renderer: &mut imgui_wgpu::Renderer,
    ) {
        let Some(pixels) = self.pixels.take() else {
            return;
        };
        if self.texture.is_none() {
            let texture = imgui_wgpu::Texture::new(
                device,
                renderer,
                imgui_wgpu::TextureConfig {
                    size: wgpu::Extent3d {
                        width: 32,
                        height: 32,
                        depth_or_array_layers: 1,
                    },
                    format: Some(wgpu::TextureFormat::Rgba8UnormSrgb),
                    sampler_desc: wgpu::SamplerDescriptor {
                        mag_filter: wgpu::FilterMode::Nearest,
                        min_filter: wgpu::FilterMode::Nearest,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            );
            self.texture = Some(renderer.textures.insert(texture));
        }
        if let Some(texture) = self.texture.and_then(|id| renderer.textures.get(id)) {
            texture.write(queue, &pixels, 32, 32);
        }
    }

    /// The runtime error the program stopped on, if any: (line, message).
    fn runtime_error(&self) -> Option<(u32, String)> {
        let machine = self.machine.as_ref()?;
        let built = self.built.as_ref()?;
        let ps::State::Program(_) = machine.state else {
            return None;
        };
        let halt = *built.program.assembly.labels.get("rt_halt")?;
        if machine.cpu.pc() != halt {
            return None;
        }
        let line = machine.hw.ram_word(0x230);
        let code = machine.hw.ram_word(0x234);
        Some((line, plua::runtime::error_message(code).to_string()))
    }

    fn running_this_app(&self) -> bool {
        matches!(
            (&self.running, self.app, self.machine.as_ref().map(|m| &m.state)),
            (Some((id, _)), Some(app), Some(ps::State::Program(_))) if *id == app
        )
    }
}

struct Cursor<'a> {
    cursor: &'a mut usize,
    goto: &'a mut Option<usize>,
}

impl InputTextCallbackHandler for Cursor<'_> {
    fn on_always(&mut self, mut data: TextCallbackData) {
        if let Some(at) = self.goto.take() {
            let at = at.min(data.str().len());
            data.set_cursor_pos(at);
            data.clear_selection();
        }
        *self.cursor = data.cursor_pos();
    }
}

fn line_offset(text: &str, line: u32) -> usize {
    let mut current = 1;
    for (i, c) in text.char_indices() {
        if current == line {
            return i;
        }
        if c == '\n' {
            current += 1;
        }
    }
    text.len()
}

fn line_column(text: &str, at: usize) -> (usize, usize) {
    let before = &text[..at.min(text.len())];
    let line = before.matches('\n').count() + 1;
    let column = before
        .rsplit('\n')
        .next()
        .map(|l| l.chars().count())
        .unwrap_or(0)
        + 1;
    (line, column)
}

pub fn window(ui: &Ui, e: &mut Editor) {
    if !e.pocketstation.open {
        return;
    }
    let mut open = true;
    let mut build = false;
    let mut create_request: Option<(Template, String)> = None;
    let mut switch_to: Option<Uuid> = None;
    let app_list = apps(e);
    e.pocketstation.tick();
    if e.pocketstation
        .dirty_since
        .is_some_and(|t| t.elapsed().as_secs_f32() > 1.5)
        && !ui.is_any_item_active()
    {
        store(e);
    }
    if e.pocketstation
        .card_checked
        .is_none_or(|t| t.elapsed().as_secs_f32() > 3.0)
    {
        e.pocketstation.card_checked = Some(Instant::now());
        save_card(e);
    }
    let s = &mut e.pocketstation;
    ui.window("\u{f10b} PocketStation###PocketStation")
        .opened(&mut open)
        .size([1180., 760.], Condition::FirstUseEver)
        .size_constraints([760., 480.], [4000., 4000.])
        .build(|| {
            let focused =
                ui.is_window_focused_with_flags(imgui::WindowFocusedFlags::ROOT_AND_CHILD_WINDOWS);
            if focused && ui.is_key_pressed(Key::F5) {
                build = true;
            }
            toolbar(ui, s, &app_list, &mut build, &mut switch_to);
            if let Some(error) = s.error.clone() {
                ui.text_colored([1.0, 0.45, 0.4, 1.0], &error);
                ui.same_line();
                if ui.small_button("Dismiss##ps-error") {
                    s.error = None;
                }
            }
            new_app_popup(ui, s, &mut create_request);
            if s.doc.is_none() {
                empty_state(ui, s, &app_list, &mut switch_to);
                return;
            }
            let avail = ui.content_region_avail();
            let right = (avail[0] * 0.36).clamp(330., 460.);
            ui.child_window("ps-left")
                .size([avail[0] - right - 8., 0.])
                .build(|| {
                    editor_tabs(ui, s);
                });
            ui.same_line();
            ui.child_window("ps-right").size([0., 0.]).build(|| {
                device(ui, s, focused);
                bottom_panels(ui, s, &mut switch_to);
            });
        });
    if let Some(id) = switch_to {
        open_app(e, id);
    }
    if let Some((template, title)) = create_request {
        if let Err(error) = create(e, template, &title) {
            e.pocketstation.error = Some(error);
        }
    }
    if build {
        build_and_run(e);
    }
    if !open {
        store(e);
        save_card(e);
        e.pocketstation.open = false;
    }
}

fn toolbar(
    ui: &Ui,
    s: &mut State,
    apps: &[(Uuid, String)],
    build: &mut bool,
    switch_to: &mut Option<Uuid>,
) {
    let current = apps
        .iter()
        .find(|(id, _)| Some(*id) == s.app)
        .map(|(_, n)| n.clone())
        .unwrap_or_else(|| "Choose an app".into());
    ui.set_next_item_width(220.);
    if let Some(_combo) = ui.begin_combo("##ps-app", &current) {
        for (id, name) in apps {
            if ui
                .selectable_config(name)
                .selected(Some(*id) == s.app)
                .build()
            {
                *switch_to = Some(*id);
            }
        }
    }
    if ui.is_item_hovered() {
        ui.tooltip_text("The PocketStation app to edit. Apps are assets in the Project browser.");
    }
    ui.same_line();
    if ui.button("New App...") {
        s.new_popup = true;
    }
    ui.same_line();
    let can_build = s.doc.is_some();
    {
        let _disabled = ui.begin_disabled(!can_build);
        let _accent = ui.push_style_color(imgui::StyleColor::Button, [0.16, 0.42, 0.24, 1.0]);
        if ui.button("\u{eb2c} Build & Run") {
            *build = true;
        }
    }
    if ui.is_item_hovered() {
        ui.tooltip_text(
            "Compile the app, put it on the emulated PocketStation's card and start it (F5).",
        );
    }
    ui.same_line();
    if let Some(machine) = &mut s.machine {
        let running = matches!(machine.state, ps::State::Program(_));
        let label = if s.paused { "Resume" } else { "Pause" };
        {
            let _disabled = ui.begin_disabled(!running);
            if ui.button(label) {
                s.paused = !s.paused;
            }
        }
        ui.same_line();
        if ui.button("Exit to Menu") {
            machine.exit_to_launcher();
            s.paused = false;
        }
        if ui.is_item_hovered() {
            ui.tooltip_text("Return to the PocketStation's clock and program menu.");
        }
        ui.same_line();
        if ui.button("Reset") {
            machine.power_on();
            machine.set_date_time(pocketstation::now());
            s.paused = false;
        }
        if ui.is_item_hovered() {
            ui.tooltip_text("Press the reset button (the card keeps its files).");
        }
        ui.same_line();
        let mut docked = machine.hw.docked;
        if ui.checkbox("In a PlayStation", &mut docked) {
            machine.set_docked(docked);
        }
        if ui.is_item_hovered() {
            ui.tooltip_text("Inserted in a PlayStation memory card slot: the PlayStation can talk to it, and the screen turns around.");
        }
    }
    if !s.status.is_empty() {
        crate::gui::muted(ui, &s.status);
    }
}

fn empty_state(ui: &Ui, s: &mut State, apps: &[(Uuid, String)], switch_to: &mut Option<Uuid>) {
    ui.spacing();
    ui.text("PocketStation apps run on Sony's PocketStation, the memory card with a screen.");
    crate::gui::muted(
        ui,
        "Write them in Pocket Lua (Lua compiled to ARM code by Epok) or in assembly, draw sprites and icons, and play them here.",
    );
    ui.spacing();
    if ui.button_with_size("Create a PocketStation app", [260., 34.]) {
        s.new_popup = true;
    }
    if !apps.is_empty() {
        ui.spacing();
        ui.text("Or open one:");
        for (id, name) in apps {
            if ui.selectable(name) {
                *switch_to = Some(*id);
            }
        }
    }
}

fn new_app_popup(ui: &Ui, s: &mut State, create: &mut Option<(Template, String)>) {
    if s.new_popup {
        ui.open_popup("New PocketStation App");
        s.new_popup = false;
    }
    ui.modal_popup_config("New PocketStation App")
        .always_auto_resize(true)
        .build(|| {
            ui.text("Name");
            ui.set_next_item_width(320.);
            ui.input_text("##ps-new-title", &mut s.new_title).build();
            ui.spacing();
            ui.text("Start from");
            for (i, template) in Template::ALL.iter().enumerate() {
                if ui.radio_button_bool(template.label(), s.new_template == i) {
                    s.new_template = i;
                }
                crate::gui::muted(ui, format!("    {}", template.description()));
            }
            ui.spacing();
            let title = s.new_title.trim().to_string();
            {
                let _disabled = ui.begin_disabled(title.is_empty());
                if ui.button_with_size("Create", [120., 0.]) {
                    *create = Some((Template::ALL[s.new_template], title));
                    ui.close_current_popup();
                }
            }
            ui.same_line();
            if ui.button_with_size("Cancel", [120., 0.]) {
                ui.close_current_popup();
            }
        });
}

fn editor_tabs(ui: &Ui, s: &mut State) {
    let select = s.select_tab.take();
    if let Some(_bar) = ui.tab_bar("ps-tabs") {
        for (tab, label) in [
            (Tab::Code, "Code"),
            (Tab::Sprites, "Sprites"),
            (Tab::Icons, "Icons"),
            (Tab::Settings, "App Settings"),
            (Tab::Card, "Card"),
        ] {
            let flags = if select == Some(tab) {
                imgui::TabItemFlags::SET_SELECTED
            } else {
                imgui::TabItemFlags::empty()
            };
            if let Some(_item) = ui.tab_item_with_flags(label, None, flags) {
                s.tab = tab;
                match tab {
                    Tab::Code => code_tab(ui, s),
                    Tab::Sprites => sprites_tab(ui, s),
                    Tab::Icons => icons_tab(ui, s),
                    Tab::Settings => settings_tab(ui, s),
                    Tab::Card => card_tab(ui, s),
                }
            }
        }
    }
}

// ---------------------------------------------------------------------- code

fn code_tab(ui: &Ui, s: &mut State) {
    let Some(doc) = &mut s.doc else { return };
    let mut changed = false;
    let mut remove: Option<usize> = None;
    for (i, f) in doc.files.iter().enumerate() {
        if i > 0 {
            ui.same_line();
        }
        let width = ui.calc_text_size(&f.name)[0] + 8.;
        if ui
            .selectable_config(&f.name)
            .selected(i == s.file)
            .size([width, 0.])
            .build()
        {
            s.file = i;
        }
        if i > 0 {
            if let Some(_menu) = ui.begin_popup_context_item() {
                if ui.menu_item("Delete file") {
                    remove = Some(i);
                }
            }
        }
    }
    ui.same_line();
    if ui.small_button("+ Assembly file") {
        let mut n = doc.files.len();
        let mut name = format!("routines{n}.s");
        while doc.files.iter().any(|f| f.name == name) {
            n += 1;
            name = format!("routines{n}.s");
        }
        doc.files.push(pocketstation::SourceFile {
            name,
            text: "; Thumb functions callable from Pocket Lua: numbers in r0-r3, result in r0.\n    .thumb\n    .global double\n    .thumb_func\ndouble:\n    add r0, r0, r0\n    bx lr\n".into(),
        });
        s.file = doc.files.len() - 1;
        changed = true;
    }
    if ui.is_item_hovered() {
        ui.tooltip_text("Add an assembly file. Its `.global` functions can be called from Pocket Lua like any function.");
    }
    if let Some(i) = remove {
        doc.files.remove(i);
        s.file = s.file.min(doc.files.len() - 1);
        changed = true;
    }
    s.file = s.file.min(doc.files.len().saturating_sub(1));
    let language = doc.language;
    let file = &mut doc.files[s.file];
    let (line, column) = line_column(&file.text, s.cursor);
    let help = match language {
        Language::PocketLua if file.name.ends_with(".plua") => {
            "Pocket Lua · F5 Build & Run · libraries: screen, button, sound, clock, save, link, system, math"
        }
        _ => "ARM/Thumb assembly · `;` comments · `.thumb_func` before Thumb labels",
    };
    crate::gui::muted(ui, format!("Ln {line}, Col {column}    {help}"));
    let mut goto = s.goto_line.take().map(|l| line_offset(&file.text, l));
    if s.focus_code {
        ui.set_keyboard_focus_here();
        s.focus_code = false;
    }
    let size = ui.content_region_avail();
    let _font = CODE_FONT.with(|f| f.get()).map(|f| ui.push_font(f));
    let _background = ui.push_style_color(imgui::StyleColor::FrameBg, [0.09, 0.10, 0.11, 1.0]);
    let handler = Cursor {
        cursor: &mut s.cursor,
        goto: &mut goto,
    };
    if ui
        .input_text_multiline("##ps-code", &mut file.text, [size[0], size[1].max(120.)])
        .allow_tab_input(true)
        .callback(InputTextMultilineCallback::ALWAYS, handler)
        .build()
    {
        changed = true;
    }
    if let Some(at) = goto {
        // The editor was not active this frame; try again next frame.
        s.goto_line = Some(line_column(&file.text, at).0 as u32);
        s.focus_code = true;
    }
    if changed {
        mark_dirty(s);
    }
}

// ------------------------------------------------------------------- pixels

/// A grid of square cells; returns the cell under a held mouse button.
fn pixel_grid(
    ui: &Ui,
    id: &str,
    columns: u32,
    rows: u32,
    cell: f32,
    colour: impl Fn(u32, u32) -> [f32; 4],
) -> Option<(u32, u32, bool)> {
    let origin = ui.cursor_screen_pos();
    let size = [columns as f32 * cell, rows as f32 * cell];
    ui.invisible_button(id, size);
    let draw = ui.get_window_draw_list();
    draw.add_rect(
        origin,
        [origin[0] + size[0], origin[1] + size[1]],
        [0.3, 0.3, 0.3, 1.0],
    )
    .filled(true)
    .build();
    for y in 0..rows {
        for x in 0..columns {
            let p = [origin[0] + x as f32 * cell, origin[1] + y as f32 * cell];
            draw.add_rect(p, [p[0] + cell - 1., p[1] + cell - 1.], colour(x, y))
                .filled(true)
                .build();
        }
    }
    let hovered = ui.is_item_hovered();
    let left = ui.is_mouse_down(MouseButton::Left);
    let right = ui.is_mouse_down(MouseButton::Right);
    if (ui.is_item_active() || hovered) && (left || right) {
        let m = ui.io().mouse_pos;
        let x = ((m[0] - origin[0]) / cell).floor();
        let y = ((m[1] - origin[1]) / cell).floor();
        if x >= 0. && y >= 0. && (x as u32) < columns && (y as u32) < rows {
            return Some((x as u32, y as u32, left));
        }
    }
    None
}

const INK: [f32; 4] = [0.11, 0.13, 0.10, 1.0];
const PAPER: [f32; 4] = [0.65, 0.70, 0.57, 1.0];

/// Paints a mono image (rows of `#`/`.`); the first pressed cell decides the colour.
fn mono_editor(
    ui: &Ui,
    id: &str,
    rows: &mut [String],
    width: u32,
    height: u32,
    cell: f32,
    paint: &mut Option<bool>,
) -> bool {
    let get = |rows: &[String], x: u32, y: u32| {
        rows.get(y as usize)
            .and_then(|r| r.as_bytes().get(x as usize))
            .is_some_and(|c| *c == b'#')
    };
    let hit = pixel_grid(ui, id, width, height, cell, |x, y| {
        if get(rows, x, y) { INK } else { PAPER }
    });
    if !ui.is_mouse_down(MouseButton::Left) && !ui.is_mouse_down(MouseButton::Right) {
        *paint = None;
    }
    if let Some((x, y, left)) = hit {
        let target = *paint.get_or_insert(if left { !get(rows, x, y) } else { false });
        if get(rows, x, y) != target {
            if let Some(row) = rows.get_mut(y as usize) {
                let mut bytes = row.clone().into_bytes();
                bytes[x as usize] = if target { b'#' } else { b'.' };
                *row = String::from_utf8(bytes).unwrap_or_default();
                return true;
            }
        }
    }
    false
}

fn frame_controls(
    ui: &Ui,
    id: &str,
    frames: &mut Vec<Vec<String>>,
    current: &mut usize,
    max: usize,
) -> bool {
    let mut changed = false;
    *current = (*current).min(frames.len() - 1);
    ui.text(format!("Frame {} of {}", *current + 1, frames.len()));
    ui.same_line();
    if ui.small_button(&format!("<##{id}-prev")) && *current > 0 {
        *current -= 1;
    }
    ui.same_line();
    if ui.small_button(&format!(">##{id}-next")) && *current + 1 < frames.len() {
        *current += 1;
    }
    ui.same_line();
    {
        let _disabled = ui.begin_disabled(frames.len() >= max);
        if ui.small_button(&format!("Duplicate##{id}-dup")) {
            let copy = frames[*current].clone();
            frames.insert(*current + 1, copy);
            *current += 1;
            changed = true;
        }
    }
    ui.same_line();
    {
        let _disabled = ui.begin_disabled(frames.len() <= 1);
        if ui.small_button(&format!("Delete##{id}-del")) {
            frames.remove(*current);
            *current = (*current).min(frames.len() - 1);
            changed = true;
        }
    }
    ui.same_line();
    if ui.small_button(&format!("Clear##{id}-clear")) {
        for row in frames[*current].iter_mut() {
            *row = ".".repeat(row.len());
        }
        changed = true;
    }
    changed
}

fn preview_mono(ui: &Ui, rows: &[String], width: u32, height: u32, scale: f32) {
    let origin = ui.cursor_screen_pos();
    ui.dummy([width as f32 * scale, height as f32 * scale]);
    let draw = ui.get_window_draw_list();
    draw.add_rect(
        origin,
        [
            origin[0] + width as f32 * scale,
            origin[1] + height as f32 * scale,
        ],
        PAPER,
    )
    .filled(true)
    .build();
    for (y, row) in rows.iter().enumerate().take(height as usize) {
        for (x, c) in row.chars().enumerate().take(width as usize) {
            if c == '#' {
                let p = [origin[0] + x as f32 * scale, origin[1] + y as f32 * scale];
                draw.add_rect(p, [p[0] + scale, p[1] + scale], INK)
                    .filled(true)
                    .build();
            }
        }
    }
}

fn sprites_tab(ui: &Ui, s: &mut State) {
    let Some(doc) = &mut s.doc else { return };
    let mut changed = false;
    ui.child_window("ps-sprite-list")
        .size([170., 0.])
        .border(true)
        .build(|| {
            ui.text("Sprites");
            crate::gui::muted(ui, "Use in code as sprite \"name\"");
            ui.separator();
            for (i, sprite) in doc.sprites.iter().enumerate() {
                if ui
                    .selectable_config(&sprite.name)
                    .selected(i == s.sprite)
                    .build()
                {
                    s.sprite = i;
                    s.frame = 0;
                }
            }
            ui.separator();
            ui.set_next_item_width(-1.);
            ui.input_text("##ps-new-sprite", &mut s.new_sprite_name)
                .build();
            let name: String = s
                .new_sprite_name
                .chars()
                .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            let taken = doc.sprites.iter().any(|sp| sp.name == name);
            {
                let _disabled = ui.begin_disabled(name.is_empty() || taken);
                if ui.button_with_size("Add sprite", [-1., 0.]) {
                    doc.sprites.push(SpriteDoc::new(&name, 8, 8));
                    s.sprite = doc.sprites.len() - 1;
                    s.frame = 0;
                    changed = true;
                }
            }
            if taken {
                crate::gui::muted(ui, "That name is in use.");
            }
        });
    ui.same_line();
    ui.child_window("ps-sprite-edit").size([0., 0.]).build(|| {
        if doc.sprites.is_empty() {
            ui.text("No sprites yet.");
            crate::gui::muted(ui, "Add one on the left, draw it here, then draw it with screen.sprite(sprite \"name\", x, y).");
            crate::gui::muted(ui, "Sprites can also be written in code as rows of # and . between [[ and ]].");
            return;
        }
        s.sprite = s.sprite.min(doc.sprites.len() - 1);
        let sprite = &mut doc.sprites[s.sprite];
        ui.text(format!("sprite \"{}\"", sprite.name));
        ui.same_line();
        if ui.small_button("Delete sprite") {
            doc.sprites.remove(s.sprite);
            s.sprite = s.sprite.saturating_sub(1);
            changed = true;
            return;
        }
        let mut width = sprite.width as i32;
        let mut height = sprite.height as i32;
        ui.set_next_item_width(90.);
        let w = ui.input_int("Width", &mut width).build();
        ui.same_line();
        ui.set_next_item_width(90.);
        let h = ui.input_int("Height", &mut height).build();
        if w || h {
            sprite.resize(width.clamp(1, 32) as u32, height.clamp(1, 32) as u32);
            changed = true;
        }
        changed |= frame_controls(ui, "sprite", &mut sprite.frames, &mut s.frame, 255);
        let avail = ui.content_region_avail();
        let cell = ((avail[0] - 140.) / sprite.width as f32)
            .min((avail[1] - 40.) / sprite.height as f32)
            .clamp(6., 28.);
        let (width, height) = (sprite.width, sprite.height);
        changed |= mono_editor(ui, "##sprite-grid", &mut sprite.frames[s.frame], width, height, cell, &mut s.paint);
        ui.same_line();
        ui.group(|| {
            ui.text("Preview");
            let tick = (Instant::now().elapsed().as_millis() as usize / 200 + frame_clock()) % sprite.frames.len();
            let scale = (96. / width.max(height) as f32).floor().clamp(2., 8.);
            preview_mono(ui, &sprite.frames[tick], width, height, scale);
            crate::gui::muted(ui, "Left: draw\nRight: erase");
        });
    });
    if changed {
        mark_dirty(s);
    }
}

fn frame_clock() -> usize {
    use std::sync::OnceLock;
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_millis() as usize / 200
}

fn icons_tab(ui: &Ui, s: &mut State) {
    let Some(doc) = &mut s.doc else { return };
    let mut changed = false;
    ui.text("Menu icon");
    crate::gui::muted(
        ui,
        "Shown in the PocketStation's program menu (32x32, animated).",
    );
    changed |= frame_controls(ui, "menu", &mut doc.menu_icon.frames, &mut s.icon_frame, 32);
    let mut delay = doc.menu_icon.delay as i32;
    ui.set_next_item_width(120.);
    if ui.input_int("Ticks per frame (1/30 s)", &mut delay).build() {
        doc.menu_icon.delay = delay.clamp(1, 255) as u16;
        changed = true;
    }
    let avail = ui.content_region_avail();
    let cell = ((avail[0] * 0.45) / 32.).clamp(5., 12.);
    changed |= mono_editor(
        ui,
        "##menu-grid",
        &mut doc.menu_icon.frames[s.icon_frame],
        32,
        32,
        cell,
        &mut s.paint,
    );
    ui.same_line();
    ui.group(|| {
        ui.text("PlayStation icon");
        crate::gui::muted(
            ui,
            "Shown by the PlayStation's memory card screen (16x16, 16 colours).",
        );
        let icon = &mut doc.color_icon;
        s.color_frame = s.color_frame.min(icon.frames.len() - 1);
        ui.text(format!(
            "Frame {} of {}",
            s.color_frame + 1,
            icon.frames.len()
        ));
        ui.same_line();
        if ui.small_button("<##color-prev") && s.color_frame > 0 {
            s.color_frame -= 1;
        }
        ui.same_line();
        if ui.small_button(">##color-next") && s.color_frame + 1 < icon.frames.len() {
            s.color_frame += 1;
        }
        ui.same_line();
        {
            let _disabled = ui.begin_disabled(icon.frames.len() >= 3);
            if ui.small_button("Add frame##color") {
                let copy = icon.frames[s.color_frame].clone();
                icon.frames.push(copy);
                s.color_frame = icon.frames.len() - 1;
                changed = true;
            }
        }
        let palette: Vec<[f32; 4]> = icon
            .palette
            .iter()
            .map(|c| {
                [
                    c[0] as f32 / 255.,
                    c[1] as f32 / 255.,
                    c[2] as f32 / 255.,
                    1.0,
                ]
            })
            .collect();
        let frame = s.color_frame;
        let index = |rows: &[String], x: u32, y: u32| {
            rows.get(y as usize)
                .and_then(|r| r.chars().nth(x as usize))
                .and_then(|c| c.to_digit(16))
                .unwrap_or(0) as usize
        };
        let rows_snapshot = icon.frames[frame].clone();
        let hit = pixel_grid(ui, "##color-grid", 16, 16, 14., |x, y| {
            let i = index(&rows_snapshot, x, y);
            if i == 0 {
                [0.2, 0.2, 0.22, 1.0]
            } else {
                palette[i]
            }
        });
        if let Some((x, y, left)) = hit {
            let value = if left { s.color_index } else { 0 };
            let row = &mut icon.frames[frame][y as usize];
            let mut chars: Vec<char> = row.chars().collect();
            while chars.len() < 16 {
                chars.push('0');
            }
            let digit = char::from_digit(value as u32, 16)
                .unwrap()
                .to_ascii_uppercase();
            if chars[x as usize] != digit {
                chars[x as usize] = digit;
                *row = chars.into_iter().collect();
                changed = true;
            }
        }
        ui.text("Palette (0 = transparent)");
        for i in 0..16usize {
            if i % 8 != 0 {
                ui.same_line();
            }
            let c = icon.palette[i];
            let mut colour = [c[0] as f32 / 255., c[1] as f32 / 255., c[2] as f32 / 255.];
            let selected = s.color_index as usize == i;
            let _border = ui.push_style_var(imgui::StyleVar::FrameBorderSize(if selected {
                2.
            } else {
                0.
            }));
            if ui
                .color_button_config(&format!("##pal{i}"), [colour[0], colour[1], colour[2], 1.0])
                .size([22., 22.])
                .build()
            {
                s.color_index = i as u8;
            }
            if selected && i > 0 {
                let _ = &mut colour;
            }
        }
        if s.color_index > 0 {
            let c = icon.palette[s.color_index as usize];
            let mut colour = [c[0] as f32 / 255., c[1] as f32 / 255., c[2] as f32 / 255.];
            ui.set_next_item_width(200.);
            if ui.color_edit3(format!("Colour {}", s.color_index), &mut colour) {
                icon.palette[s.color_index as usize] = colour.map(|v| (v * 255.).round() as u8);
                changed = true;
            }
        }
    });
    if changed {
        mark_dirty(s);
    }
}

fn settings_tab(ui: &Ui, s: &mut State) {
    let Some(doc) = &mut s.doc else { return };
    let mut changed = false;
    ui.text("Title");
    crate::gui::muted(
        ui,
        "Shown on the PlayStation memory card screen (up to 32 characters).",
    );
    ui.set_next_item_width(360.);
    if ui.input_text("##ps-title", &mut doc.title).build() {
        doc.title.truncate(32);
        changed = true;
    }
    ui.spacing();
    ui.text("Memory card file name");
    crate::gui::muted(
        ui,
        "Identifies the program on the card; a PlayStation game installs and finds it by this name. The 7th character is P for PocketStation programs.",
    );
    ui.set_next_item_width(240.);
    if ui.input_text("##ps-file-name", &mut doc.file_name).build() {
        doc.file_name
            .retain(|c| c.is_ascii_graphic() && c != '*' && c != '?');
        doc.file_name.truncate(20);
        changed = true;
    }
    ui.same_line();
    if ui.button("Generate") {
        doc.file_name = pocketstation::file_name_for(&doc.title);
        changed = true;
    }
    if let Err(problem) = pocketstation::validate_file_name(&doc.file_name) {
        ui.text_colored([1.0, 0.5, 0.4, 1.0], problem);
    } else if doc.file_name.as_bytes().get(6) != Some(&b'P') {
        ui.text_colored(
            [0.95, 0.75, 0.3, 1.0],
            "Tip: Sony's convention puts P as the 7th character of PocketStation program names.",
        );
    }
    ui.spacing();
    ui.text(format!("Language: {}", doc.language.label()));
    ui.separator();
    ui.text("Last build");
    match &s.built {
        Some(built) => {
            let ram = built.program.ram_end.saturating_sub(0x200);
            ui.text(format!(
                "Flash: {} block{} of 15 ({} bytes)",
                built.blocks,
                if built.blocks == 1 { "" } else { "s" },
                built.bytes.len()
            ));
            ui.text(format!(
                "RAM: {ram} of 1536 bytes for variables, {} left for the stack",
                built.program.stack_bytes()
            ));
            let fraction = ram as f32 / 1536.;
            imgui::ProgressBar::new(fraction)
                .size([300., 0.])
                .overlay_text(format!("{:.0}% RAM", fraction * 100.))
                .build(ui);
            let mut features = Vec::new();
            if built.program.uses_link {
                features.push("PlayStation link");
            }
            if built.program.uses_save {
                features.push("save data");
            }
            if built.program.uses_sound {
                features.push("sound");
            }
            if built.program.uses_print {
                features.push("print");
            }
            if !features.is_empty() {
                ui.text(format!("Uses: {}", features.join(", ")));
            }
            ui.spacing();
            if ui.button("Export for a real PocketStation (.mcs)...") {
                match export(doc, built, "mcs") {
                    Ok(path) => s.status = format!("Exported {}", path.display()),
                    Err(error) => s.error = Some(error),
                }
            }
            if ui.is_item_hovered() {
                ui.tooltip_text("A single-save file with its card name, for memory card managers and PocketStation transfer tools.");
            }
            ui.same_line();
            if ui.button("Export raw file (.bin)") {
                match export(doc, built, "bin") {
                    Ok(path) => s.status = format!("Exported {}", path.display()),
                    Err(error) => s.error = Some(error),
                }
            }
            ui.same_line();
            if ui.button("Show generated assembly") {
                let path = std::env::temp_dir().join(format!("{}.s", doc.file_name));
                match std::fs::write(&path, &built.program.source) {
                    Ok(()) => s.status = format!("Wrote {}", path.display()),
                    Err(error) => s.error = Some(error.to_string()),
                }
            }
        }
        None => crate::gui::muted(
            ui,
            "Build & Run (F5) to see the program's size and RAM use.",
        ),
    }
    if changed {
        mark_dirty(s);
    }
}

fn export(doc: &PocketApp, built: &Built, extension: &str) -> Result<std::path::PathBuf, String> {
    let directory = std::env::current_dir()
        .map_err(|e| e.to_string())?
        .join("exports/pocketstation");
    std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    let path = directory.join(format!("{}.{extension}", doc.file_name));
    let bytes = if extension == "mcs" {
        pocketstation::single_save(&doc.file_name, &built.bytes)?
    } else {
        built.bytes.clone()
    };
    std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
    Ok(path)
}

fn card_tab(ui: &Ui, s: &mut State) {
    let Some(machine) = &mut s.machine else {
        return;
    };
    let files = card::files(&machine.hw.flash);
    let free = card::free_blocks(&machine.hw.flash).len();
    ui.text(format!(
        "The emulated PocketStation's card: {} file{}, {free} of 15 blocks free",
        files.len(),
        if files.len() == 1 { "" } else { "s" }
    ));
    crate::gui::muted(
        ui,
        "Saved in UserSettings/PocketStation.mcd. Programs from Build & Run appear here; PlayStation games installed through the link do too.",
    );
    ui.separator();
    let mut launch = None;
    let mut delete = None;
    let mut export_file = None;
    for f in &files {
        let at = f.block as usize * card::BLOCK_SIZE;
        let info = file::parse(&machine.hw.flash[at..at + card::BLOCK_SIZE]);
        let _id = ui.push_id(f.name.as_str());
        ui.group(|| {
            if let Some(icon) = info
                .as_ref()
                .and_then(|i| i.menu_icons.first())
                .and_then(|i| i.frames.first())
            {
                let rows: Vec<String> = icon
                    .iter()
                    .map(|r| {
                        (0..32)
                            .map(|x| if r & (1 << x) != 0 { '#' } else { '.' })
                            .collect()
                    })
                    .collect();
                preview_mono(ui, &rows, 32, 32, 1.5);
            } else {
                ui.dummy([48., 48.]);
            }
        });
        ui.same_line();
        ui.group(|| {
            match &info {
                Some(info) => ui.text(format!("{}  (program)", info.title)),
                None => ui.text("Save data"),
            }
            crate::gui::muted(
                ui,
                format!(
                    "{} · block {} · {} block{}",
                    f.name,
                    f.block,
                    f.blocks.len(),
                    if f.blocks.len() == 1 { "" } else { "s" }
                ),
            );
            if info.is_some() && ui.small_button("Start") {
                launch = Some(f.block);
            }
            if info.is_some() {
                ui.same_line();
            }
            if ui.small_button("Export .mcs") {
                export_file = Some(f.clone());
            }
            ui.same_line();
            if ui.small_button("Delete") {
                delete = Some(f.clone());
            }
        });
        ui.separator();
    }
    if files.is_empty() {
        crate::gui::muted(ui, "The card is empty.");
    }
    if let Some(block) = launch {
        if let Err(error) = machine.launch(block, 0) {
            s.error = Some(error);
        }
        s.paused = false;
    }
    if let Some(f) = delete {
        if matches!(machine.state, ps::State::Program(b) if b == f.block) {
            machine.exit_to_launcher();
        }
        card::delete(&mut machine.hw.flash, &f);
    }
    if let Some(f) = export_file {
        let bytes = card::export_single(&machine.hw.flash, &f);
        let path = std::env::current_dir()
            .unwrap_or_default()
            .join("exports/pocketstation");
        let result = std::fs::create_dir_all(&path)
            .and_then(|_| std::fs::write(path.join(format!("{}.mcs", f.name)), bytes));
        match result {
            Ok(()) => {
                s.status = format!(
                    "Exported {}",
                    path.join(format!("{}.mcs", f.name)).display()
                )
            }
            Err(error) => s.error = Some(error.to_string()),
        }
    }
    ui.spacing();
    if !s.confirm_format {
        if ui.button("Format card...") {
            s.confirm_format = true;
        }
    } else {
        ui.text_colored(
            [1.0, 0.6, 0.4, 1.0],
            "Erase every file on the emulated card?",
        );
        ui.same_line();
        if ui.button("Erase") {
            machine.hw.flash = card::formatted();
            machine.exit_to_launcher();
            s.confirm_format = false;
        }
        ui.same_line();
        if ui.button("Keep") {
            s.confirm_format = false;
        }
    }
}

// ------------------------------------------------------------------ device

fn device(ui: &Ui, s: &mut State, focused: bool) {
    let avail = ui.content_region_avail();
    // Leave room for the panels below the device.
    let scale = ((avail[0] - 70.) / 32.)
        .min((avail[1] * 0.55 - 190.) / 32.)
        .floor()
        .clamp(4., 9.);
    let lcd = 32. * scale;
    let width = lcd + 56.;
    let height = lcd + 56. + 120.;
    let origin = ui.cursor_screen_pos();
    let origin = [
        origin[0] + ((avail[0] - width) / 2.).max(0.),
        origin[1] + 4.,
    ];
    let draw = ui.get_window_draw_list();
    // Body.
    draw.add_rect(
        origin,
        [origin[0] + width, origin[1] + height],
        [0.80, 0.81, 0.83, 1.0],
    )
    .filled(true)
    .rounding(26.)
    .build();
    draw.add_rect(
        origin,
        [origin[0] + width, origin[1] + height],
        [0.55, 0.56, 0.60, 1.0],
    )
    .rounding(26.)
    .thickness(2.)
    .build();
    // Screen bezel and LCD.
    let bezel = [origin[0] + 16., origin[1] + 16.];
    draw.add_rect(
        bezel,
        [bezel[0] + lcd + 24., bezel[1] + lcd + 24.],
        [0.20, 0.22, 0.25, 1.0],
    )
    .filled(true)
    .rounding(10.)
    .build();
    let screen = [bezel[0] + 12., bezel[1] + 12.];
    ui.set_cursor_screen_pos(screen);
    match s.texture {
        Some(texture) => imgui::Image::new(texture, [lcd, lcd]).build(ui),
        None => ui.dummy([lcd, lcd]),
    }
    if ui.is_item_clicked() {
        s.capture = true;
    }
    if ui.is_item_hovered() {
        ui.tooltip_text("Click to control the PocketStation with the keyboard:\narrow keys, and Z / Space / Enter for Fire. Esc releases.");
    }
    let led = s.machine.as_ref().is_some_and(|m| m.led());
    draw.add_circle(
        [bezel[0] + lcd + 16., bezel[1] + 6.],
        3.5,
        if led {
            [1.0, 0.15, 0.1, 1.0]
        } else {
            [0.35, 0.12, 0.12, 1.0]
        },
    )
    .filled(true)
    .build();
    if s.speaker > 0 {
        draw.add_text(
            [bezel[0] + 6., bezel[1] - 1.],
            [0.95, 0.95, 0.6, 1.0],
            "\u{f028}",
        );
    }
    // Buttons: a cross on the left and Fire on the right.
    let pad_y = origin[1] + lcd + 56. + 58.;
    let left_x = origin[0] + width * 0.32;
    let r = 15.;
    let mut pressed = 0u8;
    let mut button_at = |ui: &Ui, id: &str, centre: [f32; 2], radius: f32, bit: u8, label: &str| {
        ui.set_cursor_screen_pos([centre[0] - radius, centre[1] - radius]);
        ui.invisible_button(id, [radius * 2., radius * 2.]);
        let held = ui.is_item_active();
        if held {
            pressed |= bit;
        }
        let keyboard = s.mouse_buttons & bit != 0;
        let colour = if held || keyboard {
            [0.32, 0.34, 0.40, 1.0]
        } else {
            [0.45, 0.47, 0.53, 1.0]
        };
        draw.add_circle(centre, radius, colour)
            .filled(true)
            .num_segments(32)
            .build();
        let size = ui.calc_text_size(label);
        draw.add_text(
            [centre[0] - size[0] / 2., centre[1] - size[1] / 2.],
            [0.95, 0.95, 0.95, 1.0],
            label,
        );
    };
    button_at(
        ui,
        "##ps-up",
        [left_x, pad_y - 30.],
        r,
        button::UP,
        "\u{f062}",
    );
    button_at(
        ui,
        "##ps-down",
        [left_x, pad_y + 30.],
        r,
        button::DOWN,
        "\u{f063}",
    );
    button_at(
        ui,
        "##ps-left",
        [left_x - 30., pad_y],
        r,
        button::LEFT,
        "\u{f060}",
    );
    button_at(
        ui,
        "##ps-right",
        [left_x + 30., pad_y],
        r,
        button::RIGHT,
        "\u{f061}",
    );
    button_at(
        ui,
        "##ps-fire",
        [origin[0] + width * 0.76, pad_y],
        r + 5.,
        button::FIRE,
        "FIRE",
    );
    // Keyboard.
    if s.capture && (!focused || ui.is_key_pressed(Key::Escape)) {
        s.capture = false;
    }
    let mut keys = 0u8;
    if s.capture && !ui.io().want_text_input {
        for (key, bit) in [
            (Key::UpArrow, button::UP),
            (Key::DownArrow, button::DOWN),
            (Key::LeftArrow, button::LEFT),
            (Key::RightArrow, button::RIGHT),
            (Key::Z, button::FIRE),
            (Key::Space, button::FIRE),
            (Key::Enter, button::FIRE),
        ] {
            if ui.is_key_down(key) {
                keys |= bit;
            }
        }
    }
    s.mouse_buttons = pressed | keys;
    ui.set_cursor_screen_pos([origin[0], origin[1] + height + 8.]);
    // Status.
    let Some(machine) = &s.machine else { return };
    let hz = machine.hw.cpu_hz() as f32 / 1e6;
    match &machine.state {
        ps::State::Launcher => ui.text("Start-up screen (clock and program menu)"),
        ps::State::Program(block) => {
            let name = machine
                .executable(*block)
                .map(|info| info.title)
                .unwrap_or_else(|| format!("block {block}"));
            ui.text(format!(
                "Running {name}{}",
                if s.paused { " (paused)" } else { "" }
            ));
        }
        ps::State::Crashed(message) => {
            ui.text_colored([1.0, 0.45, 0.4, 1.0], format!("Stopped: {message}"));
        }
    }
    crate::gui::muted(
        ui,
        format!(
            "{hz:.2} MHz · {}{}",
            if machine.hw.docked {
                "in a PlayStation"
            } else {
                "in your hand"
            },
            if s.capture {
                " · keyboard: arrows, Z/Space = Fire (Esc releases)"
            } else {
                " · click the screen to use the keyboard"
            }
        ),
    );
    if let Some((line, message)) = s.runtime_error() {
        ui.text_colored(
            [1.0, 0.45, 0.4, 1.0],
            format!("Runtime error on line {line}: {message}"),
        );
        ui.same_line();
        if ui.small_button("Show line") {
            let file = s
                .doc
                .as_ref()
                .map(|d| d.files[0].name.clone())
                .unwrap_or_default();
            jump_to(s, &file, line);
        }
    }
}

fn bottom_panels(ui: &Ui, s: &mut State, _switch_to: &mut Option<Uuid>) {
    let select = s.select_panel.take();
    if let Some(_bar) = ui.tab_bar("ps-panels") {
        for (panel, label) in [
            (Panel::Console, "Console"),
            (Panel::Problems, "Problems"),
            (Panel::Variables, "Variables"),
            (Panel::Debug, "Debug"),
            (Panel::Link, "PlayStation Link"),
        ] {
            let flags = if select == Some(panel) {
                imgui::TabItemFlags::SET_SELECTED
            } else {
                imgui::TabItemFlags::empty()
            };
            let label = if panel == Panel::Problems && !s.problems.is_empty() {
                format!("Problems ({})###Problems", s.problems.len())
            } else {
                format!("{label}###{label}")
            };
            if let Some(_item) = ui.tab_item_with_flags(&label, None, flags) {
                s.panel = panel;
                ui.child_window("ps-panel-body")
                    .size([0., 0.])
                    .build(|| match panel {
                        Panel::Console => console(ui, s),
                        Panel::Problems => problems(ui, s),
                        Panel::Variables => variables(ui, s),
                        Panel::Debug => debug(ui, s),
                        Panel::Link => link_tester(ui, s),
                    });
            }
        }
    }
}

fn console(ui: &Ui, s: &mut State) {
    let Some(machine) = &mut s.machine else {
        return;
    };
    if ui.small_button("Clear##ps-console") {
        machine.tty.clear();
    }
    ui.same_line();
    crate::gui::muted(ui, "Text from print(...) in Pocket Lua.");
    let _font = CODE_FONT.with(|f| f.get()).map(|f| ui.push_font(f));
    if machine.tty.is_empty() {
        crate::gui::muted(ui, "(nothing printed yet)");
    } else {
        ui.text_wrapped(&machine.tty);
        if ui.scroll_y() >= ui.scroll_max_y() - 4. {
            ui.set_scroll_here_y_with_ratio(1.0);
        }
    }
}

fn problems(ui: &Ui, s: &mut State) {
    if s.problems.is_empty() {
        crate::gui::muted(ui, "No problems. Build & Run (F5) checks the app.");
        return;
    }
    let mut jump = None;
    for (i, p) in s.problems.iter().enumerate() {
        let _id = ui.push_id_usize(i);
        if ui.selectable(format!("{}:{}  {}", p.file, p.line, p.message)) {
            jump = Some((p.file.clone(), p.line));
        }
        if ui.is_item_hovered() {
            ui.tooltip_text("Click to show the line in the code editor.");
        }
    }
    if let Some((file, line)) = jump {
        jump_to(s, &file, line);
    }
}

fn variables(ui: &Ui, s: &mut State) {
    let Some(built) = &s.built else {
        crate::gui::muted(ui, "Build & Run to watch the program's variables.");
        return;
    };
    if !s.running_this_app() {
        crate::gui::muted(
            ui,
            "Values appear while this app runs on the emulated PocketStation.",
        );
    }
    let Some(machine) = &s.machine else { return };
    let live = s.running_this_app();
    if built.program.variables.is_empty() {
        crate::gui::muted(ui, "The program has no variables in RAM.");
    }
    for (name, address, ty) in &built.program.variables {
        let value = machine.hw.ram_word(*address) as i32;
        let shown = if !live {
            "-".to_string()
        } else if *ty == "array" {
            let length = (value.max(0) as u32).min(64);
            let items: Vec<String> = (0..length.min(12))
                .map(|i| (machine.hw.ram_word(address + 4 + i * 4) as i32).to_string())
                .collect();
            format!(
                "[{}{}]",
                items.join(", "),
                if length > 12 { ", ..." } else { "" }
            )
        } else if *ty == "boolean" {
            (value != 0).to_string()
        } else if *ty == "number" {
            value.to_string()
        } else {
            format!("{ty} at {value:#x}")
        };
        ui.text(format!("{name:<16} {shown}"));
        if ui.is_item_hovered() {
            ui.tooltip_text(format!("{ty} in RAM at {address:#05x}"));
        }
    }
}

fn debug(ui: &Ui, s: &mut State) {
    let Some(machine) = &mut s.machine else {
        return;
    };
    let running = matches!(machine.state, ps::State::Program(_));
    {
        let _disabled = ui.begin_disabled(!running);
        if ui.small_button(if s.paused { "Resume" } else { "Pause" }) {
            s.paused = !s.paused;
        }
        ui.same_line();
        if ui.small_button("Step instruction") {
            s.paused = true;
            machine.step_cpu();
        }
    }
    let pc = machine.cpu.pc();
    let line = s.built.as_ref().and_then(|b| {
        let file = s.doc.as_ref()?.files.first()?.name.clone();
        b.program.line_at(pc, &file).map(|l| (file, l))
    });
    match &line {
        Some((file, l)) => ui.text(format!("PC {pc:#010x}  ({file} line {l})")),
        None => ui.text(format!(
            "PC {pc:#010x}  ({} state, {})",
            if machine.cpu.thumb() { "Thumb" } else { "ARM" },
            machine.mode_name()
        )),
    }
    let _font = CODE_FONT.with(|f| f.get()).map(|f| ui.push_font(f));
    for row in 0..4 {
        let text: Vec<String> = (0..4)
            .map(|c| {
                let r = row * 4 + c;
                format!("r{r:<2} {:08x}", machine.cpu.r[r])
            })
            .collect();
        ui.text(text.join("  "));
    }
    ui.text(format!(
        "cpsr {:08x}  {}",
        machine.cpu.cpsr,
        machine.mode_name()
    ));
    drop(_font);
    if let Some((file, l)) = line {
        if ui.small_button("Show line##debug") {
            jump_to(s, &file, l);
        }
    }
    let warnings = &s.machine.as_ref().unwrap().hw.warnings;
    if !warnings.is_empty() {
        ui.separator();
        ui.text_colored(
            [0.95, 0.75, 0.3, 1.0],
            "Would behave differently on a real PocketStation:",
        );
        for w in warnings.iter().take(8) {
            ui.text_wrapped(format!("{:#010x}: {}", w.pc, w.message));
        }
    }
}

/// Speaks to the running program the way a PlayStation game would.
fn link_tester(ui: &Ui, s: &mut State) {
    let Some(machine) = &mut s.machine else {
        return;
    };
    crate::gui::muted(
        ui,
        "Test the program's link functions without a PlayStation game (Pocket Lua: link.received, link.send, link.value).",
    );
    let ready = machine.link_enabled();
    if !machine.hw.docked {
        if ui.small_button("Insert into a PlayStation") {
            machine.set_docked(true);
        }
        ui.same_line();
        crate::gui::muted(ui, "The link needs the PocketStation in a PlayStation.");
    } else if !ready {
        ui.text_colored(
            [0.95, 0.75, 0.3, 1.0],
            "The running program does not accept the link (Pocket Lua programs do).",
        );
    }
    ui.set_next_item_width(160.);
    ui.input_text("##ps-link-bytes", &mut s.link_text)
        .hint("bytes, e.g. 1 2 3")
        .build();
    ui.same_line();
    if ui.small_button("Send message") {
        let bytes: Vec<u8> = s
            .link_text
            .split_whitespace()
            .filter_map(|w| w.parse::<u8>().ok())
            .take(64)
            .collect();
        if !bytes.is_empty() {
            let mut frame = vec![0x81, 0x5C, 0x80, 0, bytes.len() as u8, 0];
            frame.extend(&bytes);
            frame.push(0);
            let reply = machine.link_transfer(&frame);
            s.link_reply = if reply.len() == frame.len() {
                format!("Sent {} bytes", bytes.len())
            } else {
                "The program did not take the message".into()
            };
        }
    }
    ui.same_line();
    if ui.small_button("Read message") {
        let mut frame = vec![0x81, 0x5B, 0x81, 0, 0];
        frame.extend([0u8; 65]);
        let reply = machine.link_transfer(&frame);
        s.link_reply = match reply.get(4) {
            Some(&n) if reply.len() >= 5 + n as usize => {
                if n == 0 {
                    "No message waiting".into()
                } else {
                    format!("Message: {:?}", &reply[5..5 + n as usize])
                }
            }
            _ => "No answer".into(),
        };
    }
    ui.set_next_item_width(70.);
    ui.input_int("##ps-value-index", &mut s.link_index).build();
    s.link_index = s.link_index.clamp(1, 16);
    ui.same_line();
    ui.set_next_item_width(90.);
    ui.input_int("##ps-value", &mut s.link_value).build();
    ui.same_line();
    if ui.small_button("Set value") {
        let mut frame = vec![0x81, 0x5C, 0x83, 0, s.link_index as u8, 0];
        frame.extend((s.link_value as u32).to_le_bytes());
        frame.push(0);
        let reply = machine.link_transfer(&frame);
        s.link_reply = if reply.len() == frame.len() {
            format!("Value {} = {}", s.link_index, s.link_value)
        } else {
            "No answer".into()
        };
    }
    ui.same_line();
    if ui.small_button("Read values") {
        let mut frame = vec![0x81, 0x5B, 0x82, 0, 0];
        frame.extend([0u8; 65]);
        let reply = machine.link_transfer(&frame);
        s.link_reply = if reply.len() >= 70 {
            let values: Vec<String> = (0..16)
                .map(|i| {
                    i32::from_le_bytes(reply[5 + i * 4..9 + i * 4].try_into().unwrap()).to_string()
                })
                .collect();
            format!("Values: {}", values.join(" "))
        } else {
            "No answer".into()
        };
    }
    if !s.link_reply.is_empty() {
        ui.text_wrapped(&s.link_reply);
    }
    ui.separator();
    ui.text("Recent transactions");
    for t in machine.link.log.iter().rev().take(12) {
        ui.text(format!("{:02X}h  {}", t.command, t.summary));
        if ui.is_item_hovered() {
            ui.tooltip_text(format!(
                "{} · sent {} bytes, answered {}",
                link::command_name(t.command),
                t.sent.len(),
                t.received.len()
            ));
        }
    }
}
