//! Native Content Browser. Paths are project-relative; selection never owns asset identity.
use crate::{assets, editor::Editor};
use imgui::{Condition, MouseButton, StyleColor as C, StyleVar as V, Ui};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

const PAYLOAD: &str = "EPOK_CONTENT";
const TEXT: [f32; 4] = [0.72, 0.72, 0.72, 1.];
const GOLD: [f32; 4] = [0.62, 0.49, 0.29, 1.];

#[derive(Serialize, Deserialize)]
#[serde(default)]
struct Preferences {
    favorites: BTreeSet<String>,
    collections: BTreeMap<String, BTreeSet<String>>,
    tile_size: f32,
    sidebar: f32,
    list: bool,
    hide_unprocessed: bool,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            favorites: BTreeSet::new(),
            collections: BTreeMap::new(),
            tile_size: 144.,
            sidebar: 205.,
            list: false,
            hide_unprocessed: true,
        }
    }
}
#[derive(Clone)]
struct Entry {
    path: String,
    name: String,
    folder: bool,
}
#[derive(Default)]
pub struct State {
    pub font: Option<imgui::FontId>,
    pub previews: crate::content_preview::Cache,
    prefs: Preferences,
    loaded: bool,
    entries: Vec<Entry>,
    scanned: Option<Instant>,
    asset_revision: u64,
    history: Vec<String>,
    history_index: usize,
    selected: BTreeSet<String>,
    anchor: Option<String>,
    drag: Vec<String>,
    collection: Option<String>,
    tree_search: String,
    favorites_search: String,
    collection_search: String,
    show_tree_search: bool,
    show_favorites_search: bool,
    show_collection_search: bool,
    filter: usize,
    descending: bool,
    error: Option<String>,
    message: String,
    name: String,
    rename: Option<String>,
    import_path: String,
    new_folder: bool,
    new_collection: bool,
    request_rename: bool,
    request_import: bool,
    request_delete: bool,
    pending_scene: Option<PathBuf>,
    request_scene: bool,
    scene_open_error: Option<String>,
    trash: Vec<(String, PathBuf)>,
    pub focused: bool,
    pub hovered: bool,
    drawer: bool,
    dock_next: bool,
    dock_id: u32,
    pending_drop: Option<Vec<(String, String)>>,
    drop_menu: bool,
    command: String,
    undo_moves: Vec<Vec<(String, String)>>,
}
impl State {
    pub fn select_asset(e: &mut Editor, path: &Path) {
        let relative = crate::assets::path_string(&e.root, path);
        let folder = relative
            .rsplit_once('/')
            .map_or("assets", |(folder, _)| folder)
            .to_string();
        let mut state = std::mem::take(&mut e.project_browser);
        state.navigate(e, folder);
        state.selected.insert(relative.clone());
        state.anchor = Some(relative);
        state.filter = 0;
        e.selected_asset = Some(path.to_path_buf());
        e.focus_project = true;
        e.project_browser = state;
    }
    fn save(&mut self, root: &Path) {
        let result = (|| -> Result<(), String> {
            fs::create_dir_all(root.join("UserSettings")).map_err(|e| e.to_string())?;
            let bytes = crate::document::to_vec(&self.prefs).map_err(|e| e.to_string())?;
            fs::write(root.join("UserSettings/ContentBrowser.epokprefs"), bytes)
                .map_err(|e| e.to_string())
        })();
        if let Err(error) = result {
            self.error = Some(error);
        }
    }
    fn navigate(&mut self, e: &mut Editor, path: String) {
        self.collection = None;
        if e.assets.folder == path && !self.history.is_empty() {
            return;
        }
        self.history.truncate(self.history_index + 1);
        self.history.push(path.clone());
        self.history_index = self.history.len() - 1;
        e.assets.folder = path;
        self.selected.clear();
        e.selected_asset = None;
        e.project_search.clear();
    }
    fn travel(&mut self, e: &mut Editor, forward: bool) {
        let next = if forward {
            self.history_index.checked_add(1)
        } else {
            self.history_index.checked_sub(1)
        };
        if let Some(next) = next.filter(|i| *i < self.history.len()) {
            self.history_index = next;
            e.assets.folder = self.history[next].clone();
            self.collection = None;
            self.selected.clear();
            e.selected_asset = None;
            e.project_search.clear();
        }
    }
    fn refresh(&mut self, e: &mut Editor) {
        self.scanned = None;
        e.assets.refresh();
    }
    fn scan(&mut self, root: &Path) {
        if self.scanned.is_some() {
            return;
        }
        self.scanned = Some(Instant::now());
        self.entries.clear();
        fn walk(root: &Path, dir: &Path, entries: &mut Vec<Entry>) -> Result<(), String> {
            for item in fs::read_dir(dir).map_err(|e| e.to_string())? {
                let item = item.map_err(|e| e.to_string())?;
                let meta = fs::symlink_metadata(item.path()).map_err(|e| e.to_string())?;
                if meta.file_type().is_symlink() {
                    continue;
                }
                #[cfg(windows)]
                {
                    use std::os::windows::fs::MetadataExt;
                    if meta.file_attributes() & 0x400 != 0 {
                        continue;
                    }
                }
                let name = item.file_name().to_string_lossy().into_owned();
                if name.starts_with('.') {
                    continue;
                }
                entries.push(Entry {
                    path: assets::path_string(root, &item.path()),
                    name,
                    folder: meta.is_dir(),
                });
                if meta.is_dir() {
                    walk(root, &item.path(), entries)?;
                }
            }
            Ok(())
        }
        if let Err(error) = walk(root, &root.join("assets"), &mut self.entries) {
            self.error = Some(error);
        }
        self.entries
            .sort_by_key(|a| (!a.folder, a.name.to_lowercase(), a.path.clone()));
    }
}

fn kind<'a>(e: &'a Editor, entry: &Entry) -> &'a str {
    if entry.folder {
        return "Folder";
    }
    let p = e.root.join(&entry.path);
    if let Some(r) = e
        .assets
        .index
        .assets
        .values()
        .flatten()
        .find(|r| r.path == p)
    {
        return match r.meta.kind {
            assets::Kind::AudioClip | assets::Kind::MusicSequence => "Audio",
            assets::Kind::SoundBank => "SoundBank",
            assets::Kind::Texture => "Texture",
            assets::Kind::EditableMesh | assets::Kind::SkeletalMesh | assets::Kind::ModelSource => {
                "Mesh"
            }
            assets::Kind::AnimationClip => "Animation",
            assets::Kind::Material => "Material",
            assets::Kind::Skeleton => "Skeleton",
        };
    }
    if entry.path.ends_with(".epokbp") {
        "Blueprint"
    } else if entry.path.ends_with(".epokmap") {
        "Scene"
    } else if entry.path.ends_with(".timeline.json") {
        "Timeline"
    } else if entry.path.ends_with(".particle-effect.json") {
        "Particle Effect"
    } else if entry.path.ends_with(".epokasset") {
        "Asset"
    } else {
        crate::content_preview::source_kind(Path::new(&entry.path)).unwrap_or("Source")
    }
}
fn unprocessed(entry: &Entry) -> bool {
    !entry.folder && crate::content_preview::source_kind(Path::new(&entry.path)).is_some()
}
fn entry_icon(e: &Editor, entry: &Entry) -> (&'static str, [f32; 4]) {
    let (icon, color) = type_icon(kind(e, entry));
    (
        icon,
        if unprocessed(entry) {
            [0.57, 0.58, 0.60, 1.]
        } else {
            color
        },
    )
}
fn request_preview(e: &Editor, s: &mut State, entry: &Entry) -> PathBuf {
    s.previews.ensure_project(&e.root, &e.assets.index);
    let path = e.root.join(&entry.path);
    if matches!(kind(e, entry), "Texture" | "Audio") {
        let revision = crate::content_preview::revision(&e.root, &e.assets.index, &path);
        s.previews.request(&path, revision);
    }
    path
}
// The file name is an identity on disk; the browser presents the asset's name.
fn display_name(entry: &Entry) -> &str {
    if !entry.folder {
        for suffix in [
            ".particle-effect.json",
            ".timeline.json",
            ".epokasset",
            ".epokbp",
            ".epokmap",
        ] {
            if let Some(name) = entry.name.strip_suffix(suffix) {
                return name;
            }
        }
    }
    &entry.name
}

// Embedded Font Awesome glyphs, shared by tile and list views.
pub const ASSET_ICON_RANGES: &[u32] = &[
    0xf001, 0xf001, 0xf008, 0xf008, 0xf03e, 0xf03e, 0xf042, 0xf042, 0xf06d, 0xf06d, 0xf07b, 0xf07b,
    0xf0e8, 0xf0e8, 0xf121, 0xf121, 0xf15b, 0xf15b, 0xf1b2, 0xf1b2, 0xf279, 0xf279, 0xf550, 0xf550,
    0xf5d7, 0xf5d7, 0,
];
fn type_icon(kind: &str) -> (&'static str, [f32; 4]) {
    match kind {
        "Folder" => ("\u{f07b}", GOLD),
        "Blueprint" => ("\u{f0e8}", [0.24, 0.64, 0.90, 1.]),
        "Scene" => ("\u{f279}", [0.40, 0.68, 0.86, 1.]),
        "Texture" => ("\u{f03e}", [0.46, 0.73, 0.39, 1.]),
        "Mesh" => ("\u{f1b2}", [0.46, 0.72, 0.80, 1.]),
        "Skeleton" => ("\u{f5d7}", [0.81, 0.78, 0.63, 1.]),
        "Audio" => ("\u{f001}", [0.73, 0.48, 0.87, 1.]),
        "SoundBank" => ("\u{f001}", [0.54, 0.65, 0.88, 1.]),
        "Animation" => ("\u{f008}", [0.90, 0.62, 0.34, 1.]),
        "Material" => ("\u{f042}", [0.40, 0.78, 0.65, 1.]),
        "Timeline" => ("\u{f550}", [0.65, 0.57, 0.89, 1.]),
        "Particle Effect" => ("\u{f06d}", [0.92, 0.48, 0.27, 1.]),
        "Source" => ("\u{f121}", [0.66, 0.68, 0.71, 1.]),
        _ => ("\u{f15b}", [0.66, 0.68, 0.71, 1.]),
    }
}
const FILTERS: [&str; 15] = [
    "All assets",
    "Folder",
    "Blueprint",
    "Scene",
    "Texture",
    "Mesh",
    "Skeleton",
    "Audio",
    "Animation",
    "Timeline",
    "Particle Effect",
    "Material",
    "Asset",
    "Source",
    "SoundBank",
];

fn visible(s: &State, e: &Editor) -> Vec<Entry> {
    let query = e.project_search.to_lowercase();
    let mut entries = s
        .entries
        .iter()
        .filter(|item| {
            let location = if let Some(collection) = &s.collection {
                s.prefs
                    .collections
                    .get(collection)
                    .is_some_and(|paths| paths.contains(&item.path))
            } else if query.is_empty() {
                Path::new(&item.path).parent() == Some(Path::new(&e.assets.folder))
            } else {
                Path::new(&item.path).starts_with(&e.assets.folder)
            };
            location
                && item.name.to_lowercase().contains(&query)
                && (s.filter == 0
                    || kind(e, item) == FILTERS[s.filter]
                    || (FILTERS[s.filter] == "Source" && unprocessed(item)))
                && (!s.prefs.hide_unprocessed || !unprocessed(item))
        })
        .cloned()
        .collect::<Vec<_>>();
    entries.sort_by(|a, b| {
        a.folder.cmp(&b.folder).reverse().then_with(|| {
            let order = a.name.to_lowercase().cmp(&b.name.to_lowercase());
            if s.descending { order.reverse() } else { order }
        })
    });
    entries
}

pub fn window(ui: &Ui, e: &mut Editor, asset_font: imgui::FontId) {
    let mut s = std::mem::take(&mut e.project_browser);
    if let Some(report) = s.previews.take_report() { s.message = report; }
    if let Some(error) = s.previews.error.take() {
        e.log(error);
    }
    let _font = s.font.map(|font| ui.push_font(font));
    if !s.loaded {
        if let Ok(bytes) = fs::read(e.root.join("UserSettings/ContentBrowser.epokprefs")) {
            match crate::document::from_slice::<Preferences>(&bytes) {
                Ok(prefs) => s.prefs = prefs,
                Err(error) => s.error = Some(error.to_string()),
            }
        }
        s.prefs.tile_size = s.prefs.tile_size.clamp(96., 240.);
        s.prefs.sidebar = s.prefs.sidebar.clamp(120., 480.);
        s.history.push(e.assets.folder.clone());
        s.loaded = true;
    }
    if s.asset_revision != e.assets.revision {
        s.asset_revision = e.assets.revision;
        s.scanned = None;
    }
    s.scan(&e.root);
    let colors = [
        (C::WindowBg, crate::gui::gray(35)),
        (C::ChildBg, crate::gui::gray(35)),
        (C::Text, TEXT),
        (C::Border, crate::gui::gray(17)),
        (C::Button, crate::gui::gray(35)),
        (C::ButtonHovered, crate::gui::gray(55)),
        (C::ButtonActive, crate::gui::gray(65)),
        (C::FrameBg, crate::gui::gray(15)),
        (C::Header, [0., 0.43, 0.85, 1.]),
        (C::HeaderHovered, [0.12, 0.25, 0.39, 1.]),
        (C::HeaderActive, [0., 0.4, 0.84, 1.]),
        (C::ScrollbarBg, crate::gui::gray(35)),
        (C::ScrollbarGrab, crate::gui::gray(81)),
    ];
    let _colors = colors.map(|(c, value)| ui.push_style_color(c, value));
    let _vars = [
        ui.push_style_var(V::WindowPadding([4., 3.])),
        ui.push_style_var(V::ItemSpacing([4., 3.])),
        ui.push_style_var(V::FramePadding([6., 3.])),
        ui.push_style_var(V::FrameBorderSize(0.)),
        ui.push_style_var(V::ScrollbarSize(9.)),
        ui.push_style_var(V::IndentSpacing(14.)),
    ];
    s.focused = false;
    s.hovered = false;
    let capture = std::env::args().any(|a| a == "--screenshot-content-browser");
    let mut window = ui.window("\u{eb30} Project###Project");
    if capture {
        window = window
            .position([0., 0.], Condition::Always)
            .size(ui.io().display_size, Condition::Always)
            .title_bar(false)
            .movable(false)
            .resizable(false);
        unsafe {
            imgui::sys::igSetNextWindowDockID(0, imgui::sys::ImGuiCond_Always as i32);
        }
    } else if s.drawer {
        let size = ui.io().display_size;
        window = window
            .position([10., (size[1] - 350.).max(30.)], Condition::Always)
            .size([size[0] - 20., 320.], Condition::Always)
            .title_bar(false)
            .movable(false);
        unsafe {
            imgui::sys::igSetNextWindowDockID(0, imgui::sys::ImGuiCond_Always as i32);
        }
    }
    if s.dock_next {
        if s.dock_id != 0 {
            unsafe {
                imgui::sys::igSetNextWindowDockID(s.dock_id, imgui::sys::ImGuiCond_Always as i32);
            }
        } else {
            e.reset_layout = true;
        }
        s.dock_next = false;
    }
    window.build(|| {
        let dock_id = unsafe { imgui::sys::igGetWindowDockID() };
        if dock_id != 0 {
            s.dock_id = dock_id;
        }
        s.focused =
            ui.is_window_focused_with_flags(imgui::WindowFocusedFlags::ROOT_AND_CHILD_WINDOWS);
        s.hovered =
            ui.is_window_hovered_with_flags(imgui::WindowHoveredFlags::ROOT_AND_CHILD_WINDOWS);
        toolbar(ui, e, &mut s);
        ui.separator();
        let mut avail = ui.content_region_avail();
        avail[1] = (avail[1] - 32.).max(20.);
        let side = s.prefs.sidebar.min((avail[0] - 150.).max(100.));
        let sources_background = ui.push_style_color(C::ChildBg, crate::gui::gray(26));
        ui.child_window("content-sources")
            .size([side, avail[1].max(20.)])
            .build(|| sidebar(ui, e, &mut s));
        drop(sources_background);
        ui.same_line_with_spacing(0., 0.);
        ui.invisible_button("resize-sources", [4., avail[1].max(20.)]);
        if ui.is_item_hovered() || ui.is_item_active() {
            ui.set_mouse_cursor(Some(imgui::MouseCursor::ResizeEW));
        }
        if ui.is_item_active() {
            s.prefs.sidebar = (s.prefs.sidebar + ui.io().mouse_delta[0]).clamp(120., 480.);
        }
        if ui.is_item_deactivated() {
            s.save(&e.root);
        }
        ui.same_line_with_spacing(0., 0.);
        ui.child_window("content-main")
            .size([0., avail[1].max(20.)])
            .build(|| {
                search_bar(ui, e, &mut s);
                let items = visible(&s, e);
                let height = (ui.content_region_avail()[1] - 27.).max(20.);
                ui.child_window("content-grid")
                    .size([0., height])
                    .build(|| {
                        if items.is_empty() {
                            ui.set_cursor_pos([20., 35.]);
                            ui.text_disabled(if e.project_search.is_empty() {
                                "This folder is empty"
                            } else {
                                "No matching items"
                            });
                        }
                        if s.prefs.list {
                            list(ui, e, &mut s, &items);
                        } else {
                            grid(ui, e, &mut s, &items, asset_font);
                        }
                        if ui.is_window_hovered()
                            && !ui.is_any_item_hovered()
                            && ui.is_mouse_clicked(MouseButton::Left)
                        {
                            s.selected.clear();
                            e.selected_asset = None;
                        }
                        if let Some(_popup) = ui.begin_popup_context_window() {
                            context_menu(ui, e, &mut s, None);
                        }
                        if s.focused && !ui.io().want_text_input {
                            shortcuts(ui, e, &mut s, &items);
                        }
                    });
                ui.separator();
                if let Some(error) = &s.error {
                    ui.text_colored(
                        [0.95, 0.48, 0.35, 1.],
                        ellipsis(ui, error, ui.content_region_avail()[0] - 35.),
                    );
                    if ui.is_item_hovered() {
                        ui.tooltip_text(error);
                    }
                    ui.same_line();
                    if ui.small_button("x##dismiss") {
                        s.error = None;
                    }
                } else {
                    ui.text(format!(
                        "{} items{}",
                        items.len(),
                        if s.selected.is_empty() {
                            String::new()
                        } else {
                            format!(" ({} selected)", s.selected.len())
                        }
                    ));
                    if !s.message.is_empty() {
                        ui.same_line();
                        ui.text_disabled(&s.message);
                    }
                }
            });
        footer(ui, e, &mut s);
        dialogs(ui, e, &mut s);
    });
    e.project_browser = s;
}

fn tool(ui: &Ui, label: &str, tip: &str) -> bool {
    let hit = ui.button(label);
    #[cfg(test)]
    track(ui, label);
    if ui.is_item_hovered() {
        ui.tooltip_text(tip);
    }
    hit
}
fn toolbar(ui: &Ui, e: &mut Editor, s: &mut State) {
    let width = ui.content_region_avail()[0];
    let row_y = ui.cursor_pos()[1];
    let _pad = ui.push_style_var(V::FramePadding([7., 6.]));
    {
        let _bg = ui.push_style_color(C::Button, crate::gui::gray(53));
        let position = ui.cursor_screen_pos();
        if ui.button_with_size("##content-add-button", [66., 24.]) {
            ui.open_popup("content-add");
        }
        ui.get_window_draw_list().add_text(
            [position[0] + 10., position[1] + 5.],
            [0.6, 0.82, 0.34, 1.],
            "\u{f067}",
        );
        ui.get_window_draw_list()
            .add_text([position[0] + 30., position[1] + 4.], TEXT, "Add");
        if ui.is_item_hovered() {
            ui.tooltip_text("Create content in this project");
        }
    }
    ui.same_line();
    if tool(
        ui,
        "\u{f56f} Import",
        "Import files into the current folder",
    ) {
        s.request_import = true;
    }
    ui.same_line();
    if tool(ui, "\u{f0c7} Save All", "Save all open documents (Ctrl+S)") {
        e.save_all();
    }
    ui.same_line();
    ui.disabled(s.history_index == 0, || {
        if tool(ui, "\u{f359}##back", "Back (Alt+Left)") {
            s.travel(e, false);
        }
    });
    ui.same_line_with_spacing(0., 0.);
    ui.disabled(s.history_index + 1 >= s.history.len(), || {
        if tool(ui, "\u{f35a}##forward", "Forward (Alt+Right)") {
            s.travel(e, true);
        }
    });
    ui.same_line();
    if tool(ui, "\u{f07b} All", "Browse project content") {
        s.navigate(e, "assets".into());
    }
    let components = e
        .assets
        .folder
        .split('/')
        .skip(1)
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let mut path = "assets".to_string();
    for name in std::iter::once("Content".to_string()).chain(components) {
        if name != "Content" {
            path.push('/');
            path.push_str(&name);
        }
        if ui.cursor_pos()[0] > width - 245. && width > 650. {
            break;
        }
        ui.same_line_with_spacing(0., 2.);
        ui.text("\u{f105}");
        ui.same_line_with_spacing(0., 2.);
        if tool(ui, &format!("{name}##{path}"), &path) {
            s.navigate(e, path.clone());
        }
        drop_target(ui, e, s, &path);
    }
    if width > 800. {
        ui.set_cursor_pos([width - 215., row_y]);
    } else {
        ui.new_line();
    }
    if tool(
        ui,
        "\u{f2d2} Dock in Layout",
        "Dock the Content Browser into the editor layout",
    ) {
        s.drawer = false;
        s.dock_next = true;
    }
    ui.same_line();
    if tool(ui, "\u{f013} Settings", "View options") {
        ui.open_popup("content-settings");
    }
    ui.popup("content-add", || creation_menu(ui, e, s));
    ui.popup("content-settings", || {
        ui.text_disabled("VIEW");
        if ui.radio_button_bool("Tiles", !s.prefs.list) {
            s.prefs.list = false;
            s.save(&e.root);
        }
        if ui.radio_button_bool("List", s.prefs.list) {
            s.prefs.list = true;
            s.save(&e.root);
        }
        ui.set_next_item_width(190.);
        if ui.slider("Thumbnail size", 96., 240., &mut s.prefs.tile_size) {
            s.save(&e.root);
        }
        if ui.checkbox("Hide unprocessed files", &mut s.prefs.hide_unprocessed) {
            s.save(&e.root);
            s.previews.stop();
        }
        #[cfg(test)]
        track(ui, "Hide unprocessed files");
        if ui.is_item_hovered() { ui.tooltip_text("Hide original images, audio and model files. Imported Epok assets and C++ scripts remain visible."); }
        ui.checkbox("Sort descending", &mut s.descending);
        ui.separator();
        if ui.menu_item("Open as Content Drawer") {
            s.drawer = true;
        }
        if ui.menu_item("Refresh (F5)") {
            s.refresh(e);
        }
        if ui.menu_item("Imports and conflicts...") {
            e.assets.window = true;
            e.assets.focus_tab = Some(0);
        }
        ui.checkbox("Auto compile", &mut e.auto_build);
    });
}

fn creation_menu(ui: &Ui, e: &mut Editor, s: &mut State) {
    if ui.menu_item("SoundBank...") { e.assets.begin_new_bank(); }
    if ui.menu_item("Retro Starter SoundBank (generated triangle)") { e.assets.start_starter_bank(); }
    if ui.menu_item("New Folder") {
        s.new_folder = true;
        s.name = "NewFolder".into();
    }
    ui.separator();
    if ui.menu_item("C++ Class...") {
        e.action("new-script");
    }
    if ui.menu_item("Blueprint Class...") {
        e.action("new-blueprint");
    }
    if ui.menu_item("Timeline") {
        if let Err(error) = e.timeline_editor.create(&e.root) {
            s.error = Some(error);
        }
        s.refresh(e);
    }
    if ui.menu_item("Particle Effect") {
        if let Err(error) = e.timeline_editor.create_effect(&e.root) {
            s.error = Some(error);
        }
        s.refresh(e);
    }
    ui.separator();
    if ui.menu_item("Install Timeline Adapters") {
        match crate::timeline_adapters::install(&e.root) {
            Ok(p) => e.log(p.display().to_string()),
            Err(error) => s.error = Some(error),
        }
        s.refresh(e);
    }
}

fn sidebar(ui: &Ui, e: &mut Editor, s: &mut State) {
    let _pad = ui.push_style_var(V::FramePadding([4., 0.]));
    let _space = ui.push_style_var(V::ItemSpacing([2., 1.]));
    let (favorites_open, search) = section(ui, "Favorites", false, false);
    if search {
        s.show_favorites_search = !s.show_favorites_search;
    }
    if s.show_favorites_search {
        ui.set_next_item_width(-1.);
        ui.input_text("##favorites-search", &mut s.favorites_search)
            .hint("Search favorites")
            .build();
    }
    if favorites_open {
        for folder in s.prefs.favorites.clone() {
            if !folder
                .to_lowercase()
                .contains(&s.favorites_search.to_lowercase())
            {
                continue;
            }
            if ui
                .selectable_config(format!("\u{f07b} {}##fav{folder}", display_folder(&folder)))
                .selected(e.assets.folder == folder && s.collection.is_none())
                .build()
            {
                s.navigate(e, folder.clone());
            }
            drop_target(ui, e, s, &folder);
            if let Some(_popup) = ui.begin_popup_context_item()
                && ui.menu_item("Remove from Favorites")
            {
                s.prefs.favorites.remove(&folder);
                s.save(&e.root);
            }
        }
    }
    ui.separator();
    let remaining = ui.content_region_avail()[1];
    let collection_height = if s.collection.is_some() || !s.prefs.collections.is_empty() {
        94.
    } else {
        30.
    };
    ui.child_window("folders")
        .size([0., (remaining - collection_height).max(45.)])
        .build(|| {
            let (open, search) = section(ui, e.project_name(), true, false);
            if search {
                s.show_tree_search = !s.show_tree_search;
            }
            if open {
                if s.show_tree_search {
                    ui.set_next_item_width(-1.);
                    ui.input_text("##folder-search", &mut s.tree_search)
                        .hint("Search folders")
                        .build();
                }
                if let Some(_all) = ui
                    .tree_node_config("\u{f07b} All")
                    .default_open(true)
                    .push()
                {
                    tree(ui, e, s, "assets");
                }
            }
        });
    ui.separator();
    let y = ui.cursor_pos()[1];
    let (open, search) = section(ui, "Collections", false, true);
    if search {
        s.show_collection_search = !s.show_collection_search;
    }
    let next = ui.cursor_pos();
    ui.set_cursor_pos([ui.content_region_avail()[0] - 49., y + 5.]);
    if ui.small_button("\u{f067}##collection") {
        s.new_collection = true;
        s.name.clear();
    }
    ui.set_cursor_pos(next);
    if s.show_collection_search {
        ui.set_next_item_width(-1.);
        ui.input_text("##collection-search", &mut s.collection_search)
            .hint("Search collections")
            .build();
    }
    if open {
        for name in s.prefs.collections.keys().cloned().collect::<Vec<_>>() {
            if !name
                .to_lowercase()
                .contains(&s.collection_search.to_lowercase())
            {
                continue;
            }
            if ui
                .selectable_config(format!(
                    "\u{f07b} {name} ({})##collection{name}",
                    s.prefs.collections[&name].len()
                ))
                .selected(s.collection.as_ref() == Some(&name))
                .build()
            {
                s.collection = Some(name.clone());
                s.selected.clear();
            }
            if let Some(target) = ui.drag_drop_target()
                && let Some(Ok(payload)) =
                    target.accept_payload::<u8, _>(PAYLOAD, imgui::DragDropFlags::empty())
                && payload.delivery
            {
                s.prefs
                    .collections
                    .entry(name.clone())
                    .or_default()
                    .extend(s.drag.clone());
                s.save(&e.root);
            }
            if let Some(_popup) = ui.begin_popup_context_item()
                && ui.menu_item("Remove Collection")
            {
                s.prefs.collections.remove(&name);
                s.collection = None;
                s.save(&e.root);
            }
        }
    }
}
fn section(ui: &Ui, name: &str, default_open: bool, _add: bool) -> (bool, bool) {
    let _button = ui.push_style_color(C::Button, [0.; 4]);
    let _bg = ui.push_style_color(C::Header, crate::gui::gray(47));
    let _hover = ui.push_style_color(C::HeaderHovered, crate::gui::gray(54));
    let _pad = ui.push_style_var(V::FramePadding([4., 7.]));
    let pos = ui.cursor_pos();
    let width = ui.content_region_avail()[0];
    let open = ui.collapsing_header(
        name,
        imgui::TreeNodeFlags::ALLOW_ITEM_OVERLAP
            | if default_open {
                imgui::TreeNodeFlags::DEFAULT_OPEN
            } else {
                imgui::TreeNodeFlags::empty()
            },
    );
    let next = ui.cursor_pos();
    ui.set_cursor_pos([pos[0] + width - 27., pos[1] + 4.]);
    let search = ui.small_button(format!("\u{f002}##{name}-search"));
    ui.set_cursor_pos(next);
    (open, search)
}
fn display_folder(path: &str) -> &str {
    if path == "assets" {
        "Content"
    } else {
        path.rsplit('/').next().unwrap_or(path)
    }
}
fn tree(ui: &Ui, e: &mut Editor, s: &mut State, path: &str) {
    let children = s
        .entries
        .iter()
        .filter(|a| a.folder && Path::new(&a.path).parent() == Some(Path::new(path)))
        .cloned()
        .collect::<Vec<_>>();
    let selected = e.assets.folder == path && s.collection.is_none();
    let mut flags = imgui::TreeNodeFlags::OPEN_ON_ARROW | imgui::TreeNodeFlags::SPAN_AVAIL_WIDTH;
    if children.is_empty() {
        flags |= imgui::TreeNodeFlags::LEAF;
    }
    if selected {
        flags |= imgui::TreeNodeFlags::SELECTED;
    }
    let _color = ui.push_style_color(C::Text, if selected { [1.; 4] } else { TEXT });
    let label = format!("   {}", display_folder(path));
    let token = ui
        .tree_node_config(path)
        .label::<&str, _>(&label)
        .flags(flags)
        .opened(e.assets.folder.starts_with(path), Condition::Appearing)
        .push();
    let row = ui.item_rect_min();
    ui.get_window_draw_list()
        .add_text([row[0] + 15., row[1]], GOLD, "\u{f07b}");
    if ui.is_item_clicked() && !ui.is_item_toggled_open() {
        s.navigate(e, path.into());
    }
    drop_target(ui, e, s, path);
    if path != "assets" {
        drag_source(ui, s, path);
    }
    if let Some(_popup) = ui.begin_popup_context_item() {
        context_menu(ui, e, s, Some(path));
    }
    if let Some(_token) = token {
        for child in children {
            if s.tree_search.is_empty()
                || child
                    .path
                    .to_lowercase()
                    .contains(&s.tree_search.to_lowercase())
                || s.entries.iter().any(|a| {
                    a.folder
                        && a.path.starts_with(&format!("{}/", child.path))
                        && a.name
                            .to_lowercase()
                            .contains(&s.tree_search.to_lowercase())
                })
            {
                tree(ui, e, s, &child.path);
            }
        }
    }
}
fn search_bar(ui: &Ui, e: &mut Editor, s: &mut State) {
    ui.set_cursor_pos([ui.cursor_pos()[0], ui.cursor_pos()[1] + 2.]);
    let _round = ui.push_style_var(V::FrameRounding(12.));
    ui.text("\u{f002}");
    ui.same_line_with_spacing(0., 3.);
    ui.set_next_item_width((ui.content_region_avail()[0] * 0.45).max(110.));
    ui.input_text("##content-search", &mut e.project_search)
        .hint("Search Content")
        .build();
    ui.same_line();
    if tool(
        ui,
        "\u{f0c7}##save-search",
        "Save these search results as a Collection",
    ) {
        s.selected = visible(s, e).into_iter().map(|a| a.path).collect();
        s.new_collection = true;
        s.name = e.project_search.clone();
    }
    ui.same_line();
    if tool(ui, "\u{f0b0} \u{f107}##filters", "Filter by asset type") {
        ui.open_popup("content-filters");
    }
    ui.popup("content-filters", || {
        for (i, name) in FILTERS.iter().enumerate() {
            if ui.menu_item_config(name).selected(s.filter == i).build() {
                s.filter = i;
            }
        }
        ui.separator();
        s.previews.mode_controls(ui);
    });
    if s.filter != 0 {
        ui.same_line();
        ui.text(FILTERS[s.filter]);
    }
    if !e.project_search.is_empty() {
        ui.same_line();
        if tool(ui, "\u{ea76}##clear", "Clear search") {
            e.project_search.clear();
        }
    }
}

fn ellipsis(ui: &Ui, text: &str, max: f32) -> String {
    if ui.calc_text_size(text)[0] <= max {
        return text.into();
    }
    let mut value = text.to_string();
    while !value.is_empty() && ui.calc_text_size(format!("{value}..."))[0] > max {
        value.pop();
    }
    format!("{value}...")
}
fn select(ui: &Ui, s: &mut State, items: &[Entry], path: &str) {
    if ui.io().key_shift
        && let Some(anchor) = &s.anchor
        && let (Some(a), Some(b)) = (
            items.iter().position(|p| &p.path == anchor),
            items.iter().position(|p| p.path == path),
        )
    {
        if !ui.io().key_ctrl {
            s.selected.clear();
        }
        s.selected
            .extend(items[a.min(b)..=a.max(b)].iter().map(|a| a.path.clone()));
    } else if ui.io().key_ctrl {
        if !s.selected.remove(path) {
            s.selected.insert(path.into());
        }
        s.anchor = Some(path.into());
    } else {
        s.selected = BTreeSet::from([path.into()]);
        s.anchor = Some(path.into());
    }
}
fn item_events(
    ui: &Ui,
    e: &mut Editor,
    s: &mut State,
    items: &[Entry],
    entry: &Entry,
    clicked: bool,
) {
    // Invisible tiles activate on release, but ImGui reports double-clicks on
    // the second press. Handle that press directly for both tiles and rows.
    let double_clicked = ui.is_item_hovered() && ui.is_mouse_double_clicked(MouseButton::Left);
    if clicked || double_clicked {
        select(ui, s, items, &entry.path);
        e.selected_asset = if s.selected.contains(&entry.path) {
            Some(e.root.join(&entry.path))
        } else {
            s.selected.first().map(|path| e.root.join(path))
        };
        e.assets.selected = e
            .assets
            .index
            .assets
            .values()
            .flatten()
            .find(|r| r.path == e.root.join(&entry.path))
            .map(|r| r.meta.id);
        if double_clicked {
            open(e, s, entry);
        }
    }
    if ui.is_item_hovered() {
        ui.tooltip_text(format!("{}\n{}", entry.path, kind(e, entry)));
    }
    drag_source(ui, s, &entry.path);
    if entry.folder {
        drop_target(ui, e, s, &entry.path);
    }
    if let Some(_popup) = ui.begin_popup_context_item() {
        if !s.selected.contains(&entry.path) {
            s.selected = BTreeSet::from([entry.path.clone()]);
        }
        context_menu(ui, e, s, Some(&entry.path));
    }
}
fn grid(ui: &Ui, e: &mut Editor, s: &mut State, items: &[Entry], asset_font: imgui::FontId) {
    let tile = s
        .prefs
        .tile_size
        .min((ui.content_region_avail()[1] - 20.).max(80.));
    let cols = (ui.content_region_avail()[0] / tile).floor().max(1.) as usize;
    let mut origin = ui.cursor_pos();
    origin[0] += 4.;
    origin[1] += 4.;
    let row_h = tile + 60.;
    for (index, entry) in items.iter().enumerate() {
        let pos = [
            origin[0] + (index % cols) as f32 * tile,
            origin[1] + (index / cols) as f32 * row_h,
        ];
        ui.set_cursor_pos(pos);
        let p = ui.cursor_screen_pos();
        let _id = ui.push_id(&entry.path);
        let clicked = ui.invisible_button("tile", [tile - 4., tile + 12.]);
        #[cfg(test)]
        track(ui, &entry.path);
        if !ui.is_item_visible() && !ui.is_item_active() {
            continue;
        }
        let hover = ui.is_item_hovered();
        let selected = s.selected.contains(&entry.path);
        if selected || hover {
            ui.get_window_draw_list()
                .add_rect(
                    p,
                    [p[0] + tile - 4., p[1] + tile + 12.],
                    if selected {
                        [0.0, 0.33, 0.70, 1.]
                    } else {
                        [0.19, 0.19, 0.19, 1.]
                    },
                )
                .filled(true)
                .build();
        }
        if entry.folder {
            folder_icon(ui, [p[0] + tile * 0.14, p[1] + 24.], tile * 0.72);
        } else {
            asset_card(ui, e, s, entry, asset_font, p, tile);
        }
        let name = ellipsis(ui, display_name(entry), tile - 16.);
        let size = ui.calc_text_size(&name);
        ui.get_window_draw_list().add_text(
            [p[0] + (tile - size[0]) * 0.5, p[1] + tile - 12.],
            if selected { [1.; 4] } else { TEXT },
            name,
        );
        let audio = kind(e, entry) == "Audio";
        let (play_min, play_max) = play_rect(p, tile);
        let over_play = audio && hover && ui.is_mouse_hovering_rect(play_min, play_max);
        if over_play {
            let path = e.root.join(&entry.path);
            if clicked {
                s.previews.toggle(&path);
            }
            ui.tooltip_text(if s.previews.active(&path) {
                "Stop preview"
            } else {
                "Play preview"
            });
        } else {
            item_events(ui, e, s, items, entry, clicked);
            if hover && let Some(error) = s.previews.failure(&e.root.join(&entry.path)) {
                ui.tooltip_text(format!("Preview unavailable: {error}"));
            }
        }
    }
    ui.set_cursor_pos([
        origin[0],
        origin[1] + items.len().div_ceil(cols) as f32 * row_h,
    ]);
    ui.dummy([1., 1.]);
}
fn play_rect(p: [f32; 2], tile: f32) -> ([f32; 2], [f32; 2]) {
    let size = if tile < 120. { 18. } else { 28. };
    let a = [p[0] + 24., p[1] + tile - 41. - size];
    (a, [a[0] + size, a[1] + size])
}
fn asset_card(
    ui: &Ui,
    e: &Editor,
    s: &mut State,
    entry: &Entry,
    font: imgui::FontId,
    p: [f32; 2],
    tile: f32,
) {
    use crate::content_preview::Preview;
    let path = if ui.is_item_visible() {
        request_preview(e, s, entry)
    } else {
        e.root.join(&entry.path)
    };
    let (icon, color) = entry_icon(e, entry);
    let lo = [p[0] + 20., p[1] + 24.];
    let hi = [p[0] + tile - 24., p[1] + tile - 36.];
    let d = ui.get_window_draw_list();
    d.add_rect(lo, hi, crate::gui::gray(35))
        .filled(true)
        .build();
    let preview = match s.previews.get(&path) {
        Some(Preview::Image {
            width,
            height,
            texture: Some(texture),
            ..
        }) => {
            // Transparency remains legible against a subtle checkerboard.
            let cell = 8.;
            for y in 0..((hi[1] - lo[1]) / cell).ceil() as usize {
                for x in 0..((hi[0] - lo[0]) / cell).ceil() as usize {
                    let a = [lo[0] + x as f32 * cell, lo[1] + y as f32 * cell];
                    d.add_rect(
                        a,
                        [(a[0] + cell).min(hi[0]), (a[1] + cell).min(hi[1])],
                        crate::gui::gray(if (x + y) % 2 == 0 { 43 } else { 51 }),
                    )
                    .filled(true)
                    .build();
                }
            }
            let scale = ((hi[0] - lo[0]) / *width as f32).min((hi[1] - lo[1]) / *height as f32);
            let size = [*width as f32 * scale, *height as f32 * scale];
            let a = [
                (lo[0] + hi[0] - size[0]) * 0.5,
                (lo[1] + hi[1] - size[1]) * 0.5,
            ];
            d.add_image(*texture, a, [a[0] + size[0], a[1] + size[1]])
                .build();
            true
        }
        Some(Preview::Audio { peaks, duration }) => {
            let compact = tile < 120.;
            let center = lo[1] + (hi[1] - lo[1]) * if compact { 0.20 } else { 0.40 };
            let width = hi[0] - lo[0] - 12.;
            let bars = (width / 3.).max(1.) as usize;
            for i in 0..bars {
                let from = i * peaks.len() / bars;
                let to = ((i + 1) * peaks.len() / bars)
                    .max(from + 1)
                    .min(peaks.len());
                let peak = peaks[from..to].iter().copied().fold(0f32, f32::max);
                let height = (peak * (hi[1] - lo[1]) * if compact { 0.15 } else { 0.28 }).max(0.75);
                let x = lo[0] + 6. + i as f32 * width / bars as f32;
                d.add_line([x, center - height], [x, center + height], color)
                    .thickness(1.5)
                    .build();
            }
            if tile >= 132. {
                d.add_text(
                    [lo[0] + 7., lo[1] + 4.],
                    TEXT,
                    format!("{}:{:02}", *duration as u32 / 60, *duration as u32 % 60),
                );
            }
            let progress = s.previews.progress(&path);
            if progress > 0. {
                d.add_rect(
                    [lo[0], hi[1] - 3.],
                    [lo[0] + (hi[0] - lo[0]) * progress, hi[1]],
                    [0.88, 0.78, 1., 1.],
                )
                .filled(true)
                .build();
            }
            true
        }
        _ => false,
    };
    d.add_rect([lo[0], hi[1]], [hi[0], hi[1] + 4.], color)
        .filled(true)
        .build();
    if preview {
        let size = ((hi[1] - lo[1]) * 0.30).clamp(12., 24.);
        d.add_rect(
            [hi[0] - size - 8., hi[1] - size - 7.],
            [hi[0] - 2., hi[1] - 1.],
            [0.10, 0.10, 0.10, 0.95],
        )
        .rounding(3.)
        .filled(true)
        .build();
        draw_asset_glyph(
            ui,
            font,
            icon,
            color,
            size,
            [hi[0] - size - 5., hi[1] - size - 5.],
        );
    } else {
        let size = (tile - 64.).clamp(16., 38.);
        let _font = ui.push_font(font);
        let text = ui
            .calc_text_size(icon)
            .map(|v| v * size / ui.current_font_size());
        draw_asset_glyph(
            ui,
            font,
            icon,
            color,
            size,
            [
                (lo[0] + hi[0] - text[0]) * 0.5,
                (lo[1] + hi[1] - text[1]) * 0.5,
            ],
        );
    }
    if kind(e, entry) == "Audio" {
        let (a, b) = play_rect(p, tile);
        let size = b[0] - a[0];
        let hover = ui.is_item_hovered() && ui.is_mouse_hovering_rect(a, b);
        d.add_rect(a, b, crate::gui::gray(if hover { 86 } else { 49 }))
            .rounding(4.)
            .filled(true)
            .build();
        if s.previews.loading(&path) {
            d.add_text([a[0] + 3., a[1] + (size - 16.) * 0.5], [1.; 4], "...");
        } else if s.previews.active(&path) {
            d.add_rect(
                [a[0] + size * 0.32, a[1] + size * 0.32],
                [a[0] + size * 0.68, a[1] + size * 0.68],
                [1.; 4],
            )
            .filled(true)
            .build();
        } else {
            d.add_triangle(
                [a[0] + size * 0.36, a[1] + size * 0.25],
                [a[0] + size * 0.36, a[1] + size * 0.75],
                [a[0] + size * 0.75, a[1] + size * 0.5],
                [1.; 4],
            )
            .filled(true)
            .build();
        }
        #[cfg(test)]
        CONTROLS.with(|controls| {
            controls.borrow_mut().insert(
                format!("Play##{}", entry.path),
                [a[0] + size * 0.5, a[1] + size * 0.5],
            );
        });
    }
}
fn draw_asset_glyph(
    ui: &Ui,
    font: imgui::FontId,
    icon: &str,
    color: [f32; 4],
    size: f32,
    at: [f32; 2],
) {
    let _font = ui.push_font(font);
    unsafe {
        imgui::sys::ImDrawList_AddText_FontPtr(
            imgui::sys::igGetWindowDrawList(),
            imgui::sys::igGetFont(),
            size,
            imgui::sys::ImVec2 { x: at[0], y: at[1] },
            imgui::ImColor32::from(color).to_bits(),
            icon.as_ptr().cast(),
            icon.as_ptr().add(icon.len()).cast(),
            0.,
            std::ptr::null(),
        );
    }
}
// Resolution independent native folder geometry, including the inset lip and warm face.
fn folder_icon(ui: &Ui, p: [f32; 2], width: f32) {
    let d = ui.get_window_draw_list();
    let h = width * 0.73;
    d.add_rect(
        [p[0] + 1., p[1] + 11.],
        [p[0] + width + 2., p[1] + h + 3.],
        [0.04, 0.04, 0.04, 0.9],
    )
    .filled(true)
    .build();
    d.add_rect(p, [p[0] + width * 0.43, p[1] + 19.], [0.52, 0.40, 0.23, 1.])
        .rounding(6.)
        .filled(true)
        .build();
    d.add_rect(
        [p[0], p[1] + 8.],
        [p[0] + width, p[1] + h],
        [0.53, 0.42, 0.25, 1.],
    )
    .rounding(4.)
    .filled(true)
    .build();
    d.add_rect_filled_multicolor(
        [p[0] + 1., p[1] + 17.],
        [p[0] + width - 1., p[1] + h - 1.],
        [0.65, 0.51, 0.30, 1.],
        [0.54, 0.43, 0.26, 1.],
        [0.57, 0.45, 0.27, 1.],
        [0.68, 0.54, 0.32, 1.],
    );
    d.add_polyline(
        vec![
            [p[0], p[1] + 17.],
            [p[0] + width * 0.35, p[1] + 17.],
            [p[0] + width * 0.44, p[1] + 8.],
            [p[0] + width - 4., p[1] + 8.],
        ],
        GOLD,
    )
    .thickness(1.)
    .build();
    d.add_line(
        [p[0], p[1] + h],
        [p[0] + width, p[1] + h],
        [0.35, 0.27, 0.16, 1.],
    )
    .build();
}
fn list(ui: &Ui, e: &mut Editor, s: &mut State, items: &[Entry]) {
    if let Some(_table) = ui.begin_table_with_flags(
        "content-list",
        4,
        imgui::TableFlags::ROW_BG | imgui::TableFlags::RESIZABLE,
    ) {
        ui.table_setup_column("Name");
        ui.table_setup_column_with(imgui::TableColumnSetup {
            name: "Type",
            flags: imgui::TableColumnFlags::WIDTH_FIXED,
            init_width_or_weight: 100.,
            ..Default::default()
        });
        ui.table_setup_column_with(imgui::TableColumnSetup {
            name: "Preview",
            flags: imgui::TableColumnFlags::WIDTH_FIXED,
            init_width_or_weight: 76.,
            ..Default::default()
        });
        ui.table_setup_column("Path");
        ui.table_headers_row();
        for entry in items {
            ui.table_next_row();
            ui.table_next_column();
            let (icon, color) = entry_icon(e, entry);
            ui.text_colored(color, icon);
            ui.same_line();
            let clicked = ui
                .selectable_config(format!("{}##{}", display_name(entry), entry.path))
                .selected(s.selected.contains(&entry.path))
                .allow_double_click(true)
                .build();
            #[cfg(test)]
            track(ui, &entry.path);
            item_events(ui, e, s, items, entry, clicked);
            ui.table_next_column();
            ui.text_disabled(kind(e, entry));
            ui.table_next_column();
            if kind(e, entry) == "Audio" {
                let path = e.root.join(&entry.path);
                let label = if s.previews.loading(&path) {
                    "Loading..."
                } else if s.previews.active(&path) {
                    "Stop"
                } else {
                    "Play"
                };
                if ui.small_button(format!("{label}##preview{}", entry.path)) {
                    s.previews.toggle(&path);
                }
                #[cfg(test)]
                track(ui, &format!("Play##{}", entry.path));
            }
            ui.table_next_column();
            ui.text_disabled(&entry.path);
        }
    }
}

pub fn open_path(e: &mut Editor, path: &Path) {
    let mut state = std::mem::take(&mut e.project_browser);
    open(
        e,
        &mut state,
        &Entry {
            path: assets::path_string(&e.root, path),
            name: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
            folder: path.is_dir(),
        },
    );
    e.project_browser = state;
}
fn open(e: &mut Editor, s: &mut State, entry: &Entry) {
    if entry.folder {
        s.navigate(e, entry.path.clone());
        return;
    }
    let path = e.root.join(&entry.path);
    let record = e.assets.index.usable().find(|r| r.path == path).cloned();
    if let Some(record) = record {
        e.assets.selected = Some(record.meta.id);
        match record.meta.kind {
            assets::Kind::EditableMesh => crate::mesh_editor::open(e, record, None),
            assets::Kind::Texture | assets::Kind::AudioClip | assets::Kind::MusicSequence | assets::Kind::SoundBank => {
                e.assets.focus_tab = Some(2);
                e.assets.operation_path = entry.path.clone();
                e.assets.window = true;
            }
            _ => crate::skeletal_ui::open(e, record),
        }
    } else {
        let result = if entry.path.ends_with(".epokbp") {
            e.blueprint_editor.open(&path)
        } else if entry.path.ends_with(".timeline.json")
            || entry.path.ends_with(".particle-effect.json")
        {
            e.timeline_editor.open(&path)
        } else if entry.path.ends_with(".epokmap") {
            request_scene_open(e, s, path)
        } else {
            e.open_code(&path, None);
            Ok(())
        };
        if let Err(error) = result {
            s.error = Some(error);
        }
    }
}
fn request_scene_open(e: &mut Editor, s: &mut State, path: PathBuf) -> Result<(), String> {
    if path == e.scene_path() {
        e.focus_scene = true;
        return Ok(());
    }
    if e.playing {
        return Err("Stop Play before opening another scene.".into());
    }
    s.error = None;
    if e.dirty {
        s.pending_scene = Some(path);
        s.request_scene = true;
        s.scene_open_error = None;
        Ok(())
    } else {
        e.begin_scene_open(path)
    }
}
fn finish_scene_open(e: &mut Editor, s: &mut State, save: bool) -> bool {
    if e.playing {
        s.scene_open_error = Some("Stop Play before opening another scene.".into());
        return false;
    }
    if save && !e.save() {
        s.scene_open_error =
            Some("Could not save the current scene. See Console for details.".into());
        return false;
    }
    let Some(path) = s.pending_scene.clone() else {
        return false;
    };
    // The worker validates before replacing any current scene data.
    match e.begin_scene_open(path) {
        Ok(()) => {
            s.pending_scene = None;
            s.scene_open_error = None;
            true
        }
        Err(error) => {
            s.scene_open_error = Some(error);
            false
        }
    }
}
fn drag_source(ui: &Ui, s: &mut State, path: &str) {
    if let Some(_source) = ui.drag_drop_source_config(PAYLOAD).begin_payload(1_u8) {
        s.drag = if s.selected.contains(path) {
            s.selected.iter().cloned().collect()
        } else {
            vec![path.into()]
        };
        ui.text(format!("{} item(s)", s.drag.len()));
        ui.text_disabled(display_folder(path));
    }
}
fn drop_target(ui: &Ui, e: &mut Editor, s: &mut State, folder: &str) {
    if let Some(target) = ui.drag_drop_target()
        && let Some(Ok(payload)) =
            target.accept_payload::<u8, _>(PAYLOAD, imgui::DragDropFlags::empty())
        && payload.delivery
    {
        let moves = s
            .drag
            .iter()
            .map(|path| (path.clone(), format!("{folder}/{}", display_folder(path))))
            .collect::<Vec<_>>();
        let _ = e;
        s.pending_drop = Some(moves);
        s.drop_menu = true;
    }
}
fn context_menu(ui: &Ui, e: &mut Editor, s: &mut State, path: Option<&str>) {
    if let Some(path) = path {
        let entry = s.entries.iter().find(|a| a.path == path).cloned();
        if ui.menu_item("Open")
            && let Some(entry) = &entry
        {
            open(e, s, entry);
        }
        if path != "assets" && ui.menu_item("Rename...    F2") {
            s.rename = Some(path.into());
            s.name = display_folder(path).into();
            s.request_rename = true;
        }
        if entry.as_ref().is_none_or(|a| a.folder) && ui.menu_item("Add to Favorites") {
            s.prefs.favorites.insert(path.into());
            s.save(&e.root);
        }
        if ui.menu_item("Copy path") {
            ui.set_clipboard_text(path);
        }
        if path != "assets" && ui.menu_item("Duplicate") {
            let selected = if s.selected.contains(path) {
                s.selected.clone()
            } else {
                BTreeSet::from([path.into()])
            };
            let copies = selected
                .iter()
                .map(|from| (from.clone(), duplicate_name(&e.root, from)))
                .collect::<Vec<_>>();
            match copy_paths(&e.root, &copies) {
                Ok(()) => {
                    s.refresh(e);
                    s.error = None;
                    s.selected = copies.into_iter().map(|(_, to)| to).collect();
                }
                Err(error) => s.error = Some(error),
            }
        }
        if path != "assets" && ui.menu_item("Delete...    Del") {
            if !s.selected.contains(path) {
                s.selected = BTreeSet::from([path.into()]);
            }
            s.request_delete = true;
        }
        if ui.menu_item("Show in Explorer") {
            reveal(&e.root.join(path));
        }
        if !s.prefs.collections.is_empty() {
            ui.menu("Add to Collection", || {
                for name in s.prefs.collections.keys().cloned().collect::<Vec<_>>() {
                    if ui.menu_item(&name) {
                        s.prefs
                            .collections
                            .entry(name)
                            .or_default()
                            .extend(s.selected.clone());
                        s.save(&e.root);
                    }
                }
            });
        }
        if let Some(name) = s.collection.clone()
            && ui.menu_item("Remove from Collection")
        {
            if let Some(paths) = s.prefs.collections.get_mut(&name) {
                paths.retain(|p| !s.selected.contains(p));
            }
            s.save(&e.root);
        }
        ui.separator();
    }
    creation_menu(ui, e, s);
    ui.separator();
    if ui.menu_item("Import...") {
        s.request_import = true;
    }
    if ui.menu_item("Refresh    F5") {
        s.refresh(e);
    }
    if ui
        .menu_item_config("Undo last move    Ctrl+Z")
        .enabled(!s.undo_moves.is_empty())
        .build()
    {
        undo(e, s);
    }
    if ui
        .menu_item_config("Restore last deleted content")
        .enabled(!s.trash.is_empty())
        .build()
    {
        match restore_trash(&e.root, &s.trash) {
            Ok(()) => {
                s.trash.clear();
                s.refresh(e);
                s.error = None;
            }
            Err(error) => s.error = Some(error),
        }
    }
}
fn duplicate_name(root: &Path, source: &str) -> String {
    let path = Path::new(source);
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    let suffix = if name.ends_with(".timeline.json") {
        ".timeline.json"
    } else if name.ends_with(".particle-effect.json") {
        ".particle-effect.json"
    } else {
        ""
    };
    let (stem, ext) = if !suffix.is_empty() {
        (&name[..name.len() - suffix.len()], suffix.to_string())
    } else if root.join(source).is_file()
        && let Some((stem, ext)) = name.rsplit_once('.')
    {
        (stem, format!(".{ext}"))
    } else {
        (name.as_ref(), String::new())
    };
    for index in 1.. {
        let candidate = format!(
            "{}/{}_Copy{}{ext}",
            path.parent().unwrap().to_string_lossy(),
            stem,
            if index == 1 {
                String::new()
            } else {
                index.to_string()
            }
        );
        if !root.join(&candidate).exists() {
            return candidate;
        }
    }
    unreachable!()
}
fn shortcuts(ui: &Ui, e: &mut Editor, s: &mut State, items: &[Entry]) {
    if ui.is_key_pressed(imgui::Key::Delete) && !s.selected.is_empty() {
        s.request_delete = true;
    }
    if ui.io().key_ctrl && ui.io().key_shift && ui.is_key_pressed(imgui::Key::N) {
        s.new_folder = true;
        s.name = "NewFolder".into();
    }
    if ui.io().key_ctrl && ui.io().mouse_wheel != 0. && ui.is_window_hovered() {
        s.prefs.tile_size = (s.prefs.tile_size + ui.io().mouse_wheel * 12.).clamp(96., 240.);
        s.save(&e.root);
    }
    if ui.io().key_ctrl && ui.is_key_pressed(imgui::Key::A) {
        s.selected = items.iter().map(|a| a.path.clone()).collect();
    }
    if ui.io().key_ctrl && ui.is_key_pressed(imgui::Key::Z) {
        undo(e, s);
    }
    if ui.io().key_alt && ui.is_key_pressed(imgui::Key::LeftArrow) {
        s.travel(e, false);
    }
    if ui.io().key_alt && ui.is_key_pressed(imgui::Key::RightArrow) {
        s.travel(e, true);
    }
    if ui.is_key_pressed(imgui::Key::Backspace)
        && let Some(parent) = Path::new(&e.assets.folder)
            .parent()
            .filter(|p| p.starts_with("assets"))
    {
        s.navigate(e, parent.to_string_lossy().replace('\\', "/"));
    }
    if ui.is_key_pressed(imgui::Key::F5) {
        s.refresh(e);
    }
    if ui.is_key_pressed(imgui::Key::F2) && s.selected.len() == 1 {
        s.rename = s.selected.first().cloned();
        s.name = display_folder(s.rename.as_ref().unwrap()).into();
        s.request_rename = true;
    }
    if ui.is_key_pressed(imgui::Key::Enter)
        && s.selected.len() == 1
        && let Some(entry) = items.iter().find(|a| s.selected.contains(&a.path))
    {
        open(e, s, entry);
    }
}
fn valid_name(name: &str) -> Result<&str, String> {
    let name = name.trim();
    if name.is_empty()
        || name == "."
        || name == ".."
        || name
            .chars()
            .any(|c| c.is_control() || "<>:\"/\\|?*".contains(c))
        || name.ends_with('.')
    {
        return Err("Enter a name without path separators or reserved characters.".into());
    }
    Ok(name)
}
fn footer(ui: &Ui, e: &mut Editor, s: &mut State) {
    ui.separator();
    let y = ui.cursor_pos()[1];
    let width = ui.content_region_avail()[0];
    let _padding = ui.push_style_var(V::FramePadding([6., 5.]));
    if tool(
        ui,
        "\u{f07b} Content Drawer",
        "Toggle docking for the Content Browser",
    ) {
        s.drawer = !s.drawer;
        if !s.drawer {
            s.dock_next = true;
        }
    }
    ui.same_line();
    if tool(ui, "\u{eb9b} Output Log", "Open the editor Console") {
        e.focus_console = true;
    }
    ui.same_line();
    if tool(ui, "\u{eae9} Cmd \u{f107}", "Console commands") {
        ui.open_popup("content-commands");
    }
    ui.popup("content-commands", || {
        for command in ["save", "build", "play", "stop", "refresh"] {
            if ui.menu_item(command) {
                if command == "refresh" {
                    s.refresh(e);
                } else {
                    e.action(command);
                }
            }
        }
    });
    if width > 600. {
        ui.same_line();
        ui.set_next_item_width((width - 690.).clamp(130., 268.));
        if ui
            .input_text("##content-command", &mut s.command)
            .hint("Enter Console Command")
            .enter_returns_true(true)
            .build()
        {
            let command = std::mem::take(&mut s.command);
            match command.trim() {
                "save" | "build" | "play" | "stop" => e.action(command.trim()),
                "refresh" => s.refresh(e),
                _ => {
                    e.log("Commands: save, build, play, stop, refresh");
                    e.focus_console = true;
                }
            }
        }
    }
    if width > 950. {
        ui.set_cursor_pos([width - 340., y]);
        if tool(
            ui,
            "\u{eb29} Derived Data  \u{f107}",
            "Inspect generated asset dependencies",
        ) {
            e.artifact_dependencies.open = true;
        }
        ui.same_line();
        if tool(ui, "\u{ea71}##imports-status", "Open Imports") {
            e.assets.window = true;
        }
        ui.same_line();
        ui.text_disabled("Source Control Off");
    }
}
fn dialogs(ui: &Ui, e: &mut Editor, s: &mut State) {
    if std::mem::take(&mut s.request_scene) {
        ui.open_popup("Open Scene");
    }
    ui.modal_popup_config("Open Scene")
        .always_auto_resize(true)
        .build(|| {
            ui.text("The current scene has unsaved changes.");
            if let Some(path) = &s.pending_scene {
                ui.text(format!(
                    "Open {}?",
                    path.file_stem().unwrap_or_default().to_string_lossy()
                ));
            }
            ui.text_disabled("Blueprint and Timeline edits will be kept.");
            if let Some(error) = &s.scene_open_error {
                ui.text_wrapped(error);
            }
            if ui.button("Save and Open") && finish_scene_open(e, s, true) {
                ui.close_current_popup();
            }
            #[cfg(test)]
            track(ui, "scene-save-open");
            ui.same_line();
            if ui.button("Discard and Open") && finish_scene_open(e, s, false) {
                ui.close_current_popup();
            }
            #[cfg(test)]
            track(ui, "scene-discard-open");
            ui.same_line();
            if ui.button("Cancel") {
                s.pending_scene = None;
                s.scene_open_error = None;
                ui.close_current_popup();
            }
            #[cfg(test)]
            track(ui, "scene-cancel-open");
        });
    if s.request_delete {
        ui.open_popup("Delete Content");
        s.request_delete = false;
    }
    ui.modal_popup_config("Delete Content")
        .always_auto_resize(true)
        .build(|| {
            ui.text(format!("Delete {} selected item(s)?", s.selected.len()));
            ui.text("References to these files will become missing.");
            ui.text_disabled("Files are retained in UserSettings/ContentTrash for recovery.");
            if let Some(error) = &s.error {
                ui.text_wrapped(error);
            }
            if ui.button("Cancel") {
                ui.close_current_popup();
            }
            ui.same_line();
            if ui.button("Move to Trash") {
                match trash_content(e, &s.selected) {
                    Ok(trash) => {
                        s.trash = trash;
                        s.selected.clear();
                        e.selected_asset = None;
                        s.previews.stop();
                        s.error = None;
                        s.refresh(e);
                        ui.close_current_popup();
                    }
                    Err(error) => s.error = Some(error),
                }
            }
        });
    if s.drop_menu {
        ui.open_popup("Move or Copy Content");
        s.drop_menu = false;
    }
    ui.popup("Move or Copy Content", || {
        let move_here = ui.menu_item("Move Here");
        #[cfg(test)]
        track(ui, "Move Here");
        if move_here && let Some(moves) = s.pending_drop.take() {
            perform_moves(e, s, moves);
        }
        if ui.menu_item("Copy Here")
            && let Some(moves) = s.pending_drop.take()
        {
            match copy_paths(&e.root, &moves) {
                Ok(()) => {
                    s.refresh(e);
                    s.message = "Content copied.".into();
                    s.error = None;
                }
                Err(error) => s.error = Some(error),
            }
        }
        if ui.menu_item("Cancel") {
            s.pending_drop = None;
        }
    });
    for (flag, title) in [
        (&mut s.new_folder, "New Folder"),
        (&mut s.new_collection, "New Collection"),
        (&mut s.request_rename, "Rename Content"),
        (&mut s.request_import, "Import Content"),
    ] {
        if *flag {
            ui.open_popup(title);
            *flag = false;
            s.error = None;
        }
    }
    for title in ["New Folder", "New Collection", "Rename Content"] {
        ui.modal_popup_config(title)
            .always_auto_resize(true)
            .build(|| {
                ui.set_next_item_width(320.);
                if ui.is_window_appearing() {
                    ui.set_keyboard_focus_here();
                }
                let enter = ui
                    .input_text("Name", &mut s.name)
                    .enter_returns_true(true)
                    .build();
                if let Some(error) = &s.error {
                    ui.text_wrapped(error);
                }
                if ui.button("Cancel") || ui.is_key_pressed(imgui::Key::Escape) {
                    ui.close_current_popup();
                }
                ui.same_line();
                if ui.button("OK") || enter {
                    let result =
                        valid_name(&s.name)
                            .map(str::to_owned)
                            .and_then(|name| match title {
                                "New Collection" => {
                                    if s.prefs.collections.contains_key(&name) {
                                        return Err("A collection with that name exists.".into());
                                    }
                                    s.prefs.collections.insert(name, s.selected.clone());
                                    s.save(&e.root);
                                    Ok(())
                                }
                                "Rename Content" => {
                                    let from =
                                        s.rename.clone().ok_or("Select an item to rename")?;
                                    let parent =
                                        Path::new(&from).parent().ok_or("Cannot rename Content")?;
                                    let to = format!("{}/{name}", parent.to_string_lossy());
                                    perform_moves(e, s, vec![(from, to)]);
                                    s.error.clone().map_or(Ok(()), Err)
                                }
                                _ => {
                                    let dest = assets::inside(
                                        &e.root,
                                        &format!("{}/{name}", e.assets.folder),
                                    )?;
                                    fs::create_dir(dest).map_err(|e| e.to_string())?;
                                    s.refresh(e);
                                    Ok(())
                                }
                            });
                    match result {
                        Ok(()) => {
                            s.error = None;
                            ui.close_current_popup();
                        }
                        Err(error) => s.error = Some(error),
                    }
                }
            });
    }
    ui.modal_popup_config("Import Content")
        .always_auto_resize(true)
        .build(|| {
            ui.text(format!("Destination: {}", e.assets.folder));
            ui.text_disabled("Drop files from Explorer onto this panel, or choose a source file.");
            ui.set_next_item_width(400.);
            ui.input_text("Source file", &mut s.import_path).build();
            ui.same_line();
            if ui.button("Browse...") {
                match pick_files() {
                    Ok(paths) => {
                        for path in paths {
                            import(e, s, &path);
                        }
                    }
                    Err(error) => s.error = Some(error),
                }
            }
            if let Some(error) = &s.error {
                ui.text_wrapped(error);
            }
            if ui.button("Close") {
                ui.close_current_popup();
            }
            ui.same_line();
            if ui.button("Import") {
                let path = PathBuf::from(s.import_path.trim().trim_matches('"'));
                if import(e, s, &path) {
                    ui.close_current_popup();
                }
            }
            ui.same_line();
            if ui.button("Review pending imports") {
                e.assets.window = true;
                e.assets.focus_tab = Some(0);
                ui.close_current_popup();
            }
        });
}

// Preflight the entire selection before touching the filesystem. Keep the journal reversible.
fn move_paths(root: &Path, moves: &[(String, String)]) -> Result<(), String> {
    let mut destinations = BTreeSet::new();
    for (from, to) in moves {
        if from == "assets" || from == to {
            return Err("Choose a different destination folder.".into());
        }
        let source = assets::inside(root, from)?;
        let dest = assets::inside(root, to)?;
        if !source.exists() {
            return Err(format!("{from} no longer exists. Refresh Content."));
        }
        if dest.exists() || !destinations.insert(to.to_lowercase()) {
            return Err(format!("{to} already exists; nothing was overwritten."));
        }
        if !dest.parent().is_some_and(Path::is_dir) {
            return Err("The destination folder no longer exists.".into());
        }
        if dest.starts_with(&source) {
            return Err("A folder cannot be moved into itself.".into());
        }
        if source.is_file() && source.extension() != dest.extension() {
            return Err("Keep the existing file extension.".into());
        }
        if moves
            .iter()
            .any(|(other, _)| other != from && Path::new(from).starts_with(other))
        {
            return Err("Select the parent folder or its contents, not both.".into());
        }
    }
    let mut done: Vec<(PathBuf, PathBuf)> = vec![];
    for (from, to) in moves {
        let source = root.join(from);
        let dest = root.join(to);
        if let Err(error) = fs::rename(&source, &dest) {
            let mut message = format!("Move failed: {error}");
            for (source, dest) in done.into_iter().rev() {
                if let Err(rollback) = fs::rename(&dest, &source) {
                    message.push_str(&format!(
                        "; could not restore {}: {rollback}",
                        source.display()
                    ));
                }
            }
            return Err(message);
        }
        done.push((source, dest));
    }
    Ok(())
}
/// Copies publish only after all sources can be cloned. Asset copies get new identities.
/// Code classes and composite model packages require their dedicated creation/import workflows.
fn copy_paths(root: &Path, copies: &[(String, String)]) -> Result<(), String> {
    let mut files = Vec::new();
    let mut dirs = BTreeSet::new();
    fn prepare(
        source: &Path,
        destination: &Path,
        files: &mut Vec<(PathBuf, Vec<u8>)>,
        dirs: &mut BTreeSet<PathBuf>,
    ) -> Result<(), String> {
        let meta = fs::symlink_metadata(source).map_err(|e| e.to_string())?;
        if meta.file_type().is_symlink() {
            return Err("Linked content cannot be copied.".into());
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if meta.file_attributes() & 0x400 != 0 {
                return Err("Linked content cannot be copied.".into());
            }
        }
        if meta.is_dir() {
            dirs.insert(destination.to_owned());
            for entry in fs::read_dir(source).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                prepare(
                    &entry.path(),
                    &destination.join(entry.file_name()),
                    files,
                    dirs,
                )?;
            }
        } else {
            let ext = source
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or_default();
            let bytes = match ext {
                "epokasset" => {
                    let mut package = assets::Package::load(source)?;
                    if !matches!(package.meta.kind, assets::Kind::AudioClip | assets::Kind::MusicSequence | assets::Kind::SoundBank | assets::Kind::Texture | assets::Kind::EditableMesh) { return Err("Reimport the FBX to create an independent copy of a composite model.".into()); }
                    package.meta.id = uuid::Uuid::new_v4(); package.bytes()?
                }
                "epokbp" | "cpp" | "hpp" | "h" | "hh" | "cc" => return Err("Use Add > C++ Class / Blueprint Class to create an independent class; class identities cannot be copied as files.".into()),
                // A map carries persistent identities: entity, actor and component
                // UUIDs and, when it has one, the class identity of its own
                // Blueprint. A byte copy would give two maps the same class, so a
                // duplicate is rebuilt with fresh identities and its embedded
                // references rewritten to the copies.
                "epokmap" => {
                    let scene = crate::scene::Scene::load_unresolved(source)?;
                    let name = destination
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned();
                    crate::document::to_vec(&scene.duplicate_document(&name))
                        .map_err(|e| e.to_string())?
                }
                _ if source.to_string_lossy().ends_with(".timeline.json") => {
                    let mut doc = crate::timeline::load(source)?; doc.id = uuid::Uuid::new_v4();
                    crate::document::to_vec(&doc).map_err(|e| e.to_string())?
                }
                _ if source.to_string_lossy().ends_with(".particle-effect.json") => {
                    let mut doc = crate::particle_effect::load(source)?; doc.id = uuid::Uuid::new_v4(); doc.timeline.id = uuid::Uuid::new_v4();
                    crate::document::to_vec(&doc).map_err(|e| e.to_string())?
                }
                _ => assets::read_bounded(source)?,
            };
            files.push((destination.to_owned(), bytes));
        }
        Ok(())
    }
    let mut targets = BTreeSet::new();
    for (from, to) in copies {
        let source = assets::inside(root, from)?;
        let destination = assets::inside(root, to)?;
        if destination.exists()
            || destination.starts_with(&source)
            || !targets.insert(to.to_lowercase())
        {
            return Err(
                "Choose a new destination; existing content will not be overwritten.".into(),
            );
        }
        if copies
            .iter()
            .any(|(other, _)| other != from && Path::new(from).starts_with(other))
        {
            return Err("Select the parent folder or its contents, not both.".into());
        }
        prepare(&source, &destination, &mut files, &mut dirs)?;
    }
    let mut created_files = Vec::new();
    let mut created_dirs = Vec::new();
    let result = (|| -> Result<(), String> {
        for dir in &dirs {
            fs::create_dir(dir).map_err(|e| e.to_string())?;
            created_dirs.push(dir);
        }
        for (path, bytes) in &files {
            assets::atomic_write(path, bytes, None)?;
            created_files.push(path);
        }
        Ok(())
    })();
    if result.is_err() {
        for path in created_files.into_iter().rev() {
            let _ = fs::remove_file(path);
        }
        for path in created_dirs.into_iter().rev() {
            let _ = fs::remove_dir(path);
        }
    }
    result
}
fn trash_content(
    e: &Editor,
    selected: &BTreeSet<String>,
) -> Result<Vec<(String, PathBuf)>, String> {
    if e.playing || e.assets.busy {
        return Err("Wait for Play or importing to finish.".into());
    }
    let mut paths = vec![];
    for path in selected {
        if path == "assets" {
            return Err("The Content root cannot be deleted.".into());
        }
        if selected
            .iter()
            .any(|parent| parent != path && Path::new(path).starts_with(parent))
        {
            continue;
        }
        let source = assets::inside(&e.root, path)?;
        if e.scene_path().starts_with(&source)
            || e.blueprint_editor.open
                && e.blueprint_editor
                    .path
                    .as_ref()
                    .is_some_and(|p| p.starts_with(&source))
            || e.timeline_editor.open
                && e.timeline_editor
                    .document_path()
                    .is_some_and(|p| p.starts_with(&source))
        {
            return Err(
                "Close this document before deleting it. The active scene cannot be deleted."
                    .into(),
            );
        }
        if !source.exists() {
            return Err(format!("{path} no longer exists."));
        }
        paths.push(path.clone());
    }
    let settings = e.root.join("UserSettings");
    fs::create_dir_all(&settings).map_err(|e| e.to_string())?;
    let canonical_root = fs::canonicalize(&e.root).map_err(|e| e.to_string())?;
    if !fs::canonicalize(&settings)
        .map_err(|e| e.to_string())?
        .starts_with(&canonical_root)
    {
        return Err("UserSettings points outside the project.".into());
    }
    let trash_root = settings.join("ContentTrash");
    fs::create_dir_all(&trash_root).map_err(|e| e.to_string())?;
    if !fs::canonicalize(&trash_root)
        .map_err(|e| e.to_string())?
        .starts_with(&canonical_root)
    {
        return Err("ContentTrash points outside the project.".into());
    }
    let folder = trash_root.join(uuid::Uuid::new_v4().to_string());
    fs::create_dir(&folder).map_err(|e| e.to_string())?;
    let mut journal = Vec::new();
    for (index, path) in paths.iter().enumerate() {
        let dest = folder.join(format!("{index}-{}", display_folder(path)));
        if let Err(error) = fs::rename(e.root.join(path), &dest) {
            let rollback = restore_trash(&e.root, &journal);
            return Err(format!("Deletion stopped: {error}. Recovery: {rollback:?}"));
        }
        journal.push((path.clone(), dest));
    }
    // Store original locations alongside the retained files, including across restarts.
    let manifest = serde_json::to_vec_pretty(&journal).map_err(|e| e.to_string())?;
    if let Err(error) = fs::write(folder.join("restore.json"), manifest) {
        let rollback = restore_trash(&e.root, &journal);
        return Err(format!(
            "Could not save recovery paths: {error}. Recovery: {rollback:?}"
        ));
    }
    Ok(journal)
}
fn restore_trash(root: &Path, journal: &[(String, PathBuf)]) -> Result<(), String> {
    let canonical_root = fs::canonicalize(root).map_err(|e| e.to_string())?;
    for (original, trashed) in journal {
        let dest = assets::inside(root, original)?;
        if dest.exists() {
            return Err(format!(
                "{original} already exists. Nothing was overwritten."
            ));
        }
        if !fs::canonicalize(trashed)
            .map_err(|e| e.to_string())?
            .starts_with(&canonical_root)
        {
            return Err("Trash entry points outside this project.".into());
        }
    }
    for (original, trashed) in journal {
        let dest = assets::inside(root, original)?;
        fs::create_dir_all(dest.parent().ok_or("Invalid original path")?)
            .map_err(|e| e.to_string())?;
        fs::rename(trashed, dest).map_err(|e| e.to_string())?;
    }
    Ok(())
}
fn perform_moves(e: &mut Editor, s: &mut State, moves: Vec<(String, String)>) {
    s.error = None;
    if e.assets.busy || e.playing {
        s.error = Some("Wait for importing or Play to finish before moving content.".into());
        return;
    }
    // Open editors retain source revisions and paths. Protect these until closed.
    if moves.iter().any(|(from, _)| {
        let source = e.root.join(from);
        e.scene_path().starts_with(&source)
            || e.blueprint_editor.open
                && e.blueprint_editor
                    .path
                    .as_ref()
                    .is_some_and(|p| p.starts_with(&source))
            || e.timeline_editor.open
                && e.timeline_editor
                    .document_path()
                    .is_some_and(|p| p.starts_with(&source))
    }) {
        s.error =
            Some("Close the document before moving it. The active scene cannot be moved.".into());
        return;
    }
    match move_paths(&e.root, &moves) {
        Ok(()) => {
            for (from, to) in &moves {
                if let Some(path) = &e.selected_asset
                    && let Ok(tail) = path.strip_prefix(e.root.join(from))
                {
                    e.selected_asset = Some(e.root.join(to).join(tail));
                }
                if let Ok(tail) = Path::new(&e.assets.folder).strip_prefix(from) {
                    e.assets.folder = Path::new(to)
                        .join(tail)
                        .to_string_lossy()
                        .replace('\\', "/");
                }
            }
            remap(s, &moves);
            s.selected = moves.iter().map(|(_, to)| to.clone()).collect();
            s.undo_moves.push(moves);
            s.message = "Moved. Ctrl+Z to undo.".into();
            s.refresh(e);
            s.save(&e.root);
        }
        Err(error) => s.error = Some(error),
    }
}
fn remap(s: &mut State, moves: &[(String, String)]) {
    let map = |path: &String| {
        for (from, to) in moves {
            if let Ok(tail) = Path::new(path).strip_prefix(from) {
                return Path::new(to)
                    .join(tail)
                    .to_string_lossy()
                    .replace('\\', "/")
                    .trim_end_matches('/')
                    .into();
            }
        }
        path.clone()
    };
    s.prefs.favorites = s.prefs.favorites.iter().map(map).collect();
    for paths in s.prefs.collections.values_mut() {
        *paths = paths.iter().map(map).collect();
    }
    s.history = s.history.iter().map(map).collect();
}
fn undo(e: &mut Editor, s: &mut State) {
    if let Some(moves) = s.undo_moves.pop() {
        let reverse = moves
            .iter()
            .rev()
            .map(|(from, to)| (to.clone(), from.clone()))
            .collect();
        perform_moves(e, s, reverse);
        if s.error.is_some() {
            s.undo_moves.push(moves);
        } else {
            s.undo_moves.pop();
            s.message = "Move undone.".into();
        }
    }
}
fn import(e: &mut Editor, s: &mut State, source: &Path) -> bool {
    let result = (|| -> Result<(), String> {
        if !source.is_file() {
            return Err("Choose a source file to import.".into());
        }
        let ext = source
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_lowercase();
        if !["png", "wav", "mp3", "flac", "ogg", "mid", "midi", "seq", "sep", "vab", "vh", "vb", "sf2", "sf3", "fbx"].contains(&ext.as_str()) {
            return Err("Supported import sources: PNG, WAV, MP3, FLAC, OGG and FBX.".into());
        }
        let name = source
            .file_name()
            .ok_or("Missing filename")?
            .to_string_lossy();
        let dest = assets::inside(&e.root, &format!("{}/{name}", e.assets.folder))?;
        if fs::canonicalize(source).ok() != fs::canonicalize(&dest).ok() {
            let mut input = fs::File::open(source).map_err(|e| e.to_string())?;
            let mut output = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&dest)
                .map_err(|e| format!("Import did not overwrite {}: {e}", dest.display()))?;
            if let Err(error) = std::io::copy(&mut input, &mut output) {
                drop(output);
                let _ = fs::remove_file(&dest);
                return Err(error.to_string());
            }
        }
        s.refresh(e);
        e.assets.window = true;
        e.assets.focus_tab = Some(0);
        e.assets.notification = false;
        s.message = format!("{name} ready in Imports");
        Ok(())
    })();
    match result {
        Ok(()) => {
            s.error = None;
            true
        }
        Err(error) => {
            s.error = Some(error);
            false
        }
    }
}
pub fn external_drop(e: &mut Editor, path: PathBuf) {
    if !e.project_browser.hovered {
        return;
    }
    let mut s = std::mem::take(&mut e.project_browser);
    import(e, &mut s, &path);
    e.project_browser = s;
}
/// Assign Project content to the object displayed in Inspector.
pub fn inspector_drop(_ui: &Ui, e: &mut Editor) {
    // The whole visible Inspector body is a target, including whitespace and
    // component fields. A custom rectangle does not introduce an overlay widget
    // that would steal normal clicks or change the layout.
    let delivered = unsafe {
        let window = imgui::sys::igGetCurrentWindow();
        let id = imgui::sys::igGetID_Str(c"assign-blueprint".as_ptr());
        if imgui::sys::igBeginDragDropTargetCustom((*window).InnerRect, id) {
            let payload = imgui::sys::igAcceptDragDropPayload(c"EPOK_CONTENT".as_ptr(), 0);
            let delivered = !payload.is_null() && (*payload).Delivery;
            imgui::sys::igEndDragDropTarget();
            delivered
        } else {
            false
        }
    };
    if delivered {
        let paths = e.project_browser.drag.clone();
        match assign_content(e, &paths) {
            Ok(()) => e.project_browser.error = None,
            Err(error) => {
                e.log(&error);
                e.project_browser.error = Some(error);
            }
        }
    }
}
fn assign_content(e: &mut Editor, paths: &[String]) -> Result<(), String> {
    if paths.len() != 1 || !paths[0].ends_with(".epokbp") {
        return Err(
            "Drop one Blueprint onto Inspector to assign its behaviour to the selected object."
                .into(),
        );
    }
    let path = assets::inside(&e.root, &paths[0])?;
    crate::blueprint_workflow::attach_asset(e, &path)
}
/// Accept the same Content payload on the real Scene viewport.
pub fn scene_drop(ui: &Ui, e: &mut Editor) {
    if let Some(target) = ui.drag_drop_target()
        && let Some(Ok(payload)) =
            target.accept_payload::<u8, _>(PAYLOAD, imgui::DragDropFlags::empty())
        && payload.delivery
    {
        let paths = e.project_browser.drag.clone();
        if let Err(error) = place_content(e, &paths) {
            e.log(&error);
            e.project_browser.error = Some(error);
        }
    }
}
fn place_content(e: &mut Editor, paths: &[String]) -> Result<(), String> {
    if e.playing {
        return Err("Stop Play before placing content.".into());
    }
    if paths.len() == 1 && paths[0].ends_with(".epokbp") {
        let path = assets::inside(&e.root, &paths[0])?;
        e.blueprint_editor.open(&path)?;
        e.refresh_scripts();
        return crate::blueprint_workflow::place_current(e, None);
    }
    let before = e.scene.clone();
    let mut scene = before.clone();
    for path in paths {
        let absolute = assets::inside(&e.root, path)?;
        let mut entity = crate::scene::Entity::cube(
            Path::new(path)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
        );
        entity.position = e.view.center;
        if path.ends_with(".timeline.json") {
            let asset = crate::timeline::load(&absolute)?;
            entity.kind = "Empty".into();
            entity.timeline = Some(crate::timeline_scene::Component {
                asset: Some(asset.id),
                ..Default::default()
            });
        } else if path.ends_with(".particle-effect.json") {
            let asset = crate::particle_effect::load(&absolute)?;
            entity.kind = "Empty".into();
            entity.particle_effect = Some(crate::particle_effect_scene::Component {
                asset: Some(asset.id),
                ..Default::default()
            });
        } else {
            let record = e
                .assets
                .index
                .usable()
                .find(|r| r.path == absolute)
                .ok_or("Import the source first, then drag its asset into Scene.")?;
            match record.meta.kind {
                assets::Kind::EditableMesh => {
                    entity.editable_mesh = Some(crate::mesh::Component::new(record.meta.id))
                }
                assets::Kind::SkeletalMesh => {
                    entity.skeletal_mesh = Some(crate::skeletal::Component::new(record.meta.id))
                }
                assets::Kind::ModelSource => {
                    let crate::import_settings::Settings::Fbx(settings) = &record.meta.settings
                    else {
                        return Err("Invalid model source".into());
                    };
                    let output = settings
                        .outputs
                        .get("SkeletalMesh/main")
                        .ok_or("Import a skeletal mesh before placing this model.")?;
                    entity.skeletal_mesh = Some(crate::skeletal::Component::new(output.id));
                }
                assets::Kind::AudioClip | assets::Kind::MusicSequence => {
                    entity.kind = "Empty".into();
                    entity.audio = Some(crate::audio::AudioSource {
                        clip: Some(record.meta.id),
                        ..Default::default()
                    });
                }
                assets::Kind::Texture => {
                    let selected = e
                        .selected
                        .ok_or("Select a mesh to apply this texture, then drop it in Scene.")?;
                    scene.entities[selected].material.texture = Some(record.meta.id);
                    continue;
                }
                _ => return Err("This asset cannot be placed directly in Scene.".into()),
            }
        }
        scene.entities.push(entity);
    }
    scene.validate()?;
    crate::mesh::resolve(&mut scene, &e.assets.index)?;
    crate::skeletal::resolve(&mut scene, &e.assets.index)?;
    crate::texture::resolve(&mut scene, &e.assets.index)?;
    let added = scene.entities.len() > before.entities.len();
    e.script_undo.push((before, scene.clone()));
    e.script_redo.clear();
    e.scene = scene;
    if added {
        e.selected = Some(e.scene.entities.len() - 1);
        e.selected_asset = None;
    }
    e.reveal_selected = true;
    e.reset_instance_baseline();
    e.changed();
    Ok(())
}
pub(crate) fn reveal(path: &Path) {
    #[cfg(windows)]
    {
        let _ = std::process::Command::new("explorer.exe")
            .arg(if path.is_dir() {
                path.to_string_lossy().into_owned()
            } else {
                format!("/select,{}", path.display())
            })
            .spawn();
    }
    #[cfg(not(windows))]
    {
        let _ = std::process::Command::new(if cfg!(target_os = "macos") {
            "open"
        } else {
            "xdg-open"
        })
        .arg(if path.is_dir() {
            path
        } else {
            path.parent().unwrap_or(path)
        })
        .spawn();
    }
}
fn pick_files() -> Result<Vec<PathBuf>, String> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let output = std::process::Command::new("powershell.exe").creation_flags(0x08000000)
            .args(["-NoProfile", "-STA", "-Command", "Add-Type -AssemblyName System.Windows.Forms; [Console]::OutputEncoding=[System.Text.UTF8Encoding]::new(); $d=New-Object System.Windows.Forms.OpenFileDialog; $d.Title='Import Content'; $d.Multiselect=$true; $d.Filter='Supported content|*.png;*.wav;*.mp3;*.flac;*.ogg;*.mid;*.midi;*.seq;*.sep;*.vab;*.vh;*.vb;*.sf2;*.sf3;*.fbx'; if($d.ShowDialog() -eq 'OK'){$d.FileNames | ForEach-Object {[Console]::WriteLine($_)}}; $d.Dispose()"])
            .output().map_err(|e| e.to_string())?;
        if !output.status.success() {
            return Err("Could not open the file picker. Enter a source path instead.".into());
        }
        Ok(String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .collect())
    }
    #[cfg(not(windows))]
    {
        Err("Enter a source path or drag files onto Content.".into())
    }
}

#[cfg(test)]
thread_local! { static CONTROLS: std::cell::RefCell<BTreeMap<String, [f32; 2]>> = const { std::cell::RefCell::new(BTreeMap::new()) }; }
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
pub fn verify_interactions(ctx: &mut imgui::Context, font: imgui::FontId) {
    let root = std::env::temp_dir().join(format!("epok-content-input-{}", uuid::Uuid::new_v4()));
    let project =
        crate::workspace::create(&root, "Content Input", crate::workspace::Template::Sample)
            .unwrap();
    fs::create_dir_all(root.join("assets/A/Empty")).unwrap();
    fs::create_dir(root.join("assets/B")).unwrap();
    let mut e = Editor::open(project).unwrap();
    e.auto_build = false;
    fn frame(ctx: &mut imgui::Context, e: &mut Editor, font: imgui::FontId) {
        let ui = ctx.frame();
        unsafe {
            imgui::sys::igSetNextWindowDockID(0, imgui::sys::ImGuiCond_Always as i32);
            imgui::sys::igSetNextWindowPos(
                imgui::sys::ImVec2 { x: 0., y: 0. },
                imgui::sys::ImGuiCond_Always as i32,
                imgui::sys::ImVec2 { x: 0., y: 0. },
            );
            imgui::sys::igSetNextWindowSize(
                imgui::sys::ImVec2 { x: 1100., y: 500. },
                imgui::sys::ImGuiCond_Always as i32,
            );
        }
        window(ui, e, font);
        unsafe {
            imgui::sys::igSetNextWindowDockID(0, imgui::sys::ImGuiCond_Always as i32);
            imgui::sys::igSetNextWindowPos(
                imgui::sys::ImVec2 { x: 1100., y: 0. },
                imgui::sys::ImGuiCond_Always as i32,
                imgui::sys::ImVec2 { x: 0., y: 0. },
            );
            imgui::sys::igSetNextWindowSize(
                imgui::sys::ImVec2 { x: 340., y: 850. },
                imgui::sys::ImGuiCond_Always as i32,
            );
        }
        crate::gui::inspector(ui, e);
        ctx.render();
    }
    fn point(label: &str) -> [f32; 2] {
        CONTROLS.with(|c| {
            *c.borrow().get(label).unwrap_or_else(|| {
                panic!(
                    "Missing control: {label}; available: {:?}",
                    c.borrow().keys().collect::<Vec<_>>()
                )
            })
        })
    }
    fn click(ctx: &mut imgui::Context, e: &mut Editor, font: imgui::FontId, label: &str) {
        ctx.io_mut().add_mouse_pos_event(point(label));
        frame(ctx, e, font);
        ctx.io_mut().add_mouse_button_event(MouseButton::Left, true);
        frame(ctx, e, font);
        ctx.io_mut()
            .add_mouse_button_event(MouseButton::Left, false);
        frame(ctx, e, font);
    }
    fn finish_loading(e: &mut Editor) {
        let started = std::time::Instant::now();
        while e.scene_loading.is_some() {
            assert!(started.elapsed().as_secs() < 10, "Scene loading timed out");
            e.tick();
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
    }
    fn double_click(ctx: &mut imgui::Context, e: &mut Editor, font: imgui::FontId, label: &str) {
        for _ in 0..25 {
            frame(ctx, e, font);
        }
        click(ctx, e, font, label);
        click(ctx, e, font, label);
        frame(ctx, e, font);
    }
    frame(ctx, &mut e, font);
    frame(ctx, &mut e, font);
    click(ctx, &mut e, font, "assets/A");
    assert_eq!(
        e.selected_asset,
        Some(e.root.join("assets/A")),
        "A single click selects the file/folder for the Inspector"
    );
    assert_eq!(
        e.project_browser.selected,
        BTreeSet::from(["assets/A".into()])
    );
    ctx.io_mut().add_key_event(imgui::Key::ModCtrl, true);
    frame(ctx, &mut e, font);
    click(ctx, &mut e, font, "assets/B");
    assert_eq!(
        e.project_browser.selected.len(),
        2,
        "Ctrl-click selects both folders"
    );
    ctx.io_mut().add_key_event(imgui::Key::ModCtrl, false);
    frame(ctx, &mut e, font);
    click(ctx, &mut e, font, "assets/A");
    ctx.io_mut().add_key_event(imgui::Key::Enter, true);
    frame(ctx, &mut e, font);
    ctx.io_mut().add_key_event(imgui::Key::Enter, false);
    frame(ctx, &mut e, font);
    assert_eq!(
        e.assets.folder, "assets/A",
        "Enter opens the selected folder"
    );
    click(ctx, &mut e, font, "\u{f359}##back");
    assert_eq!(e.assets.folder, "assets");
    frame(ctx, &mut e, font);
    let start = point("assets/A");
    let end = point("assets/B");
    ctx.io_mut().add_mouse_pos_event(start);
    frame(ctx, &mut e, font);
    ctx.io_mut().add_mouse_button_event(MouseButton::Left, true);
    frame(ctx, &mut e, font);
    for step in 1..=8 {
        let t = step as f32 / 8.;
        ctx.io_mut().add_mouse_pos_event([
            start[0] + (end[0] - start[0]) * t,
            start[1] + (end[1] - start[1]) * t,
        ]);
        frame(ctx, &mut e, font);
    }
    ctx.io_mut()
        .add_mouse_button_event(MouseButton::Left, false);
    frame(ctx, &mut e, font);
    frame(ctx, &mut e, font);
    assert!(
        e.project_browser.pending_drop.is_some(),
        "Dropping opens Move / Copy"
    );
    click(ctx, &mut e, font, "Move Here");
    assert!(
        root.join("assets/B/A/Empty").is_dir(),
        "Drag actually moves the folder on disk: {:?}",
        e.project_browser.error
    );
    let mut s = std::mem::take(&mut e.project_browser);
    undo(&mut e, &mut s);
    e.project_browser = s;
    assert!(root.join("assets/A/Empty").is_dir());
    let original_path = e.scene_path();
    let other_path = root.join("assets/B/Other.epokmap");
    let other = crate::scene::Scene {
        name: "Other Scene".into(),
        ..Default::default()
    };
    other.save(&other_path).unwrap();
    let other_path = fs::canonicalize(other_path).unwrap();
    e.assets.folder = "assets/B".into();
    e.project_browser.scanned = None;
    e.dirty = false;
    frame(ctx, &mut e, font);
    double_click(ctx, &mut e, font, "assets/B/Other.epokmap");
    finish_loading(&mut e);
    assert_eq!(
        e.scene_path(),
        other_path,
        "A real double-click on a tile opens the scene"
    );
    assert_eq!(e.scene.name, "Other Scene");
    assert!(e.focus_scene);
    e.open_scene(original_path.clone()).unwrap();
    e.project_browser.prefs.list = true;
    frame(ctx, &mut e, font);
    double_click(ctx, &mut e, font, "assets/B/Other.epokmap");
    finish_loading(&mut e);
    assert_eq!(
        e.scene_path(),
        other_path,
        "List rows open with double-click too"
    );
    e.open_scene(original_path.clone()).unwrap();
    e.project_browser.prefs.list = false;
    e.scene.name = "Unsaved edits".into();
    e.changed();
    let original_bytes = fs::read(&original_path).unwrap();
    double_click(ctx, &mut e, font, "assets/B/Other.epokmap");
    assert_eq!(e.scene_path(), original_path);
    assert_eq!(e.project_browser.pending_scene.as_ref(), Some(&other_path));
    click(ctx, &mut e, font, "scene-cancel-open");
    assert!(e.project_browser.pending_scene.is_none());
    assert_eq!(e.scene.name, "Unsaved edits");
    assert!(e.dirty);
    double_click(ctx, &mut e, font, "assets/B/Other.epokmap");
    click(ctx, &mut e, font, "scene-discard-open");
    finish_loading(&mut e);
    assert_eq!(e.scene_path(), other_path);
    assert_eq!(fs::read(&original_path).unwrap(), original_bytes);
    e.open_scene(original_path.clone()).unwrap();
    e.scene.name = "Saved by scene switch".into();
    e.changed();
    double_click(ctx, &mut e, font, "assets/B/Other.epokmap");
    click(ctx, &mut e, font, "scene-save-open");
    finish_loading(&mut e);
    assert_eq!(e.scene_path(), other_path);
    assert_eq!(
        crate::scene::Scene::load(&original_path).unwrap().name,
        "Saved by scene switch"
    );
    e.scene.name = "Keep these edits".into();
    e.changed();
    let preserved = e.scene.clone();
    let mut state = std::mem::take(&mut e.project_browser);
    request_scene_open(&mut e, &mut state, root.join("assets/B/Missing.epokmap")).unwrap();
    assert!(!finish_scene_open(&mut e, &mut state, false));
    assert_eq!(
        e.scene, preserved,
        "A failed load must never discard the current scene"
    );
    e.playing = true;
    assert!(!finish_scene_open(&mut e, &mut state, false));
    assert!(
        state
            .scene_open_error
            .as_ref()
            .unwrap()
            .contains("Stop Play")
    );
    assert_eq!(e.scene, preserved);
    e.playing = false;
    state.pending_scene = None;
    state.request_scene = false;
    e.project_browser = state;
    e.open_scene(original_path).unwrap();
    // Blueprint attachment resolves the reflected runtime bases, which only the
    // pinned Clang/MIPS SDK can extract. Hosts without that toolchain still
    // cover the rest of the browser.
    if e.class_registry.named("epok::Behaviour").is_some() {
        let parent = e
            .class_registry
            .named("epok::Behaviour")
            .unwrap()
            .id
            .clone();
        let path = crate::blueprint_workflow::create(
            &root,
            &e.class_registry,
            "BP_Cube",
            "",
            &parent,
            true,
        )
        .unwrap();
        let relative = "assets/Blueprints/BP_Cube.epokbp".to_string();
        let blueprint = crate::blueprint_asset::load(&path).unwrap();
        e.selected = Some(0);
        e.scene.entities[0] = crate::scene::Entity::cube("Existing Cube".into());
        e.attach("Spinner");
        e.scene.entities[0].position = [12., 3., -7.];
        e.scene.entities[0]
            .script
            .as_mut()
            .unwrap()
            .properties
            .insert("speed".into(), serde_json::json!(123));
        let before = e.scene.clone();
        e.assets.folder = "assets/Blueprints".into();
        // Fixture creation bypasses the UI's refresh action. Do not depend on slow
        // scene opens having expired the browser's two-second polling throttle.
        e.project_browser.scanned = None;
        e.project_browser.scan(&root);
        frame(ctx, &mut e, font);
        frame(ctx, &mut e, font);
        let start = point(&relative);
        // Drop over a component field, rather than a special attachment button.
        let end = [1250., 130.];
        ctx.io_mut().add_mouse_pos_event(start);
        frame(ctx, &mut e, font);
        ctx.io_mut().add_mouse_button_event(MouseButton::Left, true);
        frame(ctx, &mut e, font);
        for step in 1..=12 {
            let t = step as f32 / 12.;
            ctx.io_mut().add_mouse_pos_event([
                start[0] + (end[0] - start[0]) * t,
                start[1] + (end[1] - start[1]) * t,
            ]);
            frame(ctx, &mut e, font);
        }
        ctx.io_mut()
            .add_mouse_button_event(MouseButton::Left, false);
        frame(ctx, &mut e, font);
        frame(ctx, &mut e, font);
        let binding = e.scene.entities[0].script.as_ref().unwrap();
        assert_eq!(
            binding.class_id.as_ref(),
            Some(&blueprint.id),
            "Inspector drop failed: {:?}",
            e.project_browser.error
        );
        assert_eq!(binding.provider.id, "blueprint");
        let mut expected = before.clone();
        expected.entities[0].script = Some(binding.clone());
        assert_eq!(e.scene, expected, "Only the behaviour should change");
        e.undo_attachment(false).unwrap();
        assert_eq!(
            e.scene, before,
            "Undo restores the previous script and its overrides"
        );
        e.undo_attachment(true).unwrap();
        e.scene.entities[0]
            .script
            .as_mut()
            .unwrap()
            .properties
            .insert("speed".into(), serde_json::json!(42));
        let assigned = e.scene.clone();
        let undo_count = e.script_undo.len();
        assign_content(&mut e, std::slice::from_ref(&relative)).unwrap();
        assert_eq!(
            e.scene, assigned,
            "Re-dropping the same BP preserves overrides"
        );
        assert_eq!(e.script_undo.len(), undo_count);
        for paths in [
            vec![],
            vec![relative.clone(), relative.clone()],
            vec!["assets/other.png".into()],
            vec!["../outside.epokbp".into()],
        ] {
            assert!(assign_content(&mut e, &paths).is_err());
            assert_eq!(e.scene, assigned);
        }
        e.playing = true;
        assert!(
            assign_content(&mut e, std::slice::from_ref(&relative))
                .unwrap_err()
                .contains("Stop Play")
        );
        e.playing = false;
        e.selected = None;
        assert!(
            assign_content(&mut e, std::slice::from_ref(&relative))
                .unwrap_err()
                .contains("Select an object")
        );
        e.selected = Some(0);
        let parent = e
            .class_registry
            .named("epok::Behaviour")
            .unwrap()
            .id
            .clone();
        let abstract_bp = crate::blueprint_asset::BlueprintAsset::new("BP_Abstract".into(), parent);
        crate::blueprint_asset::create(
            &root.join("assets/Blueprints/BP_Abstract.epokbp"),
            &abstract_bp,
        )
        .unwrap();
        assert!(
            assign_content(&mut e, &["assets/Blueprints/BP_Abstract.epokbp".into()])
                .unwrap_err()
                .contains("abstract")
        );
        assert_eq!(e.scene, assigned);
        // A formerly valid asset becoming invalid must not attach from the old registry.
        fs::write(&path, "invalid blueprint").unwrap();
        assert!(assign_content(&mut e, &[relative]).is_err());
        assert_eq!(e.scene, assigned);
    } else {
        eprintln!(
            "Skipping blueprint drag-and-drop coverage: reflected runtime classes are unavailable without the MIPS SDK"
        );
    }
    // Inline audition consumes its click, including a second click, without opening Imports.
    fs::create_dir(root.join("assets/Previews")).unwrap();
    fs::write(
        root.join("assets/Previews/Tone.wav"),
        crate::audio_import::test_wav(),
    )
    .unwrap();
    fs::write(root.join("assets/Previews/Code.hpp"), "// source script").unwrap();
    assets::commit(
        assets::prepare(
            &root,
            "assets/Previews/Tone.wav",
            "assets/Previews/Tone.epokasset",
            Default::default(),
            None,
            false,
        )
        .unwrap(),
    )
    .unwrap();
    e.assets.index = assets::scan(&e.root, &mut Default::default());
    e.assets.folder = "assets/Previews".into();
    e.project_browser.scanned = None;
    e.project_browser.prefs.hide_unprocessed = true;
    e.assets.window = false;
    frame(ctx, &mut e, font);
    assert_eq!(
        visible(&e.project_browser, &e).len(),
        2,
        "Native assets and scripts remain visible by default"
    );
    click(ctx, &mut e, font, "\u{f013} Settings");
    frame(ctx, &mut e, font);
    click(ctx, &mut e, font, "Hide unprocessed files");
    assert!(!e.project_browser.prefs.hide_unprocessed);
    let saved: Preferences = crate::document::from_slice(
        &fs::read(root.join("UserSettings/ContentBrowser.epokprefs")).unwrap(),
    )
    .unwrap();
    assert!(!saved.hide_unprocessed, "Settings persist per project");
    ctx.io_mut().add_key_event(imgui::Key::Escape, true);
    frame(ctx, &mut e, font);
    ctx.io_mut().add_key_event(imgui::Key::Escape, false);
    frame(ctx, &mut e, font);
    assert_eq!(visible(&e.project_browser, &e).len(), 3);
    let original = visible(&e.project_browser, &e)
        .into_iter()
        .find(|entry| entry.name == "Tone.wav")
        .unwrap();
    assert_eq!(kind(&e, &original), "Audio");
    assert_ne!(entry_icon(&e, &original).1, type_icon("Audio").1);
    for list in [false, true] {
        e.project_browser.prefs.list = list;
        frame(ctx, &mut e, font);
        frame(ctx, &mut e, font);
        let path = e.root.join("assets/Previews/Tone.epokasset");
        click(ctx, &mut e, font, "Play##assets/Previews/Tone.epokasset");
        assert!(
            e.project_browser.previews.active(&path),
            "Play activates the inline audition in list={list}"
        );
        assert!(!e.assets.window, "Play does not open the asset inspector");
        click(ctx, &mut e, font, "Play##assets/Previews/Tone.epokasset");
        assert!(
            !e.project_browser.previews.active(&path),
            "A second click cancels/stops playback"
        );
        assert!(!e.assets.window);
    }
    drop(e);
    fs::remove_dir_all(root).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "Owns an ImGui context; run explicitly and serially for audio preview error routing"]
    fn midi_preview_failure_logs_once_per_play_attempt_in_tiles_and_list() {
        let mut context = imgui::Context::create();
        context.set_ini_filename(None);
        context.io_mut().display_size = [1100., 700.];
        context.io_mut().delta_time = 1. / 60.;
        let font = context.fonts().add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
        context.fonts().build_rgba32_texture();
        let root = crate::workspace::tests::temp("midi-preview-console");
        fs::create_dir_all(root.join("assets")).unwrap();
        // Owned SMF: undefined controller, then one note. Import is valid;
        // playback must retain its diagnostic without silently ignoring the event.
        let events = [0, 0xb0, 119, 1, 0, 0x90, 60, 100, 96, 0x80, 60, 0, 0, 0xff, 0x2f, 0];
        let mut midi = b"MThd\0\0\0\x06\0\0\0\x01\0\x60MTrk".to_vec();
        midi.extend((events.len() as u32).to_be_bytes());
        midi.extend(events);
        fs::write(root.join("assets/unsupported.mid"), midi).unwrap();
        assets::commit(crate::sequence::prepare(&root, "assets/unsupported.mid",
            "assets/unsupported.epokasset", Default::default(), None, false).unwrap()).unwrap();
        let path = root.join("assets/unsupported.epokasset");
        let original = fs::read(&path).unwrap();
        let mut editor = Editor::new(root.clone());
        editor.auto_build = false;
        editor.assets.index = assets::scan(&root, &mut Default::default());
        editor.assets.folder = "assets".into();
        editor.clear_logs();
        let frame = |context: &mut imgui::Context, editor: &mut Editor| {
            editor.project_browser.previews.poll();
            let ui = context.frame();
            unsafe {
                imgui::sys::igSetNextWindowSize(imgui::sys::ImVec2 { x: 1100., y: 600. }, imgui::sys::ImGuiCond_Always as i32);
            }
            window(ui, editor, font);
            context.render();
        };
        for (attempt, list) in [false, false, true].into_iter().enumerate() {
            editor.project_browser.prefs.list = list;
            frame(&mut context, &mut editor);
            frame(&mut context, &mut editor);
            let position = CONTROLS.with(|buttons| buttons.borrow()["Play##assets/unsupported.epokasset"]);
            context.io_mut().add_mouse_pos_event(position);
            frame(&mut context, &mut editor);
            context.io_mut().add_mouse_button_event(MouseButton::Left, true);
            frame(&mut context, &mut editor);
            context.io_mut().add_mouse_button_event(MouseButton::Left, false);
            frame(&mut context, &mut editor);
            let deadline = Instant::now() + std::time::Duration::from_secs(5);
            while editor.project_browser.previews.active(&path) {
                assert!(Instant::now() < deadline, "Preview worker did not finish");
                std::thread::sleep(std::time::Duration::from_millis(2));
                frame(&mut context, &mut editor);
            }
            for _ in 0..30 { frame(&mut context, &mut editor); }
            assert_eq!(editor.logs.len(), attempt + 1, "Exactly one Console entry per Play attempt");
            let message = editor.logs.last().unwrap();
            assert!(message.contains("unsupported.epokasset") && message.contains("1 unsupported events")
                && message.contains("CC 119=1") && message.contains("re-export the MIDI"), "{message}");
            assert!(editor.project_browser.error.is_none(), "Playback errors must not persist in the browser footer");
            assert!(editor.project_browser.previews.error.is_none());
        }
        editor.clear_logs();
        for _ in 0..30 { frame(&mut context, &mut editor); }
        assert!(editor.logs.is_empty(), "Clearing Console must not replay the last error");
        assert_eq!(fs::read(&path).unwrap(), original);
        drop(editor);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    #[ignore = "Requires EPOK_INSPECTOR_PROJECT pointing to a disposable Ironwood copy and matching header tool"]
    fn ironwood_blueprint_assigns_to_an_existing_cube() {
        let root = PathBuf::from(std::env::var("EPOK_INSPECTOR_PROJECT").unwrap());
        let project = crate::workspace::Project::open(&root).unwrap();
        let mut e = Editor::open(project).unwrap();
        e.auto_build = false;
        let index = e
            .scene
            .entities
            .iter()
            .position(|entity| entity.name.contains("Cube"))
            .expect("Ironwood fixture must contain its existing cube");
        e.selected = Some(index);
        let before = e.scene.clone();
        assign_content(&mut e, &["assets/Blueprints/BP_Cube.epokbp".into()]).unwrap();
        let binding = e.scene.entities[index].script.as_ref().unwrap();
        assert_eq!(binding.name, "BP_Cube");
        assert_eq!(binding.provider.id, "blueprint");
        let mut expected = before.clone();
        expected.entities[index].script = Some(binding.clone());
        assert_eq!(e.scene, expected);
        e.undo_attachment(false).unwrap();
        assert_eq!(e.scene, before);
    }
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let p = std::env::temp_dir().join(format!("epok-content-{}", uuid::Uuid::new_v4()));
            fs::create_dir_all(p.join("assets/A/Empty")).unwrap();
            fs::create_dir(p.join("assets/B")).unwrap();
            Self(p)
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    #[test]
    fn native_labels_preserve_dots_and_folder_names() {
        for (name, folder, expected) in [
            ("Hero.v2.epokbp", false, "Hero.v2"),
            ("Main.epokmap", false, "Main"),
            ("Footsteps.epokasset", false, "Footsteps"),
            ("Intro.timeline.json", false, "Intro"),
            ("Fire.particle-effect.json", false, "Fire"),
            ("Source.png", false, "Source.png"),
            ("Config.json", false, "Config.json"),
            ("Examples.epokbp", true, "Examples.epokbp"),
        ] {
            let entry = Entry {
                name: name.into(),
                path: format!("assets/{name}"),
                folder,
            };
            assert_eq!(display_name(&entry), expected);
            assert_eq!(entry.name, name, "Presentation must never rename files");
        }
    }
    #[test]
    fn legacy_preferences_default_to_hiding_only_importable_originals() {
        let prefs: Preferences =
            serde_json::from_str(r#"{"show_sources":true,"tile_size":180}"#).unwrap();
        assert!(prefs.hide_unprocessed);
        assert_eq!(prefs.tile_size, 180.);
        for (name, expected) in [
            ("Photo.PNG", true),
            ("Music.MP3", true),
            ("Mesh.fbx", true),
            ("Sound.epokasset", false),
            ("Actor.hpp", false),
        ] {
            assert_eq!(
                unprocessed(&Entry {
                    name: name.into(),
                    path: format!("assets/{name}"),
                    folder: false
                }),
                expected
            );
        }
    }
    #[test]
    fn folder_moves_preserve_bytes_and_identity_and_can_be_reversed() {
        let f = Fixture::new();
        let bytes = br#"{"id":"persistent-uuid","references":["another-uuid"]}"#;
        fs::write(f.0.join("assets/A/asset.epokbp"), bytes).unwrap();
        move_paths(&f.0, &[("assets/A".into(), "assets/B/A".into())]).unwrap();
        assert_eq!(
            fs::read(f.0.join("assets/B/A/asset.epokbp")).unwrap(),
            bytes
        );
        assert!(f.0.join("assets/B/A/Empty").is_dir());
        move_paths(&f.0, &[("assets/B/A".into(), "assets/A".into())]).unwrap();
        assert!(f.0.join("assets/A/asset.epokbp").is_file());
    }
    #[test]
    fn preflight_rejects_collisions_cycles_and_escapes_before_moving() {
        let f = Fixture::new();
        fs::write(f.0.join("assets/A/one.txt"), b"one").unwrap();
        fs::write(f.0.join("assets/B/one.txt"), b"two").unwrap();
        for moves in [
            vec![("assets/A".into(), "assets/A/Empty/A".into())],
            vec![("assets/A/one.txt".into(), "assets/B/one.txt".into())],
            vec![("assets/A".into(), "../outside".into())],
            vec![
                ("assets/A/Empty".into(), "assets/B/Empty".into()),
                ("assets/A/one.txt".into(), "assets/B/one.txt".into()),
            ],
        ] {
            assert!(move_paths(&f.0, &moves).is_err());
            assert!(f.0.join("assets/A/Empty").is_dir());
            assert_eq!(fs::read(f.0.join("assets/A/one.txt")).unwrap(), b"one");
        }
    }
    #[test]
    fn scan_includes_empty_folders_and_favorites_follow_moves() {
        let f = Fixture::new();
        let mut state = State::default();
        state.scan(&f.0);
        assert!(
            state
                .entries
                .iter()
                .any(|e| e.path == "assets/A/Empty" && e.folder)
        );
        state.prefs.favorites.insert("assets/A/Empty".into());
        state
            .prefs
            .collections
            .insert("Test".into(), BTreeSet::from(["assets/A/Empty".into()]));
        remap(&mut state, &[("assets/A".into(), "assets/B/A".into())]);
        assert!(state.prefs.favorites.contains("assets/B/A/Empty"));
        assert!(state.prefs.collections["Test"].contains("assets/B/A/Empty"));
    }
    #[test]
    fn copies_make_independent_packages_and_reject_duplicate_classes_atomically() {
        let f = Fixture::new();
        fs::write(f.0.join("assets/tone.wav"), crate::audio_import::test_wav()).unwrap();
        let id = assets::commit(
            assets::prepare(
                &f.0,
                "assets/tone.wav",
                "assets/A/tone.epokasset",
                Default::default(),
                None,
                false,
            )
            .unwrap(),
        )
        .unwrap();
        copy_paths(&f.0, &[("assets/A".into(), "assets/B/Copy".into())]).unwrap();
        let copy = assets::Package::load(&f.0.join("assets/B/Copy/tone.epokasset")).unwrap();
        assert_ne!(id, copy.meta.id);
        assert_eq!(copy.source, fs::read(f.0.join("assets/tone.wav")).unwrap());
        fs::write(f.0.join("assets/A/Class.hpp"), b"class Original {};").unwrap();
        assert!(copy_paths(&f.0, &[("assets/A".into(), "assets/B/Rejected".into())]).is_err());
        assert!(!f.0.join("assets/B/Rejected").exists());
    }
    #[test]
    fn trash_retains_files_and_restore_rejects_collisions() {
        let f = Fixture::new();
        fs::write(f.0.join("assets/A/keep.txt"), b"original").unwrap();
        let e = Editor::new(f.0.clone());
        let journal = trash_content(
            &e,
            &BTreeSet::from(["assets/A".into(), "assets/A/Empty".into()]),
        )
        .unwrap();
        assert_eq!(journal.len(), 1);
        assert!(!f.0.join("assets/A").exists());
        assert_eq!(
            fs::read(journal[0].1.join("keep.txt")).unwrap(),
            b"original"
        );
        fs::create_dir(f.0.join("assets/A")).unwrap();
        assert!(restore_trash(&f.0, &journal).is_err());
        fs::remove_dir(f.0.join("assets/A")).unwrap();
        restore_trash(&f.0, &journal).unwrap();
        assert_eq!(
            fs::read(f.0.join("assets/A/keep.txt")).unwrap(),
            b"original"
        );
    }
    #[test]
    fn external_import_copies_into_current_folder_without_overwriting() {
        let f = Fixture::new();
        let source = f.0.join("external.wav");
        fs::write(&source, crate::audio_import::test_wav()).unwrap();
        let mut e = Editor::new(f.0.clone());
        e.assets.folder = "assets/B".into();
        let mut s = State::default();
        assert!(import(&mut e, &mut s, &source));
        assert!(source.exists());
        assert!(e.assets.window);
        assert_eq!(
            fs::read(f.0.join("assets/B/external.wav")).unwrap(),
            fs::read(&source).unwrap()
        );
        fs::write(&source, b"changed source").unwrap();
        assert!(!import(&mut e, &mut s, &source));
        assert_ne!(
            fs::read(f.0.join("assets/B/external.wav")).unwrap(),
            b"changed source"
        );
    }
    #[test]
    fn scene_drop_places_audio_and_rolls_back_an_invalid_batch() {
        let f = Fixture::new();
        fs::write(f.0.join("assets/tone.wav"), crate::audio_import::test_wav()).unwrap();
        let id = assets::commit(
            assets::prepare(
                &f.0,
                "assets/tone.wav",
                "assets/A/tone.epokasset",
                Default::default(),
                None,
                false,
            )
            .unwrap(),
        )
        .unwrap();
        let mut e = Editor::new(f.0.clone());
        e.assets.index = assets::scan(&f.0, &mut Default::default());
        let initial = e.scene.clone();
        assert!(
            place_content(
                &mut e,
                &[
                    "assets/A/tone.epokasset".into(),
                    "assets/invalid.png".into()
                ]
            )
            .is_err()
        );
        assert_eq!(e.scene, initial);
        place_content(&mut e, &["assets/A/tone.epokasset".into()]).unwrap();
        assert_eq!(
            e.scene
                .entities
                .last()
                .unwrap()
                .audio
                .as_ref()
                .unwrap()
                .clip,
            Some(id)
        );
        assert!(e.dirty);
        assert_eq!(e.script_undo.len(), 1);
    }
    #[test]
    fn search_history_and_collection_membership_are_scoped() {
        let f = Fixture::new();
        fs::write(f.0.join("assets/A/needle.png"), b"fixture").unwrap();
        let mut e = Editor::new(f.0.clone());
        let mut s = State::default();
        s.prefs.hide_unprocessed = false;
        s.scan(&f.0);
        assert_eq!(visible(&s, &e).len(), 2);
        e.project_search = "needle".into();
        assert_eq!(visible(&s, &e).len(), 1);
        s.navigate(&mut e, "assets".into());
        s.navigate(&mut e, "assets/A".into());
        s.navigate(&mut e, "assets/B".into());
        s.travel(&mut e, false);
        assert_eq!(e.assets.folder, "assets/A");
        s.travel(&mut e, true);
        assert_eq!(e.assets.folder, "assets/B");
        s.prefs.collections.insert(
            "Needles".into(),
            BTreeSet::from(["assets/A/needle.png".into()]),
        );
        s.collection = Some("Needles".into());
        assert_eq!(visible(&s, &e).len(), 1);
    }
}
