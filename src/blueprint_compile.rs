//! Ahead-of-time native provider for Blueprint assets.
use crate::{
    blueprint::Registry,
    blueprint_asset::{self as asset, AssetFile},
    blueprint_ir::{self as ir, Statement},
    reflection_schema as schema,
    script_backend::Artifacts,
    scripts::{Property, Script},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub asset: PathBuf,
    pub graph: Option<String>,
    pub node: Option<String>,
    pub message: String,
}
impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.asset.display())?;
        if let Some(g) = &self.graph {
            write!(f, " [graph {g}]")?;
        }
        if let Some(n) = &self.node {
            write!(f, " [node {n}]")?;
        }
        write!(f, ": {}", self.message)
    }
}
pub struct Compilation {
    pub scripts: Vec<Script>,
    pub registry: Registry,
    pub artifacts: Artifacts,
    pub footprints: crate::blueprint_dependencies::Footprints,
    // P10 turns these two into the cooked Level data: one scene-script instance per
    // bank and the reference bindings the loader applies when it spawns it. They are
    // produced here because only the compiler knows the resolved class identities.
    /// `(map path, generated scene-script class id)` for every `.epokmap` whose
    /// embedded Blueprint was compiled, sorted by map path. P10 reads this to emit
    /// one `SceneScriptActor` instance per cooked scene bank.
    #[allow(dead_code)]
    pub scene_scripts: Vec<(PathBuf, String)>,
    /// Map-scoped references resolved by UUID and scope at compile time. The
    /// generated constructor writes the null identity; the Level loader binds these
    /// when it spawns the scene script (design.md section 6, step 3). Sorted.
    #[allow(dead_code)]
    pub scene_references: Vec<SceneReference>,
}
/// What a map-scoped reference names inside its owning map.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SceneRefKind {
    Actor,
    Component,
}
/// One persisted reference from a map's own Blueprint into that map.
///
/// The cook turns each of these into a row of the bank's `scene_reference` table
/// ([`map_scene_references`], `project::actor_table_body`), which `SceneLevel` binds to
/// live identities before the scene script's `begin_play`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SceneReference {
    /// The owning `.epokmap`.
    pub map: PathBuf,
    /// The generated scene-script class.
    pub class_id: String,
    /// `property:<name>` - the persisted member the Level loader must bind.
    pub member: String,
    pub kind: SceneRefKind,
    pub target: uuid::Uuid,
}
pub fn provider() -> schema::Extension {
    schema::Extension {
        id: "blueprint".into(),
        version: 1,
    }
}
/// Foreign calls are defined after every generated class is complete. This
/// allows A to call B and B to call a different method on A without header cycles.
fn receiver_artifacts(
    root: &Path,
    registry: &Registry,
    files: &[AssetFile],
    artifacts: &mut Artifacts,
) -> Result<bool, Vec<Diagnostic>> {
    let mut targets = BTreeMap::new();
    for file in files {
        for graph in &file.asset.functions {
            for node in graph.compilation_nodes() {
                if let asset::NodeKind::CallOn { class, function } = &node.kind {
                    let f = ir::call_on_function(registry, class, function).map_err(|e| {
                        vec![Diagnostic {
                            asset: file.path.clone(),
                            graph: Some(graph.id.clone()),
                            node: Some(node.id.clone()),
                            message: e,
                        }]
                    })?;
                    targets.insert((class.clone(), function.clone()), (file, f));
                }
            }
        }
    }
    if targets.is_empty() {
        return Ok(false);
    }
    let first = files.first().expect("receiver references require an asset");
    let mut header = String::from(
        "// Generated typed Blueprint receiver adapters.\n#pragma once\n#include \"epok.hpp\"\n",
    );
    let mut includes = BTreeSet::new();
    // Receiver declarations need the selected classes' native ancestry. SDK
    // effect-layer descriptors are not scene behaviours or project headers.
    for class in targets
        .keys()
        .flat_map(|(class, _)| registry.ancestry(&registry.classes[class].cpp_name))
        .filter(|c| c.provider.id == "cpp" && !c.cpp_name.starts_with("epok::"))
    {
        let script_root = root.join("assets/scripts");
        let script_root = std::fs::canonicalize(&script_root).unwrap_or(script_root);
        let source =
            std::fs::canonicalize(&class.source.file).unwrap_or_else(|_| class.source.file.clone());
        let relative = source.strip_prefix(&script_root).map_err(|_| {
            vec![diagnostic(
                first,
                "Receiver native header must be within assets/scripts",
            )]
        })?;
        let include = format!("../{}", relative.to_string_lossy().replace('\\', "/"));
        if include.contains(['"', '\n', '\r']) {
            return Err(vec![diagnostic(first, "Unsafe receiver include path")]);
        }
        includes.insert(include);
    }
    for include in includes {
        header.push_str(&format!("#include \"{include}\"\n"));
    }
    let mut source = String::from(
        "// Generated typed receiver calls; no RTTI or host member offsets.\n#include \"blueprint_calls.hpp\"\n#include \"blueprint_spawn.hpp\"\n",
    );
    for file in files {
        source.push_str(&format!("#include \"{}.hpp\"\n", file.asset.id));
    }
    source.push_str("namespace {uint32_t epok_receiver_depth=0;struct UqReceiverDepth {bool entered;UqReceiverDepth():entered(epok_receiver_depth<32){if(entered)++epok_receiver_depth;}~UqReceiverDepth(){if(entered)--epok_receiver_depth;}};}\n");
    for ((class_id, function_id), (file, function)) in targets {
        let class = &registry.classes[&class_id];
        let returns = ir::cpp_type(&function.returns).map_err(|e| vec![diagnostic(file, e)])?;
        let mut parameters = vec!["epok::ObjectId epok_receiver".to_string()];
        let mut arguments = vec![];
        for (index, p) in function.parameters.iter().enumerate() {
            let ty = ir::cpp_type(&p.value_type).map_err(|e| vec![diagnostic(file, e)])?;
            let ty = match p.direction {
                schema::Direction::Value => ty,
                schema::Direction::ConstReference => format!("const {ty}&"),
                schema::Direction::MutableReference => format!("{ty}&"),
            };
            parameters.push(format!("{ty} epok_argument_{index}"));
            arguments.push(format!("epok_argument_{index}"));
        }
        let signature = format!(
            "{returns} {}({})",
            ir::call_on_name(&class_id, &function_id),
            parameters.join(",")
        );
        header.push_str(&format!("{signature};\n"));
        let fallback = if function.returns == schema::Type::Void {
            "return;".to_string()
        } else {
            "return {};".to_string()
        };
        let class_hash = crate::blueprint_refs::compact_id(&class_id);
        let invoke = format!(
            "return static_cast<{}*>(epok_object)->{}({});",
            class.cpp_name,
            function.name,
            arguments.join(",")
        );
        source.push_str(&format!("{signature}{{UqReceiverDepth epok_depth;if(!epok_depth.entered){{{fallback}}}if(!epok::bp::is_a(epok_receiver,{class_hash}ULL)){{{fallback}}}auto* epok_object=epok::bp::object(epok_receiver);if(!epok_object){{{fallback}}}if(!epok::active_object_registry){{{fallback}}}epok::ObjectDispatchScope epok_scope(*epok::active_object_registry);{invoke}}}\n"));
    }
    artifacts.files.insert(
        "scripts/generated/blueprint_calls.hpp".into(),
        header.into_bytes(),
    );
    let path = PathBuf::from("scripts/generated/blueprint_calls.cpp");
    artifacts.files.insert(path.clone(), source.into_bytes());
    artifacts.native_sources.push(path);
    Ok(true)
}
fn diagnostic(file: &AssetFile, message: impl Into<String>) -> Diagnostic {
    Diagnostic {
        asset: file.path.clone(),
        graph: None,
        node: None,
        message: message.into(),
    }
}
/// The non-null UUID a typed reference value names, or `None` when the pin is not
/// a reference pin or carries the null identity.
fn reference_target(ty: &schema::Type, value: &serde_json::Value) -> Option<uuid::Uuid> {
    if !matches!(
        ty,
        schema::Type::ObjectRef { .. }
            | schema::Type::ActorRef { .. }
            | schema::Type::ComponentRef { .. }
    ) {
        return None;
    }
    value
        .as_str()
        .and_then(|text| uuid::Uuid::parse_str(text).ok())
        .filter(|id| !id.is_nil())
}
/// Resolves one persisted reference against the scope of its Blueprint.
///
/// Only a map's own Blueprint has a scope, and it is exactly the actors,
/// components and actors of that map. Resolution happens here, once, at compile
/// time: nothing looks an identity up by name during Tick. A standalone Blueprint
/// has no map, so any map-scoped value in one is a diagnostic rather than a
/// reference that would silently mean nothing at run time.
fn scene_reference(
    file: &AssetFile,
    class_id: &str,
    member: &str,
    ty: &schema::Type,
    value: &serde_json::Value,
) -> Result<Option<SceneReference>, Diagnostic> {
    let Some(target) = reference_target(ty, value) else {
        return Ok(None);
    };
    let Some(scope) = file.scope() else {
        return Err(diagnostic(
            file,
            format!(
                "{member} names {target}: a map-scoped {} is only available inside a map's scene Blueprint",
                ty.label()
            ),
        ));
    };
    let kind = match ty {
        schema::Type::ActorRef { .. } => scope
            .actors
            .contains(&target)
            .then_some(SceneRefKind::Actor),
        schema::Type::ComponentRef { .. } => scope
            .components
            .contains(&target)
            .then_some(SceneRefKind::Component),
        _ if scope.actors.contains(&target) => Some(SceneRefKind::Actor),
        _ if scope.components.contains(&target) => Some(SceneRefKind::Component),
        _ if scope.actors.contains(&target) => Some(SceneRefKind::Actor),
        _ => None,
    };
    let Some(kind) = kind else {
        return Err(diagnostic(
            file,
            format!(
                "{member} references {target}, which is not {} of {}; the value is preserved for repair",
                match ty {
                    schema::Type::ComponentRef { .. } => "a component",
                    _ => "an actor",
                },
                file.path.display()
            ),
        ));
    };
    Ok(Some(SceneReference {
        map: file.path.clone(),
        class_id: class_id.to_owned(),
        member: member.to_owned(),
        kind,
        target,
    }))
}
/// Map-scoped references of one map's own Blueprint, resolved by UUID and scope.
///
/// [`compile`] produces exactly these rows on [`Compilation::scene_references`]; the cook
/// calls this instead of threading that value through `scripts::catalog`, because the
/// only inputs are the map document and the asset embedded in it, and the resolution is
/// the same [`scene_reference`] function. A value that does not resolve is refused here
/// with the same message `compile` already raised for it.
pub fn map_scene_references(
    map: &Path,
    scene: &crate::scene::Scene,
) -> Result<Vec<SceneReference>, String> {
    let Some(file) = crate::blueprint_asset::embedded(map, scene) else {
        return Ok(vec![]);
    };
    let mut resolved = vec![];
    for variable in &file.asset.variables {
        if let Some(reference) = scene_reference(
            &file,
            &file.asset.id,
            &format!("property:{}", variable.name),
            &variable.value_type,
            &variable.default,
        )
        .map_err(|error| error.to_string())?
        {
            resolved.push(reference);
        }
    }
    resolved.sort();
    Ok(resolved)
}
fn location(path: &Path) -> schema::Location {
    schema::Location {
        file: path.into(),
        line: 1,
        column: 1,
    }
}
fn insert_function(
    functions: &mut BTreeMap<String, schema::Function>,
    function: &schema::Function,
) {
    let mut resolved = function.clone();
    for id in &function.overrides {
        if let Some(parent) = functions.get(id) {
            resolved.event |= parent.event;
            resolved.callable |= parent.callable;
            resolved.pure |= parent.pure;
            resolved.timeline = resolved.timeline.or(parent.timeline);
        }
    }
    for id in resolved
        .overrides
        .iter()
        .chain(std::iter::once(&resolved.id))
    {
        functions.insert(id.clone(), resolved.clone());
    }
}
#[derive(Default)]
struct ValidatedResources {
    paths: BTreeSet<PathBuf>,
    footprints: BTreeMap<String, BTreeMap<String, String>>,
}
fn validate_resources(
    root: &Path,
    registry: &Registry,
    files: &[AssetFile],
) -> Result<ValidatedResources, Vec<Diagnostic>> {
    let mut required = vec![];
    for file in files {
        for property in registry.properties(&file.asset.name) {
            if matches!(property.value_type, schema::Type::AssetRef { .. })
                && !property.default.is_null()
            {
                required.push((file, property.value_type.clone(), property.default.clone()));
            }
        }
        for graph in &file.asset.functions {
            for node in graph.compilation_nodes() {
                if let asset::NodeKind::Literal { value_type, value } = &node.kind
                    && matches!(value_type, schema::Type::AssetRef { .. })
                    && !value.is_null()
                {
                    required.push((file, value_type.clone(), value.clone()));
                }
                for input in node.inputs.values() {
                    if let asset::Input::Literal { value_type, value } = input
                        && matches!(value_type, schema::Type::AssetRef { .. })
                        && !value.is_null()
                    {
                        required.push((file, value_type.clone(), value.clone()));
                    }
                }
            }
        }
    }
    let templates = files
        .iter()
        .map(|file| {
            crate::blueprint_templates::resolve_assets(files, registry, &file.asset.id)
                .map(|template| (file, template))
                .map_err(|e| vec![diagnostic(file, e)])
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut dependencies = ValidatedResources::default();
    if required.is_empty()
        && templates
            .iter()
            .all(|(_, t)| crate::blueprint_templates::resource_ids(t).is_empty())
    {
        return Ok(dependencies);
    }
    let index = crate::assets::scan(root, &mut crate::assets::ScanCache::default());
    for (file, ty, value) in required {
        let id = value
            .as_str()
            .and_then(|id| uuid::Uuid::parse_str(id).ok())
            .ok_or_else(|| vec![diagnostic(file, "Asset reference requires a UUID")])?;
        let record = index.resolve(id).map_err(|e| vec![diagnostic(file, e)])?;
        if let schema::Type::AssetRef { kind } = ty {
            let expected = crate::assets::Kind::runtime_reference(&kind)
                .map_err(|e| vec![diagnostic(file, e)])?;
            if !expected.accepts_runtime(&record.meta.kind) {
                return Err(vec![diagnostic(
                    file,
                    format!("Asset {id} is {:?}, expected {kind}", record.meta.kind),
                )]);
            }
        }
        dependencies.paths.insert(record.path.clone());
        dependencies
            .footprints
            .entry(file.asset.id.clone())
            .or_default()
            .insert(
                format!("asset:{id}"),
                crate::assets::cache_key(&record.meta),
            );
    }
    for (file, template) in templates {
        crate::blueprint_templates::validate_resources(&template, &index)
            .map_err(|e| vec![diagnostic(file, e)])?;
        for id in crate::blueprint_templates::resource_ids(&template) {
            let record = index.resolve(id).map_err(|e| vec![diagnostic(file, e)])?;
            dependencies.paths.insert(record.path.clone());
            dependencies
                .footprints
                .entry(file.asset.id.clone())
                .or_default()
                .insert(
                    format!("asset:{id}"),
                    crate::assets::cache_key(&record.meta),
                );
        }
    }
    Ok(dependencies)
}
fn ordered<'a>(
    files: &'a [AssetFile],
    native: &Registry,
) -> Result<Vec<&'a AssetFile>, Vec<Diagnostic>> {
    let mut remaining = BTreeMap::new();
    let mut names = native
        .classes
        .values()
        .map(|c| c.cpp_name.to_ascii_lowercase())
        .collect::<BTreeSet<_>>();
    let mut errors = vec![];
    for file in files {
        let a = &file.asset;
        if a.version != asset::VERSION || uuid::Uuid::parse_str(&a.id).is_err() {
            errors.push(diagnostic(
                file,
                "Unsupported asset version or invalid class UUID",
            ));
        }
        if !crate::scripts::identifier(&a.name) || !names.insert(a.name.to_ascii_lowercase()) {
            errors.push(diagnostic(
                file,
                "Invalid or colliding Blueprint class name",
            ));
        }
        if native.classes.contains_key(&a.id) || remaining.insert(a.id.clone(), file).is_some() {
            errors.push(diagnostic(file, "Duplicate class UUID"));
        }
        if !a.extra.is_empty() {
            errors.push(diagnostic(file,"Unknown Blueprint fields are preserved but require a compatible compiler before cooking"));
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    let mut known = native.classes.keys().cloned().collect::<BTreeSet<_>>();
    let mut out = vec![];
    while !remaining.is_empty() {
        let ready = remaining
            .iter()
            .filter(|(_, f)| known.contains(&f.asset.parent))
            .map(|(id, _)| id.clone())
            .collect::<Vec<_>>();
        if ready.is_empty() {
            return Err(remaining
                .values()
                .map(|f| {
                    diagnostic(
                        f,
                        format!("Missing parent {} or inheritance cycle", f.asset.parent),
                    )
                })
                .collect());
        }
        for id in ready {
            let f = remaining.remove(&id).unwrap();
            known.insert(id);
            out.push(f);
        }
    }
    Ok(out)
}
fn declarations(file: &AssetFile, registry: &Registry) -> Result<schema::Class, Diagnostic> {
    let a = &file.asset;
    let parent = &registry.classes[&a.parent];
    // A Lua class is a real native subclass in every execution mode, so it is an
    // eligible Blueprint parent; `can_derive` is the single eligibility rule.
    if !crate::script_backend::can_derive(&crate::script_backend::blueprint_provider(), parent) {
        return Err(diagnostic(
            file,
            "Parent does not support native Blueprint inheritance",
        ));
    }
    let ancestry = registry.ancestry(&parent.cpp_name);
    let mut properties = BTreeMap::new();
    let mut functions = BTreeMap::new();
    let mut names = BTreeSet::new();
    let mut ids = BTreeSet::new();
    for c in ancestry {
        for p in &c.properties {
            properties.insert(p.id.clone(), p.clone());
            names.insert(p.name.clone());
            ids.insert(p.id.clone());
        }
        for f in &c.functions {
            insert_function(&mut functions, f);
            names.insert(f.name.clone());
            ids.insert(f.id.clone());
        }
    }
    let mut own = vec![];
    for (id, value) in &a.defaults {
        let mut p = properties
            .get(id)
            .ok_or_else(|| {
                diagnostic(
                    file,
                    format!("Default references missing property {id}; value preserved"),
                )
            })?
            .clone();
        if !p.editable || !crate::script_values::valid(value, &p.value_type) {
            return Err(diagnostic(
                file,
                format!("Default for {} is read-only or has the wrong type", p.name),
            ));
        }
        p.default = value.clone();
        own.push(p);
    }
    for v in &a.variables {
        if uuid::Uuid::parse_str(&v.id).is_err()
            || !ids.insert(v.id.clone())
            || !crate::scripts::identifier(&v.name)
            || v.name.starts_with("epok_")
            || !names.insert(v.name.clone())
        {
            return Err(diagnostic(
                file,
                "Variable UUID/name is invalid or shadows an inherited member",
            ));
        }
        if !crate::script_values::valid(&v.default, &v.value_type)
            || matches!(v.value_type,schema::Type::Vector{length} if length!=2&&length!=3)
        {
            return Err(diagnostic(
                file,
                format!("Variable {} has no valid bounded native default", v.name),
            ));
        }
        own.push(schema::Property {
            id: v.id.clone(),
            name: v.name.clone(),
            value_type: v.value_type.clone(),
            default: v.default.clone(),
            editable: v.editable,
            timeline: if v.timeline_animatable {
                schema::TimelineProperty::for_type(&v.value_type)
            } else {
                None
            },
            source: location(&file.path),
        });
        if matches!(
            v.value_type,
            schema::Type::SequenceHandle | schema::Type::EffectHandle
        ) && (v.editable || v.timeline_animatable)
        {
            return Err(diagnostic(
                file,
                format!(
                    "Variable {} is a transient playback handle; it cannot be editable or animatable",
                    v.name
                ),
            ));
        }
        if v.timeline_animatable
            && (!v.editable || schema::TimelineProperty::for_type(&v.value_type).is_none())
        {
            return Err(diagnostic(
                file,
                format!(
                    "Variable {} cannot expose this type/read-only state to timelines",
                    v.name
                ),
            ));
        }
    }
    if properties.len() + a.variables.len() > 16 {
        return Err(diagnostic(
            file,
            "Blueprint hierarchy exceeds the 16-property runtime budget",
        ));
    }
    let mut declared = vec![];
    let mut overrides = BTreeSet::new();
    let mut seen_overrides = BTreeSet::new();
    for graph in &a.functions {
        if uuid::Uuid::parse_str(&graph.id).is_err() || !ids.insert(graph.id.clone()) {
            return Err(diagnostic(file, "Function UUID is invalid or duplicated"));
        }
        let mut f = if let Some(id) = &graph.override_id {
            let inherited = functions
                .get(id)
                .ok_or_else(|| diagnostic(file, format!("Missing overridden function {id}")))?;
            if !inherited.event
                || inherited.final_method
                || inherited.access == "private"
                || !seen_overrides.insert(inherited.id.clone())
            {
                return Err(diagnostic(
                    file,
                    "Function is not an overridable event or is overridden twice",
                ));
            }
            if graph.name != inherited.name
                || graph.parameters.len() != inherited.parameters.len()
                || graph
                    .parameters
                    .iter()
                    .zip(&inherited.parameters)
                    .any(|(a, b)| a.value_type != b.value_type || a.direction != b.direction)
                || graph.returns != inherited.returns
            {
                return Err(diagnostic(
                    file,
                    format!(
                        "Override {} signature differs from its reflected parent",
                        graph.name
                    ),
                ));
            }
            if !graph.inherits_event() {
                overrides.insert(inherited.id.clone());
            }
            let mut f = inherited.clone();
            // Parameter names are local bindings, not part of a C++ override's
            // signature. Keep existing graph wires when SDK pin names improve.
            f.parameters = graph.parameters.clone();
            f.timeline = graph.timeline.or(f.timeline);
            f.overrides = vec![inherited.id.clone()];
            f
        } else {
            if !crate::scripts::identifier(&graph.name)
                || graph.name.starts_with("epok_")
                || matches!(
                    graph.name.as_str(),
                    "blueprint_tick"
                        | "blueprint_cancel"
                        | "blueprint_observe"
                        | "blueprint_class_id"
                )
                || !names.insert(graph.name.clone())
            {
                return Err(diagnostic(
                    file,
                    "Function name is invalid or shadows an inherited member; use Override",
                ));
            }
            schema::Function {
                id: graph.id.clone(),
                name: graph.name.clone(),
                parameters: graph.parameters.clone(),
                returns: graph.returns.clone(),
                callable: true,
                timeline: graph.timeline,
                event: true,
                pure: false,
                abstract_method: false,
                final_method: false,
                access: "public".into(),
                overrides: vec![],
                source: location(&file.path),
            }
        };
        if f.timeline.is_some() && f.returns != schema::Type::Void {
            return Err(diagnostic(file, "Timeline functions must return void"));
        }
        ir::cpp_type(&f.returns).map_err(|e| diagnostic(file, e))?;
        let mut parameter_names = BTreeSet::new();
        for p in &f.parameters {
            if !crate::scripts::identifier(&p.name)
                || p.name.starts_with("epok_")
                || !parameter_names.insert(&p.name)
                || p.value_type == schema::Type::Void
            {
                return Err(diagnostic(file, "Invalid or duplicate parameter"));
            }
            ir::cpp_type(&p.value_type).map_err(|e| diagnostic(file, e))?;
        }
        f.id = graph.id.clone();
        // Keep disconnected event declarations as aliases so existing children
        // retain their override UUIDs. They do not implement an abstract method.
        if !graph.inherits_event() {
            f.abstract_method = false;
        }
        f.source = location(&file.path);
        declared.push(f);
    }
    let abstract_class = functions
        .values()
        .any(|f| f.abstract_method && !overrides.contains(&f.id));
    Ok(schema::Class {
        family: None,
        domain: None,
        placement: Default::default(),
        component: a.component.clone(),
        default_components: vec![],
        explicit_abstract: false,
        id: a.id.clone(),
        provider: provider(),
        backend: schema::native_backend(),
        cpp_name: a.name.clone(),
        parent: Some(a.parent.clone()),
        abstract_class,
        final_class: false,
        timeline_component: None,
        blueprintable: true,
        properties: own,
        functions: declared,
        source: location(&file.path),
    })
}
fn write_body(
    body: &[Statement],
    text: &mut String,
    asset_id: &str,
    graph: &str,
    depth: usize,
    kind: ir::SelfKind,
) {
    for s in body {
        match s {
            Statement::Node(id) => {
                text.push_str(&format!(
                    "\n#line 1 \"Blueprint/{asset_id}/{graph}/{id}\"\n"
                ));
                text.push_str(&format!(
                    "epok::bp::trace({}u,{}u,this->id());\n",
                    trace_id(asset_id),
                    trace_id(id)
                ));
                text.push_str(&format!("#if defined(EPOK_BLUEPRINT_TRACE) && EPOK_BLUEPRINT_TRACE\nepok_debug({}u);\n#endif\n",trace_id(id)));
            }
            Statement::Delay(_) | Statement::Timeline { .. } | Statement::WaitPlayback { .. } => {
                unreachable!("latent bodies require continuation lowering")
            }
            Statement::StopTimeline(node) => {
                let key = node.replace('-', "");
                text.push_str(&format!("epok_track_{key}.cancel();++epok_epoch_{key};\n"));
            }
            Statement::Assign { target, value } => {
                if let schema::Type::Vector { length } = value.value_type {
                    text.push_str(&format!("{{ const auto& epok_value = {};", value.cpp));
                    for i in 0..length {
                        text.push_str(&format!("{target}[{i}]=epok_value[{i}];"));
                    }
                    text.push_str("}\n");
                } else {
                    text.push_str(&format!("{target} = {};\n", value.cpp));
                }
            }
            Statement::Evaluate(cpp) => text.push_str(&format!("{cpp};\n")),
            Statement::CheckOwner(ty) => text.push_str(&format!(
                "if(!epok_owner.get())return{};\n",
                if *ty == schema::Type::Void { "" } else { " {}" }
            )),
            Statement::Return(value) => text.push_str(&format!(
                "return{};\n",
                value
                    .as_ref()
                    .map(|v| format!(" {}", v.cpp))
                    .unwrap_or_default()
            )),
            Statement::Branch { condition, yes, no } => {
                text.push_str(&format!("if ({}) {{\n", condition.cpp));
                write_body(yes, text, asset_id, graph, depth + 1, kind);
                text.push_str("} else {\n");
                write_body(no, text, asset_id, graph, depth + 1, kind);
                text.push_str("}\n");
            }
            Statement::Loop { count, body } => {
                text.push_str(&format!("for (uint32_t epok_loop_{depth}=0;epok_loop_{depth}<{count}u;++epok_loop_{depth}) {{\n"));
                write_body(body, text, asset_id, graph, depth + 1, kind);
                text.push_str("}\n");
            }
        }
    }
}
fn trace_id(id: &str) -> u32 {
    use sha2::{Digest, Sha256};
    let hash = Sha256::digest(id.as_bytes());
    u32::from_le_bytes(hash[..4].try_into().unwrap())
}
fn self_kind_of(family: schema::ClassFamily) -> ir::SelfKind {
    match family {
        schema::ClassFamily::Actor => ir::SelfKind::Actor,
        schema::ClassFamily::Component => ir::SelfKind::Component,
        _ => unreachable!("Blueprint parent must be Actor or ActorComponent"),
    }
}

/// Owner prologue of a generated event body. Behaviour keeps the legacy single line.
/// Actor and Component resolve the legacy slot once and tolerate its absence: an
/// Actor2D or a UIActor has no 3D slot at all, so the handle is simply null there and
/// every `epok::bp::api` entry point already treats a null handle as a no-op.
fn self_prologue(_kind: ir::SelfKind) -> String {
    "const auto epok_owner=this->id();(void)epok_owner;".into()
}

/// Nodes that always lower to a continuation frame. Checked on the authored graph so the
/// class layout (tasks member, synthesized tick/end_play) is known before lowering.
fn expects_latent(asset: &asset::BlueprintAsset) -> bool {
    asset.functions.iter().any(|graph| {
        graph.compilation_nodes().any(|node| {
            matches!(
                node.kind,
                asset::NodeKind::Delay
                    | asset::NodeKind::WaitPlayback { .. }
                    | asset::NodeKind::Timeline { .. }
            )
        })
    })
}

fn debug_members(properties: &BTreeMap<String, schema::Property>, class_id: &str) -> String {
    let mut output = format!(
        "#if defined(EPOK_BLUEPRINT_TRACE) && EPOK_BLUEPRINT_TRACE\nvoid epok_debug(uint32_t node){{epok::bp::debug_begin({}u,node,this->id());\n",
        trace_id(class_id)
    );
    for property in properties.values() {
        let name = format!("this->{}", property.name);
        let (tag, values) = match &property.value_type {
            schema::Type::Bool => (1, vec![format!("uint32_t({name})")]),
            schema::Type::Int32 => (2, vec![format!("uint32_t({name})")]),
            schema::Type::UInt32 => (3, vec![name]),
            schema::Type::Fixed => (4, vec![format!("uint32_t({name}.raw())")]),
            schema::Type::Enum { .. } => (5, vec![format!("uint32_t({name})")]),
            schema::Type::Vector { length } if (2..=4).contains(length) => (
                match length {
                    2 => 6,
                    3 => 7,
                    _ => 11,
                },
                (0..*length)
                    .map(|i| format!("uint32_t({name}[{i}].raw())"))
                    .collect(),
            ),
            schema::Type::ObjectRef { .. } => (
                8,
                vec![
                    format!("uint32_t({name}.index)"),
                    format!("{name}.generation"),
                ],
            ),
            schema::Type::AssetRef { .. } => (
                9,
                vec![format!("uint32_t({name})"), format!("uint32_t({name}>>32)")],
            ),
            schema::Type::ClassRef { .. } => (
                10,
                vec![format!("uint32_t({name})"), format!("uint32_t({name}>>32)")],
            ),
            _ => continue,
        };
        output.push_str(&format!(
            "epok::bp::debug_value({}u,{tag}u,{}u,{});\n",
            trace_id(&property.id),
            values.len(),
            values.join(",")
        ));
    }
    output.push_str("epok_blueprint_debug_hook();}\n#endif\n");
    output
}
fn has_delay(body: &[Statement]) -> bool {
    body.iter().any(|s| match s {
        Statement::Delay(_) | Statement::Timeline { .. } | Statement::WaitPlayback { .. } => true,
        Statement::Branch { yes, no, .. } => has_delay(yes) || has_delay(no),
        Statement::Loop { body, .. } => has_delay(body),
        _ => false,
    })
}
fn cost(body: &[Statement]) -> u64 {
    body.iter()
        .map(|s| match s {
            Statement::Loop { count, body } => 1 + u64::from(*count) * cost(body),
            Statement::Branch { yes, no, .. } => 1 + cost(yes).max(cost(no)),
            Statement::Timeline {
                updated, finished, ..
            } => 1 + cost(updated) + cost(finished),
            Statement::WaitPlayback {
                reached,
                completed,
                cancelled,
                ..
            } => 1 + cost(reached).max(cost(completed)).max(cost(cancelled)),
            _ => 1,
        })
        .fold(0, u64::saturating_add)
}
/// Transform a structured function into a bounded continuation machine. The
/// machine retains loop counters, call results and value parameters per instance.
#[derive(Default)]
struct LatentCode {
    resume: String,
    observe: String,
    prepare: String,
    dispatch: String,
    cancel: String,
}
/// Lift each direct subscription into a captured frame scheduled by the existing
/// continuation compiler. The registering graph keeps executing its Next path;
/// a callback never borrows that graph's stack or overwrites its Delay state.
fn subscription_frames(function: &mut ir::FunctionIr) -> Vec<(String, ir::FunctionIr)> {
    let mut signature = function.signature.clone();
    signature.returns = schema::Type::Void;
    signature.parameters.extend(
        function
            .temporaries
            .iter()
            .map(|(name, ty)| schema::Parameter {
                name: name.clone(),
                value_type: ty.clone(),
                direction: schema::Direction::Value,
            }),
    );
    let arguments = signature
        .parameters
        .iter()
        .map(|p| p.name.as_str())
        .collect::<Vec<_>>()
        .join(",");
    fn lower(
        body: &mut [Statement],
        signature: &schema::Function,
        arguments: &str,
        frames: &mut BTreeMap<String, ir::FunctionIr>,
    ) {
        for statement in body {
            match statement {
                Statement::WaitPlayback {
                    node,
                    repeating: true,
                    ..
                } => {
                    let id = node.clone();
                    let name = format!("epok_subscribe_{}", id.replace('-', ""));
                    let mut listener = std::mem::replace(
                        statement,
                        Statement::Evaluate(format!(
                            "{name}({arguments});if(!epok_owner.get())return"
                        )),
                    );
                    if let Statement::WaitPlayback {
                        reached,
                        completed,
                        cancelled,
                        ..
                    } = &mut listener
                    {
                        lower(reached, signature, arguments, frames);
                        lower(completed, signature, arguments, frames);
                        lower(cancelled, signature, arguments, frames);
                    }
                    let mut signature = signature.clone();
                    signature.name = name;
                    frames.insert(
                        id,
                        ir::FunctionIr {
                            signature,
                            body: vec![listener],
                            temporaries: BTreeMap::new(),
                        },
                    );
                }
                Statement::WaitPlayback {
                    reached,
                    completed,
                    cancelled,
                    ..
                } => {
                    lower(reached, signature, arguments, frames);
                    lower(completed, signature, arguments, frames);
                    lower(cancelled, signature, arguments, frames);
                }
                Statement::Branch { yes, no, .. } => {
                    lower(yes, signature, arguments, frames);
                    lower(no, signature, arguments, frames);
                }
                Statement::Loop { body, .. } => lower(body, signature, arguments, frames),
                Statement::Timeline {
                    updated, finished, ..
                } => {
                    lower(updated, signature, arguments, frames);
                    lower(finished, signature, arguments, frames);
                }
                _ => {}
            }
        }
    }
    let mut frames = BTreeMap::new();
    lower(&mut function.body, &signature, &arguments, &mut frames);
    frames.into_iter().collect()
}
fn latent(
    file: &AssetFile,
    graph: &asset::Graph,
    frame_id: &str,
    function: &ir::FunctionIr,
    kind: ir::SelfKind,
    output: &mut String,
) -> Result<LatentCode, Diagnostic> {
    let key = frame_id.replace('-', "");
    let selfh = kind.handle();
    let token = trace_id(frame_id);
    let mut fields = String::new();
    let mut aliases = String::new();
    let mut capture = String::new();
    for p in &function.signature.parameters {
        if let schema::Type::Record { cpp_name, .. } = &p.value_type
            && cpp_name == "epok::Transform"
            && p.direction != schema::Direction::Value
        {
            // Dereferencing the legacy slot is only valid once it exists. Actor and
            // Component frames skip the resumption instead of aliasing a placeholder.
            {
                aliases.push_str(&format!("if(!{})return;\n", kind.entity_pointer()));
            }
            aliases.push_str(&format!("auto& {}={};\n", p.name, kind.transform()));
            continue;
        }
        if p.direction == schema::Direction::MutableReference {
            return Err(diagnostic(
                file,
                format!(
                    "Latent graph {} cannot retain mutable reference {}; use instance variables or an owner Transform parameter",
                    graph.name, p.name
                ),
            ));
        }
        let ty = ir::cpp_type(&p.value_type).map_err(|e| diagnostic(file, e))?;
        fields.push_str(&format!("{ty} epok_arg_{key}_{}{{}};\n", p.name));
        capture.push_str(&format!("epok_arg_{key}_{}={};\n", p.name, p.name));
        aliases.push_str(&format!(
            "{}auto& {}=epok_arg_{key}_{};\n(void){};\n",
            if p.direction == schema::Direction::ConstReference {
                "const "
            } else {
                ""
            },
            p.name,
            p.name,
            p.name
        ));
    }
    for (name, ty) in &function.temporaries {
        fields.push_str(&format!(
            "{} {name}{{}};\n",
            ir::cpp_type(ty).map_err(|e| diagnostic(file, e))?
        ));
    }
    struct Machine<'a> {
        cases: Vec<String>,
        loops: usize,
        key: &'a str,
        selfh: &'a str,
        kind: ir::SelfKind,
        asset: &'a str,
        graph: &'a str,
        token: u32,
        tracks: Vec<(String, String, usize, usize)>,
        playback: bool,
        subscriptions: Vec<(String, usize)>,
    }
    impl Machine<'_> {
        fn push(&mut self, code: String) -> usize {
            let index = self.cases.len();
            self.cases.push(code);
            index
        }
        fn lower(&mut self, body: &[Statement], mut next: usize) -> usize {
            for statement in body.iter().rev() {
                next=match statement {
                    Statement::Return(_)=>0,
                    Statement::Delay(seconds)=>self.push(format!("epok_pc_{}={next}u;epok_tasks.delay({}u,{},{},epok::blueprint_scene_generation);return;",self.key,self.token,seconds.cpp,self.selfh)),
                    Statement::WaitPlayback {node, repeating:true, target, asset, marker, reached, completed, cancelled} => {
                        let field=format!("epok_subscription_{}",node.replace('-',""));
                        let dispatch=self.push(String::new());
                        let cancel_body=self.lower(cancelled,next);
                        let cancelled=self.push(format!("{field}.active=false;epok_pc={cancel_body}u;break;"));
                        let complete_body=self.lower(completed,next);
                        let completed=self.push(format!("{field}.active=false;epok_pc={complete_body}u;break;"));
                        let schedule=format!("epok_pc_{}={dispatch}u;if(!epok_tasks.wait_external({}u,{},epok::blueprint_scene_generation)){{epok_pc={cancelled}u;break;}}if({field}.capture(epok::bp::api::playback_snapshot({field})))epok_tasks.signal({}u,epok::blueprint_scene_generation);return;",self.key,self.token,self.selfh,self.token);
                        let rearm=self.push(format!("{field}.rearm();{schedule}"));
                        let reached=self.lower(reached,rearm);
                        self.cases[dispatch]=format!("epok_pc={field}.result==epok::bp::PlaybackResult::Reached?{reached}u:{field}.result==epok::bp::PlaybackResult::Completed?{completed}u:{cancelled}u;break;");
                        self.subscriptions.push((field.clone(),dispatch));
                        self.push(format!("{field}.begin({},{asset}ULL,{marker}ULL);{field}.start(epok::bp::api::playback_snapshot({field}));{schedule}",target.cpp))
                    },
                    Statement::WaitPlayback {target, asset, marker, reached, completed, cancelled, ..} => {
                        self.playback=true;
                        let reached=self.lower(reached,next);
                        let completed=self.lower(completed,next);
                        let cancelled=self.lower(cancelled,next);
                        let field=format!("epok_wait_{}",self.key);
                        let dispatch=self.push(format!("epok_pc={field}.result==epok::bp::PlaybackResult::Reached?{reached}u:{field}.result==epok::bp::PlaybackResult::Completed?{completed}u:{cancelled}u;break;"));
                        let arguments=if target.value_type==schema::Type::EffectHandle {target.cpp.clone()} else {format!("{},{asset}ULL,{marker}ULL",target.cpp)};
                        self.push(format!("{field}.begin({arguments});epok_pc_{}={dispatch}u;if(!epok_tasks.wait_external({}u,{},epok::blueprint_scene_generation)){{epok_pc={cancelled}u;break;}}if({field}.capture(epok::bp::api::playback_snapshot({field})))epok_tasks.signal({}u,epok::blueprint_scene_generation);return;",self.key,self.token,self.selfh,self.token))
                    },
                    Statement::Branch{condition,yes,no}=>{let yes=self.lower(yes,next);let no=self.lower(no,next);self.push(format!("epok_pc=({})?{yes}u:{no}u;break;",condition.cpp))},
                    Statement::Loop{count,body}=>{let index=self.loops;self.loops+=1;let field=format!("epok_loop_{}_{index}",self.key);let condition=self.push(String::new());let increment=self.push(format!("++{field};epok_pc={condition}u;break;"));let body=self.lower(body,increment);self.cases[condition]=format!("epok_pc=({field}<{count}u)?{body}u:{next}u;break;");self.push(format!("{field}=0;epok_pc={condition}u;break;"))},
                    Statement::Timeline{node,target,keys,looping,updated,finished}=>{let track=node.replace('-',"");let updated=self.lower(updated,0);let finished=self.lower(finished,0);if !self.tracks.iter().any(|(id,_,_,_)|*id==track){self.tracks.push((track.clone(),target.clone(),updated,finished));}let data=keys.iter().map(|(time,value)|format!("{{{time},{value}}}")).collect::<Vec<_>>().join(",");self.push(format!("{{const epok::bp::TimelineKey epok_keys[]={{{data}}};++epok_epoch_{track};epok_track_{track}.configure(epok_keys,{});epok_track_{track}.play({},epok::blueprint_scene_generation,{looping});}}epok_pc={next}u;break;",keys.len(),self.selfh))},
                    _=>{let mut code=String::new();write_body(std::slice::from_ref(statement),&mut code,self.asset,self.graph,0,self.kind);code.push_str(&format!("epok_pc={next}u;break;"));self.push(code)}
                }
            }
            next
        }
    }
    let mut machine = Machine {
        cases: vec!["return;".into()],
        loops: 0,
        key: &key,
        selfh,
        kind,
        asset: &file.asset.id,
        graph: &graph.id,
        token,
        tracks: vec![],
        playback: false,
        subscriptions: vec![],
    };
    let entry = machine.lower(&function.body, 0);
    fields.push_str(&format!("uint32_t epok_pc_{key}=0;\n"));
    fields.push_str(&format!("uint32_t epok_frame_epoch_{key}=0;\n"));
    for index in 0..machine.loops {
        fields.push_str(&format!("uint32_t epok_loop_{key}_{index}=0;\n"));
    }
    let mut code = LatentCode {
        resume: format!("case {token}u:epok_run_{key}(epok_pc_{key});break;"),
        cancel: format!("++epok_frame_epoch_{key};\n"),
        ..Default::default()
    };
    if machine.playback {
        fields.push_str(&format!("epok::bp::PlaybackWait epok_wait_{key};\n"));
        code.observe = format!(
            "if(epok_tasks.waiting({token}u)&&epok_wait_{key}.capture(epok::bp::api::playback_snapshot(epok_wait_{key})))epok_tasks.signal({token}u,epok::blueprint_scene_generation);\n"
        );
    }
    let mut subscription_fields = BTreeSet::new();
    for (field, dispatch) in &machine.subscriptions {
        if subscription_fields.insert(field.clone()) {
            fields.push_str(&format!("epok::bp::PlaybackSubscription {field};\n"));
            code.cancel.push_str(&format!("{field}.active=false;\n"));
        }
        code.observe.push_str(&format!("if({field}.active&&{field}.capture(epok::bp::api::playback_snapshot({field}))&&epok_pc_{key}=={dispatch}u)epok_tasks.signal({token}u,epok::blueprint_scene_generation);\n"));
    }
    // Returning from a reached branch ends its subscriptions, including any
    // outer subscription whose branch contained a nested latent operation.
    machine.cases[0] = format!(
        "{}return;",
        subscription_fields
            .iter()
            .map(|field| format!("{field}.active=false;"))
            .collect::<String>()
    );
    for (track, target, updated, finished) in &machine.tracks {
        fields.push_str(&format!(
            "epok::bp::Timeline<16> epok_track_{track};uint32_t epok_epoch_{track}=0;\n"
        ));
        code.prepare.push_str(&format!("const auto epok_epoch_snapshot_{track}=epok_epoch_{track};const auto epok_sample_{track}=epok_track_{track}.advance(dt,epok::blueprint_scene_generation);\n"));
        code.dispatch.push_str(&format!("if(epok_epoch_snapshot_{track}==epok_epoch_{track}&&epok_sample_{track}.updated){{{target}=epok_sample_{track}.value;epok_run_{key}({updated}u);{guard}}}if(epok_epoch_snapshot_{track}==epok_epoch_{track}&&epok_sample_{track}.completed){{epok_run_{key}({finished}u);{guard}}}\n",guard="if(!epok_owner.get())return;"));
        code.cancel.push_str(&format!(
            "epok_track_{track}.cancel();++epok_epoch_{track};\n"
        ));
    }
    output.push_str(&format!(
        "epok_tasks.cancel({token}u);\n{}{capture}epok_run_{key}({entry}u);\n}}\n",
        code.cancel
    ));
    output.push_str(&fields);
    output.push_str(&format!("void epok_run_{key}(uint32_t epok_pc) {{\n{}const auto epok_frame_epoch=epok_frame_epoch_{key};\n{aliases}uint32_t epok_budget=65536u;while(epok_budget--&&epok_frame_epoch==epok_frame_epoch_{key}){{switch(epok_pc){{\n",self_prologue(kind)));
    for (index, code) in machine.cases.iter().enumerate() {
        output.push_str(&format!("case {index}u:{{{code}}}\n"));
    }
    output.push_str("default:return;}}}\n");
    Ok(code)
}
fn declaration_order<'a>(
    native: &Registry,
    files: &'a [AssetFile],
) -> Result<Vec<&'a AssetFile>, Vec<Diagnostic>> {
    let order = ordered(files, native)?;
    let mut identities = native
        .classes
        .values()
        .flat_map(|c| {
            std::iter::once(c.id.clone())
                .chain(c.properties.iter().map(|p| p.id.clone()))
                .chain(c.functions.iter().map(|f| f.id.clone()))
        })
        .collect::<BTreeSet<_>>();
    for file in files {
        for id in std::iter::once(&file.asset.id)
            .chain(file.asset.variables.iter().map(|v| &v.id))
            .chain(
                file.asset
                    .functions
                    .iter()
                    .flat_map(|g| std::iter::once(&g.id).chain(g.nodes.iter().map(|n| &n.id))),
            )
        {
            if !uuid::Uuid::parse_str(id).is_ok_and(|u| !u.is_nil() && u.to_string() == *id)
                || !identities.insert(id.clone())
            {
                return Err(vec![diagnostic(
                    file,
                    format!("Duplicate, nil, or noncanonical persistent UUID {id}"),
                )]);
            }
        }
    }
    Ok(order)
}

fn register_declarations(
    native: &Registry,
    order: &[&AssetFile],
) -> Result<Registry, Vec<Diagnostic>> {
    let mut registry = native.clone();
    for file in order {
        let class = declarations(file, &registry).map_err(|e| vec![e])?;
        registry.classes.insert(class.id.clone(), class);
    }
    Ok(registry)
}

/// Resolve current declarations through the compiler's normal identity,
/// inheritance and default validation, without compiling graphs or resources.
/// Observers use this registry only to select source dependencies; it never
/// certifies executable code or replaces a failed compilation.
pub fn declaration_registry(
    native: &Registry,
    files: &[AssetFile],
) -> Result<Registry, Vec<Diagnostic>> {
    register_declarations(native, &declaration_order(native, files)?)
}

pub fn compile(
    root: &Path,
    native: &Registry,
    files: &[AssetFile],
) -> Result<Compilation, Vec<Diagnostic>> {
    let order = declaration_order(native, files)?;
    let calls = files
        .iter()
        .flat_map(|file| {
            file.asset.functions.iter().map(|g| {
                (
                    &g.id,
                    g.compilation_nodes()
                        .filter_map(|n| {
                            if let asset::NodeKind::Call { function }
                            | asset::NodeKind::CallOn { function, .. } = &n.kind
                            {
                                Some(function.clone())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>(),
                )
            })
        })
        .collect::<BTreeMap<_, _>>();
    fn recursive(
        id: &String,
        calls: &BTreeMap<&String, Vec<String>>,
        active: &mut BTreeSet<String>,
        done: &mut BTreeSet<String>,
    ) -> bool {
        if done.contains(id) {
            return false;
        }
        if !active.insert(id.clone()) {
            return true;
        }
        if calls.get(id).is_some_and(|targets| {
            targets
                .iter()
                .any(|target| calls.contains_key(target) && recursive(target, calls, active, done))
        }) {
            return true;
        }
        active.remove(id);
        done.insert(id.clone());
        false
    }
    for file in files {
        for graph in &file.asset.functions {
            if recursive(
                &graph.id,
                &calls,
                &mut BTreeSet::new(),
                &mut BTreeSet::new(),
            ) {
                return Err(vec![diagnostic(
                    file,
                    "Recursive Blueprint calls exceed the bounded native execution profile; use an explicit loop or deferred event",
                )]);
            }
        }
    }
    let mut trace_ids = BTreeMap::new();
    for file in files {
        for id in std::iter::once(&file.asset.id).chain(
            file.asset
                .functions
                .iter()
                .flat_map(|g| std::iter::once(&g.id).chain(g.nodes.iter().map(|n| &n.id))),
        ) {
            if let Some(previous) = trace_ids.insert(trace_id(id), id.clone())
                && previous != *id
            {
                return Err(vec![diagnostic(
                    file,
                    "Compact debug/continuation ID collision; regenerate the conflicting UUID explicitly",
                )]);
            }
        }
    }
    let mut registry = register_declarations(native, &order)?;
    let resource_dependencies = validate_resources(root, &registry, files)?;
    let mut scripts = vec![];
    let mut artifacts = Artifacts::default();
    let (direct_timelines, direct_effects) =
        crate::blueprint_playback::references(files).map_err(|message| {
            vec![diagnostic(
                files.first().expect("Reference errors require a node"),
                message,
            )]
        })?;
    let playback_timelines = if direct_timelines.is_empty() {
        vec![]
    } else {
        crate::timeline::load_referenced(root, &direct_timelines, false)
            .map_err(|message| vec![diagnostic(&files[0], message)])?
    };
    let playback_effects = if direct_effects.is_empty() {
        vec![]
    } else {
        crate::particle_effect::load_referenced(root, &direct_effects)
            .map_err(|message| vec![diagnostic(&files[0], message)])?
    };
    let mut playback_declarations =
        String::from("#pragma once\n#include \"epok.hpp\"\nnamespace epok::bp::playback {\n");
    for (ids, effect) in [(&direct_timelines, false), (&direct_effects, true)] {
        for id in ids {
            let source = if effect {
                playback_effects
                    .iter()
                    .find(|(_, asset)| asset.id == *id)
                    .map(|(path, asset)| (path, &asset.timeline))
            } else {
                playback_timelines
                    .iter()
                    .find(|(_, asset)| asset.id == *id)
                    .map(|(path, asset)| (path, asset))
            };
            let (path, source) = source.ok_or_else(|| {
                vec![diagnostic(
                    &files[0],
                    format!(
                        "Missing {} {id}",
                        if effect {
                            "ParticleEffect"
                        } else {
                            "TimelineAsset"
                        }
                    ),
                )]
            })?;
            let errors = source.validate(&registry);
            if !errors.is_empty() {
                return Err(vec![diagnostic(
                    &files[0],
                    errors
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join("\n"),
                )]);
            }
            if !effect
                && source
                    .slots
                    .iter()
                    .any(|slot| matches!(slot.target, schema::Type::EffectLayerRef { .. }))
            {
                return Err(vec![diagnostic(
                    &files[0],
                    "Standalone TimelineAsset playback cannot bind internal effect layers; play the owning ParticleEffect",
                )]);
            }
            artifacts.dependencies.insert(path.clone());
            playback_declarations +=
                &crate::blueprint_playback::declaration(*id, &source.slots, effect);
        }
    }
    let has_playback_calls = !direct_timelines.is_empty() || !direct_effects.is_empty();
    if has_playback_calls {
        playback_declarations += "}\n";
        artifacts.files.insert(
            "scripts/generated/playback_calls.hpp".into(),
            playback_declarations.into_bytes(),
        );
    }
    let marker_nodes: Vec<_> = files
        .iter()
        .flat_map(|file| {
            file.asset.functions.iter().flat_map(move |graph| {
                graph.compilation_nodes().filter_map(move |node| {
                    if let asset::NodeKind::WaitPlayback {
                        condition:
                            asset::PlaybackCondition::Marker { timeline, marker }
                            | asset::PlaybackCondition::SubscribeMarker { timeline, marker },
                    } = &node.kind
                    {
                        Some((file, graph, node, timeline, marker))
                    } else {
                        None
                    }
                })
            })
        })
        .collect();
    let mut dependency_timelines = playback_timelines.clone();
    dependency_timelines.extend(
        playback_effects
            .iter()
            .map(|(path, effect)| (path.clone(), effect.timeline.clone())),
    );
    if !marker_nodes.is_empty() {
        let required = marker_nodes
            .iter()
            .filter_map(|(_, _, _, timeline, _)| uuid::Uuid::parse_str(timeline).ok())
            .collect();
        let timelines = crate::timeline::load_referenced(root, &required, true)
            .map_err(|message| vec![diagnostic(marker_nodes[0].0, message)])?;
        for (path, source) in &timelines {
            if let Some((_, old)) = dependency_timelines
                .iter()
                .find(|(_, old)| old.id == source.id)
            {
                if old.semantic_hash() != source.semantic_hash() {
                    return Err(vec![diagnostic(
                        marker_nodes[0].0,
                        "Playback source changed during compilation; retry against the current revision",
                    )]);
                }
            } else {
                dependency_timelines.push((path.clone(), source.clone()));
            }
        }
        for (file, graph, node, timeline, marker) in marker_nodes {
            let found = timelines
                .iter()
                .find(|(_, asset)| asset.id.to_string() == *timeline);
            if let Some((path, source)) =
                found.filter(|(_, asset)| asset.markers.iter().any(|m| m.id.to_string() == *marker))
            {
                artifacts.dependencies.insert(path.clone());
                // The checked source, not a stale compiled timeline, is authoritative.
                let _ = source;
            } else {
                return Err(vec![Diagnostic {
                    asset: file.path.clone(),
                    graph: Some(graph.id.clone()),
                    node: Some(node.id.clone()),
                    message: format!(
                        "Missing timeline/marker reference {timeline}/{marker}; original node preserved"
                    ),
                }]);
            }
        }
    }
    artifacts.dependencies.extend(resource_dependencies.paths);
    let model = Some(registry.model().map_err(|diagnostics| {
        diagnostics
            .iter()
            .map(|d| diagnostic(&files[0], format!("{}: {}", d.code, d.message)))
            .collect::<Vec<_>>()
    })?);
    let has_receivers = receiver_artifacts(root, &registry, files, &mut artifacts)?;
    let mut scene_scripts = vec![];
    let mut scene_references = vec![];
    for file in order {
        let a = &file.asset;
        let mut class = registry.classes[&a.id].clone();
        for property in &class.properties {
            match &property.value_type {
                schema::Type::ClassRef { base } => {
                    if !registry.classes.contains_key(base)
                        || property.default.as_str().is_some_and(|id| {
                            !crate::blueprint_refs::class_is_a(&registry, id, base)
                        })
                    {
                        return Err(vec![diagnostic(
                            file,
                            format!(
                                "Property {} has a missing or incompatible class reference",
                                property.name
                            ),
                        )]);
                    }
                }
                schema::Type::ObjectRef { class: Some(base) }
                    if !registry.classes.contains_key(base) =>
                {
                    return Err(vec![diagnostic(
                        file,
                        format!(
                            "Property {} references an unknown entity class",
                            property.name
                        ),
                    )]);
                }
                _ => {}
            }
        }
        let parent = registry.classes[&a.parent].clone();
        let family = model
            .as_ref()
            .and_then(|model| model.class(&a.id))
            .map(|class| class.family)
            .unwrap_or(schema::ClassFamily::Object);
        if !matches!(
            family,
            schema::ClassFamily::Actor | schema::ClassFamily::Component
        ) {
            return Err(vec![diagnostic(
                file,
                "Blueprint parent must derive from Actor or ActorComponent",
            )]);
        }
        if let Some(hint) = a.family
            && hint != family
        {
            return Err(vec![diagnostic(
                file,
                format!(
                    "Blueprint declares the {} family but its parent chain resolves to {}; the registry is authoritative",
                    hint.label(),
                    family.label()
                ),
            )]);
        }
        // A map's own Blueprint is a `SceneScriptActor` subclass by construction:
        // the loader creates exactly one per Level and nothing else may be in that
        // slot. The map is what the diagnostic points at, because the parent is
        // chosen in Map Settings and not in the Blueprint asset.
        if file.is_embedded() {
            if !model.as_ref().is_some_and(|model| {
                model.is_a(&a.parent, crate::object_model::SCENE_SCRIPT_ACTOR_ID)
            }) {
                return Err(vec![diagnostic(
                    file,
                    format!(
                        "The scene Blueprint of this map derives from {}, which is not an epok::SceneScriptActor subclass; choose a scene Blueprint parent in Map Settings",
                        parent.cpp_name
                    ),
                )]);
            }
            scene_scripts.push((file.path.clone(), a.id.clone()));
        }
        // Persisted references are resolved by UUID and scope now; the generated
        // constructor keeps the null identity and the Level loader binds them when
        // it spawns the class. A reference inside a graph is code, not persisted
        // data, and has no per-instance slot to bind, so it stays refused.
        let mut resolved = vec![];
        for property in &class.properties {
            if let Some(reference) = scene_reference(
                file,
                &a.id,
                &format!("property:{}", property.name),
                &property.value_type,
                &property.default,
            )
            .map_err(|e| vec![e])?
            {
                resolved.push(reference);
            }
        }
        for graph in &a.functions {
            for node in graph.compilation_nodes() {
                let literals = node
                    .inputs
                    .values()
                    .filter_map(|input| match input {
                        asset::Input::Literal { value_type, value } => Some((value_type, value)),
                        _ => None,
                    })
                    .chain(match &node.kind {
                        asset::NodeKind::Literal { value_type, value } => Some((value_type, value)),
                        _ => None,
                    });
                for (value_type, value) in literals {
                    if let Some(target) = reference_target(value_type, value) {
                        return Err(vec![Diagnostic {
                            asset: file.path.clone(),
                            graph: Some(graph.id.clone()),
                            node: Some(node.id.clone()),
                            message: format!(
                                "A graph literal cannot carry the identity {target}: promote it to a variable, which the Level loader binds when the class is spawned"
                            ),
                        }]);
                    }
                }
            }
        }
        // Null the resolved slots for code generation only. `files` keeps the
        // authored values, so semantic hashes and footprints are unchanged.
        let scoped;
        let a = if resolved.is_empty() {
            a
        } else {
            let bound = class
                .properties
                .iter()
                .filter(|p| reference_target(&p.value_type, &p.default).is_some())
                .map(|p| p.id.clone())
                .collect::<BTreeSet<_>>();
            for property in &mut class.properties {
                if bound.contains(&property.id) {
                    property.default = serde_json::Value::Null;
                }
            }
            let mut copy = a.clone();
            for variable in &mut copy.variables {
                if bound.contains(&variable.id) {
                    variable.default = serde_json::Value::Null;
                }
            }
            scoped = copy;
            &scoped
        };
        scene_references.append(&mut resolved);
        let self_kind = self_kind_of(family);
        let mut properties = BTreeMap::new();
        let mut functions = BTreeMap::new();
        for c in registry.ancestry(&parent.cpp_name) {
            for p in &c.properties {
                properties.insert(p.id.clone(), p.clone());
            }
            for f in &c.functions {
                insert_function(&mut functions, f);
            }
        }
        for p in &class.properties {
            properties.insert(p.id.clone(), p.clone());
        }
        for property in properties.values() {
            if let Some(previous) = trace_ids.insert(trace_id(&property.id), property.id.clone())
                && previous != property.id
            {
                return Err(vec![diagnostic(
                    file,
                    "Compact debugger member ID collision; migrate the conflicting stable identity",
                )]);
            }
        }
        let parent_functions = functions.clone();
        for f in &class.functions {
            functions.insert(f.id.clone(), f.clone());
        }
        let include = if parent.provider.id == "blueprint" {
            format!("{}.hpp", parent.id)
        } else if parent.cpp_name.starts_with("epok::") {
            // Behaviour, Actor/Component roots and every other reflected runtime base
            // are reachable from the umbrella header; they have no assets/scripts file.
            "epok.hpp".into()
        } else {
            let script_root = root.join("assets/scripts");
            let canonical_root = std::fs::canonicalize(&script_root).unwrap_or(script_root);
            let canonical_parent = std::fs::canonicalize(&parent.source.file)
                .unwrap_or_else(|_| parent.source.file.clone());
            let relative = canonical_parent
                .strip_prefix(&canonical_root)
                .map_err(|_| {
                    vec![diagnostic(
                        file,
                        "Native parent header must be within assets/scripts",
                    )]
                })?;
            format!("../{}", relative.to_string_lossy().replace('\\', "/"))
        };
        if include.contains('"') || include.contains('\n') {
            return Err(vec![diagnostic(file, "Unsafe parent include path")]);
        }
        let mut text = format!(
            "// Generated from Blueprint {}. Do not edit.\n#pragma once\n#include \"{include}\"\n#include \"blueprint_api.hpp\"\n#include \"blueprint_debug.hpp\"\nclass {} : public {} {{\npublic:\n",
            a.id, a.name, parent.cpp_name
        );
        if has_receivers {
            // Keep declarations outside the generated class body.
            text = text.replacen(
                "#pragma once\n",
                "#pragma once\n#include \"blueprint_calls.hpp\"\n",
                1,
            );
        }
        if has_playback_calls {
            text = text.replacen(
                "#pragma once\n",
                "#pragma once\n#include \"playback_calls.hpp\"\n",
                1,
            );
        }
        text.push_str(&format!("static constexpr uint64_t static_class_id={}ULL;\nuint64_t class_id() const override {{return static_class_id;}}\n",crate::blueprint_refs::compact_id(&a.id)));
        text.push_str(&debug_members(&properties, &a.id));
        for v in &a.variables {
            if let schema::Type::Vector { length } = v.value_type {
                text.push_str(&format!("epok::Fixed {}[{length}]{{}};\n", v.name));
            } else {
                text.push_str(&format!(
                    "{} {} = {};\n",
                    ir::cpp_type(&v.value_type).map_err(|e| vec![diagnostic(file, e)])?,
                    v.name,
                    ir::literal(&v.default, &v.value_type)
                        .map_err(|e| vec![diagnostic(file, e)])?
                ));
            }
        }
        text.push_str(&format!("{}() {{\nusing epok::Fixed;\n", a.name));
        for p in &class.properties {
            text.push_str(
                &crate::script_values::assignment(
                    &format!("this->{}", p.name),
                    &p.default,
                    &p.value_type,
                )
                .map_err(|e| vec![diagnostic(file, e)])?,
            );
        }
        text.push_str("}\n");
        let mut continuations = String::new();
        let mut timeline_prepare = String::new();
        let mut playback_observe = String::new();
        let mut timeline_dispatch = String::new();
        let mut timeline_cancel = String::new();
        let mut latent_count = 0;
        // An Actor/Component class has no `blueprint_tick`/`blueprint_cancel` hook: the
        // reflected `tick`/`end_play` events are the continuation pump. When the class
        // owns latent frames the authored graphs for those two events are emitted under
        // private names and a single synthesized override drives them.
        let latent_events = expects_latent(a);
        let tick_signature = functions
            .values()
            .find(|f| f.name == "tick" && f.parameters.len() == 1)
            .cloned();
        let end_play_signature = functions
            .values()
            .find(|f| f.name == "end_play" && f.parameters.len() == 1)
            .cloned();
        let synthesize = latent_events && tick_signature.is_some();
        let mut has_tick_graph = false;
        let mut has_end_play_graph = false;
        let writable = registry
            .ancestry(&a.name)
            .iter()
            .filter(|c| c.provider.id == "blueprint")
            .flat_map(|c| c.properties.iter().map(|p| p.id.clone()))
            .collect::<BTreeSet<_>>();
        for (graph, signature) in a
            .functions
            .iter()
            .zip(&class.functions)
            .filter(|(graph, _)| !graph.inherits_event())
        {
            let parent_function = graph
                .override_id
                .as_ref()
                .and_then(|id| parent_functions.get(id))
                .cloned()
                .map(|mut f| {
                    f.parameters = graph.parameters.clone();
                    f
                });
            let context = ir::Context {
                registry: &registry,
                model: model.as_ref(),
                self_kind,
                playback_timelines: &playback_timelines,
                playback_effects: &playback_effects,
                writable: &writable,
                properties: &properties,
                functions: &functions,
                self_class: &a.name,
                parent_name: &parent.cpp_name,
                parent_function: parent_function.as_ref(),
            };
            let mut lowered = ir::lower(graph, signature, &context).map_err(|e| {
                vec![Diagnostic {
                    asset: file.path.clone(),
                    graph: Some(graph.id.clone()),
                    node: e.node,
                    message: e.message,
                }]
            })?;
            if cost(&lowered.body) > 4096 {
                return Err(vec![Diagnostic{asset:file.path.clone(),graph:Some(graph.id.clone()),node:None,message:"Graph exceeds the 4096-statement execution budget including nested loop expansion".into()}]);
            }
            let subscriptions = subscription_frames(&mut lowered);
            if synthesize {
                match lowered.signature.name.as_str() {
                    "tick" => {
                        has_tick_graph = true;
                        lowered.signature.name = "epok_graph_tick".into();
                    }
                    "end_play" if end_play_signature.is_some() => {
                        has_end_play_graph = true;
                        lowered.signature.name = "epok_graph_end_play".into();
                    }
                    _ => {}
                }
            }
            let renamed = lowered.signature.name.starts_with("epok_graph_");
            let parameters = signature
                .parameters
                .iter()
                .map(|p| {
                    Ok(format!(
                        "{}{} {}",
                        match p.direction {
                            schema::Direction::ConstReference =>
                                format!("const {}", ir::cpp_type(&p.value_type)?),
                            _ => ir::cpp_type(&p.value_type)?,
                        },
                        if p.direction == schema::Direction::Value {
                            ""
                        } else {
                            "&"
                        },
                        p.name
                    ))
                })
                .collect::<Result<Vec<_>, String>>()
                .map_err(|e| vec![diagnostic(file, e)])?
                .join(",");
            text.push_str(&format!(
                "virtual {} {}({parameters}){} {{\n",
                ir::cpp_type(&lowered.signature.returns).map_err(|e| vec![diagnostic(file, e)])?,
                lowered.signature.name,
                if graph.override_id.is_some() && !renamed {
                    " override"
                } else {
                    ""
                }
            ));
            for p in &signature.parameters {
                text.push_str(&format!("(void){};\n", p.name));
            }
            text.push_str(&format!("{}\n", self_prologue(self_kind)));
            if has_delay(&lowered.body) {
                latent_count += 1;
                let code = latent(file, graph, &graph.id, &lowered, self_kind, &mut text)
                    .map_err(|e| vec![e])?;
                continuations.push_str(&code.resume);
                playback_observe.push_str(&code.observe);
                timeline_prepare.push_str(&code.prepare);
                timeline_dispatch.push_str(&code.dispatch);
                timeline_cancel.push_str(&code.cancel);
            } else {
                for (name, ty) in &lowered.temporaries {
                    text.push_str(&format!(
                        "{} {name}{{}};\n",
                        ir::cpp_type(ty).map_err(|e| vec![diagnostic(file, e)])?
                    ));
                }
                write_body(&lowered.body, &mut text, &a.id, &graph.id, 0, self_kind);
                text.push_str("}\n");
            }
            for (frame_id, listener) in subscriptions {
                let parameters = listener
                    .signature
                    .parameters
                    .iter()
                    .map(|p| {
                        let ty = ir::cpp_type(&p.value_type)?;
                        Ok(format!(
                            "{}{} {}",
                            if p.direction == schema::Direction::ConstReference {
                                format!("const {ty}")
                            } else {
                                ty
                            },
                            if p.direction == schema::Direction::Value {
                                ""
                            } else {
                                "&"
                            },
                            p.name
                        ))
                    })
                    .collect::<Result<Vec<_>, String>>()
                    .map_err(|e| vec![diagnostic(file, e)])?
                    .join(",");
                text.push_str(&format!(
                    "void {}({parameters}){{\n",
                    listener.signature.name
                ));
                let code = latent(file, graph, &frame_id, &listener, self_kind, &mut text)
                    .map_err(|e| vec![e])?;
                latent_count += 1;
                continuations.push_str(&code.resume);
                playback_observe.push_str(&code.observe);
                timeline_prepare.push_str(&code.prepare);
                timeline_dispatch.push_str(&code.dispatch);
                timeline_cancel.push_str(&code.cancel);
            }
        }
        if latent_count > 8 {
            return Err(vec![diagnostic(
                file,
                "A class supports at most eight simultaneously active latent event frames",
            )]);
        }
        if latent_count > 0 && !synthesize {
            return Err(vec![diagnostic(
                file,
                "Latent nodes in an Actor or Component Blueprint need the reflected tick event",
            )]);
        }
        if synthesize {
            let tick = tick_signature.expect("latent Actor events require a reflected tick");
            let dt = "dt";
            let dt_type = ir::cpp_type(&tick.parameters[0].value_type)
                .map_err(|e| vec![diagnostic(file, e)])?;
            text.push_str("epok::bp::Continuations<8> epok_tasks;\n");
            text.push_str(&format!(
                "void tick({dt_type} {dt}) override {{\n{}\n{playback_observe}epok_tasks.advance({dt},epok::blueprint_scene_generation);\n{timeline_prepare}{}{timeline_dispatch}epok::bp::Continuation epok_cont;\nwhile(epok_tasks.poll(epok_cont)){{switch(epok_cont.node){{{continuations}default:break;}}}}\n{}}}\n",
                self_prologue(self_kind),
                if has_tick_graph {
                    String::new()
                } else {
                    format!("{}::tick({dt});\n", parent.cpp_name)
                },
                if has_tick_graph {
                    format!("this->epok_graph_tick({dt});\n")
                } else {
                    String::new()
                }
            ));
            if let Some(end_play) = end_play_signature {
                let reason = end_play.parameters[0].name.clone();
                let reason_type = ir::cpp_type(&end_play.parameters[0].value_type)
                    .map_err(|e| vec![diagnostic(file, e)])?;
                text.push_str(&format!(
                    "void end_play({reason_type} {reason}) override {{\nepok_tasks.cancel_all();\n{timeline_cancel}{}}}\n",
                    if has_end_play_graph {
                        format!("this->epok_graph_end_play({reason});\n")
                    } else {
                        format!("{}::end_play({reason});\n", parent.cpp_name)
                    }
                ));
            }
            if !playback_observe.is_empty() {
                text.push_str(&format!("void blueprint_observe() override {{ {}::blueprint_observe();{playback_observe}}}\n",parent.cpp_name));
                artifacts
                    .runtime_capabilities
                    .insert("playback_wait".into());
            }
        }
        text.push_str("};\n");
        let header = PathBuf::from(format!("generated/{}.hpp", a.id));
        let artifact_header = Path::new("scripts").join(&header);
        artifacts.files.insert(artifact_header, text.into_bytes());
        artifacts.files.insert(PathBuf::from(format!("blueprints/{}.epokdebug",a.id)),crate::document::to_vec(&serde_json::json!({"version":1,"class":a.id,"trace_id":trace_id(&a.id),"debug_abi":{"version":1,"words":119,"entries":16,"entry_words":7},"members":properties.values().map(|p|serde_json::json!({"id":p.id,"trace_id":trace_id(&p.id),"name":p.name,"value_type":p.value_type})).collect::<Vec<_>>(),"source":file.path.strip_prefix(root).unwrap_or(&file.path),"semantic_hash":file.asset.semantic_hash(),"graphs":a.functions.iter().map(|g|serde_json::json!({"id":g.id,"name":g.name,"nodes":g.nodes.iter().map(|n|serde_json::json!({"id":n.id,"trace_id":trace_id(&n.id),"pins":n.inputs.keys().chain(n.outputs.keys()).map(|p|asset::pin_id(&n.id,p)).collect::<Vec<_>>(),"links":n.inputs.iter().filter_map(|(pin,input)|if let asset::Input::Link{node, pin:output}=input{Some(asset::link_id(node,output,&n.id,pin))}else{None}).collect::<Vec<_>>()})).collect::<Vec<_>>()})).collect::<Vec<_>>()})).unwrap());
        artifacts.dependencies.insert(file.path.clone());
        artifacts.runtime_capabilities.insert("blueprint".into());
        registry.classes.insert(class.id.clone(), class.clone());
        let mut classes = vec![class];
        classes.extend(registry.ancestry(&parent.cpp_name).into_iter().cloned());
        scripts.push(Script {
            name: a.name.clone(),
            parent: Some(parent.cpp_name),
            properties: properties
                .values()
                .map(|p| Property {
                    name: p.name.clone(),
                    default: p.default.clone(),
                    value_type: p.value_type.clone(),
                    id: p.id.clone(),
                })
                .collect(),
            header,
            classes,
        });
    }
    let footprints = crate::blueprint_dependencies::capture(
        files,
        &registry,
        &artifacts,
        &dependency_timelines,
        &playback_effects,
        &resource_dependencies.footprints,
    )
    .map_err(|message| {
        vec![diagnostic(
            files
                .first()
                .expect("Capture failures require a source reference"),
            message,
        )]
    })?;
    artifacts.blueprint_set = Some(crate::blueprint_dependencies::source_set(files));
    scene_scripts.sort();
    scene_references.sort();
    Ok(Compilation {
        scripts,
        registry,
        artifacts,
        footprints,
        scene_scripts,
        scene_references,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use asset::{BlueprintAsset, Graph, Input, Node, NodeKind};
    use serde_json::json;
    fn id(n: u128) -> String {
        uuid::Uuid::from_u128(n).to_string()
    }
    fn registry() -> Registry {
        let class = schema::Class {
            family: None,
            domain: None,
            placement: Default::default(),
            component: None,
            default_components: vec![],
            explicit_abstract: false,
            id: id(1),
            provider: schema::native_provider(),
            backend: schema::native_backend(),
            cpp_name: "Enemy".into(),
            parent: Some(crate::object_model::ACTOR3D_ID.into()),
            abstract_class: false,
            final_class: false,
            timeline_component: None,
            blueprintable: true,
            properties: vec![schema::Property {
                id: id(2),
                name: "health".into(),
                value_type: schema::Type::Fixed,
                default: json!(100),
                editable: true,
                timeline: None,
                source: location(Path::new("assets/scripts/Enemy.hpp")),
            }],
            functions: vec![schema::Function {
                id: id(3),
                name: "damage".into(),
                parameters: vec![schema::Parameter {
                    name: "amount".into(),
                    value_type: schema::Type::Fixed,
                    direction: schema::Direction::Value,
                }],
                returns: schema::Type::Void,
                callable: true,
                timeline: None,
                event: true,
                pure: false,
                abstract_method: false,
                final_method: false,
                access: "public".into(),
                overrides: vec![],
                source: location(Path::new("assets/scripts/Enemy.hpp")),
            }],
            source: location(Path::new("assets/scripts/Enemy.hpp")),
        };
        let mut registry = crate::actor_document::tests::registry();
        registry.classes.insert(class.id.clone(), class);
        registry
    }
    fn asset() -> AssetFile {
        let mut a = BlueprintAsset::new("Boss".into(), id(1));
        a.id = id(4);
        AssetFile::file("assets/blueprints/Boss.epokbp".into(), a)
    }
    fn node(n: u128, kind: NodeKind) -> Node {
        Node {
            id: id(n),
            kind,
            inputs: BTreeMap::new(),
            outputs: BTreeMap::new(),
        }
    }
    fn event(nodes: Vec<Node>) -> Graph {
        Graph {
            timeline: None,
            id: id(5),
            name: "damage".into(),
            override_id: Some(id(3)),
            parameters: registry().classes[&id(1)].functions[0].parameters.clone(),
            returns: schema::Type::Void,
            entry: id(10),
            nodes,
        }
    }
    fn generated(compiled: &Compilation) -> String {
        String::from_utf8(
            compiled.artifacts.files[&PathBuf::from(format!("scripts/generated/{}.hpp", id(4)))]
                .clone(),
        )
        .unwrap()
    }
    fn error(file: AssetFile) -> String {
        compile(Path::new(""), &registry(), &[file]).err().unwrap()[0].to_string()
    }
    mod connectivity {
        include!("blueprint_connectivity_tests.rs");
    }
    mod actors {
        include!("blueprint_actor_tests.rs");
    }
    mod scene_scripts {
        include!("blueprint_scene_tests.rs");
    }
    #[test]
    fn data_only_inheritance_keeps_parent_behavior_and_default_chain() {
        let mut f = asset();
        f.asset.defaults.insert(id(2), json!(150));
        let first = compile(Path::new(""), &registry(), &[f.clone()]).unwrap();
        let source = generated(&first);
        assert!(source.contains("class Boss : public Enemy"));
        assert!(!source.contains("void damage"));
        assert!(source.contains("Fixed(614400, Fixed::RAW)"));
        assert_eq!(first.scripts[0].properties[0].default, json!(150));
        assert_eq!(first.scripts[0].classes[0].id, id(4));
        let mut child = asset();
        child.asset.id = id(6);
        child.asset.name = "FinalBoss".into();
        child.asset.parent = id(4);
        let result = compile(Path::new(""), &registry(), &[child, f]).unwrap();
        assert_eq!(result.scripts[1].properties[0].default, json!(150));
    }
    #[test]
    fn event_call_parent_uses_reflected_signature_and_qualified_dispatch() {
        let mut f = asset();
        let mut entry = node(10, NodeKind::Entry);
        entry.outputs.insert("next".into(), vec![id(11)]);
        let mut parent = node(11, NodeKind::CallParent);
        parent.inputs.insert(
            "amount".into(),
            Input::Parameter {
                name: "amount".into(),
            },
        );
        f.asset.functions.push(event(vec![entry, parent]));
        let result = compile(Path::new(""), &registry(), &[f.clone()]).unwrap();
        assert!(generated(&result).contains("Enemy::damage(epok_argument_0)"));
        assert!(generated(&result).contains("void damage(epok::Fixed amount) override"));
        f.asset.functions[0].parameters[0].value_type = schema::Type::Bool;
        assert!(error(f).contains("signature differs"));
    }
    #[test]
    fn cycles_stale_ids_and_types_are_diagnosed_without_mutation() {
        let mut f = asset();
        f.asset.parent = id(4);
        assert!(error(f).contains("inheritance cycle"));
        let mut f = asset();
        f.asset.defaults.insert(id(99), json!(2));
        let bytes = serde_json::to_vec(&f.asset).unwrap();
        assert!(error(f.clone()).contains("missing property"));
        assert_eq!(bytes, serde_json::to_vec(&f.asset).unwrap());
        let mut f = asset();
        let mut entry = node(10, NodeKind::Entry);
        entry.outputs.insert("next".into(), vec![id(11)]);
        let mut set = node(11, NodeKind::SetVariable { member: id(2) });
        set.inputs.insert(
            "value".into(),
            Input::Literal {
                value_type: schema::Type::Bool,
                value: json!(true),
            },
        );
        f.asset.functions.push(event(vec![entry, set]));
        assert!(error(f).contains("Assignment expects"));
    }
    #[test]
    fn unwired_event_inherits_until_connected_and_parent_calls_are_explicit() {
        let mut file = asset();
        file.asset
            .functions
            .push(event(vec![node(10, NodeKind::Entry)]));
        let native = registry();
        let inherited = compile(Path::new(""), &native, &[file.clone()]).unwrap();
        assert!(!generated(&inherited).contains("void damage("));
        assert_eq!(inherited.registry.classes[&id(4)].functions[0].id, id(5));

        // Connecting even a Return is an intentional empty override.
        let graph = &mut file.asset.functions[0];
        graph.nodes.push(node(11, NodeKind::Return));
        graph.nodes[0].outputs.insert("next".into(), vec![id(11)]);
        let overridden = compile(Path::new(""), &native, &[file.clone()]).unwrap();
        assert!(generated(&overridden).contains("void damage("));
        assert!(!generated(&overridden).contains("Enemy::damage("));

        // Old saved pin bindings survive a descriptive SDK parameter rename.
        let graph = &mut file.asset.functions[0];
        graph.parameters[0].name = "arg0".into();
        graph.nodes[1].kind = NodeKind::CallParent;
        graph.nodes[1].inputs.insert(
            "arg0".into(),
            Input::Link {
                node: id(10),
                pin: "arg0".into(),
            },
        );
        let source = generated(&compile(Path::new(""), &native, &[file.clone()]).unwrap());
        assert!(source.contains("Enemy::damage(epok_argument_0)"));

        file.asset.functions[0].nodes[0].outputs.clear();
        let inherited_again = compile(Path::new(""), &native, &[file.clone()]).unwrap();
        assert!(!generated(&inherited_again).contains("void damage("));
        assert_eq!(file.asset.functions[0].parameters[0].name, "arg0");
    }
    #[test]
    fn execution_cycle_and_nested_loop_budget_are_checked() {
        let mut f = asset();
        let mut entry = node(10, NodeKind::Entry);
        entry.outputs.insert("next".into(), vec![id(10)]);
        f.asset.functions.push(event(vec![entry]));
        assert!(error(f).contains("Entry"));
        let mut f = asset();
        let mut entry = node(10, NodeKind::Entry);
        entry.outputs.insert("next".into(), vec![id(11)]);
        let mut outer = node(11, NodeKind::Loop { count: 1024 });
        outer.outputs.insert("body".into(), vec![id(12)]);
        let mut inner = node(12, NodeKind::Loop { count: 1024 });
        inner.outputs.insert("body".into(), vec![id(13)]);
        f.asset.functions.push(event(vec![
            entry,
            outer,
            inner,
            node(13, NodeKind::Sequence),
        ]));
        assert!(error(f).contains("execution budget"));
    }
    #[test]
    fn delay_persists_parameters_and_resumes_inside_bounded_loop() {
        let mut f = asset();
        let mut entry = node(10, NodeKind::Entry);
        entry.outputs.insert("next".into(), vec![id(11)]);
        let mut repeat = node(11, NodeKind::Loop { count: 2 });
        repeat.outputs.insert("body".into(), vec![id(12)]);
        let mut delay = node(12, NodeKind::Delay);
        delay.inputs.insert(
            "seconds".into(),
            Input::Literal {
                value_type: schema::Type::Fixed,
                value: json!(0.5),
            },
        );
        delay.outputs.insert("next".into(), vec![id(13)]);
        let mut set = node(13, NodeKind::SetVariable { member: id(2) });
        set.inputs.insert(
            "value".into(),
            Input::Parameter {
                name: "amount".into(),
            },
        );
        f.asset
            .functions
            .push(event(vec![entry, repeat, delay, set]));
        let compiled = compile(Path::new(""), &registry(), &[f]).unwrap();
        let code = generated(&compiled);
        assert!(code.contains("epok_arg_"));
        assert!(code.contains("epok_loop_"));
        assert!(code.contains("epok_tasks.cancel("));
        assert!(code.contains("epok_tasks.delay("));
        assert!(code.contains("Enemy::tick(dt)"));
        assert!(code.contains("Enemy::end_play(reason)"));
    }
    #[test]
    fn typed_playback_waits_share_latent_frames_and_reject_serialized_handles() {
        let mut file = asset();
        let mut entry = node(10, NodeKind::Entry);
        entry.outputs.insert("next".into(), vec![id(11)]);
        let self_node = node(
            12,
            NodeKind::Builtin {
                operation: asset::Builtin::SelfObject,
            },
        );
        let mut play = node(
            11,
            NodeKind::Builtin {
                operation: asset::Builtin::PlayEffectComponent,
            },
        );
        play.inputs.insert(
            "target".into(),
            Input::Link {
                node: id(12),
                pin: "value".into(),
            },
        );
        play.outputs.insert("next".into(), vec![id(13)]);
        let mut wait = node(
            13,
            NodeKind::WaitPlayback {
                condition: asset::PlaybackCondition::EffectComplete,
            },
        );
        wait.inputs.insert(
            "playback".into(),
            Input::Link {
                node: id(11),
                pin: "value".into(),
            },
        );
        wait.outputs.insert("completed".into(), vec![id(14)]);
        wait.outputs.insert("cancelled".into(), vec![id(15)]);
        let mut completed = node(14, NodeKind::SetVariable { member: id(2) });
        completed.inputs.insert(
            "value".into(),
            Input::Literal {
                value_type: schema::Type::Fixed,
                value: json!(1),
            },
        );
        let mut cancelled = node(15, NodeKind::SetVariable { member: id(2) });
        cancelled.inputs.insert(
            "value".into(),
            Input::Literal {
                value_type: schema::Type::Fixed,
                value: json!(2),
            },
        );
        file.asset.functions.push(event(vec![
            entry, play, self_node, wait, completed, cancelled,
        ]));
        let compiled = compile(Path::new(""), &registry(), &[file.clone()]).unwrap();
        let source = generated(&compiled);
        assert!(source.contains("epok_tasks.wait_external("));
        assert!(source.contains("void blueprint_observe() override"));
        assert!(source.contains("epok::effects::Handle"));
        assert!(
            compiled
                .artifacts
                .runtime_capabilities
                .contains("playback_wait")
        );
        assert_eq!(
            source
                .matches("epok::bp::Continuations<8> epok_tasks;")
                .count(),
            1
        );
        file.asset.functions[0].nodes[3].kind = NodeKind::WaitPlayback {
            condition: asset::PlaybackCondition::SequenceComplete,
        };
        assert!(error(file.clone()).contains("Playback wait requires"));
        file.asset.functions[0].nodes[3].inputs.insert(
            "playback".into(),
            Input::Literal {
                value_type: schema::Type::SequenceHandle,
                value: json!({"index":0,"generation":1}),
            },
        );
        assert!(error(file).contains("Literal"));
        for ty in [schema::Type::SequenceHandle, schema::Type::EffectHandle] {
            let mut file = asset();
            file.asset.variables.push(asset::Variable {
                id: id(55),
                name: "playback".into(),
                value_type: ty.clone(),
                default: serde_json::Value::Null,
                editable: false,
                timeline_animatable: false,
            });
            assert!(compile(Path::new(""), &registry(), &[file.clone()]).is_ok());
            file.asset.variables[0].editable = true;
            assert!(error(file).contains("transient"));
            assert!(
                ir::literal(&serde_json::Value::Null, &ty)
                    .unwrap()
                    .ends_with("Handle{}")
            );
            assert!(ir::literal(&json!(1), &ty).is_err());
        }
    }
    #[test]
    fn direct_asset_plays_use_uuid_binding_pins_and_fail_closed_on_schema_changes() {
        let root = crate::workspace::tests::temp("blueprint-asset-plays");
        std::fs::create_dir_all(root.join("assets/Timelines")).unwrap();
        let mut timeline = crate::timeline::TimelineAsset::new("Spell".into());
        let required = uuid::Uuid::new_v4();
        let optional = uuid::Uuid::new_v4();
        timeline.slots = vec![
            crate::timeline::Slot {
                id: required,
                name: "Caster".into(),
                target: schema::Type::ObjectRef { class: Some(id(1)) },
                required: true,
                extra: Default::default(),
            },
            crate::timeline::Slot {
                id: optional,
                name: "Optional target".into(),
                target: schema::Type::ObjectRef { class: Some(id(1)) },
                required: false,
                extra: Default::default(),
            },
        ];
        let path = root.join("assets/Timelines/Spell.timeline.json");
        std::fs::write(&path, serde_json::to_vec(&timeline).unwrap()).unwrap();
        let mut file = asset();
        let mut entry = node(10, NodeKind::Entry);
        entry.outputs.insert("next".into(), vec![id(13)]);
        let own = node(
            11,
            NodeKind::Builtin {
                operation: asset::Builtin::SelfObject,
            },
        );
        let mut cast = node(
            12,
            NodeKind::Builtin {
                operation: asset::Builtin::Cast { class: id(1) },
            },
        );
        cast.inputs.insert(
            "target".into(),
            Input::Link {
                node: id(11),
                pin: "value".into(),
            },
        );
        let mut play = node(
            13,
            NodeKind::Builtin {
                operation: asset::Builtin::PlayTimelineAsset {
                    asset: timeline.id.to_string(),
                },
            },
        );
        play.inputs.insert(
            "owner".into(),
            Input::Link {
                node: id(11),
                pin: "value".into(),
            },
        );
        file.asset
            .functions
            .push(event(vec![entry, own, cast, play]));
        let mut native = registry();
        native.classes.get_mut(&id(1)).unwrap().source.file = root.join("assets/scripts/Enemy.hpp");
        let compile_file = |file: &AssetFile| compile(&root, &native, std::slice::from_ref(file));
        let message = |file: &AssetFile| compile_file(file).err().unwrap()[0].to_string();
        assert!(
            message(&file).contains("Required playback binding Caster"),
            "{}",
            message(&file)
        );
        file.asset.functions[0].nodes[3].inputs.insert(
            format!("binding:{required}"),
            Input::Literal {
                value_type: schema::Type::ObjectRef { class: Some(id(1)) },
                value: serde_json::Value::Null,
            },
        );
        assert!(message(&file).contains("cannot be null"));
        file.asset.functions[0].nodes[3].inputs.insert(
            format!("binding:{required}"),
            Input::Link {
                node: id(11),
                pin: "value".into(),
            },
        );
        assert!(
            compile_file(&file).is_ok(),
            "Self carries its Actor class into typed playback bindings"
        );
        file.asset.functions[0].nodes[3].inputs.insert(
            format!("binding:{required}"),
            Input::Link {
                node: id(12),
                pin: "value".into(),
            },
        );
        let compiled = compile_file(&file).unwrap();
        assert!(compiled.artifacts.dependencies.contains(&path));
        assert!(generated(&compiled).contains("epok::bp::playback::sequence_"));
        let mut independent = file.clone();
        independent.asset.id = id(600);
        independent.asset.name = "Independent".into();
        independent.asset.functions.clear();
        independent.path = root.join("assets/Independent.epokbp");
        // Match production: observe the source set before recording compilation,
        // including resource projections which are not generated-code inputs.
        crate::blueprint_dependencies::observe_sources(
            &root,
            &[file.clone(), independent.clone()],
            Some(&native),
        )
        .unwrap();
        let compiled_pair = compile(&root, &native, &[file.clone(), independent.clone()]).unwrap();
        crate::blueprint_dependencies::record(&root, &compiled_pair).unwrap();
        let key = format!("generated-blueprint:{}", file.asset.id);
        let unrelated_key = format!("generated-blueprint:{}", independent.asset.id);
        let before = crate::artifact_dependencies::Graph::load(&root).unwrap();
        assert!(before.nodes[&key].stale.is_empty());
        assert!(
            before.nodes[&key]
                .dependencies
                .contains(&format!("timeline:{}", timeline.id))
        );
        let mut signature_changed = compiled_pair.registry.clone();
        signature_changed.classes.get_mut(&id(1)).unwrap().functions[0].name =
            "renamed_damage".into();
        crate::blueprint_dependencies::observe_reflection(&root, &signature_changed).unwrap();
        let renamed = crate::artifact_dependencies::Graph::load(&root).unwrap();
        assert!(!renamed.nodes[&key].stale.is_empty());
        assert_eq!(renamed.nodes[&unrelated_key], before.nodes[&unrelated_key]);
        crate::blueprint_dependencies::record(&root, &compiled_pair).unwrap();
        timeline.duration_ticks += 68;
        std::fs::write(&path, serde_json::to_vec(&timeline).unwrap()).unwrap();
        // Publishing an earlier successful compilation must retain its actual
        // source snapshot, even if the file changed before publication.
        crate::blueprint_dependencies::record(&root, &compiled_pair).unwrap();
        assert_eq!(
            crate::artifact_dependencies::Graph::load(&root)
                .unwrap()
                .nodes[&format!("timeline:{}", timeline.id)]
                .signature
                .as_ref(),
            Some(&compiled_pair.footprints[&key].1[&format!("timeline:{}", timeline.id)])
        );
        crate::timeline_compile::observe_sources(&root).unwrap();
        let changed = crate::artifact_dependencies::Graph::load(&root).unwrap();
        assert!(!changed.nodes[&key].stale.is_empty());
        assert_eq!(changed.nodes[&unrelated_key], before.nodes[&unrelated_key]);
        let mut layout_only = file.clone();
        layout_only
            .asset
            .layout
            .positions
            .insert(id(10), [200., 120.]);
        crate::blueprint_dependencies::observe_sources(
            &root,
            &[layout_only, independent.clone()],
            Some(&native),
        )
        .unwrap();
        assert_eq!(
            crate::artifact_dependencies::Graph::load(&root).unwrap(),
            changed
        );
        let refreshed = compile(&root, &native, &[file.clone(), independent]).unwrap();
        crate::blueprint_dependencies::record(&root, &refreshed).unwrap();
        assert!(
            crate::artifact_dependencies::Graph::load(&root)
                .unwrap()
                .nodes[&key]
                .stale
                .is_empty()
        );
        let declarations =
            &compiled.artifacts.files[&PathBuf::from("scripts/generated/playback_calls.hpp")];
        timeline.slots.reverse();
        std::fs::write(&path, serde_json::to_vec(&timeline).unwrap()).unwrap();
        assert_eq!(
            declarations,
            &compile_file(&file).unwrap().artifacts.files
                [&PathBuf::from("scripts/generated/playback_calls.hpp")]
        );
        timeline.slots.retain(|slot| slot.id != required);
        std::fs::write(&path, serde_json::to_vec(&timeline).unwrap()).unwrap();
        let before = serde_json::to_vec(&file.asset).unwrap();
        assert!(message(&file).contains("stale connection preserved"));
        assert_eq!(before, serde_json::to_vec(&file.asset).unwrap());
    }
    #[test]
    fn marker_subscription_compiles_latent_branches_and_preserves_missing_marker() {
        let root = crate::workspace::tests::temp("blueprint-marker-subscription");
        std::fs::create_dir_all(root.join("assets/Timelines")).unwrap();
        let mut timeline = crate::timeline::TimelineAsset::new("Looping spell".into());
        let marker = uuid::Uuid::new_v4();
        timeline.markers.push(crate::timeline::Marker {
            id: marker,
            name: "Impact".into(),
            tick: 100,
            extra: Default::default(),
        });
        let path = root.join("assets/Timelines/Spell.timeline.json");
        std::fs::write(&path, serde_json::to_vec(&timeline).unwrap()).unwrap();
        let mut file = asset();
        let mut entry = node(10, NodeKind::Entry);
        entry.outputs.insert("next".into(), vec![id(11)]);
        let mut subscription = node(
            11,
            NodeKind::WaitPlayback {
                condition: asset::PlaybackCondition::SubscribeMarker {
                    timeline: timeline.id.to_string(),
                    marker: marker.to_string(),
                },
            },
        );
        subscription.inputs.insert(
            "playback".into(),
            Input::Literal {
                value_type: schema::Type::SequenceHandle,
                value: serde_json::Value::Null,
            },
        );
        subscription.outputs.insert("reached".into(), vec![id(12)]);
        let mut delay = node(12, NodeKind::Delay);
        delay.inputs.insert(
            "seconds".into(),
            Input::Literal {
                value_type: schema::Type::Fixed,
                value: json!(0.1),
            },
        );
        file.asset
            .functions
            .push(event(vec![entry, subscription, delay]));
        let mut native = registry();
        native.classes.get_mut(&id(1)).unwrap().source.file = root.join("assets/scripts/Enemy.hpp");
        let compiled = compile(&root, &native, &[file.clone()]).unwrap();
        assert!(compiled.artifacts.dependencies.contains(&path));
        crate::blueprint_dependencies::record(&root, &compiled).unwrap();
        let output = format!("generated-blueprint:{}", file.asset.id);
        let marker_dependency = format!("marker:{}:{marker}", timeline.id);
        assert!(
            compiled.footprints[&output]
                .1
                .contains_key(&marker_dependency)
        );
        let source = generated(&compiled);
        assert!(source.contains("epok::bp::PlaybackSubscription"));
        assert!(source.contains(".rearm();"));
        assert!(source.contains("epok_tasks.delay("));
        assert_eq!(
            source
                .matches("epok::bp::Continuations<8> epok_tasks;")
                .count(),
            1
        );
        timeline.markers.clear();
        std::fs::write(&path, serde_json::to_vec(&timeline).unwrap()).unwrap();
        crate::timeline_compile::observe_sources(&root).unwrap();
        let provenance = crate::artifact_dependencies::Graph::load(&root).unwrap();
        assert!(
            provenance.nodes[&output]
                .stale
                .contains_key(&marker_dependency)
        );
        assert!(provenance.nodes[&output].signature.is_some());
        let before = serde_json::to_vec(&file.asset).unwrap();
        let errors = compile(&root, &native, std::slice::from_ref(&file))
            .err()
            .unwrap();
        assert!(
            errors[0]
                .to_string()
                .contains("Missing timeline/marker reference")
        );
        assert_eq!(errors[0].node.as_ref(), Some(&id(11)));
        assert_eq!(before, serde_json::to_vec(&file.asset).unwrap());
    }
    #[test]
    fn blueprint_build_tickets_recheck_sources_and_membership_without_an_editor_watcher() {
        use crate::{
            artifact_dependencies::Graph as Dependencies,
            playback_staging::Batch,
            staging_files::{BuildTicket, Capture, Files},
        };
        let root = crate::workspace::tests::temp("blueprint-build-freshness");
        std::fs::create_dir_all(root.join("assets/Blueprints")).unwrap();
        let mut native = registry();
        native.classes.get_mut(&id(1)).unwrap().source.file = root.join("assets/scripts/Enemy.hpp");
        let mut file = asset();
        file.path = root.join("assets/Blueprints/Boss.epokbp");
        let save = |file: &AssetFile| {
            std::fs::write(&file.path, serde_json::to_vec(&file.asset).unwrap()).unwrap()
        };
        save(&file);
        let stage = |target: &str, file: &AssetFile| {
            let destination = root.join(target);
            let mut batch = Batch::new(&root, &destination).unwrap();
            let capture = Capture::begin(&destination).unwrap();
            let mut compiled = compile(&root, &native, std::slice::from_ref(file)).unwrap();
            compiled.artifacts.blueprint_footprints = compiled.footprints;
            compiled.artifacts.native_set = Some(
                crate::staging_files::native_set(
                    &root,
                    &crate::staging_files::native_files(&root).unwrap(),
                )
                .unwrap(),
            );
            compiled.artifacts.stage(&root, &destination).unwrap();
            batch.scripts(&compiled.artifacts).unwrap();
            batch.files(capture.finish()).unwrap();
            batch.publish(&root).unwrap();
            crate::staging_files::patch(&root, &destination, Files::new(), "release".into())
                .unwrap();
            destination
        };
        let build = stage(".epok/build", &file);
        let export = stage("exports/retained", &file);
        BuildTicket::begin(&root, &build)
            .unwrap()
            .complete(&root, &build, b"original")
            .unwrap();
        crate::staging_files::publish_export(&root, &export, Files::new()).unwrap();
        let pending = BuildTicket::begin(&root, &build).unwrap();
        file.asset.layout.positions.insert(id(10), [10., 20.]);
        save(&file);
        pending
            .complete(&root, &build, b"layout-independent")
            .unwrap();
        let before = Dependencies::load(&root).unwrap();
        let pending = BuildTicket::begin(&root, &build).unwrap();
        file.asset.defaults.insert(id(2), json!(150));
        save(&file);
        assert!(
            pending
                .complete(&root, &build, b"obsolete")
                .unwrap_err()
                .contains("changed during compilation")
        );
        let changed = Dependencies::load(&root).unwrap();
        let executable = "executable:.epok/build/epok.ps-exe";
        assert!(
            changed.nodes[executable]
                .stale
                .contains_key(&format!("blueprint:{}", file.asset.id))
        );
        assert_eq!(
            changed.nodes[executable].signature,
            before.nodes[executable].signature
        );
        assert!(crate::staging_files::publish_export(&root, &export, Files::new()).is_err());
        stage(".epok/build", &file);
        let current = Dependencies::load(&root).unwrap();
        let pending = BuildTicket::begin(&root, &build).unwrap();
        let mut added = file.clone();
        added.path = root.join("assets/Blueprints/Added.epokbp");
        added.asset.id = id(600);
        added.asset.name = "Added".into();
        save(&added);
        assert!(
            pending
                .complete(&root, &build, b"missing new factory")
                .is_err()
        );
        let expanded = Dependencies::load(&root).unwrap();
        assert!(
            expanded.nodes["stage:.epok/build"]
                .stale
                .contains_key("blueprint-sources")
        );
        let class = format!(
            "generated-script:.epok/build/scripts/generated/{}.hpp",
            file.asset.id
        );
        assert_eq!(
            expanded.nodes[&class], current.nodes[&class],
            "Adding a class should invalidate factory glue, not unrelated class code"
        );
        // Compare the early scene snapshot with the later compiler snapshot.
        let mut mixed = Batch::new(&root, &build).unwrap();
        mixed
            .scene_input(
                "blueprint-sources".into(),
                crate::blueprint_dependencies::source_set(std::slice::from_ref(&file)),
            )
            .unwrap();
        let compiled = compile(&root, &native, &[file.clone(), added.clone()]).unwrap();
        assert!(
            mixed
                .scripts(&compiled.artifacts)
                .unwrap_err()
                .contains("changed during staging")
        );
        std::fs::remove_file(&added.path).unwrap();
        assert!(BuildTicket::begin(&root, &build).is_err());
        stage(".epok/build", &file);
        for unreadable in [false, true] {
            let pending = BuildTicket::begin(&root, &build).unwrap();
            if unreadable {
                std::fs::write(&file.path, b"{").unwrap();
            } else {
                std::fs::remove_file(&file.path).unwrap();
            }
            assert!(pending.complete(&root, &build, b"invalid source").is_err());
            assert!(
                !Dependencies::load(&root).unwrap().nodes[executable]
                    .stale
                    .is_empty()
            );
            save(&file);
            assert!(BuildTicket::begin(&root, &build).is_err());
            stage(".epok/build", &file);
        }
        BuildTicket::begin(&root, &build)
            .unwrap()
            .complete(&root, &build, b"repaired")
            .unwrap();
        assert!(
            !Dependencies::load(&root).unwrap().nodes["export:exports/retained"]
                .stale
                .is_empty()
        );
        stage("exports/retained", &file);
        crate::staging_files::publish_export(&root, &export, Files::new()).unwrap();
    }

    #[test]
    fn blueprint_audio_selection_tracks_graph_clips_and_preserves_independent_banks() {
        use crate::{artifact_dependencies::Graph as Dependencies, playback_staging::Batch};
        let root = crate::workspace::tests::temp("blueprint-audio-selection");
        std::fs::create_dir_all(root.join("assets/blueprints")).unwrap();
        std::fs::write(
            root.join("assets/tone.wav"),
            crate::audio_import::test_wav(),
        )
        .unwrap();
        let clip = crate::assets::commit(
            crate::assets::prepare(
                &root,
                "assets/tone.wav",
                "assets/tone.epokasset",
                Default::default(),
                None,
                false,
            )
            .unwrap(),
        )
        .unwrap();
        let index = crate::assets::scan(&root, &mut Default::default());
        let mut native = registry();
        native.classes.get_mut(&id(1)).unwrap().source.file = root.join("assets/scripts/Enemy.hpp");
        let mut file = asset();
        file.path = root.join("assets/blueprints/Boss.epokbp");
        let mut entry = node(10, NodeKind::Entry);
        entry.outputs.insert("next".into(), vec![id(11)]);
        let mut set_clip = node(
            11,
            NodeKind::Builtin {
                operation: asset::Builtin::SetAudioClip,
            },
        );
        set_clip.outputs.insert("next".into(), vec![id(12)]);
        set_clip.inputs.insert(
            "target".into(),
            Input::Link {
                node: id(13),
                pin: "value".into(),
            },
        );
        let audio_type = schema::Type::AssetRef {
            kind: "AudioClip".into(),
        };
        file.asset.variables.push(asset::Variable {
            id: id(90),
            name: "strength".into(),
            value_type: schema::Type::Fixed,
            default: json!(1),
            editable: true,
            timeline_animatable: false,
        });
        set_clip.inputs.insert(
            "clip".into(),
            Input::Literal {
                value_type: audio_type.clone(),
                value: json!(null),
            },
        );
        let mut parent = node(12, NodeKind::CallParent);
        parent.inputs.insert(
            "amount".into(),
            Input::Literal {
                value_type: schema::Type::Fixed,
                value: json!(5),
            },
        );
        file.asset.functions.push(event(vec![
            entry,
            set_clip,
            parent,
            node(
                13,
                NodeKind::Builtin {
                    operation: asset::Builtin::SelfObject,
                },
            ),
        ]));
        let bank = |target: &str| format!("generated-resource:{target}/audio-bank.hh");
        let selection = format!("blueprint-audio:{}", file.asset.id);
        let observe = |file: &AssetFile| {
            crate::blueprint_dependencies::observe_sources(
                &root,
                std::slice::from_ref(file),
                Some(&native),
            )
            .unwrap();
            let compiled = compile(&root, &native, std::slice::from_ref(file)).unwrap();
            crate::scene_dependencies::observe_catalog(&root, Ok(&compiled.scripts)).unwrap();
        };
        let stage = |target: &str, file: &AssetFile| {
            let destination = root.join(target);
            let mut batch = Batch::new(&root, &destination).unwrap();
            let compiled = compile(&root, &native, std::slice::from_ref(file)).unwrap();
            crate::blueprint_dependencies::record(&root, &compiled).unwrap();
            batch.scene_catalog(&root, &compiled.scripts).unwrap();
            batch
                .blueprint_audio_source(file, Some(&compiled.registry))
                .unwrap();
            let template_scene = crate::scene::Scene {
                actors: crate::blueprint_templates::resolve(&[&file.asset.template])
                    .unwrap()
                    .actors
                    .into_iter()
                    .map(|item| item.entity)
                    .collect(),
                ..Default::default()
            };
            let referenced = crate::blueprint_refs::resources(
                std::slice::from_ref(file),
                std::slice::from_ref(&template_scene),
                &compiled.registry,
                &index,
                &[],
            )
            .unwrap();
            let selected = crate::scene_bank::resources(&[template_scene, referenced]);
            batch
                .resources(crate::audio::stage(&root, &selected, &destination, &index).unwrap())
                .unwrap();
            batch.audio_bank_inputs();
            batch.publish(&root).unwrap();
            let graph = Dependencies::load(&root).unwrap();
            let output = &graph.nodes[&bank(target)];
            assert!(output.stale.is_empty());
            assert!(output.dependencies.contains(&selection));
            assert!(output.dependencies.contains("audio-catalog"));
            assert!(!output.dependencies.contains("scene-catalog"));
            assert!(
                !output
                    .dependencies
                    .iter()
                    .any(|key| key.starts_with("blueprint:"))
            );
            assert_eq!(
                output.signature,
                Some(crate::assets::hash(
                    &std::fs::read(destination.join("audio-bank.hh")).unwrap()
                ))
            );
            graph
        };
        let empty = stage(".epok/build", &file);
        stage("exports/audio", &file);
        // An inherited native numeric default changes generated code, not clips.
        file.asset.defaults.insert(id(2), json!(140));
        observe(&file);
        for target in [".epok/build", "exports/audio"] {
            assert!(
                Dependencies::load(&root).unwrap().nodes[&bank(target)]
                    .stale
                    .is_empty()
            );
        }
        assert_eq!(
            stage(".epok/build", &file).nodes[&bank(".epok/build")],
            empty.nodes[&bank(".epok/build")]
        );
        file.asset.functions[0].nodes[2].inputs.insert(
            "amount".into(),
            Input::Literal {
                value_type: schema::Type::Fixed,
                value: json!(10),
            },
        );
        observe(&file);
        let logic = Dependencies::load(&root).unwrap();
        assert_eq!(
            logic.nodes[&bank(".epok/build")],
            empty.nodes[&bank(".epok/build")]
        );
        assert!(
            !logic.nodes[&format!("generated-blueprint:{}", file.asset.id)]
                .stale
                .is_empty()
        );
        assert_eq!(
            stage(".epok/build", &file).nodes[&bank(".epok/build")],
            empty.nodes[&bank(".epok/build")]
        );
        // Numeric declaration edits change generated C++ and the scene catalog,
        // but neither destination's actual audio bank selection.
        file.asset.variables[0].default = json!(2);
        file.asset.variables[0].timeline_animatable = true;
        observe(&file);
        for target in [".epok/build", "exports/audio"] {
            assert!(
                Dependencies::load(&root).unwrap().nodes[&bank(target)]
                    .stale
                    .is_empty()
            );
        }
        assert_eq!(
            stage(".epok/build", &file).nodes[&bank(".epok/build")],
            empty.nodes[&bank(".epok/build")]
        );
        // A clip declaration with a null default still affects which instance
        // overrides are selected. Its editor flags do not affect that contract.
        file.asset.variables.push(asset::Variable {
            id: id(91),
            name: "castSound".into(),
            value_type: audio_type.clone(),
            default: json!(null),
            editable: true,
            timeline_animatable: false,
        });
        observe(&file);
        for target in [".epok/build", "exports/audio"] {
            let graph = Dependencies::load(&root).unwrap();
            assert!(
                graph.nodes[&bank(target)]
                    .stale
                    .contains_key("audio-catalog")
            );
        }
        let null_clip = stage(".epok/build", &file);
        stage("exports/audio", &file);
        file.asset.variables[1].editable = false;
        observe(&file);
        assert_eq!(
            Dependencies::load(&root).unwrap().nodes[&bank(".epok/build")],
            null_clip.nodes[&bank(".epok/build")]
        );
        file.asset.variables[1].default = json!(clip);
        observe(&file);
        for target in [".epok/build", "exports/audio"] {
            assert!(
                Dependencies::load(&root).unwrap().nodes[&bank(target)]
                    .stale
                    .contains_key("audio-catalog")
            );
        }
        let declaration_clip = stage(".epok/build", &file);
        assert!(
            declaration_clip.nodes[&bank(".epok/build")]
                .dependencies
                .contains(&format!("asset:{clip}"))
        );
        assert!(
            !declaration_clip.nodes[&bank("exports/audio")]
                .stale
                .is_empty()
        );
        stage("exports/audio", &file);
        file.asset.variables.pop();
        observe(&file);
        let declaration_removed = stage(".epok/build", &file);
        assert_eq!(
            declaration_removed.nodes[&bank(".epok/build")].signature,
            empty.nodes[&bank(".epok/build")].signature
        );
        stage("exports/audio", &file);
        file.asset.functions[0].nodes[1].inputs.insert(
            "clip".into(),
            Input::Literal {
                value_type: audio_type.clone(),
                value: json!(clip),
            },
        );
        observe(&file);
        for target in [".epok/build", "exports/audio"] {
            assert!(
                Dependencies::load(&root).unwrap().nodes[&bank(target)]
                    .stale
                    .contains_key(&selection)
            );
        }
        let selected = stage(".epok/build", &file);
        assert!(
            selected.nodes[&bank(".epok/build")]
                .dependencies
                .contains(&format!("asset:{clip}"))
        );
        assert!(!selected.nodes[&bank("exports/audio")].stale.is_empty());
        stage("exports/audio", &file);
        // Moving an inline literal to a typed literal node preserves selection.
        file.asset.functions[0].nodes[1].inputs.insert(
            "clip".into(),
            Input::Link {
                node: id(14),
                pin: "value".into(),
            },
        );
        file.asset.functions[0].nodes.push(node(
            14,
            NodeKind::Literal {
                value_type: audio_type,
                value: json!(clip),
            },
        ));
        observe(&file);
        assert_eq!(
            Dependencies::load(&root).unwrap().nodes[&bank(".epok/build")],
            selected.nodes[&bank(".epok/build")]
        );
        if let NodeKind::Literal { value, .. } = &mut file.asset.functions[0].nodes[4].kind {
            *value = json!(null);
        }
        observe(&file);
        assert!(
            Dependencies::load(&root).unwrap().nodes[&bank(".epok/build")]
                .stale
                .contains_key(&selection)
        );
        let removed = stage(".epok/build", &file);
        assert_eq!(
            removed.nodes[&bank(".epok/build")].signature,
            empty.nodes[&bank(".epok/build")].signature
        );
        // Templates can contribute clips without any graph literal. Preserve
        // their source coverage when removing the broad Blueprint graph edge.
        file.asset.template = crate::blueprint_templates::Template::root("Sound");
        file.asset.template.actors[0].entity.audio = Some(crate::audio::AudioSource {
            clip: Some(clip),
            ..Default::default()
        });
        observe(&file);
        assert!(
            Dependencies::load(&root).unwrap().nodes[&bank(".epok/build")]
                .stale
                .contains_key(&selection)
        );
        assert!(
            stage(".epok/build", &file).nodes[&bank(".epok/build")]
                .dependencies
                .contains(&format!("asset:{clip}"))
        );
        let template_bank = stage(".epok/build", &file);
        stage("exports/audio", &file);
        crate::actor_components::sync(&mut file.asset.template.actors[0].entity);
        let scale_path = format!(
            "/components/{}/properties/scale",
            file.asset.template.actors[0].entity.root().unwrap().id
        );
        let audio_path = format!(
            "/components/{}",
            file.asset.template.actors[0]
                .entity
                .components
                .iter()
                .find(|c| c.class.class_id.as_deref()
                    == Some(crate::object_model::AUDIO_COMPONENT_ID))
                .unwrap()
                .id
        );
        let template_root = file.asset.template.actors[0].entity.id;
        file.asset
            .template
            .add_child(template_root, "Untextured visual");
        file.asset.template.actors[0].entity.position = [2., 3., 4.];
        file.asset.template.actors[0]
            .entity
            .audio
            .as_mut()
            .unwrap()
            .volume = 0.5;
        file.asset.template.construction.push(
            crate::blueprint_templates::ConstructionOp::SetColor {
                entity: template_root,
                color: [0.2, 0.3, 0.4],
            },
        );
        file.asset
            .template
            .overrides
            .entry(template_root)
            .or_default()
            .members
            .insert(scale_path.clone(), json!([2, 2, 2]));
        observe(&file);
        for target in [".epok/build", "exports/audio"] {
            assert!(
                Dependencies::load(&root).unwrap().nodes[&bank(target)]
                    .stale
                    .is_empty()
            );
        }
        assert_eq!(
            stage(".epok/build", &file).nodes[&bank(".epok/build")],
            template_bank.nodes[&bank(".epok/build")]
        );
        file.asset
            .template
            .overrides
            .get_mut(&template_root)
            .unwrap()
            .members
            .insert(audio_path.clone(), json!(null));
        observe(&file);
        for target in [".epok/build", "exports/audio"] {
            assert!(
                Dependencies::load(&root).unwrap().nodes[&bank(target)]
                    .stale
                    .contains_key(&selection)
            );
        }
        let cleared = stage(".epok/build", &file);
        assert_eq!(
            cleared.nodes[&bank(".epok/build")].signature,
            empty.nodes[&bank(".epok/build")].signature
        );
        assert!(!cleared.nodes[&bank("exports/audio")].stale.is_empty());
        file.asset
            .template
            .overrides
            .get_mut(&template_root)
            .unwrap()
            .members
            .remove(&audio_path);
        observe(&file);
        stage(".epok/build", &file);
        std::fs::write(&file.path, b"{").unwrap();
        assert!(crate::scripts::compile_blueprints(&root, &[]).is_err());
        assert!(
            Dependencies::load(&root).unwrap().nodes[&bank(".epok/build")]
                .stale
                .contains_key(&selection)
        );
        std::fs::write(&file.path, serde_json::to_vec(&file.asset).unwrap()).unwrap();
        observe(&file);
        assert!(
            !Dependencies::load(&root).unwrap().nodes[&bank(".epok/build")]
                .stale
                .is_empty()
        );
        stage(".epok/build", &file);
        let duplicate_path = root.join("assets/blueprints/Duplicate.epokbp");
        std::fs::copy(&file.path, &duplicate_path).unwrap();
        let duplicates = crate::blueprint_asset::load_all(&root).unwrap();
        crate::blueprint_dependencies::observe_sources(&root, &duplicates, Some(&native)).unwrap();
        assert!(
            Dependencies::load(&root).unwrap().nodes[&selection]
                .stale
                .values()
                .any(|message| message.contains("duplicate identity"))
        );
        assert!(
            Dependencies::load(&root).unwrap().nodes[&bank(".epok/build")]
                .stale
                .contains_key(&selection)
        );
        std::fs::remove_file(duplicate_path).unwrap();
        observe(&file);
        assert!(
            !Dependencies::load(&root).unwrap().nodes[&bank(".epok/build")]
                .stale
                .is_empty()
        );
        stage(".epok/build", &file);
        crate::blueprint_dependencies::observe_sources(&root, &[], Some(&native)).unwrap();
        assert!(
            Dependencies::load(&root).unwrap().nodes[&bank(".epok/build")]
                .stale
                .contains_key(&selection)
        );
    }

    #[test]
    fn inherited_audio_defaults_use_the_compilers_current_declarations() {
        let native = registry();
        let mut parent = asset();
        parent.asset.variables.push(asset::Variable {
            id: id(80),
            name: "sound".into(),
            value_type: schema::Type::AssetRef {
                kind: "AudioClip".into(),
            },
            default: json!(null),
            editable: true,
            timeline_animatable: false,
        });
        let mut child = asset();
        child.asset.id = id(81);
        child.asset.name = "Child".into();
        child.asset.parent = parent.asset.id.clone();
        child.asset.defaults.insert(id(2), json!(120));
        child.asset.defaults.insert(id(80), json!(null));
        let mut leaf = asset();
        leaf.asset.id = id(82);
        leaf.asset.name = "Leaf".into();
        leaf.asset.parent = child.asset.id.clone();
        leaf.asset.defaults.insert(id(80), json!(id(83)));
        let resolve = |child: &AssetFile, leaf: &AssetFile| {
            declaration_registry(&native, &[leaf.clone(), parent.clone(), child.clone()])
        };
        let first = resolve(&child, &leaf).unwrap();
        let signature = |file: &AssetFile, registry: &Registry| {
            crate::audio::blueprint_selection_signature(file, Some(registry))
        };
        let baseline = signature(&child, &first);
        child.asset.defaults.insert(id(2), json!(170));
        let numeric = resolve(&child, &leaf).unwrap();
        assert_eq!(signature(&child, &numeric), baseline);
        assert_eq!(
            numeric
                .properties("Child")
                .iter()
                .find(|p| p.id == id(2))
                .unwrap()
                .default,
            json!(170)
        );
        child.asset.defaults.insert(id(80), json!(id(84)));
        let selected = resolve(&child, &leaf).unwrap();
        assert_ne!(signature(&child, &selected), baseline);
        assert_eq!(
            selected
                .properties("Leaf")
                .iter()
                .find(|p| p.id == id(80))
                .unwrap()
                .default,
            json!(id(83))
        );
        child.asset.defaults.insert(id(80), json!(null));
        assert_eq!(
            signature(&child, &resolve(&child, &leaf).unwrap()),
            baseline
        );
        child.asset.defaults.insert(id(99), json!(1));
        assert!(
            resolve(&child, &leaf)
                .unwrap_err()
                .iter()
                .any(|d| d.message.contains("missing property"))
        );
        // Unknown declarations never remove a raw reference from observation.
        let invalid = crate::audio::blueprint_selection_signature(&child, None);
        child.asset.defaults.insert(id(99), json!(2));
        assert_ne!(
            invalid,
            crate::audio::blueprint_selection_signature(&child, None)
        );
        child.asset.defaults.remove(&id(99));
        leaf.asset.variables = parent.asset.variables.clone();
        assert!(
            resolve(&child, &leaf)
                .unwrap_err()
                .iter()
                .any(|d| d.message.contains("Duplicate"))
        );
    }

    #[test]
    fn imported_resource_revisions_invalidate_only_referencing_blueprints() {
        let root = crate::workspace::tests::temp("blueprint-resource-provenance");
        std::fs::create_dir_all(root.join("assets")).unwrap();
        let resource = uuid::Uuid::new_v4();
        let path = root.join("assets/Flash.epokasset");
        let write_texture = |red: u8| {
            let mut bytes = vec![];
            {
                let mut encoder = png::Encoder::new(&mut bytes, 1, 1);
                encoder.set_color(png::ColorType::Rgba);
                encoder.set_depth(png::BitDepth::Eight);
                encoder
                    .write_header()
                    .unwrap()
                    .write_image_data(&[red, 128, 0, 255])
                    .unwrap();
            }
            let package = crate::assets::Package {
                meta: crate::assets::Metadata {
                    version: 2,
                    id: resource,
                    kind: crate::assets::Kind::Texture,
                    importer_version: 1,
                    source: "assets/Flash.png".into(),
                    source_hash: crate::assets::hash(&bytes),
                    settings: crate::import_settings::Settings::Texture,
                    extra: Default::default(),
                },
                source: bytes,
            };
            std::fs::write(&path, package.bytes().unwrap()).unwrap();
        };
        write_texture(200);
        let mut file = asset();
        file.asset.variables.push(asset::Variable {
            id: id(700),
            name: "flash".into(),
            value_type: schema::Type::AssetRef {
                kind: "Texture".into(),
            },
            default: json!(resource),
            editable: true,
            timeline_animatable: false,
        });
        let mut independent = file.clone();
        independent.asset.id = id(701);
        independent.asset.name = "Independent".into();
        independent.asset.variables.clear();
        independent.path = root.join("assets/Independent.epokbp");
        let mut native = registry();
        native.classes.get_mut(&id(1)).unwrap().source.file = root.join("assets/scripts/Enemy.hpp");
        let compiled = compile(&root, &native, &[file.clone(), independent.clone()]).unwrap();
        assert!(compiled.artifacts.dependencies.contains(&path));
        crate::blueprint_dependencies::record(&root, &compiled).unwrap();
        let key = format!("generated-blueprint:{}", file.asset.id);
        let unrelated = format!("generated-blueprint:{}", independent.asset.id);
        let before = crate::artifact_dependencies::Graph::load(&root).unwrap();
        assert!(
            before.nodes[&key]
                .dependencies
                .contains(&format!("asset:{resource}"))
        );
        write_texture(100);
        let index = crate::assets::scan(&root, &mut Default::default());
        crate::timeline_compile::observe_resources(&root, &index).unwrap();
        let changed = crate::artifact_dependencies::Graph::load(&root).unwrap();
        assert!(!changed.nodes[&key].stale.is_empty());
        assert_eq!(changed.nodes[&key].signature, before.nodes[&key].signature);
        assert_eq!(changed.nodes[&unrelated], before.nodes[&unrelated]);
        std::fs::remove_file(path).unwrap();
        assert!(compile(&root, &native, &[file, independent]).is_err());
    }
    #[test]
    fn trace_collision_is_checked() {
        assert_ne!(trace_id(&id(1)), trace_id(&id(2)));
    }
    #[test]
    fn selected_class_references_invalidate_only_their_consumers_after_reparenting() {
        // Exercise defaults, inline inputs and literal nodes through the real
        // compiler, not a synthetic dependency footprint.
        for form in 0..3 {
            let root = crate::workspace::tests::temp("class-reference-provenance");
            std::fs::create_dir_all(root.join("assets")).unwrap();
            let mut native = registry();
            native.classes.get_mut(&id(1)).unwrap().source.file =
                root.join("assets/scripts/Enemy.hpp");
            let mut derived = native.classes[&id(1)].clone();
            derived.id = id(30);
            derived.cpp_name = "DerivedEnemy".into();
            derived.parent = Some(id(1));
            derived.properties.clear();
            derived.functions.clear();
            derived.source.file = root.join("assets/scripts/DerivedEnemy.hpp");
            native.classes.insert(derived.id.clone(), derived);
            let mut file = asset();
            let ty = schema::Type::ClassRef { base: id(1) };
            if form == 0 {
                file.asset.variables.push(asset::Variable {
                    id: id(31),
                    name: "selected_class".into(),
                    value_type: ty,
                    default: json!(id(30)),
                    editable: true,
                    timeline_animatable: false,
                });
            } else {
                let mut entry = node(10, NodeKind::Entry);
                entry.outputs.insert("next".into(), vec![id(11)]);
                let mut spawn = node(
                    11,
                    NodeKind::Builtin {
                        operation: asset::Builtin::SpawnClass { base: id(1) },
                    },
                );
                spawn.inputs.insert(
                    "parent".into(),
                    Input::Literal {
                        value_type: schema::Type::ObjectRef { class: None },
                        value: json!(null),
                    },
                );
                spawn.inputs.insert(
                    "class".into(),
                    if form == 1 {
                        Input::Literal {
                            value_type: ty.clone(),
                            value: json!(id(30)),
                        }
                    } else {
                        Input::Link {
                            node: id(12),
                            pin: "value".into(),
                        }
                    },
                );
                let mut nodes = vec![entry, spawn];
                if form == 2 {
                    nodes.push(node(
                        12,
                        NodeKind::Literal {
                            value_type: ty,
                            value: json!(id(30)),
                        },
                    ));
                }
                file.asset.functions.push(event(nodes));
            }
            let mut independent = asset();
            independent.asset.id = id(32);
            independent.asset.name = "Independent".into();
            independent.path = root.join("assets/Independent.epokbp");
            let compiled = compile(&root, &native, &[file.clone(), independent.clone()]).unwrap();
            crate::blueprint_dependencies::record(&root, &compiled).unwrap();
            let key = format!("generated-blueprint:{}", file.asset.id);
            let other = format!("generated-blueprint:{}", independent.asset.id);
            let choice = format!("blueprint-class:{}", id(30));
            let before = crate::artifact_dependencies::Graph::load(&root).unwrap();
            assert!(
                before.nodes[&key].dependencies.contains(&choice),
                "form {form}"
            );
            assert!(!before.nodes[&other].dependencies.contains(&choice));
            let mut changed = compiled.registry.clone();
            changed.classes.get_mut(&id(30)).unwrap().parent = None;
            crate::blueprint_dependencies::observe_reflection(&root, &changed).unwrap();
            let graph = crate::artifact_dependencies::Graph::load(&root).unwrap();
            assert!(graph.nodes[&key].stale.contains_key(&choice), "form {form}");
            assert_eq!(graph.nodes[&other], before.nodes[&other]);
            assert_eq!(graph.nodes[&key].signature, before.nodes[&key].signature);
            native.classes.get_mut(&id(30)).unwrap().parent = None;
            assert!(
                compile(&root, &native, std::slice::from_ref(&file)).is_err(),
                "form {form}"
            );
            native.classes.get_mut(&id(30)).unwrap().parent = Some(id(1));
            let repaired = compile(&root, &native, &[file, independent]).unwrap();
            crate::blueprint_dependencies::record(&root, &repaired).unwrap();
            assert!(
                crate::artifact_dependencies::Graph::load(&root)
                    .unwrap()
                    .nodes[&key]
                    .stale
                    .is_empty()
            );
        }
    }
    #[test]
    fn timeline_has_generation_guarded_updates_and_completion() {
        let mut f = asset();
        let mut entry = node(10, NodeKind::Entry);
        entry.outputs.insert("next".into(), vec![id(11)]);
        let timeline = node(
            11,
            NodeKind::Timeline {
                keys: vec![[0., 10.], [1., 20.]],
                looping: false,
                member: id(2),
            },
        );
        f.asset.functions.push(event(vec![entry, timeline]));
        let compiled = compile(Path::new(""), &registry(), &[f.clone()]).unwrap();
        let code = generated(&compiled);
        assert!(code.contains("Timeline<16>"));
        assert!(code.contains(".completed"));
        assert!(code.contains("epok_epoch_snapshot_"));
        assert!(code.contains("this->health=epok_sample_"));
        if let NodeKind::Timeline { keys, .. } = &mut f.asset.functions[0].nodes[1].kind {
            keys[1][0] = 0.00001;
        }
        assert!(error(f).contains("Q12-representable"));
    }
    #[test]
    fn builtin_spawn_is_typed_and_checks_class_existence() {
        let mut f = asset();
        let mut entry = node(10, NodeKind::Entry);
        entry.outputs.insert("next".into(), vec![id(11)]);
        let mut spawn = node(
            11,
            NodeKind::Builtin {
                operation: asset::Builtin::Spawn { class: id(1) },
            },
        );
        spawn.inputs.insert(
            "parent".into(),
            Input::Literal {
                value_type: schema::Type::ObjectRef { class: None },
                value: json!(null),
            },
        );
        f.asset.functions.push(event(vec![entry, spawn]));
        let result = compile(Path::new(""), &registry(), &[f.clone()]).unwrap();
        assert!(generated(&result).contains("epok::bp::spawn_actor("));
        if let NodeKind::Builtin {
            operation: asset::Builtin::Spawn { class },
        } = &mut f.asset.functions[0].nodes[1].kind
        {
            *class = id(90);
        }
        assert!(error(f).contains("Unknown class reference"));
    }
    #[test]
    fn receiver_calls_generate_checked_out_of_line_thunks_and_once_only_results() {
        let mut registry = registry();
        registry.classes.get_mut(&id(1)).unwrap().functions[0].returns = schema::Type::Fixed;
        let mut sdk = registry.classes[&id(1)].clone();
        sdk.id = id(90);
        sdk.cpp_name = "epok::EffectLayer".into();
        sdk.blueprintable = false;
        sdk.properties.clear();
        sdk.functions.clear();
        sdk.source.file = "runtime/effect_types.hpp".into();
        registry.classes.insert(sdk.id.clone(), sdk);
        let mut file = asset();
        let mut entry = node(10, NodeKind::Entry);
        entry.outputs.insert("next".into(), vec![id(11)]);
        let mut call = node(
            11,
            NodeKind::CallOn {
                class: id(1),
                function: id(3),
            },
        );
        call.inputs.insert(
            "__target".into(),
            Input::Literal {
                value_type: schema::Type::ObjectRef { class: Some(id(1)) },
                value: json!(null),
            },
        );
        call.inputs.insert(
            "amount".into(),
            Input::Literal {
                value_type: schema::Type::Fixed,
                value: json!(2),
            },
        );
        call.outputs.insert("next".into(), vec![id(12)]);
        let mut assign = node(12, NodeKind::SetVariable { member: id(2) });
        assign.inputs.insert(
            "value".into(),
            Input::Link {
                node: id(11),
                pin: "value".into(),
            },
        );
        let mut graph = event(vec![entry, call, assign]);
        graph.override_id = None;
        graph.name = "invoke_other".into();
        graph.parameters.clear();
        file.asset.functions.push(graph);
        let compiled = compile(Path::new(""), &registry, &[file.clone()]).unwrap();
        assert_eq!(
            compiled.artifacts.native_sources,
            vec![PathBuf::from("scripts/generated/blueprint_calls.cpp")]
        );
        let source = String::from_utf8_lossy(
            &compiled.artifacts.files[Path::new("scripts/generated/blueprint_calls.cpp")],
        );
        assert!(source.contains("epok::bp::is_a("));
        assert!(source.contains("static_cast<Enemy*>"));
        assert!(source.contains("DispatchScope"));
        assert!(source.contains("epok_receiver_depth<32"));
        let generated = generated(&compiled);
        assert_eq!(
            generated
                .matches(&format!("{}(", ir::call_on_name(&id(1), &id(3))))
                .count(),
            1
        );
        assert!(generated.contains("auto epok_receiver="));
        assert!(generated.contains("auto epok_argument_0="));
        registry.classes.get_mut(&id(1)).unwrap().functions[0].access = "protected".into();
        assert!(
            compile(Path::new(""), &registry, &[file]).err().unwrap()[0]
                .message
                .contains("public")
        );
    }
    #[test]
    fn vector_construction_component_and_dynamic_class_spawn_are_typed() {
        let mut file = asset();
        let mut entry = node(10, NodeKind::Entry);
        entry.outputs.insert("next".into(), vec![id(11)]);
        let mut set = node(11, NodeKind::SetVariable { member: id(2) });
        set.inputs.insert(
            "value".into(),
            Input::Link {
                node: id(12),
                pin: "value".into(),
            },
        );
        let mut component = node(
            12,
            NodeKind::VectorComponent {
                length: 3,
                index: 1,
            },
        );
        component.inputs.insert(
            "value".into(),
            Input::Link {
                node: id(13),
                pin: "value".into(),
            },
        );
        let mut vector = node(13, NodeKind::MakeVector { length: 3 });
        for name in ["x", "y", "z"] {
            vector.inputs.insert(
                name.into(),
                Input::Literal {
                    value_type: schema::Type::Fixed,
                    value: json!(2),
                },
            );
        }
        set.outputs.insert("next".into(), vec![id(14)]);
        let mut spawn = node(
            14,
            NodeKind::Builtin {
                operation: crate::blueprint_asset::Builtin::SpawnClass { base: id(1) },
            },
        );
        spawn.inputs.insert(
            "class".into(),
            Input::Literal {
                value_type: schema::Type::ClassRef { base: id(1) },
                value: json!(id(1)),
            },
        );
        spawn.inputs.insert(
            "parent".into(),
            Input::Literal {
                value_type: schema::Type::ObjectRef { class: None },
                value: json!(null),
            },
        );
        file.asset
            .functions
            .push(event(vec![entry, set, component, vector, spawn]));
        let compiled = compile(Path::new(""), &registry(), &[file.clone()]).unwrap();
        let cpp = generated(&compiled);
        assert!(cpp.contains("Vector<3>{{"));
        assert!(cpp.contains("})[1]"));
        assert!(cpp.contains("epok::bp::spawn_actor("));
        if let NodeKind::VectorComponent { index, .. } = &mut file.asset.functions[0].nodes[2].kind
        {
            *index = 3;
        }
        assert!(error(file).contains("index"));
    }
}
