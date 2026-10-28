use crate::{editor::Editor, play::Content, ps_exe_export::Options};
use imgui::{Condition, StyleColor as C, StyleVar as V};
use std::path::PathBuf;

#[derive(Default)]
pub struct State {
    pub open: bool,
    options: Option<Options>,
    destination: String,
    scenes: Vec<String>,
    streaming_geometry: bool,
    /// Scenes the project's scene table loads from disc.
    external_scenes: usize,
    disc_media: Vec<String>,
    message: String,
    /// Set while this dialog's job runs. The editor's busy overlay shows its
    /// progress and Cancel; this window reports the outcome afterwards.
    exporting: bool,
    result: Option<PathBuf>,
}

pub fn open(editor: &mut Editor) {
    let mut state = std::mem::take(&mut editor.ps_exe_export_ui);
    state.message.clear();
    if state.options.is_none() {
        match Options::for_project(&editor.root) {
            Ok(options) => state.options = Some(options),
            Err(error) => state.message = error,
        }
    }
    if let Some(options) = &state.options {
        state.destination = options.destination.to_string_lossy().into_owned();
    }
    state.scenes = crate::scene_bank::available(&editor.root).unwrap_or_default();
    state.streaming_geometry = crate::settings::rendering(&editor.root)
        .map(|r| r.streaming_geometry)
        .unwrap_or_default();
    let storage = crate::workspace::optional_manifest(&editor.root)
        .ok()
        .flatten()
        .and_then(|m| m.scene_storage);
    state.external_scenes = storage.map_or(0, |storage| {
        state
            .scenes
            .iter()
            .filter(|path| !storage.contains(path))
            .count()
    });
    state.disc_media = disc_media(&editor.assets.index);
    if !state.exporting {
        state.result = None;
    }
    state.open = true;
    editor.ps_exe_export_ui = state;
}

/// The worker published a package. Show it even if the window was closed.
pub fn published(editor: &mut Editor, path: PathBuf) {
    let state = &mut editor.ps_exe_export_ui;
    state.result = Some(path);
    state.message.clear();
    state.open = true;
}

/// Project assets that a PS-EXE package cannot carry, named for the summary.
fn disc_media(index: &crate::assets::Index) -> Vec<String> {
    let mut found = index
        .assets
        .values()
        .filter_map(|records| records.first())
        .filter_map(|record| match record.meta.kind {
            crate::assets::Kind::Video => Some(format!("{} (video)", record.meta.source)),
            crate::assets::Kind::AudioClip
                if record
                    .meta
                    .settings
                    .audio()
                    .is_ok_and(|settings| settings.is_streamed()) =>
            {
                Some(format!("{} (XA music)", record.meta.source))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    found.sort();
    found
}

#[cfg(windows)]
fn pick_zip(current: &str) -> Result<Option<PathBuf>, String> {
    let current = std::path::Path::new(current);
    let mut command = std::process::Command::new("powershell.exe");
    // Paths travel through the environment, never through the script text.
    command
        .env(
            "EPOK_ZIP_DIRECTORY",
            current.parent().unwrap_or(std::path::Path::new("")),
        )
        .env("EPOK_ZIP_NAME", current.file_name().unwrap_or_default())
        .args(["-NoProfile", "-STA", "-Command", "Add-Type -AssemblyName System.Windows.Forms; [Console]::OutputEncoding = [System.Text.UTF8Encoding]::new(); $dialog = New-Object System.Windows.Forms.SaveFileDialog; $dialog.Title = 'Export PS-EXE'; $dialog.Filter = 'ZIP archive (*.zip)|*.zip'; $dialog.DefaultExt = 'zip'; $dialog.AddExtension = $true; $dialog.OverwritePrompt = $true; if (Test-Path -LiteralPath $env:EPOK_ZIP_DIRECTORY -PathType Container) { $dialog.InitialDirectory = $env:EPOK_ZIP_DIRECTORY }; $dialog.FileName = $env:EPOK_ZIP_NAME; if ($dialog.ShowDialog() -eq 'OK') { [Console]::Write($dialog.FileName) }; $dialog.Dispose()"]);
    crate::pipeline::quiet(&mut command);
    let output = command.output().map_err(|e| format!("File dialog: {e}"))?;
    if !output.status.success() {
        return Err("Could not open the file dialog. Enter the destination path instead.".into());
    }
    let path = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
    Ok((!path.is_empty()).then(|| PathBuf::from(path)))
}
#[cfg(not(windows))]
fn pick_zip(_current: &str) -> Result<Option<PathBuf>, String> {
    Err("Enter the destination path; the native file dialog currently supports Windows.".into())
}

fn section(ui: &imgui::Ui, title: &str, body: impl FnOnce()) {
    ui.spacing();
    let _header = ui.push_style_color(C::Header, [0.06, 0.18, 0.30, 1.0]);
    if ui.collapsing_header(title, imgui::TreeNodeFlags::DEFAULT_OPEN) {
        body();
    }
}

const READY: [f32; 4] = [0.38, 0.82, 0.52, 1.];
const WARNING: [f32; 4] = [1., 0.72, 0.30, 1.];
const ERROR: [f32; 4] = [1., 0.48, 0.36, 1.];

pub fn window(ui: &imgui::Ui, editor: &mut Editor) {
    let mut state = std::mem::take(&mut editor.ps_exe_export_ui);
    if state.exporting && editor.job.is_none() {
        state.exporting = false;
        if state.result.is_none() {
            state.message = editor
                .last_error
                .clone()
                .unwrap_or_else(|| "Export stopped. See Console for details.".into());
        }
    }
    if !state.open {
        editor.ps_exe_export_ui = state;
        return;
    }
    let mut open = true;
    let mut close_requested = false;
    let mut start = false;
    let mut save = false;
    let mut reveal = None;
    let screen = ui.io().display_size;
    let _vars = [
        V::WindowPadding([18., 16.]),
        V::FramePadding([9., 7.]),
        V::ItemSpacing([9., 9.]),
        V::WindowRounding(4.),
        V::FrameRounding(2.),
    ]
    .map(|value| ui.push_style_var(value));
    let unsaved = editor.has_unsaved_changes();
    let busy = editor.job.is_some();
    ui.window("Export PS-EXE")
        .opened(&mut open)
        .position([screen[0] * 0.5, screen[1] * 0.5], Condition::Appearing)
        .position_pivot([0.5, 0.5])
        .size([780., 680.], Condition::FirstUseEver)
        .size_constraints([620., 480.], [1100., 1100.])
        .build(|| {
            ui.text_colored([0.42, 0.68, 0.96, 1.], "PLATFORMS  /  PLAYSTATION");
            ui.text("Export PS-EXE");
            ui.text_wrapped("Create a ZIP containing a PlayStation executable and the files required to run it through a PCDrv-compatible loader.");
            ui.separator();
            let Some(options) = state.options.as_mut() else {
                ui.text_colored(ERROR, &state.message);
                return;
            };
            let _locked = ui.begin_disabled(state.exporting);

            section(ui, "Content", || {
                for (content, label) in [
                    (Content::WholeGame, "Whole Game"),
                    (Content::CurrentScene, "Current Scene"),
                    (Content::SelectedScenes, "Selected Scenes"),
                ] {
                    if ui.radio_button_bool(label, options.content == content) {
                        options.content = content;
                    }
                    ui.same_line();
                }
                ui.new_line();
                match options.content {
                    Content::WholeGame => ui.text_disabled("Starts at the project's startup scene and includes every scene the game can load."),
                    Content::CurrentScene => ui.text_disabled(format!("Only {}, as saved.", editor.scene_path().file_name().unwrap_or_default().to_string_lossy())),
                    Content::SelectedScenes => {
                        ui.text_disabled("Up to 16 scenes. Transitions to other scenes are not available.");
                        for path in &state.scenes {
                            let mut included = options.selected_scenes.contains(path);
                            if ui.checkbox(path, &mut included) {
                                if included {
                                    options.selected_scenes.push(path.clone());
                                    options.selected_scenes.sort();
                                } else {
                                    options.selected_scenes.retain(|p| p != path);
                                }
                            }
                        }
                        let profile = options.profile();
                        options.initial_scene = profile.initial_scene;
                        let initial = options.initial_scene.clone().unwrap_or_default();
                        ui.set_next_item_width(-1.);
                        let preview = if initial.is_empty() { "Select scenes first" } else { initial.as_str() };
                        if let Some(_combo) = ui.begin_combo("Initial scene", preview) {
                            for path in &options.selected_scenes {
                                if ui.selectable_config(path).selected(*path == initial).build() {
                                    options.initial_scene = Some(path.clone());
                                }
                            }
                        }
                    }
                }
            });

            section(ui, "External data", || {
                ui.text_wrapped("Files the game reads while it runs are served through PCDrv from the folder the ZIP is extracted to. No CD is used.");
                ui.text("Runtime files");
                ui.same_line();
                let mut files = vec!["epok.ps-exe".to_owned()];
                if state.streaming_geometry {
                    files.push("GEOMETRY.BIN (Geometry Streaming is enabled)".into());
                }
                if state.external_scenes > 0 {
                    files.push(format!(
                        "one SCENEnn.BIN per exported scene outside the EXE (up to {})",
                        state.external_scenes
                    ));
                }
                ui.text_disabled(if files.len() == 1 {
                    "epok.ps-exe only (fully resident; no PCDrv reads)".to_owned()
                } else {
                    files.join(" + ")
                });
                ui.checkbox("Include debug symbols (epok.elf, epok.map)", &mut options.debug_symbols);
                if ui.is_item_hovered() {
                    ui.tooltip_text("Stored under debug/ in the ZIP. They are not needed to run the game.");
                }
                ui.checkbox("Include debug overlay (Select shows or hides it)", &mut options.debug_overlay);
                if ui.is_item_hovered() {
                    ui.tooltip_text("Compiles the FPS, CPU, GTE, GPU and SPU meters into this package only. They start hidden; Select shows or hides them, and the game still receives Select. Project debug settings and Play are unchanged.");
                }
            });

            section(ui, "Compatibility", || {
                ui.text_wrapped("The package uses the hardware runtime: PCDrv host data and the resident loader handler preserved after PsyQo starts. Your Play profile and serial settings are not changed.");
                if state.disc_media.is_empty() {
                    ui.text_colored(READY, "No XA music or video assets in this project.");
                } else {
                    ui.text_colored(WARNING, "XA music and video need the PlayStation CD decoder. The export stops if the exported content uses:");
                    for name in &state.disc_media {
                        ui.bullet_text(name);
                    }
                }
            });

            section(ui, "Destination", || {
                ui.set_next_item_width((ui.content_region_avail()[0] - 110.).max(120.));
                if ui.input_text("##ps-exe-destination", &mut state.destination)
                    .hint("Path to the .zip file")
                    .build()
                {
                    // Relative paths name a location inside the project.
                    options.destination = editor.root.join(state.destination.trim());
                }
                ui.same_line();
                if ui.button_with_size("Browse...", [100., 0.]) {
                    match pick_zip(&state.destination) {
                        Ok(Some(path)) => {
                            state.destination = path.to_string_lossy().into_owned();
                            options.destination = path;
                        }
                        Ok(None) => {}
                        Err(error) => state.message = error,
                    }
                }
                if options.destination.is_file() {
                    ui.text_colored(WARNING, "The existing file is replaced only after the new package is verified.");
                }
            });
            drop(_locked);

            let validation = options.validate(&editor.root);
            ui.spacing();
            ui.separator();
            if let Err(error) = &validation {
                ui.text_colored(ERROR, error);
            }
            if let Some(path) = &state.result {
                ui.text_colored(READY, "Package ready");
                ui.text_wrapped(path.display().to_string());
                if ui.button("Open Folder") {
                    reveal = Some(path.clone());
                }
                #[cfg(test)]
                crate::gui::record_script_control(ui, "Open PS-EXE folder");
            } else if unsaved {
                ui.text_colored(WARNING, "Save all changes first. Packages are built from saved project files.");
                if ui.button("Save All") {
                    save = true;
                }
            }
            if !state.message.is_empty() {
                ui.text_colored(ERROR, &state.message);
            }
            ui.same_line_with_pos((ui.content_region_avail()[0] - 258.).max(0.));
            if ui.button("Close") {
                close_requested = true;
            }
            ui.same_line();
            let valid = !busy && !unsaved && validation.is_ok();
            let _disabled = ui.begin_disabled(!valid);
            let _primary = ui.push_style_color(C::Button, [0.035, 0.35, 0.65, 1.]);
            if ui.button_with_size("Export", [150., 0.]) {
                start = true;
            }
            #[cfg(test)]
            crate::gui::record_script_control(ui, "Export PS-EXE package");
            if !valid && ui.is_item_hovered() {
                ui.tooltip_text(if busy {
                    "Wait for the active build to finish."
                } else if unsaved {
                    "Save all changes first."
                } else {
                    "Correct the destination or content selection."
                });
            }
        });
    if save {
        editor.save_all();
    }
    if let Some(path) = reveal {
        crate::project_browser::reveal(&path);
    }
    if start && let Some(options) = state.options.clone() {
        match editor.export_ps_exe(options) {
            Ok(()) => {
                state.exporting = true;
                state.result = None;
                state.message.clear();
            }
            Err(error) => state.message = error,
        }
    }
    if close_requested {
        open = false;
    }
    state.open = open;
    editor.ps_exe_export_ui = state;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn frame(
        context: &mut imgui::Context,
        editor: &mut Editor,
        font: imgui::FontId,
        initial: &mut bool,
    ) {
        crate::gui::clear_script_controls();
        crate::gui::draw(
            context.frame(),
            editor,
            [imgui::TextureId::new(999); 3],
            None,
            None,
            font,
            [960., 600.],
            initial,
        );
        context.render();
    }
    fn click(
        context: &mut imgui::Context,
        editor: &mut Editor,
        font: imgui::FontId,
        initial: &mut bool,
        label: &str,
    ) {
        frame(context, editor, font, initial);
        frame(context, editor, font, initial);
        let point =
            crate::gui::script_control(label).unwrap_or_else(|| panic!("Missing control {label}"));
        context.io_mut().add_mouse_pos_event(point);
        frame(context, editor, font, initial);
        context
            .io_mut()
            .add_mouse_button_event(imgui::MouseButton::Left, true);
        frame(context, editor, font, initial);
        context
            .io_mut()
            .add_mouse_button_event(imgui::MouseButton::Left, false);
        frame(context, editor, font, initial);
    }

    #[test]
    #[ignore = "Requires the pinned MIPS SDK and built epok-header-tool; compiles and packages a real project"]
    fn build_menu_dialog_exports_a_package_and_offers_its_folder() {
        let root = crate::workspace::tests::temp("ps-exe-dialog");
        let project =
            crate::workspace::create(&root, "Dialog Export", crate::workspace::Template::Basic)
                .unwrap();
        let manifest = crate::workspace::manifest_path(&root).unwrap();
        let saved = std::fs::read(&manifest).unwrap();
        let mut editor = Editor::open(project).unwrap();
        editor.auto_build = false;
        let mut context = crate::gui::tests::imgui_context();
        context.io_mut().display_size = [1400., 900.];
        context.io_mut().delta_time = 1. / 60.;
        // The editor workspace is a dockspace, as in the application.
        context.io_mut().config_flags |= imgui::ConfigFlags::DOCKING_ENABLE;
        let font = context
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
        context.fonts().build_rgba32_texture();
        let mut initial = true;
        let (c, e, i) = (&mut context, &mut editor, &mut initial);
        click(c, e, font, i, "Build menu");
        click(c, e, font, i, "Export PS-EXE...");
        assert!(e.ps_exe_export_ui.open, "The Build menu opens the dialog");
        click(c, e, font, i, "Export PS-EXE package");
        assert!(e.job.is_some(), "{:?}", e.ps_exe_export_ui.message);
        let started = Instant::now();
        while e.job.is_some() {
            assert!(
                started.elapsed() < Duration::from_secs(600),
                "Export timed out"
            );
            e.tick();
            frame(c, e, font, i);
            std::thread::sleep(Duration::from_millis(20));
        }
        frame(c, e, font, i);
        frame(c, e, font, i);
        let archive = e
            .ps_exe_export_ui
            .result
            .clone()
            .unwrap_or_else(|| panic!("{} / {:?}", e.ps_exe_export_ui.message, e.last_error));
        assert_eq!(
            archive,
            crate::code_editor::editor_path(&root.join("exports/Dialog Export-ps-exe.zip"))
        );
        let mut package = zip::ZipArchive::new(std::fs::File::open(&archive).unwrap()).unwrap();
        let mut header = [0u8; 8];
        std::io::Read::read_exact(&mut package.by_name("epok.ps-exe").unwrap(), &mut header)
            .unwrap();
        assert_eq!(&header, b"PS-X EXE");
        assert!(package.by_name("epok-package.json").is_ok());
        assert!(
            crate::gui::script_control("Open PS-EXE folder").is_some(),
            "The finished dialog offers Open Folder"
        );
        assert_eq!(
            std::fs::read(&manifest).unwrap(),
            saved,
            "Play settings unchanged"
        );
        drop(editor);
        let _ = std::fs::remove_dir_all(root);
    }
}
