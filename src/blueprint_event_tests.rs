use super::*;

fn lifecycle_fixture() -> (BlueprintEditor, Registry) {
    let mut registry = crate::actor_document::tests::registry();
    let class = registry
        .classes
        .get_mut(crate::object_model::ACTOR_COMPONENT_ID)
        .unwrap();
    class.functions = ["begin_play", "tick", "end_play"].into_iter().map(|name| {
        let parameters = match name {
            "tick" => vec![json!({"name": "delta_seconds", "value_type": {"kind": "fixed"}, "direction": "value"})],
            "end_play" => vec![json!({"name": "end_play_reason", "value_type": {"kind": "enum", "cpp_name": "epok::EndPlayReason", "variants": {"Destroyed": 0, "LevelUnloaded": 1, "Quit": 2}}, "direction": "value"})],
            _ => vec![],
        };
        serde_json::from_value(json!({
            "id": id(), "name": name, "parameters": parameters,
            "returns": {"kind": "void"}, "callable": false, "event": true,
            "pure": false, "abstract_method": false, "final_method": false,
            "access": "public", "overrides": [],
            "source": {"file": "object_model.hpp", "line": 1, "column": 1}
        })).unwrap()
    }).collect();
    let mut doc = BlueprintAsset::new("BP_AC_Rotate".into(), class.id.clone());
    assert!(crate::blueprint_workflow::ensure_default_events(
        &mut doc, &registry
    ));
    // Exercise the exact saved pin keys from the user's previous editor build.
    for graph in &mut doc.functions {
        for parameter in &mut graph.parameters {
            parameter.name = "arg0".into();
        }
    }
    (
        BlueprintEditor {
            open: true,
            asset: Some(doc),
            ..Default::default()
        },
        registry,
    )
}

#[test]
fn lifecycle_outputs_parent_identity_and_explicit_parent_call_are_unambiguous() {
    let (mut editor, registry) = lifecycle_fixture();
    let doc = editor.asset.as_ref().unwrap();
    assert_eq!(parent_class_label(doc, &registry), "ActorComponent");
    for (graph, expected) in
        doc.functions
            .iter()
            .zip([None, Some("Delta Seconds"), Some("End Play Reason")])
    {
        assert!(graph.inherits_event());
        assert_eq!(graph.nodes.len(), 1);
        let sockets = node_sockets(doc, graph, &graph.nodes[0], &registry);
        assert!(sockets.iter().all(|s| s.output));
        assert_eq!(
            sockets
                .iter()
                .find(|s| matches!(s.ty, SocketType::Value(_)))
                .map(|s| s.label.as_str()),
            expected
        );
    }
    let entry = doc.functions[2].entry.clone();
    let before = doc.clone();
    editor.add_parent_call(&entry, &registry);
    assert_eq!(
        editor.graph, 2,
        "The clicked event, not the previously selected graph, owns the call"
    );
    let doc = editor.asset.as_ref().unwrap();
    let graph = editor.current().unwrap();
    let call = &graph.nodes[1];
    assert_eq!(node_label(call, doc, &registry), "Parent: End Play");
    assert!(
        graph.inherits_event(),
        "An unconnected parent node does not implement an override"
    );
    let input = node_sockets(doc, graph, call, &registry)
        .into_iter()
        .find(|s| s.pin == "arg0")
        .unwrap();
    assert!(!input.output);
    assert_eq!(input.label, "End Play Reason");
    editor.checkpoint(before);
    editor.undo();
    assert_eq!(editor.current().unwrap().nodes.len(), 1);
    editor.redo();
    assert_eq!(editor.current().unwrap().nodes.len(), 2);
}

#[test]
#[ignore = "Owns a real ImGui context; run explicitly and serially"]
fn event_context_menu_adds_the_clicked_parent_call_and_undoes() {
    let (mut editor, registry) = lifecycle_fixture();
    let mut ctx = crate::gui::tests::imgui_context();
    ctx.io_mut().display_size = [1400., 1000.];
    ctx.io_mut().delta_time = 1. / 60.;
    ctx.fonts()
        .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
    ctx.fonts().build_rgba32_texture();
    let frame = |ctx: &mut imgui::Context, editor: &mut BlueprintEditor| {
        editor.draw(ctx.frame(), &registry);
        ctx.render();
    };
    let click = |ctx: &mut imgui::Context, editor: &mut BlueprintEditor, point: [f32; 2], mouse| {
        ctx.io_mut().add_mouse_pos_event(point);
        frame(ctx, editor);
        ctx.io_mut().add_mouse_button_event(mouse, true);
        frame(ctx, editor);
        ctx.io_mut().add_mouse_button_event(mouse, false);
        frame(ctx, editor);
    };
    frame(&mut ctx, &mut editor);
    frame(&mut ctx, &mut editor);
    let entry = editor.asset.as_ref().unwrap().functions[1].entry.clone();
    let header = CONTROLS.with(|c| c.borrow()[&format!("bp-node-header:{entry}")]);
    click(&mut ctx, &mut editor, header, MouseButton::Right);
    let command = CONTROLS.with(|c| c.borrow()["Add Call to Parent Function"]);
    click(&mut ctx, &mut editor, command, MouseButton::Left);
    assert_eq!(editor.graph, 1);
    assert_eq!(editor.current().unwrap().nodes.len(), 2);
    assert_eq!(
        node_label(
            &editor.current().unwrap().nodes[1],
            editor.asset.as_ref().unwrap(),
            &registry
        ),
        "Parent: Tick"
    );
    assert!(editor.current().unwrap().inherits_event());
    editor.undo();
    assert_eq!(editor.current().unwrap().nodes.len(), 1);
}
