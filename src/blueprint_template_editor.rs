//! Typed template controls reuse the scene component inspectors and the same
//! bounded resolver used by placement/cooking; editor reads do not run scripts.
use crate::{
    blueprint::Registry,
    blueprint_asset::{self, BlueprintAsset},
    blueprint_templates::{self as templates, ConstructionOp, ParentOverride, Template},
    scene::Actor,
};
use imgui::Ui;
use std::collections::BTreeMap;
use uuid::Uuid;

pub struct State {
    selected: Option<Uuid>,
    name: String,
    offset: [f32; 3],
    operation: usize,
    ids: BTreeMap<String, String>,
    search: String,
}
impl Default for State {
    fn default() -> Self {
        Self {
            selected: None,
            name: "Child".into(),
            offset: [0.; 3],
            operation: 0,
            ids: BTreeMap::new(),
            search: String::new(),
        }
    }
}
fn inherited(doc: &BlueprintAsset, registry: &Registry) -> Result<Vec<Template>, String> {
    let mut layers = vec![];
    if let Some(parent) = registry.classes.get(&doc.parent) {
        for class in registry.ancestry(&parent.cpp_name) {
            if class.provider.id == "blueprint" {
                layers.push(blueprint_asset::load(&class.source.file)?.template);
            }
        }
    }
    Ok(layers)
}
fn transaction<T>(
    doc: &mut BlueprintAsset,
    registry: &Registry,
    edit: impl FnOnce(&mut BlueprintAsset) -> Result<T, String>,
) -> Result<T, String> {
    let before = doc.template.clone();
    let result = edit(doc).and_then(|value| {
        if doc.template != before {
            let mut layers = inherited(doc, registry)?;
            layers.push(doc.template.clone());
            templates::resolve(&layers.iter().collect::<Vec<_>>())?;
        }
        Ok(value)
    });
    if result.is_err() {
        doc.template = before;
    }
    result
}
pub fn browser(
    ui: &Ui,
    doc: &mut BlueprintAsset,
    registry: &Registry,
    state: &mut State,
) -> Result<bool, String> {
    let selected = state.selected;
    let result = transaction(doc, registry, |doc| browser_inner(ui, doc, registry, state));
    if result.is_err() {
        state.selected = selected;
    }
    result
}
fn browser_inner(
    ui: &Ui,
    doc: &mut BlueprintAsset,
    registry: &Registry,
    state: &mut State,
) -> Result<bool, String> {
    let mut layers = inherited(doc, registry)?;
    layers.push(doc.template.clone());
    for layer in &mut layers {
        layer.construction.clear();
    }
    let resolved = templates::resolve(&layers.iter().collect::<Vec<_>>())?;
    let mut activated = false;
    if super::button(ui, "\u{ea60} Add##component") {
        ui.open_popup("bp-components-add");
    }
    ui.same_line();
    ui.set_next_item_width(-1.);
    ui.input_text("##component-search", &mut state.search)
        .hint("Search components")
        .build();
    super::record_control(ui, "bp-component-search");
    if let Some(_popup) = ui.begin_popup("bp-components-add") {
        if resolved.actors.is_empty() {
            if ui.selectable("Actor root") {
                create_root(&mut doc.template, &doc.name)?;
                state.selected = doc.template.actors.first().map(|e| e.entity.id);
                activated = true;
            }
        } else {
            let selected = state.selected.or_else(|| resolved.root()).unwrap();
            ui.input_text("Child name", &mut state.name).build();
            if ui.selectable("Add child Actor") {
                if state.name.trim().is_empty() {
                    return Err("Child name cannot be empty.".into());
                }
                if resolved.actors.len() >= templates::MAX_ENTITIES {
                    return Err("Actor template capacity reached (32).".into());
                }
                state.selected = Some(doc.template.add_child(selected, &state.name));
                activated = true;
            }
            if let Some(item) = resolved
                .actors
                .iter()
                .find(|item| item.entity.id == selected)
            {
                ui.separator();
                let mut entity = item.entity.clone();
                for name in [
                    "Collider",
                    "Audio Source",
                    "Light",
                    "Canvas",
                    "HUD Image",
                    "HUD Text",
                    "HUD Progress",
                    "HUD Layout Element",
                    "HUD Layout Container",
                    "Blob Shadow",
                ] {
                    if super::choose(ui, name) {
                        let parent = item
                            .parent
                            .and_then(|id| resolved.actors.iter().find(|item| item.entity.id == id))
                            .map(|item| &item.entity);
                        add_component(&mut entity, parent, name)?;
                        state.selected = Some(selected);
                        activated = true;
                    }
                }
                if entity != item.entity {
                    apply_entity_changes(&mut doc.template, &item.entity, &entity)?;
                }
            }
        }
    }
    ui.separator();
    if resolved.actors.is_empty() {
        ui.text_disabled(format!("\u{eb29} {} (Self)", doc.name));
        ui.text_wrapped("Add an Actor root to author its component composition.");
    }
    fn row(
        ui: &Ui,
        all: &[templates::TemplateEntity],
        parent: Option<Uuid>,
        own: &Template,
        state: &mut State,
        activated: &mut bool,
    ) {
        for item in all.iter().filter(|item| item.parent == parent) {
            let names = component_names(&item.entity);
            let query = state.search.to_lowercase();
            let own_match = item.entity.name.to_lowercase().contains(&query)
                || names
                    .iter()
                    .any(|name| name.to_lowercase().contains(&query));
            // Preserve ancestors while searching so matches keep their hierarchy.
            if !own_match && !descendant_matches(all, item.entity.id, &query) {
                continue;
            }
            let inherited = !own.actors.iter().any(|own| own.entity.id == item.entity.id);
            let label = format!(
                "\u{eb29} {}{}##entity-{}",
                item.entity.name,
                if inherited { " (inherited)" } else { "" },
                item.entity.id
            );
            let expanded = ui
                .tree_node_config(&label)
                .default_open(true)
                .flags(
                    imgui::TreeNodeFlags::OPEN_ON_ARROW
                        | imgui::TreeNodeFlags::DEFAULT_OPEN
                        | imgui::TreeNodeFlags::SPAN_AVAIL_WIDTH
                        | if state.selected == Some(item.entity.id) {
                            imgui::TreeNodeFlags::SELECTED
                        } else {
                            imgui::TreeNodeFlags::empty()
                        },
                )
                .push();
            super::record_control(ui, &format!("bp-component:{}", item.entity.id));
            if ui.is_item_clicked() && !ui.is_item_toggled_open() {
                state.selected = Some(item.entity.id);
                state.ids.clear();
                *activated = true;
            }
            if let Some(_node) = expanded {
                for name in names {
                    if !query.is_empty() && !own_match && !name.to_lowercase().contains(&query) {
                        continue;
                    }
                    if ui.selectable(format!("\u{eae9} {name}##{}-{name}", item.entity.id)) {
                        state.selected = Some(item.entity.id);
                        state.ids.clear();
                        *activated = true;
                    }
                }
                row(ui, all, Some(item.entity.id), own, state, activated);
            }
        }
    }
    row(
        ui,
        &resolved.actors,
        None,
        &doc.template,
        state,
        &mut activated,
    );
    Ok(activated)
}
fn descendant_matches(all: &[templates::TemplateEntity], id: Uuid, query: &str) -> bool {
    all.iter()
        .filter(|child| child.parent == Some(id))
        .any(|child| {
            child.entity.name.to_lowercase().contains(query)
                || component_names(&child.entity)
                    .iter()
                    .any(|name| name.to_lowercase().contains(query))
                || descendant_matches(all, child.entity.id, query)
        })
}
fn add_component(entity: &mut Actor, parent: Option<&Actor>, name: &str) -> Result<(), String> {
    if name == "Canvas" && (parent.is_some() || entity.kind != "Empty" || entity.rect.is_some()) {
        return Err("Canvas can only be added to an Empty template root without RectTransform. Existing components were preserved.".into());
    }
    if matches!(
        name,
        "HUD Image" | "HUD Text" | "HUD Progress" | "HUD Layout Element" | "HUD Layout Container"
    ) && (entity.kind != "Empty"
        || !parent.is_some_and(|parent| parent.canvas.is_some() || parent.rect.is_some()))
    {
        return Err("HUD components need an Empty child of Canvas or RectTransform. Existing components were preserved.".into());
    }
    match name {
        "Collider" => {
            entity.collider.get_or_insert_with(Default::default);
        }
        "Audio Source" => {
            entity.audio.get_or_insert_with(Default::default);
        }
        "Light" => {
            entity.light.get_or_insert_with(Default::default);
        }
        "Canvas" => {
            entity.canvas.get_or_insert_with(Default::default);
        }
        "HUD Image" => {
            entity.image.get_or_insert_with(Default::default);
            entity.rect.get_or_insert_with(Default::default);
        }
        "HUD Text" => {
            entity.text.get_or_insert_with(Default::default);
            entity.rect.get_or_insert_with(Default::default);
        }
        "HUD Progress" => {
            entity.progress.get_or_insert_with(Default::default);
            entity.rect.get_or_insert_with(Default::default);
        }
        "HUD Layout Element" => {
            entity.layout_element.get_or_insert_with(Default::default);
            entity.rect.get_or_insert_with(Default::default);
        }
        "HUD Layout Container" => {
            entity.layout_container.get_or_insert_with(Default::default);
            entity.rect.get_or_insert_with(Default::default);
        }
        "Blob Shadow" => {
            entity.blob_shadow.get_or_insert_with(Default::default);
        }
        _ => return Err("Unknown component type".into()),
    }
    Ok(())
}
fn component_names(entity: &Actor) -> Vec<&'static str> {
    let mut names = vec!["Transform"];
    for (present, name) in [
        (entity.kind == "Mesh", "Mesh"),
        (entity.kind == "Camera", "Camera"),
        (entity.collider.is_some(), "Collider"),
        (entity.audio.is_some(), "Audio Source"),
        (entity.light.is_some(), "Light"),
        (entity.canvas.is_some(), "Canvas"),
        (entity.rect.is_some(), "Rect Transform"),
        (entity.image.is_some(), "HUD Image"),
        (entity.text.is_some(), "HUD Text"),
        (entity.progress.is_some(), "HUD Progress"),
        (entity.layout_element.is_some(), "HUD Layout Element"),
        (entity.layout_container.is_some(), "HUD Layout Container"),
        (entity.blob_shadow.is_some(), "Blob Shadow"),
    ] {
        if present {
            names.push(name);
        }
    }
    names
}
pub fn draw(
    ui: &Ui,
    doc: &mut BlueprintAsset,
    registry: &Registry,
    state: &mut State,
) -> Result<(), String> {
    let selected = state.selected;
    let result = transaction(doc, registry, |doc| draw_inner(ui, doc, registry, state));
    if result.is_err() {
        state.selected = selected;
    }
    result
}
fn draw_inner(
    ui: &Ui,
    doc: &mut BlueprintAsset,
    registry: &Registry,
    state: &mut State,
) -> Result<(), String> {
    let mut layers = inherited(doc, registry)?;
    // Construction is authored separately. Editing the constructed position must
    // not apply an additive construction operation twice on the next frame.
    layers.push(doc.template.clone());
    let preview = templates::resolve(&layers.iter().collect::<Vec<_>>());
    for layer in &mut layers {
        layer.construction.clear();
    }
    let resolved = templates::resolve(&layers.iter().collect::<Vec<_>>())?;
    if state.selected.is_none() {
        state.selected = resolved.root();
    }
    if resolved.actors.is_empty() {
        ui.text_wrapped("No entity template. Use Add in Components to create a root, or capture a scene selection.");
    }
    if let Some(selected) = state.selected
        && let Some(item) = resolved.actors.iter().find(|e| e.entity.id == selected)
    {
        let _id = ui.push_id(selected.to_string());
        ui.input_text("Child name", &mut state.name).build();
        if ui.button("Add child") {
            if resolved.actors.len() >= templates::MAX_ENTITIES {
                return Err("Actor template capacity reached (32).".into());
            }
            if state.name.trim().is_empty() {
                return Err("Child name cannot be empty.".into());
            }
            state.selected = Some(doc.template.add_child(selected, &state.name));
        }
        ui.same_line();
        if ui.button("Remove local leaf") {
            if resolved.actors.iter().any(|e| e.parent == Some(selected)) {
                return Err("Actor has inherited or local children; reparent them first.".into());
            }
            doc.template.remove_local(selected)?;
            state.selected = None;
            return Ok(());
        }
        let old = item.entity.clone();
        let mut entity = old.clone();
        ui.input_text("Actor name", &mut entity.name).build();
        ui.checkbox("Active", &mut entity.active);
        if let Some(_combo) = ui.begin_combo("Kind", &entity.kind) {
            for kind in ["Empty", "Mesh", "Camera"] {
                if ui.selectable(kind) {
                    entity.kind = kind.into();
                }
            }
        }
        for (label, value) in [
            ("Position", &mut entity.data.position),
            ("Rotation", &mut entity.data.rotation),
            ("Scale", &mut entity.data.scale),
        ] {
            let control = crate::gui::Drag::new(label).speed(0.01);
            if label == "Scale" {
                control.range(1. / 4096., 256.).build_array(ui, value);
            } else {
                control.build_array(ui, value);
            }
        }
        let parent = item
            .parent
            .and_then(|id| resolved.actors.iter().find(|e| e.entity.id == id))
            .map(|e| e.entity.name.as_str())
            .unwrap_or("Root");
        if let Some(_combo) = ui.begin_combo("Actor parent", parent) {
            for candidate in &resolved.actors {
                if candidate.entity.id != selected && ui.selectable(&candidate.entity.name) {
                    let before = doc.template.clone();
                    if let Some(local) = doc
                        .template
                        .actors
                        .iter_mut()
                        .find(|e| e.entity.id == selected)
                    {
                        local.parent = Some(candidate.entity.id);
                    } else {
                        doc.template.overrides.entry(selected).or_default().parent =
                            Some(ParentOverride::Actor {
                                entity: candidate.entity.id,
                            });
                    }
                    let mut proposed = inherited(doc, registry)?;
                    proposed.push(doc.template.clone());
                    if let Err(error) = templates::resolve(&proposed.iter().collect::<Vec<_>>()) {
                        doc.template = before;
                        return Err(error);
                    }
                }
            }
        }
        if entity.kind == "Mesh" {
            ui.color_edit3("Material color", &mut entity.material.color);
            ui.checkbox("Unlit", &mut entity.material.unlit);
            uuid_field(
                ui,
                "Texture UUID",
                &mut entity.material.texture,
                &mut state.ids,
            )?;
        }
        if entity.kind == "Camera" {
            crate::gui::Drag::new("Field of view")
                .range(25., 120.)
                .build(ui, &mut entity.camera_fov);
            ui.color_edit3("Sky color", &mut entity.camera_sky_color);
        }
        crate::collision_editor::inspector(ui, &mut entity);
        if entity.audio.is_none() && ui.small_button("Add Audio Source") {
            entity.audio = Some(Default::default());
        }
        if let Some(audio) = &mut entity.audio {
            uuid_field(ui, "Audio clip UUID", &mut audio.clip, &mut state.ids)?;
            crate::gui::Drag::new("Volume")
                .range(0., 1.)
                .speed(0.01)
                .build(ui, &mut audio.volume);
            crate::gui::Drag::new("Pitch")
                .range(0.25, 4.)
                .speed(0.01)
                .build(ui, &mut audio.pitch);
            ui.checkbox("Play on start", &mut audio.play_on_start);
            if ui.small_button("Remove Audio Source") {
                entity.audio = None;
            }
        }
        if let Some(_combo) = ui.begin_combo("Add component", "Choose...") {
            let parent = item
                .parent
                .and_then(|id| resolved.actors.iter().find(|item| item.entity.id == id))
                .map(|item| &item.entity);
            for name in [
                "Light",
                "Canvas",
                "HUD Image",
                "HUD Text",
                "HUD Progress",
                "HUD Layout Element",
                "HUD Layout Container",
                "Blob Shadow",
            ] {
                if ui.selectable(name) {
                    add_component(&mut entity, parent, name)?;
                }
            }
        }
        crate::lighting_editor::inspector(ui, &mut entity);
        crate::hud_editor::inspector(ui, &mut entity);
        crate::shadows::inspector(ui, &mut entity);
        if entity != old {
            apply_entity_changes(&mut doc.template, &old, &entity)?;
        }
        if doc.template.overrides.contains_key(&selected) && ui.button("Reset Actor to inherited") {
            doc.template.overrides.remove(&selected);
        }
        if let Some(edits) = doc.template.overrides.get_mut(&selected) {
            let keys: Vec<_> = edits.members.keys().cloned().collect();
            for key in keys {
                if ui.small_button(format!("Reset {key}")) {
                    edits.members.remove(&key);
                }
            }
        }
        if let Ok(preview) = &preview
            && let Some(entity) = preview.actors.iter().find(|e| e.entity.id == selected)
        {
            ui.text_disabled(format!(
                "Constructed position: {:.3}, {:.3}, {:.3}",
                entity.entity.position[0], entity.entity.position[1], entity.entity.position[2]
            ));
        }
        if ui.collapsing_header("Construction operations", imgui::TreeNodeFlags::empty()) {
            ui.text_wrapped("Bounded host operations edit templates only; no native script executes in the editor. Operations run in listed order at placement/cook.");
            ui.combo_simple_string(
                "Operation",
                &mut state.operation,
                &[
                    "Translate",
                    "Set position",
                    "Set rotation",
                    "Set scale",
                    "Activate",
                    "Deactivate",
                    "Set color",
                ],
            );
            crate::gui::Drag::new("Vector / color")
                .speed(0.01)
                .build_array(ui, &mut state.offset);
            if ui.button("Append construction operation") {
                let op = match state.operation {
                    0 => ConstructionOp::Translate {
                        entity: selected,
                        offset: state.offset,
                    },
                    1 => ConstructionOp::SetPosition {
                        entity: selected,
                        value: state.offset,
                    },
                    2 => ConstructionOp::SetRotation {
                        entity: selected,
                        value: state.offset,
                    },
                    3 => ConstructionOp::SetScale {
                        entity: selected,
                        value: state.offset,
                    },
                    4 => ConstructionOp::SetActive {
                        entity: selected,
                        active: true,
                    },
                    5 => ConstructionOp::SetActive {
                        entity: selected,
                        active: false,
                    },
                    _ => ConstructionOp::SetColor {
                        entity: selected,
                        color: state.offset,
                    },
                };
                doc.template.construction.push(op);
            }
            let mut remove = None;
            for (index, op) in doc.template.construction.iter().enumerate() {
                ui.text_wrapped(format!("{}. {op:?}", index + 1));
                if ui.small_button(format!("Remove operation##{index}")) {
                    remove = Some(index);
                }
            }
            if let Some(index) = remove {
                doc.template.construction.remove(index);
            }
        }
    }
    if let Err(error) = preview {
        ui.text_colored([1., 0.4, 0.3, 1.], format!("Construction preview: {error}"));
    }
    Ok(())
}
fn create_root(template: &mut Template, name: &str) -> Result<(), String> {
    if !template.actors.is_empty() {
        return Err("An Actor root already exists.".into());
    }
    if !template.overrides.is_empty()
        || !template.references.is_empty()
        || !template.construction.is_empty()
    {
        return Err("Root creation rejected: orphan overrides, references, or construction operations are preserved. Restore their actors with Undo, or repair the source before creating a new root.".into());
    }
    template.actors = Template::root(name).actors;
    Ok(())
}
fn uuid_field(
    ui: &Ui,
    label: &str,
    value: &mut Option<Uuid>,
    drafts: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    let draft = drafts
        .entry(label.into())
        .or_insert_with(|| value.map(|id| id.to_string()).unwrap_or_default());
    if ui.input_text(label, draft).enter_returns_true(true).build() {
        *value = if draft.trim().is_empty() {
            None
        } else {
            Some(
                Uuid::parse_str(draft.trim())
                    .map_err(|_| "Reference must be a UUID; use Enter to apply.".to_string())?,
            )
        };
    }
    Ok(())
}
fn apply_entity_changes(
    template: &mut Template,
    before: &Actor,
    after: &Actor,
) -> Result<(), String> {
    if let Some(local) = template
        .actors
        .iter_mut()
        .find(|e| e.entity.id == before.id)
    {
        local.entity = after.clone();
        return Ok(());
    }
    template
        .overrides
        .entry(before.id)
        .or_default()
        .members
        .extend(templates::changed_members(before, after)?);
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn component_addition_obeys_canvas_contract_without_overwriting() {
        let mut template = Template::root("Root");
        let root = template.actors[0].entity.id;
        let child = template.add_child(root, "Child");
        let root_entity = &mut template.actors[0].entity;
        add_component(root_entity, None, "Canvas").unwrap();
        assert!(root_entity.rect.is_none());
        root_entity.canvas.as_mut().unwrap().enabled = false;
        let original = root_entity.clone();
        add_component(root_entity, None, "Canvas").unwrap();
        assert_eq!(*root_entity, original);
        let child = &mut template
            .actors
            .iter_mut()
            .find(|item| item.entity.id == child)
            .unwrap()
            .entity;
        let before = child.clone();
        assert!(add_component(child, Some(&original), "Canvas").is_err());
        assert_eq!(*child, before);
        assert!(add_component(child, None, "HUD Image").is_err());
        assert_eq!(*child, before);
        add_component(child, Some(&original), "HUD Image").unwrap();
        child.rect.as_mut().unwrap().position = [37., 19.];
        child.image.as_mut().unwrap().enabled = false;
        let before = child.clone();
        add_component(child, Some(&original), "HUD Image").unwrap();
        assert_eq!(*child, before);
        templates::resolve(&[&template]).unwrap();
    }
    #[test]
    fn full_template_transaction_rolls_back_invalid_edits_and_orphan_operations() {
        let mut doc = BlueprintAsset::new("BP_Test".into(), "parent".into());
        doc.template = Template::root("Root");
        let registry = Registry::new();
        let before = doc.template.clone();
        assert!(
            transaction(&mut doc, &registry, |doc| {
                doc.template.actors[0].entity.scale = [0.; 3];
                Ok(())
            })
            .is_err()
        );
        assert_eq!(doc.template, before);
        assert!(
            transaction(&mut doc, &registry, |doc| {
                doc.template.actors[0].entity.kind = "Camera".into();
                doc.template.actors[0].entity.camera_fov = 150.;
                Ok(())
            })
            .is_err()
        );
        assert_eq!(doc.template, before);
        assert!(
            transaction(&mut doc, &registry, |doc| {
                doc.template.actors[0].entity.name.clear();
                Ok(())
            })
            .is_err()
        );
        assert_eq!(doc.template, before);
        let root = doc.template.actors[0].entity.id;
        doc.template.actors[0].entity.canvas = Some(Default::default());
        let child = doc.template.add_child(root, "HUD");
        doc.template.actors[1].entity.rect = Some(Default::default());
        doc.template.actors[1].entity.image = Some(Default::default());
        let before = doc.template.clone();
        assert!(
            transaction(&mut doc, &registry, |doc| {
                doc.template.actors[1]
                    .entity
                    .rect
                    .as_mut()
                    .unwrap()
                    .anchor_max = [-1.; 2];
                Ok(())
            })
            .is_err()
        );
        assert_eq!(doc.template, before);
        doc.template.construction.push(ConstructionOp::Translate {
            entity: child,
            offset: [1., 0., 0.],
        });
        let before = doc.template.clone();
        assert!(transaction(&mut doc, &registry, |doc| doc.template.remove_local(child)).is_err());
        assert_eq!(doc.template, before);
    }
    #[test]
    fn component_search_retains_only_matching_ancestor_chains() {
        let mut template = Template::root("Root");
        let root = template.actors[0].entity.id;
        let branch = template.add_child(root, "Branch");
        let leaf = template.add_child(branch, "Camera Mount");
        let unrelated = template.add_child(root, "Other");
        assert!(descendant_matches(&template.actors, root, "camera"));
        assert!(descendant_matches(&template.actors, branch, "camera"));
        assert!(!descendant_matches(&template.actors, unrelated, "camera"));
        let entity = &mut template
            .actors
            .iter_mut()
            .find(|item| item.entity.id == leaf)
            .unwrap()
            .entity;
        entity.audio = Some(Default::default());
        assert!(
            descendant_matches(&template.actors, root, "audio"),
            "Search includes real component names"
        );
        assert!(!descendant_matches(&template.actors, root, "nonexistent"));
    }
    #[test]
    fn root_creation_rejects_orphans_without_discarding_source() {
        let mut template = Template::default();
        template.construction.push(ConstructionOp::Translate {
            entity: Uuid::new_v4(),
            offset: [1., 2., 3.],
        });
        let before = serde_json::to_value(&template).unwrap();
        assert!(create_root(&mut template, "Root").is_err());
        assert_eq!(serde_json::to_value(&template).unwrap(), before);
        template.construction.clear();
        create_root(&mut template, "Root").unwrap();
        assert_eq!(template.actors.len(), 1);
        assert!(create_root(&mut template, "Another root").is_err());
    }
    #[test]
    fn inherited_edits_are_explicit_and_local_edits_keep_identity() {
        let base = Template::root("Root");
        let original = base.actors[0].entity.clone();
        let mut changed = original.clone();
        changed.position = [1., 2., 3.];
        changed.collider = Some(Default::default());
        let mut derived = Template::default();
        apply_entity_changes(&mut derived, &original, &changed).unwrap();
        assert!(derived.actors.is_empty());
        assert_eq!(
            derived.overrides[&original.id].members[&format!(
                "/components/{}/properties/position",
                original.root().unwrap().id
            )],
            json!([1., 2., 3.])
        );
        assert_eq!(
            templates::resolve(&[&base, &derived]).unwrap().actors[0].entity,
            changed
        );
        let mut local = base.clone();
        apply_entity_changes(&mut local, &original, &changed).unwrap();
        assert!(local.overrides.is_empty());
        assert_eq!(local.actors[0].entity.id, original.id);
    }
}
