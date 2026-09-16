//! The generated Lua API definition file.
//!
//! Every reflected class — C++, Blueprint or Lua — and the whole `epok` builtin
//! namespace are rendered as Lua Language Server (EmmyLua/LuaLS) annotations so
//! a Lua author gets completion and type information for the current project
//! without writing or maintaining anything. It is the editor's equivalent of a
//! compiled language importing its own types.
//!
//! The file is an editor artifact only: nothing in the engine, the compiler or
//! the cooked build ever reads it, and deleting it changes no behaviour. It is
//! rewritten from the registry on every catalog refresh, so it is never
//! authored and never merged.
use crate::{
    blueprint::Registry,
    reflection_schema::{self as schema, Type},
    script_ir::Intrinsic,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write,
    path::Path,
};

/// Project-relative location of the generated definitions.
pub const STUB_PATH: &str = ".epok/lua/epok.d.lua";
/// Project-relative Lua Language Server configuration. Written only when the
/// project has none: an author's own configuration is never overwritten.
pub const CONFIG_PATH: &str = ".luarc.json";

/// The Lua Language Server settings a project needs for the generated
/// definitions to be *enforced* rather than merely offered.
///
/// `workspace.library` loads the definitions and `diagnostics.globals` declares
/// the one global they introduce. The severities matter as much: LuaLS reports
/// a type mismatch against a `---@field` or a `---@param` only as a Warning by
/// default, which an editor shows as a hint an author can ignore. Raising them
/// with the `!` suffix — "this severity, whatever the check level" — is what
/// makes VS Code flag a wrong assignment, a wrong argument or an undeclared
/// member as an error, before the Epok compiler is ever run. The same three
/// constructs are rejected by the compiler with its own diagnostics, so the two
/// agree by construction.
const CONFIG: &str = r#"{
  "runtime.version": "Lua 5.2",
  "workspace.library": [".epok/lua"],
  "diagnostics.globals": ["epok"],
  "diagnostics.severity": {
    "assign-type-mismatch": "Error!",
    "param-type-mismatch": "Error!",
    "return-type-mismatch": "Error!",
    "cast-local-type": "Error!",
    "undefined-field": "Error!",
    "undefined-global": "Error!",
    "inject-field": "Error!"
  }
}
"#;

/// `epok::Actor3D` is not a legal Lua type name, and LuaLS accepts a dotted one,
/// so every reflected class is named with `.` where C++ writes `::`. The mapping
/// is total and reversible: no other character of a `cpp_name` is touched.
fn lua_name(cpp_name: &str) -> String {
    cpp_name.replace("::", ".")
}

/// The file-local variable methods of a class are declared on. Locals never
/// leave the definition file, so the generated names cannot collide with a
/// global an author relies on.
fn local_name(cpp_name: &str, taken: &mut BTreeSet<String>) -> String {
    let base = cpp_name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>();
    let mut name = format!("_{base}");
    let mut suffix = 2;
    while !taken.insert(name.clone()) {
        name = format!("_{base}_{suffix}");
        suffix += 1;
    }
    name
}

/// A named class operand of a typed reference: the wire form carries either the
/// class uuid or its `cpp_name`, and the stub always shows the name.
fn referenced(registry: &Registry, class: Option<&String>) -> Option<String> {
    let class = class?;
    let resolved = registry
        .classes
        .get(class)
        .or_else(|| registry.named(class))?;
    Some(lua_name(&resolved.cpp_name))
}

/// Enum and record types named anywhere in the rendered surface, so each one is
/// declared exactly once above the classes that use it.
#[derive(Default)]
struct Named {
    enums: BTreeMap<String, BTreeMap<String, i64>>,
    records: BTreeMap<String, Vec<schema::RecordField>>,
}

/// The LuaLS spelling of a reflected type.
fn lua_type(registry: &Registry, ty: &Type, named: &mut Named) -> String {
    let reference = |kind: &str, class: Option<&String>| match referenced(registry, class) {
        Some(name) => format!("{kind}<{name}>"),
        None => kind.to_string(),
    };
    match ty {
        Type::Void => "nil".into(),
        Type::Bool => "Bool".into(),
        Type::Int32 => "Int32".into(),
        Type::UInt32 => "UInt32".into(),
        Type::Fixed => "Fixed".into(),
        Type::Enum { cpp_name, variants } => {
            named.enums.insert(lua_name(cpp_name), variants.clone());
            lua_name(cpp_name)
        }
        Type::Vector { length: 2 } => "Vector2".into(),
        Type::Vector { length: 3 } => "Vector3".into(),
        Type::Vector { .. } => "Fixed[]".into(),
        Type::Record { cpp_name, .. } => {
            named.records.insert(lua_name(cpp_name), ty.members());
            lua_name(cpp_name)
        }
        Type::EffectLayerRef { .. } => "EffectLayerRef".into(),
        Type::SequenceHandle => "SequenceHandle".into(),
        Type::EffectHandle => "EffectHandle".into(),
        Type::AssetRef { .. } => "AssetRef".into(),
        Type::ClassRef { .. } => "ClassRef".into(),
        Type::ObjectRef { class } => reference("ObjectRef", class.as_ref()),
        Type::ActorRef { class } => reference("ActorRef", class.as_ref()),
        Type::ComponentRef { class } => reference("ComponentRef", class.as_ref()),
    }
}

/// The type of the object's own reference, by the family it resolves to.
fn self_reference(registry: &Registry, cpp_name: &str) -> String {
    let name = lua_name(cpp_name);
    match crate::object_model::resolved_shape(registry, cpp_name).0 {
        schema::ClassFamily::Actor => format!("ActorRef<{name}>"),
        schema::ClassFamily::Component => format!("ComponentRef<{name}>"),
        _ => format!("ObjectRef<{name}>"),
    }
}

/// A Lua parameter name for a reflected parameter, including the unnamed ones
/// a C++ declaration is allowed to leave anonymous.
fn parameter_name(name: &str, index: usize) -> String {
    let cleaned = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>();
    if cleaned.is_empty() || cleaned.starts_with(|c: char| c.is_ascii_digit()) {
        format!("arg{}", index + 1)
    } else {
        cleaned
    }
}

/// Every reflected definition of one project, as LuaLS annotations. The output
/// is sorted by class name and depends on nothing but the registry, so two
/// renders of the same registry are byte-identical.
pub fn render(registry: &Registry) -> String {
    let mut named = Named::default();
    let mut classes = String::new();
    let mut taken = BTreeSet::new();
    // Lua type name -> the file-local table that carries it, so every class can
    // also be named as a *value*: `epok.Actor3D:extend()` and `EnemyBase:extend()`
    // both read the binding rendered below.
    let mut bindings: Vec<(String, String)> = vec![];
    let mut sorted = registry.classes.values().collect::<Vec<_>>();
    sorted.sort_by(|a, b| a.cpp_name.cmp(&b.cpp_name).then(a.id.cmp(&b.id)));
    for class in sorted {
        let name = lua_name(&class.cpp_name);
        let parent = class
            .parent
            .as_ref()
            .and_then(|id| registry.classes.get(id))
            .map(|parent| lua_name(&parent.cpp_name));
        let local = local_name(&class.cpp_name, &mut taken);
        let _ = writeln!(classes, "-- {} ({})", class.cpp_name, class.provider.id);
        match &parent {
            Some(parent) => {
                let _ = writeln!(classes, "---@class {name} : {parent}");
            }
            None => {
                let _ = writeln!(classes, "---@class {name}");
            }
        }
        // Intrinsic places are not declared by any class: a spatial class has
        // the root transform and a UI class the rect, exactly as the compiler
        // resolves them for `self.position.x` and `self.rect_size.y`.
        let mut fields = BTreeSet::from(["ref".to_owned()]);
        if let Some(access) = crate::lua_vm::intrinsic_access(registry, &class.cpp_name) {
            for kind in Intrinsic::ALL {
                let components = crate::lua_vm::intrinsic_components(access.domain, kind);
                let ty = match components {
                    0 => continue,
                    1 => "Fixed",
                    2 => "Vector2",
                    _ => "Vector3",
                };
                fields.insert(kind.name().to_owned());
                let _ = writeln!(
                    classes,
                    "---@field {} {ty} Intrinsic place; components are readable and writable.",
                    kind.name()
                );
            }
        }
        let _ = writeln!(
            classes,
            "---@field ref {} This object's own typed reference.",
            self_reference(registry, &class.cpp_name)
        );
        // Only a Lua class has a qualified parent receiver: `Cube.super.tick`
        // is the declaration form's replacement for a dynamic `super` value.
        if class.provider == crate::lua_asset::provider()
            && let Some(parent) = &parent
        {
            fields.insert("super".to_owned());
            let _ = writeln!(
                classes,
                "---@field super {parent} The parent implementation: `{name}.super.tick(self, delta_seconds)`."
            );
        }
        for property in &class.properties {
            // `ref` and the intrinsic places are reserved names: a property of
            // the same name is addressed as the intrinsic, so it is declared once.
            if !fields.insert(property.name.clone()) {
                continue;
            }
            let _ = writeln!(
                classes,
                "---@field {} {}{}",
                property.name,
                lua_type(registry, &property.value_type, &mut named),
                if property.editable {
                    " Editable in the Inspector."
                } else {
                    ""
                }
            );
        }
        let _ = writeln!(classes, "local {local} = {{}}\n");
        // The declaration form: `local Cube = epok.Actor3D:extend()`. It is
        // declared once on the hierarchy root and inherited by every class, and
        // it is generic in the receiver so the result is the parent's own type
        // until the `---@class` line above the local names the new one.
        if class.parent.is_none() {
            let _ = writeln!(
                classes,
                "--- Declares a subclass of this class. `local Cube = epok.Actor3D:extend()`\n\
                 ---@generic T\n\
                 ---@param self T\n\
                 ---@return T\n\
                 function {local}:extend() end\n"
            );
        }
        bindings.push((lua_name(&class.cpp_name), local.clone()));
        let mut functions = class
            .functions
            .iter()
            .filter(|f| f.callable || f.event)
            .collect::<Vec<_>>();
        functions.sort_by(|a, b| a.name.cmp(&b.name).then(a.id.cmp(&b.id)));
        for function in functions {
            if function.event {
                let _ = writeln!(
                    classes,
                    "--- Event: declare `function {local}:{}(...)` in a subclass to override it.",
                    function.name
                );
            }
            let mut parameters = vec![];
            for (index, parameter) in function.parameters.iter().enumerate() {
                let name = parameter_name(&parameter.name, index);
                let _ = writeln!(
                    classes,
                    "---@param {name} {}",
                    lua_type(registry, &parameter.value_type, &mut named)
                );
                parameters.push(name);
            }
            if function.returns != Type::Void {
                let _ = writeln!(
                    classes,
                    "---@return {}",
                    lua_type(registry, &function.returns, &mut named)
                );
            }
            let _ = writeln!(
                classes,
                "function {local}:{}({}) end\n",
                function.name,
                parameters.join(", ")
            );
        }
    }

    // Gameplay service completion is generated from the same operation
    // catalog consumed by Blueprint and the compiler. No handwritten stub can
    // advertise a signature the frontend does not know.
    let mut services = String::new();
    let mut namespaces = BTreeSet::new();
    let mut operations = registry
        .operations
        .values()
        .filter(|operation| matches!(operation.receiver, schema::ReceiverKind::Service { .. }))
        .collect::<Vec<_>>();
    operations.sort_by(|a, b| {
        a.category
            .cmp(&b.category)
            .then(a.name.cmp(&b.name))
            .then(a.id.cmp(&b.id))
    });
    for operation in operations {
        let namespace = operation
            .category
            .chars()
            .map(|character| {
                if character.is_ascii_alphanumeric() {
                    character.to_ascii_lowercase()
                } else {
                    '_'
                }
            })
            .collect::<String>();
        if crate::scripts::identifier(&namespace) && namespaces.insert(namespace.clone()) {
            let _ = writeln!(services, "epok.{namespace} = {{}}");
        }
        let _ = writeln!(
            services,
            "--- {} Complexity: {}. Failure: {}",
            operation.category, operation.complexity, operation.error_contract
        );
        let mut parameters = Vec::new();
        for (index, parameter) in operation.parameters.iter().enumerate() {
            let name = parameter_name(&parameter.name, index);
            let _ = writeln!(
                services,
                "---@param {name} {}",
                lua_type(registry, &parameter.value_type, &mut named)
            );
            parameters.push(name);
        }
        if let Some(output) = operation.outputs.first() {
            let _ = writeln!(
                services,
                "---@return {}",
                lua_type(registry, &output.value_type, &mut named)
            );
        }
        let _ = writeln!(
            services,
            "function epok.{namespace}.{}({}) end\n",
            operation.name,
            parameters.join(", ")
        );
    }

    // A record field may name a further record or enum, so rendering continues
    // until the set of named types stops growing.
    let mut records = BTreeMap::new();
    loop {
        let pending = named
            .records
            .iter()
            .filter(|(name, _)| !records.contains_key(*name))
            .map(|(name, fields)| (name.clone(), fields.clone()))
            .collect::<Vec<_>>();
        if pending.is_empty() {
            break;
        }
        for (name, fields) in pending {
            let mut body = format!("---@class {name}\n");
            for field in fields {
                let _ = writeln!(
                    body,
                    "---@field {} {}",
                    field.name,
                    lua_type(registry, &field.value_type, &mut named)
                );
            }
            records.insert(name, body);
        }
    }

    let mut out = String::new();
    out.push_str(HEADER);
    out.push_str(PRIMITIVES);
    for (name, variants) in &named.enums {
        let mut sorted = variants.iter().collect::<Vec<_>>();
        sorted.sort_by(|a, b| a.1.cmp(b.1).then(a.0.cmp(b.0)));
        let _ = writeln!(out, "---@alias {name}");
        for (variant, value) in sorted {
            let _ = writeln!(out, "---| {value} # {variant}");
        }
        out.push('\n');
    }
    for body in records.values() {
        out.push_str(body);
        out.push('\n');
    }
    out.push_str(&classes);
    out.push_str(NAMESPACE);
    out.push_str("\n-- Catalog-generated Gameplay v2 services.\n");
    out.push_str(&services);
    out.push_str(&namespace_bindings(&bindings));
    out
}

/// The class values. A declaration names its parent as a value, so every
/// reflected class is bound under the very name its Lua type carries:
/// `epok.Actor3D` under the `epok` namespace, `EnemyBase` as a global and
/// `game::Enemy` as a field of a `game` table this file declares.
fn namespace_bindings(bindings: &[(String, String)]) -> String {
    let mut out = String::from(
        "\n-- Class values. A declaration names its parent as a value, so each\n         -- reflected class is bound under the name its Lua type already carries.\n",
    );
    let mut namespaces = BTreeSet::new();
    for (name, _) in bindings {
        let segments = name.split('.').collect::<Vec<_>>();
        let mut prefix = String::new();
        for segment in &segments[..segments.len() - 1] {
            prefix = if prefix.is_empty() {
                (*segment).to_owned()
            } else {
                format!("{prefix}.{segment}")
            };
            namespaces.insert(prefix.clone());
        }
    }
    // `epok` is declared by the namespace section above; every other namespace
    // root a project introduces is declared here, outermost first.
    for namespace in &namespaces {
        if namespace == "epok" || namespace.starts_with("epok.") {
            continue;
        }
        let _ = writeln!(out, "{namespace} = {{}}");
    }
    for (name, local) in bindings {
        let _ = writeln!(out, "{name} = {local}");
    }
    out
}

/// Write the definitions and, for a project that has none, a Lua Language
/// Server configuration pointing at them. Unchanged content is not rewritten.
pub fn write(root: &Path, registry: &Registry) -> Result<(), String> {
    crate::project::write_changed(&root.join(STUB_PATH), render(registry).as_bytes())?;
    let config = root.join(CONFIG_PATH);
    if !config.exists() {
        crate::project::write_changed(&config, CONFIG.as_bytes())?;
    }
    Ok(())
}

const HEADER: &str = "\
---@meta
-- Generated by the epok editor. Do not edit: every script catalog refresh
-- rewrites this file from the current C++, Blueprint and Lua declarations.
--
-- Nothing in the engine reads it. It exists so a Lua Language Server client
-- (VS Code, Neovim, any LuaLS host) can complete and type-check `.lua` classes
-- against the very declarations the compiler uses.
--
-- A C++ class name maps to its Lua type name by writing `.` where C++ writes
-- `::`, so `epok::Actor3D` is the type `epok.Actor3D`. Methods are declared on
-- a file-local table, so this file introduces no global but `epok`.

";

const PRIMITIVES: &str = "\
-- The epok value vocabulary. `Fixed` is the Q12 fixed-point scalar; it is a
-- Lua number here because LuaLS has no fixed-point kind of its own.
---@alias Fixed number
---@alias Int32 integer
---@alias UInt32 integer
---@alias Bool boolean

---@class Vector2
---@field x Fixed
---@field y Fixed

---@class Vector3
---@field x Fixed
---@field y Fixed
---@field z Fixed

-- Persistent authoring identities. A reference is opaque: it is passed to a
-- builtin or a reflected call, never decomposed.
---@class ObjectRef<T>
---@class ActorRef<T>
---@class ComponentRef<T>

-- 64-bit native handles. They are declarable and Inspector-editable, and in a
-- body they may only be named as the direct operand of a builtin.
---@class AssetRef
---@class ClassRef

-- Playback handles. They are never values in the epok-lua profile.
---@class SequenceHandle
---@class EffectHandle
---@class EffectLayerRef

";

const NAMESPACE: &str = "\
-- The `epok` namespace: the declaration entry point and every builtin of the
-- epok-lua v1 profile. Each one is a statically resolved call with a fixed
-- arity, lowered to the same `epok::bp::api` entry point the matching Blueprint
-- node calls.
epok = {}
epok.input = {}

-- Property value constructors. They appear only at file scope, as the value of
-- a `<Class>.<name> = ...` declaration, and are read from the source text: a
-- property whose type and default a bare literal can express needs none of
-- them, so `Cube.speed = 90.0` is a Fixed and `Cube.lives = 3` an Int32.

---@param value boolean
---@return Bool
function epok.Bool(value) end

---@param value number
---@return Int32
function epok.Int32(value) end

---@param value number
---@return UInt32
function epok.UInt32(value) end

---@param value number
---@return Fixed
function epok.Fixed(value) end

---@param x number
---@param y number
---@return Vector2
function epok.Vector2(x, y) end

---@param x number
---@param y number
---@param z number
---@return Vector3
function epok.Vector3(x, y, z) end

--- A reflected enum and the variant the property starts on, written unquoted
--- and quoted respectively: `Cube.mode = epok.Enum(Mode, \"Idle\")`.
---@param enumeration any
---@param variant string
---@return integer
function epok.Enum(enumeration, variant) end

--- An actor reference, optionally narrowed to a class written unquoted.
---@param class? any
---@return ActorRef
function epok.ActorRef(class) end

---@param class? any
---@return ComponentRef
function epok.ComponentRef(class) end

---@param class? any
---@return ObjectRef
function epok.ObjectRef(class) end

--- An asset reference of one imported asset kind, such as \"Mesh\".
---@param kind string
---@return AssetRef
function epok.AssetRef(kind) end

--- A class reference narrowed to a base class written unquoted.
---@param base any
---@return ClassRef
function epok.ClassRef(base) end

--- The same value, kept out of the Inspector: `Cube.hidden = epok.Hidden(1.0)`.
---@generic T
---@param value T
---@return T
function epok.Hidden(value) end

---@param value Int32
---@return Fixed
function epok.to_fixed(value) end

---@param value Fixed
---@return Int32
function epok.to_int(value) end

--- True while the button is held. `port` is 0 or 1; a button index of 16 or
--- more always reads false.
---@param button UInt32
---@param port UInt32
---@return Bool
function epok.input.held(button, port) end

---@param button UInt32
---@param port UInt32
---@return Bool
function epok.input.pressed(button, port) end

---@param button UInt32
---@param port UInt32
---@return Bool
function epok.input.released(button, port) end

---@param index UInt32
---@return Bool
function epok.request_scene(index) end

---@param reference ObjectRef|ActorRef|ComponentRef
---@return Bool
function epok.is_valid(reference) end

---@param reference ObjectRef|ActorRef|ComponentRef
---@param class string
---@return Bool
function epok.is_a(reference, class) end

--- The reference narrowed to `class`, or null when it is not that class.
---@param reference ObjectRef|ActorRef|ComponentRef
---@param class string
---@return ObjectRef|ActorRef|ComponentRef
function epok.cast(reference, class) end

--- Creates an instance of a concrete, spawnable actor class. The spawn is
--- queued, so the returned reference is not live until the current batch ends.
---@param class string
---@param parent? ActorRef
---@return ActorRef
function epok.spawn(class, parent) end

--- Creates an instance of the class held by a declared `ClassRef` property.
---@param class ClassRef
---@param parent? ActorRef
---@return ActorRef
function epok.spawn_class(class, parent) end

--- The actor that owns this component. Component classes only.
---@return ActorRef
function epok.owner() end

---@param reference ComponentRef
function epok.play_audio(reference) end

---@param reference ComponentRef
function epok.stop_audio(reference) end

---@param reference ComponentRef
---@param texture AssetRef
function epok.set_texture(reference, texture) end

---@param reference ComponentRef
---@param clip AssetRef
function epok.set_audio_clip(reference, clip) end

--- Statement only: the playback handle stays native.
---@param reference ComponentRef
function epok.play_sequence(reference) end

--- Statement only: the playback handle stays native.
---@param reference ComponentRef
function epok.play_effect(reference) end
";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        object_model,
        reflection_schema::{
            ClassFamily, ComponentContract, Direction, Domain, Function, Location, Parameter,
            Property,
        },
    };

    fn class(id: &str, cpp_name: &str, parent: Option<&str>) -> schema::Class {
        schema::Class {
            id: id.into(),
            provider: schema::native_provider(),
            backend: schema::native_backend(),
            cpp_name: cpp_name.into(),
            parent: parent.map(str::to_owned),
            abstract_class: false,
            final_class: false,
            blueprintable: true,
            timeline_component: None,
            family: None,
            domain: None,
            placement: Default::default(),
            component: None,
            default_components: vec![],
            explicit_abstract: false,
            properties: vec![],
            functions: vec![],
            source: schema::Location {
                file: "assets/scripts/Probe.hpp".into(),
                line: 1,
                column: 1,
            },
        }
    }
    fn location() -> Location {
        Location {
            file: "assets/scripts/Probe.hpp".into(),
            line: 1,
            column: 1,
        }
    }
    fn property(name: &str, value_type: Type) -> Property {
        Property {
            id: format!("property:{name}"),
            name: name.into(),
            value_type,
            default: serde_json::Value::Null,
            editable: true,
            timeline: None,
            source: location(),
        }
    }
    fn function(name: &str, parameters: Vec<(&str, Type)>, returns: Type) -> Function {
        Function {
            id: format!("function:{name}"),
            name: name.into(),
            parameters: parameters
                .into_iter()
                .map(|(name, value_type)| Parameter {
                    name: name.into(),
                    value_type,
                    direction: Direction::Value,
                })
                .collect(),
            returns,
            callable: true,
            timeline: None,
            event: false,
            pure: false,
            resource_demands: vec![],
            abstract_method: false,
            final_method: false,
            access: "public".into(),
            overrides: vec![],
            source: location(),
        }
    }

    /// A C++ base chain, one Lua class and one Blueprint class, plus a UI actor
    /// that owns no world transform.
    fn registry() -> Registry {
        let mut object = class(object_model::OBJECT_ID, "epok::Object", None);
        object.family = Some(ClassFamily::Object);
        object.functions.push({
            let mut tick = function("tick", vec![("delta_seconds", Type::Fixed)], Type::Void);
            tick.event = true;
            tick.callable = false;
            tick
        });
        let mut actor = class(
            object_model::ACTOR_ID,
            "epok::Actor",
            Some(object_model::OBJECT_ID),
        );
        actor.family = Some(ClassFamily::Actor);
        actor.domain = Some(Domain::None);
        actor.functions.push(function(
            "set_active",
            vec![("active", Type::Bool)],
            Type::Void,
        ));
        let mut actor3d = class(
            object_model::ACTOR3D_ID,
            "epok::Actor3D",
            Some(object_model::ACTOR_ID),
        );
        actor3d.domain = Some(Domain::World3D);
        actor3d.properties.push(property("visible", Type::Bool));
        let mut ui = class("ui-actor", "epok::ActorUI", Some(object_model::ACTOR_ID));
        ui.domain = Some(Domain::UI);
        let mut component = class(
            object_model::ACTOR_COMPONENT_ID,
            "epok::ActorComponent",
            Some(object_model::OBJECT_ID),
        );
        component.family = Some(ClassFamily::Component);
        component.domain = Some(Domain::None);
        component.component = Some(ComponentContract::default());

        let mut lua = class("lua-spinner", "Spinner", Some(object_model::ACTOR3D_ID));
        lua.provider = crate::lua_asset::provider();
        lua.properties.push(property("speed", Type::Fixed));
        lua.functions.push(function(
            "aim",
            vec![(
                "target",
                Type::ActorRef {
                    class: Some(object_model::ACTOR3D_ID.into()),
                },
            )],
            Type::Bool,
        ));
        let mut blueprint = class("bp-turret", "Turret", Some("lua-spinner"));
        blueprint.provider = crate::script_backend::blueprint_provider();
        blueprint
            .properties
            .push(property("aim_offset", Type::Vector { length: 3 }));

        let mut registry = Registry::new();
        for c in [object, actor, actor3d, ui, component, lua, blueprint] {
            registry.classes.insert(c.id.clone(), c);
        }
        registry
    }

    /// The body of one `---@class` block, up to its `local` declaration.
    fn block<'a>(stub: &'a str, name: &str) -> &'a str {
        let start = stub
            .find(&format!("---@class {name}\n"))
            .or_else(|| stub.find(&format!("---@class {name} :")))
            .unwrap_or_else(|| panic!("{name} is not declared:\n{stub}"));
        let rest = &stub[start..];
        &rest[..rest.find("\nlocal ").expect("a class declares a local")]
    }

    #[test]
    fn every_provider_is_rendered_with_its_chain_members_and_intrinsics() {
        let stub = render(&registry());
        assert!(stub.starts_with("---@meta\n"), "{stub}");
        // The C++ chain, the Lua class and the Blueprint class, each naming its
        // parent by the mapped name. Inherited members arrive through the chain.
        assert!(
            stub.contains("---@class epok.Actor : epok.Object\n"),
            "{stub}"
        );
        assert!(
            stub.contains("---@class epok.Actor3D : epok.Actor\n"),
            "{stub}"
        );
        assert!(
            stub.contains("---@class Spinner : epok.Actor3D\n"),
            "{stub}"
        );
        assert!(stub.contains("---@class Turret : Spinner\n"), "{stub}");
        assert!(stub.contains("---@class epok.Object\n"), "{stub}");

        // Own properties only, typed in the epok vocabulary.
        let spinner = block(&stub, "Spinner");
        assert!(spinner.contains("---@field speed Fixed"), "{spinner}");
        assert!(!spinner.contains("visible"), "{spinner}");
        assert!(
            block(&stub, "Turret").contains("---@field aim_offset Vector3"),
            "{stub}"
        );

        // World3D intrinsics reach the Lua class through its parent; a UI actor
        // has the rect places instead, and a familyless class has none.
        for field in [
            "---@field position Vector3",
            "---@field rotation Vector3",
            "---@field scale Vector3",
        ] {
            assert!(spinner.contains(field), "{spinner}");
        }
        assert!(!spinner.contains("rect_position"), "{spinner}");
        let ui = block(&stub, "epok.ActorUI");
        assert!(ui.contains("---@field rect_position Vector2"), "{ui}");
        assert!(ui.contains("---@field rect_size Vector2"), "{ui}");
        assert!(!ui.contains("---@field position"), "{ui}");
        let object = block(&stub, "epok.Object");
        assert!(!object.contains("---@field position"), "{object}");
        assert!(!object.contains("---@field rect_position"), "{object}");

        // The object's own reference is typed by its family.
        assert!(
            spinner.contains("---@field ref ActorRef<Spinner>"),
            "{spinner}"
        );
        assert!(
            block(&stub, "epok.ActorComponent").contains("---@field ref ComponentRef<"),
            "{stub}"
        );
        assert!(
            object.contains("---@field ref ObjectRef<epok.Object>"),
            "{object}"
        );

        // Signatures: parameters, returns, a resolved reference class, and the
        // overridable event marked as one.
        assert!(
            stub.contains("---@param target ActorRef<epok.Actor3D>\n---@return Bool\n"),
            "{stub}"
        );
        assert!(stub.contains(":aim(target) end"), "{stub}");
        assert!(stub.contains("---@param active Bool\nfunction "), "{stub}");
        assert!(
            stub.contains("--- Event: declare `function") && stub.contains(":tick(...)"),
            "{stub}"
        );
        assert!(!stub.contains("---@return nil"), "{stub}");

        // The primitive vocabulary and the builtin namespace.
        for line in [
            "---@alias Fixed number",
            "---@alias Int32 integer",
            "---@alias UInt32 integer",
            "---@alias Bool boolean",
            "---@class Vector3",
            "---@class ActorRef<T>",
            "function epok.input.held(button, port) end",
            "function epok.spawn(class, parent) end",
            "function epok.to_fixed(value) end",
            "function epok.UInt32(value) end",
            "function epok.ActorRef(class) end",
            "function epok.Hidden(value) end",
            "function epok.Enum(enumeration, variant) end",
            // The declaration form itself: `local Cube = epok.Actor3D:extend()`
            // needs the parent as a value and `extend` on the hierarchy root.
            "function _epok__Object:extend() end",
            "epok.Actor3D = _epok__Actor3D",
        ] {
            assert!(stub.contains(line), "{line} is missing from:\n{stub}");
        }
        // The replaced entry points are gone for good: there is no dual form.
        for gone in ["epok.class", "epok.super("] {
            assert!(!stub.contains(gone), "{gone} still appears in:\n{stub}");
        }
    }

    #[test]
    fn rendering_is_deterministic() {
        let registry = registry();
        assert_eq!(render(&registry), render(&registry));
        // Insertion order is not output order: the classes are sorted by name.
        let mut shuffled = Registry::new();
        for class in registry.classes.values().rev() {
            shuffled.classes.insert(class.id.clone(), class.clone());
        }
        assert_eq!(render(&shuffled), render(&registry));
        let spinner = render(&registry).find("---@class Spinner");
        let turret = render(&registry).find("---@class Turret");
        assert!(spinner < turret, "classes are sorted by name");
    }

    #[test]
    fn writing_creates_the_definitions_and_keeps_an_existing_configuration() {
        let root = crate::workspace::tests::temp("lua-api-stub");
        std::fs::create_dir_all(&root).unwrap();
        let registry = registry();
        write(&root, &registry).unwrap();
        let path = root.join(STUB_PATH);
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            render(&registry),
            "the written file is the rendered stub"
        );
        let config = std::fs::read_to_string(root.join(CONFIG_PATH)).unwrap();
        for setting in ["\"Lua 5.2\"", ".epok/lua", "\"epok\""] {
            assert!(config.contains(setting), "{config}");
        }
        // The definitions are only useful if a mismatch against them is
        // reported: the three the compiler also rejects are raised to errors.
        let parsed: serde_json::Value = serde_json::from_str(&config).unwrap();
        for check in [
            "assign-type-mismatch",
            "param-type-mismatch",
            "undefined-field",
        ] {
            assert_eq!(
                parsed["diagnostics.severity"][check].as_str(),
                Some("Error!"),
                "{config}"
            );
        }

        // An unchanged registry rewrites nothing.
        let before = std::fs::metadata(&path).unwrap().modified().unwrap();
        std::fs::write(root.join(CONFIG_PATH), "{ \"authored\": true }").unwrap();
        write(&root, &registry).unwrap();
        assert_eq!(
            std::fs::metadata(&path).unwrap().modified().unwrap(),
            before
        );
        assert_eq!(
            std::fs::read_to_string(root.join(CONFIG_PATH)).unwrap(),
            "{ \"authored\": true }",
            "an authored configuration is never overwritten"
        );

        // A changed registry does rewrite it.
        let mut changed = registry.clone();
        changed
            .classes
            .insert("late".into(), class("late", "Late", None));
        write(&root, &changed).unwrap();
        assert!(
            std::fs::read_to_string(&path)
                .unwrap()
                .contains("---@class Late")
        );
        let _ = std::fs::remove_dir_all(&root);
    }
}
