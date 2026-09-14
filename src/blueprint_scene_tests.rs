// Scene Blueprint compilation (P6): a map's own document as a compiler input, its
// `epok::SceneScriptActor` parent rule, map-scoped reference resolution and what the
// compilation hands to P10.
//
// The registry is hand built from the stable identities of design.md section 2, like
// the P5 actor tests: libclang extraction needs the MIPS include paths and cannot run
// on every machine, and these tests are about what the compiler does with a resolved
// model rather than about extraction.
use super::*;
use crate::{actor_document as doc, object_model as om, scene::Scene};

fn native(class_id: &str, cpp_name: &str, parent: Option<&str>) -> schema::Class {
    schema::Class {
        family: None,
        domain: None,
        placement: Default::default(),
        component: None,
        default_components: vec![],
        explicit_abstract: false,
        id: class_id.into(),
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
        source: location(Path::new("runtime/object_model.hpp")),
    }
}

/// `epok::Actor` -> {`epok::SceneScriptActor` -> `GameMode`, `epok::Actor3D`},
/// alongside the legacy `Enemy` Behaviour of the shared fixture. `GameMode` is the
/// C++ scene-script base two maps in these tests share.
fn scene_registry() -> Registry {
    let mut registry = registry();
    let mut actor = native(om::ACTOR_ID, "epok::Actor", None);
    actor.family = Some(schema::ClassFamily::Actor);
    actor.domain = Some(schema::Domain::None);
    actor.abstract_class = true;
    actor.explicit_abstract = true;

    let mut script = native(
        om::SCENE_SCRIPT_ACTOR_ID,
        "epok::SceneScriptActor",
        Some(om::ACTOR_ID),
    );
    script.placement = schema::Placement {
        placeable: false,
        spawnable: false,
        scene_managed: true,
    };

    let mut mode = native(&id(70), "GameMode", Some(om::SCENE_SCRIPT_ACTOR_ID));
    mode.source = location(Path::new("assets/scripts/GameMode.hpp"));

    let mut actor3d = native(om::ACTOR3D_ID, "epok::Actor3D", Some(om::ACTOR_ID));
    actor3d.domain = Some(schema::Domain::World3D);
    actor3d.placement = schema::Placement {
        placeable: true,
        spawnable: true,
        scene_managed: false,
    };

    for class in [actor, script, mode, actor3d] {
        registry.classes.insert(class.id.clone(), class);
    }
    registry
}

/// The same registry with `GameMode` declared in a real header under `root`, which
/// is what the generated include is computed from.
fn scene_registry_at(root: &Path) -> Registry {
    let header = root.join("assets/scripts/GameMode.hpp");
    std::fs::create_dir_all(header.parent().unwrap()).unwrap();
    std::fs::write(&header, b"#pragma once\n").unwrap();
    let mut registry = scene_registry();
    registry.classes.get_mut(&id(70)).unwrap().source = location(&header);
    registry
}

/// An empty map with a scene Blueprint deriving from `parent`.
fn map(name: &str, parent_name: &str, parent_id: &str) -> Scene {
    let mut scene = Scene {
        actors: vec![],
        name: name.into(),
        ..Scene::default()
    };
    assert!(scene.ensure_scene_script(Some(&doc::ClassReference::new(parent_name, parent_id))));
    scene
}

/// Gives the map's Blueprint a stable class id so assertions can name it.
fn with_class_id(scene: &mut Scene, class: u128) -> String {
    let id = id(class);
    scene.scene_script.as_mut().unwrap().blueprint.id = id.clone();
    id
}

fn one_actor(scene: &mut Scene, actor: u128) -> uuid::Uuid {
    let identity = uuid::Uuid::from_u128(actor);
    scene.actors.push(doc::ActorInstance::new(
        identity,
        doc::ClassReference::new("epok::Actor3D", om::ACTOR3D_ID),
        "Hero",
    ));
    identity
}

/// A variable of `value_type` whose persisted default is `value`.
fn reference_variable(name: &str, value_type: schema::Type, value: serde_json::Value) -> asset::Variable {
    asset::Variable {
        id: id(80),
        name: name.into(),
        value_type,
        default: value,
        editable: true,
        timeline_animatable: false,
    }
}

fn diagnostics(result: Result<Compilation, Vec<Diagnostic>>) -> Vec<Diagnostic> {
    match result {
        Ok(_) => panic!("expected a diagnostic"),
        Err(errors) => errors,
    }
}

fn source_of(compilation: &Compilation, class: &str) -> String {
    String::from_utf8(
        compilation.artifacts.files[Path::new(&format!("scripts/generated/{class}.hpp"))].clone(),
    )
    .unwrap()
}

/// The map's own Blueprint is discovered like any other compiler input, compiles
/// with no Behaviour anywhere in the document, and is reported to P10 with the map
/// it belongs to. Its footprint carries the whole parent chain, so a change to the
/// C++ base invalidates the map's generated class.
#[test]
fn a_map_blueprint_is_discovered_compiled_and_reported_for_the_level_loader() {
    let root = crate::workspace::tests::temp("p6-discovery");
    std::fs::create_dir_all(root.join("assets/scenes")).unwrap();
    let path = root.join("assets/scenes/Level1.epokmap");
    let mut scene = map("Level1", "GameMode", &id(70));
    let class = with_class_id(&mut scene, 71);
    scene.save(&path).unwrap();

    let files = crate::blueprint_asset::load_all(&root).unwrap();
    assert_eq!(files.len(), 1, "the map's own Blueprint is a compiler input");
    assert!(files[0].is_embedded());
    assert_eq!(files[0].path, path);
    // The parent is the scene script's, not whatever the embedded asset carries.
    assert_eq!(files[0].asset.parent, id(70));
    assert_eq!(files[0].asset.name, "Level1_SceneScript");

    let compiled = compile(&root, &scene_registry_at(&root), &files).unwrap();
    assert_eq!(compiled.scene_scripts, vec![(path.clone(), class.clone())]);
    let source = source_of(&compiled, &class);
    assert!(
        source.contains("class Level1_SceneScript : public GameMode"),
        "{source}"
    );
    // Transitive parents are dependencies of the map's class: the existing footprint
    // mechanism walks the ancestry, so a C++ or Blueprint base change invalidates it.
    let footprint = &compiled.footprints[&format!("generated-blueprint:{class}")].1;
    for ancestor in [&id(70), om::SCENE_SCRIPT_ACTOR_ID, om::ACTOR_ID] {
        assert!(
            footprint.contains_key(&format!("blueprint-class:{ancestor}")),
            "{ancestor} missing from {footprint:?}"
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}

/// Two maps may share one C++ `SceneScriptActor` base. Each keeps its own class.
#[test]
fn two_maps_sharing_a_cpp_scene_script_parent_both_compile() {
    let root = crate::workspace::tests::temp("p6-two-maps");
    std::fs::create_dir_all(root.join("assets/scenes")).unwrap();
    for (name, class) in [("Town", 72u128), ("Cave", 73)] {
        let mut scene = map(name, "GameMode", &id(70));
        with_class_id(&mut scene, class);
        scene
            .save(&root.join(format!("assets/scenes/{name}.epokmap")))
            .unwrap();
    }
    let files = crate::blueprint_asset::load_all(&root).unwrap();
    assert_eq!(files.len(), 2);
    let compiled = compile(&root, &scene_registry_at(&root), &files).unwrap();
    assert_eq!(
        compiled
            .scene_scripts
            .iter()
            .map(|(map, class)| (
                map.file_name().unwrap().to_string_lossy().into_owned(),
                class.clone()
            ))
            .collect::<Vec<_>>(),
        vec![
            ("Cave.epokmap".to_string(), id(73)),
            ("Town.epokmap".to_string(), id(72))
        ]
    );
    assert!(source_of(&compiled, &id(72)).contains("class Town_SceneScript : public GameMode"));
    assert!(source_of(&compiled, &id(73)).contains("class Cave_SceneScript : public GameMode"));
    std::fs::remove_dir_all(root).unwrap();
}

/// The scene Blueprint's parent is chosen in Map Settings, so the map is what the
/// diagnostic points at when the chosen class is not a `SceneScriptActor`.
#[test]
fn a_scene_blueprint_that_does_not_derive_from_scene_script_actor_is_refused() {
    let mut scene = map("Broken", "epok::Actor3D", om::ACTOR3D_ID);
    with_class_id(&mut scene, 74);
    let path = PathBuf::from("assets/scenes/Broken.epokmap");
    let file = crate::blueprint_asset::embedded(&path, &scene).unwrap();
    let errors = diagnostics(compile(Path::new(""), &scene_registry(), &[file]));
    assert_eq!(errors[0].asset, path);
    assert!(
        errors[0]
            .message
            .contains("not an epok::SceneScriptActor subclass"),
        "{}",
        errors[0].message
    );
}

/// A map's Blueprint may name the actors and actors of its own map by UUID. The
/// compiler resolves them here, by identity and scope, and hands the bindings to the
/// Level loader; the generated constructor keeps the null identity, exactly as every
/// other typed object reference does. Nothing looks an identity up by name at Tick.
#[test]
fn map_scoped_references_resolve_by_uuid_and_scope() {
    let path = PathBuf::from("assets/scenes/Level1.epokmap");
    let mut scene = map("Level1", "epok::SceneScriptActor", om::SCENE_SCRIPT_ACTOR_ID);
    let class = with_class_id(&mut scene, 75);
    let actor = one_actor(&mut scene, 76);
    let entity = scene.actors.first().map(|e| e.id);
    assert_eq!(entity, Some(actor), "one Actor collection backs scene references");
    scene
        .scene_script
        .as_mut()
        .unwrap()
        .blueprint
        .variables
        .push(reference_variable(
            "hero",
            schema::Type::ActorRef { class: None },
            json!(actor.to_string()),
        ));

    let file = crate::blueprint_asset::embedded(&path, &scene).unwrap();
    let compiled = compile(Path::new(""), &scene_registry(), &[file]).unwrap();
    assert_eq!(compiled.scene_references.len(), 1);
    let reference = &compiled.scene_references[0];
    assert_eq!(reference.map, path);
    assert_eq!(reference.class_id, class);
    assert_eq!(reference.member, "property:hero");
    assert_eq!(reference.kind, SceneRefKind::Actor);
    assert_eq!(reference.target, actor);
    let source = source_of(&compiled, &class);
    assert!(
        source.contains("epok::ObjectId hero = epok::ObjectId{};"),
        "{source}"
    );
    assert!(
        !source.contains(&actor.to_string()),
        "a persisted handle is never written into generated code: {source}"
    );

    // An actor of another map is refused by identity, and the diagnostic names it.
    let mut stranger = scene.clone();
    let outside = uuid::Uuid::from_u128(999);
    stranger
        .scene_script
        .as_mut()
        .unwrap()
        .blueprint
        .variables[0]
        .default = json!(outside.to_string());
    let file = crate::blueprint_asset::embedded(&path, &stranger).unwrap();
    let errors = diagnostics(compile(Path::new(""), &scene_registry(), &[file]));
    assert!(
        errors[0].message.contains(&outside.to_string())
            && errors[0].message.contains("not an actor of"),
        "{}",
        errors[0].message
    );
}

/// A standalone Blueprint has no map, so a map-scoped reference in one would mean
/// nothing at run time. It is refused rather than silently nulled.
#[test]
fn a_standalone_blueprint_refuses_map_scoped_references() {
    let mut file = asset();
    file.asset.parent = id(1);
    file.asset.variables.push(reference_variable(
        "hero",
        schema::Type::ActorRef { class: None },
        json!(uuid::Uuid::from_u128(76).to_string()),
    ));
    let errors = diagnostics(compile(Path::new(""), &registry(), &[file]));
    assert!(
        errors[0]
            .message
            .contains("only available inside a map's scene Blueprint"),
        "{}",
        errors[0].message
    );
}

/// A reference inside a graph is code, not persisted data: it has no per-instance
/// slot for the loader to bind, so it is refused with the node it came from.
#[test]
fn a_graph_literal_cannot_carry_a_map_identity() {
    let path = PathBuf::from("assets/scenes/Level1.epokmap");
    let mut scene = map("Level1", "epok::SceneScriptActor", om::SCENE_SCRIPT_ACTOR_ID);
    with_class_id(&mut scene, 77);
    let actor = one_actor(&mut scene, 78);
    let blueprint = &mut scene.scene_script.as_mut().unwrap().blueprint;
    blueprint.functions.push(Graph {
        id: id(90),
        name: "helper".into(),
        override_id: None,
        timeline: None,
        parameters: vec![],
        returns: schema::Type::Void,
        entry: id(91),
        nodes: vec![
            Node {
                id: id(91),
                kind: NodeKind::Entry,
                inputs: Default::default(),
                outputs: Default::default(),
            },
            Node {
                id: id(92),
                kind: NodeKind::Literal {
                    value_type: schema::Type::ActorRef { class: None },
                    value: json!(actor.to_string()),
                },
                inputs: Default::default(),
                outputs: Default::default(),
            },
        ],
    });
    blueprint.functions[0].nodes[0]
        .outputs
        .insert("exec".into(), vec![id(92)]);
    let file = crate::blueprint_asset::embedded(&path, &scene).unwrap();
    let errors = diagnostics(compile(Path::new(""), &scene_registry(), &[file]));
    assert_eq!(errors[0].node.as_deref(), Some(id(92).as_str()));
    assert!(
        errors[0].message.contains("promote it to a variable"),
        "{}",
        errors[0].message
    );
}

/// Staleness: the embedded document publishes `blueprint:{id}` like a file asset,
/// so a build that consumed the map's class is invalidated when the graph changes.
#[test]
fn observation_publishes_the_embedded_document_like_any_other_source() {
    let root = crate::workspace::tests::temp("p6-observation");
    std::fs::create_dir_all(root.join("assets/scenes")).unwrap();
    let path = root.join("assets/scenes/Level1.epokmap");
    let mut scene = map("Level1", "epok::SceneScriptActor", om::SCENE_SCRIPT_ACTOR_ID);
    let class = with_class_id(&mut scene, 79);
    scene.save(&path).unwrap();

    let files = crate::blueprint_asset::load_all(&root).unwrap();
    crate::blueprint_dependencies::observe_sources(&root, &files, None).unwrap();
    let graph = crate::artifact_dependencies::Graph::load(&root).unwrap();
    let before = graph.nodes[&format!("blueprint:{class}")].signature.clone();

    scene
        .scene_script
        .as_mut()
        .unwrap()
        .blueprint
        .variables
        .push(reference_variable("health", schema::Type::Fixed, json!(3)));
    scene.save(&path).unwrap();
    let files = crate::blueprint_asset::load_all(&root).unwrap();
    crate::blueprint_dependencies::observe_sources(&root, &files, None).unwrap();
    let graph = crate::artifact_dependencies::Graph::load(&root).unwrap();
    assert_ne!(
        graph.nodes[&format!("blueprint:{class}")].signature,
        before,
        "editing the map's Blueprint must make its class stale"
    );
    std::fs::remove_dir_all(root).unwrap();
}
