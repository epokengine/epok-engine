//! Timing/sound data in the ordinary asset Inspector. Visuals stay on widgets.
use crate::{
    assets::{self, Index, Record},
    dialogue_asset::DialogueStyle,
};
use std::{collections::BTreeMap, path::Path};
use uuid::Uuid;

#[derive(Default)]
pub struct Drafts {
    items: BTreeMap<Uuid, Draft>,
}

pub fn create(e: &mut crate::editor::Editor) -> Result<Uuid, String> {
    if e.playing {
        return Err("Stop Play before creating dialogue data".into());
    }
    for number in 0..10000 {
        let name = if number == 0 {
            "DialogueTiming".into()
        } else {
            format!("DialogueTiming{number}")
        };
        let path = format!("assets/{name}.epokasset");
        if e.root.join(&path).exists() {
            continue;
        }
        let id = crate::dialogue_asset::create_style(&e.root, &path, &Default::default())?;
        e.assets.refresh();
        e.assets.selected = Some(id);
        e.selected_asset = Some(e.root.join(path));
        return Ok(id);
    }
    Err("Choose another dialogue timing asset name".into())
}
struct Draft {
    record: Record,
    seen: String,
    saved: DialogueStyle,
    value: DialogueStyle,
    legacy: bool,
    error: Option<String>,
}
impl Draft {
    fn load(record: &Record) -> Result<Self, String> {
        let bytes = assets::read_bounded(&record.path)?;
        let package = assets::Package::parse(&bytes)?;
        if package.meta.id != record.meta.id || package.meta.kind != assets::Kind::DialogueStyle {
            return Err("Dialogue Style was replaced; refresh the asset index".into());
        }
        let value = DialogueStyle::parse(&package.source)?;
        let document: serde_json::Value = crate::document::from_slice(&package.source)?;
        let mut source = record.clone();
        source.revision = assets::hash(&bytes);
        Ok(Self {
            record: source,
            seen: record.revision.clone(),
            saved: value.clone(),
            value,
            legacy: document.get("version").and_then(serde_json::Value::as_u64) == Some(1),
            error: None,
        })
    }
    fn dirty(&self) -> bool {
        self.legacy || self.saved != self.value
    }
    fn save(&mut self, root: &Path, index: &Index) -> Result<(), String> {
        self.record.revision =
            crate::dialogue_asset::save_style(root, &self.record, &self.value, index)?;
        self.saved = self.value.clone();
        self.legacy = false;
        self.error = None;
        Ok(())
    }
}
pub fn draw(
    ui: &imgui::Ui,
    drafts: &mut Drafts,
    root: &Path,
    record: &Record,
    index: &Index,
) -> bool {
    let slot = match drafts.items.entry(record.meta.id) {
        std::collections::btree_map::Entry::Occupied(slot) => slot.into_mut(),
        std::collections::btree_map::Entry::Vacant(slot) => match Draft::load(record) {
            Ok(draft) => slot.insert(draft),
            Err(error) => {
                ui.text_wrapped(error);
                return false;
            }
        },
    };
    if slot.seen != record.revision {
        if slot.dirty() {
            slot.error=Some("The asset changed on disk. Reload it or keep this draft until the conflict is resolved.".into());
            slot.seen = record.revision.clone();
        } else {
            match Draft::load(record) {
                Ok(fresh) => *slot = fresh,
                Err(error) => slot.error = Some(error),
            }
        }
    }
    ui.separator();
    ui.text("Dialogue timing and sounds");
    crate::dialogue_editor::line_policy(ui, &mut slot.value.line_policy);
    crate::blueprint_editor::record_control(ui, "dialogue-style:policy");
    for (label, value) in [
        ("Open sound", &mut slot.value.open_sound),
        ("Advance sound", &mut slot.value.advance_sound),
        ("Close sound", &mut slot.value.close_sound),
    ] {
        crate::blueprint_refs::asset_inspector(ui, label, value, "AudioClip", index, "None");
    }
    ui.text_wrapped("Edit layout, colors, fonts and images on the widget's controls.");
    if slot.legacy {
        ui.text_wrapped("Saving upgrades this v1 asset to v2 and preserves the complete original package in .epok/migrations/dialogue-style.");
    }
    let mut saved = false;
    ui.disabled(!slot.dirty(), || {
        if ui.button("Save timing and sounds") {
            match slot.save(root, index) {
                Ok(()) => saved = true,
                Err(error) => slot.error = Some(error),
            }
        }
        crate::blueprint_editor::record_control(ui, "dialogue-style:save");
    });
    ui.same_line();
    if ui.button("Reload from disk##dialogue-style") {
        match Draft::load(record) {
            Ok(fresh) => *slot = fresh,
            Err(error) => slot.error = Some(error),
        }
    }
    crate::blueprint_editor::record_control(ui, "dialogue-style:reload");
    if let Some(error) = &slot.error {
        ui.text_wrapped(error);
    }
    saved
}

#[cfg(test)]
mod tests {
    use super::*;
    fn frame(
        ctx: &mut imgui::Context,
        drafts: &mut Drafts,
        root: &Path,
        id: Uuid,
        index: &Index,
    ) -> bool {
        crate::blueprint_editor::clear_controls();
        let ui = ctx.frame();
        let mut saved = false;
        ui.window("Asset Inspector")
            .position([0., 0.], imgui::Condition::Always)
            .size([700., 700.], imgui::Condition::Always)
            .build(|| {
                saved = draw(ui, drafts, root, index.resolve(id).unwrap(), index);
            });
        ctx.render();
        saved
    }
    fn click(
        ctx: &mut imgui::Context,
        drafts: &mut Drafts,
        root: &Path,
        id: Uuid,
        index: &Index,
        field: &str,
    ) {
        frame(ctx, drafts, root, id, index);
        let point = crate::blueprint_editor::control_position(field)
            .unwrap_or_else(|| panic!("Missing {field}"));
        ctx.io_mut().add_mouse_pos_event(point);
        frame(ctx, drafts, root, id, index);
        for down in [true, false] {
            ctx.io_mut()
                .add_mouse_button_event(imgui::MouseButton::Left, down);
            frame(ctx, drafts, root, id, index);
        }
        frame(ctx, drafts, root, id, index);
    }
    #[test]
    fn dialogue_style_inspector_edits_timing_sounds_and_preserves_conflicting_drafts() {
        let root = crate::workspace::tests::temp("dialogue-timing-inspector");
        crate::workspace::create(&root, "Timing Inspector", crate::workspace::Template::Basic)
            .unwrap();
        let id = crate::dialogue_asset::create_style(
            &root,
            "assets/Timing.epokasset",
            &Default::default(),
        )
        .unwrap();
        std::fs::write(
            root.join("assets/tone.wav"),
            crate::audio_import::test_wav(),
        )
        .unwrap();
        let audio = assets::commit(
            assets::prepare(
                &root,
                "assets/tone.wav",
                "assets/Tone.epokasset",
                Default::default(),
                None,
                false,
            )
            .unwrap(),
        )
        .unwrap();
        let other = crate::dialogue_asset::create_style(
            &root,
            "assets/OtherTiming.epokasset",
            &Default::default(),
        )
        .unwrap();
        let mut index = assets::scan(&root, &mut Default::default());
        let mut drafts = Drafts::default();
        let mut ctx = crate::gui::tests::imgui_context();
        ctx.io_mut().display_size = [900., 900.];
        ctx.io_mut().delta_time = 1. / 60.;
        ctx.fonts().build_rgba32_texture();
        click(
            &mut ctx,
            &mut drafts,
            &root,
            id,
            &index,
            "line-policy:automatic",
        );
        assert_eq!(
            drafts.items[&id].value.line_policy.auto_advance_ms,
            Some(1000)
        );
        click(
            &mut ctx,
            &mut drafts,
            &root,
            id,
            &index,
            "asset-field:Open sound",
        );
        click(
            &mut ctx,
            &mut drafts,
            &root,
            id,
            &index,
            &format!("reference-choice:{audio}"),
        );
        assert_eq!(drafts.items[&id].value.open_sound, Some(audio));
        click(
            &mut ctx,
            &mut drafts,
            &root,
            id,
            &index,
            "dialogue-style:save",
        );
        assert!(!drafts.items[&id].dirty());
        let saved = crate::dialogue_asset::style_document(index.resolve(id).unwrap()).unwrap();
        assert_eq!(saved.line_policy.auto_advance_ms, Some(1000));
        assert_eq!(saved.open_sound, Some(audio));
        let source = assets::Package::load(&index.resolve(id).unwrap().path)
            .unwrap()
            .source;
        let json: serde_json::Value = crate::document::from_slice(&source).unwrap();
        assert_eq!(json.as_object().unwrap().len(), 5);
        assert_eq!(json["version"], 2);
        index = assets::scan(&root, &mut Default::default());
        // The author may select another asset and return without losing edits.
        click(
            &mut ctx,
            &mut drafts,
            &root,
            id,
            &index,
            "line-policy:automatic",
        );
        assert_eq!(drafts.items[&id].value.line_policy.auto_advance_ms, None);
        frame(&mut ctx, &mut drafts, &root, other, &index);
        frame(&mut ctx, &mut drafts, &root, id, &index);
        assert_eq!(drafts.items[&id].value.line_policy.auto_advance_ms, None);
        let mut external = saved;
        external.line_policy.milliseconds_per_glyph = 71;
        crate::dialogue_asset::save_style(&root, index.resolve(id).unwrap(), &external, &index)
            .unwrap();
        let external_bytes = assets::read_bounded(&index.resolve(id).unwrap().path).unwrap();
        index = assets::scan(&root, &mut Default::default());
        frame(&mut ctx, &mut drafts, &root, id, &index);
        assert!(
            drafts.items[&id]
                .error
                .as_ref()
                .unwrap()
                .contains("changed on disk")
        );
        click(
            &mut ctx,
            &mut drafts,
            &root,
            id,
            &index,
            "dialogue-style:save",
        );
        assert_eq!(
            assets::read_bounded(&index.resolve(id).unwrap().path).unwrap(),
            external_bytes
        );
        assert!(drafts.items[&id].dirty());
        click(
            &mut ctx,
            &mut drafts,
            &root,
            id,
            &index,
            "dialogue-style:reload",
        );
        assert_eq!(drafts.items[&id].value, external);
        assert!(!drafts.items[&id].dirty());
        std::fs::remove_dir_all(root).unwrap();
    }
}
