use crate::editor::Editor;
use imgui::{Condition, StyleColor as C, WindowFlags as W};

const CUBE: &str = "\u{eb29}";
const CODE: &str = "\u{eae9}";
const CAMERA: &str = "\u{ead9}";
const MOVE: &str = "\u{eb22}";
const PLAY: &str = "\u{eb2c}";
const PAUSE: &str = "\u{ead1}";
const STOP: &str = "\u{ead7}";
const SEARCH: &str = "\u{ea6d}";
const ACTOR: &str = "\u{eb5b}";

pub fn configure_input(io: &mut imgui::Io) {
    // Scene/HUD handles are drawn over images rather than ImGui buttons.
    // Their drags must never also move or undock the containing panel.
    io.config_windows_move_from_title_bar_only = true;
    io.config_drag_click_to_input_text = true;
}

/// Keep ImGui's numeric editing behavior, adding a consistent drag cursor.
pub struct Drag<T, L, F = &'static str>(imgui::Drag<T, L, F>);
impl<T: imgui::internal::DataTypeKind, L: AsRef<str>> Drag<T, L> {
    pub fn new(label: L) -> Self {
        Self(imgui::Drag::new(label))
    }
}
impl<T: imgui::internal::DataTypeKind, L: AsRef<str>, F: AsRef<str>> Drag<T, L, F> {
    pub fn speed(self, speed: f32) -> Self {
        Self(self.0.speed(speed))
    }
    pub fn range(self, min: T, max: T) -> Self {
        Self(self.0.range(min, max))
    }
    pub fn display_format<F2: AsRef<str>>(self, format: F2) -> Drag<T, L, F2> {
        Drag(self.0.display_format(format))
    }
    pub fn build(self, ui: &imgui::Ui, value: &mut T) -> bool {
        let changed = self.0.build(ui, value);
        numeric_cursor(ui);
        changed
    }
    pub fn build_array(self, ui: &imgui::Ui, values: &mut [T]) -> bool {
        let changed = self.0.build_array(ui, values);
        numeric_cursor(ui);
        changed
    }
}
fn numeric_cursor(ui: &imgui::Ui) {
    if ui.is_item_active()
        && ui.is_mouse_dragging(imgui::MouseButton::Left)
        && !unsafe { imgui::sys::igTempInputIsActive(imgui::sys::igGetActiveID()) }
    {
        ui.set_mouse_cursor(Some(imgui::MouseCursor::ResizeEW));
    }
}
pub(crate) fn text_input_active(ui: &imgui::Ui) -> bool {
    // Read-only text still consumes selection, copy and delete keys, although
    // ImGui correctly does not ask the OS to show a text-entry keyboard.
    ui.io().want_text_input
        || unsafe {
            let id = imgui::sys::igGetActiveID();
            id != 0 && !imgui::sys::igGetInputTextState(id).is_null()
        }
}

pub(crate) fn script_button(ui: &imgui::Ui, label: &str) -> bool {
    let pressed = ui.button(label);
    #[cfg(test)]
    record_script_control(ui, label);
    pressed
}
#[cfg(test)]
pub(crate) fn record_script_control(ui: &imgui::Ui, label: &str) {
    SCRIPT_BUTTONS.with(|buttons| {
        let a = ui.item_rect_min();
        let b = ui.item_rect_max();
        buttons
            .borrow_mut()
            .insert(label.into(), [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5]);
    });
}
#[cfg(test)]
thread_local! {static SCRIPT_BUTTONS:std::cell::RefCell<std::collections::BTreeMap<String,[f32;2]>>=const{std::cell::RefCell::new(std::collections::BTreeMap::new())};}
fn build_menu(ui: &imgui::Ui, e: &mut Editor) {
    let menu = ui.begin_menu("Build");
    #[cfg(test)]
    record_script_control(ui, "Build menu");
    if let Some(_menu) = menu {
        let enabled = crate::lighting_editor::can_start_bake(e);
        if ui
            .menu_item_config("Build Project")
            .enabled(enabled)
            .build()
        {
            e.action("build");
        }
        if ui
            .menu_item_config("Build Lighting")
            .enabled(enabled)
            .build()
        {
            crate::lighting_editor::start_bake(e);
        }
        #[cfg(test)]
        record_script_control(ui, "Build Lighting");
        ui.separator();
        if ui
            .menu_item_config("Package PSX Disc...")
            .enabled(enabled)
            .build()
        {
            e.action("export-disc");
        }
        if ui
            .menu_item_config("Export PsyQo Project...")
            .enabled(enabled)
            .build()
        {
            e.action("export");
        }
        ui.separator();
        if ui.menu_item("Lighting Settings...") {
            e.lighting_window = true;
        }
    }
}
fn script_creation_dialog(ui: &imgui::Ui, editor: &mut Editor) {
    if editor.script_creation {
        ui.open_popup("Create C++ script");
        editor.script_creation = false;
    }
    ui.modal_popup_config("Create C++ script").always_auto_resize(true).build(|| {
        let model = editor.object_model();
        let allowed: std::collections::BTreeSet<_> = editor.class_registry.eligible_parents()
            .filter(|parent| editor.script_creation_context.allows_parent(&editor.scene, model.as_deref(), parent))
            .map(|parent| parent.id.clone()).collect();
        ui.text_wrapped("Children inherit properties and lifecycle behavior. Override only events you intend to change.");
        ui.input_text("Asset name", &mut editor.script_name).hint("Enemy").build();
        ui.input_text("Folder", &mut editor.script_folder).hint("Enemies/Bosses (under assets/scripts)").build();
        ui.input_text("Search parents", &mut editor.script_search).build();
        let query=editor.script_search.to_lowercase();
        ui.child_window("parent-tree").size([520.,180.]).border(true).build(|| {
            for parent in ["epok::Actor3D","epok::Actor2D","epok::UIActor","epok::ActorComponent"] {
                if !editor.class_registry.named(parent).is_some_and(|class| allowed.contains(&class.id)) {continue;}
                if let Some(_node)=ui.tree_node_config(parent).default_open(true).push() {
                    if ui.selectable_config(format!("Use {parent}")).selected(editor.script_parent==parent).build() {editor.script_parent=parent.into();}
                    #[cfg(test)]
                    record_script_control(ui, parent);
                    parent_class_tree(ui,&editor.class_registry,parent,&query,&mut editor.script_parent,&allowed);
                }
            }
        });
        ui.text(format!("Parent: {}",editor.script_parent));
        ui.child_window("inherited-members").size([520.,140.]).border(true).build(|| {
            let name=&editor.script_parent;
            let mut chain=editor.class_registry.ancestry(name); chain.reverse();
            if chain.is_empty() {ui.text_disabled("Reload the reflected class catalog.");}
            for class in chain {
                ui.text_disabled(format!("{}{}",class.cpp_name,if class.abstract_class {" (abstract)"} else {""}));
                for p in &class.properties {ui.bullet_text(format!("{}: {} = {}",p.name,p.value_type.label(),p.default));}
                for f in &class.functions {
                    let params=f.parameters.iter().map(|p|format!("{}{} {}", if p.direction==crate::reflection_schema::Direction::ConstReference {"const "}else{""},p.value_type.label(),if p.direction==crate::reflection_schema::Direction::Value {p.name.clone()}else{format!("&{}",p.name)})).collect::<Vec<_>>().join(", ");
                    ui.bullet_text(format!("{}({}) -> {}{}",f.name,params,f.returns.label(),if f.abstract_method {" [pure virtual]"} else {""}));
                }
            }
        });
        if let Some(error)=&editor.script_error {ui.text_colored([1.,0.5,0.4,1.],error);}
        if script_button(ui,"Cancel") { ui.close_current_popup(); }
        ui.same_line();
        let parent_allowed = editor.class_registry.named(&editor.script_parent).is_some_and(|parent| allowed.contains(&parent.id));
        let create = {let _disabled=ui.begin_disabled(!parent_allowed); script_button(ui,"Create")};
        ui.same_line();
        let mut attach=false;
        let attachment_index = editor.script_creation_context.actor_index(&editor.scene, editor.selected);
        let attachment_error = attachment_index.ok_or_else(|| "Select an Actor to enable Create and Attach.".to_owned())
            .and_then(|index| {
                let parent=editor.class_registry.named(&editor.script_parent).ok_or("Select an ActorComponent parent.")?;
                crate::actor_scripts::validate_parent(&editor.scene,index,&parent.id,&editor.class_registry)
            }).err();
        ui.disabled(!parent_allowed || attachment_error.is_some() || editor.playing, || {attach=script_button(ui,"Create and Attach");});
        if let Some(error)=&attachment_error {ui.text_wrapped(error);}
        if create || attach {
            match crate::scripts::create_in(&editor.root, editor.script_name.trim(), editor.script_folder.trim(), &editor.script_parent, attach) {
                Ok(()) => {
                    let name = editor.script_name.trim().to_owned();
                    editor.refresh_scripts();
                    if attach {
                        editor.select_actor(attachment_index.map(|index| editor.scene.actors[index].id));
                        editor.attach(&name);
                    }
                    #[cfg(not(test))]
                    editor.open_code(&crate::scripts::source(&editor.root, &name), None);
                    ui.close_current_popup();
                }
                Err(error) => {editor.log(&error); editor.script_error=Some(error);}
            }
        }
    });
}
fn parent_class_tree(
    ui: &imgui::Ui,
    registry: &crate::blueprint::Registry,
    parent: &str,
    query: &str,
    selected: &mut String,
    allowed: &std::collections::BTreeSet<String>,
) {
    for class in registry
        .eligible_parents()
        .filter(|c| c.id != crate::object_model::OBJECT_ID)
        .filter(|c| allowed.contains(&c.id))
    {
        let direct = class
            .parent
            .as_ref()
            .and_then(|id| registry.classes.get(id))
            .map(|p| p.cpp_name.as_str())
            .unwrap_or("epok::Object");
        if direct != parent {
            continue;
        }
        let visible = query.is_empty()
            || registry.eligible_parents().any(|candidate| {
                candidate.cpp_name.to_lowercase().contains(query)
                    && allowed.contains(&candidate.id)
                    && registry
                        .ancestry(&candidate.cpp_name)
                        .iter()
                        .any(|ancestor| ancestor.id == class.id)
            });
        if !visible {
            continue;
        }
        ui.tree_node_config(&class.id)
            .label::<&str, _>(&class.cpp_name)
            .default_open(true)
            .build(|| {
                if ui
                    .selectable_config(format!(
                        "Use {}{}",
                        class.cpp_name,
                        if class.abstract_class {
                            " (abstract)"
                        } else {
                            ""
                        }
                    ))
                    .selected(*selected == class.cpp_name)
                    .build()
                {
                    *selected = class.cpp_name.clone();
                }
                #[cfg(test)]
                record_script_control(ui, &class.cpp_name);
                parent_class_tree(ui, registry, &class.cpp_name, query, selected, allowed);
            });
    }
}

/// Both Inspector presentations use the same reflected, compatible components.
fn add_component_menu(ui: &imgui::Ui, editor: &mut Editor, actor: uuid::Uuid) -> bool {
    let mut added = false;
    ui.popup("add-component", || {
        let classes = editor.addable_component_classes(actor);
        for group in ["Shared", "Domain"] {
            if !classes.iter().any(|(g, _, _)| *g == group) {
                continue;
            }
            ui.text_disabled(group);
            for (_, class, label) in classes.iter().filter(|(g, _, _)| *g == group) {
                if ui.menu_item(label) {
                    if editor.catalog.iter().any(|script| script.name == *class) {
                        editor.select_actor(Some(actor));
                        editor.attach(class);
                    } else {
                        editor.add_actor_component(actor, class);
                    }
                    added = true;
                }
                #[cfg(test)]
                record_script_control(ui, class);
                if ui.is_item_hovered() {
                    ui.tooltip_text(class);
                }
            }
        }
        ui.separator();
        if ui.menu_item("Create C++ ActorComponent...") {
            editor.action("new-script");
            editor.script_creation_context =
                crate::actor_scripts::CreationContext::Component(actor);
        }
        #[cfg(test)]
        record_script_control(ui, "Create C++ ActorComponent...");
        if ui.menu_item("Create Blueprint ActorComponent...") {
            crate::blueprint_workflow::begin_component(editor, actor);
        }
        #[cfg(test)]
        record_script_control(ui, "Create Blueprint ActorComponent...");
    });
    added
}
pub(crate) fn gray(v: u8) -> [f32; 4] {
    let f = v as f32 / 255.;
    [f, f, f, 1.]
}
pub fn theme(style: &mut imgui::Style) {
    style.use_dark_colors();
    style.window_rounding = 0.;
    style.child_rounding = 0.;
    style.frame_rounding = 3.;
    style.grab_rounding = 1.;
    style.tab_rounding = 3.;
    style.popup_rounding = 2.;
    style.window_border_size = 1.;
    style.child_border_size = 1.;
    style.frame_border_size = 1.;
    style.window_padding = [10., 8.];
    style.frame_padding = [7., 4.];
    style.item_spacing = [7., 6.];
    style.item_inner_spacing = [5., 4.];
    style.cell_padding = [8., 6.];
    style.indent_spacing = 16.;
    style.scrollbar_size = 12.;
    style.scrollbar_rounding = 4.;
    for (kind, v) in [
        (C::WindowBg, 28),
        (C::ChildBg, 28),
        (C::PopupBg, 32),
        (C::Border, 48),
        (C::MenuBarBg, 18),
        (C::TitleBg, 18),
        (C::TitleBgActive, 22),
        (C::TitleBgCollapsed, 18),
        (C::FrameBg, 15),
        (C::FrameBgHovered, 48),
        (C::FrameBgActive, 52),
        (C::Button, 49),
        (C::ButtonHovered, 65),
        (C::ButtonActive, 57),
        (C::Tab, 20),
        (C::TabActive, 43),
        (C::TabUnfocused, 20),
        (C::TabUnfocusedActive, 36),
        (C::TabHovered, 57),
        (C::Separator, 48),
        (C::ResizeGrip, 48),
        (C::ScrollbarBg, 22),
        (C::ScrollbarGrab, 70),
        (C::Text, 222),
        (C::TextDisabled, 146),
        (C::TableRowBg, 30),
        (C::TableRowBgAlt, 33),
    ] {
        style.colors[kind as usize] = gray(v);
    }
    style.colors[C::Header as usize] = [0.075, 0.28, 0.48, 1.];
    style.colors[C::HeaderHovered as usize] = gray(57);
    style.colors[C::HeaderActive as usize] = [0.08, 0.32, 0.58, 1.];
    style.colors[C::CheckMark as usize] = [0.2, 0.6, 1., 1.];
    style.colors[C::SliderGrab as usize] = [0.2, 0.6, 1., 1.];
    style.colors[C::SliderGrabActive as usize] = [0.35, 0.7, 1., 1.];
    style.colors[C::DockingEmptyBg as usize] = gray(18);
    style.colors[C::ModalWindowDimBg as usize] = [0., 0., 0., 0.55];
}

/// Place labels before values; stack them when a dock panel becomes narrow.
pub(crate) fn field(ui: &imgui::Ui, label: &str) -> String {
    let visible = label.split("##").next().unwrap_or(label);
    let start = ui.cursor_pos()[0];
    let width = ui.content_region_avail()[0];
    ui.align_text_to_frame_padding();
    ui.text_wrapped(visible);
    let label_width = (width * 0.38).clamp(100., 170.);
    if width >= 340. && ui.calc_text_size(visible)[0] < label_width - 8. {
        ui.same_line_with_pos(start + label_width);
    }
    ui.set_next_item_width(-1.);
    format!("##{label}")
}

/// Continue a toolbar row only when the next control fits inside this panel.
pub(crate) fn inline(ui: &imgui::Ui, label: &str) {
    inline_width(ui, ui.calc_text_size(label)[0] + ui.frame_height() + 8.);
}
pub(crate) fn inline_width(ui: &imgui::Ui, width: f32) {
    let right = ui.cursor_screen_pos()[0] + ui.content_region_avail()[0];
    if ui.item_rect_max()[0] + 7. + width <= right {
        ui.same_line();
    }
}

pub(crate) fn muted(ui: &imgui::Ui, text: impl AsRef<str>) {
    let _color = ui.push_style_color(C::Text, gray(146));
    ui.text_wrapped(text);
}

pub(crate) fn clipped(ui: &imgui::Ui, text: &str, width: f32) {
    let mut visible = text.to_string();
    if ui.calc_text_size(text)[0] > width {
        while !visible.is_empty() && ui.calc_text_size(format!("{visible}...").as_str())[0] > width
        {
            visible.pop();
        }
        visible.push_str("...");
    }
    ui.text(&visible);
    if visible != text && ui.is_item_hovered() {
        ui.tooltip_text(text);
    }
}
fn icon(ui: &imgui::Ui, glyph: &str, id: &str, tip: &str, active: bool) -> bool {
    let _color = ui.push_style_color(
        C::Button,
        if active {
            [0.19, 0.37, 0.52, 1.]
        } else {
            gray(49)
        },
    );
    let result = ui.button_with_size(format!("{glyph}##{id}"), [28., 21.]);
    if ui.is_item_hovered_with_flags(imgui::ItemHoveredFlags::ALLOW_WHEN_DISABLED) {
        ui.tooltip_text(tip);
    }
    result
}
pub fn draw(
    ui: &imgui::Ui,
    e: &mut Editor,
    scene_textures: [imgui::TextureId; 3],
    game: Option<imgui::TextureId>,
    asset_font: imgui::FontId,
    image_size: [f32; 2],
    initial: &mut bool,
) {
    if e.critical_busy() {
        e.console.force_follow();
        e.scene_navigation = false;
        e.scene_look = false;
        e.raw_look = None;
        e.drag_axis = None;
        e.hud_drag = None;
        e.game_capture = false;
        crate::busy_ui::background(ui, || {
            draw_workspace(
                ui,
                e,
                scene_textures,
                game,
                asset_font,
                image_size,
                initial,
                true,
            )
        });
        crate::busy_ui::editor(ui, e);
    } else {
        crate::busy_ui::finish(ui);
        draw_workspace(
            ui,
            e,
            scene_textures,
            game,
            asset_font,
            image_size,
            initial,
            false,
        );
        if e.critical_busy() {
            crate::busy_ui::editor(ui, e);
        } else {
            crate::memory_ui::window(ui, e);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_workspace(
    ui: &imgui::Ui,
    e: &mut Editor,
    scene_textures: [imgui::TextureId; 3],
    game: Option<imgui::TextureId>,
    asset_font: imgui::FontId,
    image_size: [f32; 2],
    initial: &mut bool,
    background: bool,
) {
    let [scene, hud_texture, brand_texture] = scene_textures;
    if std::env::args().any(|arg| arg == "--screenshot-content-browser") {
        crate::project_browser::window(ui, e, asset_font);
        crate::asset_ui::windows(ui, e);
        return;
    }
    let size = ui.io().display_size;
    let menu_height = ui.frame_height();
    let toolbar_height = 40.;
    let status_height = ui.text_line_height() + 12.;
    let workspace_top = menu_height + toolbar_height;
    let workspace_height = (size[1] - workspace_top - status_height).max(1.);
    let reset = *initial || e.reset_layout;
    ui.main_menu_bar(|| {
        ui.menu("File", || {
            if ui.menu_item("Projects... (New / Open / Close)") {
                e.hub_requested = true;
            }
            if ui.menu_item("Open Project Folder") {
                crate::project_browser::reveal(&e.root);
            }
            ui.separator();
            if ui.menu_item("Save\tCtrl+S") {
                e.save_all();
            }
            if ui.menu_item("Reload Scene") {
                e.action("reload");
            }
            ui.separator();
            if ui.menu_item("Exit") {
                request_exit(e);
            }
        });
        ui.menu("Edit", || {
            for (label, shortcut, enabled, redo) in [
                ("Undo Scene Edit", "Ctrl+Z", e.can_undo_scene(), false),
                ("Redo Scene Edit", "Ctrl+Y", e.can_redo_scene(), true),
            ] {
                if ui
                    .menu_item_config(label)
                    .shortcut(shortcut)
                    .enabled(enabled && !e.playing)
                    .build()
                    && let Err(error) = if redo { e.redo_scene() } else { e.undo_scene() }
                {
                    e.log(error);
                }
            }
            ui.separator();
            for (label, redo) in [
                ("Undo Component Attachment / Blueprint Edit", false),
                ("Redo Component Attachment / Blueprint Edit", true),
            ] {
                if ui.menu_item(label)
                    && let Err(error) = e.undo_attachment(redo)
                {
                    e.log(error);
                }
            }
            ui.separator();
            if ui.menu_item("Duplicate\tCtrl+D") {
                e.action("duplicate");
            }
            if ui.menu_item("Delete\tDel") {
                e.action("delete");
            }
            ui.separator();
            if ui.menu_item("Frame Selected\tF") {
                e.action("frame-selected");
            }
            ui.separator();
            if ui.menu_item("Editor Preferences...") {
                crate::settings_ui::open_preferences(e);
            }
            if ui.menu_item("Project Settings...") {
                crate::settings_ui::open_project(e);
            }
        });
        ui.menu("Assets", || {
            if ui.menu_item("Import sample character (FBX)...")
                && let Err(error) = crate::skeletal_ui::sample(e)
            {
                e.log(error);
            }
            if ui.menu_item("Create > C++ Script") {
                e.action("new-script");
            }
            if ui.menu_item("Open C++ Project") {
                e.open_code(&e.root.join("assets"), None);
            }
        });
        ui.menu("GameObject", || {
            let actors = placeable_actor_classes(e);
            let mut actor_class = None;
            if let Some(command) = creation_menu(ui, false, &actors, &mut actor_class) {
                e.action(command);
            }
            if let Some(class) = actor_class {
                e.create_actor(&class);
            }
        });
        build_menu(ui, e);
        ui.menu("Window", || {
            if ui.menu_item("Artifact Dependencies") {
                e.artifact_dependencies.open = true;
            }
            if ui.menu_item("Lighting") {
                e.lighting_window = true;
            }
            for name in [
                c"\u{eb86} Hierarchy###Hierarchy",
                c"\u{eb29} Scene###Scene",
                c"\u{ec17} Game###Game",
                c"\u{ea74} Inspector###Inspector",
                c"\u{eb30} Project###Project",
                c"\u{eb9b} Console###Console",
            ] {
                if ui.menu_item(
                    name.to_string_lossy()
                        .split("###")
                        .next()
                        .unwrap_or_default(),
                ) {
                    unsafe {
                        imgui::sys::igSetWindowFocus_Str(name.as_ptr());
                    }
                }
            }
            ui.separator();
            if ui.menu_item("Reset Layout") {
                e.reset_layout = true;
            }
            if ui
                .menu_item_config("Emulator Debugger")
                .enabled(e.playing && e.emulator_pid.is_some())
                .selected(e.emulator_visible && e.playing)
                .build()
            {
                e.emulator_visible = !e.emulator_visible;
                if let Some(pid) = e.emulator_pid {
                    crate::native::emulator_window(pid, e.emulator_visible);
                }
            }
        });
        ui.menu("Help", || {
            ui.text("Epok | PSX editor");
            ui.text("Rust / Dear ImGui / PsyQo");
        });
    });
    if !background && !text_input_active(ui) && !e.game_capture {
        crate::mesh_editor::shortcuts(ui, e);
        if !e.mesh_editor.open
            && !e.blueprint_editor.focused
            && !e.timeline_editor.focused
            && !e.project_browser.focused
            && ui.is_key_pressed(imgui::Key::F2)
        {
            e.action("rename");
        }
        if ui.io().key_ctrl && ui.is_key_pressed(imgui::Key::S) {
            e.save_all();
        }
        if !e.mesh_editor.open
            && !e.blueprint_editor.focused
            && !e.timeline_editor.focused
            && !e.project_browser.focused
            && ui.io().key_ctrl
            && ui.is_key_pressed(imgui::Key::D)
        {
            e.action("duplicate");
        }
    }
    let fixed = W::NO_TITLE_BAR
        | W::NO_MOVE
        | W::NO_RESIZE
        | W::NO_COLLAPSE
        | W::NO_DOCKING
        | W::NO_SAVED_SETTINGS
        | W::NO_SCROLLBAR;
    let _padding = ui.push_style_var(imgui::StyleVar::WindowPadding([10., 7.]));
    ui.window("Toolbar")
        .position([0., menu_height], Condition::Always)
        .size([size[0], toolbar_height], Condition::Always)
        .flags(fixed)
        .build(|| {
            imgui::Image::new(brand_texture, [22., 22.]).build(ui);
            ui.same_line_with_spacing(0., 6.);
            ui.text("Epok");
            ui.same_line();
            ui.text_disabled("/  PSX");
            ui.same_line_with_pos(((size[0] - 180.) * 0.5).max(140.));
            if icon(
                ui,
                if e.job.is_some() { STOP } else { PLAY },
                "play",
                "Play / Stop PSX",
                e.job.is_some(),
            ) {
                e.action("play");
            }
            ui.same_line_with_spacing(0., 1.);
            crate::play_ui::toolbar(ui, e);
            ui.same_line_with_spacing(0., 3.);
            ui.disabled(e.job.is_some() || e.dependencies.busy(), || {
                if crate::memory_ui::button(ui) {
                    e.show_memory_report();
                }
            });
            ui.same_line_with_spacing(0., 3.);
            ui.disabled(!e.playing || e.serial_ui.command_pending, || {
                if icon(ui, PAUSE, "pause", "Pause / Resume", e.paused) {
                    e.action("pause");
                }
            });
            ui.same_line_with_spacing(0., 1.);
            if e.active_play_target == crate::play::Target::Serial && e.job.is_some() {
                ui.disabled(!e.playing || e.serial_ui.command_pending, || {
                    if icon(ui, "R", "reset-psx", "Reset PSX (reboot console)", false) {
                        e.action("serial-reset");
                    }
                });
            } else {
                ui.disabled(!e.paused, || {
                    if icon(ui, "\u{ead6}", "step", "Advance one PSX frame", false)
                        && let Some(b) = e.job.as_ref().and_then(|j| j.bridge.as_ref())
                    {
                        b.step.store(true, std::sync::atomic::Ordering::Relaxed);
                    }
                });
            }
            ui.same_line_with_pos(size[0] - 270.);
            if ui.button("Build") {
                e.action("build");
            }
            ui.same_line();
            if ui.button("Package Disc") {
                e.action("export-disc");
            }
            ui.same_line();
            if ui.button(format!("{SEARCH}##projectsearch")) {
                unsafe {
                    imgui::sys::igSetWindowFocus_Str(c"\u{eb30} Project###Project".as_ptr());
                }
            }
            ui.same_line();
            if ui.button("Layout  \u{eab4}") {
                ui.open_popup("layout-menu");
            }
            ui.popup("layout-menu", || {
                if ui.menu_item("Default") {
                    e.reset_layout = true;
                }
            });
        });
    drop(_padding);
    let padding = ui.push_style_var(imgui::StyleVar::WindowPadding([0., 0.]));
    ui.window("Workspace")
        .position([0., workspace_top], Condition::Always)
        .size([size[0], workspace_height], Condition::Always)
        .flags(fixed | W::NO_BRING_TO_FRONT_ON_FOCUS)
        .build(|| unsafe {
            let id = imgui::sys::igGetID_Str(c"EpokDock".as_ptr());
            if reset {
                imgui::sys::igDockBuilderRemoveNode(id);
                imgui::sys::igDockBuilderAddNode(id, imgui::sys::ImGuiDockNodeFlags_DockSpace);
                imgui::sys::igDockBuilderSetNodeSize(
                    id,
                    imgui::sys::ImVec2 {
                        x: size[0],
                        y: workspace_height,
                    },
                );
                let (mut main, mut inspector, mut bottom, mut hierarchy, mut console) =
                    (id, 0, 0, 0, 0);
                imgui::sys::igDockBuilderSplitNode(
                    main,
                    imgui::sys::ImGuiDir_Right,
                    (360. / size[0]).clamp(0.24, 0.32),
                    &mut inspector,
                    &mut main,
                );
                imgui::sys::igDockBuilderSplitNode(
                    main,
                    imgui::sys::ImGuiDir_Down,
                    (330. / workspace_height).clamp(0.28, 0.45),
                    &mut bottom,
                    &mut main,
                );
                imgui::sys::igDockBuilderSplitNode(
                    main,
                    imgui::sys::ImGuiDir_Left,
                    (240. / (size[0] - 360.)).clamp(0.22, 0.29),
                    &mut hierarchy,
                    &mut main,
                );
                imgui::sys::igDockBuilderSplitNode(
                    bottom,
                    imgui::sys::ImGuiDir_Right,
                    0.5,
                    &mut console,
                    &mut bottom,
                );
                for (name, node) in [
                    (c"\u{eb86} Hierarchy###Hierarchy", hierarchy),
                    (c"\u{ea74} Inspector###Inspector", inspector),
                    (c"\u{eb29} Scene###Scene", main),
                    (c"\u{ec17} Game###Game", main),
                    (c"\u{eb30} Project###Project", bottom),
                    (c"\u{eb9b} Console###Console", console),
                ] {
                    imgui::sys::igDockBuilderDockWindow(name.as_ptr(), node);
                }
                imgui::sys::igDockBuilderFinish(id);
                *initial = false;
                e.reset_layout = false;
            }
            imgui::sys::igDockSpace(id, imgui::sys::ImVec2 { x: 0., y: 0. }, 0, std::ptr::null());
        });
    drop(padding);
    // Play can start from the toolbar in this frame. Do not dispatch editing panels afterwards.
    if !background && e.critical_busy() {
        return;
    }
    hierarchy(ui, e);
    map_settings(ui, e);
    inspector(ui, e);
    project(ui, e, asset_font);
    crate::console::draw(ui, e);
    e.scene_look = false;
    scene_view(ui, e, scene, hud_texture, image_size);
    game_view(ui, e, game);
    if std::mem::take(&mut e.focus_scene) || reset {
        unsafe {
            if reset {
                imgui::sys::igSetWindowFocus_Str(c"\u{eb30} Project###Project".as_ptr());
            }
            imgui::sys::igSetWindowFocus_Str(c"\u{eb29} Scene###Scene".as_ptr());
        }
    }
    if e.focus_console {
        unsafe {
            imgui::sys::igSetWindowFocus_Str(c"\u{eb9b} Console###Console".as_ptr());
        }
        e.focus_console = false;
    }
    if e.focus_game {
        unsafe {
            imgui::sys::igSetWindowFocus_Str(c"\u{ec17} Game###Game".as_ptr());
        }
        e.focus_game = false;
    }
    // Select Project once per opened project, including layouts saved on Console.
    // Explicit Console/Game requests on later frames continue to work normally.
    if std::mem::take(&mut e.focus_project) || reset {
        unsafe {
            imgui::sys::igSetWindowFocus_Str(c"\u{eb30} Project###Project".as_ptr());
        }
    }
    ui.window("Status")
        .position([0., size[1] - status_height], Condition::Always)
        .size([size[0], status_height], Condition::Always)
        .flags(fixed)
        .build(|| {
            let text = if let Some(error) = &e.last_error {
                error.clone()
            } else if e.job_stale {
                "Sources changed. Stopping the previous build / Play.".into()
            } else if e.paused {
                "Paused".into()
            } else if e.playing {
                if e.active_play_target == crate::play::Target::Serial {
                    "Playing on PSX / serial".into()
                } else {
                    "Playing on PCSX-Redux".into()
                }
            } else if e.job.is_some() {
                "Compiling C++ scripts...".into()
            } else if e.pending_build {
                "Source changes pending".into()
            } else {
                format!("{}{}", e.scene.name, if e.dirty { " *" } else { "" })
            };
            let build_sizes = e
                .memory
                .summary
                .as_ref()
                .filter(|_| {
                    e.memory.status_current
                        && !e.memory.stale
                        && e.memory.profile == e.play_profile
                        && e.memory.debug == e.blueprint_debug_enabled
                        && !e.pending_build
                        && (e.job.is_none() || e.playing)
                })
                .map(|summary| summary.status());
            let right = build_sizes.as_deref().unwrap_or("PSX  |  C++20  |  PsyQo");
            let width = (ui.calc_text_size(right)[0] + 12.).min(size[0] * 0.7);
            clipped(
                ui,
                &format!("\u{ea71}  {text}"),
                (size[0] - width - 28.).max(1.),
            );
            ui.same_line_with_pos(size[0] - width - 10.);
            clipped(ui, right, width);
            if ui.is_item_hovered() {
                ui.tooltip_text(right);
            }
        });
    crate::asset_ui::windows(ui, e);
    crate::play_ui::warning(ui, e);
    e.artifact_dependencies.draw(ui, &e.root);
    crate::mesh_editor::window(ui, e);
    crate::settings_ui::windows(ui, e);
    if !background && e.critical_busy() {
        return;
    }
    crate::serial_ui::window(ui, e);
    if !background && e.critical_busy() {
        return;
    }
    crate::export_ui::window(ui, e);
    script_creation_dialog(ui, e);
    crate::blueprint_workflow::draw(ui, e);
    crate::actor_workflow::draw(ui, e);
    e.timeline_editor
        .draw(ui, &e.root, &e.class_registry, &e.scene, &e.assets.index);
    if e.close_requested {
        ui.open_popup("Unsaved scene");
        e.close_requested = false;
    }
    ui.modal_popup_config("Unsaved scene")
        .always_auto_resize(true)
        .build(|| {
            ui.text("Save changes before closing?");
            if ui.button("Save and close") && e.save_all() {
                e.should_close = true;
                ui.close_current_popup();
            }
            ui.same_line();
            if ui.button("Discard") {
                e.blueprint_editor.discard();
                e.timeline_editor.discard();
                e.should_close = true;
                ui.close_current_popup();
            }
            ui.same_line();
            if ui.button("Cancel") {
                ui.close_current_popup();
            }
        });
}

fn request_exit(e: &mut Editor) {
    if e.critical_busy() {
        e.log("Wait for the current operation before exiting.");
    } else if e.has_unsaved_changes() {
        e.close_requested = true;
    } else {
        e.should_close = true;
    }
}

#[cfg(test)]
mod menu_tests {
    use super::*;

    #[test]
    fn exit_closes_clean_projects_and_confirms_dirty_projects() {
        let mut clean = Editor::new(crate::workspace::tests::temp("clean-menu-exit"));
        clean.dirty = false;
        clean.blueprint_editor.discard();
        clean.timeline_editor.discard();
        request_exit(&mut clean);
        assert!(clean.should_close);
        assert!(!clean.close_requested);

        let mut dirty = Editor::new(crate::workspace::tests::temp("dirty-menu-exit"));
        dirty.blueprint_editor.discard();
        dirty.timeline_editor.discard();
        dirty.dirty = true;
        request_exit(&mut dirty);
        assert!(!dirty.should_close);
        assert!(dirty.close_requested);
    }
}

fn parent_menu(
    ui: &imgui::Ui,
    e: &Editor,
    index: usize,
    keep_world: bool,
) -> Option<Option<usize>> {
    let mut requested = None;
    if ui
        .menu_item_config("None (Scene root)")
        .selected(e.scene.actors[index].parent.is_none())
        .build()
    {
        requested = Some(None);
    }
    for (p, entity) in e.scene.actors.iter().enumerate() {
        if !e.scene.is_descendant(p, index) {
            let _id = ui.push_id_usize(p);
            if ui
                .menu_item_config(&entity.name)
                .selected(e.scene.actors[index].parent == Some(p))
                .build()
            {
                requested = Some(Some(p));
            }
        }
    }
    if keep_world {
        ui.separator();
        ui.text_disabled("Preserves world transform");
    }
    requested
}
/// One placeable actor class offered by the Hierarchy's Actor submenu.
struct ActorClass {
    domain: crate::reflection_schema::Domain,
    class: String,
    label: String,
}
/// Placeable actor classes, grouped later by domain. Empty when the project's
/// reflection data does not resolve into a model.
fn placeable_actor_classes(e: &Editor) -> Vec<ActorClass> {
    let Some(model) = e.object_model() else {
        return Vec::new();
    };
    let mut classes = model
        .placeable()
        .map(|c| ActorClass {
            domain: c.domain,
            class: c.cpp_name.clone(),
            label: c
                .cpp_name
                .rsplit("::")
                .next()
                .unwrap_or(&c.cpp_name)
                .to_owned(),
        })
        .collect::<Vec<_>>();
    classes.sort_by(|a, b| a.label.cmp(&b.label));
    classes
}
/// The domain of a legacy entity. UI and 3D authoring data claim their
/// respective spatial domain; a bare Empty with only shared components such as
/// scripts or audio is domain-neutral and remains available in every view.
fn entity_domain(entity: &crate::scene::Actor) -> crate::reflection_schema::Domain {
    use crate::reflection_schema::Domain;
    if entity.canvas.is_some() || entity.rect.is_some() {
        Domain::UI
    } else if entity.kind != "Empty"
        || entity.sprite.is_some()
        || entity.light.is_some()
        || entity.collider.is_some()
        || entity.editable_mesh.is_some()
        || entity.skeletal_mesh.is_some()
        || entity.particle_emitter.is_some()
        || entity.particle_effect.is_some()
        || entity.blob_shadow.is_some()
    {
        Domain::World3D
    } else {
        Domain::None
    }
}
/// Domain-neutral objects are logic/resources shared by every authoring view.
fn domain_visible(
    domain: crate::reflection_schema::Domain,
    mode: crate::reflection_schema::Domain,
) -> bool {
    domain == crate::reflection_schema::Domain::None || domain == mode
}
#[cfg(test)]
mod hierarchy_domain_tests {
    use super::*;
    use crate::reflection_schema::Domain;

    #[test]
    fn procedural_ui_controllers_and_resources_remain_visible_in_every_view() {
        let mut entity = crate::scene::Actor::cube("Menu Director".into());
        entity.kind = "Empty".into();
        entity.set_class_defaults(&(Default::default()));
        entity.audio = Some(Default::default());
        entity.material.texture = Some(uuid::Uuid::new_v4());

        let domain = entity_domain(&entity);
        assert_eq!(domain, Domain::None);
        for mode in crate::scene_view_mode::SceneViewMode::ALL {
            assert!(domain_visible(domain, mode.domain()), "{}", mode.label());
        }
    }

    #[test]
    fn spatial_legacy_entities_still_follow_their_authoring_view() {
        let mesh = crate::scene::Actor::cube("Mesh".into());
        assert_eq!(entity_domain(&mesh), Domain::World3D);
        assert!(domain_visible(Domain::World3D, Domain::World3D));
        assert!(!domain_visible(Domain::World3D, Domain::UI));

        let mut canvas = crate::scene::Actor::cube("Canvas".into());
        canvas.kind = "Empty".into();
        canvas.canvas = Some(Default::default());
        assert_eq!(entity_domain(&canvas), Domain::UI);
        assert!(domain_visible(Domain::UI, Domain::UI));
        assert!(!domain_visible(Domain::UI, Domain::World3D));
    }
}
/// The domain of a document actor. `None` means the class did not resolve; the
/// caller shows it in the 3D view with a "class unresolved" tooltip rather than
/// hiding it, so an actor is never invisible in every mode.
fn actor_domain(
    actor: &crate::actor_document::ActorInstance,
    model: Option<&crate::object_model::Model>,
) -> Option<crate::reflection_schema::Domain> {
    model.and_then(|m| actor.class.resolve(m)).map(|c| c.domain)
}
fn creation_menu(
    ui: &imgui::Ui,
    child: bool,
    _actors: &[ActorClass],
    _actor: &mut Option<String>,
) -> Option<&'static str> {
    ui.menu_item(if child {
        "Instantiate Child Actor..."
    } else {
        "Instantiate Actor..."
    })
    .then_some(if child {
        "instantiate-child-actor"
    } else {
        "instantiate-actor"
    })
}
/// Everything the Hierarchy computes once per frame and every node reads.
struct HierarchyContext<'a> {
    children: &'a [Vec<usize>],
    /// Matched the search filter, directly or through a descendant.
    visible: &'a [bool],
    /// Belongs to the current view mode or is domain-neutral.
    in_domain: &'a [bool],
    /// Has a descendant in that domain, so it is drawn as a presentation root.
    passthrough: &'a [bool],
    actors: &'a [ActorClass],
}
#[derive(Default)]
struct HierarchyResult {
    reparent: Option<(usize, Option<usize>, bool)>,
    action: Option<(usize, &'static str)>,
    /// A creation command issued from the map root, which has no index.
    root_action: Option<&'static str>,
    actor_class: Option<String>,
    open_map_settings: bool,
}
fn hierarchy_node(
    ui: &imgui::Ui,
    e: &mut Editor,
    index: usize,
    cx: &HierarchyContext,
    out: &mut HierarchyResult,
) {
    if !cx.visible[index] || (!cx.in_domain[index] && !cx.passthrough[index]) {
        return;
    }
    let entity = &e.scene.actors[index];
    let label = format!(
        "{} {}###entity{}",
        if entity.kind == "Camera" {
            CAMERA
        } else {
            CUBE
        },
        entity.name,
        index
    );
    let mut flags = imgui::TreeNodeFlags::SPAN_AVAIL_WIDTH
        | imgui::TreeNodeFlags::OPEN_ON_ARROW
        | imgui::TreeNodeFlags::DEFAULT_OPEN;
    if cx.children[index].is_empty() {
        flags |= imgui::TreeNodeFlags::LEAF;
    }
    // An ancestor of another domain keeps the branch's shape and order without
    // becoming selectable, a rename target or a drop target.
    if !cx.in_domain[index] {
        let _muted = ui.push_style_color(C::Text, gray(120));
        let node = ui
            .tree_node_config(label)
            .flags(flags & !imgui::TreeNodeFlags::LEAF)
            .push();
        drop(_muted);
        if ui.is_item_hovered() {
            ui.tooltip_text(format!(
                "{} is not part of the {} view; shown so its children keep their place.",
                e.scene.actors[index].name,
                e.scene_view_mode.label()
            ));
        }
        if let Some(_node) = node {
            for child in &cx.children[index] {
                hierarchy_node(ui, e, *child, cx, out);
            }
        }
        return;
    }
    if e.selected == Some(index) {
        flags |= imgui::TreeNodeFlags::SELECTED;
    }
    let mut config = ui.tree_node_config(label).flags(flags);
    if !e.search.is_empty()
        || e.reveal_selected && e.selected.is_some_and(|i| e.scene.is_descendant(i, index))
    {
        config = config.opened(true, Condition::Always);
    }
    let node = config.push();
    if e.reveal_selected && e.selected == Some(index) {
        ui.set_scroll_here_y();
    }
    if ui.is_item_hovered() {
        ui.tooltip_text(&e.scene.actors[index].name);
    }
    if ui.is_item_hovered()
        && !ui.is_item_toggled_open()
        && ui.is_mouse_double_clicked(imgui::MouseButton::Left)
    {
        e.begin_rename(index);
    }
    if ui.is_item_clicked() && !ui.is_item_toggled_open() {
        e.selected_asset = None;
        e.selected_actor = None;
        e.selected = Some(index);
        e.view_dirty = true;
    }
    if !e.playing {
        if let Some(_source) = ui
            .drag_drop_source_config("EPOK_ENTITY")
            .begin_payload(index)
        {
            ui.text(&e.scene.actors[index].name);
            ui.text_disabled("Drop on parent; drop on scene to unparent");
        }
        if let Some(target) = ui.drag_drop_target()
            && let Some(Ok(payload)) =
                target.accept_payload::<usize, _>("EPOK_ENTITY", imgui::DragDropFlags::empty())
            && payload.delivery
        {
            out.reparent = Some((payload.data, Some(index), true));
        }
    }
    if let Some(_popup) = ui.begin_popup_context_item() {
        e.selected_asset = None;
        e.selected_actor = None;
        e.selected = Some(index);
        e.view_dirty = true;
        ui.disabled(e.playing, || {
            if ui.menu_item("Rename") {
                e.begin_rename(index);
            }
            ui.separator();
            let mut actor_class = None;
            if let Some(command) = creation_menu(ui, true, cx.actors, &mut actor_class) {
                out.action = Some((index, command));
            }
            if actor_class.is_some() {
                out.actor_class = actor_class;
            }
            ui.separator();
            if let Some(_menu) = ui.begin_menu("Parent > Keep World")
                && let Some(parent) = parent_menu(ui, e, index, true)
            {
                out.reparent = Some((index, parent, true));
            }
            if let Some(_menu) = ui.begin_menu("Parent > Keep Local")
                && let Some(parent) = parent_menu(ui, e, index, false)
            {
                out.reparent = Some((index, parent, false));
            }
            ui.separator();
            if ui.menu_item("Duplicate") {
                out.action = Some((index, "duplicate"));
            }
            if ui.menu_item("Delete") {
                out.action = Some((index, "delete"));
            }
            if !cx.children[index].is_empty() {
                ui.text_disabled("Duplicate / Delete includes children");
            }
        });
    }
    if e.rename.as_ref().is_some_and(|(i, _)| *i == index) {
        ui.set_next_item_width(-1.);
        if e.rename_focus {
            ui.set_keyboard_focus_here();
            e.rename_focus = false;
        }
        let submit = ui
            .input_text(
                format!("##rename{index}"),
                &mut e.rename.as_mut().unwrap().1,
            )
            .auto_select_all(true)
            .enter_returns_true(true)
            .build();
        let blur = ui.is_item_deactivated();
        let cancel = ui.is_key_pressed(imgui::Key::Escape);
        if submit || blur || cancel {
            e.finish_rename(!cancel);
        }
    }
    if let Some(_node) = node {
        for child in &cx.children[index] {
            hierarchy_node(ui, e, *child, cx, out);
        }
    }
}
/// Domain-filtered document actors, drawn under the legacy actors.
struct ActorContext {
    children: Vec<Vec<usize>>,
    visible: Vec<bool>,
    in_domain: Vec<bool>,
    passthrough: Vec<bool>,
    resolved: Vec<bool>,
    roots: Vec<usize>,
}
fn actor_context(e: &Editor) -> ActorContext {
    let actors = &e.scene.actors;
    let model = e.object_model();
    let mode = e.scene_view_mode.domain();
    let index_of: std::collections::HashMap<uuid::Uuid, usize> =
        actors.iter().enumerate().map(|(i, a)| (a.id, i)).collect();
    let mut cx = ActorContext {
        children: vec![Vec::new(); actors.len()],
        visible: vec![false; actors.len()],
        in_domain: vec![false; actors.len()],
        passthrough: vec![false; actors.len()],
        resolved: vec![false; actors.len()],
        roots: Vec::new(),
    };
    let mut parent = vec![None; actors.len()];
    for (i, actor) in actors.iter().enumerate() {
        match actor
            .logical_parent
            .and_then(|id| index_of.get(&id))
            .copied()
            .filter(|p| *p != i)
        {
            Some(p) => {
                cx.children[p].push(i);
                parent[i] = Some(p);
            }
            None => cx.roots.push(i),
        }
        let domain = actor_domain(actor, model.as_deref());
        cx.resolved[i] = domain.is_some();
        cx.in_domain[i] = domain_visible(
            domain.unwrap_or(crate::reflection_schema::Domain::World3D),
            mode,
        );
    }
    let query = e.search.to_lowercase();
    for (i, actor) in actors.iter().enumerate() {
        if actor.name.to_lowercase().contains(&query) {
            let mut current = Some(i);
            while let Some(p) = current {
                cx.visible[p] = true;
                current = parent[p];
            }
        }
    }
    // A logical parent outside the current domain still holds the branch's place.
    for i in 0..actors.len() {
        if cx.in_domain[i] {
            let mut current = parent[i];
            while let Some(p) = current {
                cx.passthrough[p] = true;
                current = parent[p];
            }
        }
    }
    cx
}
/// Actor commands collected while the tree is drawn and applied once, after the
/// walk, so the document is never mutated under an open tree node.
#[derive(Default)]
struct ActorResult {
    /// `(child, new parent)`; `None` parent is the map root.
    reparent: Option<(uuid::Uuid, Option<uuid::Uuid>)>,
    duplicate: Option<uuid::Uuid>,
    delete: Option<uuid::Uuid>,
    rename: Option<uuid::Uuid>,
}
fn actor_node(
    ui: &imgui::Ui,
    e: &mut Editor,
    index: usize,
    cx: &ActorContext,
    out: &mut ActorResult,
) {
    if !cx.visible[index] || (!cx.in_domain[index] && !cx.passthrough[index]) {
        return;
    }
    let actor = &e.scene.actors[index];
    let id = actor.id;
    let label = format!("{ACTOR} {}###actor{}", actor.name, id);
    let mut flags = imgui::TreeNodeFlags::SPAN_AVAIL_WIDTH
        | imgui::TreeNodeFlags::OPEN_ON_ARROW
        | imgui::TreeNodeFlags::DEFAULT_OPEN;
    if cx.children[index].is_empty() {
        flags |= imgui::TreeNodeFlags::LEAF;
    }
    if !cx.in_domain[index] {
        let _muted = ui.push_style_color(C::Text, gray(120));
        let node = ui
            .tree_node_config(label)
            .flags(flags & !imgui::TreeNodeFlags::LEAF)
            .push();
        drop(_muted);
        if let Some(_node) = node {
            for child in &cx.children[index] {
                actor_node(ui, e, *child, cx, out);
            }
        }
        return;
    }
    if e.selected_actor == Some(id) {
        flags |= imgui::TreeNodeFlags::SELECTED;
    }
    let node = ui.tree_node_config(label).flags(flags).push();
    if ui.is_item_hovered() {
        if cx.resolved[index] {
            ui.tooltip_text(&e.scene.actors[index].class.name);
        } else {
            ui.tooltip_text(format!(
                "{} — class unresolved ({})",
                e.scene.actors[index].name, e.scene.actors[index].class.name
            ));
        }
    }
    if ui.is_item_hovered()
        && !ui.is_item_toggled_open()
        && ui.is_mouse_double_clicked(imgui::MouseButton::Left)
    {
        out.rename = Some(id);
    }
    if ui.is_item_clicked() && !ui.is_item_toggled_open() {
        e.select_actor(Some(id));
    }
    if !e.playing {
        if let Some(_source) = ui
            .drag_drop_source_config("EPOK_ACTOR")
            .begin_payload(index)
        {
            ui.text(&e.scene.actors[index].name);
            ui.text_disabled("Drop on an actor to parent; drop on the map to unparent");
        }
        if let Some(target) = ui.drag_drop_target()
            && let Some(Ok(payload)) =
                target.accept_payload::<usize, _>("EPOK_ACTOR", imgui::DragDropFlags::empty())
            && payload.delivery
            && let Some(child) = e.scene.actors.get(payload.data).map(|a| a.id)
        {
            out.reparent = Some((child, Some(id)));
        }
    }
    if let Some(_popup) = ui.begin_popup_context_item() {
        e.select_actor(Some(id));
        ui.disabled(e.playing, || {
            if ui.menu_item("Rename") {
                out.rename = Some(id);
            }
            ui.separator();
            if ui.menu_item("Duplicate") {
                out.duplicate = Some(id);
            }
            if ui.menu_item("Delete") {
                out.delete = Some(id);
            }
            if !cx.children[index].is_empty() {
                ui.text_disabled("Duplicate / Delete includes child actors");
            }
            ui.separator();
            if ui.menu_item("Move to Map Root") {
                out.reparent = Some((id, None));
            }
            ui.text_disabled("Parent by dragging an actor onto another");
            ui.separator();
            if ui.menu_item("Instantiate Child Actor...") {
                e.action("instantiate-child-actor");
            }
            if ui.menu_item("Convert to Actor Blueprint...") {
                crate::actor_workflow::begin_convert(e, id);
            }
        });
    }
    if e.actor_rename
        .as_ref()
        .is_some_and(|(other, _)| *other == id)
    {
        ui.set_next_item_width(-1.);
        if e.actor_rename_focus {
            ui.set_keyboard_focus_here();
            e.actor_rename_focus = false;
        }
        let submit = ui
            .input_text(
                format!("##actor-rename{id}"),
                &mut e.actor_rename.as_mut().unwrap().1,
            )
            .auto_select_all(true)
            .enter_returns_true(true)
            .build();
        let blur = ui.is_item_deactivated();
        let cancel = ui.is_key_pressed(imgui::Key::Escape);
        if submit || blur || cancel {
            e.finish_actor_rename(!cancel);
        }
    }
    if let Some(_node) = node {
        for child in &cx.children[index] {
            actor_node(ui, e, *child, cx, out);
        }
    }
}
fn hierarchy(ui: &imgui::Ui, e: &mut Editor) {
    let actors = placeable_actor_classes(e);
    ui.window("\u{eb86} Hierarchy###Hierarchy").build(|| {
        let mut out = HierarchyResult::default();
        let mut actor_out = ActorResult::default();
        if ui.button("\u{ea60} \u{eab4}") {
            ui.open_popup("create-object");
        }
        ui.popup("create-object", || {
            ui.disabled(e.playing, || {
                let mut actor_class = None;
                if let Some(command) = creation_menu(ui, false, &actors, &mut actor_class) {
                    e.action(command);
                }
                if let Some(class) = actor_class {
                    e.create_actor(&class);
                }
            });
        });
        ui.same_line();
        ui.set_next_item_width(-1.);
        ui.input_text("##hierarchy-search", &mut e.search)
            .hint(format!("{SEARCH} All"))
            .build();
        ui.separator();
        let mut scene_config = ui
            .tree_node_config(format!(
                "{CUBE} {}{}###scene-root",
                e.scene.name,
                if e.dirty { " *" } else { "" }
            ))
            .flags(imgui::TreeNodeFlags::DEFAULT_OPEN | imgui::TreeNodeFlags::SPAN_AVAIL_WIDTH);
        if e.reveal_selected || !e.search.is_empty() {
            scene_config = scene_config.opened(true, Condition::Always);
        }
        let scene_node = scene_config.push();
        // The map root is the document itself, not an instance: it is neither
        // selectable nor a reparent target. Clicking it opens Map Settings.
        if ui.is_item_hovered() {
            ui.tooltip_text("Map root — click to open Map Settings");
        }
        if ui.is_item_clicked() && !ui.is_item_toggled_open() {
            out.open_map_settings = true;
        }
        if let Some(_popup) = ui.begin_popup_context_item() {
            ui.disabled(e.playing, || {
                let mut actor_class = None;
                if let Some(command) = creation_menu(ui, false, &actors, &mut actor_class) {
                    out.root_action = Some(command);
                }
                if actor_class.is_some() {
                    out.actor_class = actor_class;
                }
                ui.separator();
                if ui.menu_item("Map Settings...") {
                    out.open_map_settings = true;
                }
            });
        }
        if let Some(_scene) = scene_node {
            let actor_cx = actor_context(e);
            if !actor_cx.roots.is_empty() {
                for root in actor_cx.roots.clone() {
                    actor_node(ui, e, root, &actor_cx, &mut actor_out);
                }
            }
        }
        ui.invisible_button(
            "hierarchy-root-drop",
            [
                ui.content_region_avail()[0].max(1.),
                ui.content_region_avail()[1].max(24.),
            ],
        );
        if !e.playing
            && let Some(target) = ui.drag_drop_target()
        {
            if let Some(Ok(payload)) =
                target.accept_payload::<usize, _>("EPOK_ENTITY", imgui::DragDropFlags::empty())
                && payload.delivery
            {
                out.reparent = Some((payload.data, None, true));
            } else if let Some(Ok(payload)) =
                target.accept_payload::<usize, _>("EPOK_ACTOR", imgui::DragDropFlags::empty())
                && payload.delivery
                && let Some(child) = e.scene.actors.get(payload.data).map(|a| a.id)
            {
                actor_out.reparent = Some((child, None));
            }
        }
        if let Some(_popup) = ui.begin_popup_context_item() {
            ui.disabled(e.playing, || {
                let mut actor_class = None;
                if let Some(command) = creation_menu(ui, false, &actors, &mut actor_class) {
                    out.root_action = Some(command);
                }
                if actor_class.is_some() {
                    out.actor_class = actor_class;
                }
            });
        }
        e.reveal_selected = false;
        if out.open_map_settings {
            e.map_settings = true;
        }
        if let Some(class) = out.actor_class {
            e.create_actor(&class);
        }
        if let Some((index, parent, keep)) = out.reparent {
            e.reparent(index, parent, keep);
        }
        if let Some(command) = out.root_action {
            e.action(command);
        }
        if let Some((index, command)) = out.action {
            e.selected = Some(index);
            e.selected_actor = None;
            e.action(command);
        }
        if let Some(id) = actor_out.rename {
            e.begin_actor_rename(id);
        }
        if let Some((child, parent)) = actor_out.reparent {
            e.reparent_actor(child, parent);
        }
        if let Some(id) = actor_out.duplicate {
            e.duplicate_actor(id);
        }
        if let Some(id) = actor_out.delete {
            e.delete_actor(id);
        }
        if ui.is_window_focused()
            && !ui.io().want_text_input
            && ui.is_key_pressed(imgui::Key::Delete)
        {
            // The Hierarchy's Delete key follows the selection, whichever kind
            // of node holds it.
            match e.selected_actor {
                Some(id) => e.delete_actor(id),
                None => e.action("delete"),
            }
        }
        scene_shortcuts(ui, e);
    });
}
/// Ctrl+Z / Ctrl+Y for scene edits, active only while an authoring window has
/// focus and the game is not running. The component-attachment stack keeps
/// priority inside `Editor::undo_scene`.
pub(crate) fn scene_shortcuts(ui: &imgui::Ui, e: &mut Editor) {
    if e.playing
        || !ui.is_window_focused_with_flags(imgui::WindowFocusedFlags::ROOT_AND_CHILD_WINDOWS)
        || ui.io().want_text_input
        || !ui.io().key_ctrl
    {
        return;
    }
    let redo =
        ui.is_key_pressed(imgui::Key::Y) || (ui.io().key_shift && ui.is_key_pressed(imgui::Key::Z));
    let undo = !redo && !ui.io().key_shift && ui.is_key_pressed(imgui::Key::Z);
    if !redo && !undo {
        return;
    }
    if let Err(error) = if redo { e.redo_scene() } else { e.undo_scene() } {
        e.log(error);
    }
}
/// Marker colour for an authored override, shared by the actor and component
/// property tables.
const OVERRIDE_COLOUR: [f32; 4] = [0.55, 0.78, 0.35, 1.];

/// One edit requested by a property table. At most one happens per frame, which
/// keeps the table a pure renderer over a borrowed document.
enum PropertyAction {
    /// An edited value. It becomes an override even when it equals the default:
    /// "I chose this" and "I did not choose" are different authored states.
    Set(String, serde_json::Value),
    /// Drops the override and lets the class default show through again.
    Reset(String),
    /// Removes a preserved value whose property the class no longer declares.
    DiscardOrphan(String),
}

/// The reflected properties of one class chain, edited with the same value
/// editors the ClassDefaults inspector uses.
///
/// `class_name` is a class id or `cpp_name`; the properties come from
/// `Registry::properties`, which already flattens the chain. Values the class
/// does not declare are shown as preserved orphans rather than dropped.
fn reflected_property_table(
    ui: &imgui::Ui,
    e: &Editor,
    id_prefix: &str,
    class_name: &str,
    properties: &std::collections::BTreeMap<String, serde_json::Value>,
    overrides: &std::collections::BTreeSet<String>,
) -> Option<PropertyAction> {
    let cpp_name = e
        .object_model()
        .and_then(|model| model.class(class_name).map(|c| c.cpp_name.clone()))
        .unwrap_or_else(|| class_name.to_owned());
    let declared = e.class_registry.properties(&cpp_name);
    let mut action = None;
    if declared.is_empty() && properties.is_empty() {
        ui.text_disabled("No reflected properties");
    }
    for property in &declared {
        if !property.editable {
            continue;
        }
        let mut value = properties
            .get(&property.name)
            .cloned()
            .unwrap_or_else(|| property.default.clone());
        let mut chars = property.name.chars();
        let label = chars
            .next()
            .map(|c| c.to_uppercase().to_string())
            .unwrap_or_default()
            + chars.as_str();
        ui.align_text_to_frame_padding();
        let _ = field(ui, &label);
        ui.set_next_item_width(-1.);
        if crate::blueprint_refs::inspector(
            ui,
            &format!("##{id_prefix}-{}", property.name),
            &mut value,
            &property.value_type,
            &e.scene,
            &e.class_registry,
            &e.assets.index,
        ) {
            action = Some(PropertyAction::Set(property.name.clone(), value));
        }
        if overrides.contains(&property.name) {
            ui.text_colored(OVERRIDE_COLOUR, "override");
            ui.same_line();
            if ui.small_button(format!("Reset to inherited##{id_prefix}-{}", property.name)) {
                action = Some(PropertyAction::Reset(property.name.clone()));
            }
        }
    }
    for key in properties
        .keys()
        .filter(|key| !declared.iter().any(|p| p.name == **key))
    {
        ui.text_colored(
            [1., 0.72, 0.3, 1.],
            format!("Preserved orphan: {key} = {}", properties[key]),
        );
        if ui.small_button(format!("Discard orphan##{id_prefix}-{key}")) {
            action = Some(PropertyAction::DiscardOrphan(key.clone()));
        }
    }
    action
}

/// Applies a [`PropertyAction`] to one property/override pair.
fn apply_property_action(
    action: PropertyAction,
    properties: &mut std::collections::BTreeMap<String, serde_json::Value>,
    overrides: &mut std::collections::BTreeSet<String>,
) {
    match action {
        PropertyAction::Set(key, value) => {
            properties.insert(key.clone(), value);
            overrides.insert(key);
        }
        PropertyAction::Reset(key) | PropertyAction::DiscardOrphan(key) => {
            properties.remove(&key);
            overrides.remove(&key);
        }
    }
}

fn edit_class_button(
    ui: &imgui::Ui,
    e: &mut Editor,
    reference: &crate::actor_document::ClassReference,
) {
    let key = reference.class_id.as_deref().unwrap_or(&reference.name);
    let class = e
        .class_registry
        .classes
        .get(key)
        .or_else(|| e.class_registry.named(key))
        .cloned();
    if let Some(class) = class {
        if class.provider.id != "blueprint"
            && crate::scripts::editable_class_source(&e.root, &class).is_none()
        {
            ui.text_disabled("Engine class (read-only)");
            return;
        }
        let label = if class.provider.id == "blueprint" {
            "Open Blueprint"
        } else {
            "Edit C++ Class"
        };
        if script_button(ui, label) {
            crate::blueprint_workflow::edit_binding(
                e,
                &crate::scene::ClassDefaults {
                    name: class.cpp_name,
                    class_id: Some(class.id),
                    provider: class.provider,
                    backend: class.backend,
                    ..Default::default()
                },
            );
        }
    }
}

fn actor_header(ui: &imgui::Ui, actor: &mut crate::scene::Actor, glyph: &str) -> bool {
    ui.align_text_to_frame_padding();
    ui.text(glyph);
    ui.same_line();
    let mut changed = false;
    if icon(ui, "A", "actor-active", "Active", actor.active) {
        actor.active = !actor.active;
        changed = true;
    }
    #[cfg(test)]
    record_script_control(ui, "Active");
    ui.same_line();
    let disabled = ui.begin_disabled(actor.skeletal_mesh.is_some());
    if icon(
        ui,
        "S",
        "actor-static",
        "Static",
        actor.lighting.static_geometry,
    ) {
        actor.lighting.static_geometry = !actor.lighting.static_geometry;
        if !actor.lighting.static_geometry {
            actor.lighting.receive = crate::lighting::Receive::Realtime;
        }
        changed = true;
    }
    drop(disabled);
    #[cfg(test)]
    record_script_control(ui, "Static");
    ui.same_line();
    ui.set_next_item_width(-1.);
    changed |= ui.input_text("##name", &mut actor.name).build();
    #[cfg(test)]
    record_script_control(ui, "Actor name");
    changed
}

/// The actor class and component scripts associated with an existing mesh/entity
/// must be visible when that object is selected in the viewport.
fn entity_actor_inspector(ui: &imgui::Ui, e: &mut Editor, entity: usize) {
    let Some(actor) = crate::actor_scripts::owner(&e.scene, entity).cloned() else {
        return;
    };
    let index = e.scene.actor_index(actor.id).unwrap();
    let _id = ui.push_id("entity-actor");
    if heading(ui, &format!("{CODE} {} (Actor Class)", actor.class.name)) {
        muted(
            ui,
            "Defines this object's actor type. Components add reusable behavior below.",
        );
        edit_class_button(ui, e, &actor.class);
        if let Some(action) = reflected_property_table(
            ui,
            e,
            "entity-actor",
            actor.class.class_id.as_deref().unwrap_or(&actor.class.name),
            &actor.properties,
            &actor.overrides,
        ) {
            let target = &mut e.scene.actors[index];
            apply_property_action(action, &mut target.properties, &mut target.overrides);
            e.changed();
        }
    }
    for component in &actor.components {
        if component.root
            || component
                .class
                .class_id
                .as_deref()
                .is_some_and(crate::actor_components::native)
        {
            continue;
        }
        let _id = ui.push_id(component.id.to_string());
        if heading(ui, &format!("{CODE} {} (Actor Component)", component.name)) {
            ui.text_disabled(&component.class.name);
            edit_class_button(ui, e, &component.class);
            if let Some(action) = reflected_property_table(
                ui,
                e,
                "entity-component",
                component
                    .class
                    .class_id
                    .as_deref()
                    .unwrap_or(&component.class.name),
                &component.properties,
                &component.overrides,
            ) {
                let target = e.scene.actors[index]
                    .components
                    .iter_mut()
                    .find(|c| c.id == component.id)
                    .unwrap();
                apply_property_action(action, &mut target.properties, &mut target.overrides);
                e.changed();
            }
            let _disabled = ui.begin_disabled(component.inherited);
            if script_button(ui, "Remove Component") {
                e.remove_actor_component(actor.id, component.id);
            }
        }
    }
    ui.separator();
}

/// The Inspector for a selected P4 document actor: identity, the editable
/// component set and the reflected properties of the actor and each component.
fn actor_inspector(ui: &imgui::Ui, e: &mut Editor) {
    let Some(id) = e.selected_actor else {
        return;
    };
    let Some(index) = e.scene.actors.iter().position(|a| a.id == id) else {
        e.selected_actor = None;
        ui.text_disabled("Nothing selected");
        return;
    };
    let model = e.object_model();
    let resolved = model
        .as_deref()
        .and_then(|m| e.scene.actors[index].class.resolve(m))
        .map(|c| (c.cpp_name.clone(), c.domain));
    ui.disabled(e.playing, || {
        let mut actor = e.scene.actors[index].clone();
        if actor_header(ui, &mut actor, ACTOR) {
            e.scene.actors[index] = actor;
            e.changed();
        }
        ui.separator();
        if heading(ui, "\u{eb5b} Actor") {
            let class = e.scene.actors[index].class.clone();
            edit_class_button(ui, e, &class);
            let _ = field(ui, "Class");
            match &resolved {
                Some((cpp_name, domain)) => {
                    ui.text(cpp_name);
                    let _ = field(ui, "Domain");
                    ui.text(domain.label());
                }
                None => {
                    ui.text_colored(
                        [1., 0.72, 0.3, 1.],
                        &e.scene.actors[index].class.name,
                    );
                    muted(ui, "Class unresolved: shown in the 3D view until the project's reflection data resolves it.");
                }
            }
            let _ = field(ui, "Identity");
            ui.text_disabled(e.scene.actors[index].id.to_string());
            if let Some(parent) = e.scene.actors[index]
                .logical_parent
                .and_then(|p| e.scene.actors.iter().find(|a| a.id == p))
                .map(|a| a.name.clone())
            {
                let _ = field(ui, "Parent");
                ui.text(parent);
            }
        }
        if heading(ui, "\u{eb65} Components") {
            if e.scene.actors[index].components.is_empty() {
                ui.text_disabled("No components");
            }
            // One edit per frame keeps the tables pure renderers over a
            // borrowed document; the mutation happens after the whole section.
            let mut remove = None;
            let mut edit = None;
            let components: Vec<_> = e.scene.actors[index]
                .components
                .iter()
                .map(|c| {
                    (
                        c.id,
                        c.name.clone(),
                        c.class.clone(),
                        c.root,
                        c.inherited,
                        c.properties.clone(),
                        c.overrides.clone(),
                    )
                })
                .collect();
            for (id, name, class, root, inherited, properties, overrides) in &components {
                let _id = ui.push_id(id.to_string());
                let open = ui.tree_node_config(format!("{name}##component"))
                    .flags(imgui::TreeNodeFlags::SPAN_AVAIL_WIDTH)
                    .push();
                ui.same_line();
                ui.text_disabled(&class.name);
                if *root {
                    ui.same_line();
                    ui.text_colored(OVERRIDE_COLOUR, "root");
                }
                if *inherited {
                    ui.same_line();
                    ui.text_disabled("(class default)");
                }
                if let Some(_node) = open {
                    edit_class_button(ui, e, class);
                    let _ = field(ui, "Identity");
                    ui.text_disabled(id.to_string());
                    if let Some(action) = reflected_property_table(
                        ui,
                        e,
                        "component",
                        class
                            .class_id
                            .as_deref()
                            .unwrap_or(class.name.as_str()),
                        properties,
                        overrides,
                    ) {
                        edit = Some((*id, action));
                    }
                    // The root defines the actor's spatial contract and a class
                    // default belongs to the class, not to this placement.
                    let blocked = *root || *inherited;
                    let _disabled = ui.begin_disabled(blocked);
                    if ui.small_button("Remove") {
                        remove = Some(*id);
                    }
                    drop(_disabled);
                    if blocked
                        && ui.is_item_hovered_with_flags(
                            imgui::ItemHoveredFlags::ALLOW_WHEN_DISABLED,
                        )
                    {
                        ui.tooltip_text(if *root {
                            "the root component cannot be removed"
                        } else {
                            "inherited from the class; disable instead"
                        });
                    }
                }
            }
            ui.spacing();
            if ui.button("Add Component") {
                ui.open_popup("add-component");
            }
            #[cfg(test)]
            record_script_control(ui, "Add Component");
            add_component_menu(ui, e, id);
            if let Some((component, action)) = edit
                && let Some(position) = e.scene.actors[index]
                    .components
                    .iter()
                    .position(|c| c.id == component)
            {
                let target = &mut e.scene.actors[index].components[position];
                let (mut properties, mut overrides) =
                    (target.properties.clone(), target.overrides.clone());
                apply_property_action(action, &mut properties, &mut overrides);
                target.properties = properties;
                target.overrides = overrides;
                e.changed();
            }
            if let Some(component) = remove {
                e.remove_actor_component(id, component);
            }
        }
        if heading(ui, "\u{eb2b} Properties") {
            let (properties, overrides) = (
                e.scene.actors[index].properties.clone(),
                e.scene.actors[index].overrides.clone(),
            );
            let class = e.scene.actors[index].class.clone();
            if let Some(action) = reflected_property_table(
                ui,
                e,
                "actor",
                class.class_id.as_deref().unwrap_or(class.name.as_str()),
                &properties,
                &overrides,
            ) {
                let (mut properties, mut overrides) = (properties, overrides);
                apply_property_action(action, &mut properties, &mut overrides);
                e.scene.actors[index].properties = properties;
                e.scene.actors[index].overrides = overrides;
                e.changed();
            }
            muted(
                ui,
                "An edited value is an override even when it equals the class default.",
            );
        }
    });
    if e.playing {
        ui.spacing();
        ui.text_disabled("Stop Play to edit this actor.");
    }
}
/// Map Settings: the document's own properties, opened from the Hierarchy's map
/// root line. Scene-wide values that used to live in Project Settings move here.
pub(crate) fn map_settings(ui: &imgui::Ui, e: &mut Editor) {
    if !e.map_settings {
        return;
    }
    let mut open = true;
    ui.window("\u{eb29} Map Settings###MapSettings")
        .opened(&mut open)
        .size([420., 380.], Condition::FirstUseEver)
        .build(|| {
            let _ = field(ui, "Name");
            ui.text(&e.scene.name);
            let _ = field(ui, "Document Version");
            ui.text(e.scene.version.to_string());
            let _ = field(ui, "Actors");
            ui.text(e.scene.actors.len().to_string());
            ui.separator();
            scene_blueprint_settings(ui, e);
            ui.separator();
            if heading(ui, "Current Scene HUD Budget") {
                let mut values = [
                    e.scene.hud_budget.layouts as i32,
                    e.scene.hud_budget.rectangles as i32,
                    e.scene.hud_budget.texts as i32,
                    e.scene.hud_budget.glyphs as i32,
                ];
                let _disabled = ui.begin_disabled(e.playing);
                if crate::gui::Drag::new("Layouts / Rects / Texts / Glyphs")
                    .speed(1.)
                    .build_array(ui, &mut values)
                {
                    let before = e.scene.hud_budget.clone();
                    e.scene.hud_budget = crate::hud::Budget {
                        layouts: values[0].max(1) as usize,
                        rectangles: values[1].max(1) as usize,
                        texts: values[2].max(1) as usize,
                        glyphs: values[3].max(1) as usize,
                    };
                    if let Err(error) = e.scene.validate() {
                        e.scene.hud_budget = before;
                        e.last_error = Some(error);
                    } else {
                        e.changed();
                    }
                }
                drop(_disabled);
                muted(ui, "Budgets apply to this map only and are saved with it.");
            }
        });
    e.map_settings = open;
}
fn scene_blueprint_settings(ui: &imgui::Ui, e: &mut Editor) {
    if !heading(ui, "Scene Blueprint") {
        return;
    }
    let model = e.object_model();
    // Only `epok::SceneScriptActor` subclasses can be a map's script: the Level
    // loader creates exactly one of them and nothing else belongs in that slot.
    let parents = model
        .as_deref()
        .map(|m| {
            let mut names = m
                .scene_script_parents()
                .map(|c| c.cpp_name.clone())
                .collect::<Vec<_>>();
            names.sort();
            names
        })
        .unwrap_or_default();
    if parents.is_empty() {
        ui.text_disabled("No SceneScriptActor class is available in this project.");
        return;
    }
    if e.map_scene_script_parent.is_empty() {
        e.map_scene_script_parent = e
            .project_default_scene_script_parent()
            .filter(|name| parents.contains(name))
            .unwrap_or_else(|| parents[0].clone());
    }
    let existing = e
        .scene
        .scene_script
        .as_ref()
        .map(|script| (script.parent.name.clone(), script.blueprint.name.clone()));
    match existing {
        Some((parent, blueprint)) => {
            let _ = field(ui, "Parent");
            ui.set_next_item_width(-1.);
            let mut chosen = None;
            let _disabled = ui.begin_disabled(e.playing);
            if let Some(_combo) = ui.begin_combo("##scene-script-parent", &parent) {
                for cpp_name in &parents {
                    if ui
                        .selectable_config(cpp_name)
                        .selected(*cpp_name == parent)
                        .build()
                    {
                        chosen = Some(cpp_name.clone());
                    }
                }
            }
            drop(_disabled);
            if let Some(chosen) = chosen.filter(|chosen| *chosen != parent) {
                e.set_scene_script_parent(&chosen);
            }
            let _ = field(ui, "Blueprint");
            ui.text(&blueprint);
            muted(
                ui,
                "Changing the parent validates the whole project first. Undo with Edit > Undo Scene Edit.",
            );
            let _disabled = ui.begin_disabled(e.playing);
            if ui.button("Open Scene Blueprint")
                && let Err(error) = e.open_scene_blueprint()
            {
                e.last_error = Some(error);
            }
            drop(_disabled);
            muted(
                ui,
                "The scene Blueprint belongs to this map: its edits are map edits and Save writes the map.",
            );
        }
        None => {
            let _ = field(ui, "Parent");
            ui.set_next_item_width(-1.);
            let current = e.map_scene_script_parent.clone();
            if let Some(_combo) = ui.begin_combo("##scene-script-parent", &current) {
                for cpp_name in &parents {
                    if ui
                        .selectable_config(cpp_name)
                        .selected(*cpp_name == current)
                        .build()
                    {
                        e.map_scene_script_parent = cpp_name.clone();
                    }
                }
            }
            let _disabled = ui.begin_disabled(e.playing);
            if ui.button("Create Scene Blueprint") {
                e.create_scene_script();
            }
            drop(_disabled);
            muted(
                ui,
                "One SceneScriptActor per map, created by the loader and never placed by hand.",
            );
        }
    }
}
pub(crate) fn heading(ui: &imgui::Ui, text: &str) -> bool {
    let _color = ui.push_style_color(C::Header, gray(43));
    ui.collapsing_header(text, imgui::TreeNodeFlags::DEFAULT_OPEN)
}
fn vector(ui: &imgui::Ui, label: &str, values: &mut [f32; 3]) {
    let _id = ui.push_id(label);
    let left = ui.cursor_pos()[0];
    let width = ui.content_region_avail()[0];
    ui.align_text_to_frame_padding();
    ui.text(label);
    let value_width = if width >= 320. {
        ui.same_line_with_pos(left + 86.);
        width - 86.
    } else {
        width
    };
    let field = ((value_width - 2. * 7. - 3. * (ui.calc_text_size("X")[0] + 2.)) / 3.).max(1.);
    for i in 0..3 {
        if i > 0 {
            ui.same_line();
        }
        ui.text_colored(
            [
                [0.87, 0.40, 0.36, 1.],
                [0.55, 0.78, 0.35, 1.],
                [0.35, 0.62, 0.90, 1.],
            ][i],
            ["X", "Y", "Z"][i],
        );
        ui.same_line_with_spacing(0., 2.);
        ui.set_next_item_width(field);
        crate::gui::Drag::new(format!("##{i}"))
            .speed(if label == "Rotation" { 0.5 } else { 0.02 })
            .display_format("%.2f")
            .build(ui, &mut values[i]);
    }
}
pub(crate) fn inspector(ui: &imgui::Ui, e: &mut Editor) {
    crate::lighting_editor::window(ui, e);
    ui.window("\u{ea74} Inspector###Inspector").build(|| {
        if e.selected_asset.is_some() {
            crate::asset_inspector::draw(ui, e);
            return;
        }
        crate::project_browser::inspector_drop(ui, e);
        if e.selected_actor.is_some()
            && e.scene_view_mode == crate::scene_view_mode::SceneViewMode::TwoD
        {
            actor_inspector(ui, e);
            return;
        }
        let Some(index) = e.selected else {
            ui.text_disabled("Nothing selected");
            return;
        };
        crate::blueprint_workflow::instance_inspector(ui, e, index);
        let mut entity = e.scene.actors[index].clone();
        let original = entity.clone();
        let mut requested_parent = None;
        ui.disabled(e.playing, || {
            let glyph = if entity.kind == "Camera" {
                CAMERA
            } else {
                CUBE
            };
            actor_header(ui, &mut entity, glyph);
            ui.separator();
            entity_actor_inspector(ui, e, index);
            crate::hud_editor::inspector(ui, &mut entity);
            if entity.rect.is_none()
                && entity.canvas.is_none()
                && heading(ui, &format!("{MOVE} Transform"))
            {
                let _ = field(ui, "Parent");
                ui.set_next_item_width(-1.);
                let parent_name = entity
                    .parent
                    .map_or("None (Scene root)", |p| e.scene.actors[p].name.as_str());
                if let Some(_combo) = ui.begin_combo("##parent", parent_name) {
                    requested_parent = parent_menu(ui, e, index, true);
                }
                if entity.parent.is_some() {
                    ui.text_disabled("Position / Rotation / Scale are local");
                }
                vector(ui, "Position", &mut entity.position);
                vector(ui, "Rotation", &mut entity.rotation);
                vector(ui, "Scale", &mut entity.scale);
                if let Some(_popup) = ui.begin_popup_context_window()
                    && ui.menu_item("Reset Transform")
                {
                    entity.position = [0.; 3];
                    entity.rotation = [0.; 3];
                    entity.scale = [1.; 3];
                }
            }
            ui.separator();
            crate::lighting_editor::inspector(ui, &mut entity);
            crate::shadows::inspector(ui, &mut entity);
            crate::sprites_editor::inspector(ui, e, &mut entity);
            crate::collision_editor::inspector(ui, &mut entity);
            crate::palette::inspector(ui, e, &mut entity);
            if entity.kind == "Mesh" {
                crate::mesh_editor::filter(ui, e, &mut entity);
            }
            if entity.kind == "Mesh"
                && entity.editable_mesh.is_none()
                && entity.skeletal_mesh.is_none()
            {
                if heading(ui, "\u{eb5c} Mesh Renderer") {
                    let _ = field(ui, "Material");
                    ui.text("PSX Material (instance)");
                    ui.align_text_to_frame_padding();
                    let _ = field(ui, "Color");
                    ui.set_next_item_width(-1.);
                    ui.color_edit3("##material-color", &mut entity.material.color);
                    crate::texture::picker(ui, &e.assets.index, &mut entity.material);
                    crate::lighting_editor::mesh(ui, &mut entity);
                    if ui.small_button("Reset Material") {
                        entity.material = Default::default();
                    }
                    inline(ui, "Remove Mesh Renderer");
                    if ui.small_button("Remove Mesh Renderer") {
                        entity.kind = "Empty".into();
                    }
                }
                ui.separator();
            } else if entity.kind == "Camera" && heading(ui, &format!("{CAMERA} Camera")) {
                let _ = field(ui, "Projection");
                ui.text_disabled("Perspective");
                crate::gui::Drag::new(field(ui, "Horizontal FOV"))
                    .range(25., 120.)
                    .speed(0.25)
                    .build(ui, &mut entity.camera_fov);
                let _ = field(ui, "Target");
                ui.text_disabled(format!(
                    "{} x {} / NTSC",
                    e.scene.display_size[0], e.scene.display_size[1]
                ));
                ui.separator();
            }
            crate::skeletal_ui::component(ui, e, &mut entity);
            crate::timeline_scene::inspector(ui, e, &mut entity);
            crate::particle_effect_scene::inspector(ui, e, &mut entity);
            crate::asset_ui::component(ui, e, &mut entity);
            let extra_audio = entity
                .components
                .iter()
                .filter(|c| {
                    c.class.class_id.as_deref() == Some(crate::object_model::AUDIO_COMPONENT_ID)
                })
                .skip(1)
                .cloned()
                .collect::<Vec<_>>();
            for component in extra_audio {
                let _id = ui.push_id(component.id.to_string());
                ui.text(&component.name);
                let mut preview = entity.clone();
                preview.audio = Some(
                    component
                        .properties
                        .get("audio")
                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                        .unwrap_or_default(),
                );
                let before = preview.audio.clone();
                crate::asset_ui::component(ui, e, &mut preview);
                if before != preview.audio {
                    if let Some(audio) = &preview.audio {
                        if let Some(target) =
                            entity.components.iter_mut().find(|c| c.id == component.id)
                        {
                            target
                                .properties
                                .insert("audio".into(), serde_json::json!(audio));
                            target.overrides.insert("audio".into());
                        }
                    } else if !component.inherited {
                        entity.components.retain(|c| c.id != component.id);
                    }
                }
            }
            crate::mesh_editor::component(ui, e, &mut entity);
            ui.dummy([0., 7.]);
            let width = ui.content_region_avail()[0];
            ui.set_cursor_pos([
                ui.cursor_pos()[0] + (width - 210.).max(0.) * 0.5,
                ui.cursor_pos()[1],
            ]);
            if ui.button_with_size("Add Component", [210_f32.min(width), 23.]) {
                ui.open_popup("add-component");
            }
            #[cfg(test)]
            record_script_control(ui, "Add Component");
            if add_component_menu(ui, e, entity.id) {
                entity = e.scene.actors[index].clone();
            }
        });
        if entity != original {
            let previous = e.scene.actors[index].clone();
            e.scene.actors[index] = entity;
            if e.scene.validate().is_ok() {
                e.changed();
            } else {
                e.scene.actors[index] = previous;
            }
        }
        if let Some(parent) = requested_parent {
            e.reparent(index, parent, true);
        }
        if e.playing {
            ui.spacing();
            ui.text_disabled("Stop Play to edit this object.");
        }
    });
}
fn project(ui: &imgui::Ui, e: &mut Editor, asset_font: imgui::FontId) {
    crate::project_browser::window(ui, e, asset_font);
}

/// One World2D actor as the 2D view draws it.
struct Actor2DView {
    id: uuid::Uuid,
    name: String,
    transform: crate::scene_view_mode::Transform2D,
    /// Identity of the root `SceneComponent2D` whose properties carry the
    /// transform, so a drag writes back to the component it read.
    root: Option<uuid::Uuid>,
}
/// The map's World2D actors, sorted back to front by `draw_order` with document
/// order breaking ties, exactly like `draw_key_2d` in `runtime/world2d.hpp`.
fn world2d_actors(e: &Editor) -> Vec<Actor2DView> {
    let Some(model) = e.object_model() else {
        return Vec::new();
    };
    let mut actors: Vec<_> = e
        .scene
        .actors
        .iter()
        .filter(|actor| {
            actor
                .class
                .resolve(&model)
                .is_some_and(|c| c.domain == crate::reflection_schema::Domain::World2D)
        })
        .map(|actor| {
            let root = actor.root();
            Actor2DView {
                id: actor.id,
                name: actor.name.clone(),
                transform: root
                    .map(|c| crate::scene_view_mode::read_transform_2d(&c.properties))
                    .unwrap_or_default(),
                root: root.map(|c| c.id),
            }
        })
        .collect();
    actors.sort_by_key(|a| a.transform.draw_order);
    actors
}
/// Whether a world point is inside one actor's unit quad, after its own rotation
/// and scale. The quad is the one unit square centred on the actor's origin: no
/// authored geometry exists yet, so a placement is drawn at its footprint.
fn inside_actor_2d(point: [f32; 2], transform: &crate::scene_view_mode::Transform2D) -> bool {
    let angle = -transform.rotation.to_radians();
    let (sin, cos) = angle.sin_cos();
    let offset = [
        point[0] - transform.position[0],
        point[1] - transform.position[1],
    ];
    let local = [
        (offset[0] * cos - offset[1] * sin) / transform.scale[0].abs().max(1e-4),
        (offset[0] * sin + offset[1] * cos) / transform.scale[1].abs().max(1e-4),
    ];
    local[0].abs() <= 0.5 && local[1].abs() <= 0.5
}
/// The Scene window in `SceneViewMode::TwoD`: an orthographic World2D view drawn
/// entirely with ImGui draw-list primitives, so it costs no render target.
fn world2d_view(ui: &imgui::Ui, e: &mut Editor) {
    ui.text("2D World");
    inline(ui, "Reset View");
    if ui.button("Reset View") {
        e.view_2d.reset();
    }
    ui.same_line();
    ui.text_disabled(format!(
        "{:.0}% | {} px per unit",
        e.view_2d.zoom * 100.,
        e.view_2d.scale().round()
    ));
    muted(
        ui,
        "World Y is up, 32 px per unit at 100%. MMB/RMB drag: pan | wheel: zoom | LMB: select and move",
    );
    ui.separator();
    let origin = ui.cursor_screen_pos();
    let size = ui.content_region_avail().map(|v| v.max(1.));
    ui.invisible_button("scene-2d-canvas", size);
    let hovered = ui.is_item_hovered();
    let mouse = ui.io().mouse_pos;
    let actors = world2d_actors(e);

    // --- navigation -------------------------------------------------------
    if hovered && ui.io().mouse_wheel != 0. {
        e.view_2d.zoom_at(ui.io().mouse_wheel, mouse, origin, size);
    }
    if (ui.is_mouse_dragging(imgui::MouseButton::Middle)
        || ui.is_mouse_dragging(imgui::MouseButton::Right))
        && (hovered || ui.is_item_active())
    {
        e.view_2d.pan_pixels(ui.io().mouse_delta);
    }

    // --- selection and drag ----------------------------------------------
    let world = e.view_2d.screen_to_world(mouse, origin, size);
    if hovered && ui.is_mouse_clicked(imgui::MouseButton::Left) && !e.playing {
        // Topmost first: the list is back to front, so the hit test walks it back.
        let hit = actors
            .iter()
            .rev()
            .find(|actor| inside_actor_2d(world, &actor.transform));
        match hit {
            Some(actor) => {
                e.select_actor(Some(actor.id));
                e.drag_2d = Some((
                    actor.id,
                    [
                        actor.transform.position[0] - world[0],
                        actor.transform.position[1] - world[1],
                    ],
                ));
            }
            None => {
                e.select_actor(None);
                e.drag_2d = None;
            }
        }
    }
    if !ui.is_mouse_down(imgui::MouseButton::Left) {
        if e.drag_2d.is_some() {
            e.drag_2d = None;
        }
        // One drag of one actor is one undo step, however many frames it took.
        e.end_coalesced();
    } else if let Some((id, grab)) = e.drag_2d
        && ui.is_mouse_dragging(imgui::MouseButton::Left)
        && !e.playing
        && let Some(index) = e.scene.actor_index(id)
        && let Some(root) = actors
            .iter()
            .find(|actor| actor.id == id)
            .and_then(|actor| actor.root)
        && let Some(component) = e.scene.actors[index]
            .components
            .iter_mut()
            .find(|c| c.id == root)
    {
        let (mut properties, mut overrides) =
            (component.properties.clone(), component.overrides.clone());
        crate::scene_view_mode::write_position_2d(
            &mut properties,
            &mut overrides,
            [
                (world[0] + grab[0]).clamp(-8192., 8192.),
                (world[1] + grab[1]).clamp(-8192., 8192.),
            ],
        );
        component.properties = properties;
        component.overrides = overrides;
        e.scene.actors[index].refresh_components();
        e.changed_coalesced("actor-2d-drag");
    }

    // --- rendering --------------------------------------------------------
    let draw = ui.get_window_draw_list();
    let corner = [origin[0] + size[0], origin[1] + size[1]];
    draw.with_clip_rect_intersect(origin, corner, || {
        draw.add_rect(origin, corner, [0.16, 0.17, 0.19, 1.])
            .filled(true)
            .build();
        // Grid spacing grows with the zoom so lines never crowd into a wash.
        let step = [1., 5., 25., 125., 625.]
            .into_iter()
            .find(|unit| unit * e.view_2d.scale() >= 12.)
            .unwrap_or(625.);
        for (spacing, colour) in [
            (step, [0.22, 0.23, 0.26, 1.]),
            (step * 5., [0.30, 0.31, 0.35, 1.]),
        ] {
            let first = ((e.view_2d.screen_to_world(origin, origin, size)[0]) / spacing).floor();
            let mut unit = first;
            loop {
                let x = e
                    .view_2d
                    .world_to_screen([unit * spacing, 0.], origin, size)[0];
                if x > corner[0] {
                    break;
                }
                draw.add_line([x, origin[1]], [x, corner[1]], colour)
                    .build();
                unit += 1.;
            }
            let first = ((e.view_2d.screen_to_world(corner, origin, size)[1]) / spacing).floor();
            let mut unit = first;
            loop {
                let y = e
                    .view_2d
                    .world_to_screen([0., unit * spacing], origin, size)[1];
                if y < origin[1] {
                    break;
                }
                draw.add_line([origin[0], y], [corner[0], y], colour)
                    .build();
                unit += 1.;
            }
        }
        // The world axes: +X red, +Y green, matching the 3D viewport's gizmo.
        let centre = e.view_2d.world_to_screen([0., 0.], origin, size);
        draw.add_line(
            [origin[0], centre[1]],
            [corner[0], centre[1]],
            [0.75, 0.32, 0.32, 1.],
        )
        .build();
        draw.add_line(
            [centre[0], origin[1]],
            [centre[0], corner[1]],
            [0.40, 0.70, 0.36, 1.],
        )
        .build();
        for actor in &actors {
            let selected = e.selected_actor == Some(actor.id);
            let transform = &actor.transform;
            let angle = transform.rotation.to_radians();
            let (sin, cos) = angle.sin_cos();
            let corners: Vec<[f32; 2]> = [[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]]
                .into_iter()
                .map(|local| {
                    let scaled = [local[0] * transform.scale[0], local[1] * transform.scale[1]];
                    e.view_2d.world_to_screen(
                        [
                            transform.position[0] + scaled[0] * cos - scaled[1] * sin,
                            transform.position[1] + scaled[0] * sin + scaled[1] * cos,
                        ],
                        origin,
                        size,
                    )
                })
                .collect();
            draw.add_polyline(
                corners.clone(),
                if selected {
                    [0.98, 0.65, 0.24, 0.30]
                } else {
                    [0.38, 0.60, 0.85, 0.25]
                },
            )
            .filled(true)
            .build();
            draw.add_polyline(
                corners.clone(),
                if selected {
                    [1., 0.65, 0.25, 1.]
                } else {
                    [0.55, 0.72, 0.92, 1.]
                },
            )
            .thickness(if selected { 2. } else { 1. })
            .build();
            let label = e.view_2d.world_to_screen(transform.position, origin, size);
            draw.add_text(
                [label[0] + 6., label[1] + 4.],
                [0.92, 0.93, 0.95, 1.],
                &actor.name,
            );
            if actor.root.is_none() {
                draw.add_text(
                    [label[0] + 6., label[1] + 20.],
                    [1., 0.72, 0.3, 1.],
                    "no root component",
                );
            }
        }
    });
    if actors.is_empty() {
        ui.set_cursor_screen_pos([origin[0] + 12., origin[1] + 12.]);
        // A project whose reflection data never resolved, and one whose model
        // simply declares no 2D actor class, are different problems.
        let placeable = e.object_model().is_some_and(|model| {
            model
                .placeable()
                .any(|c| c.domain == crate::reflection_schema::Domain::World2D)
        });
        if placeable {
            muted(
                ui,
                "No World2D actors in this map. Create one from the Hierarchy's Actor > 2D menu.",
            );
        } else {
            muted(
                ui,
                "This project declares no placeable World2D actor class. Compile the project so its reflection data resolves, or derive an Actor2D subclass.",
            );
        }
    }
    scene_shortcuts(ui, e);
}
fn scene_view(
    ui: &imgui::Ui,
    e: &mut Editor,
    texture: imgui::TextureId,
    hud_texture: imgui::TextureId,
    image_size: [f32; 2],
) {
    if e.focus_scene {
        unsafe {
            imgui::sys::igSetNextWindowCollapsed(false, imgui::sys::ImGuiCond_Always as i32);
        }
    }
    ui.window("\u{eb29} Scene###Scene").build(|| {
        for mode in crate::scene_view_mode::SceneViewMode::ALL {
            if mode != crate::scene_view_mode::SceneViewMode::ALL[0] {
                ui.same_line();
            }
            if ui.radio_button_bool(mode.label(), e.scene_view_mode == mode) {
                e.set_scene_view_mode(mode);
            }
        }
        inline(ui, "Shaded");
        match e.scene_view_mode {
            crate::scene_view_mode::SceneViewMode::UI => {
                ui.text("Canvas / HUD");
                ui.separator();
                crate::hud_editor::view(ui, e, hud_texture);
                return;
            }
            crate::scene_view_mode::SceneViewMode::TwoD => {
                // The 3D simulation and preview state are untouched: switching
                // back to 3D resumes exactly where the author left it.
                world2d_view(ui, e);
                return;
            }
            crate::scene_view_mode::SceneViewMode::ThreeD => {}
        }
        if ui.button("Shaded  \u{eab4}") {
            ui.open_popup("shading");
        }
        ui.popup("shading", || {
            if ui.menu_item_config("Shaded").selected(!e.wire).build() {
                e.wire = false;
                e.view_dirty = true;
            }
            if ui.menu_item_config("Wireframe").selected(e.wire).build() {
                e.wire = true;
                e.view_dirty = true;
            }
        });
        inline(ui, "Grid");
        if ui.checkbox("Grid", &mut e.grid) {
            e.view_dirty = true;
        }
        inline(ui, "Reset View");
        if ui.button("Reset View") {
            e.action("reset-view");
        }
        inline_width(ui, 125.);
        ui.set_next_item_width(70.);
        crate::gui::Drag::new("Speed")
            .speed(0.1)
            .range(0.1, 100.)
            .build(ui, &mut e.view.fly_speed);
        muted(ui, "RMB + WASD: fly | Q/E: down/up | Alt + LMB: orbit");
        crate::lighting_editor::preview_status(ui, e);
        ui.separator();
        let position = ui.cursor_screen_pos();
        let available = ui.content_region_avail().map(|v| v.max(1.));
        // Fill the Scene panel, preserving projection aspect by cropping the preview texture.
        let factor = (available[0] / image_size[0]).max(available[1] / image_size[1]);
        let uv = [
            available[0] / (image_size[0] * factor),
            available[1] / (image_size[1] * factor),
        ];
        imgui::Image::new(texture, available)
            .uv0([(1. - uv[0]) * 0.5, (1. - uv[1]) * 0.5])
            .uv1([(1. + uv[0]) * 0.5, (1. + uv[1]) * 0.5])
            .build(ui);
        crate::project_browser::scene_drop(ui, e);
        let hovered = ui.is_item_hovered();
        // A text field may own ActiveId until navigation explicitly relinquishes it.
        let navigation_hovered = ui
            .is_item_hovered_with_flags(imgui::ItemHoveredFlags::ALLOW_WHEN_BLOCKED_BY_ACTIVE_ITEM);
        if hovered && ui.io().mouse_wheel != 0. {
            if ui.is_mouse_down(imgui::MouseButton::Right) {
                e.view.fly_speed =
                    (e.view.fly_speed * 1.2_f32.powf(ui.io().mouse_wheel)).clamp(0.1, 100.);
            } else {
                e.view.dolly(ui.io().mouse_wheel);
            }
            e.view_dirty = true;
        }
        let right = ui.is_mouse_down(imgui::MouseButton::Right);
        let middle = ui.is_mouse_down(imgui::MouseButton::Middle);
        if navigation_hovered
            && (ui.is_mouse_clicked(imgui::MouseButton::Right)
                || ui.is_mouse_clicked(imgui::MouseButton::Middle))
        {
            e.scene_navigation = true;
            // Dear ImGui does not focus windows on middle-click by default.
            unsafe {
                imgui::sys::igClearActiveID();
                imgui::sys::igSetWindowFocus_Str(c"\u{eb29} Scene###Scene".as_ptr());
            }
        }
        if (!right && !middle) || !ui.is_window_focused() || ui.is_key_pressed(imgui::Key::Escape) {
            e.scene_navigation = false;
        }
        e.scene_look = e.scene_navigation && right && !ui.io().key_alt;
        if e.scene_navigation && middle {
            e.view.pan(ui.io().mouse_delta, factor);
            e.view_dirty = true;
        }
        if e.scene_look {
            if !ui.is_mouse_clicked(imgui::MouseButton::Right) {
                e.view
                    .look(e.raw_look.take().unwrap_or(ui.io().mouse_delta), false);
            }
            let key = |k| if ui.is_key_down(k) { 1. } else { 0. };
            let axes = [
                key(imgui::Key::D) - key(imgui::Key::A),
                key(imgui::Key::E) - key(imgui::Key::Q),
                key(imgui::Key::W) - key(imgui::Key::S),
            ];
            e.view.fly(axes, ui.io().delta_time, ui.io().key_shift);
            e.view_dirty = true;
        }
        if hovered && ui.is_mouse_dragging(imgui::MouseButton::Left) && ui.io().key_alt {
            e.view.look(ui.io().mouse_delta, true);
            e.view_dirty = true;
        }
        if !e.mesh_editor.open {
            crate::gizmo::draw(ui, e, position, available, factor, uv);
        }
        let mouse = ui.io().mouse_pos;
        let local = [mouse[0] - position[0], mouse[1] - position[1]];
        // Overlay controls own their clicks, even though they overlap the image.
        let over_tools = (7. ..=43.).contains(&local[0]) && (9. ..=121.).contains(&local[1]);
        let over_axes = local[0] >= available[0] - 95. && local[1] <= 85.;
        let eligible = hovered
            && !over_tools
            && !over_axes
            && !e.playing
            && !ui.io().key_alt
            && !ui.is_mouse_down(imgui::MouseButton::Right)
            && !ui.is_mouse_down(imgui::MouseButton::Middle)
            && ui.io().mouse_wheel == 0.
            && e.drag_axis.is_none();
        if e.scene_click.update(
            mouse,
            ui.is_mouse_clicked(imgui::MouseButton::Left),
            ui.is_mouse_released(imgui::MouseButton::Left),
            ui.is_mouse_down(imgui::MouseButton::Left),
            eligible,
        ) {
            let pixel = crate::picking::texture_pixel(mouse, position, factor, uv);
            e.selected_asset = None;
            if !crate::mesh_editor::pick(e, pixel, ui.io().key_ctrl) {
                e.selected = crate::picking::pick(&e.scene, &e.view, pixel);
            }
            e.view_dirty = true;
            e.reveal_selected = e.selected.is_some();
            if e.selected.is_some() {
                e.search.clear();
            }
        }
        if ui.is_window_focused() && !ui.io().want_text_input && !right && !middle {
            crate::mesh_editor::geometry_shortcuts(ui, e);
            for (i, key) in [imgui::Key::Q, imgui::Key::W, imgui::Key::E, imgui::Key::R]
                .iter()
                .enumerate()
            {
                if !e.mesh_editor.open && ui.is_key_pressed(*key) {
                    e.tool = i;
                }
            }
            if ui.is_key_pressed(imgui::Key::F) {
                e.action("frame-selected");
            }
        }
        ui.set_cursor_screen_pos([position[0] + 7., position[1] + 9.]);
        let tool_padding = ui.push_style_var(imgui::StyleVar::WindowPadding([3., 3.]));
        let tool_spacing = ui.push_style_var(imgui::StyleVar::ItemSpacing([3., 3.]));
        ui.child_window("scene-tools")
            .size([36., 112.])
            .border(true)
            .build(|| {
                for (i, glyph, tip) in [
                    (0, "\u{f256}", "Select: click geometry; Alt + drag orbits"),
                    (1, MOVE, "Move (W): drag an axis"),
                    (2, "\u{eb37}", "Rotate (E): drag a rotation ring"),
                    (3, "\u{eb4c}", "Scale (R): drag an axis"),
                ] {
                    if icon(ui, glyph, &format!("tool{i}"), tip, e.tool == i) {
                        e.tool = i;
                    }
                }
            });
        drop(tool_spacing);
        drop(tool_padding);
        let orientation_position = [
            position[0] + available[0] - crate::gizmo::ORIENTATION_SIZE - 9.,
            position[1] + 8.,
        ];
        if crate::gizmo::draw_orientation(ui, &mut e.view, orientation_position) {
            e.view_dirty = true;
        }
    });
}
fn game_view(ui: &imgui::Ui, e: &mut Editor, texture: Option<imgui::TextureId>) {
    let mut visible = false;
    ui.window("\u{ec17} Game###Game").build(|| {
        visible = true;
        if e.active_play_target == crate::play::Target::Serial && e.job.is_some() {
            ui.text_wrapped("PSX serial session. The picture is on the console's display; use its controller. Upload progress and TTY messages appear in Console.");
            ui.text_wrapped("Stop disconnects NOTPSXSerial. It does not reset or halt the console.");
            ui.disabled(!e.playing || e.serial_ui.command_pending, || {
                if ui.button("Pause PSX") { e.action("serial-pause"); }
                ui.same_line();
                if ui.button("Continue PSX") { e.action("serial-resume"); }
                ui.same_line();
                if ui.button("Reset PSX") { e.action("serial-reset"); }
            });
            if e.serial_ui.command_pending { ui.text_disabled("Waiting for Unirom..."); }
            ui.text_wrapped("Reset reboots the console; it does not automatically reload the game. Return to the Unirom loader before the next Play.");
            return;
        }
        ui.text("PSX native / Point filter");
        inline_width(ui, 140.);
        ui.set_next_item_width(140.);
        if let Some(_combo) = ui.begin_combo("##game-scale", e.preferences.game_scale.label()) {
            for mode in crate::settings::GameScale::ALL {
                if ui.selectable_config(mode.label()).selected(e.preferences.game_scale == mode).build() {
                    e.preferences.game_scale = mode;
                    if let Err(error) = e.preferences.save() { e.log(error); }
                }
            }
        }
        inline(ui, "Debugger");
        if ui.small_button("Debugger")
            && let Some(pid) = e.emulator_pid
        {
            e.emulator_visible = true;
            crate::native::emulator_window(pid, true);
        }
        ui.separator();
        if let Some(error) = &e.game_error {
            ui.text_colored([1., 0.55, 0.4, 1.], format!("Video disconnected: {error}"));
        }
        let region = ui.content_region_avail();
        if let (Some(frame), Some(texture)) = (&e.game_frame, texture) {
            let footer_height = ui.text_line_height() + 8.;
            let image_height = (region[1] - footer_height).max(1.);
            let size = e.preferences.game_scale.image_size([frame.width, frame.height], [region[0].max(1.), image_height]);
            let p = ui.cursor_pos();
            ui.set_cursor_pos([
                p[0] + (region[0] - size[0]) * 0.5,
                p[1] + (image_height - size[1]) * 0.5,
            ]);
            imgui::Image::new(texture, size).build(ui);
            if ui.is_item_clicked() {
                e.game_capture = true;
            }
            if !ui.is_window_focused() || ui.is_key_pressed(imgui::Key::Escape) {
                e.game_capture = false;
            }
            ui.set_cursor_pos([p[0], p[1] + image_height + 4.]);
            clipped(
                ui,
                &format!(
                    "{} x {}  |  {}",
                    frame.width,
                    frame.height,
                    if e.game_capture {
                        "Keyboard active - Esc releases"
                    } else {
                        "Click Game to control"
                    }
                ),
                region[0],
            );
            if ui.is_item_hovered() {
                ui.tooltip_text(format!(
                    "Pad {:04X} / VBlank {} / Cycles {}",
                    frame.buttons, frame.vsyncs, frame.cycles
                ));
            }
            let mut mask = 0;
            if e.game_capture && e.game_error.is_none() {
                for (key, bit) in [
                    (imgui::Key::Backspace, 0),
                    (imgui::Key::Enter, 3),
                    (imgui::Key::UpArrow, 4),
                    (imgui::Key::RightArrow, 5),
                    (imgui::Key::DownArrow, 6),
                    (imgui::Key::LeftArrow, 7),
                    (imgui::Key::W, 4),
                    (imgui::Key::D, 5),
                    (imgui::Key::S, 6),
                    (imgui::Key::A, 7),
                    (imgui::Key::Alpha1, 8),
                    (imgui::Key::Alpha3, 9),
                    (imgui::Key::Q, 10),
                    (imgui::Key::E, 11),
                    (imgui::Key::I, 12),
                    (imgui::Key::L, 13),
                    (imgui::Key::K, 14),
                    (imgui::Key::J, 15),
                ] {
                    if ui.is_key_down(key) {
                        mask |= 1 << bit;
                    }
                }
            }
            e.set_buttons(mask);
        } else {
            ui.dummy([0., region[1] * 0.35]);
            ui.text_wrapped(if e.playing {
                "Connecting to the PSX display..."
            } else if e.job.is_some() {
                "Compiling C++ scripts and scene..."
            } else {
                "Press Play to run the scene on PlayStation."
            });
        }
    });
    if !visible {
        e.game_capture = false;
        e.set_buttons(0);
    }
}
#[cfg(test)]
mod interaction_tests {
    use super::*;
    #[test]
    fn build_menu_runs_lighting_in_background_and_clears_the_stale_warning() {
        let mut context = crate::gui::tests::imgui_context();
        context.set_ini_filename(None);
        context.io_mut().display_size = [1000., 700.];
        context.io_mut().delta_time = 1. / 60.;
        context
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
        context.fonts().build_rgba32_texture();
        let root = crate::workspace::tests::temp("build-lighting-menu");
        let mut editor = Editor::new(root.clone());
        editor.auto_build = false;
        editor.scene = crate::lighting::tests::shadow_scene();
        editor.changed();
        assert!(!crate::lighting_editor::needs_rebuild(&editor));
        let saved = editor.scene.bake.clone();
        editor.scene.actors[1].lighting.static_geometry = false;
        editor.scene.actors[1].lighting.receive = crate::lighting::Receive::Realtime;
        editor.changed();
        assert!(crate::lighting_editor::needs_rebuild(&editor));
        assert!(editor.bake_job.is_none());

        fn frame(context: &mut imgui::Context, editor: &mut Editor) {
            SCRIPT_BUTTONS.with(|buttons| buttons.borrow_mut().clear());
            let ui = context.frame();
            ui.main_menu_bar(|| build_menu(ui, editor));
            ui.window("Scene lighting status")
                .position([10., 80.], Condition::Always)
                .size([700., 200.], Condition::Always)
                .build(|| {
                    crate::lighting_editor::preview_status(ui, editor);
                });
            context.render();
        }
        fn click(context: &mut imgui::Context, editor: &mut Editor, label: &str) {
            frame(context, editor);
            frame(context, editor);
            let point = SCRIPT_BUTTONS.with(|buttons| buttons.borrow()[label]);
            context.io_mut().add_mouse_pos_event(point);
            frame(context, editor);
            context
                .io_mut()
                .add_mouse_button_event(imgui::MouseButton::Left, true);
            frame(context, editor);
            context
                .io_mut()
                .add_mouse_button_event(imgui::MouseButton::Left, false);
            frame(context, editor);
        }
        fn finish_bake(editor: &mut Editor) {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
            while editor.bake_job.is_some() {
                assert!(
                    std::time::Instant::now() < deadline,
                    "Lighting worker timed out"
                );
                crate::lighting_editor::poll(editor);
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        }
        frame(&mut context, &mut editor);
        SCRIPT_BUTTONS
            .with(|buttons| assert!(buttons.borrow().contains_key("Lighting needs rebuilding")));
        click(&mut context, &mut editor, "Build menu");
        click(&mut context, &mut editor, "Build Lighting");
        assert!(editor.bake_job.is_some());
        assert_eq!(
            editor.scene.bake, saved,
            "The menu must only start the worker"
        );
        assert!(!crate::lighting_editor::can_start_bake(&editor));
        finish_bake(&mut editor);
        assert!(editor.bake_current && editor.view_dirty);
        assert!(!crate::lighting_editor::needs_rebuild(&editor));
        frame(&mut context, &mut editor);
        SCRIPT_BUTTONS
            .with(|buttons| assert!(!buttons.borrow().contains_key("Lighting needs rebuilding")));

        let rebuilt = editor.scene.bake.clone();
        crate::lighting_editor::start_bake(&mut editor);
        editor.scene.actors[2].position[0] += 1.;
        editor.changed();
        finish_bake(&mut editor);
        assert_eq!(
            editor.scene.bake, rebuilt,
            "Edits made while baking must discard outdated results"
        );
        assert!(crate::lighting_editor::needs_rebuild(&editor));
        assert!(
            editor
                .logs
                .iter()
                .any(|log| log.contains("Result discarded"))
        );
        drop(editor);
        if root.exists() {
            std::fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn inspector_header_class_editing_and_mesh_selection_follow_actor_ownership() {
        let mut context = crate::gui::tests::imgui_context();
        context.io_mut().display_size = [1000., 1500.];
        context.io_mut().delta_time = 1. / 60.;
        theme(context.style_mut());
        context
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
        context.fonts().build_rgba32_texture();
        context.load_ini_settings("[Window][Inspector]\nPos=0,0\nSize=380,1400\nCollapsed=0\n");
        let root = crate::workspace::tests::temp("inspector-ownership");
        std::fs::create_dir_all(root.join("assets/scripts")).unwrap();
        std::fs::create_dir_all(root.join("assets/Meshes")).unwrap();
        let source = root.join("assets/scripts/Actors.hpp");
        std::fs::write(&source, "// Project Actor declaration\n").unwrap();
        let mut editor = Editor::new(root.clone());
        editor.auto_build = false;
        editor.scene.actors.clear();
        editor.class_registry = crate::actor_document::tests::registry();
        let mut class = crate::actor_document::tests::class(
            "project-actor",
            "ProjectActor",
            Some(crate::object_model::ACTOR3D_ID),
        );
        class.source.file = source;
        editor
            .class_registry
            .classes
            .insert(class.id.clone(), class);
        editor.registry_revision += 1;
        editor.create_actor("epok::Actor3D");
        editor.add_actor_component(editor.selected_actor.unwrap(), "epok::Mesh3DComponent");
        let asset = crate::mesh::create(
            &root,
            "assets/Meshes/Ramp.epokasset",
            &crate::mesh::tests::shape("Ramp"),
        )
        .unwrap();
        let index = crate::assets::scan(&root, &mut Default::default());
        editor.assets.adopt(index, Default::default());
        fn frame(context: &mut imgui::Context, editor: &mut Editor) {
            SCRIPT_BUTTONS.with(|buttons| buttons.borrow_mut().clear());
            inspector(context.frame(), editor);
            context.render();
        }
        fn click(context: &mut imgui::Context, editor: &mut Editor, label: &str) {
            frame(context, editor);
            frame(context, editor);
            let point = SCRIPT_BUTTONS.with(|buttons| {
                *buttons
                    .borrow()
                    .get(label)
                    .unwrap_or_else(|| panic!("Missing control {label}"))
            });
            context.io_mut().add_mouse_pos_event(point);
            frame(context, editor);
            context
                .io_mut()
                .add_mouse_button_event(imgui::MouseButton::Left, true);
            frame(context, editor);
            context
                .io_mut()
                .add_mouse_button_event(imgui::MouseButton::Left, false);
            frame(context, editor);
            frame(context, editor);
        }
        frame(&mut context, &mut editor);
        frame(&mut context, &mut editor);
        SCRIPT_BUTTONS.with(|buttons| {
            let buttons = buttons.borrow();
            assert!(!buttons.contains_key("Edit C++ Class"));
            let [active, static_flag, name] =
                ["Active", "Static", "Actor name"].map(|key| buttons[key]);
            assert!(active[0] < static_flag[0] && static_flag[0] < name[0]);
            assert!((active[1] - name[1]).abs() < 2. && (static_flag[1] - name[1]).abs() < 2.);
        });
        let active = editor.scene.actors[0].active;
        click(&mut context, &mut editor, "Active");
        assert_eq!(editor.scene.actors[0].active, !active);
        let static_flag = editor.scene.actors[0].lighting.static_geometry;
        click(&mut context, &mut editor, "Static");
        assert_eq!(
            editor.scene.actors[0].lighting.static_geometry,
            !static_flag
        );
        click(&mut context, &mut editor, "Mesh");
        click(&mut context, &mut editor, "assets/Meshes/Ramp.epokasset");
        assert_eq!(
            editor.scene.actors[0].editable_mesh.as_ref().unwrap().asset,
            asset
        );
        click(&mut context, &mut editor, "Mesh");
        click(&mut context, &mut editor, "Engine / Cube");
        assert!(editor.scene.actors[0].editable_mesh.is_none());
        assert_eq!(editor.scene.actors[0].kind, "Mesh");
        editor.scene.actors[0].class =
            crate::actor_document::ClassReference::new("ProjectActor", "project-actor");
        frame(&mut context, &mut editor);
        frame(&mut context, &mut editor);
        SCRIPT_BUTTONS.with(|buttons| assert!(buttons.borrow().contains_key("Edit C++ Class")));
        drop(editor);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn add_component_cpp_dialog_hides_actor_and_wrong_domain_parents() {
        let mut context = crate::gui::tests::imgui_context();
        context.io_mut().display_size = [1600., 2000.];
        context.io_mut().delta_time = 1. / 60.;
        context
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
        context.fonts().build_rgba32_texture();
        context.load_ini_settings("[Window][Inspector]\nPos=0,0\nSize=1000,1800\nCollapsed=0\n");
        let root = crate::workspace::tests::temp("component-parent-dialog");
        let mut editor = Editor::new(root.clone());
        editor.auto_build = false;
        editor.scene.actors.clear();
        editor.class_registry = crate::actor_document::tests::registry();
        editor.registry_revision += 1;
        editor.create_actor("epok::Actor3D");
        fn frame(context: &mut imgui::Context, editor: &mut Editor) {
            SCRIPT_BUTTONS.with(|buttons| buttons.borrow_mut().clear());
            let ui = context.frame();
            inspector(ui, editor);
            script_creation_dialog(ui, editor);
            crate::blueprint_workflow::draw(ui, editor);
            context.render();
        }
        fn click(context: &mut imgui::Context, editor: &mut Editor, label: &str) {
            frame(context, editor);
            frame(context, editor);
            let point = SCRIPT_BUTTONS.with(|buttons| buttons.borrow()[label]);
            context.io_mut().add_mouse_pos_event(point);
            frame(context, editor);
            context
                .io_mut()
                .add_mouse_button_event(imgui::MouseButton::Left, true);
            frame(context, editor);
            context
                .io_mut()
                .add_mouse_button_event(imgui::MouseButton::Left, false);
            frame(context, editor);
            frame(context, editor);
        }
        click(&mut context, &mut editor, "Add Component");
        SCRIPT_BUTTONS.with(|buttons| {
            let buttons = buttons.borrow();
            assert!(buttons.contains_key("epok::AudioComponent"));
            assert!(buttons.contains_key("Create Blueprint ActorComponent..."));
            assert!(!buttons.contains_key("epok::SceneComponent3D"));
            assert!(!buttons.contains_key("epok::Actor3D"));
        });
        click(&mut context, &mut editor, "Create C++ ActorComponent...");
        let assert_component_parents = || {
            SCRIPT_BUTTONS.with(|buttons| {
                let buttons = buttons.borrow();
                assert!(buttons.contains_key("epok::ActorComponent"));
                assert!(buttons.contains_key("epok::AudioComponent"));
                for hidden in [
                    "epok::Actor",
                    "epok::Actor3D",
                    "epok::Actor2D",
                    "epok::UIActor",
                    "epok::SceneComponent2D",
                    "epok::UIComponent",
                    "epok::SceneScriptActor",
                ] {
                    assert!(!buttons.contains_key(hidden), "unexpected parent {hidden}");
                }
            })
        };
        assert_component_parents();
        click(&mut context, &mut editor, "Cancel");
        editor.action("new-script");
        frame(&mut context, &mut editor);
        frame(&mut context, &mut editor);
        SCRIPT_BUTTONS.with(|buttons| assert!(buttons.borrow().contains_key("epok::Actor3D")));
        click(&mut context, &mut editor, "Cancel");
        // Render the Blueprint selector against the same reflection fixture.
        editor.blueprint_creation = crate::blueprint_workflow::Creation {
            requested: true,
            context: crate::actor_scripts::CreationContext::Component(
                editor.selected_actor.unwrap(),
            ),
            parent: crate::object_model::ACTOR_COMPONENT_ID.into(),
            owner_domain: 1,
            ..Default::default()
        };
        frame(&mut context, &mut editor);
        frame(&mut context, &mut editor);
        assert_component_parents();
        editor.blueprint_creation.search = "Actor3D".into();
        frame(&mut context, &mut editor);
        SCRIPT_BUTTONS.with(|buttons| assert!(!buttons.borrow().contains_key("epok::Actor3D")));
        editor.blueprint_creation.context = crate::actor_scripts::CreationContext::Project;
        frame(&mut context, &mut editor);
        SCRIPT_BUTTONS.with(|buttons| assert!(buttons.borrow().contains_key("epok::Actor3D")));
        drop(editor);
        if root.exists() {
            std::fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    #[ignore = "Requires pinned reflection toolchain; exercises the real ImGui Inspector and Blueprint creation dialog"]
    fn actor_blueprint_create_attach_and_add_component_show_the_assignment() {
        let mut context = crate::gui::tests::imgui_context();
        context.set_ini_filename(None);
        context.io_mut().display_size = [1600., 2000.];
        context.io_mut().delta_time = 1. / 60.;
        context
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
        context.fonts().build_rgba32_texture();
        context.load_ini_settings("[Window][Inspector]\nPos=0,0\nSize=1000,1800\nCollapsed=0\n");
        let root = crate::workspace::tests::temp("actor-blueprint-inspector");
        let project =
            crate::workspace::create(&root, "Actor Inspector", crate::workspace::Template::Basic)
                .unwrap();
        let mut editor = Editor::open(project).unwrap();
        editor.auto_build = false;
        editor
            .scene
            .actors
            .push(crate::scene::Actor::cube("Blue Cube".into()));
        editor.scene.sync_actor_components();
        editor.select_actor(Some(editor.scene.actors[1].id));
        let original = editor.scene.actors[1].clone();
        fn frame(context: &mut imgui::Context, editor: &mut Editor, wizard: bool) {
            let ui = context.frame();
            if wizard {
                crate::blueprint_workflow::draw(ui, editor);
            } else {
                unsafe {
                    imgui::sys::igSetNextWindowSize(
                        imgui::sys::ImVec2 { x: 1000., y: 1800. },
                        imgui::sys::ImGuiCond_Always as i32,
                    );
                }
                inspector(ui, editor);
            }
            context.render();
        }
        fn click(context: &mut imgui::Context, editor: &mut Editor, label: &str, wizard: bool) {
            frame(context, editor, wizard);
            frame(context, editor, wizard);
            let point = SCRIPT_BUTTONS.with(|buttons| {
                *buttons
                    .borrow()
                    .get(label)
                    .unwrap_or_else(|| panic!("Missing control {label}"))
            });
            context.io_mut().add_mouse_pos_event(point);
            frame(context, editor, wizard);
            context
                .io_mut()
                .add_mouse_button_event(imgui::MouseButton::Left, true);
            frame(context, editor, wizard);
            context
                .io_mut()
                .add_mouse_button_event(imgui::MouseButton::Left, false);
            frame(context, editor, wizard);
        }
        click(&mut context, &mut editor, "Add Component", false);
        click(
            &mut context,
            &mut editor,
            "Create Blueprint ActorComponent...",
            false,
        );
        assert!(
            matches!(editor.blueprint_creation.context, crate::actor_scripts::CreationContext::Component(actor) if actor == original.id)
        );
        editor.blueprint_creation.name = "BP_Box".into();
        editor.blueprint_creation.owner_domain = 1;
        click(&mut context, &mut editor, "Create and Attach", true);
        assert!(
            editor.blueprint_creation.error.is_none(),
            "{:?}",
            editor.blueprint_creation.error
        );
        let actor = editor.scene.actors[1].clone();
        let added = actor
            .components
            .iter()
            .find(|c| c.class.name == "BP_Box")
            .expect("Created Blueprint must be attached");
        assert_eq!(actor.id, original.id);
        assert_eq!(actor.class, original.class);
        assert_eq!(actor.components.len(), original.components.len() + 1);
        let first_id = added.id;
        editor.undo_attachment(false).unwrap();
        assert_eq!(editor.scene.actors[1], original);
        editor.undo_attachment(true).unwrap();
        assert_eq!(editor.scene.actors[1], actor);
        editor.blueprint_editor = Default::default();
        click(&mut context, &mut editor, "Open Blueprint", false);
        assert_eq!(
            editor.blueprint_editor.asset.as_ref().unwrap().name,
            "BP_Box"
        );
        click(&mut context, &mut editor, "Remove Component", false);
        assert!(
            !editor.scene.actors[1]
                .components
                .iter()
                .any(|c| c.id == first_id)
        );
        click(&mut context, &mut editor, "Add Component", false);
        click(&mut context, &mut editor, "BP_Box", false);
        assert!(editor.last_error.is_none(), "{:?}", editor.last_error);
        assert!(
            editor.scene.actors[1]
                .components
                .iter()
                .any(|c| c.class.name == "BP_Box" && c.id != first_id)
        );
        crate::blueprint_workflow::begin(
            &mut editor,
            Some(crate::object_model::ACTOR_COMPONENT_ID.into()),
        );
        editor.blueprint_creation.name = "BP_Rotate".into();
        click(&mut context, &mut editor, "Create and Attach", true);
        assert!(
            editor.blueprint_creation.error.is_none(),
            "{:?}",
            editor.blueprint_creation.error
        );
        assert!(
            editor.scene.actors[1]
                .components
                .iter()
                .any(|c| c.class.name == "BP_Rotate")
        );
        assert_eq!(editor.scene.actors[1].id, original.id);
        crate::scripts::create_in(
            &editor.root,
            "NativeCollider",
            "",
            "epok::Collider3DComponent",
            true,
        )
        .unwrap();
        editor.refresh_scripts();
        editor.attach("NativeCollider");
        assert!(editor.last_error.is_none(), "{:?}", editor.last_error);
        assert!(
            editor.scene.actors[1]
                .components
                .iter()
                .any(|c| c.class.name == "NativeCollider")
        );
        assert!(editor.save());
        let restored = crate::scene::Scene::load(&editor.scene_path()).unwrap();
        assert_eq!(restored.actors, editor.scene.actors);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    #[ignore = "Requires pinned libclang/MIPS SDK and built epok-header-tool; run explicitly after cargo build --bins"]
    fn blueprint_creation_dialog_reflects_inherits_attaches_and_undoes() {
        let mut context = crate::gui::tests::imgui_context();
        context.set_ini_filename(None);
        context.io_mut().display_size = [1280., 720.];
        context.io_mut().delta_time = 1. / 60.;
        context
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
        context.fonts().build_rgba32_texture();
        let root = crate::workspace::tests::temp("blueprint-ui");
        let project =
            crate::workspace::create(&root, "UI acceptance", crate::workspace::Template::Basic)
                .unwrap();
        let mut editor = Editor::open(project).unwrap();
        editor.auto_build = false;
        editor.selected = Some(0);
        let frame = |context: &mut imgui::Context, editor: &mut Editor| {
            script_creation_dialog(context.frame(), editor);
            context.render();
        };
        let click = |context: &mut imgui::Context, editor: &mut Editor, label: &str| {
            let point = SCRIPT_BUTTONS.with(|buttons| buttons.borrow()[label]);
            context.io_mut().add_mouse_pos_event(point);
            frame(context, editor);
            context
                .io_mut()
                .add_mouse_button_event(imgui::MouseButton::Left, true);
            frame(context, editor);
            context
                .io_mut()
                .add_mouse_button_event(imgui::MouseButton::Left, false);
            frame(context, editor);
        };
        frame(&mut context, &mut editor);
        editor.action("new-script");
        editor.script_name = "Enemy".into();
        frame(&mut context, &mut editor);
        frame(&mut context, &mut editor);
        click(&mut context, &mut editor, "Create");
        assert!(
            root.join("assets/scripts/Enemy.hpp").exists(),
            "{:?}",
            editor.script_error
        );
        let header = root.join("assets/scripts/Enemy.hpp");
        let text=std::fs::read_to_string(&header).unwrap().replace("public:","public:\n EPOK_PROPERTY(EditAnywhere) epok::Fixed health=100.0;\n EPOK_FUNCTION(BlueprintEvent) virtual void damaged(epok::Fixed amount) {health-=amount;}\n");
        std::fs::write(&header, text).unwrap();
        editor.refresh_scripts();
        editor.action("new-script");
        editor.script_name = "Boss".into();
        editor.script_folder = "Enemies/Bosses".into();
        editor.script_parent = "Enemy".into();
        editor.script_search = "enemy".into();
        frame(&mut context, &mut editor);
        frame(&mut context, &mut editor);
        click(&mut context, &mut editor, "Create and Attach");
        assert!(
            root.join("assets/scripts/Enemies/Bosses/Boss.hpp").exists(),
            "{:?}",
            editor.script_error
        );
        assert_eq!(editor.scene.actors[0].class.name, "Boss");
        assert!(editor.scene.actors[0].class.class_id.is_some());
        assert_eq!(editor.class_registry.properties("Boss")[0].name, "health");
        assert!(editor.class_registry.ancestry("Boss").iter().any(|c| {
            c.functions
                .iter()
                .any(|f| f.name == "damaged" && f.parameters.len() == 1)
        }));
        assert!(
            !std::fs::read_to_string(root.join("assets/scripts/Enemies/Bosses/Boss.hpp"))
                .unwrap()
                .contains("override")
        );
        editor.undo_attachment(false).unwrap();
        assert!(editor.scene.actors[0].components.iter().all(|c| {
            c.class
                .class_id
                .as_deref()
                .is_some_and(crate::actor_components::native)
        }));
        editor.undo_attachment(true).unwrap();
        assert!(editor.save());
        assert_eq!(
            crate::scene::Scene::load(&editor.scene_path())
                .unwrap()
                .actors[0]
                .components,
            editor.scene.actors[0].components
        );
        // Rejected transactions leave neither half-created files nor new folders.
        editor.action("new-script");
        editor.script_name = "Boss".into();
        editor.script_folder = "ShouldNotExist".into();
        frame(&mut context, &mut editor);
        frame(&mut context, &mut editor);
        click(&mut context, &mut editor, "Create");
        assert!(
            editor
                .script_error
                .as_ref()
                .unwrap()
                .contains("already exists")
        );
        assert!(!root.join("assets/scripts/ShouldNotExist").exists());
        click(&mut context, &mut editor, "Cancel");
        // A user abstract base is creatable, but attaching a still-abstract child
        // rolls the transaction back instead of generating suppressing overrides.
        let text = std::fs::read_to_string(&header).unwrap().replace(
            "public:",
            "public:\n EPOK_FUNCTION(BlueprintEvent) virtual void required_event() = 0;\n",
        );
        std::fs::write(&header, text).unwrap();
        editor.refresh_scripts();
        editor.action("new-script");
        editor.script_name = "AbstractChild".into();
        editor.script_parent = "Enemy".into();
        editor.script_folder = "AbstractCases".into();
        frame(&mut context, &mut editor);
        frame(&mut context, &mut editor);
        click(&mut context, &mut editor, "Create and Attach");
        assert!(
            editor
                .script_error
                .as_ref()
                .unwrap()
                .contains("pure virtual")
        );
        assert!(!root.join("assets/scripts/AbstractCases").exists());
        click(&mut context, &mut editor, "Create");
        assert!(
            editor
                .catalog
                .iter()
                .any(|s| s.name == "AbstractChild" && !s.instantiable())
        );
        drop(editor);
        std::fs::remove_dir_all(root).unwrap();
    }
    // Exercise actual Dear ImGui events and the normal editor panels without a window/GPU.
    #[test]
    fn scene_clicks_and_hierarchy_context_menu_use_real_imgui_events() {
        let mut context = crate::gui::tests::imgui_context();
        configure_input(context.io_mut());
        context.set_ini_filename(None);
        context.io_mut().display_size = [1440., 900.];
        context.io_mut().delta_time = 1. / 60.;
        context.io_mut().config_flags |=
            imgui::ConfigFlags::DOCKING_ENABLE | imgui::ConfigFlags::NAV_ENABLE_KEYBOARD;
        let font = context
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
        context.fonts().build_rgba32_texture();
        theme(context.style_mut());
        // Render the actual Game panel without a GPU. Its image draw command
        // must fill one axis in Fit, both in Stretch, and stay integral in Integer.
        {
            let mut game = Editor::new(crate::workspace::tests::temp("game-scale-ui"));
            game.game_frame = Some(std::sync::Arc::new(crate::bridge::Frame {
                width: 640,
                height: 480,
                sequence: 1,
                buttons: 0,
                cycles: 0,
                vsyncs: 0,
                rgba: vec![],
            }));
            let texture = imgui::TextureId::new(913);
            context
                .load_ini_settings("[Window][###Game]\nPos=10,10\nSize=1000,760\nCollapsed=0\n\n");
            let mut sizes = vec![];
            for mode in crate::settings::GameScale::ALL {
                game.preferences.game_scale = mode;
                for _ in 0..2 {
                    game_view(context.frame(), &mut game, Some(texture));
                    context.render();
                }
                game_view(context.frame(), &mut game, Some(texture));
                let data = context.render();
                let mut min = [f32::MAX; 2];
                let mut max = [f32::MIN; 2];
                for list in data.draw_lists() {
                    for command in list.commands() {
                        if let imgui::DrawCmd::Elements { count, cmd_params } = command
                            && cmd_params.texture_id == texture
                        {
                            for &index in &list.idx_buffer()
                                [cmd_params.idx_offset..cmd_params.idx_offset + count]
                            {
                                let p =
                                    list.vtx_buffer()[index as usize + cmd_params.vtx_offset].pos;
                                for axis in 0..2 {
                                    min[axis] = min[axis].min(p[axis]);
                                    max[axis] = max[axis].max(p[axis]);
                                }
                            }
                        }
                    }
                }
                assert!(min[0].is_finite() && max[0] > min[0], "Game image missing");
                sizes.push([max[0] - min[0], max[1] - min[1]]);
            }
            let (fit, stretch, integer) = (sizes[0], sizes[1], sizes[2]);
            assert!((fit[0] / fit[1] - 4. / 3.).abs() < 0.001);
            assert!((fit[0] - stretch[0]).abs() < 0.01 || (fit[1] - stretch[1]).abs() < 0.01);
            assert!(stretch[0] >= fit[0] - 0.01 && stretch[1] >= fit[1] - 0.01);
            assert_eq!(integer, [640., 480.]);
        }
        crate::asset_inspector::verify_interactions(&mut context);
        verify_numeric_entry(&mut context);
        crate::console::verify_interactions(&mut context);
        let mut editor = Editor::new(std::env::temp_dir().join("epok-ui-interactions"));
        // Exercise dependency warnings separately, without covering scene controls.
        editor.dependencies.warning = false;
        editor.auto_build = false;
        editor.selected = None;
        editor.dirty = false;
        editor.catalog = crate::mcp_tests::actor_catalog();
        editor.class_registry =
            crate::blueprint::registry_from_catalog(&editor.root, &editor.catalog);
        editor.registry_revision += 1;
        let mut initial = true;
        let frame = |ctx: &mut imgui::Context, e: &mut Editor, initial: &mut bool| {
            draw(
                ctx.frame(),
                e,
                [
                    imgui::TextureId::new(999),
                    imgui::TextureId::new(1000),
                    imgui::TextureId::new(1001),
                ],
                None,
                font,
                [960., 600.],
                initial,
            );
            let data = ctx.render();
            let mut bounds = ([f32::INFINITY; 2], [f32::NEG_INFINITY; 2]);
            for list in data.draw_lists() {
                for command in list.commands() {
                    if let imgui::DrawCmd::Elements { count, cmd_params } = command
                        && [imgui::TextureId::new(999), imgui::TextureId::new(1000)]
                            .contains(&cmd_params.texture_id)
                    {
                        for index in
                            &list.idx_buffer()[cmd_params.idx_offset..cmd_params.idx_offset + count]
                        {
                            let p = list.vtx_buffer()[*index as usize + cmd_params.vtx_offset].pos;
                            for (axis, value) in p.into_iter().enumerate() {
                                bounds.0[axis] = bounds.0[axis].min(value);
                                bounds.1[axis] = bounds.1[axis].max(value);
                            }
                        }
                    }
                }
            }
            bounds
        };
        frame(&mut context, &mut editor, &mut initial);
        let (origin, end) = frame(&mut context, &mut editor, &mut initial);
        assert!(origin[0].is_finite());
        // A restored layout may have Game selected. Project entry selects Scene
        // once, while subsequent user tab selections remain under user control.
        unsafe {
            imgui::sys::igSetWindowFocus_Str(c"\u{ec17} Game###Game".as_ptr());
        }
        frame(&mut context, &mut editor, &mut initial);
        editor.focus_scene = true;
        frame(&mut context, &mut editor, &mut initial);
        let (restored, _) = frame(&mut context, &mut editor, &mut initial);
        assert!(
            restored[0].is_finite(),
            "Scene should be visible after reopening a project"
        );
        assert!(!editor.focus_scene);
        // Stop selects the real docked Scene tab and releases Game input, even
        // when Game had a pending focus request from the emulator startup.
        editor.focus_game = true;
        frame(&mut context, &mut editor, &mut initial);
        frame(&mut context, &mut editor, &mut initial);
        let (job, events, controls) = crate::pipeline::Job::test_channels();
        editor.job = Some(job);
        editor.playing = true;
        editor.game_capture = true;
        editor.focus_game = true;
        editor.action("play");
        assert!(matches!(
            controls.try_recv(),
            Ok(crate::pipeline::Control::Stop)
        ));
        frame(&mut context, &mut editor, &mut initial);
        let (stopped, _) = frame(&mut context, &mut editor, &mut initial);
        assert!(stopped[0].is_finite(), "Stop must select the Scene tab");
        assert!(!editor.game_capture && !editor.focus_game);
        events
            .send(crate::pipeline::Event::Finished(Ok(())))
            .unwrap();
        editor.tick();
        frame(&mut context, &mut editor, &mut initial);
        frame(&mut context, &mut editor, &mut initial);
        editor.focus_game = true;
        frame(&mut context, &mut editor, &mut initial);
        frame(&mut context, &mut editor, &mut initial);
        unsafe {
            assert!(
                (*imgui::sys::igFindWindowByName(c"\u{ec17} Game###Game".as_ptr()))
                    .DockTabIsVisible(),
                "Stop focus must not override later manual tab selection"
            );
        }
        editor.focus_scene = true;
        frame(&mut context, &mut editor, &mut initial);
        frame(&mut context, &mut editor, &mut initial);
        // A saved Console panel must yield to Project once, then remain selectable.
        // Console and Project sit in separate dock nodes, so both tabs stay
        // visible: selection is the panel that actually holds focus.
        let selected = || unsafe {
            let window = (*imgui::sys::igGetCurrentContext()).NavWindow;
            assert!(!window.is_null(), "A panel must always keep focus");
            assert!(
                (*window).DockTabIsVisible(),
                "The focused panel must be the selected tab"
            );
            std::ffi::CStr::from_ptr((*window).Name)
                .to_string_lossy()
                .into_owned()
        };
        let (project, console) = ("\u{eb30} Project###Project", "\u{eb9b} Console###Console");
        editor.focus_console = true;
        frame(&mut context, &mut editor, &mut initial);
        frame(&mut context, &mut editor, &mut initial);
        assert_eq!(selected(), console, "User can select Console");
        let mut saved = String::new();
        context.save_ini_settings(&mut saved);
        context.load_ini_settings(&saved);
        editor.focus_project = true;
        frame(&mut context, &mut editor, &mut initial);
        frame(&mut context, &mut editor, &mut initial);
        assert_eq!(
            selected(),
            project,
            "Project is active after restoring a Console layout"
        );
        assert!(!editor.focus_project);
        editor.focus_console = true;
        frame(&mut context, &mut editor, &mut initial);
        frame(&mut context, &mut editor, &mut initial);
        assert_eq!(
            selected(),
            console,
            "Startup focus must not steal later tab selection"
        );
        let size = [end[0] - origin[0], end[1] - origin[1]];
        let factor = (size[0] / 960.).max(size[1] / 600.);
        let uv = [size[0] / (960. * factor), size[1] / (600. * factor)];
        let p = crate::viewport::project(&editor.view, editor.scene.world_matrix(1).point([0.; 3]));
        let mouse = [
            origin[0] + (p[0] - (1. - uv[0]) * 480.) * factor,
            origin[1] + (p[1] - (1. - uv[1]) * 300.) * factor,
        ];
        context.io_mut().add_mouse_pos_event(mouse);
        frame(&mut context, &mut editor, &mut initial);
        context
            .io_mut()
            .add_mouse_button_event(imgui::MouseButton::Left, true);
        frame(&mut context, &mut editor, &mut initial);
        context
            .io_mut()
            .add_mouse_button_event(imgui::MouseButton::Left, false);
        frame(&mut context, &mut editor, &mut initial);
        assert_eq!(editor.selected, Some(1));
        assert!(!editor.dirty, "Selecting must not edit the scene");
        // Same click now hits the transform handle. Releasing it must not clear selection.
        context
            .io_mut()
            .add_mouse_button_event(imgui::MouseButton::Left, true);
        frame(&mut context, &mut editor, &mut initial);
        assert!(editor.drag_axis.is_some());
        context
            .io_mut()
            .add_mouse_button_event(imgui::MouseButton::Left, false);
        frame(&mut context, &mut editor, &mut initial);
        assert_eq!(editor.selected, Some(1));
        context
            .io_mut()
            .add_mouse_pos_event([origin[0] + size[0] * 0.5, origin[1] + 40.]);
        frame(&mut context, &mut editor, &mut initial);
        context
            .io_mut()
            .add_mouse_button_event(imgui::MouseButton::Left, true);
        frame(&mut context, &mut editor, &mut initial);
        context
            .io_mut()
            .add_mouse_button_event(imgui::MouseButton::Left, false);
        frame(&mut context, &mut editor, &mut initial);
        assert_eq!(editor.selected, None);
        // Right-click empty Hierarchy space, then click Create Empty in its popup.
        context.io_mut().add_mouse_pos_event([80., 400.]);
        frame(&mut context, &mut editor, &mut initial);
        context
            .io_mut()
            .add_mouse_button_event(imgui::MouseButton::Right, true);
        frame(&mut context, &mut editor, &mut initial);
        context
            .io_mut()
            .add_mouse_button_event(imgui::MouseButton::Right, false);
        frame(&mut context, &mut editor, &mut initial);
        frame(&mut context, &mut editor, &mut initial);
        let count = editor.scene.actors.len();
        context.io_mut().add_mouse_pos_event([110., 410.]);
        frame(&mut context, &mut editor, &mut initial);
        context
            .io_mut()
            .add_mouse_button_event(imgui::MouseButton::Left, true);
        frame(&mut context, &mut editor, &mut initial);
        context
            .io_mut()
            .add_mouse_button_event(imgui::MouseButton::Left, false);
        frame(&mut context, &mut editor, &mut initial);
        frame(&mut context, &mut editor, &mut initial);
        let instantiate = SCRIPT_BUTTONS
            .with(|b| b.borrow().get("Instantiate").copied())
            .expect("Actor class selection modal");
        assert_eq!(
            editor.scene.actors.len(),
            count,
            "Opening the chooser does not place an Actor"
        );
        context.io_mut().add_mouse_pos_event(instantiate);
        frame(&mut context, &mut editor, &mut initial);
        context
            .io_mut()
            .add_mouse_button_event(imgui::MouseButton::Left, true);
        frame(&mut context, &mut editor, &mut initial);
        context
            .io_mut()
            .add_mouse_button_event(imgui::MouseButton::Left, false);
        frame(&mut context, &mut editor, &mut initial);
        assert_eq!(
            editor.scene.actors.len(),
            count + 1,
            "Context menu should create an entity"
        );
        let created = &editor.scene.actors[editor.selected.unwrap()];
        assert_eq!(created.kind, "Empty");
        assert_eq!(created.parent, None);
        // F2 editing goes through the actual text input, including keyboard focus.
        frame(&mut context, &mut editor, &mut initial);
        context.io_mut().add_key_event(imgui::Key::F2, true);
        frame(&mut context, &mut editor, &mut initial);
        let focus = unsafe {
            let c = imgui::sys::igGetCurrentContext();
            let w = (*c).NavWindow;
            if w.is_null() {
                "none".into()
            } else {
                std::ffi::CStr::from_ptr((*w).Name)
                    .to_string_lossy()
                    .into_owned()
            }
        };
        assert!(
            editor.actor_rename.is_some(),
            "F2 should start rename; focus={focus}, text={}",
            context.io().want_text_input
        );
        context.io_mut().add_key_event(imgui::Key::F2, false);
        frame(&mut context, &mut editor, &mut initial);
        frame(&mut context, &mut editor, &mut initial);
        for c in "Player HUD".chars() {
            context.io_mut().add_input_character(c);
        }
        frame(&mut context, &mut editor, &mut initial);
        context.io_mut().add_key_event(imgui::Key::Enter, true);
        frame(&mut context, &mut editor, &mut initial);
        context.io_mut().add_key_event(imgui::Key::Enter, false);
        frame(&mut context, &mut editor, &mut initial);
        assert_eq!(
            editor.scene.actors[editor.selected.unwrap()].name,
            "Player HUD"
        );
        // Middle-button pan and right-button WASD move the camera, not the entity.
        let center = editor.view.center;
        let selected = editor.selected;
        let point = [origin[0] + size[0] * 0.5, origin[1] + 60.];
        context.io_mut().add_mouse_pos_event(point);
        frame(&mut context, &mut editor, &mut initial);
        context
            .io_mut()
            .add_mouse_button_event(imgui::MouseButton::Middle, true);
        frame(&mut context, &mut editor, &mut initial);
        context
            .io_mut()
            .add_mouse_pos_event([point[0] + 40., point[1] + 20.]);
        frame(&mut context, &mut editor, &mut initial);
        context
            .io_mut()
            .add_mouse_button_event(imgui::MouseButton::Middle, false);
        frame(&mut context, &mut editor, &mut initial);
        assert_ne!(center, editor.view.center);
        assert_eq!(editor.selected, selected);
        let center = editor.view.center;
        let tool = editor.tool;
        // Navigation must take keyboard ownership even if a rename field is active.
        editor.begin_actor_rename(editor.scene.actors[editor.selected.unwrap()].id);
        unsafe {
            imgui::sys::igSetWindowFocus_Str(c"\u{eb86} Hierarchy###Hierarchy".as_ptr());
        }
        frame(&mut context, &mut editor, &mut initial);
        frame(&mut context, &mut editor, &mut initial);
        frame(&mut context, &mut editor, &mut initial);
        assert!(
            context.io().want_text_input,
            "rename={:?}, focus={}",
            editor.actor_rename,
            editor.actor_rename_focus
        );
        context
            .io_mut()
            .add_mouse_button_event(imgui::MouseButton::Right, true);
        frame(&mut context, &mut editor, &mut initial);
        assert!(
            editor.scene_look,
            "RMB must release the active text field and enter free look"
        );
        context.io_mut().add_key_event(imgui::Key::W, true);
        frame(&mut context, &mut editor, &mut initial);
        context.io_mut().add_key_event(imgui::Key::W, false);
        context
            .io_mut()
            .add_mouse_button_event(imgui::MouseButton::Right, false);
        frame(&mut context, &mut editor, &mut initial);
        assert_ne!(center, editor.view.center);
        assert_eq!(tool, editor.tool);
        // A HUD Image is the same selectable entity; dragging edits RectTransform.
        editor.create_hud("panel");
        let panel = editor.selected.unwrap();
        frame(&mut context, &mut editor, &mut initial);
        let (a, b) = frame(&mut context, &mut editor, &mut initial);
        let p = [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5];
        let original = editor.scene.actors[panel].rect.clone();
        context.io_mut().add_mouse_pos_event(p);
        frame(&mut context, &mut editor, &mut initial);
        context
            .io_mut()
            .add_mouse_button_event(imgui::MouseButton::Left, true);
        frame(&mut context, &mut editor, &mut initial);
        context
            .io_mut()
            .add_mouse_pos_event([p[0] + 20., p[1] - 10.]);
        frame(&mut context, &mut editor, &mut initial);
        context
            .io_mut()
            .add_mouse_button_event(imgui::MouseButton::Left, false);
        frame(&mut context, &mut editor, &mut initial);
        assert_eq!(editor.selected, Some(panel));
        assert_ne!(original, editor.scene.actors[panel].rect);
        crate::hub::verify_interactions(&mut context);
        crate::asset_ui::interaction::verify(&mut context);
        crate::project_browser::verify_interactions(&mut context, font);
        crate::mesh_editor::verify_interactions(&mut context);
        crate::settings_ui::verify_interactions(&mut context);
        crate::dependencies::verify_interactions(&mut context);
        crate::busy_ui::verify_interactions(&mut context);
        crate::platform::verify_modal_indices(&mut context);
        verify_floating_scene_drag(&mut context);
    }

    fn verify_numeric_entry(ctx: &mut imgui::Context) {
        let mut value = 1.25f32;
        let mut integer = 3i32;
        fn frame(ctx: &mut imgui::Context, value: &mut f32, integer: &mut i32) -> [[f32; 2]; 2] {
            let mut points = [[0.; 2]; 2];
            let ui = ctx.frame();
            ui.window("Numeric entry regression")
                .position([450., 20.], Condition::Always)
                .size([400., 220.], Condition::Always)
                .build(|| {
                    crate::gui::Drag::new("Decimal").speed(0.1).build(ui, value);
                    points[0] = [
                        (ui.item_rect_min()[0] + ui.item_rect_max()[0]) * 0.5,
                        (ui.item_rect_min()[1] + ui.item_rect_max()[1]) * 0.5,
                    ];
                    crate::gui::Drag::new("Integer")
                        .speed(1.)
                        .build(ui, integer);
                    points[1] = [
                        (ui.item_rect_min()[0] + ui.item_rect_max()[0]) * 0.5,
                        (ui.item_rect_min()[1] + ui.item_rect_max()[1]) * 0.5,
                    ];
                });
            ctx.render();
            points
        }
        frame(ctx, &mut value, &mut integer);
        let points = frame(ctx, &mut value, &mut integer);
        for (i, text) in [(0, "12.5"), (1, "42")] {
            ctx.io_mut().add_mouse_pos_event(points[i]);
            frame(ctx, &mut value, &mut integer);
            ctx.io_mut()
                .add_mouse_button_event(imgui::MouseButton::Left, true);
            frame(ctx, &mut value, &mut integer);
            ctx.io_mut()
                .add_mouse_button_event(imgui::MouseButton::Left, false);
            frame(ctx, &mut value, &mut integer);
            frame(ctx, &mut value, &mut integer);
            assert!(ctx.io().want_text_input, "A short click enters text mode");
            assert_ne!(ctx.mouse_cursor(), Some(imgui::MouseCursor::ResizeEW));
            for c in text.chars() {
                ctx.io_mut().add_input_character(c);
            }
            frame(ctx, &mut value, &mut integer);
            ctx.io_mut().add_key_event(imgui::Key::Enter, true);
            frame(ctx, &mut value, &mut integer);
            ctx.io_mut().add_key_event(imgui::Key::Enter, false);
            frame(ctx, &mut value, &mut integer);
        }
        assert_eq!(value, 12.5);
        assert_eq!(integer, 42);
        for _ in 0..25 {
            frame(ctx, &mut value, &mut integer);
        }
        ctx.io_mut().add_mouse_pos_event(points[0]);
        frame(ctx, &mut value, &mut integer);
        ctx.io_mut()
            .add_mouse_button_event(imgui::MouseButton::Left, true);
        frame(ctx, &mut value, &mut integer);
        ctx.io_mut()
            .add_mouse_pos_event([points[0][0] + 50., points[0][1]]);
        frame(ctx, &mut value, &mut integer);
        assert_eq!(ctx.mouse_cursor(), Some(imgui::MouseCursor::ResizeEW));
        frame(ctx, &mut value, &mut integer);
        assert_eq!(
            ctx.mouse_cursor(),
            Some(imgui::MouseCursor::ResizeEW),
            "Keep the cursor while holding still"
        );
        ctx.io_mut()
            .add_mouse_button_event(imgui::MouseButton::Left, false);
        frame(ctx, &mut value, &mut integer);
        assert!(value > 12.5);
        assert_ne!(ctx.mouse_cursor(), Some(imgui::MouseCursor::ResizeEW));
        assert!(
            !ctx.io().want_text_input,
            "Dragging never switches to text editing on release"
        );
        let mut vector = [1f32, 2., 3.];
        let vector_frame = |ctx: &mut imgui::Context, vector: &mut [f32; 3]| {
            let ui = ctx.frame();
            let mut point = [0.; 2];
            ui.window("Numeric vector regression")
                .position([450., 20.], Condition::Always)
                .size([400., 220.], Condition::Always)
                .build(|| {
                    crate::gui::Drag::new("Vector")
                        .speed(0.1)
                        .build_array(ui, vector);
                    point = [
                        ui.item_rect_min()[0] + 20.,
                        (ui.item_rect_min()[1] + ui.item_rect_max()[1]) * 0.5,
                    ];
                });
            ctx.render();
            point
        };
        vector_frame(ctx, &mut vector);
        let point = vector_frame(ctx, &mut vector);
        ctx.io_mut().add_mouse_pos_event(point);
        vector_frame(ctx, &mut vector);
        ctx.io_mut()
            .add_mouse_button_event(imgui::MouseButton::Left, true);
        vector_frame(ctx, &mut vector);
        ctx.io_mut().add_mouse_pos_event([point[0] + 30., point[1]]);
        vector_frame(ctx, &mut vector);
        assert!(vector[0] > 1.);
        assert_eq!(ctx.mouse_cursor(), Some(imgui::MouseCursor::ResizeEW));
        ctx.io_mut()
            .add_mouse_button_event(imgui::MouseButton::Left, false);
        vector_frame(ctx, &mut vector);
        assert_ne!(ctx.mouse_cursor(), Some(imgui::MouseCursor::ResizeEW));
    }
    fn verify_floating_scene_drag(context: &mut imgui::Context) {
        let mut editor = Editor::new(std::env::temp_dir().join("epok-floating-scene-test"));
        editor.selected = Some(1);
        editor.tool = 1;
        let frame = |context: &mut imgui::Context, editor: &mut Editor, reset: bool| {
            let ui = context.frame();
            if reset {
                unsafe {
                    imgui::sys::igSetNextWindowDockID(0, imgui::sys::ImGuiCond_Always as i32);
                    imgui::sys::igSetNextWindowPos(
                        imgui::sys::ImVec2 { x: 180., y: 160. },
                        imgui::sys::ImGuiCond_Always as i32,
                        imgui::sys::ImVec2 { x: 0., y: 0. },
                    );
                    imgui::sys::igSetNextWindowSize(
                        imgui::sys::ImVec2 { x: 850., y: 620. },
                        imgui::sys::ImGuiCond_Always as i32,
                    );
                }
            }
            scene_view(
                ui,
                editor,
                imgui::TextureId::new(999),
                imgui::TextureId::new(1000),
                [960., 600.],
            );
            let window_pos = unsafe {
                let window = imgui::sys::igFindWindowByName(c"\u{eb29} Scene###Scene".as_ptr());
                [(*window).Pos.x, (*window).Pos.y]
            };
            let data = context.render();
            let mut bounds = ([f32::INFINITY; 2], [f32::NEG_INFINITY; 2]);
            for list in data.draw_lists() {
                for command in list.commands() {
                    if let imgui::DrawCmd::Elements { count, cmd_params } = command
                        && cmd_params.texture_id == imgui::TextureId::new(999)
                    {
                        for index in
                            &list.idx_buffer()[cmd_params.idx_offset..cmd_params.idx_offset + count]
                        {
                            let pos =
                                list.vtx_buffer()[*index as usize + cmd_params.vtx_offset].pos;
                            for (axis, value) in pos.iter().enumerate() {
                                bounds.0[axis] = bounds.0[axis].min(*value);
                                bounds.1[axis] = bounds.1[axis].max(*value);
                            }
                        }
                    }
                }
            }
            (window_pos, bounds)
        };
        frame(context, &mut editor, true);
        let (position, (origin, end)) = frame(context, &mut editor, false);
        let size = [end[0] - origin[0], end[1] - origin[1]];
        let factor = (size[0] / 960.).max(size[1] / 600.);
        let uv = [size[0] / (960. * factor), size[1] / (600. * factor)];
        let mut point = editor.scene.world_matrix(1).point([0.; 3]);
        point[0] += 0.75;
        let projected = crate::viewport::project(&editor.view, point);
        let mouse = [
            origin[0] + (projected[0] - (1. - uv[0]) * 480.) * factor,
            origin[1] + (projected[1] - (1. - uv[1]) * 300.) * factor,
        ];
        let original = editor.scene.actors[1].position;
        context.io_mut().add_mouse_pos_event(mouse);
        frame(context, &mut editor, false);
        context
            .io_mut()
            .add_mouse_button_event(imgui::MouseButton::Left, true);
        frame(context, &mut editor, false);
        assert_eq!(editor.drag_axis, Some(0));
        context
            .io_mut()
            .add_mouse_pos_event([mouse[0] + 35., mouse[1] + 12.]);
        frame(context, &mut editor, false);
        context
            .io_mut()
            .add_mouse_button_event(imgui::MouseButton::Left, false);
        let (after, _) = frame(context, &mut editor, false);
        assert_ne!(
            editor.scene.actors[1].position, original,
            "The gizmo must move the cube"
        );
        assert_eq!(
            after, position,
            "Dragging a gizmo must not move the floating Scene panel"
        );

        let pivot = editor.view.center;
        let lens = editor.view.zoom;
        let distance = editor.view.distance;
        let forward = editor.view.basis()[2];
        context.io_mut().add_mouse_wheel_event([0., 1.]);
        frame(context, &mut editor, false);
        for i in 0..3 {
            assert!(
                (editor.view.center[i] - pivot[i] - forward[i] * editor.view.fly_speed * 0.2).abs()
                    < 0.0001,
                "Wheel must move the camera along its viewing direction"
            );
        }
        assert_eq!(editor.view.zoom, lens, "Wheel must preserve the FOV");
        assert_eq!(editor.view.distance, distance);
        let yaw = editor.view.yaw;
        let middle = [origin[0] + size[0] * 0.5, origin[1] + size[1] * 0.5];
        context.io_mut().add_mouse_pos_event(middle);
        context.io_mut().add_key_event(imgui::Key::ModAlt, true);
        frame(context, &mut editor, false);
        context
            .io_mut()
            .add_mouse_button_event(imgui::MouseButton::Left, true);
        frame(context, &mut editor, false);
        context
            .io_mut()
            .add_mouse_pos_event([middle[0] + 40., middle[1] + 15.]);
        frame(context, &mut editor, false);
        context
            .io_mut()
            .add_mouse_button_event(imgui::MouseButton::Left, false);
        context.io_mut().add_key_event(imgui::Key::ModAlt, false);
        let (after, _) = frame(context, &mut editor, false);
        assert_ne!(editor.view.yaw, yaw);
        assert_eq!(
            after, position,
            "Camera orbit must not move the Scene panel"
        );

        let title = [position[0] + 180., position[1] + 10.];
        context.io_mut().add_mouse_pos_event(title);
        frame(context, &mut editor, false);
        context
            .io_mut()
            .add_mouse_button_event(imgui::MouseButton::Left, true);
        frame(context, &mut editor, false);
        context
            .io_mut()
            .add_mouse_pos_event([title[0] + 45., title[1] + 25.]);
        frame(context, &mut editor, false);
        context
            .io_mut()
            .add_mouse_button_event(imgui::MouseButton::Left, false);
        let (after, _) = frame(context, &mut editor, false);
        assert_ne!(after, position, "Title-bar dragging must remain available");
        context.io_mut().add_key_event(imgui::Key::F, true);
        frame(context, &mut editor, false);
        context.io_mut().add_key_event(imgui::Key::F, false);
        frame(context, &mut editor, false);
        assert_eq!(
            editor.view.zoom, lens,
            "Frame Selected must preserve the FOV"
        );
        for (a, b) in editor
            .view
            .center
            .into_iter()
            .zip(editor.scene.world_matrix(1).point([0.; 3]))
        {
            assert!((a - b).abs() < 0.0001);
        }
        assert!(editor.view.distance < distance);
    }
}

/// Serialised ownership of the process-global Dear ImGui context.
///
/// `igCreateContext` asserts that no other context is active, so every test
/// that owns a context must hold this guard for as long as it uses ImGui.
/// The guard also pins the keyboard shortcut scheme: Dear ImGui enables
/// `ConfigMacOSXBehaviors` on `__APPLE__`, which moves copy/paste and
/// line-navigation from Ctrl to Super and would otherwise make simulated
/// Ctrl-key input behave differently on macOS than on CI hosts.
#[cfg(test)]
pub mod tests {
    use std::sync::{Mutex, MutexGuard};

    static ACTIVE: Mutex<()> = Mutex::new(());

    pub struct ImguiContext {
        // Declaration order is the drop order: destroy the context first, then
        // release the lock, so a panicking test never leaks the global context.
        context: imgui::Context,
        _active: MutexGuard<'static, ()>,
    }

    impl std::ops::Deref for ImguiContext {
        type Target = imgui::Context;
        fn deref(&self) -> &imgui::Context {
            &self.context
        }
    }
    impl std::ops::DerefMut for ImguiContext {
        fn deref_mut(&mut self) -> &mut imgui::Context {
            &mut self.context
        }
    }

    /// Create the one ImGui context, waiting for any other test to release it.
    pub fn imgui_context() -> ImguiContext {
        // A test that panicked while holding the guard poisons the lock; the
        // context it owned was already destroyed while unwinding.
        let active = ACTIVE
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut context = imgui::Context::create();
        context.set_ini_filename(None);
        context.io_mut().config_mac_os_behaviors = false;
        ImguiContext {
            context,
            _active: active,
        }
    }
}
