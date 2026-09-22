//! Project operations shared by MCP transports; never called from a network thread.
use crate::{
    assets,
    editor::Editor,
    mcp::State,
    scene::{Actor, Scene},
};
use base64::Engine as _;
use serde_json::{Value, json};
use std::{fs, path::Path};

fn string(value: &Value, key: &str) -> Result<String, String> {
    value[key]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| format!("'{key}' must be a string"))
}
fn decode<T: serde::de::DeserializeOwned>(value: Value) -> Result<T, String> {
    let mut ignored = Vec::new();
    let result = serde_ignored::deserialize(value, |path| ignored.push(path.to_string()))
        .map_err(|e| e.to_string())?;
    if !ignored.is_empty() {
        return Err(format!("Unknown fields: {}", ignored.join(", ")));
    }
    Ok(result)
}
fn inside(root: &Path, relative: &str) -> Result<std::path::PathBuf, String> {
    if relative.contains(':') {
        return Err("Use a relative path inside assets/".into());
    }
    let path = assets::inside(root, relative)?;
    let ancestor = path
        .ancestors()
        .find(|p| p.exists())
        .ok_or("Missing assets folder")?;
    let assets_root = fs::canonicalize(root.join("assets")).map_err(|e| e.to_string())?;
    if !fs::canonicalize(ancestor)
        .map_err(|e| e.to_string())?
        .starts_with(assets_root)
    {
        return Err("MCP asset paths cannot link to other project directories".into());
    }
    Ok(path)
}
pub fn revision(scene: &Scene) -> String {
    assets::hash(&serde_json::to_vec(scene).expect("Serializable scene"))
}
fn expect_revision(scene: &Scene, args: &Value) -> Result<(), String> {
    if string(args, "revision")? != revision(scene) {
        return Err("Scene changed. Read scene_read and retry with its revision.".into());
    }
    Ok(())
}
fn editable(e: &Editor) -> Result<(), String> {
    if e.job.is_some() || e.dependencies.busy() || e.assets.busy || e.bake_job.is_some() {
        return Err("Wait for installation/build/import/bake or stop Play before editing.".into());
    }
    Ok(())
}
fn index(scene: &Scene, args: &Value) -> Result<usize, String> {
    args["index"]
        .as_u64()
        .and_then(|v| usize::try_from(v).ok())
        .filter(|i| *i < scene.actors.len())
        .ok_or("Actor index is out of range".into())
}
fn merge(target: &mut Value, patch: &Value) {
    if let (Some(target), Some(patch)) = (target.as_object_mut(), patch.as_object()) {
        for (key, value) in patch {
            merge(target.entry(key).or_insert(Value::Null), value);
        }
    } else {
        *target = patch.clone();
    }
}
fn actor_patch(entity: &Actor, patch: &Value) -> Result<Actor, String> {
    const FIELDS: &[&str] = &[
        "name",
        "kind",
        "parent",
        "position",
        "rotation",
        "scale",
        "material",
        "canvas",
        "rect",
        "image",
        "text",
        "progress",
        "lighting",
        "light",
        "blob_shadow",
        "audio",
        "skeletal_mesh",
        "editable_mesh",
    ];
    for key in patch
        .as_object()
        .ok_or("Actor patch must be an object")?
        .keys()
    {
        if !FIELDS.contains(&key.as_str()) {
            return Err(format!("Unknown Actor field: {key}"));
        }
    }
    let mut value = json!(entity.data);
    merge(&mut value, patch);
    let mut actor = entity.clone();
    actor.data = decode(value)?;
    actor.name = actor.data.name.clone();
    actor.active = actor.data.active;
    crate::actor_components::sync(&mut actor);
    Ok(actor)
}
fn install(e: &mut Editor, mut scene: Scene) -> Result<(), String> {
    scene.display_size = e.scene.display_size;
    scene.validate()?;
    // The asynchronous asset index may still be warming up immediately after
    // project open. Resolve against disk before publishing a scene transaction.
    e.assets.index = assets::scan(&e.root, &mut Default::default());
    crate::mesh::resolve(&mut scene, &e.assets.index)?;
    crate::terrain::resolve(&mut scene, &e.assets.index)?;
    crate::skeletal::resolve(&mut scene, &e.assets.index)?;
    e.scene = scene;
    e.reset_scene_tools();
    e.changed();
    Ok(())
}
fn refresh(e: &mut Editor) {
    e.assets.refresh();
    e.assets.index = assets::scan(&e.root, &mut Default::default());
    let _ = crate::mesh::resolve(&mut e.scene, &e.assets.index);
    let _ = crate::terrain::resolve(&mut e.scene, &e.assets.index);
    let _ = crate::skeletal::resolve(&mut e.scene, &e.assets.index);
    e.refresh_scripts();
    e.view_dirty = true;
}
fn snapshot(e: &Editor) -> Value {
    let profile = json!({"saved":e.play_profile,"active_target":e.active_play_target});
    let operation = e.critical_busy().then(|| {
        if e.dependencies.busy() {
            e.dependencies.message.clone()
        } else {
            e.job_stage.clone()
        }
    });
    let serial = json!({"setup_open":e.serial_ui.open,"tools_ready":e.serial_ui.tools_ready,"tools_directory":crate::serial_support::managed_directory(),"port":e.preferences.serial.port,"status":e.serial_ui.status,"error":e.serial_ui.error,"command_pending":e.serial_ui.command_pending});
    json!({"project":e.project_name(),"root":e.root,"scene_path":assets::path_string(&e.root,&e.scene_path()),"revision":revision(&e.scene),"dirty":e.dirty,"actors":e.scene.actors.len(),"selected":e.selected,"playing":e.playing,"paused":e.paused,"play_profile":profile,"serial":serial,"build_or_play_active":e.job.is_some(),"editing_locked":e.critical_busy(),"installation_active":e.dependencies.busy(),"operation":operation,"import_active":e.assets.busy,"import_error":e.assets.error,"bake_active":e.bake_job.is_some(),"last_error":e.last_error,"game_error":e.game_error,"game_frame":e.game_frame.as_ref().map(|f|json!({"sequence":f.sequence,"width":f.width,"height":f.height,"buttons":f.buttons,"vsyncs":f.vsyncs,"cycles":f.cycles})),"view":e.view,"scene_2d":e.scene_2d(),"view_mode":e.scene_view_mode.key(),"grid":e.grid,"wire":e.wire})
}

/// Resolved Object/Actor/Component model for the editor's current catalog. `None` when
/// the project has never compiled: the actor tools then run their structural half only.
fn actor_model(e: &Editor) -> Option<crate::object_model::Model> {
    crate::blueprint::registry_from_catalog(&e.root, &e.catalog)
        .model()
        .ok()
}
/// Root component class for a placeable actor's domain, so a new actor satisfies
/// `Model::validate_component_set` the moment it is created.
pub fn root_component_class(
    model: &crate::object_model::Model,
    class: &crate::object_model::ClassModel,
) -> Option<crate::actor_document::ClassReference> {
    let id = match class.domain {
        crate::reflection_schema::Domain::World3D => crate::object_model::SCENE_COMPONENT3D_ID,
        crate::reflection_schema::Domain::World2D => crate::object_model::SCENE_COMPONENT2D_ID,
        crate::reflection_schema::Domain::UI => crate::object_model::RECT_TRANSFORM_COMPONENT_ID,
        crate::reflection_schema::Domain::None => return None,
    };
    let root = model.class(id)?;
    Some(crate::actor_document::ClassReference::new(
        &root.cpp_name,
        &root.id,
    ))
}
/// Whole-actor component validation, shared by the Inspector's Add Component
/// popup and by `scene_set_actor`. Neither re-implements root uniqueness,
/// requires/excludes, cardinality or capabilities: they are the model's rules.
pub fn validate_actor_components(
    model: &crate::object_model::Model,
    actor: &crate::actor_document::ActorInstance,
) -> Result<(), String> {
    let owner = actor
        .class
        .resolve(model)
        .ok_or_else(|| format!("`{}` is not a reflected class", actor.class.name))?;
    let specs: Vec<_> = actor
        .components
        .iter()
        .map(|c| crate::object_model::ComponentSpec {
            id: c.id,
            class: c
                .class
                .class_id
                .clone()
                .unwrap_or_else(|| c.class.name.clone()),
            root: c.root,
        })
        .collect();
    model
        .validate_component_set(&owner.id, &specs)
        .map_err(|diagnostics| {
            diagnostics
                .iter()
                .map(|d| d.message.clone())
                .collect::<Vec<_>>()
                .join("\n")
        })
}
/// Why one component may not be removed from its actor, or `None` when it may.
/// The Inspector shows these as tooltips on a disabled Remove button.
pub fn component_removal_refusal(
    component: &crate::actor_document::ComponentInstance,
) -> Option<&'static str> {
    if component.root {
        Some("the root component cannot be removed")
    } else if component.inherited {
        Some("inherited from the class; disable instead")
    } else {
        None
    }
}
/// The spatial attachment a logical reparent implies.
///
/// `attach` exists only between actors of one spatial domain: a UI actor
/// parented under a 3D actor keeps its place in the hierarchy and loses its
/// transform relationship, which is exactly what the domains mean.
pub fn implied_attachment(
    model: Option<&crate::object_model::Model>,
    scene: &Scene,
    child: uuid::Uuid,
    parent: Option<uuid::Uuid>,
) -> Option<crate::actor_document::Attachment> {
    let parent = parent?;
    let model = model?;
    let domain = |id: uuid::Uuid| {
        scene
            .actors
            .iter()
            .find(|a| a.id == id)
            .and_then(|a| a.class.resolve(model))
            .map(|c| c.domain)
    };
    let child = domain(child)?;
    if child == crate::reflection_schema::Domain::None || Some(child) != domain(parent) {
        return None;
    }
    Some(crate::actor_document::Attachment {
        actor: parent,
        component: None,
    })
}
fn actor_id(scene: &Scene, args: &Value) -> Result<uuid::Uuid, String> {
    let id: uuid::Uuid = decode(args["id"].clone())?;
    if !scene.actors.iter().any(|actor| actor.id == id) {
        return Err(
            "No authored actor with that id. Read scene_actors; derived actors must be migrated before they can be edited.".into(),
        );
    }
    Ok(id)
}
/// Publish a scene transaction and record it for History/undo, exactly like the entity
/// tools do: `install` runs `Editor::changed()`, which is what marks the build stale.
fn commit(e: &mut Editor, state: &mut State, before: Scene, scene: Scene) -> Result<(), String> {
    install(e, scene)?;
    state.undo.push((before, e.scene.clone()));
    if state.undo.len() > 32 {
        state.undo.remove(0);
    }
    state.redo.clear();
    Ok(())
}

pub fn execute(e: &mut Editor, state: &mut State, name: &str, a: Value) -> Result<Value, String> {
    if name != "guide" {
        validate_arguments(name, &a)?;
    }
    if e.critical_busy() {
        let read = matches!(
            name,
            "guide" | "editor_state" | "scene_read" | "scene_schema" | "scene_actors" | "logs_read"
        ) || (name == "project_settings" && a.get("patch").is_none())
            || (name == "asset_document" && a.get("document").is_none())
            || (name == "project_files" && matches!(a["action"].as_str(), Some("list" | "read")))
            || (name == "editor_control" && a["action"] == "stop");
        if !read {
            return Err("Editing is locked during component installation, compilation and serial preparation. Read editor_state/logs_read or cancel the operation.".into());
        }
    }
    match name {
        "guide" => Ok(json!({"guide":crate::mcp::GUIDE})),
        "editor_state" => Ok(snapshot(e)),
        "scene_read" => Ok(json!({"revision":revision(&e.scene),"scene":e.scene})),
        "scene_schema" => Ok(schema()),
        "scene_apply" => {
            editable(e)?;
            expect_revision(&e.scene, &a)?;
            let operations = a["operations"]
                .as_array()
                .filter(|v| !v.is_empty() && v.len() <= 128)
                .ok_or("Provide 1..128 operations")?;
            let before = e.scene.clone();
            let mut scene = before.clone();
            let mut results = vec![];
            for op in operations {
                let result = match string(op, "op")?.as_str() {
                    "create" => {
                        let entity = actor_patch(&Actor::cube("Actor".into()), &op["actor"])?;
                        let i = scene.actors.len();
                        scene.actors.push(entity);
                        json!({"index":i})
                    }
                    "update" => {
                        let i = index(&scene, op)?;
                        scene.actors[i] = actor_patch(&scene.actors[i], &op["patch"])?;
                        json!({"index":i})
                    }
                    "delete" => {
                        let i = index(&scene, op)?;
                        scene.delete_branch(i);
                        json!({"deleted":i})
                    }
                    "duplicate" => {
                        let i = index(&scene, op)?;
                        json!({"index":scene.duplicate_branch(i)?})
                    }
                    "reparent" => {
                        let i = index(&scene, op)?;
                        let parent: Option<usize> = decode(op["parent"].clone())?;
                        scene.reparent(i, parent, op["keep_world"].as_bool().unwrap_or(true))?;
                        json!({"index":i})
                    }
                    "environment" => {
                        let mut value = json!(scene.environment);
                        merge(&mut value, &op["patch"]);
                        scene.environment = decode(value)?;
                        json!({"environment":true})
                    }
                    "replace" => {
                        scene = decode(op["scene"].clone())?;
                        json!({"replaced":true})
                    }
                    other => return Err(format!("Unknown scene operation: {other}")),
                };
                // Validate each intermediate hierarchy before operations that traverse it.
                scene.validate()?;
                results.push(result);
            }
            install(e, scene)?;
            state.undo.push((before, e.scene.clone()));
            if state.undo.len() > 32 {
                state.undo.remove(0);
            }
            state.redo.clear();
            e.log(format!(
                "MCP: applied {} scene operations",
                operations.len()
            ));
            Ok(json!({"revision":revision(&e.scene),"results":results,"saved":false}))
        }
        "scene_actors" => {
            let model = actor_model(e);

            let describe = |actor: &crate::actor_document::ActorInstance, authored: bool| {
                let class = model.as_ref().and_then(|m| actor.class.resolve(m));
                json!({
                    "id": actor.id,
                    "name": actor.name,
                    "class": actor.class.name,
                    "class_id": actor.class.class_id,
                    "domain": class.map(|c| format!("{:?}", c.domain)),
                    "family": class.map(|c| format!("{:?}", c.family)),
                    "active": actor.active,
                    "authored": authored,
                    "logical_parent": actor.logical_parent,
                    "properties": actor.properties,
                    "overrides": actor.overrides,
                    "attach": actor.attach.as_ref().map(|at| json!({"actor": at.actor, "component": at.component})),
                    "components": actor.components.iter().map(|c| json!({
                        "id": c.id,
                        "name": c.name,
                        "class": c.class.name,
                        "class_id": c.class.class_id,
                        "root": c.root,
                        "inherited": c.inherited,
                        "attach_parent": c.attach_parent,
                        "properties": c.properties,
                        // Which keys the author set explicitly. A value may equal
                        // the class default and still be an override.
                        "overrides": c.overrides,
                        "removable": component_removal_refusal(c).is_none(),
                    })).collect::<Vec<_>>(),
                })
            };
            let authored = e
                .scene
                .actors
                .iter()
                .map(|actor| describe(actor, true))
                .collect::<Vec<_>>();
            Ok(json!({
                "revision": revision(&e.scene),
                "model_available": model.is_some(),
                "actors": authored,
                "placeable": model.as_ref().map(|m| m.placeable()
                    .map(|c| json!({"class": c.cpp_name, "class_id": c.id, "domain": format!("{:?}", c.domain)}))
                    .collect::<Vec<_>>()),
            }))
        }
        "scene_add_actor" => {
            editable(e)?;
            expect_revision(&e.scene, &a)?;
            let model = actor_model(e)
                .ok_or("The reflected class model is unavailable; compile the project first.")?;
            let requested = string(&a, "class")?;
            let class = model
                .placeable()
                .find(|c| c.cpp_name == requested || c.id == requested)
                .ok_or_else(|| {
                    format!(
                        "{requested} is not a placeable actor class. Placeable classes: {}",
                        model
                            .placeable()
                            .map(|c| c.cpp_name.clone())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                })?;
            let before = e.scene.clone();
            let mut scene = before.clone();
            let mut actor = crate::actor_document::ActorInstance::new(
                uuid::Uuid::new_v4(),
                crate::actor_document::ClassReference::new(&class.cpp_name, &class.id),
                a["name"].as_str().unwrap_or(&class.cpp_name),
            );
            if let Some(parent) = a.get("parent").filter(|v| !v.is_null()) {
                let parent: uuid::Uuid = decode(parent.clone())?;
                if !scene.actors.iter().any(|other| other.id == parent) {
                    return Err("parent must be the id of an actor in this scene".into());
                }
                actor.logical_parent = Some(parent);
            }
            if let Some(root) = root_component_class(&model, class) {
                let mut component = crate::actor_document::ComponentInstance::new(
                    uuid::Uuid::new_v4(),
                    root,
                    "Root",
                );
                component.root = true;
                actor.components.push(component);
            }
            let id = actor.id;
            scene.actors.push(actor);
            scene.validate_with_model(Some(&model))?;
            commit(e, state, before, scene)?;
            e.log(format!("MCP: added actor {} ({})", id, class.cpp_name));
            Ok(json!({"revision":revision(&e.scene),"id":id,"class":class.cpp_name,"saved":false}))
        }
        "scene_remove_actor" => {
            editable(e)?;
            expect_revision(&e.scene, &a)?;
            let model = actor_model(e);
            let before = e.scene.clone();
            let mut scene = before.clone();
            let id = actor_id(&scene, &a)?;
            scene.actors.retain(|actor| actor.id != id);
            // Never leave a dangling reference behind: clear, rather than orphan.
            for actor in &mut scene.actors {
                if actor.logical_parent == Some(id) {
                    actor.logical_parent = None;
                }
                if actor.attach.as_ref().is_some_and(|at| at.actor == id) {
                    actor.attach = None;
                }
            }
            scene.validate_with_model(model.as_ref())?;
            commit(e, state, before, scene)?;
            Ok(json!({"revision":revision(&e.scene),"removed":id,"saved":false}))
        }
        "scene_set_actor" => {
            editable(e)?;
            expect_revision(&e.scene, &a)?;
            let model = actor_model(e);
            let before = e.scene.clone();
            let mut scene = before.clone();
            let id = actor_id(&scene, &a)?;
            let index = scene
                .actors
                .iter()
                .position(|actor| actor.id == id)
                .unwrap();
            {
                let actor = &mut scene.actors[index];
                if let Some(active) = a["active"].as_bool() {
                    actor.active = active;
                }
                if let Some(name) = a["name"].as_str() {
                    actor.name = name.to_owned();
                }
                if let Some(properties) = a.get("properties").filter(|v| !v.is_null()) {
                    let properties = properties
                        .as_object()
                        .ok_or("properties must be an object")?;
                    for (key, value) in properties {
                        // Null resets an override back to the class default.
                        if value.is_null() {
                            actor.properties.remove(key);
                            actor.overrides.remove(key);
                        } else {
                            actor.properties.insert(key.clone(), value.clone());
                            actor.overrides.insert(key.clone());
                        }
                    }
                }
            }
            // Reparent, mirroring the Hierarchy's drag-and-drop: `null` moves the
            // actor to the map root, and the spatial `attach` follows only when
            // both actors resolve to the same spatial domain.
            if let Some(parent) = a.get("parent") {
                let parent = if parent.is_null() {
                    None
                } else {
                    let parent: uuid::Uuid = decode(parent.clone())?;
                    if parent == id {
                        return Err("An actor cannot be its own parent".into());
                    }
                    if !scene.actors.iter().any(|other| other.id == parent) {
                        return Err("parent must be the id of an actor in this scene".into());
                    }
                    Some(parent)
                };
                let attach = implied_attachment(model.as_ref(), &scene, id, parent);
                scene.actors[index].logical_parent = parent;
                scene.actors[index].attach = attach;
            }
            // Component add/remove, mirroring the Inspector's Components section.
            if let Some(components) = a.get("components").filter(|v| !v.is_null()) {
                let components = components
                    .as_object()
                    .ok_or("components must be an object with add and/or remove")?;
                for key in components.keys() {
                    if key != "add" && key != "remove" {
                        return Err(format!(
                            "components: unknown operation {key}; use add and remove"
                        ));
                    }
                }
                if let Some(remove) = components.get("remove").filter(|v| !v.is_null()) {
                    for value in remove
                        .as_array()
                        .ok_or("components.remove must be an array")?
                    {
                        let component: uuid::Uuid = decode(value.clone())?;
                        let position = scene.actors[index]
                            .components
                            .iter()
                            .position(|c| c.id == component)
                            .ok_or("No component with that id on this actor")?;
                        if let Some(refusal) =
                            component_removal_refusal(&scene.actors[index].components[position])
                        {
                            return Err(format!(
                                "{}: {refusal}",
                                scene.actors[index].components[position].name
                            ));
                        }
                        scene.actors[index].components.remove(position);
                        for other in &mut scene.actors {
                            if other
                                .attach
                                .as_ref()
                                .is_some_and(|at| at.component == Some(component))
                            {
                                other.attach = None;
                            }
                            for sibling in &mut other.components {
                                if sibling.attach_parent == Some(component) {
                                    sibling.attach_parent = None;
                                }
                            }
                        }
                    }
                }
                if let Some(add) = components.get("add").filter(|v| !v.is_null()) {
                    let model = model.as_ref().ok_or(
                        "The reflected class model is unavailable; compile the project first.",
                    )?;
                    let owner = scene.actors[index]
                        .class
                        .resolve(model)
                        .ok_or(
                            "This actor's class is not reflected; its components cannot be edited",
                        )?
                        .id
                        .clone();
                    for value in add.as_array().ok_or("components.add must be an array")? {
                        let requested = value["class"]
                            .as_str()
                            .ok_or("components.add entries need a class")?;
                        let class = model
                            .class(requested)
                            .ok_or_else(|| format!("{requested} is not a reflected class"))?;
                        model
                            .validate_component(&owner, &class.id)
                            .map_err(|d| d.message)?;
                        let base = value["name"]
                            .as_str()
                            .map(str::to_owned)
                            .unwrap_or_else(|| {
                                crate::actor_document::short_class_name(&class.cpp_name).to_owned()
                            });
                        let name = crate::actor_document::unique_component_name(
                            &scene.actors[index],
                            &base,
                        );
                        scene.actors[index].components.push(
                            crate::actor_document::ComponentInstance::new(
                                uuid::Uuid::new_v4(),
                                crate::actor_document::ClassReference::new(
                                    &class.cpp_name,
                                    &class.id,
                                ),
                                &name,
                            ),
                        );
                    }
                }
                if let Some(model) = model.as_ref() {
                    validate_actor_components(model, &scene.actors[index])?;
                }
            }
            scene.validate_with_model(model.as_ref())?;
            commit(e, state, before, scene)?;
            let actor = &e.scene.actors[index];
            Ok(json!({
                "revision": revision(&e.scene),
                "id": id,
                "logical_parent": actor.logical_parent,
                "attach": actor.attach.as_ref().map(|at| at.actor),
                "components": actor.components.iter()
                    .map(|c| json!({"id": c.id, "name": c.name, "class": c.class.name, "root": c.root}))
                    .collect::<Vec<_>>(),
                "saved": false,
            }))
        }
        "scene_history" => {
            editable(e)?;
            expect_revision(&e.scene, &a)?;
            let undo = match string(&a, "action")?.as_str() {
                "undo" => true,
                "redo" => false,
                _ => return Err("Use undo or redo".into()),
            };
            let history = if undo {
                &mut state.undo
            } else {
                &mut state.redo
            };
            let (before, after) = history
                .last()
                .ok_or("No MCP scene edit to restore")?
                .clone();
            if revision(&e.scene) != revision(if undo { &after } else { &before }) {
                return Err("Intervening scene edits prevent this undo/redo. Read and edit the current scene instead.".into());
            }
            install(e, if undo { before.clone() } else { after.clone() })?;
            history.pop();
            if undo {
                state.redo.push((before, after));
            } else {
                state.undo.push((before, after));
            }
            Ok(json!({"revision":revision(&e.scene),"saved":false}))
        }
        "scene_save" => {
            editable(e)?;
            expect_revision(&e.scene, &a)?;
            if !e.save() {
                return Err("Scene save failed. Read logs_read for details.".into());
            }
            Ok(snapshot(e))
        }
        "scene_open" => {
            editable(e)?;
            expect_revision(&e.scene, &a)?;
            if e.dirty && a["discard_unsaved"] != true {
                return Err("Save the scene first or explicitly set discard_unsaved=true.".into());
            }
            let path = inside(&e.root, &string(&a, "path")?)?;
            e.open_scene(path)?;
            state.undo.clear();
            state.redo.clear();
            Ok(snapshot(e))
        }
        "actor_select" => {
            expect_revision(&e.scene, &a)?;
            let id = if a["id"].is_null() {
                None
            } else {
                let id = decode::<uuid::Uuid>(a["id"].clone())?;
                if e.scene.actor_index(id).is_none() {
                    return Err("Actor does not exist".into());
                }
                Some(id)
            };
            e.select_actor(id);
            e.reveal_selected = true;
            e.view_dirty = true;
            if a["frame"] == true {
                e.action("frame-selected");
            }
            Ok(snapshot(e))
        }
        "editor_view" => {
            if let Some(patch) = a.get("view") {
                let mut v = json!(e.view);
                for key in patch.as_object().ok_or("view must be an object")?.keys() {
                    if v.get(key).is_none() {
                        return Err(format!("Unknown view field: {key}"));
                    }
                }
                merge(&mut v, patch);
                let view: crate::viewport::View = decode(v)?;
                if !view
                    .center
                    .iter()
                    .chain([
                        &view.yaw,
                        &view.pitch,
                        &view.zoom,
                        &view.distance,
                        &view.fly_speed,
                        &view.phase,
                    ])
                    .all(|v| v.is_finite() && v.abs() <= 1e6)
                    || !(0.05..=8.).contains(&view.zoom)
                    || !(1.05..=10000.).contains(&view.distance)
                    || !(0.01..=1000.).contains(&view.fly_speed)
                {
                    return Err("Invalid camera values: zoom 0.05..8, distance 1.05..10000, fly_speed 0.01..1000, finite coordinates.".into());
                }
                e.view = view;
            }
            for (key, target) in [("grid", &mut e.grid), ("wire", &mut e.wire)] {
                if let Some(v) = a.get(key) {
                    *target = v.as_bool().ok_or("View flags must be booleans")?;
                }
            }
            // `scene_2d` is the legacy switch: true is the UI mode, false is 3D.
            // `view_mode` is the full three-way control and wins when both are given.
            if let Some(v) = a.get("scene_2d") {
                let ui = v.as_bool().ok_or("View flags must be booleans")?;
                e.set_scene_2d(ui);
            }
            if let Some(v) = a.get("view_mode") {
                let requested = v.as_str().ok_or("view_mode must be a string")?;
                let mode = crate::scene_view_mode::SceneViewMode::parse(requested)
                    .ok_or("view_mode must be one of: 3d, 2d, ui")?;
                e.set_scene_view_mode(mode);
            }
            e.view_dirty = true;
            Ok(snapshot(e))
        }
        "game_input" => {
            if e.active_play_target == crate::play::Target::Serial {
                return Err("Use the physical PSX controller during Serial Play.".into());
            }
            if !e.playing {
                return Err("Start Play before sending controller input".into());
            }
            let mask = a["buttons"]
                .as_u64()
                .filter(|n| *n <= 65535)
                .ok_or("buttons must be a 16-bit mask")? as u16;
            let duration = a["duration_ms"].as_u64().unwrap_or(150);
            if !(1..=5000).contains(&duration) {
                return Err("duration_ms must be 1..5000".into());
            }
            state.buttons = mask;
            state.buttons_until =
                Some(std::time::Instant::now() + std::time::Duration::from_millis(duration));
            e.set_buttons(mask);
            Ok(json!({"buttons":mask,"duration_ms":duration}))
        }
        "editor_control" => {
            match string(&a, "action")?.as_str() {
                "build" | "play" => {
                    editable(e)?;
                    e.last_error = None;
                    e.build(a["action"] == "play");
                }
                "stop" => {
                    if let Some(job) = &e.job {
                        job.control(crate::pipeline::Control::Stop);
                    }
                }
                "step" => {
                    if !e.playing || !e.paused {
                        return Err("Pause Play before stepping".into());
                    }
                    e.job
                        .as_ref()
                        .and_then(|j| j.bridge.as_ref())
                        .ok_or("Emulator bridge unavailable")?
                        .step
                        .store(true, std::sync::atomic::Ordering::Relaxed);
                }
                "pause" | "resume" | "reset_psx" => {
                    if !e.playing {
                        return Err("Play is not running".into());
                    }
                    if e.serial_ui.command_pending {
                        return Err("Waiting for the previous PSX command".into());
                    }
                    if a["action"] == "reset_psx"
                        && e.active_play_target != crate::play::Target::Serial
                    {
                        return Err("Reset PSX requires an active physical PSX session".into());
                    }
                    e.job
                        .as_ref()
                        .ok_or("No Play job")?
                        .control(if a["action"] == "reset_psx" {
                            crate::pipeline::Control::Reset
                        } else if a["action"] == "pause" {
                            crate::pipeline::Control::Pause
                        } else {
                            crate::pipeline::Control::Resume
                        });
                    if e.active_play_target == crate::play::Target::Serial {
                        e.serial_ui.command_pending = true;
                    }
                }
                "bake" => {
                    editable(e)?;
                    crate::lighting_editor::start_bake(e);
                }
                "export" => {
                    editable(e)?;
                    return Ok(
                        json!({"path":crate::export::export_project(&e.root,&crate::scene_dependencies::Input::editor(e.scene_path(),e.scene.clone()))?}),
                    );
                }
                "reset_layout" => e.reset_layout = true,
                "project_settings" => crate::settings_ui::open_project(e),
                "editor_preferences" => crate::settings_ui::open_preferences(e),
                "serial_connection" => crate::serial_ui::open(e),
                "serial_prepare" => crate::serial_ui::start(e, crate::serial_ui::Action::Prepare),
                "imports" => e.assets.window = true,
                "frame_selected" => e.action("frame-selected"),
                other => return Err(format!("Unknown editor action: {other}")),
            }
            Ok(json!({"accepted":true,"state":snapshot(e)}))
        }
        "logs_read" => {
            let limit = a["limit"].as_u64().unwrap_or(100).min(400) as usize;
            Ok(
                json!({"lines":e.logs.iter().skip(e.logs.len().saturating_sub(limit)).collect::<Vec<_>>()}),
            )
        }
        "project_settings" => {
            let mut value = json!(crate::workspace::read_manifest(&e.root)?);
            if let Some(patch) = a.get("patch") {
                editable(e)?;
                let hash = assets::hash(value.to_string().as_bytes());
                if a["revision"] != hash {
                    return Err("Project settings changed. Read project_settings first.".into());
                }
                merge(&mut value, patch);
                e.apply_project_settings(decode(value.clone())?)?;
            }
            Ok(json!({"settings":value,"revision":assets::hash(value.to_string().as_bytes())}))
        }
        "project_files" => files(e, &a),
        "asset_list" => {
            refresh(e);
            Ok(
                json!({"assets":e.assets.index.usable().map(|r|json!({"path":assets::path_string(&e.root,&r.path),"revision":r.revision,"metadata":r.meta})).collect::<Vec<_>>(),"problems":e.assets.index.problems,"pending":e.assets.pending.iter().map(|p|json!({"source":p.source,"status":p.status})).collect::<Vec<_>>()}),
            )
        }
        "asset_import" => {
            editable(e)?;
            if e.assets.form.is_some() {
                return Err("Finish or close the current import dialog first.".into());
            }
            let source = string(&a, "source")?;
            let destination = string(&a, "destination")?;
            inside(&e.root, &source)?;
            let path = inside(&e.root, &destination)?;
            let existing = if path.exists() {
                let p = assets::Package::load(&path)?;
                refresh(e);
                let r = e.assets.index.resolve(p.meta.id)?.clone();
                if a["revision"] != r.revision {
                    return Err(
                        "Read asset_list and supply the current revision to reimport.".into(),
                    );
                }
                Some(r)
            } else {
                None
            };
            let model = source.to_lowercase().ends_with(".fbx");
            let settings = if let Some(value) = a.get("audio_settings") {
                decode(value.clone())?
            } else {
                Default::default()
            };
            let model_storage = if let Some(value) = a.get("animation_storage") {
                decode(value.clone())?
            } else {
                existing
                    .as_ref()
                    .and_then(|r| match &r.meta.settings {
                        crate::import_settings::Settings::Fbx(s) => Some(s.animation_storage),
                        _ => None,
                    })
                    .unwrap_or_default()
            };
            let detected = assets::audio_source_kind(&e.root, &source);
            let soundfont = crate::soundfont_asset::has_source_header(&e.root.join(&source));
            e.assets.form = Some(crate::asset_manager::ImportForm {
                sequence: if a.get("sequence_settings").is_some()
                    || detected == Some(assets::Kind::MusicSequence)
                    || existing
                        .as_ref()
                        .is_some_and(|r| r.meta.kind == assets::Kind::MusicSequence)
                    || (detected.is_none()
                        && std::path::Path::new(&source).extension().is_some_and(|e| {
                            matches!(
                                e.to_ascii_lowercase().to_str(),
                                Some("mid" | "midi" | "seq" | "sep")
                            )
                        })) {
                    Some(if let Some(value) = a.get("sequence_settings") {
                        decode(value.clone())?
                    } else {
                        existing
                            .as_ref()
                            .and_then(|r| r.meta.settings.sequence().ok())
                            .cloned()
                            .unwrap_or_default()
                    })
                } else {
                    None
                },
                bank: if let Some(value) = a.get("bank_settings") {
                    Some(decode(value.clone())?)
                } else {
                    existing
                        .as_ref()
                        .and_then(|r| r.meta.settings.sound_bank().ok())
                        .cloned()
                        .or_else(|| {
                            if soundfont {
                                Some(crate::sound_bank::Settings {
                                    schema_version: 2,
                                    library: Some(crate::soundfont_asset::Definition::detected(
                                        crate::sf2::SourceFormat::Sf2Pcm16,
                                    )),
                                    ..Default::default()
                                })
                            } else {
                                (detected == Some(assets::Kind::SoundBank)
                                    || (detected.is_none()
                                        && crate::bank_compat::source_candidate(
                                            std::path::Path::new(&source),
                                        )))
                                .then(|| {
                                    crate::sound_bank::Settings {
                                        imported: Some(Default::default()),
                                        ..Default::default()
                                    }
                                })
                            }
                        })
                },
                bank_companion: a
                    .get("vb_source")
                    .and_then(|v| v.as_str())
                    .map(str::to_owned)
                    .or_else(|| {
                        existing
                            .as_ref()
                            .and_then(crate::bank_compat::parts)
                            .and_then(|p| p.get(1))
                            .map(|p| p.path.clone())
                    })
                    .unwrap_or_default(),
                sequence_catalog: None,
                texture: source.to_ascii_lowercase().ends_with(".png"),
                model,
                model_storage,
                source,
                destination,
                settings,
                existing,
                snapshot: false,
                queue_key: None,
            });
            e.assets.start_import();
            if !e.assets.busy {
                return Err(e
                    .assets
                    .error
                    .clone()
                    .unwrap_or("Import did not start".into()));
            }
            Ok(json!({"accepted":true,"poll":"editor_state; then asset_list"}))
        }
        "mesh_create" => {
            editable(e)?;
            let path = string(&a, "path")?;
            if !path.ends_with(".epokasset") {
                return Err("Mesh path must end in .epokasset".into());
            }
            let mut doc = crate::mesh::Document::default();
            if let Some(value) = a.get("document") {
                doc = decode(value.clone())?;
            } else {
                let shape = a["shape"].as_str().unwrap_or("Box");
                if !["Box", "Plane", "Ramp", "Stairs"].contains(&shape) {
                    return Err("Use Box, Plane, Ramp or Stairs".into());
                }
                let size = decode(a.get("size").cloned().unwrap_or(json!([1, 1, 1])))?;
                let steps = a["steps"].as_u64().unwrap_or(4);
                if !(1..=64).contains(&steps) {
                    return Err("steps must be 1..64".into());
                }
                doc.primitive(
                    shape,
                    [0.; 3],
                    size,
                    steps as usize,
                    doc.groups[0].id,
                    doc.materials[0].id,
                );
            }
            let id = crate::mesh::create(&e.root, &path, &doc)?;
            refresh(e);
            Ok(json!({"id":id,"path":path,"component":crate::mesh::Component::new(id)}))
        }
        "asset_manage" => {
            editable(e)?;
            let path = inside(&e.root, &string(&a, "path")?)?;
            refresh(e);
            let package = assets::Package::load(&path)?;
            let record = e.assets.index.resolve(package.meta.id)?.clone();
            if a["revision"] != record.revision {
                return Err("Asset changed. Read asset_list first.".into());
            }
            let result = match string(&a, "action")?.as_str() {
                "move" => {
                    let destination = string(&a, "destination")?;
                    inside(&e.root, &destination)?;
                    assets::move_asset(&e.root, &record, &destination)?;
                    json!({"id":record.meta.id,"path":destination})
                }
                "duplicate" => {
                    let destination = string(&a, "destination")?;
                    let to = inside(&e.root, &destination)?;
                    if to.extension().is_none_or(|e| e != "epokasset") {
                        return Err("Keep the .epokasset extension".into());
                    }
                    let id = assets::duplicate(&record, &to)?;
                    json!({"id":id,"path":destination})
                }
                "trash" => {
                    check_local_directory(&e.root, &e.root.join("UserSettings/AssetTrash"))?;
                    if serde_json::to_string(&e.scene)
                        .map_err(|e| e.to_string())?
                        .contains(&record.meta.id.to_string())
                    {
                        return Err(
                            "The open scene references this asset. Remove its references first."
                                .into(),
                        );
                    }
                    let path = assets::trash(&e.root, &record)?;
                    json!({"trash":path})
                }
                _ => return Err("Use move, duplicate or trash".into()),
            };
            refresh(e);
            Ok(result)
        }
        "asset_document" => {
            let path = inside(&e.root, &string(&a, "path")?)?;
            let mut package = assets::Package::load(&path)?;
            if matches!(
                package.meta.kind,
                assets::Kind::AudioClip
                    | assets::Kind::MusicSequence
                    | assets::Kind::SoundBank
                    | assets::Kind::ModelSource
            ) {
                return Err(
                    "Use project_files read for binary source packages; asset_import for changes."
                        .into(),
                );
            }
            let bytes = assets::read_bounded(&path)?;
            let hash = assets::hash(&bytes);
            if let Some(doc) = a.get("document") {
                editable(e)?;
                if a["revision"] != hash {
                    return Err("Asset changed. Read asset_document again.".into());
                }
                let source = serde_json::to_vec(doc).map_err(|e| e.to_string())?;
                if package.meta.kind == assets::Kind::EditableMesh {
                    decode::<crate::mesh::Document>(doc.clone())?.validate()?;
                } else if crate::skeletal::Data::parse(&source)?.kind() != package.meta.kind {
                    return Err("Asset document kind cannot change".into());
                } else {
                    decode::<crate::skeletal::Data>(doc.clone())?;
                }
                package.source = source;
                package.meta.source_hash = assets::hash(&package.source);
                backup(e, &path, &bytes)?;
                assets::atomic_write(&path, &package.bytes()?, Some(&hash))?;
                refresh(e);
            }
            Ok(
                json!({"metadata":package.meta,"document":serde_json::from_slice::<Value>(&package.source).map_err(|e|e.to_string())?,"revision":assets::hash(&assets::read_bounded(&path)?)}),
            )
        }
        _ => Err(format!("Unknown tool: {name}")),
    }
}
fn backup(e: &Editor, path: &Path, bytes: &[u8]) -> Result<String, String> {
    let relative = format!(
        ".epok/mcp-backups/{}/{}",
        uuid::Uuid::new_v4(),
        path.file_name()
            .ok_or("Missing filename")?
            .to_string_lossy()
    );
    let backup = e.root.join(&relative);
    check_local_directory(&e.root, &backup)?;
    assets::atomic_write(&backup, bytes, None)?;
    Ok(relative)
}
fn check_local_directory(root: &Path, path: &Path) -> Result<(), String> {
    let ancestor = path
        .ancestors()
        .find(|p| p.exists())
        .ok_or("Missing project root")?;
    if !fs::canonicalize(ancestor)
        .map_err(|e| e.to_string())?
        .starts_with(fs::canonicalize(root).map_err(|e| e.to_string())?)
    {
        return Err("Backup directory must stay inside the project".into());
    }
    Ok(())
}
fn files(e: &mut Editor, a: &Value) -> Result<Value, String> {
    let action = a["action"].as_str().unwrap_or("list");
    let relative = a["path"].as_str().unwrap_or("assets");
    let path = inside(&e.root, relative)?;
    match action {
        "list" => {
            let mut files = vec![];
            // One directory per request: bounded output, no recursive symlink traversal.
            for entry in fs::read_dir(&path).map_err(|e| e.to_string())?.take(1001) {
                let entry = entry.map_err(|e| e.to_string())?;
                let ty = entry.file_type().map_err(|e| e.to_string())?;
                files.push(json!({"path":assets::path_string(&e.root,&entry.path()),"directory":ty.is_dir(),"symlink":ty.is_symlink(),"bytes":entry.metadata().map(|m|m.len()).unwrap_or(0)}));
            }
            let truncated = files.len() > 1000;
            files.truncate(1000);
            Ok(json!({"entries":files,"truncated":truncated}))
        }
        "read" => {
            let bytes = read_file(&path)?;
            let encoding = a["encoding"].as_str().unwrap_or("text");
            let content = match encoding {
                "text" => String::from_utf8(bytes.clone())
                    .map_err(|_| "Binary file: request encoding=base64")?,
                "base64" => base64::engine::general_purpose::STANDARD.encode(&bytes),
                _ => return Err("Use text or base64 encoding".into()),
            };
            Ok(
                json!({"path":relative,"revision":assets::hash(&bytes),"encoding":encoding,"content":content}),
            )
        }
        "write" | "delete" => {
            editable(e)?;
            if path == e.scene_path()
                || (path.exists()
                    && fs::canonicalize(&path).ok() == fs::canonicalize(e.scene_path()).ok())
            {
                return Err("Edit the active scene with scene_apply and scene_save.".into());
            }
            if relative.ends_with(".epokasset") {
                return Err("Use asset_document, mesh_create or asset_import for packages.".into());
            }
            let expected = string(a, "revision")?;
            let previous = if path.exists() {
                Some(read_file(&path)?)
            } else {
                None
            };
            if previous
                .as_ref()
                .map_or("absent".into(), |b| assets::hash(b))
                != expected
            {
                return Err(
                    "File changed. Read it again, or use revision='absent' for a new file.".into(),
                );
            }
            let bytes = if action == "write" {
                let content = string(a, "content")?;
                match a["encoding"].as_str().unwrap_or("text") {
                    "text" => content.into_bytes(),
                    "base64" => base64::engine::general_purpose::STANDARD
                        .decode(content)
                        .map_err(|e| e.to_string())?,
                    _ => return Err("Use text or base64 encoding".into()),
                }
            } else {
                vec![]
            };
            if bytes.len() > 2 * 1024 * 1024 {
                return Err("File write limit is 2 MiB per request".into());
            }
            if action == "write" && relative.ends_with(".epokmap") {
                let scene: Scene =
                    crate::document::from_slice(&bytes).map_err(|e| e.to_string())?;
                scene.validate()?;
            }
            let saved = previous.as_ref().map(|b| backup(e, &path, b)).transpose()?;
            if action == "write" {
                assets::atomic_write(&path, &bytes, previous.as_ref().map(|_| expected.as_str()))?;
            } else {
                if previous.is_none() {
                    return Err("File does not exist".into());
                }
                fs::remove_file(&path).map_err(|e| e.to_string())?;
            }
            refresh(e);
            e.log(format!("MCP: {action} {relative}"));
            Ok(
                json!({"path":relative,"revision":if action=="write"{assets::hash(&bytes)}else{"absent".into()},"backup":saved}),
            )
        }
        _ => Err("Use list, read, write or delete".into()),
    }
}
fn read_file(path: &Path) -> Result<Vec<u8>, String> {
    if fs::metadata(path).map_err(|e| e.to_string())?.len() > 2 * 1024 * 1024 {
        return Err("File read limit is 2 MiB per request".into());
    }
    assets::read_bounded(path)
}
fn schema() -> Value {
    json!({"actor":Actor::cube("Example".into()),"components":{"audio":crate::audio::AudioSource::default(),"light":crate::lighting::Light::default(),"blob_shadow":crate::shadows::BlobShadow::default(),"skeletal_mesh":crate::skeletal::Component::new(uuid::Uuid::nil()),"editable_mesh":crate::mesh::Component::new(uuid::Uuid::nil()),"material":crate::scene::Material::default(),"canvas":crate::hud::Canvas::default(),"rect":crate::hud::RectTransform::default(),"image":crate::hud::Image::default(),"text":crate::hud::Text::default(),"progress":crate::hud::ProgressBar::default(),"lighting":crate::lighting::MeshLighting::default(),"environment":crate::lighting::Settings::default()},"kinds":["Empty","Mesh","Camera"],"notes":"Transforms are local to parent. Actor indices change after deletion. Replace nil asset UUIDs in examples with UUIDs from asset_list or mesh_create. Optional components can be removed with null.","example":{"op":"create","actor":{"name":"Example Cube","kind":"Mesh","position":[0,0.5,0],"material":{"color":[0.2,0.6,1.0]}}}})
}
pub fn validate_arguments(name: &str, value: &Value) -> Result<(), String> {
    let tool = catalog()
        .into_iter()
        .find(|t| t.name == name)
        .ok_or_else(|| format!("Unknown tool: {name}"))?;
    validate_schema(value, &json!(tool.input_schema), "arguments")
}
// Only the JSON Schema vocabulary used by the catalog is needed here. Domain
// validators still check scene structure, geometry, references and PSX budgets.
fn validate_schema(value: &Value, schema: &Value, path: &str) -> Result<(), String> {
    let types = match &schema["type"] {
        Value::String(t) => vec![t.as_str()],
        Value::Array(ts) => ts.iter().filter_map(Value::as_str).collect(),
        _ => vec![],
    };
    if !types.is_empty()
        && !types.iter().any(|t| match *t {
            "object" => value.is_object(),
            "array" => value.is_array(),
            "string" => value.is_string(),
            "integer" => value.is_i64() || value.is_u64(),
            "number" => value.is_number(),
            "boolean" => value.is_boolean(),
            "null" => value.is_null(),
            _ => false,
        })
    {
        return Err(format!("{path}: expected {}", types.join(" or ")));
    }
    if let Some(values) = schema["enum"].as_array()
        && !values.contains(value)
    {
        return Err(format!("{path}: value is not one of {values:?}"));
    }
    if let Some(number) = value.as_f64() {
        for (key, minimum) in [("minimum", true), ("maximum", false)] {
            if let Some(bound) = schema[key].as_f64()
                && if minimum {
                    number < bound
                } else {
                    number > bound
                }
            {
                return Err(format!("{path}: violates {key} {bound}"));
            }
        }
    }
    if let Some(object) = value.as_object() {
        if let Some(required) = schema["required"].as_array() {
            for key in required.iter().filter_map(Value::as_str) {
                if !object.contains_key(key) {
                    return Err(format!("{path}: missing '{key}'"));
                }
            }
        }
        for (key, value) in object {
            if let Some(child) = schema["properties"].get(key) {
                validate_schema(value, child, &format!("{path}.{key}"))?;
            } else if schema["additionalProperties"] == false {
                return Err(format!("{path}: unknown field '{key}'"));
            }
        }
    }
    if let Some(array) = value.as_array() {
        if schema["minItems"]
            .as_u64()
            .is_some_and(|n| array.len() < n as usize)
            || schema["maxItems"]
                .as_u64()
                .is_some_and(|n| array.len() > n as usize)
        {
            return Err(format!("{path}: invalid array length"));
        }
        if let Some(item) = schema.get("items") {
            for (i, value) in array.iter().enumerate() {
                validate_schema(value, item, &format!("{path}[{i}]"))?;
            }
        }
    }
    Ok(())
}
pub fn catalog() -> Vec<rmcp::model::Tool> {
    let s = json!({"type":"string"});
    let n = json!({"type":"integer","minimum":0});
    let b = json!({"type":"boolean"});
    let o = json!({"type":"object"});
    let specs = vec![
        (
            "asset_manage",
            "Move, duplicate or trash an imported asset by path and revision. Trash refuses referenced assets. ModelSource duplication uses a fresh FBX import.",
            json!({"action":{"enum":["move","duplicate","trash"]},"path":s,"revision":s,"destination":s}),
            vec!["action", "path", "revision"],
            false,
        ),
        (
            "editor_state",
            "Read the active project, selection, revision, camera, job state and errors.",
            json!({}),
            vec![],
            true,
        ),
        (
            "scene_read",
            "Read the complete scene and its revision before editing.",
            json!({}),
            vec![],
            true,
        ),
        (
            "scene_schema",
            "Read Actor/component defaults and an example scene operation.",
            json!({}),
            vec![],
            true,
        ),
        (
            "scene_apply",
            "Atomically edit the scene. Operations: create(actor), update(index,patch), delete(index, including children), duplicate(index), reparent(index,parent,keep_world), environment(patch), replace(scene). Read scene_read first. No automatic save.",
            json!({"revision":s,"operations":{"type":"array","minItems":1,"maxItems":128,"items":{"type":"object","properties":{"op":{"enum":["create","update","delete","duplicate","reparent","environment","replace"]},"index":n,"actor":o,"patch":o,"scene":o,"parent":{"type":["integer","null"],"minimum":0},"keep_world":b},"required":["op"],"additionalProperties":false}}}),
            vec!["revision", "operations"],
            false,
        ),
        (
            "scene_actors",
            "List the actors of the open scene: class, family, domain, active flag, logical parent, spatial attachment, property overrides and components. Each component reports its class, root and inherited flags, its authored properties, which of them are explicit overrides, and whether it can be removed. Authored actors come from the document; derived ones are the in-memory view of legacy actors that have not been migrated yet and cannot be edited. Also returns the placeable actor classes for scene_add_actor.",
            json!({}),
            vec![],
            true,
        ),
        (
            "scene_add_actor",
            "Add an actor of a placeable class to the open scene, with its root component for the class domain. Optional parent is the id of another actor (logical parent, not a transform parent). Actor tools stay the legacy path: scene_apply create/update/delete still authors epok::Actor records.",
            json!({"revision":s,"class":s,"name":s,"parent":{"type":["string","null"]}}),
            vec!["revision", "class"],
            false,
        ),
        (
            "scene_remove_actor",
            "Remove one authored actor by id. References to it from other actors are cleared rather than left dangling. Legacy actors are removed with scene_apply delete.",
            json!({"revision":s,"id":s}),
            vec!["revision", "id"],
            false,
        ),
        (
            "scene_set_actor",
            "Change an authored actor: active flag, name, reflected property overrides, logical parent and component set. A null property value clears the override and restores the class default. parent is the id of another actor or null for the map root; the spatial attachment follows only when both actors share a spatial domain. components is {\"add\":[{\"class\":\"epok::AudioComponent\",\"name\":\"Music\"}],\"remove\":[\"component-uuid\"]}; the root component and components inherited from the class cannot be removed, and the whole component set is validated against the class model before anything is applied.",
            json!({"revision":s,"id":s,"active":b,"name":s,"properties":o,"parent":{"type":["string","null"]},"components":o}),
            vec!["revision", "id"],
            false,
        ),
        (
            "scene_history",
            "Undo or redo the last MCP scene batch, rejecting intervening edits.",
            json!({"revision":s,"action":{"enum":["undo","redo"]}}),
            vec!["revision", "action"],
            false,
        ),
        (
            "scene_save",
            "Save the current scene after verifying its revision.",
            json!({"revision":s}),
            vec!["revision"],
            false,
        ),
        (
            "scene_open",
            "Open a scene under assets/. Create new scene files with project_files first. Unsaved changes require explicit discard.",
            json!({"revision":s,"path":s,"discard_unsaved":b}),
            vec!["revision", "path"],
            false,
        ),
        (
            "actor_select",
            "Select an Actor by UUID (null clears selection); optionally frame it in the viewport.",
            json!({"revision":s,"id":{"type":["string","null"]},"frame":b}),
            vec!["revision", "id"],
            false,
        ),
        (
            "editor_view",
            "Read/change Scene camera, grid, wireframe and the authoring domain. view_mode is \"3d\", \"2d\" or \"ui\"; the legacy scene_2d boolean still works (true = ui, false = 3d) and view_mode wins when both are sent. view fields: yaw, pitch, center[3] (orbit pivot), distance (eye to pivot), zoom (lens), fly_speed, phase.",
            json!({"view":o,"grid":b,"wire":b,"scene_2d":b,"view_mode":{"enum":["3d","2d","ui"]}}),
            vec![],
            false,
        ),
        (
            "editor_control",
            "Build, play, stop, pause, resume, reset_psx, step, bake, export, frame selection or open settings/imports. reset_psx reboots an active physical console; step requires the emulator. Jobs return immediately; poll editor_state and logs_read for confirmation.",
            json!({"action":{"enum":["build","play","stop","pause","resume","reset_psx","step","bake","export","reset_layout","project_settings","editor_preferences","serial_connection","serial_prepare","imports","frame_selected"]}}),
            vec!["action"],
            false,
        ),
        (
            "game_input",
            "Hold PSX controller button bits for 1..5000 ms (default 150); 0 releases. Bits: Select=0, L3=1, R3=2, Start=3, Up=4, Right=5, Down=6, Left=7, L2=8, R2=9, L1=10, R1=11, Triangle=12, Circle=13, Cross=14, Square=15.",
            json!({"buttons":{"type":"integer","minimum":0,"maximum":65535},"duration_ms":{"type":"integer","minimum":1,"maximum":5000}}),
            vec!["buttons"],
            false,
        ),
        (
            "viewer_screenshot",
            "Capture a PNG from editor, scene (full viewport), hud, or game (emulator required). Returns MCP image content.",
            json!({"target":{"enum":["editor","scene","hud","game"]}}),
            vec!["target"],
            true,
        ),
        (
            "logs_read",
            "Read the most recent editor/build/import log lines (maximum 400).",
            json!({"limit":{"type":"integer","minimum":1,"maximum":400}}),
            vec![],
            true,
        ),
        (
            "project_settings",
            "Read project settings and revision; optionally merge a patch with the current revision.",
            json!({"patch":o,"revision":s}),
            vec![],
            false,
        ),
        (
            "project_files",
            "List one assets/ directory, read text/base64 (2 MiB max), write or delete. Mutations require current SHA256 revision or 'absent' for new files. Prior bytes are backed up. Active scene and packages use dedicated tools.",
            json!({"action":{"enum":["list","read","write","delete"]},"path":s,"encoding":{"enum":["text","base64"]},"content":s,"revision":s}),
            vec![],
            false,
        ),
        (
            "asset_list",
            "Refresh and list imported asset UUIDs, metadata, revisions, pending sources and problems.",
            json!({}),
            vec![],
            true,
        ),
        (
            "asset_import",
            "Import an FBX or audio source already in assets/. FBX animation_storage selects RigidGte or BakedVertices. Destination must be .epokasset. Reimport requires current asset revision. Poll editor_state for completion.",
            json!({"source":s,"destination":s,"revision":s,"animation_storage":{"enum":["RigidGte","BakedVertices"]},"audio_settings":o,"sequence_settings":o,"bank_settings":o,"vb_source":s}),
            vec!["source", "destination"],
            false,
        ),
        (
            "mesh_create",
            "Create an EditableMesh asset from a Box/Plane/Ramp/Stairs primitive or a complete mesh document. Returns UUID and component to attach via scene_apply.",
            json!({"path":s,"shape":{"enum":["Box","Plane","Ramp","Stairs"]},"size":{"type":"array","items":{"type":"number"},"minItems":3,"maxItems":3},"steps":{"type":"integer","minimum":1,"maximum":64},"document":o}),
            vec!["path"],
            false,
        ),
        (
            "asset_document",
            "Read or replace an authored mesh, skeleton, animation or material document. Replacement requires revision and preserves the asset UUID. Source FBX/audio packages use asset_import.",
            json!({"path":s,"document":o,"revision":s}),
            vec!["path"],
            false,
        ),
    ];
    specs.into_iter().map(|(name,description,properties,required,read)|decode(json!({"name":name,"description":description,"inputSchema":{"type":"object","properties":properties,"required":required,"additionalProperties":false},"annotations":{"readOnlyHint":read,"destructiveHint":!read,"openWorldHint":false}})).expect("Static MCP tool schema")).collect()
}
