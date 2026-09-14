//! A compact modal over a visible, inert workspace during critical operations.
pub fn draw(ui: &imgui::Ui, title: &str, details: &str, cancellable: bool) -> bool {
    progress_with_details(ui, title, None, cancellable, Some(details))
}

pub fn progress(ui: &imgui::Ui, title: &str, fraction: Option<f32>, cancellable: bool) -> bool {
    progress_with_details(ui, title, fraction, cancellable, None)
}

fn progress_with_details(
    ui: &imgui::Ui,
    title: &str,
    fraction: Option<f32>,
    cancellable: bool,
    details: Option<&str>,
) -> bool {
    let mut cancel = false;
    let name = "Working###CriticalOperation";
    if !popup_open(name) {
        ui.open_popup(name);
    }
    let [w, h] = ui.io().display_size;
    unsafe {
        imgui::sys::igSetNextWindowPos(
            [w * 0.5, h * 0.5].into(),
            imgui::sys::ImGuiCond_Always as i32,
            [0.5, 0.5].into(),
        );
        imgui::sys::igSetNextWindowSize(
            [420_f32.min((w - 32.).max(240.)), 0.].into(),
            imgui::sys::ImGuiCond_Always as i32,
        );
    }
    let _dim = ui.push_style_color(imgui::StyleColor::ModalWindowDimBg, [0., 0., 0., 0.62]);
    ui.modal_popup_config(name)
        .always_auto_resize(true)
        .movable(false)
        .resizable(false)
        .save_settings(false)
        .build(|| {
            ui.text_wrapped(title);
            ui.spacing();
            let width = ui.content_region_avail()[0];
            if let Some(fraction) = fraction.filter(|v| v.is_finite()) {
                imgui::ProgressBar::new(fraction.clamp(0., 1.))
                    .size([width, 18.])
                    .overlay_text(format!("{:.0}%", fraction.clamp(0., 1.) * 100.))
                    .build(ui);
            } else {
                // A moving segment denotes activity; it is never a fabricated percentage.
                let a = ui.cursor_screen_pos();
                let b = [a[0] + width, a[1] + 18.];
                let draw = ui.get_window_draw_list();
                draw.add_rect(a, b, ui.style_color(imgui::StyleColor::FrameBg))
                    .filled(true)
                    .build();
                let t = ((ui.time() * 0.65) % 1.) as f32;
                let x = a[0] + t * width * 0.75;
                draw.add_rect(
                    [x, a[1]],
                    [x + width * 0.25, b[1]],
                    ui.style_color(imgui::StyleColor::PlotHistogram),
                )
                .filled(true)
                .build();
                ui.dummy([width, 18.]);
            }
            if let Some(details) = details.filter(|text| !text.trim().is_empty()) {
                ui.separator();
                ui.text_disabled("Live installation output");
                ui.child_window("critical-operation-details")
                    .size([width, 150.])
                    .build(|| ui.text_wrapped(details));
            } else {
                ui.text_disabled("Please wait.");
            }
            if cancellable && ui.button("Cancel operation") {
                cancel = true;
            }
        });
    cancel
}

pub fn popup_open(name: &str) -> bool {
    let name = std::ffi::CString::new(name).expect("Static popup name");
    unsafe { imgui::sys::igIsPopupOpen_Str(name.as_ptr(), 0) }
}

/// End the popup explicitly when the worker finishes, before other dialogs open.
pub fn finish(ui: &imgui::Ui) {
    if let Some(_popup) = ui.begin_modal_popup("Working###CriticalOperation") {
        ui.close_current_popup();
    }
}

/// ImGui's disabled widgets do not suppress raw IsKeyPressed/IsMouseDragging
/// queries used by editor canvases. Mask those queries for background rendering
/// only, then restore input before drawing the modal. RAII also restores on unwind.
pub fn background(ui: &imgui::Ui, render: impl FnOnce()) {
    struct InputRestore(imgui::sys::ImGuiIO);
    impl Drop for InputRestore {
        fn drop(&mut self) {
            unsafe {
                let io = &mut *imgui::sys::igGetIO();
                macro_rules! restore { ($($field:ident),*) => { $(io.$field = self.0.$field;)* }; }
                restore!(
                    KeysData,
                    KeysDown,
                    NavInputs,
                    KeyCtrl,
                    KeyShift,
                    KeyAlt,
                    KeySuper,
                    KeyMods,
                    MouseDown,
                    MouseClicked,
                    MouseReleased,
                    MouseDoubleClicked,
                    MouseClickedCount,
                    MousePos,
                    MouseDelta,
                    MouseWheel,
                    MouseWheelH,
                    InputQueueCharacters
                );
            }
        }
    }
    let _restore = unsafe {
        let io = &mut *imgui::sys::igGetIO();
        let saved = InputRestore(*io);
        for key in &mut io.KeysData {
            key.Down = false;
            key.DownDuration = -1.;
            key.DownDurationPrev = -1.;
            key.AnalogValue = 0.;
        }
        io.KeysDown.fill(false);
        io.NavInputs.fill(0.);
        io.KeyCtrl = false;
        io.KeyShift = false;
        io.KeyAlt = false;
        io.KeySuper = false;
        io.KeyMods = 0;
        io.MouseDown.fill(false);
        io.MouseClicked.fill(false);
        io.MouseReleased.fill(false);
        io.MouseDoubleClicked.fill(false);
        io.MouseClickedCount.fill(0);
        io.MousePos = [-f32::MAX, -f32::MAX].into();
        io.MouseDelta = [0., 0.].into();
        io.MouseWheel = 0.;
        io.MouseWheelH = 0.;
        io.InputQueueCharacters.Size = 0;
        saved
    };
    let _disabled = ui.begin_disabled(true);
    render();
}

pub fn editor(ui: &imgui::Ui, e: &mut crate::editor::Editor) {
    // ImGui needs the dockspace submitted even when its editing windows are hidden.
    // Keep the user's panel layout alive throughout long builds/installations.
    ui.window("Workspace")
        .flags(
            imgui::WindowFlags::NO_DECORATION
                | imgui::WindowFlags::NO_INPUTS
                | imgui::WindowFlags::NO_BACKGROUND
                | imgui::WindowFlags::NO_DOCKING
                | imgui::WindowFlags::NO_SAVED_SETTINGS,
        )
        .build(|| unsafe {
            let id = imgui::sys::igGetID_Str(c"EpokDock".as_ptr());
            let node = imgui::sys::igDockBuilderGetNode(id);
            if node.is_null() || (*node).LastFrameAlive != imgui::sys::igGetFrameCount() {
                imgui::sys::igDockSpace(
                    id,
                    imgui::sys::ImVec2 { x: 0., y: 0. },
                    imgui::sys::ImGuiDockNodeFlags_KeepAliveOnly as i32,
                    std::ptr::null(),
                );
            }
        });
    e.scene_navigation = false;
    e.scene_look = false;
    e.raw_look = None;
    e.drag_axis = None;
    e.hud_drag = None;
    e.scene_click = Default::default();
    e.game_capture = false;
    if e.dependencies.busy() {
        draw(
            ui,
            "Installing components",
            &e.dependencies.progress(),
            false,
        );
    } else if let Some(loading) = &e.scene_loading {
        progress(ui, &loading.stage, None, false);
    } else {
        if progress(ui, &e.job_stage, e.job_progress, true) {
            e.action("play");
            e.job_stage = "Cancelling operation...".into();
            e.job_progress = None;
        }
    }
}

#[cfg(test)]
pub fn verify_interactions(context: &mut imgui::Context) {
    use crate::{
        editor::Editor,
        pipeline::{Control, Event, Job},
    };
    let root = std::env::temp_dir().join(format!("epok-critical-ui-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&root).unwrap();
    let mut e = Editor::new(root.clone());
    e.auto_build = false;
    e.dependencies.warning = false;
    e.selected = Some(0);
    e.dirty = true;
    let before = crate::mcp_tools::revision(&e.scene);
    let font = context.fonts().fonts()[0];
    let mut initial = true;
    let frame = |context: &mut imgui::Context, e: &mut Editor, initial: &mut bool| {
        crate::gui::draw(
            context.frame(),
            e,
            [imgui::TextureId::new(999); 3],
            None,
            font,
            [960., 600.],
            initial,
        );
        if !e.critical_busy() {
            assert!(!popup_open("Working###CriticalOperation"));
        }
        context.render();
    };
    frame(context, &mut e, &mut initial);
    frame(context, &mut e, &mut initial);
    let dock_id = || unsafe {
        (*imgui::sys::igFindWindowByName(c"\u{eb86} Hierarchy###Hierarchy".as_ptr())).DockId
    };
    let dock = dock_id();
    assert_ne!(dock, 0);
    let (job, events, controls) = Job::test_channels();
    e.job = Some(job);
    e.job_stage = "Compiling test project".into();
    frame(context, &mut e, &mut initial);
    frame(context, &mut e, &mut initial);
    unsafe {
        let popup = imgui::sys::igFindWindowByName(c"Working###CriticalOperation".as_ptr());
        assert!(!popup.is_null());
        assert!(
            (*popup).Size.x <= 440. && (*popup).Size.y < 220.,
            "Progress must be a compact modal"
        );
        assert!((*popup).Flags & imgui::sys::ImGuiWindowFlags_Modal as i32 != 0);
        let hierarchy = imgui::sys::igFindWindowByName(c"\u{eb86} Hierarchy###Hierarchy".as_ptr());
        assert_eq!(
            (*hierarchy).LastFrameActive,
            imgui::sys::igGetFrameCount(),
            "The blocked workspace must remain visible"
        );
    }
    context.io_mut().add_key_event(imgui::Key::ModCtrl, true);
    context.io_mut().add_key_event(imgui::Key::D, true);
    context.io_mut().add_key_event(imgui::Key::S, true);
    context.io_mut().add_key_event(imgui::Key::Delete, true);
    frame(context, &mut e, &mut initial);
    e.action("duplicate");
    e.action("delete");
    assert!(!e.save_all());
    assert_eq!(crate::mcp_tools::revision(&e.scene), before);
    assert!(e.dirty);
    assert!(!root.join("assets/scenes/SampleScene.epokmap").exists());
    let mut mcp = crate::mcp::State::default();
    assert!(
        crate::mcp_tools::execute(
            &mut e,
            &mut mcp,
            "editor_control",
            serde_json::json!({"action":"build"})
        )
        .unwrap_err()
        .contains("locked")
    );
    assert!(
        crate::mcp_tools::execute(&mut e, &mut mcp, "logs_read", serde_json::json!({})).is_ok()
    );
    crate::mcp_tools::execute(
        &mut e,
        &mut mcp,
        "editor_control",
        serde_json::json!({"action":"stop"}),
    )
    .unwrap();
    assert!(matches!(controls.try_recv(), Ok(Control::Stop)));
    for key in [
        imgui::Key::ModCtrl,
        imgui::Key::D,
        imgui::Key::S,
        imgui::Key::Delete,
    ] {
        context.io_mut().add_key_event(key, false);
    }
    frame(context, &mut e, &mut initial);
    events
        .send(Event::Finished(Err("Cancelled test operation".into())))
        .unwrap();
    e.tick();
    assert!(!e.critical_busy());
    assert_eq!(crate::mcp_tools::revision(&e.scene), before);
    let count = e.scene.actors.len();
    e.action("duplicate");
    assert_eq!(e.scene.actors.len(), count + 1);
    frame(context, &mut e, &mut initial);
    assert_eq!(
        dock_id(),
        dock,
        "Critical operations must preserve the panel layout"
    );
    drop(e);
    std::fs::remove_dir_all(root).unwrap();
}
