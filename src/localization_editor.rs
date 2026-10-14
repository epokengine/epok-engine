//! The string table window (World2D initiative, phase 3): the project's translated texts
//! as a grid of keys and languages, with search, missing-translation counts, a preview
//! language for the editor, and CSV import/export for spreadsheets. Every edit saves the
//! table and refreshes the HUD preview.
use crate::{
    editor::Editor,
    localization::{self, Entry, StringTable},
};
use imgui::Ui;

const TABLE_PATH: &str = "assets/Localization/Strings.epokasset";
const CSV_PATH: &str = "assets/Localization/strings.csv";

#[derive(Default)]
pub struct State {
    pub open: bool,
    filter: String,
    new_key: String,
    new_language: String,
    status: String,
    dirty: bool,
    table: Option<StringTable>,
}

/// Opens the window on the project's table, creating an empty one the first time.
pub fn open(e: &mut Editor) {
    if crate::localization::project_table(&e.assets.index).is_none() {
        match localization::create(&e.root, TABLE_PATH, &StringTable::default()) {
            Ok(_) => {
                e.assets.index = crate::assets::scan(&e.root, &mut Default::default());
                e.assets.refresh();
                e.log(format!("Created {TABLE_PATH}."));
            }
            Err(error) => {
                e.last_error = Some(error.clone());
                e.log(error);
                return;
            }
        }
    }
    e.localization.table = crate::localization::project_table(&e.assets.index).map(|(_, t)| t);
    e.localization.open = true;
}
fn store(e: &mut Editor) {
    let Some(table) = e.localization.table.clone() else {
        return;
    };
    let result = crate::localization::project_table(&e.assets.index)
        .ok_or_else(|| "The string table asset is missing".to_string())
        .and_then(|(record, _)| localization::save(&record, &table));
    match result {
        Ok(()) => {
            e.localization.status = "Saved".into();
            // The open map previews the new strings at once.
            let keyed = e
                .scene
                .actors
                .iter()
                .any(|a| a.text.as_ref().is_some_and(|t| !t.key.is_empty()));
            e.scene.strings = keyed.then(|| std::sync::Arc::new(table));
            e.assets.refresh();
            e.view_dirty = true;
        }
        Err(error) => e.localization.status = error,
    }
}

pub fn window(ui: &Ui, e: &mut Editor) {
    if !e.localization.open {
        return;
    }
    let Some(mut table) = e.localization.table.take() else {
        e.localization.open = false;
        return;
    };
    let before = table.clone();
    let mut open = true;
    // Beside the Inspector, so the canvas it previews stays in sight.
    let [width, _] = ui.io().display_size;
    ui.window("Strings###localization")
        .opened(&mut open)
        .size([660., 400.], imgui::Condition::FirstUseEver)
        .position([width - 400., 96.], imgui::Condition::FirstUseEver)
        .position_pivot([1., 0.])
        .build(|| body(ui, e, &mut table));
    table.normalize();
    if table != before {
        e.localization.dirty = true;
    }
    e.localization.table = Some(table);
    if e.localization.dirty && !ui.is_any_item_active() {
        e.localization.dirty = false;
        store(e);
    }
    e.localization.open = open;
}

fn body(ui: &Ui, e: &mut Editor, table: &mut StringTable) {
    // Languages: the first one is the language the game starts in.
    ui.align_text_to_frame_padding();
    ui.text("Languages");
    ui.same_line();
    let mut make_first = None;
    let mut remove_language = None;
    for (i, language) in table.languages.iter().enumerate() {
        let _id = ui.push_id_usize(i);
        let label = if i == 0 {
            format!("{language} (start)")
        } else {
            language.clone()
        };
        if ui.small_button(&label) {
            ui.open_popup("language-menu");
        }
        ui.popup("language-menu", || {
            if i > 0 && ui.menu_item("Start the game in this language") {
                make_first = Some(i);
            }
            if table.languages.len() > 1 && ui.menu_item("Remove language") {
                remove_language = Some(i);
            }
        });
        ui.same_line();
    }
    ui.set_next_item_width(110.);
    ui.input_text("##new-language", &mut e.localization.new_language)
        .hint("New language")
        .build();
    ui.same_line();
    if ui.small_button("Add language")
        && !e.localization.new_language.trim().is_empty()
        && table.languages.len() < localization::MAX_LANGUAGES
    {
        table
            .languages
            .push(e.localization.new_language.trim().to_owned());
        e.localization.new_language.clear();
    }
    if let Some(i) = make_first {
        let name = table.languages.remove(i);
        table.languages.insert(0, name);
        for entry in &mut table.entries {
            let value = entry.values.remove(i);
            entry.values.insert(0, value);
        }
    }
    if let Some(i) = remove_language {
        table.languages.remove(i);
        for entry in &mut table.entries {
            if i < entry.values.len() {
                entry.values.remove(i);
            }
        }
    }
    // Preview language and spreadsheet round trip.
    let languages: Vec<&str> = table.languages.iter().map(String::as_str).collect();
    let mut preview = e
        .scene
        .preview_language
        .min(languages.len().saturating_sub(1));
    ui.set_next_item_width(160.);
    if ui.combo_simple_string("Preview in", &mut preview, &languages) {
        e.scene.preview_language = preview;
        e.view_dirty = true;
    }
    ui.same_line_with_spacing(0., 16.);
    if ui.button("Export CSV") {
        let result = crate::assets::inside(&e.root, CSV_PATH).and_then(|path| {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            crate::project::write_changed(&path, table.to_csv().as_bytes())
        });
        e.localization.status = match result {
            Ok(()) => format!("Exported {CSV_PATH}"),
            Err(error) => error,
        };
    }
    ui.same_line();
    if ui.button("Import CSV") {
        let result = crate::assets::inside(&e.root, CSV_PATH)
            .and_then(|path| std::fs::read_to_string(&path).map_err(|e| format!("{CSV_PATH}: {e}")))
            .and_then(|text| StringTable::from_csv(&text));
        match result {
            Ok(imported) => {
                *table = imported;
                e.localization.status = format!("Imported {CSV_PATH}");
            }
            Err(error) => e.localization.status = error,
        }
    }
    if ui.is_item_hovered() {
        ui.tooltip_text(format!("Replaces the table with {CSV_PATH}: a header row (key, then one column per language) and one row per key"));
    }
    if !e.localization.status.is_empty() {
        ui.same_line();
        ui.text_disabled(&e.localization.status);
    }
    // Missing translations per language.
    let missing: Vec<String> = table
        .languages
        .iter()
        .enumerate()
        .skip(1)
        .map(|(i, l)| {
            let count = table
                .entries
                .iter()
                .filter(|e| e.values.get(i).is_none_or(|v| v.is_empty()))
                .count();
            format!("{l}: {count} untranslated")
        })
        .collect();
    if !missing.is_empty() {
        ui.text_disabled(missing.join("   "));
    }
    ui.separator();
    ui.set_next_item_width(200.);
    ui.input_text("##filter", &mut e.localization.filter)
        .hint("Search keys and texts")
        .build();
    ui.same_line();
    ui.set_next_item_width(200.);
    let add = ui
        .input_text("##new-key", &mut e.localization.new_key)
        .hint("new.key")
        .enter_returns_true(true)
        .build();
    ui.same_line();
    let key = e.localization.new_key.trim().to_owned();
    if (ui.button("Add key") || add)
        && !key.is_empty()
        && table.key_index(&key).is_none()
        && table.entries.len() < localization::MAX_KEYS
    {
        table.entries.push(Entry {
            key,
            values: vec![String::new(); table.languages.len()],
        });
        e.localization.new_key.clear();
    }
    let filter = e.localization.filter.to_lowercase();
    let columns = table.languages.len() + 3;
    let mut remove = None;
    let available = ui.content_region_avail();
    if let Some(_table) = ui.begin_table_with_sizing(
        "##strings",
        columns,
        imgui::TableFlags::BORDERS
            | imgui::TableFlags::ROW_BG
            | imgui::TableFlags::SCROLL_Y
            | imgui::TableFlags::RESIZABLE,
        [available[0], (available[1] - 44.).max(120.)],
        0.,
    ) {
        ui.table_setup_scroll_freeze(0, 1);
        ui.table_setup_column("#");
        ui.table_setup_column("Key");
        for language in &table.languages {
            ui.table_setup_column(language);
        }
        ui.table_setup_column("");
        ui.table_headers_row();
        for (index, entry) in table.entries.iter_mut().enumerate() {
            if !filter.is_empty()
                && !entry.key.to_lowercase().contains(&filter)
                && !entry
                    .values
                    .iter()
                    .any(|v| v.to_lowercase().contains(&filter))
            {
                continue;
            }
            let _id = ui.push_id_usize(index);
            ui.table_next_row();
            ui.table_next_column();
            ui.text_disabled(index.to_string());
            ui.table_next_column();
            ui.set_next_item_width(-1.);
            ui.input_text("##key", &mut entry.key).build();
            for (language, value) in entry.values.iter_mut().enumerate() {
                ui.table_next_column();
                ui.set_next_item_width(-1.);
                let _colour = (value.is_empty() && language > 0).then(|| {
                    ui.push_style_color(imgui::StyleColor::FrameBg, [0.35, 0.22, 0.12, 1.])
                });
                ui.input_text(format!("##value{language}"), value).build();
            }
            ui.table_next_column();
            if ui.small_button("x") {
                remove = Some(index);
            }
        }
    }
    if let Some(index) = remove {
        table.entries.remove(index);
    }
    ui.text_wrapped("Texts pick a key in the Inspector. Gameplay switches with Localization > set_language; # is the number set_text_key takes. [color=#rrggbb] tags work here too.");
    if let Err(error) = table.validate() {
        ui.same_line();
        ui.text_colored([1., 0.45, 0.4, 1.], error);
    }
}
