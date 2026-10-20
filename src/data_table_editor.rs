//! A table editor with checked asset pickers and optimistic saving.
use crate::{
    assets::{self, Kind},
    data_table::{self, DataTable, Row},
    editor::Editor,
};
use imgui::Ui;
use uuid::Uuid;

pub struct Window {
    pub asset: Uuid,
    table: DataTable,
    record: assets::Record,
    filter: String,
    error: Option<String>,
    dirty: bool,
    undo: Vec<DataTable>,
    redo: Vec<DataTable>,
}
impl Window {
    pub fn dirty(&self) -> bool {
        self.dirty
    }
}
fn store(e: &mut Editor, w: &mut Window) -> Result<(), String> {
    let index = assets::scan(&e.root, &mut Default::default());
    let revision = data_table::save(&w.record, &w.table, &index)?;
    w.record.revision = revision;
    w.dirty = false;
    w.error = None;
    e.assets.index = assets::scan(&e.root, &mut Default::default());
    e.assets.refresh();
    e.log("Saved DataTable.");
    Ok(())
}
pub fn save_all(e: &mut Editor) -> Result<(), String> {
    let Some(mut w) = e.data_table.take() else {
        return Ok(());
    };
    let result = if w.dirty { store(e, &mut w) } else { Ok(()) };
    if let Err(error) = &result {
        w.error = Some(error.clone());
    }
    e.data_table = Some(w);
    result
}
pub fn open(e: &mut Editor, id: Uuid) {
    if let Some(window) = &mut e.data_table {
        if window.asset == id {
            return;
        }
        if window.dirty {
            window.error = Some("Save or Revert this DataTable before opening another one.".into());
            return;
        }
    }
    let result = e.assets.index.resolve(id).cloned().and_then(|record| {
        let table = data_table::document(&record)?;
        Ok(Window {
            asset: id,
            table,
            record,
            filter: String::new(),
            error: None,
            dirty: false,
            undo: vec![],
            redo: vec![],
        })
    });
    match result {
        Ok(w) => e.data_table = Some(w),
        Err(error) => e.last_error = Some(error),
    }
}
pub fn create(e: &mut Editor) -> Result<Uuid, String> {
    if e.data_table.as_ref().is_some_and(Window::dirty) {
        return Err("Save or Revert the open DataTable before creating another one.".into());
    }
    let mut n = 1;
    let path = loop {
        let suffix = if n == 1 { String::new() } else { n.to_string() };
        let path = format!("assets/DataTables/DataTable{suffix}.epokasset");
        if !e.root.join(&path).exists() {
            break path;
        }
        n += 1;
    };
    let id = data_table::create(&e.root, &path, &DataTable::default())?;
    e.assets.index = assets::scan(&e.root, &mut Default::default());
    e.assets.refresh();
    open(e, id);
    Ok(id)
}
pub fn window(ui: &Ui, e: &mut Editor) {
    let Some(mut w) = e.data_table.take() else {
        return;
    };
    let name = w
        .record
        .path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();
    let mut open = true;
    ui.window(format!(
        "DataTable: {name}{}###data-table-editor",
        if w.dirty { " *" } else { "" }
    ))
    .opened(&mut open)
    .size([930., 570.], imgui::Condition::FirstUseEver)
    .build(|| body(ui, e, &mut w));
    if open {
        e.data_table = Some(w);
    } else if w.dirty {
        // A close never discards an unfinished or invalid edit.
        w.error = Some("Save or Revert changes before closing this DataTable.".into());
        e.data_table = Some(w);
    }
}
fn body(ui: &Ui, e: &mut Editor, w: &mut Window) {
    let before = w.table.clone();
    if ui.button("Save") {
        // The captured revision belongs to the document that was opened, not a
        // fresh index record: external changes cannot be silently overwritten.
        match store(e, w) {
            Ok(()) => {}
            Err(error) => w.error = Some(error),
        }
    }
    ui.same_line();
    if ui.button("Revert") {
        match assets::read_metadata(&w.record.path).and_then(|meta| {
            let bytes = std::fs::read(&w.record.path).map_err(|e| e.to_string())?;
            let record = assets::Record {
                path: w.record.path.clone(),
                meta,
                revision: assets::hash(&bytes),
            };
            Ok((data_table::document(&record)?, record))
        }) {
            Ok((table, record)) => {
                w.table = table;
                w.record = record;
                w.dirty = false;
                w.undo.clear();
                w.redo.clear();
                w.error = None;
            }
            Err(error) => w.error = Some(error),
        }
        return;
    }
    ui.same_line();
    if ui.button("Undo") {
        if let Some(table) = w.undo.pop() {
            w.redo.push(w.table.clone());
            w.table = table;
            w.dirty = true;
        }
        return;
    }
    ui.same_line();
    if ui.button("Redo") {
        if let Some(table) = w.redo.pop() {
            w.undo.push(w.table.clone());
            w.table = table;
            w.dirty = true;
        }
        return;
    }
    ui.same_line();
    ui.text_disabled(format!("{} rows", w.table.rows.len()));
    ui.separator();
    let selected = w
        .table
        .row_type
        .as_ref()
        .map(|k| format!("{k:?}"))
        .unwrap_or_else(|| "Asset".into());
    if let Some(_combo) = ui.begin_combo("Row type", &selected) {
        for name in data_table::KINDS {
            if ui
                .selectable_config(name)
                .selected(*name == selected)
                .build()
            {
                let kind = if *name == "Asset" {
                    None
                } else {
                    Some(assets::Kind::runtime_reference(name).unwrap())
                };
                if w.table
                    .rows
                    .iter()
                    .all(|row| kind.as_ref().is_none_or(|k| k == &row.kind))
                {
                    w.table.row_type = kind;
                    w.error = None;
                } else {
                    w.error = Some("Existing rows have another type. Choose Asset for a mixed table, or remove those rows first.".into());
                }
            }
        }
    }
    ui.input_text("Filter", &mut w.filter).build();
    if ui.button("Add row") && w.table.rows.len() < data_table::MAX_ROWS {
        let mut n = 1;
        let key = loop {
            let key = format!("NewAsset{n}");
            if !w.table.rows.iter().any(|r| r.key == key) {
                break key;
            }
            n += 1;
        };
        w.table.rows.push(Row {
            key,
            kind: w.table.row_type.clone().unwrap_or(Kind::AudioClip),
            asset: Uuid::nil(),
        });
    }
    ui.same_line();
    if ui.button("Add all matching assets") {
        let records: Vec<_> = e
            .assets
            .index
            .usable()
            .filter(|r| {
                r.meta.id != w.asset
                    && w.table
                        .row_type
                        .as_ref()
                        .is_some_and(|kind| kind == &r.meta.kind)
            })
            .cloned()
            .collect();
        if w.table.row_type.is_none() {
            w.error = Some("Select a row type before adding all matching assets.".into());
        }
        for record in records {
            if w.table.rows.len() == data_table::MAX_ROWS {
                break;
            }
            if w.table.rows.iter().any(|row| row.asset == record.meta.id) {
                continue;
            }
            let stem = record
                .path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy();
            let mut key = stem
                .chars()
                .filter(|c| !c.is_control())
                .take(96)
                .collect::<String>()
                .trim()
                .to_owned();
            if key.is_empty() {
                key = "Asset".into();
            }
            let base = key.clone();
            let mut n = 2;
            while w.table.rows.iter().any(|row| row.key == key) {
                key = format!("{base}_{n}");
                n += 1;
            }
            w.table.rows.push(Row {
                key,
                kind: record.meta.kind,
                asset: record.meta.id,
            });
        }
    }
    ui.text_wrapped("Keys are case-sensitive and local to this table. Choose an asset to replace a sound everywhere that uses its key.");
    ui.separator();
    let mut remove = None;
    ui.child_window("data-table-rows")
        .size([0., -65.])
        .build(|| {
            if let Some(_table) = ui.begin_table_with_flags(
                "references",
                3,
                imgui::TableFlags::BORDERS
                    | imgui::TableFlags::ROW_BG
                    | imgui::TableFlags::RESIZABLE,
            ) {
                for (name, width) in [("Key", 210.), ("Asset reference", 0.), ("", 75.)] {
                    ui.table_setup_column_with(imgui::TableColumnSetup {
                        name,
                        flags: if width == 0. {
                            imgui::TableColumnFlags::WIDTH_STRETCH
                        } else {
                            imgui::TableColumnFlags::WIDTH_FIXED
                        },
                        init_width_or_weight: if width == 0. { 1. } else { width },
                        ..Default::default()
                    });
                }
                ui.table_headers_row();
                for (i, row) in w.table.rows.iter_mut().enumerate() {
                    if !w.filter.is_empty()
                        && !row.key.to_lowercase().contains(&w.filter.to_lowercase())
                    {
                        continue;
                    }
                    let _id = ui.push_id_usize(i);
                    ui.table_next_row();
                    ui.table_next_column();
                    ui.set_next_item_width(-1.);
                    ui.input_text("##key", &mut row.key).build();
                    ui.table_next_column();
                    let browse_width = ui.frame_height();
                    let spacing = ui.clone_style().item_spacing[0];
                    ui.set_next_item_width(
                        (ui.content_region_avail()[0] - browse_width - spacing).max(1.),
                    );
                    let label = e
                        .assets
                        .index
                        .resolve(row.asset)
                        .map(|r| {
                            r.path
                                .file_stem()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .into_owned()
                        })
                        .unwrap_or_else(|_| {
                            if row.asset.is_nil() {
                                "Choose asset".into()
                            } else {
                                format!("Missing: {}", row.asset)
                            }
                        });
                    if let Some(_combo) = ui.begin_combo("##asset", label) {
                        for record in e.assets.index.usable().filter(|r| {
                            r.meta.id != w.asset
                                && w.table.row_type.as_ref().map_or_else(
                                    || data_table::kind_code(&format!("{:?}", r.meta.kind)).is_ok(),
                                    |kind| r.meta.kind == *kind,
                                )
                        }) {
                            let label = assets::path_string(&e.root, &record.path);
                            if ui
                                .selectable_config(&label)
                                .selected(record.meta.id == row.asset)
                                .build()
                            {
                                row.asset = record.meta.id;
                                row.kind = record.meta.kind.clone();
                            }
                        }
                    }
                    if ui.is_item_hovered() {
                        let path = e
                            .assets
                            .index
                            .resolve(row.asset)
                            .map(|r| assets::path_string(&e.root, &r.path))
                            .unwrap_or_else(|error| error);
                        ui.tooltip_text(format!("{path}\n{}", row.asset));
                    }
                    ui.same_line();
                    let path = e
                        .assets
                        .index
                        .resolve(row.asset)
                        .ok()
                        .map(|record| record.path.clone());
                    let disabled = ui.begin_disabled(path.is_none());
                    if ui.button_with_size("\u{ea6d}##find-in-project", [browse_width, 0.]) {
                        if let Some(path) = &path {
                            crate::project_browser::State::select_asset(e, path);
                        }
                    }
                    drop(disabled);
                    if ui.is_item_hovered_with_flags(imgui::ItemHoveredFlags::ALLOW_WHEN_DISABLED) {
                        ui.tooltip_text(if path.is_some() {
                            "Find in Project"
                        } else {
                            "Asset unavailable"
                        });
                    }
                    ui.table_next_column();
                    if ui.small_button("Remove") {
                        remove = Some(i);
                    }
                }
            }
        });
    if let Some(i) = remove {
        w.table.rows.remove(i);
    }
    if before != w.table {
        w.undo.push(before);
        if w.undo.len() > 64 {
            w.undo.remove(0);
        }
        w.redo.clear();
        w.dirty = true;
    }
    if let Some(error) = &w.error {
        ui.text_wrapped(error);
    } else if let Err(error) = w.table.validate_assets(&e.assets.index) {
        ui.text_wrapped(error);
    } else {
        ui.text_disabled(if w.dirty {
            "Unsaved changes"
        } else {
            "Saved - all references valid"
        });
    }
}
