use crate::{mcp, mcp_tools, settings::Mcp};
use serde_json::json;
use std::{
    fs,
    time::{Duration, Instant},
};

fn editor() -> crate::editor::Editor {
    let root = crate::workspace::editor_home()
        .join(".epok")
        .join(format!("mcp-test-{}", uuid::Uuid::new_v4()));
    let project =
        crate::workspace::create(&root, "MCP Test", crate::workspace::Template::Basic).unwrap();
    let mut e = crate::editor::Editor::open(project).unwrap();
    e.preferences.mcp = Mcp::default();
    e
}
#[test]
fn serial_controls_report_pending_and_guard_reset_target() {
    let mut e = editor();
    e.auto_build = false;
    let mut state = mcp::State::default();
    let (job, events, controls) = crate::pipeline::Job::test_channels();
    e.job = Some(job);
    e.playing = true;
    assert!(mcp_tools::execute(&mut e, &mut state, "editor_control", json!({"action":"reset_psx"})).is_err());
    assert!(controls.try_recv().is_err());
    e.active_play_target = crate::play::Target::Serial;
    let reply = mcp_tools::execute(&mut e, &mut state, "editor_control", json!({"action":"pause"})).unwrap();
    assert_eq!(reply["state"]["serial"]["command_pending"], true);
    assert!(matches!(controls.try_recv(), Ok(crate::pipeline::Control::Pause)));
    assert!(mcp_tools::execute(&mut e, &mut state, "editor_control", json!({"action":"resume"})).is_err());
    events.send(crate::pipeline::Event::SerialCommandPending(false)).unwrap();
    events.send(crate::pipeline::Event::Paused(true)).unwrap();
    e.tick();
    mcp_tools::execute(&mut e, &mut state, "editor_control", json!({"action":"resume"})).unwrap();
    assert!(matches!(controls.try_recv(), Ok(crate::pipeline::Control::Resume)));
    e.serial_ui.command_pending = false;
    mcp_tools::execute(&mut e, &mut state, "editor_control", json!({"action":"reset_psx"})).unwrap();
    assert!(matches!(controls.try_recv(), Ok(crate::pipeline::Control::Reset)));
    mcp_tools::execute(&mut e, &mut state, "editor_control", json!({"action":"stop"})).unwrap();
    assert!(matches!(controls.try_recv(), Ok(crate::pipeline::Control::Stop)));
    let root = e.root.clone();
    drop(e);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn scene_batches_are_atomic_versioned_and_reversible() {
    let mut e = editor();
    let mut state = mcp::State::default();
    let initial = mcp_tools::revision(&e.scene);
    let original = e.scene.clone();
    e.rename = Some((0, "Pending rename".into()));
    e.hud_drag = Some((0, false));
    let run = |e: &mut crate::editor::Editor, s: &mut mcp::State, ops| {
        mcp_tools::execute(
            e,
            s,
            "scene_apply",
            json!({"revision":mcp_tools::revision(&e.scene),"operations":ops}),
        )
    };
    assert!(run(&mut e,&mut state,json!([{"op":"create","entity":{"name":"Temporary"}},{"op":"update","index":999,"patch":{"name":"Bad"}}])).is_err());
    assert_eq!(e.scene, original);
    assert!(state.undo.is_empty());
    assert!(
        run(
            &mut e,
            &mut state,
            json!([{"op":"create","entity":{"name":"Bad","material":{"typo":1}}}])
        )
        .is_err()
    );
    run(&mut e,&mut state,json!([{"op":"create","entity":{"name":"AI Mesh","position":[1,2,3]}},{"op":"update","index":0,"patch":{"material":{"color":[0.2,0.5,1.0]}}}])).unwrap();
    let updated = mcp_tools::revision(&e.scene);
    assert!(e.rename.is_none() && e.hud_drag.is_none());
    assert_ne!(initial, updated);
    assert!(
        mcp_tools::execute(
            &mut e,
            &mut state,
            "scene_save",
            json!({"revision":initial})
        )
        .is_err()
    );
    mcp_tools::execute(
        &mut e,
        &mut state,
        "scene_history",
        json!({"revision":updated,"action":"undo"}),
    )
    .unwrap();
    assert_eq!(e.scene, original);
    mcp_tools::execute(
        &mut e,
        &mut state,
        "scene_history",
        json!({"revision":initial,"action":"redo"}),
    )
    .unwrap();
    e.scene.entities[0].name = "Human edit".into();
    let hash = mcp_tools::revision(&e.scene);
    assert!(
        mcp_tools::execute(
            &mut e,
            &mut state,
            "scene_history",
            json!({"revision":hash,"action":"undo"})
        )
        .is_err()
    );
    assert_eq!(e.scene.entities[0].name, "Human edit");
}
#[test]
fn files_preserve_conflicts_backups_and_project_boundary() {
    let mut e = editor();
    let mut state = mcp::State::default();
    for path in [
        "../outside",
        "assets/../../outside",
        "assets/../epok.project.json",
        "C:/Windows/system.ini",
        "assets/test.txt:secret",
    ] {
        assert!(
            mcp_tools::execute(
                &mut e,
                &mut state,
                "project_files",
                json!({"action":"read","path":path})
            )
            .is_err()
        );
    }
    let a = json!({"action":"write","path":"assets/scripts/AI note.txt","revision":"absent","content":"first"});
    let saved = mcp_tools::execute(&mut e, &mut state, "project_files", a.clone()).unwrap();
    assert!(mcp_tools::execute(&mut e, &mut state, "project_files", a).is_err());
    let updated=mcp_tools::execute(&mut e,&mut state,"project_files",json!({"action":"write","path":"assets/scripts/AI note.txt","revision":saved["revision"],"content":"second"})).unwrap();
    assert_eq!(
        fs::read_to_string(e.root.join(updated["backup"].as_str().unwrap())).unwrap(),
        "first"
    );
    let active = crate::assets::path_string(&e.root, &e.scene_path());
    assert!(
        mcp_tools::execute(
            &mut e,
            &mut state,
            "project_files",
            json!({"action":"delete","path":active,"revision":"absent"})
        )
        .is_err()
    );
    #[cfg(windows)]
    {
        use std::os::windows::fs::symlink_dir;
        let outside = e.root.parent().unwrap();
        if symlink_dir(outside, e.root.join("assets/escape")).is_ok() {
            assert!(mcp_tools::execute(&mut e,&mut state,"project_files",json!({"action":"write","path":"assets/escape/test","revision":"absent","content":"bad"})).is_err());
        }
    }
}
#[test]
fn mesh_assets_can_be_created_attached_and_edited() {
    let mut e = editor();
    let mut state = mcp::State::default();
    let asset = mcp_tools::execute(
        &mut e,
        &mut state,
        "mesh_create",
        json!({"path":"assets/meshes/AI.epokasset","shape":"Stairs","steps":4}),
    )
    .unwrap();
    let hash = mcp_tools::revision(&e.scene);
    mcp_tools::execute(&mut e,&mut state,"scene_apply",json!({"revision":hash,"operations":[{"op":"create","entity":{"name":"AI Stairs","editable_mesh":asset["component"]}}]})).unwrap();
    assert!(
        e.scene
            .entities
            .last()
            .unwrap()
            .editable_mesh
            .as_ref()
            .unwrap()
            .document
            .is_some()
    );
    let current = mcp_tools::execute(
        &mut e,
        &mut state,
        "asset_document",
        json!({"path":asset["path"]}),
    )
    .unwrap();
    let mut document = current["document"].clone();
    let expected = document["vertices"][0][0].as_f64().unwrap() + 2.0;
    for vertex in document["vertices"].as_array_mut().unwrap() {
        vertex[0] = json!(vertex[0].as_f64().unwrap() + 2.0);
    }
    mcp_tools::execute(
        &mut e,
        &mut state,
        "asset_document",
        json!({"path":asset["path"],"revision":current["revision"],"document":document}),
    )
    .unwrap();
    assert_eq!(
        e.scene
            .entities
            .last()
            .unwrap()
            .editable_mesh
            .as_ref()
            .unwrap()
            .document
            .as_ref()
            .unwrap()
            .vertices[0][0],
        expected as f32
    );
    let index = crate::assets::scan(&e.root, &mut Default::default());
    assert!(index.problems.is_empty(), "{:?}", index.problems);
    assert_eq!(index.usable().count(), 1);
    let record = index.usable().next().unwrap();
    assert!(
        mcp_tools::execute(
            &mut e,
            &mut state,
            "asset_manage",
            json!({"action":"trash","path":asset["path"],"revision":record.revision})
        )
        .is_err()
    );
    let duplicate = mcp_tools::execute(&mut e,&mut state,"asset_manage",json!({"action":"duplicate","path":asset["path"],"revision":record.revision,"destination":"assets/meshes/Copy.epokasset"})).unwrap();
    assert_ne!(duplicate["id"], asset["id"]);
    mcp_tools::execute(&mut e,&mut state,"asset_manage",json!({"action":"move","path":asset["path"],"revision":record.revision,"destination":"assets/meshes/Moved.epokasset"})).unwrap();
    assert!(
        e.scene
            .entities
            .last()
            .unwrap()
            .editable_mesh
            .as_ref()
            .unwrap()
            .document
            .is_some()
    );
}
#[test]
fn real_mcp_client_can_negotiate_call_tools_and_read_resources() {
    use rmcp::{
        ServiceExt,
        model::*,
        transport::{
            StreamableHttpClientTransport,
            streamable_http_client::StreamableHttpClientTransportConfig,
        },
    };
    let mut e = editor();
    assert!(!e.preferences.mcp.enabled);
    let probe = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = probe.local_addr().unwrap().port();
    e.preferences.mcp.port = port;
    mcp::tick(&mut e);
    assert_eq!(e.mcp.status, "Disabled");
    e.preferences.mcp.enabled = true;
    e.preferences.mcp.prepare();
    mcp::tick(&mut e);
    assert!(e.mcp.status.contains("Cannot listen"));
    drop(probe);
    e.preferences.mcp.enabled = false;
    mcp::tick(&mut e);
    e.preferences.mcp.enabled = true;
    mcp::tick(&mut e);
    assert!(e.mcp.status.starts_with("Listening"), "{}", e.mcp.status);
    let settings = e.preferences.mcp.clone();
    let client = std::thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async move {
                let transport = StreamableHttpClientTransport::from_config(
                    StreamableHttpClientTransportConfig::with_uri(settings.endpoint())
                        .auth_header(settings.token),
                );
                let client = ().serve(transport).await.unwrap();
                let listed = client.list_all_tools().await.unwrap();
                assert!(listed.iter().any(|t| t.name == "viewer_screenshot"));
                let result = client
                    .call_tool(CallToolRequestParams::new("scene_read"))
                    .await
                    .unwrap();
                assert_ne!(result.is_error, Some(true));
                let data: serde_json::Value =
                    serde_json::from_str(&result.content[0].as_text().unwrap().text).unwrap();
                assert!(data["scene"]["entities"].is_array());
                let result = client
                    .read_resource(ReadResourceRequestParams::new("epok://editor/state"))
                    .await
                    .unwrap();
                assert_eq!(result.contents.len(), 1);
                client.cancel().await.unwrap();
            });
    });
    let deadline = Instant::now() + Duration::from_secs(20);
    while !client.is_finished() {
        assert!(Instant::now() < deadline, "MCP client timed out");
        mcp::tick(&mut e);
        std::thread::sleep(Duration::from_millis(5));
    }
    client.join().unwrap();
    // Test auth and DNS rebinding defenses using actual HTTP, independent of the SDK client.
    for (token, host, origin, expected) in [
        (
            "wrong".to_string(),
            format!("127.0.0.1:{port}"),
            None,
            "401",
        ),
        (
            e.preferences.mcp.token.clone(),
            "evil.example".into(),
            None,
            "403",
        ),
        (
            e.preferences.mcp.token.clone(),
            format!("127.0.0.1:{port}"),
            Some("https://evil.example"),
            "403",
        ),
    ] {
        use std::io::{Read, Write};
        let mut socket = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        write!(socket,"POST /mcp HTTP/1.1\r\nHost: {host}\r\nAuthorization: Bearer {token}\r\n{}Content-Length: 0\r\nConnection: close\r\n\r\n",origin.map_or(String::new(),|o|format!("Origin: {o}\r\n"))).unwrap();
        let mut output = String::new();
        socket.read_to_string(&mut output).unwrap();
        assert!(
            output.lines().next().unwrap().contains(expected),
            "{}",
            output.lines().next().unwrap()
        );
    }
    e.preferences.mcp.enabled = false;
    mcp::tick(&mut e);
    assert_eq!(e.mcp.status, "Disabled");
    assert!(std::net::TcpListener::bind(("127.0.0.1", port)).is_ok());
}

#[test]
fn editor_view_translates_the_legacy_scene_2d_flag_and_the_three_way_view_mode() {
    let mut e = editor();
    let mut state = mcp::State::default();
    let view = |e: &mut crate::editor::Editor, s: &mut mcp::State, a| {
        mcp_tools::execute(e, s, "editor_view", a)
    };
    // The default is the 3D world and both spellings report it.
    let reply = view(&mut e, &mut state, json!({})).unwrap();
    assert_eq!(reply["view_mode"], "3d");
    assert_eq!(reply["scene_2d"], false);
    // Legacy: true is the UI (Canvas/HUD) mode, false is 3D.
    let reply = view(&mut e, &mut state, json!({"scene_2d":true})).unwrap();
    assert_eq!(reply["view_mode"], "ui");
    assert_eq!(reply["scene_2d"], true);
    assert!(e.scene_2d());
    let reply = view(&mut e, &mut state, json!({"scene_2d":false})).unwrap();
    assert_eq!(reply["view_mode"], "3d");
    assert_eq!(reply["scene_2d"], false);
    // The 2D world is reachable only through view_mode, and is not scene_2d.
    let reply = view(&mut e, &mut state, json!({"view_mode":"2d"})).unwrap();
    assert_eq!(reply["view_mode"], "2d");
    assert_eq!(reply["scene_2d"], false);
    assert_eq!(
        e.scene_view_mode,
        crate::scene_view_mode::SceneViewMode::TwoD
    );
    // view_mode wins when both are sent.
    let reply = view(
        &mut e,
        &mut state,
        json!({"scene_2d":true,"view_mode":"3d"}),
    )
    .unwrap();
    assert_eq!(reply["view_mode"], "3d");
    assert_eq!(reply["scene_2d"], false);
    // Unknown spellings are refused; the mode is unchanged.
    assert!(view(&mut e, &mut state, json!({"view_mode":"hud"})).is_err());
    assert!(view(&mut e, &mut state, json!({"view_mode":true})).is_err());
    assert!(view(&mut e, &mut state, json!({"scene_2d":"ui"})).is_err());
    assert_eq!(
        e.scene_view_mode,
        crate::scene_view_mode::SceneViewMode::ThreeD
    );
    // Leaving the UI mode stops the HUD preview; entering it never starts one.
    view(&mut e, &mut state, json!({"view_mode":"ui"})).unwrap();
    e.hud_simulation.running = true;
    view(&mut e, &mut state, json!({"view_mode":"2d"})).unwrap();
    assert!(!e.hud_simulation.running);
    let root = e.root.clone();
    drop(e);
    fs::remove_dir_all(root).unwrap();
}

/// Calls one tool with the scene's current revision. Keeps the borrow of `Editor` and
/// the borrow of `Editor::scene` out of the same expression.
fn at_revision(
    e: &mut crate::editor::Editor,
    state: &mut mcp::State,
    name: &str,
    mut args: serde_json::Value,
) -> Result<serde_json::Value, String> {
    if let Some(object) = args.as_object_mut() {
        object.insert("revision".into(), json!(mcp_tools::revision(&e.scene)));
    }
    mcp_tools::execute(e, state, name, args)
}

/// Native bases in the shape `Model::from_registry` resolves. Reflection needs the MIPS
/// include paths, so every object-model test in the tree builds its classes by hand.
pub(crate) fn actor_catalog() -> Vec<crate::scripts::Script> {
    use crate::{object_model as om, reflection_schema as schema};
    fn class(id: &str, cpp_name: &str, parent: Option<&str>) -> schema::Class {
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
            source: schema::Location {
                file: std::path::PathBuf::from("runtime/object_model.hpp"),
                line: 0,
                column: 0,
            },
        }
    }
    let mut actor = class(om::ACTOR_ID, "epok::Actor", None);
    actor.family = Some(schema::ClassFamily::Actor);
    actor.abstract_class = true;
    let mut actor3d = class(om::ACTOR3D_ID, "epok::Actor3D", Some(om::ACTOR_ID));
    actor3d.domain = Some(schema::Domain::World3D);
    actor3d.placement = schema::Placement {
        placeable: true,
        spawnable: true,
        scene_managed: false,
    };
    let mut script_actor = class(
        om::SCENE_SCRIPT_ACTOR_ID,
        "epok::SceneScriptActor",
        Some(om::ACTOR_ID),
    );
    script_actor.placement = schema::Placement {
        placeable: false,
        spawnable: false,
        scene_managed: true,
    };
    let mut component = class(om::ACTOR_COMPONENT_ID, "epok::ActorComponent", None);
    component.family = Some(schema::ClassFamily::Component);
    component.abstract_class = true;
    let mut root = class(
        om::SCENE_COMPONENT3D_ID,
        "epok::SceneComponent3D",
        Some(om::ACTOR_COMPONENT_ID),
    );
    root.domain = Some(schema::Domain::World3D);
    root.component = Some(schema::ComponentContract {
        owners: [schema::Domain::World3D].into_iter().collect(),
        requires: vec![],
        excludes: vec![],
        cardinality: schema::Cardinality::Single,
        can_root: true,
        capabilities: Default::default(),
    });
    // A second domain, so the attachment rule of a cross-domain reparent is
    // exercised rather than assumed.
    let mut actor2d = class(om::ACTOR2D_ID, "epok::Actor2D", Some(om::ACTOR_ID));
    actor2d.domain = Some(schema::Domain::World2D);
    actor2d.placement = schema::Placement {
        placeable: true,
        spawnable: true,
        scene_managed: false,
    };
    let mut root2d = class(
        om::SCENE_COMPONENT2D_ID,
        "epok::SceneComponent2D",
        Some(om::ACTOR_COMPONENT_ID),
    );
    root2d.domain = Some(schema::Domain::World2D);
    root2d.component = Some(schema::ComponentContract {
        owners: [schema::Domain::World2D].into_iter().collect(),
        requires: vec![],
        excludes: vec![],
        cardinality: schema::Cardinality::Single,
        can_root: true,
        capabilities: Default::default(),
    });
    // A domain-less component every spatial actor may own, several times over.
    let mut audio = class(
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
        capabilities: Default::default(),
    });
    vec![crate::scripts::Script {
        name: "epok::Actor".into(),
        parent: None,
        properties: vec![],
        header: std::path::PathBuf::from("object_model.hpp"),
        classes: vec![
            actor,
            actor3d,
            actor2d,
            script_actor,
            component,
            root,
            root2d,
            audio,
        ],
    }]
}

#[test]
fn actor_tools_place_validate_edit_and_undo_through_the_class_model() {
    let mut e = editor();
    e.auto_build = false;
    e.catalog = actor_catalog();
    let mut state = mcp::State::default();

    // Reading is class-aware and shows the derived view of the legacy entities.
    let listed = mcp_tools::execute(&mut e, &mut state, "scene_actors", json!({})).unwrap();
    assert_eq!(listed["model_available"], true);
    assert!(listed["actors"].as_array().unwrap().is_empty());
    assert!(!listed["derived"].as_array().unwrap().is_empty());
    assert!(
        listed["placeable"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["class"] == "epok::Actor3D")
    );

    // Placement is validated by object_model::Model::placeable(), never by name.
    let error = at_revision(
        &mut e,
        &mut state,
        "scene_add_actor",
        json!({"class":"epok::SceneScriptActor"}),
    )
    .unwrap_err();
    assert!(error.contains("not a placeable actor class"), "{error}");
    assert!(e.scene.actors.is_empty());

    let added = at_revision(
        &mut e,
        &mut state,
        "scene_add_actor",
        json!({"class":"epok::Actor3D","name":"Hero"}),
    )
    .unwrap();
    // Editor::changed() ran, so the build is stale and P7's History has the edit.
    assert!(e.dirty && e.view_dirty && state.undo.len() == 1);
    assert_eq!(e.scene.actors.len(), 1);
    assert_eq!(e.scene.actors[0].name, "Hero");
    assert!(
        e.scene.actors[0]
            .root()
            .is_some_and(|c| c.class.name == "epok::SceneComponent3D")
    );
    let hero: uuid::Uuid = serde_json::from_value(added["id"].clone()).unwrap();

    // A logical parent must be an actor of this scene.
    assert!(
        at_revision(
            &mut e,
            &mut state,
            "scene_add_actor",
            json!({"class":"epok::Actor3D","parent":uuid::Uuid::new_v4()}),
        )
        .is_err()
    );
    at_revision(
        &mut e,
        &mut state,
        "scene_add_actor",
        json!({"class":"epok::Actor3D","name":"Child","parent":hero}),
    )
    .unwrap();
    assert_eq!(e.scene.actors[1].logical_parent, Some(hero));

    // Overrides and the active flag; null clears an override.
    at_revision(
        &mut e,
        &mut state,
        "scene_set_actor",
        json!({"id":hero,"active":false,"name":"Champion","properties":{"speed":2.5}}),
    )
    .unwrap();
    assert!(!e.scene.actors[0].active && e.scene.actors[0].name == "Champion");
    assert_eq!(e.scene.actors[0].properties["speed"], json!(2.5));
    assert!(e.scene.actors[0].overrides.contains("speed"));
    at_revision(
        &mut e,
        &mut state,
        "scene_set_actor",
        json!({"id":hero,"properties":{"speed":null}}),
    )
    .unwrap();
    assert!(e.scene.actors[0].properties.is_empty() && e.scene.actors[0].overrides.is_empty());

    // Undo is the shared MCP history, so actor edits reverse like entity edits.
    at_revision(
        &mut e,
        &mut state,
        "scene_history",
        json!({"action":"undo"}),
    )
    .unwrap();
    assert_eq!(e.scene.actors[0].properties["speed"], json!(2.5));

    // Removing the parent clears the child's dangling reference instead of orphaning it.
    at_revision(&mut e, &mut state, "scene_remove_actor", json!({"id":hero})).unwrap();
    assert_eq!(e.scene.actors.len(), 1);
    assert_eq!(e.scene.actors[0].logical_parent, None);
    assert!(at_revision(&mut e, &mut state, "scene_remove_actor", json!({"id":hero}),).is_err());

    // The tools are in the advertised catalog with their schemas.
    let names: Vec<String> = mcp_tools::catalog()
        .into_iter()
        .map(|tool| tool.name.to_string())
        .collect();
    for tool in [
        "scene_actors",
        "scene_add_actor",
        "scene_remove_actor",
        "scene_set_actor",
    ] {
        assert!(
            names.contains(&tool.to_string()),
            "{tool} is not advertised"
        );
    }
    assert!(mcp_tools::validate_arguments("scene_add_actor", &json!({"revision":"x"})).is_err());

    let root = e.root.clone();
    drop(e);
    fs::remove_dir_all(root).unwrap();
}

/// `scene_set_actor` mirrors the Hierarchy's reparent and the Inspector's
/// Components section, under exactly the same model rules.
#[test]
fn scene_set_actor_reparents_and_edits_the_component_set_like_the_inspector_does() {
    let mut e = editor();
    e.auto_build = false;
    e.catalog = actor_catalog();
    let mut state = mcp::State::default();
    let add = |e: &mut crate::editor::Editor, state: &mut mcp::State, class: &str, name: &str| {
        let value = at_revision(
            e,
            state,
            "scene_add_actor",
            json!({"class":class,"name":name}),
        )
        .unwrap_or_else(|error| panic!("{name}: {error}"));
        serde_json::from_value::<uuid::Uuid>(value["id"].clone()).unwrap()
    };
    let hero = add(&mut e, &mut state, "epok::Actor3D", "Hero");
    let sidekick = add(&mut e, &mut state, "epok::Actor3D", "Sidekick");
    let sprite = add(&mut e, &mut state, "epok::Actor2D", "Sprite");

    // Same domain: the logical parent carries a spatial attachment with it.
    at_revision(
        &mut e,
        &mut state,
        "scene_set_actor",
        json!({"id":sidekick,"parent":hero}),
    )
    .unwrap();
    let at = |e: &crate::editor::Editor, id: uuid::Uuid| {
        let actor = e.scene.actors.iter().find(|a| a.id == id).unwrap();
        (
            actor.logical_parent,
            actor.attach.as_ref().map(|at| at.actor),
        )
    };
    assert_eq!(at(&e, sidekick), (Some(hero), Some(hero)));

    // Across domains the hierarchy holds and the attachment does not.
    at_revision(
        &mut e,
        &mut state,
        "scene_set_actor",
        json!({"id":sprite,"parent":hero}),
    )
    .unwrap();
    assert_eq!(at(&e, sprite), (Some(hero), None));

    // Null unparents, clearing the attachment with it.
    at_revision(
        &mut e,
        &mut state,
        "scene_set_actor",
        json!({"id":sidekick,"parent":null}),
    )
    .unwrap();
    assert_eq!(at(&e, sidekick), (None, None));

    // A cycle is refused by the document validator, and nothing is applied.
    at_revision(
        &mut e,
        &mut state,
        "scene_set_actor",
        json!({"id":sidekick,"parent":hero}),
    )
    .unwrap();
    let error = at_revision(
        &mut e,
        &mut state,
        "scene_set_actor",
        json!({"id":hero,"parent":sidekick}),
    )
    .unwrap_err();
    assert!(error.contains("is its own ancestor"), "{error}");
    assert_eq!(at(&e, hero), (None, None));
    assert!(
        at_revision(
            &mut e,
            &mut state,
            "scene_set_actor",
            json!({"id":hero,"parent":hero}),
        )
        .is_err()
    );

    // Adding a component: fresh identity, unique name inside the actor, not root
    // and not inherited.
    let added = at_revision(
        &mut e,
        &mut state,
        "scene_set_actor",
        json!({"id":hero,"components":{"add":[{"class":"epok::AudioComponent"},{"class":"epok::AudioComponent","name":"Music"}]}}),
    )
    .unwrap();
    assert_eq!(added["components"].as_array().unwrap().len(), 3);
    let hero_index = e.scene.actor_index(hero).unwrap();
    let names: Vec<_> = e.scene.actors[hero_index]
        .components
        .iter()
        .map(|c| c.name.clone())
        .collect();
    assert_eq!(names, vec!["Root", "AudioComponent", "Music"]);
    let audio = e.scene.actors[hero_index].components[1].clone();
    assert!(!audio.root && !audio.inherited && !audio.id.is_nil());

    // A component the owner's domain does not accept is refused by the model.
    let error = at_revision(
        &mut e,
        &mut state,
        "scene_set_actor",
        json!({"id":hero,"components":{"add":[{"class":"epok::SceneComponent2D"}]}}),
    )
    .unwrap_err();
    assert!(error.contains("World2D"), "{error}");

    // Cardinality=Single is the model's rule, enforced on the whole set.
    let error = at_revision(
        &mut e,
        &mut state,
        "scene_set_actor",
        json!({"id":hero,"components":{"add":[{"class":"epok::SceneComponent3D"}]}}),
    )
    .unwrap_err();
    assert!(error.contains("Cardinality=Single"), "{error}");
    assert_eq!(e.scene.actors[hero_index].components.len(), 3);

    // The root is never removable.
    let root_component = e.scene.actors[hero_index].components[0].id;
    let error = at_revision(
        &mut e,
        &mut state,
        "scene_set_actor",
        json!({"id":hero,"components":{"remove":[root_component]}}),
    )
    .unwrap_err();
    assert!(
        error.contains("root component cannot be removed"),
        "{error}"
    );

    // Nor is a component contributed by the class.
    e.scene.actors[hero_index].components[1].inherited = true;
    let error = at_revision(
        &mut e,
        &mut state,
        "scene_set_actor",
        json!({"id":hero,"components":{"remove":[audio.id]}}),
    )
    .unwrap_err();
    assert!(error.contains("inherited from the class"), "{error}");
    e.scene.actors[hero_index].components[1].inherited = false;

    // Removing clears an attachment that named the component instead of dangling.
    let sidekick_index = e.scene.actor_index(sidekick).unwrap();
    e.scene.actors[sidekick_index].attach = Some(crate::actor_document::Attachment {
        actor: hero,
        component: Some(audio.id),
    });
    at_revision(
        &mut e,
        &mut state,
        "scene_set_actor",
        json!({"id":hero,"components":{"remove":[audio.id]}}),
    )
    .unwrap();
    assert_eq!(e.scene.actors[hero_index].components.len(), 2);
    assert!(e.scene.actors[sidekick_index].attach.is_none());

    // An unknown operation is refused rather than silently ignored.
    assert!(
        at_revision(
            &mut e,
            &mut state,
            "scene_set_actor",
            json!({"id":hero,"components":{"replace":[]}}),
        )
        .is_err()
    );

    // scene_actors reports the component's override set and removability.
    at_revision(
        &mut e,
        &mut state,
        "scene_set_actor",
        json!({"id":hero,"properties":{"speed":1.0}}),
    )
    .unwrap();
    let music = e.scene.actors[hero_index].components[1].id;
    e.scene.actors[hero_index].components[1]
        .properties
        .insert("volume".into(), json!(0.5));
    e.scene.actors[hero_index].components[1]
        .overrides
        .insert("volume".into());
    let listed = mcp_tools::execute(&mut e, &mut state, "scene_actors", json!({})).unwrap();
    let reported = listed["actors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["id"] == json!(hero))
        .unwrap()
        .clone();
    let component = reported["components"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == json!(music))
        .unwrap();
    assert_eq!(component["properties"]["volume"], json!(0.5));
    assert_eq!(component["overrides"], json!(["volume"]));
    assert_eq!(component["removable"], json!(true));
    assert_eq!(reported["components"][0]["removable"], json!(false));
    assert_eq!(reported["components"][0]["root"], json!(true));

    // The new arguments are in the advertised schema and are type checked.
    assert!(
        mcp_tools::validate_arguments(
            "scene_set_actor",
            &json!({"revision":"x","id":"y","parent":null,"components":{}}),
        )
        .is_ok()
    );
    assert!(
        mcp_tools::validate_arguments(
            "scene_set_actor",
            &json!({"revision":"x","id":"y","components":[]}),
        )
        .is_err()
    );

    let root = e.root.clone();
    drop(e);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn soundfont_mcp_import_detects_library_and_accepts_partial_bank_settings() {
    let root = crate::workspace::tests::temp("mcp-soundfont");
    std::fs::create_dir_all(root.join("assets")).unwrap();
    std::fs::write(root.join("assets/library.sf2"), crate::sf2::fixture()).unwrap();
    let mut editor = crate::editor::Editor::new(root.clone());
    editor.auto_build = false;
    let mut state = crate::mcp::State::default();
    for (filename, partial) in [("default", false), ("partial", true)] {
        let mut args = serde_json::json!({"source": "assets/library.sf2", "destination": format!("assets/{filename}.epokasset")});
        if partial { args["bank_settings"] = serde_json::json!({"provenance": "Original synthetic test instrument"}); }
        assert_eq!(crate::mcp_tools::execute(&mut editor, &mut state, "asset_import", args).unwrap()["accepted"], true);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while editor.assets.busy {
            editor.assets.tick();
            assert!(std::time::Instant::now() < deadline, "{:?}", editor.assets.error);
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(editor.assets.form.is_none(), "{:?}", editor.assets.error);
        let package = crate::assets::Package::load(&root.join(format!("assets/{filename}.epokasset"))).unwrap();
        let bank = package.meta.settings.sound_bank().unwrap();
        assert!(bank.library.is_some() && bank.imported.is_none());
        assert_eq!(package.source, crate::sf2::fixture());
        if partial { assert_eq!(bank.provenance, "Original synthetic test instrument"); }
    }
}
