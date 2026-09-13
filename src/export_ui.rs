use crate::{disc, editor::Editor};
use imgui::{Condition, StyleColor as C, StyleVar as V};

#[derive(Default)]
pub struct State {
    pub open: bool,
    options: disc::Settings,
    message: String,
}

pub fn open(editor: &mut Editor) {
    let mut state = std::mem::take(&mut editor.export_ui);
    state.options = disc::Settings::load(&editor.root).unwrap_or_else(|error| {
        state.message = error;
        Default::default()
    });
    state.message.clear();
    state.open = true;
    editor.export_ui = state;
}

fn section(ui: &imgui::Ui, title: &str, body: impl FnOnce()) {
    ui.spacing();
    let _header = ui.push_style_color(C::Header, [0.06, 0.18, 0.30, 1.0]);
    if ui.collapsing_header(title, imgui::TreeNodeFlags::DEFAULT_OPEN) {
        body();
    }
}

pub fn window(ui: &imgui::Ui, editor: &mut Editor) {
    let mut state = std::mem::take(&mut editor.export_ui);
    if !state.open {
        editor.export_ui = state;
        return;
    }
    let mut open = true;
    let mut close_requested = false;
    let screen = ui.io().display_size;
    let _vars = [
        V::WindowPadding([18., 16.]),
        V::FramePadding([9., 7.]),
        V::ItemSpacing([9., 9.]),
        V::WindowRounding(4.),
        V::FrameRounding(2.),
    ]
    .map(|value| ui.push_style_var(value));
    ui.window("Package PSX Disc")
        .opened(&mut open)
        .position([screen[0] * 0.5, screen[1] * 0.5], Condition::Appearing)
        .position_pivot([0.5, 0.5])
        .size([780., 620.], Condition::FirstUseEver)
        .size_constraints([620., 480.], [1100., 1000.])
        .build(|| {
            ui.text_colored([0.42, 0.68, 0.96, 1.], "PLATFORMS  /  PLAYSTATION");
            ui.text("Package Project");
            ui.text_disabled("Create a physical-disc image from the current project.");
            ui.separator();

            section(ui, "Target", || {
                ui.text_disabled("Select the hardware region expected by the supplied system-area license and your loader.");
                let mut index = disc::Region::ALL
                    .iter()
                    .position(|region| *region == state.options.region)
                    .unwrap_or_default();
                let labels = disc::Region::ALL.map(disc::Region::label);
                if ui.combo_simple_string("Disc Region", &mut index, &labels) {
                    state.options.region = disc::Region::ALL[index];
                }
                let mut format = disc::ImageFormat::ALL
                    .iter()
                    .position(|value| *value == state.options.format)
                    .unwrap_or_default();
                let labels = disc::ImageFormat::ALL.map(disc::ImageFormat::label);
                if ui.combo_simple_string("Image Format", &mut format, &labels) {
                    state.options.format = disc::ImageFormat::ALL[format];
                }
                if state.options.format == disc::ImageFormat::Iso {
                    ui.text_colored([1., 0.72, 0.30, 1.], "ISO is for burner compatibility. BIN/CUE is the preferred PSX image format.");
                }
            });

            section(ui, "System-area license", || {
                ui.text_wrapped("Physical CD-R boot requires both a compatible loader/modchip and a system-area license file. Epok does not supply Sony license data; select a file you are authorized to use.");
                ui.set_next_item_width(-1.);
                ui.input_text("##license-file", &mut state.options.license_file)
                    .hint("Path to your license data file (.dat)")
                    .build();
                match state.options.license_path(&editor.root) {
                    Ok(path) => ui.text_colored([0.38, 0.82, 0.52, 1.], format!("Ready: {}", path.display())),
                    Err(error) => ui.text_colored([1., 0.48, 0.36, 1.], error),
                }
            });

            section(ui, "Output", || {
                ui.text("Build folder");
                ui.same_line();
                ui.text_disabled(editor.root.join(".epok/build").display().to_string());
                ui.text("Generated image");
                ui.same_line();
                ui.text_disabled(match state.options.format {
                    disc::ImageFormat::BinCue => "epok.bin + epok.cue",
                    disc::ImageFormat::Iso => "epok.iso",
                });
                ui.text_wrapped("The package step compiles the game, validates the image output, and retains the build log beside the image.");
            });

            ui.spacing();
            ui.separator();
            ui.text_wrapped(&state.message);
            ui.same_line_with_pos((ui.content_region_avail()[0] - 258.).max(0.));
            if ui.button("Cancel") {
                close_requested = true;
            }
            ui.same_line();
            let valid = state.options.license_path(&editor.root).is_ok() && editor.job.is_none();
            let _disabled = ui.begin_disabled(!valid);
            let _primary = ui.push_style_color(C::Button, [0.035, 0.35, 0.65, 1.]);
            if ui.button_with_size("Package", [150., 0.]) {
                match state.options.save(&editor.root) {
                    Ok(()) => {
                        editor.action("build-disc");
                        state.message = "Packaging started. Follow progress in Console.".into();
                        close_requested = true;
                    }
                    Err(error) => state.message = error,
                }
            }
            if !valid && ui.is_item_hovered() {
                ui.tooltip_text(if editor.job.is_some() { "Wait for the active build to finish." } else { "A valid system-area license file is required." });
            }
        });
    if close_requested {
        open = false;
    }
    state.open = open;
    editor.export_ui = state;
}
