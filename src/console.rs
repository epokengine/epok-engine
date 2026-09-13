//! Selectable, read-only console output. Rebuild the text only when logs change.
use crate::editor::Editor;
use imgui::{InputTextCallbackHandler, InputTextMultilineCallback, TextCallbackData, Ui};
use std::path::{Path, PathBuf};

pub struct State {
    snapshot: Vec<(String, String)>,
    text: String,
    cursor: usize,
    pub auto_scroll: bool,
    follow_frames: u8,
    last_scroll: Option<f32>,
    #[cfg(test)]
    bounds: [[f32; 2]; 2],
    #[cfg(test)]
    scroll_bounds: [f32; 2],
}
impl Default for State {
    fn default() -> Self {
        Self {
            snapshot: Vec::new(),
            text: String::new(),
            cursor: 0,
            auto_scroll: true,
            follow_frames: 0,
            last_scroll: None,
            #[cfg(test)]
            bounds: [[0.; 2]; 2],
            #[cfg(test)]
            scroll_bounds: [0.; 2],
        }
    }
}
impl State {
    pub fn force_follow(&mut self) {
        self.auto_scroll = true;
        self.follow_frames = 2;
    }
    fn sync(&mut self, logs: &[String], times: &[String]) {
        debug_assert_eq!(logs.len(), times.len());
        let snapshot = logs
            .iter()
            .zip(times)
            .map(|(line, time)| (line.clone(), time.clone()))
            .collect::<Vec<_>>();
        if self.snapshot != snapshot {
            self.text = snapshot
                .iter()
                .flat_map(|(message, time)| {
                    message
                        .split('\n')
                        .map(move |line| format!("[{time}] {}", line.trim_end_matches('\r')))
                })
                .collect::<Vec<_>>()
                .join("\n")
                .replace('\0', "");
            self.snapshot = snapshot;
            self.cursor = self.cursor.min(self.text.len());
            if self.auto_scroll {
                self.follow_frames = 2;
            }
        }
    }
    fn source(&self, root: &Path) -> Option<(PathBuf, usize)> {
        let before = self.text.get(..self.cursor)?;
        let start = before.rfind('\n').map_or(0, |i| i + 1);
        diagnostic(root, self.text[start..].split('\n').next()?)
    }
}

#[cfg(test)]
#[test]
fn every_line_of_a_multiline_message_keeps_its_timestamp() {
    let mut state = State::default();
    state.sync(
        &["first\r\nsecond".into()],
        &["2026-09-12 13:45:06.789".into()],
    );
    assert_eq!(
        state.text,
        "[2026-09-12 13:45:06.789] first\n[2026-09-12 13:45:06.789] second"
    );
}

struct Cursor<'a>(&'a mut usize);
impl InputTextCallbackHandler for Cursor<'_> {
    fn on_always(&mut self, data: TextCallbackData) {
        *self.0 = data.cursor_pos();
    }
}
pub fn draw(ui: &Ui, e: &mut Editor) {
    let forced = e.critical_busy() || e.assets.busy || e.bake_job.is_some();
    if forced {
        e.console.force_follow();
    }
    ui.window("\u{eb9b} Console###Console").build(|| {
        if ui.small_button("Clear") {
            e.clear_logs();
        }
        e.reconcile_log_times();
        e.console.sync(&e.logs, &e.log_times);
        ui.same_line();
        let source = e.console.source(&e.root);
        ui.disabled(source.is_none(), || {
            if ui.small_button("Open source")
                && let Some((path, line)) = &source
            {
                e.open_code(path, Some(*line));
            }
        });
        ui.same_line();
        ui.disabled(forced, || {
            if ui.checkbox("Auto-scroll", &mut e.console.auto_scroll) {
                e.console.follow_frames = if e.console.auto_scroll { 2 } else { 0 };
            }
        });
        ui.same_line();
        ui.text_disabled("Select text / Ctrl+C to copy");
        ui.separator();
        let size = ui.content_region_avail().map(|v| v.max(1.));
        let _background = ui.push_style_color(imgui::StyleColor::FrameBg, crate::gui::gray(28));
        ui.input_text_multiline("##console-output", &mut e.console.text, size)
            .read_only(true)
            .callback(
                InputTextMultilineCallback::ALWAYS,
                Cursor(&mut e.console.cursor),
            )
            .build();
        // Scroll the multiline input's child, without moving its text cursor or
        // selection. This keeps Ctrl+C and diagnostic navigation intact.
        unsafe {
            let child = output_window();
            if !child.is_null() {
                let scroll = (*child).Scroll.y;
                let maximum = (*child).ScrollMax.y;
                let wheel = ui.is_item_hovered() && ui.io().mouse_wheel != 0.;
                let moved = e
                    .console
                    .last_scroll
                    .is_some_and(|old| (old - scroll).abs() > 0.5);
                let key_scroll = ui.is_item_active()
                    && [
                        imgui::Key::PageUp,
                        imgui::Key::PageDown,
                        imgui::Key::Home,
                        imgui::Key::End,
                        imgui::Key::UpArrow,
                        imgui::Key::DownArrow,
                    ]
                    .iter()
                    .any(|key| ui.is_key_pressed(*key));
                let manual =
                    wheel || (moved && (ui.is_mouse_down(imgui::MouseButton::Left) || key_scroll));
                if manual && !forced {
                    if scroll >= maximum - 1. && ui.io().mouse_wheel <= 0. {
                        e.console.auto_scroll = true;
                    } else {
                        e.console.auto_scroll = false;
                        e.console.follow_frames = 0;
                    }
                }
                if e.console.auto_scroll && e.console.follow_frames > 0 {
                    imgui::sys::igSetScrollY_WindowPtr(child, 1.0e9);
                    e.console.follow_frames -= 1;
                }
                e.console.last_scroll = Some(scroll);
                #[cfg(test)]
                {
                    e.console.scroll_bounds = [scroll, maximum];
                }
            }
        }
        #[cfg(test)]
        {
            e.console.bounds = [ui.item_rect_min(), ui.item_rect_max()];
        }
    });
}

/// ImGui 0.12's InputTextMultiline uses BeginChildEx(label, id, ...).
unsafe fn output_window() -> *mut imgui::sys::ImGuiWindow {
    unsafe {
        let parent = imgui::sys::igGetCurrentWindow();
        let name = std::ffi::CStr::from_ptr((*parent).Name).to_string_lossy();
        let id = imgui::sys::igGetID_Str(c"##console-output".as_ptr());
        let name = std::ffi::CString::new(format!("{name}/##console-output_{id:08X}")).unwrap();
        imgui::sys::igFindWindowByName(name.as_ptr())
    }
}

fn diagnostic(root: &Path, line: &str) -> Option<(PathBuf, usize)> {
    let start = line.find("scripts/").or_else(|| line.find("scripts\\"))? + 8;
    let (file, tail) = line[start..].split_once(':')?;
    if file.contains(['/', '\\']) || !file.ends_with(".cpp") && !file.ends_with(".hpp") {
        return None;
    }
    Some((
        root.join("assets/scripts").join(file),
        tail.split(':').next()?.parse().ok()?,
    ))
}

#[cfg(test)]
#[test]
fn autoscroll_follows_output_yields_to_scroll_and_is_forced_by_builds() {
    let mut ctx = crate::gui::tests::imgui_context();
    ctx.set_ini_filename(None);
    ctx.io_mut().display_size = [1024., 768.];
    ctx.io_mut().delta_time = 1. / 60.;
    ctx.fonts()
        .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
    ctx.fonts().build_rgba32_texture();
    let root = crate::workspace::tests::temp("console-follow");
    let mut e = Editor::new(root.clone());
    let frame = |ctx: &mut imgui::Context, e: &mut Editor| {
        let ui = ctx.frame();
        unsafe {
            imgui::sys::igSetNextWindowPos(
                [20., 20.].into(),
                imgui::Condition::Always as i32,
                [0., 0.].into(),
            );
            imgui::sys::igSetNextWindowSize([800., 300.].into(), imgui::Condition::Always as i32);
        }
        draw(ui, e);
        ctx.render();
    };
    for i in 0..100 {
        e.log(format!("Message {i}"));
    }
    for _ in 0..4 {
        frame(&mut ctx, &mut e);
    }
    assert!(e.console.auto_scroll);
    assert!(e.console.scroll_bounds[1] > 500.);
    assert!(
        e.console.scroll_bounds[0] >= e.console.scroll_bounds[1] - 1.,
        "{:?}",
        e.console.scroll_bounds
    );
    let bounds = e.console.bounds;
    ctx.io_mut().add_mouse_pos_event([
        (bounds[0][0] + bounds[1][0]) * 0.5,
        (bounds[0][1] + bounds[1][1]) * 0.5,
    ]);
    frame(&mut ctx, &mut e);
    ctx.io_mut().add_mouse_wheel_event([0., 5.]);
    for _ in 0..3 {
        frame(&mut ctx, &mut e);
    }
    assert!(!e.console.auto_scroll);
    let position = e.console.scroll_bounds[0];
    e.log("Output while reading older lines");
    for _ in 0..3 {
        frame(&mut ctx, &mut e);
    }
    assert!((e.console.scroll_bounds[0] - position).abs() < 1.);
    ctx.io_mut().add_mouse_wheel_event([0., -1000.]);
    for _ in 0..3 {
        frame(&mut ctx, &mut e);
    }
    assert!(e.console.auto_scroll, "Reaching the bottom enables follow");
    e.log("Follow this new output");
    for _ in 0..3 {
        frame(&mut ctx, &mut e);
    }
    assert!(e.console.scroll_bounds[0] >= e.console.scroll_bounds[1] - 1.);
    e.console.auto_scroll = false;
    e.console.follow_frames = 0;
    e.log("Must not follow when manually unchecked");
    for _ in 0..3 {
        frame(&mut ctx, &mut e);
    }
    assert!(!e.console.auto_scroll);
    let (job, _events, _controls) = crate::pipeline::Job::test_channels();
    e.job = Some(job);
    for _ in 0..3 {
        frame(&mut ctx, &mut e);
    }
    assert!(e.console.auto_scroll && e.console.scroll_bounds[0] >= e.console.scroll_bounds[1] - 1.);
    drop(e);
    if root.exists() {
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[cfg(test)]
#[test]
fn autoscroll_preserves_read_only_selection_and_copy() {
    let mut ctx = crate::gui::tests::imgui_context();
    crate::gui::configure_input(ctx.io_mut());
    ctx.set_ini_filename(None);
    ctx.io_mut().display_size = [1024., 768.];
    ctx.io_mut().delta_time = 1. / 60.;
    ctx.fonts()
        .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
    ctx.fonts().build_rgba32_texture();
    verify_interactions(&mut ctx);
}

#[cfg(test)]
pub fn verify_interactions(ctx: &mut imgui::Context) {
    use imgui::{Condition, Key, MouseButton};
    use std::{cell::RefCell, rc::Rc};
    struct Clipboard(Rc<RefCell<String>>);
    impl imgui::ClipboardBackend for Clipboard {
        fn get(&mut self) -> Option<String> {
            Some(self.0.borrow().clone())
        }
        fn set(&mut self, value: &str) {
            *self.0.borrow_mut() = value.into();
        }
    }
    let copied = Rc::new(RefCell::new(String::new()));
    ctx.set_clipboard_backend(Clipboard(copied.clone()));
    let mut e = Editor::new(std::env::temp_dir().join("epok-console-selection"));
    e.logs = vec![
        "alpha one".into(),
        "beta two".into(),
        "scripts/Spinner.cpp:42: error: diagnóstico".into(),
    ];
    let original = e.logs.clone();
    fn frame(ctx: &mut imgui::Context, e: &mut Editor) {
        let ui = ctx.frame();
        unsafe {
            imgui::sys::igSetNextWindowDockID(0, Condition::Always as i32);
            imgui::sys::igSetNextWindowPos(
                imgui::sys::ImVec2 { x: 20., y: 20. },
                Condition::Always as i32,
                imgui::sys::ImVec2 { x: 0., y: 0. },
            );
            imgui::sys::igSetNextWindowSize(
                imgui::sys::ImVec2 { x: 650., y: 300. },
                Condition::Always as i32,
            );
        }
        draw(ui, e);
        if unsafe { imgui::sys::igGetActiveID() } != 0 {
            assert!(
                crate::gui::text_input_active(ui),
                "Read-only selection must block scene editing shortcuts"
            );
        }
        ctx.render();
    }
    fn key(ctx: &mut imgui::Context, e: &mut Editor, key: Key) {
        ctx.io_mut().add_key_event(key, true);
        frame(ctx, e);
        ctx.io_mut().add_key_event(key, false);
        frame(ctx, e);
    }
    frame(ctx, &mut e);
    frame(ctx, &mut e);
    assert!(
        e.console
            .text
            .lines()
            .all(|line| line.starts_with("[20") && line.get(24..26) == Some("] ")),
        "Every Console line starts with a millisecond timestamp: {:?}",
        e.console.text
    );
    let origin = e.console.bounds[0];
    let start = [origin[0] + 22., origin[1] + 9.];
    let end = [origin[0] + 39., origin[1] + 22.];
    ctx.io_mut().add_mouse_pos_event(start);
    frame(ctx, &mut e);
    ctx.io_mut().add_mouse_button_event(MouseButton::Left, true);
    frame(ctx, &mut e);
    ctx.io_mut().add_mouse_pos_event(end);
    frame(ctx, &mut e);
    ctx.io_mut()
        .add_mouse_button_event(MouseButton::Left, false);
    frame(ctx, &mut e);
    ctx.io_mut().add_key_event(Key::ModCtrl, true);
    key(ctx, &mut e, Key::C);
    ctx.io_mut().add_key_event(Key::ModCtrl, false);
    frame(ctx, &mut e);
    let selection = copied.borrow().clone();
    assert!(
        selection.contains('\n'),
        "Mouse selection crosses lines: {selection:?}"
    );
    assert!(
        !selection.starts_with("alpha"),
        "Selection starts inside a line"
    );
    e.log("new output");
    frame(ctx, &mut e);
    ctx.io_mut().add_key_event(Key::ModCtrl, true);
    key(ctx, &mut e, Key::C);
    assert_eq!(
        *copied.borrow(),
        selection,
        "Appending output keeps the selection"
    );
    key(ctx, &mut e, Key::A);
    key(ctx, &mut e, Key::C);
    assert_eq!(*copied.borrow(), e.console.text);
    let output = e.console.text.clone();
    *copied.borrow_mut() = "replacement".into();
    key(ctx, &mut e, Key::V);
    ctx.io_mut().add_key_event(Key::ModCtrl, false);
    key(ctx, &mut e, Key::Delete);
    ctx.io_mut().add_input_character('X');
    frame(ctx, &mut e);
    assert_eq!(
        e.console.text, output,
        "Typing, paste and delete cannot edit output"
    );
    assert_eq!(&e.logs[..3], original);
    ctx.io_mut().add_key_event(Key::ModCtrl, true);
    key(ctx, &mut e, Key::Home);
    ctx.io_mut().add_key_event(Key::ModCtrl, false);
    frame(ctx, &mut e);
    key(ctx, &mut e, Key::DownArrow);
    key(ctx, &mut e, Key::DownArrow);
    assert_eq!(
        e.console.source(&e.root),
        Some((e.root.join("assets/scripts/Spinner.cpp"), 42))
    );
    e.clear_logs();
    frame(ctx, &mut e);
    assert!(e.console.text.is_empty());
    assert!(e.console.source(&e.root).is_none());
    key(ctx, &mut e, Key::Escape);
}
