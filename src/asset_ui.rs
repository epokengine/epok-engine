use crate::{
    asset_manager::{Manager, Status},
    assets,
    editor::Editor,
};
use imgui::{Condition, Ui};

pub fn windows(ui: &Ui, e: &mut Editor) {
    let m = &mut e.assets;
    if m.notification && !m.window {
        let size = ui.io().display_size;
        ui.window("Files detected##import-toast")
            .position([size[0] - 440., size[1] - 160.], Condition::Always)
            .size([420., 130.], Condition::Always)
            .resizable(false)
            .collapsible(false)
            .build(|| {
                ui.text_wrapped(format!(
                    "{} source file changes are ready to import.",
                    m.pending
                        .iter()
                        .filter(|p| p.status == Status::Pending)
                        .count()
                ));
                if ui.button("Import...") {
                    m.window = true;
                    m.focus_tab = Some(0);
                    m.notification = false;
                }
                crate::gui::inline(ui, "Omit");
                if ui.button("Omit") {
                    let keys = m
                        .pending
                        .iter()
                        .filter(|p| p.status == Status::Pending)
                        .map(|p| p.key())
                        .collect::<Vec<_>>();
                    for key in keys {
                        m.omit(&key);
                    }
                    m.notification = false;
                }
            });
    }
    let mut open = m.window;
    if open {
        ui.window("Imports and Assets")
            .opened(&mut open)
            .position(
                [ui.io().display_size[0] * 0.5, ui.io().display_size[1] * 0.5],
                Condition::Appearing,
            )
            .position_pivot([0.5, 0.5])
            .size([800., 580.], Condition::FirstUseEver)
            .size_constraints([580., 380.], [1800., 1200.])
            .build(|| manager_body(ui, m, &e.scene));
    }
    m.window = open;
    import_dialog(ui, m);
    crate::skeletal_ui::window(ui, e);
}
fn manager_body(ui: &Ui, m: &mut Manager, scene: &crate::scene::Scene) {
    if ui.button("Create SoundBank...") { m.begin_new_bank(); }
    ui.same_line();
    if ui.button("Refresh") {
        m.refresh();
    }
    ui.same_line();
    crate::gui::muted(
        ui,
        if m.busy {
            "Converting in background..."
        } else {
            "Copy PNG, audio or FBX sources into assets/ to import them."
        },
    );
    if let Some(error) = &m.error {
        ui.text_colored([1., 0.5, 0.35, 1.], error);
    }
    ui.separator();
    if let Some(_tab_bar) = ui.tab_bar("ImportTabs") {
        let focus = m.focus_tab.take();
        let flags = |tab| {
            if focus == Some(tab) {
                imgui::TabItemFlags::SET_SELECTED
            } else {
                imgui::TabItemFlags::empty()
            }
        };
        if let Some(_tab) = ui.tab_item_with_flags("Pending / Omitted", None, flags(0)) {
            if m.pending.is_empty() {
                crate::gui::muted(ui, "No source changes pending.");
            }
            for item in m.pending.clone() {
                let _id = ui.push_id(item.key());
                ui.text_wrapped(&item.source);
                match &item.status {
                    Status::Pending => crate::gui::muted(
                        ui,
                        if item.existing.is_some() {
                            "Reimport available"
                        } else {
                            "Pending import"
                        },
                    ),
                    Status::Omitted => {
                        crate::gui::muted(ui, "Omitted - import manually whenever ready")
                    }
                    Status::Error(error) => ui.text_colored([1., 0.5, 0.35, 1.], error),
                }
                ui.disabled(m.busy, || {
                    let import = ui.button(if item.existing.is_some() {
                        "Reimport..."
                    } else {
                        "Import..."
                    });
                    #[cfg(test)]
                    interaction::track(ui, "pending");
                    if import {
                        m.begin_pending(&item);
                    }
                    crate::gui::inline(ui, "Omit");
                    if ui.button("Omit") {
                        m.omit(&item.key());
                    }
                });
                ui.separator();
            }
        }
        if let Some(_tab) = ui.tab_item_with_flags("Conflicts / Problems", None, flags(1)) {
            for records in m.index.assets.clone().values().filter(|v| v.len() > 1) {
                ui.text_colored(
                    [1., 0.65, 0.3, 1.],
                    format!("Duplicate UUID: {}", records[0].meta.id),
                );
                ui.text_wrapped("Choose the copies that should become independent assets. Existing references retain the original UUID.");
                for record in records {
                    let _id = ui.push_id(record.path.to_string_lossy());
                    ui.text_wrapped(assets::path_string(&m.root, &record.path));
                    ui.disabled(m.busy, || {
                        if ui.button("Give this copy a new UUID") {
                            m.report(assets::make_independent(record));
                        }
                    });
                }
                ui.separator();
            }
            for problem in &m.index.problems {
                ui.text_wrapped(problem);
            }
            for entity in &scene.entities {
                if let Some(id) = entity.audio.as_ref().and_then(|a| a.clip)
                    && let Err(error) = m.index.resolve(id)
                {
                    ui.text_wrapped(format!("{}: {error}", entity.name));
                }
            }
            crate::gui::muted(
                ui,
                "Missing UUID references are preserved. Restore the asset anywhere under assets/ to recover them.",
            );
        }
        if let Some(_tab) = ui.tab_item_with_flags("Selected asset", None, flags(2)) {
            if let Some(id) = m.selected {
                match m.index.resolve(id).cloned() {
                    Ok(record) => {
                        if matches!(record.meta.kind, assets::Kind::MusicSequence | assets::Kind::SoundBank) {
                            ui.text_wrapped(assets::path_string(&m.root, &record.path));
                            ui.text(format!("{:?} / {}", record.meta.kind, record.meta.id));
                            ui.text_wrapped("Select in Content for analysis and audition. Original source and dependencies remain in the asset.");
                            if ui.button("Change sequence / bank settings...") { m.begin_reimport(&record, true); }
                            if ui.button("Reimport / Locate source...") { m.begin_reimport(&record, false); }
                            return;
                        }
                        if record.meta.kind == assets::Kind::Texture {
                            texture_inspector(ui, m, &record);
                            return;
                        }
                        if !matches!(
                            record.meta.kind,
                            assets::Kind::AudioClip
                                | assets::Kind::EditableMesh
                                | assets::Kind::Texture
                        ) {
                            ui.text_wrapped(assets::path_string(&m.root, &record.path));
                            ui.text(
                                "Select this asset in Project to open the skeletal model preview.",
                            );
                            return;
                        }
                        if record.meta.kind == assets::Kind::EditableMesh {
                            ui.text_wrapped(assets::path_string(&m.root, &record.path));
                            ui.text("EditableMesh: select it in Project to open Blockout.");
                            crate::gui::muted(
                                ui,
                                "Authored geometry has no external import source.",
                            );
                            return;
                        }

                        ui.text_wrapped(assets::path_string(&m.root, &record.path));
                        crate::gui::muted(ui, format!("AudioClip / {id}"));
                        ui.text_wrapped(format!("Imported from: {}", record.meta.source));
                        ui.text(format!(
                            "{:?} / {} channel(s)",
                            record.meta.settings.audio().unwrap().role,
                            record.meta.settings.audio().unwrap().channels
                        ));
                        match m.index.linked_source(&record) {
                            Ok(Some(source)) => {
                                ui.text_wrapped(format!("Linked source: {}", source.path))
                            }
                            Ok(None) => ui.text_colored(
                                [1., 0.7, 0.4, 1.],
                                "External source missing. The imported snapshot is preserved.",
                            ),
                            Err(error) => ui.text_wrapped(error),
                        }
                        ui.text(format!(
                            "{} Hz / loop {}",
                            record.meta.settings.audio().unwrap().rate(),
                            record.meta.settings.audio().unwrap().looping
                        ));
                        if let Ok(bytes) = std::fs::read(
                            m.root
                                .join(".epok/imported")
                                .join(assets::cache_key(&record.meta))
                                .join("Import.epokcache"),
                        ) && let Ok(info) =
                            crate::document::from_slice::<serde_json::Value>(&bytes)
                            && let Some(wave) = info["waveform"].as_array()
                        {
                            ui.text(format!("Last PSX cook: {} encoded bytes", info["bytes"]));
                            let values = wave
                                .iter()
                                .map(|v| v.as_f64().unwrap_or(0.) as f32)
                                .collect::<Vec<_>>();
                            ui.plot_histogram("Waveform", &values)
                                .scale_min(0.)
                                .scale_max(1.)
                                .graph_size([ui.content_region_avail()[0].max(1.), 80.])
                                .build();
                        }
                        ui.disabled(m.busy, || {
                            if ui.button("Reimport / Locate source...") {
                                m.begin_reimport(&record, false);
                            }
                            crate::gui::inline(ui, "Change import settings...");
                            if ui.button("Change import settings...") {
                                m.begin_reimport(&record, true);
                            }
                            ui.separator();
                            ui.set_next_item_width(-1.);
                            ui.input_text("##asset-destination", &mut m.operation_path)
                                .hint("assets/Audio/NewName.epokasset")
                                .build();
                            if ui.button("Move / Rename") {
                                let result =
                                    assets::move_asset(&m.root, &record, &m.operation_path);
                                m.report(result);
                            }
                            crate::gui::inline(ui, "Duplicate");
                            if ui.button("Duplicate") {
                                let result =
                                    assets::inside(&m.root, &m.operation_path).and_then(|path| {
                                        if path.extension().is_none_or(|e| e != "epokasset") {
                                            return Err("Use the .epokasset extension".into());
                                        }
                                        assets::duplicate(&record, &path)
                                    });
                                m.report(result);
                            }
                            if ui.button("Used by...") {
                                match assets::dependencies(&m.root, id) {
                                    Ok(uses) => m.messages.push(if uses.is_empty() {
                                        "Asset has no saved scene references.".into()
                                    } else {
                                        uses.join("\n")
                                    }),
                                    Err(error) => m.error = Some(error),
                                }
                            }
                            crate::gui::inline(ui, "Delete...");
                            if ui.button("Delete...") {
                                ui.open_popup("Delete asset?");
                            }
                        });
                        ui.modal_popup_config("Delete asset?").always_auto_resize(true).build(||{
                                ui.text_wrapped("Move this asset to UserSettings/AssetTrash? Saved and current scene references must be removed first.");
                                if ui.button("Move to trash"){
                                    let in_use=scene.entities.iter().any(|entity|entity.audio.as_ref().is_some_and(|a|a.clip==Some(id)));
                                    if in_use{m.error=Some("The open scene still references this asset.".into());}
                                    else{match assets::trash(&m.root,&record){Ok(path)=>{m.last_deleted=Some(path);m.selected=None;m.refresh();},Err(error)=>m.error=Some(error)}}
                                    ui.close_current_popup();
                                }
                                crate::gui::inline(ui, "Cancel");if ui.button("Cancel"){ui.close_current_popup();}
                            });
                    }
                    Err(error) => ui.text_wrapped(error),
                }
            } else {
                crate::gui::muted(ui, "Select an imported asset in Project.");
            }
            if let Some(path) = m.last_deleted.clone() {
                ui.separator();
                ui.text_wrapped(
                    "Restore the last deleted asset to the project-relative destination below.",
                );
                ui.input_text(crate::gui::field(ui, "Restore path"), &mut m.operation_path)
                    .build();
                if ui.button("Restore") {
                    let result =
                        assets::inside(&m.root, &m.operation_path).and_then(|destination| {
                            if destination.extension().is_none_or(|e| e != "epokasset") {
                                return Err("Use .epokasset".into());
                            }
                            assets::atomic_write(&destination, &assets::read_bounded(&path)?, None)
                        });
                    m.report(result);
                }
            }
        }
    }
}
fn import_dialog(ui: &Ui, m: &mut Manager) {
    if m.form.as_ref().is_some_and(|f| f.sequence.is_some() || f.bank.is_some()) {
        portable_audio_dialog(ui, m);
        return;
    }
    if m.form.as_ref().is_some_and(|f| f.texture) {
        m.open_dialog = false;
        let mut open = true;
        let mut start = false;
        ui.window("Import PNG Texture").opened(&mut open).size([560.,260.],Condition::FirstUseEver).build(||{
            let form=m.form.as_mut().unwrap();
            ui.text_wrapped("PNG up to 256 x 256. PSX 8-bit palette (255 colors + transparent). Alpha below 128 is cut out. Nearest-color quantization is deterministic.");
            ui.disabled(m.busy,||{ui.disabled(form.snapshot,||{ui.input_text("Source",&mut form.source).build();});ui.disabled(form.existing.is_some(),||{ui.input_text("Asset",&mut form.destination).build();});if ui.button("Import"){start=true;}});
            if let Some(error)=&m.error{ui.text_wrapped(error);}
        });
        if start {
            m.start_import();
        }
        if !open && !m.busy {
            m.form = None;
        }
        return;
    }
    if m.form.as_ref().is_some_and(|f| f.model) {
        m.open_dialog = false;
        let mut open = true;
        let mut start = false;
        ui.window("Import FBX model").opened(&mut open)
            .position([ui.io().display_size[0]*0.5,ui.io().display_size[1]*0.5],Condition::Appearing).position_pivot([0.5,0.5])
            .size([680.,380.],Condition::FirstUseEver).size_constraints([520.,300.],[1600.,1000.]).build(||{
            let form=m.form.as_mut().unwrap();
            ui.text("FBX -> PlayStation skeletal assets");
            ui.text_wrapped("512 vertices / 1024 triangles / 64 bones and helpers. Strongest weight per vertex. Clips sampled at 30 Hz; flat diffuse colors.");
            ui.disabled(m.busy,||{
                ui.disabled(form.snapshot,||{ui.input_text(crate::gui::field(ui, "Source"),&mut form.source).build();});
                ui.disabled(form.existing.is_some(),||{ui.input_text(crate::gui::field(ui, "Model asset"),&mut form.destination).build();});
                if form.snapshot{ui.text("Using the original FBX stored in ModelSource.");}
                if ui.button(if form.existing.is_some(){"Reimport"}else{"Import"}){start=true;}
            });
            if m.busy{ui.text("Converting model and clips...");}
            if let Some(error)=&m.error{ui.text_wrapped(error);}
        });
        if start {
            m.start_import();
        }
        if !open && !m.busy {
            m.form = None;
        }
        return;
    }

    if m.open_dialog {
        ui.open_popup("Import AudioClip");
        m.open_dialog = false;
    }
    let mut start = false;
    // Modal builders do not expose size constraints in imgui-rs 0.12.
    unsafe {
        imgui::sys::igSetNextWindowSize(
            imgui::sys::ImVec2 {
                x: 720.,
                y: (ui.io().display_size[1] - 100.).min(560.),
            },
            imgui::sys::ImGuiCond_Appearing as i32,
        );
        imgui::sys::igSetNextWindowSizeConstraints(
            imgui::sys::ImVec2 { x: 560., y: 420. },
            imgui::sys::ImVec2 { x: 1600., y: 1200. },
            None,
            std::ptr::null_mut(),
        );
    }
    ui.modal_popup_config("Import AudioClip").build(||{
        let Some(form)=&mut m.form else{ui.close_current_popup();return;};
        ui.text("Audio -> PlayStation");
        crate::gui::muted(ui, "WAV, MP3, FLAC and OGG. Role describes intent; Load Mode chooses residency.");
        ui.disabled(m.busy,||{
            ui.set_next_item_width(570.);
            ui.disabled(form.snapshot,||{ui.input_text(crate::gui::field(ui, "Source"),&mut form.source).build();});
            ui.set_next_item_width(570.);
            ui.disabled(form.existing.is_some(),||{ui.input_text(crate::gui::field(ui, "Asset"),&mut form.destination).build();});
            audio_settings(ui, &mut form.settings);
            crate::gui::Drag::new(crate::gui::field(ui, "Trim start (seconds)")).range(0.,600.).speed(0.1).build(ui,&mut form.settings.trim_start);
            let mut trim=form.settings.trim_end.is_some();if ui.checkbox("Trim end",&mut trim){form.settings.trim_end=trim.then_some(form.settings.trim_start+1.);}
            if let Some(end)=&mut form.settings.trim_end{crate::gui::Drag::new(crate::gui::field(ui, "End (seconds)")).range(0.,600.).speed(0.1).build(ui,end);}
            ui.checkbox("Normalize",&mut form.settings.normalize);crate::gui::inline(ui, "Loop entire clip");ui.checkbox("Loop entire clip",&mut form.settings.looping);
            if form.snapshot{ui.text_wrapped("Converting the original stored inside the asset. No external source is required.");}
            if ui.button(if form.existing.is_some(){"Reimport"}else{"Import"}){start=true;}
            #[cfg(test)]
            interaction::track(ui, "import");
        });
        if m.busy {ui.text("Converting... Existing assets stay usable until validation succeeds.");}
        if let Some(error)=&m.error {ui.text_wrapped(error);}
        if !m.busy {crate::gui::inline(ui, "Cancel");if ui.button("Cancel"){m.form=None;ui.close_current_popup();}}
    });
    if start {
        m.start_import();
    }
}

pub fn bank_selector(ui: &Ui, label: &str, selected: &mut Option<uuid::Uuid>, index: &assets::Index, default_label: &str) {
    let title = selected.map_or_else(|| default_label.to_string(), |id| index.resolve(id).map(|r| r.path.file_stem().unwrap_or_default().to_string_lossy().into_owned()).unwrap_or_else(|_| format!("Missing {id}")));
    if let Some(_combo) = ui.begin_combo(label, title) {
        if ui.selectable_config(default_label).selected(selected.is_none()).build() { *selected = None; }
        for r in index.usable().filter(|r| r.meta.kind == assets::Kind::SoundBank) {
            if ui.selectable_config(format!("{}##{}", r.path.file_stem().unwrap_or_default().to_string_lossy(), r.meta.id)).selected(*selected == Some(r.meta.id)).build() { *selected = Some(r.meta.id); }
            #[cfg(test)]
            interaction::track(ui, "choose-bank");
        }
    }
    #[cfg(test)]
    interaction::track(ui, "bank-selector");
}
fn portable_audio_dialog(ui: &Ui, m: &mut Manager) {
    m.open_dialog = false;
    let mut open = true;
    let mut start = false;
    let mut install_reference = false;
    ui.window("MusicSequence / SoundBank").opened(&mut open)
        .size([760., 650.], Condition::FirstUseEver).size_constraints([540., 350.], [1600., 1200.]).build(|| {
        let form = m.form.as_mut().unwrap();
        ui.disabled(m.busy, || {
            if !form.source.is_empty() { ui.disabled(form.snapshot, || { if ui.input_text("Source", &mut form.source).build() { form.sequence_catalog=None; } });
                crate::music_conversion_ui::help(ui,"source","Original project-relative source. Snapshot reimport reads the source preserved inside the asset, so it remains reproducible if the external file changes."); }
            ui.disabled(form.existing.is_some(), || { ui.input_text("Asset", &mut form.destination).build(); });
            crate::music_conversion_ui::help(ui,"asset","Destination asset package. Reimport preserves its UUID so scenes and scripts keep their references.");
            if let Some(settings) = &mut form.sequence {
                ui.text("MusicSequence");
                if ui.button("Inspect source (Auto)") {
                    form.sequence_catalog=Some((|| {
                        let bytes=if form.snapshot {
                            assets::Package::load(&form.existing.as_ref().ok_or("No sequence snapshot")?.path)?.source
                        } else { assets::read_bounded(&assets::inside(&m.root,&form.source)?)? };
                        crate::sequence::catalog_source(&bytes,None)
                    })());
                }
                #[cfg(test)]
                interaction::track(ui, "inspect-sequence-source");
                for (label, profile) in [
                    ("Sony SEQ profile", crate::sequence::SourceProfile::SonySeqV1),
                    ("Sony SEP profile", crate::sequence::SourceProfile::SonySepV0),
                    ("Converted SEQ/SEP (LE32)", crate::sequence::SourceProfile::ConvertedSeqLe32V1),
                ] {
                    ui.same_line();
                    if ui.small_button(label) {
                        form.sequence_catalog=Some((|| {
                            let bytes=if form.snapshot {
                                assets::Package::load(&form.existing.as_ref().ok_or("No sequence snapshot")?.path)?.source
                            } else { assets::read_bounded(&assets::inside(&m.root,&form.source)?)? };
                            crate::sequence::catalog_source(&bytes,Some(profile))
                        })());
                    }
                }
                if let Some(selection)=&settings.source_selection { ui.text(format!("Source profile: {}; song ID {:?}; ordinal {:?}",selection.profile.id(),selection.song_id,selection.song_index)); }
                if let Some(catalog)=&form.sequence_catalog {
                    match catalog {
                        Err(error)=>ui.text_wrapped(error),
                        Ok(catalog)=>{
                            ui.text(format!("Detected: {}; {} independent songs",catalog.profile.map(|p|p.id()).unwrap_or("Standard MIDI"),catalog.songs.len()));
                            if let Some(profile)=catalog.profile {
                                let source_song_menu=ui.begin_combo("Source song","Choose an explicit song");
                                #[cfg(test)]
                                interaction::track(ui, "source-song-combo");
                                if let Some(_menu)=source_song_menu {
                                    for song in &catalog.songs {
                                        let label=format!("ID {} / ordinal {}: {:.3} s, {} events",song.id,song.ordinal,song.duration_micros as f64/1_000_000.,song.events);
                                        if ui.selectable(label) {
                                            let converted = profile == crate::sequence::SourceProfile::ConvertedSeqLe32V1;
                                            settings.source_selection=Some(crate::sequence::SourceSelection {schema_version:1,profile,
                                                song_id:if converted {None}else{Some(song.id)},
                                                song_index:if converted {Some(song.ordinal)}else{None},
                                                record_hash:if converted {Some(song.record_hash.clone())}else{None},extra:Default::default()});
                                        }
                                        #[cfg(test)]
                                        match song.ordinal {
                                            0 => interaction::track(ui, "source-song-first"),
                                            1 => interaction::track(ui, "source-song-second"),
                                            _ => {}
                                        }
                                    }
                                }
                                ui.text_wrapped("Each SEP entry is an independent song. Reimport retains Sony IDs; changed converted entries require explicit reselection. Unsupported Sony semantics cannot be ignored.");
                            }
                        }
                    }
                }
                let roles = [crate::audio_import::AudioRole::Sfx, crate::audio_import::AudioRole::Music, crate::audio_import::AudioRole::Ambience, crate::audio_import::AudioRole::Dialogue];
                let mut role = roles.iter().position(|r| *r == settings.role).unwrap_or(1);
                if ui.combo_simple_string("Role", &mut role, &["SFX", "Music", "Ambience", "Dialogue"]) { settings.role = roles[role]; }
                crate::music_conversion_ui::help(ui,"role","Selects the audio mixer role. It is independent of event loading and instrument-bank residency.");
                load_mode(ui, "Event Load Mode", &mut settings.load_mode);
                crate::music_conversion_ui::help(ui,"event-load","Resident loads sequence events in RAM; Auto currently chooses Resident. Stream uses the target event-stream path and requires its build prerequisites. Instrument samples have separate bank residency.");
                ui.text_wrapped("Auto resolves to Resident events. Bank residency is selected independently in the SoundBank.");
                bank_selector(ui, "SoundBank", &mut settings.sound_bank, &m.index, "Project Default SoundBank");
                crate::music_conversion_ui::help(ui,"sound-bank","MIDI stores notes and instrument numbers, not recordings. Select a SoundFont library to resolve bank, program and drum-key mappings, or use the project default.");
                if ui.button("Install reference instrument library") { install_reference = true; }
                crate::music_conversion_ui::help(ui,"reference-library","Installs the bundled MIT-licensed FluidR3Mono GM 2.315 source and SoundBank in assets/AudioLibraries, then selects it for this sequence. Existing different files are never overwritten. The converter selects the required instruments automatically.");
                if settings.source_selection.is_none() {
                    let mut profile = usize::from(settings.midi_profile == crate::midi::MidiProfile::MusicalV2);
                    if ui.combo_simple_string("MIDI interpretation", &mut profile, &["Legacy v1", "Musical v2"]) {
                        settings.midi_profile = if profile == 0 { crate::midi::MidiProfile::LegacyV1 } else { crate::midi::MidiProfile::MusicalV2 };
                    }
                    crate::music_conversion_ui::help(ui,"midi-profile","Legacy preserves the original fixed bend range and unsupported-event policy. Musical v2 interprets source RPN tuning, pitch-bend sensitivity and bank selection. Applying this choice recooks the original MIDI and preserves its asset identity.");
                }
                let modes = [crate::sequence::LoopMode::Off, crate::sequence::LoopMode::Whole, crate::sequence::LoopMode::Markers];
                let mut mode = modes.iter().position(|v| *v == settings.loop_mode).unwrap_or(0);
                if ui.combo_simple_string("Loop", &mut mode, &["Off", "Whole", "MIDI markers"]) { settings.loop_mode = modes[mode]; }
                crate::music_conversion_ui::help(ui,"song-loop","Off plays once. Whole repeats the complete song. MIDI markers uses explicit loop markers in the source. This is independent of each instrument's sustain loop.");
                let mut automatic = settings.voice_limit.is_none();
                if ui.checkbox("Auto voice limit (16)", &mut automatic) { settings.voice_limit = if automatic { None } else { Some(16) }; }
                crate::music_conversion_ui::help(ui,"auto-voices","Auto permits 16 physical music voices. The PSX has 24 shared voices; layered instruments can use several for one MIDI note. Reserve capacity for SFX.");
                if let Some(limit) = &mut settings.voice_limit {
                    let mut value = i32::from(*limit);
                    if ui.input_int("Voice limit", &mut value).build() { *limit = value.clamp(1, 24) as u16; }
                    crate::music_conversion_ui::help(ui,"voice-limit","Physical music-voice budget, from 1 to 24 on PSX. Too few voices can cause audible stealing. The tested Ironwood recipe uses 21 with an explicit 450 ms instrument-release cap.");
                }
                let legacy_exception = settings.source_selection.is_none() && settings.midi_profile == crate::midi::MidiProfile::LegacyV1;
                ui.disabled(!legacy_exception, || { ui.checkbox("Ignore reported unsupported MIDI events", &mut settings.ignore_unsupported); });
                crate::music_conversion_ui::help(ui,"unsupported","Legacy-only explicit exception. Musical v2 requires meaningful unsupported operations to be resolved instead of silently discarding them. Review the import report for every adaptation.");
                ui.text_wrapped("Ignoring unsupported events is a Legacy v1 exception; Musical v2 requires unresolved operations to be fixed in the source. Instruments come from the assigned SoundBank. Musical v2 uses source RPN pitch settings; Legacy retains the fixed ±2-semitone range.");
                if let Err(error) = crate::sequence::resolve_bank(&m.root, settings, &m.index) { ui.text_wrapped(error); }
                crate::music_conversion_ui::controls(ui, settings, &m.root, &m.index, &form.source,
                    if form.snapshot { form.existing.as_ref() } else { None }, &mut m.music_conversion);
            }
            if let Some(bank) = &mut form.bank {
                if bank.library.is_some() {
                    ui.text("SoundFont instrument library");
                    ui.text_wrapped("Import preserves the complete SF2/SF3 source in one SoundBank. Instrument and percussion mappings are resolved from this library when converting a MusicSequence.");
                    load_mode(ui, "Sample Load Mode", &mut bank.load_mode);
                    ui.input_text_multiline("Provenance / license", &mut bank.provenance, [-1., 58.]).build();
                } else if bank.imported.is_some() {
                    ui.text("Imported Sony SoundBank");
                    ui.text_wrapped("Source must be a complete VAB or the VH header. For split files choose its VB explicitly. File contents determine the profile; no filename pairing is inferred.");
                    ui.disabled(form.snapshot,||{ui.input_text("VB companion (empty for complete VAB)",&mut form.bank_companion).build();});
                    load_mode(ui,"Sample Load Mode",&mut bank.load_mode);
                    ui.input_text_multiline("Provenance / license",&mut bank.provenance,[-1.,58.]).build();
                    ui.text_wrapped(crate::bank_compat::PLAYBACK_BLOCKER);
                    ui.text_wrapped("Import preserves every tone, layer, raw parameter, ADPCM block and source part. Inspect the saved SoundBank for its complete source and compatibility report.");
                } else {
                    ui.child_window("sound-bank-zones").size([0., (ui.content_region_avail()[1] - 95.).max(180.)]).build(|| bank_editor(ui, bank, &m.index));
                }
            }
            ui.separator();
            if ui.button(if form.existing.is_some() { "Save settings / Reimport" } else { "Import" }) { start = true; }
            #[cfg(test)]
            interaction::track(ui, "portable-import");
            crate::music_conversion_ui::help(ui,"save-import","Validates and saves this recipe with the original source snapshot. Existing asset identity is preserved and derived data is invalidated. Then use Source / PSX Target Preview in Explorer or Inspector.");
        });
        if m.busy { ui.text("Validating the source snapshot..."); }
        if let Some(error) = &m.error { ui.text_wrapped(error); }
    });
    if install_reference { m.start_reference_bank(); }
    if start { m.start_import(); }
    if !open && !m.busy { m.music_conversion.cancel(); m.form = None; }
}
fn load_mode(ui: &Ui, label: &str, value: &mut crate::audio_import::LoadMode) {
    use crate::audio_import::LoadMode;
    let modes = [LoadMode::Auto, LoadMode::Resident, LoadMode::Stream];
    let mut selected = modes.iter().position(|m| *m == *value).unwrap_or(0);
    if ui.combo_simple_string(label, &mut selected, &["Auto", "Resident", "Stream"]) { *value = modes[selected]; }
}
fn midi_byte(ui: &Ui, label: &str, value: &mut u8, min: i32) {
    let mut edit = i32::from(*value);
    if ui.input_int(label, &mut edit).build() { *value = edit.clamp(min, 127) as u8; }
}
fn bank_editor(ui: &Ui, bank: &mut crate::sound_bank::Settings, index: &assets::Index) {
    ui.text("SoundBank");
    load_mode(ui, "Sample Load Mode", &mut bank.load_mode);
    ui.text_wrapped("Auto resolves to Resident. One matching zone per note. Select source AudioClips; their Load Mode does not change bank residency.");
    ui.input_text_multiline("Provenance / license", &mut bank.provenance, [-1., 58.]).build();
    let mut remove_program = None;
    for (pi, program) in bank.programs.iter_mut().enumerate() {
        let _id = ui.push_id(format!("program-{pi}"));
        if !ui.collapsing_header(format!("Program {} / {:?}##mapping", program.program, program.drum_key), imgui::TreeNodeFlags::DEFAULT_OPEN) { continue; }
        midi_byte(ui, "Program (0–127)", &mut program.program, 0);
        let mut drums = program.drum_key.is_some();
        if ui.checkbox("Channel 10 percussion mapping", &mut drums) { program.drum_key = drums.then_some(36); }
        if let Some(key) = &mut program.drum_key { midi_byte(ui, "Drum key", key, 0); }
        if ui.small_button("Remove mapping") { remove_program = Some(pi); }
        let mut remove_zone = None;
        for (zi, zone) in program.zones.iter_mut().enumerate() {
            let _zone_id = ui.push_id(format!("zone-{zi}"));
            if !ui.collapsing_header(format!("Zone {}", zi + 1), imgui::TreeNodeFlags::DEFAULT_OPEN) { continue; }
            let name = index.resolve(zone.sample).map(|r| r.path.file_stem().unwrap_or_default().to_string_lossy().into_owned()).unwrap_or_else(|_| "Choose an AudioClip".into());
            if let Some(_combo) = ui.begin_combo("Sample", name) {
                for r in index.usable().filter(|r| r.meta.kind == assets::Kind::AudioClip) {
                    if ui.selectable(format!("{}##{}", r.path.file_stem().unwrap_or_default().to_string_lossy(), r.meta.id)) { zone.sample = r.meta.id; }
                }
            }
            midi_byte(ui, "Lowest key", &mut zone.key_range[0], 0); midi_byte(ui, "Highest key", &mut zone.key_range[1], 0);
            midi_byte(ui, "Lowest velocity", &mut zone.velocity_range[0], 1); midi_byte(ui, "Highest velocity", &mut zone.velocity_range[1], 1);
            midi_byte(ui, "Root key", &mut zone.root_key, 0);
            crate::gui::Drag::new("Fine tune (cents)").range(-100., 100.).speed(0.1).build(ui, &mut zone.fine_tune_cents);
            crate::gui::Drag::new("Gain").range(0., 4.).speed(0.01).build(ui, &mut zone.gain);
            crate::gui::Drag::new("Pan").range(-1., 1.).speed(0.01).build(ui, &mut zone.pan);
            for (label, value) in [("Attack (ms)", &mut zone.envelope.attack_ms), ("Decay (ms)", &mut zone.envelope.decay_ms), ("Release (ms)", &mut zone.envelope.release_ms)] {
                crate::gui::Drag::new(label).range(0., 60_000.).speed(1.).build(ui, value);
            }
            crate::gui::Drag::new("Sustain level").range(0., 1.).speed(0.01).build(ui, &mut zone.envelope.sustain);
            let mut looping = zone.sample_loop.is_some();
            if ui.checkbox("Sample loop (original source frames)", &mut looping) { zone.sample_loop = looping.then_some([0, 28]); }
            if let Some(region) = &mut zone.sample_loop {
                let mut edit = [region[0] as i32, region[1] as i32];
                if ui.input_int2("Start / exclusive end", &mut edit).build() { *region = [edit[0].max(0) as u32, edit[1].max(0) as u32]; }
            }
            if ui.small_button("Remove zone") { remove_zone = Some(zi); }
        }
        if let Some(i) = remove_zone { program.zones.remove(i); }
        if ui.small_button("Add zone") { program.zones.push(crate::sound_bank::Zone::new(uuid::Uuid::nil())); }
    }
    if let Some(i) = remove_program { bank.programs.remove(i); }
    if ui.button("Add program / drum mapping") {
        let program = (0..=127).find(|p| !bank.programs.iter().any(|b| b.program == *p && b.drum_key.is_none())).unwrap_or(0);
        bank.programs.push(crate::sound_bank::Program { program, drum_key: None, zones: vec![crate::sound_bank::Zone::new(uuid::Uuid::nil())], extra: Default::default() });
    }
    if let Err(error) = bank.validate() { ui.text_wrapped(error); }
}

pub fn audio_settings(ui: &Ui, settings: &mut crate::audio_import::Settings) -> bool {
    use crate::audio_import::{AudioRole, LoadMode, Quality};
    let before = settings.clone();
    ui.text("Role");
    for role in [
        AudioRole::Sfx,
        AudioRole::Music,
        AudioRole::Ambience,
        AudioRole::Dialogue,
    ] {
        if ui.radio_button_bool(format!("{role:?}"), settings.role == role) {
            settings.role = role;
        }
        ui.same_line();
    }
    ui.new_line();
    ui.text("Load Mode");
    for mode in [LoadMode::Auto, LoadMode::Resident, LoadMode::Stream] {
        if ui.radio_button_bool(format!("{mode:?}"), settings.load_mode == mode) {
            settings.load_mode = mode;
        }
        ui.same_line();
    }
    ui.new_line();
    ui.text("Quality");
    for quality in [
        Quality::Low,
        Quality::Medium,
        Quality::High,
        Quality::Custom,
    ] {
        if ui.radio_button_bool(format!("{quality:?}"), settings.quality == quality) {
            settings.quality = quality;
        }
        ui.same_line();
    }
    ui.new_line();
    for channels in [1, 2] {
        if ui.radio_button_bool(
            if channels == 1 { "Mono" } else { "Stereo" },
            settings.channels == channels,
        ) {
            settings.channels = channels;
        }
        ui.same_line();
    }
    ui.new_line();
    if settings.quality == Quality::Custom
        && ui.collapsing_header(
            "Advanced Target Overrides / PSX",
            imgui::TreeNodeFlags::empty(),
        )
    {
        let rates: &[u32] = if settings.is_streamed() {
            &[18900, 37800]
        } else {
            &[11025, 22050, 44100]
        };
        for &rate in rates {
            if ui.radio_button_bool(format!("{rate} Hz"), settings.sample_rate == rate) {
                settings.sample_rate = rate;
            }
            ui.same_line();
        }
        ui.new_line();
    }
    let result = settings.target_result();
    ui.text_wrapped(format!(
        "PSX: {} / {} Hz",
        result.representation, result.sample_rate
    ));
    if settings.load_mode == LoadMode::Auto {
        ui.text_wrapped("Auto profile v3: SFX is resident; Music, Ambience and Dialogue stream. Budget overflow fails without changing this choice.");
    }
    if let Some(error) = result.error {
        ui.text_wrapped(error);
    }
    *settings != before
}

fn texture_inspector(ui: &Ui, m: &mut Manager, r: &assets::Record) {
    ui.text_wrapped(&r.meta.source);
    ui.text_wrapped(format!("Texture / {}", r.meta.id));
    match assets::Package::load(&r.path).and_then(|p| crate::texture::decode(&p.source)) {
        Ok(t) => {
            ui.text(format!(
                "{} x {} / 8-bit indexed / {} bytes + 512-byte palette",
                t.width,
                t.height,
                t.words.len() * 2
            ));
            let origin = ui.cursor_screen_pos();
            let scale = (256. / f32::from(t.width.max(t.height))).max(1.);
            let step = (t.width.max(t.height) as usize).div_ceil(64).max(1);
            let draw = ui.get_window_draw_list();
            for y in (0..t.height as usize).step_by(step) {
                for x in (0..t.width as usize).step_by(step) {
                    let i = (y * t.width as usize + x) * 4;
                    let c = &t.rgba[i..i + 4];
                    let color = if c[3] == 0 {
                        let v = if (x / step + y / step).is_multiple_of(2) {
                            70
                        } else {
                            100
                        };
                        imgui::ImColor32::from_rgb(v, v, v)
                    } else {
                        imgui::ImColor32::from_rgb(c[0], c[1], c[2])
                    };
                    draw.add_rect(
                        [origin[0] + x as f32 * scale, origin[1] + y as f32 * scale],
                        [
                            origin[0] + (x + step).min(t.width as usize) as f32 * scale,
                            origin[1] + (y + step).min(t.height as usize) as f32 * scale,
                        ],
                        color,
                    )
                    .filled(true)
                    .build();
                }
            }
            ui.dummy([f32::from(t.width) * scale, f32::from(t.height) * scale]);
        }
        Err(e) => ui.text_wrapped(e),
    }
    ui.text_wrapped("Nearest sampling. Alpha < 128 cuts out; blend mode is controlled per material. VRAM layout is validated on export against framebuffers and font.");
    if ui.button("Reimport / Locate source") {
        m.begin_reimport(r, false);
    }
    if ui.button("Rebuild from stored PNG") {
        m.begin_reimport(r, true);
    }
}

pub fn component(ui: &Ui, e: &Editor, entity: &mut crate::scene::Entity) {
    let Some(audio) = &mut entity.audio else {
        return;
    };
    ui.separator();
    if !crate::gui::heading(ui, "Audio Source") {
        return;
    }
    let label = audio
        .clip
        .map(|id| {
            e.assets
                .index
                .resolve(id)
                .map(|r| assets::path_string(&e.root, &r.path))
                .unwrap_or_else(|_| format!("Missing / conflicting {id}"))
        })
        .unwrap_or_else(|| "None".into());
    if let Some(_combo) = ui.begin_combo(crate::gui::field(ui, "Playable Audio"), label) {
        if ui.selectable("None") {
            audio.clip = None;
        }
        for record in e
            .assets
            .index
            .usable()
            .filter(|r| matches!(r.meta.kind, assets::Kind::AudioClip | assets::Kind::MusicSequence))
        {
            if ui.selectable(assets::path_string(&e.root, &record.path)) {
                audio.clip = Some(record.meta.id);
                if record.meta.settings.audio().is_ok_and(|s| s.is_streamed()) {
                    audio.pitch = 1.;
                }
            }
        }
    }
    crate::gui::Drag::new(crate::gui::field(ui, "Volume"))
        .range(0., 1.)
        .speed(0.01)
        .build(ui, &mut audio.volume);
    let music = audio
        .clip
        .and_then(|id| e.assets.index.resolve(id).ok())
        .is_some_and(|r| r.meta.settings.audio().is_ok_and(|s| s.is_streamed()));
    if music {
        audio.pitch = 1.;
    }
    ui.disabled(music, || {
        crate::gui::Drag::new(crate::gui::field(ui, "Pitch"))
            .range(0.25, 4.)
            .speed(0.01)
            .build(ui, &mut audio.pitch);
    });
    if music {
        crate::gui::muted(ui, "XA: one CD stream; pitch is fixed at 1.0.");
    }
    ui.checkbox("Play on start", &mut audio.play_on_start);
    crate::gui::Drag::new(crate::gui::field(ui, "Priority"))
        .range(0, 255)
        .build(ui, &mut audio.priority);
    crate::gui::muted(
        ui,
        if music {
            "Higher priority can replace the current BGM."
        } else {
            "Higher priority wins when all 24 SPU voices are busy."
        },
    );
    if ui.small_button("Remove Audio Source") {
        entity.audio = None;
    }
}

#[cfg(test)]
pub mod interaction {
    use super::*;
    use std::{
        cell::RefCell,
        collections::BTreeMap,
        time::{Duration, Instant},
    };
    thread_local! {static BUTTONS:RefCell<BTreeMap<&'static str,[f32;2]>>=const {RefCell::new(BTreeMap::new())};}
    pub fn track(ui: &Ui, key: &'static str) {
        let a = ui.item_rect_min();
        let b = ui.item_rect_max();
        BUTTONS.with(|buttons| {
            buttons
                .borrow_mut()
                .insert(key, [(a[0] + b[0]) / 2., (a[1] + b[1]) / 2.])
        });
    }
    pub fn verify(context: &mut imgui::Context) {
        let root = std::env::temp_dir().join(format!("epok-import-ui-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(root.join("assets")).unwrap();
        std::fs::write(
            root.join("assets/tone.wav"),
            crate::audio_import::test_wav(),
        )
        .unwrap();
        let mut e = Editor::new(root.clone());
        e.auto_build = false;
        e.assets.index = assets::scan(&root, &mut Default::default());
        let source = e.assets.index.sources.values().next().unwrap();
        e.assets.pending.push(crate::asset_manager::Pending {
            source: source.path.clone(),
            hash: source.hash.clone(),
            existing: None,
            status: Status::Pending,
        });
        e.assets.window = true;
        e.assets.focus_tab = Some(0);
        fn frame(ctx: &mut imgui::Context, e: &mut Editor) {
            windows(ctx.frame(), e);
            ctx.render();
        }
        fn click(ctx: &mut imgui::Context, e: &mut Editor, key: &str) {
            let position = BUTTONS.with(|buttons| buttons.borrow()[key]);
            ctx.io_mut().add_mouse_pos_event(position);
            frame(ctx, e);
            ctx.io_mut()
                .add_mouse_button_event(imgui::MouseButton::Left, true);
            frame(ctx, e);
            ctx.io_mut()
                .add_mouse_button_event(imgui::MouseButton::Left, false);
            frame(ctx, e);
        }
        frame(context, &mut e);
        frame(context, &mut e);
        click(context, &mut e, "pending");
        assert!(
            e.assets.form.is_some(),
            "Inbox click must open import settings"
        );
        frame(context, &mut e);
        click(context, &mut e, "import");
        assert!(
            e.assets.busy,
            "Modal click must start the background importer"
        );
        let deadline = Instant::now() + Duration::from_secs(8);
        while e.assets.form.is_some() {
            e.assets.tick();
            frame(context, &mut e);
            assert!(Instant::now() < deadline, "{:?}", e.assets.error);
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(e.assets.selected.is_some());
        assert!(root.join("assets/tone.epokasset").exists());
        e.assets.window = false;
        frame(context, &mut e);
        drop(e);
        assert!(
            root.starts_with(std::env::temp_dir())
                && root
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("epok-import-ui-")
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    pub fn verify_sequence(context: &mut imgui::Context) {
        let root = crate::workspace::tests::temp("midi-import-ui");
        let (_, bank) = crate::sequence_preview::tests::fixture(&root);
        std::fs::write(root.join("assets/song.mid"), crate::midi::fixture()).unwrap();
        let mut e = Editor::new(root.clone()); e.auto_build = false;
        e.assets.index = assets::scan(&root, &mut Default::default());
        let source = &e.assets.index.sources["assets/song.mid"];
        let pending = crate::asset_manager::Pending { source: source.path.clone(), hash: source.hash.clone(), existing: None, status: Status::Pending };
        e.assets.begin_pending(&pending);
        fn frame(ctx: &mut imgui::Context, e: &mut Editor) { windows(ctx.frame(), e); ctx.render(); }
        fn click(ctx: &mut imgui::Context, e: &mut Editor, key: &str) {
            let position = BUTTONS.with(|buttons| buttons.borrow()[key]);
            ctx.io_mut().add_mouse_pos_event(position); frame(ctx, e);
            ctx.io_mut().add_mouse_button_event(imgui::MouseButton::Left, true); frame(ctx, e);
            ctx.io_mut().add_mouse_button_event(imgui::MouseButton::Left, false); frame(ctx, e);
        }
        frame(context, &mut e); frame(context, &mut e);
        assert!(e.assets.form.as_ref().unwrap().sequence.as_ref().unwrap().sound_bank.is_none());
        click(context, &mut e, "bank-selector"); frame(context, &mut e);
        click(context, &mut e, "choose-bank");
        assert_eq!(e.assets.form.as_ref().unwrap().sequence.as_ref().unwrap().sound_bank, Some(bank));
        click(context, &mut e, "portable-import");
        assert!(e.assets.busy, "Actual Import button must start the sequence transaction");
        let deadline = Instant::now() + Duration::from_secs(10);
        while e.assets.form.is_some() {
            e.assets.tick(); frame(context, &mut e);
            assert!(Instant::now() < deadline, "{:?}", e.assets.error);
            std::thread::sleep(Duration::from_millis(10));
        }
        let package = assets::Package::load(&root.join("assets/song.epokasset")).unwrap();
        assert_eq!(package.meta.kind, assets::Kind::MusicSequence);
        assert_eq!(package.meta.settings.sequence().unwrap().sound_bank, Some(bank));
        assert_eq!(package.source, crate::midi::fixture());
    }

    #[test]
    #[ignore = "Owns an ImGui context; run serially for the actual SoundFont library importer"]
    fn soundfont_library_import_dialog_preserves_one_authoritative_bank() {
        let mut context = imgui::Context::create();
        context.set_ini_filename(None);
        context.io_mut().display_size = [1440., 1000.];
        context.io_mut().delta_time = 1. / 60.;
        context.fonts().add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
        context.fonts().build_rgba32_texture();
        BUTTONS.with(|buttons| buttons.borrow_mut().clear());
        let root = crate::workspace::tests::temp("soundfont-import-ui");
        std::fs::create_dir_all(root.join("assets")).unwrap();
        let original = crate::sf2::fixture();
        std::fs::write(root.join("assets/library.sf2"), &original).unwrap();
        let mut e = Editor::new(root.clone()); e.auto_build = false;
        e.assets.index = assets::scan(&root, &mut Default::default());
        let source = &e.assets.index.sources["assets/library.sf2"];
        let pending = crate::asset_manager::Pending { source: source.path.clone(), hash: source.hash.clone(), existing: None, status: Status::Pending };
        e.assets.begin_pending(&pending);
        fn frame(ctx: &mut imgui::Context, editor: &mut Editor) { windows(ctx.frame(), editor); ctx.render(); }
        frame(&mut context, &mut e); frame(&mut context, &mut e);
        let form = e.assets.form.as_ref().unwrap();
        assert!(form.sequence.is_none());
        assert!(form.bank.as_ref().unwrap().library.is_some());
        assert!(form.bank.as_ref().unwrap().imported.is_none());
        let position = BUTTONS.with(|buttons| buttons.borrow()["portable-import"]);
        context.io_mut().add_mouse_pos_event(position); frame(&mut context, &mut e);
        context.io_mut().add_mouse_button_event(imgui::MouseButton::Left, true); frame(&mut context, &mut e);
        context.io_mut().add_mouse_button_event(imgui::MouseButton::Left, false); frame(&mut context, &mut e);
        assert!(e.assets.busy, "Actual Import control starts the worker");
        let deadline = Instant::now() + Duration::from_secs(10);
        while e.assets.busy {
            e.assets.tick(); frame(&mut context, &mut e);
            assert!(Instant::now() < deadline, "{:?}", e.assets.error);
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(e.assets.form.is_none(), "{:?}", e.assets.error);
        let mut cache = Default::default();
        let index = assets::scan(&root, &mut cache);
        assert_eq!(index.usable().count(), 1, "embedded samples never become AudioClip assets");
        let record = index.usable().next().unwrap();
        let package = assets::Package::load(&record.path).unwrap();
        assert_eq!(package.meta.kind, assets::Kind::SoundBank);
        assert_eq!(package.source, original);
        assert_eq!(e.assets.selected, Some(package.meta.id));
        e.assets.begin_reimport(record, true);
        assert_eq!(e.assets.form.as_ref().unwrap().bank.as_ref().unwrap(), package.meta.settings.sound_bank().unwrap());
        assert!(!root.join(".epok/imported").exists(), "authoring import must not cook PSX samples");
    }

    #[test]
    #[ignore = "Owns an ImGui context; run explicitly and serially for real SEP selection/import controls"]
    fn sony_sep_import_dialog_requires_and_persists_explicit_song() {
        let mut context = crate::gui::tests::imgui_context();
        context.set_ini_filename(None);
        context.io_mut().display_size = [1440., 1000.];
        context.io_mut().delta_time = 1. / 60.;
        context.fonts().add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
        context.fonts().build_rgba32_texture();
        BUTTONS.with(|buttons| buttons.borrow_mut().clear());

        let root = crate::workspace::tests::temp("sony-sep-import-ui");
        let (_, bank) = crate::sequence_preview::tests::fixture(&root);
        // Original synthetic Sony SEP: two independent songs with source IDs 4/9.
        let mut original = b"pQES\0\0".to_vec();
        for (id, key) in [(4_u16, 60_u8), (9_u16, 72_u8)] {
            let events = [0, 0x90, key, 100, 96, 0x80, key, 0, 0, 0xff, 0x2f, 0];
            original.extend(id.to_be_bytes());
            original.extend([0, 96, 7, 0xa1, 0x20, 4, 2]);
            original.extend((events.len() as u32).to_be_bytes());
            original.extend(events);
        }
        // SEQ-v1 and SEP-v0/song-ID-1 overlap in their first eight bytes.
        // Explicit profile selection still validates the entire structure.
        let mut overlapping_prefix = original.clone();
        overlapping_prefix[6..8].copy_from_slice(&1_u16.to_be_bytes());
        assert_eq!(crate::sequence::catalog_source(&overlapping_prefix, None).unwrap().profile, Some(crate::sequence::SourceProfile::SonySepV0));
        assert!(crate::sequence::catalog_source(&overlapping_prefix, Some(crate::sequence::SourceProfile::SonySeqV1)).is_err());
        assert_eq!(crate::sequence::catalog_source(&overlapping_prefix, Some(crate::sequence::SourceProfile::SonySepV0)).unwrap().songs.len(), 2);
        std::fs::write(root.join("assets/two-songs.sep"), &original).unwrap();
        let destination = root.join("assets/two-songs.epokasset");
        let mut e = Editor::new(root.clone()); e.auto_build = false;
        e.assets.index = assets::scan(&root, &mut Default::default());
        let source = &e.assets.index.sources["assets/two-songs.sep"];
        let pending = crate::asset_manager::Pending { source: source.path.clone(), hash: source.hash.clone(), existing: None, status: Status::Pending };
        e.assets.begin_pending(&pending);

        fn frame(ctx: &mut imgui::Context, editor: &mut Editor) { windows(ctx.frame(), editor); ctx.render(); }
        fn click(ctx: &mut imgui::Context, editor: &mut Editor, key: &str) {
            let position = BUTTONS.with(|buttons| *buttons.borrow().get(key).unwrap_or_else(|| panic!("Visible control {key} was not rendered")));
            ctx.io_mut().add_mouse_pos_event(position); frame(ctx, editor);
            ctx.io_mut().add_mouse_button_event(imgui::MouseButton::Left, true); frame(ctx, editor);
            ctx.io_mut().add_mouse_button_event(imgui::MouseButton::Left, false); frame(ctx, editor);
        }
        fn settle(ctx: &mut imgui::Context, editor: &mut Editor) {
            let deadline = Instant::now() + Duration::from_secs(10);
            while editor.assets.busy {
                editor.assets.tick(); frame(ctx, editor);
                assert!(Instant::now() < deadline, "{:?}", editor.assets.error);
                std::thread::sleep(Duration::from_millis(10));
            }
        }
        frame(&mut context, &mut e); frame(&mut context, &mut e);
        assert!(e.assets.form.as_ref().unwrap().sequence.as_ref().unwrap().source_selection.is_none());
        click(&mut context, &mut e, "bank-selector"); frame(&mut context, &mut e);
        click(&mut context, &mut e, "choose-bank");
        assert_eq!(e.assets.form.as_ref().unwrap().sequence.as_ref().unwrap().sound_bank, Some(bank));

        click(&mut context, &mut e, "portable-import");
        assert!(e.assets.busy, "Real Import button must start the validating worker");
        settle(&mut context, &mut e);
        assert!(!destination.exists(), "Unselected SEP must not publish an authoritative asset");
        assert!(e.assets.form.is_some());
        assert!(e.assets.error.as_deref().is_some_and(|error| error.contains("explicitly")), "{:?}", e.assets.error);

        click(&mut context, &mut e, "inspect-sequence-source");
        assert_eq!(e.assets.form.as_ref().unwrap().sequence_catalog.as_ref().unwrap().as_ref().unwrap().songs.len(), 2);
        click(&mut context, &mut e, "source-song-combo"); frame(&mut context, &mut e);
        click(&mut context, &mut e, "source-song-second");
        let selection = e.assets.form.as_ref().unwrap().sequence.as_ref().unwrap().source_selection.as_ref().unwrap();
        assert_eq!(selection.profile, crate::sequence::SourceProfile::SonySepV0);
        assert_eq!(selection.song_id, Some(9));
        assert!(selection.song_index.is_none());
        frame(&mut context, &mut e);
        click(&mut context, &mut e, "portable-import");
        assert!(e.assets.busy); settle(&mut context, &mut e);
        assert!(e.assets.form.is_none(), "{:?}", e.assets.error);

        let package = assets::Package::load(&destination).unwrap();
        assert_eq!(package.meta.kind, assets::Kind::MusicSequence);
        assert!(!package.meta.id.is_nil());
        assert_eq!(e.assets.selected, Some(package.meta.id));
        assert_eq!(package.source, original, "Both SEP songs, headers and bytes must survive import");
        let settings = package.meta.settings.sequence().unwrap();
        assert_eq!(settings.sound_bank, Some(bank));
        assert_eq!(settings.source_selection.as_ref().unwrap().song_id, Some(9));
        assert_eq!(crate::sequence::decode_source(&package.source, settings).unwrap().source_events[0].data, [72, 100]);
        assert_eq!(crate::sequence::catalog_source(&package.source, None).unwrap().songs.len(), 2);
        assert!(!root.join(".epok/imported").exists(), "Inspection/import must not cook a target");
    }
}
