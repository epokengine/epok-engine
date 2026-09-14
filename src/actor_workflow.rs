//! Scene-first Actor creation and promotion of a placement to a reusable Blueprint.
use crate::{
    editor::Editor,
    object_model as om,
    reflection_schema::{ClassFamily, Domain},
};

#[derive(Default)]
pub struct State {
    pub requested: bool,
    pub search: String,
    pub class: String,
    pub parent: Option<uuid::Uuid>,
    pub convert: Option<uuid::Uuid>,
    pub name: String,
    pub error: Option<String>,
}
pub fn begin(e: &mut Editor, parent: Option<uuid::Uuid>) {
    let class = match e.scene_view_mode.domain() {
        Domain::World3D => om::ACTOR3D_ID,
        Domain::World2D => om::ACTOR2D_ID,
        Domain::UI => om::UI_ACTOR_ID,
        Domain::None => om::ACTOR3D_ID,
    };
    e.actor_creation = State {
        requested: true,
        class: class.into(),
        parent,
        ..Default::default()
    };
}
pub fn begin_convert(e: &mut Editor, id: uuid::Uuid) {
    let Some(actor) = e.scene.actors.iter().find(|a| a.id == id) else {
        return;
    };
    let name = actor
        .name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();
    e.actor_creation = State {
        requested: true,
        convert: Some(id),
        name: format!("BP_{name}"),
        ..Default::default()
    };
}
fn class_tree(
    ui: &imgui::Ui,
    classes: &[&om::ClassModel],
    parent: Option<&str>,
    query: &str,
    selected: &mut String,
) {
    for class in classes.iter().copied().filter(|c| {
        c.parent
            .as_deref()
            .filter(|p| classes.iter().any(|c| c.id == *p))
            == parent
    }) {
        let direct = class.cpp_name.to_lowercase().contains(query);
        let has_matching_child = classes.iter().any(|child| {
            child.cpp_name.to_lowercase().contains(query) && child.ancestry.contains(&class.id)
        });
        if !query.is_empty() && !direct && !has_matching_child {
            continue;
        }
        let has_children = classes
            .iter()
            .any(|c| c.parent.as_deref() == Some(&class.id));
        let mut flags = imgui::TreeNodeFlags::OPEN_ON_ARROW
            | imgui::TreeNodeFlags::SPAN_AVAIL_WIDTH
            | imgui::TreeNodeFlags::DEFAULT_OPEN;
        if !has_children {
            flags |= imgui::TreeNodeFlags::LEAF;
        }
        if selected == &class.id {
            flags |= imgui::TreeNodeFlags::SELECTED;
        }
        let node = ui
            .tree_node_config(format!(
                "{}###{}",
                crate::actor_document::short_class_name(&class.cpp_name),
                class.id
            ))
            .flags(flags)
            .push();
        if ui.is_item_clicked()
            && !ui.is_item_toggled_open()
            && class.instantiable()
            && class.placement.placeable
        {
            *selected = class.id.clone();
        }
        if class.abstract_class {
            ui.same_line();
            ui.text_disabled("abstract");
        }
        if let Some(_node) = node {
            class_tree(ui, classes, Some(&class.id), query, selected);
        }
    }
}
pub fn draw(ui: &imgui::Ui, e: &mut Editor) {
    let mut state = std::mem::take(&mut e.actor_creation);
    if state.requested {
        ui.open_popup(if state.convert.is_some() {
            "Convert to Actor Blueprint"
        } else {
            "Instantiate Actor"
        });
        state.requested = false;
    }
    ui.modal_popup_config("Instantiate Actor").always_auto_resize(true).build(|| {
        ui.text(format!("Actor classes for the {} view",e.scene_view_mode.label()));
        ui.set_next_item_width(500.);
        ui.input_text("Search classes",&mut state.search).build();
        let model=e.object_model();
        ui.child_window("actor-class-tree").size([550.,320.]).border(true).build(|| {
            if let Some(model)=&model {
                let mut classes=model.iter().filter(|c|c.family==ClassFamily::Actor && c.domain==e.scene_view_mode.domain() && !c.placement.scene_managed).collect::<Vec<_>>();
                classes.sort_by(|a,b|a.cpp_name.cmp(&b.cpp_name));
                class_tree(ui,&classes,None,&state.search.to_lowercase(),&mut state.class);
            } else {ui.text_wrapped("The class catalog is unavailable. Resolve the project compilation errors to create an Actor.");}
        });
        let selected=model.as_ref().and_then(|m|m.class(&state.class));
        if let Some(class)=selected {ui.text(format!("Class: {}",class.cpp_name));}
        let disabled=ui.begin_disabled(e.playing || selected.is_none_or(|c|!c.instantiable() || !c.placement.placeable));
        if crate::gui::script_button(ui,"Instantiate") {
            e.create_actor(&state.class);
            if e.last_error.is_none() {
                if let Some(id)=e.selected_actor && state.parent.is_some() {e.reparent_actor(id,state.parent);}
                ui.close_current_popup();
            } else {state.error=e.last_error.clone();}
        }
        drop(disabled);
        ui.same_line();if ui.button("Cancel") {ui.close_current_popup();}
        if let Some(error)=&state.error {ui.text_wrapped(error);}
    });
    ui.modal_popup_config("Convert to Actor Blueprint")
        .always_auto_resize(true)
        .build(|| {
            ui.text_wrapped("Save this Actor and its child Actors as a reusable Blueprint class.");
            ui.set_next_item_width(440.);
            ui.input_text("Asset name", &mut state.name).build();
            ui.text_disabled("assets/Blueprints");
            if ui.button("Create Blueprint") {
                match state
                    .convert
                    .ok_or("Select an Actor first.".into())
                    .and_then(|id| convert(e, id, state.name.trim()))
                {
                    Ok(()) => ui.close_current_popup(),
                    Err(error) => state.error = Some(error),
                }
            }
            ui.same_line();
            if ui.button("Cancel##convert") {
                ui.close_current_popup();
            }
            if let Some(error) = &state.error {
                ui.text_wrapped(error);
            }
        });
    e.actor_creation = state;
}

pub fn convert(e: &mut Editor, id: uuid::Uuid, name: &str) -> Result<(), String> {
    if e.playing {
        return Err("Stop Play before converting an Actor.".into());
    }
    crate::workspace::validate_name(name)?;
    if !crate::scripts::identifier(name) {
        return Err("Use a valid class name with letters, digits and underscores.".into());
    }
    e.scene.sync_actor_components();
    let index = e
        .scene
        .actor_index(id)
        .ok_or("The selected Actor no longer exists.")?;
    let parent = e.scene.actors[index]
        .class
        .class_id
        .clone()
        .ok_or("Resolve the Actor's class before converting it.")?;
    let mut asset = crate::blueprint_asset::BlueprintAsset::new(name.into(), parent);
    asset.family = Some(ClassFamily::Actor);
    asset.template = crate::blueprint_templates::capture(&e.scene, index, &e.class_registry)?;
    crate::blueprint_workflow::ensure_default_events(&mut asset, &e.class_registry);
    let path = crate::assets::inside(&e.root, &format!("assets/Blueprints/{name}.epokbp"))?;
    let mut files = crate::blueprint_asset::load_all(&e.root)?;
    files.push(crate::blueprint_asset::AssetFile::file(
        path.clone(),
        asset.clone(),
    ));
    let native = crate::blueprint::Registry {
        classes: e
            .class_registry
            .classes
            .iter()
            .filter(|(_, c)| c.provider.id != "blueprint")
            .map(|(id, c)| (id.clone(), c.clone()))
            .collect(),
    };
    crate::blueprint_compile::compile(&e.root, &native, &files).map_err(|errors| {
        errors
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    if let Some(directory) = path.parent() {
        std::fs::create_dir_all(directory).map_err(|err| err.to_string())?;
    }
    let branch = e.scene.actor_branch(id);
    let mut candidate = e.scene.clone();
    let mut items = asset.template.actors.iter();
    for actor in candidate
        .actors
        .iter_mut()
        .filter(|a| branch.contains(&a.id))
    {
        let template = items
            .next()
            .ok_or("Captured Actor hierarchy is incomplete.")?;
        actor.data.blueprint_instance = Some(crate::blueprint_templates::Instance {
            class: asset.id.clone(),
            template_entity: template.entity.id,
            instance: id,
            component_ids: template
                .entity
                .components
                .iter()
                .zip(&actor.components)
                .map(|(a, b)| (a.id, b.id))
                .collect(),
            overrides: Default::default(),
            parent: None,
        });
        if actor.id == id {
            actor.class = crate::actor_document::ClassReference::new(name, &asset.id);
        }
    }
    candidate.validate()?;
    crate::blueprint_asset::create(&path, &asset)?;
    let before = e.scene.clone();
    crate::blueprint_workflow::commit_scene(e, before, candidate, Some(index));
    e.refresh_scripts();
    e.assets.refresh();
    e.selected_asset = None;
    e.select_actor(Some(id));
    e.log(format!(
        "Created assets/Blueprints/{name}.epokbp and linked the selected Actor."
    ));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn domains_components_and_reusable_placements_preserve_identity_and_overrides() {
        let root = crate::workspace::tests::temp("actor-workflow");
        let project =
            crate::workspace::create(&root, "Actors", crate::workspace::Template::Basic).unwrap();
        let mut editor = Editor::open(project).unwrap();
        editor.auto_build = false;
        editor.create_actor(om::ACTOR3D_ID);
        assert!(editor.last_error.is_none(), "{:?}", editor.last_error);
        let hero = editor.selected_actor.unwrap();
        editor.create_actor(om::ACTOR2D_ID);
        let sprite = editor.selected_actor.unwrap();
        editor.create_actor(om::UI_ACTOR_ID);
        let ui = editor.selected_actor.unwrap();
        for (id, root_class) in [
            (hero, om::SCENE_COMPONENT3D_ID),
            (sprite, om::SCENE_COMPONENT2D_ID),
            (ui, om::RECT_TRANSFORM_COMPONENT_ID),
        ] {
            let actor = &editor.scene.actors[editor.scene.actor_index(id).unwrap()];
            assert_eq!(
                actor.root().unwrap().class.class_id.as_deref(),
                Some(root_class)
            );
        }
        editor.reparent_actor(ui, Some(hero));
        let panel = &editor.scene.actors[editor.scene.actor_index(ui).unwrap()];
        assert_eq!(panel.logical_parent, Some(hero));
        assert!(panel.attach.is_none());
        editor.reparent_actor(ui, None);
        editor.select_actor(Some(hero));
        let bp = crate::blueprint_workflow::create(
            &root,
            &editor.class_registry,
            "BP_Logic",
            "",
            om::ACTOR_COMPONENT_ID,
            true,
        )
        .unwrap();
        editor.refresh_scripts();
        crate::blueprint_workflow::attach_asset(&mut editor, &bp).unwrap();
        let index = editor.scene.actor_index(hero).unwrap();
        let component = editor.scene.actors[index]
            .components
            .iter()
            .find(|c| c.class.name == "BP_Logic")
            .unwrap()
            .id;
        editor.add_actor_component(hero, "epok::AudioComponent");
        editor.add_actor_component(hero, "epok::AudioComponent");
        assert!(editor.last_error.is_none(), "{:?}", editor.last_error);
        editor.scene.actors[index].position = [2., 3., 4.];
        editor.changed();
        convert(&mut editor, hero, "BP_Reusable").unwrap();
        assert_eq!(editor.scene.actors[index].id, hero);
        assert!(
            editor.scene.actors[index]
                .components
                .iter()
                .any(|c| c.id == component)
        );
        let ids = editor.scene.actors[index]
            .components
            .iter()
            .map(|c| c.id)
            .collect::<Vec<_>>();
        editor.create_actor("BP_Reusable");
        assert!(editor.last_error.is_none(), "{:?}", editor.last_error);
        let copy = editor.selected_actor.unwrap();
        assert_ne!(copy, hero);
        let other = editor.scene.actor_index(copy).unwrap();
        assert_eq!(editor.scene.actors[other].position, [2., 3., 4.]);
        assert!(
            editor.scene.actors[other]
                .components
                .iter()
                .all(|c| !ids.contains(&c.id))
        );
        let before = editor.scene.actors[index].clone();
        editor.scene.actors[index].position = [9., 0., 0.];
        crate::blueprint_templates::record_overrides(&before, &mut editor.scene.actors[index]);
        let files = crate::blueprint_asset::load_all(&root).unwrap();
        crate::blueprint_templates::refresh_instances(
            &mut editor.scene,
            &files,
            &editor.class_registry,
        )
        .unwrap();
        assert_eq!(editor.scene.actors[index].position, [9., 0., 0.]);
        assert_eq!(editor.scene.actors[other].position, [2., 3., 4.]);
        assert_eq!(
            editor.scene.actors[index]
                .components
                .iter()
                .map(|c| c.id)
                .collect::<Vec<_>>(),
            ids
        );
        let path = root.join("assets/scenes/Actors.epokmap");
        editor.scene.save(&path).unwrap();
        let loaded = crate::scene::Scene::load(&path).unwrap();
        assert_eq!(editor.scene, loaded);
        crate::project::stage(&root, &loaded).unwrap();
        drop(editor);
        std::fs::remove_dir_all(root).unwrap();
    }
}
