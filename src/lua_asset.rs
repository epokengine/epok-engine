//! `.lua` discovery and the statically extracted `epok.class` declaration.
//!
//! Declarations are read from the AST. `epok.class` is never executed: the
//! editor must be able to list classes, properties and functions without a Lua
//! interpreter, and the same metadata must be identical in every build.
#![allow(dead_code)] // M2 (`lua_compile`) publishes these declarations; M1 only produces them.
use crate::{
    blueprint::Registry,
    lua_frontend::ast,
    reflection_schema::{self as schema, Type},
    script_ir::Span,
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt, fs,
    io::Write,
    path::{Path, PathBuf},
};

/// The authoring language profile. Bumping it invalidates compiled artifacts.
pub const PROFILE_VERSION: u32 = 1;
/// Reflected events every `epok::Object` descendant may override without
/// repeating the signature in the metadata table.
pub const LIFECYCLE: &[&str] = &["begin_play", "tick", "end_play", "on_enable", "on_disable"];

pub fn provider() -> schema::Extension {
    schema::Extension {
        id: "lua".into(),
        version: 1,
    }
}

#[derive(Clone, Debug)]
pub struct LuaFile {
    pub path: PathBuf,
    pub source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub file: PathBuf,
    pub line: u32,
    pub column: u32,
    pub message: String,
}
impl Diagnostic {
    pub fn new(file: &Path, span: Span, message: impl Into<String>) -> Self {
        Self {
            file: file.into(),
            line: span.line,
            column: span.column,
            message: message.into(),
        }
    }
}
impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}: {}",
            self.file.display(),
            self.line,
            self.column,
            self.message
        )
    }
}

#[derive(Clone, Debug)]
pub struct DeclaredProperty {
    pub id: String,
    pub name: String,
    pub value_type: Type,
    pub default: Value,
    pub editable: bool,
    pub span: Span,
}
#[derive(Clone, Debug)]
pub struct DeclaredFunction {
    pub id: String,
    pub name: String,
    pub parameters: Vec<(String, Type)>,
    pub returns: Type,
    pub callable: bool,
    /// Reflected parent function name when this is an event override.
    pub overrides: Option<String>,
    /// The signature comes from the reflected parent, not the metadata table.
    pub inferred: bool,
    pub span: Span,
}
#[derive(Clone, Debug)]
pub struct Declaration {
    pub file: PathBuf,
    pub id: String,
    pub name: String,
    /// `cpp_name` of the parent: reflected C++, Blueprint or another Lua class.
    pub extends: String,
    pub profile: u32,
    pub properties: Vec<DeclaredProperty>,
    pub functions: Vec<DeclaredFunction>,
    /// Local the class table is bound to; methods must be declared on it.
    pub binding: String,
    pub span: Span,
}

// ------------------------------------------------------------ discovery ----

pub fn load_all(root: &Path) -> Result<Vec<LuaFile>, String> {
    fn visit(path: &Path, result: &mut Vec<LuaFile>) -> Result<(), String> {
        if !path.exists() {
            return Ok(());
        }
        let metadata = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if metadata.file_attributes() & 0x400 != 0 {
                return Err(format!(
                    "Lua discovery rejects reparse point {}",
                    path.display()
                ));
            }
        }
        if metadata.file_type().is_symlink() {
            return Err(format!(
                "Lua discovery rejects linked path {}",
                path.display()
            ));
        }
        for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let kind = entry.file_type().map_err(|e| e.to_string())?;
            if kind.is_symlink() {
                return Err(format!(
                    "Lua discovery rejects linked asset {}",
                    entry.path().display()
                ));
            }
            if kind.is_dir() {
                visit(&entry.path(), result)?;
            } else if entry.file_name().to_string_lossy().ends_with(".lua") {
                let path = entry.path();
                let source =
                    fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
                result.push(LuaFile { path, source });
            }
        }
        Ok(())
    }
    let mut result = vec![];
    visit(&root.join("assets/scripts"), &mut result)?;
    result.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(result)
}

// ----------------------------------------------------------- extraction ----

fn fields<'a>(
    expr: &'a ast::Expr,
    file: &Path,
    what: &str,
) -> Result<&'a [ast::Field], Diagnostic> {
    match expr {
        ast::Expr::Table { fields, .. } => Ok(fields),
        other => Err(Diagnostic::new(
            file,
            other.span(),
            format!("{what} must be a table"),
        )),
    }
}
fn entry<'a>(fields: &'a [ast::Field], key: &str) -> Option<&'a ast::Field> {
    fields.iter().find(|f| f.key.as_deref() == Some(key))
}
fn text(fields: &[ast::Field], key: &str, file: &Path, span: Span) -> Result<String, Diagnostic> {
    match entry(fields, key).map(|f| &f.value) {
        Some(ast::Expr::Str { value, .. }) => Ok(value.clone()),
        Some(other) => Err(Diagnostic::new(
            file,
            other.span(),
            format!("{key} must be a string"),
        )),
        None => Err(Diagnostic::new(file, span, format!("Missing {key}"))),
    }
}
fn flag(fields: &[ast::Field], key: &str, file: &Path) -> Result<bool, Diagnostic> {
    match entry(fields, key).map(|f| &f.value) {
        None => Ok(false),
        Some(ast::Expr::Bool(value, _)) => Ok(*value),
        Some(other) => Err(Diagnostic::new(
            file,
            other.span(),
            format!("{key} must be a boolean"),
        )),
    }
}

/// The metadata type vocabulary. It is deliberately a closed set: an unknown
/// name is a diagnostic, never an opaque pass-through to the code generator.
pub fn value_type(name: &str) -> Option<Type> {
    let generic = |prefix: &str| {
        name.strip_prefix(prefix)
            .and_then(|rest| rest.strip_prefix('<'))
            .and_then(|rest| rest.strip_suffix('>'))
            .filter(|inner| !inner.is_empty())
    };
    Some(match name {
        "void" => Type::Void,
        "Bool" => Type::Bool,
        "Int32" => Type::Int32,
        "UInt32" => Type::UInt32,
        "Fixed" => Type::Fixed,
        "Vector2" => Type::Vector { length: 2 },
        "Vector3" => Type::Vector { length: 3 },
        "ActorRef" => Type::ActorRef { class: None },
        "ComponentRef" => Type::ComponentRef { class: None },
        "ObjectRef" => Type::ObjectRef { class: None },
        _ => {
            if let Some(class) = generic("ActorRef") {
                Type::ActorRef {
                    class: Some(class.into()),
                }
            } else if let Some(class) = generic("ComponentRef") {
                Type::ComponentRef {
                    class: Some(class.into()),
                }
            } else if let Some(class) = generic("ObjectRef") {
                Type::ObjectRef {
                    class: Some(class.into()),
                }
            } else if let Some(kind) = generic("AssetRef") {
                Type::AssetRef { kind: kind.into() }
            } else {
                Type::ClassRef {
                    base: generic("ClassRef")?.into(),
                }
            }
        }
    })
}
fn declared_type(
    fields: &[ast::Field],
    key: &str,
    file: &Path,
    span: Span,
) -> Result<Type, Diagnostic> {
    let name = text(fields, key, file, span)?;
    value_type(&name)
        .ok_or_else(|| Diagnostic::new(file, span, format!("{name} is not an epok-lua type name")))
}
/// Literal defaults only. The metadata table is data, never an expression.
fn default_value(
    field: Option<&ast::Field>,
    ty: &Type,
    file: &Path,
    _span: Span,
) -> Result<Value, Diagnostic> {
    let Some(field) = field else {
        return Ok(crate::script_values::default_value(ty));
    };
    let invalid = || Diagnostic::new(file, field.value.span(), "Default is not a literal value");
    let value = match &field.value {
        ast::Expr::Nil(_) => Value::Null,
        ast::Expr::Bool(value, _) => Value::from(*value),
        ast::Expr::Str { value, .. } => Value::from(value.clone()),
        ast::Expr::Number { value, .. } => number(*value, ty),
        ast::Expr::Unary {
            op: ast::UnOp::Neg,
            operand,
            ..
        } => match &**operand {
            ast::Expr::Number { value, .. } => number(-*value, ty),
            _ => return Err(invalid()),
        },
        ast::Expr::Table { fields, .. } => {
            let component = match ty {
                Type::Vector { .. } => Type::Fixed,
                _ => return Err(invalid()),
            };
            Value::Array(
                fields
                    .iter()
                    .map(|f| default_value(Some(f), &component, file, f.span))
                    .collect::<Result<Vec<_>, _>>()?,
            )
        }
        _ => return Err(invalid()),
    };
    Ok(value)
}
fn number(value: f64, ty: &Type) -> Value {
    match ty {
        Type::Int32 => Value::from(value as i64),
        Type::UInt32 => Value::from(value as u64),
        Type::Enum { .. } => Value::from(value as i64),
        _ => Value::from(value),
    }
}

fn canonical(id: &str) -> bool {
    uuid::Uuid::parse_str(id).is_ok_and(|value| !value.is_nil() && value.to_string() == id)
}
/// Engine-derived identity. The editor assigns it when the author writes no
/// `id`, exactly as a C++ declaration without `Id=` is identified by
/// `cpp:<USR>`. The `lua:` marker cannot collide with a UUID or a `cpp:`
/// identity by construction.
pub fn derived(id: &str) -> bool {
    id.starts_with(DERIVED)
}
const DERIVED: &str = "lua:";

/// Class identity when the metadata table declares no `id`. An explicit id
/// survives a rename; a derived id is stable across machines and moves, but
/// renaming the class changes it, so a renamed class needs an explicit id or a
/// migration.
fn derived_class_id(name: &str) -> String {
    format!("{DERIVED}{name}")
}
/// Member identity when the entry declares no `id`, and the identity of a
/// lifecycle event written as a bare method with no `functions` entry at all.
/// It is derived from the class *identity*, so a class pinned to an explicit
/// UUID keeps its members stable across a class rename too.
fn derived_member_id(class_id: &str, member: &str) -> String {
    let stem = class_id.strip_prefix(DERIVED).unwrap_or(class_id);
    format!("{DERIVED}{stem}:{member}")
}
/// A persistent Lua id: an explicit canonical UUID, or a derived identity.
pub fn identity(id: &str) -> bool {
    canonical(id) || derived(id)
}
/// Filesystem-safe stem for the artifacts generated for a class. An explicit id
/// is a canonical UUID and already safe; a derived `lua:<Name>` identity becomes
/// `lua_<Name>`, unique because generated class names are unique.
pub fn artifact_stem(class_id: &str) -> String {
    match class_id.strip_prefix(DERIVED) {
        Some(name) => format!("lua_{name}"),
        None => class_id.to_owned(),
    }
}
/// An authored `id`, which is optional everywhere. When present it must be a
/// canonical UUID: the derived form is assigned by the engine, never typed.
fn explicit_id(
    fields: &[ast::Field],
    file: &Path,
    what: &str,
) -> Result<Option<String>, Diagnostic> {
    match entry(fields, "id").map(|f| &f.value) {
        None => Ok(None),
        Some(ast::Expr::Str { value, .. }) if canonical(value) => Ok(Some(value.clone())),
        Some(other) => Err(Diagnostic::new(
            file,
            other.span(),
            format!("{what} id must be a canonical UUID"),
        )),
    }
}

pub fn extract(file: &LuaFile) -> Result<Declaration, Diagnostic> {
    let chunk = crate::lua_frontend::parse(file)?;
    let path = file.path.as_path();
    let top = Span { line: 1, column: 1 };
    let mut declarations = chunk.locals.iter().filter(|(_, value, _)| {
        matches!(value, ast::Expr::Call { base, .. }
            if matches!(&**base, ast::Expr::Field { base, name, .. }
                if name == "class" && matches!(&**base, ast::Expr::Name{name,..} if name == "epok")))
    });
    let Some((binding, value, span)) = declarations.next() else {
        return Err(Diagnostic::new(
            path,
            top,
            "No `local <Class> = epok.class{...}` declaration in this file",
        ));
    };
    if let Some((_, _, extra)) = declarations.next() {
        return Err(Diagnostic::new(
            path,
            *extra,
            "A Lua script declares exactly one class",
        ));
    }
    let span = *span;
    let ast::Expr::Call { args, .. } = value else {
        unreachable!("filtered above")
    };
    if args.len() != 1 {
        return Err(Diagnostic::new(
            path,
            span,
            "`epok.class` takes exactly one metadata table",
        ));
    }
    let table = fields(&args[0], path, "`epok.class` metadata")?;

    match &chunk.returns {
        Some((name, _)) if name == binding => {}
        Some((_, at)) => {
            return Err(Diagnostic::new(
                path,
                *at,
                format!("The file must end with `return {binding}`"),
            ));
        }
        None => {
            return Err(Diagnostic::new(
                path,
                span,
                format!("The file must end with `return {binding}`"),
            ));
        }
    }

    // `profile` is optional: absent means the profile this editor supports. It
    // is written only to pin a script to one profile version on purpose.
    let profile = match entry(table, "profile").map(|f| &f.value) {
        None => PROFILE_VERSION,
        Some(ast::Expr::Number { value, .. }) => *value as u32,
        Some(other) => {
            return Err(Diagnostic::new(
                path,
                other.span(),
                "profile must be a version number",
            ));
        }
    };
    if profile != PROFILE_VERSION {
        return Err(Diagnostic::new(
            path,
            span,
            format!("Profile {profile} is not epok-lua v{PROFILE_VERSION}"),
        ));
    }
    let name = text(table, "name", path, span)?;
    let id = explicit_id(table, path, "Class")?.unwrap_or_else(|| derived_class_id(&name));
    let extends = text(table, "extends", path, span)?;

    let mut properties = vec![];
    if let Some(field) = entry(table, "properties") {
        for property in fields(&field.value, path, "properties")? {
            let Some(key) = property.key.clone() else {
                return Err(Diagnostic::new(
                    path,
                    property.span,
                    "Properties are declared as `<name> = { ... }`",
                ));
            };
            let body = fields(&property.value, path, "A property")?;
            let value_type = declared_type(body, "type", path, property.span)?;
            properties.push(DeclaredProperty {
                id: explicit_id(body, path, &format!("Property {key}"))?
                    .unwrap_or_else(|| derived_member_id(&id, &key)),
                name: key,
                default: default_value(entry(body, "default"), &value_type, path, property.span)?,
                value_type,
                editable: flag(body, "editable", path)?,
                span: property.span,
            });
        }
    }

    let mut functions = vec![];
    if let Some(field) = entry(table, "functions") {
        for function in fields(&field.value, path, "functions")? {
            let Some(key) = function.key.clone() else {
                return Err(Diagnostic::new(
                    path,
                    function.span,
                    "Functions are declared as `<name> = { ... }`",
                ));
            };
            let body = fields(&function.value, path, "A function")?;
            let mut parameters = vec![];
            if let Some(list) = entry(body, "parameters") {
                for parameter in fields(&list.value, path, "parameters")? {
                    if parameter.key.is_some() {
                        return Err(Diagnostic::new(
                            path,
                            parameter.span,
                            "Parameters are an ordered list of `{ name = ..., type = ... }`",
                        ));
                    }
                    let entry = fields(&parameter.value, path, "A parameter")?;
                    parameters.push((
                        text(entry, "name", path, parameter.span)?,
                        declared_type(entry, "type", path, parameter.span)?,
                    ));
                }
            }
            let overrides = match entry(body, "overrides").map(|f| &f.value) {
                None => None,
                Some(ast::Expr::Str { value, .. }) => Some(value.clone()),
                Some(other) => {
                    return Err(Diagnostic::new(
                        path,
                        other.span(),
                        "overrides must name a reflected parent function",
                    ));
                }
            };
            functions.push(DeclaredFunction {
                id: explicit_id(body, path, &format!("Function {key}"))?
                    .unwrap_or_else(|| derived_member_id(&id, &key)),
                name: key,
                parameters,
                returns: match entry(body, "returns") {
                    Some(_) => declared_type(body, "returns", path, function.span)?,
                    None => Type::Void,
                },
                callable: flag(body, "callable", path)?,
                overrides,
                inferred: false,
                span: function.span,
            });
        }
    }

    for method in &chunk.methods {
        if &method.object != binding {
            return Err(Diagnostic::new(
                path,
                method.span,
                format!("Methods must be declared on {binding}"),
            ));
        }
        if functions.iter().any(|f| f.name == method.name) {
            continue;
        }
        if !LIFECYCLE.contains(&method.name.as_str()) {
            return Err(Diagnostic::new(
                path,
                method.span,
                format!(
                    "{} is neither declared in `functions` nor a reflected lifecycle event",
                    method.name
                ),
            ));
        }
        functions.push(DeclaredFunction {
            id: derived_member_id(&id, &method.name),
            name: method.name.clone(),
            parameters: vec![],
            returns: Type::Void,
            callable: false,
            overrides: Some(method.name.clone()),
            inferred: true,
            span: method.span,
        });
    }

    Ok(Declaration {
        file: file.path.clone(),
        id,
        name,
        extends,
        profile,
        properties,
        functions,
        binding: binding.clone(),
        span,
    })
}

// ---------------------------------------------------------- declarations ----

fn location(path: &Path, span: Span) -> schema::Location {
    schema::Location {
        file: path.into(),
        line: span.line,
        column: span.column,
    }
}
/// Inherited exposure is folded into overrides exactly as the Blueprint
/// compiler does, so a Lua class sees the same reflected API as a Blueprint.
fn inherited(registry: &Registry, parent: &str) -> BTreeMap<String, schema::Function> {
    let mut by_id = BTreeMap::<String, schema::Function>::new();
    for ancestor in registry.ancestry(parent) {
        for original in &ancestor.functions {
            let mut resolved = original.clone();
            for id in &original.overrides {
                if let Some(parent) = by_id.get(id) {
                    resolved.callable |= parent.callable;
                    resolved.event |= parent.event;
                    resolved.pure |= parent.pure;
                }
            }
            for id in resolved
                .overrides
                .iter()
                .chain(std::iter::once(&resolved.id))
            {
                by_id.insert(id.clone(), resolved.clone());
            }
        }
    }
    by_id
}

pub fn declarations(
    decl: &Declaration,
    file: &LuaFile,
    registry: &Registry,
) -> Result<schema::Class, Diagnostic> {
    let path = file.path.as_path();
    let fail = |span: Span, message: String| Diagnostic::new(path, span, message);
    if !identity(&decl.id) {
        return Err(fail(
            decl.span,
            "Class id is neither a canonical UUID nor a derived identity".into(),
        ));
    }
    if !crate::scripts::class_identifier(&decl.name) || decl.name.starts_with("epok_") {
        return Err(fail(
            decl.span,
            format!("{} is not a valid generated class name", decl.name),
        ));
    }
    let Some(parent) = registry.named(&decl.extends) else {
        return Err(fail(decl.span, format!("Unknown parent {}", decl.extends)));
    };
    if !crate::script_backend::can_derive(&provider(), parent) {
        return Err(fail(
            decl.span,
            format!("{} does not support native Lua inheritance", decl.extends),
        ));
    }
    if registry.classes.contains_key(&decl.id) {
        return Err(fail(decl.span, "Duplicate class UUID".into()));
    }

    let mut inherited_properties = 0;
    let mut names = BTreeSet::new();
    let mut ids = BTreeSet::new();
    for ancestor in registry.ancestry(&parent.cpp_name) {
        for property in &ancestor.properties {
            if names.insert(property.name.clone()) {
                inherited_properties += 1;
            }
            ids.insert(property.id.clone());
        }
        for function in &ancestor.functions {
            names.insert(function.name.clone());
            ids.insert(function.id.clone());
        }
    }
    let functions = inherited(registry, &parent.cpp_name);

    let mut properties = vec![];
    for property in &decl.properties {
        if !identity(&property.id) || !ids.insert(property.id.clone()) {
            return Err(fail(
                property.span,
                format!("Property {} has an invalid or duplicated id", property.name),
            ));
        }
        if crate::script_ir::Intrinsic::from_name(&property.name).is_some() {
            return Err(fail(
                property.span,
                format!(
                    "{}: {} cannot be declared as a property",
                    crate::lua_frontend::profile::TRANSFORM_RESERVED,
                    property.name
                ),
            ));
        }
        if !crate::scripts::identifier(&property.name)
            || property.name.starts_with("epok_")
            || !names.insert(property.name.clone())
        {
            return Err(fail(
                property.span,
                format!(
                    "Property {} has an invalid name or shadows an inherited member",
                    property.name
                ),
            ));
        }
        if !crate::script_values::valid(&property.default, &property.value_type)
            || matches!(property.value_type, Type::Vector { length } if length != 2 && length != 3)
            || property.value_type == Type::Void
        {
            return Err(fail(
                property.span,
                format!(
                    "Property {} has no valid bounded native default",
                    property.name
                ),
            ));
        }
        properties.push(schema::Property {
            id: property.id.clone(),
            name: property.name.clone(),
            value_type: property.value_type.clone(),
            default: property.default.clone(),
            editable: property.editable,
            timeline: None,
            source: location(path, property.span),
        });
    }
    if inherited_properties + properties.len() > 16 {
        return Err(fail(
            decl.span,
            "Lua hierarchy exceeds the 16-property runtime budget".into(),
        ));
    }

    let mut declared = vec![];
    let mut overridden = BTreeSet::new();
    for function in &decl.functions {
        if !identity(&function.id) || !ids.insert(function.id.clone()) {
            return Err(fail(
                function.span,
                format!("Function {} has an invalid or duplicated id", function.name),
            ));
        }
        let mut resolved = if let Some(parent_name) = &function.overrides {
            let Some(inherited) = functions.values().find(|f| &f.name == parent_name) else {
                return Err(fail(
                    function.span,
                    format!("Missing overridden function {parent_name}"),
                ));
            };
            if !inherited.event
                || inherited.final_method
                || inherited.access == "private"
                || !overridden.insert(inherited.id.clone())
            {
                return Err(fail(
                    function.span,
                    format!("{parent_name} is not an overridable event or is overridden twice"),
                ));
            }
            if !function.inferred
                && (function.name != inherited.name
                    || function.returns != inherited.returns
                    || function.parameters.len() != inherited.parameters.len()
                    || function
                        .parameters
                        .iter()
                        .zip(&inherited.parameters)
                        .any(|((name, ty), p)| name != &p.name || ty != &p.value_type))
            {
                return Err(fail(
                    function.span,
                    format!(
                        "Override {} signature differs from its reflected parent",
                        function.name
                    ),
                ));
            }
            let mut resolved = inherited.clone();
            resolved.overrides = vec![inherited.id.clone()];
            resolved
        } else {
            if !crate::scripts::identifier(&function.name)
                || function.name.starts_with("epok_")
                || !names.insert(function.name.clone())
            {
                return Err(fail(
                    function.span,
                    format!(
                        "Function {} has an invalid name or shadows an inherited member; declare it as an override",
                        function.name
                    ),
                ));
            }
            schema::Function {
                id: function.id.clone(),
                name: function.name.clone(),
                parameters: function
                    .parameters
                    .iter()
                    .map(|(name, value_type)| schema::Parameter {
                        name: name.clone(),
                        value_type: value_type.clone(),
                        direction: schema::Direction::Value,
                    })
                    .collect(),
                returns: function.returns.clone(),
                callable: true,
                timeline: None,
                event: true,
                pure: false,
                abstract_method: false,
                final_method: false,
                access: "public".into(),
                overrides: vec![],
                source: location(path, function.span),
            }
        };
        crate::blueprint_ir::cpp_type(&resolved.returns).map_err(|e| fail(function.span, e))?;
        let mut parameter_names = BTreeSet::new();
        for parameter in &resolved.parameters {
            if !crate::scripts::identifier(&parameter.name)
                || parameter.name.starts_with("epok_")
                || !parameter_names.insert(parameter.name.clone())
                || parameter.value_type == Type::Void
            {
                return Err(fail(
                    function.span,
                    format!("{}: invalid or duplicate parameter", function.name),
                ));
            }
            crate::blueprint_ir::cpp_type(&parameter.value_type)
                .map_err(|e| fail(function.span, e))?;
        }
        resolved.id = function.id.clone();
        resolved.abstract_method = false;
        resolved.final_method = false;
        resolved.source = location(path, function.span);
        declared.push(resolved);
    }

    Ok(schema::Class {
        id: decl.id.clone(),
        provider: provider(),
        backend: schema::native_backend(),
        cpp_name: decl.name.clone(),
        parent: Some(parent.id.clone()),
        abstract_class: functions
            .values()
            .any(|f| f.abstract_method && !overridden.contains(&f.id)),
        final_class: false,
        blueprintable: true,
        timeline_component: None,
        family: None,
        domain: None,
        placement: Default::default(),
        component: None,
        default_components: vec![],
        explicit_abstract: false,
        properties,
        functions: declared,
        source: location(path, decl.span),
    })
}

/// The `.lua` file a Lua class was authored in, so the editor opens the real
/// asset instead of the generated C++ subclass. Mirrors
/// `scripts::editable_class_source`: only project-owned sources, never an SDK
/// header or a guessed filename.
pub fn editable_class_source(root: &Path, class: &schema::Class) -> Option<PathBuf> {
    if class.provider != provider() {
        return None;
    }
    let root = fs::canonicalize(root).ok()?;
    let source = if class.source.file.is_absolute() {
        class.source.file.clone()
    } else {
        root.join(&class.source.file)
    };
    let source = fs::canonicalize(source).ok()?;
    (source.is_file()
        && source.extension().is_some_and(|e| e == "lua")
        && source.starts_with(root.join("assets/scripts")))
    .then_some(source)
}

// -------------------------------------------------------------- creation ----

/// Publish a new `.lua` class with `create_new` and roll back only the files
/// and directories this call owns, then prove the whole project still compiles.
/// Source of a freshly created class. The metadata table is read statically;
/// the comments explain each key without adding declarations the author has
/// to delete. Tests patch `properties = {}` and the empty `begin_play` body.
pub fn template(name: &str, id: &str, parent_cpp_name: &str) -> String {
    format!(
        "-- Class metadata. The editor reads this table from the source text; it is\n\
-- never executed to discover the class.\n\
local {name} = epok.class {{\n\
    id = \"{id}\", -- assigned by the editor; keeps placed instances bound if the class is renamed\n\
    name = \"{name}\", -- generated C++ class name\n\
    extends = \"{parent_cpp_name}\", -- any Blueprintable reflected parent\n\
    -- Inspector-editable fields, for example:\n\
    -- speed = {{ type = \"Fixed\", default = 1.0, editable = true }},\n\
    properties = {{}},\n\
    -- Methods other classes and Blueprints may call, for example:\n\
    -- reset = {{ callable = true, parameters = {{}}, returns = \"void\" }},\n\
    functions = {{}}\n\
}}\n\
\n\
-- Lifecycle events need no `functions` entry: begin_play, tick, end_play,\n\
-- on_enable and on_disable take their signature from the parent.\n\
function {name}:begin_play()\nend\n\
\n\
-- function {name}:tick(arg0) -- arg0: elapsed seconds (Fixed, Q12)\n\
-- end\n\
\n\
return {name}\n"
    )
}

pub fn create_in(
    root: &Path,
    name: &str,
    folder: &str,
    parent_cpp_name: &str,
) -> Result<PathBuf, String> {
    if !crate::scripts::identifier(name) || name.starts_with("epok_") {
        return Err("Lua class names must be valid, non-reserved C++ identifiers.".into());
    }
    crate::workspace::validate_name(name)?;
    let catalog = crate::scripts::catalog(root)?;
    if catalog.iter().any(|s| s.name.eq_ignore_ascii_case(name)) {
        return Err(format!("A class named {name} already exists"));
    }
    let registry = crate::blueprint::native_registry(root, &catalog)?;
    let parent = registry
        .named(parent_cpp_name)
        .ok_or_else(|| format!("Unknown parent {parent_cpp_name}"))?;
    if !crate::script_backend::can_derive(&provider(), parent) {
        return Err(format!("{parent_cpp_name} is not an eligible parent"));
    }
    let relative = folder.replace('\\', "/");
    if !relative.is_empty() && !relative.split('/').all(crate::scripts::identifier) {
        return Err(
            "Use relative folder components such as Enemies/Bosses (letters, digits, underscores)."
                .into(),
        );
    }
    let scripts_root = fs::canonicalize(root.join("assets/scripts")).map_err(|e| e.to_string())?;
    let canonical_root = fs::canonicalize(root).map_err(|e| e.to_string())?;
    if !scripts_root.starts_with(&canonical_root) {
        return Err("Script folders must remain inside the project.".into());
    }
    let mut dir = scripts_root.clone();
    let mut owned_dirs = Vec::new();
    for part in relative.split('/').filter(|p| !p.is_empty()) {
        dir.push(part);
        match fs::create_dir(&dir) {
            Ok(()) => owned_dirs.push(dir.clone()),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists && dir.is_dir() => {}
            Err(e) => {
                for path in owned_dirs.iter().rev() {
                    let _ = fs::remove_dir(path);
                }
                return Err(e.to_string());
            }
        }
        if !fs::canonicalize(&dir)
            .map_err(|e| e.to_string())?
            .starts_with(&scripts_root)
        {
            for path in owned_dirs.iter().rev() {
                let _ = fs::remove_dir(path);
            }
            return Err("Script folder links must remain inside assets/scripts.".into());
        }
    }
    let source = template(name, &uuid::Uuid::new_v4().to_string(), parent_cpp_name);
    let path = dir.join(format!("{name}.lua"));
    let result = (|| {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|e| format!("{}: {e}", path.display()))?;
        file.write_all(source.as_bytes())
            .map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
        drop(file);
        if !crate::scripts::catalog(root)?
            .iter()
            .any(|s| s.name == name)
        {
            return Err(format!("{name}: the generated Lua class did not compile"));
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&path);
        for owned in owned_dirs.iter().rev() {
            let _ = fs::remove_dir(owned);
        }
    }
    result.map(|()| path)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::actor_document::tests::class;
    use crate::object_model::{ACTOR_COMPONENT_ID, OBJECT_ID};

    pub(crate) const ARMOUR_ID: &str = "b1d9e0a2-4c6f-4e8a-9b3d-5f7a1c2e4d60";
    pub(crate) const ENEMY_BASE_ID: &str = "c2e0f1b3-5d7a-4f9b-8c4e-6a8b2d3f5e71";
    pub(crate) const SEALED_ID: &str = "d3f1a2c4-6e8b-4a0c-9d5f-7b9c3e4a6f82";
    pub(crate) const OFFSET_ID: &str = "e4a2b3c5-7f9c-4b1d-8e6a-9c1d4e5b7a94";
    pub(crate) const HEADER: &str = r#"
local EnemyLogic = epok.class {
    profile = 1,
    id = "956f4946-0c61-42f8-899e-2db063b42420",
    name = "EnemyLogic",
    extends = "epok::ActorComponent",
    properties = {
        health = { id = "3d352b2b-c2d7-4b99-9ba1-a003d648e897",
            type = "Fixed", default = 100, editable = true },
        charges = { id = "5c1a5a1e-1d0e-4f3a-9a2b-1c2d3e4f5a6b",
            type = "UInt32", default = 3, editable = true },
        ready = { id = "7f9c0b1d-2e3f-4a5b-8c9d-0e1f2a3b4c5d",
            type = "Bool", default = true, editable = true }
    },
    functions = {
        damage = { id = "224e6b46-e4b4-475c-9744-8b9fb4c0baaa", callable = true,
            parameters = { { name = "amount", type = "Fixed" } }, returns = "void" },
        absorb = { id = "9a8b7c6d-5e4f-4a3b-2c1d-0e9f8a7b6c5d", callable = true,
            parameters = { { name = "amount", type = "Fixed" } }, returns = "Fixed" }
    }
}
"#;

    fn function(
        id: &str,
        name: &str,
        parameters: &[(&str, Type)],
        returns: Type,
    ) -> schema::Function {
        schema::Function {
            id: id.into(),
            name: name.into(),
            parameters: parameters
                .iter()
                .map(|(name, value_type)| schema::Parameter {
                    name: (*name).into(),
                    value_type: value_type.clone(),
                    direction: schema::Direction::Value,
                })
                .collect(),
            returns,
            callable: false,
            timeline: None,
            event: true,
            pure: false,
            abstract_method: false,
            final_method: false,
            access: "public".into(),
            overrides: vec![],
            source: schema::Location {
                file: "object_model.hpp".into(),
                line: 1,
                column: 1,
            },
        }
    }

    /// Hand-built reflected declarations: libclang extraction needs the MIPS
    /// include paths and cannot run on a host, exactly as in `object_model.rs`.
    pub(crate) fn registry() -> Registry {
        let mut registry = Registry::new();
        registry
            .classes
            .insert(OBJECT_ID.into(), class(OBJECT_ID, "epok::Object", None));

        let mut component = class(ACTOR_COMPONENT_ID, "epok::ActorComponent", Some(OBJECT_ID));
        component.properties.push(schema::Property {
            id: ARMOUR_ID.into(),
            name: "armour".into(),
            value_type: Type::Fixed,
            default: serde_json::json!(0.0),
            editable: true,
            timeline: None,
            source: schema::Location {
                file: "object_model.hpp".into(),
                line: 1,
                column: 1,
            },
        });
        // A reflected vector field: profile v1 exposes its components, never
        // the whole value, and the frontend tests rely on it being present.
        component.properties.push(schema::Property {
            id: OFFSET_ID.into(),
            name: "offset".into(),
            value_type: Type::Vector { length: 3 },
            default: serde_json::json!([0.0, 0.0, 0.0]),
            editable: true,
            timeline: None,
            source: schema::Location {
                file: "object_model.hpp".into(),
                line: 1,
                column: 1,
            },
        });
        for (index, name) in LIFECYCLE.iter().enumerate() {
            let id = format!("00000000-0000-4000-8000-00000000000{index}");
            let parameters: &[(&str, Type)] = if *name == "tick" {
                &[("delta", Type::Fixed)]
            } else {
                &[]
            };
            component
                .functions
                .push(function(&id, name, parameters, Type::Void));
        }
        let mut hit = function(
            "11111111-1111-4111-8111-111111111111",
            "hit",
            &[("amount", Type::Fixed)],
            Type::Void,
        );
        hit.callable = true;
        component.functions.push(hit);
        registry
            .classes
            .insert(ACTOR_COMPONENT_ID.into(), component);

        let mut base = class(ENEMY_BASE_ID, "EnemyBase", Some(ACTOR_COMPONENT_ID));
        let mut damage = function(
            "22222222-2222-4222-8222-222222222222",
            "damage",
            &[("amount", Type::Fixed)],
            Type::Void,
        );
        damage.callable = true;
        base.functions.push(damage);
        let mut sealed = function(SEALED_ID, "sealed", &[], Type::Void);
        sealed.final_method = true;
        sealed.callable = true;
        base.functions.push(sealed);
        registry.classes.insert(ENEMY_BASE_ID.into(), base);

        let mut final_parent = class(
            "e4a2b3c5-7f9c-4b1d-8e6a-9c1d4e5b7a93",
            "SealedBase",
            Some(ACTOR_COMPONENT_ID),
        );
        final_parent.final_class = true;
        registry
            .classes
            .insert(final_parent.id.clone(), final_parent);
        registry.normalize_functions();
        registry
    }

    pub(crate) fn file(source: &str) -> LuaFile {
        LuaFile {
            path: "assets/scripts/EnemyLogic.lua".into(),
            source: source.into(),
        }
    }
    fn example() -> LuaFile {
        file(&format!(
            "{HEADER}\nfunction EnemyLogic:damage(amount)\n    self.health = self.health - amount\nend\n\nreturn EnemyLogic\n"
        ))
    }

    #[test]
    fn lua_asset_extracts_the_documented_declaration() {
        let decl = extract(&example()).unwrap();
        assert_eq!(decl.id, "956f4946-0c61-42f8-899e-2db063b42420");
        assert_eq!(decl.name, "EnemyLogic");
        assert_eq!(decl.extends, "epok::ActorComponent");
        assert_eq!(decl.profile, PROFILE_VERSION);
        assert_eq!(decl.binding, "EnemyLogic");
        let health = decl.properties.iter().find(|p| p.name == "health").unwrap();
        assert_eq!(health.value_type, Type::Fixed);
        assert_eq!(health.default, serde_json::json!(100.0));
        assert!(health.editable);
        let damage = decl.functions.iter().find(|f| f.name == "damage").unwrap();
        assert_eq!(damage.parameters, vec![("amount".into(), Type::Fixed)]);
        assert_eq!(damage.returns, Type::Void);
        assert!(damage.callable && damage.overrides.is_none());

        let class = declarations(&decl, &example(), &registry()).unwrap();
        assert_eq!(class.provider, provider());
        assert_eq!(class.backend, schema::native_backend());
        assert_eq!(class.parent.as_deref(), Some(ACTOR_COMPONENT_ID));
        assert!(class.blueprintable && !class.final_class && !class.abstract_class);
        let damage = class.functions.iter().find(|f| f.name == "damage").unwrap();
        assert!(damage.callable && damage.event && !damage.pure);
        assert_eq!(damage.access, "public");
        assert!(damage.overrides.is_empty());
        assert_eq!(
            class.properties[0].source.file,
            PathBuf::from("assets/scripts/EnemyLogic.lua")
        );
    }

    #[test]
    fn lua_asset_requires_the_declaration_shape() {
        let missing_return = file(&format!("{HEADER}\n"));
        assert!(
            extract(&missing_return)
                .unwrap_err()
                .message
                .contains("must end with `return EnemyLogic`")
        );
        let twice = file(&format!("{HEADER}{HEADER}\nreturn EnemyLogic\n"));
        assert!(
            extract(&twice)
                .unwrap_err()
                .message
                .contains("exactly one class")
        );
        let dot = file(&format!(
            "{HEADER}\nfunction EnemyLogic.damage(amount)\nend\nreturn EnemyLogic\n"
        ));
        assert!(
            extract(&dot)
                .unwrap_err()
                .message
                .contains("must be declared as `function EnemyLogic:damage`")
        );
        let undeclared = file(&format!(
            "{HEADER}\nfunction EnemyLogic:mystery()\nend\nreturn EnemyLogic\n"
        ));
        assert!(
            extract(&undeclared)
                .unwrap_err()
                .message
                .contains("neither declared in `functions`")
        );
        // A bare lifecycle method needs no `functions` entry.
        let lifecycle = file(&format!(
            "{HEADER}\nfunction EnemyLogic:begin_play()\nend\nreturn EnemyLogic\n"
        ));
        let decl = extract(&lifecycle).unwrap();
        let begin = decl
            .functions
            .iter()
            .find(|f| f.name == "begin_play")
            .unwrap();
        assert_eq!(begin.overrides.as_deref(), Some("begin_play"));
        // The synthesized id follows the same scheme as an omitted member id.
        assert_eq!(
            begin.id,
            "lua:956f4946-0c61-42f8-899e-2db063b42420:begin_play"
        );
        assert!(begin.inferred);
        let class = declarations(&decl, &lifecycle, &registry()).unwrap();
        let begin = class
            .functions
            .iter()
            .find(|f| f.name == "begin_play")
            .unwrap();
        assert_eq!(
            begin.overrides,
            vec!["00000000-0000-4000-8000-000000000000"]
        );
    }

    #[test]
    fn lua_asset_rejects_invalid_declarations() {
        let registry = registry();
        let check = |source: String, expected: &str| {
            let file = file(&source);
            let message = match extract(&file) {
                Err(error) => error.message,
                Ok(decl) => {
                    declarations(&decl, &file, &registry)
                        .expect_err(&format!("expected a diagnostic:\n{source}"))
                        .message
                }
            };
            assert!(
                message.contains(expected),
                "got `{message}`, expected `{expected}`"
            );
        };
        let tail = "\nreturn EnemyLogic\n";
        check(
            HEADER.replace("epok::ActorComponent", "epok::Nonexistent") + tail,
            "Unknown parent",
        );
        check(
            HEADER.replace("epok::ActorComponent", "SealedBase") + tail,
            "does not support native Lua inheritance",
        );
        check(
            HEADER.replace("956f4946-0c61-42f8-899e-2db063b42420", "not-a-uuid") + tail,
            "Class id must be a canonical UUID",
        );
        // Shadowing an inherited property name.
        check(
            HEADER.replace("health =", "armour =") + tail,
            "shadows an inherited member",
        );
        // The intrinsic transform names are reserved in every class, spatial or
        // not: a property called `rotation` would shadow `self.rotation`.
        check(
            HEADER.replace("health =", "rotation =") + tail,
            crate::lua_frontend::profile::TRANSFORM_RESERVED,
        );
        // A default that is not valid for the declared type.
        check(
            HEADER.replace("default = true", "default = 2") + tail,
            "no valid bounded native default",
        );
        // Overriding a final function.
        check(
            HEADER
                .replace("epok::ActorComponent", "EnemyBase")
                .replace("damage = {", "smash = {")
                .replace(
                    r#"returns = "Fixed" }"#,
                    r#"returns = "Fixed" },
        sealed = { id = "a5b6c7d8-1e2f-4a3b-8c4d-5e6f7a8b9c01",
            overrides = "sealed", returns = "void" }"#,
                )
                + tail,
            "not an overridable event",
        );
        // An override whose signature differs from the reflected parent.
        check(
            HEADER.replace("epok::ActorComponent", "EnemyBase").replace(
                r#"damage = { id = "224e6b46-e4b4-475c-9744-8b9fb4c0baaa", callable = true,
            parameters = { { name = "amount", type = "Fixed" } }, returns = "void" },"#,
                r#"damage = { id = "224e6b46-e4b4-475c-9744-8b9fb4c0baaa", overrides = "damage",
            parameters = { { name = "amount", type = "Int32" } }, returns = "void" },"#,
            ) + tail,
            "signature differs from its reflected parent",
        );
    }

    #[test]
    fn lua_asset_enforces_the_property_budget() {
        let mut extra = String::new();
        for index in 0..16 {
            extra.push_str(&format!(
                "        slot{index} = {{ id = \"aaaaaaaa-0000-4000-8000-{index:012}\",
            type = \"Int32\", default = 0, editable = true }},\n"
            ));
        }
        let source = HEADER.replace("        health =", &format!("{extra}        health ="))
            + "\nreturn EnemyLogic\n";
        let file = file(&source);
        let decl = extract(&file).unwrap();
        assert!(
            declarations(&decl, &file, &registry())
                .unwrap_err()
                .message
                .contains("16-property runtime budget")
        );
    }

    #[test]
    fn lua_asset_discovers_scripts_and_rejects_links() {
        let root = crate::workspace::tests::temp("lua-discovery");
        let scripts = root.join("assets/scripts/enemies");
        fs::create_dir_all(&scripts).unwrap();
        fs::write(scripts.join("B.lua"), "-- b").unwrap();
        fs::write(root.join("assets/scripts/A.lua"), "-- a").unwrap();
        fs::write(root.join("assets/scripts/ignored.txt"), "-- no").unwrap();
        let files = load_all(&root).unwrap();
        assert_eq!(files.len(), 2);
        assert!(files[0].path.ends_with("A.lua"));
        assert!(files[1].path.ends_with("enemies/B.lua"));
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(
                root.join("assets/scripts/A.lua"),
                root.join("assets/scripts/Link.lua"),
            )
            .unwrap();
            assert!(load_all(&root).unwrap_err().contains("linked asset"));
        }
    }

    #[test]
    fn lua_asset_maps_the_metadata_type_vocabulary() {
        assert_eq!(value_type("Fixed"), Some(Type::Fixed));
        assert_eq!(value_type("Vector3"), Some(Type::Vector { length: 3 }));
        assert_eq!(
            value_type("ActorRef<epok::Actor3D>"),
            Some(Type::ActorRef {
                class: Some("epok::Actor3D".into())
            })
        );
        assert_eq!(
            value_type("ComponentRef"),
            Some(Type::ComponentRef { class: None })
        );
        assert_eq!(
            value_type("AssetRef<Mesh>"),
            Some(Type::AssetRef {
                kind: "Mesh".into()
            })
        );
        assert_eq!(
            value_type("ClassRef<epok::Actor>"),
            Some(Type::ClassRef {
                base: "epok::Actor".into()
            })
        );
        assert_eq!(value_type("void"), Some(Type::Void));
        assert_eq!(value_type("String"), None);
    }

    #[test]
    fn creation_template_extracts_and_keeps_the_patched_anchors() {
        let source = template(
            "Spinner",
            "1f4d9c8e-2b3a-4c5d-8e6f-7a8b9c0d1e2f",
            "epok::ActorComponent",
        );
        assert!(source.contains("properties = {},"));
        assert!(source.contains("function Spinner:begin_play()\nend"));
        assert!(!source.contains("PSX"));
        assert!(!source.contains("profile ="));
        let file = LuaFile {
            path: PathBuf::from("assets/scripts/Spinner.lua"),
            source,
        };
        let declaration = extract(&file).unwrap();
        assert_eq!(declaration.name, "Spinner");
        assert_eq!(declaration.extends, "epok::ActorComponent");
        assert!(declaration.properties.is_empty());
        // The empty lifecycle body is the only declared member: a synthesized
        // override of the parent's begin_play, nothing from the comments.
        assert_eq!(
            declaration
                .functions
                .iter()
                .map(|f| f.name.as_str())
                .collect::<Vec<_>>(),
            ["begin_play"]
        );
        // The engine assigns the identity and the profile: the author is never
        // asked to type either one.
        assert_eq!(declaration.id, "1f4d9c8e-2b3a-4c5d-8e6f-7a8b9c0d1e2f");
        assert_eq!(declaration.profile, PROFILE_VERSION);
    }

    /// The identity a script does not spell out is assigned by the engine,
    /// deterministically, exactly as a C++ declaration without `Id=` is
    /// identified by its USR.
    #[test]
    fn lua_identities_are_derived_when_the_script_declares_none() {
        let source = r#"
local Drone = epok.class {
    name = "Drone",
    extends = "epok::ActorComponent",
    properties = {
        speed = { type = "Fixed", default = 1.0, editable = true },
        armed = { type = "Bool", default = false, editable = true }
    },
    functions = {
        reset = { callable = true, parameters = {}, returns = "void" }
    }
}
function Drone:reset()
end
function Drone:begin_play()
end
return Drone
"#;
        let file = file(source);
        let decl = extract(&file).unwrap();
        // Missing `profile` means the profile this editor supports.
        assert_eq!(decl.profile, PROFILE_VERSION);
        assert_eq!(decl.id, "lua:Drone");
        assert!(derived(&decl.id) && identity(&decl.id) && !canonical(&decl.id));
        let member = |name: &str| {
            decl.properties
                .iter()
                .map(|p| (&p.name, &p.id))
                .chain(decl.functions.iter().map(|f| (&f.name, &f.id)))
                .find(|(member, _)| *member == name)
                .map(|(_, id)| id.clone())
                .unwrap()
        };
        assert_eq!(member("speed"), "lua:Drone:speed");
        assert_eq!(member("armed"), "lua:Drone:armed");
        assert_eq!(member("reset"), "lua:Drone:reset");
        // A bare lifecycle method is identified by the same scheme, so
        // promoting it into `functions` does not change its identity.
        assert_eq!(member("begin_play"), "lua:Drone:begin_play");
        // Distinct per member, and reproducible across extractions.
        let again = extract(&file).unwrap();
        assert_eq!(decl.id, again.id);
        let ids = decl
            .properties
            .iter()
            .map(|p| p.id.clone())
            .chain(decl.functions.iter().map(|f| f.id.clone()))
            .collect::<BTreeSet<_>>();
        assert_eq!(ids.len(), decl.properties.len() + decl.functions.len());
        assert!(ids.iter().all(|id| identity(id)));
        // Derived identities never reach the filesystem with their separator.
        assert_eq!(artifact_stem(&decl.id), "lua_Drone");
        assert_eq!(
            artifact_stem("1f4d9c8e-2b3a-4c5d-8e6f-7a8b9c0d1e2f"),
            "1f4d9c8e-2b3a-4c5d-8e6f-7a8b9c0d1e2f"
        );
        declarations(&decl, &file, &registry()).unwrap();
    }

    /// An explicit id still wins, and it must still be a canonical UUID. A
    /// class pinned to a UUID passes that stability on to its members.
    #[test]
    fn lua_explicit_identities_win_and_must_be_canonical_uuids() {
        let pinned = file(
            r#"
local Drone = epok.class {
    profile = 1,
    id = "6a1e8f20-4c3d-4b5e-9f70-1a2b3c4d5e6f",
    name = "Drone",
    extends = "epok::ActorComponent",
    properties = {
        speed = { id = "7b2f9031-5d4e-4c6f-8a81-2b3c4d5e6f70",
            type = "Fixed", default = 1.0, editable = true },
        armed = { type = "Bool", default = false, editable = true }
    },
    functions = {}
}
return Drone
"#,
        );
        let decl = extract(&pinned).unwrap();
        assert_eq!(decl.id, "6a1e8f20-4c3d-4b5e-9f70-1a2b3c4d5e6f");
        assert_eq!(
            decl.properties[0].id,
            "7b2f9031-5d4e-4c6f-8a81-2b3c4d5e6f70"
        );
        assert_eq!(
            decl.properties[1].id,
            "lua:6a1e8f20-4c3d-4b5e-9f70-1a2b3c4d5e6f:armed"
        );
        declarations(&decl, &pinned, &registry()).unwrap();

        let message = |source: String| extract(&file(&source)).unwrap_err().message;
        assert!(
            message(
                HEADER.replace(
                    "3d352b2b-c2d7-4b99-9ba1-a003d648e897",
                    "lua:EnemyLogic:health"
                ) + "
return EnemyLogic
"
            )
            .contains("Property health id must be a canonical UUID")
        );
        assert!(
            message(
                HEADER.replace("224e6b46-e4b4-475c-9744-8b9fb4c0baaa", "not-a-uuid")
                    + "
return EnemyLogic
"
            )
            .contains("Function damage id must be a canonical UUID")
        );
        // A `profile` that is present must still name the supported profile.
        assert!(
            message(
                HEADER.replace("profile = 1", "profile = 2")
                    + "
return EnemyLogic
"
            )
            .contains("Profile 2 is not epok-lua v1")
        );
        assert!(
            message(
                HEADER.replace("profile = 1", "profile = \"v1\"")
                    + "
return EnemyLogic
"
            )
            .contains("profile must be a version number")
        );
    }
}
