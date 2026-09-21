//! Lua VM packaging: normalized Lua emission and chunk payloads (M4).
//!
//! The chunks emitted here are not the author's source. They are a normalized
//! form (contract §10.2) in which every arithmetic operation, field access and
//! dispatch is a call into a registered C function, so the numeric contract of
//! §3 is the runtime's `epok::bp::*` rather than the Lua fork's `long`
//! arithmetic. A chunk is therefore mode-independent: the same text is either
//! packaged verbatim (`VmSource`) or cooked to target bytecode (`VmBytecode`).
#![allow(dead_code)] // Consumed by lua_compile once M4 lands.
use crate::{
    lua_asset, lua_bytecode, reflection_schema as schema, script_ir,
    script_ir::{BinaryOp, Conversion, Expr, Place, Statement, StatementKind, UnaryOp},
    settings::LuaExecution,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
    path::{Path, PathBuf},
};

/// Everything the VM modes add on top of the shared generated C++ shell.
#[derive(Default)]
pub struct VmPackage {
    pub files: BTreeMap<PathBuf, Vec<u8>>,
    pub native_sources: Vec<PathBuf>,
    pub dependencies: BTreeSet<PathBuf>,
    /// Class uuid -> C++ symbol of the chunk payload declared in `files`.
    pub chunk_symbols: BTreeMap<String, String>,
}

/// Directory, relative to the staging root, that holds everything below.
pub const GENERATED_DIR: &str = "scripts/generated/lua";
/// Native source carrying every class's chunk payload.
pub const CHUNKS_SOURCE: &str = "scripts/generated/lua/lua_chunks.cpp";

/// Identity of a field slot, and the key both emitters look a `Place` up by.
/// `component` is the index inside a `Vector` and 0 for every scalar.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SlotKey {
    /// `component` is `None` for a scalar property and the index inside a
    /// `Vector` property otherwise, so a scalar never collides with a x-component.
    Property {
        member_id: String,
        component: Option<usize>,
    },
    Intrinsic {
        kind: script_ir::Intrinsic,
        component: usize,
    },
}

/// One entry of a class's dense field-slot table. `value_type` is the type
/// actually crossing the boundary, so a vector component reports `Fixed`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldSlot {
    pub key: SlotKey,
    /// Authored name: a property's, or the intrinsic's.
    pub name: String,
    pub value_type: schema::Type,
}

/// Dense field slots for a class: every reflected property in
/// `Registry::properties` order with vector properties expanded into
/// consecutive components, then the intrinsic transform components of a spatial
/// class (World3D: position.xyz, rotation.xyz, scale.xyz; World2D: position.xy,
/// the scalar rotation, scale.xy). Intrinsics come last so a property never
/// lands on an intrinsic slot.
///
/// `lua_aot::emit_bindings` and the emitter below must agree slot for slot;
/// this is the single definition both call.
pub fn field_slots(class_cpp_name: &str, registry: &crate::blueprint::Registry) -> Vec<FieldSlot> {
    let mut slots = Vec::new();
    for property in registry.properties(class_cpp_name) {
        let (name, member_id) = (property.name.clone(), property.id.clone());
        match &property.value_type {
            schema::Type::Vector { length } => {
                for component in 0..*length {
                    slots.push(FieldSlot {
                        key: SlotKey::Property {
                            member_id: member_id.clone(),
                            component: Some(component),
                        },
                        name: name.clone(),
                        value_type: schema::Type::Fixed,
                    });
                }
            }
            value_type if value_type.wire_words() == 1 => slots.push(FieldSlot {
                key: SlotKey::Property {
                    member_id,
                    component: None,
                },
                name,
                value_type: value_type.clone(),
            }),
            // Composite slots are appended only when typed IR actually reads
            // or writes the whole property. This preserves every profile-v1
            // scalar slot and retains no unused wide binding case.
            _ => {}
        }
    }
    if let Some(access) = intrinsic_access(registry, class_cpp_name) {
        for kind in script_ir::Intrinsic::ALL {
            for component in 0..intrinsic_components(access.domain, kind) {
                slots.push(FieldSlot {
                    key: SlotKey::Intrinsic { kind, component },
                    name: kind.name().into(),
                    value_type: schema::Type::Fixed,
                });
            }
        }
    }
    slots
}

pub fn field_slots_for_ir(
    ir: &script_ir::ClassIr,
    registry: &crate::blueprint::Registry,
) -> Vec<FieldSlot> {
    fn place(target: &Place, used: &mut BTreeMap<String, (String, schema::Type)>) {
        match target {
            Place::Property {
                member_id,
                name,
                value_type,
            } if value_type.wire_words() > 1 => {
                used.insert(member_id.clone(), (name.clone(), value_type.clone()));
            }
            Place::VectorComponent { base, .. } => place(base, used),
            _ => {}
        }
    }
    fn expr(value: &Expr, used: &mut BTreeMap<String, (String, schema::Type)>) {
        match value {
            Expr::Read { place: source, .. } => place(source, used),
            Expr::Unary { operand, .. }
            | Expr::Convert { operand, .. }
            | Expr::Member { base: operand, .. } => expr(operand, used),
            Expr::Binary { left, right, .. } => {
                expr(left, used);
                expr(right, used);
            }
            Expr::CallSelf { args, .. } | Expr::CallParent { args, .. } => {
                for argument in args {
                    expr(argument, used);
                }
            }
            Expr::CallBuiltin { args, .. } => {
                for argument in args.iter().filter_map(script_ir::BuiltinArg::value) {
                    expr(argument, used);
                }
            }
            Expr::CallOperation { receiver, args, .. } => {
                if let Some(receiver) = receiver {
                    expr(receiver, used);
                }
                for argument in args {
                    expr(argument, used);
                }
            }
            Expr::MakeVector { components, .. } => {
                for component in components {
                    expr(component, used);
                }
            }
            Expr::Literal { .. } => {}
        }
    }
    fn block(statements: &[Statement], used: &mut BTreeMap<String, (String, schema::Type)>) {
        for statement in statements {
            match &statement.kind {
                StatementKind::Local { value, .. } | StatementKind::Evaluate(value) => {
                    expr(value, used)
                }
                StatementKind::Assign { target, value } => {
                    place(target, used);
                    expr(value, used);
                }
                StatementKind::Return(Some(value)) => expr(value, used),
                StatementKind::Return(None) => {}
                StatementKind::If {
                    cond,
                    then,
                    otherwise,
                } => {
                    expr(cond, used);
                    block(then, used);
                    block(otherwise, used);
                }
                StatementKind::For { body, .. } => block(body, used),
            }
        }
    }
    let mut slots = field_slots(&ir.cpp_name, registry);
    let mut used = BTreeMap::new();
    for method in &ir.methods {
        block(&method.body, &mut used);
    }
    for (member_id, (name, value_type)) in used {
        let key = SlotKey::Property {
            member_id,
            component: None,
        };
        if !slots.iter().any(|slot| slot.key == key) {
            slots.push(FieldSlot {
                key,
                name,
                value_type,
            });
        }
    }
    slots
}

/// The intrinsic namespace a class exposes. A World3D or World2D class exposes
/// the root transform; a UI class exposes the rect of its `RectTransform`. Both
/// lower to the `epok::bp::api` entry points the matching Blueprint Get/Set
/// nodes call, so the two authoring surfaces address one storage.
pub fn intrinsic_access(
    registry: &crate::blueprint::Registry,
    class_cpp_name: &str,
) -> Option<crate::object_model::TransformAccess> {
    if let Some(access) = crate::object_model::transform_access(registry, class_cpp_name) {
        return Some(access);
    }
    let (family, domain, owners) = crate::object_model::resolved_shape(registry, class_cpp_name);
    let through_owner = match family {
        schema::ClassFamily::Actor if domain == schema::Domain::UI => false,
        schema::ClassFamily::Component if owners.contains(&schema::Domain::UI) => true,
        _ => return None,
    };
    Some(crate::object_model::TransformAccess {
        domain: schema::Domain::UI,
        through_owner,
    })
}

/// Scalars an intrinsic occupies in a domain: the World2D rotation is a single
/// `Fixed` angle, the rect places belong to the UI domain alone, and every
/// other intrinsic is a vector. Zero means the domain does not expose the kind.
pub fn intrinsic_components(domain: schema::Domain, kind: script_ir::Intrinsic) -> usize {
    use script_ir::Intrinsic as Kind;
    let rect = matches!(kind, Kind::RectPosition | Kind::RectSize);
    match (domain, rect) {
        (schema::Domain::UI, true) => 2,
        (schema::Domain::UI, false) | (_, true) => 0,
        (schema::Domain::World2D, false) if kind == Kind::Rotation => 1,
        (schema::Domain::World2D, false) => 2,
        _ => 3,
    }
}

/// Every builtin call site of a class, in the dense order the frontend assigned.
/// `lua_aot::emit_bindings` generates one switch case per entry and the chunk
/// emitter below dispatches on the same index, so this is the single ordering
/// both backends see.
pub fn builtin_sites(ir: &script_ir::ClassIr) -> Result<Vec<Expr>, String> {
    fn walk(block: &[Statement], out: &mut BTreeMap<u32, Expr>) {
        fn expr(value: &Expr, out: &mut BTreeMap<u32, Expr>) {
            match value {
                Expr::CallBuiltin { site, args, .. } => {
                    out.insert(*site, value.clone());
                    for arg in args.iter().filter_map(script_ir::BuiltinArg::value) {
                        expr(arg, out);
                    }
                }
                Expr::CallOperation { receiver, args, .. } => {
                    if let Some(receiver) = receiver {
                        expr(receiver, out);
                    }
                    for arg in args {
                        expr(arg, out);
                    }
                }
                Expr::Unary { operand, .. }
                | Expr::Convert { operand, .. }
                | Expr::Member { base: operand, .. } => expr(operand, out),
                Expr::Binary { left, right, .. } => {
                    expr(left, out);
                    expr(right, out);
                }
                Expr::MakeVector { components, .. } => {
                    for component in components {
                        expr(component, out);
                    }
                }
                Expr::CallSelf { args, .. } | Expr::CallParent { args, .. } => {
                    for arg in args {
                        expr(arg, out);
                    }
                }
                Expr::Literal { .. } | Expr::Read { .. } => {}
            }
        }
        for statement in block {
            match &statement.kind {
                StatementKind::Local { value, .. }
                | StatementKind::Assign { value, .. }
                | StatementKind::Evaluate(value) => expr(value, out),
                StatementKind::Return(Some(value)) => expr(value, out),
                StatementKind::Return(None) => {}
                StatementKind::If {
                    cond,
                    then,
                    otherwise,
                } => {
                    expr(cond, out);
                    walk(then, out);
                    walk(otherwise, out);
                }
                StatementKind::For { body, .. } => walk(body, out),
            }
        }
    }
    let mut sites = BTreeMap::new();
    for method in &ir.methods {
        walk(&method.body, &mut sites);
    }
    // A gap would silently shift every later case of the generated switch.
    for (index, site) in sites.keys().enumerate() {
        if index as u32 != *site {
            return Err(format!(
                "{}: builtin site {site} is not dense; the generated dispatch would misalign",
                ir.cpp_name
            ));
        }
    }
    Ok(sites.into_values().collect())
}

/// Every reflected catalog-operation site, independently dense from legacy
/// builtin sites so v1 numbering and serialized behavior stay unchanged.
pub fn operation_sites(ir: &script_ir::ClassIr) -> Result<Vec<Expr>, String> {
    fn walk_expr(value: &Expr, out: &mut BTreeMap<u32, Expr>) {
        match value {
            Expr::CallOperation {
                site,
                receiver,
                args,
                ..
            } => {
                out.insert(*site, value.clone());
                if let Some(receiver) = receiver {
                    walk_expr(receiver, out);
                }
                for argument in args {
                    walk_expr(argument, out);
                }
            }
            Expr::CallBuiltin { args, .. } => {
                for argument in args.iter().filter_map(script_ir::BuiltinArg::value) {
                    walk_expr(argument, out);
                }
            }
            Expr::CallSelf { args, .. } | Expr::CallParent { args, .. } => {
                for argument in args {
                    walk_expr(argument, out);
                }
            }
            Expr::Unary { operand, .. }
            | Expr::Convert { operand, .. }
            | Expr::Member { base: operand, .. } => walk_expr(operand, out),
            Expr::Binary { left, right, .. } => {
                walk_expr(left, out);
                walk_expr(right, out);
            }
            Expr::MakeVector { components, .. } => {
                for component in components {
                    walk_expr(component, out);
                }
            }
            Expr::Literal { .. } | Expr::Read { .. } => {}
        }
    }
    fn walk(block: &[Statement], out: &mut BTreeMap<u32, Expr>) {
        for statement in block {
            match &statement.kind {
                StatementKind::Local { value, .. }
                | StatementKind::Assign { value, .. }
                | StatementKind::Evaluate(value)
                | StatementKind::Return(Some(value)) => walk_expr(value, out),
                StatementKind::Return(None) => {}
                StatementKind::If {
                    cond,
                    then,
                    otherwise,
                } => {
                    walk_expr(cond, out);
                    walk(then, out);
                    walk(otherwise, out);
                }
                StatementKind::For { body, .. } => walk(body, out),
            }
        }
    }
    let mut sites = BTreeMap::new();
    for method in &ir.methods {
        walk(&method.body, &mut sites);
    }
    for (index, site) in sites.keys().enumerate() {
        if index as u32 != *site {
            return Err(format!(
                "{}: operation site {site} is not dense; generated dispatch would misalign",
                ir.cpp_name
            ));
        }
    }
    Ok(sites.into_values().collect())
}

/// Dense method slots for a class. The first `ClassIr::methods.len()` slots are
/// the class's Lua-bodied methods, in declaration order, because generated
/// trampolines index the chunk by that slot. Only inherited native call sites
/// actually referenced by the typed IR follow. Merely publishing a large
/// engine API must not consume the 64 Lua-body slots or retain one dispatch
/// case per inherited callable in every class.
///
/// `lua_aot::emit_bindings` and `emit_chunk` must agree slot for slot; this is
/// the single definition both call.
pub fn method_slots(
    ir: &script_ir::ClassIr,
    registry: &crate::blueprint::Registry,
) -> Vec<schema::Function> {
    let mut slots: Vec<schema::Function> = ir.methods.iter().map(|m| m.function.clone()).collect();
    let mut names: BTreeSet<String> = slots.iter().map(|f| f.name.clone()).collect();
    let mut referenced = BTreeSet::new();
    fn expression(value: &Expr, referenced: &mut BTreeSet<String>) {
        match value {
            Expr::CallSelf {
                function_id, args, ..
            }
            | Expr::CallParent {
                function_id, args, ..
            } => {
                referenced.insert(function_id.clone());
                for argument in args {
                    expression(argument, referenced);
                }
            }
            Expr::CallBuiltin { args, .. } => {
                for argument in args.iter().filter_map(script_ir::BuiltinArg::value) {
                    expression(argument, referenced);
                }
            }
            Expr::CallOperation { receiver, args, .. } => {
                if let Some(receiver) = receiver {
                    expression(receiver, referenced);
                }
                for argument in args {
                    expression(argument, referenced);
                }
            }
            Expr::Unary { operand, .. }
            | Expr::Convert { operand, .. }
            | Expr::Member { base: operand, .. } => expression(operand, referenced),
            Expr::Binary { left, right, .. } => {
                expression(left, referenced);
                expression(right, referenced);
            }
            Expr::MakeVector { components, .. } => {
                for component in components {
                    expression(component, referenced);
                }
            }
            Expr::Literal { .. } | Expr::Read { .. } => {}
        }
    }
    fn block(statements: &[Statement], referenced: &mut BTreeSet<String>) {
        for statement in statements {
            match &statement.kind {
                StatementKind::Local { value, .. }
                | StatementKind::Assign { value, .. }
                | StatementKind::Evaluate(value)
                | StatementKind::Return(Some(value)) => expression(value, referenced),
                StatementKind::Return(None) => {}
                StatementKind::If {
                    cond,
                    then,
                    otherwise,
                } => {
                    expression(cond, referenced);
                    block(then, referenced);
                    block(otherwise, referenced);
                }
                StatementKind::For { body, .. } => block(body, referenced),
            }
        }
    }
    for method in &ir.methods {
        block(&method.body, &mut referenced);
    }
    for class in registry.ancestry(&ir.cpp_name) {
        for function in &class.functions {
            let reachable = referenced.contains(&function.id)
                || function
                    .overrides
                    .iter()
                    .any(|identity| referenced.contains(identity));
            if reachable && names.insert(function.name.clone()) {
                slots.push(function.clone());
            }
        }
    }
    slots
}

/// Function id -> dense slot, including every id a slot's method overrides:
/// `Expr::CallParent` carries the PARENT's function id, and the generated
/// `super_call` switch uses the same dense slot as `self_call`.
pub fn method_index(slots: &[schema::Function]) -> BTreeMap<String, usize> {
    let mut index = BTreeMap::new();
    for (slot, function) in slots.iter().enumerate() {
        index.insert(function.id.clone(), slot);
        for inherited in &function.overrides {
            index.entry(inherited.clone()).or_insert(slot);
        }
    }
    index
}

/// C++ identifier of the chunk payload array for a class uuid.
pub fn chunk_symbol(class_id: &str) -> String {
    let hex: String = class_id
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .flat_map(char::to_lowercase)
        .collect();
    // A derived identity carries separators the filter above folds away, so
    // `lua:my_class` and `lua:myclass` would share a symbol. The hash of the
    // whole id keeps one symbol per identity whatever the spelling.
    format!(
        "epok_lua_chunk_{hex}_{:016x}",
        crate::blueprint_refs::compact_id(class_id)
    )
}

/// Lua chunk name; it is what a runtime error message quotes, so it names the
/// authored file rather than the generated artifact.
/// `lua_Number` is a signed 32-bit `long` on target, and the fork's decimal
/// scanner is `strtol`, which saturates. `-2147483648` therefore cannot be
/// written as a literal: the token `2147483648` already saturates to
/// `LONG_MAX` before the unary minus applies. Lua folds the subtraction below
/// at compile time, and does so identically on the host and on target.
fn int32_literal(value: i32) -> String {
    if value == i32::MIN {
        "(-2147483647 - 1)".into()
    } else {
        value.to_string()
    }
}

fn chunk_name(class: &schema::Class) -> String {
    format!(
        "@{}",
        class.source.file.display().to_string().replace('\\', "/")
    )
}

// ---------------------------------------------------------------------------
// Normalized Lua emission
// ---------------------------------------------------------------------------

struct Emitter<'a> {
    out: String,
    /// Local id -> emitted Lua name. Parameters are `epok_p<index>`, body
    /// locals `epok_l<id>`, matching the trampoline signatures in §10.3.
    names: BTreeMap<script_ir::LocalId, String>,
    fields: &'a BTreeMap<SlotKey, usize>,
    methods: &'a BTreeMap<String, usize>,
    class: &'a script_ir::ClassIr,
    method: &'a str,
    error: Option<(script_ir::Span, String)>,
}

impl Emitter<'_> {
    fn fail(&mut self, span: script_ir::Span, message: impl Into<String>) {
        if self.error.is_none() {
            self.error = Some((span, message.into()));
        }
    }
    fn indent(&mut self, depth: usize) {
        for _ in 0..depth {
            self.out.push_str("  ");
        }
    }

    /// Slot of a property place, resolving a vector component to its own slot.
    fn field_slot(&mut self, place: &Place, span: script_ir::Span) -> usize {
        let key = match place {
            // The frontend rejects whole-vector values in every mode, so a
            // vector Place here would be a lowering bug, never user source.
            // Narrowing it to component 0 would silently change the program.
            Place::Property {
                member_id,
                name,
                value_type: schema::Type::Vector { .. },
            } => {
                self.fail(
                    span,
                    format!(
                        "{name} is a whole vector; the VM boundary moves one component at a time"
                    ),
                );
                let _ = member_id;
                return 0;
            }
            Place::Property { member_id, .. } => SlotKey::Property {
                member_id: member_id.clone(),
                component: None,
            },
            Place::Intrinsic {
                kind, component, ..
            } => SlotKey::Intrinsic {
                kind: *kind,
                component: *component,
            },
            Place::VectorComponent { base, index } => match &**base {
                Place::Property { member_id, .. } => SlotKey::Property {
                    member_id: member_id.clone(),
                    component: Some(*index),
                },
                _ => {
                    self.fail(
                        span,
                        "Vector locals are not representable across the VM boundary in epok-lua v1; keep vector values in properties.",
                    );
                    return 0;
                }
            },
            Place::Local(_) => {
                self.fail(span, "Internal: local place has no field slot.");
                return 0;
            }
        };
        match self.fields.get(&key) {
            Some(slot) => *slot,
            None => {
                self.fail(
                    span,
                    format!(
                        "{key:?} is not a bound field of {}; the VM binding has no slot for it.",
                        self.class.cpp_name
                    ),
                );
                0
            }
        }
    }

    /// `Bool` crosses the boundary as 0/1; inside a chunk it is a Lua boolean.
    fn boundary_in(&self, text: String, value_type: &schema::Type) -> String {
        if value_type == &schema::Type::Bool {
            format!("({text}) and 1 or 0")
        } else {
            text
        }
    }
    fn boundary_out(&self, text: String, value_type: &schema::Type) -> String {
        if value_type == &schema::Type::Bool {
            format!("({text} ~= 0)")
        } else {
            text
        }
    }

    fn literal(
        &mut self,
        value: &serde_json::Value,
        value_type: &schema::Type,
        span: script_ir::Span,
    ) -> String {
        match value_type {
            schema::Type::Bool => {
                if value.as_bool().unwrap_or(false) {
                    "true".into()
                } else {
                    "false".into()
                }
            }
            // The target's lua_Number is a signed 32-bit long, so a UInt32
            // literal is emitted as its bit pattern reinterpreted as int32 —
            // the same ABI the helpers and field accessors use.
            schema::Type::UInt32 => int32_literal(value.as_u64().unwrap_or(0) as u32 as i32),
            schema::Type::Fixed => match value.as_f64() {
                Some(number) => int32_literal(script_ir::fixed_raw(number)),
                None => {
                    self.fail(span, format!("Fixed literal {value} is not a number."));
                    "0".into()
                }
            },
            schema::Type::Int32 | schema::Type::Enum { .. } => {
                int32_literal(value.as_i64().unwrap_or(0) as i32)
            }
            other => {
                self.fail(
                    span,
                    format!(
                        "Literals of type {} are not supported by the VM backends in epok-lua v1.",
                        other.label()
                    ),
                );
                "0".into()
            }
        }
    }

    /// Prefix selecting the helper family for a value type: the numeric
    /// contract is the runtime's, so the operand type picks the C function.
    fn numeric_prefix(&mut self, value_type: &schema::Type, span: script_ir::Span) -> char {
        match value_type {
            schema::Type::Int32 | schema::Type::Enum { .. } => 'i',
            schema::Type::UInt32 => 'u',
            schema::Type::Fixed => 'f',
            other => {
                self.fail(
                    span,
                    format!(
                        "Arithmetic on {} is not supported by the VM backends in epok-lua v1.",
                        other.label()
                    ),
                );
                'i'
            }
        }
    }

    fn expr(&mut self, expr: &Expr, span: script_ir::Span) -> String {
        match expr {
            Expr::Literal { value, value_type } => self.literal(value, value_type, span),
            Expr::Read { place, value_type } => match place {
                Place::Local(id) => match self.names.get(id) {
                    Some(name) => name.clone(),
                    None => {
                        self.fail(span, format!("Internal: local {} has no name.", id.0));
                        "0".into()
                    }
                },
                _ => {
                    let slot = self.field_slot(place, span);
                    let helper = if value_type.wire_words() > 1 {
                        "__epok_getwf"
                    } else {
                        "__epok_getf"
                    };
                    self.boundary_out(format!("{helper}(self, {slot})"), value_type)
                }
            },
            Expr::Unary {
                op,
                operand,
                value_type,
            } => {
                let inner = self.expr(operand, span);
                match op {
                    UnaryOp::Not => format!("(not {inner})"),
                    UnaryOp::Negate => {
                        let prefix = self.numeric_prefix(value_type, span);
                        format!("__epok_{prefix}neg({inner})")
                    }
                }
            }
            Expr::Binary {
                op,
                left,
                right,
                value_type,
            } => self.binary(*op, left, right, value_type, span),
            Expr::Convert { kind, operand } => {
                let inner = self.expr(operand, span);
                match kind {
                    Conversion::IntToFixed => format!("__epok_from_int({inner})"),
                    Conversion::FixedToInt => format!("__epok_to_int({inner})"),
                }
            }
            Expr::Member {
                base,
                index,
                value_type,
                ..
            } => {
                // The binding gives every component of a vector property a slot
                // of its own, exactly as a write to one does; the whole array
                // never crosses the boundary. A record is a single payload, so
                // its members stay an offset into the value.
                if let Expr::Read {
                    place,
                    value_type: base_type,
                } = &**base
                    && matches!(base_type, schema::Type::Vector { .. })
                    && matches!(place, Place::Property { .. })
                {
                    let component = Place::VectorComponent {
                        base: Box::new(place.clone()),
                        index: *index,
                    };
                    let slot = self.field_slot(&component, span);
                    return self.boundary_out(format!("__epok_getf(self, {slot})"), value_type);
                }
                let base = self.expr(base, span);
                self.boundary_out(
                    format!(
                        "__epok_member({base}, {index}, {})",
                        value_type.wire_words()
                    ),
                    value_type,
                )
            }
            Expr::CallSelf {
                function_id,
                name,
                args,
                returns,
            } => self.call("__epok_call", function_id, name, args, returns, span),
            Expr::CallParent {
                function_id,
                name,
                args,
                returns,
            } => self.call("__epok_super", function_id, name, args, returns, span),
            Expr::CallBuiltin {
                site,
                args,
                returns,
                ..
            } => {
                // Only the wire operands travel. A 64-bit asset or class id is
                // read by the generated binding case from the native field, so
                // it never enters Lua at all (contract section 10.1).
                let mut text = format!("__epok_builtin(self, {site}");
                for arg in args.iter().filter_map(script_ir::BuiltinArg::value) {
                    let value_type = arg.value_type().clone();
                    let rendered = self.expr(arg, span);
                    let _ = write!(text, ", {}", self.boundary_in(rendered, &value_type));
                }
                text.push(')');
                self.boundary_out(text, returns)
            }
            Expr::CallOperation {
                site,
                receiver,
                args,
                returns,
                ..
            } => {
                let mut text = format!("__epok_operation(self, {site}");
                if let Some(receiver) = receiver {
                    let rendered = self.expr(receiver, span);
                    let rendered = self.boundary_in(rendered, receiver.value_type());
                    let _ = write!(text, ", {rendered}");
                }
                for argument in args {
                    let rendered = self.expr(argument, span);
                    let rendered = if argument.value_type().wire_words() == 1 {
                        self.boundary_in(rendered, argument.value_type())
                    } else {
                        rendered
                    };
                    let _ = write!(text, ", {rendered}");
                }
                text.push(')');
                if returns.wire_words() == 1 {
                    self.boundary_out(text, returns)
                } else {
                    text
                }
            }
            Expr::MakeVector { .. } => {
                self.fail(
                    span,
                    "Vector values are not representable across the VM boundary in epok-lua v1; assign components individually.",
                );
                "0".into()
            }
        }
    }

    fn binary(
        &mut self,
        op: BinaryOp,
        left: &Expr,
        right: &Expr,
        value_type: &schema::Type,
        span: script_ir::Span,
    ) -> String {
        let operand_type = left.value_type().clone();
        let a = self.expr(left, span);
        let b = self.expr(right, span);
        if op.logical() {
            let keyword = if op == BinaryOp::And { "and" } else { "or" };
            // Both operands are Bool, so Lua's own short-circuit is exact.
            return format!("({a} {keyword} {b})");
        }
        if op.comparison() {
            return self.comparison(op, a, b, &operand_type, span);
        }
        if matches!(operand_type, schema::Type::Vector { .. }) {
            self.fail(
                span,
                "Vector arithmetic is not representable across the VM boundary in epok-lua v1.",
            );
            return "0".into();
        }
        let prefix = self.numeric_prefix(value_type, span);
        let name = match op {
            BinaryOp::Add => "add",
            BinaryOp::Sub => "sub",
            BinaryOp::Mul => "mul",
            BinaryOp::Div => "div",
            BinaryOp::Mod => "mod",
            _ => unreachable!("comparison and logical operators handled above"),
        };
        format!("__epok_{prefix}{name}({a}, {b})")
    }

    fn comparison(
        &mut self,
        op: BinaryOp,
        a: String,
        b: String,
        operand_type: &schema::Type,
        span: script_ir::Span,
    ) -> String {
        if matches!(op, BinaryOp::Eq | BinaryOp::Ne) {
            let symbol = if op == BinaryOp::Eq { "==" } else { "~=" };
            return format!("({a} {symbol} {b})");
        }
        // Ordering. Int32, Fixed and Enum are signed int32 on both sides, so
        // Lua's own comparison already matches C++. UInt32 is a bit pattern in
        // a signed number, so it goes through the runtime's unsigned helpers.
        if operand_type == &schema::Type::UInt32 {
            return match op {
                BinaryOp::Lt => format!("__epok_ult({a}, {b})"),
                BinaryOp::Le => format!("__epok_ule({a}, {b})"),
                BinaryOp::Gt => format!("__epok_ult({b}, {a})"),
                BinaryOp::Ge => format!("__epok_ule({b}, {a})"),
                _ => unreachable!("ordering operators only"),
            };
        }
        if matches!(
            operand_type,
            schema::Type::Vector { .. } | schema::Type::Bool
        ) {
            self.fail(span, "Ordering is undefined for this type.");
            return "false".into();
        }
        let symbol = match op {
            BinaryOp::Lt => "<",
            BinaryOp::Le => "<=",
            BinaryOp::Gt => ">",
            BinaryOp::Ge => ">=",
            _ => unreachable!("ordering operators only"),
        };
        format!("({a} {symbol} {b})")
    }

    fn call(
        &mut self,
        helper: &str,
        function_id: &str,
        name: &str,
        args: &[Expr],
        returns: &schema::Type,
        span: script_ir::Span,
    ) -> String {
        let wide = returns.wire_words() > 1
            || args
                .iter()
                .any(|argument| argument.value_type().wire_words() > 1);
        let helper = match (helper, wide) {
            ("__epok_call", true) => "__epok_wide_call",
            ("__epok_super", true) => "__epok_wide_super",
            _ => helper,
        };
        let text = self.call_raw(helper, function_id, name, args, span, wide);
        self.boundary_out(text, returns)
    }

    fn call_raw(
        &mut self,
        helper: &str,
        function_id: &str,
        name: &str,
        args: &[Expr],
        span: script_ir::Span,
        wide: bool,
    ) -> String {
        let slot = match self.methods.get(function_id) {
            Some(slot) => *slot,
            None => {
                self.fail(
                    span,
                    format!(
                        "{name} has no dispatch slot on {}; the VM binding cannot reach it.",
                        self.class.cpp_name
                    ),
                );
                0
            }
        };
        let mut text = format!("{helper}(self, {slot}");
        for arg in args {
            let value_type = arg.value_type().clone();
            let rendered = self.expr(arg, span);
            let rendered = if wide && value_type.wire_words() > 1 {
                rendered
            } else {
                self.boundary_in(rendered, &value_type)
            };
            let _ = write!(text, ", {rendered}");
        }
        text.push(')');
        text
    }

    fn block(&mut self, block: &[Statement], depth: usize) {
        for statement in block {
            self.statement(statement, depth);
        }
    }

    fn statement(&mut self, statement: &Statement, depth: usize) {
        let span = statement.span;
        match &statement.kind {
            // Every local is declared once at the top of the function, so a
            // declaration inside a branch stays visible to later statements
            // exactly as the IR's single numbering space intends.
            StatementKind::Local { target, value } => {
                let rendered = self.expr(value, span);
                let name = self
                    .names
                    .get(target)
                    .cloned()
                    .unwrap_or_else(|| format!("epok_l{}", target.0));
                self.indent(depth);
                let _ = writeln!(self.out, "{name} = {rendered}");
            }
            StatementKind::Assign { target, value } => {
                let value_type = value.value_type().clone();
                let rendered = self.expr(value, span);
                self.indent(depth);
                match target {
                    Place::Local(id) => {
                        let name = self
                            .names
                            .get(id)
                            .cloned()
                            .unwrap_or_else(|| format!("epok_l{}", id.0));
                        let _ = writeln!(self.out, "{name} = {rendered}");
                    }
                    _ => {
                        let slot = self.field_slot(target, span);
                        if value_type.wire_words() > 1 {
                            let _ = writeln!(self.out, "__epok_setwf(self, {slot}, {rendered})");
                        } else {
                            let value = self.boundary_in(rendered, &value_type);
                            let _ = writeln!(self.out, "__epok_setf(self, {slot}, {value})");
                        }
                    }
                }
            }
            // A call evaluated for effect is a Lua call statement, so it is
            // emitted unwrapped: the `~= 0` a Bool result would otherwise carry
            // is an expression and not a statement.
            StatementKind::Evaluate(value) => {
                let rendered = match value {
                    Expr::CallSelf {
                        function_id,
                        name,
                        args,
                        returns,
                    } => {
                        let wide = returns.wire_words() > 1
                            || args.iter().any(|arg| arg.value_type().wire_words() > 1);
                        self.call_raw(
                            if wide {
                                "__epok_wide_call"
                            } else {
                                "__epok_call"
                            },
                            function_id,
                            name,
                            args,
                            span,
                            wide,
                        )
                    }
                    Expr::CallParent {
                        function_id,
                        name,
                        args,
                        returns,
                    } => {
                        let wide = returns.wire_words() > 1
                            || args.iter().any(|arg| arg.value_type().wire_words() > 1);
                        self.call_raw(
                            if wide {
                                "__epok_wide_super"
                            } else {
                                "__epok_super"
                            },
                            function_id,
                            name,
                            args,
                            span,
                            wide,
                        )
                    }
                    Expr::CallBuiltin {
                        site,
                        args,
                        returns,
                        ..
                    } => {
                        let mut text = format!("__epok_builtin(self, {site}");
                        for arg in args.iter().filter_map(script_ir::BuiltinArg::value) {
                            let value_type = arg.value_type().clone();
                            let rendered = self.expr(arg, span);
                            let _ = write!(text, ", {}", self.boundary_in(rendered, &value_type));
                        }
                        text.push(')');
                        let _ = returns;
                        text
                    }
                    Expr::CallOperation {
                        site,
                        receiver,
                        args,
                        ..
                    } => {
                        let mut text = format!("__epok_operation(self, {site}");
                        if let Some(receiver) = receiver {
                            let rendered = self.expr(receiver, span);
                            let rendered = self.boundary_in(rendered, receiver.value_type());
                            let _ = write!(text, ", {rendered}");
                        }
                        for argument in args {
                            let rendered = self.expr(argument, span);
                            let rendered = if argument.value_type().wire_words() == 1 {
                                self.boundary_in(rendered, argument.value_type())
                            } else {
                                rendered
                            };
                            let _ = write!(text, ", {rendered}");
                        }
                        text.push(')');
                        text
                    }
                    other => self.expr(other, span),
                };
                self.indent(depth);
                let _ = writeln!(self.out, "{rendered}");
            }
            StatementKind::If {
                cond,
                then,
                otherwise,
            } => {
                let rendered = self.expr(cond, span);
                self.indent(depth);
                let _ = writeln!(self.out, "if {rendered} then");
                self.block(then, depth + 1);
                if !otherwise.is_empty() {
                    self.indent(depth);
                    self.out.push_str("else\n");
                    self.block(otherwise, depth + 1);
                }
                self.indent(depth);
                self.out.push_str("end\n");
            }
            // Lua's numeric `for` has an inclusive limit and evaluates its
            // bounds once; the IR's bounds are literals the frontend proved.
            StatementKind::For {
                var,
                start,
                limit,
                step,
                body,
            } => {
                let name = self
                    .names
                    .get(var)
                    .cloned()
                    .unwrap_or_else(|| format!("epok_l{}", var.0));
                self.indent(depth);
                let _ = writeln!(
                    self.out,
                    "for {name} = {}, {}, {} do",
                    int32_literal(*start as i32),
                    int32_literal(*limit as i32),
                    int32_literal(*step as i32)
                );
                self.block(body, depth + 1);
                self.indent(depth);
                self.out.push_str("end\n");
            }
            StatementKind::Return(value) => {
                self.indent(depth);
                match value {
                    // `Frame::ret` accepts a Lua boolean and reports 0/1, so a
                    // Bool return needs no conversion here.
                    Some(value) => {
                        let rendered = self.expr(value, span);
                        let _ = writeln!(self.out, "return {rendered}");
                    }
                    None => self.out.push_str("return\n"),
                }
            }
        }
    }
}

/// Emits one class's normalized chunk (contract §10.2).
pub fn emit_chunk(
    ir: &script_ir::ClassIr,
    fields: &[FieldSlot],
    methods: &[schema::Function],
    file: &Path,
) -> Result<String, lua_asset::Diagnostic> {
    let field_index: BTreeMap<SlotKey, usize> = fields
        .iter()
        .enumerate()
        .map(|(slot, field)| (field.key.clone(), slot))
        .collect();
    let method_index = method_index(methods);

    let mut out = String::from("local C = {}\n");
    for method in &ir.methods {
        let mut names = BTreeMap::new();
        let mut signature = String::from("self");
        for (index, parameter) in method.parameters.iter().enumerate() {
            let name = format!("epok_p{index}");
            let _ = write!(signature, ", {name}");
            names.insert(parameter.id, name);
        }
        for local in &method.locals {
            names.insert(local.id, format!("epok_l{}", local.id.0));
        }
        let mut emitter = Emitter {
            out: String::new(),
            names,
            fields: &field_index,
            methods: &method_index,
            class: ir,
            method: &method.function.name,
            error: None,
        };
        emitter.block(&method.body, 1);
        if let Some((span, message)) = emitter.error {
            return Err(lua_asset::Diagnostic {
                file: file.to_path_buf(),
                line: span.line.max(method.span.line),
                column: span.column,
                message: format!("{}::{}: {message}", ir.cpp_name, method.function.name),
            });
        }
        let _ = writeln!(out, "function C.{}({signature})", method.function.name);
        if !method.locals.is_empty() {
            let declared: Vec<String> = method
                .locals
                .iter()
                .map(|l| format!("epok_l{}", l.id.0))
                .collect();
            let _ = writeln!(out, "  local {}", declared.join(", "));
        }
        out.push_str(&emitter.out);
        out.push_str("end\n");
    }
    out.push_str("return C\n");
    Ok(out)
}

// ---------------------------------------------------------------------------
// Packaging
// ---------------------------------------------------------------------------

fn payload_source(entries: &[(String, String, Vec<u8>)], mode: LuaExecution) -> String {
    let mut out = String::from(
        "// Generated by the Epok editor (lua_vm.rs). Chunk payloads for the Lua VM runtime.\n\
         // One array per Lua class, referenced by the ClassBinding table in lua_bindings.cpp.\n",
    );
    let _ = writeln!(
        out,
        "// Packaging: {}\n",
        match mode {
            LuaExecution::VmBytecode => "cooked bytecode (pinned psxlua ABI)",
            _ => "normalized Lua source",
        }
    );
    for (_, symbol, bytes) in entries {
        let _ = writeln!(out, "extern const unsigned char {symbol}[{}];", bytes.len());
        let _ = writeln!(out, "extern const unsigned long {symbol}_size;");
        let _ = writeln!(out, "const unsigned char {symbol}[{}] = {{", bytes.len());
        for line in bytes.chunks(16) {
            out.push_str("   ");
            for byte in line {
                let _ = write!(out, " 0x{byte:02x},");
            }
            out.push('\n');
        }
        out.push_str("};\n");
        let _ = writeln!(
            out,
            "const unsigned long {symbol}_size = {};\n",
            bytes.len()
        );
    }
    out
}

pub fn package(
    _root: &Path,
    mode: LuaExecution,
    classes: &[(schema::Class, script_ir::ClassIr)],
    registry: &crate::blueprint::Registry,
) -> Result<VmPackage, Vec<lua_asset::Diagnostic>> {
    if !mode.is_vm() {
        return Err(vec![lua_asset::Diagnostic {
            file: PathBuf::new(),
            line: 0,
            column: 0,
            message: "Lua VM packaging was requested for the native execution mode.".into(),
        }]);
    }
    if mode == LuaExecution::VmBytecode && !lua_bytecode::available() {
        return Err(vec![lua_asset::Diagnostic {
            file: PathBuf::new(),
            line: 0,
            column: 0,
            message: format!(
                "Lua VM — bytecode cannot package this project: {}",
                lua_bytecode::MISSING_SOURCES
            ),
        }]);
    }
    let mut package = VmPackage::default();
    let mut diagnostics = Vec::new();
    let mut entries: Vec<(String, String, Vec<u8>)> = Vec::new();
    for (class, ir) in classes {
        let file = class.source.file.clone();
        let chunk = match emit_chunk(
            ir,
            &field_slots_for_ir(ir, registry),
            &method_slots(ir, registry),
            &file,
        ) {
            Ok(chunk) => chunk,
            Err(diagnostic) => {
                diagnostics.push(diagnostic);
                continue;
            }
        };
        let symbol = chunk_symbol(&class.id);
        let bytes = match mode {
            LuaExecution::VmBytecode => match lua_bytecode::cook(&chunk_name(class), &chunk) {
                Ok(bytes) => bytes,
                // Never fall back to source: a bytecode build that silently
                // shipped text would be rejected by the no-parser core on the
                // console instead of here.
                Err(cause) => {
                    diagnostics.push(lua_asset::Diagnostic {
                        file,
                        line: class.source.line,
                        column: class.source.column,
                        message: format!(
                            "{}: the bytecode cooker could not produce a target chunk: {cause}",
                            ir.cpp_name
                        ),
                    });
                    continue;
                }
            },
            // A source chunk is loaded by the parser build as text, so it is
            // NUL-terminated for the reader and kept human-readable on disk.
            _ => {
                let path = PathBuf::from(GENERATED_DIR)
                    .join("chunks")
                    .join(format!("{}.lua", lua_asset::artifact_stem(&class.id)));
                package.files.insert(path.clone(), chunk.clone().into());
                package.dependencies.insert(path);
                chunk.clone().into_bytes()
            }
        };
        package
            .chunk_symbols
            .insert(class.id.clone(), symbol.clone());
        entries.push((class.id.clone(), symbol, bytes));
    }
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let source = PathBuf::from(CHUNKS_SOURCE);
    package
        .files
        .insert(source.clone(), payload_source(&entries, mode).into_bytes());
    package.native_sources.push(source);
    Ok(package)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script_ir::{Local, LocalId, Span};
    use serde_json::json;

    fn span() -> Span {
        Span { line: 4, column: 3 }
    }
    fn function(name: &str, id: &str, returns: schema::Type) -> schema::Function {
        schema::Function {
            id: id.into(),
            name: name.into(),
            parameters: vec![schema::Parameter {
                name: "amount".into(),
                value_type: schema::Type::Fixed,
                direction: schema::Direction::Value,
            }],
            returns,
            callable: true,
            timeline: None,
            event: true,
            pure: false,
            resource_demands: vec![],
            abstract_method: false,
            final_method: false,
            access: "public".into(),
            overrides: vec![],
            source: schema::Location {
                file: "Guard.lua".into(),
                line: 1,
                column: 1,
            },
        }
    }
    fn health() -> Place {
        Place::Property {
            member_id: "health-id".into(),
            name: "health".into(),
            value_type: schema::Type::Fixed,
        }
    }
    /// The slot table a class with no reachable inherited functions gets.
    fn own_methods(ir: &script_ir::ClassIr) -> Vec<schema::Function> {
        ir.methods.iter().map(|m| m.function.clone()).collect()
    }

    fn slots() -> Vec<FieldSlot> {
        vec![
            FieldSlot {
                key: SlotKey::Property {
                    member_id: "health-id".into(),
                    component: None,
                },
                name: "health".into(),
                value_type: schema::Type::Fixed,
            },
            FieldSlot {
                key: SlotKey::Property {
                    member_id: "alive-id".into(),
                    component: None,
                },
                name: "alive".into(),
                value_type: schema::Type::Bool,
            },
        ]
    }

    #[test]
    fn lua_vm_dispatches_builtins_through_the_generated_binding_site() {
        let registry = crate::lua_frontend::tests::builtin_registry();
        let source = format!(
            "{}\n{}\nreturn EnemyLogic\n",
            lua_asset::tests::HEADER.trim_start(),
            crate::lua_aot::tests::BUILTIN_BODY
        );
        let (registry, ir) =
            crate::lua_aot::tests::lower("assets/scripts/EnemyLogic.lua", &source, &registry);
        let fields = field_slots(&ir.cpp_name, &registry);
        let methods = method_slots(&ir, &registry);
        let chunk = emit_chunk(
            &ir,
            &fields,
            &methods,
            Path::new("assets/scripts/EnemyLogic.lua"),
        )
        .unwrap();
        for expected in [
            // Every builtin is one dispatch on its own site; the adapter itself
            // runs natively, so the chunk carries no `epok::bp::api` knowledge.
            "epok_l1 = __epok_builtin(self, 1, __epok_builtin(self, 0))",
            "if (__epok_builtin(self, 2, epok_l1) ~= 0) then",
            "__epok_builtin(self, 4, __epok_builtin(self, 3))",
            "__epok_builtin(self, 5, epok_l1)",
            "if (__epok_builtin(self, 6, 4, 0) ~= 0) then",
        ] {
            assert!(chunk.contains(expected), "missing {expected:?} in\n{chunk}");
        }
        // The 64-bit asset id is read by the binding case, so it never appears
        // in the chunk in any form.
        assert!(!chunk.contains("skin"), "{chunk}");
    }

    #[test]
    fn lua_vm_emits_helper_calls_for_every_arithmetic_operation() {
        let body = vec![
            Statement::new(
                StatementKind::Assign {
                    target: health(),
                    value: Expr::Binary {
                        op: BinaryOp::Sub,
                        left: Box::new(Expr::Read {
                            place: health(),
                            value_type: schema::Type::Fixed,
                        }),
                        right: Box::new(Expr::Read {
                            place: Place::Local(LocalId(0)),
                            value_type: schema::Type::Fixed,
                        }),
                        value_type: schema::Type::Fixed,
                    },
                },
                span(),
            ),
            Statement::new(
                StatementKind::Assign {
                    target: Place::Property {
                        member_id: "alive-id".into(),
                        name: "alive".into(),
                        value_type: schema::Type::Bool,
                    },
                    value: Expr::Binary {
                        op: BinaryOp::Gt,
                        left: Box::new(Expr::Read {
                            place: health(),
                            value_type: schema::Type::Fixed,
                        }),
                        right: Box::new(Expr::Literal {
                            value: json!(0.0),
                            value_type: schema::Type::Fixed,
                        }),
                        value_type: schema::Type::Bool,
                    },
                },
                span(),
            ),
            Statement::new(StatementKind::Return(None), span()),
        ];
        let ir = script_ir::ClassIr {
            class_id: "3f1c0b6e-0000-4000-8000-000000000001".into(),
            cpp_name: "Guard".into(),
            parent_cpp_name: "EnemyBase".into(),
            methods: vec![script_ir::MethodIr {
                function: function("damage", "damage-id", schema::Type::Void),
                is_override: true,
                parameters: vec![Local {
                    id: LocalId(0),
                    name: "amount".into(),
                    value_type: schema::Type::Fixed,
                }],
                locals: vec![],
                body,
                span: span(),
            }],
        };
        script_ir::validate(&ir).unwrap();
        let chunk = emit_chunk(&ir, &slots(), &own_methods(&ir), Path::new("Guard.lua")).unwrap();
        assert!(chunk.starts_with("local C = {}\n"), "{chunk}");
        assert!(chunk.ends_with("return C\n"), "{chunk}");
        assert!(
            chunk.contains("function C.damage(self, epok_p0)"),
            "{chunk}"
        );
        // Every arithmetic operation is a helper call; no raw operator on a
        // user value survives into the chunk.
        assert!(
            chunk.contains("__epok_fsub(__epok_getf(self, 0), epok_p0)"),
            "{chunk}"
        );
        assert!(
            chunk.contains("__epok_setf(self, 0, __epok_fsub"),
            "{chunk}"
        );
        // Bool crosses as 0/1 and is a Lua boolean inside the chunk.
        assert!(
            chunk.contains("__epok_setf(self, 1, ((__epok_getf(self, 0) > 0)) and 1 or 0)"),
            "{chunk}"
        );
    }

    #[test]
    fn lua_vm_routes_unsigned_ordering_and_dispatch_through_helpers() {
        let call = Expr::CallSelf {
            function_id: "ping-id".into(),
            name: "ping".into(),
            args: vec![Expr::Read {
                place: Place::Local(LocalId(1)),
                value_type: schema::Type::Bool,
            }],
            returns: schema::Type::Bool,
        };
        let body = vec![
            Statement::new(
                StatementKind::Local {
                    target: LocalId(1),
                    value: Expr::Binary {
                        op: BinaryOp::Gt,
                        left: Box::new(Expr::Literal {
                            value: json!(2_147_483_648u64),
                            value_type: schema::Type::UInt32,
                        }),
                        right: Box::new(Expr::Literal {
                            value: json!(1u64),
                            value_type: schema::Type::UInt32,
                        }),
                        value_type: schema::Type::Bool,
                    },
                },
                span(),
            ),
            Statement::new(
                StatementKind::For {
                    var: LocalId(2),
                    start: 0,
                    limit: 3,
                    step: 1,
                    body: vec![Statement::new(StatementKind::Evaluate(call), span())],
                },
                span(),
            ),
        ];
        let ir = script_ir::ClassIr {
            class_id: "3f1c0b6e-0000-4000-8000-000000000002".into(),
            cpp_name: "Guard".into(),
            parent_cpp_name: "EnemyBase".into(),
            methods: vec![
                script_ir::MethodIr {
                    function: function("tick", "tick-id", schema::Type::Void),
                    is_override: true,
                    parameters: vec![Local {
                        id: LocalId(0),
                        name: "amount".into(),
                        value_type: schema::Type::Fixed,
                    }],
                    locals: vec![
                        Local {
                            id: LocalId(1),
                            name: "hot".into(),
                            value_type: schema::Type::Bool,
                        },
                        Local {
                            id: LocalId(2),
                            name: "i".into(),
                            value_type: schema::Type::Int32,
                        },
                    ],
                    body,
                    span: span(),
                },
                script_ir::MethodIr {
                    function: function("ping", "ping-id", schema::Type::Bool),
                    is_override: false,
                    parameters: vec![Local {
                        id: LocalId(0),
                        name: "amount".into(),
                        value_type: schema::Type::Fixed,
                    }],
                    locals: vec![],
                    body: vec![Statement::new(
                        StatementKind::Return(Some(Expr::Literal {
                            value: json!(true),
                            value_type: schema::Type::Bool,
                        })),
                        span(),
                    )],
                    span: span(),
                },
            ],
        };
        let chunk = emit_chunk(&ir, &slots(), &own_methods(&ir), Path::new("Guard.lua")).unwrap();
        // 0x80000000 is emitted as its int32 bit pattern and ordered with the
        // runtime's unsigned helper, so `0x80000000 > 1` is true on target.
        assert!(
            chunk.contains("__epok_ult(1, (-2147483647 - 1))"),
            "{chunk}"
        );
        assert!(chunk.contains("for epok_l2 = 0, 3, 1 do"), "{chunk}");
        // Slot 1 is `ping`, the second method; Bool arguments cross as 0/1.
        // Evaluated for effect, the call is a statement and carries no `~= 0`.
        assert!(
            chunk.contains("\n    __epok_call(self, 1, (epok_l1) and 1 or 0)\n"),
            "{chunk}"
        );
        assert!(chunk.contains("  local epok_l1, epok_l2\n"), "{chunk}");
        // A Bool return stays a Lua boolean: Frame::ret folds it to 0/1.
        assert!(chunk.contains("return true\n"), "{chunk}");
        if lua_bytecode::available() {
            lua_bytecode::cook("@Guard.lua", &chunk).unwrap_or_else(|e| panic!("{e}\n{chunk}"));
        }
    }

    #[test]
    fn lua_vm_field_slots_expand_vectors_and_report_unbound_members() {
        let vector = schema::Property {
            id: "pos-id".into(),
            name: "offset".into(),
            value_type: schema::Type::Vector { length: 3 },
            default: json!([0.0, 0.0, 0.0]),
            editable: true,
            timeline: None,
            source: schema::Location {
                file: "Guard.lua".into(),
                line: 1,
                column: 1,
            },
        };
        let mut registry = crate::blueprint::Registry::new();
        registry.classes.insert(
            "guard-id".into(),
            schema::Class {
                id: "guard-id".into(),
                provider: lua_asset::provider(),
                backend: schema::native_backend(),
                cpp_name: "Guard".into(),
                parent: None,
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
                properties: vec![vector],
                functions: vec![],
                source: schema::Location {
                    file: "Guard.lua".into(),
                    line: 1,
                    column: 1,
                },
            },
        );
        let slots = field_slots("Guard", &registry);
        assert_eq!(slots.len(), 3);
        assert_eq!(
            slots[2].key,
            SlotKey::Property {
                member_id: "pos-id".into(),
                component: Some(2)
            }
        );
        assert!(slots.iter().all(|s| s.value_type == schema::Type::Fixed));

        // A property the binding has no slot for is a named diagnostic, not a
        // chunk that would fail on the console.
        let ir = script_ir::ClassIr {
            class_id: "3f1c0b6e-0000-4000-8000-000000000003".into(),
            cpp_name: "Guard".into(),
            parent_cpp_name: "EnemyBase".into(),
            methods: vec![script_ir::MethodIr {
                function: function("tick", "tick-id", schema::Type::Void),
                is_override: true,
                parameters: vec![Local {
                    id: LocalId(0),
                    name: "amount".into(),
                    value_type: schema::Type::Fixed,
                }],
                locals: vec![],
                body: vec![Statement::new(
                    StatementKind::Assign {
                        target: health(),
                        value: Expr::Literal {
                            value: json!(1.0),
                            value_type: schema::Type::Fixed,
                        },
                    },
                    span(),
                )],
                span: span(),
            }],
        };
        let error = emit_chunk(&ir, &slots, &own_methods(&ir), Path::new("Guard.lua")).unwrap_err();
        assert!(error.message.contains("health-id"), "{}", error.message);
        assert_eq!(error.line, 4);
    }

    #[test]
    fn lua_vm_resolves_parent_calls_through_the_overridden_function_id() {
        // `Expr::CallParent` names the PARENT's reflected function, which is a
        // different id from the override's. Both must reach the override's
        // dense slot, because the generated super_call switch is indexed the
        // same way as self_call.
        let mut overriding = function("begin_play", "child-id", schema::Type::Void);
        overriding.parameters.clear();
        overriding.overrides = vec!["parent-id".into()];
        let ir = script_ir::ClassIr {
            class_id: "3f1c0b6e-0000-4000-8000-000000000004".into(),
            cpp_name: "Guard".into(),
            parent_cpp_name: "EnemyBase".into(),
            methods: vec![script_ir::MethodIr {
                function: overriding,
                is_override: true,
                parameters: vec![],
                locals: vec![],
                body: vec![Statement::new(
                    StatementKind::Evaluate(Expr::CallParent {
                        function_id: "parent-id".into(),
                        name: "begin_play".into(),
                        args: vec![],
                        returns: schema::Type::Void,
                    }),
                    span(),
                )],
                span: span(),
            }],
        };
        let chunk = emit_chunk(&ir, &slots(), &own_methods(&ir), Path::new("Guard.lua")).unwrap();
        assert!(chunk.contains("__epok_super(self, 0)"), "{chunk}");
    }

    /// The host hashes every `LIBRARIES` entry as a build input and re-runs the
    /// inspection target with -B, which rebuilds this archive. Without `ar`'s
    /// deterministic mode the identical objects would produce different bytes
    /// each time and certification would reject the build as "inputs changed
    /// during compilation".
    #[test]
    fn lua_archive_is_written_in_deterministic_mode() {
        let makefile = include_str!("../runtime/lua.mk");
        assert!(
            makefile.contains("$(AR) rcsD $@ $(LUA_OBJS)"),
            "the Lua archive recipe lost deterministic mode"
        );
    }

    #[test]
    fn lua_vm_chunk_symbols_are_stable_c_identifiers() {
        let uuid = chunk_symbol("3F1C0B6E-0000-4000-8000-000000000001");
        assert!(
            uuid.starts_with("epok_lua_chunk_3f1c0b6e000040008000000000000001_"),
            "{uuid}"
        );
        assert!(uuid.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'));
        // Derived identities that the alphanumeric filter folds together stay
        // distinct: the symbol also carries a hash of the whole id.
        assert_ne!(chunk_symbol("lua:my_class"), chunk_symbol("lua:myclass"));
    }
}
