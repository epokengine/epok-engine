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
    let registry = crate::blueprint::legacy_registry(root, catalog);
    let mut result = vec![];
    for file in &files {
        let template =
            crate::blueprint_templates::resolve_assets(&files, &registry, &file.asset.id)?;
        if template.entities.is_empty() {
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
        let binding = crate::scene::ScriptBinding {
            name: class.cpp_name.clone(),
            class_id: Some(class.id.clone()),
            provider: class.provider.clone(),
            backend: class.backend.clone(),
            ..Default::default()
        };
        let mut scene = template.scene(binding, &registry)?;
        crate::mesh::resolve(&mut scene, index)?;
        crate::skeletal::resolve(&mut scene, index)?;
        crate::texture::resolve(&mut scene, index)?;
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
    let registry = crate::blueprint::legacy_registry(std::path::Path::new(""), catalog);
    header_with_templates(catalog, &[], false, &registry)
}
pub fn header_with_templates(
    catalog: &[Script],
    templates: &[CookedTemplate],
    timeline_metadata: bool,
    object_registry: &crate::blueprint::Registry,
) -> Result<String, String> {
    let visual_classes = catalog.iter().any(|s| {
        s.classes
            .first()
            .is_some_and(|c| c.provider.id == "blueprint")
    });
    if !visual_classes && !timeline_metadata {
        // The legacy Behaviour tables stay out of a project that has none, but
        // object_classes[]/object_class_count are declared extern by the runtime and
        // main.cpp links against them unconditionally. Emit the (possibly empty) table.
        return object_class_table_with_registry(object_registry);
    }
    let registry = crate::blueprint::legacy_registry(std::path::Path::new(""), catalog);
    let mut ids = BTreeMap::new();
    for class in registry.classes.values() {
        let id = crate::blueprint_refs::compact_id(&class.id);
        if id == 0 || ids.insert(id, &class.id).is_some() {
            return Err(format!("Runtime class ID collision: {}", class.id));
        }
    }
    if registry.classes.len() > 64 {
        return Err(
            "Blueprint runtime class catalog exceeds 64 classes; reduce cooked class dependencies."
                .into(),
        );
    }
    let mut output = String::from("#include \"blueprint_template.hpp\"\nnamespace epok::bp {\n");
    for template in templates {
        output.push_str(&format!(
            "bool configure_{}(EntityHandle);\n",
            crate::blueprint_refs::compact_id(&template.class)
        ));
    }
    output.push_str("inline const ClassInfo classes[] = {\n");
    let mut budgets = vec![];
    for class in registry.classes.values() {
        let id = crate::blueprint_refs::compact_id(&class.id);
        let parent = class
            .parent
            .as_ref()
            .map(|id| crate::blueprint_refs::compact_id(id))
            .unwrap_or(0);
        let factory = if visual_classes
            && !class.abstract_class
            && class.blueprintable
            && matches!(class.provider.id.as_str(), "cpp" | "blueprint")
        {
            budgets.push(format!("TypedPool<{},4>::storage_bytes", class.cpp_name));
            format!(
                "&TypedPool<{},4>::acquire,&TypedPool<{},4>::release",
                class.cpp_name, class.cpp_name
            )
        } else {
            "nullptr,nullptr".into()
        };
        let configure = if templates.iter().any(|template| template.class == class.id) {
            format!("&configure_{id}")
        } else {
            "nullptr".into()
        };
        output.push_str(&format!(
            "{{UINT64_C({id}),UINT64_C({parent}),{factory},{configure}}},\n"
        ));
    }
    if registry.classes.is_empty() {
        output.push_str("{},\n");
    }
    // Separate native translation units only see the extern declaration in
    // blueprint_spawn.hpp. Keep storage even when main folds this constant.
    output.push_str(&format!(
        "}};\n[[gnu::used]] inline const size_t class_count={};\n",
        registry.classes.len()
    ));
    // Bound compiled storage, not host sizeof/offset guesses. The linker also reports
    // the real MIPS data footprint for the complete game.
    if !budgets.is_empty() {
        output.push_str(&format!("static_assert(({}) <= 65536, \"Blueprint typed pools exceed the 64 KiB cook limit\");\n",budgets.join("+")));
    }
    output.push_str("}\n");
    output.push_str(&object_class_table_with_registry(object_registry)?);
    Ok(output)
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
    let registry = crate::blueprint::legacy_registry(std::path::Path::new(""), catalog);
    object_class_table_with_registry(&registry)
}

/// Emit the runtime object table from the same authoritative registry used to resolve
/// actor records. Real projects need SDK-native classes that intentionally stay out of
/// the project script catalog, including the default `epok::SceneScriptActor`.
pub fn object_class_table_with_registry(
    registry: &crate::blueprint::Registry,
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
        if class.family == crate::reflection_schema::ClassFamily::Behaviour {
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
        let storage = if class.instantiable() {
            budgets.push(format!("epok::ObjectPool<{name},4>::storage_bytes"));
            format!(
                "&epok::object_construct<{name}>,&epok::object_destruct,sizeof({name}),alignof({name}),&epok::ObjectPool<{name},4>::acquire,&epok::ObjectPool<{name},4>::release"
            )
        } else {
            "nullptr,nullptr,0,0,nullptr,nullptr".into()
        };
        rows.push((
            id,
            format!(
                "{{UINT64_C({id}),UINT64_C({parent}),epok::ObjectFamily::{family},epok::ObjectDomain::{domain},{owners},{flags},{storage}}},\n"
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
) -> Result<String, String> {
    let registry = crate::blueprint::legacy_registry(std::path::Path::new(""), catalog);
    let mut output = String::new();
    for template in templates {
        let id = crate::blueprint_refs::compact_id(&template.class);
        let scene = &template.scene;
        let mut components = scene.clone();
        for entity in &mut components.entities {
            entity.script = None;
        }
        let generated = crate::project::scene_header_body_with_layout(
            &components,
            catalog,
            resources,
            false,
            resources,
        )?;
        let body = generated
            .split("inline constexpr std::array<size_t,")
            .next()
            .ok_or("Missing template component boundary")?;
        let body = body
            .lines()
            .filter(|line| {
                !line.starts_with('#')
                    && !line.starts_with("lighting_environment=")
                    && !line.starts_with("fog_environment=")
            })
            .collect::<Vec<_>>()
            .join("\n")
            .replace(
                "namespace epok {",
                &format!("namespace epok::bp::prototype_{id} {{"),
            )
            .replace(
                &format!("std::array<Entity, {}>", scene.entities.len() + 32),
                &format!("std::array<Entity, {}>", scene.entities.len()),
            );
        output.push_str(&body);
        output.push_str("\n}\n");
        output.push_str(&format!("namespace epok::bp::prototype_{id} {{\n"));
        let mut bindings = vec![];
        for (i, entity) in scene.entities.iter().enumerate() {
            let Some(binding) = &entity.script else {
                continue;
            };
            let script = crate::script_backend::resolve(binding, catalog)?;
            let class = registry
                .bound(binding)
                .ok_or("Missing template binding class")?;
            if !script.instantiable()
                || !class.blueprintable
                || !matches!(class.provider.id.as_str(), "cpp" | "blueprint")
            {
                return Err(format!(
                    "{}: template child class {} has no concrete typed spawn factory",
                    entity.name, class.cpp_name
                ));
            }
            for key in binding.properties.keys() {
                if !script
                    .properties
                    .iter()
                    .any(|property| property.name == *key)
                {
                    return Err(format!(
                        "{}: orphaned template property {key} is preserved; migrate or explicitly reset it before cooking",
                        entity.name
                    ));
                }
            }
            output.push_str(&format!("inline void apply_{i}(Behaviour* value,const EntityHandle* handles,size_t){{auto* typed=static_cast<{}*>(value);\n",script.name));
            for property in &script.properties {
                let value = binding
                    .properties
                    .get(&property.name)
                    .unwrap_or(&property.default);
                let mut assignment = crate::blueprint_refs::assignment(
                    &format!("typed->{}", property.name),
                    value,
                    &property.value_type,
                    scene,
                    &registry,
                )?;
                for index in 0..scene.entities.len() {
                    assignment = assignment.replace(
                        &format!("epok::handle(&objects[{index}])"),
                        &format!("handles[{index}]"),
                    );
                }
                output.push_str(&assignment);
            }
            output.push_str("}\n");
            bindings.push(format!(
                "{{{i},UINT64_C({}),apply_{i}}}",
                crate::blueprint_refs::compact_id(&class.id)
            ));
        }
        output.push_str(&format!(
            "inline const TemplateBinding bindings[]={{{}}};\n",
            bindings.join(",")
        ));
        let has_timelines = scene
            .entities
            .iter()
            .any(|entity| entity.timeline.is_some() || entity.particle_effect.is_some());
        if has_timelines {
            output
                .push_str("inline bool configure_timelines(const EntityHandle* handles,size_t){\n");
            output.push_str(&crate::timeline_scene::setup_template(
                scene, timelines, &registry,
            )?);
            output.push_str(&crate::particle_effect_scene::setup(
                scene, effects, &registry, true,
            )?);
            output.push_str("return true;}\n");
        }
        output.push_str("}\n");
        let root = scene
            .entities
            .iter()
            .position(|entity| entity.parent.is_none())
            .ok_or("Missing template root")?;
        let configure = if has_timelines {
            format!(",prototype_{id}::configure_timelines")
        } else {
            String::new()
        };
        output.push_str(&format!("namespace epok::bp {{inline bool configure_{id}(EntityHandle root){{static bool initialized=false;if(!initialized){{prototype_{id}::initialize_components();initialized=true;}}return instantiate_template(root,prototype_{id}::objects.data(),prototype_{id}::objects.size(),{root},prototype_{id}::bindings,{}{configure});}}}}\n",bindings.len()));
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{object_model as om, reflection_schema as schema};

    #[test]
    fn native_only_games_do_not_emit_blueprint_factories() {
        // No Behaviour ClassInfo table, but the extern object_classes[] symbols the
        // runtime declares still exist so main.cpp links in every project.
        let text = header(&[]).unwrap();
        assert!(!text.contains("ClassInfo classes[]"));
        assert_eq!(text, empty_object_class_table());
        assert!(text.contains("object_class_count=0"));
    }

    fn class(id: &str, cpp_name: &str, parent: Option<&str>) -> schema::Class {
        schema::Class {
            family: None,
            domain: None,
            placement: Default::default(),
            component: None,
            default_components: vec![],
            explicit_abstract: false,
            id: id.into(),
            provider: schema::native_provider(),
            backend: schema::native_backend(),
            cpp_name: cpp_name.into(),
            parent: parent.map(str::to_owned),
            abstract_class: false,
            final_class: false,
            timeline_component: None,
            blueprintable: true,
            properties: vec![],
            functions: vec![],
            source: schema::Location {
                file: std::path::PathBuf::from("runtime/object_model.hpp"),
                line: 0,
                column: 0,
            },
        }
    }

    fn script(classes: Vec<schema::Class>) -> Script {
        Script {
            name: classes[0].cpp_name.clone(),
            parent: None,
            properties: vec![],
            header: std::path::PathBuf::from("object_model.hpp"),
            classes,
        }
    }

    /// The object class table is the cooked half of `runtime/object_model.hpp`: one
    /// `ClassDescriptor` per non-Behaviour class, pool-backed storage for the concrete
    /// ones, and a deterministic order so the emitted text is a stable build input.
    #[test]
    fn object_class_table_emits_descriptors_pools_and_a_bounded_budget() {
        let mut actor = class(om::ACTOR_ID, "epok::Actor", None);
        actor.family = Some(schema::ClassFamily::Actor);
        actor.abstract_class = true;
        let mut actor3d = class(om::ACTOR3D_ID, "epok::Actor3D", Some(om::ACTOR_ID));
        actor3d.domain = Some(schema::Domain::World3D);
        actor3d.placement = schema::Placement {
            placeable: true,
            spawnable: true,
            scene_managed: false,
        };
        let mut component = class(om::ACTOR_COMPONENT_ID, "epok::ActorComponent", None);
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
        let catalog = vec![script(vec![actor, actor3d, component, audio])];

        let text = object_class_table(&catalog).unwrap();
        assert!(
            text.starts_with(
                "namespace epok {\ninline const ClassDescriptor object_classes[] = {\n"
            )
        );
        assert!(text.contains("[[gnu::used]] inline const size_t object_class_count=4;"));

        let actor_id = crate::blueprint_refs::compact_id(om::ACTOR_ID);
        let actor3d_id = crate::blueprint_refs::compact_id(om::ACTOR3D_ID);
        let audio_id = crate::blueprint_refs::compact_id(om::AUDIO_COMPONENT_ID);
        // Abstract bases stay in the table so `object_class_is_a` can walk them, but
        // carry no storage at all.
        assert!(text.contains(&format!(
            "{{UINT64_C({actor_id}),UINT64_C(0),epok::ObjectFamily::Actor,epok::ObjectDomain::None,0,1,nullptr,nullptr,0,0,nullptr,nullptr}},"
        )));
        // Concrete actor: placeable|spawnable, pool-backed, placement-new pair exposed.
        assert!(text.contains(&format!(
            "{{UINT64_C({actor3d_id}),UINT64_C({actor_id}),epok::ObjectFamily::Actor,epok::ObjectDomain::World3D,0,6,&epok::object_construct<epok::Actor3D>,&epok::object_destruct,sizeof(epok::Actor3D),alignof(epok::Actor3D),&epok::ObjectPool<epok::Actor3D,4>::acquire,&epok::ObjectPool<epok::Actor3D,4>::release}},"
        )));
        // Component: owners mask World3D|World2D|UI = 7, Multiple cardinality = flag 32.
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
