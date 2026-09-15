//! Language-neutral typed body IR shared by every scripting backend.
//!
//! The frontend resolves names, types, member identities and evaluation order
//! once. An emitter walks this tree and never re-analyses the source: every
//! member id and name is kept so generated code can carry `#line` source maps
//! and stable bindings.
#![allow(dead_code)] // M2 (the AOT and VM emitters) consumes this IR; M1 only produces it.
use crate::reflection_schema::{self as schema, Type};
use std::collections::BTreeSet;

const BOOL: Type = Type::Bool;
const INT32: Type = Type::Int32;
const FIXED: Type = Type::Fixed;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Span {
    pub line: u32,
    pub column: u32,
}
impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocalId(pub u32);

#[derive(Clone, Debug)]
pub struct Local {
    pub id: LocalId,
    pub name: String,
    pub value_type: Type,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnaryOp {
    /// `ineg`/`neg` depending on the operand type. Never a raw C++ negation.
    Negate,
    Not,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}
impl BinaryOp {
    pub fn arithmetic(self) -> bool {
        matches!(
            self,
            Self::Add | Self::Sub | Self::Mul | Self::Div | Self::Mod
        )
    }
    pub fn comparison(self) -> bool {
        matches!(
            self,
            Self::Eq | Self::Ne | Self::Lt | Self::Le | Self::Gt | Self::Ge
        )
    }
    pub fn logical(self) -> bool {
        matches!(self, Self::And | Self::Or)
    }
    /// Ordering comparisons are undefined for the unordered types.
    pub fn ordering(self) -> bool {
        matches!(self, Self::Lt | Self::Le | Self::Gt | Self::Ge)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Conversion {
    IntToFixed,
    FixedToInt,
}

/// Transform place every spatial class owns without declaring it. It is not a
/// reflected property: it addresses the owning actor's root component through
/// the same `epok::bp::api` entry points the Blueprint Get/Set nodes use.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Intrinsic {
    Position,
    Rotation,
    Scale,
    /// UI-domain rect places, addressing the same `RectTransformComponent` the
    /// Blueprint Get/Set Rect Position and Rect Size nodes address.
    RectPosition,
    RectSize,
}
impl Intrinsic {
    pub fn name(self) -> &'static str {
        match self {
            Self::Position => "position",
            Self::Rotation => "rotation",
            Self::Scale => "scale",
            Self::RectPosition => "rect_position",
            Self::RectSize => "rect_size",
        }
    }
    pub const ALL: [Self; 5] = [
        Self::Position,
        Self::Rotation,
        Self::Scale,
        Self::RectPosition,
        Self::RectSize,
    ];
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.name() == name)
    }
}

/// A writable storage location. Property places keep the reflected member id so
/// an emitter binds to the native field without consulting the registry again.
#[derive(Clone, Debug)]
pub enum Place {
    Local(LocalId),
    Property {
        member_id: String,
        name: String,
        value_type: Type,
    },
    VectorComponent {
        base: Box<Place>,
        index: usize,
    },
    /// One scalar of an intrinsic transform. `component` indexes the vector and
    /// is 0 for the scalar World2D rotation; `value_type` is always `Fixed`.
    Intrinsic {
        kind: Intrinsic,
        component: usize,
        value_type: Type,
    },
}
impl Place {
    pub fn root_local(&self) -> Option<LocalId> {
        match self {
            Self::Local(id) => Some(*id),
            Self::Property { .. } | Self::Intrinsic { .. } => None,
            Self::VectorComponent { base, .. } => base.root_local(),
        }
    }
    pub fn value_type(&self, locals: &[Local]) -> Option<Type> {
        match self {
            Self::Local(id) => locals
                .iter()
                .find(|l| l.id == *id)
                .map(|l| l.value_type.clone()),
            Self::Property { value_type, .. } | Self::Intrinsic { value_type, .. } => {
                Some(value_type.clone())
            }
            Self::VectorComponent { base, index } => match base.value_type(locals)? {
                Type::Vector { length } if *index < length => Some(Type::Fixed),
                _ => None,
            },
        }
    }
}

#[derive(Clone, Debug)]
pub enum Expr {
    Literal {
        value: serde_json::Value,
        value_type: Type,
    },
    /// `value_type` is redundant with `place` but keeps `Expr::value_type`
    /// total without threading the local table through every emitter call.
    Read { place: Place, value_type: Type },
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
        value_type: Type,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
        value_type: Type,
    },
    Convert {
        kind: Conversion,
        operand: Box<Expr>,
    },
    CallSelf {
        function_id: String,
        name: String,
        args: Vec<Expr>,
        returns: Type,
    },
    /// Lexically qualified parent call; never the most-derived runtime type.
    CallParent {
        function_id: String,
        name: String,
        args: Vec<Expr>,
        returns: Type,
    },
    MakeVector {
        components: Vec<Expr>,
        length: usize,
        value_type: Type,
    },
    /// A Blueprint builtin node invoked from a script body. The operation is the
    /// Blueprint node itself, not a copy of it, so both authoring surfaces reach
    /// the same `epok::bp::api` entry point by construction.
    ///
    /// `site` is a class-wide dense index assigned by the frontend. VM backends
    /// dispatch on it through the generated per-class `builtin_call` switch, so
    /// the 64-bit operands a builtin may take (asset ids, class ids) are read
    /// natively and never cross the 32-bit boundary of contract section 10.1.
    CallBuiltin {
        site: u32,
        operation: crate::blueprint_asset::Builtin,
        args: Vec<BuiltinArg>,
        returns: Type,
        pure: bool,
    },
}

/// One operand of a builtin call.
#[derive(Clone, Debug)]
pub enum BuiltinArg {
    /// An ordinary value; it crosses the VM boundary as an `int32_t`.
    Value(Expr),
    /// A direct read of a declared 64-bit `AssetRef`/`ClassRef` property. It is
    /// never a value in a body: the generated binding case reads the native
    /// field, so the id never enters Lua.
    Property {
        member_id: String,
        name: String,
        value_type: Type,
    },
}
impl BuiltinArg {
    pub fn value(&self) -> Option<&Expr> {
        match self {
            Self::Value(expr) => Some(expr),
            Self::Property { .. } => None,
        }
    }
}
impl Expr {
    pub fn value_type(&self) -> &Type {
        match self {
            Self::Literal { value_type, .. }
            | Self::Read { value_type, .. }
            | Self::Unary { value_type, .. }
            | Self::Binary { value_type, .. }
            | Self::MakeVector { value_type, .. } => value_type,
            Self::Convert { kind, .. } => match kind {
                Conversion::IntToFixed => &FIXED,
                Conversion::FixedToInt => &INT32,
            },
            Self::CallSelf { returns, .. }
            | Self::CallParent { returns, .. }
            | Self::CallBuiltin { returns, .. } => returns,
        }
    }
    /// Calls are hoisted into temporaries by the frontend, so a well-formed
    /// expression tree never contains one below its own root.
    pub fn has_call(&self) -> bool {
        match self {
            Self::CallSelf { .. } | Self::CallParent { .. } => true,
            // A pure builtin has no effect to order and no reentrancy to guard,
            // so it may stay nested inside a larger expression.
            Self::CallBuiltin { pure, args, .. } => {
                !*pure
                    || args
                        .iter()
                        .filter_map(BuiltinArg::value)
                        .any(Self::has_call)
            }
            Self::Literal { .. } | Self::Read { .. } => false,
            Self::Unary { operand, .. } | Self::Convert { operand, .. } => operand.has_call(),
            Self::Binary { left, right, .. } => left.has_call() || right.has_call(),
            Self::MakeVector { components, .. } => components.iter().any(Self::has_call),
        }
    }
}

pub type Block = Vec<Statement>;

#[derive(Clone, Debug)]
pub struct Statement {
    pub kind: StatementKind,
    pub span: Span,
}
#[derive(Clone, Debug)]
pub enum StatementKind {
    Local {
        target: LocalId,
        value: Expr,
    },
    Assign {
        target: Place,
        value: Expr,
    },
    Evaluate(Expr),
    If {
        cond: Expr,
        then: Block,
        otherwise: Block,
    },
    /// Constant bounds only; the frontend already proved the iteration budget.
    For {
        var: LocalId,
        start: i64,
        limit: i64,
        step: i64,
        body: Block,
    },
    Return(Option<Expr>),
}
impl Statement {
    pub fn new(kind: StatementKind, span: Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Clone, Debug)]
pub struct MethodIr {
    pub function: schema::Function,
    pub is_override: bool,
    pub parameters: Vec<Local>,
    pub locals: Vec<Local>,
    pub body: Block,
    pub span: Span,
}
impl MethodIr {
    /// Parameters and body locals share one numbering space.
    pub fn all_locals(&self) -> Vec<Local> {
        self.parameters
            .iter()
            .chain(&self.locals)
            .cloned()
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct ClassIr {
    pub class_id: String,
    pub cpp_name: String,
    pub parent_cpp_name: String,
    pub methods: Vec<MethodIr>,
}

/// Assignment compatibility for the structural check. A reference may widen to
/// a less specific reference of the same family: both are exactly one
/// `ObjectId`, and the frontend already checked the class relation against the
/// registry, which this pass deliberately does not consult.
fn compatible(declared: &Type, value: &Type) -> bool {
    declared == value
        || matches!(
            (declared, value),
            (Type::ActorRef { .. }, Type::ActorRef { .. })
                | (Type::ComponentRef { .. }, Type::ComponentRef { .. })
                | (
                    Type::ObjectRef { .. },
                    Type::ObjectRef { .. } | Type::ActorRef { .. } | Type::ComponentRef { .. }
                )
        )
}

/// Q12 raw encoding, identical to `blueprint_ir::literal` and the runtime.
pub fn fixed_raw(value: f64) -> i32 {
    (value * 4096.).round() as i32
}

/// Structural check. It does not repeat name resolution: it verifies that the
/// tree an emitter is about to walk is internally consistent.
pub fn validate(class: &ClassIr) -> Result<(), String> {
    if class.class_id.is_empty() || class.cpp_name.is_empty() {
        return Err("Class IR has no identity".into());
    }
    let mut names = BTreeSet::new();
    for method in &class.methods {
        if !names.insert(method.function.name.clone()) {
            return Err(format!("Duplicate method {}", method.function.name));
        }
        validate_method(class, method)?;
    }
    Ok(())
}
fn validate_method(class: &ClassIr, method: &MethodIr) -> Result<(), String> {
    let where_ = |m: String| format!("{}::{}: {m}", class.cpp_name, method.function.name);
    if method.parameters.len() != method.function.parameters.len() {
        return Err(where_(
            "Parameter count differs from the resolved signature".into(),
        ));
    }
    for (local, declared) in method.parameters.iter().zip(&method.function.parameters) {
        if local.value_type != declared.value_type || local.name != declared.name {
            return Err(where_(format!(
                "Parameter {} differs from the signature",
                local.name
            )));
        }
    }
    let locals = method.all_locals();
    let mut ids = BTreeSet::new();
    for local in &locals {
        if !ids.insert(local.id) {
            return Err(where_(format!("Duplicate local id {}", local.id.0)));
        }
    }
    validate_block(&method.body, &locals, &method.function.returns).map_err(where_)
}
/// Structural check for the one place shape that carries no reflected member to
/// cross-check against: an intrinsic addresses at most three Fixed components.
fn validate_place(place: &Place) -> Result<(), String> {
    match place {
        Place::Intrinsic {
            component,
            value_type,
            ..
        } if *component > 2 || value_type != &FIXED => {
            Err("Intrinsic transform place is not a Fixed component".into())
        }
        Place::VectorComponent { base, .. } => validate_place(base),
        _ => Ok(()),
    }
}
fn validate_block(block: &Block, locals: &[Local], returns: &Type) -> Result<(), String> {
    for statement in block {
        let at = |m: String| format!("{} {m}", statement.span);
        match &statement.kind {
            StatementKind::Local { target, value } => {
                let declared = Place::Local(*target)
                    .value_type(locals)
                    .ok_or_else(|| at(format!("Undeclared local {}", target.0)))?;
                validate_expr(value, locals).map_err(&at)?;
                if !compatible(&declared, value.value_type()) {
                    return Err(at("Local initializer type differs from the local".into()));
                }
            }
            StatementKind::Assign { target, value } => {
                validate_place(target).map_err(&at)?;
                let declared = target
                    .value_type(locals)
                    .ok_or_else(|| at("Assignment target is not a valid place".into()))?;
                validate_expr(value, locals).map_err(&at)?;
                if !compatible(&declared, value.value_type()) {
                    return Err(at("Assigned value type differs from the target".into()));
                }
            }
            StatementKind::Evaluate(value) => {
                validate_expr(value, locals).map_err(&at)?;
                if !matches!(
                    value,
                    Expr::CallSelf { .. } | Expr::CallParent { .. } | Expr::CallBuiltin { .. }
                ) {
                    return Err(at("Only calls may be evaluated for effect".into()));
                }
            }
            StatementKind::If {
                cond,
                then,
                otherwise,
            } => {
                validate_expr(cond, locals).map_err(&at)?;
                if cond.value_type() != &BOOL {
                    return Err(at("Condition is not Bool".into()));
                }
                validate_block(then, locals, returns)?;
                validate_block(otherwise, locals, returns)?;
            }
            StatementKind::For {
                var,
                start,
                limit,
                step,
                body,
            } => {
                if Place::Local(*var).value_type(locals) != Some(Type::Int32) {
                    return Err(at("Loop variable must be a declared Int32 local".into()));
                }
                if *step == 0 {
                    return Err(at("Loop step is zero".into()));
                }
                let count = (limit - start) / step;
                if !(0..65536).contains(&count) {
                    return Err(at(
                        "Loop is unbounded or exceeds the iteration budget".into()
                    ));
                }
                validate_block(body, locals, returns)?;
            }
            StatementKind::Return(value) => match (value, returns) {
                (None, Type::Void) => {}
                (Some(value), expected) if expected != &Type::Void => {
                    validate_expr(value, locals).map_err(&at)?;
                    if !compatible(expected, value.value_type()) {
                        return Err(at("Returned value differs from the declared return".into()));
                    }
                }
                _ => return Err(at("Return arity differs from the declared return".into())),
            },
        }
    }
    Ok(())
}
fn validate_expr(expr: &Expr, locals: &[Local]) -> Result<(), String> {
    match expr {
        Expr::Literal { value, value_type } => {
            if !crate::script_values::valid(value, value_type) {
                return Err(format!("Literal {value} is not {}", value_type.label()));
            }
        }
        Expr::Read { place, value_type } => {
            validate_place(place)?;
            let actual = place
                .value_type(locals)
                .ok_or_else(|| "Read from an unresolved place".to_owned())?;
            if &actual != value_type {
                return Err("Cached read type differs from its place".into());
            }
        }
        Expr::Unary {
            op,
            operand,
            value_type,
        } => {
            validate_expr(operand, locals)?;
            let ok = match op {
                UnaryOp::Not => value_type == &BOOL && operand.value_type() == &BOOL,
                UnaryOp::Negate => {
                    value_type == operand.value_type()
                        && matches!(value_type, Type::Int32 | Type::Fixed)
                }
            };
            if !ok {
                return Err("Unary operand type is not supported".into());
            }
        }
        Expr::Binary {
            op,
            left,
            right,
            value_type,
        } => {
            validate_expr(left, locals)?;
            validate_expr(right, locals)?;
            if left.value_type() != right.value_type() {
                return Err("Binary inputs must have exactly the same type".into());
            }
            let operand = left.value_type();
            let ok = if op.logical() {
                operand == &BOOL && value_type == &BOOL
            } else if op.comparison() {
                value_type == &BOOL
                    && matches!(
                        operand,
                        Type::Bool | Type::Int32 | Type::UInt32 | Type::Fixed | Type::Enum { .. }
                    )
                    && (!op.ordering() || operand != &BOOL)
            } else if *op == BinaryOp::Mod {
                value_type == operand && matches!(operand, Type::Int32 | Type::UInt32)
            } else {
                value_type == operand
                    && matches!(
                        operand,
                        Type::Int32 | Type::UInt32 | Type::Fixed | Type::Vector { .. }
                    )
            };
            if !ok {
                return Err("Unsupported operator for the declared type".into());
            }
        }
        Expr::Convert { kind, operand } => {
            validate_expr(operand, locals)?;
            let expected = match kind {
                Conversion::IntToFixed => &INT32,
                Conversion::FixedToInt => &FIXED,
            };
            if operand.value_type() != expected {
                return Err("Conversion operand has the wrong type".into());
            }
        }
        Expr::CallSelf {
            function_id,
            name,
            args,
            ..
        }
        | Expr::CallParent {
            function_id,
            name,
            args,
            ..
        } => {
            if function_id.is_empty() || name.is_empty() {
                return Err("Call has no resolved member identity".into());
            }
            for arg in args {
                validate_expr(arg, locals)?;
                if arg.has_call() {
                    return Err(format!("{name}: nested call was not hoisted"));
                }
            }
        }
        Expr::CallBuiltin {
            operation,
            args,
            returns,
            ..
        } => {
            let (parameters, output, pure) = crate::blueprint_ir::builtin_signature(operation);
            if args.len() != parameters.len() {
                return Err(format!(
                    "{operation:?}: {} operand(s) for a {}-operand builtin",
                    args.len(),
                    parameters.len()
                ));
            }
            for (arg, (name, expected)) in args.iter().zip(&parameters) {
                match arg {
                    BuiltinArg::Value(value) => {
                        validate_expr(value, locals)?;
                        if value.has_call() {
                            return Err(format!("{name}: nested call was not hoisted"));
                        }
                    }
                    // The 64-bit operands never become values; only their own
                    // reference types may be spelled this way.
                    BuiltinArg::Property { value_type, .. } => {
                        if !matches!(value_type, Type::AssetRef { .. } | Type::ClassRef { .. })
                            || value_type != expected
                        {
                            return Err(format!(
                                "{name}: property operand is not {}",
                                expected.label()
                            ));
                        }
                    }
                }
            }
            // `Cast` and the spawn family narrow the declared output to the
            // resolved class; everything else keeps the node's own return.
            if *returns != output
                && !matches!(
                    returns,
                    Type::ActorRef { .. } | Type::ComponentRef { .. } | Type::ObjectRef { .. }
                )
            {
                return Err(format!(
                    "{operation:?}: return type differs from the builtin"
                ));
            }
            let _ = pure;
        }
        Expr::MakeVector {
            components,
            length,
            value_type,
        } => {
            if components.len() != *length || value_type != &(Type::Vector { length: *length }) {
                return Err("Vector construction has an inconsistent length".into());
            }
            for component in components {
                validate_expr(component, locals)?;
                if component.value_type() != &FIXED {
                    return Err("Vector components must be Fixed".into());
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn function(name: &str, returns: Type) -> schema::Function {
        schema::Function {
            id: "224e6b46-e4b4-475c-9744-8b9fb4c0baaa".into(),
            name: name.into(),
            parameters: vec![schema::Parameter {
                name: "amount".into(),
                value_type: Type::Fixed,
                direction: schema::Direction::Value,
            }],
            returns,
            callable: true,
            timeline: None,
            event: true,
            pure: false,
            abstract_method: false,
            final_method: false,
            access: "public".into(),
            overrides: vec![],
            source: schema::Location {
                file: "Enemy.lua".into(),
                line: 1,
                column: 1,
            },
        }
    }
    fn class(body: Block, locals: Vec<Local>) -> ClassIr {
        ClassIr {
            class_id: "956f4946-0c61-42f8-899e-2db063b42420".into(),
            cpp_name: "EnemyLogic".into(),
            parent_cpp_name: "epok::ActorComponent".into(),
            methods: vec![MethodIr {
                function: function("damage", Type::Void),
                is_override: false,
                parameters: vec![Local {
                    id: LocalId(0),
                    name: "amount".into(),
                    value_type: Type::Fixed,
                }],
                locals,
                body,
                span: Span { line: 1, column: 1 },
            }],
        }
    }
    fn health() -> Place {
        Place::Property {
            member_id: "3d352b2b-c2d7-4b99-9ba1-a003d648e897".into(),
            name: "health".into(),
            value_type: Type::Fixed,
        }
    }

    #[test]
    fn script_ir_accepts_a_resolved_body_and_rejects_mixed_types() {
        let read = |place: Place, value_type: Type| Expr::Read { place, value_type };
        let subtract = Expr::Binary {
            op: BinaryOp::Sub,
            left: Box::new(read(health(), Type::Fixed)),
            right: Box::new(read(Place::Local(LocalId(0)), Type::Fixed)),
            value_type: Type::Fixed,
        };
        let body = vec![Statement::new(
            StatementKind::Assign {
                target: health(),
                value: subtract,
            },
            Span { line: 2, column: 5 },
        )];
        validate(&class(body, vec![])).unwrap();

        let mixed = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(read(health(), Type::Fixed)),
            right: Box::new(Expr::Literal {
                value: json!(1),
                value_type: Type::Int32,
            }),
            value_type: Type::Fixed,
        };
        let body = vec![Statement::new(
            StatementKind::Assign {
                target: health(),
                value: mixed,
            },
            Span { line: 2, column: 5 },
        )];
        assert!(
            validate(&class(body, vec![]))
                .unwrap_err()
                .contains("exactly the same type")
        );
    }

    #[test]
    fn script_ir_rejects_unhoisted_calls_and_unbounded_loops() {
        let call = Expr::CallSelf {
            function_id: "224e6b46-e4b4-475c-9744-8b9fb4c0baaa".into(),
            name: "damage".into(),
            args: vec![Expr::CallSelf {
                function_id: "224e6b46-e4b4-475c-9744-8b9fb4c0baaa".into(),
                name: "damage".into(),
                args: vec![],
                returns: Type::Fixed,
            }],
            returns: Type::Void,
        };
        let body = vec![Statement::new(
            StatementKind::Evaluate(call),
            Span { line: 3, column: 1 },
        )];
        assert!(
            validate(&class(body, vec![]))
                .unwrap_err()
                .contains("was not hoisted")
        );

        let counter = Local {
            id: LocalId(1),
            name: "i".into(),
            value_type: Type::Int32,
        };
        let body = vec![Statement::new(
            StatementKind::For {
                var: LocalId(1),
                start: 1,
                limit: 1_000_000,
                step: 1,
                body: vec![],
            },
            Span { line: 3, column: 1 },
        )];
        assert!(
            validate(&class(body, vec![counter]))
                .unwrap_err()
                .contains("iteration budget")
        );
    }

    #[test]
    fn fixed_raw_matches_the_blueprint_q12_encoding() {
        assert_eq!(fixed_raw(0.5), 2048);
        assert_eq!(fixed_raw(100.), 409600);
        assert_eq!(fixed_raw(-1.25), -5120);
    }
}
