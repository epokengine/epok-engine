//! IR -> generated C++ subclass. One shell for every execution mode: the class
//! layout, identity, defaults and virtual signatures are identical; only the
//! method bodies differ (lowered statements vs. VM trampolines), so a mode
//! change can never move a field or change a reflected contract.
use crate::{
    blueprint::Registry,
    blueprint_ir as bir,
    lua_vm::SlotKey,
    object_model::TransformAccess,
    reflection_schema::{self as schema, Domain, Type},
    script_ir::{
        BinaryOp, Block, BuiltinArg, ClassIr, Conversion, Expr, Intrinsic, LocalId, MethodIr,
        Place, Statement, StatementKind, UnaryOp,
    },
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BodyMode {
    Native,
    /// Dense per-build class index, shared with the emitted `ClassBinding` row.
    Vm {
        class_index: u32,
    },
}

/// Project-relative spelling of an authored path, for `#line` maps and the
/// generated banner. Generated code must never embed a host directory.
pub fn project_relative(path: &std::path::Path) -> String {
    let text = path.to_string_lossy().replace('\\', "/");
    match text.find("assets/") {
        Some(at) => text[at..].to_owned(),
        None => text,
    }
}

/// Parent header, resolved exactly like `blueprint_compile`: a generated parent
/// is reachable by its own artifact path, a runtime base through the umbrella
/// header, and a user C++ base relative to `scripts/generated/lua`.
fn parent_include(parent: &schema::Class) -> Result<String, String> {
    let include = match parent.provider.id.as_str() {
        "blueprint" => format!("../{}.hpp", parent.id),
        "lua" => format!("{}.hpp", crate::lua_asset::artifact_stem(&parent.id)),
        _ if parent.cpp_name.starts_with("epok::") => "epok.hpp".into(),
        _ => {
            let relative = project_relative(&parent.source.file);
            let relative = relative
                .strip_prefix("assets/scripts/")
                .ok_or("Native parent header must be within assets/scripts")?;
            format!("../../{relative}")
        }
    };
    if include.contains('"') || include.contains('\n') {
        return Err("Unsafe parent include path".into());
    }
    Ok(include)
}

fn cpp_storage(name: &str, ty: &Type, default: &serde_json::Value) -> Result<String, String> {
    Ok(match ty {
        Type::Vector { length } => format!("epok::Fixed {name}[{length}]{{}};\n"),
        _ => format!(
            "{} {name} = {};\n",
            bir::cpp_type(ty)?,
            bir::literal(default, ty)?
        ),
    })
}

fn parameter_list(function: &schema::Function) -> Result<String, String> {
    function
        .parameters
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let ty = bir::cpp_type(&p.value_type)?;
            Ok(match p.direction {
                schema::Direction::Value => format!("{ty} epok_p{i}"),
                schema::Direction::ConstReference => format!("const {ty}& epok_p{i}"),
                _ => format!("{ty}& epok_p{i}"),
            })
        })
        .collect::<Result<Vec<_>, String>>()
        .map(|v| v.join(","))
}

fn default_return(returns: &Type) -> &'static str {
    if *returns == Type::Void { "" } else { " {}" }
}

// ------------------------------------------------------------ ABI packing ----

/// `int32_t` wire form of a typed C++ value (contract section 10.1).
pub fn pack(expression: &str, ty: &Type) -> Result<String, String> {
    Ok(match ty {
        Type::Bool => format!("(({expression})?1:0)"),
        Type::Int32 => format!("int32_t({expression})"),
        Type::UInt32 | Type::Enum { .. } => format!("int32_t(uint32_t({expression}))"),
        Type::Fixed => format!("({expression}).raw()"),
        Type::ObjectRef { .. } | Type::ActorRef { .. } | Type::ComponentRef { .. } => format!(
            "int32_t((uint32_t(({expression}).generation)<<16)|uint32_t(({expression}).index))"
        ),
        _ => {
            return Err(format!(
                "{} values do not cross the Lua boundary in profile 1",
                ty.label()
            ));
        }
    })
}
/// Typed C++ value from its `int32_t` wire form.
pub fn unpack(expression: &str, ty: &Type) -> Result<String, String> {
    Ok(match ty {
        Type::Bool => format!("(({expression})!=0)"),
        Type::Int32 => format!("int32_t({expression})"),
        Type::UInt32 => format!("uint32_t({expression})"),
        Type::Fixed => format!("epok::Fixed({expression},epok::Fixed::RAW)"),
        Type::Enum { cpp_name, .. } => {
            bir::cpp_type(ty)?;
            format!("static_cast<{cpp_name}>({expression})")
        }
        Type::ObjectRef { .. } | Type::ActorRef { .. } | Type::ComponentRef { .. } => format!(
            "epok::ObjectId{{uint16_t(uint32_t({expression})&0xffffu),uint16_t(uint32_t({expression})>>16)}}"
        ),
        _ => {
            return Err(format!(
                "{} values do not cross the Lua boundary in profile 1",
                ty.label()
            ));
        }
    })
}

fn wire_pack(target: &str, expression: &str, ty: &Type, offset: usize) -> Result<String, String> {
    Ok(match ty {
        Type::Void => String::new(),
        Type::Record { .. } => {
            let mut text = String::new();
            let mut at = offset;
            for field in ty.members() {
                text.push_str(&wire_pack(
                    target,
                    &format!("({expression}).{}", field.name),
                    &field.value_type,
                    at,
                )?);
                at += field.value_type.wire_words();
            }
            text
        }
        Type::Vector { length } => {
            let mut text = String::new();
            for index in 0..*length {
                text.push_str(&wire_pack(
                    target,
                    &format!("({expression})[{index}]"),
                    &Type::Fixed,
                    offset + index,
                )?);
            }
            text
        }
        Type::AssetRef { .. } | Type::ClassRef { .. } => format!(
            "{target}.words[{offset}]=int32_t(uint32_t(uint64_t({expression})));{target}.words[{}]=int32_t(uint32_t(uint64_t({expression})>>32));\n",
            offset + 1
        ),
        Type::SequenceHandle | Type::EffectHandle | Type::EffectLayerRef { .. } => format!(
            "{target}.words[{offset}]=int32_t(uint32_t(({expression}).index));{target}.words[{}]=int32_t(uint32_t(({expression}).generation));\n",
            offset + 1
        ),
        _ => format!("{target}.words[{offset}]={};\n", pack(expression, ty)?),
    })
}

fn wire_unpack(expression: &str, ty: &Type, offset: usize) -> Result<String, String> {
    Ok(match ty {
        Type::Record { cpp_name, .. } => {
            bir::cpp_type(ty)?;
            let mut at = offset;
            let mut values = Vec::new();
            for field in ty.members() {
                values.push(wire_unpack(expression, &field.value_type, at)?);
                at += field.value_type.wire_words();
            }
            format!("{cpp_name}{{{}}}", values.join(","))
        }
        Type::Vector { length } => {
            let values = (0..*length)
                .map(|index| wire_unpack(expression, &Type::Fixed, offset + index))
                .collect::<Result<Vec<_>, _>>()?;
            format!("{{{}}}", values.join(","))
        }
        Type::AssetRef { .. } | Type::ClassRef { .. } => format!(
            "(uint64_t(uint32_t({expression}.words[{}]))|(uint64_t(uint32_t({expression}.words[{}]))<<32))",
            offset,
            offset + 1
        ),
        Type::SequenceHandle | Type::EffectHandle | Type::EffectLayerRef { .. } => {
            let cpp = bir::cpp_type(ty)?;
            format!(
                "{cpp}{{uint16_t(uint32_t({expression}.words[{offset}])),uint32_t({expression}.words[{}])}}",
                offset + 1
            )
        }
        Type::Void => "{}".into(),
        _ => unpack(&format!("{expression}.words[{offset}]"), ty)?,
    })
}

fn wire_dispatch_case(slot: usize, call: &str, returns: &Type) -> Result<String, String> {
    let words = returns.wire_words();
    if words > 32 {
        return Err(format!(
            "{} exceeds the 32-word Lua ABI v2 limit",
            returns.label()
        ));
    }
    if *returns == Type::Void {
        return Ok(format!(
            "case {slot}: {call}; return epok::lua::WireValue{{}};\n"
        ));
    }
    let packed = wire_pack("epok_wire", "epok_result", returns, 0)?;
    Ok(format!(
        "case {slot}: {{ const auto epok_result={call}; epok::lua::WireValue epok_wire{{}}; epok_wire.count={words};\n{packed}return epok_wire; }}\n"
    ))
}

// ------------------------------------------------------ intrinsic access ----

/// Getter and setter an intrinsic transform place lowers to. They are the very
/// entry points the Blueprint Get/Set Position, Rotation and Scale nodes call,
/// so a Lua body and a Blueprint graph address the same transform.
fn intrinsic_api(domain: Domain, kind: Intrinsic) -> (String, String) {
    // The rect places carry their own entry points; the World places take the
    // `_2d` suffix in the two-dimensional domain.
    let name = match kind {
        Intrinsic::RectPosition | Intrinsic::RectSize => kind.name().to_owned(),
        _ if domain == Domain::World2D => format!("{}_2d", kind.name()),
        _ => kind.name().to_owned(),
    };
    (
        format!("epok::bp::api::{name}"),
        format!("epok::bp::api::set_{name}"),
    )
}
/// `true` when the intrinsic is a single `Fixed`, which only the World2D
/// rotation is; every other intrinsic is indexed.
fn intrinsic_scalar(domain: Domain, kind: Intrinsic) -> bool {
    crate::lua_vm::intrinsic_components(domain, kind) == 1
}
/// Reads one intrinsic component of `handle`.
fn intrinsic_read(
    access: TransformAccess,
    kind: Intrinsic,
    component: usize,
    handle: &str,
) -> String {
    let (get, _) = intrinsic_api(access.domain, kind);
    if intrinsic_scalar(access.domain, kind) {
        format!("{get}({handle})")
    } else {
        format!("{get}({handle})[{component}]")
    }
}
/// Writes one intrinsic component of `handle`. A vector is read back, patched
/// and stored whole because the runtime exposes no component setter.
fn intrinsic_write(
    access: TransformAccess,
    kind: Intrinsic,
    component: usize,
    handle: &str,
    value: &str,
) -> String {
    let (get, set) = intrinsic_api(access.domain, kind);
    if intrinsic_scalar(access.domain, kind) {
        return format!("{set}({handle}, {value});\n");
    }
    format!(
        "{{ auto epok_v = {get}({handle}); epok_v[{component}] = {value}; {set}({handle}, epok_v); }}\n"
    )
}
/// The `ObjectId` an instance addresses: an actor is its own target, a
/// component targets the actor that owns it.
fn intrinsic_handle(access: TransformAccess, receiver: &str) -> String {
    let accessor = if access.through_owner {
        "owner_id()"
    } else {
        "id()"
    };
    format!("{receiver}{accessor}")
}

/// The actor a class's bodies address: an actor is its own, a component's is its
/// owner. It is the very expression `blueprint_ir::SelfKind::actor` produces, so
/// a spawn from Lua parents its instance exactly as the Blueprint node does.
fn self_actor(registry: &Registry, cpp_name: &str) -> &'static str {
    match crate::object_model::resolved_shape(registry, cpp_name).0 {
        schema::ClassFamily::Component => "this->get_owner()",
        _ => "this",
    }
}

pub fn operation_call_name(operation: &str) -> String {
    use sha2::{Digest, Sha256};
    format!(
        "epok_lua_operation_{:x}",
        Sha256::digest(operation.as_bytes())
    )
}

/// Generate one typed, bounds-checked adapter for every reachable foreign
/// receiver operation. Definitions live in a translation unit that includes
/// `scene.hh`, after every generated class is complete, avoiding header cycles
/// between mutually-calling Lua/Blueprint classes.
pub fn emit_operation_adapters(
    classes: &[(schema::Class, ClassIr)],
    registry: &Registry,
) -> Result<(String, Option<String>), String> {
    let mut operations = BTreeMap::<String, schema::Operation>::new();
    for (_, ir) in classes {
        for expression in crate::lua_vm::operation_sites(ir)? {
            let Expr::CallOperation { operation, .. } = expression else {
                continue;
            };
            if matches!(operation.receiver, schema::ReceiverKind::Instance { .. }) {
                operations.insert(operation.id.clone(), operation);
            }
        }
    }
    let mut header = String::from(
        "// Generated typed Lua foreign-receiver adapters.\n#pragma once\n#include \"epok.hpp\"\n",
    );
    if operations.is_empty() {
        return Ok((header, None));
    }
    let mut source = String::from(
        "// Generated Lua receiver calls; target classes are complete in scene.hh.\n#include \"scene.hh\"\n#include \"lua_calls.hpp\"\nnamespace { uint32_t epok_lua_receiver_depth=0; struct EpokLuaReceiverDepth { bool entered; EpokLuaReceiverDepth():entered(epok_lua_receiver_depth<32){if(entered)++epok_lua_receiver_depth;} ~EpokLuaReceiverDepth(){if(entered)--epok_lua_receiver_depth;} }; }\n",
    );
    for operation in operations.values() {
        let schema::ReceiverKind::Instance { class } = &operation.receiver else {
            continue;
        };
        let target = registry
            .classes
            .get(class)
            .ok_or_else(|| format!("Unknown receiver class {class}"))?;
        let returns = operation
            .outputs
            .first()
            .map(|output| output.value_type.clone())
            .unwrap_or(Type::Void);
        if operation.outputs.len() > 1 {
            return Err(format!("{} has multiple unwrapped outputs", operation.name));
        }
        let mut parameters = vec!["epok::ObjectId epok_receiver".to_string()];
        let mut arguments = Vec::new();
        for (index, parameter) in operation.parameters.iter().enumerate() {
            let ty = bir::cpp_type(&parameter.value_type)?;
            let ty = match parameter.direction {
                schema::Direction::Value => ty,
                schema::Direction::ConstReference => format!("const {ty}&"),
                schema::Direction::MutableReference => format!("{ty}&"),
            };
            parameters.push(format!("{ty} epok_argument_{index}"));
            arguments.push(format!("epok_argument_{index}"));
        }
        let signature = format!(
            "{} {}({})",
            bir::cpp_type(&returns)?,
            operation_call_name(&operation.id),
            parameters.join(",")
        );
        header.push_str(&format!("{signature};\n"));
        let fallback = if returns == Type::Void {
            "return;"
        } else {
            "return {};"
        };
        let object = format!("static_cast<{}*>(epok_object)", target.cpp_name);
        let invoke = if let Some(name) = operation.native_target.strip_prefix("property:get:") {
            if let Type::Vector { length } = &returns {
                let ty = bir::cpp_type(&returns)?;
                format!(
                    "{ty} epok_value{{}};for(unsigned i=0;i<{length};++i)epok_value[i]={object}->{name}[i];return epok_value;"
                )
            } else {
                format!("return {object}->{name};")
            }
        } else if let Some(name) = operation.native_target.strip_prefix("property:set:") {
            if let Some(schema::OperationParameter {
                value_type: Type::Vector { length },
                ..
            }) = operation.parameters.first()
            {
                format!(
                    "for(unsigned i=0;i<{length};++i){object}->{name}[i]=epok_argument_0[i];return;"
                )
            } else {
                format!("{object}->{name}=epok_argument_0;return;")
            }
        } else if returns == Type::Void {
            format!(
                "{object}->{}({});return;",
                operation.name,
                arguments.join(",")
            )
        } else {
            format!(
                "return {object}->{}({});",
                operation.name,
                arguments.join(",")
            )
        };
        source.push_str(&format!(
            "{signature}{{EpokLuaReceiverDepth epok_depth;if(!epok_depth.entered){{{fallback}}}if(!epok::active_object_registry){{{fallback}}}auto* epok_object=epok::active_object_registry->get(epok_receiver);if(!epok_object||!epok_object->is_a({}ULL)){{{fallback}}}epok::ObjectDispatchScope epok_scope(*epok::active_object_registry);{invoke}}}\n",
            crate::blueprint_refs::compact_id(class)
        ));
    }
    Ok((header, Some(source)))
}

// ---------------------------------------------------------- native bodies ----

struct Emitter<'a> {
    names: BTreeMap<LocalId, String>,
    parent: &'a str,
    source: String,
    transform: Option<TransformAccess>,
    /// The actor this class belongs to, for the builtins that spawn.
    self_actor: &'a str,
}
impl Emitter<'_> {
    fn access(&self) -> Result<TransformAccess, String> {
        self.transform
            .ok_or_else(|| "Intrinsic transform place on a non-spatial class".to_owned())
    }
}
impl Emitter<'_> {
    fn local(&self, id: LocalId) -> Result<String, String> {
        self.names
            .get(&id)
            .cloned()
            .ok_or_else(|| format!("Unresolved local {}", id.0))
    }
    fn place(&self, place: &Place) -> Result<String, String> {
        Ok(match place {
            Place::Local(id) => self.local(*id)?,
            Place::Property { name, .. } => format!("this->{name}"),
            Place::VectorComponent { base, index } => {
                format!("{}[{index}]", self.place(base)?)
            }
            Place::Intrinsic {
                kind, component, ..
            } => {
                let access = self.access()?;
                intrinsic_read(
                    access,
                    *kind,
                    *component,
                    &intrinsic_handle(access, "this->"),
                )
            }
        })
    }
    fn expr(&self, expr: &Expr) -> Result<String, String> {
        Ok(match expr {
            Expr::Literal { value, value_type } => bir::literal(value, value_type)?,
            Expr::Read { place, .. } => self.place(place)?,
            Expr::Unary {
                op,
                operand,
                value_type,
            } => {
                let operand = self.expr(operand)?;
                match op {
                    UnaryOp::Not => format!("(!({operand}))"),
                    UnaryOp::Negate if *value_type == Type::Int32 => {
                        format!("epok::bp::ineg({operand})")
                    }
                    UnaryOp::Negate => format!("epok::bp::neg({operand})"),
                }
            }
            Expr::Binary {
                op, left, right, ..
            } => {
                let (a, b) = (self.expr(left)?, self.expr(right)?);
                let operand = left.value_type();
                if op.logical() {
                    format!(
                        "(({a}){}({b}))",
                        if *op == BinaryOp::And { "&&" } else { "||" }
                    )
                } else if op.comparison() {
                    let token = match op {
                        BinaryOp::Eq => "==",
                        BinaryOp::Ne => "!=",
                        BinaryOp::Lt => "<",
                        BinaryOp::Le => "<=",
                        BinaryOp::Gt => ">",
                        _ => ">=",
                    };
                    format!("(({a}){token}({b}))")
                } else {
                    let prefix = match operand {
                        Type::Int32 => "i",
                        Type::UInt32 => "u",
                        _ => "",
                    };
                    let name = match op {
                        BinaryOp::Add => "add",
                        BinaryOp::Sub => "sub",
                        BinaryOp::Mul => "mul",
                        BinaryOp::Div => "div",
                        _ => "mod",
                    };
                    format!("epok::bp::{prefix}{name}({a}, {b})")
                }
            }
            Expr::Convert { kind, operand } => {
                let operand = self.expr(operand)?;
                match kind {
                    Conversion::IntToFixed => format!("epok::bp::from_int({operand})"),
                    Conversion::FixedToInt => format!("epok::bp::to_int({operand})"),
                }
            }
            Expr::Member {
                base, name, index, ..
            } => {
                // A record's components are named members; a vector's are a
                // native `Fixed[N]`, so they are reached by index exactly as a
                // write to the same component is.
                if matches!(base.value_type(), Type::Vector { .. }) {
                    format!("({})[{index}]", self.expr(base)?)
                } else {
                    format!("({}).{name}", self.expr(base)?)
                }
            }
            Expr::CallSelf { name, args, .. } => {
                format!("this->{name}({})", self.arguments(args)?)
            }
            Expr::CallParent { name, args, .. } => {
                format!("{}::{name}({})", self.parent, self.arguments(args)?)
            }
            // The very call the matching Blueprint node lowers to: both surfaces
            // go through `blueprint_ir::builtin_cpp`, so they cannot diverge.
            Expr::CallBuiltin {
                operation, args, ..
            } => {
                let lowered = args
                    .iter()
                    .map(|arg| match arg {
                        BuiltinArg::Value(value) => self.expr(value),
                        // A 64-bit asset or class handle is read straight out of
                        // its native field; it is never a value in a body.
                        BuiltinArg::Property { name, .. } => Ok(format!("this->{name}")),
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                bir::builtin_cpp(
                    operation,
                    &lowered,
                    &bir::SelfReceiver::this(self.self_actor),
                )?
            }
            Expr::CallOperation {
                operation,
                receiver,
                args,
                ..
            } => {
                let arguments = self.arguments(args)?;
                match &operation.receiver {
                    schema::ReceiverKind::Service { .. } => {
                        format!("{}({arguments})", operation.native_target)
                    }
                    schema::ReceiverKind::Instance { .. } => {
                        let receiver = receiver
                            .as_deref()
                            .ok_or_else(|| format!("{} has no receiver", operation.name))?;
                        let receiver = self.expr(receiver)?;
                        let separator = if arguments.is_empty() { "" } else { "," };
                        format!(
                            "{}({receiver}{separator}{arguments})",
                            operation_call_name(&operation.id)
                        )
                    }
                    schema::ReceiverKind::Value { .. } => {
                        return Err(format!(
                            "{} value-receiver lowering is not defined",
                            operation.name
                        ));
                    }
                }
            }
            Expr::MakeVector {
                components, length, ..
            } => format!(
                "epok::bp::Vector<{length}>{{{{{}}}}}",
                components
                    .iter()
                    .map(|c| self.expr(c))
                    .collect::<Result<Vec<_>, String>>()?
                    .join(",")
            ),
        })
    }
    fn arguments(&self, args: &[Expr]) -> Result<String, String> {
        Ok(args
            .iter()
            .map(|a| self.expr(a))
            .collect::<Result<Vec<_>, String>>()?
            .join(", "))
    }
    /// Vector places have array storage, so a whole-vector assignment copies
    /// component by component through one bound temporary.
    fn assign(&self, target: &Place, value: &Expr) -> Result<String, String> {
        let cpp = self.expr(value)?;
        if let Place::Intrinsic {
            kind, component, ..
        } = target
        {
            let access = self.access()?;
            return Ok(intrinsic_write(
                access,
                *kind,
                *component,
                &intrinsic_handle(access, "this->"),
                &cpp,
            ));
        }
        if let Type::Vector { length } = value.value_type() {
            let target = self.place(target)?;
            let mut text = format!("{{ const auto& epok_value = {cpp};");
            for i in 0..*length {
                text.push_str(&format!("{target}[{i}]=epok_value[{i}];"));
            }
            text.push_str("}\n");
            return Ok(text);
        }
        Ok(format!("{} = {cpp};\n", self.place(target)?))
    }
    fn block(&self, block: &Block, returns: &Type, out: &mut String) -> Result<(), String> {
        for statement in block {
            self.statement(statement, returns, out)?;
        }
        Ok(())
    }
    fn statement(
        &self,
        statement: &Statement,
        returns: &Type,
        out: &mut String,
    ) -> Result<(), String> {
        out.push_str(&format!(
            "\n#line {} \"{}\"\n",
            statement.span.line.max(1),
            self.source
        ));
        let mut call = false;
        match &statement.kind {
            StatementKind::Local { target, value } => {
                call = value.has_call();
                out.push_str(&self.assign(&Place::Local(*target), value)?);
            }
            StatementKind::Assign { target, value } => {
                call = value.has_call();
                out.push_str(&self.assign(target, value)?);
            }
            StatementKind::Evaluate(value) => {
                call = true;
                out.push_str(&format!("{};\n", self.expr(value)?));
            }
            StatementKind::If {
                cond,
                then,
                otherwise,
            } => {
                out.push_str(&format!("if ({}) {{\n", self.expr(cond)?));
                self.block(then, returns, out)?;
                out.push_str("} else {\n");
                self.block(otherwise, returns, out)?;
                out.push_str("}\n");
            }
            StatementKind::For {
                var,
                start,
                limit,
                step,
                body,
            } => {
                let name = self.local(*var)?;
                let test = if *step > 0 { "<=" } else { ">=" };
                out.push_str(&format!(
                    "for (int32_t {name}={start}; {name}{test}{limit}; {name}+={step}) {{\n"
                ));
                self.block(body, returns, out)?;
                out.push_str("}\n");
            }
            StatementKind::Return(value) => {
                let value = value
                    .as_ref()
                    .map(|v| self.expr(v))
                    .transpose()?
                    .map(|cpp| format!(" {cpp}"))
                    .unwrap_or_default();
                out.push_str(&format!("return{value};\n"));
            }
        }
        // Reentrancy: a call may destroy this object, so every statement that
        // entered user code rechecks the owner before touching a field again.
        if call {
            out.push_str(&format!(
                "if(!epok_owner.get())return{};\n",
                default_return(returns)
            ));
        }
        Ok(())
    }
}

fn native_body(
    method: &MethodIr,
    parent: &str,
    source: &str,
    transform: Option<TransformAccess>,
    self_actor: &'static str,
) -> Result<String, String> {
    let mut names = BTreeMap::new();
    for (i, parameter) in method.parameters.iter().enumerate() {
        names.insert(parameter.id, format!("epok_p{i}"));
    }
    for local in &method.locals {
        names.insert(local.id, format!("epok_l{}", local.id.0));
    }
    let emitter = Emitter {
        names,
        parent,
        source: source.into(),
        transform,
        self_actor,
    };
    let mut out = String::from("const auto epok_owner=this->id();(void)epok_owner;\n");
    for local in &method.locals {
        // Loop variables are declared by their own `for` statement.
        if method.body_declares_loop_variable(local.id) {
            continue;
        }
        out.push_str(&match &local.value_type {
            Type::Vector { length } => {
                format!("epok::bp::Vector<{length}> epok_l{}{{}};\n", local.id.0)
            }
            ty => format!("{} epok_l{}{{}};\n", bir::cpp_type(ty)?, local.id.0),
        });
    }
    emitter.block(&method.body, &method.function.returns, &mut out)?;
    Ok(out)
}

fn vm_body(method: &MethodIr, class_index: u32, slot: usize) -> Result<String, String> {
    let returns = &method.function.returns;
    let void = *returns == Type::Void;
    let wide = returns.wire_words() > 1
        || method
            .function
            .parameters
            .iter()
            .any(|parameter| parameter.value_type.wire_words() > 1);
    let mut out = format!(
        "epok::lua::Frame f(*this, kEpokLuaClass_{class_index}, /*slot*/ {slot});\nif (!f.bound()) return{};\n",
        default_return(returns)
    );
    for (i, parameter) in method.function.parameters.iter().enumerate() {
        if parameter.value_type.wire_words() > 1 {
            let words = parameter.value_type.wire_words();
            if words > 32 {
                return Err(format!(
                    "{} exceeds the 32-word Lua ABI v2 limit",
                    parameter.value_type.label()
                ));
            }
            out.push_str(&format!(
                "epok::lua::WireValue epok_argument_{i}{{}};epok_argument_{i}.count={words};\n{}f.arg(epok_argument_{i});\n",
                wire_pack(
                    &format!("epok_argument_{i}"),
                    &format!("epok_p{i}"),
                    &parameter.value_type,
                    0
                )?
            ));
        } else {
            out.push_str(&format!(
                "f.arg({});\n",
                pack(&format!("epok_p{i}"), &parameter.value_type)?
            ));
        }
    }
    if void {
        out.push_str("f.call(0);\n");
    } else if wide {
        out.push_str(&format!(
            "if (!f.call(1)) return {{}};\nconst auto epok_result=f.ret_wire();\nreturn {};\n",
            wire_unpack("epok_result", returns, 0)?
        ));
    } else {
        out.push_str(&format!(
            "if (!f.call(1)) return {{}};\nreturn {};\n",
            unpack("f.ret()", returns)?
        ));
    }
    Ok(out)
}

// ----------------------------------------------------------------- shell ----

pub fn emit_class(
    class: &schema::Class,
    ir: &ClassIr,
    registry: &Registry,
    mode: BodyMode,
    chunk_symbol: Option<&str>,
) -> Result<String, String> {
    let _ = chunk_symbol; // The payload is referenced by `emit_bindings`, not the shell.
    let parent = registry
        .named(&ir.parent_cpp_name)
        .ok_or_else(|| format!("Unknown parent {}", ir.parent_cpp_name))?;
    let source = project_relative(&class.source.file);
    let mut text = format!(
        "// Generated from Lua class {} ({source}). Do not edit.\n#pragma once\n#include \"{}\"\n#include \"blueprint_runtime.hpp\"\n#include \"blueprint_api.hpp\"\n#include \"lua_calls.hpp\"\n",
        class.id,
        parent_include(parent)?
    );
    if let BodyMode::Vm { class_index } = mode {
        text.push_str("#include \"lua_runtime.hpp\"\n");
        text.push_str(&format!(
            "constexpr uint32_t kEpokLuaClass_{class_index} = {class_index};\n"
        ));
    }
    text.push_str(&format!(
        "class {} : public {} {{\npublic:\n",
        class.cpp_name, parent.cpp_name
    ));
    text.push_str(&format!(
        "static constexpr uint64_t static_class_id={}ULL;\nuint64_t class_id() const override {{return static_class_id;}}\n",
        crate::blueprint_refs::compact_id(&class.id)
    ));
    for property in &class.properties {
        text.push_str(&cpp_storage(
            &property.name,
            &property.value_type,
            &property.default,
        )?);
    }
    text.push_str(&format!("{}() {{\nusing epok::Fixed;\n", class.cpp_name));
    for property in &class.properties {
        text.push_str(&crate::script_values::assignment(
            &format!("this->{}", property.name),
            &property.default,
            &property.value_type,
        )?);
    }
    text.push_str("}\n");
    for (slot, method) in ir.methods.iter().enumerate() {
        text.push_str(&format!(
            "virtual {} {}({}){} {{\n",
            bir::cpp_type(&method.function.returns)?,
            method.function.name,
            parameter_list(&method.function)?,
            if method.is_override { " override" } else { "" }
        ));
        for i in 0..method.function.parameters.len() {
            text.push_str(&format!("(void)epok_p{i};\n"));
        }
        text.push_str(&match mode {
            BodyMode::Native => native_body(
                method,
                &parent.cpp_name,
                &source,
                crate::lua_vm::intrinsic_access(registry, &ir.parent_cpp_name),
                self_actor(registry, &class.cpp_name),
            )?,
            BodyMode::Vm { class_index } => vm_body(method, class_index, slot)?,
        });
        text.push_str("}\n");
    }
    text.push_str("};\n");
    Ok(text)
}

/// One switch case of a generated dispatch. The call is bound to a temporary
/// before packing, because a wire form may mention its operand more than once
/// and a builtin that spawns must run exactly once.
fn dispatch_case(slot: usize, call: &str, returns: &Type) -> Result<String, String> {
    // A playback handle is wider than the value ABI, so it stays native: the
    // frontend only accepts such a builtin as a statement.
    if matches!(
        returns,
        Type::Void | Type::SequenceHandle | Type::EffectHandle
    ) {
        return Ok(format!("case {slot}: {call}; return 0;\n"));
    }
    Ok(format!(
        "case {slot}: {{ const auto epok_result = {call}; return {}; }}\n",
        pack("epok_result", returns)?
    ))
}

// -------------------------------------------------------------- bindings ----

pub fn emit_bindings(
    classes: &[(schema::Class, ClassIr)],
    registry: &Registry,
    chunk_symbols: &BTreeMap<String, String>,
) -> Result<String, String> {
    let mut text = String::from(
        "// Generated Lua class bindings. Do not edit.\n#include \"scene.hh\"\n#include \"lua_runtime.hpp\"\n#include \"blueprint_api.hpp\"\n#include \"lua_calls.hpp\"\n",
    );
    let mut rows = String::new();
    for (index, (class, ir)) in classes.iter().enumerate() {
        let symbol = chunk_symbols
            .get(&class.id)
            .ok_or_else(|| format!("Lua class {} has no packaged chunk", class.id))?;
        if !crate::scripts::identifier(symbol) {
            return Err(format!("Unsafe Lua chunk symbol {symbol}"));
        }
        text.push_str(&format!(
            "extern const unsigned char {symbol}[];\nextern const unsigned long {symbol}_size;\n"
        ));
        let name = &class.cpp_name;
        // `lua_vm::field_slots` is the single slot numbering both emitters use;
        // an intrinsic slot lowers to the same `epok::bp::api` call the native
        // bodies emit, so the two execution paths address one transform.
        let slots = crate::lua_vm::field_slots_for_ir(ir, registry);
        let transform = crate::lua_vm::intrinsic_access(registry, &class.cpp_name);
        let (mut get, mut set) = (String::new(), String::new());
        let (mut wide_get, mut wide_set) = (String::new(), String::new());
        for (slot, field) in slots.iter().enumerate() {
            if field.value_type.wire_words() > 1 {
                let SlotKey::Property {
                    component: None, ..
                } = &field.key
                else {
                    return Err(format!("{name} has an invalid composite field slot"));
                };
                let access = format!("self.{}", field.name);
                wide_get.push_str(&wire_dispatch_case(slot, &access, &field.value_type)?);
                let value = wire_unpack("value", &field.value_type, 0)?;
                if let Type::Vector { length } = &field.value_type {
                    wide_set.push_str(&format!(
                        "case {slot}: {{ const auto epok_value={value};{}return; }}\n",
                        (0..*length)
                            .map(|component| format!(
                                "{access}[{component}]=epok_value[{component}];"
                            ))
                            .collect::<String>()
                    ));
                } else {
                    wide_set.push_str(&format!("case {slot}: {access}={value};return;\n"));
                }
                continue;
            }
            let (read, write) = match &field.key {
                SlotKey::Property { component, .. } => {
                    let access = match component {
                        Some(i) => format!("self.{}[{i}]", field.name),
                        None => format!("self.{}", field.name),
                    };
                    (
                        pack(&access, &field.value_type)?,
                        format!(
                            "{access} = {}; return;\n",
                            unpack("value", &field.value_type)?
                        ),
                    )
                }
                SlotKey::Intrinsic { kind, component } => {
                    let access = transform.ok_or_else(|| {
                        format!("{name} has an intrinsic slot but no transform access")
                    })?;
                    let handle = intrinsic_handle(access, "self.");
                    (
                        pack(
                            &intrinsic_read(access, *kind, *component, &handle),
                            &Type::Fixed,
                        )?,
                        format!(
                            "{} return;\n",
                            intrinsic_write(
                                access,
                                *kind,
                                *component,
                                &handle,
                                &unpack("value", &Type::Fixed)?
                            )
                            .trim_end()
                        ),
                    )
                }
            };
            get.push_str(&format!("case {slot}: return {read};\n"));
            set.push_str(&format!("case {slot}: {write}"));
        }
        text.push_str(&format!(
            "static int32_t epok_lua_get_{index}(epok::Object& o, uint32_t slot) {{\nauto& self=static_cast<{name}&>(o);(void)self;\nswitch(slot) {{\n{get}default: return 0;\n}}\n}}\n"
        ));
        text.push_str(&format!(
            "static void epok_lua_set_{index}(epok::Object& o, uint32_t slot, int32_t value) {{\nauto& self=static_cast<{name}&>(o);(void)self;(void)value;\nswitch(slot) {{\n{set}default: return;\n}}\n}}\n"
        ));
        text.push_str(&format!(
            "static epok::lua::WireValue epok_lua_wide_get_{index}(epok::Object& o, uint32_t slot) {{\nauto& self=static_cast<{name}&>(o);(void)self;\nswitch(slot) {{\n{wide_get}default: return {{}};\n}}\n}}\n"
        ));
        text.push_str(&format!(
            "static void epok_lua_wide_set_{index}(epok::Object& o, uint32_t slot, const epok::lua::WireValue& value) {{\nauto& self=static_cast<{name}&>(o);(void)self;(void)value;\nswitch(slot) {{\n{wide_set}default: return;\n}}\n}}\n"
        ));
        // Own methods first, then only inherited functions referenced by this
        // class's typed IR. Unused engine APIs retain no binding cases.
        let methods = crate::lua_vm::method_slots(ir, registry);
        // A method the parent does not declare has no qualified parent call to
        // emit: `self.Parent::report()` would not compile for a method this
        // class introduces, and `__epok_super` can never reach that slot.
        let inherited: BTreeSet<&str> = registry
            .ancestry(&ir.parent_cpp_name)
            .iter()
            .flat_map(|c| c.functions.iter().map(|f| f.name.as_str()))
            .collect();
        let (mut self_cases, mut super_cases) = (String::new(), String::new());
        let (mut wide_self_cases, mut wide_super_cases) = (String::new(), String::new());
        for (slot, function) in methods.iter().enumerate() {
            let name = &function.name;
            let wide = function.returns.wire_words() > 1
                || function
                    .parameters
                    .iter()
                    .any(|parameter| parameter.value_type.wire_words() > 1);
            let args = function
                .parameters
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    if wide {
                        wire_unpack(&format!("args[{i}]"), &p.value_type, 0)
                    } else {
                        unpack(&format!("args[{i}]"), &p.value_type)
                    }
                })
                .collect::<Result<Vec<_>, String>>()?
                .join(", ");
            let mut targets = if wide {
                vec![(&mut wide_self_cases, format!("self.{name}"))]
            } else {
                vec![(&mut self_cases, format!("self.{name}"))]
            };
            if inherited.contains(name.as_str()) {
                if wide {
                    targets.push((
                        &mut wide_super_cases,
                        format!("self.{}::{name}", ir.parent_cpp_name),
                    ));
                } else {
                    targets.push((
                        &mut super_cases,
                        format!("self.{}::{name}", ir.parent_cpp_name),
                    ));
                }
            }
            for (cases, receiver) in targets {
                if wide {
                    cases.push_str(&wire_dispatch_case(
                        slot,
                        &format!("{receiver}({args})"),
                        &function.returns,
                    )?);
                } else {
                    cases.push_str(&dispatch_case(
                        slot,
                        &format!("{receiver}({args})"),
                        &function.returns,
                    )?);
                }
            }
        }
        let dispatch = |suffix: &str, receiver: &str, cases: &str| {
            format!(
                "static int32_t epok_lua_{suffix}_{index}(epok::Object& o, uint32_t slot, const int32_t* args, uint32_t argc) {{\nauto& self=static_cast<{name}&>(o);(void)self;(void)args;(void)argc;\nswitch(slot) {{\n{cases}default: return 0;\n}}\n}}\n// {receiver}\n"
            )
        };
        text.push_str(&dispatch("self_call", "virtual dispatch", &self_cases));
        text.push_str(&dispatch(
            "super_call",
            "qualified parent dispatch",
            &super_cases,
        ));
        let wide_dispatch = |suffix: &str, receiver: &str, cases: &str| {
            format!(
                "static epok::lua::WireValue epok_lua_{suffix}_{index}(epok::Object& o, uint32_t slot, const epok::lua::WireValue* args, uint32_t argc) {{\nauto& self=static_cast<{name}&>(o);(void)self;(void)args;(void)argc;\nswitch(slot) {{\n{cases}default: return {{}};\n}}\n}}\n// {receiver}\n"
            )
        };
        text.push_str(&wide_dispatch(
            "wide_self_call",
            "wide virtual dispatch",
            &wide_self_cases,
        ));
        text.push_str(&wide_dispatch(
            "wide_super_call",
            "wide qualified parent dispatch",
            &wide_super_cases,
        ));
        // Builtin call sites. The case body is the same `builtin_cpp` lowering
        // the native backend emits, so a VM build reaches the identical
        // `epok::bp::api` entry point with the identical operands. A 64-bit
        // asset or class id is read here, from the native field, and never
        // crosses the boundary.
        let sites = crate::lua_vm::builtin_sites(ir)?;
        let actor = match crate::object_model::resolved_shape(registry, &class.cpp_name).0 {
            schema::ClassFamily::Component => "self.get_owner()",
            _ => "&self",
        };
        let receiver = bir::SelfReceiver {
            id: "self.id()",
            object: "&self",
            actor,
        };
        let mut builtin_cases = String::new();
        for (site, expr) in sites.iter().enumerate() {
            let Expr::CallBuiltin {
                operation,
                args,
                returns,
                ..
            } = expr
            else {
                return Err(format!("{name}: builtin site {site} is not a builtin call"));
            };
            let mut lowered = Vec::new();
            let mut wire = 0usize;
            for arg in args {
                lowered.push(match arg {
                    BuiltinArg::Value(value) => {
                        let text = unpack(&format!("args[{wire}]"), value.value_type())?;
                        wire += 1;
                        text
                    }
                    BuiltinArg::Property { name, .. } => format!("self.{name}"),
                });
            }
            let call = bir::builtin_cpp(operation, &lowered, &receiver)?;
            builtin_cases.push_str(&dispatch_case(site, &call, returns)?);
        }
        text.push_str(&format!(
            "static int32_t epok_lua_builtin_{index}(epok::Object& o, uint32_t site, const int32_t* args, uint32_t argc) {{\nauto& self=static_cast<{name}&>(o);(void)self;(void)args;(void)argc;\nswitch(site) {{\n{builtin_cases}default: return 0;\n}}\n}}\n"
        ));
        let operation_sites = crate::lua_vm::operation_sites(ir)?;
        let mut operation_cases = String::new();
        for (site, expr) in operation_sites.iter().enumerate() {
            let Expr::CallOperation {
                operation,
                receiver,
                args,
                returns,
                ..
            } = expr
            else {
                return Err(format!(
                    "{name}: operation site {site} is not a catalog call"
                ));
            };
            if args.len() != operation.parameters.len() {
                return Err(format!("{} operation argument count drift", operation.name));
            }
            let receiver_words = usize::from(receiver.is_some());
            let lowered = operation
                .parameters
                .iter()
                .enumerate()
                .map(|(argument, parameter)| {
                    wire_unpack(
                        &format!("args[{}]", argument + receiver_words),
                        &parameter.value_type,
                        0,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            let call = match &operation.receiver {
                schema::ReceiverKind::Service { .. } => {
                    if receiver.is_some() {
                        return Err(format!("{} service has a receiver value", operation.name));
                    }
                    format!("{}({})", operation.native_target, lowered.join(","))
                }
                schema::ReceiverKind::Instance { .. } => {
                    let receiver = receiver.as_deref().ok_or_else(|| {
                        format!("{} instance call has no receiver", operation.name)
                    })?;
                    let target = wire_unpack("args[0]", receiver.value_type(), 0)?;
                    let separator = if lowered.is_empty() { "" } else { "," };
                    format!(
                        "{}({target}{separator}{})",
                        operation_call_name(&operation.id),
                        lowered.join(",")
                    )
                }
                schema::ReceiverKind::Value { .. } => {
                    return Err(format!(
                        "{} value-receiver lowering is not defined",
                        operation.name
                    ));
                }
            };
            operation_cases.push_str(&wire_dispatch_case(site, &call, returns)?);
        }
        text.push_str(&format!(
            "static epok::lua::WireValue epok_lua_operation_{index}(epok::Object& o, uint32_t site, const epok::lua::WireValue* args, uint32_t argc) {{\nauto& self=static_cast<{name}&>(o);(void)self;(void)args;(void)argc;\nswitch(site) {{\n{operation_cases}default: return {{}};\n}}\n}}\n"
        ));
        let names = methods
            .iter()
            .map(|f| format!("\"{}\"", f.name))
            .collect::<Vec<_>>()
            .join(",");
        text.push_str(&format!(
            "static const char* const epok_lua_methods_{index}[] = {{{names}}};\n"
        ));
        rows.push_str(&format!(
            "{{UINT64_C({}), \"{name}\", {symbol}, size_t({symbol}_size), epok_lua_methods_{index}, {}, epok_lua_get_{index}, epok_lua_set_{index}, epok_lua_wide_get_{index}, epok_lua_wide_set_{index}, epok_lua_self_call_{index}, epok_lua_super_call_{index}, epok_lua_wide_self_call_{index}, epok_lua_wide_super_call_{index}, epok_lua_builtin_{index}, epok_lua_operation_{index}}},\n",
            crate::blueprint_refs::compact_id(&class.id),
            methods.len()
        ));
    }
    text.push_str(&format!(
        "namespace epok::lua {{\nconst ClassBinding class_bindings[] = {{\n{rows}}};\nconst uint32_t class_binding_count = {};\n}}\n",
        classes.len()
    ));
    Ok(text)
}

impl MethodIr {
    fn body_declares_loop_variable(&self, id: LocalId) -> bool {
        fn scan(block: &Block, id: LocalId) -> bool {
            block.iter().any(|s| match &s.kind {
                StatementKind::For { var, body, .. } => *var == id || scan(body, id),
                StatementKind::If {
                    then, otherwise, ..
                } => scan(then, id) || scan(otherwise, id),
                _ => false,
            })
        }
        scan(&self.body, id)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::lua_asset;

    pub(crate) fn lower(path: &str, source: &str, registry: &Registry) -> (Registry, ClassIr) {
        let file = lua_asset::tests::fixture(path, source);
        let declaration = lua_asset::extract(&file).expect("declaration");
        let class = lua_asset::declarations(&declaration, &file, registry).expect("class");
        let mut registry = registry.clone();
        registry.classes.insert(class.id.clone(), class);
        registry.normalize_functions();
        let chunk = crate::lua_frontend::parse(&file).expect("chunk");
        let class = registry.classes[&declaration.id].clone();
        let ir = crate::lua_frontend::lower_class(&declaration, &chunk, &class, &registry)
            .expect("lowered body");
        (registry, ir)
    }
    /// The profile new projects use, where a whole vector is a value and
    /// reading one of its components takes the member path.
    fn lower_v2(path: &str, source: &str, registry: &Registry) -> (Registry, ClassIr) {
        let file = lua_asset::tests::fixture(path, source);
        let declaration = lua_asset::extract(&file).expect("declaration");
        let class = lua_asset::declarations(&declaration, &file, registry).expect("class");
        let mut registry = registry.clone();
        registry.classes.insert(class.id.clone(), class);
        registry.normalize_functions();
        let chunk = crate::lua_frontend::parse(&file).expect("chunk");
        let class = registry.classes[&declaration.id].clone();
        let ir = crate::lua_frontend::lower_class_with_profile(
            &declaration,
            &chunk,
            &class,
            &registry,
            crate::settings::LuaProfile::GameplayV2,
        )
        .expect("lowered body");
        (registry, ir)
    }

    fn enemy_logic(body: &str) -> String {
        format!(
            "{}\n{body}\nreturn EnemyLogic\n",
            lua_asset::tests::HEADER.trim_start()
        )
    }

    #[test]
    fn lua_aot_emits_the_documented_native_class() {
        let registry = lua_asset::tests::registry();
        let source = enemy_logic(&format!(
            "{}function EnemyLogic:damage(amount)\n    self.health = self.health - amount\nend\n",
            lua_asset::tests::DAMAGE
        ));
        let (registry, ir) = lower("assets/scripts/EnemyLogic.lua", &source, &registry);
        let class = registry.classes["956f4946-0c61-42f8-899e-2db063b42420"].clone();
        let text = emit_class(&class, &ir, &registry, BodyMode::Native, None).unwrap();
        for expected in [
            "// Generated from Lua class 956f4946-0c61-42f8-899e-2db063b42420 (assets/scripts/EnemyLogic.lua). Do not edit.\n#pragma once\n#include \"epok.hpp\"\n#include \"blueprint_runtime.hpp\"\n",
            "class EnemyLogic : public epok::ActorComponent {\npublic:\n",
            "uint64_t class_id() const override {return static_class_id;}\n",
            "epok::Fixed health = epok::Fixed(409600,epok::Fixed::RAW);\n",
            "uint32_t charges = 3u;\n",
            "bool ready = true;\n",
            "EnemyLogic() {\nusing epok::Fixed;\n",
            "this->health = Fixed(409600, Fixed::RAW);\n",
            "virtual void damage(epok::Fixed epok_p0) {\n",
            "const auto epok_owner=this->id();(void)epok_owner;\n",
            "this->health = epok::bp::sub(this->health, epok_p0);\n",
        ] {
            assert!(text.contains(expected), "missing {expected:?} in\n{text}");
        }
        // A native build never reaches the VM runtime or its class indices.
        assert!(!text.contains("lua_runtime.hpp") && !text.contains("kEpokLuaClass_"));
    }

    /// A vector property is a native `epok::Fixed[N]`, so reading one of its
    /// components is an index. Emitting `.y` compiled as a member access and
    /// only failed in the target compiler, long after the Lua frontend had
    /// accepted the source.
    #[test]
    fn a_vector_property_component_is_read_by_index() {
        let registry = lua_asset::tests::registry();
        let source = enemy_logic(&format!(
            "---@id 9a13c2d4-4f2f-4a0d-8f2a-4a1f2c3d4e5f\n\
             EnemyLogic.velocity = epok.Vector3(0.0, 0.0, 0.0)\n\
             {}function EnemyLogic:damage(amount)\n    \
             self.velocity.y = self.velocity.y - amount\n    \
             if self.velocity.y > amount then\n        self.health = amount\n    end\nend\n",
            lua_asset::tests::DAMAGE
        ));
        let (registry, ir) = lower_v2("assets/scripts/EnemyLogic.lua", &source, &registry);
        let class = registry.classes["956f4946-0c61-42f8-899e-2db063b42420"].clone();
        let text = emit_class(&class, &ir, &registry, BodyMode::Native, None).unwrap();
        assert!(text.contains("epok::Fixed velocity[3]{};\n"), "{text}");
        assert!(
            text.contains("this->velocity[1] = epok::bp::sub((this->velocity)[1], epok_p0);"),
            "a vector component must be read and written by index in\n{text}"
        );
        assert!(
            text.contains("((this->velocity)[1])>(epok_p0)"),
            "a vector component read must be an index in\n{text}"
        );
        assert!(
            !text.contains("velocity).y") && !text.contains("velocity.y"),
            "{text}"
        );
    }

    /// The body both backends must lower identically: one builtin of every
    /// shape the profile exposes.
    pub(crate) const BUILTIN_BODY: &str = r#"---@id 224e6b46-e4b4-475c-9744-8b9fb4c0baaa
---@param amount Fixed
function EnemyLogic:damage(amount)
    local other = epok.spawn("Spinner")
    if epok.is_valid(other) then
        epok.set_texture(self.ref, self.skin)
        epok.play_audio(other)
    end
    if epok.input.held(4, 0) then
        self.health = amount
    end
end
"#;

    fn builtin_class() -> (Registry, schema::Class, ClassIr) {
        let registry = crate::lua_frontend::tests::builtin_registry();
        let source = enemy_logic(BUILTIN_BODY);
        let (registry, ir) = lower("assets/scripts/EnemyLogic.lua", &source, &registry);
        let class = registry.classes["956f4946-0c61-42f8-899e-2db063b42420"].clone();
        (registry, class, ir)
    }

    #[test]
    fn lua_aot_lowers_builtins_to_the_blueprint_adapter_entry_points() {
        let (registry, class, ir) = builtin_class();
        let text = emit_class(&class, &ir, &registry, BodyMode::Native, None).unwrap();
        let spinner = crate::blueprint_refs::compact_id(crate::lua_frontend::tests::SPINNER_ID);
        for expected in [
            // The spawn parents its instance on this actor, exactly as the
            // Blueprint Spawn node does, and names the class by its uuid.
            format!("epok::bp::spawn_actor(this,{spinner}ULL,this->id())"),
            "epok::bp::api::valid(epok_l1)".into(),
            // The 64-bit asset id is read from the native field, never boxed.
            "epok::bp::api::set_texture(this->id(),this->skin)".into(),
            "epok::bp::api::play_audio(epok_l1)".into(),
            "epok::bp::api::held(4u,0u)".into(),
        ] {
            assert!(text.contains(&expected), "missing {expected:?} in\n{text}");
        }
        // An impure builtin may destroy this object, so the owner is rechecked.
        assert!(text.contains("if(!epok_owner.get())return;"));
    }

    #[test]
    fn lua_aot_emits_one_binding_case_per_builtin_site() {
        let (registry, class, ir) = builtin_class();
        let symbols = BTreeMap::from([(class.id.clone(), crate::lua_vm::chunk_symbol(&class.id))]);
        let text = emit_bindings(&[(class, ir)], &registry, &symbols).unwrap();
        let spinner = crate::blueprint_refs::compact_id(crate::lua_frontend::tests::SPINNER_ID);
        for expected in [
            // The identical adapter calls, with the wire operands unpacked from
            // the argument array and the asset id still read natively.
            // The call is bound once: a spawn must never run twice because its
            // wire form mentions the value more than one time.
            format!(
                "case 1: {{ const auto epok_result = epok::bp::spawn_actor(&self,{spinner}ULL,epok::ObjectId"
            ),
            "case 2: { const auto epok_result = epok::bp::api::valid(epok::ObjectId".into(),
            "case 4: epok::bp::api::set_texture(epok::ObjectId{uint16_t(uint32_t(args[0])&0xffffu),uint16_t(uint32_t(args[0])>>16)},self.skin); return 0;".into(),
            "case 6: { const auto epok_result = epok::bp::api::held(uint32_t(args[0]),uint32_t(args[1])); return ((epok_result)?1:0); }".into(),
            "epok_lua_builtin_0(epok::Object& o, uint32_t site".into(),
            "epok_lua_super_call_0, epok_lua_wide_self_call_0, epok_lua_wide_super_call_0, epok_lua_builtin_0, epok_lua_operation_0}".into(),
        ] {
            assert!(text.contains(&expected), "missing {expected:?} in\n{text}");
        }
    }

    #[test]
    fn lua_aot_calls_the_lexical_parent_not_the_virtual_override() {
        let mut registry = lua_asset::tests::registry();
        // A reflected user base is authored inside assets/scripts; its header is
        // what the generated class includes.
        registry
            .classes
            .get_mut(lua_asset::tests::ENEMY_BASE_ID)
            .unwrap()
            .source
            .file = "assets/scripts/Enemies/EnemyBase.hpp".into();
        let source = "---@class Guard : EnemyBase\nlocal Guard = EnemyBase:extend()\nfunction Guard:begin_play()\n    Guard.super.begin_play(self)\nend\nreturn Guard\n";
        let (registry, ir) = lower("assets/scripts/Guard.lua", source, &registry);
        assert!(
            emit_class(
                &registry.classes["b7c9d1e3-4f5a-4b6c-8d7e-9f0a1b2c3d4e"],
                &ir,
                &registry,
                BodyMode::Native,
                None
            )
            .unwrap()
            .contains("#include \"../../Enemies/EnemyBase.hpp\"")
        );
        let class = registry.classes["b7c9d1e3-4f5a-4b6c-8d7e-9f0a1b2c3d4e"].clone();
        let text = emit_class(&class, &ir, &registry, BodyMode::Native, None).unwrap();
        assert!(text.contains("class Guard : public EnemyBase {"));
        assert!(text.contains("virtual void begin_play() override {"));
        assert!(text.contains("EnemyBase::begin_play();"), "{text}");
        assert!(!text.contains("this->begin_play()"));
    }

    /// The shared fixture parent, restated as a spatial class so the chunk in
    /// `HEADER` becomes a World3D actor or a component attached to one.
    fn spatial(family: schema::ClassFamily) -> Registry {
        let mut registry = lua_asset::tests::registry();
        let parent = registry
            .classes
            .get_mut(crate::object_model::ACTOR_COMPONENT_ID)
            .unwrap();
        parent.family = Some(family);
        if family == schema::ClassFamily::Component {
            parent.component = Some(schema::ComponentContract {
                owners: [schema::Domain::World3D].into_iter().collect(),
                ..Default::default()
            });
        } else {
            parent.domain = Some(schema::Domain::World3D);
        }
        registry
    }

    /// Intrinsic places lower to the very `epok::bp::api` calls the Blueprint
    /// Get/Set Rotation nodes emit, in both execution modes and through the
    /// handle the class's family dictates.
    #[test]
    fn lua_aot_lowers_intrinsic_transform_places_to_the_blueprint_api() {
        let source = enemy_logic(&format!(
            "{}function EnemyLogic:damage(amount)\n    self.rotation.y = self.rotation.y + amount\nend\n",
            lua_asset::tests::DAMAGE
        ));
        for (family, handle) in [
            (schema::ClassFamily::Actor, "this->id()"),
            (schema::ClassFamily::Component, "this->owner_id()"),
        ] {
            let (registry, ir) = lower("assets/scripts/EnemyLogic.lua", &source, &spatial(family));
            let class = registry.classes["956f4946-0c61-42f8-899e-2db063b42420"].clone();
            let text = emit_class(&class, &ir, &registry, BodyMode::Native, None).unwrap();
            assert!(text.contains("#include \"blueprint_api.hpp\"\n"), "{text}");
            assert!(
                text.contains(&format!(
                    "{{ auto epok_v = epok::bp::api::rotation({handle}); epok_v[1] = epok::bp::add(epok::bp::api::rotation({handle})[1], epok_p0); epok::bp::api::set_rotation({handle}, epok_v); }}\n"
                )),
                "{text}"
            );
        }

        // Slots: the reflected hierarchy first, then position.xyz, rotation.xyz
        // and scale.xyz, so a new property can never land on an intrinsic slot.
        let (registry, ir) = lower(
            "assets/scripts/EnemyLogic.lua",
            &source,
            &spatial(schema::ClassFamily::Actor),
        );
        let class = registry.classes["956f4946-0c61-42f8-899e-2db063b42420"].clone();
        let names = crate::lua_vm::field_slots(&class.cpp_name, &registry)
            .into_iter()
            .map(|s| s.name)
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            [
                "armour", "charges", "health", "offset", "offset", "offset", "ready", "position",
                "position", "position", "rotation", "rotation", "rotation", "scale", "scale",
                "scale"
            ]
        );
        // The normalized Lua reaches the same slot through the ordinary field
        // helpers, so nothing in a chunk names a transform function at all.
        let chunk = crate::lua_vm::emit_chunk(
            &ir,
            &crate::lua_vm::field_slots(&class.cpp_name, &registry),
            &crate::lua_vm::method_slots(&ir, &registry),
            std::path::Path::new("assets/scripts/EnemyLogic.lua"),
        )
        .unwrap();
        assert!(
            chunk.contains("__epok_setf(self, 11, __epok_fadd(__epok_getf(self, 11), epok_p0))"),
            "{chunk}"
        );
        let symbols = BTreeMap::from([(class.id.clone(), "epok_lua_chunk_0".to_string())]);
        let bindings = emit_bindings(&[(class, ir)], &registry, &symbols).unwrap();
        assert!(
            bindings.contains("case 11: return (epok::bp::api::rotation(self.id())[1]).raw();\n"),
            "{bindings}"
        );
        assert!(
            bindings.contains(
                "case 11: { auto epok_v = epok::bp::api::rotation(self.id()); epok_v[1] = epok::Fixed(value,epok::Fixed::RAW); epok::bp::api::set_rotation(self.id(), epok_v); } return;\n"
            ),
            "{bindings}"
        );
        assert!(
            bindings.contains("case 7: return (epok::bp::api::position(self.id())[0]).raw();\n"),
            "{bindings}"
        );
        assert!(
            bindings.contains("case 15: return (epok::bp::api::scale(self.id())[2]).raw();\n"),
            "{bindings}"
        );
    }

    #[test]
    fn lua_aot_rechecks_the_owner_after_a_call_and_maps_lines_to_the_lua_source() {
        let registry = lua_asset::tests::registry();
        let source = enemy_logic(&format!(
            "{}function EnemyLogic:damage(amount)\nend\n\
             ---@param amount Fixed\n---@return Fixed\n\
             function EnemyLogic:absorb(amount)\n    self:damage(amount)\n    return self.health\nend\n",
            lua_asset::tests::DAMAGE
        ));
        let (registry, ir) = lower("assets/scripts/EnemyLogic.lua", &source, &registry);
        let class = registry.classes["956f4946-0c61-42f8-899e-2db063b42420"].clone();
        let text = emit_class(&class, &ir, &registry, BodyMode::Native, None).unwrap();
        assert!(
            text.contains("this->damage(epok_p0);\nif(!epok_owner.get())return {};\n"),
            "{text}"
        );
        assert!(
            text.contains("#line 18 \"assets/scripts/EnemyLogic.lua\"\nthis->damage"),
            "{text}"
        );
        assert!(
            text.contains("#line 19 \"assets/scripts/EnemyLogic.lua\"\nreturn this->health;"),
            "{text}"
        );
    }

    #[test]
    fn lua_aot_emits_vm_trampolines_and_slot_accurate_bindings() {
        let registry = lua_asset::tests::registry();
        let source = enemy_logic(
            "function EnemyLogic:tick(delta)\n    self.health = self.health - delta\nend\n\
             ---@param amount Fixed\n---@return Fixed\n\
             function EnemyLogic:absorb(amount)\n    return self.health\nend\n",
        );
        let (registry, ir) = lower("assets/scripts/EnemyLogic.lua", &source, &registry);
        let class = registry.classes["956f4946-0c61-42f8-899e-2db063b42420"].clone();
        let text = emit_class(
            &class,
            &ir,
            &registry,
            BodyMode::Vm { class_index: 0 },
            Some("epok_lua_chunk_0"),
        )
        .unwrap();
        assert!(
            text.contains(
                "#include \"lua_runtime.hpp\"\nconstexpr uint32_t kEpokLuaClass_0 = 0;\n"
            )
        );
        let tick = ir
            .methods
            .iter()
            .position(|m| m.function.name == "tick")
            .unwrap();
        let absorb = ir
            .methods
            .iter()
            .position(|m| m.function.name == "absorb")
            .unwrap();
        assert!(text.contains(&format!(
            "virtual void tick(epok::Fixed epok_p0) override {{\n(void)epok_p0;\nepok::lua::Frame f(*this, kEpokLuaClass_0, /*slot*/ {tick});\nif (!f.bound()) return;\nf.arg((epok_p0).raw());\nf.call(0);\n}}\n"
        )), "{text}");
        assert!(text.contains(&format!(
            "virtual epok::Fixed absorb(epok::Fixed epok_p0) {{\n(void)epok_p0;\nepok::lua::Frame f(*this, kEpokLuaClass_0, /*slot*/ {absorb});\nif (!f.bound()) return {{}};\nf.arg((epok_p0).raw());\nif (!f.call(1)) return {{}};\nreturn epok::Fixed(f.ret(),epok::Fixed::RAW);\n}}\n"
        )), "{text}");

        let symbols = BTreeMap::from([(class.id.clone(), "epok_lua_chunk_0".to_string())]);
        let bindings = emit_bindings(&[(class.clone(), ir.clone())], &registry, &symbols).unwrap();
        // Slots follow `registry.properties` order: armour (inherited), charges,
        // health, offset.{x,y,z}, ready. Inherited fields get slots exactly like
        // own fields, and a vector occupies one slot per component.
        let slots = crate::lua_vm::field_slots(&class.cpp_name, &registry)
            .into_iter()
            .map(|s| s.name)
            .collect::<Vec<_>>();
        assert_eq!(
            slots,
            [
                "armour", "charges", "health", "offset", "offset", "offset", "ready"
            ]
        );
        assert!(
            bindings.contains("case 0: return (self.armour).raw();\n"),
            "{bindings}"
        );
        assert!(
            bindings
                .contains("case 0: self.armour = epok::Fixed(value,epok::Fixed::RAW); return;\n")
        );
        assert!(
            bindings.contains("case 4: return (self.offset[1]).raw();\n"),
            "{bindings}"
        );
        assert!(
            bindings.contains("case 6: return ((self.ready)?1:0);\n"),
            "{bindings}"
        );
        assert!(bindings.contains("case 6: self.ready = ((value)!=0); return;\n"));
        assert!(bindings.contains(&format!(
            "case {tick}: self.tick(epok::Fixed(args[0],epok::Fixed::RAW)); return 0;\n"
        )));
        assert!(bindings.contains(&format!(
            "case {absorb}: {{ const auto epok_result = self.absorb(epok::Fixed(args[0],epok::Fixed::RAW)); return (epok_result).raw(); }}\n"
        )));
        assert!(bindings.contains(&format!(
            "case {tick}: self.epok::ActorComponent::tick(epok::Fixed(args[0],epok::Fixed::RAW)); return 0;\n"
        )), "{bindings}");
        // Unreferenced inherited callables do not consume a native dispatch
        // case merely because the registry advertises them.
        let methods = crate::lua_vm::method_slots(&ir, &registry);
        assert!(methods.iter().all(|f| f.name != "hit"), "{methods:?}");
        assert!(!bindings.contains("self.hit("), "{bindings}");
        // `absorb` is introduced by this class, so there is no parent member to
        // qualify; emitting a super case for it would not compile.
        let super_switch = bindings.split("epok_lua_super_call_0").nth(1).unwrap();
        assert!(
            !super_switch.contains(&format!("case {absorb}:")),
            "{super_switch}"
        );
        assert!(!bindings.contains("::absorb("), "{bindings}");
        assert!(bindings.contains("epok_lua_methods_0[] = {"));
        assert!(bindings.contains("const ClassBinding class_bindings[] = {"));
        assert!(bindings.contains("const uint32_t class_binding_count = 1;"));
        assert!(bindings.contains("epok_lua_chunk_0, size_t(epok_lua_chunk_0_size)"));
    }
}
