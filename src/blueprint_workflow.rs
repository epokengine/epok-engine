//! Project-browser creation and opening for original Blueprint class assets.
use crate::{blueprint_asset as asset, reflection_schema as schema};
use std::{
    collections::BTreeMap,
    path::{Component, Path, PathBuf},
};

#[derive(Default)]
pub struct Creation {
    pub requested: bool,
    pub context: crate::actor_scripts::CreationContext,
    pub name: String,
    pub folder: String,
    pub parent: String,
    pub search: String,
    pub error: Option<String>,
    pub created: Option<PathBuf>,
    pub owner_domain: usize,
}

/// Family of the chain a new Blueprint would join, without building the resolved
/// model: a declared family wins, otherwise the native family roots are recognised by
/// id. Non-Actor roots are reflected objects and cannot be attached to an Actor.
pub(crate) fn parent_family(
    registry: &crate::blueprint::Registry,
    parent: &schema::Class,
) -> schema::ClassFamily {
    for class in registry.ancestry(&parent.cpp_name).into_iter().rev() {
        if let Some(declared) = class.family {
            return declared;
        }
        match class.id.as_str() {
            crate::object_model::ACTOR_ID => return schema::ClassFamily::Actor,
            crate::object_model::ACTOR_COMPONENT_ID => return schema::ClassFamily::Component,
            _ => {}
        }
    }
    schema::ClassFamily::Object
}

/// Lifecycle entry points seeded into a new Blueprint of this family. Actors and
/// components share the object-model events.
fn default_event_names(family: schema::ClassFamily) -> &'static [&'static str] {
    match family {
        schema::ClassFamily::Actor | schema::ClassFamily::Component => {
            &["begin_play", "tick", "end_play"]
        }
        _ => &[],
    }
}

/// Supply the lifecycle entry points supported by the PSX runtime for this family.
/// Implemented parent events keep their qualified parent dispatch.
pub(crate) fn ensure_default_events(
    draft: &mut asset::BlueprintAsset,
    registry: &crate::blueprint::Registry,
) -> bool {
    let Some(parent) = registry.classes.get(&draft.parent) else {
        return false;
    };
    let family = parent_family(registry, parent);
    let events = default_event_names(family);
    let ancestry = registry.ancestry(&parent.cpp_name);
    if events.is_empty() {
        return false;
    }
    let mut functions = BTreeMap::new();
    for class in ancestry {
        for function in &class.functions {
            functions.insert(function.name.as_str(), function);
        }
    }
    let mut changed = false;
    let mut y = if draft.functions.is_empty() {
        0.
    } else {
        draft
            .layout
            .positions
            .values()
            .map(|p| p[1])
            .fold(0_f32, f32::max)
            + 220.
    };
    for name in events.iter().copied() {
        let Some(function) = functions.get(name) else {
            continue;
        };
        if function.final_method
            || function.returns != schema::Type::Void
            || draft.functions.iter().any(|graph| graph.name == name)
        {
            continue;
        }
        let entry = uuid::Uuid::new_v4().to_string();
        let mut nodes = vec![asset::Node {
            id: entry.clone(),
            kind: asset::NodeKind::Entry,
            inputs: BTreeMap::new(),
            outputs: BTreeMap::new(),
        }];
        // Even an apparently empty native default may be overridden by an
        // intermediate parent. Never replace inherited behavior with a no-op.
        if !function.abstract_method {
            let call = uuid::Uuid::new_v4().to_string();
            nodes[0].outputs.insert("next".into(), vec![call.clone()]);
            nodes.push(asset::Node {
                id: call.clone(),
                kind: asset::NodeKind::CallParent,
                inputs: function
                    .parameters
                    .iter()
                    .map(|p| {
                        (
                            p.name.clone(),
                            asset::Input::Parameter {
                                name: p.name.clone(),
                            },
                        )
                    })
                    .collect(),
                outputs: BTreeMap::new(),
            });
            draft.layout.positions.insert(call, [340., y]);
        }
        draft.functions.push(asset::Graph {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            override_id: Some(function.id.clone()),
            timeline: None,
            parameters: function.parameters.clone(),
            returns: function.returns.clone(),
            entry: entry.clone(),
            nodes,
        });
        draft.layout.positions.insert(entry, [0., y]);
        y += 220.;
        changed = true;
    }
    changed
}

pub fn create(
    root: &Path,
    registry: &crate::blueprint::Registry,
    name: &str,
    folder: &str,
    parent: &str,
    concrete: bool,
) -> Result<PathBuf, String> {
    create_with_owner(root, registry, name, folder, parent, concrete, None)
}
fn create_with_owner(
    root: &Path,
    registry: &crate::blueprint::Registry,
    name: &str,
    folder: &str,
    parent: &str,
    concrete: bool,
    owner: Option<schema::Domain>,
) -> Result<PathBuf, String> {
    crate::workspace::validate_name(name)?;
    if !crate::scripts::identifier(name) {
        return Err("Blueprint names must be portable, non-reserved identifiers.".into());
    }
    let parent = registry
        .classes
        .get(parent)
        .ok_or("Select a reflected parent class.")?;
    if !crate::script_backend::can_derive(&crate::script_backend::blueprint_provider(), parent) {
        return Err("This class/provider does not permit Blueprint inheritance.".into());
    }
    let folder = PathBuf::from(folder.replace('\\', "/"));
    if folder.components().any(|c| !matches!(c, Component::Normal(s) if s.to_str().is_some_and(crate::scripts::identifier))) {
        return Err("Folder components must be portable identifiers inside assets/Blueprints.".into());
    }
    let directory = root.join("assets/Blueprints").join(folder);
    // Validate existing ancestors, including Windows junctions, before creating files.
    let relative = directory.join(format!("{name}.epokbp"));
    let relative = relative.strip_prefix(root).map_err(|e| e.to_string())?;
    let path = crate::assets::inside(root, &relative.to_string_lossy().replace('\\', "/"))?;
    let mut draft = asset::BlueprintAsset::new(name.into(), parent.id.clone());
    if parent_family(registry, parent) == schema::ClassFamily::Component {
        draft.component = owner.map(|domain| schema::ComponentContract {
            owners: [domain].into(),
            ..Default::default()
        });
    }
    ensure_default_events(&mut draft, registry);
    // Fulfil inherited abstract events only. Never suppress an implemented parent event.
    let mut functions = BTreeMap::new();
    for class in registry.ancestry(&parent.cpp_name) {
        for function in &class.functions {
            functions.insert(function.name.clone(), function);
        }
    }
    for function in functions.values().filter(|f| f.abstract_method) {
        if draft
            .functions
            .iter()
            .any(|graph| graph.name == function.name)
        {
            continue;
        }
        if function.returns != schema::Type::Void {
            if concrete {
                return Err(format!(
                    "Implement abstract function {} before attaching this class.",
                    function.name
                ));
            }
            continue;
        }
        let entry = uuid::Uuid::new_v4().to_string();
        let exit = uuid::Uuid::new_v4().to_string();
        draft.functions.push(asset::Graph {
            timeline: None,
            id: uuid::Uuid::new_v4().to_string(),
            name: function.name.clone(),
            override_id: Some(function.id.clone()),
            parameters: function.parameters.clone(),
            returns: function.returns.clone(),
            entry: entry.clone(),
            nodes: vec![
                asset::Node {
                    id: entry.clone(),
                    kind: asset::NodeKind::Entry,
                    inputs: BTreeMap::new(),
                    outputs: BTreeMap::from([("next".into(), vec![exit.clone()])]),
                },
                asset::Node {
                    id: exit.clone(),
                    kind: asset::NodeKind::Return,
                    inputs: BTreeMap::new(),
                    outputs: BTreeMap::new(),
                },
            ],
        });
        draft.layout.positions.insert(entry, [50., 100.]);
        draft.layout.positions.insert(exit, [350., 100.]);
    }
    let mut candidates = asset::load_all(root)?;
    if candidates
        .iter()
        .any(|a| a.asset.name.eq_ignore_ascii_case(name))
        || registry
            .classes
            .values()
            .any(|c| c.cpp_name.eq_ignore_ascii_case(name))
    {
        return Err(format!("A class named {name} already exists."));
    }
    candidates.push(asset::AssetFile::file(path.clone(), draft.clone()));
    let native_registry = crate::blueprint::Registry {
        classes: registry
            .classes
            .iter()
            .filter(|(_, class)| class.provider.id != "blueprint")
            .map(|(id, class)| (id.clone(), class.clone()))
            .collect(),
    };
    let compiled = crate::blueprint_compile::compile(root, &native_registry, &candidates).map_err(
        |errors| {
            errors
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        },
    )?;
    if concrete
        && compiled
            .registry
            .classes
            .get(&draft.id)
            .is_none_or(|class| class.abstract_class)
    {
        return Err(
            "Blueprint still has abstract events; use Create and implement them before attachment."
                .into(),
        );
    }
    let directory = path.parent().ok_or("Missing Blueprint directory")?;
    let mut missing = vec![];
    let mut ancestor = directory;
    while !ancestor.exists() {
        missing.push(ancestor.to_path_buf());
        ancestor = ancestor.parent().ok_or("Invalid Blueprint directory")?;
    }
    let result = std::fs::create_dir_all(directory)
        .map_err(|e| e.to_string())
        .and_then(|_| asset::create(&path, &draft));
    if result.is_err() {
        for directory in missing {
            let _ = std::fs::remove_dir(directory);
        }
    }
    result.map(|_| path)
}

pub fn begin(editor: &mut crate::editor::Editor, parent: Option<String>) {
    begin_with_context(editor, parent, Default::default());
}

pub fn begin_component(editor: &mut crate::editor::Editor, actor: uuid::Uuid) {
    begin_with_context(
        editor,
        Some(crate::object_model::ACTOR_COMPONENT_ID.into()),
        crate::actor_scripts::CreationContext::Component(actor),
    );
}

fn begin_with_context(
    editor: &mut crate::editor::Editor,
    parent: Option<String>,
    context: crate::actor_scripts::CreationContext,
) {
    if editor.playing {
        editor.log("Stop Play before creating a Blueprint.");
        return;
    }
    if editor.blueprint_editor.dirty() {
        editor.log("Save or revert the open Blueprint before creating another class.");
        return;
    }
    match crate::scripts::native_catalog(&editor.root)
        .and_then(|scripts| crate::blueprint::native_registry(&editor.root, &scripts))
    {
        Ok(registry) => {
            for (id, class) in registry.classes {
                editor.class_registry.classes.insert(id, class);
            }
            editor.registry_revision = editor.registry_revision.wrapping_add(1);
        }
        Err(error) => {
            editor.log(error);
            return;
        }
    }
    editor.blueprint_creation = Creation {
        requested: true,
        context,
        owner_domain: context
            .actor_index(&editor.scene, editor.selected)
            .and_then(|i| editor.scene.actors.get(i))
            .and_then(|a| {
                editor
                    .object_model()
                    .and_then(|m| a.class.resolve(&m).map(|c| c.domain))
            })
            .map_or(0, |d| match d {
                schema::Domain::World3D => 1,
                schema::Domain::World2D => 2,
                schema::Domain::UI => 3,
                _ => 0,
            }),
        parent: parent
            .or_else(|| {
                editor
                    .class_registry
                    .named("epok::ActorComponent")
                    .map(|c| c.id.clone())
            })
            .unwrap_or_default(),
        ..Default::default()
    };
}

pub fn draw(ui: &imgui::Ui, editor: &mut crate::editor::Editor) {
    crate::blueprint_debug_ui::draw(ui, editor);
    let mut creation = std::mem::take(&mut editor.blueprint_creation);
    let model = editor.object_model();
    if creation.requested {
        ui.open_popup("Create Blueprint class");
        creation.requested = false;
    }
    ui.modal_popup_config("Create Blueprint class").always_auto_resize(true).build(|| {
        ui.text_wrapped("A Blueprint inherits native or visual behavior and changes only explicit defaults/events.");
        let inputs_disabled = ui.begin_disabled(creation.created.is_some());
        ui.input_text("Asset name", &mut creation.name).hint("BP_Enemy").build();
        ui.input_text("Folder", &mut creation.folder).hint("Under assets/Blueprints").build();
        ui.input_text("Search parent classes", &mut creation.search).build();
        let query = creation.search.to_lowercase();
        ui.child_window("blueprint-parent-tree").size([540.,210.]).border(true).build(|| {
            for class in editor.class_registry.blueprint_parents() {
                if !creation.context.allows_parent(&editor.scene, model.as_deref(), class) {continue;}
                if !query.is_empty() && !class.cpp_name.to_lowercase().contains(&query) {continue;}
                let depth=editor.class_registry.ancestry(&class.cpp_name).len().saturating_sub(1);
                if ui.selectable_config(format!("{}{}{}", "  ".repeat(depth), class.cpp_name, if class.abstract_class {" (abstract)"} else {""})).selected(creation.parent==class.id).build() {creation.parent=class.id.clone();}
                #[cfg(test)]
                crate::gui::record_script_control(ui, &class.cpp_name);
            }
        });
        drop(inputs_disabled);
        if let Some(parent)=editor.class_registry.classes.get(&creation.parent) {
            ui.text(format!("Parent: {}",parent.cpp_name));
            ui.text_wrapped(match parent_family(&editor.class_registry, parent) {
                schema::ClassFamily::Actor => "Actor Blueprint: defines the object's class. A 3D object can have one actor class.",
                schema::ClassFamily::Component => "Component Blueprint: adds reusable behavior to the object's actor.",
                _ => "Object Blueprint: create the asset without attaching it to a scene object.",
            });
            if parent_family(&editor.class_registry,parent)==schema::ClassFamily::Component {
                let choices=[("Inherit parent compatibility",None),("Actor3D",Some(schema::Domain::World3D)),("Actor2D",Some(schema::Domain::World2D)),("UIActor",Some(schema::Domain::UI))];
                let target_domain=creation.context.actor_index(&editor.scene,editor.selected)
                    .and_then(|index| model.as_deref().and_then(|model| editor.scene.actors[index].class.resolve(model).map(|class| class.domain)));
                if let Some(_combo)=ui.begin_combo("Compatible Actors",choices[creation.owner_domain.min(3)].0) {
                    for (index,(label,domain)) in choices.iter().enumerate() {
                        if matches!(creation.context,crate::actor_scripts::CreationContext::Component(_)) && domain.is_some() && *domain!=target_domain {continue;}
                        if ui.selectable_config(label).selected(creation.owner_domain==index).build() {creation.owner_domain=index;}
                    }
                }
            }
            ui.child_window("blueprint-inherited").size([540.,120.]).border(true).build(|| {
                for property in editor.class_registry.properties(&parent.cpp_name) {ui.bullet_text(format!("{}: {} = {}",property.name,property.value_type.label(),property.default));}
                for class in editor.class_registry.ancestry(&parent.cpp_name) {for function in &class.functions {if function.event {ui.bullet_text(format!("Event {}({})", function.name,function.parameters.iter().map(|p|p.value_type.label()).collect::<Vec<_>>().join(", ")));}}}
            });
        }
        if let Some(error)=&creation.error {ui.text_colored([1.,0.5,0.4,1.],error);}
        let owner=match creation.owner_domain {1=>Some(schema::Domain::World3D),2=>Some(schema::Domain::World2D),3=>Some(schema::Domain::UI),_=>None};
        let parent_allowed = editor.class_registry.blueprint_parents().any(|parent| parent.id == creation.parent && creation.context.allows_parent(&editor.scene, model.as_deref(), parent));
        let attachment_index = creation.context.actor_index(&editor.scene, editor.selected);
        let attachment_error = attachment_index.ok_or_else(|| "Select an Actor to enable Create and Attach.".to_string())
            .and_then(|index| crate::actor_scripts::validate_parent_with_owner(&editor.scene, index, &creation.parent, &editor.class_registry,owner)).err();
        if let Some(error) = &attachment_error { ui.text_wrapped(error); }
        if crate::gui::script_button(ui,"Cancel") {ui.close_current_popup();}
        ui.same_line(); let create_only={let _disabled=ui.begin_disabled(!parent_allowed); crate::gui::script_button(ui,if creation.created.is_some() {"Open Created Blueprint"} else {"Create"})};
        ui.same_line(); let attach={let _disabled=ui.begin_disabled(editor.playing || !parent_allowed || attachment_error.is_some()); crate::gui::script_button(ui, if creation.created.is_some() {"Attach Created Blueprint"} else {"Create and Attach"})};
        if create_only || attach {
            let owner=match creation.owner_domain {1=>Some(schema::Domain::World3D),2=>Some(schema::Domain::World2D),3=>Some(schema::Domain::UI),_=>None};
            let result = creation.created.clone().map(Ok).unwrap_or_else(|| create_with_owner(&editor.root,&editor.class_registry,creation.name.trim(),creation.folder.trim(),&creation.parent,attach,owner));
            match result {
                Ok(path) => {
                    creation.created = Some(path.clone());
                    editor.refresh_scripts(); editor.assets.refresh();
                    if attach { editor.select_actor(attachment_index.map(|index| editor.scene.actors[index].id)); }
                    match if attach { attach_asset(editor, &path) } else { Ok(()) } {
                        Ok(()) => {
                            if let Err(error)=editor.blueprint_editor.open(&path) {editor.log(error);}
                            ui.close_current_popup();
                        }
                        Err(error) => {
                            editor.log(format!("Blueprint created, but could not be attached: {error}"));
                            creation.error=Some(format!("Blueprint created, but could not be attached: {error}"));
                        }
                    }
                }
                Err(error) => creation.error=Some(error),
            }
        }
    });
    editor.blueprint_creation = creation;
    if editor
        .blueprint_editor
        .draw_with_options(ui, &editor.class_registry, |ui| {
            let _disabled = ui.begin_disabled(editor.playing || editor.job.is_some());
            ui.checkbox(
                "Instrument Blueprint Debugger",
                &mut editor.blueprint_debug_enabled,
            );
            let _color = ui.push_style_color(
                imgui::StyleColor::Text,
                ui.style_color(imgui::StyleColor::TextDisabled),
            );
            ui.text_wrapped(
                "Applies to the next Build / Play. Release builds remain uninstrumented.",
            );
        })
    {
        editor.refresh_scripts();
        editor.assets.refresh();
    }
    // A map's own Blueprint is part of the map document: its edits become scene
    // edits (dirty, Undo, Save) and its Save button saves the map.
    editor.sync_embedded_blueprint();
    if std::mem::take(&mut editor.blueprint_editor.save_to_map) {
        editor.save_all();
        editor.refresh_scripts();
    }
    if editor.blueprint_editor.compile_requested {
        editor.blueprint_editor.compile_requested = false;
        let valid = editor
            .blueprint_editor
            .compile(&editor.root, &editor.class_registry);
        if valid {
            editor.log("Blueprint compilation succeeded.");
        } else {
            editor.log(format!(
                "Blueprint compilation failed. Build / Play is blocked.\n{}",
                editor.blueprint_editor.diagnostics.join("\n")
            ));
        }
    }
    if std::mem::take(&mut editor.blueprint_editor.place_requested)
        && let Err(error) = place_current(editor, None)
    {
        editor.log(error);
    }
    if std::mem::take(&mut editor.blueprint_editor.derived_requested) {
        let parent = editor.blueprint_editor.asset.as_ref().map(|a| a.id.clone());
        begin(editor, parent);
    }
    if std::mem::take(&mut editor.blueprint_editor.capture_requested) {
        ui.open_popup("Capture Blueprint template");
    }
    ui.modal_popup_config("Capture Blueprint template").always_auto_resize(true).build(||{
        ui.text_wrapped("Replace the open Blueprint's local entity template with the selected scene subtree? Components and internal references are captured. The Blueprint parent does not change. Existing local construction operations are replaced by their captured result. This remains unsaved and can be undone in the Blueprint editor.");
        if ui.button("Capture subtree") {match capture_current(editor){Ok(())=>ui.close_current_popup(),Err(error)=>editor.log(error)}}
        ui.same_line();if ui.button("Cancel##capture-blueprint"){ui.close_current_popup();}
    });
}

pub fn edit_binding(editor: &mut crate::editor::Editor, binding: &crate::scene::ClassDefaults) {
    let class = editor.class_registry.bound(binding);
    let visual =
        class.is_some_and(|c| c.provider.id == "blueprint") || binding.provider.id == "blueprint";
    if visual {
        let path = class.map(|c| c.source.file.clone()).or_else(|| {
            asset::load_all(&editor.root)
                .ok()?
                .into_iter()
                .find(|f| {
                    binding
                        .class_id
                        .as_ref()
                        .is_some_and(|id| *id == f.asset.id)
                        || f.asset.name == binding.name
                })
                .map(|f| f.path)
        });
        match path {
            Some(path) => {
                if let Err(error) = editor.blueprint_editor.open(&path) {
                    editor.log(error);
                }
            }
            None => editor.log("Blueprint source is unavailable. Instance values are preserved."),
        }
    } else {
        let source = class.and_then(|class| {
            crate::scripts::editable_class_source(&editor.root, class)
                .map(|path| (path, class.source.line as usize))
        });
        if let Some((path, line)) = source {
            editor.open_code(&path, Some(line.max(1)));
        } else {
            editor.log("Engine classes are read-only. Create a derived class in the project to customize their behavior.");
        }
    }
}

fn binding(class: &schema::Class) -> crate::scene::ClassDefaults {
    crate::scene::ClassDefaults {
        name: class.cpp_name.clone(),
        class_id: Some(class.id.clone()),
        provider: class.provider.clone(),
        backend: class.backend.clone(),
        ..Default::default()
    }
}
/// Assign a saved Blueprint using its family's runtime representation.
pub fn attach_asset(editor: &mut crate::editor::Editor, path: &Path) -> Result<(), String> {
    if editor.playing {
        return Err("Stop Play before assigning a Blueprint.".into());
    }
    let index = editor
        .selected
        .filter(|&i| i < editor.scene.actors.len())
        .ok_or("Select an object in Scene or Hierarchy before assigning a Blueprint.")?;
    let doc = asset::load(path)?;
    if editor.blueprint_editor.dirty()
        && editor
            .blueprint_editor
            .asset
            .as_ref()
            .is_some_and(|a| a.id == doc.id)
    {
        return Err("Save the Blueprint before assigning it to an object.".into());
    }
    // Resolve from current sources: a failed refresh must never attach a stale
    // cached class, or another class that happens to share its display name.
    let catalog = crate::scripts::catalog(&editor.root)?;
    let registry = crate::blueprint::native_registry(&editor.root, &catalog)?;
    let class = registry
        .classes
        .get(&doc.id)
        .filter(|c| c.provider.id == "blueprint")
        .ok_or("The Blueprint is unavailable. Resolve its compilation errors first.")?;
    if class.abstract_class {
        return Err("Implement the Blueprint's abstract functions before assigning it.".into());
    }
    let before = editor.scene.clone();
    let scene = crate::actor_scripts::assign(&before, index, class, &registry)?;
    editor.catalog = catalog;
    editor.class_registry = registry;
    editor.registry_revision = editor.registry_revision.wrapping_add(1);
    commit_scene(editor, before, scene, Some(index));
    editor.log(format!(
        "Assigned {} to {}.",
        doc.name, editor.scene.actors[index].name
    ));
    Ok(())
}
pub(crate) fn commit_scene(
    editor: &mut crate::editor::Editor,
    before: crate::scene::Scene,
    scene: crate::scene::Scene,
    selected: Option<usize>,
) {
    if before == scene {
        return;
    }
    editor.scene = scene;
    editor.selected = selected;
    editor.script_undo.push((before, editor.scene.clone()));
    if editor.script_undo.len() > 32 {
        editor.script_undo.remove(0);
    }
    editor.script_redo.clear();
    editor.reset_instance_baseline();
    editor.changed();
    editor.reveal_selected = true;
}
pub fn place_current(
    editor: &mut crate::editor::Editor,
    parent: Option<usize>,
) -> Result<(), String> {
    if editor.playing {
        return Err("Stop Play before placing Blueprint actors.".into());
    }
    if editor.blueprint_editor.dirty() {
        return Err("Save the Blueprint before placing a linked instance.".into());
    }
    let id = editor
        .blueprint_editor
        .asset
        .as_ref()
        .ok_or("Open a Blueprint first.")?
        .id
        .clone();
    let class = editor
        .class_registry
        .classes
        .get(&id)
        .ok_or("Compile and save the Blueprint before placement.")?;
    if class.abstract_class {
        return Err("Abstract Blueprint classes cannot be placed.".into());
    }
    let files = asset::load_all(&editor.root)?;
    let template = crate::blueprint_templates::resolve_assets(&files, &editor.class_registry, &id)?;
    crate::blueprint_templates::validate_resources(&template, &editor.assets.index)?;
    let before = editor.scene.clone();
    let mut candidate = before.clone();
    let placement = crate::blueprint_templates::place(
        &mut candidate,
        &template,
        binding(class),
        &editor.class_registry,
        parent,
    )?;
    let count = placement.actors.len();
    if placement.identities.len() < count {
        return Err("Template placement produced an invalid identity map; scene preserved.".into());
    }
    commit_scene(editor, before, candidate, Some(placement.root));
    editor.log(format!(
        "Placed {count} linked Blueprint actors. Undo restores the previous scene."
    ));
    Ok(())
}
pub fn capture_current(editor: &mut crate::editor::Editor) -> Result<(), String> {
    if editor.playing {
        return Err("Stop Play before capturing an entity template.".into());
    }
    let selected = editor
        .selected
        .ok_or("Select the root of the scene subtree to capture.")?;
    let doc = editor
        .blueprint_editor
        .asset
        .as_ref()
        .ok_or("Open a Blueprint first.")?
        .clone();
    let files = asset::load_all(&editor.root)?;
    if editor
        .class_registry
        .classes
        .get(&doc.parent)
        .is_some_and(|c| c.provider.id == "blueprint")
    {
        let inherited = crate::blueprint_templates::resolve_assets(
            &files,
            &editor.class_registry,
            &doc.parent,
        )?;
        if !inherited.actors.is_empty() {
            return Err("This Blueprint inherits an entity template. Edit its inherited members/children in Actor Template instead of replacing its root with a capture.".into());
        }
    }
    let mut template =
        crate::blueprint_templates::capture(&editor.scene, selected, &editor.class_registry)?;
    let selected_indices: Vec<_> = (0..editor.scene.actors.len())
        .filter(|&i| editor.scene.is_descendant(i, selected))
        .collect();
    let existing_root = doc
        .template
        .actors
        .iter()
        .find(|e| e.parent.is_none())
        .map(|e| e.entity.id);
    let mut ids = BTreeMap::new();
    for (entity, index) in template.actors.iter().zip(selected_indices) {
        let source = &editor.scene.actors[index];
        let stable = if index == selected {
            existing_root
        } else {
            source
                .blueprint_instance
                .as_ref()
                .filter(|i| i.class == doc.id)
                .map(|i| i.template_entity)
        };
        ids.insert(entity.entity.id, stable.unwrap_or(entity.entity.id));
    }
    for entity in &mut template.actors {
        entity.entity.id = ids[&entity.entity.id];
        entity.parent = entity.parent.map(|id| ids[&id]);
    }
    for reference in &mut template.references {
        reference.owner = ids[&reference.owner];
        reference.target = ids[&reference.target];
    }
    let resolved = crate::blueprint_templates::resolve(&[&template])?;
    crate::blueprint_templates::validate_resources(&resolved, &editor.assets.index)?;
    editor.blueprint_editor.replace_template(template)?;
    editor.blueprint_editor.compile_requested = true;
    editor.log("Captured entity subtree into the open Blueprint. Save to propagate it to linked instances.");
    Ok(())
}

pub fn instance_inspector(ui: &imgui::Ui, editor: &mut crate::editor::Editor, index: usize) {
    let Some(instance) = editor
        .scene
        .actors
        .get(index)
        .and_then(|e| e.blueprint_instance.clone())
    else {
        return;
    };
    if !ui.collapsing_header(
        "Linked Blueprint instance",
        imgui::TreeNodeFlags::DEFAULT_OPEN,
    ) {
        return;
    }
    let _disabled = ui.begin_disabled(editor.playing);
    let class = editor.class_registry.classes.get(&instance.class).cloned();
    ui.text(
        class
            .as_ref()
            .map(|c| c.cpp_name.as_str())
            .unwrap_or("Unavailable Blueprint (preserved)"),
    );
    if ui.small_button("Open Blueprint")
        && let Some(class) = &class
    {
        edit_binding(editor, &binding(class));
    }
    ui.text_disabled(format!(
        "{} explicit entity/component overrides",
        instance.overrides.len()
    ));
    let mut reset = None;
    for key in instance.overrides.keys() {
        if ui.small_button(format!("Reset {key}")) {
            reset = Some(key.clone());
        }
    }
    if instance.parent.is_some() && ui.small_button("Reset parent to inherited") {
        reset = Some("parent".into());
    }
    if ui.button("Reset all instance member overrides") {
        reset = Some("all".into());
    }
    if let Some(key) = reset {
        let before = editor.scene.clone();
        let mut candidate = before.clone();
        let state = candidate.actors[index].blueprint_instance.as_mut().unwrap();
        match key.as_str() {
            "all" => {
                state.overrides.clear();
                state.parent = None;
            }
            "parent" => state.parent = None,
            _ => {
                state.overrides.remove(&key);
            }
        }
        match asset::load_all(&editor.root).and_then(|files| {
            crate::blueprint_templates::refresh_instances(
                &mut candidate,
                &files,
                &editor.class_registry,
            )
        }) {
            Ok(()) => commit_scene(editor, before, candidate, Some(index)),
            Err(error) => editor.log(format!("Reset rejected; overrides preserved: {error}")),
        }
    }
    if ui.button("Unlink complete instance") {
        let before = editor.scene.clone();
        let mut candidate = before.clone();
        for entity in &mut candidate.actors {
            if entity
                .blueprint_instance
                .as_ref()
                .is_some_and(|i| i.instance == instance.instance)
            {
                entity.blueprint_instance = None;
            }
        }
        commit_scene(editor, before, candidate, Some(index));
        editor.log("Blueprint instance unlinked; current actors/components remain. Undo restores the link.");
    }
    ui.text_wrapped("Unchanged members follow the class template. User edits create explicit overrides; Reset restores inheritance. Unlink keeps the current scene objects.");
    ui.separator();
}
