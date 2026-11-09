//! Typed template controls reuse the scene component inspectors and the same
//! bounded resolver used by placement/cooking; editor reads do not run scripts.
use crate::{
    blueprint::Registry,
    blueprint_asset::BlueprintAsset,
    blueprint_templates::{self as templates, ConstructionOp, ParentOverride, Template},
    scene::Actor,
};
use imgui::Ui;
use uuid::Uuid;

/// A borrowed editor projection, shared by Blueprint assets and source classes.
/// It is not a second persisted class or behavior document.
pub struct Document<'a> {
    pub name: &'a str,
    pub class: Option<&'a str>,
    pub template: &'a mut Template,
    pub inherited: &'a [Template],
    pub sources: Option<&'a templates::CompositionSources>,
    pub owner: Option<&'a str>,
}

pub struct State {
    selected: Option<Uuid>,
    component: Option<Uuid>,
    name: String,
    offset: [f32; 3],
    operation: usize,
    search: String,
    open_strings: bool,
}
impl Default for State {
    fn default() -> Self {
        Self {
            selected: None,
            component: None,
            name: "Child".into(),
            offset: [0.; 3],
            operation: 0,
            search: String::new(),
            open_strings: false,
        }
    }
}
impl State {
    pub fn selected(&self) -> Option<Uuid> {
        self.selected
    }
    pub fn select(&mut self, id: Option<Uuid>) {
        self.selected = id;
        self.component = None;
    }
    pub fn component(&self) -> Option<Uuid> {
        self.component
    }
    pub fn select_component(&mut self, actor: Uuid, component: Uuid) {
        self.selected = Some(actor);
        self.component = Some(component);
    }
    pub fn take_strings_request(&mut self) -> bool {
        std::mem::take(&mut self.open_strings)
    }
}
fn inherited(doc: &Document<'_>, _registry: &Registry) -> Result<Vec<Template>, String> {
    Ok(doc.inherited.to_vec())
}
fn resolve_layers(
    doc: &Document<'_>,
    registry: &Registry,
    layers: &[Template],
) -> Result<templates::ResolvedTemplate, String> {
    let layers = layers.iter().collect::<Vec<_>>();
    if let Some(sources) = doc.sources {
        sources.resolve_layers(
            registry,
            doc.owner.or(doc.class).unwrap_or("document"),
            &layers,
        )
    } else {
        templates::resolve(&layers)
    }
}
fn transaction<T>(
    doc: &mut Document<'_>,
    registry: &Registry,
    edit: impl FnOnce(&mut Document<'_>) -> Result<T, String>,
) -> Result<T, String> {
    let before = doc.template.clone();
    let result = edit(doc).and_then(|value| {
        if *doc.template != before {
            let mut layers = inherited(doc, registry)?;
            layers.push(doc.template.clone());
            let resolved = resolve_layers(doc, registry, &layers)?;
            if let Some(class) = doc.owner.and_then(|id| registry.classes.get(id))
                && !resolved.actors.is_empty()
            {
                resolved.scene(
                    crate::scene::ClassDefaults {
                        name: class.cpp_name.clone(),
                        class_id: Some(class.id.clone()),
                        provider: class.provider.clone(),
                        backend: class.backend.clone(),
                        ..Default::default()
                    },
                    registry,
                )?;
            }
        }
        Ok(value)
    });
    if result.is_err() {
        *doc.template = before;
    }
    result
}
pub fn browser(
    ui: &Ui,
    doc: &mut BlueprintAsset,
    registry: &Registry,
    state: &mut State,
    root: &std::path::Path,
) -> Result<bool, String> {
    let inherited = crate::class_composition::inherited(root, registry, &doc.parent)?;
    let sources = crate::class_composition::sources(root, registry)?;
    class_browser(
        ui,
        &mut Document {
            name: &doc.name,
            class: Some(&doc.parent),
            template: &mut doc.template,
            inherited: &inherited,
            sources: Some(&sources),
            owner: Some(&doc.id),
        },
        registry,
        state,
    )
}
pub fn class_browser(
    ui: &Ui,
    doc: &mut Document<'_>,
    registry: &Registry,
    state: &mut State,
) -> Result<bool, String> {
    let selected = (state.selected, state.component);
    let result = transaction(doc, registry, |doc| browser_inner(ui, doc, registry, state));
    if result.is_err() {
        (state.selected, state.component) = selected;
    }
    result
}
fn browser_inner(
    ui: &Ui,
    doc: &mut Document<'_>,
    registry: &Registry,
    state: &mut State,
) -> Result<bool, String> {
    let mut layers = inherited(doc, registry)?;
    layers.push(doc.template.clone());
    for layer in &mut layers {
        layer.construction.clear();
    }
    let resolved = resolve_layers(doc, registry, &layers)?;
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
                create_document_root(doc, registry)?;
                state.selected = doc.template.actors.first().map(|e| e.entity.id);
                activated = true;
            }
        } else {
            let selected = state.selected.or_else(|| resolved.root()).unwrap();
            ui.input_text("Child name", &mut state.name).build();
            if doc.sources.is_some()
                && let Some(_menu) = ui.begin_menu("Reusable class")
            {
                let model = registry.model().map_err(|errors| {
                    errors
                        .into_iter()
                        .map(|e| e.message)
                        .collect::<Vec<_>>()
                        .join("\n")
                })?;
                let domain = resolved
                    .actors
                    .iter()
                    .find(|item| item.entity.id == selected)
                    .and_then(|item| item.entity.class.resolve(&model))
                    .map(|c| c.domain);
                for class in model.iter().filter(|class| {
                    class.family == crate::reflection_schema::ClassFamily::Actor
                        && class.instantiable()
                        && Some(class.id.as_str()) != doc.owner
                        && Some(class.domain) == domain
                }) {
                    if ui.selectable(&class.cpp_name) {
                        let id = add_class_instance(
                            doc.template,
                            &model,
                            selected,
                            &class.id,
                            &state.name,
                        )?;
                        state.select(Some(id));
                        activated = true;
                    }
                }
            }
            if ui.selectable("Add child Actor") {
                if state.name.trim().is_empty() {
                    return Err("Child name cannot be empty.".into());
                }
                if resolved.actors.len() >= templates::MAX_ENTITIES {
                    return Err("Actor template capacity reached (32).".into());
                }
                state.selected = Some(add_document_child(doc, registry, selected, &state.name)?);
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
                    "HUD Focusable",
                    "Blob Shadow",
                ] {
                    if entity.rect.is_some() && matches!(name, "Collider" | "Light" | "Blob Shadow")
                    {
                        continue;
                    }
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
                    apply_entity_changes(doc.template, &item.entity, &entity)?;
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
                    .any(|name| name.to_lowercase().contains(&query))
                || item.entity.components.iter().any(|c| {
                    c.name.to_lowercase().contains(&query)
                        || c.class.name.to_lowercase().contains(&query)
                });
            // Preserve ancestors while searching so matches keep their hierarchy.
            if !own_match && !descendant_matches(all, item.entity.id, &query) {
                continue;
            }
            let inherited = !own.actors.iter().any(|own| own.entity.id == item.entity.id);
            let label = format!(
                "\u{eb29} {}{}##entity-{}",
                item.entity.name,
                if own.instances.contains(&item.entity.id) {
                    " (class instance)"
                } else if inherited {
                    " (inherited)"
                } else {
                    ""
                },
                item.entity.id
            );
            let expanded = ui
                .tree_node_config(&label)
                .default_open(true)
                .flags(
                    imgui::TreeNodeFlags::OPEN_ON_ARROW
                        | imgui::TreeNodeFlags::DEFAULT_OPEN
                        | imgui::TreeNodeFlags::SPAN_AVAIL_WIDTH
                        | if state.selected == Some(item.entity.id) && state.component.is_none() {
                            imgui::TreeNodeFlags::SELECTED
                        } else {
                            imgui::TreeNodeFlags::empty()
                        },
                )
                .push();
            super::record_control(ui, &format!("bp-component:{}", item.entity.id));
            if ui.is_item_clicked() && !ui.is_item_toggled_open() {
                state.select(Some(item.entity.id));
                *activated = true;
            }
            if let Some(_node) = expanded {
                for component in &item.entity.components {
                    let name = &component.name;
                    if !query.is_empty()
                        && !own_match
                        && !name.to_lowercase().contains(&query)
                        && !component.class.name.to_lowercase().contains(&query)
                    {
                        continue;
                    }
                    if ui
                        .selectable_config(format!("\u{eae9} {name}##{}", component.id))
                        .selected(state.component == Some(component.id))
                        .build()
                    {
                        state.select_component(item.entity.id, component.id);
                        *activated = true;
                    }
                    super::record_control(ui, &format!("bp-component-object:{}", component.id));
                }
                row(ui, all, Some(item.entity.id), own, state, activated);
            }
        }
    }
    row(
        ui,
        &resolved.actors,
        None,
        doc.template,
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
                || child.entity.components.iter().any(|c| {
                    c.name.to_lowercase().contains(query)
                        || c.class.name.to_lowercase().contains(query)
                })
                || descendant_matches(all, child.entity.id, query)
        })
}
fn add_component(entity: &mut Actor, parent: Option<&Actor>, name: &str) -> Result<(), String> {
    if name == "Canvas"
        && (entity.kind != "Empty"
            || parent.is_some_and(|p| p.canvas.is_none() && p.rect.is_none()))
    {
        return Err("Canvas needs an Empty root or a child of Canvas or RectTransform. Existing components were preserved.".into());
    }
    if matches!(
        name,
        "HUD Image"
            | "HUD Text"
            | "HUD Progress"
            | "HUD Layout Element"
            | "HUD Layout Container"
            | "HUD Focusable"
    ) && (entity.kind != "Empty"
        || (entity.canvas.is_none()
            && !parent.is_some_and(|parent| parent.canvas.is_some() || parent.rect.is_some())))
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
            if parent.is_some() {
                entity.rect.get_or_insert_with(Default::default);
            }
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
        "HUD Focusable" => {
            entity.focusable.get_or_insert_with(Default::default);
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
    let mut names = vec![];
    if entity.rect.is_none() {
        names.push("Transform");
    }
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
        (entity.focusable.is_some(), "HUD Focusable"),
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
    root: &std::path::Path,
    resources: &crate::hud_editor::Resources<'_>,
) -> Result<(), String> {
    let inherited = crate::class_composition::inherited(root, registry, &doc.parent)?;
    let sources = crate::class_composition::sources(root, registry)?;
    class_draw(
        ui,
        &mut Document {
            name: &doc.name,
            class: Some(&doc.parent),
            template: &mut doc.template,
            inherited: &inherited,
            sources: Some(&sources),
            owner: Some(&doc.id),
        },
        registry,
        state,
        resources,
    )
}
pub fn class_draw(
    ui: &Ui,
    doc: &mut Document<'_>,
    registry: &Registry,
    state: &mut State,
    resources: &crate::hud_editor::Resources<'_>,
) -> Result<(), String> {
    let selected = state.selected;
    let result = transaction(doc, registry, |doc| {
        draw_inner(ui, doc, registry, state, resources)
    });
    if result.is_err() {
        state.selected = selected;
    }
    result
}
fn draw_inner(
    ui: &Ui,
    doc: &mut Document<'_>,
    registry: &Registry,
    state: &mut State,
    resources: &crate::hud_editor::Resources<'_>,
) -> Result<(), String> {
    let mut layers = inherited(doc, registry)?;
    // Construction is authored separately. Editing the constructed position must
    // not apply an additive construction operation twice on the next frame.
    layers.push(doc.template.clone());
    let preview = resolve_layers(doc, registry, &layers);
    for layer in &mut layers {
        layer.construction.clear();
    }
    let resolved = resolve_layers(doc, registry, &layers)?;
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
        let mut projection = crate::scene::Scene {
            actors: resolved
                .actors
                .iter()
                .map(|item| {
                    let mut actor = item.entity.clone();
                    actor.logical_parent = item.parent;
                    actor
                })
                .collect(),
            ..Default::default()
        };
        projection.refresh_actor_hierarchy();
        crate::hud::stabilize_focus(&mut projection.actors);
        let focus_targets = projection
            .actor_index(selected)
            .map_or_else(Vec::new, |i| crate::hud::focus_targets(&projection, i));
        if let Some(component) = state
            .component
            .and_then(|id| item.entity.components.iter().find(|c| c.id == id))
            && let Some(class) = component.class.class_id.as_deref()
            && (crate::hud_editor::has_component_inspector(class)
                || class == crate::object_model::AUDIO_COMPONENT_ID)
        {
            let old = projection.actors.iter().find(|a| a.id == selected).unwrap();
            let mut entity = old.clone();
            if class == crate::object_model::AUDIO_COMPONENT_ID {
                if let Some(audio) = &mut entity.audio {
                    audio_fields(ui, audio, resources);
                }
            } else {
                state.open_strings |= crate::hud_editor::component_inspector(
                    ui,
                    &mut entity,
                    &focus_targets,
                    resources,
                    class,
                );
            }
            if entity != *old {
                apply_entity_changes(doc.template, old, &entity)?;
            }
            component_resets(ui, doc.template, selected, component.id);
            return Ok(());
        }
        ui.input_text("Child name", &mut state.name).build();
        if ui.button("Add child") {
            if resolved.actors.len() >= templates::MAX_ENTITIES {
                return Err("Actor template capacity reached (32).".into());
            }
            if state.name.trim().is_empty() {
                return Err("Child name cannot be empty.".into());
            }
            state.selected = Some(add_document_child(doc, registry, selected, &state.name)?);
        }
        ui.same_line();
        if doc.template.instances.contains(&selected) && ui.button("Remove class instance") {
            remove_class_instance(doc.template, &resolved, selected)?;
            state.select(None);
            return Ok(());
        }
        if ui.button("Remove local leaf") {
            if resolved.actors.iter().any(|e| e.parent == Some(selected)) {
                return Err("Actor has inherited or local children; reparent them first.".into());
            }
            doc.template.remove_local(selected)?;
            state.selected = None;
            return Ok(());
        }
        let old = projection
            .actors
            .iter()
            .find(|actor| actor.id == selected)
            .unwrap()
            .clone();
        let mut entity = old.clone();
        let ui_control = entity.rect.is_some();
        ui.input_text("Actor name", &mut entity.name).build();
        ui.checkbox("Active", &mut entity.active);
        if !ui_control {
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
                    if let Err(error) = resolve_layers(doc, registry, &proposed) {
                        *doc.template = before;
                        return Err(error);
                    }
                }
            }
        }
        if entity.kind == "Mesh" {
            ui.color_edit3("Material color", &mut entity.material.color);
            ui.checkbox("Unlit", &mut entity.material.unlit);
            crate::blueprint_refs::asset_inspector(
                ui,
                "Texture",
                &mut entity.material.texture,
                "Texture",
                resources.index,
                "None",
            );
        }
        if entity.kind == "Camera" {
            crate::gui::Drag::new("Field of view")
                .range(25., 120.)
                .build(ui, &mut entity.camera_fov);
            ui.color_edit3("Sky color", &mut entity.camera_sky_color);
            let mut gradient = entity.camera_sky_gradient.is_some();
            if ui.checkbox("Sky gradient", &mut gradient) {
                entity.camera_sky_gradient =
                    gradient.then(|| crate::scene::SkyGradient::for_sky(entity.camera_sky_color));
            }
            if let Some(g) = entity.camera_sky_gradient.as_mut() {
                ui.color_edit3("Horizon color", &mut g.horizon);
                ui.color_edit3("Ground color", &mut g.ground);
            }
            ui.checkbox("Mirror X", &mut entity.camera_mirror_x);
        }
        if !ui_control {
            crate::collision_editor::inspector(ui, &mut entity);
        }
        if entity.audio.is_none() && ui.small_button("Add Audio Source") {
            entity.audio = Some(Default::default());
        }
        if let Some(audio) = &mut entity.audio {
            crate::blueprint_refs::asset_inspector(
                ui,
                "Audio clip",
                &mut audio.clip,
                "PlayableAudio",
                resources.index,
                "None",
            );
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
                "HUD Focusable",
                "Blob Shadow",
            ] {
                if ui_control && matches!(name, "Light" | "Blob Shadow") {
                    continue;
                }
                if ui.selectable(name) {
                    add_component(&mut entity, parent, name)?;
                }
            }
        }
        if !ui_control {
            crate::lighting_editor::inspector(ui, &mut entity);
        }
        state.open_strings |=
            crate::hud_editor::inspector(ui, &mut entity, &focus_targets, resources);
        if !ui_control {
            crate::shadows::inspector(ui, &mut entity);
        }
        if entity != old {
            apply_entity_changes(doc.template, &old, &entity)?;
        }
        if doc.template.overrides.contains_key(&selected) && ui.button("Reset Actor to inherited") {
            doc.template.overrides.remove(&selected);
            let owners = std::iter::once(selected)
                .chain(item.entity.components.iter().map(|c| c.id))
                .collect::<std::collections::BTreeSet<_>>();
            doc.template
                .references
                .retain(|r| !owners.contains(&r.owner));
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
        if !ui_control
            && ui.collapsing_header("Construction operations", imgui::TreeNodeFlags::empty())
        {
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
fn audio_fields(
    ui: &Ui,
    audio: &mut crate::audio::AudioSource,
    resources: &crate::hud_editor::Resources<'_>,
) {
    crate::blueprint_refs::asset_inspector(
        ui,
        "Audio clip",
        &mut audio.clip,
        "PlayableAudio",
        resources.index,
        "None",
    );
    crate::gui::Drag::new(crate::gui::field(ui, "Volume"))
        .range(0., 1.)
        .speed(0.01)
        .build(ui, &mut audio.volume);
    crate::gui::Drag::new(crate::gui::field(ui, "Pitch"))
        .range(0.25, 4.)
        .speed(0.01)
        .build(ui, &mut audio.pitch);
    ui.checkbox("Play on start", &mut audio.play_on_start);
    let mut priority = i32::from(audio.priority);
    if crate::gui::Drag::new(crate::gui::field(ui, "Priority"))
        .range(0, 255)
        .build(ui, &mut priority)
    {
        audio.priority = priority as u8;
    }
}
fn component_resets(ui: &Ui, template: &mut Template, actor: Uuid, component: Uuid) {
    let Some(edits) = template.overrides.get_mut(&actor) else {
        return;
    };
    let prefix = format!("/components/{component}");
    let keys = edits
        .members
        .keys()
        .filter(|key| key.starts_with(&format!("{prefix}/")))
        .cloned()
        .collect::<Vec<_>>();
    for key in keys {
        let field = key
            .rsplit('/')
            .next()
            .unwrap_or("property")
            .replace('_', " ");
        if ui.small_button(format!("Reset {field}##{key}")) {
            edits.members.remove(&key);
        }
    }
}
fn add_document_child(
    doc: &mut Document<'_>,
    registry: &Registry,
    parent: Uuid,
    name: &str,
) -> Result<Uuid, String> {
    if let Some(class) = doc.class
        && registry
            .ancestry(
                &registry
                    .classes
                    .get(class)
                    .ok_or("Class is missing.")?
                    .cpp_name,
            )
            .iter()
            .any(|c| c.id == crate::object_model::UI_ACTOR_ID)
    {
        let model = registry.model().map_err(|errors| {
            errors
                .into_iter()
                .map(|e| e.message)
                .collect::<Vec<_>>()
                .join("\n")
        })?;
        let mut actor =
            crate::class_composition::actor(&model, crate::object_model::UI_ACTOR_ID, name)?;
        actor.attach = Some(crate::actor_document::Attachment {
            actor: parent,
            component: None,
        });
        let id = actor.id;
        doc.template.actors.push(templates::TemplateEntity {
            entity: actor,
            parent: Some(parent),
        });
        Ok(id)
    } else {
        Ok(doc.template.add_child(parent, name))
    }
}
fn create_document_root(doc: &mut Document<'_>, registry: &Registry) -> Result<(), String> {
    if let Some(class) = doc.class {
        let model = registry.model().map_err(|errors| {
            errors
                .into_iter()
                .map(|e| e.message)
                .collect::<Vec<_>>()
                .join("\n")
        })?;
        let actor = crate::class_composition::actor(&model, class, doc.name)?;
        create_root(doc.template, doc.name)?;
        doc.template.actors[0].entity = actor;
        Ok(())
    } else {
        create_root(doc.template, doc.name)
    }
}
fn create_root(template: &mut Template, name: &str) -> Result<(), String> {
    if !template.actors.is_empty() {
        return Err("An Actor root already exists.".into());
    }
    if !template.overrides.is_empty()
        || !template.instances.is_empty()
        || !template.references.is_empty()
        || !template.construction.is_empty()
    {
        return Err("Root creation rejected: orphan overrides, references, or construction operations are preserved. Restore their actors with Undo, or repair the source before creating a new root.".into());
    }
    template.actors = Template::root(name).actors;
    Ok(())
}
pub(crate) fn apply_entity_changes(
    template: &mut Template,
    before: &Actor,
    after: &Actor,
) -> Result<(), String> {
    if !template.instances.contains(&before.id)
        && let Some(local) = template
            .actors
            .iter_mut()
            .find(|e| e.entity.id == before.id)
    {
        local.entity = after.clone();
        // Preview scenes project UUID hierarchy to numeric parent indices. Keep
        // those transient indices out of the portable class template; its UUID
        // parent remains on TemplateEntity.
        local.entity.parent = None;
        return Ok(());
    }
    let members = &mut template.overrides.entry(before.id).or_default().members;
    for (path, value) in templates::changed_members(before, after)? {
        members.retain(|key, _| !key.starts_with(&format!("{path}/")));
        members.insert(path, value);
    }
    Ok(())
}
pub(crate) fn add_class_instance(
    template: &mut Template,
    model: &crate::object_model::Model,
    parent: Uuid,
    class: &str,
    name: &str,
) -> Result<Uuid, String> {
    if name.trim().is_empty() {
        return Err("Child name cannot be empty.".into());
    }
    let mut actor = crate::class_composition::actor(model, class, name)?;
    actor.attach = Some(crate::actor_document::Attachment {
        actor: parent,
        component: None,
    });
    let id = actor.id;
    template.actors.push(templates::TemplateEntity {
        entity: actor,
        parent: Some(parent),
    });
    template.instances.insert(id);
    Ok(id)
}
fn remove_class_instance(
    template: &mut Template,
    resolved: &templates::ResolvedTemplate,
    root: Uuid,
) -> Result<(), String> {
    let mut branch = std::collections::BTreeSet::from([root]);
    loop {
        let before = branch.len();
        branch.extend(
            resolved
                .actors
                .iter()
                .filter(|item| item.parent.is_some_and(|id| branch.contains(&id)))
                .map(|item| item.entity.id)
                .collect::<Vec<_>>(),
        );
        if branch.len() == before {
            break;
        }
    }
    let objects = resolved
        .actors
        .iter()
        .filter(|item| branch.contains(&item.entity.id))
        .flat_map(|item| {
            std::iter::once(item.entity.id).chain(item.entity.components.iter().map(|c| c.id))
        })
        .collect::<std::collections::BTreeSet<_>>();
    if template
        .references
        .iter()
        .any(|r| !objects.contains(&r.owner) && objects.contains(&r.target))
    {
        return Err("Class instance is referenced by another control; clear that binding before removing it.".into());
    }
    template
        .actors
        .retain(|item| !branch.contains(&item.entity.id));
    template.instances.retain(|id| !branch.contains(id));
    template.overrides.retain(|id, _| !branch.contains(id));
    template.references.retain(|r| !objects.contains(&r.owner));
    template.construction.retain(|op| {
        let id = match op {
            ConstructionOp::Translate { entity, .. }
            | ConstructionOp::SetPosition { entity, .. }
            | ConstructionOp::SetRotation { entity, .. }
            | ConstructionOp::SetScale { entity, .. }
            | ConstructionOp::SetActive { entity, .. }
            | ConstructionOp::SetColor { entity, .. }
            | ConstructionOp::Reparent { entity, .. } => entity,
        };
        !branch.contains(id)
    });
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
        assert!(add_component(child, None, "HUD Image").is_err());
        assert_eq!(*child, before);
        add_component(child, Some(&original), "HUD Image").unwrap();
        child.rect.as_mut().unwrap().position = [37., 19.];
        child.image.as_mut().unwrap().enabled = false;
        let before = child.clone();
        add_component(child, Some(&original), "HUD Image").unwrap();
        assert_eq!(*child, before);
        add_component(child, Some(&original), "Canvas").unwrap();
        assert_eq!(child.rect.as_ref().unwrap().position, [37., 19.]);
        assert!(!child.image.as_ref().unwrap().enabled);
        let before = child.clone();
        add_component(child, Some(&original), "Canvas").unwrap();
        assert_eq!(*child, before);
        let mut ui_root = before.clone();
        ui_root.canvas = None;
        let rect = ui_root.rect.clone();
        add_component(&mut ui_root, None, "Canvas").unwrap();
        add_component(&mut ui_root, None, "HUD Text").unwrap();
        assert_eq!(
            ui_root.rect, rect,
            "A UIActor already owns its RectTransform"
        );
        let mut invalid = before.clone();
        invalid.kind = "Mesh".into();
        let before = invalid.clone();
        assert!(add_component(&mut invalid, None, "Canvas").is_err());
        assert_eq!(invalid, before);
        templates::resolve(&[&template]).unwrap();
    }
    #[test]
    fn full_template_transaction_rolls_back_invalid_edits_and_orphan_operations() {
        let mut template = Template::root("Root");
        let mut doc = Document {
            name: "BP_Test",
            class: None,
            template: &mut template,
            inherited: &[],
            sources: None,
            owner: None,
        };
        let registry = Registry::new();
        let before = doc.template.clone();
        assert!(
            transaction(&mut doc, &registry, |doc| {
                doc.template.actors[0].entity.scale = [0.; 3];
                Ok(())
            })
            .is_err()
        );
        assert_eq!(*doc.template, before);
        assert!(
            transaction(&mut doc, &registry, |doc| {
                doc.template.actors[0].entity.kind = "Camera".into();
                doc.template.actors[0].entity.camera_fov = 150.;
                Ok(())
            })
            .is_err()
        );
        assert_eq!(*doc.template, before);
        assert!(
            transaction(&mut doc, &registry, |doc| {
                doc.template.actors[0].entity.name.clear();
                Ok(())
            })
            .is_err()
        );
        assert_eq!(*doc.template, before);
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
        assert_eq!(*doc.template, before);
        doc.template.construction.push(ConstructionOp::Translate {
            entity: child,
            offset: [1., 0., 0.],
        });
        let before = doc.template.clone();
        assert!(transaction(&mut doc, &registry, |doc| doc.template.remove_local(child)).is_err());
        assert_eq!(*doc.template, before);
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
    fn local_control_edit_discards_preview_parent_index_and_keeps_uuid_hierarchy() {
        let mut template = Template::root("Root");
        let root = template.actors[0].entity.id;
        let child = template.add_child(root, "Local control");
        let mut original = template
            .actors
            .iter()
            .find(|item| item.entity.id == child)
            .unwrap()
            .entity
            .clone();
        // class_draw and the UI canvas both edit a hierarchy-refreshed Scene.
        original.logical_parent = Some(root);
        original.parent = Some(0);
        let mut edited = original.clone();
        edited.position = [13., 7., 0.];
        crate::actor_components::sync(&mut edited);
        apply_entity_changes(&mut template, &original, &edited).unwrap();
        let local = template
            .actors
            .iter()
            .find(|item| item.entity.id == child)
            .unwrap();
        assert_eq!(local.parent, Some(root));
        assert_eq!(
            local.entity.parent, None,
            "Numeric parents belong to the Scene preview"
        );
        assert_eq!(local.entity.position, [13., 7., 0.]);
        let resolved = templates::resolve(&[&template]).unwrap();
        let child = resolved
            .actors
            .iter()
            .find(|item| item.entity.id == child)
            .unwrap();
        assert_eq!(child.parent, Some(root));
        assert_eq!(child.entity.position, [13., 7., 0.]);
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
