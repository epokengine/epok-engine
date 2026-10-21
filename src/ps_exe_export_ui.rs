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
                ui.text_disabled(if state.streaming_geometry {
                    "epok.ps-exe + GEOMETRY.BIN (Geometry Streaming is enabled)"
                } else {
                    "epok.ps-exe only (fully resident; no PCDrv reads)"
                });
                ui.checkbox("Include debug symbols (epok.elf, epok.map)", &mut options.debug_symbols);
                if ui.is_item_hovered() {
                    ui.tooltip_text("Stored under debug/ in the ZIP. They are not needed to run the game.");
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
