//! A view of the existing class model, not a second class/asset registry.
use crate::{
    editor::Editor,
    game_helpers as helpers,
    object_model::{ClassModel, Model},
    reflection_schema::{ClassFamily, Domain},
};

const PAYLOAD: &str = "EPOK_HELPER_CLASS";
const GROUPS: &[&str] = &["Interaction", "Dialogue", "Characters", "World Gameplay"];

#[derive(Debug)]
pub struct PlacementPreview {
    pub actors: usize,
    pub components: usize,
    pub level_actors: usize,
    pub owner_components: usize,
    pub dependencies: std::collections::BTreeSet<String>,
}

/// Resolve inherited compositions and component contracts with the ordinary
/// placement path. Preview never installs assets or edits the current document.
pub fn preview(
    e: &Editor,
    id: &str,
    actor: Option<uuid::Uuid>,
) -> Result<PlacementPreview, String> {
    let model = e.object_model().ok_or("The class model is unavailable.")?;
    let class = model
        .class(id)
        .ok_or("The helper class no longer exists.")?;
    let (candidate, first) = match (class.family, actor) {
        (ClassFamily::Actor, None) => (
            crate::editor::preview_class(&e.scene, &e.root, &e.class_registry, &model, id)?,
            e.scene.actors.len(),
        ),
        (ClassFamily::Component, Some(owner)) => {
            let mut candidate = e.scene.clone();
            let index = candidate
                .actor_index(owner)
                .ok_or("Select an Actor first.")?;
            crate::actor_components::add(&mut candidate.actors[index], &model, id)?;
            candidate.validate_with_model(Some(&model))?;
            (candidate, index)
        }
        _ => return Err("Select an Actor to preview this component's attachment.".into()),
    };
    let component = class.family == ClassFamily::Component;
    let added = if component {
        &candidate.actors[first..first + 1]
    } else {
        &candidate.actors[first..]
    };
    let mut dependencies = std::collections::BTreeSet::new();
    for owner in added {
        for instance in &owner.components {
            dependencies.insert(instance.class.name.clone());
            if let Some(contract) = instance
                .class
                .resolve(&model)
                .and_then(|c| c.component.as_ref())
            {
                dependencies.extend(contract.requires.iter().cloned());
                dependencies.extend(
                    contract
                        .capabilities
                        .iter()
                        .map(|s| format!("Capability: {s}")),
                );
            }
        }
        fn assets(
            value: &serde_json::Value,
            index: &crate::assets::Index,
            out: &mut std::collections::BTreeSet<String>,
        ) {
            match value {
                serde_json::Value::String(value) => {
                    if let Ok(id) = uuid::Uuid::parse_str(value)
                        && let Some(records) = index.assets.get(&id)
                    {
                        for record in records {
                            out.insert(format!("Asset: {} ({id})", record.path.display()));
                        }
                    }
                }
                serde_json::Value::Array(values) => {
                    for value in values {
                        assets(value, index, out);
                    }
                }
                serde_json::Value::Object(values) => {
                    for value in values.values() {
                        assets(value, index, out);
                    }
                }
                _ => {}
            }
        }
        assets(
            &serde_json::to_value(owner).map_err(|e| e.to_string())?,
            &e.assets.index,
            &mut dependencies,
        );
        for reference in
            std::iter::once(&owner.class).chain(owner.components.iter().map(|c| &c.class))
        {
            if let Some(resolved) = reference.resolve(&model) {
                for property in e.class_registry.properties(&resolved.cpp_name) {
                    assets(&property.default, &e.assets.index, &mut dependencies);
                }
            }
        }
    }
    if class.id == crate::dialogue_layout::PRESENTER {
        dependencies.insert(
            "Project texture: assets/GameHelpers/UI/dialogue-frame.epokasset (reused or installed)"
                .into(),
        );
    }
    Ok(PlacementPreview {
        actors: usize::from(!component) * added.len(),
        components: if component {
            1
        } else {
            added.iter().map(|a| a.components.len()).sum()
        },
        level_actors: candidate.actors.len(),
        owner_components: added.iter().map(|a| a.components.len()).max().unwrap_or(0),
        dependencies,
    })
}

pub fn group(class: &ClassModel) -> Option<&'static str> {
    for id in class.ancestry.iter().rev() {
        for (label, classes) in [
            ("World Gameplay", helpers::WORLD_GAMEPLAY_CLASSES),
            ("Queries", helpers::QUERY_CLASSES),
            ("Dialogue", helpers::DIALOGUE_CLASSES),
            ("Dialogue", helpers::UI_CLASSES),
            ("Interaction", helpers::INTERACTION_CLASSES),
            ("Characters", helpers::CHARACTER_CLASSES),
            ("Characters", helpers::CAMERA_CLASSES),
            ("Characters", helpers::COMMAND_SOURCE_CLASSES),
            ("Characters", helpers::CHARACTER_INPUT_CLASSES),
            ("Characters", helpers::ANIMATION_CLASSES),
        ] {
            if classes.iter().any(|(_, key)| *key == id) {
                return Some(label);
            }
        }
    }
    None
}

pub fn entries(model: &Model, domain: Domain) -> Vec<&ClassModel> {
    let mut entries = model
        .iter()
        .filter(|class| class.instantiable() && group(class).is_some())
        .filter(|class| match class.family {
            ClassFamily::Actor => class.domain == domain && class.placement.placeable,
            ClassFamily::Component => class
                .component
                .as_ref()
                .is_some_and(|contract| contract.owners.contains(&domain)),
            _ => false,
        })
        .collect::<Vec<_>>();
    entries.sort_by(|a, b| a.cpp_name.cmp(&b.cpp_name));
    entries
}

/// Dispatch through the same undoable operations as Instantiate Actor and Add
/// Component. The destination decides whether this is placement or attachment.
pub fn apply(e: &mut Editor, id: &str, actor: Option<uuid::Uuid>) -> Result<(), String> {
    if e.playing {
        return Err("Stop Play before placing Game Helpers.".into());
    }
    let model = e.object_model().ok_or("The class model is unavailable.")?;
    let class = model
        .class(id)
        .ok_or("The helper class no longer exists.")?;
    if !class.instantiable() || group(class).is_none() {
        return Err("Choose a concrete Game Helper class.".into());
    }
    match (class.family, actor) {
        (ClassFamily::Actor, None)
            if class.domain == e.scene_view_mode.domain() && class.placement.placeable =>
        {
            e.create_actor(id);
        }
        (ClassFamily::Component, Some(actor)) => {
            if !e
                .addable_component_classes(actor)
                .iter()
                .any(|(_, name, _)| name == &class.cpp_name)
            {
                return Err("This helper is incompatible with the selected Actor's domain, dependencies or component capacity.".into());
            }
            e.add_actor_component(actor, id);
        }
        (ClassFamily::Actor, _) => {
            return Err("Drop this Actor helper in its matching scene view.".into());
        }
        _ => return Err("Drop a component helper onto an Actor's Inspector.".into()),
    }
    e.last_error.clone().map_or(Ok(()), Err)
}

pub fn draw(ui: &imgui::Ui, e: &mut Editor) {
    ui.text("Game Helpers");
    ui.text_wrapped("Drag an Actor into its scene view, or a component onto an Actor's Inspector. Double-click to place or attach to the selected Actor.");
    ui.set_next_item_width(-1.);
    ui.input_text("##helper-search", &mut e.project_search)
        .hint("Search helper classes")
        .build();
    let Some(model) = e.object_model() else {
        ui.text_wrapped("Resolve project compilation errors to load the class model.");
        return;
    };
    let query = e.project_search.to_lowercase();
    let entries = entries(&model, e.scene_view_mode.domain());
    let mut activate = None;
    for group_name in GROUPS {
        let matching = entries
            .iter()
            .filter(|class| group(class) == Some(*group_name))
            .filter(|class| class.cpp_name.to_lowercase().contains(&query))
            .collect::<Vec<_>>();
        if matching.is_empty() {
            continue;
        }
        ui.separator();
        ui.text_disabled(group_name);
        for class in matching {
            let label = crate::actor_document::short_class_name(&class.cpp_name);
            ui.selectable_config(format!("{label}##helper-{}", class.id))
                .allow_double_click(true)
                .build();
            crate::blueprint_editor::record_control(ui, &format!("helper:{}", class.id));
            if ui.is_item_hovered() && ui.is_mouse_double_clicked(imgui::MouseButton::Left) {
                activate = Some((class.id.clone(), class.family));
            }
            if let Ok(id) = uuid::Uuid::parse_str(&class.id)
                && let Some(_drag) = ui
                    .drag_drop_source_config(PAYLOAD)
                    .begin_payload(*id.as_bytes())
            {
                ui.text(label);
            }
            if ui.is_item_hovered() {
                ui.tooltip(|| {
                    ui.text(&class.cpp_name);
                    let owner = (class.family == ClassFamily::Component).then_some(e.selected_actor).flatten();
                    match preview(e, &class.id, owner) {
                        Ok(cost) => {
                            ui.text(format!("Adds {} actors and {} components", cost.actors, cost.components));
                            ui.text(format!("Level: {}/{} actors; largest owner: {}/{} components", cost.level_actors, crate::scene_bank::LEVEL_ACTOR_CAPACITY, cost.owner_components, crate::actor_components::CAPACITY));
                            for dependency in cost.dependencies { ui.text_wrapped(dependency); }
                            ui.text_wrapped("Object counts estimate runtime storage. Build's memory report measures linked RAM and executable size.");
                        }
                        Err(error) => ui.text_wrapped(error),
                    }
                    ui.text("Edit defaults, derive a class, or change placed properties through the normal class and component tools.");
                });
            }
        }
    }
    if let Some((id, family)) = activate {
        let actor = (family == ClassFamily::Component)
            .then_some(e.selected_actor)
            .flatten();
        if let Err(error) = apply(e, &id, actor) {
            e.log(&error);
            e.last_error = Some(error);
        }
    }
}

pub fn scene_drop(ui: &imgui::Ui, e: &mut Editor) {
    if let Some(target) = ui.drag_drop_target()
        && let Some(Ok(payload)) =
            target.accept_payload::<[u8; 16], _>(PAYLOAD, imgui::DragDropFlags::empty())
        && payload.delivery
        && let Err(error) = apply(e, &uuid::Uuid::from_bytes(payload.data).to_string(), None)
    {
        e.log(&error);
        e.last_error = Some(error);
    }
}

/// Inspector uses a window rectangle so empty space is a valid drop target.
pub fn inspector_drop(e: &mut Editor) {
    let delivered = unsafe {
        let window = imgui::sys::igGetCurrentWindow();
        let id = imgui::sys::igGetID_Str(c"attach-helper-class".as_ptr());
        if imgui::sys::igBeginDragDropTargetCustom((*window).InnerRect, id) {
            let payload = imgui::sys::igAcceptDragDropPayload(c"EPOK_HELPER_CLASS".as_ptr(), 0);
            let result = (!payload.is_null() && (*payload).Delivery && (*payload).DataSize == 16)
                .then(|| std::ptr::read_unaligned((*payload).Data.cast::<[u8; 16]>()));
            imgui::sys::igEndDragDropTarget();
            result
        } else {
            None
        }
    };
    if let Some(id) = delivered
        && let Err(error) = apply(e, &uuid::Uuid::from_bytes(id).to_string(), e.selected_actor)
    {
        e.log(&error);
        e.last_error = Some(error);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_helpers_duplicate_shared_bindings_reset_and_round_trip_in_both_domains() {
        let root = crate::workspace::tests::temp("world-helper-duplicates");
        let project =
            crate::workspace::create(&root, "Helpers", crate::workspace::Template::Basic).unwrap();
        let registry = crate::blueprint::native_registry(&root, &[]).unwrap();
        let mut e = Editor::open(project).unwrap();
        e.auto_build = false;
        e.class_registry = registry;
        e.registry_revision += 1;
        for domain in [3, 2] {
            let mut ids = Vec::new();
            for name in ["Actor", "Pickup", "Door", "Switch"] {
                e.create_actor(&format!("epok::{name}{domain}D"));
                assert!(e.last_error.is_none(), "{:?}", e.last_error);
                ids.push(e.selected_actor.unwrap());
            }
            e.add_actor_component(ids[0], "epok::GameplayFlagComponent");
            let flag = e.scene.actors[e.scene.actor_index(ids[0]).unwrap()]
                .components
                .last()
                .unwrap()
                .id;
            let mut endpoints = Vec::new();
            for (actor, field) in
                ids[1..]
                    .iter()
                    .zip(["collected_flag", "requirement_flag", "target_flag"])
            {
                let endpoint = e.scene.actors[e.scene.actor_index(*actor).unwrap()]
                    .components
                    .last()
                    .unwrap()
                    .id;
                e.edit_actor_property(
                    *actor,
                    endpoint,
                    crate::gui::PropertyAction::Set(field.into(), serde_json::json!(flag)),
                );
                endpoints.push(endpoint);
            }
            e.edit_actor_property(
                ids[1],
                endpoints[0],
                crate::gui::PropertyAction::Set(
                    "accepted_initiator".into(),
                    serde_json::json!(ids[0]),
                ),
            );
            e.edit_actor_property(
                ids[3],
                endpoints[2],
                crate::gui::PropertyAction::Set("toggle".into(), serde_json::json!(false)),
            );
            e.edit_actor_property(
                ids[3],
                endpoints[2],
                crate::gui::PropertyAction::Reset("toggle".into()),
            );
            let switch = e.scene.actors[e.scene.actor_index(ids[3]).unwrap()]
                .components
                .last()
                .unwrap();
            assert!(!switch.overrides.contains("toggle"));
            let before = e.scene.clone();
            let first = before.actors.len();
            e.duplicate_actors(&ids);
            assert!(e.last_error.is_none(), "{:?}", e.last_error);
            assert_eq!(e.scene.actors.len(), first + 4);
            let copies = &e.scene.actors[first..];
            let copied_flag = copies[0].components.last().unwrap().id;
            assert_ne!(copied_flag, flag);
            for (copy, field) in
                copies[1..]
                    .iter()
                    .zip(["collected_flag", "requirement_flag", "target_flag"])
            {
                assert_eq!(
                    copy.components.last().unwrap().properties[field],
                    serde_json::json!(copied_flag)
                );
            }
            assert_eq!(
                copies[1].components.last().unwrap().properties["accepted_initiator"],
                serde_json::json!(copies[0].id)
            );
            assert_eq!(&e.scene.actors[..first], before.actors.as_slice());
            let duplicated = e.scene.clone();
            e.undo_scene().unwrap();
            assert_eq!(e.scene, before);
            e.redo_scene().unwrap();
            assert_eq!(e.scene, duplicated);
            let path = root.join(format!("assets/scenes/Helpers{domain}D.epokmap"));
            e.scene.save(&path).unwrap();
            let reopened = crate::scene::Scene::load(&path).unwrap();
            assert_eq!(reopened.actors[first..], duplicated.actors[first..]);
        }
        drop(e);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn helper_preview_resolves_composition_without_writes_and_rejects_overflow() {
        let root = crate::workspace::tests::temp("helper-cost-preview");
        let project =
            crate::workspace::create(&root, "Helpers", crate::workspace::Template::Basic).unwrap();
        let registry = crate::blueprint::native_registry(&root, &[]).unwrap();
        let mut e = Editor::open(project).unwrap();
        e.auto_build = false;
        e.class_registry = registry;
        e.registry_revision += 1;
        let before = e.scene.clone();
        for (name, id) in helpers::WORLD_GAMEPLAY_CLASSES {
            if !name.ends_with("3D") {
                continue;
            }
            let cost = preview(&e, id, None).unwrap();
            assert_eq!(cost.actors, 1, "{name}");
            assert_eq!(
                cost.components,
                if name.contains("Door") { 3 } else { 2 },
                "{name}"
            );
            assert_eq!(cost.level_actors, before.actors.len() + 1);
        }
        let widget = preview(&e, crate::dialogue_layout::PRESENTER, None).unwrap();
        assert_eq!((widget.actors, widget.components), (12, 29));
        assert!(
            widget
                .dependencies
                .iter()
                .any(|s| s.contains("dialogue-frame.epokasset"))
        );
        assert!(!root.join("assets/GameHelpers").exists());
        assert_eq!(e.scene, before);
        assert!(!e.can_undo_scene());
        // Use the same bound as cook, including composed children, before any
        // installation, editor mutation or undo record can be published.
        while e.scene.actors.len() < crate::scene_bank::LEVEL_ACTOR_CAPACITY {
            let actor = crate::class_composition::actor(
                &e.object_model().unwrap(),
                crate::object_model::ACTOR3D_ID,
                &format!("Filler{}", e.scene.actors.len()),
            )
            .unwrap();
            e.scene.actors.push(actor);
        }
        let full = e.scene.clone();
        let error = apply(&mut e, helpers::WORLD_GAMEPLAY_CLASSES[4].1, None).unwrap_err();
        assert!(error.contains("capacity"), "{error}");
        assert_eq!(e.scene, full);
        assert!(!e.can_undo_scene());
        assert!(!root.join("assets/GameHelpers").exists());
        drop(e);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn helper_catalog_uses_resolved_class_identity_and_domain() {
        let root = crate::workspace::tests::temp("helper-catalog");
        crate::workspace::create(&root, "Helpers", crate::workspace::Template::Basic).unwrap();
        let mut registry = crate::blueprint::native_registry(&root, &[]).unwrap();
        let mut derived = registry.classes[helpers::CAMERA_CLASSES[2].1].clone();
        derived.id = uuid::Uuid::new_v4().to_string();
        derived.cpp_name = "ProjectFollowView".into();
        derived.parent = Some(helpers::CAMERA_CLASSES[2].1.into());
        derived.default_components.clear();
        derived.properties.clear();
        derived.functions.clear();
        derived.default_overrides.clear();
        let id = derived.id.clone();
        registry.classes.insert(id.clone(), derived);
        let model = Model::from_registry(&registry).unwrap();
        assert_eq!(group(model.class(&id).unwrap()), Some("Characters"));
        let names = |domain| {
            entries(&model, domain)
                .iter()
                .map(|class| class.cpp_name.as_str())
                .collect::<Vec<_>>()
        };
        assert!(names(Domain::World3D).contains(&"ProjectFollowView"));
        assert!(names(Domain::World3D).contains(&"epok::Player3D"));
        assert!(!names(Domain::World2D).contains(&"epok::Player3D"));
        assert!(names(Domain::World2D).contains(&"epok::Player2D"));
        assert!(names(Domain::UI).contains(&"epok::DialoguePresenter"));
        assert!(!names(Domain::UI).contains(&"epok::InteractableComponent"));
        assert!(!names(Domain::World3D).contains(&"epok::CharacterCommandSource3DComponent"));
        assert!(model.class(crate::object_model::ACTOR3D_ID).is_some());
        assert_eq!(
            group(model.class(crate::object_model::ACTOR3D_ID).unwrap()),
            None
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn helper_catalog_operations_keep_domain_capacity_and_undo_contracts() {
        let root = crate::workspace::tests::temp("helper-operations");
        let project =
            crate::workspace::create(&root, "Helpers", crate::workspace::Template::Basic).unwrap();
        let registry = crate::blueprint::native_registry(&root, &[]).unwrap();
        let mut e = Editor::open(project).unwrap();
        e.auto_build = false;
        e.class_registry = registry;
        e.registry_revision += 1;
        let initial = e.scene.clone();
        apply(&mut e, helpers::CHARACTER_INPUT_CLASSES[2].1, None).unwrap();
        let player = e.selected_actor.unwrap();
        let placed = e.scene.clone();
        assert_eq!(placed.actors.len(), initial.actors.len() + 1);
        assert!(e.can_undo_scene());
        assert!(apply(&mut e, helpers::CHARACTER_INPUT_CLASSES[3].1, None).is_err());
        assert_eq!(e.scene, placed);
        e.undo_scene().unwrap();
        assert_eq!(e.scene, initial);
        assert!(!e.can_undo_scene());
        e.redo_scene().unwrap();
        assert_eq!(e.scene, placed);
        apply(&mut e, helpers::INTERACTION_CLASSES[1].1, Some(player)).unwrap();
        let attached = e.scene.clone();
        assert!(apply(&mut e, helpers::INTERACTION_CLASSES[1].1, Some(player)).is_err());
        assert_eq!(e.scene, attached);
        e.undo_scene().unwrap();
        assert_eq!(e.scene, placed);
        e.redo_scene().unwrap();
        assert_eq!(e.scene, attached);
        e.playing = true;
        assert!(apply(&mut e, helpers::CAMERA_CLASSES[2].1, None).is_err());
        assert_eq!(e.scene, attached);
        e.playing = false;
        drop(e);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn helper_catalog_real_drag_places_and_attaches_with_persistent_identities() {
        let root = crate::workspace::tests::temp("helper-drag");
        let project =
            crate::workspace::create(&root, "Helpers", crate::workspace::Template::Basic).unwrap();
        let mut e = Editor::open(project).unwrap();
        e.auto_build = false;
        let mut context = crate::gui::tests::imgui_context();
        context.io_mut().display_size = [1100., 600.];
        context.io_mut().delta_time = 1. / 60.;
        context.fonts().build_rgba32_texture();
        fn frame(context: &mut imgui::Context, e: &mut Editor, inspector: bool) {
            let ui = context.frame();
            ui.window("Helpers")
                .position([0., 0.], imgui::Condition::Always)
                .size([500., 550.], imgui::Condition::Always)
                .build(|| draw(ui, e));
            ui.window("Destination")
                .position([510., 0.], imgui::Condition::Always)
                .size([500., 550.], imgui::Condition::Always)
                .build(|| {
                    ui.invisible_button("destination", [460., 480.]);
                    if inspector {
                        inspector_drop(e);
                    } else {
                        scene_drop(ui, e);
                    }
                });
            context.render();
        }
        fn drag(context: &mut imgui::Context, e: &mut Editor, id: &str, inspector: bool) {
            for _ in 0..3 {
                frame(context, e, inspector);
            }
            let source =
                crate::blueprint_editor::control_position(&format!("helper:{id}")).unwrap();
            context.io_mut().add_mouse_pos_event(source);
            frame(context, e, inspector);
            context
                .io_mut()
                .add_mouse_button_event(imgui::MouseButton::Left, true);
            frame(context, e, inspector);
            context
                .io_mut()
                .add_mouse_pos_event([source[0] + 16., source[1]]);
            frame(context, e, inspector);
            context.io_mut().add_mouse_pos_event([760., 180.]);
            frame(context, e, inspector);
            frame(context, e, inspector);
            context
                .io_mut()
                .add_mouse_button_event(imgui::MouseButton::Left, false);
            frame(context, e, inspector);
            frame(context, e, inspector);
        }
        let count = e.scene.actors.len();
        e.project_search = "TriggerBox3D".into();
        drag(
            &mut context,
            &mut e,
            helpers::INTERACTION_CLASSES[4].1,
            false,
        );
        assert_eq!(e.scene.actors.len(), count + 1, "{:?}", e.last_error);
        let id = e.selected_actor.unwrap();
        let placed = e.scene.clone();
        let components = e
            .scene
            .actors
            .iter()
            .find(|a| a.id == id)
            .unwrap()
            .components
            .len();
        e.project_search = "InteractorComponent".into();
        drag(
            &mut context,
            &mut e,
            helpers::INTERACTION_CLASSES[1].1,
            true,
        );
        let actor = e.scene.actors.iter().find(|a| a.id == id).unwrap();
        assert_eq!(actor.components.len(), components + 1, "{:?}", e.last_error);
        let component = actor.components.last().unwrap().id;
        let attached = e.scene.clone();
        e.undo_scene().unwrap();
        assert_eq!(e.scene, placed);
        e.redo_scene().unwrap();
        assert_eq!(e.scene, attached);
        let path = root.join("assets/scenes/Helpers.epokmap");
        e.scene.save(&path).unwrap();
        let reopened = crate::scene::Scene::load(&path).unwrap();
        assert!(
            reopened
                .actors
                .iter()
                .any(|a| a.id == id && a.components.iter().any(|c| c.id == component))
        );
        drop(context);
        drop(e);
        std::fs::remove_dir_all(root).unwrap();
    }
}
