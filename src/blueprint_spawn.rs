//! Cook collision-checked compact class identities and bounded typed factories.
use crate::scripts::Script;
use std::collections::BTreeMap;

pub struct CookedTemplate {
    pub class: String,
    pub scene: crate::scene::Scene,
}
pub fn prepare(
    root: &std::path::Path,
    catalog: &[Script],
    index: &crate::assets::Index,
) -> Result<Vec<CookedTemplate>, String> {
    let files = crate::blueprint_asset::load_all(root)?;
    let registry = crate::blueprint::registry_from_catalog(root, catalog);
    let mut result = vec![];
    for file in &files {
        let template =
            crate::blueprint_templates::resolve_assets(&files, &registry, &file.asset.id)?;
        if template.actors.is_empty() {
            continue;
        }
        crate::blueprint_templates::validate_resources(&template, index)?;
        let class = registry
            .classes
            .get(&file.asset.id)
            .ok_or("Missing template class")?;
        if class.abstract_class {
            continue;
        }
        let binding = crate::scene::ClassDefaults {
            name: class.cpp_name.clone(),
            class_id: Some(class.id.clone()),
            provider: class.provider.clone(),
            backend: class.backend.clone(),
            ..Default::default()
        };
        let mut scene = template.scene(binding, &registry)?;
        crate::mesh::resolve(&mut scene, index)?;
        crate::terrain::resolve(&mut scene, index)?;
        crate::skeletal::resolve(&mut scene, index)?;
        crate::texture::resolve(&mut scene, index)?;
        crate::hud::resolve_fonts(&mut scene, index)?;
        crate::audio::validate_assets(&scene, index)?;
        result.push(CookedTemplate {
            class: class.id.clone(),
            scene,
        });
    }
    Ok(result)
}

#[cfg(test)]
pub fn header(catalog: &[Script]) -> Result<String, String> {
    let registry = crate::blueprint::registry_from_catalog(std::path::Path::new(""), catalog);
    header_with_templates(catalog, &[], false, &registry)
}
pub fn header_with_templates(
    catalog: &[Script],
    templates: &[CookedTemplate],
    timeline_metadata: bool,
    object_registry: &crate::blueprint::Registry,
) -> Result<String, String> {
    let _ = (catalog, templates, timeline_metadata);
    object_class_table_with_registry(object_registry)
}

/// Cooked `epok::object_classes[]` table for the Object/Actor/Component runtime model.
///
/// Every class the registry resolves to a family other than the legacy `Behaviour` gets
/// one `ClassDescriptor`. Concrete classes also get storage: `ObjectPool<T,4>` supplies
/// `acquire`/`release`, `object_construct<T>`/`object_destruct` the placement-new pair
/// used for an actor's embedded default components. Abstract bases stay in the table so
/// `object_class_is_a` can walk them, but carry no factories.
///
/// Rows are sorted by compact id, so the emitted text only changes when the class set
/// or a class contract changes. The legacy Behaviour `classes[]` table is untouched.
#[cfg(test)]
pub fn object_class_table(catalog: &[Script]) -> Result<String, String> {
    let registry = crate::blueprint::registry_from_catalog(std::path::Path::new(""), catalog);
    object_class_table_with_registry(&registry)
}

/// Emit the runtime object table from the same authoritative registry used to resolve
/// actor records. Real projects need SDK-native classes that intentionally stay out of
/// the project script catalog, including the default `epok::SceneScriptActor`.
pub fn object_class_table_with_registry(
    registry: &crate::blueprint::Registry,
) -> Result<String, String> {
    object_class_table_with_capacities(registry, &BTreeMap::new())
}
pub fn object_class_table_for_scenes(
    registry: &crate::blueprint::Registry,
    scenes: &[crate::scene::Scene],
) -> Result<String, String> {
    object_class_table_for_scenes_and_templates(registry, scenes, &[])
}
pub fn object_class_table_for_scenes_and_templates(
    registry: &crate::blueprint::Registry,
    scenes: &[crate::scene::Scene],
    templates: &[CookedTemplate],
) -> Result<String, String> {
    let mut capacities = BTreeMap::<String, usize>::new();
    for scene in scenes {
        let mut counts = BTreeMap::<String, usize>::new();
        for actor in &scene.actors {
            if let Some(id) = &actor.class.class_id {
                *counts.entry(id.clone()).or_default() += 1;
            }
            for component in actor.components.iter().filter(|c| !c.root) {
                if let Some(id) = &component.class.class_id {
                    *counts.entry(id.clone()).or_default() += 1;
                }
            }
        }
        for (id, count) in counts {
            capacities
                .entry(id)
                .and_modify(|n| *n = (*n).max(count))
                .or_insert(count);
        }
    }
    let mut dynamic = BTreeMap::<String, usize>::new();
    for template in templates {
        let mut counts = BTreeMap::<String, usize>::new();
        for actor in &template.scene.actors {
            if let Some(id) = &actor.class.class_id {
                *counts.entry(id.clone()).or_default() += 1;
            }
            for component in actor
                .components
                .iter()
                .filter(|c| !c.root && c.default_id.is_none())
            {
                if let Some(id) = &component.class.class_id {
                    *counts.entry(id.clone()).or_default() += 1;
                }
            }
        }
        for (id, count) in counts {
            dynamic
                .entry(id)
                .and_modify(|n| *n = (*n).max(count))
                .or_insert(count);
        }
    }
    for (id, count) in dynamic {
        *capacities.entry(id).or_default() += 4 * count.saturating_sub(1);
    }
    object_class_table_with_capacities(registry, &capacities)
}
fn object_class_table_with_capacities(
    registry: &crate::blueprint::Registry,
    capacities: &BTreeMap<String, usize>,
) -> Result<String, String> {
    let model = match registry.model() {
        Ok(model) => model,
        // A project whose catalog declares no object-model class compiles exactly as it
        // did before; someone else's broken class graph must not fail its build.
        Err(_)
            if registry
                .classes
                .values()
                .all(|class| class.family.is_none() && class.domain.is_none()) =>
        {
            return Ok(empty_object_class_table());
        }
        Err(diagnostics) => {
            return Err(diagnostics
                .iter()
                .map(|d| format!("{}: {}", d.code, d.message))
                .collect::<Vec<_>>()
                .join("\n"));
        }
    };
    let mut rows: Vec<(u64, String)> = vec![];
    let mut budgets = vec![];
    for class in model.iter() {
        if !model.is_a(&class.id, crate::object_model::OBJECT_ID) {
            continue;
        }
        let id = crate::blueprint_refs::compact_id(&class.id);
        if id == 0 {
            return Err(format!("Runtime class ID collision: {}", class.id));
        }
        let parent = class
            .parent
            .as_ref()
            .filter(|parent| model.class(parent).is_some())
            .map(|parent| crate::blueprint_refs::compact_id(parent))
            .unwrap_or(0);
        let family = match class.family {
            crate::reflection_schema::ClassFamily::Actor => "Actor",
            crate::reflection_schema::ClassFamily::Component => "Component",
            crate::reflection_schema::ClassFamily::World => "World",
            crate::reflection_schema::ClassFamily::Level => "Level",
            _ => "Object",
        };
        let domain = match class.domain {
            crate::reflection_schema::Domain::World3D => "World3D",
            crate::reflection_schema::Domain::World2D => "World2D",
            crate::reflection_schema::Domain::UI => "UI",
            crate::reflection_schema::Domain::None => "None",
        };
        let owners = class
            .component
            .as_ref()
            .map(|contract| {
                contract
                    .owners
                    .iter()
                    .map(|domain| match domain {
                        crate::reflection_schema::Domain::World3D => 1u8,
                        crate::reflection_schema::Domain::World2D => 2,
                        crate::reflection_schema::Domain::UI => 4,
                        crate::reflection_schema::Domain::None => 0,
                    })
                    .fold(0u8, |mask, bit| mask | bit)
            })
            .unwrap_or(0);
        let mut flags = 0u8;
        if class.abstract_class {
            flags |= 1;
        }
        if class.placement.placeable {
            flags |= 2;
        }
        if class.placement.spawnable {
            flags |= 4;
        }
        if class.placement.scene_managed {
            flags |= 8;
        }
        if let Some(contract) = &class.component {
            if contract.can_root {
                flags |= 16;
            }
            if contract.cardinality == crate::reflection_schema::Cardinality::Multiple {
                flags |= 32;
            }
        }
        let name = &class.cpp_name;
        if !crate::scripts::class_identifier(name) {
            return Err(format!("Invalid reflected C++ class name {name}"));
        }
        let capacity = capacities.get(&class.id).copied().unwrap_or(0) + 4;
        let storage = if class.instantiable() {
            budgets.push(format!(
                "epok::ObjectPool<{name},{capacity}>::storage_bytes"
            ));
            format!(
                "&epok::object_construct<{name}>,&epok::object_destruct,sizeof({name}),alignof({name}),&epok::ObjectPool<{name},{capacity}>::acquire,&epok::ObjectPool<{name},{capacity}>::release"
            )
        } else {
            "nullptr,nullptr,0,0,nullptr,nullptr".into()
        };
        let defaults = if class.family == crate::reflection_schema::ClassFamily::Actor
            && !class.default_components.is_empty()
        {
            let mut cases = String::new();
            for (i, component) in class.default_components.iter().enumerate() {
                if !crate::scripts::identifier(&component.field) {
                    return Err("Invalid native component field".into());
                }
                let label =
                    serde_json::to_string(component.name.as_deref().unwrap_or(&component.field))
                        .unwrap();
                let parent = component
                    .attach_to
                    .as_ref()
                    .map(|field| {
                        class
                            .default_components
                            .iter()
                            .position(|c| &c.field == field)
                            .map(|i| i as i16)
                            .ok_or_else(|| {
                                format!(
                                    "{}: missing default component parent {field}",
                                    class.cpp_name
                                )
                            })
                    })
                    .transpose()?
                    .unwrap_or(-1);
                let component_class = model
                    .class(&component.class)
                    .ok_or_else(|| format!("Unknown native component class {}", component.class))?;
                let component_id = crate::blueprint_refs::compact_id(&component_class.id);
                cases += &format!(
                    "case {i}:return {{&self.{},{label},{},{parent},UINT64_C({component_id})}};",
                    component.field, component.root
                );
            }
            format!(
                ",{},+[](epok::Object& value,size_t index)->epok::NativeComponentDefault{{auto& self=static_cast<{name}&>(value);switch(index){{{cases}default:return {{}};}}}}",
                class.default_components.len()
            )
        } else {
            String::new()
        };
        let callbacks = if class.family == crate::reflection_schema::ClassFamily::Component {
            format!(
                ",0,nullptr,epok::ComponentCallbacks<{name}>::tick,epok::ComponentCallbacks<{name}>::frame"
            )
        } else if class.family == crate::reflection_schema::ClassFamily::Actor {
            let prefix = if defaults.is_empty() {
                ",0,nullptr"
            } else {
                ""
            };
            format!(
                "{prefix},epok::ActorCallbacks<{name}>::tick,epok::ActorCallbacks<{name}>::frame"
            )
        } else {
            String::new()
        };
        rows.push((
            id,
            format!(
                "{{UINT64_C({id}),UINT64_C({parent}),epok::ObjectFamily::{family},epok::ObjectDomain::{domain},{owners},{flags},{storage}{defaults}{callbacks}}},\n"
            ),
        ));
    }
    if rows.is_empty() {
        return Ok(empty_object_class_table());
    }
    rows.sort_by_key(|(id, _)| *id);
    let count = rows.len();
    let mut output =
        String::from("namespace epok {\ninline const ClassDescriptor object_classes[] = {\n");
    for (_, row) in &rows {
        output.push_str(row);
    }
    output.push_str(&format!(
        "}};\n[[gnu::used]] inline const size_t object_class_count={count};\n"
    ));
    if !budgets.is_empty() {
        budgets.sort();
        output.push_str(&format!(
            "static_assert(({}) <= 65536, \"Object pools exceed the 64 KiB cook limit\");\n",
            budgets.join("+")
        ));
    }
    output.push_str("}\n");
    Ok(output)
}

/// Placeholder table for a project with no Object/Actor/Component class. The runtime
/// declares `object_classes[]`/`object_class_count` extern, so the symbols must exist
/// even when nothing populates them; `find_object_class` then resolves nothing.
fn empty_object_class_table() -> String {
    "namespace epok {\ninline const ClassDescriptor object_classes[] = {\n{},\n};\n[[gnu::used]] inline const size_t object_class_count=0;\n}\n".into()
}

/// Reuse the ordinary component cooker, but never run scene startup or write
/// scene-global lighting/fog while preparing immutable spawn prototypes.
pub fn prototypes(
    catalog: &[Script],
    templates: &[CookedTemplate],
    resources: &crate::scene::Scene,
    timelines: &[crate::timeline_scene::Prepared],
    effects: &[crate::particle_effect_scene::Prepared],
    registry: &crate::blueprint::Registry,
) -> Result<String, String> {
    let mut output = String::new();
    let mut cases = String::new();
    for template in templates {
        let id = crate::blueprint_refs::compact_id(&template.class);
        let scene = &template.scene;
        let generated = crate::project::scene_header_with_registry(
            scene, catalog, resources, false, resources, registry,
        )?;
        let body = generated
            .lines()
            .filter(|line| !line.starts_with('#'))
            .collect::<Vec<_>>()
            .join("\n")
            .replace(
                "namespace epok {",
                &format!("namespace epok::prototype_{id} {{"),
            )
            .replace(
                &format!("std::array<ActorData, {}>", scene.actors.len() + 32),
                &format!("std::array<ActorData, {}>", scene.actors.len()),
            );
        output.push_str(&body);
        output.push_str(&format!("\nnamespace epok::prototype_{id} {{\n"));
        let has_services = scene
            .actors
            .iter()
            .any(|a| a.timeline.is_some() || a.particle_effect.is_some());
        if has_services {
            output.push_str("inline bool configure_services(const DataHandle* handles,size_t){\n");
            output.push_str(&crate::timeline_scene::setup_template(
                scene, timelines, registry,
            )?);
            output.push_str(&crate::particle_effect_scene::setup(
                scene, effects, registry, true,
            )?);
            output.push_str("return true;}\n");
        }
        let configure = if has_services {
            "&configure_services"
        } else {
            "nullptr"
        };
        output.push_str(&format!("inline const ActorPrototype prototype={{&actor_table,objects.data(),objects.size(),{configure}}};\n}}\n"));
        cases.push_str(&format!("if(id==UINT64_C({id})){{static bool initialized=false;if(!initialized){{prototype_{id}::initialize_components();initialized=true;}}return &prototype_{id}::prototype;}}\n"));
    }
    output.push_str(&format!("namespace epok {{inline const ActorPrototype* find_cooked_actor_template(uint64_t id){{(void)id;{cases}return nullptr;}}}}\n"));
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actor_document::tests::class;
    use crate::{object_model as om, reflection_schema as schema};
    fn script(classes: Vec<schema::Class>) -> Script {
        let class = &classes[0];
        Script {
            name: class.cpp_name.clone(),
            parent: None,
            properties: vec![],
            header: "Test.hpp".into(),
            classes,
        }
    }

    #[test]
    fn native_only_games_do_not_emit_blueprint_factories() {
        // No Behaviour ClassInfo table, but the extern object_classes[] symbols the
        // runtime declares still exist so main.cpp links in every project.
        let text = header(&[]).unwrap();
        assert!(!text.contains("ClassInfo classes[]"));
        assert_eq!(text, empty_object_class_table());
        assert!(text.contains("object_class_count=0"));
    }

    #[test]
    fn object_class_table_emits_descriptors_pools_and_a_bounded_budget() {
        let mut actor = class(om::ACTOR_ID, "epok::Actor", Some(om::OBJECT_ID));
        actor.family = Some(schema::ClassFamily::Actor);
        actor.abstract_class = true;
        let mut actor3d = class(om::ACTOR3D_ID, "epok::Actor3D", Some(om::ACTOR_ID));
        actor3d.domain = Some(schema::Domain::World3D);
        actor3d.placement = schema::Placement {
            placeable: true,
            spawnable: true,
            scene_managed: false,
        };
        let mut component = class(
            om::ACTOR_COMPONENT_ID,
            "epok::ActorComponent",
            Some(om::OBJECT_ID),
        );
        component.family = Some(schema::ClassFamily::Component);
        component.abstract_class = true;
        let mut audio = class(
            om::AUDIO_COMPONENT_ID,
            "epok::AudioComponent",
            Some(om::ACTOR_COMPONENT_ID),
        );
        audio.component = Some(schema::ComponentContract {
            owners: [
                schema::Domain::World3D,
                schema::Domain::World2D,
                schema::Domain::UI,
            ]
            .into_iter()
            .collect(),
            requires: vec![],
            excludes: vec![],
            cardinality: schema::Cardinality::Multiple,
            can_root: false,
            capabilities: ["audio".to_string()].into_iter().collect(),
        });
        let mut object = class(om::OBJECT_ID, "epok::Object", None);
        object.family = Some(schema::ClassFamily::Object);
        object.abstract_class = true;
        let catalog = vec![script(vec![actor, actor3d, component, audio, object])];

        let text = object_class_table(&catalog).unwrap();
        assert!(
            text.starts_with(
                "namespace epok {\ninline const ClassDescriptor object_classes[] = {\n"
            )
        );
        assert!(text.contains("[[gnu::used]] inline const size_t object_class_count=5;"));

        let object_id = crate::blueprint_refs::compact_id(om::OBJECT_ID);
        let actor_id = crate::blueprint_refs::compact_id(om::ACTOR_ID);
        let actor3d_id = crate::blueprint_refs::compact_id(om::ACTOR3D_ID);
        let audio_id = crate::blueprint_refs::compact_id(om::AUDIO_COMPONENT_ID);
        // Abstract bases stay in the table so `object_class_is_a` can walk them, but
        // carry no storage at all.
        assert!(text.contains(&format!(
            "{{UINT64_C({actor_id}),UINT64_C({object_id}),epok::ObjectFamily::Actor,epok::ObjectDomain::None,0,1,nullptr,nullptr,0,0,nullptr,nullptr,0,nullptr,epok::ActorCallbacks<epok::Actor>::tick,epok::ActorCallbacks<epok::Actor>::frame}},"
        )));
        // Concrete actor: placeable|spawnable, pool-backed, placement-new pair exposed.
        assert!(text.contains(&format!(
            "{{UINT64_C({actor3d_id}),UINT64_C({actor_id}),epok::ObjectFamily::Actor,epok::ObjectDomain::World3D,0,6,&epok::object_construct<epok::Actor3D>,&epok::object_destruct,sizeof(epok::Actor3D),alignof(epok::Actor3D),&epok::ObjectPool<epok::Actor3D,4>::acquire,&epok::ObjectPool<epok::Actor3D,4>::release,0,nullptr,epok::ActorCallbacks<epok::Actor3D>::tick,epok::ActorCallbacks<epok::Actor3D>::frame}},"
        )));
        // Component: owners mask World3D|World2D|UI = 7, Multiple cardinality = flag 32.
        assert!(text.contains("epok::ComponentCallbacks<epok::AudioComponent>::tick"));
        assert!(text.contains("epok::ComponentCallbacks<epok::AudioComponent>::frame"));
        assert!(text.contains(&format!(
            "{{UINT64_C({audio_id}),UINT64_C({}),epok::ObjectFamily::Component,epok::ObjectDomain::None,7,32,",
            crate::blueprint_refs::compact_id(om::ACTOR_COMPONENT_ID)
        )));
        assert!(text.contains(
            "static_assert((epok::ObjectPool<epok::Actor3D,4>::storage_bytes+epok::ObjectPool<epok::AudioComponent,4>::storage_bytes) <= 65536, \"Object pools exceed the 64 KiB cook limit\");"
        ));

        // Deterministic: rows sorted by compact id, so re-cooking is byte stable.
        assert_eq!(text, object_class_table(&catalog).unwrap());
        let rows: Vec<u64> = text
            .lines()
            .filter_map(|line| line.strip_prefix("{UINT64_C("))
            .filter_map(|line| line.split(')').next())
            .filter_map(|id| id.parse().ok())
            .collect();
        let mut sorted = rows.clone();
        sorted.sort_unstable();
        assert_eq!(rows, sorted);

        // A legacy Behaviour-only catalog emits the placeholder table: no descriptor
        // rows, but the symbols main.cpp links against are always defined.
        let empty = object_class_table(&[]).unwrap();
        assert_eq!(empty, empty_object_class_table());
        assert!(empty.contains("object_class_count=0") && !empty.contains("UINT64_C("));
    }
}
