use crate::{
    editor::Editor,
    project_templates,
    workspace::{self, CreateOptions, GameplayFlavor, TargetPlatform, Template},
};
use imgui::{Condition, StyleColor as C, StyleVar as V, Ui, WindowFlags as W};
use std::{collections::HashMap, path::PathBuf};

const BLUE: [f32; 4] = [0.02, 0.40, 0.76, 1.];
const TEXT: [f32; 4] = [0.91, 0.91, 0.92, 1.];
const MUTED: [f32; 4] = [0.61, 0.62, 0.64, 1.];
fn gray(value: u8) -> [f32; 4] {
    let v = value as f32 / 255.;
    [v, v, v, 1.]
}

#[derive(Clone)]
struct PendingUpgrade {
    path: PathBuf,
    from: String,
}

pub struct Hub {
    pub dependencies: crate::dependencies::State,
    name: String,
    location: String,
    open_path: String,
    template: Template,
    gameplay: GameplayFlavor,
    target: TargetPlatform,
    creating: bool,
    open_requested: bool,
    pending_upgrade: Option<PendingUpgrade>,
    search: String,
    alphabetical: bool,
    recent: Vec<workspace::Recent>,
    versions: HashMap<PathBuf, Result<String, String>>,
    registry: PathBuf,
    fonts: Option<[imgui::FontId; 3]>,
    pub logo: Option<imgui::TextureId>,
    pub lockup: Option<imgui::TextureId>,
    /// Renderer-owned template artwork, in `Template::ALL` order. `None` is a
    /// preview that could not be decoded; the card falls back to a plain tile.
    pub previews: [Option<imgui::TextureId>; Template::ALL.len()],
    /// Hit targets the interaction test clicks: 0 Create, 1 Open in the popup,
    /// 2 New project, 3 Open project, 4 Projects, 5 the first recent row,
    /// 6-8 the template cards, 9-11 the gameplay flavors, 12 Cancel and
    /// 13 the target platform tile.
    #[cfg(test)]
    buttons: [[f32; 2]; 14],
    pub error: Option<String>,
}
impl Hub {
    pub fn new(error: Option<String>) -> Self {
        let recent = workspace::recent_projects();
        let error = error.or_else(|| recent.as_ref().err().cloned());
        let recent = recent.unwrap_or_default();
        let versions = recent
            .iter()
            .map(|r| (r.path.clone(), workspace::project_editor_version(&r.path)))
            .collect();
        Self {
            dependencies: crate::dependencies::State::new(&workspace::editor_home()),
            name: "New Game".into(),
            location: workspace::user_home()
                .join("Documents/Epok Projects")
                .to_string_lossy()
                .into(),
            open_path: String::new(),
            template: Template::Basic,
            gameplay: GameplayFlavor::default(),
            target: TargetPlatform::default(),
            creating: false,
            open_requested: false,
            pending_upgrade: None,
            search: String::new(),
            alphabetical: false,
            error,
            recent,
            versions,
            registry: workspace::user_data().join("RecentProjects.epokprefs"),
            fonts: None,
            logo: None,
            lockup: None,
            previews: [None; Template::ALL.len()],
            #[cfg(test)]
            buttons: [[0.; 2]; 14],
        }
    }

    /// Opens on the template browser instead of the project list. Visual QA of
    /// the New Project view needs it, because nothing else reaches that view
    /// without a click.
    pub fn open_new_project(&mut self) {
        self.creating = true;
    }

    /// Separate Hub typography; the dense editor keeps its own font and style.
    pub fn load_fonts(&mut self, context: &mut imgui::Context) {
        let regular = std::fs::read("C:/Windows/Fonts/segoeui.ttf").ok();
        let semibold = std::fs::read("C:/Windows/Fonts/seguisb.ttf").ok();
        self.fonts = Some([18., 30., 22.].map(|size| {
            let data = if size == 18. {
                regular.as_ref()
            } else {
                semibold.as_ref().or(regular.as_ref())
            };
            match data {
                Some(data) => context.fonts().add_font(&[imgui::FontSource::TtfData {
                    data,
                    size_pixels: size,
                    config: None,
                }]),
                None => context
                    .fonts()
                    .add_font(&[imgui::FontSource::DefaultFontData {
                        config: Some(imgui::FontConfig {
                            size_pixels: size,
                            ..Default::default()
                        }),
                    }]),
            }
        }));
    }

    pub fn refresh(&mut self) {
        let fonts = self.fonts;
        let logo = self.logo;
        let lockup = self.lockup;
        let previews = self.previews;
        *self = Self::new(None);
        self.fonts = fonts;
        self.logo = logo;
        self.lockup = lockup;
        self.previews = previews;
    }

    fn heading(&self, ui: &Ui, text: &str, large: bool) {
        let _font = self
            .fonts
            .map(|fonts| ui.push_font(fonts[if large { 1 } else { 2 }]));
        ui.text(text);
    }

    pub fn draw(&mut self, ui: &Ui) -> Option<crate::loading::Request> {
        self.dependencies.poll();
        if self.dependencies.busy() {
            crate::busy_ui::draw(
                ui,
                "Installing components",
                &self.dependencies.progress(),
                false,
            );
            return None;
        }
        let mut result = None;
        let _font = self.fonts.map(|fonts| ui.push_font(fonts[0]));
        let _vars = [
            V::WindowPadding([24., 24.]),
            V::ItemSpacing([10., 12.]),
            V::FramePadding([12., 9.]),
            V::FrameRounding(5.),
            V::ChildRounding(10.),
            V::PopupRounding(8.),
            V::WindowBorderSize(0.),
            V::FrameBorderSize(1.),
        ]
        .map(|v| ui.push_style_var(v));
        let _colors = [
            (C::WindowBg, gray(18)),
            (C::ChildBg, gray(25)),
            (C::PopupBg, gray(28)),
            (C::Border, gray(49)),
            (C::Separator, gray(48)),
            (C::FrameBg, gray(38)),
            (C::FrameBgHovered, gray(45)),
            (C::FrameBgActive, gray(42)),
            (C::Button, gray(35)),
            (C::ButtonHovered, gray(49)),
            (C::ButtonActive, gray(58)),
            (C::Header, gray(46)),
            (C::HeaderHovered, gray(42)),
            (C::HeaderActive, gray(52)),
            (C::Text, TEXT),
            (C::TextDisabled, MUTED),
            (C::CheckMark, BLUE),
            (C::ScrollbarBg, gray(25)),
            (C::ScrollbarGrab, gray(65)),
            (C::NavHighlight, BLUE),
        ]
        .map(|(c, value)| ui.push_style_color(c, value));
        ui.window("Epok Projects")
            .position([0., 0.], Condition::Always)
            .size(ui.io().display_size, Condition::Always)
            .flags(
                W::NO_DECORATION
                    | W::NO_MOVE
                    | W::NO_DOCKING
                    | W::NO_SCROLLBAR
                    | W::NO_SAVED_SETTINGS,
            )
            .build(|| {
                let [width, height] = ui.io().display_size;
                let draw = ui.get_window_draw_list();
                draw.add_rect([0., 0.], [width, 184.], [0.063, 0.067, 0.075, 1.])
                    .filled(true)
                    .build();
                draw.add_line([24., 184.], [width - 24., 184.], gray(43))
                    .build();
                drop(draw);
                ui.set_cursor_pos([26., 2.]);
                if let Some(lockup) = self.lockup {
                    imgui::Image::new(lockup, [360., 180.]).build(ui);
                } else {
                    if let Some(logo) = self.logo {
                        imgui::Image::new(logo, [60., 60.]).build(ui);
                        ui.same_line_with_spacing(0., 14.);
                    }
                    self.heading(ui, "Epok Engine", true);
                }
                if width >= 820. {
                    ui.set_cursor_pos([width - 350., 61.]);
                    self.heading(ui, "Your next game starts here.", false);
                    ui.set_cursor_pos([width - 350., 96.]);
                    ui.text_disabled("Create for the original PlayStation.");
                    ui.set_cursor_pos([width - 350., 125.]);
                    ui.text_disabled(format!("Epok Engine  {}", env!("CARGO_PKG_VERSION")));
                }
                ui.set_cursor_pos([14., 200.]);
                let _side = ui.push_style_color(C::ChildBg, gray(18));
                ui.child_window("Navigation")
                    .size([196., (height - 216.).max(200.)])
                    .build(|| {
                        let _padding = ui.push_style_var(V::FramePadding([16., 12.]));
                        let _align = ui.push_style_var(V::ButtonTextAlign([0.10, 0.5]));
                        let _border = ui.push_style_var(V::FrameBorderSize(0.));
                        let _active = ui.push_style_color(
                            C::Button,
                            if self.creating { gray(18) } else { gray(47) },
                        );
                        if ui.button_with_size("Projects", [-1., 42.]) {
                            self.creating = false;
                        }
                        drop(_active);
                        #[cfg(test)]
                        {
                            self.buttons[4] = button_center(ui);
                        }
                        let _active = ui.push_style_color(
                            C::Button,
                            if self.creating { gray(47) } else { gray(18) },
                        );
                        if ui.button_with_size("New project", [-1., 42.]) {
                            self.creating = true;
                        }
                        #[cfg(test)]
                        {
                            self.buttons[2] = button_center(ui);
                        }
                        drop(_active);
                        if ui.button_with_size("Dependencies", [-1., 42.]) {
                            self.dependencies.open = true;
                        }
                        ui.set_cursor_pos([16., ui.window_size()[1] - 76.]);
                        ui.text_disabled("EPOK ENGINE");
                        ui.set_cursor_pos([16., ui.cursor_pos()[1]]);
                        ui.text_disabled(format!("Version {}", env!("CARGO_PKG_VERSION")));
                    });
                drop(_side);
                ui.set_cursor_pos([226., 200.]);
                ui.child_window("Project content")
                    .border(true)
                    .size([width - 242., (height - 216.).max(200.)])
                    .build(|| {
                        if self.creating {
                            result = self.create_view(ui);
                        } else {
                            result = self.projects_view(ui);
                        }
                    });
                if self.open_requested {
                    self.open_requested = false;
                    ui.open_popup("Open project folder");
                }
                self.open_dialog(ui, &mut result);
                self.upgrade_dialog(ui, &mut result);
            });
        self.dependencies.hub_window(ui);
        if self.dependencies.busy() {
            None
        } else {
            result
        }
    }

    fn projects_view(&mut self, ui: &Ui) -> Option<crate::loading::Request> {
        let mut result = None;
        let width = ui.content_region_avail()[0];
        let top = ui.cursor_pos();
        self.heading(ui, "Projects", true);
        ui.text_disabled("Your games, ready to build.");
        ui.set_cursor_pos([top[0] + width - 284., top[1] + 4.]);
        if ui.button_with_size("Open project", [130., 36.]) {
            self.open_requested = true;
        }
        #[cfg(test)]
        {
            self.buttons[3] = button_center(ui);
        }
        ui.same_line();
        if primary_button(ui, "+ New project", [144., 36.]) {
            self.creating = true;
        }
        ui.set_cursor_pos([top[0], top[1] + 82.]);
        ui.set_next_item_width((width - 242.).max(160.));
        ui.input_text("##Search projects", &mut self.search)
            .hint("Search projects by name or path")
            .build();
        ui.same_line();
        if ui.button_with_size(
            if self.alphabetical {
                "Sort: Name  A-Z"
            } else {
                "Sort: Recently opened"
            },
            [222., 35.],
        ) {
            self.alphabetical = !self.alphabetical;
        }
        ui.dummy([0., 3.]);
        let header = ui.cursor_pos();
        ui.text_disabled("NAME");
        ui.set_cursor_pos([header[0] + width - 264., header[1]]);
        ui.text_disabled("PLATFORM");
        ui.set_cursor_pos([header[0] + width - 144., header[1]]);
        ui.text_disabled("EDITOR");
        ui.separator();
        let query = self.search.trim().to_lowercase();
        let mut entries: Vec<_> = self
            .recent
            .iter()
            .filter(|r| {
                r.name.to_lowercase().contains(&query)
                    || r.path.to_string_lossy().to_lowercase().contains(&query)
            })
            .cloned()
            .collect();
        if self.alphabetical {
            entries.sort_by_key(|r| r.name.to_lowercase());
        }
        let count = entries.len();
        let footer = if self.error.is_some() { 118. } else { 38. };
        ui.child_window("Project list")
            .size([0., (ui.content_region_avail()[1] - footer).max(120.)])
            .build(|| {
                let _spacing = ui.push_style_var(V::ItemSpacing([10., 6.]));
                if entries.is_empty() {
                    ui.dummy([0., (ui.content_region_avail()[1] * 0.25).max(24.)]);
                    centered(
                        ui,
                        if self.recent.is_empty() {
                            "Your next game starts here"
                        } else {
                            "No projects found"
                        },
                        self.fonts.map(|f| f[2]),
                    );
                    ui.dummy([0., 2.]);
                    centered(
                        ui,
                        if self.recent.is_empty() {
                            "Create a new project or open an existing project folder."
                        } else {
                            "Try another name or clear your search."
                        },
                        None,
                    );
                    ui.dummy([0., 12.]);
                    ui.set_cursor_pos([
                        ((ui.window_size()[0] - 176.) * 0.5).max(0.),
                        ui.cursor_pos()[1],
                    ]);
                    if self.recent.is_empty() {
                        if primary_button(ui, "+ New project", [176., 38.]) {
                            self.creating = true;
                        }
                    } else if ui.button_with_size("Clear search", [176., 38.]) {
                        self.search.clear();
                    }
                }
                for (i, recent) in entries.iter().enumerate() {
                    let _id = ui.push_id_usize(i);
                    let start = ui.cursor_screen_pos();
                    let row_width = ui.content_region_avail()[0];
                    if ui
                        .selectable_config("##Open project row")
                        .size([row_width - 46., 65.])
                        .build()
                    {
                        result = self.request_open(recent.path.clone());
                    }
                    #[cfg(test)]
                    if i == 0 {
                        self.buttons[5] = button_center(ui);
                    }
                    if ui.is_item_hovered() {
                        ui.tooltip_text(format!("Open {}\n{}", recent.name, recent.path.display()));
                    }
                    let next = ui.cursor_screen_pos();
                    let draw = ui.get_window_draw_list();
                    let version = self.versions.get(&recent.path);
                    let missing = version.is_some_and(|v| v.is_err());
                    draw.add_rect(
                        [start[0] + 8., start[1] + 14.],
                        [start[0] + 44., start[1] + 50.],
                        if missing {
                            [0.29, 0.23, 0.13, 1.]
                        } else {
                            gray(44)
                        },
                    )
                    .filled(true)
                    .rounding(7.)
                    .build();
                    if missing {
                        draw.add_text([start[0] + 22., start[1] + 21.], [1., 0.72, 0.35, 1.], "!");
                    } else {
                        draw.add_rect(
                            [start[0] + 17., start[1] + 29.],
                            [start[0] + 35., start[1] + 40.],
                            MUTED,
                        )
                        .rounding(2.)
                        .build();
                        for (a, b) in [
                            ([17., 29.], [17., 25.]),
                            ([17., 25.], [24., 25.]),
                            ([24., 25.], [28., 29.]),
                        ] {
                            draw.add_line(
                                [start[0] + a[0], start[1] + a[1]],
                                [start[0] + b[0], start[1] + b[1]],
                                MUTED,
                            )
                            .build();
                        }
                    }
                    let name_width = row_width - 326.;
                    draw.add_text(
                        [start[0] + 58., start[1] + 10.],
                        TEXT,
                        ellipsis(ui, &recent.name, name_width),
                    );
                    draw.add_text(
                        [start[0] + 58., start[1] + 34.],
                        MUTED,
                        ellipsis(ui, &recent.path.to_string_lossy(), name_width),
                    );
                    draw.add_text(
                        [start[0] + row_width - 264., start[1] + 22.],
                        MUTED,
                        "PlayStation",
                    );
                    draw.add_text(
                        [start[0] + row_width - 144., start[1] + 22.],
                        if missing { [1., 0.72, 0.35, 1.] } else { MUTED },
                        version
                            .and_then(|v| v.as_ref().ok())
                            .map(String::as_str)
                            .unwrap_or("Unavailable"),
                    );
                    drop(draw);
                    ui.set_cursor_screen_pos([start[0] + row_width - 40., start[1] + 15.]);
                    if ui.button_with_size("...", [36., 32.]) {
                        ui.open_popup("Project options");
                    }
                    ui.popup("Project options", || {
                        if ui.menu_item("Open project") {
                            result = self.request_open(recent.path.clone());
                        }
                        if ui.menu_item("Open project folder") {
                            crate::project_browser::reveal(&recent.path);
                        }
                        ui.separator();
                        if ui.menu_item("Remove from list") {
                            match workspace::update_recent(&self.registry, &recent.path, None) {
                                Ok(()) => {
                                    self.recent.retain(|r| r.path != recent.path);
                                    self.versions.remove(&recent.path);
                                }
                                Err(error) => self.error = Some(error),
                            }
                        }
                        ui.text_disabled("Files stay on disk.");
                    });
                    ui.set_cursor_screen_pos(next);
                    ui.separator();
                }
            });
        ui.text_disabled(format!(
            "{count} project{}",
            if count == 1 { "" } else { "s" }
        ));
        self.error_banner(ui);
        result
    }

    fn create_view(&mut self, ui: &Ui) -> Option<crate::loading::Request> {
        let mut result = None;
        self.heading(ui, "New project", true);
        ui.text_disabled("Pick a starting point, then choose how its gameplay is written.");
        ui.dummy([0., 1.]);

        let available = ui.content_region_avail();
        // The project fields and the actions stay visible at every Hub size, so
        // the browser above them is what gives room back when the window
        // shrinks. The reservation is measured from the current style rather
        // than guessed, because a different Hub font would move every row.
        let line = ui.text_line_height_with_spacing();
        let field = ui.frame_height() + 12.;
        let footer =
            14. + 2. * field + line + 54. + if self.error.is_some() { 3. * line } else { 0. };
        let browser = (available[1] - footer).max(210.);
        let details = (available[0] * 0.46).clamp(268., 470.);
        let cards = (available[0] - details - 12.).max(190.);

        let top = ui.cursor_pos();
        {
            // The cards are their own frames, so the list itself carries no
            // padding: three of them fit before the column has to scroll.
            let _padding = ui.push_style_var(V::WindowPadding([0., 0.]));
            let _spacing = ui.push_style_var(V::ItemSpacing([10., 8.]));
            ui.child_window("Template cards")
                .size([cards, browser])
                .build(|| self.template_cards(ui));
        }
        ui.set_cursor_pos([top[0] + cards + 12., top[1]]);
        ui.child_window("Template details")
            .border(true)
            .size([details, browser])
            .build(|| self.template_details(ui));

        ui.set_cursor_pos([top[0], top[1] + browser + 8.]);
        ui.separator();
        let label = 116.;
        ui.align_text_to_frame_padding();
        ui.text("Location");
        ui.same_line_with_pos(top[0] + label);
        ui.set_next_item_width((ui.content_region_avail()[0] - 116.).max(120.));
        ui.input_text("##Project location", &mut self.location)
            .build();
        ui.same_line();
        if ui.button_with_size("Browse...", [106., ui.frame_height()]) {
            match pick_folder() {
                Ok(Some(path)) => self.location = path.to_string_lossy().into(),
                Err(e) => self.error = Some(e),
                _ => {}
            }
        }
        ui.align_text_to_frame_padding();
        ui.text("Name");
        ui.same_line_with_pos(top[0] + label);
        ui.set_next_item_width(-1.);
        let committed = ui
            .input_text("##Project name", &mut self.name)
            .hint("My game")
            .enter_returns_true(true)
            .build();
        let path = PathBuf::from(self.location.trim()).join(self.name.trim());
        let invalid = self.creation_error();
        let _muted = ui.push_style_color(C::Text, MUTED);
        ui.text_wrapped(match &invalid {
            Some(_) => "PROJECT FOLDER  —  unavailable".to_string(),
            None => format!("PROJECT FOLDER  {}", path.display()),
        });
        drop(_muted);
        if let Some(problem) = &invalid {
            let _color = ui.push_style_color(C::Text, [1., 0.69, 0.40, 1.]);
            ui.text_wrapped(problem.as_str());
        }
        ui.dummy([0., 2.]);
        if ui.button_with_size("Cancel", [100., 38.]) {
            self.creating = false;
        }
        #[cfg(test)]
        {
            self.buttons[12] = button_center(ui);
        }
        ui.same_line();
        let ready = invalid.is_none();
        let create = {
            let _disabled = ui.begin_disabled(!ready);
            primary_button(ui, "Create project", [160., 38.])
        };
        #[cfg(test)]
        {
            self.buttons[0] = button_center(ui);
        }
        // Enter creates from the name field. A popup owns the keyboard while it
        // is up, so the field cannot commit behind the open or upgrade dialog,
        // and the pending upgrade is checked anyway because it has its own
        // default action.
        let entered = committed && self.pending_upgrade.is_none();
        if ready && (create || entered) {
            let options = CreateOptions {
                template: self.template,
                gameplay: self.gameplay,
                target: self.target,
            };
            let creation = workspace::create_with_options(&path, self.name.trim(), options);
            result = self
                .activate(creation.map(|project| crate::loading::Request::Created(project.into())));
        }
        self.error_banner(ui);
        result
    }

    /// The actionable reason Create is unavailable, or `None` when it is ready.
    fn creation_error(&self) -> Option<String> {
        if self.location.trim().is_empty() {
            return Some("Choose a project location.".into());
        }
        if let Err(problem) = workspace::validate_name(self.name.trim()) {
            return Some(problem);
        }
        if !project_templates::info(self.template).supports(self.gameplay) {
            return Some(format!(
                "{} has no {} starter yet. Choose another gameplay flavor.",
                project_templates::info(self.template).title,
                self.gameplay.title()
            ));
        }
        None
    }

    fn template_cards(&mut self, ui: &Ui) {
        for (index, info) in project_templates::CATALOG.iter().enumerate() {
            let selected = self.template == info.template;
            let origin = ui.cursor_screen_pos();
            let width = ui.content_region_avail()[0].max(160.);
            let height = 76.;
            // One invisible button behind the whole card: the thumbnail, the
            // title and the blank space all select, and keyboard navigation
            // reaches the card because it is a real item.
            let pressed = ui.invisible_button(info.title, [width, height]);
            let hovered = ui.is_item_hovered();
            #[cfg(test)]
            {
                self.buttons[6 + index] = button_center(ui);
            }
            let end = [origin[0] + width, origin[1] + height];
            let draw = ui.get_window_draw_list();
            draw.add_rect(
                origin,
                end,
                if selected {
                    [0.08, 0.15, 0.22, 1.]
                } else if hovered {
                    gray(34)
                } else {
                    gray(28)
                },
            )
            .filled(true)
            .rounding(8.)
            .build();
            draw.add_rect(origin, end, if selected { BLUE } else { gray(53) })
                .rounding(8.)
                .thickness(if selected { 2. } else { 1. })
                .build();
            // 16:9 thumbnail, letterboxed into a fixed box so the row height
            // never changes when the Hub is resized.
            let thumb = [origin[0] + 8., origin[1] + 8.];
            let thumb_end = [thumb[0] + 107., thumb[1] + 60.];
            match self.previews[index] {
                Some(texture) => draw.add_image(texture, thumb, thumb_end).build(),
                None => draw
                    .add_rect(thumb, thumb_end, gray(40))
                    .filled(true)
                    .rounding(4.)
                    .build(),
            }
            let text = [thumb_end[0] + 12., origin[1] + 16.];
            draw.add_text(text, TEXT, info.title);
            draw.add_text([text[0], text[1] + 24.], MUTED, info.summary);
            drop(draw);
            if pressed {
                self.template = info.template;
            }
        }
    }

    fn template_details(&mut self, ui: &Ui) {
        let info = project_templates::info(self.template);
        let slot = project_templates::CATALOG
            .iter()
            .position(|entry| entry.template == self.template)
            .unwrap_or(0);
        let top = ui.cursor_pos();
        let panel = ui.content_region_avail();
        // The defaults own a fixed strip at the bottom of the panel and scroll
        // inside it, so the gameplay selector is reachable at every Hub size
        // however long the explanation under it wraps.
        let defaults = (panel[1] * 0.34).clamp(126., 196.);
        let width = panel[0].max(120.);
        let preview = (width * 9. / 16.).min((panel[1] * 0.30).max(68.));
        let origin = ui.cursor_screen_pos();
        ui.dummy([width, preview]);
        let end = [origin[0] + width, origin[1] + preview];
        let draw = ui.get_window_draw_list();
        match self.previews[slot] {
            // Letterboxed into the panel's width: the aspect never changes when
            // the Hub is resized, so neither does the composition.
            Some(texture) => {
                let scaled = preview * 16. / 9.;
                let inset = ((width - scaled) * 0.5).max(0.);
                draw.add_image(
                    texture,
                    [origin[0] + inset, origin[1]],
                    [end[0] - inset, end[1]],
                )
                .build()
            }
            None => {
                draw.add_rect(origin, end, gray(34))
                    .filled(true)
                    .rounding(6.)
                    .build();
                draw.add_text(
                    [origin[0] + 14., origin[1] + preview * 0.5 - 8.],
                    MUTED,
                    "Preview unavailable",
                );
            }
        }
        drop(draw);
        let prose = (panel[1] - defaults - preview - 14.).max(40.);
        {
            // Prose is denser than the surrounding controls, so a short panel
            // still shows the whole description before the list scrolls.
            let _spacing = ui.push_style_var(V::ItemSpacing([10., 4.]));
            ui.child_window("Template summary")
                .size([0., prose])
                .build(|| {
                    self.heading(ui, info.title, false);
                    ui.text_wrapped(info.description);
                    ui.dummy([0., 2.]);
                    let _muted = ui.push_style_color(C::Text, MUTED);
                    for feature in info.features {
                        ui.text_wrapped(format!("-  {feature}"));
                    }
                });
        }
        ui.set_cursor_pos([top[0], top[1] + panel[1] - defaults]);
        ui.separator();
        ui.child_window("Project defaults")
            .size([0., 0.])
            .build(|| self.project_defaults(ui, info));
    }

    /// Gameplay flavor and target platform. The controls come first so they
    /// stay visible even when the explanation below them wraps.
    fn project_defaults(&mut self, ui: &Ui, info: &project_templates::Info) {
        let _spacing = ui.push_style_var(V::ItemSpacing([10., 6.]));
        ui.text_disabled("PROJECT DEFAULTS  ·  GAMEPLAY");
        let segment = ((ui.content_region_avail()[0] - 16.) / 3.).max(52.);
        for (slot, flavor) in GameplayFlavor::ALL.into_iter().enumerate() {
            if slot > 0 {
                ui.same_line_with_spacing(0., 8.);
            }
            let selected = self.gameplay == flavor;
            let supported = info.supports(flavor);
            let _colors = [
                (C::Button, if selected { BLUE } else { gray(35) }),
                (
                    C::ButtonHovered,
                    if selected {
                        [0.05, 0.48, 0.88, 1.]
                    } else {
                        gray(49)
                    },
                ),
                (
                    C::ButtonActive,
                    if selected {
                        [0.02, 0.33, 0.65, 1.]
                    } else {
                        gray(58)
                    },
                ),
                (C::Border, if selected { BLUE } else { gray(53) }),
                (C::Text, if selected { [1.; 4] } else { TEXT }),
            ]
            .map(|(c, value)| ui.push_style_color(c, value));
            let _disabled = ui.begin_disabled(!supported);
            if ui.button_with_size(flavor.title(), [segment, 34.]) {
                self.gameplay = flavor;
            }
            #[cfg(test)]
            {
                self.buttons[9 + slot] = button_center(ui);
            }
        }
        ui.align_text_to_frame_padding();
        ui.text_disabled("TARGET");
        ui.same_line();
        // One target today. It is a selectable tile rather than a label so the
        // row already looks like the list it will become.
        let _colors = [
            (C::Button, [0.08, 0.15, 0.22, 1.]),
            (C::ButtonHovered, [0.08, 0.15, 0.22, 1.]),
            (C::ButtonActive, [0.08, 0.15, 0.22, 1.]),
            (C::Border, BLUE),
        ]
        .map(|(c, value)| ui.push_style_color(c, value));
        if ui.button_with_size(self.target.title(), [(segment * 1.4).max(108.), 30.]) {
            self.target = TargetPlatform::PlayStation;
        }
        #[cfg(test)]
        {
            self.buttons[13] = button_center(ui);
        }
        drop(_colors);
        let _muted = ui.push_style_color(C::Text, MUTED);
        ui.text_wrapped(info.gameplay_note(self.gameplay));
    }

    fn open_dialog(&mut self, ui: &Ui, result: &mut Option<crate::loading::Request>) {
        ui.popup("Open project folder", || {
            self.heading(ui, "Open project", false);
            ui.text_disabled("Select an .epokproject descriptor or its project folder.");
            ui.set_next_item_width(460.);
            ui.input_text("##Existing project folder", &mut self.open_path)
                .hint("Project folder or .epokproject file")
                .build();
            ui.same_line();
            if ui.button("File...") {
                match pick_descriptor() {
                    Ok(Some(path)) => self.open_path = path.to_string_lossy().into(),
                    Err(e) => self.error = Some(e),
                    _ => {}
                }
            }
            ui.same_line();
            if ui.button("Folder...") {
                match pick_folder() {
                    Ok(Some(path)) => self.open_path = path.to_string_lossy().into(),
                    Err(e) => self.error = Some(e),
                    _ => {}
                }
            }
            ui.dummy([0., 4.]);
            if ui.button_with_size("Cancel", [100., 36.]) {
                ui.close_current_popup();
            }
            ui.same_line();
            if primary_button(ui, "Open project", [140., 36.]) {
                *result = self.request_open(PathBuf::from(self.open_path.trim()));
                if result.is_some() {
                    ui.close_current_popup();
                }
            }
            #[cfg(test)]
            {
                self.buttons[1] = button_center(ui);
            }
            self.error_banner(ui);
        });
    }

    fn error_banner(&mut self, ui: &Ui) {
        if let Some(error) = self.error.clone() {
            let _color = ui.push_style_color(C::Text, [1., 0.69, 0.40, 1.]);
            ui.text_wrapped(error);
            if ui.small_button("Dismiss") {
                self.error = None;
            }
        }
    }

    fn request_open(&mut self, path: PathBuf) -> Option<crate::loading::Request> {
        match workspace::editor_version(&path) {
            Ok(workspace::EditorVersion::Older(from)) => {
                self.error = None;
                self.pending_upgrade = Some(PendingUpgrade { path, from });
                None
            }
            Ok(workspace::EditorVersion::Current | workspace::EditorVersion::CurrentOrNewer(_))
            | Err(_) => self.activate(Ok(crate::loading::Request::Open(path))),
        }
    }

    fn upgrade_dialog(&mut self, ui: &Ui, result: &mut Option<crate::loading::Request>) {
        if self.pending_upgrade.is_some() {
            ui.open_popup("Upgrade project?");
        }
        ui.modal_popup_config("Upgrade project?")
            .always_auto_resize(true)
            .build(|| {
                let Some(upgrade) = self.pending_upgrade.clone() else {
                    ui.close_current_popup();
                    return;
                };
                ui.text(format!("{} was created with Epok {}.", upgrade.path.display(), upgrade.from));
                ui.text_wrapped(format!(
                    "Upgrade its project descriptor to Epok {} before opening? A copy of the current descriptor will be kept in .epok/migrations.",
                    env!("CARGO_PKG_VERSION")
                ));
                ui.dummy([0., 4.]);
                if ui.button("Cancel") {
                    self.pending_upgrade = None;
                    ui.close_current_popup();
                }
                ui.same_line();
                if primary_button(ui, "Upgrade and open", [150., 36.]) {
                    match workspace::upgrade_editor_version(&upgrade.path) {
                        Ok(_) => {
                            self.versions.insert(
                                upgrade.path.clone(),
                                Ok(env!("CARGO_PKG_VERSION").into()),
                            );
                            *result = self.activate(Ok(crate::loading::Request::Open(upgrade.path)));
                            self.pending_upgrade = None;
                            ui.close_current_popup();
                        }
                        Err(error) => {
                            self.pending_upgrade = None;
                            self.error = Some(error);
                            ui.close_current_popup();
                        }
                    }
                }
            });
    }

    fn activate(
        &mut self,
        project: Result<crate::loading::Request, String>,
    ) -> Option<crate::loading::Request> {
        if self.dependencies.busy() {
            self.error =
                Some("Wait for dependency installation to finish before opening a project.".into());
            return None;
        }
        match project {
            Ok(editor) => {
                self.error = None;
                Some(editor)
            }
            Err(error) => {
                self.error = Some(error);
                None
            }
        }
    }
}

fn primary_button(ui: &Ui, label: &str, size: [f32; 2]) -> bool {
    let _colors = [
        (C::Button, BLUE),
        (C::ButtonHovered, [0.05, 0.48, 0.88, 1.]),
        (C::ButtonActive, [0.02, 0.33, 0.65, 1.]),
        (C::Border, BLUE),
        (C::Text, [1.; 4]),
    ]
    .map(|(c, value)| ui.push_style_color(c, value));
    ui.button_with_size(label, size)
}

fn centered(ui: &Ui, text: &str, font: Option<imgui::FontId>) {
    let _font = font.map(|f| ui.push_font(f));
    ui.set_cursor_pos([
        ((ui.window_size()[0] - ui.calc_text_size(text)[0]) * 0.5).max(0.),
        ui.cursor_pos()[1],
    ]);
    ui.text(text);
}

fn ellipsis(ui: &Ui, text: &str, width: f32) -> String {
    if ui.calc_text_size(text)[0] <= width {
        return text.into();
    }
    let mut value = text.to_string();
    while !value.is_empty() && ui.calc_text_size(format!("{value}..."))[0] > width {
        value.pop();
    }
    format!("{value}...")
}

fn pick_descriptor() -> Result<Option<PathBuf>, String> {
    #[cfg(windows)]
    {
        let mut command = std::process::Command::new("powershell.exe");
        command.args(["-NoProfile","-STA","-Command","Add-Type -AssemblyName System.Windows.Forms; [Console]::OutputEncoding=[System.Text.UTF8Encoding]::new(); $dialog=New-Object System.Windows.Forms.OpenFileDialog; $dialog.Title='Open Epok project'; $dialog.Filter='Epok project (*.epokproject)|*.epokproject'; $dialog.CheckFileExists=$true; if($dialog.ShowDialog() -eq 'OK'){[Console]::Write($dialog.FileName)}; $dialog.Dispose()"]);
        crate::pipeline::quiet(&mut command);
        let output = command
            .output()
            .map_err(|e| format!("Project file browser: {e}"))?;
        if !output.status.success() {
            return Err(
                "Could not open the file browser; enter the descriptor path instead.".into(),
            );
        }
        let path = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
        Ok((!path.is_empty()).then(|| PathBuf::from(path)))
    }
    #[cfg(not(windows))]
    {
        Err("Enter a descriptor path; the native picker currently supports Windows.".into())
    }
}
fn pick_folder() -> Result<Option<PathBuf>, String> {
    #[cfg(windows)]
    {
        let mut command = std::process::Command::new("powershell.exe");
        command.args(["-NoProfile", "-STA", "-Command", "Add-Type -AssemblyName System.Windows.Forms; [Console]::OutputEncoding = [System.Text.UTF8Encoding]::new(); $dialog = New-Object System.Windows.Forms.FolderBrowserDialog; $dialog.Description = 'Select an Epok project folder or location'; if ($dialog.ShowDialog() -eq 'OK') { [Console]::Write($dialog.SelectedPath) }; $dialog.Dispose()"]);
        crate::pipeline::quiet(&mut command);
        let output = command
            .output()
            .map_err(|e| format!("Folder browser: {e}"))?;
        if !output.status.success() {
            return Err("Could not open the folder browser. Enter the folder path instead.".into());
        }
        let path = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
        Ok((!path.is_empty()).then(|| PathBuf::from(path)))
    }
    #[cfg(not(windows))]
    {
        Err("Enter the folder path; the native picker currently supports Windows.".into())
    }
}

pub fn close_project_dialog(ui: &imgui::Ui, editor: &mut Editor) {
    if editor.hub_requested {
        editor.hub_requested = false;
        if editor.has_unsaved_changes() {
            ui.open_popup("Close project?");
        } else {
            editor.return_to_hub = true;
        }
    }
    ui.modal_popup_config("Close project?")
        .always_auto_resize(true)
        .build(|| {
            ui.text("Save scene and Blueprint changes before returning to Projects?");
            if ui.button("Save and close") && editor.save_all() {
                editor.return_to_hub = true;
                ui.close_current_popup();
            }
            ui.same_line();
            if ui.button("Discard changes") {
                editor.blueprint_editor.discard();
                editor.return_to_hub = true;
                ui.close_current_popup();
            }
            ui.same_line();
            if ui.button("Cancel") {
                ui.close_current_popup();
            }
        });
}

#[cfg(test)]
fn button_center(ui: &imgui::Ui) -> [f32; 2] {
    let min = ui.item_rect_min();
    let max = ui.item_rect_max();
    [(min[0] + max[0]) * 0.5, (min[1] + max[1]) * 0.5]
}

/// Reuse the existing UI test's context: Dear ImGui permits one active context per process.
#[cfg(test)]
pub fn verify_interactions(context: &mut imgui::Context) {
    let parent = std::env::temp_dir().join(format!(
        "epok-hub-ui-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let mut hub = Hub::new(None);
    // Dependency warning interactions are covered separately with isolated config.
    hub.dependencies.warning = false;
    hub.load_fonts(context);
    context.fonts().build_rgba32_texture();
    hub.location = parent.to_string_lossy().into();
    hub.registry = parent.join("preferences/RecentProjects.epokprefs");
    hub.recent.clear();
    hub.error = None;
    let frame = |context: &mut imgui::Context, hub: &mut Hub| {
        let result = hub.draw(context.frame());
        context.render();
        result.map(|request| {
            let mut loading = crate::loading::Loading::new(request, hub.registry.clone());
            loading.start();
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
            loop {
                if let Some(result) = loading.poll() {
                    break Editor::open_prepared(result.expect("Project preparation").project)
                        .unwrap();
                }
                assert!(
                    std::time::Instant::now() < deadline,
                    "Project loader timed out"
                );
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
        })
    };
    let click = |context: &mut imgui::Context, hub: &mut Hub, button: usize| {
        frame(context, hub);
        frame(context, hub);
        context.io_mut().add_mouse_pos_event(hub.buttons[button]);
        frame(context, hub);
        context
            .io_mut()
            .add_mouse_button_event(imgui::MouseButton::Left, true);
        frame(context, hub);
        context
            .io_mut()
            .add_mouse_button_event(imgui::MouseButton::Left, false);
        frame(context, hub)
    };
    // Creation goes through the real button, shared service, editor constructor and recent registry.
    click(context, &mut hub, 2);
    assert!(hub.creating);

    // The whole card selects, not only its title: the click lands on the body,
    // to the right of the thumbnail and below the text.
    for (slot, template) in Template::ALL.into_iter().enumerate() {
        click(context, &mut hub, 6 + slot);
        assert_eq!(hub.template, template, "card {slot} did not select");
    }
    for (slot, flavor) in workspace::GameplayFlavor::ALL.into_iter().enumerate() {
        click(context, &mut hub, 9 + slot);
        assert_eq!(hub.gameplay, flavor, "flavor {slot} did not select");
    }
    // One target, and selecting it is a no-op rather than a hidden state change.
    click(context, &mut hub, 13);
    assert_eq!(hub.target, workspace::TargetPlatform::PlayStation);
    assert_eq!(workspace::TargetPlatform::ALL.len(), 1);

    // A preview that cannot be decoded renders the fallback tile instead of
    // taking the Hub down with it, and the whole view still lays out at the
    // smallest window Epok supports.
    hub.previews = [None; Template::ALL.len()];
    frame(context, &mut hub);
    let full = context.io().display_size;
    context.io_mut().display_size = [1024., 720.];
    frame(context, &mut hub);
    frame(context, &mut hub);
    context.io_mut().display_size = full;
    frame(context, &mut hub);

    // Neither an empty location nor an unusable name may create anything.
    let location = std::mem::take(&mut hub.location);
    assert!(click(context, &mut hub, 0).is_none());
    assert!(hub.creation_error().is_some());
    hub.location = location;
    let name = std::mem::replace(&mut hub.name, "  ".into());
    assert!(click(context, &mut hub, 0).is_none());
    hub.name = "bad/name".into();
    assert!(click(context, &mut hub, 0).is_none());
    assert!(!parent.join("bad").exists());
    hub.name = name;

    // Cancel returns to the project list with the selections untouched.
    click(context, &mut hub, 12);
    assert!(!hub.creating);
    assert_eq!(hub.template, Template::ThirdPerson);
    assert_eq!(hub.gameplay, workspace::GameplayFlavor::Lua);
    click(context, &mut hub, 2);
    assert!(hub.creating);
    hub.template = Template::Basic;
    hub.gameplay = workspace::GameplayFlavor::Cpp;
    assert!(hub.creation_error().is_none());

    let mut editor =
        click(context, &mut hub, 0).expect("Create project button should open the editor");
    assert_eq!(editor.scene.actors.len(), 1);
    let root = editor.root.clone();
    assert!(workspace::manifest_path(&root).unwrap().is_file());
    editor.scene.name = "Saved from project UI".into();
    editor.changed();
    editor.hub_requested = true;
    close_project_dialog(context.frame(), &mut editor);
    context.render();
    assert!(
        !editor.return_to_hub,
        "Unsaved changes must wait for a choice"
    );
    // Save against the project's configured startup path, then simulate the normal clean close.
    assert!(editor.save());
    let ui = context.frame();
    ui.modal_popup_config("Close project?")
        .build(|| ui.close_current_popup());
    context.render();
    editor.hub_requested = true;
    close_project_dialog(context.frame(), &mut editor);
    context.render();
    assert!(editor.return_to_hub);
    drop(editor);
    hub.open_path = root.to_string_lossy().into();
    click(context, &mut hub, 4);
    click(context, &mut hub, 3);
    let editor =
        click(context, &mut hub, 1).expect("Open project button should load the saved project");
    assert_eq!(editor.scene.name, "Saved from project UI");
    assert_eq!(
        workspace::read_recent(&hub.registry).unwrap().len(),
        1,
        "Recent opens must deduplicate"
    );
    drop(editor);
    // Recreating in the existing destination stays in the Hub and preserves the authored scene.
    click(context, &mut hub, 2);
    assert!(click(context, &mut hub, 0).is_none());
    assert!(hub.error.is_some());
    assert_eq!(
        crate::scene::Scene::load(&workspace::startup_scene(&root).unwrap())
            .unwrap()
            .name,
        "Saved from project UI"
    );
    // Render populated and empty search states at the minimum supported window size,
    // then open a filtered row through real input (including custom draw-list content).
    hub.error = None;
    hub.creating = false;
    hub.recent = workspace::read_recent(&hub.registry).unwrap();
    hub.versions
        .insert(root.clone(), Ok(env!("CARGO_PKG_VERSION").into()));
    let previous_size = context.io().display_size;
    context.io_mut().display_size = [1024., 720.];
    hub.search = "no matching project".into();
    frame(context, &mut hub);
    hub.search = "NEW GAME".into();
    hub.alphabetical = true;
    let editor = click(context, &mut hub, 5).expect("Filtered recent row should open the project");
    assert_eq!(editor.scene.name, "Saved from project UI");
    drop(editor);
    context.io_mut().display_size = previous_size;
    workspace::update_recent(&hub.registry, &root, None).unwrap();
    assert!(workspace::read_recent(&hub.registry).unwrap().is_empty());
    assert!(workspace::manifest_path(&root).unwrap().exists());
    std::fs::remove_dir_all(parent).unwrap();
}
