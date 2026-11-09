//! Authored class composition for source-based Actor classes. Blueprint classes
//! keep this same Template in their class asset; no extra runtime class is made.
use crate::{blueprint::Registry, blueprint_templates::Template};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

pub const VERSION: u32 = 4;
pub const SOURCE_SET: &str = "class-composition-sources";
const MAX_BYTES: usize = 4 * 1024 * 1024;

fn component_identity(actor: uuid::Uuid, member: &str) -> uuid::Uuid {
    use sha2::{Digest, Sha256};
    let mut hash = Sha256::new();
    hash.update(b"class-component-v1");
    hash.update(actor.as_bytes());
    hash.update(member.as_bytes());
    let mut bytes = [0; 16];
    bytes.copy_from_slice(&hash.finalize()[..16]);
    uuid::Uuid::from_bytes(bytes)
}
/// Materialize reflected native defaults without replacing authored components.
/// Stable local identities keep new defaults stable across class refresh/cook.
pub fn hydrate(
    actor: &mut crate::scene::Actor,
    model: &crate::object_model::Model,
) -> Result<(), String> {
    use crate::{actor_document as doc, object_model as om, reflection_schema::Domain};
    // Inspectors and the canvas edit the existing built-in projection. Commit
    // those edits before reading it back after adding derived native defaults.
    // A fresh actor has no projection yet; its root comes from the class domain.
    if !actor.components.is_empty() {
        crate::actor_components::sync(actor);
    }
    let class = actor
        .class
        .resolve(model)
        .ok_or("Actor class is unresolved.")?;
    for default in &class.default_components {
        let component = model.class(&default.class).ok_or_else(|| {
            format!(
                "{}: native default {}.{} has unresolved component class {}.",
                actor.name, class.cpp_name, default.field, default.class
            )
        })?;
        let existing = actor
            .components
            .iter()
            .position(|c| c.default_id.as_ref() == Some(&default.id))
            .or_else(|| {
                default
                    .root
                    .then(|| {
                        actor.components.iter().position(|c| {
                            c.root
                                && c.default_id.is_none()
                                && c.class.resolve(model).is_some_and(|c| c.id == component.id)
                        })
                    })
                    .flatten()
            })
            .or_else(|| {
                // A newly declared singleton default can adopt a previously
                // authored control of that class instead of duplicating it.
                component
                    .component
                    .as_ref()
                    .filter(|c| c.cardinality == crate::reflection_schema::Cardinality::Single)
                    .and_then(|_| {
                        actor.components.iter().position(|c| {
                            c.default_id.is_none()
                                && c.class.resolve(model).is_some_and(|c| c.id == component.id)
                        })
                    })
            });
        let index = if let Some(index) = existing {
            if actor.components[index]
                .class
                .resolve(model)
                .is_none_or(|c| c.id != component.id)
            {
                return Err(format!(
                    "{}: native component {} changed class; repair its authored values first.",
                    actor.name, default.field
                ));
            }
            index
        } else {
            let label = default.name.as_deref().unwrap_or(&default.field);
            actor.components.push(doc::ComponentInstance::new(
                component_identity(actor.id, &default.id),
                doc::ClassReference::new(&component.cpp_name, &component.id),
                label,
            ));
            actor.components.len() - 1
        };
        let instance = &mut actor.components[index];
        instance.root = default.root;
        instance.inherited = true;
        instance.default_id = Some(default.id.clone());
    }
    for default in &class.default_components {
        if let Some(parent) = &default.attach_to {
            let parent = class
                .default_components
                .iter()
                .find(|d| &d.field == parent)
                .and_then(|d| {
                    actor
                        .components
                        .iter()
                        .find(|c| c.default_id.as_ref() == Some(&d.id))
                })
                .map(|c| c.id)
                .ok_or("Native component attachment parent is missing.")?;
            actor
                .components
                .iter_mut()
                .find(|c| c.default_id.as_ref() == Some(&default.id))
                .unwrap()
                .attach_parent = Some(parent);
        }
    }
    if actor.root().is_none()
        && let Some(root) = match class.domain {
            Domain::World3D => Some(om::SCENE_COMPONENT3D_ID),
            Domain::World2D => Some(om::SCENE_COMPONENT2D_ID),
            Domain::UI => Some(om::RECT_TRANSFORM_COMPONENT_ID),
            Domain::None => None,
        }
        .and_then(|id| model.class(id))
    {
        let mut component = doc::ComponentInstance::new(
            component_identity(actor.id, "root"),
            doc::ClassReference::new(&root.cpp_name, &root.id),
            "Root",
        );
        component.root = true;
        actor.components.insert(0, component);
    }
    actor.refresh_components();
    Ok(())
}
pub fn actor(
    model: &crate::object_model::Model,
    class: &str,
    name: &str,
) -> Result<crate::scene::Actor, String> {
    let class = model.class(class).ok_or("Actor class is unresolved.")?;
    let mut actor = crate::actor_document::ActorInstance::new(
        uuid::Uuid::new_v4(),
        crate::actor_document::ClassReference::new(&class.cpp_name, &class.id),
        name,
    );
    hydrate(&mut actor, model)?;
    Ok(actor)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub version: u32,
    pub class: String,
    pub template: Template,
}
impl Document {
    pub fn new(class: String, template: Template) -> Self {
        Self {
            version: VERSION,
            class,
            template,
        }
    }
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > MAX_BYTES {
            return Err("Class composition exceeds 4 MiB.".into());
        }
        let mut document: Self =
            crate::document::from_slice(bytes).map_err(|e| format!("Class composition: {e}"))?;
        if (document.version == 3
            || ((document.version == 2
                || (document.version == 1 && document.template.instances.is_empty()))
                && !crate::blueprint_templates::uses_literal_null(&document.template)))
            && !crate::blueprint_templates::uses_focus_links(&document.template)
        {
            document.version = VERSION;
        }
        document.validate()?;
        Ok(document)
    }
    fn validate(&self) -> Result<(), String> {
        if self.version != VERSION
            || uuid::Uuid::parse_str(&self.class).is_err()
            || self.class == uuid::Uuid::nil().to_string()
        {
            return Err("Class composition needs version 4 and a persistent class UUID.".into());
        }
        if self.template.actors.len() > crate::blueprint_templates::MAX_ENTITIES
            || self.template.instances.len() > crate::blueprint_templates::MAX_ENTITIES
            || self.template.instances.iter().any(uuid::Uuid::is_nil)
            || self.template.construction.len() > crate::blueprint_templates::MAX_CONSTRUCTION_OPS
        {
            return Err("Class composition exceeds the existing template bounds.".into());
        }
        let mut identities = BTreeSet::new();
        for actor in &self.template.actors {
            for id in
                std::iter::once(actor.entity.id).chain(actor.entity.components.iter().map(|c| c.id))
            {
                if id.is_nil() || !identities.insert(id) {
                    return Err(
                        "Class composition contains an empty or duplicated local object identity."
                            .into(),
                    );
                }
            }
        }
        Ok(())
    }
}
#[derive(Clone)]
pub struct Source {
    pub path: PathBuf,
    pub document: Document,
    pub revision: String,
}
#[derive(Default)]
pub struct Index {
    pub sources: BTreeMap<String, Source>,
}
impl Index {
    pub fn templates(&self) -> BTreeMap<String, Template> {
        self.sources
            .iter()
            .map(|(class, source)| (class.clone(), source.document.template.clone()))
            .collect()
    }
    pub fn signature(&self, root: &Path) -> String {
        crate::scene_dependencies::hash(
            self.sources
                .iter()
                .map(|(class, source)| {
                    (
                        class,
                        crate::assets::path_string(root, &source.path),
                        &source.revision,
                    )
                })
                .collect::<Vec<_>>(),
        )
    }
    pub fn validate(&self, root: &Path, registry: &Registry) -> Result<(), String> {
        if self.sources.is_empty() {
            return Ok(());
        }
        let assets = root
            .join("assets")
            .canonicalize()
            .map_err(|e| e.to_string())?;
        let model = registry.model().map_err(|errors| {
            errors
                .into_iter()
                .map(|e| e.message)
                .collect::<Vec<_>>()
                .join("\n")
        })?;
        for (id, source) in &self.sources {
            let class = registry
                .classes
                .get(id)
                .ok_or_else(|| format!("{}: class {id} is missing.", source.path.display()))?;
            let declaration = if class.source.file.is_absolute() {
                class.source.file.clone()
            } else {
                root.join(&class.source.file)
            };
            if !declaration
                .canonicalize()
                .is_ok_and(|path| path.starts_with(&assets))
            {
                return Err(format!(
                    "{}: derive a project class before editing SDK class composition.",
                    source.path.display()
                ));
            }
            if class.provider.id == "blueprint" {
                return Err(format!(
                    "{}: Blueprint composition belongs in its existing class asset.",
                    source.path.display()
                ));
            }
            if !model
                .class(id)
                .is_some_and(|c| c.family == crate::reflection_schema::ClassFamily::Actor)
            {
                return Err(format!(
                    "{}: class composition requires an Actor class.",
                    source.path.display()
                ));
            }
        }
        Ok(())
    }
}
pub fn load(root: &Path) -> Result<Index, String> {
    let mut paths = vec![];
    let mut errors = vec![];
    if root.join("assets").exists() {
        crate::assets::walk(&root.join("assets"), &mut paths, &mut errors);
    }
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    paths.sort();
    let mut index = Index::default();
    for path in paths.into_iter().filter(|p| {
        p.extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("epokclass"))
    }) {
        let bytes = crate::assets::read_bounded(&path)?;
        let document = Document::parse(&bytes).map_err(|e| format!("{}: {e}", path.display()))?;
        let class = document.class.clone();
        let source = Source {
            path,
            document,
            revision: crate::assets::hash(&bytes),
        };
        if let Some(previous) = index.sources.insert(class.clone(), source) {
            return Err(format!(
                "Class {class} has more than one composition, including {}.",
                previous.path.display()
            ));
        }
    }
    Ok(index)
}
/// Editor projection of the parent's authored layers. Both providers use the
/// same Template; this never executes construction scripts or project methods.
pub fn inherited(root: &Path, registry: &Registry, parent: &str) -> Result<Vec<Template>, String> {
    let index = load(root)?;
    index.validate(root, registry)?;
    let class = registry
        .classes
        .get(parent)
        .ok_or("Parent class is missing.")?;
    let files = crate::blueprint_asset::load_all(root)?;
    registry
        .ancestry(&class.cpp_name)
        .into_iter()
        .filter_map(|class| {
            if class.provider.id == "blueprint" {
                Some(
                    files
                        .iter()
                        .find(|f| f.asset.id == class.id)
                        .map(|f| f.asset.template.clone())
                        .ok_or_else(|| format!("Blueprint class {} is missing.", class.cpp_name)),
                )
            } else {
                index
                    .sources
                    .get(&class.id)
                    .map(|s| Ok(s.document.template.clone()))
            }
        })
        .collect()
}
pub fn sources(
    root: &Path,
    registry: &Registry,
) -> Result<crate::blueprint_templates::CompositionSources, String> {
    let index = load(root)?;
    index.validate(root, registry)?;
    Ok(crate::blueprint_templates::CompositionSources {
        blueprints: crate::blueprint_asset::load_all(root)?,
        companions: index.templates(),
    })
}
fn validate_candidate(
    root: &Path,
    document: &Document,
    registry: &Registry,
    path: &Path,
) -> Result<(), String> {
    document.validate()?;
    let mut index = load(root)?;
    if index
        .sources
        .get(&document.class)
        .is_some_and(|s| s.path != path)
    {
        return Err("This class already has a composition; edit that class document.".into());
    }
    index.sources.insert(
        document.class.clone(),
        Source {
            path: path.into(),
            document: document.clone(),
            revision: String::new(),
        },
    );
    index.validate(root, registry)?;
    let files = crate::blueprint_asset::load_all(root)?;
    let templates = index.templates();
    let resolved =
        crate::blueprint_templates::resolve_class(&files, registry, &document.class, &templates)?;
    if !resolved.actors.is_empty() {
        let class = &registry.classes[&document.class];
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
    Ok(())
}
pub fn create(
    root: &Path,
    path: &str,
    document: &Document,
    registry: &Registry,
) -> Result<(), String> {
    let destination = crate::assets::inside(root, path)?;
    if !path.replace('\\', "/").starts_with("assets/")
        || destination.extension().is_none_or(|e| e != "epokclass")
    {
        return Err("Class composition must be an .epokclass document under assets/.".into());
    }
    if destination.exists() {
        return Err("Class composition destination already exists.".into());
    }
    validate_candidate(root, document, registry, &destination)?;
    crate::assets::atomic_write(&destination, &crate::document::to_vec(document)?, None)
}
pub fn save(
    root: &Path,
    source: &Source,
    document: &Document,
    registry: &Registry,
) -> Result<String, String> {
    if document.class != source.document.class {
        return Err("Changing a class composition's owner requires creating a new class.".into());
    }
    validate_candidate(root, document, registry, &source.path)?;
    let bytes = crate::document::to_vec(document)?;
    crate::assets::atomic_write(&source.path, &bytes, Some(&source.revision))?;
    Ok(crate::assets::hash(&bytes))
}
pub fn observe_current(root: &Path) -> Result<(), String> {
    match load(root) {
        Ok(index) => crate::artifact_dependencies::transaction(root, |graph| {
            graph.publish(SOURCE_SET, index.signature(root), Default::default());
        })
        .map(|_| ()),
        Err(error) => {
            crate::artifact_dependencies::transaction(root, |graph| {
                graph.invalidate(SOURCE_SET, &error)
            })?;
            Err(error)
        }
    }
}
pub fn refresh_instances(
    root: &Path,
    scene: &mut crate::scene::Scene,
    files: &[crate::blueprint_asset::AssetFile],
    registry: &Registry,
) -> Result<(), String> {
    let index = load(root)?;
    index.validate(root, registry)?;
    crate::blueprint_templates::refresh_class_instances(scene, files, registry, &index.templates())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::{actor_document::Attachment, blueprint_templates as templates, scene::Scene};
    use serde_json::json;
    pub const BASE: &str = "301a291b-4772-4ab6-ae77-d6f973639242";
    const CPP: &str = "eb34d3af-eed3-41bd-a1be-315508df1318";
    const LABEL: &str = "47ec3265-ddab-4d28-87c5-6915dd3d7a32";
    pub fn fixture() -> (PathBuf, Registry, Vec<crate::scripts::Script>) {
        let root = crate::workspace::tests::temp("class-composition");
        crate::workspace::create(&root, "Widget Classes", crate::workspace::Template::Basic)
            .unwrap();
        std::fs::write(root.join("assets/scripts/WidgetClasses.hpp"),r#"#pragma once
#include "epok.hpp"
class EPOK_CLASS(Blueprintable, Placeable, Spawnable, Id="301a291b-4772-4ab6-ae77-d6f973639242") CppWidgetBase : public epok::UIActor {
public:
EPOK_COMPONENT(Name="Canvas",Id="c30fa1ed-9cc8-4544-833f-f5d059b7adeb") epok::CanvasComponent canvas;
EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::TextComponent",Id="47ec3265-ddab-4d28-87c5-6915dd3d7a32") epok::ObjectId label{};
};
class EPOK_CLASS(Blueprintable, Placeable, Spawnable, Id="eb34d3af-eed3-41bd-a1be-315508df1318") CppWidget : public CppWidgetBase {
public:
EPOK_COMPONENT(Name="Audio",Id="1b9cbfd1-0450-4927-81dd-4cd66dc62dcd") epok::AudioComponent audio;
};
"#).unwrap();
        std::fs::write(
            root.join("assets/scripts/LuaWidget.lua"),
            crate::lua_asset::template("LuaWidget", "CppWidgetBase"),
        )
        .unwrap();
        std::fs::create_dir_all(root.join("assets/Blueprints")).unwrap();
        let bp = crate::blueprint_asset::BlueprintAsset::new("BP_Widget".into(), BASE.into());
        crate::blueprint_asset::create(&root.join("assets/Blueprints/BP_Widget.epokbp"), &bp)
            .unwrap();
        let catalog = crate::scripts::catalog(&root).unwrap();
        let registry = crate::blueprint::native_registry(&root, &catalog).unwrap();
        (root, registry, catalog)
    }
    pub fn base_template(root: &Path, registry: &Registry) -> Template {
        let model = registry.model().unwrap();
        let mut scene = Scene::default();
        scene.actors.clear();
        let root_index =
            crate::editor::place_class(&mut scene, root, registry, &model, BASE).unwrap();
        scene.actors[root_index].rect = Some(crate::hud::RectTransform {
            anchor_min: [0.; 2],
            anchor_max: [1.; 2],
            pivot: [0.; 2],
            position: [0.; 2],
            size: [0.; 2],
            ..Default::default()
        });
        let child = crate::editor::place_class(
            &mut scene,
            root,
            registry,
            &model,
            crate::object_model::UI_ACTOR_ID,
        )
        .unwrap();
        scene.actors[child].name = "Label".into();
        scene.actors[child].logical_parent = Some(scene.actors[root_index].id);
        scene.actors[child].attach = Some(Attachment {
            actor: scene.actors[root_index].id,
            component: None,
        });
        scene.actors[child].rect = Some(crate::hud::RectTransform {
            size: [140., 24.],
            position: [12., 34.],
            ..Default::default()
        });
        scene.actors[child].text = Some(crate::hud::Text {
            text: "Authored widget".into(),
            ..Default::default()
        });
        scene.sync_actor_components();
        let label = scene.actors[child]
            .components
            .iter()
            .find(|c| c.class.name == "epok::TextComponent")
            .unwrap()
            .id;
        scene.actors[root_index]
            .properties
            .insert("label".into(), json!(label));
        scene.actors[root_index].overrides.insert("label".into());
        scene.refresh_actor_hierarchy();
        templates::capture(&scene, root_index, registry).unwrap()
    }
    #[test]
    fn nested_widget_classes_share_resolution_placement_refresh_and_cook_in_all_providers() {
        let (root, _, _) = fixture();
        const HOST: &str = "d5fcb678-1ec7-51ab-ad0b-87d11bf26845";
        const FIRST: &str = "878d0c47-b322-52f1-a7b7-f88a371db313";
        std::fs::write(root.join("assets/scripts/WidgetHost.hpp"), r#"#pragma once
#include "epok.hpp"
class EPOK_CLASS(Blueprintable,Placeable,Spawnable,Id="d5fcb678-1ec7-51ab-ad0b-87d11bf26845") WidgetHost:public epok::UIActor {
public:
EPOK_COMPONENT(Name="Canvas",Id="4013c6d0-d477-524a-b081-0dc9eb0367bc") epok::CanvasComponent canvas;
EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::TextComponent",Id="878d0c47-b322-52f1-a7b7-f88a371db313") epok::ObjectId first{};
};
"#).unwrap();
        let catalog = crate::scripts::catalog(&root).unwrap();
        let registry = crate::blueprint::native_registry(&root, &catalog).unwrap();
        let model = registry.model().unwrap();
        create(
            &root,
            "assets/WidgetBase.epokclass",
            &Document::new(BASE.into(), base_template(&root, &registry)),
            &registry,
        )
        .unwrap();
        let root_actor = actor(&model, HOST, "Host").unwrap();
        let host_root = root_actor.id;
        let mut template = Template {
            actors: vec![templates::TemplateEntity {
                entity: root_actor,
                parent: None,
            }],
            ..Default::default()
        };
        let classes = [
            CPP.to_owned(),
            registry.named("LuaWidget").unwrap().id.clone(),
            registry.named("BP_Widget").unwrap().id.clone(),
        ];
        let mut sites = vec![];
        for (index, class) in classes.iter().enumerate() {
            let site = crate::blueprint_editor::template_editor::add_class_instance(
                &mut template,
                &model,
                host_root,
                class,
                &format!("Item {index}"),
            )
            .unwrap();
            sites.push(site);
        }
        let context = sources(&root, &registry).unwrap();
        let binding = || crate::scene::ClassDefaults {
            name: "WidgetHost".into(),
            class_id: Some(HOST.into()),
            ..Default::default()
        };
        let first = context
            .resolve_layers(&registry, HOST, &[&template])
            .unwrap();
        assert_eq!(first.actors.len(), 7);
        assert_eq!(first.nested_roots.len(), 3);
        let prototype = first.scene(binding(), &registry).unwrap();
        for (n, site) in sites.iter().enumerate() {
            let original = prototype.actors.iter().find(|a| a.id == *site).unwrap();
            let mut edited = original.clone();
            let rect = edited.rect.as_mut().unwrap();
            rect.anchor_min = [0.; 2];
            rect.anchor_max = [0.; 2];
            rect.pivot = [0.; 2];
            rect.position = [10. + n as f32 * 200., 40.];
            rect.size = [160., 100.];
            crate::actor_components::sync(&mut edited);
            crate::blueprint_editor::template_editor::apply_entity_changes(
                &mut template,
                original,
                &edited,
            )
            .unwrap();
        }
        let label = uuid::Uuid::parse_str(
            prototype
                .actors
                .iter()
                .find(|a| a.id == sites[0])
                .unwrap()
                .properties["label"]
                .as_str()
                .unwrap(),
        )
        .unwrap();
        template.references.push(templates::ObjectReference {
            owner: host_root,
            member: FIRST.into(),
            target: label,
        });
        let resolved = context
            .resolve_layers(&registry, HOST, &[&template])
            .unwrap();
        assert_eq!(
            resolved,
            context
                .resolve_layers(&registry, HOST, &[&template])
                .unwrap(),
            "namespace identities never depend on random placement IDs"
        );
        let mut preview = resolved.scene(binding(), &registry).unwrap();
        preview.display_size = [640, 480];
        let boxes = crate::hud::layouts(&preview);
        let order = crate::hud::order(&preview);
        assert_eq!(order.len(), 7);
        assert_eq!(
            order.iter().collect::<BTreeSet<_>>().len(),
            7,
            "nested Canvas controls appear once in paint/hit order"
        );
        for (n, site) in sites.iter().enumerate() {
            let i = preview.actors.iter().position(|a| a.id == *site).unwrap();
            assert_eq!(
                boxes[i].unwrap().rect,
                [10. + n as f32 * 200., 40., 160., 100.]
            );
            let label_id = preview.actors[i].properties["label"].as_str().unwrap();
            let child = preview
                .actors
                .iter()
                .position(|a| a.components.iter().any(|c| c.id.to_string() == label_id))
                .unwrap();
            assert_eq!(
                boxes[child].unwrap().rect,
                [32. + n as f32 * 200., 112., 140., 24.]
            );
        }
        let (_, stats) = crate::hud_native::compile(&preview);
        assert_eq!(stats[1], 42);
        assert_eq!(stats[4], 0);
        let nested = preview
            .actors
            .iter()
            .position(|a| a.id == sites[2])
            .unwrap();
        preview.actors[nested].rect.as_mut().unwrap().rotation = 90.;
        preview.sync_actor_components();
        preview.hud_budget.rotated = 14;
        assert!(
            crate::hud::validate(&preview)
                .unwrap_err()
                .contains("budget")
        );
        preview.hud_budget.rotated = 15;
        crate::hud::validate(&preview).unwrap();
        let (_, stats) = crate::hud_native::compile(&preview);
        assert_eq!(stats[5], 14);
        assert_eq!(stats[1], 28);
        assert_eq!(stats[4], 0);
        preview.actors[nested]
            .canvas
            .as_mut()
            .unwrap()
            .screen_overlay = true;
        preview.sync_actor_components();
        let boxes = crate::hud::layouts(&preview);
        assert_eq!(boxes[nested].unwrap().rect, [0., 0., 640., 480.]);
        let (_, stats) = crate::hud_native::compile(&preview);
        assert_eq!(stats[5], 0);
        assert_eq!(stats[1], 42);
        assert_eq!(stats[4], 0);
        // Importing a saved scene keeps child classes linked, including direct
        // authored edits that have not been recorded as placement overrides.
        let mut capture_scene = Scene {
            actors: vec![actor(&model, HOST, "Capture Host").unwrap()],
            ..Default::default()
        };
        let capture_root = capture_scene.actors[0].id;
        for (n, class) in classes.iter().enumerate() {
            let i = crate::editor::place_class(&mut capture_scene, &root, &registry, &model, class)
                .unwrap();
            capture_scene.actors[i].logical_parent = Some(capture_root);
            capture_scene.actors[i].attach = Some(Attachment {
                actor: capture_root,
                component: None,
            });
            let rect = capture_scene.actors[i].rect.as_mut().unwrap();
            rect.anchor_min = [0.; 2];
            rect.anchor_max = [0.; 2];
            rect.pivot = [0.; 2];
            rect.position = [10. + n as f32 * 200., 40.];
            rect.size = [160., 100.];
            if n == 0 {
                let label = capture_scene.actors[i].properties["label"].clone();
                capture_scene.actors[0]
                    .properties
                    .insert("first".into(), label);
            }
        }
        capture_scene.refresh_actor_hierarchy();
        capture_scene.sync_actor_components();
        let capture =
            templates::capture_with_sources(&capture_scene, 0, &registry, &context).unwrap();
        assert_eq!(capture.template.actors.len(), 4);
        assert_eq!(capture.template.instances.len(), 3);
        assert_eq!(capture.template.references.len(), 1);
        let captured = context
            .resolve_layers(&registry, HOST, &[&capture.template])
            .unwrap();
        let mut captured = captured.scene(binding(), &registry).unwrap();
        captured.display_size = [640, 480];
        assert_eq!(captured.actors.len(), 7);
        let places = crate::hud::layouts(&captured);
        for i in captured
            .actors
            .iter()
            .enumerate()
            .filter_map(|(i, a)| capture.template.instances.contains(&a.id).then_some(i))
        {
            assert_eq!(places[i].unwrap().rect[2..], [160., 100.]);
        }
        let (_, stats) = crate::hud_native::compile(&captured);
        assert_eq!(stats[1], 42);
        assert_eq!(stats[4], 0);
        let document = Document::new(HOST.into(), template.clone());
        create(&root, "assets/WidgetHost.epokclass", &document, &registry).unwrap();
        let mut scene = Scene::default();
        scene.actors.clear();
        for _ in 0..2 {
            crate::editor::place_class(&mut scene, &root, &registry, &model, HOST).unwrap();
        }
        assert_eq!(scene.actors.len(), 14);
        let owner = scene
            .actors
            .iter()
            .find(|a| a.class.class_id.as_deref() == Some(HOST))
            .unwrap()
            .id;
        scene.duplicate_actor_branch(owner).unwrap();
        let first_control = scene
            .actors
            .iter()
            .find(|a| a.class.class_id.as_deref() == Some(HOST))
            .unwrap()
            .properties["first"]
            .clone();
        let control = scene
            .actors
            .iter()
            .position(|a| a.components.iter().any(|c| json!(c.id) == first_control))
            .unwrap();
        let before = scene.actors[control].clone();
        scene.actors[control].text.as_mut().unwrap().color = [0.3, 0.7, 0.9];
        crate::actor_components::sync(&mut scene.actors[control]);
        templates::record_overrides(&before, &mut scene.actors[control]);
        let identities = scene
            .actors
            .iter()
            .map(|a| (a.id, a.components.iter().map(|c| c.id).collect::<Vec<_>>()))
            .collect::<Vec<_>>();
        let source = load(&root).unwrap().sources.remove(BASE).unwrap();
        let mut changed = source.document.clone();
        changed.template.actors[1]
            .entity
            .text
            .as_mut()
            .unwrap()
            .text = "Updated".into();
        crate::actor_components::sync(&mut changed.template.actors[1].entity);
        let mut added = actor(&model, crate::object_model::UI_ACTOR_ID, "Added label").unwrap();
        added.text = Some(crate::hud::Text {
            text: "New".into(),
            ..Default::default()
        });
        added.attach = Some(Attachment {
            actor: changed.template.actors[0].entity.id,
            component: None,
        });
        crate::actor_components::sync(&mut added);
        changed.template.actors.push(templates::TemplateEntity {
            entity: added,
            parent: Some(changed.template.actors[0].entity.id),
        });
        save(&root, &source, &changed, &registry).unwrap();
        let files = crate::blueprint_asset::load_all(&root).unwrap();
        refresh_instances(&root, &mut scene, &files, &registry).unwrap();
        assert_eq!(scene.actors.len(), 30);
        for (id, components) in identities {
            let actor = scene.actors.iter().find(|a| a.id == id).unwrap();
            assert_eq!(
                actor.components.iter().map(|c| c.id).collect::<Vec<_>>(),
                components
            );
        }
        assert_eq!(
            scene.actors[control].text.as_ref().unwrap().color,
            [0.3, 0.7, 0.9]
        );
        let refreshed = scene.clone();
        refresh_instances(&root, &mut scene, &files, &registry).unwrap();
        assert_eq!(scene, refreshed);
        let path = root.join("assets/scenes/Nested.epokmap");
        scene.save(&path).unwrap();
        assert_eq!(Scene::load(&path).unwrap(), scene);
        let assets = crate::assets::scan(&root, &mut Default::default());
        let cooked = crate::blueprint_spawn::prepare(&root, &catalog, &assets).unwrap();
        let host = cooked.iter().find(|t| t.class == HOST).unwrap();
        assert_eq!(host.scene.actors.len(), 10);
        assert_eq!(
            host.scene
                .actors
                .iter()
                .filter(|a| a.text.as_ref().is_some_and(|t| t.text == "Updated"))
                .count(),
            3
        );
        // Self/indirect cycles, missing identities, and full expansion budgets
        // reject the proposed definition without overwriting its saved source.
        let mut invalid = document.clone();
        crate::blueprint_editor::template_editor::add_class_instance(
            &mut invalid.template,
            &model,
            host_root,
            HOST,
            "Cycle",
        )
        .unwrap();
        let source = load(&root).unwrap().sources.remove(HOST).unwrap();
        let bytes = std::fs::read(&source.path).unwrap();
        assert!(
            save(&root, &source, &invalid, &registry)
                .unwrap_err()
                .contains("cycle")
        );
        assert_eq!(std::fs::read(&source.path).unwrap(), bytes);
        let mut recursive = sources(&root, &registry).unwrap();
        let base = recursive.companions.get_mut(BASE).unwrap();
        let base_root = base.actors[0].entity.id;
        crate::blueprint_editor::template_editor::add_class_instance(
            base,
            &model,
            base_root,
            HOST,
            "Indirect cycle",
        )
        .unwrap();
        assert!(
            recursive
                .resolve_class(&registry, HOST)
                .unwrap_err()
                .contains("cycle")
        );
        // Each of three child classes inherits 85 operations. Include the
        // parent's own operations in the same bound instead of resetting it
        // for every instance.
        let mut bounded = sources(&root, &registry).unwrap();
        let base = bounded.companions.get_mut(BASE).unwrap();
        base.construction = vec![
            templates::ConstructionOp::Translate {
                entity: base.actors[0].entity.id,
                offset: [0.; 3],
            };
            85
        ];
        let host = bounded.companions.get_mut(HOST).unwrap();
        let operation = templates::ConstructionOp::Translate {
            entity: host_root,
            offset: [0.; 3],
        };
        host.construction.push(operation.clone());
        bounded.resolve_class(&registry, HOST).unwrap();
        bounded
            .companions
            .get_mut(HOST)
            .unwrap()
            .construction
            .push(operation);
        assert!(
            bounded
                .resolve_class(&registry, HOST)
                .unwrap_err()
                .contains("256 operations")
        );
        let mut invalid = document.template.clone();
        invalid.actors[1].entity.class.class_id = Some(uuid::Uuid::new_v4().to_string());
        assert!(
            sources(&root, &registry)
                .unwrap()
                .resolve_layers(&registry, HOST, &[&invalid])
                .unwrap_err()
                .contains("missing")
        );
        let mut large = document.template.clone();
        for _ in 0..8 {
            crate::blueprint_editor::template_editor::add_class_instance(
                &mut large, &model, host_root, CPP, "Extra",
            )
            .unwrap();
        }
        assert!(
            sources(&root, &registry)
                .unwrap()
                .resolve_layers(&registry, HOST, &[&large])
                .unwrap_err()
                .contains("32 actors")
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn preceding_source_composition_version_migrates_without_accepting_mislabelled_instances() {
        let mut old = Document::new(uuid::Uuid::new_v4().to_string(), Template::root("Widget"));
        old.version = 1;
        let bytes = crate::document::to_vec(&old).unwrap();
        let migrated = Document::parse(&bytes).unwrap();
        assert_eq!(migrated.version, VERSION);
        assert_eq!(migrated.template, old.template);
        old.template
            .instances
            .insert(old.template.actors[0].entity.id);
        assert!(
            Document::parse(&crate::document::to_vec(&old).unwrap())
                .unwrap_err()
                .contains("version 4")
        );
        old.version = 2;
        assert_eq!(
            Document::parse(&crate::document::to_vec(&old).unwrap())
                .unwrap()
                .version,
            VERSION
        );
        old.template
            .overrides
            .entry(old.template.actors[0].entity.id)
            .or_default()
            .members
            .insert("/properties/target".into(), templates::literal_null());
        assert!(
            Document::parse(&crate::document::to_vec(&old).unwrap())
                .unwrap_err()
                .contains("version 4")
        );
        old.version = 3;
        assert_eq!(
            Document::parse(&crate::document::to_vec(&old).unwrap())
                .unwrap()
                .version,
            VERSION
        );
        old.template
            .overrides
            .entry(old.template.actors[0].entity.id)
            .or_default()
            .members
            .insert(
                "/components/focus/properties/canvas/initial_focus".into(),
                json!({"target":null}),
            );
        assert!(
            Document::parse(&crate::document::to_vec(&old).unwrap())
                .unwrap_err()
                .contains("version 4")
        );
    }
    #[test]
    fn native_blueprint_and_lua_share_inherited_widget_controls_through_place_refresh_and_cook() {
        let (root, registry, catalog) = fixture();
        let template = base_template(&root, &registry);
        assert_eq!(template.references.len(), 1);
        assert_eq!(template.references[0].member, LABEL);
        let document = Document::new(BASE.into(), template);
        create(&root, "assets/WidgetBase.epokclass", &document, &registry).unwrap();
        let lua = registry.named("LuaWidget").unwrap();
        let bp = registry.named("BP_Widget").unwrap();
        assert!(
            lua.source
                .file
                .canonicalize()
                .unwrap()
                .starts_with(root.join("assets").canonicalize().unwrap())
        );
        let model = registry.model().unwrap();
        let mut scene = Scene::default();
        scene.actors.clear();
        let mut owners = vec![];
        for class in [CPP, lua.id.as_str(), bp.id.as_str()] {
            for _ in 0..2 {
                let index = crate::editor::place_class(
                    &mut scene,
                    &root.canonicalize().unwrap(),
                    &registry,
                    &model,
                    class,
                )
                .unwrap();
                let owner = &scene.actors[index];
                owners.push((owner.id, owner.properties["label"].clone()));
                assert_eq!(owner.class.class_id.as_deref(), Some(class));
                assert_eq!(owner.components.len(), if class == CPP { 3 } else { 2 });
            }
        }
        assert_eq!(scene.actors.len(), 12);
        let component_ids = scene
            .actors
            .iter()
            .flat_map(|a| a.components.iter().map(|c| c.id))
            .collect::<BTreeSet<_>>();
        assert_eq!(component_ids.len(), 26);
        for (owner, label) in &owners {
            assert!(scene.actors.iter().any(|a| a.logical_parent == Some(*owner)
                && a.components.iter().any(|c| json!(c.id) == *label)));
        }
        let copy = scene.duplicate_actor_branch(owners[0].0).unwrap();
        let copy_index = scene.actors.iter().position(|a| a.id == copy).unwrap();
        let old = scene.actors[copy_index].clone();
        scene.actors[copy_index].name = "Renamed instance".into();
        templates::record_overrides(&old, &mut scene.actors[copy_index]);
        let before_ids = scene
            .actors
            .iter()
            .map(|a| (a.id, a.components.iter().map(|c| c.id).collect::<Vec<_>>()))
            .collect::<Vec<_>>();
        let mut source = load(&root).unwrap().sources.remove(BASE).unwrap();
        let mut changed = source.document.clone();
        changed.template.actors[1]
            .entity
            .text
            .as_mut()
            .unwrap()
            .text = "Updated base".into();
        source.revision = save(&root, &source, &changed, &registry).unwrap();
        let files = crate::blueprint_asset::load_all(&root).unwrap();
        refresh_instances(&root, &mut scene, &files, &registry).unwrap();
        assert_eq!(scene.actors[copy_index].name, "Renamed instance");
        assert!(
            scene
                .actors
                .iter()
                .filter_map(|a| a.text.as_ref())
                .all(|t| t.text == "Updated base")
        );
        assert_eq!(
            before_ids,
            scene
                .actors
                .iter()
                .map(|a| (a.id, a.components.iter().map(|c| c.id).collect::<Vec<_>>()))
                .collect::<Vec<_>>()
        );
        let map = root.join("assets/scenes/Widgets.epokmap");
        std::fs::write(&map, crate::document::to_vec(&scene).unwrap()).unwrap();
        let reopened = Scene::load(&map).unwrap();
        assert_eq!(reopened, scene);
        let index = crate::assets::scan(&root, &mut Default::default());
        let cooked = crate::blueprint_spawn::prepare(&root, &catalog, &index).unwrap();
        for class in [CPP, lua.id.as_str(), bp.id.as_str()] {
            let template = cooked.iter().find(|t| t.class == class).unwrap();
            assert_eq!(template.scene.actors.len(), 2);
            let root = template
                .scene
                .actors
                .iter()
                .find(|a| a.logical_parent.is_none())
                .unwrap();
            assert_eq!(root.class.class_id.as_deref(), Some(class));
            assert_eq!(
                template
                    .scene
                    .actors
                    .iter()
                    .find_map(|a| a.text.as_ref())
                    .unwrap()
                    .text,
                "Updated base"
            );
        }
        // A stale editor cannot overwrite an external class change.
        let stale = load(&root).unwrap().sources.remove(BASE).unwrap();
        std::fs::write(&stale.path, b"external write").unwrap();
        assert!(save(&root, &stale, &changed, &registry).is_err());
        assert_eq!(std::fs::read(&stale.path).unwrap(), b"external write");
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn class_documents_reject_duplicate_owners_sdk_edits_and_invalid_versions() {
        let (root, registry, _) = fixture();
        let document = Document::new(BASE.into(), base_template(&root, &registry));
        create(&root, "assets/Widget.epokclass", &document, &registry).unwrap();
        assert!(
            create(&root, "assets/Duplicate.epokclass", &document, &registry)
                .unwrap_err()
                .contains("already")
        );
        assert!(!root.join("assets/Duplicate.epokclass").exists());
        let mut sdk = document.clone();
        sdk.class = crate::object_model::UI_ACTOR_ID.into();
        assert!(
            create(&root, "assets/SDK.epokclass", &sdk, &registry)
                .unwrap_err()
                .contains("SDK")
        );
        let mut future = document.clone();
        future.version += 1;
        assert!(Document::parse(&crate::document::to_vec(&future).unwrap()).is_err());
        let first = load(&root).unwrap().signature(&root);
        observe_current(&root).unwrap();
        crate::artifact_dependencies::transaction(&root, |graph| {
            graph.publish(
                "test-widget-cook",
                "compiled".into(),
                BTreeSet::from([SOURCE_SET.into()]),
            )
        })
        .unwrap();
        let source = load(&root).unwrap().sources.remove(BASE).unwrap();
        let mut changed = document.clone();
        changed.template.actors[1].entity.name = "Renamed control".into();
        save(&root, &source, &changed, &registry).unwrap();
        assert_ne!(load(&root).unwrap().signature(&root), first);
        crate::scene_dependencies::observe_saved(&root).unwrap();
        assert!(
            !crate::artifact_dependencies::Graph::load(&root)
                .unwrap()
                .nodes["test-widget-cook"]
                .stale
                .is_empty()
        );
        std::fs::copy(&source.path, root.join("assets/Duplicate.epokclass")).unwrap();
        assert!(load(&root).err().unwrap().contains("more than one"));
        assert!(observe_current(&root).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn adding_a_native_singleton_adopts_the_authored_control_and_keeps_its_identity_and_values() {
        let (root, registry, _) = fixture();
        let model = registry.model().unwrap();
        let mut actor = actor(&model, BASE, "Widget").unwrap();
        let canvas = actor
            .components
            .iter_mut()
            .find(|c| c.class.name == "epok::CanvasComponent")
            .unwrap();
        let id = canvas.id;
        canvas.default_id = None;
        canvas.inherited = false;
        actor.canvas.as_mut().unwrap().enabled = false;
        hydrate(&mut actor, &model).unwrap();
        assert_eq!(actor.components.len(), 2);
        assert!(
            actor
                .components
                .iter()
                .any(|c| c.id == id && c.default_id.is_some() && c.inherited)
        );
        assert!(!actor.canvas.as_ref().unwrap().enabled);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn refreshing_an_unlinked_native_subclass_adopts_controls_before_cooking() {
        let (root, registry, catalog) = fixture();
        let model = registry.model().unwrap();
        let mut widget = actor(&model, BASE, "Authored widget").unwrap();
        let canvas = widget
            .components
            .iter_mut()
            .find(|component| component.class.name == "epok::CanvasComponent")
            .unwrap();
        let canvas_id = canvas.id;
        canvas.default_id = None;
        canvas.inherited = false;
        widget.canvas.as_mut().unwrap().enabled = false;
        widget.class = crate::actor_document::ClassReference::new("CppWidget", CPP);
        let mut scene = Scene {
            actors: vec![widget],
            ..Default::default()
        };
        scene.sync_actor_components();
        scene.bake = Some(crate::lighting::bake(&scene).unwrap());
        assert!(crate::lighting::valid_bake(&scene));
        let bake = scene.bake.clone();
        let path = root.join("assets/scenes/Unlinked.epokmap");
        scene.save(&path).unwrap();
        let authored = std::fs::read(&path).unwrap();
        refresh_instances(&root, &mut scene, &[], &registry).unwrap();
        let widget = &scene.actors[0];
        assert!(widget.blueprint_instance.is_none());
        assert_eq!(widget.components.len(), 3);
        assert!(
            widget
                .components
                .iter()
                .all(|c| c.inherited && c.default_id.is_some())
        );
        assert!(widget.components.iter().any(|c| c.id == canvas_id));
        assert!(!widget.canvas.as_ref().unwrap().enabled);
        assert_eq!(scene.bake, bake);
        let generated = crate::project::scene_header_with_registry(
            &scene, &catalog, &scene, false, &scene, &registry,
        )
        .unwrap();
        assert!(generated.contains("false,-1,0,1}"));
        assert!(generated.contains("false,-1,0,2}"));
        let refreshed = scene.clone();
        refresh_instances(&root, &mut scene, &[], &registry).unwrap();
        assert_eq!(scene, refreshed);
        assert_eq!(std::fs::read(&path).unwrap(), authored);
        std::fs::remove_dir_all(root).unwrap();
    }
}
