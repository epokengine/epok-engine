use super::*;
fn socket(e: &BlueprintEditor, index: usize, pin: &str, output: bool) -> Socket {
    let doc = e.asset.as_ref().unwrap();
    let g = &doc.functions[0];
    node_sockets(doc, g, &g.nodes[index], &Registry::new()).into_iter().find(|s| s.pin == pin && s.output == output).unwrap()
}
fn vector_editor() -> BlueprintEditor {
    let mut e = editor();
    e.add_node(NodeKind::VectorComponent { length: 3, index: 0 });
    e.asset.as_mut().unwrap().functions[0].nodes[1].inputs.insert("value".into(), Input::Literal { value_type: schema::Type::Vector { length: 3 }, value: json!([2,3,4]) });
    e
}
#[test]
fn split_inputs_roundtrip_values_clipboard_and_undo() {
    let mut e = vector_editor();
    let root = socket(&e, 1, "value", false);
    let before = e.asset.clone().unwrap();
    e.split_pin(&root, &Registry::new()).unwrap();
    e.checkpoint(before.clone());
    let child = socket(&e, 1, "value.x", false);
    assert_eq!(child.ty, SocketType::Value(schema::Type::Fixed));
    e.undo();
    assert_eq!(bytes(e.asset.as_ref().unwrap()), bytes(&before));
    e.redo();
    e.asset.as_mut().unwrap().functions[0].nodes[1].inputs.insert("value.y".into(), Input::Literal { value_type: schema::Type::Fixed, value: json!(9) });
    e.selected = BTreeSet::from([root.node.clone()]);
    e.copy_selected(); e.paste();
    assert_eq!(socket(&e, 2, "value.z", false).ty, child.ty);
    let persisted = bytes(e.asset.as_ref().unwrap());
    e.asset = Some(crate::document::from_slice(&persisted).unwrap());
    e.recombine_pin(&child, true).unwrap();
    assert!(matches!(&e.current().unwrap().nodes[1].inputs["value"], Input::Literal { value, .. } if *value == json!([2,9,4])));
    assert_eq!(e.current().unwrap().nodes[1].inputs.len(), 1);
}
#[test]
fn split_output_connections_are_typed_and_recombine_is_lossless() {
    let mut e = vector_editor();
    e.add_node(NodeKind::Literal { value_type: schema::Type::Vector { length: 3 }, value: json!([5,6,7]) });
    let root_in = socket(&e, 1, "value", false);
    let root_out = socket(&e, 2, "value", true);
    e.split_pin(&root_in, &Registry::new()).unwrap();
    e.split_pin(&root_out, &Registry::new()).unwrap();
    let input = socket(&e, 1, "value.y", false);
    let output = socket(&e, 2, "value.z", true);
    e.connect(&output, &input, &Registry::new()).unwrap();
    assert!(e.recombine_pin(&input, false).is_err());
    assert!(e.recombine_pin(&socket(&e, 2, "value.x", true), false).is_err());
    let doc = e.asset.as_ref().unwrap();
    assert_eq!(input_type(doc, &doc.functions[0], &doc.functions[0].nodes[1].inputs["value.y"], &Registry::new(), 0), Some(schema::Type::Fixed));
    e.disconnect(&input);
    e.recombine_pin(&input, true).unwrap();
    e.recombine_pin(&output, true).unwrap();
    assert!(e.asset.as_ref().unwrap().layout.split_pins.is_empty());
}
#[test]
fn nested_transform_split_preserves_scale_and_member_edits() {
    let mut e = editor();
    let ty = schema::Type::Record { cpp_name: "epok::Transform".into(), fields: vec![] };
    e.asset.as_mut().unwrap().functions[0].returns = ty;
    e.add_node(NodeKind::Return);
    e.split_pin(&socket(&e, 1, "value", false), &Registry::new()).unwrap();
    e.split_pin(&socket(&e, 1, "value.position", false), &Registry::new()).unwrap();
    e.asset.as_mut().unwrap().functions[0].nodes[1].inputs.insert("value.position.z".into(), Input::Literal { value_type: schema::Type::Fixed, value: json!(42) });
    e.recombine_pin(&socket(&e, 1, "value.scale", false), true).unwrap();
    assert!(matches!(&e.current().unwrap().nodes[1].inputs["value"], Input::Literal { value, .. } if value["position"][2] == 42 && value["scale"] == json!([1,1,1])));
}
#[test]
fn function_transform_input_splits_each_vector_then_recombines_without_losing_defaults() {
    let mut e=editor();
    let ty=schema::Type::Record {cpp_name:"epok::Transform".into(),fields:vec![]};
    let g=&mut e.asset.as_mut().unwrap().functions[0];
    g.parameters.push(schema::Parameter {name:"pose".into(),value_type:ty.clone(),direction:schema::Direction::ConstReference});
    let function=g.id.clone();
    e.add_node(NodeKind::Call {function});
    e.split_pin(&socket(&e,1,"pose",false),&Registry::new()).unwrap();
    for member in ["position","rotation","scale"] {
        let parent=socket(&e,1,&format!("pose.{member}"),false);
        assert_eq!(inline_values::parts(&e.current().unwrap().nodes[1],&parent).len(),3);
        e.split_pin(&parent,&Registry::new()).unwrap();
        for axis in ["x","y","z"] {
            let child=socket(&e,1,&format!("pose.{member}.{axis}"),false);
            let parts=inline_values::parts(&e.current().unwrap().nodes[1],&child);
            assert_eq!(parts.len(),1);
            assert_eq!(parts[0].1,if member=="scale" {json!(1)} else {json!(0.)});
        }
    }
    for member in ["position","rotation","scale"] {
        e.recombine_pin(&socket(&e,1,&format!("pose.{member}.x"),false),true).unwrap();
    }
    e.recombine_pin(&socket(&e,1,"pose.position",false),true).unwrap();
    assert!(matches!(&e.current().unwrap().nodes[1].inputs["pose"],Input::Literal {value,..} if *value==default_value(&ty)));
    assert_eq!(e.current().unwrap().nodes[1].inputs.len(),1);
}
#[test]
fn connected_parents_and_mutable_reference_inputs_cannot_split() {
    let mut e = vector_editor();
    let input = socket(&e, 1, "value", false);
    e.asset.as_mut().unwrap().functions[0].nodes[1].inputs.insert("value".into(), Input::Parameter { name: "vector".into() });
    assert!(e.split_pin(&input, &Registry::new()).unwrap_err().contains("Disconnect"));
    let ty = schema::Type::Record { cpp_name: "epok::Transform".into(), fields: vec![] };
    let doc = e.asset.as_mut().unwrap();
    doc.functions[0].parameters.push(schema::Parameter { name: "pose".into(), value_type: ty.clone(), direction: schema::Direction::MutableReference });
    let function = doc.functions[0].id.clone();
    e.add_node(NodeKind::Call { function });
    let input = socket(&e, 2, "pose", false);
    assert!(e.split_pin(&input, &Registry::new()).unwrap_err().contains("Mutable reference"));
    // Event outputs still expose readable fields from the actual argument.
    let output = socket(&e, 0, "pose", true);
    e.split_pin(&output, &Registry::new()).unwrap();
    assert_eq!(socket(&e, 0, "pose.scale", true).ty, SocketType::Value(schema::Type::Vector { length: 3 }));
}
#[test]
fn promote_input_preserves_default_and_output_creates_set_with_unique_variable() {
    let mut e = vector_editor();
    let input = socket(&e, 1, "value", false);
    let before = e.asset.clone().unwrap();
    e.promote_pin(&input, &Registry::new()).unwrap();
    e.checkpoint(before.clone());
    assert_eq!(e.asset.as_ref().unwrap().variables[0].default, json!([2,3,4]));
    let g = e.current().unwrap();
    assert!(matches!(g.nodes.last().unwrap().kind, NodeKind::GetVariable { .. }));
    assert!(matches!(&g.nodes[1].inputs["value"], Input::Link { node, pin } if node == &g.nodes[2].id && pin == "value"));
    e.undo(); assert_eq!(bytes(e.asset.as_ref().unwrap()), bytes(&before)); e.redo();
    let output = socket(&e, 2, "value", true);
    e.promote_pin(&output, &Registry::new()).unwrap();
    assert_ne!(e.asset.as_ref().unwrap().variables[0].name, e.asset.as_ref().unwrap().variables[1].name);
    assert!(matches!(e.current().unwrap().nodes.last().unwrap().kind, NodeKind::SetVariable { .. }));
    assert!(matches!(&e.current().unwrap().nodes.last().unwrap().inputs["value"], Input::Link { node, .. } if node == &output.node));
}
#[test]
fn promote_event_output_splices_execution_and_preview_never_edits_document() {
    let mut e = editor();
    e.asset.as_mut().unwrap().functions[0].parameters.push(schema::Parameter { name: "speed".into(), value_type: schema::Type::Fixed, direction: schema::Direction::Value });
    e.add_node(NodeKind::Return);
    let next = e.current().unwrap().nodes[1].id.clone();
    e.asset.as_mut().unwrap().functions[0].nodes[0].outputs.insert("next".into(), vec![next.clone()]);
    e.promote_pin(&socket(&e, 0, "speed", true), &Registry::new()).unwrap();
    let g = e.current().unwrap();
    assert_eq!(g.nodes[0].outputs["next"], vec![g.nodes[2].id.clone()]);
    assert_eq!(g.nodes[2].outputs["next"], vec![next]);
    let before = bytes(e.asset.as_ref().unwrap());
    assert!(!e.can_connect(&socket(&e, 0, "speed", true), &socket(&e, 2, "value", false), &Registry::new()));
    assert_eq!(before, bytes(e.asset.as_ref().unwrap()));
    e.asset.as_mut().unwrap().functions[0].nodes[0].outputs.clear();
    e.promote_pin(&socket(&e, 0, "speed", true), &Registry::new()).unwrap();
    let g=e.current().unwrap();
    assert_eq!(g.nodes[0].outputs["next"],vec![g.nodes.last().unwrap().id.clone()]);
}
#[test]
fn inline_defaults_and_splitting_follow_pin_types_for_vector2_vector3_and_structs() {
    for length in [2,3] {
        let mut e = editor();
        e.add_node(NodeKind::VectorComponent { length, index: 0 });
        let pin = socket(&e, 1, "value", false);
        let parts = inline_values::parts(&e.current().unwrap().nodes[1], &pin);
        assert_eq!(parts.len(), length);
        let field = inline_values::Field { socket:pin.clone(), axis:Some(length-1), value:json!(0), min:[0.,0.],max:[35.,15.] };
        let before = e.asset.clone().unwrap();
        e.commit_inline_value(&field,"-.5").unwrap(); e.checkpoint(before.clone());
        assert!(matches!(&e.current().unwrap().nodes[1].inputs["value"],Input::Literal{value,..} if value[length-1]==json!(-0.5)));
        let edited = bytes(e.asset.as_ref().unwrap());
        assert!(e.commit_inline_value(&field,"NaN").is_err());
        assert_eq!(edited,bytes(e.asset.as_ref().unwrap()));
        e.undo(); assert_eq!(bytes(&before),bytes(e.asset.as_ref().unwrap())); e.redo();
        e.split_pin(&pin,&Registry::new()).unwrap();
        let child = socket(&e,1,if length==2 { "value.y" } else { "value.z" },false);
        let field = inline_values::Field { socket:child.clone(), axis:None, value:json!(-0.5),min:[0.,0.],max:[35.,15.] };
        e.commit_inline_value(&field,"12.25").unwrap();
        e.recombine_pin(&child,true).unwrap();
        assert!(matches!(&e.current().unwrap().nodes[1].inputs["value"],Input::Literal{value,..} if value[length-1]==json!(12.25)));
    }
    let mut e = editor();
    let ty = schema::Type::Record { cpp_name:"Settings".into(), fields:vec![
        schema::RecordField { name:"offset".into(),value_type:schema::Type::Vector { length:2 } },
        schema::RecordField { name:"count".into(),value_type:schema::Type::Int32 },
    ] };
    e.asset.as_mut().unwrap().functions[0].returns=ty;
    e.add_node(NodeKind::Return);
    e.split_pin(&socket(&e,1,"value",false),&Registry::new()).unwrap();
    e.split_pin(&socket(&e,1,"value.offset",false),&Registry::new()).unwrap();
    let count=socket(&e,1,"value.count",false);
    let field=inline_values::Field {socket:count.clone(),axis:None,value:json!(0),min:[0.,0.],max:[35.,15.]};
    assert!(e.commit_inline_value(&field,"1.5").is_err());
    e.commit_inline_value(&field,"42").unwrap();
    e.promote_pin(&count,&Registry::new()).unwrap();
    assert_eq!(e.asset.as_ref().unwrap().variables[0].default,json!(42));
    assert_eq!(socket(&e,1,"value.offset.x",false).ty,SocketType::Value(schema::Type::Fixed));
}
#[test]
#[ignore = "Owns a real ImGui context; run explicitly and serially"]
fn canvas_vector_axis_accepts_keyboard_input_and_undo_as_one_transaction() {
    let mut context = crate::gui::tests::imgui_context();
    context.set_ini_filename(None);
    context.io_mut().display_size=[1280.,850.];context.io_mut().delta_time=1./60.;
    context.fonts().add_font(&[imgui::FontSource::DefaultFontData {config:None}]);context.fonts().build_rgba32_texture();
    let mut e=vector_editor();e.open=true;
    let pin=socket(&e,1,"value",false);
    e.asset.as_mut().unwrap().layout.positions.insert(pin.node.clone(),[250.,100.]);
    let frame=|c:&mut imgui::Context,e:&mut BlueprintEditor| {e.draw(c.frame(),&Registry::new());c.render();};
    frame(&mut context,&mut e);
    let before=bytes(e.asset.as_ref().unwrap());
    let p=CONTROLS.with(|s|s.borrow()[&format!("bp-number:{}:value:Some(1)",pin.node)]);
    context.io_mut().add_mouse_pos_event(p);frame(&mut context,&mut e);
    context.io_mut().add_mouse_button_event(MouseButton::Left,true);frame(&mut context,&mut e);
    context.io_mut().add_mouse_button_event(MouseButton::Left,false);frame(&mut context,&mut e);frame(&mut context,&mut e);
    for ch in "18.5".chars(){context.io_mut().add_input_character(ch);}frame(&mut context,&mut e);
    context.io_mut().add_key_event(imgui::Key::Enter,true);frame(&mut context,&mut e);
    context.io_mut().add_key_event(imgui::Key::Enter,false);frame(&mut context,&mut e);
    assert!(matches!(&e.current().unwrap().nodes[1].inputs["value"],Input::Literal{value,..} if *value==json!([2,18.5,4])));
    assert!(e.wire.is_none());
    e.undo();assert_eq!(before,bytes(e.asset.as_ref().unwrap()));
}
#[test]
#[ignore = "Owns a real ImGui context; run explicitly and serially"]
fn dropping_wire_on_blank_opens_filtered_menu_and_connects_selection() {
    let mut context = crate::gui::tests::imgui_context();
    context.set_ini_filename(None);
    context.io_mut().display_size = [1600.,1000.];
    context.io_mut().delta_time = 1./60.;
    context.fonts().add_font(&[imgui::FontSource::DefaultFontData {config:None}]);
    context.fonts().build_rgba32_texture();
    let frame = |c: &mut imgui::Context, e: &mut BlueprintEditor| {
        CONTROLS.with(|v| v.borrow_mut().clear());
        e.draw(c.frame(), &Registry::new()); c.render();
    };
    let mut e = editor(); e.open = true;
    e.add_node(NodeKind::Builtin {operation:asset::Builtin::GetRotation});
    let from = socket(&e,1,"value",true);
    e.asset.as_mut().unwrap().layout.positions.insert(from.node.clone(),[50.,120.]);
    frame(&mut context,&mut e);
    let before = bytes(e.asset.as_ref().unwrap());
    let drag = |c:&mut imgui::Context,e:&mut BlueprintEditor| {
        e.action_menu.context_sensitive = false;
        let a = SOCKETS.with(|s|s.borrow()[&format!("{}:value:true",from.node)]);
        c.io_mut().add_mouse_pos_event(a);frame(c,e);
        c.io_mut().add_mouse_button_event(MouseButton::Left,true);frame(c,e);
        c.io_mut().add_mouse_pos_event([a[0]+250.,a[1]+140.]);frame(c,e);
        c.io_mut().add_mouse_button_event(MouseButton::Left,false);frame(c,e);frame(c,e);
    };
    drag(&mut context,&mut e);
    assert!(CONTROLS.with(|s|s.borrow().contains_key("bp-action-context")));
    assert!(e.action_menu.context_sensitive);
    assert_eq!(before,bytes(e.asset.as_ref().unwrap()));
    let point=e.catalog_position.unwrap();
    let filtered=e.catalog_actions(&Registry::new(),true);
    assert!(filtered.iter().any(|a|matches!(a.kind,NodeKind::Builtin {operation:asset::Builtin::SetRotation})));
    assert!(!filtered.iter().any(|a|matches!(a.kind,NodeKind::Branch|NodeKind::Literal {..})));
    e.action_menu.query="set rotation".into();frame(&mut context,&mut e);
    context.io_mut().add_key_event(imgui::Key::Enter,true);frame(&mut context,&mut e);
    context.io_mut().add_key_event(imgui::Key::Enter,false);frame(&mut context,&mut e);
    let doc=e.asset.as_ref().unwrap();let node=e.current().unwrap().nodes.last().unwrap();
    assert!(matches!(node.kind,NodeKind::Builtin {operation:asset::Builtin::SetRotation}));
    assert_eq!(doc.layout.positions[&node.id],point);
    assert!(matches!(&node.inputs["value"],Input::Link {node,pin} if node==&from.node&&pin=="value"));
    assert!(e.wire.is_none());
    e.undo();assert_eq!(before,bytes(e.asset.as_ref().unwrap()));
    frame(&mut context,&mut e);drag(&mut context,&mut e);
    context.io_mut().add_key_event(imgui::Key::Escape,true);frame(&mut context,&mut e);
    context.io_mut().add_key_event(imgui::Key::Escape,false);frame(&mut context,&mut e);
    assert!(e.wire.is_none());assert_eq!(before,bytes(e.asset.as_ref().unwrap()));
}

#[test]
fn contextual_actions_support_input_output_exec_vectors_and_records() {
    let registry=Registry::new();
    for ty in [schema::Type::Vector {length:2},schema::Type::Vector {length:3},
        schema::Type::Record {cpp_name:"epok::Transform".into(),fields:vec![]}] {
        let mut e=editor();e.asset.as_mut().unwrap().functions[0].returns=ty.clone();
        e.add_node(NodeKind::Return);
        let input=socket(&e,1,"value",false);e.wire=Some(input.clone());
        let actions=e.catalog_actions(&registry,true);
        assert!(e.can_create_connected(&NodeKind::Literal {
            value_type:ty.clone(),value:default_value(&ty),
        },&input,&registry));
        if matches!(ty,schema::Type::Vector {..}) {
            assert!(actions.iter().any(|a|matches!(&a.kind,NodeKind::Literal {value_type,..} if value_type==&ty)));
        }
        assert!(actions.iter().any(|a|matches!(a.kind,NodeKind::Reroute)));
        assert!(!actions.iter().any(|a|matches!(a.kind,NodeKind::Branch)));
        assert!(actions.iter().all(|a|e.can_create_connected(&a.kind,&input,&registry)));
    }
    let mut e=editor();e.wire=Some(socket(&e,0,"next",true));
    let actions=e.catalog_actions(&registry,true);
    assert!(actions.iter().any(|a|matches!(a.kind,NodeKind::Branch)));
    assert!(!actions.iter().any(|a|matches!(a.kind,NodeKind::Literal {..})));
}

#[test]
#[ignore = "Owns a real ImGui context; run explicitly and serially"]
fn rotation_output_menu_and_drag_preview_snap_before_release() {
    let mut context = crate::gui::tests::imgui_context();
    context.set_ini_filename(None);
    context.io_mut().display_size = [1600.,1000.];
    context.io_mut().delta_time = 1. / 60.;
    context.fonts().add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
    context.fonts().build_rgba32_texture();
    let mut e = editor(); e.open = true;
    e.add_node(NodeKind::Builtin { operation: asset::Builtin::GetRotation });
    e.add_node(NodeKind::Builtin { operation: asset::Builtin::SetRotation });
    let from = socket(&e, 1, "value", true); let to = socket(&e, 2, "value", false);
    e.asset.as_mut().unwrap().layout.positions.insert(from.node.clone(), [50.,120.]);
    e.asset.as_mut().unwrap().layout.positions.insert(to.node.clone(), [310.,80.]);
    let frame = |c: &mut imgui::Context, e: &mut BlueprintEditor| {
        CONTROLS.with(|v| v.borrow_mut().clear());
        e.draw(c.frame(), &Registry::new()); c.render();
    };
    let click = |c: &mut imgui::Context, e: &mut BlueprintEditor, p, button| {
        c.io_mut().add_mouse_pos_event(p); frame(c,e);
        c.io_mut().add_mouse_button_event(button,true); frame(c,e);
        c.io_mut().add_mouse_button_event(button,false); frame(c,e);
    };
    for zoom in [0.65, 1., 1.5] {
        e.zoom = zoom; frame(&mut context,&mut e);
        let p = SOCKETS.with(|s| s.borrow()[&format!("{}:value:true", from.node)]);
        click(&mut context,&mut e,[p[0]+5.*zoom,p[1]],MouseButton::Right);
        assert!(CONTROLS.with(|s| s.borrow().contains_key("Promote to Variable")));
        let menu = CONTROLS.with(|s| s.borrow()["Split Struct Pin"]);
        click(&mut context,&mut e,menu,MouseButton::Left);
        let child = socket(&e, 1, "value.x", true);
        e.recombine_pin(&child,true).unwrap();
    }
    e.zoom = 1.; frame(&mut context,&mut e);
    let a = SOCKETS.with(|s| s.borrow()[&format!("{}:value:true", from.node)]);
    let b = SOCKETS.with(|s| s.borrow()[&format!("{}:value:false", to.node)]);
    let before = bytes(e.asset.as_ref().unwrap());
    context.io_mut().add_mouse_pos_event(a); frame(&mut context,&mut e);
    context.io_mut().add_mouse_button_event(MouseButton::Left,true); frame(&mut context,&mut e);
    context.io_mut().add_mouse_pos_event([b[0]+3.,b[1]]); frame(&mut context,&mut e);
    assert_eq!(CONTROLS.with(|s| s.borrow()["bp-connection-preview"]), b);
    assert_eq!(bytes(e.asset.as_ref().unwrap()), before, "Hover must not commit a wire");
    context.io_mut().add_mouse_button_event(MouseButton::Left,false); frame(&mut context,&mut e);
    assert!(e.wire.is_none());
    assert!(matches!(&e.current().unwrap().nodes[2].inputs["value"], Input::Link { node, pin } if node == &from.node && pin == "value"));
    e.undo(); assert_eq!(bytes(e.asset.as_ref().unwrap()), before);
    // A vector cannot light up the target ObjectRef input.
    let wrong = socket(&e, 2, "target", false);
    assert!(!e.can_connect(&from, &wrong, &Registry::new()));
}
#[test]
#[ignore = "Owns a real ImGui context; run explicitly and serially"]
fn right_click_pin_opens_split_menu_and_changes_canvas() {
    let mut context = crate::gui::tests::imgui_context();
    context.set_ini_filename(None);
    context.io_mut().display_size = [1280.,850.];
    context.io_mut().delta_time = 1. / 60.;
    context.fonts().add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
    context.fonts().build_rgba32_texture();
    let mut e = vector_editor(); e.open = true;
    let root = socket(&e, 1, "value", false);
    e.asset.as_mut().unwrap().layout.positions.insert(root.node.clone(), [300.,100.]);
    let frame = |c: &mut imgui::Context, e: &mut BlueprintEditor| { e.draw(c.frame(), &Registry::new()); c.render(); };
    let click = |c: &mut imgui::Context, e: &mut BlueprintEditor, p, button| {
        c.io_mut().add_mouse_pos_event(p); frame(c,e);
        c.io_mut().add_mouse_button_event(button,true); frame(c,e);
        c.io_mut().add_mouse_button_event(button,false); frame(c,e);
    };
    frame(&mut context,&mut e);
    let p = SOCKETS.with(|s| s.borrow()[&format!("{}:value:false", root.node)]);
    click(&mut context,&mut e,p,MouseButton::Right);
    let menu = CONTROLS.with(|s| s.borrow()["Split Struct Pin"]);
    click(&mut context,&mut e,menu,MouseButton::Left);
    frame(&mut context,&mut e);
    assert_eq!(socket(&e, 1, "value.x", false).ty, SocketType::Value(schema::Type::Fixed));
    let p = SOCKETS.with(|s| s.borrow()[&format!("{}:value.x:false", root.node)]);
    click(&mut context,&mut e,p,MouseButton::Right);
    let menu = CONTROLS.with(|s| s.borrow()["Recombine Struct Pin"]);
    click(&mut context,&mut e,menu,MouseButton::Left);
    assert_eq!(socket(&e, 1, "value", false).ty, root.ty);
}
