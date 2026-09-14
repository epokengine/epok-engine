// Actor- and Component-family Blueprint compilation (P5).
//
// The registry here is hand built rather than extracted: libclang cannot run on every
// machine, and these tests are about what the compiler does with a resolved family, not
// about extraction. The ids are the stable native identities of design.md section 2, so
// `object_model::Model` recognises the family roots exactly as it does in a real project.
use super::*;
use crate::object_model as om;

fn location_of(path: &str) -> schema::Location {
    location(Path::new(path))
}

fn event_function(id: &str, name: &str, parameters: Vec<schema::Parameter>) -> schema::Function {
    schema::Function {
        id: id.into(),
        name: name.into(),
        parameters,
        returns: schema::Type::Void,
        callable: false,
        timeline: None,
        event: true,
        pure: false,
        abstract_method: false,
        final_method: false,
        access: "public".into(),
        overrides: vec![],
        source: location_of("runtime/object_model.hpp"),
    }
}

fn native_class(id: &str, cpp_name: &str, parent: Option<&str>) -> schema::Class {
    schema::Class {
        family: None,
        domain: None,
        placement: Default::default(),
        component: None,
        default_components: vec![],
        explicit_abstract: false,
        id: id.into(),
        provider: schema::native_provider(),
        backend: schema::native_backend(),
        cpp_name: cpp_name.into(),
        parent: parent.map(str::to_owned),
        abstract_class: false,
        final_class: false,
        timeline_component: None,
        blueprintable: true,
        properties: vec![],
        functions: vec![],
        source: location_of("runtime/object_model.hpp"),
    }
}

/// Stable ids for the five reflected lifecycle events. Real extraction derives them from
/// the declaration site; the values only have to be stable and distinct.
fn event_id(name: &str) -> String {
    let index = ["begin_play", "tick", "end_play", "on_enable", "on_disable"]
        .iter()
        .position(|candidate| *candidate == name)
        .expect("reflected lifecycle event");
    id(900 + index as u128)
}

fn lifecycle_events() -> Vec<schema::Function> {
    vec![
        event_function(&event_id("begin_play"), "begin_play", vec![]),
        event_function(
            &event_id("tick"),
            "tick",
            vec![schema::Parameter {
                name: "dt".into(),
                value_type: schema::Type::Fixed,
                direction: schema::Direction::Value,
            }],
        ),
        event_function(
            &event_id("end_play"),
            "end_play",
            vec![schema::Parameter {
                name: "reason".into(),
                value_type: schema::Type::Enum {
                    cpp_name: "epok::EndPlayReason".into(),
                    variants: Default::default(),
                },
                direction: schema::Direction::Value,
            }],
        ),
        event_function(&event_id("on_enable"), "on_enable", vec![]),
        event_function(&event_id("on_disable"), "on_disable", vec![]),
    ]
}

/// `epok::Actor` -> `epok::Actor3D` plus `epok::ActorComponent` -> `epok::AudioComponent`,
/// alongside the legacy `Enemy` Behaviour of the shared fixture.
fn actor_registry() -> Registry {
    let mut registry = registry();

    let mut actor = native_class(om::ACTOR_ID, "epok::Actor", None);
    actor.family = Some(schema::ClassFamily::Actor);
    actor.domain = Some(schema::Domain::None);
    actor.abstract_class = true;
    actor.explicit_abstract = true;
    actor.functions = lifecycle_events();

    let mut actor3d = native_class(om::ACTOR3D_ID, "epok::Actor3D", Some(om::ACTOR_ID));
    actor3d.domain = Some(schema::Domain::World3D);
    actor3d.placement = schema::Placement {
        placeable: true,
        spawnable: true,
        scene_managed: false,
    };

    let mut component = native_class(om::ACTOR_COMPONENT_ID, "epok::ActorComponent", None);
    component.family = Some(schema::ClassFamily::Component);
    component.abstract_class = true;
    component.explicit_abstract = true;
    component.functions = lifecycle_events();

    let mut audio = native_class(
        om::AUDIO_COMPONENT_ID,
        "epok::AudioComponent",
        Some(om::ACTOR_COMPONENT_ID),
    );
    audio.component = Some(schema::ComponentContract {
        owners: [
            schema::Domain::World3D,
            schema::Domain::World2D,
            schema::Domain::UI,
        ]
        .into_iter()
        .collect(),
        requires: vec![],
        excludes: vec![],
        cardinality: schema::Cardinality::Multiple,
        can_root: false,
        capabilities: ["audio".to_string()].into_iter().collect(),
    });

    for class in [actor, actor3d, component, audio] {
        registry.classes.insert(class.id.clone(), class);
    }
    registry
}

fn blueprint(class: u128, name: &str, parent: &str) -> AssetFile {
    let mut a = asset::BlueprintAsset::new(name.into(), parent.into());
    a.id = id(class);
    AssetFile::file(PathBuf::from(format!("assets/blueprints/{name}.epokbp")), a)
}

fn graph(id_n: u128, name: &str, override_id: &str, parameters: Vec<schema::Parameter>) -> Graph {
    Graph {
        timeline: None,
        id: id(id_n),
        name: name.into(),
        override_id: Some(override_id.into()),
        parameters,
        returns: schema::Type::Void,
        entry: id(id_n + 1),
        nodes: vec![node(id_n + 1, NodeKind::Entry)],
    }
}

/// `Compilation` is not `Debug`, so `unwrap_err` is unavailable; the diagnostics are
/// what these tests assert on.
fn errors(result: Result<Compilation, Vec<Diagnostic>>) -> Vec<Diagnostic> {
    match result {
        Ok(_) => panic!("expected a diagnostic"),
        Err(diagnostics) => diagnostics,
    }
}

fn text_of(compiled: &Compilation, class: u128) -> String {
    String::from_utf8(
        compiled.artifacts.files[&PathBuf::from(format!("scripts/generated/{}.hpp", id(class)))]
            .clone(),
    )
    .unwrap()
}

#[test]
fn actor_blueprints_derive_across_levels_and_override_each_event_once() {
    let registry = actor_registry();
    let mut goblin = blueprint(20, "BP_Goblin", om::ACTOR3D_ID);
    let mut tick = graph(
        30,
        "tick",
        &event_id("tick"),
        vec![schema::Parameter {
            name: "dt".into(),
            value_type: schema::Type::Fixed,
            direction: schema::Direction::Value,
        }],
    );
    tick.nodes[0].outputs.insert("next".into(), vec![id(32)]);
    let mut call_parent = node(32, NodeKind::CallParent);
    call_parent
        .inputs
        .insert("dt".into(), Input::Parameter { name: "dt".into() });
    tick.nodes.push(call_parent);
    goblin.asset.functions.push(tick);
    goblin
        .asset
        .functions
        .push(graph(40, "begin_play", &event_id("begin_play"), vec![]));

    let mut fire = blueprint(21, "BP_GoblinFire", &id(20));
    fire.asset
        .functions
        .push(graph(50, "begin_play", &event_id("begin_play"), vec![]));

    let compiled = compile(Path::new(""), &registry, &[goblin, fire]).unwrap();
    let goblin_text = text_of(&compiled, 20);
    let fire_text = text_of(&compiled, 21);

    assert!(goblin_text.contains("class BP_Goblin : public epok::Actor3D"));
    assert!(fire_text.contains("class BP_GoblinFire : public BP_Goblin"));
    // The runtime bases live behind the umbrella header, never in assets/scripts.
    assert!(goblin_text.contains("#include \"epok.hpp\""));
    // Each reflected event is overridden exactly once, at the level that authored it.
    assert_eq!(goblin_text.matches("void tick(epok::Fixed dt) override").count(), 1);
    assert_eq!(goblin_text.matches("void begin_play() override").count(), 1);
    assert_eq!(fire_text.matches("void begin_play() override").count(), 1);
    assert!(!fire_text.contains("void tick("));
    // Call Parent emits one qualified dispatch to the immediate parent.
    assert_eq!(goblin_text.matches("epok::Actor3D::tick(").count(), 1);
    // Self is the Actor: the legacy slot is reached null-safely, never dereferenced blind.
    assert!(goblin_text.contains("const auto epok_owner=this->id();"));
    assert!(!goblin_text.contains("&this->entity()"));
    // Behaviour-only hooks stay out of an Actor class.
    assert!(!goblin_text.contains("blueprint_class_id"));
    assert!(!goblin_text.contains("blueprint_tick"));
}

#[test]
fn actor_blueprint_latent_graphs_pump_tasks_from_tick_and_cancel_in_end_play() {
    let registry = actor_registry();
    let mut goblin = blueprint(20, "BP_Goblin", om::ACTOR3D_ID);
    let mut begin = graph(30, "begin_play", &event_id("begin_play"), vec![]);
    begin.nodes[0].outputs.insert("next".into(), vec![id(32)]);
    let mut delay = node(32, NodeKind::Delay);
    delay.inputs.insert(
        "seconds".into(),
        Input::Literal {
            value_type: schema::Type::Fixed,
            value: serde_json::json!(1),
        },
    );
    begin.nodes.push(delay);
    goblin.asset.functions.push(begin);

    let text = text_of(
        &compile(Path::new(""), &registry, &[goblin]).unwrap(),
        20,
    );
    assert!(text.contains("epok::bp::Continuations<8> epok_tasks;"));
    // No authored tick graph: the synthesized override still dispatches the parent.
    assert_eq!(text.matches("void tick(epok::Fixed dt) override").count(), 1);
    assert!(text.contains("epok_tasks.advance(dt,epok::blueprint_scene_generation);"));
    assert!(text.contains("epok::Actor3D::tick(dt);"));
    assert_eq!(
        text.matches("void end_play(epok::EndPlayReason reason) override")
            .count(),
        1
    );
    assert!(text.contains("epok_tasks.cancel_all();"));
    assert!(text.contains("epok::Actor3D::end_play(reason);"));
    // Latent frames wait on the actor's own handle, resolved through the adapter.
    assert!(text.contains("this->id()"));
}

#[test]
fn authored_tick_graph_keeps_its_body_behind_the_synthesized_pump() {
    let registry = actor_registry();
    let mut goblin = blueprint(20, "BP_Goblin", om::ACTOR3D_ID);
    let mut tick = graph(
        30,
        "tick",
        &event_id("tick"),
        vec![schema::Parameter {
            name: "dt".into(),
            value_type: schema::Type::Fixed,
            direction: schema::Direction::Value,
        }],
    );
    tick.nodes[0].outputs.insert("next".into(), vec![id(32)]);
    let mut delay = node(32, NodeKind::Delay);
    delay.inputs.insert(
        "seconds".into(),
        Input::Literal {
            value_type: schema::Type::Fixed,
            value: serde_json::json!(1),
        },
    );
    tick.nodes.push(delay);
    goblin.asset.functions.push(tick);

    let text = text_of(&compile(Path::new(""), &registry, &[goblin]).unwrap(), 20);
    // Exactly one `tick` override; the authored graph moves behind a private name so the
    // pump and the user body cannot both claim the reflected event.
    assert_eq!(text.matches("void tick(epok::Fixed dt) override").count(), 1);
    assert_eq!(text.matches("void epok_graph_tick(epok::Fixed dt)").count(), 1);
    assert!(text.contains("this->epok_graph_tick(dt);"));
    // The parent is dispatched by the authored graph (or not at all), never twice.
    assert!(!text.contains("epok::Actor3D::tick(dt);"));
}

#[test]
fn component_blueprints_resolve_their_owner_through_a_typed_actor_reference() {
    let registry = actor_registry();
    let mut bp = blueprint(22, "BP_Siren", om::AUDIO_COMPONENT_ID);
    bp.asset.variables.push(asset::Variable {
        id: id(60),
        name: "owner".into(),
        value_type: schema::Type::ActorRef { class: None },
        default: serde_json::json!(null),
        editable: true,
        timeline_animatable: false,
    });
    let mut begin = graph(30, "begin_play", &event_id("begin_play"), vec![]);
    begin.nodes[0].outputs.insert("next".into(), vec![id(33)]);
    let mut set = node(
        33,
        NodeKind::SetVariable {
            member: id(60).to_string(),
        },
    );
    set.inputs.insert(
        "value".into(),
        Input::Link {
            node: id(32),
            pin: "value".into(),
        },
    );
    begin.nodes.push(set);
    begin.nodes.push(node(
        32,
        NodeKind::Builtin {
            operation: asset::Builtin::GetOwner,
        },
    ));
    bp.asset.functions.push(begin);

    let text = text_of(&compile(Path::new(""), &registry, &[bp.clone()]).unwrap(), 22);
    assert!(text.contains("class BP_Siren : public epok::AudioComponent"));
    // Typed object references share one ABI word: a compact generational ObjectId.
    assert!(text.contains("epok::ObjectId owner"));
    assert!(text.contains("epok::bp::component_owner_id(this)"));

    // The same node in a legacy Behaviour Blueprint has no owner to resolve.
    let mut behaviour = blueprint(24, "BP_Legacy", &id(1));
    let mut damage = event(vec![]);
    damage.nodes = vec![node(10, NodeKind::Entry)];
    damage.nodes[0].outputs.insert("next".into(), vec![id(33)]);
    let mut set = node(
        33,
        NodeKind::SetVariable {
            member: id(60).to_string(),
        },
    );
    set.inputs.insert(
        "value".into(),
        Input::Link {
            node: id(32),
            pin: "value".into(),
        },
    );
    damage.nodes.push(set);
    damage.nodes.push(node(
        32,
        NodeKind::Builtin {
            operation: asset::Builtin::GetOwner,
        },
    ));
    behaviour.asset.variables = bp.asset.variables.clone();
    behaviour.asset.functions.push(damage);
    let message = errors(compile(Path::new(""), &registry, &[behaviour]))[0].to_string();
    assert!(
        message.contains("only available inside a Component Blueprint"),
        "{message}"
    );
}

#[test]
fn spawn_actor_requires_an_actor_class_reference() {
    let registry = actor_registry();
    let mut spawner = blueprint(23, "BP_Spawner", om::ACTOR3D_ID);
    let make = |base: &str| {
        let mut begin = graph(30, "begin_play", &event_id("begin_play"), vec![]);
        begin.nodes[0].outputs.insert("next".into(), vec![id(32)]);
        let mut spawn = node(
            32,
            NodeKind::Builtin {
                operation: asset::Builtin::SpawnActor { base: base.into() },
            },
        );
        spawn.inputs.insert(
            "class".into(),
            Input::Literal {
                value_type: schema::Type::ClassRef { base: base.into() },
                value: serde_json::json!(base),
            },
        );
        spawn.inputs.insert(
            "parent".into(),
            Input::Literal {
                value_type: schema::Type::ActorRef { class: None },
                value: serde_json::json!(null),
            },
        );
        begin.nodes.push(spawn);
        begin
    };

    spawner.asset.functions.push(make(om::ACTOR3D_ID));
    let text = text_of(
        &compile(Path::new(""), &registry, &[spawner.clone()]).unwrap(),
        23,
    );
    assert!(text.contains("epok::bp::spawn_actor(this,"));

    // A Behaviour class reference is not an Actor and cannot be spawned as one.
    spawner.asset.functions = vec![make(om::AUDIO_COMPONENT_ID)];
    let message = errors(compile(Path::new(""), &registry, &[spawner]))[0].to_string();
    assert!(message.contains("Spawn Actor requires an Actor class"), "{message}");
    assert!(message.contains("Component family"), "{message}");
}

#[test]
fn the_family_hint_never_overrides_the_resolved_parent_chain() {
    let registry = actor_registry();
    let mut goblin = blueprint(20, "BP_Goblin", om::ACTOR3D_ID);
    goblin.asset.family = Some(schema::ClassFamily::Actor);
    assert!(compile(Path::new(""), &registry, &[goblin.clone()]).is_ok());

    goblin.asset.family = Some(schema::ClassFamily::Component);
    let message = errors(compile(Path::new(""), &registry, &[goblin]))[0].to_string();
    assert!(message.contains("declares the Component family"), "{message}");
    assert!(message.contains("resolves to Actor"), "{message}");
}

#[test]
fn reparenting_an_actor_blueprint_onto_a_component_is_rejected_by_the_model() {
    let registry = actor_registry();
    let goblin = blueprint(20, "BP_Goblin", om::ACTOR3D_ID);
    let compiled = compile(Path::new(""), &registry, &[goblin]).unwrap();
    let model = compiled.registry.model().unwrap();

    // Same family, still blueprintable: allowed.
    assert!(model.validate_reparent(&id(20), om::ACTOR3D_ID).is_ok());
    // Crossing from Actor to Component is never a reparent, it is a different class.
    let diagnostics = model
        .validate_reparent(&id(20), om::AUDIO_COMPONENT_ID)
        .unwrap_err();
    assert!(diagnostics.iter().any(|d| d.code == "family-crossing"));
    // The abstract root is still a legal parent; the cook, not the model, rejects
    // instantiating it.
    assert!(model.validate_reparent(&id(20), om::ACTOR_ID).is_ok());
}
