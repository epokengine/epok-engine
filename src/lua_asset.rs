//! `.lua` discovery and the statically extracted class declaration.
//!
//! A class is declared by `local <Class> = <Parent>:extend()`, its properties by
//! `<Class>.<name> = <value>` assignments and its functions by the methods
//! written on the same local. All of it is read from the AST: the engine never
//! executes a line of user Lua to discover a class, because the editor must be
//! able to list classes, properties and functions without a Lua interpreter and
//! the same metadata must be identical in every build.
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
    /// Class identity recorded for this path in
    /// `ProjectSettings/LuaClasses.epoksettings`. `None` means the project has
    /// no entry yet and the class is identified by its derived `lua:<Name>`.
    pub id: Option<String>,
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
                result.push(LuaFile {
                    path,
                    source,
                    id: None,
                });
            }
        }
        Ok(())
    }
    let mut result = vec![];
    visit(&root.join("assets/scripts"), &mut result)?;
    result.sort_by(|a, b| a.path.cmp(&b.path));
    crate::lua_identity::resolve(root, &mut result)?;
    Ok(result)
}

// ----------------------------------------------------------- extraction ----

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
/// The type of a member written as a bare literal: `90.0` is `Fixed`, `true` is
/// `Bool`, `3` is `Int32`, and a unary minus keeps the literal's own type.
/// Anything else — a string, an expression such as `1 + 1`, a table — is not a
/// literal and has no shorthand type.
pub fn literal_type(expr: &ast::Expr) -> Option<Type> {
    let number = |integer: bool| Some(if integer { Type::Int32 } else { Type::Fixed });
    match expr {
        ast::Expr::Bool(..) => Some(Type::Bool),
        ast::Expr::Number { integer, .. } => number(*integer),
        ast::Expr::Unary {
            op: ast::UnOp::Neg,
            operand,
            ..
        } => match &**operand {
            ast::Expr::Number { integer, .. } => number(*integer),
            _ => None,
        },
        _ => None,
    }
}

/// The type vocabulary as an annotation spells it. `value_type` is the whole
/// set; on top of it the Lua Language Server definitions name a reflected class
/// with `.` where C++ writes `::`, so `ActorRef<epok.Actor3D>` — the exact text
/// the generated stub shows in completion — resolves to the same type as
/// `ActorRef<epok::Actor3D>`.
pub fn annotation_type(name: &str) -> Option<Type> {
    // No name in the vocabulary contains a `.`, so the rewrite is total and
    // touches nothing but a dotted class operand.
    value_type(&name.replace('.', "::"))
}
/// A literal default: a number or a boolean, with a unary minus allowed. The
/// declaration is data read from the source text, never an evaluated
/// expression, so nothing else can spell a default.
fn literal_value(expr: &ast::Expr, ty: &Type, file: &Path) -> Result<Value, Diagnostic> {
    let invalid = || Diagnostic::new(file, expr.span(), "Default is not a literal value");
    Ok(match expr {
        ast::Expr::Bool(value, _) => Value::from(*value),
        ast::Expr::Number { value, .. } => number(*value, ty),
        ast::Expr::Unary {
            op: ast::UnOp::Neg,
            operand,
            ..
        } => match &**operand {
            ast::Expr::Number { value, .. } => number(-*value, ty),
            _ => return Err(invalid()),
        },
        _ => return Err(invalid()),
    })
}

fn number(value: f64, ty: &Type) -> Value {
    match ty {
        Type::Int32 => Value::from(value as i64),
        // A negative literal is kept negative so the bounds check rejects it,
        // rather than wrapping silently into an unsigned default.
        Type::UInt32 if value < 0.0 => Value::from(value as i64),
        Type::UInt32 => Value::from(value as u64),
        Type::Enum { .. } => Value::from(value as i64),
        _ => Value::from(value),
    }
}

pub fn canonical(id: &str) -> bool {
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
/// `---@id <uuid>` above a property assignment or a method pins that member's
/// identity. It is optional everywhere: without it the engine derives one. When
/// written it must be a canonical UUID, because the derived form is assigned by
/// the engine and never typed.
fn annotated_id(
    file: &Path,
    annotations: &[ast::Annotation],
    what: &str,
) -> Result<Option<String>, Diagnostic> {
    let Some(annotation) = annotations.iter().find(|a| a.tag == "id") else {
        return Ok(None);
    };
    match annotation.operands.first() {
        Some(id) if canonical(id) && annotation.operands.len() == 1 => Ok(Some(id.clone())),
        _ => Err(Diagnostic::new(
            file,
            annotation.span,
            format!("{what} id must be a canonical UUID"),
        )),
    }
}

/// Asked of a method that has parameters but describes none of them. The text
/// names the annotation an author writes, not the tooling it also serves.
pub const PARAMETER_ANNOTATIONS: &str =
    "Declare the parameter types with ---@param annotations, for example ---@param amount Fixed";

/// A signature read from the `---@` block above a method. The tags are the ones
/// the Lua Language Server already understands, so the same lines type the
/// method in the editor and declare it to the compiler.
#[derive(Clone, Debug, Default)]
pub struct Annotated {
    pub parameters: Vec<(String, Type)>,
    pub returns: Option<Type>,
    /// `---@override`: the equivalent of an `overrides = "..."` entry.
    pub overrides: bool,
}
impl Annotated {
    /// Whether the block states a signature, as opposed to only marking the
    /// method as an override whose signature comes from the parent.
    fn typed(&self) -> bool {
        !self.parameters.is_empty() || self.returns.is_some()
    }
    /// The annotations and an explicit `functions` entry describe one function,
    /// so they must agree; a silent winner would make the editor and the
    /// language server disagree about the same method.
    fn agrees_with(
        &self,
        file: &Path,
        method: &ast::Method,
        declared: &DeclaredFunction,
    ) -> Result<(), Diagnostic> {
        let conflict = |what: &str| {
            Err(Diagnostic::new(
                file,
                method.span,
                format!(
                    "{}: the annotated {what} differs from its `functions` entry",
                    method.name
                ),
            ))
        };
        if self.overrides && declared.overrides.is_none() {
            return conflict("override");
        }
        if self.typed() {
            if self.parameters != declared.parameters {
                return conflict("parameters");
            }
            if self.returns.clone().unwrap_or(Type::Void) != declared.returns {
                return conflict("return type");
            }
        }
        Ok(())
    }
}

/// The `---@param`, `---@return` and `---@override` lines above a method.
/// Returns `None` when the block declares nothing about the signature: any
/// other tag, and any ordinary comment, is left to the language server.
fn annotated_signature(file: &Path, method: &ast::Method) -> Result<Option<Annotated>, Diagnostic> {
    let mut out = Annotated::default();
    let mut seen = false;
    for annotation in &method.annotations {
        let fail = |message: String| Diagnostic::new(file, annotation.span, message);
        let named = |index: usize| -> Result<Type, Diagnostic> {
            let name = annotation.operands.get(index).ok_or_else(|| {
                fail(format!(
                    "---@{} needs a type from the epok value vocabulary",
                    annotation.tag
                ))
            })?;
            annotation_type(name)
                .ok_or_else(|| fail(format!("{name} is not an epok-lua type name")))
        };
        match annotation.tag.as_str() {
            "param" => {
                seen = true;
                let name = annotation
                    .operands
                    .first()
                    .ok_or_else(|| fail("---@param needs a parameter name and a type".into()))?;
                out.parameters.push((name.clone(), named(1)?));
            }
            "return" => {
                seen = true;
                if out.returns.is_some() {
                    return Err(fail(format!(
                        "{}: a method returns at most one value",
                        method.name
                    )));
                }
                out.returns = Some(named(0)?);
            }
            "override" => {
                seen = true;
                out.overrides = true;
            }
            _ => {}
        }
    }
    if !seen {
        return Ok(None);
    }
    // One `---@param` per parameter, in the order they are written: the
    // annotations name the very parameters of the method they sit above.
    if !out.parameters.is_empty() && out.parameters.len() != method.parameters.len() {
        return Err(Diagnostic::new(
            file,
            method.span,
            format!(
                "{} takes {} parameter(s) but declares {} with ---@param",
                method.name,
                method.parameters.len(),
                out.parameters.len()
            ),
        ));
    }
    for ((annotated, _), (written, span)) in out.parameters.iter().zip(&method.parameters) {
        if annotated != written {
            return Err(Diagnostic::new(
                file,
                *span,
                format!("Parameter {written} is annotated as {annotated}"),
            ));
        }
    }
    Ok(Some(out))
}

/// A dotted name written in Lua, as the registry spells it: `epok.Actor3D` is
/// `epok::Actor3D` and `game.Enemy` is `game::Enemy`. A bare identifier names a
/// project class at global scope and is returned unchanged.
fn dotted_name(expr: &ast::Expr) -> Option<String> {
    match expr {
        ast::Expr::Name { name, .. } => Some(name.clone()),
        ast::Expr::Field { base, name, .. } => Some(format!("{}::{name}", dotted_name(base)?)),
        _ => None,
    }
}
/// The `epok.<Name>(` constructor a property value is written with, if any.
fn constructor(expr: &ast::Expr) -> Option<(&str, &[ast::Expr], Span)> {
    let ast::Expr::Call { base, args, span } = expr else {
        return None;
    };
    let ast::Expr::Field {
        base: root, name, ..
    } = &**base
    else {
        return None;
    };
    matches!(&**root, ast::Expr::Name { name, .. } if name == "epok").then_some((
        name.as_str(),
        args.as_slice(),
        *span,
    ))
}

/// A property as the source text declares it: its type, its default and whether
/// the Inspector shows it.
#[derive(Clone, Debug)]
pub struct Shape {
    pub value_type: Type,
    pub default: Value,
    pub editable: bool,
}

/// The declared shape of `<Class>.<name> = <value>`.
///
/// A bare literal says everything about the simple cases; each remaining kind
/// of value has one `epok.<Type>(...)` constructor, so the vocabulary stays the
/// closed set `value_type` already names and the file is still pure data.
pub fn shape(expr: &ast::Expr, file: &Path) -> Result<Shape, Diagnostic> {
    let editable = |value_type: Type, default: Value| {
        Ok(Shape {
            value_type,
            default,
            editable: true,
        })
    };
    if let Some(value_type) = literal_type(expr) {
        let default = literal_value(expr, &value_type, file)?;
        return editable(value_type, default);
    }
    let Some((name, args, span)) = constructor(expr) else {
        return Err(Diagnostic::new(
            file,
            expr.span(),
            format!(
                "A property is a literal default or an epok value constructor such as {}",
                "epok.UInt32(0), epok.ActorRef(EnemyBase) or epok.Hidden(1.0)"
            ),
        ));
    };
    let fail = |message: String| Diagnostic::new(file, span, message);
    let arity = |wanted: usize| {
        (args.len() == wanted)
            .then_some(())
            .ok_or_else(|| fail(format!("epok.{name} takes {wanted} argument(s)")))
    };
    // An optional class operand narrows a reference; without one the reference
    // accepts any class, exactly as `ActorRef` does in an annotation.
    let narrowed = |kind: fn(Option<String>) -> Type| {
        if args.is_empty() {
            return Ok(kind(None));
        }
        if args.len() != 1 {
            return Err(fail(format!("epok.{name} takes an optional class name")));
        }
        let class = dotted_name(&args[0])
            .ok_or_else(|| fail(format!("epok.{name} takes a class name, written unquoted")))?;
        Ok(kind(Some(class)))
    };
    let text = |index: usize| match args.get(index) {
        Some(ast::Expr::Str { value, .. }) => Ok(value.clone()),
        _ => Err(fail(format!("epok.{name} takes a quoted name"))),
    };
    match name {
        "Bool" => {
            arity(1)?;
            editable(Type::Bool, literal_value(&args[0], &Type::Bool, file)?)
        }
        "Int32" | "UInt32" | "Fixed" => {
            arity(1)?;
            let value_type = value_type(name).expect("named above");
            let default = literal_value(&args[0], &value_type, file)?;
            editable(value_type, default)
        }
        "Vector2" | "Vector3" => {
            let length = if name == "Vector2" { 2 } else { 3 };
            arity(length)?;
            let mut components = vec![];
            for argument in args {
                components.push(literal_value(argument, &Type::Fixed, file)?);
            }
            editable(Type::Vector { length }, Value::Array(components))
        }
        // The variants are reflected, so the enum is resolved against the
        // registry when the declaration is published, not here.
        "Enum" => {
            arity(2)?;
            let cpp_name = dotted_name(&args[0]).ok_or_else(|| {
                fail("epok.Enum takes a reflected enum name and a variant".into())
            })?;
            editable(
                Type::Enum {
                    cpp_name,
                    variants: BTreeMap::new(),
                },
                Value::from(text(1)?),
            )
        }
        "ActorRef" => editable(narrowed(|class| Type::ActorRef { class })?, Value::Null),
        "ComponentRef" => editable(narrowed(|class| Type::ComponentRef { class })?, Value::Null),
        "ObjectRef" => editable(narrowed(|class| Type::ObjectRef { class })?, Value::Null),
        "AssetRef" => {
            arity(1)?;
            editable(Type::AssetRef { kind: text(0)? }, Value::Null)
        }
        "ClassRef" => {
            arity(1)?;
            let base = dotted_name(&args[0])
                .ok_or_else(|| fail("epok.ClassRef takes a base class name".into()))?;
            editable(Type::ClassRef { base }, Value::Null)
        }
        // The one wrapper: it changes nothing but the Inspector.
        "Hidden" => {
            arity(1)?;
            let inner = shape(&args[0], file)?;
            if !inner.editable {
                return Err(fail("epok.Hidden wraps a value once".into()));
            }
            Ok(Shape {
                editable: false,
                ..inner
            })
        }
        _ => Err(fail(format!(
            "epok.{name} is not an epok value constructor"
        ))),
    }
}

/// `---@class <Name> : <Base>` above the class local. It is what gives the Lua
/// Language Server the type of the local, so the template always writes it, but
/// the compiler reads the class from the file name and the `extend()` call: the
/// annotation only has to agree with them.
fn annotated_class(
    file: &Path,
    annotations: &[ast::Annotation],
    name: &str,
    extends: &str,
) -> Result<(), Diagnostic> {
    let Some(annotation) = annotations.iter().find(|a| a.tag == "class") else {
        return Ok(());
    };
    let fail = |message: String| Diagnostic::new(file, annotation.span, message);
    let written = annotation.operands.concat();
    let (declared, base) = written
        .split_once(':')
        .ok_or_else(|| fail(format!("---@class must read ---@class {name} : {extends}")))?;
    if declared != name {
        return Err(fail(format!(
            "---@class names {declared}; the file names the class {name}"
        )));
    }
    if base.replace('.', "::") != extends {
        return Err(fail(format!(
            "---@class extends {base}; the declaration extends {extends}"
        )));
    }
    Ok(())
}

pub fn extract(file: &LuaFile) -> Result<Declaration, Diagnostic> {
    let chunk = crate::lua_frontend::parse(file)?;
    let path = file.path.as_path();
    let top = Span { line: 1, column: 1 };

    // `local <Class> = <Parent>:extend()` — the whole declaration head.
    let mut locals = chunk.locals.iter();
    let Some(local) = locals.next() else {
        return Err(Diagnostic::new(path, top, DECLARATION));
    };
    if let Some(extra) = locals.next() {
        return Err(Diagnostic::new(
            path,
            extra.span,
            "A Lua script declares exactly one class",
        ));
    }
    let binding = local.name.clone();
    let span = local.span;
    if let ast::Expr::Call { base, .. } = &local.value
        && dotted_name(base).as_deref() == Some("epok::class")
    {
        return Err(Diagnostic::new(path, span, RETIRED_TABLE));
    }
    let ast::Expr::MethodCall {
        base,
        name: method,
        args,
        ..
    } = &local.value
    else {
        return Err(Diagnostic::new(path, span, DECLARATION));
    };
    if method != "extend" || !args.is_empty() {
        return Err(Diagnostic::new(path, span, DECLARATION));
    }
    let extends = dotted_name(base).ok_or_else(|| Diagnostic::new(path, span, PARENT))?;

    match &chunk.returns {
        Some((name, _)) if name == &binding => {}
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

    // The class is named by its file, which is what an author already has to
    // keep unique, and the local must be bound to that same name so the
    // qualified parent call and the language server agree with the compiler.
    let stem = path
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default();
    if !crate::scripts::identifier(&stem) || stem.starts_with("epok_") {
        return Err(Diagnostic::new(
            path,
            span,
            format!("The file name is not a usable class name ({stem})"),
        ));
    }
    if binding != stem {
        return Err(Diagnostic::new(
            path,
            span,
            format!("The class local is named {binding}; {stem}.lua declares {stem}"),
        ));
    }
    let name = stem;
    annotated_class(path, &local.annotations, &name, &extends)?;

    // Identity comes from ProjectSettings/LuaClasses.epoksettings, which the
    // editor maintains, so renaming the file never changes the class identity.
    // A file with no entry yet is identified by its name until the next catalog
    // refresh records one.
    let id = file.id.clone().unwrap_or_else(|| derived_class_id(&name));

    let mut properties = vec![];
    for assignment in &chunk.assignments {
        if assignment.object != binding {
            return Err(Diagnostic::new(
                path,
                assignment.span,
                format!("Properties must be declared on {binding}"),
            ));
        }
        let shape = shape(&assignment.value, path)?;
        properties.push(DeclaredProperty {
            id: annotated_id(
                path,
                &assignment.annotations,
                &format!("Property {}", assignment.name),
            )?
            .unwrap_or_else(|| derived_member_id(&id, &assignment.name)),
            name: assignment.name.clone(),
            value_type: shape.value_type,
            default: shape.default,
            editable: shape.editable,
            span: assignment.span,
        });
    }

    let mut functions = vec![];
    for method in &chunk.methods {
        if method.object != binding {
            return Err(Diagnostic::new(
                path,
                method.span,
                format!("Methods must be declared on {binding}"),
            ));
        }
        let annotated = annotated_signature(path, method)?;
        let lifecycle = LIFECYCLE.contains(&method.name.as_str());
        let (parameters, returns, overrides, inferred) = match &annotated {
            // A reflected event override takes its whole signature from the
            // parent, so it needs no annotation at all.
            None if lifecycle => (vec![], Type::Void, Some(method.name.clone()), true),
            // A method with nothing to describe is a new `void` callable.
            None if method.parameters.is_empty() => (vec![], Type::Void, None, false),
            None => {
                return Err(Diagnostic::new(path, method.span, PARAMETER_ANNOTATIONS));
            }
            Some(signature) => {
                let overrides = (signature.overrides || lifecycle).then(|| method.name.clone());
                let inferred = overrides.is_some() && !signature.typed();
                (
                    signature.parameters.clone(),
                    signature.returns.clone().unwrap_or(Type::Void),
                    overrides,
                    inferred,
                )
            }
        };
        functions.push(DeclaredFunction {
            id: annotated_id(
                path,
                &method.annotations,
                &format!("Function {}", method.name),
            )?
            .unwrap_or_else(|| derived_member_id(&id, &method.name)),
            name: method.name.clone(),
            parameters,
            returns,
            callable: overrides.is_none(),
            overrides,
            inferred,
            span: method.span,
        });
    }

    Ok(Declaration {
        file: file.path.clone(),
        id,
        name,
        extends,
        properties,
        functions,
        binding,
        span,
    })
}

/// `ref` and `super` name the intrinsic receivers of the profile, so neither is
/// available as a member name.
pub const RESERVED_MEMBERS: &str =
    "ref and super are reserved member names in the epok-lua profile";

/// The one declaration head a `.lua` class file may open with.
pub const DECLARATION: &str =
    "A Lua class opens with `local <Class> = <Parent>:extend()`, where <Class> is the file name";
/// Files written by an earlier editor open with a metadata table; the class
/// they describe is recoverable, so the message says what to write instead.
pub const RETIRED_TABLE: &str = "This file opens with the retired `epok.class { ... }` metadata table. Rewrite it as `local <Class> = <Parent>:extend()` with properties as `<Class>.<name> = <default>` assignments; the editor keeps the class id in ProjectSettings/LuaClasses.epoksettings";
/// Written when the parent is not a name at all; an unknown but well-formed
/// name is resolved, and reported, against the registry.
pub const PARENT: &str =
    "The parent is `epok.<EngineClass>`, or the name of a project C++ or Lua class";

// ---------------------------------------------------------- declarations ----

/// A reflected enum, found by name. Enums are not declared on their own: the
/// registry spells one out wherever a member uses it, so an authored
/// `epok.Enum(Mode, "Idle")` is resolved by looking for that spelling.
fn reflected_enum(registry: &Registry, cpp_name: &str) -> Option<Type> {
    fn walk(ty: &Type, cpp_name: &str) -> Option<Type> {
        match ty {
            Type::Enum {
                cpp_name: name,
                variants,
            } if name == cpp_name && !variants.is_empty() => Some(ty.clone()),
            Type::Record { fields, .. } => fields
                .iter()
                .find_map(|field| walk(&field.value_type, cpp_name)),
            _ => None,
        }
    }
    registry.classes.values().find_map(|class| {
        class
            .properties
            .iter()
            .map(|p| &p.value_type)
            .chain(class.functions.iter().flat_map(|f| {
                f.parameters
                    .iter()
                    .map(|p| &p.value_type)
                    .chain(std::iter::once(&f.returns))
            }))
            .find_map(|ty| walk(ty, cpp_name))
    })
}

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
        // `epok.Enum(Mode, "Idle")` names a reflected enum and one of its
        // variants; the variants themselves live in the registry, so the type
        // and the default are completed here rather than during extraction.
        let (value_type, default) = match (&property.value_type, &property.default) {
            (Type::Enum { cpp_name, variants }, Value::String(variant)) if variants.is_empty() => {
                let Some(resolved) = reflected_enum(registry, cpp_name) else {
                    return Err(fail(
                        property.span,
                        format!("{cpp_name} is not a reflected enum"),
                    ));
                };
                let Type::Enum { variants, .. } = &resolved else {
                    unreachable!("reflected_enum returns an enum")
                };
                let Some(value) = variants.get(variant).copied() else {
                    return Err(fail(
                        property.span,
                        format!("{cpp_name} has no variant named {variant}"),
                    ));
                };
                (resolved.clone(), Value::from(value))
            }
            (value_type, default) => (value_type.clone(), default.clone()),
        };
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
        // `self.ref` is the object's own typed reference and `<Class>.super` the
        // qualified parent receiver: a property of either name would be
        // addressed as the intrinsic and never read.
        if property.name == crate::lua_frontend::SELF_REFERENCE || property.name == "super" {
            return Err(fail(
                property.span,
                format!("{RESERVED_MEMBERS}: {} cannot be declared", property.name),
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
        if !crate::script_values::valid(&default, &value_type)
            || matches!(value_type, Type::Vector { length } if length != 2 && length != 3)
            || value_type == Type::Void
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
            value_type,
            default,
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
/// Source of a freshly created class.
///
/// `---@class` is what gives the Lua Language Server the type of the local, so
/// the template always writes it even though the compiler reads the class from
/// the file name and the `extend()` call. The comments show the two remaining
/// shapes — a reflected event and a new callable — without declaring anything
/// the author has to delete. Tests patch the `speed` property line and the
/// empty `begin_play` body.
pub fn template(name: &str, parent_cpp_name: &str) -> String {
    let parent = parent_cpp_name.replace("::", ".");
    format!(
        "---@class {name} : {parent}\n\
local {name} = {parent}:extend()\n\
\n\
-- Properties are plain assignments, read from the source text and never\n\
-- executed. A literal declares the type and the default; everything a literal\n\
-- cannot say has a constructor, such as epok.UInt32(0), epok.Vector3(0.0, 1.0,\n\
-- 0.0), epok.ActorRef(SomeClass) or epok.Hidden(1.0) for a field the Inspector\n\
-- does not show.\n\
{name}.speed = 1.0\n\
\n\
-- Lifecycle events take their signature from the parent: begin_play, tick,\n\
-- end_play, on_enable and on_disable.\n\
function {name}:begin_play()\nend\n\
\n\
-- function {name}:tick(delta_seconds)\n\
--     {name}.super.tick(self, delta_seconds) -- the qualified parent call\n\
-- end\n\
\n\
-- A method other classes and Blueprints may call declares its signature with\n\
-- the annotations the language server already reads:\n\
-- ---@param amount Fixed\n\
-- ---@return Fixed\n\
-- function {name}:take_damage(amount)\n\
--     return amount\n\
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
    let source = template(name, parent_cpp_name);
    let path = dir.join(format!("{name}.lua"));
    let result = (|| {
        // The identity is recorded before the file exists, so the very first
        // catalog refresh already sees the class the editor assigned.
        crate::lua_identity::assign(root, &path)?;
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
        let _ = crate::lua_identity::removed(
            root,
            &crate::lua_identity::relative(root, &path)
                .into_iter()
                .collect::<Vec<_>>(),
        );
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
    /// The identity `ProjectSettings/LuaClasses.epoksettings` records for the
    /// `EnemyLogic.lua` fixture, and the one the `Guard.lua` fixture carries.
    pub(crate) const CLASS_ID: &str = "956f4946-0c61-42f8-899e-2db063b42420";
    pub(crate) const GUARD_ID: &str = "b7c9d1e3-4f5a-4b6c-8d7e-9f0a1b2c3d4e";
    /// The shared declaration head: the class, its properties, and nothing else.
    /// Methods are appended by each test, because in the declaration form a
    /// function *is* its definition.
    pub(crate) const HEADER: &str = r#"
---@class EnemyLogic : epok.ActorComponent
local EnemyLogic = epok.ActorComponent:extend()

---@id 3d352b2b-c2d7-4b99-9ba1-a003d648e897
EnemyLogic.health = 100.0
---@id 5c1a5a1e-1d0e-4f3a-9a2b-1c2d3e4f5a6b
EnemyLogic.charges = epok.UInt32(3)
---@id 7f9c0b1d-2e3f-4a5b-8c9d-0e1f2a3b4c5d
EnemyLogic.ready = true
"#;
    /// `damage` as the fixtures declare it, so a chunk that defines the body
    /// still carries the annotated signature the compiler reads.
    pub(crate) const DAMAGE: &str =
        "---@id 224e6b46-e4b4-475c-9744-8b9fb4c0baaa\n---@param amount Fixed\n";

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

    /// A discovered script with the identity the project records for it. The
    /// two fixture classes are pinned; anything else is an unrecorded file and
    /// falls back to its derived `lua:<Name>` identity, as a hand-copied script
    /// does before the next catalog refresh adopts it.
    pub(crate) fn fixture(path: &str, source: &str) -> LuaFile {
        let stem = Path::new(path)
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        LuaFile {
            path: path.into(),
            source: source.into(),
            id: match stem.as_str() {
                "EnemyLogic" => Some(CLASS_ID.into()),
                "Guard" => Some(GUARD_ID.into()),
                _ => None,
            },
        }
    }
    pub(crate) fn file(source: &str) -> LuaFile {
        fixture("assets/scripts/EnemyLogic.lua", source)
    }
    /// The same fixture with no recorded identity, so extraction derives one.
    fn unrecorded(path: &str, source: &str) -> LuaFile {
        LuaFile {
            path: path.into(),
            source: source.into(),
            id: None,
        }
    }
    fn example() -> LuaFile {
        file(&format!(
            "{HEADER}\n{DAMAGE}function EnemyLogic:damage(amount)\n    self.health = self.health - amount\nend\n\nreturn EnemyLogic\n"
        ))
    }
    const TAIL: &str = "\nreturn EnemyLogic\n";

    #[test]
    fn lua_asset_extracts_the_documented_declaration() {
        let decl = extract(&example()).unwrap();
        assert_eq!(decl.id, CLASS_ID);
        assert_eq!(decl.name, "EnemyLogic");
        assert_eq!(decl.extends, "epok::ActorComponent");
        assert_eq!(decl.binding, "EnemyLogic");
        let health = decl.properties.iter().find(|p| p.name == "health").unwrap();
        assert_eq!(health.value_type, Type::Fixed);
        assert_eq!(health.default, serde_json::json!(100.0));
        assert!(health.editable);
        assert_eq!(health.id, "3d352b2b-c2d7-4b99-9ba1-a003d648e897");
        let damage = decl.functions.iter().find(|f| f.name == "damage").unwrap();
        assert_eq!(damage.parameters, vec![("amount".into(), Type::Fixed)]);
        assert_eq!(damage.returns, Type::Void);
        assert!(damage.callable && damage.overrides.is_none());
        assert_eq!(damage.id, "224e6b46-e4b4-475c-9744-8b9fb4c0baaa");

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

    /// The declaration head: one class per file, named by the file, bound to a
    /// local the whole file agrees on, and returned at the end.
    #[test]
    fn lua_asset_requires_the_declaration_shape() {
        let message = |source: String| extract(&file(&source)).unwrap_err().message;
        assert!(message(HEADER.into()).contains("must end with `return EnemyLogic`"));
        assert!(
            message(format!("{HEADER}{HEADER}{TAIL}")).contains("exactly one class"),
            "a second local is a second class"
        );
        assert!(
            message(format!("local EnemyLogic = epok.ActorComponent{TAIL}")).contains(DECLARATION),
            "the parent must actually be extended"
        );
        assert!(
            message("local Other = epok.ActorComponent:extend()\nreturn Other\n".into())
                .contains("EnemyLogic.lua declares EnemyLogic"),
            "the local is named after the file"
        );
        assert!(
            message(format!(
                "{HEADER}\nfunction EnemyLogic.damage(amount)\nend{TAIL}"
            ))
            .contains("must be declared as `function EnemyLogic:damage`")
        );
        // A property written after the first method would be read out of order.
        assert!(
            message(format!(
                "{HEADER}\nfunction EnemyLogic:begin_play()\nend\nEnemyLogic.late = 1{TAIL}"
            ))
            .contains("declared before the first method")
        );
        // Nothing else may appear at file scope.
        assert!(
            message(format!("{HEADER}\nEnemyLogic.speed = 1.0\nprint(1){TAIL}"))
                .contains(crate::lua_frontend::profile::FILE_SCOPE)
        );
        // A file whose stem is not a C++ identifier cannot name a class.
        for stem in ["enemy-logic", "2Fast", "class"] {
            let source = "local X = epok.ActorComponent:extend()\nreturn X\n";
            let message = extract(&unrecorded(&format!("assets/scripts/{stem}.lua"), source))
                .unwrap_err()
                .message;
            assert!(
                message.contains("not a usable class name") && message.contains(stem),
                "{stem}: {message}"
            );
        }
    }

    /// The parent expression: an engine class, a project C++ class, another Lua
    /// class, or a namespaced C++ class written with `.` for `::`.
    #[test]
    fn lua_asset_resolves_every_eligible_parent_spelling() {
        let extends = |parent: &str| {
            extract(&file(&format!(
                "local EnemyLogic = {parent}:extend(){TAIL}"
            )))
            .unwrap()
            .extends
        };
        assert_eq!(extends("epok.ActorComponent"), "epok::ActorComponent");
        assert_eq!(extends("EnemyBase"), "EnemyBase");
        assert_eq!(extends("game.Enemy"), "game::Enemy");

        // The registry has the last word on which of those actually exist.
        let unknown = file(&format!(
            "local EnemyLogic = epok.Nonexistent:extend(){TAIL}"
        ));
        assert!(
            declarations(&extract(&unknown).unwrap(), &unknown, &registry())
                .unwrap_err()
                .message
                .contains("Unknown parent epok::Nonexistent")
        );
        let sealed = file(&format!("local EnemyLogic = SealedBase:extend(){TAIL}"));
        assert!(
            declarations(&extract(&sealed).unwrap(), &sealed, &registry())
                .unwrap_err()
                .message
                .contains("does not support native Lua inheritance")
        );
        // A parent that is not a name at all never reaches the registry.
        assert!(
            extract(&file(&format!(
                "local EnemyLogic = epok.ActorComponent:extend():extend(){TAIL}"
            )))
            .unwrap_err()
            .message
            .contains(PARENT)
        );
    }

    /// `---@class` is optional for compiling, and when written it must say the
    /// same thing as the file name and the `extend()` call.
    #[test]
    fn lua_asset_checks_the_class_annotation_against_the_declaration() {
        let with = |annotation: &str| {
            extract(&file(&format!(
                "{annotation}\nlocal EnemyLogic = epok.ActorComponent:extend(){TAIL}"
            )))
        };
        // Absent: the file still compiles, because the file names the class.
        assert_eq!(
            extract(&file(&format!(
                "local EnemyLogic = epok.ActorComponent:extend(){TAIL}"
            )))
            .unwrap()
            .name,
            "EnemyLogic"
        );
        with("---@class EnemyLogic : epok.ActorComponent").unwrap();
        with("---@class EnemyLogic:epok.ActorComponent").unwrap();
        assert!(
            with("---@class Other : epok.ActorComponent")
                .unwrap_err()
                .message
                .contains("---@class names Other")
        );
        assert!(
            with("---@class EnemyLogic : epok.Object")
                .unwrap_err()
                .message
                .contains("---@class extends epok.Object")
        );
        assert!(
            with("---@class EnemyLogic")
                .unwrap_err()
                .message
                .contains("must read ---@class EnemyLogic : epok::ActorComponent")
        );
    }

    /// Every literal and every typed constructor, as the reference table in
    /// `docs/lua-scripting.md` lists them.
    #[test]
    fn lua_asset_reads_literals_and_typed_constructors() {
        let single = |value: &str| {
            let mut all = extract(&file(&format!(
                "local EnemyLogic = epok.ActorComponent:extend()\nEnemyLogic.member = {value}{TAIL}"
            )))
            .unwrap()
            .properties;
            assert_eq!(all.len(), 1, "{value}");
            all.remove(0)
        };
        let shape = |value: &str| {
            let p = single(value);
            (p.value_type, p.default, p.editable)
        };
        // Literals.
        assert_eq!(shape("90.0"), (Type::Fixed, serde_json::json!(90.0), true));
        assert_eq!(shape("true"), (Type::Bool, serde_json::json!(true), true));
        assert_eq!(shape("3"), (Type::Int32, serde_json::json!(3), true));
        assert_eq!(shape("-2.5"), (Type::Fixed, serde_json::json!(-2.5), true));
        assert_eq!(shape("-7"), (Type::Int32, serde_json::json!(-7), true));
        // Typed constructors.
        assert_eq!(
            shape("epok.Bool(false)"),
            (Type::Bool, serde_json::json!(false), true)
        );
        assert_eq!(
            shape("epok.Int32(-4)"),
            (Type::Int32, serde_json::json!(-4), true)
        );
        assert_eq!(
            shape("epok.UInt32(0)"),
            (Type::UInt32, serde_json::json!(0), true)
        );
        assert_eq!(
            shape("epok.Fixed(1)"),
            (Type::Fixed, serde_json::json!(1.0), true)
        );
        assert_eq!(
            shape("epok.Vector2(1.0, 2.0)"),
            (
                Type::Vector { length: 2 },
                serde_json::json!([1.0, 2.0]),
                true
            )
        );
        assert_eq!(
            shape("epok.Vector3(1.0, 0.5, 0.0)"),
            (
                Type::Vector { length: 3 },
                serde_json::json!([1.0, 0.5, 0.0]),
                true
            )
        );
        assert_eq!(
            shape("epok.ActorRef()"),
            (Type::ActorRef { class: None }, Value::Null, true)
        );
        assert_eq!(
            shape("epok.ActorRef(EnemyBase)"),
            (
                Type::ActorRef {
                    class: Some("EnemyBase".into())
                },
                Value::Null,
                true
            )
        );
        assert_eq!(
            shape("epok.ComponentRef(epok.ActorComponent)"),
            (
                Type::ComponentRef {
                    class: Some("epok::ActorComponent".into())
                },
                Value::Null,
                true
            )
        );
        assert_eq!(
            shape("epok.ObjectRef()"),
            (Type::ObjectRef { class: None }, Value::Null, true)
        );
        assert_eq!(
            shape("epok.AssetRef(\"Mesh\")"),
            (
                Type::AssetRef {
                    kind: "Mesh".into()
                },
                Value::Null,
                true
            )
        );
        assert_eq!(
            shape("epok.ClassRef(EnemyBase)"),
            (
                Type::ClassRef {
                    base: "EnemyBase".into()
                },
                Value::Null,
                true
            )
        );
        // The Inspector wrapper changes nothing but the visibility.
        assert_eq!(
            shape("epok.Hidden(1.0)"),
            (Type::Fixed, serde_json::json!(1.0), false)
        );
        assert_eq!(
            shape("epok.Hidden(epok.UInt32(2))"),
            (Type::UInt32, serde_json::json!(2), false)
        );
        // An enum is named, not resolved: the variants are reflected, so the
        // type is completed when the declaration is published.
        assert_eq!(
            shape("epok.Enum(Mode, \"Idle\")"),
            (
                Type::Enum {
                    cpp_name: "Mode".into(),
                    variants: BTreeMap::new()
                },
                serde_json::json!("Idle"),
                true
            )
        );

        let rejected = |value: &str| {
            extract(&file(&format!(
                "local EnemyLogic = epok.ActorComponent:extend()\nEnemyLogic.member = {value}{TAIL}"
            )))
            .unwrap_err()
            .message
        };
        for (value, expected) in [
            ("1 + 1", "epok value constructor"),
            ("\"fast\"", "epok value constructor"),
            ("nil", "epok value constructor"),
            ("epok.Money(1)", "is not an epok value constructor"),
            ("epok.UInt32()", "epok.UInt32 takes 1 argument(s)"),
            ("epok.Vector3(1.0, 2.0)", "epok.Vector3 takes 3 argument(s)"),
            ("epok.Vector2(1.0, self)", "Default is not a literal value"),
            ("epok.AssetRef(Mesh)", "epok.AssetRef takes a quoted name"),
            ("epok.ActorRef(\"EnemyBase\")", "written unquoted"),
            ("epok.Hidden(epok.Hidden(1.0))", "wraps a value once"),
        ] {
            let message = rejected(value);
            assert!(message.contains(expected), "{value} gave: {message}");
        }
    }

    /// `epok.Enum` is completed against the reflected variants, or refused.
    #[test]
    fn lua_asset_resolves_enum_properties_against_the_registry() {
        // Enums are declared by the members that use them, which is how
        // `epok.Enum(Mode, "Idle")` finds its variants.
        let mut registry = registry();
        registry
            .classes
            .get_mut(ACTOR_COMPONENT_ID)
            .unwrap()
            .properties
            .push(schema::Property {
                id: "f5b3c4d6-8a0d-4c2e-9f7b-0d2e5f6c8b05".into(),
                name: "stance".into(),
                value_type: Type::Enum {
                    cpp_name: "Mode".into(),
                    variants: [("Idle".to_owned(), 0), ("Alert".to_owned(), 1)]
                        .into_iter()
                        .collect(),
                },
                default: serde_json::json!(0),
                editable: true,
                timeline: None,
                source: schema::Location {
                    file: "object_model.hpp".into(),
                    line: 1,
                    column: 1,
                },
            });
        let declared = |value: &str| {
            let source = format!(
                "local EnemyLogic = epok.ActorComponent:extend()\nEnemyLogic.mode = {value}{TAIL}"
            );
            let file = file(&source);
            declarations(&extract(&file).unwrap(), &file, &registry).map(|class| {
                let property = class.properties.iter().find(|p| p.name == "mode").unwrap();
                (property.value_type.clone(), property.default.clone())
            })
        };
        let (value_type, default) = declared("epok.Enum(Mode, \"Alert\")").unwrap();
        assert_eq!(
            value_type,
            Type::Enum {
                cpp_name: "Mode".into(),
                variants: [("Idle".to_owned(), 0), ("Alert".to_owned(), 1)]
                    .into_iter()
                    .collect()
            }
        );
        assert_eq!(default, serde_json::json!(1));
        assert!(
            declared("epok.Enum(Mood, \"Idle\")")
                .unwrap_err()
                .message
                .contains("Mood is not a reflected enum")
        );
        assert!(
            declared("epok.Enum(Mode, \"Asleep\")")
                .unwrap_err()
                .message
                .contains("Mode has no variant named Asleep")
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
        // Shadowing an inherited property name.
        check(
            HEADER.replace("EnemyLogic.health", "EnemyLogic.armour") + TAIL,
            "shadows an inherited member",
        );
        // The intrinsic transform names are reserved in every class, spatial or
        // not: a property called `rotation` would shadow `self.rotation`.
        check(
            HEADER.replace("EnemyLogic.health", "EnemyLogic.rotation") + TAIL,
            crate::lua_frontend::profile::TRANSFORM_RESERVED,
        );
        // The two intrinsic receivers are reserved for the same reason.
        for name in ["ref", "super"] {
            check(
                HEADER.replace("EnemyLogic.health", &format!("EnemyLogic.{name}")) + TAIL,
                RESERVED_MEMBERS,
            );
        }
        // A default that is not valid for the declared type.
        check(
            HEADER.replace("epok.UInt32(3)", "epok.UInt32(-1)") + TAIL,
            "no valid bounded native default",
        );
        // Overriding a final function.
        check(
            HEADER.replace("epok.ActorComponent", "EnemyBase")
                + "\n---@override\nfunction EnemyLogic:sealed()\nend"
                + TAIL,
            "not an overridable event",
        );
        // An override whose signature differs from the reflected parent.
        check(
            HEADER.replace("epok.ActorComponent", "EnemyBase")
                + "\n---@override\n---@param amount Int32\nfunction EnemyLogic:damage(amount)\nend"
                + TAIL,
            "signature differs from its reflected parent",
        );
        // A `---@id` that is not a canonical UUID.
        check(
            HEADER.replace(
                "3d352b2b-c2d7-4b99-9ba1-a003d648e897",
                "lua:EnemyLogic:health",
            ) + TAIL,
            "Property health id must be a canonical UUID",
        );
        check(
            format!("{HEADER}\n---@id not-a-uuid\nfunction EnemyLogic:reset()\nend{TAIL}"),
            "Function reset id must be a canonical UUID",
        );
    }

    #[test]
    fn lua_asset_enforces_the_property_budget() {
        let mut extra = String::new();
        for index in 0..16 {
            extra.push_str(&format!("EnemyLogic.slot{index} = 0\n"));
        }
        let file = file(&format!("{HEADER}{extra}{TAIL}"));
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
        // Discovery also answers "which class is this file?": nothing is
        // recorded yet, so both fall back to their derived identity.
        assert!(files.iter().all(|file| file.id.is_none()));
        let recorded =
            crate::lua_identity::assign(&root, &root.join("assets/scripts/A.lua")).unwrap();
        assert_eq!(
            load_all(&root).unwrap()[0].id.as_deref(),
            Some(recorded.as_str())
        );
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

        // An annotation may also use the spelling the generated Lua Language
        // Server definitions show, which writes `.` where C++ writes `::`.
        assert_eq!(
            annotation_type("ActorRef<epok.Actor3D>"),
            value_type("ActorRef<epok::Actor3D>")
        );
        assert_eq!(annotation_type("Fixed"), Some(Type::Fixed));
        assert_eq!(annotation_type("string"), None);
    }

    /// Function signatures read from the annotations the language server reads.
    #[test]
    fn lua_asset_reads_function_signatures_from_annotations() {
        let source =
            |body: &str| format!("local EnemyLogic = epok.ActorComponent:extend()\n{body}{TAIL}");
        let extracted = |body: &str| extract(&file(&source(body)));
        let named = |body: &str, name: &str| {
            extracted(body)
                .unwrap()
                .functions
                .into_iter()
                .find(|f| f.name == name)
                .unwrap()
        };

        let take = named(
            "---@param amount Fixed\n---@return Fixed\nfunction EnemyLogic:take_damage(amount)\nend",
            "take_damage",
        );
        assert_eq!(take.parameters, vec![("amount".into(), Type::Fixed)]);
        assert_eq!(take.returns, Type::Fixed);
        assert!(take.callable && take.overrides.is_none() && !take.inferred);

        // Several parameters, in the order they are written, and the dotted
        // spelling of a reflected class name.
        let aim = named(
            "---@param target ActorRef<epok.Actor3D>\n---@param force Int32\nfunction EnemyLogic:aim(target, force)\nend",
            "aim",
        );
        assert_eq!(
            aim.parameters,
            vec![
                (
                    "target".into(),
                    Type::ActorRef {
                        class: Some("epok::Actor3D".into())
                    }
                ),
                ("force".into(), Type::Int32),
            ]
        );
        assert_eq!(aim.returns, Type::Void, "no ---@return means void");

        // `---@override` is the annotation form for a reflected event that is
        // not one of the five lifecycle names.
        let hit = named("---@override\nfunction EnemyLogic:hit(amount)\nend", "hit");
        assert_eq!(hit.overrides.as_deref(), Some("hit"));
        assert!(hit.inferred && !hit.callable);

        // A bare lifecycle method needs no annotation at all.
        let begin = named("function EnemyLogic:begin_play()\nend", "begin_play");
        assert_eq!(begin.overrides.as_deref(), Some("begin_play"));
        assert!(begin.inferred);

        // A method with no parameters has nothing left to declare.
        let bare = named("function EnemyLogic:mystery()\nend", "mystery");
        assert_eq!(bare.parameters, vec![]);
        assert_eq!(bare.returns, Type::Void);
        assert!(bare.callable && bare.overrides.is_none() && !bare.inferred);

        // Count, order and vocabulary are all checked.
        for (body, expected) in [
            (
                "function EnemyLogic:mystery(amount)\nend",
                PARAMETER_ANNOTATIONS,
            ),
            (
                "---@param amount Fixed\nfunction EnemyLogic:aim(target, force)\nend",
                "takes 2 parameter(s) but declares 1",
            ),
            (
                "---@param force Int32\n---@param target Fixed\nfunction EnemyLogic:aim(target, force)\nend",
                "Parameter target is annotated as force",
            ),
            (
                "---@param amount Money\nfunction EnemyLogic:aim(amount)\nend",
                "Money is not an epok-lua type name",
            ),
            (
                "---@return Money\nfunction EnemyLogic:aim()\nend",
                "Money is not an epok-lua type name",
            ),
            (
                "---@return Fixed\n---@return Fixed\nfunction EnemyLogic:aim()\nend",
                "returns at most one value",
            ),
        ] {
            let message = extracted(body).unwrap_err().message;
            assert!(message.contains(expected), "{body}\ngave: {message}");
        }

        // Only the unbroken run directly above `function` is the signature, and
        // a commented-out annotation is an ordinary comment.
        let spaced = extracted(
            "---@param amount Fixed\n\n-- ---@param amount Fixed\nfunction EnemyLogic:aim(amount)\nend",
        );
        assert_eq!(spaced.unwrap_err().message, PARAMETER_ANNOTATIONS);
    }

    #[test]
    fn creation_template_extracts_and_keeps_the_patched_anchors() {
        let source = template("Spinner", "epok::ActorComponent");
        assert!(source.contains("Spinner.speed = 1.0"));
        assert!(source.contains("function Spinner:begin_play()\nend"));
        assert!(source.contains("---@class Spinner : epok.ActorComponent"));
        assert!(!source.contains("PSX"));
        // The engine assigns the identity: the author is never asked to type it.
        assert!(!source.contains("id ="));
        let file = LuaFile {
            path: PathBuf::from("assets/scripts/Spinner.lua"),
            source,
            id: Some("1f4d9c8e-2b3a-4c5d-8e6f-7a8b9c0d1e2f".into()),
        };
        let declaration = extract(&file).unwrap();
        assert_eq!(declaration.name, "Spinner");
        assert_eq!(declaration.extends, "epok::ActorComponent");
        assert_eq!(declaration.id, "1f4d9c8e-2b3a-4c5d-8e6f-7a8b9c0d1e2f");
        let speed = &declaration.properties[0];
        assert_eq!(speed.name, "speed");
        assert_eq!(speed.value_type, Type::Fixed);
        assert_eq!(declaration.properties.len(), 1);
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
        declarations(&declaration, &file, &registry()).unwrap();
    }

    /// The identity a project does not record is derived from the class name,
    /// deterministically, exactly as a C++ declaration without `Id=` is
    /// identified by its USR.
    #[test]
    fn lua_identities_are_derived_when_the_project_records_none() {
        let source = "\
local Drone = epok.ActorComponent:extend()
Drone.speed = 1.0
Drone.armed = false

function Drone:reset()
end

function Drone:begin_play()
end

return Drone
";
        let file = unrecorded("assets/scripts/Drone.lua", source);
        let decl = extract(&file).unwrap();
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

        // The very same file, once the project records an identity for it: the
        // class is pinned, and so is every member that has no `---@id`.
        let pinned = LuaFile {
            id: Some("6a1e8f20-4c3d-4b5e-9f70-1a2b3c4d5e6f".into()),
            ..file.clone()
        };
        let decl = extract(&pinned).unwrap();
        assert_eq!(decl.id, "6a1e8f20-4c3d-4b5e-9f70-1a2b3c4d5e6f");
        assert_eq!(
            decl.properties[0].id,
            "lua:6a1e8f20-4c3d-4b5e-9f70-1a2b3c4d5e6f:speed"
        );
        // And `---@id` still wins over both.
        let annotated = LuaFile {
            source: source.replace(
                "Drone.speed",
                "---@id 7b2f9031-5d4e-4c6f-8a81-2b3c4d5e6f70\nDrone.speed",
            ),
            ..pinned.clone()
        };
        assert_eq!(
            extract(&annotated).unwrap().properties[0].id,
            "7b2f9031-5d4e-4c6f-8a81-2b3c4d5e6f70"
        );
    }

    #[test]
    fn retired_metadata_table_and_byte_order_mark_are_reported_usefully() {
        let message = |source: &str| extract(&file(source)).unwrap_err().message;
        let retired = message(
            "local Cube = epok.class {\n    profile = 1,\n    name = \"Cube\",\n    extends = \"epok::ActorComponent\",\n    properties = {},\n    functions = {}\n}\n\nfunction Cube:begin_play()\nend\n\nreturn Cube\n",
        );
        assert!(retired.contains("retired `epok.class"), "{retired}");
        assert!(retired.contains(":extend()"), "{retired}");
        // A UTF-8 byte order mark is not content; the same source with and
        // without it extracts identically.
        let plain = "---@class Cube : epok.Actor3D\nlocal Cube = epok.Actor3D:extend()\nCube.speed = 1.0\nreturn Cube\n";
        let with_bom = format!("\u{feff}{plain}");
        let cube = |source: &str| LuaFile {
            path: PathBuf::from("assets/scripts/Cube.lua"),
            source: source.into(),
            id: None,
        };
        let a = extract(&cube(plain)).unwrap();
        let b = extract(&cube(&with_bom)).unwrap();
        assert_eq!(a.extends, b.extends);
        assert_eq!(a.properties.len(), 1);
        assert_eq!(b.properties.len(), 1);
    }
}
