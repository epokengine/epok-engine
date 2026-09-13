use super::*;

#[test]
fn disconnected_islands_are_preserved_but_not_validated_or_cooked() {
    let mut file = asset();
    let mut entry = node(10, NodeKind::Entry);
    entry.outputs.insert("next".into(), vec![id(11)]);
    file.asset
        .functions
        .push(event(vec![entry, node(11, NodeKind::Return)]));
    let missing = uuid::Uuid::new_v4().to_string();
    let kinds = [
        NodeKind::Branch,
        NodeKind::Delay,
        NodeKind::Reroute,
        NodeKind::Call { function: id(5) }, // disconnected recursive call
        NodeKind::CallOn {
            class: missing.clone(),
            function: missing.clone(),
        },
        NodeKind::Builtin {
            operation: asset::Builtin::PlayTimelineAsset {
                asset: missing.clone(),
            },
        },
        NodeKind::Builtin {
            operation: asset::Builtin::SpawnParticleEffect {
                asset: missing.clone(),
            },
        },
        NodeKind::WaitPlayback {
            condition: asset::PlaybackCondition::Marker {
                timeline: missing.clone(),
                marker: missing.clone(),
            },
        },
        NodeKind::Literal {
            value_type: schema::Type::AssetRef {
                kind: "Texture".into(),
            },
            value: json!(missing),
        },
        NodeKind::Literal {
            value_type: schema::Type::Fixed,
            value: json!("unfinished"),
        },
    ];
    for (index, kind) in kinds.into_iter().enumerate() {
        file.asset.functions[0]
            .nodes
            .push(node(20 + index as u128, kind));
    }
    let before = crate::document::to_vec(&file.asset).unwrap();
    let root = std::env::temp_dir().join(format!("epok-dead-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&root).unwrap();
    let mut registry = registry();
    registry.classes.get_mut(&id(1)).unwrap().source.file = root.join("assets/scripts/Enemy.hpp");
    let result = compile(&root, &registry, &[file.clone()]);
    assert!(result.is_ok(), "{:?}", result.err());
    assert_eq!(crate::document::to_vec(&file.asset).unwrap(), before);
    assert_eq!(
        crate::blueprint_refs::graph_resources(&file.asset).count(),
        0
    );
    assert!(
        crate::blueprint_playback::references(&[file.clone()])
            .unwrap()
            .0
            .is_empty()
    );
    // The very same incomplete nodes must be diagnosed once execution reaches them.
    for target in [20, 21, 23, 24, 25, 26, 27] {
        file.asset.functions[0].nodes[0]
            .outputs
            .insert("next".into(), vec![id(target)]);
        assert!(
            compile(&root, &registry, &[file.clone()]).is_err(),
            "node {target}"
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn live_data_dependencies_and_execution_reroutes_lower_normally() {
    let mut file = asset();
    let mut entry = node(10, NodeKind::Entry);
    entry.outputs.insert("next".into(), vec![id(11)]);
    let mut exec = node(11, NodeKind::Reroute);
    exec.outputs.insert("next".into(), vec![id(12)]);
    let mut set = node(12, NodeKind::SetVariable { member: id(2) });
    set.inputs.insert(
        "value".into(),
        Input::Link {
            node: id(13),
            pin: "value".into(),
        },
    );
    let mut data = node(13, NodeKind::Reroute);
    data.inputs.insert(
        "value".into(),
        Input::Link {
            node: id(14),
            pin: "value".into(),
        },
    );
    file.asset.functions.push(event(vec![
        entry,
        exec,
        set,
        data,
        node(
            14,
            NodeKind::Literal {
                value_type: schema::Type::Fixed,
                value: json!(4),
            },
        ),
    ]));
    let compiled = compile(Path::new(""), &registry(), &[file.clone()]).unwrap();
    assert!(generated(&compiled).contains("epok::Fixed(16384,epok::Fixed::RAW)"));
    if let NodeKind::Literal { value, .. } = &mut file.asset.functions[0].nodes[4].kind {
        *value = json!("unfinished");
    }
    assert!(compile(Path::new(""), &registry(), &[file]).is_err());
}

#[test]
fn data_reachability_does_not_execute_a_disconnected_impure_producer() {
    let mut file = asset();
    let mut entry = node(10, NodeKind::Entry);
    entry.outputs.insert("next".into(), vec![id(11)]);
    let mut branch = node(11, NodeKind::Branch);
    branch.inputs.insert(
        "condition".into(),
        Input::Link {
            node: id(12),
            pin: "value".into(),
        },
    );
    let mut producer = node(
        12,
        NodeKind::Builtin {
            operation: asset::Builtin::DestroyEntity,
        },
    );
    producer.outputs.insert("next".into(), vec![id(13)]);
    file.asset.functions.push(event(vec![
        entry,
        branch,
        producer,
        node(13, NodeKind::Delay),
    ]));
    let live: Vec<_> = file.asset.functions[0]
        .compilation_nodes()
        .map(|n| n.id.clone())
        .collect();
    assert!(live.contains(&id(12)));
    assert!(!live.contains(&id(13)));
    assert!(compile(Path::new(""), &registry(), &[file]).is_err());
}

#[test]
fn default_lifecycle_events_preserve_parent_dispatch_and_are_idempotent() {
    let mut registry = registry();
    let class = registry.classes.get_mut(&id(1)).unwrap();
    class.cpp_name = "epok::Behaviour".into();
    let template = class.functions[0].clone();
    class.functions = ["start", "update", "on_trigger"]
        .into_iter()
        .enumerate()
        .map(|(index, name)| {
            let mut function = template.clone();
            function.name = name.into();
            function.id = id(50 + index as u128);
            function.abstract_method = name == "update";
            function
        })
        .collect();
    let mut file = asset();
    assert!(crate::blueprint_workflow::ensure_default_events(
        &mut file.asset,
        &registry
    ));
    assert_eq!(
        file.asset
            .functions
            .iter()
            .map(|g| g.name.as_str())
            .collect::<Vec<_>>(),
        ["start", "update", "on_trigger"]
    );
    assert_eq!(file.asset.functions[1].nodes.len(), 1);
    for graph in [&file.asset.functions[0], &file.asset.functions[2]] {
        assert!(matches!(graph.nodes[1].kind, NodeKind::CallParent));
        assert_eq!(graph.nodes[1].inputs.len(), graph.parameters.len());
    }
    let source = generated(&compile(Path::new(""), &registry, &[file.clone()]).unwrap());
    assert!(source.contains("epok::Behaviour::start("));
    assert!(source.contains("epok::Behaviour::on_trigger("));
    let before = crate::document::to_vec(&file.asset).unwrap();
    assert!(!crate::blueprint_workflow::ensure_default_events(
        &mut file.asset,
        &registry
    ));
    assert_eq!(crate::document::to_vec(&file.asset).unwrap(), before);
}
