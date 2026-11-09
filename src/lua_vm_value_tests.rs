use super::*;
use crate::script_ir::{Local, LocalId, MethodIr, Span};

fn snapshot() -> schema::Type {
    schema::Type::Record {
        cpp_name: "LifetimeSnapshot".into(),
        fields: (0..32)
            .map(|index| schema::RecordField {
                name: format!("word_{index}"),
                value_type: schema::Type::Int32,
            })
            .collect(),
    }
}
fn read() -> Expr {
    Expr::Read {
        place: Place::Property {
            member_id: "snapshot".into(),
            name: "snapshot".into(),
            value_type: snapshot(),
        },
        value_type: snapshot(),
    }
}
fn statement(kind: StatementKind) -> Statement {
    Statement::new(kind, Span { line: 1, column: 1 })
}
fn project(method: MethodIr) -> (schema::Class, script_ir::ClassIr) {
    let registry = crate::lua_frontend::tests::builtin_registry();
    let source = format!("{}\nreturn EnemyLogic\n", lua_asset::tests::HEADER);
    let (registry, mut ir) =
        crate::lua_aot::tests::lower("assets/scripts/EnemyLogic.lua", &source, &registry);
    let class = registry.named(&ir.cpp_name).unwrap().clone();
    ir.methods = vec![method];
    (class, ir)
}
fn method() -> MethodIr {
    MethodIr {
        function: schema::Function {
            id: "lifetime-method".into(),
            name: "exercise".into(),
            parameters: vec![],
            returns: schema::Type::Void,
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
                file: "Lifetime.lua".into(),
                line: 1,
                column: 1,
            },
        },
        is_override: false,
        parameters: vec![Local {
            id: LocalId(0),
            name: "input".into(),
            value_type: snapshot(),
        }],
        locals: vec![
            Local {
                id: LocalId(1),
                name: "saved".into(),
                value_type: snapshot(),
            },
            Local {
                id: LocalId(2),
                name: "copy".into(),
                value_type: snapshot(),
            },
        ],
        body: vec![],
        span: Span { line: 1, column: 1 },
    }
}

#[test]
fn value_plan_covers_roots_nested_expressions_branches_and_replace() {
    let operation = Expr::CallSelf {
        function_id: "lifetime-method".into(),
        name: "exercise".into(),
        args: vec![read(), read()],
        returns: snapshot(),
    };
    let mut method = method();
    method.body = vec![
        statement(StatementKind::Local {
            target: LocalId(1),
            value: read(),
        }),
        statement(StatementKind::If {
            cond: Expr::Literal {
                value: serde_json::json!(true),
                value_type: schema::Type::Bool,
            },
            then: vec![statement(StatementKind::Evaluate(operation))],
            otherwise: vec![statement(StatementKind::Return(Some(read())))],
        }),
    ];
    let pair = project(method);
    assert_eq!(value_slot_capacity(std::slice::from_ref(&pair)), 6); // Three roots, three expression snapshots.
    let slots = vec![FieldSlot {
        key: SlotKey::Property {
            member_id: "snapshot".into(),
            component: None,
        },
        name: "snapshot".into(),
        value_type: snapshot(),
    }];
    let chunk = emit_chunk(
        &pair.1,
        &slots,
        &[pair.1.methods[0].function.clone()],
        Path::new("Lifetime.lua"),
    )
    .unwrap();
    assert!(chunk.contains("S(epok_p0, epok_l1, epok_l2)"));
    assert_eq!(chunk.matches("local S=__epok_sweep\n").count(), 1);
    assert_eq!(chunk.matches("S(").count(), 3); // Only allocating statements.
    let assignment = statement(StatementKind::Assign {
        target: Place::Member {
            base: Box::new(Place::Local(LocalId(1))),
            name: "word_0".into(),
            index: 0,
            value_type: schema::Type::Int32,
        },
        value: Expr::Member {
            base: Box::new(read()),
            name: "word_0".into(),
            index: 0,
            value_type: schema::Type::Int32,
        },
    });
    assert_eq!(statement_value_slots(&assignment), 2); // Getter and immutable replacement.
}

#[test]
fn value_plan_is_independent_of_loop_iterations_and_keeps_all_64_live_slots() {
    let mut method = method();
    method.body = vec![statement(StatementKind::For {
        var: LocalId(3),
        start: 0,
        limit: 4095,
        step: 1,
        body: vec![statement(StatementKind::Local {
            target: LocalId(1),
            value: read(),
        })],
    })];
    let mut pair = project(method);
    assert_eq!(value_slot_capacity(std::slice::from_ref(&pair)), 4);
    let slots = vec![FieldSlot {
        key: SlotKey::Property {
            member_id: "snapshot".into(),
            component: None,
        },
        name: "snapshot".into(),
        value_type: snapshot(),
    }];
    let chunk = emit_chunk(
        &pair.1,
        &slots,
        &[pair.1.methods[0].function.clone()],
        Path::new("Lifetime.lua"),
    )
    .unwrap();
    assert!(chunk.contains("for epok_l3 = 0, 4095, 1 do\n    S(epok_p0, epok_l1, epok_l2)"));
    pair.1.methods[0].locals = (1..64)
        .map(|index| Local {
            id: LocalId(index),
            name: format!("root_{index}"),
            value_type: snapshot(),
        })
        .collect();
    assert_eq!(value_slot_capacity(&[pair]), 64);
}
