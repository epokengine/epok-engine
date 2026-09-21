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
/// Distance fog used to be reachable only through the raw `epok::fog_environment`
/// global, so it reached no authoring provider at all. These are the `Scene` group's
/// fog record and its getter/setter.
const SCENE_LIBRARY: &str = "e018f41e-b530-46f2-9331-9cf15f459fd2";
const FOG_SETTINGS: &str = "8b4375d1-f9cf-431c-9194-b16612bcf12d";
const FOG: &str = "5c70f7f6-3789-423e-83f1-608bc604bc2e";
const SET_FOG: &str = "3de7b991-edcb-40e6-8739-ca4cc5e4039b";
/// The post-HUD screen fade, which reached no authoring provider either while
/// `epok::screen_fade` was a bare mutable global. These are the `Scene` group's
/// accessors over it; there is no record, because the state is one scalar.
const SCREEN_FADE: &str = "91c95c93-f719-430c-b8e0-1c5863468be9";
const SET_SCREEN_FADE: &str = "2c441c7c-296e-4932-9b55-65a8f042a398";
/// The deepened tween surface: the pure-value scalar plan, the easing catalogue on
/// its own, and the Vector3 half. The Vector3 calls sit in a second library only
/// because `GameplayVector3` is declared in `gameplay_api.hpp`; they share the
/// `Utilities` category, so both halves reach one script namespace and one
/// Blueprint palette entry.
const UTILITY_LIBRARY: &str = "6499b0ab-c987-4e1b-b3f3-4918236ca9f1";
const UTILITY_VECTOR_LIBRARY: &str = "07422ba3-cdbf-4384-bde6-19521f4e1c2c";
const TWEEN_STATE: &str = "5e278107-d8f8-4a76-8c72-568e0d4d3fc2";
const VECTOR_TWEEN_STATE: &str = "c4306dbe-18ba-4f38-98bc-bdb558ed3667";
const VECTOR_TWEEN_SAMPLE: &str = "872b1e73-f320-4bf9-a6c3-e77ea6e92371";
const TWEEN_SCHEDULE: &str = "80e1ced0-392c-4534-85bf-d8bef9d016ff";
const EASE: &str = "15fc8b3a-80e8-4757-8216-38c89d3e3c7c";
const VECTOR_TWEEN_START: &str = "18823cc1-6141-40e6-8c2a-dc097bc32bfa";
const VECTOR_TWEEN_SCHEDULE: &str = "5cd97b29-6570-4071-aede-8f095a2d59c6";
const VECTOR_TWEEN_ADVANCE: &str = "ba4cbaf3-43a7-4476-aff8-c1cd48442c98";
const VECTOR_TWEEN_CANCEL: &str = "4e926f38-ba79-4cd8-9a64-aafd484d1c79";
const VECTOR_TWEEN_VALUE: &str = "9ffb6016-7a91-49ec-b8a6-315f6f5b91f0";

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

/// Fog is scene-global state, so it is an annotated pair on the `Scene` function
/// library rather than a component member. Nothing here is Lua- or Blueprint-specific:
/// the extractor turns the two `EPOK_FUNCTION` declarations into resolved operations
/// over one registered record, which is what every provider lowers.
#[test]
#[ignore = "requires the pinned host extractor; stages a disposable project without building"]
fn scene_fog_is_a_reflected_operation_pair_over_a_registered_record() {
    let root = project("runtime-api-fog");
    let catalog = crate::scripts::catalog(&root).unwrap();
    let registry = crate::blueprint::native_registry(&root, &catalog).unwrap();

    let record = registry
        .value_types
        .get(FOG_SETTINGS)
        .expect("epok::FogSettings");
    assert_eq!(record.cpp_name, "epok::FogSettings");
    // `end` is a Lua keyword, so the distances have to be spelled out or the
    // member would be unreadable in the two VM modes and in AOT alike.
    assert_eq!(
        record
            .value_type
            .members()
            .iter()
            .map(|f| (f.name.clone(), f.value_type.clone()))
            .collect::<Vec<_>>(),
        [
            ("enabled".to_owned(), schema::Type::Bool),
            ("start_distance".to_owned(), schema::Type::Fixed),
            ("end_distance".to_owned(), schema::Type::Fixed),
            ("red".to_owned(), schema::Type::UInt32),
            ("green".to_owned(), schema::Type::UInt32),
            ("blue".to_owned(), schema::Type::UInt32),
        ]
    );

    let library = registry
        .function_libraries
        .get(SCENE_LIBRARY)
        .expect("epok::SceneLibrary");
    let operation = |name: &str| {
        library
            .operations
            .iter()
            .find(|o| o.name == name)
            .unwrap_or_else(|| {
                panic!(
                    "{} does not reflect {name}: {:?}",
                    library.cpp_name,
                    library
                        .operations
                        .iter()
                        .map(|o| &o.name)
                        .collect::<Vec<_>>()
                )
            })
    };

    // The getter is a state read, so a data-only graph and a Lua expression can
    // both ask for the current settings without a mutation edge.
    let getter = operation("fog");
    assert_eq!(getter.id, FOG, "fog identity drifted");
    assert_eq!(getter.effect, schema::OperationEffect::StateRead);
    assert_eq!(getter.category, "Scene");
    assert_eq!(getter.native_target, "epok::SceneLibrary::fog");
    assert!(getter.parameters.is_empty());
    assert_eq!(getter.outputs.len(), 1);
    assert_eq!(getter.outputs[0].value_type, record.value_type);

    // The setter mutates and reports the deterministic rejection as its Bool
    // result rather than through an out parameter or a global error.
    let setter = operation("set_fog");
    assert_eq!(setter.id, SET_FOG, "set_fog identity drifted");
    assert_eq!(setter.effect, schema::OperationEffect::Mutation);
    assert_eq!(setter.category, "Scene");
    assert_eq!(setter.native_target, "epok::SceneLibrary::set_fog");
    assert_eq!(setter.parameters.len(), 1);
    assert_eq!(setter.parameters[0].value_type, record.value_type);
    assert_eq!(setter.outputs.len(), 1);
    assert_eq!(setter.outputs[0].value_type, schema::Type::Bool);
    assert!(setter.resource_demands.is_empty(), "fog cooks no sidecar");
    std::fs::remove_dir_all(&root).unwrap();
}

/// The screen fade is one scalar, so it is an annotated pair on the same `Scene`
/// library over a plain `uint32_t` rather than a record of its own. The C++ member
/// names are `screen_fade`/`set_screen_fade` and the backing global keeps its name,
/// which only compiles because the bodies qualify it; this pins the spellings that
/// every provider then lowers, none of which are Lua- or Blueprint-specific.
#[test]
#[ignore = "requires the pinned host extractor; stages a disposable project without building"]
fn scene_screen_fade_is_a_reflected_operation_pair_over_a_scalar() {
    let root = project("runtime-api-screen-fade");
    let catalog = crate::scripts::catalog(&root).unwrap();
    let registry = crate::blueprint::native_registry(&root, &catalog).unwrap();

    let library = registry
        .function_libraries
        .get(SCENE_LIBRARY)
        .expect("epok::SceneLibrary");
    let operation = |name: &str| {
        library
            .operations
            .iter()
            .find(|o| o.name == name)
            .unwrap_or_else(|| {
                panic!(
                    "{} does not reflect {name}: {:?}",
                    library.cpp_name,
                    library
                        .operations
                        .iter()
                        .map(|o| &o.name)
                        .collect::<Vec<_>>()
                )
            })
    };

    // The getter is a state read of the authored amount. The drawn amount is the
    // larger of it and the running transition's opacity, which `transition_snapshot`
    // reports; reading back what was just written therefore cannot surprise.
    let getter = operation("screen_fade");
    assert_eq!(getter.id, SCREEN_FADE, "screen_fade identity drifted");
    assert_eq!(getter.effect, schema::OperationEffect::StateRead);
    assert_eq!(getter.category, "Scene");
    assert_eq!(getter.native_target, "epok::SceneLibrary::screen_fade");
    assert!(getter.parameters.is_empty());
    assert_eq!(getter.outputs.len(), 1);
    assert_eq!(getter.outputs[0].value_type, schema::Type::UInt32);

    // The setter clamps into 0..255 instead of rejecting, so it has nothing to
    // report and reflects no result pin at all.
    let setter = operation("set_screen_fade");
    assert_eq!(
        setter.id, SET_SCREEN_FADE,
        "set_screen_fade identity drifted"
    );
    assert_eq!(setter.effect, schema::OperationEffect::Mutation);
    assert_eq!(setter.category, "Scene");
    assert_eq!(setter.native_target, "epok::SceneLibrary::set_screen_fade");
    assert_eq!(setter.parameters.len(), 1);
    assert_eq!(setter.parameters[0].value_type, schema::Type::UInt32);
    assert!(
        setter.outputs.is_empty(),
        "a clamping setter reports nothing"
    );
    assert!(
        setter.resource_demands.is_empty(),
        "the screen fade cooks no sidecar"
    );

    // An operation spelled like a Lua keyword is a lexer error, which would make
    // it unreachable from all three Lua modes. The check is against the lexer's
    // own list rather than a copy of it.
    for spelling in [&getter.name, &setter.name] {
        assert!(
            !crate::lua_frontend::KEYWORDS.contains(&spelling.as_str()),
            "{spelling} is a Lua keyword"
        );
    }
    std::fs::remove_dir_all(&root).unwrap();
}

/// The Q12 easing catalogue, reimplemented here rather than copied out of a run, and
/// swept over every one of the 4097 inputs. `runtime/utility.hpp` is the implementation;
/// this is the independent statement of what it has to satisfy.
///
/// Exact endpoints are load-bearing: `Tween::advance` decides completion with
/// `elapsed >= duration`, so a curve that did not return exactly 4096 would leave the
/// last frame short of `to`. The three curves that predate the catalogue are additionally
/// pinned to their original integer expressions, because `Ease` is stored in authored
/// content and their shape may not move.
#[test]
fn the_easing_catalogue_holds_its_endpoints_range_and_error_bound() {
    const ONE: i64 = 4096;
    fn powq(t: i64, n: u32) -> i64 {
        let mut acc = t;
        for _ in 1..n {
            acc = acc * t / ONE;
        }
        acc
    }
    fn q_sqrt(raw: i64) -> i64 {
        if raw <= 0 { 0 } else { (raw * ONE).isqrt() }
    }
    // Index is the serialized enumerator: Linear=0, SmoothStep=1, InQuad=2, OutQuad=3,
    // then the appended curves in declaration order.
    const KINDS: [&str; 17] = [
        "Linear",
        "SmoothStep",
        "InQuad",
        "OutQuad",
        "InOutQuad",
        "InCubic",
        "OutCubic",
        "InOutCubic",
        "InQuart",
        "OutQuart",
        "InOutQuart",
        "InQuint",
        "OutQuint",
        "InOutQuint",
        "InCirc",
        "OutCirc",
        "InOutCirc",
    ];
    fn ease(t: i64, kind: &str) -> i64 {
        let (v, rising) = (ONE - t, t * 2 <= ONE);
        match kind {
            // The three shipped curves, written exactly as the header writes them:
            // SmoothStep and OutQuad truncate a product the canonical form would not.
            "SmoothStep" => t * t / ONE * (3 * ONE - 2 * t) / ONE,
            "InQuad" => t * t / ONE,
            "OutQuad" => t * (2 * ONE - t) / ONE,
            "InCubic" => powq(t, 3),
            "InQuart" => powq(t, 4),
            "InQuint" => powq(t, 5),
            "OutCubic" => ONE - powq(v, 3),
            "OutQuart" => ONE - powq(v, 4),
            "OutQuint" => ONE - powq(v, 5),
            "InOutQuad" | "InOutCubic" | "InOutQuart" | "InOutQuint" => {
                // The doubled argument goes into the power; the halving comes after.
                // Scaling a truncated power instead costs 25.6 raw units on InOutQuint.
                let n = match kind {
                    "InOutQuad" => 2,
                    "InOutCubic" => 3,
                    "InOutQuart" => 4,
                    _ => 5,
                };
                if rising {
                    powq(2 * t, n) / 2
                } else {
                    ONE - powq(2 * v, n) / 2
                }
            }
            "InCirc" => ONE - q_sqrt(ONE - powq(t, 2)),
            "OutCirc" => q_sqrt(ONE - powq(v, 2)),
            "InOutCirc" => {
                if rising {
                    (ONE - q_sqrt(ONE - powq(2 * t, 2))) / 2
                } else {
                    (ONE + q_sqrt(ONE - powq(2 * v, 2))) / 2
                }
            }
            _ => t,
        }
    }
    fn float_reference(u: f64, kind: &str) -> f64 {
        let v = 1.0 - u;
        match kind {
            "SmoothStep" => u * u * (3.0 - 2.0 * u),
            "InQuad" => u * u,
            "OutQuad" => 1.0 - v * v,
            "InOutQuad" => {
                if u <= 0.5 {
                    2.0 * u * u
                } else {
                    1.0 - 2.0 * v * v
                }
            }
            "InCubic" => u.powi(3),
            "OutCubic" => 1.0 - v.powi(3),
            "InOutCubic" => {
                if u <= 0.5 {
                    4.0 * u.powi(3)
                } else {
                    1.0 - 4.0 * v.powi(3)
                }
            }
            "InQuart" => u.powi(4),
            "OutQuart" => 1.0 - v.powi(4),
            "InOutQuart" => {
                if u <= 0.5 {
                    8.0 * u.powi(4)
                } else {
                    1.0 - 8.0 * v.powi(4)
                }
            }
            "InQuint" => u.powi(5),
            "OutQuint" => 1.0 - v.powi(5),
            "InOutQuint" => {
                if u <= 0.5 {
                    16.0 * u.powi(5)
                } else {
                    1.0 - 16.0 * v.powi(5)
                }
            }
            "InCirc" => 1.0 - (1.0 - u * u).max(0.0).sqrt(),
            "OutCirc" => (1.0 - v * v).max(0.0).sqrt(),
            "InOutCirc" => {
                if u <= 0.5 {
                    (1.0 - (1.0 - 4.0 * u * u).max(0.0).sqrt()) / 2.0
                } else {
                    (1.0 + (1.0 - 4.0 * v * v).max(0.0).sqrt()) / 2.0
                }
            }
            _ => u,
        }
    }
    let mut table = Vec::new();
    for kind in KINDS {
        assert_eq!(ease(0, kind), 0, "{kind} does not start at 0");
        assert_eq!(ease(ONE, kind), ONE, "{kind} does not end at 4096");
        // SmoothStep truncates twice and dips one raw unit at 108 inputs. That is its
        // shipped behaviour; every appended curve truncates once and never dips.
        let tolerance = i64::from(kind == "SmoothStep");
        let (mut previous, mut worst, mut dips) = (0, 0.0f64, 0);
        for raw in 0..=ONE {
            let value = ease(raw, kind);
            assert!((0..=ONE).contains(&value), "{kind} left 0..4096 at t={raw}");
            assert!(value + tolerance >= previous, "{kind} decreased at t={raw}");
            dips += i32::from(value < previous);
            previous = value;
            let want = float_reference(raw as f64 / 4096.0, kind) * 4096.0;
            worst = worst.max((value as f64 - want).abs());
        }
        assert!(worst <= 4.0, "{kind} deviates {worst} raw units");
        assert_eq!(
            dips > 0,
            kind == "SmoothStep",
            "{kind} monotonicity changed"
        );
        table.push((kind, worst, dips));
    }
    assert_eq!(table.iter().filter(|(_, _, dips)| *dips > 0).count(), 1);
    assert_eq!(
        table.iter().find(|(k, ..)| *k == "SmoothStep").unwrap().2,
        108
    );
    // The measured table, pinned so an implementation change has to restate it.
    let worst_of =
        |name: &str| (table.iter().find(|(k, ..)| *k == name).unwrap().1 * 100.0).round() as i64;
    assert_eq!(
        KINDS.map(worst_of),
        [
            0, 385, 100, 100, 100, 189, 189, 140, 275, 275, 185, 354, 354, 192, 249, 249, 174
        ]
    );
}

/// The tween surface was already at full tri-parity; deepening it may not narrow that.
/// These assertions are about the extracted declarations, not about one provider: the
/// appended `Ease` enumerators keep their stored numbers, `GameplayTweenState` grows
/// only at the end, and every new call is one resolved operation that Blueprint, the
/// three Lua modes and C++ all lower from the same catalogue entry.
#[test]
#[ignore = "requires the pinned host extractor; stages a disposable project without building"]
fn the_tween_surface_reflects_its_appended_easings_delay_loops_and_vector_form() {
    let root = project("runtime-api-tween");
    let catalog = crate::scripts::catalog(&root).unwrap();
    let registry = crate::blueprint::native_registry(&root, &catalog).unwrap();

    let value = |id: &str, cpp: &str| {
        let record = registry
            .value_types
            .get(id)
            .unwrap_or_else(|| panic!("{cpp}"));
        assert_eq!(record.cpp_name, cpp);
        record.value_type.clone()
    };
    let members = |ty: &schema::Type| {
        ty.members()
            .iter()
            .map(|f| (f.name.clone(), f.value_type.clone()))
            .collect::<Vec<_>>()
    };
    let tween_state = value(TWEEN_STATE, "epok::GameplayTweenState");
    let names = members(&tween_state)
        .into_iter()
        .map(|(name, _)| name)
        .collect::<Vec<_>>();
    // Append-only: saved graphs address split members by dotted name, so the seven
    // that existed keep their spelling and position and the four new ones follow.
    assert_eq!(
        names,
        [
            "from",
            "to",
            "duration",
            "elapsed",
            "easing",
            "running",
            "completion_pending",
            "delay",
            "loop",
            "cycles_remaining",
            "reversed",
        ]
    );
    let member_type = |ty: &schema::Type, name: &str| ty.member_type(name).unwrap();
    assert_eq!(member_type(&tween_state, "delay"), schema::Type::Fixed);
    assert_eq!(
        member_type(&tween_state, "cycles_remaining"),
        schema::Type::UInt32
    );
    assert_eq!(member_type(&tween_state, "reversed"), schema::Type::Bool);

    // `Ease` is stored in authored content, so the four original numbers are pinned
    // and the thirteen appended curves take 4..16. Nothing is reordered or inserted.
    let schema::Type::Enum { cpp_name, variants } = member_type(&tween_state, "easing") else {
        panic!("easing is not an enum");
    };
    assert_eq!(cpp_name, "epok::Ease");
    let mut easings = variants
        .iter()
        .map(|(k, v)| (k.clone(), *v))
        .collect::<Vec<_>>();
    easings.sort_by_key(|(_, value)| *value);
    assert_eq!(
        easings,
        [
            ("Linear", 0),
            ("SmoothStep", 1),
            ("InQuad", 2),
            ("OutQuad", 3),
            ("InOutQuad", 4),
            ("InCubic", 5),
            ("OutCubic", 6),
            ("InOutCubic", 7),
            ("InQuart", 8),
            ("OutQuart", 9),
            ("InOutQuart", 10),
            ("InQuint", 11),
            ("OutQuint", 12),
            ("InOutQuint", 13),
            ("InCirc", 14),
            ("OutCirc", 15),
            ("InOutCirc", 16),
        ]
        .map(|(name, value)| (name.to_owned(), value))
    );
    let schema::Type::Enum { cpp_name, variants } = member_type(&tween_state, "loop") else {
        panic!("loop is not an enum");
    };
    assert_eq!(cpp_name, "epok::TweenLoop");
    let mut loops = variants
        .iter()
        .map(|(k, v)| (k.clone(), *v))
        .collect::<Vec<_>>();
    loops.sort_by_key(|(_, value)| *value);
    assert_eq!(
        loops,
        [("None", 0), ("Restart", 1), ("PingPong", 2)].map(|(n, v)| (n.to_owned(), v))
    );

    // A member or enumerator spelled like a Lua keyword is a lexer error, which would
    // make it unreachable from all three Lua modes. `repeat` and `until` are the two
    // that a loop plan invites, so the check is against the lexer's own list.
    let vector_state = value(VECTOR_TWEEN_STATE, "epok::GameplayVector3TweenState");
    let sample = value(VECTOR_TWEEN_SAMPLE, "epok::Vector3TweenAdvanceSample");
    let mut spellings = vec![cpp_name];
    for ty in [&tween_state, &vector_state, &sample] {
        for (name, member) in members(ty) {
            spellings.push(name);
            if let schema::Type::Enum { variants, .. } = member {
                spellings.extend(variants.into_keys());
            }
        }
    }
    for spelling in &spellings {
        assert!(
            !crate::lua_frontend::KEYWORDS.contains(&spelling.as_str()),
            "{spelling} is a Lua keyword"
        );
    }

    // The Vector3 state is two endpoints plus the scalar clock: the timing, easing and
    // loop plan are shared with the scalar form rather than duplicated.
    let vector3 = member_type(&vector_state, "from");
    assert_eq!(
        members(&vector_state),
        [
            ("from".to_owned(), vector3.clone()),
            ("to".to_owned(), vector3.clone()),
            ("timing".to_owned(), tween_state.clone()),
        ]
    );
    assert_eq!(
        members(&sample),
        [
            ("state".to_owned(), vector_state.clone()),
            ("value".to_owned(), vector3.clone()),
            ("completed".to_owned(), schema::Type::Bool),
        ]
    );
    // Both records stay inside the 32-word Lua value budget.
    assert_eq!(
        [
            tween_state.wire_words(),
            vector_state.wire_words(),
            sample.wire_words()
        ],
        [11, 17, 21]
    );

    let library = |id: &str, cpp: &str| {
        let library = registry
            .function_libraries
            .get(id)
            .unwrap_or_else(|| panic!("{cpp}"));
        assert_eq!(library.cpp_name, cpp);
        library
    };
    let scalar = library(UTILITY_LIBRARY, "epok::UtilityLibrary");
    let vector = library(UTILITY_VECTOR_LIBRARY, "epok::UtilityVectorLibrary");
    let operation = |library: &schema::FunctionLibrary, name: &str| {
        library
            .operations
            .iter()
            .find(|o| o.name == name)
            .unwrap_or_else(|| panic!("{} does not reflect {name}", library.cpp_name))
            .clone()
    };
    // The four calls that already existed keep their identities and their shapes.
    for (name, id) in [
        ("tween_start", "96f61f10-0581-43a7-8b91-f1564e3f5a02"),
        ("tween_advance", "8a083901-1054-40d6-9395-0230935d8ad1"),
        ("tween_cancel", "246c8362-87f8-479d-9b0f-352cc292ac27"),
        ("tween_value", "ee9dc203-4641-43aa-a6e4-e270536a1458"),
    ] {
        let existing = operation(scalar, name);
        assert_eq!(existing.id, id, "{name} identity drifted");
        assert_eq!(existing.effect, schema::OperationEffect::PureValue);
    }
    assert_eq!(operation(scalar, "tween_start").parameters.len(), 4);
    // Everything new is a pure value in the same category, so both halves land in one
    // script namespace and one Blueprint group with the calls they deepen.
    for (library, name, id, parameters, returns) in [
        (
            scalar,
            "tween_schedule",
            TWEEN_SCHEDULE,
            7,
            tween_state.clone(),
        ),
        (scalar, "ease", EASE, 2, schema::Type::Fixed),
        (
            vector,
            "vector_tween_start",
            VECTOR_TWEEN_START,
            4,
            vector_state.clone(),
        ),
        (
            vector,
            "vector_tween_schedule",
            VECTOR_TWEEN_SCHEDULE,
            7,
            vector_state.clone(),
        ),
        (
            vector,
            "vector_tween_advance",
            VECTOR_TWEEN_ADVANCE,
            2,
            sample.clone(),
        ),
        (
            vector,
            "vector_tween_cancel",
            VECTOR_TWEEN_CANCEL,
            1,
            vector_state.clone(),
        ),
        (
            vector,
            "vector_tween_value",
            VECTOR_TWEEN_VALUE,
            1,
            vector3.clone(),
        ),
    ] {
        let added = operation(library, name);
        assert_eq!(added.id, id, "{name} identity drifted");
        assert_eq!(added.effect, schema::OperationEffect::PureValue);
        assert_eq!(
            added.category, "Utilities",
            "{name} left the utilities group"
        );
        assert_eq!(added.native_target, format!("{}::{name}", library.cpp_name));
        assert_eq!(added.parameters.len(), parameters, "{name} arity");
        assert_eq!(added.outputs.len(), 1, "{name} returns one value");
        assert_eq!(added.outputs[0].value_type, returns, "{name} result");
        assert!(added.resource_demands.is_empty(), "{name} cooks no sidecar");
        assert!(
            added
                .parameters
                .iter()
                .all(|p| p.direction == schema::Direction::Value),
            "{name} takes a reference; the surface is pure value"
        );
    }

    // A graph that pinned an `Ease` literal before the catalogue grew stored the
    // enum type it saw then, and `Type::Enum` is structural. Appending enumerators
    // therefore has to stay assignable to the current type, or `tween_start` would
    // stop compiling in every already-authored graph. Removing one still fails.
    let schema::Type::Enum { cpp_name, variants } = member_type(&tween_state, "easing") else {
        panic!("easing is not an enum");
    };
    let saved = |names: &[&str]| schema::Type::Enum {
        cpp_name: cpp_name.clone(),
        variants: names
            .iter()
            .map(|name| ((*name).to_owned(), variants[*name]))
            .collect(),
    };
    let original = saved(&["Linear", "SmoothStep", "InQuad", "OutQuad"]);
    let easing = operation(scalar, "tween_start").parameters[3]
        .value_type
        .clone();
    assert_eq!(easing, member_type(&tween_state, "easing"));
    assert!(
        crate::blueprint_ir::assignable(&original, &easing, &registry),
        "a saved four-curve Ease literal no longer matches the reflected enum"
    );
    let renumbered = schema::Type::Enum {
        cpp_name: cpp_name.clone(),
        variants: [("Linear".to_owned(), 1i64), ("SmoothStep".to_owned(), 0)]
            .into_iter()
            .collect(),
    };
    assert!(
        !crate::blueprint_ir::assignable(&renumbered, &easing, &registry),
        "a renumbered enumerator must not silently pass"
    );
    assert!(
        !crate::blueprint_ir::assignable(&easing, &original, &registry),
        "a removed enumerator must not silently pass"
    );
    // The same structural question for the record itself: promoting a record pin to a
    // Blueprint variable stores its field list, so a graph promoted before the append
    // must still hand that variable to a call that now expects four more members.
    // A record variable compiles from its own stored type, so nothing rewrites the
    // asset; only the flow into a current-typed pin has to keep type-checking.
    let schema::Type::Record { cpp_name, fields } = tween_state.clone() else {
        panic!("the tween state is not a record");
    };
    let promoted = schema::Type::Record {
        cpp_name: cpp_name.clone(),
        fields: fields[..7].to_vec(),
    };
    assert!(
        crate::blueprint_ir::assignable(&promoted, &tween_state, &registry),
        "a record variable promoted before the append no longer matches its pin"
    );
    assert!(
        crate::blueprint_ir::assignable(&tween_state, &promoted, &registry),
        "the append-only rule has to hold in both directions"
    );
    let reordered = schema::Type::Record {
        cpp_name,
        fields: fields[..7].iter().rev().cloned().collect(),
    };
    assert!(
        !crate::blueprint_ir::assignable(&reordered, &tween_state, &registry),
        "a reordered member must not silently pass"
    );
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
