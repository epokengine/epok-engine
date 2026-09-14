//! The engine's own Actor/ActorComponent operations are reflected members, not private
//! builtins of one authoring provider.
//!
//! `runtime/object_model.hpp` annotates `active`, `set_active`, `destroy`, `wants_tick`,
//! `set_wants_tick`, the identity/hierarchy readers and `ActorComponent::owner_id`. Every
//! provider therefore inherits them: C++ calls the member directly, a Blueprint drops a
//! Call node, and a Lua class writes `self:set_active(false)`. These tests prove that on a
//! real extracted project rather than on a hand-built registry, because the claim is about
//! what the Clang extractor produces from the shipped runtime header.
use crate::{
    blueprint_asset::{AssetFile, BlueprintAsset, Graph, Node, NodeKind},
    object_model as om, reflection_schema as schema,
};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

/// Reflected identities of the runtime operations under test. They are pinned here so a
/// silent renumbering of `Id=` in the header fails loudly instead of dropping the node
/// from every saved graph that references it.
const ACTIVE: &str = "66f0ed92-0e29-498a-aff0-1102dd0c9783";
const SET_ACTIVE: &str = "6eb5a977-358e-45a2-a8ab-317646d124f2";
const DESTROY: &str = "cf849e67-22a1-4838-b593-774689e03195";
const WANTS_TICK: &str = "9771750e-0866-439b-acdc-7c76c271e46e";
const SET_WANTS_TICK: &str = "ffe58520-ad64-4c77-a2ce-85c0d3ba18dc";
const LEVEL_ID: &str = "8e2697a0-4267-4923-808b-62acb68d5f3e";
const ROOT_ID: &str = "d8217460-caff-411c-b7bc-f0ebb34b1c83";
const LOGICAL_PARENT: &str = "146a6de0-328f-4273-bc7e-ba379ff26ad7";
const COMPONENT_ID: &str = "3538d7f8-8886-4437-a7ab-41afa18aad93";
const COMPONENT_COUNT: &str = "3092c8e4-43ff-47f5-ab0c-290ba4685978";
const OWNER_ID: &str = "845721eb-1ae9-49f4-b207-607875ce46ef";

fn project(label: &str) -> PathBuf {
    let root = crate::workspace::tests::temp(label);
    crate::workspace::create(&root, "Runtime API", crate::workspace::Template::Basic).unwrap();
    root
}

fn function<'a>(class: &'a schema::Class, name: &str) -> &'a schema::Function {
    class
        .functions
        .iter()
        .find(|f| f.name == name)
        .unwrap_or_else(|| {
            panic!(
                "{} does not reflect {name}: {:?}",
                class.cpp_name,
                class.functions.iter().map(|f| &f.name).collect::<Vec<_>>()
            )
        })
}

#[test]
#[ignore = "requires the pinned host extractor; stages a disposable project without building"]
fn runtime_actor_operations_are_reflected_on_the_native_bases() {
    let root = project("runtime-api-registry");
    let catalog = crate::scripts::catalog(&root).unwrap();
    let registry = crate::blueprint::native_registry(&root, &catalog).unwrap();

    let actor = registry.classes.get(om::ACTOR_ID).expect("epok::Actor");
    assert_eq!(actor.cpp_name, "epok::Actor");

    // Activation and destruction: callable, impure, void, and reached by every provider.
    for (name, id) in [
        ("set_active", SET_ACTIVE),
        ("destroy", DESTROY),
        ("set_wants_tick", SET_WANTS_TICK),
    ] {
        let f = function(actor, name);
        assert_eq!(f.id, id, "{name} identity drifted");
        assert!(f.callable && !f.pure && !f.event, "{name} is not callable");
        assert_eq!(f.returns, schema::Type::Void, "{name} returns");
        assert_eq!(f.access, "public");
    }
    assert_eq!(
        function(actor, "set_active").parameters[0].value_type,
        schema::Type::Bool
    );
    assert!(function(actor, "destroy").parameters.is_empty());
    assert_eq!(
        function(actor, "set_wants_tick").parameters[0].value_type,
        schema::Type::Bool
    );

    // Pure readers. `Pure` implies callable, so a data-only graph can read them.
    for (name, id, returns) in [
        ("active", ACTIVE, schema::Type::Bool),
        ("wants_tick", WANTS_TICK, schema::Type::Bool),
        ("component_count", COMPONENT_COUNT, schema::Type::UInt32),
        (
            "level_id",
            LEVEL_ID,
            schema::Type::ObjectRef { class: None },
        ),
        ("root_id", ROOT_ID, schema::Type::ObjectRef { class: None }),
        (
            "logical_parent",
            LOGICAL_PARENT,
            schema::Type::ObjectRef { class: None },
        ),
        (
            "component_id",
            COMPONENT_ID,
            schema::Type::ObjectRef { class: None },
        ),
    ] {
        let f = function(actor, name);
        assert_eq!(f.id, id, "{name} identity drifted");
        assert!(f.pure && f.callable && !f.event, "{name} is not pure");
        assert_eq!(f.returns, returns, "{name} returns");
    }
    assert_eq!(
        function(actor, "component_id").parameters[0].value_type,
        schema::Type::UInt32
    );

    // The component side of the same contract: Get Owner as a reflected member.
    let component = registry
        .classes
        .get(om::ACTOR_COMPONENT_ID)
        .expect("epok::ActorComponent");
    let owner = function(component, "owner_id");
    assert_eq!(owner.id, OWNER_ID);
    assert!(owner.pure && owner.callable);
    assert_eq!(owner.returns, schema::Type::ObjectRef { class: None });
    // Storage lifetime stays a runtime concern, never an authoring one.
    assert!(
        !component.functions.iter().any(|f| f.name == "releasable"),
        "releasable must not be reflected"
    );

    // Every derived native base inherits the operations through the registry, which is
    // what a Blueprint or Lua class deriving from Actor3D actually resolves against.
    let inherited = registry
        .ancestry("epok::Actor3D")
        .into_iter()
        .flat_map(|c| c.functions.iter())
        .map(|f| f.name.clone())
        .collect::<Vec<_>>();
    for name in ["set_active", "destroy", "active", "wants_tick"] {
        assert!(
            inherited.iter().any(|f| f == name),
            "epok::Actor3D does not inherit {name}: {inherited:?}"
        );
    }
    std::fs::remove_dir_all(&root).unwrap();
}

#[test]
#[ignore = "requires the pinned host extractor; stages a disposable project without building"]
fn a_blueprint_graph_calls_the_reflected_actor_operations_on_self() {
    let root = project("runtime-api-blueprint");
    let catalog = crate::scripts::catalog(&root).unwrap();
    let registry = crate::blueprint::native_registry(&root, &catalog).unwrap();

    let asset_id = uuid::Uuid::new_v4().to_string();
    let mut asset = BlueprintAsset::new("BP_Sentry".into(), om::ACTOR3D_ID.into());
    asset.id = asset_id.clone();
    let begin_play = registry
        .ancestry("epok::Actor3D")
        .into_iter()
        .flat_map(|c| c.functions.iter())
        .find(|f| f.name == "begin_play")
        .expect("reflected begin_play")
        .id
        .clone();

    let node = |n: u8, kind: NodeKind| Node {
        id: format!("00000000-0000-4000-8000-0000000000{n:02x}"),
        kind,
        inputs: BTreeMap::new(),
        outputs: BTreeMap::new(),
    };
    let mut entry = node(1, NodeKind::Entry);
    entry
        .outputs
        .insert("next".into(), vec![node(2, NodeKind::Entry).id]);
    let mut call = node(
        2,
        NodeKind::Call {
            function: SET_ACTIVE.into(),
        },
    );
    call.inputs.insert(
        "active".into(),
        crate::blueprint_asset::Input::Literal {
            value_type: schema::Type::Bool,
            value: serde_json::json!(false),
        },
    );
    let entry_id = entry.id.clone();
    asset.functions.push(Graph {
        timeline: None,
        id: "00000000-0000-4000-8000-0000000000ff".into(),
        name: "begin_play".into(),
        override_id: Some(begin_play),
        parameters: vec![],
        returns: schema::Type::Void,
        entry: entry_id,
        nodes: vec![entry, call],
    });

    let file = AssetFile::file("assets/blueprints/BP_Sentry.epokbp".into(), asset);
    let compiled = crate::blueprint_compile::compile(Path::new(""), &registry, &[file])
        .unwrap_or_else(|d| panic!("{d:?}"));
    let text = String::from_utf8(
        compiled.artifacts.files[&PathBuf::from(format!("scripts/generated/{asset_id}.hpp"))]
            .clone(),
    )
    .unwrap();
    // The reflected member is lowered as a direct call on self, with no builtin adapter.
    assert!(text.contains("this->set_active("), "{text}");
    assert!(!text.contains("epok::bp::api::set_active"), "{text}");
    std::fs::remove_dir_all(&root).unwrap();
}

/// Writes a Lua class over `epok::Actor3D` whose `begin_play` uses nothing but the
/// engine base's own reflected operations, and returns the project root plus the id of
/// the generated class.
fn lua_actor_project(label: &str) -> (PathBuf, String) {
    let root = project(label);
    let mut manifest = crate::workspace::read_manifest(&root).unwrap();
    manifest.lua_execution = crate::settings::LuaExecution::NativeCpp;
    crate::workspace::save_manifest(&root, &manifest).unwrap();

    let created = crate::lua_asset::create_in(&root, "Sentry", "", "epok::Actor3D").unwrap();
    let source = std::fs::read_to_string(&created).unwrap();
    // The identity lives beside the script, not inside it: the editor recorded
    // it when it created the class.
    let class_id =
        crate::lua_identity::read(&root).unwrap().classes["assets/scripts/Sentry.lua"].clone();
    // Nothing here is a C++ helper: these are the engine base's own reflected members.
    let source = source.replace(
        "function Sentry:begin_play()\nend",
        "function Sentry:begin_play()\n    if self:active() then\n        self:set_active(false)\n    else\n        self:destroy()\n    end\nend",
    );
    assert!(source.contains("self:set_active(false)"));
    std::fs::write(&created, &source).unwrap();

    let catalog = crate::scripts::catalog(&root).unwrap();
    assert!(catalog.iter().any(|s| s.name == "Sentry"));
    (root, class_id)
}

#[test]
#[ignore = "requires the pinned host extractor; stages a disposable project without building"]
fn a_lua_class_calls_the_inherited_actor_operations_in_native_mode() {
    let (root, class_id) = lua_actor_project("runtime-api-lua");
    let scene =
        crate::scene::Scene::load(&crate::workspace::startup_scene(&root).unwrap()).unwrap();
    let build = root.join(".epok/build");
    crate::project::stage_into(&root, &scene, &build).unwrap();
    let generated = build.join(format!("scripts/generated/lua/{class_id}.hpp"));
    let text = std::fs::read_to_string(&generated).unwrap();
    // Inherited reflected members are lowered as direct calls on the generated class,
    // with no Lua-specific bridge and no C++ helper written by the author.
    assert!(text.contains("this->set_active(false)"), "{text}");
    assert!(text.contains("this->destroy()"), "{text}");
    assert!(text.contains("this->active()"), "{text}");
    std::fs::remove_dir_all(&root).unwrap();
}

/// The real proof that the reflected operations are a usable authoring surface: the
/// generated C++ for a Lua class that calls them compiles for MIPS and links.
#[test]
#[ignore = "requires pinned MIPS tools and the host extractor; builds without launching the emulator"]
fn a_lua_class_using_the_reflected_actor_operations_builds_for_mips() {
    let (root, _) = lua_actor_project("runtime-api-lua-mips");
    let path = crate::workspace::startup_scene(&root).unwrap();
    let scene = crate::scene::Scene::load(&path).unwrap();
    let input =
        crate::play::input(&root, path, scene, crate::play::Profile::default(), false).unwrap();
    let job = crate::pipeline::Job::start_with_debug(root.clone(), input, false, false);
    let started = std::time::Instant::now();
    loop {
        assert!(started.elapsed().as_secs() < 600, "Build timed out");
        match job.events.recv_timeout(std::time::Duration::from_secs(2)) {
            Ok(crate::pipeline::Event::Log(line)) => eprintln!("[build] {line}"),
            Ok(crate::pipeline::Event::Finished(result)) => {
                result.unwrap();
                break;
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                panic!("Build worker disconnected")
            }
            _ => {}
        }
    }
    assert!(
        std::fs::read(root.join(".epok/build/epok.ps-exe"))
            .unwrap()
            .starts_with(b"PS-X EXE")
    );
    std::fs::remove_dir_all(&root).unwrap();
}
