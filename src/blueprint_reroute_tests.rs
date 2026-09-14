use super::*;

fn socket(editor: &BlueprintEditor, index: usize, pin: &str, output: bool) -> Socket {
    let doc = editor.asset.as_ref().unwrap();
    let graph = editor.current().unwrap();
    node_sockets(doc, graph, &graph.nodes[index], &Registry::new())
        .into_iter()
        .find(|s| s.pin == pin && s.output == output)
        .unwrap()
}

#[test]
fn data_reroutes_preserve_parameter_wires_types_fanout_and_undo() {
    let mut editor = editor();
    editor.asset.as_mut().unwrap().functions[0]
        .parameters
        .push(schema::Parameter {
            name: "enabled".into(),
            value_type: schema::Type::Bool,
            direction: schema::Direction::Value,
        });
    editor.add_node(NodeKind::Branch);
    editor.add_node(NodeKind::Branch);
    let entry = editor.current().unwrap().entry.clone();
    for node in &mut editor.asset.as_mut().unwrap().functions[0].nodes[1..] {
        node.inputs.insert(
            "condition".into(),
            Input::Parameter {
                name: "enabled".into(),
            },
        );
    }
    let before = editor.asset.clone().unwrap();
    let wire = CanvasWire {
        from: socket(&editor, 0, "enabled", true),
        to: socket(&editor, 1, "condition", false),
        points: [[0., 0.]; 4],
    };
    editor.insert_reroute(&wire, [210., 90.]);
    editor.checkpoint(before.clone());
    let knot = &editor.current().unwrap().nodes[3];
    assert!(matches!(&knot.inputs["value"], Input::Parameter { name } if name == "enabled"));
    assert_eq!(
        socket(&editor, 3, "value", true).ty,
        SocketType::Value(schema::Type::Bool)
    );
    assert!(matches!(
        &editor.current().unwrap().nodes[2].inputs["condition"],
        Input::Parameter { .. }
    ));
    let target = socket(&editor, 2, "condition", false);
    editor.disconnect(&target);
    editor
        .connect(
            &socket(&editor, 3, "value", true),
            &target,
            &Registry::new(),
        )
        .unwrap();
    let graph = editor.current().unwrap();
    assert_eq!(
        input_connection(&entry, &graph.nodes[1].inputs["condition"]),
        input_connection(&entry, &graph.nodes[2].inputs["condition"])
    );
    editor.undo();
    assert_eq!(bytes(editor.asset.as_ref().unwrap()), bytes(&before));
    editor.redo();
    assert_eq!(editor.current().unwrap().nodes.len(), 4);
}

#[test]
fn execution_reroutes_split_only_one_branch_and_reject_cycles() {
    let mut editor = editor();
    editor.add_node(NodeKind::Sequence);
    editor.add_node(NodeKind::Return);
    let first=editor.current().unwrap().nodes[1].id.clone();
    editor.asset.as_mut().unwrap().functions[0].nodes[0].outputs.insert("next".into(),vec![first.clone()]);
    let wire=CanvasWire{from:socket(&editor,0,"next",true),to:socket(&editor,1,"exec",false),points:[[0.,0.];4]};
    editor.insert_reroute(&wire,[100.,100.]);
    let graph=editor.current().unwrap();
    assert_eq!(graph.nodes[0].outputs["next"],[graph.nodes[3].id.clone()]);
    assert_eq!(graph.nodes[3].outputs["next"],[first]);
    assert!(editor.connect(&socket(&editor,1,"then_0",true),&socket(&editor,3,"exec",false),&Registry::new()).is_err());
    editor.connect(&socket(&editor,3,"next",true),&socket(&editor,2,"exec",false),&Registry::new()).unwrap();
    assert_eq!(editor.current().unwrap().nodes[3].outputs["next"],[editor.current().unwrap().nodes[2].id.clone()]);
}

#[test]
fn wire_hit_testing_uses_the_drawn_curve_including_backward_wires() {
    let editor = editor();
    for points in [
        [[0., 0.], [100., 0.], [100., 200.], [200., 200.]],
        [[200., 0.], [300., 0.], [-100., 200.], [0., 200.]],
    ] {
        let wire = CanvasWire {
            from: socket(&editor, 0, "next", true),
            to: socket(&editor, 0, "next", true),
            points,
        };
        for t in [0., 0.1, 0.33, 0.8, 1.] {
            assert!(wire.distance(wire.point(t)) < 0.2);
        }
        assert!(wire.distance([500., 500.]) > 6.);
    }
}

#[test]
fn shared_event_canvas_connects_unassigned_islands_without_changing_other_events() {
    let mut editor = editor();
    editor.asset.as_mut().unwrap().functions[0].override_id = Some(id());
    let first_entry = editor.current().unwrap().entry.clone();
    let mut second = editor.current().unwrap().clone();
    second.id = id();
    second.name = "second_event".into();
    second.entry = id();
    second.nodes[0].id = second.entry.clone();
    editor.asset.as_mut().unwrap().functions.push(second);
    editor.graph = 1;
    editor.add_node(NodeKind::Sequence);
    let target = socket(&editor, 1, "exec", false);
    editor.graph = 0;
    let source = socket(&editor, 0, "next", true);
    editor.connect(&source, &target, &Registry::new()).unwrap();
    let doc = editor.asset.as_ref().unwrap();
    assert_eq!(canvas_graphs(doc, 0).len(), 2);
    assert_eq!(doc.functions[0].nodes.len(), 2);
    assert_eq!(doc.functions[1].nodes.len(), 1);
    assert_eq!(doc.functions[0].entry, first_entry);
    assert_eq!(doc.functions[0].nodes[0].outputs["next"], [target.node]);
    let before = bytes(doc);
    let target = socket(&editor, 1, "exec", false);
    editor.graph = 1;
    let other = socket(&editor, 0, "next", true);
    assert!(editor.connect(&other, &target, &Registry::new()).is_err());
    assert_eq!(bytes(editor.asset.as_ref().unwrap()), before);
}

#[test]
#[ignore = "Owns a real ImGui context; run explicitly and serially"]
fn double_click_wire_inserts_draggable_point_with_single_undo() {
    let mut context = crate::gui::tests::imgui_context();
    context.set_ini_filename(None);
    context.io_mut().display_size = [1600., 1000.];
    context.io_mut().delta_time = 1. / 60.;
    context
        .fonts()
        .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
    context.fonts().build_rgba32_texture();
    let mut editor = editor();
    editor.open = true;
    editor.maximized = true;
    editor.add_node(NodeKind::Sequence);
    let entry = editor.current().unwrap().entry.clone();
    let target = editor.current().unwrap().nodes[1].id.clone();
    let doc = editor.asset.as_mut().unwrap();
    doc.layout.positions.insert(entry.clone(), [0., 0.]);
    doc.layout.positions.insert(target.clone(), [350., 0.]);
    doc.functions[0].nodes[0]
        .outputs
        .insert("next".into(), vec![target.clone()]);
    let registry = Registry::new();
    let frame = |ctx: &mut imgui::Context, e: &mut BlueprintEditor| {
        e.draw(ctx.frame(), &registry);
        ctx.render();
    };
    let click = |ctx: &mut imgui::Context, e: &mut BlueprintEditor, p: [f32; 2]| {
        ctx.io_mut().add_mouse_pos_event(p);
        frame(ctx, e);
        ctx.io_mut().add_mouse_button_event(MouseButton::Left, true);
        frame(ctx, e);
        ctx.io_mut()
            .add_mouse_button_event(MouseButton::Left, false);
        frame(ctx, e);
    };
    frame(&mut context, &mut editor);
    frame(&mut context, &mut editor);
    let a = SOCKETS.with(|s| s.borrow()[&format!("{entry}:next:true")]);
    let b = SOCKETS.with(|s| s.borrow()[&format!("{target}:exec:false")]);
    let center = [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5];
    let before = editor.asset.clone().unwrap();
    let history = editor.undo.len();
    click(&mut context, &mut editor, center);
    click(&mut context, &mut editor, center);
    assert_eq!(editor.current().unwrap().nodes.len(), 3);
    assert_eq!(editor.undo.len(), history + 1);
    let knot = editor.current().unwrap().nodes[2].id.clone();
    let position = editor.asset.as_ref().unwrap().layout.positions[&knot];
    // Let the double-click interval expire before dragging the point's center.
    for _ in 0..25 {
        frame(&mut context, &mut editor);
    }
    context.io_mut().add_mouse_pos_event(center);
    context
        .io_mut()
        .add_mouse_button_event(MouseButton::Left, true);
    frame(&mut context, &mut editor);
    context
        .io_mut()
        .add_mouse_pos_event([center[0], center[1] + 70.]);
    frame(&mut context, &mut editor);
    context
        .io_mut()
        .add_mouse_button_event(MouseButton::Left, false);
    frame(&mut context, &mut editor);
    assert_ne!(
        editor.asset.as_ref().unwrap().layout.positions[&knot],
        position
    );
    editor.undo();
    assert_eq!(
        editor.asset.as_ref().unwrap().layout.positions[&knot],
        position
    );
    editor.undo();
    assert_eq!(bytes(editor.asset.as_ref().unwrap()), bytes(&before));
}
