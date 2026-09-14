//! Semantic reflection using libclang. Source text is not used to parse declarations.
use crate::reflection_schema as schema;
use clang::{Accessibility, Entity as Actor, EntityKind as K, EvaluationResult as Eval, TypeKind};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
};

fn annotations(entity: Actor<'_>, prefix: &str) -> Vec<String> {
    entity
        .get_children()
        .into_iter()
        .filter(|e| e.get_kind() == K::AnnotateAttr)
        .filter_map(|e| e.get_name())
        .filter_map(|text| text.strip_prefix(prefix).map(str::to_owned))
        .flat_map(|text| {
            text.split(',')
                .map(str::trim)
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .collect()
}

/// Parsed `EPOK_CLASS(...)` metadata. Reflection schema 8.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ClassOptions {
    pub blueprintable: bool,
    /// `Abstract` token. `is_abstract_record` sets `abstract_class` on its own.
    pub explicit_abstract: bool,
    /// `None` inherits the parent's family; only native family roots declare one.
    pub family: Option<schema::ClassFamily>,
    /// `None` inherits the parent's domain.
    pub domain: Option<schema::Domain>,
    pub placement: schema::Placement,
    /// Present when the class declares at least one component token.
    pub component: Option<schema::ComponentContract>,
    /// `TimelineRequires=` values, validated by the caller against the schema enum.
    pub timeline_requires: Vec<String>,
}

fn family_token(value: &str) -> Option<schema::ClassFamily> {
    use schema::ClassFamily as F;
    Some(match value {
        "Object" => F::Object,
        "Actor" => F::Actor,
        "Component" => F::Component,
        "World" => F::World,
        "Level" => F::Level,
        _ => return None,
    })
}
fn domain_token(value: &str) -> Option<schema::Domain> {
    use schema::Domain as D;
    Some(match value {
        "None" => D::None,
        "World3D" => D::World3D,
        "World2D" => D::World2D,
        "UI" => D::UI,
        _ => return None,
    })
}
/// `A|B` class-name lists. Empty entries are rejected so a typo is never silent.
fn name_list(token: &str, value: &str) -> Result<Vec<String>, String> {
    let names = value.split('|').map(str::trim).collect::<Vec<_>>();
    if names.iter().any(|name| name.is_empty()) {
        return Err(format!(
            "`{token}` requires a `|` separated list of class names"
        ));
    }
    Ok(names.into_iter().map(str::to_owned).collect())
}

/// Parses the `EPOK_CLASS(...)` option tokens (design.md section 3) into schema
/// values. Pure over the comma-split token list so the grammar has unit tests
/// without libclang; option values therefore may not contain a comma.
pub fn class_options(options: &[String]) -> Result<ClassOptions, String> {
    let mut parsed = ClassOptions::default();
    let mut component = schema::ComponentContract::default();
    let mut declared_component = false;
    let (mut family_seen, mut domain_seen, mut cardinality_seen) = (false, false, false);
    for option in options {
        let option = option.trim();
        if option.is_empty() {
            continue;
        }
        let (key, value) = match option.split_once('=') {
            Some((key, value)) => (key.trim(), value.trim().trim_matches('"')),
            None => (option, ""),
        };
        match key {
            // Existing schema-7 tokens.
            "Blueprintable" => parsed.blueprintable = true,
            "Id" => {}
            "TimelineRequires" => parsed.timeline_requires.push(value.into()),
            // Schema 8.
            "Abstract" => parsed.explicit_abstract = true,
            "Family" => {
                if std::mem::replace(&mut family_seen, true) {
                    return Err("`Family` is declared more than once".into());
                }
                parsed.family =
                    Some(family_token(value).ok_or_else(|| format!("Unknown Family `{value}`"))?);
            }
            "Domain" => {
                if std::mem::replace(&mut domain_seen, true) {
                    return Err("`Domain` is declared more than once".into());
                }
                parsed.domain =
                    Some(domain_token(value).ok_or_else(|| format!("Unknown Domain `{value}`"))?);
            }
            "Placeable" => parsed.placement.placeable = true,
            "Spawnable" => parsed.placement.spawnable = true,
            "SceneManaged" => parsed.placement.scene_managed = true,
            "Root" => {
                declared_component = true;
                component.can_root = true;
            }
            "Owners" => {
                declared_component = true;
                for name in name_list("Owners", value)? {
                    let domain = domain_token(&name)
                        .filter(|d| *d != schema::Domain::None)
                        .ok_or_else(|| format!("Unknown owner domain `{name}`"))?;
                    component.owners.insert(domain);
                }
            }
            "Requires" => {
                declared_component = true;
                component.requires.extend(name_list("Requires", value)?);
            }
            "Excludes" => {
                declared_component = true;
                component.excludes.extend(name_list("Excludes", value)?);
            }
            "Cardinality" => {
                declared_component = true;
                if std::mem::replace(&mut cardinality_seen, true) {
                    return Err("`Cardinality` is declared more than once".into());
                }
                component.cardinality = match value {
                    "Single" => schema::Cardinality::Single,
                    "Multiple" => schema::Cardinality::Multiple,
                    _ => return Err(format!("Unknown Cardinality `{value}`")),
                };
            }
            "Capability" => {
                declared_component = true;
                if value.is_empty() {
                    return Err("`Capability` requires a name".into());
                }
                component.capabilities.insert(value.into());
            }
            _ => return Err(format!("Unknown EPOK_CLASS option `{option}`")),
        }
    }
    if parsed.placement.scene_managed && (parsed.placement.placeable || parsed.placement.spawnable)
    {
        return Err("`SceneManaged` actors are never Placeable or Spawnable".into());
    }
    if declared_component {
        parsed.component = Some(component);
    }
    Ok(parsed)
}

/// Parsed `EPOK_COMPONENT(...)` field metadata.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DefaultComponentOptions {
    pub root: bool,
    pub name: Option<String>,
    /// Field name of the sibling default component this one attaches to.
    pub attach_to: Option<String>,
}
/// Parses `EPOK_COMPONENT(...)`. `Id=` is consumed by `id()`, not here.
pub fn component_options(options: &[String]) -> Result<DefaultComponentOptions, String> {
    let mut parsed = DefaultComponentOptions::default();
    for option in options {
        let option = option.trim();
        if option.is_empty() {
            continue;
        }
        let (key, value) = match option.split_once('=') {
            Some((key, value)) => (key.trim(), value.trim().trim_matches('"')),
            None => (option, ""),
        };
        match key {
            "Root" => parsed.root = true,
            "Id" => {}
            "Name" => {
                if value.is_empty() {
                    return Err("`Name` requires a display name".into());
                }
                parsed.name = Some(value.into());
            }
            "AttachTo" => {
                if value.is_empty() {
                    return Err("`AttachTo` requires a field name".into());
                }
                parsed.attach_to = Some(value.into());
            }
            _ => return Err(format!("Unknown EPOK_COMPONENT option `{option}`")),
        }
    }
    if parsed.root && parsed.attach_to.is_some() {
        return Err("A root default component has no attachment parent".into());
    }
    Ok(parsed)
}

fn id(entity: Actor<'_>, prefix: &str) -> Result<String, String> {
    for option in annotations(entity, prefix) {
        if let Some(value) = option.strip_prefix("Id=") {
            let value = value.trim_matches('"');
            return uuid::Uuid::parse_str(value)
                .map(|id| id.to_string())
                .map_err(|_| error(entity, "Id must be a UUID"));
        }
    }
    // An explicit Id survives a rename. An unannotated identity is stable across
    // machines and relocation; renaming such a declaration requires migration.
    let usr = entity
        .get_usr()
        .ok_or_else(|| error(entity, "Declaration has no stable Clang USR"))?;
    Ok(format!("cpp:{}", usr.0))
}

fn qualified(entity: Actor<'_>) -> String {
    entity
        .get_type()
        .map(|t| t.get_canonical_type().get_display_name())
        .unwrap_or_default()
}

fn location(entity: Actor<'_>) -> schema::Location {
    let loc = entity
        .get_location()
        .expect("declaration location")
        .get_spelling_location();
    schema::Location {
        file: loc.file.expect("declaration file").get_path(),
        line: loc.line,
        column: loc.column,
    }
}
fn error(entity: Actor<'_>, message: &str) -> String {
    let loc = location(entity);
    format!(
        "{}:{}:{}: {}: {message}",
        loc.file.display(),
        loc.line,
        loc.column,
        entity.get_display_name().unwrap_or_default()
    )
}

fn value_type(ty: clang::Type<'_>, context: Actor<'_>) -> Result<schema::Type, String> {
    value_type_at_depth(ty, context, 0)
}
fn value_type_at_depth(
    ty: clang::Type<'_>,
    context: Actor<'_>,
    depth: usize,
) -> Result<schema::Type, String> {
    if depth > 8 {
        return Err(error(context, "Struct nesting exceeds 8 levels"));
    }
    use schema::Type;
    let canonical = ty.get_canonical_type();
    let cpp = canonical
        .get_display_name()
        .trim_start_matches("const ")
        .to_owned();
    match canonical.get_kind() {
        TypeKind::Void => Ok(Type::Void),
        TypeKind::Bool => Ok(Type::Bool),
        TypeKind::Int | TypeKind::Long if canonical.get_sizeof() == Ok(4) => Ok(Type::Int32),
        TypeKind::UInt | TypeKind::ULong if canonical.get_sizeof() == Ok(4) => Ok(Type::UInt32),
        TypeKind::Record if cpp == "psyqo::FixedPoint<>" || cpp == "psyqo::FixedPoint<12>" => {
            Ok(Type::Fixed)
        }
        TypeKind::Record if ["epok::Transform", "epok::ObjectId"].contains(&cpp.as_str()) => {
            Ok(Type::Record {
                cpp_name: cpp,
                fields: vec![],
            })
        }
        TypeKind::Record if cpp == "epok::timeline::Handle" => Ok(Type::SequenceHandle),
        TypeKind::Record if cpp == "epok::effects::Handle" => Ok(Type::EffectHandle),
        TypeKind::Record => {
            let definition = canonical
                .get_declaration()
                .and_then(|e| e.get_definition())
                .ok_or_else(|| error(context, "Struct definition unavailable"))?;
            let children = definition.get_children();
            if definition.get_kind() != K::StructDecl
                || cpp.contains('<')
                || children.iter().any(|e| {
                    matches!(
                        e.get_kind(),
                        K::BaseSpecifier | K::Constructor | K::Destructor
                    ) || (e.get_kind() == K::Method && e.is_virtual_method())
                })
            {
                return Err(error(
                    context,
                    "Blueprint structs must be plain values without bases, constructors or virtual methods",
                ));
            }
            let mut fields = vec![];
            for field in children
                .into_iter()
                .filter(|e| e.get_kind() == K::FieldDecl)
            {
                let ty = field.get_type().unwrap();
                if field.get_accessibility() != Some(Accessibility::Public)
                    || field.is_bit_field()
                    || ty.is_const_qualified()
                    || ty.is_volatile_qualified()
                {
                    return Err(error(
                        field,
                        "Split struct fields must be public mutable values, without bitfields",
                    ));
                }
                let name = field
                    .get_name()
                    .ok_or_else(|| error(field, "Anonymous struct fields are unsupported"))?;
                fields.push(schema::RecordField {
                    name,
                    value_type: value_type_at_depth(ty, field, depth + 1)?,
                });
            }
            if fields.is_empty() || fields.len() > 32 {
                return Err(error(
                    context,
                    "Blueprint structs require between 1 and 32 supported fields",
                ));
            }
            Ok(Type::Record {
                cpp_name: cpp,
                fields,
            })
        }
        TypeKind::Enum => {
            if !canonical.get_sizeof().is_ok_and(|size| size <= 4) {
                return Err(error(context, "Reflected enum storage must fit 32 bits"));
            }
            let definition = canonical
                .get_declaration()
                .ok_or_else(|| error(context, "Enum definition unavailable"))?;
            let variants = definition
                .get_children()
                .into_iter()
                .filter(|e| e.get_kind() == K::EnumConstantDecl)
                .map(|e| {
                    (
                        e.get_name().unwrap(),
                        e.get_enum_constant_value().unwrap().0,
                    )
                })
                .collect();
            Ok(Type::Enum {
                cpp_name: cpp,
                variants,
            })
        }
        TypeKind::ConstantArray if matches!(canonical.get_size(), Some(2 | 3)) => {
            if value_type_at_depth(canonical.get_element_type().unwrap(), context, depth + 1)?
                != Type::Fixed
            {
                return Err(error(
                    context,
                    "Only Fixed[2] and Fixed[3] vector properties are supported",
                ));
            }
            Ok(Type::Vector {
                length: canonical.get_size().unwrap(),
            })
        }
        _ => Err(error(
            context,
            &format!(
                "Unsupported reflected type `{cpp}` ({:?}); raw pointers, dynamic containers, and open templates have no invocation/lifetime policy",
                canonical.get_kind()
            ),
        )),
    }
}

fn parameter(entity: Actor<'_>, index: usize) -> Result<schema::Parameter, String> {
    let ty = entity.get_type().unwrap();
    let (value, direction) = if ty.get_kind() == TypeKind::LValueReference {
        let pointee = ty.get_pointee_type().unwrap();
        (
            pointee,
            if pointee.is_const_qualified() {
                schema::Direction::ConstReference
            } else {
                schema::Direction::MutableReference
            },
        )
    } else {
        (ty, schema::Direction::Value)
    };
    Ok(schema::Parameter {
        name: entity.get_name().unwrap_or_else(|| format!("arg{index}")),
        value_type: value_type(value, entity)?,
        direction,
    })
}

fn numeric(eval: Eval) -> Option<Value> {
    match eval {
        Eval::SignedInteger(v) => Some(json!(v)),
        Eval::UnsignedInteger(v) => Some(json!(v)),
        Eval::Float(v) if v.is_finite() => Some(json!(v)),
        _ => None,
    }
}

fn fixed_default(entity: Actor<'_>) -> Result<Value, String> {
    match entity.get_kind() {
        K::UnexposedExpr | K::FunctionalCastExpr | K::ParenExpr => {
            let children = entity
                .get_children()
                .into_iter()
                .filter(|e| e.is_expression())
                .collect::<Vec<_>>();
            if children.len() == 1 {
                return fixed_default(children[0]);
            }
        }
        K::CallExpr => {
            if entity.get_reference().is_some_and(|e| {
                e.get_kind() == K::Constructor
                    && qualified(e.get_semantic_parent().unwrap()).starts_with("psyqo::FixedPoint<")
            }) {
                let args = entity.get_arguments().unwrap_or_default();
                if args.len() == 1 {
                    // PsyQo's floating constructor quantizes units; its integer
                    // constructor has different semantics, so it is not guessed.
                    if let Some(Eval::Float(value)) = args[0].evaluate()
                        && value.is_finite()
                        && (-524288.0..524288.0).contains(&value)
                    {
                        return Ok(json!(value));
                    }
                }
            }
        }
        // C++ list initialization exposes a literal directly in the field AST.
        K::FloatingLiteral | K::UnaryOperator
            if matches!(entity.evaluate(), Some(Eval::Float(_))) =>
        {
            let value = numeric(entity.evaluate().unwrap()).unwrap();
            if value
                .as_f64()
                .is_some_and(|v| (-524288.0..524288.0).contains(&v))
            {
                return Ok(value);
            }
        }
        _ => {}
    }
    Err(error(
        entity,
        "Fixed defaults require a constant floating-point literal/constructor within Q12 range; arbitrary constructor/function execution is unsupported",
    ))
}

fn property(entity: Actor<'_>) -> Result<schema::Property, String> {
    let ty = entity.get_type().unwrap();
    if entity.get_accessibility() != Some(Accessibility::Public)
        || entity.is_bit_field()
        || ty.is_const_qualified()
        || ty.is_volatile_qualified()
    {
        return Err(error(
            entity,
            "Reflected properties must be public, non-const, non-volatile instance fields (no bitfields)",
        ));
    }
    let value_type = match value_type(ty, entity)? {
        schema::Type::Record { cpp_name, .. } if cpp_name == "epok::ObjectId" => {
            schema::Type::ObjectRef { class: None }
        }
        other => other,
    };
    let expressions = entity
        .get_children()
        .into_iter()
        .filter(|e| e.is_expression())
        .collect::<Vec<_>>();
    let default = match &value_type {
        schema::Type::ObjectRef { .. } => {
            fn empty(entity: Actor<'_>) -> bool {
                match entity.get_kind() {
                    K::CallExpr => {
                        entity
                            .get_arguments()
                            .is_some_and(|arguments| arguments.is_empty())
                            && entity
                                .get_reference()
                                .is_some_and(|reference| reference.get_kind() == K::Constructor)
                    }
                    K::InitListExpr => entity
                        .get_children()
                        .iter()
                        .all(|child| !child.is_expression()),
                    K::UnexposedExpr | K::FunctionalCastExpr | K::ParenExpr => {
                        let children = entity
                            .get_children()
                            .into_iter()
                            .filter(|child| child.is_expression())
                            .collect::<Vec<_>>();
                        children.len() == 1 && empty(children[0])
                    }
                    _ => false,
                }
            }
            if !expressions.iter().copied().all(empty) {
                return Err(error(
                    entity,
                    "ObjectId properties require the empty/default handle; assign persistent scene references in the Inspector",
                ));
            }
            Value::Null
        }
        schema::Type::Fixed => fixed_default(*expressions.last().ok_or_else(|| {
            error(
                entity,
                "Reflected property requires a declarative initializer",
            )
        })?)?,
        schema::Type::Vector { length } => {
            let init = expressions
                .last()
                .filter(|e| e.get_kind() == K::InitListExpr)
                .ok_or_else(|| error(entity, "Vector requires a complete initializer list"))?;
            let values = init
                .get_children()
                .into_iter()
                .map(fixed_default)
                .collect::<Result<Vec<_>, _>>()?;
            if values.len() != *length {
                return Err(error(entity, "Vector requires every component default"));
            }
            json!(values)
        }
        schema::Type::Bool => match entity.evaluate() {
            Some(Eval::UnsignedInteger(v)) => json!(v != 0),
            Some(Eval::SignedInteger(v)) => json!(v != 0),
            _ => return Err(error(entity, "Bool requires a constant initializer")),
        },
        schema::Type::Int32 | schema::Type::UInt32 | schema::Type::Enum { .. } => entity
            .evaluate()
            .and_then(numeric)
            .ok_or_else(|| error(entity, "Property requires a constant initializer"))?,
        _ => {
            return Err(error(
                entity,
                "This runtime record has no editable authoring value; use a typed ObjectId reference or a supported scalar/vector property",
            ));
        }
    };
    let options = annotations(entity, "EPOK_PROPERTY:");
    let timeline = if options.iter().any(|v| v == "TimelineAnimatable") {
        if schema::TimelineProperty::for_type(&value_type).is_none()
            || options.iter().any(|v| v == "ReadOnly")
        {
            return Err(error(
                entity,
                "TimelineAnimatable requires a writable supported scalar, enum or Fixed vector field",
            ));
        }
        schema::TimelineProperty::for_type(&value_type)
    } else {
        None
    };
    Ok(schema::Property {
        id: id(entity, "EPOK_PROPERTY:")?,
        name: entity.get_name().unwrap(),
        value_type,
        default,
        editable: !options.iter().any(|v| v == "ReadOnly"),
        timeline,
        source: location(entity),
    })
}

fn function(entity: Actor<'_>) -> Result<schema::Function, String> {
    let options = annotations(entity, "EPOK_FUNCTION:");
    let access = entity.get_accessibility();
    let timeline = if options.iter().any(|v| v == "TimelineCallable") {
        Some(schema::TimelineCall::CrossingEvent)
    } else if options.iter().any(|v| v == "TimelineAction") {
        Some(schema::TimelineCall::IdempotentAction)
    } else {
        None
    };
    if timeline.is_some()
        && (access != Some(Accessibility::Public)
            || value_type(entity.get_result_type().unwrap(), entity)? != schema::Type::Void)
    {
        return Err(error(
            entity,
            "Timeline calls require a public void instance function",
        ));
    }
    if entity.is_variadic() || entity.is_static_method() || access == Some(Accessibility::Private) {
        return Err(error(
            entity,
            "Reflected functions must be public/protected, non-static, and non-variadic",
        ));
    }
    let event = options
        .iter()
        .any(|v| v == "BlueprintEvent" || v == "Event");
    if event && !entity.is_virtual_method() {
        return Err(error(
            entity,
            "BlueprintEvent requires a virtual C++ method",
        ));
    }
    let pure = options.iter().any(|v| v == "Pure" || v == "BlueprintPure");
    if pure && !entity.is_const_method() {
        return Err(error(entity, "Pure functions must be const"));
    }
    Ok(schema::Function {
        id: id(entity, "EPOK_FUNCTION:")?,
        name: entity.get_name().unwrap(),
        parameters: entity
            .get_arguments()
            .unwrap_or_default()
            .into_iter()
            .enumerate()
            .map(|(i, e)| parameter(e, i))
            .collect::<Result<_, _>>()?,
        returns: value_type(entity.get_result_type().unwrap(), entity)?,
        callable: access == Some(Accessibility::Public)
            && options.iter().any(|v| {
                ["Callable", "BlueprintCallable", "Pure", "BlueprintPure"].contains(&v.as_str())
            }),
        timeline,
        event,
        pure,
        abstract_method: entity.is_pure_virtual_method(),
        final_method: entity
            .get_children()
            .iter()
            .any(|e| e.get_kind() == K::FinalAttr),
        access: if access == Some(Accessibility::Protected) {
            "protected"
        } else {
            "public"
        }
        .into(),
        overrides: entity
            .get_overridden_methods()
            .unwrap_or_default()
            .into_iter()
            .map(|e| id(e, "EPOK_FUNCTION:"))
            .collect::<Result<_, _>>()?,
        source: location(entity),
    })
}

fn class(entity: Actor<'_>) -> Result<schema::Class, String> {
    // An override inherits reflection only from an explicitly reflected method.
    // Runtime-owned hooks (continuation/component synchronization) are not an
    // authoring API merely because they are virtual.
    fn reflected_method(entity: Actor<'_>) -> bool {
        !annotations(entity, "EPOK_FUNCTION:").is_empty()
            || entity
                .get_overridden_methods()
                .unwrap_or_default()
                .into_iter()
                .any(reflected_method)
    }
    let children = entity.get_children();
    let bases = children
        .iter()
        .filter(|e| e.get_kind() == K::BaseSpecifier)
        .collect::<Vec<_>>();
    if bases.len() > 1
        || bases
            .iter()
            .any(|e| e.is_virtual_base() || e.get_accessibility() != Some(Accessibility::Public))
    {
        return Err(error(
            entity,
            "Reflected classes require at most one public, non-virtual base",
        ));
    }
    let parent = bases
        .first()
        .map(|base| {
            let definition = base
                .get_type()
                .and_then(|t| t.get_declaration())
                .ok_or_else(|| error(entity, "Parent declaration unavailable"))?;
            if annotations(definition, "EPOK_CLASS:").is_empty() {
                return Err(error(entity, "Parent must have EPOK_CLASS metadata"));
            }
            id(definition, "EPOK_CLASS:")
        })
        .transpose()?;
    let mut properties = Vec::new();
    let mut functions = Vec::new();
    let mut default_components = Vec::new();
    for child in children {
        if !annotations(child, "EPOK_CLASS:").is_empty() {
            return Err(error(
                child,
                "Nested reflected classes are unsupported; declare the class at namespace scope.",
            ));
        }
        if child.get_kind() == K::Constructor && !child.is_defaulted() {
            return Err(error(
                child,
                "Reflected classes currently require defaulted/implicit constructors and declarative field defaults; custom constructor effects cannot be inspected",
            ));
        }
        if !annotations(child, "EPOK_COMPONENT:").is_empty() {
            if child.get_kind() != K::FieldDecl {
                return Err(error(
                    child,
                    "EPOK_COMPONENT must annotate an instance field",
                ));
            }
            if !annotations(child, "EPOK_PROPERTY:").is_empty() {
                return Err(error(
                    child,
                    "A field is either a default component or a reflected property, not both",
                ));
            }
            let options = component_options(&annotations(child, "EPOK_COMPONENT:"))
                .map_err(|message| error(child, &message))?;
            // The field type must itself be a reflected class; the family check
            // (Component) belongs to the host model, which sees the whole graph.
            let declaration = child
                .get_type()
                .map(|t| t.get_canonical_type())
                .and_then(|t| t.get_declaration())
                .filter(|d| !annotations(*d, "EPOK_CLASS:").is_empty())
                .ok_or_else(|| {
                    error(
                        child,
                        "EPOK_COMPONENT requires a by-value field of a reflected component class",
                    )
                })?;
            default_components.push(schema::DefaultComponent {
                id: id(child, "EPOK_COMPONENT:")?,
                field: child
                    .get_name()
                    .ok_or_else(|| error(child, "Anonymous default component field"))?,
                class: qualified(declaration),
                root: options.root,
                attach_to: options.attach_to,
                name: options.name,
            });
            continue;
        }
        if !annotations(child, "EPOK_PROPERTY:").is_empty() {
            if child.get_kind() != K::FieldDecl {
                return Err(error(
                    child,
                    "EPOK_PROPERTY must annotate an instance field",
                ));
            }
            properties.push(property(child)?);
        }
        if !annotations(child, "EPOK_FUNCTION:").is_empty() {
            if child.get_kind() != K::Method {
                return Err(error(
                    child,
                    "EPOK_FUNCTION must annotate a non-template method",
                ));
            }
            functions.push(function(child)?);
        } else if child.get_kind() == K::Method && reflected_method(child) {
            functions.push(function(child)?);
        } else if child.get_kind() == K::Method && child.is_pure_virtual_method() {
            return Err(error(
                child,
                "A pure virtual method in a reflected class must have EPOK_FUNCTION(Event) so Blueprint descendants can implement its contract",
            ));
        }
    }
    let options =
        class_options(&annotations(entity, "EPOK_CLASS:")).map_err(|m| error(entity, &m))?;
    if options.timeline_requires.len() > 1 {
        return Err(error(
            entity,
            "A class may declare only one TimelineRequires component",
        ));
    }
    let timeline_component = options
        .timeline_requires
        .first()
        .map(|value| {
            serde_json::from_value::<schema::TimelineComponentRequirement>(json!(value))
                .map_err(|_| error(entity, "Unknown TimelineRequires component"))
        })
        .transpose()?;
    Ok(schema::Class {
        family: options.family,
        domain: options.domain,
        placement: options.placement,
        component: options.component,
        default_components,
        explicit_abstract: options.explicit_abstract,
        timeline_component,
        provider: schema::native_provider(),
        backend: schema::native_backend(),
        id: id(entity, "EPOK_CLASS:")?,
        cpp_name: qualified(entity),
        parent,
        abstract_class: entity.is_abstract_record(),
        final_class: entity
            .get_children()
            .iter()
            .any(|e| e.get_kind() == K::FinalAttr),
        blueprintable: options.blueprintable,
        properties,
        functions,
        source: location(entity),
    })
}

pub fn extract(
    unit: &clang::TranslationUnit<'_>,
    source: &std::path::Path,
) -> Result<schema::Manifest, String> {
    fn visit(
        entity: Actor<'_>,
        classes: &mut Vec<schema::Class>,
        dependencies: &mut BTreeSet<std::path::PathBuf>,
    ) -> Result<(), String> {
        if entity.get_kind() == K::InclusionDirective
            && let Some(file) = entity.get_file()
        {
            dependencies.insert(file.get_path());
        }
        if !annotations(entity, "EPOK_CLASS:").is_empty() {
            if entity.get_semantic_parent().is_some_and(|p| {
                matches!(
                    p.get_kind(),
                    K::ClassDecl | K::StructDecl | K::ClassTemplate
                )
            }) {
                return Err(error(
                    entity,
                    "Nested reflected classes are unsupported; declare the class at namespace scope.",
                ));
            }
            if !matches!(entity.get_kind(), K::ClassDecl | K::StructDecl) {
                return Err(error(
                    entity,
                    "Reflected templates and nested declarations are unsupported",
                ));
            }
            if entity.is_definition() {
                classes.push(class(entity)?);
            }
        }
        if matches!(
            entity.get_kind(),
            K::TranslationUnit | K::Namespace | K::ClassDecl | K::StructDecl | K::ClassTemplate
        ) {
            for child in entity.get_children() {
                visit(child, classes, dependencies)?;
            }
        }
        Ok(())
    }
    let mut classes = Vec::new();
    let mut dependencies = BTreeSet::new();
    dependencies.insert(source.into());
    for entity in unit.get_entity().get_children() {
        visit(entity, &mut classes, &mut dependencies)?;
    }
    let mut ids = BTreeSet::new();
    for class in &classes {
        for id in std::iter::once(&class.id)
            .chain(class.properties.iter().map(|p| &p.id))
            .chain(class.functions.iter().map(|f| &f.id))
        {
            if !ids.insert(id) {
                return Err(format!("Duplicate reflected identity {id}"));
            }
        }
    }
    let dependencies = dependencies
        .into_iter()
        .map(|path| {
            let bytes = fs::read(&path)
                .map_err(|e| format!("Reflection dependency {}: {e}", path.display()))?;
            Ok((path, format!("{:x}", Sha256::digest(bytes))))
        })
        .collect::<Result<BTreeMap<_, _>, String>>()?;
    Ok(schema::Manifest {
        schema_version: schema::SCHEMA_VERSION,
        clang_version: schema::CLANG_VERSION.into(),
        target: "mipsel-none-elf/mips1/o32/little-endian".into(),
        classes,
        dependencies,
    })
}

#[cfg(test)]
mod tests {
    use super::{ClassOptions, class_options, component_options};
    use crate::reflection_schema as schema;

    fn tokens(text: &str) -> Vec<String> {
        text.split(',').map(str::trim).map(str::to_owned).collect()
    }
    fn parse(text: &str) -> ClassOptions {
        class_options(&tokens(text)).expect(text)
    }

    #[test]
    fn class_options_parse_the_documented_grammar() {
        let actor = parse("Blueprintable, Placeable, Spawnable, Domain=World3D");
        assert!(actor.blueprintable && actor.placement.placeable && actor.placement.spawnable);
        assert!(!actor.placement.scene_managed);
        assert_eq!(actor.domain, Some(schema::Domain::World3D));
        assert_eq!(actor.family, None);
        assert!(actor.component.is_none());

        let base = parse("Abstract, Family=Object, Id=\"26a54c0d-ca81-41ca-aecc-d0346a6357d2\"");
        assert!(base.explicit_abstract);
        assert_eq!(base.family, Some(schema::ClassFamily::Object));

        let audio = parse(
            "Blueprintable, Domain=None, Owners=World3D|World2D|UI, Cardinality=Multiple, Capability=audio",
        );
        let contract = audio.component.expect("component contract");
        assert_eq!(
            contract.owners,
            [
                schema::Domain::World3D,
                schema::Domain::World2D,
                schema::Domain::UI
            ]
            .into_iter()
            .collect()
        );
        assert_eq!(contract.cardinality, schema::Cardinality::Multiple);
        assert!(contract.capabilities.contains("audio"));
        assert!(!contract.can_root);

        let rooted = parse("Root, Owners=UI, Requires=epok::A|epok::B, Excludes=epok::C");
        let contract = rooted.component.expect("component contract");
        assert!(contract.can_root);
        assert_eq!(contract.requires, ["epok::A", "epok::B"]);
        assert_eq!(contract.excludes, ["epok::C"]);
        assert_eq!(contract.cardinality, schema::Cardinality::Single);

        // Schema-7 tokens keep their meaning and empty option lists are legal.
        assert_eq!(
            parse("TimelineRequires=Camera").timeline_requires,
            ["Camera"]
        );
        assert_eq!(class_options(&tokens("")).unwrap(), ClassOptions::default());
    }

    #[test]
    fn class_options_reject_unknown_or_contradictory_tokens() {
        let message = class_options(&tokens("Blueprintable, Plazeable")).unwrap_err();
        assert!(message.contains("Plazeable"), "{message}");
        assert!(class_options(&tokens("Family=Widget")).is_err());
        assert!(class_options(&tokens("Domain=World4D")).is_err());
        assert!(class_options(&tokens("Cardinality=Many")).is_err());
        assert!(class_options(&tokens("Owners=World3D|None")).is_err());
        assert!(class_options(&tokens("Owners=World3D|")).is_err());
        assert!(class_options(&tokens("Capability=")).is_err());
        assert!(class_options(&tokens("Family=Actor, Family=Component")).is_err());
        assert!(class_options(&tokens("Domain=UI, Domain=World3D")).is_err());
        assert!(class_options(&tokens("SceneManaged, Placeable")).is_err());
        assert!(class_options(&tokens("SceneManaged, Spawnable")).is_err());
    }

    #[test]
    fn the_runtime_object_model_header_parses_with_this_grammar() {
        // The Clang path cannot run on a host without the MIPS include paths, so
        // the annotation text of the native bases is checked directly.
        let header = include_str!("../runtime/object_model.hpp");
        let mut declarations = 0;
        for (index, _) in header.match_indices("class EPOK_CLASS(") {
            let rest = &header[index + "class EPOK_CLASS(".len()..];
            // The header has no nested parentheses inside EPOK_CLASS.
            let stop = rest.find(')').expect("closing parenthesis");
            let name = rest[stop + 1..]
                .trim_start()
                .split([' ', ':', '{'])
                .next()
                .expect("class name");
            let options = class_options(&tokens(&rest[..stop]))
                .unwrap_or_else(|message| panic!("{name}: {message}"));
            assert!(
                options.timeline_requires.is_empty(),
                "{name} declares TimelineRequires"
            );
            declarations += 1;
        }
        assert_eq!(declarations, 28, "native base count changed");
    }

    #[test]
    fn component_field_options_parse_root_name_and_attachment() {
        let root = component_options(&tokens("Root, Name=\"Body\"")).unwrap();
        assert!(root.root);
        assert_eq!(root.name.as_deref(), Some("Body"));
        assert_eq!(root.attach_to, None);

        let attached = component_options(&tokens("Name=\"Muzzle\", AttachTo=body")).unwrap();
        assert!(!attached.root);
        assert_eq!(attached.attach_to.as_deref(), Some("body"));

        // Id is consumed by the shared identity helper so an explicit Id survives.
        assert_eq!(
            component_options(&tokens("Id=\"29ad1a8f-9a3c-4d05-9e4a-9a2f0f1c0f3e\"")).unwrap(),
            super::DefaultComponentOptions::default()
        );
        assert!(component_options(&tokens("Rooot")).is_err());
        assert!(component_options(&tokens("Name=")).is_err());
        assert!(component_options(&tokens("AttachTo=")).is_err());
        assert!(component_options(&tokens("Root, AttachTo=body")).is_err());
    }
}
