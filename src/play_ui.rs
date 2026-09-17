use crate::play::{Content, DataSource, Profile, Target};

pub fn controls(ui: &imgui::Ui, profile: &mut Profile, root: &std::path::Path, width: f32) -> bool {
    let before = profile.clone();
    ui.set_next_item_width(width);
    if let Some(_combo) = ui.begin_combo("##play-target", profile.target.label()) {
        for target in Target::ALL {
            if ui
                .selectable_config(target.label())
                .selected(profile.target == target)
                .build()
            {
                profile.target = target;
                profile.normalize();
            }
        }
    }
    if ui.is_item_hovered() {
        ui.tooltip_text("Play destination. Saved in this project.");
    }
    ui.same_line();
    ui.set_next_item_width(width);
    if let Some(_combo) = ui.begin_combo("##play-content", content_label(profile.content)) {
        content_menu(ui, profile, root);
    }
    if ui.is_item_hovered() {
        ui.tooltip_text("Choose the scenes used by both Build and Play. Whole game starts at the project startup scene. Selected scenes lets you choose an initial scene.");
    }
    ui.same_line();
    ui.set_next_item_width(width);
    if let Some(_combo) = ui.begin_combo("##play-data", profile.data.label()) {
        for source in DataSource::ALL {
            let disabled = profile.target == Target::Serial && source == DataSource::Disc;
            let _disabled = ui.begin_disabled(disabled);
            if ui
                .selectable_config(source.label())
                .selected(profile.data == source)
                .build()
            {
                profile.data = source;
            }
        }
    }
    if ui.is_item_hovered() {
        ui.tooltip_text("Where external geometry is read. Enable Geometry Streaming in Project Settings to use on-demand pages. Scene banks, textures and scripts remain prelinked. CD is unavailable for Serial Play.");
    }
    *profile != before
}

pub fn toolbar(ui: &imgui::Ui, e: &mut crate::editor::Editor) {
    let _disabled = ui.begin_disabled(e.job.is_some() || e.dependencies.busy());
    let mut profile = e.play_profile.clone();
    if std::env::args().any(|a| a == "--screenshot-play-menu")
        && !crate::busy_ui::popup_open("play-options")
    {
        ui.open_popup("play-options");
    }
    if ui.button("\u{eab4}##play-options") {
        ui.open_popup("play-options");
    }
    if ui.is_item_hovered() {
        ui.tooltip_text(format!(
            "Play options\n{} / {} / {}",
            profile.target.label(),
            content_label(profile.content),
            profile.data.label()
        ));
    }
    let anchor = [ui.item_rect_min()[0], ui.item_rect_max()[1] + 4.];
    if crate::busy_ui::popup_open("play-options") {
        // Anchor to the arrow even when opened using keyboard navigation.
        unsafe {
            imgui::sys::igSetNextWindowPos(
                imgui::sys::ImVec2 {
                    x: anchor[0],
                    y: anchor[1],
                },
                imgui::Condition::Always as i32,
                imgui::sys::ImVec2 { x: 0., y: 0. },
            );
        }
    }
    ui.popup("play-options", || {
        menu(ui, &mut profile, &e.root);
        ui.separator();
        if ui.menu_item("PSX connection...") {
            crate::serial_ui::open(e);
        }
    });
    if profile != e.play_profile
        && let Err(error) = e.set_play_profile(profile)
    {
        e.log(error);
    }
}

pub fn content_label(content: Content) -> &'static str {
    match content {
        Content::CurrentScene => "Current scene",
        Content::WholeGame => "Whole game",
        Content::SelectedScenes => "Selected scenes",
    }
}

/// Check marks represent one mutually exclusive choice within each section.
pub fn menu(ui: &imgui::Ui, profile: &mut Profile, root: &std::path::Path) {
    ui.text_disabled("Destination");
    for target in Target::ALL {
        if ui
            .menu_item_config(target.label())
            .selected(profile.target == target)
            .build()
        {
            profile.target = target;
            profile.normalize();
        }
        #[cfg(test)]
        tests::record(ui, &format!("target:{target:?}"));
    }
    ui.separator();
    let scenes_menu = ui.begin_menu(format!(
        "Scenes: {}###play-scenes",
        content_label(profile.content)
    ));
    #[cfg(test)]
    tests::record(ui, "scenes");
    if let Some(_menu) = scenes_menu {
        content_menu(ui, profile, root);
    }
    ui.separator();
    ui.text_disabled("Data source");
    for source in DataSource::ALL {
        if ui
            .menu_item_config(source.label())
            .selected(profile.data == source)
            .enabled(profile.target != Target::Serial || source != DataSource::Disc)
            .build()
        {
            profile.data = source;
        }
        #[cfg(test)]
        tests::record(ui, &format!("data:{source:?}"));
    }
    ui.separator();
    if ui
        .menu_item_config("Analog controller (port 1)")
        .selected(profile.analog_controller)
        .enabled(profile.target != Target::Serial)
        .build()
    {
        profile.analog_controller = !profile.analog_controller;
    }
    if ui.is_item_hovered() {
        ui.tooltip_text("Enable both sticks on the first emulated pad. Physical pads use their own analog mode.");
    }
}

fn content_menu(ui: &imgui::Ui, profile: &mut Profile, root: &std::path::Path) {
    for content in [Content::CurrentScene, Content::WholeGame] {
        if ui
            .menu_item_config(content_label(content))
            .selected(profile.content == content)
            .build()
        {
            profile.content = content;
        }
    }
    let selected = profile.content == Content::SelectedScenes;
    let pos = ui.cursor_screen_pos();
    let parent_draw = ui.get_window_draw_list();
    let submenu = ui.begin_menu("Selected scenes    ###selected-scenes");
    #[cfg(test)]
    tests::record(ui, "selected");
    // The parent row is a choice as well as a submenu. Hover only opens it;
    // clicking selects the mode without closing the scene checklist.
    if ui.is_item_clicked() {
        profile.content = Content::SelectedScenes;
    }
    if profile.content == Content::SelectedScenes {
        let x = pos[0] + ui.calc_text_size("Selected scenes")[0] + 6.;
        let y = pos[1] + ui.text_line_height() * 0.5;
        parent_draw
            .add_line([x, y], [x + 3., y + 3.], [0.9, 0.9, 0.9, 1.])
            .thickness(1.5)
            .build();
        parent_draw
            .add_line([x + 3., y + 3.], [x + 8., y - 3.], [0.9, 0.9, 0.9, 1.])
            .thickness(1.5)
            .build();
    }
    drop(parent_draw);
    if let Some(_menu) = submenu {
        if ui.radio_button_bool("Use Selected scenes", selected) {
            profile.content = Content::SelectedScenes;
        }
        ui.separator();
        match crate::scene_bank::available(root) {
            Ok(mut paths) => {
                // Keep missing selections visible so they can be removed.
                for path in &profile.selected_scenes {
                    if !paths.contains(path) {
                        paths.push(path.clone());
                    }
                }
                if paths.is_empty() {
                    ui.text_disabled("No saved scenes in this project.");
                }
                if let Some(_table) = ui.begin_table_with_flags(
                    "scene-selection",
                    2,
                    imgui::TableFlags::SIZING_FIXED_FIT,
                ) {
                    ui.table_setup_column("Include");
                    ui.table_setup_column("Initial");
                    ui.table_headers_row();
                    for path in paths {
                        let _id = ui.push_id(&path);
                        ui.table_next_row();
                        ui.table_next_column();
                        let mut included = profile.selected_scenes.contains(&path);
                        let name = path.strip_prefix("assets/scenes/").unwrap_or(&path);
                        if ui.checkbox(format!("{name}##include"), &mut included) {
                            profile.set_scene_included(&path, included);
                        }
                        #[cfg(test)]
                        tests::record(ui, &format!("include:{path}"));
                        ui.table_next_column();
                        let _disabled = ui.begin_disabled(!included);
                        if ui.radio_button_bool(
                            "##initial",
                            profile.initial_scene.as_deref() == Some(&path),
                        ) {
                            profile.initial_scene = Some(path.clone());
                        }
                        #[cfg(test)]
                        tests::record(ui, &format!("initial:{path}"));
                        if ui.is_item_hovered() {
                            ui.tooltip_text("Initial scene to execute");
                        }
                    }
                }
                if profile.selected_scenes.is_empty() {
                    ui.text_colored(
                        [1., 0.75, 0.3, 1.],
                        "Select at least one scene before Build or Play.",
                    );
                }
            }
            Err(error) => {
                ui.text_wrapped(error);
            }
        }
    }
}

pub fn warning(ui: &imgui::Ui, e: &mut crate::editor::Editor) {
    if e.play_warning.is_some() && !crate::busy_ui::popup_open("Build / Play warning") {
        ui.open_popup("Build / Play warning");
    }
    ui.modal_popup_config("Build / Play warning")
        .always_auto_resize(true)
        .build(|| {
            if let Some(message) = &e.play_warning {
                ui.text_wrapped(message);
            }
            if ui.button("OK") {
                e.play_warning = None;
                ui.close_current_popup();
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    thread_local! {
        static ITEMS: std::cell::RefCell<std::collections::BTreeMap<String, [f32; 2]>> = Default::default();
    }
    pub(super) fn record(ui: &imgui::Ui, name: &str) {
        let a = ui.item_rect_min();
        let b = ui.item_rect_max();
        ITEMS.with_borrow_mut(|items| {
            items.insert(name.into(), [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5]);
        });
    }
    #[test]
    fn selected_parent_checkboxes_and_initial_radio_accept_real_mouse_input() {
        let root = crate::workspace::tests::temp("play-menu");
        std::fs::create_dir_all(root.join("assets/scenes")).unwrap();
        for name in ["A", "B"] {
            std::fs::write(root.join(format!("assets/scenes/{name}.epokmap")), "{}").unwrap();
        }
        let mut context = crate::gui::tests::imgui_context();
        context.set_ini_filename(None);
        context.io_mut().display_size = [1024., 768.];
        context.io_mut().delta_time = 0.1;
        context
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
        context.fonts().build_rgba32_texture();
        let mut profile = Profile {
            data: DataSource::Disc,
            ..Default::default()
        };
        let frame = |context: &mut imgui::Context, profile: &mut Profile, open: bool| {
            ITEMS.with_borrow_mut(|items| items.clear());
            let ui = context.frame();
            ui.window("Test")
                .position([20., 20.], imgui::Condition::Always)
                .size([600., 600.], imgui::Condition::Always)
                .build(|| {
                    if open {
                        ui.open_popup("options");
                    }
                    ui.popup("options", || menu(ui, profile, &root));
                });
            context.render();
        };
        frame(&mut context, &mut profile, true);
        for _ in 0..3 {
            frame(&mut context, &mut profile, false);
        }
        let click = |context: &mut imgui::Context, profile: &mut Profile, name: &str| {
            let pos = ITEMS.with_borrow(|items| {
                *items
                    .get(name)
                    .unwrap_or_else(|| panic!("Missing {name}: {items:?}"))
            });
            context.io_mut().add_mouse_pos_event(pos);
            frame(context, profile, false);
            context
                .io_mut()
                .add_mouse_button_event(imgui::MouseButton::Left, true);
            frame(context, profile, false);
            context
                .io_mut()
                .add_mouse_button_event(imgui::MouseButton::Left, false);
            frame(context, profile, false);
            for _ in 0..3 {
                frame(context, profile, false);
            }
        };
        click(&mut context, &mut profile, "target:Serial");
        assert_eq!(profile.data, DataSource::Host);
        frame(&mut context, &mut profile, true);
        for _ in 0..3 {
            frame(&mut context, &mut profile, false);
        }
        click(&mut context, &mut profile, "data:Disc");
        assert_eq!(
            profile.data,
            DataSource::Host,
            "CD is disabled while Serial is selected"
        );
        click(&mut context, &mut profile, "scenes");
        click(&mut context, &mut profile, "selected");
        assert_eq!(
            profile.content,
            Content::SelectedScenes,
            "Parent click must also select the mode"
        );
        click(
            &mut context,
            &mut profile,
            "include:assets/scenes/A.epokmap",
        );
        click(
            &mut context,
            &mut profile,
            "include:assets/scenes/B.epokmap",
        );
        click(
            &mut context,
            &mut profile,
            "initial:assets/scenes/B.epokmap",
        );
        assert_eq!(
            profile.initial_scene.as_deref(),
            Some("assets/scenes/B.epokmap")
        );
        click(
            &mut context,
            &mut profile,
            "include:assets/scenes/B.epokmap",
        );
        assert_eq!(
            profile.initial_scene.as_deref(),
            Some("assets/scenes/A.epokmap")
        );
        click(
            &mut context,
            &mut profile,
            "include:assets/scenes/A.epokmap",
        );
        assert!(profile.selected_scenes.is_empty() && profile.initial_scene.is_none());
        std::fs::remove_dir_all(root).unwrap();
    }
}
